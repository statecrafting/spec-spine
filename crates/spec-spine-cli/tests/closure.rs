// Spec: specs/107-a-context-closure-is-declared/spec.md
//! Spec 107 §3.5, §3.6: `registry closure` through the shipped binary: stdin
//! and file requests, the read document, and each exit code.

use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Output, Stdio};

fn run(root: &Path, args: &[&str], stdin: Option<&str>) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_spec-spine"))
        .arg("--repo")
        .arg(root)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn spec-spine");
    if let Some(text) = stdin {
        child
            .stdin
            .take()
            .unwrap()
            .write_all(text.as_bytes())
            .unwrap();
    } else {
        drop(child.stdin.take());
    }
    child.wait_with_output().unwrap()
}

fn corpus() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("specs/001-a");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("spec.md"),
        "---\nid: \"001-a\"\ntitle: \"T\"\nstatus: draft\ncreated: \"2026-09-22\"\nsummary: \"s\"\n\
         obligations:\n  - id: \"R-1\"\n    kind: requirement\n    text: \"The rule holds.\"\n    anchor: \"3-1-the-rule\"\n\
         ---\n# 001\n\n## 3. Behavior\n\n### 3.1 The rule\n\nText.\n",
    )
    .unwrap();
    let out = run(tmp.path(), &["compile"], None);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    tmp
}

const REQUEST: &str = r#"{"specs":["001"],"obligations":["001#R-1"],"rationale":"r"}"#;

#[test]
fn stdin_and_a_file_give_the_same_read_document() {
    let tmp = corpus();
    let from_stdin = run(
        tmp.path(),
        &["registry", "closure", "--request", "-", "--json"],
        Some(REQUEST),
    );
    assert_eq!(
        from_stdin.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&from_stdin.stderr)
    );
    let req = tmp.path().join("closure.json");
    fs::write(&req, REQUEST).unwrap();
    let from_file = run(
        tmp.path(),
        &[
            "registry",
            "closure",
            "--request",
            req.to_str().unwrap(),
            "--json",
        ],
        None,
    );
    assert_eq!(from_file.stdout, from_stdin.stdout);
    let doc: serde_json::Value = serde_json::from_slice(&from_stdin.stdout).unwrap();
    assert_eq!(doc["schemaVersion"], spec_spine_types::READ_SCHEMA_VERSION);
    assert_eq!(doc["rationale"], "r");
    assert_eq!(doc["members"].as_array().unwrap().len(), 2);
    assert_eq!(doc["members"][1]["kind"], "spec");
    // The spec member's identity is the committed shard's hash.
    let shard: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(tmp.path().join(".derived/spec-registry/by-spec/001-a.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(doc["members"][1]["contentHash"], shard["shardHash"]);

    // The text form ends with the same digest.
    let text = run(
        tmp.path(),
        &["registry", "closure", "--request", "-"],
        Some(REQUEST),
    );
    let text = String::from_utf8_lossy(&text.stdout);
    assert!(
        text.trim_end()
            .ends_with(&format!("digest: {}", doc["digest"].as_str().unwrap())),
        "{text}"
    );
}

#[test]
fn each_refusal_has_its_exit_code() {
    let tmp = corpus();
    let code = |stdin: &str| {
        run(
            tmp.path(),
            &["registry", "closure", "--request", "-"],
            Some(stdin),
        )
        .status
        .code()
    };
    assert_eq!(code("{}"), Some(3), "an empty closure");
    assert_eq!(code(r#"{"obligations":["R-1"]}"#), Some(3), "unqualified");
    assert_eq!(code(r#"{"spec":["001"]}"#), Some(3), "an unknown member");
    assert_eq!(code("not json"), Some(3), "unparseable");
    assert_eq!(code(r#"{"specs":["999"]}"#), Some(1), "unresolved");

    // Edited and not recompiled: the ledger no longer vouches for the corpus.
    let spec = tmp.path().join("specs/001-a/spec.md");
    let text = fs::read_to_string(&spec).unwrap();
    fs::write(&spec, text.replace("Text.", "Edited.")).unwrap();
    assert_eq!(code(REQUEST), Some(2), "stale");
}
