// Spec: specs/169-declared-obligation-traceability/spec.md
//! Spec 169: declared obligation traceability. Every rule runs over a
//! disposable corpus through the real `compile`, so a refusal that cannot
//! fire fails here.

use std::fs;
use std::path::Path;

use spec_spine_core::compile;
use spec_spine_types::{Config, REGISTRY_SCHEMA_VERSION, Registry};

const BODY: &str = "# spec\n\n## 1. Purpose\n\nWhy.\n\n## 3. Behavior\n\n### 3.1 The rule\n\nThe rule text.\n\n### 3.2 The invariant\n\nAlways.\n\n### 3.3 The old rule\n\nOld.\n\n## Verification\n\nChecks.\n";

/// 001-a: a live requirement, a live invariant, a withdrawn requirement; it
/// claims `src/lib.rs` and, planned, `src/later.rs`.
const A: &str = "establishes:\n  - \"src/lib.rs\"\n  - { kind: file, path: \"src/later.rs\", planned: true }\n  - \"docs/a.md\"\nobligations:\n  - id: \"R-1\"\n    kind: requirement\n    text: \"The rule holds.\"\n    anchor: \"3-1-the-rule\"\n  - id: \"I-1\"\n    kind: invariant\n    text: \"Always.\"\n    anchor: \"3-2-the-invariant\"\n  - id: \"R-2\"\n    kind: requirement\n    text: \"Old.\"\n    anchor: \"3-3-the-old-rule\"\n    withdrawn: true\n";

/// 002-b: one live requirement and one cross-corpus interface reference.
const B_HEAD: &str = "obligations:\n  - id: \"R-9\"\n    kind: requirement\n    text: \"New.\"\n    anchor: \"3-1-the-rule\"\ninterface_references:\n  - corpus: \"other\"\n    spec: \"010-x\"\n    digest: \"sha256:0000000000000000000000000000000000000000000000000000000000000000\"\n    obtained: \"2026-10-10\"\n";

/// One relation per (state, relation kind, target kind) the read must reach,
/// in deliberately unsorted authoring order.
const RELATIONS: &[&str] = &[
    "  - id: \"t-unowned\"\n    obligation: \"001#R-1\"\n    relation: implemented-by\n    target: { kind: unit, spec: \"002\", unit: \"src/lib.rs\" }\n",
    "  - id: \"t-impl\"\n    obligation: \"001#R-1\"\n    relation: implemented-by\n    target: { kind: unit, spec: \"001\", unit: \"src/lib.rs\" }\n",
    "  - id: \"t-test\"\n    obligation: \"001#R-1\"\n    relation: tested-by\n    target: { kind: test, selector: \"demo::tests::orders_relations\" }\n",
    "  - id: \"t-inv\"\n    obligation: \"001#R-1\"\n    relation: enforced-by\n    target: { kind: invariant, obligation: \"001#I-1\" }\n",
    "  - id: \"t-inv-kind\"\n    obligation: \"002#R-9\"\n    relation: enforced-by\n    target: { kind: invariant, obligation: \"001#R-1\" }\n",
    "  - id: \"t-inv-gone\"\n    obligation: \"002#R-9\"\n    relation: enforced-by\n    target: { kind: invariant, obligation: \"001#R-2\" }\n",
    "  - id: \"t-doc\"\n    obligation: \"001#R-1\"\n    relation: documented-by\n    target: { kind: documentation, selector: { kind: file, path: \"docs/a.md\" } }\n",
    "  - id: \"t-doc-section\"\n    obligation: \"002#R-9\"\n    relation: documented-by\n    target: { kind: documentation, selector: { kind: spec-section, spec: \"001\", anchor: \"3-1-the-rule\" } }\n",
    "  - id: \"t-doc-missing\"\n    obligation: \"001#R-1\"\n    relation: documented-by\n    target: { kind: documentation, selector: { kind: file, path: \"docs/missing.md\" } }\n",
    "  - id: \"t-doc-proj\"\n    obligation: \"001#R-1\"\n    relation: documented-by\n    target: { kind: documentation, selector: { kind: file, path: \"docs/a.md\", projection: signature } }\n",
    "  - id: \"t-planned\"\n    obligation: \"001#R-1\"\n    relation: implemented-by\n    target: { kind: unit, spec: \"001\", unit: { kind: file, path: \"src/later.rs\", planned: true } }\n",
    "  - id: \"t-prod\"\n    obligation: \"002#R-9\"\n    relation: produced-by\n    target: { kind: interface, role: producer, spec: \"001\", unit: \"src/lib.rs\" }\n",
    "  - id: \"t-cons\"\n    obligation: \"002#R-9\"\n    relation: consumed-by\n    target: { kind: interface, role: consumer, corpus: \"other\", spec: \"010-x\" }\n",
    "  - id: \"t-gone\"\n    obligation: \"002#R-9\"\n    relation: implemented-by\n    target: { kind: unit, spec: \"001\", unit: \"src/lib.rs\" }\n    withdrawn: true\n",
    "  - id: \"t-sym\"\n    obligation: \"002#R-9\"\n    relation: documented-by\n    target: { kind: documentation, selector: { kind: symbol, id: \"demo::twice\" } }\n",
];

fn write_spec(root: &Path, id: &str, extra: &str) {
    let dir = root.join("specs").join(id);
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("spec.md"),
        format!(
            "---\nid: \"{id}\"\ntitle: \"T\"\nstatus: draft\ncreated: \"2026-10-10\"\n\
             summary: \"s\"\n{extra}---\n{BODY}"
        ),
    )
    .unwrap();
}

/// The source tree: a Rust package whose `twice` is declared twice under
/// mutually exclusive `cfg`s, which spec 155 refuses to choose between.
fn write_tree(root: &Path) {
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("docs")).unwrap();
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    fs::write(
        root.join("src/lib.rs"),
        "#[cfg(unix)]\npub fn twice() {}\n#[cfg(not(unix))]\npub fn twice() {}\n",
    )
    .unwrap();
    fs::write(root.join("docs/a.md"), "# A\n\nThe documentation.\n").unwrap();
}

fn corpus_with(b_extra: &str) -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    write_tree(tmp.path());
    write_spec(tmp.path(), "001-a", A);
    write_spec(tmp.path(), "002-b", &format!("{B_HEAD}{b_extra}"));
    tmp
}

fn relations_block(order: &[usize]) -> String {
    let mut s = String::from("traceability:\n");
    for i in order {
        s.push_str(RELATIONS[*i]);
    }
    s
}

fn full_corpus(order: &[usize]) -> tempfile::TempDir {
    corpus_with(&relations_block(order))
}

fn compiled_registry(root: &Path) -> Registry {
    let out = compile(&Config::default(), root).unwrap();
    assert!(
        out.validation_passed,
        "{:?}",
        out.registry.validation.violations
    );
    out.registry
}

fn all() -> Vec<usize> {
    (0..RELATIONS.len()).collect()
}

fn violations(b_extra: &str) -> Vec<(String, String)> {
    let tmp = corpus_with(b_extra);
    let out = compile(&Config::default(), tmp.path()).unwrap();
    out.registry
        .validation
        .violations
        .iter()
        .map(|v| (v.code.clone(), v.message.clone()))
        .collect()
}

fn refused(b_extra: &str, code: &str, needle: &str) {
    let v = violations(b_extra);
    assert!(
        v.iter().any(|(c, m)| c == code && m.contains(needle)),
        "expected {code} containing {needle:?}, got {v:?}"
    );
}

fn one(relation: &str) -> String {
    format!("traceability:\n{relation}")
}

// ---- 3.1, 3.4: the declaration compiles, normalized, into the registry ----

#[test]
fn declarations_compile_with_every_spec_reference_normalized() {
    let tmp = full_corpus(&all());
    let registry = compiled_registry(tmp.path());
    let b = registry.specs.iter().find(|s| s.id == "002-b").unwrap();
    assert_eq!(b.traceability.len(), RELATIONS.len());
    let impl_ = b.traceability.iter().find(|t| t.id == "t-impl").unwrap();
    assert_eq!(impl_.obligation, "001-a#R-1", "source spec half normalized");
    let json = serde_json::to_value(&impl_.target).unwrap();
    assert_eq!(json["spec"], "001-a", "unit target spec normalized");
    let inv = b.traceability.iter().find(|t| t.id == "t-inv").unwrap();
    assert_eq!(
        serde_json::to_value(&inv.target).unwrap()["obligation"],
        "001-a#I-1"
    );
    let sec = b
        .traceability
        .iter()
        .find(|t| t.id == "t-doc-section")
        .unwrap();
    assert_eq!(
        serde_json::to_value(&sec.target).unwrap()["selector"]["spec"],
        "001-a"
    );
    // The cross-corpus spec names another corpus: never rewritten.
    let cons = b.traceability.iter().find(|t| t.id == "t-cons").unwrap();
    assert_eq!(serde_json::to_value(&cons.target).unwrap()["spec"], "010-x");
}

#[test]
fn a_shard_carries_the_member_and_the_schema_admits_it() {
    let tmp = full_corpus(&all());
    let out = compile(&Config::default(), tmp.path()).unwrap();
    let shard = out
        .shards
        .spec_shards
        .iter()
        .find(|s| s.record.id == "002-b")
        .unwrap();
    assert_eq!(shard.spec_version, REGISTRY_SCHEMA_VERSION);
    let value = serde_json::to_value(shard).unwrap();
    assert_eq!(value["record"]["traceability"][0]["id"], "t-unowned");
    let withdrawn = value["record"]["traceability"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["id"] == "t-gone")
        .unwrap();
    assert_eq!(withdrawn["withdrawn"], true);
    let schema: serde_json::Value =
        serde_json::from_str(spec_spine_types::REGISTRY_SPEC_SHARD_SCHEMA).unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();
    let errors: Vec<String> = validator
        .iter_errors(&value)
        .map(|e| e.to_string())
        .collect();
    assert!(errors.is_empty(), "{errors:?}");
}

// ---- 3.9: optional, and absent means byte-identical apart from specVersion -

#[test]
fn an_absent_declaration_is_omitted_and_keeps_the_shard_hash() {
    let tmp = corpus_with("");
    let out = compile(&Config::default(), tmp.path()).unwrap();
    assert!(out.validation_passed);
    let b = out
        .shards
        .spec_shards
        .iter()
        .find(|s| s.record.id == "002-b")
        .unwrap();
    let value = serde_json::to_value(b).unwrap();
    assert!(value["record"].get("traceability").is_none());

    // The same spec with an empty list: the member is still omitted, and the
    // hash moves only because the source bytes did.
    let tmp2 = corpus_with("traceability: []\n");
    let out2 = compile(&Config::default(), tmp2.path()).unwrap();
    let b2 = out2
        .shards
        .spec_shards
        .iter()
        .find(|s| s.record.id == "002-b")
        .unwrap();
    assert!(
        serde_json::to_value(b2).unwrap()["record"]
            .get("traceability")
            .is_none()
    );
    let a1 = out
        .shards
        .spec_shards
        .iter()
        .find(|s| s.record.id == "001-a")
        .unwrap();
    let a2 = out2
        .shards
        .spec_shards
        .iter()
        .find(|s| s.record.id == "001-a")
        .unwrap();
    assert_eq!(a1.shard_hash, a2.shard_hash, "an untouched spec's hash");
    assert_eq!(a1, a2, "an untouched spec's shard is byte-identical");
}

// ---- 4: the negative cases compile refuses ---------------------------------

#[test]
fn an_invalid_or_duplicate_id_is_refused() {
    refused(
        &one(
            "  - id: \"1-bad\"\n    obligation: \"001#R-1\"\n    relation: tested-by\n    target: { kind: test, selector: \"x\" }\n",
        ),
        "V-044",
        "not a valid relation id",
    );
    refused(
        &format!(
            "{}{}",
            one(RELATIONS[2]),
            RELATIONS[2].replace("tests::orders_relations", "other")
        ),
        "V-044",
        "declared twice",
    );
}

#[test]
fn a_bad_source_obligation_is_refused() {
    refused(
        &one(
            "  - id: \"t\"\n    obligation: \"R-1\"\n    relation: tested-by\n    target: { kind: test, selector: \"x\" }\n",
        ),
        "V-044",
        "not a qualified obligation reference",
    );
    refused(
        &one(
            "  - id: \"t\"\n    obligation: \"001#R-404\"\n    relation: tested-by\n    target: { kind: test, selector: \"x\" }\n",
        ),
        "V-045",
        "does not resolve",
    );
    refused(
        &one(
            "  - id: \"t\"\n    obligation: \"404#R-1\"\n    relation: tested-by\n    target: { kind: test, selector: \"x\" }\n",
        ),
        "V-045",
        "does not resolve",
    );
    refused(
        &one(
            "  - id: \"t\"\n    obligation: \"001#R-2\"\n    relation: tested-by\n    target: { kind: test, selector: \"x\" }\n",
        ),
        "V-045",
        "is withdrawn",
    );
    // A withdrawn relation keeps its identity after its source is retired.
    let ok = violations(&one(
        "  - id: \"t\"\n    obligation: \"001#R-2\"\n    relation: tested-by\n    target: { kind: test, selector: \"x\" }\n    withdrawn: true\n",
    ));
    assert!(ok.is_empty(), "{ok:?}");
    // A relation MAY cite the declaring spec's own obligation.
    let own = violations(&one(
        "  - id: \"t\"\n    obligation: \"002#R-9\"\n    relation: tested-by\n    target: { kind: test, selector: \"x\" }\n",
    ));
    assert!(own.is_empty(), "{own:?}");
}

#[test]
fn an_ambiguous_source_is_refused() {
    // Two specs share the ordinal 001, so `001#R-1` names neither.
    let tmp = full_corpus(&[1]);
    write_spec(tmp.path(), "001-z", A);
    let out = compile(&Config::default(), tmp.path()).unwrap();
    assert!(
        out.registry
            .validation
            .violations
            .iter()
            .any(|v| v.code == "V-045" && v.message.contains("001#R-1"))
    );
}

#[test]
fn an_unknown_relation_target_kind_or_member_is_malformed() {
    for extra in [
        "  - id: \"t\"\n    obligation: \"001#R-1\"\n    relation: verified-by\n    target: { kind: test, selector: \"x\" }\n",
        "  - id: \"t\"\n    obligation: \"001#R-1\"\n    relation: tested-by\n    target: { kind: benchmark, selector: \"x\" }\n",
        "  - id: \"t\"\n    obligation: \"001#R-1\"\n    relation: tested-by\n    target: { kind: test, selector: \"x\", extra: 1 }\n",
        "  - id: \"t\"\n    obligation: \"001#R-1\"\n    relation: tested-by\n    target: { kind: test }\n",
        "  - id: \"t\"\n    obligation: \"001#R-1\"\n    relation: implemented-by\n    target: { kind: unit, unit: \"src/lib.rs\" }\n",
        "  - id: \"t\"\n    obligation: \"001#R-1\"\n    relation: tested-by\n    target: { kind: test, selector: \"x\" }\n    bogus: true\n",
        "  - {}\n",
    ] {
        let v = violations(&one(extra));
        assert!(v.iter().any(|(c, _)| c == "V-002"), "{extra}: {v:?}");
    }
}

#[test]
fn a_disallowed_pairing_is_refused() {
    refused(
        &one(
            "  - id: \"t\"\n    obligation: \"001#R-1\"\n    relation: tested-by\n    target: { kind: unit, spec: \"001\", unit: \"src/lib.rs\" }\n",
        ),
        "V-044",
        "does not allow",
    );
    refused(
        &one(
            "  - id: \"t\"\n    obligation: \"001#R-1\"\n    relation: documented-by\n    target: { kind: documentation, selector: { kind: test, id: \"x\" } }\n",
        ),
        "V-044",
        "test selector as documentation",
    );
}

#[test]
fn a_duplicate_normalized_tuple_is_refused() {
    let dup = RELATIONS[1]
        .replace("t-impl", "t-again")
        .replace("001#R-1", "001-a#R-1")
        .replace("spec: \"001\"", "spec: \"001-a\"");
    refused(
        &format!("{}{dup}", one(RELATIONS[1])),
        "V-046",
        "already declares",
    );
}

#[test]
fn an_interface_with_a_wrong_role_or_form_is_refused() {
    refused(
        &one(
            "  - id: \"t\"\n    obligation: \"002#R-9\"\n    relation: produced-by\n    target: { kind: interface, role: consumer, spec: \"001\", unit: \"src/lib.rs\" }\n",
        ),
        "V-044",
        "a producer is 'produced-by'",
    );
    refused(
        &one(
            "  - id: \"t\"\n    obligation: \"002#R-9\"\n    relation: produced-by\n    target: { kind: interface, role: producer, spec: \"010-x\", corpus: \"other\", unit: \"src/lib.rs\" }\n",
        ),
        "V-044",
        "both of its local",
    );
    refused(
        &one(
            "  - id: \"t\"\n    obligation: \"002#R-9\"\n    relation: produced-by\n    target: { kind: interface, role: producer, spec: \"001\" }\n",
        ),
        "V-044",
        "neither of its local",
    );
    refused(
        &one(
            "  - id: \"t\"\n    obligation: \"002#R-9\"\n    relation: consumed-by\n    target: { kind: interface, role: consumer, corpus: \"other\", spec: \"011-y\" }\n",
        ),
        "V-044",
        "matches 0 of this spec's interface_references",
    );
}

#[test]
fn a_well_formed_unresolvable_declaration_still_compiles() {
    // Unresolved, unsupported and withdrawn are data, not compile failures.
    let v = violations(&relations_block(&all()));
    assert!(v.is_empty(), "{v:?}");
}
