---
id: "094-one-gate-and-the-boundaries-it-holds"
title: "One gate, and the boundaries it holds"
status: draft
kind: "tooling"
created: "2026-09-20"
summary: >
  The governed loop has one definition and three boundaries where it is
  enforced: the commit, the pull request, and the merge. Five specs described
  that, each written against a copy the repository distributed. 020 put the
  merge driver in `.githooks/`; 064 wrote the composite gate as `kit/Makefile`
  and the workflow as `kit/govern.yml`; 089 fixed the guard that conflated a
  skip with a failure; 090 added the commit-boundary hook; 114 gave the target
  its controls and made the workflow call it instead of restating it. Spec 092
  removed the copies and moved the gate to the repository root. This spec is the
  gate, in one place: the target and its controls, the two CI legs that call it,
  the commit-boundary hook that refuses without repairing, and the merge driver
  that resolves a same-shard conflict by regenerating rather than by leaving
  markers.
implementation: pending
owner: "The spec-spine Authors"
risk: high
depends_on:
  - "005-coupling-gate"
  - "022-index-sharding"
  - "047-effective-config-is-a-governed-read"
  - "052-read-verbs-on-a-code-free-corpus"
  - "092-the-engine-ships-governance-not-an-environment"
establishes:
  - "Makefile"
  - { kind: directory, path: ".githooks/" }
  - ".github/workflows/ci.yml"
  - "crates/spec-spine-core/tests/gate.rs"
---

# 093: One gate, and the boundaries it holds

## 1. Purpose

### 1.1 One chain, five documents, and a copy in each

The governed loop is six commands in an order. Where that order was written down
mattered more than it sounds: `064-the-kit-ships-the-composite-gate`'s whole
argument is that a chain with two spellings drifts, and
`114-one-gate-definition-that-holds-on-a-code-free-corpus` D-1 records what
happened the one time a workflow
restated the chain beside the target instead of calling it (the pull-request leg
coupled, the push leg coupled too, and the header comment claimed one
definition while the file carried two).

Until spec 092 the definition lived in `kit/Makefile`, a file this repository
distributed and then ran on itself, with `kit/govern.yml` as the shipped
workflow. The kit is gone and the gate moved to the repository root. What is
left is one target, one workflow, one commit hook and one merge driver, still
described in five documents most of whose clauses name a deleted file.

### 1.2 Three boundaries, one definition

| Boundary | Who acts | What runs |
|---|---|---|
| commit | the committer | `.githooks/pre-commit`: refuses a stale or drifting tree, writes nothing |
| pull request | CI | `make gate` with the coupling half on, against frozen SHAs |
| merge | CI, and git | `make gate` with the coupling half off; the merge driver for a same-shard conflict |

Each reads the same definition. None restates it.

### 1.3 The five this replaces

- `020-derived-artifact-merge-driver`
- `064-the-kit-ships-the-composite-gate`
- `089-a-skip-and-a-failure-are-different-answers`
- `090-a-hook-bound-to-a-tool-route-misses-the-work`
- `114-one-gate-definition-that-holds-on-a-code-free-corpus`

They are not named in a `supersedes` edge: the edge takes a spec id, the corpus
refuses an id it cannot resolve (`L-004`), and after the collapse these
documents are in git rather than in `specs/`. `docs/corpus-map.md` is the map.

## 2. Territory

| Path | What |
|---|---|
| `Makefile` | the one definition of the governed loop |
| `.github/workflows/ci.yml` | the two legs that call it, plus the stack gate |
| `.githooks/` | the commit-boundary hook, the merge driver, the two registrars |
| `crates/spec-spine-core/tests/gate.rs` | what holds all of it to this spec |

`.gitattributes` is not claimed: it is on the coupling bypass floor and is the
binding, not the driver. §5.3 asserts its content anyway.

## 3. Behavior: the one definition

### 3.1 The target

The root `Makefile` MUST carry `gate`, `refresh`, `verify`, `build`, `test`,
`fmt`, `clippy` and `help`.

`gate` is read-only throughout: it runs `check`, `lint`, conditionally
`index coverage`, and conditionally `couple`, and it MUST NOT run a writing
verb. A gate that writes repairs what it is meant to judge. `refresh` is the
writing half, for a session that can commit what it regenerates.

`verify SPEC=<id>` runs one spec's declared acceptance and is deliberately not
part of `gate`: it executes what the corpus declares.

### 3.2 The language targets are guarded on a manifest, and the guard is an `if`

`test`, `build`, `fmt` and `clippy` MUST be guarded on a **file probe**
(`test -f Cargo.toml`, `test -f package.json`), never on a command probe. A tree
with `cargo` installed and no manifest is the specify-first case, which three of
four governed repositories are in, and probing for the tool answers the wrong
question.

Each guard MUST be written as an explicit `if … then … else … fi`. It MUST NOT
be `test -f M && cmd || echo skipping`: `&&`/`||` is not if/else, the `||` branch
fires when **either** the probe is false **or** the command fails, so on a
repository that has the manifest a failing command exits 0 having printed a
false skip. That makes a CI job running `make build test fmt clippy` unfailable,
which is the worst shape a gate can have: the repository looks defended and is
not.

### 3.3 The ownership assertion is conditional, and every skip is announced

Whether `gate` runs `index coverage --fail-on-untraced` MUST be decided by the
`OWNERSHIP` variable:

- `auto` (default): ask the repository's own **effective configuration**, through
  `spec-spine config show`, for `[coupling] require_ownership`, so a corpus
  relying on the default gets the default's answer rather than a missing key;
- `1`: run it whatever the configuration says;
- `0`: do not.

Any other value MUST be refused (exit 3), never read as `auto`. A caller writing
`OWNERSHIP=yes` is asking for the assertion, and falling through to the
configuration would hand them a config-governed run under a word they chose to
override it with.

When the assertion does not run, the target MUST print one line saying so and
saying that whole-tree ownership was **not** verified. A silent skip recreates,
one layer up, the vacuous pass spec 052 removed from the verb: a green step named
for an assertion that never ran.

**A failed read is not a skip.** The probe MUST capture the governed read, check
its status, and only then look at the text. `config show | grep -q` reports
grep's status and discards the read's, so a binary too old to have the verb, an
unparsable `spec-spine.toml`, or a config key the binary rejects would every one
of them read as "ownership is off" and produce a green gate. If the read exits
non-zero the target MUST fail with that status; if it succeeds and names neither
spelling of the setting, the target MUST refuse rather than guess.

### 3.4 The coupling step has explicit controls

`gate` MUST accept:

- `COUPLE`, defaulting to `1`, deciding whether the coupling step runs. `0`
  skips it, announced like every other skip. `1` and `0` are the only values;
  anything else is refused rather than read as "not 0, so couple". A control
  whose typo means the opposite of what was typed is not a control.
- `PR_BODY`, a path to a file holding the pull-request body, passed to `couple`
  as `--pr-body` only when set. The path MUST reach `couple` as one argument, so
  a body file under a directory with a space in its name is one path.
- `BASE` and `HEAD`, the refs the coupling gate diffs. `BASE` defaults to the
  resolved default branch on its remote: `$SPEC_SPINE_DEFAULT_BRANCH`, then the
  remote's own `HEAD`, then `main`, with `?=` so an explicit assignment still
  wins. `HEAD` defaults to `HEAD`.

`COUPLE` MUST NOT be inferred from whether `PR_BODY` is set. The two answer
different questions: whether to couple is about the event, whether a waiver is
reachable is about the body. GitHub permits an empty description, so a
body-shaped inference would turn every description-less pull request into a
coupling gate that quietly did not run.

### 3.5 The gate holds on a corpus with no code

`make gate` MUST exit 0 on a freshly initialized corpus, after `compile` and
`index` have run once, with no source file and no package manifest anywhere in
the tree. §3.2 and §3.3 are what make that true: the language targets no-op and
the ownership assertion is off by configuration, both announced.

## 4. Behavior: the boundaries

### 4.1 The pull-request and push legs call the target

`.github/workflows/ci.yml` MUST invoke the `gate` target from **both** event
legs and MUST NOT restate the chain beside it:

- pull request: `COUPLE=1`, with `BASE` and `HEAD` set to the event's **frozen**
  `base.sha` and `head.sha`, and `PR_BODY` naming a file the step wrote under
  `$RUNNER_TEMP`. Frozen SHAs because the checked-out `refs/pull/N/merge` HEAD
  re-resolves against the base on every run, so a gate diffing to it folds in
  changes merged after the pull request opened and reports them as this pull
  request's drift.
- push and merge queue: `COUPLE=0`. The change has already merged, so there is
  nothing left to refuse, and there is no body, so a `Spec-Drift-Waiver:` line is
  unrecoverable; it lives in the pull-request description, not in the squash
  commit message.

The environment variable holding the body's **text** MUST NOT be named
`PR_BODY`: make imports the environment, `PR_BODY` names a **path**, and a step
exporting the text under that name would hand the gate a path whose value is the
body.

The body MUST reach the gate through a **file**. A body carrying a waiver line
has no safe shell quoting.

The workflow MUST keep `ci-gate` as the single aggregating required check, and
MUST keep the coupling half off the `merge_group` event.

### 4.2 The commit boundary refuses; it does not repair

`.githooks/pre-commit` MUST NOT run a `spec-spine` subcommand that writes into
the repository and MUST NOT stage a file. A hook that regenerated the derived
tree and added it would silently enlarge the commit the committer wrote, and a
commit is the one boundary where what is being recorded is exactly what was
staged.

It MUST: resolve the repository from `git rev-parse --show-toplevel` and exit 0
silently when there is no corpus; resolve the binary as `$SPEC_SPINE_BIN`, then
`<root>/target/release/spec-spine`, then `PATH`, printing a skip and exiting 0
when none is found; read the configured derived directory through the governed
read rather than assuming the default; and refuse when the committed derived
tree is stale.

It MUST NOT run `couple`. `couple` builds its diff from a range of commits,
which cannot contain the change being committed, so a coupling verdict here
would be a verdict about the previous commit wearing this one's name.

The escape is the committer's: the hook MUST name `git commit --no-verify` in
its refusal. Registration is per clone and opt-in, through
`.githooks/enable-hooks.sh`, which MUST say how to undo it.

### 4.3 The merge boundary regenerates rather than marking

`.githooks/merge-derived-index.sh` is a git low-level merge driver invoked as
`<driver> %O %A %B %P`. On a conflict in a committed shard it MUST regenerate the
shard trees from the merged working tree and hand the fresh bytes back to git,
rather than leaving conflict markers in a compiled artifact.

Registration is per clone and opt-in, through
`.githooks/enable-merge-driver.sh`. The driver is **inert** until registered, and
it MUST NOT be presented as a replacement for the staleness gate: since spec 022
the shards are per unit, so two pull requests touching different specs write
disjoint files and never conflict at all; the driver is for the rare same-shard
case.

`.gitattributes` MUST bind the driver to the shard globs of the **configured**
derived directory. A binding that still named the default after a relocation
registers the driver on files nothing writes, which is exactly the conflict the
driver exists to prevent, arriving unhandled.

## 5. The tests

`crates/spec-spine-core/tests/gate.rs` MUST assert, against the checked-in files:

1. every `spec-spine` invocation in `Makefile` and `ci.yml` names a verb this
   binary has, and every invocation in the `gate` target is a read-only form;
2. the chain in `gate` is the chain `AGENTS.md` lists, in order;
3. §3.2's manifest probe and explicit `if`, statically **and** by running the
   shipped guarded lines in both states: manifest absent (a skip, exit 0) and
   manifest present with a failing command (a failure, non-zero);
4. §3.3's captured read, including that the probe is not a pipeline;
5. §4.1's two legs, over the workflow's **executable** steps parsed as YAML, not
   over its text: each leg invokes the target, no step restates a verb the target
   runs, and the `PR_BODY` value names a file that same step writes;
6. §4.3's binding, derived from the effective configuration rather than restated.

The step reader MUST model shell quoting: a separator inside a quoted string, or
behind a backslash, does not divide one command into two, and a `>` inside a
quoted string is not a redirection. It MUST refuse a construct it does not
model, and the test MUST fail on a refusal rather than reporting the commands it
managed to find: an invocation hidden inside a construct the reader stepped over
is reported as a step that invokes nothing, which is a false answer pointing the
wrong way. Refusing an unmodelled construct includes refusing an unmodelled
**operand**: on an output-descriptor duplication (`>&`) the reader MUST accept
only a run of ASCII digits or a bare `-`, and MUST refuse a missing or otherwise
shaped operand.

The negative cases MUST be built as fixtures in the test itself, so the detector
is shown capable of refusing rather than assumed to be: the target's name in a
comment, in a step's `name:`, and in text the script echoes, each MUST NOT count
as an invocation.

## 6. What the five said that is not carried here

| Dropped | Its subject |
|---|---|
| 064 §3.1, §3.2, §3.3 as written | `kit/Makefile`, `kit/govern.yml`, `kit/.githooks/`, `kit/README.md` |
| 064 §3.4's "this repository adopts the kit's Makefile" | there is no kit to adopt from; the file is the repository's |
| 089 §3.3 | the embedded copy in `kit_embedded.rs` |
| 090 §3.6 | `kit/.githooks/pre-commit` |
| 114 §3.4's `govern.yml` clauses | the shipped workflow; the requirements move to `ci.yml` unchanged |
| 114 §3.5, §3.6 | the acceptance replacement for what is now spec 064, which spec 092 §3.12 holds |
| 020's merge-queue enablement prose | an administrative act, done once, not a requirement on a file |

Nothing about the gate's strength is relaxed. Every `MUST` above is one of the
five's, with the file renamed where spec 092 moved it.

## 7. Out of scope

- **The Claude Code hooks and the skills.** Spec 093. The boundary is the actor.
- **What `check`, `couple` and `index coverage` decide.** Their own specs; this
  spec governs only the order they run in and the conditions they run under.
- **Whether the merge queue is enabled.** A repository-administration setting.
- **The workflow governance projection.** Spec 060 owns how a workflow bump
  folds into the ledger; this spec owns the workflow as a caller of the gate.

## 8. Resolved decisions

D-1 (2026-09-20, the definition is a `Makefile`, not a composite CLI verb). A
`spec-spine gate` verb was considered and refused for the reason
`064-the-kit-ships-the-composite-gate` gives: the
chain's steps differ between a local session (which may write) and CI (which may
not), and the variability lives in the caller. A make target is where a caller's
variability belongs, and it stays runnable by a human with no wrapper.

D-2 (2026-09-20, `.githooks/` is the gate's, not the harness's). Spec 093 D-3
draws the line at the actor. A pre-commit hook and a merge driver are run by
whoever commits and whoever merges, not by a session reading instructions, and
both exist to enforce the same chain this spec defines.

D-4 (2026-09-20, an assertion over a message does not read the path inside it).
The hook messages interpolate the repository they judged, and the PR gate's
exit-3 assertion is that the word `stale` is **absent**. The sweep names its
per-spec temporary directory after the spec id, and this corpus holds
`...-is-not-a-stale-shard`, so the word arrived in the path and the assertion
failed for a reason that has nothing to do with the hook. It passed everywhere
else, which is the whole character of the defect: an absence assertion over a
string the test only partly chooses is green until the part it does not choose
changes. The path is replaced by a token before the message is returned, so what
is asserted is what the hook wrote. The same shape is worth watching wherever a
test asserts that something is missing from output it did not fully author.

D-3 (2026-09-20, the workflow's two legs are asserted over parsed steps). Spec
094 D-1 and D-17 are carried forward whole: a text-level count of `make gate`
passes against a file whose header comment claims one definition while the
pull-request leg restates the chain, and a quoting-blind reader miscounts a
command inside an echoed string. The reader is the assertion.

## Verification

Each line is one command, run independently.

**Fail-first evidence**, measured on 2026-09-20 at the parent of this branch:
`registry show 093` is red (not found, exit 1); the five predecessor directories
exist and none reports a `supersededBy`.

```verify:cli
cargo build --release --locked
# 2: the territory, and that this spec owns it.
test -f Makefile
test -f .github/workflows/ci.yml
test -d .githooks
test -f crates/spec-spine-core/tests/gate.rs
target/release/spec-spine index owner Makefile > "${TMPDIR:-/tmp}/ss094-own.txt"
grep -qF '094-one-gate-and-the-boundaries-it-holds' "${TMPDIR:-/tmp}/ss094-own.txt"
rm -f "${TMPDIR:-/tmp}/ss094-own.txt"
# 3 and 5: the whole of it, asserted against the checked-in files.
cargo test -p spec-spine-core --test gate --locked > "${TMPDIR:-/tmp}/ss093-g.txt" 2>&1
grep -qE 'test result: ok\. [1-9][0-9]* passed' "${TMPDIR:-/tmp}/ss093-g.txt"
rm -f "${TMPDIR:-/tmp}/ss093-g.txt"
# 3.1: the target runs, read-only, on this repository.
make gate SPEC_SPINE=target/release/spec-spine COUPLE=0
# 3.3 and 3.4: an unrecognised control word is refused, not read as the default.
! make gate SPEC_SPINE=target/release/spec-spine OWNERSHIP=yes COUPLE=0
! make gate SPEC_SPINE=target/release/spec-spine COUPLE=maybe
# 3.4: the four controls exist by name in the one definition.
grep -qE '^OWNERSHIP' Makefile
grep -qE '^COUPLE' Makefile
grep -qE '^PR_BODY' Makefile
grep -qE '^HEAD' Makefile
grep -qE '^BASE' Makefile
# 4.1: the workflow calls the target and spells the flag nowhere else.
grep -qF 'make gate' .github/workflows/ci.yml
grep -qF 'RUNNER_TEMP' .github/workflows/ci.yml
! grep -qF 'PR_BODY:' .github/workflows/ci.yml
grep -qF -- '--pr-body' Makefile
# 4.2: the commit boundary writes nothing and names the committer's escape.
! grep -qE '^[^#]*git add' .githooks/pre-commit
grep -qF -- '--no-verify' .githooks/pre-commit
! grep -qE '"\$sc" .*(compile|index)( |$)' .githooks/pre-commit
# 4.3: the driver is registered on the configured shard globs and nowhere else.
grep -qF 'merge=spec-spine-derived-regen' .gitattributes
! grep -qE '^\.derived/.*merge=' .gitattributes
test -x .githooks/enable-merge-driver.sh
test -x .githooks/enable-hooks.sh
# 1.3 and D-2: all five predecessors are named here, and every one of them is
# in the map, which is the answer a `supersedes` edge would have given.
test "$(grep -cE '^- `[0-9]{3}-' specs/094-one-gate-and-the-boundaries-it-holds/spec.md)" = 5
sh -c 'miss=0; for id in $(grep -oE "^- .[0-9]{3}-[a-z0-9-]+" specs/094-one-gate-and-the-boundaries-it-holds/spec.md | sed "s/^- .//"); do grep -qF "$id" docs/corpus-map.md || { echo "not in the map: $id"; miss=1; }; done; exit $miss'
# And no part of the gate still RUNS anything from the tree spec 092 removed.
# Read over executable lines only: the `Makefile` header says where this file
# used to live, and a history sentence is not an invocation (1.1).
! grep -qE '^[^#]*kit/' Makefile
! grep -rqE '^[^#]*kit/' .githooks/
! grep -qE '^[^#]*kit/' .github/workflows/ci.yml
```
