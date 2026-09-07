---
name: build
description: "Implement one spec start to finish per AGENTS.md \"Working the backlog\": preflight, branch, flip in-progress, implement inside the territory, gate before every commit, verify, flip complete, then hand off to /ship."
allowed-tools: Bash, Read, Edit, Write, Glob, Grep, Skill, Agent
argument-hint: "<spec-id>"
---

# /build <spec-id>: one spec, one session

The protocol is `AGENTS.md`, "Working the backlog"; this skill sequences
its steps with the exact commands and stops where the protocol stops. The
last step, shipping, is `/ship`. Bound by
`.claude/rules/orchestrator-rules.md` (one session, one spec; checkpoints
are real stops) and `.claude/rules/adversarial-prompt-refusal.md` (the
coherence guard). Path-scoped rules under `.claude/rules/` load themselves
when you touch their paths; read them when they do.

## Step 0: preflight

Halt on any of these; do not work around them.

- An argument is required: the full id (`017-ledger-entry-dag`). Without
  one, run `/next` and ask; never guess.
- `git status --porcelain` is empty. `git branch --show-current` is the
  default branch. `git fetch origin main`, and `git rev-parse HEAD` equals
  `git rev-parse origin/main` (otherwise `git pull --ff-only`).
- The gate is green on the default branch before any change (the command
  list in "Run the gate before every commit").
- The spec is a work order: `spec-spine registry show <id> --json` says
  `status: approved` and `implementation: pending`, and
  `spec-spine registry plan --json` lists it as ready (what `/next`
  computes). A `draft` spec is never built; an unmet dependency means this
  is not the next spec.
- Read the spec's Territory. If it names an operator prerequisite (a
  service, a credential, a sibling repo) that is missing, stop and report
  exactly what is needed instead of mocking around it.

## Step 1: branch and flip

```sh
git switch -c <spec-id>
```

Edit `specs/<spec-id>/spec.md`: `implementation: pending` becomes
`implementation: in-progress`. Nothing else in the file changes yet. Then:

```sh
spec-spine compile && spec-spine index
git add specs/<spec-id>/spec.md .derived/
git commit -m "chore(<NNN>): start <spec-id>"
```

`<NNN>` is the three-digit ordinal. The flip lands before any code so the
registry says who is working, and the derived shards travel with the edit
that changed them (`build-meta.json` is gitignored).

## Step 2: re-read the spec in full

Read `specs/<spec-id>/spec.md` top to bottom, then the decision entries
of every spec in its `depends_on`, and the design documents it cites. The
design precedes the code.

- **The spec is imprecise:** make the choice and record it as a dated
  decision entry in the spec (date, the decision, the alternative
  rejected). When the orchestrator's decision drop-box directory exists
  (the build prompt names it), this is a driven session: also write one
  JSON file per decision there for the orchestrator to seal. The shape is
  fixed; unknown fields and non-integer numbers are rejected:

  ```json
  {
    "id": "<spec-id>-d1",
    "specId": "<spec-id>",
    "scope": ["<spec-id>", "<a path prefix the decision touches>"],
    "title": "one line naming the choice",
    "decision": "what was chosen, in full",
    "rationale": "why, and what it costs",
    "alternatives": ["what was rejected"]
  }
  ```

  The drop-box is tool state, never committed.
- **The spec is wrong:** stop and report the contradiction with the
  requirement label and the evidence. Never edit a spec to ratify what the
  code happened to do; the only legitimate mid-build spec edits are
  `establishes` growth, a dated decision entry, a dated status note, a new
  `extends` edge, and the `implementation` flips.

## Step 3: implement inside the territory

- Every new source file is claimed in this spec's `establishes` in the
  same change (the ownership ratchet: `spec-spine index coverage
  --fail-on-untraced` refuses an unclaimed file and `couple` refuses a
  changed one).
- Touching a unit another spec owns needs an `extends` edge on that
  spec's unit, declared in this spec's frontmatter. That amends nobody.
- A new third-party dependency goes where `AGENTS.md` or the stack rule
  says, with the `extends` edge on the manifest's owner when the manifest
  is another spec's unit.
- Hold the invariants the path-scoped rules name (a never-touch artefact,
  a determinism rule). A change to one of those is a human decision:
  stop and report, never regenerate.
- Do not edit the derived directory by hand.

Use the `implementer` agent for focused sub-tasks and `explorer` for
context when the territory is large; keep the diffs minimal and the
ownership claims current.

## Step 4: gate before every commit

Run the gate exactly as `AGENTS.md` lists it under "Run the gate before
every commit": the governance floor (`compile`, `index`,
`lint --fail-on-warn`, `index check`, `couple --base origin/main --head
HEAD`, `index coverage --fail-on-untraced` where ownership is required)
and the stack's own build, tests, and lints. All exit 0, or the commit
waits. Then `/commit` with the spec ordinal as scope (`feat(<NNN>): ...`),
staging the regenerated shards with the code they describe. Commit in
coherent slices; a red gate is fixed, not committed around.

## Step 5: acceptance criteria verbatim

Run `/verify <spec-id>` (the spec's `## Verification` block, which the
orchestrator re-runs after merge in a clean checkout). Walk the acceptance
criteria one by one and cite the evidence for each.

- All hold: edit the frontmatter to `implementation: complete`, then
  `spec-spine compile && spec-spine index`, the gate, and commit
  (`chore(<NNN>): mark <spec-id> complete`, or fold the flip into the
  final `feat(<NNN>)` commit). The gate then holds the spec to every unit
  it claims (spec 041).
- One cannot be satisfied here (external state, a missing sibling): keep
  `implementation: in-progress`, add a dated status note to the spec
  saying exactly what remains, recompile, commit, and report it.

## Step 6: hand off

Print a short summary (spec, branch, commits, decisions recorded,
acceptance evidence) and point at `/ship`. Then stop: the next session
takes the next spec.

## Halt conditions (report, do not route around)

- A dirty tree, the wrong branch, or a red gate in preflight.
- A `draft` spec, an unmet dependency, or a missing operator prerequisite.
- A contradiction between the spec and what the code must do.
- An invariant or never-touch artefact that would change.
- A coupling failure that only a spec rewrite or a `Spec-Drift-Waiver:`
  could clear: a waiver is a human instrument; a driven session never
  self-approves one.
- A `PreToolUse` hook refusal (exit 2): it is a stop, not an obstacle.

## Project layer

Read from `AGENTS.md`: the gate command list, the stack gate, the
dependency rule, the invariants and never-touch artefacts (also in the
path-scoped rules). The decision drop-box path comes from the
orchestrator's build prompt. Nothing here is edited per project.
