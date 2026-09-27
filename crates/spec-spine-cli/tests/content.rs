//! Spec 155: first-party Git binding and error envelope.

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn git(root: &Path, args: &[&str]) {
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
}

fn fixture() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("selected.txt"), "selected\n").unwrap();
    git(tmp.path(), &["init", "-q"]);
    git(
        tmp.path(),
        &["config", "user.email", "test@example.invalid"],
    );
    git(tmp.path(), &["config", "user.name", "Test"]);
    git(tmp.path(), &["add", "selected.txt"]);
    git(tmp.path(), &["commit", "-qm", "fixture"]);
    tmp
}

fn run(root: &Path, request: &Path, revision: Option<&str>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_spec-spine"));
    command
        .arg("--repo")
        .arg(root)
        .args(["content", "select", "--request"])
        .arg(request)
        .args(["--repository", "example/repo", "--json"]);
    if let Some(revision) = revision {
        command.args(["--revision", revision]);
    }
    command.output().unwrap()
}

#[test]
fn clean_head_and_exported_revision_are_bound_to_git_objects() {
    let repo = fixture();
    let request_dir = tempfile::tempdir().unwrap();
    let request = request_dir.path().join("request.json");
    fs::write(
        &request,
        r#"{"selectors":[{"kind":"file","path":"selected.txt"}]}"#,
    )
    .unwrap();
    for revision in [None, Some("HEAD")] {
        let output = run(repo.path(), &request, revision);
        assert_eq!(
            output.status.code(),
            Some(0),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(value["items"][0]["content"], "selected\n");
        assert_eq!(value["repository"], "example/repo");
        assert_eq!(value["schemaVersion"], "0.9.0");
    }
}

#[test]
fn dirty_head_is_refused_in_the_json_error_envelope() {
    let repo = fixture();
    let request_dir = tempfile::tempdir().unwrap();
    let request = request_dir.path().join("request.json");
    fs::write(
        &request,
        r#"{"selectors":[{"kind":"file","path":"selected.txt"}]}"#,
    )
    .unwrap();
    fs::write(repo.path().join("selected.txt"), "dirty\n").unwrap();
    let output = run(repo.path(), &request, None);
    assert_eq!(output.status.code(), Some(2));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["verb"], "content.select");
    assert_eq!(value["outcome"], "refused");
}

#[test]
fn ignored_content_is_refused_even_when_the_working_tree_is_clean() {
    let repo = fixture();
    fs::write(repo.path().join(".gitignore"), "secret.txt\n").unwrap();
    git(repo.path(), &["add", ".gitignore"]);
    git(repo.path(), &["commit", "-qm", "ignore secret"]);
    fs::write(repo.path().join("secret.txt"), "credential\n").unwrap();

    let request_dir = tempfile::tempdir().unwrap();
    let request = request_dir.path().join("request.json");
    fs::write(
        &request,
        r#"{"selectors":[{"kind":"file","path":"secret.txt"}]}"#,
    )
    .unwrap();

    let output = run(repo.path(), &request, None);
    assert_eq!(output.status.code(), Some(2));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["verb"], "content.select");
    assert_eq!(value["outcome"], "refused");
    assert!(
        value["summary"]
            .as_str()
            .unwrap()
            .contains("ignored-content")
    );
}
