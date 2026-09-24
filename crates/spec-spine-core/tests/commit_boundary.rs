//! The commit boundary refuses an unresolved merge and an unformatted change
//! (spec 122), exercised by running this repository's real
//! `.githooks/pre-commit` through `git commit` in a throwaway repository.
//!
//! The hook is registered with `core.hooksPath` pointing at the checked-in
//! directory, so what is tested is the file a contributor's clone runs, not a
//! copy. `PATH` is narrowed to the system directories plus a shim directory,
//! so an installed `spec-spine` or `cargo` on the machine cannot answer for
//! the hook: the freshness half (spec 094) is skipped as "absent", and the
//! formatting half sees only the shim this test writes.

// Spec 134: this suite runs POSIX shell (WF-4 in docs/windows-findings.md),
// so it is compiled on Unix only; the Linux job runs it on every change.
#![cfg(unix)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn hooks_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(".githooks")
        .canonicalize()
        .unwrap()
}

struct Repo {
    _tmp: tempfile::TempDir,
    root: PathBuf,
    shims: PathBuf,
}

impl Repo {
    fn new() -> Self {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("repo");
        let shims = tmp.path().join("shims");
        fs::create_dir_all(root.join("specs")).unwrap();
        fs::create_dir_all(&shims).unwrap();
        let repo = Repo {
            _tmp: tmp,
            root,
            shims,
        };
        repo.git_ok(&["init", "-q", "-b", "main"]);
        for (k, v) in [
            ("user.name", "t"),
            ("user.email", "t@example.invalid"),
            ("commit.gpgsign", "false"),
            ("core.hooksPath", hooks_dir().to_str().unwrap()),
        ] {
            repo.git_ok(&["config", k, v]);
        }
        repo.write("specs/.keep", "");
        repo.write("notes.md", "base\n");
        repo.git_ok(&["add", "-A"]);
        repo.git_ok(&["commit", "-q", "-m", "base"]);
        repo
    }

    fn write(&self, rel: &str, body: &str) {
        let p = self.root.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, body).unwrap();
    }

    fn git(&self, args: &[&str]) -> Output {
        Command::new("git")
            .args(args)
            .current_dir(&self.root)
            .env("PATH", format!("{}:/usr/bin:/bin", self.shims.display()))
            .env_remove("SPEC_SPINE_BIN")
            .env_remove("GIT_DIR")
            .env_remove("GIT_INDEX_FILE")
            .output()
            .unwrap()
    }

    fn git_ok(&self, args: &[&str]) {
        let out = self.git(args);
        assert!(out.status.success(), "git {args:?}: {}", text(&out));
    }

    fn head(&self) -> String {
        String::from_utf8(self.git(&["rev-parse", "HEAD"]).stdout).unwrap()
    }

    /// A `cargo` shim whose `fmt` answers with `code`.
    fn cargo_fmt_exits(&self, code: i32) {
        let p = self.shims.join("cargo");
        fs::write(
            &p,
            format!("#!/bin/sh\necho \"shim cargo $*\"\nexit {code}\n"),
        )
        .unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&p, fs::Permissions::from_mode(0o755)).unwrap();
        }
    }

    /// Two branches that edit the same line, merged: the merge stops with
    /// `notes.md` unmerged and markers in the working tree.
    fn conflicted_merge(&self) {
        self.git_ok(&["checkout", "-q", "-b", "side"]);
        self.write("notes.md", "side\n");
        self.git_ok(&["commit", "-q", "-am", "side"]);
        self.git_ok(&["checkout", "-q", "main"]);
        self.write("notes.md", "main\n");
        self.git_ok(&["commit", "-q", "-am", "main"]);
        let out = self.git(&["merge", "side"]);
        assert!(!out.status.success(), "the fixture merge must conflict");
    }
}

fn text(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

#[test]
fn a_conflict_resolved_by_staging_the_markers_is_refused() {
    let repo = Repo::new();
    repo.conflicted_merge();
    // The failure this spec exists for: `git add` clears the unmerged stages
    // without reading the file, so git itself would now let the commit through.
    repo.git_ok(&["add", "notes.md"]);
    let before = repo.head();
    let out = repo.git(&["commit", "-q", "--no-edit"]);
    let said = text(&out);
    assert!(!out.status.success(), "committed a conflict: {said}");
    assert!(said.contains("conflict markers"), "{said}");
    assert!(said.contains("notes.md"), "{said}");
    assert_eq!(repo.head(), before, "HEAD moved");

    // Resolving the file is what lets the same merge commit through.
    repo.write("notes.md", "main and side\n");
    repo.git_ok(&["add", "notes.md"]);
    let out = repo.git(&["commit", "-q", "--no-edit"]);
    assert!(out.status.success(), "{}", text(&out));
    assert_ne!(repo.head(), before);
}

#[test]
fn unmerged_index_entries_are_refused_by_the_hook_itself() {
    let repo = Repo::new();
    repo.conflicted_merge();
    // git refuses this commit on its own; run the hook directly so the test
    // asserts the hook's check and not git's.
    let out = Command::new("sh")
        .arg(hooks_dir().join("pre-commit"))
        .current_dir(&repo.root)
        .env("PATH", format!("{}:/usr/bin:/bin", repo.shims.display()))
        .env_remove("SPEC_SPINE_BIN")
        .env_remove("GIT_DIR")
        .env_remove("GIT_INDEX_FILE")
        .output()
        .unwrap();
    let said = text(&out);
    assert_eq!(out.status.code(), Some(1), "{said}");
    assert!(said.contains("unmerged entries"), "{said}");
    assert!(said.contains("notes.md"), "{said}");
}

#[test]
fn a_fixture_that_declares_its_markers_is_not_refused() {
    let repo = Repo::new();
    let fixture = "<<<<<<< ours\nleft\n=======\nright\n>>>>>>> theirs\n";
    repo.write("fixtures/conflict.txt", fixture);
    let out = {
        repo.git_ok(&["add", "-A"]);
        repo.git(&["commit", "-q", "-m", "undeclared"])
    };
    assert!(!out.status.success(), "an undeclared marker file passed");

    // Declared by name, with git's own attribute: no pattern in the hook.
    repo.write(
        ".gitattributes",
        "fixtures/conflict.txt conflict-marker-size=200\n",
    );
    repo.git_ok(&["add", "-A"]);
    let out = repo.git(&["commit", "-q", "-m", "declared"]);
    assert!(out.status.success(), "{}", text(&out));
}

#[test]
fn a_marker_already_in_history_is_not_reported_again() {
    let repo = Repo::new();
    repo.write(
        ".gitattributes",
        "fixtures/conflict.txt conflict-marker-size=200\n",
    );
    repo.write("fixtures/conflict.txt", "<<<<<<< a\n=======\n>>>>>>> b\n");
    repo.git_ok(&["add", "-A"]);
    repo.git_ok(&["commit", "-q", "-m", "fixture"]);
    // An unrelated change is judged on what it adds, not on the tree.
    repo.write("notes.md", "unrelated\n");
    repo.git_ok(&["add", "-A"]);
    let out = repo.git(&["commit", "-q", "-m", "unrelated"]);
    assert!(out.status.success(), "{}", text(&out));
}

#[test]
fn a_failed_format_check_refuses_a_rust_change_and_a_passing_one_does_not() {
    let repo = Repo::new();
    repo.write("Cargo.toml", "[workspace]\n");
    repo.write("src/lib.rs", "fn a() {}\n");
    repo.git_ok(&["add", "-A"]);

    repo.cargo_fmt_exits(1);
    let before = repo.head();
    let out = repo.git(&["commit", "-q", "-m", "unformatted"]);
    let said = text(&out);
    assert!(!out.status.success(), "committed past a failed fmt: {said}");
    assert!(said.contains("cargo fmt --all --check failed"), "{said}");
    assert!(said.contains("shim cargo fmt --all --check"), "{said}");
    assert_eq!(repo.head(), before);

    repo.cargo_fmt_exits(0);
    let out = repo.git(&["commit", "-q", "-m", "formatted"]);
    assert!(out.status.success(), "{}", text(&out));
}

#[test]
fn a_change_with_no_rust_does_not_run_the_format_check() {
    let repo = Repo::new();
    repo.write("Cargo.toml", "[workspace]\n");
    repo.cargo_fmt_exits(1);
    repo.write("notes.md", "prose only\n");
    repo.git_ok(&["add", "-A"]);
    let out = repo.git(&["commit", "-q", "-m", "prose"]);
    assert!(out.status.success(), "{}", text(&out));
}
