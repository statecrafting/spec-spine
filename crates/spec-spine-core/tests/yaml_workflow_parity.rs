//! Spec 196 §3.4: a GitHub Actions workflow is read exactly as before the
//! parser moved. Two halves:
//!
//! - every workflow in `.github/workflows/` parses (so it folds as its
//!   projection, never as raw bytes), and its digest is the one the committed
//!   inputs record holds, which the pre-196 reader wrote;
//! - fixtures for the hazards the spec names, with the projection and the
//!   dependency-only waiver's answer pinned as the pre-196 reader gave them
//!   (captured by running this file against serde_yaml 0.9.34 before it was
//!   removed).

use std::fs;
use std::path::{Path, PathBuf};

use spec_spine_core::manifest::workflow_hash_projection;
use spec_spine_core::workflow_dependency_only_change;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf()
}

#[test]
fn every_repository_workflow_folds_to_its_committed_digest() {
    let root = repo_root();
    let cfg =
        spec_spine_types::load_config(&fs::read_to_string(root.join("spec-spine.toml")).unwrap())
            .unwrap();
    let record: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join(".statecraft/derived/codebase-index/inputs.json")).unwrap(),
    )
    .unwrap();
    let digests = spec_spine_core::shard::input_digests(&cfg, &root);
    let mut seen = 0;
    for entry in fs::read_dir(root.join(".github/workflows")).unwrap() {
        let path = entry.unwrap().path();
        let rel = format!(
            ".github/workflows/{}",
            path.file_name().unwrap().to_string_lossy()
        );
        if !rel.ends_with(".yml") && !rel.ends_with(".yaml") {
            continue;
        }
        let text = fs::read_to_string(&path).unwrap();
        assert!(
            workflow_hash_projection(&text).is_some(),
            "{rel} parses, so it folds as its projection rather than raw bytes"
        );
        let committed = record["inputs"][&rel]["contentHash"]
            .as_str()
            .unwrap_or_else(|| panic!("{rel} is in the committed inputs record"));
        assert_eq!(
            digests.get(&rel).map(String::as_str),
            Some(committed),
            "{rel} folds to the digest the committed record holds"
        );
        seen += 1;
    }
    assert!(seen >= 9, "the workflows were read: {seen}");
}

/// `(name, workflow, projection the pre-196 reader gave)`.
const PROJECTIONS: &[(&str, &str, &str)] = &[
    (
        "on as a scalar",
        "on: push\njobs: {}\n",
        r#"{"s:jobs":{},"s:on":"push"}"#,
    ),
    (
        "on as a sequence",
        "on: [push, pull_request]\njobs: {}\n",
        r#"{"s:jobs":{},"s:on":["push","pull_request"]}"#,
    ),
    (
        "on as a mapping",
        "on:\n  push:\n    branches: [main]\n  workflow_dispatch:\njobs: {}\n",
        r#"{"s:jobs":{},"s:on":{"s:push":{"s:branches":["main"]},"s:workflow_dispatch":null}}"#,
    ),
    (
        "true as a key",
        "true: push\njobs: {}\n",
        r#"{"s:jobs":{},"y:true":"push"}"#,
    ),
    (
        "uses refs",
        "jobs:\n  a:\n    steps:\n      - uses: actions/checkout@8f152de45cc393bb48ce5d89d36b731f54556e65 # v4.2.2\n      - uses: actions/setup-node@v4\n      - uses: owner/repo/path@main\n      - uses: ./local\n      - uses: docker://alpine:3\n",
        r#"{"s:jobs":{"s:a":{"s:steps":[{"s:uses":"actions/checkout@"},{"s:uses":"actions/setup-node@"},{"s:uses":"owner/repo/path@"},{"s:uses":"./local"},{"s:uses":"docker://alpine:3"}]}}}"#,
    ),
    (
        "anchors, aliases and a merge key",
        "defaults: &d\n  runs-on: ubuntu-latest\njobs:\n  a:\n    <<: *d\n    steps: []\n  b: *d\n",
        r#"{"s:defaults":{"s:runs-on":"ubuntu-latest"},"s:jobs":{"s:a":{"s:<<":{"s:runs-on":"ubuntu-latest"},"s:steps":[]},"s:b":{"s:runs-on":"ubuntu-latest"}}}"#,
    ),
    (
        "block scalars",
        "jobs:\n  a:\n    steps:\n      - run: |\n          one\n          two\n      - run: >\n          folded\n          line\n      - run: |-\n          strip\n      - run: >+\n          keep\n\n",
        r#"{"s:jobs":{"s:a":{"s:steps":[{"s:run":"one\ntwo\n"},{"s:run":"folded line\n"},{"s:run":"strip"},{"s:run":"keep\n\n"}]}}}"#,
    ),
    (
        "numbers and booleans",
        "jobs:\n  a:\n    timeout-minutes: 15\n    continue-on-error: false\n    strategy:\n      matrix:\n        v: [1.90, 2, yes]\n",
        r#"{"s:jobs":{"s:a":{"s:continue-on-error":false,"s:strategy":{"s:matrix":{"s:v":[1.9,2,"yes"]}},"s:timeout-minutes":15}}}"#,
    ),
];

#[test]
fn the_hazard_fixtures_project_as_before() {
    let mut failures = Vec::new();
    for (name, workflow, expected) in PROJECTIONS {
        let got = workflow_hash_projection(workflow).unwrap_or_else(|| "<raw>".into());
        if std::env::var_os("SPEC_SPINE_PRINT_PARITY").is_some() {
            println!("{name}\t{got}");
        }
        if got != *expected {
            failures.push(format!("{name}:\n  expected {expected}\n  got      {got}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// `(name, base, head, the waiver clears it)`.
const WAIVERS: &[(&str, &str, &str, bool)] = &[
    (
        "a SHA pin bump with its comment",
        "jobs:\n  a:\n    steps:\n      - uses: actions/checkout@8f152de45cc393bb48ce5d89d36b731f54556e65 # v4.2.2\n",
        "jobs:\n  a:\n    steps:\n      - uses: actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683 # v4.3.0\n",
        true,
    ),
    (
        "a tag bump",
        "jobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n",
        "jobs:\n  a:\n    steps:\n      - uses: actions/checkout@v5\n",
        true,
    ),
    (
        "a branch ref moved to a tag",
        "jobs:\n  a:\n    steps:\n      - uses: owner/action@main\n",
        "jobs:\n  a:\n    steps:\n      - uses: owner/action@v1\n",
        true,
    ),
    (
        "a changed action path",
        "jobs:\n  a:\n    steps:\n      - uses: actions/checkout@v4\n",
        "jobs:\n  a:\n    steps:\n      - uses: actions/checkoutx@v4\n",
        false,
    ),
    (
        "a changed run block",
        "jobs:\n  a:\n    steps:\n      - run: |\n          one\n",
        "jobs:\n  a:\n    steps:\n      - run: |\n          two\n",
        false,
    ),
    (
        "a bump under a merged anchor",
        "d: &d\n  steps:\n    - uses: a/b@v1\njobs:\n  x:\n    <<: *d\n",
        "d: &d\n  steps:\n    - uses: a/b@v2\njobs:\n  x:\n    <<: *d\n",
        true,
    ),
    (
        "on read as a string both sides",
        "on: push\njobs: {}\n",
        "on: pull_request\njobs: {}\n",
        false,
    ),
    ("keys reordered", "a: 1\nb: 2\n", "b: 2\na: 1\n", true),
];

#[test]
fn the_dependency_only_waiver_clears_the_same_changes() {
    for (name, base, head, cleared) in WAIVERS {
        let got = workflow_dependency_only_change(base, head);
        if std::env::var_os("SPEC_SPINE_PRINT_PARITY").is_some() {
            println!("{name}\t{got}");
        }
        assert_eq!(got, *cleared, "{name}");
    }
}
