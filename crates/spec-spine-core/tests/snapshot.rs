//! The authority snapshot (spec 087): the `frame/1` construction's guards
//! (length framing, piece kinds, dedup), the piece rule of §3.3.1, the
//! separation between the committed tree and the recompute, the unavailable
//! join hash of §3.2.1, determinism, and verification under spec 085's rules.

use std::fs;
use std::path::Path;

use spec_spine_core::shard::{self, BY_PACKAGE_DIR, BY_SPEC_DIR};
use spec_spine_core::snapshot::{PieceKind, PieceSet, frame_digest};
use spec_spine_core::{
    NON_CANONICAL_BYTES, VerifyOutcome, attest_spec, compile, index, index_dir, index_shard_files,
    registry_dir, registry_shard_files, snapshot, verify_snapshot_recompute,
    with_stored_bytes_snapshot,
};
use spec_spine_types::{AuthoritySnapshot, Config, NON_UTF8_DIRECT_CLAIM, SNAPSHOT_SCHEMA_VERSION};

fn write(root: &Path, rel: &str, content: &[u8]) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, content).unwrap();
}

/// One spec, `001-a`, owning whatever `units` lists (YAML list items).
fn corpus(units: &[&str]) -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    set_units(tmp.path(), units);
    tmp
}

fn set_units(root: &Path, units: &[&str]) {
    let list: String = units.iter().map(|u| format!("  - \"{u}\"\n")).collect();
    write(
        root,
        "specs/001-a/spec.md",
        format!(
            "---\nid: \"001-a\"\ntitle: \"t\"\nstatus: draft\ncreated: \"2026-09-11\"\n\
             summary: \"s\"\nestablishes:\n{list}---\n\n# t\n"
        )
        .as_bytes(),
    );
}

fn snap(root: &Path) -> AuthoritySnapshot {
    snapshot(&Config::default(), root).unwrap().snapshot
}

fn territory(root: &Path) -> Option<String> {
    snap(root).specs[0].territory_digest.clone()
}

/// Write both committed trees exactly as `compile` and `index` do.
fn emit(root: &Path) {
    let cfg = Config::default();
    let registry = compile(&cfg, root).unwrap();
    shard::sync_dir(
        &registry_dir(&cfg, root).join(BY_SPEC_DIR),
        &registry_shard_files(&registry.shards).unwrap(),
    )
    .unwrap();
    let outcome = index(&cfg, root).unwrap();
    let dir = index_dir(&cfg, root);
    let (by_spec, by_package) = index_shard_files(&outcome.shards).unwrap();
    shard::sync_dir(&dir.join(BY_SPEC_DIR), &by_spec).unwrap();
    shard::sync_dir(&dir.join(BY_PACKAGE_DIR), &by_package).unwrap();
}

#[test]
fn the_schema_axis_starts_at_0_1_0() {
    assert_eq!(SNAPSHOT_SCHEMA_VERSION, "0.1.0");
}

// ── frame/1 (§3.3) ────────────────────────────────────────────────────────

#[test]
fn length_framing_separates_a_split_piece_set_from_a_joined_one() {
    // The unframed fold hashes `a\0x` then `b\0y` with nothing between `x` and
    // `b`, which is the same byte stream as one piece `a` holding `xb\0y`.
    let mut split = PieceSet::new();
    split.insert("a".into(), PieceKind::Text, b"x".to_vec());
    split.insert("b".into(), PieceKind::Text, b"y".to_vec());
    let mut joined = PieceSet::new();
    joined.insert("a".into(), PieceKind::Text, b"xb\0y".to_vec());
    assert_ne!(frame_digest(&split), frame_digest(&joined));
}

#[test]
fn every_kind_byte_is_part_of_the_digest() {
    let one = |kind: PieceKind, content: &[u8]| {
        let mut set = PieceSet::new();
        set.insert("p".into(), kind, content.to_vec());
        frame_digest(&set)
    };
    // Same path, same content, different kind: all four digests differ.
    let digests = [
        one(PieceKind::Text, b""),
        one(PieceKind::Bytes, b""),
        one(PieceKind::Link, b""),
        one(PieceKind::EmptyDir, b""),
    ];
    for (i, a) in digests.iter().enumerate() {
        for b in &digests[i + 1..] {
            assert_ne!(a, b);
        }
    }
}

#[test]
fn non_utf8_content_is_a_bytes_piece_and_utf8_is_normalized_text() {
    let raw: &[u8] = &[0xff, 0xfe, 0x00, 0x01];
    let mut bytes = PieceSet::new();
    bytes.insert_file_content("f".into(), raw.to_vec());
    let mut forced = PieceSet::new();
    forced.insert("f".into(), PieceKind::Bytes, raw.to_vec());
    assert_eq!(frame_digest(&bytes), frame_digest(&forced));

    // A BOM and CRLF normalize away for text, which is the stated contract.
    let mut lf = PieceSet::new();
    lf.insert_file_content("t".into(), b"a\nb\n".to_vec());
    let mut crlf = PieceSet::new();
    crlf.insert_file_content("t".into(), "\u{feff}a\r\nb\r\n".as_bytes().to_vec());
    assert_eq!(frame_digest(&lf), frame_digest(&crlf));
}

#[test]
fn a_path_appears_once_and_the_empty_set_has_no_digest() {
    let mut once = PieceSet::new();
    once.insert("a".into(), PieceKind::Text, b"x".to_vec());
    let mut twice = once.clone();
    twice.insert("a".into(), PieceKind::Text, b"x".to_vec());
    twice.insert("a".into(), PieceKind::Bytes, b"other".to_vec());
    assert_eq!(twice.len(), 1, "the first piece for a path is kept");
    assert_eq!(frame_digest(&once), frame_digest(&twice));
    assert_eq!(PieceSet::new().digest(), None);
}

// ── the piece rule (§3.3.1) ──────────────────────────────────────────────

#[test]
fn an_overlapping_claim_is_not_a_doubled_one() {
    let fx = corpus(&["g/"]);
    write(fx.path(), "g/a", b"x");
    write(fx.path(), "g/b", b"y");
    let alone = territory(fx.path()).expect("a digest");
    set_units(fx.path(), &["g/", "g/a"]);
    assert_eq!(territory(fx.path()).unwrap(), alone);
}

#[test]
fn a_spec_whose_units_resolve_to_nothing_has_no_territory_digest() {
    let fx = corpus(&["nowhere/at/all.txt"]);
    let s = snap(fx.path());
    assert_eq!(s.specs[0].territory_digest, None);
    assert!(!s.verdicts.resolution.ok);
    assert_eq!(s.verdicts.resolution.unresolved, 1);
}

#[test]
fn an_empty_directory_and_an_empty_file_at_one_path_differ() {
    let fx = corpus(&["g/"]);
    write(fx.path(), "g/a", b"x");
    fs::create_dir_all(fx.path().join("g/e")).unwrap();
    let dir = territory(fx.path()).unwrap();
    fs::remove_dir(fx.path().join("g/e")).unwrap();
    write(fx.path(), "g/e", b"");
    let file = territory(fx.path()).unwrap();
    assert_ne!(dir, file);
    // And an empty directory is not the same as no directory at all.
    fs::remove_file(fx.path().join("g/e")).unwrap();
    assert_ne!(territory(fx.path()).unwrap(), dir);
}

#[test]
fn a_directly_claimed_empty_directory_is_distinguishable_by_place() {
    let fx = corpus(&["one/"]);
    fs::create_dir_all(fx.path().join("one")).unwrap();
    fs::create_dir_all(fx.path().join("two")).unwrap();
    let one = territory(fx.path()).expect("an empty claim is not no claim");
    set_units(fx.path(), &["two/"]);
    assert_ne!(territory(fx.path()).unwrap(), one);
}

#[test]
fn the_digest_binds_the_whole_file_and_moves_with_its_content() {
    let fx = corpus(&["g/a"]);
    write(fx.path(), "g/a", b"x");
    let before = territory(fx.path()).unwrap();
    write(fx.path(), "g/a", b"changed");
    assert_ne!(territory(fx.path()).unwrap(), before);
}

#[cfg(unix)]
#[test]
fn a_symlink_is_its_target_text_and_is_never_followed() {
    let fx = corpus(&["g/"]);
    write(fx.path(), "g/a", b"x");
    write(fx.path(), "outside.txt", b"one");
    std::os::unix::fs::symlink("../outside.txt", fx.path().join("g/link")).unwrap();
    let before = territory(fx.path()).unwrap();
    // The target's content moves; the link text does not, so neither does the
    // digest.
    write(fx.path(), "outside.txt", b"two");
    assert_eq!(territory(fx.path()).unwrap(), before);
}

// ── the join hash with no answer (§3.2.1) ────────────────────────────────

#[test]
fn a_non_utf8_direct_claim_keeps_its_territory_and_states_why_the_join_hash_is_absent() {
    let fx = corpus(&["bin.dat"]);
    write(fx.path(), "bin.dat", &[0xff, 0xfe, 0x00, 0x01]);
    // The per-spec verb still refuses, as §3.7 requires.
    assert!(attest_spec(&Config::default(), fx.path(), "001-a").is_err());
    let s = snap(fx.path());
    let entry = &s.specs[0];
    assert!(entry.territory_digest.is_some());
    assert_eq!(entry.spec_attestation_hash, None);
    assert_eq!(
        entry.spec_attestation_unavailable.as_deref(),
        Some(NON_UTF8_DIRECT_CLAIM)
    );
}

#[test]
fn the_join_hash_is_the_per_spec_attestation_hash() {
    let fx = corpus(&["g/"]);
    write(fx.path(), "g/a", b"x");
    let per_spec = attest_spec(&Config::default(), fx.path(), "001-a").unwrap();
    let s = snap(fx.path());
    assert_eq!(
        s.specs[0].spec_attestation_hash.as_deref(),
        Some(per_spec.attestation_hash.as_str())
    );
    assert_eq!(s.specs[0].spec_attestation_unavailable, None);
}

// ── the payload (§3.1, §3.2, §3.4) ───────────────────────────────────────

#[test]
fn a_snapshot_is_byte_identical_on_an_unchanged_tree() {
    let fx = corpus(&["g/"]);
    write(fx.path(), "g/a", b"x");
    emit(fx.path());
    let a = snapshot(&Config::default(), fx.path()).unwrap();
    let b = snapshot(&Config::default(), fx.path()).unwrap();
    assert_eq!(a.json, b.json);
    assert_eq!(a.attestation_hash, b.attestation_hash);
    assert!(a.json.ends_with("}\n"));
    assert_eq!(a.snapshot.digest, "frame/1");
}

#[test]
fn absent_trees_and_an_absent_config_are_absent_not_the_digest_of_nothing() {
    let fx = corpus(&["g/a"]);
    write(fx.path(), "g/a", b"x");
    let s = snap(fx.path());
    assert!(!s.config.present);
    assert_eq!(s.config.hash, None);
    for tree in [&s.committed.registry, &s.committed.index] {
        assert_eq!(tree.files, 0);
        assert_eq!(tree.hash, None);
        assert!(!tree.matches_recompute);
    }
    assert!(s.governance_inputs.paths.is_empty());
    assert_eq!(s.governance_inputs.hash, None);
}

#[test]
fn matches_recompute_reads_the_committed_tree_and_the_recompute_does_not_move() {
    let fx = corpus(&["g/a"]);
    write(fx.path(), "g/a", b"x");
    emit(fx.path());
    let before = snap(fx.path());
    assert!(before.committed.registry.matches_recompute);
    assert!(before.committed.index.matches_recompute);

    let shard = registry_dir(&Config::default(), fx.path()).join("by-spec/001-a.json");
    let mut bytes = fs::read(&shard).unwrap();
    bytes.push(b' ');
    fs::write(&shard, bytes).unwrap();
    let after = snap(fx.path());
    assert!(!after.committed.registry.matches_recompute);
    assert_ne!(
        after.committed.registry.hash,
        before.committed.registry.hash
    );
    assert_eq!(after.corpus.registry_hash, before.corpus.registry_hash);
    assert!(
        after.committed.index.matches_recompute,
        "the other tree is untouched"
    );
}

#[test]
fn governance_inputs_are_named_and_hashed_as_their_own_content() {
    let fx = corpus(&["g/a"]);
    write(fx.path(), "g/a", b"x");
    write(
        fx.path(),
        "spec-spine.toml",
        b"[index]\nextra_hashed_inputs = [\"gov/*\"]\n",
    );
    write(fx.path(), "gov/rules.md", b"one\n");
    let cfg = spec_spine_types::load_config(
        &fs::read_to_string(fx.path().join("spec-spine.toml")).unwrap(),
    )
    .unwrap();
    let s = snapshot(&cfg, fx.path()).unwrap().snapshot;
    assert!(s.config.present && s.config.hash.is_some());
    assert_eq!(
        s.governance_inputs.paths,
        ["gov/rules.md", "spec-spine.toml"]
    );
    let before = s.governance_inputs.hash.clone();
    write(fx.path(), "gov/rules.md", b"two\n");
    let after = snapshot(&cfg, fx.path()).unwrap().snapshot;
    assert_ne!(after.governance_inputs.hash, before);
}

// ── verification (§3.5, spec 085) ────────────────────────────────────────

#[test]
fn recompute_matches_then_names_what_moved() {
    let fx = corpus(&["g/a"]);
    write(fx.path(), "g/a", b"x");
    let cfg = Config::default();
    let outcome = snapshot(&cfg, fx.path()).unwrap();
    assert_eq!(
        verify_snapshot_recompute(&cfg, fx.path(), &outcome.snapshot).unwrap(),
        VerifyOutcome::Match
    );

    write(fx.path(), "g/a", b"changed");
    match verify_snapshot_recompute(&cfg, fx.path(), &outcome.snapshot).unwrap() {
        VerifyOutcome::ContentMismatch { differences } => {
            assert!(
                differences.contains(&"specs[001-a]".to_string()),
                "{differences:?}"
            );
        }
        other => panic!("expected a content mismatch, got {other:?}"),
    }

    let mut other_version = outcome.snapshot.clone();
    other_version.tool.version = "0.0.1".into();
    assert!(matches!(
        verify_snapshot_recompute(&cfg, fx.path(), &other_version).unwrap(),
        VerifyOutcome::VersionMismatch { .. }
    ));
}

#[test]
fn stored_bytes_that_are_not_canonical_are_a_mismatch() {
    let fx = corpus(&["g/a"]);
    write(fx.path(), "g/a", b"x");
    let cfg = Config::default();
    let outcome = snapshot(&cfg, fx.path()).unwrap();
    let reformatted = serde_json::to_string(&outcome.snapshot).unwrap();
    let verdict = with_stored_bytes_snapshot(
        verify_snapshot_recompute(&cfg, fx.path(), &outcome.snapshot).unwrap(),
        &outcome.snapshot,
        reformatted.as_bytes(),
    )
    .unwrap();
    assert_eq!(
        verdict,
        VerifyOutcome::ContentMismatch {
            differences: vec![NON_CANONICAL_BYTES.to_string()]
        }
    );
}

#[test]
fn the_payload_refuses_an_unknown_member() {
    let fx = corpus(&["g/a"]);
    write(fx.path(), "g/a", b"x");
    let json = snapshot(&Config::default(), fx.path()).unwrap().json;
    let mut value: serde_json::Value = serde_json::from_str(&json).unwrap();
    value["obligations"] = serde_json::json!([]);
    assert!(serde_json::from_value::<AuthoritySnapshot>(value).is_err());
}
