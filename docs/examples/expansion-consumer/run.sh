#!/usr/bin/env bash
# Run the expansion-wave consumer against THIS checkout's packaged library.
#
# Local source/package verification only: it packages spec-spine-types and
# spec-spine-core with `cargo package`, unpacks them into a scratch directory,
# builds the consumer there against them (off the workspace lockfile), and
# drives it with the CLI built from the same tree. Nothing is fetched from a
# registry for spec-spine itself; serde_json comes from crates.io or the Cargo
# cache. It is NOT registry-backed release verification.
#
# Usage: docs/examples/expansion-consumer/run.sh [--keep]
set -euo pipefail

KEEP=0
[ "${1:-}" = "--keep" ] && KEEP=1
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO"
VERSION="$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -1)"
COMMIT="$(git rev-parse HEAD)"
ALLOW=()
if [ -n "$(git status --porcelain)" ]; then
  ALLOW=(--allow-dirty)
  echo "note: working tree dirty; the packages describe the tree, not $COMMIT"
fi

SCRATCH="$(mktemp -d "${HOME}/.cache/spec-spine-consumer.XXXXXX" 2>/dev/null || mktemp -d)"
cleanup() { if [ "$KEEP" = 0 ]; then rm -rf "$SCRATCH"; else echo "kept: $SCRATCH"; fi; }
trap cleanup EXIT

echo "=== expansion consumer: spec-spine $VERSION at $COMMIT"
cargo build --release --locked -p spec-spine-cli
BIN="$REPO/target/release/spec-spine"
"$BIN" --version

PKG="$SCRATCH/target"
cargo package --locked -p spec-spine-types -p spec-spine-core --target-dir "$PKG" "${ALLOW[@]}"
mkdir -p "$SCRATCH/vendor"
for c in types core; do
  tar xzf "$PKG/package/spec-spine-$c-$VERSION.crate" -C "$SCRATCH/vendor"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$PKG/package/spec-spine-$c-$VERSION.crate"
  else
    shasum -a 256 "$PKG/package/spec-spine-$c-$VERSION.crate"
  fi
done

cp -R "$REPO/docs/examples/expansion-consumer" "$SCRATCH/consumer"
sed -i.bak \
  -e "s#PACKAGED_CORE#$SCRATCH/vendor/spec-spine-core-$VERSION#" \
  -e "s#PACKAGED_TYPES#$SCRATCH/vendor/spec-spine-types-$VERSION#" \
  "$SCRATCH/consumer/Cargo.toml"
rm -f "$SCRATCH/consumer/Cargo.toml.bak" "$SCRATCH/consumer/run.sh"

# The fixture set the consumer replays is the one INSIDE the packaged crate.
FIXTURES="$SCRATCH/vendor/spec-spine-core-$VERSION/fixtures/verifier"
test -f "$FIXTURES/index.json"

mkdir -p "$SCRATCH/work"
cargo run --quiet --release --manifest-path "$SCRATCH/consumer/Cargo.toml" \
  --target-dir "$SCRATCH/consumer-target" -- "$BIN" "$FIXTURES" "$SCRATCH/work"
echo "=== expansion consumer: all contracts held"
