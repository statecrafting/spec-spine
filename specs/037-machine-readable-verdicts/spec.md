---
id: "037-machine-readable-verdicts"
title: "Machine-readable verdicts: `--json` on the adjudicating verbs"
status: draft
kind: "tooling"
created: "2026-09-05"
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "001-compile-registry"
  - "003-conformance-lint"
  - "004-codebase-index"
  - "005-coupling-gate"
  - "023-ledger-seal"
  - "035-stdout-closed-reader"
establishes:
  # The verdict envelope DTO: new territory, no existing owner.
  - "crates/spec-spine-types/src/verdict.rs"
extends:
  - { spec: "001-compile-registry", unit: "crates/spec-spine-cli/src/cmd_compile.rs", nature: additive }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-cli/src/main.rs", nature: additive }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-cli/tests/cli.rs", nature: additive }
  - { spec: "003-conformance-lint", unit: "crates/spec-spine-cli/src/cmd_lint.rs", nature: additive }
  - { spec: "004-codebase-index", unit: "crates/spec-spine-cli/src/cmd_index.rs", nature: additive }
  - { spec: "005-coupling-gate", unit: "crates/spec-spine-cli/src/cmd_couple.rs", nature: additive }
  - { spec: "023-ledger-seal", unit: "crates/spec-spine-cli/src/cmd_attest.rs", nature: additive }
  - { spec: "023-ledger-seal", unit: "crates/spec-spine-cli/src/verify_attestation.rs", nature: additive }
  - { spec: "035-stdout-closed-reader", unit: "crates/spec-spine-cli/src/out.rs", nature: additive }
  # The envelope's schema constant and its re-export; 000 floors the types crate
  # (the same additive shape specs 012, 013 and 032 used).
  - { spec: "000-spec-spine-bootstrap", unit: "crates/spec-spine-types/src/version.rs", nature: additive }
  - { spec: "000-spec-spine-bootstrap", unit: "crates/spec-spine-types/src/lib.rs", nature: additive }
  - { spec: "000-spec-spine-bootstrap", unit: "crates/spec-spine-types/tests/dtos.rs", nature: additive }
references:
  # The precedent (`--json` on the read verbs) and the design note this is wave 1 of.
  - { unit: { kind: file, path: "specs/010-registry-query-projection-flags/spec.md" }, role: context }
  - { unit: { kind: file, path: "docs/design/02-agentic-builder-substrate.md" }, role: context }
summary: >
  The read verbs speak JSON; the verbs that render a verdict do not. `registry`
  (four subcommands), `index orphans` and `index coverage` have `--json`,
  while `couple`, `lint`, `compile --check`, `index check`, `attest` and
  `verify-attestation` emit prose to stdout and nothing else. A programmatic consumer of the gate
  chain is therefore forced to string-match sentences like `index is fresh` and
  to infer the reasons for a refusal from formatted text, which is precisely the
  ad-hoc parsing constitution §II forbids and the governed-artifact-reads rule
  was written against; the CLI currently leaves no alternative for exactly the
  commands whose output is a decision. The reports already exist: every one of
  these verbs is backed by a facade function returning a typed structure, and
  only the CLI surface is missing. This spec adds `--json` to each of them,
  behind one versioned envelope (`schemaVersion`, `verb`, `ok`, `exitCode`,
  and either `report` or `error`) whose `report` member is byte-identical to the
  corresponding facade output, so a consumer parses one header shape across the
  whole chain and one payload shape per verb, never two spellings of the same
  verdict. `--json` changes what is written, never what is decided: the exit
  code for every outcome is unchanged, and a failure is reported inside the
  envelope rather than as bare prose on stderr, so the machine consumer's happy
  path and error path have the same shape.
---

# 037: Machine-readable verdicts

Wave 1 of `docs/design/02-agentic-builder-substrate.md`. An external driver
(an autonomous builder, a CI job, a bindings consumer) runs the gate chain and
must act on each verdict. Today it can act on the read verbs and must guess at
the adjudicating ones.

## 1. Purpose

`spec-spine` has two kinds of command. The **read** verbs project committed
state (`registry list`, `registry show`, `registry status-report`,
`registry relationships`, `index render`, `index orphans`, `index coverage`),
and since spec 010 the ones with structured payloads take `--json`
(`registry`'s four, `index orphans`, `index coverage`). The **adjudicating** verbs
render a verdict, and none of them do:

| Verb | What it decides | Machine-readable today |
|---|---|---|
| `compile --check` | are the committed registry shards current | no |
| `index check` | is the committed index current | no |
| `lint` | is the corpus conformant | no |
| `couple` | does the change drift from its owning spec | no |
| `attest` | the frozen verdict set over the corpus | file only, no stdout contract |
| `verify-attestation` | does the attestation reproduce and verify | no |

The asymmetry is backwards. A projection is the output most likely to be read by
a person; a verdict is the output most likely to be consumed by a program.

The consequence is not hypothetical. A consumer that wants the coupling gate's
reasons has to parse the formatted violation lines, which is a shape this project
has changed more than once and reserves the right to change again. A consumer
that wants staleness has to distinguish the literal string `index is fresh` from
its negation. That consumer is doing exactly what
`.claude/rules/governed-artifact-reads.md` forbids, and it has no supported
alternative, because the rule directs it to the subcommands and the subcommands
answer in prose.

Nothing has to be computed to fix this. `couple_json`, `lint_json`,
`check_freshness_json`, `check_registry_freshness_json`, `attest_json` and
`verify_attestation_json` all exist and all return typed structures. The CLI
throws that away and formats sentences.

## 2. Territory

The six `cmd_*` / verb modules above, `main.rs` for flag plumbing, `out.rs` for
the stdout write, and one new types module holding the envelope DTO plus its
schema constant. No engine module changes: this spec adds no computation, only a
second rendering of a verdict the engine already produces.

## 3. Behavior

### 3.1 One envelope, one payload per verb

Under `--json` a verb MUST write exactly one JSON object to stdout:

```json
{
  "schemaVersion": "0.1.0",
  "verb": "couple",
  "ok": false,
  "exitCode": 1,
  "report": { }
}
```

- `verb` is the stable dotted command path (`compile.check`, `index.check`,
  `lint`, `couple`, `attest`, `verify-attestation`).
- `ok` is the verdict, and MUST equal `exitCode == 0`. It is redundant on
  purpose: a consumer that has the exit code and a consumer that has only the
  document both read the same fact.
- `exitCode` is the code the process will actually return.
- `report` carries the verb's existing facade output **verbatim**. It MUST be
  byte-identical to what the corresponding `*_json` function returns for the same
  inputs, so there is one payload shape per verb rather than a library spelling
  and a CLI spelling that drift.

`report` and `error` are mutually exclusive; exactly one MUST be present.

### 3.2 `--json` changes what is written, never what is decided

Every exit code is unchanged by the flag. A stale index is exit `2` with and
without `--json`; a drifting change is exit `1`; a waived one is `0`. The
envelope reports the same verdict the prose reported.

This is the property the flag exists for. A driver reads the exit code for
control flow and the envelope for reasons, and the two can never disagree.

### 3.3 Failure is inside the envelope

Under `--json`, an outcome that `Error::exit_code()` maps MUST be emitted as an
envelope on stdout, not as bare prose on stderr:

```json
{
  "schemaVersion": "0.1.0",
  "verb": "index.check",
  "ok": false,
  "exitCode": 3,
  "error": { "kind": "parse", "message": "..." }
}
```

`kind` is a stable lowercase token derived from the `Error` variant, so a
consumer can branch on the class without matching the message. The message
remains human text and carries no stability promise.

The set is closed, and enumerating it is what makes the stability promise
verifiable rather than decorative:

| `kind` | `Error` variant | `exitCode` |
|---|---|---|
| `config` | `Config` | 3 |
| `validation` | `Validation` | 1 |
| `not-found` | `NotFound` | 1 |
| `stale` | `Stale` | 2 |
| `io` | `Io` | 3 |
| `parse` | `Parse` | 3 |
| `schema` | `Schema` | 3 |

Evolution follows `VERDICT_SCHEMA_VERSION`, which exists for exactly this:
adding a `kind` is a MINOR (a consumer's existing branches still match), while
renaming or removing one is a MAJOR (they stop matching). The token is
deliberately not the Rust variant name spelled automatically, because that would
make an internal rename a silent breaking change to an external contract with no
gate to catch it.

Without `--json` nothing changes: prose to stdout, diagnostics to stderr, exactly
as today. The rule is scoped to the flag so no existing consumer moves.

Rationale: a machine consumer whose success path is a parsed document and whose
failure path is an unparsed sentence has to implement two readers and will
implement one. Panics stay out of this entirely; core is panic-free on user input
and remains so.

### 3.4 The envelope is versioned

`VERDICT_SCHEMA_VERSION` joins the constants in `version.rs`, starting at
`0.1.0`, and is pinned by a test in `types/tests/dtos.rs` the way the other
schema versions are. It follows the same MINOR-is-additive / MAJOR-is-breaking
discipline as the registry and index versions, and is **independent** of them: a
consumer pins the shape of the verdict it parses without pinning the ledger.

An unversioned contract that external tools parse is the drift trap this project
exists to prevent, so the envelope carries its version from the first release
rather than acquiring one after the first consumer breaks.

### 3.5 Canonical bytes and a closed reader

The envelope is emitted with the project's canonical JSON discipline (sorted
keys, two-space pretty-print, LF, trailing newline), so `--json` output diffs
cleanly and can be committed by a consumer that chooses to.

The write goes through `out::block` (spec 035). `spec-spine couple --json | head`
MUST exit `0`, not `101`.

### 3.6 Tests (minimum)

- For each of the six verbs: `--json` on a passing corpus yields a parseable
  envelope with `ok: true`, `exitCode: 0`, and a `report` member.
- For each verb with a failure mode: the envelope carries the same exit code the
  prose form returns for identical inputs (a stale tree yields `2` both ways, a
  drifting diff yields `1` both ways).
- `report` equals the corresponding facade function's output byte for byte.
- An `Error` path under `--json` yields an `error` envelope on stdout with the
  mapped `exitCode`, and stdout contains nothing else.
- For each of the six verbs, `--json` piped into a reader that closes early
  exits `0`. The requirement in §3.5 is universal, so one `couple` test does not
  discharge it.
- The prose output of every verb is unchanged when the flag is absent.

## 4. Out of scope

**`init --json`.** `init` already returns files-as-data through
`scaffold_init_json`, and its CLI form writes files rather than rendering a
verdict. Adding a flag there is a separate, smaller question.

**Retrofitting the envelope onto the read verbs.** `registry --json` and
`index coverage --json` emit bare reports today and consumers exist. Wrapping
them would be a breaking change to a shipped surface for symmetry alone. If it is
ever wanted it is a MAJOR of their own contract, argued separately.

**A per-spec attestation.** The corpus-scoped `attest` payload is unchanged here;
this spec only gives its verdict a stdout contract. Scoping attestation to a
single spec is wave 2 of the design note.

**The writing forms of `compile` and `index`.** Both render a verdict on the
way past (`compile` exits 1 on a validation violation), but their purpose is to
mutate `.derived`, and a consumer that wants the verdict without the write has
`compile --check` and `index check`, which this spec covers. Giving a writing
command a machine-readable verdict invites a driver to parse the output of the
command that just changed the tree underneath it. If it is ever wanted, it is
additive and separate.

`attest` writes a file too and is nonetheless in scope, which is worth
distinguishing. What it writes *is* the verdict, so its stdout and its artifact
agree by construction, and it does not rewrite the ledger a consumer is about to
read: `compile` and `index` regenerate the very shards the next gate compares
against, which is the coupling that makes their verdicts awkward to consume mid-
chain. The flag is opt-in besides, so no existing caller of any of these commands
changes behavior until it passes `--json`.

**Changing any exit code.** The `0/1/2/3` mapping is a stable contract and this
spec does not touch it.

## 5. Resolved decisions

Three points where 3 was silent or where its wording admitted two readings.
Recorded at implementation, with the alternative that was rejected, rather than
resolved silently in code.

- **D-1 (2026-09-05): "byte-identical" is a claim about the payload, not the
  encoding.** 3.1 says `report` MUST be "byte-identical to what the
  corresponding `*_json` function returns". Read literally that is
  unsatisfiable alongside 3.5: the facade returns compact JSON, the envelope is
  canonical (sorted keys, two-space pretty-print), so embedding one inside the
  other re-encodes it by construction. 3.1's own example settles which reading
  was meant, since `"report": { }` is an object member and not a string holding
  an encoded document. The requirement is therefore implemented and tested as
  payload equality: the CLI's `report` and the facade's output parse to the same
  JSON value, so a divergence in members, spelling or values fails
  `json_report_equals_the_facade_payload`. Rejected: embedding the facade's
  string verbatim, which would make `report` a double-encoded string and defeat
  the flag's purpose.

- **D-2 (2026-09-05): `verify-attestation`'s report carries both modes.** The
  verb runs `--recompute` and `--signature` independently, and the facade
  (`verify_attestation_json`) models only recompute. A report that carried the
  facade's payload alone would silently drop the signature verdict, which 3.2
  forbids: the envelope must report the same verdict the prose reports, and the
  prose reports both. The report is therefore the facade's payload with an
  additive `signature: { valid, keyId }` member present exactly when
  `--signature` ran. For a recompute-only invocation, the inputs the facade
  accepts, the report is the facade's payload exactly, so D-1's equality holds
  there without qualification. Rejected: a `{ recompute, signature }` wrapper,
  which would have broken that equality for every invocation rather than
  extending it for one.

- **D-3 (2026-09-05): the freshness payload is rebuilt in the CLI, and pinned by
  test.** `compile --check --json` and `index check --json` need the
  `{ fresh, expected?, actual? }` shape that `check_registry_freshness_json` and
  `check_freshness_json` return, but reaching for those facades would recompile
  the corpus a second time on a command that runs in CI, and the shaping helper
  behind them is private to the facade. `cmd_compile::freshness_report` builds it
  from the typed `Freshness` the CLI already holds, and
  `json_report_equals_the_facade_payload` compares the two so the duplication
  cannot drift unnoticed. Rejected: widening `spec-spine-core`'s public surface,
  which 2 excludes from this spec's territory and which would make an internal
  shaping helper part of the library contract for no consumer's benefit.

- **D-4 (2026-09-06): the error envelope carries a validation failure's
  violations.** 3.3's example shows `error` as `{ kind, message }`, which is
  sufficient for six of the seven `Error` variants. `Error::Validation` is the
  exception: it carries a violation list, and `Display` reduces it to a count, so
  routing it through the generic error path discarded the only structured payload
  any error class has. The asymmetry was visible in the chain itself, since
  `lint --json` puts its violations in `report` on the same exit code while
  `compile --check --json` would have offered a sentence, leaving a consumer that
  handles both to fall back to parsing stderr for one of them. `VerdictError`
  therefore carries an additive `violations` array, present only when `kind` is
  `validation` and omitted rather than emitted empty. Rejected: putting them in
  `report` on that path, which would make `compile --check`'s report a union of
  a freshness object and a validation object and force every consumer to
  discriminate before reading either.

  With the violations inside the envelope, the stderr copy this path used to
  print became the one failure in the chain writing prose to a second channel
  under `--json`, so it is suppressed there and unchanged without the flag.

  The remaining asymmetry with `lint` is deliberate. Lint's violations *are* its
  verdict, so they are its report; a validation failure is the thing that
  prevented `compile --check` from reaching a freshness verdict at all, so it is
  an error that happens to carry detail. Same data, and the shape says which
  question was answered.
