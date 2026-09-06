//! Index integration tests: determinism, conformance, manifest + npm discovery
//! (the encore fix), file/section/symbol resolution with per-platform span
//! goldens, staleness, and authorities.

use std::fs;
use std::path::Path;

use spec_spine_core::{authorities, index, index_shard_files};
// Freshness / shard-emit helpers are exercised only by the staleness tests, which
// are gated on `symbol-resolution` (spec 027): their `mixed_fixture` declares
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
/// per-spec/per-package shard tree (spec 024), not a monolithic `index.json`.
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

/// Like [`spec`] but with an explicit lifecycle `status` (spec 025 fixtures need
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
        .find(|u| matches!(&u.unit, Unit::Symbol { id } if id == sym_id))
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

// ===== spec 026: resolution + discovery fixes =====

/// AC-1 (spec 026 D1): a section unit on a foreign (non-workflow) YAML resolves
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

/// AC-4 (spec 026 D3): a non-root pnpm-workspace.yaml resolves its member globs
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

// ===== spec 025: lifecycle- and edge-aware unresolved-unit severity =====

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
/// unresolved owning unit is `W-001` (spec 025 §3.1 arm 2 keys on either signal).
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
/// hard `I-004` error, unchanged by spec 025 (the complement of AC-2 / AC-3).
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

// Gated on `symbol-resolution` (spec 027): `mixed_fixture` declares symbol units
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

// ===== spec 017: crate / directory / module unit kinds =====

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
                id: "rs-thing".into()
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
        .find(|u| matches!(&u.unit, Unit::Module { id } if id == "rs_thing::tests"))
        .unwrap();
    assert_eq!(tests_unit.locations[0].file, "rs-thing/src/lib.rs");
    assert!(
        tests_unit.locations[0].span.is_some(),
        "inline mod resolves to a block span"
    );
    let helper_unit = m
        .resolved_units
        .iter()
        .find(|u| matches!(&u.unit, Unit::Module { id } if id == "rs_thing::helper"))
        .unwrap();
    assert_eq!(helper_unit.locations[0].file, "rs-thing/src/helper.rs");
    assert_eq!(
        helper_unit.locations[0].span, None,
        "a file-module resolves whole-file"
    );
}

#[test]
fn spec_scoped_constrains_produces_no_resolved_unit() {
    // Spec 018: a constrains item with target_specs and no unit claims no code
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

// ===== spec 034: `references` is non-owning, so it seeds no implementing path =====

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
        .find(|u| matches!(&u.unit, Unit::File { path } if path == "src/cited.rs"))
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

// ===== spec 041: `implementation: complete` defeats draft leniency =====

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

/// Specs 041 and 044: the lifecycle fields are read as what they say. A spec
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
        // Spec 044: the one cell that spec moves. `pending` and `in-progress`
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

/// Spec 041 3.5: this touches the lifecycle arm only, never edge authority. An
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

/// Spec 044 3.2: leniency is not a pass. A spec at `in-progress` whose units
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
