//! Spec 112: typed overlays ride the existing seam.
//!
//! Several independent overlays in one corpus, each a declared extra
//! frontmatter key (spec 012). What is asserted is what the engine promises:
//! each value reaches its own spec's record as canonical JSON, a nested value
//! needs a declaration to be admitted, an overlay edit stales exactly the
//! declaring spec's shards, and no verdict reads an overlay. What is not
//! asserted is anything about an overlay's meaning, which is its consumer's.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde_json::json;
use spec_spine_core::shard::{self, BY_PACKAGE_DIR, BY_SPEC_DIR};
use spec_spine_core::{
    DiffFile, DiffInput, check_report, compile, couple, coverage, index, index_dir,
    index_shard_files, lint, registry_dir, registry_shard_files,
};
use spec_spine_types::{Config, load_config};

const OVERLAY_KEYS: &str =
    "extra_known_keys = [\"effects\", \"budget\", \"rationale\", \"adapter\"]";

/// Four overlays on two specs. `001-a` carries three, keyed per unit inside
/// their own values; `002-b` carries one, nested several levels deep. Mapping
/// keys are authored out of canonical order on purpose.
const OVERLAYS_A: &str = "\
effects:
  \"crate-a/src/a.rs\":
    writes: []
    reads: [\"the committed index\"]
    network: false
budget:
  ms: 5
  bytes: 1024
rationale:
  - dep: \"serde\"
    why: \"canonical JSON\"
  - dep: \"glob\"
    why: \"hashed inputs\"
";
const OVERLAYS_B: &str = "\
adapter:
  corpus: \"other\"
  map:
    \"crate-a/src/b.rs\":
      to: { spec: \"010-x\", unit: [\"lib.rs\", { anchor: \"a-b\", depth: 3 }] }
      lossy: null
";

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, content).unwrap();
}

fn spec(id: &str, unit: &str, extra: &str) -> String {
    format!(
        "---\nid: \"{id}\"\ntitle: \"T\"\nstatus: approved\ncreated: \"2026-09-23\"\n\
         summary: \"s\"\nestablishes:\n  - \"{unit}\"\n{extra}---\n# {id}\n\n## 1. Purpose\n\nText.\n"
    )
}

/// A two-spec workspace. `declared` puts the overlay keys in the config;
/// `present` writes the overlay values into the two specs.
fn corpus(root: &Path, declared: bool, present: bool) -> Config {
    write(root, "Cargo.toml", "[workspace]\nmembers = [\"crate-a\"]\n");
    write(
        root,
        "crate-a/Cargo.toml",
        "[package]\nname = \"crate-a\"\nversion = \"0.1.0\"\n",
    );
    write(root, "crate-a/src/a.rs", "pub fn a() {}\n");
    write(root, "crate-a/src/b.rs", "pub fn b() {}\n");
    let (a, b) = if present {
        (OVERLAYS_A, OVERLAYS_B)
    } else {
        ("", "")
    };
    write(
        root,
        "specs/001-a/spec.md",
        &spec("001-a", "crate-a/src/a.rs", a),
    );
    write(
        root,
        "specs/002-b/spec.md",
        &spec("002-b", "crate-a/src/b.rs", b),
    );
    let toml = format!(
        "[coupling]\nrequire_ownership = true\n\n[frontmatter]\n{}\n",
        if declared { OVERLAY_KEYS } else { "" }
    );
    write(root, "spec-spine.toml", &toml);
    load_config(&toml).unwrap()
}

/// Write both committed shard trees, as `compile` and `index` do.
fn regenerate(cfg: &Config, root: &Path) {
    let outcome = compile(cfg, root).unwrap();
    assert!(
        outcome.validation_passed,
        "{:?}",
        outcome.registry.validation.violations
    );
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

/// Every committed shard file under the derived directory, by relative path.
fn shard_tree(cfg: &Config, root: &Path) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for (tree, dir) in [
        ("registry", registry_dir(cfg, root)),
        ("index", index_dir(cfg, root)),
    ] {
        for sub in [BY_SPEC_DIR, BY_PACKAGE_DIR] {
            let Ok(entries) = fs::read_dir(dir.join(sub)) else {
                continue;
            };
            for e in entries {
                let e = e.unwrap();
                let name = e.file_name().to_string_lossy().into_owned();
                if name.ends_with(".json") {
                    out.insert(
                        format!("{tree}/{sub}/{name}"),
                        fs::read_to_string(e.path()).unwrap(),
                    );
                }
            }
        }
    }
    out
}

fn extra(cfg: &Config, root: &Path, id: &str) -> BTreeMap<String, serde_json::Value> {
    let outcome = compile(cfg, root).unwrap();
    outcome
        .registry
        .specs
        .into_iter()
        .find(|s| s.id == id)
        .unwrap()
        .extra_frontmatter
}

#[test]
fn each_declared_overlay_reaches_its_own_record_independently() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let cfg = corpus(root, true, true);

    let a = extra(&cfg, root, "001-a");
    assert_eq!(
        a.keys().map(String::as_str).collect::<Vec<_>>(),
        ["budget", "effects", "rationale"],
        "three overlays on 001-a, none merged, renamed or lost"
    );
    assert_eq!(
        a["effects"],
        json!({"crate-a/src/a.rs": {"network": false, "reads": ["the committed index"], "writes": []}})
    );
    assert_eq!(a["budget"], json!({"bytes": 1024, "ms": 5}));
    assert_eq!(
        a["rationale"],
        json!([{"dep": "serde", "why": "canonical JSON"}, {"dep": "glob", "why": "hashed inputs"}]),
        "a sequence keeps its order"
    );

    let b = extra(&cfg, root, "002-b");
    assert_eq!(
        b.keys().map(String::as_str).collect::<Vec<_>>(),
        ["adapter"],
        "an overlay on one spec does not leak onto another"
    );
    assert_eq!(
        b["adapter"],
        json!({"corpus": "other", "map": {"crate-a/src/b.rs": {
            "lossy": null,
            "to": {"spec": "010-x", "unit": ["lib.rs", {"anchor": "a-b", "depth": 3}]}
        }}}),
        "a deeply nested value round-trips with every scalar type intact"
    );
}

#[test]
fn values_are_canonical_not_the_authored_bytes() {
    // §1.2, §3.2: the engine parses and canonicalizes. Two authorings of the
    // same values, one reordered and reformatted, emit the same registry
    // shards apart from the hashes of their differing `spec.md` bytes; the
    // emitted object keys are sorted, not in authoring order.
    let one = tempfile::tempdir().unwrap();
    let cfg = corpus(one.path(), true, true);
    regenerate(&cfg, one.path());

    let two = tempfile::tempdir().unwrap();
    let cfg2 = corpus(two.path(), true, true);
    let reordered = "\
rationale: [ { why: \"canonical JSON\", dep: \"serde\" }, { why: \"hashed inputs\", dep: \"glob\" } ]
budget: { bytes: 1024, ms: 5 }
effects: { \"crate-a/src/a.rs\": { network: false, writes: [], reads: [ \"the committed index\" ] } }
";
    let path = "specs/001-a/spec.md";
    write(
        two.path(),
        path,
        &spec("001-a", "crate-a/src/a.rs", reordered),
    );
    regenerate(&cfg2, two.path());

    let reg = |root: &Path, cfg: &Config| {
        shard_tree(cfg, root)
            .into_iter()
            .filter(|(k, _)| k.starts_with("registry/"))
            .map(|(k, v)| {
                // The spec.md bytes differ, so the content hash does; the
                // transported values are the claim under test.
                let mut doc: serde_json::Value = serde_json::from_str(&v).unwrap();
                strip_hashes(&mut doc);
                (k, doc)
            })
            .collect::<BTreeMap<_, _>>()
    };
    assert_eq!(reg(one.path(), &cfg), reg(two.path(), &cfg2));

    let shard = shard_tree(&cfg, one.path())
        .into_iter()
        .find(|(k, _)| k.ends_with("001-a.json") && k.starts_with("registry/"))
        .unwrap()
        .1;
    let budget_at = shard.find("\"budget\"").unwrap();
    let effects_at = shard.find("\"effects\"").unwrap();
    let rationale_at = shard.find("\"rationale\"").unwrap();
    assert!(budget_at < effects_at && effects_at < rationale_at);
    let bytes_at = shard.find("\"bytes\"").unwrap();
    let ms_at = shard.find("\"ms\"").unwrap();
    assert!(
        bytes_at < ms_at,
        "authored `ms` before `bytes`; emitted sorted"
    );
}

fn strip_hashes(v: &mut serde_json::Value) {
    match v {
        serde_json::Value::Object(m) => {
            m.retain(|k, _| !k.to_ascii_lowercase().contains("hash"));
            m.values_mut().for_each(strip_hashes);
        }
        serde_json::Value::Array(a) => a.iter_mut().for_each(strip_hashes),
        _ => {}
    }
}

#[test]
fn declaring_is_what_admits_a_nested_overlay() {
    // §3.4: undeclared, a nested value is refused as malformed frontmatter
    // (V-002) naming the spec; declared, the same bytes compile.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let cfg = corpus(root, false, true);
    let outcome = compile(&cfg, root).unwrap();
    assert!(!outcome.validation_passed);
    let v002: Vec<_> = outcome
        .registry
        .validation
        .violations
        .iter()
        .filter(|v| v.code == "V-002")
        .collect();
    assert_eq!(v002.len(), 2, "both specs carry a nested overlay: {v002:?}");
    assert!(v002.iter().all(|v| v.message.contains("nested maps")));

    let cfg = corpus(root, true, true);
    assert!(compile(&cfg, root).unwrap().validation_passed);

    // An undeclared scalar key is spec 012's other half, unchanged: accepted
    // silently, with no warning of any code.
    let cfg = corpus(root, false, false);
    write(
        root,
        "specs/001-a/spec.md",
        &spec("001-a", "crate-a/src/a.rs", "budget: 5\n"),
    );
    let outcome = compile(&cfg, root).unwrap();
    assert!(outcome.validation_passed);
    assert!(outcome.registry.validation.violations.is_empty());
    assert_eq!(extra(&cfg, root, "001-a")["budget"], json!(5));
}

#[test]
fn an_overlay_edit_stales_exactly_the_declaring_specs_shards() {
    // §3.3, measured rather than assumed.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let cfg = corpus(root, true, true);
    regenerate(&cfg, root);
    let before = shard_tree(&cfg, root);
    let fresh = check_report(&cfg, root).unwrap();
    assert!(fresh.registry.fresh && fresh.index.fresh);

    // One value in one overlay of 001-a.
    let edited = OVERLAYS_A.replace("ms: 5", "ms: 6");
    write(
        root,
        "specs/001-a/spec.md",
        &spec("001-a", "crate-a/src/a.rs", &edited),
    );
    let stale = check_report(&cfg, root).unwrap();
    assert!(
        !stale.registry.fresh && !stale.index.fresh,
        "an overlay edit is an edit to spec.md, so both trees are stale until regenerated"
    );
    assert!(stale.registry.validation_passed);

    regenerate(&cfg, root);
    let after = shard_tree(&cfg, root);
    assert_eq!(
        before.keys().collect::<Vec<_>>(),
        after.keys().collect::<Vec<_>>()
    );
    let changed: Vec<&str> = before
        .iter()
        .filter(|(k, v)| after[*k] != **v)
        .map(|(k, _)| k.as_str())
        .collect();
    assert_eq!(
        changed,
        ["index/by-spec/001-a.json", "registry/by-spec/001-a.json",],
        "exactly the declaring spec's shards, in each tree; 002-b and the \
         package shard are untouched"
    );
    let now = check_report(&cfg, root).unwrap();
    assert!(now.registry.fresh && now.index.fresh);
}

#[test]
fn no_verdict_reads_an_overlay() {
    // §3.6: two equivalent fresh trees, one with the four overlays declared
    // and present and one with neither, reach the same answers from every
    // verb in the gate chain.
    #[derive(Debug, PartialEq)]
    struct Answers {
        compile: (bool, Vec<(String, String)>),
        check: (bool, bool, bool),
        lint: Vec<(String, String)>,
        coverage: (usize, usize, Vec<String>, Vec<String>),
        couple: Vec<Vec<(String, String)>>,
    }
    let answers = |declared_and_present: bool| {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let cfg = corpus(root, declared_and_present, declared_and_present);
        regenerate(&cfg, root);
        let c = compile(&cfg, root).unwrap();
        let chk = check_report(&cfg, root).unwrap();
        let cov = coverage(&cfg, root).unwrap();
        let diffs: [&[&str]; 3] = [
            // Owned code changed without its spec: C-001.
            &["crate-a/src/a.rs"],
            // With its spec: clean.
            &["crate-a/src/a.rs", "specs/001-a/spec.md"],
            // An unclaimed source file: C-002.
            &["crate-a/src/c.rs"],
        ];
        Answers {
            compile: (
                c.validation_passed,
                c.registry
                    .validation
                    .violations
                    .into_iter()
                    .map(|v| (v.code, v.message))
                    .collect(),
            ),
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
            coverage: (
                cov.source_files,
                cov.claimed_files,
                cov.floor_only_files,
                cov.unclaimed_files,
            ),
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
    let with = answers(true);
    let without = answers(false);
    assert!(with.check.0 && with.check.1, "both trees fresh: {with:?}");
    assert_eq!(
        with.couple.iter().map(|v| v.len()).collect::<Vec<_>>(),
        [1, 0, 1],
        "the diffs exercise a refusal, a clearance and an ownership refusal: {with:?}"
    );
    assert_eq!(with, without);
}
