---
id: "066-the-contract-records-the-lifecycle-table"
title: "The contract records the lifecycle table and the extra keys"
status: approved
kind: "governance"
created: "2026-09-07"
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "041-completion-held-to-claims"
  - "043-governance-document-gaps"
  - "044-in-progress-is-in-flight"
  - "045-absent-implementation-defers-to-status"
extends:
  - { spec: "006-init-scaffold", unit: "crates/spec-spine-core/src/scaffold.rs", nature: additive }
  - { spec: "006-init-scaffold", unit: "crates/spec-spine-core/tests/scaffold.rs", nature: additive }
  # The two sections are added to this repository's own contract as well, whose
  # section units spec 043 established are claimed rather than amended.
  - { spec: "043-governance-document-gaps", unit: "standards/spec/contract.md", nature: additive }
references:
  - { unit: { kind: file, path: "docs/design/03-adopter-audit-2026-09.md" }, role: context }
  - { unit: { kind: file, path: "standards/spec/contract.md" }, role: context }
summary: >
  `status` and `implementation` are the two keys every adopter reasons about
  daily, and the contract mentions `status` once, in a list of required
  frontmatter, and `implementation` not at all. The mechanical consequences are
  spread across four specs: 041 makes a `complete` claim checkable, 044 defines
  the in-flight window, 045 says an absent key takes its answer from `status`,
  and 038 reads both to build the ready set. Every specify-first adopter wrote
  the resulting table into its own contract by hand, and hqgit had to invent a
  section called "Lifecycle as scheduling" because ours has none. This spec adds
  that section and an "Extra keys" section telling adopters to document their
  `extra_known_keys`, and claims both as section units of the contract.
---

# 066: The contract records the lifecycle table and the extra keys

## 1. Purpose

`standards/spec/contract.md` is the one-page operational summary an adopter
reads to learn what a spec is. It covers inputs, outputs, required frontmatter,
the eight edges, amendment authoring, amending the constitution, authority
units, the gate chain and determinism. It is a good page.

It says nothing about the lifecycle.

`status` appears once, inside the required-frontmatter list, as four permitted
values. `implementation` does not appear at all. Those two keys are what every
adopter reasons about every day, and their meaning is now mechanical and spread
across four specs:

- **041** holds a `complete` claim to its claims: a spec asserting completion
  whose units do not resolve is an error, not a warning.
- **044** defines the in-flight window (`draft`, or `pending` / `in-progress`,
  and not `complete`) and makes an unresolved unit inside it a `W-001` warning
  rather than a refusal.
- **045** settles the absent key: it takes its answer from `status`, reading as
  `pending` on a draft and as settled on anything ratified.
- **038** reads both to partition the corpus into ready and blocked.

Four specs, one table, and the table exists nowhere. So adopters wrote it.
Every specify-first repository the audit examined has a `status` by
`implementation` severity table in its own contract, and hqgit went further,
adding a section titled **Lifecycle as scheduling** stating that `approved` plus
`pending` is a work order, that `draft` is never schedulable, and that
`complete` and `n-a` count as shipped. Spec 043 quoted that section and said the
reason it exists is that ours does not.

The second, smaller gap is `frontmatter.extra_known_keys`. It is how an adopter
adds a key of their own without tripping the unknown-key diagnostic, and the
contract never mentions it, so the audit found adopters either not knowing it
exists or using it without recording what their keys mean. A key declared in
config and documented nowhere is a private convention that the next person on
the project has to reverse-engineer from a TOML list.

This is item 7 of the adopter audit's ranked backlog for the kit and the
scaffold.

## 2. Territory

| Unit | Owner | What changes |
|---|---|---|
| `standards/spec/contract.md` §Lifecycle as scheduling | this spec | new section |
| `standards/spec/contract.md` §Extra keys | this spec | new section |
| `crates/spec-spine-core/src/scaffold.rs` | 006 | the scaffolded `CONTRACT` |

Both contract sections are `establishes`, not `refines`: they are text that does
not exist, added to a tier-3 document, which is the ordinary ownership
vocabulary spec 043 §3 describes for a governance file. The contract is on the
coupling gate's built-in bypass floor as part of `standards/`, so these are
ledger facts rather than gate-enforced ones.

Nothing in specs 038, 041, 044 or 045 is edited. This spec states in one place
what those four decided; it decides nothing itself, and where this text and one
of those specs disagree, the spec governs.

## 3. Behavior

### 3.1 Lifecycle as scheduling

The contract MUST gain a section, anchored `lifecycle-as-scheduling`, stating
the two keys, their values, and the joint table:

| `status` | `implementation` | schedulable | unresolved unit is |
|---|---|---|---|
| `draft` | absent, `pending`, `in-progress` | yes | `W-001` warning |
| `approved` | `pending`, `in-progress` | yes | `W-001` warning |
| `approved` | absent | no (settled, spec 045) | error |
| any | `complete` | no | error (spec 041) |
| any | `n-a`, `deferred` | no | takes its answer from `status` |
| `superseded`, `retired` | any | no | takes its answer from `status` |

and the three sentences that make it usable:

- **`approved` plus `pending` is a work order.** It is the state a specify-first
  corpus lives in for months, and it is the state `registry plan` offers.
- **`draft` is never a claim about code.** A draft's unresolved units are
  expected, which is why they warn instead of refusing.
- **An absent `implementation` is not a third value.** It defers to `status`,
  which is what keeps a bootstrap spec that owns no code from being offered as
  ready forever (spec 045), and which is why `n-a` exists for a record spec that
  is ratified and owns nothing.

The section MUST name the spec behind each row, so a reader who needs the full
argument knows where it is and so this page stays a summary rather than becoming
a fifth authority.

### 3.2 Extra keys

The contract MUST gain a section, anchored `extra-keys`, stating that
`frontmatter.extra_known_keys` declares adopter-specific frontmatter keys, that
a declared key's value is preserved verbatim into the registry (spec 013), and
that an adopter who declares keys **should document what they mean**, in their
own constitution or contract, because the config lists the names and nothing
records the semantics.

Short, three or four sentences. The point is discoverability: the key exists,
here is what it does, write down what yours mean.

### 3.3 The scaffold ships both

`scaffold.rs`'s `CONTRACT` constant MUST gain both sections, so a new adopter
gets them rather than writing them.

This is the pattern spec 043 §3.4 established when it added the amendment
mechanism to the scaffolded `CONSTITUTION` and one line to `CONTRACT`. The
scaffolded copy is terser than this repository's own, as it already is: an
adopter needs the table and the three sentences, not the citations to specs they
do not have.

The generator stays a pure function of `Config` with no IO, and the scaffolded
corpus MUST still compile and lint clean.

### 3.4 The lifecycle table is a summary, not a fifth authority

The contract's own preamble already says it: the bootstrap spec and the
constitution are authoritative, and where this summary is terser, they govern.
The new section MUST carry the same disclaimer inline, because a table is
exactly the shape of thing a reader will treat as the definition.

The risk is concrete. If 045's rule for an absent key were ever revised, a table
in a summary document would be the last place anyone looked, and a stale table is
worse than no table because it is confidently wrong. Naming the owning spec per
row is the mitigation and it is why §3.1 requires it.

### 3.5 Nothing mechanical changes

No code path reads the contract. No committed artifact changes shape, no schema
version moves, and no lint or gate behaves differently.

`standards/**` is a hashed input, so editing the contract restamps the global
scalar and therefore every index shard. The implementing change regenerates and
commits them, as any edit under `standards/` requires.

## 4. Out of scope

**Changing any lifecycle rule.** §2. This spec is a transcription with
citations.

**A fifth `implementation` value.** The five are `pending`, `in-progress`,
`complete`, `n-a`, `deferred`, and nothing here proposes a sixth.

**Enforcing that adopters document their extra keys.** §3.2 says they should.
A lint for it would be a lint on prose.

**Restructuring the contract.** Two sections added in the register of the rest
of the page.

## 5. Verification

Each line is one command (spec 049 §3.2). Both sections fail against pre-066
state: neither existed, in this repository's contract or the scaffolded one.

```verify:cli
# Self-contained: the commands below invoke the release binary.
cargo build --release --locked
cargo test -p spec-spine-core --test scaffold --locked
# 3.1 + 3.2: this repository's contract carries both sections.
grep -q '## Lifecycle as scheduling' standards/spec/contract.md
grep -q '## Extra keys' standards/spec/contract.md
# 3.1: and the row a specify-first corpus lives in for months.
grep -q 'approved. | .pending., .in-progress. | yes' standards/spec/contract.md
# 3.3: a new adopter gets them rather than writing them.
rm -rf "${TMPDIR:-/tmp}/ss066" && mkdir -p "${TMPDIR:-/tmp}/ss066" && target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss066" init >/dev/null
grep -q 'Lifecycle as scheduling' "${TMPDIR:-/tmp}/ss066/standards/spec/contract.md"
grep -q 'Extra keys' "${TMPDIR:-/tmp}/ss066/standards/spec/contract.md"
# 3.4: and the scaffolded corpus still compiles and lints clean.
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss066" compile >/dev/null
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss066" lint --fail-on-warn
rm -rf "${TMPDIR:-/tmp}/ss066"
# The ledger is untouched by a documentation change to a bypassed path.
target/release/spec-spine compile --check
```
