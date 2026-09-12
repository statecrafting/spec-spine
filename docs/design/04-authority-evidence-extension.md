# 04: Authority evidence for consumers outside the repository

A design note, not a spec. It records what spec-spine should grow so that a
local orchestrator, a hosted control plane and a portable review record can
consume its authority results bound to exact revisions, and, with equal weight,
what it must refuse to grow. Four increments are filed alongside as drafts
(specs 085 to 088). Everything else here is a proposal for review; no
record shape in this note is emitted by any build.

The factual base is [authority-evidence.md](../authority-evidence.md), which
maps the existing verbs, digests and consumer fields with real outputs. This
note does not repeat it; it cites its sections as AE §n.

## 1. Where the requirement came from

A planning packet dated 2026-09-11 (the "September 11 realignment" of the
Statecraft family plan, kept outside this repository) proposes that
spec-spine compile deterministic **authority snapshots**, **authority-change
classifications against a trusted base**, **obligation records**, **context
closures** and **work scopes**, while a separate issuer grants run-specific
permits and executors enforce them. It numbers the ideas A01 to A10 and B01 to
B35, and assigns this repository A01 to A09, B01, B03 to B05, B11, B15, B17,
B18, B22, B24, B33 and B35.

The packet is input, not authority. Its sample commands and record names are
proposals, it approves nothing, and it assigns no spec ids. Where it disagrees
with this repository's approved specs, the specs win and the disagreement is
recorded in §10.

## 2. The boundary, stated before the backlog

Design note 02's test still applies: *would an adopter who will never run the
orchestrator still want it?* The packet adds a second, which this note adopts:

> spec-spine computes **deterministic payloads over explicit inputs**. It never
> holds a key in the core, reads a clock, resolves an identity, issues a
> permission, keeps a lease, executes a declared command on a stranger's behalf,
> or queries telemetry.

| Belongs here | Belongs to a consumer |
|---|---|
| Snapshot of the authority state of one tree, with exact coverage | Choosing which trees to snapshot, retaining them |
| Classification of a change against a trusted base | Deciding who may approve which class, and recording the approval |
| Obligation ids, their text digests and their declared verifiers | Running the verifiers, keeping their outputs |
| Required context closure with inclusion reasons and gaps | Deciding what to send to a model, and reporting what was truncated |
| Work scope: candidate territory, obligations, declared shared outputs, overlaps | Issuing a permit, enforcing it in an executor, leasing and fencing |
| Strict, offline verification of spec-spine's own records | Composing records from several producers, trust roots, revocation |
| Pure evaluation of explicit inputs (time, usage, policy) passed in by a caller | The clock, the usage counter, the policy store |

Ownership cannot confer anything on the right-hand side. A work scope names
where a candidate may write and what it must satisfy; it does not grant read
access, network, secrets or the right to execute, and nothing in a spec's
frontmatter can make it do so.

## 3. Findings the proposals rest on

Each was measured on 2026-09-11 against `75181a5` unless noted. AE §8 has the
tables.

| # | Finding | Evidence | Carried by |
|---|---|---|---|
| F1 | `verify-attestation` accepts unknown members and re-canonicalizes before checking the seal: an injected `"prCouple": {"ok": true}` verifies as `match` and `valid`, exit 0, in both scopes | AE §8; `verify_attestation.rs` `load_json`, `seal::verify` over `attestation_hash(loaded)` | 085 |
| F2 | The corpus recompute never compares `schemaVersion`: `"9.0.0"` recomputes as `match`, exit 0. The per-spec path reports a generic content mismatch instead of refusing | AE §8; `attest.rs` `verify_recompute` | 085 |
| F3 | `tool.version` is not a build identity. Released `v0.18.0` cannot attest 14 of 85 specs; a main build stamped `0.18.0` can; a per-spec attestation from the second is unverifiable by the first with exit 3, not the named `versionMismatch` | AE §8 | open decision D1 |
| F4 | `hash::content_hash` frames the path but not the content, so two trees can fold to one digest; a binary file and a text file holding `sha256:<its digest>` attest identically | AE §4 | 087 (new records only) |
| F5 | No attestation records committed-shard freshness, a config digest, or which governance files were read; `check --json` reports freshness as booleans without the digests compared | AE §4 | 087 |
| F6 | `couple` diffs from a merge base it never reports, and a corpus attestation's `couple` block is a different question (resolution, not diff) | AE §5 | 088, request R2 |
| F7 | C-001 clears on **any** owner's `spec.md` in the diff, and owners are read from the candidate's own index, so a candidate can weaken its own `## Verification` block, or claim territory with a new `extends`, and pass | AE §5; `couple.rs` "primary-owner heuristic"; reproduced in 088's `## Verification` | 088 (report only; note 02 G7 stands) |
| F8 | `registry plan --json`, `index owner --json`, `index coverage --json` carry no version and emit unsorted keys, contrary to `api.md` §7 | AE §2 | small fix, unfiled |
| F9 | `registry show` prints `contentHash ... (sha256 of this spec.md)` but the value is the path-prefixed shard hash, which differs from `specSourceHash` for the same file | AE §4 | small fix, unfiled |
| F10 | `docs/schema-versioning.md` lists registry and index at `1.0.0` and omits the attestation and envelope axes, and says the artifact DTOs deny unknown fields, which only `Config`, the edge items and `Unit` do | AE §2 | 085 |
| F11 | `index check` never compares the committed index body with a fresh resolution: it trusts the body to name its own span files and compares only `shardHash`. A shard rewritten so its spec owns nothing, `shardHash` untouched, reads fresh under `index check` and `check`, and `couple` then derives ownership from it. `.derived/` is bypassed, and no CI job diffs a regenerated index against the committed one | AE §5; `index.rs` `check_index_freshness`; spec 031 3.1 records the weaker comparison as a cost trade (a full `index` here takes 0.04 s against 0.03 s) | 086 |

## 4. Proposed records

**Every example in this section is a proposal.** Field names, versions and
digest rules may change before a spec that defines them is approved. The
`schemaVersion` values are what a first implementation would stamp; they are
not emitted today. Digests shown as `...` are placeholders.

### 4.1 Digest rules for new record types

Existing payloads keep their constructions forever, so historical evidence
stays interpretable (AE §4). New record types use one framed construction:

```
frame/1 = SHA-256( "spec-spine/frame/1" 0x00
                   for each piece, sorted by path (byte order):
                     kind   : 1 byte, 't' text | 'b' bytes | 'l' symlink target
                     u64be(len(path)) path
                     u64be(len(content)) content )
```

Text pieces keep the standing normalization (BOM stripped, CRLF and CR to
LF) so a checkout's line endings do not change a digest; `b` pieces are raw
bytes; `l` pieces are the link's target text. A record's own reference digest
is SHA-256 of its canonical JSON bytes, as for the existing attestations, and a
verifier hashes the bytes it holds rather than a re-serialization (085).

Nothing here is called a Merkle root. A membership proof would need a defined
tree, leaf encoding and proof format; §5 leaves that to a later increment that
has a consumer for selective verification.

### 4.2 AuthoritySnapshot (filed as draft 087)

One on-demand, sealable record of the authority state of a tree: what was
read, what it compiled to, whether the committed ledger matched, and what each
spec's own attestation hashes to.

```json
{
  "schemaVersion": "0.1.0",
  "tool": { "name": "spec-spine", "version": "0.19.0" },
  "digest": "frame/1",
  "config": { "present": true, "hash": "..." },
  "schemas": { "registry": "1.2.0", "index": "1.1.0", "corpusAttestation": "0.1.0", "specAttestation": "0.1.0" },
  "corpus": { "specs": 85, "inputsManifestHash": "65b0edf5...", "registryHash": "b6eabbbb..." },
  "committed": {
    "registry": { "files": 85, "hash": "...", "matchesRecompute": true },
    "index": { "files": 89, "hash": "...", "matchesRecompute": true }
  },
  "governanceInputs": { "paths": ["AGENTS.md", "CLAUDE.md", "spec-spine.toml", "..."], "hash": "..." },
  "verdicts": {
    "compile": { "ok": true, "errors": 0, "warnings": 0 },
    "lint": { "ok": true, "findingsHash": "37517e5f..." },
    "resolution": { "ok": true, "blocking": 0, "unresolved": 0 },
    "ownership": { "sourceFiles": 80, "claimed": 80, "floorOnly": 0, "unclaimed": 0 }
  },
  "specs": [
    { "id": "083-an-attestation-covers-the-territory-it-claims", "status": "approved",
      "implementation": "complete", "specAttestationHash": "ef0f30a7..." }
  ],
  "exclusions": {
    "resolverExclusions": ["target", "node_modules", ".derived", "dist", "build", ".next"],
    "stateDir": null,
    "unwitnessedAllowed": ["..."],
    "bypassPrefixes": [".github/", "docs/", "..."]
  }
}
```

It proves, for a tree and a tool version: exactly which inputs were read and
what they hashed to; whether the committed shards equal the recompute (the
freshness answer, carried as data rather than inferred from a recompute); the
gate verdicts; and a per-spec attestation hash for every spec, so one spec's
evidence can be checked against the snapshot without the rest. It does not
prove: anything about a revision (the consumer binds the tree, AE §6),
anything about unclaimed files beyond their count, the correctness of any
spec, or that anyone approved the state.

### 4.3 AuthorityDelta (filed as draft 088)

A classification of every path a change touches, computed under the **base**
revision's configuration so a candidate cannot reclassify its own change by
editing `spec-spine.toml`.

```json
{
  "schemaVersion": "0.1.0",
  "tool": { "name": "spec-spine", "version": "0.19.0" },
  "classifiedUnder": "base",
  "base": { "commit": "e4032ff2...", "mergeBase": "e4032ff2..." },
  "head": { "commit": "75181a51..." },
  "changes": [
    { "path": "specs/084-a-short-id-names-the-same-spec-at-every-verb/spec.md", "change": "added",
      "specId": "084-a-short-id-names-the-same-spec-at-every-verb",
      "classes": ["authority", "requirement", "verification", "lifecycle"],
      "authority": { "added": { "establishes": 3, "extends": 10, "references": 2, "amends": ["016-short-id-resolution"] } },
      "verification": { "basePlanHash": null, "headPlanHash": "..." } },
    { "path": ".derived/spec-registry/by-spec/084-a-short-id-names-the-same-spec-at-every-verb.json",
      "change": "added", "classes": ["derived"] }
  ],
  "counts": { "implementation": 0, "requirement": 1, "verification": 1, "authority": 1, "lifecycle": 1,
              "constitutional": 0, "policy": 0, "derived": 2, "bypassed": 0, "unowned": 0, "unknown": 0 },
  "priorPolicy": { "required": true, "classes": ["authority", "lifecycle", "requirement", "verification"] }
}
```

It proves which structural classes a change falls into under the base's rules.
It does not decide whether the change is acceptable, and it does not interpret
prose: any byte change to a spec body outside `## Verification` is a
`requirement` change, conservatively, whatever the words say.

### 4.4 Obligation records (proposed P1, not filed)

Stable ids for what a spec requires, so evidence can name what it evaluates.
Declared in frontmatter, compiled into the registry record:

```yaml
obligations:
  - { id: REQ-1, kind: requirement, section: "3.1", units: ["crates/spec-spine-cli/src/verify_attestation.rs"] }
  - { id: INV-1, kind: invariant, section: "3.3" }
  - { id: VER-1, kind: verification, verifies: [REQ-1, INV-1],
      inputs: ["crates/spec-spine-cli/tests/verify_attestation_tamper.rs"] }
```

```json
{ "id": "085-a-verifier-checks-the-bytes-it-was-given#REQ-1",
  "kind": "requirement", "sectionHash": "...", "units": [ { "kind": "file", "path": "..." } ],
  "verifiedBy": ["085-...#VER-1"] }
```

Rules a compile would enforce: an id is unique within its spec; a `verifies`
reference names an id that exists; a `section` names a heading that exists;
`sectionHash` is the frame/1 digest of that section's text, alongside the
spec's full-content hash, never instead of it. A `verification` obligation's
`inputs` are the files its commands read; declaring them is what lets a delta
(4.3) see that a candidate changed the tests that judge it, which today it
cannot (they classify as `implementation`). An evidence record produced by a
consumer references obligations as `{ "obligation", "subject", "relation":
"evaluates", "result", "evidence" }`; spec-spine can check that the ids exist
in a snapshot, never that the result is true.

### 4.5 ContextClosure (proposed P1, not filed)

The context a spec's work structurally requires, with a reason for every item,
under a named rule version:

```json
{
  "schemaVersion": "0.1.0",
  "rules": "closure/1",
  "snapshot": "<AuthoritySnapshot attestationHash>",
  "root": "085-a-verifier-checks-the-bytes-it-was-given",
  "required": [
    { "kind": "spec", "id": "085-...", "reason": "root", "shardHash": "..." },
    { "kind": "spec", "id": "023-ledger-seal", "reason": "depends_on", "shardHash": "..." },
    { "kind": "spec", "id": "084-a-short-id-names-the-same-spec-at-every-verb", "reason": "co-owner",
      "unit": "crates/spec-spine-cli/src/verify_attestation.rs", "shardHash": "..." },
    { "kind": "file", "path": "standards/spec/constitution.md", "reason": "constitution", "hash": "..." }
  ],
  "optional": [ { "kind": "file", "path": "docs/design/04-authority-evidence-extension.md", "reason": "references" } ],
  "missing": [ { "unit": { "kind": "file", "path": "crates/spec-spine-cli/tests/verify_attestation_tamper.rs" },
                 "reason": "planned-not-written" } ],
  "complete": false
}
```

`closure/1` would include the root, its transitive `depends_on`, every spec it
amends, extends, refines, supersedes or constrains, every spec that amends it,
every co-owner of a unit it owns, the constitution and contract, and the root's
owned units; `references` go to `optional`. A consumer that sends less than
`required` must report what it omitted, and a closure with anything omitted or
missing is not complete. It proves inclusion under those rules. It does not
prove that the graph is complete, that the context was sufficient, or that a
model read it.

### 4.6 WorkScope (proposed P1, not filed)

Where a candidate for one spec may write, what it must satisfy, and what it may
collide with. Not a permit.

```json
{
  "schemaVersion": "0.1.0",
  "snapshot": "<AuthoritySnapshot attestationHash>",
  "closure": "<ContextClosure hash>",
  "spec": "085-a-verifier-checks-the-bytes-it-was-given",
  "mutable": [
    { "unit": { "kind": "file", "path": "crates/spec-spine-cli/src/verify_attestation.rs" },
      "via": "extends", "coOwners": ["023-ledger-seal", "042-per-spec-attestation", "084-a-short-id-names-the-same-spec-at-every-verb"] },
    { "unit": { "kind": "file", "path": "crates/spec-spine-cli/tests/verify_attestation_tamper.rs", "planned": true },
      "via": "establishes" }
  ],
  "ownSpec": { "path": "specs/085-.../spec.md", "permittedEdits": ["claim-created-file", "dated-decision", "implementation-field"] },
  "shared": [ { "path": ".derived/", "reason": "regenerated" }, { "path": "Cargo.lock", "reason": "lockfile" } ],
  "obligations": ["085-...#VER-1"],
  "overlaps": [
    { "spec": "084-a-short-id-names-the-same-spec-at-every-verb", "kind": "file",
      "paths": ["crates/spec-spine-cli/src/verify_attestation.rs", "crates/spec-spine-core/src/attest.rs",
                "crates/spec-spine-core/src/lib.rs"] }
  ]
}
```

The overlap in this example is real: drafts 084 and 085 both extend those
files, so they are not safe to build in parallel even though neither depends
on the other. `registry plan` lists both as ready today and has no way to say
so. Disjoint scopes would still not prove independence (a shared lockfile, a
generated shard, an API another spec consumes); `shared` names the declared
ones and anything undeclared stays unknown, never "safe".

### 4.7 Verifier coverage rules

What an independent verifier must do with any spec-spine record, whoever
packages it:

1. **Hash the bytes it holds.** A record is referenced by SHA-256 over its
   exact stored bytes. Refuse bytes that are not the canonical serialization of
   their own parse (sorted keys, two-space indent, LF, one trailing newline);
   spec-spine writes nothing else.
2. **Parse strictly.** An unknown member or a duplicate key is a refusal, not
   an ignored extra.
3. **Dispatch on type and version.** The record type comes from the
   consumer's envelope; an unknown `schemaVersion` MAJOR is `unsupported`, which
   is neither pass nor fail. Each version keeps its own digest construction;
   never re-derive an old digest under a new rule.
4. **Keep outcomes separate.** Integrity, signature, subject binding,
   recompute, freshness and policy are each `pass`, `fail`, `unknown` or
   `not-applicable`. None is folded into another.
5. **Recompute only under the recorded tool version and subject tree.** A
   missing tree or a different version is `unknown`.
6. **Trust nothing the bundle selects for itself.** A key or trust root shipped
   inside a bundle does not authorize it.
7. **Never execute.** A verification plan is data. A verifier reads it and
   never runs it.

Draft 085 makes spec-spine's own `verify-attestation` satisfy 1 to 3 for the
existing attestations; the rest are properties of the consumer's verifier.

## 5. Increments, in dependency order

| Phase | Increment | Register ids | State | Exit evidence |
|---|---|---|---|---|
| P0 | **085** a verifier checks the bytes it was given | B33, B34 (prerequisite), A08 | draft, filed | every F1/F2 tamper row refuses; historical corpus and file-unit attestations still `match` |
| P0 | **086** the committed index is compared, not trusted | B11, B33 | draft, filed (amends 024 5) | a rewritten shard body reads stale under `index check`, `check` and `couple`; a regenerated tree reads fresh |
| P0 | **087** an authority snapshot says what it read | B11, A08, B33 | draft, filed | one record answers AE §4's "not covered" list for a tree; framed digests; recompute and seal via 085 |
| P0 | **088** a change is classified under the base's rules | A03, A08, B35 | draft, filed | weakening one's own plan, a new `extends` claim, a config edit and an ambiguous move each classify; `couple` alone passes all four |
| P1 | obligation records (4.4) | A01, A02, A08 | proposed | duplicate or unknown ids refuse at compile; declared verifier inputs reach the delta |
| P1 | context closure (4.5) | B04, B05 | proposed | missing and optional items explicit; a truncated delivery cannot claim completeness |
| P1 | work scope (4.6) | B01 | proposed | overlap between two ready specs is reported; scope references snapshot and closure digests |
| P1 | receipt links and an offline verifier | A01, B33, B34 | consumer-owned; spec-spine supplies fixtures | one existing-repo change explained and independently checked (AE §6 is the manual form) |
| P2 | declared impact and conflict sets | A04, A05, B03 | proposed | direct and transitive impact with provenance; unknown boundaries explicit |
| P2 | interface references and pinned imports | B22, A06 | proposed | a provider/consumer seam pinned by digest across two corpora in this family |
| P2 | reviewed move mapping | A07 | proposed | a rename needs an explicit mapping; git similarity is a suggestion only |
| P2 | typed overlays: effect contracts, budgets, dependency rationale, contract adapters | B15, B18, B24, A09 | proposed, via the overlay seam | overlay records validate and are digest-pinned; core `Unit` identity untouched |
| P2 | waiver lifecycle over explicit inputs | B17 | proposed | expiry and usage evaluated from caller-supplied time and usage; no counter in an authored file |

Nothing in P1 or P2 is filed. Each should be filed only after the P0 record
vocabulary it builds on is approved, because obligations, closures and scopes
all reference a snapshot digest.

## 6. The register, disposition by disposition

| Id | Proposal | Here | What spec-spine does | What stays outside |
|---|---|---|---|---|
| A01 | Evidence graph | P1 | obligation ids, typed references, validation that referenced ids exist | observations, retention, the claim that a result supports anything beyond the bounded obligation |
| A02 | Typed invariants | P1 | `invariant` obligations with scope and an optional overlay predicate reference | executing predicates; prose stays authored truth |
| A03 | Authority-transition policy | P0 (088) | classification against the base | identity, approval, the decision |
| A04 | Change-impact compiler | P2 | declared impact over existing edges, with provenance and unknown boundaries | semantic blast radius |
| A05 | Parallel-work planner | P1/P2 | overlap and shared-output sets (4.6) | reservation, scheduling |
| A06 | Cross-repository federation | P2 interface snapshots only | digest-pinned imports with namespace and version | discovery, fetching, trust in the exporter |
| A07 | Rename / move semantics | P2 | a reviewed mapping as authored data; 088 reports delete and add, never inferred identity | similarity detection as authority |
| A08 | Semantic fingerprints | P0/P1 | framed structured digests beside full-content hashes | any heuristic or model deciding a prose change is harmless |
| A09 | Machine-checkable contracts | P2 overlay | digest of the contract and of the verifier's declared inputs | running the checkers |
| B01 | Compile work permits | P1 as WorkScope | the scope | the permit, its issuer, its enforcement |
| B03 | Authority leases | P2 | conflict scopes | TTL, fencing, revocation, the lease store |
| B04 | Smallest sufficient context | P1 as required closure | required plus separately labeled optional | any claim of minimality or sufficiency |
| B05 | Context sufficiency proofs | P1 as closure certificate | inclusion under a named rule version | truncation by a provider, comprehension |
| B11 | Governance Merkle root | P0 as snapshot digests | 086 (the committed ledger is what it claims) and 087 (framed digests, coverage, exclusions) | membership proofs until a consumer needs selective verification |
| B15 | Effects as authority units | P2 overlay prototype | typed effect-contract references | mapping to real effects; `Unit` identity unchanged |
| B17 | Waiver half-life | P2 | pure evaluation of explicit time, ancestry and usage inputs | atomic consumption of usage |
| B18 | Constraint budgets | P2 overlay | typed declarations: dimension, unit, window, authority | measurement and enforcement |
| B22 | Interface boundaries | P2 | typed provider and consumer references with pinned digests | API-specific checks |
| B24 | Dependency justification | P2 overlay | declared rationale bound to package identity | any inference that a dependency is unused |
| B33 | Minimal proof protocol | P0/P1 | its own payloads, validated strictly (085), named with stable types and versions | the envelope, composition, trust decisions |
| B35 | Policy compiler, not runtime | throughout | pure functions of explicit inputs | keys, clocks, locks, approvals, I/O, revocation state |

## 7. Requests to other repositories

Each names the consumer that would act on it. None changes that repository's
contract from here; they are proposals for its own governance.

**statecraft-cli**

- **R1.** Parse envelopes instead of prose. `members/src/orchestrator/export.ts`
  (`runCorpusAttest`) and `adopt/holdback.ts` (`attestationHash`) match
  `attestationHash:` in prose; use `spec-spine attest --json` and read
  `.report.attestationHash`. The attestation's path is
  `<derived_dir>/attestation/attestation.json`, a function of the repository,
  not of stdout.
- **R2.** A receipt revision that carries spec-spine results, not just exit
  codes: per gate command, the envelope `schemaVersion` and SHA-256 of the
  envelope bytes (run with `--json`); an `authority` block with the corpus
  `attestationHash`, the spec's `SpecAttestation` hash and `specSourceHash`, and
  the plan digest (AE §6 is a worked example); and the merge base `couple`
  diffed from. When 087 lands, the snapshot hash replaces most of that block.
- **R3.** Read acceptance through `spec-spine verify --plan --json` rather than
  a second `## Verification` parser (`stages/verify.ts`), and take the plan from
  the base revision's copy of the spec, recording its `specSourceHash`.
  `verify:browser` blocks appear under `skipped`, so the CLI's browser path
  keeps working.
- **R4.** Hash stored attestation bytes and refuse unknown members yourself
  until a release contains 085 (AE §8). `export.ts` already hashes the bytes;
  keep that and add the member check.
- **R5.** Do not promise the spec 083 subtree fix to users of `v0.18.0`. The
  pin is `required_version = "0.18.0"` (caret) and CI installs `main`'s
  `install.sh` unpinned, which resolves to `v0.18.0`, which predates 083. After
  the release that contains it, raise the floor and pin the installer to the
  same version.
- **R6.** Decide whether the universal gate floor should add
  `--fail-on-unresolved` to `check` (this repository's floor has it; the CLI's
  does not), and record 1, 2 and 3 as distinct failures in evidence.
- **R7.** Run declared acceptance only in an explicitly authorized worker,
  behind the credential fence (spec 125) or the sandbox draft (126); not in a
  process holding credentials.

**Statecraft (hosted control plane)**

- **S1.** Never execute `## Verification` commands in the control plane. Accept
  spec-spine envelopes and attestation payloads, recomputed or verified by a
  trusted worker under the recorded tool version.
- **S2.** Store `{repo, commit, tree}` beside every spec-spine digest; treat a
  `versionMismatch` or an unavailable tree as `unknown`.
- **S3.** Evaluate 088's `priorPolicy.required` classes under the base's policy
  and record the approval reference in the permit; spec-spine only classifies.
- **S4.** Lead the composition envelope (payload type names, media types, any
  in-toto predicate type). spec-spine will supply stable type names and
  versions for its payloads and review the mapping.

**hqgit**

- **H1.** Reference spec-spine records by `(type, schemaVersion, SHA-256 of the
  record bytes)` together with the subject tree and commit. Where hqgit's own
  identifiers use a different algorithm or encoding, keep both in the cross
  reference with explicit algorithm ids; do not re-derive spec-spine digests.

**Fixtures offered to all three:** the tamper and cross-version cases of AE §8,
as runnable commands in 085's `## Verification` block, and AE §6's worked
binding, which a verifier can reproduce from the public history of this
repository.

## 8. Declared versus enforced

| Guarantee as declared | Where declared | Enforced today? |
|---|---|---|
| A single tampered payload byte fails the signature | spec 023 AC-4 | For existing members only. Added members and reformatting verify (F1) |
| Loaders reject an unknown schema MAJOR | `docs/schema-versioning.md`; `types/src/attest.rs` | For registry and index loaders. Not for attestation verification (F2) |
| Artifact DTOs deny unknown fields | `docs/schema-versioning.md` | `Config`, edge items and `Unit` only (F10) |
| Every emitted JSON document has sorted keys | `api.md` §7 | Not `registry plan`, `index owner` or `index coverage` (F8) |
| `tool.version` is the reproducibility anchor | spec 023 FR-005 | Not a build identity (F3) |
| The corpus attestation with coupling covers "specs and code in sync" | spec 023 FR-002 | Resolution only; no code bytes and no diff (AE §4, §5) |
| `verify` never enters the gate chain | spec 049 3.6 | Yes: CI and the floor exclude it |
| No lint rule consumes a `SpecAttestation` | spec 042 3.4 | By review only, as 042 says itself |
| An agent does not weaken the spec it is implementing | `.claude/rules/adversarial-prompt-refusal.md` | A prompt; the gate passes it (F7) |
| A fresh `check` means the committed shards are exactly what the corpus compiles to | `AGENTS.md` (Freshness) | For the registry. For the index, only the hash fields are compared (F11) |

## 9. Decisions this note does not make

- **D1. Build identity.** Options: stamp new record types with the source
  commit (reproducible per source, but absent from a crates.io build); give
  `main` a pre-release version after every release (distinguishes builds, but
  this repository's own `required_version = ">=0.17.0"` does not match
  pre-release versions under Cargo's semver rules); or document the limit.
  Recommendation: document now, decide with the next release.
- **D2. Refusal codes in 085.** The draft refuses unknown members and unknown
  MAJOR versions at exit 3 (parse and schema, the standing contract) and makes
  a non-canonical byte sequence a failed verification at exit 1. A reviewer may
  prefer one named `unsupported` outcome at exit 1 for all three.
- **D2a. The 024 amendment in 086.** Comparing index bytes reports a
  sibling-caused resolution flip that spec 024 5 deliberately left unreported.
  The draft declares `amends: 024`; a reviewer may instead prefer to make only
  the coupling gate resolve ownership afresh (086 D-2 records why the draft
  does not).
- **D3. What 087 embeds per spec.** The draft embeds each `SpecAttestation`
  hash, which inherits F4 for unit bytes. The alternative is a framed territory
  digest per spec, which is new work and a second hash over the same files.
- **D4. Where prior-policy enforcement lives.** Recommended: in the consumer
  that issues permits, reading 088. A refusing mode in `couple` is note 02 G7
  and should stay design-first.
- **D5. Obligation vocabulary.** Kinds, id syntax, frontmatter versus a fenced
  block, and whether `constraint` is a fourth kind.
- **D6. The neutral verifier's home.** A family decision; spec-spine's part is
  the fixtures and strict payload validation in its own crates.
- **D7. Envelopes for the unversioned reads.** Wrap `registry plan`,
  `index owner` and `index coverage` in the spec 037 envelope, or add a version
  field to each.
- **D8. Constitutional edits and bootstrap.** Whether any change to tier-1
  anchors or `standards/spec/` can be accepted under the candidate's policy
  (this note's answer is no), and what authorizes the first snapshot a consumer
  trusts.

## 10. Where the packet and this repository disagree

- **Baseline.** The packet records `HEAD 6fca5ee` with 083 draft. At `75181a5`,
  083 is approved (ratified in #176) and 084 is filed as a draft (#177): 85
  specs, 84 approved and complete, one draft and pending.
- **Enforcement.** The packet asks for prior-policy handling. Note 02 G7 says no
  mechanism should be asserted before it is designed, because the naive rule
  refuses legitimate refactors. 088 therefore reports and does not refuse.
- **Committed evidence.** None of the proposed records is committed. Specs 042
  3.3 and note 02 §5 explain why: a committed evidence tree needs its own
  freshness gate and restales on every edit.
- **Vocabulary.** This repository reserves "certificate" for testimony of a
  non-reproducible event (spec 023 §7) and uses "attestation" for recomputable
  records. The packet's "proof" language is avoided here for the same reason it
  gives: a digest commits to bytes; it proves nothing about intent.
- **Merkle.** Not used. The existing folds are sorted hashes, not trees.
