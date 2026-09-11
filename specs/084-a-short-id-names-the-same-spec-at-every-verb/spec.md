---
id: "084-a-short-id-names-the-same-spec-at-every-verb"
title: "A short id names the same spec at every verb"
status: draft
kind: "tooling"
created: "2026-09-11"
implementation: pending
owner: "The spec-spine Authors"
risk: high
depends_on:
  - "002-registry-query"
  - "004-codebase-index"
  - "016-short-id-resolution"
  - "042-per-spec-attestation"
  - "049-verify-declared-acceptance"
  - "056-compile-one-spec"
amends:
  # 016 2 records the compile-time resolver as "a local mirror of
  # `index.rs::resolve_id` rather than a shared call", pinned equal "by behavior
  # and a citing comment, not by linkage". 3.4 below makes both a shared call
  # into a module neither 001 nor 004 owns. 016 3.1, the policy, is unchanged.
  # See 5, D-1.
  - "016-short-id-resolution"
establishes:
  # Planned (spec 076) until the build writes them; the build drops the flag.
  # 3.4: the one policy, and the directory listing its two filesystem callers share.
  - { kind: file, path: "crates/spec-spine-core/src/spec_id.rs", planned: true }
  # 3.5: the policy, the library entry points, and the facade.
  - { kind: file, path: "crates/spec-spine-core/tests/spec_id.rs", planned: true }
  # 3.5: the six-argument matrix.
  - { kind: file, path: "crates/spec-spine-cli/tests/spec_id.rs", planned: true }
extends:
  # 3.4: the module is declared and re-exported.
  - { spec: "001-compile-registry", unit: "crates/spec-spine-core/src/lib.rs", nature: additive }
  # 3.4: `resolve_spec_dir` is removed; `resolve_spec_ref` delegates to the shared match.
  - { spec: "001-compile-registry", unit: "crates/spec-spine-core/src/compile.rs", nature: additive }
  # 3.4: `resolve_id` delegates to the shared match.
  - { spec: "004-codebase-index", unit: "crates/spec-spine-core/src/index.rs", nature: additive }
  # 3.4: the private `resolve_spec_id` is removed.
  - { spec: "049-verify-declared-acceptance", unit: "crates/spec-spine-core/src/verify.rs", nature: additive }
  # 3.1 and 3.3: `show` resolves; `relationships` compares the resolved id.
  - { spec: "002-registry-query", unit: "crates/spec-spine-core/src/query.rs", nature: additive }
  # 3.1: `attest_spec` resolves against the ids of its own compile.
  - { spec: "023-ledger-seal", unit: "crates/spec-spine-core/src/attest.rs", nature: additive }
  # 3.3: the attestation file is named by the resolved id.
  - { spec: "023-ledger-seal", unit: "crates/spec-spine-cli/src/cmd_attest.rs", nature: additive }
  # 3.2: `verify-attestation` resolves against the attestation files it reads.
  - { spec: "023-ledger-seal", unit: "crates/spec-spine-cli/src/verify_attestation.rs", nature: additive }
  # 3.1 and 3.5: help text on each id argument, and the census.
  - { spec: "001-compile-registry", unit: "crates/spec-spine-cli/src/main.rs", nature: additive }
  # 3.1: help text on `show` and `relationships`.
  - { spec: "002-registry-query", unit: "crates/spec-spine-cli/src/cmd_registry.rs", nature: additive }
references:
  - { unit: { kind: file, path: "specs/019-structured-partial-supersedes/spec.md" }, role: context }
  - { unit: { kind: file, path: "specs/083-an-attestation-covers-the-territory-it-claims/spec.md" }, role: context }
summary: >
  Six arguments take a spec id, and they disagree about what one is. `verify`
  and `compile --spec` accept the short form spec 016 defines; `registry show`,
  `registry relationships` and `attest --spec` refuse it at exit 1; and
  `verify-attestation --spec` fails at exit 3 reading a file named after the raw
  argument. Specs 049 3.2 and 056 3.1 each state the cross-verb rule as already
  true elsewhere, and nothing enforces it. The policy itself exists in four
  private copies, two strict ones for arguments and two lenient ones for
  frontmatter references, and the two strict ones test the exact case by joining
  the argument onto a path, so `verify ../specs/<id>` resolves and
  `compile --spec ./<id>` reports a false `V-001` on a valid spec. This spec puts
  the policy in one module that every argument and both reference resolvers
  call. Each verb resolves against the ids it already reads, and every value
  derived from the id after resolution uses the resolved id, which is what keeps
  `relationships` from computing incoming edges against the raw argument and
  `attest` from naming its output file after it. A census over the CLI's
  argument tree fails when a seventh id argument appears outside the matrix. It
  amends 016 2, whose mirror becomes a shared call. The implementation changes
  no committed shard: the lenient adapters return what the mirrors returned.
---

# 084: A short id names the same spec at every verb

## 1. Purpose

Measured on 0.18.0 at `6fca5ee`, one short id at each of the six arguments that
take a spec id:

```
$ spec-spine verify 070 --plan                 # exit 0
$ spec-spine compile --spec 070                # exit 0
$ spec-spine registry show 070                 # exit 1: not found: spec '070'
$ spec-spine registry relationships 070        # exit 1: not found: spec '070'
$ spec-spine attest --spec 070                 # exit 1: not found: spec '070'
$ spec-spine verify-attestation --spec 070 --recompute
spec-spine: io error: read attestation .../by-spec/070.json
(run `spec-spine attest --spec 070` first?): No such file or directory
                                               # exit 3
```

The last one's advice is to run a command that refuses the same argument.

Two approved specs describe this as solved. 049 3.2 says `spec-spine verify 049`
"resolves as `registry show 049` does", naming as its model a verb that refuses
the input. 056 3.1 says an unmatched id is `NotFound` "as everywhere else the
short form is accepted". Each sentence was true of the verb its own spec built
and assumed of the rest. Nothing checks either, which is how the claim survived
two ratifications. 083 4 met the `attest` case, recognized it as corpus-wide,
and handed it here.

### 1.1 Four copies of one policy

The rule is 016 3.1: an exact id first, otherwise the one id whose whole leading
dash-segment equals the reference. Core implements it four times:

| copy | serves | resolves against | on no match or several |
|---|---|---|---|
| `compile.rs::resolve_spec_ref` | `depends_on`, `superseded_by`, `supersedes` (016, 019) | the compiled id set | keeps the raw string |
| `index.rs::resolve_id` | `amends` in the trace graph (004) | the compiled id set | keeps the raw string |
| `compile.rs::resolve_spec_dir` | `compile --spec` (056) | `specs/` on disk | `NotFound` |
| `verify.rs::resolve_spec_id` | `verify` (049) | `specs/` on disk | `NotFound` |

083 3.2 counted three; the first row is the fourth. The two strict copies already
disagree about what they print on an ambiguous ordinal, and only one names the
candidates:

```
spec '001' is ambiguous (2 directories carry that ordinal)             # compile --spec
ambiguous spec id 001: 2 specs share that ordinal (001-a, 001-b)       # verify
```

The two lenient copies agree only because 016 2 pinned them "by behavior and a
citing comment, not by linkage". The other three arguments never got a copy at
all.

### 1.2 The exact case joins the argument onto a path

Both strict copies test the exact case as
`specs_dir.join(id).join("spec.md").is_file()`. That admits anything the
filesystem can walk to, not only ids:

```
$ spec-spine verify ../specs/070-a-malformed-id-is-refused-not-a-panic --plan   # exit 0
$ spec-spine verify 070-a-malformed-id-is-refused-not-a-panic/ --plan           # exit 0
$ spec-spine compile --spec ./070-a-malformed-id-is-refused-not-a-panic         # exit 1
  V-001 [specs/070-.../spec.md] directory './070-a-malformed-id-is-refused-not-a-panic'
  does not equal id '070-a-malformed-id-is-refused-not-a-panic'
```

The third is a false refusal. A valid spec is reported as failing validation,
because the "resolved id" is the path the caller typed, and `V-001` then
compares a directory name against it.

### 1.3 The obvious fix is wrong twice

Resolving inside `query::show` alone fixes `registry show` and silently breaks
`registry relationships`. `relationships` calls `show` for the outgoing edges,
then compares the **raw argument** against every other spec's `depends_on`,
`amends` and `supersedes` to compute the incoming ones. `registry relationships
016` would print 016's outgoing edges and an empty `depended_on_by`, where the
full id prints `033-dependency-cycle-refusal, 049-verify-declared-acceptance`.
That is a wrong answer at exit 0, which is worse than today's refusal.

`attest --spec` uses the argument in four places, and resolving it in one of
them produces a worse attestation than today's refusal. Inside
`attest::attest_spec`, the argument selects the registry record, selects the
index mapping whose units are hashed, and is written as the payload's `specId`.
Resolve only the record lookup and the mapping lookup finds nothing: the payload
lists **zero units**, `resolution.ok` stays true because nothing failed to
resolve, and `specId` reads `070`. That is a confident attestation over nothing,
at exit 0. Resolve all three and the CLI still names the output
`by-spec/<argument>.json`, so `attest --spec 070` writes `by-spec/070.json`: a
second file for one spec, which `verify-attestation --spec <full id>` never
reads.

These are all one mistake, resolving in one place and continuing to use the
unresolved string in another. 3.3 is the rule against it.

## 2. Territory

Three new files, claimed here, and ten units owned elsewhere, `extends`-ed:

| Unit | Owner | What changes |
|---|---|---|
| `crates/spec-spine-core/src/spec_id.rs` | 084 (new) | the one policy (3.4) |
| `crates/spec-spine-core/tests/spec_id.rs` | 084 (new) | the policy, the library entry points, the facade |
| `crates/spec-spine-cli/tests/spec_id.rs` | 084 (new) | the six-argument matrix (3.5) |
| `crates/spec-spine-core/src/lib.rs` | 001 | declares and re-exports the module |
| `crates/spec-spine-core/src/compile.rs` | 001 | `resolve_spec_dir` removed; `resolve_spec_ref` delegates |
| `crates/spec-spine-core/src/index.rs` | 004 | `resolve_id` delegates |
| `crates/spec-spine-core/src/verify.rs` | 049 | `resolve_spec_id` removed |
| `crates/spec-spine-core/src/query.rs` | 002 | `show` resolves; `relationships` compares the resolved id |
| `crates/spec-spine-core/src/attest.rs` | 023 | `attest_spec` resolves |
| `crates/spec-spine-cli/src/cmd_attest.rs` | 023 | the output file is named by the resolved id |
| `crates/spec-spine-cli/src/verify_attestation.rs` | 023 | resolves against the attestation files |
| `crates/spec-spine-cli/src/main.rs` | 001 | help text on each id argument; the census (3.5) |
| `crates/spec-spine-cli/src/cmd_registry.rs` | 002 | help text on `show` and `relationships` |

One `amends`, on 016, because 3.4 changes what 016 2 says about its own
territory (5, D-1). None on 002, 042, 049 or 056: each states behavior this spec
makes true rather than behavior it changes (5, D-2). `docs/api.md` gains the two
public functions of 3.4; it is on the bypass floor and needs no claim.

## 3. Behavior

### 3.1 The six arguments accept the short form, under one policy

| argument | resolves against (3.2) | short form today | after |
|---|---|---|---|
| `compile --spec <ID>` | spec directories on disk | accepted | unchanged |
| `verify <id>` | spec directories on disk | accepted | unchanged |
| `registry show <id>` | the committed registry | exit 1 | accepted |
| `registry relationships <id>` | the committed registry | exit 1 | accepted |
| `attest --spec <ID>` | the ids of its in-memory compile | exit 1 | accepted |
| `verify-attestation --spec <ID>` | the per-spec attestation files | exit 3 | accepted |

Each MUST resolve its argument through the policy of 3.4, which has four
outcomes:

1. The argument equals an id in the universe: that id.
2. Otherwise, exactly one id's whole leading dash-segment equals the argument:
   that id. `070` resolves `070-a-malformed-id-is-refused-not-a-panic`; `70`
   resolves nothing; `070-typo` resolves nothing rather than snapping to a
   neighbour (016 3.1).
3. Several ids share that segment: **ambiguous**. Refused as `Error::NotFound`,
   exit 1, with a message that contains the word `ambiguous` and names every
   candidate in sorted order. Never guessed.
4. None: **no match**. `Error::NotFound`, exit 1, except at `verify-attestation`
   (3.2).

Steps 1 and 2 are 016 3.1 unchanged. Step 3 is reachable at the argument surface
even though `V-004` forbids a shared ordinal: `compile --spec` exists for a
draft that has not yet been judged (056 3.1), and a duplicate ordinal is the
mistake a new draft makes.

For the same argument and the same candidates, the refusal of step 3 MUST be the
same message at all six arguments, and the refusal of step 4 the same message at
the five that refuse it. It is one refusal, and a reader should not be able to
tell from it which verb produced it.

Each argument's help text MUST say that it accepts the short form, as `verify`'s
already does.

### 3.2 Each verb resolves against the ids it already reads

The policy is one. The set it runs over is per verb, and it is whatever that
verb reads anyway:

- **`compile --spec` and `verify`**: the names of the directories under
  `layout.specs_dir` that contain a `spec.md`. A draft that has never compiled
  has no shard, and 056 3.1 exists for exactly that draft.
- **`registry show` and `registry relationships`**: the ids in the committed
  registry. These verbs answer from the ledger, and the facade's `query_json` is
  handed registry text and nothing else, so no other set is available to it.
- **`attest --spec`**: the ids of the compile it already runs in memory.
- **`verify-attestation --spec`**: the file stems under
  `<derived_dir>/attestation/by-spec/`. This verb reads an attestation file, not
  the corpus, and a `--signature` check is legitimate on an attestation whose
  spec has since left the corpus. A corpus-wide set would refuse it.

At `verify-attestation`, step 4 does not refuse. The argument is used as given,
and the read fails exactly as it does today: exit 3, with the hint to run
`attest --spec` first. A missing attestation file is I/O, which 042 3.5 assigns
to exit 3 (5, D-4). An ambiguous argument is refused at exit 1, as it is
everywhere else. With `--attestation <path>` the id locates nothing, so it is not
resolved, and nothing about that form changes.

`verify_attestation.rs::validate_spec_id` stays, and MUST still run on the raw
argument before any resolution. Today it refuses, at exit 3, an argument that is
empty, that contains `/`, `\` or a NUL anywhere, or that is exactly `.` or `..`.
Without a separator, `..` can only name a parent as the whole argument, so the
last two are equality tests and the first three are not. It exists because this
verb reads a file whose name is built from the argument. The fall-through of step 4 builds that name from the raw
argument, so the guard is what keeps the fall-through from reading outside
`by-spec/`. A rewrite that validated only the resolved id would reopen it on
exactly the path D-4 keeps.

Membership is a string comparison. An argument MUST NOT be joined onto a path
before it has resolved. That is what closes 1.2: `../specs/<id>`, `./<id>` and
`<id>/` are not ids, match nothing, and are refused at exit 1 instead of
resolving through the filesystem (5, D-6).

### 3.3 Resolve once, then use only the resolved id

Every value derived from a spec id after resolution MUST come from the resolved
id, never from the argument. That covers a comparison against another spec's
edges, a file name, and an emitted field. In particular:

- `relationships` computes `superseded_by`, `amended_by` and `depended_on_by`
  against the resolved id. The short and full forms of one spec MUST produce
  byte-identical output.
- `attest_spec` selects the record, selects the index mapping, and writes
  `specId` by the resolved id. Today all three use the argument, which is safe
  only because an argument that is not an exact id never gets past the first.
  The short and full forms of one spec MUST produce byte-identical payloads.
- `attest --spec` writes `<derived_dir>/attestation/by-spec/<resolved id>.json`
  and no file named after the argument.
- `verify-attestation --spec` reads the file named by the resolved id.

Resolution happens in the library entry point wherever one exists
(`query::show`, `query::relationships`, `attest::attest_spec`,
`compile::compile_spec`, `verify::plan`). The facade functions over them
(`query_json`, `attest_spec_json`, `verify_plan_json`) therefore accept the short
form with no change of their own. The library is the surface bindings wrap, and a
binding should not have to reimplement a rule the CLI got for free. The CLI
resolves only what the library does not own, which is the attestation file name.

### 3.4 One implementation

`crates/spec-spine-core/src/spec_id.rs` MUST hold the policy, and nothing else in
the workspace may reimplement it. `spec-spine-core`'s public API gains two
functions from it:

- a **match** over any set of ids, with three outcomes: resolved (steps 1 and 2
  of 3.1), ambiguous with its sorted candidates, and no match;
- a **strict** form that maps ambiguous and no match to the `Error::NotFound` of
  3.1.

The four copies of 1.1 go:

- `compile.rs::resolve_spec_dir` and `verify.rs::resolve_spec_id` are removed.
  Their callers use the strict form over the directory listing, which also lives
  in `spec_id.rs` so the two filesystem callers stop carrying two `read_dir`
  loops.
- `compile.rs::resolve_spec_ref` and `index.rs::resolve_id` keep their names and
  their contract, and become the lenient mapping of the shared match: a line
  each, not a mirror. No match and ambiguous still return the raw string, so
  `V-008` and `V-010` still name a dangling reference. The names stay because
  016 3.1 and 019 3.4 cite them.

The module is owned by neither 001 nor 004. 016 2 used a mirror to keep the
compile gate from taking a code dependency on the indexer's file, and that is
still true: both now depend on a third file instead of on each other.

### 3.5 The rule is enforced, not asserted

049 and 056 each asserted the cross-verb rule in prose and nothing held it, so
this spec adds two tests.

**The matrix** (`crates/spec-spine-cli/tests/spec_id.rs`) drives all six
arguments against one fixture corpus. For each it MUST assert that the full id
and the short id produce byte-identical stdout and the same exit code, that an
ambiguous ordinal exits 1 with the one message of 3.1, and that an unknown id
exits as it does today, including a path-shaped argument at `verify-attestation`,
which `validate_spec_id` refuses at exit 3. It MUST include the regressions 1.3
describes: the
incoming edges of `relationships` under the short form, a short-form attestation
that lists the same units as the full form, and the name of the attestation
file. Those are what a partial fix produces.

**The census** is a unit test named `spec_id_census` in
`crates/spec-spine-cli/src/main.rs`, over clap's command tree (`Cli::command()`).
It MUST collect every argument, at any subcommand depth, whose argument id is
`id` or `spec`, and fail when that set differs from the six in 3.1, with a
message naming the matrix and the module of 3.4. A seventh verb that takes a
spec id then cannot land without the resolver by accident, which is the failure
this spec exists to end. Keying on those two argument ids is a known limit
(5, D-5).

### 3.6 What must keep working

- Every committed shard is byte-identical before and after. The lenient adapters
  return what the mirrors returned for every input, so the registry and the
  index record the ids they always did. `spec-spine check` is the witness.
- A full id is accepted at all six arguments, with output unchanged.
- An unknown id keeps its exit code: 1 at five arguments, and 3 at
  `verify-attestation` when no attestation file exists.
- `verify`'s existing ambiguity test (`an_ambiguous_short_id_is_refused_not_guessed`,
  which asserts the word `ambiguous`) holds unmodified.
- No schema version moves, and no emitted field is added.

## 4. Out of scope

**Cross-checking `--spec` against an explicit `--attestation`.** With
`--attestation <path>`, `verify-attestation --spec` selects the scope and
locates nothing. Whether it should also refuse a payload whose `specId` names a
different spec is a real question about that verb, not about id spelling.

**Wider matching.** Case folding, arbitrary prefixes, and suggestions for a near
miss stay out, as 016 4 left them. An argument that matches nothing is refused,
not corrected.

**Which frontmatter fields resolve a short id.** That is 016 3.2's and 019 3.4's
decision. This spec moves where the policy lives; it does not widen where the
policy is applied.

**Editing 049 3.2 and 056 3.1.** Both sentences become true when this lands.
They are left as written (spec 040), and this spec is where a reader finds out
they were not true before it.

**`index owner <path>`.** It takes a path, not an id.

**`init`'s ordinal-collision warning.** `cmd_init.rs` reads the leading digit
run of the spec it scaffolds and warns when a sibling directory carries the
same one. It resolves no argument to an id, so it is not a copy of this policy,
and 3.4's single-copy rule does not reach it.

## 5. Resolved decisions

**D-1 (2026-09-11): `amends` 016, for its section 2.** 016 2 records the
compile resolver as "a local mirror of `index.rs::resolve_id` rather than a
shared call", pinned equal "by behavior and a citing comment, not by linkage".
3.4 makes it a shared call. That changes what 016 says about its own territory,
so it is recorded as an amendment rather than done quietly under an `extends`.
The maintainer chose on 2026-09-11 to fold the reference family in here rather
than leave it for a follow-up, accepting this edge as the cost. 016's reason for
the mirror survives by a different mechanism, since the shared module belongs to
neither 001 nor 004. 016 3.1, the policy, is unchanged, and 016's text is not
edited: an `amends` edge is declared once, in the amending spec, and the inbound
view is the compiled read `registry relationships 016`, which reports it as
`amended_by (incoming)` (spec 040).

**D-2 (2026-09-11): no `amends` on 002, 042, 049 or 056.** 002 says `show`
returns one spec or `NotFound`: a short id naming exactly one spec now returns
it, and an id naming none is still `NotFound`. 042 names the output
`by-spec/<id>.json`, and the resolved id is the spec's id. 049 3.2 and 056 3.1
require the short form at their own verbs, which keep it, and describe the other
verbs as already accepting it, which this spec makes true. An `amends` on any of
them would record a contradiction that does not exist.

**D-3 (2026-09-11): a set per verb, not one set for all.** One corpus-wide set
was the first design, and it fails three ways. `compile --spec` could not resolve
a draft that has no shard, which is 056's reason for existing. `query_json`
would need a filesystem it is never given. And `verify-attestation` would refuse
a signature check on an attestation whose spec was removed, a regression for the
one verb meant to work across a trust boundary. The policy is what must be one;
the set it runs over is what each verb reads.

**D-4 (2026-09-11): `verify-attestation` keeps exit 3 when no attestation file
matches.** Refusing at exit 1 would align it with the other five and change 042
3.5's exit code for a missing file, which is I/O. The argument falls through
unchanged instead, so an exact id, a short id with no attestation yet, and
today's message all behave as they do now. Only an argument that resolves is
newly accepted, and only an ambiguous one is newly refused.

**D-5 (2026-09-11): the census keys on the argument ids `id` and `spec`.** Those
are the two spellings all six arguments use today. A future argument spelled
differently (`--target <ID>`) would evade it. Keying on `value_name = "ID"`
instead would catch `attest --key-id`, which takes a key id; renaming value names
to tell the two apart was rejected as help-text churn to close a gap a reviewer
can see. The census and the matrix are two lists, kept equal by the census's
failure message rather than by linkage, because an integration test cannot
import from a binary crate.

**D-6 (2026-09-11): a path is not an id.** Closing 1.2 is the one behavior
change to an existing verb that is not an acceptance: `verify ../specs/<id>` and
`verify <id>/` go from exit 0 to exit 1, and `compile --spec ./<id>` goes from a
false `V-001` to `NotFound`, still exit 1. No spec states the old behavior. 049
and 056 each call the argument an id, and a path is not one; the old behavior
is what `Path::join` happens to do. It is recorded because a caller relying on
it will see exit 1 where they saw exit 0.

## Verification

Each line below is one command: spec 049 3.2 makes each fenced body line a
command, and 049 3.5 runs each one in its own `sh -c`, so no line may depend on a
variable another line set. The scratch
corpus is materialized at a fixed path for that reason, and the multi-step
comparisons run inside one `sh -c` each.

Against pre-084 code, the lines split three ways, and the split is stated rather
than left to be inferred:

- **Fail first.** The three test targets (two do not exist yet, and the census
  filter matches no test), the four short-form acceptances, the four
  byte-identity and file-name lines of 3.3, the path-is-not-an-id line, the
  ambiguity line, the no-match line, and the single-copy line. These are the
  evidence that the defect is fixed.
- **Pass before and after.** The full-id line, the three unknown-id lines, the
  `validate_spec_id` line, and the closing `check`. They are regression guards
  for 3.2 and 3.6 and are not evidence of anything 084 adds.
- **Setup and cleanup.** The three scratch-corpus lines, and the final `rm -rf`.

Four vacuous passes are closed on purpose. `cargo test <filter>` exits 0 when
the filter matches no test, so the census line requires a non-zero passed
count. `test "$A" = "$B"` passes between two empty strings, so each comparison
first requires a non-empty reading; before 084 the short-form reading is empty.
The two one-message lines also count their stderr lines, one per argument,
before counting distinct ones: a verb that wrote no refusal at all would
otherwise add nothing to the file and leave the distinct count at one. And two
comparisons guard their full-form reading against a regression that breaks both
forms alike, which equality alone cannot see: the `relationships` line requires
`049-verify-declared-acceptance`, the incoming edge a raw-argument comparison
drops, and the attest line requires at least one unit `contentHash`, which the
zero-unit payload of 1.3 does not have.

The ambiguity line puts **empty** files in the scratch corpus's attestation
directory. Their content is never read by a correct implementation, because an
ambiguous argument is refused before any read. An implementation that read
first would fail to parse them and exit 3, not 1, so the emptiness is what makes
"resolve before read" observable. The same line asserts that all six refusals
are one message, and the no-match line asserts the same of step 4 at the five
arguments that refuse it. Together they are the behavioral half of 3.4: a
private copy would have to reproduce both messages byte for byte to pass.

The single-copy line is the structural half, and it is weaker. It counts source
files that contain the ordinal-segment expression outside `spec_id.rs`, which
catches the four copies as they are written today and would not catch a fifth
spelled differently. That is why the ambiguity line exists alongside it.

The scratch corpus lines mutate a shared filesystem, which is safe only while
every assertion undoes its own mutation on the failing path as well as the
passing one. The ambiguity line removes the attestation directory and its error
file before it compares. The three setup lines are the stated exception: they
build the tree every later line reads, and the final `rm -rf` undoes them, so a
run that stops at a failing assertion leaves the corpus on disk for diagnosis,
and the next run's leading `rm -rf` rebuilds it from nothing.

The third setup line relies on `compile` writing the registry for a corpus that
fails validation, which is its contract rather than an accident: 001 3.5 has it
write the registry and then exit by the verdict, and the registry records that
verdict as `validation.passed: false` (001 3.2). The line asserts both halves,
exit 1 and the shard on disk, so a `compile` that stopped writing on failure
would fail there, loudly. It could not hollow out the ambiguity line either,
because `registry show` over a missing registry exits 3, not 1. Writing the
shards by hand instead would be the hand-edit constitution II forbids.

The lines that write into this repository's own `.derived/attestation/` write
only on-demand, gitignored attestations, and each one that depends on a file's
absence removes that file itself first.

```verify:cli
# Self-contained: the assertions below drive the release binary.
cargo build --release --locked
# 3.4 the policy, the library entry points, and the facade.
cargo test -p spec-spine-core --test spec_id --locked
# 3.5 the six-argument matrix.
cargo test -p spec-spine-cli --test spec_id --locked
# 3.5 the census; the grep refuses a filter that matched nothing, which cargo reports as a pass.
sh -c 'cargo test -p spec-spine-cli --bin spec-spine --locked spec_id_census 2>&1 | grep -q "test result: ok. [1-9]"'
# 3.1 the four arguments that refuse the short form today accept it.
target/release/spec-spine registry show 016 >/dev/null
target/release/spec-spine registry relationships 016 >/dev/null
target/release/spec-spine attest --spec 070 >/dev/null
sh -c 'target/release/spec-spine attest --spec 070-a-malformed-id-is-refused-not-a-panic >/dev/null && target/release/spec-spine verify-attestation --spec 070 --recompute >/dev/null'
# 3.3 show: the short and full forms print the same bytes.
sh -c 'A=$(target/release/spec-spine registry show 016-short-id-resolution --json); B=$(target/release/spec-spine registry show 016 --json); test -n "$A" && test "$A" = "$B"'
# 3.3 relationships: the same bytes, and the incoming edge a raw-argument comparison drops is present.
sh -c 'A=$(target/release/spec-spine registry relationships 016-short-id-resolution --json); B=$(target/release/spec-spine registry relationships 016 --json); echo "$A" | grep -q 049-verify-declared-acceptance && test "$A" = "$B"'
# 3.3 attest: the same payload bytes, which a short-form attestation over zero units would not be.
sh -c 'A=$(target/release/spec-spine attest --spec 070-a-malformed-id-is-refused-not-a-panic --json); B=$(target/release/spec-spine attest --spec 070 --json); echo "$A" | grep -q "\"contentHash\": \"" && test "$A" = "$B"'
# 3.3 attest: the file is named by the resolved id, and nothing is named after the argument.
sh -c 'D=.derived/attestation/by-spec; rm -f "$D/070.json" "$D/070-a-malformed-id-is-refused-not-a-panic.json"; target/release/spec-spine attest --spec 070 >/dev/null || exit 1; test -f "$D/070-a-malformed-id-is-refused-not-a-panic.json" && test ! -e "$D/070.json"'
# 3.2 and D-6 a path is not an id: each of these resolved through the filesystem before 084.
sh -c 'for a in ../specs/070-a-malformed-id-is-refused-not-a-panic 070-a-malformed-id-is-refused-not-a-panic/; do target/release/spec-spine verify "$a" --plan >/dev/null 2>&1; test $? -eq 1 || { echo "resolved a path: $a" >&2; exit 1; }; done; E=$(target/release/spec-spine compile --spec ./070-a-malformed-id-is-refused-not-a-panic 2>&1); echo "$E" | grep -q "not found" && ! echo "$E" | grep -q V-001'
# 3.4 no source file outside spec_id.rs carries the ordinal-segment match.
sh -c 'test $(grep -rl "split(.-.).next()" crates/spec-spine-core/src crates/spec-spine-cli/src | grep -v "/spec_id.rs$" | wc -l) -eq 0'
# 3.1 the five arguments that refuse no match do it at exit 1 with one message.
sh -c 'E="${TMPDIR:-/tmp}/ss084.nm"; : > "$E"; X=0; for c in "registry show 999" "registry relationships 999" "compile --spec 999" "verify 999 --plan" "attest --spec 999"; do target/release/spec-spine $c >/dev/null 2>>"$E"; test $? -eq 1 || { echo "not exit 1: $c" >&2; X=1; }; done; U=$(sort -u "$E" | grep -c .); L=$(grep -c . "$E"); rm -f "$E"; test $X -eq 0 && test $L -eq 5 && test $U -eq 1'
# 3.6 a full id is still accepted at all six arguments (a guard: passes before and after).
sh -c 'for c in "registry show 016-short-id-resolution" "registry relationships 016-short-id-resolution" "compile --spec 016-short-id-resolution" "verify 016-short-id-resolution --plan" "attest --spec 016-short-id-resolution" "verify-attestation --spec 016-short-id-resolution --recompute"; do target/release/spec-spine $c >/dev/null || { echo "full id refused: $c" >&2; exit 1; }; done'
# 3.6 an unknown id keeps its exit code, and a partial ordinal is not an ordinal (guards).
sh -c 'target/release/spec-spine registry show 999 >/dev/null 2>&1; test $? -eq 1'
sh -c 'target/release/spec-spine registry show 16 >/dev/null 2>&1; test $? -eq 1'
sh -c 'rm -f .derived/attestation/by-spec/016-short-id-resolution.json; target/release/spec-spine verify-attestation --spec 016 --recompute >/dev/null 2>&1; test $? -eq 3'
# 3.2 validate_spec_id still refuses a path-shaped argument at verify-attestation, at exit 3 (a guard).
sh -c 'target/release/spec-spine verify-attestation --spec ../x --recompute >/dev/null 2>&1; test $? -eq 3'
# A scratch corpus whose two specs share an ordinal, the mistake a new draft makes.
rm -rf "${TMPDIR:-/tmp}/ss084" && mkdir -p "${TMPDIR:-/tmp}/ss084/specs/001-a" "${TMPDIR:-/tmp}/ss084/specs/001-b"
sh -c 'for n in a b; do printf -- "---\nid: \"001-%s\"\ntitle: \"t\"\nstatus: draft\ncreated: \"2026-09-11\"\nsummary: \"s\"\n---\n\n# t\n" "$n" > "${TMPDIR:-/tmp}/ss084/specs/001-$n/spec.md"; done'
# compile refuses that corpus (V-004, exit 1) and still writes the shards registry show reads.
sh -c 'target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss084" compile >/dev/null 2>&1; test $? -eq 1 && test -f "${TMPDIR:-/tmp}/ss084/.derived/spec-registry/by-spec/001-a.json"'
# 3.1 all six arguments refuse the shared ordinal at exit 1, with one message naming both candidates.
sh -c 'R="${TMPDIR:-/tmp}/ss084"; E="${TMPDIR:-/tmp}/ss084.err"; D="$R/.derived/attestation/by-spec"; mkdir -p "$D"; : > "$D/001-a.json"; : > "$D/001-b.json"; : > "$E"; X=0; for c in "registry show 001" "registry relationships 001" "compile --spec 001" "verify 001 --plan" "attest --spec 001" "verify-attestation --spec 001 --recompute"; do target/release/spec-spine --repo "$R" $c >/dev/null 2>>"$E"; test $? -eq 1 || { echo "not exit 1: $c" >&2; X=1; }; done; U=$(sort -u "$E" | grep -c .); L=$(grep -c . "$E"); grep -q ambiguous "$E" && grep -q 001-a "$E" && grep -q 001-b "$E" || X=1; rm -rf "$R/.derived/attestation" "$E"; test $X -eq 0 && test $L -eq 6 && test $U -eq 1'
rm -rf "${TMPDIR:-/tmp}/ss084"
# 3.6 no committed shard moved: the lenient adapters return what the mirrors returned (a guard).
target/release/spec-spine check
```
