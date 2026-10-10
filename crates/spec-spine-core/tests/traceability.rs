// Spec: specs/169-declared-obligation-traceability/spec.md
//! Spec 169: declared obligation traceability. Every rule runs over a
//! disposable corpus through the real `compile` and `index`, so a refusal
//! that cannot fire, or a state that cannot be reached, fails here.

use std::fs;
use std::path::Path;

use spec_spine_core::{
    Versioning, compile, index, parse_traceability_report, read_document, traceability,
};
use spec_spine_types::{
    CodebaseIndex, Config, REGISTRY_SCHEMA_VERSION, Registry, TRACEABILITY_SCHEMA_VERSION,
    TraceRelation, TraceState, TraceabilityRequest,
};

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

fn registry_and_index(root: &Path) -> (Registry, CodebaseIndex) {
    let out = compile(&Config::default(), root).unwrap();
    assert!(
        out.validation_passed,
        "{:?}",
        out.registry.validation.violations
    );
    let idx = index(&Config::default(), root).unwrap().index;
    (out.registry, idx)
}

fn report(root: &Path, request: &TraceabilityRequest) -> spec_spine_types::TraceabilityReport {
    let (registry, idx) = registry_and_index(root);
    traceability(&Config::default(), root, &registry, Some(&idx), request).unwrap()
}

fn by_id<'a>(rels: &'a [TraceRelation], id: &str) -> &'a TraceRelation {
    rels.iter()
        .find(|r| r.id == id)
        .unwrap_or_else(|| panic!("no relation {id}"))
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
    let (registry, _) = registry_and_index(tmp.path());
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

#[test]
fn an_obligation_without_relations_is_not_reported() {
    let tmp = corpus_with("");
    let r = report(tmp.path(), &TraceabilityRequest::default());
    assert!(r.relations.is_empty());
    assert!(r.summary.values().all(|n| *n == 0));
    assert_eq!(r.summary.len(), 6, "every state is counted");
}

// ---- 3.5, 3.6: every state, every relation kind, every target kind --------

#[test]
fn every_state_is_reached_and_none_stands_in_for_another() {
    let tmp = full_corpus(&all());
    let r = report(tmp.path(), &TraceabilityRequest::default());
    let state = |id: &str| by_id(&r.relations, id).state;
    assert_eq!(state("t-impl"), TraceState::Resolved);
    assert_eq!(state("t-inv"), TraceState::Resolved);
    assert_eq!(state("t-doc"), TraceState::Resolved);
    assert_eq!(state("t-doc-section"), TraceState::Resolved);
    assert_eq!(state("t-prod"), TraceState::Resolved);
    assert_eq!(state("t-cons"), TraceState::Resolved);
    assert_eq!(state("t-unowned"), TraceState::Unresolved);
    assert_eq!(state("t-doc-missing"), TraceState::Unresolved);
    assert_eq!(state("t-planned"), TraceState::Unresolved);
    assert_eq!(state("t-inv-kind"), TraceState::Unresolved);
    assert_eq!(state("t-inv-gone"), TraceState::Withdrawn);
    assert_eq!(state("t-gone"), TraceState::Withdrawn);
    assert_eq!(state("t-test"), TraceState::Unsupported);
    assert_eq!(state("t-doc-proj"), TraceState::Unsupported);
    #[cfg(feature = "symbol-resolution")]
    assert_eq!(state("t-sym"), TraceState::Ambiguous);
    #[cfg(not(feature = "symbol-resolution"))]
    assert_eq!(state("t-sym"), TraceState::Unsupported);

    // Every state but `resolved` says why.
    for rel in &r.relations {
        assert_eq!(
            rel.detail.is_none(),
            rel.state == TraceState::Resolved,
            "{}",
            rel.id
        );
    }
    // The summary counts what the relations say.
    for s in TraceState::ALL {
        assert_eq!(
            r.summary[&s],
            r.relations.iter().filter(|x| x.state == s).count()
        );
    }
}

#[test]
fn the_planned_unit_resolves_once_it_exists() {
    let tmp = full_corpus(&all());
    fs::write(tmp.path().join("src/later.rs"), "pub fn later() {}\n").unwrap();
    let r = report(tmp.path(), &TraceabilityRequest::default());
    assert_eq!(by_id(&r.relations, "t-planned").state, TraceState::Resolved);
}

#[test]
fn without_a_fresh_index_a_unit_target_is_unknown() {
    let tmp = full_corpus(&all());
    let out = compile(&Config::default(), tmp.path()).unwrap();
    let r = traceability(
        &Config::default(),
        tmp.path(),
        &out.registry,
        None,
        &TraceabilityRequest::default(),
    )
    .unwrap();
    for id in ["t-impl", "t-unowned", "t-planned", "t-prod"] {
        assert_eq!(by_id(&r.relations, id).state, TraceState::Unknown, "{id}");
    }
    // A target that needs no index is unaffected.
    assert_eq!(by_id(&r.relations, "t-inv").state, TraceState::Resolved);
    assert_eq!(by_id(&r.relations, "t-doc").state, TraceState::Resolved);
}

// ---- 3.5, 3.10: ownership is never proof, and nothing is inferred ---------

#[test]
fn ownership_alone_produces_no_relation_and_no_resolution() {
    // 001-a claims src/lib.rs and docs/a.md, and declares nothing: the read
    // reports no relation for either, however exactly the claim resolves.
    let tmp = full_corpus(&all());
    let r = report(
        tmp.path(),
        &TraceabilityRequest {
            declared_by: Some("001".into()),
            ..TraceabilityRequest::default()
        },
    );
    assert!(r.relations.is_empty());

    // 002-b names src/lib.rs as a unit of 002-b, which does not claim it. The
    // file resolves and 001-a owns it, and the relation is still unresolved:
    // the claim is checked against the spec named, never substituted.
    let all = report(tmp.path(), &TraceabilityRequest::default());
    let unowned = by_id(&all.relations, "t-unowned");
    assert_eq!(unowned.state, TraceState::Unresolved);
    assert!(
        unowned
            .detail
            .as_deref()
            .unwrap()
            .contains("does not claim")
    );

    // Every reported relation is one the corpus declares, and only those.
    let declared: Vec<&str> = RELATIONS
        .iter()
        .map(|r| r.split('"').nth(1).unwrap())
        .collect();
    assert_eq!(all.relations.len(), declared.len());
    for rel in &all.relations {
        assert!(declared.contains(&rel.id.as_str()), "{}", rel.id);
    }
}

// ---- 3.4: canonical identity and order, whatever the authoring order ------

#[test]
fn output_bytes_do_not_depend_on_authoring_order() {
    let forward = full_corpus(&all());
    let mut rev = all();
    rev.reverse();
    let backward = full_corpus(&rev);
    let a = read_document(
        &report(forward.path(), &TraceabilityRequest::default()),
        Versioning::Stamp,
    )
    .unwrap();
    let b = read_document(
        &report(backward.path(), &TraceabilityRequest::default()),
        Versioning::Stamp,
    )
    .unwrap();
    assert_eq!(a, b);

    let r = report(forward.path(), &TraceabilityRequest::default());
    let ids: Vec<&str> = r.relations.iter().map(|x| x.id.as_str()).collect();
    let mut sorted = ids.clone();
    sorted.sort();
    assert_eq!(ids, sorted, "declaring spec, then relation id");
}

#[test]
fn the_report_matches_its_committed_fixture() {
    let tmp = full_corpus(&all());
    let doc = read_document(
        &report(tmp.path(), &TraceabilityRequest::default()),
        Versioning::Stamp,
    )
    .unwrap();
    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join(if cfg!(feature = "symbol-resolution") {
            "tests/fixtures/traceability/report.json"
        } else {
            "tests/fixtures/traceability/report-no-symbols.json"
        });
    if std::env::var_os("SPEC_SPINE_BLESS").is_some() {
        fs::create_dir_all(fixture.parent().unwrap()).unwrap();
        fs::write(&fixture, &doc).unwrap();
    }
    let expected = fs::read_to_string(&fixture).unwrap().replace("\r\n", "\n");
    assert_eq!(doc, expected);
}

#[test]
fn target_identities_have_the_canonical_forms() {
    let tmp = full_corpus(&all());
    let r = report(tmp.path(), &TraceabilityRequest::default());
    let ident = |id: &str| by_id(&r.relations, id).target_identity.clone();
    assert_eq!(
        ident("t-impl"),
        r#"unit:001-a:{"kind":"file","path":"src/lib.rs"}"#
    );
    assert_eq!(
        ident("t-planned"),
        r#"unit:001-a:{"kind":"file","path":"src/later.rs"}"#,
        "planned never enters an identity"
    );
    assert_eq!(
        ident("t-test"),
        r#"test:{"id":"demo::tests::orders_relations","kind":"test"}"#
    );
    assert_eq!(ident("t-inv"), "invariant:001-a#I-1");
    assert_eq!(
        ident("t-doc"),
        r#"documentation:{"kind":"file","path":"docs/a.md"}"#
    );
    assert_eq!(
        ident("t-prod"),
        r#"interface:producer:local:001-a:{"kind":"file","path":"src/lib.rs"}"#
    );
    assert_eq!(
        ident("t-cons"),
        "interface:consumer:external:002-b:other:010-x"
    );
    assert_eq!(by_id(&r.relations, "t-impl").identity, "002-b#trace:t-impl");
}

// ---- 3.9: filters intersect ------------------------------------------------

#[test]
fn filters_intersect() {
    let tmp = full_corpus(&all());
    let r = report(
        tmp.path(),
        &TraceabilityRequest {
            declared_by: Some("002".into()),
            obligation: Some("001#R-1".into()),
            state: Some(TraceState::Resolved),
        },
    );
    let ids: Vec<&str> = r.relations.iter().map(|x| x.id.as_str()).collect();
    assert_eq!(ids, ["t-doc", "t-impl", "t-inv"]);

    let none = report(
        tmp.path(),
        &TraceabilityRequest {
            declared_by: Some("001".into()),
            obligation: Some("001#R-1".into()),
            state: None,
        },
    );
    assert!(none.relations.is_empty());
}

#[test]
fn an_unqualified_or_unknown_filter_is_refused() {
    let tmp = full_corpus(&all());
    let (registry, idx) = registry_and_index(tmp.path());
    let run = |req: TraceabilityRequest| {
        traceability(&Config::default(), tmp.path(), &registry, Some(&idx), &req)
    };
    assert!(matches!(
        run(TraceabilityRequest {
            obligation: Some("R-1".into()),
            ..Default::default()
        }),
        Err(spec_spine_types::Error::Usage(_))
    ));
    assert!(matches!(
        run(TraceabilityRequest {
            declared_by: Some("999".into()),
            ..Default::default()
        }),
        Err(spec_spine_types::Error::NotFound(_))
    ));
}

// ---- 3.9: the document's own axis ------------------------------------------

#[test]
fn a_reader_accepts_a_minor_and_refuses_another_major() {
    let tmp = full_corpus(&all());
    let doc = read_document(
        &report(tmp.path(), &TraceabilityRequest::default()),
        Versioning::Stamp,
    )
    .unwrap();
    let parsed = parse_traceability_report(&doc).unwrap();
    assert_eq!(parsed.traceability_version, TRACEABILITY_SCHEMA_VERSION);

    let mut value: serde_json::Value = serde_json::from_str(&doc).unwrap();
    value["traceabilityVersion"] = "1.7.0".into();
    value["relations"][0]["addedLater"] = true.into();
    assert!(parse_traceability_report(&value.to_string()).is_ok());

    value["traceabilityVersion"] = "2.0.0".into();
    assert!(matches!(
        parse_traceability_report(&value.to_string()),
        Err(spec_spine_types::Error::Refused(_))
    ));

    // An unknown state is a parse failure, never downcast to `unresolved`.
    let mut value: serde_json::Value = serde_json::from_str(&doc).unwrap();
    value["relations"][0]["state"] = "maybe".into();
    assert!(matches!(
        parse_traceability_report(&value.to_string()),
        Err(spec_spine_types::Error::Parse(_))
    ));
    let mut value: serde_json::Value = serde_json::from_str(&doc).unwrap();
    value["relations"][0]["relation"] = "verified-by".into();
    assert!(parse_traceability_report(&value.to_string()).is_err());
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

// ---- 3.9: the read document validates against its embedded schema --------

#[test]
fn the_report_conforms_to_its_embedded_schema() {
    let tmp = full_corpus(&all());
    let doc = read_document(
        &report(tmp.path(), &TraceabilityRequest::default()),
        Versioning::Stamp,
    )
    .unwrap();
    let value: serde_json::Value = serde_json::from_str(&doc).unwrap();
    let schema: serde_json::Value =
        serde_json::from_str(spec_spine_types::TRACEABILITY_SCHEMA).unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();
    let errors: Vec<String> = validator
        .iter_errors(&value)
        .map(|e| e.to_string())
        .collect();
    assert!(errors.is_empty(), "{errors:?}");

    // And the schema can refuse: an unknown state does not validate.
    let mut bad = value.clone();
    bad["relations"][0]["state"] = "maybe".into();
    assert!(!validator.is_valid(&bad));
    let mut bad = value;
    bad["traceabilityVersion"] = "2.0.0".into();
    assert!(!validator.is_valid(&bad));
}
