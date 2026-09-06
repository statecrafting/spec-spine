---
id: overview
title: CLI Overview
sidebar_position: 1
---

# CLI Overview

The `spec-spine` command-line tool is a thin translation of `spec-spine-core` results into stdout/stderr and stable exit codes. All `process::exit`, stdout, and `git` side effects live here; the engine stays pure.

## Global Flags

| Flag | Value | Default | Effect |
|---|---|---|---|
| `--repo <DIR>` | Path | `.` (current directory) | Selects the repository root. |

## Exit Codes

Exit codes are a stable contract across the entire CLI surface:

| Code | Meaning |
|---|---|
| `0` | Success / OK. |
| `1` | Validation failure, not-found, or coupling drift. |
| `2` | Staleness (a committed registry or index shard is out of date). |
| `3` | I/O, parse, schema, or config error. |

## Command Surface

| Command | Capability |
|---|---|
| [`spec-spine init`](init.md) | Scaffold a new adopter (config, standards, specs/000, rules). |
| [`spec-spine compile`](compile.md) | Validate frontmatter and emit the deterministic registry. |
| [`spec-spine index`](index.md) | Scan manifests and specs to emit the codebase index. Includes `check`, `render`, and `orphans` subcommands. |
| [`spec-spine registry`](registry.md) | Typed read-only queries against the compiled registry. Includes `list`, `show`, `status-report`, `relationships`, and `plan` (the ready set). |
| [`spec-spine lint`](lint.md) | Check corpus conformance. |
| [`spec-spine couple`](couple.md) | The PR-time drift gate. |
| [`spec-spine attest` / `verify-attestation`](attest.md) | A signed, reproducible record of the gate verdicts over the corpus or one spec (`--spec`), and its verification. |

## Machine-readable verdicts (`--json`)

The verbs that render a **verdict** (`compile --check`, `index check`, `lint`, `couple`, `attest`, `verify-attestation`) take `--json` (spec 037). Under the flag a verb writes exactly one JSON object to stdout:

```json
{
  "schemaVersion": "0.1.0",
  "verb": "couple",
  "ok": false,
  "exitCode": 1,
  "report": { }
}
```

- `verb` is the stable dotted command path (`compile.check`, `index.check`, `lint`, `couple`, `attest`, `verify-attestation`).
- `ok` always equals `exitCode == 0`.
- `report` carries the verb's library facade output verbatim (the same bytes the matching `*_json` function returns), so there is one payload shape per verb.
- On failure `error` replaces `report`: `{ "kind", "message" }`, where `kind` is one of `config`, `validation`, `not-found`, `stale`, `io`, `parse`, `schema`. Exactly one of `report` / `error` is present.

**The flag changes what is written, never what is decided.** Every exit code is identical with and without it, so a driver reads the exit code for control flow and the envelope for reasons, and the two can never disagree. The envelope carries its own `schemaVersion` (`VERDICT_SCHEMA_VERSION`, independent of the registry and index versions) under the usual MINOR-is-additive discipline.

`spec-spine compile` (the writing form) deliberately has no `--json`: it mutates `.derived/`, and the verdict that is machine-readable is `compile --check`.
