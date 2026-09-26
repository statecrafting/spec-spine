// Spec: specs/113-a-waiver-has-a-declared-lifecycle/spec.md
//! A waiver's declared lifecycle (spec 113) through the shipped binary: the
//! pull request body's lifecycle lines, `--waiver-as-of`, `--waiver-uses`,
//! ancestry answered by git, the verdict envelope, and the facade agreeing.

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use serde_json::Value;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_spec-spine"))
}

fn code(out: &Output) -> i32 {
    out.status.code().unwrap_or(-1)
}

fn text(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, content).unwrap();
}

fn git(root: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(args)
        .current_dir(root)
        .env_remove("GIT_DIR")
        .env_remove("GIT_INDEX_FILE")
        .output()
        .unwrap();
    assert!(out.status.success(), "git {args:?}: {}", text(&out));
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// Two owned files in one crate, each owned by its own spec, in a git
/// repository with two commits on `main` and one on a side branch, so a
/// `-Since:` can name an ancestor of `HEAD` and a commit that is not one.
struct Fixture {
    _tmp: tempfile::TempDir,
    root: std::path::PathBuf,
    first: String,
    side: String,
}

fn fixture() -> Fixture {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().to_path_buf();
    write(
        &root,
        "Cargo.toml",
        "[workspace]\nmembers = [\"crate-a\"]\n",
    );
    write(
        &root,
        "crate-a/Cargo.toml",
        "[package]\nname = \"crate-a\"\nversion = \"0.1.0\"\n\
         [package.metadata.spec-spine]\nspec = \"001-a\"\n",
    );
    write(&root, "crate-a/src/lib.rs", "pub fn a() {}\n");
    write(&root, "crate-a/src/other.rs", "pub fn other() {}\n");
    for (id, path) in [
        ("001-a", "crate-a/src/lib.rs"),
        ("002-b", "crate-a/src/other.rs"),
    ] {
        write(
            &root,
            &format!("specs/{id}/spec.md"),
            &format!(
                "---\nid: \"{id}\"\ntitle: \"T\"\nstatus: approved\ncreated: \"2026-09-23\"\n\
                 summary: \"s\"\nestablishes:\n  - \"{path}\"\n---\n# {id}\n## body\n"
            ),
        );
    }
    for args in [
        &["init", "-q", "-b", "main"][..],
        &["config", "user.name", "t"],
        &["config", "user.email", "t@example.invalid"],
        &["config", "commit.gpgsign", "false"],
    ] {
        git(&root, args);
    }
    for verb in ["compile", "index"] {
        let out = bin().arg("--repo").arg(&root).arg(verb).output().unwrap();
        assert_eq!(code(&out), 0, "{verb}: {}", text(&out));
    }
    git(&root, &["add", "-A"]);
    git(&root, &["commit", "-q", "-m", "first"]);
    let first = git(&root, &["rev-parse", "HEAD"]);
    git(&root, &["checkout", "-q", "-b", "side"]);
    write(&root, "notes.md", "side\n");
    git(&root, &["add", "-A"]);
    git(&root, &["commit", "-q", "-m", "side"]);
    let side = git(&root, &["rev-parse", "HEAD"]);
    git(&root, &["checkout", "-q", "main"]);
    write(&root, "notes.md", "main\n");
    git(&root, &["add", "-A"]);
    git(&root, &["commit", "-q", "-m", "second"]);
    Fixture {
        _tmp: tmp,
        root,
        first,
        side,
    }
}

const BOTH: &[&str] = &["crate-a/src/lib.rs", "crate-a/src/other.rs"];

/// `couple` over both owned files (a `C-001` on each), with `body` as the
/// pull request body and `extra` flags.
fn couple(f: &Fixture, body: &str, extra: &[&str]) -> Output {
    write(&f.root, "changed.txt", &format!("{}\n", BOTH.join("\n")));
    write(&f.root, "pr-body.txt", body);
    bin()
        .arg("--repo")
        .arg(&f.root)
        .arg("couple")
        .arg("--paths-from")
        .arg(f.root.join("changed.txt"))
        .arg("--pr-body")
        .arg(f.root.join("pr-body.txt"))
        .args(extra)
        .output()
        .unwrap()
}

fn report(out: &Output) -> Value {
    let v: Value = serde_json::from_slice(&out.stdout).expect("one JSON envelope on stdout");
    assert_eq!(v["schemaVersion"], spec_spine_types::VERDICT_SCHEMA_VERSION);
    assert_eq!(
        v["schemaVersion"], "1.1.0",
        "spec 132: the family envelope, a MAJOR over spec 113's 0.6.0; spec 152's MINOR"
    );
    v["report"].clone()
}

#[test]
fn a_scoped_waiver_clears_its_path_and_the_other_still_refuses() {
    let f = fixture();
    let body = "Spec-Drift-Waiver: lib only\nSpec-Drift-Waiver-Paths: crate-a/src/lib.rs\n";
    let out = couple(&f, body, &[]);
    assert_eq!(code(&out), 1, "{}", text(&out));
    let e = String::from_utf8_lossy(&out.stderr);
    assert!(
        e.contains("1 drift violation(s)"),
        "only the uncleared one is counted: {e}"
    );
    assert!(e.contains("'crate-a/src/other.rs' changed without"), "{e}");
    assert!(!e.contains("'crate-a/src/lib.rs' changed without"), "{e}");
    assert!(
        e.contains("scoped to crate-a/src/lib.rs, effective, cleared 1"),
        "{e}"
    );
    assert!(e.contains("clears C-001 crate-a/src/lib.rs"), "{e}");

    let r = report(&couple(&f, body, &["--json"]));
    assert_eq!(r["waivers"][0]["clears"][0]["path"], "crate-a/src/lib.rs");
    assert_eq!(r["waivers"][0]["scoped"], true);
    assert!(
        r.get("waiver").is_none(),
        "a blocked run is not a waived run"
    );
}

#[test]
fn a_plain_waiver_renders_exactly_as_before() {
    let f = fixture();
    let out = couple(&f, "Spec-Drift-Waiver: hotfix OPS-9\n", &[]);
    assert_eq!(code(&out), 0, "{}", text(&out));
    let o = String::from_utf8_lossy(&out.stdout);
    assert!(
        o.starts_with("spec-spine couple: 2 violation(s) waived, reason: hotfix OPS-9\n"),
        "{o}"
    );
    assert!(
        !o.contains("waiver 1"),
        "no section for one plain waiver: {o}"
    );
    let r = report(&couple(
        &f,
        "Spec-Drift-Waiver: hotfix OPS-9\n",
        &["--json"],
    ));
    assert_eq!(r["waiver"], "hotfix OPS-9");
    assert_eq!(
        r["waivers"][0]["scoped"], false,
        "§3.2: reported as unscoped"
    );
}

#[test]
fn a_run_with_no_waiver_adds_no_member() {
    let f = fixture();
    let r = report(&couple(&f, "no waiver\n", &["--json"]));
    let keys: Vec<&String> = r.as_object().unwrap().keys().collect();
    assert_eq!(keys, vec!["checkedPaths", "violations"], "{r}");
}

#[test]
fn an_expiry_is_judged_only_against_the_date_the_operator_gives() {
    let f = fixture();
    let body = "Spec-Drift-Waiver: bump\nSpec-Drift-Waiver-Until: 2026-12-31\n";
    let before = couple(&f, body, &["--waiver-as-of", "2026-12-31"]);
    assert_eq!(code(&before), 0, "{}", text(&before));
    let after = couple(&f, body, &["--waiver-as-of", "2027-01-01"]);
    assert_eq!(code(&after), 1, "{}", text(&after));
    assert!(
        text(&after).contains("expiry: failed (declared 2026-12-31, input 2027-01-01): expired")
    );
    // No date given: not evaluated, and never the clock. The exit code is the
    // same as a satisfied expiry's, so the verdict is what is read.
    let undated = couple(&f, body, &[]);
    assert_eq!(code(&undated), 0);
    assert!(text(&undated).contains("expiry: not evaluated (declared 2026-12-31)"));
    let r = report(&couple(&f, body, &["--json"]));
    assert_eq!(r["waivers"][0]["checks"][0]["outcome"], "not-evaluated");
    assert!(r["waivers"][0]["checks"][0].get("input").is_none());
}

#[test]
fn an_as_of_that_is_not_a_date_is_a_usage_error() {
    let f = fixture();
    let out = couple(
        &f,
        "Spec-Drift-Waiver: x\n",
        &["--waiver-as-of", "tomorrow"],
    );
    assert_eq!(code(&out), 3, "{}", text(&out));
}

#[test]
fn ancestry_is_answered_by_git() {
    let f = fixture();
    let since = |c: &str| format!("Spec-Drift-Waiver: bump\nSpec-Drift-Waiver-Since: {c}\n");
    let ancestor = couple(&f, &since(&f.first), &[]);
    assert_eq!(code(&ancestor), 0, "{}", text(&ancestor));
    assert!(text(&ancestor).contains("ancestry: satisfied"));
    let not_ancestor = couple(&f, &since(&f.side), &[]);
    assert_eq!(code(&not_ancestor), 1, "{}", text(&not_ancestor));
    assert!(text(&not_ancestor).contains("is not an ancestor of the head under judgment"));
    // A commit git does not know is no answer: not evaluated, not false.
    let unknown = couple(&f, &since("deadbeefdeadbeef"), &[]);
    assert_eq!(code(&unknown), 0, "{}", text(&unknown));
    assert!(text(&unknown).contains("ancestry: not evaluated"));
}

#[test]
fn a_use_count_is_the_callers_and_absent_is_not_zero() {
    let f = fixture();
    let body = "Spec-Drift-Waiver: once\nSpec-Drift-Waiver-Max-Uses: 1\n";
    let r = report(&couple(&f, body, &["--json"]));
    let id = r["waivers"][0]["id"].as_str().unwrap().to_string();
    assert_eq!(r["waivers"][0]["checks"][0]["outcome"], "not-evaluated");

    let spent = couple(&f, body, &["--waiver-uses", &format!("{id}=1")]);
    assert_eq!(code(&spent), 1, "{}", text(&spent));
    assert!(text(&spent).contains("already used 1 time(s), limit 1"));
    let fresh = couple(&f, body, &["--waiver-uses", &format!("{id}=0")]);
    assert_eq!(code(&fresh), 0, "{}", text(&fresh));

    let bad = couple(&f, body, &["--waiver-uses", "no-equals-sign"]);
    assert_eq!(code(&bad), 3, "{}", text(&bad));
}

#[test]
fn two_waivers_pair_by_scope_and_a_failed_one_is_named() {
    let f = fixture();
    let body = "Spec-Drift-Waiver: for other\nSpec-Drift-Waiver-Paths: crate-a/src/other.rs\n\
                Spec-Drift-Waiver: for lib\nSpec-Drift-Waiver-Paths: crate-a/src/lib.rs\n";
    let r = report(&couple(&f, body, &["--json"]));
    assert_eq!(r["waivers"][0]["clears"][0]["path"], "crate-a/src/other.rs");
    assert_eq!(r["waivers"][1]["clears"][0]["path"], "crate-a/src/lib.rs");
    assert_eq!(r["waiver"], "for other");

    let body = "Spec-Drift-Waiver-Paths: crate-a/src/lib.rs\nSpec-Drift-Waiver: late scope\n\
                Spec-Drift-Waiver-Until: 2020-01-01\n";
    let out = couple(&f, body, &["--waiver-as-of", "2026-09-23"]);
    assert_eq!(code(&out), 1);
    let t = text(&out);
    assert!(t.contains("REFUSED"), "{t}");
    assert!(
        t.contains("unattached waiver line, narrows nothing: Spec-Drift-Waiver-Paths"),
        "{t}"
    );
}

/// The facade decides what the CLI decides, from the same body and inputs.
#[test]
fn the_facade_and_the_cli_agree() {
    let f = fixture();
    let body = "Spec-Drift-Waiver: bump\nSpec-Drift-Waiver-Paths: crate-a/src/lib.rs\n\
                Spec-Drift-Waiver-Until: 2026-12-31\n";
    let cli = report(&couple(
        &f,
        body,
        &["--json", "--waiver-as-of", "2026-10-01"],
    ));
    let files: Vec<Value> = BOTH
        .iter()
        .map(|p| serde_json::json!({ "path": p, "hunks": [], "deleted": false }))
        .collect();
    let request = serde_json::json!({
        "repoRoot": f.root.to_str().unwrap(),
        "diff": { "files": files },
        "prBody": body,
        "waiverInputs": { "asOf": "2026-10-01" },
    });
    let facade: Value =
        serde_json::from_str(&spec_spine_core::couple_json(&request.to_string()).unwrap()).unwrap();
    assert_eq!(cli, facade);

    let both_sources = serde_json::json!({
        "repoRoot": f.root.to_str().unwrap(),
        "diff": { "files": [] },
        "prBody": body,
        "waiver": { "reason": "x" },
    });
    assert!(matches!(
        spec_spine_core::couple_json(&both_sources.to_string()),
        Err(spec_spine_types::Error::Usage(_))
    ));
}
