//! Spec 141: a governance edit rewrites one file.
//!
//! The global inputs (`spec-spine.toml` and every `[index] extra_hashed_inputs`
//! match) used to fold into every shard's `shardHash`, so one edit to a root
//! document restamped the whole index and two pull requests editing different
//! root documents conflicted on every shard. These tests pin the replacement:
//! one committed sidecar, `codebase-index/inputs.json`, compared like a shard
//! and folded into the aggregate content hash.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::Command;

use spec_spine_core::shard::{self, BY_PACKAGE_DIR, BY_SPEC_DIR};
use spec_spine_core::{
    Freshness, INPUTS_FILE, check_index_freshness, index, index_dir, index_inputs_file,
    index_shard_files, load_committed_index,
};
use spec_spine_types::{Config, INDEX_INPUTS_SCHEMA, IndexInputs, load_config};

const TOML: &str =
    "[index]\nextra_hashed_inputs = [\"AGENTS.md\", \"CLAUDE.md\", \".github/workflows/*.yml\"]\n";

const WORKFLOW: &str = "name: ci\non: [push]\njobs:\n  test:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - run: cargo test\n";

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, content).unwrap();
}

fn spec(id: &str, file: &str) -> String {
    format!(
        "---\nid: \"{id}\"\ntitle: \"T\"\nstatus: approved\ncreated: \"2026-09-25\"\nimplementation: complete\nsummary: \"s\"\nestablishes:\n  - \"{file}\"\n---\n# {id}\n"
    )
}

/// Two specs, each owning one file, and three governance inputs.
fn fixture() -> (tempfile::TempDir, Config) {
    let tmp = tempfile::tempdir().unwrap();
    let r = tmp.path();
    write(r, "spec-spine.toml", TOML);
    write(r, "specs/001-a/spec.md", &spec("001-a", "src/a.rs"));
    write(r, "specs/002-b/spec.md", &spec("002-b", "src/b.rs"));
    write(r, "src/a.rs", "pub fn a() {}\n");
    write(r, "src/b.rs", "pub fn b() {}\n");
    write(r, "AGENTS.md", "# Agents\n\nOne.\n");
    write(r, "CLAUDE.md", "# Claude\n\nOne.\n");
    write(r, ".github/workflows/ci.yml", WORKFLOW);
    let cfg = load_config(TOML).unwrap();
    (tmp, cfg)
}

/// Write the index as `spec-spine index` does: both shard directories and the
/// inputs sidecar.
fn emit(cfg: &Config, root: &Path) {
    let outcome = index(cfg, root).unwrap();
    let dir = index_dir(cfg, root);
    let (by_spec, by_package) = index_shard_files(&outcome.shards).unwrap();
    shard::sync_dir(&dir.join(BY_SPEC_DIR), &by_spec).unwrap();
    shard::sync_dir(&dir.join(BY_PACKAGE_DIR), &by_package).unwrap();
    let (name, content) = index_inputs_file(&outcome.shards).unwrap();
    fs::write(dir.join(name), content).unwrap();
}

/// Every file under the index directory, relative path to bytes.
fn tree(cfg: &Config, root: &Path) -> BTreeMap<String, Vec<u8>> {
    fn walk(base: &Path, dir: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
        for e in fs::read_dir(dir).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                walk(base, &p, out);
            } else {
                let rel = p
                    .strip_prefix(base)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                out.insert(rel, fs::read(&p).unwrap());
            }
        }
    }
    let mut out = BTreeMap::new();
    let dir = index_dir(cfg, root);
    walk(&dir, &dir, &mut out);
    out
}

fn changed(a: &BTreeMap<String, Vec<u8>>, b: &BTreeMap<String, Vec<u8>>) -> Vec<String> {
    let mut keys: Vec<&String> = a.keys().chain(b.keys()).collect();
    keys.sort();
    keys.dedup();
    keys.into_iter()
        .filter(|k| a.get(*k) != b.get(*k))
        .cloned()
        .collect()
}

fn stale_lines(cfg: &Config, root: &Path) -> Vec<String> {
    match check_index_freshness(cfg, root).unwrap() {
        Freshness::Fresh => Vec::new(),
        Freshness::Stale { actual, .. } => actual
            .lines()
            .skip(1)
            .map(|l| l.trim().to_string())
            .collect(),
    }
}

/// 3.1 and 3.2: an edit to a governance input rewrites the sidecar and nothing
/// else. Before spec 141 it rewrote every shard.
#[test]
fn a_governance_edit_rewrites_only_the_inputs_file() {
    let (tmp, cfg) = fixture();
    let r = tmp.path();
    emit(&cfg, r);
    let before = tree(&cfg, r);
    assert!(before.contains_key(INPUTS_FILE), "{:?}", before.keys());
    assert!(before.keys().filter(|k| k.starts_with("by-spec/")).count() == 2);

    write(r, "AGENTS.md", "# Agents\n\nTwo.\n");
    emit(&cfg, r);
    assert_eq!(
        changed(&before, &tree(&cfg, r)),
        vec![INPUTS_FILE.to_string()]
    );

    write(r, "spec-spine.toml", &format!("{TOML}# a comment\n"));
    let cfg2 = load_config(&fs::read_to_string(r.join("spec-spine.toml")).unwrap()).unwrap();
    let mid = tree(&cfg2, r);
    emit(&cfg2, r);
    assert_eq!(
        changed(&mid, &tree(&cfg2, r)),
        vec![INPUTS_FILE.to_string()],
        "a config edit that changes no resolution is also one file"
    );
}

/// 3.1: a spec's own inputs still stale exactly its shard, and not the sidecar.
#[test]
fn a_spec_edit_still_rewrites_only_its_shard() {
    let (tmp, cfg) = fixture();
    let r = tmp.path();
    emit(&cfg, r);
    let before = tree(&cfg, r);
    write(
        r,
        "specs/001-a/spec.md",
        &format!("{}\nMore.\n", spec("001-a", "src/a.rs")),
    );
    emit(&cfg, r);
    assert_eq!(
        changed(&before, &tree(&cfg, r)),
        vec!["by-spec/001-a.json".to_string()]
    );
}

/// 3.3: freshness names the sidecar as the one drifted file, and a missing
/// sidecar is drift rather than a pass.
#[test]
fn freshness_names_the_inputs_file() {
    let (tmp, cfg) = fixture();
    let r = tmp.path();
    emit(&cfg, r);
    assert!(stale_lines(&cfg, r).is_empty());

    write(r, "CLAUDE.md", "# Claude\n\nTwo.\n");
    assert_eq!(
        stale_lines(&cfg, r),
        vec![format!("modified {INPUTS_FILE}")]
    );

    emit(&cfg, r);
    assert!(stale_lines(&cfg, r).is_empty());

    fs::remove_file(index_dir(&cfg, r).join(INPUTS_FILE)).unwrap();
    assert_eq!(stale_lines(&cfg, r), vec![format!("missing {INPUTS_FILE}")]);
}

/// 3.2: one member per input, keyed by path, and spec 060's projection still
/// applies: a `uses:` ref bump leaves the sidecar unchanged, a `run:` edit does
/// not.
#[test]
fn the_sidecar_lists_each_input_and_projects_workflows() {
    let (tmp, cfg) = fixture();
    let r = tmp.path();
    emit(&cfg, r);
    let path = index_dir(&cfg, r).join(INPUTS_FILE);
    let parsed: IndexInputs = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let keys: Vec<&str> = parsed.inputs.keys().map(String::as_str).collect();
    assert_eq!(
        keys,
        vec![
            ".github/workflows/ci.yml",
            "AGENTS.md",
            "CLAUDE.md",
            "spec-spine.toml"
        ]
    );

    let before = fs::read(&path).unwrap();
    write(
        r,
        ".github/workflows/ci.yml",
        &WORKFLOW.replace("checkout@v4", "checkout@v5"),
    );
    emit(&cfg, r);
    assert_eq!(
        fs::read(&path).unwrap(),
        before,
        "a uses: ref bump is not a governance change"
    );

    write(
        r,
        ".github/workflows/ci.yml",
        &WORKFLOW.replace("cargo test", "cargo bench"),
    );
    emit(&cfg, r);
    assert_ne!(fs::read(&path).unwrap(), before, "a run: edit is");
}

/// 3.4: the aggregate content hash still moves with a governance input, and the
/// value assembled from the committed tree equals the one computed at emit, so
/// an attestation's `indexHash` covers the inputs as it did before.
#[test]
fn the_aggregate_still_covers_the_inputs() {
    let (tmp, cfg) = fixture();
    let r = tmp.path();
    emit(&cfg, r);
    let emitted = index(&cfg, r).unwrap().index.build.content_hash;
    let assembled = load_committed_index(&cfg, r).unwrap().build.content_hash;
    assert_eq!(emitted, assembled);

    write(r, "AGENTS.md", "# Agents\n\nThree.\n");
    let moved = index(&cfg, r).unwrap().index.build.content_hash;
    assert_ne!(emitted, moved, "a governance edit must move the aggregate");
    emit(&cfg, r);
    assert_eq!(
        moved,
        load_committed_index(&cfg, r).unwrap().build.content_hash
    );
}

/// 3.2: the sidecar conforms to its embedded schema.
#[test]
fn the_sidecar_conforms_to_its_schema() {
    let (tmp, cfg) = fixture();
    let (_, content) = index_inputs_file(&index(&cfg, tmp.path()).unwrap().shards).unwrap();
    let schema: serde_json::Value = serde_json::from_str(INDEX_INPUTS_SCHEMA).unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();
    let instance: serde_json::Value = serde_json::from_str(&content).unwrap();
    let errs: Vec<String> = validator
        .iter_errors(&instance)
        .map(|e| e.to_string())
        .collect();
    assert!(errs.is_empty(), "{errs:?}");
}

fn git(root: &Path, args: &[&str]) -> std::process::Output {
    Command::new("git")
        .current_dir(root)
        .args(["-c", "user.email=t@example.invalid", "-c", "user.name=t"])
        .args(["-c", "commit.gpgsign=false", "-c", "core.autocrlf=false"])
        .args(args)
        .output()
        .expect("git runs")
}

/// D-2 and the purpose itself: two branches each edit a different governance
/// document, regenerate and commit; git merges them with no conflict (the two
/// inputs are adjacent entries in the sidecar), and the merged tree is fresh
/// once regenerated, with only the sidecar to rewrite.
#[test]
fn two_edits_to_different_inputs_merge_cleanly() {
    let (tmp, cfg) = fixture();
    let r = tmp.path();
    emit(&cfg, r);
    assert!(git(r, &["init", "-q", "-b", "main"]).status.success());
    assert!(git(r, &["add", "-A"]).status.success());
    assert!(git(r, &["commit", "-qm", "base"]).status.success());

    assert!(git(r, &["checkout", "-qb", "a"]).status.success());
    write(r, "AGENTS.md", "# Agents\n\nEdited on a.\n");
    emit(&cfg, r);
    assert!(git(r, &["commit", "-qam", "a"]).status.success());

    assert!(git(r, &["checkout", "-q", "main"]).status.success());
    assert!(git(r, &["checkout", "-qb", "b"]).status.success());
    write(r, "CLAUDE.md", "# Claude\n\nEdited on b.\n");
    emit(&cfg, r);
    assert!(git(r, &["commit", "-qam", "b"]).status.success());

    let merge = git(r, &["merge", "-q", "--no-edit", "a"]);
    assert!(
        merge.status.success(),
        "the merge conflicted:\n{}\n{}",
        String::from_utf8_lossy(&merge.stdout),
        String::from_utf8_lossy(&merge.stderr)
    );
    // Each side recorded its own edit, and git combined them line by line: the
    // merged sidecar is exactly what the merged documents index to.
    assert!(
        stale_lines(&cfg, r).is_empty(),
        "{:?}",
        stale_lines(&cfg, r)
    );
}
