//! The coupling gate (spec 005): join the spec-as-source registry and the
//! code-as-source index, and refuse drift.
//!
//! `couple_with` is a pure function of `(config, registry, index, diff, waiver)`;
//! `couple` is the freshness-guarded form that loads the committed artifacts.
//! **`git` never runs here**: the CLI parses `git diff --no-color -U0
//! base...head` into a typed [`DiffInput`] and passes it in.
//!
//! Spec 029 adds a second, opt-in verdict: with `[coupling] require_ownership`
//! on, a changed source file that no spec *specifically* owns is `C-002`. The
//! ownership question is answered by [`crate::coverage`], the same classifier
//! `spec-spine index coverage` reports with, so the report predicts the gate.
//!
//! The behavioral semantics are ported intact from OAP
//! `tools/spec-spine/spec-code-coupling-check/src/lib.rs`
//! (`legitimate_owners` + the FR-005 strict-expansion guard, `is_bypass_against`,
//! `claim_matches`, `parse_waiver`, `span_overlaps_hunk`, `build_unit_claim_index`)
//! and `main.rs` (`parse_hunk_header`, in `cmd_couple`). Structure is fresh; the
//! algorithm is re-derived, not reinvented.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::{Deserialize, Serialize};
use spec_spine_types::{
    BypassEntry, BypassSource, CodebaseIndex, Config, Error, LineSpan, Registry, Severity,
    Violation,
};

use crate::coverage::{Ownership, classify, in_coverage_universe_with};
use crate::index::{Freshness, check_index_freshness, spec_md_rel};

/// The hardcoded generic bypass floor (spec 005 §3.5): the **single built-in
/// source** of bypass entries. The adopter's `config.coupling.bypass_prefixes`
/// (default **empty**) is unioned with this; it is **additive and cannot remove
/// a floor entry** (ported from OAP `BYPASS_PREFIXES`, pruned to the generic
/// subset). Match rules: trailing `/` ⇒ directory prefix; leading `**/` ⇒
/// tail-suffix anywhere; else exact file.
pub const DEFAULT_BYPASS_PREFIXES: &[&str] = &[
    ".github/",
    "docs/",
    "README.md",
    "CHANGELOG.md",
    "LICENSE",
    "CODEOWNERS",
    ".gitignore",
    ".gitattributes",
    "standards/spec/constitution.md",
    ".derived/",
    "**/Cargo.lock",
    "**/package-lock.json",
    "**/pnpm-lock.yaml",
];

/// The parsed diff handed to the gate. The CLI builds it from `git diff -U0`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffInput {
    pub files: Vec<DiffFile>,
}

/// One changed file with its new-side hunk spans. **Empty `hunks`** denotes a
/// whole-file change (a deletion, or `--paths-from` mode): it overlaps every
/// unit span.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffFile {
    pub path: String,
    #[serde(default)]
    pub hunks: Vec<LineSpan>,
    /// The path no longer exists at head (the CLI sets this from a
    /// `+++ /dev/null` header). Drift (`C-001`) treats a deletion like any
    /// whole-file change; the ownership ratchet (`C-002`, spec 029) never
    /// refuses one, since removing unowned code is how coverage goes up.
    /// Additive: absent in older `couple_json` requests, so it defaults off.
    #[serde(default)]
    pub deleted: bool,
}

/// A PR-body waiver: the trimmed reason after the configured keyword.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Waiver {
    pub reason: String,
}

/// Ownership as of a commit that is **not** the tree under judgment (spec 100
/// §3.2).
///
/// Reconstructed from that commit's own source bytes by compiling and indexing
/// its exported tree, never read from its committed derived shards: the
/// historical ledger is evidence somebody wrote and can be stale, absent or
/// wrong, while the corpus source at that commit is immutable and is what the
/// ledger was a function of. Removing the read removes the whole class of
/// "the base index is stale" states the gate would otherwise have to rule on.
#[derive(Clone, Debug)]
pub struct PriorOwnership {
    index: CodebaseIndex,
    superseders: BTreeMap<String, BTreeSet<String>>,
    /// The snapshot's tracked path inventory, when the caller has one.
    ///
    /// Only used to tell "the snapshot says nobody owned this path" from "the
    /// path was not in the snapshot at all" (spec 100 §3.5). Both answers give
    /// an empty owner set and the same verdict; the distinction is reported,
    /// never acted on. `None` means the caller did not supply an inventory, in
    /// which case no absence is claimed.
    paths: Option<BTreeSet<String>>,
}

impl PriorOwnership {
    /// Build a snapshot from a registry and index already compiled for the
    /// commit it describes.
    pub fn new(registry: &Registry, index: CodebaseIndex) -> Self {
        PriorOwnership {
            superseders: build_superseders(registry),
            index,
            paths: None,
        }
    }

    /// Declare the snapshot's tracked path inventory (spec 100 §3.5).
    pub fn with_paths(mut self, paths: BTreeSet<String>) -> Self {
        self.paths = Some(paths);
        self
    }

    /// The snapshot's index, for callers that need the claim-aware bypass
    /// predicate against it.
    pub fn index(&self) -> &CodebaseIndex {
        &self.index
    }

    /// True only when an inventory was supplied and does not list `path`.
    fn is_absent(&self, path: &str) -> bool {
        self.paths.as_ref().is_some_and(|p| !p.contains(path))
    }
}

/// Which prior snapshots a run holds, and which deletions belong to the
/// working-tree segment (spec 100 §3.1).
///
/// A deleted path is judged at the snapshot immediately preceding the segment
/// that recorded its deletion: the merge base for the committed range, HEAD for
/// the working-tree diff. The segment is carried here rather than on
/// [`DiffFile`] so the diff's serialized shape does not move, and because the
/// caller that computes the segments is the same one that builds the snapshots.
#[derive(Clone, Debug, Default)]
pub struct PriorSnapshots<'a> {
    /// Answers for deletions recorded in `merge-base...head`.
    pub merge_base: Option<&'a PriorOwnership>,
    /// Answers for deletions recorded in `git diff HEAD` (spec 081).
    pub head_commit: Option<&'a PriorOwnership>,
    /// The paths whose deletion the working-tree segment recorded.
    pub worktree_deletions: BTreeSet<String>,
}

impl PriorSnapshots<'_> {
    /// The snapshot that answers for a deleted `path`, and its report token.
    fn resolve(&self, path: &str) -> (Option<&PriorOwnership>, &'static str) {
        if self.worktree_deletions.contains(path) {
            match self.head_commit {
                Some(s) => (Some(s), SNAPSHOT_HEAD_COMMIT),
                None => (None, SNAPSHOT_HEAD_TREE),
            }
        } else {
            match self.merge_base {
                Some(s) => (Some(s), SNAPSHOT_MERGE_BASE),
                None => (None, SNAPSHOT_HEAD_TREE),
            }
        }
    }
}

/// The committed range's prior snapshot answered (spec 100 §3.7).
pub const SNAPSHOT_MERGE_BASE: &str = "merge-base";
/// The working-tree segment's prior snapshot, HEAD, answered (spec 100 §3.7).
pub const SNAPSHOT_HEAD_COMMIT: &str = "head-commit";
/// No prior snapshot was supplied, so the tree under judgment answered. This is
/// the compatibility path of spec 100 §3.8 and carries none of its guarantee.
pub const SNAPSHOT_HEAD_TREE: &str = "head-tree";

/// Which snapshot resolved one deleted path's owners (spec 100 §3.7).
///
/// Recorded for every deleted path the gate examined, so a reader of the report
/// can tell a path judged at a prior snapshot from one judged at head. A
/// verdict whose provenance is invisible is a verdict whose correctness cannot
/// be reviewed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeletionProvenance {
    pub path: String,
    /// One of [`SNAPSHOT_MERGE_BASE`], [`SNAPSHOT_HEAD_COMMIT`],
    /// [`SNAPSHOT_HEAD_TREE`].
    pub snapshot: String,
    /// The snapshot was read and does not contain the path (spec 100 §3.5).
    /// A computed answer, not a fallback: nobody owned it there.
    #[serde(skip_serializing_if = "is_false")]
    pub absent_at_snapshot: bool,
}

fn is_false(b: &bool) -> bool {
    !*b
}

/// The coupling outcome. Returned `Ok` for any completed analysis (clean, drift,
/// or waived); the CLI maps it to an exit code. A blocking drift is
/// `!violations.is_empty() && waiver.is_none()`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoupleReport {
    pub violations: Vec<Violation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub waiver: Option<String>,
    /// Non-bypassed diff paths that were examined.
    pub checked_paths: usize,
    /// Which snapshot answered for each deleted path examined (spec 100 §3.7).
    ///
    /// **Omitted when empty**, which is every run that examines no deletion.
    /// That is what lets a new report member coexist with preserving the exact
    /// bytes every pre-spec-100 input produced.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deletions: Vec<DeletionProvenance>,
}

impl CoupleReport {
    /// True when there is drift that no waiver excuses (exit 1).
    pub fn has_blocking_drift(&self) -> bool {
        !self.violations.is_empty() && self.waiver.is_none()
    }

    /// `1` for blocking drift, else `0`. Stale / IO are `Err`, not a report.
    pub fn exit_code(&self) -> u8 {
        if self.has_blocking_drift() { 1 } else { 0 }
    }
}

/// Reconstruct a prior snapshot from an exported tree (spec 100 §3.2).
///
/// `cfg` is the **snapshot's own** configuration, read from the exported tree,
/// not the run's: a change must not be able to re-own a path it is deleting by
/// editing `spec-spine.toml` in the same commit (the position spec 071 §3.2
/// takes for `delta`).
///
/// This compiles and indexes the tree rather than reading its committed
/// shards, which is what makes the answer trustworthy: the shards are evidence
/// somebody wrote, and the corpus source is immutable. A tree whose corpus does
/// not compile is an `Err`, never an empty snapshot, because a snapshot that
/// could not be built has not answered (spec 100 §3.5).
pub fn prior_ownership_from_root(cfg: &Config, root: &Path) -> Result<PriorOwnership, Error> {
    let registry = crate::compile::compile(cfg, root)?.registry;
    // `compile` reports a malformed spec as a validation violation rather than
    // an `Err`, so a corrupt historical corpus would otherwise produce a
    // snapshot missing exactly the specs that failed to parse: a confident
    // "nobody owned this path" derived from evidence that did not load. Spec
    // 100 §3.5 says such a snapshot has not answered, so it is refused here
    // rather than silently narrowed.
    if !registry.validation.passed {
        return Err(Error::Validation(registry.validation.violations.clone()));
    }
    let index = crate::index::index(cfg, root)?.index;
    Ok(PriorOwnership::new(&registry, index))
}

/// Freshness-guarded coupling. Refuses a stale index (exit 2, recompute first),
/// then loads the committed `registry.json` + `index.json` from `derived_dir`
/// and delegates to [`couple_with`].
pub fn couple(
    cfg: &Config,
    repo_root: &Path,
    diff: &DiffInput,
    waiver: Option<&Waiver>,
) -> Result<CoupleReport, Error> {
    couple_snapshots(cfg, repo_root, diff, waiver, &PriorSnapshots::default())
}

/// [`couple`] with the prior snapshots a deletion is judged against (spec 100).
///
/// The freshness guard, the committed-artifact load and the governed-scope
/// resolution are exactly [`couple`]'s; only the deletion resolution differs.
/// The CLI calls this form, and it is the one that carries spec 100's
/// two-snapshot guarantee.
pub fn couple_snapshots(
    cfg: &Config,
    repo_root: &Path,
    diff: &DiffInput,
    waiver: Option<&Waiver>,
    prior: &PriorSnapshots<'_>,
) -> Result<CoupleReport, Error> {
    match check_index_freshness(cfg, repo_root)? {
        Freshness::Stale { expected, actual } => return Err(Error::Stale { expected, actual }),
        Freshness::Fresh => {}
    }
    let registry = load_committed_registry(cfg, repo_root)?;
    let index = load_committed_index(cfg, repo_root)?;
    // Spec 078 §3.3: the gate reads the universe the coverage report reads. The
    // scope is resolved against the tree without an enumeration: every path
    // judged here is in the diff, which is tracked by construction or is the
    // caller's own `--paths-from` list, so no inventory could remove one (D-7).
    let scope = if cfg.coverage.governed_scope.is_empty() {
        crate::coverage::GovernedScope::empty()
    } else {
        crate::coverage::GovernedScope::from_globs(cfg, repo_root)
    };
    couple_with_prior(cfg, &registry, &index, &scope, prior, diff, waiver)
}

/// Pure coupling over already-loaded artifacts (overlays, tests). No IO.
///
/// **Compatibility entry point.** It holds one snapshot, the tree under
/// judgment, so a deleted path's owners are resolved at head: the claim the
/// change withdrew is already gone, which is the defect spec 100 §1 describes.
/// The behavior is retained deliberately and unchanged, and the report labels
/// every deletion it judged this way [`SNAPSHOT_HEAD_TREE`]. A caller that
/// wants spec 100's guarantee calls [`couple_with_prior`] (or [`couple_snapshots`],
/// or the CLI) and supplies the snapshots.
pub fn couple_with(
    cfg: &Config,
    registry: &Registry,
    index: &CodebaseIndex,
    diff: &DiffInput,
    waiver: Option<&Waiver>,
) -> Result<CoupleReport, Error> {
    couple_with_scope(
        cfg,
        registry,
        index,
        &crate::coverage::GovernedScope::empty(),
        diff,
        waiver,
    )
}

/// [`couple_with`] with a resolved governed scope (spec 078 §3.3), which widens
/// the universe the `C-002` arm asks about. No IO.
///
/// **Compatibility entry point**, on exactly [`couple_with`]'s terms: it holds
/// one snapshot and resolves deletions at head. See that function.
pub fn couple_with_scope(
    cfg: &Config,
    registry: &Registry,
    index: &CodebaseIndex,
    scope: &crate::coverage::GovernedScope,
    diff: &DiffInput,
    waiver: Option<&Waiver>,
) -> Result<CoupleReport, Error> {
    couple_with_prior(
        cfg,
        registry,
        index,
        scope,
        &PriorSnapshots::default(),
        diff,
        waiver,
    )
}

/// The gate, with the prior snapshots a deletion is judged against (spec 100).
///
/// The one entry point that carries spec 100's two-snapshot guarantee. Pure:
/// the snapshots are supplied already compiled, so no git and no IO happen
/// here, exactly as spec 005 §3.1 requires.
///
/// A deleted path resolves its owners, and its claim-aware bypass verdict,
/// against the snapshot preceding the segment that recorded the deletion. Every
/// other path resolves at head, unchanged. With no snapshots supplied this is
/// [`couple_with_scope`] byte for byte.
pub fn couple_with_prior(
    cfg: &Config,
    registry: &Registry,
    index: &CodebaseIndex,
    scope: &crate::coverage::GovernedScope,
    prior: &PriorSnapshots<'_>,
    diff: &DiffInput,
    waiver: Option<&Waiver>,
) -> Result<CoupleReport, Error> {
    let diff_paths: BTreeSet<String> = diff.files.iter().map(|f| f.path.clone()).collect();
    let superseders = build_superseders(registry);
    // Spec 033: every "is this a spec.md, and whose?" question below is asked
    // against the configured corpus root, never a literal `specs/`.
    let specs_dir = cfg.layout.specs_dir.as_str();

    let mut violations: Vec<Violation> = Vec::new();
    let mut checked_paths = 0usize;
    let mut deletions: Vec<DeletionProvenance> = Vec::new();

    for file in &diff.files {
        let path = &file.path;
        // Spec 100 §3.1: a deleted path is judged at the snapshot preceding the
        // segment that recorded its deletion. Both questions below (is this
        // path governed at all, and who owns it) read that one snapshot, so
        // there is no state in which the gate decides a path is governed under
        // one snapshot and resolves its owners under another (§3.3).
        let (snapshot, snapshot_token) = if file.deleted {
            prior.resolve(path)
        } else {
            (None, SNAPSHOT_HEAD_TREE)
        };
        let owning_index = snapshot.map_or(index, PriorOwnership::index);
        let owning_superseders = snapshot.map_or(&superseders, |s| &s.superseders);
        // Effective bypass = declared state root (spec 036), else the
        // hardcoded floor ∪ adopter list (additive) UNLESS an explicit,
        // resolved unit claim covers the path (spec 008). One predicate,
        // shared with `index coverage`, so the report and the gate cannot
        // disagree about which paths are even looked at.
        if is_bypassed_path(cfg, owning_index, path) {
            continue;
        }
        checked_paths += 1;
        if file.deleted {
            deletions.push(DeletionProvenance {
                path: path.clone(),
                snapshot: snapshot_token.to_string(),
                absent_at_snapshot: snapshot.is_some_and(|s| s.is_absent(path)),
            });
        }

        // Spec 029: the ownership ratchet. Asked before the drift question,
        // over exactly the universe `index coverage` reports on, so the report
        // predicts this arm. A deleted path is never refused for lacking an
        // owner. `C-002` takes precedence over `C-001` for one path (claiming
        // the file in a spec resolves both), so a path raises at most one code.
        if cfg.coupling.require_ownership
            && !file.deleted
            && in_coverage_universe_with(cfg, index, scope, path)
        {
            let message = match classify(index, path) {
                Ownership::Specific => None,
                Ownership::FloorOnly(floors) => Some(format!(
                    "'{path}' has no specific owning spec; only the package floor of {} covers it (require_ownership is on)",
                    floors.join(", ")
                )),
                Ownership::Unowned => Some(format!(
                    "'{path}' is not claimed by any spec (require_ownership is on)"
                )),
            };
            if let Some(message) = message {
                // No `owners` (spec 045 §3.1): `C-002` fires precisely when no
                // spec specifically claims the path, so there is no owner to
                // name. The floor specs the message reports are why the path is
                // debt, not who owns it.
                violations.push(Violation::new("C-002", Severity::Error, message).at(path.clone()));
                continue;
            }
        }

        let owners = owners_for_path(
            specs_dir,
            path,
            &file.hunks,
            owning_index,
            owning_superseders,
        );
        if owners.is_empty() {
            continue; // unclaimed path: not a drift concern (see the ratchet above)
        }
        if any_owner_in_diff(specs_dir, &owners, &diff_paths) {
            continue; // primary-owner heuristic: any one owner's spec.md cleared it
        }

        // Spec 045 §3.1: the owner set is carried as data as well as rendered
        // into the message. `owners` is the sole non-prose copy, in the same
        // sorted order the sentence names, so a consumer of the spec 034
        // envelope reads it instead of regexing English. `owners` came from a
        // `BTreeSet`, so the order is the message's by construction.
        let names: Vec<String> = owners.iter().cloned().collect();
        let mut v = Violation::new(
            "C-001",
            Severity::Error,
            format!(
                "'{path}' changed without an authoring edit to any owning spec ({})",
                names.join(", ")
            ),
        )
        .at(path.clone());
        v.owners = names;
        violations.push(v);
    }

    violations.sort_by(|a, b| a.path.cmp(&b.path));

    deletions.sort_by(|a, b| a.path.cmp(&b.path));

    Ok(CoupleReport {
        violations,
        waiver: waiver.map(|w| w.reason.clone()),
        checked_paths,
        deletions,
    })
}

/// The merged bypass list the gate matches against, attributed (spec 047 §3.2).
///
/// The built-in floor first, then the adopter's entries in their declared
/// order, which is the order [`is_bypassed_path`] evaluates them. A prefix
/// present in both lists appears once, carrying both sources: the match is an
/// `or`, so a duplicate is harmless, and reporting it twice would suggest
/// something to clean up that is not there.
///
/// This exists because half the list is a constant compiled into the binary and
/// therefore unreadable from outside the process. claude-observatory, which
/// judges repositories other than itself, hand-parses each target's TOML and
/// records in its spec 015 D-3 that the built-in floor is simply unknowable to
/// it. That is a consumer reimplementing half a contract and knowing the other
/// half is wrong. Nothing about the gate's decision changes here: a value it
/// already computes becomes reachable.
pub fn effective_bypass_prefixes(cfg: &Config) -> Vec<BypassEntry> {
    let mut entries: Vec<BypassEntry> = DEFAULT_BYPASS_PREFIXES
        .iter()
        .map(|p| BypassEntry {
            prefix: (*p).to_string(),
            sources: vec![BypassSource::BuiltIn],
        })
        .collect();
    // Spec 092 §3.8: the configured derived root is a built-in floor entry, so
    // a reader of this list sees the same set the gate applies. Reported as
    // built-in rather than as config: the adopter did not ask for a bypass,
    // they named where the compiler writes, and the gate derived the rest.
    // Skipped when it is already the default `.derived`, which is on the
    // constant above and would otherwise be listed twice.
    let derived = format!("{}/", cfg.layout.derived_dir.trim_end_matches('/'));
    if derived != "/" && !entries.iter().any(|e| e.prefix == derived) {
        entries.push(BypassEntry {
            prefix: derived,
            sources: vec![BypassSource::BuiltIn],
        });
    }
    for declared in &cfg.coupling.bypass_prefixes {
        match entries.iter_mut().find(|e| &e.prefix == declared) {
            Some(existing) => {
                if !existing.sources.contains(&BypassSource::Config) {
                    existing.sources.push(BypassSource::Config);
                }
            }
            None => entries.push(BypassEntry {
                prefix: declared.clone(),
                sources: vec![BypassSource::Config],
            }),
        }
    }
    entries
}

/// The effective bypass verdict for one path: hardcoded floor ∪ adopter list
/// (additive), with explicit unit claims taking precedence (spec 008),
/// exactly as [`couple_with`] applies it. Public so the CLI's
/// dependency-only auto-waiver pre-filter (spec 005 §3.5) examines the same
/// non-bypassed path set the gate itself will check; the claim-awareness is
/// what keeps a claim-overridden floor path from slipping past that
/// pre-filter into a mechanical waiver.
pub fn is_bypassed_path(cfg: &Config, index: &CodebaseIndex, path: &str) -> bool {
    // Spec 036: a declared state root is bypassed unconditionally, and the
    // check precedes the spec 008 claim override rather than joining it. A
    // claim inside the root is not a precedence question to resolve in either
    // direction: it is a contradiction, and `lint` reports it as `L-006`.
    // Letting the claim win would reintroduce override into a directory whose
    // whole purpose is to be ungoverned; letting the bypass win silently would
    // discard a unit an author wrote deliberately.
    if cfg.layout.is_state_path(path) {
        return true;
    }
    // Spec 092 §3.8: the CONFIGURED derived root, on the same terms. The floor
    // below spells `.derived/`, which is the default and not the answer: a
    // repository whose `derived_dir` is elsewhere has every regenerated shard
    // judged as source, so the change that recomputes the ledger becomes a
    // `C-001` drift refusal against a spec that says nothing about shard bytes
    // and, under `require_ownership`, a `C-002` unclaimed file as well. The
    // gate should not need a configuration key to recognise its own output.
    //
    // Placed with the state root, before the spec 008 claim override, for the
    // same reason: a unit claiming a compiled artifact is a contradiction
    // (`lint` reports the derived tree as the compiler's), not a precedence
    // question. The literal `.derived/` stays on the floor below, so a
    // repository that configures nothing keeps exactly the behavior it has and
    // one mid-migration has both paths answered.
    if cfg.layout.is_derived_path(path) {
        return true;
    }
    !explicitly_claimed(path, index)
        && (is_bypass(path, DEFAULT_BYPASS_PREFIXES)
            || is_bypass(path, &cfg.coupling.bypass_prefixes))
}

/// Spec 008 §3.1: true iff at least one **resolved, ownership-bearing unit
/// claim** covers `path`: a location file matching exactly, or by directory
/// prefix for a directory-form file unit (004 §3.3). Implicit path-level
/// ownership (manifest metadata, comment headers → `implementingPaths`)
/// deliberately does NOT count (§3.2): an explicit unit in spec frontmatter
/// is an author saying *this exact surface is governed*; a crate floor is a
/// blanket safety net that keeps deferring to bypass.
fn explicitly_claimed(path: &str, index: &CodebaseIndex) -> bool {
    index.traceability.mappings.iter().any(|m| {
        m.resolved_units.iter().filter(|ru| ru.ownership).any(|ru| {
            ru.locations
                .iter()
                .any(|loc| claim_matches(&loc.file, path))
        })
    })
}

/// Parse a waiver from the PR body using the configured keyword. Returns the
/// first line's trimmed reason (ported from OAP `parse_waiver`).
pub fn parse_waiver(cfg: &Config, body: &str) -> Option<Waiver> {
    let keyword = &cfg.coupling.waiver_keyword;
    for line in body.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix(keyword.as_str()) {
            let reason = rest.trim();
            if !reason.is_empty() {
                return Some(Waiver {
                    reason: reason.to_string(),
                });
            }
        }
    }
    None
}

// ===== owner derivation (the ported algorithm) =====

/// The legitimate owners of a changed `(path, hunks)`. Unions span-aware
/// resolved-unit ownership with whole-file `implementingPaths`, applies
/// supersedes transfer, then amends-awareness under the FR-005 strict guard.
///
/// Public since spec 048: `index owner <path>` reports what the gate would
/// decide, and it must call this rather than reimplement it. A second
/// implementation that agreed today and drifted next quarter would be worse
/// than no verb at all, because a consumer would have stopped rebuilding the
/// map by hand and started trusting an answer nobody tests against the gate.
/// Pass empty `hunks` for the whole-file interpretation the gate already
/// defines (`--paths-from` mode uses it today), which is the right reading of
/// "who owns this file" when there is no diff.
pub fn owners_for_path(
    specs_dir: &str,
    path: &str,
    hunks: &[LineSpan],
    index: &CodebaseIndex,
    superseders: &BTreeMap<String, BTreeSet<String>>,
) -> BTreeSet<String> {
    // 1. Candidate (owner, span) pairs across both linkage sources.
    let mut candidates: Vec<(String, Option<LineSpan>)> = Vec::new();
    for m in &index.traceability.mappings {
        for ru in &m.resolved_units {
            if !ru.ownership {
                continue; // `references` is non-owning
            }
            for loc in &ru.locations {
                if claim_matches(&loc.file, path) {
                    candidates.push((m.spec_id.clone(), loc.span));
                }
            }
        }
        for ip in &m.implementing_paths {
            if claim_matches(&ip.path, path) {
                candidates.push((m.spec_id.clone(), None)); // whole-file
            }
        }
    }

    // 2. Supersedes transfer: a successor inherits its predecessor's authority
    //    (with the same span). Additive: the predecessor is not removed.
    let mut transferred: Vec<(String, Option<LineSpan>)> = Vec::new();
    for (owner, span) in &candidates {
        for succ in transitive_superseders(owner, superseders) {
            transferred.push((succ, *span));
        }
    }
    candidates.extend(transferred);

    // 3. Keep owners whose span overlaps a hunk (empty hunks ⇒ whole-file change
    //    overlaps everything).
    let mut owners: BTreeSet<String> = BTreeSet::new();
    for (spec_id, span) in &candidates {
        let applies = hunks.is_empty() || hunks.iter().any(|h| span_overlaps_hunk(*span, *h));
        if applies {
            owners.insert(spec_id.clone());
        }
    }

    // 4. Amends-awareness, only when the base owner set is non-empty (FR-005
    //    strict-expansion guard) and the path is `<specs_dir>/<id>/spec.md`.
    if !owners.is_empty() {
        if let Some(amended_id) = spec_id_for_spec_md_path(specs_dir, path) {
            for m in &index.traceability.mappings {
                if m.amends.iter().any(|a| a == amended_id) {
                    owners.insert(m.spec_id.clone());
                }
                if m.spec_id == amended_id {
                    if let Some(record) = &m.amendment_record {
                        owners.insert(record.clone());
                    }
                }
            }
        }
    }

    owners
}

/// True when at least one owner's `<specs_dir>/<id>/spec.md` is in the diff (the
/// primary-owner heuristic, ported from OAP `OwnerSet::any_owner_in_diff`).
fn any_owner_in_diff(
    specs_dir: &str,
    owners: &BTreeSet<String>,
    diff_paths: &BTreeSet<String>,
) -> bool {
    owners
        .iter()
        .any(|id| diff_paths.contains(&spec_md_rel(specs_dir, id)))
}

/// Direct `predecessor → {superseders}` map from the registry's `supersedes`.
///
/// Public since spec 048, so a caller can assemble the same input
/// [`owners_for_path`] is given inside the gate.
///
/// Only **full** supersession contributes a whole-spec authority transfer (spec
/// 018). A **partial** item transfers authority over a single unit only: that
/// is threaded through the index instead, as a `SourceField::Supersedes`
/// resolved unit owned by the superseder, so it is already an owner of that
/// unit's paths via `owners_for_path` step 1 and must NOT also inherit the
/// predecessor's entire surface here.
pub fn build_superseders(registry: &Registry) -> BTreeMap<String, BTreeSet<String>> {
    let mut map: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for spec in &registry.specs {
        for item in &spec.supersedes {
            if item.is_full() {
                map.entry(item.spec().to_string())
                    .or_default()
                    .insert(spec.id.clone());
            }
        }
    }
    map
}

/// Transitive closure of superseders of `id` (handles supersedes chains).
fn transitive_superseders(
    id: &str,
    superseders: &BTreeMap<String, BTreeSet<String>>,
) -> BTreeSet<String> {
    let mut out: BTreeSet<String> = BTreeSet::new();
    let mut stack: Vec<String> = vec![id.to_string()];
    while let Some(cur) = stack.pop() {
        if let Some(succs) = superseders.get(&cur) {
            for s in succs {
                if out.insert(s.clone()) {
                    stack.push(s.clone());
                }
            }
        }
    }
    out
}

/// Inclusive span overlap. `None` (whole file) overlaps any hunk (ported from
/// OAP `span_overlaps_hunk`, here on inclusive [start,end] both sides).
fn span_overlaps_hunk(span: Option<LineSpan>, hunk: LineSpan) -> bool {
    match span {
        None => true,
        Some(s) => s.start_line <= hunk.end_line && hunk.start_line <= s.end_line,
    }
}

/// Slash-anchored prefix match: a directory `claim` (trailing `/`, or any path
/// treated as a directory) owns every file under it; an exact path matches
/// itself (ported from OAP `claim_matches`).
pub(crate) fn claim_matches(claim: &str, path: &str) -> bool {
    if claim == path {
        return true;
    }
    let claim_dir = if claim.ends_with('/') {
        claim.to_string()
    } else {
        format!("{claim}/")
    };
    path.starts_with(&claim_dir)
}

/// Bypass match against a prefix slice (ported from OAP `is_bypass_against`).
fn is_bypass<S: AsRef<str>>(path: &str, prefixes: &[S]) -> bool {
    prefixes.iter().any(|prefix| {
        let prefix = prefix.as_ref();
        if let Some(tail) = prefix.strip_prefix("**/") {
            path == tail || path.ends_with(&format!("/{tail}"))
        } else if prefix.ends_with('/') {
            path.starts_with(prefix)
        } else {
            path == prefix || path == format!("{prefix}/")
        }
    })
}

/// Parse `<specs_dir>/<id>/spec.md` into `<id>` (ported from OAP
/// `spec_id_for_spec_md_path`). `None` for any other path.
///
/// The exact inverse of [`spec_md_rel`], against the configured
/// `layout.specs_dir` rather than a literal `specs/` (spec 033). The prefix and
/// the separator are stripped in two steps on purpose: a single
/// `strip_prefix("specs")` would accept `specsX/005-x/spec.md`.
///
/// Public since spec 045: the CLI's resolution footer asks the same question of
/// the diff (is exactly one spec.md edited, and whose?) and must ask it with the
/// gate's own answer rather than a second, drifting copy of the path grammar.
pub fn spec_id_for_spec_md_path<'p>(specs_dir: &str, path: &'p str) -> Option<&'p str> {
    let rest = path
        .strip_prefix(specs_dir.trim_end_matches('/'))?
        .strip_prefix('/')?;
    let (id, tail) = rest.split_once('/')?;
    if tail == "spec.md" { Some(id) } else { None }
}

// ===== committed-artifact loaders (the IO half of `couple`) =====
//
// Both artifacts are stored as a per-spec/per-package shard tree (spec 022);
// these delegate to the single assembler each producer owns, which reconstructs
// the aggregate `Registry` / `CodebaseIndex` from the shard set. The gate logic
// in [`couple_with`] consumes those aggregates unchanged.

fn load_committed_registry(cfg: &Config, repo_root: &Path) -> Result<Registry, Error> {
    crate::compile::load_committed_registry(cfg, repo_root)
}

fn load_committed_index(cfg: &Config, repo_root: &Path) -> Result<CodebaseIndex, Error> {
    crate::index::load_committed_index(cfg, repo_root)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bypass_match_rules() {
        assert!(is_bypass(
            ".github/workflows/ci.yml",
            DEFAULT_BYPASS_PREFIXES
        ));
        assert!(is_bypass("docs/x.md", DEFAULT_BYPASS_PREFIXES));
        assert!(is_bypass("README.md", DEFAULT_BYPASS_PREFIXES));
        assert!(is_bypass("crates/core/Cargo.lock", DEFAULT_BYPASS_PREFIXES)); // **/ tail
        assert!(is_bypass(".derived/x.json", DEFAULT_BYPASS_PREFIXES));
        assert!(!is_bypass(
            "crates/core/src/lib.rs",
            DEFAULT_BYPASS_PREFIXES
        ));
    }

    #[test]
    fn claim_match_exact_and_dir_prefix() {
        assert!(claim_matches("crates/core", "crates/core/src/lib.rs"));
        assert!(claim_matches("crates/core/", "crates/core/src/lib.rs"));
        assert!(claim_matches("Makefile", "Makefile"));
        assert!(!claim_matches("crates/cor", "crates/core/src/lib.rs"));
    }

    #[test]
    fn span_overlap_inclusive() {
        assert!(span_overlaps_hunk(None, LineSpan::new(1, 1)));
        assert!(span_overlaps_hunk(
            Some(LineSpan::new(10, 20)),
            LineSpan::new(20, 25)
        ));
        assert!(!span_overlaps_hunk(
            Some(LineSpan::new(10, 20)),
            LineSpan::new(21, 25)
        ));
    }

    #[test]
    fn spec_md_path_parse() {
        assert_eq!(
            spec_id_for_spec_md_path("specs", "specs/005-x/spec.md"),
            Some("005-x")
        );
        assert_eq!(
            spec_id_for_spec_md_path("specs", "specs/005-x/plan.md"),
            None
        );
        assert_eq!(
            spec_id_for_spec_md_path("specs", "crates/core/src/lib.rs"),
            None
        );
    }

    #[test]
    fn spec_md_path_parse_honors_configured_dir() {
        // Spec 033: the corpus root is configuration, not a literal.
        assert_eq!(
            spec_id_for_spec_md_path("contracts", "contracts/005-x/spec.md"),
            Some("005-x")
        );
        // The default root is not privileged once another one is configured.
        assert_eq!(
            spec_id_for_spec_md_path("contracts", "specs/005-x/spec.md"),
            None
        );
        // A nested root works, and a sibling that merely shares its prefix does
        // not: the separator is stripped as its own step for exactly this.
        assert_eq!(
            spec_id_for_spec_md_path("docs/specs", "docs/specs/005-x/spec.md"),
            Some("005-x")
        );
        assert_eq!(
            spec_id_for_spec_md_path("specs", "specsX/005-x/spec.md"),
            None
        );
        // A trailing slash names the same corpus root.
        assert_eq!(
            spec_id_for_spec_md_path("specs/", "specs/005-x/spec.md"),
            Some("005-x")
        );
    }

    #[test]
    fn owner_in_diff_matches_the_configured_dir() {
        let owners: BTreeSet<String> = ["005-x".to_string()].into_iter().collect();
        let under_contracts: BTreeSet<String> = ["contracts/005-x/spec.md".to_string()]
            .into_iter()
            .collect();
        let under_specs: BTreeSet<String> =
            ["specs/005-x/spec.md".to_string()].into_iter().collect();

        assert!(any_owner_in_diff("contracts", &owners, &under_contracts));
        assert!(!any_owner_in_diff("contracts", &owners, &under_specs));
        assert!(any_owner_in_diff("specs", &owners, &under_specs));
    }
}
