#!/bin/sh
# Spec: specs/149-the-installer-is-tested/spec.md
#
# test-install.sh: run install.sh offline against a fixture release (spec 149
# 3.1). The network is a `curl` stub that serves the fixture directory and
# fails every other URL; the platform is the host's `uname`, or a `uname` stub
# in the two cases that need an unsupported one. The installer runs under
# `env -i` with a PATH of the stub directory and a toolbox of symlinks to the
# host tools it needs, so no host `gh`, `wget` or SPEC_SPINE_* variable can
# reach it (D-1).
#
# usage: sh scripts/test-install.sh
#   SPEC_SPINE_INSTALLER  installer under test (default: install.sh beside
#                         this script's directory); a mutation harness points
#                         it at an edited copy (D-2)
#
# Prints one line per case and ends with `install.sh: N of 7 cases passed`.
# Exits 0 only when all seven pass, 1 otherwise.

set -u

here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
installer=${SPEC_SPINE_INSTALLER:-$here/../install.sh}
[ -f "$installer" ] || { echo "test-install.sh: no installer at $installer" >&2; exit 1; }
host_sh=$(command -v sh)

# --- the host's triple, by the installer's own mapping -----------------------
host_os=$(uname -s)
host_arch=$(uname -m)
case "$host_os" in
  Darwin) plat="apple-darwin" ;;
  Linux)  plat="unknown-linux-gnu" ;;
  *) echo "test-install.sh: host OS '$host_os' is not one install.sh supports" >&2; exit 1 ;;
esac
case "$host_arch" in
  x86_64|amd64)  cpu="x86_64" ;;
  arm64|aarch64) cpu="aarch64" ;;
  *) echo "test-install.sh: host architecture '$host_arch' is not one install.sh supports" >&2; exit 1 ;;
esac
triple="${cpu}-${plat}"

work=$(mktemp -d "${TMPDIR:-/tmp}/ss-test-install.XXXXXX") || exit 1
trap 'rm -rf "$work"' EXIT
trap 'exit 1' INT TERM

sha256() {
  if command -v sha256sum >/dev/null 2>&1; then sha256sum "$1" | awk '{print $1}'
  elif command -v shasum >/dev/null 2>&1; then shasum -a 256 "$1" | awk '{print $1}'
  else openssl dgst -sha256 "$1" | awk '{print $NF}'; fi
}

# --- toolbox: the host tools install.sh uses, and nothing else ---------------
toolbox="$work/toolbox"
mkdir -p "$toolbox"
for t in sh cat grep sed awk tr mktemp tar gzip chmod mkdir mv rm uname \
         sha256sum shasum openssl perl ldd env; do
  p=$(command -v "$t" 2>/dev/null) || continue
  case "$p" in /*) ln -s "$p" "$toolbox/$t" ;; esac
done
for t in sh cat grep sed awk tr mktemp tar gzip chmod mkdir mv rm uname; do
  [ -e "$toolbox/$t" ] || { echo "test-install.sh: host has no '$t'" >&2; exit 1; }
done

# --- the fixture release -----------------------------------------------------
fixture="$work/fixture"
mkdir -p "$fixture/releases"

# release <tag> <version-or-empty> [bad]: one archive and its sidecar. An empty
# version builds an archive without the binary; `bad` writes a sidecar that
# does not match the archive.
release() {
  dir="$fixture/releases/$1"
  name="spec-spine-$1-$triple.tar.gz"
  stage="$work/stage-$1"
  mkdir -p "$dir" "$stage"
  if [ -n "$2" ]; then
    printf '#!/bin/sh\necho "spec-spine %s"\n' "$2" > "$stage/spec-spine"
    chmod +x "$stage/spec-spine"
    tar -C "$stage" -czf "$dir/$name" spec-spine
  else
    echo "no binary here" > "$stage/README"
    tar -C "$stage" -czf "$dir/$name" README
  fi
  if [ "${3:-}" = bad ]; then
    printf 'decoy\n' > "$stage/decoy"
    printf '%s  %s\n' "$(sha256 "$stage/decoy")" "$name" > "$dir/$name.sha256"
  else
    printf '%s  %s\n' "$(sha256 "$dir/$name")" "$name" > "$dir/$name.sha256"
  fi
}
release v9.9.1 9.9.1
release v9.9.2 9.9.2 bad
release v9.9.3 ""
release v9.9.7 9.9.7
printf '{"tag_name": "v9.9.7"}\n' > "$fixture/latest.json"

stubs="$work/stubs"
mkdir -p "$stubs"
cat > "$stubs/curl" <<EOF
#!/bin/sh
# curl stub: serves the fixture release at $fixture, fails any other URL.
url=""; out=""
while [ \$# -gt 0 ]; do
  case "\$1" in
    -o) out="\${2:-}"; shift; [ \$# -gt 0 ] && shift; continue ;;
    http://*|https://*) url="\$1" ;;
  esac
  shift
done
case "\$url" in
  *..*) src="" ;;
  https://api.github.com/repos/statecrafting/spec-spine/releases/latest)
    src="$fixture/latest.json" ;;
  https://github.com/statecrafting/spec-spine/releases/download/*/*)
    src="$fixture/releases/\${url#https://github.com/statecrafting/spec-spine/releases/download/}" ;;
  *) src="" ;;
esac
if [ -z "\$src" ] || [ ! -f "\$src" ]; then
  echo "curl: (22) The requested URL returned error: 404 (\$url)" >&2
  exit 22
fi
if [ -n "\$out" ]; then cat "\$src" > "\$out"; else cat "\$src"; fi
EOF
chmod +x "$stubs/curl"

# --- running one case --------------------------------------------------------
passed=0
n=0

# run_installer <case dir> <uname stub dir or empty> <VAR=value>...: runs the
# installer in a clean environment. Sets rc; stderr lands in <case dir>/err.
run_installer() {
  cdir=$1; ustub=$2; shift 2
  mkdir -p "$cdir/tmp" "$cdir/home"
  path="$stubs:$toolbox"
  [ -n "$ustub" ] && path="$ustub:$path"
  env -i PATH="$path" HOME="$cdir/home" TMPDIR="$cdir/tmp" \
    SPEC_SPINE_BIN_DIR="$cdir/bin" "$@" \
    "$host_sh" "$installer" > "$cdir/out" 2> "$cdir/err" < /dev/null
  rc=$?
}

# uname_stub <dir> <os> <arch>
uname_stub() {
  mkdir -p "$1"
  cat > "$1/uname" <<EOF
#!/bin/sh
case "\${1:-}" in
  -s) echo '$2' ;;
  -m) echo '$3' ;;
  *)  echo '$2' ;;
esac
EOF
  chmod +x "$1/uname"
}

# report <name> <failure reason or empty>
report() {
  n=$((n + 1))
  if [ -z "$2" ]; then
    passed=$((passed + 1))
    echo "case $n ($1): passed"
  else
    echo "case $n ($1): FAILED: $2"
    sed 's/^/    stderr: /' "$work/case$n/err" 2>/dev/null
  fi
}

installed_nothing() { [ ! -e "$1/bin/spec-spine" ]; }

# 1. a matching checksum installs the binary, and it runs.
c="$work/case1"
run_installer "$c" "" SPEC_SPINE_VERSION=v9.9.1 SPEC_SPINE_SKIP_ATTESTATION=1
why=""
if [ "$rc" -ne 0 ]; then why="exit $rc, expected 0"
elif [ ! -x "$c/bin/spec-spine" ]; then why="no executable at SPEC_SPINE_BIN_DIR/spec-spine"
elif [ "$("$c/bin/spec-spine" --version 2>&1)" != "spec-spine 9.9.1" ]; then why="installed binary did not print 'spec-spine 9.9.1'"
fi
report "matching checksum installs a binary that runs" "$why"

# 2. a sidecar that does not match refuses and installs nothing.
c="$work/case2"
run_installer "$c" "" SPEC_SPINE_VERSION=v9.9.2 SPEC_SPINE_SKIP_ATTESTATION=1
why=""
if [ "$rc" -ne 1 ]; then why="exit $rc, expected 1"
elif ! grep -q "checksum mismatch" "$c/err"; then why="no 'checksum mismatch' on stderr"
elif ! installed_nothing "$c"; then why="a binary was installed"
fi
report "mismatched sidecar refuses" "$why"

# 3. an archive without the binary, checksum matching, refuses at extraction.
c="$work/case3"
run_installer "$c" "" SPEC_SPINE_VERSION=v9.9.3 SPEC_SPINE_SKIP_ATTESTATION=1
why=""
if [ "$rc" -ne 1 ]; then why="exit $rc, expected 1"
elif ! grep -q "checksum verified" "$c/err"; then why="the installer did not get past the checksum"
elif ! grep -q "archive did not contain spec-spine" "$c/err"; then why="no 'archive did not contain spec-spine' on stderr"
elif ! installed_nothing "$c"; then why="a binary was installed"
fi
report "archive without the binary refuses" "$why"

# 4. an unsupported OS refuses, naming it.
c="$work/case4"
uname_stub "$c/uname" Plan9 "$host_arch"
run_installer "$c" "$c/uname" SPEC_SPINE_VERSION=v9.9.1 SPEC_SPINE_SKIP_ATTESTATION=1
why=""
if [ "$rc" -ne 1 ]; then why="exit $rc, expected 1"
elif ! grep -q "unsupported OS 'Plan9'" "$c/err"; then why="stderr does not name the OS 'Plan9'"
elif ! installed_nothing "$c"; then why="a binary was installed"
fi
report "unsupported OS refuses" "$why"

# 5. an unsupported architecture refuses, naming it.
c="$work/case5"
uname_stub "$c/uname" "$host_os" sparc64
run_installer "$c" "$c/uname" SPEC_SPINE_VERSION=v9.9.1 SPEC_SPINE_SKIP_ATTESTATION=1
why=""
if [ "$rc" -ne 1 ]; then why="exit $rc, expected 1"
elif ! grep -q "unsupported architecture 'sparc64'" "$c/err"; then why="stderr does not name the architecture 'sparc64'"
elif ! installed_nothing "$c"; then why="a binary was installed"
fi
report "unsupported architecture refuses" "$why"

# 6. required attestation with no gh on PATH refuses and installs nothing.
c="$work/case6"
why=""
if [ -e "$toolbox/gh" ] || [ -e "$stubs/gh" ]; then why="the installer's PATH holds a gh"
else
  run_installer "$c" "" SPEC_SPINE_VERSION=v9.9.1 SPEC_SPINE_REQUIRE_ATTESTATION=1
  if [ "$rc" -ne 1 ]; then why="exit $rc, expected 1"
  elif ! grep -q "provenance attestation could NOT be verified" "$c/err"; then why="stderr does not report the unverified attestation"
  elif ! installed_nothing "$c"; then why="a binary was installed"
  fi
fi
report "required attestation without gh refuses" "$why"

# 7. latest resolves the tag from the releases API and installs that tag.
c="$work/case7"
run_installer "$c" "" SPEC_SPINE_VERSION=latest SPEC_SPINE_SKIP_ATTESTATION=1
why=""
if [ "$rc" -ne 0 ]; then why="exit $rc, expected 0"
elif ! grep -q "installing spec-spine v9.9.7 for $triple" "$c/err"; then why="the installer did not resolve latest to v9.9.7"
elif [ "$("$c/bin/spec-spine" --version 2>&1)" != "spec-spine 9.9.7" ]; then why="installed binary is not v9.9.7's"
fi
report "latest resolves the tag from the releases API" "$why"

echo "install.sh: $passed of $n cases passed"
[ "$passed" -eq 7 ] && [ "$n" -eq 7 ]
