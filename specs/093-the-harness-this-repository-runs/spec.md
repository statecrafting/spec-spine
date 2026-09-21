---
id: "093-the-harness-this-repository-runs"
title: "The harness this repository runs"
status: approved
kind: "harness"
created: "2026-09-20"
summary: >
  Sixteen specs governed one tree. `.claude/` is four hooks, ten skills, four
  agents and four rules, and the requirements on them were spread over specs 093,
  047, 048, 051, 063, 068, 071, 072, 078, 080, 081, 082, 099, 104, 110 and 116,
  each written when that tree was a product this repository distributed.
  Spec 092 removed the distribution; what is left is one tree that one repository
  reads, described in sixteen places, most of them half about copies that no
  longer exist. This spec is that description, in one place: what the hooks may
  do, what the push gate refuses, how every hook reads a verdict, what the ten
  skills are and what each must contain, what the four rules say, and who owns
  the protocol. Every surviving requirement of the seventeen is carried here;
  what is dropped is named in §6 with the reason, and in every case the reason is
  that its subject was a copy spec 092 deleted.
implementation: complete
owner: "The spec-spine Authors"
risk: high
depends_on:
  - "005-coupling-gate"
  - "043-verify-declared-acceptance"
  - "092-the-engine-ships-governance-not-an-environment"
establishes:
  # The tree a session here loads, and the protocol it reads first.
  - "AGENTS.md"
  - { kind: directory, path: ".claude/skills/" }
  - { kind: directory, path: ".claude/agents/" }
  - ".claude/settings.json"
  # What holds it to the above.
  - "crates/spec-spine-core/tests/harness_hooks.rs"
  - "crates/spec-spine-core/tests/harness_skills.rs"
---

# 093: The harness this repository runs

## 1. Purpose

### 1.1 One tree, sixteen owners

`.claude/` is 19 files: four hook bodies in one `settings.json`, ten
`SKILL.md`, four agent briefs and four rules. Until spec 092 it was also a
product: the same files sat in `kit/`, were embedded in the binary, were
projected into `.agents/` and `.codex/`, and were held equal by three parity
suites. Sixteen specs governed that arrangement, and most of each one is about
the copies.

With the copies gone, what those sixteen say about the tree that remains is
scattered across sixteen documents, roughly half of whose clauses name a file
that no longer exists. A reader who wants to know what the push gate must refuse
reads 071, then 072 for the branch name, then 110 for the message, and each of
the three tells them something about `kit/settings.json` on the way.

### 1.2 What this spec is

The whole of it, once. Every clause below is a surviving requirement of one of
the sixteen, carried forward with its reason. Nothing is relaxed: §6 lists
what was dropped, and each entry is a clause whose only subject was a deleted
copy. The historical documents are in git; this is what the corpus holds now.

### 1.3 The sixteen this replaces

- `046-kit-hooks-read-never-write`
- `047-harness-rules-name-the-legitimate-edits`
- `048-kit-ships-the-governed-loop-skills`
- `051-harness-runs-the-verbs-it-ships`
- `063-a-stale-binary-is-not-a-stale-ledger`
- `068-a-path-scoped-rule-example`
- `071-a-tag-push-is-not-a-push-to-main`
- `072-the-default-branch-is-configured-not-assumed`
- `078-the-protocol-has-an-owner`
- `080-a-gate-that-cannot-ask-says-so`
- `081-the-kit-ships-what-the-loop-calls`
- `082-a-refusal-is-not-a-remediation-round`
- `099-the-session-hooks-report-the-verdict`
- `104-every-hook-reads-the-code-the-same-way`
- `110-a-refusal-names-the-branch-it-resolved`
- `116-shepherd-reads-every-reviewer`

They are not named in a `supersedes` edge: the edge takes a spec id, the corpus
refuses an id it cannot resolve (`L-004`), and after spec 095's collapse these
documents are in git rather than in `specs/`. The map from each of them to this
spec is `docs/corpus-map.md`, which spec 095 establishes for exactly this reason.

## 2. Territory

| Path | What |
|---|---|
| `AGENTS.md` | the cross-agent protocol, the four standing rules, the gate list, the project layer |
| `.claude/settings.json` | four hook bodies and the destructive-command refusals |
| `.claude/skills/` | the ten loop skills |
| `.claude/agents/` | the four agent briefs |
| `crates/spec-spine-core/tests/harness_hooks.rs` | the hook bodies, run as programs |
| `crates/spec-spine-core/tests/harness_skills.rs` | the skills, agents and rules |

`.githooks/`, the root `Makefile` and `.github/workflows/` are the gate's, spec
094's. The boundary is the actor: a Claude Code hook and a skill are read by a
session, a git hook and a make target are run by whoever commits or builds.

## 3. Behavior: the hooks

### 3.1 Hooks read; they do not repair

No hook body in `.claude/settings.json` MUST invoke a writing `spec-spine`
subcommand: not a bare `compile`, not a bare `index`, not `attest`. Freshness is
read with `check` (or the primitives' read forms, `compile --check` and
`index check`).

The reason is not style. A hook fires in a context that cannot commit what it
regenerates, so a writing hook leaves the derived tree dirty; an orchestrator
that refuses to start on a dirty tree then never starts, and one adopter's
pipeline stalled for eleven hours on dirt it had produced itself.

### 3.2 The one sanctioned write

Exactly one hook may write, and it writes outside the repository: the session
banner may create its own scratch file under the configured `state_dir` or the
system temporary directory. Nothing may write inside the repository.

### 3.3 A hook acts on the repository the action targets

An action hook (`PreToolUse`) MUST resolve the repository from the **command it
is judging**, not from the session's project directory: an explicit `cd <dir>`
prefix, else the hook's own `cwd`, else `git rev-parse --show-toplevel`. A
multi-repository session pushes and opens pull requests in whichever tree the
command names, and a gate that judged the session's project would refuse the
wrong repository's state.

### 3.4 A hook that skips says so

Every hook that cannot perform its read MUST print that it skipped and why, and
exit 0. A silent skip is a gate that looks green; §3.9 is the same rule for the
verdict-reading hooks, and spec 052's refusal of a vacuous coverage pass is the
same reasoning one layer down.

### 3.5 The push gate: the match is anchored on the invocation

The `PreToolUse` gate MUST match only a command that **runs** `git push`: at the
start of the command, or immediately after a `&&` or `;` separator. It MUST NOT
match a command that merely contains the text in an argument, a heredoc body or
a quoted pattern. The unanchored form refused the greps, heredocs and test
fixtures that describe the gate, in a session the gate governs.

### 3.6 The push gate: the protected branch is resolved, never assumed

The gate MUST resolve the protected branch for the repository the command acts
on, highest wins:

1. `$SPEC_SPINE_DEFAULT_BRANCH`;
2. `git -C "$root" symbolic-ref --short refs/remotes/origin/HEAD`, minus the
   `origin/` prefix (a clone sets it, so most repositories need no configuration);
3. `main`, as a compatibility floor.

Resolution MUST NOT depend on the `spec-spine` binary. The push half runs on git
alone, which is why it still protects a repository where the binary is absent or
setup has not been run; making the branch a governed read would trade that away
for a fact git already holds. Resolution MUST fall back rather than refuse: step
3 always answers, so the gate reaches a verdict for every repository, and a
repository whose default is `main` MUST behave exactly as it did before the name
was resolved rather than written.

Every comparison MUST use the resolved value: the refspec forms
(`origin <branch>`, `HEAD:<branch>`, `:<branch>`, and the force-with-plus form),
which are therefore **built from the variable** rather than written as literal
patterns; the test on the current branch; the test on the trailing positional
argument, alongside `HEAD`; and the refusal message.

Each expansion inside a `case` pattern MUST be quoted. `$SPEC_SPINE_DEFAULT_BRANCH`
is user-supplied and git's refname rules never see it, so an unquoted `ma*n`
would silently turn each refspec test into a wildcard.

### 3.7 The push gate: only what would update that branch

On the protected branch, a push MUST be refused when it would update that
branch, and only then. Counting the positional arguments after the anchored
verb, with flags dropped: fewer than two means no explicit refspec, so the push
follows the current branch and is refused; a trailing `HEAD` or the resolved
name is refused; anything else is allowed. A tag push carries its own refspec,
updates no branch, and is what a maintainer runs from the default branch
immediately after a release merges.

The arguments counted MUST be those following the **anchored** invocation, not
those following the first appearance of the verb anywhere in the command: a
command may name the verb in an argument before it ever runs one, and reading
those words as a refspec produces a nonsense verdict.

A command chaining **more than one** push MUST be refused on the protected
branch, whatever its arguments say: only the anchored push is walked, and a walk
cannot speak for a push it never examined. Off that branch the chain is allowed,
because the refspec check still sees the whole command.

The refusal message MUST say what was actually refused rather than restating the
rule, MUST **name the branch it resolved**, and MUST name the tag push as
allowed, so a maintainer can tell a correct refusal from a misfire. The literal
`main` MUST NOT appear in the message where the resolved name belongs.

### 3.8 The PR gate reads `check`'s exit code

The `PreToolUse` gate on `gh pr create` MUST branch on the **exit status** of
`spec-spine check`, never on its truth value:

- **0**: both trees fresh; proceed to `couple`.
- **2**: see §3.9. When the binary carries the verb this is staleness, and the
  message names which tree and the `compile` / `index` remedy.
- **1**: the corpus does not validate, or an unresolved claim was refused. The
  message MUST say the corpus failed validation and MUST NOT call the tree
  stale: staleness is not meaningful against a corpus that does not compile.
- **3**: the read was not performed. The message MUST say so, MUST name the
  binary and what it answers to `--version`, and MUST NOT name a remedy.

Every branch except `0` MUST still refuse. A gate whose check did not run is not
green.

The gate MUST then refuse when the derived tree is not committed (§3.13), and
MUST NOT stage, commit or regenerate anything to fix it.

### 3.9 Exit 2 is the one ambiguous code, so it is probed

This tool spends exit 2 on staleness and clap spends it on an unrecognised
subcommand, so a binary predating the `check` verb answers 2 without having read
either tree. On exit 2 and **only** on exit 2, a verdict-reading hook MUST run
`"$sc" check --help` before printing:

- probe succeeds: the binary carries the verb, so 2 is staleness and the message
  is unchanged, wording included;
- probe fails: the binary never read either tree. The hook MUST say the read was
  not performed, MUST name the binary and its `--version` answer, MUST name the
  version floor the verb needs, and MUST NOT say "stale" or name a remedy that
  repairs nothing.

`--version` MUST NOT be asked on the happy path. Exit 0 and exit 2 are answers;
the version read exists to qualify a non-answer, and these hooks run on every
response.

### 3.10 The session hooks report the verdict, not a guess

The `Stop` hook MUST capture `check`'s combined output **and** its status, probe
per §3.9, and branch:

- **0**: print nothing.
- **2**: read the report back rather than assume which refusal it is. On
  `spec-registry: STALE` or `codebase-index: STALE`, print the staleness line
  unchanged, with the sentence explaining why the hook does not regenerate. On
  `codebase-index: UNRESOLVED CLAIM`, say that a spec claims a unit that does not
  resolve, that this is not staleness, that regenerating does not clear it
  (the diagnostic is recomputed from the corpus every run), and name
  `index diagnostics` as the read that lists them. A report matching neither MUST
  be reported as unrecognised and MUST NOT be called fresh.
- **1**: the corpus does not validate; regenerating does not clear it.
- **3**: the read was not performed; name the binary and its version.

Every branch MUST exit 0: the hook advises and never refuses. A session that has
ended cannot act on a refusal.

The `SessionStart` banner MUST report the same four verdicts in the same
vocabulary, including the unresolved-claim case and an exit 3 reported as a read
that was not performed for **both** trees. The `unknown (check exit N)` fallback
MUST remain for a code neither hook recognises: it is the right answer to an
unexpected shape and the wrong answer to one the verb documents.

### 3.11 The destructive-command refusals

`.claude/settings.json` MUST carry a permission `deny` list refusing
`cargo publish`, `npm publish`, `gh release create`, `git push --force` and the
destructive `rm -rf` forms over the corpus and the derived tree. The `allow`
list is machine-local convenience and is not governed.

### 3.12 The hooks resolve the binary the project declares

Every hook that invokes `spec-spine` MUST resolve it in this order, first that
exists: `$SPEC_SPINE_BIN`, then `<root>/target/release/spec-spine`, then `PATH`.
A repository that builds its own binary must be governed by the one it builds;
the `PATH` fallback keeps an installed CLI working.

### 3.13 The derived-tree question is asked in every state git distinguishes

§3.8's refusal is only as good as the question underneath it, and the question
the gate asked was `git diff --quiet -- <derived>/`. That compares the **index**
to the **working tree**: it is silent about anything staged and it never
mentions an untracked file. Reproduced against plain git, in a repository where
a shard is committed:

```
$ printf 'b\n' > <derived>/x.json && git add <derived>/x.json
$ git diff --quiet -- <derived>/ ; echo $?
0
$ printf 'c\n' > <derived>/new.json          # untracked
$ git diff --quiet -- <derived>/ ; echo $?
0
```

Neither blind spot is a corner. **Staged** is what `git add` leaves behind, and
`git add` was the first half of the remedy the refusal itself printed, so the
message produced the state the test could not see. **Untracked** is what a new
spec's shards are before anyone adds them, one per authority unit in `by-spec/`
and `by-package/`, which is the shape of most pull requests this repository
opens. The consequence is not a missed warning: `gh pr create` proceeds, the
branch is pushed without the shards, and CI fails on a stale tree, which is the
failure the hook exists to move to the second it takes to read a `git` exit
code.

The gate MUST therefore refuse when the derived tree differs from `HEAD` in any
of the three states git distinguishes: unstaged changes to tracked files, staged
changes, and untracked files under the derived directory. Ignored files are not
part of the question; `build-meta.json` is gitignored by design, and a gate
refusing on it would refuse every clone.

Each state MUST be read **independently**, and `git diff HEAD` is not the
instrument for the first two. It **cancels**: a shard edited, staged, and then
restored in the working tree to the bytes `HEAD` already holds leaves
`git diff HEAD` empty while both `git diff --cached` and `git diff` report the
file, so a gate asking that one question calls a half-finished refresh clean.
And one `HEAD`-relative comparison cannot say **which** state it found, which
the message has to name because the remedies differ. So the reads are
`git diff --cached --name-only`, `git diff --name-only` and
`git ls-files --others --exclude-standard`, each scoped to the derived
directory; `--exclude-standard` is what keeps the gitignored build metadata out
of the third and MUST NOT be dropped.

The refusal MUST name each state it found, and a tree carrying two of them MUST
name both rather than the first. It MUST print the paths. "Uncommitted" covers
all three and tells a reader nothing about which command to run next: the remedy
for a staged shard is `git commit`, and for an untracked one it is `git add`
first. This is §3.8's rule for `check`'s exit codes, applied to the one question
the gate asks git rather than the tool.

The derived directory MUST be the configured one, read as the tool's own typed
answer (`config show --json`) rather than spelled into the hook, for spec 094
§3.2's reason: `[layout] derived_dir` is configurable, and a hook hard-coding
the default silently stops gating a repository that changed it. A configuration
the binary could not answer is **not** evidence of a dirty tree: the hook
announces the skip (§3.4) and does not manufacture a refusal out of a read that
did not happen.

The gate stays read-only. It MUST NOT stage, commit or regenerate anything; §3.1
is unchanged and so is its reason.

## 4. Behavior: the skills, agents and rules

### 4.1 Ten skills

`.claude/skills/` MUST hold exactly these ten, the loop in the order
"Working the backlog" runs it, then the two it calls:

`prime`, `setup`, `next`, `build`, `verify`, `ship`, `shepherd`, `spec`,
`commit`, `code-review`.

The session skill is `prime`, never `init`: Claude Code ships its own `/init`,
which writes a CLAUDE.md, and shadowing a built-in while inverting its meaning
is the defect. No alias is kept.

### 4.2 The frontmatter contract

Every skill MUST declare `name` (equal to its directory), `description` and
`allowed-tools`; a skill taking an argument declares `argument-hint`. Tool names
MUST be the current ones (`Agent`, never `Task`). A rule file a skill teaches an
agent to write uses `paths:` frontmatter, never `globs:` or `imports:`, which
Claude Code does not read.

### 4.3 Repository-invariant, with a project layer

Every skill MUST be free of project nouns, adopter names, composite target
names, placeholder build commands, hardcoded temporary directories and pre-024
artifact paths, and MUST end with a `## Project layer` section naming what it
reads from `AGENTS.md` (the binary invocation, the version pin, the gate list,
the stack gate, the default branch) and from `spec-spine.toml` (closed enums,
extra keys, `state_dir`).

### 4.4 A read skill reads

`prime`, `next`, `verify` and `code-review` MUST NOT invoke a writing
`spec-spine` verb. Freshness is read with `check`; a stale verdict is reported
and repaired later, as committed work.

### 4.5 One gate list, and a test that pins it

`AGENTS.md`'s "Run the gate before every commit" block MUST list every command
CI enforces, with its flags. `harness_skills.rs` MUST assert that the governance
floor each skill inlines is a **subset** of that list: a skill telling its reader
to run a command the authority does not list, or an authority dropping a command
the skills run, MUST fail the build.

The commands MUST stay runnable. A placeholder such as `origin/<default>` would
substitute a broken command for a wrong one; the base ref is written as the
resolution §3.6 performs.

### 4.6 `verify` wraps the verb

The `verify` skill MUST invoke `spec-spine verify <id>` through the binary
invocation `AGENTS.md` names, MUST NOT invoke `scripts/verify-spec.sh`, MUST
report the verb's outcomes in the verb's own words, and MUST describe `--plan` as
the way to read a `## Verification` block before running it. When the configured
binary does not carry the verb, the skill MUST say so and name the upgrade rather
than silently running a second implementation.

### 4.7 `next` and `build` apply the lifecycle rules

`next` MUST apply two rules on top of `registry plan`: a spec whose `status` is
not `approved` is listed as awaiting approval and never offered (approval is a
human act), and a spec at `implementation: in-progress` is listed as in flight
and never offered as new work. `build` MUST refuse a missing argument, an unmet
dependency and a dirty tree, and MUST refuse committing to the default branch.

### 4.8 `shepherd` classifies before it edits

`shepherd` MUST classify every failing required check **before** it edits any
file, and record the classification in its report. The classes are CRITICAL,
HIGH, MEDIUM, LOW. Classification reads the run log, which the skill already
fetches; it MUST NOT require a local gate run first, because the failing evidence
is the run's.

A **CRITICAL** finding MUST stop the skill: it MUST NOT be remediated, MUST NOT
consume one of the two remediation rounds, and MUST be reported with its evidence
and a proposed remedy for a human. CRITICAL is exactly: a coupling refusal whose
only remedy is a waiver or an edit to a spec the session is not implementing; a
change to an approved spec's requirements; a governance or security control being
weakened; and a failure whose class the session cannot determine.

Within a round, `shepherd` MUST fix in descending severity, and MUST re-run the
gate **to completion** locally before pushing, so a round is not spent
discovering the next link in a chain that stopped early.

### 4.9 `shepherd` reads every reviewer, on every path

Step 3b MUST read all three of `pulls/<n>/comments`, `issues/<n>/comments` and
`pulls/<n>/reviews`, and MUST say which endpoint each thread came from.

Each read MUST be **paginated** (`--paginate`): `gh api` returns one page by
default, and the reviews endpoint is the one most likely to exceed it. Each
read's **exit status** MUST be checked before its output is believed, and it MUST
be the status of the command performing the read, not of a downstream pipeline
stage: a failed call prints nothing and is otherwise indistinguishable from an
endpoint with no threads. Each read MUST land in a defined shape (`--slurp` with
`--paginate`, so the document's top level is an outer array of pages) and MUST be
parsed before anything is claimed about it, inspecting every item of every page
and counting **feedback items**, not outer pages.

A failed read permits **one** retry of that endpoint. If the retry also fails the
run MUST stop before the merge checkpoint, name the endpoint, preserve the
command, status and stderr, and report which reads did succeed.

An all-green pull request MUST go **through** the thread read before the merge
checkpoint, not around it.

The report's thread line MUST distinguish "no threads exist" from "threads were
not read". `none` MUST mean all three reads succeeded and returned no feedback;
otherwise the value MUST be `could not read <endpoint>`.

A comment from an automated reviewer is a reviewer's comment and is triaged by
§4.8 like any other. Reading it is not agreeing with it: a design disagreement
is a human's to settle. Reading, classifying, retrying and rejecting a finding
consume **no** round.

### 4.10 The four rules

The four standing rules are `AGENTS.md`'s `## Rules` section, one `###`
subsection each (D-4). `.claude/rules/` MUST NOT exist: a rule that binds every
agent does not belong in one agent's harness directory.

**Governed artifact reads** (unconditional). Derived artifacts are read only
through `spec-spine` subcommands, never by ad-hoc `jq`, `grep`, `python`, `awk`
or `sed` over the JSON. It MUST state that parsing the **output** of a subcommand
is a typed read and is allowed, and why: the tool has already deserialized the
shards and answers in a contract it versions.

**Adversarial prompt refusal** (unconditional). It MUST name the two edits
always legitimate for the spec a session is implementing (claiming a file the
session created, and recording a dated decision the spec was silent on), MUST say
that changing what a spec requires is never the session's to do mid-build, MUST
point at the `extends` edge as the way to touch another spec's unit, and MUST say
that a `Spec-Drift-Waiver` is a human instrument an agent never writes on its own
authority.

**Orchestrator rules** (unconditional). It MUST say that regenerated shards
are committed with the change that made them stale, and why, and MUST state "one
session, one spec" pointing at `AGENTS.md`.

**Derived artifacts are compiler output**. It MUST name the **configured**
derived directory as its scope, in its own text, and it MUST say that it does
not replace **Governed artifact reads** and cannot: the mistake that rule
prevents is reaching for `jq` **instead of** the subcommand, and a session that
does so may never touch a path the scope matches. The scope is now prose rather
than `paths:` frontmatter, because frontmatter scoping is a loading mechanism of
one agent's harness and the rule outlived it.

### 4.11 The four agents

`.claude/agents/` MUST hold `architect`, `explorer`, `implementer` and
`reviewer`. None may contain U+2014. The reviewer brief MUST name the legitimate
mid-build edits §4.10 defines, so a review does not report one as drift.

### 4.12 The protocol has an owner

`AGENTS.md` is established by this spec, as a bare `file` unit: its headings are
edited as a document, and per-heading claims would multiply the coupling surface
without telling anyone anything. The file MUST carry a line naming the spec that
governs it, by path, so a rename breaks it visibly. That line MUST NOT be a
`// Spec:` header: that grammar is defined for `SOURCE_EXTS`, markdown is not
among them, and a markdown dialect of it would be a second claim mechanism the
indexer does not implement.

`AGENTS.md` MUST schedule `spec-spine --version` among its parallel startup
reads and MUST say that no exit code below is meaningful until it has answered.
Where `[meta] required_version` is set the CLI checks it on every run and the
manual step is unnecessary, and the document MUST say so.

## 5. The tests

`harness_hooks.rs` MUST read the hook bodies out of `.claude/settings.json`
rather than restating them, and MUST:

- assert all four events are present and each invokes `spec-spine` at least once;
- refuse a mutating invocation in any hook (§3.1) and any write into the
  repository (§3.2);
- assert the binary resolution order in every hook that invokes it (§3.12);
- run the **shipped** `PreToolUse` body as a program against a throwaway
  repository, feeding it a synthesized payload and reading its exit code, over a
  matrix covering §3.5's anchoring, §3.7's three argument cases and the refspec
  forms, on a repository whose default branch is `main` **and** on one whose
  default branch is not, plus a `$SPEC_SPINE_DEFAULT_BRANCH` row;
- run the shipped `PreToolUse`, `Stop` and `SessionStart` bodies against a
  stand-in binary whose `check` exit code, `check --help` exit code and output
  the test chooses, asserting §3.8, §3.9 and §3.10 verdict by verdict, including
  that the exit-3 message names the binary and does **not** say "stale";
- run the shipped `PreToolUse` body against a scratch repository whose derived
  tree is in each state of §3.13, asserting the **verdict** rather than the
  source: a committed tree passes; an unstaged, a staged and an untracked shard
  are each refused and each named; a staged edit cancelled by a working-tree
  restoration to `HEAD` is refused (the tree where `git diff HEAD` is empty, so
  the case must assert that emptiness or it stops being that case); two states
  at once name both; a gitignored `build-meta.json` alone passes; and, in a
  repository whose `derived_dir` is not the default, a change under the
  configured path is refused while one under `.derived/` is not. A grep for the
  new commands would pass against a body that still let a staged shard through
  on another branch of the same `if`, which is why the verdict is what is
  asserted, and the configured-path pair is the only case a hook spelling both
  paths cannot satisfy.

`harness_skills.rs` MUST assert §4.1's set, §4.2's frontmatter, §4.3's
invariance and project layer, §4.4's read-only forms, §4.5's subset property,
§4.8's four classes and four CRITICAL rows, §4.9's three endpoints with
`--paginate` and `--slurp` and the three distinct thread values, §4.10's four rule sections
and the scoped one's self-limiting clause, and §4.11's agents.

A filter-based run MUST assert a **non-zero** pass count: a name filter matching
nothing exits 0, so the bare invocation would stay green while asserting nothing.

## 6. What the sixteen said that is not carried here

Each of these had exactly one subject and spec 092 deleted it. Nothing about the
tree that remains is relaxed.

| Dropped | Its subject |
|---|---|
| 047 §3.4, "one text, three homes" | the `scaffold.rs` constants and `kit/.claude/rules/` |
| 048 §3.1's fifteen, §3.5's script, §3.7's wiring | `kit/.claude/skills/`, `kit/scripts/`, the kit README |
| 051 §3.1's last clause, §3.2, §3.7 | `kit/AGENTS.md`, `kit/README.md`, `/validate-and-fix` |
| 063 §3.3 | `kit/settings.json` |
| 068 §3.1's config-aware glob, §3.3, §3.4's parity | `init --with-kit`, the kit README, the kit copy |
| 071 §3.3, 072 §3.4, 080 §3.3, 081 §3.3, 099 §3.4, 104 §3.3, 110 §3.3 | assertions over `kit/`, `.codex/`, `.agents/` and `kit_embedded.rs` |
| 082 §3.5, 116 §3.5 | the two-tree and four-copy equality assertions |
| 068 §3.1's `paths:` frontmatter requirement | a loading mechanism of one agent's harness, dropped with the directory by D-4 |

One clause moved rather than dropped: 072 §3.3's `Makefile` base-ref derivation
is spec 094's, with the rest of the gate. Spec 094, the commit-boundary git
hook, is superseded there rather than here, for D-3's reason.

## 7. Out of scope

- **Whether `.claude/` should exist here at all.** Spec 092 §3.5 holds it until
  Statecraft's global delivery concretely replaces it. This spec describes what
  it must contain while it does.
- **The gate.** Spec 094.
- **The engine verbs the hooks call.** `check`, `couple` and their exit codes are
  the engine's specs; this spec governs only how a hook reads them.
- **The `Stop` hook refusing rather than advising.** It advises; a session that
  has ended cannot act on a refusal.
- **A `spec-spine` verb answering "is the derived tree committed".** It is the
  better shape: a tested verb with one definition, and no `git` in a shell
  script embedded in JSON. It is also new CLI surface with its own exit-code
  contract, and the core may not shell out, so the verb would take its answer
  from the CLI layer the way `couple` takes a `DiffInput`. That is a spec, not
  a clause of §3.13. When it exists the hook calls it, and §3.13's matrix is
  what proves the replacement kept the three answers. D-5.

## 8. Resolved decisions

D-1 (2026-09-20, consolidation rather than sixteen amendments). The
alternative was to amend each of the sixteen in place, which is sixteen
documents each still opening with a page about a tree that no longer exists, and
a reader still visiting three of them to learn what the push gate refuses. The
owner authorized a one-time collapse; spec 095 records the authority and the map.

D-2 (2026-09-20, the predecessors are named in prose, not in `supersedes`). A
`supersedes` edge naming a spec the corpus no longer holds is an `L-004` warning,
and `lint --fail-on-warn` is in the gate; relaxing that lint to accommodate a
one-time collapse would trade a standing check for a convenience. §1.3 names all
sixteen, and `docs/corpus-map.md` is the machine-readable answer to "where did
071 go", which is the question the edge would have answered.

D-3 (2026-09-20, the split with spec 094 is by actor, not by file type). A
Claude Code hook and a skill are read by a session; a git hook, a make target and
a CI job are run by whoever commits, builds or opens the pull request. That line
puts `.githooks/pre-commit` in 093 even though it is a hook, and puts
`AGENTS.md` here even though the gate list lives in it, because the gate list is
read by a session and executed by `make`.

D-5 (2026-09-21, the derived-tree question is repaired here rather than in a
successor spec). §3.13's defect was measured on 2026-09-17 and drafted as a
spec of its own against four copies of the hook; spec 092 deleted three of
them, withdrew the draft, and recorded the defect as open against the copy that
survives. That copy is this spec's territory, so the requirement belongs in
this document: filing a separate spec to change one clause of a hook this spec
establishes adds a document to a corpus that has just been compressed, and
leaves a reader of §3.8 with a refusal whose question is specified elsewhere.
The owner authorized the direct edit on 2026-09-21. What is added is a
requirement the gate did not meet, not a relaxation of one it did: the
measurement, the three reads, the cancellation case and the matrix are carried
over from the withdrawn draft intact, which is why they are stated here in full
rather than cited to a branch.

D-4 (2026-09-20, the four rules move into `AGENTS.md` and `.claude/rules/` is
deleted). `.claude/` is the harness of one agent. Three of these four rules bind
every agent, are cited by 18 approved specs, and were loaded by Claude Code's
own auto-discovery of `.claude/rules/*.md` and by step 0 of the protocol, which
read them by path. Only the first of those two paths is Claude-specific, and it
is the one a fold into the cross-agent protocol gives up: a session now meets
them at `/prime` rather than at startup. What it buys is that the rules stop
depending on a directory this repository intends to remove, and the citations in
the corpus point at a document that is staying. The fourth rule loses its
`paths:` scoping with the directory; its own text already argued the
unconditional rule is the one doing the work, and it now says its scope in
prose. The citation rewrite that came with the fold is four literal full-path
strings, which is why it was safe to do by hand where spec 095's ordinal
rewrites were not.

## Verification

Each line is one command, run independently.

**Fail-first evidence**, measured on 2026-09-20 at the parent of this branch:
`registry show 092` is red (not found, exit 1); `registry show 046` reports
`approved` with no `supersededBy`; the sixteen directories exist; and
`harness_skills.rs` carries no assertion naming this spec.

**Fail-first evidence for §3.13**, measured on 2026-09-21 against the hook body
this section replaces: the four structural lines below are red (the old body is
what `! grep -qF 'diff --quiet -- .statecraft/derived/'` matches, and neither
per-state read is in it), and the matrix run is red **seven cases of eight**,
the eighth being `the_pr_gate_ignores_the_gitignored_build_metadata`, which is a
guard against over-refusing and passes on both sides by construction. The
`! grep -qF 'diff HEAD'` line is likewise green on both sides: it guards against
the *drafted* fix rather than against the old body, because a single
HEAD-relative comparison satisfies every other line here and is the mechanism
the cancellation case rules out.

```verify:cli
cargo build --release --locked
# 2: the territory exists and is this spec's.
test -f AGENTS.md
test -d .claude/skills
test -d .claude/agents
test -f .claude/settings.json
test -f crates/spec-spine-core/tests/harness_hooks.rs
test -f crates/spec-spine-core/tests/harness_skills.rs
target/release/spec-spine index owner AGENTS.md > "${TMPDIR:-/tmp}/ss092-own.txt"
grep -qF '093-the-harness-this-repository-runs' "${TMPDIR:-/tmp}/ss092-own.txt"
rm -f "${TMPDIR:-/tmp}/ss092-own.txt"
# 3 and 5: the hook bodies, run as programs, over the whole matrix.
cargo test -p spec-spine-core --test harness_hooks --locked > "${TMPDIR:-/tmp}/ss092-h.txt" 2>&1
grep -qE 'test result: ok\. [1-9][0-9]* passed' "${TMPDIR:-/tmp}/ss092-h.txt"
rm -f "${TMPDIR:-/tmp}/ss092-h.txt"
# 4 and 5: the skills, agents and rules.
cargo test -p spec-spine-core --test harness_skills --locked > "${TMPDIR:-/tmp}/ss092-s.txt" 2>&1
grep -qE 'test result: ok\. [1-9][0-9]* passed' "${TMPDIR:-/tmp}/ss092-s.txt"
rm -f "${TMPDIR:-/tmp}/ss092-s.txt"
# 3.13: the derived-tree question, in each state git distinguishes. The matrix
# is what decides it, under its own filter and with a non-zero pass count, so a
# name matching nothing cannot pass for a run.
cargo test -p spec-spine-core --test harness_hooks --locked pr_gate_ > "${TMPDIR:-/tmp}/ss092-d.txt" 2>&1
grep -qE 'test result: ok\. [1-9][0-9]* passed' "${TMPDIR:-/tmp}/ss092-d.txt"
rm -f "${TMPDIR:-/tmp}/ss092-d.txt"
# 3.13: the index-versus-worktree form is gone rather than joined by the others.
# Two branches of one `if` can coexist, and a body that kept the old test on
# another arm would satisfy the matrix's name and still ship the blind spot.
! grep -qF 'diff --quiet -- .statecraft/derived/' .claude/settings.json
# 3.13: and the HEAD-relative form is not what replaced it (D-5's cancellation).
! grep -qF 'diff HEAD' .claude/settings.json
grep -qF 'diff --cached --name-only' .claude/settings.json
grep -qF 'ls-files --others --exclude-standard' .claude/settings.json
# 4.1: ten skills, and the session skill is not the shadowed name.
test "$(ls -1 .claude/skills | wc -l | tr -d ' ')" = 10
test -f .claude/skills/prime/SKILL.md
! test -e .claude/skills/init
# 4.10: the four rules are sections of the protocol, and the directory is gone.
! test -e .claude/rules
grep -qF '### Governed artifact reads' AGENTS.md
grep -qF '### Adversarial prompt refusal' AGENTS.md
grep -qF '### Orchestrator rules' AGENTS.md
grep -qF '### Derived artifacts are compiler output' AGENTS.md
# 4.12: the protocol names its owner, by path, and not as a claim header.
grep -qF 'specs/093-the-harness-this-repository-runs/spec.md' AGENTS.md
! grep -qE '^// Spec:' AGENTS.md
# 1.3 and D-2: all sixteen predecessors are named here, and every one of them
# is in the map, which is the answer the `supersedes` edge would have given.
test "$(grep -cE '^- `[0-9]{3}-' specs/093-the-harness-this-repository-runs/spec.md)" = 16
sh -c 'miss=0; for id in $(grep -oE "^- .[0-9]{3}-[a-z0-9-]+" specs/093-the-harness-this-repository-runs/spec.md | sed "s/^- .//"); do grep -qF "$id" docs/corpus-map.md || { echo "not in the map: $id"; miss=1; }; done; exit $miss'
# The governed loop, over the corpus this spec is part of.
target/release/spec-spine check --fail-on-unresolved --fail-on-warn
target/release/spec-spine lint --fail-on-warn
```
