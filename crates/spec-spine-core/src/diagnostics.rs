//! Reading the diagnostics the indexer already recorded (spec 050).
//!
//! The indexer classifies an unresolved unit into one of three tiers (spec 025):
//! `W-002` for a non-owning `references` edge, `W-001` for an owning edge on a
//! spec that is in flight, and the natural `I-0xx` hard error otherwise. All
//! three land in the committed shard. The error tier is gated, because
//! `index::BLOCKING_CODES` makes its shard read as stale. The warning tier was
//! gated by nothing and readable only by re-running the *writing* command or by
//! parsing `index render`'s markdown.
//!
//! This module is the reader that was missing. Everything here loads the
//! **committed shard set** and recomputes nothing: the counts describe the
//! ledger the corpus actually compiled to, which is the only answer that can be
//! checked against what is in git. When the ledger and the tree disagree, that
//! disagreement is staleness and `index check` reports it separately (spec 050
//! 3.3, 3.5).

use std::collections::BTreeMap;
use std::path::Path;

use spec_spine_types::{
    Config, Diagnostic, Error, INDEX_SCHEMA_VERSION, IndexPackageShard, IndexSpecShard, Severity,
};

use crate::index::{index_dir, read_committed_index_shards};
use crate::shard::{self, BY_PACKAGE_DIR, BY_SPEC_DIR};

/// The two codes spec 025 uses for a unit that resolved to nothing.
///
/// `--fail-on-unresolved` is defined over exactly this set, which is why it is
/// not spelled `--fail-on-warn`: it names what the codes *mean*, and `lint`
/// already owns that flag over a different code set (`L-001`..`L-006`).
pub const UNRESOLVED_CODES: &[&str] = &["W-001", "W-002"];

/// One recorded diagnostic, with the spec whose shard carries it.
///
/// [`Diagnostic`] itself has no spec member: the attribution lives in the shard
/// it sits in, and `load_committed_index` flattens both shard sets into one
/// aggregate and drops it. A reader that wants to say *which* spec has 248
/// unresolved units therefore has to go to the shards, not the aggregate.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttributedDiagnostic {
    pub spec_id: String,
    /// The tier the shard recorded it in. Typed rather than a string: the
    /// counting fold below branches on it, and a stringly-typed tier makes
    /// "anything that is not `error` counts as a warning" an invariant nothing
    /// enforces. Serializes as `"error"` / `"warning"`, unchanged on the wire.
    pub severity: Severity,
    pub code: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

/// Counts over the committed diagnostics.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticCounts {
    pub warnings: usize,
    pub errors: usize,
    /// Per-code totals, sorted by code. Zero entries are omitted, so the shape
    /// does not grow with codes a corpus does not have.
    pub by_code: BTreeMap<String, usize>,
    /// Committed shard files the tally could not read, as `<dir>/<name>`,
    /// sorted (spec 095 §3.3). Their diagnostics are not in the counts above.
    ///
    /// Not serialized: the payload carries the number, as
    /// [`IndexCheckReport::skipped_shards`], and this list exists so the prose
    /// form can name the files beside the stale-shard lines. Empty whenever the
    /// counts came from [`count`] or from a tree that reads fresh, since a fresh
    /// tree is byte-identical to what the indexer emits.
    #[serde(skip)]
    pub unreadable: Vec<String>,
}

impl DiagnosticCounts {
    /// Whether any unit resolved to nothing under a warning tier: what
    /// `--fail-on-unresolved` refuses (spec 050 3.2).
    pub fn has_unresolved(&self) -> bool {
        UNRESOLVED_CODES
            .iter()
            .any(|c| self.by_code.get(*c).copied().unwrap_or(0) > 0)
    }

    /// Nothing recorded in either tier. A clean corpus keeps the bare verdict
    /// line (spec 050 3.1).
    pub fn is_empty(&self) -> bool {
        self.warnings == 0 && self.errors == 0
    }
}

/// Every diagnostic in the committed spec shards, sorted by spec, then code,
/// then path, so the listing is byte-identical for one committed tree.
pub fn committed_diagnostics(
    cfg: &Config,
    repo_root: &Path,
) -> Result<Vec<AttributedDiagnostic>, Error> {
    let (spec_shards, _) = read_committed_index_shards(cfg, repo_root)?;
    let mut out = Vec::new();
    for sh in &spec_shards {
        let spec_id = &sh.mapping.spec_id;
        let mut push = |severity: Severity, d: &Diagnostic| {
            out.push(AttributedDiagnostic {
                spec_id: spec_id.clone(),
                severity,
                code: d.code.clone(),
                message: d.message.clone(),
                path: d.path.clone(),
            });
        };
        for d in &sh.diagnostics.errors {
            push(Severity::Error, d);
        }
        for d in &sh.diagnostics.warnings {
            push(Severity::Warning, d);
        }
    }
    out.sort_by(|a, b| {
        a.spec_id
            .cmp(&b.spec_id)
            .then(a.code.cmp(&b.code))
            .then(a.path.cmp(&b.path))
            .then(a.message.cmp(&b.message))
    });
    Ok(out)
}

/// Count the committed diagnostics without materializing the listing.
///
/// Folds straight over the shards rather than counting
/// [`committed_diagnostics`]'s vector, because `index check` calls this on
/// every CI run and has no use for the listing it would allocate and drop. The
/// two paths are pinned against each other in the fixtures, so they cannot
/// drift apart.
///
/// This still reads the shard set a second time: `check_index_freshness` has
/// already read it, and does not hand it back. Folding the two into one read
/// means refactoring the staleness gate itself, which is load-bearing and owned
/// by spec 004, so it is left alone deliberately (spec 050 3.5). The cost is
/// one extra pass over small per-spec JSON files, paid by a verb that already
/// hashes every input.
///
/// **Best-effort since spec 095.** A file in either shard directory that does
/// not deserialize into its shard type, or that carries a foreign schema MAJOR,
/// is skipped and named in [`DiagnosticCounts::unreadable`] instead of raising
/// `Error::Parse`. The callers are the judging verbs (`index check`, `check` and
/// their facades), which compute the freshness verdict first; spec 086 already
/// reads such a file as `orphaned` or `modified`, and a tally that threw would
/// discard that verdict for exit 3 (095 §1.1). The consumers that must refuse a
/// corrupt ledger read it through `read_committed_index_shards`, which is
/// unchanged. The one remaining error is an index that was never built, the
/// state `check_index_freshness` refuses before any caller gets here.
pub fn committed_counts(cfg: &Config, repo_root: &Path) -> Result<DiagnosticCounts, Error> {
    let dir = index_dir(cfg, repo_root);
    if !dir.exists() {
        return Err(Error::Io(format!(
            "read {} (run `spec-spine index` first?): not found",
            dir.display()
        )));
    }
    let mut counts = DiagnosticCounts::default();
    for (name, bytes) in shard_files(&dir, BY_SPEC_DIR, &mut counts.unreadable) {
        let Some(sh) = serde_json::from_slice::<IndexSpecShard>(&bytes)
            .ok()
            .filter(|sh| known_major(&sh.schema_version))
        else {
            counts.unreadable.push(format!("{BY_SPEC_DIR}/{name}"));
            continue;
        };
        for d in &sh.diagnostics.errors {
            counts.errors += 1;
            *counts.by_code.entry(d.code.clone()).or_insert(0) += 1;
        }
        for d in &sh.diagnostics.warnings {
            counts.warnings += 1;
            *counts.by_code.entry(d.code.clone()).or_insert(0) += 1;
        }
    }
    // Package shards record no diagnostics, so they add nothing to the counts;
    // they are read so an unreadable one is reported like a spec shard (095
    // §3.6 case 3) rather than passing unmentioned.
    for (name, bytes) in shard_files(&dir, BY_PACKAGE_DIR, &mut counts.unreadable) {
        let readable = serde_json::from_slice::<IndexPackageShard>(&bytes)
            .ok()
            .is_some_and(|sh| known_major(&sh.schema_version));
        if !readable {
            counts.unreadable.push(format!("{BY_PACKAGE_DIR}/{name}"));
        }
    }
    counts.unreadable.sort();
    Ok(counts)
}

/// The `.json` files of one shard directory. A directory whose files cannot be
/// read is recorded as unreadable as a whole, so the tally never throws.
fn shard_files(dir: &Path, label: &str, unreadable: &mut Vec<String>) -> Vec<(String, Vec<u8>)> {
    shard::read_shard_files(&dir.join(label)).unwrap_or_else(|_| {
        unreadable.push(format!("{label}/"));
        Vec::new()
    })
}

/// Whether a shard's `schemaVersion` is one this build reads (spec 086 D-5's
/// MAJOR rule, applied as a skip rather than a refusal).
fn known_major(found: &str) -> bool {
    shard::check_major("index", found, INDEX_SCHEMA_VERSION).is_ok()
}

/// Annotate a stale report's drift lines with the files the tally could not
/// read (spec 095 §3.3): the prose form names them on the lines that already
/// list the stale shards. A name the capped list does not show gets its own
/// line, so no skipped file goes unnamed. Shared by `index check` and `check`.
pub fn annotate_unreadable(actual: &str, unreadable: &[String]) -> String {
    const NOTE: &str = "(unreadable: not counted in the diagnostics tally)";
    let mut named = vec![false; unreadable.len()];
    let mut lines: Vec<String> = actual
        .lines()
        .map(|line| {
            let file = line.trim_start().split_once(' ').map(|(_, f)| f);
            match unreadable.iter().position(|u| Some(u.as_str()) == file) {
                Some(i) => {
                    named[i] = true;
                    format!("{line} {NOTE}")
                }
                None => line.to_string(),
            }
        })
        .collect();
    for (file, _) in unreadable.iter().zip(&named).filter(|(_, n)| !**n) {
        lines.push(format!("  unreadable {file} {NOTE}"));
    }
    lines.join("\n")
}

/// Fold a listing into counts. Split out so the arithmetic is testable without
/// a corpus on disk.
pub fn count(diagnostics: &[AttributedDiagnostic]) -> DiagnosticCounts {
    let mut counts = DiagnosticCounts::default();
    for d in diagnostics {
        match d.severity {
            Severity::Error => counts.errors += 1,
            // The indexer records two tiers. `Info` is in the shared `Severity`
            // enum for lint's benefit and never reaches a shard; counting it
            // with the warnings keeps the two totals exhaustive rather than
            // letting a diagnostic fall out of both.
            Severity::Warning | Severity::Info => counts.warnings += 1,
        }
        *counts.by_code.entry(d.code.clone()).or_insert(0) += 1;
    }
    counts
}

/// `index check`'s report payload: the freshness verdict plus the diagnostics
/// the committed ledger records (spec 050 3.1).
///
/// Defined here, and serialized by both the facade (`check_freshness_json`) and
/// the CLI's `--json` arm, so the two cannot drift. Spec 037's parity test pins
/// them against each other; before this type they were composed twice.
///
/// `compile --check` deliberately does **not** use this shape. It renders the
/// bare freshness object, because index diagnostics are meaningless for the
/// registry and a permanently-zero member would be worse than none.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexCheckReport {
    pub fresh: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actual: Option<String>,
    pub diagnostics: DiagnosticCounts,
    /// Claimed paths that exist and that no content hash covers (spec 057).
    ///
    /// Additive on one verb's report payload, so `VERDICT_SCHEMA_VERSION` does
    /// not move: spec 050 §3.6 settled that, and this follows it rather than
    /// reopening it. Reporting only; `index check`'s exit code is unchanged
    /// with and without `--fail-on-unresolved`, because the gate half is the
    /// lint's and one flag must not mean two conditions.
    #[serde(default)]
    pub unwitnessed: UnwitnessedCounts,
    /// How many committed shard files the diagnostics tally skipped because
    /// they would not deserialize (spec 095 §3.3).
    ///
    /// Additive: `default` so a payload from a pre-095 producer reads zero, and
    /// omitted when zero so a tree with nothing skipped emits exactly the bytes
    /// it emitted before, which is the common case in every corpus. The verdict
    /// schema does not move (spec 050 §3.6). A non-zero value accompanies a
    /// stale verdict: the files it counts are drift by spec 086's comparison.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub skipped_shards: usize,
}

fn is_zero(n: &usize) -> bool {
    *n == 0
}

/// The registry half of a composed `check` verdict (spec 075 §3.4).
///
/// Freshness **and** validation, because the two answer different questions
/// about one tree and the composed exit code needs both: a corpus that fails
/// validation makes staleness meaningless, which is why `1` outranks `2` in the
/// fold. `compile --check` reports them as an exit code and a stale report;
/// this carries them as data so the composed verb can attribute each.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryCheckReport {
    pub fresh: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actual: Option<String>,
    /// False when the corpus itself does not validate, in which case `fresh`
    /// was never computed and is reported `false`.
    pub validation_passed: bool,
    /// Warning-tier violations the compile produced (spec 077 §3.4).
    ///
    /// Carried so the composed verb can refuse under `--fail-on-warn` without
    /// recompiling, and can say **which tree** refused. Without it an exit `1`
    /// from `check` would be unattributable, and the reader would be sent to
    /// the wrong verb. Additive with a `default`, so a report deserialized from
    /// a pre-077 producer reads zero rather than failing.
    #[serde(default)]
    pub warnings: usize,
}

/// Both trees' verdicts, from the one verb the session protocol calls
/// (spec 075 §3.2).
///
/// Each half keeps the shape its own primitive emits, unsummarized and
/// unmerged: spec 031 §3.3 makes the registry stale report's structure
/// contractual because a protocol reads the drifted shard names back to an
/// operator, and exit 2 alone cannot say which shard moved. A composed verb
/// that flattened the two would break a contract that already exists.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckReport {
    pub registry: RegistryCheckReport,
    pub index: IndexCheckReport,
}

/// The claimed-but-unwitnessed tally `index check` reports (spec 057 §3.3).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnwitnessedCounts {
    /// Every claimed path in no content hash, deliberate ones included. The
    /// total is the honest number: an allowlist makes a gap explicit, not
    /// smaller.
    pub total: usize,
    /// How many of `total` a `[lint] unwitnessed_allowed` pattern covers.
    pub allowed: usize,
}

impl IndexCheckReport {
    /// Build the report from the two answers the CLI already holds.
    pub fn new(freshness: &crate::index::Freshness, counts: DiagnosticCounts) -> Self {
        Self::with_unwitnessed(freshness, counts, UnwitnessedCounts::default())
    }

    /// As [`Self::new`], carrying the spec 057 tally.
    pub fn with_unwitnessed(
        freshness: &crate::index::Freshness,
        counts: DiagnosticCounts,
        unwitnessed: UnwitnessedCounts,
    ) -> Self {
        match freshness {
            crate::index::Freshness::Fresh => Self {
                fresh: true,
                expected: None,
                actual: None,
                skipped_shards: counts.unreadable.len(),
                diagnostics: counts,
                unwitnessed,
            },
            crate::index::Freshness::Stale { expected, actual } => Self {
                fresh: false,
                expected: Some(expected.clone()),
                actual: Some(actual.clone()),
                skipped_shards: counts.unreadable.len(),
                diagnostics: counts,
                unwitnessed,
            },
        }
    }
}
