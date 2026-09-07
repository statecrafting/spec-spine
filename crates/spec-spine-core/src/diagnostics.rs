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

use spec_spine_types::{Config, Diagnostic, Error, Severity};

use crate::index::read_committed_index_shards;

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
pub fn committed_counts(cfg: &Config, repo_root: &Path) -> Result<DiagnosticCounts, Error> {
    let (spec_shards, _) = read_committed_index_shards(cfg, repo_root)?;
    let mut counts = DiagnosticCounts::default();
    for sh in &spec_shards {
        for d in &sh.diagnostics.errors {
            counts.errors += 1;
            *counts.by_code.entry(d.code.clone()).or_insert(0) += 1;
        }
        for d in &sh.diagnostics.warnings {
            counts.warnings += 1;
            *counts.by_code.entry(d.code.clone()).or_insert(0) += 1;
        }
    }
    Ok(counts)
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
}

impl IndexCheckReport {
    /// Build the report from the two answers the CLI already holds.
    pub fn new(freshness: &crate::index::Freshness, counts: DiagnosticCounts) -> Self {
        match freshness {
            crate::index::Freshness::Fresh => Self {
                fresh: true,
                expected: None,
                actual: None,
                diagnostics: counts,
            },
            crate::index::Freshness::Stale { expected, actual } => Self {
                fresh: false,
                expected: Some(expected.clone()),
                actual: Some(actual.clone()),
                diagnostics: counts,
            },
        }
    }
}
