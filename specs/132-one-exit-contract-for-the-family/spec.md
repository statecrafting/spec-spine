---
id: "132-one-exit-contract-for-the-family"
title: "One exit contract for the family"
status: draft
kind: "tooling"
created: "2026-09-24"
summary: >
  spec-spine and the Statecraft CLI answered in two exit vocabularies. spec-spine
  spent 2 on staleness and 3 on everything that went wrong, from a malformed
  `spec-spine.toml` to a failed read to a refused write, so a consumer could not
  tell "you asked for something this tool will not do" from "the disk failed",
  and a containment refusal was reported as I/O. This spec adopts the family
  contract the two tools share: 0 ok, 1 finding (validation, drift, stale,
  unresolved, not found, authored content that does not parse), 2 refused (an
  invalid configuration, a version pin not met, a containment refusal: nothing
  done), 3 usage, 4 failed (I/O, internal, a tool-produced artifact that fails
  its schema). `Error` gains `Refused`, `Usage` and `Internal`, and every call
  site is classified. The `--json` envelope becomes the family envelope,
  version 1.0.0: `schemaVersion`, `tool`, `verb`, `outcome`, `exitCode`,
  `summary`, then `report` or `error`, with `ok` removed and `error.kind` a
  closed set of ten tokens. It is a deliberate breaking change, accepted by the
  owner, taken once so the consumer migrates once.
implementation: complete
owner: "The spec-spine Authors"
risk: high
depends_on:
  - "034-machine-readable-verdicts"
  - "062-one-name-one-freshness-verb"
  - "080-an-unresolved-claim-is-not-stale"
  - "093-the-harness-this-repository-runs"
  - "128-a-derived-tree-stays-in-its-repository"
  - "129-a-json-config-obeys-the-loaders-rules"
# Every approved spec whose normative text states a code this spec moves, or
# the envelope members it changes. The sections are listed per spec in §2,
# because `amends_sections` is one flat list shared by every target.
amends:
  - "034-machine-readable-verdicts"
  - "055-a-version-pin-the-cli-can-check"
  - "061-shipped-is-not-the-same-as-working"
  - "062-one-name-one-freshness-verb"
  - "079-a-blocking-claim-is-not-a-stale-shard"
  - "080-an-unresolved-claim-is-not-stale"
  - "093-the-harness-this-repository-runs"
  - "103-a-verifier-fixture-is-a-published-artifact"
  - "126-a-derived-file-stays-in-its-directory"
  - "127-a-derived-file-is-where-its-path-says"
  - "128-a-derived-tree-stays-in-its-repository"
  - "129-a-json-config-obeys-the-loaders-rules"
establishes:
  - { kind: file, path: "crates/spec-spine-cli/tests/exit_contract.rs" }
extends:
  # 3.1: the variants, the mapping, the outcome vocabulary.
  - { spec: "000-spec-spine-bootstrap", unit: "crates/spec-spine-types/src/error.rs", nature: corrective }
  - { spec: "000-spec-spine-bootstrap", unit: "crates/spec-spine-types/src/lib.rs", nature: additive }
  - { spec: "000-spec-spine-bootstrap", unit: "crates/spec-spine-types/src/config.rs", nature: corrective }
  # 3.4: the envelope and its version.
  - { spec: "034-machine-readable-verdicts", unit: "crates/spec-spine-types/src/verdict.rs", nature: corrective }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-types/src/version.rs", nature: corrective }
  # 3.2: the CLI's mapping, the folds, and the classified call sites.
  - { spec: "001-compile-registry", unit: "crates/spec-spine-cli/src/main.rs", nature: corrective }
  - { spec: "062-one-name-one-freshness-verb", unit: "crates/spec-spine-cli/src/cmd_check.rs", nature: corrective }
  - { spec: "004-codebase-index", unit: "crates/spec-spine-cli/src/cmd_index.rs", nature: corrective }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-cli/src/cmd_compile.rs", nature: corrective }
  - { spec: "005-coupling-gate", unit: "crates/spec-spine-cli/src/cmd_couple.rs", nature: corrective }
  - { spec: "021-ledger-seal", unit: "crates/spec-spine-cli/src/cmd_attest.rs", nature: corrective }
  - { spec: "021-ledger-seal", unit: "crates/spec-spine-cli/src/verify_attestation.rs", nature: corrective }
  - { spec: "021-ledger-seal", unit: "crates/spec-spine-cli/src/seal.rs", nature: corrective }
  - { spec: "096-compaction-is-a-verb-not-a-session", unit: "crates/spec-spine-cli/src/cmd_compact.rs", nature: corrective }
  - { spec: "002-registry-query", unit: "crates/spec-spine-cli/src/cmd_registry.rs", nature: corrective }
  - { spec: "108-a-work-scope-is-declared", unit: "crates/spec-spine-cli/src/cmd_scope.rs", nature: corrective }
  - { spec: "110-an-interface-reference-is-digest-pinned", unit: "crates/spec-spine-cli/src/cmd_interface.rs", nature: corrective }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-core/src/lib.rs", nature: corrective }
  - { spec: "022-index-sharding", unit: "crates/spec-spine-core/src/shard.rs", nature: corrective }
  - { spec: "004-codebase-index", unit: "crates/spec-spine-core/src/index.rs", nature: corrective }
  - { spec: "002-registry-query", unit: "crates/spec-spine-core/src/query.rs", nature: corrective }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-core/src/compile.rs", nature: corrective }
  - { spec: "021-ledger-seal", unit: "crates/spec-spine-core/src/attest.rs", nature: corrective }
  - { spec: "107-a-context-closure-is-declared", unit: "crates/spec-spine-core/src/closure.rs", nature: corrective }
  - { spec: "108-a-work-scope-is-declared", unit: "crates/spec-spine-core/src/scope.rs", nature: corrective }
  - { spec: "109-impact-and-conflict-are-declared", unit: "crates/spec-spine-core/src/impact.rs", nature: corrective }
  - { spec: "071-a-change-is-classified-under-the-bases-rules", unit: "crates/spec-spine-core/src/delta.rs", nature: corrective }
  - { spec: "110-an-interface-reference-is-digest-pinned", unit: "crates/spec-spine-core/src/interface.rs", nature: corrective }
  - { spec: "113-a-waiver-has-a-declared-lifecycle", unit: "crates/spec-spine-core/src/waiver.rs", nature: corrective }
  # 3.2: own-output serialization failures become `Internal`, and the doc
  # comments that stated the old codes.
  - { spec: "071-a-change-is-classified-under-the-bases-rules", unit: "crates/spec-spine-cli/src/cmd_delta.rs", nature: corrective }
  - { spec: "003-conformance-lint", unit: "crates/spec-spine-cli/src/cmd_lint.rs", nature: corrective }
  - { spec: "043-verify-declared-acceptance", unit: "crates/spec-spine-cli/src/cmd_verify.rs", nature: corrective }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-core/src/canonical_json.rs", nature: corrective }
  - { spec: "005-coupling-gate", unit: "crates/spec-spine-core/src/couple.rs", nature: corrective }
  - { spec: "029-ownership-coverage", unit: "crates/spec-spine-core/src/coverage.rs", nature: corrective }
  - { spec: "044-index-diagnostics-reach-a-gate", unit: "crates/spec-spine-core/src/diagnostics.rs", nature: corrective }
  - { spec: "074-a-governed-read-names-its-version", unit: "crates/spec-spine-core/src/read.rs", nature: corrective }
  - { spec: "067-a-short-id-names-the-same-spec-at-every-verb", unit: "crates/spec-spine-core/src/spec_id.rs", nature: corrective }
  # 3.6: the verifier fixture set, a published artifact, moves to 1.0.0.
  - { spec: "103-a-verifier-fixture-is-a-published-artifact", unit: "crates/spec-spine-core/fixtures/verifier/", nature: corrective }
  # 3.7: the harness and the scripts that read the codes.
  - { spec: "093-the-harness-this-repository-runs", unit: ".claude/settings.json", nature: corrective }
  - { spec: "093-the-harness-this-repository-runs", unit: "AGENTS.md", nature: corrective }
  - { spec: "093-the-harness-this-repository-runs", unit: ".claude/skills/code-review/SKILL.md", nature: corrective }
  - { spec: "093-the-harness-this-repository-runs", unit: ".claude/skills/ship/SKILL.md", nature: corrective }
  - { spec: "094-one-gate-and-the-boundaries-it-holds", unit: ".githooks/pre-commit", nature: corrective }
  - { spec: "099-a-merged-acceptance-is-asked-again", unit: "scripts/acceptance-scope.sh", nature: corrective }
  - { spec: "105-governed-scope-is-enabled-here", unit: "CLAUDE.md", nature: corrective }
references:
  - { unit: { kind: file, path: "docs/cli-reference.md" }, role: context }
  - { unit: { kind: file, path: "docs/design/00-architecture.md" }, role: context }
intent:
  goal: "one exit and JSON vocabulary for spec-spine and Statecraft, in which a refusal, a usage error and a failure are three different answers and staleness is a finding"
  non_goals:
    - "changing any verdict: every refusal still refuses and every pass still passes; only the code and the envelope change (3.2)"
    - "a lock: spec-spine takes none, so `lock busy` has no call site here (4)"
    - "renaming diagnostic codes (V-, L-, C-, I-, W-): those are finding identities, not outcomes (4)"
---

# 132: One exit contract for the family

## 1. Purpose

### 1.1 Two vocabularies

spec-spine's exit codes, as of 0.25.0 (`Error::exit_code`):

| Code | Meaning in spec-spine 0.25.0 |
|---|---|
| 0 | ok |
| 1 | validation failure, not found, drift, an unresolved claim (080) |
| 2 | stale |
| 3 | everything else: a malformed `spec-spine.toml`, a version pin not met, a containment refusal (126 to 128), a request document that does not parse, a missing file, a committed shard that does not parse, a clap usage error (093) |

The Statecraft CLI's (its spec 006 §3.3): 0 ok, 1 finding, 2 refused, 3 usage,
4 failed. A consumer that runs both reads the same number with two meanings,
and within spec-spine one number, 3, means both "the tool declined to do what
you asked" and "the disk failed". Spec 128's containment refusal is the sharpest
case: a deliberate refusal that nothing was written, reported as `io error`.

### 1.2 Measured before the change

`crates/spec-spine-cli/tests/exit_contract.rs` was written first and run
against the unchanged tree (`39911d1a`): 9 of its 10 cases failed, each on the
old code. The tenth, a spec whose frontmatter does not parse, already exited 1
and is the control that the finding tier was not moved by accident.

Writing the case for a failed read found a defect the old contract hid: a
committed shard directory that cannot be listed was read as empty, so
`index check` reported every shard as missing and prescribed `spec-spine index`
(exit 2, "stale"). §3.2 makes that exit 4.

## 2. Territory

`tests/exit_contract.rs` is new and established here. Everything else is an
`extends` crossing on the file that constructs or maps the error, listed in the
frontmatter.

**This spec amends twelve approved specs.** Each states a code this spec moves
or an envelope member it changes; none is edited (spec 037).

| Spec | Section | What it says, and what it becomes |
|---|---|---|
| 034 | 3.1, 3.3, 3.6 | envelope members `schemaVersion, verb, ok, exitCode`; `kind` table with `parse` at 3 | §3.4 here |
| 055 | 3.2 | an unsatisfiable pin exits 3 | 2, `kind: refused` |
| 061 | 3.7 | the "2 (stale)" row of the exit-code table | stale is 1 |
| 062 | 3.3 | `check` composes by "3 then 1 then 2 then 0" | the higher code |
| 079 | 3.1 | a stale tree MUST produce exit 2 | 1 |
| 080 | 3.1, 3.3, 3.4 | a stale shard alone keeps exit 2; envelope keeps its version | 1; envelope 1.0.0 |
| 093 | 3.8, 3.9, 3.10 | the hooks read 2 as staleness, 3 as a read not performed | §3.7 here |
| 103 | 3.5 | a payload that fails to load exits 3, `kind: parse` or `schema` | 4, `kind: schema`; fixture set 1.0.0 |
| 126 | 3.2, 3.3, 3.4 | a refused derived write exits 3 | 2, `kind: refused` |
| 127 | 3.2, 3.4, 3.5 | a linked or wrong-kind component refuses with exit 3 | 2 |
| 128 | 3.1, 3.2, 3.3 | an escaping `derived_dir` is a configuration error, exit 3 | 2, `kind: config`; the writer's backstop 2, `kind: refused` |
| 129 | 3.1 | `Error::Config`, exit 3 | exit 2 |

## 3. Behavior

### 3.1 The contract

| Code | `outcome` | Meaning |
|---|---|---|
| 0 | `ok` | the verb did what was asked and found nothing |
| 1 | `finding` | validation, drift, stale, an unresolved claim, not found, authored content that does not parse |
| 2 | `refused` | a precondition or policy was not met, and nothing was done |
| 3 | `usage` | the invocation is wrong |
| 4 | `failed` | the tool could not do its work: I/O, an internal error, a tool-produced artifact that fails its schema |

`Error` MUST map as follows, in one function (`Error::exit_code`):

| Variant | Code | `error.kind` |
|---|---|---|
| `Validation` | 1 | `validation` |
| `NotFound` | 1 | `not-found` |
| `Stale` | 1 | `stale` |
| `Parse` (authored content only) | 1 | `validation` |
| `Config` | 2 | `config` |
| `Refused` (new) | 2 | `refused` |
| `Usage` (new) | 3 | `usage` |
| `Io` | 4 | `io` |
| `Schema` | 4 | `schema` |
| `Internal` (new) | 4 | `internal` |

A refusal MUST NOT be reported as I/O, and I/O MUST NOT be reported as a
finding.

### 3.2 Where each case lands

- **Refused (2).** A `spec-spine.toml` or configuration JSON the loader refuses
  (`Config`). A `[meta] required_version` the binary does not satisfy, or a
  running version it cannot compare (`Refused`; a requirement that is not
  semver stays `Config`). The derived-write containment refusals of 126 to 128
  and an interface export path that escapes its root (`Refused`). `compact` on
  a dirty tree (`Refused`) and a compaction plan it will not apply (`Config`,
  unchanged variant). `couple`'s prior snapshot whose configuration or corpus
  cannot be built (`Refused`: the question cannot be asked of that commit).
- **Usage (3).** Clap's errors, as 093 §3.1 requires. An argument combination
  a verb rejects (`compile --json` without `--check`, `compile --spec` with
  `--check`, `attest --snapshot` with a scope flag or `--with-coupling` with
  `--spec`, `attest --sign` without `--key`, `verify-attestation` with no mode,
  `--snapshot` with `--spec`, or `--signature` without `--public-key`, `couple
  --include-uncommitted` with `--paths-from` or with a `--head` other than
  `HEAD`). A request document or argument the caller supplied that is not one:
  every facade request that does not deserialize, `--export`, `--waiver-uses`,
  `--waiver-as-of`, an unqualified obligation reference, a scope document with
  a malformed path, a `delta` path that is not repository-relative, an
  `index check --slice` that names an undeclared slice, a `verify-attestation
  --spec` that is not one path segment.
- **Failed (4).** Every genuine I/O and git failure (`Io`). A committed shard,
  registry, index, attestation or seal payload that does not parse, lacks its
  `schemaVersion`, or carries an unsupported MAJOR (`Schema`). A value the tool
  built that will not serialize (`Internal`). A shard directory that exists and
  cannot be listed MUST be `Io`, not an empty listing.
- **Finding (1).** Staleness in every verb that reads it (`check`, `index
  check`, `compile --check`, and every freshness-guarded read: `index
  coverage`, `index owner`, `couple`, `delta`, `registry closure`, `scope
  evaluate`, `interface verify`). Frontmatter and PR-body waiver lines that do
  not parse (`Parse`).

No verdict changes: a case that refused still refuses, and a case that passed
still passes.

### 3.3 `check` folds by the higher code (amends 062 §3.3)

Under this contract the numeric order is the severity order, so `check`'s
composed code MUST be the higher of its two halves. A stale half and an invalid
half are both 1; the report lines say which (the hooks read `INVALID` ahead of
`STALE`, 062 §3.3's reasoning, which survives: staleness is not meaningful
against a corpus that does not validate).

### 3.4 The family envelope (amends 034 §3.1, §3.3, §3.6)

Every verb that takes `--json` MUST write exactly these members:

```json
{
  "schemaVersion": "1.0.0",
  "tool": "spec-spine",
  "verb": "index.check",
  "outcome": "finding",
  "exitCode": 1,
  "summary": "index.check: finding",
  "report": { }
}
```

or `error` in place of `report`, carrying `kind`, `message`, and `violations`
when `kind` is `validation` and there are any. `outcome` MUST be derived from
`exitCode` and never passed separately. `summary` is one line of human text
with no stability promise. `ok` is removed: `outcome` states it and four more
answers. `error.kind` MUST be one of `validation`, `stale`, `not-found`,
`drift`, `refused`, `config`, `io`, `schema`, `usage`, `internal`
(`verdict::ERROR_KINDS`); `parse` is gone. `drift` is `couple`'s, carried in
its report, and is in the set so a family consumer can name it.

`VERDICT_SCHEMA_VERSION` MUST be `1.0.0`: a MAJOR, since members were removed
and codes moved.

A version-pin refusal under `--json` MUST be an envelope like any other
refusal (before this spec it was prose on stderr).

### 3.5 Verbs whose code changed

Every verb can now return 2, 3 or 4 where it returned 3, and every verb that
reads freshness returns 1 where it returned 2. By name: `check`, `compile`
(`--check`, `--spec`, the writing form), `index` (`check`, `coverage`, `owner`,
`render`, `orphans`, `diagnostics`), `lint`, `couple`, `delta`, `attest`,
`verify-attestation`, `verify`, `registry` (every query, `closure`, `impacts`,
`obligation`), `scope` (`evaluate`, `compare`), `interface verify`, `config
show`, `compact`. The mapping table in §3.1 is the whole of the change; no verb
has a rule of its own.

### 3.6 The verifier fixture set moves to 1.0.0 (amends 103 §3.5)

The fixture set under `crates/spec-spine-core/fixtures/verifier/` is a
published artifact. Its six load-failure cases (`unreadable-json`,
`missing-schema-version`, `unsupported-major`, `unknown-member` twice,
`duplicate-key`) MUST expect exit 4 and `error.kind: schema`, and the set's
`schemaVersion` MUST be `1.0.0`, because a consumer comparing exit codes must
notice the move. The payloads and every reason are unchanged.

### 3.7 The harness reads the five codes (amends 093 §3.8 to §3.10)

The `PreToolUse` PR gate, the `Stop` and `SessionStart` hooks and the
`pre-commit` guard MUST read `check` as: 0 fresh; 1 a finding, told apart by
the report (`INVALID` ahead of `STALE`, then `UNRESOLVED CLAIM`); 2 a refusal,
except that a report still saying `STALE` is a binary before 0.26.0 and is read
as stale, and a failed `check --help` probe is a binary before 0.18.0; 3 and 4 a
read not performed. Every non-zero code still refuses at the gates, and the
session hooks still only advise. `scripts/acceptance-scope.sh` MUST read
`index owner`'s 1 as stale and every other non-zero code as a refusal.

`AGENTS.md`'s freshness protocol MUST state the five codes.

## 4. Out of scope

**A lock.** The contract names "lock busy" as a refusal. spec-spine takes no
lock, so there is no call site; a future lock refuses with `Refused`.

**Diagnostic codes.** `V-`, `L-`, `C-`, `I-`, `W-` codes identify findings, not
outcomes, and are unchanged. Namespacing them by tool is a separate decision.

**Releases before 0.26.0.** Their binaries keep their codes. The hooks read a
stale report at exit 2 as stale so a pinned adopter is not misread.

## 5. Resolved decisions

**D-1 (2026-09-24): new variants, not contextual mapping at the CLI.** `Config`,
`Io`, `Parse` and `Schema` each straddled two of the new outcomes. Mapping by
call site at the CLI boundary would recreate the ad-hoc mapping
`Error::exit_code` exists to prevent, so the call sites construct the variant
that says what happened: `Refused`, `Usage`, `Internal`.

**D-2 (2026-09-24): `Parse` keeps its name and narrows its meaning.** It now
means authored content (frontmatter, a PR body's waiver line) and reports
`kind: validation`. Every site that parsed a document the caller supplied
became `Usage`, and every site that parsed a document the tool produced became
`Schema`.

**D-3 (2026-09-24): a missing registry or index is a failed read, 4.** `registry
list` before any `compile` reads a file the tool expected and did not find. It
could be argued as a refusal ("compile first"); it is classified by what
happened, a read that failed, and its message names the remedy.

**D-4 (2026-09-24): a corrupt committed shard read by `index check` is still
stale.** `index check` byte-compares (086), so a shard with the wrong bytes is
a shard that differs from what the corpus compiles to: a finding, 1. A consumer
verb that must parse the shard to answer (`registry list`) fails with 4.

**D-5 (2026-09-24): the fold is `max`.** The old fold needed a rank table
because 2 (stale) was less severe than 1 (invalid). With both at 1, the numeric
order is the severity order, and the function is `a.max(b)`.

## Verification

```verify:cli
# 3.1 to 3.5: one case per remapped outcome, each with its envelope; written
# first and red against 39911d1a in 9 of 10 cases.
cargo test -p spec-spine-cli --test exit_contract --locked
# 3.1, 3.4: the mapping table, the closed kind set, the envelope members.
cargo test -p spec-spine-types --lib verdict --locked
cargo test -p spec-spine-types --test dtos verdict --locked
# 3.3: the fold.
cargo test -p spec-spine-cli --bin spec-spine check_exit_order --locked
# 3.6: the fixture set describes the shipped verifier at 1.0.0.
cargo test -p spec-spine-cli --test verifier_fixtures --locked
# 3.7: the hooks read the five codes, and older binaries' codes.
cargo test -p spec-spine-core --test harness_hooks --locked
cargo test -p spec-spine-core --test commit_boundary --locked
```
