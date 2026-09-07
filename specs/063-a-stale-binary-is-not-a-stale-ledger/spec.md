---
id: "063-a-stale-binary-is-not-a-stale-ledger"
title: "A stale binary is not a stale ledger"
status: draft
kind: "tooling"
created: "2026-09-07"
implementation: pending
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

- `DisplayHelp`, `DisplayVersion` and the help-on-no-subcommand kinds keep exit
  `0` and keep writing to stdout. They are successful requests for information.
- Every other clap error exits **3**, writing clap's message to stderr
  unchanged.

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

```verify:cli
# Self-contained: the commands below invoke the release binary.
cargo build --release --locked
cargo test -p spec-spine-cli --test cli --locked
# An unknown flag is exit 3, not exit 2. Until this ships, clap exits 2 and the
# second assertion fails.
target/release/spec-spine compile --no-such-flag 2>/dev/null ; test $? -eq 3
target/release/spec-spine no-such-verb 2>/dev/null ; test $? -eq 3
# --help and --version stay exit 0.
target/release/spec-spine --help >/dev/null
target/release/spec-spine --version >/dev/null
# Exit 2 still means stale, and only that.
target/release/spec-spine compile --check
# The stderr-matching ritual is gone from both harness documents.
! grep -q 'unexpected argument' AGENTS.md
! grep -q 'unexpected argument' kit/AGENTS.md
```
