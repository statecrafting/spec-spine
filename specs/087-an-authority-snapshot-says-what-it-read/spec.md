---
id: "087-an-authority-snapshot-says-what-it-read"
title: "An authority snapshot says what it read"
status: draft
kind: "tooling"
created: "2026-09-11"
implementation: pending
owner: "The spec-spine Authors"
risk: high
depends_on:
  - "023-ledger-seal"
  - "024-index-sharding"
  - "031-registry-freshness-check"
  - "032-ownership-coverage"
  - "042-per-spec-attestation"
  - "057-claimed-but-unwitnessed"
  - "075-one-name-one-freshness-verb"
  - "083-an-attestation-covers-the-territory-it-claims"
  - "085-a-verifier-checks-the-bytes-it-was-given"
  - "086-the-committed-index-is-compared-not-trusted"
establishes:
  # Planned (spec 076) until the build writes them; the build drops the flag.
  # 3.1: the payload DTO and its schema constant.
  - { kind: file, path: "crates/spec-spine-types/src/snapshot.rs", planned: true }
  # 3.1 to 3.4: the pure builder, the framed digest, and the recompute.
  - { kind: file, path: "crates/spec-spine-core/src/snapshot.rs", planned: true }
  # 3.6 and 3.7: determinism, separation and framing guards.
  - { kind: file, path: "crates/spec-spine-core/tests/snapshot.rs", planned: true }
extends:
  # 3.5: `attest --snapshot`, written and sealed like the other two scopes.
  - { spec: "023-ledger-seal", unit: "crates/spec-spine-cli/src/cmd_attest.rs", nature: additive }
  # 3.5: `verify-attestation --snapshot`, under spec 085's rules.
  - { spec: "023-ledger-seal", unit: "crates/spec-spine-cli/src/verify_attestation.rs", nature: additive }
  # 3.5: the flag declarations.
  - { spec: "001-compile-registry", unit: "crates/spec-spine-cli/src/main.rs", nature: additive }
  # 3.5: the module, its re-exports and the two facades.
  - { spec: "001-compile-registry", unit: "crates/spec-spine-core/src/lib.rs", nature: additive }
  # 3.1: the DTO re-export and `SNAPSHOT_SCHEMA_VERSION`.
  - { spec: "000-spec-spine-bootstrap", unit: "crates/spec-spine-types/src/lib.rs", nature: additive }
  - { spec: "000-spec-spine-bootstrap", unit: "crates/spec-spine-types/src/version.rs", nature: additive }
  # 3.6: end-to-end coverage of the flag.
  - { spec: "001-compile-registry", unit: "crates/spec-spine-cli/tests/cli.rs", nature: additive }
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
  counts, the exclusions in force, and each spec's lifecycle with the hash of
  the `SpecAttestation` spec 042 would emit for it. New members use a framed
  digest, because the existing fold does not frame content and two trees can
  share one hash; existing members and payloads keep their constructions. The
  payload is pure, on demand, gitignored, sealable and verified under spec
  085's rules. It commits to no git revision: the consumer binds the tree.
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
NUL, `y`. And since spec 083 encodes a non-UTF-8 file as the text
`sha256:<hex>`, a binary file and a text file holding that string hash alike.
Neither happens by accident in ordinary text. Both can be done on purpose, and
a record whose job is exact coverage should not inherit them.

## 2. Territory

- `crates/spec-spine-types/src/snapshot.rs`: `AuthoritySnapshot` and its
  members, all refusing unknown fields (spec 085 3.2), and
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
  "corpus": { "specs": 85, "inputsManifestHash": "<spec 023>", "registryHash": "<spec 023>" },
  "committed": {
    "registry": { "files": 85, "hash": "<frame/1>", "matchesRecompute": true },
    "index": { "files": 89, "hash": "<frame/1>", "matchesRecompute": true }
  },
  "governanceInputs": { "paths": ["AGENTS.md", "..."], "hash": "<frame/1>" },
  "verdicts": {
    "compile": { "ok": true, "errors": 0, "warnings": 0 },
    "lint": { "ok": true, "findingsHash": "<spec 023>" },
    "resolution": { "ok": true, "blocking": 0, "unresolved": 0 },
    "ownership": { "sourceFiles": 80, "claimed": 80, "floorOnly": 0, "unclaimed": 0 },
    "unwitnessed": { "total": 72, "allowed": 72 }
  },
  "specs": [
    { "id": "<id>", "status": "approved", "implementation": "complete", "specAttestationHash": "<hex>" }
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
payload, never inside it, exactly as for specs 023 and 042.

### 3.2 What each member covers

- **`config`**: `present` is false and `hash` is absent when there is no
  `spec-spine.toml` (the defaults applied). Otherwise `hash` is `frame/1` over
  that one file as text.
- **`schemas`**: the build's constants, so a consumer knows which record lines
  the rest of the payload was computed under.
- **`corpus`**: the spec count and spec 023's two hashes, under 023's
  construction, so a snapshot joins an existing corpus attestation by value.
- **`committed.registry`, `committed.index`**: `files` counts the committed
  shard files the freshness reads compare (the registry's `by-spec/`, the
  index's `by-spec/` and `by-package/`, and the slices sidecar when it exists).
  `hash` is `frame/1` over those files, read as text, keyed by repo-relative
  path. `matchesRecompute` is true exactly when every committed shard file is
  byte-identical to the shard the recompute emits and the two sets match: the
  comparison `compile --check` makes for the registry (spec 031 3.1) and the
  one spec 086 makes `index check` perform for the index. It carries no
  `--fail-on-*` refusal; those inputs are counts under `verdicts`. When a tree is absent, `files` is 0,
  `matchesRecompute` is false, and `hash` is absent rather than the digest of
  nothing (spec 083 3.3's reasoning).
- **`governanceInputs`**: `paths` lists, sorted, every file that feeds the
  index's global-inputs scalar (`spec-spine.toml` and every
  `[index] extra_hashed_inputs` match outside `layout.state_dir`); `hash` is
  `frame/1` over those files as raw text. Workflows are hashed as written, not
  as spec 073's governance projection: see 5, D-4.
- **`verdicts`**: `compile` and `lint` mean what they mean in spec 023 (lint's
  floor is error or warning). `resolution.ok` is false when any owning unit is
  unresolved or any blocking resolver diagnostic exists, and the two counts say
  which. `ownership` is the `index coverage` report's four counts.
  `unwitnessed` is spec 057's pair.
- **`specs`**: one entry per spec in registry order. `implementation` is
  omitted only when the frontmatter omits it, as in spec 042 3.1.
  `specAttestationHash` is the `attestationHash` that `attest --spec <id>`
  would emit for the same tree and tool version, byte for byte. A consumer
  holding a snapshot and one spec's attestation checks the one against the
  other by equality, without recomputing the rest.
- **`exclusions`**: what was deliberately not read or not held to account: the
  resolver exclusions, the state root, the unwitnessed allowances, and the
  effective bypass prefixes (the floor plus the configured ones).

### 3.3 The framed digest

Every member this spec introduces with a `hash` uses `frame/1`:

```
SHA-256( "spec-spine/frame/1" 0x00
         for each piece, sorted by path in byte order:
           kind: one byte, 't' for text, 'b' for bytes, 'l' for a symlink's target text
           u64 big-endian length of path, path
           u64 big-endian length of content, content )
```

A file that is valid UTF-8 is a `t` piece, with the standing normalization (BOM
stripped, CRLF and CR to LF) applied before its length is taken. Any other file
is a `b` piece holding its exact bytes. Paths are repo-relative POSIX. The
construction is injective over piece sets, so the two collisions in 1 cannot
occur in a `frame/1` digest.

Members that exist in other payloads (`inputsManifestHash`, `registryHash`,
`findingsHash`, `specAttestationHash`) keep their constructions. Changing them
would change every historical digest a consumer holds.

### 3.4 Purity

The payload MUST be a pure function of `(config, file contents)`, the
committed shard files included: no clock, no environment, no git, no key.
Re-running on an unchanged tree at the same tool version MUST yield
byte-identical output. It is a snapshot of the tree it read: an untracked file
inside a claimed directory changes that spec's `specAttestationHash` (spec 083
3.4), so a consumer binding a snapshot to a revision computes it on a clean
export of that revision.

### 3.5 Surface

- `spec-spine attest --snapshot [--sign --key <path>] [--json]` writes
  `<derived_dir>/attestation/snapshot.json`, and its seal beside it as
  `snapshot.sig`, under the existing `.gitignore` entry. It exits 0 whenever a
  payload was written, whatever the verdicts say (spec 042 3.1: a record, not a
  gate). `--snapshot` combined with `--spec` or `--with-coupling` is refused at
  exit 3 with a message saying the two flags cannot combine, because each names
  a different scope.
- `spec-spine verify-attestation --snapshot --recompute | --signature
  --public-key <path>` verifies it under every rule of spec 085. The recompute
  names the member that moved, as the other two scopes do.
- `attest_snapshot_json(config_json, repo_root)` returns
  `{ "attestation", "attestationHash" }`. `verify_snapshot_attestation_json`
  takes the request shape spec 085 3.4 defines.
- `SNAPSHOT_SCHEMA_VERSION = "0.1.0"`, independent of every other axis, pinned
  by a test like the others.

### 3.6 What it proves, and what it does not

For one tree and one tool version it establishes exactly which inputs were
read and what they hashed to, whether the committed ledger equals the
recompute, the verdicts the gate would reach, and every spec's attestation
hash. It does not establish anything about a revision (the consumer binds
`{repo, commit, tree}` and recomputes), anything about unclaimed files beyond
their count, whether any specification is correct, or that anyone approved the
state. `docs/authority-evidence.md` MUST say so in its digest table when this
ships.

### 3.7 What must keep working

`CorpusAttestation` and `SpecAttestation` are byte-identical to their pre-087
output. No committed shard moves. No gate verb changes its answer.

## 4. Out of scope

**A revision or repository identity in the payload.** See 5, D-1.

**Membership proofs.** The `specs` list allows one-by-one checking by equality.
A tree with proofs for partial disclosure is a later record with a named
consumer.

**Imported snapshots from other repositories.** Pinned cross-corpus imports are
a P2 proposal in design note 04.

**Obligations, context closures, work scopes.** Proposed in design note 04; each
references this payload's hash, which is why this comes first.

**Committing snapshots.** Spec 042 3.3's reasons apply with more force to a
record that changes on every edit to any claimed file.

## 5. Resolved decisions

**D-1 (2026-09-11): the payload is git-free.** The core has no git (a
workspace invariant), and a tree is what the payload reads. The mapping from a
revision to a tree is git's, recorded by the consumer next to the snapshot hash
and checkable by anyone who exports that revision and recomputes.

**D-2 (2026-09-11): a third scope of `attest`, not a new version of the
corpus attestation.** Adding members to `CorpusAttestation` would change the
`attestationHash` consumers already pin (statecraft-cli's adoption holdback
records it), and would turn a stable record into a moving one. A new record on
its own schema axis leaves every existing hash meaning what it meant.

**D-3 (2026-09-11): each spec is represented by its existing attestation
hash.** The alternative, a framed territory digest per spec, would hash the
same files a second way. The cost is that unit bytes inside a
`SpecAttestation` keep the unframed construction; design note 04 D3 records
the trade for review.

**D-4 (2026-09-11): governance inputs are hashed as written.** Spec 073 folds a
workflow as its governance projection so that an action-version bump does not
restale every committed shard. A snapshot is on demand and never committed, so
that churn does not arise, and the true answer to "which bytes were read" is the
bytes.

**D-5 (2026-09-11): `matchesRecompute` is the plain freshness answer.** The
`--fail-on-unresolved` and `--fail-on-warn` refusals are policy on top of
freshness; the snapshot records their inputs as counts and leaves the policy to
the reader.

## Verification

Each line runs in its own `sh -c` from the repository root (spec 049 3.5). The
scratch corpus lives at a fixed path, and each line that mutates it undoes the
mutation before its final assertion.

Against pre-087 code every line that calls `attest --snapshot` or
`verify-attestation --snapshot` fails, since neither flag exists. That includes
the scope-refusal line: an unknown flag also exits 3, so the line asserts the
refusal's own wording, which a usage error does not contain. The setup lines
and the closing `check` pass before and after.

The framing line is the only one that can tell `frame/1` from the unframed
fold: the two scratch trees fold to one index scalar today, so their
`governanceInputs.hash` values differ only if the framing does its job. The
separation line proves `matchesRecompute` reads the committed tree: it changes
a committed shard and nothing the recompute reads, then asserts the recomputed
`registryHash` did not move while `matchesRecompute` did.

```verify:cli
cargo build --release --locked
target/release/spec-spine attest --snapshot --json > "${TMPDIR:-/tmp}/ss087-self.json" && grep -q '"matchesRecompute": true' "${TMPDIR:-/tmp}/ss087-self.json"
A=$(target/release/spec-spine attest --snapshot --json); B=$(target/release/spec-spine attest --snapshot --json); test -n "$A" && test "$A" = "$B"
H=$(target/release/spec-spine attest --spec 083-an-attestation-covers-the-territory-it-claims --json | sed -n 's/.*"attestationHash": "\([0-9a-f]*\)".*/\1/p'); test -n "$H" && grep -q "\"specAttestationHash\": \"$H\"" .derived/attestation/snapshot.json
test "$(grep -c '"specAttestationHash"' .derived/attestation/snapshot.json)" -eq "$(target/release/spec-spine registry list --ids-only | wc -l)"
target/release/spec-spine verify-attestation --snapshot --recompute
rm -rf "${TMPDIR:-/tmp}/ss087" && mkdir -p "${TMPDIR:-/tmp}/ss087/specs/001-a" "${TMPDIR:-/tmp}/ss087/g"
printf -- '[index]\nextra_hashed_inputs = ["g/*"]\n' > "${TMPDIR:-/tmp}/ss087/spec-spine.toml"
printf -- '---\nid: "001-a"\ntitle: "t"\nstatus: draft\ncreated: "2026-09-11"\nsummary: "s"\nestablishes:\n  - "g/"\n---\n\n# t\n' > "${TMPDIR:-/tmp}/ss087/specs/001-a/spec.md"
D="${TMPDIR:-/tmp}/ss087"; S=target/release/spec-spine; printf 'x' > "$D/g/a"; printf 'y' > "$D/g/b"; A=$($S --repo "$D" attest --snapshot --json | awk '/"governanceInputs": \{/{getline; print; exit}'); rm "$D/g/b"; printf 'xg/b\000y' > "$D/g/a"; B=$($S --repo "$D" attest --snapshot --json | awk '/"governanceInputs": \{/{getline; print; exit}'); rm -f "$D/g/a"; printf 'x' > "$D/g/a"; test -n "$A" && test -n "$B" && test "$A" != "$B"
D="${TMPDIR:-/tmp}/ss087"; S=target/release/spec-spine; $S --repo "$D" compile >/dev/null && $S --repo "$D" index >/dev/null && $S --repo "$D" attest --snapshot --json > "$D/before.json" && F="$D/.derived/spec-registry/by-spec/001-a.json" && cp "$F" "$D/shard.bak" && printf ' ' >> "$F" && $S --repo "$D" attest --snapshot --json > "$D/after.json"; R=$?; cp "$D/shard.bak" "$F"; test $R -eq 0 && grep -q '"matchesRecompute": false' "$D/after.json" && test "$(grep '"registryHash"' "$D/before.json")" = "$(grep '"registryHash"' "$D/after.json")"
D="${TMPDIR:-/tmp}/ss087"; S=target/release/spec-spine; A=$($S --repo "$D" attest --snapshot --json); printf '# a comment\n' >> "$D/spec-spine.toml"; B=$($S --repo "$D" attest --snapshot --json); printf -- '[index]\nextra_hashed_inputs = ["g/*"]\n' > "$D/spec-spine.toml"; test "$A" != "$B" && test "$(echo "$A" | grep '"inputsManifestHash"')" = "$(echo "$B" | grep '"inputsManifestHash"')"
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss087" attest --snapshot --spec 001-a >/dev/null 2> "${TMPDIR:-/tmp}/ss087/scope.err"; test $? -eq 3 && grep -q "cannot combine" "${TMPDIR:-/tmp}/ss087/scope.err"
rm -rf "${TMPDIR:-/tmp}/ss087" "${TMPDIR:-/tmp}/ss087-self.json"
target/release/spec-spine check
```
