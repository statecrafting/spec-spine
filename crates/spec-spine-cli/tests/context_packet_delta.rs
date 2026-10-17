//! Spec 164: `context delta` binds one verified Git tree and answers with a
//! delta or a fallback, and `context apply` reproduces the target pages or
//! writes nothing.

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

fn ok(out: &Output) -> String {
    assert_eq!(
        out.status.code(),
        Some(0),
        "stdout {} stderr {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout.clone()).unwrap()
}

/// A one-spec corpus with a committed ledger and two plain files.
fn fixture() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("specs/001-a");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("spec.md"),
        "---\nid: \"001-a\"\ntitle: \"T\"\nstatus: draft\ncreated: \"2026-10-15\"\n\
         summary: \"s\"\n---\n# 001\n\nThe body.\n",
    )
    .unwrap();
    fs::write(tmp.path().join("a.txt"), "alpha\n").unwrap();
    fs::write(tmp.path().join("b.txt"), "bravo\n").unwrap();
    ok(&bin(tmp.path(), &["compile"]));
    git(tmp.path(), &["init", "-q"]);
    git(
        tmp.path(),
        &["config", "user.email", "test@example.invalid"],
    );
    git(tmp.path(), &["config", "user.name", "Test"]);
    git(tmp.path(), &["config", "commit.gpgsign", "false"]);
    fs::write(tmp.path().join(".gitignore"), "build-meta.json\n").unwrap();
    git(tmp.path(), &["add", "-A"]);
    git(tmp.path(), &["commit", "-qm", "fixture"]);
    tmp
}

fn commit(root: &Path, file: &str, text: &str) {
    fs::write(root.join(file), text).unwrap();
    git(root, &["add", "-A"]);
    git(root, &["commit", "-qm", "change"]);
}

const REQUEST: &str = r#"{"closure":{"root":"001"},"members":[{"selector":{"kind":"file","path":"a.txt"},"requirement":"required"},{"selector":{"kind":"file","path":"b.txt"},"requirement":"optional"}],"maxItems":1,"consumerSchemaVersion":"1.1.0"}"#;

struct Files {
    dir: tempfile::TempDir,
}

impl Files {
    fn new() -> Self {
        Files {
            dir: tempfile::tempdir().unwrap(),
        }
    }

    fn put(&self, name: &str, text: &str) -> PathBuf {
        let path = self.dir.path().join(name);
        fs::write(&path, text).unwrap();
        path
    }

    fn path(&self, name: &str) -> PathBuf {
        self.dir.path().join(name)
    }
}

fn s(p: &Path) -> &str {
    p.to_str().unwrap()
}

/// Every page `context packet` writes for `request`, following continuations.
fn packet_pages(root: &Path, files: &Files, request: &str, tag: &str) -> Vec<PathBuf> {
    let mut pages = Vec::new();
    let mut req: serde_json::Value = serde_json::from_str(request).unwrap();
    loop {
        let req_path = files.put(&format!("{tag}-req-{}.json", pages.len()), &req.to_string());
        let out = bin(
            root,
            &[
                "context",
                "packet",
                "--request",
                s(&req_path),
                "--repository",
                "example/repo",
                "--json",
            ],
        );
        let text = ok(&out);
        let page: serde_json::Value = serde_json::from_str(&text).unwrap();
        pages.push(files.put(&format!("{tag}-page-{}.json", pages.len() + 1), &text));
        match page["continuation"].as_str() {
            Some(token) => req["continuation"] = token.into(),
            None => return pages,
        }
    }
}

fn delta_args<'a>(request: &'a Path, pages: &'a [PathBuf]) -> Vec<&'a str> {
    let mut args = vec!["context", "delta", "--request", s(request)];
    for p in pages {
        args.extend(["--predecessor", s(p)]);
    }
    args.extend(["--repository", "example/repo", "--json"]);
    args
}

#[test]
fn a_delta_applied_reproduces_the_packet_pages_byte_for_byte() {
    let repo = fixture();
    let files = Files::new();
    let request = files.put("request.json", REQUEST);
    let before = packet_pages(repo.path(), &files, REQUEST, "before");
    assert!(before.len() > 1, "the set spans pages");
    commit(repo.path(), "a.txt", "alpha, changed\n");

    let first = ok(&bin(repo.path(), &delta_args(&request, &before)));
    let again = ok(&bin(repo.path(), &delta_args(&request, &before)));
    assert_eq!(first, again, "a repeated delta is byte-identical");
    let doc: serde_json::Value = serde_json::from_str(&first).unwrap();
    assert_eq!(doc["outcome"], "delta");
    assert_eq!(doc["schemaVersion"], "1.0.0");
    assert_eq!(doc["changes"][0]["key"]["identity"], "file:a.txt");
    assert_eq!(doc["changes"][0]["change"], "changed");
    let delta = files.put("delta.json", &first);

    let out_dir = files.path("out");
    let mut args = vec!["context", "apply"];
    for p in &before {
        args.extend(["--predecessor", s(p)]);
    }
    args.extend(["--delta", s(&delta), "--out-dir", s(&out_dir), "--json"]);
    let written: Vec<String> = serde_json::from_str(&ok(&bin(repo.path(), &args))).unwrap();
    let after = packet_pages(repo.path(), &files, REQUEST, "after");
    assert_eq!(written.len(), after.len());
    for (w, a) in written.iter().zip(&after) {
        assert_eq!(fs::read(w).unwrap(), fs::read(a).unwrap(), "{w}");
    }
}

#[test]
fn a_fallback_is_an_answer_and_exits_zero() {
    let repo = fixture();
    let files = Files::new();
    let before = packet_pages(repo.path(), &files, REQUEST, "before");
    let other = files.put(
        "other.json",
        &REQUEST.replace(r#""maxItems":1"#, r#""maxItems":2"#),
    );
    let out = ok(&bin(repo.path(), &delta_args(&other, &before)));
    let doc: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(doc["outcome"], "fallback");
    assert_eq!(doc["reason"], "budget-changed");
    assert!(doc.get("changes").is_none());
}

#[test]
fn an_incomplete_target_is_a_finding_with_the_delta_in_its_envelope() {
    let repo = fixture();
    let files = Files::new();
    let request = files.put("request.json", REQUEST);
    let before = packet_pages(repo.path(), &files, REQUEST, "before");
    fs::remove_file(repo.path().join("a.txt")).unwrap();
    git(repo.path(), &["add", "-A"]);
    git(repo.path(), &["commit", "-qm", "drop a"]);
    let out = bin(repo.path(), &delta_args(&request, &before));
    assert_eq!(out.status.code(), Some(1));
    let envelope: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(envelope["verb"], "context.delta");
    assert_eq!(envelope["report"]["outcome"], "delta");
    assert_eq!(
        envelope["report"]["changes"][0]["change"],
        "newly-unresolved"
    );
}

#[test]
fn a_dirty_tree_is_refused() {
    let repo = fixture();
    let files = Files::new();
    let request = files.put("request.json", REQUEST);
    let before = packet_pages(repo.path(), &files, REQUEST, "before");
    fs::write(repo.path().join("a.txt"), "uncommitted\n").unwrap();
    let out = bin(repo.path(), &delta_args(&request, &before));
    assert_eq!(out.status.code(), Some(2));
    let envelope: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(envelope["verb"], "context.delta");
}

#[test]
fn apply_refuses_other_pages_and_a_non_empty_directory_writing_nothing() {
    let repo = fixture();
    let files = Files::new();
    let request = files.put("request.json", REQUEST);
    let before = packet_pages(repo.path(), &files, REQUEST, "before");
    commit(repo.path(), "a.txt", "alpha 2\n");
    let delta = files.put(
        "delta.json",
        &ok(&bin(repo.path(), &delta_args(&request, &before))),
    );
    let after = packet_pages(repo.path(), &files, REQUEST, "after");

    let apply = |pages: &[PathBuf], out_dir: &Path| {
        let mut args = vec!["context", "apply"];
        for p in pages {
            args.extend(["--predecessor", s(p)]);
        }
        args.extend(["--delta", s(&delta), "--out-dir", s(out_dir), "--json"]);
        bin(repo.path(), &args)
    };

    let out_dir = files.path("refused");
    let out = apply(&after, &out_dir);
    assert_eq!(out.status.code(), Some(2), "other bytes refuse");
    let envelope: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(envelope["verb"], "context.apply");
    assert!(!out_dir.exists(), "nothing is written");

    let busy = files.path("busy");
    fs::create_dir_all(&busy).unwrap();
    fs::write(busy.join("keep.txt"), "keep\n").unwrap();
    assert_eq!(apply(&before, &busy).status.code(), Some(3));
    assert_eq!(fs::read_dir(&busy).unwrap().count(), 1);

    let not_a_delta = files.put("fallback.json", r#"{"outcome":"fallback"}"#);
    let out = bin(
        repo.path(),
        &[
            "context",
            "apply",
            "--predecessor",
            s(&before[0]),
            "--delta",
            s(&not_a_delta),
            "--out-dir",
            s(&files.path("x")),
        ],
    );
    assert_eq!(out.status.code(), Some(3));
}

#[test]
fn both_subcommands_document_themselves() {
    let repo = fixture();
    for verb in ["delta", "apply"] {
        let out = bin(repo.path(), &["context", verb, "--help"]);
        assert_eq!(out.status.code(), Some(0));
        assert!(String::from_utf8_lossy(&out.stdout).contains("--predecessor"));
    }
}
