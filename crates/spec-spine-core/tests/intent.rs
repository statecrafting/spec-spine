//! Spec 114: an intent is a standing, optional declaration on a spec.
//!
//! What is asserted: a well-formed intent reaches the registry record as
//! authored; a spec without one has an unchanged record; every malformed shape
//! is refused as a declaration, naming the spec; and a well-formed intent moves
//! no verdict on equivalent fresh trees. The refusals and the neutrality are
//! asserted side by side on purpose (114 D-1): validating a declaration is not
//! enforcing what it says.

use std::fs;
use std::path::Path;

use spec_spine_core::shard::{self, BY_PACKAGE_DIR, BY_SPEC_DIR};
use spec_spine_core::{
    DiffFile, DiffInput, check_report, compile, couple, coverage, index, index_dir,
    index_shard_files, lint, registry_dir, registry_shard_files,
};
use spec_spine_types::{Config, load_config};

const INTENT: &str = "\
intent:
  goal: \"a correct removal stops being refused\"
  non_goals:
    - \"changing what C-001 means for an unowned deletion\"
    - \"pairing a deletion with an addition\"
";

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, content).unwrap();
}

fn spec(id: &str, unit: &str, extra: &str) -> String {
    format!(
        "---\nid: \"{id}\"\ntitle: \"T\"\nstatus: approved\ncreated: \"2026-09-23\"\n\
         summary: \"s\"\nestablishes:\n  - \"{unit}\"\n{extra}---\n# {id}\n\n\
         ## 1. Purpose\n\nWhy.\n\n## 4. Out of scope\n\nWhat not.\n"
    )
}

fn corpus(root: &Path, a_extra: &str) -> Config {
    write(root, "Cargo.toml", "[workspace]\nmembers = [\"crate-a\"]\n");
    write(
        root,
        "crate-a/Cargo.toml",
        "[package]\nname = \"crate-a\"\nversion = \"0.1.0\"\n",
    );
    write(root, "crate-a/src/a.rs", "pub fn a() {}\n");
    write(root, "crate-a/src/b.rs", "pub fn b() {}\n");
    write(
        root,
        "specs/001-a/spec.md",
        &spec("001-a", "crate-a/src/a.rs", a_extra),
    );
    write(
        root,
        "specs/002-b/spec.md",
        &spec("002-b", "crate-a/src/b.rs", ""),
    );
    let toml = "[coupling]\nrequire_ownership = true\n";
    write(root, "spec-spine.toml", toml);
    load_config(toml).unwrap()
}

#[test]
fn a_well_formed_intent_reaches_the_record_as_authored() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let cfg = corpus(root, INTENT);
    let outcome = compile(&cfg, root).unwrap();
    assert!(
        outcome.validation_passed,
        "{:?}",
        outcome.registry.validation.violations
    );
    let a = outcome
        .registry
        .specs
        .iter()
        .find(|s| s.id == "001-a")
        .unwrap();
    let intent = a.intent.as_ref().expect("001-a declares an intent");
    assert_eq!(intent.goal, "a correct removal stops being refused");
    assert_eq!(
        intent.non_goals,
        [
            "changing what C-001 means for an unowned deletion",
            "pairing a deletion with an addition"
        ],
        "non-goals keep their authored order"
    );

    // The emitted member is `intent: { goal, nonGoals }` on the record.
    let v: serde_json::Value = serde_json::to_value(a).unwrap();
    assert_eq!(
        v["intent"],
        serde_json::json!({
            "goal": "a correct removal stops being refused",
            "nonGoals": [
                "changing what C-001 means for an unowned deletion",
                "pairing a deletion with an addition"
            ]
        })
    );

    // `non_goals` is optional.
    let cfg = corpus(root, "intent:\n  goal: \"a goal without exclusions\"\n");
    let outcome = compile(&cfg, root).unwrap();
    assert!(outcome.validation_passed);
    let a = outcome
        .registry
        .specs
        .iter()
        .find(|s| s.id == "001-a")
        .unwrap();
    let v: serde_json::Value = serde_json::to_value(a).unwrap();
    assert_eq!(
        v["intent"],
        serde_json::json!({"goal": "a goal without exclusions"}),
        "an empty non-goal list is omitted"
    );
}

#[test]
fn a_spec_without_an_intent_has_no_member() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let cfg = corpus(root, INTENT);
    let outcome = compile(&cfg, root).unwrap();
    let b = outcome
        .registry
        .specs
        .iter()
        .find(|s| s.id == "002-b")
        .unwrap();
    assert!(b.intent.is_none());
    let v: serde_json::Value = serde_json::to_value(b).unwrap();
    assert!(
        v.get("intent").is_none(),
        "the key is optional and its absence leaves the record unchanged: {v}"
    );
}

#[test]
fn every_malformed_intent_is_refused_naming_the_spec() {
    // §3.4. Each shape is refused as a validation error on the declaring
    // spec, and the other spec in the corpus is unaffected.
    let cases: &[(&str, &str)] = &[
        ("not a mapping", "intent: \"a sentence\"\n"),
        ("no goal", "intent:\n  non_goals: [\"x\"]\n"),
        ("empty goal", "intent:\n  goal: \"\"\n"),
        ("whitespace goal", "intent:\n  goal: \"   \"\n"),
        ("goal not a string", "intent:\n  goal: [\"a\", \"b\"]\n"),
        (
            "empty non-goal",
            "intent:\n  goal: \"g\"\n  non_goals: [\"x\", \"\"]\n",
        ),
        (
            "whitespace non-goal",
            "intent:\n  goal: \"g\"\n  non_goals: [\"  \\t \"]\n",
        ),
        (
            "non-goal not a string",
            "intent:\n  goal: \"g\"\n  non_goals: [{a: 1}]\n",
        ),
        (
            "attempt-specific member",
            "intent:\n  goal: \"g\"\n  approach: \"how this attempt does it\"\n",
        ),
        (
            "camelCase member",
            "intent:\n  goal: \"g\"\n  nonGoals: [\"x\"]\n",
        ),
    ];
    for (name, extra) in cases {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let cfg = corpus(root, extra);
        let outcome = compile(&cfg, root).unwrap();
        assert!(!outcome.validation_passed, "{name}: must be refused");
        let errors: Vec<_> = outcome
            .registry
            .validation
            .violations
            .iter()
            .filter(|v| v.severity == spec_spine_types::Severity::Error)
            .collect();
        assert_eq!(errors.len(), 1, "{name}: exactly one refusal: {errors:?}");
        let e = errors[0];
        assert!(
            e.code == "V-002" || e.code == "V-039",
            "{name}: malformed frontmatter or an empty intent string: {e:?}"
        );
        assert_eq!(
            e.path.as_deref(),
            Some("specs/001-a/spec.md"),
            "{name}: names the declaring spec: {e:?}"
        );
    }
}

#[test]
fn an_empty_string_is_refused_by_its_own_code() {
    // The empty and whitespace-only cases are the ones serde cannot see, so
    // they carry their own code and say which member is empty.
    for (extra, member) in [
        ("intent:\n  goal: \"  \"\n", "goal"),
        (
            "intent:\n  goal: \"g\"\n  non_goals: [\"ok\", \"\"]\n",
            "non_goals[1]",
        ),
    ] {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let cfg = corpus(root, extra);
        let outcome = compile(&cfg, root).unwrap();
        let v = outcome
            .registry
            .validation
            .violations
            .iter()
            .find(|v| v.code == "V-039")
            .unwrap_or_else(|| panic!("V-039 for {member}"));
        assert!(
            v.message.contains("'001-a'") && v.message.contains(member),
            "{}",
            v.message
        );
    }
}

fn regenerate(cfg: &Config, root: &Path) {
    let outcome = compile(cfg, root).unwrap();
    assert!(outcome.validation_passed);
    shard::sync_dir(
        &registry_dir(cfg, root).join(BY_SPEC_DIR),
        &registry_shard_files(&outcome.shards).unwrap(),
    )
    .unwrap();
    let ix = index(cfg, root).unwrap();
    let dir = index_dir(cfg, root);
    let (by_spec, by_package) = index_shard_files(&ix.shards).unwrap();
    shard::sync_dir(&dir.join(BY_SPEC_DIR), &by_spec).unwrap();
    shard::sync_dir(&dir.join(BY_PACKAGE_DIR), &by_package).unwrap();
}

#[test]
fn a_valid_intent_moves_no_verdict() {
    // §3.5, on equivalent fresh trees: every gate verb answers the same with
    // and without a well-formed intent. The couple diffs exercise a refusal,
    // a clearance and an ownership refusal, so equality is not two empties.
    #[derive(Debug, PartialEq)]
    struct Answers {
        compile: Vec<(String, String)>,
        check: (bool, bool, bool),
        lint: Vec<(String, String)>,
        coverage: (usize, usize, Vec<String>),
        couple: Vec<Vec<(String, String)>>,
    }
    let answers = |extra: &str| {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let cfg = corpus(root, extra);
        regenerate(&cfg, root);
        let chk = check_report(&cfg, root).unwrap();
        let cov = coverage(&cfg, root).unwrap();
        let diffs: [&[&str]; 3] = [
            &["crate-a/src/a.rs"],
            &["crate-a/src/a.rs", "specs/001-a/spec.md"],
            &["crate-a/src/c.rs"],
        ];
        Answers {
            compile: compile(&cfg, root)
                .unwrap()
                .registry
                .validation
                .violations
                .into_iter()
                .map(|v| (v.code, v.message))
                .collect(),
            check: (
                chk.registry.fresh,
                chk.index.fresh,
                chk.registry.validation_passed,
            ),
            lint: lint(&cfg, root)
                .unwrap()
                .violations
                .into_iter()
                .map(|v| (v.code, v.message))
                .collect(),
            coverage: (cov.source_files, cov.claimed_files, cov.unclaimed_files),
            couple: diffs
                .iter()
                .map(|paths| {
                    let diff = DiffInput {
                        files: paths
                            .iter()
                            .map(|p| DiffFile {
                                path: p.to_string(),
                                hunks: Vec::new(),
                                deleted: false,
                            })
                            .collect(),
                    };
                    couple(&cfg, root, &diff, None)
                        .unwrap()
                        .violations
                        .into_iter()
                        .map(|v| (v.code, v.message))
                        .collect()
                })
                .collect(),
        }
    };
    let with = answers(INTENT);
    let without = answers("");
    assert!(with.check.0 && with.check.1 && with.check.2, "{with:?}");
    assert_eq!(
        with.couple.iter().map(Vec::len).collect::<Vec<_>>(),
        [1, 0, 1],
        "{with:?}"
    );
    assert_eq!(with, without);
}
