//! Coupling-gate tests (spec 005): drift detection at file / section / symbol
//! granularity, the primary-owner clearance heuristic, waivers, the additive
//! bypass floor, amends-awareness + the FR-005 strict-expansion guard, and
//! supersedes authority transfer. Artifacts are built as JSON (exercising the
//! deserialize path too) and fed to the pure `couple_with`.

use serde_json::{Value, json};
use spec_spine_core::{DiffFile, DiffInput, Waiver, couple_with};
use spec_spine_types::{CodebaseIndex, Config, LineSpan, Registry};

fn index_from(mappings: Value) -> CodebaseIndex {
    index_with_packages(json!([]), mappings)
}

/// Like [`index_from`] but with a package inventory, which the spec 029
/// ownership ratchet needs (its universe is "source files inside a package").
fn index_with_packages(packages: Value, mappings: Value) -> CodebaseIndex {
    serde_json::from_value(json!({
        "schemaVersion": "0.1.0",
        "build": {
            "indexerId": "t", "indexerVersion": "0.1.0",
            "repoRoot": ".", "contentHash": "t"
        },
        "packages": packages,
        "traceability": {
            "mappings": mappings,
            "orphanedSpecs": [],
            "untracedCode": []
        },
        "diagnostics": { "warnings": [], "errors": [] }
    }))
    .expect("index json")
}

fn registry_from(specs: Value) -> Registry {
    serde_json::from_value(json!({
        "specVersion": "0.1.0",
        "build": {
            "compilerId": "t", "compilerVersion": "0.1.0",
            "inputRoot": ".", "contentHash": "t"
        },
        "specs": specs,
        "validation": { "passed": true, "violations": [] }
    }))
    .expect("registry json")
}

/// An empty registry (no supersedes edges) for tests that don't exercise transfer.
fn empty_registry() -> Registry {
    registry_from(json!([]))
}

fn file(path: &str, hunks: &[LineSpan]) -> DiffFile {
    DiffFile {
        path: path.to_string(),
        hunks: hunks.to_vec(),
        deleted: false,
    }
}

fn deleted(path: &str) -> DiffFile {
    DiffFile {
        path: path.to_string(),
        hunks: Vec::new(),
        deleted: true,
    }
}

fn diff(files: Vec<DiffFile>) -> DiffInput {
    DiffInput { files }
}

fn run(
    index: &CodebaseIndex,
    registry: &Registry,
    diff: &DiffInput,
) -> spec_spine_core::CoupleReport {
    couple_with(&Config::default(), registry, index, diff, None).unwrap()
}

// ── file granularity ──────────────────────────────────────────────────────

#[test]
fn file_drift_then_clearance() {
    let index = index_from(json!([{
        "specId": "001-a",
        "implementingPaths": [],
        "resolvedUnits": [{
            "unit": { "kind": "file", "path": "src/lib.rs" },
            "sourceField": "establishes",
            "ownership": true,
            "locations": [{ "file": "src/lib.rs" }]
        }]
    }]));
    let reg = empty_registry();

    // Code changed, owning spec not edited → drift.
    let drift = run(
        &index,
        &reg,
        &diff(vec![file("src/lib.rs", &[LineSpan::new(5, 8)])]),
    );
    assert!(drift.has_blocking_drift());
    assert_eq!(drift.violations[0].path.as_deref(), Some("src/lib.rs"));

    // Same change + the owning spec.md → cleared.
    let cleared = run(
        &index,
        &reg,
        &diff(vec![
            file("src/lib.rs", &[LineSpan::new(5, 8)]),
            file("specs/001-a/spec.md", &[]),
        ]),
    );
    assert!(!cleared.has_blocking_drift(), "{:?}", cleared.violations);
}

#[test]
fn whole_file_floor_via_implementing_path() {
    // A crate-level manifest claim (directory prefix) owns every file beneath it.
    let index = index_from(json!([{
        "specId": "001-a",
        "implementingPaths": [{ "path": "crates/x", "source": "manifest-metadata" }],
        "resolvedUnits": []
    }]));
    let reg = empty_registry();
    let drift = run(
        &index,
        &reg,
        &diff(vec![file(
            "crates/x/src/deep/mod.rs",
            &[LineSpan::new(1, 1)],
        )]),
    );
    assert!(drift.has_blocking_drift());
    assert!(drift.violations[0].message.contains("001-a"));
}

// ── section granularity ───────────────────────────────────────────────────

#[test]
fn section_granularity_distinguishes_owners() {
    // Two specs own disjoint sections of the same file. A hunk in B's section is
    // NOT cleared by editing A; span overlap selects the right owner.
    let index = index_from(json!([
        {
            "specId": "010-top",
            "implementingPaths": [],
            "resolvedUnits": [{
                "unit": { "kind": "section", "file": "Makefile", "anchor": "top" },
                "sourceField": "co_authority", "ownership": true,
                "locations": [{ "file": "Makefile", "span": { "startLine": 10, "endLine": 20 } }]
            }]
        },
        {
            "specId": "020-bot",
            "implementingPaths": [],
            "resolvedUnits": [{
                "unit": { "kind": "section", "file": "Makefile", "anchor": "bot" },
                "sourceField": "co_authority", "ownership": true,
                "locations": [{ "file": "Makefile", "span": { "startLine": 30, "endLine": 40 } }]
            }]
        }
    ]));
    let reg = empty_registry();

    // Hunk at lines 12-14 (B's section is 30-40) + edit to 020-bot → still drift,
    // because the hunk is in 010-top's section.
    let wrong = run(
        &index,
        &reg,
        &diff(vec![
            file("Makefile", &[LineSpan::new(12, 14)]),
            file("specs/020-bot/spec.md", &[]),
        ]),
    );
    assert!(wrong.has_blocking_drift());
    assert!(wrong.violations[0].message.contains("010-top"));
    assert!(!wrong.violations[0].message.contains("020-bot"));

    // Editing the correct section owner clears it.
    let right = run(
        &index,
        &reg,
        &diff(vec![
            file("Makefile", &[LineSpan::new(12, 14)]),
            file("specs/010-top/spec.md", &[]),
        ]),
    );
    assert!(!right.has_blocking_drift(), "{:?}", right.violations);
}

// ── symbol granularity ────────────────────────────────────────────────────

#[test]
fn symbol_granularity_drift_detection() {
    let index = index_from(json!([{
        "specId": "030-sym",
        "implementingPaths": [],
        "resolvedUnits": [{
            "unit": { "kind": "symbol", "id": "crate::foo" },
            "sourceField": "establishes", "ownership": true,
            "locations": [{ "file": "src/lib.rs", "span": { "startLine": 50, "endLine": 70 } }]
        }]
    }]));
    let reg = empty_registry();

    // Hunk inside the symbol span → drift naming the symbol's owner.
    let inside = run(
        &index,
        &reg,
        &diff(vec![file("src/lib.rs", &[LineSpan::new(55, 60)])]),
    );
    assert!(inside.has_blocking_drift());
    assert!(inside.violations[0].message.contains("030-sym"));

    // Hunk OUTSIDE the symbol span and no other owner → unclaimed → no drift.
    let outside = run(
        &index,
        &reg,
        &diff(vec![file("src/lib.rs", &[LineSpan::new(5, 8)])]),
    );
    assert!(!outside.has_blocking_drift(), "{:?}", outside.violations);
}

// ── bypass + waiver ───────────────────────────────────────────────────────

#[test]
fn bypass_floor_and_additive_config() {
    let index = index_from(json!([{
        "specId": "001-a",
        "implementingPaths": [{ "path": "src", "source": "manifest-metadata" }],
        "resolvedUnits": []
    }]));
    let reg = empty_registry();

    // docs/ is on the hardcoded floor → never a violation.
    let docs = run(&index, &reg, &diff(vec![file("docs/guide.md", &[])]));
    assert!(!docs.has_blocking_drift());
    assert_eq!(docs.checked_paths, 0);

    // An additive config entry exempts a real owned path.
    let mut cfg = Config::default();
    cfg.coupling
        .bypass_prefixes
        .push("src/generated/".to_string());
    let report = couple_with(
        &cfg,
        &reg,
        &index,
        &diff(vec![file("src/generated/api.rs", &[LineSpan::new(1, 1)])]),
        None,
    )
    .unwrap();
    assert!(!report.has_blocking_drift());
    assert_eq!(report.checked_paths, 0);
}

#[test]
fn waiver_suppresses_exit_but_retains_violations() {
    let index = index_from(json!([{
        "specId": "001-a",
        "implementingPaths": [],
        "resolvedUnits": [{
            "unit": { "kind": "file", "path": "src/lib.rs" },
            "sourceField": "establishes", "ownership": true,
            "locations": [{ "file": "src/lib.rs" }]
        }]
    }]));
    let reg = empty_registry();
    let waiver = Waiver {
        reason: "dependency refresh OPS-1".to_string(),
    };
    let report = couple_with(
        &Config::default(),
        &reg,
        &index,
        &diff(vec![file("src/lib.rs", &[LineSpan::new(1, 1)])]),
        Some(&waiver),
    )
    .unwrap();
    assert!(!report.has_blocking_drift(), "waiver clears the exit");
    assert_eq!(report.violations.len(), 1, "but the violation is retained");
    assert_eq!(report.waiver.as_deref(), Some("dependency refresh OPS-1"));
}

// ── amends-awareness + FR-005 strict-expansion guard ──────────────────────

#[test]
fn amends_strict_guard_never_enrols_unowned_spec_md() {
    // specs/100-x/spec.md has NO base owner. An amender 101 exists but is not in
    // the diff. The strict guard suppresses expansion → the path stays unclaimed
    // (no drift), so editing your own spec while an unrelated amender exists is
    // never a false failure.
    let index = index_from(json!([{
        "specId": "101-amender",
        "amends": ["100-x"],
        "implementingPaths": [],
        "resolvedUnits": []
    }]));
    let reg = empty_registry();
    let report = run(&index, &reg, &diff(vec![file("specs/100-x/spec.md", &[])]));
    assert!(!report.has_blocking_drift(), "{:?}", report.violations);
}

#[test]
fn amends_expands_owners_when_base_set_nonempty() {
    // specs/100-x/spec.md DOES have a base owner (200 claims it as a file unit).
    // Amender 101 then expands the owner set; editing the amender clears the path.
    let index = index_from(json!([
        {
            "specId": "200-claims-spec-md",
            "implementingPaths": [],
            "resolvedUnits": [{
                "unit": { "kind": "file", "path": "specs/100-x/spec.md" },
                "sourceField": "constrains", "ownership": true,
                "locations": [{ "file": "specs/100-x/spec.md" }]
            }]
        },
        {
            "specId": "101-amender",
            "amends": ["100-x"],
            "implementingPaths": [],
            "resolvedUnits": []
        }
    ]));
    let reg = empty_registry();

    // Neither base owner nor amender edited → drift listing both.
    let drift = run(&index, &reg, &diff(vec![file("specs/100-x/spec.md", &[])]));
    assert!(drift.has_blocking_drift());
    assert!(drift.violations[0].message.contains("200-claims-spec-md"));
    assert!(drift.violations[0].message.contains("101-amender"));

    // Editing the amender (not the base owner) clears it: amends-awareness.
    let cleared = run(
        &index,
        &reg,
        &diff(vec![
            file("specs/100-x/spec.md", &[]),
            file("specs/101-amender/spec.md", &[]),
        ]),
    );
    assert!(!cleared.has_blocking_drift(), "{:?}", cleared.violations);
}

// ── the configured corpus root (spec 033) ─────────────────────────────────

/// A repo whose corpus lives somewhere other than `specs/`.
fn contracts_config() -> Config {
    let mut cfg = Config::default();
    cfg.layout.specs_dir = "contracts".to_string();
    cfg
}

fn owns_lib_rs() -> CodebaseIndex {
    index_from(json!([{
        "specId": "001-a",
        "implementingPaths": [],
        "resolvedUnits": [{
            "unit": { "kind": "file", "path": "src/lib.rs" },
            "sourceField": "establishes",
            "ownership": true,
            "locations": [{ "file": "src/lib.rs" }]
        }]
    }]))
}

#[test]
fn custom_specs_dir_clears_drift_via_the_owning_spec() {
    // Before spec 033 the primary-owner heuristic looked for a literal
    // `specs/<id>/spec.md`, so under a non-default `layout.specs_dir` NO edit to
    // the owning spec could ever clear `C-001`: the path it searched for did not
    // exist in the repo. The gate was unusable for such an adopter.
    let index = owns_lib_rs();
    let reg = empty_registry();
    let cfg = contracts_config();

    let drift = couple_with(
        &cfg,
        &reg,
        &index,
        &diff(vec![file("src/lib.rs", &[LineSpan::new(5, 8)])]),
        None,
    )
    .unwrap();
    assert!(drift.has_blocking_drift());

    let cleared = couple_with(
        &cfg,
        &reg,
        &index,
        &diff(vec![
            file("src/lib.rs", &[LineSpan::new(5, 8)]),
            file("contracts/001-a/spec.md", &[]),
        ]),
        None,
    )
    .unwrap();
    assert!(!cleared.has_blocking_drift(), "{:?}", cleared.violations);
}

#[test]
fn default_specs_dir_does_not_clear_under_a_custom_root() {
    // The mirror image: `specs/` carries no authority once the corpus root is
    // configured elsewhere, so a file sitting there must not clear the gate.
    let cleared = couple_with(
        &contracts_config(),
        &empty_registry(),
        &owns_lib_rs(),
        &diff(vec![
            file("src/lib.rs", &[LineSpan::new(5, 8)]),
            file("specs/001-a/spec.md", &[]),
        ]),
        None,
    )
    .unwrap();
    assert!(cleared.has_blocking_drift(), "{:?}", cleared.violations);
}

#[test]
fn custom_specs_dir_keeps_amends_awareness() {
    // The second reader of the path shape: amends expansion parses
    // `<specs_dir>/<id>/spec.md` back into an id. Under a custom root it must
    // still recognize the amended spec, or the amender could not clear it.
    let index = index_from(json!([
        {
            "specId": "200-claims-spec-md",
            "implementingPaths": [],
            "resolvedUnits": [{
                "unit": { "kind": "file", "path": "contracts/100-x/spec.md" },
                "sourceField": "constrains", "ownership": true,
                "locations": [{ "file": "contracts/100-x/spec.md" }]
            }]
        },
        {
            "specId": "101-amender",
            "amends": ["100-x"],
            "implementingPaths": [],
            "resolvedUnits": []
        }
    ]));
    let reg = empty_registry();
    let cfg = contracts_config();

    let drift = couple_with(
        &cfg,
        &reg,
        &index,
        &diff(vec![file("contracts/100-x/spec.md", &[])]),
        None,
    )
    .unwrap();
    assert!(drift.has_blocking_drift());
    assert!(drift.violations[0].message.contains("101-amender"));

    let cleared = couple_with(
        &cfg,
        &reg,
        &index,
        &diff(vec![
            file("contracts/100-x/spec.md", &[]),
            file("contracts/101-amender/spec.md", &[]),
        ]),
        None,
    )
    .unwrap();
    assert!(!cleared.has_blocking_drift(), "{:?}", cleared.violations);
}

// ── supersedes authority transfer ─────────────────────────────────────────

#[test]
fn supersedes_transfers_authority_additively() {
    // P established the file; S supersedes P. S inherits authority; P keeps its
    // historical authority. Editing EITHER clears; editing neither drifts.
    let index = index_from(json!([{
        "specId": "040-pred",
        "implementingPaths": [],
        "resolvedUnits": [{
            "unit": { "kind": "file", "path": "src/old.rs" },
            "sourceField": "establishes", "ownership": true,
            "locations": [{ "file": "src/old.rs" }]
        }]
    }]));
    let reg = registry_from(json!([
        { "id": "040-pred", "title": "p", "status": "superseded",
          "created": "d", "summary": "s", "specPath": "specs/040-pred/spec.md" },
        { "id": "041-succ", "title": "s", "status": "approved",
          "created": "d", "summary": "s", "specPath": "specs/041-succ/spec.md",
          "supersedes": ["040-pred"] }
    ]));

    let change = || diff(vec![file("src/old.rs", &[LineSpan::new(1, 1)])]);

    // Neither edited → drift naming both predecessor and successor.
    let drift = run(&index, &reg, &change());
    assert!(drift.has_blocking_drift());
    assert!(drift.violations[0].message.contains("040-pred"));
    assert!(drift.violations[0].message.contains("041-succ"));

    // Editing the successor (which inherited authority) clears it.
    let via_succ = run(
        &index,
        &reg,
        &diff(vec![
            file("src/old.rs", &[LineSpan::new(1, 1)]),
            file("specs/041-succ/spec.md", &[]),
        ]),
    );
    assert!(!via_succ.has_blocking_drift(), "{:?}", via_succ.violations);
}

// ── spec 018: partial (unit-scoped) supersession ──────────────────────────

#[test]
fn partial_supersedes_scopes_transfer_to_the_named_unit() {
    // P (040) established two files. S (041) PARTIALLY supersedes P over old.rs
    // only: the index models that as a `supersedes` resolved unit on S. S thus
    // owns old.rs but NOT keep.rs; the predecessor keeps both (additive).
    let index = index_from(json!([
        {
            "specId": "040-pred",
            "implementingPaths": [],
            "resolvedUnits": [
                { "unit": { "kind": "file", "path": "src/old.rs" },
                  "sourceField": "establishes", "ownership": true,
                  "locations": [{ "file": "src/old.rs" }] },
                { "unit": { "kind": "file", "path": "src/keep.rs" },
                  "sourceField": "establishes", "ownership": true,
                  "locations": [{ "file": "src/keep.rs" }] }
            ]
        },
        {
            "specId": "041-succ",
            "implementingPaths": [],
            "resolvedUnits": [
                { "unit": { "kind": "file", "path": "src/old.rs" },
                  "sourceField": "supersedes", "ownership": true,
                  "locations": [{ "file": "src/old.rs" }] }
            ]
        }
    ]));
    let reg = registry_from(json!([
        { "id": "040-pred", "title": "p", "status": "superseded",
          "created": "d", "summary": "s", "specPath": "specs/040-pred/spec.md" },
        { "id": "041-succ", "title": "s", "status": "approved",
          "created": "d", "summary": "s", "specPath": "specs/041-succ/spec.md",
          "supersedes": [{ "spec": "040-pred", "scope": "partial",
                           "unit": { "kind": "file", "path": "src/old.rs" } }] }
    ]));

    // The superseded unit: editing the successor clears it (S owns old.rs).
    let old_via_succ = run(
        &index,
        &reg,
        &diff(vec![
            file("src/old.rs", &[LineSpan::new(1, 1)]),
            file("specs/041-succ/spec.md", &[]),
        ]),
    );
    assert!(
        !old_via_succ.has_blocking_drift(),
        "{:?}",
        old_via_succ.violations
    );

    // The OTHER unit (keep.rs): S is NOT an owner, so editing S does not clear it;
    // the partial transfer reached old.rs only. Only the predecessor clears it.
    let keep_via_succ = run(
        &index,
        &reg,
        &diff(vec![
            file("src/keep.rs", &[LineSpan::new(1, 1)]),
            file("specs/041-succ/spec.md", &[]),
        ]),
    );
    assert!(
        keep_via_succ.has_blocking_drift(),
        "partial transfer must not reach keep.rs"
    );
    assert!(keep_via_succ.violations[0].message.contains("040-pred"));
    assert!(!keep_via_succ.violations[0].message.contains("041-succ"));

    let keep_via_pred = run(
        &index,
        &reg,
        &diff(vec![
            file("src/keep.rs", &[LineSpan::new(1, 1)]),
            file("specs/040-pred/spec.md", &[]),
        ]),
    );
    assert!(
        !keep_via_pred.has_blocking_drift(),
        "{:?}",
        keep_via_pred.violations
    );
}

#[test]
fn partial_supersedes_without_unit_transfers_nothing() {
    // S (041) partially supersedes P (040) with only a note: a documentary
    // lifecycle marker (OAP spec 199's shape). It transfers no authority: there
    // is no `supersedes` resolved unit on S, and `build_superseders` skips the
    // partial item, so the predecessor alone owns the file.
    let index = index_from(json!([{
        "specId": "040-pred",
        "implementingPaths": [],
        "resolvedUnits": [{
            "unit": { "kind": "file", "path": "src/old.rs" },
            "sourceField": "establishes", "ownership": true,
            "locations": [{ "file": "src/old.rs" }]
        }]
    }]));
    let reg = registry_from(json!([
        { "id": "040-pred", "title": "p", "status": "approved",
          "created": "d", "summary": "s", "specPath": "specs/040-pred/spec.md" },
        { "id": "041-succ", "title": "s", "status": "approved",
          "created": "d", "summary": "s", "specPath": "specs/041-succ/spec.md",
          "supersedes": [{ "spec": "040-pred", "scope": "partial",
                           "note": "retires the read-time projection layer" }] }
    ]));

    let via_succ = run(
        &index,
        &reg,
        &diff(vec![
            file("src/old.rs", &[LineSpan::new(1, 1)]),
            file("specs/041-succ/spec.md", &[]),
        ]),
    );
    assert!(
        via_succ.has_blocking_drift(),
        "a unit-less partial supersedes transfers nothing"
    );
    assert!(via_succ.violations[0].message.contains("040-pred"));
    assert!(!via_succ.violations[0].message.contains("041-succ"));
}

// ── spec 008: explicit claims take precedence over bypass ─────────────────

#[test]
fn explicit_claim_overrides_the_floor() {
    // .github/ sits on the hardcoded floor, but 007-d claims the workflow
    // file explicitly -> evaluated: drifts alone, clears with the owner.
    let index = index_from(json!([{
        "specId": "007-d",
        "implementingPaths": [],
        "resolvedUnits": [{
            "unit": { "kind": "file", "path": ".github/workflows/release.yml" },
            "sourceField": "establishes", "ownership": true,
            "locations": [{ "file": ".github/workflows/release.yml" }]
        }]
    }]));
    let reg = empty_registry();

    let drift = run(
        &index,
        &reg,
        &diff(vec![file(".github/workflows/release.yml", &[])]),
    );
    assert!(drift.has_blocking_drift());
    assert_eq!(drift.checked_paths, 1);
    assert!(drift.violations[0].message.contains("007-d"));

    let cleared = run(
        &index,
        &reg,
        &diff(vec![
            file(".github/workflows/release.yml", &[]),
            file("specs/007-d/spec.md", &[]),
        ]),
    );
    assert!(!cleared.has_blocking_drift(), "{:?}", cleared.violations);

    // A sibling workflow nobody claims stays floor-bypassed.
    let sibling = run(
        &index,
        &reg,
        &diff(vec![file(".github/workflows/ci.yml", &[])]),
    );
    assert!(!sibling.has_blocking_drift());
    assert_eq!(sibling.checked_paths, 0);
}

#[test]
fn implicit_ownership_does_not_override_bypass() {
    // Spec 008 §3.2: manifest-floor / comment-header ownership (the
    // implementingPaths sources) keeps deferring to bypass.
    let index = index_from(json!([{
        "specId": "001-a",
        "implementingPaths": [
            { "path": ".github", "source": "manifest-metadata" },
            { "path": "docs/guide.md", "source": "comment-header" }
        ],
        "resolvedUnits": []
    }]));
    let reg = empty_registry();
    let report = run(
        &index,
        &reg,
        &diff(vec![
            file(".github/workflows/ci.yml", &[]),
            file("docs/guide.md", &[]),
        ]),
    );
    assert!(!report.has_blocking_drift());
    assert_eq!(report.checked_paths, 0, "implicit ownership stays bypassed");
}

#[test]
fn claim_overrides_adopter_bypass_for_exactly_the_claimed_file() {
    // Spec 008 §3.3: the rule overrides config additions too; the specific
    // intent (the claim) beats the broad one (the bypass pattern).
    let index = index_from(json!([{
        "specId": "002-docs",
        "implementingPaths": [],
        "resolvedUnits": [{
            "unit": { "kind": "file", "path": "crates/x/README.md" },
            "sourceField": "constrains", "ownership": true,
            "locations": [{ "file": "crates/x/README.md" }]
        }]
    }]));
    let reg = empty_registry();
    let mut cfg = Config::default();
    cfg.coupling
        .bypass_prefixes
        .push("**/README.md".to_string());

    let claimed = couple_with(
        &cfg,
        &reg,
        &index,
        &diff(vec![file("crates/x/README.md", &[])]),
        None,
    )
    .unwrap();
    assert!(claimed.has_blocking_drift());
    assert!(claimed.violations[0].message.contains("002-docs"));

    let sibling = couple_with(
        &cfg,
        &reg,
        &index,
        &diff(vec![file("crates/y/README.md", &[])]),
        None,
    )
    .unwrap();
    assert!(!sibling.has_blocking_drift());
    assert_eq!(sibling.checked_paths, 0, "unclaimed siblings stay bypassed");
}

#[test]
fn directory_form_claim_overrides_for_the_subtree() {
    let index = index_from(json!([{
        "specId": "008-py",
        "implementingPaths": [],
        "resolvedUnits": [{
            "unit": { "kind": "file", "path": "docs/runbooks/" },
            "sourceField": "establishes", "ownership": true,
            "locations": [{ "file": "docs/runbooks/" }]
        }]
    }]));
    let reg = empty_registry();

    let inside = run(
        &index,
        &reg,
        &diff(vec![file("docs/runbooks/restore.md", &[])]),
    );
    assert!(inside.has_blocking_drift());
    assert_eq!(inside.checked_paths, 1);

    let outside = run(&index, &reg, &diff(vec![file("docs/guide.md", &[])]));
    assert!(!outside.has_blocking_drift());
    assert_eq!(outside.checked_paths, 0);
}

#[test]
fn section_claim_under_floor_is_evaluated_with_span_semantics() {
    // A co_authority section unit on jobs.<name> of a floored workflow: the
    // path is evaluated; span overlap then decides ownership as usual.
    let index = index_from(json!([{
        "specId": "118-wf",
        "implementingPaths": [],
        "resolvedUnits": [{
            "unit": { "kind": "section", "file": ".github/workflows/release.yml", "anchor": "publish" },
            "sourceField": "co_authority", "ownership": true,
            "locations": [{ "file": ".github/workflows/release.yml", "span": { "startLine": 10, "endLine": 20 } }]
        }]
    }]));
    let reg = empty_registry();

    let inside = run(
        &index,
        &reg,
        &diff(vec![file(
            ".github/workflows/release.yml",
            &[LineSpan::new(12, 14)],
        )]),
    );
    assert!(inside.has_blocking_drift());
    assert!(inside.violations[0].message.contains("118-wf"));

    // A hunk outside the span: evaluated (not bypassed) but unowned -> clean.
    let outside = run(
        &index,
        &reg,
        &diff(vec![file(
            ".github/workflows/release.yml",
            &[LineSpan::new(30, 31)],
        )]),
    );
    assert!(!outside.has_blocking_drift(), "{:?}", outside.violations);
    assert_eq!(outside.checked_paths, 1, "evaluated, not bypassed");
}

#[test]
fn is_bypassed_path_is_claim_aware() {
    // The CLI's auto-waiver pre-filter must see the same path set the gate
    // checks (spec 005 §3.5 x spec 008).
    let index = index_from(json!([{
        "specId": "007-d",
        "implementingPaths": [],
        "resolvedUnits": [{
            "unit": { "kind": "file", "path": ".github/workflows/release.yml" },
            "sourceField": "establishes", "ownership": true,
            "locations": [{ "file": ".github/workflows/release.yml" }]
        }]
    }]));
    let cfg = Config::default();
    assert!(!spec_spine_core::is_bypassed_path(
        &cfg,
        &index,
        ".github/workflows/release.yml"
    ));
    assert!(spec_spine_core::is_bypassed_path(
        &cfg,
        &index,
        ".github/workflows/ci.yml"
    ));
    assert!(!spec_spine_core::is_bypassed_path(
        &cfg,
        &index,
        "src/lib.rs"
    ));
}

// ── spec 029: the ownership ratchet (C-002) ───────────────────────────────

/// `require_ownership = true`, everything else default.
fn ratchet_config() -> Config {
    let mut cfg = Config::default();
    cfg.coupling.require_ownership = true;
    cfg
}

fn run_with(
    cfg: &Config,
    index: &CodebaseIndex,
    diff: &DiffInput,
    waiver: Option<&Waiver>,
) -> spec_spine_core::CoupleReport {
    couple_with(cfg, &empty_registry(), index, diff, waiver).unwrap()
}

fn codes(report: &spec_spine_core::CoupleReport) -> Vec<(&str, &str)> {
    report
        .violations
        .iter()
        .map(|v| (v.path.as_deref().unwrap_or(""), v.code.as_str()))
        .collect()
}

/// One crate at `crates/x` whose manifest names `000-floor`. Spec `001-a`
/// claims `src/lib.rs` by file unit and `src/hdr.rs` by comment header; spec
/// `002-r` only *references* `src/ref.rs` (its location still lands in
/// `implementingPaths` as a spec-edge path, which must not count).
fn floored_index() -> CodebaseIndex {
    index_with_packages(
        json!([{ "name": "x", "path": "crates/x", "kind": "rust-lib", "specRef": "000-floor" }]),
        json!([
            {
                "specId": "000-floor",
                "implementingPaths": [{ "path": "crates/x", "source": "manifest-metadata" }],
                "resolvedUnits": []
            },
            {
                "specId": "001-a",
                "implementingPaths": [
                    { "path": "crates/x/src/hdr.rs", "source": "comment-header" },
                    { "path": "crates/x/src/lib.rs", "source": "spec-edge" }
                ],
                "resolvedUnits": [{
                    "unit": { "kind": "file", "path": "crates/x/src/lib.rs" },
                    "sourceField": "establishes",
                    "ownership": true,
                    "locations": [{ "file": "crates/x/src/lib.rs" }]
                }]
            },
            {
                "specId": "002-r",
                "implementingPaths": [{ "path": "crates/x/src/ref.rs", "source": "spec-edge" }],
                "resolvedUnits": [{
                    "unit": { "kind": "file", "path": "crates/x/src/ref.rs" },
                    "sourceField": "references",
                    "ownership": false,
                    "locations": [{ "file": "crates/x/src/ref.rs" }]
                }]
            }
        ]),
    )
}

/// The same crate with no manifest floor: an unclaimed file has no owner at all.
fn unfloored_index() -> CodebaseIndex {
    index_with_packages(
        json!([{ "name": "x", "path": "crates/x", "kind": "rust-lib" }]),
        json!([{
            "specId": "001-a",
            "implementingPaths": [{ "path": "crates/x/src/lib.rs", "source": "spec-edge" }],
            "resolvedUnits": [{
                "unit": { "kind": "file", "path": "crates/x/src/lib.rs" },
                "sourceField": "establishes",
                "ownership": true,
                "locations": [{ "file": "crates/x/src/lib.rs" }]
            }]
        }]),
    )
}

#[test]
fn ratchet_off_skips_an_unowned_path() {
    // The pre-032 contract, pinned: an unowned path is not a coupling concern.
    let report = run_with(
        &Config::default(),
        &unfloored_index(),
        &diff(vec![file("crates/x/src/stray.rs", &[LineSpan::new(1, 2)])]),
        None,
    );
    assert!(report.violations.is_empty(), "{:?}", report.violations);
    assert_eq!(report.checked_paths, 1, "walked, just not judged");
}

#[test]
fn ratchet_on_refuses_an_unowned_path() {
    let report = run_with(
        &ratchet_config(),
        &unfloored_index(),
        &diff(vec![file("crates/x/src/stray.rs", &[LineSpan::new(1, 2)])]),
        None,
    );
    assert_eq!(codes(&report), vec![("crates/x/src/stray.rs", "C-002")]);
    assert!(
        report.violations[0]
            .message
            .contains("is not claimed by any spec"),
        "{}",
        report.violations[0].message
    );
    assert!(report.has_blocking_drift());
}

#[test]
fn floor_only_path_is_c002_naming_the_floor() {
    // Owned by the manifest floor and nothing else. Without the ratchet this
    // is C-001 (the floor is an owner); with it, the floor is debt.
    let report = run_with(
        &ratchet_config(),
        &floored_index(),
        &diff(vec![file("crates/x/src/stray.rs", &[LineSpan::new(1, 2)])]),
        None,
    );
    assert_eq!(codes(&report), vec![("crates/x/src/stray.rs", "C-002")]);
    let msg = &report.violations[0].message;
    assert!(msg.contains("only the package floor of 000-floor"), "{msg}");

    // Editing the floor spec clears drift, never coverage: the ratchet is not a
    // clearance rule, it is a claim requirement.
    let report = run_with(
        &ratchet_config(),
        &floored_index(),
        &diff(vec![
            file("crates/x/src/stray.rs", &[LineSpan::new(1, 2)]),
            file("specs/000-floor/spec.md", &[]),
        ]),
        None,
    );
    assert_eq!(codes(&report), vec![("crates/x/src/stray.rs", "C-002")]);
}

#[test]
fn floor_only_path_without_the_ratchet_is_plain_c001() {
    let report = run_with(
        &Config::default(),
        &floored_index(),
        &diff(vec![file("crates/x/src/stray.rs", &[LineSpan::new(1, 2)])]),
        None,
    );
    assert_eq!(codes(&report), vec![("crates/x/src/stray.rs", "C-001")]);
}

#[test]
fn unit_claimed_path_is_c001_not_c002() {
    // A specific claim satisfies the ratchet; drift is judged as before.
    let report = run_with(
        &ratchet_config(),
        &floored_index(),
        &diff(vec![file("crates/x/src/lib.rs", &[LineSpan::new(1, 2)])]),
        None,
    );
    assert_eq!(codes(&report), vec![("crates/x/src/lib.rs", "C-001")]);
    let cleared = run_with(
        &ratchet_config(),
        &floored_index(),
        &diff(vec![
            file("crates/x/src/lib.rs", &[LineSpan::new(1, 2)]),
            file("specs/001-a/spec.md", &[]),
        ]),
        None,
    );
    assert!(cleared.violations.is_empty(), "{:?}", cleared.violations);
}

#[test]
fn header_claimed_path_is_specific() {
    let report = run_with(
        &ratchet_config(),
        &floored_index(),
        &diff(vec![
            file("crates/x/src/hdr.rs", &[LineSpan::new(1, 2)]),
            file("specs/001-a/spec.md", &[]),
        ]),
        None,
    );
    assert!(report.violations.is_empty(), "{:?}", report.violations);
}

#[test]
fn references_never_satisfy_the_ratchet() {
    // 002-r references ref.rs; that is context, not ownership. The file is
    // floor-only, and the message names the floor, not the referencing spec.
    let report = run_with(
        &ratchet_config(),
        &floored_index(),
        &diff(vec![
            file("crates/x/src/ref.rs", &[LineSpan::new(1, 2)]),
            file("specs/002-r/spec.md", &[]),
        ]),
        None,
    );
    assert_eq!(codes(&report), vec![("crates/x/src/ref.rs", "C-002")]);
    let msg = &report.violations[0].message;
    assert!(msg.contains("000-floor") && !msg.contains("002-r"), "{msg}");
}

#[test]
fn deleted_paths_are_never_c002() {
    // Removing unowned code is how coverage goes up.
    let report = run_with(
        &ratchet_config(),
        &unfloored_index(),
        &diff(vec![deleted("crates/x/src/stray.rs")]),
        None,
    );
    assert!(report.violations.is_empty(), "{:?}", report.violations);

    // A deleted floor-only file: no C-002 either. Drift (C-001) still applies
    // to it exactly as to any whole-file change of an owned path.
    let report = run_with(
        &ratchet_config(),
        &floored_index(),
        &diff(vec![deleted("crates/x/src/stray.rs")]),
        None,
    );
    assert_eq!(codes(&report), vec![("crates/x/src/stray.rs", "C-001")]);
}

#[test]
fn paths_outside_the_universe_never_raise_c002() {
    // Not source, not in a package, or pruned by resolver_exclusions: the
    // ratchet asks its question only where `index coverage` counts.
    let report = run_with(
        &ratchet_config(),
        &unfloored_index(),
        &diff(vec![
            file("spec-spine.toml", &[]),            // not a source file
            file("crates/x/notes.md", &[]),          // not a source file
            file("scripts/tool.py", &[]),            // outside every package
            file("crates/x/target/gen.rs", &[]),     // excluded directory
            file("crates/x/node_modules/a.js", &[]), // excluded directory
        ]),
        None,
    );
    assert!(report.violations.is_empty(), "{:?}", report.violations);
}

#[test]
fn bypassed_paths_never_raise_c002() {
    // The floor and adopter additions are filtered before the question is
    // asked, so coverage is asked only of paths the gate would judge.
    let mut cfg = ratchet_config();
    cfg.coupling
        .bypass_prefixes
        .push("crates/x/vendor/".to_string());
    let report = run_with(
        &cfg,
        &unfloored_index(),
        &diff(vec![
            file("crates/x/vendor/v.rs", &[LineSpan::new(1, 1)]),
            file(".derived/x.json", &[]),
        ]),
        None,
    );
    assert!(report.violations.is_empty(), "{:?}", report.violations);
    assert_eq!(report.checked_paths, 0);
}

#[test]
fn explicit_claim_under_bypass_is_c001_not_c002() {
    // Spec 008: an explicit unit claim beats the bypass set. Such a path has a
    // specific owner by construction, so it can only ever be C-001.
    let mut cfg = ratchet_config();
    cfg.coupling
        .bypass_prefixes
        .push("crates/x/vendor/".to_string());
    let index = index_with_packages(
        json!([{ "name": "x", "path": "crates/x", "kind": "rust-lib" }]),
        json!([{
            "specId": "001-a",
            "implementingPaths": [],
            "resolvedUnits": [{
                "unit": { "kind": "file", "path": "crates/x/vendor/v.rs" },
                "sourceField": "establishes",
                "ownership": true,
                "locations": [{ "file": "crates/x/vendor/v.rs" }]
            }]
        }]),
    );
    let report = run_with(
        &cfg,
        &index,
        &diff(vec![file("crates/x/vendor/v.rs", &[LineSpan::new(1, 1)])]),
        None,
    );
    assert_eq!(codes(&report), vec![("crates/x/vendor/v.rs", "C-001")]);
}

#[test]
fn waiver_clears_c002_exactly_as_c001() {
    let waiver = Waiver {
        reason: "vendored drop; spec to follow".to_string(),
    };
    let report = run_with(
        &ratchet_config(),
        &unfloored_index(),
        &diff(vec![file("crates/x/src/stray.rs", &[LineSpan::new(1, 2)])]),
        Some(&waiver),
    );
    assert_eq!(codes(&report), vec![("crates/x/src/stray.rs", "C-002")]);
    assert!(
        !report.has_blocking_drift(),
        "retained for review, not blocking"
    );
}

#[test]
fn one_diff_can_carry_both_codes_but_one_path_carries_one() {
    let report = run_with(
        &ratchet_config(),
        &floored_index(),
        &diff(vec![
            file("crates/x/src/stray.rs", &[LineSpan::new(1, 2)]),
            file("crates/x/src/lib.rs", &[LineSpan::new(1, 2)]),
        ]),
        None,
    );
    assert_eq!(
        codes(&report),
        vec![
            ("crates/x/src/lib.rs", "C-001"),
            ("crates/x/src/stray.rs", "C-002")
        ],
        "sorted by path; each path raises exactly one code"
    );
}

// ── spec 036: the declared state root is bypassed unconditionally ─────────

/// A changed file under `layout.state_dir` trips neither `C-001` nor `C-002`,
/// and the effect is not reachable through the spec 008 claim override: a claim
/// inside the root is a contradiction `lint` reports as `L-006`, not a
/// precedence question the gate resolves in either direction.
#[test]
fn a_declared_state_root_is_bypassed_and_a_claim_cannot_override_it() {
    // The state file is claimed as explicitly as a unit can be, which is what
    // would otherwise beat the entire bypass set under spec 008.
    let index = index_with_packages(
        json!([{ "name": "app", "path": "", "kind": "rust-lib", "specRef": "001-a" }]),
        json!([{
            "specId": "001-a",
            "implementingPaths": [],
            "resolvedUnits": [{
                "unit": { "kind": "file", "path": "state/journal.rs" },
                "sourceField": "establishes", "ownership": true,
                "locations": [{ "file": "state/journal.rs" }]
            }]
        }]),
    );
    let reg = empty_registry();

    // Undeclared: the claim is honoured and the change drifts.
    let undeclared = run(&index, &reg, &diff(vec![file("state/journal.rs", &[])]));
    assert!(undeclared.has_blocking_drift(), "control: the claim fires");
    assert_eq!(undeclared.checked_paths, 1);

    // Declared: bypassed before the claim is ever consulted.
    let mut cfg = Config::default();
    cfg.layout.state_dir = "state".to_string();
    cfg.coupling.require_ownership = true;
    let report = couple_with(
        &cfg,
        &reg,
        &index,
        &diff(vec![file("state/journal.rs", &[])]),
        None,
    )
    .unwrap();
    assert!(!report.has_blocking_drift(), "{:?}", report.violations);
    assert_eq!(report.checked_paths, 0, "the path is not even examined");

    // A sibling sharing the prefix is still governed: `state` is a root, not a
    // string prefix, so `stateful/` must not slip through with it.
    let sibling = couple_with(
        &cfg,
        &reg,
        &index,
        &diff(vec![file("stateful/journal.rs", &[])]),
        None,
    )
    .unwrap();
    assert_eq!(
        sibling.checked_paths, 1,
        "stateful/ is not under the `state` root"
    );
}

/// Spec 036 3.2: with `require_ownership` on, an unclaimed file under the root
/// is not `C-002` debt either. State is not source, so the ratchet has nothing
/// to say about it.
#[test]
fn the_ownership_ratchet_does_not_reach_into_the_state_root() {
    let index = index_with_packages(
        json!([{ "name": "app", "path": "", "kind": "rust-lib", "specRef": "001-a" }]),
        json!([{ "specId": "001-a", "implementingPaths": [], "resolvedUnits": [] }]),
    );
    let reg = empty_registry();
    let mut cfg = Config::default();
    cfg.coupling.require_ownership = true;

    // Control: with no state root declared, an unclaimed source file is C-002.
    let debt = couple_with(
        &cfg,
        &reg,
        &index,
        &diff(vec![file("state/journal.rs", &[])]),
        None,
    )
    .unwrap();
    assert!(
        debt.violations.iter().any(|v| v.code == "C-002"),
        "{debt:?}"
    );

    cfg.layout.state_dir = "state/".to_string();
    let declared = couple_with(
        &cfg,
        &reg,
        &index,
        &diff(vec![file("state/journal.rs", &[])]),
        None,
    )
    .unwrap();
    assert!(declared.violations.is_empty(), "{:?}", declared.violations);
    assert_eq!(declared.checked_paths, 0);
}

// ── spec 045: the owner set is carried as data, not only as prose ─────────

/// §3.1: every `C-001` carries `owners`, holding exactly the owner set its
/// message names, in the same sorted order. Before this spec the gate computed
/// that set and then destroyed it by formatting it into English, so an
/// orchestrator reading the spec 034 envelope had to regex a sentence back into
/// a list.
#[test]
fn c001_carries_its_owners_as_data() {
    let index = index_from(json!([
        {
            "specId": "004-codebase-index",
            "implementingPaths": [],
            "resolvedUnits": [{
                "unit": { "kind": "file", "path": "src/index.rs" },
                "sourceField": "establishes",
                "ownership": true,
                "locations": [{ "file": "src/index.rs" }]
            }]
        },
        {
            "specId": "001-compile-registry",
            "implementingPaths": [],
            "resolvedUnits": [{
                "unit": { "kind": "file", "path": "src/index.rs" },
                "sourceField": "co_authority",
                "ownership": true,
                "locations": [{ "file": "src/index.rs" }]
            }]
        }
    ]));

    let report = run(
        &index,
        &empty_registry(),
        &diff(vec![file("src/index.rs", &[LineSpan::new(5, 8)])]),
    );
    let v = &report.violations[0];
    assert_eq!(v.code, "C-001");
    assert_eq!(
        v.owners,
        vec![
            "001-compile-registry".to_string(),
            "004-codebase-index".to_string()
        ],
        "owners are the message's set, sorted"
    );
    // The prose and the data cannot disagree: the field is the same list the
    // sentence renders, in the same order.
    assert!(
        v.message
            .contains("001-compile-registry, 004-codebase-index"),
        "{}",
        v.message
    );
}

/// §3.1: `C-002` deliberately keeps an empty `owners`. It fires precisely when
/// no spec specifically claims the path, so there is no owner to name; the
/// floor specs its message reports are why the path is debt, not who owns it.
#[test]
fn c002_carries_no_owners() {
    let index = index_with_packages(
        json!([{ "name": "a", "path": ".", "kind": "rust-lib", "specId": "001-a" }]),
        json!([{
            "specId": "001-a",
            "implementingPaths": [],
            "resolvedUnits": []
        }]),
    );
    let report = run_with(
        &ratchet_config(),
        &index,
        &diff(vec![file("src/orphan.rs", &[])]),
        None,
    );
    let v = &report.violations[0];
    assert_eq!(v.code, "C-002");
    assert!(
        v.owners.is_empty(),
        "no owner exists to name: {:?}",
        v.owners
    );
}

/// §3.4: an empty `owners` is omitted from serialization, so every producer
/// other than `C-001` emits exactly the JSON it emitted before the field
/// existed. This is why no schema file and no schema version had to move.
#[test]
fn empty_owners_is_omitted_from_json() {
    let index = index_with_packages(
        json!([{ "name": "a", "path": ".", "kind": "rust-lib", "specId": "001-a" }]),
        json!([{ "specId": "001-a", "implementingPaths": [], "resolvedUnits": [] }]),
    );
    let report = run_with(
        &ratchet_config(),
        &index,
        &diff(vec![file("src/orphan.rs", &[])]),
        None,
    );
    let json = serde_json::to_string(&report).unwrap();
    assert!(
        !json.contains("\"owners\""),
        "an owner-less violation must serialize as it always did: {json}"
    );
}

// ── spec 078: the gate reads the declared governed scope ─────────────────

mod governed_scope {
    use std::fs;
    use std::path::Path;

    use spec_spine_core::shard::{self, BY_PACKAGE_DIR, BY_SPEC_DIR};
    use spec_spine_core::{
        DiffFile, DiffInput, compile, couple, index, index_dir, index_shard_files, registry_dir,
        registry_shard_files,
    };
    use spec_spine_types::{Config, load_config};

    fn write(root: &Path, rel: &str, content: &str) {
        let p = root.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, content).unwrap();
    }

    /// A crate claimed by `001-a`, root files outside every package, a claimed
    /// and an unclaimed `.github/` workflow, and `require_ownership` on with a
    /// scope naming them all.
    fn fixture(extra_units: &str) -> (tempfile::TempDir, Config) {
        let tmp = tempfile::tempdir().unwrap();
        let r = tmp.path();
        write(r, "Cargo.toml", "[workspace]\nmembers = [\"crate-a\"]\n");
        write(
            r,
            "crate-a/Cargo.toml",
            "[package]\nname = \"crate-a\"\nversion = \"0.1.0\"\n",
        );
        write(r, "crate-a/src/lib.rs", "pub fn a() {}\n");
        write(r, "scripts/run.sh", "echo hi\n");
        write(r, "scripts/owned.sh", "echo owned\n");
        write(r, ".github/workflows/ci.yml", "name: ci\n");
        write(r, ".github/workflows/claimed.yml", "name: claimed\n");
        write(
            r,
            "specs/001-a/spec.md",
            &format!(
                "---\nid: \"001-a\"\ntitle: \"A\"\nstatus: approved\ncreated: \"2026-09-15\"\n\
                 summary: \"s\"\nestablishes:\n  - \"crate-a/src/lib.rs\"\n  - \"scripts/owned.sh\"\n\
                 {extra_units}---\n# a\n"
            ),
        );
        let toml = "[coupling]\nrequire_ownership = true\n\n[coverage]\n\
                    governed_scope = [\"scripts/*\", \".github/workflows/*\"]\n";
        write(r, "spec-spine.toml", toml);
        let cfg = load_config(toml).unwrap();
        let registry = compile(&cfg, r).unwrap();
        shard::sync_dir(
            &registry_dir(&cfg, r).join(BY_SPEC_DIR),
            &registry_shard_files(&registry.shards).unwrap(),
        )
        .unwrap();
        let outcome = index(&cfg, r).unwrap();
        let dir = index_dir(&cfg, r);
        let (by_spec, by_package) = index_shard_files(&outcome.shards).unwrap();
        shard::sync_dir(&dir.join(BY_SPEC_DIR), &by_spec).unwrap();
        shard::sync_dir(&dir.join(BY_PACKAGE_DIR), &by_package).unwrap();
        (tmp, cfg)
    }

    fn changed(paths: &[&str]) -> DiffInput {
        DiffInput {
            files: paths
                .iter()
                .map(|p| DiffFile {
                    path: p.to_string(),
                    hunks: Vec::new(),
                    deleted: false,
                })
                .collect(),
        }
    }

    fn codes(cfg: &Config, root: &Path, paths: &[&str]) -> Vec<(String, String)> {
        couple(cfg, root, &changed(paths), None)
            .unwrap()
            .violations
            .into_iter()
            .map(|v| (v.code, v.message))
            .collect()
    }

    /// A changed governed-scope file with no claim is `C-002`, naming no floor;
    /// the same kind of file with a claim is not.
    #[test]
    fn an_unclaimed_governed_scope_file_is_c002_and_a_claimed_one_is_not() {
        let (fx, cfg) = fixture("");
        let refused = codes(&cfg, fx.path(), &["scripts/run.sh"]);
        assert_eq!(refused.len(), 1, "{refused:?}");
        assert_eq!(refused[0].0, "C-002");
        assert!(
            refused[0].1.contains("is not claimed by any spec"),
            "{refused:?}"
        );
        assert!(
            !refused[0].1.contains("floor"),
            "no floor to name: {refused:?}"
        );

        // Claimed, and its spec edited in the same diff: nothing to refuse.
        let clean = codes(
            &cfg,
            fx.path(),
            &["scripts/owned.sh", "specs/001-a/spec.md"],
        );
        assert!(clean.is_empty(), "{clean:?}");
    }

    /// §3.3, D-2: an unclaimed file under a built-in bypass prefix stays
    /// bypassed although the scope names it; the same prefix with an explicit
    /// unit claim is governed exactly as spec 008 makes it (drift without its
    /// spec, clean with it).
    #[test]
    fn a_bypassed_path_stays_bypassed_unless_a_unit_claims_it() {
        let (fx, cfg) = fixture("  - \".github/workflows/claimed.yml\"\n");
        assert!(codes(&cfg, fx.path(), &[".github/workflows/ci.yml"]).is_empty());

        let drift = codes(&cfg, fx.path(), &[".github/workflows/claimed.yml"]);
        assert_eq!(drift.len(), 1, "{drift:?}");
        assert_eq!(drift[0].0, "C-001", "owned under 009, not C-002: {drift:?}");
        assert!(
            codes(
                &cfg,
                fx.path(),
                &[".github/workflows/claimed.yml", "specs/001-a/spec.md"]
            )
            .is_empty()
        );
    }

    /// §3.1: without the scope the same unclaimed script is outside the universe,
    /// so the gate's answer is the one it gave before this spec.
    #[test]
    fn without_a_scope_the_script_is_not_asked_about() {
        let (fx, _) = fixture("");
        let cfg = load_config("[coupling]\nrequire_ownership = true\n").unwrap();
        fs::write(
            fx.path().join("spec-spine.toml"),
            "[coupling]\nrequire_ownership = true\n",
        )
        .unwrap();
        // The config edit restales the index; regenerate so the gate answers.
        let outcome = index(&cfg, fx.path()).unwrap();
        let dir = index_dir(&cfg, fx.path());
        let (by_spec, by_package) = index_shard_files(&outcome.shards).unwrap();
        shard::sync_dir(&dir.join(BY_SPEC_DIR), &by_spec).unwrap();
        shard::sync_dir(&dir.join(BY_PACKAGE_DIR), &by_package).unwrap();
        assert!(codes(&cfg, fx.path(), &["scripts/run.sh"]).is_empty());
    }
}

// ── spec 092 §3.8: the CONFIGURED derived root is bypassed ────────────────

/// A configuration whose derived tree is not at the default path.
fn relocated_derived() -> Config {
    let mut cfg = Config::default();
    cfg.layout.derived_dir = ".statecraft/derived".to_string();
    cfg
}

/// The built-in floor spells `.derived/`. A repository that configures another
/// derived root has every regenerated shard judged as source: `C-001` drift
/// against a spec that says nothing about shard bytes, and `C-002` on top of it
/// where the ratchet is on. The gate reads the configuration instead.
///
/// Both directions in one test, because the interesting half is the contrast:
/// under the default configuration the relocated path is NOT bypassed (it is
/// just an ordinary unclaimed file), and under the relocated configuration it
/// is. A single-configuration assertion would pass against a gate that bypassed
/// `.statecraft/derived/` unconditionally.
#[test]
fn the_configured_derived_root_is_bypassed_and_the_default_is_not_a_synonym() {
    let index = index_from(json!([{
        "specId": "001-a",
        "implementingPaths": [],
        "resolvedUnits": [{
            "unit": { "kind": "file", "path": "src/lib.rs" },
            "sourceField": "establishes",
            "ownership": true,
            "locations": [{ "file": "src/lib.rs" }]
        }]
    }]));

    let shard = ".statecraft/derived/spec-registry/by-spec/001-a.json";
    let old_shard = ".derived/spec-registry/by-spec/001-a.json";

    assert!(
        spec_spine_core::couple::is_bypassed_path(&relocated_derived(), &index, shard),
        "the configured derived root must be bypassed"
    );
    assert!(
        !spec_spine_core::couple::is_bypassed_path(&Config::default(), &index, shard),
        "and it is bypassed BECAUSE it is configured, not because of its name"
    );
    // The default keeps working, unchanged, and keeps working in a repository
    // that has moved: a tree mid-migration has both paths answered.
    assert!(spec_spine_core::couple::is_bypassed_path(
        &Config::default(),
        &index,
        old_shard
    ));
    assert!(spec_spine_core::couple::is_bypassed_path(
        &relocated_derived(),
        &index,
        old_shard
    ));
    // Separator-aware: a sibling that merely shares the prefix is not derived.
    assert!(!spec_spine_core::couple::is_bypassed_path(
        &relocated_derived(),
        &index,
        ".statecraft/derived-backup/x.json"
    ));
    // And the rest of `.statecraft/` is ordinary governed territory.
    assert!(!spec_spine_core::couple::is_bypassed_path(
        &relocated_derived(),
        &index,
        ".statecraft/AGENTS.md"
    ));
}

/// End to end through the gate: a change that touches only the relocated shard
/// tree clears, and the same change under a configuration that does not declare
/// that root does not.
#[test]
fn a_regenerated_shard_at_the_configured_derived_root_is_not_drift() {
    let index = index_from(json!([{
        "specId": "001-a",
        "implementingPaths": [],
        "resolvedUnits": [{
            "unit": { "kind": "file", "path": "src/lib.rs" },
            "sourceField": "establishes",
            "ownership": true,
            "locations": [{ "file": "src/lib.rs" }]
        }]
    }]));
    let reg = empty_registry();
    let changed = diff(vec![file(
        ".statecraft/derived/spec-registry/by-spec/001-a.json",
        &[LineSpan::new(1, 3)],
    )]);

    let cleared = couple_with(&relocated_derived(), &reg, &index, &changed, None).unwrap();
    assert!(
        !cleared.has_blocking_drift(),
        "compiler output is not drift: {:?}",
        cleared.violations
    );
    assert_eq!(
        cleared.checked_paths, 0,
        "a bypassed path is not even checked: {cleared:?}"
    );

    let unconfigured = couple_with(&Config::default(), &reg, &index, &changed, None).unwrap();
    assert_eq!(
        unconfigured.checked_paths, 1,
        "without the configuration the same path is an ordinary changed file, \
         so this test's green is not a property of the path's name"
    );
}

/// The effective-bypass read (spec 015 D-3's consumer contract) reports the
/// configured root too, so a tool judging another repository sees the same set
/// the gate applies rather than a floor that is half a contract.
#[test]
fn the_effective_bypass_list_reports_the_configured_derived_root() {
    let entries = spec_spine_core::couple::effective_bypass_prefixes(&relocated_derived());
    let prefixes: Vec<&str> = entries.iter().map(|e| e.prefix.as_str()).collect();
    assert!(prefixes.contains(&".statecraft/derived/"), "{prefixes:?}");
    assert!(
        prefixes.contains(&".derived/"),
        "the floor is intact: {prefixes:?}"
    );

    // And the default configuration does not list it twice.
    let default = spec_spine_core::couple::effective_bypass_prefixes(&Config::default());
    assert_eq!(
        default.iter().filter(|e| e.prefix == ".derived/").count(),
        1,
        "{default:?}"
    );
}

// ── spec 100: a deleted path is judged where it lived ─────────────────────
//
// Behavioral assertions, not string searches. Each case builds the two
// snapshots explicitly and asserts the verdict the gate reaches, so a change
// that keeps the reason tokens and breaks the decision fails here.

/// The reproduced shape of spec 100 §1.1: spec `001-a` owns two files inside a
/// package whose manifest floor is `002-floor`.
fn prior_index_with_claim() -> CodebaseIndex {
    index_with_packages(
        json!([{ "name": "pkg", "path": "pkg", "kind": "rust-lib", "specRef": "002-floor" }]),
        json!([
            {
                "specId": "001-a",
                "implementingPaths": [],
                "resolvedUnits": [
                    {
                        "unit": { "kind": "file", "path": "pkg/src/doomed.rs" },
                        "sourceField": "establishes",
                        "ownership": true,
                        "locations": [{ "file": "pkg/src/doomed.rs" }]
                    },
                    {
                        "unit": { "kind": "file", "path": "pkg/src/kept.rs" },
                        "sourceField": "establishes",
                        "ownership": true,
                        "locations": [{ "file": "pkg/src/kept.rs" }]
                    }
                ]
            },
            {
                "specId": "002-floor",
                "implementingPaths": [{ "path": "pkg", "source": "manifest-metadata" }],
                "resolvedUnits": []
            }
        ]),
    )
}

/// The same corpus after the withdrawal: `001-a` no longer claims the file, so
/// only the package floor spells the vanished path.
fn head_index_without_claim() -> CodebaseIndex {
    index_with_packages(
        json!([{ "name": "pkg", "path": "pkg", "kind": "rust-lib", "specRef": "002-floor" }]),
        json!([
            {
                "specId": "001-a",
                "implementingPaths": [],
                "resolvedUnits": [{
                    "unit": { "kind": "file", "path": "pkg/src/kept.rs" },
                    "sourceField": "establishes",
                    "ownership": true,
                    "locations": [{ "file": "pkg/src/kept.rs" }]
                }]
            },
            {
                "specId": "002-floor",
                "implementingPaths": [{ "path": "pkg", "source": "manifest-metadata" }],
                "resolvedUnits": []
            }
        ]),
    )
}

fn with_prior(
    head: &CodebaseIndex,
    reg: &Registry,
    prior: &spec_spine_core::PriorSnapshots<'_>,
    d: &DiffInput,
) -> spec_spine_core::CoupleReport {
    spec_spine_core::couple_with_prior(
        &Config::default(),
        reg,
        head,
        &spec_spine_core::GovernedScope::empty(),
        prior,
        d,
        None,
    )
    .unwrap()
}

#[test]
fn deleted_path_resolves_owners_at_the_merge_base() {
    let reg = empty_registry();
    let head = head_index_without_claim();
    let change = diff(vec![
        deleted("pkg/src/doomed.rs"),
        file("specs/001-a/spec.md", &[]),
    ]);

    // Today's behavior, kept by the compatibility entry point: the head index
    // answers, the floor is the only spec left spelling the path, and the
    // correct removal is refused. This is the defect, asserted as present.
    let legacy = run(&head, &reg, &change);
    assert!(
        legacy.has_blocking_drift(),
        "the defect must still reproduce"
    );
    assert_eq!(legacy.violations[0].owners, vec!["002-floor".to_string()]);

    // Spec 100 §3.2: the merge base's snapshot answers, `001-a` is still an
    // owner there, and its spec.md is in the diff, so the removal clears.
    let snap = spec_spine_core::PriorOwnership::new(&reg, prior_index_with_claim());
    let prior = spec_spine_core::PriorSnapshots {
        merge_base: Some(&snap),
        ..Default::default()
    };
    let fixed = with_prior(&head, &reg, &prior, &change);
    assert!(
        !fixed.has_blocking_drift(),
        "a withdrawn claim must clear its own removal: {:?}",
        fixed.violations
    );
}

#[test]
fn deleted_path_with_surviving_co_owner_still_refuses() {
    // Two owners at the prior snapshot; the change edits neither spec.md. The
    // refusal must survive, and must name both.
    let reg = empty_registry();
    let prior_index = index_with_packages(
        json!([]),
        json!([
            {
                "specId": "001-a",
                "implementingPaths": [],
                "resolvedUnits": [{
                    "unit": { "kind": "file", "path": "src/shared.rs" },
                    "sourceField": "establishes",
                    "ownership": true,
                    "locations": [{ "file": "src/shared.rs" }]
                }]
            },
            {
                "specId": "003-b",
                "implementingPaths": [],
                "resolvedUnits": [{
                    "unit": { "kind": "file", "path": "src/shared.rs" },
                    "sourceField": "extends",
                    "ownership": true,
                    "locations": [{ "file": "src/shared.rs" }]
                }]
            }
        ]),
    );
    let snap = spec_spine_core::PriorOwnership::new(&reg, prior_index);
    let prior = spec_spine_core::PriorSnapshots {
        merge_base: Some(&snap),
        ..Default::default()
    };
    let head = index_from(json!([]));
    let r = with_prior(&head, &reg, &prior, &diff(vec![deleted("src/shared.rs")]));
    assert!(r.has_blocking_drift(), "a legitimate refusal must survive");
    assert_eq!(
        r.violations[0].owners,
        vec!["001-a".to_string(), "003-b".to_string()]
    );

    // One owner's spec.md clears it, exactly as spec 005 says.
    let cleared = with_prior(
        &head,
        &reg,
        &prior,
        &diff(vec![
            deleted("src/shared.rs"),
            file("specs/003-b/spec.md", &[]),
        ]),
    );
    assert!(!cleared.has_blocking_drift(), "{:?}", cleared.violations);
}

#[test]
fn deleted_path_floor_only_still_refuses() {
    // No spec specifically owned it at the prior snapshot; the manifest floor
    // did. Spec 100 §3.3 keeps that refusal.
    let reg = empty_registry();
    let prior_index = index_with_packages(
        json!([{ "name": "pkg", "path": "pkg", "kind": "rust-lib", "specRef": "002-floor" }]),
        json!([{
            "specId": "002-floor",
            "implementingPaths": [{ "path": "pkg", "source": "manifest-metadata" }],
            "resolvedUnits": []
        }]),
    );
    let snap = spec_spine_core::PriorOwnership::new(&reg, prior_index);
    let prior = spec_spine_core::PriorSnapshots {
        merge_base: Some(&snap),
        ..Default::default()
    };
    let head = index_from(json!([]));
    let r = with_prior(
        &head,
        &reg,
        &prior,
        &diff(vec![deleted("pkg/src/orphan.rs")]),
    );
    assert!(r.has_blocking_drift());
    assert_eq!(r.violations[0].owners, vec!["002-floor".to_string()]);
}

#[test]
fn deleted_path_explicit_claim_overrides_bypass_at_the_prior_snapshot() {
    // `docs/` is on the built-in bypass floor. A resolved unit claim lifts a
    // path out of it (spec 008). Spec 100 §3.3: for a deleted path that
    // override is read from the prior snapshot, so a removal of a claimed
    // doc is still examined even though the claim is gone at head.
    let reg = empty_registry();
    let claimed = index_from(json!([{
        "specId": "001-a",
        "implementingPaths": [],
        "resolvedUnits": [{
            "unit": { "kind": "file", "path": "docs/governed.md" },
            "sourceField": "establishes",
            "ownership": true,
            "locations": [{ "file": "docs/governed.md" }]
        }]
    }]));
    let head = index_from(json!([]));
    let change = diff(vec![deleted("docs/governed.md")]);

    // At head the claim is gone, so the bypass floor swallows the path and the
    // gate examines nothing. That is the state spec 100 corrects.
    let legacy = run(&head, &reg, &change);
    assert_eq!(legacy.checked_paths, 0, "bypassed at head");

    let snap = spec_spine_core::PriorOwnership::new(&reg, claimed);
    let prior = spec_spine_core::PriorSnapshots {
        merge_base: Some(&snap),
        ..Default::default()
    };
    let r = with_prior(&head, &reg, &prior, &change);
    assert_eq!(r.checked_paths, 1, "the prior claim lifts it out of bypass");
    assert!(r.has_blocking_drift());
    assert_eq!(r.violations[0].owners, vec!["001-a".to_string()]);
}

#[test]
fn deleted_path_supersedes_transfer_applies_at_the_prior_snapshot() {
    // `004-successor` supersedes `001-a`, so it inherits authority over the
    // deleted path and its spec.md clears the removal. The transfer is
    // computed from the PRIOR snapshot's registry, not the run's.
    let prior_registry = registry_from(json!([
        { "id": "001-a", "title": "a", "status": "approved",
          "created": "d", "summary": "s", "specPath": "specs/001-a/spec.md" },
        { "id": "004-successor", "title": "s", "status": "approved",
          "created": "d", "summary": "s", "specPath": "specs/004-successor/spec.md",
          "supersedes": ["001-a"] }
    ]));
    let snap = spec_spine_core::PriorOwnership::new(&prior_registry, prior_index_with_claim());
    let prior = spec_spine_core::PriorSnapshots {
        merge_base: Some(&snap),
        ..Default::default()
    };
    let head = head_index_without_claim();
    // The run's own registry knows nothing of the transfer: if the gate read
    // it instead of the snapshot's, this would refuse.
    let run_registry = empty_registry();
    let r = with_prior(
        &head,
        &run_registry,
        &prior,
        &diff(vec![
            deleted("pkg/src/doomed.rs"),
            file("specs/004-successor/spec.md", &[]),
        ]),
    );
    assert!(
        !r.has_blocking_drift(),
        "the successor inherited authority at the prior snapshot: {:?}",
        r.violations
    );
}

#[test]
fn deleted_path_absent_from_a_read_snapshot_has_no_owner() {
    // Spec 100 §3.5: a snapshot that WAS read and simply does not contain the
    // path answers "nobody owned it here". That is a computed answer, not a
    // fallback: the gate must not reach for head, and must record the absence.
    let reg = empty_registry();
    let snap = spec_spine_core::PriorOwnership::new(&reg, index_from(json!([])))
        .with_paths(["src/other.rs".to_string()].into_iter().collect());
    let prior = spec_spine_core::PriorSnapshots {
        merge_base: Some(&snap),
        ..Default::default()
    };
    // Head DOES claim the path. If the gate fell back to head, this would
    // refuse; the snapshot's answer is that nobody owned it there.
    let head = index_from(json!([{
        "specId": "009-late",
        "implementingPaths": [],
        "resolvedUnits": [{
            "unit": { "kind": "file", "path": "src/gone.rs" },
            "sourceField": "establishes",
            "ownership": true,
            "locations": [{ "file": "src/gone.rs" }]
        }]
    }]));
    let r = with_prior(&head, &reg, &prior, &diff(vec![deleted("src/gone.rs")]));
    assert!(!r.has_blocking_drift(), "{:?}", r.violations);
    assert_eq!(r.deletions.len(), 1);
    assert_eq!(
        r.deletions[0].snapshot,
        spec_spine_core::SNAPSHOT_MERGE_BASE
    );
    assert!(r.deletions[0].absent_at_snapshot);
}

#[test]
fn deletion_reports_which_snapshot_answered() {
    // Spec 100 §3.7: every deleted path the gate examined carries the snapshot
    // that resolved it, from the closed set.
    let reg = empty_registry();
    let base_snap = spec_spine_core::PriorOwnership::new(&reg, prior_index_with_claim());
    let head_snap = spec_spine_core::PriorOwnership::new(&reg, prior_index_with_claim());
    let prior = spec_spine_core::PriorSnapshots {
        merge_base: Some(&base_snap),
        head_commit: Some(&head_snap),
        worktree_deletions: ["pkg/src/kept.rs".to_string()].into_iter().collect(),
    };
    let head = head_index_without_claim();
    let r = with_prior(
        &head,
        &reg,
        &prior,
        &diff(vec![
            deleted("pkg/src/doomed.rs"),
            deleted("pkg/src/kept.rs"),
            file("specs/001-a/spec.md", &[]),
        ]),
    );
    let seen: Vec<(&str, &str)> = r
        .deletions
        .iter()
        .map(|d| (d.path.as_str(), d.snapshot.as_str()))
        .collect();
    assert_eq!(
        seen,
        vec![
            ("pkg/src/doomed.rs", spec_spine_core::SNAPSHOT_MERGE_BASE),
            ("pkg/src/kept.rs", spec_spine_core::SNAPSHOT_HEAD_COMMIT),
        ]
    );
}

#[test]
fn legacy_couple_with_is_byte_identical_and_reports_head_tree() {
    // Spec 100 §3.8: the compatibility entry points keep their behavior, and
    // the report says so rather than implying the two-snapshot guarantee.
    let reg = empty_registry();
    let head = head_index_without_claim();

    // No deletions: the new member is omitted, so the emitted bytes of every
    // input that produced a report before spec 100 are unmoved.
    let clean = run(&head, &reg, &diff(vec![file("pkg/src/kept.rs", &[])]));
    let json = serde_json::to_string(&clean).unwrap();
    assert!(!json.contains("deletions"), "{json}");

    // A deletion through the compatibility path is labelled honestly.
    let legacy = run(&head, &reg, &diff(vec![deleted("pkg/src/doomed.rs")]));
    assert_eq!(legacy.deletions.len(), 1);
    assert_eq!(
        legacy.deletions[0].snapshot,
        spec_spine_core::SNAPSHOT_HEAD_TREE
    );
    assert!(!legacy.deletions[0].absent_at_snapshot);

    // And it is exactly what `couple_with_prior` with no snapshots produces.
    let same = with_prior(
        &head,
        &reg,
        &spec_spine_core::PriorSnapshots::default(),
        &diff(vec![deleted("pkg/src/doomed.rs")]),
    );
    assert_eq!(legacy, same);
}

#[test]
fn modifications_and_additions_are_unaffected_by_a_prior_snapshot() {
    // Spec 100 §3.1: only deletions move. With a prior snapshot present that
    // would give a different answer, a modification and an addition must still
    // resolve at head.
    let reg = empty_registry();
    let base_snap = spec_spine_core::PriorOwnership::new(&reg, prior_index_with_claim());
    let prior = spec_spine_core::PriorSnapshots {
        merge_base: Some(&base_snap),
        ..Default::default()
    };
    let head = head_index_without_claim();

    // `pkg/src/doomed.rs` is owned by `001-a` at the prior snapshot and by the
    // floor at head. Modified, not deleted: the head answer must decide, so
    // editing `001-a`'s spec.md must NOT clear it.
    let modified = with_prior(
        &head,
        &reg,
        &prior,
        &diff(vec![
            file("pkg/src/doomed.rs", &[LineSpan::new(1, 1)]),
            file("specs/001-a/spec.md", &[]),
        ]),
    );
    assert!(
        modified.has_blocking_drift(),
        "a modification must resolve at head"
    );
    assert_eq!(modified.violations[0].owners, vec!["002-floor".to_string()]);
    assert!(modified.deletions.is_empty(), "no deletion, no provenance");

    // And the head answer clears when the head owner is edited.
    let cleared = with_prior(
        &head,
        &reg,
        &prior,
        &diff(vec![
            file("pkg/src/doomed.rs", &[LineSpan::new(1, 1)]),
            file("specs/002-floor/spec.md", &[]),
        ]),
    );
    assert!(!cleared.has_blocking_drift(), "{:?}", cleared.violations);
}
