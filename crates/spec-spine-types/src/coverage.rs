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
    /// Territory a spec has declared it will own and has not written yet
    /// (spec 076 §3.6), as `<spec id>: <unit identity>` entries, sorted.
    ///
    /// **Not** part of the numerator or the denominator, and deliberately so.
    /// These paths are not on disk, so they are not source files; counting a
    /// planned claim as coverage would let a spec satisfy `--fail-on-untraced`
    /// by declaring an intention. What this answers is the question the report
    /// could not answer before: a file nothing claims and a file something has
    /// planned now look different to a reader.
    ///
    /// Omitted when empty, so a corpus using no planned units emits exactly
    /// what it did before.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub planned_territory: Vec<String>,
    /// Comment headers that tried to claim their file and did not (spec 094
    /// §3.3, §3.4), sorted by path then line.
    ///
    /// Explains a classification and never alters one: a file listed here is
    /// counted exactly as it would be without the list, and
    /// `--fail-on-untraced` does not read it. Omitted when empty, following
    /// `planned_territory`, so a corpus with none emits what it did before.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub near_miss_headers: Vec<NearMissHeader>,
}

/// One comment header that did not claim its file (spec 094 §3.3).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NearMissHeader {
    /// Repo-relative POSIX path of the file.
    pub path: String,
    /// 1-based line number of the header.
    pub line: usize,
    pub reason: NearMissReason,
    /// The spec the reference resolves to, when it resolves. Absent for
    /// `unknown-spec` by definition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spec_id: Option<String>,
}

/// Why a header claimed nothing (spec 094 §3.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NearMissReason {
    /// A resolving header below the claim window, inside the report window, in
    /// a file that claimed nothing inside the claim window.
    OutsideWindow,
    /// The first header inside the claim window names no spec in the corpus.
    /// The scan stops there, so it also shadows any header below it.
    UnknownSpec,
    /// `//! Spec:` inside the claim window: an inner doc comment is prose, and
    /// the scanner deliberately does not read it as a claim.
    DocCommentMarker,
}

impl NearMissReason {
    /// The kebab-case spelling the payload uses, for the prose form.
    pub fn as_str(self) -> &'static str {
        match self {
            NearMissReason::OutsideWindow => "outside-window",
            NearMissReason::UnknownSpec => "unknown-spec",
            NearMissReason::DocCommentMarker => "doc-comment-marker",
        }
    }
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
