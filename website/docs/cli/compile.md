---
id: compile
title: spec-spine compile
sidebar_position: 3
---

# spec-spine compile

Validates spec frontmatter and emits the deterministic registry.

## Usage

```bash
spec-spine compile [--check]
```

## Description

The compiler reads the markdown spec corpus (`specs/*/spec.md`), validates the YAML frontmatter, and emits the spec-as-source view.

The output is written as per-unit registry shards to `.derived/spec-registry/by-spec/<id>.json`. The output is deterministic: the same inputs produce byte-identical output on every platform.

## `--check`

`--check` is the non-writing form, the registry counterpart of [`index check`](./index.md). It compiles in memory and compares the result byte-for-byte against the committed shards, reporting three kinds of drift:

- **modified**: a committed shard whose bytes differ from the compiled one (a `spec.md` was edited without recompiling).
- **missing**: a spec with no committed shard (a spec was added without recompiling).
- **orphaned**: a committed shard with no matching spec (a spec was removed without recompiling).

Use it in CI if you commit `.derived/`. It never writes: no shard, no `build-meta.json`, no pruning.

:::warning
Do not run `compile --check` after a plain `spec-spine compile` in the same job. The check would compare the committed shards against files that run just overwrote, and would pass unconditionally. `--check` compiles on its own, so it replaces the plain `compile` step rather than following it.
:::

## Exit Codes

- `0`: Validation passed and registry written (or, with `--check`, every shard matches).
- `1`: Validation failed (e.g., malformed frontmatter, invalid edge). With `--check`, validation outranks staleness: a corpus that does not validate cannot vouch for its shards.
- `2`: `--check` only. Validation passed but one or more committed shards are stale. A registry that was never built is stale, not an error.
- `3`: I/O, parse, schema, or config error.

## Example

```bash
$ spec-spine compile
Compiled 42 specs to .derived/spec-registry/

$ spec-spine compile --check
spec-registry is fresh: 42 shard(s) match the corpus

$ spec-spine compile --check      # after editing a spec.md
1 stale shard(s): modified 017-directory-crate-module-units.json
spec-registry is STALE: run `spec-spine compile` and commit the result
```
