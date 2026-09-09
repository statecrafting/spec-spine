//! Package discovery (spec 004 §3.1): the configurable manifest scan.
//!
//! Rust crates come from the root Cargo workspace members plus
//! `layout.standalone_rust_workspaces`; npm/pnpm packages from the workspace
//! globs declared by `layout.npm_workspaces` (the default reads root
//! `package.json#workspaces`, the template-encore fix) plus
//! `layout.standalone_npm_packages`. The owning spec is read from the configurable
//! `manifest.metadata_namespace`. Everything is path-sorted for determinism and
//! `index.resolver_exclusions` directories are never descended.

use std::fs;
use std::path::{Path, PathBuf};

use spec_spine_types::{Config, Diagnostic, PackageKind, PackageRecord};

use crate::pathutil::{is_excluded, rel_posix};

/// Discovered packages plus the manifest paths to fold into the content hash.
pub struct Discovered {
    pub packages: Vec<PackageRecord>,
    pub manifest_paths: Vec<PathBuf>,
    pub diagnostics: Vec<Diagnostic>,
}

/// Discover all Rust and npm packages under `repo_root`.
pub fn discover(cfg: &Config, repo_root: &Path) -> Discovered {
    let mut packages = Vec::new();
    let mut manifest_paths = Vec::new();
    let mut diagnostics = Vec::new();

    discover_rust(
        cfg,
        repo_root,
        &mut packages,
        &mut manifest_paths,
        &mut diagnostics,
    );
    discover_npm(
        cfg,
        repo_root,
        &mut packages,
        &mut manifest_paths,
        &mut diagnostics,
    );

    packages.sort_by(|a, b| a.path.cmp(&b.path));
    packages.dedup_by(|a, b| a.path == b.path);
    manifest_paths.sort();
    manifest_paths.dedup();
    Discovered {
        packages,
        manifest_paths,
        diagnostics,
    }
}

// ===== Rust =====

fn discover_rust(
    cfg: &Config,
    repo_root: &Path,
    packages: &mut Vec<PackageRecord>,
    manifests: &mut Vec<PathBuf>,
    diags: &mut Vec<Diagnostic>,
) {
    let root_manifest = repo_root.join(&cfg.layout.cargo_workspace);
    let mut members: Vec<String> = Vec::new();
    if let Ok(src) = fs::read_to_string(&root_manifest) {
        manifests.push(root_manifest.clone());
        if let Ok(doc) = toml::from_str::<toml::Value>(&src) {
            // A root [package], if present, is itself a crate.
            if doc.get("package").is_some() {
                if let Some(rec) = parse_cargo(
                    repo_root,
                    &root_manifest,
                    &cfg.manifest.metadata_namespace,
                    diags,
                ) {
                    packages.push(rec);
                }
            }
            if let Some(arr) = doc
                .get("workspace")
                .and_then(|w| w.get("members"))
                .and_then(|m| m.as_array())
            {
                members.extend(arr.iter().filter_map(|v| v.as_str().map(String::from)));
            }
        }
    }
    members.extend(cfg.layout.standalone_rust_workspaces.iter().cloned());

    for member in &members {
        // A standalone entry may name a directory or a `Cargo.toml` path; strip a
        // trailing manifest filename so glob_manifests appends it exactly once
        // (spec 026 FR-005: a `.../Cargo.toml` entry must not double-join).
        let member = member
            .strip_suffix("Cargo.toml")
            .map(|s| s.trim_end_matches('/'))
            .unwrap_or(member.as_str());
        for manifest in glob_manifests(
            repo_root,
            member,
            "Cargo.toml",
            &cfg.index.resolver_exclusions,
        ) {
            manifests.push(manifest.clone());
            if let Some(rec) = parse_cargo(
                repo_root,
                &manifest,
                &cfg.manifest.metadata_namespace,
                diags,
            ) {
                packages.push(rec);
            }
        }
    }
}

fn parse_cargo(
    repo_root: &Path,
    manifest: &Path,
    namespace: &str,
    diags: &mut Vec<Diagnostic>,
) -> Option<PackageRecord> {
    let src = fs::read_to_string(manifest).ok()?;
    let doc = match toml::from_str::<toml::Value>(&src) {
        Ok(d) => d,
        Err(e) => {
            diags.push(diag(
                "I-001",
                format!("cannot parse {}: {e}", manifest.display()),
                repo_root,
                manifest,
            ));
            return None;
        }
    };
    let pkg = doc.get("package")?;
    let name = pkg.get("name").and_then(|v| v.as_str())?.to_string();
    let dir = manifest.parent().unwrap_or(repo_root);

    let has_lib = doc.get("lib").is_some() || dir.join("src/lib.rs").is_file();
    let has_bin = doc.get("bin").is_some() || dir.join("src/main.rs").is_file();
    let kind = match (has_lib, has_bin) {
        (true, true) => PackageKind::RustLibBin,
        (false, true) => PackageKind::RustBin,
        _ => PackageKind::RustLib,
    };

    Some(PackageRecord {
        name,
        path: rel_posix(repo_root, dir),
        kind,
        version: pkg
            .get("version")
            .and_then(|v| v.as_str())
            .map(String::from),
        edition: pkg
            .get("edition")
            .and_then(|v| v.as_str())
            .map(String::from),
        spec_ref: pkg
            .get("metadata")
            .and_then(|m| m.get(namespace))
            .and_then(|n| n.get("spec"))
            .and_then(|s| s.as_str())
            .map(String::from),
    })
}

// ===== npm / pnpm =====

fn discover_npm(
    cfg: &Config,
    repo_root: &Path,
    packages: &mut Vec<PackageRecord>,
    manifests: &mut Vec<PathBuf>,
    diags: &mut Vec<Diagnostic>,
) {
    let mut globs: Vec<String> = Vec::new();

    for decl in &cfg.layout.npm_workspaces {
        let decl_path = repo_root.join(decl);
        if !decl_path.is_file() {
            continue;
        }
        let Ok(src) = fs::read_to_string(&decl_path) else {
            continue;
        };
        if decl.ends_with(".json") {
            // package.json: a `workspaces` array, or `{ "workspaces": { "packages": [...] } }`.
            if let Ok(doc) = serde_json::from_str::<serde_json::Value>(&src) {
                manifests.push(decl_path.clone());
                let ws = doc.get("workspaces");
                if let Some(arr) = ws.and_then(|w| w.as_array()) {
                    globs.extend(
                        arr.iter()
                            .filter_map(|v| v.as_str())
                            .map(|g| rebase_glob(decl, g)),
                    );
                } else if let Some(arr) = ws
                    .and_then(|w| w.get("packages"))
                    .and_then(|p| p.as_array())
                {
                    globs.extend(
                        arr.iter()
                            .filter_map(|v| v.as_str())
                            .map(|g| rebase_glob(decl, g)),
                    );
                }
                // The root package.json that declares workspaces is itself a record.
                if doc.get("name").is_some() {
                    if let Some(rec) =
                        npm_record(repo_root, &decl_path, &cfg.manifest.metadata_namespace)
                    {
                        packages.push(rec);
                    }
                }
            }
        } else {
            // pnpm-workspace.yaml: a `packages` list.
            if let Ok(doc) = serde_yaml::from_str::<serde_yaml::Value>(&src) {
                manifests.push(decl_path.clone());
                if let Some(arr) = doc.get("packages").and_then(|p| p.as_sequence()) {
                    globs.extend(
                        arr.iter()
                            .filter_map(|v| v.as_str())
                            .map(|g| rebase_glob(decl, g)),
                    );
                }
            }
        }
    }

    for member in globs
        .iter()
        .chain(cfg.layout.standalone_npm_packages.iter())
    {
        for manifest in glob_manifests(
            repo_root,
            member,
            "package.json",
            &cfg.index.resolver_exclusions,
        ) {
            manifests.push(manifest.clone());
            if let Some(rec) = npm_record(repo_root, &manifest, &cfg.manifest.metadata_namespace) {
                packages.push(rec);
            } else {
                diags.push(diag(
                    "I-002",
                    "cannot parse package.json".into(),
                    repo_root,
                    &manifest,
                ));
            }
        }
    }
}

fn npm_record(repo_root: &Path, manifest: &Path, namespace: &str) -> Option<PackageRecord> {
    let src = fs::read_to_string(manifest).ok()?;
    let doc = serde_json::from_str::<serde_json::Value>(&src).ok()?;
    let name = doc.get("name").and_then(|v| v.as_str())?.to_string();
    let dir = manifest.parent().unwrap_or(repo_root);
    let kind = if doc.get("workspaces").is_some() {
        PackageKind::NpmWorkspace
    } else {
        PackageKind::NpmPackage
    };
    Some(PackageRecord {
        name,
        path: rel_posix(repo_root, dir),
        kind,
        version: doc
            .get("version")
            .and_then(|v| v.as_str())
            .map(String::from),
        edition: None,
        spec_ref: doc
            .get(namespace)
            .and_then(|n| n.get("spec"))
            .and_then(|s| s.as_str())
            .map(String::from),
    })
}

/// The governance projection of an npm manifest: exactly the fields
/// discovery consumes (`npm_record` + workspace-glob extraction): `name`,
/// `version`, `workspaces`, and the adopter's metadata-namespace object.
/// The index content hash folds this INSTEAD of the raw bytes (spec 004
/// §3.5 amendment, 2026-06-11): a dependency-table version bump
/// (dependabot-class) is not a governed input and must not stale the
/// committed index, while any change to a field the indexer actually reads
/// still does. `None` (unparseable / non-object) tells the caller to fall
/// back to raw bytes: over-hashing is the fail-closed direction.
pub fn npm_hash_projection(content: &str, namespace: &str) -> Option<String> {
    let doc: serde_json::Value = serde_json::from_str(content).ok()?;
    let obj = doc.as_object()?;
    let mut proj = serde_json::Map::new();
    for key in ["name", "version", "workspaces"] {
        if let Some(v) = obj.get(key) {
            proj.insert(key.to_string(), v.clone());
        }
    }
    if let Some(v) = obj.get(namespace) {
        proj.insert(namespace.to_string(), v.clone());
    }
    serde_json::to_string(&serde_json::Value::Object(proj)).ok()
}

/// Cargo dependency tables stripped from the governance projection: a
/// dependabot-class version bump lives entirely inside these (at the top level,
/// under `[workspace]`, or under `[target.<cfg>]`). Mirrors
/// `dep_only::CARGO_DEPENDENCY_TABLES`.
const CARGO_DEP_TABLES: &[&str] = &["dependencies", "dev-dependencies", "build-dependencies"];

/// The governance projection of a Cargo manifest (spec 030, extending the
/// 2026-06-11 npm projection to the cargo ecosystem). The manifest with its
/// dependency tables removed, rendered as canonical JSON so the hash is
/// deterministic (the same sorted-key path the npm projection folds through).
/// A dependabot-class version bump changes only a stripped table, so it leaves
/// the projection (and the shard hash) unchanged, while any other edit (name,
/// version, edition, `[package.metadata.<ns>]`, a `[lib]` / `[bin]` presence
/// flip that would move the package kind, a feature-flag change, a new table)
/// still stales it. `None` (unparseable / not a table) tells the caller to fall
/// back to raw bytes: over-hashing is the fail-closed direction.
pub fn cargo_hash_projection(content: &str) -> Option<String> {
    let mut doc: toml::Value = toml::from_str(content).ok()?;
    let table = doc.as_table_mut()?;
    strip_cargo_dep_tables(table);
    serde_json::to_string(&toml_to_json(&doc)).ok()
}

/// The governance projection of a GitHub Actions workflow (spec 073, extending
/// the npm and cargo projections to the third ecosystem whose bumps arrive by
/// bot). The parsed document with the pinned ref of every `uses:` reference
/// removed and the action path kept, rendered as canonical JSON so the hash is
/// deterministic across the release matrix (the same sorted-key path the npm
/// and cargo projections fold through).
///
/// A Dependabot `uses:` bump therefore leaves the projection, and so every
/// shard hash, unchanged; a changed action path, an unpin, a `run:` / `with:` /
/// `env:` / `if:` edit, an added step or job, and a changed trigger all still
/// stale it. `None` (unparseable / non-mapping) tells the caller to fall back
/// to raw bytes: over-hashing is the fail-closed direction, and a workflow the
/// parser rejects stales on every edit rather than silently on none.
pub fn workflow_hash_projection(content: &str) -> Option<String> {
    let mut doc: serde_yaml::Value = serde_yaml::from_str(content).ok()?;
    doc.as_mapping()?;
    strip_uses_refs(&mut doc);
    serde_json::to_string(&yaml_to_json(&doc)).ok()
}

/// Remove the pinned ref of every `uses:` scalar, at any nesting depth.
///
/// A `uses:` whose value is not a string is left alone entirely, and not
/// descended into: `dep_only::uses_ref_only_differs` requires exact equality
/// there, and the two rules have to agree case for case (spec 073 3.2).
fn strip_uses_refs(value: &mut serde_yaml::Value) {
    use serde_yaml::Value::{Mapping, Sequence};
    match value {
        Mapping(m) => {
            let keys: Vec<serde_yaml::Value> = m.keys().cloned().collect();
            for k in keys {
                let is_uses = k.as_str() == Some("uses");
                let Some(v) = m.get_mut(&k) else { continue };
                match (is_uses, projected_uses(v)) {
                    (true, Some(stripped)) => *v = serde_yaml::Value::String(stripped),
                    (true, None) => {}
                    (false, _) => strip_uses_refs(v),
                }
            }
        }
        Sequence(s) => s.iter_mut().for_each(strip_uses_refs),
        _ => {}
    }
}

/// `owner/action@<ref>` projected to `owner/action@`. Anything else yields
/// `None`, meaning preserved verbatim: a non-string value, an unpinned or
/// local reference (`./path`, `docker://image`), and an empty action path.
///
/// **The action path is kept**, subpath included, because swapping which
/// action runs is a governed change: dropping the whole `uses:` value would
/// make replacing `actions/checkout` with a fork invisible to the ledger, and
/// spec 030's waiver already draws the line in the same place.
///
/// **The `@` is kept as a marker**, so a pinned reference never projects onto
/// the unpinned spelling of the same action. `a/b@v4` folds to `a/b@` while
/// `a/b` stays `a/b`, so an unpin moves the hash. Projecting to a bare
/// `owner/action`, which is how 073 3.1 words the rule, would collide the two
/// and make unpinning invisible, contradicting the same spec's 3.5 and its
/// summary; the marker is the reading that satisfies all three.
fn projected_uses(value: &serde_yaml::Value) -> Option<String> {
    let s = value.as_str()?;
    let (path, _) = s.split_once('@')?;
    if path.is_empty() {
        // `@v1` has no action to preserve. `uses_ref_only_differs` refuses the
        // waiver on it (`!ba.is_empty()`), so preserving it verbatim is what
        // keeps the two rules in agreement in both directions.
        return None;
    }
    Some(format!("{path}@"))
}

/// Deterministic YAML to JSON, so a workflow projection hashes through the same
/// canonical (sorted-key) serializer the npm and cargo projections use.
///
/// Mapping keys are tagged by kind, so a key that differs in the document
/// cannot collide here. YAML 1.2 reads `on` as a string rather than a boolean,
/// which is the case that would otherwise matter most in a workflow.
fn yaml_to_json(value: &serde_yaml::Value) -> serde_json::Value {
    use serde_yaml::Value as Y;
    match value {
        Y::Null => serde_json::Value::Null,
        Y::Bool(b) => serde_json::Value::Bool(*b),
        Y::Number(n) => n
            .as_i64()
            .map(serde_json::Value::from)
            .or_else(|| n.as_u64().map(serde_json::Value::from))
            .or_else(|| {
                n.as_f64()
                    .and_then(serde_json::Number::from_f64)
                    .map(serde_json::Value::Number)
            })
            // A non-finite float has no JSON spelling; cargo manifests fold the
            // same way (`toml_to_json`), and a workflow carries none.
            .unwrap_or(serde_json::Value::Null),
        Y::String(s) => serde_json::Value::String(s.clone()),
        Y::Sequence(s) => serde_json::Value::Array(s.iter().map(yaml_to_json).collect()),
        Y::Mapping(m) => serde_json::Value::Object(
            m.iter()
                .map(|(k, v)| (yaml_key(k), yaml_to_json(v)))
                .collect(),
        ),
        Y::Tagged(t) => serde_json::Value::Object(
            [
                (
                    "!tag".to_string(),
                    serde_json::Value::String(t.tag.to_string()),
                ),
                ("!value".to_string(), yaml_to_json(&t.value)),
            ]
            .into_iter()
            .collect(),
        ),
    }
}

/// A mapping key as a JSON object key, prefixed by kind so the rendering is
/// injective: a string key `1` and a number key `1` must not fold together.
fn yaml_key(k: &serde_yaml::Value) -> String {
    match k {
        serde_yaml::Value::String(s) => format!("s:{s}"),
        other => format!(
            "y:{}",
            serde_json::to_string(&yaml_to_json(other)).unwrap_or_default()
        ),
    }
}

/// Remove the cargo dependency tables at every location cargo allows them.
fn strip_cargo_dep_tables(table: &mut toml::Table) {
    for key in CARGO_DEP_TABLES {
        table.remove(*key);
    }
    if let Some(toml::Value::Table(ws)) = table.get_mut("workspace") {
        ws.remove("dependencies");
    }
    if let Some(toml::Value::Table(targets)) = table.get_mut("target") {
        for (_, cfg) in targets.iter_mut() {
            if let toml::Value::Table(cfg) = cfg {
                for key in CARGO_DEP_TABLES {
                    cfg.remove(*key);
                }
            }
        }
    }
}

/// Deterministic TOML → JSON, so the cargo projection hashes through the same
/// canonical (sorted-key) serializer the npm projection uses. `Datetime` folds
/// to its string form; a non-finite float folds to null (cargo manifests carry
/// none of either in practice).
fn toml_to_json(value: &toml::Value) -> serde_json::Value {
    use serde_json::Value as J;
    match value {
        toml::Value::String(s) => J::String(s.clone()),
        toml::Value::Integer(i) => J::Number((*i).into()),
        toml::Value::Float(f) => serde_json::Number::from_f64(*f).map_or(J::Null, J::Number),
        toml::Value::Boolean(b) => J::Bool(*b),
        toml::Value::Datetime(d) => J::String(d.to_string()),
        toml::Value::Array(a) => J::Array(a.iter().map(toml_to_json).collect()),
        toml::Value::Table(t) => J::Object(
            t.iter()
                .map(|(k, v)| (k.clone(), toml_to_json(v)))
                .collect(),
        ),
    }
}

// ===== shared =====

/// Rebase a workspace-member glob declared in `decl` (a repo-relative declaration
/// file path) onto that file's parent directory, so a NON-root workspace file
/// (e.g. `product/pnpm-workspace.yaml` declaring `apps/*`) resolves its members
/// relative to itself, not the repo root (spec 026 D3). A root-level declaration
/// (no parent) returns the glob unchanged. `standalone_npm_packages` are
/// repo-root-relative by contract and are deliberately NOT routed through here.
fn rebase_glob(decl: &str, glob: &str) -> String {
    match Path::new(decl).parent() {
        Some(parent) if !parent.as_os_str().is_empty() => {
            format!("{}/{}", parent.to_string_lossy(), glob)
        }
        _ => glob.to_string(),
    }
}

/// Expand `<member>/<manifest_file>` under `repo_root` (member may contain glob
/// metacharacters), returning matches not inside an excluded directory, sorted.
fn glob_manifests(
    repo_root: &Path,
    member: &str,
    manifest_file: &str,
    exclusions: &[String],
) -> Vec<PathBuf> {
    let pattern = repo_root.join(member).join(manifest_file);
    let pattern = pattern.to_string_lossy();
    let mut out: Vec<PathBuf> = match glob::glob(&pattern) {
        Ok(paths) => paths
            .filter_map(std::result::Result::ok)
            .filter(|p| !is_excluded(repo_root, p, exclusions))
            .collect(),
        Err(_) => Vec::new(),
    };
    out.sort();
    out.dedup();
    out
}

fn diag(code: &str, message: String, repo_root: &Path, path: &Path) -> Diagnostic {
    Diagnostic {
        code: code.to_string(),
        message,
        path: Some(rel_posix(repo_root, path)),
    }
}
