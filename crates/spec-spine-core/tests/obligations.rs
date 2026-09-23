// Spec: specs/106-obligations-are-declared-constraints/spec.md
//! Spec 106: obligations are declared constraints, and every section has a
//! digest. Each rule is exercised over a disposable corpus through the real
//! `compile`, so a code that cannot fire fails here rather than in review.

use std::fs;
use std::path::Path;

use spec_spine_core::{DiffFile, DiffInput, compile, couple_with, index, lint, obligation};
use spec_spine_types::{Config, LineSpan, ObligationKind, Registry};

const BODY: &str = "# 001\n\n## 1. Purpose\n\nWhy.\n\n## 3. Behavior\n\n### 3.1 The rule\n\nThe rule text.\n\n### 3.2 The other rule\n\nOther text.\n\n## Verification\n\nChecks.\n";

fn write(root: &Path, id: &str, extra: &str, body: &str) {
    let dir = root.join("specs").join(id);
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("spec.md"),
        format!(
            "---\nid: \"{id}\"\ntitle: \"T\"\nstatus: draft\ncreated: \"2026-09-22\"\n\
             summary: \"s\"\n{extra}---\n{body}"
        ),
    )
    .unwrap();
}

fn compiled(extra: &str, body: &str) -> (tempfile::TempDir, spec_spine_core::CompileOutcome) {
    let tmp = tempfile::tempdir().unwrap();
    write(tmp.path(), "001-a", extra, body);
    let out = compile(&Config::default(), tmp.path()).unwrap();
    (tmp, out)
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

const GOOD: &str = "obligations:\n  - id: \"R-1\"\n    kind: requirement\n    text: \"The rule holds.\"\n    anchor: \"3-1-the-rule\"\n  - id: \"I-1\"\n    kind: invariant\n    text: \"Nothing moves.\"\n    anchor: \"3-2-the-other-rule\"\n  - id: \"V-1\"\n    kind: verification\n    text: \"It is checked.\"\n    anchor: \"verification\"\n    inputs: [\"tests/a.rs\"]\n  - id: \"R-2\"\n    kind: requirement\n    text: \"The old rule.\"\n    anchor: \"3-1-the-rule\"\n    withdrawn: true\n";

// ---- 3.1, 3.2: a valid declaration compiles and is carried verbatim --------

#[test]
fn valid_obligations_compile_into_the_record() {
    let (_t, out) = compiled(GOOD, BODY);
    assert!(out.validation_passed, "{:?}", codes(&out));
    let rec = &out.registry.specs[0];
    let ids: Vec<&str> = rec.obligations.iter().map(|o| o.id.as_str()).collect();
    assert_eq!(ids, ["R-1", "I-1", "V-1", "R-2"], "authored order is kept");
    assert_eq!(rec.obligations[2].kind, ObligationKind::Verification);
    assert_eq!(rec.obligations[2].inputs, ["tests/a.rs"]);
    assert!(rec.obligations[3].withdrawn);
    // The shard carries them too, and a withdrawn flag is only written when true.
    let shard = serde_json::to_value(&out.shards.spec_shards[0]).unwrap();
    let obs = &shard["record"]["obligations"];
    assert_eq!(obs[3]["withdrawn"], true);
    assert!(obs[0].get("withdrawn").is_none(), "{obs}");
    assert!(obs[0].get("inputs").is_none(), "{obs}");
}

#[test]
fn an_unknown_kind_or_member_is_malformed_frontmatter() {
    for (extra, why) in [
        (
            "obligations:\n  - id: \"R-1\"\n    kind: constraint\n    text: \"t\"\n    anchor: \"3-1-the-rule\"\n",
            "a fourth kind",
        ),
        (
            "obligations:\n  - id: \"R-1\"\n    kind: requirement\n    text: \"t\"\n    anchor: \"3-1-the-rule\"\n    withdrawm: true\n",
            "a misspelt member",
        ),
        (
            "obligations:\n  - id: \"R-1\"\n    kind: requirement\n    text: \"t\"\n",
            "a missing anchor",
        ),
    ] {
        let (_t, out) = compiled(extra, BODY);
        assert!(!out.validation_passed, "{why}");
        assert!(
            codes(&out).iter().any(|(c, _)| c == "V-002"),
            "{why}: {:?}",
            codes(&out)
        );
    }
}

// ---- 3.3: ids ---------------------------------------------------------------

#[test]
fn a_bad_or_duplicate_id_is_v021_and_a_tombstone_still_occupies_its_id() {
    let (_t, out) = compiled(
        "obligations:\n  - id: \"R#1\"\n    kind: requirement\n    text: \"t\"\n    anchor: \"3-1-the-rule\"\n",
        BODY,
    );
    assert!(
        has(&out, "V-021", "'R#1' does not match"),
        "{:?}",
        codes(&out)
    );

    // Reuse of a withdrawn id for a new obligation is an ordinary duplicate.
    let (_t, out) = compiled(
        "obligations:\n  - id: \"R-1\"\n    kind: requirement\n    text: \"old\"\n    anchor: \"3-1-the-rule\"\n    withdrawn: true\n  - id: \"R-1\"\n    kind: requirement\n    text: \"new\"\n    anchor: \"3-2-the-other-rule\"\n",
        BODY,
    );
    assert!(
        has(&out, "V-021", "'R-1' is declared twice"),
        "{:?}",
        codes(&out)
    );
}

#[test]
fn empty_text_is_v024() {
    let (_t, out) = compiled(
        "obligations:\n  - id: \"R-1\"\n    kind: requirement\n    text: \"  \"\n    anchor: \"3-1-the-rule\"\n",
        BODY,
    );
    assert!(
        has(&out, "V-024", "'R-1' has empty text"),
        "{:?}",
        codes(&out)
    );
}

// ---- 3.4: anchors -----------------------------------------------------------

#[test]
fn a_dangling_or_ambiguous_anchor_is_v022_naming_the_anchor() {
    let (_t, out) = compiled(
        "obligations:\n  - id: \"R-1\"\n    kind: requirement\n    text: \"t\"\n    anchor: \"3-9-nowhere\"\n",
        BODY,
    );
    assert!(
        has(&out, "V-022", "'3-9-nowhere' names no heading"),
        "{:?}",
        codes(&out)
    );

    let twice = format!("{BODY}\n### 3.1 The rule\n\nAgain.\n");
    let (_t, out) = compiled(
        "obligations:\n  - id: \"R-1\"\n    kind: requirement\n    text: \"t\"\n    anchor: \"3-1-the-rule\"\n",
        &twice,
    );
    assert!(
        has(&out, "V-022", "is ambiguous: 2 headings"),
        "{:?}",
        codes(&out)
    );
}

#[test]
fn a_frontmatter_comment_is_never_a_heading() {
    // `# 3.9 ghost` sits in the frontmatter as a YAML comment. The anchor it
    // would slug to must not resolve: headings come from the body only.
    let (_t, out) = compiled(
        "# 3.9 ghost\nobligations:\n  - id: \"R-1\"\n    kind: requirement\n    text: \"t\"\n    anchor: \"3-9-ghost\"\n",
        BODY,
    );
    assert!(
        has(&out, "V-022", "'3-9-ghost' names no heading"),
        "{:?}",
        codes(&out)
    );
    assert!(
        !out.registry.specs[0]
            .section_digests
            .contains_key("3-9-ghost")
    );
}

// ---- 3.7: inputs ------------------------------------------------------------

#[test]
fn inputs_belong_to_a_verification_and_only_to_one_v023() {
    let (_t, out) = compiled(
        "obligations:\n  - id: \"V-1\"\n    kind: verification\n    text: \"t\"\n    anchor: \"verification\"\n",
        BODY,
    );
    assert!(
        has(&out, "V-023", "'V-1' declares no inputs"),
        "{:?}",
        codes(&out)
    );

    let (_t, out) = compiled(
        "obligations:\n  - id: \"R-1\"\n    kind: requirement\n    text: \"t\"\n    anchor: \"3-1-the-rule\"\n    inputs: [\"x\"]\n",
        BODY,
    );
    assert!(
        has(&out, "V-023", "only a verification obligation may"),
        "{:?}",
        codes(&out)
    );
}

// ---- 3.5: section digests ---------------------------------------------------

fn digests(body: &str) -> (std::collections::BTreeMap<String, String>, String) {
    let (_t, out) = compiled("", body);
    let shard = &out.shards.spec_shards[0];
    (
        shard.record.section_digests.clone(),
        shard.shard_hash.clone(),
    )
}

#[test]
fn every_section_is_digested_and_a_digest_moves_only_with_its_section() {
    let (base, base_hash) = digests(BODY);
    let anchors: Vec<&str> = base.keys().map(String::as_str).collect();
    assert_eq!(
        anchors,
        [
            "001",
            "1-purpose",
            "3-1-the-rule",
            "3-2-the-other-rule",
            "3-behavior",
            "verification"
        ],
        "every body heading, not only anchored ones"
    );
    assert!(base.values().all(|d| d.len() == 64));

    // Edit 3.2's prose only.
    let (edited, edited_hash) = digests(&BODY.replace("Other text.", "Other text, changed."));
    assert_ne!(edited["3-2-the-other-rule"], base["3-2-the-other-rule"]);
    // Its parent contains it, so the parent moves too.
    assert_ne!(edited["3-behavior"], base["3-behavior"]);
    // A sibling and an unrelated section do not.
    assert_eq!(edited["3-1-the-rule"], base["3-1-the-rule"]);
    assert_eq!(edited["1-purpose"], base["1-purpose"]);
    assert_eq!(edited["verification"], base["verification"]);
    // The spec's full identity moves in both cases.
    assert_ne!(edited_hash, base_hash);
    let (other, other_hash) = digests(&BODY.replace("Why.", "Why, changed."));
    assert_ne!(other["1-purpose"], base["1-purpose"]);
    assert_eq!(other["3-2-the-other-rule"], base["3-2-the-other-rule"]);
    assert_ne!(other_hash, base_hash);
}

#[test]
fn a_digest_is_the_one_hash_construction_over_the_named_section() {
    let (_t, out) = compiled("", BODY);
    let rec = &out.registry.specs[0];
    let expected = {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(b"specs/001-a/spec.md#3-1-the-rule");
        h.update([0u8]);
        h.update(b"### 3.1 The rule\n\nThe rule text.\n\n");
        h.finalize()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    };
    assert_eq!(rec.section_digests["3-1-the-rule"], expected);
}

// ---- 3.9: compatibility -----------------------------------------------------

#[test]
fn a_spec_without_the_key_carries_no_obligations_and_its_shard_hash_is_the_file_hash() {
    let (tmp, out) = compiled("", BODY);
    let shard = serde_json::to_value(&out.shards.spec_shards[0]).unwrap();
    assert!(shard["record"].get("obligations").is_none(), "{shard}");
    let raw = fs::read_to_string(tmp.path().join("specs/001-a/spec.md")).unwrap();
    let expected = {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(b"specs/001-a/spec.md");
        h.update([0u8]);
        h.update(raw.as_bytes());
        h.finalize()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    };
    assert_eq!(out.shards.spec_shards[0].shard_hash, expected);
    // Spec 109's and 110's extends edges on this file: each additive registry
    // MINOR after 106's shipped (109's `impacts`/`conflicts`, 110's
    // `interfaceReferences`) moves this pin the same way 106 itself once
    // moved a pin in 082's territory (106 D-8).
    assert_eq!(out.shards.spec_shards[0].spec_version, "1.6.0");
}

#[test]
fn no_gate_reads_an_obligation() {
    // The same corpus twice, once with obligations declared on the owning
    // spec. `couple` over the same diff and `lint` reach the same verdicts.
    let setup = |extra: &str| {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(
            root,
            "001-a",
            &format!("establishes: [\"src/lib.rs\"]\n{extra}"),
            BODY,
        );
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/lib.rs"), "pub fn f() {}\n").unwrap();
        let cfg = Config::default();
        let registry: Registry = compile(&cfg, root).unwrap().registry;
        let idx = index(&cfg, root).unwrap().index;
        let diff = DiffInput {
            files: vec![DiffFile {
                path: "src/lib.rs".into(),
                hunks: vec![LineSpan::new(1, 1)],
                deleted: false,
            }],
        };
        let couple = couple_with(&cfg, &registry, &idx, &diff, None).unwrap();
        let lint = lint(&cfg, root).unwrap();
        (tmp, couple.violations, lint.violations.len())
    };
    let (_a, without, lint_without) = setup("");
    let (_b, with, lint_with) = setup(GOOD);
    assert_eq!(without, with);
    assert!(
        !without.is_empty(),
        "the diff drifts either way; the verdict is not vacuous"
    );
    assert_eq!(lint_without, lint_with);
}

// ---- 3.6: the read ----------------------------------------------------------

#[test]
fn a_qualified_reference_resolves_and_an_unqualified_one_is_refused() {
    let (_t, out) = compiled(GOOD, BODY);
    let reg = &out.registry;
    let view = obligation(reg, "001#R-1").unwrap();
    assert_eq!(view.spec, "001-a");
    assert_eq!(view.obligation.text, "The rule holds.");
    assert_eq!(
        view.section_digest,
        Some(reg.specs[0].section_digests["3-1-the-rule"].as_str())
    );
    // A withdrawn obligation still resolves, and says so.
    assert!(obligation(reg, "001-a#R-2").unwrap().obligation.withdrawn);

    let err = obligation(reg, "R-1").unwrap_err();
    assert_eq!(err.exit_code(), 3, "{err}");
    assert!(err.to_string().contains("a qualified form"), "{err}");
    for bad in ["#R-1", "001#", "001#R#1"] {
        assert_eq!(obligation(reg, bad).unwrap_err().exit_code(), 3, "{bad}");
    }
    assert_eq!(obligation(reg, "001#R-9").unwrap_err().exit_code(), 1);
    assert_eq!(obligation(reg, "999#R-1").unwrap_err().exit_code(), 1);
}

#[test]
fn the_facade_answers_the_same_question_without_a_content_hash() {
    let (_t, out) = compiled(GOOD, BODY);
    let req = serde_json::json!({ "registry": out.json, "op": "obligation", "id": "001#V-1" });
    let doc: serde_json::Value =
        serde_json::from_str(&spec_spine_core::query_json(&req.to_string()).unwrap()).unwrap();
    assert_eq!(
        doc["obligation"]["inputs"],
        serde_json::json!(["tests/a.rs"])
    );
    assert_eq!(doc["spec"], "001-a");
    assert!(doc["sectionDigest"].is_string());
    assert!(doc.get("contentHash").is_none(), "{doc}");
    assert!(doc["schemaVersion"].is_string());
}

/// D-11: the registry schema refuses the obligation `text` compile refuses.
/// A whitespace-only `text` validated against the schema before the pattern
/// was added; both embedded registry schemas now reject it, and a real text
/// still passes.
#[test]
fn the_registry_schema_refuses_a_whitespace_only_obligation_text() {
    let schema: serde_json::Value =
        serde_json::from_str(spec_spine_types::REGISTRY_SPEC_SHARD_SCHEMA).unwrap();
    let text_schema =
        &schema["$defs"]["specRecord"]["properties"]["obligations"]["items"]["properties"]["text"];
    assert!(
        text_schema.is_object(),
        "the shard schema no longer has an obligation text at the expected path: {schema}"
    );
    let v = jsonschema::validator_for(text_schema).unwrap();
    assert!(!v.is_valid(&serde_json::json!("   ")));
    assert!(!v.is_valid(&serde_json::json!("")));
    assert!(v.is_valid(&serde_json::json!("The rule holds.")));

    let full: serde_json::Value = serde_json::from_str(spec_spine_types::REGISTRY_SCHEMA).unwrap();
    let rendered = full.to_string();
    assert!(
        rendered.contains(r#""text":{"minLength":1,"pattern":"\\S","type":"string"}"#),
        "the aggregate registry schema's obligation text lost its pattern"
    );
}
