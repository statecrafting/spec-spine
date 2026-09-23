# Release candidate `v0.22.0`

**Status: candidate re-frozen at `f9fa6a8f56b82c8d97cf2803bc31838a0b455a21`,
not tagged and not published.** §14 (2026-09-23) keeps that candidate apart
from current `main` and from the next expansion release, and supersedes 13.7's
ratification text; nothing in it moves the tag revision. **§13 is the
candidate's record**: it
supersedes §11's proposed tag revision (`da47632b`), its checks, its digests
and its procedure, because `da47632b` lacks the `Acceptance` corrections of
specs 120 and 121 (§13.1). Sections 0.1 to 8 are the pre-integration record,
§9 the integration record, §10 the owner rulings and §11 the first freeze, all
preserved as written. §12 corrects §10.5's account of the first sweep run's
failure.

**Superseded status line (first freeze):** corrected, ratified, candidate
frozen at `da47632b`, not tagged and not published. §11 was then the state
that was true.

**Original status line, as written before integration:**

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

## 9. The integration record (2026-09-22)

Written after the six pull requests below merged. It supersedes §3's digests,
§4's test table and §5's checklist; §5.1's account of the refusal and its
reasoning is unchanged and was executed as written.

### 9.1 The merged source identity

**Release revision: `835dd2e4c460d24a3bc95d565345d5bea8afd964`** (`835dd2e4`),
the tip of the default branch after the sixth merge.

| # | Pull request | Squash commit | What |
|---|---|---|---|
| 1 | #288 | `0fa49bb6` | disposition, note 09, specs 100 to 103 filed as drafts, owner decisions D-1 to D-6 |
| 2 | #289 | `684e568f` | **spec 100**, a deleted path is judged where it lived |
| 3 | #290 | `ff79c68a` | **spec 101**, readiness is scheduling, not approval |
| 4 | #291 | `e16ba434` | the version bump to 0.22.0 -- the one pull request carrying a waiver |
| 5 | #292 | `2d3673db` | **spec 104**, a producer is tested as published, and this record |
| 6 | #293 | `835dd2e4` | **spec 117**, the gate resolves the binary it documents |

Each merged with the required `ci-gate` status green on its own head. None was
judged against an intermediate branch.

### 9.2 The waiver, and the evidence that it hid nothing

The waiver §5.1 proposed was approved by the repository owner for the two
manifest version changes and nothing else, and was placed in #291's body at
creation. #291 carries the version bump **alone**, which is what bounds the
waiver: a blanket waiver at the verb level cannot excuse a refusal that is not
in the pull request it sits on.

Measured, per pull request, with `couple` at each one's own base and head:

| Pull request | `couple` without a waiver | Waiver |
|---|---|---|
| #288 | OK, 5 paths | none |
| #289 | OK, 12 paths | none |
| #290 | OK, 2 paths | none |
| #291 | **2 violations**: `npm/package.json`, `py/pyproject.toml` | the approved line |
| #292 | OK, 3 paths | none |
| #293 | OK, 3 paths | none |

#291's `self-governance` job logged `2 violation(s) waived` naming exactly
those two paths. `Cargo.toml` and `Cargo.lock` carry the same bump and refuse
nothing: `index owner` reports no owning spec for either, so there is nothing
for `C-001` to compare against and nothing the waiver had to cover.

### 9.3 Checks at the merged revision, by what they actually prove

| Check | Where it ran | Result |
|---|---|---|
| `ci-gate` | CI, run `35710705254` on `835dd2e4` | **passed** |
| `build · test · clippy` | CI, same run | **passed** |
| `self-governance (compile · index · lint · couple)` | CI, same run | **passed** |
| determinism, four release triples, byte-identical | CI, same run | **passed** |
| `Acceptance`, merge-scope sweep | CI, run `35710704945` | **passed**: 3 passed, 0 failed |
| `make gate` | local, clean tree at `835dd2e4` | **passed**, all four steps; announces `./target/release/spec-spine ... spec-spine 0.22.0` |
| `cargo package --workspace --locked` | local | **passed** (see §9.5) |
| `./scripts/verify-packaged-producer.sh` | local, clean tree | **passed**, 37 assertions |
| `./scripts/verify-sweep.sh --rev origin/main`, whole corpus | local | **FAILED**: 60 passed, 3 failed, 43 exempt, 0 not-declared (see §9.4) |
| publication, any channel | -- | **not performed** |

Four categories, kept apart deliberately: **CI** ran the gate, the suite and
the four-triple determinism comparison on the merged head. **Local packaging**
proved the archives build and the producer behaves. **Registry-backed
verification** has not happened and cannot until something is published.
**The full sweep** is local and is the one red result.

### 9.4 The blocker: spec 095's acceptance now fails

`./scripts/verify-sweep.sh --rev origin/main` at `835dd2e4`:

```
verify-sweep.sh: 835dd2e4  passed=60 failed=3 not-declared=0 exempt=43 not-run=0
```

Three failures, of two different kinds.

**Kind 1, and the real one: `095-the-corpus-describes-what-exists`,** failing at
command 9:

```python
o == list(range(len(o)))   # AssertionError: [0, 1, 2, 3, 4]
```

Spec 095's acceptance asserts the corpus ordinals are **contiguous**. Merging
spec 117 while 105 to 116 stay reserved on an unmerged branch makes them not
contiguous, so the block fails. Measured cause, not inference: at `2d3673db`,
the revision immediately before #293, `registry list --ids-only` returns 105
ids and the same predicate is `True`.

**The spec's normative text and its acceptance disagree.** §3.3 requires the
one-time renumber to leave the survivors contiguous, which it did. D-2 says, in
terms: *"Gaps are not a defect on their own; spec 095's and 117's reserved gaps
were deliberate."* The acceptance encodes a standing invariant the spec does not
state and its own decision record contradicts.

Nothing in the gate chain catches this, by design: `verify` is the one verb
that executes and is deliberately outside the chain, and the `Acceptance`
workflow on a merge sweeps only the specs the merge changed, which is why
#293 was green on every required check.

This needs an owner decision and is **not** something a session may resolve:
every available route is corpus authority. Three routes, with what each costs:

1. **Correct spec 095's acceptance to match §3.3 and D-2** -- a new spec
   carrying an `amends_verification` edge, filed and built together. Editing
   095 directly is the move `AGENTS.md` "Adversarial prompt refusal" forbids.
   This is the route the evidence points at: the assertion is stronger than the
   requirement it was written to check.
2. **Renumber spec 117 to the next free ordinal.** Restores contiguity and
   costs a rename plus every citation, and it consumes an ordinal the deferred
   drafts reserved. It also treats an acceptance defect as a corpus defect.
3. **Merge specs 105 to 116.** Excluded from this release path by the owner.

Until one is taken, `docs/releasing.md`'s "verification sweep green" pre-flight
item is **not satisfied**, and no tag should be cut against a checklist item
that is red.

**Kind 2, structural and pre-existing: `102-a-ready-spec-carries-its-status`
and `103-a-verifier-fixture-is-a-published-artifact`.** Both are
`implementation: pending`: filed as drafts by #288 and deliberately not built.
Their acceptance blocks are written to fail against the tree they are filed on,
which is this repository's fail-first discipline working correctly. The sweep
has no `implementation` predicate and no flag to select on one, so a filed
unbuilt draft is always counted `failed`. That makes "sweep green" unreachable
for as long as any draft is filed ahead of its build, which is this
repository's normal state. Worth recording; not this release's to fix, and not
a defect in either spec.

### 9.5 `cargo package --workspace --locked`: the recorded limitation did not reproduce

§4.1 records this command failing at the `spec-spine-cli` verify step with
`no hash listed for spec-spine-core v0.22.0`, and attributes it to an
unpublished sibling version. **At `835dd2e4` it succeeds**, exit 0, with the
CLI's verify step compiling `spec-spine-types` and `spec-spine-core` 0.22.0 out
of cargo's own `target/package/tmp-registry`.

So the limitation as §4.1 states it does not hold at this revision, and §4.2's
isolated build is no longer the only evidence for the CLI archive. The checklist
item §5 marked owed is now green. What §4.1 established and still stands is the
control: the same command on the released `v0.21.0` tree also succeeds.
Registry-backed verification, the kind that can only happen after a publish,
remains untested and is listed as such in §9.3.

### 9.6 Package identities, re-cut at the merged revision

Cut at `835dd2e4`, clean working tree, and **independently reproduced** by two
separate runs of `cargo package` at the same commit with identical results:

| Package | File | SHA-256 |
|---|---|---|
| `spec-spine-types` | `spec-spine-types-0.22.0.crate` | `38af52dcf4961144a60be59261c6b1f8537e59177f9f8bd50b2ef958618a78fb` |
| `spec-spine-core` | `spec-spine-core-0.22.0.crate` | `c3ade5a94263ee7cb72c315ac0ff8e06a080150b56bb936c66122e3fb8a044b8` |
| `spec-spine-cli` | `spec-spine-cli-0.22.0.crate` | `ca52766902b955943467925eca9c1d2f15539763039ccc84c9602497a086870d` |

These supersede §3's, which described `5b8c201a`, a revision that is not an
ancestor of the default branch.

**They describe `835dd2e4` and nothing else.** A `.crate` archive carries
`.cargo_vcs_info.json` with the git sha, so any later commit -- including the
one that adds this section -- moves all three digests while the crate sources
stay byte-identical. Whatever revision is finally tagged, the three archives
must be cut from **that one revision** and these numbers replaced, for the
reason §3 gives: the packaged CLI's own `Cargo.lock` pins the two sibling
`.crate` checksums, so three archives cut from different commits do not agree.

### 9.7 The ordered publication procedure, unchanged and not performed

§6.1 still describes it. Restated with what is now settled and what is not:

0. **Resolve §9.4's blocker.** Owner decision. Nothing below should happen
   first.
1. Review and merge -- **done**, §9.1.
2. The waiver -- **done**, §9.2.
3. **Ratify.** Specs 100, 101, 104 and 117 are `implementation: complete` and
   `status: draft`. Four `status` flips, owed, human, and separate from every
   merge above. Specs 102 and 103 stay `draft`/`pending` and are not ratified.
4. **Re-run the pre-flight on whatever revision is finally tagged**, including
   a fresh `verify-packaged-producer.sh` whose digests replace §9.6's, and a
   green whole-corpus sweep.
5. **Tag** `v0.22.0`, signed and annotated (`git tag -s v0.22.0 -m ...`; the
   `-m` is required or the signing step is silently skipped), on the merged
   revision after the default branch's own CI is green on it -- a squash lands
   on a base no pull request tested.
6. **Publish in dependency order**: `spec-spine-types`, then
   `spec-spine-core`, then `spec-spine-cli` to crates.io, then the tag-driven
   workflows for the prebuilt binaries, npm and PyPI. All three archives cut
   from the one tagged revision (§9.6).
7. **Release notes** lead with `docs/adopter-migration.md` §9.2, the clone-depth
   change, the only item that can turn an adopter's green job red. Prepend them
   to the generated body with `gh release edit --notes-file` after the run
   completes.
8. **Tell Statecraft** the producer version to move to, and that `0.21.0`'s
   output is the pre-092 shape.

Steps 0 and 3 are decisions. Steps 4 to 8 are the maintainer's, and none of
them has been performed.

## 10. Owner rulings, and the first sweep run preserved (2026-09-22)

Recorded before any change they govern is built. §9 stands as written; this
section adds the decisions §9.4 asked for and the evidence §9.3 summarized
without keeping.

### 10.1 Spec 095's acceptance: the owner's ruling

The repository owner ruled on §9.4, in these terms:

- Spec 095's renumbering requirement describes the **historical** corpus
  transformation. It does not impose perpetual contiguity on every future
  corpus. Legitimately reserved later ordinals are permitted.
- The acceptance comparing every current ordinal with `range(len(ids))` is to
  be corrected. Spec 117 keeps its identity. Specs 105 to 116 are not merged and
  no published identity is renumbered to satisfy the assertion.
- The correction uses the supported verification-amendment mechanism where
  appropriate (route 1 of §9.4); a direct corrective edit is authorized only
  where needed to remove a contradictory executable contract.
- The replacement must keep meaningful verification of the historical renumber
  and its map, unique and valid identifiers and ordering, reference and
  ownership integrity, and the removal of the surfaces 095 governed, and must
  not hard-code today's corpus size.

It is to be carried by **spec 118**, a new spec declaring `amends` and
`amends_verification` on 095, filed and built in its own pull request after
this record merges. 095's requirements and commands are not edited.

### 10.2 The sweep: the owner's ruling

The owner authorized a narrowly scoped correction of the sweep contract (spec
089, as amended by 095 and 099) that distinguishes **implemented-release
acceptance** from **pending-feature acceptance**, subject to: every selected
implemented obligation is judged; pending and in-progress work stays visible
with its actual lifecycle; nothing unimplemented becomes `passed` or `exempt`;
missing plans, unknown lifecycle, parse errors, interrupted commands and
unavailable evidence are never success; an implemented draft's failure is not
hidden by its `status`; the closed exemption ledger is not expanded; the
whole-corpus run keeps reporting what it observed; and execution stays off
every untrusted event. It is to be carried by **spec 119**, in its own pull
request after 118.

This is the ruling on §9.4's other two failures, `102` and `103` ("Kind 2").
Neither is built, exempted or reclassified. Both stay failed in the
whole-corpus count and are reported as pending, with their real outcomes, in the
release verdict spec 119 adds. Spec 102 still waits on a named consumer (note 09
§5). Spec 103's build is next-wave work, kept off this release path.

### 10.3 What these rulings do not authorize

No ratification (the four flips in #295 and any later ones stay separate human
decisions), no tag, no publication to any registry, no deployment, and no
Statecraft activation. #291's waiver covered the two manifest version lines
and nothing else; it is not reused by anything after it.

### 10.4 The first whole-corpus run at `835dd2e4`, preserved

§9.3 reports the second run (60 / 3 / 43). The **first** run, started
09:30:07Z and finished 09:41:11Z on 2026-09-22 against the same revision with
the same command, reported:

```
verify-sweep.sh: 835dd2e4  passed=37 failed=26 not-declared=0 exempt=43 not-run=0
```

Its report directory no longer exists: the second run used the same default
`--out`, and spec 089 §3.6 clears a marked run directory before use. What
survives is each run's console log, committed with this record so the digests
can be checked from the repository:

| Run | File | SHA-256 | Bytes |
|---|---|---|---|
| first (37 / 26 / 43) | `docs/evidence/sweep-835dd2e4-run1.console.log` | `03b48024d5f34456fc2620fa306e84c060116f5b9e71bdf38b1757b520e1f3c9` | 8475 |
| second (60 / 3 / 43) | `docs/evidence/sweep-835dd2e4-run2.console.log` | `bd7698e33cde2187edeacecdf9e3ec6a35cfd2c9d401a7978c1ebb2e186b707b` | 6547 |

The first run's per-spec logs are gone; the two excerpts below were read from
them before they were cleared. The 26 failures were
`079` to `100`, `102`, `103`, `104` and `117`; every spec before `079` passed
or was exempt. The first failure, in `079`'s log:

```
[verify] $ cargo clippy --workspace --all-targets --locked -- -D warnings
    Checking tree-sitter v0.27.0
error: couldn't read `.../spec-spine-sweep-835dd2e4/tree/target/debug/build/tree-sitter-413b4d7f936d18f0/out/stdlib-symbols.txt`: No such file or directory (os error 2)
[verify] exit 101
verify: 079-a-blocking-claim-is-not-a-stale-shard: FAILED at command 74 (exit 101)
```

and, in `094`'s, the same error against the **release** profile's output
directory (`target/release/build/tree-sitter-c45c6c9252d2c28d/out/`) at
`cargo build --release --locked`. `079`'s command 73, `cargo test`, had
passed immediately before on already-compiled artifacts.

### 10.5 The cause: the operating system's temporary-file cleaner

> **[corrected 2026-09-22, §12]** This heading and the phrase "the evidence
> points elsewhere" read as an established cause. It is not established: the
> deletion was not observed. §12 separates what was observed, reproduced and
> inferred. The text below is preserved as written.

The earlier reading, "the first `cargo clippy` poisoned the shared target
directory", is **not supported**: the release-profile output directory that
`094` found empty is one `clippy` never writes. The evidence points elsewhere:

- The default run directory, and so the worktree and its `target/`, is under
  `$TMPDIR` (`/var/folders/.../T/`). macOS's `com.apple.bsd.dirhelper` runs
  daily at 03:35 local time with `CLEAN_FILES_OLDER_THAN_DAYS=3` and removes
  older files there (`/System/Library/LaunchDaemons/com.apple.bsd.dirhelper.plist`).
- The unified log shows it ran at **03:35:04 -0600 on 2026-09-22** ("cleaning
  directories"), which is 09:35:04Z, inside the first run's window. The
  console log shows `055` as the last spec finished at about that minute and
  the first failure at `079`. The 23 specs between them passed, which is
  consistent with Cargo reusing artifacts compiled before the deletion; that
  was not verified spec by spec.
- `tree-sitter`'s build script copies `src/wasm-stdlib/imports.txt` into
  `OUT_DIR/stdlib-symbols.txt` with `std::fs::copy`, which on macOS clones the
  file and keeps its timestamps. Crate archives carry normalized timestamps:
  every `stdlib-symbols.txt` in this machine's build directories has an mtime
  and birth time of **2006-07-23**. A file born seconds earlier therefore looks
  twenty years old to the cleaner.
- Cargo does not re-check a build script's output files once the script has
  run, so nothing rebuilt the missing file and every later compilation of
  `tree-sitter` failed. **Reproduced**, with cargo 1.92.0 (the pinned
  toolchain) in a fresh target directory outside the temporary tree:
  `cargo build -p tree-sitter@0.27.0 --locked`, then delete
  `debug/build/tree-sitter-*/out/stdlib-symbols.txt`, then build again. The
  other files in that `out/` survive, the build script is not rerun, and the
  recompile fails with the first run's message verbatim:
  `error: couldn't read .../out/stdlib-symbols.txt: No such file or directory
  (os error 2)`, then `could not compile tree-sitter (lib)`. A rerun fails the
  same way, so the state does not repair itself.

The second run started at 03:42, after the cleaner, and was not affected; it is
a run after an identified environment change, not a retry of an unchanged one.
So the Cargo half is reproduced. The deletion half is inferred from the
cleaner's schedule, its logged run inside the window, and the files'
timestamps: the per-spec logs that would show the order of events are gone.
Not done: running the cleaner deliberately to reproduce the deletion, which
needs privileges this session does not use. The bounded remedy, a default run
directory outside the purged temporary tree and unique per run so a rerun can
never clear an earlier run's evidence, is part of spec 119.

## 11. The candidate, frozen (2026-09-22)

### 11.1 What landed after §9, one pull request each

| # | Pull request | Squash commit (full) | What |
|---|---|---|---|
| 1 | #296 | `b04a138b623e0ec4a08b47aceae30f0f4d531b04` | §10: the owner rulings, the preserved first sweep run, its inferred cause (§12) |
| 2 | #297 | `bf910840941d79ff1cbe9be4242c017289fb0443` | **spec 118**: 095's acceptance corrected through `amends_verification` |
| 3 | #298 | `8d091f915be37782bcc479a0600cb269dafb527b` | **spec 119**: the sweep's release verdict and its run directory |
| 4 | #295 | `da47632b8ac413fccd2518dbd7326f44328712d6` | ratification of 100, 101, 104 and 117, on the owner's approval |

Each merged with `ci-gate` green on its own head, with its branch brought up to
date with `main` by an ordinary merge (no force-push), `couple` clean at its
own base and head, and no waiver. #291's waiver was not reused.

### 11.2 The proposed tag revision

**`da47632b8ac413fccd2518dbd7326f44328712d6`**, the tip of `main` after #295.
The commit that adds this section comes after it and changes documentation only.
It is deliberately **not** the tag revision: a record cannot carry the digests
of the commit that contains it.

The candidate was first cut at `8d091f91`, the tip after #298. #295 merged
after that, so every check whose result depends on the source identity was run
again at `da47632b`. Both sets are kept below, and the `8d091f91` digests are
void.

### 11.3 Checks, by where they ran

| Check | Where | Revision | Result |
|---|---|---|---|
| `ci-gate`, `build · test · clippy`, `self-governance`, determinism across the four release triples (byte-identical) | CI, run `35770599303` (push to `main`) | `da47632b` | **passed** |
| the same | CI, run `35768189973` | `8d091f91` | passed |
| `make gate` (check, lint, coverage, couple) | local, clean detached worktree | `da47632b` | **passed** |
| `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings` | local | `da47632b` | **passed** |
| `cargo test --workspace --locked`; `cargo test -p spec-spine-core --no-default-features --locked` | local | `da47632b` | **passed** |
| `scripts/bump_version.py --check` (distribution parity: Cargo, npm, PyPI agree on 0.22.0) | local | `da47632b` | **passed** |
| `./scripts/verify-packaged-producer.sh` | local, packaged archives | `da47632b` | **passed**, 37 assertions |
| `cargo package --workspace --locked`, verification enabled | local, twice, independent target directories | `da47632b` | **passed** both times, identical digests (§11.5) |
| whole-corpus sweep, `--release` | local, isolated worktree, binary built from the revision | `da47632b` | **release verdict clean**; corpus verdict not clean (§11.4) |
| `Acceptance`, push leg | CI, run `35770598972` | `da47632b` | **failed**, not an acceptance failure (§11.6) |
| registry-backed consumer verification | nowhere | none | **not performed**: nothing is published |

Four kinds of evidence, kept apart: **CI** at the merged revision; **local
checks** in a clean worktree at the same revision; **local packaging**, which
proves the archives build, verify against each other and behave as a
producer; and **registry-backed verification**, which cannot exist before a
publish.

### 11.4 The sweep, both verdicts

`./scripts/verify-sweep.sh --rev da47632b8ac413fccd2518dbd7326f44328712d6 --release`,
binary `spec-spine 0.22.0` built from `da47632b` inside the sweep's worktree,
report `sweep.json` schema 1.1.0 (SHA-256
`180a633df3f3d8062fd31040da318da0ce30bdac971f3b45d8eded1cae51d295`), console log
committed as `docs/evidence/sweep-da47632b-release.console.log`:

```
verify-sweep.sh: da47632b  passed=63 failed=2 not-declared=0 exempt=43 not-run=0
verify-sweep.sh: release verdict: clean  (not passing=0 pending=2)
```

| | passed | failed | not-declared | exempt | not-run | pending |
|---|---|---|---|---|---|---|
| corpus verdict (not clean) | 63 | 2 | 0 | 43 | 0 | n/a |
| release verdict (**clean**) | 63 | 0 | 0 | 43 | 0 | 2 |

All 108 specs are accounted for. The two corpus failures are the two pending
drafts, shown with the lifecycle the report read:

- `102-a-ready-spec-carries-its-status`: draft, `implementation: pending`;
  block failed at command 2, fail-first as filed. Deferred until a consumer
  names the need (note 09 §5, 102's D-1); none has.
- `103-a-verifier-fixture-is-a-published-artifact`: draft,
  `implementation: pending`; block failed at command 1, fail-first as filed.
  Its build is next-wave work on its own branch, not in this candidate.

`095` passes, now running 118's block. No block left the worktree dirty.
`--release` exited 0. The same run at `8d091f91` gave identical counts.

### 11.5 Package identities at `da47632b`

| Package | File | SHA-256 |
|---|---|---|
| `spec-spine-types` | `spec-spine-types-0.22.0.crate` | `db56db128f091abd838db6051ffef6e056f71fa51a767affffb0ef0d17a44b74` |
| `spec-spine-core` | `spec-spine-core-0.22.0.crate` | `9f4e5e1426c21fcde00a3a411d04d1b7161f0c908d23d721b983e2608779065b` |
| `spec-spine-cli` | `spec-spine-cli-0.22.0.crate` | `836ffdd243a18235a881fe2101ef72b727b0b6b381427bc11a32efde2ff9ee06` |

- **One revision.** All three archives' `.cargo_vcs_info.json` name
  `sha1 da47632b8ac413fccd2518dbd7326f44328712d6`, with no dirty flag.
- **Sibling checksums agree.** The CLI archive's `Cargo.lock` pins
  `spec-spine-types` at `db56db12...` and `spec-spine-core` at `9f4e5e14...`,
  and the core archive's pins `spec-spine-types` at `db56db12...`, exactly the
  digests above.
- **Reproduced.** A second `cargo package --workspace --locked` in an
  independent target directory produced the same three digests.
  `verify-packaged-producer.sh` re-cuts types and core as well, and those
  matched too.
- **The package-resolution limitation does not recur.** §4.1's `no hash listed
  for spec-spine-core` did not appear. The CLI's verify step compiled both
  siblings from cargo's local package registry, with verification enabled and
  without `--no-verify`, at `da47632b`, as §9.5 found at `835dd2e4`.
- **Void:** §9.6's digests (`835dd2e4`), and the ones cut at `8d091f91`
  (types `82d734db...`, core `2d9f9e0b...`, cli `d0ff5499...`).
- **Not established:** that the archives `release.yml` cuts on its runner
  from the tag are byte-identical to these. Compare the checksums crates.io
  serves after publication against this table and record either result.

### 11.6 Outstanding: the `Acceptance` push leg is red at `da47632b`

The push leg swept the four specs #295 touched and reported all four `failed`
at exit 127: `./target/release/spec-spine: not found`. The acceptance is not
failing. The workflow hands the sweep a binary built outside the sweep's
worktree (`SPEC_SPINE_BIN`), and these four blocks call
`./target/release/spec-spine` without building it first. In a whole-corpus run
an earlier block's `cargo build --release` creates that file, which is why the
local and nightly runs pass them. A scoped run of exactly these four specs has
nothing to create it. The same failure is in the push leg at `ff79c68a` (#290)
and went undiagnosed there.

`Acceptance` is outside `ci-gate` by design (spec 099) and gates nothing, and
the workflow is not in any published artifact. The fix, which is to let the
sweep build the binary from the revision as spec 089 §3.8's default does, is
filed as its own spec after this record and does not change the tag revision.

### 11.7 The ordered publication procedure

Not performed. Each numbered step is a human action.

0. **Decide the tag revision.** Proposed: `da47632b` (§11.2). Specs 118 and
   119 are `draft` / `complete`. If the owner wants them ratified before the
   tag, that merge moves the proposed revision and everything in §11.3 and
   §11.5 that depends on source identity must be cut again.
1. **Tag** `v0.22.0` on the chosen revision, signed and annotated:
   `git tag -s -m "spec-spine 0.22.0" v0.22.0 <sha>`. `-s` is what signs.
   `-m` supplies the message; without it, a non-interactive run fails with
   `fatal: no tag message?` (this repository's v0.11.0 release). §9.7's note
   that a missing `-m` skips the signature silently is wrong and is corrected
   here. Confirm with `git tag -v v0.22.0`, then push the tag only.
2. **The tag drives `release.yml`:** `build` (prebuilt binaries per triple),
   then in parallel `publish` (the GitHub Release), `publish-crates`,
   `publish-npm` and `publish-pypi`. `publish-crates` publishes
   `spec-spine-types`, `spec-spine-core` and `spec-spine-cli` in that order
   with `cargo publish --locked`. Each publish verifies against the index and
   waits for the previous crate to be visible.
3. **Registry visibility, per stage.** crates.io's API (send a `User-Agent`)
   serves each of the three at 0.22.0. Record the served checksums against
   §11.5. The npm `spec-spine@0.22.0` and its platform packages resolve; npm
   view can lag a publish and is not a failure. PyPI `spec-spine==0.22.0`
   resolves, and its `info.version` can lag too.
4. **Consumer checks, registry-backed**, in clean directories:
   `cargo install spec-spine-cli --version 0.22.0 --locked` then
   `spec-spine --version`; `npx spec-spine@0.22.0 --version`;
   `uvx --refresh spec-spine==0.22.0 --version` (without `--refresh`, a stale
   cache gives a false negative); and the spec 104 producer checks against the
   **published** `spec-spine-core`.
5. **Release notes** lead with `docs/adopter-migration.md` §9.2 (the
   clone-depth change) and are prepended to the generated body with
   `gh release edit --notes-file` once the run completes.
6. **Tell Statecraft** the producer version (0.22.0), and that 0.21.0's output
   is the pre-092 shape.

## 12. The first run's failure: what is observed, reproduced and inferred (corrected 2026-09-22)

An earlier report of this candidate said the failure's "cause [was]
established" while also saying the deletion was inferred and the per-spec logs
were lost. Both cannot be true, and the second is. This section replaces §10.5
as the current account; §10.5 stays as written, marked. Nothing was re-run to
produce it: it restates the evidence already recorded, with its limits.

**Observed.**

- The first run at `835dd2e4` started 09:30:07Z and finished 09:41:11Z on
  2026-09-22 and reported `passed=37 failed=26` (its console log, §10.4, digest
  `03b48024...`).
- Its run directory, and so the sweep's worktree and `target/`, was under
  `$TMPDIR`.
- macOS's `com.apple.bsd.dirhelper` is scheduled daily at 03:35 local time with
  `CLEAN_FILES_OLDER_THAN_DAYS=3`, and the unified log records it "cleaning
  directories" at 03:35:04 -0600 (09:35:04Z), inside that window.
- The console log shows `055` as the last spec finishing at about that minute
  and `079` as the first failure. Two error excerpts, read from the per-spec
  logs before they were cleared, show `stdlib-symbols.txt` missing from
  `tree-sitter`'s `OUT_DIR` in the debug and in the release profile.
- Every `stdlib-symbols.txt` in this machine's build directories carries an
  mtime and birth time of 2006-07-23, the crate archive's normalized
  timestamp, because the build script copies the file with its timestamps.
- The second run, started 03:42 local, after the cleaner, reported
  `passed=60 failed=3`.

**Reproduced**, with the pinned cargo 1.92.0 in a fresh target directory
outside the temporary tree: build `tree-sitter@0.27.0`, delete
`debug/build/tree-sitter-*/out/stdlib-symbols.txt`, build again. The build
script is not rerun, the recompile fails with the first run's message
verbatim, and a rerun fails the same way. So **if** that file disappears
mid-sweep, every later `tree-sitter` compilation in that worktree fails with
exactly the observed error.

**Inferred, and not observed.** That the cleaner deleted `stdlib-symbols.txt`
from the sweep's target directory during the first run. The inference rests on
the timing, the file's old timestamps meeting the cleaner's age rule, and the
exact error text. Nothing observed the deletion: the cleaner logs that it ran,
not what it removed; the first run's per-spec logs, which would show the order
of events, were cleared by the second run; and the cleaner was not run
deliberately, which needs privileges this work does not use. The earlier
reading, a `cargo clippy` corrupting the shared target, is not supported,
because the release-profile directory it would have to corrupt is one clippy
does not write. That rules out one explanation; it does not prove this one.

**The practical remedy does not depend on the inference.** Spec 119 §3.5 makes
the default run directory new per run under the user cache directory, outside
the purged temporary tree. That removes the exposure whether or not the purge
was the cause, and it keeps each run's report, so a later failure of this kind
leaves its own evidence instead of being overwritten.

## 13. The candidate, re-frozen at `f9fa6a8f` (2026-09-22)

### 13.1 Why `da47632b` is not the candidate

§11.6 recorded that the `Acceptance` push leg was red at `da47632b` for a
reason outside the corpus (the binary's location). That correction, spec 120,
merged after §11 was written, so `da47632b` does not contain it. The nightly
leg had a second, independent problem: it exited on the corpus verdict, so it
was red whenever a draft was filed ahead of its build, which is this
repository's normal state. Spec 121 corrects that. Both are operational
corrections to the release's own acceptance signal and are in the candidate.
Tagging `da47632b` would ship a revision whose `Acceptance` workflow is known
to be wrong in two ways.

### 13.2 What landed after `da47632b`, one pull request each

| # | Pull request | Squash commit (full) | What |
|---|---|---|---|
| 1 | #299 | `1cff0431efa0db1bf4f2eb5c5710c83be237e167` | §11: the first freeze's record |
| 2 | #300 | `737c1c89d7771bd03ef4db224fc71148cb57b8d1` | **spec 120**: the push leg builds the binary its blocks name |
| 3 | #303 | `481f3f1e8268012f3a6b2cad99e77232642ee5c2` | spec 119's and §10.5's account of the first run's failure, corrected (§12); wording only |
| 4 | #302 | `f9fa6a8f56b82c8d97cf2803bc31838a0b455a21` | **spec 121**: `Acceptance` exits on the release verdict and reports both |

Each merged with `ci-gate` green on its own head, `couple` clean, and no
waiver.

### 13.3 The proposed tag revision, and its difference from `da47632b`

**`f9fa6a8f56b82c8d97cf2803bc31838a0b455a21`**, the tip of `main` after #302.
Everything merged after it (this record, the owner-decision record #304, and
any expansion work) is not in the tag.

`git diff --name-only da47632b f9fa6a8f`, outside the derived shard trees, is
exactly ten files:

- `.github/workflows/acceptance.yml` (specs 120, 121)
- `scripts/acceptance-report.py` (new, spec 121)
- `scripts/test-verify-sweep.py` (spec 121's regressions)
- `scripts/verify-sweep.sh` (a comment, §12)
- `specs/119-.../spec.md` (wording, D-6), `specs/120-.../spec.md` and
  `specs/121-.../spec.md` (new, both `draft` / `complete`)
- `docs/release-candidate-0.22.0.md`, `docs/releasing.md` (one sentence) and
  `docs/evidence/sweep-da47632b-release.console.log`

**Nothing under `crates/`, `Cargo.toml`, `Cargo.lock`, `npm/` or `py/`
changed** (the same diff restricted to those paths is empty). The engine, the
CLI and both shims are byte-identical in source to `da47632b`; the packages
differ from §11.5's only in VCS metadata and in what follows from it (13.6).

### 13.4 Checks at `f9fa6a8f`, by where they ran

| Check | Where | Result |
|---|---|---|
| `ci-gate`, `build · test · clippy`, `self-governance`, determinism across the four release triples (byte-identical) | CI, run `35786274180` (push to `main`) | **passed** |
| `Acceptance`, push leg: scope 5 specs, `--release`, report step | CI, run `35786273449` | **passed**: `passed=5`, release verdict clean; the new "Report both verdicts" step succeeded |
| `Acceptance`, whole corpus, `--release`, dispatched on `main` at `f9fa6a8f` | CI, run `35787507923` | **passed**: release verdict clean; corpus not clean (13.10) |
| `make gate` (check, lint, coverage, couple), binary built from the revision | local, clean detached worktree | **passed** |
| `cargo fmt --all --check`; `cargo clippy --workspace --all-targets --locked -- -D warnings` | local | **passed** |
| `cargo test --workspace --locked`; `cargo test -p spec-spine-core --no-default-features --locked` | local | **passed** |
| `python3 scripts/test-verify-sweep.py` (15 tests) | local | **passed** |
| `scripts/bump_version.py --check 0.22.0` (distribution parity: Cargo, npm, PyPI) | local | **passed** |
| `./scripts/verify-packaged-producer.sh` | local, packaged archives | **passed**, 37 assertions |
| `cargo package --workspace --locked`, verification enabled | local, twice, independent target directories | **passed** both times, identical digests (13.6) |
| whole-corpus sweep, `--release` | local, isolated worktree, binary built from the revision | release verdict **clean**, exit 0 (13.5) |
| registry-backed consumer verification | nowhere | **not performed**: nothing is published |

### 13.5 The sweep, both verdicts

`./scripts/verify-sweep.sh --rev f9fa6a8f56b82c8d97cf2803bc31838a0b455a21 --release`,
binary `spec-spine 0.22.0 built from f9fa6a8f` inside the sweep's worktree,
report `sweep.json` schema 1.1.0 (SHA-256
`6b521df9178ad0092827e2895a0207019e0b17512d095cec4ca4e1dc855ad0f5`), console
log committed as `docs/evidence/sweep-f9fa6a8f-release.console.log` (SHA-256
`661608f75667500b66d8edc137a9aae59550d608840cfeb2d72091d536d9088c`, 6790
bytes):

```
verify-sweep.sh: f9fa6a8f  passed=65 failed=2 not-declared=0 exempt=43 not-run=0
verify-sweep.sh: release verdict: clean  (not passing=0 pending=2)
```

| | passed | failed | not-declared | exempt | not-run | pending |
|---|---|---|---|---|---|---|
| corpus verdict (**not clean**) | 65 | 2 | 0 | 43 | 0 | n/a |
| release verdict (**clean**) | 65 | 0 | 0 | 43 | 0 | 2 |

All 110 specs are accounted for. The two corpus failures are the two pending
drafts, run and reported with their real outcomes:

- `102-a-ready-spec-carries-its-status`: `draft` / `pending`; failed at
  command 2, fail-first as filed. Its build is next-wave work (PR, not in the
  tag).
- `103-a-verifier-fixture-is-a-published-artifact`: `draft` / `pending`;
  failed at command 1, fail-first as filed. Its build is #301, not in the tag.

No block left the worktree dirty. The run directory was
`~/.cache/spec-spine/sweeps/f9fa6a8f-20260922T212305Z-8591`, outside the
purged temporary tree (§12).

The whole-corpus CI run (`35787507923`) is the first nightly-shaped run under
spec 121, recorded in 13.10. It is the CI counterpart of this local run, not a
substitute for it, and neither is reported as the other.

### 13.6 Package identities at `f9fa6a8f`

| Package | File | SHA-256 |
|---|---|---|
| `spec-spine-types` | `spec-spine-types-0.22.0.crate` | `6b1a2cb685d72d7032ebf647c427af6cec5217269d9626ce13c7e10eec011eb8` |
| `spec-spine-core` | `spec-spine-core-0.22.0.crate` | `8db10dd0b5552f429e90b96446278e385235069316942b9551d0101f179d2c23` |
| `spec-spine-cli` | `spec-spine-cli-0.22.0.crate` | `ffeea42520d7d78adb0023729201ed4f62631e64aa4df7024473d1012719a370` |

- **One revision.** All three archives' `.cargo_vcs_info.json` name `sha1
  f9fa6a8f56b82c8d97cf2803bc31838a0b455a21`, with no dirty flag.
- **Sibling checksums agree.** The core archive's `Cargo.lock` pins
  `spec-spine-types` at `6b1a2cb6...`; the CLI archive's pins
  `spec-spine-core` at `8db10dd0...` and `spec-spine-types` at `6b1a2cb6...`:
  exactly the digests above.
- **Verification ran.** `cargo package --workspace --locked` verified all three
  crates (the `Verifying` step compiled each from the packaged sources),
  without `--no-verify`.
- **Void:** §11.5's digests (`da47632b`).

What has and has not been measured, kept apart:

| Identity | Measured? |
|---|---|
| Local reproducibility | **Yes.** Two `cargo package` runs in independent target directories produced the three digests above, byte-identical. |
| CI artifact identity | **No.** `release.yml` packages and builds only on a `v*` tag, so no CI run has produced these archives. |
| Registry-served identity | **No.** Nothing is published. After publication, compare the checksums crates.io serves with this table and record either result. |

### 13.7 Ratification decisions still open

Prepared, not merged, and not required for the tag unless the owner wants them
in it:

- **#305**: `118`, `119` and `120`, `draft` to `approved`, status-only.
- **Spec 121**, `draft` / `complete` at `f9fa6a8f`: not in #305 because its
  ratification was not requested; it can be added on the same terms.

If either merges **before** the tag, the proposed tag revision moves to that
merge commit, and 13.4's source-identity-dependent rows and 13.6's digests must
be cut again. If they merge after, the release carries these specs as
`draft` / `complete`.

### 13.8 What is deliberately not in this candidate

- Specs **102**, **103** (#301), **106** and **107**, and the owner-decision
  record **#304**: next-wave work and the decision that authorized it, merged
  after the freeze or still under review.
- Any ratification (13.7).

### 13.9 The ordered publication procedure

Not performed. Each numbered step is a human action. **Pushing the tag starts
publication**: `release.yml` runs on the tag push and publishes to crates.io,
npm and PyPI and creates the GitHub Release without a further approval step.

0. **Decide the tag revision.** Proposed: `f9fa6a8f` (13.3). Decide 13.7 first:
   a ratification merge before the tag moves it.
1. **Tag, signed and annotated:**
   `git tag -s -m "spec-spine 0.22.0" v0.22.0 f9fa6a8f56b82c8d97cf2803bc31838a0b455a21`,
   then `git tag -v v0.22.0`. Push the tag only:
   `git push origin refs/tags/v0.22.0`. This is the step that publishes.
2. **`release.yml`**: `build` (prebuilt binaries per triple), then in parallel
   `publish` (the GitHub Release), `publish-crates` (types, core, cli, in that
   order, `cargo publish --locked`, each waiting for the previous to be
   visible), `publish-npm` and `publish-pypi`.
3. **Registry visibility, per stage.** crates.io's API (send a `User-Agent`)
   serves each crate at 0.22.0; record the served checksums against 13.6. npm
   `spec-spine@0.22.0` and its platform packages resolve (`npm view` can lag).
   PyPI `spec-spine==0.22.0` resolves (`info.version` can lag).
4. **Consumer checks, registry-backed**, in clean directories:
   `cargo install spec-spine-cli --version 0.22.0 --locked` then
   `spec-spine --version`; `npx spec-spine@0.22.0 --version`;
   `uvx --refresh spec-spine==0.22.0 --version`; and the spec 104 producer
   checks against the **published** `spec-spine-core`.
5. **Release notes** lead with `docs/adopter-migration.md` §9.2 and are
   prepended with `gh release edit --notes-file` once the run completes.
6. **Tell Statecraft** the producer version (0.22.0).

### 13.10 The first nightly-shaped CI run under spec 121

`Acceptance`, `workflow_dispatch` with an empty scope (the whole corpus, the
nightly's selection), run `35787507923`, at `f9fa6a8f`, binary built from the
revision inside the sweep's worktree:

```
verify-sweep.sh: f9fa6a8  passed=65 failed=2 not-declared=0 exempt=43 not-run=0
verify-sweep.sh: release verdict: clean  (not passing=0 pending=2)
```

The job **passed**. The report step published two `warning` annotations, one
per pending draft, each with its lifecycle and the block's real outcome
("block failed ... pending work, not a release obligation"), and no `error`.
Under the pre-121 workflow the same corpus made the nightly red (run
`35723398403` at `3d4f3902`). The counts equal the local run's in 13.5; they
were measured independently, on a GitHub-hosted runner and on this machine.


## 14. Three revisions kept apart, after the expansion wave merged (2026-09-23)

The expansion wave merged on `main` after the freeze. None of it moves the
proposed tag. This section keeps three things separate so that no later merge
can be read as a change to 0.22.0.

### 14.1 The three revisions

| | Revision | What it is |
|---|---|---|
| **Frozen 0.22.0 candidate** | `f9fa6a8f56b82c8d97cf2803bc31838a0b455a21` | Unchanged. §13 is its whole record: checks (13.4), sweep (13.5), package digests (13.6), procedure (13.9). Not tagged, not published, not recut. |
| **Current `main`** | `2fff1e494cabc40746ccc77a8610f67ce0807175` | The candidate plus the evidence records, the expansion wave and the two acceptance repairs below. Reports `0.22.0` too, so the version string does not distinguish it from the candidate. |
| **Next expansion release** | not cut | Specs 102, 103, 105, 106, 107, 109, 110 and 122 (14.2). Needs its own version bump, candidate record, checks and ratification decisions. |

**No recut.** A recut is owed only if the chosen tag revision changes.
Nothing merged after `f9fa6a8f` changes a byte of that revision, and 14.4
records that nothing merged after it corrects a defect in it.

### 14.2 Merged after the freeze, one pull request each

| # | Pull request | Squash commit (full) | What | In 0.22.0? |
|---|---|---|---|---|
| 1 | #304 | `d29fa048943711898954a170ed9da650b3f84b4f` | owner decision D-7, opportunity-led expansion | no, record |
| 2 | #306 | `03a204b94ada8d5cfca49a7566fb3ec8e98fe308` | §13, the re-freeze record | no, record |
| 3 | #301 | `c638093216f99221f33d2a76975c242c65b8802f` | **spec 103**, verifier fixtures | no |
| 4 | #307 | `75a998f7122f32392c38c8b052c8e686511e1230` | **spec 102**, readiness status | no |
| 5 | #308 | `088d6d4b711659ce563f496d6c5c421ac2f9a84f` | **spec 106**, obligations and section digests | no |
| 6 | #310 | `45becbbeb960dc95df3c201cd027c366b15693cd` | **spec 122**, the commit refuses an unresolved merge or unformatted Rust | no |
| 7 | #309 | `6e123d2e4632ef2feb8d65e8e8843ff8bf545482` | **spec 107**, context closures | no |
| 8 | #311 | `b7c13452a3cdcc924ddc7cf720810b571325c803` | **spec 105**, governed scope | no |
| 9 | #312 | `0ca3001d6b5cc36bec14fde8b2852360572fa63c` | **spec 109**, impact and conflict | no |
| 10 | #313 | `8c6c4d73d25d7c586fee2b6a49d0b11b8db302c8` | **spec 110**, digest-pinned interface references | no |
| 11 | #314 | `73182329f548c7c54572d4bee80260c212581c43` | spec 102 carries 053's acceptance (held by 087), with `status` (14.9) | no |
| 12 | #315 | `2fff1e494cabc40746ccc77a8610f67ce0807175` | spec 105 carries 078's acceptance, unset case on a scratch corpus (14.9) | no |

Every implementation PR merged through branch protection with the required
checks green on its head. The stacked 107 was verified against its effective
diff after 106's squash; 110's branch carried 109's pre-squash history and, after
merging `main` at `0ca3001d`, differed from `main` by 110's files only.

### 14.3 Composed verification on `main` at `8c6c4d73d25d7c586fee2b6a49d0b11b8db302c8`

Measured in a clean detached worktree at the merge commit, with the binary
built from it (`spec-spine 0.22.0`). Local unless the row says CI.

| Check | Result |
|---|---|
| `make gate` (check, lint, coverage, couple) | **passed** |
| `cargo fmt --all --check`; `cargo clippy --workspace --all-targets --locked -- -D warnings` | **passed** |
| `cargo test --workspace --locked --no-fail-fast` (53 test binaries); `cargo test -p spec-spine-core --no-default-features --locked` | **passed** |
| `spec-spine verify` for 102 (10), 103 (13), 105 (16), 106 (11), 107 (7), 109 (9), 110 (8), 122 (12) | **all passed**, 86 commands |
| `./scripts/verify-packaged-producer.sh` (spec 104), packaged crates | **passed**, 37 assertions |
| `docs/examples/expansion-consumer/run.sh`, packaged crates | **passed**, every contract below |
| CI on the merge commit: `CI` run `35811669236` | **passed** |
| CI on the merge commit: `Acceptance` push leg run `35811669203` | **failed**: `053` and `078`, both `approved`, release verdict not clean (14.9) |

The packaged runs were cut from `52ae253b` (this docs change), whose
`crates/`, `Cargo.toml` and `Cargo.lock` are identical to `8c6c4d73`'s; the
archives' digests describe that commit's VCS metadata, not a release
(`spec-spine-types` `3c2c7fdf...`, `spec-spine-core` `bc17feae...`). This is
local source/package verification, not registry-backed verification.

The combined contracts, and where each is asserted:

| Contract | Asserted by |
|---|---|
| A ready entry carries its real status; scheduling and approval stay distinct (a `draft` is ready) | `closure_composed.rs` (102 with 106/107), consumer §102 |
| An obligation resolves the same through `registry obligation`, `registry show` and `query_json` | `closure_composed.rs`, `interface_composed.rs`, consumer §106 |
| A withdrawn obligation keeps its identity; re-declaring the id is refused, and a reused id cannot keep a recorded digest | `closure_composed.rs`; its impacts report `targetWithdrawn` (`interface_composed.rs`, consumer §109) |
| A section digest moves when its governed content moves, and only then | `closure_composed.rs`, `interface_composed.rs` (a status flip moves the whole-spec identity and no section) |
| A closure normalizes order, duplicates and short ids | `closure_composed.rs`, consumer §107 |
| Missing members, absent section digests and stale ledgers refuse; what a tolerant read accepts cannot enter a closure | `closure_composed.rs`, consumer §107 |
| A changed obligation or section changes the closure, and a pin on it goes stale while the declared impact set stays byte-identical | `interface_composed.rs`, consumer §110 |
| A stale exporter ledger cannot hide a change from a pin; a stale importer ledger refuses (exit 2) | `interface_composed.rs` |
| Verifier-fixture coverage measures real outcomes, from the packaged crate | `verifier_fixtures.rs` (every build), consumer §103 (all 11 cases through `verify_attestation_json`) |
| Fixture regeneration after a registry MINOR moves only `registryHash`/`attestationHash`; `version-mismatch` stays a version mismatch | 110 D-2 (diff measured), consumer §103 |

### 14.4 The frozen candidate: defects looked for, none found

Examined: `git diff f9fa6a8f 8c6c4d73` over `crates/*/src`, `npm/`, `py/`,
`Cargo.toml` and `Cargo.lock`, every `*_SCHEMA_VERSION` constant, and every post-freeze commit subject. The engine
diff is additive (new modules, new fields, new verbs): 2405 lines added, 17
deleted. The deleted lines restructure `plan`'s ready entries and a re-export
list to carry 102's new field, replace two doc comments, widen two compile
helpers to `pub(crate)`, and move the two schema constants; none of them
changes an existing behavior, and no post-freeze commit is a fix to code in
the candidate. Spec 122 corrects this repository's own
commit procedure (`.githooks/pre-commit`), which is not in any package.
**Result: no defect in `f9fa6a8f` was found by this work.** That is a
statement about what was examined, not a proof of absence.

### 14.5 Schema axes: candidate against `main`

| Axis | `f9fa6a8f` | `main` |
|---|---|---|
| registry `specVersion` | `1.3.0` | `1.6.0` (106, 109, 110) |
| read documents `schemaVersion` | `0.1.0` | `0.6.0` (102, 106, 107, 109, 110) |
| verifier fixture set | absent | `0.1.0` (103) |
| index, verdict, attestation, delta, snapshot, config | unchanged | unchanged |

All moves are MINOR. Each MINOR has one owner and one number: 1.4.0/1.5.0/1.6.0
and 0.2.0 to 0.6.0 were each taken once, in merge order, and
`schema_versions_are_pinned` (`dtos.rs`) and
`a_stamped_document_carries_the_read_axis_version` (`read.rs`) pin the head of
each axis with its history. `docs/schema-versioning.md` lists every step.

### 14.6 Ratification decisions still open (supersedes 13.7)

Status changes are the owner's. Nothing here is ratified.

| Specs | Where | State | Effect on 0.22.0 |
|---|---|---|---|
| 118, 119, 120, 121 | #305, status-only, brought up to date with `main` | awaiting owner approval | none if merged on `main`: the candidate stays `f9fa6a8f` and these ship there as `draft` / `complete`. Ratifying *inside* 0.22.0 needs a release branch from `f9fa6a8f` and a full recut (#305 option 2), not prepared. |
| 102, 103, 105, 106, 107, 109, 110, 122 | not proposed | `draft` / `complete` | none; they are next-release content |

13.7's sentence that a ratification merge before the tag moves the tag
revision is superseded: `main` now carries the wave, so no merge commit on
`main` is a 0.22.0 candidate.

### 14.7 Publication

Unchanged from 13.9 and not performed. **Pushing the tag starts
publication**: `release.yml` publishes to crates.io, npm and PyPI and creates
the GitHub Release on the tag push, with no further approval step. The tag,
if the owner chooses 0.22.0 as frozen, is
`git tag -s -m "spec-spine 0.22.0" v0.22.0 f9fa6a8f56b82c8d97cf2803bc31838a0b455a21`,
followed by `git tag -v v0.22.0` and `git push origin refs/tags/v0.22.0`.

### 14.8 The merge and commit failure path, corrected and observed

Two execution mistakes during the wave's integration committed work that should
have been refused: conflict markers after a merge that reported conflicts, and
a commit after a failed `cargo fmt --all --check`. Both passed through a
command chain in which a `;` (or a pipe whose exit status was the last
command's) let the commit run whatever the earlier step returned, and through a
pre-commit hook that checked neither.

The correction is in two places, neither of them new machinery:

- **The repository's hook (spec 122, #310).** Before resolving any binary,
  `.githooks/pre-commit` refuses unmerged index entries (`git ls-files -u`),
  refuses conflict markers on added lines only (`git diff --cached --check`,
  so a marker already in history is never re-reported, and an intentional
  fixture opts out by the `conflict-marker-size` attribute, never by a path
  pattern in the hook), and refuses a staged Rust change that fails
  `cargo fmt --all --check`. Generated shards left stale by a conflict
  resolution are still refused by the freshness read that follows. Asserted
  behaviorally through `git commit` in a throwaway repository
  (`commit_boundary.rs`), and `verify 122` passes at `8c6c4d73`.
- **The procedure.** A required check and the commit it guards are separate
  commands, or joined only by `&&` with no `;` and no pipe between them; a
  pipe that must carry the check's status runs under `pipefail`.

Observed once, unprompted, on 2026-09-23 while addressing review on #313: a
chain of the form `cargo fmt --all --check && <build> ; git add ... && git
commit ...` ran `git commit` after the format check had failed. The hook
answered `REFUSED: cargo fmt --all --check failed`, `HEAD` did not move, and
the push that followed sent nothing. The commit was redone as separate steps
(`2d45831d`). Separately, a commit in a fresh worktree with no in-tree binary
was refused because the hook fell back to an installed `spec-spine 0.20.0`
that predates spec 106 and judged the corpus invalid. That refusal was a
wrong-judge refusal, not a pass: building `target/release/spec-spine` first
is the remedy, and the hook's resolution order (spec 093) is unchanged.

### 14.9 Two approved acceptance blocks the wave broke, and their repair

The push leg of `Acceptance` runs only after a merge (spec 099 keeps it off
pull requests, because it executes what the corpus declares), so no pull
request check could see this. It was red on `main` from #307 onward, and the
wave kept merging:

| Run | Commit | `053` | `078` |
|---|---|---|---|
| `35793731561` | `75a998f7` (#307, 102) | failed | passed |
| `35795405838` | `088d6d4b` (#308, 106) | failed | passed |
| `35798066907` | `6e123d2e` (#309, 107) | failed | passed |
| `35799033728` | `b7c13452` (#311, 105) | not in scope | not in scope |
| `35808640704` | `0ca3001d` (#312, 109) | failed | failed |
| `35811669203` | `8c6c4d73` (#313, 110) | failed | failed |

The leg sweeps a scope computed from what the merge touched. At 105's own
merge it swept four specs and 078 was not among them, so 105's break surfaced
one merge later, on 109's run. A scoped leg can therefore report a break on a
later, unrelated merge; the whole-corpus run (nightly, or `verify-sweep.sh`
by hand) is the one that attributes a break to nothing but the tree.

- **053**, held by 087: three lines compare a ready entry, and the `--next`
  pick, to exactly `{ id, title }`. Spec 102 added `status`. 102's D-5 moved
  the two test pins of that shape and missed the block, because a block is
  not a test the build runs.
- **078**: its block asserts that *this repository* leaves `governed_scope`
  unset. Spec 105 set it. The assertion was about corpus state.

The whole-corpus release sweep at `8c6c4d73` (`verify-sweep.sh --rev
8c6c4d73d25d7c586fee2b6a49d0b11b8db302c8 --release`, run locally in its own
worktree with the binary built from the revision) accounts for all 116 specs:

```
verify-sweep.sh: 8c6c4d73  passed=70 failed=3 not-declared=0 exempt=43 not-run=0
verify-sweep.sh: release verdict: NOT CLEAN  (not passing=3 pending=0)
```

The three are `053`, `078` and `087` (087 is the same break as 053, seen at the
spec that holds 053's acceptance). Nothing else in the corpus is red.

Neither is a defect in the code: the behavior each block was written to pin
still holds, and each block's failure is an exact-shape or corpus-state
assertion that the later spec made false on purpose. Both specs are
`approved`, so each is repaired by amendment (spec 082 §3.2), never by an
edit: #314 (spec 102 carries 087's block, and through it 053's, with each
exact assertion extended by `status`) and #315 (spec 105 carries 078's
block, with the unset case moved to a scratch corpus and the set case asserted
here). Each keeps its exact assertions exact and adds a line that goes red if
the approved file is edited in place.

**After the repairs.** #314 merged as `73182329`; its push leg (run
`35814736994`) swept `053`, `087` and `102`: `passed=3 failed=0`, release
verdict clean. #315 merged as `2fff1e49`, whose tree is byte-identical to the
head that was verified locally: `make gate`, and `verify` for 053, 087 and 102
(36 commands each, all running 102's block) and for 078 and 105 (38 commands
each, running 105's block), all passed there. On `main` at `2fff1e49`, `CI` (run `35815207825`) passed and the
`Acceptance` push leg (run `35815207681`) swept `078` and `105`: `passed=2
failed=0`, release verdict clean.

**Procedure correction.** After each merge, read the post-merge `Acceptance`
verdict on `main` before merging the next change. A red push leg is a finding
about the merged revision, and the next merge's green PR checks do not answer
it.
