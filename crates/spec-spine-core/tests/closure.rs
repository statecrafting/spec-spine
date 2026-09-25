// Spec: specs/107-a-context-closure-is-declared/spec.md
//! Spec 107: a context closure is resolved against the ledger and
//! content-addressed. Each property of the digest is asserted in both
//! directions, so a digest that ignored a member, or folded one it should not,
//! fails here.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use spec_spine_core::{
    ClosureMember, ClosureRequest, SectionRef, closure, closure_json, compile, resolve_closure,
};
use spec_spine_types::{Config, Error, Registry};

const A_BODY: &str = "# 001\n\n## 3. Behavior\n\n### 3.1 The rule\n\nThe rule text.\n\n### 3.2 The other rule\n\nOther text.\n\n## Verification\n\nChecks.\n";
const A_OBS: &str = "obligations:\n  - id: \"R-1\"\n    kind: requirement\n    text: \"The rule holds.\"\n    anchor: \"3-1-the-rule\"\n  - id: \"V-1\"\n    kind: verification\n    text: \"It is checked.\"\n    anchor: \"verification\"\n    inputs: [\"tests/a.rs\"]\n";
const B_BODY: &str = "# 002\n\n## 1. Purpose\n\nWhy.\n\n## 2. Territory\n\nWhere.\n";

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

/// A two-spec corpus compiled in memory: the registry and each spec's shard
/// hash, which is exactly what the resolver reads.
fn ledger(a_extra: &str, a_body: &str, b_body: &str) -> (Registry, BTreeMap<String, String>) {
    let tmp = tempfile::tempdir().unwrap();
    write(tmp.path(), "001-a", a_extra, a_body);
    write(tmp.path(), "002-b", "", b_body);
    let out = compile(&Config::default(), tmp.path()).unwrap();
    assert!(out.validation_passed, "{:?}", out.registry.validation);
    let hashes = out
        .shards
        .spec_shards
        .iter()
        .map(|s| (s.record.id.clone(), s.shard_hash.clone()))
        .collect();
    (out.registry, hashes)
}

fn request(specs: &[&str], sections: &[(&str, &str)], obligations: &[&str]) -> ClosureRequest {
    ClosureRequest {
        specs: specs.iter().map(|s| s.to_string()).collect(),
        sections: sections
            .iter()
            .map(|(spec, anchor)| SectionRef {
                spec: spec.to_string(),
                anchor: anchor.to_string(),
            })
            .collect(),
        obligations: obligations.iter().map(|s| s.to_string()).collect(),
        rationale: None,
    }
}

fn digest(reg: &(Registry, BTreeMap<String, String>), req: &ClosureRequest) -> String {
    resolve_closure(&reg.0, &reg.1, req).unwrap().digest
}

/// The closure most tests use: one whole spec, one section, one obligation.
fn standard() -> ClosureRequest {
    request(&["002-b"], &[("001", "3-2-the-other-rule")], &["001#R-1"])
}

// ---- 3.2, 3.3: resolution ----------------------------------------------------

#[test]
fn every_kind_resolves_to_its_identity_in_a_stable_order() {
    let reg = ledger(A_OBS, A_BODY, B_BODY);
    let mut req = standard();
    req.rationale = Some("why these".into());
    let c = resolve_closure(&reg.0, &reg.1, &req).unwrap();
    assert_eq!(c.rationale.as_deref(), Some("why these"));
    assert_eq!(c.digest.len(), 64);
    let a = reg.0.specs.iter().find(|s| s.id == "001-a").unwrap();
    assert_eq!(
        c.members,
        vec![
            ClosureMember::Obligation {
                spec: "001-a".into(),
                id: "R-1".into(),
                obligation_kind: spec_spine_types::ObligationKind::Requirement,
                text: "The rule holds.".into(),
                anchor: "3-1-the-rule".into(),
                inputs: vec![],
                withdrawn: false,
                section_digest: a.section_digests["3-1-the-rule"].clone(),
            },
            ClosureMember::Section {
                spec: "001-a".into(),
                anchor: "3-2-the-other-rule".into(),
                digest: a.section_digests["3-2-the-other-rule"].clone(),
            },
            ClosureMember::Spec {
                spec: "002-b".into(),
                content_hash: reg.1["002-b"].clone(),
            },
        ]
    );
}

// ---- 3.4: what the digest is over --------------------------------------------

#[test]
fn the_digest_ignores_order_repetition_and_how_a_spec_was_named() {
    let reg = ledger(A_OBS, A_BODY, B_BODY);
    let base = digest(&reg, &standard());
    let shuffled = request(
        &["002", "002-b"],
        &[
            ("001-a", "3-2-the-other-rule"),
            ("001", "3-2-the-other-rule"),
        ],
        &["001-a#R-1", "001#R-1"],
    );
    assert_eq!(digest(&reg, &shuffled), base);
    // Rationale is carried, never digested.
    let mut with_why = standard();
    with_why.rationale = Some("anything".into());
    assert_eq!(digest(&reg, &with_why), base);
}

#[test]
fn the_digest_moves_with_every_named_member_and_with_nothing_else() {
    let base = digest(&ledger(A_OBS, A_BODY, B_BODY), &standard());
    let moved = |a_extra: &str, a_body: &str, b_body: &str| {
        digest(&ledger(a_extra, a_body, b_body), &standard())
    };
    // A named section's prose.
    assert_ne!(
        moved(
            A_OBS,
            &A_BODY.replace("Other text.", "Other, edited."),
            B_BODY
        ),
        base
    );
    // The prose of the section a named obligation points at.
    assert_ne!(
        moved(A_OBS, &A_BODY.replace("The rule text.", "Edited."), B_BODY),
        base
    );
    // The obligation's own text, and its withdrawal.
    assert_ne!(
        moved(
            &A_OBS.replace("The rule holds.", "It holds."),
            A_BODY,
            B_BODY
        ),
        base
    );
    assert_ne!(
        moved(
            &A_OBS.replace(
                "anchor: \"3-1-the-rule\"\n",
                "anchor: \"3-1-the-rule\"\n    withdrawn: true\n"
            ),
            A_BODY,
            B_BODY
        ),
        base
    );
    // A named whole spec: any edit to it.
    assert_ne!(
        moved(A_OBS, A_BODY, &B_BODY.replace("Where.", "Elsewhere.")),
        base
    );
    // An unnamed section of a spec that is not named whole, and an obligation
    // nobody named: no movement.
    assert_eq!(
        moved(A_OBS, &A_BODY.replace("Checks.", "More checks."), B_BODY),
        base
    );
    assert_eq!(
        moved(&A_OBS.replace("It is checked.", "Checked."), A_BODY, B_BODY),
        base
    );
}

// ---- 3.1, 3.2: refusals -------------------------------------------------------

#[test]
fn an_empty_request_and_an_unqualified_obligation_are_parse_errors() {
    let reg = ledger(A_OBS, A_BODY, B_BODY);
    let empty = resolve_closure(&reg.0, &reg.1, &request(&[], &[], &[])).unwrap_err();
    assert_eq!(empty.exit_code(), 3, "{empty}");
    // Refused before anything else, even beside a reference that is missing.
    let unq = resolve_closure(&reg.0, &reg.1, &request(&["999"], &[], &["R-1"])).unwrap_err();
    assert!(matches!(unq, Error::Usage(_)), "{unq}");
    assert!(unq.to_string().contains("a qualified form"), "{unq}");
}

#[test]
fn every_unresolved_reference_is_named_in_one_refusal() {
    let reg = ledger(A_OBS, A_BODY, B_BODY);
    let err = resolve_closure(
        &reg.0,
        &reg.1,
        &request(&["999"], &[("001", "3-9-none")], &["001#R-9", "888#R-1"]),
    )
    .unwrap_err();
    assert_eq!(err.exit_code(), 1, "{err}");
    let msg = err.to_string();
    for needle in [
        "spec '999'",
        "001-a#3-9-none",
        "declares no 'R-9'",
        "'888#R-1'",
    ] {
        assert!(msg.contains(needle), "missing {needle}: {msg}");
    }
}

#[test]
fn an_unknown_request_member_is_refused() {
    let err = serde_json::from_str::<ClosureRequest>(r#"{ "obligation": ["001#R-1"] }"#)
        .unwrap_err()
        .to_string();
    assert!(err.contains("unknown field"), "{err}");
}

// ---- 3.5, 3.6, 3.8: the IO wrapper and the facade --------------------------------

fn repo() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    write(tmp.path(), "001-a", A_OBS, A_BODY);
    write(tmp.path(), "002-b", "", B_BODY);
    let cfg = Config::default();
    let out = compile(&cfg, tmp.path()).unwrap();
    let dir = spec_spine_core::registry_dir(&cfg, tmp.path()).join("by-spec");
    for (name, bytes) in spec_spine_core::registry_shard_files(&out.shards).unwrap() {
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(name), bytes).unwrap();
    }
    tmp
}

fn snapshot(root: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut out = BTreeMap::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for e in fs::read_dir(&dir).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                stack.push(p);
            } else {
                out.insert(p.display().to_string(), fs::read(&p).unwrap());
            }
        }
    }
    out
}

#[test]
fn the_committed_ledger_answers_and_nothing_is_written() {
    let tmp = repo();
    let before = snapshot(tmp.path());
    let c = closure(&Config::default(), tmp.path(), &standard()).unwrap();
    assert_eq!(snapshot(tmp.path()), before, "a closure is a read");
    // The facade answers the same digest, as a read document.
    let doc: serde_json::Value = serde_json::from_str(
        &closure_json(
            "{}",
            tmp.path().to_str().unwrap(),
            r#"{"specs":["002"],"sections":[{"spec":"001","anchor":"3-2-the-other-rule"}],"obligations":["001#R-1"]}"#,
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(doc["digest"], c.digest.as_str());
    assert!(doc["schemaVersion"].is_string());
    assert_eq!(doc["members"][0]["kind"], "obligation");
}

#[test]
fn a_stale_ledger_is_refused_before_anything_is_digested() {
    let tmp = repo();
    let spec = tmp.path().join("specs/001-a/spec.md");
    let text = fs::read_to_string(&spec).unwrap();
    fs::write(
        &spec,
        text.replace("Other text.", "Edited, not recompiled."),
    )
    .unwrap();
    let err = closure(&Config::default(), tmp.path(), &standard()).unwrap_err();
    assert_eq!(err.exit_code(), 1, "{err}");
}

/// D-9: the two guards a valid ledger never reaches, asserted on a ledger that
/// breaks each invariant. An obligation whose anchor has no section digest,
/// and a named spec with no committed content hash, are refused by name; an
/// empty digest or an absent hash is never folded in.
#[test]
fn a_ledger_missing_a_digest_or_a_content_hash_is_refused_not_folded() {
    let (mut registry, hashes) = ledger(A_OBS, A_BODY, B_BODY);
    let a = registry.specs.iter_mut().find(|s| s.id == "001-a").unwrap();
    assert!(a.section_digests.remove("3-1-the-rule").is_some());
    let err = resolve_closure(&registry, &hashes, &request(&[], &[], &["001#R-1"])).unwrap_err();
    assert_eq!(err.exit_code(), 1, "{err}");
    assert!(
        err.to_string()
            .contains("its anchor '3-1-the-rule' has no section digest in the ledger"),
        "{err}"
    );
    // The same obligation's sibling, whose digest is intact, still resolves.
    assert!(resolve_closure(&registry, &hashes, &request(&[], &[], &["001#V-1"])).is_ok());

    let (registry, mut hashes) = ledger(A_OBS, A_BODY, B_BODY);
    assert!(hashes.remove("002-b").is_some());
    let err = resolve_closure(&registry, &hashes, &request(&["002"], &[], &[])).unwrap_err();
    assert_eq!(err.exit_code(), 1, "{err}");
    assert!(
        err.to_string()
            .contains("spec '002-b' has no committed shard"),
        "{err}"
    );
}
