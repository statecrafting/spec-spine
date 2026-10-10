# Context packets

A context packet is one ordered, digestible bundle of governed text from one
immutable repository snapshot (spec 159). It composes two existing contracts
and changes neither:

- a **context closure** (spec 107) names the specs, sections and obligations a
  piece of work depends on; and
- **selected content** (spec 155) returns bounded normalized text for an
  explicit selector.

The packet adds composition only: whether each member is required, where it
came from, what was omitted and why, whether the result is complete, how to
resume after a budget, and two digests.

## Request

```json
{
  "closure": { "root": "155-selected-content-accessor" },
  "members": [
    {
      "selector": { "kind": "file", "path": "docs/api.md" },
      "requirement": "optional"
    }
  ],
  "maxBytes": 524288,
  "maxItems": 96,
  "continuation": null,
  "rationale": null,
  "workIdentity": null,
  "consumerSchemaVersion": "1.0.0"
}
```

- `closure` names exactly one of `root` (a spec id, resolved as the closure of
  that one spec) or `digest` together with the closure `document` it names.
  The producer re-resolves the document against the snapshot and refuses it
  (`closure-mismatch`) unless the result is byte-equal and carries that digest.
  It never discovers a closure by digest.
- `members` are spec 155 selectors, each `required` or `optional`. The
  selector's own `required` flag is left unset: requirement belongs to the
  packet.
- `maxBytes` (1 to 4194304, default 524288) and `maxItems` (1 to 512, default
  96) bound one page. Only member `content` bytes spend the byte budget.
- `rationale` and `workIdentity` are opaque audit strings of at most 1024
  bytes. They have no effect on selection or authority.
- `consumerSchemaVersion` is the packet schema the caller reads. Another major,
  or a minor above the producer's, is refused.

Unknown members are usage errors.

## Members, order and deduplication

Every closure member becomes the most specific selector that keeps its
identity (a spec, a section, or a qualified obligation) and is **required**: it
is the declared context. A withdrawn obligation has no current text and is
omitted as `withdrawn`.

A member's key is its spec 155 identity plus its projection. Two requests for
one key collapse: `required` wins and both origins are kept, `closure` before
`additional`. Equal bytes under two identities stay two members.

Members are ordered by identity, then projection name, bytewise. The order of
the request's `members` never changes the packet or its digests.

## Omissions, warnings and completeness

An omission names the member's key, requirement and origins, and exactly one
reason:

| Reason | Emitted when |
|---|---|
| `missing` | The named content does not exist in this snapshot. |
| `unresolved` | A symbol, module or owned unit does not resolve. |
| `withdrawn` | A closure obligation is withdrawn. |
| `unsupported-selector` | The selector kind has no resolver (a test selector, for example). |
| `unsupported-projection` | The selector does not support the projection. |
| `non-text` | The content is not UTF-8 text. |
| `oversized-member` | The member alone is larger than `maxBytes`, so no page could hold it. |
| `removed`, `ambiguous`, `item-budget`, `byte-budget` | Reserved in schema 1.0 and never emitted by this producer (spec 159 D-9). |

An ambiguous selector, a path escape, a stale ledger, a changed snapshot and an
invalid continuation are **refusals**, not omissions. No packet is returned.

Warnings are typed and never decide completeness:
`unverified-producer-build`, `closure-member-unsupported`, and
`optional-member-omitted`.

`completeness` is:

- `complete`: every required member is present and no page remains;
- `partial`: no required member is omitted, and a `continuation` remains; or
- `incomplete`: a required member is omitted. The CLI exits `1` and reports the
  whole packet inside the `context.packet` verdict envelope.

Omissions do not depend on the page. Every page of one packet carries the same
omissions, so any page tells the caller whether the whole packet is usable.

## Pages and continuation

A page holds whole members up to `maxItems` and `maxBytes`. When the next member
would not fit, the page ends and carries a `continuation`: unpadded base64url
over a payload naming the schema major, the snapshot, request and closure
digests, both budgets, and the last emitted key, plus a SHA-256 checksum of that
payload. The next request repeats the original with `continuation` set. It
resumes strictly after the last key, repeating nothing and skipping nothing.

The continuation is integrity checked, not secret, and carries no server state.
One bound to another request, snapshot, closure, schema major or budget is
refused as `stale-continuation`. One that does not decode or whose checksum does
not match is a usage error.

## Digests

- `requestDigest` is `sha256:` over the canonical normalized request: members
  sorted and exact duplicates removed, and no continuation.
- `closureDigest` is the spec 107 closure digest, reused unchanged.
- `packetDigest` is `sha256:` over the canonical packet with `packetDigest`
  absent and `continuation` present. Two pages have two digests. A consumer may
  record the ordered page digests; spec-spine defines no aggregate.

The document is canonical JSON (sorted keys, two-space indent, LF, one trailing
newline) under its own `schemaVersion` axis. Its schema is embedded as
`CONTEXT_PACKET_SCHEMA`. The typed API, the JSON facade and the CLI write the
same bytes for the same inputs and producer build.

## Producer identity

`producer` names the package, its version, and a build: `sha256:<hex>`
supplied by the binding, or `unverified`. The CLI hashes its own executable.
The JSON facade has nothing to hash and records `unverified`, with a warning.
Core never infers its own build.

## Trust boundary

Packet content is **untrusted repository data**. spec-spine selects, contains
and budgets it. It does not follow links, execute examples, or treat text that
claims authority as instructions. A packet reads only the named export through
the existing containment and exclusion rules, never ignored files, the
environment, Git remotes or credentials. Error detail names identities and
reasons and never echoes content.

A packet proves deterministic assembly from declared inputs, and nothing more.
It does not prove correctness, approval, evidence admission, acceptance,
release, adoption, or freshness anywhere else. One packet covers one
repository. Composing packets across repositories, and deciding what a caller
may disclose, belong to the consumer (Statecraft).
