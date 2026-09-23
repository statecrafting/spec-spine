// Spec: specs/109-impact-and-conflict-are-declared/spec.md
//! Spec 109 §3.6: `registry impacts` through the shipped binary. The core
//! tests prove the rules; this proves the verb a consumer actually calls: the
//! read document, both filters, exit codes, and the lint warning.

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn run_in(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_spec-spine"))
        .arg("--repo")
        .arg(root)
        .args(args)
        .output()
        .expect("spawn spec-spine")
}

fn write(root: &Path, id: &str, extra: &str) {
    let dir = root.join("specs").join(id);
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("spec.md"),
        format!(
            "---\nid: \"{id}\"\ntitle: \"T\"\nstatus: draft\ncreated: \"2026-09-22\"\nsummary: \"s\"\n{extra}\
             ---\n# {id}\n\n## 3. Behavior\n\n### 3.1 The rule\n\nText.\n\n## Verification\n\nChecks.\n"
        ),
    )
    .unwrap();
}

fn corpus() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    write(
        tmp.path(),
        "001-a",
        "obligations:\n  - id: \"R-1\"\n    kind: requirement\n    text: \"The rule holds.\"\n    anchor: \"3-1-the-rule\"\n",
    );
    write(
        tmp.path(),
        "002-b",
        "obligations:\n  - id: \"R-9\"\n    kind: requirement\n    text: \"New.\"\n    anchor: \"3-1-the-rule\"\n\
         impacts:\n  - obligation: \"001#R-1\"\n    nature: refines\n\
         conflicts:\n  - obligation: \"001#R-1\"\n    reason: \"why\"\n    resolution: unresolved\n",
    );
    let out = run_in(tmp.path(), &["compile"]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    tmp
}

#[test]
fn the_verb_answers_a_read_document_with_both_arrays() {
    let tmp = corpus();
    let out = run_in(tmp.path(), &["registry", "impacts", "--json"]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let doc: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(doc["schemaVersion"], spec_spine_types::READ_SCHEMA_VERSION);
    assert_eq!(doc["impacts"][0]["declaredBy"], "002-b");
    assert_eq!(doc["impacts"][0]["target"], "001-a#R-1");
    assert_eq!(doc["impacts"][0]["nature"], "refines");
    assert_eq!(doc["conflicts"][0]["resolution"], "unresolved");
    assert_eq!(doc["conflicts"][0]["target"], "001-a#R-1");
}

#[test]
fn both_filters_compose_by_intersection() {
    let tmp = corpus();
    let out = run_in(
        tmp.path(),
        &[
            "registry",
            "impacts",
            "--target",
            "001-a",
            "--declared-by",
            "002",
            "--json",
        ],
    );
    assert_eq!(out.status.code(), Some(0));
    let doc: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(doc["impacts"].as_array().unwrap().len(), 1);
    assert_eq!(doc["conflicts"].as_array().unwrap().len(), 1);

    // A declared_by that names a different spec matches nothing.
    let out = run_in(
        tmp.path(),
        &["registry", "impacts", "--declared-by", "001-a", "--json"],
    );
    assert_eq!(out.status.code(), Some(0));
    let doc: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(doc["impacts"].as_array().unwrap().len(), 0);
    assert_eq!(doc["conflicts"].as_array().unwrap().len(), 0);
}

#[test]
fn an_unqualified_hash_target_exits_3_and_an_unknown_spec_exits_1() {
    let tmp = corpus();
    let out = run_in(tmp.path(), &["registry", "impacts", "--target", "#R-1"]);
    assert_eq!(
        out.status.code(),
        Some(3),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("a qualified form"),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let out = run_in(tmp.path(), &["registry", "impacts", "--target", "999-none"]);
    assert_eq!(out.status.code(), Some(1));

    let out = run_in(
        tmp.path(),
        &["registry", "impacts", "--declared-by", "999-none"],
    );
    assert_eq!(out.status.code(), Some(1));
}

#[test]
fn the_text_form_names_both_identities() {
    let tmp = corpus();
    let out = run_in(tmp.path(), &["registry", "impacts"]);
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("impact"), "{text}");
    assert!(text.contains("conflict"), "{text}");
    assert!(text.contains("002-b"), "{text}");
    assert!(text.contains("001-a#R-1"), "{text}");
}

#[test]
fn an_unresolved_conflict_warns_and_the_gate_is_unmoved_without_fail_on_warn() {
    let tmp = corpus();
    let out = run_in(tmp.path(), &["lint", "--json"]);
    assert_eq!(out.status.code(), Some(0));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let violations = json["report"].as_array().cloned().unwrap_or_default();
    assert!(
        violations
            .iter()
            .any(|v| v["code"] == "L-014" && v["severity"] == "warning"),
        "{json}"
    );

    // Bare `lint` (no --fail-on-warn) still exits 0.
    let out = run_in(tmp.path(), &["lint"]);
    assert_eq!(out.status.code(), Some(0));

    // `--fail-on-warn` refuses.
    let out = run_in(tmp.path(), &["lint", "--fail-on-warn"]);
    assert_ne!(out.status.code(), Some(0));
}
