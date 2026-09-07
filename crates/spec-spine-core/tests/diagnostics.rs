//! Committed-diagnostics reader tests (spec 050).
//!
//! The tier split under test is spec 025's: `W-002` for a non-owning
//! `references` edge, `W-001` for an owning edge on a spec in flight, and the
//! `I-0xx` hard error otherwise. Nothing here changes that classification; these
//! assert that what it recorded can be read back, counted, and attributed to the
//! spec that caused it.

use std::fs;
use std::path::Path;

use spec_spine_core::shard::{self, BY_PACKAGE_DIR, BY_SPEC_DIR};
use spec_spine_core::{
    AttributedDiagnostic, UNRESOLVED_CODES, committed_counts, committed_diagnostics,
    count_diagnostics as count, index, index_dir, index_shard_files,
};
use spec_spine_types::{Config, Severity, load_config};

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, content).unwrap();
}

fn emit_index(cfg: &Config, repo: &Path) {
    let outcome = index(cfg, repo).unwrap();
    let dir = index_dir(cfg, repo);
    let (by_spec, by_package) = index_shard_files(&outcome.shards).unwrap();
    shard::sync_dir(&dir.join(BY_SPEC_DIR), &by_spec).unwrap();
    shard::sync_dir(&dir.join(BY_PACKAGE_DIR), &by_package).unwrap();
}

fn cfg() -> Config {
    load_config("").unwrap()
}

fn diag(spec: &str, code: &str, severity: Severity) -> AttributedDiagnostic {
    AttributedDiagnostic {
        spec_id: spec.to_string(),
        severity,
        code: code.to_string(),
        message: "m".to_string(),
        path: None,
    }
}

// --- counting (no corpus needed) ------------------------------------------

#[test]
fn counts_split_by_tier_and_code() {
    let c = count(&[
        diag("001-a", "W-001", Severity::Warning),
        diag("001-a", "W-001", Severity::Warning),
        diag("002-b", "W-002", Severity::Warning),
        diag("003-c", "I-004", Severity::Error),
    ]);
    assert_eq!(c.warnings, 3);
    assert_eq!(c.errors, 1);
    assert_eq!(c.by_code.get("W-001"), Some(&2));
    assert_eq!(c.by_code.get("W-002"), Some(&1));
    assert_eq!(c.by_code.get("I-004"), Some(&1));
}

#[test]
fn zero_entries_are_omitted_from_by_code() {
    let c = count(&[diag("001-a", "W-001", Severity::Warning)]);
    assert_eq!(c.by_code.len(), 1, "{:?}", c.by_code);
    assert!(!c.by_code.contains_key("W-002"));
}

#[test]
fn empty_is_empty() {
    let c = count(&[]);
    assert!(c.is_empty());
    assert!(!c.has_unresolved());
    assert!(c.by_code.is_empty());
}

#[test]
fn has_unresolved_covers_both_warning_codes() {
    for code in UNRESOLVED_CODES {
        let c = count(&[diag("001-a", code, Severity::Warning)]);
        assert!(c.has_unresolved(), "{code} must count as unresolved");
    }
}

#[test]
fn an_error_tier_diagnostic_is_not_unresolved_for_the_flag() {
    // `--fail-on-unresolved` is defined over the warning codes. The error tier
    // is already gated: its shard reads as stale and `index check` exits 2.
    let c = count(&[diag("001-a", "I-004", Severity::Error)]);
    assert!(!c.is_empty());
    assert!(!c.has_unresolved());
}

// --- reading the committed shards -----------------------------------------

/// A corpus with one of each warning tier and nothing else.
///
/// `001-flight` is `draft` + `pending`, so its unresolved **owning** claim is a
/// `W-001` (spec 025). `002-ref` is `approved` + `complete` but the unresolved
/// unit is a non-owning `references` edge, so it is a `W-002` at any lifecycle.
fn fixture() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write(
        root,
        "Cargo.toml",
        "[workspace]\nmembers = [\"crates/a\"]\n",
    );
    write(
        root,
        "crates/a/Cargo.toml",
        "[package]\nname = \"a\"\nversion = \"0.1.0\"\n\n[package.metadata.spec-spine]\nspec = \"001-flight\"\n",
    );
    write(
        root,
        "crates/a/src/lib.rs",
        "// Spec: specs/001-flight/spec.md\npub fn a() {}\n",
    );
    write(
        root,
        "specs/001-flight/spec.md",
        "---\nid: \"001-flight\"\ntitle: \"T\"\nstatus: draft\ncreated: \"2026-09-06\"\nimplementation: pending\nsummary: \"s\"\nestablishes:\n  - \"crates/a/src/lib.rs\"\n  - \"crates/a/src/not_yet.rs\"\n---\n# 001\n",
    );
    write(
        root,
        "specs/002-ref/spec.md",
        "---\nid: \"002-ref\"\ntitle: \"T\"\nstatus: approved\ncreated: \"2026-09-06\"\nimplementation: complete\nsummary: \"s\"\nreferences:\n  - { unit: { kind: file, path: \"docs/absent.md\" }, role: context }\n---\n# 002\n",
    );
    emit_index(&cfg(), root);
    tmp
}

#[test]
fn both_warning_tiers_are_read_back_with_their_spec() {
    let t = fixture();
    let diags = committed_diagnostics(&cfg(), t.path()).unwrap();
    let found: Vec<_> = diags
        .iter()
        .map(|d| (&*d.spec_id, &*d.code, d.severity))
        .collect();
    assert!(
        found.contains(&("001-flight", "W-001", Severity::Warning)),
        "{found:?}"
    );
    assert!(
        found.contains(&("002-ref", "W-002", Severity::Warning)),
        "{found:?}"
    );
}

#[test]
fn the_diagnostic_names_the_unit_that_did_not_resolve() {
    let t = fixture();
    let diags = committed_diagnostics(&cfg(), t.path()).unwrap();
    let w1 = diags.iter().find(|d| d.code == "W-001").unwrap();
    assert!(
        w1.message.contains("not_yet.rs")
            || w1.path.as_deref().is_some_and(|p| p.contains("not_yet")),
        "the evidence must survive the downgrade: {w1:?}"
    );
}

#[test]
fn counts_match_the_listing() {
    let t = fixture();
    let listing = committed_diagnostics(&cfg(), t.path()).unwrap();
    let counts = committed_counts(&cfg(), t.path()).unwrap();
    assert_eq!(counts, count(&listing));
    assert!(counts.has_unresolved());
}

#[test]
fn the_listing_is_sorted_and_deterministic() {
    let t = fixture();
    let first = committed_diagnostics(&cfg(), t.path()).unwrap();
    for _ in 0..5 {
        assert_eq!(committed_diagnostics(&cfg(), t.path()).unwrap(), first);
    }
    let mut sorted = first.clone();
    sorted.sort_by(|a, b| {
        a.spec_id
            .cmp(&b.spec_id)
            .then(a.code.cmp(&b.code))
            .then(a.path.cmp(&b.path))
            .then(a.message.cmp(&b.message))
    });
    assert_eq!(first, sorted, "the reader must emit sorted order");
}

#[test]
fn a_clean_corpus_records_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write(
        root,
        "Cargo.toml",
        "[workspace]\nmembers = [\"crates/a\"]\n",
    );
    write(
        root,
        "crates/a/Cargo.toml",
        "[package]\nname = \"a\"\nversion = \"0.1.0\"\n\n[package.metadata.spec-spine]\nspec = \"001-a\"\n",
    );
    write(
        root,
        "crates/a/src/lib.rs",
        "// Spec: specs/001-a/spec.md\npub fn a() {}\n",
    );
    write(
        root,
        "specs/001-a/spec.md",
        "---\nid: \"001-a\"\ntitle: \"T\"\nstatus: approved\ncreated: \"2026-09-06\"\nimplementation: complete\nsummary: \"s\"\nestablishes:\n  - \"crates/a/src/lib.rs\"\n---\n# 001\n",
    );
    emit_index(&cfg(), root);

    let counts = committed_counts(&cfg(), root).unwrap();
    assert!(counts.is_empty(), "{counts:?}");
    assert!(!counts.has_unresolved());
    assert!(committed_diagnostics(&cfg(), root).unwrap().is_empty());
}

#[test]
fn the_reader_reads_the_committed_shards_not_a_fresh_run() {
    let t = fixture();
    // Create the missing file *without* re-emitting. A fresh index run would
    // now resolve it; the committed shards still record the warning, and spec
    // 050 3.5 says the reader must describe the committed ledger.
    write(t.path(), "crates/a/src/not_yet.rs", "pub fn b() {}\n");
    let counts = committed_counts(&cfg(), t.path()).unwrap();
    assert!(
        counts.has_unresolved(),
        "the reader must not recompute: {counts:?}"
    );
}
