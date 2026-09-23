---
id: "102-a-ready-spec-carries-its-status"
title: "A ready spec carries its status"
status: draft
kind: "tooling"
created: "2026-09-21"
implementation: complete
owner: "The spec-spine Authors"
depends_on:
  - "035-registry-plan-ready-set"
  - "101-readiness-is-scheduling-not-approval"
# D-7: 087 holds spec 053's acceptance, and it pins the ready entry's exact
# shape, which this spec changed. The replacement attaches to the current
# holder (spec 082 3.2), so `verify 053` and `verify 087` both run this block.
amends: ["087-the-answer-is-a-member-not-the-document"]
amends_verification: ["087-the-answer-is-a-member-not-the-document"]
summary: >
  `ReadySpec` carries `id` and `title`, so a consumer reading `registry plan`
  directly cannot apply an approval rule without a second query per entry. An
  additive `status` field is a read-schema MINOR that lets a direct consumer
  see what `/next` sees. An enhancement with a named cost, not a defect fix.
extends:
  - spec: "035-registry-plan-ready-set"
    paths:
      - "crates/spec-spine-core/src/query.rs"
      - "crates/spec-spine-core/tests/query.rs"
    nature: additive
  - spec: "057-the-docs-name-what-adopters-derived"
    unit: { kind: file, path: "docs/api.md" }
    nature: additive
  # 3.3: the read-schema MINOR moves the one constant every read document
  # carries, and the table that documents it (D-3).
  - spec: "074-a-governed-read-names-its-version"
    unit: { kind: file, path: "crates/spec-spine-types/src/version.rs" }
    nature: additive
  - spec: "057-the-docs-name-what-adopters-derived"
    unit: { kind: file, path: "docs/schema-versioning.md" }
    nature: additive
  # D-5: the two existing tests that pinned the old shape and the old version.
  - spec: "074-a-governed-read-names-its-version"
    unit: { kind: file, path: "crates/spec-spine-core/tests/read.rs" }
    nature: additive
  - spec: "053-plan-answers-the-whole-question"
    unit: { kind: file, path: "crates/spec-spine-cli/tests/cli.rs" }
    nature: additive
---

# 102: A ready spec carries its status

## 1. Purpose

Spec 101 makes the contract truthful in prose: readiness is scheduling and the
approval rule belongs to the consumer. This spec makes it **cheap to apply**.

`ReadySpec` is `{ id, title }`. A consumer holding the plan document and
wanting to drop unapproved specs, which is what this repository's own `/next`
does and what spec 101 tells every consumer to do, must issue one
`registry show <id>` per ready entry to learn a field the planner already had
in hand when it built the set.

## 1.1 Why this is filed separately from 101

Note 05 §9.3 says it in one sentence and this spec exists to honor it: the field
"is a reasonable spec but it is not required to make the contract truthful, and
it should not be filed as a defect fix."

Two consequences, both deliberate:

- **101 does not depend on this spec landing.** The documentation is the fix;
  this is an ergonomic improvement on top of it.
- ~~**This spec is buildable only for a named consumer.**~~ Superseded
  2026-09-22 by D-2: it is buildable on the owner's opportunity evaluation
  (design note 09 section 10). Filing it recorded the contract; D-2 schedules
  it.

## 2. Territory

`ReadySpec` in `crates/spec-spine-core/src/query.rs`, its acceptance in
`crates/spec-spine-core/tests/query.rs`, and the `plan` paragraph of
`docs/api.md` that spec 101 rewrote. The read-schema MINOR in §3.3 also moves
`READ_SCHEMA_VERSION` in `crates/spec-spine-types/src/version.rs` and its row
in `docs/schema-versioning.md` (D-3).

## 3. Behavior

### 3.1 One additive field

`ReadySpec` gains exactly one member:

```json
{ "id": "100-a-deleted-path-is-judged-where-it-lived",
  "title": "A deleted path is judged where it lived",
  "status": "draft" }
```

`status` MUST be the spec's `status` frontmatter value as the registry records
it, verbatim and unmapped. It MUST NOT be a derived boolean such as `approved:
true`: the corpus has four statuses, two of which (`superseded`, `retired`) the
planner excludes, and a boolean would answer a question this spec is not
entitled to answer on the consumer's behalf.

### 3.2 It changes no partition and no ordering

The ready set's membership MUST be identical before and after. `status` is
reported, never consulted. Spec 035 §3.1's partition rules and spec 053's
ordering contract are untouched, and the acceptance MUST assert that the id
sequence of `ready` is unchanged across the field's introduction.

### 3.3 The version that moves, and the one that does not

Adding a member to a read document is **additive**, so the read schema takes a
MINOR bump under `docs/schema-versioning.md`, and a loader rejecting an unknown
MAJOR is unaffected. The registry schema does **not** move: no registry shard
gains a field and no `shardHash` changes, because the hash is over `spec.md`
source bytes.

`BlockedSpec` gained `title` the same way under spec 053, which is the
precedent for both the shape and the version handling.

### 3.4 `blocked` is left alone

A blocked entry is not a candidate to approve, so carrying `status` on
`BlockedSpec` would add a field no consumer has asked for. This spec MUST NOT
add it. If it is later wanted, that is a second MINOR in its own spec, justified
on its own evaluation (D-2), not folded into this one.

## 4. Out of scope

- **Filtering by approval inside `plan`.** That would move the consumer's rule
  into the engine and undo the layering spec 101 §1.1 preserves.
- **`implementation` on `ReadySpec`.** The planner consults it, so reporting it
  is defensible. Same rule as §3.4: its own spec, its own evaluation.
- **Any change to `/next`.** It resolves `status` per entry today and would
  simply stop needing to.

## 5. Resolved decisions

*(Filed as a draft. Decisions taken during the build are appended here.)*

**D-1 (2026-09-21, specified now, built for a named consumer).** *Superseded
2026-09-22 by D-2; preserved as the record of the earlier condition.* This spec is
filed so the contract exists and is reviewable, not so it is scheduled. It
should be built when a consumer names the need: an adopter's `/next`
equivalent, an orchestrator's scheduling stage, or a dashboard that renders the
plan. That is the disposition
`docs/design/09-disposition-2026-09-21.md` §5 proposes as a clarification of
grand-refactor's SP-03, and it is recorded here as this spec's own build
condition rather than as an adoption of SP-03, which is not this corpus's to
make.

**D-2 (2026-09-22, the named-consumer condition is withdrawn by the owner).**
The owner replaced "specify now; implement only for a named consumer need"
with an opportunity-led evaluation and named this spec in the ruling (design
note 09 section 10, D-7). A consumer request is evidence, not a prerequisite.
The contract in section 3 is unchanged by this decision: the field, its
verbatim value, the untouched partition and ordering, the read-schema MINOR,
and `blocked` left alone all stand. What changed is only that the build is
authorized. No consumer has asked for the field, and this spec does not claim
one has.

**D-3 (2026-09-22, territory corrected before the build).** §3.3 requires a
read-schema MINOR, and the filed territory did not include the constant that
carries it or the document that tables it. Both are added as `extends` edges:
`version.rs` on spec 074, which introduced `READ_SCHEMA_VERSION`, and
`docs/schema-versioning.md` on spec 057, which established the table. No
behavior in section 3 changes. The version moves once, for every read
document, because spec 074 made it one axis ("per-verb axes would always move
together"); that is the precedent this spec follows rather than a choice it
makes.

**D-4 (2026-09-22, build: "never consulted" is about this field, and a
sentence in `docs/api.md` was wrong).** The first draft of the §3.2 test
flipped every status between `draft` and `approved` and expected an identical
ready set. It was not identical: a spec with no `implementation` key is
scheduled when `draft` and settled when `approved`, which is spec 042's
absent-key rule. That rule predates this field and is unchanged by it; §3.2's
guarantee is that the new member is never read, not that `status` never is.
The test now flips statuses only for specs that declare `implementation`, and
a second test asserts the 042 case and that its ready entry reports the status
that scheduled it. The same measurement showed `docs/api.md` saying `status` is
consulted "only" to exclude `superseded` and `retired`; the sentence now names
the 042 case too. It sits in the `plan` paragraph this spec extends.

**D-5 (2026-09-22, build: two existing pins moved with the contract).** The
workspace tests found two assertions of the old shape. `cli.rs`'s
`registry_plan_partitions_the_corpus` compared a ready entry to
`{ id, title }`; it now expects `status` too, and still checks that blocked
entries are unchanged. The same test's `plan --next --json` case gains `status`
as well, because the pick is a `ReadySpec`: one type, one shape, and no second
member added anywhere. `read.rs` pinned `READ_SCHEMA_VERSION` to `0.1.0` as
"the axis starts at 0.1.0"; it now pins `0.2.0` and says where each value came
from. Both files are other specs' territory, so both are declared here as
`extends` edges rather than edited silently. Neither assertion was loosened:
each still pins an exact value. Spec 074's own acceptance greps for the
constant's name, not its value, and is unaffected.

**D-6 (2026-09-22, review: the spelling helper).** The helper that spells a
`Status` fell back to an empty string on any non-string serialization, and it
had been inserted between `plan`'s rustdoc and `pub fn plan`, so the public
function lost its documentation. It is now an exhaustive `match` above that
block, returning the four spellings; a new `Status` variant fails to compile
rather than reaching a consumer as `""`, and a unit test pins every arm to
serde's spelling so the plan document and a registry shard cannot disagree.
The member stays `status: String`, as §3.1 states.

**D-7 (2026-09-23, the acceptance this spec broke, repaired by amendment).**
Spec 053's acceptance is held by spec 087 (`amends_verification`), and three
of its lines compare a ready entry, and the `--next` pick, to exactly
`{ id, title }`. D-5 moved the two test pins of that shape and missed these,
because a pin inside a `## Verification` block is not a test the build runs.
The Acceptance push leg on `main` has been red on `053` since this spec's merge
(`75a998f7`), and nothing on a pull request runs it.

Both 053 and 087 are `approved`, so neither file is edited (spec 037 §3.1).
This spec declares `amends` and `amends_verification` on 087, the current
holder (spec 082 §3.2), and its block below carries 087's acceptance in full,
with each exact-shape assertion kept exact and extended to
`{ id, title, status }`: the pick is still compared by value, so a `--next`
that dropped a member still fails. 087's own mechanism lines are kept, and a
line asserts that 087's and 053's files still carry the superseded form, so a
later edit to either approved file goes red here.

Spec 082 §3.4 requires every spec whose acceptance another holds to say so
above its own fence, naming the holder. 087 gains that note, naming this spec,
and 053's existing note gains one sentence saying 087's acceptance is now held
here. The notes are the rule's own mandated edit; no command under either
fence changes, and the lines this block greps in both files are untouched.

The carried copy cites current ids. 087's block was written before the
renumber (spec 095) and names 053 as `060` and itself as `109`, in comments
and in scratch-file labels (`ss060`, `ss109-*`). 087's own copy keeps them
verbatim; this copy says `053` and `087`, because a reader following a comment
here should land on the spec it means (review of #314).

## Verification

Written to fail against the tree this spec is filed on: the field does not
exist.

Since D-7 this block is also spec 087's acceptance, and through 087 spec 053's
(spec 082 §3.2), so it is read in three labelled parts. The carried part is
087's block with each exact-shape assertion extended by `status`, and nothing
dropped.

```verify:cli
# --- spec 053's acceptance, held by 087 and carried here (D-7) ---
# Self-contained: the commands below invoke the release binary.
cargo build --release --locked
cargo test -p spec-spine-core --test query --locked
# A scratch corpus with two ready specs and one blocked by the first, so the
# shape assertions below hold whatever this repository's own backlog is doing,
# and so "the first element of ready" has a failing case (3.4, D-3).
rm -rf "${TMPDIR:-/tmp}/ss053" && mkdir -p "${TMPDIR:-/tmp}/ss053/specs/001-alpha" "${TMPDIR:-/tmp}/ss053/specs/002-beta" "${TMPDIR:-/tmp}/ss053/specs/003-gamma" && : > "${TMPDIR:-/tmp}/ss053/spec-spine.toml" && printf -- '---\nid: "001-alpha"\ntitle: "First thing"\nstatus: approved\ncreated: "2026-09-07"\nsummary: "s"\nimplementation: pending\nestablishes:\n  - "specs/001-alpha/spec.md"\n---\n\n# 001-alpha\n## body\n' > "${TMPDIR:-/tmp}/ss053/specs/001-alpha/spec.md" && printf -- '---\nid: "002-beta"\ntitle: "Second thing"\nstatus: approved\ncreated: "2026-09-07"\nsummary: "s"\nimplementation: pending\ndepends_on:\n  - "001-alpha"\nestablishes:\n  - "specs/002-beta/spec.md"\n---\n\n# 002-beta\n## body\n' > "${TMPDIR:-/tmp}/ss053/specs/002-beta/spec.md" && printf -- '---\nid: "003-gamma"\ntitle: "Third thing"\nstatus: approved\ncreated: "2026-09-07"\nsummary: "s"\nimplementation: pending\nestablishes:\n  - "specs/003-gamma/spec.md"\n---\n\n# 003-gamma\n## body\n' > "${TMPDIR:-/tmp}/ss053/specs/003-gamma/spec.md" && target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss053" compile >/dev/null
# The two documents are captured to files, so each verb's own exit status is its
# line's status, which a pipeline into `python3` would not be (3.2, D-5).
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss053" registry plan --json > "${TMPDIR:-/tmp}/ss053/plan.json"
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss053" registry plan --next --json > "${TMPDIR:-/tmp}/ss053/next.json"
# 053 3.3: the ready array carries titles, so no consumer needs a second call.
# With two ready specs this is also a claim about order (3.4).
python3 -c "import json; p=json.load(open('${TMPDIR:-/tmp}/ss053/plan.json')); assert p['ready'][0]=={'id':'001-alpha','title':'First thing','status':'approved'}, p"
# 053 3.1: and each blocked entry carries its title and the state of every
# blocker, rather than a count of them.
python3 -c "import json; b=json.load(open('${TMPDIR:-/tmp}/ss053/plan.json'))['blocked'][0]; assert b['title']=='Second thing', b; assert b['blockedBy'][0]['id']=='001-alpha', b; assert b['blockedBy'][0]['state'], b"
# 053 3.1: the prose form renders what the structure holds, remainder included.
# Captured once rather than piped twice, so the verb's own exit status is a
# line's status and the two assertions read the same rendering (3.2).
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss053" registry plan > "${TMPDIR:-/tmp}/ss053/plan.txt"
grep -q 'not schedulable' "${TMPDIR:-/tmp}/ss053/plan.txt"
grep -q 'blocked by 001-alpha' "${TMPDIR:-/tmp}/ss053/plan.txt"
# 087 3.3: 053 3.2's pick, read from the member spec 074 moved it into. Sorted keys
# and the version member are 074 3.2's rule for every governed read, and the
# pick is compared by value so a `--next` that dropped the title fails (D-1).
python3 -c "import json; d=json.load(open('${TMPDIR:-/tmp}/ss053/next.json')); k=list(d); assert k==sorted(k), k; assert d['schemaVersion'], d; assert d['next']=={'id':'001-alpha','title':'First thing','status':'approved'}, d"
# 3.4: 053 3.2's projection requirement, which a one-element ready set cannot
# show. `--next` names the first element of `ready` rather than reimplementing
# selection; with two ready specs, answering `003-gamma` fails this line.
python3 -c "import json; n=json.load(open('${TMPDIR:-/tmp}/ss053/next.json'))['next']; p=json.load(open('${TMPDIR:-/tmp}/ss053/plan.json')); assert p['ready'][0]==n, (p['ready'], n)"
# 053 3.2: and an empty ready set is a true answer at exit 0, not a failure.
# This repository is that case now. Nothing is asserted about the contents:
# that document's `next: null` path is guarded by spec 074 3.8 in
# crates/spec-spine-cli/tests/cli.rs, and asserting it here would pin corpus
# state, which 053's own decision of 2026-09-08 rejects (3.6, D-4).
target/release/spec-spine registry plan --next
rm -rf "${TMPDIR:-/tmp}/ss053"
# 053 3.3: the ledger is untouched by a read verb.
target/release/spec-spine compile --check
# --- spec 087's own mechanism (087 3.5), carried unchanged ---
# The replacement is declared, read through the CLI rather than off the shard.
# Redirected, not piped, for the reason D-5 gives: at the parent commit this
# verb exits 1 and prints nothing. The file is named for this spec, whose
# mechanism it is, not for 053, whose acceptance the half above is (087 3.2).
target/release/spec-spine registry show 087 --json > "${TMPDIR:-/tmp}/ss087-show.json"
python3 -c "import json; d=json.load(open('${TMPDIR:-/tmp}/ss087-show.json')); assert d['amendsVerification'] == ['053-plan-answers-the-whole-question'], d; assert d['amends'] == ['053-plan-answers-the-whole-question'], d"
rm -f "${TMPDIR:-/tmp}/ss087-show.json"
# Spec 053's file is not edited (spec 037 3.1): its own block still carries the
# superseded whole-document equality. This goes red the moment someone resolves
# this by editing 053 instead.
grep -qF 'assert json.load(sys.stdin)=={"id":"001-alpha","title":"First thing"}' specs/053-plan-answers-the-whole-question/spec.md
# The resolution this spec relies on is spec 082's and is unchanged here, so
# what is asserted is that mechanism, not a new one. No `verify` command may
# appear in this block: it is the block `verify 053`, `verify 087` and
# `verify 102` run, and cmd_verify's
# re-entry guard refuses a nested call before it honours `--plan` (3.5).
# The run is captured and its summary asserted to name a non-zero pass count: a
# name filter that matches nothing exits 0, so the bare line would stay green
# while asserting nothing (spec 084 D-7).
cargo test -p spec-spine-core --test verify --locked spec103_ > "${TMPDIR:-/tmp}/ss087-verify-tests.txt" 2>&1
grep -qE 'test result: ok\. [1-9][0-9]* passed' "${TMPDIR:-/tmp}/ss087-verify-tests.txt"
rm -f "${TMPDIR:-/tmp}/ss087-verify-tests.txt"
# --- this spec's amendment of 087 (D-7) ---
target/release/spec-spine registry show 102 --json > "${TMPDIR:-/tmp}/ss102-show.json"
python3 -c "import json; d=json.load(open('${TMPDIR:-/tmp}/ss102-show.json')); assert d['amendsVerification'] == ['087-the-answer-is-a-member-not-the-document'], d; assert d['amends'] == ['087-the-answer-is-a-member-not-the-document'], d"
rm -f "${TMPDIR:-/tmp}/ss102-show.json"
# 087's commands are not edited either (only its superseded note, spec 082
# 3.4): it still carries the two-member form this block replaced. Red if
# someone repairs 087's commands in place instead.
grep -qF "assert p['ready'][0]=={'id':'001-alpha','title':'First thing'}, p" specs/087-the-answer-is-a-member-not-the-document/spec.md
# --- this spec's own acceptance ---
# 3.1: the field exists, and is the verbatim status string.
grep -q 'pub struct ReadySpec' crates/spec-spine-core/src/query.rs
grep -A6 'pub struct ReadySpec' crates/spec-spine-core/src/query.rs | grep -q 'pub status: String'
# 3.1: and is not a derived boolean.
test -z "$(grep -A6 'pub struct ReadySpec' crates/spec-spine-core/src/query.rs | grep 'approved: bool')"
# 3.4: blocked entries did not gain it.
test -z "$(grep -A8 'pub struct BlockedSpec' crates/spec-spine-core/src/query.rs | grep 'pub status')"
# 3.2: membership and order are unchanged, asserted by name.
grep -q 'ready_order_is_unchanged_by_status' crates/spec-spine-core/tests/query.rs
cargo test -p spec-spine-core --test query --locked
# 3.3: the read schema version moved and the registry schema did not.
grep -q 'READ_SCHEMA_VERSION' crates/spec-spine-types/src/version.rs
cargo test --workspace emitted_registry_conforms --locked
# The document actually carries it.
./target/release/spec-spine registry plan --json
./target/release/spec-spine check --fail-on-unresolved --fail-on-warn
```
