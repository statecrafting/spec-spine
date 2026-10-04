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

fn run_in(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_spec-spine"))
        .arg("--repo")
        .arg(root)
        .args(args)
        .output()
        .unwrap()
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
        assert_eq!(value["schemaVersion"], "0.10.0");
    }
}

#[test]
fn exported_content_ignores_checkout_filters() {
    let repo = fixture();
    fs::write(
        repo.path().join(".gitattributes"),
        "selected.txt text eol=crlf\n",
    )
    .unwrap();
    git(repo.path(), &["add", ".gitattributes"]);
    git(
        repo.path(),
        &["commit", "-qm", "declare checkout conversion"],
    );
    let request_dir = tempfile::tempdir().unwrap();
    let request = request_dir.path().join("request.json");
    fs::write(
        &request,
        r#"{"selectors":[{"kind":"file","path":"selected.txt"}]}"#,
    )
    .unwrap();

    let output = run(repo.path(), &request, Some("HEAD"));
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["items"][0]["content"], "selected\n");
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
fn ignored_content_cannot_enter_a_clean_head_response() {
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
    assert_eq!(output.status.code(), Some(0));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(value["items"].as_array().unwrap().is_empty());
    assert_eq!(value["omissions"][0]["reason"], "missing-content");
    assert!(!String::from_utf8_lossy(&output.stdout).contains("credential"));
}

#[test]
fn normalized_ignored_directory_member_cannot_enter_the_response() {
    let repo = fixture();
    fs::create_dir(repo.path().join("private")).unwrap();
    fs::write(repo.path().join(".gitignore"), "private/secret.txt\n").unwrap();
    git(repo.path(), &["add", ".gitignore"]);
    git(repo.path(), &["commit", "-qm", "ignore directory member"]);
    fs::write(repo.path().join("private/secret.txt"), "credential\n").unwrap();

    let request_dir = tempfile::tempdir().unwrap();
    let request = request_dir.path().join("request.json");
    fs::write(
        &request,
        r#"{"selectors":[{"kind":"directory-member","directory":"./private/","member":"./secret.txt"}]}"#,
    )
    .unwrap();

    let output = run(repo.path(), &request, None);
    assert_eq!(output.status.code(), Some(0));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["omissions"][0]["reason"], "missing-content");
    assert!(!String::from_utf8_lossy(&output.stdout).contains("credential"));
}

#[test]
fn head_reads_committed_bytes_hidden_by_skip_worktree() {
    let repo = fixture();
    git(
        repo.path(),
        &["update-index", "--skip-worktree", "selected.txt"],
    );
    fs::write(repo.path().join("selected.txt"), "not in tree\n").unwrap();
    let request_dir = tempfile::tempdir().unwrap();
    let request = request_dir.path().join("request.json");
    fs::write(
        &request,
        r#"{"selectors":[{"kind":"file","path":"selected.txt"}]}"#,
    )
    .unwrap();

    let output = run(repo.path(), &request, None);
    assert_eq!(output.status.code(), Some(0));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["items"][0]["content"], "selected\n");
}

#[cfg(unix)]
#[test]
fn tracked_symlink_cannot_reveal_an_ignored_worktree_target() {
    use std::os::unix::fs::symlink;

    let repo = fixture();
    fs::write(repo.path().join(".gitignore"), "secret.txt\n").unwrap();
    symlink("secret.txt", repo.path().join("link.txt")).unwrap();
    git(repo.path(), &["add", ".gitignore", "link.txt"]);
    git(repo.path(), &["commit", "-qm", "add link"]);
    fs::write(repo.path().join("secret.txt"), "TOPSECRET\n").unwrap();
    let request_dir = tempfile::tempdir().unwrap();
    let request = request_dir.path().join("request.json");
    fs::write(
        &request,
        r#"{"selectors":[{"kind":"file","path":"link.txt"}]}"#,
    )
    .unwrap();

    let output = run(repo.path(), &request, None);
    assert_eq!(output.status.code(), Some(0));
    assert!(!String::from_utf8_lossy(&output.stdout).contains("TOPSECRET"));
}

#[test]
fn revision_export_accepts_a_gitlink() {
    let child = fixture();
    let repo = fixture();
    let output = Command::new("git")
        .arg("-c")
        .arg("protocol.file.allow=always")
        .arg("-C")
        .arg(repo.path())
        .args(["submodule", "add", "-q"])
        .arg(child.path())
        .arg("vendor/sub")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    git(repo.path(), &["commit", "-qam", "add submodule"]);
    let request_dir = tempfile::tempdir().unwrap();
    let request = request_dir.path().join("request.json");
    fs::write(
        &request,
        r#"{"selectors":[{"kind":"file","path":"selected.txt"}]}"#,
    )
    .unwrap();

    let output = run(repo.path(), &request, Some("HEAD"));
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn governed_selectors_return_bounded_content_from_one_fixture() {
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir_all(tmp.path().join("specs/001-fixture")).unwrap();
    fs::create_dir_all(tmp.path().join("src")).unwrap();
    fs::write(
        tmp.path().join("Cargo.toml"),
        "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[package.metadata.spec-spine]\nspec = \"001-fixture\"\n",
    )
    .unwrap();
    fs::write(
        tmp.path().join("src/lib.rs"),
        "/// A documented type.\n#[derive(Clone)]\npub struct Thing {\n    value: u8,\n}\n\npub mod nested {\n    pub fn action() {\n        let _value = 1;\n    }\n}\n",
    )
    .unwrap();
    fs::write(
        tmp.path().join("specs/001-fixture/spec.md"),
        "---\nid: \"001-fixture\"\ntitle: \"Fixture\"\nstatus: draft\ncreated: \"2026-09-27\"\nimplementation: complete\nsummary: \"fixture\"\nestablishes:\n  - \"src/lib.rs\"\nobligations:\n  - id: \"R-1\"\n    kind: requirement\n    text: \"The fixture holds.\"\n    anchor: \"3-1-rule\"\nintent:\n  goal: \"exercise selectors\"\n---\n# Fixture\n\n## 3. Behavior\n\n### 3.1 Rule\n\nBounded body.\n",
    )
    .unwrap();
    for verb in ["compile", "index"] {
        let output = run_in(tmp.path(), &[verb]);
        assert_eq!(
            output.status.code(),
            Some(0),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    git(tmp.path(), &["init", "-q"]);
    git(
        tmp.path(),
        &["config", "user.email", "test@example.invalid"],
    );
    git(tmp.path(), &["config", "user.name", "Test"]);
    git(tmp.path(), &["add", "."]);
    git(tmp.path(), &["commit", "-qm", "fixture"]);
    let request_dir = tempfile::tempdir().unwrap();
    let request = request_dir.path().join("request.json");
    fs::write(
        &request,
        serde_json::to_vec(&serde_json::json!({
            "selectors": [
                {"kind":"spec", "spec":"001", "projection":"body"},
                {"kind":"spec-section", "spec":"001", "anchor":"3-1-rule"},
                {"kind":"obligation", "obligation":"001#R-1", "projection":"declaration"},
                {"kind":"owned-unit", "spec":"001", "unit":{"kind":"file", "path":"src/lib.rs"}},
                {"kind":"symbol", "id":"fixture::Thing", "projection":"documentation"},
                {"kind":"module", "id":"fixture::nested"}
            ]
        }))
        .unwrap(),
    )
    .unwrap();

    let first = run(tmp.path(), &request, None);
    let second = run(tmp.path(), &request, None);
    assert_eq!(
        first.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert_eq!(first.stdout, second.stdout);
    let value: serde_json::Value = serde_json::from_slice(&first.stdout).unwrap();
    assert_eq!(value["completeness"], "complete", "{value}");
    assert_eq!(value["items"].as_array().map(Vec::len), Some(6), "{value}");
    assert!(
        value["items"].as_array().unwrap().iter().any(|item| {
            item["identity"] == "symbol:fixture::Thing"
                && item["content"] == "/// A documented type."
        }),
        "{value}"
    );
}
