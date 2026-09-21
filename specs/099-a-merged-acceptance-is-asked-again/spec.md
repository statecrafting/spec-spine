---
id: "099-a-merged-acceptance-is-asked-again"
title: "A merged acceptance is asked again"
status: approved
kind: "governance"
created: "2026-09-21"
summary: >
  `verify` is outside the gate chain because it executes what a spec declares
  and a pull request is a stranger's code. Spec 089 built the sweep that re-asks
  the corpus's acceptance and made it maintainer-invoked, which leaves the
  window it was built to close open between runs: spec 092's block was red on
  `main` from #277 until #278 with every required check green, and no light
  turned. This spec runs the sweep where the trust boundary already allows it,
  on the default branch after a merge and nightly, scoped after a merge to the
  specs that own a changed path. It amends spec 089's "never CI" to "never on
  untrusted input", which is the rule that clause was written to hold.
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "043-verify-declared-acceptance"
  - "089-nothing-reruns-a-merged-acceptance"
  - "094-one-gate-and-the-boundaries-it-holds"
establishes:
  - { kind: file, path: ".github/workflows/acceptance.yml" }
  - { kind: file, path: "scripts/acceptance-scope.sh" }
extends:
  - { spec: "089-nothing-reruns-a-merged-acceptance", unit: { kind: file, path: "scripts/verify-sweep.sh" }, nature: additive }
  - { spec: "094-one-gate-and-the-boundaries-it-holds", unit: { kind: file, path: "crates/spec-spine-core/tests/gate.rs" }, nature: additive }
amends:
  - "089-nothing-reruns-a-merged-acceptance"
references:
  - { unit: { kind: file, path: ".github/workflows/ci.yml" }, role: context }
---

# 099: A merged acceptance is asked again

## 1. Purpose

### 1.1 The window spec 089 left open

Spec 089 §1.1 states the gap in the corpus's own record: a `## Verification`
block is run once, by the session that writes it, and then never again, so a
block a later spec legitimately invalidates goes red silently and stays red. It
built `scripts/verify-sweep.sh` to re-ask the question, and §3.1 made the sweep
a thing "a maintainer runs, by hand. Never CI".

That closes the gap only as often as a maintainer runs it, and §3.9 names two
triggers: before a release, and after merging a spec carrying `amends` or
`amends_verification`. Neither fires for an ordinary merge, and an ordinary
merge is what breaks a block.

**Measured, on this repository, twice.**

Spec 092's declared acceptance was red on `main` from the merge of #277 until
#278 repaired it. Every required check was green for that whole window, because
the gate chain does not contain `verify` and nothing else asks. The break was
found by a human reading a diff.

Spec **089's own** acceptance was red on `main` for longer, and nobody had read
that diff. Its block runs `verify-sweep.sh --rev origin/main --only 012` and
then asserts the answer is `011-index-hash-slices`; the collapse renumbered the
full id and left the bare ordinal, so the block asked for one spec and asserted
another from spec 095's merge until spec 098 §3.7 repaired it on 2026-09-21. It
was found by running the sweep by hand while writing this spec. The instrument
spec 089 built works; what it lacked was anything that runs it.

A full sweep of `main` at `1c133c20` on 2026-09-21, before that repair, reached
every spec: **54 passed, 43 exempt, 1 failed**, and the one failure is spec 089's
own block. Two caveats belong with that figure and neither changes it. The run's
report writer died before it could print a summary, for the reason D-7 records,
so these counts are read off its per-spec lines. And spec 089's fixture path
collided with a hand run of the same block in the same window, so the sweep's
answer for that one spec is not evidence on its own; it is evidence because the
same failure was reproduced by hand at command 47, and passes at 66 commands
once spec 098 §3.7's repair is applied.

The corpus spent **17 minutes** inside `verify` for those 98 specs, plus a
release build. That is the cost of the nightly leg, measured rather than
guessed.

### 1.2 The boundary the "never CI" clause was protecting

Spec 089 §3.7 and spec 083 §4 draw the boundary that matters: `verify` executes
arbitrary declared shell, so it must never be pointed at code nobody has
accepted. Spec 089 made that mechanical rather than documentary, and the sweep
refuses (exit 3) a revision that is not an ancestor of a trusted ref.

"Never CI" is a stronger statement than that boundary requires, and §4 of spec
089 says so in the same breath: running the sweep in CI "is its own spec, with
its own decision about the token". The default branch after a merge is not
untrusted input. It is the revision the gate has already cleared into the trunk,
which is exactly the revision §3.7 says the sweep is entitled to. This spec is
the one 089 invited, and its decision about the token is §3.4.

### 1.3 Why this spec is its own witness

Adding a workflow that calls `scripts/verify-sweep.sh` turns one of spec 089's
own acceptance lines red:

```
! grep -rqF 'verify-sweep' .github/workflows/ .claude/skills/
```

Nothing in the gate chain would have noticed. The change that closes "a merged
acceptance can go red with every check green" would itself have gone red with
every check green, and the mechanism it adds is the one that catches it. §3.6
says what is done about that line, and it is done under this spec's authority,
not by quietly editing an approved document.

That is the second time spec 089's block has been the evidence. §1.1's other
measurement is the same file going red for a different reason and staying red
for weeks. A spec whose acceptance nothing re-asks is not a spec with an
acceptance; it is a spec with a paragraph that used to be true.

## 2. Territory

| Path | What |
|---|---|
| `.github/workflows/acceptance.yml` | where the sweep runs, on which events, with which permissions |
| `scripts/acceptance-scope.sh` | a revision range to the spec ids it can have invalidated |
| `crates/spec-spine-core/tests/gate.rs` | the workflow's shape, asserted the way spec 094 asserts `ci.yml` |
| `scripts/verify-sweep.sh` | the header sentence this spec amends |

## 3. Behavior

### 3.1 Where it runs, and where it MUST NOT

The sweep MUST run from a workflow triggered **only** by:

- `push` to the default branch;
- `schedule`;
- `workflow_dispatch`.

The workflow MUST NOT carry a `pull_request`, `pull_request_target` or
`merge_group` trigger, and MUST NOT appear in any `needs:` list of `ci.yml`.
Both halves matter and they are different: the first keeps it from ever reading
a stranger's revision, the second keeps it from ever blocking a pull request.
Spec 094 §3.x makes `ci-gate` the single required check by aggregating the jobs
of one workflow; a separate workflow is outside that aggregate by construction,
which is why this is a structural property and not a promise.

A red sweep is therefore a failed run on the default branch's commit, which is
where a human looks, and never a blocked merge.

### 3.2 The scope after a merge

On `push`, the selection MUST be every spec that `index owner` names for a path
the pushed range changed, of any ownership kind, together with every spec whose
own `spec.md` the range changed.

The selection MUST be computed by a script, not spelled out in the workflow.
Spec 094 §3.6's rule for the gate is the same rule: a workflow calls the
definition, it does not restate it.

A selection may be the whole corpus. The commit that introduced this rule
touched 146 files and selected all 99 specs, which is the correct answer for
that change and not a degenerate one: the scope is proportional to the change,
not uniformly small.

An **empty** selection MUST be a success and MUST NOT run the sweep. A push that
changed only bypassed paths can have invalidated nothing that ownership knows
about, and a sweep of nothing that reports "all passed" would be a green light
for a question nobody asked.

### 3.3 The nightly leg sweeps the whole corpus

`schedule` MUST sweep the whole corpus, and `workflow_dispatch` MUST default to
the whole corpus while accepting a narrowed selection.

The nightly leg is not redundant with §3.2. A spec's acceptance can depend on a
file the spec does not own: spec 092's block asserts the **absence** of paths,
and an absence is owned by nobody. The push leg cannot select a spec that owns
none of the changed paths, so the scoped run is the fast signal and the full run
is the one that actually closes §1.1's gap.

### 3.4 The decision about the token

The workflow MUST declare `permissions: contents: read` and MUST NOT be given
any secret.

The sweep executes every command the corpus declares. What that code can reach
is what this block grants it, and a read-only checkout token is the whole of it:
no package-registry token, no review token, no write to the repository. Spec 089
§4 asked for this decision explicitly and this is it, stated as a requirement so
a later edit that adds `secrets: inherit` is a change to a spec rather than a
line in a YAML file.

### 3.5 The trusted ref is named, not inherited

The workflow MUST pass the default branch as the sweep's trusted ref, and MUST
NOT pass any other. Spec 089 §3.7 makes an explicit trusted ref an override that
is recorded in the report; naming the default branch is an override in form
only, and it is used because the remote-tracking refs a CI checkout leaves
behind are not something a trust check should depend on.

### 3.6 Spec 089's acceptance line, corrected in place

Spec 089's block asserts `! grep -rqF 'verify-sweep' .github/workflows/
.claude/skills/`. Half of that assertion is what this spec changes and half of
it still holds.

The line MUST be corrected in place, under this spec's authority, to assert the
half that survives: the sweep is in no skill's gate floor, and it is on no
`pull_request` leg. Correction in place MUST NOT weaken it: the replacement
asserts §3.1's refusals, which is strictly more than "the string does not
appear" said about the boundary either spec cares about.

The corrected line MUST be guarded by a `test -f` on the workflow. A bare
`! grep` against a file that does not exist succeeds, so an assertion about a
file this spec creates would turn green the moment somebody deleted it, which is
the one state it most needs to catch.

Replacement under `amends_verification` is **not** used. That would make this
spec's block spec 089's acceptance (spec 082 §3.2), discarding sixty commands
that test the sweep's five outcomes, its isolation and its refusals, none of
which this spec touches. An amendment that changes one sentence should not
delete the evidence for the rest.

### 3.7 What it does not do

The workflow MUST NOT repair anything the sweep finds, MUST NOT commit, and MUST
NOT open an issue or a pull request. Spec 089 §4 keeps remediation out of the
instrument, and a workflow that edits the corpus in response to its own reading
of the corpus is a different and much larger idea.

## 4. Out of scope

- **Making the sweep a required check.** §3.1. The reason `verify` is outside
  the gate chain is unchanged, and a job that spent 17 minutes in `verify` on
  this corpus (§1.1), on every pull request and every push to every branch,
  would be a different trade than the one this spec is making.
- **Failing the nightly for the 48-spec legacy ledger.** Spec 089 §3.4's ledger
  is applied unchanged; this spec neither shrinks nor extends it.
- **Notifying anywhere but the run's own status.** A failed scheduled workflow
  is visible in the repository's Actions view and in GitHub's own notification
  to the workflow's owner. Routing it to a chat or an issue is a preference with
  a token attached, and §3.4 is the only token decision this spec makes.
- **Adopters.** This is this repository's CI. What an adopter runs is Statecraft's
  to deliver (spec 092).

## 5. Resolved decisions

**D-1 (2026-09-21): amend 089 rather than write a second sweep.** The sweep is
correct; only the sentence about who may run it is too broad. A second
CI-shaped runner would duplicate the five outcomes, the ledger, the isolation
and the trust check, and the two would drift the first time one of them was
fixed.

**D-2 (2026-09-21): both legs, not one.** A per-merge scoped run is minutes and
catches the common case; a nightly full run is the only thing that catches a
break in a spec that owns none of the changed paths, which is precisely the
shape of the break that motivated this spec (§3.3). Keeping only the scoped leg
would have left #277's defect undetected.

**D-3 (2026-09-21): an empty scope is a pass with nothing run.** The alternative,
sweeping the whole corpus whenever the scope is empty, turns a docs-only push
into a ninety-minute job and trains a reader to ignore the workflow. The nightly
already covers what the scope cannot see.

**D-4 (2026-09-21): the step's exit status is the sweep's, explicitly.** The
first draft of this workflow ended its run block with
`echo "sweep exit: $?"`, which makes every sweep green because the step's status
is the last command's. It is the same defect spec 089's own family is about, and
it survived one reading. The block now captures `rc` and ends with `exit $rc`,
and `tests/gate.rs` asserts that.

**D-5 (2026-09-21): the assertions about this workflow skip its comments, and
were mutation-tested.** The workflow explains at length why it has no secret,
and the first draft of §3.4's assertion read that sentence and refused the file
for saying so. Both the suite's copy and the acceptance line now exclude comment
lines, which is the rule `tests/gate.rs::invocations` already applies for the
same reason: prose naming a thing is not a declaration of it.

An assertion that skips prose can skip too much, so both were run against a
deliberately broken copy on 2026-09-21: the workflow given
`pull-requests: write`, `secrets.NPM_TOKEN` in the sweep step's `env`, and a
`git push origin HEAD` in its run block. Two of the five tests went red and both
acceptance lines returned 1. Without that, "the grep finds nothing" and "the
grep looks at nothing" are the same green.

**D-6 (2026-09-21): the fixture writes its answers outside the repository it is
measuring.** The first draft wrote each scope into `${TMPDIR}/ss099/…`, inside
the fixture repo. The next commit's `git add -A` carried that file, the scope of
the commit after it asked `index owner` about a path no shard covers, and the
run refused. It is spec 089 §3.6's rule in miniature, the same reason that
script refuses an `--out` inside the repository under test. The refusal was
correct behaviour and is now asserted deliberately, with the accident that found
it recorded beside it: a stale ledger is a question that cannot be asked, not an
empty answer, because an empty answer here means "sweep nothing".

**D-7 (2026-09-21): a sweep reads the script it is running from, and a
maintainer's checkout is a place where that file changes.** The full sweep taken
for §1.1 died after its last spec with a syntax error on a line that is
syntactically fine. `bash` reads a script incrementally; the session editing
`scripts/verify-sweep.sh` under §3.6 shifted every byte after the header, and
the interpreter resumed at an offset that no longer began a statement.

What survived is the per-spec loop, already parsed and in memory, which is why
§1.1 can read its counts off those lines; what died is the report writer after
it. That is luck, not design. A second attempt from a frozen copy was killed by
memory pressure before it reached a single spec.

This is a second argument for §3.1's home rather than an accident beside it. A
maintainer sweeps from the working tree they are working in, with the rest of
their session competing for the machine; the workflow sweeps from a checkout
nobody is editing, of a revision that is already merged, on a runner doing
nothing else. §1.1's two caveats are both instances of that difference.

## Verification

Each line is one command, run independently: no shell variable survives to the
next line, so the fixture under `${TMPDIR:-/tmp}/ss099` carries the state.

The fixture is a two-spec-file corpus with one owned source file and one
unowned one, which is the smallest tree on which the two halves of §3.2 differ:
a change to the owned file must select its spec, and a change to the unowned one
must select nothing.

Each negative assertion about a file this spec creates is guarded by a `test -f`
on the same line. A bare `! grep` on a file that does not exist passes, which
would make three of the lines below green before anything was written.

**Fail-first evidence**, measured on 2026-09-21 on this branch before the build:
`.github/workflows/acceptance.yml` and `scripts/acceptance-scope.sh` do not
exist, so every line below that names either is red; and
`grep -c 'verify-sweep' .github/workflows/*` is `0`, which is the state spec
089's acceptance asserted and §3.6 corrects.

```verify:cli
cargo build --release --locked
# --- 3.1: what the workflow can read, and what it can block ---
test -f .github/workflows/acceptance.yml
test -f .github/workflows/acceptance.yml && ! grep -qE '^[[:space:]]*(pull_request|pull_request_target|merge_group):' .github/workflows/acceptance.yml
grep -qE '^[[:space:]]*schedule:' .github/workflows/acceptance.yml
grep -qE '^[[:space:]]*workflow_dispatch:' .github/workflows/acceptance.yml
! grep -qF 'acceptance' .github/workflows/ci.yml
# --- 3.4: the decision about the token ---
grep -qF 'contents: read' .github/workflows/acceptance.yml
# Comment lines excluded: the workflow explains at length why it has no
# secret, and a grep that read that sentence would refuse the file for saying
# so. The same exclusion is in the suite's copy of this assertion.
test -f .github/workflows/acceptance.yml && ! grep -v '^[[:space:]]*#' .github/workflows/acceptance.yml | grep -qE 'secrets:|secrets\.'
# --- 3.5 and D-4: the trusted ref is named, and the sweep's exit is the step's
grep -qF -- '--trusted-ref origin/main' .github/workflows/acceptance.yml
grep -qF 'exit $rc' .github/workflows/acceptance.yml
# --- 3.7: the instrument reports, it does not repair ---
test -f .github/workflows/acceptance.yml && ! grep -v '^[[:space:]]*#' .github/workflows/acceptance.yml | grep -qE 'git (push|commit)|gh (pr|issue) create'
# --- 3.2: the scope is a script, and it refuses a question it cannot ask ---
test -x scripts/acceptance-scope.sh
scripts/acceptance-scope.sh --head HEAD >/dev/null 2>&1; test $? -eq 3
scripts/acceptance-scope.sh --base nope-not-a-rev --head HEAD >/dev/null 2>&1; test $? -eq 3
# --- 3.2 on a fixture: an owned change selects its spec ---
rm -rf "${TMPDIR:-/tmp}/ss099" "${TMPDIR:-/tmp}/ss099-owned.txt" "${TMPDIR:-/tmp}/ss099-unowned.txt" "${TMPDIR:-/tmp}/ss099-doc.txt"
mkdir -p "${TMPDIR:-/tmp}/ss099/specs/000-alpha" "${TMPDIR:-/tmp}/ss099/src"
printf '[layout]\nspecs_dir = "specs"\nderived_dir = ".derived"\n' > "${TMPDIR:-/tmp}/ss099/spec-spine.toml"
printf -- '---\nid: "000-alpha"\ntitle: "Alpha"\nstatus: approved\ncreated: "2026-09-21"\nsummary: "s"\nimplementation: complete\nestablishes:\n  - "src/a.rs"\n---\n\n# 000: Alpha\n\n## 1. Purpose\n' > "${TMPDIR:-/tmp}/ss099/specs/000-alpha/spec.md"
printf 'fn a() {}\n' > "${TMPDIR:-/tmp}/ss099/src/a.rs"
printf 'fn b() {}\n' > "${TMPDIR:-/tmp}/ss099/src/b.rs"
git -C "${TMPDIR:-/tmp}/ss099" init -q
git -C "${TMPDIR:-/tmp}/ss099" add -A
git -C "${TMPDIR:-/tmp}/ss099" -c user.email=t@example.invalid -c user.name=t commit -qm base
target/release/spec-spine compile --repo "${TMPDIR:-/tmp}/ss099" > /dev/null
target/release/spec-spine index --repo "${TMPDIR:-/tmp}/ss099" > /dev/null
git -C "${TMPDIR:-/tmp}/ss099" add -A
git -C "${TMPDIR:-/tmp}/ss099" -c user.email=t@example.invalid -c user.name=t commit -qm derived
printf '// owned change\n' >> "${TMPDIR:-/tmp}/ss099/src/a.rs"
git -C "${TMPDIR:-/tmp}/ss099" add -A
git -C "${TMPDIR:-/tmp}/ss099" -c user.email=t@example.invalid -c user.name=t commit -qm owned
scripts/acceptance-scope.sh --base HEAD~1 --head HEAD --repo "${TMPDIR:-/tmp}/ss099" --bin "$PWD/target/release/spec-spine" > "${TMPDIR:-/tmp}/ss099-owned.txt"
grep -qx '000-alpha' "${TMPDIR:-/tmp}/ss099-owned.txt"
# --- 3.2: and an unowned change selects nothing, which is a pass, not a sweep
printf '// unowned change\n' >> "${TMPDIR:-/tmp}/ss099/src/b.rs"
git -C "${TMPDIR:-/tmp}/ss099" add -A
git -C "${TMPDIR:-/tmp}/ss099" -c user.email=t@example.invalid -c user.name=t commit -qm unowned
scripts/acceptance-scope.sh --base HEAD~1 --head HEAD --repo "${TMPDIR:-/tmp}/ss099" --bin "$PWD/target/release/spec-spine" > "${TMPDIR:-/tmp}/ss099-unowned.txt"
test ! -s "${TMPDIR:-/tmp}/ss099-unowned.txt"
# --- 3.2: a change to a spec's own document selects that spec ---
printf 'A sentence.\n' >> "${TMPDIR:-/tmp}/ss099/specs/000-alpha/spec.md"
git -C "${TMPDIR:-/tmp}/ss099" add -A
git -C "${TMPDIR:-/tmp}/ss099" -c user.email=t@example.invalid -c user.name=t commit -qm doc
scripts/acceptance-scope.sh --base HEAD~1 --head HEAD --repo "${TMPDIR:-/tmp}/ss099" --bin "$PWD/target/release/spec-spine" > "${TMPDIR:-/tmp}/ss099-doc.txt"
grep -qx '000-alpha' "${TMPDIR:-/tmp}/ss099-doc.txt"
# 3.2: a stale ledger is a question that cannot be asked, not an empty answer.
# Found by this block: an earlier draft wrote its output INTO the fixture repo,
# the next commit carried it, and the scope of an unindexed file refused.
printf 'fn c() {}\n' > "${TMPDIR:-/tmp}/ss099/src/c.rs"
git -C "${TMPDIR:-/tmp}/ss099" add -A
git -C "${TMPDIR:-/tmp}/ss099" -c user.email=t@example.invalid -c user.name=t commit -qm stale
scripts/acceptance-scope.sh --base HEAD~1 --head HEAD --repo "${TMPDIR:-/tmp}/ss099" --bin "$PWD/target/release/spec-spine" >/dev/null 2>&1; test $? -eq 3
rm -rf "${TMPDIR:-/tmp}/ss099" "${TMPDIR:-/tmp}/ss099-owned.txt" "${TMPDIR:-/tmp}/ss099-unowned.txt" "${TMPDIR:-/tmp}/ss099-doc.txt"
# --- 3.6: spec 089's line, corrected rather than deleted ---
grep -qF '.claude/skills/' specs/089-nothing-reruns-a-merged-acceptance/spec.md
grep -qF 'workflows/acceptance.yml' specs/089-nothing-reruns-a-merged-acceptance/spec.md
# --- the workflow's shape, asserted in the suite as well as here ---
cargo test -p spec-spine-core --test gate --locked acceptance > "${TMPDIR:-/tmp}/ss099-g.txt" 2>&1
grep -qE 'test result: ok\. [1-9][0-9]* passed' "${TMPDIR:-/tmp}/ss099-g.txt"
rm -f "${TMPDIR:-/tmp}/ss099-g.txt"
# --- the governed loop, over the corpus this spec is part of ---
target/release/spec-spine check --fail-on-unresolved --fail-on-warn
target/release/spec-spine lint --fail-on-warn
```
