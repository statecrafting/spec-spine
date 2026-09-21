// Spec: specs/096-compaction-is-a-verb-not-a-session/spec.md
//! Compaction tests (spec 096): every rule of §3, and each of §1.1's seven
//! measured defects with a case that fails against the rule that produced it.
//!
//! The defects are the point. Spec 095's renumber was green under `compile`,
//! `index`, `lint`, `couple` and `check` with 355 citations pointing at the
//! wrong document, so a test that only asserts "it rewrote something" asserts
//! what was already true when the corpus was wrong.

use std::fs;
use std::path::Path;

use spec_spine_core::compact::{
    CompactPlan, Form, RemoveEntry, Renumber, compact, parse_plan, verify_cli_blocks,
};
use spec_spine_types::{Config, load_config};

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, content).unwrap();
}

fn spec_doc(id: &str, body: &str) -> String {
    format!(
        "---\nid: \"{id}\"\ntitle: \"T\"\nstatus: approved\ncreated: \"2026-06-09\"\n\
         summary: \"s\"\nimplementation: complete\n---\n\n# {id}\n\n{body}\n"
    )
}

fn cfg() -> Config {
    load_config("[layout]\nspecs_dir = \"specs\"\n").unwrap()
}

/// Four specs, `000` through `003`. The prose lives in `docs/note.md` so a
/// rewrite of citations is separable from a rewrite of the corpus itself.
fn fixture(prose: &str) -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    let r = tmp.path();
    for id in ["000-alpha", "001-beta", "002-gamma", "003-delta"] {
        write(r, &format!("specs/{id}/spec.md"), &spec_doc(id, "Body."));
    }
    write(r, "docs/note.md", prose);
    tmp
}

/// Remove `001-beta`, answered by `000-alpha`, and compact the ordinals: the
/// survivors become `000-alpha`, `001-gamma`, `002-delta`.
fn plan() -> CompactPlan {
    CompactPlan {
        remove: vec![RemoveEntry {
            spec: "001-beta".into(),
            answered_by: "000-alpha".into(),
        }],
        renumber: Renumber::Contiguous,
        foreign_projects: vec!["OAP".into()],
        ..Default::default()
    }
}

/// The rewrites in `docs/note.md`. Every run renumbers the corpus itself, so a
/// "left alone" assertion is about the prose, never about the whole report.
fn prose_rewrites(
    c: &spec_spine_core::compact::Compaction,
) -> Vec<spec_spine_core::compact::Rewrite> {
    c.rewrites
        .iter()
        .find(|f| f.rel_path == "docs/note.md")
        .map(|f| f.rewrites.clone())
        .unwrap_or_default()
}

fn rewritten(c: &spec_spine_core::compact::Compaction, rel: &str) -> String {
    c.files
        .iter()
        .find(|f| f.from_rel_path == rel)
        .unwrap_or_else(|| panic!("no rewrite of {rel}: {:?}", c.files))
        .contents
        .clone()
}

// ── the plan and the map ─────────────────────────────────────────────────────

#[test]
fn a_plan_naming_a_spec_the_corpus_does_not_have_is_refused() {
    let tmp = fixture("nothing to see");
    let mut p = plan();
    p.remove[0].spec = "009-nope".into();
    let err = compact(&cfg(), tmp.path(), &p).unwrap_err();
    assert_eq!(err.exit_code(), 3, "{err}");
    assert!(format!("{err}").contains("009-nope"), "{err}");
}

#[test]
fn a_plan_whose_answering_spec_is_itself_removed_is_refused() {
    let tmp = fixture("x");
    let mut p = plan();
    p.remove.push(RemoveEntry {
        spec: "002-gamma".into(),
        answered_by: "001-beta".into(),
    });
    let err = compact(&cfg(), tmp.path(), &p).unwrap_err();
    assert_eq!(err.exit_code(), 3, "{err}");
    assert!(format!("{err}").contains("001-beta"), "{err}");
}

/// §1.1 defect 1 and §3.2: two specs sharing an ordinal is an ambiguity in an
/// identifier, and last-write-wins is never the right answer to one. The
/// original map was keyed on the ordinal and silently kept the last write,
/// which sent 355 citations across 34 files to the wrong document.
#[test]
fn an_ordinal_collision_is_refused_and_names_both() {
    let tmp = fixture("x");
    write(
        tmp.path(),
        "specs/002-twin/spec.md",
        &spec_doc("002-twin", "Body."),
    );
    let err = compact(&cfg(), tmp.path(), &plan()).unwrap_err();
    assert_eq!(err.exit_code(), 3, "{err}");
    let msg = format!("{err}");
    assert!(
        msg.contains("002-gamma") && msg.contains("002-twin"),
        "{msg}"
    );
}

// ── form 1: the full id ──────────────────────────────────────────────────────

#[test]
fn a_full_id_is_rewritten_to_its_new_id() {
    let tmp = fixture("See `002-gamma` and `003-delta`.\n");
    let c = compact(&cfg(), tmp.path(), &plan()).unwrap();
    let out = rewritten(&c, "docs/note.md");
    assert!(
        out.contains("`001-gamma`") && out.contains("`002-delta`"),
        "{out}"
    );
}

#[test]
fn a_removed_specs_full_id_becomes_the_spec_that_answers_for_it() {
    let tmp = fixture("Superseded by `001-beta`.\n");
    let c = compact(&cfg(), tmp.path(), &plan()).unwrap();
    assert!(rewritten(&c, "docs/note.md").contains("`000-alpha`"));
    // The directory, not the document: a spec is its directory, and a consumer
    // deleting only the `spec.md` would leave whatever else was in it behind.
    assert!(
        c.removed_paths.contains(&"specs/001-beta".to_string()),
        "{:?}",
        c.removed_paths
    );
}

#[test]
fn a_spec_directory_follows_its_new_id() {
    let tmp = fixture("x");
    let c = compact(&cfg(), tmp.path(), &plan()).unwrap();
    let moved = c
        .files
        .iter()
        .find(|f| f.from_rel_path == "specs/002-gamma/spec.md")
        .expect("gamma is renumbered");
    assert_eq!(moved.rel_path, "specs/001-gamma/spec.md");
}

// ── form 2: the prose citation ───────────────────────────────────────────────

#[test]
fn a_prose_citation_is_rewritten() {
    let tmp = fixture("As spec 002 says, and Spec 003 too.\n");
    let c = compact(&cfg(), tmp.path(), &plan()).unwrap();
    let out = rewritten(&c, "docs/note.md");
    assert!(out.contains("spec 001 says"), "{out}");
    assert!(out.contains("Spec 002 too"), "{out}");
}

/// §1.1 defect 3: the citation rule matched `spec 002` INSIDE
/// `--spec 002-gamma`, because the hyphen after the digits is a word boundary,
/// so every `--spec <full-id>` was rewritten twice and named a third,
/// unrelated document.
#[test]
fn a_citation_rule_does_not_fire_inside_a_full_id() {
    let tmp = fixture("Run it with --spec 002-gamma now.\n");
    let c = compact(&cfg(), tmp.path(), &plan()).unwrap();
    let out = rewritten(&c, "docs/note.md");
    assert!(out.contains("--spec 001-gamma"), "{out}");
    assert!(!out.contains("001-001"), "double-shifted: {out}");
    let note = c
        .rewrites
        .iter()
        .find(|f| f.rel_path == "docs/note.md")
        .expect("the prose is reported");
    let forms: Vec<Form> = note.rewrites.iter().map(|r| r.form).collect();
    assert_eq!(
        forms,
        vec![Form::FullId],
        "form 1 owns a full id: {forms:?}"
    );
}

/// §1.1 defect 6, measured on 2026-09-20: 28 citations across 24 files survived
/// spec 095's rewrite because the keyword ended a line and the ordinal began
/// the next one, behind a `///` or `#` continuation. Every one of them now
/// resolves to a different document.
#[test]
fn a_citation_broken_across_a_line_is_rewritten() {
    let tmp = fixture(
        "/// Every file under the root, for a caller that supplied no\n/// inventory (spec\n/// 002 §3.6). Skips the rest.\n",
    );
    let c = compact(&cfg(), tmp.path(), &plan()).unwrap();
    let out = rewritten(&c, "docs/note.md");
    assert!(out.contains("/// 001 §3.6)"), "{out}");
}

#[test]
fn a_citation_broken_across_two_blank_lines_is_not_a_citation() {
    let tmp = fixture("This is a spec\n\n002 is a heading of its own.\n");
    let c = compact(&cfg(), tmp.path(), &plan()).unwrap();
    assert!(
        prose_rewrites(&c).is_empty(),
        "a paragraph break is not a wrapped sentence: {:?}",
        c.rewrites
    );
}

/// §1.1 defect 7, measured the same day: 13 citations naming a second ordinal
/// after a separator kept the ordinal the first one lost.
#[test]
fn a_citation_naming_more_than_one_ordinal_rewrites_every_one() {
    let tmp = fixture("the spec 002/003 drift, and specs 002 and 003, and specs 002, 003.\n");
    let c = compact(&cfg(), tmp.path(), &plan()).unwrap();
    let out = rewritten(&c, "docs/note.md");
    assert!(out.contains("spec 001/002 drift"), "{out}");
    assert!(out.contains("specs 001 and 002"), "{out}");
    assert!(out.contains("specs 001, 002."), "{out}");
}

#[test]
fn a_citation_of_another_projects_corpus_is_left_alone() {
    let tmp = fixture("OAP spec 002 is not ours.\n");
    let c = compact(&cfg(), tmp.path(), &plan()).unwrap();
    assert!(prose_rewrites(&c).is_empty(), "{:?}", c.rewrites);
}

#[test]
fn an_unmapped_ordinal_is_left_alone() {
    let tmp = fixture("V-016, C-001, L-008, 0.18.0, spec 777, and 2026 are not ids.\n");
    let c = compact(&cfg(), tmp.path(), &plan()).unwrap();
    assert!(prose_rewrites(&c).is_empty(), "{:?}", c.rewrites);
}

/// An unmapped ordinal in a list does not stop the walk: each member of a list
/// is its own citation. Asserted because the loop's shape made it look like the
/// first member spoke for the rest.
#[test]
fn a_list_rewrites_its_mapped_ordinals_and_leaves_the_unmapped_one() {
    let tmp = fixture("the spec 777/002 pair, and specs 003 and 888.\n");
    let c = compact(&cfg(), tmp.path(), &plan()).unwrap();
    let out = rewritten(&c, "docs/note.md");
    assert!(out.contains("spec 777/001 pair"), "{out}");
    assert!(out.contains("specs 002 and 888."), "{out}");
}

/// A renamed spec directory moves file by file, and the walk only sees files the
/// rewrite can carry. A binary beside the `spec.md` would stay at the old path
/// while the document moved, splitting one spec across two directories.
#[test]
fn a_renamed_spec_directory_holding_an_uncarryable_file_is_refused() {
    let tmp = fixture("x");
    fs::write(
        tmp.path().join("specs/002-gamma/diagram.png"),
        [0x89u8, 0x50],
    )
    .unwrap();
    let err = compact(&cfg(), tmp.path(), &plan()).unwrap_err();
    assert_eq!(err.exit_code(), 3, "{err}");
    let msg = format!("{err}");
    assert!(
        msg.contains("diagram.png") && msg.contains("002-gamma"),
        "{msg}"
    );
}

// ── form 3: the bare short id ────────────────────────────────────────────────

/// §1.1 defect 2: a bare short id in a command carries no slug and no `spec`
/// before it, so no rule matched it and 32 acceptance commands addressed the
/// wrong spec or none. A form that fires zero times is the failure shape.
#[test]
fn a_bare_short_id_in_a_command_is_rewritten() {
    let tmp = fixture("run `spec-spine registry show 002` then `spec-spine verify 003`\n");
    let c = compact(&cfg(), tmp.path(), &plan()).unwrap();
    let out = rewritten(&c, "docs/note.md");
    assert!(out.contains("registry show 001"), "{out}");
    assert!(out.contains("verify 002"), "{out}");
    assert!(c.counts[Form::ShortIdArg.as_str()] >= 2, "{:?}", c.counts);
}

#[test]
fn a_short_id_behind_repo_addresses_a_fixture_corpus_and_is_left_alone() {
    let tmp = fixture("spec-spine --repo \"$t\" registry show 002\n");
    let c = compact(&cfg(), tmp.path(), &plan()).unwrap();
    assert!(prose_rewrites(&c).is_empty(), "{:?}", c.rewrites);
}

#[test]
fn a_short_id_after_a_flag_is_still_the_commands_argument() {
    let tmp = fixture("spec-spine registry show --json 002\n");
    let c = compact(&cfg(), tmp.path(), &plan()).unwrap();
    assert!(rewritten(&c, "docs/note.md").contains("--json 001"));
}

// ── §3.4 idempotence ─────────────────────────────────────────────────────────

/// §3.4: applying the output to the output changes nothing. Stated as a
/// requirement rather than left to follow from §3.3 because it is the property
/// defect 3 violated, and an assertion of it fails loudly where the rule's
/// subtlety does not.
#[test]
fn applying_the_output_to_the_output_is_idempotent() {
    let prose = "spec 002, `003-delta`, registry show 002, and spec\n002 wrapped.\n";
    let tmp = fixture(prose);
    let c = compact(&cfg(), tmp.path(), &plan()).unwrap();

    // Write the output back over the tree, then run the same verb again with a
    // plan that has nothing left to remove and a corpus already contiguous.
    let root = tmp.path();
    for p in &c.removed_paths {
        fs::remove_dir_all(root.join(p)).unwrap();
    }
    for f in &c.files {
        if f.rel_path != f.from_rel_path {
            let _ = fs::remove_file(root.join(&f.from_rel_path));
        }
        write(root, &f.rel_path, &f.contents);
    }
    let again = compact(
        &cfg(),
        root,
        &CompactPlan {
            remove: vec![],
            renumber: Renumber::Contiguous,
            foreign_projects: vec!["OAP".into()],
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        again.rewrite_count(),
        0,
        "second pass rewrote: {:?}",
        again.rewrites
    );
}

// ── §3.5 the fenced block ────────────────────────────────────────────────────

/// §1.1 defect 4: the rewrite was scoped to acceptance blocks with a non-greedy
/// fence regex, and one block greps for the literal opening fence, which closed
/// the match early; the file was rewritten in half and the second pass then
/// double-applied. The locator scans lines and closes only on a line that is
/// exactly the closing fence.
#[test]
fn a_block_that_names_its_own_fence_is_located_whole() {
    let doc = "intro\n\n```verify:cli\ngrep -q '```verify:cli' specs/x/spec.md\necho tail\n```\n\nafter\n";
    let blocks = verify_cli_blocks(doc);
    assert_eq!(blocks.len(), 1, "{blocks:?}");
    let body = &doc[blocks[0].0..blocks[0].1];
    assert!(body.contains("echo tail"), "closed early: {body:?}");
}

#[test]
fn a_fence_inside_a_block_does_not_open_a_second_one() {
    let doc = "```verify:cli\necho one\n```\n\n```verify:cli\necho two\n```\n";
    assert_eq!(verify_cli_blocks(doc).len(), 2);
}

// ── §3.6 the report, §3.7 the map ────────────────────────────────────────────

/// §3.6: a six-thousand-line diff is not reviewable, and the question a
/// reviewer has is not "what changed" but "was each change the right rule".
#[test]
fn the_report_names_the_form_behind_every_rewrite_and_counts_per_form() {
    let tmp = fixture("`002-gamma`, spec 003, registry show 002\n");
    let c = compact(&cfg(), tmp.path(), &plan()).unwrap();
    let file = c
        .rewrites
        .iter()
        .find(|f| f.rel_path == "docs/note.md")
        .expect("the prose is reported");
    let forms: Vec<Form> = file.rewrites.iter().map(|r| r.form).collect();
    assert!(forms.contains(&Form::FullId), "{forms:?}");
    assert!(forms.contains(&Form::Citation), "{forms:?}");
    assert!(forms.contains(&Form::ShortIdArg), "{forms:?}");
    for r in &file.rewrites {
        assert!(r.line > 0 && !r.old.is_empty() && r.old != r.new, "{r:?}");
    }
    // Every form is counted, including the ones that fired zero times: a form
    // absent from the table cannot be noticed as absent (defect 2).
    for form in [
        Form::FullId,
        Form::Citation,
        Form::ShortIdArg,
        Form::RetiredPath,
    ] {
        assert!(c.counts.contains_key(form.as_str()), "{:?}", c.counts);
    }
}

#[test]
fn the_report_of_a_plan_that_changes_nothing_is_empty_not_absent() {
    let tmp = fixture("no ids here\n");
    let c = compact(
        &cfg(),
        tmp.path(),
        &CompactPlan {
            remove: vec![],
            renumber: Renumber::None,
            foreign_projects: vec![],
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(c.rewrite_count(), 0);
    // Every form is present with a zero, not absent: a form absent from the
    // table cannot be noticed as absent, which is defect 2. Asserted per form
    // rather than by a count, so adding a form does not silently satisfy it.
    for form in [
        Form::FullId,
        Form::Citation,
        Form::ShortIdArg,
        Form::RetiredPath,
    ] {
        assert_eq!(c.counts.get(form.as_str()), Some(&0), "{:?}", c.counts);
    }
}

/// §3.7: a citation outlives the document it cites, and after a renumber a bare
/// old ordinal is worse than a dangling one, because it resolves.
#[test]
fn the_map_document_carries_every_id_in_both_directions() {
    let tmp = fixture("x");
    let c = compact(&cfg(), tmp.path(), &plan()).unwrap();
    for (from, to) in [
        ("000-alpha", "000-alpha"),
        ("001-beta", "000-alpha"),
        ("002-gamma", "001-gamma"),
        ("003-delta", "002-delta"),
    ] {
        assert!(
            c.map_document.contains(&format!("| `{from}` | `{to}`")),
            "{from} -> {to} missing from:\n{}",
            c.map_document
        );
    }
    assert!(c.map_document.contains("(removed)"), "{}", c.map_document);
}

// ── the plan file ────────────────────────────────────────────────────────────

#[test]
fn a_plan_parses_from_yaml() {
    let p = parse_plan(
        "remove:\n  - { spec: \"001-beta\", answered_by: \"000-alpha\" }\nrenumber: contiguous\n",
    )
    .unwrap();
    assert_eq!(p.remove.len(), 1);
    assert_eq!(p.renumber, Renumber::Contiguous);
    assert_eq!(p.foreign_projects, vec!["OAP".to_string()]);
}

#[test]
fn a_plan_with_an_unknown_key_is_refused() {
    let err = parse_plan("renumber: contiguous\nrenumberr: none\n").unwrap_err();
    assert_eq!(err.exit_code(), 3, "{err}");
}

// ── form 4: the document's own title heading (spec 098 §3.1) ────────────────

/// A spec document whose title heading is written `# NNN: Title` rather than
/// `# NNN-slug`. Spec 095's renumber left 90 of 98 documents in this corpus
/// titled with another spec's ordinal, because none of 096's three forms
/// matches a heading.
fn titled_spec_doc(id: &str, heading_ordinal: &str, body: &str) -> String {
    format!(
        "---\nid: \"{id}\"\ntitle: \"T\"\nstatus: approved\ncreated: \"2026-06-09\"\n\
         summary: \"s\"\nimplementation: complete\n---\n\n# {heading_ordinal}: Title\n\n{body}\n"
    )
}

#[test]
fn a_documents_title_heading_follows_its_new_ordinal() {
    let tmp = fixture("x");
    write(
        tmp.path(),
        "specs/002-gamma/spec.md",
        &titled_spec_doc("002-gamma", "002", "Body."),
    );
    let c = compact(&cfg(), tmp.path(), &plan()).unwrap();
    let out = rewritten(&c, "specs/002-gamma/spec.md");
    assert!(out.contains("# 001: Title"), "{out}");
    assert!(!out.contains("# 002: Title"), "{out}");
    let forms: Vec<Form> = c
        .rewrites
        .iter()
        .filter(|f| f.rel_path == "specs/002-gamma/spec.md")
        .flat_map(|f| f.rewrites.iter().map(|r| r.form))
        .collect();
    assert!(forms.contains(&Form::TitleHeading), "{forms:?}");
    assert_eq!(c.counts.get("title-heading"), Some(&1), "{:?}", c.counts);
}

/// The rule is "this document's own old ordinal", not "any heading". A spec
/// quoting another spec's heading is citing it, and citing is form 2's
/// business: rewriting it here would move a citation to the citing document's
/// new ordinal, which is the wrong document twice over.
#[test]
fn a_heading_naming_another_spec_is_left_alone_by_form_4() {
    let tmp = fixture("x");
    write(
        tmp.path(),
        "specs/002-gamma/spec.md",
        &titled_spec_doc("002-gamma", "003", "Body."),
    );
    let c = compact(&cfg(), tmp.path(), &plan()).unwrap();
    let out = rewritten(&c, "specs/002-gamma/spec.md");
    assert!(out.contains("# 003: Title"), "{out}");
    assert_eq!(c.counts.get("title-heading"), Some(&0), "{:?}", c.counts);
}

/// A `#` comment inside YAML frontmatter is not a heading. This corpus has
/// several that open with an ordinal, and reading one as the title would both
/// miss the real title and rewrite a citation under the wrong form.
#[test]
fn a_frontmatter_comment_opening_with_an_ordinal_is_not_the_title() {
    let tmp = fixture("x");
    write(
        tmp.path(),
        "specs/002-gamma/spec.md",
        "---\nid: \"002-gamma\"\ntitle: \"T\"\nstatus: approved\ncreated: \"2026-06-09\"\n\
         # 002: an ordinal in a frontmatter comment\nsummary: \"s\"\n\
         implementation: complete\n---\n\n# 002: Title\n\nBody.\n",
    );
    let c = compact(&cfg(), tmp.path(), &plan()).unwrap();
    let out = rewritten(&c, "specs/002-gamma/spec.md");
    assert!(
        out.contains("# 002: an ordinal in a frontmatter comment"),
        "{out}"
    );
    assert!(out.contains("# 001: Title"), "{out}");
    assert_eq!(c.counts.get("title-heading"), Some(&1), "{:?}", c.counts);
}

// ── form 5: a bare ordinal in a citation's position (spec 098 §3.2) ─────────

#[test]
fn a_bare_ordinal_followed_by_a_reference_is_rewritten() {
    let tmp = fixture(
        "Section 002 \u{a7}3.1, decimal 002 3.6, decision 002 D-4, possessive 002's rule.\n",
    );
    let c = compact(&cfg(), tmp.path(), &plan()).unwrap();
    let out = rewritten(&c, "docs/note.md");
    assert_eq!(
        out, "Section 001 \u{a7}3.1, decimal 001 3.6, decision 001 D-4, possessive 001's rule.\n",
        "{out}"
    );
    assert_eq!(c.counts.get("bare-ordinal"), Some(&4), "{:?}", c.counts);
}

/// The exclusions are as normative as the form (096 §3.3). Every token here is
/// three digits that a wider rule would have taken: a validation code, a
/// version, a citation with no reference after it, and another project's.
#[test]
fn a_bare_ordinal_in_any_other_position_is_left_alone() {
    let prose = "V-002 \u{a7}3.1, 0.002 \u{a7}3.1, before 002, (002, 002-gamma \u{a7}3.1, \
                 OAP 002 \u{a7}3.1, exit 002.\n";
    let tmp = fixture(prose);
    let c = compact(&cfg(), tmp.path(), &plan()).unwrap();
    let out = rewritten(&c, "docs/note.md");
    // `002-gamma` is form 1's, and only form 1's: the ordinal inside a full id
    // must not be touched twice (defect 3).
    assert_eq!(
        out,
        "V-002 \u{a7}3.1, 0.002 \u{a7}3.1, before 002, (002, 001-gamma \u{a7}3.1, \
         OAP 002 \u{a7}3.1, exit 002.\n",
        "{out}"
    );
    assert_eq!(c.counts.get("bare-ordinal"), Some(&0), "{:?}", c.counts);
}

/// §3.4: applying the output to the output changes nothing, for the two forms
/// this spec adds as well. Form 4 is the one with a real risk here: a heading
/// rewritten to `001` in a corpus that also maps `001` would shift twice if the
/// rule read the heading rather than the document's own old id.
#[test]
fn the_new_forms_are_idempotent() {
    let tmp = fixture("Section 002 \u{a7}3.1 and 002's rule.\n");
    write(
        tmp.path(),
        "specs/002-gamma/spec.md",
        &titled_spec_doc("002-gamma", "002", "Body."),
    );
    let root = tmp.path();
    let first = compact(&cfg(), root, &plan()).unwrap();
    for p in &first.removed_paths {
        fs::remove_dir_all(root.join(p)).unwrap();
    }
    for f in &first.files {
        if f.rel_path != f.from_rel_path {
            let _ = fs::remove_file(root.join(&f.from_rel_path));
        }
        write(root, &f.rel_path, &f.contents);
    }
    let again = compact(
        &cfg(),
        root,
        &CompactPlan {
            remove: vec![],
            renumber: Renumber::Contiguous,
            foreign_projects: vec!["OAP".into()],
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        again.rewrite_count(),
        0,
        "second pass rewrote: {:?}",
        again.rewrites
    );
}

/// An elided full id: the slug replaced by an ellipsis, which the corpus writes
/// when a path is too long for the sentence. Form 1 matches the map's keys and
/// `002-...` is not one; form 2 needs the keyword; and the `-` is exactly what
/// keeps the digits out of every other rule.
#[test]
fn an_elided_full_id_is_rewritten() {
    let tmp = fixture("See `specs/002-.../spec.md` and `002-gamma`.\n");
    let c = compact(&cfg(), tmp.path(), &plan()).unwrap();
    let out = rewritten(&c, "docs/note.md");
    assert_eq!(
        out, "See `specs/001-.../spec.md` and `001-gamma`.\n",
        "{out}"
    );
}

/// The ellipsis is the whole of it: `002-` followed by anything else is a full
/// id form 1 owns, or a token that is not a reference at all.
#[test]
fn a_hyphen_that_is_not_an_ellipsis_is_not_an_elided_id() {
    let tmp = fixture("Range 002-003, and 002-unknown.\n");
    let c = compact(&cfg(), tmp.path(), &plan()).unwrap();
    // Not "rewritten to the same text": not rewritten at all, so the file is
    // not in the output set. A rule that matched either token would put it
    // there.
    assert!(
        !c.files.iter().any(|f| f.from_rel_path == "docs/note.md"),
        "{:?}",
        c.files
    );
    assert!(prose_rewrites(&c).is_empty(), "{:?}", prose_rewrites(&c));
}

// ── §3.3: the scan opens a file that carries no extension ───────────────────

/// Defect 8. `.gitignore` reads as an empty stem with the extension
/// `gitignore`, so it was on no list and was never opened; six plain citations
/// survived spec 095's renumber in it and in `.gitattributes`.
#[test]
fn a_citation_in_an_extension_less_file_is_rewritten() {
    let tmp = fixture("x");
    write(tmp.path(), ".gitignore", "# generated by spec 002\nout/\n");
    write(
        tmp.path(),
        "Makefile",
        "# spec 002 \u{a7}3.6\nall:\n\t@true\n",
    );
    let c = compact(&cfg(), tmp.path(), &plan()).unwrap();
    assert!(
        rewritten(&c, ".gitignore").contains("spec 001"),
        "{}",
        rewritten(&c, ".gitignore")
    );
    assert!(
        rewritten(&c, "Makefile").contains("spec 001 \u{a7}3.6"),
        "{}",
        rewritten(&c, "Makefile")
    );
}

/// Decided by content, not by a name: an extension-less file holding NUL bytes
/// is not prose and is never read as prose.
#[test]
fn an_extension_less_binary_file_is_not_scanned() {
    let tmp = fixture("x");
    fs::write(tmp.path().join("blob"), [0x00u8, 0xff, 0x00, 0xfe]).unwrap();
    let c = compact(&cfg(), tmp.path(), &plan()).unwrap();
    assert!(
        !c.files.iter().any(|f| f.from_rel_path == "blob"),
        "{:?}",
        c.files
    );
}
