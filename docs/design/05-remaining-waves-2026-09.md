# 05: The remaining waves (2026-09-14)

A design note, not a spec. Every backlog this repository keeps in
`docs/design/` has now been filed out except note 04's forward plan, and four
findings live only in an agent's memory or in a review thread. This note is one
record of what is left, in the order it should land, so the next session starts
from the finding rather than from a re-audit.

Wave A is filed alongside this note as drafts 092 to 096. Waves B, C and D are
proposals; nothing in them is filed, and §8 lists what a human must decide
before any of them can be approved.

Every measurement below was taken on 2026-09-14 against `709b620` with a
`0.19.0` binary built from that commit. Where a number contradicts an earlier
note, this one is the later measurement.

> **Reconciliation, 2026-09-15 (`3bc004b`).** Wave A shipped: 092 to 097 are
> approved and complete. The sections below are preserved as the dated
> measurements that produced them, and **§9 is the current record**: it corrects
> what this note got wrong, accounts for every item against one table, and
> disposes of the two findings routed here from aicortex. A new sibling,
> [note 06](06-harness-and-distribution-2026-09.md), records the harness and
> distribution redesign that none of waves A to D reaches. Read §9 first; read
> §2 to §5 for how each item was measured.

## 1. What the other notes still owe

| Note | Disposition |
|---|---|
| 01 partial supersession | Shipped as 019. Owes nothing. |
| 02 agentic-builder substrate | Waves 1 and 2 shipped (037 to 042). Wave 3's G5 landed as 061, 065 and 081; G8 (redaction) is answered by 042. Wave 4 (G7, the coherence gate) is deliberately unfiled, and note 04 D-4 re-affirms that: the naive rule refuses legitimate refactors, so no mechanism is asserted before it is designed. |
| 03 adopter audit | Both backlogs filed as 045 to 068 (note 03 §7 records the mapping). §5 is adopter-side work and is not this repository's. |
| 04 authority evidence | P0 is shipped, 087 included (built #208, ratified #212 on 2026-09-15). **P1 and P2 are entirely unfiled.** Findings F8 and F9 were recorded there as "small fix, unfiled"; they shipped as 093 and 096. D7 is decided by 093. Note 04's own lifecycle labels were refreshed on 2026-09-15; its measurements are preserved with links to the specs that resolved them. |

So note 04 is the only note with an unfiled forward plan, and the unfiled pool
splits into two waves that need no new record vocabulary (A and B) and two that
build on 087's framed digest (C and D).

**Corrected 2026-09-15.** That sentence was true of the *authority* backlog and
false of the whole one. A third area had no forward plan in any note: the
harness that carries the records, and how it is versioned and distributed.
[Note 06](06-harness-and-distribution-2026-09.md) now holds it.

## 2. Wave A: the measured holes (filed as 092 to 096, all shipped)

Each row is a defect reproduced this session, not a proposal. None of them
needs a new record type, a schema MAJOR or a decision from another repository.

**Status, 2026-09-15:** every row below shipped, and 097 with them. §8 carries
the per-item mapping. The measurements are preserved as they were taken.

### 2.1 A mode-only or binary change is invisible to the gate (092)

`git diff --no-color -U0 --no-renames`, the invocation `couple` runs, prints
no `---` or `+++` header for either shape:

```
diff --git a/a.sh b/a.sh          diff --git a/b.bin b/b.bin
old mode 100644                   index 8352675..1592e5c 100644
new mode 100755                   Binary files a/b.bin and b/b.bin differ
```

`cmd_couple.rs::parse_unified_diff` registers a path only from those headers,
so both changes reach `DiffInput` as nothing at all. An owned binary can change
without its spec and `C-001` stays silent; a mode flip on an unclaimed `.sh` is
invisible to `C-002`. A binary add and a binary delete print the same way, with
`/dev/null` on the missing side.

Spec 071 met this while building `delta` and sidestepped it with
`changed_path_names` (`git diff --name-only -z`), recording in its D-9 that
`couple` is spec 005's territory. 092 is that follow-up.

### 2.2 Six governed reads carry no version, and three are unsorted (093)

`docs/api.md` line 302 states that all emitted JSON is pretty-printed with
sorted keys. Measured:

| Verb | Top-level key order | `schemaVersion` |
|---|---|---|
| `registry plan --json` | `ready,blocked,notSchedulable,planned` | absent |
| `index owner --json` | `path,owners` | absent |
| `index coverage --json` | `sourceFiles,claimedFiles,floorOnlyFiles,...` | absent |
| `registry show --json` | sorted | absent |
| `registry list --json` | bare array | absent |
| `index diagnostics --json` | bare array | absent |
| `lint`, `check`, `couple`, `attest` `--json` | sorted | present (spec 034) |

The read verbs serialize with `serde_json::to_string_pretty`, which emits
fields in declaration order, while every committed artifact goes through
`canonical_json`. This is F8, and D7 (envelope or version field) is the
decision 093 makes.

### 2.3 The claim window is a constant no spec records (094)

`index.rs::scan_comment_headers` reads `content.lines().take(16)`. A
`// Spec:` header on line 17 claims nothing, and under
`[coupling] require_ownership` the file then reads as unclaimed, so a change to
it is a `C-002` refusal for a bound no document states. The constant has been
in the tree since 2026-06-09 and appears in no spec, no standard and (until
`CLAUDE.md` documented it in a review) no prose.

094 declares the bound and reports the silent cases. The reporting design is
constrained by a measurement: a substring search for `Spec:` past line 16 hits
**eleven** files in this repository, every one of them prose about the
mechanism or a test fixture string. Applying the scanner's own recognizer (strip
one `//` or `#`, then require `Spec:`, then resolve the id) reduces that to
**one**, `kit_embedded.rs` line 1948, which is an embedded copy of a hook that
legitimately claims itself. Bounding the report to lines 17 through 64 leaves
**zero**, so the new diagnostic changes no committed shard in this corpus.

### 2.4 An unparseable stray shard discards the freshness verdict (095)

Measured at the verbs, which is where spec 069's own follow-up ruling said to
measure:

| Stray file in `by-spec/` | `couple` | `index check` | `check` |
|---|---|---|---|
| a copy of a valid shard | 2, `orphaned by-spec/999-stray.json` | 2, `orphaned` | 2, `orphaned` |
| `{"nope": 1}` | 2 | **3, parse error** | **3, parse error** |

`check_report` and `check_freshness_json` compute the freshness verdict and
then call `diagnostics::committed_counts(config, repo_root)?`, which parses
every file in the shard directory; the `?` throws the verdict away. Spec 069
§3.1 says `orphaned` covers "a stray file in either shard directory", and its
test asserts that on the library function only, which is exactly how this
passed. Not a regression (0.18.0 behaves the same) and not a bypass (exit 3
still refuses in CI), so it is a small spec, not a hotfix.

### 2.5 One value, two constructions, one name (096)

For `specs/086-.../spec.md`:

```
registry show 086 --json .contentHash   a4a0098235e9...  = sha256("<path>\0<normalized bytes>")
attest --spec 069      .specSourceHash  ae0144ca7e4a...  = sha256(<normalized bytes>)
```

`cmd_registry.rs` line 128 prints the first as `(sha256 of this spec.md)`, and
approved spec 048 §3.4 says the same thing in prose: "`contentHash` is SHA-256
over that spec's `spec.md` alone". It is not; it is the path-framed
construction every content hash in this repository uses. 055 §3.3 was written
for consumers who reimplement the normalization to pin against the value, and
one adopter does exactly that, so the wrong gloss is the whole defect. This is
F9, and it is the one item in wave A that carries an `amends` edge, because an
approved spec states a behavior the code never had.

## 3. Wave B: the kit and the harness deliver what they document

Not filed. Every item is measured; none needs a new record type.

| # | Claim | Measurement |
|---|---|---|
| B1 | ~~`init --with-kit` delivers the whole kit~~ **The generated protocol says what the kit says** | **Rewritten 2026-09-15; the original row was wrong on both of its facts. See §9.1.** |
| B2 | One gate definition in the shipped workflow | `kit/govern.yml` line 57 runs `make gate`; the pull-request leg at lines 70 to 73 restates the commands, so the file adopters copy holds two definitions of one gate. |
| B3 | Installing the kit does not trip the ratchet | `.githooks/*.sh` are `SOURCE_EXTS` sources, and a `README` claim beats `bypass_prefixes`, so following the kit's own install instructions can raise `C-002` in the adopter's first PR. |
| B4 | `/shepherd` sees every reviewer | `shepherd/SKILL.md` line 137 queries `pulls/<n>/comments` only. An AI review pass posts to `issues/<n>/comments`, and the skill's green path returns before Step 3b, so that reviewer is invisible on both counts. The kit copy and this repository's copy are byte-identical (048, 081), so it is one edit in two pinned places. |
| B5 | One governed source generates the agent instruction trees | **Shipped as spec 095** (#222, ratified #223, 2026-09-16). `scripts/gen-agent-trees.py` writes `.claude/skills/`, `.agents/skills/` and `.codex/agents/` from one source, deletes what no source maps to, rewrites nothing, and all three trees are claimed and hashed. The measurement below is what it found. `.agents/skills/` was fifteen tracked files carrying the pre-081 skill set, with `.Codex/rules/` paths that exist on no filesystem, added by spec 093's own commit and unchanged since. No spec claims it and no `extra_hashed_inputs` glob covers it. The maintainer's 2026-09-12 ruling is to generate the supported trees from one source with a parity test, in the shape `kit_embedded.rs` already uses, not to delete the tree. **B5 is parity, and parity only: it delivers no global installation, upgrade, pinning, compatibility floor or recorded resolved identity. Those are [note 06](06-harness-and-distribution-2026-09.md) §3.2 and §3.4, and shipping B5 must not be reported as delivering them.** |

One rider, which should not ride: the ownership ratchet's markdown blind spot.
`C-002` reaches only `coverage.rs::SOURCE_EXTS`, so a tracked unclaimed
`.md` can be added or deleted with `couple` reporting no drift (this was
observed once, on an accidental deletion of all fifteen `.agents/skills/`
files). Closing it widens refusals in every adopter corpus, so it wants its own
spec and a decision, not a quiet extension of B5.

**The rider shipped.** It was filed as 097, built in #211 and ratified in #213
on 2026-09-15, and the decision it needed was taken (§7.1 row 6). The mechanism
is an **opt-in** `[coverage] governed_scope`: named path patterns join the
coverage universe whatever their extension, empty by default so no adopter's
verdict changes on upgrade. What remains here is not the mechanism but the
adoption: **this repository has not enabled it**, so the nineteen tracked files
097 measured are still outside `C-002`, and the seven that already carry a
valid `// Spec:` header are still inert (097 §3.4 leaves the claim scanner
untouched by design). Turning it on is a separate change with its own
consequences, and it belongs in §9.2, not in wave B.

## 4. Wave C: note 04's P1 vocabulary (prerequisite met, priority contested)

Note 04 §5 already states the ordering rule: obligations, closures and scopes
all reference a snapshot digest. The concrete dependency is narrower than that
sentence and worth naming: **`frame/1` is defined by 087 §3.3 and implemented in
the file 087 claims** (`crates/spec-spine-core/src/snapshot.rs`). An obligation's
`sectionHash` and a closure's item digests are framed digests, so filing them
before 087 is approved would mean specifying a construction that no approved
spec owns.

**That prerequisite is met, and it does not make this wave next.** 087 is
approved and complete as of 2026-09-15, so the construction these records would
reference now has an owner. Separately and in the other direction,
grand-refactor revision 4 **SP-03 recommends deferring obligations, WorkScope,
ContextClosure, A10 adapters and B23 expansion beyond the local slice**,
reopening them for a named consumer need. Those are two different statements:
one is a technical fact about this corpus, the other is a proposed ecosystem
sequencing. Checked on 2026-09-15 against revision 4 §5: **SP-03 is not
adopted.** Its own header reads "Approval status: PROPOSED, awaiting Bart's
adoption", and the only recorded adoption is the four statecrafting-profile
rows of 2026-09-12. So neither direction is settled, and a session must not
treat "newly buildable" as "next". §9.2 carries the row.

| Item | Note 04 | What changed since the note was written |
|---|---|---|
| Obligation records | §4.4 | Unchanged. The payoff worth restating: a `verification` obligation's declared `inputs` are what let 088's delta see a candidate editing the tests that judge it, which today classify as `implementation`. |
| ContextClosure | §4.5 | Unchanged. |
| WorkScope | §4.6 | **Narrower.** Spec 072 shipped overlap reporting on `registry plan`, which was §4.6's motivating example. What remains is the mutable-territory set, the `permittedEdits` list for the spec's own file, declared shared outputs, and the snapshot and closure digest references. |
| Verifier fixtures | §7 | 085 shipped the tamper cases in its `## Verification`. What remains is packaging them as a fixture set a consumer can run, with the neutral verifier itself staying consumer-owned (D6). |

## 5. Wave D: note 04's P2

Unchanged from note 04 §5 and §6: declared impact and conflict sets (A04, A05,
B03); interface references with digest-pinned imports across family corpora
(B22, A06); a reviewed move mapping, where git similarity is a suggestion and
never authority (A07); the typed overlay seam for effect contracts, budgets,
dependency rationale and contract adapters (B15, B18, B24, A09); and waiver
lifecycle evaluated purely from caller-supplied time, ancestry and usage (B17).

## 6. What is deliberately not proposed

- **A refusing coherence gate** (note 02 G7, note 04 D-4). Design first.
- **Build identity** (note 04 D1). Decided at v0.19.0: document the limit.
- **Committed evidence trees** (042 §3.3, note 02 §5). A committed evidence
  class needs its own freshness gate and restales on every edit.
- **Merkle membership proofs** (B11). Deferred until a consumer needs selective
  verification; the existing folds are sorted hashes, not trees.
- **Changing the 16-line claim window's value** (094 §4). Raising or lowering it
  reclassifies files in every adopter corpus. 094 declares the bound it has.

## 7. Decisions, and how they were taken

A review pass on 2026-09-15 ruled on all four. The rulings are recorded here
rather than in the drafts' `## 5. Resolved decisions`, because three of them are
decisions *about* a draft (whether it may be built) rather than decisions the
draft records.

| # | Decision | Ruling (2026-09-15) |
|---|---|---|
| 1 | Authorize 087 for build | **Proceed**, after §3.3's piece-selection rules are made explicit. It stays `draft` through the build PR and is ratified in a separate PR after merge, per `AGENTS.md` "Working the backlog" step 6; this row said "approve then build" in error, which inverts the cadence this repository runs. **Done: built #208, ratified #212, 2026-09-15** |
| 2 | Accept 092's refusal widening: after it lands, an owned binary or a mode flip needs its spec edited like any other change | **Accepted** as filed. Governance must not depend on whether git prints a textual hunk. The bypass floor and the clearance rules are unchanged, so the widening reaches only paths a spec already claims |
| 3 | Accept 093's one breaking output change | **Direction accepted, draft revised first.** The compatibility surface is wider than the draft stated: `--ids-only --json` is also an array and approved spec 009 §3.1 requires it, so 093 now carries an `amends` edge; `plan --next --json` emits `null` on an empty ready set, which the proposed emitter would have refused; and two read verbs were missing from the inventory |
| 4 | Whether the ownership ratchet should reach tracked files outside `SOURCE_EXTS` | **Not by extending the list.** The extension is not the binding constraint: the coverage universe is a conjunction of four tests, and discovered-package membership is the one that excludes most governance files, so a longer extension list still would not reach `AGENTS.md`, `.github/workflows/` or `scripts/`. Measured 2026-09-15: 19 tracked files have a `SOURCE_EXTS` extension and are invisible on the package test alone, 11 of them are already `[index] extra_hashed_inputs` entries, and **7 already carry a valid `// Spec:` claim header that nothing reads**. Filed as 097: an explicit opt-in governed scope, empty by default so no adopter's verdict changes on upgrade |

The five drafts the same pass revised (087, 093, 094, 095, 096) carry their
corrections in place; none of the corrections changed a draft's claim, only what
it promised about existing behavior. That pass recommended the build order 092,
095, 096, 094, 093, then 087; §7.1 settles the remaining questions and appends
097 to it.

### 7.1 The second pass (2026-09-15)

A second review pass read the revised drafts and ruled on what the first pass
left open. The rulings are recorded here; each draft carries the corrections in
its own text, and none of them reopened a design.

| # | Question | Ruling (2026-09-15) |
|---|---|---|
| 5 | 093's compatibility surface grew from one breaking document to four across three verbs, and from one `amends` edge to two | **Accepted in full.** A consumer gets a predictable, versioned answer including when nothing is ready, which is worth the cost. All four transitions stand: `registry list`, `registry list --ids-only` and `index diagnostics` move under `items`; `plan --next` moves under a nullable `next`. The shared read axis, the `config_version` exception, the unchanged text forms and the amendments to 010 and 060 are unchanged |
| 6 | Whether 097's opt-in direction is build-ready as drafted | **Direction right, text not ready; corrected before its build.** Four contradictions: it implied the seven inert headers would arrive owned while §3.4 leaves the scanner untouched; it offered an escape hatch (removing `.github/` from `[coupling] bypass_prefixes`) that does not exist, since the configured list only adds to the built-in floor; its enumeration left an absent inventory and an empty one indistinguishable and allowed a silent fallback after a git failure; and it promised "byte-for-byte" unchanged output while adding config keys and scaffold text |
| 7 | 087's remaining details | **Two, both settled.** An empty directory needs its own piece kind (`d`), or it frames identically to an empty file at the same path; and a spec directly claiming a non-UTF-8 file, which makes `attest --spec` exit 3 (reproduced), must not take the snapshot down with it: the join hash is omitted with a stated reason while `territoryDigest` is still produced. Existing attestation behaviour is preserved, and a genuinely unreadable input remains an error |
| 8 | Where 097 sits in the build order | **Last.** 092 → 095 → 096 → 094 → 093 → 087 → 097, one spec per build PR, `draft` through the build and ratified separately. 097 depends on 094, and enabling the scope in this repository is a separate adoption change after the mechanism ships |

What the corrections changed, precisely, is what each draft promises about
behavior that already exists: 097 now states that spec 008's explicit-claim
precedence is the override that exists and that scope membership is not a second
one, and 087 now states that the per-spec verb keeps refusing what it refuses
today. No draft's claim moved.

A third read (2026-09-15) kept `index coverage --paths-from` and corrected two
wording contradictions in 097 before its build, recorded there as D-6: ignore
rules exclude only untracked files (a tracked file matching `.gitignore` stays
in the inventory, which is what the prescribed `git ls-files` invocation already
does), and the two new report members are omitted only when the configured
scope is empty, never when a set scope matched nothing, so `enumeration`
survives the case that needs it. The build order is unchanged.

## 8. Where each item is filed

Lifecycle read from the corpus at `3bc004b` on 2026-09-15.

| Item | Filed as | Lifecycle |
|---|---|---|
| 2.1 mode-only and binary changes reach the gate | 073-a-mode-only-or-binary-change-is-a-change | approved, complete (#200, ratified #201) |
| 2.2 F8 and D7: versioned, sorted read documents | 074-a-governed-read-names-its-version | approved, complete (#207, ratified #210) |
| 2.3 the claim window, declared and its silent cases reported | 075-a-claim-below-the-header-window-is-not-silent | approved, complete (#205, ratified #209) |
| 2.4 the stray shard verdict survives the tally | 076-a-stray-shard-is-orphaned-at-the-verbs | approved, complete (#202, ratified #204) |
| 2.5 F9: one name, one construction | 077-one-hash-one-construction-one-name | approved, complete (#203, ratified #206) |
| §3 wave B rider: the ownership ratchet's reach | 078-governed-scope-is-declared-not-inferred | approved, complete (#211, ratified #213). **Mechanism shipped, not enabled here** |
| §3 wave B (five items) | not filed | B1 rewritten, see §9.1 |
| §4 wave C (four items) | not filed | prerequisite met; priority contested, see §4 and §9.2 |
| §5 wave D | not filed | |
| §9 the harness and its distribution | not filed | recorded in [note 06](06-harness-and-distribution-2026-09.md) |

## 9. Reconciliation (2026-09-15, `3bc004b`)

This section is the current record. It was written after a pass that read this
note, [note 04](04-authority-evidence-extension.md), grand-refactor's
[README](/Users/bart/DevWork/grand-refactor/README.md),
[revision 4](/Users/bart/DevWork/grand-refactor/07-revision-4-decision-package.md),
its [feature-disposition register](/Users/bart/DevWork/grand-refactor/analysis/spec-spine-feature-disposition.md)
and the [findings routed from aicortex](/Users/bart/DevWork/grand-refactor/analysis/spec-spine-findings-from-aicortex-2026-09-12.md),
alongside a Claude Code session of 2026-09-15 on the user's global agent
configuration. It changes no spec and approves nothing.

### 9.1 What this note had wrong

**B1's two facts were both false.** The row claimed `KIT_FILES` carries 27
entries against 29 tracked files with two the generator "cannot deliver", and
that `tests/scaffold.rs` "cannot notice a file the generator never carried".
Checked at `3bc004b`:

- The two omissions are **deliberate and recorded**. Spec 095 §3.2's dated
  decision of 2026-09-07 states that `kit/README.md` documents the kit rather
  than being part of it, that `kit/.gitattributes-stanza` is a block to append
  rather than a file to write, and that `kit/AGENTS.md` is a third case because
  §3.1 requires the scaffold to emit a **config-aware** `AGENTS.md`
  unconditionally, whose corpus and derived paths match the adopter's layout.
- The test does **not** compare against `KIT_FILES` alone. It walks `kit/` from
  disk, filters an explicit `let skipped = ["kit/README.md", "kit/AGENTS.md"]`
  plus `.DS_Store`, `__pycache__` and `.git`, and asserts the count equals
  `KIT_FILES.len()`. It is a parity test with named exclusions, counted from
  the tree rather than pinned to a number.

So there is no accidental omission to fix, and a spec that prescribes copying
every kit file would contradict an approved decision.

**The concrete defect is protocol drift.** The `AGENTS.md` that
`scaffold.rs` generates and the `kit/AGENTS.md` an adopter reads disagree about
the gate they both call "the gate":

| | Scaffolded (`scaffold.rs`) | `kit/AGENTS.md` |
|---|---|---|
| Freshness verb | `spec-spine index check --fail-on-unresolved` | `spec-spine check` (spec 062, both trees in one verb) |
| Base ref | `couple --base origin/main` hard-coded | `--base "$(git symbolic-ref --short refs/remotes/origin/HEAD ...)"` (spec 093) |
| Coverage | `index coverage --fail-on-untraced` unconditional | commented, conditional on `[coupling] require_ownership` |
| Session read | `spec-spine compile --check` | `spec-spine check` |

Both invocations still work, so nothing is broken; the generated protocol is
simply the older shape, and an adopter who reads the scaffolded file gets a
gate that does not match the kit's.

**The territory is wider than one file, and none of it is generated.**
`kit_embedded.rs` is generated from `kit/`; `scaffold.rs` is not. The
scaffolded `AGENTS.md` is a hand-maintained string literal in
`crates/spec-spine-core/src/scaffold.rs`, which 006 establishes and **nine
specs already extend** (043, 045, 047, 061, 062, 065, 066, 069, 074, 097).
Anything that closes this drift touches at least:

| Path | Why |
|---|---|
| `crates/spec-spine-core/src/scaffold.rs` | The literal itself. Hand-maintained, heavily co-owned |
| `crates/spec-spine-core/tests/scaffold.rs` | Where the scaffolded text is asserted; 006 establishes it, ten specs extend it |
| `crates/spec-spine-cli/tests/init.rs` | 065 extends it for the `--with-kit` path |
| `kit/README.md`, `docs/adoption-guide.md` | Only if either describes the gate an adopter is given. Checked 2026-09-15: `adoption-guide.md` already says `spec-spine check`, so the guide is ahead of the scaffold, not behind it |

**Whether it is `extends` or `amends` is not decidable in advance, and one
measurement says it is probably `amends`.** 065's own `## Verification` block
runs `grep -q 'spec-spine compile --check' .../AGENTS.md` against a freshly
scaffolded tree. Aligning the scaffolded protocol to the kit's `spec-spine
check` would make that line fail, which means the change falsifies acceptance
an approved spec states, and that is what `amends` is for (040). A narrower
spec that only resolved the base ref, say, might need no such edge. The edge
follows from what the final spec actually promises; it is not a choice to make
now, and it is never an edit to 065.

The other two constraints stand: the adopter's **custom paths** (the scaffold
is config-aware precisely so an adopter's corpus and derived directories are
right) and **destination conflicts** (065 §3.3 keeps `overwrite: false` on
every kit file, so a rewrite must not clobber).

**The rider prose predated its own filing.** §3's rider paragraph still read as
an open question while §8 already recorded 097. Corrected in place: the
mechanism shipped and is not enabled here.

**Wave A read as unfiled.** §2's heading and §1's table row described drafts and
a held 087. All seven specs are approved and complete. Corrected in place.

### 9.2 Every item, with its owner and status

Decision status uses note 06 §0's vocabulary: proposed, adopted, implemented,
released.

| Source item | Owner | Decision status | Spec or design reference | Remaining action |
|---|---|---|---|---|
| Mode-only and binary changes reach the gate | spec-spine | implemented; released in no tag yet | 092 | Ships with the next release |
| Versioned, sorted read documents (note 04 F8, D7) | spec-spine | implemented | 093, amended by 103 | Its `## Verification` line 13 failed on an empty ready set. Spec 082 built the route an acceptance amendment needed (`amends_verification`, resolved by `verify`) and holds 093's corrected block; 093's file is untouched and `verify 093` is green |
| Claim window declared, near misses reported | spec-spine | implemented | 094 | None |
| Stray shard orphaned at the verbs | spec-spine | implemented | 095 | None |
| One hash, one construction, one name (note 04 F9) | spec-spine | implemented | 096 | None |
| AuthoritySnapshot, `frame/1` | spec-spine | implemented | 087, note 04 §4.2 | None |
| AuthorityDelta | spec-spine | implemented | 088, note 04 §4.3 | None |
| Governed scope, declared not inferred | spec-spine | implemented, **not enabled here** | 097 | Enabling `[coverage] governed_scope` in this repository is separate adoption work with its own blast radius |
| Markdown blind spot in `C-002` | spec-spine | answered by 097's opt-in | 097 §3.4 | The seven inert `// Spec:` headers stay inert by design; if that should change it is a new spec |
| B1 generated protocol drift | spec-spine | proposed | §9.1, 065 §3.1 to §3.3 | Candidate next spec, §9.4. Territory is hand-maintained `scaffold.rs` plus two test files; likely `amends` 065, whose `## Verification` pins the current text |
| B2 `kit/govern.yml` holds two gate definitions | spec-spine | proposed | §3 B2 | Unfiled |
| B3 kit install trips the ratchet | spec-spine | proposed | §3 B3 | Unfiled |
| B4 `/shepherd` misses `issues/<n>/comments` | spec-spine | proposed | §3 B4 | Unfiled; one edit in two byte-identical places |
| B5 one source generates the agent trees | spec-spine | proposed | §3 B5, note 06 §3.4 | Unfiled. Parity only; not distribution |
| Staged coupling (`couple --head HEAD` reads `base...head`) | spec-spine | **implemented** | 102, amending 090 §4 | H-7 decided 2026-09-16: fixed in `couple`. `--include-uncommitted` unions the committed range with `git diff HEAD`; the default is unchanged so CI stays reproducible. 090 §4's "is filed separately" is amended; 092 §4's sentence was accurate and is untouched |
| Obligation records | spec-spine | proposed here, **proposed for deferral** by SP-03 | note 04 §4.4, revision 4 SP-03 | Neither adopted; needs a named consumer or an owner decision |
| ContextClosure | spec-spine | proposed here, proposed for deferral by SP-03 | note 04 §4.5 | As above |
| WorkScope | spec-spine | proposed here, proposed for deferral by SP-03 | note 04 §4.6, narrowed by 091 | As above |
| Verifier fixture packaging | spec-spine emits; verifier stays consumer-owned | proposed | note 04 §7, D6; 085 shipped the tamper cases | Unfiled |
| A10 authoring-tool adapters, B23 intent manifest | spec-spine | proposed for deferral by SP-03 | feature register A10, B23 | Not this repository's next work |
| Wave D (A04, A05, A06, A07, B15, B17, B18, B22, B24) | spec-spine for the declarations, consumers for the checks | proposed | note 04 §6, §5 here | Several need decisions first |
| Global personal policy | the user's global configuration | proposed | note 06 §3.1 | Not spec-spine work |
| Versioned namespaced harness package | spec-spine | proposed | note 06 §3.2 | Open questions H-1, H-2, H-3. `kit/**` stays the governed source here; H-3 is the adopter's substitution, not this repository's |
| Repository-local layer named explicitly | adopting repository | mostly implemented (048, 081) | note 06 §3.3 | Only the lifecycle-policy half is open (H-5) |
| Supported-agent adapters, one maintained source | spec-spine | proposed | note 06 §3.4 | Superset of B5 |
| Task-specific startup reads, explicit full `/prime` | spec-spine | proposed | note 06 §3.5 | Which reads each path carries is performance-sensitive and wants note 06 §3.10's numbers; that a worker should not run the full orientation does not. `AGENTS.md` is a hashed input, so this restales every shard |
| Build eligibility from repository policy | spec-spine plus each repository | proposed | note 06 §3.6 | Live contradiction between the kit's `build` skill and this repository's `AGENTS.md`. Specify that the skill defers to policy and handles new, resumed and repair work; how policy is expressed stays open (H-5) |
| Statecraft owns scheduling, retries, model, effort, budget, completion | Statecraft | proposed | note 06 §3.7 | spec-spine's constraint: no competing stage loops in skills |
| Guarded profile lacks the Rust and Make toolchain | Statecraft | proposed | note 06 §3.7 | Handback, not spec-spine work |
| Validation boundaries and evidence reuse | spec-spine skills | proposed | note 06 §3.8 | Unfiled |
| `Stop` hook calls every nonzero `check` "STALE" | spec-spine | proposed | note 06 §3.9 | Decision H-6 |
| PR hook's `git diff --quiet -- .derived/` misses staged and untracked shards | spec-spine | proposed | note 06 §3.9 | Unfiled |
| Commit-boundary freshness | spec-spine | implemented | 090 | 090 §4 explicitly excludes staged coupling; see the unowned row above |
| Exit-code-aware PR gate | spec-spine | **implemented** | 080, 099, 104 | 099 gave the session hooks the treatment; 104 closed the last two gaps: the gate now probes `check --help` on its exit-2 arm (spec 093) and `SessionStart` reports exit 3 as a read that was not performed |
| Measurement plan | spec-spine and Statecraft | proposed | note 06 §3.10 | Precedes filing note 06's cost-justified items |
| N2 draft in the ready set | spec-spine | see §9.3 | 038 §3.1, 048 D-2 | Consumer guidance owed |
| N6 `I-004` refusal exits 2 | spec-spine | **implemented** | 101, amending 086 §3.1 and 098 §3.1 | Part 1 (the message) shipped with 098. Part 2 decided 2026-09-16: an unresolved claim exits 1, the validation code. Both approved specs stating the old rule are amended |
| Personal settings inventory, Statecraft execution profiles, Aicortex retrieval | their owners | out of scope here | note 06 §4 | Boundaries recorded; nothing imported |

### 9.3 The two findings routed from aicortex

Both were observed with `spec-spine 0.18.0` on 2026-09-12 against an aicortex
export, and were explicitly filed as findings rather than requests. Both were
**re-reproduced on 2026-09-15 with the 0.19.0 binary**, in a scratch corpus
built by `spec-spine init`, so these dispositions rest on current behavior and
not on the dated report.

**N2, a draft appears in `registry plan`'s ready set: intentional layering,
consumer guidance owed.**

Reproduced: a `status: draft`, `implementation: pending` spec is listed under
`ready` and the entry is `{"id": ..., "title": ...}` with nothing else.
`ReadySpec` in `query.rs` carries exactly those two fields, so 093's versioning
did not touch this. The layering is deliberate and documented in two approved
places: 038 §3.1 partitions by `status: superseded | retired` and by
`implementation`, not by approval, and 048 D-2 puts the approval rule in the
kit's `/next` on top of `plan`. aicortex relies on that layering and asserts no
answer.

Disposition: **intentional, and the design is not reopened.** What is owed is
guidance, not a behavior change: `docs/api.md` describes `plan`'s document
without saying that readiness is a scheduling fact and never an approval, and a
consumer reading `plan` directly has no governed sentence telling it so. The
smallest honest fix is documentation. An additive `status` field on `ReadySpec`
is a schema MINOR that would let a direct consumer see what `/next` sees; it is
a reasonable spec but it is not required to make the contract truthful, and it
should not be filed as a defect fix.

**N6, an `I-004` refusal exits through the staleness code: needs a fix, and the
message is worse than the code.**

Reproduced at 0.19.0 with a spec declaring `implementation: complete` over a
file that does not exist:

```
$ spec-spine check
spec-registry: fresh
codebase-index: STALE (run `spec-spine index`)
1 stale shard(s):
  blocking-diagnostics by-spec/001-missing-territory.json
$ echo $?
2
$ spec-spine index && spec-spine check   # the advice, followed
codebase-index: STALE (run `spec-spine index`)
$ echo $?
2
```

The exit code is the smaller half. The verb **names a remedy that provably does
not work**: `index` exits 0, writes the shard, and `check` still refuses,
because re-indexing cannot conjure a file the spec claims. An adopter's
`CLAUDE.md` that documents exit 2 as staleness (aicortex's does) will print
"run `spec-spine index`" forever. The `Stop` hook amplifies it: every nonzero
`check` becomes `[freshness] STALE`, so the same wrong remedy reaches the
session end (note 06 §3.9).

Disposition: **future work, and a real defect rather than a semantics
preference.** Two parts, separable:

1. The advice line must not say "run `spec-spine index`" for a blocking
   diagnostic. This is a message fix and carries no exit-code compatibility
   question.
2. Whether an `I-004` refusal belongs under exit 1 (validation) rather than 2
   (stale) is a contract change: 086 §3.1 states "the exit code is unchanged:
   2 when anything drifted", and adopters branch on it. It needs an `amends`
   edge on 086 and a decision, not a build's judgment.

aicortex's second observation, that `draft` plus `implementation: complete`
compiles and lints clean, also reproduced. Disposition: **intentional here.**
This repository's own cadence files a spec as `draft`, builds it to
`implementation: complete`, and ratifies afterwards, so a lint refusing that
pair would refuse this corpus's normal state. An adopter whose cadence is
ratify-then-build may want it; that is a configurable lint, not a default.

### 9.4 What is still open

| # | Question | Who decides |
|---|---|---|
| ~~R-1~~ | **Answered 2026-09-16, shipped as spec 082.** The one-line amendment needed a mechanism first: `verify` executes the amended file, so an acceptance amendment that did not redirect the executor changed nothing. 103 adds `amends_verification`, resolved through a chain and past a withdrawn holder, stated on every run. 093 is not edited | Decided |
| R-2 | What goes next. Wave B's numbering is not an ordering: rank by impact and dependency. The three live candidates are the generated-protocol drift (§9.1), the `I-004` remedy line (§9.3), and R-1's acceptance amendment | Human |
| ~~R-3~~ | **Answered 2026-09-16, shipped as spec 081.** Fixed in `couple`, not the harness: an adopter running it by hand pre-commit needed the same answer. `--include-uncommitted`, off by default. Only 090 §4's "is filed separately" was untrue and only it is amended | Decided |
| ~~R-4~~ | **Answered 2026-09-16, shipped as spec 080.** It moves. Both 086 §3.1 and 098 §3.1 state the old rule normatively, so both are amended; 098's own forecast named only 086 | Decided |
| R-5 | Whether SP-03's proposed deferral of obligations, WorkScope, ContextClosure, A10 and B23 is adopted. Until it is, wave C is neither scheduled nor withdrawn | Human, recorded in grand-refactor's adoption record, not here |
| R-6 | Note 06's H-1 to H-6: package shape, revision declaration, the adopter's governed revision and resolved-package evidence, startup-path shape, how eligibility policy is expressed, and what the `Stop` hook does with exit 1 and 3. **Open questions, not gates**: none of them blocks an independent correctness fix | Human |
| R-7 | Whether this repository enables `[coverage] governed_scope` | Human |
