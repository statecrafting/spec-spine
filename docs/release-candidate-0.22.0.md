# Release candidate `v0.22.0`

**Status: prepared, not published, and not green.** No tag exists, nothing is
pushed, and no package has been uploaded to any registry. Everything below
describes local artifacts and local runs. The publication plan in §6 is a
proposal awaiting a human decision, not a record of anything performed.

**This candidate carries two outstanding `C-001` coupling refusals** (§5.1),
re-measured at `5b8c201a` on 2026-09-21 and still standing. Self-governance on
this branch is therefore **not wholly green**: four of the five gate steps pass
and the coupling step refuses. Nothing in this record may be read as saying the
candidate is clear, and no summary of it should.

### 0.1 Reading the revisions this record names

Every sha below is a **pre-integration branch revision**, measured before
anything was pushed. Integration squash-merges each change, so none of them is
an ancestor of the default branch; they are the revisions the measurements were
actually taken at, which is what a record of a measurement has to name.

| Pre-integration revision | Landed as |
|---|---|
| `c39ee6dc`, `fbb1f245`, `df6fb4f7` | `0fa49bb6` (#288) |
| `a3d5213d` | `684e568f` (#289) |
| `de854d14` | `ff79c68a` (#290) |
| `71665ef9` | `e16ba434` (#291) |
| `52cae451`, `da1cd99b`, `5b8c201a`, `461be3fb` | this pull request |
| `02c19e9e` (spec 117) | its own pull request, after this one |

Three of those pull requests carry review fixes made during integration that
are not in the revisions above; the merged commits are the authority for what
the code does. The merged identity of the release, and the package digests
re-cut at it, are recorded in **§9** once integration completes. Until §9
exists, §3's digests describe `5b8c201a` and nothing that is merged.

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

**Source commit:** `5b8c201a337e0388b2a0ceb27ffa87a9ba73f641`, the tip of this
branch, clean working tree at package time. Re-cut 2026-09-21 after the commit
that carries this record; the numbers below describe `5b8c201a` and nothing
else.

**Commands:**

```sh
cargo package --workspace --locked          # see 4 for how this ends
shasum -a 256 target/package/spec-spine-{cli,core,types}-0.22.0.crate
./scripts/verify-packaged-producer.sh
```

**Package identity and digests.** Two digests per archive, because they answer
different questions:

| Package | `.crate` SHA-256 (archive bytes) | source-content SHA-256 |
|---|---|---|
| `spec-spine-types` | `41c9e4575c51252a98a37541c682eec7a1fc00340c0ce2370a1b4560eee7f992` | `e52fe66075255a0e59dc93b3407692fb9b2fda737cf6a86a4fe95b95d920ecc4` |
| `spec-spine-core` | `5fc5cbc80732df8a14ab0655e7b9dc7731c0b1671a6d2ec0902aec7671f0d66b` | `ce10bbf814fb6773365a494ec8edf5d73f08fa8ffede42451625fbb86716b455` |
| `spec-spine-cli` | `db74a9b1d5a4d42d86246b11a7474af5db12caa78e98346dd2890d5159758392` | `f1ce056d339f012c2e8a2b59cc5ec9d3ad225fe74a323abb9e312fd56c4850b6` |

The **`.crate` digest is the SHA-256 of the archive**, and the archive holds
both the crate's sources and generated metadata: the rewritten `Cargo.toml`,
`Cargo.toml.orig`, `.cargo_vcs_info.json` (which carries the git sha) and, for
the binary crate, a `Cargo.lock`. It therefore moves when the sources move
**and** when only the commit moves. An earlier revision of this record said it
was "a function of the commit, not of the crate's sources", which is the
correction over-applied: it is a function of both, and of the metadata besides.

The **source-content digest** is defined here, for this record only, and is not
a release artifact: SHA-256 over sorted `<path within the crate>\0<bytes>` for
every archive member except `.cargo_vcs_info.json`. It is what "did the crate's
contents change" needs, and it is not produced by `cargo` or promised by any
spec.

Measured on this stack, cutting the same three crates at `5b8c201a` (this
branch's tip) and at `02c19e9e` (spec 117's tip, which edits only the root
`Makefile` and adds one test):

| Package | `.crate` digest | source-content digest |
|---|---|---|
| `spec-spine-types` | **moved** | **identical** -- nothing packaged changed |
| `spec-spine-core` | moved | moved: 53 archive members became 54, the added test is packaged |
| `spec-spine-cli` | moved | moved, and only because its packaged `Cargo.lock` pins the two sibling `.crate` checksums |

The `types` row is the demonstration: identical crate contents, different
archive digest. The `cli` row is the one with a release consequence, below.

**The packaged CLI's lockfile pins its siblings' archive digests.** The
repository's own `Cargo.lock` carries no checksum for `spec-spine-types` or
`spec-spine-core` (they are path dependencies). The `Cargo.lock` that
`cargo package -p spec-spine-cli` writes into the archive does: it records the
two sibling `.crate` SHA-256 values. Measured by unpacking both cuts and
diffing. The consequence for the release is that the three archives must be cut
from **one** revision and that same revision published, or a `--locked` build
from the published CLI crate resolves sibling checksums that no published
archive has.

Two consequences of the first paragraph stand unchanged. The numbers above
describe `5b8c201a` and must be re-cut at the exact revision that is published.
And a `.crate` digest is not usable on its own as "did the library change".

**Producer acceptance, run against the packaged crate from outside the
workspace** (spec 104): all assertions passed. The same consumer built against
the published `=0.21.0` fails seven of them and names the four extra files,
which is the negative control that makes the pass meaningful.

## 4. Tests actually run

Every row is a local run on this branch at `5b8c201a`, with
`./target/release/spec-spine` (0.22.0) as the governing binary. Nothing here is
a CI result.

| Check | Result |
|---|---|
| `cargo test --workspace --locked` | **passed**: 40 test targets green |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | **passed**: clean |
| `cargo fmt --all --check` | **passed**: clean |
| `make gate` step 1, `check --fail-on-unresolved --fail-on-warn` | **passed**: both trees fresh |
| `make gate` step 2, `lint --fail-on-warn` | **passed**: 0 errors, 0 warnings |
| `make gate` step 3, `index coverage --fail-on-untraced` | **passed**: 97/97 claimed, 0 unclaimed |
| `make gate` step 4, `couple --base origin/main` | **FAILED**: 2 `C-001`, exit 1 (§5.1) |
| `spec-spine verify 100` | **passed**, 5 commands |
| `spec-spine verify 101` | **passed**, 11 commands |
| `spec-spine verify 104` | **passed**, 5 commands |
| `cargo package --workspace --locked` | **FAILED at the `spec-spine-cli` verify step**, for a registry reason, not a packaging one (below) |
| `cargo package -p spec-spine-{types,core,cli} --locked --no-verify` | **passed**: all three archives written |
| isolated local build of the packaged CLI | **passed** (below) |
| `./scripts/verify-packaged-producer.sh` | **passed**: 37 assertions, 0 failures |
| `./scripts/verify-sweep.sh` (spec 089) | **not run**: it tests a merged revision and nothing is merged |
| determinism workflow, four release triples | **unavailable locally**: CI-only |
| CI, any workflow | **unavailable**: nothing is pushed |
| publication, any channel | **not performed** |

### 4.1 `cargo package --workspace --locked`, and why its failure is not a defect

It fails, at the last of six steps:

```
   Verifying spec-spine-cli v0.22.0
   Unpacking spec-spine-core v0.22.0 (registry `target/package/tmp-registry`)
error: failed to verify package tarball
Caused by: failed to download `spec-spine-core v0.22.0`
Caused by: no hash listed for spec-spine-core v0.22.0
note: this is an unexpected cargo internal error
```

All three archives are **packaged** successfully before this; `spec-spine-types`
and `spec-spine-core` are then **verified** successfully. Only the CLI's verify
step fails, and it fails inside cargo's temporary local registry while resolving
a sibling version that is not published.

The control separates the two explanations. The same command on the released
`v0.21.0` tree (`694dd294`, a clean worktree) **succeeds**, and its log shows it
compiling `spec-spine-types v0.21.0` and `spec-spine-core v0.21.0` with no local
path: cargo resolved the siblings from crates.io, because 0.21.0 is published.
Nothing about the CLI crate differs between the two; the published sibling does.
So this is a **registry-resolution limitation for an unpublished sibling
version**, and it will disappear once `spec-spine-core 0.22.0` is on the index,
which is the order `docs/releasing.md` §1 already publishes in.

That is an explanation, not an acceptance, so the check was replaced rather than
skipped.

### 4.2 The explicit isolated local packaging test for the CLI

Run because §4.1's failure left the CLI archive unproven, and run from the
archives rather than from the workspace:

```sh
W=/tmp/ss-cli-verify-0.22.0; mkdir -p $W; cd $W
for c in types core cli; do tar xzf .../spec-spine-$c-0.22.0.crate; done
mkdir -p spec-spine-cli-0.22.0/.cargo
cat > spec-spine-cli-0.22.0/.cargo/config.toml <<'EOF'
[patch.crates-io]
spec-spine-types = { path = "../spec-spine-types-0.22.0" }
spec-spine-core  = { path = "../spec-spine-core-0.22.0" }
EOF
cd spec-spine-cli-0.22.0 && cargo build --release
```

Result: **passed**. The build log names
`spec-spine-types v0.22.0 (/tmp/ss-cli-verify-0.22.0/spec-spine-types-0.22.0)`
and the same for core, so both halves under test are the packaged ones and
nothing reached the registry or the workspace. The resulting binary reports
`spec-spine 0.22.0` and governs this repository correctly:
`check --fail-on-unresolved --fail-on-warn` exit 0 and `lint --fail-on-warn`
exit 0.

`--locked` is deliberately absent: `[patch.crates-io]` changes resolution, so
the packaged lockfile no longer applies. That is a property of the test
harness, not of the crate.

### 4.3 The binary that judged, and the defect that made it worth saying

`SPEC_SPINE ?= spec-spine` in the root `Makefile` resolves on `PATH`; on the
machine this was prepared on that is `~/.cargo/bin/spec-spine 0.20.0`, three
releases behind the checkout. Every verdict above was produced with
`./target/release/spec-spine` and the binary is named in each run's
announcement.

**This is fixed, and the fix is not in this candidate.** Spec 117, on branch
`117-the-gate-resolves-the-binary-it-documents`, corrects the assignment to the
order the header documents, refuses an unusable explicit override instead of
silently substituting one, and announces the resolved binary and its version
before the chain runs. It is reviewable on its own endpoints and reaches this
candidate only through the explicit integration branch §6.0 describes.

## 5. Release checklist status

Against `docs/releasing.md`'s pre-flight. "unavailable" means the check cannot
be performed from here at all; "owed" means it can be performed and has not.

| Item | Status |
|---|---|
| working tree clean | **passed** |
| self-governance green | **FAILED**: four gate steps pass, `couple` refuses (§5.1) |
| merge driver binary current | **not applicable** on this clone |
| versions bumped with `scripts/bump_version.py` | **passed**, `--check 0.22.0` green |
| `Cargo.lock` regenerated after the bump | **passed** |
| `cargo package --workspace --locked` | **FAILED for a registry reason** (§4.1); the CLI archive is instead proven by §4.2 |
| packaged producer green (spec 104) | **passed**, 37 assertions (§3) |
| verification sweep (spec 089) | **unavailable**: needs a merged revision |

The sweep's entry point was checked rather than copied forward:
`./scripts/verify-sweep.sh --rev <rev>`, `--rev` defaulting to `HEAD`, is what
the script accepts today, and `docs/releasing.md` calls it with
`--rev origin/main`. It is not run here because nothing is merged, and a sweep
against an unmerged branch answers a different question.

## 5.1 The coupling refusal: outstanding, and why no honest authority path closes it

**Still refusing.** Re-measured at `5b8c201a` on 2026-09-21 with
`./target/release/spec-spine` 0.22.0, `couple --base origin/main --head HEAD`,
exit 1:

```
C-001 'npm/package.json' changed without an authoring edit to any owning spec
      (006-distribution, 092-the-engine-ships-governance-not-an-environment)
C-001 'py/pyproject.toml' changed without an authoring edit to any owning spec
      (007-python-distribution)
```

**Exact affected paths and diff.** Two files, `npm/package.json` and
`py/pyproject.toml`, and in both the whole diff is the version string:

```diff
--- a/npm/package.json
+++ b/npm/package.json
-  "version": "0.21.0",
+  "version": "0.22.0",
-    "@spec-spine/cli-darwin-arm64": "0.21.0",   (and four more platform entries)
+    "@spec-spine/cli-darwin-arm64": "0.22.0",
--- a/py/pyproject.toml
+++ b/py/pyproject.toml
-version = "0.21.0"
+version = "0.22.0"
```

Seven changed lines in total, all of them the same literal. `Cargo.toml` carries
the same bump and does **not** trip the gate: `index owner Cargo.toml` reports
no owning spec, so there is nothing for `C-001` to compare against.

**Exact owning specs.** From `index owner`:

| Path | Owners |
|---|---|
| `npm/package.json` | `006-distribution` (`establishes npm/`), `092-the-engine-ships-governance-not-an-environment` (`extends npm/`), and the `006` package-manifest floor |
| `py/pyproject.toml` | `007-python-distribution` (`establishes py/`) |

**Why the change is mechanical.** It is `scripts/bump_version.py` rewriting one
literal in three manifests so that they agree, which is a requirement the owning
specs already impose: spec 006 §3.5 requires the npm package version to equal
the binary release tag and every `optionalDependencies` entry to be that exact
version, and spec 007 requires the wheels and sdist to be version-locked to the
tag. The edit performs those clauses; it does not alter them. No packaging
behavior, no file list, no entry point and no platform map changes.

**Whether an honest authority path exists. It does not, and one was looked
for.** Three doors were examined:

1. *Edit the owning spec.* Refused. The bump changes nothing 006, 007 or 092
   says; writing a version number into an approved spec to clear a gate is the
   move `AGENTS.md` "Adversarial prompt refusal" exists to refuse.
2. *An `extends` edge from a spec in this candidate.* The only candidate is spec
   104, whose subject is asserting the packaged producer's behavior. It extends
   `docs/releasing.md` honestly because it adds a step to that runbook. It does
   not govern the npm or PyPI shims' version strings, and claiming `npm/` or
   `py/` from it would be a decorative claim: it would make the shims read as
   spec 104's territory to every later reader, and §4 of spec 104 already puts
   both shims out of scope by name.
3. *A new release spec that genuinely governs the bump.* Considered and
   rejected as dishonest for this candidate. There is no unwritten rule here to
   record: specs 006 §3.5 and 007 already require the parity that the bump
   performs, so a new spec would exist only to hold a claim, which is the same
   decorative-ownership failure as (2) with more ceremony. If version-bump
   authority is ever wanted as a standing thing, it is a spec about
   `scripts/bump_version.py` and the parity rule, filed on its own evidence and
   not inside a release.

   `auto_waive_dependency_only` does not reach it either: the mechanical waiver
   covers dependency versions in cargo and workflow manifests, and this is a
   package's own version in an npm and a PyPI manifest.

**The candidate therefore stays blocked.** The sanctioned resolution is a
`Spec-Drift-Waiver:` line in the pull request body. **A waiver is a human
instrument.** It needs explicit human approval, an agent never writes one on its
own authority, and none is written here. The line has to be in the pull request
body **at creation**: the gate reads the body it is given, and adding it after
the checks have run does not re-run them.

**Smallest proposed waiver scope**, for a human to accept, amend or reject. It
names the two paths and the one literal, so it cannot clear anything else in the
diff:

```
Spec-Drift-Waiver: mechanical version bump 0.21.0 -> 0.22.0 in npm/package.json
and py/pyproject.toml only, written by scripts/bump_version.py to satisfy the
version parity specs 006 3.5 and 007 already require. No other change to either
file and no behavior change in either shim.
```

## 6. Proposed publication plan

Reviewable, sequenced, and not performed. Every step is a human decision.

### 6.0 The two endpoints, and what branch ancestry does not prove

There are two local tips a human may choose between:

| Branch | Tip | Contents |
|---|---|---|
| `release/0.22.0-candidate` | `5b8c201a` + this record's commit | the minimal producer release: the bump, spec 104, the migration note |
| `release/0.22.0-integration` | a `--no-ff` merge of spec 117 onto the above | the same, plus the gate binary-selection fix |

Spec 117 branches from this candidate's tip, so the integration merge is a
fast-forwardable ancestry and its tree is byte-identical to spec 117's own tip.
**That does not make either branch merge-ready.** Ancestry says a merge can be
computed without conflict; it says nothing about whether a protected pull
request would pass. A future integration is judged at its own endpoints: the
base it is actually opened against, the head sha the checks actually run on, and
the pull-request body the coupling gate is actually given. Both branches carry
§5.1's refusal against `origin/main` and neither clears it.

Nothing in this stack has been pushed, so no branch has a CI verdict of any
kind.

### 6.1 The sequence

1. **Review and merge**, one pull request per spec, in this order: the
   disposition and decisions commit, spec 100, spec 101, then this release
   branch. Each is independently reviewable and the later ones are stacked on
   the earlier, so merging out of order means rebasing rather than conflict
   resolution. Spec 117 is a separate decision and is not required by any of
   them.
2. **The waiver**, in the release pull request body at creation (§5.1).
3. **Ratify**, as separate changes: specs 100, 101 and 104 flip `draft` to
   `approved`. This repository does not ratify inside a build.
4. **Re-run the pre-flight on the merged revision**: `./scripts/verify-sweep.sh
   --rev origin/main`, a fresh `cargo package --workspace --locked` (which
   should succeed once the siblings are published, §4.1), and a fresh
   `scripts/verify-packaged-producer.sh` whose digests replace §3's.
5. **Tag** `v0.22.0` as a signed annotated tag (`git tag -s v0.22.0 -m ...`;
   the `-m` is required or the signing step is skipped silently).
6. **Publish in dependency order**: `spec-spine-types`, `spec-spine-core`,
   `spec-spine-cli` to crates.io, then the tag-driven workflows for the
   prebuilt binaries, npm and PyPI, per `docs/releasing.md`. Cut all three
   archives from the one published revision (§3, the CLI lockfile note).
7. **Release notes** point at `docs/adopter-migration.md` and lead with §9.2,
   the clone-depth change, because it is the only item that can turn an
   adopter's green job red.
8. **Tell Statecraft** the producer version it should move to, and that
   `0.21.0`'s output is the pre-092 shape. §8 is the isolated artifact for
   that.

## 7. What is deliberately not in this candidate

- **Any ratification.** Three specs are complete and draft; the flips are owed
  and are a separate, human change.
- **Spec 102.** Deferred until a consumer names the need (note 09 §5).
- **Spec 103**, the verifier fixture set, whose draft is present on this branch
  (filed with 100 to 102) and whose **build** is not: it lives on
  `103-a-verifier-fixture-is-a-published-artifact`, a sibling of this branch,
  so that it cannot gate the candidate. `index coverage` on this branch reports
  two of its units as `planned`, which is that separation showing through.
- **Spec 105**, `governed_scope` enabled here (note 09 D-2), same reason.
- **Spec 116 and the deferred contracts 106 to 115.** A separate branch with a
  separate adoption question; none of them is required by this release.
- **Spec 117**, the gate binary-selection fix: reviewable on its own branch and
  reaching a release only through §6.0's integration branch.
- **Any harness delivery or Statecraft enrollment.** Unrelated and blocked on
  the counterparty.

## 8. The artifact Statecraft can consume for isolated integration

`target/package/spec-spine-core-0.22.0.crate` and
`target/package/spec-spine-types-0.22.0.crate`, with the digests in §3. Both
were cut at `5b8c201a` and both must be re-cut at whatever revision is actually
published (§3). `target/package/spec-spine-cli-0.22.0.crate` exists too and is
proven by §4.2; Statecraft consumes the library, not the CLI, so it is not part
of this artifact.

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
- **The facade's `config_json` argument is the `spec-spine.toml` model, and it
  is snake_case.** Every facade entry point takes the configuration as its own
  string argument, and that string deserializes into `Config`: table names and
  keys exactly as they are spelled in `spec-spine.toml`
  (`{"layout":{"derived_dir":"..."}}`), with no `rename_all`. Every **other**
  request object the facade accepts, and every DTO it emits, is camelCase. All
  of them carry `deny_unknown_fields`, so a casing mistake is a loud refusal and
  never a silent default: `{"layout":{"derivedDir":".x"}}` is a config error at
  exit 3. This is asserted, not changed. `scripts/verify-packaged-producer.sh`
  carries the contract test ("a camelCase config key is refused, not silently
  ignored") and it runs against the packaged crate on every release pre-flight;
  renaming the argument or adding camelCase aliases would be a breaking change
  to a type spec 000 and spec 001 own, and would need its own compatibility
  spec.
