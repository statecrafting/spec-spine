//! Ownership-coverage tests (spec 029): the classifier's tiers, the universe
//! the report and the gate share, determinism, the freshness guard, and the
//! JSON facade. Fixtures are real trees fed through `index`, so the
//! implementing-path and resolved-unit shapes are the indexer's own.

use std::fs;
use std::path::Path;

use spec_spine_core::shard::{self, BY_PACKAGE_DIR, BY_SPEC_DIR};
use spec_spine_core::{
    GovernedScope, Ownership, classify, coverage, coverage_json, coverage_with,
    coverage_with_inventory, enumerate_source_files, in_coverage_universe,
    in_coverage_universe_with, index, index_dir, index_shard_files, walk_repository,
};
use spec_spine_types::{
    Config, CoverageReport, Enumeration, Error, Inventory, InventoryProvenance, load_config,
};

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
    // Spec 141: the inputs sidecar, as `spec-spine index` writes it.
    let (inputs_name, inputs) = spec_spine_core::index_inputs_file(&outcome.shards).unwrap();
    std::fs::write(dir.join(inputs_name), inputs).unwrap();
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

// ===== spec 036: layout.state_dir leaves the coverage universe =====

/// A repo at 100% stays at 100% when state files appear under a declared root,
/// and the denominator shrinks by exactly the number of files there.
///
/// Asserted as counts rather than as a percentage: declaring a state root moves
/// a coverage figure, and the movement should be auditable rather than merely
/// plausible.
#[test]
fn a_declared_state_root_leaves_both_sides_of_the_coverage_ratio() {
    let tmp = tempfile::tempdir().unwrap();
    let r = tmp.path();
    write(r, "Cargo.toml", "[workspace]\nmembers = [\"crates/a\"]\n");
    write(
        r,
        "crates/a/Cargo.toml",
        "[package]\nname = \"a\"\nversion = \"0.1.0\"\n\
         [package.metadata.spec-spine]\nspec = \"001-a\"\n",
    );
    write(r, "crates/a/src/lib.rs", "pub fn a() {}\n");
    write(
        r,
        "specs/001-a/spec.md",
        &spec("001-a", "establishes:\n  - \"crates/a/src/lib.rs\"\n"),
    );

    let clean = load_config("").unwrap();
    emit_index(&clean, r);
    let before = coverage(&clean, r).unwrap();
    assert_eq!((before.source_files, before.claimed_files), (1, 1));
    assert!(before.is_fully_claimed());

    // A tool writes two state files inside the package. Undeclared, they are
    // unclaimed debt and the ratchet would refuse the next PR touching them.
    write(r, "crates/a/state/journal.rs", "pub fn j() {}\n");
    write(r, "crates/a/state/queue.rs", "pub fn q() {}\n");
    emit_index(&clean, r);
    let undeclared = coverage(&clean, r).unwrap();
    assert_eq!(
        (undeclared.source_files, undeclared.claimed_files),
        (3, 1),
        "control: state files count as source until the root is declared"
    );
    assert_eq!(
        undeclared.floor_only_files.len(),
        2,
        "the package has a manifest floor, so they are floor-only debt"
    );

    // Declared, they leave the universe entirely: numerator and denominator.
    let declared_cfg = load_config("[layout]\nstate_dir = \"crates/a/state\"\n").unwrap();
    emit_index(&declared_cfg, r);
    let after = coverage(&declared_cfg, r).unwrap();
    assert_eq!(
        (after.source_files, after.claimed_files),
        (1, 1),
        "the denominator shrinks by exactly the two state files"
    );
    assert!(
        after.unclaimed_files.is_empty(),
        "{:?}",
        after.unclaimed_files
    );
    assert!(after.floor_only_files.is_empty());
    assert!(after.is_fully_claimed(), "100% before, 100% after");

    // The enumeration agrees with the classifier: the walk never yields them.
    let index = spec_spine_core::load_committed_index(&declared_cfg, r).unwrap();
    let files = enumerate_source_files(&declared_cfg, r, &index);
    assert_eq!(files, vec!["crates/a/src/lib.rs".to_string()]);
    for path in ["crates/a/state/journal.rs", "crates/a/state/queue.rs"] {
        assert!(
            !in_coverage_universe(&declared_cfg, &index, path),
            "{path} is state, not source"
        );
    }
}

/// Spec 036 3.2: files under the root contribute to no content hash, so a tool
/// writing its own state can never make the committed ledger stale.
#[test]
fn writing_state_does_not_stale_the_committed_index() {
    let tmp = tempfile::tempdir().unwrap();
    let r = tmp.path();
    write(r, "Cargo.toml", "[workspace]\nmembers = [\"crates/a\"]\n");
    write(
        r,
        "crates/a/Cargo.toml",
        "[package]\nname = \"a\"\nversion = \"0.1.0\"\n\
         [package.metadata.spec-spine]\nspec = \"001-a\"\n",
    );
    write(r, "crates/a/src/lib.rs", "pub fn a() {}\n");
    write(
        r,
        "specs/001-a/spec.md",
        &spec("001-a", "establishes:\n  - \"crates/a/src/lib.rs\"\n"),
    );
    // An `extra_hashed_inputs` pattern wide enough to reach into the state root
    // is the ordinary case: the adopter states the root once, not twice.
    write(
        r,
        "spec-spine.toml",
        "[layout]\nstate_dir = \"tool-state\"\n\
         [index]\nextra_hashed_inputs = [\"**/*.jsonl\"]\n",
    );
    let cfg = load_config(&fs::read_to_string(r.join("spec-spine.toml")).unwrap()).unwrap();

    write(r, "tool-state/journal.jsonl", "{\"runs\":1}\n");
    emit_index(&cfg, r);
    assert!(matches!(
        spec_spine_core::check_index_freshness(&cfg, r).unwrap(),
        spec_spine_core::Freshness::Fresh
    ));

    // The tool writes again. Nothing about the corpus changed, so the ledger
    // must still be vouching for it.
    write(r, "tool-state/journal.jsonl", "{\"runs\":2}\n");
    write(r, "tool-state/nested/queue.jsonl", "{\"depth\":1}\n");
    assert!(
        matches!(
            spec_spine_core::check_index_freshness(&cfg, r).unwrap(),
            spec_spine_core::Freshness::Fresh
        ),
        "state writes must not stale the index"
    );

    // Control: a hashed input outside the root still stales it, so the filter
    // is scoped to the state root rather than switching hashing off.
    write(r, "other.jsonl", "{\"x\":1}\n");
    assert!(
        matches!(
            spec_spine_core::check_index_freshness(&cfg, r).unwrap(),
            spec_spine_core::Freshness::Stale { .. }
        ),
        "a hashed input outside the state root still stales the index"
    );
}

/// Spec 036 3.4: a spec claiming a unit inside the ungoverned root is a
/// contradiction, reported at error tier so `lint` exits 1 without
/// `--fail-on-warn`. Neither the claim nor the bypass wins.
#[test]
fn a_claim_inside_the_state_root_is_an_l006_error() {
    let tmp = tempfile::tempdir().unwrap();
    let r = tmp.path();
    write(
        r,
        "specs/001-a/spec.md",
        &spec(
            "001-a",
            "establishes:\n  - \"tool-state/journal.rs\"\n  - \"src/lib.rs\"\n\
             references:\n  - { unit: { kind: file, path: \"tool-state/cited.rs\" }, role: context }\n",
        ),
    );
    let cfg = load_config("[layout]\nstate_dir = \"tool-state\"\n").unwrap();
    let report = spec_spine_core::lint(&cfg, r).unwrap();

    let l006: Vec<_> = report
        .violations
        .iter()
        .filter(|v| v.code == "L-006")
        .collect();
    assert_eq!(
        l006.len(),
        1,
        "one claim is inside the root: {:?}",
        report.violations
    );
    assert_eq!(l006[0].severity, spec_spine_types::Severity::Error);
    assert!(
        l006[0].message.contains("001-a"),
        "names the spec: {}",
        l006[0].message
    );
    assert!(
        l006[0].message.contains("tool-state/journal.rs"),
        "names the unit: {}",
        l006[0].message
    );
    // `references` is non-owning (spec 031), so citing a file inside the root
    // is not the contradiction this reports.
    assert!(
        !l006[0].message.contains("cited.rs"),
        "a citation is not a claim: {}",
        l006[0].message
    );

    // Undeclared, the same corpus lints clean of L-006.
    let clean = spec_spine_core::lint(&Config::default(), r).unwrap();
    assert!(!clean.violations.iter().any(|v| v.code == "L-006"));
}

/// Every ownership-bearing edge is checked, `supersedes` included.
///
/// A partial `supersedes` item carries the unit whose authority transfers (spec
/// 018), so it claims a path exactly as `establishes` does. Missing it would let
/// a superseding spec hold a claim inside the ungoverned root that the gate
/// bypasses unconditionally and no diagnostic ever names, which is precisely the
/// contradiction `L-006` exists to surface.
#[test]
fn l006_covers_every_ownership_bearing_edge() {
    let tmp = tempfile::tempdir().unwrap();
    let r = tmp.path();
    write(
        r,
        "specs/001-old/spec.md",
        &spec("001-old", "establishes:\n  - \"src/old.rs\"\n"),
    );
    // One claim per path-bearing ownership edge other than `establishes`, which
    // the neighbouring test already covers. Written as flow mappings on single
    // lines: a wrapped one silently becomes a different YAML document.
    let edges = concat!(
        "supersedes:\n",
        "  - { spec: \"001-old\", scope: partial, unit: { kind: file, path: \"tool-state/old.rs\" } }\n",
        "co_authority:\n",
        "  - { unit: { kind: section, file: \"tool-state/shared.md\", anchor: \"x\" } }\n",
        "constrains:\n",
        "  - { flavor: invariant-freeze, unit: { kind: file, path: \"tool-state/frozen.rs\" } }\n",
    );
    write(r, "specs/002-new/spec.md", &spec("002-new", edges));

    let cfg = load_config("[layout]\nstate_dir = \"tool-state\"\n").unwrap();
    let report = spec_spine_core::lint(&cfg, r).unwrap();
    let claimed: Vec<&str> = report
        .violations
        .iter()
        .filter(|v| v.code == "L-006")
        .map(|v| v.message.as_str())
        .collect();

    assert_eq!(
        claimed.len(),
        3,
        "one per claim inside the root: {claimed:?}"
    );
    for path in [
        "tool-state/old.rs",
        "tool-state/shared.md",
        "tool-state/frozen.rs",
    ] {
        assert!(
            claimed.iter().any(|m| m.contains(path)),
            "{path} is claimed and must be reported: {claimed:?}"
        );
    }
}

/// No two lint diagnostics share a code. `L-006` was the next free code when
/// spec 036 was written; the spec makes "the next free code in the band" the
/// binding rule, so this asserts the namespace rather than trusting a comment.
#[test]
fn lint_diagnostic_codes_are_unique() {
    let src =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/lint.rs")).unwrap();
    let mut codes: Vec<String> = Vec::new();
    for (i, _) in src.match_indices("\"L-") {
        let code: String = src[i + 1..].chars().take(5).collect();
        if code.len() == 5 && code[2..].chars().all(|c| c.is_ascii_digit()) {
            codes.push(code);
        }
    }
    codes.sort();
    codes.dedup();
    assert!(codes.contains(&"L-006".to_string()), "{codes:?}");
    // Each code appears at exactly one emission site.
    for code in &codes {
        let sites = src.matches(&format!("\"{code}\"")).count();
        assert_eq!(sites, 1, "{code} is emitted from {sites} places");
    }
}

/// Spec 036 3.2: the resolver does not scan the root, so no unit ever resolves
/// to a path inside it. Asserted through a symbol unit, which is the only kind
/// that could reach in without naming the path.
///
/// Gated on `symbol-resolution` (spec 025) because the control half needs the
/// symbol to resolve when nothing is declared, and feature-off it never does.
/// The walk itself is covered feature-independently by the `enumerate_source_files`
/// assertion in `a_declared_state_root_leaves_both_sides_of_the_coverage_ratio`.
#[cfg(feature = "symbol-resolution")]
#[test]
fn the_resolver_does_not_reach_into_the_state_root() {
    let tmp = tempfile::tempdir().unwrap();
    let r = tmp.path();
    write(r, "Cargo.toml", "[workspace]\nmembers = [\"crates/a\"]\n");
    write(
        r,
        "crates/a/Cargo.toml",
        "[package]\nname = \"a\"\nversion = \"0.1.0\"\n\
         [package.metadata.spec-spine]\nspec = \"001-a\"\n",
    );
    // The only definition of `hidden` lives inside what will become the root.
    write(r, "crates/a/src/lib.rs", "pub mod state;\n");
    write(r, "crates/a/src/state/mod.rs", "pub fn hidden() {}\n");
    write(
        r,
        "specs/001-a/spec.md",
        &spec(
            "001-a",
            "establishes:\n  - { kind: symbol, id: \"a::state::hidden\" }\n",
        ),
    );

    // Control: undeclared, the symbol resolves to the file that defines it.
    let clean = load_config("").unwrap();
    let resolved = index(&clean, r).unwrap();
    let located = resolved
        .index
        .traceability
        .mappings
        .iter()
        .flat_map(|m| &m.resolved_units)
        .any(|u| !u.locations.is_empty());
    assert!(
        located,
        "control: the symbol resolves when nothing is declared"
    );

    // Declared: the definition is never scanned, so the unit does not resolve.
    let cfg = load_config("[layout]\nstate_dir = \"crates/a/src/state\"\n").unwrap();
    let out = index(&cfg, r).unwrap();
    for unit in out
        .index
        .traceability
        .mappings
        .iter()
        .flat_map(|m| &m.resolved_units)
    {
        for loc in &unit.locations {
            assert!(
                !cfg.layout.is_state_path(&loc.file),
                "resolved into the state root: {}",
                loc.file
            );
        }
    }
}

// ── spec 052: an empty coverage universe is a refusal, not a pass ─────────

/// §3.2 + §3.3: no discovered package is an empty universe, and the reason
/// names the likely cause rather than only the condition.
#[test]
fn an_empty_universe_with_no_packages_is_reported_with_its_cause() {
    let tmp = tempfile::tempdir().unwrap();
    let r = tmp.path();
    write(r, "specs/001-x/spec.md", &spec("001-x", ""));
    let cfg = Config::default();
    emit_index(&cfg, r);

    let report = coverage(&cfg, r).unwrap();
    assert_eq!(report.source_files, 0, "the denominator is zero");
    assert!(
        report.is_fully_claimed(),
        "vacuously true, which is the trap this spec closes"
    );

    let reason = spec_spine_core::empty_universe(&report).expect("empty");
    assert_eq!(reason, spec_spine_core::EmptyUniverse::NoPackages);
    assert!(
        reason.explain().contains("standalone_rust_workspaces"),
        "names the likely fix: {}",
        reason.explain()
    );
}

/// §3.2: a package with source files is not an empty universe, so the refusal
/// is unreachable on an ordinary repository. This one has four packages and
/// seventy-four source files.
#[test]
fn a_populated_universe_is_not_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let r = tmp.path();
    write(r, "Cargo.toml", "[workspace]\nmembers = [\"crate-a\"]\n");
    write(
        r,
        "crate-a/Cargo.toml",
        "[package]\nname = \"crate-a\"\nversion = \"0.1.0\"\n",
    );
    write(r, "crate-a/src/lib.rs", "pub fn a() {}\n");
    write(r, "specs/001-x/spec.md", &spec("001-x", ""));
    let cfg = Config::default();
    emit_index(&cfg, r);

    let report = coverage(&cfg, r).unwrap();
    assert!(report.source_files > 0);
    assert!(spec_spine_core::empty_universe(&report).is_none());
}

/// §3.3: packages discovered but no source file in them is the other empty
/// case, and it points at a different knob.
#[test]
fn packages_without_source_files_point_at_resolver_exclusions() {
    let tmp = tempfile::tempdir().unwrap();
    let r = tmp.path();
    write(r, "Cargo.toml", "[workspace]\nmembers = [\"crate-a\"]\n");
    write(
        r,
        "crate-a/Cargo.toml",
        "[package]\nname = \"crate-a\"\nversion = \"0.1.0\"\n",
    );
    write(r, "crate-a/src/lib.rs", "pub fn a() {}\n");
    write(r, "specs/001-x/spec.md", &spec("001-x", ""));
    // Prune the only source tree.
    let cfg = load_config("[index]\nresolver_exclusions = [\"src\"]\n").unwrap();
    emit_index(&cfg, r);

    let report = coverage(&cfg, r).unwrap();
    assert!(!report.packages.is_empty(), "a package was discovered");
    assert_eq!(report.source_files, 0, "and none of its files were seen");
    let reason = spec_spine_core::empty_universe(&report).expect("empty");
    assert_eq!(reason, spec_spine_core::EmptyUniverse::NoSourceFiles);
    assert!(
        reason.explain().contains("resolver_exclusions"),
        "names the other knob: {}",
        reason.explain()
    );
}

// ── spec 063 §3.6: coverage can see planned territory ───────────────────────

/// §3.6: a file nothing claims and a file something has planned are different
/// states, and the report could not tell them apart. It reads the DECLARED
/// state from the registry, because a planned unit that has not resolved
/// contributes no `ResolvedUnit` at all (§3.2), which is exactly why coverage
/// was blind to it.
#[test]
fn coverage_reports_planned_territory_separately_from_the_counts() {
    let tmp = tempfile::tempdir().unwrap();
    write(tmp.path(), "Cargo.toml", "[workspace]\nmembers = [\"a\"]\n");
    write(
        tmp.path(),
        "a/Cargo.toml",
        "[package]\nname = \"a\"\nversion = \"0.1.0\"\n",
    );
    write(tmp.path(), "a/src/lib.rs", "pub fn a() {}\n");
    write(
        tmp.path(),
        "specs/001-a/spec.md",
        "---\nid: \"001-a\"\ntitle: \"A\"\nstatus: approved\ncreated: \"2026-09-08\"\n\
         implementation: pending\nsummary: \"s\"\nestablishes:\n  - \"a/src/lib.rs\"\n\
         \x20 - { kind: file, path: \"a/src/future.rs\", planned: true }\n---\n# 001-a\n## body\n",
    );

    let cfg = Config::default();
    let out = index(&cfg, tmp.path()).unwrap();
    let dir = index_dir(&cfg, tmp.path());
    let (by_spec, by_package) = index_shard_files(&out.shards).unwrap();
    shard::sync_dir(&dir.join(BY_SPEC_DIR), &by_spec).unwrap();
    shard::sync_dir(&dir.join(BY_PACKAGE_DIR), &by_package).unwrap();
    // Spec 141: the inputs sidecar, as `spec-spine index` writes it.
    let (inputs_name, inputs) = spec_spine_core::index_inputs_file(&out.shards).unwrap();
    std::fs::write(dir.join(inputs_name), inputs).unwrap();
    let compiled = spec_spine_core::compile(&cfg, tmp.path()).unwrap();
    let reg_dir = spec_spine_core::registry_dir(&cfg, tmp.path());
    shard::sync_dir(
        &reg_dir.join(spec_spine_core::shard::BY_SPEC_DIR),
        &spec_spine_core::registry_shard_files(&compiled.shards).unwrap(),
    )
    .unwrap();

    let report = coverage(&cfg, tmp.path()).unwrap();
    assert_eq!(
        report.planned_territory,
        vec!["001-a: file:a/src/future.rs".to_string()],
        "the declared intention must be visible"
    );
    // And it is NOT counted: these paths are not on disk, so counting a
    // declared intention as coverage would let a spec satisfy
    // `--fail-on-untraced` by promising rather than by writing.
    assert_eq!(report.source_files, 1, "only the file that exists");
    assert_eq!(report.claimed_files, 1);
    assert!(report.is_fully_claimed());
}

// ── spec 130 §3.3: a planned unit that has resolved is not "not yet written" ──

/// The report's planned list reads "declared, not yet written". A planned unit
/// whose path is on disk is written: it resolves, it is counted as claimed,
/// and `L-012` tells the author to drop the flag. Listing it as unwritten as
/// well contradicts the counts beside it.
#[test]
fn coverage_lists_only_planned_territory_that_is_not_yet_written() {
    let tmp = tempfile::tempdir().unwrap();
    write(tmp.path(), "Cargo.toml", "[workspace]\nmembers = [\"a\"]\n");
    write(
        tmp.path(),
        "a/Cargo.toml",
        "[package]\nname = \"a\"\nversion = \"0.1.0\"\n",
    );
    write(tmp.path(), "a/src/lib.rs", "pub fn a() {}\n");
    write(
        tmp.path(),
        "specs/001-a/spec.md",
        "---\nid: \"001-a\"\ntitle: \"A\"\nstatus: approved\ncreated: \"2026-09-08\"\n\
         implementation: in-progress\nsummary: \"s\"\nestablishes:\n\
         \x20 - { kind: file, path: \"a/src/lib.rs\", planned: true }\n\
         \x20 - { kind: file, path: \"a/src/future.rs\", planned: true }\n---\n# 001-a\n## body\n",
    );

    let cfg = Config::default();
    let out = index(&cfg, tmp.path()).unwrap();
    let dir = index_dir(&cfg, tmp.path());
    let (by_spec, by_package) = index_shard_files(&out.shards).unwrap();
    shard::sync_dir(&dir.join(BY_SPEC_DIR), &by_spec).unwrap();
    shard::sync_dir(&dir.join(BY_PACKAGE_DIR), &by_package).unwrap();
    // Spec 141: the inputs sidecar, as `spec-spine index` writes it.
    let (inputs_name, inputs) = spec_spine_core::index_inputs_file(&out.shards).unwrap();
    std::fs::write(dir.join(inputs_name), inputs).unwrap();
    let compiled = spec_spine_core::compile(&cfg, tmp.path()).unwrap();
    let reg_dir = spec_spine_core::registry_dir(&cfg, tmp.path());
    shard::sync_dir(
        &reg_dir.join(spec_spine_core::shard::BY_SPEC_DIR),
        &spec_spine_core::registry_shard_files(&compiled.shards).unwrap(),
    )
    .unwrap();

    let report = coverage(&cfg, tmp.path()).unwrap();
    assert_eq!(
        report.planned_territory,
        vec!["001-a: file:a/src/future.rs".to_string()],
        "the written one is claimed, not planned; the unwritten one stays listed"
    );
    assert_eq!(
        report.claimed_files, 1,
        "the resolved planned unit still claims"
    );
    assert!(report.is_fully_claimed());
}

// ── spec 075: near-miss comment headers ──────────────────────────────────

/// One floorless crate. `claimed.rs` claims through a header; `low.rs` puts a
/// resolving header on line 17; `ghost.rs` names a spec not in the corpus;
/// `doc.rs` uses `//!`; `deep.rs` has a resolving header on line 65, past the
/// report window. `with_headers: false` writes the same files with no header
/// lines at all, the tree a classification comparison needs.
fn near_miss_fixture(with_headers: bool) -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    let r = tmp.path();
    let filler = |n: usize| "// filler\n".repeat(n);
    let header = |h: &str| {
        if with_headers {
            format!("{h}\n")
        } else {
            "// no header\n".into()
        }
    };
    write(r, "Cargo.toml", "[workspace]\nmembers = [\"crates/a\"]\n");
    write(
        r,
        "crates/a/Cargo.toml",
        "[package]\nname = \"a\"\nversion = \"0.1.0\"\n",
    );
    write(
        r,
        "crates/a/src/claimed.rs",
        "// Spec: specs/001-a/spec.md\npub fn c() {}\n",
    );
    write(
        r,
        "crates/a/src/low.rs",
        &format!(
            "{}{}pub fn l() {{}}\n",
            filler(16),
            header("// Spec: specs/001-a/spec.md")
        ),
    );
    write(
        r,
        "crates/a/src/ghost.rs",
        &format!(
            "{}pub fn g() {{}}\n",
            header("// Spec: specs/404-gone/spec.md")
        ),
    );
    write(
        r,
        "crates/a/src/doc.rs",
        &format!(
            "{}pub fn d() {{}}\n",
            header("//! Spec: specs/001-a/spec.md")
        ),
    );
    write(
        r,
        "crates/a/src/deep.rs",
        &format!(
            "{}{}pub fn e() {{}}\n",
            filler(64),
            header("// Spec: specs/001-a/spec.md")
        ),
    );
    write(r, "specs/001-a/spec.md", &spec("001-a", ""));
    emit_index(&Config::default(), r);
    tmp
}

/// §3.3, §3.4: each reason appears with its path, line and reason; the claiming
/// file and the header past the report window produce none.
#[test]
fn coverage_reports_each_near_miss_reason() {
    use spec_spine_types::NearMissReason;
    let fx = near_miss_fixture(true);
    let report = coverage(&Config::default(), fx.path()).unwrap();
    let got: Vec<(&str, usize, NearMissReason, Option<&str>)> = report
        .near_miss_headers
        .iter()
        .map(|m| (m.path.as_str(), m.line, m.reason, m.spec_id.as_deref()))
        .collect();
    assert_eq!(
        got,
        vec![
            (
                "crates/a/src/doc.rs",
                1,
                NearMissReason::DocCommentMarker,
                Some("001-a")
            ),
            (
                "crates/a/src/ghost.rs",
                1,
                NearMissReason::UnknownSpec,
                None
            ),
            (
                "crates/a/src/low.rs",
                17,
                NearMissReason::OutsideWindow,
                Some("001-a")
            ),
        ],
        "claimed.rs claims and deep.rs is past line {}: {got:?}",
        spec_spine_core::index::COMMENT_HEADER_REPORT_WINDOW
    );
    // The payload spelling, since a consumer reads the JSON.
    // Parsed rather than matched as text: the facade's layout is the read
    // emitter's (spec 074), and the member spelling is what is asserted.
    let json: serde_json::Value =
        serde_json::from_str(&coverage_json("{}", fx.path().to_str().unwrap()).unwrap()).unwrap();
    let reasons: Vec<&str> = json["nearMissHeaders"]
        .as_array()
        .expect("nearMissHeaders")
        .iter()
        .map(|m| m["reason"].as_str().unwrap())
        .collect();
    assert_eq!(
        reasons,
        ["doc-comment-marker", "unknown-spec", "outside-window"],
        "{json}"
    );
}

/// §3.4, §3.5: the list explains a classification and changes none. The same
/// tree without the header lines classifies identically, so
/// `--fail-on-untraced`'s verdict is the same with and without near misses.
#[test]
fn near_misses_change_no_classification() {
    let with = coverage(&Config::default(), near_miss_fixture(true).path()).unwrap();
    let without = coverage(&Config::default(), near_miss_fixture(false).path()).unwrap();
    assert!(!with.near_miss_headers.is_empty());
    assert!(without.near_miss_headers.is_empty());
    assert_eq!(with.source_files, without.source_files);
    assert_eq!(with.claimed_files, without.claimed_files);
    assert_eq!(with.unclaimed_files, without.unclaimed_files);
    assert_eq!(with.floor_only_files, without.floor_only_files);
    assert_eq!(with.is_fully_claimed(), without.is_fully_claimed());
    assert!(
        !with.is_fully_claimed(),
        "the near-miss files are unclaimed debt"
    );
    // An empty list is omitted, so a corpus with none emits what it did before.
    let json = serde_json::to_string(&without).unwrap();
    assert!(!json.contains("nearMissHeaders"), "{json}");
}

/// §3.3: the report window's edges, over the pure per-file scan.
#[test]
fn the_report_window_ends_at_line_64() {
    use spec_spine_core::index::near_miss_headers_in;
    let ids: std::collections::BTreeSet<String> = ["001-a".to_string()].into();
    let at = |n: usize| format!("{}// Spec: 001-a\n", "// filler\n".repeat(n - 1));
    assert_eq!(near_miss_headers_in("f.rs", &at(64), &ids).len(), 1);
    assert!(near_miss_headers_in("f.rs", &at(65), &ids).is_empty());
    // A claim inside the window reports nothing, even with a second header low.
    let both = format!(
        "// Spec: 001-a\n{}// Spec: 001-a\n",
        "// filler\n".repeat(20)
    );
    assert!(near_miss_headers_in("f.rs", &both, &ids).is_empty());
}

// ── spec 078: governed scope is declared, not inferred ───────────────────

/// One crate (`crate-a`, no floor) claiming `src/lib.rs`, plus files the
/// inferred universe cannot see: a root `AGENTS.md` (claimed by frontmatter),
/// `scripts/run.sh` and `scripts/notes.txt` (no package, unclaimed), a
/// `docs/guide.md` under the bypass floor, a `target/gen.sh` under a resolver
/// exclusion, and `.git/hooks/pre-commit` for the walk to skip.
fn scope_fixture(coverage_toml: &str) -> (tempfile::TempDir, Config) {
    let tmp = tempfile::tempdir().unwrap();
    let r = tmp.path();
    write(r, "Cargo.toml", "[workspace]\nmembers = [\"crate-a\"]\n");
    write(
        r,
        "crate-a/Cargo.toml",
        "[package]\nname = \"crate-a\"\nversion = \"0.1.0\"\n",
    );
    write(r, "crate-a/src/lib.rs", "pub fn a() {}\n");
    write(r, "AGENTS.md", "# agents\n");
    write(r, "scripts/run.sh", "echo hi\n");
    write(r, "scripts/notes.txt", "notes\n");
    write(r, "docs/guide.md", "# guide\n");
    write(r, "target/gen.sh", "echo gen\n");
    write(r, ".git/hooks/pre-commit", "echo hook\n");
    write(
        r,
        "specs/001-a/spec.md",
        &spec(
            "001-a",
            "establishes:\n  - \"crate-a/src/lib.rs\"\n  - \"AGENTS.md\"\n",
        ),
    );
    write(r, "spec-spine.toml", coverage_toml);
    let cfg = load_config(coverage_toml).unwrap();
    emit_index(&cfg, r);
    (tmp, cfg)
}

fn cov(cfg: &Config, root: &Path, inventory: Option<&Inventory>) -> CoverageReport {
    coverage_with_inventory(cfg, root, inventory).unwrap()
}

fn supplied(paths: &[&str]) -> Inventory {
    Inventory {
        provenance: InventoryProvenance::Supplied,
        paths: paths.iter().map(|p| p.to_string()).collect(),
    }
}

/// §3.1: unset and `[]` are the same report, byte for byte, and neither carries
/// the two new members.
#[test]
fn an_empty_scope_is_the_report_spec_032_emits() {
    let (a, cfg_a) = scope_fixture("");
    let (b, cfg_b) = scope_fixture("[coverage]\ngoverned_scope = []\n");
    let ra = serde_json::to_string(&cov(&cfg_a, a.path(), None)).unwrap();
    let rb = serde_json::to_string(&cov(&cfg_b, b.path(), None)).unwrap();
    assert_eq!(ra, rb);
    assert!(
        !ra.contains("declaredScopeFiles") && !ra.contains("enumeration"),
        "{ra}"
    );
    assert_eq!(cov(&cfg_a, a.path(), None).source_files, 1);
}

/// §3.3, §3.4: a file outside every package and a file with no source extension
/// both enter; the unclaimed one is `Unowned`, never floor-only; the claimed one
/// is claimed from frontmatter.
#[test]
fn a_scope_adds_files_outside_packages_and_outside_source_exts() {
    let (fx, cfg) = scope_fixture("[coverage]\ngoverned_scope = [\"AGENTS.md\", \"scripts/*\"]\n");
    let r = cov(&cfg, fx.path(), None);
    assert_eq!(
        r.declared_scope_files.as_deref(),
        Some(
            &[
                "AGENTS.md".to_string(),
                "scripts/notes.txt".into(),
                "scripts/run.sh".into()
            ][..]
        )
    );
    assert_eq!(r.enumeration, Some(Enumeration::Walk));
    assert_eq!(r.source_files, 4);
    assert_eq!(r.claimed_files, 2, "lib.rs and AGENTS.md: {r:?}");
    assert_eq!(r.unclaimed_files, ["scripts/notes.txt", "scripts/run.sh"]);
    assert!(r.floor_only_files.is_empty());
    // Their denominator is not a package.
    assert_eq!(r.packages[0].source_files, 1);
    assert_eq!(
        classify_outside(&cfg, fx.path(), "scripts/run.sh"),
        Ownership::Unowned
    );
}

fn classify_outside(cfg: &Config, root: &Path, path: &str) -> Ownership {
    let index = spec_spine_core::load_committed_index(cfg, root).unwrap();
    classify(&index, path)
}

/// §3.2: exclusions carve out of the scope's own additions and cannot remove a
/// file the inferred universe already holds.
#[test]
fn exclusions_remove_only_what_the_scope_added() {
    let (fx, cfg) = scope_fixture(
        "[coverage]\ngoverned_scope = [\"scripts/*\", \"crate-a/src/*\"]\n\
         governed_scope_exclusions = [\"scripts/notes.txt\", \"crate-a/src/lib.rs\"]\n",
    );
    let r = cov(&cfg, fx.path(), None);
    assert_eq!(
        r.declared_scope_files.as_deref(),
        Some(&["scripts/run.sh".to_string()][..])
    );
    assert_eq!(r.source_files, 2, "lib.rs stays, via its package: {r:?}");
    assert_eq!(r.packages[0].source_files, 1);
}

/// §3.3: a resolver exclusion and a bypass still win over scope membership.
#[test]
fn a_resolver_exclusion_and_a_bypass_stay_out() {
    let (fx, cfg) = scope_fixture("[coverage]\ngoverned_scope = [\"target/*\", \"docs/*\"]\n");
    let r = cov(
        &cfg,
        fx.path(),
        Some(&supplied(&["target/gen.sh", "docs/guide.md"])),
    );
    assert_eq!(r.declared_scope_files.as_deref(), Some(&[][..]));
    assert_eq!(r.source_files, 1);
    let index = spec_spine_core::load_committed_index(&cfg, fx.path()).unwrap();
    let scope = GovernedScope::from_globs(&cfg, fx.path());
    assert!(scope.contains("docs/guide.md"), "matched by the glob");
    assert!(!in_coverage_universe_with(
        &cfg,
        &index,
        &scope,
        "docs/guide.md"
    ));
    assert!(!in_coverage_universe_with(
        &cfg,
        &index,
        &scope,
        "target/gen.sh"
    ));
}

/// §3.5: a set scope that matches nothing still reports both members, with
/// `declaredScopeFiles: []`, so the enumeration survives.
#[test]
fn a_set_scope_matching_nothing_keeps_both_members() {
    let (fx, cfg) = scope_fixture("[coverage]\ngoverned_scope = [\"nothing/*\"]\n");
    let r = cov(&cfg, fx.path(), None);
    let json = serde_json::to_string(&r).unwrap();
    assert!(json.contains("\"declaredScopeFiles\":[]"), "{json}");
    assert!(json.contains("\"enumeration\":\"walk\""), "{json}");
}

/// §3.6: absent walks; supplied-empty matches nothing; they are two different
/// reports. And a supplied list is matched exactly, carrying its provenance.
#[test]
fn absent_and_supplied_empty_inventories_are_different_answers() {
    let (fx, cfg) = scope_fixture("[coverage]\ngoverned_scope = [\"scripts/*\"]\n");
    let walked = cov(&cfg, fx.path(), None);
    let empty = cov(&cfg, fx.path(), Some(&supplied(&[])));
    assert_ne!(walked, empty);
    assert_eq!(walked.declared_scope_files.as_ref().unwrap().len(), 2);
    assert_eq!(empty.declared_scope_files.as_deref(), Some(&[][..]));
    assert_eq!(empty.enumeration, Some(Enumeration::Supplied));

    let one = cov(&cfg, fx.path(), Some(&supplied(&["./scripts/run.sh"])));
    assert_eq!(
        one.declared_scope_files.as_deref(),
        Some(&["scripts/run.sh".to_string()][..])
    );
    let tracked = Inventory {
        provenance: InventoryProvenance::Tracked,
        paths: vec!["scripts/run.sh".into()],
    };
    assert_eq!(
        cov(&cfg, fx.path(), Some(&tracked)).enumeration,
        Some(Enumeration::Tracked)
    );
}

/// §3.6: the walk skips `.git/` and the declared state root.
#[test]
fn the_walk_skips_git_and_the_state_root() {
    let (fx, _) = scope_fixture("");
    write(fx.path(), "tool-state/cache.sh", "echo state\n");
    let cfg = load_config("[layout]\nstate_dir = \"tool-state\"\n").unwrap();
    let walked = walk_repository(&cfg, fx.path());
    assert!(walked.contains("scripts/run.sh"));
    assert!(!walked.iter().any(|p| p.starts_with(".git")), "{walked:?}");
    assert!(
        !walked.iter().any(|p| p.starts_with("tool-state")),
        "{walked:?}"
    );
    assert!(
        !walked.iter().any(|p| p.starts_with("target/")),
        "{walked:?}"
    );
}
