//! Spec 165: `edit-plan` binds one verified Git tree and answers with a ready
//! plan, a refused plan as a finding, or a refusal envelope. It writes
//! nothing.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn git(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

fn bin(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_spec-spine"))
        .arg("--repo")
        .arg(root)
        .args(args)
        .output()
        .unwrap()
}

const SPEC: &str = "---\nid: \"001-a\"\ntitle: \"T\"\nstatus: draft\ncreated: \"2026-10-15\"\n\
implementation: pending\nsummary: \"s\"\nestablishes:\n  - { kind: file, path: \"docs/a.md\" }\n---\n\
# 001\n\n## 3. Behavior\n\n### 3.1 Notes\n\nNotes.\n";

/// A one-spec corpus with a committed ledger, under Git.
fn fixture() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("specs/001-a")).unwrap();
    fs::create_dir_all(root.join("docs")).unwrap();
    fs::write(root.join("specs/001-a/spec.md"), SPEC).unwrap();
    fs::write(root.join("docs/a.md"), "# A\n").unwrap();
    fs::write(root.join("docs/b.md"), "# B\n").unwrap();
    for verb in ["compile", "index"] {
        let out = bin(root, &[verb]);
        assert_eq!(
            out.status.code(),
            Some(0),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    git(root, &["init", "-q"]);
    git(root, &["config", "user.email", "test@example.invalid"]);
    git(root, &["config", "user.name", "Test"]);
    git(root, &["config", "commit.gpgsign", "false"]);
    fs::write(root.join(".gitignore"), "build-meta.json\n").unwrap();
    git(root, &["add", "-A"]);
    git(root, &["commit", "-qm", "fixture"]);
    tmp
}

/// The spec 155 item digest of spec `001-a` at its full projection, read
/// through `content select` on the same tree.
fn spec_digest(root: &Path, dir: &Path) -> String {
    let req = dir.join("content.json");
    fs::write(&req, r#"{"selectors":[{"kind":"spec","spec":"001-a"}]}"#).unwrap();
    let out = bin(
        root,
        &[
            "content",
            "select",
            "--request",
            req.to_str().unwrap(),
            "--repository",
            "example/repo",
            "--json",
        ],
    );
    assert!(
        out.status.success(),
        "content select failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let doc: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    doc["items"][0]["digest"].as_str().unwrap().to_string()
}

fn request(dir: &Path, unit_path: &str, expected: &str) -> PathBuf {
    let path = dir.join("plan.json");
    let req = serde_json::json!({
        "scope": { "ownSpec": "001", "mutable": ["specs/001-a/", "docs/"] },
        "operations": [{
            "op": "add-claim",
            "target": { "kind": "spec", "spec": "001-a" },
            "expected": expected,
            "edge": "establishes",
            "unit": { "kind": "file", "path": unit_path },
        }],
        "consumerSchemaVersion": "1.0.0",
    });
    fs::write(&path, req.to_string()).unwrap();
    path
}

fn plan(root: &Path, req: &Path) -> Output {
    bin(
        root,
        &[
            "edit-plan",
            "--request",
            req.to_str().unwrap(),
            "--repository",
            "example/repo",
            "--json",
        ],
    )
}

fn files(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for e in fs::read_dir(dir).unwrap() {
            let p = e.unwrap().path();
            if p.file_name().is_some_and(|n| n == ".git") {
                continue;
            }
            if p.is_dir() {
                stack.push(p);
            } else {
                out.push((p.clone(), fs::read(&p).unwrap()));
            }
        }
    }
    out.sort();
    out
}

#[test]
fn a_ready_plan_is_the_bare_document_and_writes_nothing() {
    let repo = fixture();
    let dir = tempfile::tempdir().unwrap();
    let before = files(repo.path());
    let req = request(
        dir.path(),
        "docs/b.md",
        &spec_digest(repo.path(), dir.path()),
    );
    let out = plan(repo.path(), &req);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let again = plan(repo.path(), &req);
    assert_eq!(
        out.stdout, again.stdout,
        "repeated plans are byte-identical"
    );
    let doc: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(doc["status"], "ready");
    assert_eq!(doc["subject"], "001-a");
    assert_eq!(
        doc["snapshot"]["revision"],
        git(repo.path(), &["rev-parse", "HEAD"])
    );
    assert!(
        doc["previews"][0]["afterContent"]
            .as_str()
            .unwrap()
            .contains("  - { kind: file, path: \"docs/b.md\" }\n")
    );
    assert_eq!(files(repo.path()), before, "planning writes nothing");
    assert_eq!(git(repo.path(), &["status", "--porcelain"]), "");
}

#[test]
fn a_refused_plan_is_a_finding_whose_envelope_carries_it() {
    let repo = fixture();
    let dir = tempfile::tempdir().unwrap();
    let req = request(
        dir.path(),
        "docs/a.md",
        &spec_digest(repo.path(), dir.path()),
    );
    let out = plan(repo.path(), &req);
    assert_eq!(out.status.code(), Some(1));
    let envelope: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(envelope["verb"], "edit-plan");
    assert_eq!(envelope["report"]["status"], "refused");
    assert_eq!(
        envelope["report"]["refusals"][0]["reason"],
        "duplicate-declaration"
    );
    assert!(
        envelope["report"]["previews"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn a_dirty_tree_refuses_and_a_malformed_request_is_usage() {
    let repo = fixture();
    let dir = tempfile::tempdir().unwrap();
    let req = request(
        dir.path(),
        "docs/b.md",
        &spec_digest(repo.path(), dir.path()),
    );
    fs::write(repo.path().join("docs/a.md"), "# A, uncommitted\n").unwrap();
    assert_eq!(plan(repo.path(), &req).status.code(), Some(2));
    git(repo.path(), &["checkout", "--", "docs/a.md"]);

    let bad = dir.path().join("bad.json");
    fs::write(
        &bad,
        r#"{"scope":{"ownSpec":"001"},"operations":[{"op":"apply","target":"a","expected":"b"}],"consumerSchemaVersion":"1.0.0"}"#,
    )
    .unwrap();
    let out = plan(repo.path(), &bad);
    assert_eq!(out.status.code(), Some(3));
    let envelope: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(envelope["verb"], "edit-plan");
}

#[test]
fn the_facade_matches_the_cli() {
    let repo = fixture();
    let dir = tempfile::tempdir().unwrap();
    let req = request(
        dir.path(),
        "docs/b.md",
        &spec_digest(repo.path(), dir.path()),
    );
    let out = plan(repo.path(), &req);
    let doc: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let mut snapshot = doc["snapshot"].clone();
    snapshot["binding"] = "caller-supplied".into();
    // The CLI binds the clean working tree at HEAD; the facade is handed the
    // same snapshot and the same bytes.
    let facade = spec_spine_core::edit_plan_json(
        &serde_json::to_string(&spec_spine_types::Config::default()).unwrap(),
        repo.path().to_str().unwrap(),
        &fs::read_to_string(&req).unwrap(),
        &snapshot.to_string(),
    )
    .unwrap();
    assert_eq!(facade.as_bytes(), out.stdout.as_slice());
}

#[test]
fn there_is_no_apply_flag() {
    let repo = fixture();
    let out = bin(repo.path(), &["edit-plan", "--help"]);
    assert_eq!(out.status.code(), Some(0));
    let help = String::from_utf8_lossy(&out.stdout);
    assert!(help.contains("--request"));
    assert!(!help.contains("--apply"));
}
