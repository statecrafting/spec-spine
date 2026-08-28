//! Ownership-coverage tests (spec 032): the classifier's tiers, the universe
//! the report and the gate share, determinism, the freshness guard, and the
//! JSON facade. Fixtures are real trees fed through `index`, so the
//! implementing-path and resolved-unit shapes are the indexer's own.

use std::fs;
use std::path::Path;

use spec_spine_core::shard::{self, BY_PACKAGE_DIR, BY_SPEC_DIR};
use spec_spine_core::{
    Ownership, classify, coverage, coverage_json, coverage_with, enumerate_source_files,
    in_coverage_universe, index, index_dir, index_shard_files,
};
use spec_spine_types::{Config, CoverageReport, Error, load_config};

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, content).unwrap();
}

fn spec(id: &str, body: &str) -> String {
    format!(
        "---\nid: \"{id}\"\ntitle: \"T\"\nstatus: approved\ncreated: \"2026-06-09\"\nsummary: \"s\"\n{body}---\n# {id}\n"
    )
}

/// Write the index shards as the CLI's `spec-spine index` does.
fn emit_index(cfg: &Config, repo: &Path) {
    let outcome = index(cfg, repo).unwrap();
    let dir = index_dir(cfg, repo);
    let (by_spec, by_package) = index_shard_files(&outcome.shards).unwrap();
    shard::sync_dir(&dir.join(BY_SPEC_DIR), &by_spec).unwrap();
    shard::sync_dir(&dir.join(BY_PACKAGE_DIR), &by_package).unwrap();
}

/// Two crates. `crates/a` names `000-floor` in its manifest; `001-a` claims
/// `src/lib.rs` (file unit) and `src/sub/` (subtree unit) and is named by a
/// comment header in `src/hdr.rs`; `002-r` only references `src/ref.rs`;
/// `src/stray.rs` is floor-only. `crates/b` has no floor; `003-d` claims
/// `src/deep` by directory unit; `src/lib.rs` is unclaimed. Plus the noise the
/// universe must ignore: an excluded dir, a bypassed dir, a non-source file,
/// and a script outside every package.
fn fixture() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    let r = tmp.path();
    write(
        r,
        "Cargo.toml",
        "[workspace]\nmembers = [\"crates/a\", \"crates/b\"]\n",
    );
    write(
        r,
        "crates/a/Cargo.toml",
        "[package]\nname = \"a\"\nversion = \"0.1.0\"\n\
         [package.metadata.spec-spine]\nspec = \"000-floor\"\n",
    );
    write(r, "crates/a/src/lib.rs", "pub fn a() {}\n");
    write(r, "crates/a/src/sub/one.rs", "pub fn one() {}\n");
    write(
        r,
        "crates/a/src/hdr.rs",
        "// Spec: specs/001-a/spec.md\npub fn h() {}\n",
    );
    write(r, "crates/a/src/ref.rs", "pub fn r() {}\n");
    write(r, "crates/a/src/stray.rs", "pub fn s() {}\n");
    write(r, "crates/a/target/gen.rs", "pub fn g() {}\n");
    write(r, "crates/a/vendor/v.rs", "pub fn v() {}\n");
    write(r, "crates/a/notes.md", "# notes\n");
    write(
        r,
        "crates/b/Cargo.toml",
        "[package]\nname = \"b\"\nversion = \"0.1.0\"\n",
    );
    write(r, "crates/b/src/lib.rs", "pub fn b() {}\n");
    write(r, "crates/b/src/deep/x.rs", "pub fn x() {}\n");
    write(r, "scripts/tool.py", "print('outside every package')\n");

    write(r, "specs/000-floor/spec.md", &spec("000-floor", ""));
    write(
        r,
        "specs/001-a/spec.md",
        &spec(
            "001-a",
            "establishes:\n  - \"crates/a/src/lib.rs\"\n  - \"crates/a/src/sub/\"\n",
        ),
    );
    write(
        r,
        "specs/002-r/spec.md",
        &spec(
            "002-r",
            "references:\n  - { unit: { kind: file, path: \"crates/a/src/ref.rs\" }, role: context }\n",
        ),
    );
    write(
        r,
        "specs/003-d/spec.md",
        &spec(
            "003-d",
            "establishes:\n  - { kind: directory, path: \"crates/b/src/deep\" }\n",
        ),
    );
    tmp
}

fn fixture_config() -> Config {
    let mut cfg = Config::default();
    cfg.coupling
        .bypass_prefixes
        .push("crates/a/vendor/".to_string());
    cfg
}

#[test]
fn report_classifies_every_tier() {
    let fx = fixture();
    let cfg = fixture_config();
    emit_index(&cfg, fx.path());
    let report = coverage(&cfg, fx.path()).unwrap();

    assert_eq!(report.source_files, 7, "{report:#?}");
    assert_eq!(report.claimed_files, 4, "{report:#?}");
    assert_eq!(
        report.floor_only_files,
        vec!["crates/a/src/ref.rs", "crates/a/src/stray.rs"],
        "a referenced file is floor-only: references are not ownership"
    );
    assert_eq!(report.unclaimed_files, vec!["crates/b/src/lib.rs"]);
    assert_eq!(report.untraced_files(), 3);
    assert!(!report.is_fully_claimed());

    assert_eq!(report.packages.len(), 2);
    let a = &report.packages[0];
    assert_eq!(a.path, "crates/a");
    assert_eq!(a.floor_spec.as_deref(), Some("000-floor"));
    assert_eq!(
        (a.source_files, a.claimed_files, a.floor_only, a.unclaimed),
        (5, 3, 2, 0)
    );
    let b = &report.packages[1];
    assert_eq!(b.path, "crates/b");
    assert_eq!(b.floor_spec, None);
    assert_eq!(
        (b.source_files, b.claimed_files, b.floor_only, b.unclaimed),
        (2, 1, 0, 1)
    );
}

#[test]
fn classifier_tiers_directly() {
    let fx = fixture();
    let cfg = fixture_config();
    let idx = index(&cfg, fx.path()).unwrap().index;
    assert_eq!(classify(&idx, "crates/a/src/lib.rs"), Ownership::Specific);
    assert_eq!(
        classify(&idx, "crates/a/src/sub/one.rs"),
        Ownership::Specific,
        "a subtree unit covers its files"
    );
    assert_eq!(
        classify(&idx, "crates/a/src/hdr.rs"),
        Ownership::Specific,
        "a comment header is a specific claim"
    );
    assert_eq!(
        classify(&idx, "crates/a/src/ref.rs"),
        Ownership::FloorOnly(vec!["000-floor".to_string()])
    );
    assert_eq!(
        classify(&idx, "crates/a/src/stray.rs"),
        Ownership::FloorOnly(vec!["000-floor".to_string()])
    );
    assert_eq!(
        classify(&idx, "crates/b/src/deep/x.rs"),
        Ownership::Specific,
        "a directory-kind unit covers its subtree"
    );
    assert_eq!(classify(&idx, "crates/b/src/lib.rs"), Ownership::Unowned);
}

#[test]
fn universe_excludes_noise_and_is_the_gates_predicate() {
    let fx = fixture();
    let cfg = fixture_config();
    let idx = index(&cfg, fx.path()).unwrap().index;
    let files = enumerate_source_files(&cfg, fx.path(), &idx);
    assert_eq!(
        files,
        vec![
            "crates/a/src/hdr.rs",
            "crates/a/src/lib.rs",
            "crates/a/src/ref.rs",
            "crates/a/src/stray.rs",
            "crates/a/src/sub/one.rs",
            "crates/b/src/deep/x.rs",
            "crates/b/src/lib.rs",
        ]
    );
    for f in &files {
        assert!(in_coverage_universe(&cfg, &idx, f), "{f}");
    }
    for out in [
        "crates/a/target/gen.rs", // resolver exclusion
        "crates/a/vendor/v.rs",   // adopter bypass
        "crates/a/notes.md",      // not a source file
        "scripts/tool.py",        // outside every package
        "crates/a/Cargo.toml",    // a manifest, not source
        "specs/001-a/spec.md",    // the corpus
    ] {
        assert!(!in_coverage_universe(&cfg, &idx, out), "{out}");
    }
}

#[test]
fn report_is_deterministic_and_a_function_of_the_path_set() {
    let fx = fixture();
    let cfg = fixture_config();
    let idx = index(&cfg, fx.path()).unwrap().index;
    let files = enumerate_source_files(&cfg, fx.path(), &idx);
    let a = coverage_with(&cfg, &idx, &files);
    let b = coverage_with(&cfg, &idx, &files);
    assert_eq!(a, b);

    // Unsorted, duplicated, and noisy input yields the same report.
    let mut noisy: Vec<String> = files.iter().rev().cloned().collect();
    noisy.extend(files.iter().cloned());
    noisy.push("scripts/tool.py".to_string());
    noisy.push("crates/a/target/gen.rs".to_string());
    assert_eq!(coverage_with(&cfg, &idx, &noisy), a);
}

#[test]
fn coverage_is_freshness_guarded() {
    let fx = fixture();
    let cfg = fixture_config();
    // No index at all: an artifact-missing I/O error (3), as for `couple`.
    assert!(matches!(coverage(&cfg, fx.path()), Err(Error::Io(_))));

    emit_index(&cfg, fx.path());
    assert!(coverage(&cfg, fx.path()).is_ok());

    // A spec edit without re-indexing: stale (2), never a report over the
    // wrong ledger.
    write(
        fx.path(),
        "specs/001-a/spec.md",
        &spec("001-a", "establishes:\n  - \"crates/a/src/\"\n"),
    );
    assert!(matches!(
        coverage(&cfg, fx.path()),
        Err(Error::Stale { .. })
    ));
}

#[test]
fn crate_unit_claims_the_whole_package() {
    let tmp = tempfile::tempdir().unwrap();
    let r = tmp.path();
    write(r, "Cargo.toml", "[workspace]\nmembers = [\"pkg\"]\n");
    write(
        r,
        "pkg/Cargo.toml",
        "[package]\nname = \"pkg\"\nversion = \"0.1.0\"\n",
    );
    write(r, "pkg/src/lib.rs", "pub fn a() {}\n");
    write(r, "pkg/src/other.rs", "pub fn b() {}\n");
    write(
        r,
        "specs/001-x/spec.md",
        &spec("001-x", "establishes:\n  - { kind: crate, id: \"pkg\" }\n"),
    );
    let cfg = Config::default();
    emit_index(&cfg, r);
    let report = coverage(&cfg, r).unwrap();
    assert_eq!((report.source_files, report.claimed_files), (2, 2));
    assert!(report.is_fully_claimed(), "{report:#?}");
}

#[test]
fn root_package_attributes_nested_files_to_the_nested_package() {
    let tmp = tempfile::tempdir().unwrap();
    let r = tmp.path();
    write(
        r,
        "Cargo.toml",
        "[package]\nname = \"root\"\nversion = \"0.1.0\"\n\
         [package.metadata.spec-spine]\nspec = \"000-root\"\n\
         [workspace]\nmembers = [\"crates/inner\"]\n",
    );
    write(r, "src/main.rs", "fn main() {}\n");
    write(
        r,
        "crates/inner/Cargo.toml",
        "[package]\nname = \"inner\"\nversion = \"0.1.0\"\n",
    );
    write(r, "crates/inner/src/lib.rs", "pub fn i() {}\n");
    write(r, "specs/000-root/spec.md", &spec("000-root", ""));
    let cfg = Config::default();
    emit_index(&cfg, r);
    let report = coverage(&cfg, r).unwrap();

    assert_eq!(
        report.source_files, 2,
        "nested file counted once: {report:#?}"
    );
    let root = report.packages.iter().find(|p| p.path.is_empty()).unwrap();
    let inner = report
        .packages
        .iter()
        .find(|p| p.path == "crates/inner")
        .unwrap();
    assert_eq!((root.source_files, root.floor_only), (1, 1));
    assert_eq!((inner.source_files, inner.floor_only), (1, 1));
    // The root floor still covers the nested file (it is inside the root
    // package too), so it is floor-only rather than unclaimed.
    assert_eq!(
        report.floor_only_files,
        vec!["crates/inner/src/lib.rs", "src/main.rs"]
    );
    assert!(report.unclaimed_files.is_empty());
}

#[test]
fn require_ownership_defaults_off_and_parses() {
    assert!(!Config::default().coupling.require_ownership);
    let cfg = load_config("[coupling]\nrequire_ownership = true\n").unwrap();
    assert!(cfg.coupling.require_ownership);
}

#[test]
fn facade_round_trips_the_report() {
    let fx = fixture();
    let cfg = fixture_config();
    emit_index(&cfg, fx.path());
    let config_json = serde_json::to_string(&cfg).unwrap();
    let json = coverage_json(&config_json, fx.path().to_str().unwrap()).unwrap();
    let report: CoverageReport = serde_json::from_str(&json).unwrap();
    assert_eq!(report, coverage(&cfg, fx.path()).unwrap());
    assert!(json.contains("\"floorOnlyFiles\""), "camelCase wire form");
}
