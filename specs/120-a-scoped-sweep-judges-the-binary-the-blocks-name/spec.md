---
id: "120-a-scoped-sweep-judges-the-binary-the-blocks-name"
title: "A scoped sweep judges the binary the blocks name"
status: approved
kind: "tooling"
created: "2026-09-22"
summary: >
  The `Acceptance` workflow gives the sweep a binary built outside the sweep's
  worktree through `SPEC_SPINE_BIN`. Most acceptance blocks call
  `./target/release/spec-spine` relative to the worktree, and several do not
  build it first. In a whole-corpus run an earlier block's `cargo build
  --release` creates the file, so they pass by ordering. The push leg sweeps
  only the specs a merge touched, and when none of those builds, every block
  fails at exit 127, `not found`. That is how the ratification merge at
  `da47632b` and the merge at `ff79c68a` both went red with no acceptance
  failing. The workflow now lets the sweep build the binary from the revision
  inside its worktree, spec 089 3.8's default, so a block judges the binary it
  names.
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "089-nothing-reruns-a-merged-acceptance"
  - "099-a-merged-acceptance-is-asked-again"
extends:
  - { spec: "099-a-merged-acceptance-is-asked-again", unit: { kind: file, path: ".github/workflows/acceptance.yml" }, nature: corrective }
references:
  - { unit: { kind: file, path: "scripts/verify-sweep.sh" }, role: context }
  - { unit: { kind: file, path: "docs/release-candidate-0.22.0.md" }, role: context }
---

# 120: A scoped sweep judges the binary the blocks name

## 1. Purpose

`Acceptance` run `35770598972`, the push leg for the merge of #295 at
`da47632b`, swept the four specs that merge touched (100, 101, 104, 117) and
reported all four `failed`:

```
[verify] $ ./target/release/spec-spine check --fail-on-unresolved --fail-on-warn
sh: 1: ./target/release/spec-spine: not found
[verify] exit 127
```

The acceptance is not failing. The whole-corpus sweep at the same revision,
with a binary built from it inside the worktree, passes all four. The push leg
at `ff79c68a` (#290) failed 100 and 101 the same way and was not diagnosed.

The cause is the binary's location, not the blocks:

- the workflow builds `target/release/spec-spine` in the checkout and passes it
  to the sweep as `SPEC_SPINE_BIN`;
- the sweep runs each block in a detached worktree, whose `target/` is empty;
- a block calling `./target/release/spec-spine` without a `cargo build` of its
  own finds nothing, unless an earlier block in the same run built it.

A whole-corpus run always has such an earlier block. A scoped run has one only
by chance. So the push leg's verdict depended on which specs a merge happened
to touch.

## 2. Territory

`.github/workflows/acceptance.yml` (`extends` 099, corrective): the sweep step
no longer sets `SPEC_SPINE_BIN`. Nothing else changes.

## 3. Behavior

### 3.1 The sweep builds the binary it judges

The `Acceptance` workflow's sweep step MUST NOT pass `SPEC_SPINE_BIN`. The
sweep then builds `spec-spine` from the swept revision inside its worktree
(spec 089 3.8), which is the path the blocks invoke and the only binary whose
version provably matches the corpus under test. The workflow's own build stays,
because the scope computation reads the corpus with it.

### 3.2 Nothing else about the workflow moves

The triggers, the trusted ref, the absence of any `pull_request` leg, the
`contents: read` permission, the absence of secrets, and the rule that the
step's status is the sweep's (spec 099) are unchanged.

## 4. Out of scope

- **Editing the four blocks to build first.** They are approved specs, and a
  block that relies on the binary at the path the sweep documents is not wrong.
  Adding a build line to each would need four `amends_verification`
  amendments to work around a defect that is in the caller.
- **Changing the sweep's `SPEC_SPINE_BIN` semantics.** A maintainer who
  supplies a binary still gets exactly that binary (089 3.8). The fix is in the
  caller that should not have supplied one.

## 5. Resolved decisions

D-1 (2026-09-22, one more build per run). The sweep's in-worktree build adds a
release compile to each `Acceptance` run. That buys a verdict that no longer
depends on scope. The workflow's 120-minute limit has room for it.

## Verification

Fail-first: at `da47632b`, `acceptance.yml` sets `SPEC_SPINE_BIN` in the sweep
step (the first assertion is red), and the push leg's run `35770598972`
recorded exit 127 for all four specs. Reproduced locally at `da47632b`: the
same `--only 100,101,104,117` sweep with `SPEC_SPINE_BIN` set reports 0
passed, 4 failed, and without it 4 passed. The last assertion reruns that exact
scope the way the corrected workflow does. It unsets any inherited
`SPEC_SPINE_BIN` so the sweep builds from the revision, and requires success.

```verify:cli
# 3.1: the sweep step supplies no binary. Comments are excluded, because the
# workflow explains the absence in a comment that names the variable.
test -f .github/workflows/acceptance.yml && ! grep -v '^[[:space:]]*#' .github/workflows/acceptance.yml | grep -qF 'SPEC_SPINE_BIN'
# 3.1: the scope computation still has its own build.
grep -qF 'cargo build --release --locked -p spec-spine-cli' .github/workflows/acceptance.yml
# 3.2: the trust boundary is untouched.
! grep -qE '^[[:space:]]*(pull_request|pull_request_target|merge_group):' .github/workflows/acceptance.yml
grep -qF 'contents: read' .github/workflows/acceptance.yml
grep -qF -- '--trusted-ref origin/main' .github/workflows/acceptance.yml
# 3.1, behavior: the scope that failed at exit 127, swept as the corrected
# workflow sweeps it, passes. Needs origin/main at or after da47632b. Its
# output is kept: `verify` stops here if the sweep exits non-zero, and the
# sweep's own progress lines are the diagnosis.
env -u SPEC_SPINE_BIN scripts/verify-sweep.sh --rev origin/main --trusted-ref origin/main --only 100,101,104,117 --out "${TMPDIR:-/tmp}/ss120/run"
python3 -c "import json;d=json.load(open('${TMPDIR:-/tmp}/ss120/run/sweep.json'));assert d['binaryOrigin'].startswith('built from'), d['binaryOrigin'];o={s['id'][:3]:s['outcome'] for s in d['specs']};assert o=={'100':'passed','101':'passed','104':'passed','117':'passed'}, o"
```
