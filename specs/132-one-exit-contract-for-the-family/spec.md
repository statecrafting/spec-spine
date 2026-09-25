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
amends_verification:
  - "067-a-short-id-names-the-same-spec-at-every-verb"
  - "068-a-verifier-checks-the-bytes-it-was-given"
  - "069-the-committed-index-is-compared-not-trusted"
  - "070-an-authority-snapshot-says-what-it-read"
amends:
  - "034-machine-readable-verdicts"
  - "055-a-version-pin-the-cli-can-check"
  - "061-shipped-is-not-the-same-as-working"
  - "062-one-name-one-freshness-verb"
  - "067-a-short-id-names-the-same-spec-at-every-verb"
  - "068-a-verifier-checks-the-bytes-it-was-given"
  - "069-the-committed-index-is-compared-not-trusted"
  - "070-an-authority-snapshot-says-what-it-read"
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
  # 3.1 to 3.7: every test that pinned a moved code or the old envelope.
  - { spec: "034-machine-readable-verdicts", unit: "crates/spec-spine-cli/tests/cli.rs", nature: additive }
  - { spec: "107-a-context-closure-is-declared", unit: "crates/spec-spine-cli/tests/closure.rs", nature: additive }
  - { spec: "107-a-context-closure-is-declared", unit: "crates/spec-spine-cli/tests/closure_composed.rs", nature: additive }
  - { spec: "047-effective-config-is-a-governed-read", unit: "crates/spec-spine-cli/tests/config.rs", nature: additive }
  - { spec: "005-coupling-gate", unit: "crates/spec-spine-cli/tests/couple.rs", nature: additive }
  - { spec: "128-a-derived-tree-stays-in-its-repository", unit: "crates/spec-spine-cli/tests/derived_dir_contained.rs", nature: additive }
  - { spec: "126-a-derived-file-stays-in-its-directory", unit: "crates/spec-spine-cli/tests/derived_paths.rs", nature: additive }
  - { spec: "127-a-derived-file-is-where-its-path-says", unit: "crates/spec-spine-cli/tests/derived_symlinks.rs", nature: additive }
  - { spec: "110-an-interface-reference-is-digest-pinned", unit: "crates/spec-spine-cli/tests/interface.rs", nature: additive }
  - { spec: "110-an-interface-reference-is-digest-pinned", unit: "crates/spec-spine-cli/tests/interface_composed.rs", nature: additive }
  - { spec: "108-a-work-scope-is-declared", unit: "crates/spec-spine-cli/tests/scope.rs", nature: additive }
  - { spec: "067-a-short-id-names-the-same-spec-at-every-verb", unit: "crates/spec-spine-cli/tests/spec_id.rs", nature: additive }
  - { spec: "103-a-verifier-fixture-is-a-published-artifact", unit: "crates/spec-spine-cli/tests/verifier_fixtures.rs", nature: additive }
  - { spec: "068-a-verifier-checks-the-bytes-it-was-given", unit: "crates/spec-spine-cli/tests/verify_attestation_bytes.rs", nature: additive }
  - { spec: "090-the-verdict-is-the-only-thing-on-stdout", unit: "crates/spec-spine-cli/tests/verify_streams.rs", nature: additive }
  - { spec: "113-a-waiver-has-a-declared-lifecycle", unit: "crates/spec-spine-cli/tests/waiver.rs", nature: additive }
  - { spec: "021-ledger-seal", unit: "crates/spec-spine-core/tests/attest.rs", nature: additive }
  - { spec: "107-a-context-closure-is-declared", unit: "crates/spec-spine-core/tests/closure.rs", nature: additive }
  - { spec: "096-compaction-is-a-verb-not-a-session", unit: "crates/spec-spine-core/tests/compact.rs", nature: additive }
  - { spec: "071-a-change-is-classified-under-the-bases-rules", unit: "crates/spec-spine-core/tests/delta.rs", nature: additive }
  - { spec: "093-the-harness-this-repository-runs", unit: "crates/spec-spine-core/tests/harness_hooks.rs", nature: additive }
  - { spec: "110-an-interface-reference-is-digest-pinned", unit: "crates/spec-spine-core/tests/interface.rs", nature: additive }
  - { spec: "002-registry-query", unit: "crates/spec-spine-core/tests/query.rs", nature: additive }
  - { spec: "097-a-path-leaves-the-corpus-the-way-a-spec-does", unit: "crates/spec-spine-core/tests/retire.rs", nature: additive }
  - { spec: "108-a-work-scope-is-declared", unit: "crates/spec-spine-core/tests/scope.rs", nature: additive }
  - { spec: "113-a-waiver-has-a-declared-lifecycle", unit: "crates/spec-spine-core/tests/waiver.rs", nature: additive }
  - { spec: "055-a-version-pin-the-cli-can-check", unit: "crates/spec-spine-types/tests/config.rs", nature: additive }
  - { spec: "034-machine-readable-verdicts", unit: "crates/spec-spine-types/tests/dtos.rs", nature: additive }
  - { spec: "012-declared-extra-frontmatter-passthrough", unit: "crates/spec-spine-types/tests/frontmatter.rs", nature: additive }
  - { spec: "057-the-docs-name-what-adopters-derived", unit: "docs/adoption-guide.md", nature: additive }
  - { spec: "100-a-deleted-path-is-judged-where-it-lived", unit: "docs/api.md", nature: additive }
  - { spec: "115-bindings-are-designed-not-shipped", unit: "docs/bindings-plan.md", nature: additive }
  - { spec: "055-a-version-pin-the-cli-can-check", unit: "docs/schema-versioning.md", nature: additive }
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

**This spec amends sixteen approved specs.** Each states a code this spec moves
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
| 067 | acceptance | a missing attestation falls through to exit 3 | 4; carried (§3.8) |
| 068 | acceptance | a payload that fails to load exits 3 | 4; carried (§3.8) |
| 069 | acceptance | a corrupted shard is stale at exit 2 | 1; carried (§3.8) |
| 070 | acceptance | a non-UTF-8 direct claim refuses `attest --spec` at exit 3 | 2; carried (§3.8) |

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
  A per-spec attestation over a directly claimed file that is not UTF-8
  (`Refused`: spec 070 §3.2.1's construction cannot answer it, and nothing is
  written).
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
  `schemaVersion`, or carries an unsupported MAJOR (`Schema`); a seal whose
  signature or algorithm cannot be read is the same (`Schema`), while the same
  malformed hex in an operator's public key stays `Config`. A value the tool
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

### 3.8 The acceptance this contract changes is carried here

Four approved specs' own `## Verification` blocks assert codes this spec
moves: 067 (a missing attestation file, 3 to 4), 068 (a payload that fails to
load, 3 to 4, eight lines), 069 (a corrupted shard is stale, 2 to 1, three
lines), 070 (a non-UTF-8 direct claim, 3 to 2). This spec MUST carry each of
those blocks in full through `amends_verification` (spec 082, as 083 did for
079), with only those codes migrated and every other line unchanged, and MUST
NOT edit the four files. The blocks spec 092 already carries (for 055, 062,
079 through 083) assert no moved code and are not taken over.

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
# ---- carried for 067-a-short-id-names-the-same-spec-at-every-verb (amends_verification), migrated to spec 132's codes ----
# Self-contained: the assertions below drive the release binary.
cargo build --release --locked
# 3.4 the policy, the library entry points, and the facade.
cargo test -p spec-spine-core --test spec_id --locked
# 3.5 the six-argument matrix.
cargo test -p spec-spine-cli --test spec_id --locked
# 3.5 the census; the grep refuses a filter that matched nothing, which cargo reports as a pass.
sh -c 'cargo test -p spec-spine-cli --bin spec-spine --locked spec_id_census 2>&1 | grep -q "test result: ok. [1-9]"'
# 3.1 the four arguments that refuse the short form today accept it.
target/release/spec-spine registry show 015 >/dev/null
target/release/spec-spine registry relationships 015 >/dev/null
target/release/spec-spine attest --spec 059 >/dev/null
sh -c 'target/release/spec-spine attest --spec 059-a-malformed-id-is-refused-not-a-panic >/dev/null && target/release/spec-spine verify-attestation --spec 059 --recompute >/dev/null'
# 3.3 show: the short and full forms print the same bytes.
sh -c 'A=$(target/release/spec-spine registry show 015-short-id-resolution --json); B=$(target/release/spec-spine registry show 015 --json); test -n "$A" && test "$A" = "$B"'
# 3.3 relationships: the same bytes, and the incoming edge a raw-argument comparison drops is present.
sh -c 'A=$(target/release/spec-spine registry relationships 015-short-id-resolution --json); B=$(target/release/spec-spine registry relationships 015 --json); echo "$A" | grep -q 043-verify-declared-acceptance && test "$A" = "$B"'
# 3.3 attest: the same payload bytes, which a short-form attestation over zero units would not be.
sh -c 'A=$(target/release/spec-spine attest --spec 059-a-malformed-id-is-refused-not-a-panic --json); B=$(target/release/spec-spine attest --spec 059 --json); echo "$A" | grep -q "\"contentHash\": \"" && test "$A" = "$B"'
# 3.3 attest: the file is named by the resolved id, and nothing is named after the argument.
sh -c 'D=.statecraft/derived/attestation/by-spec; rm -f "$D/070.json" "$D/059-a-malformed-id-is-refused-not-a-panic.json"; target/release/spec-spine attest --spec 059 >/dev/null || exit 1; test -f "$D/059-a-malformed-id-is-refused-not-a-panic.json" && test ! -e "$D/070.json"'
# 3.2 and D-6 a path is not an id: each of these resolved through the filesystem before 084.
sh -c 'for a in ../specs/059-a-malformed-id-is-refused-not-a-panic 059-a-malformed-id-is-refused-not-a-panic/; do target/release/spec-spine verify "$a" --plan >/dev/null 2>&1; test $? -eq 1 || { echo "resolved a path: $a" >&2; exit 1; }; done; E=$(target/release/spec-spine compile --spec ./059-a-malformed-id-is-refused-not-a-panic 2>&1); echo "$E" | grep -q "not found" && ! echo "$E" | grep -q V-001'
# 3.4 no source file outside spec_id.rs carries the ordinal-segment match.
sh -c 'test $(grep -rl "split(.-.).next()" crates/spec-spine-core/src crates/spec-spine-cli/src | grep -v "/spec_id.rs$" | wc -l) -eq 0'
# 3.1 the five arguments that refuse no match do it at exit 1 with one message.
sh -c 'E="${TMPDIR:-/tmp}/ss084.nm"; : > "$E"; X=0; for c in "registry show 999" "registry relationships 999" "compile --spec 999" "verify 999 --plan" "attest --spec 999"; do target/release/spec-spine $c >/dev/null 2>>"$E"; test $? -eq 1 || { echo "not exit 1: $c" >&2; X=1; }; done; U=$(sort -u "$E" | grep -c .); L=$(grep -c . "$E"); rm -f "$E"; test $X -eq 0 && test $L -eq 5 && test $U -eq 1'
# 3.6 a full id is still accepted at all six arguments (a guard: passes before and after).
sh -c 'for c in "registry show 015-short-id-resolution" "registry relationships 015-short-id-resolution" "compile --spec 015-short-id-resolution" "verify 015-short-id-resolution --plan" "attest --spec 015-short-id-resolution" "verify-attestation --spec 015-short-id-resolution --recompute"; do target/release/spec-spine $c >/dev/null || { echo "full id refused: $c" >&2; exit 1; }; done'
# 3.6 an unknown id keeps its exit code, and a partial ordinal is not an ordinal (guards).
sh -c 'target/release/spec-spine registry show 999 >/dev/null 2>&1; test $? -eq 1'
sh -c 'target/release/spec-spine registry show 16 >/dev/null 2>&1; test $? -eq 1'
# 3.2 and D-4 step 4 at verify-attestation, whose set is the attestation files: with 015's removed, 016 matches none and falls through to a failed read, exit 4 (132; was 3).
sh -c 'rm -f .statecraft/derived/attestation/by-spec/015-short-id-resolution.json; target/release/spec-spine verify-attestation --spec 015 --recompute >/dev/null 2>&1; test $? -eq 4'
# 3.2 validate_spec_id still refuses a path-shaped argument at verify-attestation, at exit 3 (a usage error since 132).
sh -c 'target/release/spec-spine verify-attestation --spec ../x --recompute >/dev/null 2>&1; test $? -eq 3'
# A scratch corpus whose two specs share an ordinal, the mistake a new draft makes.
rm -rf "${TMPDIR:-/tmp}/ss084" && mkdir -p "${TMPDIR:-/tmp}/ss084/specs/001-a" "${TMPDIR:-/tmp}/ss084/specs/001-b"
sh -c 'for n in a b; do printf -- "---\nid: \"001-%s\"\ntitle: \"t\"\nstatus: draft\ncreated: \"2026-09-11\"\nsummary: \"s\"\n---\n\n# t\n" "$n" > "${TMPDIR:-/tmp}/ss084/specs/001-$n/spec.md"; done'
# compile refuses that corpus (V-004, exit 1) and still writes the shards registry show reads.
sh -c 'target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss084" compile >/dev/null 2>&1; test $? -eq 1 && test -f "${TMPDIR:-/tmp}/ss084/.derived/spec-registry/by-spec/001-a.json" && test -f "${TMPDIR:-/tmp}/ss084/.derived/spec-registry/by-spec/001-b.json"'
# 3.1 all six arguments refuse the shared ordinal at exit 1, with one message naming both candidates.
sh -c 'R="${TMPDIR:-/tmp}/ss084"; E="${TMPDIR:-/tmp}/ss084.err"; D="$R/.derived/attestation/by-spec"; mkdir -p "$D"; : > "$D/001-a.json"; : > "$D/001-b.json"; : > "$E"; X=0; for c in "registry show 001" "registry relationships 001" "compile --spec 001" "verify 001 --plan" "attest --spec 001" "verify-attestation --spec 001 --recompute"; do target/release/spec-spine --repo "$R" $c >/dev/null 2>>"$E"; test $? -eq 1 || { echo "not exit 1: $c" >&2; X=1; }; done; U=$(sort -u "$E" | grep -c .); L=$(grep -c . "$E"); grep -q ambiguous "$E" && grep -q 001-a "$E" && grep -q 001-b "$E" || X=1; rm -rf "$R/.derived/attestation" "$E"; test $X -eq 0 && test $L -eq 6 && test $U -eq 1'
rm -rf "${TMPDIR:-/tmp}/ss084"
# 3.6 no committed shard moved: the lenient adapters return what the mirrors returned (a guard).
target/release/spec-spine check
# ---- carried for 068-a-verifier-checks-the-bytes-it-was-given (amends_verification), migrated to spec 132's codes ----
cargo build --release --locked
rm -rf "${TMPDIR:-/tmp}/ss085" && mkdir -p "${TMPDIR:-/tmp}/ss085"
printf '0707070707070707070707070707070707070707070707070707070707070707' > "${TMPDIR:-/tmp}/ss085/k"
target/release/spec-spine attest --sign --key "${TMPDIR:-/tmp}/ss085/k" >/dev/null
target/release/spec-spine attest --spec 066-an-attestation-covers-the-territory-it-claims --sign --key "${TMPDIR:-/tmp}/ss085/k" >/dev/null
sed -n 's/.*"keyId": "\([0-9a-f]*\)".*/\1/p' .statecraft/derived/attestation/attestation.sig > "${TMPDIR:-/tmp}/ss085/pub" && test -s "${TMPDIR:-/tmp}/ss085/pub"
target/release/spec-spine verify-attestation --recompute --signature --public-key "${TMPDIR:-/tmp}/ss085/pub"
D="${TMPDIR:-/tmp}/ss085"; awk 'NR==1{print; print "  \"prCouple\": {\"ok\": true},"; next} {print}' .statecraft/derived/attestation/attestation.json > "$D/t1.json"; target/release/spec-spine verify-attestation --attestation "$D/t1.json" --seal .statecraft/derived/attestation/attestation.sig --signature --public-key "$D/pub" 2> "$D/t1.err"; test $? -eq 4 && grep -q prCouple "$D/t1.err"
D="${TMPDIR:-/tmp}/ss085"; awk 'NR==1{print; print "  \"prCouple\": {\"ok\": true},"; next} {print}' .statecraft/derived/attestation/attestation.json > "$D/t1r.json"; target/release/spec-spine verify-attestation --attestation "$D/t1r.json" --recompute 2> "$D/t1r.err"; test $? -eq 4 && grep -q prCouple "$D/t1r.err"
D="${TMPDIR:-/tmp}/ss085"; awk '/"compile": \{/{print; print "      \"ignoredWarnings\": 12,"; next} {print}' .statecraft/derived/attestation/attestation.json > "$D/t2.json"; target/release/spec-spine verify-attestation --attestation "$D/t2.json" --seal .statecraft/derived/attestation/attestation.sig --recompute --signature --public-key "$D/pub" 2> "$D/t2.err"; test $? -eq 4 && grep -q ignoredWarnings "$D/t2.err"
D="${TMPDIR:-/tmp}/ss085"; A=.statecraft/derived/attestation/by-spec/066-an-attestation-covers-the-territory-it-claims; awk '/"contentHash":/ && !done {print "        \"coveredBy\": \"review\","; done=1} {print}' "$A.json" > "$D/s1.json"; target/release/spec-spine verify-attestation --spec 066-an-attestation-covers-the-territory-it-claims --attestation "$D/s1.json" --seal "$A.sig" --recompute --signature --public-key "$D/pub" 2> "$D/s1.err"; test $? -eq 4 && grep -q coveredBy "$D/s1.err"
D="${TMPDIR:-/tmp}/ss085"; awk 'NR==1{print; print "  \"revokedAt\": \"2026-01-01\","; next} {print}' .statecraft/derived/attestation/attestation.sig > "$D/sl.sig"; target/release/spec-spine verify-attestation --seal "$D/sl.sig" --signature --public-key "$D/pub" 2> "$D/sl.err"; test $? -eq 4 && grep -q revokedAt "$D/sl.err"
D="${TMPDIR:-/tmp}/ss085"; sed 's/"schemaVersion": "0.1.0"/"schemaVersion": "9.0.0"/' .statecraft/derived/attestation/attestation.json > "$D/t3.json"; target/release/spec-spine verify-attestation --attestation "$D/t3.json" --recompute 2> "$D/t3.err"; test $? -eq 4 && grep -q "MAJOR 9" "$D/t3.err"
D="${TMPDIR:-/tmp}/ss085"; A=.statecraft/derived/attestation/by-spec/066-an-attestation-covers-the-territory-it-claims.json; sed 's/"schemaVersion": "0.1.0"/"schemaVersion": "9.0.0"/' "$A" > "$D/s3.json"; target/release/spec-spine verify-attestation --spec 066-an-attestation-covers-the-territory-it-claims --attestation "$D/s3.json" --recompute 2> "$D/s3.err"; test $? -eq 4 && grep -q "MAJOR 9" "$D/s3.err"
D="${TMPDIR:-/tmp}/ss085"; sed 's/"schemaVersion": "0.1.0"/"schemaVersion": "0.2.0"/' .statecraft/derived/attestation/attestation.json > "$D/t4.json"; target/release/spec-spine verify-attestation --attestation "$D/t4.json" --recompute --json > "$D/t4.out"; test $? -eq 1 && grep -q '"schemaVersion (0.2.0' "$D/t4.out"
D="${TMPDIR:-/tmp}/ss085"; tr -d '\n' < .statecraft/derived/attestation/attestation.json > "$D/t6.json"; target/release/spec-spine verify-attestation --attestation "$D/t6.json" --seal .statecraft/derived/attestation/attestation.sig --signature --public-key "$D/pub"; test $? -eq 1
D="${TMPDIR:-/tmp}/ss085"; tr -d '\n' < .statecraft/derived/attestation/attestation.json > "$D/t6r.json"; target/release/spec-spine verify-attestation --attestation "$D/t6r.json" --recompute --json > "$D/t6r.out"; test $? -eq 1 && grep -q contentMismatch "$D/t6r.out" && grep -q "bytes are not the canonical serialization" "$D/t6r.out"
D="${TMPDIR:-/tmp}/ss085"; sed 's/"version": "[^"]*"/"version": "0.0.1"/' .statecraft/derived/attestation/attestation.json > "$D/r1.json"; target/release/spec-spine verify-attestation --attestation "$D/r1.json" --recompute --json > "$D/r1.out"; test $? -eq 1 && grep -q versionMismatch "$D/r1.out"
D="${TMPDIR:-/tmp}/ss085"; sed 's/"ok": true/"ok": false/' .statecraft/derived/attestation/attestation.json > "$D/r2.json"; target/release/spec-spine verify-attestation --attestation "$D/r2.json" --recompute --json > "$D/r2.out"; test $? -eq 1 && grep -q contentMismatch "$D/r2.out"
D="${TMPDIR:-/tmp}/ss085"; awk '/"registryHash":/ && !done {print "  \"registryHash\": \"00\","; done=1} {print}' .statecraft/derived/attestation/attestation.json > "$D/r3.json"; target/release/spec-spine verify-attestation --attestation "$D/r3.json" --recompute 2> "$D/r3.err"; test $? -eq 4 && grep -q registryHash "$D/r3.err"
target/release/spec-spine attest --spec 093-the-harness-this-repository-runs >/dev/null && target/release/spec-spine verify-attestation --spec 093-the-harness-this-repository-runs --recompute
target/release/spec-spine verify-attestation --spec 066-an-attestation-covers-the-territory-it-claims --recompute --signature --public-key "${TMPDIR:-/tmp}/ss085/pub"
H=$(target/release/spec-spine attest --json | sed -n 's/.*"attestationHash": "\([0-9a-f]*\)".*/\1/p'); F=.statecraft/derived/attestation/attestation.json; if command -v sha256sum >/dev/null 2>&1; then G=$(sha256sum "$F" | cut -d' ' -f1); else G=$(shasum -a 256 "$F" | cut -d' ' -f1); fi; test -n "$H" && test "$H" = "$G"
target/release/spec-spine check
rm -rf "${TMPDIR:-/tmp}/ss085"
# ---- carried for 069-the-committed-index-is-compared-not-trusted (amends_verification), migrated to spec 132's codes ----
cargo build --release --locked
rm -rf "${TMPDIR:-/tmp}/ss086" && mkdir -p "${TMPDIR:-/tmp}/ss086/specs/001-a" "${TMPDIR:-/tmp}/ss086/src"
printf -- '---\nid: "001-a"\ntitle: "a"\nstatus: approved\ncreated: "2026-09-11"\nimplementation: complete\nsummary: "s"\nestablishes:\n  - "src/a.rs"\n---\n\n# a\n' > "${TMPDIR:-/tmp}/ss086/specs/001-a/spec.md"
printf 'pub fn a() {}\n' > "${TMPDIR:-/tmp}/ss086/src/a.rs"
D="${TMPDIR:-/tmp}/ss086"; S=target/release/spec-spine; $S --repo "$D" compile >/dev/null && $S --repo "$D" index >/dev/null && $S --repo "$D" check
D="${TMPDIR:-/tmp}/ss086"; S=target/release/spec-spine; F="$D/.derived/codebase-index/by-spec/001-a.json"; cp "$F" "$D/bak"; sed 's#"src/a.rs"#"src/zz.rs"#g' "$D/bak" > "$F"; $S --repo "$D" index check > "$D/out" 2>&1; R=$?; cp "$D/bak" "$F"; test $R -eq 1 && grep -q "001-a" "$D/out"
D="${TMPDIR:-/tmp}/ss086"; S=target/release/spec-spine; F="$D/.derived/codebase-index/by-spec/001-a.json"; cp "$F" "$D/bak"; sed 's#"src/a.rs"#"src/zz.rs"#g' "$D/bak" > "$F"; $S --repo "$D" check >/dev/null 2>&1; R=$?; cp "$D/bak" "$F"; test $R -eq 1
D="${TMPDIR:-/tmp}/ss086"; S="$PWD/target/release/spec-spine"; cd "$D" && git init -q -b main && git -c user.email=t@example.invalid -c user.name=t add -A && git -c user.email=t@example.invalid -c user.name=t commit -qm base && sed -i.orig 's#"src/a.rs"#"src/zz.rs"#g' .derived/codebase-index/by-spec/001-a.json && rm .derived/codebase-index/by-spec/001-a.json.orig && printf 'pub fn a() { edited(); }\n' > src/a.rs && git -c user.email=t@example.invalid -c user.name=t commit -qam tamper && "$S" couple --base main~1 --head HEAD >/dev/null 2>&1; R=$?; git reset -q --hard main~1; test $R -eq 1
D="${TMPDIR:-/tmp}/ss086"; S=target/release/spec-spine; $S --repo "$D" index >/dev/null && $S --repo "$D" index check
target/release/spec-spine check
rm -rf "${TMPDIR:-/tmp}/ss086"
# ---- carried for 070-an-authority-snapshot-says-what-it-read (amends_verification), migrated to spec 132's codes ----
cargo build --release --locked
target/release/spec-spine attest --snapshot --json > "${TMPDIR:-/tmp}/ss087-self.json" && grep -q '"matchesRecompute": true' "${TMPDIR:-/tmp}/ss087-self.json"
A=$(target/release/spec-spine attest --snapshot --json); B=$(target/release/spec-spine attest --snapshot --json); test -n "$A" && test "$A" = "$B"
H=$(target/release/spec-spine attest --spec 066-an-attestation-covers-the-territory-it-claims --json | sed -n 's/.*"attestationHash": "\([0-9a-f]*\)".*/\1/p'); test -n "$H" && grep -q "\"specAttestationHash\": \"$H\"" .statecraft/derived/attestation/snapshot.json
test "$(( $(grep -c '"specAttestationHash"' .statecraft/derived/attestation/snapshot.json) + $(grep -c '"specAttestationUnavailable"' .statecraft/derived/attestation/snapshot.json) ))" -eq "$(target/release/spec-spine registry list --ids-only | wc -l | tr -d ' ')"
target/release/spec-spine verify-attestation --snapshot --recompute
rm -rf "${TMPDIR:-/tmp}/ss087" && mkdir -p "${TMPDIR:-/tmp}/ss087/specs/001-a" "${TMPDIR:-/tmp}/ss087/g"
printf -- '[index]\nextra_hashed_inputs = ["g/*"]\n' > "${TMPDIR:-/tmp}/ss087/spec-spine.toml"
printf -- '---\nid: "001-a"\ntitle: "t"\nstatus: draft\ncreated: "2026-09-11"\nsummary: "s"\nestablishes:\n  - "g/"\n---\n\n# t\n' > "${TMPDIR:-/tmp}/ss087/specs/001-a/spec.md"
D="${TMPDIR:-/tmp}/ss087"; S=target/release/spec-spine; printf 'x' > "$D/g/a"; printf 'y' > "$D/g/b"; A=$($S --repo "$D" attest --snapshot --json | awk '/"governanceInputs": \{/{getline; print; exit}'); rm "$D/g/b"; printf 'xg/b\000y' > "$D/g/a"; B=$($S --repo "$D" attest --snapshot --json | awk '/"governanceInputs": \{/{getline; print; exit}'); rm -f "$D/g/a" "$D/g/b"; printf 'x' > "$D/g/a"; printf 'y' > "$D/g/b"; test -n "$A" && test -n "$B" && test "$A" != "$B"
D="${TMPDIR:-/tmp}/ss087"; S=target/release/spec-spine; printf '\377\376\000\001' > "$D/g/a"; if command -v sha256sum >/dev/null 2>&1; then H=$(sha256sum "$D/g/a" | cut -d' ' -f1); else H=$(shasum -a 256 "$D/g/a" | cut -d' ' -f1); fi; B=$($S --repo "$D" attest --snapshot --json | awk '/"governanceInputs": \{/{getline; print; exit}'); printf 'sha256:%s' "$H" > "$D/g/a"; A=$($S --repo "$D" attest --snapshot --json | awk '/"governanceInputs": \{/{getline; print; exit}'); printf 'x' > "$D/g/a"; test -n "$A" && test -n "$B" && test "$A" != "$B"
D="${TMPDIR:-/tmp}/ss087"; S=target/release/spec-spine; $S --repo "$D" compile >/dev/null && $S --repo "$D" index >/dev/null || exit 1; A=$($S --repo "$D" attest --snapshot --json | grep -c '"territoryDigest"'); T=$($S --repo "$D" registry list --ids-only | wc -l | tr -d ' '); printf 'changed' > "$D/g/a"; B=$($S --repo "$D" attest --snapshot --json | sed -n 's/.*"territoryDigest": "\([0-9a-f]*\)".*/\1/p' | head -1); printf 'x' > "$D/g/a"; C=$($S --repo "$D" attest --snapshot --json | sed -n 's/.*"territoryDigest": "\([0-9a-f]*\)".*/\1/p' | head -1); test "$A" -eq "$T" && test -n "$B" && test -n "$C" && test "$B" != "$C"
D="${TMPDIR:-/tmp}/ss087"; S=target/release/spec-spine; $S --repo "$D" compile >/dev/null && $S --repo "$D" index >/dev/null && $S --repo "$D" attest --snapshot --json > "$D/before.json" && F="$D/.derived/spec-registry/by-spec/001-a.json" && cp "$F" "$D/shard.bak" && printf ' ' >> "$F" && $S --repo "$D" attest --snapshot --json > "$D/after.json"; R=$?; cp "$D/shard.bak" "$F"; test $R -eq 0 && grep -q '"matchesRecompute": false' "$D/after.json" && test "$(grep '"registryHash"' "$D/before.json")" = "$(grep '"registryHash"' "$D/after.json")"
D="${TMPDIR:-/tmp}/ss087"; S=target/release/spec-spine; A=$($S --repo "$D" attest --snapshot --json); printf '# a comment\n' >> "$D/spec-spine.toml"; B=$($S --repo "$D" attest --snapshot --json); printf -- '[index]\nextra_hashed_inputs = ["g/*"]\n' > "$D/spec-spine.toml"; test "$A" != "$B" && test "$(echo "$A" | grep '"inputsManifestHash"')" = "$(echo "$B" | grep '"inputsManifestHash"')"
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss087" attest --snapshot --spec 001-a >/dev/null 2> "${TMPDIR:-/tmp}/ss087/scope.err"; test $? -eq 3 && grep -q "cannot combine" "${TMPDIR:-/tmp}/ss087/scope.err"
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss087" attest --snapshot --with-coupling >/dev/null 2> "${TMPDIR:-/tmp}/ss087/scope2.err"; test $? -eq 3 && grep -q "cannot combine" "${TMPDIR:-/tmp}/ss087/scope2.err"
D="${TMPDIR:-/tmp}/ss087"; S=target/release/spec-spine; M="$D/specs/001-a/spec.md"; cp "$M" "$D/spec.bak"; A=$($S --repo "$D" attest --snapshot --json | sed -n 's/.*"territoryDigest": "\([0-9a-f]*\)".*/\1/p' | head -1); printf -- '---\nid: "001-a"\ntitle: "t"\nstatus: draft\ncreated: "2026-09-11"\nsummary: "s"\nestablishes:\n  - "g/"\n  - "g/a"\n---\n\n# t\n' > "$M"; B=$($S --repo "$D" attest --snapshot --json | sed -n 's/.*"territoryDigest": "\([0-9a-f]*\)".*/\1/p' | head -1); cp "$D/spec.bak" "$M"; test -n "$A" && test "$A" = "$B"
D="${TMPDIR:-/tmp}/ss087"; S=target/release/spec-spine; M="$D/specs/001-a/spec.md"; cp "$M" "$D/spec.bak"; printf -- '---\nid: "001-a"\ntitle: "t"\nstatus: draft\ncreated: "2026-09-11"\nsummary: "s"\nestablishes:\n  - "nowhere/at/all.txt"\n---\n\n# t\n' > "$M"; N=$($S --repo "$D" attest --snapshot --json | grep -c '"territoryDigest"'); cp "$D/spec.bak" "$M"; test "$N" -eq 0
D="${TMPDIR:-/tmp}/ss087"; S=target/release/spec-spine; mkdir -p "$D/g/e"; A=$($S --repo "$D" attest --snapshot --json | sed -n 's/.*"territoryDigest": "\([0-9a-f]*\)".*/\1/p' | head -1); rmdir "$D/g/e"; : > "$D/g/e"; B=$($S --repo "$D" attest --snapshot --json | sed -n 's/.*"territoryDigest": "\([0-9a-f]*\)".*/\1/p' | head -1); rm -f "$D/g/e"; test -n "$A" && test -n "$B" && test "$A" != "$B"
D="${TMPDIR:-/tmp}/ss087"; S=target/release/spec-spine; M="$D/specs/001-a/spec.md"; cp "$M" "$D/spec.bak"; printf '\377\376\000\001' > "$D/bin.dat"; printf -- '---\nid: "001-a"\ntitle: "t"\nstatus: draft\ncreated: "2026-09-11"\nsummary: "s"\nestablishes:\n  - "bin.dat"\n---\n\n# t\n' > "$M"; $S --repo "$D" attest --spec 001-a >/dev/null 2>&1; E=$?; J=$($S --repo "$D" attest --snapshot --json); R=$?; cp "$D/spec.bak" "$M"; rm -f "$D/bin.dat"; test $E -eq 2 && test $R -eq 0 && printf '%s' "$J" | grep -q '"specAttestationUnavailable": "non-utf8-direct-claim"' && printf '%s' "$J" | grep -q '"territoryDigest"' && ! printf '%s' "$J" | grep -q '"specAttestationHash"'
rm -rf "${TMPDIR:-/tmp}/ss087" "${TMPDIR:-/tmp}/ss087-self.json"
target/release/spec-spine check
```
