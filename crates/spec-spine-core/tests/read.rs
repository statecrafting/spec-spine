//! The read-document emitter's properties (spec 074 §3.8): sorted keys whatever
//! the struct order, array wrapping, the canonical layout, and the three
//! refusals. The per-document assertions live in `spec-spine-cli/tests/cli.rs`,
//! because the defect was a set of call sites, not this function.

use serde::Serialize;
use spec_spine_core::{Versioning, read_document};
use spec_spine_types::{Error, READ_SCHEMA_VERSION};

/// Declared out of alphabetical order on purpose, nested too.
#[derive(Serialize)]
struct Unsorted {
    zebra: u8,
    mango: Inner,
    apple: u8,
}

#[derive(Serialize)]
struct Inner {
    yak: u8,
    bee: u8,
}

fn top_level_keys(doc: &str) -> Vec<String> {
    let v: serde_json::Value = serde_json::from_str(doc).expect("the document is JSON");
    v.as_object()
        .expect("the document is an object")
        .keys()
        .cloned()
        .collect()
}

#[test]
fn keys_are_sorted_at_every_depth_whatever_the_struct_order() {
    let doc = read_document(
        &Unsorted {
            zebra: 1,
            mango: Inner { yak: 2, bee: 3 },
            apple: 4,
        },
        Versioning::Stamp,
    )
    .unwrap();
    assert_eq!(
        top_level_keys(&doc),
        ["apple", "mango", "schemaVersion", "zebra"]
    );
    // serde_json's parser sorts too, so read the bytes for the nested order.
    assert!(
        doc.find("\"bee\"").unwrap() < doc.find("\"yak\"").unwrap(),
        "{doc}"
    );
    assert!(
        doc.find("\"apple\"").unwrap() < doc.find("\"zebra\"").unwrap(),
        "{doc}"
    );
}

#[test]
fn a_stamped_document_carries_the_read_axis_version() {
    let doc = read_document(&Inner { yak: 1, bee: 2 }, Versioning::Stamp).unwrap();
    let v: serde_json::Value = serde_json::from_str(&doc).unwrap();
    assert_eq!(v["schemaVersion"], READ_SCHEMA_VERSION);
    assert_eq!(
        READ_SCHEMA_VERSION, "0.1.0",
        "the axis starts at 0.1.0 (§3.4)"
    );
}

#[test]
fn an_array_is_wrapped_under_items_in_order() {
    let doc = read_document(&["c", "a", "b"], Versioning::Stamp).unwrap();
    let v: serde_json::Value = serde_json::from_str(&doc).unwrap();
    assert_eq!(v["items"], serde_json::json!(["c", "a", "b"]), "order kept");
    assert_eq!(top_level_keys(&doc), ["items", "schemaVersion"]);
    // An empty array is still a document, not a refusal.
    let empty: Vec<String> = Vec::new();
    let doc = read_document(&empty, Versioning::Stamp).unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&doc).unwrap()["items"],
        serde_json::json!([])
    );
}

#[test]
fn the_layout_is_canonical() {
    let doc = read_document(&Inner { yak: 1, bee: 2 }, Versioning::Stamp).unwrap();
    assert!(
        doc.ends_with("}\n") && !doc.ends_with("\n\n"),
        "one trailing newline"
    );
    assert!(!doc.contains('\r'), "LF only");
    assert!(doc.contains("\n  \"bee\": 2"), "two-space indent: {doc}");
}

#[test]
fn a_bare_null_is_refused_because_the_caller_names_an_absent_answer() {
    let err = read_document(&serde_json::Value::Null, Versioning::Stamp).unwrap_err();
    assert!(matches!(err, Error::Schema(_)), "{err:?}");
    assert!(err.to_string().contains("null"), "{err}");
    // The caller's shape for the same answer is accepted.
    let doc = read_document(&serde_json::json!({ "next": null }), Versioning::Stamp).unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&doc).unwrap()["next"],
        serde_json::Value::Null
    );
}

#[test]
fn a_scalar_is_refused() {
    for scalar in [
        serde_json::json!(1),
        serde_json::json!("x"),
        serde_json::json!(true),
    ] {
        let err = read_document(&scalar, Versioning::Stamp).unwrap_err();
        assert!(matches!(err, Error::Schema(_)), "{scalar}: {err:?}");
    }
}

#[test]
fn a_preexisting_version_is_kept_and_nothing_is_stamped() {
    let doc = read_document(
        &serde_json::json!({ "zeta": 1, "config_version": "0.1.0" }),
        Versioning::Preexisting("config_version"),
    )
    .unwrap();
    assert_eq!(top_level_keys(&doc), ["config_version", "zeta"]);
}

#[test]
fn a_preexisting_mode_without_its_member_is_refused() {
    // The exemption cannot be claimed by a document with nothing to exempt, and
    // the mode is never inferred from a member that merely looks like a version.
    let err = read_document(
        &serde_json::json!({ "version": "1" }),
        Versioning::Preexisting("config_version"),
    )
    .unwrap_err();
    assert!(matches!(err, Error::Schema(_)), "{err:?}");
    assert!(err.to_string().contains("config_version"), "{err}");
}

#[test]
fn a_member_named_like_a_version_does_not_exempt_a_stamped_document() {
    let doc = read_document(&serde_json::json!({ "version": "9" }), Versioning::Stamp).unwrap();
    let v: serde_json::Value = serde_json::from_str(&doc).unwrap();
    assert_eq!(v["schemaVersion"], READ_SCHEMA_VERSION, "{doc}");
    assert_eq!(v["version"], "9");
}

#[test]
fn stamping_a_document_that_already_names_schema_version_is_refused() {
    // Overwriting would silently replace a version another contract put there.
    let err = read_document(
        &serde_json::json!({ "schemaVersion": "9.9.9" }),
        Versioning::Stamp,
    )
    .unwrap_err();
    assert!(matches!(err, Error::Schema(_)), "{err:?}");
    assert!(err.to_string().contains("schemaVersion"), "{err}");
    assert!(
        err.to_string()
            .contains("already carries `schemaVersion`; stamping"),
        "the message reads as one sentence: {err}"
    );
}
