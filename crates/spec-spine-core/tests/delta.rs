//! Spec 071: a change classified under the base's rules.
//!
//! The classification matrix of §3.8, library side. Every case builds a base
//! tree with its committed index written exactly as `spec-spine index` writes
//! it, copies it to a head tree, applies one change, and asks [`delta`] what the
//! change is. No git: the core reads two roots, and the CLI's tests exercise the
//! git half.
//!
//! The head tree is deliberately **not** re-indexed after its change. The head's
//! committed index is the candidate's to write, so the report must not read it
//! (D-5): a case that passed only because the head's shards had been refreshed
//! would say nothing about that.

use std::fs;
use std::path::{Path, PathBuf};

use spec_spine_core::shard::{self, BY_PACKAGE_DIR, BY_SPEC_DIR};
use spec_spine_core::{
    Freshness, check_index_freshness, delta, delta_json, index, index_dir, index_shard_files,
};
use spec_spine_types::{
    ChangeKind, Config, DELTA_SCHEMA_VERSION, DeltaChange, DeltaClass, DeltaCommits, DeltaReport,
    Error,
};

use DeltaClass::*;

const SPEC_A: &str = "---\nid: \"001-a\"\ntitle: \"a\"\nstatus: approved\ncreated: \"2026-09-11\"\nimplementation: complete\nsummary: \"s\"\nestablishes:\n  - \"src/a.rs\"\n---\n\n# a\n\nThe body.\n\n## Verification\n\n```verify:cli\ntest -f src/a.rs\ngrep -q \"fn a\" src/a.rs\n```\n";

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, content).unwrap();
}

fn read(root: &Path, rel: &str) -> String {
    fs::read_to_string(root.join(rel)).unwrap()
}

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let dest = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &dest);
        } else {
            fs::copy(entry.path(), dest).unwrap();
        }
    }
}

/// Write the committed index for `root`, as `spec-spine index` does.
fn emit_index(cfg: &Config, root: &Path) {
    let outcome = index(cfg, root).unwrap();
    let dir = index_dir(cfg, root);
    let (by_spec, by_package) = index_shard_files(&outcome.shards).unwrap();
    shard::sync_dir(&dir.join(BY_SPEC_DIR), &by_spec).unwrap();
    shard::sync_dir(&dir.join(BY_PACKAGE_DIR), &by_package).unwrap();
}

/// A base tree (`001-a` owns `src/a.rs` and declares two acceptance commands)
/// and a head tree identical to it, ready to be changed.
struct Fixture {
    _tmp: tempfile::TempDir,
    cfg: Config,
    base: PathBuf,
    head: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        Self::with(|_| {})
    }

    /// `setup` runs on the base before its index is written.
    fn with(setup: impl FnOnce(&Path)) -> Self {
        Self::with_config(Config::default(), setup)
    }

    /// `cfg` is the base's configuration, and the one the report is run under.
    fn with_config(cfg: Config, setup: impl FnOnce(&Path)) -> Self {
        let tmp = tempfile::tempdir().unwrap();
        let base = tmp.path().join("base");
        let head = tmp.path().join("head");
        write(&base, "specs/001-a/spec.md", SPEC_A);
        write(&base, "src/a.rs", "pub fn a() {}\n");
        setup(&base);
        emit_index(&cfg, &base);
        copy_tree(&base, &head);
        Fixture {
            _tmp: tmp,
            cfg,
            base,
            head,
        }
    }

    fn run(&self, changed: &[&str]) -> DeltaReport {
        self.try_run(changed).unwrap()
    }

    fn try_run(&self, changed: &[&str]) -> Result<DeltaReport, Error> {
        let changed: Vec<String> = changed.iter().map(|s| s.to_string()).collect();
        delta(&self.cfg, &self.base, &self.head, &changed, &commits())
    }
}

fn commits() -> DeltaCommits {
    DeltaCommits {
        base: "b".repeat(40),
        merge_base: "m".repeat(40),
        head: "h".repeat(40),
    }
}

fn change<'r>(report: &'r DeltaReport, path: &str) -> &'r DeltaChange {
    report
        .changes
        .iter()
        .find(|c| c.path == path)
        .unwrap_or_else(|| panic!("no change for {path}: {report:#?}"))
}

/// Row 1: a code edit with its owning spec untouched.
#[test]
fn a_code_edit_is_implementation_and_needs_no_prior_policy() {
    let f = Fixture::new();
    write(&f.head, "src/a.rs", "pub fn a() { changed(); }\n");
    let r = f.run(&["src/a.rs"]);

    let c = change(&r, "src/a.rs");
    assert_eq!(c.classes, vec![Implementation]);
    assert_eq!(c.change, ChangeKind::Modified);
    assert!(c.authority.is_none() && c.verification.is_none() && c.lifecycle.is_none());
    assert!(!r.prior_policy.required);
    assert!(r.prior_policy.classes.is_empty());
    assert_eq!(r.counts[&Implementation], 1);
}

/// Row 2: a body edit outside `## Verification`.
#[test]
fn a_body_edit_is_requirement() {
    let f = Fixture::new();
    let spec = read(&f.head, "specs/001-a/spec.md").replace("The body.", "The body, reworded.");
    write(&f.head, "specs/001-a/spec.md", &spec);
    let r = f.run(&["specs/001-a/spec.md"]);

    assert_eq!(change(&r, "specs/001-a/spec.md").classes, vec![Requirement]);
    assert!(r.prior_policy.required);
    assert_eq!(r.prior_policy.classes, vec![Requirement]);
}

/// Row 3: the first row of §1. A command removed from the block that judges the
/// change, and the code it checked rewritten in the same diff.
#[test]
fn removing_a_verification_command_is_verification() {
    let f = Fixture::new();
    let spec = read(&f.head, "specs/001-a/spec.md").replace("grep -q \"fn a\" src/a.rs\n", "");
    write(&f.head, "specs/001-a/spec.md", &spec);
    write(&f.head, "src/a.rs", "pub fn b() {}\n");
    let r = f.run(&["specs/001-a/spec.md", "src/a.rs"]);

    let c = change(&r, "specs/001-a/spec.md");
    assert_eq!(
        c.classes,
        vec![Verification],
        "the body outside the section did not change"
    );
    let v = c.verification.as_ref().unwrap();
    let (base_hash, head_hash) = (
        v.base_plan_hash.as_deref().unwrap(),
        v.head_plan_hash.as_deref().unwrap(),
    );
    assert_ne!(base_hash, head_hash);
    assert_eq!(base_hash.len(), 64);
    assert_eq!((v.commands_only_in_base, v.commands_only_in_head), (1, 0));

    assert_eq!(change(&r, "src/a.rs").classes, vec![Implementation]);
    assert_eq!(r.prior_policy.classes, vec![Verification]);
}

/// Row 4: the second row of §1. A new draft spec with an `extends` edge on
/// another spec's unit, and that unit rewritten.
#[test]
fn a_new_extends_claim_is_authority_on_the_spec_and_the_unit() {
    let f = Fixture::new();
    write(
        &f.head,
        "specs/002-b/spec.md",
        "---\nid: \"002-b\"\ntitle: \"b\"\nstatus: draft\ncreated: \"2026-09-11\"\nimplementation: pending\nsummary: \"s\"\nextends:\n  - { spec: \"001-a\", unit: \"src/a.rs\", nature: additive }\n---\n\n# b\n",
    );
    write(&f.head, "src/a.rs", "pub fn a() { rewritten(); }\n");
    let r = f.run(&["specs/002-b/spec.md", "src/a.rs"]);

    let unit = change(&r, "src/a.rs");
    assert_eq!(unit.classes, vec![Authority, Implementation]);
    let a = unit.authority.as_ref().unwrap();
    assert_eq!(a.base_owners, vec!["001-a"]);
    assert_eq!(
        a.head_owners,
        vec!["001-a", "002-b"],
        "the head's ownership is resolved from its tree, not its stale committed index"
    );

    let spec = change(&r, "specs/002-b/spec.md");
    assert_eq!(spec.change, ChangeKind::Added);
    assert_eq!(
        spec.classes,
        vec![Authority, Lifecycle, Requirement, Verification]
    );
    let edges = &spec.authority.as_ref().unwrap().edges_added["extends"];
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0]["spec"], "001-a");
    assert_eq!(edges[0]["unit"]["path"], "src/a.rs");
    let v = spec.verification.as_ref().unwrap();
    assert!(
        v.base_plan_hash.is_none(),
        "absent where the spec does not exist"
    );
    assert!(v.head_plan_hash.is_some());

    assert!(r.prior_policy.required);
    assert_eq!(
        r.prior_policy.classes,
        vec![Authority, Lifecycle, Requirement, Verification]
    );
}

/// Row 5: a status flip, and nothing else.
#[test]
fn a_status_flip_is_lifecycle_with_both_values() {
    let f = Fixture::new();
    let spec = read(&f.head, "specs/001-a/spec.md").replace("status: approved", "status: retired");
    write(&f.head, "specs/001-a/spec.md", &spec);
    let r = f.run(&["specs/001-a/spec.md"]);

    let c = change(&r, "specs/001-a/spec.md");
    assert_eq!(c.classes, vec![Lifecycle]);
    let status = &c.lifecycle.as_ref().unwrap()["status"];
    assert_eq!(status.base, "approved");
    assert_eq!(status.head, "retired");
    assert_eq!(r.prior_policy.classes, vec![Lifecycle]);
}

/// Row 6: a configuration edit that widens the bypass floor to cover its own
/// diff. It is policy, and the rest of the diff is classified as if it had not
/// been made (D-1).
#[test]
fn a_config_edit_cannot_reclassify_its_own_diff() {
    let f = Fixture::new();
    write(
        &f.head,
        "spec-spine.toml",
        "[coupling]\nbypass_prefixes = [\"src/\"]\n",
    );
    write(&f.head, "src/a.rs", "pub fn a() { unreviewed(); }\n");
    let r = f.run(&["spec-spine.toml", "src/a.rs"]);

    let config = change(&r, "spec-spine.toml");
    assert_eq!(config.classes, vec![Policy]);
    assert_eq!(config.change, ChangeKind::Added);
    assert_eq!(
        change(&r, "src/a.rs").classes,
        vec![Implementation],
        "not bypassed: the base declares no such prefix"
    );
    assert_eq!(r.prior_policy.classes, vec![Policy]);
}

/// Row 7: an edit under `standards/`.
#[test]
fn a_standards_edit_is_constitutional() {
    let f = Fixture::with(|base| {
        write(base, "standards/spec/constitution.md", "# c\n\n## I. One\n");
    });
    write(
        &f.head,
        "standards/spec/constitution.md",
        "# c\n\n## I. One, amended\n",
    );
    let r = f.run(&["standards/spec/constitution.md"]);

    // `standards/**/*` is in the default `extra_hashed_inputs`, so the path is
    // policy as well: every class that applies.
    assert_eq!(
        change(&r, "standards/spec/constitution.md").classes,
        vec![Constitutional, Policy]
    );
    assert_eq!(r.prior_policy.classes, vec![Constitutional, Policy]);
}

/// Row 8: a `git mv` of an owned file. A removal and an addition, never one
/// file moved (§3.3).
#[test]
fn a_move_is_a_removal_and_an_addition() {
    let f = Fixture::new();
    fs::rename(f.head.join("src/a.rs"), f.head.join("src/b.rs")).unwrap();
    let r = f.run(&["src/a.rs", "src/b.rs"]);

    let old = change(&r, "src/a.rs");
    assert_eq!(old.change, ChangeKind::Deleted);
    assert_eq!(old.classes, vec![Authority, Implementation]);
    let a = old.authority.as_ref().unwrap();
    assert_eq!(a.base_owners, vec!["001-a"]);
    assert!(
        a.head_owners.is_empty(),
        "the claim no longer resolves at head"
    );

    let new = change(&r, "src/b.rs");
    assert_eq!(new.change, ChangeKind::Added);
    assert_eq!(new.classes, vec![Unowned], "nothing infers the move");
    assert_eq!(r.prior_policy.classes, vec![Authority]);
}

/// Row 9: a `spec.md` that fails to parse at head.
#[test]
fn a_spec_that_fails_to_parse_is_unknown_never_implementation() {
    let f = Fixture::new();
    let spec =
        read(&f.head, "specs/001-a/spec.md").replace("status: approved", "status: [unclosed");
    write(&f.head, "specs/001-a/spec.md", &spec);
    write(&f.head, "src/a.rs", "pub fn a() { unowned_now(); }\n");
    let r = f.run(&["specs/001-a/spec.md", "src/a.rs"]);

    let c = change(&r, "specs/001-a/spec.md");
    assert!(c.classes.contains(&Unknown), "{:?}", c.classes);
    assert!(!c.classes.contains(&Implementation));
    // The unparseable spec no longer owns anything at head, which is itself an
    // authority change on the path it claimed.
    let unit = change(&r, "src/a.rs");
    assert_eq!(unit.classes, vec![Authority, Implementation]);
    assert!(r.prior_policy.required);
    assert!(r.prior_policy.classes.contains(&Unknown));
}

/// Row 10: an unchanged tree.
#[test]
fn an_unchanged_tree_reports_no_changes() {
    let f = Fixture::new();
    let r = f.run(&[]);
    assert!(r.changes.is_empty());
    assert!(!r.prior_policy.required);
    assert_eq!(
        r.counts.len(),
        DeltaClass::ALL.len(),
        "every class is counted"
    );
    assert!(r.counts.values().all(|n| *n == 0));
    assert_eq!(r.schema_version, DELTA_SCHEMA_VERSION);
    assert_eq!(r.classified_under, "base");
    assert_eq!(r.merge_base, "m".repeat(40), "commit ids are echoed");
}

/// §3.2: the base's committed index is compared before it is used.
#[test]
fn a_stale_base_index_is_refused_not_classified_under() {
    let f = Fixture::new();
    // The base's committed index no longer matches its tree.
    let spec = read(&f.base, "specs/001-a/spec.md").replace("\"src/a.rs\"", "\"src/other.rs\"");
    write(&f.base, "specs/001-a/spec.md", &spec);
    match f.try_run(&["src/a.rs"]) {
        Err(Error::Stale { .. }) => {}
        other => panic!("expected Error::Stale, got {other:?}"),
    }
}

/// D-4: a frontmatter key this tool does not model is a requirement change.
#[test]
fn an_unmodeled_frontmatter_key_is_requirement() {
    let f = Fixture::new();
    let spec = read(&f.head, "specs/001-a/spec.md")
        .replace("summary: \"s\"", "summary: \"s\"\nreviewer: \"someone\"");
    write(&f.head, "specs/001-a/spec.md", &spec);
    let r = f.run(&["specs/001-a/spec.md"]);
    assert_eq!(change(&r, "specs/001-a/spec.md").classes, vec![Requirement]);
}

/// Derived and bypassed paths, and a configuration that no longer parses.
#[test]
fn derived_bypassed_and_unparseable_config() {
    let f = Fixture::with(|base| write(base, "docs/guide.md", "# guide\n"));
    write(&f.head, "docs/guide.md", "# guide, edited\n");
    write(&f.head, ".derived/notes.json", "{}\n");
    write(&f.head, "spec-spine.toml", "[coupling\n");
    let r = f.run(&["docs/guide.md", ".derived/notes.json", "spec-spine.toml"]);

    assert_eq!(change(&r, "docs/guide.md").classes, vec![Bypassed]);
    assert_eq!(change(&r, ".derived/notes.json").classes, vec![Derived]);
    assert_eq!(
        change(&r, "spec-spine.toml").classes,
        vec![Policy, Unknown],
        "a configuration that fails to parse on either side is unknown"
    );
}

/// A spec declaring `unamendable` anchors is constitutional whatever else moved.
#[test]
fn a_spec_declaring_unamendable_anchors_is_constitutional() {
    let f = Fixture::new();
    let spec = read(&f.head, "specs/001-a/spec.md")
        .replace("summary: \"s\"", "summary: \"s\"\nunamendable:\n  - \"a\"");
    write(&f.head, "specs/001-a/spec.md", &spec);
    let r = f.run(&["specs/001-a/spec.md"]);
    assert_eq!(
        change(&r, "specs/001-a/spec.md").classes,
        vec![Constitutional]
    );
}

/// §3.3's last row: a change inside `## Verification` that moves no command is
/// placed by no rule, so it is `unknown` rather than nothing (D-10).
#[test]
fn prose_inside_the_verification_section_is_unknown() {
    let f = Fixture::new();
    let spec = read(&f.head, "specs/001-a/spec.md").replace(
        "## Verification\n\n",
        "## Verification\n\nRun from the root.\n\n",
    );
    write(&f.head, "specs/001-a/spec.md", &spec);
    let r = f.run(&["specs/001-a/spec.md"]);
    assert_eq!(change(&r, "specs/001-a/spec.md").classes, vec![Unknown]);
    assert!(r.prior_policy.required);
}

/// D-11: `policy` is what the hash folds, found with the hash's own matcher. A
/// pattern ending in a bare `**` walks to directories and folds no file (spec
/// 058), so a file under it is not policy, while `dir/**/*` folds every file,
/// including one added or deleted at head. The last block holds the report to
/// the ledger: an edit the report calls policy restales the index, and one it
/// does not leaves it fresh.
#[test]
fn policy_is_what_the_hash_folds_not_what_a_pattern_resembles() {
    let mut cfg = Config::default();
    cfg.index.extra_hashed_inputs = vec!["kit/**".into(), "gov/**/*".into()];
    let f = Fixture::with_config(cfg.clone(), |base| {
        write(base, "kit/a.md", "a\n");
        write(base, "gov/a.md", "a\n");
        write(base, "gov/old.md", "old\n");
    });
    write(&f.head, "kit/a.md", "edited\n");
    write(&f.head, "gov/a.md", "edited\n");
    write(&f.head, "gov/new.md", "new\n");
    fs::remove_file(f.head.join("gov/old.md")).unwrap();
    let r = f.run(&["gov/a.md", "gov/new.md", "gov/old.md", "kit/a.md"]);

    assert_eq!(change(&r, "gov/a.md").classes, vec![Policy]);
    assert_eq!(
        change(&r, "gov/new.md").classes,
        vec![Policy],
        "found at head"
    );
    assert_eq!(
        change(&r, "gov/old.md").classes,
        vec![Policy],
        "found at base"
    );
    assert_eq!(
        change(&r, "kit/a.md").classes,
        vec![Unowned],
        "a bare trailing `**` folds no file"
    );

    // The same verdicts, asked of the ledger itself.
    let probe = |rel: &str| {
        let tmp = tempfile::tempdir().unwrap();
        copy_tree(&f.base, tmp.path());
        write(tmp.path(), rel, "probe\n");
        check_index_freshness(&cfg, tmp.path()).unwrap()
    };
    assert!(matches!(probe("gov/a.md"), Freshness::Stale { .. }));
    assert_eq!(probe("kit/a.md"), Freshness::Fresh);
}

/// §3.6: the facade returns the same report, reads the base's configuration
/// when none is supplied, and refuses a path that escapes the roots.
#[test]
fn the_facade_matches_the_typed_call() {
    // `tools/x.sh` is claimed by nobody, so only a bypass prefix decides it, and
    // the head declares one. A facade that read the head's configuration would
    // report it `bypassed`; the base's rules say `unowned`.
    let f = Fixture::with(|base| write(base, "tools/x.sh", "echo x\n"));
    write(&f.head, "src/a.rs", "pub fn a() { changed(); }\n");
    write(&f.head, "tools/x.sh", "echo unreviewed\n");
    write(
        &f.head,
        "spec-spine.toml",
        "[coupling]\nbypass_prefixes = [\"tools/\"]\n",
    );
    let request = serde_json::json!({
        "baseRoot": f.base,
        "headRoot": f.head,
        "changed": ["spec-spine.toml", "src/a.rs", "tools/x.sh"],
        "commits": { "base": "b".repeat(40), "mergeBase": "m".repeat(40), "head": "h".repeat(40) },
    });
    let via_facade: serde_json::Value =
        serde_json::from_str(&delta_json(&request.to_string()).unwrap()).unwrap();
    let typed = f.run(&["spec-spine.toml", "src/a.rs", "tools/x.sh"]);
    assert_eq!(change(&typed, "tools/x.sh").classes, vec![Unowned]);
    assert_eq!(via_facade, serde_json::to_value(&typed).unwrap());

    let mut escaping = request.clone();
    escaping["changed"] = serde_json::json!(["../outside"]);
    assert!(matches!(
        delta_json(&escaping.to_string()),
        Err(Error::Parse(_))
    ));
}
