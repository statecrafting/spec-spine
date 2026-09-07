//! Declared-acceptance tests (spec 049).
//!
//! Spec 049 §3.2's table is the contract these fixtures assert, one test per
//! row. The grammar is ported from `scripts/verify-spec.sh`, which carries no
//! tests in any of the four repositories that hold a copy of it, so these are
//! the first assertions the parse has ever had.
//!
//! Nothing here runs a command: the engine returns a plan and the CLI executes
//! it (spec 049 §3.1), so the whole grammar is testable over strings.

use std::fs;
use std::path::Path;

use spec_spine_core::{plan_from_markdown, verify_plan, verify_plan_json};
use spec_spine_types::{Config, Error, VerifyPlan, load_config};

/// Wrap a `## Verification` body in a minimal spec document.
fn doc(section: &str) -> String {
    format!(
        "---\nid: \"049-x\"\n---\n\n# 049: x\n\n## 1. Purpose\n\np\n\n## Verification\n\n{section}\n"
    )
}

fn commands(section: &str) -> Vec<String> {
    plan_from_markdown("049-x", &doc(section)).commands
}

// --- §3.2: the section heading -------------------------------------------

#[test]
fn unnumbered_heading_is_the_section() {
    let md = "# t\n\n## Verification\n\n```verify:cli\ntrue\n```\n";
    assert_eq!(plan_from_markdown("a", md).commands, ["true"]);
}

#[test]
fn numbered_heading_is_the_same_section() {
    let md = "# t\n\n## 5. Verification\n\n```verify:cli\ntrue\n```\n";
    assert_eq!(plan_from_markdown("a", md).commands, ["true"]);
}

#[test]
fn a_multi_digit_ordinal_still_matches() {
    let md = "# t\n\n## 12. Verification\n\n```verify:cli\ntrue\n```\n";
    assert_eq!(plan_from_markdown("a", md).commands, ["true"]);
}

#[test]
fn no_verification_heading_is_not_declared() {
    let md = "# t\n\n## Behavior\n\n```verify:cli\ntrue\n```\n";
    let plan = plan_from_markdown("a", md);
    assert!(plan.commands.is_empty(), "a fence outside the section");
    assert!(!plan.is_declared());
}

#[test]
fn the_section_ends_at_the_next_h2() {
    let md =
        "## Verification\n\n```verify:cli\nfirst\n```\n\n## Notes\n\n```verify:cli\nsecond\n```\n";
    assert_eq!(plan_from_markdown("a", md).commands, ["first"]);
}

#[test]
fn a_deeper_heading_does_not_end_the_section() {
    let md = "## Verification\n\n### Detail\n\n```verify:cli\ntrue\n```\n";
    assert_eq!(plan_from_markdown("a", md).commands, ["true"]);
}

#[test]
fn a_similar_heading_is_not_the_section() {
    for h in [
        "## Verification notes",
        "## Verifications",
        "## verification",
    ] {
        let md = format!("{h}\n\n```verify:cli\ntrue\n```\n");
        assert!(
            plan_from_markdown("a", &md).commands.is_empty(),
            "matched {h}"
        );
    }
}

// --- §3.2: what counts as a command --------------------------------------

#[test]
fn each_body_line_is_a_command_in_order() {
    assert_eq!(
        commands("```verify:cli\nalpha\nbeta\ngamma\n```"),
        ["alpha", "beta", "gamma"]
    );
}

#[test]
fn blank_lines_are_not_commands() {
    assert_eq!(
        commands("```verify:cli\nalpha\n\n\nbeta\n```"),
        ["alpha", "beta"]
    );
}

#[test]
fn comments_are_not_commands() {
    assert_eq!(
        commands("```verify:cli\n# a note\nalpha\n   # indented note\nbeta\n```"),
        ["alpha", "beta"]
    );
}

#[test]
fn whitespace_is_trimmed() {
    assert_eq!(commands("```verify:cli\n   alpha   \n```"), ["alpha"]);
}

#[test]
fn a_command_may_contain_a_hash_that_is_not_a_comment() {
    assert_eq!(
        commands("```verify:cli\ngrep '#!' install.sh\n```"),
        ["grep '#!' install.sh"]
    );
}

#[test]
fn multiple_cli_fences_concatenate_in_document_order() {
    assert_eq!(
        commands("```verify:cli\nalpha\n```\n\nprose\n\n```verify:cli\nbeta\n```"),
        ["alpha", "beta"]
    );
}

#[test]
fn a_section_with_no_cli_fence_is_not_declared() {
    let plan = plan_from_markdown("a", &doc("- a prose bullet\n- another"));
    assert!(!plan.is_declared());
    assert!(plan.commands.is_empty());
}

#[test]
fn an_empty_cli_fence_is_not_declared() {
    assert!(!plan_from_markdown("a", &doc("```verify:cli\n```")).is_declared());
}

// --- §3.2: declined fence tags -------------------------------------------

#[test]
fn other_tags_are_counted_and_not_run() {
    let plan = plan_from_markdown(
        "a",
        &doc("```verify:browser\nclick\n```\n\n```verify:cli\nalpha\n```"),
    );
    assert_eq!(plan.commands, ["alpha"]);
    assert_eq!(plan.skipped.len(), 1);
    assert_eq!(plan.skipped[0].tag, "verify:browser");
    assert_eq!(plan.skipped[0].count, 1);
}

#[test]
fn repeated_declined_tags_are_counted_together_and_sorted() {
    let plan = plan_from_markdown(
        "a",
        &doc("```verify:browser\nx\n```\n\n```rust\ny\n```\n\n```verify:browser\nz\n```"),
    );
    assert!(plan.commands.is_empty());
    let tags: Vec<_> = plan.skipped.iter().map(|s| (&*s.tag, s.count)).collect();
    assert_eq!(tags, [("rust", 1), ("verify:browser", 2)]);
}

#[test]
fn an_untagged_fence_is_neither_run_nor_counted() {
    let plan = plan_from_markdown("a", &doc("```\nplain\n```"));
    assert!(plan.commands.is_empty());
    assert!(
        plan.skipped.is_empty(),
        "a bare fence is prose, not declined work"
    );
}

// --- determinism ----------------------------------------------------------

#[test]
fn the_plan_is_a_pure_function_of_the_markdown() {
    let md = doc("```verify:browser\nb\n```\n\n```verify:cli\n# c\nalpha\n\nbeta\n```");
    let first = plan_from_markdown("a", &md);
    for _ in 0..8 {
        assert_eq!(plan_from_markdown("a", &md), first);
    }
}

// --- the on-disk entry point ---------------------------------------------

fn corpus(specs: &[(&str, &str)]) -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    for (id, md) in specs {
        let dir = tmp.path().join("specs").join(id);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("spec.md"), md).unwrap();
    }
    tmp
}

fn cfg() -> Config {
    load_config("").unwrap()
}

fn plan_at(root: &Path, id: &str) -> Result<VerifyPlan, Error> {
    verify_plan(&cfg(), root, id)
}

#[test]
fn the_full_id_resolves() {
    let t = corpus(&[("049-slug", &doc("```verify:cli\nalpha\n```"))]);
    let plan = plan_at(t.path(), "049-slug").unwrap();
    assert_eq!(plan.spec_id, "049-slug");
    assert_eq!(plan.commands, ["alpha"]);
}

#[test]
fn the_short_id_resolves_and_the_plan_names_the_full_one() {
    let t = corpus(&[("049-slug", &doc("```verify:cli\nalpha\n```"))]);
    let plan = plan_at(t.path(), "049").unwrap();
    assert_eq!(plan.spec_id, "049-slug", "spec 016 short-id resolution");
}

#[test]
fn a_missing_spec_is_not_found_never_stale() {
    let t = corpus(&[("049-slug", &doc("```verify:cli\nalpha\n```"))]);
    let err = plan_at(t.path(), "999").unwrap_err();
    assert!(matches!(err, Error::NotFound(_)), "got {err:?}");
    // Spec 049 §3.3: exit 2 is reserved for staleness across every verb.
    assert_eq!(err.exit_code(), 1);
}

#[test]
fn an_ambiguous_short_id_is_refused_not_guessed() {
    let t = corpus(&[
        ("049-one", &doc("```verify:cli\na\n```")),
        ("049-two", &doc("```verify:cli\nb\n```")),
    ]);
    let err = plan_at(t.path(), "049").unwrap_err();
    assert!(matches!(err, Error::NotFound(_)), "got {err:?}");
    assert!(err.to_string().contains("ambiguous"), "{err}");
}

#[test]
fn the_json_facade_returns_the_same_plan() {
    let t = corpus(&[("049-slug", &doc("```verify:cli\nalpha\n```"))]);
    let cfg_json = serde_json::to_string(&cfg()).unwrap();
    let out = verify_plan_json(&cfg_json, t.path().to_str().unwrap(), "049").unwrap();
    let value: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(value["specId"], "049-slug");
    assert_eq!(value["commands"][0], "alpha");
}

// --- this repository's own corpus ----------------------------------------

/// The parse must agree with the corpus it governs. 048 is the only approved
/// spec carrying `verify:cli` fences, so it is the one real-world fixture
/// available, and its six commands are the shape the ported script produced.
#[test]
fn spec_048_parses_to_its_six_commands() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let plan = verify_plan(&cfg(), repo, "048").unwrap();
    assert_eq!(plan.spec_id, "048-kit-ships-the-governed-loop-skills");
    assert_eq!(plan.commands.len(), 6, "{:?}", plan.commands);
    assert!(plan.commands[0].starts_with("cargo test"));
    assert!(plan.skipped.is_empty());
}

/// 044 declares acceptance in prose only, which is the majority shape in this
/// corpus and the case spec 049 §1.2 says must stay distinguishable from a pass.
#[test]
fn spec_044_is_not_declared() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let plan = verify_plan(&cfg(), repo, "044").unwrap();
    assert!(!plan.is_declared());
}
