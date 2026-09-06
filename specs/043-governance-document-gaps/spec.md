---
id: "043-governance-document-gaps"
title: "Three governance statements the first adopter could not act on"
status: draft
kind: "governance"
created: "2026-09-05"
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "006-init-scaffold"
  - "040-amendment-authoring"
amends:
  # 000 3 classifies `depends_on` and `implementation` as descriptive. Specs
  # 033, 038 and 041 give them mechanical consequences. See 3.2. 000's own
  # text is untouched (spec 040).
  - "000-spec-spine-bootstrap"
establishes:
  # The answer spec 040 4 left open, stated where a reader checking the edge
  # vocabulary will find it.
  - { kind: section, file: "standards/spec/contract.md", anchor: "amending-the-constitution" }
refines:
  # The two tier-2 principles this spec rewrites. The constitution is on the
  # gate's bypass floor, so these claims are ledger facts, not gate-enforced
  # ones; 3.1.3 says so rather than implying enforcement.
  - { aspect: "constitution-amendment", unit: { kind: section, file: "standards/spec/constitution.md", anchor: "amendment" } }
  - { aspect: "adopted-code-as-evidence", unit: { kind: section, file: "standards/spec/constitution.md", anchor: "v-legacy-as-evidence" } }
extends:
  - { spec: "006-init-scaffold", unit: "crates/spec-spine-core/src/scaffold.rs", nature: additive }
  - { spec: "006-init-scaffold", unit: "crates/spec-spine-core/tests/scaffold.rs", nature: additive }
references:
  - { unit: { kind: file, path: "specs/040-amendment-authoring/spec.md" }, role: context }
  - { unit: { kind: file, path: "specs/041-completion-held-to-claims/spec.md" }, role: context }
summary: >
  hqgit is the first repository outside this one to adopt spec-spine, and what
  its author had to write by hand is a measurement of what the governance
  documents fail to supply. Three findings. First, the constitution's own
  Amendment clause names a mechanism that cannot be written: it says the
  constitution may be amended by "an ordinary spec that `amends` it", but
  `amends` resolves to spec ids and the constitution is not a spec, so the
  sentence has no implementation, and hqgit copied it verbatim into a corpus
  where it is equally unwritable. Spec 040 4 named this as an open question
  and deliberately did not answer it; this spec answers it. Second, spec 000 3
  files `depends_on` and `implementation` under "optional descriptive keys",
  which was true when it was written and is now false: 033 refuses a cycle in
  one, and 038 and 041 make the other decide scheduling and diagnostic
  severity. hqgit had to write its own "lifecycle as scheduling" section
  because tier 1 reads as though the field were inert. Third, constitution V
  says pre-existing code is evidence but never says what to write about code
  adopted with defects; hqgit supplied the missing half. The scaffold learns
  all three, because the scaffolded constitution is four bullets with no tier
  statement, no normative hierarchy and no amendment clause, and the evidence
  is that the first adopter did not use it.
---

# 043: Three governance statements the first adopter could not act on

## 1. Purpose

Every claim in this corpus so far has been tested against one repository. hqgit
is the second: a 64-spec greenfield corpus, governed by spec-spine 0.11.0, whose
`spec-spine.toml` turns on `require_ownership` from the first commit and whose
sessions are driven by an orchestrator against the corpus itself.

An adoption is a measurement. What an adopter uses unchanged is load-bearing;
what it rewrites is a preference; and what it has to **write from nothing**, in
a document this project ships, is a defect in that document. This spec acts on
the third category only. Three items qualify.

### 1.1 The constitution cannot be amended by the mechanism it names

`standards/spec/constitution.md` ends:

> This constitution may be amended by an ordinary spec that `amends` it and is
> approved, **provided** the amendment does not contradict a `specs/000`
> `unamendable` anchor.

`amends` is a typed edge whose targets are spec ids: `amends:
["033-dependency-cycle-refusal"]`. The constitution is `standards/spec/constitution.md`.
There is no id to name, so the edge the clause instructs an author to declare
cannot be declared. The clause states a precondition (do not contradict an
anchor) attached to a procedure that does not exist.

Spec 040 4 saw this and set it aside explicitly:

> **Amending the constitution.** Whether an ordinary spec can amend
> `standards/spec/constitution.md`, and by what edge given that `amends`
> resolves to spec ids and the constitution is not a spec, is a real question
> this spec does not answer.

It did not need to, because 040 was about the `amends` edge between specs. The
question is now due, for a reason 040 could not have: hqgit's constitution
carries the same paragraph, word for word, over fifteen principles instead of
five. The defect propagated on first contact. An adopter reading it either
concludes the constitution is frozen, or invents a local procedure, and the
corpus has no way to tell which happened.

### 1.2 Tier 1 calls two operational fields descriptive

Spec 000 3 lists the frontmatter grammar. Under **Optional descriptive keys**
it puts `authors`, `owner`, `risk`, `depends_on`, `code_aliases`,
`feature_branch` and `implementation`.

That was accurate on 2026-06-08. It is not accurate now:

| Key | Mechanical consequence | Where |
|---|---|---|
| `depends_on` | a cycle is a compile-time refusal | spec 033 |
| `depends_on` | the topological order of the ready set | spec 038 |
| `implementation` | which specs a scheduler may hand out | spec 038 3.1 |
| `implementation` | whether an unresolved owning unit is `W-001` or a hard error | spec 041 3.1 |

Two of the seven keys in a list titled *descriptive* decide a compile failure, a
scheduling answer and a diagnostic tier. A reader who trusts tier 1, which is
what tier 1 is for, will conclude that neither field does anything.

hqgit's bootstrap spec answers this by adding a section its author had to write:
`8. Lifecycle as scheduling`, stating that `approved` plus `pending` is a work
order, that `draft` is never schedulable, and that `complete` and `n-a` count as
shipped. That section exists because ours does not.

### 1.3 Constitution V has no operational half

Principle V says code predating a governing spec is evidence, not a violation,
and that a spec claiming such code declares `origin.retroactive: true`. It says
what the code *is* and which marker to set. It does not say what the adopting
spec should **write** about code it did not author and would not have written
that way.

hqgit supplies the missing sentence: such code is specced as found, and its
defects are recorded under a `## Known defects` heading rather than described as
if intended. Without that, an adopting spec has two bad options: describe the
code accurately and thereby ratify its defects as the specified behavior, or
describe the intended behavior and ship a spec that its own coupling gate will
contradict.

### 1.4 What this measurement also says about the scaffold

`spec-spine init` scaffolds a constitution. It is four numbered bullets with no
tier statement, no normative hierarchy, no amendment clause, and no indication
of where an adopter's own principles belong.

hqgit's constitution has the tier statement, the four-level normative hierarchy
block, roman-numeral principles, and the `## Amendment` paragraph verbatim, all
of which are the shape of `standards/spec/constitution.md` in **this** repository
and none of which are in the scaffold's output. The observable is that the
document an adopter ends up with resembles our own file and not the one the tool
writes.

That is not a complaint about brevity. A scaffolded constitution that omits the
amendment clause omits the only paragraph explaining how the document may ever
change, which is the paragraph an adopter needs first and the one 1.1 shows is
broken anyway. Fixing 1.1 and 1.3 without fixing the scaffold would leave every
future adopter to rediscover both.

## 2. Territory

Two governance documents and the generator that ships their equivalents:

- `standards/spec/constitution.md`: the `## Amendment` section is rewritten to
  state a mechanism that can be executed, and `## V. Legacy as evidence` gains
  its operational half. Both are claimed by `refines` edges naming the aspect.
- `standards/spec/contract.md`: one new `## Amending the constitution` section,
  placed immediately after `## Amendment authoring` so that a reader who has just
  learned how an `amends` edge is authored learns in the same place why the
  constitution is not amended that way. Claimed by `establishes`, the shape spec
  040 used for the section beside it.
- `crates/spec-spine-core/src/scaffold.rs` and its integration test: the
  scaffolded `CONSTITUTION` and `CONTRACT` learn 3.1 and 3.3, and the
  constitution gains the seam 1.4 describes. Additive to spec 006's territory.

Spec 000 is **not** edited. Its 3 classification is amended by the edge in this
spec's frontmatter and by nothing else, which is the rule spec 040 3.1 states.

## 3. Behavior

### 3.1 Amending the constitution

#### 3.1.1 The mechanism

The constitution is changed by an ordinary spec that:

1. is `status: approved` (a draft changes nothing, per 000 3),
2. **claims the affected constitution text as an authority unit**, using the
   ordinary ownership vocabulary rather than `amends`:
   - `establishes` a `{ kind: section, file: <constitution>, anchor: <slug> }`
     unit for a principle it adds,
   - `refines` that unit, with a named `aspect`, for a principle it tightens or
     restates,
   - `co_authority` on that unit where a principle is genuinely shared, and
3. does not contradict any anchor in the `unamendable` list of `specs/000`.

The section anchor is the heading slug computed by the indexer, so
`## V. Legacy as evidence` is `v-legacy-as-evidence`. This spec's own frontmatter
is the worked example: two `refines` edges, one per principle touched.

`amends` is not the instrument, and the reason is not a technicality. `amends`
means *co-authority over another spec's `spec.md`* (000 4.1), and it exists so
that editing the amender can clear a gate violation on the amended spec's
territory (005 3.3). Neither half applies to a file that is not a spec. The
edge vocabulary already had the right instrument, which is why this spec adds no
mechanism.

#### 3.1.2 The constitution is edited in place, and why that is not spec 040's rule reversed

Spec 040 3.1 forbids editing an amended spec to record that it was amended.
This spec directs an author to edit the constitution. The two are consistent, and
the distinction is worth stating because it is the first thing a careful reader
will challenge.

A `spec.md` is a **record**: it says what the corpus held when that spec was
ratified, and constitution V's "who established this" question is answerable only
while the earlier document still says what it said. A later spec narrowing it does
not make the earlier text wrong; it makes it historical.

The constitution is a **standing document**: every principle in it is a claim
about what is true now. A constitution annotated with superseded principles and
back-pointers would not preserve history, it would stop being readable as a
statement of current principle, which is its only job. History for the
constitution lives where history for everything else lives: in the specs that
claimed each section, and in git.

So the test is not "is this file edited in place" but "is this file a record or a
standing statement". Records are appended to and never rewritten; standing
statements are rewritten and their history is the ledger.

#### 3.1.3 The claim is a ledger fact, not a gate refusal

`standards/spec/constitution.md` is on the coupling gate's built-in bypass floor
(`couple.rs::DEFAULT_BYPASS_PREFIXES`). A change to it therefore raises no
`C-001`, whether or not a spec claims the section.

This is stated rather than quietly worked around, because an author who declares
the edge in 3.1.1 and expects the gate to defend it would be wrong. What the
claim buys is what the ledger always buys: `spec-spine registry relationships`
and `index render` answer "which spec owns this principle", and the next author
to touch it can find the reasoning instead of guessing. The gate's silence here
is deliberate: a governance document is not code, and refusing a constitutional
edit for want of a matching source change would be the wrong refusal.

An adopter who wants the enforcement can have it, by removing nothing (the floor
is additive-only and cannot be shrunk from config) but by claiming the sections
and reviewing the edges. That limitation is real and is recorded here rather than
in a footnote.

### 3.2 `depends_on` and `implementation` are lifecycle keys

Spec 000 3's frontmatter grammar is amended as follows. No `unamendable`
anchor is touched: 3 is the grammar section, and none of the six frozen anchors
covers it.

`depends_on` and `implementation` are removed from **Optional descriptive keys**
and stated as **lifecycle and ordering keys**: optional to declare, and not inert
once declared. Declaring either one submits the spec to whatever mechanical
consequences the corpus's ordinary specs attach to it.

The consequences themselves stay where they are specified. This spec deliberately
does not restate 033's cycle refusal, 038's partition or 041's severity rule;
tier 1 says the fields are operational, and the ordinary specs say what the
operations are. Tier 1 restating an ordinary spec's rule would create exactly the
duplicated-fact rot that spec 040 3.2 argues against.

The remaining descriptive keys (`authors`, `owner`, `risk`, `code_aliases`,
`feature_branch`) are unaffected: nothing reads them but a human.

### 3.3 Adopted code is specced as found

Constitution V gains its operational half:

> Code adopted from outside the corpus is specced **as found**. The adopting
> spec describes the behavior that exists, and records the behavior it would not
> have chosen under a `## Known defects` heading, with the defect named. A defect
> recorded under that heading is not thereby blessed: it is the reason a later
> spec can be written against it.

This closes the dilemma in 1.3. Describing the code accurately no longer ratifies
its defects, because the heading marks the difference between "this is the
contract" and "this is what is there".

`origin.retroactive: true` is unchanged and still marks the pre-graph claim. The
two are orthogonal: `retroactive` says *when* the authority began, `## Known
defects` says *what* the adopting spec thinks of what it found.

### 3.4 The scaffold ships all three

`scaffold_init` (spec 006) is additively updated:

- **`CONSTITUTION`** gains the tier statement, the normative hierarchy, an
  `## Amendment` section stating 3.1's mechanism, principle IV in the form 3.3
  gives it, and an explicit seam saying that principles from the next numeral
  onward are the adopter's own and govern the system the corpus describes. The
  five existing principles keep their substance.
- **`CONTRACT`** gains one line pointing at the amendment mechanism, in the same
  register as the rest of that file.

The scaffolded corpus MUST still compile and lint clean (`tests/scaffold.rs`),
and the generator MUST stay a pure function of `Config` with no new IO. The test
gains an assertion that the scaffolded constitution states an amendment
mechanism, so the 1.1 defect cannot silently return.

## 4. Out of scope

**Enforcing the amendment procedure.** Nothing checks that a spec editing the
constitution declared an edge for the section it edited, and 3.1.3 explains why
the gate is silent there by construction. A lint that cross-referenced constitution
headings against claimed section units is conceivable and is not specified here;
it would need a rule for headings no spec has yet claimed, of which there are
currently five.

**The remaining hqgit findings.** The adoption surfaced more than three items.
Two were considered and deliberately excluded:

- *The ordinal as build order.* hqgit's bootstrap 2 requires every `depends_on`
  target to be lower-numbered, and hqgit ships `scripts/spec-dag.sh` to check it
  because spec 033 refuses cycles only. That is a genuine tooling gap, but the
  rule is a corpus convention rather than a universal truth: a corpus that files
  specs by topic rather than by build order would be refused by it for no reason.
  It belongs in a feature spec with an opt-in config knob, not in tier 1.
- *`## Verification` blocks.* hqgit's specs carry `verify:cli` fenced blocks and a
  `scripts/verify-spec.sh` runner, so that acceptance is executed against the
  merged sha rather than asserted. This is the most substantial thing the
  adoption invented and the one most worth having, and it is a capability, not a
  document correction. Spec 042 already moves in that direction. Filing it
  properly is a separate spec.

**Rewriting the scaffolded bootstrap spec.** `bootstrap_spec()` is two sections
and has the same thinness problem as the constitution had. It is left alone here
so that this spec's scaffold change stays reviewable against 3.4's list.
