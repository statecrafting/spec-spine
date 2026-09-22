---
id: "119-a-release-verdict-judges-what-was-built"
title: "A release verdict judges what was built"
status: approved
kind: "tooling"
created: "2026-09-22"
summary: >
  The whole-corpus sweep (spec 089) runs every declared acceptance block and
  exits 1 when any fails. This repository files a spec's acceptance before the
  spec is built and writes it to fail first, so any filed-but-unbuilt draft
  keeps the sweep red. "Sweep green" as a release pre-flight is then
  unreachable for as long as the backlog is honest. The sweep had no lifecycle
  predicate, and the only ways to get a green run were to build unrequested
  features or to hide the failures. This spec adds a release verdict next to
  the corpus verdict. It is judged over the specs whose implementation is
  built, with pending work run, listed with its real outcome and never counted
  as success or exemption, and `--release` puts it in the exit code. The
  corpus verdict and its exit code are unchanged. It also moves the default run
  directory out of the macOS temporary tree, whose daily purge is the inferred
  cause of a run in which 26 blocks failed that had not, and makes that
  directory new per run, so a rerun cannot clear the evidence of the run before
  it.
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "042-absent-implementation-defers-to-status"
  - "089-nothing-reruns-a-merged-acceptance"
  - "099-a-merged-acceptance-is-asked-again"
extends:
  # 3.1 - 3.5: the release verdict, the lifecycle read and the run directory.
  - { spec: "089-nothing-reruns-a-merged-acceptance", unit: { kind: file, path: "scripts/verify-sweep.sh" }, nature: corrective }
  # 3.6: the regressions, including the repaired ledger-boundary pin.
  - { spec: "089-nothing-reruns-a-merged-acceptance", unit: { kind: file, path: "scripts/test-verify-sweep.py" }, nature: corrective }
  # 3.7: the pre-flight item names the verdict it reads.
  - { spec: "006-distribution", unit: { kind: file, path: "docs/releasing.md" }, nature: additive }
# 3.3: 089 3.3's exit rule gains a second, opt-in basis, and 3.8's report gains
# fields. Both are changes to what 089 requires of the sweep, so they are
# declared here rather than made by editing 089. 089's acceptance is unchanged
# and still holds: nothing it asserts moved.
amends:
  - "089-nothing-reruns-a-merged-acceptance"
references:
  - { unit: { kind: file, path: "docs/release-candidate-0.22.0.md" }, role: context }
  - { unit: { kind: file, path: ".github/workflows/acceptance.yml" }, role: context }
---

# 119: A release verdict judges what was built

## 1. Purpose

### 1.1 "Sweep green" was unreachable, measured

The whole-corpus sweep at `835dd2e4`, and the nightly `Acceptance` run at
`3d4f3902` (run `35723398403`), both report three failures. One is spec 095's
over-strong acceptance, which spec 118 corrects. The other two are
`102-a-ready-spec-carries-its-status` and
`103-a-verifier-fixture-is-a-published-artifact`. Both are `draft` with
`implementation: pending`, filed deliberately ahead of their build, and their
blocks are written to fail against the tree they are filed on, which is this
repository's fail-first discipline working (`AGENTS.md`).

The sweep had no lifecycle predicate. A filed, unbuilt draft therefore always
counts as `failed`, and `docs/releasing.md`'s "verification sweep green"
cannot be satisfied while any draft is filed ahead of its build, which is this
repository's normal state. The three ways out that remain without this spec
are all wrong: build an unrequested feature to make a block pass; put an unbuilt
spec on the closed legacy ledger, which spec 089 3.4 forbids and which would
turn missing work into an exemption; or tag against a red checklist item.

### 1.2 A run that failed for a reason outside the corpus

The first whole-corpus run at `835dd2e4` reported 26 failures, where the
second found 3. `docs/release-candidate-0.22.0.md` 12 separates what is known
about why (D-6 records the correction of this paragraph):

- **Observed:** the default run directory was under `$TMPDIR`; macOS's
  `dirhelper` removes files there older than three days, daily at 03:35, and
  the unified log records it running at 03:35:04 local time, inside the first
  run's window. The console log puts the last passing spec before the first
  failure at about that minute.
- **Reproduced:** once `tree-sitter`'s build-script output
  `stdlib-symbols.txt` is deleted from a target directory, Cargo does not rerun
  the build script, and every later compilation fails with the first run's
  error text verbatim. That file is cloned from the crate archive with its
  2006-07-23 timestamps, so an age-based purge would treat it as old.
- **Inferred, not observed:** that the purge deleted that file from the
  sweep's target directory during the run. Nothing recorded the deletion
  itself, the per-spec logs that would show the order of events are gone, and
  the cleaner was not run deliberately to reproduce it.

The fixed default directory also let the rerun clear the first run's report, so
the only record of the failure is its console log. The remedy in 3.5 does not
depend on the inference being right: it takes the run out of the purged tree
and keeps every run's evidence.

### 1.3 The owner's ruling

Recorded in `docs/release-candidate-0.22.0.md` 10.2 before this was built: a
narrowly scoped correction distinguishing implemented-release acceptance from
pending-feature acceptance, with the constraints §3 turns into requirements.

## 2. Territory

- `scripts/verify-sweep.sh` (`extends` 089): the lifecycle read, the release
  outcome, the report fields, `--release`, and the default run directory.
- `scripts/test-verify-sweep.py` (`extends` 089): the regressions for all of
  the above, and the repair of its stale ledger-boundary pin (§3.6).
- `docs/releasing.md` (`extends` 006): the pre-flight item names the verdict.

## 3. Behavior

### 3.1 The lifecycle is a governed read

The sweep MUST read each spec's `status` and `implementation` through
`spec-spine registry list --json` at the tested revision, which is the CLI's
typed answer (`AGENTS.md` "Governed artifact reads"). A failure of that read
MUST refuse the sweep, exit 3, before any block runs.

Each spec is classed by what the corpus says about its implementation, using
spec 042's rule for an absent key:

| `implementation` | class |
|---|---|
| `complete`, `n-a` | implemented |
| `pending`, `in-progress`, `deferred` | pending |
| absent, on a `draft` | pending |
| absent, on `approved`, `superseded` or `retired` | implemented |
| anything else, or no answer for the spec | unknown |

`status` MUST NOT otherwise affect the class. An implemented draft's failure is
a release failure.

### 3.2 The release outcome

Every row MUST carry, besides its corpus outcome (089 3.3, unchanged), the
lifecycle as read and a release outcome:

- `not-run` when the spec's plan could not be read, whatever its class. A plan
  the verb cannot answer is missing evidence, not deferred work.
- `not-run` when the class is unknown.
- `pending` when the class is pending. The block still runs and its corpus
  outcome is still reported. `pending` is neither success nor failure of the
  release, and a pending spec on the legacy ledger is `pending`, never
  `exempt`.
- otherwise the corpus outcome.

### 3.3 Two verdicts, one exit code

The report MUST carry both verdicts. The **corpus** verdict is 089 3.3's: clean
only when every selected spec is `passed` or `exempt`. The **release** verdict
is clean only when no release outcome is `failed`, `not-declared` or `not-run`.

Without `--release`, the exit code MUST be the corpus verdict's, exactly as
089 3.3 states it. With `--release`, it MUST be the release verdict's: `0`
clean, `1` not. Refusals are `3` in both modes. The whole-corpus run is not
redefined. A maintainer asks for the release verdict by name, and the report
carries the corpus verdict alongside it.

### 3.4 The report is versioned, and the change is additive

`sweep.json`'s `schemaVersion` MUST become `1.1.0`. The additions are `mode`,
`verdicts`, `releaseCounts` (the five outcomes plus `pending`), and per row
`lifecycle` (`status` and `implementation`, null when absent, and `class`) and
`releaseOutcome`. Every 1.0.0 field MUST keep its meaning, and `counts` MUST
keep exactly its five keys. `sweep.md` MUST gain a release section listing each
release failure with its lifecycle, and each pending spec with the outcome its
block produced.

### 3.5 The default run directory

With no `--out`, the run directory MUST be a new directory per run under
`${XDG_CACHE_HOME:-$HOME/.cache}/spec-spine/sweeps/`, named from the short
revision, a UTC timestamp and the process id. With neither variable set, the
sweep MUST refuse rather than guess. An explicit `--out` keeps 089 3.6's rules
unchanged. On macOS, an `--out` under `$TMPDIR`, `/private/tmp` or
`/private/var/tmp` MUST produce a warning naming the purge, and MUST NOT be
refused.

### 3.6 The regressions run, and they cannot rot silently again

`scripts/test-verify-sweep.py` pinned `ledgerClosedAt` to `48`. Spec 095 moved
the boundary to `43` and nothing ran the file, so three of its four tests have
been red since. The pin MUST be read from the script under test. This spec's
acceptance MUST run the file, so a later drift turns a declared block red
instead of a file nobody runs.

The file MUST cover: pending, in-progress and deferred specs visible with their
real outcomes and outside the release verdict; a failing implemented draft
failing the release; spec 042's absent-key rule in both directions; a pending
ledger entry reported `pending` and not `exempt`; an unreadable plan on a
pending spec reported `not-run`; a timed-out implemented block reported
`not-run`; a spec the lifecycle read does not answer for reported `not-run`;
the corpus exit code unchanged without `--release`; the default run directory
new per run and outside the temporary tree; and the macOS warning.

### 3.7 The runbook reads the release verdict

`docs/releasing.md`'s pre-flight item MUST name `--release`, say which specs
the verdict is judged over and that pending ones still run, and ask for the
corpus counts to be recorded next to it.

### 3.8 What does not change

The trust boundary (089 3.7, 099 3.1) is untouched: no new trigger, no
`pull_request` leg, no credentials. The closed ledger is not expanded.
`verify` and every engine verb are unchanged.

## 4. Out of scope

- **The `Acceptance` workflow.** Its nightly leg keeps the corpus verdict and
  stays red while a draft is filed ahead of its build. Whether the nightly
  should exit on the release verdict is a separate decision about what that
  signal is for. Both verdicts are now in its uploaded report either way.
- **A build-directory remedy inside Cargo or tree-sitter.** The inferred
  failure is the operating system deleting files the build still needed (1.2).
  Moving the run directory out of the purged tree removes that exposure without
  depending on a dependency's copy semantics.
- **Retiring or reclassifying any spec.** The sweep reports the lifecycle the
  corpus declares. It does not decide it.

## 5. Resolved decisions

D-1 (2026-09-22, pending blocks still run). Skipping them would make the
release run faster and would also hide whether a pending block is still
fail-first or has started passing on its own, which is a finding. The owner
asked that the raw run keep reporting what it observed. So the block runs, its
corpus outcome is kept, and only the release verdict sets it aside.

D-2 (2026-09-22, `deferred` is pending). `deferred` records a decision not to
schedule the work (spec 035), not that it is done. Counting its block toward the
release would make an unbuilt feature's green block look like acceptance of
something that was never built.

D-3 (2026-09-22, `superseded` and `retired` are judged when implemented). No
spec in this corpus carries either status. Setting them aside would be a new
exemption with no case in front of it. A later spec that retires one can decide
what its acceptance means.

D-4 (2026-09-22, `--release` is opt-in). Making the release verdict the default
exit code would silently change what `Acceptance`'s nightly leg and every
maintainer's existing command mean. The owner asked for an explicit selection.

D-5 (2026-09-22, the cache directory, not a new temporary directory). Any path
under the operating system's temporary tree is subject to the purge that caused
§1.2. `XDG_CACHE_HOME`, falling back to `~/.cache`, is where per-user tool state
lives on both platforms, and it is not swept by age.

D-6 (2026-09-22, the account of the first run is corrected, not the remedy).
This spec's 1.2 first said the candidate record "establishes the cause" while
the same record called the deletion inferred and the per-spec logs lost. That
overstated the evidence. 1.2 now separates the observed cleaner timing, the
reproduced Cargo behavior, and the inferred and unobserved deletion. No
requirement moved: 3.5's run directory was always justified by the exposure
and by the lost report, not by proof of the deletion. No investigation was
rerun to obtain a stronger account.

## Verification

Written to fail against the tree it was built on. With this change's
`scripts/test-verify-sweep.py` run against `origin/main`'s `verify-sweep.sh`
(`--sweep-script`), all eight spec-119 tests fail and the four spec-089 tests
pass. With the unknown-lifecycle guard removed from the new script, exactly the
test for it fails. At the parent revision the file itself was already red: three
of its four tests failed on the stale `48`.

```verify:cli
cargo build --release --locked
bash -n scripts/verify-sweep.sh
# 3.1 - 3.6: the regressions, in disposable repositories, including the four of
# spec 089's review that were red on the stale ledger boundary.
python3 scripts/test-verify-sweep.py
# 3.4: the report version moved with the additive fields.
grep -qF '"schemaVersion": "1.1.0"' scripts/verify-sweep.sh
# 3.1: the lifecycle comes from the governed read.
grep -qF 'registry list --json' scripts/verify-sweep.sh
# 3.5: the old fixed default under the temporary tree is gone.
! grep -qF 'out="${TMPDIR:-/tmp}/spec-spine-sweep-' scripts/verify-sweep.sh
# 3.6: the pin is read, not written down.
! grep -qE 'ledgerClosedAt"\], (43|48)\)' scripts/test-verify-sweep.py
# 3.7: the runbook names the release verdict.
grep -qF 'verify-sweep.sh --rev origin/main --release' docs/releasing.md
# 3.8: the trust boundary is untouched: the workflow still has no pull request
# leg, and the ledger's boundary did not move.
! grep -qE '^[[:space:]]*pull_request' .github/workflows/acceptance.yml
grep -qx 'readonly LEDGER_CLOSED_AT=43' scripts/verify-sweep.sh
# The governed loop.
target/release/spec-spine check --fail-on-unresolved --fail-on-warn
target/release/spec-spine lint --fail-on-warn
```
