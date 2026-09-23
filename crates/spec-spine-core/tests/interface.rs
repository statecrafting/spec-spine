// Spec: specs/110-an-interface-reference-is-digest-pinned/spec.md
//! Spec 110: an interface reference is digest-pinned. Every compile rule
//! (§3.1, §3.2) over a disposable corpus through the real `compile`, and every
//! verifier outcome (§3.3, §3.4) over an exporter corpus that is itself
//! compiled, so a pin is always the digest the exporter's own registry
//! records, never one this file computed the same way the verifier does.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use spec_spine_core::{
    ExportedSpec, Exports, compile, interface_verify, interface_verify_json, lint, load_export,
    verify_interface_references,
};
use spec_spine_types::{Config, Error, Outcome, Registry, SectionOutcome};

const EXPORTER_ID: &str = "002-environment-lifecycle";

const EXPORTER_BODY: &str = "# 002\n\n## 3. Behavior\n\n### 3.13 The managed instruction file\n\nThe bridge line is inserted once.\n\n### 3.14 Unrelated\n\nOther text.\n";

fn write_spec(root: &Path, dir: &str, id: &str, extra: &str, body: &str) {
    let d = root.join(dir).join(id);
    fs::create_dir_all(&d).unwrap();
    fs::write(
        d.join("spec.md"),
        format!(
            "---\nid: \"{id}\"\ntitle: \"T\"\nstatus: draft\ncreated: \"2026-09-22\"\n\
             summary: \"s\"\n{extra}---\n{body}"
        ),
    )
    .unwrap();
}

/// What the exporting corpus's own registry records for its spec: the shard
/// hash and the section digests, both `sha256:`-prefixed as §3.1 says a
/// reader copies them.
struct Pin {
    digest: String,
    sections: BTreeMap<String, String>,
}

fn pin_of(exporter_root: &Path) -> Pin {
    let out = compile(&Config::default(), exporter_root).unwrap();
    assert!(out.validation_passed, "{:?}", out.registry.validation);
    let shard = out
        .shards
        .spec_shards
        .iter()
        .find(|s| s.record.id == EXPORTER_ID)
        .unwrap();
    Pin {
        digest: format!("sha256:{}", shard.shard_hash),
        sections: shard
            .record
            .section_digests
            .iter()
            .map(|(k, v)| (k.clone(), format!("sha256:{v}")))
            .collect(),
    }
}

fn exporter(body: &str) -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    write_spec(tmp.path(), "specs", EXPORTER_ID, "", body);
    tmp
}

/// One reference in YAML, pinning `sections` (anchor, digest) when given.
fn reference(corpus: &str, spec: &str, digest: &str, sections: &[(&str, &str)]) -> String {
    let mut s = format!(
        "  - corpus: \"{corpus}\"\n    spec: \"{spec}\"\n    digest: \"{digest}\"\n    obtained: \"2026-09-22\"\n"
    );
    if !sections.is_empty() {
        s.push_str("    sections:\n");
        for (a, d) in sections {
            s.push_str(&format!(
                "      - anchor: \"{a}\"\n        digest: \"{d}\"\n"
            ));
        }
    }
    s
}

fn importer(references: &[String]) -> (tempfile::TempDir, spec_spine_core::CompileOutcome) {
    let tmp = tempfile::tempdir().unwrap();
    let extra = if references.is_empty() {
        String::new()
    } else {
        format!("interface_references:\n{}", references.concat())
    };
    write_spec(
        tmp.path(),
        "specs",
        "001-bridge",
        &extra,
        "# 001\n\n## 3. Behavior\n\nText.\n",
    );
    let out = compile(&Config::default(), tmp.path()).unwrap();
    (tmp, out)
}

fn commit_ledger(root: &Path, out: &spec_spine_core::CompileOutcome) {
    let cfg = Config::default();
    let dir = spec_spine_core::registry_dir(&cfg, root).join("by-spec");
    fs::create_dir_all(&dir).unwrap();
    for (name, bytes) in spec_spine_core::registry_shard_files(&out.shards).unwrap() {
        fs::write(dir.join(name), bytes).unwrap();
    }
}

fn codes(out: &spec_spine_core::CompileOutcome) -> Vec<(String, String)> {
    out.registry
        .validation
        .violations
        .iter()
        .map(|v| (v.code.clone(), v.message.clone()))
        .collect()
}

fn has(out: &spec_spine_core::CompileOutcome, code: &str, needle: &str) -> bool {
    codes(out)
        .iter()
        .any(|(c, m)| c == code && m.contains(needle))
}

const HEX: &str = "0000000000000000000000000000000000000000000000000000000000000000";

fn zero() -> String {
    format!("sha256:{HEX}")
}

fn exports_from(root: &Path) -> Exports {
    let mut e = Exports::new();
    e.insert(
        "statecraft-cli".to_string(),
        load_export(root, &[EXPORTER_ID.to_string()]).unwrap(),
    );
    e
}

fn verify(reg: &Registry, exports: &Exports) -> spec_spine_types::InterfaceReport {
    verify_interface_references(reg, exports, None).unwrap()
}

// ---- 3.1, 3.2: the reference compiles, verbatim, and every rule fires ------

#[test]
fn a_valid_reference_compiles_and_is_carried_verbatim() {
    let (_t, out) = importer(&[reference(
        "statecraft-cli",
        EXPORTER_ID,
        &zero(),
        &[("3-13-the-managed-instruction-file", &zero())],
    )]);
    assert!(out.validation_passed, "{:?}", codes(&out));
    let rec = &out.registry.specs[0];
    assert_eq!(rec.interface_references.len(), 1);
    let r = &rec.interface_references[0];
    assert_eq!(r.corpus, "statecraft-cli");
    assert_eq!(r.spec, EXPORTER_ID);
    assert_eq!(r.digest, zero());
    assert_eq!(r.obtained, "2026-09-22");
    assert_eq!(r.sections[0].anchor, "3-13-the-managed-instruction-file");
    let json = serde_json::to_value(rec).unwrap();
    assert!(json["interfaceReferences"].is_array(), "{json}");
}

#[test]
fn a_spec_without_the_key_carries_no_member_and_its_shard_hash_is_the_file_hash() {
    let (t, out) = importer(&[]);
    assert!(out.validation_passed);
    let json = serde_json::to_value(&out.registry.specs[0]).unwrap();
    assert!(json.get("interfaceReferences").is_none(), "{json}");
    let raw = fs::read_to_string(t.path().join("specs/001-bridge/spec.md")).unwrap();
    let recomputed = {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(b"specs/001-bridge/spec.md\0");
        h.update(raw.as_bytes());
        h.finalize()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    };
    assert_eq!(out.shards.spec_shards[0].shard_hash, recomputed);
}

#[test]
fn every_malformed_member_is_a_named_compile_error() {
    let cases: Vec<(String, &str, &str)> = vec![
        (
            reference("Statecraft", EXPORTER_ID, &zero(), &[]),
            "V-032",
            "corpus 'Statecraft'",
        ),
        (
            reference("https://x", EXPORTER_ID, &zero(), &[]),
            "V-032",
            "corpus 'https://x'",
        ),
        (reference("sc", "002", &zero(), &[]), "V-033", "a short id"),
        (
            reference("sc", "not an id", &zero(), &[]),
            "V-033",
            "full id",
        ),
        (
            reference("sc", EXPORTER_ID, &format!("sha1:{HEX}"), &[]),
            "V-034",
            "only sha256 is supported",
        ),
        (
            reference("sc", EXPORTER_ID, HEX, &[]),
            "V-034",
            "no algorithm prefix",
        ),
        (
            reference("sc", EXPORTER_ID, &format!("sha256:{}", &HEX[1..]), &[]),
            "V-034",
            "does not match",
        ),
        (
            reference("sc", EXPORTER_ID, &zero(), &[("a", "sha256:zz")]),
            "V-035",
            "section 'a'",
        ),
        (
            reference(
                "sc",
                EXPORTER_ID,
                &zero(),
                &[("a", &zero()), ("a", &zero())],
            ),
            "V-037",
            "pins anchor 'a' twice",
        ),
        (
            reference("sc", EXPORTER_ID, &zero(), &[]).replace("2026-09-22", "22/09/2026"),
            "V-036",
            "obtained '22/09/2026'",
        ),
    ];
    for (r, code, needle) in cases {
        let (_t, out) = importer(std::slice::from_ref(&r));
        assert!(!out.validation_passed, "{code} did not refuse:\n{r}");
        assert!(
            has(&out, code, needle),
            "{code} / {needle}: {:?}\n{r}",
            codes(&out)
        );
        // Every refusal names the declaring spec's file.
        assert!(
            out.registry
                .validation
                .violations
                .iter()
                .all(|v| v.path.as_deref() == Some("specs/001-bridge/spec.md")),
            "{:?}",
            out.registry.validation.violations
        );
    }
}

#[test]
fn a_second_reference_to_the_same_pair_is_v038_and_different_pairs_are_fine() {
    let (_t, out) = importer(&[
        reference("sc", EXPORTER_ID, &zero(), &[]),
        reference("sc", EXPORTER_ID, &zero(), &[]),
    ]);
    assert!(has(&out, "V-038", "declared twice"), "{:?}", codes(&out));

    let (_t, out) = importer(&[
        reference("sc", EXPORTER_ID, &zero(), &[]),
        reference("other", EXPORTER_ID, &zero(), &[]),
        reference("sc", "003-another", &zero(), &[]),
    ]);
    assert!(out.validation_passed, "{:?}", codes(&out));
}

#[test]
fn a_missing_digest_or_an_unknown_member_is_malformed_frontmatter() {
    // §3.2: there is no placeholder form. No digest is a parse refusal, not a
    // reference the verifier fills in later.
    let no_digest = "  - corpus: \"sc\"\n    spec: \"002-x\"\n    obtained: \"2026-09-22\"\n";
    let unknown = format!(
        "{}    trust: \"first-use\"\n",
        reference("sc", "002-x", &zero(), &[])
    );
    for r in [no_digest.to_string(), unknown] {
        let (_t, out) = importer(std::slice::from_ref(&r));
        assert!(!out.validation_passed, "{r}");
        assert!(
            codes(&out).iter().any(|(c, _)| c == "V-002"),
            "{:?}",
            codes(&out)
        );
    }
}

// ---- 3.3: every outcome, against pins the exporter recorded -----------------

#[test]
fn a_pin_taken_from_the_exporters_registry_is_current() {
    let exp = exporter(EXPORTER_BODY);
    let pin = pin_of(exp.path());
    let (_t, out) = importer(&[reference("statecraft-cli", EXPORTER_ID, &pin.digest, &[])]);
    assert!(out.validation_passed, "{:?}", codes(&out));
    let report = verify(&out.registry, &exports_from(exp.path()));
    let r = &report.references[0];
    assert_eq!(r.outcome, Outcome::Current);
    assert_eq!(r.observed_digest.as_deref(), Some(pin.digest.as_str()));
    assert_eq!(report.summary.current, 1);
}

#[test]
fn crlf_and_a_bom_in_the_export_do_not_move_the_digest() {
    // The exporter's compile normalizes (spec 077); so must the verifier, or a
    // Windows checkout of an unchanged corpus would read as stale.
    let exp = exporter(EXPORTER_BODY);
    let pin = pin_of(exp.path());
    let path = exp.path().join("specs").join(EXPORTER_ID).join("spec.md");
    let text = fs::read_to_string(&path).unwrap();
    fs::write(&path, format!("\u{feff}{}", text.replace('\n', "\r\n"))).unwrap();
    let anchor = "3-13-the-managed-instruction-file";
    let (_t, out) = importer(&[reference(
        "statecraft-cli",
        EXPORTER_ID,
        &pin.digest,
        &[(anchor, &pin.sections[anchor])],
    )]);
    let report = verify(&out.registry, &exports_from(exp.path()));
    assert_eq!(report.references[0].outcome, Outcome::Current);
    assert_eq!(
        report.references[0].sections[0].outcome,
        SectionOutcome::Current
    );
}

#[test]
fn an_edit_outside_the_pinned_section_is_sections_current_and_stale_without_sections() {
    let exp = exporter(EXPORTER_BODY);
    let pin = pin_of(exp.path());
    let anchor = "3-13-the-managed-instruction-file";
    // The exporter edits 3.14 only.
    write_spec(
        exp.path(),
        "specs",
        EXPORTER_ID,
        "",
        &EXPORTER_BODY.replace("Other text.", "Other text, rewritten."),
    );
    let (_t1, sectioned) = importer(&[reference(
        "statecraft-cli",
        EXPORTER_ID,
        &pin.digest,
        &[(anchor, &pin.sections[anchor])],
    )]);
    let (_t2, whole) = importer(&[reference("statecraft-cli", EXPORTER_ID, &pin.digest, &[])]);
    let exports = exports_from(exp.path());

    let r = &verify(&sectioned.registry, &exports).references[0];
    assert_eq!(r.outcome, Outcome::SectionsCurrent);
    assert_ne!(r.observed_digest.as_deref(), Some(pin.digest.as_str()));
    assert_eq!(r.sections[0].outcome, SectionOutcome::Current);

    let r = &verify(&whole.registry, &exports).references[0];
    assert_eq!(r.outcome, Outcome::Stale);
}

#[test]
fn an_edit_inside_the_pinned_section_or_a_removed_anchor_is_stale() {
    let anchor = "3-13-the-managed-instruction-file";
    for (edit, want) in [
        (
            EXPORTER_BODY.replace("inserted once", "inserted twice"),
            SectionOutcome::Stale,
        ),
        (
            EXPORTER_BODY.replace("### 3.13 The managed instruction file", "### 3.13 Renamed"),
            SectionOutcome::Missing,
        ),
    ] {
        let exp = exporter(EXPORTER_BODY);
        let pin = pin_of(exp.path());
        write_spec(exp.path(), "specs", EXPORTER_ID, "", &edit);
        let (_t, out) = importer(&[reference(
            "statecraft-cli",
            EXPORTER_ID,
            &pin.digest,
            &[(anchor, &pin.sections[anchor])],
        )]);
        let report = verify(&out.registry, &exports_from(exp.path()));
        let r = &report.references[0];
        assert_eq!(r.outcome, Outcome::Stale, "{edit}");
        assert_eq!(r.sections[0].outcome, want, "{edit}");
        match want {
            SectionOutcome::Stale => assert!(r.sections[0].observed_digest.is_some()),
            _ => assert!(r.sections[0].observed_digest.is_none()),
        }
        assert_eq!(report.summary.stale, 1);
    }
}

#[test]
fn a_mismatched_pin_on_an_unchanged_export_is_stale() {
    // The pin is wrong, not the export: nothing trusts what it first saw.
    let exp = exporter(EXPORTER_BODY);
    let (_t, out) = importer(&[reference("statecraft-cli", EXPORTER_ID, &zero(), &[])]);
    let r = &verify(&out.registry, &exports_from(exp.path())).references[0];
    assert_eq!(r.outcome, Outcome::Stale);
    assert_eq!(
        r.digest,
        zero(),
        "the declared pin is reported, not replaced"
    );
}

#[test]
fn a_spec_absent_from_the_export_is_missing_and_an_absent_export_is_unverified() {
    let exp = tempfile::tempdir().unwrap();
    fs::create_dir_all(exp.path().join("specs")).unwrap();
    let (_t, out) = importer(&[
        reference("statecraft-cli", EXPORTER_ID, &zero(), &[]),
        reference("nobody-supplied", EXPORTER_ID, &zero(), &[]),
    ]);
    let report = verify(&out.registry, &exports_from(exp.path()));
    let by_corpus: BTreeMap<&str, Outcome> = report
        .references
        .iter()
        .map(|r| (r.corpus.as_str(), r.outcome))
        .collect();
    assert_eq!(by_corpus["statecraft-cli"], Outcome::Missing);
    assert_eq!(by_corpus["nobody-supplied"], Outcome::Unverified);
    assert!(
        report
            .references
            .iter()
            .all(|r| r.observed_digest.is_none())
    );
    assert_eq!((report.summary.missing, report.summary.unverified), (1, 1));
}

#[test]
fn results_are_sorted_and_the_same_inputs_give_the_same_bytes() {
    let exp = exporter(EXPORTER_BODY);
    let (_t, out) = importer(&[
        reference("zeta", EXPORTER_ID, &zero(), &[]),
        reference("alpha", "009-late", &zero(), &[]),
        reference("alpha", "003-early", &zero(), &[]),
    ]);
    let exports = exports_from(exp.path());
    let a = verify(&out.registry, &exports);
    let keys: Vec<(&str, &str)> = a
        .references
        .iter()
        .map(|r| (r.corpus.as_str(), r.spec.as_str()))
        .collect();
    assert_eq!(
        keys,
        vec![
            ("alpha", "003-early"),
            ("alpha", "009-late"),
            ("zeta", EXPORTER_ID)
        ]
    );
    let b = verify(&out.registry, &exports);
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        serde_json::to_string(&b).unwrap()
    );
}

#[test]
fn only_spec_resolves_the_short_form_and_an_unknown_spec_is_not_found() {
    let (_t, out) = importer(&[reference("sc", EXPORTER_ID, &zero(), &[])]);
    let e = Exports::new();
    let full = verify_interface_references(&out.registry, &e, Some("001-bridge")).unwrap();
    let short = verify_interface_references(&out.registry, &e, Some("001")).unwrap();
    assert_eq!(full, short);
    assert_eq!(full.references.len(), 1);
    let err = verify_interface_references(&out.registry, &e, Some("999")).unwrap_err();
    assert!(matches!(err, Error::NotFound(_)), "{err}");
    assert_eq!(err.exit_code(), 1);
}

#[test]
fn a_corpus_with_no_references_answers_an_empty_report() {
    let (_t, out) = importer(&[]);
    let r = verify(&out.registry, &Exports::new());
    assert!(r.references.is_empty());
    assert_eq!(r.summary, spec_spine_types::Summary::default());
}

// ---- 3.3: the IO wrapper ---------------------------------------------------

#[test]
fn the_ledger_is_read_committed_and_a_stale_one_refuses_before_any_export_is_read() {
    let exp = exporter(EXPORTER_BODY);
    let pin = pin_of(exp.path());
    let (t, out) = importer(&[reference("statecraft-cli", EXPORTER_ID, &pin.digest, &[])]);
    commit_ledger(t.path(), &out);
    let mut dirs: BTreeMap<String, PathBuf> = BTreeMap::new();
    dirs.insert("statecraft-cli".into(), exp.path().to_path_buf());
    let report = interface_verify(&Config::default(), t.path(), &dirs, None).unwrap();
    assert_eq!(report.references[0].outcome, Outcome::Current);

    // spec.md now says something the committed ledger does not. The export
    // directory is pointed at a path that does not exist, so a verifier that
    // read exports first would answer exit 3, not exit 2.
    let spec = t.path().join("specs/001-bridge/spec.md");
    let text = fs::read_to_string(&spec).unwrap();
    fs::write(&spec, text.replace("Text.", "Text, edited.")).unwrap();
    dirs.insert("statecraft-cli".into(), t.path().join("no-such-dir"));
    let err = interface_verify(&Config::default(), t.path(), &dirs, None).unwrap_err();
    assert!(matches!(err, Error::Stale { .. }), "{err}");
    assert_eq!(err.exit_code(), 2);
}

#[test]
fn the_exporter_config_names_its_specs_directory_and_nothing_else_is_read() {
    let exp = tempfile::tempdir().unwrap();
    fs::write(
        exp.path().join("spec-spine.toml"),
        "[layout]\nspecs_dir = \"docs/specs\"\n",
    )
    .unwrap();
    write_spec(exp.path(), "docs/specs", EXPORTER_ID, "", EXPORTER_BODY);
    let cfg: Config =
        spec_spine_types::load_config("[layout]\nspecs_dir = \"docs/specs\"\n").unwrap();
    let out = compile(&cfg, exp.path()).unwrap();
    let digest = format!("sha256:{}", out.shards.spec_shards[0].shard_hash);

    let loaded = load_export(exp.path(), &[EXPORTER_ID.to_string()]).unwrap();
    assert_eq!(
        loaded[EXPORTER_ID].path,
        format!("docs/specs/{EXPORTER_ID}/spec.md"),
        "the path the exporter's own compile keyed its hash on"
    );
    let (_t, imp) = importer(&[reference("statecraft-cli", EXPORTER_ID, &digest, &[])]);
    let r = &verify(&imp.registry, &exports_from(exp.path())).references[0];
    assert_eq!(r.outcome, Outcome::Current);
}

#[test]
fn an_unparseable_exporter_config_or_an_unreadable_root_is_exit_3() {
    let exp = tempfile::tempdir().unwrap();
    fs::write(exp.path().join("spec-spine.toml"), "[layout\n").unwrap();
    let err = load_export(exp.path(), &[EXPORTER_ID.to_string()]).unwrap_err();
    assert_eq!(err.exit_code(), 3, "{err}");

    let err = load_export(&exp.path().join("absent"), &[]).unwrap_err();
    assert_eq!(err.exit_code(), 3, "{err}");
}

#[cfg(unix)]
#[test]
fn a_spec_linked_out_of_the_export_root_is_refused() {
    let outside = exporter(EXPORTER_BODY);
    let exp = tempfile::tempdir().unwrap();
    fs::create_dir_all(exp.path().join("specs")).unwrap();
    std::os::unix::fs::symlink(
        outside.path().join("specs").join(EXPORTER_ID),
        exp.path().join("specs").join(EXPORTER_ID),
    )
    .unwrap();
    let err = load_export(exp.path(), &[EXPORTER_ID.to_string()]).unwrap_err();
    assert_eq!(err.exit_code(), 3, "{err}");
    assert!(err.to_string().contains("escapes"), "{err}");
}

// ---- 3.4: the facade -------------------------------------------------------

#[test]
fn the_facade_answers_the_same_document_without_a_filesystem() {
    let exp = exporter(EXPORTER_BODY);
    let pin = pin_of(exp.path());
    let (_t, out) = importer(&[reference("statecraft-cli", EXPORTER_ID, &pin.digest, &[])]);
    let registry = serde_json::to_string(&out.registry).unwrap();
    let text =
        fs::read_to_string(exp.path().join("specs").join(EXPORTER_ID).join("spec.md")).unwrap();
    let request = serde_json::json!({
        "registry": registry,
        "exports": { "statecraft-cli": { EXPORTER_ID: {
            "path": format!("specs/{EXPORTER_ID}/spec.md"),
            "text": text,
        }}},
    });
    let doc: serde_json::Value =
        serde_json::from_str(&interface_verify_json(&request.to_string()).unwrap()).unwrap();
    assert_eq!(doc["schemaVersion"], spec_spine_types::READ_SCHEMA_VERSION);
    assert_eq!(doc["references"][0]["outcome"], "current");
    assert_eq!(doc["references"][0]["declaredBy"], "001-bridge");
    assert_eq!(doc["summary"]["current"], 1);

    // The same report the library answers, byte for byte once stamped.
    let direct = verify(&out.registry, &exports_from(exp.path()));
    let stamped =
        spec_spine_core::read_document(&direct, spec_spine_core::Versioning::Stamp).unwrap();
    assert_eq!(
        interface_verify_json(&request.to_string()).unwrap(),
        stamped
    );

    // A path the exporter never used moves the hash: the path is part of the
    // construction, so a binding must pass the exporter's own path.
    let mut moved = request.clone();
    moved["exports"]["statecraft-cli"][EXPORTER_ID]["path"] = serde_json::json!("elsewhere.md");
    let doc: serde_json::Value =
        serde_json::from_str(&interface_verify_json(&moved.to_string()).unwrap()).unwrap();
    assert_eq!(doc["references"][0]["outcome"], "stale");
}

#[test]
fn the_facade_refuses_an_unknown_member_and_an_unknown_spec() {
    let (_t, out) = importer(&[]);
    let registry = serde_json::to_string(&out.registry).unwrap();
    let err = interface_verify_json(
        &serde_json::json!({ "registry": registry, "fetch": true }).to_string(),
    )
    .unwrap_err();
    assert_eq!(err.exit_code(), 3, "{err}");
    let err = interface_verify_json(
        &serde_json::json!({ "registry": registry, "spec": "999" }).to_string(),
    )
    .unwrap_err();
    assert_eq!(err.exit_code(), 1, "{err}");
}

// ---- 3.7: nothing else gates on a reference --------------------------------

#[test]
fn lint_is_silent_about_references_to_corpora_that_exist_nowhere() {
    let (t, with) = importer(&[reference("nowhere", EXPORTER_ID, &zero(), &[])]);
    assert!(with.validation_passed);
    let cfg = Config::default();
    let report = lint(&cfg, t.path()).unwrap();
    assert!(
        report
            .violations
            .iter()
            .all(|f| !f.message.contains("nowhere")),
        "{:?}",
        report.violations
    );
    let (t0, _) = importer(&[]);
    let report0 = lint(&cfg, t0.path()).unwrap();
    let codes = |r: &spec_spine_core::LintReport| {
        r.violations
            .iter()
            .map(|f| f.code.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(codes(&report), codes(&report0));
}

#[test]
fn verification_reads_exports_only_and_writes_nothing() {
    let exp = exporter(EXPORTER_BODY);
    let pin = pin_of(exp.path());
    let (t, out) = importer(&[reference("statecraft-cli", EXPORTER_ID, &pin.digest, &[])]);
    commit_ledger(t.path(), &out);
    let snap = |p: &Path| {
        let mut v: Vec<(PathBuf, Vec<u8>)> = walk(p)
            .into_iter()
            .map(|f| {
                let b = fs::read(&f).unwrap();
                (f, b)
            })
            .collect();
        v.sort();
        v
    };
    let (a0, b0) = (snap(t.path()), snap(exp.path()));
    let mut dirs = BTreeMap::new();
    dirs.insert("statecraft-cli".to_string(), exp.path().to_path_buf());
    interface_verify(&Config::default(), t.path(), &dirs, None).unwrap();
    assert_eq!(snap(t.path()), a0);
    assert_eq!(snap(exp.path()), b0);
}

fn walk(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in fs::read_dir(&d).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                stack.push(p);
            } else {
                out.push(p);
            }
        }
    }
    out
}

#[test]
fn exported_spec_is_constructible_by_a_binding() {
    // §3.4: the input types are public and plain, so a binding with no
    // filesystem can build them.
    let mut e = Exports::new();
    e.entry("c".into()).or_default().insert(
        "001-x".into(),
        ExportedSpec {
            path: "specs/001-x/spec.md".into(),
            text: "---\nid: \"001-x\"\n---\n# x\n".into(),
        },
    );
    assert_eq!(e["c"]["001-x"].path, "specs/001-x/spec.md");
}

// ---- 3.2: the embedded schemas hold the same grammar compile does ----------

#[test]
fn the_schemas_refuse_what_compile_refuses_in_a_reference() {
    let exp = exporter(EXPORTER_BODY);
    let pin = pin_of(exp.path());
    let (_t, out) = importer(&[reference("statecraft-cli", EXPORTER_ID, &pin.digest, &[])]);
    let shard: serde_json::Value =
        serde_json::from_str(&spec_spine_core::registry_shard_files(&out.shards).unwrap()[0].1)
            .unwrap();
    let registry = serde_json::to_value(&out.registry).unwrap();
    for (schema_src, instance, pointer) in [
        (
            spec_spine_types::REGISTRY_SPEC_SHARD_SCHEMA,
            shard,
            "/record/interfaceReferences/0",
        ),
        (
            spec_spine_types::REGISTRY_SCHEMA,
            registry,
            "/specs/0/interfaceReferences/0",
        ),
    ] {
        let schema: serde_json::Value = serde_json::from_str(schema_src).unwrap();
        let v = jsonschema::validator_for(&schema).unwrap();
        assert!(v.is_valid(&instance), "the emitted form conforms");
        for (member, bad) in [
            ("spec", "002"),
            ("spec", "not an id"),
            ("corpus", "Upper"),
            ("digest", "sha1:00"),
            ("obtained", "22/09/2026"),
        ] {
            let mut m = instance.clone();
            *m.pointer_mut(pointer).unwrap().get_mut(member).unwrap() = serde_json::json!(bad);
            assert!(!v.is_valid(&m), "{pointer}/{member} = {bad:?} was accepted");
        }
    }
}
