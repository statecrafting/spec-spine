//! Spec 053: `L-007`, the opt-in ordinal-monotonicity check on `depends_on`.
//!
//! The corpus had no dedicated lint test file before this spec: `L-006`'s
//! acceptance lives in `tests/coverage.rs` and the scaffold's in
//! `tests/scaffold.rs`. This is that file, and spec 053 claims it.

use std::fs;
use std::path::Path;

use spec_spine_types::{Config, Severity, load_config};

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, content).unwrap();
}

/// A minimal well-formed spec. `establishes` keeps `L-001` quiet so the
/// assertions below read against `L-007` alone.
fn spec(id: &str, depends_on: &[&str]) -> String {
    let deps = if depends_on.is_empty() {
        String::new()
    } else {
        let items: String = depends_on
            .iter()
            .map(|d| format!("  - \"{d}\"\n"))
            .collect();
        format!("depends_on:\n{items}")
    };
    format!(
        "---\nid: \"{id}\"\ntitle: \"T\"\nstatus: approved\ncreated: \"2026-09-07\"\n\
         summary: \"s\"\nestablishes:\n  - \"src/{id}.rs\"\n{deps}---\n# {id}\n## body\n"
    )
}

/// The knob on, everything else default.
fn opted_in() -> Config {
    load_config("[lint]\nrequire_ordinal_monotonic_depends_on = true\n").unwrap()
}

fn l007(cfg: &Config, root: &Path) -> Vec<spec_spine_types::Violation> {
    spec_spine_core::lint(cfg, root)
        .unwrap()
        .violations
        .into_iter()
        .filter(|v| v.code == "L-007")
        .collect()
}

/// A corpus with one forward edge (`012 -> 044`) and one backward one
/// (`044 -> 012` would be a cycle, so `044 -> 001`).
fn forward_corpus(root: &Path) {
    write(root, "specs/001-base/spec.md", &spec("001-base", &[]));
    write(
        root,
        "specs/012-early/spec.md",
        &spec("012-early", &["044-late"]),
    );
    write(
        root,
        "specs/044-late/spec.md",
        &spec("044-late", &["001-base"]),
    );
}

/// §3.1: the knob defaults off, and off means *not emitted*, not
/// emitted-and-filtered. A corpus that has not opted in sees byte-identical
/// lint output before and after this spec.
#[test]
fn the_knob_defaults_off_and_off_emits_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    forward_corpus(tmp.path());
    assert!(
        Config::default()
            .lint
            .require_ordinal_monotonic_depends_on
            .eq(&false),
        "the knob defaults off"
    );
    assert!(
        l007(&Config::default(), tmp.path()).is_empty(),
        "a corpus that did not opt in is not told about its edges"
    );
}

/// §3.2: with the knob on, a forward edge is one error-tier `L-007` naming
/// both ids, and the backward edge in the same corpus is silent.
#[test]
fn a_forward_dependency_is_an_error_and_a_backward_one_is_silent() {
    let tmp = tempfile::tempdir().unwrap();
    forward_corpus(tmp.path());
    let found = l007(&opted_in(), tmp.path());

    assert_eq!(found.len(), 1, "only the forward edge fires: {found:?}");
    let v = &found[0];
    assert_eq!(v.severity, Severity::Error, "the knob alone decides");
    assert!(v.message.contains("012-early"), "{}", v.message);
    assert!(v.message.contains("044-late"), "{}", v.message);
    assert!(
        v.message.contains("points backward in filing order"),
        "{}",
        v.message
    );
    // §3.2: the path is the *declaring* spec, so an editor jumping to the
    // diagnostic lands on the file that has to change.
    assert_eq!(v.path.as_deref(), Some("specs/012-early/spec.md"));
}

/// §3.3: equality is a violation, not a pass. Two specs cannot share an
/// ordinal (`V-004` refuses it), so an entry whose ordinal equals the
/// declaring spec's is a spec depending on itself.
#[test]
fn a_self_dependency_is_caught_by_the_equality_arm() {
    let tmp = tempfile::tempdir().unwrap();
    write(
        tmp.path(),
        "specs/007-self/spec.md",
        &spec("007-self", &["007-self"]),
    );
    let found = l007(&opted_in(), tmp.path());
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0].message.contains("007 >= 007"),
        "{}",
        found[0].message
    );
}

/// §3.3: an id with no leading digit run has no ordinal, and silence is the
/// honest answer when the order is undefined. Checked in both positions: the
/// declaring spec and the target.
#[test]
fn an_id_without_an_ordinal_is_silence_in_either_position() {
    let tmp = tempfile::tempdir().unwrap();
    let r = tmp.path();
    write(r, "specs/auth-login/spec.md", &spec("auth-login", &[]));
    // Declaring spec has no ordinal.
    write(
        r,
        "specs/auth-logout/spec.md",
        &spec("auth-logout", &["auth-login"]),
    );
    // Target has no ordinal, declaring spec does.
    write(
        r,
        "specs/003-numbered/spec.md",
        &spec("003-numbered", &["auth-login"]),
    );
    assert!(
        l007(&opted_in(), r).is_empty(),
        "a corpus that never claimed the convention is not held to it"
    );
}

/// §3.3: the comparison is numeric, not lexical, so a corpus that outgrows
/// three digits orders `1001` above `999` instead of below it.
#[test]
fn the_comparison_is_numeric_not_lexical() {
    let tmp = tempfile::tempdir().unwrap();
    let r = tmp.path();
    write(r, "specs/999-nine/spec.md", &spec("999-nine", &[]));
    // 1001 -> 999 decreases numerically; lexically "1001" < "999" would have
    // called this forward and fired.
    write(
        r,
        "specs/1001-thousand/spec.md",
        &spec("1001-thousand", &["999-nine"]),
    );
    assert!(
        l007(&opted_in(), r).is_empty(),
        "1001 -> 999 points backward"
    );

    // And the genuine inversion still fires.
    let tmp2 = tempfile::tempdir().unwrap();
    let r2 = tmp2.path();
    write(
        r2,
        "specs/1001-thousand/spec.md",
        &spec("1001-thousand", &[]),
    );
    write(
        r2,
        "specs/999-nine/spec.md",
        &spec("999-nine", &["1001-thousand"]),
    );
    assert_eq!(l007(&opted_in(), r2).len(), 1);
}

/// §3.4: `L-007` is a lint and never a `V-` code. The corpus above compiles,
/// its shard is valid, and `compile` reports no violation of its own.
#[test]
fn a_forward_dependency_does_not_fail_compile() {
    let tmp = tempfile::tempdir().unwrap();
    forward_corpus(tmp.path());
    let outcome = spec_spine_core::compile(&opted_in(), tmp.path()).unwrap();
    assert!(
        outcome.registry.validation.passed,
        "a convention the corpus may not hold is not a structural defect: {:?}",
        outcome.registry.validation.violations
    );
    assert!(
        !outcome
            .registry
            .validation
            .violations
            .iter()
            .any(|v| v.code == "L-007"),
        "the lint's code never appears in compile's report"
    );
}

// ── spec 057: a claim no hash witnesses ───────────────────────────────────

/// Commit the index shard tree, as `spec-spine index` does. `L-008` reads the
/// committed index rather than recomputing one, so a corpus without this step
/// is silent: there is no ledger yet for a claim to be invisible to.
fn commit_index(cfg: &Config, root: &Path) {
    use spec_spine_core::shard::{self, BY_PACKAGE_DIR, BY_SPEC_DIR};
    let outcome = spec_spine_core::index(cfg, root).unwrap();
    let dir = spec_spine_core::index_dir(cfg, root);
    let (by_spec, by_package) = spec_spine_core::index_shard_files(&outcome.shards).unwrap();
    shard::sync_dir(&dir.join(BY_SPEC_DIR), &by_spec).unwrap();
    shard::sync_dir(&dir.join(BY_PACKAGE_DIR), &by_package).unwrap();
}

fn l008(cfg: &Config, root: &Path) -> Vec<spec_spine_types::Violation> {
    spec_spine_core::lint(cfg, root)
        .unwrap()
        .violations
        .into_iter()
        .filter(|v| v.code == "L-008")
        .collect()
}

/// A corpus claiming one existing file that nothing hashes.
fn unwitnessed_fixture(root: &Path) {
    write(root, "thing.sh", "echo claimed\n");
    write(
        root,
        "specs/001-x/spec.md",
        "---\nid: \"001-x\"\ntitle: \"x\"\nstatus: draft\ncreated: \"2026-09-07\"\n\
         summary: \"x\"\nestablishes:\n  - \"thing.sh\"\n---\n# x\n## body\n",
    );
}

/// §3.2: a claimed file that exists and that no content hash covers is one
/// warning naming the spec, the path, and both remedies. They are genuinely
/// different choices and the right one depends on the file.
#[test]
fn an_unwitnessed_claim_is_an_l008_warning_naming_both_remedies() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg = Config::default();
    unwitnessed_fixture(tmp.path());
    commit_index(&cfg, tmp.path());

    let found = l008(&cfg, tmp.path());
    assert_eq!(found.len(), 1, "{found:?}");
    let v = &found[0];
    assert_eq!(
        v.severity,
        Severity::Warning,
        "a legitimate state, not a defect"
    );
    assert!(v.message.contains("001-x"), "{}", v.message);
    assert!(v.message.contains("thing.sh"), "{}", v.message);
    assert!(v.message.contains("extra_hashed_inputs"), "{}", v.message);
    assert!(
        v.message.contains("section or symbol unit"),
        "{}",
        v.message
    );
    // Spec 074 3.2, conformance with 057: naming both remedies is not enough.
    // 057 requires the message to say how they DIFFER, and the shipped string
    // dropped that, so it read as a pick-either and sent an adopter to the
    // wrong one. A glob restamps every shard; a span-backed unit stales one.
    assert!(
        v.message.contains("EVERY shard"),
        "the glob's cost must be stated: {}",
        v.message
    );
    assert!(
        v.message.contains("span"),
        "the unit's narrower blast radius must be stated: {}",
        v.message
    );
    assert_eq!(v.path.as_deref(), Some("specs/001-x/spec.md"));
}

/// §3.2: a claimed path that does not exist produces no `L-008`. It is already
/// diagnosed as an unresolved unit (`W-001` / `W-002`), and a second warning on
/// every pending claim would make a specify-first corpus unreadable.
#[test]
fn a_claim_on_a_file_that_does_not_exist_is_silent() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg = Config::default();
    write(
        tmp.path(),
        "specs/001-x/spec.md",
        "---\nid: \"001-x\"\ntitle: \"x\"\nstatus: draft\ncreated: \"2026-09-07\"\n\
         summary: \"x\"\nestablishes:\n  - \"not/written/yet.rs\"\n---\n# x\n## body\n",
    );
    commit_index(&cfg, tmp.path());
    assert!(l008(&cfg, tmp.path()).is_empty());
}

/// §3.1: a covering `extra_hashed_inputs` glob is one of the two remedies the
/// message names, and it works. This also pins the glob form: `dir/**` matches
/// directories only, which is how this repository's own two entries came to
/// match nothing at all.
#[test]
fn a_covering_glob_witnesses_the_claim_and_the_directory_form_does_not() {
    let tmp = tempfile::tempdir().unwrap();
    unwitnessed_fixture(tmp.path());

    let works = load_config("[index]\nextra_hashed_inputs = [\"*.sh\"]\n").unwrap();
    commit_index(&works, tmp.path());
    assert!(
        l008(&works, tmp.path()).is_empty(),
        "a matching glob covers it"
    );

    // The trap: `dir/**` matches directories, so it hashes no files.
    write(tmp.path(), "sub/thing2.sh", "echo two\n");
    write(
        tmp.path(),
        "specs/002-y/spec.md",
        "---\nid: \"002-y\"\ntitle: \"y\"\nstatus: draft\ncreated: \"2026-09-07\"\n\
         summary: \"y\"\nestablishes:\n  - \"sub/thing2.sh\"\n---\n# y\n## body\n",
    );
    let trap = load_config("[index]\nextra_hashed_inputs = [\"*.sh\", \"sub/**\"]\n").unwrap();
    commit_index(&trap, tmp.path());
    assert!(
        l008(&trap, tmp.path())
            .iter()
            .any(|v| v.message.contains("sub/thing2.sh")),
        "`sub/**` matches directories, not the files under them"
    );

    let fixed = load_config("[index]\nextra_hashed_inputs = [\"*.sh\", \"sub/**/*\"]\n").unwrap();
    commit_index(&fixed, tmp.path());
    assert!(
        l008(&fixed, tmp.path()).is_empty(),
        "`sub/**/*` matches them"
    );
}

/// §3.2 + the config knob: an allowlisted path is suppressed from `L-008` and
/// still counted, so an exception is explicit rather than invisible.
#[test]
fn the_allowlist_suppresses_the_warning_but_not_the_count() {
    let tmp = tempfile::tempdir().unwrap();
    unwitnessed_fixture(tmp.path());
    let cfg = load_config("[lint]\nunwitnessed_allowed = [\"*.sh\"]\n").unwrap();
    commit_index(&cfg, tmp.path());

    assert!(l008(&cfg, tmp.path()).is_empty(), "suppressed");

    let index = spec_spine_core::load_committed_index(&cfg, tmp.path()).unwrap();
    let claims = spec_spine_core::unwitnessed_claims(&cfg, tmp.path(), &index);
    assert_eq!(claims.len(), 1, "still counted: {claims:?}");
    assert!(claims[0].allowed, "and marked as a declared exception");
}

/// §3.1: a claim inside the ungoverned state root is `L-006`'s business, at
/// error tier. "You claimed the ungoverned directory" is the whole diagnosis,
/// and "and it is not hashed" is noise on a path that has to move.
#[test]
fn a_claim_inside_the_state_root_defers_to_l006() {
    let tmp = tempfile::tempdir().unwrap();
    write(tmp.path(), "tool-state/journal.rs", "// state\n");
    write(
        tmp.path(),
        "specs/001-x/spec.md",
        "---\nid: \"001-x\"\ntitle: \"x\"\nstatus: draft\ncreated: \"2026-09-07\"\n\
         summary: \"x\"\nestablishes:\n  - \"tool-state/journal.rs\"\n---\n# x\n## body\n",
    );
    let cfg = load_config("[layout]\nstate_dir = \"tool-state\"\n").unwrap();
    commit_index(&cfg, tmp.path());

    let report = spec_spine_core::lint(&cfg, tmp.path()).unwrap();
    assert!(
        report.violations.iter().any(|v| v.code == "L-006"),
        "{:?}",
        report.violations
    );
    assert!(
        !report.violations.iter().any(|v| v.code == "L-008"),
        "L-008 defers: {:?}",
        report.violations
    );
}

/// §3.1: the predicate is derived from what the hashers actually consume, so
/// each of the four contributors is witnessed.
#[test]
fn witnessed_paths_covers_every_hash_contributor() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write(root, "Cargo.toml", "[workspace]\nmembers = [\"crate-a\"]\n");
    write(
        root,
        "crate-a/Cargo.toml",
        "[package]\nname = \"crate-a\"\nversion = \"0.1.0\"\n",
    );
    write(
        root,
        "spec-spine.toml",
        "[index]\nextra_hashed_inputs = [\"extra.txt\"]\n",
    );
    write(root, "extra.txt", "hashed\n");
    write(
        root,
        "specs/001-x/spec.md",
        "---\nid: \"001-x\"\ntitle: \"x\"\nstatus: draft\ncreated: \"2026-09-07\"\n\
         summary: \"x\"\nestablishes:\n  - \"src/x.rs\"\n---\n# x\n## body\n",
    );
    let cfg = load_config(&std::fs::read_to_string(root.join("spec-spine.toml")).unwrap()).unwrap();
    let index = spec_spine_core::index(&cfg, root).unwrap().index;
    let witnessed = spec_spine_core::witnessed_paths(&cfg, root, &index);

    assert!(witnessed.contains("specs/001-x/spec.md"), "{witnessed:?}");
    assert!(witnessed.contains("crate-a/Cargo.toml"), "{witnessed:?}");
    assert!(witnessed.contains("spec-spine.toml"), "{witnessed:?}");
    assert!(witnessed.contains("extra.txt"), "{witnessed:?}");
}

// ── spec 058: the defects heading has one spelling ────────────────────────

use spec_spine_core::sections::{anchor_of, is_defects_anchor, is_near_miss_defects_anchor};

/// §3.1: the heading is an anchor, not a string. Case, level and section
/// numbering all come free from the indexer's own slug rule, which is why the
/// rule is that rule rather than a second copy of it.
#[test]
fn the_defects_heading_is_matched_by_anchor_at_any_level_or_numbering() {
    for heading in [
        "Known defects",
        "Known Defects",
        "KNOWN DEFECTS",
        "Known defects:",
        "5. Known defects",
        "4. Known defects",
    ] {
        assert!(
            is_defects_anchor(&anchor_of(heading)),
            "'{heading}' -> '{}' should be the defects anchor",
            anchor_of(heading)
        );
    }
}

/// §3.1: suffix, not prefix or substring. A heading that continues past the
/// anchor is a section about something wider, and a consumer extracting defects
/// from it would extract the open questions too.
#[test]
fn a_heading_that_continues_past_the_anchor_is_not_the_section() {
    for heading in [
        "Known defects and open questions",
        "Known defect",
        "Defects",
        "Why known defects matter",
    ] {
        assert!(
            !is_defects_anchor(&anchor_of(heading)),
            "'{heading}' must not be the defects anchor"
        );
    }
}

/// §3.2: the near-miss rule catches an attempt at the section, and not prose
/// that merely mentions defects. This spec's own title is the case that
/// narrowed the rule: `# 058: The defects heading has one spelling` is about
/// defects and is not an attempt at a defects section.
#[test]
fn the_near_miss_rule_catches_attempts_and_not_prose() {
    for heading in [
        "Known defect",
        "Defects",
        "Known defects and open questions",
    ] {
        assert!(
            is_near_miss_defects_anchor(&anchor_of(heading)),
            "'{heading}' is an attempt that a consumer will not find"
        );
    }
    for heading in [
        "058: The defects heading has one spelling",
        "Known defects",
        "5. Known defects",
        "Purpose",
    ] {
        assert!(
            !is_near_miss_defects_anchor(&anchor_of(heading)),
            "'{heading}' must not be reported"
        );
    }
}

/// §3.2: `L-009` is info tier, so it surfaces under `--fail-on-info` and
/// refuses nothing at the tier this repository gates on.
#[test]
fn l009_is_info_tier_and_names_the_anchor() {
    let tmp = tempfile::tempdir().unwrap();
    write(
        tmp.path(),
        "specs/001-x/spec.md",
        "---\nid: \"001-x\"\ntitle: \"x\"\nstatus: draft\ncreated: \"2026-09-07\"\n\
         summary: \"x\"\nestablishes:\n  - \"src/x.rs\"\n---\n# x\n\n## Known defect\n\nprose\n",
    );
    let report = spec_spine_core::lint(&Config::default(), tmp.path()).unwrap();
    let found: Vec<_> = report
        .violations
        .iter()
        .filter(|v| v.code == "L-009")
        .collect();
    assert_eq!(found.len(), 1, "{:?}", report.violations);
    assert_eq!(found[0].severity, Severity::Info);
    assert!(
        found[0].message.contains("known-defect"),
        "{}",
        found[0].message
    );
    assert!(
        found[0].message.contains("known-defects"),
        "{}",
        found[0].message
    );
    assert_eq!(found[0].path.as_deref(), Some("specs/001-x/spec.md"));
}

/// §3.3: `origin.retroactive: true` does not imply the heading. The
/// counterexample is in this corpus: spec 000 declares it and rightly has no
/// defects section, so a lint implementing the audit's proposal would fire on
/// the tier-1 bootstrap spec, which is both wrong and unfixable.
#[test]
fn retroactive_origin_alone_produces_no_diagnostic() {
    let tmp = tempfile::tempdir().unwrap();
    write(
        tmp.path(),
        "specs/001-x/spec.md",
        "---\nid: \"001-x\"\ntitle: \"x\"\nstatus: draft\ncreated: \"2026-09-07\"\n\
         summary: \"x\"\nestablishes:\n  - \"src/x.rs\"\norigin:\n  retroactive: true\n\
         ---\n# x\n\n## Purpose\n\nprose\n",
    );
    let report = spec_spine_core::lint(&Config::default(), tmp.path()).unwrap();
    assert!(
        !report.violations.iter().any(|v| v.code == "L-009"),
        "{:?}",
        report.violations
    );
}

// ── spec 074 3.1 and 3.10: L-010, a pattern that can match no file ──────────

/// A corpus with one well-formed spec and the given `extra_hashed_inputs`.
fn corpus_with_hashed_inputs(patterns: &[&str]) -> (tempfile::TempDir, Config) {
    let tmp = tempfile::tempdir().unwrap();
    write(tmp.path(), "specs/001-a/spec.md", &spec("001-a", &[]));
    write(tmp.path(), "src/001-a.rs", "pub fn a() {}\n");
    let list: String = patterns
        .iter()
        .map(|p| format!("\"{p}\", "))
        .collect::<String>();
    let cfg = load_config(&format!("[index]\nextra_hashed_inputs = [{list}]\n")).unwrap();
    (tmp, cfg)
}

fn codes(tmp: &tempfile::TempDir, cfg: &Config) -> Vec<(String, Severity)> {
    spec_spine_core::lint::lint(cfg, tmp.path())
        .unwrap()
        .violations
        .into_iter()
        .map(|v| (v.code, v.severity))
        .collect()
}

/// Spec 074 3.1: `dir/**` enumerates directories and the hasher keeps only
/// files, so the pattern can contribute no bytes to any content hash whatever
/// the tree contains. Two adopters hit this on the same day.
#[test]
fn l010_refuses_a_hashed_input_pattern_that_can_match_no_file() {
    let (tmp, cfg) = corpus_with_hashed_inputs(&["standards/**", "crates/**"]);
    let found = codes(&tmp, &cfg);
    let l010: Vec<_> = found.iter().filter(|(c, _)| c == "L-010").collect();
    assert_eq!(l010.len(), 2, "one per broken pattern: {found:?}");
    // Warning tier, so `lint --fail-on-warn` refuses it. Unlike `L-008` this is
    // never a state a corpus holds deliberately: an adopter who wants to match
    // nothing writes no entry.
    assert!(
        l010.iter().all(|(_, sev)| *sev == Severity::Warning),
        "L-010 must be warning tier: {found:?}"
    );
}

/// The other half of 3.10's requirement: it must NOT fire on the working form.
/// A rule that flagged both would just be noise on the fix it recommends.
#[test]
fn l010_is_silent_on_the_working_glob_form() {
    let (tmp, cfg) = corpus_with_hashed_inputs(&["standards/**/*", ".github/workflows/**/*"]);
    assert!(
        !codes(&tmp, &cfg).iter().any(|(c, _)| c == "L-010"),
        "the working form must not be flagged"
    );
}

/// Spec 074 3.1: the check is on the PATTERN, not on whether it currently
/// matches. A forward-looking entry in a specify-first corpus matches nothing
/// today and is legitimate; that false positive is why spec 069 4 deferred this
/// lint, and narrowing to the one unconditionally inert form is the answer.
#[test]
fn l010_does_not_fire_on_a_pattern_that_merely_matches_nothing_yet() {
    let (tmp, cfg) = corpus_with_hashed_inputs(&["not/written/yet/**/*", "future.md"]);
    assert!(
        !codes(&tmp, &cfg).iter().any(|(c, _)| c == "L-010"),
        "a pattern that matches nothing YET is not the same as one that never can"
    );
}

/// Spec 074 3.2: the message must name the working form, because the entire
/// cost of this defect is that the broken form looks correct.
#[test]
fn the_l010_message_names_the_working_form() {
    let (tmp, cfg) = corpus_with_hashed_inputs(&["standards/**"]);
    let msg = spec_spine_core::lint::lint(&cfg, tmp.path())
        .unwrap()
        .violations
        .into_iter()
        .find(|v| v.code == "L-010")
        .expect("L-010 fired")
        .message;
    assert!(msg.contains("standards/**/*"), "{msg}");
}

// ── spec 076 §3.3 and §3.4: the flag cannot outlive the work ────────────────

/// A corpus whose one spec claims `unit_yaml` at the given lifecycle.
fn planned_corpus(unit_yaml: &str, status: &str, implementation: &str) -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    write(
        tmp.path(),
        "specs/001-a/spec.md",
        &format!(
            "---\nid: \"001-a\"\ntitle: \"A\"\nstatus: {status}\ncreated: \"2026-09-08\"\n\
             implementation: {implementation}\nsummary: \"s\"\nestablishes:\n{unit_yaml}\
             ---\n# 001-a\n## body\n"
        ),
    );
    tmp
}

fn lint_codes(tmp: &tempfile::TempDir) -> Vec<(String, Severity)> {
    spec_spine_core::lint::lint(&Config::default(), tmp.path())
        .unwrap()
        .violations
        .into_iter()
        .map(|v| (v.code, v.severity))
        .collect()
}

/// §3.3: completion asserts the work is done and a planned unit asserts it is
/// not, so the two together are a contradiction inside one file's frontmatter.
/// Error tier, not warning: it is not a state a corpus holds deliberately.
#[test]
fn l011_refuses_a_completed_spec_that_still_plans_territory() {
    let tmp = planned_corpus(
        "  - { kind: file, path: \"src/later.rs\", planned: true }\n",
        "approved",
        "complete",
    );
    let found = lint_codes(&tmp);
    assert!(
        found.contains(&("L-011".to_string(), Severity::Error)),
        "L-011 must fire at error tier: {found:?}"
    );
}

/// §3.3: `complete` is the only bound, deliberately. An `approved` spec at
/// `pending` may carry planned units for as long as that state is honest, which
/// on a specify-first corpus is months. An intermediate deadline would mean
/// inventing a clock.
#[test]
fn l011_is_silent_while_the_work_is_genuinely_in_flight() {
    for implementation in ["pending", "in-progress"] {
        let tmp = planned_corpus(
            "  - { kind: file, path: \"src/later.rs\", planned: true }\n",
            "approved",
            implementation,
        );
        assert!(
            !lint_codes(&tmp).iter().any(|(c, _)| c == "L-011"),
            "{implementation}: a spec still doing the work may plan territory"
        );
    }
}

/// §3.4: the other direction. Without it the flag rots: the file lands, the
/// claim is satisfied, and nothing notices that the spec still calls it future
/// work. Warning tier, so `--fail-on-warn` refuses it in a corpus running the
/// gate without imposing it on one that does not.
#[test]
fn l012_reports_a_planned_unit_that_has_resolved() {
    let tmp = planned_corpus(
        "  - { kind: file, path: \"src/landed.rs\", planned: true }\n",
        "draft",
        "pending",
    );
    write(tmp.path(), "src/landed.rs", "pub fn a() {}\n");
    write(tmp.path(), "Cargo.toml", "[workspace]\nmembers = []\n");
    // `L-012` reads the COMMITTED index, by the same rule `L-008` follows: lint
    // is a read verb, and indexing inside it would make it write-shaped.
    let out = spec_spine_core::index(&Config::default(), tmp.path()).unwrap();
    let dir = spec_spine_core::index_dir(&Config::default(), tmp.path());
    let (by_spec, by_package) = spec_spine_core::index_shard_files(&out.shards).unwrap();
    spec_spine_core::shard::sync_dir(&dir.join(spec_spine_core::shard::BY_SPEC_DIR), &by_spec)
        .unwrap();
    spec_spine_core::shard::sync_dir(
        &dir.join(spec_spine_core::shard::BY_PACKAGE_DIR),
        &by_package,
    )
    .unwrap();

    let found = lint_codes(&tmp);
    assert!(
        found.contains(&("L-012".to_string(), Severity::Warning)),
        "L-012 must fire at warning tier once the claim lands: {found:?}"
    );
}

/// §3.4: and it must not fire while the claim is honestly unwritten, or the
/// diagnostic would just be a second name for "planned".
#[test]
fn l012_is_silent_while_the_planned_unit_is_still_unwritten() {
    let tmp = planned_corpus(
        "  - { kind: file, path: \"src/never.rs\", planned: true }\n",
        "draft",
        "pending",
    );
    assert!(!lint_codes(&tmp).iter().any(|(c, _)| c == "L-012"));
}

// ── spec 076 §3.5: collisions reuse the ownership rules ─────────────────────

/// Write one spec with arbitrary extra frontmatter lines.
fn spec_with(tmp: &tempfile::TempDir, id: &str, extra: &str) {
    write(
        tmp.path(),
        &format!("specs/{id}/spec.md"),
        &format!(
            "---\nid: \"{id}\"\ntitle: \"T\"\nstatus: approved\ncreated: \"2026-09-08\"\n\
             implementation: pending\nsummary: \"s\"\n{extra}---\n# {id}\n## body\n"
        ),
    );
}

fn compile_codes(tmp: &tempfile::TempDir) -> Vec<String> {
    spec_spine_core::compile(&Config::default(), tmp.path())
        .unwrap()
        .registry
        .validation
        .violations
        .into_iter()
        .map(|v| v.code)
        .collect()
}

/// §3.5: two specs planning the same unit is the duplicate-ownership refusal.
/// Decided over DECLARED units at compile time, because the existing machinery
/// runs on `TraceMapping` and §3.2 keeps a planned unit from ever producing
/// one: a rule that cannot fire is worse than no rule.
#[test]
fn v015_refuses_two_specs_planning_the_same_unit() {
    let tmp = tempfile::tempdir().unwrap();
    let unit = "establishes:\n  - { kind: file, path: \"src/x.rs\", planned: true }\n";
    spec_with(&tmp, "001-a", unit);
    spec_with(&tmp, "002-b", unit);
    assert!(
        compile_codes(&tmp).iter().any(|c| c == "V-015"),
        "{:?}",
        compile_codes(&tmp)
    );
}

/// §3.5: with the same `co_authority` escape as any other shared claim, so the
/// new rule introduces no second ownership vocabulary.
#[test]
fn v015_allows_a_shared_plan_declared_as_co_authority() {
    let tmp = tempfile::tempdir().unwrap();
    let unit = "establishes:\n  - { kind: file, path: \"src/x.rs\", planned: true }\n\
                co_authority:\n  - { unit: { kind: file, path: \"src/x.rs\" }, with_specs: [\"002-b\"] }\n";
    spec_with(&tmp, "001-a", unit);
    spec_with(
        &tmp,
        "002-b",
        "establishes:\n  - { kind: file, path: \"src/x.rs\", planned: true }\n",
    );
    assert!(
        !compile_codes(&tmp).iter().any(|c| c == "V-015"),
        "a genuinely shared claim has an escape: {:?}",
        compile_codes(&tmp)
    );
}

/// §3.5: planning a unit another spec already owns is refused. The correct
/// declaration is an `extends` edge naming that spec and unit, which is what
/// crossing into owned territory has always meant.
#[test]
fn v016_refuses_planning_territory_another_spec_already_owns() {
    let tmp = tempfile::tempdir().unwrap();
    spec_with(&tmp, "001-a", "establishes:\n  - \"src/x.rs\"\n");
    spec_with(
        &tmp,
        "002-b",
        "establishes:\n  - { kind: file, path: \"src/x.rs\", planned: true }\n",
    );
    assert!(
        compile_codes(&tmp).iter().any(|c| c == "V-016"),
        "{:?}",
        compile_codes(&tmp)
    );
}

/// §3.5: a planned unit must not be the target of an `extends` edge. There is
/// nothing to extend: the unit does not exist and its planner does not own it.
#[test]
fn v017_refuses_extending_a_unit_that_is_only_planned() {
    let tmp = tempfile::tempdir().unwrap();
    spec_with(
        &tmp,
        "001-a",
        "establishes:\n  - { kind: file, path: \"src/x.rs\", planned: true }\n",
    );
    spec_with(
        &tmp,
        "002-b",
        "extends:\n  - { spec: \"001-a\", unit: { kind: file, path: \"src/x.rs\" }, nature: additive }\n",
    );
    assert!(
        compile_codes(&tmp).iter().any(|c| c == "V-017"),
        "{:?}",
        compile_codes(&tmp)
    );
}

/// The corpus this repository actually has must stay clean, or the three new
/// refusals would be a change of behaviour dressed as a new rule.
#[test]
fn the_planned_refusals_are_silent_on_a_corpus_that_plans_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    spec_with(&tmp, "001-a", "establishes:\n  - \"src/x.rs\"\n");
    spec_with(
        &tmp,
        "002-b",
        "extends:\n  - { spec: \"001-a\", unit: \"src/x.rs\", nature: additive }\n",
    );
    let codes = compile_codes(&tmp);
    for code in ["V-015", "V-016", "V-017"] {
        assert!(!codes.iter().any(|c| c == code), "{code} fired: {codes:?}");
    }
}
