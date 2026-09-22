# 11: Producer-release review and local completion (2026-09-21)

The second authorized session on the same stack. It corrected the delivery
record, fixed the one contradiction note 10 left unfixed, finished the
pre-publication checks that can run locally, and dispositioned the rest.

**Nothing was pushed, opened as a pull request, merged, ratified, tagged or
published. No waiver was written. No sibling repository was touched. Statecraft
was not activated.**

Note 09 is the backlog record. Note 10 is the first session's delivery record
and now carries **[corrected]** markers pointing here. This note is the second
session's delivery record.

## 1. Branches, and their exact tips

Ten local branches are in play; none is pushed. `origin/main` is at `4ab1b31e`.

| Branch | Tip | Base it was judged against | Added by |
|---|---|---|---|
| `specs/disposition-and-drafts-2026-09-21` | `df6fb4f7` | `origin/main` | session 1 |
| `100-a-deleted-path-is-judged-where-it-lived` | `a3d5213d` | the disposition branch | session 1 |
| `101-readiness-is-scheduling-not-approval` | `de854d14` | `100` | session 1 |
| `103-a-verifier-fixture-is-a-published-artifact` | `13c3ccd8` | `101` (a deliberate sibling) | session 1 |
| `release/0.22.0-candidate` | **`461be3fb`** | `origin/main` | session 1, record corrected by session 2 |
| `105-governed-scope-is-enabled-here` | `a48a1691` | the release candidate | session 1 |
| `specs/deferred-contracts-2026-09-21` | **`f5e72976`**, plus this note | `105` | session 1, notes added by session 2 |
| `117-the-gate-resolves-the-binary-it-documents` | **`02c19e9e`** | the release candidate | session 2 |
| `release/0.22.0-integration` | **`6dd96aa7`** | `origin/main` | session 2 |

The bolded tips are the ones note 10's first revision got wrong or did not
have. `release/0.22.0-candidate` moved because session 2 committed the
corrected record onto it; the drafts branch moved because note 10 itself was
committed onto it after its own table was written.

**A branch's comparison base is not the eventual integration candidate.** Spec
117 is judged against `release/0.22.0-candidate`, the branch it was cut from,
and against that base it is clean. The same branch diffed against `origin/main`
inherits the candidate's two coupling refusals, which are not spec 117's and
are not resolved by it. Both numbers appear in §3 and neither is offered as the
other.

**Ancestry is not merge-readiness.** Spec 117 descends from the candidate, so
the integration merge is fast-forwardable and its tree is byte-identical to
spec 117's tip. That says a merge can be computed. It says nothing about a
protected pull request, which is judged at endpoints that do not exist yet: the
base it is actually opened against, the head sha its checks actually run on,
and the body its coupling gate is actually given.

## 2. Spec 117: the gate resolves the binary it documents

Note 10 §7.1 found that `make gate` governs with whatever `spec-spine` is on
`PATH` while the `Makefile`'s own header documents a resolution order
preferring the binary the checkout builds. It was left unfixed, correctly: it
is another spec's file and fixing it inside an unrelated build is the drift
this corpus refuses. It is fixed here, as its own spec, with its own evidence.

**Territory.** `index owner Makefile` reports
`094-one-gate-and-the-boundaries-it-holds` as the establisher and `092` as an
`extends`; note 10 attributed the file to 092 alone. Spec 117 declares an
`extends` edge naming 094 at `Makefile` and establishes one new file, the
selection test. No approved prose was edited: spec 094 has nothing to say about
binary resolution and is not made wrong by this. Spec 093 §3.12 states the same
order for the **hooks**, is correct about them, and is referenced rather than
amended.

**What changed.** The assignment became a real three-step resolution: an
explicit `SPEC_SPINE` (command line or environment), then
`./target/release/spec-spine` when it is an executable regular file, then
`spec-spine` on `PATH`. An explicit override that names nothing executable is
**refused**, not replaced; so is an empty one; so is finding nothing anywhere.
`gate`, `refresh` and `verify` all announce the resolved binary, where it came
from, and its `--version`, on stderr.

**What deliberately did not change.** The gate stays read-only and does not
build the binary it judges with. `[meta] required_version` stays the one
version floor: the announcement is announcement, not a second comparison that
could drift from it. The `.githooks/pre-commit` fall-through is left alone,
and spec 117 §3.2 records why the two callers differ so that nobody later
reconciles them by making the gate quieter.

**How selection is tested.** `crates/spec-spine-core/tests/gate_binary.rs`
copies the real root `Makefile` into a temp tree, plants stub binaries with
distinct identities and distinct versions, runs `make` there, and asserts on
**which stub printed** rather than on what a variable held. Eight cases:
explicit override, in-tree available, in-tree missing, an older binary on
`PATH` with a built checkout, an invalid explicit override, an empty one,
nothing anywhere, and that the preflight builds nothing. The two refusal cases
assert the negative as well: no substitute ran. A non-zero exit on its own is
compatible with the gate having run the wrong binary and that binary having
failed, which is the exact failure being refused.

**Fail-first.** All eight fail against the assignment they replace. They do not
all fail for the same reason, and spec 117 D-4 records which: two rows fail on
**selection** (the old assignment ran the `PATH` stub), three on the **absent
refusal**, and two only on the announcement, because `?=` already honoured an
override. Recording that is the difference between "the tests are red" and
"the tests are red for the reason claimed".

**Two traps the build hit**, both recorded as decisions rather than smoothed
over:

- `tests/gate.rs` reads `$(SPEC_SPINE) <word>` in the `Makefile` as an
  invocation and checks `<word>` against the verb list. The first draft's error
  prose (`no spec-spine binary found`, `governing with $(SPEC_SPINE) [`) fed it
  two false invocations. The fix was to stop feeding false positives to a
  working detector, not to teach the detector to skip a line: the resolved path
  now goes through a shell variable (D-6).
- The acceptance block first asserted that no writing verb appears anywhere in
  the `Makefile`. It failed, correctly: `refresh` is the writing half and runs
  `compile` and `index` by design. A file-wide grep for a writing verb refuses
  the correct file. The assertion now runs spec 094's own detector, which reads
  the `gate` target (D-7).

## 3. Gate status, per branch, per step

Run with `./target/release/spec-spine` 0.22.0, which the Makefile now announces
on every run. "unavailable" means the check cannot be performed from here.

| | `117` vs its base | `release/0.22.0-candidate` vs `origin/main` | `release/0.22.0-integration` vs `origin/main` |
|---|---|---|---|
| `check --fail-on-unresolved --fail-on-warn` | **passed**, both trees fresh | **passed** | **passed** |
| `lint --fail-on-warn` | **passed**, 0/0 | **passed**, 0/0 | **passed**, 0/0 |
| `index coverage --fail-on-untraced` | **passed**, 98/98 | **passed**, 97/97 | **passed**, 98/98 |
| `couple` | **passed**, 3 paths, no drift | **FAILED**, 2 `C-001`, exit 1 | **FAILED**, the same 2 |
| `cargo test --workspace --locked` | **passed**, 41 targets | **passed**, 40 targets | tree identical to `117`'s |
| `cargo clippy ... -D warnings` | **passed** | **passed** | tree identical to `117`'s |
| `cargo fmt --all --check` | **passed** | **passed** | tree identical to `117`'s |
| `spec-spine verify 117` | **passed**, 6 commands | n/a | **passed**, 6 commands |
| CI, any workflow | **unavailable**, nothing pushed | **unavailable** | **unavailable** |
| determinism matrix, four triples | **unavailable**, CI-only | **unavailable** | **unavailable** |
| `verify-sweep.sh` (merged-revision acceptance) | **unavailable**, nothing merged | **unavailable** | **unavailable** |
| publication, any channel | **not performed** | **not performed** | **not performed** |

The integration branch's tree is byte-identical to spec 117's tip
(`git diff` between them is empty), which is why three rows are reported as
identity rather than re-run.

## 4. The five findings, re-measured

Note 10 §7 carries the corrections inline. The measurements are here.

### 4.1 The binary defect

Fixed; §2. The measurement that motivated it stands: `spec-spine` on `PATH` is
`0.20.0` on this machine, `./target/release/spec-spine` is `0.22.0`, and
`[meta] required_version = ">=0.17.0"` admits the older one, because it is a
floor and was chosen as one. Admitted is not the same as agrees.

### 4.2 The MINOR-ahead attestation

Four layers, measured against a freshly generated attestation over this corpus:

| probe | command | result |
|---|---|---|
| MAJOR ahead | `schemaVersion` `0.1.0` → `9.0.0`, `--recompute` | exit 3, `kind: "schema"`, "MAJOR 9 is unsupported" |
| MINOR ahead, version string only | `0.1.0` → `0.9.0`, `--recompute` | exit 1, `outcome: "contentMismatch"`, `differences: ["schemaVersion (0.9.0 -> 0.1.0)"]` |
| MINOR ahead, **with an added field** | `0.9.0` plus `futureField` | exit 3, `kind: "parse"`, "unknown field `futureField`" |
| unmodified | as emitted | exit 0, `outcome: "match"` |

Read together: the MAJOR gate admits a MINOR-ahead payload; the strict parse
then refuses any payload that actually uses the MINOR, because every
attestation DTO is `deny_unknown_fields` (spec 068 §3.5); a version-only edit
survives both and fails recomputation equality on `schemaVersion` alone.

That third row's `differences` list is also the digest-integrity evidence: it
names `schemaVersion` and **nothing else**, so every unit content hash in the
attestation still matched. Territory integrity is intact; document equality is
not. Those are different questions and the verb answers both.

No normative contract conflicts with another, so **no compatibility amendment
is proposed**. The only change that would make a MINOR-ahead payload verify is
dropping `schemaVersion` from the comparison, which spec 068 §3.3 forbids by
name. Spec 103's fixture keeps its measured expectation.

### 4.3 The working-tree deletion precondition

Reproduced by deleting `crates/spec-spine-core/src/coverage.rs` in the working
tree of the candidate:

```
$ spec-spine check --fail-on-unresolved --fail-on-warn     → exit 1
codebase-index: UNRESOLVED CLAIM: 7 unresolved claim(s) over 7 spec(s),
                which is not staleness
  I-004 029-ownership-coverage: file unit '…/coverage.rs' does not exist

$ spec-spine couple --base origin/main --head HEAD         → exit 2
spec-spine: index is stale: …, got 7 stale shard(s):
  blocking-diagnostics by-spec/029-ownership-coverage.json
```

**Classification: unresolved ownership, a validation failure.** Not freshness.
`check` says so in as many words; `couple` refuses the same state through the
shared `Error::Stale`, whose `Display` reads "index is stale" and whose
per-shard reason is `blocking-diagnostics`. The label and the exit code are
freshness's and the condition is not.

This is not new and not spec 005/023's: spec 079 §6 tracks it under
"`Error::Stale`'s wording at the freshness guard", and spec 079 D-4 records why
it was scoped out (a shared error variant, and a tier-1 claimed file). Nothing
is changed here.

The distinction to preserve: this is a **precondition refusal**, not a coupling
verdict. `C-001` and `C-002` never ran. No validation is weakened to reach the
later gate, and none should be.

### 4.4 What a `.crate` digest covers

The same three crates cut at two commits, `5b8c201a` (the candidate) and
`02c19e9e` (spec 117, which edits the root `Makefile` and adds one test file):

| Package | `.crate` digest | source-content digest |
|---|---|---|
| `spec-spine-types` | `41c9e457…` → `bb3a7eb6…`, **moved** | `e52fe660…` → `e52fe660…`, **identical** |
| `spec-spine-core` | `5fc5cbc8…` → `3f545ff4…` | `ce10bbf8…` → `cd752b79…`: 53 archive members became 54 |
| `spec-spine-cli` | `db74a9b1…` → `0a7685ad…` | `f1ce056d…` → `21517c90…` |

The source-content digest is defined for this record only: SHA-256 over sorted
`<path within the crate>\0<bytes>` for every archive member except
`.cargo_vcs_info.json`. It is not produced by cargo and not promised by any
spec.

The `types` row is the demonstration. The `cli` row was the surprise: diffing
the two archives shows the only content differences are `.cargo_vcs_info.json`
and **`Cargo.lock`**, which the packaged binary crate carries and which pins
the two sibling `.crate` SHA-256 values. The repository's own `Cargo.lock`
carries no such checksums, because the siblings are path dependencies there.

So a `.crate` digest is a digest of the archive: sources **and** generated
metadata. "A function of the commit, not of the sources" is false in its second
half. And the release consequence is concrete: all three archives must be cut
from the one revision that is actually published, or a `--locked` build from
the published CLI crate pins sibling checksums no published archive has.

### 4.5 The configuration casing

Documented exactly in the candidate record §8 and note 10 §7.3. The contract
test is retained, runs against the **packaged** crate, and is part of the
release pre-flight. Nothing was renamed and no alias was added.

## 5. Spec 116, reviewed independently

Reviewed against the question "does the exemption reach anything it should
not", by reading the implementation rather than the spec's claim about it.

The whole change to `lint.rs` is one boolean added to one condition:

```rust
let deferred = spec.implementation == Some(Implementation::Deferred);
if !retroactive && !withdrawn && !deferred && !has_ownership_edge(spec) { … L-001 … }
```

That condition's body is the single `L-001` warning for a spec that declares no
ownership edge, and the `!has_ownership_edge(spec)` conjunct is still there. So
the exemption can only ever suppress the no-territory warning, and only for a
spec that has no ownership edge at all. A deferred spec that names a unit
reaches the same conjunct as every other spec and is held to every diagnostic
about that unit; `L-012`, the unresolved-unit diagnostics and the coupling gate
are computed elsewhere and the diff touches none of them. **It exempts only the
intended no-territory deferred declaration.**

The ten contracts it was written for do declare nothing: specs 106 to 115 each
carry `implementation: deferred` and zero ownership edges, checked directly on
the branch. The re-arm is structural rather than remembered: nothing records
that a spec was once deferred, so the moment `implementation` moves the warning
applies again. `n-a` is deliberately excluded.

**Its adoption and release disposition stays separate.** Spec 116 is on
`specs/deferred-contracts-2026-09-21`, is not on either release branch, and is
not required by the release. Whether the deferred-contract corpus is adopted at
all is note 09 D-1's question, not this release's.

## 6. Handoff

### 6.1 The minimal producer-release branch

`release/0.22.0-candidate`, tip **`461be3fb`**. It contains the version bump,
spec 104, the finished migration note and the corrected candidate record, and
nothing else. Its gate status is §3's middle column: four steps pass, `couple`
refuses two paths.

`release/0.22.0-integration`, tip **`6dd96aa7`**, is the same plus spec 117.
Offered as an alternative endpoint; nothing in the release requires it.

### 6.2 Local pre-publication checks completed

- `cargo test --workspace --locked`, `cargo clippy --all-targets -D warnings`,
  `cargo fmt --all --check` on the candidate and on spec 117.
- The four gate steps on both, and on the integration branch, with the binary
  named in every run.
- `spec-spine verify 117`, 6 commands.
- `cargo package --workspace --locked`: run, fails at the CLI verify step for a
  registry reason; the control on the released `v0.21.0` tree succeeds.
- All three archives packaged, digested, and the CLI proven by an explicit
  isolated local build from the packaged sources.
- `./scripts/verify-packaged-producer.sh`: 37 assertions, 0 failures, against
  the packaged producer from outside the workspace, at `5b8c201a`.

### 6.3 Remaining decisions, none of them an agent's

1. **The coupling waiver.** Candidate record §5.1 proposes the smallest scope.
   No honest authority path exists; the candidate is blocked until a human
   decides. The line must be in the pull request body **at creation**.
2. **Spec 117**: merge on its own, take the integration branch, or neither.
3. **Every push, pull request, merge and ratify flip.** Specs 100, 101, 104 and
   117 are all `implementation: complete` and `status: draft`.
4. **CI, the determinism matrix and the merged-revision sweep**, all of which
   are unavailable until something is pushed and merged.
5. **Tag and publish**, per `docs/releasing.md`, cutting all three archives
   from the one published revision (§4.4).

### 6.4 Updated candidate artifacts for Statecraft

`docs/release-candidate-0.22.0.md` on `release/0.22.0-candidate` is the record;
§8 is the isolated-integration recipe. The two producer archives it names were
cut at `5b8c201a` and carry the digests in its §3. They must be re-cut at
whatever revision is published, and that re-cut is not optional (§4.4).

### 6.5 Separately reviewed optional branches

- **`117-…`**: new, green against its base, independently reviewable, depended
  on by nothing.
- **`103-…`**: session 1's, a deliberate sibling of the release branch. Its
  draft is on the candidate (filed with 100 to 102); its build is not, which is
  why `index coverage` on the candidate reports two of its units as `planned`.
- **`105-…`** and **`specs/deferred-contracts-2026-09-21`** (spec 116 and the
  ten deferred contracts): separate adoption questions, not release blockers.
  Spec 116 reviewed in §5.
