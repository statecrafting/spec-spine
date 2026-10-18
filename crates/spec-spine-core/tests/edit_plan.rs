//! Spec 165: digest-guarded edit plans, in core.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use spec_spine_core::shard::{self, BY_PACKAGE_DIR, BY_SPEC_DIR};
use spec_spine_core::{edit_plan, selected_content};
use spec_spine_types::{
    AuthorityEffect, ContentDirtyState, ContentProjection, ContentRequest, ContentSelector,
    ContentSnapshot, ContentSnapshotBinding, EditPlan, EditPlanRequest, PlanStatus, RefusalReason,
};

const BASE: &str = r#"---
id: "001-base"
title: "Base"
status: approved
created: "2026-10-01"
implementation: complete
summary: "s"
establishes:
  - { kind: file, path: "docs/base.md" }
unamendable:
  - "3-2-fixed"
obligations:
  - id: "R-1"
    kind: requirement
    text: "The rule holds."
    anchor: "3-1-the-rule"
  - id: "R-2"
    kind: requirement
    text: "Fixed."
    anchor: "3-2-fixed"
---
# 001

## 3. Behavior

### 3.1 The rule

Rule text.

### 3.2 Fixed

Fixed text.

## 6. Resolved decisions

**D-1:** one.
"#;

const WORK: &str = r#"---
id: "002-work"
title: "Work"
status: draft
created: "2026-10-02"
implementation: pending
summary: "s"
establishes:
  - { kind: file, path: "docs/work.md" }
obligations:
  - id: "R-1"
    kind: requirement
    text: "Work holds."
    anchor: "3-1-work"
---
# 002

## 3. Behavior

### 3.1 Work

Work text.

### 3.2 Notes

Notes text.
"#;

fn write(root: &Path, rel: &str, text: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

/// Commit the registry and index as `compile` and `index` write them.
fn commit_ledger(root: &Path) {
    let cfg = spec_spine_types::Config::default();
    let compiled = spec_spine_core::compile(&cfg, root).unwrap();
    assert!(
        compiled.validation_passed,
        "{:?}",
        compiled.registry.validation
    );
    let dir = spec_spine_core::registry_dir(&cfg, root).join(BY_SPEC_DIR);
    shard::sync_dir(
        &dir,
        &spec_spine_core::registry_shard_files(&compiled.shards).unwrap(),
    )
    .unwrap();
    let outcome = spec_spine_core::index(&cfg, root).unwrap();
    let dir = spec_spine_core::index_dir(&cfg, root);
    let (by_spec, by_package) = spec_spine_core::index_shard_files(&outcome.shards).unwrap();
    shard::sync_dir(&dir.join(BY_SPEC_DIR), &by_spec).unwrap();
    shard::sync_dir(&dir.join(BY_PACKAGE_DIR), &by_package).unwrap();
    let (name, inputs) = spec_spine_core::index_inputs_file(&outcome.shards).unwrap();
    fs::write(dir.join(name), inputs).unwrap();
}

fn corpus() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write(root, "specs/001-base/spec.md", BASE);
    write(root, "specs/002-work/spec.md", WORK);
    write(root, "docs/base.md", "# Base\n\n## Usage\n\nBase usage.\n");
    write(
        root,
        "docs/work.md",
        "# Work\n\n## Usage\n\nWork usage.\n\n## More\n\nMore.\n",
    );
    write(root, "docs/free.md", "# Free\n");
    write(root, "src/new.rs", "fn main() {}\n");
    commit_ledger(root);
    tmp
}

fn snapshot() -> ContentSnapshot {
    ContentSnapshot {
        repository: "example/repo".into(),
        revision: "1".repeat(40),
        tree: "2".repeat(40),
        dirty_state: ContentDirtyState::CleanExport,
        binding: ContentSnapshotBinding::CallerSupplied,
    }
}

fn spec_sel(spec: &str) -> ContentSelector {
    ContentSelector::Spec {
        spec: spec.into(),
        projection: None,
        required: true,
    }
}

/// The spec 155 item digest of `selector` at its full projection.
fn digest(root: &Path, selector: &ContentSelector) -> String {
    let response = selected_content(
        &spec_spine_types::Config::default(),
        root,
        &ContentRequest {
            selectors: vec![selector.clone()],
            default_projection: ContentProjection::Full,
            max_bytes: 1_048_576,
            max_items: 10,
            continuation: None,
        },
        &snapshot(),
    )
    .unwrap();
    response.items[0].digest.clone()
}

fn scope() -> serde_json::Value {
    serde_json::json!({ "ownSpec": "002", "mutable": ["specs/002-work/", "specs/003-amend/", "docs/", "src/"] })
}

fn request(operations: Vec<serde_json::Value>) -> EditPlanRequest {
    serde_json::from_value(serde_json::json!({
        "scope": scope(),
        "operations": operations,
        "consumerSchemaVersion": "1.0.0",
    }))
    .unwrap()
}

fn plan(root: &Path, operations: Vec<serde_json::Value>) -> EditPlan {
    edit_plan(
        &spec_spine_types::Config::default(),
        root,
        &request(operations),
        &snapshot(),
    )
    .unwrap()
}

fn reasons(p: &EditPlan) -> Vec<RefusalReason> {
    p.refusals.iter().map(|r| r.reason).collect()
}

fn claim(root: &Path, unit: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "op": "add-claim",
        "target": { "kind": "spec", "spec": "002-work" },
        "expected": digest(root, &spec_sel("002-work")),
        "edge": "establishes",
        "unit": unit,
    })
}

fn after(p: &EditPlan, path: &str) -> String {
    p.previews
        .iter()
        .find(|f| f.path == path)
        .unwrap_or_else(|| panic!("no preview for {path}: {:?}", p.refusals))
        .after_content
        .clone()
}

/// Every file under `root`, by relative path, as bytes.
fn tree(root: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut out = BTreeMap::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for e in fs::read_dir(dir).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                stack.push(p);
            } else {
                out.insert(
                    p.strip_prefix(root).unwrap().to_string_lossy().into_owned(),
                    fs::read(&p).unwrap(),
                );
            }
        }
    }
    out
}

// ---- 3.4: add-claim ----------------------------------------------------------

#[test]
fn an_establishes_claim_appends_one_flow_line_and_claims_territory() {
    let tmp = corpus();
    let root = tmp.path();
    let before = tree(root);
    let p = plan(
        root,
        vec![claim(
            root,
            serde_json::json!({ "kind": "file", "path": "src/new.rs" }),
        )],
    );
    assert_eq!(p.status, PlanStatus::Ready, "{:?}", p.refusals);
    let image = after(&p, "specs/002-work/spec.md");
    assert_eq!(
        image,
        WORK.replace(
            "  - { kind: file, path: \"docs/work.md\" }\n",
            "  - { kind: file, path: \"docs/work.md\" }\n  - { kind: file, path: \"src/new.rs\" }\n"
        )
    );
    assert_eq!(p.authority_effect, AuthorityEffect::ClaimsTerritory);
    assert_eq!(
        p.stales,
        vec![
            spec_spine_types::DerivedTree::SpecRegistry,
            spec_spine_types::DerivedTree::CodebaseIndex
        ]
    );
    let preview = &p.previews[0];
    assert!(preview.before_raw_digest.starts_with("sha256:"));
    assert!(
        preview
            .diff
            .contains("+  - { kind: file, path: \"src/new.rs\" }\n")
    );
    assert_eq!(tree(root), before, "planning writes nothing");
}

#[test]
fn claims_refuse_duplicates_foreign_establishers_and_unwritten_paths() {
    let tmp = corpus();
    let root = tmp.path();
    let dup = plan(
        root,
        vec![claim(
            root,
            serde_json::json!({ "kind": "file", "path": "docs/work.md" }),
        )],
    );
    assert_eq!(reasons(&dup), vec![RefusalReason::DuplicateDeclaration]);
    assert!(dup.previews.is_empty());

    let foreign = plan(
        root,
        vec![claim(
            root,
            serde_json::json!({ "kind": "file", "path": "docs/base.md" }),
        )],
    );
    assert_eq!(reasons(&foreign), vec![RefusalReason::UnauthorizedTarget]);

    let missing = plan(
        root,
        vec![claim(
            root,
            serde_json::json!({ "kind": "file", "path": "src/later.rs" }),
        )],
    );
    assert_eq!(reasons(&missing), vec![RefusalReason::UnresolvedTarget]);
    let planned = plan(
        root,
        vec![claim(
            root,
            serde_json::json!({ "kind": "file", "path": "src/later.rs", "planned": true }),
        )],
    );
    assert_eq!(planned.status, PlanStatus::Ready, "{:?}", planned.refusals);
    assert!(
        after(&planned, "specs/002-work/spec.md").contains("path: \"src/later.rs\", planned: true")
    );
}

#[test]
fn an_extends_edge_to_a_spec_that_never_claimed_the_unit_is_accepted() {
    let tmp = corpus();
    let root = tmp.path();
    let p = plan(
        root,
        vec![serde_json::json!({
            "op": "add-claim",
            "target": { "kind": "spec", "spec": "002" },
            "expected": digest(root, &spec_sel("002-work")),
            "edge": "extends",
            "spec": "001",
            "nature": "additive",
            "unit": { "kind": "file", "path": "src/new.rs" },
        })],
    );
    assert_eq!(p.status, PlanStatus::Ready, "{:?}", p.refusals);
    assert!(after(&p, "specs/002-work/spec.md").contains(
        "extends:\n  - { spec: \"001-base\", unit: { kind: file, path: \"src/new.rs\" }, nature: additive }\n---"
    ));
}

#[test]
fn a_changed_target_is_stale_with_both_digests_never_rebased() {
    let tmp = corpus();
    let root = tmp.path();
    let mut op = claim(
        root,
        serde_json::json!({ "kind": "file", "path": "src/new.rs" }),
    );
    let current = op["expected"].as_str().unwrap().to_string();
    let read_at = format!("sha256:{}", "0".repeat(64));
    op["expected"] = read_at.clone().into();
    let p = plan(root, vec![op]);
    assert_eq!(reasons(&p), vec![RefusalReason::StaleTarget]);
    assert!(p.refusals[0].detail.contains(&read_at) && p.refusals[0].detail.contains(&current));
}

#[test]
fn a_flow_sequence_layout_is_refused_rather_than_reserialized() {
    let tmp = corpus();
    let root = tmp.path();
    write(
        root,
        "specs/002-work/spec.md",
        &WORK.replace(
            "establishes:\n  - { kind: file, path: \"docs/work.md\" }\n",
            "establishes: [\"docs/work.md\"]\n",
        ),
    );
    commit_ledger(root);
    let p = plan(
        root,
        vec![claim(
            root,
            serde_json::json!({ "kind": "file", "path": "src/new.rs" }),
        )],
    );
    assert_eq!(reasons(&p), vec![RefusalReason::UnsupportedLayout]);
}

/// A single-quoted scalar holding `&` or `*` is text, not an anchor or alias,
/// so the block sequence it sits in is still edited in place.
#[test]
fn an_ampersand_inside_a_single_quoted_scalar_is_not_an_anchor() {
    let tmp = corpus();
    let root = tmp.path();
    write(
        root,
        "specs/002-work/spec.md",
        &WORK.replace(
            "  - { kind: file, path: \"docs/work.md\" }\n",
            "  - { kind: file, path: \"docs/work.md\" }\n  - { kind: file, path: 'docs/a &b *c.md', planned: true }\n",
        ),
    );
    commit_ledger(root);
    let p = plan(
        root,
        vec![claim(
            root,
            serde_json::json!({ "kind": "file", "path": "src/new.rs" }),
        )],
    );
    assert_eq!(p.status, PlanStatus::Ready, "{:?}", p.refusals);
    assert!(
        after(&p, "specs/002-work/spec.md")
            .contains("path: 'docs/a &b *c.md', planned: true }\n  - ")
    );
}

/// A stale amendment target is the whole answer, as for every other target:
/// nothing else about the operation is judged until the caller re-plans.
#[test]
fn a_stale_amendment_target_says_only_that_it_is_stale() {
    let tmp = corpus();
    let root = tmp.path();
    write(root, "specs/003-amend/spec.md", "already here\n");
    let p = plan(
        root,
        vec![serde_json::json!({
            "op": "propose-amendment",
            "target": "specs/003-amend/spec.md",
            "expected": "absent",
            "spec": "003-amend",
            "obligations": ["001-base#R-99"],
            "content": "not a spec\n",
        })],
    );
    assert_eq!(reasons(&p), vec![RefusalReason::StaleTarget]);
}

// ---- containment -------------------------------------------------------------

/// A linked parent directory is refused as a link at the last component is.
/// The link stays inside the repository: one leaving it is refused earlier,
/// by the repository-wide walk (spec 144), so only this case reaches the
/// per-component check. Without it the target reads as present.
#[cfg(unix)]
#[test]
fn a_target_under_a_linked_directory_is_refused_not_read() {
    let tmp = corpus();
    let root = tmp.path();
    write(root, "elsewhere/spec.md", "elsewhere\n");
    std::os::unix::fs::symlink("../elsewhere", root.join("specs/003-amend")).unwrap();
    let err = edit_plan(
        &spec_spine_types::Config::default(),
        root,
        &request(vec![serde_json::json!({
            "op": "propose-amendment",
            "target": "specs/003-amend/spec.md",
            "expected": "absent",
            "spec": "003-amend",
            "obligations": ["001-base#R-1"],
            "content": "---\nid: \"003-amend\"\n---\n",
        })]),
        &snapshot(),
    )
    .unwrap_err();
    assert!(
        matches!(&err, spec_spine_types::Error::Refused(m) if m.contains("symbolic link")),
        "{err:?}"
    );
}

/// The existence check a claim makes follows the same containment as a read:
/// a unit reached through a link, at the last component or a parent, is
/// refused rather than resolved through it.
#[cfg(unix)]
#[test]
fn a_claim_reached_through_a_link_is_refused_not_resolved() {
    let tmp = corpus();
    let root = tmp.path();
    std::os::unix::fs::symlink("new.rs", root.join("src/alias.rs")).unwrap();
    std::os::unix::fs::symlink("src", root.join("linked")).unwrap();
    for path in ["src/alias.rs", "linked/new.rs"] {
        let err = edit_plan(
            &spec_spine_types::Config::default(),
            root,
            &request(vec![claim(
                root,
                serde_json::json!({ "kind": "file", "path": path }),
            )]),
            &snapshot(),
        )
        .unwrap_err();
        assert!(
            matches!(&err, spec_spine_types::Error::Refused(m) if m.contains("symbolic link")),
            "{path}: {err:?}"
        );
    }
}
