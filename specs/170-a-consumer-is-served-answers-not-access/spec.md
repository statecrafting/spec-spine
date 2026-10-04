---
id: "170-a-consumer-is-served-answers-not-access"
title: "A consumer is served answers, not access"
status: draft
kind: "governance"
created: "2026-09-29"
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "034-machine-readable-verdicts"
  - "104-a-producer-is-tested-as-published"
  - "132-one-exit-contract-for-the-family"
summary: >
  Fixes how spec-spine serves a programmatic consumer, the Statecraft CLI
  first. There are two tiers, split by whose pinned version decides the
  answer rather than by who is asking: a judging tier (the CLI's JSON
  envelopes and the family exit contract, answered by the binary the judged
  repository's own pin admits) and a producer tier (pure library functions
  whose output the caller owns). No surface is shaped for, gated to, or named
  after one consumer. A consumer's depth comes from a declared consumer
  contract instead: the verbs, schema axes, envelope members and exit codes it
  relies on, asserted by this repository's own suite, plus one public
  `capabilities` verb that states what a binary supports so no consumer has to
  probe `--help`.
establishes:
  - { kind: file, path: "crates/spec-spine-types/src/capabilities.rs" }
  - { kind: file, path: "crates/spec-spine-cli/src/cmd_capabilities.rs" }
  - { kind: file, path: "crates/spec-spine-cli/tests/capabilities.rs" }
  - { kind: file, path: "crates/spec-spine-cli/tests/consumer_contract.rs" }
  - { kind: directory, path: "crates/spec-spine-cli/tests/consumers/" }
extends:
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-types/src/lib.rs" }, nature: additive }
  - { spec: "074-a-governed-read-names-its-version", unit: { kind: file, path: "crates/spec-spine-types/src/version.rs" }, nature: additive }
  - { spec: "002-registry-query", unit: { kind: file, path: "crates/spec-spine-cli/src/main.rs" }, nature: additive }
  - { spec: "057-the-docs-name-what-adopters-derived", unit: { kind: section, file: "docs/cli-reference.md", anchor: "cli-reference" }, nature: additive }
  - { spec: "057-the-docs-name-what-adopters-derived", unit: { kind: file, path: "docs/schema-versioning.md" }, nature: additive }
references:
  - { unit: { kind: file, path: "specs/092-the-engine-ships-governance-not-an-environment/spec.md" }, role: "the engine and environment boundary this spec serves" }
  - { unit: { kind: file, path: "specs/107-a-context-closure-is-declared/spec.md" }, role: "the closure whose binding needs one path per question" }
obligations:
  - id: "R-1"
    kind: requirement
    text: "An answer about a repository is served only by the judging tier, from the binary that repository's own pin admits."
    anchor: "3-1-two-tiers-split-by-whose-version-decides"
  - id: "R-2"
    kind: requirement
    text: "One question has one path: no second verb, flag, feature or library entry answers a question a judging verb already answers."
    anchor: "3-2-one-question-one-path"
  - id: "I-1"
    kind: invariant
    text: "No surface is gated to, shaped for, or named after one consumer; a consumer's name appears only as data in its declared contract."
    anchor: "3-3-no-surface-belongs-to-a-consumer"
  - id: "R-3"
    kind: requirement
    text: "Every surface a declared consumer contract lists is asserted by this repository's own test suite, and changing one changes the contract in the same change."
    anchor: "3-4-a-consumer-contract-is-declared-and-tested"
  - id: "R-4"
    kind: requirement
    text: "The capabilities verb states every judging verb the binary supports with its schema axis and version, and writes nothing."
    anchor: "3-5-a-binary-states-what-it-supports"
  - id: "V-1"
    kind: verification
    text: "The capabilities verb and every declared consumer contract, the statecraft-cli contract first, are exercised against the in-tree binary."
    anchor: "verification"
    inputs:
      - "crates/spec-spine-cli/tests/capabilities.rs"
      - "crates/spec-spine-cli/tests/consumer_contract.rs"
      - "crates/spec-spine-cli/tests/consumers/statecraft-cli/contract.json"
intent:
  goal: "A consumer gets the depth it needs from public answers and a tested compatibility promise, never from privileged access."
  non_goals:
    - "a partner-only API tier, Cargo feature, or verb"
    - "new reading verbs (a non-writing snapshot, batch closure, revision-bound reads), each of which is its own spec"
---

# 170: A consumer is served answers, not access

## 1. Purpose

spec-spine has one programmatic consumer that depends on it deeply, the
Statecraft CLI, and the question keeps coming back whether that consumer should
get a deeper API than everyone else: a partner tier, gated behind a feature,
that exposes more of the engine. This spec answers no, and says what gives the
consumer its depth instead.

The answer follows from how the consumer already works. At `statecraft-cli`
`ee9f385` it links exactly one library function, `scaffold_init_json`, to
produce the governance files it writes. Every question it asks about a target
(`check`, `registry list`, `registry plan`, `registry closure`, `verify`,
`delta`) goes to the `spec-spine` binary as a child process in that target,
where the target's own `required_version` decides which binary may answer. That
split is what makes the consumer's answers trustworthy:

1. **The judged repository chooses its judge.** Two spec-spine releases can
   classify the same diff differently (the bypass floor alone is compiled into
   the binary). An in-process answer from the consumer's linked version would
   judge a target with a version the target never pinned.
2. **A contract binding needs one path.** The consumer binds an attempt to a
   context-closure digest (spec 107) before any effect and resolves it again at
   acceptance. The comparison proves "the contract did not move" only because
   both digests come from the same verb. A second path to the same answer would
   make a mismatch mean "a different path answered" as easily as "the contract
   moved".
3. **One question, one answer.** A verb only one consumer can reach creates a
   truth that continuous integration, other adopters and a human at a terminal
   cannot see, so "spec-spine says X" would depend on who asked.

What the consumer does lack is a way to rely on the public surface: nothing in
this repository fails when a change breaks an envelope member the consumer
reads, and the consumer detects a verb by running `registry closure --help` and
reading the exit status (`statecraft-run/src/contract.rs`), because no verb
states what a binary supports. This spec supplies both.

## 2. Territory

This spec owns:

- the capabilities verb and its document (`cmd_capabilities.rs`, the
  `capabilities.rs` type module, and `tests/capabilities.rs`), all planned;
- the declared consumer contracts, under `crates/spec-spine-cli/tests/consumers/`,
  and the test that asserts them, `tests/consumer_contract.rs`, all planned.

It extends, additively: the types crate's module list (`lib.rs`, spec 001) and
its version constants (`version.rs`, spec 074) for the capabilities schema
axis; the CLI's subcommand wiring (`main.rs`, spec 002); and the CLI reference
and schema-versioning documents (spec 057).

It changes no existing verb's output, adds no Cargo feature, and changes nothing
Statecraft does. Statecraft adopting the capabilities verb is Statecraft's own
change, under its own adoption ledger.

## 3. Behavior

### 3.1 Two tiers, split by whose version decides

spec-spine serves a programmatic consumer through exactly two tiers.

- **The judging tier** answers a question about a repository: its freshness,
  lifecycle, readiness, closure, classification, verification. It is the CLI's
  `--json` envelopes (spec 034) under the family exit contract (spec 132). An
  answer about a repository MUST come from this tier, and the binary that
  answers MUST be one the repository's `[meta] required_version` admits. The
  judging tier is public: every verb, flag and envelope member is available to
  every caller.
- **The producer tier** is the library facade's pure producers, whose output the
  caller owns and writes: `scaffold_init_json` and `scaffold_init_opts_json`
  today. A producer answers under the caller's linked version, because the
  caller is producing its own artifact, not judging someone else's.

The core library's other public functions remain public Rust. They are the
engine's internal structure made reusable, and they carry no promise to a
consumer: the stable library surface is the JSON facade (CLAUDE.md, "the FFI
seam"), and a consumer that judges a target by linking the engine has stepped
outside both tiers.

### 3.2 One question, one path

A question the judging tier answers MUST have exactly one path to its answer.
No verb, flag, feature, environment variable or facade entry may compute the
same answer by a different route, for any caller. When a faster or batched form
is needed, it is a form of the same verb, computed by the same code, and its
result for one item MUST be byte-identical to the unbatched answer for that
item.

### 3.3 No surface belongs to a consumer

No Cargo feature, verb, flag, envelope member or configuration key may be gated
to, shaped for, or named after one consumer. The engine does not know who is
calling it. A consumer's name appears in this repository only as data, in its
declared contract (section 3.4), and in prose that cites it as a user.

### 3.4 A consumer contract is declared and tested

A consumer that depends on the judging tier MAY be declared by one directory
under `crates/spec-spine-cli/tests/consumers/<consumer>/` holding:

- `contract.json`: for each verb the consumer invokes, the exact arguments, the
  envelope's schema axis and the versions of it the consumer reads, the
  envelope members it reads (as JSON Pointers), and the exit codes it
  distinguishes;
- the fixture repositories those invocations run against.

`tests/consumer_contract.rs` MUST run every declared invocation with the
in-tree binary against its fixture and assert that each listed member is
present with the declared type, the schema axis is one of the declared
versions, and the exit code is one of the declared codes.

A change that breaks a declared surface therefore fails this repository's own
suite. The change MUST update the contract in the same change, and MUST record
a dated decision naming the consumer and the surface, so the consumer's
adoption review of the release that carries it can find it.

**Stability is membership, not a version number.** A surface listed in any
declared contract is held to this section; an unlisted one is best-effort.
Schema-axis numbering cannot carry this, because the axes already mix `0.x`
and `1.x` for reasons unrelated to stability (registry `1.9.0`, delta `0.2.0`,
read `0.9.0`).

The first declared consumer is `statecraft-cli`, from the invocations its
readers make at `ee9f385`:

| Invocation | What it reads |
|---|---|
| `--version` | the version token |
| `check` | the exit code |
| `registry list --json` | the spec rows and their lifecycle |
| `registry plan --json` | the ready and blocked sets |
| `registry closure --request - --json` | the resolved members and the closure digest |
| `verify <id> --json` | the per-command results |
| `delta --base <rev> --head <rev> --json` | the classes and prior-policy answer |

The contract lists the members those readers actually use, taken from the
consumer's source when the contract is written, not the whole envelope.

### 3.5 A binary states what it supports

A new judging verb, `spec-spine capabilities --json`, MUST emit one document
naming the binary's version and, for every judging verb it supports, the verb
path, its flags, and the schema axis and version its `--json` envelope carries.
It reads no repository, writes nothing, and exits 0. It is the supported way to
ask whether a verb exists; a consumer MUST NOT need to run a verb's `--help` or
infer support from an exit code.

The document has its own schema axis, starting at `0.1.0`, recorded in
`version.rs` and `docs/schema-versioning.md`. Every verb this binary wires MUST
appear in it, and `tests/capabilities.rs` asserts that the listed set equals the
wired set, so the document cannot fall behind the CLI.

### 3.6 Observable negative cases

| Case | Required behavior |
|---|---|
| A change removes an envelope member a declared contract lists | `consumer_contract.rs` fails naming the consumer, the invocation and the member |
| A change bumps a schema axis to a version no contract declares | The same test fails naming the axis and both versions |
| A verb is wired and absent from `capabilities` | `tests/capabilities.rs` fails naming the verb |
| A proposal adds a feature, flag or verb available to one consumer only | Refused by review under section 3.3; there is no mechanical test for intent |
| `capabilities` is run outside any repository | It answers, exit 0; it reads no repository |

## 4. Out of scope

Each is a separate spec, reopened by a consumer that needs it:

- **A snapshot that writes nothing.** `attest --snapshot` always writes
  `<derived>/attestation/snapshot.json` into the repository, so a consumer
  cannot use spec 070's `AuthoritySnapshot` as a read.
- **Batch closure.** One `registry closure` process per spec is the consumer's
  cost today. A batched form is bound by section 3.2.
- **Revision-bound judging reads.** `content select --revision` (spec 155) and
  `delta` (spec 071) already read an exact commit; `registry plan` and
  `registry closure` do not.
- **A long-lived process speaking JSON over standard input and output.** If
  process start-up cost becomes measured, it would still be the binary the
  repository's pin admits, started per invocation.
- **Any change to how Statecraft consumes spec-spine**, including adopting
  `capabilities` in place of its `--help` probe. That is Statecraft's change.

## 5. Resolved decisions

**D-1 (2026-09-29, no partner tier).** The owner asked whether a deeper,
feature-gated partner API for Statecraft would keep spec-spine most useful to
it. Rejected for the three reasons in section 1. The depth is supplied by
section 3.4's tested contract and section 3.5's capabilities document instead,
both open to every caller.

**D-2 (2026-09-29, stability by contract membership).** Considered and rejected:
marking unstable surfaces by a `0.x` schema axis. The existing axes already mix
`0.x` and `1.x` for reasons unrelated to stability, so the version number could
not carry the signal without renumbering every axis.

**D-3 (2026-09-29, the contract lives in this repository).** The consumer's
requirements are recorded here as test data, rather than fetched from the
consumer, so the suite stays hermetic. Pinning the consumer's reader spec with
an `interface_references` digest (spec 110) is left to the implementation,
when the digest can be read from the consumer's corpus under its own pin.

**D-4 (2026-09-29, build): a contract's versions are prefixes.** A declared
version admits a document whose axis equals it or continues it at a `.`
(`0.2` admits `0.2.x`, not `0.20.0`). statecraft-cli checks one axis, the
delta report's, and accepts `0.1.x` and `0.2.x` (`delta.rs:54`); the contract
declares exactly that. Every other invocation it makes reads no version, so the
contract declares the MAJOR line its members were measured on (`0` for read,
`1` for verdict): an additive MINOR keeps every listed member by the policy in
`docs/schema-versioning.md`, and a MAJOR bump fails the suite naming the axis
and both versions (3.6). Declaring exact versions was rejected: every additive
read document would fail the contract of a consumer that does not read it.

**D-5 (2026-09-29, build): the contract is what the readers read, 3.4's table
and one more.** Measured from statecraft-cli `ee9f385`: the seven tabled
invocations, plus `verify <id> --plan --json`
(`statecraft-adapter/src/coverage.rs:302`), which the table omitted and whose
reader requires `verb`, `exitCode` and `outcome` by value. Members the reader
requires by value carry `equals`; members it treats as optional carry
`optional`. The `verify <id> --json` reader locates the envelope by the bytes
`{\n  "exitCode"`, so the contract holds that layout too (`stdoutContains`).
`config show` (run only for a non-exact pin, stdout unread) and the `compile`
and `index` runs (exit status only) are not listed.

**D-6 (2026-09-29, build): the fixture is a directory, the history is made
by the test.** `consumers/statecraft-cli/fixture/` holds a two-spec corpus (one
ready spec with an obligation and a declined `verify:browser` block, one
blocked draft). The test copies it, compiles and indexes it, commits it as the
base, applies the contract's `change` and commits the head, so `delta` has a
range to classify. Nothing in the fixture is Git state.

**D-7 (2026-09-29, build): D-3's digest pin is not taken.** Pinning the
consumer's reader spec with `interface_references` (spec 110) needs a digest
read from statecraft-cli's corpus under its own pin, and statecraft-cli's
readers are code, not a spec that names these members. The contract records
the consumer revision it was measured from (`measuredAt`) and its dated
`decisions` instead.

**D-8 (2026-09-29, build): `capabilities` is answered before the pin.** 3.5
says it reads no repository, and 3.6 that it answers outside one. `main`
answers it before `[meta] required_version` is read, so neither a pin this
binary does not meet nor an unreadable `spec-spine.toml` refuses it; the test
asserts both. The verb list is walked from the clap tree the binary parses its
arguments with; the one hand-kept table is each verb's `--json` schema axes,
and a verb that takes `--json` and is missing from it fails the suite.

## Verification

```verify:cli
cargo build --release -p spec-spine-cli --locked
cargo test -p spec-spine-cli --locked --test capabilities
cargo test -p spec-spine-cli --locked --test consumer_contract
test -f crates/spec-spine-cli/tests/consumers/statecraft-cli/contract.json
```
