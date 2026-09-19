//! `spec-spine verify <id>`: run a spec's declared acceptance (spec 049).
//!
//! The engine parses; this module executes. That split is spec 049 §3.1 and it
//! is the same seam spec 005 draws for `git`: everything that touches a
//! process lives here, so `spec_spine_core` stays a pure function of
//! `(config, file contents)`.
//!
//! Under `--json` stdout belongs to the one verdict envelope (spec 037 §3.1), so
//! an acceptance command's own output is forwarded to stderr rather than
//! inherited, and the transcript spec 049 §3.5 requires goes there with it
//! (spec 118). Without the flag every byte goes where it always has.
//!
//! **This command runs code the corpus declares** (spec 049 §3.6). That is safe
//! where the corpus and the operator share a trust domain, and it is why
//! `verify` is not part of the gate chain, which runs on branches whose
//! contents are in the general case a stranger's.

use std::io::{ErrorKind, Read, Write};
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};

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

/// Write one transcript line to the channel the mode assigns it (spec 118 §3.3).
///
/// Spec 049 §3.5 requires the echo and names no channel. Without `--json` it is
/// stdout, which is what `verify` has printed since 049 shipped. Under `--json`
/// stdout carries the one verdict envelope and nothing else (spec 037 §3.1), so
/// the transcript joins the child's own bytes on stderr, where spec 035 §3.3
/// puts every CLI diagnostic. Reading it as a stdout requirement is what made
/// it vanish under `--json` altogether, which is the mode a CI log is most
/// likely to be produced in.
fn transcript(json: bool, args: std::fmt::Arguments<'_>) {
    if json {
        diagnostic(args);
    } else {
        out::line(args);
    }
}

/// Write one line to the parent's stderr, best effort.
///
/// `eprintln!` unwraps its write, so a consumer that read the opening
/// transcript line and then closed the parent's stderr made `verify --json`
/// exit **101** with an empty stdout: the next `[verify] exit 0` panicked
/// before the envelope was written. That is spec 035 §3.2's argument about
/// stdout, met again on the channel spec 118 §3.3 moved this mode's
/// diagnostics to, and it has the same answer. A diagnostic that cannot be
/// delivered is dropped; it never decides the verdict and it never decides the
/// exit code (spec 118 D-3).
fn diagnostic(args: std::fmt::Arguments<'_>) {
    let stderr = std::io::stderr();
    let mut handle = stderr.lock();
    let _ = writeln!(handle, "{args}");
}

/// The size of the drain buffer. Fixed, so spec 118 §3.2's bound holds: the
/// verb does not grow with the child's output.
const DRAIN_BUF: usize = 16 * 1024;

/// How a drain of one child stream ended (spec 118 D-5).
///
/// The three are deliberately not one value. Spec 118 D-3 excuses exactly one
/// of them, and reporting the other two as that one would assert something
/// about the parent's stderr that was never observed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Drained {
    /// Read to EOF, every byte delivered to the parent's stderr.
    Delivered,
    /// Read to EOF, but the parent's stderr stopped accepting bytes partway and
    /// the remainder was discarded. This is the condition spec 118 D-3 names.
    Discarded,
    /// The pipe could not be read to EOF. Nothing is known about what the child
    /// still had to say, and D-3 does not speak to this case.
    InputFailed,
}

/// Read `src` to end-of-file, writing what it yields to the parent's stderr and
/// discarding the rest once stderr stops accepting bytes (spec 118 D-3, D-4).
///
/// Reading continues past a destination failure, and that is the whole point.
/// An `io::copy` returns on the first failed write, which left the child's pipe
/// undrained while `wait` blocked on the child: a command writing more than a
/// pipe buffer then blocked on its own write forever, and `verify` hung with
/// it. Measured at `71a423a`: a child writing ~1.3 MB to stderr completes in
/// ~0.5 s with the consumer open and does not complete at all once the consumer
/// closes. Draining to EOF costs nothing when the destination is healthy and is
/// the only thing that lets the child finish when it is not.
///
/// The pipe is not closed early either. Dropping it would hand the child an
/// `EPIPE` on its next write and turn a failure to deliver logs into a change
/// of the child's exit status, which is precisely the outcome spec 118 D-3
/// forbids: the verdict is computed from exit statuses, so it must not depend
/// on whether the parent could write its logs anywhere.
///
/// Memory stays bounded: one fixed buffer per stream, nothing accumulated,
/// which is the same guarantee `io::copy` gave and spec 118 §3.2 requires.
fn drain_to_stderr<R: Read>(src: &mut R) -> Drained {
    let mut buf = [0u8; DRAIN_BUF];
    let mut outcome = Drained::Delivered;
    loop {
        let n = match src.read(&mut buf) {
            Ok(0) => return outcome,
            Ok(n) => n,
            Err(e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(_) => return Drained::InputFailed,
        };
        if outcome == Drained::Delivered {
            let stderr = std::io::stderr();
            let mut handle = stderr.lock();
            if handle.write_all(&buf[..n]).is_err() {
                outcome = Drained::Discarded;
            }
        }
    }
}

/// Say, on stderr, what a drain that was neither complete nor merely
/// undeliverable actually was.
///
/// `Discarded` is silent on purpose: it means the parent's stderr is gone, so
/// there is no channel left to report it on, and spec 118 D-3 already rules it
/// out as a failure of the verb. The other two are different facts. A thread
/// that panicked is a defect in this CLI, and a pipe that could not be read is
/// an operating-system failure; neither is evidence that the parent's stderr
/// stopped accepting bytes, and stderr is very likely still working, so the
/// verb says so rather than filing all three under D-3. None of the three
/// changes the verdict, which spec 118 D-3 computes from exit statuses alone.
fn note_drain(stream: &str, command: &str, drained: &std::thread::Result<Drained>) {
    let what = match drained {
        Ok(Drained::Delivered | Drained::Discarded) => return,
        Ok(Drained::InputFailed) => "could not be read to end",
        Err(_) => "forwarding panicked",
    };
    diagnostic(format_args!(
        "[verify] warning: the child's {stream} {what} while running `{command}`; its output is incomplete and the verdict is unaffected"
    ));
}

/// Run one acceptance command from the repository root and return its status.
///
/// Without `--json` the child inherits both of the parent's streams, which is
/// spec 049's shipped behaviour and stays byte for byte what it was. Under
/// `--json` the parent's stdout is reserved for the verdict envelope, so the
/// child is given pipes and both of its streams are forwarded to the parent's
/// stderr (spec 118 §3.1). Diverting at the spawn rather than filtering at the
/// write is what makes that total: the report path and the error path are then
/// covered by the same fact, that the child never holds the stdout descriptor.
///
/// The forwarding is concurrent with the child and with itself (spec 118 §3.2).
/// A pipe buffer is finite, so a forwarder that waited for the child to exit
/// would deadlock on the first command verbose enough to fill one, and
/// `cargo test` over a workspace is that command. Nothing is accumulated:
/// `io::copy` streams through a fixed buffer, so a command's output may be
/// megabytes without the verb growing with it. The price is that the two pipes
/// are buffered independently, so their interleaving is not preserved; spec 118
/// §3.2 promises no ordering between them for exactly that reason.
///
/// A failed write of the forwarded bytes discards the rest of that stream and
/// is not a failure of the verb (spec 118 D-3, following spec 035 §3.3): a
/// process whose stderr has gone has no channel left to report the fact, and
/// the verdict is computed from exit statuses, which are unaffected. Reading
/// does not stop with delivery, and neither pipe is closed early: spec 118 D-4
/// separates the obligation to deliver the bytes, which D-3 excuses, from the
/// obligation to drain the pipe, which nothing excuses, because a child blocked
/// on a pipe nobody is reading never reaches an exit status at all.
fn run_one(repo: &Path, command: &str, child_stack: &str, json: bool) -> Result<ExitStatus, Error> {
    let mut cmd = Command::new("sh");
    cmd.arg("-c")
        .arg(command)
        .current_dir(repo)
        .env(STACK_VAR, child_stack);

    if !json {
        return cmd
            .status()
            .map_err(|e| Error::Io(format!("cannot run `{command}`: {e}")));
    }

    let mut child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| Error::Io(format!("cannot run `{command}`: {e}")))?;

    // The child's stdout gets a thread of its own; its stderr is drained on
    // this one. Two readers, so neither pipe can block the other, and each
    // reads to EOF whether or not its bytes can be delivered.
    let mut child_out = child.stdout.take().expect("stdout was piped");
    let pump = std::thread::spawn(move || drain_to_stderr(&mut child_out));
    let mut child_err = child.stderr.take().expect("stderr was piped");
    // Wrapped so `note_drain` reads both results through one shape. The `Err`
    // arm is unreachable for this one by construction: a panic here would
    // unwind `run_one` itself rather than be caught, so only the pump thread
    // can ever report `Err`. That is a property of where the drain runs, not a
    // claim that this stream cannot fail, and `InputFailed` still reaches
    // `note_drain` from here.
    let err_drained = Ok(drain_to_stderr(&mut child_err));

    // The wait's result is held rather than propagated, so the join happens on
    // the failing path too. With `?` here the thread outlived a `wait` error,
    // and the ordering this comment claims held on every path but that one.
    //
    // The stderr drain has reached EOF by this point; the stdout drain is still
    // running on the pump thread and finishes on its own schedule. What matters
    // is not that either has finished but that both are reading concurrently
    // with the child, so neither pipe can be left full while this wait runs,
    // whatever the destination did with the bytes.
    let waited = child.wait();
    // Joined before returning, so every forwarded byte is on stderr ahead of
    // this command's `exit` line and the next command's transcript.
    let out_drained = pump.join();
    note_drain("stdout", command, &out_drained);
    note_drain("stderr", command, &err_drained);
    let status = waited.map_err(|e| Error::Io(format!("cannot wait for `{command}`: {e}")))?;
    Ok(status)
}

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
        return Err(Error::Validation(vec![
            Violation::new(
                RE_ENTRY_CODE,
                Severity::Error,
                format!(
                    "verification re-entered itself: {}. A `## Verification` command \
                     that runs `verify` on its own spec recurses without bound.",
                    stack.join(" -> ")
                ),
            )
            .at(format!("{}/{}/spec.md", cfg.layout.specs_dir, plan.spec_id)),
        ]));
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
        // Spec 103 §3.4: a block running under another spec's name, with no
        // line saying so, is the laundering shape spec 040 §3.2 refuses. The
        // whole value of amending rather than editing is that both documents
        // stay readable, and that is worth nothing if the reader is not told to
        // look at the second one.
        if let Some(from) = &plan.acceptance_from {
            outln!("verify: {}", plan.spec_id);
            outln!("  acceptance amended by {from} (spec 040); its block is the one that runs");
        }
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
        transcript(json, format_args!("[verify] $ {command}"));
        let status = run_one(repo, command, &child_stack, json)?;
        ran += 1;
        match status.code() {
            Some(c) => transcript(json, format_args!("[verify] exit {c}")),
            None => transcript(json, format_args!("[verify] killed by signal")),
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
