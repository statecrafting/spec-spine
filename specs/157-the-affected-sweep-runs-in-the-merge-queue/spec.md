---
id: "157-the-affected-sweep-runs-in-the-merge-queue"
title: "The affected-acceptance sweep runs in the merge queue"
status: approved
kind: "process"
created: "2026-09-27"
summary: >
  Spec 150 runs the affected-acceptance sweep in the session that drives a
  pull request, on its head, and records the counts in the pull request body
  (D-1). Any engine change selects the whole corpus, so each remediation round
  costs the session about 45 minutes and a fresh body record, and two pull
  requests swept at once share one machine. 150 D-1 named the alternative and
  left it to a later spec: run the sweep in CI on `merge_group`, which sweeps
  only a change a maintainer has queued. This spec takes it. The sweep becomes
  a required merge-queue check, sharded across a job matrix so a whole-corpus
  run is not one sequential job, and the session step and the body record
  retire. The job belongs to the Statecraft profile, so every governed
  repository gets it; this repository declares it until the profile carries it.
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "089-nothing-reruns-a-merged-acceptance"
  - "099-a-merged-acceptance-is-asked-again"
  - "150-a-change-runs-the-acceptance-it-can-break"
  - "154-the-pre-merge-sweep-is-a-skill-step"
  - "156-statecraft-profile-10-governs-this-repository"
amends:
  - "150-a-change-runs-the-acceptance-it-can-break"
extends:
  - { spec: "089-nothing-reruns-a-merged-acceptance", unit: "scripts/verify-sweep.sh", nature: additive }
  - { spec: "089-nothing-reruns-a-merged-acceptance", unit: "scripts/test-verify-sweep.py", nature: additive }
  - { spec: "064-compile-warnings-reach-the-gate", unit: "AGENTS.md", nature: additive }
  - { spec: "064-compile-warnings-reach-the-gate", unit: ".claude/skills/", nature: additive }
  - { spec: "156-statecraft-profile-10-governs-this-repository", unit: ".statecraft/environment.json", nature: additive }
  - { spec: "156-statecraft-profile-10-governs-this-repository", unit: ".statecraft/setup/github-actions-rust.json", nature: additive }
  - { spec: "156-statecraft-profile-10-governs-this-repository", unit: ".github/workflows/statecraft-ci.yml", nature: additive }
  # D-7: 099 3.1's guard reads the file it names, not a substring.
  - { spec: "094-one-gate-and-the-boundaries-it-holds", unit: "crates/spec-spine-core/tests/gate.rs", nature: corrective }
establishes:
  - { kind: file, path: ".github/workflows/affected-acceptance.yml" }
references:
  - { unit: { kind: file, path: ".github/workflows/acceptance.yml" }, role: context }
  - { unit: { kind: file, path: ".github/workflows/spec-spine-required.yml" }, role: context }
obligations:
  - { id: "R-1", kind: requirement, text: "A queued change is merged only after every affected-acceptance shard reports a clean release verdict: no implemented or unreadable spec has a failed, not-declared or not-run release outcome; pending specs still run and remain reported but do not block.", anchor: "3-1-the-sweep-is-a-merge-queue-check" }
  - { id: "R-2", kind: requirement, text: "verify-sweep.sh --shard i/n runs exactly the i-th of n disjoint parts of its selection, and the n parts cover the selection once.", anchor: "3-2-the-selection-is-sharded" }
  - { id: "R-3", kind: requirement, text: "No step requires a session to run the affected sweep locally or to record its counts in the pull request body.", anchor: "3-3-the-session-step-retires" }
  - { id: "R-4", kind: requirement, text: "The sweep never runs on a pull_request event.", anchor: "3-4-the-trust-boundary-holds" }
intent:
  goal: "Move the whole-corpus pre-merge acceptance run off the driving session and into the merge queue, without sweeping a change nobody has queued."
  non_goals:
    - "Narrowing the engine-source selection rule."
    - "Replacing the post-merge and scheduled sweeps of specs 099 and 121."
---

# 157: The affected-acceptance sweep runs in the merge queue

## 1. Purpose

### 1.1 What 150 costs

Spec 150 §1.2 measured that a reference-keyed selector selects close to the
whole corpus for any engine change: 68 plans run `check`, `lint` or
`index check` against this repository. So `--affected-by` falls back to every
spec when engine source changes, and D-1 put that run in the driving session
at about 45 minutes per engine change.

Observed 2026-09-27 on pull requests #402 (spec 155) and #403 (spec 156):

- #402's review round moved its head, and 150 §3.2 requires the record for the
  head that merges, so the 155-spec sweep started again from zero.
- Both sweeps ran at once on one machine, each at about 30 specs in 11 minutes.
- The AI reviewer skipped the round's head as oversized, so the only thing
  between the round and the default branch was the session's own sweep, run
  against a ref the session itself trusts (`--trusted-ref <head>`).

The cost scales with review rounds, not with the change: a pull request that
needs three rounds on engine code pays three whole-corpus sweeps before merge.

### 1.2 The alternative 150 left open

150 D-1: "CI on the merge queue (`merge_group`), which runs only on a change a
maintainer has queued and so stays closer to 099's rule, but costs the queue
about 45 minutes per engine change, sequentially; its runner cost could be
carried by self-hosted runners paid with cloud credits. That option stays open
for a later spec if session time becomes the constraint."

Session time is now the constraint. This repository already requires its
checks on `merge_group`: since spec 156, Statecraft Profile 11's
`statecraft-ci.yml` runs on it, and its `ci-gate` needs every job the project
declares in `ci.extra_required_jobs`. So the queue exists; what it lacks is the sweep. The sequential cost D-1 names is
answered by sharding (§3.2), which 150 §4 left out of scope, not rejected.

## 2. Territory

- `scripts/verify-sweep.sh`: `--shard <i>/<n>`.
- `scripts/test-verify-sweep.py`: its cases.
- `.github/workflows/affected-acceptance.yml` (planned): the merge-queue job,
  a reusable workflow, until the Statecraft profile carries it (§3.5).
- `.statecraft/environment.json`, `.statecraft/setup/github-actions-rust.json`
  and `.github/workflows/statecraft-ci.yml` (spec 156's): the job's
  `ci.extra_required_jobs` declaration and the files Statecraft re-renders
  from it. The rendered files are never hand-edited.
- `AGENTS.md` "Working the backlog" step 6 and the `/ship` and `/shepherd`
  skills: the step retires.

## 3. Behavior

### 3.1 The sweep is a merge-queue check

On every `merge_group` event, CI MUST run
`verify-sweep.sh --release --affected-by <merge_group.base_sha> --rev <merge_group.head_sha> --trusted-ref <merge_group.head_sha>`
over the affected selection, and the change MUST NOT merge unless every
shard's release verdict is clean: no implemented or unreadable spec has a
`failed`, `not-declared` or `not-run` release outcome. Pending specs still run
and their corpus outcomes remain reported, but their `pending` release outcome
does not block. An empty selection passes. The exemption of 150 §3.2 is kept:
a change touching only `docs/`, `website/`, or Markdown outside `specs/`
selects nothing to run and passes.

The job's result MUST reach the required aggregate (`ci-gate`) as a failure
when any shard fails, is cancelled, or does not report. A skipped job on
`merge_group` is a failure, never a pass.

Each shard MUST upload its `sweep.json`, `sweep.md` and `logs/` as an artifact,
so a refused queue entry is diagnosed from the run, not reproduced from
memory.

### 3.2 The selection is sharded

`verify-sweep.sh` MUST accept `--shard <i>/<n>` (1 ≤ i ≤ n) alongside
`--affected-by` and the whole-corpus mode. It computes the selection exactly as
it would without the flag, then runs only the specs whose position in that
selection, in corpus order, is congruent to `i - 1` modulo `n`. The report
MUST record the shard (`i`, `n`), the size of the whole selection and the
specs this shard ran, so the union of the shard reports can be checked
against one selection.

`--shard` with `--only`, a malformed value, `i` outside `1..n`, or `n < 1` is a
usage refusal (exit 3) before anything is created.

`scripts/test-verify-sweep.py` MUST carry
`test_shards_partition_the_selection` (every selected spec runs in exactly one
of `n` shards, for a selection larger and smaller than `n`) and
`test_shard_refuses_a_bad_value`.

### 3.3 The session step retires

AGENTS.md "Working the backlog" step 6 MUST NOT require the driving session to
run the affected sweep or to record its head SHA and counts in the pull request
body. It MUST say that the merge queue runs it (§3.1) and that a session may
still run it locally, `--shard` included, to find a failure before queuing.

`/ship` and `/shepherd` MUST NOT refuse to merge for want of a body record.
Every mention of `verify-sweep` in `.claude/skills/` stays the
`--affected-by` step (spec 154), now described as optional and local.

### 3.4 The trust boundary holds

The sweep executes the acceptance blocks of the change under test (spec 043),
so it runs only where a maintainer has chosen to run that change:

- It MUST NOT run on `pull_request` or `pull_request_target`. A stranger's pull
  request is still never swept (150 §3.2, 099).
- The job MUST run with `contents: read`, no other permission, no secret, and
  `persist-credentials: false` on checkout.
- `--trusted-ref` names the merge-group commit, which exists only because a
  maintainer queued the change (the override 089 §3.7 allows).

### 3.5 The job belongs to the Statecraft profile

The job is not specific to this repository: every repository governed by a
Statecraft profile with a merge queue has the same cost. It MUST be delivered
by the profile, rendered and identity-pinned like the rest of the managed
surface. Statecraft schedules it for Profile 13, with the per-event
declaration below; Profile 12 does not carry it.

Until a profile revision carries it, this repository declares it the way
Profile 11 declares `determinism` and `spec-spine-required`: an entry
`{ "job": "affected-acceptance", "workflow": ".github/workflows/affected-acceptance.yml" }`
in `ci.extra_required_jobs` of `.statecraft/environment.json`, with the
workflow triggered `on: workflow_call` only. Statecraft re-renders
`statecraft-ci.yml` and the policy `github-actions-rust.json` from the
declaration, so `ci-gate` needs the job and blocks on failed, cancelled and
skipped exactly as for its own jobs (Profile 11 revision 7). The repository
removes the declaration and the workflow when it adopts the revision that
carries the job.

Profiles 11 and 12 declare an extra required job for every event
(`pull_request`, `push`, `merge_group`); §3.4 needs one required on
`merge_group` only. Until Profile 13's per-event declaration
(`events: ["merge_group"]`, a skipped or missing result on a required event
failing), the interim job MUST read the caller's event
(`github.event_name`, which a called workflow inherits), sweep only when it
is `merge_group`, and on any other event pass in a step that runs nothing
from the checkout. The pass MUST be a step, not a job-level `if:`, because
`ci-gate` treats a skipped job as a failure. Statecraft confirmed (2026-09-27) that this
interim job is compatible with Profile 12.

### 3.6 The post-merge sweeps stay

The push-to-`main` and scheduled sweeps of specs 099 and 121 are unchanged.
This spec moves the pre-merge run; it replaces neither the backstop nor the
release verdict.

## 4. Out of scope

- Narrowing the engine-source rule (150 §1.2 measured it as buying little).
- Choosing the runner fleet. Hosted runners are the default; self-hosted
  runners paid with cloud credits (150 D-1) are an operator choice that
  changes no requirement here.
- The selection read other repositories call: spec 158 (D-2).

## 5. Resolved decisions

**D-1 (2026-09-27, owner): the pre-merge sweep moves to `merge_group`, band
wide.** The owner chose 150 D-1's alternative and asked for it across every
Statecraft-governed repository, not this one alone. Hence §3.5.

**D-2 (2026-09-27, owner): spec-spine owns selection.** Other repositories
do not carry `verify-sweep.sh`, so the profile job needs selection from
somewhere they all have. Asked which side owns it, the Statecraft owner
answered that spec-spine does: the profile job (Statecraft Profile 13) calls a
spec-spine read and shards the plan it returns, and `accept` evidence does not
select. The read is spec 158. This spec's job uses the script until 158
ships and the read after, with no change to what it requires.

**D-3 (2026-09-27, draft): shards are assigned by position, not by cost.** A
cost-balanced assignment needs recorded timings, which would make the
partition depend on a previous run. Position modulo `n` is a pure function of
the selection, so every shard computes the same partition independently.

**D-4 (2026-09-29, build): four shards, one list.** The matrix is `[1, 2, 3,
4]` on `merge_group` and `[1]` on every other event, where the one job passes
in a step without a checkout (§3.5). Four keeps a whole-corpus selection near
a quarter of the 45 minutes 150 D-1 measured, plus one build per shard; `n` is
the workflow's `SHARDS` and the list beside it, and changing it changes no
requirement here.

**D-5 (2026-09-29, build): a shard report is sweep report 1.3.0.** A `--shard`
run adds `shard` (`index`, `count`, `selectionSize`, and the `specs` that shard
ran) and is otherwise the 1.1.0 or 1.2.0 document it was. Under
`--affected-by`, `affectedBy.selected` is the whole selection's size, not the
shard's, so each shard names the same number; unsharded, it is unchanged.

**D-6 (2026-09-29, build): the render is Profile 11's own.** The declaration
was added to `.statecraft/environment.json` and `statecraft-ci.yml` and
`.statecraft/setup/github-actions-rust.json` were re-rendered by `init apply`
from statecraft-cli `7c59640`, the revision 11 producer, which first reproduced
every managed file of the unchanged tree byte for byte. The render added the
`affected-acceptance` call and its entry in `ci-gate`'s `needs`, and nothing
else. Its policy states the job `required` on all three events, which is why
§3.5's pass step exists.

**D-7 (2026-09-29, build): 099's guard reads the workflow it names.** Spec 099
§3.1 keeps `acceptance.yml`, the post-merge sweep, out of every `needs:` list.
`gate.rs` asserted that by refusing the substring `acceptance` anywhere in
`statecraft-ci.yml`, which the `affected-acceptance` job this spec requires
also contains. The assertion now refuses `/acceptance.yml` and a job named
`acceptance`, which is what 099 §3.1 requires; `acceptance.yml` is unchanged
and still in no `needs:` list. Before the change the test failed on this
tree, after it passes, and it still fails when `statecraft-ci.yml` calls
`./.github/workflows/acceptance.yml`.

**D-8 (2026-09-29, owner): the queue refuses on the release verdict.** 3.1
names `failed` and `not-run`, and the sweep reports two verdicts (spec 119).
Under the corpus verdict a pull request that files a draft is refused by the
draft's own block, which fails by design until the spec is built; before this
spec a session recorded that failure and a maintainer merged past it (#399), a
judgment a required check cannot make. The job passes `--release`: every
selected block still runs and is reported, a pending spec's outcome is not
counted, and an implemented spec that fails or does not run refuses the
change.

## Verification

```verify:cli
# 3.2: the shard cases exist and pass.
sh -c 'O="${TMPDIR:-/tmp}/ss157.out"; python3 scripts/test-verify-sweep.py > "$O" 2>&1; rc=$?; r=0; for t in test_shards_partition_the_selection test_shard_refuses_a_bad_value; do grep -q "^$t .* ok$" "$O" || { echo "missing or failed: $t"; r=1; }; done; rm -f "$O"; test $rc -eq 0 && test $r -eq 0'
sh -c 'scripts/verify-sweep.sh --help 2>&1 | grep -q -- "--shard"'
# 3.1, 3.4, 3.5: the job exists as a Profile 11 extra required job, sweeps on
# merge_group only, and is never triggered by a pull request of its own.
test -f .github/workflows/affected-acceptance.yml
grep -qE '^  workflow_call:' .github/workflows/affected-acceptance.yml
! grep -qE '^  (pull_request(_target)?|merge_group|push):' .github/workflows/affected-acceptance.yml
grep -qF "github.event_name == 'merge_group'" .github/workflows/affected-acceptance.yml
grep -qF '.github/workflows/affected-acceptance.yml' .statecraft/environment.json
grep -qF 'affected-acceptance' .github/workflows/statecraft-ci.yml
grep -qF -- '--release' .github/workflows/affected-acceptance.yml
grep -qF -- '--affected-by' .github/workflows/affected-acceptance.yml
grep -qF -- '--shard' .github/workflows/affected-acceptance.yml
grep -qF 'persist-credentials: false' .github/workflows/affected-acceptance.yml
! grep -qE 'secrets\.' .github/workflows/affected-acceptance.yml
# 3.3: step 6 no longer asks for a body record.
! grep -qF 'records the head SHA and the counts in the PR body' AGENTS.md
grep -qF 'merge queue' AGENTS.md
```
