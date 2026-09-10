//! Attest (spec 023): determinism (AC-1), recompute match/mismatch (AC-2), the
//! coupling scope and its independently-checkable verdict (AC-3), and
//! version-aware verification (AC-5). The signature round-trip (AC-4) lives in
//! the CLI crate's `seal.rs` unit tests: the core is key-free.

use std::fs;
use std::path::Path;

use spec_spine_core::{AttestOptions, VerifyOutcome, attest, verify_recompute};
use spec_spine_types::Config;

/// Write `specs/<id>/spec.md` under `root` with the given extra frontmatter.
fn write_spec(root: &Path, dir: &str, id: &str, extra: &str) {
    let spec_dir = root.join("specs").join(dir);
    fs::create_dir_all(&spec_dir).unwrap();
    let body = format!(
        "---\nid: \"{id}\"\ntitle: \"Title {id}\"\nstatus: draft\ncreated: \"2026-06-08\"\nsummary: \"s\"\n{extra}---\n# {id}\n"
    );
    fs::write(spec_dir.join("spec.md"), body).unwrap();
}

/// An ownership edge keeps the spec warning-clean (no L-001), so the lint
/// verdict is `ok` under the `--fail-on-warn`-equivalent rule. The claimed file
/// need not exist for the non-coupling cases (compile/lint do not check units).
const OWNED: &str = "establishes:\n  - \"code.txt\"\n";

#[test]
fn ac1_attest_is_byte_identical_on_an_unchanged_corpus() {
    let tmp = tempfile::tempdir().unwrap();
    write_spec(tmp.path(), "001-a", "001-a", OWNED);
    let cfg = Config::default();

    let a = attest(&cfg, tmp.path(), AttestOptions::default()).unwrap();
    let b = attest(&cfg, tmp.path(), AttestOptions::default()).unwrap();

    assert_eq!(a.json, b.json, "attestation must be byte-identical");
    assert_eq!(a.attestation_hash, b.attestation_hash);
    assert!(
        a.attestation.verdicts.couple.is_none(),
        "no coupling block without --with-coupling"
    );
    assert!(a.attestation.verdicts.compile.ok);
    assert!(a.attestation.verdicts.lint.ok);
    assert!(a.json.ends_with("}\n"), "canonical trailing newline");
}

#[test]
fn ac2_recompute_matches_unchanged_then_flags_an_edited_spec() {
    let tmp = tempfile::tempdir().unwrap();
    write_spec(tmp.path(), "001-a", "001-a", OWNED);
    let cfg = Config::default();
    let outcome = attest(&cfg, tmp.path(), AttestOptions::default()).unwrap();

    assert_eq!(
        verify_recompute(&cfg, tmp.path(), &outcome.attestation).unwrap(),
        VerifyOutcome::Match,
        "an unchanged corpus reproduces the attestation"
    );

    // A single spec edit flips it to a named mismatch citing the changed input.
    write_spec(
        tmp.path(),
        "001-a",
        "001-a",
        &format!("{OWNED}owner: \"someone\"\n"),
    );
    match verify_recompute(&cfg, tmp.path(), &outcome.attestation).unwrap() {
        VerifyOutcome::ContentMismatch { differences } => assert!(
            differences
                .iter()
                .any(|d| d.contains("inputsManifestHash") || d.contains("registryHash")),
            "mismatch must cite the changed input/registry: {differences:?}"
        ),
        other => panic!("expected ContentMismatch, got {other:?}"),
    }
}

#[test]
fn ac5_a_different_tool_version_is_a_named_version_mismatch() {
    let tmp = tempfile::tempdir().unwrap();
    write_spec(tmp.path(), "001-a", "001-a", OWNED);
    let cfg = Config::default();
    let mut attestation = attest(&cfg, tmp.path(), AttestOptions::default())
        .unwrap()
        .attestation;

    attestation.tool.version = "0.0.1-other".to_string();
    match verify_recompute(&cfg, tmp.path(), &attestation).unwrap() {
        VerifyOutcome::VersionMismatch { expected, actual } => {
            assert_eq!(expected, "0.0.1-other");
            assert_ne!(actual, "0.0.1-other", "actual is this build's version");
        }
        other => panic!("expected VersionMismatch, got {other:?}"),
    }
}

#[test]
fn ac3_with_coupling_verdict_is_independently_checkable() {
    let tmp = tempfile::tempdir().unwrap();
    // A spec that claims a code unit via a file establishes edge. It must be
    // settled (approved + complete): under spec 025 a missing owning unit only
    // blocks the coupling verdict for a settled spec; a draft/pending spec's
    // unbuilt unit is a non-blocking W-001 (legitimate in-flight work).
    let spec_dir = tmp.path().join("specs").join("001-a");
    fs::create_dir_all(&spec_dir).unwrap();
    fs::write(
        spec_dir.join("spec.md"),
        "---\nid: \"001-a\"\ntitle: \"Title 001-a\"\nstatus: approved\nimplementation: complete\ncreated: \"2026-06-08\"\nsummary: \"s\"\nestablishes:\n  - \"code.txt\"\n---\n# 001-a\n",
    )
    .unwrap();
    fs::write(tmp.path().join("code.txt"), "fn main() {}\n").unwrap();
    let cfg = Config::default();

    // In sync: the claimed unit resolves, so the coupling verdict is ok.
    let synced = attest(
        &cfg,
        tmp.path(),
        AttestOptions {
            with_coupling: true,
        },
    )
    .unwrap();
    let couple = synced
        .attestation
        .verdicts
        .couple
        .as_ref()
        .expect("coupling block present under --with-coupling");
    assert!(couple.ok, "a resolving claim is in sync");
    assert!(!couple.index_hash.is_empty());
    assert!(!couple.join_hash.is_empty());

    // Remove the claimed code: the coupling block fails while the
    // registry/lint scope still verifies (the two scopes are independent).
    fs::remove_file(tmp.path().join("code.txt")).unwrap();
    let drifted = attest(
        &cfg,
        tmp.path(),
        AttestOptions {
            with_coupling: true,
        },
    )
    .unwrap();
    let couple = drifted.attestation.verdicts.couple.as_ref().unwrap();
    assert!(!couple.ok, "a missing claimed unit is not in sync");
    assert!(drifted.attestation.verdicts.compile.ok);
    assert!(drifted.attestation.verdicts.lint.ok);
}

// ===== spec 042: per-spec attestation =====

use spec_spine_core::{attest_spec, verify_spec_recompute};

/// A repo whose one spec claims a file that exists, plus a manifest so the
/// resolver has a package to walk.
fn spec_fixture(extra: &str) -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    let r = tmp.path();
    fs::write(r.join("Cargo.toml"), "[workspace]\nmembers = []\n").unwrap();
    fs::write(r.join("code.txt"), "one\n").unwrap();
    write_spec(r, "001-a", "001-a", extra);
    tmp
}

/// 3.2: the payload is a pure function of `(config, file contents)`. Two runs
/// on an unchanged corpus, and a run from a different working directory, are
/// byte-identical.
#[test]
fn the_payload_is_reproducible() {
    let tmp = spec_fixture(OWNED);
    let cfg = Config::default();
    let first = attest_spec(&cfg, tmp.path(), "001-a").unwrap();
    let second = attest_spec(&cfg, tmp.path(), "001-a").unwrap();
    assert_eq!(first.json, second.json, "no clock, no env, no git");
    assert_eq!(first.attestation_hash, second.attestation_hash);

    // `--repo` from elsewhere: the payload carries no absolute path.
    let relative = attest_spec(&cfg, &tmp.path().canonicalize().unwrap(), "001-a").unwrap();
    assert_eq!(relative.json, first.json);
    assert!(!first.json.contains(tmp.path().to_str().unwrap()));
}

/// Editing an owning unit moves exactly that unit's `contentHash` and the
/// attestation hash, and nothing else.
#[test]
fn editing_a_unit_moves_exactly_that_units_hash() {
    let tmp = spec_fixture(OWNED);
    let cfg = Config::default();
    let before = attest_spec(&cfg, tmp.path(), "001-a").unwrap().attestation;

    fs::write(tmp.path().join("code.txt"), "two\n").unwrap();
    let after = attest_spec(&cfg, tmp.path(), "001-a").unwrap().attestation;

    assert_ne!(before.units[0].content_hash, after.units[0].content_hash);
    assert_eq!(
        before.spec_source_hash, after.spec_source_hash,
        "the spec did not change"
    );
    assert_eq!(before.lifecycle, after.lifecycle);
    assert_eq!(before.verdicts, after.verdicts);
}

/// Editing the spec's own text moves `specSourceHash`.
#[test]
fn editing_the_spec_moves_its_source_hash() {
    let tmp = spec_fixture(OWNED);
    let cfg = Config::default();
    let before = attest_spec(&cfg, tmp.path(), "001-a").unwrap().attestation;

    let path = tmp.path().join("specs/001-a/spec.md");
    let text = fs::read_to_string(&path).unwrap();
    fs::write(&path, text.replace("# 001-a", "# 001-a\n\n## added")).unwrap();
    let after = attest_spec(&cfg, tmp.path(), "001-a").unwrap().attestation;

    assert_ne!(before.spec_source_hash, after.spec_source_hash);
    assert_eq!(before.units, after.units, "the claimed file is untouched");
}

/// 3.1: `lifecycle` mirrors the declared fields. `implementation` is omitted
/// only when the key is absent, and `n-a` is present rather than collapsed into
/// that absence: an absent key means nobody stated an intention, `n-a` means
/// someone stated that none applies.
#[test]
fn lifecycle_distinguishes_an_absent_key_from_n_a() {
    let cfg = Config::default();

    let absent = spec_fixture(OWNED);
    let a = attest_spec(&cfg, absent.path(), "001-a").unwrap();
    assert_eq!(a.attestation.lifecycle.status, "draft");
    assert_eq!(a.attestation.lifecycle.implementation, None);
    assert!(
        !a.json.contains("implementation"),
        "omitted, not null: {}",
        a.json
    );

    let na = spec_fixture(&format!("implementation: n-a\n{OWNED}"));
    let b = attest_spec(&cfg, na.path(), "001-a").unwrap();
    assert_eq!(
        b.attestation.lifecycle.implementation.as_deref(),
        Some("n-a")
    );

    // Spec 015 accepts `n/a` on the way in and normalizes it, so the emitted
    // value is the canonical `n-a` in both dialects. Everything except
    // `specSourceHash` matches; that field hashes the spec's own bytes, which
    // genuinely differ, so it cannot and must not match (spec 042 D-1).
    let slash = spec_fixture(&format!("implementation: n/a\n{OWNED}"));
    let c = attest_spec(&cfg, slash.path(), "001-a").unwrap();
    assert_eq!(
        b.attestation.lifecycle, c.attestation.lifecycle,
        "the dialect is normalized away in the emitted value"
    );
    let (mut b_norm, mut c_norm) = (b.attestation.clone(), c.attestation.clone());
    b_norm.spec_source_hash.clear();
    c_norm.spec_source_hash.clear();
    assert_eq!(b_norm, c_norm, "the dialect changes nothing else");
    assert_ne!(
        b.attestation.spec_source_hash, c.attestation.spec_source_hash,
        "different source bytes must hash differently"
    );

    assert_ne!(
        a.attestation.lifecycle, b.attestation.lifecycle,
        "absent and n-a are distinguishable"
    );
}

/// A `references` unit appears in no `units` entry: spec 034 settled that a
/// cited file is not a claimed one, and an attestation of territory must not
/// assert authority the gate does not.
#[test]
fn a_reference_is_not_attested_territory() {
    let tmp = spec_fixture(
        "establishes:\n  - \"code.txt\"\n\
         references:\n  - { unit: { kind: file, path: \"cited.txt\" }, role: context }\n",
    );
    fs::write(tmp.path().join("cited.txt"), "cited\n").unwrap();
    let out = attest_spec(&Config::default(), tmp.path(), "001-a").unwrap();
    assert_eq!(
        out.attestation.units.len(),
        1,
        "{:?}",
        out.attestation.units
    );
    assert!(!out.json.contains("cited.txt"), "{}", out.json);
}

/// 3.1: an unresolved owning unit yields `resolution.ok: false` and still
/// produces a payload. The flag records the **fact**, never the indexer's
/// severity tier: this spec is `draft`, so its phantom unit is only a `W-001`,
/// and the attestation says `false` regardless.
#[test]
fn an_unresolved_unit_is_recorded_not_refused() {
    let tmp = spec_fixture("establishes:\n  - \"code.txt\"\n  - \"never_written.rs\"\n");
    let out = attest_spec(&Config::default(), tmp.path(), "001-a").unwrap();

    assert!(!out.attestation.verdicts.resolution.ok, "{}", out.json);
    assert_eq!(out.attestation.units.len(), 2);
    let phantom = out
        .attestation
        .units
        .iter()
        .find(|u| out.json.contains("never_written.rs") && u.content_hash.is_none())
        .expect("the phantom unit is present with no hash");
    assert!(phantom.content_hash.is_none(), "no hash of nothing");
    // The record exists and is signable: it is evidence, not a gate.
    assert_eq!(out.attestation_hash.len(), 64);
}

/// 3.5: `--recompute` matches an unchanged corpus, fails with a named outcome
/// after any owned file changes, and a `tool.version` mismatch is a distinct
/// outcome rather than a false content mismatch.
#[test]
fn recompute_matches_then_names_what_moved() {
    let tmp = spec_fixture(OWNED);
    let cfg = Config::default();
    let attestation = attest_spec(&cfg, tmp.path(), "001-a").unwrap().attestation;
    assert_eq!(
        verify_spec_recompute(&cfg, tmp.path(), &attestation).unwrap(),
        VerifyOutcome::Match
    );

    fs::write(tmp.path().join("code.txt"), "changed\n").unwrap();
    let VerifyOutcome::ContentMismatch { differences } =
        verify_spec_recompute(&cfg, tmp.path(), &attestation).unwrap()
    else {
        panic!("an edited unit must be a content mismatch");
    };
    assert!(
        differences.iter().any(|d| d.contains("contentHash")),
        "{differences:?}"
    );

    let mut older = attestation.clone();
    older.tool.version = "0.0.1-not-this-build".to_string();
    let outcome = verify_spec_recompute(&cfg, tmp.path(), &older).unwrap();
    assert!(
        matches!(outcome, VerifyOutcome::VersionMismatch { .. }),
        "a version mismatch is never a false content mismatch: {outcome:?}"
    );
}

/// An id that is not in the corpus is `NotFound` (exit 1), not a payload over
/// nothing.
#[test]
fn an_unknown_spec_id_is_not_found() {
    let tmp = spec_fixture(OWNED);
    let err = attest_spec(&Config::default(), tmp.path(), "999-nope").unwrap_err();
    assert_eq!(err.exit_code(), 1);
}

/// A location that resolves but cannot be read is an error, not a hash of
/// nothing.
///
/// Reached by making the claimed path a **directory**: it exists, so the
/// resolver still resolves the unit to it, and `read_to_string` then fails for
/// every caller regardless of platform or user. Skipping that read would hash
/// an empty set and still emit `Some(hash)` with `resolution.ok` left true, an
/// attestation asserting a resolved unit it never read, which is the claim 3.1
/// says this payload exists to make impossible.
///
/// Deleting the file does **not** reach this path: `attest_spec` re-indexes, so
/// a removed file simply stops resolving and takes the honest
/// `resolution.ok: false` route, which the case below asserts as the control.
#[test]
fn a_location_that_resolves_but_cannot_be_read_is_an_error() {
    let tmp = spec_fixture(OWNED);
    let cfg = Config::default();
    assert!(
        attest_spec(&cfg, tmp.path(), "001-a")
            .unwrap()
            .attestation
            .verdicts
            .resolution
            .ok
    );

    // Control: a removed file stops resolving, which is recorded, not an error.
    fs::remove_file(tmp.path().join("code.txt")).unwrap();
    let gone = attest_spec(&cfg, tmp.path(), "001-a").unwrap().attestation;
    assert!(!gone.verdicts.resolution.ok);
    assert_eq!(gone.units[0].content_hash, None, "no hash of nothing");

    // A directory at the claimed path used to be this test's "unreadable" case
    // and asserted exit 3. Spec 083 3.1 changes that deliberately: a resolved
    // location that is a directory is walked, not opened, because most of this
    // corpus claims subtrees exactly that way. The error path below is what
    // survives of the original assertion.
    fs::create_dir(tmp.path().join("code.txt")).unwrap();
    let walked = attest_spec(&cfg, tmp.path(), "001-a")
        .expect("spec 083 3.1: a directory location is walked, never opened")
        .attestation;
    assert!(walked.verdicts.resolution.ok);
    assert!(
        walked.units[0].content_hash.is_some(),
        "an empty claimed directory still hashes (spec 083 3.3)"
    );

    // Resolvable but genuinely unreadable: the read is what fails, and it must
    // still propagate rather than being skipped (spec 042 3.1, spec 083 3.1).
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::remove_dir(tmp.path().join("code.txt")).unwrap();
        fs::write(tmp.path().join("code.txt"), "x").unwrap();
        fs::set_permissions(
            tmp.path().join("code.txt"),
            fs::Permissions::from_mode(0o000),
        )
        .unwrap();
        // Root ignores the mode bits, so the case is unconstructible there.
        // Detected rather than assumed, since `unsafe` is forbidden here and a
        // uid lookup would need it.
        if fs::read(tmp.path().join("code.txt")).is_err() {
            let err = attest_spec(&cfg, tmp.path(), "001-a").unwrap_err();
            assert_eq!(err.exit_code(), 3, "an unreadable input is an I/O error");
            let message = err.to_string();
            assert!(message.contains("code.txt"), "names the file: {message}");
            assert!(message.contains("001-a"), "and the spec: {message}");
        }
    }
}

/// A changed unit set is reported by unit, not by position.
///
/// Inserting a unit ahead of the others would, under a positional walk, compare
/// every later entry against an unrelated one and report each as a changed
/// hash: a report whose own noise buries the one line that explains it.
#[test]
fn recompute_reports_unit_changes_by_identity_not_position() {
    let tmp = spec_fixture(OWNED);
    let cfg = Config::default();
    let attestation = attest_spec(&cfg, tmp.path(), "001-a").unwrap().attestation;

    // Add a claim that sorts ahead of `code.txt`, leaving its content alone.
    fs::write(tmp.path().join("aaa.txt"), "first\n").unwrap();
    write_spec(
        tmp.path(),
        "001-a",
        "001-a",
        "establishes:\n  - \"aaa.txt\"\n  - \"code.txt\"\n",
    );

    let VerifyOutcome::ContentMismatch { differences } =
        verify_spec_recompute(&cfg, tmp.path(), &attestation).unwrap()
    else {
        panic!("a new unit is a content mismatch");
    };
    assert!(
        differences.iter().any(|d| d.contains("is new")),
        "the added unit is named as added: {differences:?}"
    );
    assert!(
        !differences.iter().any(|d| d.contains("contentHash")),
        "no unit's content changed, so no hash difference is reported: {differences:?}"
    );
}

/// `lint.ok` and `compile.ok` actually go false for a spec with findings.
///
/// Both are filtered by matching the violation's `path` against the record's
/// `spec_path`, so a divergence in how either engine spells that path would
/// leave both verdicts unconditionally true and hash an empty findings list: a
/// clean-looking attestation over a spec that is not clean. The earlier tests
/// only exercised `resolution.ok`, which is computed a different way, so
/// neither filter was covered.
#[test]
fn lint_and_compile_verdicts_go_false_for_the_attested_spec() {
    // A spec with no ownership edge trips L-001, a warning, and `lint_ok`
    // mirrors this repo's own `--fail-on-warn` gate.
    let tmp = tempfile::tempdir().unwrap();
    let r = tmp.path();
    fs::write(r.join("Cargo.toml"), "[workspace]\nmembers = []\n").unwrap();
    write_spec(r, "001-a", "001-a", "");
    let attested = attest_spec(&Config::default(), r, "001-a")
        .unwrap()
        .attestation;
    assert!(
        !attested.verdicts.lint.ok,
        "an L-001 finding on this spec makes its lint verdict false"
    );
    assert!(
        attested.verdicts.compile.ok,
        "compile is unaffected: the corpus is structurally valid"
    );

    // A findings hash over a non-empty set differs from one over nothing, so a
    // silently-empty filter is detectable even when `ok` happens to agree.
    let clean = spec_fixture(OWNED);
    let clean_attested = attest_spec(&Config::default(), clean.path(), "001-a")
        .unwrap()
        .attestation;
    assert!(clean_attested.verdicts.lint.ok);
    assert_ne!(
        attested.verdicts.lint.findings_hash, clean_attested.verdicts.lint.findings_hash,
        "the findings hash distinguishes a spec with findings from one without"
    );

    // `compile.ok` goes false on an error-tier violation attributed to the spec:
    // V-001, a directory name that does not equal the frontmatter id.
    let bad = tempfile::tempdir().unwrap();
    let b = bad.path();
    fs::write(b.join("Cargo.toml"), "[workspace]\nmembers = []\n").unwrap();
    fs::write(b.join("code.txt"), "one\n").unwrap();
    write_spec(b, "001-a", "999-x", OWNED);
    let broken = attest_spec(&Config::default(), b, "999-x")
        .unwrap()
        .attestation;
    assert!(
        !broken.verdicts.compile.ok,
        "a V-001 on this spec makes its compile verdict false"
    );
    // ...and the record is still produced, because it is evidence, not a gate.
    assert_eq!(broken.spec_id, "999-x");
}

// ===== spec 083: an attestation covers the territory it claims =====
//
// Every guard 083 3.5 names. The first is the one the corpus actually
// exercises: thirteen of the fourteen specs the defect reached claim a subtree
// with a trailing-slash `file` unit rather than an explicit `directory` unit,
// so a fix keyed to `Unit::Directory` would have left them broken. It must not
// be dropped as redundant with the `directory` case.

/// A corpus whose one spec claims `d1/` (the trailing-slash `file` shorthand)
/// and `d2/` (an explicit `directory` unit), with both directories present.
fn territory_fixture() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    write_spec(
        tmp.path(),
        "001-t",
        "001-t",
        "establishes:\n  - \"d1/\"\n  - { kind: directory, path: \"d2/\" }\n",
    );
    fs::create_dir_all(tmp.path().join("d1")).unwrap();
    fs::create_dir_all(tmp.path().join("d2")).unwrap();
    tmp
}

/// The hash recorded for the unit whose debug form contains `needle`.
fn unit_hash(att: &spec_spine_types::SpecAttestation, needle: &str) -> Option<String> {
    att.units
        .iter()
        .find(|u| format!("{:?}", u.unit).contains(needle))
        .and_then(|u| u.content_hash.clone())
}

#[test]
fn spec083_a_trailing_slash_file_unit_attests_rather_than_erroring() {
    let tmp = territory_fixture();
    fs::write(tmp.path().join("d1/a.txt"), "a").unwrap();
    let out = attest_spec(&Config::default(), tmp.path(), "001-t")
        .expect("a subtree claimed as a file unit must not be opened as a file");
    assert!(
        unit_hash(&out.attestation, "d1/").is_some(),
        "the trailing-slash file unit must carry a hash"
    );
    assert!(out.attestation.verdicts.resolution.ok);
}

#[test]
fn spec083_an_explicit_directory_unit_attests() {
    let tmp = territory_fixture();
    fs::write(tmp.path().join("d2/b.txt"), "b").unwrap();
    let out = attest_spec(&Config::default(), tmp.path(), "001-t").unwrap();
    assert!(unit_hash(&out.attestation, "d2/").is_some());
}

#[test]
fn spec083_a_crate_unit_attests_over_its_package_root() {
    let tmp = tempfile::tempdir().unwrap();
    write_spec(
        tmp.path(),
        "001-c",
        "001-c",
        "establishes:\n  - { kind: crate, id: \"demo\" }\n",
    );
    let pkg = tmp.path().join("demo");
    fs::create_dir_all(pkg.join("src")).unwrap();
    fs::write(
        pkg.join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    fs::write(pkg.join("src/lib.rs"), "pub fn f() {}\n").unwrap();
    // Discovery reads the root workspace manifest, so a lone crate has to be
    // declared standalone to be found at all.
    let mut cfg = Config::default();
    cfg.layout.standalone_rust_workspaces = vec!["demo".to_string()];
    let out = attest_spec(&cfg, tmp.path(), "001-c").unwrap();
    assert!(
        unit_hash(&out.attestation, "demo").is_some(),
        "a crate unit resolves to a package directory and must be walked"
    );
}

#[test]
fn spec083_an_empty_directory_hashes_its_own_path_not_the_empty_input() {
    // 3.3: SHA-256 of the empty input is one constant shared by every empty
    // claim everywhere, so it would read as evidence while distinguishing
    // nothing. Two empty claims must differ from it and from each other.
    const SHA256_OF_NOTHING: &str =
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
    let tmp = territory_fixture();
    let out = attest_spec(&Config::default(), tmp.path(), "001-t").unwrap();
    let d1 = unit_hash(&out.attestation, "d1/").expect("an empty claim still hashes");
    let d2 = unit_hash(&out.attestation, "d2/").expect("an empty claim still hashes");
    assert_ne!(d1, SHA256_OF_NOTHING);
    assert_ne!(d2, SHA256_OF_NOTHING);
    assert_ne!(d1, d2, "two empty directories must not share one hash");
    assert!(
        out.attestation.verdicts.resolution.ok,
        "an empty directory exists, so the unit resolved"
    );
}

#[test]
fn spec083_a_pruned_to_empty_directory_hashes_like_an_empty_one() {
    let tmp = territory_fixture();
    let mut cfg = Config::default();
    cfg.index.resolver_exclusions = vec!["skipme".to_string()];
    let empty = attest_spec(&cfg, tmp.path(), "001-t").unwrap();
    let before = unit_hash(&empty.attestation, "d1/").unwrap();

    fs::create_dir_all(tmp.path().join("d1/skipme")).unwrap();
    fs::write(tmp.path().join("d1/skipme/x.txt"), "x").unwrap();
    let after = attest_spec(&cfg, tmp.path(), "001-t").unwrap();

    assert_eq!(
        before,
        unit_hash(&after.attestation, "d1/").unwrap(),
        "a fully pruned directory hashes as an empty one"
    );
}

#[test]
fn spec083_the_declared_state_root_is_pruned() {
    // 3.2: `layout.state_dir` is not expressible as a `resolver_exclusions`
    // entry (that list matches directory names, not path prefixes), so an
    // implementation could prune one and ignore the other.
    let tmp = territory_fixture();
    let mut cfg = Config::default();
    cfg.layout.state_dir = "d1/state".to_string();
    let before = unit_hash(
        &attest_spec(&cfg, tmp.path(), "001-t").unwrap().attestation,
        "d1/",
    )
    .unwrap();

    fs::create_dir_all(tmp.path().join("d1/state")).unwrap();
    fs::write(tmp.path().join("d1/state/s.txt"), "s").unwrap();
    let after = unit_hash(
        &attest_spec(&cfg, tmp.path(), "001-t").unwrap().attestation,
        "d1/",
    )
    .unwrap();

    assert_eq!(before, after, "the declared state root must be pruned");
}

#[cfg(unix)]
#[test]
fn spec083_a_symlink_is_recorded_by_target_text_and_never_followed() {
    let tmp = territory_fixture();
    let before = unit_hash(
        &attest_spec(&Config::default(), tmp.path(), "001-t")
            .unwrap()
            .attestation,
        "d1/",
    )
    .unwrap();

    // Points outside both claimed subtrees, so its content can reach the
    // payload only by dereference.
    fs::create_dir_all(tmp.path().join("outside")).unwrap();
    fs::write(tmp.path().join("outside/p.txt"), "one").unwrap();
    std::os::unix::fs::symlink("../outside/p.txt", tmp.path().join("d1/ptr")).unwrap();

    let with_link = unit_hash(
        &attest_spec(&Config::default(), tmp.path(), "001-t")
            .unwrap()
            .attestation,
        "d1/",
    )
    .unwrap();
    assert_ne!(before, with_link, "a symlink is recorded, not skipped");

    // Retargeting the file the link points at must be invisible: the link's
    // target text is what was hashed.
    fs::write(tmp.path().join("outside/p.txt"), "two").unwrap();
    let after_edit = unit_hash(
        &attest_spec(&Config::default(), tmp.path(), "001-t")
            .unwrap()
            .attestation,
        "d1/",
    )
    .unwrap();
    assert_eq!(
        with_link, after_edit,
        "the link's target content must not be dereferenced into the hash"
    );
}

#[cfg(unix)]
#[test]
fn spec083_a_symlink_to_a_directory_is_not_descended_into() {
    // The cycle case: `is_dir` follows links, so a walk that recursed on it
    // would never terminate. This test hangs rather than fails on a regression,
    // which is itself the signal.
    let tmp = territory_fixture();
    std::os::unix::fs::symlink("..", tmp.path().join("d1/loop")).unwrap();
    let out = attest_spec(&Config::default(), tmp.path(), "001-t")
        .expect("a symlink cycle must not make the walk diverge");
    assert!(unit_hash(&out.attestation, "d1/").is_some());
}

#[test]
fn spec083_a_non_utf8_file_is_hashed_rather_than_refused() {
    // D-5: a claimed subtree holds whatever it holds. Refusing put 3.1 out of
    // reach for twelve of the fourteen specs; skipping would hide claimed
    // content from the hash, which 3.4 forbids.
    let tmp = territory_fixture();
    let before = unit_hash(
        &attest_spec(&Config::default(), tmp.path(), "001-t")
            .unwrap()
            .attestation,
        "d1/",
    )
    .unwrap();

    fs::write(tmp.path().join("d1/blob.bin"), [0xffu8, 0xfe, 0x00, 0x01]).unwrap();
    let out = attest_spec(&Config::default(), tmp.path(), "001-t")
        .expect("a binary file must not refuse the attestation");
    let after = unit_hash(&out.attestation, "d1/").unwrap();
    assert_ne!(before, after, "a binary file still enters the hash");

    fs::write(tmp.path().join("d1/blob.bin"), [0x00u8, 0x01, 0x02, 0x03]).unwrap();
    let changed = unit_hash(
        &attest_spec(&Config::default(), tmp.path(), "001-t")
            .unwrap()
            .attestation,
        "d1/",
    )
    .unwrap();
    assert_ne!(after, changed, "editing binary bytes moves the hash");
}

#[test]
fn spec083_a_regular_file_unit_does_not_take_the_directory_path() {
    // 3.5: a unit whose locations are all regular files must attest exactly as
    // it did before. The discriminating check is a sibling file: if the file
    // unit had started walking its parent directory, an unrelated neighbour
    // would move its hash. It must not.
    let tmp = tempfile::tempdir().unwrap();
    write_spec(
        tmp.path(),
        "001-f",
        "001-f",
        "establishes:\n  - \"code.txt\"\n",
    );
    fs::write(tmp.path().join("code.txt"), "fn main() {}\n").unwrap();

    let a = attest_spec(&Config::default(), tmp.path(), "001-f").unwrap();
    let before = unit_hash(&a.attestation, "code.txt").expect("a file unit hashes");
    assert_eq!(
        a.json,
        attest_spec(&Config::default(), tmp.path(), "001-f")
            .unwrap()
            .json,
        "the payload stays reproducible"
    );

    fs::write(tmp.path().join("neighbour.txt"), "unrelated\n").unwrap();
    let after = unit_hash(
        &attest_spec(&Config::default(), tmp.path(), "001-f")
            .unwrap()
            .attestation,
        "code.txt",
    )
    .unwrap();
    assert_eq!(
        before, after,
        "a file unit hashes its own bytes, never its parent directory"
    );

    fs::write(tmp.path().join("code.txt"), "fn main() { () }\n").unwrap();
    assert_ne!(
        before,
        unit_hash(
            &attest_spec(&Config::default(), tmp.path(), "001-f")
                .unwrap()
                .attestation,
            "code.txt",
        )
        .unwrap(),
        "editing the file itself still moves its hash"
    );
}
