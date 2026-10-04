// Spec: specs/158-the-affected-selection-is-a-governed-read/spec.md
//! The affected-acceptance selection (spec 158): one case per rule, in the
//! precedence 150 D-3 gave them, and a case proving the read is narrower than
//! the corpus when no trigger fires.
//!
//! Every case is over strings: the selector is a pure function of the
//! configuration, the corpus read as text and the changed paths.

use spec_spine_core::{
    AffectedCommits, AffectedRule, AffectedSpec, affected_document, affected_json, select_affected,
};
use spec_spine_types::{Config, Error};

fn spec(id: &str, commands: &[&str]) -> AffectedSpec {
    spec_with(id, commands, "")
}

fn spec_with(id: &str, commands: &[&str], frontmatter: &str) -> AffectedSpec {
    let mut md = format!(
        "---\nid: \"{id}\"\ntitle: \"{id}\"\nstatus: draft\n{frontmatter}created: \"2026-09-27\"\nsummary: \"s\"\n---\n# {id}\n"
    );
    if !commands.is_empty() {
        md.push_str("\n## Verification\n\n```verify:cli\n");
        for c in commands {
            md.push_str(c);
            md.push('\n');
        }
        md.push_str("```\n");
    }
    AffectedSpec {
        id: id.to_string(),
        markdown: Some(md),
    }
}

fn paths(p: &[&str]) -> Vec<String> {
    p.iter().map(|s| s.to_string()).collect()
}

fn config_selecting_all_on(globs: &[&str]) -> Config {
    let mut cfg = Config::default();
    cfg.acceptance.select_all_on = paths(globs);
    cfg
}

fn picked(cfg: &Config, corpus: &[AffectedSpec], changed: &[&str]) -> Vec<(String, AffectedRule)> {
    select_affected(cfg, corpus, &paths(changed))
        .unwrap()
        .selected
        .into_iter()
        .map(|e| (e.id, e.rule))
        .collect()
}

fn of(id: &str, rule: AffectedRule) -> (String, AffectedRule) {
    (id.to_string(), rule)
}

#[test]
fn a_configured_trigger_selects_every_spec_with_select_all() {
    let corpus = [spec("001-a", &["true"]), spec("002-b", &["true"])];
    let cfg = config_selecting_all_on(&["crates/*/src/**/*", "Cargo.lock"]);
    let got = select_affected(
        &cfg,
        &corpus,
        &paths(&["crates/demo/src/deep/mod.rs", "README"]),
    )
    .unwrap();
    assert!(got.select_all);
    assert_eq!(got.select_all_paths, ["crates/demo/src/deep/mod.rs"]);
    assert_eq!(
        got.selected.iter().map(|e| e.rule).collect::<Vec<_>>(),
        [AffectedRule::SelectAll, AffectedRule::SelectAll]
    );
    // The default configuration has no trigger: the same change selects nothing.
    assert!(
        picked(
            &Config::default(),
            &corpus,
            &["crates/demo/src/deep/mod.rs"]
        )
        .is_empty()
    );
    // A glob's `*` stops at `/`, so a crate manifest is not `crates/*/src`.
    assert!(picked(&cfg, &corpus, &["crates/demo/Cargo.toml"]).is_empty());
}

#[test]
fn a_changed_spec_selects_itself() {
    let corpus = [spec("001-a", &["true"]), spec("002-b", &["true"])];
    let got = picked(&Config::default(), &corpus, &["specs/002-b/spec.md"]);
    assert_eq!(got, [of("002-b", AffectedRule::ChangedSpec)]);
}

#[test]
fn a_plan_naming_a_changed_spec_is_selected_by_id_or_ordinal_token() {
    let corpus = [
        spec("001-a", &["true see 002-b"]),
        spec("002-b", &["true"]),
        spec("003-c", &["true verify 002 --plan"]),
        // Neither a longer number nor a word joined to the ordinal names it.
        spec("004-d", &["true 0020 x002"]),
    ];
    let got = picked(&Config::default(), &corpus, &["specs/002-b/spec.md"]);
    assert_eq!(
        got,
        [
            of("001-a", AffectedRule::NamesSpec),
            of("002-b", AffectedRule::ChangedSpec),
            of("003-c", AffectedRule::NamesSpec),
        ]
    );
}

#[test]
fn a_plan_naming_a_changed_path_is_selected() {
    let corpus = [
        spec("001-a", &["test -f tools/helper.sh"]),
        spec("002-b", &["true"]),
    ];
    // A deletion and the old side of a rename are paths like any other.
    let got = picked(&Config::default(), &corpus, &["tools/helper.sh"]);
    assert_eq!(got, [of("001-a", AffectedRule::NamesPath)]);
}

#[test]
fn a_plan_running_a_changed_test_is_selected() {
    let corpus = [
        spec("001-a", &["cargo test -p demo --test widget --locked"]),
        spec("002-b", &["cargo test -p demo --locked"]),
        spec("003-c", &["cargo test -p demo --test other --locked"]),
        spec("004-d", &["cargo test -p elsewhere --test widget"]),
        spec("005-e", &["echo cargo test -p demo --test widget"]),
    ];
    let got = picked(
        &Config::default(),
        &corpus,
        &["crates/demo/tests/widget.rs"],
    );
    assert_eq!(
        got,
        [
            of("001-a", AffectedRule::NamesTest),
            of("002-b", AffectedRule::NamesTest),
            // `echo cargo test` is still a line that runs `cargo test`: the
            // rule is 150 D-3's, a line-level reading, and is kept as written.
            of("005-e", AffectedRule::NamesTest),
        ]
    );
    // A fixture under tests/ reaches every target of its crate.
    let got = picked(
        &Config::default(),
        &corpus,
        &["crates/demo/tests/fixtures/x.json"],
    );
    assert_eq!(
        got.iter().map(|(id, _)| id.as_str()).collect::<Vec<_>>(),
        ["001-a", "002-b", "003-c", "005-e"]
    );
}

#[test]
fn the_first_matching_rule_wins() {
    // 001 names a changed path, a changed spec and is itself changed.
    let corpus = [
        spec("001-a", &["cat specs/002-b/spec.md"]),
        spec("002-b", &["true"]),
    ];
    let cfg = config_selecting_all_on(&["tools/*"]);
    let changed = ["specs/001-a/spec.md", "specs/002-b/spec.md"];
    let got = picked(&Config::default(), &corpus, &changed);
    assert_eq!(
        got,
        [
            of("001-a", AffectedRule::ChangedSpec),
            of("002-b", AffectedRule::ChangedSpec),
        ]
    );
    let got = picked(&cfg, &corpus, &["tools/x", "specs/002-b/spec.md"]);
    assert!(got.iter().all(|(_, rule)| *rule == AffectedRule::SelectAll));
}

#[test]
fn the_read_is_narrower_than_the_corpus_when_no_trigger_fires() {
    let corpus = [
        spec("001-a", &["test -f tools/helper.sh"]),
        spec("002-b", &["true"]),
        spec(
            "003-c",
            &["cargo --version >/dev/null || true", "test -f README"],
        ),
    ];
    let cfg = config_selecting_all_on(&["crates/*/src/**/*"]);
    let got = select_affected(&cfg, &corpus, &paths(&["tools/helper.sh"])).unwrap();
    assert!(!got.select_all && got.select_all_paths.is_empty());
    assert_eq!(got.corpus_size, 3);
    assert_eq!(got.selected.len(), 1);
    assert_eq!(got.selected[0].id, "001-a");
    assert!(got.selected.len() < got.corpus_size);
    // The entry carries the plan `verify <id> --plan` prints.
    assert_eq!(got.selected[0].plan, ["test -f tools/helper.sh"]);
}

#[test]
fn an_empty_change_is_an_empty_selection() {
    let corpus = [spec("001-a", &["true"])];
    let got = select_affected(&Config::default(), &corpus, &[]).unwrap();
    assert!(got.selected.is_empty() && got.changed_paths.is_empty());
    assert_eq!(got.corpus_size, 1);
}

#[test]
fn changed_paths_are_sorted_and_unique() {
    let corpus = [spec("001-a", &["true"])];
    let got = select_affected(&Config::default(), &corpus, &paths(&["b", "a", "b"])).unwrap();
    assert_eq!(got.changed_paths, ["a", "b"]);
}

#[test]
fn an_amended_acceptance_is_planned_from_the_spec_that_holds_it() {
    // 002 amends 001's verification: 001's effective plan is 002's block, so a
    // path 002's block names selects 001, and 001's own block does not count.
    let corpus = [
        spec("001-a", &["test -f old/path"]),
        spec_with(
            "002-b",
            &["test -f new/path"],
            "amends_verification:\n  - \"001-a\"\n",
        ),
    ];
    let got = picked(&Config::default(), &corpus, &["new/path"]);
    assert_eq!(
        got,
        [
            of("001-a", AffectedRule::NamesPath),
            of("002-b", AffectedRule::NamesPath),
        ]
    );
    assert!(picked(&Config::default(), &corpus, &["old/path"]).is_empty());
}

#[test]
fn a_plan_that_cannot_be_read_refuses_and_names_the_spec() {
    let corpus = [
        spec("001-a", &["true"]),
        AffectedSpec {
            id: "002-b".to_string(),
            markdown: None,
        },
    ];
    let err = select_affected(&Config::default(), &corpus, &[]).unwrap_err();
    assert!(
        matches!(err, Error::NotFound(ref m) if m.contains("002-b")),
        "{err:?}"
    );
    assert_eq!(err.exit_code(), 1);
}

#[test]
fn a_select_all_on_entry_that_is_not_a_glob_is_a_config_error() {
    let corpus = [spec("001-a", &["true"])];
    let cfg = config_selecting_all_on(&["crates/[*"]);
    let err = select_affected(&cfg, &corpus, &paths(&["x"])).unwrap_err();
    assert!(matches!(err, Error::Config(_)), "{err:?}");
}

#[test]
fn the_document_is_a_deterministic_read_document() {
    let corpus = [
        spec("001-a", &["test -f tools/helper.sh"]),
        spec("002-b", &["true"]),
    ];
    let got = select_affected(&Config::default(), &corpus, &paths(&["tools/helper.sh"])).unwrap();
    let sha = |c: char| c.to_string().repeat(40);
    let commits = AffectedCommits {
        base: sha('a'),
        merge_base: sha('b'),
        head: sha('c'),
    };
    let doc = affected_document(&got, &commits).unwrap();
    assert_eq!(doc, affected_document(&got, &commits).unwrap());
    let v: serde_json::Value = serde_json::from_str(&doc).unwrap();
    assert_eq!(v["schemaVersion"], spec_spine_types::READ_SCHEMA_VERSION);
    assert_eq!(v["mergeBase"], sha('b'));
    assert_eq!(v["changedPaths"][0], "tools/helper.sh");
    assert_eq!(v["corpusSize"], 2);
    assert_eq!(v["selectAll"], false);
    assert_eq!(v["selected"][0]["rule"], "names-path");
    assert_eq!(v["selected"][0]["plan"][0], "test -f tools/helper.sh");
    assert!(doc.ends_with("}\n"));
}

#[test]
fn the_facade_answers_the_same_document_and_refuses_a_bad_request() {
    let request = serde_json::json!({
        "commits": { "base": "a".repeat(40), "mergeBase": "a".repeat(40), "head": "b".repeat(40) },
        "changed": ["specs/001-a/spec.md"],
        "corpus": [spec("001-a", &["true"]), spec("002-b", &["true"])],
    });
    let doc: serde_json::Value =
        serde_json::from_str(&affected_json(&request.to_string()).unwrap()).unwrap();
    assert_eq!(doc["selected"].as_array().unwrap().len(), 1);
    assert_eq!(doc["selected"][0]["id"], "001-a");
    assert_eq!(doc["selected"][0]["rule"], "changed-spec");

    let err = affected_json("{}").unwrap_err();
    assert!(matches!(err, Error::Usage(_)), "{err:?}");
}
