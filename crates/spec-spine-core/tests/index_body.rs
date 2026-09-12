//! Spec 086: the committed index is compared, not trusted.
//!
//! The tamper matrix against the committed index, library side. Every case here
//! rewrites the committed tree and asks `check_index_freshness` what it makes of
//! it, because before spec 086 that function never compared the shard *body*
//! with anything: it read each committed mapping, asked that mapping which files
//! backed its spans, and compared a recomputed hash with the shard's own
//! `shardHash` field.
//!
//! The fixture is deliberately span-free (one `file` unit, no `symbol` or
//! `section` claims). `span_files_for_mapping` folds a source file into the
//! shard hash only for a `section`, `symbol` or `module` unit, so for a `file`
//! unit the hash covers `spec.md` and the global-inputs scalar and nothing else.
//! Rewriting a path inside such a body therefore cannot move `shardHash`, which
//! is what makes these assertions evidence rather than coincidence: they fail
//! against pre-086 code with the committed tree reading fresh, and they cannot
//! start passing because a hash happened to change.
//!
//! Being span-free also keeps the fixture identical with and without the
//! `symbol-resolution` feature (spec 027), which CI builds both ways.

use std::fs;
use std::path::{Path, PathBuf};

use spec_spine_core::shard::{self, BY_PACKAGE_DIR, BY_SPEC_DIR};
use spec_spine_core::{
    Freshness, check_index_freshness, compile, index, index_dir, index_shard_files, registry_dir,
    registry_shard_files,
};
use spec_spine_types::{Config, Error};

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, content).unwrap();
}

/// One crate, one spec, one `file` unit: the shape 086 measured, and the common
/// case in an adopter corpus.
fn fixture() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    let r = tmp.path();
    write(r, "Cargo.toml", "[workspace]\nmembers = [\"a\"]\n");
    // No `[package.metadata.spec-spine]` stanza, deliberately: a manifest
    // spec_ref would put a *floor* under every file in the crate, and a floor
    // survives a tampered `establishes` claim. Spec 086 §1 measured the case
    // where the specific claim is the only ownership, which is the case a
    // `C-001` decision turns on.
    write(
        r,
        "a/Cargo.toml",
        "[package]\nname = \"a\"\nversion = \"0.1.0\"\n",
    );
    write(r, "a/src/lib.rs", "pub fn a() {}\n");
    write(
        r,
        "specs/001-a/spec.md",
        "---\nid: \"001-a\"\ntitle: \"T\"\nstatus: approved\ncreated: \"2026-09-11\"\nimplementation: complete\nsummary: \"s\"\nestablishes:\n  - \"a/src/lib.rs\"\n---\n\n# 001-a\n",
    );
    tmp
}

/// Write both committed trees exactly as `spec-spine compile` and
/// `spec-spine index` do.
///
/// The registry is written too, though only the index is under test, so that a
/// reader verb reaches the question this file is about. Without it
/// `owner` fails on the absent registry, and the tamper probe below would pass
/// against pre-086 code for a reason that has nothing to do with the tamper.
fn emit(cfg: &Config, repo: &Path) {
    let registry = compile(cfg, repo).unwrap();
    shard::sync_dir(
        &registry_dir(cfg, repo).join(BY_SPEC_DIR),
        &registry_shard_files(&registry.shards).unwrap(),
    )
    .unwrap();

    let outcome = index(cfg, repo).unwrap();
    let dir = index_dir(cfg, repo);
    let (by_spec, by_package) = index_shard_files(&outcome.shards).unwrap();
    shard::sync_dir(&dir.join(BY_SPEC_DIR), &by_spec).unwrap();
    shard::sync_dir(&dir.join(BY_PACKAGE_DIR), &by_package).unwrap();
}

fn spec_shard(cfg: &Config, repo: &Path, id: &str) -> PathBuf {
    index_dir(cfg, repo)
        .join(BY_SPEC_DIR)
        .join(format!("{id}.json"))
}

/// The `shardHash` line, so a test can prove a tamper left it alone.
fn hash_line(text: &str) -> String {
    text.lines()
        .find(|l| l.contains("\"shardHash\""))
        .expect("a shard carries its own hash")
        .to_string()
}

/// The drift report, or a panic naming the verdict that was expected to be stale.
fn stale_report(cfg: &Config, repo: &Path) -> String {
    match check_index_freshness(cfg, repo).unwrap() {
        Freshness::Stale { actual, .. } => actual,
        Freshness::Fresh => {
            panic!("the committed index must read STALE here, and it read fresh")
        }
    }
}

#[test]
fn a_regenerated_tree_reads_fresh() {
    // Spec 086 §3.5: the byte comparison must not cost the ordinary case. A tree
    // the indexer just wrote is what every green run looks like.
    let fx = fixture();
    let cfg = Config::default();
    emit(&cfg, fx.path());
    assert_eq!(
        check_index_freshness(&cfg, fx.path()).unwrap(),
        Freshness::Fresh
    );
}

#[test]
fn a_rewritten_body_reads_modified_though_its_own_hash_still_matches() {
    // The defect spec 086 §1 measured, in one assertion: the committed body says
    // 001-a owns `a/src/zz.rs`, the corpus says it owns `a/src/lib.rs`, and
    // `shardHash` is untouched and still correct for its own declared inputs.
    // Pre-086 this read fresh, and `couple` then derived ownership from it.
    let fx = fixture();
    let cfg = Config::default();
    emit(&cfg, fx.path());

    let path = spec_shard(&cfg, fx.path(), "001-a");
    let before = fs::read_to_string(&path).unwrap();
    let after = before.replace("a/src/lib.rs", "a/src/zz.rs");
    assert_ne!(before, after, "the tamper must actually change the body");
    fs::write(&path, &after).unwrap();

    assert_eq!(
        hash_line(&before),
        hash_line(&after),
        "the tamper must leave `shardHash` alone, or this test proves nothing: a \
         `file` unit contributes no span file, so the hash cannot move"
    );

    let report = stale_report(&cfg, fx.path());
    assert!(
        report.contains("modified by-spec/001-a.json"),
        "the drifted shard must be named with its class: {report}"
    );
}

#[test]
fn an_owner_query_refuses_a_tampered_tree() {
    // Spec 086 §3.3: every reader inherits the comparison, so the owner set a
    // C-001 decision uses is the one the corpus resolves to. Pre-086 this query
    // answered "no spec owns this path" from the rewritten body, with exit 0.
    let fx = fixture();
    let cfg = Config::default();
    emit(&cfg, fx.path());

    let path = spec_shard(&cfg, fx.path(), "001-a");
    let body = fs::read_to_string(&path).unwrap();
    fs::write(&path, body.replace("a/src/lib.rs", "a/src/zz.rs")).unwrap();

    match spec_spine_core::owner(&cfg, fx.path(), "a/src/lib.rs") {
        Err(Error::Stale { .. }) => {}
        other => panic!("an owner query over a tampered tree must refuse, got: {other:?}"),
    }
}

#[test]
fn a_deleted_shard_reads_missing() {
    let fx = fixture();
    let cfg = Config::default();
    emit(&cfg, fx.path());
    fs::remove_file(spec_shard(&cfg, fx.path(), "001-a")).unwrap();

    let report = stale_report(&cfg, fx.path());
    assert!(
        report.contains("missing by-spec/001-a.json"),
        "a spec with no committed shard is `missing`: {report}"
    );
}

#[test]
fn a_stray_file_reads_orphaned() {
    // Spec 086 §3.1 counts a stray file in either shard directory as orphaned.
    // It is classified by name, without being parsed: the comparison is what
    // decides, so a file that would not deserialize is drift to report rather
    // than an error to raise.
    let fx = fixture();
    let cfg = Config::default();
    emit(&cfg, fx.path());
    fs::write(
        index_dir(&cfg, fx.path())
            .join(BY_SPEC_DIR)
            .join("099-ghost.json"),
        "{ not even valid json\n",
    )
    .unwrap();

    let report = stale_report(&cfg, fx.path());
    assert!(
        report.contains("orphaned by-spec/099-ghost.json"),
        "a committed file no spec accounts for is `orphaned`: {report}"
    );
}

#[test]
fn a_package_shard_body_edit_reads_modified() {
    // The package half has the same hole: `package_shard_hash` hashes the
    // manifest named by the committed body, so rewriting the body's own recorded
    // version leaves the hash correct. Pre-086 this read fresh.
    let fx = fixture();
    let cfg = Config::default();
    emit(&cfg, fx.path());

    let dir = index_dir(&cfg, fx.path()).join(BY_PACKAGE_DIR);
    let path = fs::read_dir(&dir)
        .unwrap()
        .filter_map(std::result::Result::ok)
        .map(|e| e.path())
        .find(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
        .expect("the fixture discovers a package");
    let before = fs::read_to_string(&path).unwrap();
    let after = before.replace("0.1.0", "9.9.9");
    assert_ne!(before, after, "the fixture must record a package version");
    fs::write(&path, &after).unwrap();
    assert_eq!(
        hash_line(&before),
        hash_line(&after),
        "the tamper must leave `shardHash` alone: the hash is over the manifest"
    );

    let report = stale_report(&cfg, fx.path());
    assert!(
        report.contains("modified by-package/"),
        "a rewritten package body is `modified`: {report}"
    );
}

#[test]
fn a_blocking_shard_that_also_differs_is_named_once() {
    // One line per shard, so the count line cannot overstate how many shards
    // moved. A spec claiming a file that does not exist emits `I-004`, which is
    // blocking, and tampering its committed body also makes the bytes differ;
    // both conditions hold at once. The blocking line is the one kept, because
    // it is the one whose remedy differs: regenerating fixes bytes and does not
    // fix an unresolved unit.
    let fx = fixture();
    let cfg = Config::default();
    write(
        fx.path(),
        "specs/002-gone/spec.md",
        "---\nid: \"002-gone\"\ntitle: \"T\"\nstatus: approved\ncreated: \"2026-09-12\"\nimplementation: complete\nsummary: \"s\"\nestablishes:\n  - \"a/src/absent.rs\"\n---\n\n# 002-gone\n",
    );
    emit(&cfg, fx.path());

    let path = spec_shard(&cfg, fx.path(), "002-gone");
    let body = fs::read_to_string(&path).unwrap();
    let tampered = body.replace("does not exist", "does not exist ");
    assert_ne!(body, tampered, "the tamper must change the committed bytes");
    fs::write(&path, tampered).unwrap();

    let report = stale_report(&cfg, fx.path());
    assert!(
        report.contains("blocking-diagnostics by-spec/002-gone.json"),
        "the blocking refusal must survive (spec 050): {report}"
    );
    assert!(
        !report.contains("modified by-spec/002-gone.json"),
        "the same shard must not also be named `modified`: {report}"
    );
    assert_eq!(
        report.matches("by-spec/002-gone.json").count(),
        1,
        "one line per shard, so the count line stays honest: {report}"
    );
}

#[test]
fn a_foreign_schema_major_is_refused_not_reported_as_drift() {
    // A must-keep-working line, not a defect probe: it passes before and after
    // spec 086. The read boundary refused a shard from an unknown MAJOR with a
    // clean schema error, and byte comparison must not turn that into "stale,
    // run `spec-spine index`", whose remedy would overwrite a tree written by a
    // newer build.
    let fx = fixture();
    let cfg = Config::default();
    emit(&cfg, fx.path());

    let path = spec_shard(&cfg, fx.path(), "001-a");
    let body = fs::read_to_string(&path).unwrap();
    let major: u32 = spec_spine_types::INDEX_SCHEMA_VERSION
        .split('.')
        .next()
        .unwrap()
        .parse()
        .unwrap();
    fs::write(
        &path,
        body.replace(
            &format!("\"{}\"", spec_spine_types::INDEX_SCHEMA_VERSION),
            &format!("\"{}.0.0\"", major + 1),
        ),
    )
    .unwrap();

    match check_index_freshness(&cfg, fx.path()) {
        Err(Error::Schema(msg)) => assert!(
            msg.contains("MAJOR"),
            "the refusal must name the schema MAJOR: {msg}"
        ),
        other => panic!("a foreign schema MAJOR must be refused, got: {other:?}"),
    }
}

#[test]
fn a_schema_restamp_inside_our_major_reads_modified() {
    // Spec 086 §3.1: the byte comparison catches a stale hash, a hand-edited
    // body and a schema restamp alike. A MINOR restamp stays inside the MAJOR
    // this build understands, so it is drift to report, not a schema error: the
    // remedy is to regenerate, which is what a stale verdict says.
    let fx = fixture();
    let cfg = Config::default();
    emit(&cfg, fx.path());

    let path = spec_shard(&cfg, fx.path(), "001-a");
    let body = fs::read_to_string(&path).unwrap();
    let major = spec_spine_types::INDEX_SCHEMA_VERSION
        .split('.')
        .next()
        .unwrap()
        .to_string();
    let restamped = body.replace(
        &format!("\"{}\"", spec_spine_types::INDEX_SCHEMA_VERSION),
        &format!("\"{major}.99.0\""),
    );
    assert_ne!(body, restamped, "the fixture must stamp a schema version");
    fs::write(&path, restamped).unwrap();

    let report = stale_report(&cfg, fx.path());
    assert!(
        report.contains("modified by-spec/001-a.json"),
        "a schema restamp is `modified`: {report}"
    );
}
