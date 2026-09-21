//! Spec 069: the committed index is compared, not trusted.
//!
//! The tamper matrix against the committed index, library side. Every case here
//! rewrites the committed tree and asks `check_index_freshness` what it makes of
//! it, because before spec 069 that function never compared the shard *body*
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
//! `symbol-resolution` feature (spec 025), which CI builds both ways.

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
    // survives a tampered `establishes` claim. Spec 069 §1 measured the case
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

/// Restamp a shard's schema version to another MINOR of the same MAJOR.
///
/// The stable way to make a committed body differ. Rewriting some phrase out of
/// the body would couple the test to wording it is not about (a reworded
/// diagnostic would fire the "the tamper changed nothing" guard and send the
/// reader after the wrong thing), whereas `schemaVersion` is a constant this
/// crate exports and every shard carries. Staying inside the MAJOR keeps it
/// drift rather than the refusal `reject_foreign_major` raises.
fn restamp_minor(body: &str) -> String {
    let current = spec_spine_types::INDEX_SCHEMA_VERSION;
    let major = current.split('.').next().expect("a semver MAJOR");
    let out = body.replace(&format!("\"{current}\""), &format!("\"{major}.99.0\""));
    assert_ne!(
        body, out,
        "every shard stamps {current}; the fixture must carry one to restamp"
    );
    out
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
    // Spec 069 §3.5: the byte comparison must not cost the ordinary case. A tree
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
    // The defect spec 069 §1 measured, in one assertion: the committed body says
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
    // Spec 069 §3.3: every reader inherits the comparison, so the owner set a
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
    // Spec 069 §3.1 counts a stray file in either shard directory as orphaned.
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
    assert!(
        body.contains("I-004"),
        "the fixture must make this shard block, or the test asserts nothing: {body}"
    );
    fs::write(&path, restamp_minor(&body)).unwrap();

    let report = stale_report(&cfg, fx.path());
    assert!(
        report.contains("blocking-diagnostics by-spec/002-gone.json"),
        "the blocking refusal must survive (spec 044): {report}"
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
fn a_blocking_shard_that_is_also_missing_is_named_once() {
    // The other pairing, and the reason the guard matches on the file rather
    // than on one class name: a new spec whose unit does not resolve, never
    // indexed, is blocking (from the fresh index) and missing (from the
    // committed tree) at the same time. A guard naming only `modified` would
    // let this one through while reading as though it covered every case.
    let fx = fixture();
    let cfg = Config::default();
    emit(&cfg, fx.path());

    // Filed after the tree was written, so no shard for it was ever committed.
    write(
        fx.path(),
        "specs/002-gone/spec.md",
        "---\nid: \"002-gone\"\ntitle: \"T\"\nstatus: approved\ncreated: \"2026-09-12\"\nimplementation: complete\nsummary: \"s\"\nestablishes:\n  - \"a/src/absent.rs\"\n---\n\n# 002-gone\n",
    );

    let report = stale_report(&cfg, fx.path());
    assert!(
        report.contains("blocking-diagnostics by-spec/002-gone.json"),
        "the blocking refusal must survive (spec 044): {report}"
    );
    assert_eq!(
        report.matches("by-spec/002-gone.json").count(),
        1,
        "one line per shard, `missing` included: {report}"
    );
    assert!(
        report.starts_with("1 stale shard(s):"),
        "the count line must agree with the lines under it: {report}"
    );
}

#[test]
fn a_foreign_schema_major_is_refused_not_reported_as_drift() {
    // A must-keep-working line, not a defect probe: it passes before and after
    // spec 069. The read boundary refused a shard from an unknown MAJOR with a
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
    // Spec 069 §3.1: the byte comparison catches a stale hash, a hand-edited
    // body and a schema restamp alike. A MINOR restamp stays inside the MAJOR
    // this build understands, so it is drift to report, not a schema error: the
    // remedy is to regenerate, which is what a stale verdict says.
    let fx = fixture();
    let cfg = Config::default();
    emit(&cfg, fx.path());

    let path = spec_shard(&cfg, fx.path(), "001-a");
    let body = fs::read_to_string(&path).unwrap();
    fs::write(&path, restamp_minor(&body)).unwrap();

    let report = stale_report(&cfg, fx.path());
    assert!(
        report.contains("modified by-spec/001-a.json"),
        "a schema restamp is `modified`: {report}"
    );
}

#[test]
fn the_facade_keeps_the_verdict_when_a_shard_will_not_parse() {
    // Spec 076 §3.1, §3.6: `a_stray_file_reads_orphaned` above is still true and
    // is not sufficient, because the verbs discarded that verdict *after* this
    // function returned. The facade is one of the four entry points (§3.4), so
    // it is asserted here, over both routes to drift: an unexpected stray and an
    // expected shard corrupted in place.
    let fx = fixture();
    let cfg = Config::default();
    emit(&cfg, fx.path());
    let repo = fx.path().to_str().unwrap();
    fs::write(
        index_dir(&cfg, fx.path())
            .join(BY_SPEC_DIR)
            .join("099-ghost.json"),
        "{ not even valid json\n",
    )
    .unwrap();
    fs::write(spec_shard(&cfg, fx.path(), "001-a"), "{\"truncated\": ").unwrap();

    let report: serde_json::Value =
        serde_json::from_str(&spec_spine_core::check_freshness_json("{}", repo).unwrap())
            .expect("the verdict survives the tally");
    assert_eq!(report["fresh"], false, "{report}");
    assert_eq!(report["skippedShards"], 2, "{report}");
    let actual = report["actual"].as_str().unwrap();
    assert!(
        actual.contains("orphaned by-spec/099-ghost.json"),
        "{actual}"
    );
    assert!(actual.contains("modified by-spec/001-a.json"), "{actual}");

    let composed = spec_spine_core::check_report(&cfg, fx.path()).expect("no parse error");
    assert_eq!(composed.index.skipped_shards, 2);
    assert_eq!(
        composed.index.diagnostics.unreadable,
        vec!["by-spec/001-a.json", "by-spec/099-ghost.json"]
    );
}

// --- spec 079: a blocking claim is not a stale shard -------------------------
//
// Spec 069 made this file the place the committed index is interrogated, and
// 079's subject is the verdict that interrogation produces: `check_index_
// freshness` folded two independent refusals into one `Stale`, which carries
// exactly one remedy. The cases below are the four states of that partition,
// asserted as data. The regeneration case (AC-2) is the one the old output got
// wrong and is asserted directly rather than inferred.

/// Add a spec that claims a unit which does not exist, with the lifecycle axes
/// that make the claim *block* rather than warn.
///
/// `status` and `implementation` are both passed because they decide the tier:
/// spec 023 §3.1 arm 2 with spec 038's table makes `approved` + `complete` and
/// `approved` + `deferred` blocking, and anything in flight a `W-001` warning.
fn claim_a_missing_unit(root: &Path, id: &str, status: &str, implementation: &str) {
    write(
        root,
        &format!("specs/{id}/spec.md"),
        &format!(
            "---\nid: \"{id}\"\ntitle: \"T\"\nstatus: {status}\ncreated: \"2026-09-16\"\n\
             implementation: {implementation}\nsummary: \"s\"\nestablishes:\n  - \"src/gone.rs\"\n\
             ---\n\n# {id}\n"
        ),
    );
}

fn report(cfg: &Config, repo: &Path) -> spec_spine_core::IndexFreshnessReport {
    spec_spine_core::index_freshness_report(cfg, repo).unwrap()
}

#[test]
fn a_blocking_claim_is_carried_as_data_not_as_a_stale_shard() {
    // Spec 079 §3.2, FR-002/FR-004: the partition reaches the caller as data,
    // with the code, the owning spec and the unit on it. Before 098 all of that
    // survived only as the text `blocking-diagnostics by-spec/<id>.json`.
    let fx = fixture();
    let cfg = Config::default();
    claim_a_missing_unit(fx.path(), "002-missing", "approved", "complete");
    emit(&cfg, fx.path());

    let r = report(&cfg, fx.path());
    assert!(
        r.stale.is_empty(),
        "the committed tree was just written: nothing moved, yet {:?}",
        r.stale
    );
    assert_eq!(r.blocking.len(), 1, "{:?}", r.blocking);
    let c = &r.blocking[0];
    assert_eq!(c.code, "I-004");
    assert_eq!(c.spec_id, "002-missing");
    assert_eq!(c.unit.as_deref(), Some("src/gone.rs"));
    assert!(
        c.claims_complete,
        "the spec declares implementation: complete"
    );
    assert!(!r.is_fresh());

    // FR-001: the verdict every existing caller reads is unchanged, down to the
    // line it has carried since spec 044.
    assert_eq!(
        r.freshness(),
        check_index_freshness(&cfg, fx.path()).unwrap(),
        "the fold and the partition must answer alike"
    );
    match r.freshness() {
        Freshness::Stale { actual, .. } => assert!(
            actual.contains("blocking-diagnostics by-spec/002-missing.json"),
            "{actual}"
        ),
        Freshness::Fresh => panic!("a blocking claim still refuses"),
    }
}

#[test]
fn regenerating_does_not_clear_a_blocking_claim() {
    // Spec 079 AC-2, the regression the old message promised away: `index`
    // exits 0, writes the same bytes, and the next read refuses identically,
    // because the diagnostic is recomputed from the corpus and re-indexing
    // cannot create a file a spec claims.
    let fx = fixture();
    let cfg = Config::default();
    claim_a_missing_unit(fx.path(), "002-missing", "approved", "complete");
    emit(&cfg, fx.path());
    let before = report(&cfg, fx.path());

    emit(&cfg, fx.path()); // the remedy the old verdict named
    let after = report(&cfg, fx.path());

    assert_eq!(
        before.blocking, after.blocking,
        "regeneration changed nothing"
    );
    assert!(
        after.stale.is_empty(),
        "and there was never a stale shard to repair: {:?}",
        after.stale
    );
    let lines = after.unresolved_claim_lines().join("\n");
    assert!(
        lines.contains("regenerating the index does not clear this"),
        "{lines}"
    );
}

#[test]
fn a_mixed_tree_reports_both_and_attributes_the_remedy_to_one() {
    // Spec 079 §3.3 mixed, FR-006. A reader who regenerates must find the second
    // half still refusing, and must have been told so in advance.
    let fx = fixture();
    let cfg = Config::default();
    claim_a_missing_unit(fx.path(), "002-missing", "approved", "complete");
    emit(&cfg, fx.path());
    let shard = spec_shard(&cfg, fx.path(), "001-a");
    let body = fs::read_to_string(&shard).unwrap();
    fs::write(&shard, restamp_minor(&body)).unwrap();

    let r = report(&cfg, fx.path());
    assert_eq!(r.blocking.len(), 1, "{:?}", r.blocking);
    assert_eq!(r.stale, vec!["modified by-spec/001-a.json".to_string()]);
    let lines = r.unresolved_claim_lines().join("\n");
    assert!(
        lines.contains(
            "regenerating addresses the stale shard(s) only, not the unresolved claim(s)"
        ),
        "{lines}"
    );

    // Neither half is elided, and the stale half names its shard with its class
    // (spec 028 §3.3, spec 069 §3.2).
    match r.stale_verdict() {
        Freshness::Stale { actual, .. } => {
            assert!(actual.contains("modified by-spec/001-a.json"), "{actual}");
            assert!(
                !actual.contains("blocking-diagnostics"),
                "the stale half counts only what moved: {actual}"
            );
        }
        Freshness::Fresh => panic!("a restamped shard is drift"),
    }

    // After regenerating, the stale half is gone and the blocking half stands.
    emit(&cfg, fx.path());
    let after = report(&cfg, fx.path());
    assert!(after.stale.is_empty(), "{:?}", after.stale);
    assert_eq!(after.blocking.len(), 1);
}

#[test]
fn the_stale_only_verdict_is_unchanged() {
    // Spec 079 FR-008 / AC-3: a caller that reads staleness today reads exactly
    // the same staleness after this spec, wording included. With nothing
    // blocking, the two renderings are the same value.
    let fx = fixture();
    let cfg = Config::default();
    emit(&cfg, fx.path());
    let shard = spec_shard(&cfg, fx.path(), "001-a");
    let body = fs::read_to_string(&shard).unwrap();
    fs::write(&shard, restamp_minor(&body)).unwrap();

    let r = report(&cfg, fx.path());
    assert!(r.blocking.is_empty());
    assert_eq!(r.freshness(), r.stale_verdict());
    assert_eq!(
        r.freshness(),
        check_index_freshness(&cfg, fx.path()).unwrap()
    );
    assert!(
        r.unresolved_claim_lines().is_empty(),
        "nothing unresolved, so nothing is said about it"
    );
    match r.freshness() {
        Freshness::Stale { actual, .. } => {
            assert!(actual.starts_with("1 stale shard(s):"), "{actual}");
            assert!(actual.contains("  modified by-spec/001-a.json"), "{actual}");
        }
        Freshness::Fresh => panic!("a restamped shard is drift"),
    }
}

#[test]
fn a_healthy_tree_reports_neither() {
    // Spec 079 AC-5.
    let fx = fixture();
    let cfg = Config::default();
    emit(&cfg, fx.path());
    let r = report(&cfg, fx.path());
    assert!(r.is_fresh());
    assert_eq!(r.freshness(), Freshness::Fresh);
    assert!(r.unresolved_claim_lines().is_empty());
}

#[test]
fn the_completion_claim_is_named_only_when_it_was_made() {
    // Spec 079 §3.4 / AC-6. `implementation: complete` is a falsifiable claim
    // that the files exist (spec 038), so a blocking diagnostic against it is a
    // contradiction in the spec's own frontmatter and the report says so. A spec
    // that blocks *without* declaring completion is refused for the same code
    // and accused of nothing further.
    let fx = fixture();
    let cfg = Config::default();
    claim_a_missing_unit(fx.path(), "002-missing", "approved", "complete");
    emit(&cfg, fx.path());
    let lines = report(&cfg, fx.path()).unresolved_claim_lines().join("\n");
    assert!(
        lines.contains("declares `implementation: complete`"),
        "{lines}"
    );
    assert!(lines.contains("the spec and the tree disagree"), "{lines}");

    // `approved` + `deferred` is not in flight (spec 038's table), so it still
    // blocks, and it claims no completion.
    claim_a_missing_unit(fx.path(), "002-missing", "approved", "deferred");
    emit(&cfg, fx.path());
    let r = report(&cfg, fx.path());
    assert_eq!(r.blocking.len(), 1, "{:?}", r.blocking);
    assert!(!r.blocking[0].claims_complete);
    let lines = r.unresolved_claim_lines().join("\n");
    assert!(!lines.contains("implementation: complete"), "{lines}");

    // And the in-flight arm blocks nothing at all, which is why it cannot be
    // the negative above (spec 079 D-5).
    claim_a_missing_unit(fx.path(), "002-missing", "draft", "in-progress");
    emit(&cfg, fx.path());
    let r = report(&cfg, fx.path());
    assert!(r.blocking.is_empty(), "{:?}", r.blocking);
    assert!(r.is_fresh());
}

#[test]
fn the_report_offers_no_way_out_of_the_refusal() {
    // Spec 079 §3.4 / AC-7 / D-3. Narrowing a claim until the gate passes is
    // what `AGENTS.md` "Adversarial prompt refusal" exists to refuse, and
    // `planned: true` beneath `implementation: complete` is spec 063 §3.3's
    // `L-011`: a message proposing either would be proposing a defect.
    let fx = fixture();
    let cfg = Config::default();
    claim_a_missing_unit(fx.path(), "002-missing", "approved", "complete");
    emit(&cfg, fx.path());
    let lines = report(&cfg, fx.path()).unresolved_claim_lines().join("\n");
    for forbidden in ["planned: true", "remove the claim", "narrow"] {
        assert!(
            !lines.contains(forbidden),
            "the report must not propose '{forbidden}': {lines}"
        );
    }
}
