# 10: Ratification paths, the Statecraft-side contract (2026-09)

Spec 168 chooses ratify-then-build for this repository and makes ratification
an owner approval that a workflow applies, on two paths. Spec 092 puts managed
workflows in Statecraft's hands, so the workflow is Statecraft's to deliver.
This note is what it has to do, written so a Statecraft profile can implement
it without re-reading the spec's argument.

## The one mechanism

A GitHub deployment environment, `ratification`:

- required reviewers: the owner only; "prevent self-review" on;
- deployment branches: the default branch and same-repository pull request
  branches;
- secrets: the credential the apply job uses (a GitHub App installation
  token is the expected form). It is an environment secret, so a job that has
  not been approved cannot read it.

Approving a deployment to `ratification` is the ratification. Every other step
is mechanical and must be refusable.

## Path A: `workflow_dispatch`, ratify then build

```text
dispatch(spec)            owner approves             auto-merge
   │                           │                          │
   ▼                           ▼                          ▼
detect ──summary──► apply [env: ratification] ──► PR ratify/<id> ──► main
 (no secrets)          digest re-check,                   │
                       flip status, compile          optional start:
                                                      branch <id>,
                                                      implementation:
                                                      in-progress
```

- `detect`: resolve the id with `spec-spine registry show <id> --json`;
  refuse unless `status` is `draft`; write id, title, `implementation` and
  `contentHash` to the step summary and job outputs.
- `apply`: `environment: ratification`. Re-read `contentHash` on the default
  branch and refuse on any difference. Edit the one `status:` line, run
  `spec-spine compile`, and create the commit through the GitHub API (API
  commits are signed by GitHub; the default branch requires signatures). Open
  `ratify/<id>` as a pull request with auto-merge on.
- `start` (optional input, spec 168 D-4): after the ratify pull request merges,
  create branch `<id>` from the merged default branch with
  `implementation: in-progress`. Nothing is built.

## Path B: required check `ratification`, ratify at merge

Triggers: `pull_request` (opened, synchronize, reopened) and `merge_group`.

- `detect` computes the ratification set: specs that are `draft` at the head
  and whose `implementation` became `complete` in this pull request (base not
  `complete`, head `complete`). The base side is read with `git show
  <base>:specs/<id>/spec.md` and the registry at the head.
- Empty set: the check succeeds, no approval requested.
- Non-empty set on `pull_request`: `detect` writes the set, each
  `contentHash`, and the head SHA to the step summary and job outputs.
  `apply` reads the outputs (never the summary), runs in the
  `ratification` environment, refuses if the pull request head is no longer
  that SHA, flips `status` for exactly the listed specs, runs `compile`, and
  commits to the pull request branch through the API with the App token, so
  the push re-triggers the required workflows. The new run finds an empty set
  and passes.
- Non-empty set on `merge_group`: fail. Nothing in the queue is ratified.
- Fork pull request with a non-empty set: fail, naming Path A. The
  environment secret is not available to a fork, and `pull_request_target` is
  not used.

## What the owner configures (reserved gate changes)

1. Create the `ratification` environment as above and install the App.
2. Add `ratification` to the default branch's required status checks, next to
   `ci-gate`.

## Traps this contract avoids

- A push made with `GITHUB_TOKEN` starts no workflow run, so the flipped head
  would never report its required checks.
- A plain `git push` from a runner produces an unsigned commit, and the branch
  requires signatures.
- An approval that is not bound to a digest or a head SHA would ratify
  whatever the branch holds when the job resumes, not what the owner read.
- A queued commit cannot be amended, so Path B must finish before enqueueing.
