// Spec: specs/111-a-move-is-a-reviewed-mapping/spec.md
//! Spec 111: a move is declared, its shape validated at compile, its paths
//! checked against the tree as lint warnings, and a lookup derived from every
//! declaration follows declared chains, reporting ambiguity and cycles by
//! name instead of guessing through them. Each rule is exercised over a
//! disposable corpus through the real `compile`/`lint`, so a code that cannot
//! fire fails here rather than in review.

use std::fs;
use std::path::Path;

use spec_spine_core::moves::{MoveLookup, flattened_moves, lookup};
use spec_spine_core::{compile, lint};
use spec_spine_types::{Config, Severity};

const BODY: &str = "# T\n\n## 1. Purpose\n\nWhy.\n\n## Verification\n\nChecks.\n";

fn write(root: &Path, id: &str, extra: &str) {
    let dir = root.join("specs").join(id);
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("spec.md"),
        format!(
            "---\nid: \"{id}\"\ntitle: \"T\"\nstatus: approved\ncreated: \"2026-09-23\"\n\
             summary: \"s\"\n{extra}---\n{BODY}"
        ),
    )
    .unwrap();
}

fn codes(out: &spec_spine_core::CompileOutcome) -> Vec<(String, Severity, String)> {
    out.registry
        .validation
        .violations
        .iter()
        .map(|v| (v.code.clone(), v.severity, v.message.clone()))
        .collect()
}

fn has(out: &spec_spine_core::CompileOutcome, code: &str, needle: &str) -> bool {
    codes(out)
        .iter()
        .any(|(c, _, m)| c == code && m.contains(needle))
}

fn severity_of(out: &spec_spine_core::CompileOutcome, code: &str) -> Severity {
    codes(out)
        .into_iter()
        .find(|(c, ..)| c == code)
        .map(|(_, s, _)| s)
        .unwrap_or_else(|| panic!("no violation with code {code}"))
}

fn record<'a>(
    out: &'a spec_spine_core::CompileOutcome,
    id: &str,
) -> &'a spec_spine_types::SpecRecord {
    out.registry.specs.iter().find(|s| s.id == id).unwrap()
}

// ===== 3.2: shape validation at compile (V-040 errors, V-041 warning) ======

#[test]
fn arity_mismatch_is_v040() {
    let tmp = tempfile::tempdir().unwrap();
    // `relocated` names two `from` paths: arity is one to one.
    write(
        tmp.path(),
        "001-a",
        "moves:\n  - from: [\"a/x.rs\", \"a/y.rs\"]\n    to: \"a/z.rs\"\n    kind: relocated\n",
    );
    let out = compile(&Config::default(), tmp.path()).unwrap();
    assert!(!out.validation_passed, "{:?}", codes(&out));
    assert!(
        has(&out, "V-040", "does not match its kind"),
        "{:?}",
        codes(&out)
    );
    assert_eq!(severity_of(&out, "V-040"), Severity::Error);
}

#[test]
fn split_needs_at_least_two_to_paths() {
    let tmp = tempfile::tempdir().unwrap();
    write(
        tmp.path(),
        "001-a",
        "moves:\n  - from: \"a/x.rs\"\n    to: \"a/y.rs\"\n    kind: split\n",
    );
    let out = compile(&Config::default(), tmp.path()).unwrap();
    assert!(!out.validation_passed);
    assert!(has(&out, "V-040", "does not match its kind"));
}

#[test]
fn merged_needs_at_least_two_from_paths() {
    let tmp = tempfile::tempdir().unwrap();
    write(
        tmp.path(),
        "001-a",
        "moves:\n  - from: \"a/x.rs\"\n    to: \"a/y.rs\"\n    kind: merged\n",
    );
    let out = compile(&Config::default(), tmp.path()).unwrap();
    assert!(!out.validation_passed);
    assert!(has(&out, "V-040", "does not match its kind"));
}

#[test]
fn removed_with_a_to_path_is_v040() {
    let tmp = tempfile::tempdir().unwrap();
    write(
        tmp.path(),
        "001-a",
        "moves:\n  - from: \"a/x.rs\"\n    to: \"a/y.rs\"\n    kind: removed\n",
    );
    let out = compile(&Config::default(), tmp.path()).unwrap();
    assert!(!out.validation_passed);
    assert!(has(&out, "V-040", "does not match its kind"));
}

#[test]
fn an_empty_path_is_v040() {
    let tmp = tempfile::tempdir().unwrap();
    write(
        tmp.path(),
        "001-a",
        "moves:\n  - from: \"\"\n    to: \"a/y.rs\"\n    kind: relocated\n",
    );
    let out = compile(&Config::default(), tmp.path()).unwrap();
    assert!(!out.validation_passed);
    assert!(has(&out, "V-040", "is empty"), "{:?}", codes(&out));
}

#[test]
fn an_absolute_path_is_v040() {
    let tmp = tempfile::tempdir().unwrap();
    write(
        tmp.path(),
        "001-a",
        "moves:\n  - from: \"/a/x.rs\"\n    to: \"a/y.rs\"\n    kind: relocated\n",
    );
    let out = compile(&Config::default(), tmp.path()).unwrap();
    assert!(!out.validation_passed);
    assert!(has(&out, "V-040", "is absolute"), "{:?}", codes(&out));
}

#[test]
fn a_dot_dot_segment_is_v040() {
    let tmp = tempfile::tempdir().unwrap();
    write(
        tmp.path(),
        "001-a",
        "moves:\n  - from: \"a/../x.rs\"\n    to: \"a/y.rs\"\n    kind: relocated\n",
    );
    let out = compile(&Config::default(), tmp.path()).unwrap();
    assert!(!out.validation_passed);
    assert!(has(&out, "V-040", "'..' segment"), "{:?}", codes(&out));
}

#[test]
fn the_same_path_on_both_sides_is_v040() {
    let tmp = tempfile::tempdir().unwrap();
    write(
        tmp.path(),
        "001-a",
        "moves:\n  - from: \"a/x.rs\"\n    to: \"a/x.rs\"\n    kind: relocated\n",
    );
    let out = compile(&Config::default(), tmp.path()).unwrap();
    assert!(!out.validation_passed);
    assert!(
        has(&out, "V-040", "on both sides of one entry"),
        "{:?}",
        codes(&out)
    );
}

#[test]
fn answered_by_on_a_kind_other_than_removed_is_v040() {
    let tmp = tempfile::tempdir().unwrap();
    write(
        tmp.path(),
        "001-a",
        "moves:\n  - from: \"a/x.rs\"\n    to: \"a/y.rs\"\n    kind: relocated\n    answered_by: \"001-a\"\n",
    );
    let out = compile(&Config::default(), tmp.path()).unwrap();
    assert!(!out.validation_passed);
    assert!(
        has(&out, "V-040", "meaningful only for a 'removed' entry"),
        "{:?}",
        codes(&out)
    );
}

#[test]
fn a_dangling_answered_by_is_v041_and_only_a_warning() {
    let tmp = tempfile::tempdir().unwrap();
    write(
        tmp.path(),
        "001-a",
        "moves:\n  - from: \"a/x.rs\"\n    to: null\n    kind: removed\n    answered_by: \"999-none\"\n",
    );
    let out = compile(&Config::default(), tmp.path()).unwrap();
    // D-3: a dangling `answered_by` is a compile WARNING, not an error, the
    // tier `depends_on` (V-010) and every edge target take; only
    // `superseded_by` (V-008), which moves authority, is an error. A move
    // moves no authority (§3.5).
    assert!(
        out.validation_passed,
        "a warning must not fail validation: {:?}",
        codes(&out)
    );
    assert!(
        has(&out, "V-041", "does not resolve to an existing spec"),
        "{:?}",
        codes(&out)
    );
    assert_eq!(severity_of(&out, "V-041"), Severity::Warning);
}

#[test]
fn valid_declarations_compile_and_answered_by_normalizes_a_short_id() {
    let tmp = tempfile::tempdir().unwrap();
    write(
        tmp.path(),
        "001-a",
        "moves:\n  \
           - from: \"a/relocated.rs\"\n    to: \"a/relocated2.rs\"\n    kind: relocated\n  \
           - from: \"a/split.rs\"\n    to: [\"a/split1.rs\", \"a/split2.rs\"]\n    kind: split\n  \
           - from: [\"a/m1.rs\", \"a/m2.rs\"]\n    to: \"a/merged.rs\"\n    kind: merged\n  \
           - from: \"a/gone.rs\"\n    to: null\n    kind: removed\n    answered_by: \"002\"\n",
    );
    write(tmp.path(), "002-b", "");
    let out = compile(&Config::default(), tmp.path()).unwrap();
    assert!(out.validation_passed, "{:?}", codes(&out));
    let a = record(&out, "001-a");
    assert_eq!(a.moves.len(), 4);
    assert_eq!(a.moves[3].answered_by.as_deref(), Some("002-b"));

    // The shard carries them too, camelCase.
    let shard = serde_json::to_value(
        out.shards
            .spec_shards
            .iter()
            .find(|s| s.record.id == "001-a")
            .unwrap(),
    )
    .unwrap();
    assert_eq!(shard["record"]["moves"].as_array().unwrap().len(), 4);
    assert_eq!(shard["record"]["moves"][0]["from"], "a/relocated.rs");
    assert_eq!(shard["record"]["moves"][3]["answeredBy"], "002-b");
    assert!(shard["record"]["moves"][3]["to"].is_null());
}

#[test]
fn a_spec_without_the_key_has_no_moves_member() {
    let tmp = tempfile::tempdir().unwrap();
    write(tmp.path(), "001-a", "");
    let out = compile(&Config::default(), tmp.path()).unwrap();
    assert!(out.validation_passed, "{:?}", codes(&out));
    let a = record(&out, "001-a");
    assert!(a.moves.is_empty());
    let shard = serde_json::to_value(
        out.shards
            .spec_shards
            .iter()
            .find(|s| s.record.id == "001-a")
            .unwrap(),
    )
    .unwrap();
    assert!(
        shard["record"].get("moves").is_none(),
        "a spec that declares no move must not carry the member at all: {shard}"
    );
}

// ===== 3.3: the paths are checked against the tree, as warnings (L-015/016) =

#[test]
fn l015_a_missing_to_path_is_warned_and_an_existing_one_is_silent() {
    let tmp = tempfile::tempdir().unwrap();
    write(
        tmp.path(),
        "001-a",
        "moves:\n  - from: \"a/old.rs\"\n    to: \"a/new.rs\"\n    kind: relocated\n",
    );
    // Neither path exists on disk.
    let report = lint(&Config::default(), tmp.path()).unwrap();
    assert!(
        report.violations.iter().any(|v| v.code == "L-015"
            && v.message.contains("a/new.rs")
            && v.severity == Severity::Warning),
        "{:?}",
        report.violations
    );

    // Now the `to` path exists: no L-015.
    fs::create_dir_all(tmp.path().join("a")).unwrap();
    fs::write(tmp.path().join("a/new.rs"), "pub fn new() {}\n").unwrap();
    let report = lint(&Config::default(), tmp.path()).unwrap();
    assert!(
        !report.violations.iter().any(|v| v.code == "L-015"),
        "{:?}",
        report.violations
    );
}

#[test]
fn l016_a_removed_path_that_still_exists_is_warned_and_its_absence_is_silent() {
    let tmp = tempfile::tempdir().unwrap();
    write(
        tmp.path(),
        "001-a",
        "moves:\n  - from: \"a/gone.rs\"\n    to: null\n    kind: removed\n",
    );
    fs::create_dir_all(tmp.path().join("a")).unwrap();
    fs::write(tmp.path().join("a/gone.rs"), "pub fn gone() {}\n").unwrap();
    let report = lint(&Config::default(), tmp.path()).unwrap();
    assert!(
        report.violations.iter().any(|v| v.code == "L-016"
            && v.message.contains("a/gone.rs")
            && v.severity == Severity::Warning),
        "{:?}",
        report.violations
    );

    // Removed and gone: no L-016.
    fs::remove_file(tmp.path().join("a/gone.rs")).unwrap();
    let report = lint(&Config::default(), tmp.path()).unwrap();
    assert!(
        !report.violations.iter().any(|v| v.code == "L-016"),
        "{:?}",
        report.violations
    );
}

// ===== 3.4: the lookup is derived and never guesses =========================

fn compiled_registry(root: &Path) -> spec_spine_types::Registry {
    let out = compile(&Config::default(), root).unwrap();
    assert!(out.validation_passed, "{:?}", codes(&out));
    out.registry
}

#[test]
fn an_unmapped_path_has_no_declaration() {
    let tmp = tempfile::tempdir().unwrap();
    write(tmp.path(), "001-a", "");
    let registry = compiled_registry(tmp.path());
    let outcome = lookup(&registry, "never/declared.rs");
    assert_eq!(
        outcome,
        MoveLookup::Unmapped {
            path: "never/declared.rs".to_string()
        }
    );
}

#[test]
fn a_two_hop_chain_across_two_specs_resolves() {
    let tmp = tempfile::tempdir().unwrap();
    write(
        tmp.path(),
        "001-a",
        "moves:\n  - from: \"a.rs\"\n    to: \"b.rs\"\n    kind: relocated\n",
    );
    write(
        tmp.path(),
        "002-b",
        "moves:\n  - from: \"b.rs\"\n    to: \"c.rs\"\n    kind: relocated\n",
    );
    let registry = compiled_registry(tmp.path());
    let outcome = lookup(&registry, "a.rs");
    match outcome {
        MoveLookup::Resolved {
            path,
            hops,
            terminals,
        } => {
            assert_eq!(path, "a.rs");
            assert_eq!(hops.len(), 2);
            assert_eq!(hops[0].from, "a.rs");
            assert_eq!(hops[0].to.as_deref(), Some("b.rs"));
            assert_eq!(hops[0].declared_by, vec!["001-a".to_string()]);
            assert_eq!(hops[1].from, "b.rs");
            assert_eq!(hops[1].to.as_deref(), Some("c.rs"));
            assert_eq!(hops[1].declared_by, vec!["002-b".to_string()]);
            assert_eq!(terminals.len(), 1);
            assert_eq!(terminals[0].path.as_deref(), Some("c.rs"));
        }
        other => panic!("expected resolved, got {other:?}"),
    }
}

#[test]
fn a_split_declaration_is_followed_not_ambiguous() {
    let tmp = tempfile::tempdir().unwrap();
    write(
        tmp.path(),
        "001-a",
        "moves:\n  - from: \"a.rs\"\n    to: [\"b.rs\", \"c.rs\"]\n    kind: split\n",
    );
    let registry = compiled_registry(tmp.path());
    let outcome = lookup(&registry, "a.rs");
    match outcome {
        MoveLookup::Resolved {
            hops, terminals, ..
        } => {
            assert_eq!(hops.len(), 2, "one hop per branch, {hops:?}");
            let mut targets: Vec<&str> = hops.iter().filter_map(|h| h.to.as_deref()).collect();
            targets.sort();
            assert_eq!(targets, vec!["b.rs", "c.rs"]);
            let mut term_paths: Vec<&str> =
                terminals.iter().filter_map(|t| t.path.as_deref()).collect();
            term_paths.sort();
            assert_eq!(term_paths, vec!["b.rs", "c.rs"]);
        }
        other => panic!("split branches must be followed, not ambiguous: {other:?}"),
    }
}

#[test]
fn a_merged_declaration_resolves_from_either_source() {
    let tmp = tempfile::tempdir().unwrap();
    write(
        tmp.path(),
        "001-a",
        "moves:\n  - from: [\"a.rs\", \"b.rs\"]\n    to: \"c.rs\"\n    kind: merged\n",
    );
    let registry = compiled_registry(tmp.path());
    for from in ["a.rs", "b.rs"] {
        match lookup(&registry, from) {
            MoveLookup::Resolved { hops, .. } => {
                assert_eq!(hops.len(), 1);
                assert_eq!(hops[0].to.as_deref(), Some("c.rs"));
            }
            other => panic!("{from}: expected resolved, got {other:?}"),
        }
    }
}

#[test]
fn a_removed_step_ends_with_its_answered_by() {
    let tmp = tempfile::tempdir().unwrap();
    write(
        tmp.path(),
        "001-a",
        "moves:\n  - from: \"a.rs\"\n    to: null\n    kind: removed\n    answered_by: \"002-b\"\n",
    );
    write(tmp.path(), "002-b", "");
    let registry = compiled_registry(tmp.path());
    match lookup(&registry, "a.rs") {
        MoveLookup::Resolved {
            hops, terminals, ..
        } => {
            assert_eq!(hops.len(), 1);
            assert_eq!(hops[0].to, None);
            assert_eq!(hops[0].answered_by.as_deref(), Some("002-b"));
            assert_eq!(terminals.len(), 1);
            assert_eq!(terminals[0].path, None);
            assert_eq!(terminals[0].answered_by.as_deref(), Some("002-b"));
        }
        other => panic!("expected resolved, got {other:?}"),
    }
}

#[test]
fn a_removed_step_with_no_answered_by_still_resolves() {
    let tmp = tempfile::tempdir().unwrap();
    write(
        tmp.path(),
        "001-a",
        "moves:\n  - from: \"a.rs\"\n    to: null\n    kind: removed\n",
    );
    let registry = compiled_registry(tmp.path());
    match lookup(&registry, "a.rs") {
        MoveLookup::Resolved { terminals, .. } => {
            assert_eq!(terminals.len(), 1);
            assert_eq!(terminals[0].path, None);
            assert_eq!(terminals[0].answered_by, None);
        }
        other => panic!("expected resolved, got {other:?}"),
    }
}

#[test]
fn two_specs_disagreeing_on_the_same_from_is_ambiguous() {
    let tmp = tempfile::tempdir().unwrap();
    write(
        tmp.path(),
        "001-a",
        "moves:\n  - from: \"a.rs\"\n    to: \"b.rs\"\n    kind: relocated\n",
    );
    write(
        tmp.path(),
        "002-b",
        "moves:\n  - from: \"a.rs\"\n    to: \"c.rs\"\n    kind: relocated\n",
    );
    let registry = compiled_registry(tmp.path());
    match lookup(&registry, "a.rs") {
        MoveLookup::Ambiguous {
            path,
            at,
            candidates,
        } => {
            assert_eq!(path, "a.rs");
            assert_eq!(at, "a.rs");
            assert_eq!(candidates.len(), 2);
            let mut by: Vec<(&str, Option<&str>)> = candidates
                .iter()
                .map(|c| (c.declared_by.as_str(), c.to.as_deref()))
                .collect();
            by.sort();
            assert_eq!(by, vec![("001-a", Some("b.rs")), ("002-b", Some("c.rs"))]);
        }
        other => panic!("expected ambiguous, got {other:?}"),
    }
}

#[test]
fn a_two_spec_cycle_is_reported_and_never_guessed_through() {
    let tmp = tempfile::tempdir().unwrap();
    write(
        tmp.path(),
        "001-a",
        "moves:\n  - from: \"a.rs\"\n    to: \"b.rs\"\n    kind: relocated\n",
    );
    write(
        tmp.path(),
        "002-b",
        "moves:\n  - from: \"b.rs\"\n    to: \"a.rs\"\n    kind: relocated\n",
    );
    let registry = compiled_registry(tmp.path());
    match lookup(&registry, "a.rs") {
        MoveLookup::Cycle { path, chain } => {
            assert_eq!(path, "a.rs");
            assert_eq!(
                chain,
                vec!["a.rs".to_string(), "b.rs".to_string(), "a.rs".to_string()]
            );
        }
        other => panic!("expected cycle, got {other:?}"),
    }
}

#[test]
fn byte_identical_duplicate_declarations_collapse_to_one_step_naming_both_specs() {
    // D-6: two specs declaring the exact same step is not ambiguity, it is
    // one step two authors happened to both write down.
    let tmp = tempfile::tempdir().unwrap();
    write(
        tmp.path(),
        "001-a",
        "moves:\n  - from: \"a.rs\"\n    to: \"b.rs\"\n    kind: relocated\n",
    );
    write(
        tmp.path(),
        "002-b",
        "moves:\n  - from: \"a.rs\"\n    to: \"b.rs\"\n    kind: relocated\n",
    );
    let registry = compiled_registry(tmp.path());
    match lookup(&registry, "a.rs") {
        MoveLookup::Resolved { hops, .. } => {
            assert_eq!(hops.len(), 1);
            assert_eq!(
                hops[0].declared_by,
                vec!["001-a".to_string(), "002-b".to_string()]
            );
        }
        other => panic!("expected resolved (collapsed), got {other:?}"),
    }
}

#[test]
fn flattened_moves_is_sorted_and_covers_every_branch() {
    let tmp = tempfile::tempdir().unwrap();
    write(
        tmp.path(),
        "001-a",
        "moves:\n  - from: \"z.rs\"\n    to: [\"y.rs\", \"x.rs\"]\n    kind: split\n",
    );
    write(
        tmp.path(),
        "002-b",
        "moves:\n  - from: \"a.rs\"\n    to: null\n    kind: removed\n",
    );
    let registry = compiled_registry(tmp.path());
    let entries = flattened_moves(&registry);
    // Sorted by (from, to, declaredBy): "a.rs" before "z.rs".
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].from, "a.rs");
    assert_eq!(entries[0].to, None);
    assert_eq!(entries[0].declared_by, "002-b");
    assert_eq!(entries[1].from, "z.rs");
    assert_eq!(entries[1].to.as_deref(), Some("x.rs"));
    assert_eq!(entries[2].from, "z.rs");
    assert_eq!(entries[2].to.as_deref(), Some("y.rs"));
}

// ===== 3.5, 3.6: no gate reads a move ========================================
//
// `compile`/`lint` never fail solely because a well-formed move exists: a
// corpus that declares one, with its paths matching the tree, passes both
// gates exactly as one that declares none does. The CLI's `couple`-level
// proof (a move clearing no deletion) is `crates/spec-spine-cli/tests/moves.rs`,
// since it needs a real git history, not this crate's compile/lint surface.

#[test]
fn a_well_formed_move_with_paths_matching_the_tree_passes_compile_and_lint() {
    let tmp = tempfile::tempdir().unwrap();
    write(
        tmp.path(),
        "001-a",
        "moves:\n  - from: \"a.rs\"\n    to: \"b.rs\"\n    kind: relocated\n",
    );
    fs::write(tmp.path().join("b.rs"), "pub fn b() {}\n").unwrap();
    let out = compile(&Config::default(), tmp.path()).unwrap();
    assert!(out.validation_passed, "{:?}", codes(&out));
    let report = lint(&Config::default(), tmp.path()).unwrap();
    assert!(
        !report
            .violations
            .iter()
            .any(|v| v.code == "L-015" || v.code == "L-016"),
        "a well-formed move whose paths match the tree raises no move-specific \
         diagnostic: {:?}",
        report.violations
    );
}
