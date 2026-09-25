//! `spec-spine check`: both freshness reads, one verb (spec 062).
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

use spec_spine_core::{CheckReport, Freshness, IndexFreshnessReport};
use spec_spine_types::{Error, verdict::Verdict, verdict::verb};

use crate::load_repo_config;
use crate::out;

/// Run both freshness reads and compose one exit code.
///
/// Never writes. A verb the protocol calls to read the committed state cannot
/// repair that state as a side effect of reading it: doing so hides that the
/// *committed* copy was stale, so the drift reads as an uncommitted local edit
/// instead of a defect already on the branch, which is exactly how the spec
/// 016/019 drift reached the default branch.
pub fn run(
    repo: &Path,
    fail_on_unresolved: bool,
    fail_on_warn: bool,
    json: bool,
) -> Result<u8, Error> {
    let cfg = load_repo_config(repo)?;
    // An `Err` from either half propagates, and `Error::exit_code()` spends 2
    // or 4 on it (spec 132). Either outranks every finding the fold below can
    // produce, which is the right shape: a read that could not be performed has
    // not answered, so no verdict from the other tree makes the overall answer
    // trustworthy.
    // Spec 079 §3.2: one read, two facts. The index half's blocking set and its
    // stale set arrive apart, so this verb can say which refusal it is holding
    // without indexing again and without reading back its own prose.
    let (report, freshness) = spec_spine_core::check_report_full(&cfg, repo)?;
    let code = exit_code(&report, &freshness, fail_on_unresolved, fail_on_warn);

    if json {
        let value = serde_json::to_value(&report).map_err(|e| Error::Internal(e.to_string()))?;
        out::verdict(&Verdict::report(verb::CHECK, code, value))?;
        return Ok(code);
    }

    report_registry(&report, fail_on_warn);
    report_index(&report, &freshness, fail_on_unresolved);
    Ok(code)
}

/// The composed exit code: the higher of the two halves (spec 132 §3.3,
/// amending spec 062 §3.3).
///
/// Under spec 132's contract every code is ordered by severity numerically:
/// `4` failed outranks `3` usage outranks `2` refused outranks `1` finding
/// outranks `0`. Staleness is a finding like a validation failure, so the two
/// halves' findings no longer need ranking against each other: either one is
/// `1`, and the report lines say which. `2` and `4` arrive as an `Err` before
/// this function runs; they are in the order because the order is the contract.
///
/// Pinned by test rather than only documented, because it is the one part of
/// this verb a caller cannot observe from a single run.
fn exit_code(
    report: &CheckReport,
    freshness: &IndexFreshnessReport,
    fail_on_unresolved: bool,
    fail_on_warn: bool,
) -> u8 {
    let registry = if !report.registry.validation_passed {
        1
    } else if fail_on_warn && report.registry.warnings > 0 {
        // Spec 064 §3.3: the forwarded refusal is a `1`, which lands inside the
        // fold below rather than altering it. It sits above freshness for the
        // same reason validation does: a refused corpus makes its own staleness
        // the less useful answer.
        1
    } else if report.registry.fresh {
        0
    } else {
        // Spec 132 §3.2: staleness is a finding.
        1
    };
    let index = if !freshness.blocking.is_empty() {
        // Spec 080 §3.1, amending spec 069 §3.1: an unresolved claim is a
        // validation failure, not staleness. The decision reads the partition
        // spec 079 §3.2 built rather than the composed `fresh` flag, which
        // cannot tell the two refusals apart: a spec claiming a unit that does
        // not resolve describes a corpus that does not match its tree, and
        // regenerating provably cannot clear it. Spending 2 here sent every
        // consumer that branches on the code to `spec-spine index`, forever.
        //
        // It is checked FIRST, so a tree holding both refusals exits 1. That is
        // spec 062 §3.3's order (1 dominates 2) and not a new rule; the report
        // still names both halves and still attributes regeneration to the
        // stale one alone.
        1
    } else if !report.index.fresh {
        1
    } else if fail_on_unresolved && report.index.diagnostics.has_unresolved() {
        // A different axis: the warning-tier W-001 / W-002 claims a spec makes
        // over territory it has not written yet (specs 023, 044). Unchanged.
        1
    } else {
        0
    };
    severity_max(registry, index)
}

/// The more severe of two exit codes: under spec 132 the numeric order is the
/// severity order, so this is `max`.
fn severity_max(a: u8, b: u8) -> u8 {
    a.max(b)
}

/// The registry half, attributed to its tree (spec 062 §3.4).
///
/// The stale report passes through with its structure intact: spec 028 §3.3
/// makes it contractual because the session protocol reads the drifted shard
/// names back to the operator, and exit 1 alone cannot say which shard moved.
fn report_registry(report: &CheckReport, fail_on_warn: bool) {
    let r = &report.registry;
    if !r.validation_passed {
        eprintln!(
            "spec-registry: INVALID: the corpus fails validation, so staleness was not \
             computed (run `spec-spine compile --check` for the violations)"
        );
        return;
    }
    // Spec 064 §3.4: name the count and the tree, so an exit 1 from this verb
    // is attributable. The pointer to `compile --check` for the individual
    // violations stays correct: that primitive still prints them.
    if fail_on_warn && r.warnings > 0 {
        outln!(
            "spec-registry: REFUSED: {} warning(s) (--fail-on-warn) \
             (run `spec-spine compile --check --fail-on-warn` for the violations)",
            r.warnings
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

/// The index half, attributed to its tree (spec 062 §3.4).
///
/// Spec 079 §3.3 splits the refusal this used to print one way. Staleness means
/// "the committed artifact is behind the source, regenerate it"; an unresolved
/// claim means "the spec and the tree disagree about what exists", which no
/// command repairs. Both exit 1 (spec 132), and the stale-only report is
/// unchanged, wording included (FR-008).
fn report_index(report: &CheckReport, freshness: &IndexFreshnessReport, fail_on_unresolved: bool) {
    let i = &report.index;
    if !i.fresh {
        if !freshness.stale.is_empty() {
            eprintln!("codebase-index: STALE (run `spec-spine index`)");
            if let Freshness::Stale { actual, .. } = freshness.stale_verdict() {
                // Spec 076 §3.3: a shard the diagnostics tally could not read is
                // named on its drift line, not left to the payload count alone.
                eprintln!(
                    "{}",
                    spec_spine_core::annotate_unreadable(&actual, &i.diagnostics.unreadable)
                );
            }
        }
        if !freshness.blocking.is_empty() {
            eprintln!(
                "codebase-index: UNRESOLVED CLAIM: {}",
                freshness.unresolved_claim_summary()
            );
            for line in freshness.unresolved_claim_lines() {
                eprintln!("{line}");
            }
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

    /// Spec 062 §3.3: the order is the contract, so it is pinned here rather
    /// than only described. A caller cannot observe a precedence from a single
    /// run, which is why documenting it would not have been enough.
    /// Spec 132 §3.3. Named with the `check_exit_order` prefix 062's
    /// acceptance filters on, so that filter still runs a test.
    #[test]
    fn check_exit_order_is_the_higher_code() {
        // Every pair, in both argument orders, so the fold cannot be
        // accidentally asymmetric.
        for a in 0u8..=4 {
            for b in 0u8..=4 {
                assert_eq!(severity_max(a, b), a.max(b), "severity_max({a}, {b})");
            }
        }
        // The two findings a tree can hold at once fold to one finding.
        assert_eq!(severity_max(1, 1), 1);
    }
}
