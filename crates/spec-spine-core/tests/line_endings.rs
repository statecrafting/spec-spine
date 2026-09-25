//! Spec 143: git's line-ending conversion does not stale the ledger (WF-7).
//!
//! A Windows checkout with `core.autocrlf=true` and no `.gitattributes` rule
//! for the derived tree hands every committed shard back with CRLF endings.
//! These tests do what that checkout does, on every platform, by rewriting the
//! committed tree with CRLF, and assert that the freshness reads see the same
//! text while still seeing a real change.

use std::fs;
use std::path::Path;

use spec_spine_core::shard::{self, BY_PACKAGE_DIR, BY_SPEC_DIR};
use spec_spine_core::{
    Freshness, check_index_freshness, check_registry_freshness, compile, index, index_dir,
    index_inputs_file, index_shard_files, registry_dir, registry_shard_files,
};
use spec_spine_types::Config;

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, content).unwrap();
}

fn fixture() -> (tempfile::TempDir, Config) {
    let tmp = tempfile::tempdir().unwrap();
    let r = tmp.path();
    write(
        r,
        "specs/001-a/spec.md",
        "---\nid: \"001-a\"\ntitle: \"A\"\nstatus: approved\ncreated: \"2026-09-25\"\nimplementation: complete\nsummary: \"s\"\nestablishes:\n  - \"src/a.rs\"\n---\n# 001-a\n\nBody.\n",
    );
    write(r, "src/a.rs", "pub fn a() {}\n");
    let cfg = Config::default();
    let out = index(&cfg, r).unwrap();
    let dir = index_dir(&cfg, r);
    let (by_spec, by_package) = index_shard_files(&out.shards).unwrap();
    shard::sync_dir(&dir.join(BY_SPEC_DIR), &by_spec).unwrap();
    shard::sync_dir(&dir.join(BY_PACKAGE_DIR), &by_package).unwrap();
    let (name, content) = index_inputs_file(&out.shards).unwrap();
    fs::write(dir.join(name), content).unwrap();
    let compiled = compile(&cfg, r).unwrap();
    shard::sync_dir(
        &registry_dir(&cfg, r).join(BY_SPEC_DIR),
        &registry_shard_files(&compiled.shards).unwrap(),
    )
    .unwrap();
    (tmp, cfg)
}

/// Rewrite every file under `dir` with CRLF line endings, as a Windows checkout
/// with `core.autocrlf=true` does. Returns how many files it rewrote.
fn to_crlf(dir: &Path) -> usize {
    let mut n = 0;
    for entry in fs::read_dir(dir).unwrap() {
        let p = entry.unwrap().path();
        if p.is_dir() {
            n += to_crlf(&p);
        } else {
            let text = fs::read_to_string(&p).unwrap();
            assert!(!text.contains('\r'));
            fs::write(&p, text.replace('\n', "\r\n")).unwrap();
            n += 1;
        }
    }
    n
}

fn fresh(f: Freshness) -> bool {
    matches!(f, Freshness::Fresh)
}

/// §3.1: a CRLF checkout of a fresh tree is fresh, in both trees.
#[test]
fn a_crlf_checkout_of_a_fresh_tree_is_fresh() {
    let (tmp, cfg) = fixture();
    let r = tmp.path();
    let derived = r.join(&cfg.layout.derived_dir);
    assert_eq!(
        to_crlf(&derived),
        3,
        "one registry shard, one index shard, the inputs sidecar"
    );
    assert!(fresh(check_registry_freshness(&cfg, r).unwrap()));
    assert!(fresh(check_index_freshness(&cfg, r).unwrap()));
}

/// §3.1: CRLF does not hide a real change. The spec is edited after the tree
/// was committed and checked out with CRLF, and both trees report it.
#[test]
fn a_crlf_checkout_still_sees_a_real_change() {
    let (tmp, cfg) = fixture();
    let r = tmp.path();
    to_crlf(&r.join(&cfg.layout.derived_dir));
    write(
        r,
        "specs/001-a/spec.md",
        "---\nid: \"001-a\"\ntitle: \"A\"\nstatus: approved\ncreated: \"2026-09-25\"\nimplementation: complete\nsummary: \"s\"\nestablishes:\n  - \"src/a.rs\"\n---\n# 001-a\n\nBody, edited.\n",
    );
    assert!(!fresh(check_registry_freshness(&cfg, r).unwrap()));
    assert!(!fresh(check_index_freshness(&cfg, r).unwrap()));
}

/// §3.1: only CRLF folds. A lone CR, or a trailing space, is still a
/// different file.
#[test]
fn only_crlf_folds() {
    assert!(shard::same_committed_text(
        b"{\n  \"a\": 1\n}\n",
        "{\n  \"a\": 1\n}\n"
    ));
    assert!(shard::same_committed_text(
        b"{\r\n  \"a\": 1\r\n}\r\n",
        "{\n  \"a\": 1\n}\n"
    ));
    assert!(!shard::same_committed_text(
        b"{\r  \"a\": 1\r}\r",
        "{\n  \"a\": 1\n}\n"
    ));
    assert!(!shard::same_committed_text(
        b"{\r\n  \"a\": 1 \r\n}\r\n",
        "{\n  \"a\": 1\n}\n"
    ));
    assert!(!shard::same_committed_text(
        b"{\n  \"a\": 2\n}\n",
        "{\n  \"a\": 1\n}\n"
    ));
}
