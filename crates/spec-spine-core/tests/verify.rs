//! Declared-acceptance tests (spec 043).
//!
//! Spec 043 §3.2's table is the contract these fixtures assert, one test per
//! row. The grammar is ported from `scripts/verify-spec.sh`, which carries no
//! tests in any of the four repositories that hold a copy of it, so these are
//! the first assertions the parse has ever had.
//!
//! Nothing here runs a command: the engine returns a plan and the CLI executes
//! it (spec 043 §3.1), so the whole grammar is testable over strings.

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
    assert_eq!(plan.spec_id, "049-slug", "spec 015 short-id resolution");
}

#[test]
fn a_missing_spec_is_not_found_never_stale() {
    let t = corpus(&[("049-slug", &doc("```verify:cli\nalpha\n```"))]);
    let err = plan_at(t.path(), "999").unwrap_err();
    assert!(matches!(err, Error::NotFound(_)), "got {err:?}");
    // Spec 043 §3.3: exit 2 is reserved for staleness across every verb.
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

/// The parse must agree with the corpus it governs, so one real spec is pinned
/// command for command rather than only fixtures.
///
/// It was 048 from spec 043 until spec 092: 048 was the first approved spec
/// carrying `verify:cli` fences, and its block was five commands (six until
/// spec 061 §3.6). Spec 092 §3.12 declared 048's acceptance replaced, so a plan
/// built for `048` is no longer 048's block, and the pin moved to a spec that
/// still holds its own. 091 is that spec: six commands, no fixture setup, and
/// a mix of positive and negated forms, which is what exercises the parse.
#[test]
fn spec_072_parses_to_its_six_commands() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let plan = verify_plan(&cfg(), repo, "072").unwrap();
    assert_eq!(plan.spec_id, "072-two-ready-specs-can-collide");
    assert_eq!(plan.commands.len(), 6, "{:?}", plan.commands);
    assert!(plan.commands[0].starts_with("cargo test"));
    assert!(plan.commands.iter().any(|c| c.starts_with("! ")));
    assert!(plan.skipped.is_empty());
}

/// Spec 082 §3.2's substitution, exercised against the real corpus rather than
/// a fixture: a plan built for a spec whose acceptance another spec holds is
/// the holder's block, under the holder's id.
///
/// 048 is the case spec 092 §3.12 created and the one the pin above used to
/// read. The corpus always has such a pair while a spec holds another's block,
/// so the case is exercised against whatever pair it currently has.
#[test]
fn an_amended_acceptance_resolves_to_its_holder() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let plan = verify_plan(&cfg(), repo, "054").unwrap();
    // Spec 082 §3.4: the substitution is STATED, never silent. The plan keeps
    // the id that was asked for and names the holder separately, so a caller
    // reading a verdict can tell whose block ran.
    assert_eq!(plan.spec_id, "054-the-scaffold-ships-what-adopters-wrote");
    assert_eq!(
        plan.acceptance_from.as_deref(),
        Some("092-the-engine-ships-governance-not-an-environment"),
        "054's block is held by 092 (amends_verification)"
    );
    // The holder's block, not 054's own, and not empty.
    assert!(plan.commands.len() > 5, "{:?}", plan.commands);
    assert!(plan.skipped.is_empty());
}

/// 037 declares acceptance in prose only, the case spec 043 §1.2 says must stay
/// distinguishable from a pass. It was 041 until spec 139 carried 041's
/// acceptance; 037 stays prose-only by design (139 3.3), so it is the stable
/// live example.
#[test]
fn spec_037_is_not_declared() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let plan = verify_plan(&cfg(), repo, "037").unwrap();
    assert!(!plan.is_declared());
}

// --- spec 082: an amended acceptance is the one that runs -----------------
//
// Spec 037 forbids editing an amended spec's file and `verify` executes that
// file, so before spec 082 an amendment could say an acceptance line was wrong
// and change nothing about what ran. These pin the redirection.

/// A spec document with a `## Verification` block and the given frontmatter
/// lines beyond `id` and `status`.
fn doc103(id: &str, status: &str, extra: &str, command: &str) -> String {
    format!(
        "---\nid: \"{id}\"\ntitle: \"t\"\nstatus: {status}\ncreated: \"2026-09-16\"\n\
         summary: \"s\"\nimplementation: complete\n{extra}---\n\n# {id}\n\n\
         ## Verification\n\n```verify:cli\n{command}\n```\n"
    )
}

/// Spec 082 §3.2: `verify <amended>` runs the amender's block, and the plan
/// names whose block it is holding.
#[test]
fn spec103_verify_runs_the_replacement_block() {
    let t = corpus(&[
        ("093-a", &doc103("093-a", "approved", "", "the-old-line")),
        (
            "103-b",
            &doc103(
                "103-b",
                "approved",
                "amends: [\"093-a\"]\namends_verification: [\"093-a\"]\n",
                "the-corrected-line",
            ),
        ),
    ]);
    let plan = plan_at(t.path(), "093-a").unwrap();
    assert_eq!(
        plan.spec_id, "093-a",
        "the plan answers for the spec asked for"
    );
    assert_eq!(plan.commands, ["the-corrected-line"]);
    assert_eq!(plan.acceptance_from.as_deref(), Some("103-b"));

    // The amender still runs its own block under its own name, with no
    // attribution line to print.
    let own = plan_at(t.path(), "103-b").unwrap();
    assert_eq!(own.commands, ["the-corrected-line"]);
    assert_eq!(own.acceptance_from, None);
}

/// Spec 082 §3.2: resolution follows the chain to its end, so a later
/// amendment attaches to whichever spec currently holds the acceptance.
#[test]
fn spec103_resolution_follows_the_chain() {
    let t = corpus(&[
        ("093-a", &doc103("093-a", "approved", "", "first")),
        (
            "103-b",
            &doc103(
                "103-b",
                "approved",
                "amends: [\"093-a\"]\namends_verification: [\"093-a\"]\n",
                "second",
            ),
        ),
        (
            "110-c",
            &doc103(
                "110-c",
                "approved",
                "amends: [\"103-b\"]\namends_verification: [\"103-b\"]\n",
                "third",
            ),
        ),
    ]);
    let plan = plan_at(t.path(), "093-a").unwrap();
    assert_eq!(plan.commands, ["third"], "the chain resolves to its end");
    assert_eq!(plan.acceptance_from.as_deref(), Some("110-c"));
}

/// Spec 082 §3.2, D-5: a superseded or retired amender is skipped, so a
/// retirement cannot silently change what a third spec asserts.
#[test]
fn spec103_a_withdrawn_amender_does_not_hold_the_acceptance() {
    for status in ["superseded", "retired"] {
        let t = corpus(&[
            ("093-a", &doc103("093-a", "approved", "", "the-original")),
            (
                "103-b",
                &doc103(
                    "103-b",
                    status,
                    "amends: [\"093-a\"]\namends_verification: [\"093-a\"]\n",
                    "the-withdrawn-one",
                ),
            ),
        ]);
        let plan = plan_at(t.path(), "093-a").unwrap();
        assert_eq!(
            plan.commands,
            ["the-original"],
            "a {status} amender must not hold another spec's acceptance"
        );
        assert_eq!(plan.acceptance_from, None);
    }
}

/// Spec 082 §3.2: a spec nobody amends is untouched, which is every spec in
/// the corpus but one.
#[test]
fn spec103_a_spec_with_no_amender_runs_its_own_block() {
    let t = corpus(&[
        ("093-a", &doc103("093-a", "approved", "", "its-own")),
        ("103-b", &doc103("103-b", "approved", "", "unrelated")),
    ]);
    let plan = plan_at(t.path(), "093-a").unwrap();
    assert_eq!(plan.commands, ["its-own"]);
    assert_eq!(plan.acceptance_from, None);
}

/// Spec 082 §3.2: a cycle resolves to no replacement rather than looping.
/// `compile` refuses it (`V-020`); `verify` runs against uncompiled trees too.
#[test]
fn spec103_a_cycle_does_not_hang_verify() {
    let t = corpus(&[
        (
            "093-a",
            &doc103(
                "093-a",
                "approved",
                "amends: [\"103-b\"]\namends_verification: [\"103-b\"]\n",
                "a",
            ),
        ),
        (
            "103-b",
            &doc103(
                "103-b",
                "approved",
                "amends: [\"093-a\"]\namends_verification: [\"093-a\"]\n",
                "b",
            ),
        ),
    ]);
    let plan = plan_at(t.path(), "093-a").unwrap();
    assert_eq!(plan.commands, ["a"], "falls back to the spec's own block");
    assert_eq!(plan.acceptance_from, None);
}

/// Spec 082 §3.4, the document half: a spec whose acceptance another spec
/// holds says so above its own fence, and a spec that still holds its own does
/// not say it.
///
/// Quantified over the real corpus, because the corpus is the set the rule is
/// about. A fixture would assert the marker's spelling and nothing about the
/// documents a reader opens, and the failure this closes is exactly a reader
/// opening one: `verify` prints the substitution, which is not where that
/// reader is, so thirteen `## Verification` sections read as live instruction
/// while nothing ran any of them.
#[test]
fn every_superseded_verification_block_says_so_in_its_own_document() {
    const MARK: &str = "> **Superseded acceptance";
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let mut held = 0usize;
    let mut own = 0usize;
    for entry in fs::read_dir(repo.join("specs")).expect("specs/ is readable") {
        let dir = entry.expect("dir entry").path();
        let md = dir.join("spec.md");
        if !md.is_file() {
            continue;
        }
        let id = dir
            .file_name()
            .expect("named directory")
            .to_string_lossy()
            .to_string();
        let text = fs::read_to_string(&md).expect("spec.md is readable");
        let plan = verify_plan(&cfg(), repo, &id).expect("the corpus plans");
        // The fence is a line that opens with it, not the first mention: a
        // spec may name the fence in prose or a table before its block (043
        // does, in its grammar table), and a substring match would judge the
        // note against that mention (spec 140 D-2).
        let fence = text.find("\n```verify:cli").map(|at| at + 1);
        match (plan.acceptance_from.as_deref(), fence) {
            // A block that no longer runs, in a document that has one.
            (Some(holder), Some(at)) => {
                held += 1;
                let head = &text[..at];
                let note = head.rfind(MARK).unwrap_or_else(|| {
                    panic!(
                        "{id}: its acceptance is {holder}'s and its own \
                         `## Verification` section does not say so"
                    )
                });
                assert!(
                    head[note..].contains(holder),
                    "{id}: the note must name the spec that holds the \
                     acceptance ({holder}), so a reader can follow it"
                );
            }
            // A spec that still holds its own block must not claim otherwise,
            // or the note stops meaning anything wherever it appears.
            //
            // Judged over the SAME region as the positive direction, the text
            // before the fence, and for the same reason: that is where the note
            // is defined to live and the only place it can mislead a reader.
            // A whole-file test reads this spec's own acceptance, which names
            // the marker in a command, and fails on the sentence that specifies
            // the rule. That is spec 071's pattern assertion matching itself.
            (None, Some(at)) => {
                own += 1;
                assert!(
                    !text[..at].contains(MARK),
                    "{id}: it runs its own block and must not be marked superseded"
                );
            }
            (_, None) => {}
        }
    }
    // Not a pinned count: the corpus always has such a pair while any spec
    // holds another's block (spec 082 §3.5), and pinning the number would make
    // every future amendment edit this line.
    assert!(held > 0, "no spec's acceptance is held by another");
    assert!(own > 0, "no spec runs its own block");
}
