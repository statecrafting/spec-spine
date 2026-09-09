//! `spec-spine check`: both freshness reads, one verb (spec 075).
//!
//! The session protocol asks one question, "is the committed state current",
//! and had to know two spellings to ask it: `compile --check` is a flag where
//! `index check` is a subcommand. Every consumer carried both, `AGENTS.md`, the
//! `SessionStart` hook, the skills, the Makefile and CI among them.
//!
//! This verb is **additive**. Neither primitive changes, and a caller that
//! regenerated only one tree can still ask about that tree alone. What goes
//! down is the number of things the protocol has to know.

use std::path::Path;

use spec_spine_core::CheckReport;
use spec_spine_types::{Error, verdict::Verdict, verdict::verb};

use crate::load_repo_config;
use crate::out;

/// Run both freshness reads and compose one exit code.
///
/// Never writes. A verb the protocol calls to read the committed state cannot
/// repair that state as a side effect of reading it: doing so hides that the
/// *committed* copy was stale, so the drift reads as an uncommitted local edit
/// instead of a defect already on the branch, which is exactly how the spec
/// 017/021 drift reached the default branch.
pub fn run(repo: &Path, fail_on_unresolved: bool, json: bool) -> Result<u8, Error> {
    let cfg = load_repo_config(repo)?;
    // An `Err` from either half propagates, and `Error::exit_code()` spends 3
    // on it. That is the top of the precedence in 3.3, and it is the right
    // shape: a read that could not be performed has not answered, so no verdict
    // from the other tree makes the overall answer trustworthy.
    let report = spec_spine_core::check_report(&cfg, repo)?;
    let code = exit_code(&report, fail_on_unresolved);

    if json {
        let value = serde_json::to_value(&report).map_err(|e| Error::Schema(e.to_string()))?;
        out::verdict(&Verdict::report(verb::CHECK, code, value))?;
        return Ok(code);
    }

    report_registry(&report);
    report_index(&report, fail_on_unresolved);
    Ok(code)
}

/// The composed exit code: **`3` dominates `1` dominates `2` dominates `0`**
/// (spec 075 §3.3).
///
/// `3` is not reachable here because an unperformed read is an `Err` that never
/// arrives at this function; it is named in the order because the order is the
/// contract, and a reader checking the fold should find all four rungs.
///
/// `1` outranks `2` because **staleness is not meaningful against a corpus that
/// does not validate**. `AGENTS.md` already draws that conclusion for the
/// reporting case, requiring lifecycle counts to be reported as unverified when
/// validation fails; the exit code now agrees with the prose.
///
/// Pinned by test rather than only documented, because it is the one part of
/// this verb a caller cannot observe from a single run.
fn exit_code(report: &CheckReport, fail_on_unresolved: bool) -> u8 {
    let registry = if !report.registry.validation_passed {
        1
    } else if report.registry.fresh {
        0
    } else {
        2
    };
    let index = if !report.index.fresh {
        2
    } else if fail_on_unresolved && report.index.diagnostics.has_unresolved() {
        1
    } else {
        0
    };
    severity_max(registry, index)
}

/// The more severe of two exit codes under 3.3's order.
///
/// A rank table rather than `max`, because the numeric order is not the
/// severity order: `2` is numerically larger than `1` and less severe.
fn severity_max(a: u8, b: u8) -> u8 {
    let rank = |c: u8| match c {
        3 => 3,
        1 => 2,
        2 => 1,
        _ => 0,
    };
    if rank(a) >= rank(b) { a } else { b }
}

/// The registry half, attributed to its tree (spec 075 §3.4).
///
/// The stale report passes through with its structure intact: spec 031 §3.3
/// makes it contractual because the session protocol reads the drifted shard
/// names back to the operator, and exit 2 alone cannot say which shard moved.
fn report_registry(report: &CheckReport) {
    let r = &report.registry;
    if !r.validation_passed {
        eprintln!(
            "spec-registry: INVALID: the corpus fails validation, so staleness was not \
             computed (run `spec-spine compile --check` for the violations)"
        );
        return;
    }
    if r.fresh {
        outln!("spec-registry: fresh");
        return;
    }
    // stderr, so it surfaces in a CI log, and verbatim, so the shard names the
    // primitive named are the ones a reader sees.
    eprintln!("spec-registry: STALE");
    if let Some(actual) = &r.actual {
        eprintln!("{actual}");
    }
}

/// The index half, attributed to its tree (spec 075 §3.4).
fn report_index(report: &CheckReport, fail_on_unresolved: bool) {
    let i = &report.index;
    if !i.fresh {
        eprintln!("codebase-index: STALE (run `spec-spine index`)");
        if let Some(actual) = &i.actual {
            eprintln!("{actual}");
        }
        return;
    }
    let refused = fail_on_unresolved && i.diagnostics.has_unresolved();
    if refused {
        // Say the refusal beside the word "fresh", which is true on its own
        // axis: a reader seeing only "fresh" would take it for a pass.
        outln!(
            "codebase-index: fresh, but REFUSED: {} unresolved unit diagnostic(s) \
             (--fail-on-unresolved)",
            i.diagnostics.warnings + i.diagnostics.errors
        );
    } else {
        outln!("codebase-index: fresh");
    }
    if i.unwitnessed.total > 0 {
        outln!(
            "  unwitnessed claims: {} ({} allowed by [lint] unwitnessed_allowed)",
            i.unwitnessed.total,
            i.unwitnessed.allowed
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Spec 075 §3.3: the order is the contract, so it is pinned here rather
    /// than only described. A caller cannot observe a precedence from a single
    /// run, which is why documenting it would not have been enough.
    #[test]
    fn check_exit_order_is_three_one_two_zero() {
        // Every pair, in both argument orders, so the fold cannot be
        // accidentally asymmetric.
        for (a, b, want) in [
            (0, 0, 0),
            (0, 2, 2),
            (2, 0, 2),
            (0, 1, 1),
            (1, 0, 1),
            (2, 1, 1), // validation failure outranks staleness
            (1, 2, 1),
            (3, 1, 3),
            (1, 3, 3),
            (3, 2, 3),
            (2, 3, 3),
            (3, 0, 3),
        ] {
            assert_eq!(
                severity_max(a, b),
                want,
                "severity_max({a}, {b}) must be {want}"
            );
        }
    }
}
