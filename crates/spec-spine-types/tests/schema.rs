// Spec: specs/000-spec-spine-bootstrap/spec.md
//! The embedded schemas must be valid JSON and aligned with the version consts.

use spec_spine_types::{
    BUILD_META_SCHEMA, BUILD_META_SCHEMA_VERSION, REGISTRY_SCHEMA, REGISTRY_SCHEMA_VERSION,
    version::parse_semver,
};

#[test]
fn embedded_schemas_are_valid_json() {
    serde_json::from_str::<serde_json::Value>(REGISTRY_SCHEMA).expect("registry schema is JSON");
    serde_json::from_str::<serde_json::Value>(BUILD_META_SCHEMA)
        .expect("build-meta schema is JSON");
}

#[test]
fn schema_version_constants_are_semver() {
    assert!(parse_semver(REGISTRY_SCHEMA_VERSION).is_some());
    assert!(parse_semver(BUILD_META_SCHEMA_VERSION).is_some());
}
