//! Ownership-coverage DTOs (spec 032): the file-granular answer to "how much
//! of the code does the corpus specifically claim?", emitted by `spec-spine
//! index coverage` and by the `coverage_json` facade.
//!
//! These are read-side report shapes, not committed artifacts: nothing here is
//! written under the derived directory, so no schema version carries them.

use serde::{Deserialize, Serialize};

/// The whole-tree coverage report. Every path is repo-relative POSIX; every
/// list is sorted and deduplicated, so the report is byte-identical across
/// platforms for one tree.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverageReport {
    /// Source files enumerated across every discovered package: the
    /// denominator. Same universe the coupling gate's `C-002` examines.
    pub source_files: usize,
    /// Files with a **specific** owner: a resolved ownership-bearing unit or a
    /// `// Spec:` comment header covers them.
    pub claimed_files: usize,
    /// Files whose only owner is a package's manifest floor
    /// (`[package.metadata.<ns>].spec`, spec 005 §3.6).
    pub floor_only_files: Vec<String>,
    /// Files no spec owns at all.
    pub unclaimed_files: Vec<String>,
    /// Per-package breakdown, sorted by package path.
    pub packages: Vec<PackageCoverage>,
}

impl CoverageReport {
    /// Files `[coupling] require_ownership` would refuse: floor-only plus
    /// unclaimed.
    pub fn untraced_files(&self) -> usize {
        self.floor_only_files.len() + self.unclaimed_files.len()
    }

    /// True when every source file has a specific owner (the state
    /// `require_ownership` defends, and what `--fail-on-untraced` asserts).
    pub fn is_fully_claimed(&self) -> bool {
        self.untraced_files() == 0
    }
}

/// One discovered package's share of the report. Counts only; the file lists
/// live on the aggregate report (their repo-relative paths attribute them).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageCoverage {
    /// Repo-relative POSIX path of the package directory (`""` for a root
    /// package).
    pub path: String,
    /// The floor spec named in the package manifest, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub floor_spec: Option<String>,
    pub source_files: usize,
    pub claimed_files: usize,
    pub floor_only: usize,
    pub unclaimed: usize,
}
