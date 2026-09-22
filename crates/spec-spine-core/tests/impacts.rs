// Spec: specs/109-impact-and-conflict-are-declared/spec.md
//! Spec 109: impact and conflict are declared against obligation ids,
//! validated, and answered by one inverting read. Each rule is exercised over
//! a disposable two-spec corpus through the real `compile`, so a code that
//! cannot fire fails here rather than in review.

use std::fs;
use std::path::Path;

use spec_spine_core::{DiffFile, DiffInput, compile, couple_with, impacts, index, lint};
use spec_spine_types::{Config, LineSpan, Registry};

const BODY: &str = "# 001\n\n## 1. Purpose\n\nWhy.\n\n## 3. Behavior\n\n### 3.1 The rule\n\nThe rule text.\n\n### 3.2 The other rule\n\nOther text.\n\n## Verification\n\nChecks.\n";

/// 001-a's obligations: `R-1` live, `R-2` withdrawn.
const A_OBLIGATIONS: &str = "obligations:\n  - id: \"R-1\"\n    kind: requirement\n    text: \"The rule holds.\"\n    anchor: \"3-1-the-rule\"\n  - id: \"R-2\"\n    kind: requirement\n    text: \"Old.\"\n    anchor: \"3-2-the-other-rule\"\n    withdrawn: true\n";

/// 002-b's obligations: `R-9` live, `R-8` withdrawn (for the successor rules).
const B_OBLIGATIONS: &str = "obligations:\n  - id: \"R-9\"\n    kind: requirement\n    text: \"New.\"\n    anchor: \"3-1-the-rule\"\n  - id: \"R-8\"\n    kind: requirement\n    text: \"Old own.\"\n    anchor: \"3-2-the-other-rule\"\n    withdrawn: true\n";

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

/// A two-spec corpus: `001-a` (declares `A_OBLIGATIONS`) and `002-b` (declares
/// `B_OBLIGATIONS` plus whatever `b_extra` adds, typically `impacts`/`conflicts`
/// against `001-a`).
fn compiled(b_extra: &str) -> (tempfile::TempDir, spec_spine_core::CompileOutcome) {
    let tmp = tempfile::tempdir().unwrap();
    write(tmp.path(), "001-a", A_OBLIGATIONS, BODY);
    write(
        tmp.path(),
        "002-b",
        &format!("{B_OBLIGATIONS}{b_extra}"),
        BODY,
    );
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

fn record<'a>(
    out: &'a spec_spine_core::CompileOutcome,
    id: &str,
) -> &'a spec_spine_types::SpecRecord {
    out.registry.specs.iter().find(|s| s.id == id).unwrap()
}

// ---- 3.1, 3.2, 3.3, 3.7: a valid declaration compiles, normalized ----------

// The two impacts target DIFFERENT obligations of 001-a (R-1, then the
// withdrawn R-2): the same target twice in one list is V-030, and a
// declaration against a withdrawn obligation is valid (§3.4).
const GOOD: &str = "impacts:\n  - obligation: \"001#R-1\"\n    nature: refines\n    note: \"n\"\n  - obligation: \"001-a#R-2\"\n    nature: supersedes\n    successor: \"R-9\"\nconflicts:\n  - obligation: \"001#R-1\"\n    reason: \"r\"\n    resolution: deliberate\n  - obligation: \"001-a#R-2\"\n    reason: \"r2\"\n    resolution: pending\n    settled_by: \"001\"\n";

#[test]
fn valid_declarations_compile_and_the_target_is_normalized_to_the_full_qualified_form() {
    let (_t, out) = compiled(GOOD);
    assert!(out.validation_passed, "{:?}", codes(&out));
    let b = record(&out, "002-b");
    assert_eq!(b.impacts.len(), 2);
    assert_eq!(b.impacts[0].obligation, "001-a#R-1", "short id normalized");
    assert_eq!(b.impacts[0].note.as_deref(), Some("n"));
    assert_eq!(b.impacts[1].successor.as_deref(), Some("R-9"));
    assert_eq!(b.conflicts.len(), 2);
    assert_eq!(b.conflicts[0].obligation, "001-a#R-1");
    // `settled_by` is a bare spec id, normalized the same way as `depends_on`.
    assert_eq!(b.conflicts[1].settled_by.as_deref(), Some("001-a"));

    // The shard carries them too, camelCase and `settledBy`.
    let shard = serde_json::to_value(
        out.shards
            .spec_shards
            .iter()
            .find(|s| s.record.id == "002-b")
            .unwrap(),
    )
    .unwrap();
    assert_eq!(shard["record"]["conflicts"][1]["settledBy"], "001-a");
    assert!(shard["record"]["conflicts"][1].get("settled_by").is_none());
}

// ---- 3.1: an unknown member or unqualified reference -----------------------

#[test]
fn an_unknown_member_is_malformed_frontmatter() {
    for extra in [
        "impacts:\n  - obligation: \"001#R-1\"\n    nature: refines\n    bogus: true\n",
        "conflicts:\n  - obligation: \"001#R-1\"\n    reason: \"r\"\n    resolution: deliberate\n    bogus: true\n",
    ] {
        let (_t, out) = compiled(extra);
        assert!(!out.validation_passed);
        assert!(codes(&out).iter().any(|(c, _)| c == "V-002"), "{extra}");
    }
}

#[test]
fn an_unqualified_reference_is_v025_for_impacts_and_conflicts() {
    for bad in ["R-1", "#R-1", "001#", "001#R#1"] {
        let (_t, out) = compiled(&format!(
            "impacts:\n  - obligation: \"{bad}\"\n    nature: informs\n"
        ));
        assert!(
            has(&out, "V-025", "not a qualified obligation reference"),
            "{bad}: {:?}",
            codes(&out)
        );
        let (_t, out) = compiled(&format!(
            "conflicts:\n  - obligation: \"{bad}\"\n    reason: \"r\"\n    resolution: deliberate\n"
        ));
        assert!(
            has(&out, "V-025", "not a qualified obligation reference"),
            "{bad}: {:?}",
            codes(&out)
        );
    }
}

// ---- 3.4 rule 2: dangling ----------------------------------------------------

#[test]
fn a_dangling_spec_or_obligation_is_v028() {
    let (_t, out) = compiled("impacts:\n  - obligation: \"999-none#R-1\"\n    nature: informs\n");
    assert!(
        has(&out, "V-028", "no spec '999-none'"),
        "{:?}",
        codes(&out)
    );

    let (_t, out) = compiled("impacts:\n  - obligation: \"001#R-99\"\n    nature: informs\n");
    assert!(
        has(&out, "V-028", "declares no obligation 'R-99'"),
        "{:?}",
        codes(&out)
    );
}

// ---- 3.4 rule 3: self-reference ---------------------------------------------

#[test]
fn a_self_targeting_reference_is_v029_even_when_the_obligation_exists() {
    let (_t, out) = compiled("impacts:\n  - obligation: \"002-b#R-9\"\n    nature: informs\n");
    assert!(
        has(
            &out,
            "V-029",
            "names an obligation of the declaring spec itself"
        ),
        "{:?}",
        codes(&out)
    );
    // The short form of the declaring spec's own id resolves the same way.
    let (_t, out) = compiled(
        "conflicts:\n  - obligation: \"002#R-9\"\n    reason: \"r\"\n    resolution: deliberate\n",
    );
    assert!(
        has(&out, "V-029", "declaring spec itself"),
        "{:?}",
        codes(&out)
    );
}

// ---- 3.4 rule 4: duplicate target, once short ids are normalized -----------

#[test]
fn a_duplicate_target_within_one_list_is_v030_and_across_lists_is_fine() {
    let (_t, out) = compiled(
        "impacts:\n  - obligation: \"001#R-1\"\n    nature: informs\n  - obligation: \"001-a#R-1\"\n    nature: refines\n",
    );
    assert!(has(&out, "V-030", "twice in impacts"), "{:?}", codes(&out));

    let (_t, out) = compiled(
        "conflicts:\n  - obligation: \"001#R-1\"\n    reason: \"a\"\n    resolution: deliberate\n  - obligation: \"001-a#R-1\"\n    reason: \"b\"\n    resolution: deliberate\n",
    );
    assert!(
        has(&out, "V-030", "twice in conflicts"),
        "{:?}",
        codes(&out)
    );

    // The same target once in impacts and once in conflicts is not a
    // duplicate: rule 4 is scoped to one list.
    let (_t, out) = compiled(GOOD);
    assert!(!has(&out, "V-030", ""), "{:?}", codes(&out));
}

// ---- 3.2: successor rules ----------------------------------------------------

#[test]
fn supersedes_requires_a_live_successor_this_spec_declares_v026() {
    let (_t, out) = compiled("impacts:\n  - obligation: \"001#R-1\"\n    nature: supersedes\n");
    assert!(
        has(&out, "V-026", "names no successor"),
        "{:?}",
        codes(&out)
    );

    let (_t, out) = compiled(
        "impacts:\n  - obligation: \"001#R-1\"\n    nature: supersedes\n    successor: \"R-none\"\n",
    );
    assert!(
        has(&out, "V-026", "is not an obligation this spec declares"),
        "{:?}",
        codes(&out)
    );

    // `R-8` is withdrawn on the declaring spec itself.
    let (_t, out) = compiled(
        "impacts:\n  - obligation: \"001#R-1\"\n    nature: supersedes\n    successor: \"R-8\"\n",
    );
    assert!(has(&out, "V-026", "is withdrawn"), "{:?}", codes(&out));

    // A live successor is fine.
    let (_t, out) = compiled(
        "impacts:\n  - obligation: \"001#R-1\"\n    nature: supersedes\n    successor: \"R-9\"\n",
    );
    assert!(out.validation_passed, "{:?}", codes(&out));
}

#[test]
fn a_successor_on_any_other_nature_is_v026() {
    for nature in ["refines", "extends", "informs"] {
        let (_t, out) = compiled(&format!(
            "impacts:\n  - obligation: \"001#R-1\"\n    nature: {nature}\n    successor: \"R-9\"\n"
        ));
        assert!(
            has(&out, "V-026", "only a 'supersedes' impact may"),
            "{nature}: {:?}",
            codes(&out)
        );
    }
}

// ---- 3.3: reason and settled_by ---------------------------------------------

#[test]
fn an_empty_reason_is_v027() {
    let (_t, out) = compiled(
        "conflicts:\n  - obligation: \"001#R-1\"\n    reason: \"   \"\n    resolution: deliberate\n",
    );
    assert!(has(&out, "V-027", "empty reason"), "{:?}", codes(&out));
}

#[test]
fn settled_by_presence_and_resolution_are_v031() {
    // pending without settled_by
    let (_t, out) = compiled(
        "conflicts:\n  - obligation: \"001#R-1\"\n    reason: \"r\"\n    resolution: pending\n",
    );
    assert!(
        has(&out, "V-031", "names no settled_by"),
        "{:?}",
        codes(&out)
    );

    // settled_by present on a non-pending conflict
    let (_t, out) = compiled(
        "conflicts:\n  - obligation: \"001#R-1\"\n    reason: \"r\"\n    resolution: deliberate\n    settled_by: \"001\"\n",
    );
    assert!(
        has(&out, "V-031", "only a 'pending' conflict may"),
        "{:?}",
        codes(&out)
    );

    // settled_by naming a spec that does not exist
    let (_t, out) = compiled(
        "conflicts:\n  - obligation: \"001#R-1\"\n    reason: \"r\"\n    resolution: pending\n    settled_by: \"999-none\"\n",
    );
    assert!(
        has(&out, "V-031", "does not resolve to an existing spec"),
        "{:?}",
        codes(&out)
    );

    // A valid pending conflict passes.
    let (_t, out) = compiled(
        "conflicts:\n  - obligation: \"001#R-1\"\n    reason: \"r\"\n    resolution: pending\n    settled_by: \"001\"\n",
    );
    assert!(out.validation_passed, "{:?}", codes(&out));
}

// ---- 3.4: a withdrawn target is valid ---------------------------------------

#[test]
fn a_declaration_naming_a_withdrawn_obligation_is_valid() {
    let (_t, out) = compiled("impacts:\n  - obligation: \"001#R-2\"\n    nature: informs\n");
    assert!(out.validation_passed, "{:?}", codes(&out));
    let set = impacts(&out.registry, None, None).unwrap();
    assert!(set.impacts[0].target_withdrawn, "{:?}", set.impacts);
}

// ---- 3.7: compatibility ------------------------------------------------------

#[test]
fn a_corpus_without_either_key_compiles_to_the_file_hash_unchanged() {
    let (tmp, out) = compiled("");
    let shard = out
        .shards
        .spec_shards
        .iter()
        .find(|s| s.record.id == "002-b")
        .unwrap();
    let value = serde_json::to_value(shard).unwrap();
    assert!(value["record"].get("impacts").is_none(), "{value}");
    assert!(value["record"].get("conflicts").is_none(), "{value}");
    let raw = fs::read_to_string(tmp.path().join("specs/002-b/spec.md")).unwrap();
    let expected = {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(b"specs/002-b/spec.md");
        h.update([0u8]);
        h.update(raw.as_bytes());
        h.finalize()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    };
    assert_eq!(shard.shard_hash, expected);
    assert_eq!(shard.spec_version, "1.5.0");
}

#[test]
fn no_gate_reads_an_impact_or_a_conflict() {
    // The same corpus twice, once with impacts/conflicts declared on 002-b.
    // `couple` over the same diff and `lint` reach the same verdicts (the
    // unresolved-conflict L-014 warning is exercised separately below).
    let setup = |extra: &str| {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(root, "001-a", A_OBLIGATIONS, BODY);
        write(
            root,
            "002-b",
            &format!("establishes: [\"src/lib.rs\"]\n{B_OBLIGATIONS}{extra}"),
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
        (tmp, couple.violations)
    };
    let (_a, without) = setup("");
    let (_b, with) = setup(GOOD);
    assert_eq!(without, with);
    assert!(!without.is_empty(), "the diff drifts either way");
}

// ---- 3.3: the lint warning ---------------------------------------------------

#[test]
fn an_unresolved_conflict_is_exactly_one_l014_warning_and_deliberate_pending_are_silent() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write(root, "001-a", A_OBLIGATIONS, BODY);
    write(
        root,
        "002-b",
        &format!(
            "{B_OBLIGATIONS}conflicts:\n  - obligation: \"001#R-1\"\n    reason: \"a\"\n    resolution: unresolved\n"
        ),
        BODY,
    );
    let cfg = Config::default();
    let report = lint(&cfg, root).unwrap();
    let l014: Vec<_> = report
        .violations
        .iter()
        .filter(|v| v.code == "L-014")
        .collect();
    assert_eq!(l014.len(), 1, "{:?}", report.violations);
    assert!(l014[0].message.contains("002-b"), "{:?}", l014[0]);
    assert!(l014[0].message.contains("001-a#R-1"), "{:?}", l014[0]);

    // deliberate and pending conflicts raise no L-014.
    let tmp2 = tempfile::tempdir().unwrap();
    let root2 = tmp2.path();
    write(root2, "001-a", A_OBLIGATIONS, BODY);
    write(root2, "002-b", &format!("{B_OBLIGATIONS}{GOOD}"), BODY);
    let report2 = lint(&cfg, root2).unwrap();
    assert!(
        report2.violations.iter().all(|v| v.code != "L-014"),
        "{:?}",
        report2.violations
    );
}

// ---- 3.6: the read -----------------------------------------------------------

#[test]
fn the_read_inverts_sorted_by_target_then_declared_by() {
    let (_t, out) = compiled(GOOD);
    let set = impacts(&out.registry, None, None).unwrap();
    assert_eq!(set.impacts.len(), 2);
    assert_eq!(set.conflicts.len(), 2);
    // Every entry's target and declaredBy are full ids.
    for e in &set.impacts {
        assert_eq!(e.declared_by, "002-b");
        assert!(e.target.starts_with("001-a#"), "{}", e.target);
    }
    // Sorted by (target, declaredBy): the two impacts target R-1 and the
    // (withdrawn) R-2, so the sort is provable from the target order alone.
    let targets: Vec<&str> = set.impacts.iter().map(|e| e.target.as_str()).collect();
    let mut sorted = targets.clone();
    sorted.sort();
    assert_eq!(targets, sorted);
}

#[test]
fn the_target_filter_composes_with_declared_by_by_intersection() {
    let (_t, out) = compiled(GOOD);
    let reg = &out.registry;

    // target as a bare spec id: every obligation it declares.
    let by_spec = impacts(reg, Some("001-a"), None).unwrap();
    assert_eq!(by_spec.impacts.len(), 2);
    assert_eq!(by_spec.conflicts.len(), 2);

    // target as a qualified reference: exact match.
    let by_ref = impacts(reg, Some("001#R-1"), None).unwrap();
    assert_eq!(by_ref.impacts.len(), 1);
    assert_eq!(
        by_ref.impacts[0].nature,
        spec_spine_types::ImpactNature::Refines
    );

    // declared_by alone.
    let by_declarer = impacts(reg, None, Some("002")).unwrap();
    assert_eq!(by_declarer.impacts.len(), 2);

    // both, composed by intersection: R-2 carries one impact (the
    // supersession) and one conflict (the pending one), both declared by 002-b.
    let both = impacts(reg, Some("001-a#R-2"), Some("002-b")).unwrap();
    assert_eq!(both.conflicts.len(), 1);
    assert_eq!(both.impacts.len(), 1);
    assert_eq!(
        both.impacts[0].nature,
        spec_spine_types::ImpactNature::Supersedes
    );

    // an intersection that matches nothing is a true empty result, not a
    // refusal: 001-a exists and declares nothing about 002-b's obligations.
    let empty = impacts(reg, Some("002-b"), Some("001-a")).unwrap();
    assert!(empty.impacts.is_empty() && empty.conflicts.is_empty());
}

#[test]
fn the_reads_three_refusals() {
    let (_t, out) = compiled(GOOD);
    let reg = &out.registry;

    // 1. a `#` reference that is not qualified: exit 3, never resolved.
    for bad in ["#R-1", "001#", "001#R#1"] {
        let err = impacts(reg, Some(bad), None).unwrap_err();
        assert_eq!(err.exit_code(), 3, "{bad}: {err}");
    }
    // 2. a target spec that does not exist.
    assert_eq!(
        impacts(reg, Some("999-none"), None)
            .unwrap_err()
            .exit_code(),
        1
    );
    // 3. a target obligation that does not exist on an existing spec.
    assert_eq!(
        impacts(reg, Some("001#R-99"), None)
            .unwrap_err()
            .exit_code(),
        1
    );
    // a declaring spec that does not exist.
    assert_eq!(
        impacts(reg, None, Some("999-none"))
            .unwrap_err()
            .exit_code(),
        1
    );
}

#[test]
fn the_facade_answers_the_same_op() {
    let (_t, out) = compiled(GOOD);
    let req = serde_json::json!({
        "registry": out.json,
        "op": "impacts",
        "target": "001-a",
    });
    let doc: serde_json::Value =
        serde_json::from_str(&spec_spine_core::query_json(&req.to_string()).unwrap()).unwrap();
    assert!(doc["schemaVersion"].is_string());
    assert_eq!(doc["impacts"].as_array().unwrap().len(), 2);
    assert_eq!(doc["conflicts"].as_array().unwrap().len(), 2);
}
