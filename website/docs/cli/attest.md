---
id: attest
title: spec-spine attest
sidebar_position: 8
---

# spec-spine attest / verify-attestation

A signed, reproducible record of what the corpus (or one spec) looked like and
what the gates said about it. `attest` writes the record; `verify-attestation`
checks that a record still reproduces from the tree, or that its detached seal
verifies against a public key.

## Usage

```bash
spec-spine attest [--with-coupling] [--sign --key FILE [--key-id ID]] [--json]
spec-spine attest --spec <id> [--sign --key FILE] [--json]
spec-spine verify-attestation --recompute [--attestation FILE] [--json]
spec-spine verify-attestation --signature --public-key FILE [--seal FILE] [--json]
spec-spine verify-attestation --spec <id> --recompute [--json]
```

## Description

### Corpus scope (spec 023)

`attest` compiles the corpus in memory, records the compile, lint and (with
`--with-coupling`) coupling verdicts together with the registry content hash,
and writes `<derived_dir>/attestation/attestation.json`. The payload is a pure
function of the tree: no clock, no signer, no environment. With `--sign` the CLI
adds a detached Ed25519 seal (`attestation.json.sig`) over the attestation hash.
The seal, not the payload, is where the key and any timestamp live, so the
payload stays byte-reproducible by anyone.

### Per-spec scope (spec 042)

`attest --spec <id>` narrows the subject to one spec and writes
`<derived_dir>/attestation/by-spec/<id>.json`. The payload carries:

- `specSourceHash`: the normalized `spec.md` bytes.
- `lifecycle`: the spec's declared `status` and `implementation`.
- `units`: every **owning** unit, in registry order, with the content hash of
  what it resolved to. Non-owning `references` units are excluded.
- `verdicts`: `compile`, `resolution` (did every owning unit resolve) and `lint`
  (with a hash over that spec's canonical findings).

This is the artifact an autonomous builder hands over when it says a spec is
done: the yardstick records the claim and the evidence, the builder does not
grade itself. **A failing verdict still yields a payload.** `attest` is a record,
not a gate, so exit `0` means only that an attestation was written. Read
`verdicts` (or the `--json` envelope) for the outcome. No lint rule consumes a
per-spec attestation, so the record can never feed back into the verdict it
records.

Neither file is committed: `.derived/attestation/` is gitignored. A per-spec
attestation hashes every claimed unit's bytes, so committing it would restale on
every code edit and would need its own freshness gate.

### `verify-attestation`

- **`--recompute`** re-reads the corpus and checks that the attestation
  reproduces, keyless. A `tool.version` mismatch is reported as its own outcome
  (`versionMismatch`), never as a false content mismatch.
- **`--signature`** checks the detached seal against `--public-key`.
- Both modes accept `--spec <id>` to verify the per-spec record instead.

## Flags

| Flag | Verb | Effect |
|---|---|---|
| `--spec <id>` | both | Scope to one spec (`by-spec/<id>.json`). |
| `--with-coupling` | attest | Also record the coupling (specs-and-code-in-sync) verdict. |
| `--sign` / `--key <PATH>` | attest | Produce a detached Ed25519 seal. The key is a 32-byte seed, raw or hex. |
| `--key-id <ID>` | attest | Override the seal's key id (defaults to the hex public key). |
| `--recompute` | verify | Recompute from the tree and compare. |
| `--signature` / `--public-key <PATH>` | verify | Check the seal. The key is 32 bytes, raw or hex. |
| `--attestation <PATH>` / `--seal <PATH>` | verify | Override the default file locations. |
| `--json` | both | Emit the [verdict envelope](./overview.md#machine-readable-verdicts---json) on stdout. |

## Exit Codes

- **`attest`:** `0` an attestation was written (regardless of the verdicts it
  records) · `1` the spec id was not found · `3` I/O, parse, schema, or config
  error.
- **`verify-attestation`:** `0` the attestation reproduces / the seal verifies ·
  `1` verification failed · `3` I/O, parse, schema, or config error.

## Example

```bash
$ spec-spine attest --spec 042-per-spec-attestation
attested 042-per-spec-attestation -> .derived/attestation/by-spec/042-per-spec-attestation.json
  attestationHash: cf10f56d53f98d2ba12c7b60510c6f089cb219458965be2f32669cd4761e2a97

$ spec-spine verify-attestation --spec 042-per-spec-attestation --recompute --json
{
  "exitCode": 0,
  "ok": true,
  "report": {
    "outcome": "match"
  },
  "schemaVersion": "0.1.0",
  "verb": "verify-attestation"
}
```
