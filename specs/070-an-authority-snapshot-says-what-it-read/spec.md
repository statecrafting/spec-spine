---
id: "070-an-authority-snapshot-says-what-it-read"
title: "An authority snapshot says what it read"
status: approved
kind: "tooling"
created: "2026-09-11"
implementation: complete
owner: "The spec-spine Authors"
risk: high
depends_on:
  - "021-ledger-seal"
  - "022-index-sharding"
  - "028-registry-freshness-check"
  - "029-ownership-coverage"
  - "039-per-spec-attestation"
  - "050-claimed-but-unwitnessed"
  - "062-one-name-one-freshness-verb"
  - "066-an-attestation-covers-the-territory-it-claims"
  - "068-a-verifier-checks-the-bytes-it-was-given"
  - "069-the-committed-index-is-compared-not-trusted"
establishes:
  # 3.1: the payload DTO and its schema constant.
  - { kind: file, path: "crates/spec-spine-types/src/snapshot.rs" }
  # 3.1 to 3.4: the pure builder, the framed digest, and the recompute.
  - { kind: file, path: "crates/spec-spine-core/src/snapshot.rs" }
  # 3.6 and 3.7: determinism, separation and framing guards.
  - { kind: file, path: "crates/spec-spine-core/tests/snapshot.rs" }
extends:
  # 3.5: `attest --snapshot`, written and sealed like the other two scopes.
  - { spec: "021-ledger-seal", unit: "crates/spec-spine-cli/src/cmd_attest.rs", nature: additive }
  # 3.5: `verify-attestation --snapshot`, under spec 068's rules.
  - { spec: "021-ledger-seal", unit: "crates/spec-spine-cli/src/verify_attestation.rs", nature: additive }
  # 3.5: the flag declarations.
  - { spec: "001-compile-registry", unit: "crates/spec-spine-cli/src/main.rs", nature: additive }
  # 3.5: the module, its re-exports and the two facades.
  - { spec: "001-compile-registry", unit: "crates/spec-spine-core/src/lib.rs", nature: additive }
  # 3.1: the DTO re-export and `SNAPSHOT_SCHEMA_VERSION`.
  - { spec: "000-spec-spine-bootstrap", unit: "crates/spec-spine-types/src/lib.rs", nature: additive }
  - { spec: "000-spec-spine-bootstrap", unit: "crates/spec-spine-types/src/version.rs", nature: additive }
  # 3.6: end-to-end coverage of the flag.
  - { spec: "001-compile-registry", unit: "crates/spec-spine-cli/tests/cli.rs", nature: additive }
  # 3.2, D-9: the per-spec attestation is split into shared helpers so one
  # compile, index and lint serve every spec's join hash; its bytes do not move.
  - { spec: "039-per-spec-attestation", unit: "crates/spec-spine-core/src/attest.rs", nature: additive }
  # 3.3.1, D-9: the empty-directory companion to spec 066's walk, sharing its
  # pruning predicate, and the index comparison made crate-visible.
  - { spec: "004-codebase-index", unit: "crates/spec-spine-core/src/index.rs", nature: additive }
  # 3.6: the digest table names the new record beside the other axes.
  - { spec: "057-the-docs-name-what-adopters-derived", unit: "docs/schema-versioning.md", nature: additive }
references:
  - { unit: { kind: file, path: "docs/authority-evidence.md" }, role: context }
  - { unit: { kind: file, path: "docs/design/04-authority-evidence-extension.md" }, role: context }
summary: >
  A consumer that binds spec-spine's authority results to a revision needs five
  separate outputs today and still cannot answer four questions from them:
  whether the committed ledger matched the corpus (attestations recompute and
  never read the committed shards, and `check --json` reports booleans without
  the digests it compared), which configuration and governance files were read,
  what every spec's territory hashed to, and how much source no spec claims.
  This spec adds a third attestation scope, `attest --snapshot`, emitting an
  `AuthoritySnapshot`: the tool and schema versions, a digest of
  `spec-spine.toml`, the existing corpus hashes, the committed registry and
  index trees with a digest each and whether each matches the recompute, the
  governance inputs by path and digest, the gate verdicts, the ownership
  counts, the exclusions in force, and each spec's lifecycle with a framed
  digest of its resolved territory (plus, for joining records issued earlier,
  the hash of the `SpecAttestation` spec 039 would emit for it, or a stated
  reason where that construction has no answer). New members
  use a framed digest that binds normalized text rather than exact bytes, a
  contract 3.3 states explicitly and distinguishes from spec 068's, because
  the existing fold does not frame content and two trees can
  share one hash; existing members and payloads keep their constructions. The
  payload is pure, on demand, gitignored, sealable and verified under spec
  068's rules. It commits to no git revision: the consumer binds the tree.
---
# 087: An authority snapshot says what it read

## 1. Purpose

`docs/authority-evidence.md` §6 binds spec-spine's results to one commit by
hand: `check --json`, `attest --with-coupling`, `attest --spec`, `verify --plan`
and `couple`, five outputs a consumer must store and re-run separately. Even
together they leave four gaps, all measured at `75181a5`:

- **Freshness is not in any record.** `attest` compiles and indexes in memory
  and never reads `.derived/`, so a stale committed tree and a fresh one produce
  the same attestation. `check --json` reports `"fresh": true` and nothing it
  compared. A consumer can store that the ledger was fresh; it cannot store
  what "fresh" was a statement about.
- **The inputs are not named.** `inputsManifestHash` covers the `spec.md` files
  and nothing else. The configuration reaches a corpus attestation only through
  what it changes in the compile, and the governance files (`AGENTS.md`, the
  constitution, the workflows, this repository's thirty-odd
  `extra_hashed_inputs` patterns) reach it only inside the index scalar, under
  `--with-coupling`, and never by name.
- **Territory needs one call per spec.** A per-spec attestation is the only
  record that hashes claimed bytes. Eighty-five specs means eighty-five
  invocations and eighty-five hashes with nothing tying them together.
- **Unclaimed source has no record at all.** `index coverage` counts it; no
  attestation mentions it.

A fifth problem is in the hash construction the new record would otherwise
reuse. `hash::content_hash` writes each piece as path, NUL, content, with
nothing between one piece's content and the next piece's path. Measured with the
current binary: a claimed directory holding `a` = `x` and `b` = `y` produces
the same unit hash as one holding a single `a` whose content is `x`, `d/b`,
NUL, `y`. And since spec 066 encodes a non-UTF-8 file as the text
`sha256:<hex>`, a binary file and a text file holding that string hash alike.
Neither happens by accident in ordinary text. Both can be done on purpose, and
a record whose job is exact coverage should not inherit them.

## 2. Territory

- `crates/spec-spine-types/src/snapshot.rs`: `AuthoritySnapshot` and its
  members, all refusing unknown fields (spec 068 3.2), and
  `SNAPSHOT_SCHEMA_VERSION` re-exported through `version.rs`.
- `crates/spec-spine-core/src/snapshot.rs`: the builder, the `frame/1` digest,
  and `verify_snapshot_recompute`.
- `crates/spec-spine-core/tests/snapshot.rs`: the guards in 3.6 and 3.7.
- `cmd_attest.rs`, `verify_attestation.rs`, `main.rs`: the `--snapshot` flag on
  both verbs.
- Both `lib.rs` files: the module, the re-exports, and two facades.
- `crates/spec-spine-cli/tests/cli.rs`: end-to-end coverage.

No committed artifact changes and no existing payload changes.

## 3. Behavior

### 3.1 The payload

```json
{
  "schemaVersion": "0.1.0",
  "tool": { "name": "spec-spine", "version": "<x.y.z>" },
  "digest": "frame/1",
  "config": { "present": true, "hash": "<frame/1 over spec-spine.toml>" },
  "schemas": { "registry": "1.2.0", "index": "1.1.0", "corpusAttestation": "0.1.0", "specAttestation": "0.1.0" },
  "corpus": { "specs": 85, "inputsManifestHash": "<spec 021>", "registryHash": "<spec 021>" },
  "committed": {
    "registry": { "files": 85, "hash": "<frame/1>", "matchesRecompute": true },
    "index": { "files": 89, "hash": "<frame/1>", "matchesRecompute": true }
  },
  "governanceInputs": { "paths": ["AGENTS.md", "..."], "hash": "<frame/1>" },
  "verdicts": {
    "compile": { "ok": true, "errors": 0, "warnings": 0 },
    "lint": { "ok": true, "findingsHash": "<spec 021>" },
    "resolution": { "ok": true, "blocking": 0, "unresolved": 0 },
    "ownership": { "sourceFiles": 80, "claimed": 80, "floorOnly": 0, "unclaimed": 0 },
    "unwitnessed": { "total": 72, "allowed": 72 }
  },
  "specs": [
    { "id": "<id>", "status": "approved", "implementation": "complete",
      "territoryDigest": "<frame/1 over the spec's resolved owning units>",
      "specAttestationHash": "<hex, historical evidence only>" },
    { "id": "<id>", "status": "approved", "implementation": "complete",
      "territoryDigest": "<frame/1>",
      "specAttestationUnavailable": "non-utf8-direct-claim" }
  ],
  "exclusions": {
    "resolverExclusions": ["target", "node_modules", ".derived", "dist", "build", ".next"],
    "stateDir": null,
    "unwitnessedAllowed": ["crates/**/*.rs"],
    "bypassPrefixes": [".github/", "docs/", "..."]
  }
}
```

The payload is canonical JSON (sorted keys, two-space indent, LF, one trailing
newline). `attestationHash` is SHA-256 of those bytes and is emitted beside the
payload, never inside it, exactly as for specs 021 and 042.

### 3.2 What each member covers

- **`config`**: `present` is false and `hash` is absent when there is no
  `spec-spine.toml` (the defaults applied). Otherwise `hash` is `frame/1` over
  that one file, under 3.3's piece rule.
- **`schemas`**: the build's constants, so a consumer knows which record lines
  the rest of the payload was computed under.
- **`corpus`**: the spec count and spec 021's two hashes, under 023's
  construction, so a snapshot joins an existing corpus attestation by value.
- **`committed.registry`, `committed.index`**: `files` counts the committed
  shard files the freshness reads compare (the registry's `by-spec/`, the
  index's `by-spec/` and `by-package/`, and the slices sidecar when it exists).
  `hash` is `frame/1` over those files under 3.3's piece rule, keyed by
  repo-relative path. `matchesRecompute` is true exactly when every committed shard file is
  byte-identical to the shard the recompute emits and the two sets match: the
  comparison `compile --check` makes for the registry (spec 028 3.1) and the
  one spec 069 makes `index check` perform for the index. It carries no
  `--fail-on-*` refusal; those inputs are counts under `verdicts`. When a tree is absent, `files` is 0,
  `matchesRecompute` is false, and `hash` is absent rather than the digest of
  nothing (spec 066 3.3's reasoning).
- **`governanceInputs`**: `paths` lists, sorted, every file that feeds the
  index's global-inputs scalar (`spec-spine.toml` and every
  `[index] extra_hashed_inputs` match outside `layout.state_dir`); `hash` is
  `frame/1` over those files, which is normalized text for a UTF-8 file and
  exact bytes for any other (3.3). Workflows enter as their own content, not as
  spec 060's governance projection: see 5, D-4.
- **`verdicts`**: `compile` and `lint` mean what they mean in spec 021 (lint's
  floor is error or warning). `resolution.ok` is false when any owning unit is
  unresolved or any blocking resolver diagnostic exists, and the two counts say
  which. `ownership` is the `index coverage` report's four counts.
  `unwitnessed` is spec 050's pair.
- **`specs`**: one entry per spec in registry order. `implementation` is
  omitted only when the frontmatter omits it, as in spec 039 3.1. Each entry
  carries two hashes, and they answer different questions:

  - `territoryDigest` is **the content binding**: `frame/1` over the locations
    the spec's owning units resolve to, each a piece under 3.3's rule, sorted
    by repo-relative path, under the piece rule §3.3.1 states in full. A unit
    that resolves to nothing contributes no piece, and `verdicts.resolution` is
    where a consumer sees that it did not resolve. This is the member that says
    what the spec's territory came to, and it is framed, so the two collisions
    in §1 cannot occur in it. It is **absent** for a spec whose owning units all
    resolve to nothing (§3.3.1).
  - `specAttestationHash` is **historical evidence only, and explicitly not a
    content binding**: the `attestationHash` that `attest --spec <id>` would
    emit for the same tree and tool version, byte for byte. It exists so a
    consumer holding a snapshot and a per-spec attestation issued before this
    spec shipped can check the one against the other by equality. It is
    computed under spec 039's unframed construction, which is exactly why it
    cannot carry the binding: 1 measures two ways to collide it. A consumer
    binding territory MUST read `territoryDigest`; a consumer joining an older
    record MAY read `specAttestationHash`. It is **omitted, with a stated
    reason, for the one tree shape where spec 039's construction has no
    answer**; §3.2.1 states which shape and what stands in its place.

- **`exclusions`**: what was deliberately not read or not held to account: the
  resolver exclusions, the state root, the unwitnessed allowances, and the
  effective bypass prefixes (the floor plus the configured ones).

#### 3.2.1 When the historical join hash has no answer

There is one known shape where `attest --spec <id>` cannot produce a record: a
spec that claims a **file** unit directly whose content is not valid UTF-8.
Spec 066 reads a directly claimed file as text, so the verb exits 3 (reproduced
2026-09-15 at `0.19.0`). That behaviour MUST NOT change here. §3.7 holds, and a
new scope of `attest` is not a licence to loosen the per-spec verb underneath
it.

The snapshot MUST NOT inherit the failure either. For such a spec the entry:

- **keeps** its `territoryDigest`, which `frame/1` computes without difficulty,
  because a non-UTF-8 file is a `b` piece (§3.3.1). The content binding is
  unaffected by the join hash having no answer;
- **omits** `specAttestationHash`;
- carries `specAttestationUnavailable`, a short machine-readable reason whose
  only value in this spec is `non-utf8-direct-claim`.

A member absent with a stated reason is evidence a consumer can act on; a member
silently absent is a guess. And a snapshot that aborted here would let one
unhashable file cost every other spec in the corpus its record, which is the
opposite of what the payload is for.

The exemption is exactly this shape and MUST NOT widen. An input that cannot be
**read** at all (a permission error, a file that vanished mid-run, any other I/O
failure) remains an error that fails the payload, as everywhere else in this
tool: the reason member reports a construction that has no answer over content
that was read, never a read that did not happen. The Verification block tests
this through `attest --snapshot` on a tree carrying such a claim, not through
the framing helper alone, because what is being asserted is that the verb
completes while the per-spec verb still refuses.

### 3.3 The framed digest

Every member this spec introduces with a `hash` uses `frame/1`:

```
SHA-256( "spec-spine/frame/1" 0x00
         for each piece, sorted by path in byte order:
           kind: one byte, 't' for text, 'b' for bytes, 'l' for a symlink's
                 target text, 'd' for an empty directory
           u64 big-endian length of path, path
           u64 big-endian length of content, content )
```

A file that is valid UTF-8 is a `t` piece, with the standing normalization (BOM
stripped, CRLF and CR to LF) applied before its length is taken. Any other file
is a `b` piece holding its exact bytes. A symlink is an `l` piece holding its
target text as stored, never the content it points at. An empty directory
(§3.3.1) is a `d` piece whose content is always empty. Paths are repo-relative
POSIX.

The `d` kind is not decoration. An empty claimed directory and an empty claimed
**file** at the same path are different facts about a tree, and with only three
kinds both would frame as the same piece: same path, same empty content, and
the only thing left to tell them apart would be a kind byte neither has. A
digest whose job is exact coverage must not answer the same for a directory
somebody emptied and a file somebody truncated. The Verification block pins the
pair, and `tests/snapshot.rs` carries the same case. The construction is injective over piece sets, so the two collisions in
§1 cannot occur in a `frame/1` digest, **given** a piece set with no repeated
path: §3.3.1 is what guarantees that, and without it the framing alone does not
determine a digest.

**This is a binding over normalized text, not over exact bytes, and the two are
different contracts.** Spec 068 answers "are these the bytes I was given": it
verifies the attestation file as stored, and a single byte anywhere in it is a
mismatch. This spec answers "which inputs were read, and what did their content
come to": for a UTF-8 file that content is the normalized text, so a governance
file rewritten from LF to CRLF, or given a BOM, produces the **same**
`frame/1` digest. A consumer that needs exact-byte identity of a working tree
does not get it from a snapshot and must hash the tree itself; what a snapshot
gives is a platform-independent identity that a Windows checkout and a Linux
checkout of one revision agree on, which is the property every other hash in
this tool already has (spec 003's normalization, `.gitattributes`). Stating it
plainly is the point: a record whose job is evidence must not be read as
promising more than it holds. See 5, D-4.

Members that exist in other payloads (`inputsManifestHash`, `registryHash`,
`findingsHash`, `specAttestationHash`) keep their constructions. Changing them
would change every historical digest a consumer holds.

### 3.3.1 Which pieces a digest is taken over

`frame/1` says how a set of pieces becomes a digest. It does not say what the
set is, and two implementations agreeing on the framing can still disagree on
the digest. The piece rule is therefore stated here, once, for every member
this spec introduces with a `hash`.

**A piece is a path.** The unit of a piece is a file, a symlink or an empty
directory, addressed by its repo-relative POSIX path. Pieces MUST be
deduplicated by path, and a path MUST appear at most once in a digest. Nothing
below can therefore produce a duplicate, and `frame/1`'s injectivity over piece
sets is a property of what is actually fed to it.

**The walk is spec 066's walk.** The locations a unit resolves to are expanded
with `index::walk_territory`, the same function `attest --spec` uses, under the
same `resolver_exclusions` and the same `layout` pruning. A snapshot and a
per-spec attestation that disagreed about which files a subtree contains would
make `specAttestationHash` uncomparable with `territoryDigest` inside one
payload.

| Case | Piece rule |
|---|---|
| A `file` unit, or a location resolving to a file | one `t` or `b` piece: the **whole file**, never a slice of it |
| A `section`, `symbol` or `module` unit | the whole file the span lies in, as above. The span is **not** part of the digest; see the paragraph below |
| Two or more units of one spec resolving into the same file | **one** piece. The second and later occurrences are dropped by the dedup rule, not hashed again |
| A `directory`, `crate` or trailing-slash `file` unit | one piece per entry the walk yields, each keyed by its own path |
| A file unit whose path also lies under a directory unit of the same spec | **one** piece, by the dedup rule. An overlapping claim is not a doubled claim |
| A symlink met by the walk | one `l` piece holding the link's target text as stored. The link is never followed, so a cycle inside a claimed subtree terminates and content outside the repository never enters the payload (spec 066 §3.2) |
| An empty or wholly pruned directory | one **`d`** piece at the directory's own path with **empty content**, so an empty claim is distinguishable from no claim, two empty claims in different places are distinguishable from each other (spec 066 §3.3), and an empty directory is distinguishable from an empty file at the same path (§3.3) |
| A unit that resolves to nothing | **no** piece. `verdicts.resolution` is where a consumer sees that it did not resolve |
| A spec whose owning units all resolve to nothing, or which owns none | `territoryDigest` is **absent**, not the digest of an empty piece set. The digest of nothing is one constant shared by every such spec in every repository, which reads as evidence and distinguishes nothing (the reasoning spec 066 §3.3 applies to an empty directory, applied one level up) |

**`territoryDigest` binds files, not spans.** A spec owning one section of a
file has that whole file in its digest, so an edit elsewhere in the file moves
the spec's `territoryDigest` even though the owned section is untouched. This is
deliberate and it is the conservative direction: the alternative binds a byte
range whose boundaries are themselves recomputed by the resolver, so a span that
shifted would produce an unchanged digest over different content. A consumer
that needs span-level attribution reads the index shard, which records the
resolved spans; a snapshot answers "what did this spec's territory come to",
and the answer is the files.

**Non-UTF-8 content is a `b` piece, everywhere.** Spec 066's per-unit hash
substitutes the string `sha256:<hex>` for a file inside a claimed subtree that
is not valid UTF-8, and reads a directly claimed `file` unit as text, which
fails outright if that file is binary. `frame/1` needs neither workaround: a
`b` piece carries exact bytes and the kind byte keeps it distinct from a `t`
piece that happened to hold the same bytes. This is why the snapshot's own
members use `frame/1` and `specAttestationHash` keeps spec 039's construction
unchanged (§3.3, last paragraph).

### 3.4 Purity

The payload MUST be a pure function of `(config, file contents)`, the
committed shard files included: no clock, no environment, no git, no key.
Re-running on an unchanged tree at the same tool version MUST yield
byte-identical output. It is a snapshot of the tree it read: an untracked file
inside a claimed directory changes that spec's `specAttestationHash` (spec 066
3.4), so a consumer binding a snapshot to a revision computes it on a clean
export of that revision.

### 3.5 Surface

- `spec-spine attest --snapshot [--sign --key <path>] [--json]` writes
  `<derived_dir>/attestation/snapshot.json`, and its seal beside it as
  `snapshot.sig`, under the existing `.gitignore` entry. It exits 0 whenever a
  payload was written, whatever the verdicts say (spec 039 3.1: a record, not a
  gate). `--snapshot` combined with `--spec` or `--with-coupling` is refused at
  exit 3 with a message saying the two flags cannot combine, because each names
  a different scope.
- `spec-spine verify-attestation --snapshot --recompute | --signature
  --public-key <path>` verifies it under every rule of spec 068. The recompute
  names the member that moved, as the other two scopes do.
- `attest_snapshot_json(config_json, repo_root)` returns
  `{ "attestation", "attestationHash" }`. `verify_snapshot_attestation_json`
  takes the request shape spec 068 3.4 defines.
- `SNAPSHOT_SCHEMA_VERSION = "0.1.0"`, independent of every other axis, pinned
  by a test like the others.

### 3.6 What it proves, and what it does not

For one tree and one tool version it establishes exactly which inputs were
read and what they hashed to, whether the committed ledger equals the
recompute, the verdicts the gate would reach, and every spec's territory digest
(with its historical attestation hash wherever spec 039's construction has an
answer, §3.2.1). It does not establish anything about a revision (the consumer binds
`{repo, commit, tree}` and recomputes), anything about unclaimed files beyond
their count, whether any specification is correct, or that anyone approved the
state. `docs/authority-evidence.md` MUST say so in its digest table when this
ships.

### 3.7 What must keep working

`CorpusAttestation` and `SpecAttestation` are byte-identical to their pre-087
output. No committed shard moves. No gate verb changes its answer. `attest
--spec` keeps refusing a directly claimed non-UTF-8 file at exit 3: §3.2.1
routes around that refusal inside the snapshot and does not repair it.

## 4. Out of scope

**A revision or repository identity in the payload.** See 5, D-1.

**Membership proofs.** The `specs` list allows one-by-one checking by equality.
A tree with proofs for partial disclosure is a later record with a named
consumer.

**Imported snapshots from other repositories.** Pinned cross-corpus imports are
a P2 proposal in design note 04.

**Obligations, context closures, work scopes.** Proposed in design note 04; each
references this payload's hash, which is why this comes first.

**Committing snapshots.** Spec 039 3.3's reasons apply with more force to a
record that changes on every edit to any claimed file.

## 5. Resolved decisions

**Status note (2026-09-14): this spec was held at draft on 2026-09-12 and the
held text is now corrected.** The maintainer refused approval because 3.3 and
D-4 could not both hold: D-4 called the governance-input hash "the bytes as
written" while 3.3 normalized every UTF-8 piece before framing it, so approving
the spec would have made a self-contradiction governing. The ruling was to name
two contracts rather than pick a winner, since 085 and 087 answer different
questions and only one of them is a byte identity. What changed, and nothing
else did: 3.3 now states the normalized-text contract and contrasts it with
085's; D-4 withdraws the "bytes as written" claim and names what the trade
costs; D-3 adds the framed `territoryDigest` and demotes `specAttestationHash`
to historical evidence explicitly not a content binding; and the framing
acceptance line, which moved several things at once, is now two lines that move
one thing each. No behaviour this spec requires of a build was widened. It
remains `status: draft`: approval is a human flip, and this revision is a
request for one, not a substitute.

**Revised again 2026-09-15, and authorized for build.** The review that
authorized it made the authorization conditional on one gap: §3.2 and §3.3
defined the framing without defining the piece set, so two implementations
could conform and still disagree. §3.3.1 is new and states the piece rule in
full; §3.3 now names the `l` piece its own formula always had and says plainly
that injectivity holds only over a set with no repeated path; §3.2's
`territoryDigest` bullet points at the rule and names the absent case; D-6
records the four choices that were open. No member changed, no construction
changed, and no behaviour this spec requires of a build was widened. Per
`AGENTS.md` "Working the backlog", it stays `draft` through the build PR and is
ratified in a separate PR after merge.


**D-1 (2026-09-11): the payload is git-free.** The core has no git (a
workspace invariant), and a tree is what the payload reads. The mapping from a
revision to a tree is git's, recorded by the consumer next to the snapshot hash
and checkable by anyone who exports that revision and recomputes.

**D-2 (2026-09-11): a third scope of `attest`, not a new version of the
corpus attestation.** Adding members to `CorpusAttestation` would change the
`attestationHash` consumers already pin (statecraft-cli's adoption holdback
records it), and would turn a stable record into a moving one. A new record on
its own schema axis leaves every existing hash meaning what it meant.

**D-3 (2026-09-11, revised 2026-09-14): each spec carries a framed territory
digest, and its attestation hash is demoted to historical evidence.** The
first version of this entry chose the existing per-spec `attestationHash` as
the sole representation, to avoid hashing the same files a second way. That
made the one member a consumer would use to bind a spec's territory the one
member computed under the construction 1 measures as collidable, in a payload
whose stated job is exact coverage. The cost it was avoiding is a second pass
over bytes already read; the cost it accepted is a binding that does not bind.

So `territoryDigest` is added under `frame/1` (3.2), and `specAttestationHash`
stays with its meaning narrowed in the text to joining records issued before
this spec shipped. Nothing about `SpecAttestation` changes: its unit bytes keep
the unframed construction, every historical digest keeps its value, and 3.7
still holds. Rejected: removing `specAttestationHash`, which would break the
join it exists for; and reframing `SpecAttestation` itself, which would move
every digest a consumer has pinned. Design note 04 D3 records the wider
trade.

**D-4 (2026-09-11, revised 2026-09-14): governance inputs enter as their own
content, under 3.3's piece rule, not as spec 060's projection.** Spec 060 folds
a workflow as its governance projection so that an action-version bump does not
restale every committed shard. A snapshot is on demand and never committed, so
that churn does not arise: a consumer asking which governance files were read
wants the file, not a projection of it that deliberately ignores part of it.

The first version of this entry justified that by saying "the true answer to
'which bytes were read' is the bytes". That claim is withdrawn, because 3.3
does not hash bytes: a UTF-8 governance file enters as **normalized text** (BOM
stripped, CRLF and CR to LF), and only a non-UTF-8 file enters as its exact
bytes. What this member records is therefore the content of each governance
file up to line-ending and BOM spelling, and the payload cannot distinguish two
trees that differ only there. That is the deliberate trade 3.3 names: it is
what makes the digest identical across a Windows and a Linux checkout of one
revision, and it is why this spec binds normalized text while spec 068 binds
exact bytes. Rejected: hashing governance inputs as raw bytes, which would make
a snapshot platform-dependent and disagree with every other hash in the tool;
and using spec 060's projection, which answers a different question than "what
did I read".

**D-5 (2026-09-11): `matchesRecompute` is the plain freshness answer.** The
`--fail-on-unresolved` and `--fail-on-warn` refusals are policy on top of
freshness; the snapshot records their inputs as counts and leaves the policy to
the reader.

**D-6 (2026-09-15): the piece set is specified, not left to the framing.** A
review pass held that §3.2 and §3.3 defined how pieces become a digest without
defining which pieces there are, so two conforming implementations could agree
on `frame/1` and still emit different `territoryDigest` values. §3.3.1 is the
answer, and four of its rules were genuinely open rather than obvious:

- **Dedup is by path, and overlapping claims collapse.** A spec owning both a
  directory and a file inside it, or two `section` units in one file, has that
  file once. The alternative, one piece per unit, puts a repeated path in the
  set and forfeits the injectivity §3.3 claims, which is the property the whole
  framing exists for.
- **The digest binds whole files, never spans.** A span-level digest would hash
  boundaries the resolver itself recomputes, so a span that shifted over
  changed content could hash the same. Whole files are conservative in the safe
  direction: the digest can move when the owned region did not, and it cannot
  fail to move when the owned region did.
- **An all-unresolved spec has no `territoryDigest`**, rather than the digest of
  an empty piece set, which is one constant shared by every such spec in every
  repository. Spec 066 §3.3 already reasoned this way one level down, for an
  empty directory.
- **The walk is `index::walk_territory`**, the one `attest --spec` uses, so
  `territoryDigest` and `specAttestationHash` in the same payload cannot
  disagree about what a subtree contains. Symlinks are `l` pieces carrying the
  link text, unfollowed, which is spec 066 §3.2's rule and keeps a cycle
  terminating and out-of-repository content out of the payload.

**D-7 (2026-09-15): an empty directory is its own piece kind.** §3.3. The held
text gave an empty directory a piece with empty content and named only three
kinds, so it would have framed identically to an empty file at the same path.
Adding `d` costs one byte in the formula and keeps two different facts about a
tree apart. Rejected: encoding the distinction in the path (a trailing slash),
which makes the path of a piece depend on what the piece is and would let a
future caller construct either spelling for one filesystem object.

**D-8 (2026-09-15): a join hash with no answer is omitted with a reason, and
only for that one shape.** §3.2.1. A spec directly claiming a non-UTF-8 file
makes `attest --spec` exit 3, and the first version of this spec required a
`specAttestationHash` for every spec, so one such claim anywhere in a corpus
would have taken the whole snapshot down. Three alternatives were rejected:
changing spec 039 so the per-spec verb succeeds, which moves historical digests
and is the thing §3.7 exists to prevent; emitting some placeholder hash, which
puts a value in a member whose meaning is "the record you would get", where no
such record exists; and omitting the member silently, which leaves a consumer
unable to tell an unsupported shape from an implementation that forgot. The
reason is a closed vocabulary of one so that a second reason is a spec change
rather than a build's judgement call, and an unreadable input stays an error.


**D-9 (2026-09-15): seven choices the text left open, settled at build.**

- **Every empty directory the walk reaches is a `d` piece, not only a claimed
  one.** §3.3.1's row says "an empty or wholly pruned directory", and the
  Verification block's empty-piece line claims `g/` while `g/e` is empty: under
  spec 066's walk, which yields files and symlinks only, a nested empty
  directory contributed nothing, so that line passed without ever producing a
  `d` piece. `index::empty_territory_dirs` yields the **leaf** directories the
  walk reaches that hold nothing it does not prune, sharing the walk's pruning
  predicate, and each is a `d` piece. A directory whose only children are empty
  directories is not itself empty, so the pieces sit at the directories that
  are. The file set is still exactly spec 066's, so `specAttestationHash` and
  `territoryDigest` still agree about what a subtree contains.
- **A directly claimed symlink is an `l` piece.** §3.3.1 states the rule for a
  symlink "met by the walk"; a unit resolving to a symlink directly is framed
  the same way, unfollowed, so one filesystem object has one piece kind however
  it was claimed. `specAttestationHash` keeps spec 039's behavior.
- **`governanceInputs.hash` is absent when there are no paths**, by the same
  reasoning §3.2 applies to an absent committed tree.
- **`committed.index.files` counts the slices sidecar; `matchesRecompute`
  compares the shard directories.** §3.2 lists the sidecar among the files and
  defines the flag over shard files, and those are the two sets spec 069's
  comparison already reads.
- **Verdicts come from the recompute.** `ownership` is `coverage_with` over the
  in-memory index and `unwitnessed` is spec 050's pair over the same index,
  rather than the freshness-guarded verbs, so a stale committed tree still
  yields a complete record with `matchesRecompute: false` beside it.
- **`attest_spec` is split, not duplicated.** Its territory hashing and payload
  assembly became crate-visible helpers the snapshot calls with one compile,
  index and lint; `attest_spec` calls the same helpers in the same order, and
  its error text is unchanged. The existing per-spec suites pin the bytes, and a
  CLI test asserts the snapshot's join hash equals `attest --spec`'s.
- **`verify-attestation --snapshot --spec` is refused at exit 3**, the mirror of
  §3.5's refusal on `attest`.

## Verification

Each line runs in its own `sh -c` from the repository root (spec 043 3.5). The
scratch corpus lives at a fixed path, and each line that mutates it undoes the
mutation before its final assertion.

Against pre-087 code every line that calls `attest --snapshot` or
`verify-attestation --snapshot` fails, since neither flag exists. That includes
both scope-refusal lines, one per flag 3.5 names: an unknown flag also exits 3,
so each line asserts the refusal's own wording, which a usage error does not
contain. The setup lines
and the closing `check` pass before and after.

`frame/1` makes two independent claims, and the first version of this block
tested them in one line that changed several things at once, so a pass could
not say which claim held. They are now one line each, and each moves exactly
one thing:

- **The length-framing line** takes a tree holding `g/a` = `x` and `g/b` = `y`
  to one holding a single `g/a` = `xg/b\0y`. Under the unframed fold both
  produce the same scalar, because nothing separates one piece's content from
  the next piece's path. Under `frame/1` the `u64` lengths differ, so the two
  `governanceInputs.hash` values must differ. It changes the piece set and
  nothing else: both trees are UTF-8, so both are `t` pieces throughout, and
  the piece-type rule is not exercised.
- **The piece-type line** reads one file twice: first holding the four bytes
  `FF FE 00 01`, which are not valid UTF-8, then holding the literal text
  `sha256:<hex>` where the hex is the SHA-256 of exactly those four bytes.
  Spec 066 encodes a non-UTF-8 file as exactly that text, so before `frame/1`
  the two hash alike; under `frame/1` one is a `b` piece and the other a `t`
  piece, so the kind byte differs and the digests must differ. It changes one
  file's content and no path, so the length framing is not what is under test.

  The byte file is **written first and hashed from disk**, never carried in a
  shell variable: a command substitution drops NUL bytes, so `B1=$(printf
  '\377\376\000\001')` would silently hold three bytes and hash three, and
  the line would still pass while testing something other than what this
  paragraph says. The digest tool is selected as spec 068's block selects it,
  because `sha256sum` and `shasum` are not both present everywhere.

Neither line can run before `frame/1` exists, which is the point: both fail
against pre-087 code with the flag absent, and both would still fail against an
implementation that shipped the flag over the unframed fold.

Two lines test §3.3.1 directly, and neither can pass by accident:

- **The dedup line** gives one spec the directory `g/` and the file `g/a`,
  which lies inside it, and asserts `territoryDigest` is **unchanged** from the
  same spec claiming `g/` alone. An implementation that emitted one piece per
  unit would hash `g/a` twice and the digests would differ.
- **The unresolved line** gives that spec a single unit resolving to nothing and
  asserts the payload carries **no** `territoryDigest` at all, rather than the
  digest of an empty piece set. It counts occurrences rather than comparing a
  value, because the value it is asserting the absence of is precisely the one
  constant a wrong implementation would emit here for every such spec.

Both restore the spec file from a backup before asserting, so a later line in
this block reads the corpus it expects.

Two further lines test the two rules this revision added, and both go through
`attest --snapshot` rather than through the digest helper, because each is a
claim about what the verb emits:

- **The empty-piece line** claims `g/` while `g/e` is an empty **directory**,
  then again while `g/e` is an empty **file**, and asserts the two
  `territoryDigest` values differ. With only three piece kinds they would be the
  same digest: same path, empty content, nothing left to separate them. It is
  the only line that exercises the `d` kind, and it changes nothing else.
- **The unavailable-join line** gives the spec a direct `file` claim on
  non-UTF-8 content and asserts three things at once, deliberately: `attest
  --spec` still exits 3 (§3.7 is not loosened), `attest --snapshot` still exits
  0, and the entry carries `territoryDigest` and
  `specAttestationUnavailable: "non-utf8-direct-claim"` with no
  `specAttestationHash`. Asserting the refusal and the snapshot in one line is
  what makes it evidence: a build that "fixed" the refusal would satisfy the
  snapshot half and fail the line.

The per-spec count line asserts that every spec carries **either** a
`specAttestationHash` **or** a stated reason, which is the invariant §3.2.1
leaves in place of "every spec carries a hash".

The separation line proves `matchesRecompute` reads the committed tree: it
changes a committed shard and nothing the recompute reads, then asserts the
recomputed `registryHash` did not move while `matchesRecompute` did. The
territory line asserts `territoryDigest` is present for every spec and that it
moves when a claimed file's content moves, which `specAttestationHash` alone
could not demonstrate is framed. It compiles and indexes the scratch corpus
first, because `registry list` reads the **committed** registry and the scratch
corpus has none until something writes one: without that, `registry list` exits
3, `T` is empty, and `test "$A" -eq "$T"` errors on a non-integer rather than
asserting anything. The two framing lines above need no such call, because
`attest --snapshot` compiles in memory. A line that depends on committed state
builds it itself rather than relying on a neighbour that happens to run first.

```verify:cli
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
D="${TMPDIR:-/tmp}/ss087"; S=target/release/spec-spine; M="$D/specs/001-a/spec.md"; cp "$M" "$D/spec.bak"; printf '\377\376\000\001' > "$D/bin.dat"; printf -- '---\nid: "001-a"\ntitle: "t"\nstatus: draft\ncreated: "2026-09-11"\nsummary: "s"\nestablishes:\n  - "bin.dat"\n---\n\n# t\n' > "$M"; $S --repo "$D" attest --spec 001-a >/dev/null 2>&1; E=$?; J=$($S --repo "$D" attest --snapshot --json); R=$?; cp "$D/spec.bak" "$M"; rm -f "$D/bin.dat"; test $E -eq 3 && test $R -eq 0 && printf '%s' "$J" | grep -q '"specAttestationUnavailable": "non-utf8-direct-claim"' && printf '%s' "$J" | grep -q '"territoryDigest"' && ! printf '%s' "$J" | grep -q '"specAttestationHash"'
rm -rf "${TMPDIR:-/tmp}/ss087" "${TMPDIR:-/tmp}/ss087-self.json"
target/release/spec-spine check
```
