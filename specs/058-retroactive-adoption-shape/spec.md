---
id: "058-retroactive-adoption-shape"
title: "The defects heading has one spelling"
status: draft
kind: "governance"
created: "2026-09-07"
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "003-conformance-lint"
  - "022-keypath-section-anchors"
  - "043-governance-document-gaps"
extends:
  - { spec: "003-conformance-lint", unit: "crates/spec-spine-core/src/lint.rs", nature: additive }
  # The anchor rule itself, beside the slug it is asked of (3.1).
  - { spec: "022-keypath-section-anchors", unit: "crates/spec-spine-core/src/sections.rs", nature: additive }
  - { spec: "003-conformance-lint", unit: "crates/spec-spine-core/tests/lint.rs", nature: additive }
refines:
  # 043 refined this principle with the aspect `adopted-code-as-evidence`,
  # adding the heading. This spec refines a different aspect of the same
  # section: what counts as that heading. Not `amends` (its targets are spec
  # ids and the constitution is not a spec) and not `co_authority` (the two
  # aspects are separable, not shared), per the amendment clause 043 wrote.
  - { aspect: "defects-heading-anchor", unit: { kind: section, file: "standards/spec/constitution.md", anchor: "v-legacy-as-evidence" } }
references:
  - { unit: { kind: file, path: "docs/design/03-adopter-audit-2026-09.md" }, role: context }
summary: >
  Constitution V requires code adopted from outside the corpus to be specced as
  found, with the behavior the adopting spec would not have chosen recorded
  under a `## Known defects` heading. It does not say what counts as that
  heading, and the first consumer to check mechanically got it wrong:
  claude-observatory's `defects.ts` matches the string `## Known defects`
  exactly and therefore rejects the numbered headings of the very specs it cites
  as precedent. This spec settles the spelling as an anchor rather than a
  string: any heading whose computed slug is or ends with `known-defects`, at
  any level, under the same slug rule the indexer already uses for section
  units. It adds `L-009`, an info-tier lint for a near-miss spelling, and it
  declines to lint `origin.retroactive` into implying the heading, because spec
  043 established that the two are orthogonal.
---

# 058: The defects heading has one spelling

## 1. Purpose

Constitution V gained its operational half in spec 043:

> Code adopted from outside the corpus is specced **as found**. The adopting
> spec describes the behavior that exists, and records the behavior it would not
> have chosen under a `## Known defects` heading, with the defect named.

That sentence closes a real dilemma. Without the heading, a spec adopting
foreign code must either describe it accurately, thereby ratifying its defects
as the specified behavior, or describe the intended behavior and ship a spec its
own coupling gate will contradict. The heading is the seam between "this is the
contract" and "this is what is there".

It is a load-bearing convention that no code has ever checked, and the first
consumer to check it got it wrong. claude-observatory's `defects.ts` matches the
literal string `## Known defects`. The specs it names as precedent write the
heading with a section number, `## 5. Known defects`, so the checker rejects its
own citations. That is not carelessness. It is what happens when a governance
document specifies a heading by quoting it, and a consumer has to guess whether
the quote is the exact form or an example.

The corpus already knows how to answer this, and has since spec 022. Section
units name a heading by its **anchor**, the slug the indexer computes:
alphanumerics lowercased, every other run collapsed to a single dash, ends
trimmed. `## Known defects` and `## Known Defects` and `### known defects` all
slug to `known-defects`. `## 5. Known defects` slugs to `5-known-defects`.
Specifying the heading as an anchor rather than a string makes the question
mechanical, and makes it the same question the section-unit resolver already
answers.

This is item 10 of the adopter audit's ranked backlog for the tool.

## 2. Territory

| Unit | Owner | What changes |
|---|---|---|
| `crates/spec-spine-core/src/lint.rs` | 003 | the `L-009` check |
| `standards/spec/constitution.md` §V | 043, refined here | the spelling, stated |

The constitution edit is a `refines` claim, with the named aspect
`defects-heading-anchor`, on the same section spec 043 refined under the aspect
`adopted-code-as-evidence`. It is not an `amends` edge: `amends` resolves to
spec ids and the constitution is not a spec. That is the mechanism spec 043
itself defined, and this spec is its second use. The file is on the coupling gate's built-in bypass
floor, so the claim is a ledger fact rather than a `C-001` refusal, exactly as
043 §3 describes.

## 3. Behavior

### 3.1 The heading is an anchor

A defects section is any heading in a `spec.md` whose computed anchor is
`known-defects` or ends with `-known-defects`.

The slug rule is the existing one in `sections.rs`, unchanged and not
reimplemented. That gives, for free:

- case insensitivity (`Known Defects`, `KNOWN DEFECTS`),
- any heading level (`##`, `###`),
- section numbering (`## 5. Known defects` slugs to `5-known-defects`, which
  ends with the anchor),
- punctuation tolerance (`## Known defects:` trims to `known-defects`).

And it excludes the things it should: `## Known defect` (singular) slugs to
`known-defect`, `## Defects` slugs to `defects`, and `## Known defects and open
questions` slugs to `known-defects-and-open-questions`, which does not end with
the anchor. The last is the interesting one and the rule is deliberate: a
heading that continues past the anchor is a section about something wider, and
a consumer extracting defects from it would extract the open questions too.

**Suffix, not prefix or substring.** A prefix rule would accept the
open-questions case. A substring rule would accept both that and
`## Why known defects matter`. The suffix rule accepts exactly the numbering and
prefixing that real specs use, which is the case that broke `defects.ts`.

The constitution's §V MUST state this, in one sentence, where it currently
quotes the heading. Quoting a heading and meaning an anchor is the defect; a
governance document that names the rule cannot be misread into a string match.

### 3.2 `L-009`

`lint` MUST emit one `L-009` diagnostic, at **info** severity, for each heading
in a `spec.md` whose anchor is a near miss for the defects anchor: it contains
`defect` and is not itself a defects anchor under §3.1.

```
L-009  spec '031-registry-freshness-check' has heading '## Known defect'
       (anchor 'known-defect'), which is not the defects anchor: a consumer
       looking for 'known-defects' will not find this section
```

**Info tier.** It is a spelling nudge on a section that is otherwise valid
prose, and info is the tier this corpus reserves for exactly that (`L-005`, the
stub check). It surfaces under `--fail-on-info`, which no gate here runs, so a
corpus is told without being refused.

**Decision, 2026-09-07: "contains `defect`" was too loose, and this spec's own
title proved it.** The first implementation followed the sentence above
literally and immediately reported
`# 058: The defects heading has one spelling`, whose anchor contains `defect`
and which is prose about defects rather than an attempt at a defects section.

The rule is narrowed to the two shapes that are attempts. An anchor **ending**
`defect` or `defects` is a section named for defects that got the spelling
wrong (`## Known defect`, `## Defects`). An anchor **containing**
`known-defects-` got the name right and kept going
(`## Known defects and open questions`), which §3.1 refuses for its own reason.
Both of §3.2's motivating cases survive; the title does not match either shape.

That a spec's title can trip its own lint is worth the two lines it costs to
record: the near-miss check reads every heading in a `spec.md`, and a spec whose
subject is a heading will always have that subject in its title.

The check reads `section_headings`, which `SpecRecord` already carries and
`compile` already populates. No new parsing, no new field, no new IO.

### 3.3 `origin.retroactive` does not imply the heading

There is deliberately **no** lint requiring a spec with `origin.retroactive: true`
to carry a defects section.

The audit's item 10 proposed one, and spec 043 already answered it in the
sentence that added the heading to the constitution: `origin.retroactive: true`
says *when* the authority began; `## Known defects` says *what* the adopting spec
makes of what it found. The two are orthogonal, and 043 wrote that down as a
conclusion, not as a hedge.

The counterexample is in this corpus. `specs/000-spec-spine-bootstrap` declares
`origin.retroactive: true` and has no defects section, and it is right not to:
it claims authority over a corpus convention that predates the graph, not over
foreign code with behavior it would not have chosen. A lint implementing the
audit's proposal would fire on the tier-1 bootstrap spec, which is both wrong
and unfixable, since spec 000's `unamendable` anchors are non-overridable.

Nothing in the frontmatter distinguishes "adopted foreign code" from "authority
that predates the graph", and inventing a key to carry that distinction would be
adding grammar to make a lint possible rather than because an author needs to
say it. So the corpus checks the spelling of the heading, which is mechanical,
and leaves the judgement of when one is owed to the constitution, which is where
judgement belongs.

### 3.4 Nothing else moves

No committed artifact changes, and no schema version moves. `compile --check`
stays fresh, `L-009` is info tier so `lint --fail-on-warn` is unaffected, and
the constitution is a hashed input under `standards/**`, so editing it restamps
the global scalar and therefore every shard. The implementing change regenerates
and commits them, as any edit under `standards/` requires.

## 4. Out of scope

**Requiring a defects section anywhere.** No spec is refused for lacking one.
Constitution V says when one is owed and that is a judgement a person makes about
code they adopted.

**Checking what is inside the section.** "With the defect named" is prose the
author writes and no lint can evaluate.

**A machine-readable defects list.** Extracting the section's contents into the
registry as structured data is a real idea, and it is the one that would let a
consumer stop reading markdown. It needs a grammar for a defect entry and a
schema version, and it should be designed after the heading has one spelling,
not at the same time.

**Fixing claude-observatory's `defects.ts`.** An adopter-side follow-up, recorded
in the audit's §5. This spec gives it the rule to implement.

## 5. Verification

`L-009` and the constitution's anchor sentence both fail against pre-058 state:
the code did not exist and §V quoted a heading.

```verify:cli
# Self-contained: the commands below invoke the release binary.
cargo build --release --locked
cargo test -p spec-spine-core --test lint --locked
# 3.1: the constitution states the anchor rule rather than quoting a heading.
grep -q 'known-defects' standards/spec/constitution.md
grep -q 'identified by its \*\*anchor\*\*' standards/spec/constitution.md
# 3.2: a near-miss heading is reported, at info tier.
tmp=$(mktemp -d) && mkdir -p "$tmp/specs/001-x" && : > "$tmp/spec-spine.toml" && printf -- '---\nid: "001-x"\ntitle: "x"\nstatus: draft\ncreated: "2026-09-07"\nsummary: "x"\nestablishes:\n  - "specs/001-x/spec.md"\n---\n\n# x\n\n## Known defect\n\nprose\n' > "$tmp/specs/001-x/spec.md" && target/release/spec-spine --repo "$tmp" lint | grep -q 'L-009'
# 3.2: and only under --fail-on-info. Without it the same corpus exits 0.
tmp=$(mktemp -d) && mkdir -p "$tmp/specs/001-x" && : > "$tmp/spec-spine.toml" && printf -- '---\nid: "001-x"\ntitle: "x"\nstatus: draft\ncreated: "2026-09-07"\nsummary: "x"\nestablishes:\n  - "specs/001-x/spec.md"\n---\n\n# x\n\n## Known defect\n\nprose\n' > "$tmp/specs/001-x/spec.md" && target/release/spec-spine --repo "$tmp" lint --fail-on-warn
# 3.2: this corpus is clean at info tier, which is the assertion the narrowed
# rule earns: before it, this spec's own title was reported.
target/release/spec-spine lint --fail-on-info
# The shards were committed.
target/release/spec-spine compile --check
target/release/spec-spine index check
```
