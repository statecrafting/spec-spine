# Release record: `v0.23.0`

**Status: published 2026-09-23** to crates.io, npm, PyPI and GitHub Releases
from source revision `d2bb47634404b874ca36cf8bf75b4a31a70328f7`. This record
supersedes the preparation record (`docs/release-candidate-0.23.0.md`), which
proposed `97f82ee5` and is left as written. The frozen 0.22.0 candidate
(`f9fa6a8f`, `docs/release-candidate-0.22.0.md`) was never published and is
not retagged, rewritten or reused.

## 1. Identity

| | |
|---|---|
| tag | `v0.23.0`, signed annotated tag object `69ba6f9959cb786efbd8f8e8cddd0cad178c411b` |
| source revision | `d2bb47634404b874ca36cf8bf75b4a31a70328f7`, `main` after #325 |
| release workflow | `release.yml` run `35838082358`, every job `success` |
| registry schema (`specVersion`) | `1.6.0` |
| read documents | `0.7.0` |
| verdict envelope | `0.6.0` |
| index | `1.1.0` |
| verifier fixture set | `0.1.0` |
| `[meta] required_version` here | `>=0.23.0` |

**Why this revision.** The owner approved ratifying the thirteen specs of the
line (#324, #325) and publishing 0.23.0 without first publishing 0.22.0.
`d2bb4763` is `main` directly after the second ratification: the preparation
record's `97f82ee5`, plus documentation (#323) and the two status-only
ratifications. So the release carries every spec of the line as `approved`.

**Why 0.22.0 was not published first.** Nothing required it. The release
workflow has no ordering between versions, the registries accept any unused
version, and the one established consumer, the Statecraft CLI, pins
`spec-spine-core =0.21.0` (library) and `spec-spine =0.20.0` (CLI) and depends
on no 0.22.0 artifact. The 0.22.0 decision stays with the owner. If it is ever
published, note that `release.yml` publishes npm under the default `latest`
dist-tag, so a later 0.22.0 publish would move `latest` backwards unless the
tag is overridden.

## 2. What it contains

Everything on the 0.22.0 candidate (the realignment: no `init`, no kit,
renumbered ids, specs 100 and 101, the two verdict-affecting coupling
changes), followed by the expansion and corrective line recorded in the
preparation record §2: specs 102, 103, 105 to 110, 113 (expansion) and 122 to
125 (corrective). All thirteen were ratified before the tag: #324
(`d95e1f0ef4dfe524441ca5ad72cf7c199d819a8d`, specs 122 to 125) and #325
(`d2bb47634404b874ca36cf8bf75b4a31a70328f7`, specs 102, 103, 105 to 110,
113). Both were status-only (status lines and their shards), checked against
`main` before merging.

Upgrading from 0.21.0 is described in `docs/adopter-migration.md` (written for
the 0.22.0 tag; it applies to this one) and in the GitHub Release notes.

## 3. Qualification at `d2bb4763`

Clean detached worktree at the revision, binary built from it. Every step's
exit code is recorded; nothing was retried.

| Check | Result |
|---|---|
| `cargo build --release --locked -p spec-spine-cli` | exit 0 |
| `scripts/reader-identity.sh` | current build of `d2bb4763`; registry 1.6.0, index 1.1.0, read 0.7.0, verdict 0.6.0; executable SHA-256 `ec170c4c8c19f70fce5da0ca8feb18f5b44d25a4ff96a561978a7a0a162ce026` (local aarch64-apple-darwin build, identical to the `97f82ee5` build: no compiled input changed) |
| `make gate` (check, lint, coverage, couple) | exit 0 |
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | exit 0 |
| `cargo test --workspace --locked --no-fail-fast` | exit 0 (1197 passed, 0 failed) |
| `cargo test -p spec-spine-core --no-default-features --locked` | exit 0 |
| `scripts/bump_version.py --check 0.23.0` | exit 0 |
| `cargo package -p spec-spine-types -p spec-spine-core --locked` | exit 0 |
| `scripts/verify-packaged-producer.sh` | exit 0, all producer checks passed |
| `docs/examples/expansion-consumer/run.sh` (packaged) | exit 0, every contract held |
| `npm test`, `npm run smoke` | exit 0 |
| `py` unit tests, `py/scripts/smoke_test.sh` | exit 0 |
| `scripts/verify-sweep.sh --rev d2bb4763 --release` | **release verdict clean**: 121 accounted, `passed=78 failed=0 not-declared=0 exempt=43 not-run=0`, pending 0 |

**The 43 exemptions** are exactly specs 000 to 042: the sweep's closed legacy
ledger (`LEDGER_CLOSED_AT=43`), specs filed before spec 043 built `verify`, so
no acceptance was declarable for them. The ledger refuses any entry at or above
043 and only shrinks. No spec from 043 on is exempt.

**CI on `d2bb4763`** (post-merge push): `CI` run `35834750124` success,
including the determinism gate's byte-identical shard trees across four
triples; `Acceptance` run `35834749692` success.

**The CLI crate before publication.** `cargo package` cannot verify
`spec-spine-cli` until its two siblings exist on crates.io, so the local check
packaged `types` and `core` only. The CLI crate was verified by `cargo
publish` itself in the release run, against the just-published siblings. No
`--no-verify` was used anywhere.

## 4. Published artifacts

### crates.io

| Crate | Checksum (SHA-256 of the `.crate`) |
|---|---|
| `spec-spine-types` 0.23.0 | `dcd35073ecd95b9aa21b0173e0351fc44398d2a6019d6de73948d5287b319dc4` |
| `spec-spine-core` 0.23.0 | `3dca8f6819d7951110757fbafba31897a3ed6dad516a85599e05e334f6b3e492` |
| `spec-spine-cli` 0.23.0 | `6b0e780069a88d1a9f1ecf9d0d49cab1308bef0d44fbae63d1e5f3322975ca91` |

The `types` and `core` checksums equal the digests `verify-packaged-producer.sh`
printed for its local packages at `d2bb4763`: the qualified bytes are the
published bytes. Published in dependency order (types, core, cli).

### GitHub Release assets

Each archive has a `.sha256` sidecar (all five verified against the download),
a CycloneDX SBOM (282 components; 308 for Windows), and a build-provenance
attestation. `gh attestation verify` passes for all five, each naming source
commit `d2bb4763` and workflow ref `refs/tags/v0.23.0`.

| Archive | SHA-256 |
|---|---|
| `spec-spine-v0.23.0-aarch64-apple-darwin.tar.gz` | `f6788b3c0f0839aab6953005e5089e050bdc5613333c400687c2f1ecc8a47b38` |
| `spec-spine-v0.23.0-x86_64-apple-darwin.tar.gz` | `564008e1d64cca365309c30075ec532d274c2418c26308075f11ae56123391ff` |
| `spec-spine-v0.23.0-aarch64-unknown-linux-gnu.tar.gz` | `cf8a0bf5e988bdf90747da3a5fe6d71f4e95f2cfc01aee8b63d63c8290fb91ed` |
| `spec-spine-v0.23.0-x86_64-unknown-linux-gnu.tar.gz` | `7cc6e4da6dbb91775a27b44d7ff59ae5e8bf1aeeafc0d7021b69fd12d7a35c6d` |
| `spec-spine-v0.23.0-x86_64-pc-windows-msvc.zip` | `2faf12b45ff03343c1f17e4b81a3452743c047435e398b3669259ffbb7a9ae88` |

### npm

`spec-spine@0.23.0` (`latest`), integrity
`sha512-rXbz22i/074gagOlK10+5wX32eFFOQJSBjpPey3/cXK9HyDGLMsSPohwDZP6pUx7EKWYJ1KS5Fkl9b9B5Lp6iw==`,
with its five platform packages at 0.23.0:

| Package | Integrity |
|---|---|
| `@spec-spine/cli-darwin-arm64` | `sha512-CioodVI1Dhx3GMsahKK506CkSMXigkUwqb/Ptlx2Ck0WP4B6qmQaxCnZiQRIsq2MBWbbBmRkD7KDSQqrU+K4uQ==` |
| `@spec-spine/cli-darwin-x64` | `sha512-pMAs9pdF8fCjJZvxm1p5+Pvecq4bIRhejdiVp0a+CPR2N01wrfOLvude5FK51mvuVc0BGGUsoVjv3ERQbR21Aw==` |
| `@spec-spine/cli-linux-arm64` | `sha512-qkzyJVdivhZArw5+IhaPEdgJReWTPc4zZWowF4L9kLti9p9qKKakKGLl1rwfMCDxVf96tu1sadjPnRrN//f08w==` |
| `@spec-spine/cli-linux-x64` | `sha512-jXbaD05NjW0pRLMXy2r0kutzCvG0TlMHsoifD4ZdGE/IQAobyWuQCRUxG0NeUFuJKe098h/mnEgct25QHT/28g==` |
| `@spec-spine/cli-win32-x64` | `sha512-lTkfLM4TQvqa68zYy/SLSE6+GCZUTrv62OiMUQzjBXIbnjesqeWbJlibCoNmvs6djvzMHt4hkMV4cWquzGglbg==` |

### PyPI

`spec-spine` 0.23.0, five platform wheels and an sdist, each with a PEP 740
publish attestation. The SHA-256 digests PyPI reports equal the ones attested
in the publish job:

| File | SHA-256 |
|---|---|
| `spec_spine-0.23.0-py3-none-macosx_11_0_arm64.whl` | `fa21d7dc2e1b49f9754fb730d76c8e2fe3232582d6327024e60597761019ba63` |
| `spec_spine-0.23.0-py3-none-macosx_10_12_x86_64.whl` | `46d06ce6825dd7035c138194dfed22eac5f1678f1dad8d2411495d16ed6d3327` |
| `spec_spine-0.23.0-py3-none-manylinux_2_17_aarch64.whl` | `71465018543e536bbf4793ca417ebc336d289bf8b65aee0e6ada69899ed7ddc1` |
| `spec_spine-0.23.0-py3-none-manylinux_2_17_x86_64.whl` | `6f8ba82c2ece7b0793b89ae95b9ed3bba8a3225160d3e85c76c31a114eae04db` |
| `spec_spine-0.23.0-py3-none-win_amd64.whl` | `a48ab49e3bf030852258d57dc94bbc71ccf5dbfabae42d8dd7580f584e831485` |
| `spec_spine-0.23.0.tar.gz` | `db08aadc57781a0b2393e9c736c1beb8bb518ee263c01f4e75e2ec3c879019ee` |

## 5. External consumer qualification

Run after publication in a scratch directory with a **fresh** `CARGO_HOME`,
npm cache and uv cache, no path, git or patch dependency and no workspace
inheritance, so a missing or unpublished package cannot be hidden.

| Check | Result |
|---|---|
| `cargo install spec-spine-cli --version =0.23.0 --locked` | exit 0; answers `spec-spine 0.23.0`; reader identity reports registry 1.6.0, index 1.1.0, read 0.7.0, verdict 0.6.0 |
| library: the expansion consumer (`docs/examples/expansion-consumer/src`) against `spec-spine-core = "=0.23.0"` from crates.io, driven by the installed CLI, replaying the fixture set shipped inside the registry crate | every contract held: 102, 106, 109, 107, 110, 103 (11 of 11 fixture cases), 108, 113 and the composed flow |
| lockfile | `spec-spine-core` and `spec-spine-types` 0.23.0 resolved from `registry+https://github.com/rust-lang/crates.io-index` |
| library, Statecraft's shape: `default-features = false`, `scaffold_init_json` only | 7 governance files; pure and deterministic; no `AGENTS.md`, `CLAUDE.md`, `Makefile`, `.claude/` or `.github/`; a camelCase config key refused; no tree-sitter in the lockfile |
| CLI negative: a corpus requiring a newer spec-spine | exit 3 |
| `npx -y spec-spine@0.23.0 --version` | `spec-spine 0.23.0` |
| `uvx spec-spine@0.23.0` | **not yet resolvable** at the time of writing (see §6); the wheel itself, fetched from PyPI's file host, installs with `uvx --from` and answers `spec-spine 0.23.0` |

The same script was first run against the published 0.21.0 as a control: it
failed exactly where 0.21.0 differs (its scaffold emits `AGENTS.md`, it ships
no fixture set, it lacks the expansion API), so the checks can fail.

## 6. Open at the time of writing

- **PyPI's JSON simple API is serving a stale cached page.** The upload
  succeeded (the JSON project API and the HTML simple index list all six
  files), but `https://pypi.org/simple/spec-spine/` requested as
  `application/vnd.pypi.simple.v1+json`, which uv and pip use, still returned
  `_last-serial` 41270277 without 0.23.0 more than 30 minutes after the
  upload. This is PyPI's CDN, not the artifact; it is rechecked before this
  line is closed.

  **Closed 2026-09-23 (recheck, 09:40 UTC).** With a fresh uv cache and no
  configuration, `uvx spec-spine@0.23.0 --version` (uv 0.10.12) answers
  `spec-spine 0.23.0`, and `pip install spec-spine==0.23.0` (pip 26.1.2, fresh
  venv, `--no-cache-dir`) installs and answers the same. uv's verbose log shows
  a fresh GET of `https://pypi.org/simple/spec-spine/` followed by the 0.23.0
  wheel's `.metadata` and the wheel. The simple index now answers per
  representation: the JSON form requested with uv's `Accept` header and
  compression reports `X-PyPI-Last-Serial` 41362461 and lists 0.23.0, while an
  uncompressed JSON request and the `text/html` form still served the cached
  41270277 page without it. So the earlier failure was a cached variant of the
  JSON index, the variant the resolvers use is now current, and the artifact,
  its metadata and the client needed no change. The failed observation above
  is kept as it was made.
- **Statecraft adoption** is Statecraft's work, against
  `docs/consumer-integration-expansion.md` §11.
