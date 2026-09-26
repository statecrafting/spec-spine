---
id: "124-the-expansion-line-carries-its-own-version"
title: "The expansion line carries its own version"
status: approved
kind: "tooling"
created: "2026-09-23"
summary: >
  The frozen 0.22.0 candidate (`f9fa6a8f`) and `main` both answered
  `spec-spine 0.22.0` while emitting different registry, read and verdict
  schemas, so the version string could not say which engine a verdict came
  from, and an older reader judged a newer corpus invalid instead of refusing
  it. `main` carries the version the next release will be cut at (0.23.0
  when filed, 0.24.0 since D-3, 0.25.0 since D-4, 0.26.0 since D-5, 0.27.0 since D-6, 0.28.0 since D-7), in all three package manifests; this repository's version floor
  moves with it, so a reader older than the corpus's grammar refuses by name
  (exit 3); the release runbook makes both a standing step; and a script
  records a binary's identity from evidence (path, digest, revision, whether it
  is that checkout's current build, and the schema axes it emits) rather than
  from `--version`. The frozen candidate is not recut.
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "006-distribution"
  - "061-shipped-is-not-the-same-as-working"
extends:
  # 3.1: the package version literal in the two shim manifests.
  - { spec: "006-distribution", unit: { kind: file, path: "npm/package.json" }, nature: additive }
  # A directory unit, as 007 establishes it: `py/pyproject.toml` alone is in
  # no content hash (L-008), and the version literal is 007's shim's.
  - { spec: "007-python-distribution", unit: { kind: directory, path: "py/" }, nature: additive }
  # 3.2: the floor, which 061 §3.8 says matches this repository's version.
  - { spec: "061-shipped-is-not-the-same-as-working", unit: { kind: file, path: "spec-spine.toml" }, nature: additive }
  # 3.3: the runbook's two standing steps.
  - { spec: "006-distribution", unit: { kind: file, path: "docs/releasing.md" }, nature: additive }
  # 3.2, 3.4: the instruction file's floor sentence and the identity script.
  - { spec: "105-governed-scope-is-enabled-here", unit: { kind: file, path: "CLAUDE.md" }, nature: additive }
  # 3.5: the verifier fixtures record the producer's version.
  - { spec: "103-a-verifier-fixture-is-a-published-artifact", unit: { kind: directory, path: "crates/spec-spine-core/fixtures/verifier/" }, nature: additive }
establishes:
  # 3.4: the identity record.
  - { kind: file, path: "scripts/reader-identity.sh" }
references:
  - { unit: { kind: file, path: "docs/release-candidate-0.22.0.md" }, role: context }
  - { unit: { kind: file, path: "scripts/bump_version.py" }, role: context }
---

# 124: The expansion line carries its own version

## 1. Purpose

### 1.1 One version string, two engines

The 0.22.0 release record freezes the candidate at `f9fa6a8f` (§13, §14). The
expansion wave (specs 102, 103, 105, 106, 107, 109, 110, 122, then 113 and
123) merged on `main` after the freeze, without a version change. Measured on
2026-09-23 with `scripts/reader-identity.sh` (§3.4):

| | Frozen candidate | `main` at `0bf9ff78` |
|---|---|---|
| answers `--version` | `spec-spine 0.22.0` | `spec-spine 0.22.0` |
| registry schema | `1.3.0` | `1.6.0` |
| read documents | `0.1.0` | `0.6.0` |
| verdict envelope | `0.5.0` | `0.6.0` |
| index schema | `1.1.0` | `1.1.0` |

Two materially different engines shared one package identity. Spec 123 §1.1
records what that cost: a build of the candidate's engine, left in this
repository's `target/release/` while the checkout moved on, judged a valid
corpus invalid three times, and a session attributed the warning to the wrong
binary. `--version` could not have separated them.

### 1.2 The floor could not help either

Spec 061 §3.8 requires this repository's `[meta] required_version` to match
its own package version, and records keeping it current as a follow-on (its
2026-09-08 decision). It stayed at `>=0.17.0`. A 0.22.0 or 0.21.0 reader
passed the floor, read frontmatter its grammar predates, and reported
`INVALID` (exit 1) instead of refusing (exit 3). With a version that
distinguishes the expansion line, the floor can do what 055 and 061 built it
for.

### 1.3 Why a spec rather than a waiver

The 0.22.0 bump (#291) cleared `C-001` on `npm/package.json` and
`py/pyproject.toml` with a waiver, on the reasoning that no spec governed the
version literal and a release spec would record no unwritten rule. Both
premises changed on 2026-09-23. The owner stated the rule this spec records
(§3.1): materially different releases do not share a package identity, and
the frozen candidate is preserved rather than recut. That rule is about the
version field of exactly these manifests. Claiming them through `extends`
edges on their owners records it where the gate can see it, and the next bump
edits this spec's record rather than asking for an unscoped waiver again.

The refusal a waiver would have been asked to clear, measured on 2026-09-23 by
committing the bump alone (manifests, lockfile, floor, regenerated fixtures)
on `main` at `0bf9ff78` with no spec, and running `couple --base origin/main`:
**24 `C-001`**, one per changed owned path.

```
C-001 'npm/package.json' changed without an authoring edit to any owning spec (006-distribution, 092-the-engine-ships-governance-not-an-environment)
C-001 'py/pyproject.toml' changed without an authoring edit to any owning spec (007-python-distribution)
C-001 'spec-spine.toml' changed without an authoring edit to any owning spec (061-shipped-is-not-the-same-as-working, 092-the-engine-ships-governance-not-an-environment, 101-readiness-is-scheduling-not-approval, 105-governed-scope-is-enabled-here)
C-001 'crates/spec-spine-core/fixtures/verifier/<case>/{case,payload}.json' changed without an authoring edit to any owning spec (001-compile-registry, 103-a-verifier-fixture-is-a-published-artifact, 106-obligations-are-declared-constraints, 109-impact-and-conflict-are-declared, 110-an-interface-reference-is-digest-pinned)   (21 files, one line each)
```

The earlier bump's waiver covered two of these because the fixture set did
not exist then. A waiver for this one would have had to excuse the
configuration and 21 published fixture files as well, which is the
"clears more than you meant" shape spec 113 describes.

If the owner prefers the waiver route for the two manifests, spec 113 now lets
it be scoped to them (`-Paths: npm/package.json, py/pyproject.toml`), and this
spec's other claims stand without those two edges.

## 2. Territory

Extends the owners of the two shim manifests, the configuration, the release
runbook, the instruction file and the verifier fixtures, for the version
literal and the floor only. Establishes the identity script.

## 3. Behavior

### 3.1 A version names one behavior

Once `main` differs from a frozen or published candidate in any engine source
or schema version, `main`'s package version MUST differ from that candidate's.
The candidate is not recut to follow `main`: its record names its revision and
it keeps it.

Applied here: the frozen candidate stays `0.22.0` at
`f9fa6a8f56b82c8d97cf2803bc31838a0b455a21`, and `main` moves to `0.23.0`, the
version the next expansion release is cut at. The bump is
`scripts/bump_version.py 0.23.0` with `--check 0.23.0` green: the workspace
`Cargo.toml` (version and the two internal pins), `npm/package.json` (version
and the five platform pins), `py/pyproject.toml`, and the three workspace
crates' entries in `Cargo.lock`, and nothing else in the lockfile.

A version is still not a source identity. A development build of `main` and
the eventual 0.23.0 release will both answer `0.23.0` until the release is
cut, which is why §3.4 exists.

### 3.2 The floor moves with the version

`spec-spine.toml` `[meta] required_version` MUST be `>=` this repository's
package version, spec 061 §3.8's "matching this repository's own package
version": `>=0.23.0` when this spec was filed, `>=0.24.0` since D-3, `>=0.25.0` since D-4, `>=0.26.0` since D-5, `>=0.27.0` since D-6, `>=0.28.0` since D-7. A reader older than
0.23.0 then refuses this corpus with exit 3, naming the requirement, the
running version and where the pin lives, instead of reading a grammar it
predates. Measured: the candidate's build (`0.22.0`) and the `0.21.0` build
both answer

```
spec-spine: config error: this repository requires spec-spine >=0.23.0 (spec-spine.toml [meta] required_version); running 0.22.0. Install the required version, or change the pin deliberately
```

which spec 123's hooks report as `NOT READ` with the binary named, and which no
session can mistake for a finding about the corpus.

The floor is one-directional (061's decision stands): a newer reader passes
it, and within one version the in-tree build's age (spec 123) is what
separates a current reader from a stale one.

### 3.3 Both are standing steps

`docs/releasing.md` gains two checklist items beside the bump: move the floor
to `>=<version>` (which `bump_version.py` deliberately does not touch), and
treat a version as naming one behavior, recording each built binary's
identity with §3.4's script rather than from `--version`. `CLAUDE.md`'s floor
sentence stops naming a literal that goes stale, and names the script.

### 3.4 A binary's identity is recorded from evidence

`scripts/reader-identity.sh BINARY [CHECKOUT]` prints, one line each:

- the executable's absolute path and SHA-256 digest;
- what it answers to `--version`;
- the checkout's revision and whether its tracked tree is clean;
- whether the binary is that checkout's in-tree build with no compiled input
  newer than it (the comparison spec 123 makes, from cargo's dep-info), or why
  that cannot be said;
- each schema axis **as the binary emits it**: registry and index from the
  shards it writes for a one-spec scratch corpus, read documents from
  `registry plan --json`, the verdict envelope from `check --json`.

Nothing is read from the source for the schema lines, so a stale build cannot
report the source's numbers. The script reads only; its scratch corpus lives in
a temporary directory and is removed. It exits 0 when every line was measured,
1 when any was not (the line says why), and 3 on a usage error.

### 3.5 The verifier fixtures follow the producer

Spec 103's fixture set records the producer's version, so a bump makes
`the_fixture_set_describes_the_shipped_verifier` refuse with its own
remedy. Regenerated with `generate.py` at 0.23.0: the diff is exactly
`toolVersion`, the payload `version` literal, `registryHash` and
`attestationHash` (the registry records the compiler's version), which is the
movement spec 110 D-2 measured for a schema MINOR. No case changed its
outcome.

## 4. Out of scope

- **Tagging, publishing or recutting anything.** The owner decides 0.22.0's
  tag and 0.23.0's release separately; nothing here pushes a tag.
- **Embedding a source revision in the binary.** Spec 123 D-1.
- **Making `bump_version.py` move the floor.** The script is spec 105's and
  its "does not touch" is documented behavior; the runbook step is the
  change.
- **The PATH binary.** Nothing here installs anything.

## 5. Resolved decisions

**D-1 (2026-09-23, 0.23.0 rather than a pre-release).** A pre-release
(`0.23.0-dev`) would need three spellings, since PyPI's is `0.23.0.dev0`, which
`bump_version.py` and 006 §3.5's parity do not support, and `required_version`
comparisons against pre-releases have their own semver rules. The plain MINOR
matches this repository's practice (the 0.22.0 bump preceded its tag the same
way). The limit is stated in §3.1: until 0.23.0 is cut, the version names the
expansion line, and §3.4 names the build.

**D-2 (2026-09-23, review of #322).** The script trusted only absolute
dep-info entries under the checkout, so a record written with cargo's
`build.dep-info-basedir` (relative paths) matched nothing and the script
reported "built-from: yes" having compared nothing. A relative entry is now
resolved against the checkout, and a record that names none of the checkout's
files is reported `NOT MEASURED` (exit 1), never as current. A missing
`python3` is named as that, not reported as a missing shard. Exercised on a
scratch checkout: a relative entry newer than the build reports `NO`, and a
record naming only another directory's file reports `NOT MEASURED`.

**D-3 (2026-09-23, the 0.24.0 bump).** 0.23.0 was published from `d2bb4763`.
`main` then merged engine and schema changes beyond it: specs 116 (#327), 112
(#328) and 114 (#329, registry `1.7.0`). §3.1 applies again, so `main` moves to
0.24.0, the version the next release is cut at, and, as §1.3 prescribes, the
bump edits this record rather than asking for a waiver:
`scripts/bump_version.py 0.24.0` with `--check 0.24.0` green, the three
workspace crates' `Cargo.lock` entries, the floor at `>=0.24.0`, and the
fixture set regenerated with `generate.py` (the four-field movement §3.5
describes, 21 files, no case changed its outcome). The Verification block's
version literals follow the bump; its shape and every assertion it makes are
unchanged. Published 0.23.0 is not recut.

**D-4 (2026-09-24, the 0.25.0 bump).** 0.24.0 was published from `a812f72d`.
`main` then merged engine changes beyond it: specs 126 (#336), 127 (#337),
128 (#338) and 129 (#339), each of which changes what a verb or a facade entry
refuses. §3.1 applies again, so `main` moves to 0.25.0, the version the next
release is cut at, and the bump edits this record rather than asking for a
waiver: `scripts/bump_version.py 0.25.0` with `--check 0.25.0` green, the
three workspace crates' `Cargo.lock` entries, the floor at `>=0.25.0`, and the
fixture set regenerated with `generate.py` (the same four-field movement, 21
files, no case changed its outcome). The Verification block's version
literals follow the bump; its shape and every assertion it makes are
unchanged. Published 0.24.0 is not recut. No schema axis moves.

**D-5 (2026-09-24, the 0.26.0 bump).** 0.25.0 was published from `25d46b9f`.
`main` then merged engine changes beyond it: specs 130 (#346), 131 (#347,
#352), 132 (#355), 134 (#351) and 135 (#354), which change a classification,
a facade entry, every exit code and the envelope. §3.1 applies again, so
`main` moves to 0.26.0, the version the next release is cut at, by the same
steps as D-4: `scripts/bump_version.py 0.26.0` with `--check` green, the
three workspace crates' `Cargo.lock` entries, the floor at `>=0.26.0`, and
the fixture set regenerated with `generate.py`. The Verification block's
version literals follow the bump. Published 0.25.0 is not recut.

**D-6 (2026-09-25, the 0.27.0 bump).** 0.26.0 was published from `8f2a8f75`.
`main` then merged engine changes beyond it: specs 141 (#362), 142 (#363),
143 (#364), 144 (#365) and 145 (#366), which move the index and registry
schemas, the delta report, what the freshness reads accept, the configuration
rules and the guarded readers' refusals, plus the clap 4.6.7 lockfile bump
(#275). §3.1 applies again, so `main` moves to 0.27.0 by the same steps as
D-5, and the Verification block's version literals follow the bump.
Published 0.26.0 is not recut.

**D-7 (2026-09-26, the 0.28.0 bump).** 0.27.0 was published from `d78fb09a`.
`main` then merged engine changes beyond it: specs 147 (#393, links at and
above the derived and state roots are checked on read), 152 (#395, verdict
1.1.0, envelopes for every `--json` read's failure, the compact refusal), and
the tests and scripts of 148 (#394), 149 (#396), 150 (#397) and 154 (#398).
§3.1 applies again, so `main` moves to 0.28.0 by the same steps as D-6, and the
Verification block's version literals follow the bump. Published 0.27.0 is not
recut.

## Verification

Written to fail against the tree this spec is filed on: the version is 0.22.0,
the floor is `>=0.17.0`, and the script does not exist.

```verify:cli
cargo build --release --locked
# 3.1: one version in all three manifests, and the build answers it.
python3 scripts/bump_version.py --check 0.28.0
./target/release/spec-spine --version | grep -qx 'spec-spine 0.28.0'
# 3.2: the floor matches the version.
grep -qx 'required_version = ">=0.28.0"' spec-spine.toml
# 3.3: both standing steps are in the runbook.
grep -qF 'The floor moves with the version' docs/releasing.md
grep -qF 'A version names one behavior' docs/releasing.md
# 3.4: the identity record measures every line from the binary itself.
scripts/reader-identity.sh target/release/spec-spine . > "${TMPDIR:-/tmp}/ss124.txt"
grep -qE '^answers +spec-spine 0\.28\.0$' "${TMPDIR:-/tmp}/ss124.txt"
grep -qE '^sha256 +[0-9a-f]{64}$' "${TMPDIR:-/tmp}/ss124.txt"
grep -qE '^registry schema +[0-9]+\.[0-9]+\.[0-9]+$' "${TMPDIR:-/tmp}/ss124.txt"
grep -qE '^read schema +[0-9]+\.[0-9]+\.[0-9]+$' "${TMPDIR:-/tmp}/ss124.txt"
grep -qE '^verdict schema +[0-9]+\.[0-9]+\.[0-9]+$' "${TMPDIR:-/tmp}/ss124.txt"
rm -f "${TMPDIR:-/tmp}/ss124.txt"
sh -c 'scripts/reader-identity.sh; test $? -eq 3'
# 3.5: the fixture set describes the producer at this version.
cargo test -p spec-spine-cli --test verifier_fixtures --locked
```
