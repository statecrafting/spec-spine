---
id: "103-an-amended-acceptance-is-the-one-that-runs"
title: "An amended acceptance is the one that runs"
status: approved
kind: "core"
created: "2026-09-16"
summary: >
  Spec 093's `## Verification` line 13 asserts `d["next"]["id"]` on `registry
  plan --next`, which is a claim about the corpus rather than about the code: it
  holds only while something is ready to build. The corpus finished, `next`
  became `null`, and `spec-spine verify 093` has been red ever since, failing an
  assertion that contradicts 093 §3.3's own requirement that an absent answer be
  a present `null` member. The line is wrong and 093 is approved, so correcting
  it is an amendment. Spec 040 forbids editing the amended file and `verify`
  executes that file, so until now an amendment could not reach an acceptance
  block at all. This spec gives it a route: a spec that `amends` another may
  declare that its own `## Verification` block replaces the amended spec's, and
  `verify` runs the replacement and says whose it ran.
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "040-amendment-authoring"
  - "049-verify-declared-acceptance"
  - "093-a-governed-read-names-its-version"
amends: ["093-a-governed-read-names-its-version"]
# 3.5: this spec's `## Verification` block IS 093's acceptance from now on.
# 093's own file is not edited (spec 040 3.1).
amends_verification: ["093-a-governed-read-names-its-version"]
extends:
  # 3.1: the frontmatter key and the registry field it compiles to.
  - { spec: "000-spec-spine-bootstrap", unit: "crates/spec-spine-types/src/frontmatter.rs", nature: additive }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-types/src/registry.rs", nature: additive }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-types/src/version.rs", nature: additive }
  # 3.2, 3.4: `VerifyPlan` carries the spec whose block it holds (D-7).
  - { spec: "049-verify-declared-acceptance", unit: "crates/spec-spine-types/src/verify.rs", nature: additive }
  # 3.1: the version pin moves with the constant it pins.
  - { spec: "049-verify-declared-acceptance", unit: "crates/spec-spine-types/tests/dtos.rs", nature: additive }
  # 3.2, 3.3: resolution, and the validation that keeps it unambiguous.
  - { spec: "049-verify-declared-acceptance", unit: "crates/spec-spine-core/src/verify.rs", nature: additive }
  - { spec: "001-compile-registry", unit: "crates/spec-spine-core/src/compile.rs", nature: additive }
  # 3.4: the CLI says whose block it ran.
  - { spec: "049-verify-declared-acceptance", unit: "crates/spec-spine-cli/src/cmd_verify.rs", nature: additive }
  # 3.6: the acceptance.
  - { spec: "049-verify-declared-acceptance", unit: "crates/spec-spine-core/tests/verify.rs", nature: additive }
  - { spec: "010-registry-query-projection-flags", unit: "crates/spec-spine-cli/tests/cli.rs", nature: additive }
references:
  - { unit: { kind: file, path: "docs/design/05-remaining-waves-2026-09.md" }, role: context }
  - { unit: { kind: file, path: "docs/schema-versioning.md" }, role: context }
---

# 103: An amended acceptance is the one that runs

## 1. Purpose

### 1.1 An acceptance line that asserted a fact about the calendar

Spec 093's `## Verification` block, command 13:

```
target/release/spec-spine registry plan --next --json | python3 -c 'import json,sys; d=json.load(sys.stdin); k=list(d); assert k==sorted(k), k; assert d["schemaVersion"]; assert d["next"]["id"]'
```

Its comment reads "the pick is a named member, **and it is populated on this
corpus**". That second clause is the defect. It is true of a corpus with
something ready to build and false of a finished one, and which of those this
repository is changes every time a spec is ratified.

The corpus finished. Measured 2026-09-16 at `a6ef6e3`, with 100 specs all
approved:

```
$ spec-spine registry plan --next --json
{
  "next": null,
  "schemaVersion": "0.1.0"
}
$ spec-spine verify 093
...
[verify] exit 1
verify: 093-a-governed-read-names-its-version: FAILED at command 13 (exit 1)
```

The assertion does not merely fail; it asserts the opposite of what its own
spec requires. Spec 093 §3.3 fixes `next` as "present and `null`" for an absent
answer and gives the reason: "A consumer that must test whether `id` is present
to learn whether anything is ready is sniffing for members, which is precisely
what §1.2 records as the cost this spec removes." Line 13 sniffs for the
member. Spec 093 §3.8 even places the empty-ready-set case deliberately in
`tests/cli.rs` "because `verify` runs against this tree", and then wrote the
populated case into the block that runs against this tree.

### 1.2 An amendment with nowhere to land

Spec 093 is `approved`, so the line is not an edit anyone may make: spec 040
§3.1 is absolute that the amended spec's `spec.md` is not touched, and design
note 05 §9.4 records R-1 as "a one-line `amends` in a new spec; do not edit
093".

But `spec-spine verify <id>` (spec 049) reads the `## Verification` section out
of `specs/<id>/spec.md` and runs it. `verify.rs` contains no reference to
`amends` at all. So an amendment can say the line is wrong and change nothing:
the block in 093's file is still the block that runs, and `verify 093` stays red
for as long as the corpus is finished.

That is the gap. The amendment mechanism reaches prose, because prose is read by
people who can also read the ledger. It does not reach an executable block,
because the executor reads one file and knows nothing about the edge.

### 1.3 Why a mechanism rather than one exception

This is the first acceptance block in the corpus to rot, and it will not be the
last: a hundred specs carry executable blocks, and any block asserting a
property of the tree rather than of the code has the same failure ahead of it.
Handling this one by hand means deciding the same question again next time,
under the same pressure, with the honest answer ("edit the approved file, just
this once") available and wrong each time.

The route this spec builds is the one spec 040 §3.3 already argues for in the
prose case: **discovery is a typed read of the edge set, never a prose pointer
in the amended file**. A reader asks `registry relationships` whether a spec's
text has been amended; `verify` asks the same edge set whether its acceptance
has. D-6 records why the verb reads that edge set out of the corpus rather than
out of the committed ledger.

## 2. Territory

No new file. One frontmatter key, one registry field, one resolution step, one
validation, and the reporting line that makes the substitution visible:

| Unit | Edge | Why |
|---|---|---|
| `093` | `amends` | §3.5's replacement block. |
| `crates/spec-spine-types/src/frontmatter.rs` | `extends` 000, additive | The key. |
| `crates/spec-spine-types/src/registry.rs` | `extends` 001, additive | The compiled field. |
| `crates/spec-spine-types/src/version.rs` | `extends` 001, additive | `REGISTRY_SCHEMA_VERSION` MINOR. |
| `crates/spec-spine-types/src/verify.rs` | `extends` 049, additive | `VerifyPlan.acceptance_from`. |
| `crates/spec-spine-types/tests/dtos.rs` | `extends` 049, additive | The version pin. |
| `crates/spec-spine-core/src/verify.rs` | `extends` 049, additive | Resolution. |
| `crates/spec-spine-core/src/compile.rs` | `extends` 001, additive | The validation. |
| `crates/spec-spine-cli/src/cmd_verify.rs` | `extends` 049, additive | The reporting line. |
| `crates/spec-spine-core/tests/verify.rs`, `crates/spec-spine-cli/tests/cli.rs` | `extends`, additive | The acceptance. |

## 3. Behavior

### 3.1 `amends_verification` declares the replacement

A spec MAY declare, in frontmatter:

```yaml
amends: ["093-a-governed-read-names-its-version"]
amends_verification: ["093-a-governed-read-names-its-version"]
```

`amends_verification` is a list of spec ids. Every entry MUST also appear in
that spec's `amends` list; an entry that does not is a validation error
(`V-018`). Replacing what a spec accepts is an amendment of that spec, so the
edge that records the amendment must be there to be read.

It compiles to an `amendsVerification` array on the registry DTO, ordered as
written, empty when absent. This is an additive registry field: a MINOR bump of
`REGISTRY_SCHEMA_VERSION` to `1.3.0`, which restamps every shard's
`specVersion` and leaves every `shardHash` alone, the shape spec 028 already
established.

The embedded JSON Schema needs no edit and gets none. Its `specRecord` sets no
`additionalProperties`, which its own description says is deliberate so "an
additive minor can extend it", so the new member validates as it stands. The
conformance test still runs against the bumped version, which is what keeps the
DTO and the schema from drifting apart.

### 3.2 `verify` runs the replacement

`spec-spine verify <id>` MUST, before reading `<id>`'s own `## Verification`
section, ask the registry whether another spec declares `<id>` in
`amends_verification`. If one does, the plan is built from **that** spec's
`## Verification` section, and `<id>`'s own section is not run.

Resolution MUST follow the chain: if `B` replaces `A`'s acceptance and `C`
replaces `B`'s, then `verify A` runs `C`'s block. A spec whose acceptance has
been replaced has no acceptance of its own left to run, so a later amendment
attaches to the spec that currently holds it.

A `superseded` or `retired` amender MUST be skipped: its acceptance is no longer
the corpus's, and leaving a withdrawn spec holding another's acceptance would
make a retirement silently change what a third spec asserts.

### 3.3 An ambiguous replacement is refused, not guessed

`compile` MUST refuse, as a validation error, when two specs that are neither
`superseded` nor `retired` name the same id in `amends_verification`
(`V-019`), and when the chain of §3.2 contains a cycle (`V-020`).

Two specs each claiming to hold one spec's acceptance is a question about
authority, and picking the higher ordinal would be this tool inventing an answer
to it. The corpus refuses and a human decides which amendment stands.

### 3.4 The substitution is stated, never silent

`verify` MUST print, before the first command, which spec's block it is running
when that is not the spec named on the command line:

```
verify: 093-a-governed-read-names-its-version
  acceptance amended by 103-an-amended-acceptance-is-the-one-that-runs (spec 040)
```

A block that runs under another spec's name with no line saying so is the
laundering shape spec 040 §3.2 exists to refuse: the whole value of amending
rather than editing is that both documents remain readable, and that is worth
nothing if the reader is not told to look.

`registry show <id>` MUST carry `amendsVerification` like any other edge, so
the fact is answerable without running anything.

### 3.5 What spec 093's acceptance now is

This spec's `## Verification` block replaces spec 093's in full, and 093's file
is not edited. It is 093's block with command 13 corrected:

```
target/release/spec-spine registry plan --next --json | python3 -c 'import json,sys; d=json.load(sys.stdin); k=list(d); assert k==sorted(k), k; assert d["schemaVersion"]; assert "next" in d; assert d["next"] is None or d["next"]["id"]'
```

The corrected line asserts what 093 §3.3 actually requires and nothing about
the calendar: the document is an object with sorted keys, it is versioned,
`next` is **present** (which is the member-presence rule, and the half that
fails against pre-093 code), and when it is populated it carries an `id`. It
holds on a corpus with a ready set and on a finished one.

Replacing the block in full rather than the line is deliberate: a reader asking
what 093 accepts today should find one block that answers, not a base document
plus a patch to apply in their head.

### 3.6 The acceptance

`crates/spec-spine-core/tests/verify.rs` MUST assert that `plan` resolves
through a replacement, through a two-step chain, past a `superseded` amender,
and that a spec with no amender is unaffected.

`crates/spec-spine-cli/tests/cli.rs` MUST assert that `verify` prints the
attribution line of §3.4 and runs the amender's commands, that `compile`
refuses a fork with `V-019` and a cycle with `V-020`, that an
`amends_verification` entry missing from `amends` is `V-018`, and that
`registry show --json` carries `amendsVerification`.

The conformance test (`core/tests/conformance.rs`) MUST keep passing against
the bumped schema, which is what makes the DTO and the schema move together.

## 4. Out of scope

- **Editing spec 093.** §1.2. Its file keeps the block it was ratified with,
  which is the record spec 040 §3.2 protects.
- **Per-line or per-command replacement.** §3.5. A block is the unit.
- **Auditing the other ninety-nine blocks for the same defect.** This spec
  fixes the one that is red and builds the route for the next. Sweeping the
  corpus for acceptance lines that assert a property of the tree is worth doing
  and is its own work, with its own findings.
- **A lint that refuses a corpus-dependent assertion.** Tempting and not
  mechanically decidable: `assert d["next"]["id"]` is indistinguishable from a
  legitimate assertion about a value the code must produce. What makes it wrong
  is knowledge of the corpus, which a linter does not have.
- **`amends_sections`.** The key exists in the grammar and no spec uses it;
  spec 096 D-2 declined to establish the convention. This spec does not revive
  it, and `amends_verification` deliberately does not reuse it: an anchor
  narrowing prose and a replacement of an executable block are different acts,
  and one key meaning both would make neither readable.

## 5. Resolved decisions

D-1 (2026-09-16, why a new frontmatter key rather than a marker in the
Verification block). The declaration is a fact about authority, and every other
such fact in this corpus is frontmatter that compiles to the registry, so
`registry show` and `registry relationships` can answer it without parsing
markdown. A marker inside the block would be readable only by running the file.

D-2 (2026-09-16, why `amends_verification` must be a subset of `amends`).
Replacing what a spec accepts changes what that spec requires of the tree, which
is an amendment by any reading. Allowing the two to diverge would let a spec
take over another's acceptance while the ledger's `amended_by (incoming)`
projection said nothing, which is precisely the discovery path spec 040 §3.3
makes authoritative.

D-3 (2026-09-16, why a fork is refused rather than resolved by ordinal). Picking
the higher ordinal is a rule that always produces an answer and never produces a
reason. Two specs claiming one acceptance is a disagreement about authority, and
constitution V makes that a question for a person.

D-4 (2026-09-16, why the whole block rather than a diff). §3.5. A patch that
must be applied mentally to a document in another file is a worse record than a
complete restatement beside a preserved original, and the original is preserved
either way.

D-6 (2026-09-16, why resolution reads the corpus rather than the committed
registry). `verify` today is a function of `<specs_dir>/<id>/spec.md` and
nothing else, and it must stay runnable on a tree whose `.derived/` is stale or
absent: a verb that refuses until the ledger is regenerated cannot be the verb
an operator reaches for while fixing the corpus. Resolution therefore reads the
`amends_verification` and `status` frontmatter of every `specs/*/spec.md`,
which is the directory `plan` already lists. The compiled field of §3.1 exists
for `registry show` and for consumers, not because `verify` needs it.

D-7 (2026-09-16, why `VerifyPlan` carries the source spec id). §3.4's line has
to come from somewhere, and computing it twice, once for the plan and once for
the message, is how the two drift. The plan names the spec whose block it holds;
the CLI prints the line when that name differs from the one asked for.

D-5 (2026-09-16, why a superseded amender is skipped rather than refused).
Superseding a spec is a normal lifecycle act and must not become a validation
failure in a third spec. Skipping returns the acceptance to whatever held it
before, which is the state the corpus was in before the superseded spec existed.

## Verification

Each line is one command (spec 049 §3.2).

**This block is spec 093's acceptance as well as this spec's** (§3.5). Spec 093's
own file is not edited, so a reader comparing the two sees exactly what changed:
one assertion, on `registry plan --next`. Everything else is 093's block
verbatim, and running it here is what makes `verify 093` mean something again.

The corrected line is the fail-first evidence for §3.5 in both directions. It
fails against pre-093 code, which emits a bare `null` carrying no `next` member
at all, and it fails against the pre-103 corpus, where `verify 093` runs 093's
own block and dies on `d["next"]["id"]`.

The `spec103_` lines are the fail-first evidence for the mechanism. Their filter
selects tests absent at the parent commit, where `cargo test` runs zero of them
and reports `ok`, so the count is asserted before the suite is trusted.

§3.4's attribution line is asserted by `spec103_verify_states_the_substitution`
in `tests/cli.rs`, on a scratch corpus, and **not** by a line in this block. A
block that is 093's acceptance cannot run `verify 093`: spec 049 §3.7's `R-001`
guard refuses the re-entry, and correctly, since the recursion is unbounded.
That is a property of every acceptance a spec holds for another, so it is worth
stating rather than discovering twice.

The `cargo test` lines inherited from 093 are **not** fail-first, and were not
when 093 shipped: the assertions they run do not exist at the parent commit, so
those suites pass vacuously.

```verify:cli
# --- spec 103's own mechanism ---
sh -c 'n=$(cargo test -p spec-spine-core --test verify --locked spec103_ 2>&1 | grep -c "^test spec103_"); test "$n" -ge 4 || { echo "expected at least 4 spec103_ tests, ran $n"; exit 1; }'
cargo test -p spec-spine-core --test verify --locked spec103_
cargo test -p spec-spine-cli --test cli --locked spec103_
# 3.1: the compiled field is a governed read.
target/release/spec-spine registry show 103 --json | python3 -c 'import json,sys; d=json.load(sys.stdin); assert d["amendsVerification"] == ["093-a-governed-read-names-its-version"], d'
# --- spec 093's acceptance, which this block now holds (3.5) ---
# 3.4: the axis exists and starts where the note says.
grep -qF 'READ_SCHEMA_VERSION' crates/spec-spine-types/src/version.rs
# 3.2, 3.5: every object read is sorted and versioned.
target/release/spec-spine registry plan --json | python3 -c 'import json,sys; d=json.load(sys.stdin); k=list(d); assert k==sorted(k), k; assert d["schemaVersion"]'
target/release/spec-spine registry show 093 --json | python3 -c 'import json,sys; d=json.load(sys.stdin); k=list(d); assert k==sorted(k), k; assert d["schemaVersion"]'
target/release/spec-spine registry status-report --json | python3 -c 'import json,sys; d=json.load(sys.stdin); k=list(d); assert k==sorted(k), k; assert d["schemaVersion"]'
target/release/spec-spine registry status-report --nonzero-only --json | python3 -c 'import json,sys; d=json.load(sys.stdin); k=list(d); assert k==sorted(k), k; assert d["schemaVersion"]'
target/release/spec-spine registry relationships 093 --json | python3 -c 'import json,sys; d=json.load(sys.stdin); k=list(d); assert k==sorted(k), k; assert d["schemaVersion"]'
target/release/spec-spine index owner Cargo.toml --json | python3 -c 'import json,sys; d=json.load(sys.stdin); k=list(d); assert k==sorted(k), k; assert d["schemaVersion"]'
target/release/spec-spine index coverage --json | python3 -c 'import json,sys; d=json.load(sys.stdin); k=list(d); assert k==sorted(k), k; assert d["schemaVersion"]'
target/release/spec-spine index orphans --json | python3 -c 'import json,sys; d=json.load(sys.stdin); k=list(d); assert k==sorted(k), k; assert d["schemaVersion"]'
# 3.3, 3.6: the three array reads carry their items under a versioned object,
# and the ids-only projection still projects ids.
target/release/spec-spine registry list --json | python3 -c 'import json,sys; d=json.load(sys.stdin); assert isinstance(d["items"], list); assert d["schemaVersion"]'
target/release/spec-spine registry list --ids-only --json | python3 -c 'import json,sys; d=json.load(sys.stdin); assert all(isinstance(x, str) for x in d["items"]); assert d["schemaVersion"]'
target/release/spec-spine index diagnostics --json | python3 -c 'import json,sys; d=json.load(sys.stdin); assert isinstance(d["items"], list); assert d["schemaVersion"]'
# 3.3, 3.6: the pick is a named member, present whether or not it is
# populated. Spec 103 3.5 corrected this line: it asserted the corpus had
# something ready, which 093 3.3 never required and a finished corpus denies.
target/release/spec-spine registry plan --next --json | python3 -c 'import json,sys; d=json.load(sys.stdin); k=list(d); assert k==sorted(k), k; assert d["schemaVersion"]; assert "next" in d; assert d["next"] is None or d["next"]["id"]'
# 3.7: `config show` is sorted and keeps 054's version member, with no second one.
target/release/spec-spine config show --json | python3 -c 'import json,sys; d=json.load(sys.stdin); k=list(d); assert k==sorted(k), k; assert "config_version" in d; assert "schemaVersion" not in d'
# 3.8: the emitter's own properties, and the per-document assertions.
cargo test -p spec-spine-core --test read --locked
cargo test -p spec-spine-cli --test cli --locked
# 3.4: the axis is documented where the others are.
grep -qF 'READ_SCHEMA_VERSION' docs/schema-versioning.md
```
