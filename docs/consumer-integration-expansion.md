# Consumer integration: the expansion wave after 0.22.0

What a library or CLI consumer needs in order to use specs 102, 103, 105, 106,
107, 109 and 110, which are merged on `main` **after** the frozen 0.22.0
candidate (`f9fa6a8f`). None of it is in 0.22.0 and none of it is published.
Every contract below is exact as of the `main` revision named in §8. The
design rationale lives in each spec; this page is the integration surface.

## 0. Prerequisites that apply to all of it

- **Producer version.** A binary or library from `main` at or after the
  revision in §8. Until the next release is cut it reports `0.22.0`, which is
  also the frozen candidate's version, so **the version string does not tell
  the two apart**: identify the producer by source revision or package digest.
- **Schema versions** (compile-time constants in
  `spec-spine-types/src/version.rs`):

  | Axis | 0.22.0 candidate | `main` | Moved by |
  |---|---|---|---|
  | registry (`specVersion`) | `1.3.0` | `1.6.0` | 106 (`1.4.0`), 109 (`1.5.0`), 110 (`1.6.0`) |
  | read documents (`schemaVersion`) | `0.1.0` | `0.6.0` | 102, 106, 107, 109, 110 (one MINOR each) |
  | verifier fixture set | none | `0.1.0` | 103 |

  Every move is additive (MINOR). A loader that knows MAJOR 1 of the registry
  keeps working; a binary predating a MINOR that meets its new member refuses
  with exit 3 rather than guessing.
- **The facade is `&str -> Result<String, Error>`**; `Error::exit_code()` is
  the CLI's exit code (`1` not found / refused, `2` stale, `3` usage, parse,
  I/O, config). The `config_json` argument is the `spec-spine.toml` model in
  snake_case; every other request and every emitted document is camelCase, and
  all of them refuse unknown members.
- **Stale ledgers.** The read verbs that compose identities (`registry
  closure`, `interface verify`) refuse a stale committed registry with exit 2
  before resolving anything. `registry show` and `registry obligation` are
  inspection reads and answer from whatever is committed; do not build an
  identity from them without a freshness check (`spec-spine check`).
- **Local verification only** so far: the example in §7 packages the library
  from source. Registry-backed verification waits for a published release.

## 1. Spec 102: a ready entry carries its status

- **Library:** `plan(&Registry) -> Plan`; `ReadySpec { id, status, title }`.
- **Facade:** `query_json({"registry", "op": "plan"})`.
- **CLI:** `spec-spine registry plan --json`.
- **Contract:** `ready[].status` is the spec's `status` verbatim. Readiness is
  **scheduling, not approval** (spec 101): a `draft` can be ready. A consumer
  that only works approved specs filters on `status` itself, from this one
  document, with no second query.

## 2. Spec 103: the verifier fixture set

- **Where:** `fixtures/verifier/` inside the packaged `spec-spine-core` crate
  (and in the repository at `crates/spec-spine-core/fixtures/verifier/`).
- **Shape:** `index.json` (`schemaVersion`, `cases`, `payloadTypes`,
  `reasons`); per case, `case.json` (`expect: { outcome, exit, reason? }`,
  `needsCorpus`), `payload.json` (the exact bytes), and `corpus/` when the
  outcome needs a recompute.
- **Contract:** each case's `expect` is what this repository's own verifier
  answers, asserted in `verifier_fixtures.rs` on every build. A consumer
  verifier reproduces the outcome and reason; how it reports them is its own.
  The version-mismatch case stays a version mismatch: a regeneration after a
  registry MINOR moves `registryHash` and `attestationHash` only.
- **Replay through the facade:** `verify_attestation_json({"repoRoot":
  <case corpus>, "attestationText": <payload.json text>})`. `Ok` answers
  `match`, `versionMismatch` or `contentMismatch`; a refusal before the
  recompute is `Err` with the recorded exit code. §7 replays all eleven.

## 3. Spec 105: governed scope

- **Surface:** configuration only, `[coverage] governed_scope` and
  `governed_scope_exclusions` (spec 078's keys), read by `index coverage` and
  by `couple`'s `C-002` when `[coupling] require_ownership = true`.
- **What 105 changes:** this repository turns the key on over the files it
  already hashes, and claims the four that were unowned. For an adopter the
  capability is the demonstrated recipe: list governance paths outside every
  package (scripts, hooks, schemas, root docs) in `governed_scope`, and an
  unowned new one is refused. The refusal gained was measured (105 D-7): a new
  unclaimed script fails `index coverage --fail-on-untraced` with the scope on
  and passes with it off.
- **Compatibility:** no engine, schema or CLI change. The bypass floor and
  `bypass_prefixes` are unchanged. Editing `spec-spine.toml` restales every
  index shard.

## 4. Spec 106: obligations and section digests

- **Authoring:** `obligations: [{ id, kind: requirement|invariant|verification,
  text, anchor, inputs?, withdrawn? }]` in frontmatter. `inputs` only on a
  `verification`. A withdrawn obligation stays declared with `withdrawn: true`;
  reusing its id is a compile error, so an id never changes meaning.
- **Registry:** `obligations` (verbatim) and `sectionDigests` (anchor to hex
  SHA-256 over `<specPath>#<anchor>`, NUL, normalized section lines) on every
  record.
- **Library / facade / CLI:** `resolve_obligation`; `query_json({"registry",
  "op": "obligation", "id": "<spec>#<id>"})`; `spec-spine registry obligation
  <spec>#<id> --json` (adds the spec's `contentHash`). An unqualified id is
  exit 3; an unknown one exit 1.

## 5. Specs 107 and 109: closures and impacts

- **107, closures.** Request `{ specs?, sections?: [{spec, anchor}],
  obligations?: ["<spec>#<id>"], rationale? }`. `closure_json(config, root,
  request)` or `spec-spine registry closure --request <file|-> --json` returns
  `members` (each with its identity) and one `digest`. The digest ignores
  order, duplicates and short ids, moves when any named member's content
  moves, and never moves for a section nobody named. Refusals: empty or
  unqualified request exit 3; any unresolved member exit 1 (all named, no
  digest); stale ledger exit 2; a member whose section digest is absent is
  refused, so a gap cannot enter a digest that looks complete.
- **109, impacts and conflicts.** Authoring `impacts: [{ obligation:
  "<spec>#<id>", nature: refines|extends|supersedes|informs, successor?,
  note? }]` and `conflicts: [{ obligation, reason, resolution:
  deliberate|unresolved|pending, settled_by? }]`. The read,
  `query_json({"registry", "op": "impacts", "target"?, "declaredBy"?})` or
  `spec-spine registry impacts [--target] [--declared-by] --json`, inverts
  every declaration onto its target, sorted by `(target, declaredBy)`, with
  `targetWithdrawn` when the obligation was withdrawn. Declared, never
  computed: an impact set is what authors said, not a blast radius, and no
  gate reads it. `lint` warns (L-014) once per `unresolved` conflict.

## 6. Spec 110: digest-pinned interface references

- **Authoring:** `interface_references: [{ corpus, spec, digest, sections?:
  [{ anchor, digest }], obtained, rationale? }]`. `corpus` is a name, not a
  URL; `spec` a full id; digests are `sha256:` + 64 lowercase hex copied from
  the cited corpus's `registry show <id> --json` (`contentHash`,
  `sectionDigests`). No placeholder form exists. Malformed members are
  compile errors V-032 to V-038.
- **Library:** `verify_interface_references(&Registry, &Exports,
  Option<&str>)` (pure); `interface_verify(&Config, root, &BTreeMap<corpus,
  dir>, Option<&str>)` adds the committed-ledger read and `load_export`.
- **Facade:** `interface_verify_json({"registry", "exports": {corpus: {spec:
  {path, text}}}, "spec"?})`. `path` must be the cited spec's repo-relative
  path **in the exporting corpus**; it is part of the hash.
- **CLI:** `spec-spine interface verify [--export <corpus>=<dir>]... [--spec
  <id>] [--json]`.
- **Outcomes:** `current` and `sections-current` exit 0; `stale`, `missing`,
  `unverified` exit 1; stale local ledger exit 2; malformed `--export`,
  unreadable directory or a spec linked out of its export root exit 3.
- **Boundary:** nothing fetches, discovers or trusts a corpus. Which checkout
  is authoritative is the caller's decision, and the observed digest is
  printed for a human to copy; nothing rewrites a pin.

## 7. A working example

`docs/examples/expansion-consumer/` is a standalone crate that uses only the
surfaces above. `run.sh` packages `spec-spine-types` and `spec-spine-core` with
`cargo package`, unpacks them, builds the consumer against them off the
workspace lockfile, builds the CLI from the same tree, and runs:

```bash
docs/examples/expansion-consumer/run.sh
```

It writes two disposable corpora (an exporter with obligations and impacts,
and an importer pinning it), writes their ledgers with the CLI, and reads
everything through the library facade:

```
ok: 102 plan entries carry status (a draft is ready; approval is the consumer's rule)
ok: 106 obligation resolves; its section digest equals show's; unqualified id exits 3
ok: 109 impacts inverted onto 001-a, the withdrawn target reported as withdrawn
ok: 107 closure digest normalized; a missing member exits 1 with no digest
   110 unchanged: ["current", "current"]
   110 3.2 edited: ["stale", "sections-current"]
   110 3.1 edited: ["stale", "stale"]
ok: 110 current / sections-current / stale / unverified, and the closure moved with the section
ok: 103 all 11 fixture cases reproduce their recorded outcome
```

The pins it uses are the exporter's own identities, reached through supported
reads: a closure member's `contentHash` for the whole spec and the registry
record's `sectionDigests` entry for the section.

## 8. Evidence and its limits

Recorded against the merged revision in `docs/release-candidate-0.22.0.md`
§14, which also separates this content from the frozen 0.22.0 candidate.
Everything here is **local source/package verification**: the crates were
packaged from a checkout, not downloaded from a registry.

## 9. For Statecraft

Nothing in Statecraft needs to change to keep consuming 0.22.0. To adopt this
wave after it is released:

1. Pin the release that carries it; do not rely on the version string before
   then (§0).
2. Readiness (102): filter `plan.ready` on `status` to apply the approval rule
   in one read.
3. Closures (107): record the `digest` with the work it authorized; a later
   resolve with a different digest means a named member moved. Resolve only
   against a fresh ledger (exit 2 otherwise).
4. Interface pins (110): for any Statecraft spec that cites a producer spec,
   copy `contentHash` (and the sections relied on) from the producer's
   `registry show --json`, and run `interface verify --export
   <producer>=<checkout>` in the release job. Fetching and choosing the
   checkout stay with Statecraft.
5. Fixtures (103): an independent verifier replays `fixtures/verifier/` from
   the packaged crate, as §7 does.
