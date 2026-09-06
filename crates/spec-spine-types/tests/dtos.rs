//! Registry DTO round-trips, version constants, severity/validation logic.

use spec_spine_types::{
    BUILD_META_SCHEMA_VERSION, CONFIG_VERSION, Error, INDEX_SCHEMA_VERSION,
    REGISTRY_SCHEMA_VERSION, Registry, Severity, Status, VERDICT_SCHEMA_VERSION, ValidationReport,
    Verdict, Violation, parse_semver, verdict::verb,
};

const REGISTRY_JSON: &str = r#"{
  "specVersion": "0.1.0",
  "build": {
    "compilerId": "spec-spine",
    "compilerVersion": "0.1.0",
    "inputRoot": ".",
    "contentHash": "0000000000000000000000000000000000000000000000000000000000000000"
  },
  "specs": [
    {
      "id": "000-x",
      "title": "T",
      "status": "approved",
      "created": "2026-06-08",
      "summary": "s",
      "specPath": "specs/000-x/spec.md",
      "coAuthority": [
        { "unit": { "kind": "section", "file": "Cargo.toml", "anchor": "deps" }, "with_specs": ["001-y"] }
      ],
      "origin": { "retroactive": true }
    }
  ],
  "validation": { "passed": true, "violations": [] }
}"#;

#[test]
fn registry_round_trips_camelcase() {
    let reg: Registry = serde_json::from_str(REGISTRY_JSON).unwrap();
    assert_eq!(reg.spec_version, "0.1.0");
    assert_eq!(reg.specs.len(), 1);
    let s = &reg.specs[0];
    assert_eq!(s.spec_path, "specs/000-x/spec.md");
    assert_eq!(s.status, Status::Approved);
    assert_eq!(s.co_authority.len(), 1);
    assert!(reg.validation.passed);

    // Serialize back; top-level keys are camelCase.
    let out = serde_json::to_string(&reg).unwrap();
    assert!(out.contains("\"specPath\""));
    assert!(out.contains("\"coAuthority\""));
    assert!(out.contains("\"contentHash\""));
}

#[test]
fn validation_passed_follows_error_tier() {
    let warn = ValidationReport::from_violations(vec![Violation {
        code: "L-001".into(),
        severity: Severity::Warning,
        message: "w".into(),
        path: None,
    }]);
    assert!(warn.passed, "warnings alone do not fail validation");

    let err = ValidationReport::from_violations(vec![Violation {
        code: "V-001".into(),
        severity: Severity::Error,
        message: "e".into(),
        path: None,
    }]);
    assert!(!err.passed, "any error-tier violation fails validation");
}

#[test]
fn schema_versions_are_pinned() {
    // 1.0.0: MAJOR, sharded registry (spec 024); 1.1.0: additive MINOR (spec
    // 028), optional `references` provenance `derived_at` timestamp.
    assert_eq!(REGISTRY_SCHEMA_VERSION, "1.1.0");
    // 1.1.0: additive MINOR (spec 025): unresolved-unit severity tiers (W-001 /
    // W-002 warnings) on top of the spec-024 sharded MAJOR.
    assert_eq!(INDEX_SCHEMA_VERSION, "1.1.0");
    assert_eq!(BUILD_META_SCHEMA_VERSION, "0.1.0");
    assert_eq!(CONFIG_VERSION, "0.1.0");
    // Spec 037: the verdict envelope, versioned from its first release rather
    // than acquiring a version after the first consumer breaks.
    assert_eq!(VERDICT_SCHEMA_VERSION, "0.1.0");
}

/// Spec 037 3.1: the envelope a consumer parses is one shape across six verbs.
/// Pinned here rather than only in the CLI tests, because the members and their
/// spelling are the external contract, not an implementation detail.
#[test]
fn verdict_envelope_round_trips_with_the_documented_members() {
    let v = Verdict::report(verb::COUPLE, 1, serde_json::json!({ "violations": [] }));
    let json = v.to_canonical_json().unwrap();
    let back: Verdict = serde_json::from_str(&json).unwrap();
    assert_eq!(back, v);

    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["schemaVersion"], VERDICT_SCHEMA_VERSION);
    assert_eq!(value["verb"], "couple");
    assert_eq!(value["ok"], false);
    assert_eq!(value["exitCode"], 1);
    assert!(value.get("report").is_some());
    assert!(value.get("error").is_none());
}

/// Spec 037 3.3: `kind` is a closed set of stable tokens, and the exit code in
/// the envelope is the one the process returns.
#[test]
fn verdict_failure_kinds_are_the_documented_tokens() {
    let v = Verdict::failure(
        verb::INDEX_CHECK,
        &Error::Stale {
            expected: "a".to_string(),
            actual: "b".to_string(),
        },
    );
    let value: serde_json::Value = serde_json::from_str(&v.to_canonical_json().unwrap()).unwrap();
    assert_eq!(value["error"]["kind"], "stale");
    assert_eq!(value["exitCode"], 2);
    assert!(value.get("report").is_none());
}

#[test]
fn parse_semver_works() {
    assert_eq!(parse_semver("0.1.0"), Some((0, 1, 0)));
    assert_eq!(parse_semver("2.13.4"), Some((2, 13, 4)));
    assert_eq!(parse_semver("0.1"), None);
    assert_eq!(parse_semver("0.1.0.0"), None);
    assert_eq!(parse_semver("x.y.z"), None);
}
