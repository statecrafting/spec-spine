// Spec: specs/106-obligations-are-declared-constraints/spec.md
//! Spec 106 §3.6: `registry obligation` through the shipped binary. The core
//! tests prove the rules; this proves the verb a consumer actually calls: exit
//! codes, the read document, and the content hash read from the shard.

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

fn corpus() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("specs/001-a");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("spec.md"),
        "---\nid: \"001-a\"\ntitle: \"T\"\nstatus: draft\ncreated: \"2026-09-22\"\nsummary: \"s\"\n\
         obligations:\n  - id: \"R-1\"\n    kind: requirement\n    text: \"The rule holds.\"\n    anchor: \"3-1-the-rule\"\n\
         \x20 - id: \"V-1\"\n    kind: verification\n    text: \"It is checked.\"\n    anchor: \"verification\"\n    inputs: [\"tests/a.rs\"]\n\
         \x20 - id: \"R-2\"\n    kind: requirement\n    text: \"The old rule.\"\n    anchor: \"3-1-the-rule\"\n    withdrawn: true\n\
         ---\n# 001\n\n## 3. Behavior\n\n### 3.1 The rule\n\nText.\n\n## Verification\n\nChecks.\n",
    )
    .unwrap();
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
fn a_qualified_reference_resolves_to_a_read_document_with_both_identities() {
    let tmp = corpus();
    let out = run_in(tmp.path(), &["registry", "obligation", "001#V-1", "--json"]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let doc: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(doc["spec"], "001-a");
    assert_eq!(doc["specPath"], "specs/001-a/spec.md");
    assert_eq!(doc["obligation"]["kind"], "verification");
    assert_eq!(
        doc["obligation"]["inputs"],
        serde_json::json!(["tests/a.rs"])
    );
    assert_eq!(doc["schemaVersion"], spec_spine_types::READ_SCHEMA_VERSION);

    // Both identities are read from the committed shard, never recomputed.
    let shard: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(tmp.path().join(".derived/spec-registry/by-spec/001-a.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(doc["contentHash"], shard["shardHash"]);
    assert_eq!(
        doc["sectionDigest"],
        shard["record"]["sectionDigests"]["verification"]
    );
}

#[test]
fn a_withdrawn_obligation_resolves_and_says_so() {
    let tmp = corpus();
    let out = run_in(tmp.path(), &["registry", "obligation", "001-a#R-2"]);
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.starts_with("001-a#R-2  (withdrawn)"), "{text}");
    assert!(text.contains("sectionDigest: "), "{text}");
    assert!(text.contains("contentHash:   "), "{text}");
}

#[test]
fn an_unqualified_reference_exits_3_and_an_unknown_one_exits_1() {
    let tmp = corpus();
    for bad in ["R-1", "#R-1", "001#"] {
        let out = run_in(tmp.path(), &["registry", "obligation", bad]);
        assert_eq!(out.status.code(), Some(3), "{bad}");
        assert!(
            String::from_utf8_lossy(&out.stderr).contains("a qualified form"),
            "{bad}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    for missing in ["001#R-9", "999#R-1"] {
        let out = run_in(tmp.path(), &["registry", "obligation", missing]);
        assert_eq!(out.status.code(), Some(1), "{missing}");
    }
}
