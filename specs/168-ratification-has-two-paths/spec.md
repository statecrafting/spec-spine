---
id: "168-ratification-has-two-paths"
title: "Ratification has two paths, both an owner approval"
status: draft
kind: "process"
created: "2026-09-27"
summary: >
  This repository builds a spec while it is still a draft and ratifies it in a
  second, hand-authored pull request. The owner has chosen ratify-then-build
  instead, with ratification as a triggerable action gated on an owner-only
  deployment environment, and a second path that ratifies at merge when a pull
  request lands a spec at draft and complete. Both paths flip only `status`,
  bind the approval to the exact bytes shown, and leave ratification owner-only.
implementation: pending
owner: "The spec-spine Authors"
risk: high
depends_on:
  - "093-the-harness-this-repository-runs"
  - "156-statecraft-profile-10-governs-this-repository"
extends:
  # "Working the backlog" steps 1, 2 and 6 describe build-as-draft; they are
  # rewritten to the two paths below.
  - { spec: "093-the-harness-this-repository-runs", unit: { kind: file, path: "AGENTS.md" }, nature: corrective }
  # /next, /build, /ship and /shepherd name the path a spec is on.
  - { spec: "093-the-harness-this-repository-runs", unit: { kind: directory, path: ".claude/skills/" }, nature: corrective }
references:
  - { unit: { kind: file, path: "standards/spec/contract.md" }, role: "the lifecycle table: approved plus pending is a work order" }
  - { unit: { kind: file, path: "specs/041-in-progress-is-in-flight/spec.md" }, role: "section 1.2 records the build-as-draft practice this spec retires" }
  - { unit: { kind: file, path: "docs/design/10-ratification-paths-2026-09.md" }, role: "the Statecraft-side workflow contract" }
intent:
  goal: "Make ratification an owner approval that a workflow records and applies, on one of two declared paths, instead of a hand-authored second pull request."
  non_goals:
    - "letting any agent, bot or token ratify without the owner's approval"
    - "a spec-spine verb that edits frontmatter"
    - "changing what the lifecycle table in the contract schedules"
---

# 168: Ratification has two paths, both an owner approval

## 1. Purpose

`AGENTS.md` "Working the backlog" says this repository files a spec as
`draft`, builds it, merges it, and then ratifies it in a separate pull request.
Two things are wrong with that in practice.

- **The coordination policy disagrees.** The contract campaign of 2026-09-26
  was run under a policy that ratifies a batch before implementation, and
  `docs/backlog.md` recorded the conflict as an owner decision that blocked
  implementation, merge and lifecycle changes until one procedure was chosen.
- **The second pull request is a manual step that lags.** Specs merge as
  `draft` with `implementation: complete` and stay that way until someone
  authors the flip. The corpus then carries implemented specs that nobody has
  ratified, and a reader cannot tell a spec awaiting review from one that
  was simply never followed up.

On 2026-09-27 the owner chose ratify-then-build as the procedure, with two
automated ways to perform the ratification:

- **Path A, ratify then build.** Ratification is a triggerable action. The
  owner approves it, a workflow flips `status: draft` to `approved`, and the
  spec becomes a work order (`approved` plus `pending`, the contract's
  lifecycle table) that `/next` offers and `/build` builds.
- **Path B, ratify at merge.** When a pull request brings a spec to
  `status: draft` with `implementation: complete`, a required check holds the
  merge on the same owner approval, and the approval ratifies the spec in the
  merge itself. The separate ratify pull request goes away.

Both paths use one mechanism: a GitHub deployment environment named
`ratification` whose only required reviewer is the owner. The approval of a
deployment to it is the ratification; the workflow only records and applies
it. Ratification stays on the owner's reserved list in `AGENTS.md`.

## 2. Territory

This spec owns no new file in this repository. It extends two units spec 093
establishes:

- `AGENTS.md`, whose "Working the backlog" steps 1, 2 and 6 are rewritten to
  describe the two paths (section 3.4);
- `.claude/skills/`, where `/next`, `/build`, `/ship` and `/shepherd` name
  the path a spec is on (section 3.5).

The workflow that performs both paths is delivered by Statecraft (spec 092:
the managed workflows are Statecraft's), and its contract is written down in
`docs/design/10-ratification-paths-2026-09.md`. When Statecraft delivers it,
the build claims the delivered file here (section 5, D-1).

## 3. Behavior

### 3.1 One approval authority

- The `ratification` environment MUST have the owner as its only required
  reviewer, with self-review prevention on, and MUST hold the credential the
  apply step uses as an environment secret, so no job that has not been
  approved can read it.
- No agent, bot, token or workflow may ratify without that approval. An
  agent MUST NOT approve a deployment to `ratification`, MUST NOT edit
  `status: draft` to `approved` by hand, and MUST NOT add itself or a bot as a
  reviewer of the environment.
- The owner MAY still ratify by hand-authoring the flip, as today. That is the
  owner's act on either path, not a third path an agent may take.

### 3.2 Path A: ratify then build

Triggered by `workflow_dispatch` with one input, a spec id.

1. An unprivileged job MUST resolve the id, refuse a spec that is not
   `status: draft`, and write to the run summary the spec's full id, title,
   `implementation` value and its registry `contentHash`
   (`spec-spine registry show <id> --json`). That summary is what the owner
   approves.
2. The apply job MUST run in the `ratification` environment, and MUST refuse
   if the spec's `contentHash` on the default branch no longer equals the one
   the first job printed. An approval ratifies the bytes that were shown.
3. The apply job MUST change exactly one line of `specs/<id>/spec.md`,
   `status: draft` to `status: approved`, regenerate the derived registry with
   `spec-spine compile`, and propose both on a branch named `ratify/<id>`
   through a pull request that merges itself when its checks pass. It MUST
   NOT push to the default branch.
4. Path A MUST accept a draft at any `implementation` value, so it also
   ratifies a spec that already merged as `draft` plus `complete` under the
   old procedure.
5. The workflow MAY take a second input, `start`, that, after the ratify pull
   request has merged, creates the spec's feature branch from the merged
   default branch with `implementation: in-progress`. It MUST NOT build
   anything; `/build` does that on the branch.

### 3.3 Path B: ratify at merge

A required status check named `ratification`, separate from `ci-gate`, runs
on `pull_request` and `merge_group`.

1. **What it looks at.** The specs whose `implementation` is `complete` at
   the head and was not `complete` at the base, and whose `status` at the head
   is `draft`. Call this the ratification set.
2. **Empty set.** The check MUST pass without an approval. This is every pull
   request that does not complete a draft, including a Path A ratify pull
   request and a spec filed as a draft.
3. **Non-empty set.** An unprivileged job MUST list the set in the run
   summary with each spec's `contentHash` and the head SHA. The apply job
   MUST run in the `ratification` environment, MUST refuse when the pull
   request's head has moved since the listing, and on approval MUST flip
   `status` for exactly the listed specs, regenerate the registry, and commit
   the result to the pull request's branch. The commit MUST be signed (the
   branch requires signatures) and MUST be made with a credential whose push
   starts the required workflows again; a `GITHUB_TOKEN` push starts none.
4. **Re-run.** On the new head the set is empty, so the check passes and the
   merge proceeds with the specs already `approved`.
5. **Merge queue.** On `merge_group` the check MUST NOT request an approval.
   It passes when the set is empty and fails otherwise: a pull request enters
   the queue after it is ratified, never before.
6. **Forks.** A pull request from a fork cannot receive the environment's
   credential. The check MUST fail with a message that names Path A for a
   non-empty set, and a maintainer carries the change onto a same-repository
   branch.

### 3.4 `AGENTS.md` names the two paths

"Working the backlog" MUST say:

- step 1: a filed spec is born `draft` and is not built until it is ratified
  on Path A, unless the owner names a draft for Path B;
- step 2: `/build` builds an `approved` spec, and builds a `draft` only when
  the owner named that id for Path B;
- step 6: after `/verify`, a Path A spec merges with its status unchanged, and
  a Path B spec merges only once the `ratification` check has ratified it.
  The paragraph describing a separate ratify pull request after merge MUST be
  removed, along with the sentence that this repository builds a spec as a
  draft and ratifies it in a separate pull request.

The phrases `Ratify, then build` and `Ratify at merge` MUST name the two paths
in that section, so the skills and this spec's acceptance can cite them.

### 3.5 The skills agree

- `/next` MUST report an unratified draft as awaiting Path A (it already
  lists it as awaiting approval and never offers it, spec 093 section 4.7).
- `/build` MUST keep refusing a draft unless the owner named the id, and in
  that case MUST say the pull request will need Path B.
- `/ship` and `/shepherd` MUST treat a pending `ratification` check as an owner
  checkpoint: report it and stop, never retry it, approve it or work around
  it.

## 4. Out of scope

- The workflow's YAML and the Statecraft profile that delivers it. Its
  contract is `docs/design/10-ratification-paths-2026-09.md`.
- Creating the `ratification` environment and adding the `ratification`
  check to branch protection. Both are gate changes reserved to the owner.
- Ratifying specs that merged as `draft` plus `complete` before this spec is
  built. Path A covers them one at a time once it exists.
- The campaign's continuation identity (`docs/backlog.md`), which is a
  coordination task, not repository behavior.
- Any spec-spine verb. The workflow reads through `registry show --json` and
  writes through `compile`; the frontmatter flip is a one-line text edit.

## 5. Resolved decisions

- **D-1 (2026-09-27, owner to confirm) The workflow's file is claimed at
  build time.** Statecraft chooses its name and path. The build adds it to
  this spec's `establishes`, which is claiming a file the build brought in.
- **D-2 (2026-09-27, owner to confirm) The set is "became complete in this
  pull request".** Using "any draft spec whose file the pull request touches"
  would make a typo fix in an old unratified spec demand its ratification.
  Using "any draft plus complete spec at the head" would make every pull
  request ratify the backlog of specs merged under the old procedure.
- **D-3 (2026-09-27, owner to confirm) `ratification` is a separate required
  check, not a job inside `ci-gate`.** A pending approval would otherwise hold
  the aggregate status that carries the gate's evidence, and `ci-gate` is
  managed by Statecraft's profile identity.
- **D-4 (2026-09-27, owner to confirm) `start` is optional and comes last.**
  Path A is complete without it. It exists because the owner asked for a flow
  that can move a ratified spec to `in-progress`.

## Verification

```verify:cli
# 3.4: AGENTS.md no longer describes build-as-draft with a separate ratify PR.
sh -c '! grep -qF "builds it, then ratifies it in a" AGENTS.md'
sh -c '! grep -qF "second PR flips" AGENTS.md'
# 3.4: both paths are named where the skills can cite them.
grep -qF 'Ratify, then build' AGENTS.md
grep -qF 'Ratify at merge' AGENTS.md
# 3.5: the delivery skills treat a pending ratification as an owner checkpoint.
grep -qF 'Ratify at merge' .claude/skills/ship/SKILL.md
grep -qF 'Ratify at merge' .claude/skills/shepherd/SKILL.md
grep -qF 'Ratify, then build' .claude/skills/next/SKILL.md
grep -qF 'Ratify at merge' .claude/skills/build/SKILL.md
# 3.1 to 3.3: a delivered workflow runs a job in the ratification environment.
grep -rqE '^[[:space:]]*environment:[[:space:]]*ratification[[:space:]]*$' .github/workflows/
# The skills still carry the gate floor AGENTS.md lists.
cargo test -p spec-spine-core --locked --test harness_skills
```
