#!/usr/bin/env bash
# Spec: specs/104-a-producer-is-tested-as-published/spec.md
#
# Assert the producer contract against the PACKAGED crate, through its public
# facade, from outside the workspace.
#
# Why this is not a workspace test: the published spec-spine-core 0.21.0 emits
# AGENTS.md and a .claude/rules/ tree, and the source tree under the same
# version string emits neither. Both states passed a green workspace suite,
# including twenty-one tests in crates/spec-spine-core/tests/scaffold.rs that
# assert exactly this contract. "The workspace is green" and "the artifact a
# consumer installs behaves this way" are different claims, and only the first
# was ever being made.
#
# Usage: ./scripts/verify-packaged-producer.sh [--keep]
#   --keep  leave the scratch directory in place for inspection
#
# Exit 0 only if every assertion passed.

set -euo pipefail

KEEP=0
[ "${1:-}" = "--keep" ] && KEEP=1

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO"

VERSION="$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -1)"
if [ -z "$VERSION" ]; then
  echo "could not read the workspace version from Cargo.toml" >&2
  exit 1
fi
COMMIT="$(git rev-parse HEAD 2>/dev/null || echo '(not a git repository)')"
DIRTY=""
if ! git diff --quiet HEAD 2>/dev/null; then
  DIRTY=" (working tree dirty; the digests below describe the tree, not the commit)"
fi

echo "=== packaged-producer acceptance (spec 104)"
echo "source commit: ${COMMIT}${DIRTY}"
echo "version:       ${VERSION}"
echo

# ---- 1. package, exactly as a publish would -------------------------------
PKG_ARGS=(-p spec-spine-types -p spec-spine-core)
if [ -n "$DIRTY" ]; then
  PKG_ARGS+=(--allow-dirty)
fi
echo "--- cargo package ${PKG_ARGS[*]}"
cargo package "${PKG_ARGS[@]}" --locked >/dev/null

CORE="target/package/spec-spine-core-${VERSION}.crate"
TYPES="target/package/spec-spine-types-${VERSION}.crate"
for f in "$CORE" "$TYPES"; do
  if [ ! -f "$f" ]; then
    echo "expected package not produced: $f" >&2
    exit 1
  fi
done

echo
echo "--- package identity"
if command -v shasum >/dev/null 2>&1; then
  shasum -a 256 "$CORE" "$TYPES"
else
  sha256sum "$CORE" "$TYPES"
fi
echo

# ---- 2. unpack OUTSIDE the workspace --------------------------------------
# Outside, deliberately: a consumer built inside the workspace can resolve a
# path dependency back into the tree and stop testing the package at all.
WORK="$(mktemp -d "${TMPDIR:-/tmp}/spec-spine-packaged-producer.XXXXXX")"
cleanup() {
  if [ "$KEEP" -eq 1 ]; then
    echo "scratch kept at: $WORK"
  else
    rm -rf "$WORK"
  fi
}
trap cleanup EXIT

tar xzf "$CORE" -C "$WORK"
tar xzf "$TYPES" -C "$WORK"
mkdir -p "$WORK/consumer/src"

cat > "$WORK/consumer/Cargo.toml" <<TOML
[package]
name = "packaged-producer-acceptance"
version = "0.0.0"
edition = "2021"

[dependencies]
spec-spine-core = { path = "../spec-spine-core-${VERSION}" }
serde_json = "1"

# The packaged core depends on spec-spine-types by version from the registry.
# Point it at the packaged types crate so nothing in this consumer reaches the
# registry or the workspace.
[patch.crates-io]
spec-spine-types = { path = "../spec-spine-types-${VERSION}" }

[workspace]
TOML

cat > "$WORK/consumer/src/main.rs" <<'RS'
//! The producer contract (spec 104 §3.2, §3.3), asserted through the public
//! facade of the packaged crate.

use std::collections::BTreeSet;

static mut FAILED: u32 = 0;

fn check(name: &str, cond: bool) {
    println!("{} {name}", if cond { "PASS" } else { "FAIL" });
    if !cond {
        unsafe { FAILED += 1 };
    }
}

fn scaffold(config: &str) -> serde_json::Value {
    let out = spec_spine_core::scaffold_init_json(config)
        .unwrap_or_else(|e| panic!("scaffold_init_json({config}): {e}"));
    serde_json::from_str(&out).expect("valid json")
}

fn rel_paths(v: &serde_json::Value) -> BTreeSet<String> {
    v["files"]
        .as_array()
        .expect("files array")
        .iter()
        .map(|f| f["relPath"].as_str().expect("relPath").to_string())
        .collect()
}

fn allowed() -> BTreeSet<String> {
    [
        "spec-spine.toml",
        "standards/spec/constitution.md",
        "standards/spec/contract.md",
        "standards/spec/templates/spec-template.md",
        "standards/spec/templates/constitution-template.md",
        "specs/000-bootstrap/spec.md",
        ".gitignore",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

fn main() {
    let d = scaffold("{}");
    let got = rel_paths(&d);
    let want = allowed();

    // ---- 3.2.1 exact allowed governance output ----------------------------
    // Equality, not containment: containment is what lets an extra file ride
    // along unnoticed, which is this check's entire subject.
    check("emits exactly the allowed governance file set", got == want);
    if got != want {
        let extra: Vec<_> = got.difference(&want).collect();
        let missing: Vec<_> = want.difference(&got).collect();
        println!("     extra:   {extra:?}");
        println!("     missing: {missing:?}");
    }

    // ---- 3.2.2 absence, by name and by content ----------------------------
    for f in ["AGENTS.md", "CLAUDE.md", "Makefile", ".mcp.json"] {
        check(&format!("emits no {f}"), !got.contains(f));
    }
    for p in [".claude/", ".agents/", ".codex/", ".github/", ".githooks/"] {
        check(
            &format!("emits nothing under {p}"),
            !got.iter().any(|g| g.starts_with(p)),
        );
    }
    let all: String = d["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["contents"].as_str().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n");
    // By content too: a harness that comes back under a different filename is
    // the same regression.
    for marker in ["SessionStart", "PostToolUse", "PreToolUse", "slash command"] {
        check(
            &format!("no emitted file carries the harness token {marker:?}"),
            !all.contains(marker),
        );
    }

    // ---- 3.2.3 append-marker behavior -------------------------------------
    let gitignore = d["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["relPath"] == ".gitignore")
        .expect(".gitignore must be emitted");
    check(
        ".gitignore is an append, not a whole-file write",
        gitignore["append"] == serde_json::Value::Bool(true),
    );
    let marker = gitignore["appendMarker"].as_str().unwrap_or("");
    check("the append carries a non-empty marker", !marker.is_empty());
    check(
        "the marker is inside the appended contents, so re-applying is idempotent",
        !marker.is_empty() && gitignore["contents"].as_str().unwrap().contains(marker),
    );
    for f in d["files"].as_array().unwrap() {
        let p = f["relPath"].as_str().unwrap();
        if p != ".gitignore" {
            check(
                &format!("{p} is a whole-file write with a null marker"),
                f["append"] == serde_json::Value::Bool(false) && f["appendMarker"].is_null(),
            );
        }
        check(
            &format!("{p} does not overwrite an existing file"),
            f["overwrite"] == serde_json::Value::Bool(false),
        );
    }

    // ---- 3.2.4 determinism and documented purity --------------------------
    let a = spec_spine_core::scaffold_init_json("{}").unwrap();
    let b = spec_spine_core::scaffold_init_json("{}").unwrap();
    check("two calls are byte-identical", a == b);

    let probe = std::env::temp_dir().join(format!("ssp-purity-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&probe);
    std::fs::create_dir_all(&probe).unwrap();
    let cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(&probe).unwrap();
    let _ = spec_spine_core::scaffold_init_json("{}").unwrap();
    let left = std::fs::read_dir(&probe).unwrap().count();
    std::env::set_current_dir(cwd).unwrap();
    let _ = std::fs::remove_dir_all(&probe);
    check("the producer writes nothing", left == 0);

    // ---- 3.3 the configuration argument is snake_case ---------------------
    check(
        "a camelCase config key is refused, not silently ignored",
        spec_spine_core::scaffold_init_json(r#"{"layout":{"derivedDir":".x"}}"#).is_err(),
    );

    // ---- 3.2.5 Statecraft's requested layout ------------------------------
    let managed = scaffold(
        r#"{"layout":{"derived_dir":".statecraft/derived","state_dir":".statecraft/state"}}"#,
    );
    let text = |v: &serde_json::Value, rel: &str| -> String {
        v["files"]
            .as_array()
            .unwrap()
            .iter()
            .find(|f| f["relPath"] == rel)
            .unwrap_or_else(|| panic!("{rel} must be emitted"))["contents"]
            .as_str()
            .unwrap()
            .to_string()
    };
    let toml = text(&managed, "spec-spine.toml");
    check(
        "the managed derived root reaches the scaffolded config",
        toml.contains(".statecraft/derived"),
    );
    check(
        "the managed state root reaches the scaffolded config",
        toml.contains(".statecraft/state"),
    );
    check(
        "the scaffolded .gitignore follows the configured state root",
        text(&managed, ".gitignore").contains(".statecraft/state"),
    );
    check(
        "the managed layout emits the same file set and nothing more",
        rel_paths(&managed) == want,
    );

    let failed = unsafe { FAILED };
    println!();
    if failed == 0 {
        println!("ALL PACKAGED-PRODUCER ACCEPTANCE CHECKS PASSED");
    } else {
        println!("{failed} PACKAGED-PRODUCER ACCEPTANCE CHECK(S) FAILED");
        std::process::exit(1);
    }
}
RS

echo "--- building a consumer against the unpacked packaged sources"
echo "    (outside the workspace: $WORK/consumer)"
echo
( cd "$WORK/consumer" && cargo run --quiet )

echo
echo "=== packaged-producer acceptance: OK"
echo "This records that an artifact built from ${COMMIT} behaves correctly."
echo "It is not a claim that anything was published."
