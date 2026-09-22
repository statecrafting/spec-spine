# Release candidate `v0.22.0`

**Status: prepared, not published.** No tag exists, nothing is pushed, and no
package has been uploaded to any registry. Everything below describes local
artifacts and local runs. The publication plan in §6 is a proposal awaiting a
human decision, not a record of anything performed.

## 1. Why this candidate exists, and why the version must be new

`0.21.0` is published and names two different producers.

Measured 2026-09-21 by building a consumer against each and calling
`spec_spine_core::scaffold_init_json` through its public facade:

| | published `spec-spine-core` 0.21.0 | source at `de854d14`, also versioned 0.21.0 |
|---|---|---|
| governance files emitted | 11 | 7 |
| `AGENTS.md` | emitted, 4,505 bytes | not emitted |
| `.claude/rules/*.md` | three files emitted | not emitted |
| `.gitignore` | whole-file write, no marker | `append: true` with an append marker |

Statecraft is implementing against the published artifact and therefore
receives the first column. The second column is correct: spec 092 removed the
harness from this engine deliberately. What is not acceptable is that one
version string identifies both, so the corrected producer gets a version of its
own. `0.21.0` is not reused.

MINOR rather than PATCH: the release carries two behavioral corrections to the
coupling gate and an additive verdict-schema change. MINOR rather than MAJOR:
no public signature is removed or changed, and no schema MAJOR moves.

## 2. What is in it

| Commit | What |
|---|---|
| `c39ee6dc` | note 09, the disposition record; specs 100 to 103 filed as drafts |
| `fbb1f245` | spec 101's hashed-input crossing declared |
| `df6fb4f7` | the owner decisions D-1 to D-6 recorded; the deleted site audited and its unique engine content retained as `docs/cli-reference.md` and `docs/configuration.md` |
| `a3d5213d` | **spec 100**: a deleted path is judged where it lived |
| `de854d14` | **spec 101**: readiness is scheduling, not approval |
| `71665ef9` | the version bump to `0.22.0` |
| this branch's tip | **spec 104**: a producer is tested as published, plus `docs/adopter-migration.md` finished and this record |

Specs 100, 101 and 104 are `implementation: complete` and `status: draft`.
This repository builds a named draft and ratifies separately, so the ratify
flips are owed and are **not** part of this candidate (§7).

Adopter-facing behavior changes are in `docs/adopter-migration.md` §9. The one
that can turn a green CI job red is §9.2: a gate job whose checkout cannot
reach the merge base now exits 3 rather than passing on a partial read.

## 3. Packaged-candidate evidence

This section records what a locally packaged artifact did. It is **not**
publication evidence: nothing here was uploaded, and no registry holds any of
these digests.

**Source commit:** `71665ef9e318f21c040cce0c3d29429346cb0ee4`, clean working
tree at package time.

**Commands:**

```sh
cargo package -p spec-spine-types -p spec-spine-core --locked
shasum -a 256 target/package/spec-spine-{core,types}-0.22.0.crate
./scripts/verify-packaged-producer.sh
```

**Package identity and digest:**

| Package | File | SHA-256 |
|---|---|---|
| `spec-spine-core` | `spec-spine-core-0.22.0.crate` | `6c7fdbf343c9977910e1a2fa4a9b3cf351aaca97f344de8e1af445a61af9c1e4` |
| `spec-spine-types` | `spec-spine-types-0.22.0.crate` | `3251695e013e25394a6067807ca13a1e794633ae15fbe8096879cf29437c8eea` |

A digest is a function of the tree it was cut from. Re-cut it after any further
commit on this branch; the numbers above describe `71665ef9` and nothing else.

**Producer acceptance, run against the packaged crate from outside the
workspace** (spec 104): all assertions passed. The same consumer built against
the published `=0.21.0` fails seven of them and names the four extra files,
which is the negative control that makes the pass meaningful.

## 4. Tests actually run

| Check | Result |
|---|---|
| `cargo test --workspace --locked` | 40 test targets, all green |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | clean |
| `cargo fmt --all --check` | clean |
| `make gate` (the `AGENTS.md` chain) | `check`, `lint --fail-on-warn` and `index coverage --fail-on-untraced` green; `couple` refuses the two version-bumped manifests, which needs a human waiver (§5.1) |
| `spec-spine verify 100` | passed, 5 commands |
| `spec-spine verify 101` | passed, 11 commands |
| `spec-spine verify 104` | passed, 5 commands |
| `cargo package -p spec-spine-types -p spec-spine-core --locked` | succeeded, verify step included |
| `./scripts/verify-packaged-producer.sh` | all assertions passed |

**Not run, and owed before a tag:**

- `cargo package --workspace --locked`, which adds `spec-spine-cli`. Only the
  two producer crates were packaged here, because the producer contract is what
  this candidate had to prove.
- `./scripts/verify-sweep.sh` from a clean checkout against the merged
  revision (spec 089). It runs against a merged revision and nothing is merged.
- CI. Nothing is pushed, so no workflow has run. A local green is not a CI
  green and is not reported as one.
- The determinism workflow's four-release-triple comparison.

## 5. Release checklist status

Against `docs/releasing.md`'s pre-flight:

| Item | Status |
|---|---|
| working tree clean and green | done |
| self-governance green | done, locally |
| merge driver binary current | n/a on this clone |
| versions bumped with `scripts/bump_version.py` | done, `--check 0.22.0` green |
| `Cargo.lock` regenerated after the bump | done |
| `cargo package --workspace --locked` | **owed** (see §4) |
| packaged producer green (spec 104) | done |
| verification sweep (spec 089) | **owed**, needs a merged revision |

## 5.1 The coupling waiver this candidate requires, which no agent may grant

`make gate` on this branch is green except for the coupling verb, which refuses
two paths:

```
C-001 'npm/package.json' changed without an authoring edit to any owning spec
      (006-distribution, 092-the-engine-ships-governance-not-an-environment)
C-001 'py/pyproject.toml' changed without an authoring edit to any owning spec
      (007-python-distribution)
```

This is correct and expected. `scripts/bump_version.py` rewrites the version in
the two distribution shims, and their owning specs describe what the shims *do*
rather than what version they carry, so there is no honest authoring edit to
make. Editing 006, 007 or 092 to mention a version number would be writing
something into an approved spec to make a gate pass.

The sanctioned resolution is a `Spec-Drift-Waiver:` line in the pull request
body, naming the bump. **A waiver is a human instrument.** It needs explicit
human approval and an agent never writes one on its own authority, so this
candidate is prepared with the refusal standing and the waiver owed. The line
has to be in the pull request body **at creation**: the gate reads the body it
is given, and adding it after the checks have run does not re-run them.

Suggested wording, for a human to accept, amend or reject:

```
Spec-Drift-Waiver: mechanical version bump to 0.22.0 across the three
release-versioned manifests (scripts/bump_version.py). No behavior in the npm
or PyPI shims changes; their owning specs describe the shims, not the version
they carry.
```

## 6. Proposed publication plan

Reviewable, sequenced, and not performed. Every step is a human decision.

1. **Review and merge**, one pull request per spec, in this order: the
   disposition and decisions commit, spec 100, spec 101, then this release
   branch. Each is independently reviewable and the later ones are stacked on
   the earlier, so merging out of order means rebasing rather than conflict
   resolution.
2. **Ratify**, as separate changes: specs 100, 101 and 104 flip `draft` to
   `approved`. This repository does not ratify inside a build.
3. **Re-run the pre-flight on the merged revision**, including the two owed
   items in §5 and a fresh `scripts/verify-packaged-producer.sh` run whose
   digests replace §3's.
4. **Tag** `v0.22.0` as a signed annotated tag (`git tag -s v0.22.0 -m ...`;
   the `-m` is required or the signing step is skipped silently).
5. **Publish in dependency order**: `spec-spine-types`, `spec-spine-core`,
   `spec-spine-cli` to crates.io, then the tag-driven workflows for the
   prebuilt binaries, npm and PyPI, per `docs/releasing.md`.
6. **Release notes** point at `docs/adopter-migration.md` and lead with §9.2,
   the clone-depth change, because it is the only item that can turn an
   adopter's green job red.
7. **Tell Statecraft** the producer version it should move to, and that
   `0.21.0`'s output is the pre-092 shape. §8 is the isolated artifact for
   that.

## 7. What is deliberately not in this candidate

- **Any ratification.** Three specs are complete and draft; the flips are owed
  and are a separate, human change.
- **Spec 102.** Deferred until a consumer names the need (note 09 §5).
- **Spec 103**, the verifier fixture set, which is being built separately and
  must not gate this candidate.
- **The governed-scope spec** (note 09 D-2), same reason.
- **Any harness delivery or Statecraft enrollment.** Unrelated and blocked on
  the counterparty.

## 8. The artifact Statecraft can consume for isolated integration

`target/package/spec-spine-core-0.22.0.crate` and
`target/package/spec-spine-types-0.22.0.crate`, with the digests in §3.

To integrate against it before anything is published, without a registry:

```sh
mkdir -p vendor && cd vendor
tar xzf .../spec-spine-core-0.22.0.crate
tar xzf .../spec-spine-types-0.22.0.crate
```

then, in the consuming manifest:

```toml
[dependencies]
spec-spine-core = { path = "vendor/spec-spine-core-0.22.0" }

[patch.crates-io]
spec-spine-types = { path = "vendor/spec-spine-types-0.22.0" }
```

The `[patch.crates-io]` line is not optional: the packaged core depends on
`spec-spine-types` by registry version, so without it the build resolves the
published types crate against the candidate core.

Two facts the integration needs and cannot get from the type signatures:

- **The producer's exact output is the seven-file governance set**, with
  `.gitignore` as an `append: true` entry carrying an `appendMarker` the
  consumer reconciles. No `AGENTS.md`, no `.claude/`, no hooks, no CI, no
  `Makefile`. `scripts/verify-packaged-producer.sh` is the executable statement
  of that contract and can be run against this artifact.
- **The facade's `Config` argument is snake_case**, matching `spec-spine.toml`,
  while every DTO it emits is camelCase. A camelCase config key is refused with
  a config error at exit 3 rather than ignored.
