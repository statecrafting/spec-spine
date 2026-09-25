// Spec: specs/090-the-verdict-is-the-only-thing-on-stdout/spec.md
//! `spec-spine verify`'s two output channels (spec 090).
//!
//! Every fixture here runs a command that writes to **both** of its streams,
//! which is the property spec 090 §1.2 found missing from the `verify` cases in
//! `cli.rs`: each of those runs `true`, `exit 7` or a fence that is never
//! executed, so not one of them writes a byte, and the assertion that stdout is
//! one envelope could not fail against them.
//!
//! What is asserted is the channel, not the verdict. The verdict, the report
//! fields and the exit-code mapping are spec 043's and are covered in `cli.rs`;
//! the cases below assert that under `--json` stdout carries exactly one
//! envelope, that the child's bytes and the transcript reach stderr instead,
//! and that without the flag none of that moves.
//!
//! Markers are assembled by the child at run time (`printf 'O%sT-A\n' U`), never
//! written literally in the command. A literal marker appears in the transcript
//! and in `failure.command`, so `stderr.contains("OUT-A")` would pass on the
//! echo of the command rather than on the forwarded byte, and
//! `!stdout.contains("OUT-A")` would fail on the envelope's own payload. Both
//! mistakes were made and measured while building this file.

use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::process::{Command, Output};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, Instant};

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_spec-spine"))
}

fn code(out: &Output) -> i32 {
    out.status.code().unwrap_or(-1)
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// Write a spec whose `## Verification` section holds one `verify:cli` fence
/// with `commands` as its body.
fn write_spec(root: &Path, dir: &str, commands: &str) {
    let spec_dir = root.join("specs").join(dir);
    fs::create_dir_all(&spec_dir).unwrap();
    let body = format!(
        "---\nid: \"{dir}\"\ntitle: \"T\"\nstatus: approved\ncreated: \"2026-09-19\"\nsummary: \"s\"\n---\n# {dir}\n\n## Verification\n\n```verify:cli\n{commands}\n```\n"
    );
    fs::write(spec_dir.join("spec.md"), body).unwrap();
}

fn run(root: &Path, args: &[&str]) -> Output {
    bin().arg("--repo").arg(root).args(args).output().unwrap()
}

/// Parse the WHOLE of stdout as exactly one JSON document. `from_slice` refuses
/// trailing content, so a byte in front of the envelope or after it fails here,
/// which is the defect spec 090 §1.1 measured.
fn one_envelope(out: &Output) -> serde_json::Value {
    serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!(
            "the whole of stdout must be one verdict envelope: {e}\n--- stdout ---\n{}\n--- stderr ---\n{}",
            stdout(out),
            stderr(out)
        )
    })
}

/// Spec 090 §3.1, §3.2, §3.3 on a passing command: stdout is one envelope, both
/// of the child's streams are on stderr, and the transcript is there too.
#[test]
fn json_keeps_a_passing_commands_output_off_stdout() {
    let tmp = tempfile::tempdir().unwrap();
    write_spec(
        tmp.path(),
        "001-noisy",
        "printf 'O%sT-A\\n' U; printf 'E%sR-A\\n' R >&2",
    );

    let out = run(tmp.path(), &["verify", "001-noisy", "--json"]);
    assert_eq!(code(&out), 0);

    let v = one_envelope(&out);
    assert_eq!(v["verb"], "verify");
    assert_eq!(v["outcome"], "ok");
    assert_eq!(v["exitCode"], 0);
    assert_eq!(v["report"]["outcome"], "passed");
    assert_eq!(v["report"]["ran"], 1);
    assert_eq!(v["report"]["total"], 1);

    let so = stdout(&out);
    let se = stderr(&out);
    assert!(
        !so.contains("OUT-A"),
        "child stdout must not be on stdout: {so}"
    );
    assert!(
        !so.contains("ERR-A"),
        "child stderr must not be on stdout: {so}"
    );
    assert!(se.contains("OUT-A"), "child stdout must be on stderr: {se}");
    assert!(se.contains("ERR-A"), "child stderr must be on stderr: {se}");
    // The transcript spec 043 §3.5 requires, on the channel spec 090 §3.3
    // assigns it. Under `--json` it was suppressed entirely before this spec.
    assert!(
        se.contains("[verify] $ printf 'O%sT-A\\n' U; printf 'E%sR-A\\n' R >&2"),
        "the command line belongs on stderr: {se}"
    );
    assert!(se.contains("[verify] exit 0"), "the exit line too: {se}");
}

/// Spec 090 §3.1, §3.4 on a failure: the channel holds, the failure detail is
/// retained, and the next command in the block does not run.
#[test]
fn json_keeps_a_failing_commands_output_off_stdout_and_stops_there() {
    let tmp = tempfile::tempdir().unwrap();
    write_spec(
        tmp.path(),
        "002-fail",
        "printf 'O%sT-B\\n' U; printf 'E%sR-B\\n' R >&2; exit 7\nprintf c > ran-c.txt",
    );

    let out = run(tmp.path(), &["verify", "002-fail", "--json"]);
    // Spec 043 §3.3: the drift-tier 1, never the command's own 7.
    assert_eq!(code(&out), 1);

    let v = one_envelope(&out);
    assert_eq!(v["exitCode"], 1);
    assert_eq!(v["report"]["outcome"], "failed");
    assert_eq!(v["report"]["failure"]["exitCode"], 7);
    assert_eq!(v["report"]["failure"]["index"], 1);
    assert_eq!(
        v["report"]["failure"]["command"],
        "printf 'O%sT-B\\n' U; printf 'E%sR-B\\n' R >&2; exit 7"
    );
    assert_eq!(v["report"]["ran"], 1);
    assert_eq!(v["report"]["total"], 2);

    let so = stdout(&out);
    let se = stderr(&out);
    assert!(!so.contains("OUT-B"), "{so}");
    assert!(!so.contains("ERR-B"), "{so}");
    assert!(se.contains("OUT-B"), "{se}");
    assert!(se.contains("ERR-B"), "{se}");
    assert!(se.contains("[verify] exit 7"), "{se}");
    // Asserted by the absence of a file side effect rather than by counting
    // output: the output channel is the thing under test here, so it cannot
    // also be the witness.
    assert!(
        !tmp.path().join("ran-c.txt").exists(),
        "a later command must not run after a failure"
    );
}

/// Spec 090 §3.1's error path: the `R-001` refusal of spec 043 §3.7 is one
/// error envelope and the whole of stdout, with nothing beside it.
#[test]
fn json_error_envelope_is_the_whole_of_stdout() {
    let tmp = tempfile::tempdir().unwrap();
    write_spec(tmp.path(), "004-loop", "printf 'never-runs\\n'");

    let out = bin()
        .arg("--repo")
        .arg(tmp.path())
        .args(["verify", "004-loop", "--json"])
        .env("SPEC_SPINE_VERIFY_STACK", "004-loop")
        .output()
        .unwrap();
    assert_eq!(code(&out), 1);

    let v = one_envelope(&out);
    assert_eq!(v["error"]["kind"], "validation");
    assert_eq!(v["error"]["violations"][0]["code"], "R-001");
    assert!(
        v.get("report").is_none(),
        "`report` and `error` are exclusive (spec 034 §3.1): {v}"
    );
    // The refusal precedes execution, so nothing was run; what this asserts is
    // that the error path carries one envelope and no prose.
    assert!(!stdout(&out).contains("never-runs"), "{}", stdout(&out));
}

/// Spec 090 §3.4: `--plan --json` is one envelope and spawns nothing.
#[test]
fn plan_json_is_one_envelope_and_runs_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    write_spec(
        tmp.path(),
        "003-plan",
        "printf ran > side_effect.txt; printf 'ERR-C\\n' >&2",
    );

    let out = run(tmp.path(), &["verify", "003-plan", "--plan", "--json"]);
    assert_eq!(code(&out), 0);

    let v = one_envelope(&out);
    assert_eq!(v["verb"], "verify");
    assert_eq!(
        v["report"]["commands"][0],
        "printf ran > side_effect.txt; printf 'ERR-C\\n' >&2"
    );
    assert!(
        !tmp.path().join("side_effect.txt").exists(),
        "--plan must run nothing"
    );
    assert!(!stderr(&out).contains("ERR-C"), "{}", stderr(&out));
}

/// Spec 090 §3.4's preservation half: without `--json` the child inherits both
/// streams and the transcript is on stdout, exactly as spec 043 shipped it.
/// Green before spec 090 on purpose; this is what stops the correction from
/// moving the prose mode too.
#[test]
fn prose_mode_channels_are_unchanged() {
    let tmp = tempfile::tempdir().unwrap();
    write_spec(
        tmp.path(),
        "005-prose",
        "printf 'O%sT-D\\n' U; printf 'E%sR-D\\n' R >&2",
    );

    let out = run(tmp.path(), &["verify", "005-prose"]);
    assert_eq!(code(&out), 0);

    let so = stdout(&out);
    let se = stderr(&out);
    assert!(
        so.contains("OUT-D"),
        "the child's stdout stays on stdout: {so}"
    );
    assert!(se.contains("ERR-D"), "and its stderr on stderr: {se}");
    assert!(
        so.contains("[verify] $ printf 'O%sT-D\\n' U; printf 'E%sR-D\\n' R >&2"),
        "the transcript stays on stdout: {so}"
    );
    assert!(so.contains("[verify] exit 0"), "{so}");
    assert!(so.contains("passed (1 command(s))"), "{so}");
    assert!(
        !se.contains("[verify] $"),
        "and does not also appear on stderr: {se}"
    );
}

/// Spec 090 §3.2: output larger than a pipe buffer neither deadlocks nor
/// reaches stdout. A forwarder that read either stream only after the child
/// exited would hang here rather than fail, which is why the case is a test and
/// not a comment.
#[test]
fn json_forwards_more_than_a_pipe_buffer_without_deadlocking() {
    let tmp = tempfile::tempdir().unwrap();
    // ~512 KiB on each stream, well past the 64 KiB a pipe typically buffers,
    // and written to both so neither reader can be starved by the other.
    write_spec(
        tmp.path(),
        "006-flood",
        "i=0; while [ $i -lt 8192 ]; do printf 'O%sT-E-0123456789012345678901234567890123456789012345678901\\n' U; printf 'E%sR-E-0123456789012345678901234567890123456789012345678901\\n' R >&2; i=$((i+1)); done",
    );

    let out = run(tmp.path(), &["verify", "006-flood", "--json"]);
    assert_eq!(code(&out), 0);

    let v = one_envelope(&out);
    assert_eq!(v["report"]["outcome"], "passed");
    let se = stderr(&out);
    assert_eq!(
        se.lines().filter(|l| l.starts_with("OUT-E-")).count(),
        8192,
        "every forwarded line must arrive"
    );
    assert_eq!(se.lines().filter(|l| l.starts_with("ERR-E-")).count(), 8192);
}

// ---------------------------------------------------------------------------
// A consumer that stops reading the parent's stderr (spec 090 §3.2, D-3, D-4),
// and the harness that bounds it (D-6).
//
// The cases above all keep both of the parent's pipes drained to the end, so
// every write from the parent succeeds. That is the healthy half of the
// channel contract, and it left the unhealthy half unasserted: spec 090 D-3
// says that an inability to deliver logs must not change the acceptance
// verdict, and at `71a423a` it changed it twice. A consumer that read the
// opening transcript line and then closed the parent's stderr made a child
// writing ~1.3 MB hang past four seconds (measured: ~0.52 s with the consumer
// open), because `io::copy` returned on the first failed write and left the
// child's pipe undrained while `wait` blocked on the child; and it made
// `sleep 0.2; true` exit 101 with an empty stdout, because the `[verify] exit`
// line went through `eprintln!`, which unwraps its write.
//
// Each case below closes the consumer *after* that first line, so what it
// exercises is forwarding and completion rather than start-up.
//
// The harness itself is the subject of D-6. Its first shape bounded only the
// middle of the run: the opening `read_line` happened before the deadline loop
// was entered, so a fixture that never wrote a transcript blocked outside the
// bound, and its cleanup killed and reaped the fixture leader alone, which
// leaves a descendant of that leader running (an isolated probe measured one
// writing a file after the leader had been killed and reaped). Both are closed
// below: the budget is taken from before the spawn and covers start-up, the
// transcript read, completion and reader shutdown, and the fixture is spawned
// into a process group of its own that is signalled as a whole. The two
// safeguards have their own cases at the end of this file, each under an outer
// deadline that does not depend on the one being tested.
// ---------------------------------------------------------------------------

/// ~1 MB on one stream, generated by the child in one `awk` pass rather than a
/// shell loop, so the volume is past any pipe buffer without the fixture itself
/// costing seconds.
const VOLUME: &str =
    r#"awk 'BEGIN{s=sprintf("%1000s","");gsub(/ /,"X",s);for(i=0;i<1000;i++)print s}'"#;

/// How long one fixture may take, measured across its **whole** lifecycle:
/// spawn, the opening transcript line, the run itself, and the shutdown of the
/// reader threads. Generous enough for a loaded CI runner and far short of an
/// unbounded wait; the corrected verb finishes every case here in well under a
/// second locally.
const DEADLINE: Duration = Duration::from_secs(30);

/// The opening line every `verify --json` run writes to stderr (spec 090 §3.3).
const TRANSCRIPT: &str = "[verify] $ ";

/// What one run under a stderr consumer produced.
#[derive(Debug)]
struct Consumed {
    status: std::process::ExitStatus,
    stdout: Vec<u8>,
    /// Empty when the consumer closed: the point of closing is not to read.
    stderr: Vec<u8>,
}

impl Consumed {
    /// The whole of stdout as exactly one JSON document, per spec 090 §3.1.
    fn envelope(&self) -> serde_json::Value {
        serde_json::from_slice(&self.stdout).unwrap_or_else(|e| {
            panic!(
                "the whole of stdout must be one verdict envelope: {e}\n--- stdout ---\n{}",
                String::from_utf8_lossy(&self.stdout)
            )
        })
    }
}

/// Why a fixture run did not produce a `Consumed`. Every variant is reached
/// only after the fixture tree has been signalled and the leader reaped, so a
/// failure never leaves the harness's own processes behind.
#[derive(Debug)]
enum FixtureFailure {
    /// The budget expired with no line at all on the fixture's stderr. The
    /// leader's status at termination is carried because it is the first thing
    /// a recurrence needs: a leader that was still running had started and gone
    /// quiet, and one that had already exited never got as far as its first
    /// write. Neither is inferable from the elapsed time alone, and the merged
    /// harness discarded it.
    NoTranscript {
        waited: Duration,
        status: std::process::ExitStatus,
    },
    /// A line arrived, but it is not the opening transcript line. An empty
    /// `got` is end-of-file: the fixture exited without writing one.
    BadTranscript { got: String },
    /// Reading the fixture's stderr failed outright. Kept apart from
    /// `BadTranscript`, which is a statement about content: an operating-system
    /// failure reported as a content mismatch sends the reader looking at the
    /// fixture instead of at the pipe.
    TranscriptUnreadable { error: String },
    /// The fixture leader was still running when the budget expired.
    NotFinished { waited: Duration },
    /// The leader had exited on its own, but a reader was still waiting for
    /// end-of-file: a descendant of the fixture was holding the pipe open.
    ReaderStuck {
        stream: &'static str,
        waited: Duration,
    },
    /// The run was cancelled by the supervisor across the startup lifecycle,
    /// and the worker returned rather than proceeding under it. `leader` is the
    /// status of a leader that had already been created when the cancellation
    /// was found and was terminated and reaped here; `None` means the
    /// cancellation was seen before anything was spawned, so no process was
    /// ever created.
    Cancelled {
        leader: Option<std::process::ExitStatus>,
    },
}

impl std::fmt::Display for FixtureFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FixtureFailure::NoTranscript { waited, status } => write!(
                f,
                "no transcript line on the fixture's stderr within {waited:?} (the leader was {} when the harness terminated it)",
                if looks_killed_by_the_harness(status) {
                    "still running".to_string()
                } else {
                    format!("already finished, {status}")
                }
            ),
            FixtureFailure::BadTranscript { got } => write!(
                f,
                "the transcript's first line must start with {TRANSCRIPT:?} under --json, got {got:?}"
            ),
            FixtureFailure::TranscriptUnreadable { error } => {
                write!(f, "the fixture's stderr could not be read: {error}")
            }
            FixtureFailure::NotFinished { waited } => {
                write!(f, "the fixture did not finish within {waited:?}")
            }
            FixtureFailure::ReaderStuck { stream, waited } => write!(
                f,
                "the fixture exited but its {stream} was still open after {waited:?}: a descendant held it"
            ),
            FixtureFailure::Cancelled { leader: None } => write!(
                f,
                "the run was cancelled before the fixture was spawned; no process was created"
            ),
            FixtureFailure::Cancelled {
                leader: Some(status),
            } => write!(
                f,
                "the run was cancelled between the spawn and the publication of the leader's pid; the leader was terminated and reaped ({status})"
            ),
        }
    }
}

/// Signal the whole fixture tree.
///
/// `kill -9 -<pgid>`: the fixture is spawned into a process group of its own
/// whose id is the leader's pid, so the negative target names that tree and
/// nothing else. The shell builtin is used rather than taking a `libc`
/// dependency for one signal; every environment this suite is supported in is
/// Unix (CI is `ubuntu-latest`, development is macOS) and ships `/bin/sh`.
///
/// **Callers must not have reaped the leader.** While the leader is unreaped
/// its pid cannot be recycled, so the group id cannot have come to name some
/// other tree in the meantime. `Fixture` holds that invariant.
#[cfg(unix)]
fn signal_fixture_group(pid: u32) {
    let _ = Command::new("/bin/sh")
        .arg("-c")
        .arg(format!("kill -9 -{pid} 2>/dev/null"))
        .status();
}

/// Off Unix there is no process-group equivalent wired up here, and
/// `Fixture::terminate` falls back to killing the leader alone. The two
/// descendant-cleanup cases are `#[cfg(unix)]` for the same reason: this suite
/// is only supported on Unix, and a safeguard that cannot be exercised is not
/// asserted.
#[cfg(not(unix))]
fn signal_fixture_group(_pid: u32) {}

/// Does this status look like the harness's own `SIGKILL`?
///
/// A diagnostic, not a determination: a `SIGKILL` delivered from anywhere else
/// (an out-of-memory killer, a stray `pkill`) is indistinguishable from this
/// one, and would be read as a leader that was still running. Nothing branches
/// on the answer except which of two failure messages is printed; the tree has
/// been signalled and the leader reaped either way.
#[cfg(unix)]
fn looks_killed_by_the_harness(status: &std::process::ExitStatus) -> bool {
    use std::os::unix::process::ExitStatusExt;
    status.signal() == Some(9)
}

#[cfg(not(unix))]
fn looks_killed_by_the_harness(_status: &std::process::ExitStatus) -> bool {
    true
}

/// The state of one fixture tree, as the two threads that may act on it see it.
///
/// The four states cover the **whole** startup lifecycle, which is what the
/// first shape of this type did not. It began at `Live`, with "nothing has been
/// spawned yet" carried outside the enum as `Option::None`, and a cancellation
/// arriving in that state matched a catch-all arm that sent no signal and
/// **recorded nothing**. The worker then went on to spawn a fixture under a
/// cancellation it could not see, and the supervisor reported
/// `Overran { signalled: false, reaped: None }` while the fixture ran on and
/// performed its delayed side effect (D-8).
///
/// ```text
///   Unspawned ──cancel──▶ Cancelled ──publish──▶ Reaped{after_cancel: true}
///       │                     │                    (the spawner cleans up)
///       │                     └──spawn refused──▶ Cancelled (terminal)
///       └──publish──▶ Live(pid) ──cancel──▶ Live ──reap──▶ Reaped{false}
///                          └────────────────reap──────────▶ Reaped{false}
/// ```
#[derive(Debug, Clone, Copy)]
enum TreeState {
    /// The handle exists; no process does. Nothing to signal, and nothing has
    /// asked for one.
    Unspawned,
    /// Termination was requested before any pid was published, and nothing has
    /// been spawned under that request yet. **This state is the correction.**
    /// It binds whoever spawns next: a worker that reaches its pre-spawn check
    /// here does not spawn at all, and a worker that has already spawned when
    /// it reaches publication terminates and reaps what it started.
    Cancelled,
    /// A cancellation was found at publication, and the leader that had just
    /// been created is being terminated and reaped **by its spawner**, which is
    /// the only holder of its `Child`.
    ///
    /// This is a state of its own rather than more `Cancelled` because a
    /// process exists in it. Folded into `Cancelled` it read back as "nothing
    /// was ever started", which is one of the three answers this type exists to
    /// keep apart, told about the wrong one. The window is short (the `wait` it
    /// covers follows a `SIGKILL`) and it is still a window.
    CancellingSpawn(u32),
    /// Spawned and **unreaped**. The pid is the process-group id, and while the
    /// leader is unreaped that pid cannot be recycled, so signalling the
    /// negative of it cannot reach some other tree.
    Live(u32),
    /// The leader has been reaped. Nothing may be signalled from here.
    ///
    /// `after_cancel` records which reap this was: the ordinary end of a run
    /// (`false`), or the cleanup of a leader that was spawned into a standing
    /// cancellation (`true`). The supervisor reports the two apart, because a
    /// tree it never signalled and a tree it signalled are different findings.
    Reaped {
        status: std::process::ExitStatus,
        after_cancel: bool,
    },
}

/// The authority to terminate one fixture tree.
///
/// This exists because the thread that *owns* a fixture is, in the safeguard
/// cases, the thread whose deadline is under suspicion: a supervisor that could
/// only ask that worker to clean up would be relying on the very mechanism it is
/// testing. The handle is created by the supervisor before the worker starts,
/// so cleanup never travels through the worker.
///
/// **The invariant, and how it is held.** Nothing signals a process group whose
/// leader has been reaped. Signalling and reaping both take this mutex, and a
/// reap publishes `Reaped` in the same critical section as the `wait` that
/// produced it, so there is no window in which a signal can follow a reap. That
/// is a stronger guarantee than a poll-then-signal test, which reaps as a side
/// effect of asking and thereby destroys the thing it was checking for.
#[derive(Debug)]
struct Tree {
    state: Mutex<TreeState>,
}

/// What the supervisor's request to terminate a tree achieved, read back after
/// the grace it allowed the worker.
///
/// The three failure modes the report has to keep apart are named here rather
/// than folded into one `Option`. A `reaped: None` said all three of "the tree
/// was never terminated", "it was terminated and the leader was not reaped" and
/// "there was nothing to terminate yet" in the same word, and the first two are
/// defects in different halves of the harness while the third is not a defect
/// at all (D-8).
#[derive(Debug)]
enum Cleanup {
    /// A live group was signalled and its leader reaped by its owner inside the
    /// grace. The guarantee this harness is asked for, met.
    Reaped(std::process::ExitStatus),
    /// A live group was signalled and the leader was still unreaped when the
    /// grace expired. Termination happened; **reaping** did not. Distinct from
    /// `NeverStarted`: the tree is dead either way here.
    SignalledNotReaped,
    /// Cancellation was recorded before any pid was published, and the worker
    /// had still not spawned anything when the grace expired. Nothing existed
    /// to terminate; the cancellation stands and binds the worker if it ever
    /// resumes. This is delayed worker scheduling, not a failure to terminate.
    NeverStarted,
    /// Cancellation was recorded before publication, and the worker then
    /// spawned, found the cancellation at publication, and terminated and
    /// reaped what it had started.
    CancelledThenCleanedUp(std::process::ExitStatus),
    /// Cancellation was recorded before publication, the worker spawned into
    /// it, and its cleanup of that leader had not finished when the grace
    /// expired. A process existed: this is **not** `NeverStarted`, and the
    /// difference matters, because one says the supervisor found nothing to
    /// terminate and the other says a termination is still in flight.
    CancelledCleanupInFlight(u32),
    /// The leader had already been reaped by its owner before the supervisor
    /// asked. Nothing was signalled, and nothing needed to be.
    AlreadyReaped(std::process::ExitStatus),
}

impl Cleanup {
    /// The leader's status where one was reaped. `None` is the two states in
    /// which no leader was reaped, which are told apart by the variant.
    fn reaped(&self) -> Option<std::process::ExitStatus> {
        match self {
            Cleanup::Reaped(status)
            | Cleanup::CancelledThenCleanedUp(status)
            | Cleanup::AlreadyReaped(status) => Some(*status),
            Cleanup::SignalledNotReaped
            | Cleanup::NeverStarted
            | Cleanup::CancelledCleanupInFlight(_) => None,
        }
    }
}

/// What `Tree::cancel` found when it was asked to terminate a tree.
#[derive(Debug, Clone, Copy)]
enum CancelOutcome {
    /// A live group was signalled.
    Signalled,
    /// Nothing had been published yet. The cancellation is now recorded on the
    /// tree and binds whoever spawns next.
    Recorded,
    /// The leader had already been reaped by its owner, which is the one state
    /// in which signalling the group would no longer be safe.
    AlreadyReaped(std::process::ExitStatus),
}

/// What a spawner found when it asked whether it may still start, or publish.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Permit {
    /// No cancellation is standing. Go ahead.
    Proceed,
    /// A cancellation is standing. Nothing may be left running.
    Cancelled,
}

impl Tree {
    fn new() -> Arc<Tree> {
        Arc::new(Tree {
            state: Mutex::new(TreeState::Unspawned),
        })
    }

    /// Poisoning is recovered from rather than propagated: a poisoned lock here
    /// means a case is already failing, and turning that into a second panic
    /// inside cleanup would replace a readable failure with a crash.
    fn lock(&self) -> MutexGuard<'_, TreeState> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// May a spawn start? Read under the lock and the lock **released** before
    /// the caller spawns.
    ///
    /// Holding the lock across the spawn would close the window this returns
    /// into, and would do it by blocking the supervisor behind a `fork`/`exec`
    /// of unbounded duration: the one thread whose whole purpose is to act when
    /// the worker cannot. The window is absorbed at `publish` instead, which
    /// re-reads the state after the process exists.
    fn may_spawn(&self) -> Permit {
        match *self.lock() {
            TreeState::Unspawned => Permit::Proceed,
            TreeState::Cancelled => Permit::Cancelled,
            TreeState::CancellingSpawn(_) | TreeState::Live(_) | TreeState::Reaped { .. } => {
                unreachable!("a tree is spawned into once")
            }
        }
    }

    /// Record the leader the harness just spawned, or refuse it.
    ///
    /// `Permit::Cancelled` means a cancellation arrived while the spawn was in
    /// flight. The state is left `Cancelled`, and the caller, which is the only
    /// holder of the `Child` and so the only party that can `wait`, owns
    /// terminating and reaping what it started; it publishes the result through
    /// `cancelled_reap`.
    fn publish(&self, pid: u32) -> Permit {
        let mut guard = self.lock();
        match *guard {
            TreeState::Unspawned => {
                *guard = TreeState::Live(pid);
                Permit::Proceed
            }
            TreeState::Cancelled => {
                // A process exists from here until the spawner's reap
                // publishes `Reaped`. Leaving the state at `Cancelled` across
                // that `wait` would have it read back as "nothing was ever
                // started".
                *guard = TreeState::CancellingSpawn(pid);
                Permit::Cancelled
            }
            // A wildcard here would silently overwrite a live or reaped pid
            // with a `Live` that has no process behind it, and answer
            // `Proceed`. These states are refused for the same reason
            // `terminate` and `poll_exit` refuse them.
            TreeState::CancellingSpawn(_) | TreeState::Live(_) | TreeState::Reaped { .. } => {
                unreachable!("a tree is published into once")
            }
        }
    }

    /// Record the reap of a leader that was spawned into a standing
    /// cancellation, so the supervisor can tell that cleanup from the ordinary
    /// one.
    fn cancelled_reap(&self, status: std::process::ExitStatus) {
        let mut guard = self.lock();
        match *guard {
            // The only legal predecessor: `publish` set it on this same thread,
            // and nothing else writes it. Refused rather than overwritten, and
            // in every build rather than only in debug, because an unconditional
            // write here would answer a mis-call with a plausible
            // `Reaped { after_cancel: true }` that no state ever passed through.
            TreeState::CancellingSpawn(_) => {
                *guard = TreeState::Reaped {
                    status,
                    after_cancel: true,
                };
            }
            other => unreachable!("a cancelled reap follows a cancelled spawn, not {other:?}"),
        }
    }

    /// Terminate the tree, from any thread, and **record the request** whether
    /// or not there was anything to signal yet.
    ///
    /// The recording is the correction. The previous shape answered a
    /// cancellation arriving before publication with a bare `false` and no
    /// state change, so the request evaporated and the worker spawned into a
    /// tree that no longer remembered being cancelled.
    fn cancel(&self) -> CancelOutcome {
        let mut guard = self.lock();
        match *guard {
            TreeState::Live(pid) => {
                // The state is left `Live`: the worker owns the reap, and
                // publishing `Reaped` here without the `wait` that produced it
                // is the very thing this type forbids. Re-entering is
                // deliberate and safe. A second `SIGKILL` at an unreaped
                // leader is idempotent, and the pid cannot have been recycled
                // while it is unreaped, so the group is still this fixture's.
                signal_fixture_group(pid);
                CancelOutcome::Signalled
            }
            TreeState::Unspawned => {
                *guard = TreeState::Cancelled;
                CancelOutcome::Recorded
            }
            // Already cancelled: the record is there and still binds.
            TreeState::Cancelled => CancelOutcome::Recorded,
            // The spawner has already signalled this group and is reaping it.
            // Signalling again would be safe, since the leader is unreaped, but
            // it is the spawner's cleanup to finish and nothing here hurries
            // it.
            TreeState::CancellingSpawn(_) => CancelOutcome::Recorded,
            // The one state in which signalling would be unsafe: the pid may
            // since have been recycled onto somebody else's tree.
            TreeState::Reaped { status, .. } => CancelOutcome::AlreadyReaped(status),
        }
    }

    /// The published pid while the leader is live. A case that has to wait
    /// until publication has actually happened reads this rather than sleeping
    /// for a length of time it hopes is enough.
    fn live_pid(&self) -> Option<u32> {
        match *self.lock() {
            TreeState::Live(pid) => Some(pid),
            _ => None,
        }
    }

    /// The leader's status, once its owner has reaped it.
    fn reaped_status(&self) -> Option<std::process::ExitStatus> {
        match *self.lock() {
            TreeState::Reaped { status, .. } => Some(status),
            _ => None,
        }
    }

    /// What the state says happened, for the supervisor's report. Read after
    /// the grace, so a `Live` here is a leader its owner never reaped.
    fn cleanup(&self) -> Cleanup {
        match *self.lock() {
            TreeState::Reaped {
                status,
                after_cancel: true,
            } => Cleanup::CancelledThenCleanedUp(status),
            TreeState::Reaped {
                status,
                after_cancel: false,
            } => Cleanup::Reaped(status),
            TreeState::Live(_) => Cleanup::SignalledNotReaped,
            TreeState::CancellingSpawn(pid) => Cleanup::CancelledCleanupInFlight(pid),
            TreeState::Cancelled | TreeState::Unspawned => Cleanup::NeverStarted,
        }
    }
}

/// A fixture process and its tree, terminated and reaped on drop.
///
/// Drop is what makes an assertion failure anywhere in a case clean up: the
/// panic unwinds through the harness, the tree is signalled, and the leader is
/// reaped before the test thread reports. The `Tree` it holds is shared with
/// the supervisor, and every reap below happens inside that lock.
struct Fixture {
    child: std::process::Child,
    tree: Arc<Tree>,
}

/// Interposition points inside the spawn lifecycle.
///
/// Each closure is run by the worker at exactly the moment named, and nothing
/// else. They exist so the three cancellation orderings can be produced by
/// **synchronisation** rather than by racing the machine: a case blocks the
/// worker at the point it wants to test, cancels from the test thread, and
/// releases it. An ordering reproduced by a sleep would be a case that passes
/// or fails with the load on the runner.
///
/// Both are `None` on every production path, where `Fixture::spawn` is exactly
/// what it was.
#[derive(Default)]
struct SpawnGates {
    /// Run before the pre-spawn cancellation check, so a case can cancel while
    /// no process exists.
    before_spawn: Option<Box<dyn FnOnce() + Send>>,
    /// Run after the process exists and before its pid is published, so a case
    /// can cancel in the window between the two.
    before_publish: Option<Box<dyn FnOnce() + Send>>,
}

/// The result of asking for a fixture, which a standing cancellation can
/// refuse.
enum Spawned {
    Started(Fixture),
    /// The cancellation was already recorded when the pre-spawn check ran, so
    /// **no process was created**. There is nothing to terminate and nothing to
    /// reap.
    RefusedBeforeSpawn,
    /// The cancellation arrived while the spawn was in flight. The leader that
    /// had just been created was terminated as a group and reaped here, by the
    /// thread that created it, before this returned.
    CleanedUpAfterSpawn(std::process::ExitStatus),
}

impl Fixture {
    /// Spawn the fixture, unless a cancellation is standing.
    ///
    /// Three orderings, and the cleanup owner of each:
    ///
    /// 1. **Cancelled before the spawn.** The pre-spawn check sees `Cancelled`
    ///    and returns without creating a process. Nobody owns cleanup, because
    ///    nothing was created.
    /// 2. **Cancelled between the spawn and publication.** `publish` re-reads
    ///    the state after the `Child` exists and answers `Cancelled`. This
    ///    thread owns the cleanup: it holds the only `Child`, so it is the only
    ///    party that can `wait`, and it signals the group and reaps before
    ///    returning.
    /// 3. **Not cancelled.** The pid is published and the returned `Fixture`
    ///    owns cleanup, on its own paths and on `Drop`.
    ///
    /// The tree's lock is **not** held across `cmd.spawn()`. Doing so would
    /// close ordering 2 by blocking the supervisor behind a `fork`/`exec`,
    /// which is the one thread that must be able to act while the worker
    /// cannot; the window is absorbed at publication instead of hidden.
    fn spawn(mut cmd: Command, tree: Arc<Tree>, gates: SpawnGates) -> Spawned {
        if let Some(gate) = gates.before_spawn {
            gate();
        }
        if tree.may_spawn() == Permit::Cancelled {
            return Spawned::RefusedBeforeSpawn;
        }

        cmd.stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            // The isolation the cleanup depends on: without it the fixture
            // joins the test runner's group and a group signal would take the
            // runner down with it.
            cmd.process_group(0);
        }
        let mut child = cmd.spawn().expect("the fixture must spawn");
        let pid = child.id();

        if let Some(gate) = gates.before_publish {
            gate();
        }
        if tree.publish(pid) == Permit::Cancelled {
            // Ordering 2. The pid was never published, so nothing else can
            // signal this group; it is cleaned up here or not at all. The
            // leader is unreaped at the moment of the signal, so its pid cannot
            // have been recycled onto another tree.
            signal_fixture_group(pid);
            let _ = child.kill();
            let status = child.wait().expect("the fixture leader must reap");
            tree.cancelled_reap(status);
            return Spawned::CleanedUpAfterSpawn(status);
        }
        Spawned::Started(Fixture { child, tree })
    }

    /// Kill the whole tree and reap the leader, inside the tree's lock. A no-op
    /// once the leader has been reaped, which is where the pid-recycling
    /// invariant is kept: nothing signals a group whose leader is gone, and
    /// the supervisor cannot slip a signal in between the two halves here.
    ///
    /// The lock is deliberately held across `wait`, which blocks a concurrent
    /// `Tree::kill_group` for the duration. That window is the price of the
    /// invariant, and it is bounded: the `wait` is preceded by a `SIGKILL`,
    /// which cannot be caught or blocked, so the leader is already dying.
    /// Releasing the lock to wait and re-taking it to publish `Reaped` would
    /// narrow the window by reopening the signal-after-reap race this type
    /// exists to close.
    fn terminate(&mut self) -> std::process::ExitStatus {
        let tree = Arc::clone(&self.tree);
        let mut guard = tree.lock();
        match *guard {
            TreeState::Reaped { status, .. } => status,
            TreeState::Live(pid) => {
                signal_fixture_group(pid);
                // Belt for the non-Unix fallback, and harmless where the group
                // signal already landed.
                let _ = self.child.kill();
                let status = self.child.wait().expect("the fixture leader must reap");
                *guard = TreeState::Reaped {
                    status,
                    after_cancel: false,
                };
                status
            }
            TreeState::Unspawned | TreeState::Cancelled | TreeState::CancellingSpawn(_) => {
                unreachable!("a Fixture exists only once its leader has been published")
            }
        }
    }

    /// Has the leader exited? Reaps it if so, under the lock, so the `Reaped`
    /// state is published in the same breath as the `try_wait` that found it.
    fn poll_exit(&mut self) -> Option<std::process::ExitStatus> {
        let tree = Arc::clone(&self.tree);
        let mut guard = tree.lock();
        match *guard {
            TreeState::Reaped { status, .. } => Some(status),
            TreeState::Live(_) => match self.child.try_wait().expect("try_wait on the fixture") {
                Some(status) => {
                    *guard = TreeState::Reaped {
                        status,
                        after_cancel: false,
                    };
                    Some(status)
                }
                None => None,
            },
            TreeState::Unspawned | TreeState::Cancelled | TreeState::CancellingSpawn(_) => {
                unreachable!("a Fixture exists only once its leader has been published")
            }
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let tree = Arc::clone(&self.tree);
        let mut guard = tree.lock();
        if let TreeState::Live(pid) = *guard {
            signal_fixture_group(pid);
            let _ = self.child.kill();
            // Deliberately not `terminate`: a `Drop` that panics while a case's
            // assertion is unwinding aborts the test process, which would
            // replace a readable failure with a crash. The wait is still made,
            // so the leader is reaped here too.
            if let Ok(status) = self.child.wait() {
                *guard = TreeState::Reaped {
                    status,
                    after_cancel: false,
                };
            }
        }
    }
}

/// What is left of `budget` since `started`; zero once it is spent.
fn left(started: Instant, budget: Duration) -> Duration {
    budget.saturating_sub(started.elapsed())
}

/// Read a pipe to end-of-file on its own thread, handing the bytes back over a
/// channel so the wait for them can be bounded like everything else.
fn drain(mut pipe: impl Read + Send + 'static) -> Receiver<Vec<u8>> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = pipe.read_to_end(&mut buf);
        let _ = tx.send(buf);
    });
    rx
}

/// Run `cmd` with a consumer on its stderr that reads the first transcript line
/// and then either keeps reading (`close` false, the control) or closes the
/// stream (`close` true), within `budget`.
///
/// The budget starts before the spawn and covers every stage: start-up, the
/// opening transcript line, the run, and the shutdown of the reader threads.
/// Whichever stage overruns, the fixture tree is signalled while its leader is
/// still unreaped and the leader is then reaped, so no path here leaves a
/// fixture process or a descendant of one running.
///
/// `tree` is the caller's handle on the same fixture. It is passed in rather
/// than created here so that a supervisor holds termination authority from
/// before the spawn: if this function's own budget is broken, the supervisor can
/// still terminate the tree without going through this thread.
fn try_run_fixture(
    cmd: Command,
    close: bool,
    budget: Duration,
    tree: Arc<Tree>,
) -> Result<Consumed, FixtureFailure> {
    try_run_fixture_gated(cmd, close, budget, tree, SpawnGates::default())
}

/// `try_run_fixture` with the spawn-lifecycle interposition points open.
///
/// Only the three cancellation-ordering cases pass a non-default `gates`;
/// every other caller goes through `try_run_fixture`, where both are `None`
/// and this is the function it always was.
fn try_run_fixture_gated(
    cmd: Command,
    close: bool,
    budget: Duration,
    tree: Arc<Tree>,
    gates: SpawnGates,
) -> Result<Consumed, FixtureFailure> {
    let started = Instant::now();
    // A cancellation standing anywhere across the startup lifecycle ends the
    // run here, with the leader (if one was ever created) already terminated
    // and reaped by `Fixture::spawn`. Proceeding instead is what left a fixture
    // running behind a supervisor that had given up on it (D-8).
    let mut fixture = match Fixture::spawn(cmd, tree, gates) {
        Spawned::Started(fixture) => fixture,
        Spawned::RefusedBeforeSpawn => return Err(FixtureFailure::Cancelled { leader: None }),
        Spawned::CleanedUpAfterSpawn(status) => {
            return Err(FixtureFailure::Cancelled {
                leader: Some(status),
            });
        }
    };

    // The stdout pump starts before anything is awaited. A fixture that floods
    // stdout while the harness is still waiting for its transcript would
    // otherwise fill that pipe, block, and be reported as a missing transcript.
    let out_rx = drain(fixture.child.stdout.take().expect("stdout was piped"));

    // The opening `[verify] $ ...` line, read on its own thread. Reading it
    // inline is what put start-up outside the bound: a fixture that never
    // writes one blocks in `read_line` for as long as it lives.
    let err_pipe = fixture.child.stderr.take().expect("stderr was piped");
    let (first_tx, first_rx) = mpsc::channel();
    thread::spawn(move || {
        let mut err = BufReader::new(err_pipe);
        let mut first = String::new();
        let read = err.read_line(&mut first);
        let _ = first_tx.send((read.map(|_| first), err));
    });
    let (read, err) = match first_rx.recv_timeout(left(started, budget)) {
        Ok(pair) => pair,
        Err(_) => {
            let waited = started.elapsed();
            let status = fixture.terminate();
            return Err(FixtureFailure::NoTranscript { waited, status });
        }
    };
    // The line's content is the assertion; nothing downstream needs the text,
    // so no arm produces a value.
    match read {
        Ok(line) if line.starts_with(TRANSCRIPT) => {}
        Ok(line) => {
            fixture.terminate();
            return Err(FixtureFailure::BadTranscript { got: line });
        }
        Err(e) => {
            fixture.terminate();
            return Err(FixtureFailure::TranscriptUnreadable {
                error: e.to_string(),
            });
        }
    }

    let err_rx = if close {
        // Closing the read end is what makes the parent's subsequent writes
        // fail. Everything after this point must survive that.
        drop(err);
        None
    } else {
        Some(drain(err))
    };

    // The readers are awaited before the leader, deliberately: a reader that is
    // still waiting for end-of-file means something in the tree still holds the
    // pipe, and the leader is unreaped at that moment, so the tree can still be
    // signalled as a whole.
    let stdout = match out_rx.recv_timeout(left(started, budget)) {
        Ok(bytes) => bytes,
        Err(_) => return Err(stuck(&mut fixture, started, "stdout")),
    };
    let stderr = match err_rx {
        None => Vec::new(),
        Some(rx) => match rx.recv_timeout(left(started, budget)) {
            Ok(bytes) => bytes,
            Err(_) => return Err(stuck(&mut fixture, started, "stderr")),
        },
    };

    loop {
        if let Some(status) = fixture.poll_exit() {
            return Ok(Consumed {
                status,
                stdout,
                stderr,
            });
        }
        if left(started, budget).is_zero() {
            let waited = started.elapsed();
            fixture.terminate();
            return Err(FixtureFailure::NotFinished { waited });
        }
        thread::sleep(Duration::from_millis(5));
    }
}

/// A reader overran the budget. Signal the tree first (the leader has not been
/// polled, so it is unreaped and the group id is still this fixture's), then
/// read the status back to say which of the two things happened: a leader that
/// was still running is a hang, and a leader that had already exited means a
/// descendant was holding the pipe. That reading is a diagnostic label only,
/// with the ambiguity `looks_killed_by_the_harness` names; both labels describe
/// a tree that has been terminated and a leader that has been reaped.
fn stuck(fixture: &mut Fixture, started: Instant, stream: &'static str) -> FixtureFailure {
    let waited = started.elapsed();
    let status = fixture.terminate();
    if looks_killed_by_the_harness(&status) {
        FixtureFailure::NotFinished { waited }
    } else {
        FixtureFailure::ReaderStuck { stream, waited }
    }
}

/// `verify <id> --json` under a stderr consumer, within the suite's deadline.
fn run_with_stderr_consumer(root: &Path, id: &str, close: bool) -> Consumed {
    let mut cmd = bin();
    cmd.arg("--repo").arg(root).args(["verify", id, "--json"]);
    try_run_fixture(cmd, close, DEADLINE, Tree::new()).unwrap_or_else(|e| {
        panic!(
            "`verify {id}` with the stderr consumer {}: {e}",
            if close { "closed" } else { "open" }
        )
    })
}

/// Spec 090 D-3 on a quiet successful command: closing the consumer changes
/// nothing about the verdict. At `71a423a` this exited **101** with an empty
/// stdout, because the `[verify] exit 0` line panicked on the failed write
/// before the envelope was ever produced.
#[test]
fn a_closed_stderr_consumer_does_not_change_a_quiet_passs_verdict() {
    let tmp = tempfile::tempdir().unwrap();
    write_spec(tmp.path(), "007-quiet", "sleep 0.2; true");

    let run = run_with_stderr_consumer(tmp.path(), "007-quiet", true);

    assert_eq!(
        run.status.code(),
        Some(0),
        "a passing acceptance run stays exit 0 when its logs cannot be delivered"
    );
    let v = run.envelope();
    assert_eq!(v["verb"], "verify");
    assert_eq!(v["exitCode"], 0);
    assert_eq!(v["report"]["outcome"], "passed");
    assert_eq!(v["report"]["ran"], 1);
}

/// Spec 090 §3.2 with the destination gone, on the child's **stderr**. This is
/// the hang: at `71a423a` the parent stopped reading this pipe on the first
/// failed write and then waited on a child that was blocked filling it.
#[test]
fn a_closed_stderr_consumer_does_not_hang_a_child_flooding_its_stderr() {
    let tmp = tempfile::tempdir().unwrap();
    write_spec(tmp.path(), "008-flood-err", &format!("{VOLUME} >&2"));

    let run = run_with_stderr_consumer(tmp.path(), "008-flood-err", true);

    assert_eq!(run.status.code(), Some(0));
    assert_eq!(run.envelope()["report"]["outcome"], "passed");
}

/// The same, on the child's **stdout**: the stream forwarded from the pump
/// thread rather than from the waiting one. A pump that dropped its input on a
/// forwarding failure would close the child's stdout and hand it an `EPIPE`,
/// turning an undeliverable log into a changed exit status, which is the
/// outcome spec 090 D-3 forbids.
#[test]
fn a_closed_stderr_consumer_does_not_hang_a_child_flooding_its_stdout() {
    let tmp = tempfile::tempdir().unwrap();
    write_spec(tmp.path(), "009-flood-out", VOLUME);

    let run = run_with_stderr_consumer(tmp.path(), "009-flood-out", true);

    assert_eq!(run.status.code(), Some(0));
    assert_eq!(run.envelope()["report"]["outcome"], "passed");
}

/// Spec 090 §3.4 with the destination gone: a failing command keeps its own
/// exit code and its position, and stop-on-first-failure still holds. The
/// failure details reach the consumer on the channel that still works, and the
/// second command's absence is asserted by its missing side effect rather than
/// by counting output that was deliberately discarded.
#[test]
fn a_closed_stderr_consumer_keeps_a_failing_commands_details() {
    let tmp = tempfile::tempdir().unwrap();
    write_spec(
        tmp.path(),
        "010-flood-fail",
        &format!("{VOLUME}; {VOLUME} >&2; exit 7\nprintf c > ran-c.txt"),
    );

    let run = run_with_stderr_consumer(tmp.path(), "010-flood-fail", true);

    assert_eq!(run.status.code(), Some(1));
    let v = run.envelope();
    assert_eq!(v["exitCode"], 1);
    assert_eq!(v["report"]["outcome"], "failed");
    assert_eq!(v["report"]["failure"]["exitCode"], 7);
    assert_eq!(v["report"]["failure"]["index"], 1);
    assert_eq!(v["report"]["ran"], 1);
    assert_eq!(v["report"]["total"], 2);
    assert!(
        !tmp.path().join("ran-c.txt").exists(),
        "the command after a failing one must not run"
    );
}

/// The control the three cases above are measured against: the same volume on
/// both streams with the consumer left open. Every byte is delivered, the
/// verdict is the same one, and the run finishes inside the same deadline, so a
/// pass above cannot be explained by the fixture never having written anything.
#[test]
fn an_open_stderr_consumer_receives_every_forwarded_byte() {
    let tmp = tempfile::tempdir().unwrap();
    write_spec(
        tmp.path(),
        "011-flood-open",
        &format!("{VOLUME}; {VOLUME} >&2"),
    );

    let run = run_with_stderr_consumer(tmp.path(), "011-flood-open", false);

    assert_eq!(run.status.code(), Some(0));
    assert_eq!(run.envelope()["report"]["outcome"], "passed");
    // Counted rather than weighed. `awk` writes 1000 lines of 1000 `X` to each
    // stream, so an exact count says both arrived whole and says it without a
    // margin: a byte threshold near 2 * 1_001_000 clears the transcript line
    // the harness already consumed by about 1800 bytes, which is thin enough
    // that a longer fixture command would turn a correct run red.
    let whole = String::from_utf8_lossy(&run.stderr);
    let payload = "X".repeat(1000);
    assert_eq!(
        whole.lines().filter(|l| *l == payload).count(),
        2000,
        "both forwarded streams must arrive whole, got {} bytes",
        run.stderr.len()
    );
}

// ---------------------------------------------------------------------------
// The safeguards themselves (spec 090 D-6).
//
// Neither case runs `verify`: the fixture is a shell script chosen to break the
// harness in one specific way, because what is under test is the harness, and a
// correct `verify` cannot produce either condition. Each runs the harness on a
// worker thread and waits for it under an **outer** bound that does not depend
// on the inner budget, so a harness whose own deadline does not work fails the
// case instead of hanging it.
// ---------------------------------------------------------------------------

/// The inner budget the safeguard cases give the harness: short, because what
/// they measure is that it is honoured at all.
#[cfg(unix)]
const SAFEGUARD_BUDGET: Duration = Duration::from_secs(2);

/// The independent outer bound. Twenty times the inner budget: a harness that
/// honours its own deadline is nowhere near this, and one that does not is
/// reported rather than waited on.
#[cfg(unix)]
const OUTER_BOUND: Duration = Duration::from_secs(40);

/// How long the supervisor waits, after terminating the tree, for the worker to
/// notice and reap its own leader. The group signal closes the fixture's pipes,
/// which is what unblocks a worker stuck reading them, so this is a short grace
/// rather than a second deadline.
#[cfg(unix)]
const OUTER_GRACE: Duration = Duration::from_secs(5);

/// What an outer-bounded run produced.
#[derive(Debug)]
enum Supervised<T> {
    Returned(T),
    /// The worker overran the bound. The supervising thread cancelled the tree
    /// and `cleanup` says what that achieved, read back after `OUTER_GRACE`.
    ///
    /// This carried `signalled: bool` and `reaped: Option<ExitStatus>` before
    /// D-8, and those two fields could not tell the three answers apart: a tree
    /// that was never terminated, a tree that was terminated whose leader was
    /// never reaped, and a worker that had not yet spawned anything all read
    /// `reaped: None`, the last two of them defects and the third not.
    Overran {
        cleanup: Cleanup,
    },
}

/// Run `f` on a worker thread, and if it has not returned within `bound`,
/// terminate the fixture tree from **this** thread.
///
/// The bound exists precisely for the case where the harness's own deadline is
/// broken, so neither the bound nor the cleanup may be built from it. Before
/// this correction the worker owned the only `Fixture`, so a worker abandoned at
/// the bound took its `Drop` guard with it: the fixture survived the reported
/// failure, kept whatever descendants it had, and performed their delayed side
/// effects. Exiting the test binary does not fix that either, because the
/// fixture is deliberately in a process group of its own.
///
/// So `tree` is created by the caller before the worker starts, and cleanup goes
/// straight to it. Afterwards the worker is given `OUTER_GRACE` to return, not
/// because the report needs it but because the leader should be reaped by its
/// owner; whether that happened is reported rather than assumed.
///
/// The cancellation is recorded on the tree **whether or not there was anything
/// to signal**, which is D-8. A bound that expires before the worker has
/// published a pid used to send no signal and leave no trace, so the worker went
/// on to spawn a fixture into a tree that had been given up on; the standing
/// cancellation now binds it instead. `Cleanup` reports which of the orderings
/// this was.
#[cfg(unix)]
fn supervise<T: Send + 'static>(
    bound: Duration,
    tree: &Arc<Tree>,
    f: impl FnOnce() -> T + Send + 'static,
) -> Supervised<T> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let _ = tx.send(f());
    });
    // `Disconnected` arrives the moment the worker's sender drops, which is
    // what a panic inside the worker looks like from here. Reporting that as a
    // timeout would name the wrong failure: the real panic is already on
    // stderr, and the case should say to go and read it.
    match rx.recv_timeout(bound) {
        Ok(value) => Supervised::Returned(value),
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            panic!("the fixture harness panicked; its own message is above this one")
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            let outcome = tree.cancel();
            // The grace is given in every case, including the one where there
            // was nothing to signal: a cancellation recorded before publication
            // is honoured by the worker when it resumes, and this is the window
            // in which that happens.
            let _ = rx.recv_timeout(OUTER_GRACE);
            let cleanup = match outcome {
                // Terminal already; nothing was signalled and nothing needed
                // to be.
                CancelOutcome::AlreadyReaped(status) => Cleanup::AlreadyReaped(status),
                CancelOutcome::Signalled | CancelOutcome::Recorded => tree.cleanup(),
            };
            Supervised::Overran { cleanup }
        }
    }
}

/// `supervise` for the cases that expect the harness to honour its own budget:
/// an overrun is the harness defect being reported, and the tree has already
/// been terminated by the time the panic is raised.
#[cfg(unix)]
fn within<T: Send + 'static>(
    bound: Duration,
    tree: &Arc<Tree>,
    f: impl FnOnce() -> T + Send + 'static,
) -> T {
    match supervise(bound, tree, f) {
        Supervised::Returned(value) => value,
        Supervised::Overran { cleanup } => panic!(
            "the fixture harness did not return within its outer bound of {bound:?} \
             (the supervisor cancelled the tree: {cleanup:?})"
        ),
    }
}

/// A fixture that never writes a transcript line must be given up on at the
/// budget, not waited on for as long as it lives.
///
/// Before this correction the opening `read_line` ran before the deadline loop
/// was entered, so this fixture's ten minutes of silence were spent inside the
/// harness with no bound on them at all; the case would hit the outer bound
/// above rather than return.
#[test]
#[cfg(unix)]
fn a_fixture_that_never_emits_a_transcript_is_given_up_at_the_budget() {
    let mut cmd = Command::new("/bin/sh");
    // Silent on both streams and long-lived: the only thing that can end this
    // is the harness.
    cmd.arg("-c").arg("sleep 600");

    let tree = Tree::new();
    let started = Instant::now();
    let outcome = within(OUTER_BOUND, &tree, {
        let tree = Arc::clone(&tree);
        move || try_run_fixture(cmd, true, SAFEGUARD_BUDGET, tree)
    });
    let elapsed = started.elapsed();

    match outcome {
        Err(FixtureFailure::NoTranscript { .. }) => {}
        other => panic!("expected the budget to expire waiting for a transcript, got {other:?}"),
    }
    assert!(
        elapsed < SAFEGUARD_BUDGET * 4,
        "giving up took {elapsed:?}, which is not inside the budget of {SAFEGUARD_BUDGET:?}"
    );
}

/// A descendant of the fixture must not outlive the harness's cleanup.
///
/// The fixture backgrounds a subshell that sleeps and then writes a file, and
/// then blocks forever itself. Killing the leader alone leaves that subshell
/// running, which an isolated probe measured: it wrote its file after the
/// leader had been killed and reaped. It also keeps the leader's stdout pipe
/// open, so nothing that waits for end-of-file on that pipe can be relied on to
/// notice, which is why the cleanup signals the process group rather than
/// waiting for a broken pipe to propagate.
///
/// The delayed side effect is the witness: it is checked after the moment it
/// was scheduled for has passed, so its absence is cleanup and not timing.
#[test]
#[cfg(unix)]
fn a_timed_out_fixtures_descendant_is_terminated_before_its_side_effect() {
    let tmp = tempfile::tempdir().unwrap();
    let marker = tmp.path().join("delayed-side-effect.txt");
    // Comfortably past the budget, so the descendant is certain to still be
    // asleep when the harness gives up: what the case measures is cleanup.
    let delay = Duration::from_secs(6);

    let mut cmd = Command::new("/bin/sh");
    cmd.arg("-c").arg(format!(
        "( sleep {}; printf x > \"{}\" ) &\nprintf '{TRANSCRIPT}fixture\\n' >&2\nsleep 600\n",
        delay.as_secs(),
        marker.display()
    ));

    let tree = Tree::new();
    let started = Instant::now();
    let outcome = within(OUTER_BOUND, &tree, {
        let tree = Arc::clone(&tree);
        move || try_run_fixture(cmd, true, SAFEGUARD_BUDGET, tree)
    });
    let elapsed = started.elapsed();

    match outcome {
        Err(FixtureFailure::NotFinished { .. }) => {}
        other => panic!("expected the budget to expire on a live fixture, got {other:?}"),
    }
    assert!(
        elapsed < SAFEGUARD_BUDGET * 4,
        "giving up took {elapsed:?}, which is not inside the budget of {SAFEGUARD_BUDGET:?}"
    );

    // Past the moment the descendant was scheduled to write, with margin.
    thread::sleep((delay + Duration::from_secs(2)).saturating_sub(started.elapsed()));
    assert!(
        !marker.exists(),
        "a descendant of the fixture survived cleanup and performed its side effect at {}",
        marker.display()
    );
}

/// The outer supervisor must terminate the fixture, not merely report that the
/// inner deadline was missed.
///
/// The inner budget here is deliberately broken: ten minutes, which the case
/// never reaches, so the only thing that can end this run is the supervisor.
/// Before this correction the supervisor panicked on its waiting thread while
/// the worker stayed blocked owning the `Fixture`; dropping the waiting thread
/// runs no guard, and the fixture is in a process group of its own, so exiting
/// the test binary does not reach it either. The witness is the descendant's
/// delayed side effect, checked past the moment it was scheduled for.
#[test]
#[cfg(unix)]
fn a_broken_inner_deadline_is_terminated_by_the_outer_supervisor() {
    let tmp = tempfile::tempdir().unwrap();
    let marker = tmp.path().join("delayed-side-effect.txt");
    let delay = Duration::from_secs(6);

    let mut cmd = Command::new("/bin/sh");
    cmd.arg("-c").arg(format!(
        "( sleep {}; printf x > \"{}\" ) &\nprintf '{TRANSCRIPT}fixture\\n' >&2\nsleep 600\n",
        delay.as_secs(),
        marker.display()
    ));

    // Ten minutes: a budget that cannot expire inside this case.
    let broken_budget = Duration::from_secs(600);
    let tree = Tree::new();
    let started = Instant::now();
    let outcome = supervise(SAFEGUARD_BUDGET, &tree, {
        let tree = Arc::clone(&tree);
        move || try_run_fixture(cmd, true, broken_budget, tree)
    });
    let elapsed = started.elapsed();

    match outcome {
        // The three answers are separated rather than collapsed, so a
        // recurrence names which half of the guarantee was missed. The
        // reaping assertion is kept, not dropped: it is `Cleanup::Reaped`
        // here, and every other variant is a distinct, named failure.
        Supervised::Overran { cleanup } => match cleanup {
            Cleanup::Reaped(status) => assert!(
                looks_killed_by_the_harness(&status),
                "the leader was terminated by the supervisor, got {status:?}"
            ),
            Cleanup::SignalledNotReaped => panic!(
                "the tree was terminated but its leader was never reaped: the group signal \
                 closes the pipes its owner is blocked on, which is what lets that owner \
                 finish, and it did not within {OUTER_GRACE:?}"
            ),
            Cleanup::NeverStarted => panic!(
                "the supervisor found nothing to terminate: the worker had published no pid \
                 by the bound and had still spawned nothing {OUTER_GRACE:?} later, so this \
                 case did not exercise the after-publication path it is written for"
            ),
            Cleanup::CancelledThenCleanedUp(status) => panic!(
                "the cancellation was recorded before publication rather than signalled at a \
                 live tree ({status:?}); this case is the after-publication ordering, and the \
                 pre-publication ones have their own cases"
            ),
            Cleanup::CancelledCleanupInFlight(pid) => panic!(
                "the cancellation was recorded before publication and the spawner's cleanup of \
                 leader {pid} was still in flight after {OUTER_GRACE:?}; this case is the \
                 after-publication ordering, and the pre-publication ones have their own cases"
            ),
            Cleanup::AlreadyReaped(status) => panic!(
                "the leader was already reaped when the supervisor acted ({status:?}), so the \
                 supervisor's own termination was not what ended this fixture"
            ),
        },
        Supervised::Returned(returned) => {
            panic!("a ten-minute inner budget must not have returned on its own: {returned:?}")
        }
    }
    assert!(
        elapsed < SAFEGUARD_BUDGET + OUTER_GRACE + SAFEGUARD_BUDGET,
        "the supervisor took {elapsed:?} to give up on a {SAFEGUARD_BUDGET:?} bound"
    );

    // Past the moment the descendant was scheduled to write, with margin.
    thread::sleep((delay + Duration::from_secs(2)).saturating_sub(started.elapsed()));
    assert!(
        !marker.exists(),
        "a descendant of the fixture survived the supervisor's cleanup and performed its \
         side effect at {}",
        marker.display()
    );
}

/// A leader that exits while a descendant keeps its pipes is cleaned up too.
///
/// This is the case spec 090 D-6 described as left unhandled, on the reasoning
/// that the leader would be reaped by then and the group signal no longer safe
/// to send. That reasoning does not match the harness it describes: the readers
/// are awaited **before** the leader is ever polled, so at the moment a reader
/// overruns the leader is still unreaped, its pid cannot have been recycled, and
/// `stuck` signals the group before reaping it. The case is green on purpose,
/// and what it corrects is the prose rather than the code.
///
/// The fixture writes its transcript line, backgrounds a descendant that would
/// write a file after a delay, and exits 0. The descendant inherits the leader's
/// stdout pipe, so no end-of-file arrives; the absence of the file, checked past
/// the moment it was scheduled for, is the witness that cleanup reached it.
#[test]
#[cfg(unix)]
fn an_exited_leaders_descendant_on_the_pipes_is_still_terminated() {
    let tmp = tempfile::tempdir().unwrap();
    let marker = tmp.path().join("delayed-side-effect.txt");
    let delay = Duration::from_secs(6);

    let mut cmd = Command::new("/bin/sh");
    cmd.arg("-c").arg(format!(
        "( sleep {}; printf x > \"{}\" ) &\nprintf '{TRANSCRIPT}fixture\\n' >&2\nexit 0\n",
        delay.as_secs(),
        marker.display()
    ));

    let tree = Tree::new();
    let started = Instant::now();
    let outcome = within(OUTER_BOUND, &tree, {
        let tree = Arc::clone(&tree);
        move || try_run_fixture(cmd, true, SAFEGUARD_BUDGET, tree)
    });
    let elapsed = started.elapsed();

    match outcome {
        // The leader's own exit 0 is read back after the group signal, which is
        // what tells the two apart: a live leader comes back killed.
        //
        // `"stdout"` is not a race with the other pipe, even though the
        // descendant inherits both. This case runs with the consumer closed,
        // and a closed consumer drops the stderr reader rather than draining
        // it, so stdout is the only stream the harness is waiting on and the
        // only one `stuck` can ever name here.
        Err(FixtureFailure::ReaderStuck { stream, .. }) => assert_eq!(stream, "stdout"),
        other => panic!("expected a reader held open by a descendant, got {other:?}"),
    }
    assert!(
        elapsed < SAFEGUARD_BUDGET * 4,
        "giving up took {elapsed:?}, which is not inside the budget of {SAFEGUARD_BUDGET:?}"
    );
    assert!(
        tree.reaped_status().is_some(),
        "the leader must be reaped by the time the harness reports"
    );

    thread::sleep((delay + Duration::from_secs(2)).saturating_sub(started.elapsed()));
    assert!(
        !marker.exists(),
        "a descendant of an exited leader survived cleanup and performed its side effect at {}",
        marker.display()
    );
}

// ---------------------------------------------------------------------------
// Cancellation across the whole startup lifecycle (spec 090 D-8).
//
// The safeguard cases above all cancel a tree whose pid has already been
// published, which is the one ordering the merged `Tree` handled. The three
// below cover the lifecycle end to end: cancelled before the worker starts
// spawning, cancelled after a process exists but before its pid is published,
// and cancelled after publication, which is the already-correct path kept
// under a case of its own so a change to the other two cannot quietly move it.
//
// None of them uses a sleep to produce its ordering. The worker is blocked at
// the exact point under test by a rendezvous, or waited for until the state it
// is being tested at is observably reached, so the case decides the ordering
// rather than the load on the machine.
// ---------------------------------------------------------------------------

/// How long a case waits for a rendezvous, a worker's result, or a condition
/// the other thread will certainly reach. Not a race budget: it exists so a
/// regression is reported instead of hanging the suite.
#[cfg(unix)]
const SYNC_BOUND: Duration = Duration::from_secs(20);

/// The worker half of a one-shot rendezvous, and the case's handle on it.
///
/// The worker announces it has arrived and then blocks until released. It
/// treats a **disconnected** release channel as a release, which is what makes
/// the cleanup below work: dropping the case's end frees a worker blocked here
/// even when the case is unwinding from a failed assertion.
#[cfg(unix)]
fn rendezvous() -> (
    Box<dyn FnOnce() + Send>,
    mpsc::Receiver<()>,
    mpsc::Sender<()>,
) {
    let (arrived_tx, arrived_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel::<()>();
    let gate = Box::new(move || {
        let _ = arrived_tx.send(());
        let _ = release_rx.recv();
    }) as Box<dyn FnOnce() + Send>;
    (gate, arrived_rx, release_tx)
}

/// One cancellation-ordering case's worker, with cleanup that does not depend
/// on the case reaching its end.
///
/// **Why this is a guard rather than a sequence of statements.** Every case
/// below can fail an assertion while a fixture is running and a worker is
/// blocked at a rendezvous. Statements written after that assertion do not run.
/// `Drop` does, on the unwinding thread, and it does the three things in the
/// order that makes them work: record the cancellation **first**, so a worker
/// that has not spawned yet is bound by it; then drop the release channel, so a
/// worker parked at a rendezvous is freed to honour it; then wait for that
/// worker, so the case does not report while its fixture is still being cleaned
/// up. Doing the middle step first would free the worker to spawn a fixture
/// into a tree that had not yet been cancelled, which is the defect this whole
/// section is about.
#[cfg(unix)]
struct GatedRun<T> {
    tree: Arc<Tree>,
    arrived: Option<mpsc::Receiver<()>>,
    release: Option<mpsc::Sender<()>>,
    result: Option<Receiver<T>>,
}

#[cfg(unix)]
impl<T> GatedRun<T> {
    /// Start `f` on a worker thread. `gate` is its rendezvous handle, or
    /// `None` for a case that synchronises on observed state instead.
    fn start(
        tree: &Arc<Tree>,
        gate: Option<(mpsc::Receiver<()>, mpsc::Sender<()>)>,
        f: impl FnOnce() -> T + Send + 'static,
    ) -> GatedRun<T>
    where
        T: Send + 'static,
    {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let _ = tx.send(f());
        });
        let (arrived, release) = match gate {
            Some((a, r)) => (Some(a), Some(r)),
            None => (None, None),
        };
        GatedRun {
            tree: Arc::clone(tree),
            arrived,
            release,
            result: Some(rx),
        }
    }

    /// Block until the worker has reached its rendezvous.
    fn await_gate(&self) {
        self.arrived
            .as_ref()
            .expect("this case was started with a rendezvous")
            .recv_timeout(SYNC_BOUND)
            .expect("the worker must reach its rendezvous");
    }

    /// Let the worker past its rendezvous.
    fn release(&mut self) {
        drop(self.release.take());
    }

    /// The worker's result, within `SYNC_BOUND`.
    fn collect(&mut self) -> T {
        self.result
            .take()
            .expect("the worker's result is collected once")
            .recv_timeout(SYNC_BOUND)
            .expect("the worker must return once it is released")
    }
}

#[cfg(unix)]
impl<T> Drop for GatedRun<T> {
    fn drop(&mut self) {
        // Order matters; see the type's documentation.
        self.tree.cancel();
        drop(self.release.take());
        if let Some(rx) = self.result.take() {
            let _ = rx.recv_timeout(SYNC_BOUND);
        }
    }
}

/// Wait until `cond` holds, failing the case rather than hanging it.
///
/// Polling a condition another thread will certainly reach is a
/// synchronisation point, not a guess at how long something takes: the case
/// proceeds at the moment the state it needs is observable, and `SYNC_BOUND` is
/// only there so a regression reports.
#[cfg(unix)]
fn wait_until(what: &str, mut cond: impl FnMut() -> bool) {
    let started = Instant::now();
    while !cond() {
        assert!(
            started.elapsed() < SYNC_BOUND,
            "waited {SYNC_BOUND:?} for {what}"
        );
        thread::sleep(Duration::from_millis(2));
    }
}

/// A fixture that backgrounds a descendant which would write `marker` after
/// `delay`, touches `ready` once that descendant exists, writes its transcript
/// line, and then blocks for as long as it is allowed to.
///
/// `ready` is what lets a case wait for the descendant to exist rather than
/// hope it does: a group signal sent before the fork would clean up a tree of
/// one, and the case would assert nothing about descendants.
#[cfg(unix)]
fn descendant_fixture(marker: &Path, ready: &Path, delay: Duration) -> Command {
    let mut cmd = Command::new("/bin/sh");
    cmd.arg("-c").arg(format!(
        "( sleep {}; printf x > \"{}\" ) &\nprintf r > \"{}\"\nprintf '{TRANSCRIPT}fixture\\n' >&2\nsleep 600\n",
        delay.as_secs(),
        marker.display(),
        ready.display()
    ));
    cmd
}

/// How long past a descendant's scheduled write a case waits before reading the
/// marker. The absence is only evidence once the moment it was scheduled for
/// has passed.
#[cfg(unix)]
fn past_the_side_effect(started: Instant, delay: Duration) {
    thread::sleep((delay + Duration::from_secs(2)).saturating_sub(started.elapsed()));
}

/// **Ordering 1: cancelled before the worker starts spawning.**
///
/// The worker is held at a rendezvous immediately before its pre-spawn check,
/// so no process exists when the cancellation is recorded. The merged `Tree`
/// answered that cancellation with a bare `false` and no state change, and the
/// worker then spawned a fixture nothing would ever terminate.
///
/// The witness is that the fixture **never ran at all**: its `ready` file, which
/// it writes as its second act, is absent, and so is the descendant's delayed
/// side effect, checked past the moment it was scheduled for.
#[test]
#[cfg(unix)]
fn a_cancellation_before_the_spawn_starts_no_fixture() {
    let tmp = tempfile::tempdir().unwrap();
    let marker = tmp.path().join("delayed-side-effect.txt");
    let ready = tmp.path().join("fixture-ran.txt");
    let delay = Duration::from_secs(6);
    let cmd = descendant_fixture(&marker, &ready, delay);

    let tree = Tree::new();
    let (gate, arrived, release) = rendezvous();
    let started = Instant::now();
    let mut run = GatedRun::start(&tree, Some((arrived, release)), {
        let tree = Arc::clone(&tree);
        move || {
            try_run_fixture_gated(
                cmd,
                true,
                Duration::from_secs(600),
                tree,
                SpawnGates {
                    before_spawn: Some(gate),
                    ..SpawnGates::default()
                },
            )
        }
    });

    run.await_gate();
    // The cancellation the merged code dropped on the floor.
    match tree.cancel() {
        CancelOutcome::Recorded => {}
        other => panic!("a cancellation before any spawn must be recorded, got {other:?}"),
    }
    run.release();

    match run.collect() {
        Err(FixtureFailure::Cancelled { leader: None }) => {}
        other => panic!("expected the spawn to be refused outright, got {other:?}"),
    }
    match tree.cleanup() {
        Cleanup::NeverStarted => {}
        other => panic!("nothing was spawned, so there is nothing to have reaped: {other:?}"),
    }
    assert!(
        !ready.exists(),
        "the fixture ran under a standing cancellation: {}",
        ready.display()
    );

    past_the_side_effect(started, delay);
    assert!(
        !marker.exists(),
        "a fixture spawned under a standing cancellation performed its delayed side effect at {}",
        marker.display()
    );
}

/// **Ordering 2: cancelled after a process exists but before its pid is
/// published.**
///
/// The worker is held at a rendezvous between `cmd.spawn()` returning and the
/// publication of the leader's pid. The fixture is therefore running, with a
/// descendant of its own, and the tree does not yet know its pid: nothing but
/// the worker can signal that group, which is why the worker is the cleanup
/// owner of this ordering.
///
/// The case waits for the fixture's `ready` file before cancelling, so the
/// descendant demonstrably exists and the group signal has something to reach
/// beyond the leader. The witnesses are all three of the guarantees asked for:
/// the leader was **terminated** (a `SIGKILL` status, not an exit of its own),
/// it was **reaped** (`CancelledThenCleanedUp` carries the status the `wait`
/// returned), and the descendant's **delayed side effect does not occur**.
#[test]
#[cfg(unix)]
fn a_cancellation_between_the_spawn_and_the_publication_cleans_up() {
    let tmp = tempfile::tempdir().unwrap();
    let marker = tmp.path().join("delayed-side-effect.txt");
    let ready = tmp.path().join("fixture-ran.txt");
    let delay = Duration::from_secs(6);
    let cmd = descendant_fixture(&marker, &ready, delay);

    let tree = Tree::new();
    let (gate, arrived, release) = rendezvous();
    let started = Instant::now();
    let mut run = GatedRun::start(&tree, Some((arrived, release)), {
        let tree = Arc::clone(&tree);
        move || {
            try_run_fixture_gated(
                cmd,
                true,
                Duration::from_secs(600),
                tree,
                SpawnGates {
                    before_publish: Some(gate),
                    ..SpawnGates::default()
                },
            )
        }
    });

    // The worker is parked with a live, unpublished leader.
    run.await_gate();
    assert!(
        tree.live_pid().is_none(),
        "the rendezvous sits before publication, so no pid can be on the tree yet"
    );
    // The fixture has forked its descendant by the time this file exists, so
    // the cancellation below is measured against a tree of more than one.
    wait_until("the fixture to fork its descendant", || ready.exists());

    match tree.cancel() {
        CancelOutcome::Recorded => {}
        other => panic!("the pid is unpublished, so the cancellation must be recorded: {other:?}"),
    }
    run.release();

    let status = match run.collect() {
        Err(FixtureFailure::Cancelled {
            leader: Some(status),
        }) => status,
        other => panic!("expected the spawner to clean up the leader it created, got {other:?}"),
    };
    assert!(
        looks_killed_by_the_harness(&status),
        "the leader must have been terminated rather than have exited on its own, got {status:?}"
    );
    match tree.cleanup() {
        Cleanup::CancelledThenCleanedUp(recorded) => assert_eq!(
            recorded, status,
            "the reap the spawner performed is the one recorded on the tree"
        ),
        other => panic!("the leader must be reaped by the thread that created it: {other:?}"),
    }
    assert_eq!(
        tree.cleanup().reaped(),
        Some(status),
        "a cleanup that reaped a leader reports its status"
    );

    past_the_side_effect(started, delay);
    assert!(
        !marker.exists(),
        "a descendant of a leader cancelled before publication survived and performed its \
         side effect at {}",
        marker.display()
    );
}

/// **Ordering 3: cancelled after publication.** The path that was already
/// correct, kept under a case of its own.
///
/// The case waits until the pid is observably on the tree, so the ordering is
/// decided rather than raced, then cancels from the test thread exactly as the
/// supervisor does. Cleanup is the worker's here: the group signal closes the
/// pipes it is blocked on, and it reaps its own leader.
///
/// `a_broken_inner_deadline_is_terminated_by_the_outer_supervisor` covers the
/// same ordering through `supervise`, where the cancellation is produced by an
/// expiring bound. This one isolates the transition from the bound.
#[test]
#[cfg(unix)]
fn a_cancellation_after_publication_is_signalled_and_reaped_by_the_worker() {
    let tmp = tempfile::tempdir().unwrap();
    let marker = tmp.path().join("delayed-side-effect.txt");
    let ready = tmp.path().join("fixture-ran.txt");
    let delay = Duration::from_secs(6);
    let cmd = descendant_fixture(&marker, &ready, delay);

    let tree = Tree::new();
    let started = Instant::now();
    // A budget that cannot expire inside this case: the only thing that can end
    // this run is the cancellation below.
    let mut run = GatedRun::start(&tree, None, {
        let tree = Arc::clone(&tree);
        move || try_run_fixture(cmd, true, Duration::from_secs(600), tree)
    });

    wait_until("the worker to publish the leader's pid", || {
        tree.live_pid().is_some()
    });
    wait_until("the fixture to fork its descendant", || ready.exists());

    match tree.cancel() {
        CancelOutcome::Signalled => {}
        other => panic!("a published, unreaped leader must be signalled, got {other:?}"),
    }

    // The group signal closes the pipes the worker is blocked on, which is what
    // lets it finish and reap its own leader.
    let _ = run.collect();
    let status = match tree.cleanup() {
        Cleanup::Reaped(status) => status,
        other => panic!("the worker must reap the leader the supervisor signalled: {other:?}"),
    };
    assert!(
        looks_killed_by_the_harness(&status),
        "the leader was terminated by the cancellation, got {status:?}"
    );

    past_the_side_effect(started, delay);
    assert!(
        !marker.exists(),
        "a descendant of a leader cancelled after publication survived and performed its \
         side effect at {}",
        marker.display()
    );
}

/// The window between a spawner finding a cancellation and finishing its reap
/// is one in which a **process exists**, and it must not read back as one in
/// which none ever did.
///
/// The `wait` that window covers follows a `SIGKILL` and is therefore short,
/// which is why folding it into `Cancelled` did not show up in the orderings
/// above: `run.collect()` returns after the reap, so those cases only ever see
/// the state on the far side of it. A supervisor's grace can expire inside it,
/// and `NeverStarted` would then tell it there had been nothing to terminate.
///
/// Driven directly on the state machine, with no process: what is under test is
/// the reading, and a fixture would only reintroduce the timing that hides it.
#[test]
#[cfg(unix)]
fn a_spawners_cleanup_in_flight_does_not_read_as_nothing_started() {
    let tree = Tree::new();
    assert!(matches!(tree.cleanup(), Cleanup::NeverStarted));
    assert!(matches!(tree.cancel(), CancelOutcome::Recorded));
    assert!(
        matches!(tree.cleanup(), Cleanup::NeverStarted),
        "a cancellation with nothing spawned under it is still nothing started"
    );

    // What `Fixture::spawn` does on finding the cancellation at publication,
    // minus the process: the pid is recorded, and the reap that follows it has
    // not happened yet.
    assert_eq!(tree.publish(4242), Permit::Cancelled);
    match tree.cleanup() {
        Cleanup::CancelledCleanupInFlight(pid) => assert_eq!(pid, 4242),
        other => panic!("a spawner's cleanup in flight must not read as {other:?}"),
    }
    // A supervisor asking again here must not signal: the spawner has already
    // signalled this group and owns the reap. Nothing in this case may reach
    // pid 4242, which belongs to whoever happens to hold it.
    assert!(matches!(tree.cancel(), CancelOutcome::Recorded));
    assert_eq!(
        tree.cleanup().reaped(),
        None,
        "nothing has been reaped while the cleanup is still in flight"
    );

    // And the spawner's reap closes it.
    let reaped = Command::new("/bin/sh")
        .arg("-c")
        .arg("exit 3")
        .status()
        .unwrap();
    tree.cancelled_reap(reaped);
    match tree.cleanup() {
        Cleanup::CancelledThenCleanedUp(status) => assert_eq!(status.code(), Some(3)),
        other => panic!("the spawner's reap must close the in-flight state, got {other:?}"),
    }
}
