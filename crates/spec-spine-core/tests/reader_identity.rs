// Spec: specs/123-a-startup-verdict-names-its-reader/spec.md
//! The commit boundary names the reader behind a freshness refusal (spec 123
//! §3.4), exercised by running this repository's real `.githooks/pre-commit`
//! through `git commit` in a throwaway repository. The fixture is spec 122's
//! (`commit_boundary.rs`), restated rather than shared: an integration test
//! file is its own crate, and 122's acceptance pins that file's case count.
//!
//! The hook is registered with `core.hooksPath` pointing at the checked-in
//! directory, so what is tested is the file a contributor's clone runs, not a
//! copy. `PATH` is narrowed to the system directories plus a shim directory,
//! so an installed `spec-spine` or `cargo` on the machine cannot answer for
//! the hook: the freshness half (spec 094) is skipped as "absent", and the
//! formatting half sees only the shim this test writes.

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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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

/// Spec 123 §3.4: the refusal §14.8 of the 0.22.0 record describes. A fresh
/// worktree had no in-tree build, the hook fell back to an installed
/// `spec-spine 0.20.0` on `PATH`, and that reader's grammar predated the
/// corpus. The refusal stands (a read that did not pass is not a pass), and it
/// now names the executable that made it, so an incompatible reader is
/// distinguishable from a corpus found invalid by the right one.
#[test]
fn a_freshness_refusal_names_the_reader_that_made_it() {
    let repo = Repo::new();
    let shim = repo.shims.join("spec-spine");
    fs::write(
        &shim,
        "#!/bin/sh\ncase \"$*\" in\n  *--version*) echo 'spec-spine 0.20.0-on-path'; exit 0 ;;\nesac\n\
         echo '  V-002 [specs/106/spec.md] malformed frontmatter: extra-frontmatter lists must contain only strings'\n\
         exit 1\n",
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&shim, fs::Permissions::from_mode(0o755)).unwrap();
    }
    let before = repo.head();
    repo.write("notes.md", "changed\n");
    repo.git_ok(&["add", "-A"]);
    let out = repo.git(&["commit", "-q", "-m", "c"]);
    let t = text(&out);
    assert!(!out.status.success(), "{t}");
    assert_eq!(repo.head(), before, "{t}");
    assert!(
        t.contains(&format!(
            "the corpus does not validate, as read by {} (spec-spine 0.20.0-on-path)",
            shim.display()
        )),
        "{t}"
    );
    // The reader's own report is printed unchanged.
    assert!(t.contains("V-002 [specs/106/spec.md]"), "{t}");
}

/// Spec 123 §3.4: an in-tree build older than the source it is built from is
/// refused as that, not as a finding about the corpus.
#[test]
fn an_in_tree_reader_older_than_its_source_is_refused_as_that() {
    let repo = Repo::new();
    repo.write(".gitignore", "target/\n");
    repo.write("Cargo.toml", "[workspace]\n");
    let built = repo.root.join("target/release/spec-spine");
    fs::create_dir_all(built.parent().unwrap()).unwrap();
    fs::write(
        &built,
        "#!/bin/sh\ncase \"$*\" in\n  *--version*) echo 'spec-spine 0.22.0-in-tree'; exit 0 ;;\nesac\n\
         echo 'spec-registry: INVALID: the corpus fails validation'\nexit 1\n",
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&built, fs::Permissions::from_mode(0o755)).unwrap();
    }
    for (p, stamp) in [
        (&built, "202609220406"),
        (&repo.root.join("Cargo.toml"), "202609222001"),
    ] {
        assert!(
            Command::new("touch")
                .args(["-t", stamp])
                .arg(p)
                .status()
                .unwrap()
                .success()
        );
    }
    repo.git_ok(&["add", "-A"]);
    let out = repo.git(&["commit", "-q", "-m", "c"]);
    let t = text(&out);
    assert!(!out.status.success(), "{t}");
    assert!(
        t.contains("older than this checkout's source (Cargo.toml is newer)"),
        "{t}"
    );
    assert!(t.contains(&built.display().to_string()), "{t}");
    assert!(
        t.contains("spec-registry: INVALID"),
        "the report is kept: {t}"
    );
    assert!(!t.contains("resolve the violations printed above"), "{t}");
}

/// Spec 123 §3.5: the hook names a rebuild and never runs one. Every line of
/// the real hook that mentions `cargo build` is a message argument.
#[test]
fn the_hook_names_a_rebuild_and_never_runs_one() {
    let hook = fs::read_to_string(hooks_dir().join("pre-commit")).unwrap();
    let mut seen = 0;
    for line in hook.lines().filter(|l| l.contains("cargo build")) {
        seen += 1;
        let t = line.trim_start();
        assert!(
            t.starts_with('#') || t.starts_with("refuse ") || t.starts_with("echo "),
            "`cargo build` outside a message: {line}"
        );
    }
    assert!(
        seen > 0,
        "the remedy is named somewhere, or this asserts nothing"
    );
}
