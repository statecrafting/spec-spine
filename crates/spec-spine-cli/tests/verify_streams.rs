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
use std::path::Path;
use std::process::{Command, Output};

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
