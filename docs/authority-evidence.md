# Consuming spec-spine authority evidence

> For a consumer that stores, forwards or verifies what spec-spine says about a
> repository: a local orchestrator, a hosted control plane, or a portable
> review record. It maps each question to the verb and library function that
> already answers it, states what every digest covers, and says what each
> result proves and what it does not. Nothing here is a new API. Proposed
> additions are in [design/04](design/04-authority-evidence-extension.md).
> Specs 068 and 086 shipped in `v0.19.0`. Spec 071 (`delta`) is built on `main`
> and not yet in a release; draft 087 is still a proposal, and is marked as one
> wherever it appears.

**Measured state.** Every output below was produced on 2026-09-11 at commit
`75181a5` (main) by a binary built from that commit. The binary reports
`spec-spine 0.18.0`, and so does the released `v0.18.0`, which is eight commits
older and does not contain spec 066 (see §8). The corpus held 85 specs: 84
`approved` with `implementation: complete`, and 084 `draft` with
`implementation: pending`. Both committed trees were fresh. The hashes below
reproduce only on a clean export of `75181a5` (§6); any other tree, including a
working tree with later specs in it, gives different values, which is the
point of them.

**Since then.** Spec 068 (a verifier checks the bytes it was given) and spec
069 (the committed index is compared, not trusted) shipped in `v0.19.0`. The
measurements are kept as taken, and §4, §5 and §8 say beside them what
`v0.19.0` does. A consumer running a binary older than `0.19.0` still gets the
behavior measured here.

## 1. The questions, and what already answers them

| Question | CLI (read verbs never write) | Library / JSON facade | Output and version marker | Exit codes |
|---|---|---|---|---|
| Are the committed trees what the corpus compiles to? | `check [--fail-on-unresolved] [--fail-on-warn] --json` | `check_freshness_json`, `check_registry_freshness_json` | verdict envelope `0.3.0`, verb `check` | 0 fresh, 2 stale, 1 invalid or refused, 3 cannot read |
| Registry half, index half separately | `compile --check --json`, `index check --json` | same | envelope `0.3.0`, verbs `compile.check`, `index.check` | same |
| Does one spec validate? | `compile --spec <id> --json` | `compile_json` | envelope `0.3.0`, verb `compile.spec` | 0, 1, 3 |
| Is the corpus conformant? | `lint --fail-on-warn --json` | `lint_json` | envelope `0.3.0`, report is the finding list | 0, 1, 3 |
| Who owns a path, and how? | `index owner <path> --json` | `owners_for_path`, `classify`, `authorities` | raw JSON, **no envelope, no version field** | 0 (also when nothing owns it), 3 |
| Which source files does no spec claim? | `index coverage [--fail-on-untraced] --json` | `coverage_json` | raw JSON, **no envelope, no version field** | 0, 1, 2, 3 |
| Does a change drift from its owning spec? | `couple --base B --head H [--pr-body F] --json` | `couple_json` (caller supplies the parsed diff) | envelope `0.3.0`, verb `couple` | 0, 1 drift, 2 stale index, 3 |
| What kind of change is it, under the base's rules? (spec 071, unreleased) | `delta --base B --head H --json` | `delta_json` (caller supplies both exported trees, the changed paths and the commit ids) | `DeltaReport` `schemaVersion 0.1.0` inside envelope `0.4.0`, verb `delta` | 0 whenever a report was produced, 2 stale index at the merge base, 3 |
| What does a spec declare as acceptance? | `verify --plan --json <id>` | `verify_plan_json` | envelope `0.3.0`, verb `verify` | 0, 1 unknown id, 3 |
| Run that acceptance (**executes**) | `verify <id> --json` | none: the library never runs a command | envelope `0.3.0`, `VerifyReport` | 0, 1, 3 |
| What is workable now? | `registry plan --json` | `query_json` op `plan` | raw JSON, **no envelope, no version field** | 0, 3 |
| Freeze the corpus verdict | `attest [--with-coupling] [--sign --key K] --json` | `attest_json` | `CorpusAttestation` `schemaVersion 0.1.0` inside envelope `0.3.0` | 0 whenever a payload was written, 3 |
| Freeze one spec's territory | `attest --spec <full-id> [--sign --key K] --json` | `attest_spec_json` | `SpecAttestation` `schemaVersion 0.1.0` | 0 whenever a payload was written, 1 unknown id (the short id is refused until 084), 3 |
| Check a frozen record | `verify-attestation [--spec <full-id>] --recompute \| --signature --public-key P --json` | `verify_attestation_json`, `verify_spec_attestation_json` (recompute only) | envelope `0.3.0` | 0 match, 1 mismatch or invalid, 3 |

The envelope versions above are `v0.19.0`'s. Spec 071 adds the `delta` verb
token, an additive change that moves every verb's envelope to `0.4.0` from the
first release carrying it; nothing else in an envelope changes.

Three rules from the existing specs govern how these may be read:

- **An `attest` exit code is not a verdict** (spec 039 3.1, amending 023). Exit
  0 means a payload was written. The verdicts are inside it, and a failing one
  is still exit 0. A caller that wants a refusal runs `check`, `lint` or
  `couple`.
- **`--json` changes what is written, never what is decided** (spec 034). Parse
  the envelope; do not match prose. The prose is a rendering and has changed.
- **`verify` is the one verb that executes** (spec 043 3.6). It runs commands
  written in a markdown file, with the caller's environment, and is not a
  sandbox. `verify --plan` reads the same commands without running them.

## 2. Version axes

Each is a compile-time constant in `crates/spec-spine-types/src/version.rs`
or `attest.rs`, independent of the others and of the package version.

| Artifact | Field | Current |
|---|---|---|
| registry shards | `specVersion` | `1.2.0` |
| index shards | `schemaVersion` | `1.1.0` |
| `CorpusAttestation` | `schemaVersion` | `0.1.0` |
| `SpecAttestation` | `schemaVersion` | `0.1.0` |
| verdict envelope | `schemaVersion` | `0.3.0` in `v0.19.0`; `0.4.0` since spec 071 |
| `DeltaReport` (spec 071) | `schemaVersion` | `0.1.0`, unreleased |
| `build-meta.json` | `schemaVersion` | `0.1.0` (non-deterministic, gitignored) |
| `spec-spine.toml` | `config_version` | `0.1.0` (optional key) |
| the tool | `spec-spine --version`, `tool.version` in an attestation | `0.18.0` when measured; every release moves it |

`registry plan --json`, `index owner --json` and `index coverage --json` carry
no version marker and do not emit sorted keys, although
[api.md](api.md) §7 says every emitted JSON document does. A consumer that
stores or digests their bytes has nothing to pin; use the enveloped verbs, or
the facade, for anything retained.

When measured, `docs/schema-versioning.md` still listed the registry and index
at `1.0.0` and omitted the two attestation axes and the envelope. Spec 068
corrected it; the constants above remain the authority.

**The tool version is not a build identity, and spec-spine does not make it
one.** A version string identifies a release only for a binary built from that
release's tag, or installed from crates.io, npm, PyPI or the release archives.
A binary built from `main` prints the version of the last release it was bumped
to, so between releases two binaries with different behavior print the same
string (§8 shows one such pair). `tool.version` is what
`verify-attestation --recompute` compares, so the same limit applies to it: a
record made by a development build and checked by the release that prints the
same version cannot report the named `versionMismatch` outcome, because the
stamps are equal. A consumer that accepts evidence from builds other than a
release must record the source commit next to `tool.version` itself; nothing
spec-spine emits carries it. This is the limit design/04 D1 chose to document
for `v0.19.0`.

## 3. Real outputs

`check --fail-on-unresolved --fail-on-warn --json` at `75181a5` (running it
twice gives byte-identical output):

```json
{
  "exitCode": 0,
  "ok": true,
  "report": {
    "index": {
      "diagnostics": { "byCode": {}, "errors": 0, "warnings": 0 },
      "fresh": true,
      "unwitnessed": { "allowed": 72, "total": 72 }
    },
    "registry": { "fresh": true, "validationPassed": true, "warnings": 0 }
  },
  "schemaVersion": "0.3.0",
  "verb": "check"
}
```

`couple --base e4032ff --head 75181a5 --json`, in a clean checkout of
`75181a5`:

```json
{
  "exitCode": 0,
  "ok": true,
  "report": { "checkedPaths": 1, "violations": [] },
  "schemaVersion": "0.3.0",
  "verb": "couple"
}
```

`attest --with-coupling --json`:

```json
{
  "exitCode": 0,
  "ok": true,
  "report": {
    "attestation": {
      "inputsManifestHash": "65b0edf5fd8d55610acecac28b84c30fb9774acb4a61e6b280572b27ec0ffee4",
      "registryHash": "b6eabbbb0c8d9cb48361083241da8bb5e86e0eec759063329352282d3119d9f2",
      "schemaVersion": "0.1.0",
      "tool": { "name": "spec-spine", "version": "0.18.0" },
      "verdicts": {
        "compile": { "ok": true },
        "couple": {
          "indexHash": "40146f4027e559bf1fa3eea0e7f5d3937968d7b23586d7815a81b806e62508d3",
          "joinHash": "88e45bbaf3f2b5ca105460c2193ee41c7e2c0a47a824564db784e988e0e7dce4",
          "ok": true
        },
        "lint": {
          "findingsHash": "37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570",
          "ok": true
        }
      }
    },
    "attestationHash": "44046b3957ede75148d10db258cca3e7c544fb90a7b13f124996c750ad6caf99"
  },
  "schemaVersion": "0.3.0",
  "verb": "attest"
}
```

`attest --spec 056-an-attestation-covers-the-territory-it-claims --json`
(report only):

```json
{
  "attestation": {
    "lifecycle": { "implementation": "complete", "status": "approved" },
    "schemaVersion": "0.1.0",
    "specId": "066-an-attestation-covers-the-territory-it-claims",
    "specSourceHash": "d99fcb2660da006550cc9af5141520b2609d90dd847985e0cc550768fbc4e789",
    "tool": { "name": "spec-spine", "version": "0.18.0" },
    "units": [
      { "contentHash": "dbdaf78f424e1cc9aac210628de619f674d7aa0585299a79c2290949f55aed3d",
        "unit": { "kind": "file", "path": "crates/spec-spine-cli/tests/cli.rs" } },
      { "contentHash": "0e187540170816c69fa393a3ce7431a43477b25fb86f15ac6998957649ca25c3",
        "unit": { "kind": "file", "path": "crates/spec-spine-core/src/attest.rs" } },
      { "contentHash": "0cadd7dadbac3838681748e20686f3ae2db1be9a06e0c0e37ecb2741696a70e2",
        "unit": { "kind": "file", "path": "crates/spec-spine-core/src/index.rs" } },
      { "contentHash": "f7baeb9ff2eab0a8d0e3c9b9269c8a32692341833d54125d40c7ed2fe80b2600",
        "unit": { "kind": "file", "path": "crates/spec-spine-core/tests/attest.rs" } }
    ],
    "verdicts": {
      "compile": { "ok": true },
      "lint": { "findingsHash": "37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570", "ok": true },
      "resolution": { "ok": true }
    }
  },
  "attestationHash": "ef0f30a757d9e5a5a64df62c2d3e010be7e989df5cafcbfa38097b5547a013f1"
}
```

The same verb for the draft 084 reports `resolution.ok: false`, because three
of its thirteen owning units are `planned` and not yet written; its lifecycle
reads `draft` / `pending`, which is what makes that `false` interpretable
(spec 039 3.1).

`verify --plan --json 067-a-short-id-names-the-same-spec-at-every-verb`
returns `{ "specId", "commands": [26 strings], "skipped": [] }` in the same
envelope. `verify-attestation --recompute --signature --public-key P --json`
on an untouched sealed pair:

```json
{
  "exitCode": 0,
  "ok": true,
  "report": {
    "outcome": "match",
    "signature": { "keyId": "ea4a6c63e29c520abef5507b132ec5f9954776aebebe7b92421eea691446d22c", "valid": true }
  },
  "schemaVersion": "0.3.0",
  "verb": "verify-attestation"
}
```

## 4. What each digest covers

`content_hash` below is `hash::content_hash`: SHA-256 over
`<repo-relative-POSIX-path> NUL <normalized text>` for each piece, pieces
sorted by path, **concatenated with no separator between pieces**.
Normalization strips a leading BOM and folds CRLF and CR to LF.

| Digest | Where | Covers | Construction |
|---|---|---|---|
| `shardHash` | each committed registry shard | one `spec.md` | `content_hash([(spec path, text)])` |
| `inputsManifestHash` | `CorpusAttestation`; the registry's `build.contentHash` | every `spec.md`, and nothing else | `content_hash` over `("spec:<id>", shardHash)` pairs |
| `registryHash` | `CorpusAttestation` | the compiled aggregate registry, **recomputed in memory** | SHA-256 of its canonical JSON |
| index shard hash | each committed index shard | the spec's `spec.md`, the files backing its `section` / `symbol` spans, and the global-inputs scalar | `content_hash` |
| global-inputs scalar | inside every index shard hash | `spec-spine.toml` plus every `[index] extra_hashed_inputs` match, minus `layout.state_dir`; workflows fold as their governance projection (spec 060) | `content_hash` |
| `indexHash` | `CorpusAttestation` under `--with-coupling` | the recomputed index's `contentHash` | fold of index shard hashes |
| `joinHash` | same | the pair | SHA-256 of `"<registryHash>:<indexHash>"` |
| `findingsHash` | both attestations | every lint finding, info tier included (corpus); the findings whose path is the spec's (per spec) | SHA-256 of canonical JSON of the list |
| `specSourceHash` | `SpecAttestation` | one `spec.md` | SHA-256 of its normalized text, **no path prefix** |
| unit `contentHash` | `SpecAttestation` | what an owning unit resolves to: files; directories walked under `resolver_exclusions` and `state_dir`; symlinks as their target text; non-UTF-8 files as the text `sha256:<hex of bytes>` | `content_hash` |
| `attestationHash` | beside a payload, never inside it | the payload | SHA-256 of its canonical JSON, which is exactly the bytes written to disk |
| seal `sig` | detached `.sig` | `attestationHash` | Ed25519 over its 32 raw bytes |
| `registry show` `contentHash` | query output | one `spec.md` | the `shardHash` construction; it does **not** equal `specSourceHash` for the same file. Until spec 077 the prose line glossed it as "sha256 of this spec.md" and approved spec 048 §3.4 said the same; 096 amends that sentence and the line now names the path framing, with a CLI test pinning both constructions (closed) |

### The authority snapshot (spec 070)

`attest --snapshot` writes `<derived>/attestation/snapshot.json`, an
`AuthoritySnapshot` on its own schema axis. Every digest it **introduces** is
`frame/1`: SHA-256 over `"spec-spine/frame/1" NUL`, then for each piece sorted
by path, one kind byte (`t` text, `b` bytes, `l` symlink target, `d` empty
directory), the path's length as a big-endian u64, the path, the content's
length as a big-endian u64, and the content. A UTF-8 file is a `t` piece with
the standing normalization applied; any other file is a `b` piece with its
exact bytes. Paths are deduplicated, so a path appears once. The length prefix
and the kind byte are what the unframed `content_hash` above lacks: under
`frame/1` a split piece set cannot collide with a joined one, and a binary file
cannot collide with the text `sha256:<hex>` spec 066 substitutes for it. It
binds **normalized text**, not exact bytes: a governance file rewritten from LF
to CRLF has the same `frame/1` digest, which is what makes a Windows and a
Linux checkout of one revision agree. Spec 068's verifier, by contrast, checks
exact bytes.

| Digest | Where | Covers | Construction |
|---|---|---|---|
| `config.hash` | `AuthoritySnapshot` | `spec-spine.toml`; absent when there is none | `frame/1` |
| `committed.registry.hash`, `committed.index.hash` | same | the committed shard files (and the index's slices sidecar), as stored; absent when the tree is | `frame/1` |
| `committed.*.matchesRecompute` | same | whether every committed shard is byte-identical to the recompute and the sets match | a flag, not a digest |
| `governanceInputs.hash` | same | `spec-spine.toml` and every `extra_hashed_inputs` match outside the state root, by path, as their own content (not spec 060's projection) | `frame/1` |
| `specs[].territoryDigest` | same | the files, symlinks and empty directories a spec's owning units resolve to, whole files never spans; absent when nothing resolves | `frame/1` |
| `specs[].specAttestationHash` | same | historical evidence only: the `attestationHash` `attest --spec` emits for that spec, or absent with `specAttestationUnavailable: "non-utf8-direct-claim"` where that verb cannot produce a record | spec 039's |
| `corpus.inputsManifestHash`, `corpus.registryHash`, `verdicts.lint.findingsHash` | same | as in `CorpusAttestation` | unchanged |

What a snapshot establishes, for one tree and one tool version: exactly which
inputs were read and what they hashed to, whether the committed ledger equals
the recompute, the verdicts the gate would reach, and every spec's territory
digest. What it does **not** establish: anything about a revision (the consumer
binds `{repo, commit, tree}` and recomputes from that tree), anything about
unclaimed files beyond their count, whether any specification is correct, or
that anyone approved the state.

Every value above is independently recomputable from a tree and the same tool
version, except the seal, which needs the public key. No key and no network is
needed for anything else.

### What the digests do not cover

- **No attestation commits to a git revision or a repository identity.** The
  payloads are content-addressed and git-agnostic by design (spec 021 3). The
  binding to a commit is the consumer's record, and it is checkable by
  recomputing from that commit's tree (§6).
- **The corpus attestation does not commit to source code bytes**, even with
  `--with-coupling`. A `file`, `directory` or `crate` unit carries no span, so
  it contributes to no index hash (spec 050); this repository allows all 72 of
  its such claims. `couple.ok` in a corpus attestation means "every claimed
  unit resolves with no blocking resolver diagnostic". Only `SpecAttestation`
  unit hashes cover claimed bytes, and only for the one spec's owning units.
- **Nothing covers an unclaimed file.** `index coverage` reports them; no
  attestation hashes them.
- **Committed-shard freshness is not in any attestation.** `attest` recomputes
  the registry and index from the tree. A committed shard tree that disagrees
  with the corpus produces the same attestation as a fresh one. Freshness is a
  separate answer, from `check`, and `check --json` reports booleans, not the
  digests it compared.
- **An index shard's hash does not cover its own body.** `shardHash` covers
  the shard's inputs (the `spec.md`, the span files, the global scalar). Up to
  `v0.18.0`, `index check` compared only that field; since `v0.19.0` it compares
  the whole shard's bytes with a fresh index, so the body is checked even
  though the hash still does not cover it (§5).
- **`spec-spine.toml` has no digest of its own** in a spec-corpus-only
  attestation. It reaches `registryHash` only through what it changes in the
  compile, and reaches `indexHash` through the global-inputs scalar.
- **The normalization is deliberate and lossy.** CRLF and LF variants, and a
  file with or without a BOM, hash identically.
- **The fold is not injective.** Measured with the current binary: a claimed
  directory holding `a` = `x` and `b` = `y` attests the same unit
  `contentHash` as one holding only `a` = `x`, NUL, `d/b`, NUL, `y`, because
  nothing frames one piece from the next. A binary file and a text file whose
  content is the string `sha256:<that file's digest>` also attest identically.
  Neither is reachable by accident in ordinary text; both are reachable on
  purpose. Existing digests keep this construction so that historical evidence
  stays interpretable; design/04 proposes a framed construction for new record
  types only.

## 5. Semantics that must not be collapsed

| These are different | Because |
|---|---|
| `attest --with-coupling` and `couple --base B --head H` | The first asks whether every claimed unit resolves in one tree. The second asks whether a diff between two revisions touched owned code without its owning spec. Neither implies the other. |
| a recomputed attestation and a fresh committed tree | `attest` never reads the committed shards. `check` does. |
| `check` fresh and "the committed index is what the corpus indexes to", up to `v0.18.0` | Up to `v0.18.0`, `index check` did not re-resolve. It read each committed shard's own mapping, hashed the span files that mapping named, and compared the result with `shardHash`. For a unit with no span (`file`, `directory`, `crate`) nothing constrained the body. Measured on a scratch repository: a shard rewritten so its spec owns nothing, `shardHash` untouched, read fresh, and `couple` then derived ownership from it. Since `v0.19.0` (spec 069) the two are the same claim: `index check` indexes in memory and compares shard bytes and the shard set, as the registry side has since spec 028 3.1, and `couple`, `index coverage` and `index owner` read the committed index only after that comparison passes. |
| `couple` exit 0 and "the owning spec approves this change" | C-001 clears when **any** owning spec's `spec.md` is in the diff. An edit that weakens that spec, including its `## Verification` block, clears it. The guard against that is a rule addressed to agents (`AGENTS.md` "Adversarial prompt refusal"), not a mechanism; design note 02 G7 records it as unsolved. Spec 071's `delta` reports such an edit as `verification`, with the base and head plan digests; it reports, and refuses nothing. |
| `couple` exit 0 and "the owners were the owners before this change" | Owners are read from the committed index **at the candidate**. A candidate that files a new spec with an `extends` edge on a unit becomes an owner of that unit in the same diff and clears C-001 with its own `spec.md`. That is the sanctioned route for legitimate work (spec 093), and it is also an authority transfer no one outside the diff approved. Spec 071's `delta` reports it as `authority` on the unit, with `baseOwners` from the merge base's index and `headOwners` from the head tree resolved under the merge base's configuration. |
| `delta`'s `priorPolicy.required: false` and "this change is safe" | `required: false` means only that no path carries `requirement`, `verification`, `authority`, `lifecycle`, `constitutional`, `policy` or `unknown`. It does not mean the change is safe, correct or approved. `delta` interprets no prose, sees a change to a file a verification command reads only as `implementation`, and decides nothing (spec 071 3.5, 3.7). A consumer judges each listed class under the base revision's policy and records who approved it. |
| `couple`'s subject and the candidate commit | `couple` diffs `merge-base(B, H)...H` but checks freshness and resolves units in the working tree. It speaks for `H` only when the working tree is a clean checkout of `H`. Its report echoes neither commit. |
| `verify` passing and the spec being satisfied | A declared command exited 0 on one machine at one time, under the candidate's own copy of the block. |
| a valid seal and a trustworthy signer | The seal proves possession of a key. Which keys to trust is the consumer's policy (spec 021 6). |

## 6. Binding authority evidence to a candidate revision, offline

No hosted account is involved. The consumer supplies the revision; spec-spine
supplies content-addressed results; a third party recomputes.

```sh
# The subject, from git (the consumer's job; spec-spine never runs git in its core).
git rev-parse 75181a5 '75181a5^{tree}'
#   75181a518e8bf69cc3a003b106cb483a9ed32454
#   fc93c505763e60f3a23f9425b6f8c724009be578

# A clean export of exactly that tree. An untracked or ignored file inside a
# claimed directory changes a unit hash (spec 066 3.4), so never attest a
# working tree you did not just check out.
W=$(mktemp -d) && git archive 75181a5 | tar -x -C "$W"

spec-spine --repo "$W" check --fail-on-unresolved --fail-on-warn --json > check.json
spec-spine --repo "$W" attest --with-coupling --json > corpus.json
spec-spine --repo "$W" attest --spec 057-a-short-id-names-the-same-spec-at-every-verb --json > spec.json
spec-spine --repo "$W" verify --plan --json 067-a-short-id-names-the-same-spec-at-every-verb > plan.json

# The diff verdict needs git: run it in a clean checkout whose HEAD is the candidate.
spec-spine couple --base e4032ff --head 75181a5 --json > couple.json
```

A record the consumer keeps (this shape is a **proposal**, not a spec-spine
output; the values are the real ones from that run):

```json
{
  "subject": {
    "repo": "git@github.com:statecrafting/spec-spine.git",
    "commit": "75181a518e8bf69cc3a003b106cb483a9ed32454",
    "tree": "fc93c505763e60f3a23f9425b6f8c724009be578"
  },
  "base": { "commit": "e4032ff2795bd132527726f38b4d29283b6d1413", "mergeBase": "e4032ff2795bd132527726f38b4d29283b6d1413" },
  "tool": { "name": "spec-spine", "version": "0.18.0" },
  "reads": [
    { "verb": "check", "envelopeSchema": "0.3.0", "exitCode": 0,
      "sha256": "aa1fc653cc1ce679f7ffb270e120421b14ba758ddb067336a814d7bec02a0d4d" },
    { "verb": "couple", "envelopeSchema": "0.3.0", "exitCode": 0,
      "sha256": "0f971f3223b514f29d8a8aa25e26696f941f3709088f743c4005b0ca1940db0b" }
  ],
  "attestations": [
    { "type": "CorpusAttestation", "schemaVersion": "0.1.0", "scope": "specs+code",
      "attestationHash": "44046b3957ede75148d10db258cca3e7c544fb90a7b13f124996c750ad6caf99" },
    { "type": "SpecAttestation", "schemaVersion": "0.1.0",
      "specId": "067-a-short-id-names-the-same-spec-at-every-verb",
      "specSourceHash": "de71e87232a69abd2c1e45ba2415f0f8db68c72fdd6c727df7568e21357d8692",
      "attestationHash": "ab0e784071cd3be65060f2ead325e0e2d87d3ff8f803f8fbdcea249b8ca05644" }
  ],
  "plan": { "specId": "067-a-short-id-names-the-same-spec-at-every-verb", "commands": 26,
            "sha256": "0172fbfd35a0cd50be10d10c71802ac65d1d591c10677c11c8a5a390b7a1a3cc" }
}
```

A verifier with the same tool version fetches the commit, checks that its tree
id matches, exports it, reruns the same verbs and compares bytes, and runs
`verify-attestation --recompute` against each stored payload. Every `sha256`
above is over the exact envelope bytes, which are deterministic for a given
tree and binary (measured for `check`). A `versionMismatch` is an **unknown**,
not a failure of content: rerun under the recorded version.

What that record then establishes, and nothing more: at that tree, under that
tool version, the committed ledger matched the corpus; the corpus validated and
linted clean; every claimed unit resolved; the diff from the merge base did not
touch owned code without an owning `spec.md` in the same diff; spec 067's own
text and owning units hashed to the listed values; and its declared acceptance
was those 26 commands. It does not establish that the commands were run, that
they pass, that the specification is correct, or that anyone with authority
approved the change.

## 7. Verification planning for content you do not trust

1. **Read the plan, never the block.** `verify --plan --json <id>` is a pure
   read and runs nothing. Parse it; do not re-implement the `## Verification`
   grammar (spec 043 3.2). Blocks the tool does not run are listed under
   `skipped` rather than dropped.
2. **Read it from the trusted base, not from the candidate.** A candidate that
   edits its own `## Verification` block and its code in one diff passes
   `couple` (§5). Take the plan from the base revision's copy of the spec, or
   treat any difference between the base and candidate plans as a change that
   needs approval under the base's policy. Record `specSourceHash` for the copy
   the plan came from. `delta --base B --head H --json` (spec 071) makes this
   difference mechanical: a spec whose plan differs carries `verification`,
   with `basePlanHash`, `headPlanHash` and how many commands are only on each
   side.
3. **Execute only in a worker that was explicitly authorized to run it.** The
   commands are shell (`sh -c`), inherit the caller's environment (spec 043
   3.5), and are a stranger's in the general case. Never run them in a process
   holding control-plane credentials. Isolation, credentials and time limits
   are the consumer's policy; spec-spine does not sandbox and does not claim to.
4. **Bind outputs to the evaluated revision.** Run in a clean checkout of the
   candidate commit; record the commit, the tree, the plan digest and the
   base-side `specSourceHash` with each command's exit code. `verify --json`
   reports the failing command's own exit code inside the report, separately
   from the process exit status.

## 8. Cross-version and tamper behavior, measured

Released `v0.18.0` was built from its tag and compared with a binary built
from `75181a5`. Both print `spec-spine 0.18.0`.

| Case | Result | Reading |
|---|---|---|
| `v0.18.0` `attest --spec` over all 85 specs | 71 succeed, 14 exit 3 ("Is a directory") | spec 066's fix is not released |
| `75181a5` over the same | 85 succeed | |
| per-spec attestation of 048 made at `75181a5`, verified by `v0.18.0` | exit 3, I/O error | the version stamps are equal, so the named `versionMismatch` outcome cannot fire |
| corpus attestation made by either, verified by the other | `match` | historical corpus evidence is unaffected |
| per-spec attestation of file-only 083, either direction | `match` | historical file-unit evidence is unaffected |

Against a sealed attestation, using a scratch key:

| Tamper | `--recompute` at `75181a5` | `--signature` at `75181a5` | Verdict at `75181a5` | Since `v0.19.0` (spec 068) |
|---|---|---|---|---|
| none | match | valid | exit 0 | exit 0 |
| add an unknown member, top level or nested (`"prCouple": {"ok": true}`) | match | **valid** | **exit 0** | exit 3, parse error naming the member |
| add an unknown member to a per-spec attestation | match | **valid** | **exit 0** | exit 3, parse error naming the member |
| corpus `schemaVersion` set to `9.0.0` | **match** | invalid | exit 0 with recompute alone | exit 3, `attestation schema MAJOR 9 is unsupported` |
| per-spec `schemaVersion` set to `9.0.0` | content mismatch, "tool.name or schemaVersion" | not run | exit 1 | exit 3, the same MAJOR refusal |
| reformat without changing values (compact, reordered keys) | match | **valid** | exit 0 | exit 1 in either mode: `--recompute` reports `contentMismatch`, `bytes are not the canonical serialization`; `--signature` reports the seal invalid, because it is checked over the stored bytes |
| `tool.version` changed | `versionMismatch` | | exit 1 | exit 1, `versionMismatch` |
| a verdict flipped | content mismatch naming the field | invalid | exit 1 | exit 1, `contentMismatch` |
| duplicate key | refused, parse error | | exit 3 | exit 3 |

The four bold rows were contract gaps, not intended behavior. Spec 021 AC-4
says a single tampered payload byte fails the signature, and the attestation
module's own documentation says loaders reject an unknown MAJOR. The verifier
deserialized without refusing unknown members and re-canonicalized before
checking the seal, so the bytes it verified were not the bytes a consumer
reads. Spec 068 closes all four in `v0.19.0`: the verifier reads the file once
and both modes decide on those bytes. The last column is spec 068's own
`## Verification` block, which passed in full (25 commands) on a `0.19.0`
build; the reformat row was measured with a compact copy.

Two things follow for a consumer. **Store the bytes `attest` wrote**: a
re-serialized copy no longer verifies. And **with a verifier older than
`0.19.0`, hash the stored file bytes yourself and compare against
`attestationHash`**, and refuse members you do not know, rather than rely on
`verify-attestation` to do either.

## 9. The current consumer, mapped

statecraft-cli at `6f49d9a` mints `acceptance.receipt` records
(`members/src/orchestrator/receipt.ts`, `RECEIPT_SCHEMA_VERSION = 1`) and runs
the gate floor `check --fail-on-warn`, `lint --fail-on-warn`,
`couple --base <sha> --head HEAD` (`gate-contract.ts` `gateFloor`), without
`--json`.

| Receipt field | spec-spine counterpart | Binding today | Gap |
|---|---|---|---|
| `specId` | the registry id; `SpecAttestation.specId` | a string | no digest of the spec's text at base or candidate |
| `repo.baseSha` | `couple --base` | argv only | `couple` resolves a merge base and echoes neither commit |
| `repo.candidateSha` | the tree every read verb judged | none from spec-spine | holds only if the verbs ran in a clean checkout of it |
| `suite.commands`, `suite.digest` | the verbs' argv | SHA-256 of canonical argv arrays | covers command text only: no outputs, no tool build, no config |
| `policy.digest` | none | CLI gate contract plus profile | excludes `spec-spine.toml`, the constitution and the specs themselves |
| `verifier.specSpine` | `spec-spine --version` | the raw string | not a build identity (§2) |
| `results[].exitCode` | envelope `exitCode` | integers | the reports are discarded; exit codes 1, 2 and 3 are not distinguished in failure evidence |
| `sensitivePaths` | overlaps the global-inputs set and the constitution | prefix list | a partial, CLI-local policy-delta detector |
| none | `attestationHash` (corpus and per spec), `specSourceHash`, plan digest, envelope digests | none | the receipt carries no spec-spine result at all |

Separately, the CLI's export and adoption paths run `attest --with-coupling` and
`attest` and match `attestationHash:` in prose with a regular expression
(`export.ts`, `adopt/holdback.ts`), the pattern spec 034's migration note
names as the one that broke; `stages/verify.ts` re-implements the
`## Verification` grammar rather than reading `verify --plan --json`. Its
`spec-spine.toml` pins `required_version = "0.18.0"` (a caret range) and CI
installs from `main`'s `install.sh` without a version, so it runs `v0.18.0`,
which predates spec 066; it does not call `attest --spec` today. The requests
this implies are in design/04 §7.
