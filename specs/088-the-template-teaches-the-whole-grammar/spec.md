---
id: "088-the-template-teaches-the-whole-grammar"
title: "The template teaches the whole grammar"
status: approved
kind: "governance"
created: "2026-09-17"
summary: >
  `standards/spec/templates/spec-template.md` is the file `/spec` copies to
  start every spec, and it is the only place in the corpus where the frontmatter
  grammar is written down for an author. It has fallen behind the parser. Spec
  082 added `amends_verification` and the template never gained it, a gap five
  specs have now recorded in their own section 4 with nobody able to close it,
  because no spec claims the file. Measured against `KNOWN_KEYS`, four more keys
  are missing for the same reason, and the `extends` example omits `nature`,
  which 96 of the 111 specs in this corpus write. This spec claims the template,
  completes it, and gives it an acceptance that reads the parser's own key list,
  so the next key added to the grammar cannot reach `main` without the document
  that teaches it.
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "043-verify-declared-acceptance"
  - "063-planned-territory-is-declared-not-inferred"
  - "082-an-amended-acceptance-is-the-one-that-runs"
establishes:
  # 3.1, 3.2, 3.3, 3.4: the authoring template. Unowned until now.
  - "standards/spec/templates/spec-template.md"
extends:
  # 3.5: the dogfood test that holds the corpus to the grammar this crate
  # defines. Owned by spec 000 by package floor and a `// Spec:` header; this
  # adds a case to it and claims nothing else in the file.
  - { spec: "000-spec-spine-bootstrap", unit: "crates/spec-spine-types/tests/dogfood.rs", nature: additive }
references:
  - { unit: { kind: file, path: "crates/spec-spine-types/src/frontmatter.rs" }, role: "context" }
  - { unit: { kind: file, path: "docs/design/02-agentic-builder-substrate.md" }, role: "context" }
---
# 088: The template teaches the whole grammar

## 1. Purpose

### 1.1 A gap five specs recorded and none could close

Specs 084, 107, 108, 109 and 110 each carry, in their own `## 4. Out of scope`,
the same sentence:

> `standards/spec/templates/spec-template.md` does not carry the key spec 082
> added. 084 §4 names it as a real gap in territory neither spec owns.

Spec 093 §4 adds that with the acceptance repairs done, this is "the one item of
the audit left standing". Five specs saw the gap, and each correctly declined to
fix it, because a spec may claim territory and none of them claimed this file.

Measured at `2a3e0e7`:

```
$ spec-spine index owner standards/spec/templates/spec-template.md
standards/spec/templates/spec-template.md
  (no spec owns this path)
```

The file is nonetheless hashed: it matches `standards/**/*.md` in
`[index] extra_hashed_inputs`, so its bytes are folded into the global-inputs
scalar and an edit stales every shard. It is a governed byte with no governor,
which is the combination that lets a document drift while the ledger keeps
signing it.

### 1.2 The gap is five keys wide, not one

`amends_verification` is the reported instance. It is not the only one. The
parser's own key list, `spec_spine_types::KNOWN_KEYS`, has 29 entries. Measured
against the template at `2a3e0e7`, five are absent:

| Key | Status in the tool | In the template |
| --- | --- | --- |
| `code_aliases` | parses, compiles to the registry, no verb reads it | absent |
| `feature_branch` | parses, compiles to the registry, no verb reads it | absent |
| `amends_verification` | spec 082: `verify` resolves through it | absent |
| `amendment_record` | `couple.rs` adds it to the amended spec.md's owner set | absent |
| `origin` | the retroactive bootstrap marker; spec 000 declares it | absent |

Two nested forms are missing as well, and they are not obscure:

- `nature` on an `extends` edge. Ninety-six of this corpus's 111 specs write it.
  The template's `extends` example does not.
- `planned: true` on a unit (spec 063), written by twelve specs. The template
  has no way to say "I own this and have not written it yet", which is the
  declaration that lets a spec mid-build coexist with
  `check --fail-on-unresolved`.

Neither `standards/spec/contract.md` nor `standards/spec/constitution.md`
mentions any of the seven. The template is the only document an author has, and
on these it is silent.

### 1.3 Why adding one key leaves the mechanism in place

The template fell behind because nothing connects it to the grammar. Spec 082
added a key to `KNOWN_KEYS`, to the DTO, to the schema's tolerance, to
`verify`'s resolution and to two test suites, and the gate was green, because no
check anywhere reads the template. Adding `amends_verification` by hand and
stopping there leaves that mechanism exactly as it was: the next key lands the
same way, and a sixth spec writes the same §4 sentence.

So the repair has two halves. The document is completed (§3.1 to §3.4), and the
parser's key list becomes the thing the document is checked against (§3.5).

## 2. Territory

This spec establishes `standards/spec/templates/spec-template.md`, which no spec
owned. It extends spec 000's `crates/spec-spine-types/tests/dogfood.rs` with one
additive test case; spec 000's file is not edited and its `unamendable` anchors
are untouched.

It references `crates/spec-spine-types/src/frontmatter.rs` as the authority the
template is checked against, and does not claim it: the direction of this spec is
that the grammar leads and the document follows.

## 3. Behavior

### 3.1 The template documents `amends_verification`

The template MUST show the key's syntax alongside the `amends` entry it
requires:

```yaml
# amends: ["NNN-predecessor"]
# amends_verification: ["NNN-predecessor"]
```

and MUST carry the guidance an author needs to use it correctly, which is not
derivable from the syntax. Specifically it MUST state:

- **When to reach for it.** An approved spec's acceptance line has gone wrong:
  it asserts more than the spec requires, or it pinned an output a later
  approved spec legitimately moved. Spec 037 forbids editing the amended file
  and `verify` executes that file, so a replacement declared here is the only
  route to a red block on an approved spec.
- **That every entry MUST also appear in `amends`, and that a missing one is
  `V-018`** (spec 082 §3.1).
- **That the corpus refuses an ambiguous replacement rather than guessing**:
  two live specs naming the same id is `V-019` and a cycle is `V-020` (spec 082
  §3.3).
- **That resolution follows the chain and skips a `superseded` or `retired`
  amender** (spec 082 §3.2).
- **That the substitution is announced**: `verify` prints whose block it ran,
  and `registry show <id> --json` carries `amendsVerification` (spec 082 §3.4).
- **That the replacement carries the amended block in full with the defect
  corrected, not a patch** (spec 082 §3.5).
- **That the replacement should assert that the amended file still carries the
  superseded form**, so the line goes red if anyone ever resolves it by editing
  the amended spec instead. This is the practice specs 085 to 110 each adopted
  and none of them could write down anywhere an author would find it.

### 3.2 The template documents every key the parser accepts

Every entry of `spec_spine_types::KNOWN_KEYS` MUST appear in the template as a
key at the start of a line, allowing for indentation and a leading `# `.

`code_aliases` and `feature_branch` MUST be documented **as inert**: they parse,
they validate, they compile to the registry record, and no verb reads either.
Documenting them silently as ordinary authoring keys would teach an author to
write a key that does nothing. Documenting them with what they actually are
answers the question a reader of a registry shard will have, and keeps the check
of this section total rather than carrying a hand-maintained exception list.
`docs/design/02-agentic-builder-substrate.md` records that giving these two
meaning is a deliberate decision rather than an accretion; naming them inert is
consistent with that and does not make it.

### 3.3 The template shows the nested forms the corpus writes

The `extends` example MUST carry `nature`. The unit examples MUST include one
with `planned: true`, with the sentence that makes it safe to use: an unresolved
planned unit raises no `W-001`, while one that is merely wrong still does.

The `establishes` examples MUST cover all six unit granularities (file, section,
symbol, directory, crate, module), the bare-string shorthand, and the
trailing-slash subtree form.

### 3.4 The template carries the sections a spec is expected to have

The body MUST carry `## 5. Resolved decisions` and a `## Verification` block
with a `verify:cli` fence. The template stopped at `## 4. Out of scope`, so
neither the section that records a mid-build decision nor the block
`spec-spine verify` actually runs appeared in the document an author starts
from.

The `## Verification` guidance MUST state that a line should fail against the
tree the spec is built on: a block that is green before the work asserts
nothing.

### 3.5 The parser's key list is what the template is checked against

`crates/spec-spine-types/tests/dogfood.rs` MUST assert §3.2 by iterating
`KNOWN_KEYS` itself rather than a list transcribed into the test. A transcribed
list is a third copy of the grammar and would go stale in the same way the
template did.

It MUST also assert the `amends_verification` guidance of §3.1 and the nested
forms of §3.3, because a key name alone satisfies §3.2 while teaching nothing.

The iteration MUST be guarded against vacuity. An empty `KNOWN_KEYS` yields an
empty missing-set and a green assertion that examined nothing, which is the
silent-tripwire shape this spec exists to remove. The guard MUST name the five
keys §1.2 measured as absent and assert each is still in `KNOWN_KEYS`, so the
check fails when its own subject moves.

This is a Rust test rather than only a `## Verification` line because
`cargo test --workspace` is in the gate chain and `verify` deliberately is not
(spec 043). A check that runs only under `verify` is a check nothing reruns after
merge, which is the failure mode the acceptance audit of specs 084 to 110 spent
nine PRs on.

## 4. Out of scope

- **The adopter scaffold template.** `scaffold.rs::spec_template()` emits a
  separate, deliberately minimal starter for `spec-spine init`, and it documents
  one edge of the eight on purpose. It is spec 095's territory, a different
  audience, and completing it is a different decision. §3.5's test reads this
  repository's template only.
- **`standards/spec/contract.md` and the constitution.** Neither mentions any of
  the seven forms §1.2 measures. They are owned (040, 043, 066), and extending
  the normative summary is a claim on those specs' subject matter rather than a
  repair of the authoring document.
- **Giving `code_aliases` and `feature_branch` meaning.** §3.2 documents what
  they are today. Making a verb read either is the deliberate decision
  `docs/design/02-agentic-builder-substrate.md` says it should be, and it is not
  this spec's.
- **The green literal pins in specs 047's and 105's blocks.** Named as standing
  in spec 084 §4 and unchanged here: each is another spec's acceptance, and a
  green line is not this spec's to change.
- **Rerunning a merged acceptance block.** §3.5 puts this spec's own check in
  the gate, which fixes the recurrence for this document and for nothing else.
  Spec 083 §4 declined to move `verify` into the gate chain and recommended a
  maintainer sweep; where that sweep is declared is still unfiled and is not
  resolved here.

## 5. Resolved decisions

D-1 (2026-09-17, why the whole key list and not `amends_verification` alone).
The reported gap is one key and the measured gap is five, plus two nested forms
that 96 and 12 specs respectively write. Adding the reported key and shipping a
document known to misdescribe the grammar in six more places is not a smaller
change, it is the same change with a worse result, and it forecloses §3.5: a
check that iterates `KNOWN_KEYS` cannot be written while four of its entries are
absent, so the narrow repair would also have to be a verify-only grep for one
string, which is the instrument §3.5 exists to avoid. The wider scope is the
same file, the same defect and the same session.

D-2 (2026-09-17, why the body sections came along). §3.4 is not frontmatter and
so is outside §1.2's measurement. It is in for the same reason: the template is
what `/spec` copies, every spec since 048 carries a `## 5`, and `spec-spine
verify` runs a block the template never mentioned. Leaving it would mean filing
a further spec against the same file for the same class of omission. It is
recorded separately from D-1 because the evidence differs: §1.2 is a count
against `KNOWN_KEYS`, §3.4 is a convention read off the corpus.

D-3 (2026-09-17, why the two inert keys are documented rather than excluded).
The alternative is an exception list in the test, `KNOWN_KEYS` minus
`code_aliases` and `feature_branch`. That list is a place for a future key to
hide: an author adding a key and finding the test red could add it to the
exclusion instead of the template, and the test would stay green while the
document stayed wrong. Documenting them as inert costs four lines, leaves the
check total, and tells a reader of a registry shard what those two members are.

D-4 (2026-09-17, why `extends` on spec 000 and not a new test file). A new file
would be claimed by `establishes` and need no edge, which is cheaper to declare
and worse to live with: the assertion belongs beside
`bootstrap_spec_000_parses`, which already reads a repository document and holds
it to this crate's grammar. `dogfood.rs` is owned by 000 through the package
floor and a `// Spec:` header, and an `extends` edge carrying that unit is a
first-class claim that amends nobody (`AGENTS.md` "Adversarial prompt refusal").
Spec 000's file is not edited; seven specs already extend the sibling
`grammar.rs` the same way.

D-5 (2026-09-17, why the template is checked by substring and not by parsing it).
The template's frontmatter is mostly comments, so it does not round-trip through
`parse_frontmatter`: parsing it would assert only the six live keys and say
nothing about the 23 commented ones, which are the entire subject of §3.2. The
check therefore reads lines, stripping indentation and one leading `# `, and
requires `<key>:` at the start of what remains. That anchoring is what keeps it
from passing on prose: `kind` appears inside `{ kind: file, path: ... }` on six
example lines, and none of them satisfies the test.

D-6 (2026-09-17, why the non-vacuity guard names keys and does not pin a count).
Review proposed `assert!(KNOWN_KEYS.len() >= 29)` ahead of the loop. That reads
on the correct defect and fixes it the wrong way: it is a literal pin on a list
designed to grow, and it goes red on a legitimate retirement of any key, which
is precisely the shape specs 085 and 108 were filed to repair. Naming the five
keys of §1.2 costs the same lines, cannot fire on an unrelated change, and says
what it is protecting. The vacuous case was already unreachable in silence
(`bootstrap_spec_000_parses`, in this same file, asserts `extra_frontmatter` is
empty, which an emptied `KNOWN_KEYS` would break loudly) but an assertion whose
own tripwire depends on a second test two functions away is not one a reader can
check, and this spec's subject is checks that can fail.

## Verification

Each line is one command, run independently: no shell variable survives to the
next line.

**Fail-first evidence.** At the parent commit every line below that reads the
template is red, because the template carries none of these strings; the
`dogfood` line is red because the test does not exist, and a name filter that
matches nothing exits 0, so that line is guarded by the pass-count grep rather
than by the exit code (spec 084 D-7).

```verify:cli
cargo build --release --locked
# 3.5: the key list the template is checked against is the parser's own, and
# the test that reads it is this spec's acceptance in the gate chain. The run
# is captured and its summary asserted to name a non-zero pass count: a filter
# matching nothing exits 0 and would leave this line green while asserting
# nothing.
cargo test -p spec-spine-types --test dogfood --locked authoring_template > "${TMPDIR:-/tmp}/ss111-dogfood.txt" 2>&1
grep -qE 'test result: ok\. [1-9][0-9]* passed' "${TMPDIR:-/tmp}/ss111-dogfood.txt"
rm -f "${TMPDIR:-/tmp}/ss111-dogfood.txt"
# 3.5: the guard that keeps the iteration above from passing vacuously. Its
# subject is named, so it fires when a key of §1.2 leaves the grammar rather
# than when the list merely changes size (D-6).
grep -qF 'left KNOWN_KEYS; this test' crates/spec-spine-types/tests/dogfood.rs
# 3.1: the syntax, shown with the `amends` entry it requires.
grep -qF '# amends_verification: ["NNN-predecessor"]' standards/spec/templates/spec-template.md
# 3.1: the guidance, clause by clause. A key name alone teaches nothing.
grep -qF 'Every entry MUST also appear in `amends` (`V-018`)' standards/spec/templates/spec-template.md
grep -qF 'V-019' standards/spec/templates/spec-template.md
grep -qF 'V-019' standards/spec/templates/spec-template.md
grep -qF 'skips a `superseded` or `retired`' standards/spec/templates/spec-template.md
grep -qF "prints which spec's block it ran" standards/spec/templates/spec-template.md
grep -qF 'amendsVerification' standards/spec/templates/spec-template.md
grep -qF 'in FULL with the defect corrected, not a patch' standards/spec/templates/spec-template.md
grep -qF 'the amended file still carries the' standards/spec/templates/spec-template.md
# 3.2: the four other keys the parser accepts and the template had lost, and
# the sentence that marks the two inert ones as inert.
grep -qF '# code_aliases:' standards/spec/templates/spec-template.md
grep -qF '# feature_branch:' standards/spec/templates/spec-template.md
grep -qF '# amendment_record:' standards/spec/templates/spec-template.md
grep -qF '# origin:' standards/spec/templates/spec-template.md
grep -qF 'read by nothing today' standards/spec/templates/spec-template.md
# 3.3: the nested forms 96 and 12 specs write.
grep -qF 'nature: additive' standards/spec/templates/spec-template.md
grep -qF 'planned: true' standards/spec/templates/spec-template.md
grep -qF 'raises no `W-001`' standards/spec/templates/spec-template.md
# 3.3: all six unit granularities.
grep -qF 'kind: directory' standards/spec/templates/spec-template.md
grep -qF 'kind: crate' standards/spec/templates/spec-template.md
grep -qF 'kind: module' standards/spec/templates/spec-template.md
# 3.4: the two sections the template stopped short of.
grep -qF '## 5. Resolved decisions' standards/spec/templates/spec-template.md
grep -qF '```verify:cli' standards/spec/templates/spec-template.md
grep -qF 'green before the work asserts' standards/spec/templates/spec-template.md
# The claim is declared, read through the CLI rather than off the shard.
# Redirected, not piped: a failing verb prints nothing and a pipeline would
# report that as a JSON decode error naming the wrong defect (spec 085 D-4).
target/release/spec-spine registry show 088 --json > "${TMPDIR:-/tmp}/ss111-show.json"
python3 -c "import json; d=json.load(open('${TMPDIR:-/tmp}/ss111-show.json')); assert 'standards/spec/templates/spec-template.md' in json.dumps(d['establishes']), d['establishes']"
rm -f "${TMPDIR:-/tmp}/ss111-show.json"
# The ownership answer the five prior specs could not get. Exit 0 with the id
# named is the difference this spec makes to `index owner`.
target/release/spec-spine index owner standards/spec/templates/spec-template.md > "${TMPDIR:-/tmp}/ss111-owner.txt"
grep -qF '088-the-template-teaches-the-whole-grammar' "${TMPDIR:-/tmp}/ss111-owner.txt"
rm -f "${TMPDIR:-/tmp}/ss111-owner.txt"
```
