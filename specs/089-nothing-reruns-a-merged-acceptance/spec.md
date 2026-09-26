---
id: "089-nothing-reruns-a-merged-acceptance"
title: "Nothing reruns a merged acceptance"
status: approved
kind: "tooling"
created: "2026-09-17"
summary: >
  A `## Verification` block is run once, by the session that writes it, and then
  never again. `verify` is outside the gate chain on purpose, so a block that a
  later approved spec legitimately invalidates goes red silently and stays red:
  specs 084 through 110 each exist to repair one such block, and each was found
  by hand. Spec 083 §4 named the remedy, "a periodic sweep a maintainer runs,
  not a gate step", and left naming its home as its own work. This spec is that
  home. It establishes `scripts/verify-sweep.sh`, a maintainer-invoked sweep
  that runs the whole corpus's declared acceptance against one already-merged
  revision in an isolated worktree, accounts for every spec under five outcomes
  of which only two are success, and closes the 000-047 population as a ledger
  that can only shrink and that no future spec can enter.
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "043-verify-declared-acceptance"
  - "082-an-amended-acceptance-is-the-one-that-runs"
  - "083-an-amendment-carries-the-acceptance-it-replaces"
establishes:
  # 3.1 - 3.8: the sweep itself.
  - "scripts/verify-sweep.sh"
  - "scripts/test-verify-sweep.py"
extends:
  # 3.9: the maintainer runbook gains the pre-flight item that says when to run
  # it. `docs/releasing.md` is spec 006's unit; this adds a line and claims
  # nothing else in the file.
  - { spec: "006-distribution", unit: "docs/releasing.md", nature: additive }
references:
  - { unit: { kind: file, path: "crates/spec-spine-core/src/verify.rs" }, role: "context" }
  - { unit: { kind: file, path: "crates/spec-spine-cli/src/cmd_verify.rs" }, role: "context" }
---
# 089: Nothing reruns a merged acceptance

## 1. Purpose

### 1.1 The gap, in the corpus's own record

Spec 083 §1.3 states it exactly:

> Neither reason is reachable by the governance gate. `verify` is the one verb
> that executes what the corpus declares, so it sits outside the gate chain
> deliberately (`AGENTS.md`); nothing in CI runs it. A block can be red for
> months and every gate stays green, which is exactly what happened.

A `## Verification` block is executed by the session that writes it, at
`/verify <id>` before `/ship`, and after that by nothing. The gate chain
(`check`, `lint`, `index coverage`, `couple`) never runs it, by a decision §3.7
keeps. So a block's assertions decay against a moving corpus with no signal at
all, and the decay is not hypothetical:

| spec | its acceptance went red because | repaired by |
|---|---|---|
| 093 | spec 074's block asserted a corpus-dependent value | 103 |
| 098 | spec 080 amended away the exit code five lines required | 105 |
| 071 | a later spec moved the output an assertion pinned | 110 |
| 060 | the answer became a member rather than the document | 109 |
| 059 | an exact key set refused what the rule allows | 108 |
| 019 | a version pin stopped being a contract | 107 |
| 057 | an acceptance outlived the output it was written against | 106 |

Seven blocks, seven repairs, and every one of them found because a human went
looking. Spec 082 §4 called the audit "its own work, with its own findings";
spec 083 §4 called the remedy a periodic sweep and declined to name its home.
Both items are still open, and they are one item: the audit is what the sweep's
first run *is*.

### 1.2 What the corpus declares today, measured

Measured on 2026-09-17 against `4e114fd` with the 0.20.0 binary, over all 112
specs, using `verify <id> --plan` (which executes nothing):

```
declared=64  not-declared=48
```

The 48 are exactly `000-spec-spine-bootstrap` through
`093-the-harness-this-repository-runs`: a contiguous run with no gaps
and no member above it. That is not a coincidence and not neglect. Spec 043
built the `verify` verb and spec 093, filed immediately before it, is the first
spec in this corpus to carry a block. Every spec below 048 was written when
there was nothing to write a block *for*.

So the population has a clean, closed, historical boundary, which is what makes
it settleable rather than a standing 43% of the corpus reported as noise on
every run.

### 1.3 Why this is not `verify --all`

Spec 043 §4 declined a corpus-wide run in these words:

> **`verify --all`.** A corpus-wide run is a loop in a caller, and what it
> should do about `not-declared` specs (44 of 50 here) is a policy this spec
> has no grounds to pick.

Both halves are still right, and this spec is the one with the grounds. It
supplies the policy (§3.4, §3.5) and it puts the loop in the caller (§3.1)
rather than in the verb, so nothing about `verify` changes: the engine stays a
pure function of `(config, file contents)`, the CLI keeps executing one spec,
and no schema, no dependency and no subcommand is added to the tool that
adopters install.

### 1.4 The first run, measured

Run against `4e114fd` (the merged tip of `main` at the time of writing) with a
binary built from that revision, in an isolated worktree:

```
verify-sweep.sh: 4e114fd  passed=64 failed=0 not-declared=0 exempt=48 not-run=0
ran: 2026-09-17T18:22:19Z .. 2026-09-17T18:28:43Z   (6m24s, 112 specs)
```

The corpus is clean, and that is the expected reading rather than a
disappointing one: specs 083 through 111 repaired the seven blocks of §1.1
between 2026-09-16 and 2026-09-17, so this run is measuring a corpus whose
known debt was retired days ago. What changed is not the state, it is the cost
of knowing it: six minutes and one command, against the by-hand discovery that
produced seven specs. The run also confirms the ledger's four properties
(§3.4) against the real corpus, since a stale, dangling or out-of-range entry
would have refused before any block executed.

Worth writing down for whoever runs it next: no block left the worktree dirty,
and the whole corpus fits inside one `cargo build --release`, so after the
first build the 64 declared blocks cost about five minutes between them.

## 2. Territory

- `scripts/verify-sweep.sh` (`establishes`): the sweep. New file.
- `docs/releasing.md` (`extends` on spec 006): one pre-flight item naming when
  a maintainer runs it.

Nothing else. This spec adds no Rust, changes no verb, touches no committed
artifact by hand, and edits no approved spec.

## 3. Behavior

### 3.1 Where it lives and who runs it

The sweep MUST be a standalone script under `scripts/`, invoked by a human
maintainer. It MUST NOT be a CLI subcommand, a CI job, a git hook, a
`Makefile` gate target, or a step in any skill's gate floor.

The reason is the trust boundary of §3.7, and a second one: the sweep needs a
wall clock, a `git worktree`, a process group and a kill signal. Every one of
those is a thing `spec-spine-core` is forbidden to have (the invariant in
`CLAUDE.md`: "pure function of `(Config, file contents)`: no ambient clock, no
env reads, no `git`"). A caller is where they belong.

### 3.2 It runs the existing mechanism, unchanged

The sweep MUST obtain each spec's acceptance through `spec-spine verify`, and
MUST NOT parse `## Verification` itself. `verify` already resolves
`amends_verification` and follows the chain to its end (spec 082 §3.2), so a
block carried for an amended spec is the block the sweep runs, and the sweep
gets that for free rather than reimplementing it. Its selection MUST come from
`spec-spine registry list --ids-only`, a governed read
(`AGENTS.md` "Governed artifact reads"); the sweep MUST NOT read
`.derived/**` directly.

Whether a spec declares acceptance MUST be decided with `verify <id> --plan`,
which executes nothing (spec 043 §3.8).

### 3.3 Five outcomes, and only two of them are success

Every selected spec MUST receive exactly one outcome:

| outcome | meaning | success |
|---|---|---|
| `passed` | declared, ran, every command exited 0 | yes |
| `failed` | declared, ran, a command exited non-zero | no |
| `not-declared` | no `verify:cli` commands, and not on the ledger | no |
| `exempt` | on the closed legacy ledger of §3.4 | yes |
| `not-run` | no verdict: the verb could not answer, or the limit was exceeded | no |

`not-declared` MUST NOT count as success. This is the policy spec 043 §4 said
it had no grounds to pick, and it is picked this way because the alternative is
a report whose pass rate is computed over the specs that happened to declare
something, which is precisely the shape that let seven blocks rot unobserved.

Outcome MUST be derived from `verify`'s documented exit-code contract (`0`
passed or not-declared, `1` failed, `2` stale, `3` I/O / parse / schema /
config), not from matching its prose. A code outside `0`/`1` is `not-run`, with
the code recorded: the verb did not reach a verdict, and saying so is not the
same as saying the spec failed.

`not-declared` MUST be reported only when `verify --plan` **succeeded** and
returned no commands. A spec whose plan could not be read at all is `not-run`.
The two are different facts: one says the spec declares no acceptance, the
other says the sweep could not find out, and a spec the committed registry
still lists whose document has been removed is the second, not the first.

The sweep MUST exit `0` only when every selected spec is `passed` or `exempt`;
`1` when any is `failed`, `not-declared` or `not-run`; and `3` when it refused
to run at all (§3.4, §3.7, usage, I/O).

### 3.4 The 000-047 population: a closed ledger that only shrinks

The 48 specs of §1.2 MUST be carried as an enumerated ledger of spec ids, with
the ordinal at which the ledger is closed declared beside it. The ledger MUST
NOT be a range, a predicate, a date comparison or an "ordinal below N" rule
evaluated against the corpus: those forms cover specs that do not exist yet,
and an exemption that grows by itself is the noise this spec exists to avoid.

Four properties MUST hold, the last three enforced mechanically, each a refusal
(exit `3`) raised before any block is executed:

1. **It is tracked debt, not an exception.** The ledger names every exempted
   spec individually, so `git log -p` on the script is the debt's history and a
   removal is a reviewable line.
2. **It is closed.** An entry at or above the closing ordinal (48) MUST be
   refused. No spec filed after the mechanism existed can be exempted, so a
   future spec with no acceptance is `not-declared`, the sweep exits 1, and the
   answer is to write the block.
3. **It only shrinks.** An entry whose spec declares acceptance MUST be refused
   as a stale exemption. A spec cannot both be excused and be checked, and the
   debt cannot be paid and then silently re-incurred.
4. **It is live.** An entry naming no spec in the corpus MUST be refused. A
   dangling exemption accounts for nothing.

Retirement is therefore optional in timing and enforced in direction: nobody is
obliged to write a block for spec 011 this quarter, and the moment somebody
does, the line comes out or the sweep refuses.

The sweep MUST accept an alternative ledger path, so the mechanism above is
testable against a fixture corpus rather than only against this repository's own
48 entries.

### 3.5 Selection

The default selection MUST be the whole corpus at the tested revision. The
sweep MUST accept a narrowed selection by spec id, so a maintainer can re-run a
finding without re-running the corpus, and MUST record which selection produced
a report.

A spec MUST appear at most once in a selection: naming it twice MUST select it
once, since a spec run twice is counted twice and the report would then say the
corpus is larger than it is.

An id is a full `NNN-slug` or the 3-digit ordinal `NNN`. That is the short form
`spec-spine` itself resolves everywhere (spec 015, spec 067 §3.4): the tool
answers `not found: spec '49'`, so the sweep accepts `049` and refuses `49` too.
Inventing a wider short form here would make the sweep the one place in the
project where an id means something else, and its refusal MUST say which form it
wants rather than only that it found nothing.

### 3.6 Isolation, fixtures, and what happens after a failure

- **Isolation.** The sweep MUST run in a detached `git worktree` at the tested
  revision, never in the maintainer's checkout. Blocks are arbitrary shell that
  writes fixtures and regenerates artifacts; doing that to a tree someone is
  mid-change in is not acceptable for routine maintenance. The worktree MUST be
  removed when the run ends.
- **The report is written outside the repository.** The sweep MUST refuse an
  output directory inside the repository under test, so a sweep can never dirty
  the tree the gate judges, and it MUST refuse before creating anything: an
  `--out` naming a path whose parent directories do not exist yet must leave the
  repository exactly as it found it. The containment test MUST compare physical
  paths;
  on macOS `git rev-parse --show-toplevel` answers `/private/var/...` where a
  `cd`+`pwd` answers `/var/...`, and a guard comparing the two forms passes a
  path that is in fact inside.
- **The run directory is cleared before use, so it MUST be one the sweep is
  entitled to delete**: absent, empty, or marked by a previous run. A slipped
  `--out` at a directory holding anything else MUST be refused with its
  contents intact. The marker MUST be written when the directory is created
  rather than when the report is: a run that died half-way is exactly the
  directory a maintainer retries, and keying on the report would refuse it.
- **Fixtures.** Each spec MUST run with its own `TMPDIR`. Blocks in this corpus
  build fixture corpora under `${TMPDIR}/ssNNN`; a shared `TMPDIR` would let
  one spec's residue decide another spec's outcome.
- **After a failure the sweep continues.** A failing block is one spec's
  result. The sweep MUST NOT stop, so one red block cannot cost the verdict on
  the other 111.
- **A block that dirties the worktree is recorded and the tree restored** before
  the next spec runs, so each spec is judged against the tested revision and not
  against the leftovers of the last one. Ignored paths (`target/`,
  `build-meta.json`) are deliberately kept; rebuilding them per spec would cost
  hours.
- **A hanging block MUST NOT cost the rest of the run.** A per-spec limit is
  applied, and a spec that exceeds it is `not-run`. The limit MUST be enforced
  by the sweep itself rather than delegated to `timeout(1)`, which stock macOS
  does not ship: a limit that silently does not exist on half the machines that
  run this is worse than none, because the report would still claim one. The
  child MUST be signalled as a process group, so the tree a block spawned dies
  with it; where the shell did not give the job its own group, the sweep MUST
  signal the process alone and MUST record that it did, since whatever the
  block spawned then outlives the kill and a degraded run must not read as a
  clean one. A block that reached a verdict on its own in the moment between
  the liveness check and the signal MUST keep that verdict: the sweep reports
  what it observed, and `not-run` for a spec that in fact ran is an outcome it
  invented.

### 3.7 The trust boundary is preserved and made mechanical

`verify` executes what the corpus declares. Sweeping executes what *every* spec
declares, which is strictly more code, so the trust boundary spec 043 §3.6 and
spec 083 §4 draw MUST hold and MUST be enforced rather than documented.

The sweep MUST refuse (exit `3`) a revision that is not an ancestor of a trusted
ref, which defaults to the repository's `origin/HEAD` (falling back to
`origin/main`). A PR branch is, in the general case, a stranger's code; the
sweep cannot be pointed at one by accident. Naming a different trusted ref MUST
be an explicit argument, and MUST be recorded in the report, so the override is
visible in the evidence rather than lost in shell history.

Arbitrary corpus verification therefore stays out of the PR gate in both
directions: the gate never runs the sweep, and the sweep never runs a revision
the gate has not already cleared into the trunk.

### 3.8 The report is the evidence

The sweep MUST write a machine-readable report and a human summary, and both
MUST carry, at minimum:

- the tested revision (full sha) and the trusted ref it was checked against;
- the `spec-spine` version that produced the result, and where that binary came
  from;
- the selection and the ledger that were applied;
- per spec: the outcome, the number of declared commands, `verify`'s exit code,
  the failing command when there was one, whether the block dirtied the tree,
  and the path of the log holding that spec's output.

A log path MUST be cited only when that log exists. A spec whose plan could not
be read never reached a run and has no output, and a report that names a file
which is not there sends the reader after evidence that was never produced.

By default the binary MUST be built from the tested revision inside the
worktree. It is the only binary whose version provably corresponds to the
corpus under test, and it is also the binary the blocks themselves reach for:
they invoke `target/release/spec-spine` relative to the repository root, which
in a sweep is the worktree. An already-built binary MAY be supplied instead,
and the report MUST say which of the two produced it.

### 3.9 When a maintainer runs it

`docs/releasing.md`'s pre-flight MUST name the sweep, because a release is the
moment the corpus's claims are published and the cheapest moment to find that
one of them is no longer true. The runbook MUST also name the other trigger:
after merging a spec that carries `amends` or `amends_verification`, which is
the crossing that staled every block specs 083 through 110 repaired.

## 4. Out of scope

- **Repairing anything the first run finds.** Spec 083 fixed one block and
  specs 084-110 fixed one each, and each was a spec's worth of work because
  deciding what an acceptance should assert is a judgement, not a refresh. This
  spec builds the instrument and reports its reading; a finding becomes a filed
  spec, and this session files one spec.
- **Retiring the 000-047 debt.** §3.4 makes retirement possible, directional
  and reviewable. Actually writing 48 acceptance blocks for specs whose code
  has been under the coupling gate for months is a large, low-urgency body of
  work with no deadline this spec is entitled to set.
- **Running the sweep in CI, on a schedule, or from a hook.** §3.1 and §3.7.
  The trust boundary is the whole reason `verify` is outside the gate chain,
  and a scheduled runner in this repository's CI would execute a merged corpus
  with the repository's own credentials in scope. If that is ever wanted it is
  its own spec, with its own decision about the token.
- **Shipping the sweep to adopters in `kit/`.** The kit's skills are pinned
  byte-identical to `.claude/skills/` (specs 093, 081) and `kit_embedded.rs` is
  generated from the kit tree, so adding a maintainer tool there is a second
  change with its own regeneration and its own adopter-upgrade story. The
  mechanism should earn its first run here first. Recorded as follow-up, not
  done.
- **A `## Verification` block for any of the 48.** See above.
- **Changing `verify`, its grammar, its exit codes or its report.** §3.2. The
  sweep is a caller. Spec 043's verb is unchanged and this spec amends nobody.
- **Making the sweep's report a committed artifact, or a schema in
  `spec-spine-types`.** It is a run record written outside the repository
  (§3.6), not part of the ledger. `schemaVersion` inside it is the script's own
  and is versioned by the script.

## 5. Resolved decisions

- **D-1 (2026-09-17, the surface is a script, not `verify --all`).** Spec 043
  §4 declined the flag on two grounds: the loop belongs in a caller, and the
  `not-declared` policy needed grounds it did not have. This spec supplies the
  second and honours the first. A flag would also have put a wall clock, a
  `git worktree` and a `SIGKILL` inside the tool adopters install, for a run
  nobody but a maintainer performs. §3.1.
- **D-2 (2026-09-17, the ledger is enumerated and inline, not a range).** A
  predicate ("ordinal below 48") covers specs that do not exist yet only by
  accident of numbering, and a range hides which specs are actually excused.
  The enumeration is carried inside the script rather than in a sibling data
  file because `[index] extra_hashed_inputs` already folds `scripts/*.sh` into
  the ledger's global scalar: a new data file with another extension would be a
  claimed governance file that no content hash reads, which is exactly what
  `L-008` refuses, and adding a glob for it would restale every shard for a
  breadth nothing else needs. §3.4.
- **D-3 (2026-09-17, `not-declared` is a failure, not a warning).** The
  alternative considered was reporting it at a warning tier so the sweep could
  exit 0 on a corpus with known debt. Rejected: a sweep that exits 0 while 48
  specs assert nothing is a green light over an unmeasured corpus, and the
  ledger of §3.4 already gives the debt a home where it is accounted for
  explicitly and cannot grow. §3.3.
- **D-4 (2026-09-17, the per-spec limit is implemented here, not delegated).**
  `timeout(1)` is not on stock macOS, and half this project's development
  happens there. A limit the report claims and the platform does not enforce is
  a worse failure than no limit, so the poll-and-signal loop is in the script
  and the behaviour is the same everywhere. §3.6.
- **D-5 (2026-09-17, the trust boundary is enforced, not documented).** Spec
  083 §4 states it as a reason; a sentence in a runbook does not stop a
  maintainer from running the sweep on the branch they just checked out. The
  ancestor test makes the boundary mechanical and the override visible in the
  report. §3.7.
- **D-6 (2026-09-17, the containment guard compares physical paths).** Found by
  building this: the first `--out` guard compared `git rev-parse
  --show-toplevel` (`/private/var/...`) against a `cd`+`pwd` (`/var/...`) and
  waved through a path inside the repository. Both sides are now `pwd -P`, and
  the canonicalization creates nothing: an earlier form resolved the path by
  `mkdir -p`'ing its parent first, which refused `--out <repo>/new/run` only
  after `<repo>/new` had been created inside the tree it was protecting. It now
  walks up to the deepest existing ancestor instead. §3.6.
- **D-7 (2026-09-17, the acceptance is a fixture corpus, not this one).** This
  spec's `## Verification` cannot be "run the sweep on this repository": that
  run takes hours, and its outcome depends on 111 other specs' blocks, so the
  block would go red for reasons that are not this spec's. The fixture corpus
  exercises all five outcomes, all four ledger refusals, the trust boundary,
  the containment guard, the per-spec limit and the dirty-tree restoration, in
  seconds and hermetically.

- **D-8 (2026-09-17, `--out` is guarded by a creation marker, not by the
  report).** The guard exists because the run directory is `rm -rf`'d before
  use. The first form accepted a directory only if it held a `sweep.json`,
  which refused every retry after an aborted run, found by this spec's own
  acceptance failing at the second invocation that reused an `--out`. The
  marker is written at `mkdir` time instead. §3.6.

### Implementation review (2026-09-17)

The post-ratification review reproduced two violations of the existing contract:
an unreadable ledger member counted as `exempt` instead of `not-run` (§3.3),
and staged or committed fixture changes survived restoration (§3.6). The repair
reads a selected spec's plan before granting exemption and restores the isolated
worktree's index and tracked files from the tested SHA, detaching without moving
a branch. Restoration errors refuse the sweep before another block runs. The
requirements, lifecycle, ledger and acceptance below are unchanged.

Review remediation preserves the four regression fixtures in
`scripts/test-verify-sweep.py`, claimed above through the legitimate ownership
ratchet. Run `python3 scripts/test-verify-sweep.py` with the in-tree release
binary built. The runner uses disposable repositories and checks unreadable
exemptions, clean worktree/index/HEAD bytes in the next spec after staged and
committed mutations, and nonexecution after restoration fails. It supplements
the 66 acceptance commands below. Keeping these checks only in temporary review
files was rejected because the existing acceptance passed before the repair.
For a negative control, export the pre-repair script with
`git show 1d0e3c3c890345fd85072c0b460768be43ee6e83:scripts/verify-sweep.sh`
to a temporary file and pass its path via `--sweep-script`; all four tests must
fail. The runner never replaces the source checkout's sweep script.

## Verification

Each line is one command, run independently: no shell variable survives to the
next line, so the fixture under `${TMPDIR:-/tmp}/ss112` carries the state.

The fixture is a six-spec corpus laid out so that one run produces all five
outcomes of §3.3, with the run directory outside the fixture repository (§3.6)
and the real binary supplied through `$SPEC_SPINE_BIN` so nothing has to be
compiled twice. `006-after-dirty` asserts the absence of the file `005-dirty`
created: it is the witness for the restoration in §3.6, and it was observed to
fail when the `git clean` line was removed.

**Fail-first evidence**, measured at the parent commit `4e114fd` by running
every line of this plan in a worktree of it. Lines 1-14 build the fixture and
legitimately succeed there. The first assertion, line 15, is the first sweep
invocation and it is red (`scripts/verify-sweep.sh` does not exist, so the shell
answers 127 and the `test $? -eq 1` fails), which is where `verify` stops.

Three lines are deliberately **not** fail-first, each asserting an absence that
must be true before this spec and must stay true after it: `--help` must not
grow a `sweep` subcommand (§3.1); no workflow, skill or kit file may name the
script (§3.1, §4); and the refused `--out` must have left nothing inside the
swept repository (the refusal itself, on the line above it, is red at the
parent). Every other assertion was checked individually against that worktree
and is red there. Four were green there only because they grepped or stat'd a
path that did not exist yet; each now carries an existence precondition, so
deleting the script or skipping the run cannot make them pass.

> **Superseded acceptance (2026-09-25).** This block no longer runs.
> `146-carried-acceptance-follows-139-and-144` declares this spec in
> `amends_verification`, so `spec-spine verify 089` builds its plan from that
> spec's block, where these commands are carried with one change and every
> other line unchanged (spec 082 3.2 and 3.4). Since 2026-09-26 146's block
> is itself carried by `154-the-pre-merge-sweep-is-a-skill-step`, so `verify
> 089` runs 154's block: 146's with one skills line restated for spec 150.

```verify:cli
cargo build --release --locked
# --- the fixture corpus: repo inside, scratch outside (3.6) ---
rm -rf "${TMPDIR:-/tmp}/ss112" && mkdir -p "${TMPDIR:-/tmp}/ss112/repo/specs"
printf '%s\n' '[layout]' 'specs_dir = "specs"' 'derived_dir = ".derived"' > "${TMPDIR:-/tmp}/ss112/repo/spec-spine.toml"
for s in 000-legacy 001-green 002-red 003-silent 004-slow 005-dirty 006-after-dirty 007-ghost; do mkdir -p "${TMPDIR:-/tmp}/ss112/repo/specs/$s" && printf '%s\n' '---' "id: \"$s\"" "title: \"Sweep fixture $s\"" 'status: draft' 'implementation: pending' 'created: "2026-09-17"' 'summary: >' '  A sweep fixture.' '---' '' "# $s" > "${TMPDIR:-/tmp}/ss112/repo/specs/$s/spec.md"; done
printf '%s\n' '' '## Verification' '' '```verify:cli' 'true' '```' >> "${TMPDIR:-/tmp}/ss112/repo/specs/001-green/spec.md"
printf '%s\n' '' '## Verification' '' '```verify:cli' 'true' 'false' 'true' '```' >> "${TMPDIR:-/tmp}/ss112/repo/specs/002-red/spec.md"
printf '%s\n' '' '## Verification' '' '```verify:cli' 'sleep 120' '```' >> "${TMPDIR:-/tmp}/ss112/repo/specs/004-slow/spec.md"
printf '%s\n' '' '## Verification' '' '```verify:cli' 'touch residue.txt' '```' >> "${TMPDIR:-/tmp}/ss112/repo/specs/005-dirty/spec.md"
printf '%s\n' '' '## Verification' '' '```verify:cli' 'test ! -e residue.txt' '```' >> "${TMPDIR:-/tmp}/ss112/repo/specs/006-after-dirty/spec.md"
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss112/repo" compile >/dev/null && target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss112/repo" index >/dev/null
git -C "${TMPDIR:-/tmp}/ss112/repo" init -q -b main . && git -C "${TMPDIR:-/tmp}/ss112/repo" add -A && git -C "${TMPDIR:-/tmp}/ss112/repo" -c user.email=a@b -c user.name=t commit -qm fixture
# 3.3: a spec the committed registry still lists whose document is gone. Its
# acceptance cannot be read, which is not the same fact as having none.
git -C "${TMPDIR:-/tmp}/ss112/repo" rm -rq specs/007-ghost && git -C "${TMPDIR:-/tmp}/ss112/repo" -c user.email=a@b -c user.name=t commit -qm ghost
git -C "${TMPDIR:-/tmp}/ss112/repo" checkout -q -b side && printf '\n<!-- unmerged -->\n' >> "${TMPDIR:-/tmp}/ss112/repo/specs/003-silent/spec.md" && git -C "${TMPDIR:-/tmp}/ss112/repo" add -A && git -C "${TMPDIR:-/tmp}/ss112/repo" -c user.email=a@b -c user.name=t commit -qm side && git -C "${TMPDIR:-/tmp}/ss112/repo" checkout -q main
printf '000-legacy\n' > "${TMPDIR:-/tmp}/ss112/exempt.txt"
# --- 3.3, 3.5, 3.6: one run, five outcomes, and it never stops ---
SPEC_SPINE_BIN="$PWD/target/release/spec-spine" scripts/verify-sweep.sh --repo "${TMPDIR:-/tmp}/ss112/repo" --trusted-ref main --exempt-file "${TMPDIR:-/tmp}/ss112/exempt.txt" --out "${TMPDIR:-/tmp}/ss112/run" --timeout 5 >/dev/null 2>&1; test $? -eq 1
# 3.3: every spec is accounted for, and the four non-passing kinds are distinct
# from success. `006-after-dirty` passing is the witness that the tree was
# restored after `005-dirty` wrote to it (3.6, D-7).
python3 -c "import json;d=json.load(open('${TMPDIR:-/tmp}/ss112/run/sweep.json'));o={s['id']:s['outcome'] for s in d['specs']};assert o=={'000-legacy':'exempt','001-green':'passed','002-red':'failed','003-silent':'not-declared','004-slow':'not-run','005-dirty':'passed','006-after-dirty':'passed','007-ghost':'not-run'}, o"
python3 -c "import json;d=json.load(open('${TMPDIR:-/tmp}/ss112/run/sweep.json'));assert d['counts']=={'passed':3,'failed':1,'not-declared':1,'exempt':1,'not-run':2}, d['counts']"
# 3.3: the two `not-run` reasons are distinct and both are recorded. A spec
# whose plan cannot be read MUST NOT be reported as declaring no acceptance.
python3 -c "import json;d=json.load(open('${TMPDIR:-/tmp}/ss112/run/sweep.json'));s={x['id']:x for x in d['specs']};assert '--plan' in s['007-ghost']['failure'] and s['007-ghost']['exitCode']!=0;assert 'limit' in s['004-slow']['failure']"
# 3.8: the evidence a finding is reproduced from.
python3 -c "import json;d=json.load(open('${TMPDIR:-/tmp}/ss112/run/sweep.json'));assert len(d['revision'])==40;assert d['trustedRef']=='main';assert d['binaryVersion'].startswith('spec-spine ');assert d['selection']=='all';assert d['ledgerClosedAt']==43"
python3 -c "import json;d=json.load(open('${TMPDIR:-/tmp}/ss112/run/sweep.json'));s={x['id']:x for x in d['specs']};assert 'FAILED at command 2' in s['002-red']['failure'];assert s['002-red']['exitCode']==1;assert s['002-red']['log']=='logs/002-red.log';assert s['002-red']['commands']==3"
python3 -c "import json;d=json.load(open('${TMPDIR:-/tmp}/ss112/run/sweep.json'));s={x['id']:x for x in d['specs']};assert s['004-slow']['exitCode']==124 and 'limit' in s['004-slow']['failure'];assert s['005-dirty']['leftTreeDirty'] is True;assert s['001-green']['leftTreeDirty'] is False"
test -s "${TMPDIR:-/tmp}/ss112/run/logs/002-red.log"
# 3.8: every cited log exists, and a spec that never ran cites none.
python3 -c "import json,os;d=json.load(open('${TMPDIR:-/tmp}/ss112/run/sweep.json'));s={x['id']:x for x in d['specs']};assert s['007-ghost']['log'] is None;assert all(os.path.isfile('${TMPDIR:-/tmp}/ss112/run/'+x['log']) for x in d['specs'] if x['log'])"
grep -qF 'logs/002-red.log' "${TMPDIR:-/tmp}/ss112/run/sweep.md"
! grep -qF 'logs/007-ghost.log' "${TMPDIR:-/tmp}/ss112/run/sweep.md"
grep -q 'not-declared' "${TMPDIR:-/tmp}/ss112/run/sweep.md"
# 3.6: the worktree is removed and the swept repository is left clean.
test -f "${TMPDIR:-/tmp}/ss112/run/sweep.json" && test ! -e "${TMPDIR:-/tmp}/ss112/run/tree"
test -f "${TMPDIR:-/tmp}/ss112/run/sweep.json" && test -z "$(git -C "${TMPDIR:-/tmp}/ss112/repo" status --porcelain)"
# --- 3.5: a narrowed selection, by short ordinal, is recorded ---
SPEC_SPINE_BIN="$PWD/target/release/spec-spine" scripts/verify-sweep.sh --repo "${TMPDIR:-/tmp}/ss112/repo" --trusted-ref main --exempt-file "${TMPDIR:-/tmp}/ss112/exempt.txt" --out "${TMPDIR:-/tmp}/ss112/one" --only 001 >/dev/null 2>&1; test $? -eq 0
python3 -c "import json;d=json.load(open('${TMPDIR:-/tmp}/ss112/one/sweep.json'));assert [s['id'] for s in d['specs']]==['001-green'];assert d['selection']=='001'"
# 3.5: an unpadded ordinal is refused the way the tool refuses it, and the
# refusal names the form it wants rather than only reporting a miss.
# 3.5: a spec named twice is selected once.
SPEC_SPINE_BIN="$PWD/target/release/spec-spine" scripts/verify-sweep.sh --repo "${TMPDIR:-/tmp}/ss112/repo" --trusted-ref main --exempt-file "${TMPDIR:-/tmp}/ss112/exempt.txt" --out "${TMPDIR:-/tmp}/ss112/dup" --only 001,001,001 >/dev/null 2>&1; test $? -eq 0
python3 -c "import json;d=json.load(open('${TMPDIR:-/tmp}/ss112/dup/sweep.json'));assert [s['id'] for s in d['specs']]==['001-green'], d['specs'];assert d['counts']['passed']==1"
SPEC_SPINE_BIN="$PWD/target/release/spec-spine" scripts/verify-sweep.sh --repo "${TMPDIR:-/tmp}/ss112/repo" --trusted-ref main --exempt-file "${TMPDIR:-/tmp}/ss112/exempt.txt" --out "${TMPDIR:-/tmp}/ss112/x" --only 1 >/dev/null 2>&1; test $? -eq 3
SPEC_SPINE_BIN="$PWD/target/release/spec-spine" scripts/verify-sweep.sh --repo "${TMPDIR:-/tmp}/ss112/repo" --trusted-ref main --exempt-file "${TMPDIR:-/tmp}/ss112/exempt.txt" --out "${TMPDIR:-/tmp}/ss112/x" --only 1 2>&1 | grep -q '3-digit ordinal'
target/release/spec-spine verify 49 >/dev/null 2>&1; test $? -eq 1
# --- 3.4: the ledger refuses, before anything is executed, in all four ways ---
# Closed: nothing at or above the closing ordinal can be exempted, so no future
# spec is covered by it.
printf '050-too-late\n' > "${TMPDIR:-/tmp}/ss112/e-closed.txt"
SPEC_SPINE_BIN="$PWD/target/release/spec-spine" scripts/verify-sweep.sh --repo "${TMPDIR:-/tmp}/ss112/repo" --trusted-ref main --exempt-file "${TMPDIR:-/tmp}/ss112/e-closed.txt" --out "${TMPDIR:-/tmp}/ss112/x" >/dev/null 2>&1; test $? -eq 3
SPEC_SPINE_BIN="$PWD/target/release/spec-spine" scripts/verify-sweep.sh --repo "${TMPDIR:-/tmp}/ss112/repo" --trusted-ref main --exempt-file "${TMPDIR:-/tmp}/ss112/e-closed.txt" --out "${TMPDIR:-/tmp}/ss112/x" 2>&1 | grep -q 'closed ordinal 43'
# Only shrinks: an entry whose spec declares acceptance is a stale exemption.
printf '001-green\n' > "${TMPDIR:-/tmp}/ss112/e-stale.txt"
SPEC_SPINE_BIN="$PWD/target/release/spec-spine" scripts/verify-sweep.sh --repo "${TMPDIR:-/tmp}/ss112/repo" --trusted-ref main --exempt-file "${TMPDIR:-/tmp}/ss112/e-stale.txt" --out "${TMPDIR:-/tmp}/ss112/x" >/dev/null 2>&1; test $? -eq 3
SPEC_SPINE_BIN="$PWD/target/release/spec-spine" scripts/verify-sweep.sh --repo "${TMPDIR:-/tmp}/ss112/repo" --trusted-ref main --exempt-file "${TMPDIR:-/tmp}/ss112/e-stale.txt" --out "${TMPDIR:-/tmp}/ss112/x" 2>&1 | grep -q 'exemption is stale'
# Live: an entry naming no spec in the corpus accounts for nothing.
printf '030-gone\n' > "${TMPDIR:-/tmp}/ss112/e-dangling.txt"
SPEC_SPINE_BIN="$PWD/target/release/spec-spine" scripts/verify-sweep.sh --repo "${TMPDIR:-/tmp}/ss112/repo" --trusted-ref main --exempt-file "${TMPDIR:-/tmp}/ss112/e-dangling.txt" --out "${TMPDIR:-/tmp}/ss112/x" 2>&1 | grep -q 'names no spec in the corpus'
# Enumerated and closed: the ledger this repository actually ships carries no id
# at or above the closing ordinal, so no future spec can enter it (D-2). The
# count is deliberately not asserted: the ledger shrinks as the debt is retired,
# and a pinned size would refuse the retirement 3.4 exists to allow.
test -f scripts/verify-sweep.sh && ! grep -qE '^0(4[3-9]|[5-9][0-9])-|^[1-9][0-9][0-9]?-' scripts/verify-sweep.sh
grep -qE '^0[0-4][0-9]-' scripts/verify-sweep.sh
# And it holds against the real corpus: running the BUILT-IN ledger over a
# merged revision of this repository passes all four checks of 3.4 (closed,
# live, only-shrinking, enumerated) and reports a legacy spec as `exempt`
# rather than `not-declared`. A single `--only` keeps this to a second. This
# line needs `origin/main` present in the checkout, which a maintainer's clone
# has and a remote-less mirror does not; 3.1 makes that the only context this
# block runs in.
SPEC_SPINE_BIN="$PWD/target/release/spec-spine" scripts/verify-sweep.sh --rev origin/main --trusted-ref origin/main --only 011 --out "${TMPDIR:-/tmp}/ss112/builtin" >/dev/null 2>&1; test $? -eq 0
python3 -c "import json;d=json.load(open('${TMPDIR:-/tmp}/ss112/builtin/sweep.json'));assert d['ledgerOrigin']=='built into verify-sweep.sh';assert [(s['id'],s['outcome']) for s in d['specs']]==[('011-index-hash-slices','exempt')]"
# --- 3.7: the trust boundary is mechanical, and its override is visible ---
SPEC_SPINE_BIN="$PWD/target/release/spec-spine" scripts/verify-sweep.sh --repo "${TMPDIR:-/tmp}/ss112/repo" --rev side --trusted-ref main --exempt-file "${TMPDIR:-/tmp}/ss112/exempt.txt" --out "${TMPDIR:-/tmp}/ss112/x" >/dev/null 2>&1; test $? -eq 3
SPEC_SPINE_BIN="$PWD/target/release/spec-spine" scripts/verify-sweep.sh --repo "${TMPDIR:-/tmp}/ss112/repo" --rev side --trusted-ref main --exempt-file "${TMPDIR:-/tmp}/ss112/exempt.txt" --out "${TMPDIR:-/tmp}/ss112/x" 2>&1 | grep -q 'is not an ancestor of main'
SPEC_SPINE_BIN="$PWD/target/release/spec-spine" scripts/verify-sweep.sh --repo "${TMPDIR:-/tmp}/ss112/repo" --rev side --trusted-ref side --exempt-file "${TMPDIR:-/tmp}/ss112/exempt.txt" --out "${TMPDIR:-/tmp}/ss112/side" --timeout 5 >/dev/null 2>&1; test $? -eq 1
python3 -c "import json;d=json.load(open('${TMPDIR:-/tmp}/ss112/side/sweep.json'));assert d['trustedRef']=='side'"
# --- 3.6: the report can never be written inside the repository under test ---
SPEC_SPINE_BIN="$PWD/target/release/spec-spine" scripts/verify-sweep.sh --repo "${TMPDIR:-/tmp}/ss112/repo" --trusted-ref main --exempt-file "${TMPDIR:-/tmp}/ss112/exempt.txt" --out "${TMPDIR:-/tmp}/ss112/repo/inside" >/dev/null 2>&1; test $? -eq 3
test -d "${TMPDIR:-/tmp}/ss112/repo/specs" && test ! -e "${TMPDIR:-/tmp}/ss112/repo/inside"
# ... and it refuses before creating anything, including when the named parents
# do not exist yet (the case the `--out <repo>/inside` line above cannot reach,
# since its parent is the repository root and already exists).
SPEC_SPINE_BIN="$PWD/target/release/spec-spine" scripts/verify-sweep.sh --repo "${TMPDIR:-/tmp}/ss112/repo" --trusted-ref main --exempt-file "${TMPDIR:-/tmp}/ss112/exempt.txt" --out "${TMPDIR:-/tmp}/ss112/repo/newsub/run" >/dev/null 2>&1; test $? -eq 3
test ! -e "${TMPDIR:-/tmp}/ss112/repo/newsub"
# --- 3.6: the run directory is cleared before use, so it must be one the
# sweep is entitled to delete. A slipped --out at a directory holding anything
# else is refused and its contents survive.
mkdir -p "${TMPDIR:-/tmp}/ss112/precious" && touch "${TMPDIR:-/tmp}/ss112/precious/keepme"
SPEC_SPINE_BIN="$PWD/target/release/spec-spine" scripts/verify-sweep.sh --repo "${TMPDIR:-/tmp}/ss112/repo" --trusted-ref main --exempt-file "${TMPDIR:-/tmp}/ss112/exempt.txt" --out "${TMPDIR:-/tmp}/ss112/precious" >/dev/null 2>&1; test $? -eq 3
test -f "${TMPDIR:-/tmp}/ss112/precious/keepme"
# --- 3.1, 3.2: the sweep is a caller; it changes nothing about the tool ---
# It is not a subcommand, it is in no skill's gate floor, and it is on no
# pull-request leg. Spec 099 3.6 corrected the last assertion in place: since
# that spec the sweep runs from `.github/workflows/acceptance.yml`, on the
# default branch after a merge and on a schedule, and never on a pull request.
# The `test -f` guard is load-bearing: a bare `! grep` on a file that is not
# there passes, and would assert nothing if the workflow were deleted.
! target/release/spec-spine --help 2>&1 | grep -qE '^[[:space:]]+sweep'
! grep -rqF 'verify-sweep' .claude/skills/
! grep -qF 'verify-sweep' .github/workflows/ci.yml
test -f .github/workflows/acceptance.yml && ! grep -qE '^[[:space:]]*(pull_request|pull_request_target|merge_group):' .github/workflows/acceptance.yml
# It reads the corpus through the governed verbs only.
grep -qF 'registry list --ids-only' scripts/verify-sweep.sh
grep -qF 'verify "$1" --plan' scripts/verify-sweep.sh
test -f scripts/verify-sweep.sh && ! grep -qE '(jq|awk|sed)[^|]*\.derived' scripts/verify-sweep.sh
# --- 3.9: the runbook names the sweep and both of its triggers ---
grep -qF 'scripts/verify-sweep.sh' docs/releasing.md
grep -qF 'amends_verification' docs/releasing.md
rm -rf "${TMPDIR:-/tmp}/ss112"
```
