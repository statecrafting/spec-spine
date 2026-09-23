# Release record: `v0.24.0`

**Status: published 2026-09-23** to crates.io, npm, PyPI and GitHub Releases
from source revision `a812f72dbef80d25c4027d85f267db9ba657506e`. The frozen
0.22.0 candidate (`f9fa6a8f`) stays unpublished and is not retagged or reused.

## 1. Identity

| | |
|---|---|
| tag | `v0.24.0`, signed annotated tag object `95abc8b32c89275aa6a54e67c344ab856cd6aee7` |
| source revision | `a812f72dbef80d25c4027d85f267db9ba657506e`, `main` after #334 |
| release workflow | `release.yml` run `35885255765`, every job `success` |
| registry schema (`specVersion`) | `1.8.0` |
| read documents | `0.8.0` |
| verdict envelope | `0.6.0` |
| index | `1.1.0` |
| verifier fixture set | `0.1.0` (cases regenerated at 0.24.0) |
| `[meta] required_version` here | `>=0.24.0` |

**Why this revision.** `a812f72d` is the first revision on `main` carrying
every spec of the line as built and approved, with nothing after it:

| PR | Merge commit | What |
|---|---|---|
| #327 | `bcfdec32` | 116 built |
| #328 | `e8a6cc70` | 112 built |
| #329 | `51269cc8` | 114 built, registry `1.7.0` |
| #331 | `796bbed1` | `main` moves to 0.24.0 (spec 124 D-3), floor `>=0.24.0`, fixtures regenerated |
| #332 | `18199564` | 111 built, registry `1.8.0`, read `0.8.0` |
| #333 | `9dc0d31e` | 111, 112, 114, 115, 116 ratified, status only |
| #334 | `a812f72d` | the 0.24.0 consumer and its registry-backed harness (documentation only) |

**The version identity was corrected before the feature it describes.** Spec
124 requires that once `main` differs from a published release in any engine
source or schema, `main`'s version moves off it. 112, 114 and 116 had already
changed the engine beyond published 0.23.0, so #331 moved `main` to 0.24.0
before 111 integrated. Four merged revisions carried an engine differing from
published 0.23.0 while still answering 0.23.0: `bcfdec32` (116's `lint`
change), `e8a6cc70`, `51269cc8` (114) and `303ddff5` (documentation on top).
None was tagged or published; `796bbed1` is the first revision after them,
and every revision since answers 0.24.0.

## 2. What it contains

| Spec | Disposition | What a consumer sees |
|---|---|---|
| 111 a move is a reviewed mapping | approved, complete | `moves` frontmatter; `registry moves [<path>] [--json]` and `query_json` `op: "moves"` with outcomes `unmapped`, `resolved`, `ambiguous`, `cycle`; V-040 (shape, path grammar), V-041 (dangling `answered_by`, warning), L-015 (declared `to` missing), L-016 (`removed` path still present). Informational only: no rename inference, no ownership transfer, no clearance of a deletion; `couple`, `coverage` and `index` do not read it |
| 112 typed overlays ride the existing seam | approved, complete | no engine change; `docs/overlay-contract.md` and tests fix the spec 012 seam's guarantees |
| 114 authoring adapters and intent are separated | approved, complete | optional `intent: {goal, non_goals}` recorded as `{goal, nonGoals}`; malformed shapes refused at compile; no verdict reads it |
| 115 bindings are designed, not shipped | approved, `n-a` | a policy record: no binding code in this repository, by mandate; nothing was built to manufacture an implementation result |
| 116 a deferred contract claims nothing | approved, complete | `L-001` is no longer raised for an `implementation: deferred` spec that claims nothing; `pending` and `n-a` are not exempt |
| 124 the expansion line carries its own version | approved, complete | the 0.24.0 package version and floor |

**Review of 111.** Recorded in #332: a first independent review of the build
found a recursive walk that could overflow the stack on a long chain and
expanded a reconverging split once per path (fixed, 111 D-11); a second
independent read-only review mutation-tested that fix (the 5,000-hop test
aborts against the recursive walk, the depth-16 split reports 131,070 hops
instead of 62) and found `lint` statting move paths outside the tree (fixed,
111 D-12, with a test that fails on the reverted file); a re-review approved.

### Compatibility and migration

- **`moves` and `intent` are now typed frontmatter keys.** A corpus that
  declared either in `[frontmatter] extra_known_keys` as its own overlay, with
  another shape, was accepted by 0.23.0 and is refused by 0.24.0 with `V-002`
  (checked with both binaries: `intent: "free text"` fails as "expected struct
  IntentDeclaration", `moves: "free text"` as "expected a sequence"). Rename
  the overlay key. No adopter corpus found on this machine declares either.
- **Rust struct literals.** `Frontmatter` and `SpecRecord` gained public
  fields (`intent`, `moves`); a consumer constructing either with a struct
  literal must add them. Consumers of the JSON facade are unaffected.
- **Schema moves are MINOR.** Registry `1.6.0` to `1.8.0` (114 `intent`, 111
  `moves`), read documents `0.7.0` to `0.8.0` (the move lookup). A 0.23.0
  reader meeting a 1.8.0 member refuses with exit 3 rather than guessing.
- **What can move a verdict.** Only 116, and only toward passing: a deferred
  spec claiming nothing no longer raises `L-001`, so `lint --fail-on-warn` can
  turn green where it was red. Nothing in 0.24.0 can turn a passing gate red
  on an unchanged corpus other than the `extra_known_keys` collision above.
- **The scaffold.** `scaffold_init_json` emits the same seven files; the one
  byte difference against 0.23.0 is the commented sample
  `# required_version = "0.24.0"` in the scaffolded `spec-spine.toml`, which
  tracks the package version.

## 3. Qualification at `a812f72d`

Clean detached worktree at the revision (tracked tree clean), binary built
from it. Every step's exit code is recorded; nothing was retried.

| Check | Result |
|---|---|
| `cargo build --release --locked -p spec-spine-cli` | exit 0 |
| `scripts/reader-identity.sh` | current build of `a812f72d`; registry 1.8.0, index 1.1.0, read 0.8.0, verdict 0.6.0; executable SHA-256 `0a9f7e0260bb3684edbcffe87119b81d7556fb6615ce51b1cdc62f612bb9eef0` (local aarch64-apple-darwin build) |
| `make gate` (check, lint, coverage 175/175, couple) | exit 0 |
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | exit 0 |
| `cargo test --workspace --locked --no-fail-fast` | exit 0 (1253 passed, 0 failed) |
| `cargo test -p spec-spine-core --no-default-features --locked` | exit 0 |
| `scripts/bump_version.py --check 0.24.0` | exit 0 |
| `cargo package -p spec-spine-types -p spec-spine-core --locked` | exit 0 |
| `scripts/verify-packaged-producer.sh` | exit 0; `types` `18c0643fe1e9a3d88aa806c31f0732e47c3e8f74d3a3cf1904b9fd36e3ff02b8`, `core` `17c82ac7ff58206ce607ba7f742d4f231c23e5a90163e396d19e4bfb9e1d5b31` |
| `docs/examples/expansion-consumer/run.sh` (packaged) | exit 0, every contract held |
| `npm test`, `npm run smoke` | exit 0 |
| `py` unit tests, `py/scripts/smoke_test.sh` | exit 0 |
| `scripts/verify-sweep.sh --rev a812f72d --release` | **release verdict clean**: 126 accounted, `passed=83 failed=0 not-declared=0 exempt=43 not-run=0`, pending 0 |

**The 43 exemptions** are exactly specs 000 to 042, listed by the sweep's own
report. Their source is `scripts/verify-sweep.sh` at the revision,
`LEDGER_CLOSED_AT=43`: the closed legacy ledger of specs filed before spec 043
built `verify`. Every spec from 043 through 125 has declared acceptance and
passed; none is exempt, pending, failed or unrun. An earlier qualification
described as "clean with 1 pending" does not carry over to this verdict: at
`a812f72d` the pending count is 0.

**CI on `a812f72d`.** `CI` run `35848811002` success, including the
determinism gate's byte-identical shard trees across four triples;
`Acceptance` run `35848810769` (push) and `35856511512` (schedule) success.

**The CLI crate before publication.** As for 0.23.0, `cargo package` cannot
verify `spec-spine-cli` until its siblings exist on crates.io; it was verified
by `cargo publish` in the release run against the just-published siblings. No
`--no-verify` was used.

## 4. Published artifacts

Before tagging, no registry held 0.24.0 (crates.io `max_version` 0.23.0 for all
three crates, npm 404, PyPI 404, no GitHub Release), so nothing was
overwritten or confused with an earlier attempt.

### crates.io

| Crate | Checksum (SHA-256 of the `.crate`) |
|---|---|
| `spec-spine-types` 0.24.0 | `18c0643fe1e9a3d88aa806c31f0732e47c3e8f74d3a3cf1904b9fd36e3ff02b8` |
| `spec-spine-core` 0.24.0 | `17c82ac7ff58206ce607ba7f742d4f231c23e5a90163e396d19e4bfb9e1d5b31` |
| `spec-spine-cli` 0.24.0 | `b5e8fb2743285343e12a0090bf028134c4c874744f9739fa3bc3b5c2cea46b01` |

The `types` and `core` checksums equal the local packages at `a812f72d`
(§3): the qualified bytes are the published bytes. `core` depends on
`spec-spine-types ^0.24.0`; `cli` on `spec-spine-core ^0.24.0` and
`spec-spine-types ^0.24.0`. None is yanked.

### GitHub Release assets

Each archive's `.sha256` sidecar matches the download, each has a CycloneDX
SBOM (282 components; 308 for Windows), and `gh attestation verify` passes for
all five, each naming source commit `a812f72d` and `refs/tags/v0.24.0`. The
release is `latest`.

| Archive | SHA-256 |
|---|---|
| `spec-spine-v0.24.0-aarch64-apple-darwin.tar.gz` | `90c3cab173175c2759a3b17355564a3d6e213e41d01a16a51b783a1b9f725d31` |
| `spec-spine-v0.24.0-x86_64-apple-darwin.tar.gz` | `8161edcca39e857b69fe61b7a25e141fff59088ae59571c0019d9c4cbb42025b` |
| `spec-spine-v0.24.0-aarch64-unknown-linux-gnu.tar.gz` | `3ed51bde31b0b0b1f5c5cd9a6d3e83146301db31517545098b443b549dcde8c4` |
| `spec-spine-v0.24.0-x86_64-unknown-linux-gnu.tar.gz` | `ef879c0701688cd1395e8cb1b208b2005ec53f5b52213d085e797bde4e5dc692` |
| `spec-spine-v0.24.0-x86_64-pc-windows-msvc.zip` | `ec940648d9ebcaaf3a1a7e612270951c07ddc5c8b136f71731bc11c9c4456eae` |

Archive digests identify the release archives; they are not source-content
identities, and the local build's executable digest (§3) is a different
artifact again.

### npm

`spec-spine@0.24.0` (`latest`), integrity
`sha512-4/+Uj33BQwF+hXgr2VGd04CBT3GokF1PjX9jRv95MTqmm3FvI3pbycCF/N0MA8GgGxVM9JYt++GDjxv4ZWFI7g==`,
with its five platform packages at 0.24.0, each also `latest`:

| Package | Integrity |
|---|---|
| `@spec-spine/cli-darwin-arm64` | `sha512-H4IOsr1SNyYspa7E9CUmcNoqrQ+1deOZ+6+atd2+bqTalgLSOxtgZtPoBP5Ivt9jF2iJqlTy94nkXtKsu4h3RQ==` |
| `@spec-spine/cli-darwin-x64` | `sha512-iJhqbvsybM0tMeW0ACM0FYmXmoxQWtbl1Zlv9raYCV6C7FlNdIiJh8R5D2AsuVkTUh3rXLWvo6t3hjhrpcQo3w==` |
| `@spec-spine/cli-linux-arm64` | `sha512-GZHhECIb8RoMPGQ0bhYi9Gm2q8w99ImdevlIOKt7LULN3GhBngOMKuruLykWbbzZi0vtPIfh+7vdN+BMPzAPsQ==` |
| `@spec-spine/cli-linux-x64` | `sha512-RkCvtjpKkRLz7FAwFnDkJq7QoolYOE4PhTHPg0M1xLn2WBOeYcxmw8TwYFKUTzZ9HsBl1KB93Vvj/csStsHYWg==` |
| `@spec-spine/cli-win32-x64` | `sha512-4sEkSvSkPgDCs0WaTMKoETzqdWA0zrS5msbr7dP2+eOKfam40JYHqz86FzTWNMzJNoPht1fIGyrAeh94h0PA/Q==` |

Directly after the job, `npm view` answered 404 for four of the six while the
publish log showed all six `>> published`; a few minutes later all six
answered. That was registry propagation, not a partial publish.

### PyPI

`spec-spine` 0.24.0, five platform wheels and an sdist, each with a PEP 740
publish attestation. The SHA-256 digests PyPI reports equal the ones attested
in the publish job:

| File | SHA-256 |
|---|---|
| `spec_spine-0.24.0-py3-none-macosx_11_0_arm64.whl` | `e0f42a11fa52c86c14516f2d1dbb30d3f7fea4e02f5d1f0b3ae85ac51a817dc1` |
| `spec_spine-0.24.0-py3-none-macosx_10_12_x86_64.whl` | `cabfbcf365acbefb0fbb01d6eda0621094fcf4480cb96d6674e216df73e63e4a` |
| `spec_spine-0.24.0-py3-none-manylinux_2_17_aarch64.whl` | `2ae702d8940656c40421a38c6098cfdbc0cb4541b2e19023d37a66da5c2df416` |
| `spec_spine-0.24.0-py3-none-manylinux_2_17_x86_64.whl` | `8117074967b9e7aa6017fb18b4036c4d98b4ab173fd5f5f171a71b0d6208d6b7` |
| `spec_spine-0.24.0-py3-none-win_amd64.whl` | `2b3caf52b0d4dedbe707b788bc6ce2f4920fe3fe3250ea10c09e9cf0f197a323` |
| `spec_spine-0.24.0.tar.gz` | `cd9218357a0c43ff67b79dd6f09f9685fc2131d985a740de8d4757b02b2f9936` |

**Index visibility.** At 16:02 UTC the JSON project API listed all six files
while `https://pypi.org/simple/spec-spine/` did not list 0.24.0 in either
representation (JSON form, compressed, `X-PyPI-Last-Serial` 41362461; HTML
41270277): the same cached-variant pattern as 0.23.0's §6. At 16:05 UTC the
JSON form requested as uv and pip request it reported serial 41375554 and
listed 0.24.0, and the ordinary commands below then resolved it. No
republish, metadata change or bypass was involved.

## 5. External consumer qualification

`docs/examples/expansion-consumer/registry.sh 0.24.0 0.23.0`, run after
publication in a new scratch directory with fresh Cargo, npm, uv and pip
caches, no path, git or patch dependency and no workspace: every step
`exit=0`, `ALL=0`.

| Check | Result |
|---|---|
| `cargo install spec-spine-cli --version =0.24.0 --locked` | answers `spec-spine 0.24.0`; reader identity registry 1.8.0, index 1.1.0, read 0.8.0, verdict 0.6.0 |
| library consumer against `spec-spine-core = "=0.24.0"` from crates.io, replaying the fixture set inside the published crate | 13 `ok:` lines: 102, 106, 109, 107, 110, 103 (11 of 11 cases), 108, 113, the composed flow, 111, 112, 114, 116 |
| fixture binding | 10 of 11 cases name `toolVersion` 0.24.0; the eleventh is `tool-version-changed`, whose point is a different producer version (the harness now requires exactly that) |
| lockfile | `spec-spine-core` and `spec-spine-types` 0.24.0 from `registry+https://github.com/rust-lang/crates.io-index`; no `path+` or `git+` source |
| Statecraft's shape: `default-features = false`, `scaffold_init_json` only | 7 governance files; pure and deterministic; no `AGENTS.md`, `CLAUDE.md`, `Makefile`, `.claude/` or `.github/`; a camelCase config key refused; no tree-sitter in the lockfile |
| CLI floor | a corpus requiring 99.0.0 exits 3; one requiring 0.24.0 compiles |
| insufficient producer | the published 0.23.0 CLI on a corpus requiring `>=0.24.0` exits 3 naming `>=0.24.0` |
| `npx -y spec-spine@0.24.0 --version`, `npm i -D spec-spine@0.24.0` | `spec-spine 0.24.0`; `dist-tags.latest` is 0.24.0 |
| `uvx spec-spine@0.24.0 --version` (uv 0.10.12, fresh cache, no configuration) | `spec-spine 0.24.0` |
| `pip install spec-spine==0.24.0` (fresh venv, no cache) | `spec-spine 0.24.0` |
| `curl -fsSL .../main/install.sh \| sh` with `SPEC_SPINE_REQUIRE_ATTESTATION=1` | checksum and provenance verified; installs `spec-spine 0.24.0` |

**What the 0.24.0 sections assert.** 111: unmapped, a two-hop chain across two
specs, a split, an ambiguous disagreement naming both specs, a cycle, the
sorted flattened map; `registry moves <path> --json` equal to the facade's
answer for all five outcomes, with exit 0 for `unmapped`/`resolved` and 1 for
`ambiguous`/`cycle` (111 §3.4); a declared mapping leaving an unauthored
deletion's `C-001` verdict identical; V-040 refusing an escaping path while
`lint` stays in the tree; V-041 absent for a resolving short id and present,
as a warning, for a dangling one; L-015 for a missing `to`; L-016 for a
present `removed` path and not otherwise. 112: an overlay transported with
sorted keys, refused undeclared with V-002, the content hash moving for the
declaring spec only, no verdict change. 114: valid intent recorded as
`{goal, nonGoals}`, absence is no member, four malformed shapes refused, no
verdict change. 116: deferred exempt from L-001, `pending` and `n-a` not.

**Negative control.** `registry.sh --control 0.23.0 111,114,116` against the
published 0.23.0 library and CLI: each section, run alone after the nine
earlier contracts pass, fails on its own (111 at `compile`, which 0.23.0
refuses on `moves`; 114 at the intent record; 116 at "deferred is exempt").
Because 0.23.0 refuses `moves` outright, the control stops 111 before its
newer sub-assertions; those carry their own contrasts inside the section
(V-041 with and without a dangling reference, L-016 with and without the
removed path, the verdict with and without the mapping). 112 changed no
engine source, so no published release can serve as its control; its
section's declared/undeclared and with/without contrasts are what make it
able to fail. The first local run of the extended section failed because the
new CLI check expected exit 0 for every outcome; spec 111 prescribes exit 1
for `ambiguous` and `cycle`, so the harness was corrected, not the engine.
After review of #335 the fixture check also requires `tool-version-changed`
to exist (an absent file had let `! grep -q` succeed), and `cli_json` names
non-JSON stdout; both were rerun against the same published crates and
scratch (fixture binding exit 0; the rebuilt consumer, offline against the
same registry lockfile, 13 `ok:` lines).

## 6. The 0.23.0 `uvx` path, rechecked

`docs/release-0.23.0.md` §6 closed the item at 09:40 UTC. Rechecked once more
at 15:58 UTC with fresh caches: `uvx spec-spine@0.23.0 --version` (uv 0.10.12)
and `pip install spec-spine==0.23.0` both answer `spec-spine 0.23.0`. The
original failed observation stays in that record as it was made.

## 7. Open at the time of writing

Nothing in this release. Statecraft adopted published 0.23.0 in
`statecrafting/statecraft-cli` `49370fbe` (#70); moving it to 0.24.0 is
Statecraft's choice, described in `docs/consumer-integration-expansion.md`
§12.
