//! `spec-spine verify <id>`: run a spec's declared acceptance (spec 049).
//!
//! The engine parses; this module executes. That split is spec 049 §3.1 and it
//! is the same seam spec 005 draws for `git`: everything that touches a
//! process lives here, so `spec_spine_core` stays a pure function of
//! `(config, file contents)`.
//!
//! **This command runs code the corpus declares** (spec 049 §3.6). That is safe
//! where the corpus and the operator share a trust domain, and it is why
//! `verify` is not part of the gate chain, which runs on branches whose
//! contents are in the general case a stranger's.

use std::path::Path;
use std::process::Command;

use spec_spine_core::verify;
use spec_spine_types::{
    Error, Severity, Verdict, VerifyFailure, VerifyOutcome, VerifyReport, Violation, verdict::verb,
};

use crate::load_repo_config;
use crate::out;

/// The ids currently being verified, innermost last, passed to every child.
///
/// Read from the environment rather than held in a global: the child is a
/// separate process, so the stack has to cross a process boundary to be seen at
/// all. The CLI already reads `SPEC_SPINE_PR_BODY` the same way.
const STACK_VAR: &str = "SPEC_SPINE_VERIFY_STACK";

/// A spec whose own `## Verification` section runs `verify` on itself.
///
/// Found by building this verb: spec 049's first draft carried
/// `spec-spine verify 049` in its own block, and one invocation forked 350
/// processes before it was killed. Nothing in the grammar forbids the line, and
/// the failure is unbounded rather than merely wrong, so the verb refuses it
/// instead of executing it (spec 049 3.7).
const RE_ENTRY_CODE: &str = "R-001";

/// Returns the exit code: `0` for `passed` and `not-declared`, `1` for
/// `failed`. `plan_only` prints what would run and returns `0` without
/// running anything.
///
/// A failing command's own exit code goes into the report, never into the
/// process's status: spec 049 §3.3 keeps `verify` inside the documented
/// `0`/`1`/`2`/`3` contract, so a command killed by a signal cannot make the
/// binary exit 137 the way the ported script did.
pub fn run(repo: &Path, id: &str, json: bool, plan_only: bool) -> Result<u8, Error> {
    let cfg = load_repo_config(repo)?;
    let plan = verify::plan(&cfg, repo, id)?;

    let mut stack: Vec<String> = std::env::var(STACK_VAR)
        .ok()
        .map(|s| {
            s.split(',')
                .filter(|p| !p.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    if stack.contains(&plan.spec_id) {
        stack.push(plan.spec_id.clone());
        return Err(Error::Validation(vec![Violation {
            code: RE_ENTRY_CODE.to_string(),
            severity: Severity::Error,
            message: format!(
                "verification re-entered itself: {}. A `## Verification` command \
                 that runs `verify` on its own spec recurses without bound.",
                stack.join(" -> ")
            ),
            path: Some(format!("{}/{}/spec.md", cfg.layout.specs_dir, plan.spec_id)),
        }]));
    }
    stack.push(plan.spec_id.clone());
    let child_stack = stack.join(",");

    // `--plan` answers "what would you run?" without running it. For the one
    // verb that executes what the corpus declares (spec 049 3.6), being able
    // to read the plan first is a safety affordance, not a convenience.
    if plan_only {
        if json {
            let value = serde_json::to_value(&plan).map_err(|e| Error::Schema(e.to_string()))?;
            out::verdict(&Verdict::report(verb::VERIFY, 0, value))?;
        } else {
            for command in &plan.commands {
                outln!("{command}");
            }
        }
        return Ok(0);
    }

    let total = plan.commands.len();
    let declared = plan.is_declared();

    if !json {
        for s in &plan.skipped {
            outln!(
                "verify: {}: {} {} block(s) are driven by the orchestrator; skipped here",
                plan.spec_id,
                s.count,
                s.tag
            );
        }
    }

    let mut ran = 0usize;
    let mut failure = None;
    for (i, command) in plan.commands.iter().enumerate() {
        if !json {
            outln!("[verify] $ {command}");
        }
        let status = Command::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(repo)
            .env(STACK_VAR, &child_stack)
            .status()
            .map_err(|e| Error::Io(format!("cannot run `{command}`: {e}")))?;
        ran += 1;
        if !json {
            match status.code() {
                Some(c) => outln!("[verify] exit {c}"),
                None => outln!("[verify] killed by signal"),
            }
        }
        if !status.success() {
            failure = Some(VerifyFailure {
                index: i + 1,
                command: command.clone(),
                exit_code: status.code(),
            });
            break;
        }
    }

    let outcome = match (&failure, declared) {
        (Some(_), _) => VerifyOutcome::Failed,
        (None, true) => VerifyOutcome::Passed,
        (None, false) => VerifyOutcome::NotDeclared,
    };
    let code = outcome.exit_code();

    let report = VerifyReport {
        spec_id: plan.spec_id.clone(),
        declared,
        outcome,
        ran,
        total,
        skipped: plan.skipped.clone(),
        failure,
    };

    if json {
        let value = serde_json::to_value(&report).map_err(|e| Error::Schema(e.to_string()))?;
        out::verdict(&Verdict::report(verb::VERIFY, code, value))?;
        return Ok(code);
    }

    match report.outcome {
        VerifyOutcome::NotDeclared => outln!(
            "verify: {}: not-declared (no verify:cli commands under ## Verification)",
            report.spec_id
        ),
        VerifyOutcome::Passed => outln!("verify: {}: passed ({total} command(s))", report.spec_id),
        VerifyOutcome::Failed => {
            let f = report.failure.as_ref().expect("failed implies a failure");
            eprintln!(
                "verify: {}: FAILED at command {} (exit {})",
                report.spec_id,
                f.index,
                f.exit_code
                    .map_or_else(|| "signal".to_string(), |c| c.to_string())
            );
        }
    }
    Ok(code)
}
