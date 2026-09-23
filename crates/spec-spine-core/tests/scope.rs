// Spec: specs/108-a-work-scope-is-declared/spec.md
//! Spec 108: a work scope is evaluated against the committed ownership index,
//! and two scopes are compared for conflicting intentions. Every finding and
//! conflict kind is asserted positively and negatively (spec 108 `V-1`).

use std::fs;
use std::path::Path;

use serde_json::{Value, json};
use spec_spine_core::shard::{self, BY_PACKAGE_DIR, BY_SPEC_DIR};
use spec_spine_core::{
    Conflict, DiffFile, DiffInput, Finding, Role, ScopeRequest, SharedPath, compare_scopes,
    compile, couple, evaluate, evaluate_scope, index, index_dir, index_shard_files, registry_dir,
    registry_shard_files, scope_compare_json, scope_json,
};
use spec_spine_types::{CodebaseIndex, Config, Error, Registry};

// ---- hand-crafted registry / index, mirroring couple.rs's pattern ---------

fn registry_from(specs: Value) -> Registry {
    serde_json::from_value(json!({
        "specVersion": "0.1.0",
        "build": {
            "compilerId": "t", "compilerVersion": "0.1.0",
            "inputRoot": ".", "contentHash": "t"
        },
        "specs": specs,
        "validation": { "passed": true, "violations": [] }
    }))
    .expect("registry json")
}

fn spec_stub(id: &str) -> Value {
    json!({
        "id": id, "title": id, "status": "approved", "created": "2026-09-22",
        "summary": "s", "specPath": format!("specs/{id}/spec.md")
    })
}

fn index_from(mappings: Value) -> CodebaseIndex {
    serde_json::from_value(json!({
        "schemaVersion": "0.1.0",
        "build": {
            "indexerId": "t", "indexerVersion": "0.1.0",
            "repoRoot": ".", "contentHash": "idx-hash"
        },
        "packages": [],
        "traceability": {
            "mappings": mappings,
            "orphanedSpecs": [],
            "untracedCode": []
        },
        "diagnostics": { "warnings": [], "errors": [] }
    }))
    .expect("index json")
}

/// A whole-file `establishes` claim, the shape `index.rs` emits for one.
fn establishes(spec_id: &str, file: &str) -> Value {
    json!({
        "specId": spec_id,
        "implementingPaths": [],
        "resolvedUnits": [{
            "unit": { "kind": "file", "path": file },
            "sourceField": "establishes", "ownership": true,
            "locations": [{ "file": file }]
        }]
    })
}

/// The corpus every unit test resolves against: `100-a` owns
/// `crates/a/src/lib.rs`, `200-b` owns `crates/b/src/lib.rs`, `300-c` owns
/// the whole `crates/c/` subtree, and `crates/unowned/file.rs` is claimed by
/// nobody.
fn corpus() -> (Registry, CodebaseIndex) {
    let registry = registry_from(json!([
        spec_stub("100-a"),
        spec_stub("200-b"),
        spec_stub("300-c"),
    ]));
    let index = index_from(json!([
        establishes("100-a", "crates/a/src/lib.rs"),
        establishes("200-b", "crates/b/src/lib.rs"),
        establishes("300-c", "crates/c/"),
    ]));
    (registry, index)
}

fn req(
    own_spec: &str,
    mutable: &[&str],
    shared: &[(&str, &[&str])],
    read_only: &[&str],
) -> ScopeRequest {
    ScopeRequest {
        id: None,
        own_spec: own_spec.to_string(),
        mutable: mutable.iter().map(|s| s.to_string()).collect(),
        shared: shared
            .iter()
            .map(|(p, with)| SharedPath {
                path: p.to_string(),
                with: with.iter().map(|s| s.to_string()).collect(),
            })
            .collect(),
        read_only: read_only.iter().map(|s| s.to_string()).collect(),
    }
}

fn eval(
    own_spec: &str,
    mutable: &[&str],
    shared: &[(&str, &[&str])],
    read_only: &[&str],
) -> spec_spine_core::ScopeEvaluation {
    let (registry, index) = corpus();
    evaluate_scope(
        &Config::default(),
        &registry,
        &index,
        &req(own_spec, mutable, shared, read_only),
    )
    .unwrap()
}

fn codes(findings: &[Finding]) -> Vec<&str> {
    findings.iter().map(|f| f.code.as_str()).collect()
}

// ---- 3.1, 3.3: no finding when the declaration matches ownership ---------

#[test]
fn self_owned_mutable_path_has_no_finding() {
    let e = eval("100-a", &["crates/a/src/lib.rs"], &[], &[]);
    assert!(e.findings.is_empty(), "{:?}", e.findings);
    assert_eq!(e.entries[0].owners, vec!["100-a".to_string()]);
    assert_eq!(e.entries[0].role, Role::Mutable);
    assert_eq!(e.own_spec, "100-a");
    assert_eq!(e.index_hash, "idx-hash");
}

// ---- 3.3: S-002, and the same path cleared once declared shared -----------

#[test]
fn mutable_path_owned_elsewhere_is_s002_naming_both_then_clears_when_shared() {
    let e = eval("100-a", &["crates/b/src/lib.rs"], &[], &[]);
    assert_eq!(codes(&e.findings), vec!["S-002"]);
    let f = &e.findings[0];
    assert_eq!(f.own_spec.as_deref(), Some("100-a"));
    assert_eq!(f.owners, vec!["200-b".to_string()]);

    let cleared = eval("100-a", &[], &[("crates/b/src/lib.rs", &["200-b"])], &[]);
    assert!(cleared.findings.is_empty(), "{:?}", cleared.findings);
}

// ---- 3.3: S-001 -------------------------------------------------------------

#[test]
fn mutable_path_unowned_is_s001() {
    let e = eval("100-a", &["crates/unowned/file.rs"], &[], &[]);
    assert_eq!(codes(&e.findings), vec!["S-001"]);
    let f = &e.findings[0];
    assert_eq!(f.path, "crates/unowned/file.rs");
    assert!(f.owners.is_empty());
    assert!(f.own_spec.is_none());
    assert!(e.entries[0].owners.is_empty());
}

#[test]
fn shared_path_unowned_is_also_s001_not_s003() {
    let e = eval("100-a", &[], &[("crates/unowned/file.rs", &["200-b"])], &[]);
    assert_eq!(codes(&e.findings), vec!["S-001"]);
}

// ---- 3.3: S-003, naming both sets ------------------------------------------

#[test]
fn shared_with_mismatch_is_s003_naming_both_sets() {
    // Owned by 200-b, but declared shared with 300-c: the sets disagree.
    let e = eval("100-a", &[], &[("crates/b/src/lib.rs", &["300-c"])], &[]);
    assert_eq!(codes(&e.findings), vec!["S-003"]);
    let f = &e.findings[0];
    assert_eq!(f.own_spec.as_deref(), Some("100-a"));
    assert_eq!(f.owners, vec!["200-b".to_string()]);
    assert_eq!(f.with, vec!["300-c".to_string()]);
}

#[test]
fn shared_with_matching_owners_has_no_finding() {
    let e = eval("100-a", &[], &[("crates/b/src/lib.rs", &["200-b"])], &[]);
    assert!(e.findings.is_empty(), "{:?}", e.findings);
    assert_eq!(
        e.entries[0].with.as_deref(),
        Some(["200-b".to_string()].as_slice())
    );
}

// ---- 3.3: readOnly raises nothing, whatever it resolves to -----------------

#[test]
fn read_only_owned_elsewhere_raises_no_finding() {
    let e = eval("100-a", &[], &[], &["crates/b/src/lib.rs"]);
    assert!(e.findings.is_empty(), "{:?}", e.findings);
    assert_eq!(e.entries[0].owners, vec!["200-b".to_string()]);
    assert_eq!(e.entries[0].role, Role::ReadOnly);
    assert!(e.entries[0].with.is_none());

    let unowned = eval("100-a", &[], &[], &["crates/unowned/file.rs"]);
    assert!(unowned.findings.is_empty());
}

// ---- 3.2: subtree resolution reaches in both directions --------------------

#[test]
fn subtree_entry_owners_include_claims_inside_and_around_it() {
    let (registry, index) = corpus();
    // A subtree entry equal to a claimed subtree.
    let e = evaluate_scope(
        &Config::default(),
        &registry,
        &index,
        &req("300-c", &["crates/c/"], &[], &[]),
    )
    .unwrap();
    assert_eq!(e.entries[0].owners, vec!["300-c".to_string()]);
    assert!(e.findings.is_empty());

    // A subtree entry containing a claimed file inside it.
    let e = evaluate_scope(
        &Config::default(),
        &registry,
        &index,
        &req("100-a", &["crates/"], &[], &[]),
    )
    .unwrap();
    // Both `crates/a/src/lib.rs` (100-a) and `crates/b/src/lib.rs` (200-b) are
    // inside `crates/`, so both are owners; 200-b is a crossing.
    assert_eq!(
        e.entries[0].owners,
        vec![
            "100-a".to_string(),
            "200-b".to_string(),
            "300-c".to_string()
        ]
    );
    assert_eq!(codes(&e.findings), vec!["S-002"]);
    assert_eq!(
        e.findings[0].owners,
        vec!["200-b".to_string(), "300-c".to_string()]
    );

    // A subtree entry inside a claimed subtree (the reverse direction).
    let e = evaluate_scope(
        &Config::default(),
        &registry,
        &index,
        &req("300-c", &["crates/c/inner/"], &[], &[]),
    )
    .unwrap();
    assert_eq!(e.entries[0].owners, vec!["300-c".to_string()]);
    assert!(e.findings.is_empty());
}

// ---- 3.3: findings sorted by path, then code -------------------------------

#[test]
fn findings_are_sorted_by_path_then_code() {
    let (registry, index) = corpus();
    let e = evaluate_scope(
        &Config::default(),
        &registry,
        &index,
        &req(
            "100-a",
            &["crates/unowned/z.rs", "crates/b/src/lib.rs"],
            &[],
            &[],
        ),
    )
    .unwrap();
    let paths: Vec<&str> = e.findings.iter().map(|f| f.path.as_str()).collect();
    assert_eq!(paths, vec!["crates/b/src/lib.rs", "crates/unowned/z.rs"]);
}

// ---- 3.2: an unknown ownSpec or with is refused, naming every one ---------

#[test]
fn unknown_own_spec_and_with_are_named_in_one_refusal() {
    let (registry, index) = corpus();
    let err = evaluate_scope(
        &Config::default(),
        &registry,
        &index,
        &req("999", &[], &[("crates/b/src/lib.rs", &["888"])], &[]),
    )
    .unwrap_err();
    assert_eq!(err.exit_code(), 1, "{err}");
    let msg = err.to_string();
    assert!(msg.contains("999"), "{msg}");
    assert!(msg.contains("888"), "{msg}");
}

// ---- 3.1: malformed documents are exit-3 refusals --------------------------

#[test]
fn an_unknown_request_member_is_refused() {
    let err = serde_json::from_str::<ScopeRequest>(r#"{"ownSpec":"100","other":true}"#)
        .unwrap_err()
        .to_string();
    assert!(err.contains("unknown field"), "{err}");
}

#[test]
fn no_path_at_all_is_a_parse_error() {
    let (registry, index) = corpus();
    let err = evaluate_scope(
        &Config::default(),
        &registry,
        &index,
        &req("100-a", &[], &[], &[]),
    )
    .unwrap_err();
    assert_eq!(err.exit_code(), 3, "{err}");
}

#[test]
fn a_dotdot_segment_is_a_parse_error() {
    let (registry, index) = corpus();
    let err = evaluate_scope(
        &Config::default(),
        &registry,
        &index,
        &req("100-a", &["crates/../etc"], &[], &[]),
    )
    .unwrap_err();
    assert_eq!(err.exit_code(), 3, "{err}");
}

#[test]
fn an_absolute_path_is_a_parse_error() {
    let (registry, index) = corpus();
    let err = evaluate_scope(
        &Config::default(),
        &registry,
        &index,
        &req("100-a", &["/etc/passwd"], &[], &[]),
    )
    .unwrap_err();
    assert_eq!(err.exit_code(), 3, "{err}");
}

#[test]
fn an_empty_with_is_a_parse_error() {
    let (registry, index) = corpus();
    let err = evaluate_scope(
        &Config::default(),
        &registry,
        &index,
        &req("100-a", &[], &[("crates/b/src/lib.rs", &[])], &[]),
    )
    .unwrap_err();
    assert_eq!(err.exit_code(), 3, "{err}");
}

#[test]
fn one_path_under_two_roles_is_a_parse_error() {
    let (registry, index) = corpus();
    // The exact same path, mutable and readOnly.
    let err = evaluate_scope(
        &Config::default(),
        &registry,
        &index,
        &req(
            "100-a",
            &["crates/a/src/lib.rs"],
            &[],
            &["crates/a/src/lib.rs"],
        ),
    )
    .unwrap_err();
    assert_eq!(err.exit_code(), 3, "{err}");

    // A subtree and a path inside it, under two roles.
    let err = evaluate_scope(
        &Config::default(),
        &registry,
        &index,
        &req("300-c", &["crates/c/"], &[], &["crates/c/inner.rs"]),
    )
    .unwrap_err();
    assert_eq!(err.exit_code(), 3, "{err}");
}

#[test]
fn a_path_named_twice_under_one_role_appears_once() {
    let (registry, index) = corpus();
    let e = evaluate_scope(
        &Config::default(),
        &registry,
        &index,
        &req(
            "100-a",
            &["crates/a/src/lib.rs", "crates/a/src/lib.rs"],
            &[],
            &[],
        ),
    )
    .unwrap();
    assert_eq!(e.entries.len(), 1);
}

// ---- 3.5: comparison --------------------------------------------------------

fn conflict_kinds(c: &spec_spine_core::ScopeComparison) -> Vec<&str> {
    c.conflicts.iter().map(|k| k.kind.as_str()).collect()
}

#[test]
fn same_path_mutable_both_conflicts_as_both_mutable() {
    let a = req("100-a", &["crates/x.rs"], &[], &[]);
    let b = req("200-b", &["crates/x.rs"], &[], &[]);
    let cmp = compare_scopes(&a, &b).unwrap();
    assert_eq!(conflict_kinds(&cmp), vec!["both-mutable"]);
    assert_eq!(cmp.a.own_spec, "100-a");
    assert_eq!(cmp.b.own_spec, "200-b");
}

#[test]
fn a_subtree_against_a_file_inside_it_overlaps() {
    let a = req("100-a", &["crates/"], &[], &[]);
    let b = req("200-b", &["crates/x/y.rs"], &[], &[]);
    let cmp = compare_scopes(&a, &b).unwrap();
    assert_eq!(conflict_kinds(&cmp), vec!["both-mutable"]);
}

#[test]
fn mutable_against_shared_conflicts_as_mutable_shared() {
    let a = req("100-a", &["crates/x.rs"], &[], &[]);
    let b = req("200-b", &[], &[("crates/x.rs", &["100-a"])], &[]);
    let cmp = compare_scopes(&a, &b).unwrap();
    assert_eq!(conflict_kinds(&cmp), vec!["mutable-shared"]);
}

#[test]
fn changed_against_read_only_conflicts_as_changed_under_read() {
    // mutable vs readOnly
    let a = req("100-a", &["crates/x.rs"], &[], &[]);
    let b = req("200-b", &[], &[], &["crates/x.rs"]);
    assert_eq!(
        conflict_kinds(&compare_scopes(&a, &b).unwrap()),
        vec!["changed-under-read"]
    );

    // shared vs readOnly
    let a = req("100-a", &[], &[("crates/x.rs", &["200-b"])], &[]);
    let b = req("200-b", &[], &[], &["crates/x.rs"]);
    assert_eq!(
        conflict_kinds(&compare_scopes(&a, &b).unwrap()),
        vec!["changed-under-read"]
    );
}

#[test]
fn two_scopes_sharing_only_a_read_only_path_do_not_conflict() {
    let a = req("100-a", &[], &[], &["crates/x.rs"]);
    let b = req("200-b", &[], &[], &["crates/x.rs"]);
    assert!(compare_scopes(&a, &b).unwrap().conflicts.is_empty());
}

#[test]
fn two_scopes_sharing_only_a_shared_path_do_not_conflict() {
    let a = req("100-a", &[], &[("crates/x.rs", &["200-b"])], &[]);
    let b = req("200-b", &[], &[("crates/x.rs", &["100-a"])], &[]);
    assert!(compare_scopes(&a, &b).unwrap().conflicts.is_empty());
}

#[test]
fn compare_reads_no_ledger_an_unknown_own_spec_still_compares() {
    // Comparison never resolves ownSpec or `with` against a corpus (§3.5): an
    // id neither registry knows still compares, carried through as declared.
    let a = req("nonexistent-spec", &["crates/x.rs"], &[], &[]);
    let b = req("also-nonexistent", &["crates/x.rs"], &[], &[]);
    let cmp = compare_scopes(&a, &b).unwrap();
    assert_eq!(cmp.a.own_spec, "nonexistent-spec");
    assert_eq!(conflict_kinds(&cmp), vec!["both-mutable"]);
}

#[test]
fn compare_still_validates_each_document_structurally() {
    let bad = req("100-a", &["/etc/passwd"], &[], &[]);
    let ok = req("200-b", &["crates/x.rs"], &[], &[]);
    assert_eq!(compare_scopes(&bad, &ok).unwrap_err().exit_code(), 3);
}

// ---- 3.4, 3.6, 3.8: the IO wrapper and the facade --------------------------

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, content).unwrap();
}

fn disk_repo() -> (tempfile::TempDir, Config) {
    let tmp = tempfile::tempdir().unwrap();
    let r = tmp.path();
    write(r, "crates/a/src/lib.rs", "pub fn a() {}\n");
    write(r, "crates/b/src/lib.rs", "pub fn b() {}\n");
    write(
        r,
        "specs/100-a/spec.md",
        "---\nid: \"100-a\"\ntitle: \"A\"\nstatus: approved\ncreated: \"2026-09-22\"\n\
         summary: \"s\"\nestablishes:\n  - \"crates/a/src/lib.rs\"\n---\n# a\n",
    );
    write(
        r,
        "specs/200-b/spec.md",
        "---\nid: \"200-b\"\ntitle: \"B\"\nstatus: approved\ncreated: \"2026-09-22\"\n\
         summary: \"s\"\nestablishes:\n  - \"crates/b/src/lib.rs\"\n---\n# b\n",
    );
    let cfg = Config::default();
    let compiled = compile(&cfg, r).unwrap();
    assert!(
        compiled.validation_passed,
        "{:?}",
        compiled.registry.validation
    );
    let rdir = registry_dir(&cfg, r).join(BY_SPEC_DIR);
    fs::create_dir_all(&rdir).unwrap();
    for (name, bytes) in registry_shard_files(&compiled.shards).unwrap() {
        fs::write(rdir.join(name), bytes).unwrap();
    }
    let outcome = index(&cfg, r).unwrap();
    let idir = index_dir(&cfg, r);
    let (by_spec, by_package) = index_shard_files(&outcome.shards).unwrap();
    shard::sync_dir(&idir.join(BY_SPEC_DIR), &by_spec).unwrap();
    shard::sync_dir(&idir.join(BY_PACKAGE_DIR), &by_package).unwrap();
    (tmp, cfg)
}

fn snapshot(root: &Path) -> std::collections::BTreeMap<String, Vec<u8>> {
    let mut out = std::collections::BTreeMap::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for e in fs::read_dir(&dir).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                stack.push(p);
            } else {
                out.insert(p.display().to_string(), fs::read(&p).unwrap());
            }
        }
    }
    out
}

#[test]
fn the_committed_ledger_answers_and_nothing_is_written() {
    let (tmp, cfg) = disk_repo();
    let before = snapshot(tmp.path());
    let r = req("100-a", &["crates/a/src/lib.rs"], &[], &[]);
    let e = evaluate(&cfg, tmp.path(), &r).unwrap();
    assert_eq!(snapshot(tmp.path()), before, "a scope evaluation is a read");
    assert!(e.findings.is_empty());

    let doc: Value = serde_json::from_str(
        &scope_json(
            "{}",
            tmp.path().to_str().unwrap(),
            r#"{"ownSpec":"100","mutable":["crates/a/src/lib.rs"]}"#,
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(doc["ownSpec"], "100-a");
    assert_eq!(doc["schemaVersion"], spec_spine_types::READ_SCHEMA_VERSION);
    assert_eq!(spec_spine_types::READ_SCHEMA_VERSION, "0.8.0");
    assert_eq!(doc["findings"].as_array().unwrap().len(), 0);

    // The compare facade also carries the read axis version.
    let cmp_doc: Value = serde_json::from_str(
        &scope_compare_json(
            r#"{"ownSpec":"100","mutable":["crates/a/src/lib.rs"]}"#,
            r#"{"ownSpec":"200","mutable":["crates/a/src/lib.rs"]}"#,
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        cmp_doc["schemaVersion"],
        spec_spine_types::READ_SCHEMA_VERSION
    );
    assert_eq!(cmp_doc["conflicts"][0]["kind"], "both-mutable");
}

#[test]
fn a_stale_index_is_refused_before_anything_is_resolved() {
    let (tmp, cfg) = disk_repo();
    let spec = tmp.path().join("specs/100-a/spec.md");
    let text = fs::read_to_string(&spec).unwrap();
    fs::write(&spec, text.replace("# a", "# a, edited")).unwrap();
    let r = req("100-a", &["crates/a/src/lib.rs"], &[], &[]);
    let err = evaluate(&cfg, tmp.path(), &r).unwrap_err();
    assert_eq!(err.exit_code(), 2, "{err}");
}

// ---- gate neutrality: no verb changes because a scope exists ---------------

#[test]
fn couples_verdict_is_unaffected_by_a_committed_scope_document() {
    let (tmp, cfg) = disk_repo();
    let diff = DiffInput {
        files: vec![DiffFile {
            path: "crates/b/src/lib.rs".to_string(),
            hunks: Vec::new(),
            deleted: false,
        }],
    };
    let before = couple(&cfg, tmp.path(), &diff, None).unwrap();

    // A scope committed in the tree declares this very path `readOnly` for a
    // different piece of work entirely.
    write(
        tmp.path(),
        "work/w1-scope.json",
        r#"{"ownSpec":"100-a","readOnly":["crates/b/src/lib.rs"]}"#,
    );
    let after = couple(&cfg, tmp.path(), &diff, None).unwrap();

    assert_eq!(
        serde_json::to_string(&before).unwrap(),
        serde_json::to_string(&after).unwrap(),
        "a scope document must not change couple's verdict"
    );
}

#[allow(dead_code)]
fn assert_error_variant_reachable(_: Error) {}

#[allow(dead_code)]
fn assert_conflict_variant_reachable(_: Conflict) {}
