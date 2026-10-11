//! Spec 159: `context packet` binds one verified Git tree and answers with a
//! packet, an incomplete-packet finding, or a refusal envelope.

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

/// A one-spec corpus with a committed ledger and one plain file.
fn fixture() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("specs/001-a");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("spec.md"),
        "---\nid: \"001-a\"\ntitle: \"T\"\nstatus: draft\ncreated: \"2026-10-10\"\n\
         summary: \"s\"\n---\n# 001\n\nThe body.\n",
    )
    .unwrap();
    fs::write(tmp.path().join("selected.txt"), "selected\n").unwrap();
    let compiled = bin(tmp.path(), &["compile"]);
    assert_eq!(
        compiled.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    git(tmp.path(), &["init", "-q"]);
    git(
        tmp.path(),
        &["config", "user.email", "test@example.invalid"],
    );
    git(tmp.path(), &["config", "user.name", "Test"]);
    git(tmp.path(), &["config", "commit.gpgsign", "false"]);
    // build-meta.json carries the wall clock and is never committed.
    fs::write(tmp.path().join(".gitignore"), "build-meta.json\n").unwrap();
    git(tmp.path(), &["add", "-A"]);
    git(tmp.path(), &["commit", "-qm", "fixture"]);
    tmp
}

fn request_file(text: &str) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("request.json");
    fs::write(&path, text).unwrap();
    (dir, path)
}

fn packet(root: &Path, request: &Path, revision: Option<&str>) -> Output {
    let mut args = vec![
        "context",
        "packet",
        "--request",
        request.to_str().unwrap(),
        "--repository",
        "example/repo",
        "--json",
    ];
    if let Some(rev) = revision {
        args.extend(["--revision", rev]);
    }
    bin(root, &args)
}

fn json(out: &Output) -> serde_json::Value {
    serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!(
            "{e}: stdout {} stderr {}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        )
    })
}

const ROOT_AND_FILE: &str = r#"{"closure":{"root":"001"},"members":[{"selector":{"kind":"file","path":"selected.txt"},"requirement":"required"}],"consumerSchemaVersion":"1.0.0"}"#;

#[test]
fn a_clean_head_and_an_exported_revision_bind_the_packet_to_git_objects() {
    let repo = fixture();
    let (_dir, request) = request_file(ROOT_AND_FILE);
    let head = git(repo.path(), &["rev-parse", "HEAD"]);
    let tree = git(repo.path(), &["rev-parse", "HEAD^{tree}"]);
    for (revision, dirty) in [(None, "clean-working-tree"), (Some("HEAD"), "clean-export")] {
        let out = packet(repo.path(), &request, revision);
        assert_eq!(
            out.status.code(),
            Some(0),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let doc = json(&out);
        assert_eq!(doc["schemaVersion"], "1.0.0");
        assert_eq!(doc["snapshot"]["revision"], head);
        assert_eq!(doc["snapshot"]["tree"], tree);
        assert_eq!(doc["snapshot"]["dirtyState"], dirty);
        assert_eq!(doc["completeness"], "complete");
        let identities: Vec<&str> = doc["members"]
            .as_array()
            .unwrap()
            .iter()
            .map(|m| m["identity"].as_str().unwrap())
            .collect();
        assert_eq!(identities, vec!["file:selected.txt", "spec:001-a"]);
        assert_eq!(doc["members"][0]["item"]["revision"], head);
        // The binding supplies its own build digest (D-5).
        let build = doc["producer"]["build"].as_str().unwrap();
        assert!(build.starts_with("sha256:") && build.len() == 71, "{build}");
        assert!(doc["warnings"].as_array().unwrap().is_empty());
        assert!(out.stderr.is_empty());
    }
}

#[test]
fn a_repeated_read_is_byte_identical() {
    let repo = fixture();
    let (_dir, request) = request_file(ROOT_AND_FILE);
    let a = packet(repo.path(), &request, None);
    let b = packet(repo.path(), &request, None);
    assert_eq!(a.status.code(), Some(0));
    assert_eq!(a.stdout, b.stdout);
}

#[test]
fn a_required_omission_is_a_finding_whose_envelope_carries_the_packet() {
    let repo = fixture();
    let (_dir, request) = request_file(
        r#"{"closure":{"root":"001"},"members":[{"selector":{"kind":"file","path":"gone.txt"},"requirement":"required"}],"consumerSchemaVersion":"1.0.0"}"#,
    );
    let out = packet(repo.path(), &request, None);
    assert_eq!(out.status.code(), Some(1));
    let v = json(&out);
    assert_eq!(v["verb"], "context.packet");
    assert_eq!(v["outcome"], "finding");
    assert_eq!(v["exitCode"], 1);
    assert_eq!(v["report"]["completeness"], "incomplete");
    assert_eq!(v["report"]["omissions"][0]["identity"], "file:gone.txt");
    assert_eq!(v["report"]["omissions"][0]["reason"], "missing");
    assert!(v.get("error").is_none());
}

#[test]
fn a_dirty_tree_is_refused_and_returns_no_packet() {
    let repo = fixture();
    let (_dir, request) = request_file(ROOT_AND_FILE);
    fs::write(repo.path().join("selected.txt"), "dirty\n").unwrap();
    let out = packet(repo.path(), &request, None);
    assert_eq!(out.status.code(), Some(2));
    let v = json(&out);
    assert_eq!(v["verb"], "context.packet");
    assert_eq!(v["outcome"], "refused");
    assert!(v.get("report").is_none());
    assert!(!String::from_utf8_lossy(&out.stdout).contains("dirty\\n"));
}

#[test]
fn a_malformed_request_and_an_unsupported_schema_answer_with_envelopes() {
    let repo = fixture();
    for (text, code, outcome) in [
        (
            r#"{"closure":{"root":"001"},"consumerSchemaVersion":"1.0.0","extra":true}"#,
            3,
            "usage",
        ),
        (
            r#"{"closure":{"root":"001"},"consumerSchemaVersion":"2.0.0"}"#,
            2,
            "refused",
        ),
        (
            r#"{"closure":{"root":"001"},"consumerSchemaVersion":"1.0.0","continuation":"not-a-token"}"#,
            3,
            "usage",
        ),
    ] {
        let (_dir, request) = request_file(text);
        let out = packet(repo.path(), &request, None);
        assert_eq!(out.status.code(), Some(code), "{text}");
        let v = json(&out);
        assert_eq!(v["verb"], "context.packet", "{text}");
        assert_eq!(v["outcome"], outcome, "{text}");
        assert!(out.stderr.is_empty(), "{text}");
    }
}

#[test]
fn capabilities_name_the_verb_and_its_own_axis() {
    let out = Command::new(env!("CARGO_BIN_EXE_spec-spine"))
        .args(["capabilities", "--json"])
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let verb = v["verbs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|verb| verb["path"] == "context packet")
        .expect("context packet is listed");
    assert_eq!(verb["json"][0]["axis"], "context-packet");
    assert_eq!(verb["json"][0]["version"], "1.0.0");
}
