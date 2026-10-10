---
id: "169-declared-obligation-traceability"
title: "Declare qualified obligation traceability"
status: draft
kind: "governance"
created: "2026-09-28"
implementation: pending
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "106-obligations-are-declared-constraints"
  - "110-an-interface-reference-is-digest-pinned"
  - "155-selected-content-accessor"
summary: >
  Adds optional, qualified relations from declared obligations to implementation
  units, tests, invariants, documentation sources, and producer or consumer
  interfaces. Declarations and their deterministic resolution remain distinct
  from execution, evidence, acceptance, release, adoption, and observation;
  every unsupported, unknown, ambiguous, withdrawn, or unresolved target is
  explicit, and ownership is never behavioral proof.
establishes:
  - { kind: file, path: "crates/spec-spine-types/src/traceability.rs", planned: true }
  - { kind: file, path: "crates/spec-spine-types/schemas/traceability.schema.json", planned: true }
  - { kind: file, path: "crates/spec-spine-core/src/traceability.rs", planned: true }
  - { kind: file, path: "crates/spec-spine-core/tests/traceability.rs", planned: true }
  - { kind: directory, path: "crates/spec-spine-core/tests/fixtures/traceability/", planned: true }
  - { kind: file, path: "crates/spec-spine-cli/tests/traceability.rs", planned: true }
  - { kind: file, path: "docs/traceability.md", planned: true }
extends:
  - { spec: "000-spec-spine-bootstrap", unit: { kind: file, path: "crates/spec-spine-types/src/frontmatter.rs" }, nature: additive }
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-types/src/registry.rs" }, nature: additive }
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-types/src/lib.rs" }, nature: additive }
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-types/src/version.rs" }, nature: additive }
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-types/schemas/registry.schema.json" }, nature: additive }
  - { spec: "022-index-sharding", unit: { kind: file, path: "crates/spec-spine-types/schemas/registry-spec-shard.schema.json" }, nature: additive }
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-core/src/compile.rs" }, nature: additive }
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-core/src/lib.rs" }, nature: additive }
  - { spec: "002-registry-query", unit: { kind: file, path: "crates/spec-spine-cli/src/cmd_registry.rs" }, nature: additive }
  - { spec: "022-index-sharding", unit: { kind: file, path: "crates/spec-spine-types/tests/dtos.rs" }, nature: additive }
  - { spec: "074-a-governed-read-names-its-version", unit: { kind: file, path: "crates/spec-spine-core/tests/read.rs" }, nature: additive }
  - { spec: "106-obligations-are-declared-constraints", unit: { kind: file, path: "crates/spec-spine-core/tests/obligations.rs" }, nature: additive }
  - { spec: "088-the-template-teaches-the-whole-grammar", unit: { kind: file, path: "standards/spec/templates/spec-template.md" }, nature: additive }
  - { spec: "057-the-docs-name-what-adopters-derived", unit: { kind: file, path: "docs/schema-versioning.md" }, nature: additive }
  - { spec: "057-the-docs-name-what-adopters-derived", unit: { kind: file, path: "docs/api.md" }, nature: additive }
  - { spec: "057-the-docs-name-what-adopters-derived", unit: { kind: section, file: "docs/cli-reference.md", anchor: "cli-reference" }, nature: additive }
  - { spec: "103-a-verifier-fixture-is-a-published-artifact", unit: { kind: directory, path: "crates/spec-spine-core/fixtures/verifier/" }, nature: corrective }
references:
  - { unit: { kind: file, path: "docs/design/09-disposition-2026-09-21.md" }, role: "roadmap" }
  - { unit: { kind: file, path: "crates/spec-spine-core/src/symbols.rs" }, role: "test identity feasibility measurement" }
obligations:
  - id: "R-1"
    kind: requirement
    text: "Every trace relation has a stable qualified identity, one qualified obligation source, one typed target, a closed relation kind, and canonical order."
    anchor: "3-1-declaration-shape-and-identity"
  - id: "R-2"
    kind: requirement
    text: "Resolution reports resolved, unresolved, ambiguous, unsupported, unknown, or withdrawn explicitly and never substitutes ownership or inference for behavioral proof."
    anchor: "3-6-resolution-states-are-exhaustive"
  - id: "R-3"
    kind: requirement
    text: "Traceability is optional and incremental, so a corpus without declarations remains valid and no historical declaration is inferred or required."
    anchor: "3-9-incremental-adoption-and-compatibility"
  - id: "R-4"
    kind: requirement
    text: "Declaration validity and target resolution are distinct from execution evidence, implementation acceptance, release, adoption, and observation."
    anchor: "3-8-declaration-is-not-evidence-or-acceptance"
  - id: "I-1"
    kind: invariant
    text: "Traceability never executes a target, infers a semantic relation, guesses a dependency graph, changes authority, or gates implementation acceptance."
    anchor: "3-10-authority-and-security-boundary"
  - id: "V-1"
    kind: verification
    text: "The implementation is checked for every relation kind, target kind, resolution state, schema rule, canonical ordering rule, withdrawal rule, and negative case in this spec."
    anchor: "verification"
    inputs:
      - "crates/spec-spine-core/tests/traceability.rs"
      - "crates/spec-spine-cli/tests/traceability.rs"
intent:
  goal: "make authored obligation-to-artifact relations deterministic and inspectable without confusing declarations with proof"
  non_goals:
    - "retrofitting trace declarations into the historical corpus"
    - "semantic inference, coverage inference, or guessed dependency graphs"
    - "test execution, evidence admission, acceptance, release, adoption, or observation"
    - "changing ownership, lifecycle, waiver, or gate authority"
---

# 169: Declare qualified obligation traceability

## 1. Purpose

Spec 106 gives requirements, invariants, and verification obligations stable
qualified identities. The registry can resolve those identities, but it cannot
record which implementation unit, deterministic test, invariant, documentation
source, or interface an author intends to connect to one obligation. Consumers
therefore either reread prose or invent relations from ownership and file
proximity. Both approaches lose the author's claim and invite false proof.

This spec adds one optional declaration and one deterministic read. A trace
relation says only what an author declared. Resolution says whether its target
can be bound under a closed, structural contract. Neither statement says that
code ran, evidence was accepted, an implementation passed, a release exists,
an adopter consumed it, or production behavior was observed.

## 2. Territory

- `crates/spec-spine-types/src/traceability.rs` defines declaration, target,
  resolution, request, and response types. Its JSON schema defines the read
  document independently of the registry schema.
- `crates/spec-spine-core/src/traceability.rs` validates and resolves declared
  relations from compiled registry and index inputs. It does not inspect
  semantics or execute targets.
- `crates/spec-spine-core/tests/traceability.rs`, its fixture directory, and
  `crates/spec-spine-cli/tests/traceability.rs` establish shape, ordering,
  compatibility, and negative cases.
- `registry traceability` is added to the existing registry command and JSON
  facade. It is a read-only projection.
- `docs/traceability.md`, the template, API reference, CLI reference, and
  schema-versioning guide explain the authored contract and its limits.

Existing obligation, interface-reference, ownership, selected-content, index,
and lifecycle contracts remain owned by their current specs. This spec consumes
their public identities and adds no authority to them.

## 3. Behavior

### 3.1 Declaration shape and identity

Frontmatter MAY contain `traceability`, an array of entries with exactly these
members:

```yaml
traceability:
  - id: "R-1-test"
    obligation: "169-declared-obligation-traceability#R-1"
    relation: tested-by
    target:
      kind: test
      selector: "spec_spine_core::traceability::orders_relations"
```

`id` follows the obligation id grammar from spec 106 and is unique within the
declaring spec. Its stable qualified identity is `<declaring-spec>#trace:<id>`.
An id is never reassigned to a different source, relation, or target.

`obligation` MUST be a qualified `<full-spec-id>#<obligation-id>` reference to
a non-withdrawn obligation that resolves in this corpus. An unqualified,
dangling, ambiguous, self-missing, or withdrawn source is invalid. A relation
MAY cite an obligation in the declaring spec.

`relation` is one of `implemented-by`, `tested-by`, `enforced-by`,
`documented-by`, `produced-by`, or `consumed-by`. `target` is one tagged target
from §3.2. Unknown members, an empty array entry, a duplicate id, or a relation
whose target kind is not allowed by §3.2 is invalid at compile.

Each entry relates exactly one source to exactly one target. One obligation MAY
have zero or more relations. One target MAY be cited by zero or more
obligations. Absence never means unimplemented, untested, undocumented, or
complete.

### 3.2 Typed targets and allowed relations

The target kinds and allowed relations are closed:

| Target kind | Required identity | Allowed relation |
|---|---|---|
| `unit` | `spec` plus one exact authority unit object | `implemented-by` |
| `test` | one exact selected-content test selector | `tested-by` |
| `invariant` | one qualified obligation whose kind is `invariant` | `enforced-by` |
| `documentation` | one selected-content selector for a spec, spec section, obligation, owned unit, file, directory member, symbol, or module | `documented-by` |
| `interface` | `role`, plus either a local qualified unit or one named `interface_references` entry | `produced-by` or `consumed-by` |

An authority unit reuses the exact `kind` and identity members accepted by the
frontmatter edge grammar. Its `spec` is the full id of the spec expected to own
that unit. Resolution checks the claim and the index separately as §3.5 says.

An interface target has `role: producer` for `produced-by` and `role: consumer`
for `consumed-by`. A local interface names `spec` and `unit`. A cross-corpus
interface names the declaring spec and the exact `(corpus, spec)` pair of one
spec 110 `interface_references` entry. It copies no digest and performs no
fetch; the cited declaration already carries the pin. Exactly one local or
cross-corpus form is present.

### 3.3 Test identity is deliberately narrow

The structural index, as measured at 0.28.0 and unchanged through 0.29.0,
scans top-level Rust items under crate `src/`, does not scan integration-test roots as tests, and does not preserve
test attributes as identity. A generic function symbol therefore cannot prove
that a function is a test.

Spec 155 as merged reserves its `test` selector kind and reports it
`unsupported-selector` until a resolver binds nested and integration-test
functions together with their test attributes (155 §3.3). This spec reuses
that selector unchanged, so every `test` target resolves `unsupported` today,
never `resolved` and never `unknown`. When a measured resolver spec (RS-01)
adds a deterministic test kind to 155's matrix, targets of that kind begin to
resolve with no change here. Macro-generated, parameterized, doctest,
string-named, and framework-specific identities outside that matrix stay
`unsupported`.

### 3.4 Canonical target identity and ordering

Each target normalizes to a canonical identity:

- `unit:<spec>:<canonical-unit-json>`;
- `test:<canonical-selected-content-selector-json>`;
- `invariant:<full-spec-id>#<obligation-id>`;
- `documentation:<canonical-selected-content-selector-json>`;
- `interface:producer|consumer:local:<spec>:<canonical-unit-json>`; or
- `interface:producer|consumer:external:<declaring-spec>:<corpus>:<spec>`.

Canonical JSON uses the repository's existing sorted-object, no-insignificant-
whitespace encoding. A spec MUST NOT declare the same normalized
`(obligation, relation, target)` tuple twice, even under different ids.

Registry storage and every traceability read sort first by declaring full spec
id, then by relation id. Results for one relation sort by resolution state and
then canonical target identity. Input authoring order never changes output
bytes.

### 3.5 Resolution is structural and qualified

Resolution consumes a compiled registry and, for unit-bearing targets, a fresh
compiled index. It performs only these structural checks:

- the source obligation resolves and is not withdrawn;
- a unit is declared by the named spec and resolves to the same indexed unit;
- an invariant resolves to exactly one non-withdrawn obligation of kind
  `invariant`;
- a documentation or test selector is inside spec 155's supported matrix and
  resolves against the named snapshot inputs;
- a local interface unit is declared by the named spec and resolves in the
  index; and
- a cross-corpus interface target names exactly one local spec 110 declaration.

Ownership proves only that a spec claims a unit. It never proves that the unit
implements, enforces, tests, documents, produces, or consumes any behavior.
Likewise, matching names, paths, symbols, imports, calls, dependencies, prose,
or content never create a relation. Only the authored entry does.

Planned units are valid targets. They resolve `unresolved` until present and
indexed, without producing a compile error or an unresolved-unit warning
beyond the existing planned-unit rules.

### 3.6 Resolution states are exhaustive

Every declared relation produces exactly one of these states:

| State | Meaning |
|---|---|
| `resolved` | The target form is supported and exactly one structural target binds. |
| `unresolved` | The target form is supported, but no target binds in the supplied registry, index, or snapshot. |
| `ambiguous` | The supported target form binds more than one structural target. |
| `unsupported` | The authored target form is valid but outside the closed resolver support matrix. |
| `unknown` | Resolution needs an unavailable input, an unimplemented declared dependency, or a schema version this resolver cannot interpret. |
| `withdrawn` | The relation itself, or a target obligation or referenced declaration retained as a tombstone, is withdrawn. |

The states are serialized as these lowercase strings. A resolver MUST NOT map
one state to another for convenience, fall back to a nearby target, or omit a
relation because resolution did not succeed.

### 3.7 Withdrawal preserves identity

A relation is withdrawn in place with `withdrawn: true`. Its id, obligation,
relation, and target remain present and MUST NOT change. A withdrawn id can
never be reused. A withdrawn entry resolves `withdrawn` without attempting to
bind its target.

Deleting an entry that has never left an unratified draft is ordinary draft
editing. Once a spec containing the entry is approved, withdrawal is the only
compatible retirement. The registry retains the tombstone and canonical
identity.

### 3.8 Declaration is not evidence or acceptance

Compilation answers whether a declaration is shaped and qualified correctly.
Resolution answers whether its structural target binds under §3.5. Neither
answer executes a test or command, reads a CI result, admits evidence, judges
behavior, or establishes acceptance.

These stages remain distinct:

1. declaration records the author's relation;
2. resolution binds the declared structural identity;
3. execution, if another system performs it, produces an event;
4. evidence records what happened and under which inputs;
5. acceptance is a governed judgment over requirements and evidence;
6. release and publication make an artifact available;
7. adoption records a consumer's use; and
8. observation records behavior in an environment.

No later state is inferred from an earlier one. In particular, `resolved` does
not mean executed, passed, accepted, released, adopted, observed, or qualified.
Implementation acceptance for this spec is the separate test suite named in
§5, not the validity of a draft declaration.

### 3.9 Incremental adoption and compatibility

`traceability` is optional. An absent key compiles as an empty list and retains
the pre-feature `shardHash`; no historical spec needs a declaration. Tools
MUST NOT warn merely because an obligation has no relations or because a
relation set is not alleged to be complete.

Adding the optional registry member advances the registry schema MINOR. The
member is omitted when empty. The traceability read begins at schema `1.0.0`
on the read axis. A reader accepts compatible additive MINOR members according
to the existing schema policy, ignores no unknown relation or target kind, and
refuses an unsupported MAJOR. A producer never downcasts an unknown state to
`unresolved`.

The public read is:

```text
spec-spine registry traceability
    [--declared-by <spec>]
    [--obligation <full-spec-id>#<obligation-id>]
    [--state <state>]
    [--json]
```

Filters intersect. Human and JSON forms carry every relation, canonical
identity, normalized source and target, state, and state-specific detail.
Human output sorts identically to JSON. A valid read exits zero even when some
relations are unresolved, ambiguous, unsupported, unknown, or withdrawn.
Malformed requests and unreadable governed inputs retain the existing exit
contract.

### 3.10 Authority and security boundary

Traceability is repository-scoped. It performs no network access, provider
call, remote fetch, source execution, test execution, file edit, command
execution, evidence admission, or long-lived service operation.

The declaration grants no ownership or authority and cannot ratify a spec,
approve a waiver, weaken a gate, accept an implementation, or change a
producer or consumer contract. Paths and selectors are treated as untrusted
input and remain subject to existing containment and bounded-read rules.

No semantic inference is permitted. The resolver never guesses relations from
AST dependencies, call graphs, imports, package graphs, naming similarity,
proximity, ownership, or prose.

## 4. Negative cases

Compilation MUST refuse:

- an invalid or duplicate relation id;
- an unqualified, dangling, ambiguous, or withdrawn source obligation;
- an unknown relation or target kind;
- a relation paired with a disallowed target kind;
- a target missing a required identity member or carrying an unknown member;
- a duplicate normalized `(obligation, relation, target)` tuple;
- an interface whose role disagrees with its relation;
- an interface containing both or neither of its local and cross-corpus forms;
- a cross-corpus interface that does not name exactly one declaration on the
  declaring spec; and
- a withdrawn relation whose identity-bearing fields were changed after
  approval.

A well-formed declaration MUST remain valid while resolving `unresolved`,
`ambiguous`, `unsupported`, `unknown`, or `withdrawn`. Those are data, not
compile failures. In particular, a planned unit, unavailable snapshot input,
or currently unsupported test identity never becomes a fabricated success.

## 5. Acceptance criteria

- The optional grammar, registry records, schemas, read document, CLI read,
  template, and public documentation implement §§3.1–3.10.
- The registry schema MINOR advances, empty declarations remain omitted, and a
  corpus without `traceability` retains unchanged `shardHash` values.
- Canonical ordering and identities are byte-stable across authoring order and
  map insertion order.
- Tests cover all six relation kinds, five target kinds, six resolution states,
  cardinality, withdrawal, planned units, schema compatibility, and every
  negative case in §4.
- Test fixtures prove that ownership alone never produces `resolved` and that
  no undeclared relation appears.
- Unsupported test forms remain explicit until a later measured resolver spec
  expands the matrix.
- The governance floor and Rust checks pass on the implementation head.

## 6. Out of scope

- Retrofitting declarations into any spec approved before this one.
- Claiming completeness or computing coverage from missing declarations.
- Semantic code analysis, call graphs, dependency inference, or relation
  discovery.
- Executing tests, gathering or admitting evidence, or deciding acceptance.
- Ratification, release, publication, adoption, observation, or qualification.
- Expanding test resolution beyond the measured spec 155 support matrix.
- Cross-repository fetching or composing repository snapshots.
- Write APIs, edit plans, MCP transport, or documentation generation.

## 7. Decisions

### 2026-09-26: Keep the declaration optional and source it from obligations

Relations originate at qualified obligations because those are the stable
behavioral claims. The feature is optional and makes no historical retrofit or
completeness assertion.

### 2026-09-26: Separate validity, resolution, and lifecycle evidence

Malformed declarations fail compilation. Valid declarations always retain an
explicit resolution state. Execution, evidence, acceptance, release, adoption,
and observation remain outside both operations.

### 2026-09-26: Reuse selected-content identity instead of inventing test ids

The current index does not identify tests reliably. Test and documentation
targets reuse spec 155's bounded selectors, so every test form outside 155's
matrix (today, all of them) is reported as unsupported.

### 2026-09-26: Treat ownership as a structural check only

A named spec and authority unit keep unit targets qualified, but an ownership
match is not behavioral proof. Only an authored relation can connect the unit
to an obligation.

### 2026-09-26: Preserve withdrawn relation identity as a tombstone

Withdrawal retains the complete identity-bearing entry so consumers can
distinguish retirement from absence and an id can never be silently reused.

### 2026-09-28: Refiled under the first collision-free identity

Originally drafted as 158, then refiled as 161 after 156 through 158 were taken
on `main`. That branch did not merge, and later draft work reserved identities
through 168. This filing uses 169, preserves the reviewed behavior, keeps §3.3
aligned with spec 155 as merged, and retains a Verification block that fails
before the build.

### 2026-10-10: The registry member lands with the release that carries it

§3.9 advances the registry schema MINOR, which restamps `specVersion` in every
committed registry shard even though no spec declares `traceability`. Since
spec 156 the managed `governance` job judges the committed shards with the
exactly pinned released engine (0.29.0 at filing), while the
`self-governance` job judges them with the in-tree build. Measured on
`origin/main` at `3910c483`: restamping one shard to `1.10.0` makes the pinned
0.29.0 `check` report that shard `modified`. No committed shard tree can
satisfy both engines, so the build cannot merge before a release containing
it exists. As spec 193 §3.2 records for the pin itself, the release is tagged
at the build change's head and moves the exact pin in the same change. Cutting
that release is owner authority; the build is complete when its tests and
acceptance pass, and it waits for that release to merge.

## Verification

Each named test target fails before the build, because it does not exist while
`implementation: pending`. `cargo test --workspace` is the regression floor and
passes either way.

```verify:cli
cargo test -p spec-spine-core --test traceability --locked
cargo test -p spec-spine-cli --test traceability --locked
cargo test --workspace --locked
```
