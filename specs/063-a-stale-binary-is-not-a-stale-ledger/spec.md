---
id: "063-a-stale-binary-is-not-a-stale-ledger"
title: "A stale binary is not a stale ledger"
status: draft
kind: "tooling"
created: "2026-09-07"
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "029-claude-code-skill-kit"
  - "031-registry-freshness-check"
  - "062-a-version-pin-the-cli-can-check"
extends:
  - { spec: "001-compile-registry", unit: "crates/spec-spine-cli/src/main.rs", nature: additive }
  - { spec: "029-claude-code-skill-kit", unit: "kit/settings.json", nature: additive }
  - { spec: "029-claude-code-skill-kit", unit: "kit/AGENTS.md", nature: additive }
  - { spec: "029-claude-code-skill-kit", unit: "AGENTS.md", nature: additive }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-cli/tests/cli.rs", nature: additive }
  # This repository runs the hooks it ships (spec 051), so the kit change is
  # mirrored here, and the read-only predicate learns `--version` (3.3).
  - { spec: "051-harness-runs-the-verbs-it-ships", unit: ".claude/settings.json", nature: additive }
  - { spec: "046-kit-hooks-read-never-write", unit: "crates/spec-spine-core/tests/kit_hooks.rs", nature: additive }
references:
  - { unit: { kind: file, path: "docs/design/03-adopter-audit-2026-09.md" }, role: context }
summary: >
  `spec-spine compile --check` exits 2 when the committed registry is stale. A
  binary predating `--check` rejects the unknown flag and clap exits 2 as well.
  Two unrelated conditions, one code, distinguishable only by matching English
  on stderr, and two adopters plus the kit plus this repository's own AGENTS.md
  carry that matching ritual. This spec maps every command-line usage error to
  exit 3, so a new binary can never report a usage failure as staleness, and
  replaces the stderr ritual with the precondition that actually settles it: ask
  the binary its version first, which every binary ever released answers. The
  ritual's real defect was that it made the caller parse prose to recover a fact
  the process had already destroyed by choosing the wrong exit code.
---

# 063: A stale binary is not a stale ledger

## 1. Purpose

The exit codes are a stable contract, mapped in exactly one place: `0` ok, `1`
validation failure or not found or drift, `2` stale, `3` I/O or parse or schema
or config. Every verb honours it, and `Error::exit_code()` is the single
mapping.

Clap is not in that mapping. `Cli::parse()` handles its own errors, and clap's
default for a usage error is exit **2**. So:

```
$ spec-spine compile --check      # binary predating spec 031
error: unexpected argument '--check' found
$ echo $?
2
```

which is the code `2` means stale. Two unrelated conditions, one number.

The cost is written down in four places. This repository's own `AGENTS.md`
carries the disambiguation under a heading called **Stale binary**: "the two
separate only by stderr: a rejection says `error: unexpected argument
'--check'`, while a real report names shards". `kit/AGENTS.md` carries it for
every adopter. Two adopters carry their own copy. The kit's `SessionStart` hook
maps exit 2 to "STALE, run spec-spine compile and commit the shards", which is
the wrong instruction when the real problem is an old binary, and which the hook
cannot detect because it looked at the code.

That ritual is not a workaround for a hard problem. It is a caller parsing
English to recover a fact the process had in hand and destroyed by choosing the
wrong number.

This is item 17 of the adopter audit's ranked backlog for the tool, and it is
last on that list for a reason: it is the smallest of them. It is also the one
that has been silently teaching every reader of `AGENTS.md` that exit codes in
this system are not quite trustworthy.

## 2. Territory

| Unit | Owner | What changes |
|---|---|---|
| `crates/spec-spine-cli/src/main.rs` | 001 | usage errors map to 3 |
| `crates/spec-spine-cli/tests/cli.rs` | 001 | the exit-code acceptance |
| `AGENTS.md` | 029 | the ritual becomes a version precondition |
| `kit/AGENTS.md` | 029 | the same, for adopters |
| `kit/settings.json` | 029 | the hook stops reporting 2 as staleness |

Spec 001 owns the CLI and its exit-code mapping. It requires that the codes mean
what the table says; mapping a case that currently violates the table into the
table is holding 001 to its own contract, not changing it.

## 3. Behavior

### 3.1 A usage error is exit 3

`main` MUST use `Cli::try_parse()` and map a clap error to the exit code the
contract gives it:

- `DisplayHelp` and `DisplayVersion` keep exit `0` and keep writing to stdout.
  They are successful requests for information.
- Every other clap error exits **3**, writing clap's message unchanged.

**Decision, 2026-09-07: an incomplete invocation is a usage error, not a
request for information.** The bullet above originally grouped
`DisplayHelpOnMissingArgumentOrSubcommand` with help and version, and said the
three "keep" exit 0. Measured against the shipped binary, none of them kept
anything: a bare `spec-spine` exits **2** today, exactly like every other usage
error, because clap's default is 2 and that is the whole complaint this spec is
about.

Following the letter would therefore have made a broken invocation newly
**succeed**: a script that dropped its verb would exit 0 while doing nothing.
That is a worse failure than the one being fixed, and it is the opposite of this
spec's subject. `--help` and `--version` are asked for; a missing subcommand is
not, so it exits 3 with its siblings. Nothing regresses to zero.

`3` is the right cell rather than a new code. It already means "the tool could
not proceed for a reason that is not the corpus": I/O, parse, schema, config. An
unparseable command line is the invocation's parse failure, and it belongs with
the others. Adding a fourth code would extend a contract that four repositories
and two package shims document, in order to distinguish a case none of them
needs distinguished from `3`.

After this, exit `2` from any verb means staleness and nothing else. That is the
whole point, and it is a property callers can rely on going forward.

### 3.2 The transitional answer is the version, not the stderr

Changing the code fixes new binaries. It cannot fix the case that causes the
problem, because the binary that reports the wrong code is by definition the old
one. Any correct procedure for a caller that may be talking to an old binary
must therefore not depend on the new behavior at all.

`AGENTS.md` and `kit/AGENTS.md` MUST replace the stderr-matching ritual with the
precondition that settles it:

> Ask `spec-spine --version` before believing any exit code. Every binary ever
> released answers it, and it exits 0. If the version predates the flag you are
> about to pass, rebuild or reinstall; do not interpret the exit code of a flag
> the binary does not have.

This is strictly better than matching stderr. It is one call, it works against
every version including ones that predate every flag, and it fails in the
direction of asking rather than guessing. The stderr text it replaces is clap's,
which this project does not control and which changes between clap releases: the
ritual was pinned to a dependency's message format.

Spec 062's `[meta] required_version` is the automated form of the same
precondition, and where a repository sets it, the check happens on every run and
the manual step is unnecessary. This spec's documentation change MUST say so and
point at it, so an adopter reads the manual procedure as the fallback it is.

### 3.3 The kit's hook stops asserting staleness

`kit/settings.json`'s `SessionStart` hook maps `compile --check`'s exit `2` to
"STALE, run spec-spine compile and commit the shards".

It MUST first establish that the binary understands the flag, and report an
out-of-date binary as an out-of-date binary:

```
[session-freshness] spec registry: spec-spine 0.11.0 predates `compile --check`,
                    rebuild or reinstall (see /setup)
```

The hook already resolves a binary through `spec_spine_bin` and already
branches on the exit code, so this is one more branch in a function that exists.
It remains read-only, which spec 046 requires of every kit hook and which
nothing here relaxes.

**Decision, 2026-09-07.** Spec 046's `hooks_read_and_never_write` denies by
default, so it read the new `--version` probe as an unrecognised subcommand and
refused it. `--version` and `--help` are named explicitly in that predicate
rather than folded into a "flags are safe" rule: the deny-by-default is the
point, and the exemptions should stay countable.

Spec 051 requires this repository to run the hooks it ships, so the kit change
is mirrored into `.claude/settings.json` in the same commit. Its test caught the
omission, which is what it is for.

This is the concrete harm being fixed. A session that starts by being told its
committed shards are stale, when they are not, will regenerate and commit
artifacts that were already correct, on the authority of a diagnosis the tool
got wrong.

### 3.4 Nothing else moves

No verb's exit code changes for any corpus condition. `compile --check` still
exits `2` for a stale registry and `1` for a corpus that fails validation.
`index check` is unchanged. No committed artifact changes and no schema version
moves: `VERDICT_SCHEMA_VERSION` in particular does not, because a usage error
never reaches the verdict path (there is no verb yet to name).

## 4. Out of scope

**A distinct code for usage errors.** §3.1.

**Auditing every other clap-handled path.** `try_parse` covers argument parsing.
Errors clap raises later, from value parsing inside a subcommand, follow the same
mapping because they are the same error type.

**Making an old binary behave.** Impossible by construction, which is why §3.2
is a precondition rather than a fix.

**Enforcing the version pin.** Spec 062. This spec makes the codes honest; 062
makes the mismatch impossible to reach.

## 5. Verification

Each line is one command (spec 049 §3.2). The exit-code assertions fail against
pre-063 code, where clap exited 2 for all of them.

```verify:cli
# Self-contained: the commands below invoke the release binary.
cargo build --release --locked
cargo test -p spec-spine-cli --test cli --locked
# 3.1: a usage error is exit 3, not exit 2.
target/release/spec-spine compile --no-such-flag 2>/dev/null ; test $? -eq 3
target/release/spec-spine no-such-verb 2>/dev/null ; test $? -eq 3
# 3.1: an incomplete invocation is a usage error too, so a script that dropped
# its verb still fails.
target/release/spec-spine 2>/dev/null >/dev/null ; test $? -eq 3
# 3.1: help and version are asked for, so they succeed.
target/release/spec-spine --help >/dev/null
target/release/spec-spine --version >/dev/null
# 3.4: exit 2 still means stale, and now means only that.
target/release/spec-spine compile --check
# 3.2: the stderr-matching ritual is gone from both harness documents, replaced
# by the precondition that works against every version.
! grep -q 'unexpected argument' AGENTS.md
! grep -q 'unexpected argument' kit/AGENTS.md
grep -q 'Ask `spec-spine --version` before believing any exit code' AGENTS.md
grep -q 'before believing any exit code' kit/AGENTS.md
# 3.3: the kit hook establishes the flag exists before reading the exit code.
grep -q 'predates' kit/settings.json
```
