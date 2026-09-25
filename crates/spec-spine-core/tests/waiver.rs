// Spec: specs/113-a-waiver-has-a-declared-lifecycle/spec.md
//! A waiver's declared lifecycle (spec 113), through the pure gate: scope,
//! expiry, ancestry and a use limit over caller-supplied inputs, the pairing of
//! waivers with violations, and compatibility with every waiver written before.
//!
//! The artifacts are built as JSON and fed to `couple_with_prior_waived`, as
//! `couple.rs` does for spec 005, so nothing here depends on a repository.

use std::collections::BTreeSet;

use serde_json::{Value, json};
use spec_spine_core::{
    CheckOutcome, CoupleReport, DiffFile, DiffInput, GovernedScope, PriorSnapshots, Waiver,
    WaiverDeclaration, WaiverInputs, WaiverSet, couple_with, couple_with_prior_waived,
    parse_waiver, parse_waivers,
};
use spec_spine_types::{CodebaseIndex, Config, Registry};

/// Two files, each owned by its own spec, so one diff raises two `C-001`s.
fn index() -> CodebaseIndex {
    let owned = |spec: &str, path: &str| {
        json!({
            "specId": spec,
            "implementingPaths": [],
            "resolvedUnits": [{
                "unit": { "kind": "file", "path": path },
                "sourceField": "establishes",
                "ownership": true,
                "locations": [{ "file": path }]
            }]
        })
    };
    serde_json::from_value(json!({
        "schemaVersion": "0.1.0",
        "build": { "indexerId": "t", "indexerVersion": "0.1.0", "repoRoot": ".", "contentHash": "t" },
        "packages": [],
        "traceability": {
            "mappings": [owned("001-a", "src/a.rs"), owned("002-b", "src/b.rs")],
            "orphanedSpecs": [],
            "untracedCode": []
        },
        "diagnostics": { "warnings": [], "errors": [] }
    }))
    .unwrap()
}

fn registry() -> Registry {
    serde_json::from_value(json!({
        "specVersion": "0.1.0",
        "build": { "compilerId": "t", "compilerVersion": "0.1.0", "inputRoot": ".", "contentHash": "t" },
        "specs": [],
        "validation": { "passed": true, "violations": [] }
    }))
    .unwrap()
}

fn diff(paths: &[&str]) -> DiffInput {
    DiffInput {
        files: paths
            .iter()
            .map(|p| DiffFile {
                path: (*p).to_string(),
                hunks: Vec::new(),
                deleted: false,
            })
            .collect(),
    }
}

/// Both owned files changed, neither spec edited: `C-001` on each.
fn both() -> DiffInput {
    diff(&["src/a.rs", "src/b.rs"])
}

fn run_body(body: &str, inputs: &WaiverInputs, d: &DiffInput) -> CoupleReport {
    let cfg = Config::default();
    let set = parse_waivers(&cfg, body);
    run_set(&set, inputs, d)
}

fn run_set(set: &WaiverSet, inputs: &WaiverInputs, d: &DiffInput) -> CoupleReport {
    couple_with_prior_waived(
        &Config::default(),
        &registry(),
        &index(),
        &GovernedScope::empty(),
        &PriorSnapshots::default(),
        d,
        set,
        inputs,
    )
    .unwrap()
}

fn open_paths(r: &CoupleReport) -> Vec<String> {
    r.uncleared()
        .iter()
        .filter_map(|v| v.path.clone())
        .collect()
}

fn cleared(r: &CoupleReport, n: usize) -> Vec<String> {
    r.waivers[n]
        .clears
        .iter()
        .filter_map(|c| c.path.clone())
        .collect()
}

fn no_inputs() -> WaiverInputs {
    WaiverInputs::default()
}

fn as_of(d: &str) -> WaiverInputs {
    WaiverInputs {
        as_of: Some(d.into()),
        ..WaiverInputs::default()
    }
}

// ── §3.2 scope ──────────────────────────────────────────────────────────────

#[test]
fn a_scoped_waiver_clears_its_path_and_not_the_other() {
    let r = run_body(
        "Spec-Drift-Waiver: bump\nSpec-Drift-Waiver-Paths: src/a.rs\n",
        &no_inputs(),
        &both(),
    );
    assert_eq!(r.violations.len(), 2, "both violations are still reported");
    assert!(r.has_blocking_drift(), "the unlisted path still refuses");
    assert_eq!(open_paths(&r), vec!["src/b.rs"]);
    assert_eq!(cleared(&r, 0), vec!["src/a.rs"]);
    assert!(r.waivers[0].scoped);
    assert_eq!(r.waivers[0].paths, vec!["src/a.rs"]);
    assert_eq!(r.waiver, None, "a run left blocking is not a waived run");
    assert_eq!(r.exit_code(), 1);
}

#[test]
fn a_subtree_entry_scopes_everything_under_it() {
    let r = run_body(
        "Spec-Drift-Waiver: tree\nSpec-Drift-Waiver-Paths: src/\n",
        &no_inputs(),
        &both(),
    );
    assert!(!r.has_blocking_drift(), "{:?}", r.waivers);
    let r = run_body(
        "Spec-Drift-Waiver: other tree\nSpec-Drift-Waiver-Paths: lib/\n",
        &no_inputs(),
        &both(),
    );
    assert_eq!(open_paths(&r), vec!["src/a.rs", "src/b.rs"]);
}

#[test]
fn an_unscoped_waiver_clears_everything_and_says_it_is_unscoped() {
    let r = run_body("Spec-Drift-Waiver: hotfix OPS-9\n", &no_inputs(), &both());
    assert!(!r.has_blocking_drift());
    assert_eq!(r.waiver.as_deref(), Some("hotfix OPS-9"));
    assert!(!r.waivers[0].scoped);
    assert!(r.waivers[0].paths.is_empty());
    assert_eq!(cleared(&r, 0), vec!["src/a.rs", "src/b.rs"]);
}

// ── §3.3, §3.4 expiry ──────────────────────────────────────────────────────

const UNTIL: &str = "Spec-Drift-Waiver: bump\nSpec-Drift-Waiver-Until: 2026-12-31\n";

#[test]
fn an_expiry_on_or_before_its_date_clears() {
    for day in ["2026-09-23", "2026-12-31"] {
        let r = run_body(UNTIL, &as_of(day), &both());
        assert!(!r.has_blocking_drift(), "{day}: {:?}", r.waivers);
        let c = &r.waivers[0].checks[0];
        assert_eq!(
            (c.check.as_str(), c.outcome),
            ("expiry", CheckOutcome::Satisfied)
        );
        assert_eq!(c.input.as_deref(), Some(day));
    }
}

#[test]
fn an_expiry_past_its_date_refuses_and_is_named() {
    let r = run_body(UNTIL, &as_of("2027-01-01"), &both());
    assert!(r.has_blocking_drift());
    assert!(!r.waivers[0].effective);
    assert!(
        r.waivers[0].clears.is_empty(),
        "a failed waiver clears nothing"
    );
    let c = &r.waivers[0].checks[0];
    assert_eq!(
        (c.check.as_str(), c.outcome),
        ("expiry", CheckOutcome::Failed)
    );
    assert!(c.detail.as_deref().unwrap().contains("expired"), "{c:?}");
    assert_eq!(r.waiver, None);
}

/// §3.4, read from the verdict: the exit code is the same when satisfied and
/// when not evaluated, so a test that read only it would pass on a mechanism
/// that did nothing.
#[test]
fn an_expiry_with_no_as_of_is_not_evaluated_and_not_satisfied() {
    let r = run_body(UNTIL, &no_inputs(), &both());
    assert!(!r.has_blocking_drift(), "not evaluated does not refuse");
    let c = &r.waivers[0].checks[0];
    assert_eq!(c.outcome, CheckOutcome::NotEvaluated);
    assert_ne!(c.outcome, CheckOutcome::Satisfied);
    assert_eq!(c.input, None);
    assert!(r.waivers[0].effective);
    let v = serde_json::to_value(&r).unwrap();
    assert_eq!(v["waivers"][0]["checks"][0]["outcome"], "not-evaluated");
}

// ── §3.3 ancestry ──────────────────────────────────────────────────────────

const SINCE: &str = "Spec-Drift-Waiver: bump\nSpec-Drift-Waiver-Since: a3d5213d\n";

fn ancestry(answer: bool) -> WaiverInputs {
    let mut i = WaiverInputs::default();
    i.ancestry.insert("a3d5213d".into(), answer);
    i
}

#[test]
fn an_ancestry_answered_false_refuses_and_true_clears() {
    let r = run_body(SINCE, &ancestry(false), &both());
    assert!(r.has_blocking_drift());
    assert_eq!(r.waivers[0].checks[0].outcome, CheckOutcome::Failed);
    let r = run_body(SINCE, &ancestry(true), &both());
    assert!(!r.has_blocking_drift());
    assert_eq!(r.waivers[0].checks[0].outcome, CheckOutcome::Satisfied);
    let r = run_body(SINCE, &no_inputs(), &both());
    assert_eq!(r.waivers[0].checks[0].outcome, CheckOutcome::NotEvaluated);
}

// ── §3.3, §3.4 uses ────────────────────────────────────────────────────────

const MAX_ONE: &str = "Spec-Drift-Waiver: bump\nSpec-Drift-Waiver-Max-Uses: 2\n";

fn uses(n: u64) -> WaiverInputs {
    let id = parse_waivers(&Config::default(), MAX_ONE).declarations[0].id();
    let mut i = WaiverInputs::default();
    i.uses.insert(id, n);
    i
}

#[test]
fn a_use_count_at_the_maximum_refuses_and_below_it_clears() {
    let r = run_body(MAX_ONE, &uses(2), &both());
    assert!(r.has_blocking_drift());
    assert_eq!(r.waivers[0].checks[0].outcome, CheckOutcome::Failed);
    let r = run_body(MAX_ONE, &uses(1), &both());
    assert!(!r.has_blocking_drift());
    assert_eq!(r.waivers[0].checks[0].outcome, CheckOutcome::Satisfied);
}

#[test]
fn an_absent_use_count_is_not_evaluated_and_not_zero() {
    let r = run_body(MAX_ONE, &no_inputs(), &both());
    let c = &r.waivers[0].checks[0];
    assert_eq!(
        c.outcome,
        CheckOutcome::NotEvaluated,
        "zero would be satisfied"
    );
    assert_eq!(c.input, None);
    // A count keyed by another waiver's id is not this waiver's count.
    let mut other = WaiverInputs::default();
    other.uses.insert("sha256:not-this-one".into(), 0);
    let r = run_body(MAX_ONE, &other, &both());
    assert_eq!(r.waivers[0].checks[0].outcome, CheckOutcome::NotEvaluated);
}

// ── §3.1, §3.5 malformed and repeated declarations ─────────────────────────

#[test]
fn a_malformed_value_fails_its_waiver() {
    for (body, check) in [
        (
            "Spec-Drift-Waiver: x\nSpec-Drift-Waiver-Until: soon\n",
            "expiry",
        ),
        (
            "Spec-Drift-Waiver: x\nSpec-Drift-Waiver-Since: HEAD~1\n",
            "ancestry",
        ),
        (
            "Spec-Drift-Waiver: x\nSpec-Drift-Waiver-Max-Uses: 0\n",
            "uses",
        ),
    ] {
        let r = run_body(body, &as_of("2026-09-23"), &both());
        assert!(r.has_blocking_drift(), "{body}");
        let c = &r.waivers[0].checks[0];
        assert_eq!(
            (c.check.as_str(), c.outcome),
            (check, CheckOutcome::Failed),
            "{body}"
        );
        assert!(c.detail.is_some(), "{body}");
    }
}

#[test]
fn a_repeated_single_valued_key_fails_its_waiver() {
    let r = run_body(
        "Spec-Drift-Waiver: x\nSpec-Drift-Waiver-Until: 2026-12-31\nSpec-Drift-Waiver-Until: 2027-12-31\n",
        &as_of("2026-09-23"),
        &both(),
    );
    assert!(r.has_blocking_drift());
    assert_eq!(
        r.waivers[0].checks[0].detail.as_deref(),
        Some("declared more than once")
    );
}

// ── §3.6 pairing ───────────────────────────────────────────────────────────

/// Two waivers, two violations, and the pairing is not the positional one: the
/// first waiver covers the second violation and the second waiver the first.
#[test]
fn waivers_pair_with_violations_by_scope_not_position() {
    let r = run_body(
        "Spec-Drift-Waiver: for b\nSpec-Drift-Waiver-Paths: src/b.rs\n\
         Spec-Drift-Waiver: for a\nSpec-Drift-Waiver-Paths: src/a.rs\n",
        &no_inputs(),
        &both(),
    );
    assert!(!r.has_blocking_drift());
    assert_eq!(r.waivers[0].reason, "for b");
    assert_eq!(cleared(&r, 0), vec!["src/b.rs"]);
    assert_eq!(r.waivers[1].reason, "for a");
    assert_eq!(cleared(&r, 1), vec!["src/a.rs"]);
    assert_eq!(
        r.waiver.as_deref(),
        Some("for b"),
        "the first effective reason"
    );
}

#[test]
fn a_failed_waiver_leaves_its_path_to_the_next_one() {
    let r = run_body(
        "Spec-Drift-Waiver: expired\nSpec-Drift-Waiver-Until: 2020-01-01\n\
         Spec-Drift-Waiver: scoped\nSpec-Drift-Waiver-Paths: src/a.rs\n",
        &as_of("2026-09-23"),
        &both(),
    );
    assert!(r.waivers[0].clears.is_empty());
    assert_eq!(cleared(&r, 1), vec!["src/a.rs"]);
    assert_eq!(open_paths(&r), vec!["src/b.rs"]);
}

// ── §3.1 parsing ───────────────────────────────────────────────────────────

#[test]
fn lifecycle_lines_attach_to_the_waiver_above_them() {
    let cfg = Config::default();
    let set = parse_waivers(
        &cfg,
        "Spec-Drift-Waiver-Paths: early/\n\
         prose\n\
         Spec-Drift-Waiver: first\n\
         Spec-Drift-Waiver-Paths: b, a\n\
         Spec-Drift-Waiver-Paths: c\n\
         Spec-Drift-Waiver: second\n\
         \x20 Spec-Drift-Waiver-Until: 2026-12-31\n",
    );
    assert_eq!(set.unattached, vec!["Spec-Drift-Waiver-Paths: early/"]);
    assert_eq!(set.declarations.len(), 2);
    assert_eq!(
        set.declarations[0].paths.as_deref(),
        Some(&["a".to_string(), "b".into(), "c".into()][..])
    );
    assert_eq!(set.declarations[0].until, None);
    assert_eq!(set.declarations[1].paths, None);
    assert_eq!(set.declarations[1].until.as_deref(), Some("2026-12-31"));
}

#[test]
fn an_unattached_line_is_reported_and_narrows_nothing() {
    let r = run_body(
        "Spec-Drift-Waiver-Paths: src/a.rs\nSpec-Drift-Waiver: after it\n",
        &no_inputs(),
        &both(),
    );
    assert_eq!(
        r.unattached_waiver_lines,
        vec!["Spec-Drift-Waiver-Paths: src/a.rs"]
    );
    assert!(
        !r.waivers[0].scoped,
        "the line above the waiver is not its scope"
    );
    assert!(!r.has_blocking_drift());
}

#[test]
fn the_first_declaration_is_what_parse_waiver_returns() {
    let cfg = Config::default();
    for body in [
        "Spec-Drift-Waiver: one\nSpec-Drift-Waiver: two\n",
        "text\n   Spec-Drift-Waiver:   spaced   \n",
        "Spec-Drift-Waiver:\nSpec-Drift-Waiver: after an empty one\n",
    ] {
        assert_eq!(
            parse_waivers(&cfg, body).declarations[0].reason,
            parse_waiver(&cfg, body).unwrap().reason,
            "{body}"
        );
    }
    assert!(
        parse_waivers(&cfg, "no waiver here\n")
            .declarations
            .is_empty()
    );
}

#[test]
fn the_lifecycle_keys_follow_the_configured_keyword() {
    let mut cfg = Config::default();
    cfg.coupling.waiver_keyword = "Drift-OK:".into();
    let set = parse_waivers(
        &cfg,
        "Drift-OK: custom\nDrift-OK-Paths: x.rs\nSpec-Drift-Waiver-Paths: y.rs\n",
    );
    assert_eq!(set.declarations.len(), 1);
    assert_eq!(
        set.declarations[0].paths.as_deref(),
        Some(&["x.rs".to_string()][..])
    );
}

#[test]
fn the_id_is_the_declaration_not_its_layout() {
    let cfg = Config::default();
    let id = |b: &str| parse_waivers(&cfg, b).declarations[0].id();
    let a = id("Spec-Drift-Waiver: r\nSpec-Drift-Waiver-Paths: a, b\n");
    assert_eq!(
        a,
        id("Spec-Drift-Waiver: r\nSpec-Drift-Waiver-Paths: b\nSpec-Drift-Waiver-Paths: a\n")
    );
    assert!(a.starts_with("sha256:"), "{a}");
    assert_ne!(a, id("Spec-Drift-Waiver: r\nSpec-Drift-Waiver-Paths: a\n"));
    assert_ne!(
        a,
        id(
            "Spec-Drift-Waiver: r\nSpec-Drift-Waiver-Paths: a, b\nSpec-Drift-Waiver-Until: 2026-12-31\n"
        )
    );
    assert_ne!(
        id("Spec-Drift-Waiver: r\n"),
        id("Spec-Drift-Waiver: r\nSpec-Drift-Waiver-Paths: \n"),
        "scoped to nothing is not unscoped"
    );
}

#[test]
fn an_as_of_that_is_not_a_date_is_a_usage_error() {
    let err = couple_with_prior_waived(
        &Config::default(),
        &registry(),
        &index(),
        &GovernedScope::empty(),
        &PriorSnapshots::default(),
        &both(),
        &WaiverSet::default(),
        &as_of("23/09/2026"),
    );
    assert!(
        matches!(err, Err(spec_spine_types::Error::Usage(_))),
        "the caller's input is refused, not compared as text"
    );
    assert!(matches!(
        as_of("23/09/2026").validate(),
        Err(spec_spine_types::Error::Usage(_))
    ));
}

// ── §3.8 compatibility ─────────────────────────────────────────────────────

fn keys(r: &CoupleReport) -> BTreeSet<String> {
    let v: Value = serde_json::to_value(r).unwrap();
    v.as_object().unwrap().keys().cloned().collect()
}

#[test]
fn a_run_with_no_waiver_keeps_its_exact_members() {
    let r = run_set(&WaiverSet::default(), &no_inputs(), &both());
    assert_eq!(
        keys(&r),
        ["checkedPaths", "violations"]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        "no new member appears on a run that declares no waiver"
    );
    let legacy = couple_with(&Config::default(), &registry(), &index(), &both(), None).unwrap();
    assert_eq!(
        serde_json::to_string(&r).unwrap(),
        serde_json::to_string(&legacy).unwrap()
    );
}

#[test]
fn a_plain_waiver_decides_exactly_as_before() {
    let legacy = couple_with(
        &Config::default(),
        &registry(),
        &index(),
        &both(),
        Some(&Waiver {
            reason: "hotfix".into(),
        }),
    )
    .unwrap();
    let r = run_body("Spec-Drift-Waiver: hotfix\n", &no_inputs(), &both());
    assert_eq!(r.exit_code(), legacy.exit_code());
    assert_eq!(r.violations, legacy.violations);
    assert_eq!(r.waiver, legacy.waiver);
    // What is added is the report of the waiver itself, which §3.2 requires.
    let mut v: Value = serde_json::to_value(&r).unwrap();
    v.as_object_mut().unwrap().remove("waivers");
    let mut l: Value = serde_json::to_value(&legacy).unwrap();
    l.as_object_mut().unwrap().remove("waivers");
    assert_eq!(v, l);
    assert_eq!(r.waivers.len(), 1);
    assert!(!r.waivers[0].scoped && r.waivers[0].checks.is_empty());
}

#[test]
fn a_declaration_as_data_matches_the_same_declaration_parsed() {
    let parsed = run_body(
        "Spec-Drift-Waiver: bump\nSpec-Drift-Waiver-Paths: src/a.rs\n",
        &no_inputs(),
        &both(),
    );
    let data = run_set(
        &WaiverSet {
            declarations: vec![WaiverDeclaration {
                reason: "bump".into(),
                paths: Some(vec!["src/a.rs".into()]),
                ..WaiverDeclaration::default()
            }],
            unattached: Vec::new(),
        },
        &no_inputs(),
        &both(),
    );
    assert_eq!(parsed, data);
}
