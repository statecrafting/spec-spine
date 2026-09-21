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
    CodebaseIndex, Config, CoverageReport, Enumeration, Error, Inventory, PackageCoverage,
    PackageRecord, TraceSource,
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
    in_coverage_universe_with(cfg, index, &GovernedScope::empty(), path)
}

/// [`in_coverage_universe`] widened by a declared governed scope (spec 097
/// §3.3): a path the scope names bypasses the extension and package conjuncts,
/// and still answers to `resolver_exclusions` and the bypass verdict, spec
/// 009's claim precedence included. With an empty scope this is exactly the
/// four-conjunct universe spec 032 built.
pub fn in_coverage_universe_with(
    cfg: &Config,
    index: &CodebaseIndex,
    scope: &GovernedScope,
    path: &str,
) -> bool {
    (inferred_universe(index, path) || scope.contains(path))
        && !has_excluded_component(path, &cfg.index.resolver_exclusions)
        && !is_bypassed_path(cfg, index, path)
}

/// The two conjuncts a declared scope replaces: a source extension, inside a
/// discovered package.
fn inferred_universe(index: &CodebaseIndex, path: &str) -> bool {
    has_source_ext(path)
        && index
            .packages
            .iter()
            .any(|p| package_contains(&p.path, path))
}

/// The files `[coverage] governed_scope` names, resolved once (spec 097).
///
/// Membership is decided by globbing the declared patterns against the tree,
/// with exactly the matcher `[index] extra_hashed_inputs` uses, so the two lists
/// cannot disagree about what a pattern means; `governed_scope_exclusions` is
/// then subtracted. Only the scope's own additions are subtracted: a file in the
/// universe for another reason is untouched, since this set is only ever
/// unioned into the inferred universe.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GovernedScope {
    files: BTreeSet<String>,
}

impl GovernedScope {
    /// The empty scope: the universe spec 032 built, unchanged.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Every existing file the declared scope matches, minus its exclusions.
    /// What the coupling gate needs: every path it judges is in the diff, so
    /// no enumeration can remove one (spec 097 D-7).
    pub fn from_globs(cfg: &Config, repo_root: &Path) -> Self {
        let matched = |patterns: &[String]| -> BTreeSet<String> {
            patterns
                .iter()
                .flat_map(|p| crate::shard::glob_files(repo_root, p))
                .filter(|f| f.is_file())
                .map(|f| rel_posix(repo_root, &f))
                .collect()
        };
        let excluded = matched(&cfg.coverage.governed_scope_exclusions);
        GovernedScope {
            files: matched(&cfg.coverage.governed_scope)
                .into_iter()
                .filter(|f| !excluded.contains(f))
                .collect(),
        }
    }

    /// The declared scope restricted to an inventory: a file must be matched
    /// **and** listed (spec 097 §3.6). An empty inventory therefore matches
    /// nothing, which is the answer its caller gave.
    pub fn within(cfg: &Config, repo_root: &Path, inventory: &BTreeSet<String>) -> Self {
        let mut scope = Self::from_globs(cfg, repo_root);
        scope.files.retain(|f| inventory.contains(f));
        scope
    }

    pub fn contains(&self, path: &str) -> bool {
        self.files.contains(path)
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// The matched files, sorted.
    pub fn files(&self) -> impl Iterator<Item = &String> {
        self.files.iter()
    }
}

/// Every file under `repo_root`, for a caller that supplied no inventory (spec
/// 097 §3.6). Skips `.git/` (not corpus, and machine-specific), the declared
/// state root (bypassed unconditionally by spec 039), the configured derived
/// root (compiler output, spec 120 §3.8), and `resolver_exclusions`; never
/// descends through a symlink, so it cannot leave the repository.
/// Repo-relative POSIX, sorted.
pub fn walk_repository(cfg: &Config, repo_root: &Path) -> BTreeSet<String> {
    fn visit(cfg: &Config, repo_root: &Path, dir: &Path, out: &mut BTreeSet<String>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let rel = rel_posix(repo_root, &path);
            if rel == ".git"
                || rel.starts_with(".git/")
                || cfg.layout.is_state_path(&rel)
                || cfg.layout.is_derived_path(&rel)
                || crate::pathutil::is_excluded(repo_root, &path, &cfg.index.resolver_exclusions)
            {
                continue;
            }
            let Ok(meta) = std::fs::symlink_metadata(&path) else {
                continue;
            };
            if meta.is_dir() {
                visit(cfg, repo_root, &path, out);
            } else {
                out.insert(rel);
            }
        }
    }
    let mut out = BTreeSet::new();
    visit(cfg, repo_root, repo_root, &mut out);
    out
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
    coverage_with_scope(cfg, index, files, &GovernedScope::empty())
}

/// [`coverage_with`] over a universe widened by `scope` (spec 097 §3.5). A file
/// in the universe only because the scope names it counts toward the totals
/// and toward no package, since its denominator is not a package; it is
/// classified exactly as any other file, so outside every package it is
/// `Specific` or `Unowned`, never `FloorOnly`. The report members naming those
/// files are the caller's to set, because only the caller knows whether a scope
/// is configured and which enumeration produced it.
pub fn coverage_with_scope(
    cfg: &Config,
    index: &CodebaseIndex,
    files: &[String],
    scope: &GovernedScope,
) -> CoverageReport {
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
        .chain(scope.files())
        .filter(|f| in_coverage_universe_with(cfg, index, scope, f))
        .collect();

    let mut report = CoverageReport {
        source_files: 0,
        claimed_files: 0,
        floor_only_files: Vec::new(),
        unclaimed_files: Vec::new(),
        packages: Vec::new(),
        planned_territory: Vec::new(),
        near_miss_headers: Vec::new(),
        declared_scope_files: None,
        enumeration: None,
    };
    for file in universe {
        // A file the inferred universe holds belongs to its package; one only
        // the declared scope holds belongs to none (spec 097 §3.5).
        let mut entry = if inferred_universe(index, file) {
            owning_package(&index.packages, file).and_then(|p| per_package.get_mut(&p.path))
        } else {
            None
        };
        report.source_files += 1;
        if let Some(e) = entry.as_deref_mut() {
            e.source_files += 1;
        }
        match classify(index, file) {
            Ownership::Specific => {
                report.claimed_files += 1;
                if let Some(e) = entry {
                    e.claimed_files += 1;
                }
            }
            Ownership::FloorOnly(_) => {
                report.floor_only_files.push(file.clone());
                if let Some(e) = entry {
                    e.floor_only += 1;
                }
            }
            Ownership::Unowned => {
                report.unclaimed_files.push(file.clone());
                if let Some(e) = entry {
                    e.unclaimed += 1;
                }
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
    coverage_with_inventory(cfg, repo_root, None)
}

/// [`coverage`] with the inventory a declared governed scope is matched against
/// (spec 097 §3.6). The three cases are three different answers:
///
/// - `Some` with paths: the scope matches exactly those, and the report names
///   the provenance the caller declared;
/// - `Some` with no paths: the scope matches nothing, because the caller said
///   there is nothing to govern;
/// - `None`: the repository root is walked ([`walk_repository`]) and the report
///   says `walk`.
///
/// With `[coverage] governed_scope` empty the inventory is not consulted, the
/// root is not walked, and the report is the one spec 032 emits.
pub fn coverage_with_inventory(
    cfg: &Config,
    repo_root: &Path,
    inventory: Option<&Inventory>,
) -> Result<CoverageReport, Error> {
    match check_index_freshness(cfg, repo_root)? {
        Freshness::Stale { expected, actual } => return Err(Error::Stale { expected, actual }),
        Freshness::Fresh => {}
    }
    let index = load_committed_index(cfg, repo_root)?;
    let files = enumerate_source_files(cfg, repo_root, &index);
    let mut report = if cfg.coverage.governed_scope.is_empty() {
        coverage_with(cfg, &index, &files)
    } else {
        let (listed, enumeration) = match inventory {
            Some(inv) => (
                inv.paths.iter().map(|p| normalize_listed(p)).collect(),
                inv.provenance.enumeration(),
            ),
            None => (walk_repository(cfg, repo_root), Enumeration::Walk),
        };
        let scope = GovernedScope::within(cfg, repo_root, &listed);
        let mut report = coverage_with_scope(cfg, &index, &files, &scope);
        report.declared_scope_files = Some(
            scope
                .files()
                .filter(|f| {
                    !inferred_universe(&index, f)
                        && in_coverage_universe_with(cfg, &index, &scope, f)
                })
                .cloned()
                .collect(),
        );
        report.enumeration = Some(enumeration);
        report
    };
    // Spec 076 §3.6: planned territory is a DECLARED state, so it is read from
    // the spec-as-source view. It cannot come from the index: a planned unit
    // that has not resolved contributes no `ResolvedUnit` and no location, by
    // §3.2, which is exactly why the report was blind to it.
    //
    // A registry that will not load leaves the list empty rather than failing
    // the report: this member is additive information beside a coverage verdict
    // that has already been reached, and `compile --check` is where a broken
    // registry is the operator's problem.
    if let Ok(registry) = crate::compile::load_committed_registry(cfg, repo_root) {
        report.planned_territory = crate::query::planned_territory(&registry);
    }
    // Spec 094 §3.4: the headers that tried to claim a file and did not, over
    // the claim scan's own universe. Beside the classification, never inside
    // it: `coverage_with` above has already counted every file.
    report.near_miss_headers = crate::index::near_miss_headers(cfg, repo_root, &index.packages)?;
    Ok(report)
}

/// A listed path as the scope compares it: repo-relative POSIX with no leading
/// `./`, the spelling every other path in the report uses.
fn normalize_listed(path: &str) -> String {
    let p = path.trim().replace('\\', "/");
    p.strip_prefix("./").unwrap_or(&p).to_string()
}

/// Why a coverage universe is empty, when it is (spec 059 §3.2, §3.3).
///
/// The two cases have different fixes: no discovered package is usually
/// `layout.standalone_rust_workspaces` / `standalone_npm_packages` not naming a
/// package that exists, and packages with no source files is usually
/// `index.resolver_exclusions` pruning too much. A refusal that names the
/// condition without naming the likely cause makes the reader search for both,
/// and the audit found adopters deriving this class of thing by experiment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmptyUniverse {
    /// No package was discovered at all.
    NoPackages,
    /// Packages were discovered, but none contained a source file.
    NoSourceFiles,
}

impl EmptyUniverse {
    /// The reason line the refusal prints, cause included.
    pub fn explain(self) -> &'static str {
        match self {
            EmptyUniverse::NoPackages => {
                "no package was discovered, so there is no tree to assert about. \
                 Check [layout] standalone_rust_workspaces / standalone_npm_packages, \
                 and that the workspace manifest is where layout.cargo_workspace says"
            }
            EmptyUniverse::NoSourceFiles => {
                "packages were discovered but none contains a source file. \
                 Check [index] resolver_exclusions, which may be pruning the tree"
            }
        }
    }
}

/// `Some(reason)` when the universe is empty (spec 059 §3.2).
///
/// `--fail-on-untraced` is an assertion, and an assertion over an empty set is
/// vacuously true: mathematically correct and operationally wrong. A person
/// wiring it into CI is asserting the tree is fully owned, so if the tool cannot
/// see the tree the honest report is that the assertion did not run, and a CI
/// step that did not run its check should not be green. What matters is that
/// the denominator is zero, not why; the reason is for the message.
pub fn empty_universe(report: &CoverageReport) -> Option<EmptyUniverse> {
    if report.source_files > 0 {
        return None;
    }
    Some(if report.packages.is_empty() {
        EmptyUniverse::NoPackages
    } else {
        EmptyUniverse::NoSourceFiles
    })
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
