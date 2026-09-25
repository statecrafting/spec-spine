//! Spec 145: an unresolved claim is not called stale.
//!
//! The verbs that read the committed index refuse two different facts: shards
//! that drifted from the corpus (regenerate), and a claim that resolves to
//! nothing (regenerating does not help). `check` and `index check` have told
//! them apart since spec 079; these tests hold the readers to the same line.

use std::fs;
use std::path::Path;

use spec_spine_core::shard::{self, BY_PACKAGE_DIR, BY_SPEC_DIR};
use spec_spine_core::{
    compile, coverage, guard_committed_index, index, index_dir, index_inputs_file,
    index_shard_files, owner, registry_dir, registry_shard_files,
};
use spec_spine_types::{Config, Error};

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, content).unwrap();
}

fn emit(cfg: &Config, r: &Path) {
    let out = index(cfg, r).unwrap();
    let dir = index_dir(cfg, r);
    let (by_spec, by_package) = index_shard_files(&out.shards).unwrap();
    shard::sync_dir(&dir.join(BY_SPEC_DIR), &by_spec).unwrap();
    shard::sync_dir(&dir.join(BY_PACKAGE_DIR), &by_package).unwrap();
    let (name, content) = index_inputs_file(&out.shards).unwrap();
    fs::write(dir.join(name), content).unwrap();
    let compiled = compile(cfg, r).unwrap();
    shard::sync_dir(
        &registry_dir(cfg, r).join(BY_SPEC_DIR),
        &registry_shard_files(&compiled.shards).unwrap(),
    )
    .unwrap();
}

/// `001-a` is complete and claims a file that does not exist: an `I-004`.
fn unresolved() -> (tempfile::TempDir, Config) {
    let tmp = tempfile::tempdir().unwrap();
    let r = tmp.path();
    write(
        r,
        "specs/001-a/spec.md",
        "---\nid: \"001-a\"\ntitle: \"A\"\nstatus: approved\ncreated: \"2026-09-25\"\nimplementation: complete\nsummary: \"s\"\nestablishes:\n  - \"src/a.rs\"\n  - \"src/missing.rs\"\n---\n# a\n",
    );
    write(r, "src/a.rs", "pub fn a() {}\n");
    write(
        r,
        "specs/002-b/spec.md",
        "---\nid: \"002-b\"\ntitle: \"B\"\nstatus: approved\ncreated: \"2026-09-25\"\nimplementation: complete\nsummary: \"s\"\nestablishes:\n  - \"src/b.rs\"\n---\n# b\n",
    );
    write(r, "src/b.rs", "pub fn b() {}\n");
    let cfg = Config::default();
    emit(&cfg, r);
    (tmp, cfg)
}

fn assert_is_the_claim(err: Error, verb: &str) {
    assert_eq!(err.exit_code(), 1, "{verb}: {err}");
    let Error::Validation(v) = &err else {
        panic!("{verb}: an unresolved claim alone is a validation finding, not {err:?}");
    };
    assert_eq!(v.len(), 1, "{verb}: {v:?}");
    assert_eq!(v[0].code, "I-004");
    assert_eq!(v[0].path.as_deref(), Some("src/missing.rs"));
    assert!(
        v[0].message.contains("not staleness"),
        "{verb}: {}",
        v[0].message
    );
    assert!(!format!("{err}").contains("stale:"), "{verb}: {err}");
}

/// §3.1: every guarded reader names the claim and does not say stale.
#[test]
fn the_readers_name_an_unresolved_claim_as_one() {
    let (tmp, cfg) = unresolved();
    let r = tmp.path();
    assert_is_the_claim(guard_committed_index(&cfg, r).unwrap_err(), "guard");
    assert_is_the_claim(coverage(&cfg, r).unwrap_err(), "index coverage");
    assert_is_the_claim(owner(&cfg, r, "src/a.rs").unwrap_err(), "index owner");
}

/// §3.1: drifted shards are still staleness, naming only the drifted shard (a
/// blocked shard is reported as blocked, spec 079), and they win when both are
/// true, because regenerating is the first step.
#[test]
fn drifted_shards_are_still_stale_and_come_first() {
    let (tmp, cfg) = unresolved();
    let r = tmp.path();
    let b = fs::read_to_string(r.join("specs/002-b/spec.md")).unwrap();
    write(r, "specs/002-b/spec.md", &b.replace("# b", "# b, edited"));
    let err = guard_committed_index(&cfg, r).unwrap_err();
    let Error::Stale { actual, .. } = &err else {
        panic!("drift is staleness: {err:?}");
    };
    assert!(actual.contains("modified by-spec/002-b.json"), "{actual}");
    assert!(!actual.contains("blocking-diagnostics"), "{actual}");
}

/// §3.1: with the claim resolved, every reader answers.
#[test]
fn a_resolved_corpus_reads() {
    let (tmp, cfg) = unresolved();
    let r = tmp.path();
    write(r, "src/missing.rs", "pub fn m() {}\n");
    emit(&cfg, r);
    guard_committed_index(&cfg, r).unwrap();
    coverage(&cfg, r).unwrap();
}

/// §3.1: a shard that is blocked and whose bytes also moved is stale: the
/// committed file disagrees with the corpus, whatever else is true of it.
#[test]
fn a_blocked_shard_that_moved_is_stale() {
    let (tmp, cfg) = unresolved();
    let r = tmp.path();
    let a = fs::read_to_string(r.join("specs/001-a/spec.md")).unwrap();
    write(r, "specs/001-a/spec.md", &a.replace("# a", "# a, edited"));
    let err = guard_committed_index(&cfg, r).unwrap_err();
    let Error::Stale { actual, .. } = &err else {
        panic!("a moved shard is staleness even when blocked: {err:?}");
    };
    assert!(actual.contains("modified by-spec/001-a.json"), "{actual}");
}
