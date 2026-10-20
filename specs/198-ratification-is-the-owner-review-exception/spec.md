---
id: "198-ratification-is-the-owner-review-exception"
title: "Ratification is a ratify pull request the owner approves through the review exception"
status: draft
kind: "process"
created: "2026-10-19"
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "156-statecraft-profile-10-governs-this-repository"
supersedes:
  - "168-ratification-has-two-paths"
summary: >
  Supersedes spec 168. Ratification stays a separate pull request that flips
  `status: draft` to `approved`, and the owner's approval of that pull request
  is the review-exception environment approval that Statecraft's managed CI
  already requires for every ratification (profile revision 12). No
  `ratification` environment and no dispatch workflow are built.
references:
  - { unit: { kind: file, path: "specs/168-ratification-has-two-paths/spec.md" }, role: "the superseded design: Path A dispatch and Path B ratify-at-merge" }
  - { unit: { kind: file, path: ".github/workflows/statecraft-ci.yml" }, role: "the review-exception job and its statecraft-review-exception environment" }
  - { unit: { kind: file, path: "scripts/statecraft/ci-gate.sh" }, role: "detects a ratification and blocks it without the owner exception" }
  - { unit: { kind: file, path: "AGENTS.md" }, role: "Working the backlog, step 6: ship, then ratify" }
---

# 198: Ratification is a ratify pull request the owner approves through the review exception

## 1. Purpose

Spec 168 (draft, never built) chose ratify-then-build with two automated
paths, both resting on a deployment environment named `ratification` that a
Statecraft-delivered workflow would use. The owner asked Statecraft for that
workflow on 2026-10-10 (statecraft-cli#236); it has not been delivered.

In the meantime Statecraft's managed CI made ratification an owner act by
other means. Profile revision 12 runs the `review-exception` job, on the
protected `statecraft-review-exception` environment, for every pull request
that moves a spec to `approved`, and `ci-gate` refuses such a pull request
unless that job succeeded. The owner's environment approval is therefore
already the ratification, and it already cannot be granted by an agent.

On 2026-10-19 the owner ruled to adopt that mechanism and supersede 168. This
spec records the procedure as it runs.

## 2. Territory

None. The workflow and the gate script are Statecraft's managed files; this
spec cites them and claims nothing.

## 3. Behavior

### 3.1 The procedure

A spec is filed `draft`, built, and merged with `implementation: complete`,
as `AGENTS.md` "Working the backlog" describes. A separate ratify pull request
then flips `status: draft` to `approved` and nothing else.

### 3.2 The approval

A pull request that moves any spec to `approved` is a ratification (the
managed gate compares each changed `spec.md`'s status at base and head). Its
`review-exception` job waits on the `statecraft-review-exception`
environment, whose required reviewer is the owner, and `ci-gate` refuses the
pull request unless that job succeeded on the run. An agent MUST NOT approve
the environment, and MUST treat a waiting `review-exception` as an owner
checkpoint, never as a failure to work around.

### 3.3 What 168 asked for and this does not build

- No `ratification` environment, no dispatch workflow (168's Path A).
- No ratify-at-merge check (168's Path B): the ratify pull request stays
  separate from the build.
- `AGENTS.md` and the skills keep their current wording; 168's 3.4 and 3.5
  rewrites are not made.

## 4. Acceptance criteria

1. The managed CI runs `review-exception` on the protected environment for a
   pull request whose governance job reports a ratification.
2. The managed gate refuses a ratifying pull request whose owner exception
   was not approved.
3. `AGENTS.md` still describes the separate ratify pull request.

## 5. Out of scope

- Changing Statecraft's managed workflow or gate script.
- Ratify-then-build as this repository's default procedure.
- Shortening the lag between a build's merge and its ratification.

## 6. Resolved decisions

**D-1 (2026-10-19, owner): supersede 168 and close statecraft-cli#236.** The
review exception already makes ratification an owner-only approval; a second
environment would duplicate it.

## Verification

```verify:cli
grep -qF "needs.governance.outputs.ratification == 'true'" .github/workflows/statecraft-ci.yml
grep -qE '^[[:space:]]*environment:[[:space:]]*statecraft-review-exception[[:space:]]*$' .github/workflows/statecraft-ci.yml
grep -qF 'this candidate ratifies a spec and the owner exception' scripts/statecraft/ci-gate.sh
grep -qF 'second PR flips `status: draft` to `approved`' AGENTS.md
```
