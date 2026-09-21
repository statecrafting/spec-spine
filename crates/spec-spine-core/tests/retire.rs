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
    CompactPlan, RetireEntry, RetireKind, SkipClause, UnitAction, UnitActionKind, compact,
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
