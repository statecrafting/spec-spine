#!/bin/sh
# Spec: specs/124-the-expansion-line-carries-its-own-version/spec.md
#
# reader-identity.sh BINARY [CHECKOUT]
#
# What one spec-spine executable is, from evidence rather than from its version
# string (spec 124 §3.4). Two builds from different sources can answer the same
# `--version`; the 0.22.0 candidate and `main` did. This prints:
#
#   executable        the absolute path
#   sha256            the executable's digest
#   answers           what it says to --version
#   checkout          CHECKOUT's revision, and whether its tree is clean
#   built-from        whether BINARY is CHECKOUT's own in-tree build and no
#                     input it was compiled from is newer than it (the
#                     comparison spec 123 makes), or why that cannot be said
#   <axis> schema     each schema axis as this executable emits it, measured by
#                     running it on a one-spec scratch corpus: nothing is read
#                     from the source, so a stale build cannot report the
#                     source's numbers
#
# Reads only. The scratch corpus lives in a temporary directory and is removed.
# Exit 0 when every line was measured, 1 when any was not (the line says why),
# 3 on a usage error.

set -u

[ $# -ge 1 ] && [ $# -le 2 ] || { echo "usage: reader-identity.sh BINARY [CHECKOUT]" >&2; exit 3; }
bin=$1
checkout=${2:-}
[ -x "$bin" ] || { echo "reader-identity: '$bin' is not an executable" >&2; exit 3; }
case "$bin" in /*) ;; *) bin="$(cd "$(dirname "$bin")" && pwd)/$(basename "$bin")" ;; esac

unmeasured=0
say() { printf '%-18s %s\n' "$1" "$2"; }
miss() { say "$1" "NOT MEASURED: $2"; unmeasured=1; }

say executable "$bin"
if command -v shasum >/dev/null 2>&1; then
  say sha256 "$(shasum -a 256 "$bin" | awk '{print $1}')"
elif command -v sha256sum >/dev/null 2>&1; then
  say sha256 "$(sha256sum "$bin" | awk '{print $1}')"
else
  miss sha256 "neither shasum nor sha256sum is available"
fi
say answers "$("$bin" --version 2>/dev/null || echo '(no answer to --version)')"

if [ -n "$checkout" ]; then
  checkout="$(cd "$checkout" && pwd)"
  rev=$(git -C "$checkout" rev-parse HEAD 2>/dev/null) || rev=''
  if [ -z "$rev" ]; then
    miss checkout "$checkout is not a git checkout"
  else
    if [ -z "$(git -C "$checkout" status --porcelain --untracked-files=no 2>/dev/null)" ]; then
      say checkout "$checkout at $rev (tracked tree clean)"
    else
      say checkout "$checkout at $rev (tracked tree MODIFIED)"
    fi
    if [ "$bin" != "$checkout/target/release/spec-spine" ]; then
      say built-from "not stated: $bin is not $checkout's in-tree build"
    elif [ ! -f "$bin.d" ]; then
      miss built-from "no dep-info beside the build, so its inputs are unknown"
    else
      newer=$(sed 's/^[^:]*://' "$bin.d" | tr ' ' '\n' | while IFS= read -r f; do
        [ "${f#"$checkout"/}" != "$f" ] && [ -f "$f" ] || continue
        if [ "$f" -nt "$bin" ]; then printf '%s\n' "${f#"$checkout"/}"; break; fi
      done)
      if [ -n "$newer" ]; then
        say built-from "NO: $newer is newer than the build; rebuild before recording it"
        unmeasured=1
      else
        if [ -z "$(git -C "$checkout" status --porcelain --untracked-files=no 2>/dev/null)" ]; then
          say built-from "yes: no compiled input is newer than the build, at $rev"
        else
          say built-from "yes, from a MODIFIED tree at $rev: no compiled input is newer than the build"
        fi
      fi
    fi
  fi
fi

# The schema axes, as this executable emits them.
scratch=$(mktemp -d 2>/dev/null || mktemp -d -t reader-identity)
trap 'rm -rf "$scratch"' EXIT INT TERM
mkdir -p "$scratch/specs/000-probe" "$scratch/src"
printf 'pub fn probe() {}\n' > "$scratch/src/lib.rs"
cat > "$scratch/specs/000-probe/spec.md" <<'EOF'
---
id: "000-probe"
title: "Probe"
status: approved
created: "2026-01-01"
summary: "A one-spec corpus a reader is asked to emit its schemas over."
establishes:
  - "src/lib.rs"
---
# 000-probe
## Body
EOF
field() { python3 -c 'import json,sys; print(json.load(open(sys.argv[1])).get(sys.argv[2], ""))' "$1" "$2" 2>/dev/null; }
if "$bin" --repo "$scratch" compile >/dev/null 2>&1 && "$bin" --repo "$scratch" index >/dev/null 2>&1; then
  derived=$("$bin" --repo "$scratch" config show --json 2>/dev/null \
    | python3 -c 'import json,sys; print(json.load(sys.stdin)["layout"]["derived_dir"])' 2>/dev/null)
  shard=$(find "$scratch/$derived" -path '*spec-registry/by-spec/000-probe.json' 2>/dev/null | head -1)
  ishard=$(find "$scratch/$derived" -path '*codebase-index/by-spec/000-probe.json' 2>/dev/null | head -1)
  v=$([ -n "$shard" ] && field "$shard" specVersion); [ -n "$v" ] && say "registry schema" "$v" || miss "registry schema" "no registry shard emitted"
  v=$([ -n "$ishard" ] && field "$ishard" schemaVersion); [ -n "$v" ] && say "index schema" "$v" || miss "index schema" "no index shard emitted"
else
  miss "registry schema" "compile or index refused the scratch corpus"
  miss "index schema" "compile or index refused the scratch corpus"
fi
"$bin" --repo "$scratch" registry plan --json > "$scratch/read.json" 2>/dev/null
v=$(field "$scratch/read.json" schemaVersion); [ -n "$v" ] && say "read schema" "$v" || miss "read schema" "registry plan --json gave no schemaVersion"
"$bin" --repo "$scratch" check --json > "$scratch/verdict.json" 2>/dev/null
v=$(field "$scratch/verdict.json" schemaVersion); [ -n "$v" ] && say "verdict schema" "$v" || miss "verdict schema" "check --json gave no envelope"

exit "$unmeasured"
