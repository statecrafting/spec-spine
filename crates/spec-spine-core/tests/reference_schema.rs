//! Spec 166: the reference-page schema accepts what the Rust types accept
//! and nothing more, member by member, for the members whose Rust type is
//! narrower than a bare JSON type.

use serde_json::{Value, json};
use spec_spine_types::{Obligation, REFERENCE_PAGE_SCHEMA, ReferenceSignature};

fn page_member(name: &str) -> Value {
    let schema: Value = serde_json::from_str(REFERENCE_PAGE_SCHEMA).expect("schema is JSON");
    schema["$defs"]["page"]["properties"][name].clone()
}

fn valid(schema: &Value, instance: &Value) -> bool {
    jsonschema::validator_for(schema)
        .expect("schema compiles")
        .is_valid(instance)
}

fn signature(start: Value, end: Value) -> Value {
    json!([{
        "digest": null,
        "endLine": end,
        "language": "rust",
        "omission": null,
        "path": "src/lib.rs",
        "startLine": start,
        "symbol": "crate::f",
        "text": null,
    }])
}

#[test]
fn a_signature_line_is_what_usize_accepts() {
    let schema = page_member("signatures");
    let ok = signature(json!(1), json!(2));
    assert!(valid(&schema, &ok));
    serde_json::from_value::<Vec<ReferenceSignature>>(ok).unwrap();
    assert!(valid(&schema, &signature(Value::Null, Value::Null)));
    for (start, end) in [(json!(-1), json!(2)), (json!(1), json!(-2))] {
        let bad = signature(start, end);
        assert!(
            serde_json::from_value::<Vec<ReferenceSignature>>(bad.clone()).is_err(),
            "{bad}"
        );
        assert!(!valid(&schema, &bad), "{bad}");
    }
}

#[test]
fn a_requirement_is_an_obligation_record() {
    let schema = page_member("requirements");
    let ok = json!([{
        "anchor": "3-1-one-export-one-page-kind",
        "id": "R-1",
        "kind": "requirement",
        "text": "One page per exported spec.",
    }]);
    assert!(valid(&schema, &ok));
    serde_json::from_value::<Vec<Obligation>>(ok).unwrap();
    for bad in [
        json!([{}]),
        json!([{"id": "R-1", "kind": "requirement", "text": "t"}]),
        json!([{"anchor": "a", "id": "R-1", "kind": "rule", "text": "t"}]),
        json!([{"anchor": "a", "id": "R-1", "kind": "requirement", "text": "t", "extra": 1}]),
    ] {
        assert!(
            serde_json::from_value::<Vec<Obligation>>(bad.clone()).is_err(),
            "{bad}"
        );
        assert!(!valid(&schema, &bad), "{bad}");
    }
}
