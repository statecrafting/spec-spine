# Documentation manifests

A documentation manifest records what one generator invocation says it read
and produced (spec 160). It binds one complete repository context packet (spec
159), declares repository-local inputs by their selected-content identity (spec
155), interface pins (spec 110) and opaque external references, names the
generator, and divides each output into generated and human-authored regions,
with optional executable-example identities.

spec-spine does two things with one, and nothing else:

- **validate** it against the snapshot it records: the packet pages, every
  input's digest, every output's bytes and every region's span and digest; and
- **evaluate its freshness** against a current snapshot: one finding for every
  declared identity that moved.

Neither runs the generator or an example, fetches a producer, or judges whether
the documentation is true. `fresh` means the declared inputs and outputs still
match their recorded digests and structure. It does not mean the prose is
correct, that the declaration was exhaustive, or that anything was accepted.

## The manifest

```json
{
  "schemaVersion": "1.0.0",
  "packet": {
    "schemaVersion": "1.0.0",
    "snapshot": { "repository": "example/repo", "revision": "...", "tree": "...", "dirtyState": "clean-working-tree" },
    "requestDigest": "sha256:...",
    "closureDigest": "...",
    "pageDigests": ["sha256:..."],
    "pageCompleteness": ["complete"]
  },
  "inputs": [
    {
      "inputId": "alpha",
      "identity": "file:notes/a.txt",
      "projection": "full",
      "schemaVersion": "0.10.0",
      "digest": "sha256:...",
      "requirement": "required",
      "uses": ["out"]
    }
  ],
  "interfaces": [],
  "externalReferences": [],
  "generator": {
    "name": "docgen",
    "version": "1.2.3",
    "buildDigest": "sha256:...",
    "arguments": ["--all"],
    "configurationDigest": "sha256:..."
  },
  "outputs": [
    {
      "outputId": "out",
      "path": "docs/out.md",
      "mediaType": "text/markdown",
      "digest": "sha256:...",
      "byteLength": 61,
      "regions": [
        { "regionId": "intro", "kind": "generated", "span": { "startLine": 1, "endLine": 2 }, "digest": "sha256:...", "inputIds": ["alpha"] },
        { "regionId": "notes", "kind": "human-authored", "span": { "startLine": 3, "endLine": 3 }, "digest": "sha256:...", "inputIds": [] }
      ],
      "examples": []
    }
  ],
  "omissions": [],
  "manifestDigest": "sha256:..."
}
```

Every member is required and every collection is explicit, even when empty;
an unknown member is a usage error. The rules a manifest must satisfy:

- **One packet.** `packet` names one complete spec 159 page set: every page
  repeats one snapshot, request and closure, every page but the last carries a
  continuation, member keys strictly increase across pages, and no page omits a
  required member. `pageCompleteness` is `partial` for each page that continued
  and `complete` for the last.
- **Inputs are packet members.** A local input carries the selected-content
  identity, projection, schema version and digest the packet holds for it. An
  optional input the packet omitted goes under `omissions` with the packet's
  exact reason, never under `inputs`. A required input the packet lacks refuses.
- **Interface inputs** reuse the spec 110 pin (`corpus`, `spec`, `digest`,
  pinned `sections`). **External references** carry a `namespace`,
  `schemaVersion`, `identity`, `digest` and a `purpose` of `evidence`, `policy`
  or `toolchain`, and nothing about admission, lifecycle or disposition.
- **The generator** is an identity record: `buildDigest` and
  `configurationDigest` are required, and `arguments` are literal strings,
  redacted by the author before the manifest is built.
- **Regions cover every byte.** Output text is normalized (BOM stripped, CRLF
  and CR folded to LF); its digest and `byteLength` are over that text. Regions
  are 1-based inclusive line spans that abut from line 1 to the last line, with
  no gap or overlap, and a region's digest is over its lines' bytes, terminators
  included. A `generated` region cites at least one input. Binary or non-UTF-8
  outputs are unsupported.
- **Citations agree.** An input's `uses` names exactly the outputs whose regions
  cite it and the examples that list it. Every cited id exists.
- **Examples are declarations.** An example names its region, argument vector,
  working directory, inputs, expected exit class (`ok`, `finding`, `refused`,
  `usage`, `failed`) and optional stream digests. Nothing runs it.
- **Paths are contained.** Every output path is normalized and lies beneath the
  caller's output root; `.git`, and the configured derived and state
  directories unless the root is inside them, are refused, as is a link out of
  the root.

Collections are ordered bytewise by identity before bytes are written or
digested, so authored order never changes the digest; generator and example
argument vectors keep their order. `manifestDigest` is `sha256:` over the
canonical manifest with that member absent. A manifest holds at most 4096
local inputs and omissions, 1024 interfaces, 1024 external references, 256
outputs, 4096 regions and 2048 examples, in at most 4 MiB of canonical JSON.
There is no pagination: an excess is refused, never truncated.

## Freshness

Freshness re-reads every declared identity against one clean current snapshot,
the current pages for the recorded request, the interface exports and external
resolutions the caller supplies, and the current output bytes. Each finding
says which reason applies, decided by the contract that owns the identity:

| Identity | Reason |
|---|---|
| local input | `changed` when its digest moved; otherwise the spec 159 omission reason: `missing`, `removed`, `withdrawn`, `unresolved`, `ambiguous`; `unsupported` for an unsupported selector, projection or non-text; `unverified` for a budget omission; `missing` when absent with no omission |
| interface input | the spec 110 result: `changed` (`stale` or `sections-current`), `missing`, `unverified` (no export for its corpus) |
| external reference | `unverified` with no matching resolution, `changed` when the resolved digest differs |
| packet | `changed` when the page digests differ at the recorded snapshot |
| output | `output-changed` when its digest moved, `missing` when absent, `unsupported` when not text |
| region of a changed output | `region-shifted` when its bytes moved to another span, `changed` when they changed |

A finding carries `kind`, `identity` (`input:<id>`, `packet`, `output:<id>`,
`region:<output>#<region>`), `requirement`, the recorded digest, the current
digest when there is one, and `affects`. It never carries content.

The `status` is `incomplete` when a required identity is missing, removed,
withdrawn, unresolved, ambiguous or unsupported; else `stale` when anything
changed, shifted, or an optional identity is absent; else `unverified` when an
identity could not be checked; else `fresh`. The result repeats the recorded
and current snapshots and says whether the tree changed: a changed tree whose
declared slice is unchanged is `fresh` with `repositoryTreeChanged: true`,
which says nothing about the rest of the repository.

## The CLI

```text
spec-spine documentation manifest validate --manifest <FILE> --packet <FILE>... --repository <IDENTITY> [--revision <REV>] [--output-root <DIR>] --json
spec-spine documentation manifest freshness --manifest <FILE> --packet <FILE>... --repository <IDENTITY> [--revision <REV>] [--output-root <DIR>] [--export <CORPUS>=<DIR>]... [--resolutions <FILE>] --json
```

Both bind the clean working tree at `HEAD`, or one exported revision, as
`content select` and `context packet` do, read the outputs from that tree, and
refuse when it moved during the read. `validate` prints the canonical manifest
at exit `0`. `freshness` prints the result at exit `0` when `fresh`, and
otherwise exits `1` with the `documentation.manifest.freshness` verdict
envelope, whose `report` is the result. Malformed documents, unknown members,
duplicate ids, invalid digests or regions and limit excesses exit `3`; a dirty
or moving snapshot, a mismatched packet, input or output, and a path escape
exit `2`.

The library entry points are listed in `docs/api.md`; the schema is
`schemas/documentation-manifest.schema.json` on its own axis
(`docs/schema-versioning.md`).
