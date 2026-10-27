//! Spec 206: `spec-spine resolve-derived`, end to end.
//!
//! Every test builds a scratch repository, stops a real merge, rebase,
//! cherry-pick or stash pop on a conflict, and runs the built binary at the
//! stop. Each asserts what the verb wrote (or did not) through git itself, and
//! that the operation is still in progress afterwards (acceptance 15).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_spec-spine")
}

/// A command isolated from the caller's git configuration and state, so a
/// hook, a signing rule or an enclosing `spec-spine verify` cannot change
/// what a test observes.
fn command(program: &str, dir: &Path) -> Command {
    let mut c = Command::new(program);
    c.current_dir(dir);
    for var in [
        "GIT_DIR",
        "GIT_INDEX_FILE",
        "GIT_WORK_TREE",
        "GIT_OBJECT_DIRECTORY",
        "GIT_ALTERNATE_OBJECT_DIRECTORIES",
        "SPEC_SPINE_PR_BODY",
        "SPEC_SPINE_VERIFY_STACK",
    ] {
        c.env_remove(var);
    }
    c.env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_EDITOR", "true")
        .env("GIT_MERGE_AUTOEDIT", "no");
    c
}

fn git_out(repo: &Path, args: &[&str]) -> Output {
    command("git", repo).args(args).output().expect("git runs")
}

fn git(repo: &Path, args: &[&str]) -> String {
    let out = git_out(repo, args);
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// Run a git command expected to stop on a conflict.
fn git_stops(repo: &Path, args: &[&str]) {
    let out = git_out(repo, args);
    assert!(
        !out.status.success(),
        "git {args:?} was expected to stop on a conflict"
    );
}

fn spine(repo: &Path, args: &[&str]) -> Output {
    command(bin(), repo)
        .args(args)
        .output()
        .expect("binary runs")
}

fn spine_ok(repo: &Path, args: &[&str]) {
    let out = spine(repo, args);
    assert!(
        out.status.success(),
        "spec-spine {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn write(repo: &Path, path: &str, text: &str) {
    let p = repo.join(path);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, text).unwrap();
}

fn spec(id: &str, title: &str, claims: &str, body: &str) -> String {
    format!(
        "---\nid: \"{id}\"\ntitle: \"{title}\"\nstatus: approved\nimplementation: pending\n\
         created: \"2026-10-27\"\nsummary: \"The {id} fixture.\"\nestablishes:\n  - \"{claims}\"\n\
         ---\n\n# {id}\n\n## Behavior\n\n{body}"
    )
}

const BODY: &str = "Line one.\n\nLine two.\n\nLine three.\n\nLine four.\n\nLine five.\n";

/// A committed corpus on `main`, compiled and indexed, with `derived` as the
/// configured derived root (`None`: the default).
struct Repo {
    tmp: tempfile::TempDir,
    derived: String,
}

impl Repo {
    fn new(derived: Option<&str>) -> Repo {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path();
        let mut toml = String::from("[layout]\nspecs_dir = \"specs\"\n");
        if let Some(d) = derived {
            toml.push_str(&format!("derived_dir = \"{d}\"\n"));
        }
        write(repo, "spec-spine.toml", &toml);
        write(repo, ".gitignore", "build-meta.json\n");
        write(
            repo,
            "specs/001-alpha/spec.md",
            &spec("001-alpha", "Alpha", "src/alpha.rs", BODY),
        );
        write(
            repo,
            "specs/002-beta/spec.md",
            &spec("002-beta", "Beta", "src/beta.rs", BODY),
        );
        write(repo, "src/alpha.rs", "pub fn alpha() {}\n");
        write(repo, "src/beta.rs", "pub fn beta() {}\n");
        write(repo, "notes.txt", "x\n");
        // One package, so the indexer reads `// Spec:` comment headers.
        write(
            repo,
            "Cargo.toml",
            "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
        git(repo, &["init", "-q", "-b", "main"]);
        for (k, v) in [
            ("user.name", "Fixture"),
            ("user.email", "fixture@example.invalid"),
            ("commit.gpgsign", "false"),
            ("core.autocrlf", "false"),
        ] {
            git(repo, &["config", k, v]);
        }
        let r = Repo {
            tmp,
            derived: derived.unwrap_or(".derived").to_string(),
        };
        r.commit("base");
        r
    }

    fn path(&self) -> &Path {
        self.tmp.path()
    }

    /// Regenerate with the binary under test, then commit everything.
    fn commit(&self, message: &str) {
        spine_ok(self.path(), &["compile"]);
        spine_ok(self.path(), &["index"]);
        git(self.path(), &["add", "-A"]);
        git(self.path(), &["commit", "-qm", message]);
    }

    fn edit(&self, path: &str, from: &str, to: &str) {
        let p = self.path().join(path);
        let text = fs::read_to_string(&p).unwrap();
        assert!(text.contains(from), "{path} lacks {from:?}");
        fs::write(&p, text.replacen(from, to, 1)).unwrap();
    }

    fn append(&self, path: &str, line: &str) {
        let p = self.path().join(path);
        let mut text = fs::read_to_string(&p).unwrap();
        text.push_str(line);
        fs::write(&p, text).unwrap();
    }

    /// `main` and `feature` diverge from the base: `main` runs `on_main`,
    /// `feature` runs `on_feature`, each regenerated and committed. Leaves
    /// `feature` checked out.
    fn diverge(&self, on_main: impl Fn(&Repo), on_feature: impl Fn(&Repo)) {
        git(self.path(), &["checkout", "-q", "-b", "feature"]);
        on_feature(self);
        self.commit("feature");
        git(self.path(), &["checkout", "-q", "main"]);
        on_main(self);
        self.commit("main");
        git(self.path(), &["checkout", "-q", "feature"]);
    }

    /// Both sides edit spec 001, far apart, so the spec merges cleanly and
    /// its registry shard conflicts.
    fn same_shard(&self) {
        self.diverge(
            |r| {
                r.edit(
                    "specs/001-alpha/spec.md",
                    "title: \"Alpha\"",
                    "title: \"Alpha on main\"",
                )
            },
            |r| r.append("specs/001-alpha/spec.md", "\nA feature line.\n"),
        );
    }

    fn resolve(&self, json: bool) -> (i32, Value, String) {
        let mut args = vec!["resolve-derived"];
        if json {
            args.push("--json");
        }
        let out = spine(self.path(), &args);
        let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
        let doc = if json {
            serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("{e}: {stdout}\n{stderr}"))
        } else {
            Value::Null
        };
        (
            out.status.code().unwrap_or(-1),
            doc,
            format!("{stdout}{stderr}"),
        )
    }

    /// The whole index, every stage of every entry.
    fn index(&self) -> String {
        git(self.path(), &["ls-files", "-s"])
    }

    /// The index outside the derived root.
    fn authored_index(&self) -> String {
        git(
            self.path(),
            &[
                "ls-files",
                "-s",
                "--",
                &format!(":(exclude){}", self.derived),
            ],
        )
    }

    fn unmerged(&self) -> String {
        git(self.path(), &["ls-files", "--unmerged"])
    }

    fn unmerged_derived(&self) -> String {
        git(
            self.path(),
            &["ls-files", "--unmerged", "--", &self.derived],
        )
    }

    /// Every file under `root`, with its bytes.
    fn files(&self, root: &str) -> BTreeMap<PathBuf, Vec<u8>> {
        fn walk(dir: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
            let Ok(entries) = fs::read_dir(dir) else {
                return;
            };
            for e in entries.flatten() {
                if e.file_type().unwrap().is_dir() {
                    walk(&e.path(), out);
                } else {
                    out.insert(e.path(), fs::read(e.path()).unwrap());
                }
            }
        }
        let mut out = BTreeMap::new();
        walk(&self.path().join(root), &mut out);
        out
    }

    /// Acceptance 15: the stopped operation is still where it was.
    fn still_in_progress(&self, head: &str) {
        let out = git_out(self.path(), &["rev-parse", "-q", "--verify", head]);
        assert!(
            out.status.success(),
            "{head} is gone: the verb ended the operation"
        );
    }
}

fn paths(v: &Value) -> Vec<String> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|p| p.as_str().unwrap().to_string())
        .collect()
}

fn refusal_paths(doc: &Value, reason: &str) -> Vec<String> {
    doc["report"]["refusals"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["reason"] == reason)
        .map(|r| paths(&r["paths"]))
        .unwrap_or_else(|| panic!("no {reason} refusal in {doc}"))
}

/// Acceptance 1, 2, 15 and 16 for one stopped operation.
fn resolves_a_same_shard_conflict(stop: impl Fn(&Repo), head: &str) {
    let r = Repo::new(None);
    r.same_shard();
    stop(&r);
    assert!(
        !r.unmerged_derived().is_empty(),
        "the stop left no derived conflict"
    );
    let authored = r.authored_index();

    let (code, doc, text) = r.resolve(true);
    assert_eq!(code, 0, "{text}");
    assert_eq!(doc["verb"], "resolve-derived");
    assert_eq!(doc["tool"], "spec-spine");
    assert_eq!(doc["outcome"], "ok");
    let report = &doc["report"];
    for key in [
        "unmerged",
        "operation",
        "unstaged",
        "regenerated",
        "removed",
        "markers",
        "authoredChange",
        "freshness",
    ] {
        assert!(report.get(key).is_some(), "report lacks {key}: {report}");
    }
    assert_eq!(report["result"], "resolved");
    assert_eq!(report["authoredChange"]["preserved"], true, "{report}");
    assert!(
        paths(&report["regenerated"])
            .iter()
            .any(|p| p == ".derived/spec-registry/by-spec/001-alpha.json"),
        "{report}"
    );
    assert!(r.unmerged().is_empty(), "{}", r.unmerged());
    assert_eq!(
        r.authored_index(),
        authored,
        "the verb staged outside the derived root"
    );
    let check = spine(r.path(), &["check"]);
    assert_eq!(
        check.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&check.stderr)
    );
    r.still_in_progress(head);
}

#[test]
fn a_rebase_stopped_on_a_shard_conflict_is_resolved() {
    resolves_a_same_shard_conflict(|r| git_stops(r.path(), &["rebase", "main"]), "REBASE_HEAD");
}

#[test]
fn a_merge_stopped_on_a_shard_conflict_is_resolved() {
    resolves_a_same_shard_conflict(|r| git_stops(r.path(), &["merge", "main"]), "MERGE_HEAD");
}

#[test]
fn a_cherry_pick_stopped_on_a_shard_conflict_is_resolved() {
    resolves_a_same_shard_conflict(
        |r| {
            git(r.path(), &["checkout", "-q", "main"]);
            git_stops(r.path(), &["cherry-pick", "feature"]);
        },
        "CHERRY_PICK_HEAD",
    );
}

/// Acceptance 3.
#[test]
fn an_authored_conflict_refuses_and_leaves_every_entry() {
    let r = Repo::new(None);
    r.diverge(
        |r| {
            r.edit(
                "specs/001-alpha/spec.md",
                "Line three.",
                "Line three on main.",
            )
        },
        |r| {
            r.edit(
                "specs/001-alpha/spec.md",
                "Line three.",
                "Line three on feature.",
            )
        },
    );
    git_stops(r.path(), &["merge", "main"]);
    assert!(!r.unmerged_derived().is_empty());
    let index = r.index();
    let derived = r.files(".derived");

    let (code, doc, text) = r.resolve(true);
    assert_eq!(code, 2, "{text}");
    assert_eq!(doc["outcome"], "refused");
    assert_eq!(
        refusal_paths(&doc, "authored-unmerged"),
        vec!["specs/001-alpha/spec.md"]
    );
    assert_eq!(r.index(), index, "an index entry moved");
    assert_eq!(r.files(".derived"), derived, "a derived file was written");

    let (code, _, text) = r.resolve(false);
    assert_eq!(code, 2);
    assert!(text.contains("specs/001-alpha/spec.md"), "{text}");
    r.still_in_progress("MERGE_HEAD");
}

/// Acceptance 4.
#[test]
fn an_untracked_file_refuses_until_it_is_ignored() {
    let r = Repo::new(None);
    r.same_shard();
    git_stops(r.path(), &["rebase", "main"]);
    write(r.path(), "scratch/probe.rs", "fn probe() {}\n");
    let index = r.index();
    let derived = r.files(".derived");

    let (code, doc, text) = r.resolve(true);
    assert_eq!(code, 2, "{text}");
    assert_eq!(refusal_paths(&doc, "unstaged"), vec!["scratch/probe.rs"]);
    assert_eq!(r.index(), index);
    assert_eq!(r.files(".derived"), derived);

    r.append(".git/info/exclude", "scratch/\n");
    let (code, _, text) = r.resolve(true);
    assert_eq!(code, 0, "{text}");
    assert!(r.unmerged().is_empty());
    r.still_in_progress("REBASE_HEAD");
}

/// Acceptance 5.
#[test]
fn an_unstaged_edit_refuses_and_names_it() {
    let r = Repo::new(None);
    r.same_shard();
    git_stops(r.path(), &["merge", "main"]);
    r.append("src/alpha.rs", "// an edit nobody staged\n");
    let index = r.index();

    let (code, doc, text) = r.resolve(true);
    assert_eq!(code, 2, "{text}");
    assert_eq!(refusal_paths(&doc, "unstaged"), vec!["src/alpha.rs"]);
    assert_eq!(r.index(), index);
    r.still_in_progress("MERGE_HEAD");
}

/// Acceptance 6.
#[test]
fn a_modify_delete_shard_conflict_is_staged_as_a_deletion() {
    let r = Repo::new(None);
    r.diverge(
        |r| fs::remove_dir_all(r.path().join("specs/002-beta")).unwrap(),
        // A comment header claims a new file for 002, which moves 002's index
        // shard without touching 002's spec.
        |r| {
            write(
                r.path(),
                "src/gamma.rs",
                "// Spec: specs/002-beta/spec.md\npub fn gamma() {}\n",
            )
        },
    );
    git_stops(r.path(), &["merge", "main"]);
    let shard = ".derived/codebase-index/by-spec/002-beta.json";
    assert!(
        r.unmerged_derived().contains(shard),
        "expected a modify/delete conflict on {shard}: {}",
        r.unmerged()
    );

    let (code, doc, text) = r.resolve(true);
    assert_eq!(code, 0, "{text}");
    assert!(
        paths(&doc["report"]["removed"]).contains(&shard.to_string()),
        "{doc}"
    );
    assert!(!r.path().join(shard).exists());
    assert!(r.unmerged().is_empty());
    assert!(git(r.path(), &["ls-files", "--", shard]).is_empty());
    r.still_in_progress("MERGE_HEAD");
}

/// Acceptance 7.
#[test]
fn an_invalid_merged_corpus_writes_nothing_and_keeps_the_conflicts() {
    let r = Repo::new(None);
    r.diverge(
        // Each side is valid alone; merged, 002 is superseded by a spec that
        // no longer exists (V-008, an error).
        |r| {
            fs::remove_dir_all(r.path().join("specs/001-alpha")).unwrap();
            r.append("specs/002-beta/spec.md", "\nA line on main.\n");
        },
        |r| {
            r.edit(
                "specs/002-beta/spec.md",
                "status: approved\n",
                "status: superseded\nsuperseded_by: \"001-alpha\"\n",
            )
        },
    );
    git_stops(r.path(), &["merge", "main"]);
    let conflicts = r.unmerged_derived();
    assert!(!conflicts.is_empty());
    let index = r.index();
    let derived = r.files(".derived");

    let (code, doc, text) = r.resolve(true);
    assert_eq!(code, 1, "{text}");
    assert_eq!(doc["report"]["result"], "invalid");
    assert!(!doc["report"]["violations"].as_array().unwrap().is_empty());
    assert_eq!(r.index(), index);
    assert_eq!(r.unmerged_derived(), conflicts);
    assert_eq!(r.files(".derived"), derived);
    r.still_in_progress("MERGE_HEAD");
}

/// Acceptance 8.
#[test]
fn a_configured_derived_root_is_honored_and_the_default_untouched() {
    let r = Repo::new(Some("gen/ledger"));
    // A decoy at the default's path, committed as ordinary content.
    write(
        r.path(),
        ".derived/spec-registry/by-spec/001-alpha.json",
        "{\"decoy\": true}\n",
    );
    r.commit("decoy");
    r.same_shard();
    git_stops(r.path(), &["rebase", "main"]);
    assert!(!r.unmerged_derived().is_empty());
    let decoy_entry = git(r.path(), &["ls-files", "-s", "--", ".derived"]);
    let decoy_files = r.files(".derived");

    let (code, doc, text) = r.resolve(true);
    assert_eq!(code, 0, "{text}");
    let report = &doc["report"];
    assert!(
        paths(&report["regenerated"])
            .iter()
            .all(|p| p.starts_with("gen/ledger/")),
        "{report}"
    );
    assert!(!paths(&report["regenerated"]).is_empty());
    assert_eq!(
        git(r.path(), &["ls-files", "-s", "--", ".derived"]),
        decoy_entry
    );
    assert_eq!(r.files(".derived"), decoy_files);
    assert!(r.unmerged().is_empty());
    r.still_in_progress("REBASE_HEAD");
}

/// Acceptance 9.
#[test]
fn nothing_conflicted_under_the_derived_root_writes_nothing() {
    let r = Repo::new(None);
    r.diverge(
        |r| r.edit("notes.txt", "x\n", "main\n"),
        |r| r.edit("notes.txt", "x\n", "feature\n"),
    );
    git_stops(r.path(), &["merge", "main"]);
    assert!(r.unmerged_derived().is_empty());
    write(r.path(), "stray.txt", "untracked\n");
    let index = r.index();
    let derived = r.files(".derived");

    let (code, doc, text) = r.resolve(true);
    assert_eq!(code, 0, "{text}");
    let report = &doc["report"];
    assert_eq!(report["result"], "nothing-to-resolve");
    assert_eq!(report["unmerged"][0]["path"], "notes.txt");
    assert_eq!(report["unmerged"][0]["side"], "authored");
    assert!(paths(&report["unstaged"]).contains(&"stray.txt".to_string()));
    assert!(report["operation"].is_null());
    assert_eq!(r.index(), index);
    assert_eq!(r.files(".derived"), derived);
    r.still_in_progress("MERGE_HEAD");
}

/// Acceptance 10.
#[test]
fn a_tracked_build_meta_conflict_refuses() {
    let r = Repo::new(None);
    let meta = ".derived/spec-registry/build-meta.json";
    // Each side regenerates, then tracks its own build-meta.json against the
    // ignore rule, so the two add the file with different bytes.
    let side = |branch: &str, change: &dyn Fn(&Repo)| {
        git(r.path(), &["checkout", "-q", branch]);
        change(&r);
        spine_ok(r.path(), &["compile"]);
        spine_ok(r.path(), &["index"]);
        write(r.path(), meta, &format!("{{\"side\": \"{branch}\"}}\n"));
        git(r.path(), &["add", "-A"]);
        git(r.path(), &["add", "-f", meta]);
        git(r.path(), &["commit", "-qm", branch]);
    };
    git(r.path(), &["branch", "feature"]);
    side("feature", &|r| {
        r.append("specs/001-alpha/spec.md", "\nA feature line.\n")
    });
    side("main", &|r| {
        r.edit(
            "specs/001-alpha/spec.md",
            "title: \"Alpha\"",
            "title: \"Alpha on main\"",
        )
    });
    git(r.path(), &["checkout", "-q", "feature"]);
    git_stops(r.path(), &["merge", "main"]);
    assert!(r.unmerged_derived().contains(meta), "{}", r.unmerged());
    let index = r.index();
    let derived = r.files(".derived");

    let (code, doc, text) = r.resolve(true);
    assert_eq!(code, 2, "{text}");
    assert_eq!(refusal_paths(&doc, "unsettled-derived"), vec![meta]);
    assert_eq!(r.index(), index);
    assert_eq!(r.files(".derived"), derived);
    r.still_in_progress("MERGE_HEAD");
}

/// Acceptance 11.
#[test]
fn a_conflicted_stash_pop_refuses_for_want_of_an_operation() {
    let r = Repo::new(None);
    r.append("specs/001-alpha/spec.md", "\nA stashed line.\n");
    spine_ok(r.path(), &["compile"]);
    spine_ok(r.path(), &["index"]);
    git(r.path(), &["stash", "push", "-q"]);
    r.edit(
        "specs/001-alpha/spec.md",
        "title: \"Alpha\"",
        "title: \"Alpha committed\"",
    );
    r.commit("committed");
    git_stops(r.path(), &["stash", "pop"]);
    assert!(!r.unmerged_derived().is_empty());
    let index = r.index();

    let (code, doc, text) = r.resolve(true);
    assert_eq!(code, 2, "{text}");
    refusal_paths(&doc, "no-operation");
    assert_eq!(r.index(), index);
    let (_, _, text) = r.resolve(false);
    assert!(text.contains("no merge, rebase or cherry-pick"), "{text}");
}

/// Both sides append a different line to `notes.txt`, and edit spec 001 far
/// apart: a merge of `main` into `feature` stops on both.
fn appended_both_sides() -> Repo {
    let r = Repo::new(None);
    r.diverge(
        |r| {
            r.append("notes.txt", "c\n");
            r.edit(
                "specs/001-alpha/spec.md",
                "title: \"Alpha\"",
                "title: \"Alpha on main\"",
            );
        },
        |r| {
            r.append("notes.txt", "b\n");
            r.append("specs/001-alpha/spec.md", "\nA feature line.\n");
        },
    );
    git_stops(r.path(), &["merge", "main"]);
    assert!(r.unmerged().contains("notes.txt"));
    assert!(!r.unmerged_derived().is_empty());
    r
}

/// Acceptance 12.
#[test]
fn staged_conflict_markers_are_a_finding() {
    let r = appended_both_sides();
    git(r.path(), &["add", "notes.txt"]);

    let (code, doc, text) = r.resolve(true);
    assert_eq!(code, 1, "{text}");
    let markers = doc["report"]["markers"].as_array().unwrap();
    assert!(!markers.is_empty(), "{doc}");
    assert!(markers.iter().all(|m| m["path"] == "notes.txt"));
    assert_eq!(markers[0]["line"], 2);
    assert!(r.unmerged().is_empty(), "the derived tree was not staged");
    r.still_in_progress("MERGE_HEAD");

    // The prose form names the path and the line too.
    let r = appended_both_sides();
    git(r.path(), &["add", "notes.txt"]);
    let (code, _, text) = r.resolve(false);
    assert_eq!(code, 1, "{text}");
    assert!(text.contains("notes.txt:2:"), "{text}");
}

/// Acceptance 13, the loss.
#[test]
fn taking_the_receiving_side_loses_the_carried_lines() {
    let r = appended_both_sides();
    git(r.path(), &["checkout", "--theirs", "notes.txt"]);
    git(r.path(), &["add", "notes.txt"]);

    let (code, doc, text) = r.resolve(true);
    assert_eq!(code, 1, "{text}");
    let change = &doc["report"]["authoredChange"];
    assert_eq!(change["preserved"], false);
    assert_eq!(change["differences"][0]["path"], "notes.txt");
    assert_eq!(
        paths(&change["differences"][0]["onlyInCarried"]),
        vec!["+b"]
    );
    assert!(doc["report"]["markers"].as_array().unwrap().is_empty());
    r.still_in_progress("MERGE_HEAD");
}

/// Acceptance 13, both sides kept.
#[test]
fn keeping_both_sides_preserves_the_carried_change() {
    let r = appended_both_sides();
    write(r.path(), "notes.txt", "x\nc\nb\n");
    git(r.path(), &["add", "notes.txt"]);

    let (code, doc, text) = r.resolve(true);
    assert_eq!(code, 0, "{text}");
    assert_eq!(doc["report"]["authoredChange"]["preserved"], true);
    r.still_in_progress("MERGE_HEAD");
}

/// Acceptance 14.
#[test]
fn a_replayed_commit_next_to_a_base_edit_is_preserved() {
    let r = Repo::new(None);
    r.diverge(
        |r| {
            r.edit(
                "specs/001-alpha/spec.md",
                "Line two.",
                "Line two, edited on main.",
            )
        },
        |r| {
            r.edit(
                "specs/001-alpha/spec.md",
                "Line three.",
                "Line three, edited on feature.",
            )
        },
    );
    git_stops(r.path(), &["rebase", "main"]);
    assert!(
        !r.unmerged().contains("specs/"),
        "the nearby edits were meant to merge cleanly: {}",
        r.unmerged()
    );

    let (code, doc, text) = r.resolve(true);
    assert_eq!(code, 0, "{text}");
    assert_eq!(doc["report"]["authoredChange"]["preserved"], true, "{doc}");
    assert_eq!(doc["report"]["operation"]["kind"], "rebase");
    r.still_in_progress("REBASE_HEAD");
}

/// Acceptance 17.
#[test]
fn capabilities_lists_the_verb() {
    let tmp = tempfile::tempdir().unwrap();
    let out = spine(tmp.path(), &["capabilities", "--json"]);
    assert_eq!(out.status.code(), Some(0));
    let doc: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(
        doc["verbs"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v["path"] == "resolve-derived"),
        "{doc}"
    );
}
