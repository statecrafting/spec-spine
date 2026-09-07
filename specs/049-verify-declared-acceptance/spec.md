---
id: "049-verify-declared-acceptance"
title: "`spec-spine verify <id>`: run a spec's declared acceptance"
status: approved
kind: "tooling"
created: "2026-09-06"
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "016-short-id-resolution"
  - "037-machine-readable-verdicts"
  - "048-kit-ships-the-governed-loop-skills"
establishes:
  - "crates/spec-spine-types/src/verify.rs"
  - "crates/spec-spine-core/src/verify.rs"
  - "crates/spec-spine-core/tests/verify.rs"
  - "crates/spec-spine-cli/src/cmd_verify.rs"
extends:
  - { spec: "001-compile-registry", unit: "crates/spec-spine-core/src/lib.rs", nature: additive }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-cli/src/main.rs", nature: additive }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-cli/tests/cli.rs", nature: additive }
  - { spec: "000-spec-spine-bootstrap", unit: "crates/spec-spine-types/src/lib.rs", nature: additive }
  # The new verb constant, the schema-version bump it implies, and the pin test.
  - { spec: "037-machine-readable-verdicts", unit: "crates/spec-spine-types/src/verdict.rs", nature: additive }
  - { spec: "000-spec-spine-bootstrap", unit: "crates/spec-spine-types/src/version.rs", nature: additive }
  - { spec: "000-spec-spine-bootstrap", unit: "crates/spec-spine-types/tests/dtos.rs", nature: additive }
references:
  - { unit: { kind: file, path: "docs/design/03-adopter-audit-2026-09.md" }, role: context }
  - { unit: { kind: file, path: "specs/043-governance-document-gaps/spec.md" }, role: context }
  - { unit: { kind: file, path: "scripts/verify-spec.sh" }, role: prior-art }
summary: >
  A spec's `## Verification` section states what would make its claims true, and
  nothing runs it. Three adopters independently wrote the same 78-line
  `scripts/verify-spec.sh` to close that gap, and spec 048 added a fourth copy
  to this repository, so the kit now ships the workaround it exists to retire.
  Spec 043 4 named this "the most substantial thing the adoption invented and
  the one most worth having" and deferred filing it. This is that filing.
  `spec-spine verify <id>` reads the spec's `## Verification` section, runs each
  line of its `verify:cli` fences from the repository root in order, and stops
  at the first failure. Two things make it a spec-spine verb rather than a
  script: the parse becomes a typed, tested read of authored markdown that the
  library owns, and the outcome becomes a spec 037 verdict envelope instead of
  a sentence a caller must string-match. The engine never spawns a process. Core
  returns a `VerifyPlan`, the commands it found and the fence tags it declined,
  and the CLI executes it, which is the same seam `couple` already uses for
  `git`. Two behaviours diverge deliberately from the ported script, both to
  keep an older contract intact: a missing spec exits 1, not the script's 2,
  because 2 means stale; and a failing command exits 1 with its own code carried
  in the payload, rather than propagating a code the exit contract does not
  define. `not-declared` stays an honest zero and is distinguishable from a pass
  in the payload, never by parsing prose.
---

# 049: `spec-spine verify <id>`

## 1. Purpose

A spec ends with `## Verification`: the conditions under which its claims hold.
In this corpus that section is prose in 44 of 49 specs and executable in two,
and in neither case does the tool read it. Acceptance is therefore asserted by
whoever wrote the spec and confirmed by whoever reviews it, which is exactly the
arrangement the rest of this project refuses. The registry does not trust a
spec's claim about its territory; it compiles it. The gate does not trust a
PR's claim that code matches its spec; it computes the drift. Verification is
the one claim still taken on the author's word.

### 1.1 The adopters closed the gap, four times

`scripts/verify-spec.sh` is 78 lines of `awk` and `sh`, and it exists verbatim
in hqgit, aicortex, rahi, and, since spec 048, here. Spec 048 3 shipped it into
`kit/scripts/` deliberately, so that an adopter taking the kit gets a working
`/verify`, and 048 4 recorded that doing so put the workaround inside the
artifact meant to replace it.

Four copies of one parser is the specific failure this project was built to
name. The section grammar (a numbered or unnumbered heading, a fenced block
with a tag, comment and blank lines inside it) is authored truth being parsed
ad hoc, which constitution I and II together put on the wrong side of the line:
the parse of authored markdown belongs to the compiler, and a consumer reads
its typed answer.

### 1.2 What a verb buys that a script does not

The script works. Three arguments make the verb worth more than the 78 lines it
retires, and none of them is "it is in Rust".

**The parse gets tested.** `verify-spec.sh` has no tests in any of the four
repositories. Its heading regex, its fence handling, and its comment-stripping
are asserted by use. As a core module the same grammar is a pure function of one
string, and 3.2's table becomes acceptance fixtures.

**The outcome gets an envelope.** Spec 037 gave every adjudicating verb a
machine-readable verdict precisely so that a programmatic consumer stops
string-matching sentences. `verify` is the verb an orchestrator most needs to
read programmatically, since it gates a merge, and it is the one still emitting
`verify: 044-...: passed (6 command(s))` for a caller to parse.

**`not-declared` stops being ambiguous.** The script prints a distinct sentence
and exits 0, so a caller that reads only the exit code cannot tell "every
command passed" from "there was nothing to run". That distinction decides
whether a spec's acceptance is evidence or absence, and it currently survives
only in prose.

## 2. Territory

Four new files: the DTOs (`types/src/verify.rs`), the parser
(`core/src/verify.rs`) with its fixtures (`core/tests/verify.rs`), and the
executing command (`cli/src/cmd_verify.rs`).

Additive edits to the facade (`core/src/lib.rs`), the CLI's command enum
(`cli/src/main.rs`) and its end-to-end tests (`cli/tests/cli.rs`), the types
re-export (`types/src/lib.rs`), the verb constants (`types/src/verdict.rs`), and
the verdict schema version with its pin test (`types/src/version.rs`,
`types/tests/dtos.rs`).

No registry or index DTO changes; neither schema version moves and no committed
shard changes shape. `verify` reads `specs/<id>/spec.md` and nothing under the
derived tree.

## 3. Behavior

### 3.1 The seam: core parses, the CLI executes

The engine MUST NOT spawn a process. `core::verify::plan` MUST be a pure
function of `(Config, file contents)` returning a `VerifyPlan`: the commands it
found, in order, and the fence tags it declined. The CLI MUST be what runs them.

This is not a stylistic preference. It is the same seam spec 005 already draws
for `git`: the library never shells out, the CLI parses `git diff` and passes a
typed `DiffInput` in. Holding the line here keeps the whole engine, including
this verb's grammar, usable from a binding that has no shell, and keeps the
part worth testing (the parse) testable without a subprocess.

A caller of the library therefore gets the plan and decides what to do with it.
Deciding to run it is a decision about executing code, and the library declines
to make that decision on a caller's behalf.

### 3.2 The grammar

Ported from `scripts/verify-spec.sh`, whose semantics this table fixes. Where a
row is a change, it says so.

| input | behavior |
|---|---|
| `## Verification` heading | the section, to the next `## ` |
| a numbered heading (`## 5.` and the same word) | the same section |
| no such heading | `not-declared` |
| a ` ```verify:cli ` fence | each body line is a command, in order |
| blank line in a fence | skipped, not a command |
| line whose first non-space character is `#` | skipped, a comment |
| leading and trailing whitespace on a command | trimmed |
| multiple `verify:cli` fences | concatenated, in document order |
| ` ```verify:browser ` or any other tag | counted by tag, reported, not run |
| an untagged fence | prose; neither run nor counted as declined |
| section present, no `verify:cli` command | `not-declared` |

`<id>` MUST accept the short form spec 016 defines, so `spec-spine verify 049`
resolves as `registry show 049` does. The script required the full directory
name. A short id matching two specs MUST be refused rather than guessed.

### 3.3 Outcomes and exit codes

| outcome | exit | meaning |
|---|---|---|
| `passed` | 0 | at least one command ran; all exited 0 |
| `not-declared` | 0 | nothing to run; an honest zero, not a pass |
| `failed` | 1 | a command exited non-zero; later commands did not run |
| no such spec | 1 | `Error::NotFound` |

Two rows diverge from the ported script, both because spec-spine's exit codes
are an older contract than this verb.

**A missing spec exits 1, not 2.** The script exits 2 for a bad id. In this
tool 2 means *stale*, and a caller in the gate chain branches on it. Reusing it
for "no such spec" would make `verify` the one verb where 2 does not mean what
it means everywhere else. `Error::NotFound` already maps to 1.

**A failing command exits 1, and its own code goes in the payload.** The script
propagates the command's exit code, so a command killed by a signal makes
`verify` exit 137, outside the documented `0`/`1`/`2`/`3` set. That is the same
class of defect spec 035 fixed for a broken pipe. The failing command's exit
code MUST be reported in the payload as `failure.exitCode`; it MUST NOT be the
process's exit code.

Neither divergence loses information. Both keep it in the typed payload rather
than in a channel that only has four defined values.

### 3.4 The report

`report` under `--json` (spec 037, verb `verify`) and the prose form MUST carry
the same facts:

- `specId`, resolved from the argument.
- `declared`: whether any `verify:cli` command was found.
- `outcome`: `passed`, `not-declared` or `failed`.
- `ran` and `total`: how many commands ran, and how many the plan held. They
  differ only on a failure.
- `skipped`: the declined fence tags, each with its count.
- `failure`, when the outcome is `failed`: the 1-based `index`, the `command`
  string, and its `exitCode`.

`declared: false` with `outcome: "not-declared"` is the pair a consumer reads to
tell absence from success without matching a sentence, which is 1.2's third
argument made mechanical.

### 3.5 Execution

The CLI MUST run each command through `sh -c`, with the repository root as the
working directory, in plan order, stopping at the first non-zero exit. It MUST
echo each command and its exit code, so that a CI log reads as a transcript.

The child inherits the environment. `verify` is a developer and CI command, not
a sandbox, and pretending otherwise by filtering the environment would break
every command that needs `PATH`, `CARGO_HOME`, or a toolchain variable.

### 3.6 `verify` executes what the corpus declares

Stated normatively because it is the one genuinely sharp edge this verb adds.
`spec-spine verify` runs commands written in a markdown file. Every other verb
in this tool reads. This one, by design, executes.

That is safe in the setting it is built for, where the corpus and the person
running the command are the same trust domain, and it is exactly as safe as the
script it replaces. It is not safe on a corpus from an untrusted source, and it
MUST NOT be added to the gate chain (`compile`, `index`, `lint`, `couple`),
which runs on PR branches whose contents are, in the general case, a stranger's.
Running acceptance against a merged sha is an orchestrator's decision, made
after review; this verb serves it and does not make it.

The documentation MUST state this. A verb that executes untrusted input without
saying so is a defect whatever its behavior.

### 3.7 A spec that verifies itself is refused, not run

`verify` MUST refuse to run a spec whose verification is already in progress,
reporting `R-001`, and MUST pass the in-progress ids to every command it spawns
so that a child sees them.

This is not hypothetical. The first draft of this spec carried
`spec-spine verify 049` inside its own `## Verification` block, as the natural
way to assert that the envelope names this verb. Running it forked 350 processes
before it was killed by hand: the command runs the block, which runs the
command. Nothing in 3.2's grammar forbids the line, and it is an easy one to
write.

The failure mode is what earns a refusal rather than a documentation note. Every
other authoring mistake in a verification block produces a wrong answer, which
review catches. This one produces unbounded process growth on the machine that
ran it, and no output at all.

The stack crosses a process boundary, so it travels in an environment variable
(`SPEC_SPINE_VERIFY_STACK`) set on each child, the same mechanism `couple`
already uses for `SPEC_SPINE_PR_BODY`. It is set per child rather than on the
running process, so the verb mutates no global state.

**Decision, 2026-09-06.** The refusal is scoped to *re-entry*, not to
self-reference in general: a spec 048 block may run `verify 049`, and any depth
that does not repeat an id is allowed. Only a cycle is refused, because only a
cycle fails to terminate. Mutual recursion between two specs is caught by the
same rule, since the stack holds every id rather than only the outermost.

### 3.8 `--plan` reads without running

`verify --plan` MUST print the commands it would run, in plan order, and run
none of them, exiting 0. With `--json` it MUST emit the `VerifyPlan` in the
spec 037 envelope.

For the one verb that executes what the corpus declares, being able to ask what
it would execute before letting it is a safety affordance rather than a
convenience, and it is the natural CLI surface for the pure function 3.1 keeps
in the engine. It is also how a reviewer inspects a `## Verification` block
contributed by someone else without running it.

## 4. Out of scope

**Retiring `scripts/verify-spec.sh` and its kit copy.** The audit's shape for
this item ends with both scripts retired and `/verify` wrapping the verb, and
this spec deliberately stops one step short. Removing the script from `kit/`
strands every adopter whose pinned `spec-spine` predates the release carrying
this verb, and all four adopters currently run two to four releases behind. The
retirement is a follow-on filed once the verb ships, and it will need an
`amends` edge on spec 048, which established the script, rather than a silent
deletion of another spec's unit. Nothing here edits 048.

**Rewriting the `/verify` skill.** Same reason. The skill keeps calling the
script until the retirement spec moves it, so an adopter's harness works on
either side of the upgrade.

**Verifying that code matches a spec's prose.** Unchanged from spec 041 4 and
044 4: no gate does this, and this verb does not either. It runs what the author
declared, which tests the author's own stated conditions and nothing beyond
them.

**A `verify:browser` driver.** The tag is counted and reported so a caller knows
work was declined. Driving a browser is an orchestrator's stage, and putting it
in a deterministic engine would import a whole runtime this project does not
have.

**`verify --all`.** A corpus-wide run is a loop in a caller, and what it should
do about `not-declared` specs (44 of 50 here) is a policy this spec has no
grounds to pick.

**Any change to the gate chain or a committed artifact.** 3.6 forbids the first;
the second follows from `verify` reading only `specs/<id>/spec.md`.

## 5. Verification

```verify:cli
# The block is self-contained: the commands below invoke the release binary,
# and `cargo test` builds only debug artifacts, so it is built first. An
# orchestrator runs this in a clean checkout of the merged sha.
cargo build --release --locked
cargo test -p spec-spine-core --test verify --locked
cargo test -p spec-spine-types --test dtos --locked
cargo test -p spec-spine-cli --locked
# not-declared is an honest zero: 044 declares acceptance in prose only.
target/release/spec-spine verify 044-in-progress-is-in-flight
test "$(target/release/spec-spine verify 044 --json | python3 -c 'import json,sys; print(json.load(sys.stdin)["report"]["outcome"])')" = "not-declared"
# The short id resolves, and the envelope names this verb (spec 037).
test "$(target/release/spec-spine verify 044 --json | python3 -c 'import json,sys; print(json.load(sys.stdin)["verb"])')" = "verify"
# A missing spec is 1 (not found), never 2 (stale).
target/release/spec-spine verify 999-no-such-spec; test $? -eq 1
# 3.7, demonstrated on this very block: re-entry is refused, so this line
# terminates instead of forking without bound.
target/release/spec-spine verify 049; test $? -eq 1
```
