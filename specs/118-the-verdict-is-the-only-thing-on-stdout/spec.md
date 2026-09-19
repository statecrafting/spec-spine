---
id: "118-the-verdict-is-the-only-thing-on-stdout"
title: "The verdict is the only thing on stdout"
status: draft
kind: "tooling"
created: "2026-09-19"
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "035-stdout-closed-reader"
  - "037-machine-readable-verdicts"
  - "049-verify-declared-acceptance"
establishes:
  # 3.5: the regression suite for the channel contract. Its own file rather than
  # more of `tests/cli.rs`, because what it asserts is a property of the two
  # output channels and every case needs a child that writes to both; the
  # existing `verify` cases there all run `true`, which is exactly why none of
  # them could see this defect.
  - "crates/spec-spine-cli/tests/verify_streams.rs"
extends:
  # 3.1, 3.2, 3.3: the execution site and the two channels it writes.
  - { spec: "049-verify-declared-acceptance", unit: "crates/spec-spine-cli/src/cmd_verify.rs", nature: additive }
  # 3.4: the existing end-to-end `verify` cases gain the negative that the
  # envelope is the whole of stdout.
  - { spec: "001-compile-registry", unit: "crates/spec-spine-cli/tests/cli.rs", nature: additive }
references:
  # The envelope contract this brings `verify` back under, and the channel
  # discipline 035 3.3 set for diagnostics.
  - { unit: { kind: file, path: "specs/037-machine-readable-verdicts/spec.md" }, role: context }
  - { unit: { kind: file, path: "specs/035-stdout-closed-reader/spec.md" }, role: context }
summary: >
  Spec 037 3.1 requires that under `--json` a verb write exactly one JSON object
  to stdout, and `verify --json` does not: it spawns each acceptance command with
  stdout inherited, so every byte the command prints lands on the parent's stdout
  ahead of the envelope. The failure is quiet in the worst way, because the
  envelope is still emitted and still correct and the exit code is still right,
  so a consumer sees a successful run whose document will not parse. Measured on
  `5f95613`, a one-command block printing a single line makes `json.load` of the
  whole stdout fail at character 0 while the process exits 0. `verify` is the
  verb an orchestrator most needs to read programmatically, which is the third
  argument spec 049 1.2 used to justify building it, so the one verb that had to
  be machine-readable is the one that is not. The correction keeps the child's
  bytes and moves them: under `--json` both child streams are streamed to the
  parent's stderr as they arrive, and the command and exit transcript spec 049
  3.5 requires joins them there, so stdout carries the envelope and nothing else
  on the report path and on the error path alike. Nothing is captured in memory,
  no field joins the verdict schema, and without the flag every byte goes where
  it goes today.
---

# 118: The verdict is the only thing on stdout

## 1. Purpose

Spec 037 3.1 is unambiguous: "Under `--json` a verb MUST write exactly one JSON
object to stdout." Spec 049 gave `verify` that envelope, and the CLI runs each
acceptance command like this:

```rust
let status = Command::new("sh")
    .arg("-c")
    .arg(command)
    .current_dir(repo)
    .env(STACK_VAR, &child_stack)
    .status()
```

`Command::status` inherits the parent's stdout. So the child writes to the same
handle the envelope will be written to, and it writes there first.

### 1.1 Measured, on the default branch

A spec whose block is one command printing one line, run against `5f95613` with
`spec-spine 0.20.0`:

```
$ spec-spine --repo "$F" verify 001-noisy --json > out 2> err; echo $?
0
$ cat out
CHILD-STDOUT-LINE
{
  "exitCode": 0,
  "ok": true,
  "report": { "declared": true, "outcome": "passed", "ran": 1, ... },
  "schemaVersion": "0.4.0",
  "verb": "verify"
}
$ python3 -c 'import json,sys; json.load(open("out"))'
json.decoder.JSONDecodeError: Expecting value: line 1 column 1 (char 0)
```

Everything about that run is right except the channel. The verdict is correct,
`ok` agrees with the exit code, the report carries the facts spec 049 3.4 lists,
and the process exits 0. The only thing wrong is that the document a consumer
was promised is not the content of the stream it was promised on.

That is what makes this worth a spec rather than a patch. A consumer reading the
exit code alone sees a pass. A consumer reading the document sees a parse error
on a successful run, and the obvious diagnosis, that `verify` emitted malformed
JSON, is wrong: the JSON is canonical and valid, and the bytes in front of it
belong to somebody else. The cost of the defect is paid by whoever debugs it.

### 1.2 Why the existing tests could not see it

`verify_json_is_a_verdict_envelope_that_agrees_with_the_exit_code` already
parses stdout with `serde_json::from_slice` and already asserts
`"stdout is one JSON envelope"` in its expect message. It passes, because every
command in its fixtures is `true`, `exit 7` or a `verify:browser` fence that is
never run. Not one of them writes a byte. The assertion is real and the fixture
never exercised it, which is the shape spec 106 and spec 111 both name: an
assertion that cannot fail against the input it is given asserts nothing about
the input it will meet.

### 1.3 Spec 049 3.5 asks for a transcript and gets none here

Spec 049 3.5: the CLI "MUST echo each command and its exit code, so that a CI
log reads as a transcript." The requirement names no channel, and the
implementation reads it as a stdout requirement, so under `--json` it is
suppressed entirely: both echo sites sit inside `if !json`. A machine-readable
run therefore has no transcript at all, which is the mode a CI log is most
likely to be produced in.

Both halves of that are the same mistake about channels, so one spec fixes both.
The transcript is a diagnostic, and spec 035 3.3 already decided where the CLI's
diagnostics go.

## 2. Territory

`crates/spec-spine-cli/src/cmd_verify.rs`, established by spec 049, extended
here: how a child is spawned under `--json`, and where the transcript is
written. One new test file, `crates/spec-spine-cli/tests/verify_streams.rs`,
holding the channel regressions. One addition to the `verify` cases in
`crates/spec-spine-cli/tests/cli.rs`, so the suite that already claims stdout is
one envelope asserts it against a child that writes.

No engine module changes: the seam spec 049 3.1 draws is untouched, and
subprocess execution stays in the CLI. No DTO and no schema version moves.

## 3. Behavior

### 3.1 Under `--json`, stdout carries the envelope and nothing else

Under `--json`, the CLI MUST NOT let an acceptance command's output reach the
parent's stdout. Each child MUST be spawned with its stdout and its stderr
captured, and the bytes of both MUST be written to the parent's **stderr**.

The requirement is the whole of stdout, not the ordinary path only. On the
report path stdout holds one envelope; on an error path (spec 037 3.3, for
instance the `R-001` re-entry refusal of spec 049 3.7, or an `Error::Io` from a
spawn that fails) stdout holds one error envelope and nothing else.

Diverting at the spawn is what makes that total, and the distinction is worth
stating because the two error paths sit on opposite sides of execution. `R-001`
is decided before any command runs, so nothing has been printed yet; a spawn
failure can happen after an earlier command has already printed, and there is no
other error reachable once the loop has started. A correction applied at the
write site would have to reason about which of those it was in. A child that
never holds the parent's stdout descriptor makes the question moot, and that is
why 3.1 is a requirement about how the child is spawned rather than about what
the CLI writes.

### 3.2 The bytes are streamed, not accumulated

Both captured streams MUST be forwarded as they arrive. The CLI MUST NOT read
either stream to end-of-file into memory before forwarding it, and MUST NOT
buffer a command's output to disk.

Two reasons, and neither is style. An acceptance command here is `cargo test`
over a workspace; its output is unbounded in principle and megabytes in
practice, and a verb that grew with it would be a new failure mode traded for
the one being fixed. And a pipe has a finite buffer: a child that fills its
stdout pipe blocks until somebody reads it, so a forwarder that waits for the
child to exit before reading would deadlock on exactly the verbose command this
verb exists to run. Reading both streams concurrently with the child's execution
is therefore a correctness requirement, not a performance preference.

Forwarding and draining are separate obligations, and only the first of them
may be given up. If the parent's stderr stops accepting bytes, the CLI MUST
still read both child pipes to end-of-file, discarding what it cannot deliver,
and MUST NOT close either pipe because a write failed. D-3 excuses delivering
the bytes; nothing excuses draining the pipe, because a child blocked writing
into a pipe nobody is reading never reaches an exit status, and the verdict is
computed from exit statuses. For the same reason a transcript write under
`--json` MUST NOT panic on a failed write. D-4 records what each of those cost
when it was not done.

The two streams are buffered independently by the operating system, and this
spec promises **no** ordering between them. A command's interleaved stdout and
stderr may appear on the parent's stderr in an order neither the command nor the
CLI chose. That is a real cost of the correction and it is accepted: it is
inherent to capturing two pipes, and a contract that promised an order would be
a contract the implementation could not keep.

### 3.3 The transcript is a diagnostic, and under `--json` it goes to stderr

The transcript spec 049 3.5 requires MUST be emitted in both modes. Its channel
is the mode's: without `--json` it stays on stdout, exactly as today; with
`--json` it MUST go to stderr, where it sits with the child's own bytes and
where spec 035 3.3 puts every CLI diagnostic.

This clarifies spec 049 3.5 rather than changing it. That clause requires the
echo and names no channel; read as a requirement about stdout it is
unsatisfiable under `--json` alongside spec 037 3.1, and the implementation
resolved the conflict by dropping the requirement in that mode. Naming the
channel per mode satisfies both clauses instead, so no approved requirement is
withdrawn and spec 049's file is not edited.

The transcript's content is unchanged: the command as it will be run, and the
exit code it returned, or that it was killed by a signal.

### 3.4 Everything else is preserved, and that is a requirement

The following MUST be unchanged by this spec, and each is a line in the
acceptance rather than an assurance:

- **Without `--json`, nothing moves.** The child inherits both of the parent's
  streams, the transcript is on stdout, and the byte stream of a prose run is
  what it is today.
- **Execution.** Commands run through `sh -c`, from the repository root, in plan
  order, stopping at the first non-zero exit, with the environment inherited
  (spec 049 3.5).
- **The report.** Every field spec 049 3.4 lists, `ran` and `total` included,
  and a failing command's own exit code in `failure.exitCode` and never in the
  process's status (spec 049 3.3).
- **The exit-code mapping.** `passed` and `not-declared` are 0, `failed` is 1, a
  missing spec is 1.
- **Recursion protection.** `SPEC_SPINE_VERIFY_STACK` is set on every child and
  re-entry is refused with `R-001` (spec 049 3.7). A refusal under `--json` is
  an error envelope, and 3.1 applies to it.
- **`--plan --json` executes nothing.** It emits the `VerifyPlan` in the
  envelope (spec 049 3.8) and spawns no process, so 3.1 holds there trivially
  and the fixture's side effect must be absent afterwards.
- **The seam.** Subprocess execution stays in `spec-spine-cli`. `core::verify`
  remains a pure function of `(Config, file contents)`.

### 3.5 Tests (minimum)

In `tests/verify_streams.rs`, each case running a child that writes to **both**
streams, which is the property 1.2 found missing:

1. A passing command: exit 0, stdout parses whole as one envelope with
   `outcome: "passed"`, the child's stdout bytes are absent from stdout and
   present on stderr, and its stderr bytes are on stderr.
2. A failing command: exit 1, stdout parses whole as one envelope,
   `failure.exitCode` is the command's own code, `failure.index` and `command`
   name it, `ran` stops there, both of the failing command's streams are on
   stderr, and a later command in the same block demonstrably did not run
   (asserted by the absence of its file side effect, not by counting output).
3. The transcript is on stderr under `--json`: the command line and its exit
   code, and neither on stdout.
4. `--plan --json`: stdout is one envelope, and the fixture's side-effect file
   does not exist.
5. Without `--json`, the child's stdout is on the parent's stdout and the
   transcript is there too: the preservation half of 3.4.
6. A consumer that reads the opening transcript line and then **closes** the
   parent's stderr, with a bounded deadline, an asserted process status, and
   the whole of stdout parsed as one envelope: a quiet successful command, a
   command whose output exceeds a pipe buffer on its stdout, the same on its
   stderr, and a failing command that keeps its own exit code and position and
   still stops the block. An open-consumer control runs the same volume, so a
   pass cannot be explained by a fixture that wrote nothing. The harness bounds
   the **whole** fixture lifecycle under one budget taken before the spawn:
   start-up, the opening transcript line, completion, and the shutdown of the
   reader threads. Fixtures are spawned into a process group of their own, and
   a timeout or an assertion failure terminates that group and reaps the
   leader, so nothing is left to a broken pipe to propagate to a descendant
   (D-6).
7. An error path under `--json`: the `R-001` refusal of spec 049 3.7 puts one
   error envelope on stdout, carrying `error.kind: "validation"` and the
   `R-001` violation, with no `report` member and nothing else on the stream.
   The spawn-failure path is not tested, because making `sh` unspawnable is a
   property of the machine rather than of a fixture; 3.1 covers it by
   construction, since the child that would have failed to spawn is the one
   configured not to hold the parent's stdout.
8. The two safeguards in 6, exercised on their own terms. Neither case runs
   `verify`: what is under test is the harness, and a correct `verify` cannot
   produce either condition, so each fixture is a shell script chosen to break
   the harness in one specific way. A fixture that never writes a transcript
   line is given up at the budget rather than waited on for as long as it
   lives; a fixture that backgrounds a descendant which would write a file
   after a delay has that descendant terminated, witnessed by the file's
   absence checked after the moment it was scheduled for. Each runs under an
   **outer** bound that does not depend on the budget it is testing, so a
   broken deadline is reported rather than waited on. Both are Unix-only,
   which is where this suite is supported (D-6).

In `tests/cli.rs`, the existing envelope case gains a fixture whose command
writes to both streams, so the `"stdout is one JSON envelope"` expectation it
already carries is asserted against input that can violate it.

## 4. Out of scope

**A general process-output framework.** The forwarding is two streams for one
child inside one function. Nothing here introduces a reusable runner, an output
policy type, or a capture abstraction for other verbs; no other verb spawns a
process.

**Any change to the verdict schema.** No field is added, so
`VERDICT_SCHEMA_VERSION` does not move. In particular the child's output is not
put in the report: a log member would make the envelope grow without bound with
the output of `cargo test`, and the bytes already have a channel.

**Ordering between the two child streams**, per 3.2. A caller that needs
interleaving as the command produced it can have the command merge them itself
with `2>&1`.

**The prose mode's channels.** Without `--json` the child inherits both streams
and the transcript is on stdout. That is a shipped surface with users, and
moving it would be a breaking change for the benefit of symmetry alone.

**Spec 103 3.4's "acceptance amended by" notice.** It is printed only without
`--json` today, and it is left exactly there. It is not the transcript spec 049
3.5 requires, the fact it carries is reachable from the registry
(`registry show <id> --json` carries `amendsVerification`), and putting it on
stderr under `--json` would be a change to spec 103's requirement rather than a
clarification of spec 049's. If it is wanted there it is additive and separate.

**`verify`'s place outside the gate chain.** Unchanged, per spec 049 3.6.

## 5. Resolved decisions

- **D-1 (2026-09-19): the transcript's channel is per mode, not global.** 3.3
  could have put the transcript on stderr in both modes, which would be one
  channel for one kind of output and simpler to describe. Rejected: the prose
  mode's stdout transcript is what `verify` has printed since spec 049 shipped,
  it is what a developer reads, and moving it would break every caller that
  redirects stdout to a log for the reason 3.4 gives. The asymmetry is the
  mode's, and the mode is the thing a caller chose.

- **D-2 (2026-09-19): forwarded with a thread per stream rather than by handing
  the child the parent's stderr descriptor.** Duplicating the parent's stderr
  file descriptor into the child's two `Stdio` slots would need no forwarding at
  all, and on Unix it is reachable safely. Rejected for portability: it is a
  platform-conditional spawn on a binary released for four triples, and the
  condition would be untested on three of them. Forwarding concurrently is one
  code path everywhere, and 3.2's deadlock argument requires concurrency in any
  case.

- **D-3 (2026-09-19): a failed write of the forwarded bytes is not a failure of
  the verb.** If writing a child's output to the parent's stderr fails, the
  forward is abandoned and the command's exit status is still read and reported.
  This follows spec 035 3.3: a process whose stderr has gone has no channel left
  to report that fact, and failing the acceptance run because its log could not
  be written would report a defect in the corpus that is not there. The verdict
  is computed from exit statuses, which are unaffected.

- **D-4 (2026-09-19): D-3 excuses delivering the bytes, never draining the
  pipe.** The first implementation of D-3 read "abandon the forward" as "stop
  reading", and that is a different promise. Measured on the merged
  implementation at `71a423a`, with a consumer that reads the opening
  transcript line and then closes the parent's stderr:

  - a child writing ~1.3 MB to its stderr does not complete at all (killed at a
    30 s deadline; the same child with the consumer open completes in ~0.5 s and
    exits 0 with a valid envelope). `io::copy` returned on the first failed
    write, the child's pipe was left undrained, the child blocked filling it,
    and `child.wait()` blocked on the child.
  - `sleep 0.2; true` exits **101** with an empty stdout. The `[verify] exit 0`
    transcript line went through `eprintln!`, which unwraps its write, so the
    process panicked before the envelope was produced.
  - the same volume on the child's *stdout* exits **101** by a third route: the
    forwarding thread owned its end of that pipe and dropped it on the failed
    write, which handed the child an `EPIPE` and changed the child's own
    outcome.

  Each of those makes an undeliverable log decide the verdict, which is exactly
  what D-3 forbids; the clause is not weakened here, it is implemented. So:
  when the destination fails, both child pipes MUST still be read to
  end-of-file with the undeliverable bytes discarded, neither pipe may be closed
  because forwarding failed, and a transcript write under `--json` MUST NOT
  panic on a failed write, for the reason spec 035 §3.2 gives about stdout.
  Draining continues through a fixed buffer per stream, so §3.2's bound and its
  concurrency requirement are unchanged.

- **D-5 (2026-09-19): a destination failure, a forwarding panic and an
  unreadable pipe are three facts, and the verb does not report them as one.**
  The forwarding thread's join result was discarded, so a panic in it was
  indistinguishable from a stderr that stopped accepting bytes. They are not the
  same: a panic is a defect in this CLI and an unreadable pipe is an
  operating-system failure, and in neither case is there evidence that the
  parent's stderr failed. D-3 covers the first alone. The verb therefore keeps
  the three outcomes distinct and, for the two D-3 does not cover, writes a
  best-effort warning to stderr naming the stream and the condition (silent for
  a destination failure, which by definition has nowhere to be reported).

  One stream can meet two of these: stderr stops accepting bytes and the
  child's pipe then becomes unreadable. The read failure is reported, because
  it is the one whose consequence is unknown and the one D-3 does not excuse.
  That precedence is stated rather than left to fall out of the code, since it
  is the single place this clause's three outcomes collapse to two.

  **What is left open, deliberately:** whether a forwarding panic or an
  unreadable child pipe should affect the acceptance verdict. Today none of the
  three does, which preserves the shipped behaviour and D-3's reasoning for the
  one case D-3 actually decided. Deciding the other two is a policy change
  rather than a correction, it is not needed to fix what §1.1 and D-4 measured,
  and it is left to a later spec.

- **D-6 (2026-09-19): the regression harness's own deadline and cleanup were
  both narrower than the sentence describing them.** §3.5 item 6 said the
  harness "kills and reaps on a timeout rather than leaving a hung fixture
  behind", and the harness as merged at `4d2dbb8` did neither completely. Two
  gaps, both corrected here, and neither touching the verb:

  - **The budget did not cover the start of the run.** The opening
    `read_line` on the fixture's stderr happened before the deadline loop was
    entered, so a fixture that never writes a transcript line was waited on for
    as long as it lived, outside the bound. The Python acceptance watchdogs
    below had the same shape: the `threading.Timer` was started *after* the
    initial `readline`. Measured, with the harness's initial read made
    unbounded again and a silent `sh -c 'sleep 600'` fixture: the case does not
    return, and the outer bound reports it at 40.01 s. With the correction the
    same fixture is given up inside its 2 s budget.
  - **Cleanup reached the leader only.** A timeout killed and waited for the
    fixture process and nothing else, which leaves a descendant of it running;
    an isolated probe measured one writing a file after the leader had been
    killed and reaped. Measured here, with the group signal replaced by a
    no-op: `a_timed_out_fixtures_descendant_is_terminated_before_its_side_effect`
    fails on its witness, `a descendant of the fixture survived cleanup and
    performed its side effect`. With the correction the file is absent when
    checked two seconds past the moment it was scheduled for.

  So: the budget is taken before the spawn and covers start-up, the initial
  transcript acquisition, completion and reader shutdown; fixtures are spawned
  into a process group of their own (`Command::process_group(0)`); and a
  timeout, a rejected transcript or an assertion failure anywhere in a case
  terminates that group and reaps the leader, the last of these through the
  fixture guard's `Drop`. Termination is a group signal, never a broken pipe
  left to propagate: a descendant holding the leader's pipe open is exactly the
  case a broken pipe does not reach, and it is also why the harness waits for
  its readers *before* it reaps the leader.

  **The invariant that keeps the group signal safe:** nothing signals a process
  group whose leader has been reaped. While the leader is unreaped its pid
  cannot be recycled, so the negative pgid cannot have come to name some other
  tree. Every path that learns the leader's status either signals first or has
  already established that the tree is finished.

  **What that leaves unestablished, stated rather than implied:** a fixture
  that exits on its own while leaving a descendant behind is not cleaned up by
  this, because by then the leader is reaped and the signal would no longer be
  safe to send. The harness detects that case (a reader still waiting for
  end-of-file after a leader that exited) and reports it as a failure rather
  than silently passing; no fixture in this suite produces it.

  **Platform handling.** The group signal is `kill -9 -<pgid>` through
  `/bin/sh`, rather than a `libc` dependency taken for one signal; the process
  group is set with the standard library's `Command::process_group`. Both are
  `#[cfg(unix)]`, as are the two safeguard cases of §3.5 item 8, and the
  non-Unix fallback kills the leader alone. This suite's supported environments
  are Unix: CI is `ubuntu-latest` and development is macOS. The acceptance
  watchdogs use `start_new_session=True` and `os.killpg` for the same reason,
  guarded by `p.poll() is None` so they keep the same invariant, and registered
  with `atexit` so an assertion failure cleans up too. The timer thread is a
  daemon, so a failing assertion is not made to wait out the watchdog.

  This is a correction to test infrastructure and to the acceptance commands
  that mirror it. No requirement in §3 is weakened, no verdict semantics move,
  and no process supervision is introduced into the verb: §4's "a general
  process-output framework" stays out of scope, and the cleanup here is a
  fixture guard in one test file, not a runner.

  **D-5 is untouched.** Whether a forwarding panic or a failure to read a
  child's pipe should change the acceptance verdict is still open, and nothing
  here treats either as equivalent to a destination-write failure.

## Verification

Each line is one command and no shell variable survives to the next, so every
line that needs a fixture builds its own. The inner fences are written with
octal escapes (`\140`) so that a fixture's `verify:cli` fence cannot be read as
the end of this one, and a fixture's leading frontmatter marker with `\055`,
because a format string beginning `---` is read by `printf` as an option.

Each marker is assembled by the fixture's own command (`printf %sOISEOUTONE N`,
written `%%s` in the outer format) rather than spelled literally. A literal
marker appears in the transcript on stderr and in the envelope's
`failure.command` on stdout, so `grep -q MARKER "$err"` would pass on the echo
of the command instead of on the forwarded byte, and `! grep -q MARKER "$out"`
would fail on the envelope's own payload. Both were measured while building
this, and an assertion that a channel carries a marker the other channel also
names is not an assertion about the channel.

```verify:cli
# The block drives the release binary, and `cargo test` builds only debug
# artifacts, so it is built first.
cargo build --release --locked
# 3.5, D-4 + D-5's closed-consumer regressions, and D-6's two safeguard cases,
# which live in the same file.
# Of the four closed-consumer cases two fail at `71a423a` by exiting 101 and two
# by hitting the suite's 30 s deadline, while the open-consumer control passes
# there, so the suite distinguishes the correction from the fixture. The whole
# file is run rather than a name filter, which would pass on matching nothing.
cargo test -p spec-spine-cli --test verify_streams --locked
cargo test -p spec-spine-cli --test cli --locked
# 3.1 + 3.2 + 3.3, end to end on a passing command that writes to both streams:
# the whole of stdout is one envelope, the child's stdout is on stderr and not on
# stdout, its stderr is on stderr, and the transcript is on stderr too. Red at
# 5f95613, where the first `python3` fails at character 0 (1.1).
T=$(mktemp -d) && mkdir -p "$T/specs/001-noisy" && printf '\055\055\055\nid: "001-noisy"\ntitle: "T"\nstatus: approved\ncreated: "2026-09-19"\nsummary: "s"\n---\n# 001-noisy\n\n## Verification\n\n\140\140\140verify:cli\nprintf %%sOISEOUTONE N; printf %%sOISEERRONE N >&2\n\140\140\140\n' > "$T/specs/001-noisy/spec.md" && target/release/spec-spine --repo "$T" verify 001-noisy --json > "$T/out" 2> "$T/err" && python3 -c "import json,sys; d=json.load(open('$T/out')); assert d['verb']=='verify', d; assert d['report']['outcome']=='passed', d; assert d['exitCode']==0, d" && ! grep -q NOISEOUTONE "$T/out" && grep -q NOISEOUTONE "$T/err" && grep -q NOISEERRONE "$T/err" && grep -q 'verify. \$ printf %sOISEOUTONE N' "$T/err" && grep -q 'verify. exit 0' "$T/err" && rm -rf "$T"
# 3.1 + 3.4, on a failing command: exit 1, the whole of stdout is one envelope,
# the failing command's own code and position are in the payload, both of its
# streams are on stderr, and the next command in the block did not run. Red at
# 5f95613 on the `python3` line for the same reason.
T=$(mktemp -d) && mkdir -p "$T/specs/002-fail" && printf '\055\055\055\nid: "002-fail"\ntitle: "T"\nstatus: approved\ncreated: "2026-09-19"\nsummary: "s"\n---\n# 002-fail\n\n## Verification\n\n\140\140\140verify:cli\nprintf %%sOISEOUTTWO N; printf %%sOISEERRTWO N >&2; exit 7\nprintf c > ran-c.txt\n\140\140\140\n' > "$T/specs/002-fail/spec.md"; target/release/spec-spine --repo "$T" verify 002-fail --json > "$T/out" 2> "$T/err"; c=$?; test "$c" -eq 1 && python3 -c "import json,sys; d=json.load(open('$T/out')); assert d['exitCode']==1, d; assert d['report']['failure']['exitCode']==7, d; assert d['report']['failure']['index']==1, d; assert d['report']['ran']==1, d; assert d['report']['total']==2, d" && ! grep -q NOISEOUTTWO "$T/out" && grep -q NOISEOUTTWO "$T/err" && grep -q NOISEERRTWO "$T/err" && test ! -f "$T/ran-c.txt" && rm -rf "$T"
# 3.4: `--plan --json` is one envelope and runs nothing, so the side effect the
# fixture would have had is absent.
T=$(mktemp -d) && mkdir -p "$T/specs/003-plan" && printf '\055\055\055\nid: "003-plan"\ntitle: "T"\nstatus: approved\ncreated: "2026-09-19"\nsummary: "s"\n---\n# 003-plan\n\n## Verification\n\n\140\140\140verify:cli\nprintf ran > side_effect.txt\n\140\140\140\n' > "$T/specs/003-plan/spec.md" && target/release/spec-spine --repo "$T" verify 003-plan --plan --json > "$T/out" 2> "$T/err" && python3 -c "import json,sys; d=json.load(open('$T/out')); assert d['verb']=='verify', d; assert d['report']['commands']==['printf ran > side_effect.txt'], d" && test ! -f "$T/side_effect.txt" && rm -rf "$T"
# 3.1's error path: the `R-001` refusal is one error envelope and the whole of
# stdout. Green at 5f95613, where the refusal precedes execution: preservation,
# not evidence, and the line is here because 3.1 is a claim about every path.
T=$(mktemp -d) && mkdir -p "$T/specs/004-loop" && printf '\055\055\055\nid: "004-loop"\ntitle: "T"\nstatus: approved\ncreated: "2026-09-19"\nsummary: "s"\n---\n# 004-loop\n\n## Verification\n\n\140\140\140verify:cli\nprintf refusal-never-runs\n\140\140\140\n' > "$T/specs/004-loop/spec.md"; SPEC_SPINE_VERIFY_STACK=004-loop target/release/spec-spine --repo "$T" verify 004-loop --json > "$T/out" 2> "$T/err"; c=$?; test "$c" -eq 1 && python3 -c "import json,sys; d=json.load(open('$T/out')); assert d['error']['kind']=='validation', d; assert 'report' not in d, d; assert d['error']['violations'][0]['code']=='R-001', d" && rm -rf "$T"
# 3.4's preservation half: without the flag the child's stdout is on the
# parent's stdout, and so is the transcript. Green at 5f95613 on purpose; this
# line is what stops the correction from moving the prose mode too.
T=$(mktemp -d) && mkdir -p "$T/specs/005-prose" && printf '\055\055\055\nid: "005-prose"\ntitle: "T"\nstatus: approved\ncreated: "2026-09-19"\nsummary: "s"\n---\n# 005-prose\n\n## Verification\n\n\140\140\140verify:cli\nprintf %%sOISEOUTFIVE N; printf %%sOISEERRFIVE N >&2\n\140\140\140\n' > "$T/specs/005-prose/spec.md" && target/release/spec-spine --repo "$T" verify 005-prose > "$T/out" 2> "$T/err" && grep -q NOISEOUTFIVE "$T/out" && grep -q NOISEERRFIVE "$T/err" && grep -q 'verify. \$ printf %sOISEOUTFIVE N' "$T/out" && grep -q 'verify. exit 0' "$T/out" && grep -q 'passed (1 command(s))' "$T/out" && rm -rf "$T"
# D-4 end to end, on a quiet successful command. The consumer reads the opening
# transcript line and then closes the parent's stderr, so what is exercised is
# completion rather than start-up. Red at `71a423a`, where the `[verify] exit 0`
# line went through `eprintln!` and panicked: exit **101**, stdout empty. The
# watchdog kills and reaps rather than leaving a hung fixture behind, and it is a
# `threading.Timer` rather than `timeout(1)`, which macOS does not ship. The
# watchdog is armed **before** the initial `readline`, so a fixture that never
# writes a transcript is bounded too, and it signals the fixture's process group
# (`start_new_session=True` + `os.killpg`) so descendants go with it (D-6). The
# `p.poll() is None` guard keeps D-6's invariant: nothing signals a group whose
# leader has been reaped. `atexit` repeats the cleanup when an assertion fires,
# and the timer thread is a daemon so a failing assertion is not made to wait it
# out.
T=$(mktemp -d) && mkdir -p "$T/specs/006-closed" && printf '\055\055\055\nid: "006-closed"\ntitle: "T"\nstatus: approved\ncreated: "2026-09-19"\nsummary: "s"\n---\n# 006-closed\n\n## Verification\n\n\140\140\140verify:cli\nsleep 0.2; true\n\140\140\140\n' > "$T/specs/006-closed/spec.md" && python3 -c "import atexit,json,os,signal,subprocess,threading; p=subprocess.Popen(['target/release/spec-spine','--repo','$T','verify','006-closed','--json'],stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=True); k=lambda: p.poll() is None and os.killpg(p.pid,signal.SIGKILL); atexit.register(lambda: p.poll() is None and (os.killpg(p.pid,signal.SIGKILL) or p.wait())); t=threading.Timer(30,k); t.daemon=True; t.start(); f=p.stderr.readline(); assert f.startswith(b'[verify] '),f; p.stderr.close(); o=p.communicate()[0]; t.cancel(); assert p.returncode==0,p.returncode; d=json.loads(o); assert d['report']['outcome']=='passed',d" && rm -rf "$T"
# D-4's other half, on volume: ~1 MB on the child's stderr, well past any pipe
# buffer. Red at `71a423a`, where `io::copy` returned on the first failed write
# and left the pipe undrained while `wait` blocked on a child blocked filling it;
# the watchdog fires and the assertion reads -9 rather than 0.
T=$(mktemp -d) && mkdir -p "$T/specs/007-flood" && printf '\055\055\055\nid: "007-flood"\ntitle: "T"\nstatus: approved\ncreated: "2026-09-19"\nsummary: "s"\n---\n# 007-flood\n\n## Verification\n\n\140\140\140verify:cli\nawk %sBEGIN{s=sprintf("%%1000s","");gsub(/ /,"X",s);for(i=0;i<1000;i++)print s}%s >&2\n\140\140\140\n' "'" "'" > "$T/specs/007-flood/spec.md" && python3 -c "import atexit,json,os,signal,subprocess,threading; p=subprocess.Popen(['target/release/spec-spine','--repo','$T','verify','007-flood','--json'],stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=True); k=lambda: p.poll() is None and os.killpg(p.pid,signal.SIGKILL); atexit.register(lambda: p.poll() is None and (os.killpg(p.pid,signal.SIGKILL) or p.wait())); t=threading.Timer(30,k); t.daemon=True; t.start(); f=p.stderr.readline(); assert f.startswith(b'[verify] '),f; p.stderr.close(); o=p.communicate()[0]; t.cancel(); assert p.returncode==0,p.returncode; d=json.loads(o); assert d['report']['outcome']=='passed',d" && rm -rf "$T"
# The control the two lines above are measured against: the same volume with the
# consumer left open. Every byte is delivered and the verdict is the same one, so
# a pass above cannot be explained by a fixture that never wrote anything. The
# forwarded lines are counted rather than the file weighed: a byte threshold
# just under one stream's 1_001_000 clears by about a thousand bytes, thin
# enough that a longer fixture command would turn a correct run red. Green at
# `71a423a` on purpose: preservation, not evidence.
T=$(mktemp -d) && mkdir -p "$T/specs/008-open" && printf '\055\055\055\nid: "008-open"\ntitle: "T"\nstatus: approved\ncreated: "2026-09-19"\nsummary: "s"\n---\n# 008-open\n\n## Verification\n\n\140\140\140verify:cli\nawk %sBEGIN{s=sprintf("%%1000s","");gsub(/ /,"X",s);for(i=0;i<1000;i++)print s}%s >&2\n\140\140\140\n' "'" "'" > "$T/specs/008-open/spec.md" && target/release/spec-spine --repo "$T" verify 008-open --json > "$T/out" 2> "$T/err" && python3 -c "import json; d=json.load(open('$T/out')); assert d['report']['outcome']=='passed',d; n=sum(1 for l in open('$T/err') if l.rstrip(chr(10))=='X'*1000); assert n==1000,n" && rm -rf "$T"
# D-6, on the watchdog itself, and it needs no `spec-spine`: the fixture is a
# shell script that writes nothing at all and backgrounds a descendant which
# would write a file six seconds later. Both safeguards are exercised at once.
# `readline` returns only because the watchdog fired during it, which is the
# half that was unbounded; `$T/delayed` is absent when checked past the moment
# it was scheduled for, which is the half that reached the leader only. Measured
# with the group signal replaced by a parent-only `p.kill()`: the fixture's own
# `sleep 600` survives holding both pipes, `readline` never returns, and the
# line has to be killed by hand.
T=$(mktemp -d) && printf '( sleep 6; printf x > %s/delayed ) &\nsleep 600\n' "$T" > "$T/fx.sh" && python3 -c "import atexit,os,signal,subprocess,threading,time; p=subprocess.Popen(['/bin/sh','$T/fx.sh'],stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=True); k=lambda: p.poll() is None and os.killpg(p.pid,signal.SIGKILL); atexit.register(lambda: p.poll() is None and (os.killpg(p.pid,signal.SIGKILL) or p.wait())); t=threading.Timer(2,k); t.daemon=True; t.start(); s=time.monotonic(); f=p.stderr.readline(); p.stderr.close(); o=p.communicate()[0]; e=time.monotonic()-s; assert f==b'',f; assert p.returncode==-9,p.returncode; assert e<20,e" && sleep 7 && test ! -f "$T/delayed" && rm -rf "$T"
# The seam spec 049 3.1 draws is still drawn: the engine spawns nothing.
test "$(grep -rl 'std::process::Command' crates/spec-spine-core/src crates/spec-spine-types/src | wc -l | tr -d ' ')" = "0"
```
