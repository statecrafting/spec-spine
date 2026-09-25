//! Spec 142: a relocation is proven, not asserted.
//!
//! Claim transfer by a partial `supersedes` (§3.1), the `relocates` grammar
//! (§3.2), `delta`'s proof against the merge base (§3.3), `amends_sections`
//! validation (§3.4) and the two-establishers warning (§3.5). The fixtures are
//! the item-C shape: an approved spec `002-env` owning two files and two body
//! sections, and a new spec `008-y` taking one of each.

use std::fs;
use std::path::{Path, PathBuf};

use spec_spine_core::shard::{self, BY_PACKAGE_DIR, BY_SPEC_DIR};
use spec_spine_core::{
    compile, couple::build_superseders, couple::owners_for_path, delta, index, index_dir,
    index_inputs_file, index_shard_files, lint,
};
use spec_spine_types::{Config, DeltaClass, DeltaCommits, DeltaReport};

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, content).unwrap();
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

fn emit_index(cfg: &Config, root: &Path) {
    let outcome = index(cfg, root).unwrap();
    let dir = index_dir(cfg, root);
    let (by_spec, by_package) = index_shard_files(&outcome.shards).unwrap();
    shard::sync_dir(&dir.join(BY_SPEC_DIR), &by_spec).unwrap();
    shard::sync_dir(&dir.join(BY_PACKAGE_DIR), &by_package).unwrap();
    let (name, content) = index_inputs_file(&outcome.shards).unwrap();
    fs::write(dir.join(name), content).unwrap();
}

const ENV_BODY: &str = "# 002: Env\n\n## 1. X\n\nX holds.\n\n## 2. Y\n\nY holds, and:\n\n### 2.1 Detail\n\nThe detail.\n\n## 3. W\n\nW holds.\n";

fn env_spec(body: &str) -> String {
    format!(
        "---\nid: \"002-env\"\ntitle: \"Env\"\nstatus: approved\ncreated: \"2026-09-25\"\nimplementation: complete\nsummary: \"s\"\nestablishes:\n  - \"src/x.rs\"\n  - \"src/y.rs\"\n---\n{body}"
    )
}

fn y_spec(frontmatter_extra: &str, body: &str) -> String {
    format!(
        "---\nid: \"008-y\"\ntitle: \"Y\"\nstatus: draft\ncreated: \"2026-09-25\"\nimplementation: complete\nsummary: \"s\"\ndepends_on: [\"002-env\"]\n{frontmatter_extra}---\n{body}"
    )
}

/// 008's frontmatter taking y.rs and section 2 from 002.
const TAKE_Y: &str = "supersedes:\n  - { spec: \"002-env\", scope: partial, unit: \"src/y.rs\" }\nrelocates:\n  - { spec: \"002-env\", from: \"2-y\", to: \"1-y\" }\n";

/// The relocated text as it sits in 008: renumbered, one level deeper for the
/// subsection, otherwise the same.
const Y_BODY: &str = "# 008: Y\n\n## 1. Y\n\nY holds, and:\n\n### 1.1 Detail\n\nThe detail.\n";

/// 002 after the split: section 2 gone, section 3 renumbered.
const ENV_AFTER: &str = "# 002: Env\n\n## 1. X\n\nX holds.\n\n## 2. W\n\nW holds.\n";

struct Split {
    _tmp: tempfile::TempDir,
    cfg: Config,
    base: PathBuf,
    head: PathBuf,
}

/// A base with 002 alone, and a head where 008 took y.rs and section 2.
fn split(env_after: &str, y_body: &str, y_extra: &str) -> Split {
    let tmp = tempfile::tempdir().unwrap();
    let cfg = Config::default();
    let base = tmp.path().join("base");
    let head = tmp.path().join("head");
    write(&base, "specs/002-env/spec.md", &env_spec(ENV_BODY));
    write(&base, "src/x.rs", "pub fn x() {}\n");
    write(&base, "src/y.rs", "pub fn y() {}\n");
    emit_index(&cfg, &base);
    copy_tree(&base, &head);
    write(&head, "specs/002-env/spec.md", &env_spec(env_after));
    write(&head, "specs/008-y/spec.md", &y_spec(y_extra, y_body));
    Split {
        _tmp: tmp,
        cfg,
        base,
        head,
    }
}

fn run(s: &Split) -> DeltaReport {
    let changed = vec![
        "specs/002-env/spec.md".to_string(),
        "specs/008-y/spec.md".to_string(),
    ];
    let commits = DeltaCommits {
        base: "b".repeat(40),
        merge_base: "m".repeat(40),
        head: "h".repeat(40),
    };
    delta(&s.cfg, &s.base, &s.head, &changed, &commits).unwrap()
}

fn classes(r: &DeltaReport, path: &str) -> Vec<DeltaClass> {
    r.changes
        .iter()
        .find(|c| c.path == path)
        .map(|c| c.classes.clone())
        .unwrap_or_default()
}

fn owners(cfg: &Config, root: &Path, path: &str) -> Vec<String> {
    let idx = index(cfg, root).unwrap().index;
    let sup = build_superseders(&compile(cfg, root).unwrap().registry);
    owners_for_path(&cfg.layout.specs_dir, path, &[], &idx, &sup)
        .into_iter()
        .collect()
}

/// §3.1: a live partial `supersedes` hands the unit over. Before 142 both specs
/// owned y.rs and an edit to either spec cleared it.
#[test]
fn a_partial_supersedes_hands_the_unit_over() {
    let s = split(ENV_AFTER, Y_BODY, TAKE_Y);
    assert_eq!(
        owners(&s.cfg, &s.head, "src/y.rs"),
        vec!["008-y".to_string()]
    );
    assert_eq!(
        owners(&s.cfg, &s.head, "src/x.rs"),
        vec!["002-env".to_string()]
    );
    // The predecessor's claim stays visible, as non-owning.
    let idx = index(&s.cfg, &s.head).unwrap().index;
    let env = idx
        .traceability
        .mappings
        .iter()
        .find(|m| m.spec_id == "002-env")
        .unwrap();
    assert!(
        env.resolved_units
            .iter()
            .any(|u| !u.ownership && u.locations.iter().any(|l| l.file == "src/y.rs"))
    );
    assert!(!env.implementing_paths.iter().any(|p| p.path == "src/y.rs"));
}

/// §3.1: only a live successor takes the unit; a retired one returns it.
#[test]
fn a_retired_successor_hands_nothing_over() {
    let extra = TAKE_Y.to_string();
    let s = split(ENV_AFTER, Y_BODY, &extra);
    let retired = fs::read_to_string(s.head.join("specs/008-y/spec.md"))
        .unwrap()
        .replace(
            "status: draft",
            "status: retired\nretirement_rationale: \"r\"",
        );
    write(&s.head, "specs/008-y/spec.md", &retired);
    let got = owners(&s.cfg, &s.head, "src/y.rs");
    assert!(got.contains(&"002-env".to_string()), "{got:?}");
}

/// §3.3: a proven relocation classifies the source's body change as
/// `relocation`, never with `requirement`, and names the check.
#[test]
fn a_proven_relocation_is_classed_relocation() {
    let r = run(&split(ENV_AFTER, Y_BODY, TAKE_Y));
    let env = classes(&r, "specs/002-env/spec.md");
    assert!(env.contains(&DeltaClass::Relocation), "{env:?}");
    assert!(!env.contains(&DeltaClass::Requirement), "{env:?}");
    assert_eq!(r.relocations.len(), 1);
    let check = &r.relocations[0];
    assert!(check.proven, "{check:?}");
    assert_eq!(
        (
            check.spec.as_str(),
            check.from_spec.as_str(),
            check.from.as_str(),
            check.to.as_str()
        ),
        ("008-y", "002-env", "2-y", "1-y")
    );
    assert!(r.counts[&DeltaClass::Relocation] == 1);
    assert!(!DeltaClass::Relocation.requires_prior_policy());
}

/// §3.3: text that changed on the way is not a relocation.
#[test]
fn a_changed_text_is_not_proven_and_stays_requirement() {
    let altered = Y_BODY.replace("The detail.", "A different detail.");
    let r = run(&split(ENV_AFTER, &altered, TAKE_Y));
    let env = classes(&r, "specs/002-env/spec.md");
    assert!(env.contains(&DeltaClass::Requirement), "{env:?}");
    assert!(!env.contains(&DeltaClass::Relocation), "{env:?}");
    assert!(!r.relocations[0].proven);
    assert!(
        r.relocations[0]
            .reason
            .as_deref()
            .unwrap()
            .contains("text differs")
    );
}

/// §3.3: a proven relocation does not cover another edit to the source.
#[test]
fn another_edit_to_the_source_stays_requirement() {
    let also_edited = ENV_AFTER.replace("X holds.", "X holds, now differently.");
    let r = run(&split(&also_edited, Y_BODY, TAKE_Y));
    assert!(r.relocations[0].proven);
    let env = classes(&r, "specs/002-env/spec.md");
    assert!(env.contains(&DeltaClass::Requirement), "{env:?}");
    assert!(!env.contains(&DeltaClass::Relocation), "{env:?}");
}

/// §3.3: no declaration, no relocation: removing a section is a requirement
/// change even when the text reappears elsewhere.
#[test]
fn an_undeclared_move_is_a_requirement_change() {
    let no_reloc = "supersedes:\n  - { spec: \"002-env\", scope: partial, unit: \"src/y.rs\" }\n";
    let r = run(&split(ENV_AFTER, Y_BODY, no_reloc));
    assert!(r.relocations.is_empty());
    let env = classes(&r, "specs/002-env/spec.md");
    assert!(env.contains(&DeltaClass::Requirement), "{env:?}");
}

fn corpus(files: &[(&str, &str)]) -> (tempfile::TempDir, Config) {
    let tmp = tempfile::tempdir().unwrap();
    for (rel, content) in files {
        write(tmp.path(), rel, content);
    }
    (tmp, Config::default())
}

fn codes(v: &[spec_spine_types::Violation]) -> Vec<String> {
    v.iter().map(|x| x.code.clone()).collect()
}

/// §3.2: the receiving section must exist once, and the source must be another
/// spec in the corpus; the short id normalizes.
#[test]
fn the_relocates_grammar_is_checked() {
    let (tmp, cfg) = corpus(&[
        ("specs/002-env/spec.md", &env_spec(ENV_BODY)),
        ("src/x.rs", ""),
        ("src/y.rs", ""),
        (
            "specs/008-y/spec.md",
            &y_spec(
                "relocates:\n  - { spec: \"002\", from: \"2-y\", to: \"1-y\" }\n",
                Y_BODY,
            ),
        ),
    ]);
    let out = compile(&cfg, tmp.path()).unwrap();
    assert!(
        !codes(&out.registry.validation.violations)
            .iter()
            .any(|c| c.starts_with("V-04"))
    );
    let rec = out.registry.specs.iter().find(|s| s.id == "008-y").unwrap();
    assert_eq!(rec.relocates[0].spec, "002-env", "the short id resolves");

    for (extra, want) in [
        (
            "relocates:\n  - { spec: \"002-env\", from: \"2-y\", to: \"9-nope\" }\n",
            "V-042",
        ),
        (
            "relocates:\n  - { spec: \"002-env\", from: \"\" , to: \"1-y\" }\n",
            "V-042",
        ),
        (
            "relocates:\n  - { spec: \"099-gone\", from: \"2-y\", to: \"1-y\" }\n",
            "V-043",
        ),
        (
            "relocates:\n  - { spec: \"008-y\", from: \"2-y\", to: \"1-y\" }\n",
            "V-043",
        ),
    ] {
        write(tmp.path(), "specs/008-y/spec.md", &y_spec(extra, Y_BODY));
        let got = codes(
            &compile(&cfg, tmp.path())
                .unwrap()
                .registry
                .validation
                .violations,
        );
        assert!(got.contains(&want.to_string()), "{extra}: {got:?}");
    }
}

/// §3.4: an `amends_sections` entry must be a section number or anchor of an
/// amended spec.
#[test]
fn amends_sections_must_name_a_section() {
    let amender = |sections: &str| {
        format!(
            "---\nid: \"009-am\"\ntitle: \"Am\"\nstatus: draft\ncreated: \"2026-09-25\"\nsummary: \"s\"\namends: [\"002-env\"]\namends_sections: {sections}\n---\n# 009\n"
        )
    };
    for (sections, expect) in [
        ("[\"2\", \"2.1\", \"3-w\"]", false),
        ("[\"2\", \"7.4\"]", true),
    ] {
        let (tmp, cfg) = corpus(&[
            ("specs/002-env/spec.md", &env_spec(ENV_BODY)),
            ("src/x.rs", ""),
            ("src/y.rs", ""),
            ("specs/009-am/spec.md", &amender(sections)),
        ]);
        emit_index(&cfg, tmp.path());
        let got = codes(&lint(&cfg, tmp.path()).unwrap().violations);
        assert_eq!(
            got.contains(&"L-017".to_string()),
            expect,
            "{sections}: {got:?}"
        );
    }
}

/// §3.5: two live establishers of one unit warn, unless a partial `supersedes`
/// hands it over.
#[test]
fn two_live_establishers_warn_until_one_hands_over() {
    let taker = |extra: &str| {
        format!(
            "---\nid: \"008-y\"\ntitle: \"Y\"\nstatus: draft\ncreated: \"2026-09-25\"\nsummary: \"s\"\nestablishes:\n  - \"src/y.rs\"\n{extra}---\n# 008\n"
        )
    };
    for (extra, expect) in [
        ("", true),
        (
            "supersedes:\n  - { spec: \"002-env\", scope: partial, unit: \"src/y.rs\" }\n",
            false,
        ),
    ] {
        let (tmp, cfg) = corpus(&[
            ("specs/002-env/spec.md", &env_spec(ENV_BODY)),
            ("src/x.rs", ""),
            ("src/y.rs", ""),
            ("specs/008-y/spec.md", &taker(extra)),
        ]);
        emit_index(&cfg, tmp.path());
        let got = codes(&lint(&cfg, tmp.path()).unwrap().violations);
        assert_eq!(
            got.contains(&"L-018".to_string()),
            expect,
            "{extra}: {got:?}"
        );
    }
}
