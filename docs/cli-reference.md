# CLI reference

Every verb the `spec-spine` binary exposes, its flags, what it decides, and the
exit code it returns. Written against `spec-spine 0.21.0`; `spec-spine --help`
and `spec-spine <verb> --help` are the authority if the two ever disagree.

This document is part of the D-3 retention of the deleted documentation site
(`docs/design/09-disposition-2026-09-21.md` section 8). The site's `cli/`
section was the only place a per-verb flag and exit-code reference existed. It
is brought current here rather than copied: `init` is gone (spec 092), and
`check`, `delta`, `compact`, `config show`, `index owner`, `index diagnostics`
and the `--json` envelope all postdate the deleted pages.

## Exit codes

Mapped in exactly one place, `crates/spec-spine-cli/src/main.rs` via
`Error::exit_code()`. The contract is stable across releases.

| Code | Meaning |
|---|---|
| `0` | ok |
| `1` | validation failure, not found, or drift refused |
| `2` | stale: the committed derived tree does not match its inputs |
| `3` | I/O, parse, schema, config, or usage error |

Since spec 093 every usage error is `3`, so an exit `2` from any verb means
staleness and nothing else. A binary older than that maps some usage errors to
`2`; ask `spec-spine --version` before believing an exit code.

`delta` is the one verb with no refusal of its own: it exits `0` whenever a
report was produced, whatever the report says.

## Global flags

| Flag | Meaning |
|---|---|
| `--repo <DIR>` | Repository root. Defaults to the current directory. Accepted by every verb and subcommand. |
| `-V`, `--version` | Print the version and exit `0`. Every released binary answers it. |

## The machine-readable verdict (`--json`)

The verbs that render a *verdict* accept `--json` (spec 034). The flag changes
what is written and never what is decided: `ok` and `exitCode` always agree
with the code the process returns without it.

```json
{
  "schemaVersion": "0.3.0",
  "verb": "couple",
  "ok": false,
  "exitCode": 1,
  "report": { "...": "the verb's facade payload" }
}
```

`report` and `error` are mutually exclusive and exactly one is present. A
failure carries `error` instead:

```json
{ "kind": "stale", "message": "...", "violations": [] }
```

`kind` is a closed lowercase set to branch on: `config`, `validation`,
`not-found`, `stale`, `io`, `parse`, `schema`. `message` carries no stability
promise. `violations` is present only when `kind` is `validation`.

`verb` is a stable dotted command path, not the argv spelling:

| `verb` | Command |
|---|---|
| `compile.check` | `compile --check` |
| `compile.spec` | `compile --spec <id>` |
| `index.check` | `index check`, including `--slice` |
| `check` | `check` (both trees, spec 062) |
| `lint` | `lint` |
| `couple` | `couple` |
| `attest` | `attest` |
| `verify-attestation` | `verify-attestation` |
| `verify` | `verify <id>` |
| `delta` | `delta` |

`check` is its own verb and not a variant of either primitive: a consumer
branching on `compile.check` must not silently receive it.

The read-only query verbs (`registry *`, `index render|orphans|diagnostics|owner|coverage`,
`config show`) have spoken JSON since spec 009 and emit their document
directly, not wrapped in this envelope.

## compile

```
spec-spine compile [--check | --spec <ID>] [--fail-on-warn] [--json]
```

Compiles `specs/*/spec.md` into the deterministic registry shard tree under
`<derived_dir>/spec-registry/by-spec/<id>.json`.

| Flag | Meaning |
|---|---|
| `--check` | Verify the committed shards match the corpus, writing nothing. Exit `2` if stale. |
| `--spec <ID>` | Validate exactly one spec and write nothing (spec 049). Accepts the short id (`056`). Incompatible with `--check`. |
| `--fail-on-warn` | Exit `1` on any warning-tier violation (spec 064). Accepted on every form; changes only the exit code, never `validation.passed` and never an emitted byte. |
| `--json` | Requires `--check` or `--spec`. The writing form's verdict is deliberately not machine-readable (spec 034 section 4). |

The bare writing form is for authors. A gate calls `--check` or `check`,
because a gate must never repair the tree it is judging.

## check

```
spec-spine check [--fail-on-unresolved] [--fail-on-warn] [--json]
```

Both freshness reads in one verb (spec 062): are the committed registry shards
and the committed index shards current? It compiles in memory and compares
without writing. The exit code is the more severe of the two halves, in the
order `3`, `1`, `2`, `0`, and the two report lines under it say which tree
answered what.

| Flag | Forwarded to | Meaning |
|---|---|---|
| `--fail-on-unresolved` | the index half | Exit `1` when the committed index records any unresolved-unit diagnostic (`W-001` / `W-002`). |
| `--fail-on-warn` | the compile half | Exit `1` on any warning-tier violation. |

Neither implies the other and either may be passed alone. This is the only form
CI can call: the self-governance job runs `check` in place of `compile` and
`index`.

## registry

Read-only queries over the compiled registry. Every subcommand takes `--json`.

| Subcommand | Answers |
|---|---|
| `registry list [--status S] [--ids-only]` | The corpus, optionally filtered by status. `--ids-only` prints bare ids, one per line (a JSON string array with `--json`). |
| `registry show <ID>` | One spec. Accepts the short id (`016`). |
| `registry status-report [--nonzero-only]` | Counts by status. `--nonzero-only` omits zero counts; the total still covers the corpus. |
| `registry relationships <ID>` | A spec's relationship neighborhood: the typed edges in and out. |
| `registry obligation <SPEC>#<ID>` | One declared obligation (spec 106): its kind, text, anchor and inputs, its section's digest, and the spec's content hash. The spec half accepts the short id; an unqualified id exits `3`, an unknown one `1`. |
| `registry closure --request <FILE\|->` | Resolve a context closure (spec 107): every named spec, section and obligation with its identity, and one order-independent digest over them. Refuses a stale registry with exit `2`. |
| `registry plan [--next]` | Which specs can be worked on now and what blocks the rest (spec 035). `--next` prints only the first ready spec; an empty ready set is `(nothing ready)` at exit `0`, not a failure. |

`plan`'s `ready` set is a **scheduling** answer, not an approval. See
`docs/api.md` for what membership does and does not mean.

## index

```
spec-spine index [--repo DIR]
```

Builds the code-as-source index into `<derived_dir>/codebase-index/{by-spec,by-package}/`.

| Subcommand | Answers |
|---|---|
| `index check [--slice NAME] [--fail-on-unresolved] [--json]` | Is the committed index current? Exit `2` if stale. `--slice` gates one named `[index.slices]` group instead of the whole shard set. `--fail-on-unresolved` exits `1` on any `W-001` / `W-002`. |
| `index render` | The committed index as markdown. A projection; it never recomputes. |
| `index orphans [--json]` | Specs the committed index records as owning nothing resolvable. |
| `index diagnostics [--json]` | The diagnostics the committed index records (spec 044). Recomputes nothing and never refuses; the refusal lives on `check`. Empty output means none. |
| `index owner <PATH> [--json]` | Which specs own one path, and how (spec 048). Calls the coupling gate's own owner derivation, so this answer and a `C-001` decision cannot disagree. The path need not exist on disk. |
| `index coverage [--fail-on-untraced] [--paths-from FILE] [--json]` | Which source files no spec specifically claims (spec 029). `--fail-on-untraced` exits `1` unless every source file has a specific owning spec. `--paths-from` supplies the candidate paths instead of asking git (spec 078), which is the git-free route. |

Bare `index coverage` refuses nothing. The ratchet is the flag.

## lint

```
spec-spine lint [--fail-on-warn] [--fail-on-info] [--json]
```

Corpus conformance (the `L-` codes). Bare `lint` reports; `--fail-on-warn`
exits `1` on any warning-tier diagnostic, `--fail-on-info` on any info-tier
one.

## couple

```
spec-spine couple [--base BASE] [--head HEAD] [--pr-body FILE]
                  [--paths-from FILE] [--include-uncommitted] [--json]
```

The PR-time gate: refuse code that drifts from its owning spec. The diff it
reads is `git diff --no-color -U0 --no-renames base...head`, a three-dot range,
so the "before" side is `merge-base(base, head)`.

| Flag | Meaning |
|---|---|
| `--base <BASE>` | Base ref. Default `origin/main`. |
| `--head <HEAD>` | Head ref. Default `HEAD`. |
| `--pr-body <FILE>` | The waiver source, as a file path. Falls back to `$SPEC_SPINE_PR_BODY`. |
| `--paths-from <FILE>` | Override the diff: newline-delimited changed paths, whole-file authority, no hunk data. |
| `--include-uncommitted` | Also judge the index and working tree, so a pre-commit run sees the change being committed (spec 081). Only valid when `--head` resolves to `HEAD`, and never with `--paths-from`. |

Refusal codes: `C-001` (a path changed without an authoring edit to any owning
spec) and `C-002` (a changed source file no spec claims, when
`[coupling] require_ownership` is on).

A `Spec-Drift-Waiver:` line in the PR body clears a named path. It is a human
instrument; an agent never writes one on its own authority. A dependency-only
manifest bump self-clears through `[coupling] auto_waive_dependency_only`.

## delta

```
spec-spine delta [--base BASE] [--head HEAD] [--json]
```

Classifies every path a change touches under the **merge base's** rules (spec
071): implementation, requirement, verification, authority, lifecycle,
constitutional, policy, derived, bypassed, unowned or unknown. A path may carry
several classes.

The configuration and the index that classify are the merge base's, so a change
cannot reclassify itself by editing `spec-spine.toml`.

`priorPolicy.required` is true when any path carries requirement, verification,
authority, lifecycle, constitutional, policy or unknown. `required: false`
means only that no structural class above changed; it is not a statement that
the change is safe, correct or approved. `delta` refuses nothing.

## verify

```
spec-spine verify <ID> [--plan] [--json]
```

Runs a spec's declared acceptance: the `verify:cli` commands under its
`## Verification` heading, in order, stopping at the first failure. `<ID>`
accepts the short form (`049`).

`--plan` prints the commands and runs none of them. Reading the plan before
executing it is the safety affordance for the one verb that runs what the
corpus declares, which is why `verify` is deliberately outside the gate chain:
the chain runs against branches whose contents are, in the general case, a
stranger's.

Under `--json`, stdout is reserved for the envelope and the child commands'
output goes to stderr (spec 090; filed pre-collapse as 118, see docs/corpus-map.md).

## attest

```
spec-spine attest [--spec ID] [--with-coupling] [--snapshot]
                  [--sign --key PATH] [--key-id ID] [--json]
```

Emits a reproducible attestation into `<derived_dir>/attestation/`.

| Flag | Meaning |
|---|---|
| `--spec <ID>` | Scope to one spec (spec 039), writing `<derived>/attestation/by-spec/<id>.json`. Accepts the short id. |
| `--with-coupling` | Also record the coupling verdict. |
| `--snapshot` | Emit an authority snapshot instead (spec 070), writing `<derived>/attestation/snapshot.json`: which inputs were read, what they hashed to, whether the committed ledger matches the recompute, and every spec's territory digest. Cannot combine with `--spec` or `--with-coupling`. |
| `--sign` | Produce a detached Ed25519 seal over the attestation hash. |
| `--key <PATH>` | The signing key (32-byte seed, raw or hex). Required with `--sign`. |
| `--key-id <ID>` | Override the seal's key id. Defaults to the hex public key. |

Exit `0` means an attestation was written. It is a record, not a gate.

The document goes to the file, not to stdout: stdout carries a summary, and
redirecting it publishes prose rather than the attestation.

## verify-attestation

```
spec-spine verify-attestation [--spec ID] [--snapshot] [--recompute]
                              [--signature --public-key PATH] [--seal PATH]
                              [--attestation PATH] [--json]
```

| Flag | Meaning |
|---|---|
| `--recompute` | Re-read the corpus and check it reproduces the attestation. No key needed. |
| `--signature` | Check the detached seal against a supplied public key. |
| `--public-key <PATH>` | The Ed25519 public key (32 bytes, raw or hex). Required with `--signature`. |
| `--seal <PATH>` | The detached seal. Defaults to the attestation's sibling `.sig`. |
| `--attestation <PATH>` | The attestation file. Defaults to `<derived>/attestation/attestation.json`. |
| `--spec <ID>` | Verify the per-spec attestation for this id (spec 039). |
| `--snapshot` | Verify the authority snapshot (spec 070) instead. |

The verifier checks the bytes it was given (spec 068). A re-canonicalized copy
fails: unknown members, reformatting and duplicate keys are all refused, and a
payload whose schema MAJOR is unsupported is refused while a MINOR ahead of the
verifier verifies.

## compact

```
spec-spine compact --plan-file FILE [--plan] [--force] [--map-out FILE]
```

Rewrites the corpus under an authored compaction plan (spec 096): which specs
leave and which spec answers for each, whether the survivors' ordinals are
compacted, and which paths retire. `--plan` prints the map and the per-form
report and writes nothing; `--force` rewrites over a dirty tree; `--map-out`
defaults to `docs/corpus-map.md`.

## config show

```
spec-spine config show [--json]
```

The effective configuration: every default resolved, and the built-in bypass
floor merged with the adopter's list and attributed to its source. See
`docs/configuration.md` for what each key means.

## There is no `init`

The deleted site documented `spec-spine init` and a `--with-kit` flag. Both
were removed by spec 092: this repository ships a governance **engine**, not a
development environment. The retained producer is the library function
`spec_spine_core::scaffold_init_json`, a pure function that returns governance
starter files as data and writes nothing. The Statecraft CLI is the sole
initializer of a managed development environment.
