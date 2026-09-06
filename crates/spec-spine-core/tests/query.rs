// Spec: specs/002-registry-query/spec.md
//! Query: load_registry version gating + list / show / status_report /
//! relationships over a compiled registry.

use std::fs;
use std::path::Path;

use spec_spine_core::{
    ListFilter, compile, list, load_registry, relationships, show, status_report,
};
use spec_spine_types::{Config, Error, Status};

fn write_spec(root: &Path, id: &str, extra: &str) {
    let spec_dir = root.join("specs").join(id);
    fs::create_dir_all(&spec_dir).unwrap();
    let body = format!(
        "---\nid: \"{id}\"\ntitle: \"Title {id}\"\nstatus: approved\ncreated: \"2026-06-08\"\nsummary: \"s\"\n{extra}---\n# {id}\n"
    );
    fs::write(spec_dir.join("spec.md"), body).unwrap();
}

fn corpus() -> spec_spine_types::Registry {
    let tmp = tempfile::tempdir().unwrap();
    write_spec(tmp.path(), "001-alpha", "");
    write_spec(tmp.path(), "002-beta", "depends_on: [\"001-alpha\"]\n");
    compile(&Config::default(), tmp.path()).unwrap().registry
}

#[test]
fn load_registry_rejects_unknown_major() {
    let bad = r#"{"specVersion":"9.0.0","build":{"compilerId":"x","compilerVersion":"0.1.0","inputRoot":".","contentHash":"0000000000000000000000000000000000000000000000000000000000000000"},"specs":[],"validation":{"passed":true,"violations":[]}}"#;
    let err = load_registry(bad.as_bytes()).unwrap_err();
    assert!(matches!(err, Error::Schema(_)));
    assert_eq!(err.exit_code(), 3);
}

#[test]
fn load_registry_accepts_our_major() {
    let reg = corpus();
    let bytes = serde_json::to_vec(&reg).unwrap();
    let loaded = load_registry(&bytes).unwrap();
    assert_eq!(loaded.specs.len(), 2);
}

#[test]
fn list_filters_by_status() {
    let reg = corpus();
    assert_eq!(list(&reg, &ListFilter::default()).len(), 2);
    let approved = list(
        &reg,
        &ListFilter {
            status: Some(Status::Approved),
        },
    );
    assert_eq!(approved.len(), 2);
    let drafts = list(
        &reg,
        &ListFilter {
            status: Some(Status::Draft),
        },
    );
    assert!(drafts.is_empty());
}

#[test]
fn show_finds_or_not_found() {
    let reg = corpus();
    assert_eq!(show(&reg, "001-alpha").unwrap().id, "001-alpha");
    assert!(matches!(show(&reg, "404-x"), Err(Error::NotFound(_))));
}

#[test]
fn status_report_counts() {
    let report = status_report(&corpus());
    assert_eq!(report.total, 2);
    assert_eq!(report.approved, 2);
    assert_eq!(report.draft, 0);
}

#[test]
fn relationships_show_incoming_and_outgoing() {
    let reg = corpus();
    let alpha = relationships(&reg, "001-alpha").unwrap();
    assert_eq!(alpha.depended_on_by, vec!["002-beta".to_string()]);
    assert!(alpha.depends_on.is_empty());

    let beta = relationships(&reg, "002-beta").unwrap();
    assert_eq!(beta.depends_on, vec!["001-alpha".to_string()]);
    assert!(beta.depended_on_by.is_empty());
}

// ===== spec 038: `registry plan` =====

/// Build a registry from `(id, status, implementation, depends_on)` rows.
///
/// Goes through the deserializer rather than a struct literal because that is
/// the path a committed shard takes, and because it is the only way to reach the
/// states `compile` refuses to emit (a cycle, a dangling dependency), which are
/// exactly the ones `plan` has to survive.
fn registry_of(rows: &[(&str, &str, Option<&str>, &[&str])]) -> spec_spine_types::Registry {
    let specs: Vec<serde_json::Value> = rows
        .iter()
        .map(|(id, status, implementation, deps)| {
            let mut spec = serde_json::json!({
                "id": id,
                "title": "T",
                "status": status,
                "created": "2026-09-06",
                "summary": "s",
                "specPath": format!("specs/{id}/spec.md"),
                "dependsOn": deps,
            });
            if let Some(impl_) = implementation {
                spec["implementation"] = serde_json::json!(impl_);
            }
            spec
        })
        .collect();
    serde_json::from_value(serde_json::json!({
        "specVersion": spec_spine_types::REGISTRY_SCHEMA_VERSION,
        "build": {
            "compilerId": "spec-spine",
            "compilerVersion": "0.0.0",
            "inputRoot": ".",
            "contentHash": "0".repeat(64),
        },
        "specs": specs,
        "validation": { "passed": true, "violations": [] },
    }))
    .unwrap()
}

fn blockers(plan: &spec_spine_core::Plan, id: &str) -> Vec<(String, String)> {
    plan.blocked
        .iter()
        .find(|b| b.id == id)
        .unwrap_or_else(|| panic!("{id} is not blocked; plan: {plan:?}"))
        .blocked_by
        .iter()
        .map(|b| (b.id.clone(), b.state.clone()))
        .collect()
}

#[test]
fn plan_walks_a_linear_chain_one_step_at_a_time() {
    let reg = registry_of(&[
        ("001-a", "approved", Some("pending"), &[]),
        ("002-b", "approved", Some("pending"), &["001-a"]),
        ("003-c", "approved", Some("pending"), &["002-b"]),
    ]);
    let plan = spec_spine_core::plan(&reg).unwrap();
    assert_eq!(plan.ready, vec!["001-a"]);
    assert_eq!(
        blockers(&plan, "002-b"),
        [("001-a".into(), "pending".into())]
    );
    assert_eq!(
        blockers(&plan, "003-c"),
        [("002-b".into(), "pending".into())]
    );
}

#[test]
fn plan_offers_both_arms_of_a_diamond_in_id_order() {
    let reg = registry_of(&[
        ("001-root", "approved", Some("complete"), &[]),
        ("003-right", "approved", Some("pending"), &["001-root"]),
        ("002-left", "approved", Some("pending"), &["001-root"]),
        (
            "004-join",
            "approved",
            Some("pending"),
            &["002-left", "003-right"],
        ),
    ]);
    let plan = spec_spine_core::plan(&reg).unwrap();
    assert_eq!(plan.ready, vec!["002-left", "003-right"]);
    assert_eq!(
        blockers(&plan, "004-join"),
        [
            ("002-left".into(), "pending".into()),
            ("003-right".into(), "pending".into())
        ],
        "every blocker is named, not just the first"
    );
}

#[test]
fn plan_treats_complete_and_n_a_as_finished_and_everything_else_as_blocking() {
    // One dependent per dependency state, so each arm is asserted separately.
    let reg = registry_of(&[
        ("001-complete", "approved", Some("complete"), &[]),
        ("002-na", "approved", Some("n-a"), &[]),
        ("003-pending", "approved", Some("pending"), &[]),
        ("004-inprogress", "approved", Some("in-progress"), &[]),
        ("005-deferred", "approved", Some("deferred"), &[]),
        (
            "010-on-complete",
            "approved",
            Some("pending"),
            &["001-complete"],
        ),
        ("011-on-na", "approved", Some("pending"), &["002-na"]),
        (
            "012-on-pending",
            "approved",
            Some("pending"),
            &["003-pending"],
        ),
        (
            "013-on-inprogress",
            "approved",
            Some("pending"),
            &["004-inprogress"],
        ),
        (
            "014-on-deferred",
            "approved",
            Some("pending"),
            &["005-deferred"],
        ),
    ]);
    let plan = spec_spine_core::plan(&reg).unwrap();

    assert_eq!(
        plan.ready,
        vec![
            "003-pending",
            "004-inprogress",
            "010-on-complete",
            "011-on-na",
        ],
        "only `complete` and `n-a` dependencies clear the way"
    );
    // The blocked entries carry the state that made each one a blocker, which is
    // what lets a reader answer "why not that one" without a second lookup.
    assert_eq!(
        blockers(&plan, "012-on-pending"),
        [("003-pending".into(), "pending".into())]
    );
    assert_eq!(
        blockers(&plan, "013-on-inprogress"),
        [("004-inprogress".into(), "in-progress".into())]
    );
    // A `deferred` blocker appears in neither output set while still blocking,
    // so its entry is the only place it is named at all.
    assert_eq!(
        blockers(&plan, "014-on-deferred"),
        [("005-deferred".into(), "deferred".into())]
    );
    let named: Vec<&str> = plan
        .ready
        .iter()
        .map(String::as_str)
        .chain(plan.blocked.iter().map(|b| b.id.as_str()))
        .collect();
    assert!(!named.contains(&"005-deferred"), "{named:?}");
}

#[test]
fn plan_excludes_specs_the_corpus_has_moved_past_or_taken_off_the_schedule() {
    let reg = registry_of(&[
        ("001-superseded", "superseded", Some("pending"), &[]),
        ("002-retired", "retired", Some("pending"), &[]),
        ("003-complete", "approved", Some("complete"), &[]),
        ("004-na", "approved", Some("n-a"), &[]),
        ("005-deferred", "approved", Some("deferred"), &[]),
        ("006-draft", "draft", Some("pending"), &[]),
    ]);
    let plan = spec_spine_core::plan(&reg).unwrap();
    assert_eq!(
        plan.ready,
        vec!["006-draft"],
        "a draft is schedulable; the other five are not"
    );
    assert!(plan.blocked.is_empty(), "{plan:?}");
}

#[test]
fn plan_reads_an_absent_implementation_key_as_pending() {
    let reg = registry_of(&[
        ("001-silent", "approved", None, &[]),
        (
            "002-on-silent",
            "approved",
            Some("pending"),
            &["001-silent"],
        ),
    ]);
    let plan = spec_spine_core::plan(&reg).unwrap();
    assert_eq!(plan.ready, vec!["001-silent"], "unstated is schedulable");
    assert_eq!(
        blockers(&plan, "002-on-silent"),
        [("001-silent".into(), "pending".into())],
        "and unstated blocks its dependents, reported as pending"
    );
}

#[test]
fn plan_blocks_on_a_dependency_that_is_not_in_the_corpus() {
    let reg = registry_of(&[("002-b", "approved", Some("pending"), &["001-missing"])]);
    let plan = spec_spine_core::plan(&reg).unwrap();
    assert!(plan.ready.is_empty(), "{plan:?}");
    assert_eq!(
        blockers(&plan, "002-b"),
        [("001-missing".into(), "unresolved".into())],
        "a dangling dependency is a corpus defect, never a satisfied one"
    );
}

#[test]
fn plan_is_stable_across_runs_and_independent_of_corpus_order() {
    let rows: [(&str, &str, Option<&str>, &[&str]); 3] = [
        ("001-a", "approved", Some("pending"), &[]),
        ("002-b", "approved", Some("pending"), &[]),
        ("003-c", "approved", Some("pending"), &["001-a", "002-b"]),
    ];
    let forward = spec_spine_core::plan(&registry_of(&rows)).unwrap();
    let mut reversed = rows;
    reversed.reverse();
    let backward = spec_spine_core::plan(&registry_of(&reversed)).unwrap();
    assert_eq!(forward, backward, "output is a function of the corpus");
    assert_eq!(
        forward,
        spec_spine_core::plan(&registry_of(&rows)).unwrap(),
        "and of nothing else"
    );
    assert_eq!(forward.ready, vec!["001-a", "002-b"]);
    assert_eq!(
        blockers(&forward, "003-c"),
        [
            ("001-a".into(), "pending".into()),
            ("002-b".into(), "pending".into())
        ],
        "blockers are ordered by the spec's own depends_on list"
    );
}

#[test]
fn plan_refuses_a_cycle_rather_than_looping() {
    // `compile` refuses a cycle (V-014), so this can only arrive from a
    // hand-edited shard or another tool version; `plan` must terminate on it.
    let reg = registry_of(&[
        ("001-a", "approved", Some("pending"), &["003-c"]),
        ("002-b", "approved", Some("pending"), &["001-a"]),
        ("003-c", "approved", Some("pending"), &["002-b"]),
    ]);
    let err = spec_spine_core::plan(&reg).unwrap_err();
    assert_eq!(
        err.exit_code(),
        1,
        "a broken corpus is a validation failure"
    );
    let Error::Validation(violations) = &err else {
        panic!("expected Error::Validation, got {err:?}");
    };
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].code, "V-014", "033's classification, reused");
    let message = &violations[0].message;
    for id in ["001-a", "002-b", "003-c"] {
        assert!(
            message.contains(id),
            "the path names every spec on it: {message}"
        );
    }
}

#[test]
fn plan_over_a_compiled_corpus_only_offers_unfinished_specs() {
    let tmp = tempfile::tempdir().unwrap();
    write_spec(tmp.path(), "001-alpha", "implementation: complete\n");
    write_spec(
        tmp.path(),
        "002-beta",
        "depends_on: [\"001-alpha\"]\nimplementation: pending\n",
    );
    write_spec(
        tmp.path(),
        "003-gamma",
        "depends_on: [\"002-beta\"]\nimplementation: pending\n",
    );
    let reg = compile(&Config::default(), tmp.path()).unwrap().registry;

    let plan = spec_spine_core::plan(&reg).unwrap();
    assert_eq!(plan.ready, vec!["002-beta"]);
    assert_eq!(
        blockers(&plan, "003-gamma"),
        [("002-beta".into(), "pending".into())]
    );
    // Nothing `complete` is offered, which is the invariant a scheduler relies
    // on and the one this projection must never violate.
    let offered: Vec<&str> = plan
        .ready
        .iter()
        .map(String::as_str)
        .chain(plan.blocked.iter().map(|b| b.id.as_str()))
        .collect();
    assert!(!offered.contains(&"001-alpha"), "{offered:?}");
}

/// The degeneracy `topological`'s doc comment claims: no ready spec depends on
/// another ready spec, so the walk and an id sort agree.
///
/// Asserted rather than reasoned about in a comment, because it is a property of
/// `schedulable` and `blocker_state` rather than of the ordering code, and a
/// later change to either could break it silently.
#[test]
fn plan_ready_set_has_no_internal_edges() {
    let reg = registry_of(&[
        ("001-done", "approved", Some("complete"), &[]),
        ("002-a", "approved", Some("pending"), &["001-done"]),
        ("003-b", "approved", Some("pending"), &["001-done"]),
        ("004-c", "approved", Some("pending"), &["002-a"]),
    ]);
    let plan = spec_spine_core::plan(&reg).unwrap();
    let ready: std::collections::BTreeSet<&str> = plan.ready.iter().map(String::as_str).collect();
    for id in &ready {
        let deps = &reg.specs.iter().find(|s| s.id == *id).unwrap().depends_on;
        for dep in deps {
            assert!(
                !ready.contains(dep.as_str()),
                "{id} depends on {dep}, which is also ready"
            );
        }
    }
    let mut sorted = plan.ready.clone();
    sorted.sort();
    assert_eq!(plan.ready, sorted, "so the walk agrees with an id sort");
}

/// A long chain must terminate with the verdict, not abort the process.
///
/// This guard runs only on input `compile` refuses to emit, which is exactly the
/// input that may be arbitrarily malformed, so a recursive walk would overflow
/// the stack here and exit outside the documented `0`/`1`/`2`/`3` contract.
#[test]
fn plan_survives_a_very_long_cycle() {
    const N: usize = 20_000;
    let ids: Vec<String> = (0..N).map(|i| format!("{i:06}-spec")).collect();
    let rows: Vec<(&str, &str, Option<&str>, Vec<&str>)> = (0..N)
        .map(|i| {
            // Each spec depends on the next; the last closes the loop.
            let dep = ids[(i + 1) % N].as_str();
            (ids[i].as_str(), "approved", Some("pending"), vec![dep])
        })
        .collect();
    let borrowed: Vec<(&str, &str, Option<&str>, &[&str])> = rows
        .iter()
        .map(|(id, status, im, deps)| (*id, *status, *im, deps.as_slice()))
        .collect();

    let err = spec_spine_core::plan(&registry_of(&borrowed)).unwrap_err();
    assert_eq!(err.exit_code(), 1);
    let Error::Validation(violations) = &err else {
        panic!("expected Error::Validation, got {err:?}");
    };
    assert_eq!(violations[0].code, "V-014");
}

/// The same depth, acyclic: the walk must complete rather than overflow on the
/// way to reporting no cycle at all.
#[test]
fn plan_survives_a_very_long_acyclic_chain() {
    const N: usize = 20_000;
    let ids: Vec<String> = (0..N).map(|i| format!("{i:06}-spec")).collect();
    let rows: Vec<(&str, &str, Option<&str>, Vec<&str>)> = (0..N)
        .map(|i| {
            let deps = if i == 0 {
                Vec::new()
            } else {
                vec![ids[i - 1].as_str()]
            };
            (ids[i].as_str(), "approved", Some("pending"), deps)
        })
        .collect();
    let borrowed: Vec<(&str, &str, Option<&str>, &[&str])> = rows
        .iter()
        .map(|(id, status, im, deps)| (*id, *status, *im, deps.as_slice()))
        .collect();

    let plan = spec_spine_core::plan(&registry_of(&borrowed)).unwrap();
    assert_eq!(
        plan.ready,
        vec!["000000-spec"],
        "only the head is unblocked"
    );
    assert_eq!(plan.blocked.len(), N - 1);
}

/// `plan`'s cycle guard and `compile`'s `V-014` must agree on which specs the
/// walk covers, and the agreement is asserted rather than assumed.
///
/// `plan`'s guard is documented as unreachable on a registry `compile` accepts.
/// That is true only while both walk the same set: `compile` passes every
/// record to its detector with no `status` filter, and `plan` walks every entry
/// in the registry. Nothing enforces that pairing, so if `compile`'s check were
/// ever scoped to active specs, `plan` would start refusing a corpus `compile`
/// accepts and no test would notice.
///
/// A cycle confined to retired specs is the case where the two would diverge
/// first, so it is the one pinned here.
#[test]
fn plan_and_compile_agree_on_a_cycle_among_retired_specs() {
    let tmp = tempfile::tempdir().unwrap();
    let retired = |id: &str, dep: &str| {
        let dir = tmp.path().join("specs").join(id);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("spec.md"),
            format!(
                "---\nid: \"{id}\"\ntitle: \"T\"\nstatus: retired\ncreated: \"2026-09-06\"\n\
                 summary: \"s\"\nretirement_rationale: \"superseded by nothing\"\n\
                 depends_on: [\"{dep}\"]\n---\n# {id}\n## body\n"
            ),
        )
        .unwrap();
    };
    retired("001-a", "002-b");
    retired("002-b", "001-a");

    // The pivot of the whole test: `compile` returns `Ok` for a cycle, carrying
    // V-014 in the report. Named here so a future change to `Err` fails saying
    // that, rather than as an unexplained panic in a test about scope.
    let outcome = compile(&Config::default(), tmp.path())
        .expect("compile returns Ok on a cyclic corpus; the violation rides in the report");

    // The detector fires: it is not scoped to active specs. `compile` still
    // returns `Ok`, carrying the violation in the report rather than as an
    // error, which is why the call above unwraps.
    let compile_v014: Vec<_> = outcome
        .registry
        .validation
        .violations
        .iter()
        .filter(|v| v.code == "V-014")
        .collect();
    // Asserted before the count so a schema rejection of `depends_on` on a
    // retired spec would fail here, naming its own cause, rather than surfacing
    // as a confusing "no V-014 was raised".
    let unexpected: Vec<_> = outcome
        .registry
        .validation
        .violations
        .iter()
        .filter(|v| v.code != "V-014" && v.severity == spec_spine_types::Severity::Error)
        .collect();
    assert!(
        unexpected.is_empty(),
        "the fixture must reach the cycle detector cleanly: {unexpected:?}"
    );
    assert_eq!(
        compile_v014.len(),
        1,
        "compile must refuse a cycle among retired specs: {:?}",
        outcome.registry.validation.violations
    );

    // ...and so does `plan`, over the same registry. Were `compile` ever scoped
    // to active specs, this corpus would compile clean and this assertion would
    // fail, which is exactly the divergence the guard's comment assumes away.
    let err = spec_spine_core::plan(&outcome.registry).unwrap_err();
    let Error::Validation(violations) = &err else {
        panic!("expected Error::Validation, got {err:?}");
    };
    assert_eq!(violations[0].code, "V-014", "the same classification");
    for id in ["001-a", "002-b"] {
        assert!(
            violations[0].message.contains(id),
            "the path names every spec on it: {}",
            violations[0].message
        );
    }
}

/// `blocked` is ordered by ascending spec id, and that is a promise rather than
/// a `BTreeMap` side effect a consumer would be relying on by accident.
#[test]
fn plan_blocked_entries_are_ordered_by_id() {
    let reg = registry_of(&[
        ("001-root", "approved", Some("pending"), &[]),
        ("004-d", "approved", Some("pending"), &["001-root"]),
        ("002-b", "approved", Some("pending"), &["001-root"]),
        ("003-c", "approved", Some("pending"), &["001-root"]),
    ]);
    let plan = spec_spine_core::plan(&reg).unwrap();
    let ids: Vec<&str> = plan.blocked.iter().map(|b| b.id.as_str()).collect();
    assert_eq!(ids, vec!["002-b", "003-c", "004-d"]);
    let mut sorted = ids.clone();
    sorted.sort();
    assert_eq!(ids, sorted, "ascending id, as the contract states");
}

/// The other half of 3.2's ordering contract: each entry's `blockedBy` follows
/// that spec's own **authored** `depends_on` order, not ascending id.
///
/// The two orders are deliberately opposed in this fixture, so an implementation
/// that collected blockers from the id-sorted map instead of from the spec's own
/// list would fail rather than coincidentally agree.
#[test]
fn plan_blocked_by_follows_authored_depends_on_order() {
    let reg = registry_of(&[
        ("001-a", "approved", Some("pending"), &[]),
        ("002-b", "approved", Some("pending"), &[]),
        ("003-c", "approved", Some("pending"), &[]),
        // Authored back to front: c, a, b.
        (
            "004-join",
            "approved",
            Some("pending"),
            &["003-c", "001-a", "002-b"],
        ),
    ]);
    let plan = spec_spine_core::plan(&reg).unwrap();
    let blockers: Vec<&str> = plan
        .blocked
        .iter()
        .find(|b| b.id == "004-join")
        .expect("004-join is blocked")
        .blocked_by
        .iter()
        .map(|b| b.id.as_str())
        .collect();
    assert_eq!(
        blockers,
        vec!["003-c", "001-a", "002-b"],
        "authored order is preserved, not re-sorted by id"
    );
}
