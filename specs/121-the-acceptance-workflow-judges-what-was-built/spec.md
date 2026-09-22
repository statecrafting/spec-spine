---
id: "121-the-acceptance-workflow-judges-what-was-built"
title: "The Acceptance workflow judges what was built"
status: draft
kind: "tooling"
created: "2026-09-22"
summary: >
  The nightly `Acceptance` leg exits on the sweep's corpus verdict, so it is
  red for as long as any draft is filed ahead of its build, which is this
  repository's normal state. A signal that is always red cannot tell a reader
  that an implemented spec's acceptance just broke. Spec 119 built the release
  verdict and left the workflow's choice open. This spec takes it: both legs
  exit on the release verdict, the full selected corpus still runs, and a new
  report step puts both verdicts, every release failure and every pending
  block's real outcome into the run's summary and annotations. A missing,
  unreadable or inconsistent report fails the job on its own.
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "099-a-merged-acceptance-is-asked-again"
  - "119-a-release-verdict-judges-what-was-built"
  - "120-a-scoped-sweep-judges-the-binary-the-blocks-name"
extends:
  # 3.1, 3.2: both legs pass --release, and the report step runs after them.
  - { spec: "099-a-merged-acceptance-is-asked-again", unit: { kind: file, path: ".github/workflows/acceptance.yml" }, nature: corrective }
  # 3.5: the regressions, next to spec 089's and 119's.
  - { spec: "089-nothing-reruns-a-merged-acceptance", unit: { kind: file, path: "scripts/test-verify-sweep.py" }, nature: additive }
establishes:
  # 3.3, 3.4: the report step's reader.
  - { kind: file, path: "scripts/acceptance-report.py" }
references:
  - { unit: { kind: file, path: "scripts/verify-sweep.sh" }, role: context }
  - { unit: { kind: file, path: "docs/release-candidate-0.22.0.md" }, role: context }
---

# 121: The Acceptance workflow judges what was built

## 1. Purpose

### 1.1 A signal that is always red

`Acceptance`'s nightly leg sweeps the whole corpus and, since spec 099 D-4,
its step status is the sweep's exit code. Without `--release` that code is the
corpus verdict (spec 089 3.3): clean only when every selected spec is `passed`
or `exempt`.

This repository files a spec's acceptance before the spec is built and writes
it to fail first (`AGENTS.md`). So any legitimate pending draft keeps the
nightly red. The run at `3d4f3902` (run `35723398403`) was red on three specs;
after spec 118 corrected one of them, two remained, both `draft` with
`implementation: pending` (102 and 103). Nothing about the corpus was wrong,
and the job would have stayed red until an unrequested feature was built or a
failure was hidden.

A permanently red job is worse than no job. It trains its reader to stop
looking, and a regression in an implemented spec, which is the event spec 099
exists to surface, arrives as one more red run among many.

### 1.2 What spec 119 left open

Spec 119 added the release verdict (judged over built specs, pending work run
and listed but neither success nor failure) and `--release` to exit on it. Its
section 4 kept the workflow out of scope: "whether the nightly should exit on
the release verdict is a separate decision about what that signal is for". The
owner took that decision on 2026-09-22, with these constraints, which section 3
turns into requirements:

- the full selected corpus still executes and is recorded;
- pending acceptance failures stay visible;
- implemented obligations and genuine execution or infrastructure failures are
  actionable;
- missing-plan, unknown-lifecycle, parse, interruption and not-run conditions
  are preserved;
- pending features do not become passes or legacy exemptions;
- the raw corpus verdict is not suppressed;
- the existing release-verdict mechanism is used, not a parallel one.

## 2. Territory

- `.github/workflows/acceptance.yml` (`extends` 099): both legs pass
  `--release`; the sweep step records its exit code; a report step follows.
- `scripts/acceptance-report.py` (established): reads the sweep's report and
  renders it into the run.
- `scripts/test-verify-sweep.py` (`extends` 089): the regressions.

## 3. Behavior

### 3.1 Both legs exit on the release verdict

Every `scripts/verify-sweep.sh` invocation in the workflow MUST pass
`--release`. The selection is unchanged: the push leg's scope (099 3.2) and the
nightly's whole corpus (099 3.3). Every selected spec's block still runs,
pending ones included (119 D-1), and its corpus outcome is still recorded.

The step's status MUST still be the sweep's exit code (099 D-4): `0` release
clean, `1` release not clean, `3` a refusal, and `130` / `143` an interruption.
The sweep step MUST record that code as a step output before it exits.

The push leg takes the same flag. A merge that files a new draft selects that
draft, and its fail-first block would otherwise make the push leg red for the
same reason the nightly was.

### 3.2 A report step that cannot be skipped by a red sweep

A step MUST follow the sweep, run whenever the sweep step ran (including when
it failed or was cancelled), and invoke `scripts/acceptance-report.py` with
the report path, the sweep's recorded exit code, and the job summary file. It
MUST NOT run when the sweep did not run (the push leg's empty scope, 099 3.2).

### 3.3 What the report step renders

From `sweep.json` (schema 1.1.0, spec 119 3.4), the report MUST write to the
job summary:

- both verdicts on their own lines, the release verdict named as the one that
  decided the job, and the corpus verdict named as the raw one, with its five
  counts;
- the sweep's own `sweep.md`, unmodified, so the release section, the pending
  list and the corpus's `Not passing` list all appear.

It MUST emit one workflow annotation per row:

- `error` for every row whose release outcome is `failed`, `not-declared` or
  `not-run`, naming the spec, the outcome, the lifecycle as read, the failure
  text and the log path;
- `warning` for every pending row whose corpus outcome is not `passed`, naming
  the spec, its lifecycle and the block's real outcome, and saying it is not a
  release obligation;
- `notice` for every pending row whose block passed: a fail-first block that
  has started passing before its build is a finding (119 D-1).

An `exempt` row gets no annotation; the ledger is closed (089 3.4).

### 3.4 The report step refuses what it cannot vouch for

`acceptance-report.py` MUST exit non-zero, after writing an `error` annotation
and a summary line saying why, when:

- the recorded sweep exit code is absent or not an integer (the step was
  killed before it recorded one);
- the sweep exit code is neither `0` nor `1` (a refusal or an interruption),
  whether or not a report exists;
- the report is missing, unreadable, or not a JSON object;
- its `schemaVersion` major is not `1`, or it lacks `verdicts`,
  `releaseCounts`, `counts` or `specs` (a report older than 1.1.0);
- its `mode` is not `release`;
- the exit code disagrees with the report's release verdict (`0` with a
  release verdict not `clean`, or `1` with it `clean`).

Otherwise it MUST exit `0`: the sweep step, not the report step, decides the
job, and a second red step for the same release failure would be noise. The
report step's failure is reserved for evidence that is missing or does not
agree with itself.

### 3.5 The regressions

`scripts/test-verify-sweep.py` MUST cover, with real sweeps in disposable
repositories:

- a corpus holding a failing pending draft, a failing implemented spec and a
  spec whose plan cannot be read: `--release` exits `1`, and the report step
  emits an `error` for both the implemented failure and the unreadable plan
  and a `warning` for the pending draft. The pending draft conceals neither.
- a corpus holding only a failing pending draft beside passing implemented
  specs: `--release` exits `0`, the corpus verdict stays `not-clean` in the
  report and in the rendered summary, and the pending draft is a `warning`.
- a pending block that passes is a `notice`.
- each refusal in 3.4: a missing report, an unparseable one, a 1.0.0-shaped
  one, a `corpus`-mode one, an absent exit code, exit `3` and exit `130`, and
  both directions of disagreement.

### 3.6 What does not change

The sweep (`verify-sweep.sh`), its outcomes, its ledger and both verdicts are
untouched. The trust boundary is untouched: no new trigger, no `pull_request`
leg, `contents: read`, no secret (099 3.1, 3.4). `verify` and every engine verb
are unchanged. The uploaded report artifact is unchanged.

## 4. Out of scope

- **Notification anywhere else.** 099 section 4 stands.
- **Reclassifying any spec.** The lifecycle is what the corpus declares.
- **The maintainer's release runbook.** `docs/releasing.md` already asks for
  `--release` and for the corpus counts next to it (119 3.7).

## 5. Resolved decisions

D-1 (2026-09-22, the release verdict decides the job; the corpus verdict is
reported, not dropped). The alternative, keeping the corpus exit and adding a
second job that exits on the release verdict, leaves one of the two jobs red
for as long as a draft is filed, which is the problem. Showing the corpus
verdict in the summary, the annotations and the uploaded report keeps it one
click away without letting it decide anything.

D-2 (2026-09-22, a separate script, not a flag on the sweep). The sweep is a
maintainer tool that runs anywhere; annotation syntax and a job summary file
are GitHub's. Keeping them out of `verify-sweep.sh` keeps the sweep's contract
the one spec 089 and 119 wrote, and keeps the workflow calling a definition
rather than restating one (099 3.2's rule).

D-3 (2026-09-22, the report step's `if` reads the sweep step's conclusion).
`always()` alone would run the reader after a failed build step, where no
sweep ran and no code was recorded; that would add a second red step saying
"no exit code" to a job whose real failure is the build. Skipping it when the
sweep step was skipped keeps 3.2's rule literal: it runs whenever the sweep
ran, and the push leg's empty scope skips both.

D-4 (2026-09-22, `--release` sits on the same line as the script name). The
acceptance asserts that no `scripts/verify-sweep.sh` line lacks the flag. A
flag on a continuation line would satisfy the workflow and defeat the check,
so the invocation is written to be checkable by the line it starts on.

## Verification

Written to fail against the tree it is filed on: the report script does not
exist and the workflow does not pass `--release`. Measured at the build: with
`not-run` removed from the reporter's error set, the concealment test fails
(one error where two are required); with the exit-code agreement check
removed, the refusal test fails. Both restored, all fifteen tests pass.

```verify:cli
cargo build --release --locked
test -f scripts/acceptance-report.py
python3 -m py_compile scripts/acceptance-report.py
# 3.1: every sweep invocation in the workflow asks for the release verdict.
test "$(grep -c 'scripts/verify-sweep.sh' .github/workflows/acceptance.yml)" -ge 2
test -z "$(grep 'scripts/verify-sweep.sh' .github/workflows/acceptance.yml | grep -v -- '--release')"
# 3.1, 099 D-4: the step status is still the sweep's, and the code is recorded.
grep -qF 'exit $rc' .github/workflows/acceptance.yml
grep -qF 'echo "rc=$rc" >> "$GITHUB_OUTPUT"' .github/workflows/acceptance.yml
# 3.2: the report step reads the recorded code.
grep -qF 'scripts/acceptance-report.py' .github/workflows/acceptance.yml
grep -qF 'steps.sweep.outputs.rc' .github/workflows/acceptance.yml
# 3.5: the regressions, including 089's and 119's.
python3 scripts/test-verify-sweep.py
# 3.6: the trust boundary is untouched.
! grep -qE '^[[:space:]]*pull_request' .github/workflows/acceptance.yml
! grep -qE 'secrets\.' .github/workflows/acceptance.yml
target/release/spec-spine check --fail-on-unresolved --fail-on-warn
target/release/spec-spine lint --fail-on-warn
```
