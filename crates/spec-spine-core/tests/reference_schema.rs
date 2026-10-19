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

/// Each closed set the schema enumerates: every value it lists parses into
/// the Rust type and validates, and a value it does not list is refused by
/// both, so neither side is wider than the other.
#[test]
fn every_closed_set_is_closed_on_both_sides() {
    use spec_spine_types::{ReferenceLifecycle, ReferenceRelation, ReferenceUnit};

    fn check<T: serde::de::DeserializeOwned>(
        schema: &Value,
        make: impl Fn(Value) -> Value,
        listed: &[Value],
    ) {
        for v in listed.iter().cloned().chain([json!("bogus")]) {
            let doc = make(v.clone());
            let listed = listed.contains(&v);
            assert_eq!(valid(schema, &doc), listed, "schema: {doc}");
            assert_eq!(
                serde_json::from_value::<T>(doc.clone()).is_ok(),
                listed,
                "serde: {doc}"
            );
        }
    }
    let enum_of = |schema: &Value, member: &str| -> Vec<Value> {
        schema["properties"][member]["enum"]
            .as_array()
            .unwrap_or_else(|| panic!("{member} enumerates its values"))
            .clone()
    };

    let lifecycle = page_member("lifecycle");
    let life = |status: Value, implementation: Value| {
        json!({"created": "2026-01-01", "implementation": implementation,
               "retirementRationale": null, "status": status, "supersededBy": null})
    };
    check::<ReferenceLifecycle>(
        &lifecycle,
        |v| life(v, Value::Null),
        &enum_of(&lifecycle, "status"),
    );
    check::<ReferenceLifecycle>(
        &lifecycle,
        |v| life(json!("draft"), v),
        &enum_of(&lifecycle, "implementation"),
    );

    let units = page_member("ownership")["properties"]["units"].clone();
    let unit = units["items"].clone();
    let row = |edge: Value, resolution: Value| {
        json!({"edge": edge, "planned": false, "resolution": resolution,
               "targetSpec": null, "unit": "{\"kind\":\"file\",\"path\":\"a\"}"})
    };
    check::<ReferenceUnit>(
        &unit,
        |v| row(v, json!("resolved")),
        &enum_of(&unit, "edge"),
    );
    check::<ReferenceUnit>(
        &unit,
        |v| row(json!("establishes"), v),
        &enum_of(&unit, "resolution"),
    );

    let relation = page_member("relations")["oneOf"][1]["items"].clone();
    check::<ReferenceRelation>(
        &relation,
        |v| {
            json!({"id": "r", "obligation": "001-a#R-1", "relation": "tested-by",
                   "state": v, "targetIdentity": "t"})
        },
        &enum_of(&relation, "state"),
    );

    let signature = page_member("signatures")["items"].clone();
    check::<ReferenceSignature>(
        &signature,
        |v| {
            json!({"digest": null, "endLine": null, "language": v, "omission": "x",
                   "path": null, "startLine": null, "symbol": "s", "text": null})
        },
        &enum_of(&signature, "language"),
    );
}
