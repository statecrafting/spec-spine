// Spec: specs/097-a-path-leaves-the-corpus-the-way-a-spec-does/spec.md
//! Path-retirement tests (spec 097): the forms a path is spelled in, the
//! exclusions that are the hard part, and the refusals.
//!
//! The exclusions carry the weight. Eleven of the 81 occurrences in the
//! retirement this spec was measured on had to survive, because a sentence
//! recording that a path USED to exist is not a citation of it, and a tool that
//! rewrote them would have produced a corpus that compiles, passes every gate,
//! and lies in five places.

use std::fs;
use std::path::Path;

use spec_spine_core::compact::{
    CompactPlan, Form, RetireEntry, RetireKind, SkipClause, UnitAction, UnitActionKind, compact,
    parse_plan,
};
use spec_spine_types::{Config, load_config};

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, content).unwrap();
}

fn spec_doc(id: &str, extra: &str, body: &str) -> String {
    format!(
        "---\nid: \"{id}\"\ntitle: \"T\"\nstatus: approved\ncreated: \"2026-06-09\"\n\
         summary: \"s\"\nimplementation: complete\n{extra}---\n\n# {id}\n\n{body}\n"
    )
}

fn cfg() -> Config {
    load_config("[layout]\nspecs_dir = \"specs\"\n").unwrap()
}

/// A corpus with a retirable tree: `rules/` holds two files, `000-alpha` claims
/// the directory, `001-beta` is a draft that references one file, and the prose
/// cites both the directory and a file.
fn fixture(prose: &str) -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    let r = tmp.path();
    write(
        r,
        "specs/000-alpha/spec.md",
        &spec_doc(
            "000-alpha",
            "establishes:\n  - { kind: directory, path: \"rules/\" }\n",
            "Body.",
        ),
    );
    write(
        r,
        "specs/001-beta/spec.md",
        &spec_doc("001-beta", "", "Body."),
    );
    write(r, "rules/one.md", "# one\n");
    write(r, "rules/two.md", "# two\n");
    write(
        r,
        "AGENTS.md",
        "# protocol\n\n## Rules\n\nThe rules live here.\n",
    );
    write(r, "docs/note.md", prose);
    tmp
}

fn retire_rules() -> RetireEntry {
    RetireEntry {
        path: "rules/one.md".into(),
        kind: RetireKind::File,
        forms: [(
            "citation".to_string(),
            Some("`AGENTS.md` \"Rules\"".to_string()),
        )]
        .into_iter()
        .collect(),
        units: vec![],
        historical_files: vec![],
        historical_sections: vec![],
    }
}

fn plan_with(entry: RetireEntry) -> CompactPlan {
    CompactPlan {
        retire: vec![entry],
        ..Default::default()
    }
}

fn prose_of(c: &spec_spine_core::compact::Compaction, rel: &str) -> String {
    c.files
        .iter()
        .find(|f| f.from_rel_path == rel)
        .map(|f| f.contents.clone())
        .unwrap_or_default()
}

// ── §3.2 the forms ───────────────────────────────────────────────────────────

#[test]
fn a_backticked_citation_is_replaced() {
    let tmp = fixture("See `rules/one.md` for the rule.\n");
    let c = compact(&cfg(), tmp.path(), &plan_with(retire_rules())).unwrap();
    assert!(
        prose_of(&c, "docs/note.md").contains("See `AGENTS.md` \"Rules\" for the rule."),
        "{}",
        prose_of(&c, "docs/note.md")
    );
}

#[test]
fn a_bare_citation_is_replaced() {
    let tmp = fixture("See rules/one.md, then stop.\n");
    let c = compact(&cfg(), tmp.path(), &plan_with(retire_rules())).unwrap();
    assert!(prose_of(&c, "docs/note.md").contains("See `AGENTS.md` \"Rules\", then stop."));
}

#[test]
fn a_longer_path_is_not_a_retired_one() {
    let tmp = fixture("kit/rules/one.md is a different file.\n");
    let c = compact(&cfg(), tmp.path(), &plan_with(retire_rules())).unwrap();
    // No rewrite at all is the expected outcome, so the file is absent from the
    // result rather than present and unchanged: assert the absence, and assert
    // it for the reason claimed rather than for an empty report.
    assert!(
        c.rewrites.iter().all(|f| f.rel_path != "docs/note.md"),
        "{:?}",
        c.rewrites
    );
    assert!(c.skipped.is_empty(), "{:?}", c.skipped);
    // And §3.7 agrees with the rewrite: a longer path is not an unaccounted
    // occurrence, so it must not refuse the run. The leftover scan used a raw
    // `contains` while the rewrite used a boundary test, and the two disagreed.
    assert!(c.leftover.is_empty(), "{:?}", c.leftover);
}

// ── §3.5 the exclusions ──────────────────────────────────────────────────────

/// §1.3: an absence assertion names the path precisely BECAUSE the path is
/// gone. Rewriting it produces a corpus that compiles and lies.
#[test]
fn a_negation_is_not_a_citation() {
    let tmp = fixture("! test -e rules/one.md\n");
    let c = compact(&cfg(), tmp.path(), &plan_with(retire_rules())).unwrap();
    assert!(
        prose_of(&c, "docs/note.md").is_empty()
            || prose_of(&c, "docs/note.md").contains("! test -e rules/one.md")
    );
    assert!(
        c.skipped.iter().any(|s| s.clause == SkipClause::Negation),
        "{:?}",
        c.skipped
    );
}

#[test]
fn a_must_not_clause_is_not_a_citation() {
    let tmp = fixture("`rules/one.md` MUST NOT exist after this.\n");
    let c = compact(&cfg(), tmp.path(), &plan_with(retire_rules())).unwrap();
    assert!(
        c.skipped.iter().any(|s| s.clause == SkipClause::Negation),
        "{:?}",
        c.skipped
    );
}

#[test]
fn a_historical_file_is_left_alone_and_reported() {
    let tmp = fixture("`rules/one.md` used to live here.\n");
    let mut e = retire_rules();
    e.historical_files = vec!["docs/note.md".into()];
    let c = compact(&cfg(), tmp.path(), &plan_with(e)).unwrap();
    assert!(
        c.skipped
            .iter()
            .any(|s| s.clause == SkipClause::HistoricalFile && s.rel_path == "docs/note.md"),
        "{:?}",
        c.skipped
    );
}

#[test]
fn a_historical_section_is_left_alone_and_reported() {
    let tmp = fixture("## What was dropped\n\n`rules/one.md` was deleted two specs ago.\n");
    let mut e = retire_rules();
    e.historical_sections = vec!["What was dropped".into()];
    let c = compact(&cfg(), tmp.path(), &plan_with(e)).unwrap();
    assert!(
        c.skipped
            .iter()
            .any(|s| s.clause == SkipClause::HistoricalSection),
        "{:?}",
        c.skipped
    );
}

/// §3.5: silence is the failure. Every skip names the clause that produced it.
#[test]
fn every_skip_names_the_clause_that_produced_it() {
    let tmp = fixture("! test -e rules/one.md\n`rules/one.md` is cited.\n");
    let c = compact(&cfg(), tmp.path(), &plan_with(retire_rules())).unwrap();
    for s in &c.skipped {
        assert!(!s.text.is_empty() && s.line > 0, "{s:?}");
        assert!(!s.clause.as_str().is_empty());
    }
}

/// §3.3: a frontmatter unit claiming a LONGER path is not the retired one. The
/// unit matcher used a bare `contains` while the prose matcher had a boundary
/// test, so `kit/rules/sub/` matched a retirement of `rules/` and was edited.
#[test]
fn a_unit_claiming_a_longer_path_is_not_the_retired_one() {
    let tmp = fixture("x");
    write(
        tmp.path(),
        "specs/001-beta/spec.md",
        &spec_doc(
            "001-beta",
            "establishes:\n  - { kind: directory, path: \"kit/rules/sub/\" }\n",
            "Body.",
        ),
    );
    let mut e = retire_rules();
    e.path = "rules/".into();
    e.kind = RetireKind::Directory;
    e.forms = [("citation".to_string(), Some("`AGENTS.md`".to_string()))]
        .into_iter()
        .collect();
    e.units = vec![UnitAction {
        spec: "001-beta".into(),
        edge: "establishes".into(),
        action: UnitActionKind::Withdraw,
        to: None,
        acknowledge_approved: true,
    }];
    let c = compact(&cfg(), tmp.path(), &plan_with(e)).unwrap();
    let beta = c
        .files
        .iter()
        .find(|f| f.from_rel_path == "specs/001-beta/spec.md");
    if let Some(beta) = beta {
        assert!(
            beta.contents.contains("kit/rules/sub/"),
            "a longer path was withdrawn:\n{}",
            beta.contents
        );
    }
}

/// §3.5: a line one entry spares is spared, full stop. Advancing to the next
/// entry let it rewrite a line the report had already called left alone, so the
/// report described a file that was not the one emitted.
#[test]
fn a_line_one_entry_spares_is_not_rewritten_by_another() {
    let tmp = fixture("! test -e rules/one.md && test -e rules/two.md\n");
    let mut first = retire_rules();
    first.historical_files = vec![];
    let mut second = retire_rules();
    second.path = "rules/two.md".into();
    second.forms = [(
        "citation".to_string(),
        Some("`AGENTS.md` \"Two\"".to_string()),
    )]
    .into_iter()
    .collect();
    let plan = CompactPlan {
        retire: vec![first, second],
        ..Default::default()
    };
    let c = compact(&cfg(), tmp.path(), &plan).unwrap();
    assert!(
        !c.skipped.is_empty(),
        "the negation is reported as spared: {:?}",
        c.skipped
    );
    // The emitted file must be the one the report describes.
    let emitted = c
        .files
        .iter()
        .find(|f| f.from_rel_path == "docs/note.md")
        .map(|f| f.contents.clone())
        .unwrap_or_default();
    assert!(
        emitted.is_empty() || emitted.contains("rules/two.md"),
        "a spared line was rewritten by a second entry: {emitted}"
    );
}

/// §3.3: a key separated from its first item by a blank line still has items.
#[test]
fn a_blank_line_between_a_key_and_its_items_does_not_empty_it() {
    let tmp = fixture("x");
    write(
        tmp.path(),
        "specs/001-beta/spec.md",
        &spec_doc(
            "001-beta",
            "establishes:\n\n  - { kind: file, path: \"rules/one.md\" }\n  - \"keep.md\"\n",
            "Body.",
        ),
    );
    let mut e = retire_rules();
    e.units = vec![UnitAction {
        spec: "001-beta".into(),
        edge: "establishes".into(),
        action: UnitActionKind::Withdraw,
        to: None,
        acknowledge_approved: true,
    }];
    let c = compact(&cfg(), tmp.path(), &plan_with(e)).unwrap();
    let beta = c
        .files
        .iter()
        .find(|f| f.from_rel_path == "specs/001-beta/spec.md")
        .expect("the owning spec is rewritten");
    assert!(beta.contents.contains("establishes:"), "{}", beta.contents);
    assert!(beta.contents.contains("keep.md"), "{}", beta.contents);
    assert!(!beta.contents.contains("rules/one.md"), "{}", beta.contents);
}

// ── §3.1 and §3.7 the refusals ───────────────────────────────────────────────

#[test]
fn retiring_a_path_the_tree_does_not_have_is_refused() {
    let tmp = fixture("x");
    let mut e = retire_rules();
    e.path = "rules/absent.md".into();
    let err = compact(&cfg(), tmp.path(), &plan_with(e)).unwrap_err();
    assert_eq!(err.exit_code(), 3, "{err}");
    assert!(format!("{err}").contains("rules/absent.md"), "{err}");
}

#[test]
fn a_form_with_no_rule_is_refused_rather_than_skipped() {
    let tmp = fixture("x");
    let mut e = retire_rules();
    e.forms.insert("markdown".into(), Some("x".into()));
    let err = compact(&cfg(), tmp.path(), &plan_with(e)).unwrap_err();
    assert_eq!(err.exit_code(), 3, "{err}");
    assert!(format!("{err}").contains("markdown"), "{err}");
}

/// §3.3: there is no grammar in this corpus for withdrawing a claim, and the
/// only route is an edit to the owning spec's frontmatter. On an `approved`
/// spec that edit is a human's decision.
#[test]
fn a_unit_on_an_approved_spec_needs_the_human_acknowledgement() {
    let tmp = fixture("x");
    // The unit `000-alpha` claims is the DIRECTORY, so the entry has to name it:
    // an action whose path matches no frontmatter line would assert nothing.
    let mut e = retire_rules();
    e.path = "rules/".into();
    e.kind = RetireKind::Directory;
    e.forms = [("citation".to_string(), Some("`AGENTS.md`".to_string()))]
        .into_iter()
        .collect();
    e.units = vec![UnitAction {
        spec: "000-alpha".into(),
        edge: "establishes".into(),
        action: UnitActionKind::Withdraw,
        to: None,
        acknowledge_approved: false,
    }];
    let err = compact(&cfg(), tmp.path(), &plan_with(e.clone())).unwrap_err();
    assert_eq!(err.exit_code(), 3, "{err}");
    assert!(format!("{err}").contains("approved"), "{err}");

    e.units[0].acknowledge_approved = true;
    let c = compact(&cfg(), tmp.path(), &plan_with(e)).unwrap();
    // The acknowledgement is permission to make the edit, so the edit has to be
    // made: a plan that passes validation and changes no frontmatter leaves the
    // corpus claiming a path that is gone.
    let alpha = c
        .files
        .iter()
        .find(|f| f.from_rel_path == "specs/000-alpha/spec.md")
        .expect("the owning spec is rewritten");
    assert!(
        !alpha.contents.contains("rules/"),
        "the withdrawn unit is still claimed:\n{}",
        alpha.contents
    );
}

/// §3.3: a withdrawal that empties an edge list takes the key with it. A bare
/// `establishes:` with no items is not the same document minus a claim.
#[test]
fn a_withdrawal_that_empties_an_edge_removes_the_key() {
    let tmp = fixture("x");
    let mut e = retire_rules();
    e.path = "rules/".into();
    e.kind = RetireKind::Directory;
    e.forms = [("citation".to_string(), Some("`AGENTS.md`".to_string()))]
        .into_iter()
        .collect();
    e.units = vec![UnitAction {
        spec: "000-alpha".into(),
        edge: "establishes".into(),
        action: UnitActionKind::Withdraw,
        to: None,
        acknowledge_approved: true,
    }];
    let c = compact(&cfg(), tmp.path(), &plan_with(e)).unwrap();
    let alpha = c
        .files
        .iter()
        .find(|f| f.from_rel_path == "specs/000-alpha/spec.md")
        .expect("the owning spec is rewritten");
    assert!(
        !alpha.contents.contains("establishes:"),
        "{}",
        alpha.contents
    );
    assert!(
        alpha.contents.contains("id: \"000-alpha\""),
        "{}",
        alpha.contents
    );
}

/// §3.3: a retarget rewrites the unit's path in place and leaves the edge.
#[test]
fn a_retarget_rewrites_the_unit_and_keeps_the_edge() {
    let tmp = fixture("x");
    let mut e = retire_rules();
    e.path = "rules/".into();
    e.kind = RetireKind::Directory;
    e.forms = [("citation".to_string(), Some("`AGENTS.md`".to_string()))]
        .into_iter()
        .collect();
    e.units = vec![UnitAction {
        spec: "000-alpha".into(),
        edge: "establishes".into(),
        action: UnitActionKind::Retarget,
        to: Some("AGENTS.md".into()),
        acknowledge_approved: true,
    }];
    let c = compact(&cfg(), tmp.path(), &plan_with(e)).unwrap();
    let alpha = c
        .files
        .iter()
        .find(|f| f.from_rel_path == "specs/000-alpha/spec.md")
        .expect("the owning spec is rewritten");
    assert!(
        alpha.contents.contains("establishes:"),
        "{}",
        alpha.contents
    );
    assert!(
        alpha.contents.contains("path: \"AGENTS.md\""),
        "{}",
        alpha.contents
    );
    assert!(!alpha.contents.contains("rules/"), "{}", alpha.contents);
}

/// §3.2 form 2: a path character before the match means a DIFFERENT path. `_`
/// and `-` are path characters and are not alphanumeric, so a boundary test
/// that only asked `is_ascii_alphanumeric` let `_rules/one.md` through.
#[test]
fn an_underscore_prefixed_path_is_not_the_retired_one() {
    let tmp = fixture("_rules/one.md is a different file, and so is x-rules/one.md.\n");
    let c = compact(&cfg(), tmp.path(), &plan_with(retire_rules())).unwrap();
    assert!(
        c.rewrites.iter().all(|f| f.rel_path != "docs/note.md"),
        "{:?}",
        c.rewrites
    );
}

#[test]
fn a_retarget_with_no_target_is_refused() {
    let tmp = fixture("x");
    let mut e = retire_rules();
    e.units = vec![UnitAction {
        spec: "000-alpha".into(),
        edge: "establishes".into(),
        action: UnitActionKind::Retarget,
        to: None,
        acknowledge_approved: true,
    }];
    let err = compact(&cfg(), tmp.path(), &plan_with(e)).unwrap_err();
    assert_eq!(err.exit_code(), 3, "{err}");
}

/// §3.3: a `path` form rule and a unit action on the same frontmatter line. In
/// the wrong order the form rewrites the value, the unit matcher then finds
/// nothing, and the withdrawal silently does not happen.
#[test]
fn a_path_form_does_not_preempt_a_unit_action_on_the_same_line() {
    let tmp = fixture("x");
    let mut e = retire_rules();
    e.path = "rules/".into();
    e.kind = RetireKind::Directory;
    e.forms = [
        ("citation".to_string(), Some("`AGENTS.md`".to_string())),
        ("path".to_string(), Some("AGENTS.md".to_string())),
    ]
    .into_iter()
    .collect();
    e.units = vec![UnitAction {
        spec: "000-alpha".into(),
        edge: "establishes".into(),
        action: UnitActionKind::Withdraw,
        to: None,
        acknowledge_approved: true,
    }];
    let c = compact(&cfg(), tmp.path(), &plan_with(e)).unwrap();
    let alpha = c
        .files
        .iter()
        .find(|f| f.from_rel_path == "specs/000-alpha/spec.md")
        .expect("the owning spec is rewritten");
    assert!(
        !alpha.contents.contains("establishes:"),
        "the withdrawal did not fire:\n{}",
        alpha.contents
    );
}

/// §3.3: a withdrawal has no target, so a `to` beside one is a plan that means
/// something the tool will not do.
#[test]
fn a_withdrawal_that_also_names_a_target_is_refused() {
    let tmp = fixture("x");
    let mut e = retire_rules();
    e.units = vec![UnitAction {
        spec: "000-alpha".into(),
        edge: "establishes".into(),
        action: UnitActionKind::Withdraw,
        to: Some("AGENTS.md".into()),
        acknowledge_approved: true,
    }];
    let err = compact(&cfg(), tmp.path(), &plan_with(e)).unwrap_err();
    assert_eq!(err.exit_code(), 3, "{err}");
    assert!(format!("{err}").contains("withdrawal"), "{err}");
}

/// §3.2 form 2: `.` is both sentence punctuation and an extension separator.
/// `rules/one.md.` ends a sentence; `rules/one.md.bak` is a different file, and
/// a terminator list carrying a bare `.` rewrote it.
#[test]
fn a_period_terminates_a_citation_only_at_the_end_of_a_sentence() {
    let tmp = fixture("See rules/one.md. But rules/one.md.bak is another file.\n");
    let c = compact(&cfg(), tmp.path(), &plan_with(retire_rules())).unwrap();
    let out = c
        .files
        .iter()
        .find(|f| f.from_rel_path == "docs/note.md")
        .map(|f| f.contents.clone())
        .unwrap_or_default();
    assert!(out.contains("See `AGENTS.md` \"Rules\"."), "{out}");
    assert!(out.contains("rules/one.md.bak"), "{out}");
    // And §3.7 agrees: the rewrite was right to leave `.bak` alone, so it is not
    // an unaccounted occurrence. The scan checked the LEFT boundary only, so it
    // refused a corpus the rewrite had handled correctly.
    assert!(c.leftover.is_empty(), "{:?}", c.leftover);
}

/// A duplicated unit line is withdrawn every time it appears: the `break` ends
/// the search for THIS line's action, and the next occurrence is the next
/// iteration of the line loop. Asserted rather than argued.
#[test]
fn every_occurrence_of_a_duplicated_unit_line_is_withdrawn() {
    let tmp = fixture("x");
    write(
        tmp.path(),
        "specs/001-beta/spec.md",
        &spec_doc(
            "001-beta",
            "establishes:\n  - { kind: file, path: \"rules/one.md\" }\n  - { kind: file, path: \"rules/one.md\" }\n  - \"keep.md\"\n",
            "Body.",
        ),
    );
    let mut e = retire_rules();
    e.units = vec![UnitAction {
        spec: "001-beta".into(),
        edge: "establishes".into(),
        action: UnitActionKind::Withdraw,
        to: None,
        acknowledge_approved: true,
    }];
    let c = compact(&cfg(), tmp.path(), &plan_with(e)).unwrap();
    let beta = c
        .files
        .iter()
        .find(|f| f.from_rel_path == "specs/001-beta/spec.md")
        .expect("the owning spec is rewritten");
    assert!(!beta.contents.contains("rules/one.md"), "{}", beta.contents);
    assert!(beta.contents.contains("keep.md"), "{}", beta.contents);
}

/// §3.7: an occurrence the rules could not account for is reported, not left in
/// silence. A form the plan never declared leaves the path in place, and the
/// first build emitted that quietly: the plan looked applied and the corpus
/// still named a path that was gone.
#[test]
fn an_occurrence_no_rule_covers_is_reported_as_leftover() {
    // The citation rule covers prose. A quoted YAML value is the `path` form,
    // which this plan does not declare.
    let tmp = fixture("x");
    write(tmp.path(), "docs/data.yml", "target: \"rules/one.md\"\n");
    let c = compact(&cfg(), tmp.path(), &plan_with(retire_rules())).unwrap();
    assert!(
        c.leftover
            .iter()
            .any(|l| l.rel_path == "docs/data.yml" && l.path == "rules/one.md"),
        "{:?}",
        c.leftover
    );
}

/// §3.5: every occurrence on a spared line is spared, and every one is
/// reported. A second path sharing a line with a negation was left out of the
/// report entirely.
#[test]
fn a_second_path_on_a_spared_line_is_reported_too() {
    let tmp = fixture("! test -e rules/one.md && test -e rules/two.md\n");
    let mut second = retire_rules();
    second.path = "rules/two.md".into();
    second.forms = [(
        "citation".to_string(),
        Some("`AGENTS.md` \"Two\"".to_string()),
    )]
    .into_iter()
    .collect();
    let plan = CompactPlan {
        retire: vec![retire_rules(), second],
        ..Default::default()
    };
    let c = compact(&cfg(), tmp.path(), &plan).unwrap();
    let on_note: Vec<_> = c
        .skipped
        .iter()
        .filter(|s| s.rel_path == "docs/note.md")
        .collect();
    assert_eq!(
        on_note.len(),
        2,
        "both occurrences on the spared line are reported: {:?}",
        c.skipped
    );
    assert!(on_note.iter().all(|s| s.clause == SkipClause::Negation));
}

/// §3.1: a glob rule substitutes the path PREFIX and leaves the wildcard, so a
/// replacement that is not a directory prefix produces `AGENTS.md*.md`.
#[test]
fn a_glob_replacement_that_is_not_a_prefix_is_refused() {
    let tmp = fixture("x");
    let mut e = retire_rules();
    e.path = "rules/".into();
    e.kind = RetireKind::Directory;
    e.forms = [
        ("citation".to_string(), Some("`AGENTS.md`".to_string())),
        ("glob".to_string(), Some("AGENTS.md".to_string())),
    ]
    .into_iter()
    .collect();
    let err = compact(&cfg(), tmp.path(), &plan_with(e)).unwrap_err();
    assert_eq!(err.exit_code(), 3, "{err}");
    assert!(format!("{err}").contains("directory prefix"), "{err}");
}

/// §3.5: each occurrence carries the clause that applies to IT. A line spared
/// by one entry's `historical_files` used to label every other entry's
/// occurrence on it with that clause, which is accurate about the outcome and
/// wrong about the reason.
#[test]
fn each_occurrence_on_a_spared_line_carries_its_own_clause() {
    let tmp = fixture("! test -e rules/one.md && test -e rules/two.md\n");
    let mut first = retire_rules();
    first.historical_files = vec!["docs/note.md".into()];
    let mut second = retire_rules();
    second.path = "rules/two.md".into();
    second.historical_files = vec![];
    second.forms = [(
        "citation".to_string(),
        Some("`AGENTS.md` \"Two\"".to_string()),
    )]
    .into_iter()
    .collect();
    let plan = CompactPlan {
        retire: vec![first, second],
        ..Default::default()
    };
    let c = compact(&cfg(), tmp.path(), &plan).unwrap();
    let clauses: Vec<SkipClause> = c
        .skipped
        .iter()
        .filter(|s| s.rel_path == "docs/note.md")
        .map(|s| s.clause)
        .collect();
    assert!(
        clauses.contains(&SkipClause::HistoricalFile),
        "the first entry's own clause: {clauses:?}"
    );
    assert!(
        clauses.contains(&SkipClause::Negation),
        "the second entry's own clause, not the first's: {clauses:?}"
    );
}

/// An empty path matches at every position and advances nowhere, and the
/// presence check passes it: `repo_root.join("")` is the repository root.
#[test]
fn an_empty_retired_path_is_refused() {
    let tmp = fixture("x");
    let mut e = retire_rules();
    e.path = String::new();
    let err = compact(&cfg(), tmp.path(), &plan_with(e)).unwrap_err();
    assert_eq!(err.exit_code(), 3, "{err}");
    assert!(format!("{err}").contains("empty"), "{err}");
}

/// §3.5: a negation naming a LONGER path is not an occurrence of the retired
/// one. Reading it as one produced a spurious skip record, and that record made
/// §3.7 treat the whole line as accounted for, so a real unaccounted occurrence
/// beside it went unreported.
#[test]
fn a_negation_on_a_longer_path_is_not_a_skip_of_the_retired_one() {
    let tmp = fixture("! test -e kit/rules/one.md\n");
    let c = compact(&cfg(), tmp.path(), &plan_with(retire_rules())).unwrap();
    assert!(
        c.skipped.iter().all(|s| s.rel_path != "docs/note.md"),
        "a longer path produced a skip record: {:?}",
        c.skipped
    );
    assert!(c.leftover.is_empty(), "{:?}", c.leftover);
}

/// Two unit actions can name the same line, and both fire. Stopping at the
/// first dropped the second in silence.
#[test]
fn every_unit_action_matching_a_line_fires() {
    let tmp = fixture("x");
    write(
        tmp.path(),
        "specs/001-beta/spec.md",
        &spec_doc(
            "001-beta",
            // `paths:` sugar puts two units on ONE line, which is the shape a
            // second action on the same line actually takes in this grammar.
            "extends:\n  - { spec: \"000-alpha\", paths: [\"rules/one.md\", \"rules/two.md\"] }\n",
            "Body.",
        ),
    );
    let mut first = retire_rules();
    first.units = vec![UnitAction {
        spec: "001-beta".into(),
        edge: "extends".into(),
        action: UnitActionKind::Retarget,
        to: Some("AGENTS.md".into()),
        acknowledge_approved: true,
    }];
    let mut second = retire_rules();
    second.path = "rules/two.md".into();
    second.forms = [(
        "citation".to_string(),
        Some("`AGENTS.md` \"Two\"".to_string()),
    )]
    .into_iter()
    .collect();
    second.units = vec![UnitAction {
        spec: "001-beta".into(),
        edge: "extends".into(),
        action: UnitActionKind::Retarget,
        to: Some("CLAUDE.md".into()),
        acknowledge_approved: true,
    }];
    let plan = CompactPlan {
        retire: vec![first, second],
        ..Default::default()
    };
    let c = compact(&cfg(), tmp.path(), &plan).unwrap();
    let beta = c
        .files
        .iter()
        .find(|f| f.from_rel_path == "specs/001-beta/spec.md")
        .expect("the owning spec is rewritten");
    assert!(beta.contents.contains("AGENTS.md"), "{}", beta.contents);
    assert!(
        beta.contents.contains("CLAUDE.md"),
        "the second action fired too:\n{}",
        beta.contents
    );
}

/// A glob rule must read the same boundary as everything else: `rules/*` is a
/// substring of `extra-rules/*.md`, and because the glob branch returns early
/// the citation rule that would have handled the line correctly is never
/// reached.
#[test]
fn a_glob_rule_does_not_fire_on_a_longer_paths_glob() {
    let tmp = fixture("Patterns: `extra-rules/*.md` stays.\n");
    let mut e = retire_rules();
    e.path = "rules/".into();
    e.kind = RetireKind::Directory;
    e.forms = [
        ("citation".to_string(), Some("`AGENTS.md`".to_string())),
        ("glob".to_string(), Some("AGENTS/".to_string())),
    ]
    .into_iter()
    .collect();
    let c = compact(&cfg(), tmp.path(), &plan_with(e)).unwrap();
    let out = c
        .files
        .iter()
        .find(|f| f.from_rel_path == "docs/note.md")
        .map(|f| f.contents.clone())
        .unwrap_or_default();
    assert!(
        out.is_empty() || out.contains("extra-rules/*.md"),
        "a longer path's glob was rewritten: {out}"
    );
}

/// §3.3: a retarget replaces the unit it names, not a sibling that starts with
/// it. `String::replace` rewrote `rules/` inside `rules/one.md` when both sat on
/// one line, producing a path the plan named nowhere.
#[test]
fn a_retarget_does_not_rewrite_a_sibling_sharing_the_prefix() {
    let tmp = fixture("x");
    write(
        tmp.path(),
        "specs/001-beta/spec.md",
        &spec_doc(
            "001-beta",
            "extends:\n  - { spec: \"000-alpha\", paths: [\"rules/\", \"rules/one.md\"] }\n",
            "Body.",
        ),
    );
    let mut e = retire_rules();
    e.path = "rules/".into();
    e.kind = RetireKind::Directory;
    e.forms = [("citation".to_string(), Some("`AGENTS.md`".to_string()))]
        .into_iter()
        .collect();
    e.units = vec![UnitAction {
        spec: "001-beta".into(),
        edge: "extends".into(),
        action: UnitActionKind::Retarget,
        to: Some("AGENTS.md".into()),
        acknowledge_approved: true,
    }];
    let c = compact(&cfg(), tmp.path(), &plan_with(e)).unwrap();
    let beta = c
        .files
        .iter()
        .find(|f| f.from_rel_path == "specs/001-beta/spec.md")
        .expect("the owning spec is rewritten");
    assert!(
        beta.contents.contains("rules/one.md"),
        "the sibling was rewritten:\n{}",
        beta.contents
    );
    assert!(beta.contents.contains("AGENTS.md"), "{}", beta.contents);
}

/// §3.1: a path that leaves the corpus is not a path this plan may name.
/// `repo_root.join("/etc/passwd")` is `/etc/passwd`, so the existence check
/// passed it and the string was then matched across every scanned file.
#[test]
fn a_retired_path_outside_the_corpus_is_refused() {
    let tmp = fixture("x");
    for outside in ["/etc/passwd", "../elsewhere.md"] {
        let mut e = retire_rules();
        e.path = outside.into();
        let err = compact(&cfg(), tmp.path(), &plan_with(e)).unwrap_err();
        assert_eq!(err.exit_code(), 3, "{outside}: {err}");
        assert!(
            format!("{err}").contains("inside the corpus"),
            "{outside}: {err}"
        );
    }
}

/// §3.3: an empty `to` is the same mistake as an absent one; it writes a unit
/// no corpus can resolve.
#[test]
fn a_retarget_with_an_empty_target_is_refused() {
    let tmp = fixture("x");
    let mut e = retire_rules();
    e.units = vec![UnitAction {
        spec: "000-alpha".into(),
        edge: "establishes".into(),
        action: UnitActionKind::Retarget,
        to: Some("   ".into()),
        acknowledge_approved: true,
    }];
    let err = compact(&cfg(), tmp.path(), &plan_with(e)).unwrap_err();
    assert_eq!(err.exit_code(), 3, "{err}");
    assert!(format!("{err}").contains("no `to`"), "{err}");
}

/// §3.1: a directory path without its trailing slash is a WORD. `rules` matches
/// the English word in any sentence, which §3.7 then reports as an unaccounted
/// occurrence and refuses the run over.
#[test]
fn a_directory_entry_without_a_trailing_slash_is_refused() {
    let tmp = fixture("x");
    let mut e = retire_rules();
    e.path = "rules".into();
    e.kind = RetireKind::Directory;
    let err = compact(&cfg(), tmp.path(), &plan_with(e)).unwrap_err();
    assert_eq!(err.exit_code(), 3, "{err}");
    assert!(format!("{err}").contains("does not end with"), "{err}");
}

/// §3.5: an entry acts on occurrences the SOURCE line had. One entry's
/// replacement text can contain another entry's retired path, and sparing that
/// synthetic occurrence reports a reason that was never true of the file.
#[test]
fn an_occurrence_another_entrys_replacement_created_is_not_spared() {
    // The first entry rewrites `rules/one.md` to text naming `rules/two.md`.
    let tmp = fixture("See rules/one.md.\n");
    let mut first = retire_rules();
    first.forms = [(
        "citation".to_string(),
        Some("`rules/two.md` now".to_string()),
    )]
    .into_iter()
    .collect();
    let mut second = retire_rules();
    second.path = "rules/two.md".into();
    second.forms = [(
        "citation".to_string(),
        Some("`AGENTS.md` \"Two\"".to_string()),
    )]
    .into_iter()
    .collect();
    let plan = CompactPlan {
        retire: vec![first, second],
        ..Default::default()
    };
    let c = compact(&cfg(), tmp.path(), &plan).unwrap();
    let out = c
        .files
        .iter()
        .find(|f| f.from_rel_path == "docs/note.md")
        .map(|f| f.contents.clone())
        .unwrap_or_default();
    // The synthetic occurrence is left exactly as the first entry wrote it.
    assert!(out.contains("`rules/two.md` now"), "{out}");
    assert!(
        c.skipped.iter().all(|s| s.rel_path != "docs/note.md"),
        "a synthetic occurrence was reported as spared: {:?}",
        c.skipped
    );
    // Nor reported as unaccounted for. The first version of this test asserted
    // only the two lines above, and the run would have exited 1 on a corpus it
    // had handled exactly as the plan asked.
    assert!(
        c.leftover.iter().all(|l| l.rel_path != "docs/note.md"),
        "a synthetic occurrence was reported as leftover: {:?}",
        c.leftover
    );
}

/// §3.2 and §3.7 have to agree about what an occurrence IS. A markdown link
/// label was detected by the scan and rewritten by no form, so the run refused a
/// corpus no rule could have repaired. A label is a citation.
#[test]
fn a_markdown_link_label_is_a_citation_and_is_rewritten() {
    let tmp = fixture("See [rules/one.md] for the rule.\n");
    let c = compact(&cfg(), tmp.path(), &plan_with(retire_rules())).unwrap();
    let out = c
        .files
        .iter()
        .find(|f| f.from_rel_path == "docs/note.md")
        .map(|f| f.contents.clone())
        .unwrap_or_default();
    assert!(out.contains("[`AGENTS.md` \"Rules\"]"), "{out}");
    assert!(c.leftover.is_empty(), "{:?}", c.leftover);
}

/// §3.1: a `./` prefix passes every guard and then matches nothing, because the
/// corpus writes the path bare. The run would rewrite nothing and report every
/// bare occurrence as unaccounted for.
#[test]
fn a_dot_slash_prefixed_path_is_refused() {
    let tmp = fixture("x");
    let mut e = retire_rules();
    e.path = "./rules/one.md".into();
    let err = compact(&cfg(), tmp.path(), &plan_with(e)).unwrap_err();
    assert_eq!(err.exit_code(), 3, "{err}");
    assert!(format!("{err}").contains("./"), "{err}");
}

/// §3.3: the `path` form replaces through the same function as every other
/// form, so a replacement that itself contains the quoted path does not fire a
/// second time on the same pass.
#[test]
fn a_path_form_replacement_containing_the_path_fires_once() {
    let tmp = fixture("x");
    write(tmp.path(), "docs/data.yml", "target: \"rules/one.md\"\n");
    let mut e = retire_rules();
    e.forms = [
        ("citation".to_string(), Some("`AGENTS.md`".to_string())),
        ("path".to_string(), Some("kept/rules/one.md".to_string())),
    ]
    .into_iter()
    .collect();
    let c = compact(&cfg(), tmp.path(), &plan_with(e)).unwrap();
    let out = c
        .files
        .iter()
        .find(|f| f.from_rel_path == "docs/data.yml")
        .map(|f| f.contents.clone())
        .unwrap_or_default();
    assert!(out.contains("kept/rules/one.md"), "{out}");
    assert!(!out.contains("kept/kept/"), "applied twice: {out}");
}

/// §3.7: a glob deletion removes a line, so every later skip record's source
/// line number runs ahead of that line's position in the output. Matching a
/// spare by coordinate then failed, and a correctly spared occurrence was
/// reported as unaccounted for.
#[test]
fn a_glob_deletion_does_not_shift_a_later_skip_out_of_alignment() {
    let tmp = fixture("x");
    write(
        tmp.path(),
        "docs/list.toml",
        "patterns = [\n  \"rules/*.md\",\n]\n! test -e rules/one.md\n",
    );
    let mut e = retire_rules();
    e.path = "rules/".into();
    e.kind = RetireKind::Directory;
    e.forms = [
        ("citation".to_string(), Some("`AGENTS.md`".to_string())),
        ("glob".to_string(), None),
    ]
    .into_iter()
    .collect();
    let mut second = retire_rules();
    second.forms = [(
        "citation".to_string(),
        Some("`AGENTS.md` \"One\"".to_string()),
    )]
    .into_iter()
    .collect();
    let plan = CompactPlan {
        retire: vec![e, second],
        ..Default::default()
    };
    let c = compact(&cfg(), tmp.path(), &plan).unwrap();
    assert!(
        c.skipped
            .iter()
            .any(|s| s.rel_path == "docs/list.toml" && s.clause == SkipClause::Negation),
        "the negation is spared: {:?}",
        c.skipped
    );
    // Scoped to the file under test: `000-alpha`'s frontmatter claims `rules/`
    // and this plan declares neither a `path` form nor a unit action for it, so
    // that IS an unaccounted occurrence and §3.7 is right to report it.
    assert!(
        c.leftover.iter().all(|l| l.rel_path != "docs/list.toml"),
        "a spared line was reported as unaccounted for: {:?}",
        c.leftover
    );
}

/// §3.5: an empty `historical_sections` keyword matches every heading, because
/// `"anything".contains("")` is true, so every occurrence in every file would be
/// spared and the retirement would do nothing without saying so.
#[test]
fn an_empty_historical_entry_is_refused() {
    let tmp = fixture("x");
    for (sections, files) in [
        (vec!["".to_string()], vec![]),
        (vec![], vec!["  ".to_string()]),
    ] {
        let mut e = retire_rules();
        e.historical_sections = sections;
        e.historical_files = files;
        let err = compact(&cfg(), tmp.path(), &plan_with(e)).unwrap_err();
        assert_eq!(err.exit_code(), 3, "{err}");
        assert!(format!("{err}").contains("empty"), "{err}");
    }
}

/// A glob REPLACEMENT does not take the whole line, so a citation sharing it is
/// still rewritten. Returning early left the citation behind for §3.7 to refuse,
/// on a corpus the rules could in fact repair.
#[test]
fn a_glob_replacement_does_not_strand_a_citation_on_the_same_line() {
    let tmp = fixture("x");
    write(
        tmp.path(),
        "docs/list.toml",
        "patterns = [\"rules/*.md\"]  # see rules/one.md\n",
    );
    let mut e = retire_rules();
    e.path = "rules/".into();
    e.kind = RetireKind::Directory;
    e.forms = [
        ("citation".to_string(), Some("`AGENTS.md`".to_string())),
        ("glob".to_string(), Some("kept/".to_string())),
    ]
    .into_iter()
    .collect();
    let mut second = retire_rules();
    second.forms = [(
        "citation".to_string(),
        Some("`AGENTS.md` \"One\"".to_string()),
    )]
    .into_iter()
    .collect();
    let plan = CompactPlan {
        retire: vec![e, second],
        ..Default::default()
    };
    let c = compact(&cfg(), tmp.path(), &plan).unwrap();
    let out = c
        .files
        .iter()
        .find(|f| f.from_rel_path == "docs/list.toml")
        .map(|f| f.contents.clone())
        .unwrap_or_default();
    assert!(out.contains("kept/*.md"), "the glob is replaced: {out}");
    assert!(
        c.leftover.iter().all(|l| l.rel_path != "docs/list.toml"),
        "a citation was stranded: {:?}",
        c.leftover
    );
}

/// §3.2: the left boundary is read against the ORIGINAL line. The prose
/// replacer walked a shrinking slice and passed a relative offset, so a match
/// starting exactly where the previous one ended took the `at == 0`
/// short-circuit and skipped the boundary test entirely.
#[test]
fn a_second_occurrence_abutting_the_first_still_reads_its_left_boundary() {
    // One valid occurrence, so the line is processed at all, then an ADJACENT
    // pair. The first of the pair is refused on its right boundary (a letter
    // follows) and the walk resumes exactly at the second, which therefore
    // begins at offset 0 of the remaining slice. Its real left neighbour is
    // `d`, a path character, so it is part of a longer token and must not be
    // replaced either.
    let tmp = fixture("See rules/one.md and rules/one.mdrules/one.md here.\n");
    let c = compact(&cfg(), tmp.path(), &plan_with(retire_rules())).unwrap();
    let out = c
        .files
        .iter()
        .find(|f| f.from_rel_path == "docs/note.md")
        .map(|f| f.contents.clone())
        .unwrap_or_default();
    // The genuine citation is rewritten; the adjacent pair is left whole.
    assert!(out.contains("`AGENTS.md` \"Rules\" and"), "{out:?}");
    assert!(
        out.contains("rules/one.mdrules/one.md"),
        "an abutting occurrence was rewritten: {out:?}"
    );
}

/// §3.3: an edge is a key that opens a LIST. A block scalar such as
/// `summary: >` opens no list, and reading it as one set the current edge to
/// `summary`, so its indented continuation lines were matched against unit
/// actions and could be withdrawn outright.
#[test]
fn a_block_scalar_key_does_not_open_an_edge_list() {
    let tmp = fixture("x");
    write(
        tmp.path(),
        "specs/001-beta/spec.md",
        "---\nid: \"001-beta\"\ntitle: \"T\"\nstatus: approved\ncreated: \"2026-06-09\"\n\
         implementation: complete\nsummary: >\n  A summary that mentions rules/one.md in its\n\
         \x20 continuation line.\n---\n\n# 001-beta\n\nBody.\n",
    );
    let mut e = retire_rules();
    e.units = vec![UnitAction {
        spec: "001-beta".into(),
        edge: "summary".into(),
        action: UnitActionKind::Withdraw,
        to: None,
        acknowledge_approved: true,
    }];
    let c = compact(&cfg(), tmp.path(), &plan_with(e)).unwrap();
    let beta = c
        .files
        .iter()
        .find(|f| f.from_rel_path == "specs/001-beta/spec.md")
        .map(|f| f.contents.clone())
        .unwrap_or_default();
    assert!(
        beta.is_empty() || beta.contains("mentions"),
        "a block-scalar continuation was withdrawn as a unit: {beta:?}"
    );
}

/// A withdrawal on a line wins over a retarget on the same line, and the report
/// says so once. Recording the retarget and then discovering the withdrawal
/// left a rewrite record for a line that never reached the output.
#[test]
fn a_withdrawal_beats_a_retarget_on_the_same_line_and_is_recorded_once() {
    let tmp = fixture("x");
    write(
        tmp.path(),
        "specs/001-beta/spec.md",
        &spec_doc(
            "001-beta",
            "extends:\n  - { spec: \"000-alpha\", paths: [\"rules/\", \"rules/one.md\"] }\n",
            "Body.",
        ),
    );
    let mut first = retire_rules();
    first.path = "rules/".into();
    first.kind = RetireKind::Directory;
    first.forms = [("citation".to_string(), Some("`AGENTS.md`".to_string()))]
        .into_iter()
        .collect();
    first.units = vec![UnitAction {
        spec: "001-beta".into(),
        edge: "extends".into(),
        action: UnitActionKind::Retarget,
        to: Some("AGENTS.md".into()),
        acknowledge_approved: true,
    }];
    let mut second = retire_rules();
    second.units = vec![UnitAction {
        spec: "001-beta".into(),
        edge: "extends".into(),
        action: UnitActionKind::Withdraw,
        to: None,
        acknowledge_approved: true,
    }];
    let plan = CompactPlan {
        retire: vec![first, second],
        ..Default::default()
    };
    let c = compact(&cfg(), tmp.path(), &plan).unwrap();
    let beta = c
        .files
        .iter()
        .find(|f| f.from_rel_path == "specs/001-beta/spec.md")
        .expect("the owning spec is rewritten");
    assert!(!beta.contents.contains("rules/"), "{}", beta.contents);
    let on_line: Vec<_> = c
        .rewrites
        .iter()
        .filter(|f| f.rel_path == "specs/001-beta/spec.md")
        .flat_map(|f| f.rewrites.iter())
        .filter(|r| r.form == Form::RetiredPath)
        .collect();
    assert_eq!(
        on_line.len(),
        1,
        "one record for a line that left once: {on_line:?}"
    );
}

/// The emptied-key scan uses the same key rule as the walk.
///
/// This asserts the rule rather than reproducing a defect: the shape the review
/// described is not reachable in a corpus that compiles. A quoted scalar ends
/// with `"`, and an unquoted one ending in `:` fails to parse, so no valid
/// frontmatter reaches the `ends_with(':')` predicate the scan used to carry.
/// The fix removes an undeclared assumption, and this keeps a colon-bearing
/// scalar beside a withdrawal so a future key rule cannot quietly drop it.
#[test]
fn a_scalar_whose_value_ends_in_a_colon_is_not_an_emptied_key() {
    let tmp = fixture("x");
    write(
        tmp.path(),
        "specs/001-beta/spec.md",
        "---\nid: \"001-beta\"\ntitle: \"T\"\nstatus: approved\ncreated: \"2026-06-09\"\n\
         implementation: complete\nsummary: \"See rule:\"\nestablishes:\n\
         \x20 - { kind: file, path: \"rules/one.md\" }\n---\n\n# 001-beta\n\nBody.\n",
    );
    let mut e = retire_rules();
    e.units = vec![UnitAction {
        spec: "001-beta".into(),
        edge: "establishes".into(),
        action: UnitActionKind::Withdraw,
        to: None,
        acknowledge_approved: true,
    }];
    let c = compact(&cfg(), tmp.path(), &plan_with(e)).unwrap();
    let beta = c
        .files
        .iter()
        .find(|f| f.from_rel_path == "specs/001-beta/spec.md")
        .expect("the owning spec is rewritten");
    assert!(
        beta.contents.contains("summary: \"See rule:\""),
        "a scalar was dropped as an emptied key:\n{}",
        beta.contents
    );
    assert!(!beta.contents.contains("establishes:"), "{}", beta.contents);
}

/// A historical file is compared literally against corpus-relative paths, so a
/// traversal spelling excludes nothing and says nothing.
#[test]
fn a_historical_file_outside_the_corpus_is_refused() {
    let tmp = fixture("x");
    for bad in ["../outside.md", "/etc/passwd", "./docs/note.md"] {
        let mut e = retire_rules();
        e.historical_files = vec![bad.to_string()];
        let err = compact(&cfg(), tmp.path(), &plan_with(e)).unwrap_err();
        assert_eq!(err.exit_code(), 3, "{bad}: {err}");
        assert!(format!("{err}").contains("historical"), "{bad}: {err}");
    }
}

/// Every form reads the ORIGINAL line, so no form re-reads what another wrote.
///
/// This asserts the rule; it does not reproduce a defect. The sequential form
/// the rule replaced could in principle re-read a glob replacement that names
/// the retired path, and no replacement text was found that actually does: any
/// prefix ending in a path character blocks the prose boundary, and a suffix
/// does the same on the right. The structure is what guarantees it rather than
/// that coincidence, which is why the rule is worth having anyway.
#[test]
fn a_glob_replacement_naming_the_path_is_not_rewritten_again() {
    let tmp = fixture("x");
    write(
        tmp.path(),
        "docs/list.toml",
        "patterns = [\"rules/*.md\"]\n",
    );
    let mut e = retire_rules();
    e.path = "rules/".into();
    e.kind = RetireKind::Directory;
    e.forms = [
        ("citation".to_string(), Some("AGENTS/".to_string())),
        // The replacement text itself names the retired path.
        ("glob".to_string(), Some("old-rules/".to_string())),
    ]
    .into_iter()
    .collect();
    let c = compact(&cfg(), tmp.path(), &plan_with(e)).unwrap();
    let out = c
        .files
        .iter()
        .find(|f| f.from_rel_path == "docs/list.toml")
        .map(|f| f.contents.clone())
        .unwrap_or_default();
    assert!(
        out.contains("old-rules/*.md"),
        "the glob replacement was rewritten again: {out:?}"
    );
}

/// §3.5 and D-5: where an entry has no clause of its own, the line's clause is
/// the label. It is the best available one rather than a claim about which
/// marker applies to which path, and the fallback was documented but never
/// exercised.
#[test]
fn an_entry_with_no_clause_of_its_own_takes_the_lines() {
    let tmp = fixture("A line naming rules/one.md and rules/two.md.\n");
    let mut first = retire_rules();
    first.historical_files = vec!["docs/note.md".into()];
    let mut second = retire_rules();
    second.path = "rules/two.md".into();
    second.historical_files = vec![];
    second.forms = [(
        "citation".to_string(),
        Some("`AGENTS.md` \"Two\"".to_string()),
    )]
    .into_iter()
    .collect();
    let plan = CompactPlan {
        retire: vec![first, second],
        ..Default::default()
    };
    let c = compact(&cfg(), tmp.path(), &plan).unwrap();
    let on_note: Vec<_> = c
        .skipped
        .iter()
        .filter(|s| s.rel_path == "docs/note.md")
        .collect();
    assert_eq!(on_note.len(), 2, "both occurrences reported: {on_note:?}");
    // The second entry has no historical marker of its own and takes the
    // line's, which is the documented fallback.
    assert!(
        on_note
            .iter()
            .all(|s| s.clause == SkipClause::HistoricalFile),
        "{on_note:?}"
    );
}

/// §3.5: a section contains everything nested in it. Tracking only the last
/// heading seen meant a sub-heading replaced its parent, so a keyword naming the
/// parent stopped matching the moment a deeper heading appeared and the
/// occurrences below it were refused as unaccounted for.
#[test]
fn a_historical_section_covers_its_sub_headings() {
    let tmp = fixture(
        "## History\n\n### Prior paths\n\n`rules/one.md` was removed here.\n\n## Live\n\nSee `rules/one.md`.\n",
    );
    let mut e = retire_rules();
    e.historical_sections = vec!["History".into()];
    let c = compact(&cfg(), tmp.path(), &plan_with(e)).unwrap();
    assert!(
        c.skipped
            .iter()
            .any(|s| s.rel_path == "docs/note.md" && s.clause == SkipClause::HistoricalSection),
        "the nested occurrence is spared: {:?}",
        c.skipped
    );
    assert!(
        c.leftover.iter().all(|l| l.rel_path != "docs/note.md"),
        "a nested occurrence was refused: {:?}",
        c.leftover
    );
    // And the section ENDS: the occurrence under `## Live` is rewritten.
    let out = c
        .files
        .iter()
        .find(|f| f.from_rel_path == "docs/note.md")
        .map(|f| f.contents.clone())
        .unwrap_or_default();
    assert!(out.contains("See `AGENTS.md` \"Rules\"."), "{out}");
}

/// Every glob occurrence on a line is replaced. One array can carry the same
/// pattern twice, and rewriting only the first left the second for §3.7 to
/// refuse, on a corpus the rules could repair.
#[test]
fn every_glob_occurrence_on_a_line_is_replaced() {
    let tmp = fixture("x");
    write(
        tmp.path(),
        "docs/list.toml",
        "patterns = [\"rules/*.md\", \"rules/*.toml\"]\n",
    );
    let mut e = retire_rules();
    e.path = "rules/".into();
    e.kind = RetireKind::Directory;
    e.forms = [
        ("citation".to_string(), Some("`AGENTS.md`".to_string())),
        ("glob".to_string(), Some("kept/".to_string())),
    ]
    .into_iter()
    .collect();
    let c = compact(&cfg(), tmp.path(), &plan_with(e)).unwrap();
    let out = c
        .files
        .iter()
        .find(|f| f.from_rel_path == "docs/list.toml")
        .map(|f| f.contents.clone())
        .unwrap_or_default();
    assert!(out.contains("kept/*.md"), "{out}");
    assert!(out.contains("kept/*.toml"), "the second glob: {out}");
    assert!(
        c.leftover.iter().all(|l| l.rel_path != "docs/list.toml"),
        "{:?}",
        c.leftover
    );
}

/// The backticked citation is delimited by its backticks, and a `.` after the
/// closing one is sentence punctuation rather than a path character. The first
/// boundary test written for this used `is_path_char` on both sides and refused
/// every citation that ended a sentence.
#[test]
fn a_backticked_citation_ending_a_sentence_is_rewritten() {
    let tmp = fixture("See `rules/one.md`. And `rules/one.md`, again.\n");
    let c = compact(&cfg(), tmp.path(), &plan_with(retire_rules())).unwrap();
    let out = c
        .files
        .iter()
        .find(|f| f.from_rel_path == "docs/note.md")
        .map(|f| f.contents.clone())
        .unwrap_or_default();
    assert!(out.contains("`AGENTS.md` \"Rules\"."), "{out}");
    assert!(out.contains("`AGENTS.md` \"Rules\","), "{out}");
    assert!(c.leftover.is_empty(), "{:?}", c.leftover);
}

// ── the plan file ────────────────────────────────────────────────────────────

#[test]
fn a_retire_plan_parses_from_yaml() {
    let p = parse_plan(
        "retire:\n  - path: \"rules/one.md\"\n    kind: file\n    forms:\n      citation: \"AGENTS.md\"\n      glob: ~\n",
    )
    .unwrap();
    assert_eq!(p.retire.len(), 1);
    assert_eq!(p.retire[0].forms["glob"], None);
    assert_eq!(p.retire[0].forms["citation"].as_deref(), Some("AGENTS.md"));
}

#[test]
fn a_glob_may_be_removed_but_a_citation_may_not() {
    let tmp = fixture("x");
    let mut e = retire_rules();
    e.forms.insert("citation".into(), None);
    let err = compact(&cfg(), tmp.path(), &plan_with(e)).unwrap_err();
    assert_eq!(err.exit_code(), 3, "{err}");
    assert!(format!("{err}").contains("glob"), "{err}");
}

#[test]
fn a_glob_pattern_that_matches_only_the_retired_path_is_removed() {
    let tmp = fixture("x");
    write(
        tmp.path(),
        "spec-spine.toml",
        "[index]\nextra_hashed_inputs = [\n  \"AGENTS.md\",\n  \"rules/*.md\",\n]\n",
    );
    let mut e = retire_rules();
    e.path = "rules/".into();
    e.kind = RetireKind::Directory;
    e.forms = [
        ("citation".to_string(), Some("`AGENTS.md`".to_string())),
        ("glob".to_string(), None),
    ]
    .into_iter()
    .collect();
    let c = compact(&cfg(), tmp.path(), &plan_with(e)).unwrap();
    let toml = prose_of(&c, "spec-spine.toml");
    assert!(!toml.contains("rules/*.md"), "{toml}");
    assert!(toml.contains("AGENTS.md"), "{toml}");
}
