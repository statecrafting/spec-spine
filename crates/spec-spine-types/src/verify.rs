//! Declared-acceptance DTOs (spec 049): what a spec's `## Verification`
//! section asks to be run, and what happened when it was.
//!
//! Two shapes, because spec 049 §3.1 splits the work across the library
//! boundary. [`VerifyPlan`] is what the engine produces: a pure read of
//! authored markdown, naming the commands and the fence tags it declined.
//! [`VerifyReport`] is what the CLI produces after running them. The engine
//! never spawns a process, so it never builds a [`VerifyReport`].
//!
//! These are read-side report shapes, not committed artifacts: nothing here is
//! written under the derived directory, so no schema version carries them. The
//! `--json` envelope around a report is versioned by `VERDICT_SCHEMA_VERSION`.

use serde::{Deserialize, Serialize};

/// The commands a spec declares, in document order.
///
/// A pure function of the spec's markdown. `commands` holds only runnable
/// lines: blanks and comments are dropped by the parser, not carried here for
/// a caller to re-filter.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyPlan {
    /// The resolved spec id (the full directory name, even when the caller
    /// passed the short form spec 016 defines).
    pub spec_id: String,
    /// Every runnable line of every `verify:cli` fence, in document order.
    pub commands: Vec<String>,
    /// Fence tags found under `## Verification` that this tool does not run,
    /// sorted by tag. Reported rather than silently ignored: a caller is
    /// entitled to know work was declined.
    pub skipped: Vec<SkippedBlocks>,
}

impl VerifyPlan {
    /// Whether the spec declares executable acceptance at all.
    ///
    /// False for both "no `## Verification` section" and "a section holding no
    /// `verify:cli` command": spec 049 §3.2 makes those one outcome, because
    /// the distinction is invisible to a caller deciding whether it has
    /// evidence.
    pub fn is_declared(&self) -> bool {
        !self.commands.is_empty()
    }
}

/// One declined fence tag and how many blocks carried it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkippedBlocks {
    /// The fence tag as written, without the backticks (e.g. `verify:browser`).
    pub tag: String,
    /// How many blocks in this spec carried it.
    pub count: usize,
}

/// What happened when a [`VerifyPlan`] was run.
///
/// Built by the CLI, never by the engine. Carries the failing command's own
/// exit code in [`VerifyFailure::exit_code`] rather than in the process's exit
/// status, which spec 049 §3.3 constrains to the documented `0`/`1`/`2`/`3`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyReport {
    pub spec_id: String,
    /// Mirrors [`VerifyPlan::is_declared`]. Redundant against `outcome` on
    /// purpose: a consumer branching on either reads the same fact.
    pub declared: bool,
    pub outcome: VerifyOutcome,
    /// How many commands actually ran. Equals `total` unless one failed.
    pub ran: usize,
    /// How many the plan held.
    pub total: usize,
    pub skipped: Vec<SkippedBlocks>,
    /// Present exactly when `outcome` is [`VerifyOutcome::Failed`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure: Option<VerifyFailure>,
}

/// The three ends a verification run can reach (spec 049 §3.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VerifyOutcome {
    /// At least one command ran and every one exited 0.
    Passed,
    /// Nothing to run. An honest zero, not a pass.
    NotDeclared,
    /// A command exited non-zero; later commands did not run.
    Failed,
}

impl VerifyOutcome {
    /// The process exit code this outcome maps to.
    ///
    /// `not-declared` is 0 by design: absence of declared acceptance is not a
    /// failure of the verb, and a corpus is entitled to specs that declare
    /// none.
    pub fn exit_code(self) -> u8 {
        match self {
            Self::Passed | Self::NotDeclared => 0,
            Self::Failed => 1,
        }
    }
}

/// The command that stopped the run.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyFailure {
    /// 1-based position in the plan, matching what the transcript printed.
    pub index: usize,
    pub command: String,
    /// The command's own exit code, or `None` when a signal killed it and the
    /// platform reports no code. Never the process's exit status.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
}
