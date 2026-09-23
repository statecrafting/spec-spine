# Release preparation: `v0.23.0`, the expansion line

**Status: prepared, not cut. Not tagged and not published.** The proposed
producer revision is `97f82ee561efb656aeb3528d5ab9671bac8308bf` (`main` after
#322). It carries every crate, schema, manifest and fixture change for 0.23.0;
this record's own pull request changes documentation only. Tagging,
publishing and ratification are the owner's decisions (§8). The frozen 0.22.0
candidate is a separate line, unchanged at `f9fa6a8f` (its record is
`docs/release-candidate-0.22.0.md`, §13 to §15).

## 1. Two lines, two identities

Measured with `scripts/reader-identity.sh` (spec 124 §3.4) on clean checkouts
at each revision, each binary being that checkout's current build. Schema
lines are what each binary **emits**, not what its source says.

| | Frozen 0.22.0 candidate | 0.23.0 line |
|---|---|---|
| revision | `f9fa6a8f56b82c8d97cf2803bc31838a0b455a21` | `97f82ee561efb656aeb3528d5ab9671bac8308bf` |
| answers `--version` | `spec-spine 0.22.0` | `spec-spine 0.23.0` |
| executable SHA-256 (local build, aarch64-apple-darwin) | `e592562e939135638eb9429497443605d9af0dbee9ce943257e98ba8a443fa4b` | `ec170c4c8c19f70fce5da0ca8feb18f5b44d25a4ff96a561978a7a0a162ce026` |
| registry `specVersion` | `1.3.0` | `1.6.0` |
| read documents | `0.1.0` | `0.7.0` |
| verdict envelope | `0.5.0` | `0.6.0` |
| index | `1.1.0` | `1.1.0` |
| this repository's `required_version` | `>=0.17.0` | `>=0.23.0` |

The executable digests identify these two local builds. They are not release
artifacts and are not reproducible across machines. Release archives get their
own digests from `release.yml`, recorded when a release is cut.

Until 0.23.0 is cut, every development build of `main` also answers `0.23.0`.
The version names the line; the identity record names the build.

## 2. What the line contains

Merged after the 0.22.0 freeze, one pull request each, squash commits in full.

**Expansion** (new capability, opportunity-led under D-7):

| Spec | Pull request | Commit |
|---|---|---|
| 103, verifier fixtures | #301 | `c638093216f99221f33d2a76975c242c65b8802f` |
| 102, a ready spec carries its status | #307 | `75a998f7122f32392c38c8b052c8e686511e1230` |
| 106, obligations and section digests | #308 | `088d6d4b711659ce563f496d6c5c421ac2f9a84f` |
| 107, context closures | #309 | `6e123d2e4632ef2feb8d65e8e8843ff8bf545482` |
| 105, governed scope enabled here | #311 | `b7c13452a3cdcc924ddc7cf720810b571325c803` |
| 109, impact and conflict | #312 | `0ca3001d6b5cc36bec14fde8b2852360572fa63c` |
| 110, digest-pinned interface references | #313 | `8c6c4d73d25d7c586fee2b6a49d0b11b8db302c8` |
| 113, a waiver has a declared lifecycle | #318 | `0bf9ff780472bc39ad05ee5b9a95012c34d82f85` |
| 108, a work scope is declared | #320 | `2e2993569a6a604f7b239f06bb09ecd979bd16b9` |

**Corrective** (this repository's own tooling and evidence, or a defect):

| Spec | Pull request | Commit |
|---|---|---|
| 102 carries 053's acceptance, held by 087 | #314 | `73182329f548c7c54572d4bee80260c212581c43` |
| 105 carries 078's acceptance | #315 | `2fff1e494cabc40746ccc77a8610f67ce0807175` |
| 122, a commit refuses an unresolved merge or unformatted Rust | #310 | `45becbbeb960dc95df3c201cd027c366b15693cd` |
| 123, a startup verdict names its reader | #317 | `980dae1b76281e6aa5591205711e7d8233ff72f9` |
| 125, a forwarded line arrives whole | #321 | `d233fcff7a6b64781b7cdcd56c5018481114157b` |
| 123 corrected: the reader is aged by its compiled inputs | #319 | `1da44fa9e9f951d64fb41561753a3b4e3cab8668` |
| 124, the expansion line carries its own version | #322 | `97f82ee561efb656aeb3528d5ab9671bac8308bf` |

Also on `main` and not engine changes: D-7 (#304), the 0.22.0 re-freeze record
(#306), the consumer integration record (#316) and the ratification of 118 to
121 (#305, `3b67b63d48965fefce6732d1cc865d861ee9928b`).

## 3. The session warning of 2026-09-22 and 2026-09-23

Recorded in full in spec 123 §1.1. In short:

- **Hook:** this repository's own `.claude/settings.json` `SessionStart` (and
  `Stop`). No managed settings, no global `SessionStart` hook, and the
  untracked `.codex/hooks.json` is not a Claude Code registration.
- **Directory:** `CLAUDE_PROJECT_DIR` was the main checkout, at `6e123d2e` and
  then `0ca3001d`. `$SPEC_SPINE_BIN` was unset.
- **Selected reader:** the main checkout's `target/release/spec-spine`
  (resolver rule 2), built at `3d4f3902` with the candidate's engine, left in
  place while the checkout moved past spec 106's frontmatter. `PATH`
  (`~/.cargo/bin`, then 0.20.0) was never consulted. Attributing the warning to
  it was wrong, and replacing that binary did not fix it: the same session also
  rebuilt the in-tree reader, which is what cleared the banner.
- **Fix:** the hooks name their reader, and an in-tree build older than any
  input it was compiled from (cargo's dep-info) is reported `NOT JUDGED`, with
  what it said kept (#317, #319). A released reader older than the corpus now
  refuses by name through the floor (#322). Nothing builds, regenerates or
  installs in a hook.

Two defects were found and corrected on the way. They are preserved here as
evidence:

- As merged in #317, the age check read every file under `crates/`, so
  #317's own test-only change made the banner say `NOT JUDGED` on the main
  checkout with a rebuild that could not clear it. #319 narrowed it to the
  compiled inputs (123 D-5, D-6).
- The post-merge `Acceptance` sweep at `0bf9ff78` (#318) failed on attempt 1
  (spec 043) and on its one re-run (spec 044): `verify_streams`'
  flood test lost 1 of 8192 lines. #318 did not cause it; `verify`'s two
  drains could splice a line split by a pipe read. #321 fixed the forwarder
  (spec 125). The failed run `35822753272` keeps both attempts and their
  artifacts.

## 4. Checks at `97f82ee5`

**Local, clean detached worktree at the revision, binary built from it:**

| Check | Result |
|---|---|
| `cargo build --release --locked -p spec-spine-cli` | exit 0 |
| `make gate` (check, lint, coverage, couple) | exit 0 |
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | exit 0 |
| `cargo test --workspace --locked --no-fail-fast` | exit 0 (92 test result blocks, 0 failed) |
| `cargo test -p spec-spine-core --no-default-features --locked` | exit 0 |
| `scripts/bump_version.py --check 0.23.0` | exit 0 |
| `scripts/verify-packaged-producer.sh` (spec 104) | exit 0, "ALL PACKAGED-PRODUCER ACCEPTANCE CHECKS PASSED" |
| `scripts/verify-sweep.sh --rev 97f82ee5 --release` (whole corpus, its own worktree, binary built from the revision) | **release verdict clean**: `passed=78 failed=0 not-declared=0 exempt=43 not-run=0`, 121 specs accounted for |

Package digests from the packaged-producer run (`cargo package`, source commit
`97f82ee5`; the archive records its commit, so another commit's archive
differs even with identical sources):

- `spec-spine-core-0.23.0.crate` `d494dbb9d642a754f8490cc185504c3dcb1a0de167f651866688c389421a96f3`
- `spec-spine-types-0.23.0.crate` `5d9cd7729c315401dc3c1f0a52549ae8764d5c6074a8c669b2256cab2d162ca8`

**CI on `main`, post-merge, each merge read before the next:**

| Commit | `CI` | `Acceptance` (scoped push leg) |
|---|---|---|
| `980dae1b` (#317) | `35821856398` success | `35821856142` success |
| `0bf9ff78` (#318) | `35822753516` success | `35822753272` **failure**, attempts 1 and 2 (§3) |
| `d233fcff` (#321) | `35826380850` success | `35826380722` success (swept 043, 090, 125 and two more) |
| `1da44fa9` (#319) | `35826897899` success | `35826897654` success |
| `2e299356` (#320) | `35827354998` success | `35827354549` success (35 passed) |
| `97f82ee5` (#322) | `35828364297` success | `35828363990` success (12 passed, 4 exempt) |

**Consumer, packaged:** `docs/examples/expansion-consumer/run.sh` from this
record's branch (crates identical to `97f82ee5`). Every contract held,
including the new 108, 113 and composed checks
(`docs/consumer-integration-expansion.md` §9).

**Registry-backed: none.** Nothing is published. `cargo package --workspace`
cannot verify the CLI crate until its 0.23.0 siblings are on crates.io, which
is why the packaged checks above package `spec-spine-types` and
`spec-spine-core` only. Registry-backed checks (crates.io, npm, PyPI,
`uvx`/`npx` installs) are owed at publication.

## 5. Schema compatibility

Every move since the candidate is additive (MINOR) and has one owner.

| Axis | 0.22.0 | 0.23.0 | Steps |
|---|---|---|---|
| registry | `1.3.0` | `1.6.0` | 106 `1.4.0`, 109 `1.5.0`, 110 `1.6.0` |
| read documents | `0.1.0` | `0.7.0` | 102 `0.2.0`, 106 `0.3.0`, 107 `0.4.0`, 109 `0.5.0`, 110 `0.6.0`, 108 `0.7.0` |
| verdict envelope | `0.5.0` | `0.6.0` | 113 (`couple`'s `waivers`, omitted when no waiver is declared) |
| verifier fixture set | none | `0.1.0` | 103 |
| index, delta, snapshot, attestation, build-meta, config | unchanged | unchanged | none |

**Migration for a consumer:** none required to keep working. A loader pinned
to a MAJOR keeps loading. A consumer that wants the new members reads them. A
corpus that starts declaring obligations, impacts or interface references
should raise its own `required_version`, so an older reader refuses it by
name.

## 6. What is deliberately not in 0.23.0

Reserved ordinals 111, 112, 114, 115 and 116 (design note 09 §11). No producer
boundary moved: a scope locks and permits nothing, a waiver lifecycle counts
and consumes nothing, and interface references fetch nothing.

## 7. Handoff

The consumer surface, with every contract and exit code, is
`docs/consumer-integration-expansion.md`; its §11 is written for Statecraft.

## 8. Decisions for the owner, kept apart

Nothing below has been done. Merge authority covers none of it.

### 8.1 Ratification: corrective

Specs that repair this repository's tooling or a defect. Each is `draft` /
`complete` on `main`. **Prepared as draft #324**, status-only; approving it
means marking it ready and merging it:

| Spec | What approving it establishes |
|---|---|
| 122 | the commit hook refuses unresolved merges and unformatted Rust |
| 123 | session hooks name their reader; an older in-tree reader is not believed; one wording rule of 093 amended |
| 124 | a version names one behavior; the floor moves with the version; the identity record |
| 125 | `verify` forwards whole lines |

### 8.2 Ratification: expansion

New capability, each `draft` / `complete` on `main`. **Prepared as draft
#325**, status-only; a subset can be approved by dropping specs from it:

| Spec | Contract |
|---|---|
| 102 | `status` on a ready entry (and 102's carry of 053's acceptance, via 087) |
| 103 | the verifier fixture set as a published artifact |
| 105 | `governed_scope` enabled here (and its carry of 078's acceptance) |
| 106 | obligations and section digests |
| 107 | context closures |
| 108 | work scopes |
| 109 | impact and conflict declarations |
| 110 | digest-pinned interface references |
| 113 | waiver lifecycle |

Ratifying any of them flips `status` only (a status-only pull request, one
spec or a batch as the owner prefers, as #305 did for 118 to 121). Doing so
before cutting 0.23.0 puts the approved states in the release. Doing so after
leaves them `draft` in 0.23.0, which is how 0.22.0 would have shipped 118 to
121 before #305.

### 8.3 Publication

1. **0.22.0** (unchanged decision): whether to tag the frozen candidate
   `f9fa6a8f` as recorded in the 0.22.0 record §13.9 and §14.7. Tagging it now
   ships a release whose engine predates everything in §2.
2. **0.23.0:** whether and when to cut it, at `97f82ee5` or a later `main`.
   Pushing `v0.23.0` starts publication to crates.io, npm and PyPI (with no
   further approval step). If the owner chooses a later revision, §4's checks
   are owed again there, and this record is superseded rather than edited.
3. **Order:** if both are published, 0.22.0 first, so the version sequence on
   every registry matches the source order.

This session installed and replaced no `PATH` binary. Repository-local
builds were used throughout.
