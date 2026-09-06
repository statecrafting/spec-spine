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
