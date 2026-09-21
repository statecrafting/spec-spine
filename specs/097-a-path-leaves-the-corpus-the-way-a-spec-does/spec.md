---
id: "097-a-path-leaves-the-corpus-the-way-a-spec-does"
title: "A path leaves the corpus the way a spec does"
status: draft
kind: "tooling"
created: "2026-09-20"
summary: >
  Spec 096 mechanizes one retirement: a spec id leaving the corpus. On
  2026-09-20 a different one was done by hand, `.claude/rules/` leaving this
  repository for a section of `AGENTS.md`, and it touched 81 references in 46
  files, three frontmatter units, one hashed-input glob, one hook path pattern,
  six acceptance lines, three Rust tests and 122 derived shards. None of that is
  reachable by `compact`: its three reference forms are all spelled `NNN-slug`,
  and a path is spelled six other ways. This spec adds the second half of the
  same verb, a `retire` plan naming paths rather than specs, with the reference
  grammar a path actually has and, more importantly, the exclusions: eleven of
  those 81 occurrences had to survive, because a sentence recording that a path
  USED to exist is not a citation of it.
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "096-compaction-is-a-verb-not-a-session"
  - "050-claimed-but-unwitnessed"
  - "093-the-harness-this-repository-runs"
establishes:
  - { kind: file, path: "crates/spec-spine-core/tests/retire.rs" }
extends:
  - { spec: "096-compaction-is-a-verb-not-a-session", unit: { kind: file, path: "crates/spec-spine-core/src/compact.rs" }, nature: additive }
  - { spec: "096-compaction-is-a-verb-not-a-session", unit: { kind: file, path: "crates/spec-spine-cli/src/cmd_compact.rs" }, nature: additive }
  - { spec: "096-compaction-is-a-verb-not-a-session", unit: { kind: file, path: "crates/spec-spine-core/tests/compact.rs" }, nature: additive }
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-core/src/lib.rs" }, nature: additive }
  - { spec: "001-compile-registry", unit: { kind: file, path: "crates/spec-spine-cli/src/main.rs" }, nature: additive }
references:
  - { unit: { kind: file, path: "docs/design/08-remaining-cleanup-2026-09.md" }, role: context }
---

# 097: A path leaves the corpus the way a spec does

## 1. Purpose

### 1.1 The same operation, a different spelling

Spec 096 was filed because spec 095's renumber was done by hand and produced
five silent defects. It mechanizes the removal of a **spec id**: a plan in, a
rewritten corpus out, idempotent, reported per rewrite.

On 2026-09-20 this repository performed the other retirement. `.claude/rules/`
left, its four rules becoming `AGENTS.md`'s `## Rules` section (spec 093 D-4).
Measured on the change:

| What had to move | Count |
|---|---|
| references to the four files or the directory | 81, in 46 files |
| of those, references that had to be **left alone** | 11 |
| frontmatter units withdrawn or retargeted | 3, in 3 specs, 2 of them approved |
| `[index] extra_hashed_inputs` globs | 1 |
| path patterns inside a hook body | 1 |
| lines inside a `verify:cli` block | 6, in 3 specs |
| Rust tests reading the path | 3 |
| derived shards restaled | 122 |

`compact` reaches none of it. Its three forms (§3.3 there) are a full id, a
prose `spec NNN`, and a bare short id as a command argument. Every one is
spelled `NNN-slug` or three digits. A path is spelled as a backticked
citation, a bare word in a shell line, a glob, a YAML value, a `case` pattern
and a Rust string literal, and each of those needs a different rule.

### 1.2 Why this is not simply a rename

A rename has one output text for every input occurrence. A retirement does not.
The four rule files became **one section per file** of a different document, so
`.claude/rules/adversarial-prompt-refusal.md` became `` `AGENTS.md` "Adversarial
prompt refusal" `` while `.claude/rules/` as a directory became "`AGENTS.md`'s
`## Rules` section", and the glob `.claude/rules/*.md` became nothing at all: it
was deleted from the config, not rewritten.

So the plan carries a replacement **per form**, not per path, and a form with no
replacement is a deletion the plan states rather than a rewrite the tool infers.

### 1.3 The exclusions are the hard part

Eleven of the 81 occurrences were kept, and every one of them was kept for the
same reason: it is a sentence about the past or about an absence, not a pointer.

- `AGENTS.md` says the rules "were four files under `.claude/rules/` until spec
  093 folded them here". Rewriting that sentence makes it say the rules were
  four files under `AGENTS.md`.
- Spec 093 §4.10 says "`.claude/rules/` MUST NOT exist". That is the
  requirement; it names the path precisely because the path is gone.
- Spec 093's acceptance carries `! test -e .claude/rules`, an absence assertion,
  and spec 092's §6 drops table names `kit/.claude/rules/` as a subject that was
  deleted two specs ago.
- `scaffold.rs` and `tests/scaffold.rs` assert the governance scaffold emits
  **no** `.claude/rules/` files (spec 092 §3.3).

A tool that rewrote these would have produced a corpus that compiles, passes
every gate, and lies in five places. This is the same failure shape as spec
096's defect 5, where an absence assertion read a message with a path
interpolated into it, and it is why §3.5 below is normative rather than advisory.

## 2. Territory

| Path | What |
|---|---|
| `crates/spec-spine-core/tests/retire.rs` | the path forms and the exclusions, each with the case it prevents |

The plan type, the rewrite and the report are spec 096's: `compact.rs` gains the
`retire` section, `cmd_compact.rs` reads it, and the `Compaction` report carries
its rewrites in the shape §3.6 there already defines. Five `extends` edges name files this build
writes into and does not own: 096's `compact.rs` and `cmd_compact.rs`, its
`tests/compact.rs` (the plan type gained a field, so every literal that built
one changed), and 001's `lib.rs` and `main.rs` for the widened re-export and the
help text. They were added by this build rather than by the filing: `V-017` refuses an
`extends` onto a unit its owner has only `planned`, and 096 held them as planned
until it was built, so the edges could not be declared before the spec they
extend existed. This spec adds a second
kind of entry to one verb, not a second verb. The two retirements share the
plan file, the refusal set, the idempotence requirement and the per-form report,
and splitting them would mean maintaining two of each.

## 3. Behavior

### 3.1 The plan gains `retire`

```yaml
retire:
  - path: ".claude/rules/"
    kind: directory
    forms:
      citation: "`AGENTS.md`'s `## Rules` section"
      glob: remove
  - path: ".claude/rules/adversarial-prompt-refusal.md"
    kind: file
    forms:
      citation: '`AGENTS.md` "Adversarial prompt refusal"'
```

`compact` MUST refuse a `retire` entry whose `path` is not tracked in the tree
it is given, and MUST refuse an entry naming a form it has no rule for. A
missing form is a refusal and never a silent skip: a form that fired zero times
is spec 096 §3.6's defect 2, and a form that was never declared is the same
defect one step earlier.

Entries MUST be applied longest path first, so `.claude/rules/x.md` is rewritten
before `.claude/rules/` can match its prefix. This is spec 096 §3.3's
longest-key-first rule, restated because the shadowing here is between two
entries of the plan rather than between two keys of a computed map.

### 3.2 The six forms

A path reference is rewritten if and only if it is one of these:

1. **A backticked citation**, `` `<path>` `` in prose, replaced by
   `forms.citation`.
2. **A bare citation**, the path as a word in prose or in a comment, bounded by
   whitespace or by `.,;:)` . It MUST NOT match when the path is a substring of
   a longer path: `kit/.claude/rules/` is not `.claude/rules/`.
3. **A YAML scalar**, a `path:` value in a frontmatter unit, handled by §3.3.
4. **A glob**, the path followed by a wildcard segment, in a `spec-spine.toml`
   list. Replaced or removed per `forms.glob`.
5. **A shell word inside a `verify:cli` block**, located line by line per spec
   096 §3.5.
6. **A Rust string literal**, in an `include_str!`, a `Path::join` or a bare
   `&str`.

Anything else is left alone. In particular a path fragment inside a longer
token, a URL, and any occurrence inside a fenced block that is not `verify:cli`
are not references.

### 3.3 A withdrawn unit is named, never inferred

Where the retired path appears as a unit in a spec's frontmatter, `compact` MUST
NOT delete the unit unless the plan entry says so, per spec and per edge:

```yaml
    units:
      - { spec: "093-the-harness-this-repository-runs", edge: establishes, action: withdraw }
      - { spec: "045-couple-names-the-crossing", edge: references, action: retarget, to: "AGENTS.md" }
```

There is no grammar in this corpus for withdrawing a claim: no edge does it, and
the only route is an edit to the owning spec's frontmatter. That edit is exactly
the one an agent may not make on its own authority when the spec is `approved`,
so `compact` MUST report every unit it changed, with the owning spec's `status`
beside it, and MUST refuse the whole plan if a unit on an `approved` spec is
named without `acknowledge_approved: true` on that entry. The flag is a human's
to write, the way a `Spec-Drift-Waiver` is.

Retargeting a `references` unit is the cheap case and withdrawing an
`establishes` unit is not: the second changes what a spec owns. The tool
distinguishes them in the report and refuses neither differently, because the
judgement is the human's and the tool's job is to make it visible.

### 3.4 The hashed inputs follow the claim

If the retired path matches a `[index] extra_hashed_inputs` pattern, and the
pattern matches nothing else after the retirement, `compact` MUST remove the
pattern and MUST say in the report that it did. A glob left behind matching zero
files is spec 050's defect: it reads like a claim being hashed and hashes
nothing. `compact` MUST NOT remove a pattern that still matches another file.

The report MUST state that the change restales every shard, and `compact` MUST
NOT regenerate them: the derived tree is the caller's to rebuild and commit with
the change that staled it.

### 3.5 An absence is not a citation

`compact` MUST NOT rewrite an occurrence that is:

- inside a line matching a **negation**: `! test -e`, `! test -f`,
  `! grep`, `assert!(!`, or a `MUST NOT` sentence naming the path;
- inside a table row or paragraph under a heading the plan lists in
  `historical_sections`, which is how a spec's "what was dropped" table and a
  dated design note are protected;
- inside a file the plan lists in `historical_files`.

An occurrence skipped by this rule MUST appear in the report as skipped, with
which clause skipped it. Silence here is the failure: the eleven survivors of
the 2026-09-20 retirement were found by reading a `grep`, and a tool that keeps
them without saying so has only moved the reading.

### 3.6 Idempotent, and green is not enough

Applying the output to the output MUST produce no change, per spec 096 §3.4.

The report MUST carry, per retired path, the count of occurrences rewritten and
the count skipped, per form and per clause. A retirement that rewrote 69 and
skipped 11 is reviewable; a retirement that says "done" is not, and every defect
in this family has been silent under a green gate.

### 3.7 What it refuses

Exit 3, before writing anything:

- a plan naming a path the tree does not have;
- a plan naming a form with no rule, per §3.1;
- a unit on an `approved` spec without `acknowledge_approved: true`;
- a corpus that does not compile, per spec 096 §3.8.

Exit 1 after the rewrite, with the list: any occurrence of a retired path that
is neither rewritten nor skipped by a named clause. The tool has then found a
form it does not know, and reporting it is the only honest answer.

## 4. Out of scope

- **Deciding to retire a path.** Judgement, and in the 2026-09-20 case it took
  reading what loads a rule, what cites it, and what a global home would bind.
- **Writing the replacement text.** Authoring. The plan carries it.
- **Moving or deleting the files.** `git` does that; the engine is `git`-free
  and returns files as data (spec 001).
- **Retiring a spec id.** Spec 096.
- **A repository-wide rename.** A rename has one replacement for every
  occurrence and needs none of §3.1's per-form table; `sed` is the right tool
  and this verb would be a worse one.

## 5. Resolved decisions

D-1 (2026-09-20, the evidence is one measured retirement, not a survey). Every
count in §1.1 and every exclusion in §1.3 is from `.claude/rules/` leaving this
repository on that date, the second retirement in two weeks to be done by hand.
Spec 096 was filed off five measured defects; this one is filed off a
retirement that had none, because the four citation strings were full literal
paths and the eleven exclusions were caught by reading. Neither property is
guaranteed by anything: a path with a shorter name, or a corpus with more
occurrences than a person will read, gives the same operation both defect
families at once.

D-2 (2026-09-20, the frontmatter edit is the verb's, not the caller's). §3.3
says a withdrawal is named rather than inferred, and the first build validated
the naming and then made no edit: the plan passed, the acknowledgement was
recorded, and the spec went on claiming a path that was gone. The edit is made
here, line-scoped inside the frontmatter and inside the named edge's list, and a
withdrawal that empties a list takes the key with it. The acceptance asserts the
resulting bytes, not that the call returned `Ok`: permission to make an edit is
worth nothing if nothing checks the edit happened.

D-3 (2026-09-20, `_` and `-` are path characters). §3.2 form 2 excludes a bare
occurrence that is part of a longer path. The first boundary test asked only
`is_ascii_alphanumeric`, which let `_rules/one.md` through as a citation of
`rules/one.md`. Both characters are named explicitly now.

D-4 (2026-09-20, the two boundary tests are not the same test). A bare prose
occurrence is disqualified by a quote or a backtick, because the backticked
citation and the quoted `path` form are replaced before it runs and matching
them again rewrites one occurrence twice. A frontmatter unit is not: its value
IS quoted, so a quote there is a delimiter. Both read one `is_path_char`, and
the difference between them is stated where each is defined. Sharing a single
predicate was tried first and withdrew every unit in the corpus, because
`path: "rules/"` reads as disqualified.

D-5 (2026-09-20, a line one entry spares is spared, and what that costs).
A consequence worth stating rather than leaving to a reader of the tests: a
negation is a LINE-level marker, so on `! test -e rules/one.md && test -e
rules/two.md` the second path is spared and labelled `negation` even though the
`!` does not negate it. Sparing is per line because the report must describe the
file that is emitted, and the emitted line is whole. The clause is the best
available label rather than a claim about which marker applies to which path.

D-5a (2026-09-20, the original form). The skip loop advanced to
the next plan entry after recording a skip, so a line called left alone in the
report could still be rewritten by a later entry, and the report then described
a file that was not the one emitted. §3.5's "reported, never silent" is a claim
about the output, not about one entry's view of it.

D-6 (2026-09-20, the frontmatter edit runs before the form rewrite). A `path`
form rule and a unit action can name the same line. With the rewrite first, the
form replaces the value, the unit matcher then finds nothing, and the action
silently does not fire: a withdrawal leaves the claim standing and a retarget
writes a path the `to` never named. The order is fixed, and the acceptance
exercises a plan carrying both.

D-7 (2026-09-20, the acknowledgement rule reads the enum, not a formatted
string). The first build compared `format!("{:?}", status)` to `"approved"`.
`Debug` is not a stability contract, and the failure is silent in the permissive
direction: a rename would make every approved spec editable without the human
acknowledgement, which is the one thing §3.3 exists to demand.

D-8 (2026-09-20, a period is punctuation or an extension, and the rule has to
say which). §3.2 form 2 ends a bare citation at whitespace or sentence
punctuation, and `.` is both: `rules/one.md.` closes a sentence while
`rules/one.md.bak` is a different file. A period terminates a citation only when
nothing but space follows it. The first rule listed `.` unconditionally, which
contradicted the path-character test three lines above it.

D-9 (2026-09-20, §3.7's leftover refusal reads every file the rewrite READ).
The exit-1 rule was written into §3.7 at filing and not built: an occurrence no
rule covered stayed in place in silence, the plan looked applied, and the gate
was green over a corpus still naming a retired path. The scan reads the examined
set rather than the emitted one, because a file no rule touched is precisely
where such an occurrence hides and is not in the emitted set at all.

D-10 (2026-09-20, a spared line spares every occurrence on it, and reports each).
A line carrying two retired paths, one of them inside a negation, was spared as a
line and reported once. §3.5's "reported, never silent" is a claim about
occurrences: each one now gets its own record naming the clause that spared it.

D-11 (2026-09-20, a glob replacement is a directory prefix). The glob rule
substitutes the path prefix and leaves the wildcard, so `rules/*.md` with
`AGENTS.md` reads `AGENTS.md*.md`. A replacement that does not end in `/` is
refused; `~` still removes the pattern outright.

D-12 (2026-09-20, §3.7 reads the same boundary the rewrite reads). The leftover
scan used a raw substring test while the rewrite used §3.2's boundary rule, so
`kit/rules/one.md` was correctly left alone and then reported as an occurrence
no clause accounted for, refusing the whole run. Two readers of the same grammar
have to be the same reader.

D-13 (2026-09-20, an occurrence carries the clause that applies to it). A line
spared by one entry is spared for all of them, because the report has to
describe the file that is emitted. Labelling every occurrence on it with the
first entry's clause is accurate about the outcome and wrong about the reason,
so each entry's own clause is decided first and the line-level one is the
fallback.

D-14 (2026-09-20, an empty retired path is refused). `find("")` answers
`Some(0)` at every position and advances nothing, and the existence check passes
it because `repo_root.join("")` is the repository root. §3.7's refusal list is
the whole tree, or the scan never terminates; neither is an answer.

D-15 (2026-09-20, every unit action on a line fires). One line can name two
retired paths, which is what the `paths:` sugar writes, and stopping at the
first match dropped the second in silence. A withdrawal wins over a retarget on
the same line: the line is leaving, so there is nothing left to point elsewhere.

D-16 (2026-09-21, one boundary rule, two contexts, every reader through it).
Three rounds of review found the same defect three times: the rewrite, the
leftover scan and the clause computation each carried their own copy of §3.2's
boundary rule, and each fix repaired one copy. D-4, D-12 and the round that
bounded the clause computation are all the same finding. The rule is one
function now, taking the context that decides its delimiters: in prose a quote
means another form owns the occurrence, in a quoted value a quote IS the
delimiter, and both sides of the match are tested in both. §3.7 asks whether
EITHER context would have matched, because a scan narrower than the rules
refuses a corpus the rules were right to leave alone.

D-17 (2026-09-21, a skip record names its path). Two retired paths can share a
line, so a record carrying only the coordinate cannot say which occurrence it
spared, and §3.7 matching on the coordinate alone let one path's record account
for another path's occurrence.

D-18 (2026-09-21, the glob reader joins the convergence). D-16 converged four
readers on one boundary rule and missed a fifth: the glob branch tested a raw
substring, so `rules/*` matched inside `extra-rules/*.md`, and because that
branch returns early the citation rule that would have handled the line
correctly was never reached. The glob is located with the same rule as
everything else, and the replacement rewrites the occurrence found rather than
every substring match.

D-19 (2026-09-21, the sixth reader). D-16 named four readers of §3.2's
boundary rule and D-18 found a fifth; the retarget's `String::replace` was the
sixth, and it rewrote `rules/` inside `rules/one.md` when both sat on one
frontmatter line. Every replacement of a retired path now goes through one
function that takes the context and replaces only what that context recognises.
Six rounds of the same finding is the argument for the shared function, not an
argument that each reader was individually careless.

D-20 (2026-09-21, a retired path is inside the corpus, and a retarget names
something). `repo_root.join("/etc/passwd")` is `/etc/passwd` and
`repo_root.join("../x")` leaves the tree, so the existence check passed both:
nothing outside the corpus is ever written, because the walk is corpus-bounded,
but the string would then be matched across every scanned file. An absolute path
and a `..` component are refused. An empty `to` is refused with the absent one,
because both write a unit no corpus can resolve.

D-21 (2026-09-21, a directory entry ends in a slash, and it is enforced). Without
the trailing slash a directory path is a WORD, and §3.2's boundary rule finds it
in any sentence: `rules` matches the English word, §3.7 reports every such line
as an unaccounted occurrence, and the run is refused over prose. Every plan
written so far spelled the slash by convention; convention is not a rule.

D-22 (2026-09-21, an entry acts on the source line, not on what another entry
wrote). One entry's replacement text can contain another entry's retired path.
Acting on that occurrence rewrites something no plan named, and sparing it
records a clause that was never true of the file. Participation is decided
against the line as it arrived.

D-23 (2026-09-21, the detector and the rewriter recognise the same occurrence).
§3.7 reported `[rules/one.md]` as unaccounted for and no form rewrote it, so the
run refused a corpus no rule could have repaired. Every earlier round of this
family was two readers disagreeing about a match; this is the scan being broader
than the rules it scans for. A markdown link label is a citation, so the prose
context delimits on brackets and the citation form covers it.

D-24 (2026-09-21, a plan writes the path the corpus writes). `./rules/one.md`
passes every guard and matches nothing, because the literal carries the prefix
and the corpus spells the path bare: the run would rewrite nothing and then
report every bare occurrence as unaccounted for. Refused, naming the prefix.

## Verification

Each line is one command, run independently.

**Fail-first evidence**, measured on 2026-09-20 on this branch:
`crates/spec-spine-core/tests/retire.rs` does not exist;
`target/release/spec-spine compact --help` is red (no such subcommand, exit 2),
so every line below that names the verb is red before the build.

```verify:cli
cargo build --release --locked
test -f crates/spec-spine-core/tests/retire.rs
target/release/spec-spine compact --help > "${TMPDIR:-/tmp}/ss097-help.txt" 2>&1
grep -qF 'retire' "${TMPDIR:-/tmp}/ss097-help.txt"
rm -f "${TMPDIR:-/tmp}/ss097-help.txt"
cargo test -p spec-spine-core --test retire --locked > "${TMPDIR:-/tmp}/ss097-t.txt" 2>&1
grep -qE 'test result: ok\. [1-9][0-9]* passed' "${TMPDIR:-/tmp}/ss097-t.txt"
rm -f "${TMPDIR:-/tmp}/ss097-t.txt"
# 3.3: a unit on an approved spec is refused without the human acknowledgement.
grep -qF 'acknowledge_approved' crates/spec-spine-core/src/compact.rs
# 3.5: the exclusions exist as clauses, not as prose.
grep -qF 'historical_sections' crates/spec-spine-core/src/compact.rs
grep -qF 'historical_files' crates/spec-spine-core/src/compact.rs
# 1.3, still true of the retirement this spec was measured on: the eleven
# survivors are still there, and the path is still gone.
! test -e .claude/rules
grep -qF 'MUST NOT exist' specs/093-the-harness-this-repository-runs/spec.md
# The governed loop, over the corpus this spec is part of.
target/release/spec-spine check --fail-on-unresolved --fail-on-warn
target/release/spec-spine lint --fail-on-warn
```
