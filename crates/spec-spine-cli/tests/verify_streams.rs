// Spec: specs/118-the-verdict-is-the-only-thing-on-stdout/spec.md
//! `spec-spine verify`'s two output channels (spec 118).
//!
//! Every fixture here runs a command that writes to **both** of its streams,
//! which is the property spec 118 §1.2 found missing from the `verify` cases in
//! `cli.rs`: each of those runs `true`, `exit 7` or a fence that is never
//! executed, so not one of them writes a byte, and the assertion that stdout is
//! one envelope could not fail against them.
//!
//! What is asserted is the channel, not the verdict. The verdict, the report
//! fields and the exit-code mapping are spec 049's and are covered in `cli.rs`;
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
/// which is the defect spec 118 §1.1 measured.
fn one_envelope(out: &Output) -> serde_json::Value {
    serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!(
            "the whole of stdout must be one verdict envelope: {e}\n--- stdout ---\n{}\n--- stderr ---\n{}",
            stdout(out),
            stderr(out)
        )
    })
}

/// Spec 118 §3.1, §3.2, §3.3 on a passing command: stdout is one envelope, both
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
    assert_eq!(v["ok"], true);
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
    // The transcript spec 049 §3.5 requires, on the channel spec 118 §3.3
    // assigns it. Under `--json` it was suppressed entirely before this spec.
    assert!(
        se.contains("[verify] $ printf 'O%sT-A\\n' U; printf 'E%sR-A\\n' R >&2"),
        "the command line belongs on stderr: {se}"
    );
    assert!(se.contains("[verify] exit 0"), "the exit line too: {se}");
}

/// Spec 118 §3.1, §3.4 on a failure: the channel holds, the failure detail is
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
    // Spec 049 §3.3: the drift-tier 1, never the command's own 7.
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

/// Spec 118 §3.1's error path: the `R-001` refusal of spec 049 §3.7 is one
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
        "`report` and `error` are exclusive (spec 037 §3.1): {v}"
    );
    // The refusal precedes execution, so nothing was run; what this asserts is
    // that the error path carries one envelope and no prose.
    assert!(!stdout(&out).contains("never-runs"), "{}", stdout(&out));
}

/// Spec 118 §3.4: `--plan --json` is one envelope and spawns nothing.
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

/// Spec 118 §3.4's preservation half: without `--json` the child inherits both
/// streams and the transcript is on stdout, exactly as spec 049 shipped it.
/// Green before spec 118 on purpose; this is what stops the correction from
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

/// Spec 118 §3.2: output larger than a pipe buffer neither deadlocks nor
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
// A consumer that stops reading the parent's stderr (spec 118 §3.2, D-3, D-4),
// and the harness that bounds it (D-9).
//
// The cases above all keep both of the parent's pipes drained to the end, so
// every write from the parent succeeds. That is the healthy half of the
// channel contract, and it left the unhealthy half unasserted: spec 118 D-3
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
// The harness itself is the subject of D-9. Its first shape bounded only the
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

/// The opening line every `verify --json` run writes to stderr (spec 118 §3.3).
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
    /// The whole of stdout as exactly one JSON document, per spec 118 §3.1.
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
    /// The budget expired with no line at all on the fixture's stderr.
    NoTranscript { waited: Duration },
    /// A line arrived, but it is not the opening transcript line. An empty
    /// `got` is end-of-file: the fixture exited without writing one.
    BadTranscript { got: String },
    /// The fixture leader was still running when the budget expired.
    NotFinished { waited: Duration },
    /// The leader had exited on its own, but a reader was still waiting for
    /// end-of-file: a descendant of the fixture was holding the pipe open.
    ReaderStuck {
        stream: &'static str,
        waited: Duration,
    },
}

impl std::fmt::Display for FixtureFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FixtureFailure::NoTranscript { waited } => write!(
                f,
                "no transcript line on the fixture's stderr within {waited:?}"
            ),
            FixtureFailure::BadTranscript { got } => write!(
                f,
                "the transcript's first line must start with {TRANSCRIPT:?} under --json, got {got:?}"
            ),
            FixtureFailure::NotFinished { waited } => {
                write!(f, "the fixture did not finish within {waited:?}")
            }
            FixtureFailure::ReaderStuck { stream, waited } => write!(
                f,
                "the fixture exited but its {stream} was still open after {waited:?}: a descendant held it"
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

/// Did this status come from the harness's own `SIGKILL`?
#[cfg(unix)]
fn killed_by_the_harness(status: &std::process::ExitStatus) -> bool {
    use std::os::unix::process::ExitStatusExt;
    status.signal() == Some(9)
}

#[cfg(not(unix))]
fn killed_by_the_harness(_status: &std::process::ExitStatus) -> bool {
    true
}

/// A fixture process and its tree, terminated and reaped on drop.
///
/// Drop is what makes an assertion failure anywhere in a case clean up: the
/// panic unwinds through the harness, the tree is signalled, and the leader is
/// reaped before the test thread reports.
struct Fixture {
    child: std::process::Child,
    status: Option<std::process::ExitStatus>,
}

impl Fixture {
    fn spawn(mut cmd: Command) -> Fixture {
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
        Fixture {
            child: cmd.spawn().expect("the fixture must spawn"),
            status: None,
        }
    }

    /// Kill the whole tree and reap the leader. A no-op once the leader has
    /// been reaped, which is also where the pid-recycling invariant is kept:
    /// nothing signals a group whose leader is gone.
    fn terminate(&mut self) -> std::process::ExitStatus {
        if let Some(status) = self.status {
            return status;
        }
        signal_fixture_group(self.child.id());
        // Belt for the non-Unix fallback, and harmless where the group signal
        // already landed.
        let _ = self.child.kill();
        let status = self.child.wait().expect("the fixture leader must reap");
        self.status = Some(status);
        status
    }

    /// Has the leader exited? Reaps it if so, which is why no group signal may
    /// follow a `Some`.
    fn poll_exit(&mut self) -> Option<std::process::ExitStatus> {
        if self.status.is_some() {
            return self.status;
        }
        match self.child.try_wait().expect("try_wait on the fixture") {
            Some(status) => {
                self.status = Some(status);
                Some(status)
            }
            None => None,
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = self.terminate();
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
fn try_run_fixture(
    cmd: Command,
    close: bool,
    budget: Duration,
) -> Result<Consumed, FixtureFailure> {
    let started = Instant::now();
    let mut fixture = Fixture::spawn(cmd);

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
            fixture.terminate();
            return Err(FixtureFailure::NoTranscript { waited });
        }
    };
    match read {
        Ok(line) if line.starts_with(TRANSCRIPT) => line,
        Ok(line) => {
            fixture.terminate();
            return Err(FixtureFailure::BadTranscript { got: line });
        }
        Err(e) => {
            fixture.terminate();
            return Err(FixtureFailure::BadTranscript { got: e.to_string() });
        }
    };

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
/// descendant was holding the pipe.
fn stuck(fixture: &mut Fixture, started: Instant, stream: &'static str) -> FixtureFailure {
    let waited = started.elapsed();
    let status = fixture.terminate();
    if killed_by_the_harness(&status) {
        FixtureFailure::NotFinished { waited }
    } else {
        FixtureFailure::ReaderStuck { stream, waited }
    }
}

/// `verify <id> --json` under a stderr consumer, within the suite's deadline.
fn run_with_stderr_consumer(root: &Path, id: &str, close: bool) -> Consumed {
    let mut cmd = bin();
    cmd.arg("--repo").arg(root).args(["verify", id, "--json"]);
    try_run_fixture(cmd, close, DEADLINE).unwrap_or_else(|e| {
        panic!(
            "`verify {id}` with the stderr consumer {}: {e}",
            if close { "closed" } else { "open" }
        )
    })
}

/// Spec 118 D-3 on a quiet successful command: closing the consumer changes
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

/// Spec 118 §3.2 with the destination gone, on the child's **stderr**. This is
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
/// outcome spec 118 D-3 forbids.
#[test]
fn a_closed_stderr_consumer_does_not_hang_a_child_flooding_its_stdout() {
    let tmp = tempfile::tempdir().unwrap();
    write_spec(tmp.path(), "009-flood-out", VOLUME);

    let run = run_with_stderr_consumer(tmp.path(), "009-flood-out", true);

    assert_eq!(run.status.code(), Some(0));
    assert_eq!(run.envelope()["report"]["outcome"], "passed");
}

/// Spec 118 §3.4 with the destination gone: a failing command keeps its own
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
// The safeguards themselves (spec 118 D-9).
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

/// Run `f` on a worker thread and fail if it has not returned within `bound`.
///
/// The bound exists precisely for the case where the harness's own deadline is
/// broken, so it cannot be built from that deadline. A worker that overruns is
/// abandoned rather than joined; that is a harness defect being reported, and
/// the report is the point.
#[cfg(unix)]
fn within<T: Send + 'static>(bound: Duration, f: impl FnOnce() -> T + Send + 'static) -> T {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let _ = tx.send(f());
    });
    rx.recv_timeout(bound).unwrap_or_else(|_| {
        panic!("the fixture harness did not return within its outer bound of {bound:?}")
    })
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

    let started = Instant::now();
    let outcome = within(OUTER_BOUND, move || {
        try_run_fixture(cmd, true, SAFEGUARD_BUDGET)
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

    let started = Instant::now();
    let outcome = within(OUTER_BOUND, move || {
        try_run_fixture(cmd, true, SAFEGUARD_BUDGET)
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
