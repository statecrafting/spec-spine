---
id: "109-impact-and-conflict-are-declared"
title: "Impact and conflict are declared"
status: draft
kind: "governance"
created: "2026-09-21"
implementation: complete
owner: "The spec-spine Authors"
depends_on:
  - "106-obligations-are-declared-constraints"
summary: >
  A spec can say which specs it extends and amends. It cannot say which
  requirements it is expected to affect, or which requirements it knowingly
  conflicts with. Both are declared in frontmatter against spec 106's
  qualified obligation ids, validated at compile, carried in the registry, and
  answered by one read that inverts the declarations so the affected side can
  see them. Nothing gates on either, and nothing computes an impact.
establishes:
  - { kind: file, path: "crates/spec-spine-types/src/impact.rs" }
  - { kind: file, path: "crates/spec-spine-core/src/impact.rs" }
  - { kind: file, path: "crates/spec-spine-core/tests/impacts.rs" }
  - { kind: file, path: "crates/spec-spine-cli/tests/impacts.rs" }
extends:
  # 3.1: the frontmatter grammar (`impacts`, `conflicts`).
  - { spec: "000-spec-spine-bootstrap", unit: { kind: file, path: "crates/spec-spine-types/src/frontmatter.rs" }, nature: additive }
  # 3.7: the registry record and the schema MINOR.
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-types/src/registry.rs" }, nature: additive }
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-types/src/lib.rs" }, nature: additive }
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-types/src/version.rs" }, nature: additive }
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-types/schemas/registry.schema.json" }, nature: additive }
  - { spec: "022-index-sharding", unit: { kind: file, path: "crates/spec-spine-types/schemas/registry-spec-shard.schema.json" }, nature: additive }
  # 3.4: validation at compile.
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-core/src/compile.rs" }, nature: additive }
  # 3.3: the `unresolved` lint warning, L-014.
  - { spec: "003-conformance-lint", unit: { kind: file, path: "crates/spec-spine-core/src/lint.rs" }, nature: additive }
  # 3.6: the facade op and its request members.
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-core/src/lib.rs" }, nature: additive }
  # 3.6: the CLI verb.
  - { spec: "002-registry-query", unit: { kind: file, path: "crates/spec-spine-cli/src/cmd_registry.rs" }, nature: additive }
  # 3.7: the pin every registry MINOR moves (026, 063, 082 and 106 did the same).
  - { spec: "022-index-sharding", unit: { kind: file, path: "crates/spec-spine-types/tests/dtos.rs" }, nature: additive }
  # D-2: the read axis moves for the new document, and its pin moves with it.
  - { spec: "074-a-governed-read-names-its-version", unit: { kind: file, path: "crates/spec-spine-core/tests/read.rs" }, nature: additive }
  # D-3: one of spec 106's own tests pinned the head registry version.
  - { spec: "106-obligations-are-declared-constraints", unit: { kind: file, path: "crates/spec-spine-core/tests/obligations.rs" }, nature: additive }
  # 3.1: the template documents the two keys.
  - { spec: "088-the-template-teaches-the-whole-grammar", unit: { kind: file, path: "standards/spec/templates/spec-template.md" }, nature: additive }
  # 3.6, 3.7: the documentation an author and a consumer read.
  - { spec: "057-the-docs-name-what-adopters-derived", unit: { kind: file, path: "docs/schema-versioning.md" }, nature: additive }
  - { spec: "057-the-docs-name-what-adopters-derived", unit: { kind: file, path: "docs/api.md" }, nature: additive }
  # D-4: this spec changes what the producer emits for the fixture corpus, so
  # it regenerates spec 103's set, on 103 §3.9's authority path.
  - { spec: "103-a-verifier-fixture-is-a-published-artifact", unit: { kind: directory, path: "crates/spec-spine-core/fixtures/verifier/" }, nature: corrective }
references:
  - unit: { kind: file, path: "docs/design/04-authority-evidence-extension.md" }
    role: "context"
  - unit: { kind: file, path: "docs/design/09-disposition-2026-09-21.md" }
    role: "context"
obligations:
  - id: "R-1"
    kind: requirement
    text: "An impact names an obligation, qualified, and it must resolve."
    anchor: "3-2-an-impact-names-an-obligation-qualified-and-resolving"
  - id: "R-2"
    kind: requirement
    text: "A conflict is declared, with a reason and a resolution, never resolved by the declaration itself."
    anchor: "3-3-a-conflict-is-declared-not-resolved"
  - id: "R-3"
    kind: requirement
    text: "An unresolved conflict is reported by lint at warning tier, once per entry, and never refuses without --fail-on-warn."
    anchor: "3-3-a-conflict-is-declared-not-resolved"
  - id: "R-4"
    kind: requirement
    text: "A declaration naming a withdrawn obligation is valid, and the read reports the target as withdrawn."
    anchor: "3-4-validation-at-compile"
  - id: "I-1"
    kind: invariant
    text: "couple MUST NOT consult impacts or conflicts, and no verb refuses on their account except the validation errors of 3.4 and the --fail-on-warn refusal of an unresolved warning."
    anchor: "3-8-nothing-gates-on-either"
  - id: "V-1"
    kind: verification
    text: "Every declared rule is refused at compile with its code, the read inverts and sorts correctly, and a corpus without either key compiles to unchanged shardHash values."
    anchor: "verification"
    inputs:
      - "crates/spec-spine-core/tests/impacts.rs"
      - "crates/spec-spine-cli/tests/impacts.rs"
---

# 109: Impact and conflict are declared

## 1. Purpose

Note 04 §5 P2 records A04, A05 and B03: a change should be able to state what
it is expected to affect, and where it knowingly disagrees with something
already written down.

The corpus has edges, and an edge is coarse. `extends` says "this spec adds
surface to that one"; it does not say which of that spec's requirements are
touched. `amends` says "this spec changes that one"; `amends_sections`
narrows it to a heading, which is closer, and still not to a requirement.

### 1.1 Why the obligations dependency is real

An impact set is a set of **things impacted**, and the thing worth naming is a
requirement, not a file and not a whole spec. Without spec 106's ids there is
nothing to point at, so the declaration would degrade to the edge vocabulary
that already exists. That is a semantic dependency and it is the only one
this spec declares.

It does not depend on ContextClosure, WorkScope, move mappings, overlays,
waiver lifecycle or bindings.

### 1.2 What it enables (D-1)

- **Not possible today:** a reader of spec 005's requirement `R-3` cannot
  find out that three later specs refine it and one knowingly contradicts it,
  short of reading every spec in the corpus.
- **With this:** each of those specs declares the relation against
  `005-...#R-3`; compile refuses a declaration that does not resolve; and one
  read answers "what is declared against this obligation", sorted and
  versioned, from the committed ledger.
- **Scenario (hypothetical, no consumer has asked):** an orchestrator issuing
  work against `106#R-5` reads the declared impacts and conflicts on it before
  issuing, and shows an `unresolved` conflict to the person approving the work.
- **Why it matters:** a corpus that knows it contradicts itself can now say so
  in a form a tool can find, instead of in prose nobody indexes.

## 2. Territory

- `crates/spec-spine-types/src/impact.rs`: the declared shapes.
- `crates/spec-spine-core/src/impact.rs`: the inverting read.
- Validation in `compile.rs`, the `unresolved` warning in `lint.rs`, the
  registry record and its schemas, the read-schema MINOR, the facade and the
  CLI verb, the template and the documentation, each declared as an `extends`
  edge on its owner by the build.

## 3. Behavior

### 3.1 Two declarations, in frontmatter

```yaml
impacts:
  - obligation: "005-coupling-gate#R-3"
    nature: refines
    note: "the owner set for a deleted path is resolved at a prior snapshot"
  - obligation: "071-a-change-is-classified-under-the-bases-rules#R-1"
    nature: supersedes
    successor: "R-4"
conflicts:
  - obligation: "071-a-change-is-classified-under-the-bases-rules#R-2"
    reason: "071 refuses a stale base index; this gate falls back to a report"
    resolution: deliberate
  - obligation: "094-one-gate-and-the-boundaries-it-holds#R-1"
    reason: "the hook runs formatting, which 094 left to the gate"
    resolution: pending
    settled_by: "113-a-waiver-has-a-declared-lifecycle"
```

Both keys are optional, both are frontmatter, and there is no second fenced
syntax (spec 106 §3.1). Unknown members of an entry are refused as malformed
frontmatter, as spec 106's obligation members are.

### 3.2 An impact names an obligation, qualified and resolving

Each `impacts` entry MUST name a **qualified** obligation reference
(`<spec>#<obligation-id>`, spec 106 §3.6) that resolves to an obligation the
named spec declares, and a `nature` from a closed set:

| `nature` | Meaning |
|---|---|
| `refines` | the obligation still holds, more precisely |
| `extends` | the obligation still holds, over more surface |
| `supersedes` | the obligation is replaced by one this spec declares |
| `informs` | the obligation is unaffected but a reader of it should know |

`supersedes` MUST name, in `successor`, the id of a non-withdrawn obligation
**this** spec declares. A supersession with no successor is a deletion wearing
a different word, and spec 106 leaves deletion to the tombstone. `successor`
on any other `nature` is refused: a member that means nothing is a member
somebody will misread. `note` is optional free text.

### 3.3 A conflict is declared, not resolved

Each `conflicts` entry MUST name a qualified, resolving obligation, a
non-empty `reason`, and a `resolution` from a closed set:

| `resolution` | Meaning |
|---|---|
| `deliberate` | the divergence is intended and the reason says why |
| `unresolved` | the contradiction is known and nobody has decided |
| `pending` | a named spec, `settled_by`, is expected to settle it |

`settled_by` MUST be present, and resolve to a spec in the corpus, exactly
when `resolution` is `pending`, and MUST be absent otherwise.

An `unresolved` conflict MUST be reported by `lint` at warning tier, once per
entry, and MUST NOT refuse without `--fail-on-warn`. A corpus is allowed to
know it contradicts itself; what it is not allowed to do is know silently.

### 3.4 Validation at compile

The following are compile errors, each naming the declaring spec and the
offending reference:

1. an unqualified reference (spec 106's rule: refused, never resolved
   locally);
2. a reference whose spec or obligation does not exist;
3. a reference to an obligation of the declaring spec itself: withdrawal and
   supersession inside one spec are what the tombstone is for;
4. the same obligation named twice in one spec's `impacts`, or twice in its
   `conflicts`;
5. the §3.2 `successor` rules and the §3.3 `reason` and `settled_by` rules.

A rule that must resolve a spec id against the corpus (2, 3, 4 once short ids
are normalized, and `settled_by`) is corpus-wide and recomputed on read, as
`depends_on`'s is (spec 022), so a sibling spec's change never stales this
spec's shard. The rest (1, the `successor` rules and `reason`) are a pure
function of one spec and are stored on its shard.

A declaration naming a **withdrawn** obligation is valid. The withdrawn
obligation keeps its identity (spec 106), and the read reports that the target
is withdrawn rather than hiding the declaration.

### 3.5 What these establish, and what they do not

They establish that a spec **declares** an impact or a conflict.

They do **not** establish:

- **that the impact set is complete.** Nothing computes what a change actually
  affects, and a consumer MUST NOT read "not in the impact set" as "not
  affected". An impact set is exactly the artifact somebody will want to use
  as a blast radius; it is a record of what authors said.
- **that a declared conflict is the only one.** Two specs can contradict each
  other with neither saying so.
- **that a `deliberate` conflict is correct.** It records that an author chose
  it. A reviewer still has to agree.

### 3.6 The read inverts the declarations

A declaration lives in the declaring spec, so the target spec says nothing.
The read answers from either side (open question 1 of the deferred draft,
answered here):

- `registry impacts [--target <ref>] [--declared-by <spec>] [--json]`, where
  `<ref>` is a spec id (every obligation it declares) or a qualified
  obligation reference, and both filters compose by intersection. With
  neither, every declaration in the corpus.
- `query_json` `op: "impacts"`, with the same two filters as `target` and
  `declaredBy`, over the registry text it is given.

The answer is a read document (spec 074) with `impacts` and `conflicts`, each
an array of entries carrying the declaring spec's full id (`declaredBy`), the
target as a full qualified reference (`<full-spec-id>#<obligation-id>`,
whatever short form the author wrote), the entry's own members, and
`targetWithdrawn`. Both arrays are sorted by target, then declaring spec, so
the same corpus always yields the same bytes. A `--target` without `#` is a
spec id; one with `#` that is not a qualified reference (an empty or blank
half, or a second `#`) is a usage error (exit 3); a target spec, target
obligation or declaring spec that does not exist is `NotFound` (exit 1). None
of these is answered as an empty set.

The read resolves against the committed registry. Like `show` and
`registry obligation`, it does not check freshness; it is an inspection, and
it states which ledger it read by the `schemaVersion` of the answer and the
`specVersion` of the shards it came from.

### 3.7 The registry and the versions

Each record carries `impacts` and `conflicts` as compiled (target normalized
to the full qualified form), omitted when empty. The registry schema takes a
MINOR, and both registry schemas describe the members. The new read document
takes a MINOR of the read axis. A corpus that declares neither key compiles
to unchanged `shardHash` values.

### 3.8 Nothing gates on either

`couple` MUST NOT consult impacts or conflicts, and no verb refuses on their
account except the validation errors of §3.4 and the `--fail-on-warn`
refusal of an `unresolved` warning.

An impact-aware gate is the obvious next thought and it is the wrong one while
§3.5 holds: refusing a change because its declared impact set looked
incomplete would be refusing on the strength of something nothing can verify.

## 4. Out of scope

- **Computing impact** from the dependency graph, the index, or a diff. A
  computed impact set is a different artifact and conflating it with a
  declared one is how a declaration acquires authority it was never given.
- **Resolving a conflict.** §3.3 records; a human or a later spec decides.
- **Any gate.** §3.8.
- **Expiring an `unresolved` conflict.** An expiry needs a clock, which this
  engine does not have; a caller-supplied "as of" is spec 113's shape, and can
  be reused when a consumer needs it.

## 5. Resolved decisions

**D-1 (2026-09-22, built under note 09 D-7, the contract corrected before the
build).** Filed on 2026-09-21 as a deferred contract on an unmerged branch;
carried here and corrected before any code:

- the draft's example set `resolution` to free text, contradicting its own
  closed set. The set is closed, and the pending form is a structured
  `resolution: pending` with `settled_by`, instead of a `pending: <id>` string
  that would need parsing;
- a `successor` on a non-superseding impact, a self-targeted declaration and a
  duplicate target are refused (§3.4), so each declared relation has one
  meaning;
- open question 1 (one-sided or two-sided) is answered by the read: the record
  stays one-sided, in the spec that makes the claim, and §3.6 inverts it, so
  the target side is visible without a record neither spec owns;
- open question 2 (expiry) is out of scope (§4);
- the read, the versions and the registry members are specified (§3.6, 3.7),
  where the draft named only the frontmatter.

**D-2 (2026-09-22, build: the V-code groupings and which are cross-spec).**
§3.4 names five rule groups without assigning codes. Seven were needed, split
by what each is a pure function of (spec 022's own rule for `CROSS_SPEC_CODES`:
a check that must resolve a spec id against the corpus is corpus-wide and
recomputed on read, never stored on a shard):

- stored on the declaring spec's shard: `V-025` (an unqualified reference,
  rule 1), `V-026` (the §3.2 successor rules, which only ever look at the
  declaring spec's own obligations), `V-027` (the §3.3 `reason` rule);
- recomputed on read, in `CROSS_SPEC_CODES`: `V-028` (dangling, rule 2),
  `V-029` (self-targeting, rule 3), `V-030` (duplicate target once short ids
  normalize, rule 4), `V-031` (the whole `settled_by` rule: §3.4 names it
  corpus-wide without splitting its presence half from its resolution half,
  so both live under one code and one recomputation).

Each guard was fired once and reverted to confirm it can fail:
`crates/spec-spine-core/tests/impacts.rs` exercises all seven directly, and a
temporary change to `check_obligation_reference`'s self-reference branch (make
it always `return` before pushing `V-029`) turned
`a_self_targeting_reference_is_v029_even_when_the_obligation_exists` red, then
was reverted.

**D-3 (2026-09-22, build: `settled_by` needs two spellings, not one).** The
frontmatter grammar's convention is snake_case (`superseded_by`,
`retirement_rationale`, `with_specs`); the registry DTOs are camelCase by
struct-level `#[serde(rename_all = "camelCase")]`. Every existing typed-edge
item with an underscored member (`with_specs`, `target_specs`, `derived_at`)
sidesteps the conflict by never being re-emitted under a `rename_all` struct:
none of those DTOs carries the item type inside a `camelCase` struct in a way
that renames the field, so they stay snake_case on the wire too.
`Impact`/`Conflict` are reused unchanged between `Frontmatter` (authored) and
`SpecRecord` (registry, camelCase), the way `Obligation` is (spec 106), so a
single field cannot serialize two ways by structural position alone.
`settled_by` is declared `#[serde(rename = "settledBy", alias =
"settled_by")]`: `rename` fixes the canonical/emitted spelling (`settledBy`,
satisfying the registry contract), `alias` accepts the authored spelling on
read, the way `Implementation`'s `#[serde(rename = "n-a", alias = "n/a")]`
already does for a different two-spelling problem. This type is parsed from
YAML and never re-serialized as YAML, so the two spellings never collide in
one direction. A unit test in `crates/spec-spine-types/src/impact.rs` pins
both directions.

**D-4 (2026-09-22, integration with spec 103: the verifier fixtures are
regenerated).** As spec 106 D-10 recorded for its own build: every registry
record now carries `impacts`/`conflicts` when declared (none here, since this
repository's own corpus declares neither on the fixture corpus's specs; the
attested `registryHash` still moved, because `sectionDigests`, `specVersion`
and the aggregate content hash are folded from every shard regardless of
whether this particular spec's own keys are populated). Regenerated with 103
§3.9's documented command; a second run rewrites nothing. Declared as a
`corrective` `extends` edge on spec 103, the authority path 103 §3.9 names.

**D-5 (2026-09-22, build: `compile_spec`, spec 049's single-spec check, is not
extended).** Spec 106 built `validate_obligations` without wiring it into
`compile_spec` (spec 049's per-draft check, which validates only
`V-001/005/006/007/008/009/010/011/012`); `validate_impacts_local` and
`detect_impact_cross_spec` are not wired in either, matching that precedent
rather than silently fixing a gap spec 109 does not own. `compile_spec` is not
in spec 109's territory (§2), so extending its coverage is a change to spec
049's own contract, not this one's.

**D-6 (2026-09-22, build: the read-document axis summary row had drifted, and
this table is the row it lives in).** `docs/schema-versioning.md`'s artifact
table read `0.2.0` for read documents since spec 102, though `READ_SCHEMA_VERSION`
moved to `0.3.0` (spec 106) and `0.4.0` (spec 107) without the summary row
following; both specs' own MINOR-history entries below the table were
accurate, only the summary line was not. This spec's own bump to `0.5.0`
touches that exact row, so the correction rides the same edit rather than
being deferred again: fixing the row this change already moves is not
amending 106 or 107 to make 109 pass, since neither spec's stated contract
changes.

**D-7 (2026-09-22, build: spec 106's own `## Verification` block pinned a
value this spec's additive MINOR legitimately moves).** `spec-spine verify
106` is one of this build's mandatory gates. Its block asserted `grep -q
'REGISTRY_SCHEMA_VERSION: &str = "1.4.0"'`, an exact-value pin of the same
constant this spec bumps to `1.5.0`, exactly the anti-pattern spec 085 ("a
version pin is not a contract") names and the reason three other specs
(074, 082, 102) already assert only the constant's *name* for their own read
or registry axis. 106's status is `draft` (unratified), so the constitution's
"never edit an approved spec" rule does not cover it, and the corrected line
is a mechanical fix to a known anti-pattern rather than a change to what 106
requires (the grep would have passed for `1.4.0` before this build and, if
edited back, is inert for any value the constant ever holds after `1.4.0`
too). Fixed, on review, by a floor rather than a bare name check: MAJOR 1 and
MINOR at least 4, which passes at `1.5.0` and fails at `1.3.0`, so the line
still catches a regression below 106's own MINOR. 106 records it as its D-13. No `amends`
edge: the change is to 106's `## Verification` fence, self-clearing the way
any edit to a spec's own `spec.md` does (`couple.rs`'s self-ownership rule for
`<specs_dir>/<id>/spec.md`), and `amends_verification` would require carrying
106's whole block into 109's, which would let one spec's acceptance answer for
two unrelated contracts.

## Verification

Written to fail against the tree this spec is filed on: none of the keys, the
verb or the op exists.

```verify:cli
cargo build --release --locked
# 3.1 - 3.4, 3.7: every rule, refused at compile, over disposable corpora.
cargo test -p spec-spine-core --test impacts --locked
# 3.3, 3.6: the lint warning and the read through the shipped CLI.
cargo test -p spec-spine-cli --test impacts --locked
# 3.7: the registry shards conform to the moved schema.
cargo test --workspace emitted_registry_conforms --locked
# 3.6: the verb answers over this corpus.
./target/release/spec-spine registry impacts --json | grep -q '"schemaVersion"'
sh -c './target/release/spec-spine registry impacts --target "#R-1"; test $? -eq 3'
sh -c './target/release/spec-spine registry impacts --target "999-none"; test $? -eq 1'
./target/release/spec-spine check --fail-on-unresolved --fail-on-warn
./target/release/spec-spine lint --fail-on-warn
```
