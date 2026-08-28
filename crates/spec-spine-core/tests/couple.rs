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

/// Like [`index_from`] but with a package inventory, which the spec 032
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

// ── spec 019: partial (unit-scoped) supersession ──────────────────────────

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

// ── spec 009: explicit claims take precedence over bypass ─────────────────

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
    // Spec 009 §3.2: manifest-floor / comment-header ownership (the
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
    // Spec 009 §3.3: the rule overrides config additions too; the specific
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
    // checks (spec 005 §3.5 x spec 009).
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

// ── spec 032: the ownership ratchet (C-002) ───────────────────────────────

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
    // Spec 009: an explicit unit claim beats the bypass set. Such a path has a
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
