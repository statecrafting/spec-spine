#!/bin/sh
# spec-spine installer: curl -fsSL https://raw.githubusercontent.com/stagecraft-ing/spec-spine/main/install.sh | sh
#
# Detects your platform/arch, downloads the matching release archive and its
# .sha256 sidecar from GitHub Releases, verifies the checksum, and drops the
# `spec-spine` binary on your PATH.
#
# Environment overrides:
#   SPEC_SPINE_VERSION            release tag to install (default: latest), e.g. v0.1.0
#   SPEC_SPINE_BIN_DIR            install dir (default: ~/.local/bin, or /usr/local/bin if writable & in PATH)
#   SPEC_SPINE_REQUIRE_ATTESTATION=1  hard-fail if the build-provenance attestation cannot be verified
#   SPEC_SPINE_SKIP_ATTESTATION=1     skip the provenance check entirely (checksum still enforced)
#
# The .sha256 sidecar proves integrity; provenance verification (via `gh`, spec
# 021) proves authenticity. musl-based Linux (e.g. Alpine) is refused with a
# pointer to `cargo install`, since the prebuilt Linux binaries are glibc-only.
#
# Windows: use the .zip from the Releases page (this script targets macOS/Linux).

set -eu

REPO="stagecraft-ing/spec-spine"
BIN="spec-spine"

say()  { printf 'spec-spine: %s\n' "$1" >&2; }
die()  { printf 'spec-spine: error: %s\n' "$1" >&2; exit 1; }
have() { command -v "$1" >/dev/null 2>&1; }

# --- pick a downloader -------------------------------------------------------
if have curl; then
  dl()      { curl -fsSL "$1" -o "$2"; }
  dl_stdout(){ curl -fsSL "$1"; }
elif have wget; then
  dl()      { wget -qO "$2" "$1"; }
  dl_stdout(){ wget -qO - "$1"; }
else
  die "need curl or wget on PATH"
fi

# --- detect platform / arch --------------------------------------------------
os="$(uname -s)"
arch="$(uname -m)"
case "$os" in
  Darwin) plat="apple-darwin" ;;
  Linux)
    plat="unknown-linux-gnu"
    # The prebuilt Linux archives are glibc-only. On musl (Alpine, etc.) a glibc
    # binary fails at runtime with a cryptic dynamic-loader error, so refuse up
    # front with an actionable message (parity with the npm/py shims).
    if { ldd --version 2>&1 | grep -qi musl; } \
       || [ -e /lib/ld-musl-x86_64.so.1 ] || [ -e /lib/ld-musl-aarch64.so.1 ]; then
      die "musl libc detected (e.g. Alpine): the prebuilt binaries are glibc-only. Install with 'cargo install spec-spine-cli' (needs the Rust toolchain), or use a glibc-based distro."
    fi
    ;;
  *)      die "unsupported OS '$os' (use the .zip from the Releases page on Windows)" ;;
esac
case "$arch" in
  x86_64|amd64)  cpu="x86_64" ;;
  arm64|aarch64) cpu="aarch64" ;;
  *)             die "unsupported architecture '$arch'" ;;
esac
triple="${cpu}-${plat}"

# --- resolve version ---------------------------------------------------------
tag="${SPEC_SPINE_VERSION:-latest}"
if [ "$tag" = "latest" ]; then
  say "resolving latest release…"
  tag="$(dl_stdout "https://api.github.com/repos/${REPO}/releases/latest" \
        | grep -m1 '"tag_name"' \
        | sed -E 's/.*"tag_name"[ ]*:[ ]*"([^"]+)".*/\1/')"
  [ -n "$tag" ] || die "could not resolve the latest release tag (set SPEC_SPINE_VERSION)"
fi

archive="${BIN}-${tag}-${triple}.tar.gz"
base_url="https://github.com/${REPO}/releases/download/${tag}"
say "installing ${BIN} ${tag} for ${triple}"

# --- download archive + checksum ---------------------------------------------
tmp="$(mktemp -d "${TMPDIR:-/tmp}/spec-spine.XXXXXX")"
trap 'rm -rf "$tmp"' EXIT INT TERM

dl "${base_url}/${archive}"         "${tmp}/${archive}" \
  || die "download failed: ${base_url}/${archive}"
dl "${base_url}/${archive}.sha256"  "${tmp}/${archive}.sha256" \
  || die "checksum download failed: ${base_url}/${archive}.sha256"

# --- verify checksum ---------------------------------------------------------
expected="$(awk '{print $1}' "${tmp}/${archive}.sha256")"
[ -n "$expected" ] || die "empty checksum sidecar"
if have sha256sum;  then actual="$(sha256sum "${tmp}/${archive}" | awk '{print $1}')"
elif have shasum;   then actual="$(shasum -a 256 "${tmp}/${archive}" | awk '{print $1}')"
elif have openssl;  then actual="$(openssl dgst -sha256 "${tmp}/${archive}" | awk '{print $NF}')"
else die "need sha256sum, shasum, or openssl to verify the download"; fi
[ "$expected" = "$actual" ] || die "checksum mismatch (expected ${expected}, got ${actual})"
say "checksum verified"

# --- verify provenance attestation (authenticity, not just integrity) --------
# The .sha256 sidecar is fetched from the same release as the archive, so it
# proves integrity but NOT authenticity: a rewritten release ships a matching
# sidecar. GitHub build-provenance attestations (spec 021) close that gap. Use
# `gh attestation verify` when available. Best-effort by default (many curl|sh
# users have no authenticated `gh`); set SPEC_SPINE_REQUIRE_ATTESTATION=1 to
# make an unverifiable download a hard failure.
if [ "${SPEC_SPINE_SKIP_ATTESTATION:-0}" = "1" ]; then
  say "provenance attestation check skipped (SPEC_SPINE_SKIP_ATTESTATION=1)"
elif have gh && gh attestation verify "${tmp}/${archive}" --repo "${REPO}" >/dev/null 2>&1; then
  say "provenance attestation verified (spec 021)"
elif [ "${SPEC_SPINE_REQUIRE_ATTESTATION:-0}" = "1" ]; then
  die "provenance attestation could NOT be verified and SPEC_SPINE_REQUIRE_ATTESTATION=1 is set (rewritten release, or 'gh' missing/unauthenticated)"
else
  say "note: provenance attestation not verified (install 'gh' and authenticate for authenticity checks; checksum was verified). Set SPEC_SPINE_REQUIRE_ATTESTATION=1 to enforce."
fi

# --- extract -----------------------------------------------------------------
tar -C "$tmp" -xzf "${tmp}/${archive}" || die "extract failed"
[ -f "${tmp}/${BIN}" ] || die "archive did not contain ${BIN}"
chmod +x "${tmp}/${BIN}"

# --- choose an install dir ---------------------------------------------------
bindir="${SPEC_SPINE_BIN_DIR:-}"
if [ -z "$bindir" ]; then
  if [ -w /usr/local/bin ] && printf '%s' "$PATH" | tr ':' '\n' | grep -qx /usr/local/bin; then
    bindir="/usr/local/bin"
  else
    bindir="${HOME}/.local/bin"
  fi
fi
mkdir -p "$bindir" || die "could not create install dir ${bindir}"
mv "${tmp}/${BIN}" "${bindir}/${BIN}" || die "could not install to ${bindir} (try sudo, or set SPEC_SPINE_BIN_DIR)"

say "installed ${bindir}/${BIN}"
if printf '%s' "$PATH" | tr ':' '\n' | grep -qx "$bindir"; then
  say "run: ${BIN} --version"
else
  say "NOTE: ${bindir} is not on your PATH. Add it, e.g.:"
  printf '  export PATH="%s:$PATH"\n' "$bindir" >&2
fi
