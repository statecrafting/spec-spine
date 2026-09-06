//! Ownership coverage (spec 032): the inverse of the coupling gate's question.
//!
//! `couple` asks "did an owned path change without its spec?" (`C-001`). This
//! module asks "which source files does no spec *specifically* own?", and, when
//! `[coupling] require_ownership` is on, the gate asks the same of every changed
//! path (`C-002`). Both read ownership through one classifier ([`classify`])
//! over one universe ([`in_coverage_universe`]), so the report predicts the
//! gate exactly: a file the report calls claimed is one the gate will never
//! refuse for lack of an owner, and vice versa.
//!
//! "Specific" is the load-bearing word. A manifest floor
//! (`[package.metadata.<ns>].spec`, spec 005 §3.6) makes its spec an owner of
//! every file in the package, which is the right safety net for `C-001` and a
//! useless coverage signal: a crate with one governed file and two hundred
//! ungoverned ones would read as fully traced. The floor therefore counts as
//! ownership for drift and as **debt** for coverage.
//!
//! Pure function of `(config, index, file listing)`. The freshness-guarded
//! form [`coverage`] refuses a stale committed index (exit 2) exactly as
//! `couple` does, so the report is always read against the ledger the corpus
//! compiled to. Nothing here is committed: coverage is a read verb over the
//! tree and the ledger, like `index check`, not a field of the index.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use spec_spine_types::{
    CodebaseIndex, Config, CoverageReport, Error, PackageCoverage, PackageRecord, TraceSource,
};

use crate::couple::{claim_matches, is_bypassed_path};
use crate::index::{Freshness, check_index_freshness, load_committed_index, walk_source};
use crate::pathutil::rel_posix;

/// The source extensions the indexer treats as code: the set the comment-header
/// scan reads and the coverage universe enumerates. One list, so the coverage
/// denominator and the spec-binding scan can never disagree about what counts
/// as a source file.
pub const SOURCE_EXTS: &[&str] = &["rs", "ts", "tsx", "js", "jsx", "go", "py", "sh"];

/// How one source file is owned, at whole-file granularity (spec 032 §3.1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ownership {
    /// A resolved ownership-bearing unit (any kind, exact or subtree) or a
    /// `// Spec:` comment header covers the file: an author decided which spec
    /// governs it.
    Specific,
    /// Only a package's manifest floor (spec 005 §3.6) covers the file. Carries
    /// the floor spec ids, sorted, for the `C-002` message.
    FloorOnly(Vec<String>),
    /// No spec owns the file.
    Unowned,
}

/// Classify one path's ownership against the index. Spans are ignored: this
/// is a whole-file question, which is what keeps it answerable from a file
/// listing alone and identical between the report and the gate.
pub fn classify(index: &CodebaseIndex, path: &str) -> Ownership {
    for m in &index.traceability.mappings {
        // 1. Unit claims: ownership-bearing resolved units, exact or subtree.
        //    `references` units are non-owning and never count.
        for ru in m.resolved_units.iter().filter(|ru| ru.ownership) {
            if ru
                .locations
                .iter()
                .any(|loc| claim_matches(&loc.file, path))
            {
                return Ownership::Specific;
            }
        }
        // 2. Comment headers: a file naming its own spec. Exact match only (a
        //    header claims the file it sits in, never a subtree). `Multiple`
        //    on a file path means a header agreed with another source.
        //    `SpecEdge` paths are deliberately not read here: an owning unit
        //    was caught above, and a `references` unit's location must not
        //    become ownership through the path-level back door.
        for ip in &m.implementing_paths {
            if ip.path == path
                && matches!(
                    ip.source,
                    TraceSource::CommentHeader | TraceSource::Multiple
                )
            {
                return Ownership::Specific;
            }
        }
    }
    // 3. The manifest floor: every discovered package whose directory contains
    //    the file and whose manifest names a spec.
    let floors: BTreeSet<String> = index
        .packages
        .iter()
        .filter(|p| package_contains(&p.path, path))
        .filter_map(|p| p.spec_ref.clone())
        .collect();
    if floors.is_empty() {
        Ownership::Unowned
    } else {
        Ownership::FloorOnly(floors.into_iter().collect())
    }
}

/// Is `path` in the coverage universe: a source file (by extension) inside a
/// discovered package, outside `index.resolver_exclusions`, and not bypassed
/// by the gate (claim-aware, spec 009)? The report and `C-002` share this
/// predicate, so a path one of them ignores the other ignores too.
///
/// A declared `layout.state_dir` (spec 039) is excluded here through
/// [`is_bypassed_path`], which means excluded from the numerator **and** the
/// denominator: state is not source, so counting it as unclaimed debt would be
/// a coverage figure that can never reach 100%.
pub fn in_coverage_universe(cfg: &Config, index: &CodebaseIndex, path: &str) -> bool {
    has_source_ext(path)
        && !has_excluded_component(path, &cfg.index.resolver_exclusions)
        && index
            .packages
            .iter()
            .any(|p| package_contains(&p.path, path))
        && !is_bypassed_path(cfg, index, path)
}

/// Enumerate the coverage universe under `repo_root`: the source files of
/// every discovered package, walked in sorted order with `resolver_exclusions`
/// pruned, filtered by [`in_coverage_universe`], and deduplicated (a nested
/// package is walked by its parent too). Repo-relative POSIX, sorted.
pub fn enumerate_source_files(
    cfg: &Config,
    repo_root: &Path,
    index: &CodebaseIndex,
) -> Vec<String> {
    let mut files: BTreeSet<String> = BTreeSet::new();
    for pkg in &index.packages {
        let dir = repo_root.join(&pkg.path);
        for file in walk_source(
            &dir,
            SOURCE_EXTS,
            repo_root,
            &cfg.index.resolver_exclusions,
            &cfg.layout,
        ) {
            let rel = rel_posix(repo_root, &file);
            if in_coverage_universe(cfg, index, &rel) {
                files.insert(rel);
            }
        }
    }
    files.into_iter().collect()
}

/// The pure report over an already-loaded index and file listing (overlays,
/// tests). Paths outside the universe are ignored; the listing is sorted and
/// deduplicated before classification, so the output is a pure function of the
/// *set* of paths. Each file is attributed to the deepest package containing
/// it.
pub fn coverage_with(cfg: &Config, index: &CodebaseIndex, files: &[String]) -> CoverageReport {
    let mut per_package: BTreeMap<String, PackageCoverage> = index
        .packages
        .iter()
        .map(|p| {
            (
                p.path.clone(),
                PackageCoverage {
                    path: p.path.clone(),
                    floor_spec: p.spec_ref.clone(),
                    source_files: 0,
                    claimed_files: 0,
                    floor_only: 0,
                    unclaimed: 0,
                },
            )
        })
        .collect();
    let universe: BTreeSet<&String> = files
        .iter()
        .filter(|f| in_coverage_universe(cfg, index, f))
        .collect();

    let mut report = CoverageReport {
        source_files: 0,
        claimed_files: 0,
        floor_only_files: Vec::new(),
        unclaimed_files: Vec::new(),
        packages: Vec::new(),
    };
    for file in universe {
        let Some(entry) =
            owning_package(&index.packages, file).and_then(|p| per_package.get_mut(&p.path))
        else {
            continue; // unreachable: the universe requires a containing package
        };
        entry.source_files += 1;
        report.source_files += 1;
        match classify(index, file) {
            Ownership::Specific => {
                entry.claimed_files += 1;
                report.claimed_files += 1;
            }
            Ownership::FloorOnly(_) => {
                entry.floor_only += 1;
                report.floor_only_files.push(file.clone());
            }
            Ownership::Unowned => {
                entry.unclaimed += 1;
                report.unclaimed_files.push(file.clone());
            }
        }
    }
    report.packages = per_package.into_values().collect();
    report
}

/// The freshness-guarded report: refuses a stale committed index
/// ([`Error::Stale`], exit 2) exactly as `couple` does, loads the committed
/// shard set, enumerates the universe under `repo_root`, and classifies it.
pub fn coverage(cfg: &Config, repo_root: &Path) -> Result<CoverageReport, Error> {
    match check_index_freshness(cfg, repo_root)? {
        Freshness::Stale { expected, actual } => return Err(Error::Stale { expected, actual }),
        Freshness::Fresh => {}
    }
    let index = load_committed_index(cfg, repo_root)?;
    let files = enumerate_source_files(cfg, repo_root, &index);
    Ok(coverage_with(cfg, &index, &files))
}

/// Source-extension test on a repo-relative POSIX path, with the same
/// `Path::extension` semantics the walk uses (a dotfile has no extension).
fn has_source_ext(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| SOURCE_EXTS.contains(&e))
        .unwrap_or(false)
}

/// True if any `/`-separated component of `path` is an excluded directory
/// name (the string-path twin of `pathutil::is_excluded`).
fn has_excluded_component(path: &str, exclusions: &[String]) -> bool {
    path.split('/')
        .any(|seg| exclusions.iter().any(|ex| ex == seg))
}

/// Does the package rooted at `pkg_path` contain `path`? A root package
/// (`""` or `.`) contains everything; otherwise slash-anchored prefix.
fn package_contains(pkg_path: &str, path: &str) -> bool {
    let pkg = pkg_path.trim_end_matches('/');
    pkg.is_empty() || pkg == "." || path.starts_with(&format!("{pkg}/"))
}

/// The deepest discovered package containing `path` (a nested package wins
/// over the workspace root that also contains it).
fn owning_package<'a>(packages: &'a [PackageRecord], path: &str) -> Option<&'a PackageRecord> {
    packages
        .iter()
        .filter(|p| package_contains(&p.path, path))
        .max_by_key(|p| p.path.trim_end_matches('/').len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_ext_follows_path_extension_semantics() {
        assert!(has_source_ext("crates/x/src/lib.rs"));
        assert!(has_source_ext("scripts/run.sh"));
        assert!(!has_source_ext("README.md"));
        assert!(!has_source_ext("Cargo.toml"));
        assert!(!has_source_ext(".rs"), "a dotfile has no extension");
        assert!(!has_source_ext("noext"));
    }

    #[test]
    fn package_containment_is_slash_anchored() {
        assert!(package_contains("", "src/lib.rs"));
        assert!(package_contains(".", "src/lib.rs"));
        assert!(package_contains("crates/x", "crates/x/src/lib.rs"));
        assert!(package_contains("crates/x/", "crates/x/src/lib.rs"));
        assert!(!package_contains("crates/x", "crates/xy/src/lib.rs"));
    }

    #[test]
    fn deepest_package_wins() {
        let pkgs = vec![
            PackageRecord {
                name: "root".into(),
                path: "".into(),
                kind: spec_spine_types::PackageKind::RustLib,
                version: None,
                edition: None,
                spec_ref: None,
            },
            PackageRecord {
                name: "inner".into(),
                path: "crates/inner".into(),
                kind: spec_spine_types::PackageKind::RustLib,
                version: None,
                edition: None,
                spec_ref: None,
            },
        ];
        assert_eq!(
            owning_package(&pkgs, "crates/inner/src/lib.rs").map(|p| p.name.as_str()),
            Some("inner")
        );
        assert_eq!(
            owning_package(&pkgs, "src/main.rs").map(|p| p.name.as_str()),
            Some("root")
        );
    }
}
