# Consumer integration: the expansion wave, released as 0.23.0

What a library or CLI consumer needs in order to use specs 102, 103, 105, 106,
107, 108, 109, 110 and 113, merged on `main` **after** the frozen 0.22.0
candidate (`f9fa6a8f`) and **published as 0.23.0** on 2026-09-23 from
`d2bb47634404b874ca36cf8bf75b4a31a70328f7` (`docs/release-0.23.0.md`). The
0.22.0 candidate was never published. Every contract below is exact as of that
revision. The design rationale lives in each spec; this
page is the integration surface. Specs 122, 123 and 125 correct this
repository's own commit hook, session hooks and `verify` forwarding, and spec
124 gives the line its own version; §0 says what each means for a consumer.

## 0. Prerequisites that apply to all of it

- **Producer version.** The released producer is **`0.23.0`** (tag
  `v0.23.0`, source `d2bb4763`). The version string alone still does not name
  a build: a development build of a later `main` may answer the same version
  until the next bump. Identify a producer with
  `scripts/reader-identity.sh <binary> [<checkout>]`, which records its path,
  SHA-256, `--version`, the checkout's revision, whether it is that checkout's
  current build, and each schema axis **as the binary emits it**.
- **A reader older than the corpus refuses by name.** This repository's
  `[meta] required_version` is `>=0.23.0` (spec 124, per 061 §3.8). An adopter
  whose corpus uses a member introduced here (obligations, impacts, interface
  references) should raise its own floor the same way. An older binary then
  exits 3 naming the requirement instead of reporting a valid corpus
  `INVALID`.
- **Schema versions** (compile-time constants in
  `spec-spine-types/src/version.rs`):

  | Axis | 0.22.0 candidate | `main` (0.23.0) | Moved by |
  |---|---|---|---|
  | registry (`specVersion`) | `1.3.0` | `1.6.0` | 106 (`1.4.0`), 109 (`1.5.0`), 110 (`1.6.0`) |
  | read documents (`schemaVersion`) | `0.1.0` | `0.7.0` | 102, 106, 107, 109, 110, 108 (one MINOR each) |
  | verdict envelope (`schemaVersion`) | `0.5.0` | `0.6.0` | 113 (`couple`'s `waivers`) |
  | index | `1.1.0` | `1.1.0` | unchanged |
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
- **Verified from the registries.** The example in §9 was run against the
  crates published to crates.io, with the CLI installed from crates.io and a
  fresh Cargo home (`docs/release-0.23.0.md` §5).

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
  recompute is `Err` with the recorded exit code. §9 replays all eleven.
  The set records the producer's version: it was regenerated at 0.23.0
  (spec 124 §3.5), which moved only `toolVersion`, the payload `version`,
  `registryHash` and `attestationHash`.

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

## 7. Spec 108: a work scope is evaluated, never enforced

- **The document** (the consumer holds it, as a closure): `{ "id"?, "ownSpec",
  "mutable"?: [path], "shared"?: [{ "path", "with": [spec] }], "readOnly"?:
  [path] }`. Paths are repo-relative; a trailing `/` names a subtree. At least
  one path; no absolute path, `..`, empty entry, unknown member, or one path
  (or a subtree and a path inside it) under two roles, each exit 3.
- **Evaluate:** `scope_json(config, root, scope)` or `spec-spine scope
  evaluate --scope <file|-> [--json]`. Each path's owners come from the
  committed index by the gate's own owner derivation. Findings are **warnings,
  exit 0**: `S-001` a changed path nobody owns, `S-002` an undeclared crossing
  (a `mutable` path another spec owns, naming both), `S-003` a `shared` path
  whose `with` differs from its owners. `readOnly` raises nothing. The answer
  carries `indexHash`. An unknown `ownSpec` or `with` spec is exit 1 (all
  named); a stale index is exit 2, before anything resolves.
- **Compare:** `scope_compare_json(a, b)` or `spec-spine scope compare <A> <B>
  [--json]`, pure over two documents: `both-mutable`, `mutable-shared` and
  `changed-under-read` conflicts, with subtree overlap. Two `shared` or two
  `readOnly` declarations do not conflict. Exit 0 either way.
- **Boundary:** a scope **reserves, excludes, locks, permits and enforces
  nothing**. `couple`, `check`, `lint` and `index coverage` never read one.
  Deciding which of two conflicting scopes proceeds is the orchestrator's.

## 8. Spec 113: a waiver's lifecycle over the caller's inputs

- **Declaration:** under a `Spec-Drift-Waiver: <reason>` line in the pull
  request body, optional lines narrow that waiver: `-Paths:` (a
  comma-separated scope; a trailing `/` is a subtree), `-Until:`
  (`YYYY-MM-DD`, inclusive), `-Since:` (a commit) and `-Max-Uses:` (a positive
  integer), each spelled from the configured keyword. Several waivers may be
  declared in one body.
- **Inputs are the caller's:** CLI `--waiver-as-of <date>` (never the clock),
  `--waiver-uses <id>=<n>` (prior clearing runs, excluding this one), and
  ancestry answered by git for each `-Since:`. In the facade, `couple_json`
  takes `prBody` (or `waivers` as data) plus `waiverInputs: { asOf?,
  ancestry?: { commit: bool }, uses?: { id: n } }`. At most one of `waiver`,
  `waivers`, `prBody`; more is exit 3. A malformed `asOf` is exit 3.
- **Evaluation:** each declared check is `satisfied`, `failed` or
  `not-evaluated`. A missing input is `not-evaluated`, **never satisfied**, and
  a missing count is never zero. A waiver with a failed check clears nothing;
  a scoped waiver clears only its paths; each violation is cleared by the first
  effective waiver whose scope covers it.
- **The report** (verdict `0.6.0`): `waivers[]` with `id` (the use-count key),
  `reason`, `scoped`/`paths`, `checks[]`, `effective` and `clears[]`, plus
  `unattachedWaiverLines`. Both are omitted when no waiver is declared, so
  those runs keep their exact report bytes. `waiver` still means "this run was
  waived".
- **Boundary:** nothing is counted, consumed, stored or authorized. A use
  limit is only as good as the supplied count, and two concurrent runs given
  the same count see the same answer. `effective` says nothing about who
  approved the waiver, which remains a human instrument.

## 9. A working example

`docs/examples/expansion-consumer/` is a standalone crate that uses only the
surfaces above. `run.sh` packages `spec-spine-types` and `spec-spine-core` with
`cargo package`, unpacks them, builds the consumer against them off the
workspace lockfile, builds the CLI from the same tree, and runs:

```bash
docs/examples/expansion-consumer/run.sh
```

It writes three disposable corpora (an exporter with obligations and impacts,
an importer pinning it, and a small governed corpus with code), writes their
ledgers with the CLI, and reads everything through the library facade:

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
ok: 108 scope names the crossing, accepts declared sharing, refuses unknown and escaping inputs, and compares
ok: 113 scoped, expired, not-evaluated and spent waivers each report and clear as declared
ok: composed: scope crossing == gate refusal; a waiver scoped to it clears exactly it; closure and scope unchanged
```

The composed line is the one place the contracts meet in one flow. A work
order holds a closure (107, over an obligation from 106) and a scope (108).
The scope's `S-002` crossing is exactly the path `couple` refuses on the same
change. A waiver scoped to that path (113), with a supplied as-of date, clears
exactly it and nothing else. And neither the gate nor the waiver moves the
closure's digest or the scope's evaluation.

The pins it uses are the exporter's own identities, reached through supported
reads: a closure member's `contentHash` for the whole spec and the registry
record's `sectionDigests` entry for the section.

## 10. Evidence and its limits

Local package verification is recorded in `docs/release-candidate-0.23.0.md`
(at `97f82ee5`) and repeated at the released revision `d2bb4763` in
`docs/release-0.23.0.md` §3. Registry-backed verification (crates.io library
and CLI, npm, the PyPI wheel, the GitHub Release archives and their
provenance) is in its §4 and §5.

## 11. For Statecraft

Everything Statecraft needs to adopt this line, in one place. Nothing here
changes Statecraft, and nothing in Statecraft needs to change to keep
consuming 0.22.0.

**Producer: released.** `v0.23.0`, source revision
`d2bb47634404b874ca36cf8bf75b4a31a70328f7`, published 2026-09-23. Identify a
binary with `scripts/reader-identity.sh`, not `--version` (§0).

| Package | Identity |
|---|---|
| `spec-spine-core` 0.23.0 (crates.io) | `.crate` SHA-256 `3dca8f6819d7951110757fbafba31897a3ed6dad516a85599e05e334f6b3e492` |
| `spec-spine-types` 0.23.0 (crates.io) | `.crate` SHA-256 `dcd35073ecd95b9aa21b0173e0351fc44398d2a6019d6de73948d5287b319dc4` |
| `spec-spine-cli` 0.23.0 (crates.io) | `.crate` SHA-256 `6b0e780069a88d1a9f1ecf9d0d49cab1308bef0d44fbae63d1e5f3322975ca91` |
| `spec-spine@0.23.0` (npm) and `@spec-spine/cli-<os>-<cpu>@0.23.0` | integrities in `docs/release-0.23.0.md` §4 |
| `spec-spine` 0.23.0 (PyPI), wheels and sdist | SHA-256 in `docs/release-0.23.0.md` §4 |
| GitHub Release archives | SHA-256 sidecars and build provenance naming `d2bb4763`, `docs/release-0.23.0.md` §4 |

**What Statecraft uses today, and what moving to 0.23.0 means for it.** The
Statecraft CLI pins `spec-spine-core =0.21.0` with `default-features = false`
and calls only `scaffold_init_json`; its governed loop pins the CLI at
`=0.20.0`. Moving the library pin to `=0.23.0` keeps that call's signature and
contract: the registry-backed check in `docs/release-0.23.0.md` §5 builds
exactly that shape (`default-features = false`, `scaffold_init_json` only)
against the published crate and asserts the governance file set, purity and
the refusal of a camelCase key. The producer emits the same governance file set
Statecraft already implements against (spec 092); 0.21.0's `AGENTS.md` and
`.claude/` output is gone, which is the correction 0.22.0 was cut for. Moving
the CLI pin past 0.21.0 removes `spec-spine init` and makes a gate that cannot
read history exit 3 (`docs/adopter-migration.md` §9.2: use `fetch-depth: 0`).

**Interfaces and schema versions.**

| Capability | Facade (`spec-spine-core`) | CLI | Answer's axis |
|---|---|---|---|
| readiness with status (102) | `query_json` `op: "plan"` | `registry plan --json` | read `0.7.0` |
| obligation (106) | `query_json` `op: "obligation"` | `registry obligation <spec>#<id> --json` | read `0.7.0` |
| closure (107) | `closure_json` | `registry closure --request <file\|-> --json` | read `0.7.0` |
| impacts (109) | `query_json` `op: "impacts"` | `registry impacts --json` | read `0.7.0` |
| interface pins (110) | `interface_verify_json` | `interface verify --export <c>=<dir> --json` | read `0.7.0` |
| work scope (108) | `scope_json`, `scope_compare_json` | `scope evaluate`, `scope compare` (`--json`) | read `0.7.0` |
| waiver lifecycle (113) | `couple_json` with `prBody` or `waivers`, and `waiverInputs` | `couple --pr-body --waiver-as-of --waiver-uses --json` | verdict `0.6.0` |
| fixtures (103) | `verify_attestation_json` over `fixtures/verifier/` | `verify-attestation` | fixture set `0.1.0` |
| registry records | `compile_json`, `query_json` `op: "show"` | `registry show --json` | registry `1.6.0` |

**Examples and fixtures.** Paths inside the published `spec-spine-core`
0.23.0 crate are the same as in the repository at `d2bb4763`.

- `docs/examples/expansion-consumer/` (`run.sh`, `src/main.rs`): a disposable
  consumer of the packaged crates exercising every row above, including the
  composed flow of §9.
- `crates/spec-spine-core/fixtures/verifier/` (in the packaged crate at
  `fixtures/verifier/`): 11 cases with recorded outcomes, regenerated at
  0.23.0.
- The CLI-level cases Statecraft can copy as integration tests:
  `crates/spec-spine-cli/tests/scope.rs`, `crates/spec-spine-cli/tests/waiver.rs`,
  `crates/spec-spine-cli/tests/closure.rs`, `crates/spec-spine-cli/tests/interface.rs`.

**Positive and negative consumer checks** (each is asserted in the example or
the named tests):

| Check | Expect |
|---|---|
| plan entry for a `draft` spec | present in `ready`, `status: "draft"` (readiness is not approval) |
| obligation id without a spec | exit 3 |
| closure with a missing member | exit 1, every missing member named, no digest |
| closure over a stale ledger | exit 2 |
| scope with an undeclared crossing | exit 0, `S-002` naming both specs |
| scope with the crossing declared `shared` | exit 0, no finding |
| scope with an unknown `ownSpec` / a `..` path / a stale index | exit 1 / 3 / 2 |
| two scopes, one reading what the other changes | `changed-under-read` |
| waiver scoped to one of two refused paths | the other path still refuses; `waivers[0].clears` names one |
| waiver past its `-Until:` with an as-of supplied | clears nothing, check `failed` |
| waiver with `-Until:` and no as-of | check `not-evaluated`, clears on the rest |
| waiver at its `-Max-Uses:` per the supplied count | check `failed` |
| no waiver declared | report has no `waivers` member |
| `prBody` and `waiver` both given | exit 3 |
| a reader older than 0.23.0 on this repository | exit 3, "requires spec-spine >=0.23.0" |

**Compatibility and migration.**

- All schema moves are additive (MINOR); a loader pinned to a MAJOR keeps
  working. New members are omitted when empty wherever an old document would
  otherwise change: `waivers` and `unattachedWaiverLines` on `couple`.
- `couple`'s decision changes only for a pull request whose waiver declares a
  lifecycle line. A plain waiver clears everything, exactly as before, and is
  now reported as `scoped: false`.
- The facade signatures are unchanged; new functions and optional request
  members only (`couple_snapshots_waived`, `couple_with_prior_waived`,
  `parse_waivers`, `scope_json`, `scope_compare_json`).
- Adopting 0.23.0 in a Statecraft-managed repository: pin it, raise that
  repository's `[meta] required_version` to `>=0.23.0` once its corpus uses a
  0.23.0 member, and record the binary's identity in the job that gates.

**The boundary, restated.** A scope does not lock, reserve, exclude or permit.
A waiver's use limit is judged against the count Statecraft supplies. Nothing
here counts, consumes or records a use, and two concurrent runs given the same
count get the same answer. Scheduling, reservation, authorization, the use
store and fetching a cited corpus all stay with Statecraft.
