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

## 1. What the other notes still owe

| Note | Disposition |
|---|---|
| 01 partial supersession | Shipped as 019. Owes nothing. |
| 02 agentic-builder substrate | Waves 1 and 2 shipped (037 to 042). Wave 3's G5 landed as 061, 065 and 081; G8 (redaction) is answered by 042. Wave 4 (G7, the coherence gate) is deliberately unfiled, and note 04 D-4 re-affirms that: the naive rule refuses legitimate refactors, so no mechanism is asserted before it is designed. |
| 03 adopter audit | Both backlogs filed as 045 to 068 (note 03 §7 records the mapping). §5 is adopter-side work and is not this repository's. |
| 04 authority evidence | P0 is shipped except 087, which is ready and held. **P1 and P2 are entirely unfiled.** Findings F8 and F9 are recorded there as "small fix, unfiled" and are filed here. D7 is decided by 093. |

So note 04 is the only note with an unfiled forward plan, and the unfiled pool
splits into two waves that need no new record vocabulary (A and B) and two that
build on 087's framed digest (C and D).

## 2. Wave A: the measured holes (filed as 092 to 096)

Each row is a defect reproduced this session, not a proposal. None of them
needs a new record type, a schema MAJOR or a decision from another repository.

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

Spec 088 met this while building `delta` and sidestepped it with
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
| `lint`, `check`, `couple`, `attest` `--json` | sorted | present (spec 037) |

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

Measured at the verbs, which is where spec 086's own follow-up ruling said to
measure:

| Stray file in `by-spec/` | `couple` | `index check` | `check` |
|---|---|---|---|
| a copy of a valid shard | 2, `orphaned by-spec/999-stray.json` | 2, `orphaned` | 2, `orphaned` |
| `{"nope": 1}` | 2 | **3, parse error** | **3, parse error** |

`check_report` and `check_freshness_json` compute the freshness verdict and
then call `diagnostics::committed_counts(config, repo_root)?`, which parses
every file in the shard directory; the `?` throws the verdict away. Spec 086
§3.1 says `orphaned` covers "a stray file in either shard directory", and its
test asserts that on the library function only, which is exactly how this
passed. Not a regression (0.18.0 behaves the same) and not a bypass (exit 3
still refuses in CI), so it is a small spec, not a hotfix.

### 2.5 One value, two constructions, one name (096)

For `specs/086-.../spec.md`:

```
registry show 086 --json .contentHash   a4a0098235e9...  = sha256("<path>\0<normalized bytes>")
attest --spec 086      .specSourceHash  ae0144ca7e4a...  = sha256(<normalized bytes>)
```

`cmd_registry.rs` line 128 prints the first as `(sha256 of this spec.md)`, and
approved spec 055 §3.4 says the same thing in prose: "`contentHash` is SHA-256
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
| B1 | `init --with-kit` delivers the whole kit | `KIT_FILES` carries 27 entries against 29 tracked files in `kit/`. The two it cannot deliver are `kit/AGENTS.md` and `kit/README.md`, and all ten skills name `AGENTS.md` "Working the backlog" as the protocol they sequence. `init` writes its own shorter generated `AGENTS.md`, so an adopter gets the skills and not the kit's own copy of what they cite. `tests/scaffold.rs` compares against `KIT_FILES`, so it cannot notice a file the generator never carried. |
| B2 | One gate definition in the shipped workflow | `kit/govern.yml` line 57 runs `make gate`; the pull-request leg at lines 70 to 73 restates the commands, so the file adopters copy holds two definitions of one gate. |
| B3 | Installing the kit does not trip the ratchet | `.githooks/*.sh` are `SOURCE_EXTS` sources, and a `README` claim beats `bypass_prefixes`, so following the kit's own install instructions can raise `C-002` in the adopter's first PR. |
| B4 | `/shepherd` sees every reviewer | `shepherd/SKILL.md` line 137 queries `pulls/<n>/comments` only. An AI review pass posts to `issues/<n>/comments`, and the skill's green path returns before Step 3b, so that reviewer is invisible on both counts. The kit copy and this repository's copy are byte-identical (048, 081), so it is one edit in two pinned places. |
| B5 | One governed source generates the agent instruction trees | `.agents/skills/` is fifteen tracked files carrying the pre-081 skill set, with `.Codex/rules/` paths that exist on no filesystem, added by spec 081's own commit and unchanged since. No spec claims it and no `extra_hashed_inputs` glob covers it. The maintainer's 2026-09-12 ruling is to generate the supported trees from one source with a parity test, in the shape `kit_embedded.rs` already uses, not to delete the tree. |

One rider, which should not ride: the ownership ratchet's markdown blind spot.
`C-002` reaches only `coverage.rs::SOURCE_EXTS`, so a tracked unclaimed
`.md` can be added or deleted with `couple` reporting no drift (this was
observed once, on an accidental deletion of all fifteen `.agents/skills/`
files). Closing it widens refusals in every adopter corpus, so it wants its own
spec and a decision, not a quiet extension of B5.

## 4. Wave C: note 04's P1 vocabulary (gated on 087)

Note 04 §5 already states the ordering rule: obligations, closures and scopes
all reference a snapshot digest. The concrete dependency is narrower than that
sentence and worth naming: **`frame/1` is defined by 087 §3.3 and implemented in
the file 087 claims** (`crates/spec-spine-core/src/snapshot.rs`). An obligation's
`sectionHash` and a closure's item digests are framed digests, so filing them
before 087 is approved would mean specifying a construction that no approved
spec owns.

| Item | Note 04 | What changed since the note was written |
|---|---|---|
| Obligation records | §4.4 | Unchanged. The payoff worth restating: a `verification` obligation's declared `inputs` are what let 088's delta see a candidate editing the tests that judge it, which today classify as `implementation`. |
| ContextClosure | §4.5 | Unchanged. |
| WorkScope | §4.6 | **Narrower.** Spec 091 shipped overlap reporting on `registry plan`, which was §4.6's motivating example. What remains is the mutable-territory set, the `permittedEdits` list for the spec's own file, declared shared outputs, and the snapshot and closure digest references. |
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
| 1 | Authorize 087 for build | **Proceed**, after §3.3's piece-selection rules are made explicit. It stays `draft` through the build PR and is ratified in a separate PR after merge, per `AGENTS.md` "Working the backlog" step 6; this row said "approve then build" in error, which inverts the cadence this repository runs |
| 2 | Accept 092's refusal widening: after it lands, an owned binary or a mode flip needs its spec edited like any other change | **Accepted** as filed. Governance must not depend on whether git prints a textual hunk. The bypass floor and the clearance rules are unchanged, so the widening reaches only paths a spec already claims |
| 3 | Accept 093's one breaking output change | **Direction accepted, draft revised first.** The compatibility surface is wider than the draft stated: `--ids-only --json` is also an array and approved spec 010 §3.1 requires it, so 093 now carries an `amends` edge; `plan --next --json` emits `null` on an empty ready set, which the proposed emitter would have refused; and two read verbs were missing from the inventory |
| 4 | Whether the ownership ratchet should reach tracked files outside `SOURCE_EXTS` | **Not by extending the list.** The extension is not the binding constraint: the coverage universe is a conjunction of four tests, and discovered-package membership is the one that excludes most governance files, so a longer extension list still would not reach `AGENTS.md`, `.github/workflows/` or `scripts/`. Measured 2026-09-15: 19 tracked files have a `SOURCE_EXTS` extension and are invisible on the package test alone, 11 of them are already `[index] extra_hashed_inputs` entries, and **7 already carry a valid `// Spec:` claim header that nothing reads**. Filed as 097: an explicit opt-in governed scope, empty by default so no adopter's verdict changes on upgrade |

The three drafts the same pass revised (093, 094, 095, 096 and 087) carry their
corrections in place; none of the corrections changed a draft's claim, only what
it promised about existing behavior. The build order the pass recommends is
092, 095, 096, 094, 093, then 087.

## 8. Where each item is filed

| Item | Filed as |
|---|---|
| 2.1 mode-only and binary changes reach the gate | 092-a-mode-only-or-binary-change-is-a-change |
| 2.2 F8 and D7: versioned, sorted read documents | 093-a-governed-read-names-its-version |
| 2.3 the claim window, declared and its silent cases reported | 094-a-claim-below-the-header-window-is-not-silent |
| 2.4 the stray shard verdict survives the tally | 095-a-stray-shard-is-orphaned-at-the-verbs |
| 2.5 F9: one name, one construction | 096-one-hash-one-construction-one-name |
| §3 wave B rider: the ownership ratchet's reach | 097-governed-scope-is-declared-not-inferred |
| §3 wave B (five items) | not filed |
| §4 wave C (four items) | not filed; gated on 087 |
| §5 wave D | not filed |
