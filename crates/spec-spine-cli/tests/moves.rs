// Spec: specs/111-a-move-is-a-reviewed-mapping/spec.md
//! Spec 111 through the shipped binary: the `registry moves` verb's exit
//! codes and read documents (§3.4), and the coupling gate's refusal of an
//! unauthorized deletion, unchanged by a declared move, on a git fixture
//! whose similarity index would pair the two halves (§3.5, §3.6).

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_spec-spine"))
}

fn code(out: &Output) -> i32 {
    out.status.code().unwrap_or(-1)
}

fn run_in(root: &Path, args: &[&str]) -> Output {
    bin().arg("--repo").arg(root).args(args).output().unwrap()
}

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, content).unwrap();
}

fn write_spec(root: &Path, id: &str, extra: &str) {
    write(
        root,
        &format!("specs/{id}/spec.md"),
        &format!(
            "---\nid: \"{id}\"\ntitle: \"T\"\nstatus: approved\ncreated: \"2026-09-23\"\n\
             summary: \"s\"\n{extra}---\n# {id}\n\n## 1. Purpose\n\nWhy.\n\n## Verification\n\nChecks.\n"
        ),
    );
}

fn refresh(root: &Path) {
    assert_eq!(code(&run_in(root, &["compile"])), 0);
    assert_eq!(code(&run_in(root, &["index"])), 0);
}

// ===== 3.4: `registry moves`, through the shipped binary ====================

fn moves_corpus() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    write_spec(
        tmp.path(),
        "001-a",
        "moves:\n  - from: \"a.rs\"\n    to: \"b.rs\"\n    kind: relocated\n",
    );
    write_spec(
        tmp.path(),
        "002-b",
        "moves:\n  - from: \"b.rs\"\n    to: \"c.rs\"\n    kind: relocated\n",
    );
    assert_eq!(code(&run_in(tmp.path(), &["compile"])), 0);
    tmp
}

#[test]
fn resolved_and_unmapped_exit_0() {
    let tmp = moves_corpus();

    let out = run_in(tmp.path(), &["registry", "moves", "a.rs", "--json"]);
    assert_eq!(code(&out), 0, "{}", String::from_utf8_lossy(&out.stderr));
    let doc: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(doc["schemaVersion"], spec_spine_types::READ_SCHEMA_VERSION);
    assert_eq!(doc["outcome"], "resolved");
    assert_eq!(doc["hops"].as_array().unwrap().len(), 2);
    assert_eq!(doc["terminals"][0]["path"], "c.rs");

    let out = run_in(
        tmp.path(),
        &["registry", "moves", "never/declared.rs", "--json"],
    );
    assert_eq!(code(&out), 0);
    let doc: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(doc["outcome"], "unmapped");
}

#[test]
fn ambiguous_and_cycle_exit_1() {
    let tmp = tempfile::tempdir().unwrap();
    write_spec(
        tmp.path(),
        "001-a",
        "moves:\n  - from: \"x.rs\"\n    to: \"y.rs\"\n    kind: relocated\n",
    );
    write_spec(
        tmp.path(),
        "002-b",
        "moves:\n  - from: \"x.rs\"\n    to: \"z.rs\"\n    kind: relocated\n",
    );
    assert_eq!(code(&run_in(tmp.path(), &["compile"])), 0);

    let out = run_in(tmp.path(), &["registry", "moves", "x.rs", "--json"]);
    assert_eq!(code(&out), 1, "{}", String::from_utf8_lossy(&out.stdout));
    let doc: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(doc["outcome"], "ambiguous");
    assert_eq!(doc["candidates"].as_array().unwrap().len(), 2);

    let tmp2 = tempfile::tempdir().unwrap();
    write_spec(
        tmp2.path(),
        "001-a",
        "moves:\n  - from: \"p.rs\"\n    to: \"q.rs\"\n    kind: relocated\n",
    );
    write_spec(
        tmp2.path(),
        "002-b",
        "moves:\n  - from: \"q.rs\"\n    to: \"p.rs\"\n    kind: relocated\n",
    );
    assert_eq!(code(&run_in(tmp2.path(), &["compile"])), 0);
    let out = run_in(tmp2.path(), &["registry", "moves", "p.rs", "--json"]);
    assert_eq!(code(&out), 1);
    let doc: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(doc["outcome"], "cycle");
    assert_eq!(doc["chain"], serde_json::json!(["p.rs", "q.rs", "p.rs"]));
}

#[test]
fn no_path_lists_every_declaration_flattened_and_sorted() {
    let tmp = moves_corpus();
    let out = run_in(tmp.path(), &["registry", "moves", "--json"]);
    assert_eq!(code(&out), 0, "{}", String::from_utf8_lossy(&out.stderr));
    let doc: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let items = doc["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["from"], "a.rs");
    assert_eq!(items[0]["declaredBy"], "001-a");
    assert_eq!(items[1]["from"], "b.rs");
    assert_eq!(items[1]["declaredBy"], "002-b");
}

#[test]
fn answers_from_the_committed_registry_with_no_freshness_refusal() {
    // Like `registry show`: a stale index does not refuse this read, because
    // it never consults the index, only the committed registry (§3.4).
    let tmp = moves_corpus();
    // No `index` run at all; only `compile` above.
    let out = run_in(tmp.path(), &["registry", "moves", "a.rs", "--json"]);
    assert_eq!(code(&out), 0, "{}", String::from_utf8_lossy(&out.stderr));
}

// ===== 3.5, 3.6: a move changes no verdict, on a git fixture whose ==========
// similarity index would pair the two halves ================================

fn git_in(root: &Path, args: &[&str]) {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .env("GIT_AUTHOR_NAME", "t")
        .env("GIT_AUTHOR_EMAIL", "t@t")
        .env("GIT_COMMITTER_NAME", "t")
        .env("GIT_COMMITTER_EMAIL", "t@t")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn git_out(root: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

const RENAMEABLE_CONTENT: &str = "pub fn body() {\n    println!(\"one\");\n    println!(\"two\");\n    println!(\"three\");\n    println!(\"four\");\n    println!(\"five\");\n}\n";

const OLD_PATH: &str = "crate-a/src/old.rs";
const NEW_PATH: &str = "crate-a/src/new.rs";

/// A two-spec repo: `crate-a`'s manifest floor is `001-a` (spec 001 "owns"
/// `crate-a/src/old.rs` the way spec 048's floor owns an unclaimed path in its
/// package, without a specific `establishes` entry: `old.rs` need not exist at
/// head, and a dangling `establishes` unit on a deleted path is a different,
/// unrelated refusal this fixture must not also trip). The base commit has
/// `old.rs` in place. The head commit deletes it, adds `new.rs` with
/// identical content, and edits `002-b` to claim `new.rs` (and, when
/// `declare_move` is set, to also declare the mapping); it never edits
/// `001-a` unless `withdraw_001` is set, in which case `001-a`'s spec.md is
/// also touched in that same head commit (the legitimate clearance, spec 100
/// §3.6: any authoring edit to an owning spec clears its C-001).
fn build_fixture(declare_move: bool, withdraw_001: bool) -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    write(root, "Cargo.toml", "[workspace]\nmembers = [\"crate-a\"]\n");
    write(
        root,
        "crate-a/Cargo.toml",
        "[package]\nname = \"crate-a\"\nversion = \"0.1.0\"\n\
         [package.metadata.spec-spine]\nspec = \"001-a\"\n",
    );
    write_spec(root, "001-a", "");
    write_spec(root, "002-b", "");
    write(root, OLD_PATH, RENAMEABLE_CONTENT);

    git_in(root, &["init", "-q"]);
    refresh(root);
    git_in(root, &["add", "-A"]);
    git_in(root, &["commit", "-q", "-m", "base"]);

    // Head: delete old.rs, add new.rs with the same content (git's
    // similarity index pairs identical content at 100%, the strongest case
    // of "near-identical" a rename detector would ever see), edit 002-b to
    // claim new.rs (and, conditionally, declare the move), and touch 001-a
    // only when `withdraw_001` asks for it.
    fs::remove_file(root.join(OLD_PATH)).unwrap();
    write(root, NEW_PATH, RENAMEABLE_CONTENT);
    let moves_yaml = if declare_move {
        format!("moves:\n  - from: \"{OLD_PATH}\"\n    to: \"{NEW_PATH}\"\n    kind: relocated\n")
    } else {
        String::new()
    };
    write_spec(
        root,
        "002-b",
        &format!("establishes:\n  - \"{NEW_PATH}\"\n{moves_yaml}"),
    );
    if withdraw_001 {
        write_spec(
            root,
            "001-a",
            "references:\n  - unit: { kind: file, path: \"Cargo.toml\" }\n    role: \"context\"\n",
        );
    }
    refresh(root);
    git_in(root, &["add", "-A"]);
    git_in(root, &["commit", "-q", "-m", "move old.rs to new.rs"]);

    tmp
}

/// The `couple --json` verdict envelope's `report` object (spec 034): the
/// `CoupleReport` itself, unwrapped from `{ "exitCode", "ok", "report", ... }`.
fn couple_json(root: &Path) -> serde_json::Value {
    let out = run_in(
        root,
        &["couple", "--base", "HEAD~1", "--head", "HEAD", "--json"],
    );
    let envelope: serde_json::Value = serde_json::from_slice(&out.stdout)
        .unwrap_or_else(|e| panic!("{e}: {}", String::from_utf8_lossy(&out.stdout)));
    envelope["report"].clone()
}

fn couple_exit(root: &Path) -> i32 {
    code(&run_in(
        root,
        &["couple", "--base", "HEAD~1", "--head", "HEAD"],
    ))
}

/// The fixture is one a rename detector would pair: proven directly against
/// git, independent of anything this binary does.
fn assert_git_would_pair_them(root: &Path) {
    let status = git_out(
        root,
        &["diff", "--find-renames", "--name-status", "HEAD~1", "HEAD"],
    );
    assert!(
        status.lines().any(|l| l.starts_with('R')),
        "git itself must report a rename pairing for this fixture to be a \
         meaningful test: {status}"
    );
}

#[test]
fn a_declared_mapping_does_not_clear_the_unauthorized_deletion() {
    let tmp = build_fixture(true, false);
    let root = tmp.path();
    assert_git_would_pair_them(root);

    assert_eq!(
        couple_exit(root),
        1,
        "a declared move must not clear a deletion its owning spec never authored"
    );
    let report = couple_json(root);
    let violations = report["violations"].as_array().unwrap();
    assert_eq!(violations.len(), 1, "{report}");
    assert!(
        violations
            .iter()
            .any(|v| v["code"] == "C-001" && v["message"].as_str().unwrap().contains(OLD_PATH)),
        "{report}"
    );
    // `new.rs` is claimed by 002-b, which authored the change: no violation
    // names it.
    assert!(
        !violations
            .iter()
            .any(|v| v["message"].as_str().unwrap().contains(NEW_PATH)),
        "{report}"
    );
    // Both halves are examined independently (spec 100 §3.6): the deleted
    // path, the added path and the editing spec's own spec.md, not one
    // collapsed rename.
    assert_eq!(report["checkedPaths"], 3, "{report}");
    assert!(
        report["deletions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["path"] == OLD_PATH),
        "{report}"
    );
}

#[test]
fn the_same_fixture_without_the_mapping_reaches_the_identical_verdict() {
    let with_mapping = build_fixture(true, false);
    let without_mapping = build_fixture(false, false);
    assert_git_would_pair_them(with_mapping.path());
    assert_git_would_pair_them(without_mapping.path());

    assert_eq!(couple_exit(with_mapping.path()), 1);
    assert_eq!(couple_exit(without_mapping.path()), 1);

    let a = couple_json(with_mapping.path());
    let b = couple_json(without_mapping.path());
    // I-1: no verdict consults a declaration. The violation set and the
    // deletion provenance are byte-identical whether or not the mapping was
    // declared.
    assert_eq!(a["violations"], b["violations"], "a={a}\nb={b}");
    assert_eq!(a["deletions"], b["deletions"], "a={a}\nb={b}");
    assert_eq!(a["checkedPaths"], b["checkedPaths"]);
}

#[test]
fn editing_the_owning_spec_too_clears_it_in_both_variants() {
    for declare_move in [true, false] {
        let tmp = build_fixture(declare_move, true);
        let root = tmp.path();
        assert_git_would_pair_them(root);
        assert_eq!(
            couple_exit(root),
            0,
            "declare_move={declare_move}: withdrawing 001-a's own claim must \
             still clear the deletion, mapping or not"
        );
    }
}
