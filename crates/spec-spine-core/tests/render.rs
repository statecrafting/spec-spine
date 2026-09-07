//! Golden tests for the spec 011 projections: the rendered markdown is a pure
//! function of `(config, index.json bytes)`: byte-exact, LF endings, trailing
//! newline. Fixtures go through `load_index` so the wire shapes are exercised
//! end to end.

use spec_spine_core::{load_index, orphans, render_markdown};
use spec_spine_types::Config;

/// Packages, mappings, orphans and diagnostics deliberately unsorted in the
/// fixture: the render order below is the projection's doing.
const FULL_FIXTURE: &str = r#"{
  "schemaVersion": "1.0.0",
  "build": {
    "indexerId": "spec-spine",
    "indexerVersion": "0.2.0",
    "repoRoot": ".",
    "contentHash": "cafe1234"
  },
  "packages": [
    {
      "name": "zeta",
      "path": "crates/zeta",
      "kind": "rust-lib",
      "version": "0.1.0",
      "specRef": "002-zeta"
    },
    { "name": "alpha", "path": "npm/alpha", "kind": "npm-package" }
  ],
  "traceability": {
    "mappings": [
      {
        "specId": "002-zeta",
        "specStatus": "approved",
        "implementingPaths": [{ "path": "crates/zeta", "source": "spec-edge" }]
      },
      { "specId": "001-alpha", "implementingPaths": [] }
    ],
    "orphanedSpecs": ["009-zzz", "003-orphan"],
    "untracedCode": ["npm/alpha"]
  },
  "diagnostics": {
    "warnings": [
      { "code": "I-002", "message": "untraced package", "path": "npm/alpha" }
    ],
    "errors": [
      {
        "code": "I-003",
        "message": "unit resolved nowhere",
        "path": "crates/zeta/src/gone.rs"
      }
    ]
  }
}"#;

const FULL_EXPECTED: &str = "# spec-spine codebase index\n\
\n\
- schemaVersion: 1.0.0\n\
- contentHash: cafe1234\n\
\n\
## Packages\n\
\n\
| name | path | kind | version | spec |\n\
|---|---|---|---|---|\n\
| alpha | npm/alpha | npm-package | - | - |\n\
| zeta | crates/zeta | rust-lib | 0.1.0 | 002-zeta |\n\
\n\
## Traceability\n\
\n\
| spec | status | paths | units |\n\
|---|---|---|---|\n\
| 001-alpha | - | 0 | 0 |\n\
| 002-zeta | approved | 1 | 0 |\n\
\n\
### Orphaned specs\n\
\n\
- 003-orphan\n\
- 009-zzz\n\
\n\
### Untraced code\n\
\n\
- npm/alpha\n\
\n\
## Diagnostics\n\
\n\
- I-002 [warning] untraced package (npm/alpha)\n\
- I-003 [error] unit resolved nowhere (crates/zeta/src/gone.rs)\n";

/// No orphans, no untraced code, no diagnostics: those sections are omitted
/// entirely (spec 011 §3.2 / §3.4).
const EMPTY_SECTIONS_FIXTURE: &str = r#"{
  "schemaVersion": "1.0.0",
  "build": {
    "indexerId": "spec-spine",
    "indexerVersion": "0.2.0",
    "repoRoot": ".",
    "contentHash": "beef5678"
  },
  "packages": [
    { "name": "alpha", "path": "crates/alpha", "kind": "rust-bin" }
  ],
  "traceability": {
    "mappings": [
      {
        "specId": "001-alpha",
        "implementingPaths": [{ "path": "crates/alpha", "source": "spec-edge" }]
      }
    ],
    "orphanedSpecs": [],
    "untracedCode": []
  },
  "diagnostics": { "warnings": [], "errors": [] }
}"#;

const EMPTY_SECTIONS_EXPECTED: &str = "# spec-spine codebase index\n\
\n\
- schemaVersion: 1.0.0\n\
- contentHash: beef5678\n\
\n\
## Packages\n\
\n\
| name | path | kind | version | spec |\n\
|---|---|---|---|---|\n\
| alpha | crates/alpha | rust-bin | - | - |\n\
\n\
## Traceability\n\
\n\
| spec | status | paths | units |\n\
|---|---|---|---|\n\
| 001-alpha | - | 1 | 0 |\n";

#[test]
fn render_full_fixture_is_byte_exact() {
    let index = load_index(FULL_FIXTURE.as_bytes()).unwrap();
    assert_eq!(render_markdown(&Config::default(), &index), FULL_EXPECTED);
}

#[test]
fn render_omits_empty_sections() {
    let index = load_index(EMPTY_SECTIONS_FIXTURE.as_bytes()).unwrap();
    assert_eq!(
        render_markdown(&Config::default(), &index),
        EMPTY_SECTIONS_EXPECTED
    );
}

#[test]
fn orphans_are_id_sorted() {
    let index = load_index(FULL_FIXTURE.as_bytes()).unwrap();
    assert_eq!(orphans(&index), ["003-orphan", "009-zzz"]);

    let empty = load_index(EMPTY_SECTIONS_FIXTURE.as_bytes()).unwrap();
    assert!(orphans(&empty).is_empty());
}

// ── spec 059: orphans partitions by the in-flight predicate ───────────────

/// A registry carrying just the lifecycle fields the partition reads.
fn registry_with(specs: serde_json::Value) -> spec_spine_types::Registry {
    serde_json::from_value(serde_json::json!({
        "specVersion": "1.1.0",
        "build": {
            "compilerId": "t", "compilerVersion": "0.1.0",
            "inputRoot": ".", "contentHash": "t"
        },
        "specs": specs,
        "validation": { "passed": true, "violations": [] }
    }))
    .expect("registry json")
}

fn spec_record(id: &str, status: &str, implementation: Option<&str>) -> serde_json::Value {
    let mut v = serde_json::json!({
        "id": id,
        "title": "T",
        "status": status,
        "created": "2026-09-07",
        "summary": "s",
        "specPath": format!("specs/{id}/spec.md"),
    });
    if let Some(i) = implementation {
        v["implementation"] = serde_json::json!(i);
    }
    v
}

/// §3.1: the finding is a spec that is NOT in flight and still claims nothing
/// that resolves. Everything else is a specify-first corpus's normal state, and
/// is reported rather than filtered so the verb that answers "what has no code
/// yet" is not lost.
#[test]
fn orphans_partitions_by_the_in_flight_predicate() {
    let index = load_index(FULL_FIXTURE.as_bytes()).unwrap();
    // FULL_FIXTURE's orphans are 009-zzz and 003-orphan.
    let registry = registry_with(serde_json::json!([
        spec_record("003-orphan", "approved", Some("complete")),
        spec_record("009-zzz", "draft", Some("pending")),
    ]));
    let report = spec_spine_core::partition_orphans(&index, &registry.specs);

    assert_eq!(
        report.orphaned,
        vec!["003-orphan"],
        "approved + complete, claiming nothing that resolves: the finding"
    );
    assert_eq!(
        report.in_flight,
        vec!["009-zzz"],
        "draft + pending: normal, and still reported"
    );
    // §3.1: id-sorted within each group, as spec 011 §3.3 requires.
    let mut sorted = report.orphaned.clone();
    sorted.sort_unstable();
    assert_eq!(report.orphaned, sorted);
}

/// §3.1: every arm of spec 044's predicate, so the two verbs cannot disagree
/// about which specs are under way. `complete` settles it whatever the status.
#[test]
fn every_arm_of_the_in_flight_predicate_lands_where_044_puts_it() {
    let index = load_index(FULL_FIXTURE.as_bytes()).unwrap();
    let cases = [
        // (status, implementation, expected in-flight)
        ("draft", None, true),
        ("draft", Some("pending"), true),
        ("draft", Some("complete"), false),
        ("approved", Some("pending"), true),
        ("approved", Some("in-progress"), true),
        ("approved", Some("complete"), false),
        ("approved", None, false),
    ];
    for (status, implementation, want_in_flight) in cases {
        let registry = registry_with(serde_json::json!([
            spec_record("003-orphan", status, implementation),
            spec_record("009-zzz", "approved", Some("complete")),
        ]));
        let report = spec_spine_core::partition_orphans(&index, &registry.specs);
        let got = report.in_flight.contains(&"003-orphan");
        assert_eq!(
            got, want_in_flight,
            "status={status} implementation={implementation:?}"
        );
    }
}

/// §3.1: a spec with no registry record is treated as in flight. A corpus that
/// cannot say otherwise should not have its spec called abandoned.
#[test]
fn an_orphan_with_no_registry_record_is_in_flight() {
    let index = load_index(FULL_FIXTURE.as_bytes()).unwrap();
    let report =
        spec_spine_core::partition_orphans(&index, &registry_with(serde_json::json!([])).specs);
    assert!(report.orphaned.is_empty(), "{report:?}");
    assert_eq!(report.in_flight, vec!["003-orphan", "009-zzz"]);
}

/// §3.1: the flat list is still what the two groups partition, so no orphan is
/// gained or lost by the change.
#[test]
fn the_partition_covers_exactly_the_flat_list() {
    let index = load_index(FULL_FIXTURE.as_bytes()).unwrap();
    let registry = registry_with(serde_json::json!([
        spec_record("003-orphan", "approved", Some("complete")),
        spec_record("009-zzz", "draft", None),
    ]));
    let report = spec_spine_core::partition_orphans(&index, &registry.specs);
    let mut both: Vec<&str> = report
        .orphaned
        .iter()
        .chain(report.in_flight.iter())
        .copied()
        .collect();
    both.sort_unstable();
    assert_eq!(both, orphans(&index));
}
