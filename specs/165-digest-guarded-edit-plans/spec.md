---
id: "165-digest-guarded-edit-plans"
title: "Plan a governed edit without applying it"
status: draft
kind: "governance"
created: "2026-09-27"
implementation: pending
owner: "The spec-spine Authors"
risk: high
depends_on:
  - "037-amendment-authoring"
  - "106-obligations-are-declared-constraints"
  - "108-a-work-scope-is-declared"
  - "155-selected-content-accessor"
  - "160-documentation-manifest-and-freshness"
  - "169-declared-obligation-traceability"
summary: >
  Adds a pure, read-only edit-plan producer: a closed list of governed
  operations, each guarded by the digest of the target it was planned
  against, checked against ownership, a work scope and the amendment rules,
  previewed as exact post-image bytes, and classified by authority effect.
  Refusals come from a closed set. Nothing in spec-spine applies a plan.
establishes:
  - { kind: file, path: "crates/spec-spine-types/src/edit_plan.rs", planned: true }
  - { kind: file, path: "crates/spec-spine-types/schemas/edit-plan.schema.json", planned: true }
  - { kind: file, path: "crates/spec-spine-core/src/edit_plan.rs", planned: true }
  - { kind: file, path: "crates/spec-spine-core/tests/edit_plan.rs", planned: true }
  - { kind: directory, path: "crates/spec-spine-core/tests/fixtures/edit-plan/", planned: true }
  - { kind: file, path: "crates/spec-spine-cli/src/cmd_edit_plan.rs", planned: true }
  - { kind: file, path: "crates/spec-spine-cli/tests/edit_plan.rs", planned: true }
  - { kind: file, path: "docs/edit-plans.md", planned: true }
extends:
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-types/src/lib.rs" }, nature: additive }
  - { spec: "074-a-governed-read-names-its-version", unit: { kind: file, path: "crates/spec-spine-types/src/version.rs" }, nature: additive }
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-core/src/lib.rs" }, nature: additive }
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-cli/src/main.rs" }, nature: additive }
  - { spec: "057-the-docs-name-what-adopters-derived", unit: { kind: file, path: "docs/api.md" }, nature: additive }
  - { spec: "057-the-docs-name-what-adopters-derived", unit: { kind: section, file: "docs/cli-reference.md", anchor: "cli-reference" }, nature: additive }
  - { spec: "057-the-docs-name-what-adopters-derived", unit: { kind: file, path: "docs/schema-versioning.md" }, nature: additive }
  - { spec: "152-a-refusal-says-what-it-is-in-every-form", unit: { kind: file, path: "crates/spec-spine-cli/tests/refusal_envelopes.rs" }, nature: additive }
references:
  - { unit: { kind: file, path: "docs/design/09-disposition-2026-09-21.md" }, role: "roadmap row K" }
  - { unit: { kind: file, path: "crates/spec-spine-core/src/compact.rs" }, role: "plan and apply precedent" }
  - { unit: { kind: file, path: "crates/spec-spine-types/src/moves.rs" }, role: "authored moves, distinct from plans" }
  - { unit: { kind: file, path: "crates/spec-spine-core/src/scope.rs" }, role: "work scope evaluation" }
  - { unit: { kind: file, path: "AGENTS.md" }, role: "adversarial prompt refusal and the two legitimate edits" }
  - { unit: { kind: file, path: "specs/162-capability-catalog/spec.md" }, role: "the catalog this verb's record joins (D-9)" }
obligations:
  - id: "R-1"
    kind: requirement
    text: "Every operation names one target identity and the digest it was planned against, and a target whose current digest differs is refused, never rebased."
    anchor: "3-9-every-target-is-digest-guarded"
  - id: "R-2"
    kind: requirement
    text: "An amendment is proposed only as a new draft spec carrying an amends edge, and no operation changes the requirement text of an approved spec."
    anchor: "3-6-propose-amendment-creates-a-new-spec"
  - id: "R-3"
    kind: requirement
    text: "Every accepted operation carries exactly one authority effect from a closed ordered set, and the plan carries the greatest of them."
    anchor: "3-13-authority-effect-classification"
  - id: "R-4"
    kind: requirement
    text: "A refused plan lists every refusal of every operation from a closed reason set and carries no preview."
    anchor: "3-14-refusal-reasons-are-closed"
  - id: "R-5"
    kind: requirement
    text: "Two operations whose target identities or pre-image line ranges intersect refuse the plan as overlapping."
    anchor: "3-11-overlap-and-conflicts"
  - id: "I-1"
    kind: invariant
    text: "Planning never writes, formats, executes, applies, approves, waives, grants authority, or performs recovery."
    anchor: "3-16-nothing-applies-a-plan"
  - id: "V-1"
    kind: verification
    text: "The implemented producer is checked for every operation, authority effect, refusal reason, overlap rule, digest guard, canonical byte rule, and the no-write boundary."
    anchor: "verification"
    inputs:
      - "crates/spec-spine-core/tests/edit_plan.rs"
      - "crates/spec-spine-cli/tests/edit_plan.rs"
intent:
  goal: "let an agent obtain a reviewable, digest-guarded, authority-classified proposal for a governed edit without spec-spine ever applying it"
  non_goals:
    - "a general patch language or edits to source code"
    - "applying, formatting, committing, or recovering edits"
    - "deciding whether an edit is correct or approving one"
    - "replacing couple, delta, or human ratification as the judge of a change"
---

# 165: Plan a governed edit without applying it

## 1. Purpose

Agents make the same small governed edits repeatedly: claim a file, declare a
trace relation (169), record that an approved obligation must change, refresh
a documentation manifest (160), or rewrite a section of their draft. Each
edits YAML by hand, learns from `compile`, `lint` or `couple` afterward that
the edit was malformed or unauthorized, and sometimes takes the forbidden
repair: editing an approved spec to make code pass (AGENTS.md, spec 037).

This spec adds a producer that answers, before anything is written, what an
edit would be, whether its target is still what the agent read, what
authority it touches, and why it is refused. The plan is canonical data with
exact post-image bytes; Statecraft applies it under its own authority.

### 1.1 Why spec-spine and not Statecraft

Design note 09 row K assigns planning here and application to Statecraft, and
warns against a general patch language. The split holds only where
correctness needs what spec-spine alone knows: the frontmatter grammar,
ownership, obligation and section identities, the amendment rules, and
post-image compilation. A code patch needs none of that and is refused.

## 2. Territory

- `edit_plan.rs` in the types crate and `edit-plan.schema.json` define the
  request, operation, preview, conflict, refusal, effect and plan documents.
- `edit_plan.rs` in core builds a plan from a config, one repository export
  and a request, with no Git, network, execution or writes.
- `crates/spec-spine-core/src/lib.rs` exposes typed planning and
  `edit_plan_json(config_json, repo_root, request_json, snapshot_json)`.
- `crates/spec-spine-cli/src/cmd_edit_plan.rs` and `main.rs` expose
  `edit-plan --request <path> --repository <identity> [--revision <rev>]
  --json`, binding the snapshot exactly as `content select` does (155 §3.2).
- The planned core and CLI tests and the fixture directory hold the evidence;
  `docs/edit-plans.md`, the API, CLI and schema-versioning documents describe
  the contract. The refusal-envelope inventory gains `edit-plan`. The verbs
  and documents this calls stay owned by their specs, unchanged.

## 3. Behavior

### 3.1 One request one snapshot one subject

A plan MUST be computed against, and repeat, exactly one spec-155 snapshot
identity. A dirty or changing snapshot is refused under 155's rules first. The
committed registry and index MUST be fresh for the snapshot, else the plan is
refused `stale-ledger`: ownership read from a stale ledger is no answer.

Every request names one subject: the spec whose work the edit serves. The
subject is the `ownSpec` of a spec-108 work scope carried in the request.

### 3.2 The request document

```json
{
  "scope": { "ownSpec": "165-digest-guarded-edit-plans", "mutable": [] },
  "operations": [],
  "manifests": [],
  "consumerSchemaVersion": "1.0.0"
}
```

`scope` is a spec-108 scope document, parsed by 108's rules. `operations` is
non-empty and at most 64 entries; an empty array is a usage error (exit 3), and
more than 64 is `plan-too-large` (§3.15). `manifests` optionally carries spec-160
manifest documents the caller asserts describe outputs in this snapshot; they
are used only to identify generated regions (§3.10). Unknown members, an
unsupported consumer schema major, or an operation of unknown kind are usage
errors (exit 3).

### 3.3 The closed operation vocabulary

Every operation has an `op` from exactly this set, a `target`, and an
`expected` digest (§3.9). There is no sixth kind.

| `op` | Target | Changes |
|---|---|---|
| `add-claim` | the subject's `spec.md` | one unit appended to `establishes`, or one `extends` edge |
| `add-trace-relation` | the subject's `spec.md` | one spec-169 `traceability` entry |
| `propose-amendment` | a new `specs/<id>/spec.md` | a new draft spec that amends an approved one |
| `update-manifest` | one documentation-manifest file | the whole file, replaced by a caller-supplied manifest |
| `replace-unit` | one uniquely resolved spec or documentation unit | that unit's bytes |

Moves (111) and relocations (142) are not operations (D-2).

### 3.4 add-claim

The operation carries one unit in the frontmatter edge grammar, the edge
(`establishes` or `extends`), and for `extends` the named spec and `nature`.
The planner MUST refuse `duplicate-declaration` when the subject already
declares that unit on that edge, and `unauthorized-target` when an
`establishes` claim names a unit another spec already establishes; that path
belongs on an `extends` edge (AGENTS.md). An `extends` edge MUST NOT be refused
because the named spec does not itself claim the unit: an extends-carried unit
is a first-class claim. A unit whose path does not exist MUST carry
`planned: true`, or is refused `unresolved-target`.

The entry is rendered as one flow-style line appended to the existing block
sequence, or as a new block key before the closing fence when the key is
absent. A frontmatter whose edge key uses a flow sequence, anchors, aliases or
comments between entries is refused `unsupported-layout` rather than
re-serialized.

### 3.5 add-trace-relation

The operation carries one spec-169 entry, validated by 169's compile rules on
the post-image. A repeated id or tuple is `duplicate-declaration`, an
`ambiguous` target is `ambiguous-target`, and an interface target naming no
single declared spec-110 reference or unresolved local unit is
`unresolved-interface`. An `unresolved`, `unsupported` or `unknown` target is
accepted with its predicted 169 state, since 169 keeps such declarations valid
(169 §4).

### 3.6 propose-amendment creates a new spec

The operation names the qualified obligations it would change
(`<spec>#<id>`, each resolving under spec 106), the new spec's full id, and the
new file's complete bytes, authored by the caller. The planner MUST verify
that:

- the new path does not exist (its `expected` is `absent`) and the ordinal is
  unused and next free under the corpus's contiguity rule;
- the file parses, compiles, and is `status: draft`,
  `implementation: pending`;
- every target obligation's spec is `approved`, else
  `amendment-target-not-approved` (a draft is edited directly);
- `amends` names every target spec, and `amends_sections` names each target
  obligation's anchor;
- no named anchor is in the target's `unamendable` list, else
  `unamendable-anchor`;
- each changed obligation has a spec-109 `impacts` entry, with `supersedes`
  naming a successor obligation the new spec declares; and
- no operation in the same plan targets the amended spec's file.

The planner MUST NOT emit any byte change to the amended spec. It never
proposes editing an approved spec, marking it superseded, or rewriting its
Verification block; `amends_verification` in the new file is carried as the
caller wrote it and checked by the existing compile rules.

### 3.7 update-manifest

The target is one repository path holding a spec-160 manifest, and `expected`
is its recorded `manifestDigest`. The replacement is a complete manifest the
caller supplies, normally produced by a generator run. The planner MUST
validate it under spec 160 against the plan's snapshot and refuse
`manifest-invalid` otherwise. That validation recomputes `manifestDigest` and
checks every recorded packet and repository-local input digest against the
snapshot (160 §3.3, §3.4), so a manifest whose input digests were invented is
refused here. Output digests describe files the change itself writes, which
the snapshot does not yet hold; they are checked after application by spec
160's freshness read, which reports a mismatch as `output-changed`. The
planner MUST NOT compute, copy or rewrite a digest inside a manifest: a plan
that updated recorded digests without a generator run would launder
freshness.

### 3.8 replace-unit

The target is one spec-155 selector of kind `spec-section`, or `owned-unit`
or `file` naming a Markdown documentation file or section. It MUST resolve to
exactly one span (`ambiguous-target` otherwise). A symbol, module, test, or
any path outside Markdown documentation and spec files is refused
`unsupported-target-kind`. The replacement is the complete new bytes of that
span, which MUST end at a line boundary.

On an approved subject spec, only an append to its decisions section is
admissible. That section is the one level-2 heading whose text, after any
leading `N.` ordinal, begins with `Resolved decisions`, `Design decisions`,
or `Decisions` (every heading form the corpus uses), selected as a spec-155
`spec-section` by its anchor. A spec with no such heading, or with more than
one, has no admissible append, and the edit is refused `approved-spec-text`.
For the append: the old section bytes MUST be a prefix of the new. Any other
change to an approved spec's text is refused `approved-spec-text`, and the
refusal names `propose-amendment` as the governed route.

### 3.9 Every target is digest guarded

Each operation's `expected` is the digest the caller read the target at:

- a spec-155 item digest for the `full` projection of a spec, section, file or
  owned unit;
- a spec-160 `manifestDigest` for `update-manifest`; or
- the literal `absent` for a file that must not exist.

The planner MUST recompute the digest from the snapshot and refuse
`stale-target` on any difference, naming both digests. It MUST NOT rebase,
merge, or re-resolve the operation against the current bytes. For every file
a plan changes, the preview also carries `beforeRawDigest`, a SHA-256 over the
file's raw bytes, which is the guard an applier checks before writing.

A target whose raw bytes differ from their spec-155 normalization (a BOM, CR
or CRLF) is refused `unsupported-encoding`, since exact post-image bytes for
it could not be both normalized and faithful.

### 3.10 Authority is checked before a preview exists

For every file a plan changes or creates, in this order (a file
`propose-amendment` creates passes all four: check 3 exempts it from
`unauthorized-target`, and check 4 does not apply because a `spec.md` is not a
documentation path):

1. A path under the configured `derived_dir` or `state_dir`, or inside a
   `generated` region of a supplied manifest, is `generated-target`.
2. A path outside the scope's `mutable` and `shared` entries is
   `out-of-scope`.
3. A `spec.md` other than the subject's (except the new file of
   `propose-amendment`) is `unauthorized-target`.
4. A documentation path no spec owns, by the function `index owner` uses, is
   `unowned-target`; one the subject does not own is `unauthorized-target`.

A scope is not a permission (108 §3.7): it can only refuse.

### 3.11 Overlap and conflicts

Operations are ordered canonically by target path, operation kind, then
canonical operation identity: the canonical JSON bytes (§3.15) of the
operation object's value, `expected` included, after canonicalization, so
the whitespace and key order a request was written with never change it;
compared bytewise. Two operations overlap, and the plan is refused
`overlapping-operations` naming both, when they name the same target identity,
when a `replace-unit` span intersects another operation's pre-image lines, or
when one replaces a file another changes. Several frontmatter additions to one
spec do not overlap; they compose in canonical order into one post-image.

`conflicts` also carries non-refusing records from a closed set:
`shared-ownership` (a claimed path has other owners, listed) and
`declared-conflict` (a changed obligation carries a spec-109 `conflicts`
entry). They inform routing and never change the effect.

### 3.12 Previews are exact bytes

After every check passes, the planner applies the operations in memory, per
file, and MUST compile every changed or new spec's post-image through the
single-spec compile path. A validation error refuses `post-image-invalid`;
lint warnings on the post-image are carried as `predictedFindings`.

Each file preview carries `path`, `beforeRawDigest` (or `absent`),
`afterDigest`, the authoritative `afterContent` (exact LF bytes), and a
derived three-line-context unified `diff`. The plan names the derived trees
the change would stale and regenerates none.

### 3.13 Authority effect classification

Each operation carries one `authorityEffect`, ordered
`none` < `claims-territory` < `changes-obligation` <
`requires-human-ratification`:

| Case | Effect |
|---|---|
| `add-claim` on a draft subject, or an approved subject not yet `complete` | `claims-territory` |
| `add-claim` on an approved, `complete` subject | `requires-human-ratification` |
| `add-trace-relation` on a draft subject | `none` |
| `add-trace-relation` on an approved subject | `requires-human-ratification` |
| `propose-amendment` | `requires-human-ratification` |
| `update-manifest` | `none` |
| `replace-unit` on a draft span that is an obligation anchor or `verification` | `changes-obligation` |
| other `replace-unit` on a draft or on documentation | `none` |
| decision append on an approved subject | `requires-human-ratification` (D-4) |

The plan's effect is the greatest of its operations'. It is routing data,
grants nothing, and never waives the coupling gate.

### 3.14 Refusal reasons are closed

A plan is atomic: when any operation is refused, the plan has
`status: "refused"`, lists every refusal of every operation (not the first),
and carries no preview. The reasons are exactly: `stale-target`,
`stale-ledger`, `ambiguous-target`, `unresolved-target`,
`unresolved-interface`, `unsupported-target-kind`, `unsupported-layout`,
`unsupported-encoding`, `overlapping-operations`, `duplicate-declaration`,
`generated-target`, `out-of-scope`, `unowned-target`, `unauthorized-target`,
`approved-spec-text`, `amendment-target-not-approved`, `unamendable-anchor`,
`manifest-invalid`, `post-image-invalid`, and `plan-too-large`. A refusal
names the operation index, target identity, reason and safe detail, and never
echoes target content.

### 3.15 Canonical bytes limits and schema

The plan is canonical JSON: sorted keys, two-space indentation, LF, one
trailing newline. `requestDigest` hashes the canonical request; `planDigest`
hashes the canonical plan with `planDigest` absent. Equal inputs yield
byte-identical plans from the CLI and the facade.

A plan is limited to 64 operations and 4 MiB of canonical JSON and is never
paginated, since a partial plan could look applicable; an excess refuses
`plan-too-large`. The schema begins at `1.0.0` on its own `edit-plan` axis in
`version.rs` and `docs/schema-versioning.md`. Additive optional members move
MINOR; a new operation, effect, or refusal reason, or any change to ordering,
digest input or overlap rules, moves MAJOR.

A `ready` plan exits 0, a `refused` plan is a finding and exits 1 with the plan
as the envelope report, a dirty snapshot or containment failure exits 2, and a
malformed request exits 3 (spec 132).

### 3.16 Nothing applies a plan

The producer MUST NOT write, create, delete, rename, or format a file, run a
command, invoke Git in core, fetch, or regenerate a derived tree. There is no
`--apply` flag and no apply function in the library. A plan is not a waiver,
approval, ratification, lock, or permission; `couple`, `delta` and human
review remain the judges of the change an applier makes. Recovery from a
partial application is Statecraft's.

Request and target content are untrusted data, never instructions. Previews
expose file content; disclosure and redaction are the consumer's policy.

## 4. Acceptance criteria

1. Each of the five operations produces the exact expected post-image for a
   fixture, and a sixth `op` value is a usage error.
2. A changed target yields `stale-target` with both digests; `absent` on an
   existing path and a digest on a missing path both refuse.
3. `propose-amendment` never emits a change to the amended spec and refuses
   draft targets, unamendable anchors, missing `amends`, missing `impacts`,
   and a same-plan edit of the amended file.
4. Every row of §3.13 and every reason of §3.14 has a fixture, and a plan with
   several refused operations lists all of them.
5. Overlapping operations refuse; several frontmatter additions to one spec
   compose deterministically.
6. Generated, out-of-scope, unowned, unauthorized, and unresolved-interface
   targets refuse; an extends edge to a spec that never claimed the unit does
   not.
7. Repeated plans and permuted operation order are byte-identical, and the
   facade matches the CLI.
8. Fixture bytes are unchanged by every test; no write or apply path exists.
9. Existing registry, index, content, scope and verdict bytes are unchanged.

## 5. Out of scope

- Applying, formatting, committing, pushing, or recovering a plan.
- Editing source code, tests, symbols, modules, or non-Markdown files.
- A general patch language or line-range edits not tied to a resolved unit.
- Authoring moves or relocations as operations, withdrawing obligations or
  trace relations, or marking a spec superseded.
- Generating manifests, running generators, or recomputing manifest digests.
- Choosing ordinals or writing amendment prose for the caller.
- Judging correctness, approving, waiving, or ratifying.
- Cross-repository plans or multi-snapshot transactions.

## 6. Resolved decisions

**D-1 (2026-09-27): the producer belongs here, narrowly.** Frontmatter
grammar, ownership, amendment rules and post-image compilation are spec-spine
knowledge Statecraft would otherwise copy. Anything needing none of them,
chiefly code edits, stays out (§1.1).

**D-2 (2026-09-27): distinct from compact, moves and relocations.** Compact
(096) is plan in, bytes out, but its CLI writes. Moves (111) and relocations
(142) are authored frontmatter that compile and `delta` verify after the fact;
they are not proposals, and a draft author writes them directly.

**D-3 (2026-09-27): refusal is atomic and exhaustive.** Partial plans invite
partial application, which is exactly the recovery problem this spec leaves
to Statecraft; listing every refusal saves a caller one round per defect.

**D-4 (2026-09-27, draft, owner to confirm): a decision append on an approved
spec is `requires-human-ratification`.** AGENTS.md calls a dated decision
entry legitimate, but a planner cannot test whether the spec was silent on the
choice, and a decision entry can restate a requirement. The alternative is
`none` with a warning.

**D-5 (2026-09-27, draft, owner to confirm): ship `update-manifest` and
`replace-unit` with the claim operations.** Design note 09 recommends starting
with add-claim and add-trace only. If the owner prefers, the build may defer
both to a later spec; the vocabulary stays closed and a deferred kind is a
usage error until then.

**D-6 (2026-09-27, draft, owner to confirm): the spec-155 item digest is the
text guard.** The alternative, a spec's registry `shardHash`, covers only
whole specs and not documentation units; it is still checked indirectly
through `stale-ledger`.

**D-7 (2026-10-10, refile): traceability is spec 169.** The draft named spec
161, which was refiled as 169 before it reached `main`. Every reference now
names 169, and §3.5 accepts 169's `unknown` state alongside `unresolved` and
`unsupported`, since 169 §4 keeps all three valid at compile. `ambiguous` stays
a refusal here although 169 accepts it as data: an edit planned against a
target that binds twice cannot have one exact post-image meaning.

**D-8 (2026-10-10, refile): the planner follows 169's release.** Spec 169's
registry member lands with the release that carries it (169, decision of
2026-10-10), and this repository's gate runs the pinned released engine. The
build of this spec therefore follows a release carrying 169's build, and
`add-trace-relation` is compiled through the same binary's single-spec path
(§3.12), so the planner never predicts a grammar its own compile refuses.

**D-9 (2026-10-10, refile): the verb is cataloged.** Spec 162 holds every CLI
form and facade function to one catalog record (162 R-1). Whichever of 165 and
162 is built second adds `edit-plan` and `edit_plan_json`. Effects: reads
`caller-file`, `config`, `corpus`, `derived-ledger` and `source-tree`; executes
`git` for the snapshot binding exactly as `content select` does; writes
nothing; no network; authority empty, since a plan grants none (§3.16).

**D-10 (2026-10-10, review): three gaps in the filed text closed.** Review of
the filing found that an empty `operations` array had no stated outcome, that
§3.11's tiebreaker "canonical operation identity" was undefined, and that
§3.7 did not say whether a manifest with invented digests could pass. §3.2 now
makes an empty array a usage error; §3.11 defines the identity as the
operation's canonical JSON bytes; §3.7 states which digests spec 160's
validation checks at plan time (manifest, packet, and inputs) and which only
freshness can check after application (outputs).

**D-11 (2026-10-10, review): three definitions completed.** Review of D-10's
change found that §3.11's identity still read as the request's raw bytes, that
§3.8 never said how the decisions section is found, and that §3.10's checks did
not say whether they apply to a file `propose-amendment` creates. §3.11 now
canonicalizes the operation's value first; §3.8 identifies the section by its
heading forms in this corpus and refuses when there is not exactly one; §3.10
applies all four checks to created files and says how checks 3 and 4 treat the
new spec.

**D-12 (2026-10-10, refile): a forward `depends_on`, kept on purpose.** This
spec depends on 169, a higher ordinal, because traceability was refiled at 169
after this draft took 165. Every other `depends_on` in the corpus points
backward. The edge stays because `registry plan` must not offer this spec as
ready before 169 is complete, which a `references` entry would not prevent.
It cannot form a cycle: 169 depends only on 106, 110 and 155. Spec 046's
`L-007` would refuse it, and this repository has not enabled that opt-in rule.

**D-13 (2026-10-10, review): the catalog is referenced, not depended on.**
Review asked that spec 162, which D-9 coordinates with, appear in the
frontmatter. It is now a `references` entry, not a `depends_on`: the planner
needs nothing from the catalog to work, and whichever spec is built second
adds the record, so neither has to be complete before the other.

## Verification

Each named test target fails before the build, because it does not exist while
`implementation: pending`. `cargo test --workspace` is the regression floor and
passes either way.

```verify:cli
cargo test -p spec-spine-core --test edit_plan --locked
cargo test -p spec-spine-cli --test edit_plan --locked
cargo test --workspace --locked
```
