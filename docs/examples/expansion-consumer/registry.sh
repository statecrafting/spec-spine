#!/usr/bin/env bash
# Registry-backed consumer qualification of a PUBLISHED spec-spine version.
#
# Everything comes from the public registries, into fresh caches in a new
# scratch directory: no path, git or patch dependency, no workspace, and no
# warm Cargo, npm or uv cache. The checkout this script sits in supplies only
# the consumer's source and `scripts/reader-identity.sh`.
#
# Usage:
#   docs/examples/expansion-consumer/registry.sh VERSION [PREVIOUS]
#       Qualify VERSION. With PREVIOUS (an older published version), also
#       prove a PREVIOUS reader refuses a corpus that requires VERSION.
#   docs/examples/expansion-consumer/registry.sh --control VERSION ID[,ID...]
#       Negative control: run the consumer against VERSION once per ID with
#       CONSUMER_ONLY=ID, and pass only if EVERY run fails. Point it at a
#       release that lacks those contracts. (112 guarantees behavior every
#       published release already has, so no release can serve as its control.)
#
# Prints `=== <step> exit=<n>` per step; exits 0 only if every step held.
set -uo pipefail

CONTROL=0
if [ "${1:-}" = "--control" ]; then CONTROL=1; shift; fi
V="${1:?usage: registry.sh VERSION [PREVIOUS] | --control VERSION IDS}"
ARG2="${2:-}"
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
mkdir -p "$HOME/.cache"
T="$(mktemp -d "$HOME/.cache/spec-spine-registry-$V.XXXXXX")"
export CARGO_HOME="$T/cargo-home"
export RUSTUP_HOME="${RUSTUP_HOME:-$HOME/.rustup}"
export npm_config_cache="$T/npm-cache"
export UV_CACHE_DIR="$T/uv-cache"
export UV_TOOL_DIR="$T/uv-tools"
export PIP_NO_CACHE_DIR=1
unset CARGO_TARGET_DIR RUSTFLAGS
mkdir -p "$CARGO_HOME"
rc_all=0
step() {
  local name=$1; shift
  echo "=== $name"
  "$@"; local rc=$?
  echo "=== $name exit=$rc"
  [ $rc -eq 0 ] || rc_all=1
  return $rc
}
echo "scratch $T"
echo "consumer source: $REPO at $(git -C "$REPO" rev-parse HEAD)"

install_cli() { # version root
  cargo install spec-spine-cli --version "=$1" --locked --root "$2" --quiet
}

# The library consumer, built against spec-spine-core =$1 from crates.io.
build_lib() { # version dir
  mkdir -p "$2"
  cp -R "$REPO/docs/examples/expansion-consumer/src" "$2/src"
  cat > "$2/Cargo.toml" <<TOML
[package]
name = "expansion-consumer-registry"
version = "0.0.0"
edition = "2024"
publish = false

[dependencies]
spec-spine-core = "=$1"
serde_json = "1"

[workspace]
TOML
  cargo build --quiet --release --manifest-path "$2/Cargo.toml" --target-dir "$2/target"
}

fixtures_of() { # version
  ls -d "$CARGO_HOME"/registry/src/*/spec-spine-core-"$1"/fixtures/verifier 2>/dev/null | head -1
}

if [ "$CONTROL" = 1 ]; then
  IDS="${ARG2:?--control needs a comma-separated list of contract ids}"
  step cli-install install_cli "$V" "$T/cli" || exit 1
  step lib-build build_lib "$V" "$T/lib" || exit 1
  FIX="$(fixtures_of "$V")"
  for id in ${IDS//,/ }; do
    rm -rf "$T/work-$id"; mkdir -p "$T/work-$id"
    CONSUMER_ONLY="$id" "$T/lib/target/release/expansion-consumer-registry" \
      "$T/cli/bin/spec-spine" "$FIX" "$T/work-$id" > "$T/control-$id.log" 2>&1
    rc=$?
    echo "--- control $id against $V: consumer exit $rc; last line: $(tail -1 "$T/control-$id.log")"
    step "control-$id-fails" test "$rc" -ne 0
  done
  echo "ALL=$rc_all scratch=$T"
  exit $rc_all
fi

# 1. The CLI from crates.io.
step cli-install install_cli "$V" "$T/cli"
BIN="$T/cli/bin/spec-spine"
step cli-version sh -c "\"$BIN\" --version | grep -qx 'spec-spine $V'"
step cli-identity "$REPO/scripts/reader-identity.sh" "$BIN"

# 2. The library from crates.io, driven by that CLI, replaying the fixture set
#    shipped INSIDE the registry crate, which must name this exact version.
step lib-build build_lib "$V" "$T/lib"
FIX="$(fixtures_of "$V")"
step lib-fixtures-present test -f "$FIX/index.json"
step lib-fixtures-bound sh -c "n=\$(grep -l '\"toolVersion\": \"$V\"' '$FIX'/*/case.json | wc -l); m=\$(ls -d '$FIX'/*/ | wc -l); echo \"fixtures naming $V: \$n of \$m cases\"; test \"\$n\" -ge 1"
mkdir -p "$T/lib-work"
step lib-contracts "$T/lib/target/release/expansion-consumer-registry" "$BIN" "$FIX" "$T/lib-work"
step lib-resolved sh -c "grep -A2 'name = \"spec-spine-core\"' '$T/lib/Cargo.lock' | grep -q 'registry+https://github.com/rust-lang/crates.io-index' && grep -A1 'name = \"spec-spine-types\"' '$T/lib/Cargo.lock' | grep -q 'version = \"$V\"' && ! grep -qE 'path\\+|git\\+' '$T/lib/Cargo.lock'"

# 3. Statecraft's shape: default-features = false, the scaffold producer only.
mkdir -p "$T/sc/src"
cat > "$T/sc/Cargo.toml" <<TOML
[package]
name = "statecraft-shape-consumer"
version = "0.0.0"
edition = "2021"
publish = false

[dependencies]
spec-spine-core = { version = "=$V", default-features = false }
serde_json = "1"

[workspace]
TOML
cat > "$T/sc/src/main.rs" <<'RS'
fn main() {
    let a = spec_spine_core::scaffold_init_json("{}").expect("scaffold");
    let b = spec_spine_core::scaffold_init_json("{}").expect("scaffold");
    assert_eq!(a, b, "pure and deterministic");
    let v: serde_json::Value = serde_json::from_str(&a).unwrap();
    let files = v["files"].as_array().expect("files");
    let paths: Vec<&str> = files.iter().map(|f| f["relPath"].as_str().unwrap()).collect();
    for forbidden in ["AGENTS.md", "CLAUDE.md", "Makefile"] {
        assert!(!paths.contains(&forbidden), "{forbidden} is not a governance file");
    }
    assert!(!paths.iter().any(|p| p.starts_with(".claude/") || p.starts_with(".github/")));
    assert!(paths.contains(&"spec-spine.toml"), "{paths:?}");
    // Negative: a camelCase config key is refused, not ignored.
    assert!(spec_spine_core::scaffold_init_json(r#"{"layout":{"derivedDir":".x"}}"#).is_err());
    println!("ok: scaffold_init_json default-features=false, {} files", paths.len());
}
RS
step lib-statecraft-shape cargo run --quiet --release --manifest-path "$T/sc/Cargo.toml" --target-dir "$T/sc/target"
step lib-no-tree-sitter sh -c "! grep -q 'name = \"tree-sitter' '$T/sc/Cargo.lock'"

# 4. An insufficient reader refuses by name (exit 3), never judges.
corpus_requiring() { # dir version
  mkdir -p "$1/specs/001-a"
  printf '[meta]\nrequired_version = ">=%s"\n' "$2" > "$1/spec-spine.toml"
  printf -- '---\nid: "001-a"\ntitle: "A"\nstatus: draft\ncreated: "2026-09-23"\nsummary: "s"\n---\n# A\n' > "$1/specs/001-a/spec.md"
}
corpus_requiring "$T/neg" 99.0.0
step cli-floor-refuses-newer sh -c "cd '$T/neg' && \"$BIN\" compile >/dev/null 2>&1; test \$? -eq 3"
corpus_requiring "$T/pos" "$V"
step cli-floor-admits-own sh -c "cd '$T/pos' && \"$BIN\" compile >/dev/null"
if [ -n "$ARG2" ]; then
  step prev-install install_cli "$ARG2" "$T/prev"
  step prev-refuses sh -c "cd '$T/pos' && '$T/prev/bin/spec-spine' compile >'$T/prev.out' 2>&1; rc=\$?; cat '$T/prev.out'; test \$rc -eq 3 && grep -qF '>=$V' '$T/prev.out'"
fi

# 5. npm and PyPI, the ordinary commands, fresh caches.
step npx sh -c "npx -y spec-spine@$V --version | grep -qx 'spec-spine $V'"
step npm-install sh -c "mkdir -p '$T/npm' && cd '$T/npm' && npm init -y >/dev/null && npm i -D --no-audit --no-fund spec-spine@$V >/dev/null && ./node_modules/.bin/spec-spine --version | grep -qx 'spec-spine $V'"
step npm-dist-tag sh -c "test \"\$(npm view spec-spine dist-tags.latest)\" = '$V'"
step uvx sh -c "uvx spec-spine@$V --version | grep -qx 'spec-spine $V'"
step pip-install sh -c "python3 -m venv '$T/venv' && '$T/venv/bin/pip' install -q spec-spine==$V && '$T/venv/bin/spec-spine' --version | grep -qx 'spec-spine $V'"

echo "ALL=$rc_all scratch=$T"
exit $rc_all
