---
id: "069-the-shipped-default-hashes-what-it-names"
title: "The shipped default hashes what it names"
status: approved
kind: "tooling"
created: "2026-09-07"
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "004-codebase-index"
  - "057-claimed-but-unwitnessed"
  - "061-the-scaffold-ships-what-adopters-wrote"
  - "067-the-docs-name-what-adopters-derived"
amends:
  # 061 §4 emitted the key at its real (broken) value with a comment saying it
  # is left as-is because fixing it would restale every adopter. Both halves of
  # that sentence stop being true here, so the comment it mandates changes.
  - "061-the-scaffold-ships-what-adopters-wrote"
extends:
  # §3.1 The default value itself.
  - { spec: "062-a-version-pin-the-cli-can-check", unit: "crates/spec-spine-types/src/config.rs", nature: additive }
  # §3.2 The scaffold comment 061 §3.2 mandates, and the assertion beside it.
  - { spec: "061-the-scaffold-ships-what-adopters-wrote", unit: "crates/spec-spine-core/src/scaffold.rs", nature: additive }
  - { spec: "061-the-scaffold-ships-what-adopters-wrote", unit: "crates/spec-spine-core/tests/scaffold.rs", nature: additive }
  # §3.3 The regression guard: the default must match files, not directories.
  - { spec: "055-the-ledger-answers-what-consumers-rebuild", unit: "crates/spec-spine-core/tests/index.rs", nature: additive }
  # §3.4 The adopter-facing default table and the bypassed-vs-hashed example.
  - { spec: "067-the-docs-name-what-adopters-derived", unit: "docs/adoption-guide.md", nature: additive }
  # §3.6 030's workflow auto-waive test asserted a freshness premise that the
  # broken default was the only thing making true.
  - { spec: "030-cargo-workflow-dependency-waiver", unit: "crates/spec-spine-cli/tests/couple.rs", nature: additive }
references:
  - { unit: { kind: file, path: "docs/design/00-architecture.md" }, role: context }
summary: >
  `IndexConfig::default().extra_hashed_inputs` ships as
  `["standards/**", ".github/workflows/**"]`. In the glob crate `dir/**`
  enumerates directories, and the hasher keeps only entries that are files, so
  the shipped default matches nothing: every adopter who has not overridden it
  folds zero extra files into the content hash and is told nothing. Spec 057
  found the form in this repository's own config and fixed it there; spec 061
  found it in the default behind it, emitted the key at its real value with a
  comment naming the trap, and deferred the fix to its own spec on the grounds
  that changing it restales every adopter's committed index. This is that spec.
  The default becomes `["standards/**/*", ".github/workflows/**/*"]`, the
  working form 061's comment already tells adopters to write, so an adopter who
  followed that advice sees no change and one who did not gets a one-time
  restale and a governance surface that is finally in the ledger. A regression
  test pins the property that was never asserted: that the shipped default
  matches files.
---

# 069: The shipped default hashes what it names

## 1. Purpose

A scaffolded adopter can rewrite its entire constitution and the ledger will
report itself fresh. That is not a hypothetical:

```console
$ spec-spine --repo /tmp/probe init && spec-spine --repo /tmp/probe compile && spec-spine --repo /tmp/probe index
$ printf '\nA governed edit.\n' >> /tmp/probe/standards/spec/constitution.md
$ spec-spine --repo /tmp/probe index check
index is fresh
```

The cause is one character short in a shipped default:

```rust
extra_hashed_inputs: vec![
    "standards/**".to_string(),
    ".github/workflows/**".to_string(),
],
```

In the `glob` crate `**` matches a sequence of path components, so `standards/**`
enumerates `standards`, `standards/spec`, `standards/spec/templates`, and nothing
else: every entry it yields is a directory. Both `glob_files` helpers then apply
`.filter(|p| p.is_file())`, which discards all of them. The pattern is
well-formed, the config parses, `config show` prints it back, and it matches zero
files.

The consequence is not merely a missing hash. `standards/` is on the coupling
gate's bypass floor, so a change there raises no `C-001` by design: the ledger's
content hash is the only mechanism that was supposed to notice. Spec 023 signs
attestations over that hash, which means an adopter's signed attestation has been
attesting to a corpus whose governing documents it never read. The one surface
where "nothing detects this" is unacceptable is precisely the one the default
left uncovered.

Two specs have already walked past this. Spec 057 found the identical form in
**this repository's** `spec-spine.toml`, fixed it here, and did not look at the
default behind it. Spec 061, writing out the exhaustive scaffolded config,
found the default itself and declined to fix it, on the reasoning that changing
a shipped default restales every adopter's committed index and therefore
"belongs in its own spec with its own release note". This is that spec, and the
release note is §3.5.

## 2. Territory

This spec owns no file. Every change lands in a unit another spec already holds,
declared as an `extends` edge in the frontmatter:

| Unit | Owner crossed | What changes |
|---|---|---|
| `crates/spec-spine-types/src/config.rs` | 062 (latest claimant) | the default value (§3.1) |
| `crates/spec-spine-core/src/scaffold.rs` | 061 | the comment 061 §3.2 mandates (§3.2) |
| `crates/spec-spine-core/tests/scaffold.rs` | 061 | the assertion that the emitted default works (§3.2) |
| `crates/spec-spine-core/tests/index.rs` | 055 (latest claimant) | the regression guard (§3.3) |
| `docs/adoption-guide.md` | 067 | the default column and the worked example (§3.4) |

`docs/design/00-architecture.md` quotes the default in a design narrative and is
owned by no spec; it is a `references` edge, and §3.4 updates it.

## 3. Behavior

### 3.1 The default matches files

`IndexConfig::default().extra_hashed_inputs` MUST be
`["standards/**/*", ".github/workflows/**/*"]`.

The value is the working spelling of the intent the broken form already
declared: every file under the standards tree and every file under
`.github/workflows`, recursively, of any extension. It MUST NOT be narrowed to
a set of extensions in this spec (§4), and it MUST be character-for-character
the form spec 061's scaffold comment already tells adopters to write, so that an
adopter who followed that advice observes no change at all.

### 3.2 The scaffold stops describing the default as broken

`scaffold.rs`'s comment above the key currently reads, in part, that "the
default below carries the broken form and so hashes nothing" and that "it is
left as-is here because changing the default would restale every existing
adopter's index". Both clauses MUST go: they will be false.

The comment MUST still name the trap, because the trap outlives the default:
an adopter narrowing or extending this list can still write `dir/**` and get
silence. It MUST retain the literal phrase `matches DIRECTORIES`, which spec
061's own `## Verification` block greps for, and it SHOULD name spec 069 as
where the default was fixed so the history stays readable from the file.

`tests/scaffold.rs` MUST gain an assertion that the emitted config carries the
working form and does **not** carry the bare `"standards/**"` entry. The
existing round-trip assertion (the emitted file parses to a `Config` equal to
`Config::default()`) MUST continue to hold unchanged; it adapts to the new
default on its own, which is why it is not sufficient as the guard.

### 3.3 A regression guard that fails on the old default

`tests/index.rs` MUST gain a test asserting the property no test held: that the
**shipped default** folds real files into the content hash.

The test MUST build a fixture repository containing at least one file under
`standards/` and one under `.github/workflows/`, MUST use `Config::default()`
rather than a hand-written config (a test that spells its own globs cannot fail
when the default rots), and MUST assert that both files appear among the hashed
inputs. It MUST fail against the pre-069 default.

Asserting on the set of hashed paths is preferred to asserting on the hash
value, which would be a golden test that changes whenever anything else in the
fixture does.

### 3.4 The documentation stops quoting the broken value

`docs/adoption-guide.md` MUST change in three places:

1. the `index.extra_hashed_inputs` row of the config table, whose default column
   currently reads `["standards/**", ".github/workflows/**"]`;
2. the non-default example further down, which copies the broken form into a
   snippet an adopter is invited to paste;
3. the bypassed-versus-hashed worked example, whose second bullet claims
   `standards/**` is "bypassed and hashed". That bullet is the one place the
   guide asserts the broken form works, and it is the sentence this spec makes
   true.

The **"Watch the glob form" callout MUST be kept**, and its historical sentence
MUST remain accurate. The trap it describes is a property of the glob crate, not
of the old default, and the callout is the only place an adopter writing their
own pattern is warned.

`docs/design/00-architecture.md` quotes the default in its config narrative and
MUST be updated to match. It is owned by no spec and sits on the bypass floor,
so this is a consistency edit, not a governed claim.

### 3.5 The migration note

The change is a one-time restale for an adopter who relies on the default, and
that adopter MUST be able to find out why before it happens to them. The note
MUST read, in substance:

> `[index] extra_hashed_inputs` shipped a default that matched no files. It is
> fixed. If you did not override the key, your next `spec-spine index` will
> rewrite every shard once, because the standards tree and the workflow
> directory are entering the content hash for the first time. Commit the result.
> Nothing about what staleness *means* has changed; a surface that was silently
> outside the ledger is now inside it.

It MUST live in `docs/adoption-guide.md`, beside the knob it concerns. This
repository publishes no `CHANGELOG.md`, and `release.yml` sets
`generate_release_notes: true`, so GitHub composes the release body from merged
pull request titles: there is no committed release-notes file for a migration
note to live in, and a note that exists only in a release body is invisible to
an adopter who upgrades by version number. The pull request title for this spec
MUST therefore name the restale as well, since that title *is* the generated
release note.

`spec-spine`'s own corpus is unaffected: its `spec-spine.toml` overrides the key
with `["standards/**/*.md", ".github/workflows/*.yml"]`, which already matches.
That MUST NOT be taken as evidence the change is inert, and is the reason §3.3
requires a test rather than trusting the gate.

### 3.6 A workflow bump stales the ledger, and the test says so

`.github/workflows/**/*` is half the default, so fixing it makes a workflow edit
stale every shard for an adopter on the default. `crates/spec-spine-cli/tests/couple.rs`
asserts the opposite today, in spec 030's `workflow_uses_bump_auto_waives`:
that a Dependabot action-ref bump leaves the index fresh. That assertion MUST be
corrected rather than preserved, and the test MUST keep its actual subject,
which is that the bump self-clears the **coupling** gate.

The corrected test MUST assert the real sequence: the bump stales the index
(exit 2), a re-index restores it, and `couple` then auto-waives. It MUST NOT be
made to pass by giving the fixture a config that excludes workflows from the
hash, which would leave the suite asserting a configuration nobody runs.

This is not a behavior spec 069 invents. `spec-spine`'s own `spec-spine.toml`
has carried `.github/workflows/*.yml` since spec 057, so a workflow edit has
staled every shard **in this repository** since then, and a probe confirms it
still does. What changes is that adopters on the default now get the behavior
this repository already has.

## 4. Out of scope

**Narrowing the default to a set of extensions.** `["standards/**/*.md",
".github/workflows/*.yml"]`, which this repository uses, hashes less and would
spare an adopter a restale from an editor swapfile under `standards/`. It is
also not what the broken default said, and inventing a narrower intent inside a
bug fix is how a fix becomes a behavior change nobody asked for. An adopter who
wants the narrower form writes it; the scaffolded comment shows how.

**A lint that refuses `dir/**` in any `extra_hashed_inputs`.** This is the
generalisation, and it is genuinely attractive: it would catch the trap in an
adopter's hand-written config, which no mechanism does today. `L-008` catches
only the case where the unhashed path is also **claimed**, and this repository
suppresses 71 of those through `[lint] unwitnessed_allowed`. It is a new
diagnostic code with its own false-positive question (`dir/**` is legitimate in
a `[index.slices]` entry meant to match nothing yet), and it belongs in its own
spec.

**Deduplicating the two `glob_files` helpers.** `index.rs` and `shard.rs` carry
byte-identical private copies, and the `.filter(|p| p.is_file())` line that makes
this bug bite is duplicated in both. Consolidating them is a refactor across two
specs' territory with no behavior change, and doing it here would bury the
one-line fix this spec exists to make.

**A workflow freshness projection.** `Cargo.toml` and `package.json` fold into
the hash as governance *projections* (spec 004 §3.5, extended by 030 §3.1): the
manifest with its dependency tables stripped, so a version bump moves no hash.
No such projection exists for a workflow, so a `uses:` action bump moves the
global scalar and stales every shard, and a bot that can neither re-index nor
waive walls on exit 2. Spec 030 did not build one because it read workflow
freshness as already fine. It was not, and §3.6 records why. Building the
projection (strip `uses:` refs, hash the rest) is the real fix and is its own
spec: it is a hashing-semantics change with a migration of its own, and folding
it into a one-line default correction would hide it.

**Changing what enters the content hash.** The set of hashed inputs is
`spec-spine.toml`, every `spec.md`, each package manifest, the span-backing
files of resolved units, and every `extra_hashed_inputs` match. This spec
changes which files the last of those five resolves to, and touches none of the
other four. Spec 057 §4's separate question, whether bare `file` units should
contribute their bytes, stays open.

## 5. Resolved decisions

**Decision, 2026-09-07: the restale is accepted, not mitigated.** The
alternative considered was a compatibility path: keep the broken default,
detect it at load, and warn. That preserves every adopter's committed index at
the cost of shipping a default the tool itself reports as wrong, and it leaves
the attestation gap open for anyone who does not read warnings. A default that
does not do what it says is a defect and not a contract, so there is nothing to
preserve compatibility with. The restale is one commit for an adopter and is
what a correct ledger costs.

**Decision, 2026-09-07: 067 is extended, not amended.** Spec 067 records, as a
finding an adopter derived by experiment, that `standards/**` is in the default
`extra_hashed_inputs` and that editing a standards file therefore stales every
shard. The second half of that sentence is false today and true after this spec.
069 is not contradicting 067's stated behavior; it is making 067's description
of the world accurate. The doc file 067 owns is edited under an `extends` edge,
and 067's own `spec.md` is not touched (spec 040).

**Decision, 2026-09-07: spec 030 is not amended.** 030 §1 says that for GitHub
Actions "a `file` unit carries no span and is not a hashed input (004 §3.5), so
freshness is already fine". The first clause is true and is about the unit; the
conclusion is false whenever an `extra_hashed_inputs` glob covers the same file,
which in this repository it has since spec 057. So 069 does not change 030's
stated behavior: 030 implements a coupling waiver, that waiver is untouched, and
its test still asserts it. What 069 falsifies is a premise in 030's problem
statement that was already false when written. A premise is corrected by the
record, not by an `amends` edge, so this entry is the record and the test is
edited under an `extends` edge on the file.

**Decision, 2026-09-07: the migration note goes in the adoption guide.** The
first draft of §3.5 required it "in the release notes", which named no file.
There is no `CHANGELOG.md` here and the release workflow generates its body from
pull request titles, so that requirement was unsatisfiable as written. The guide
is where an adopter reads about the knob, so it is where the note about the knob
changing belongs. `docs/releasing.md` was considered and rejected: it is the
maintainer's runbook for cutting a release, not a document adopters read, and it
is owned by spec 007, a crossing this spec has no other reason to make.

**Decision, 2026-09-07: the edge on `config.rs` names 062.** No spec
`establishes` `crates/spec-spine-types/src/config.rs`; it is covered by the
`000` package-manifest floor and claimed by seven `extends` edges. Spec 057 set
the convention by naming the most recent claimant (053, at the time), and this
spec follows it by naming 062. The alternative, naming the `000` floor, would
assert a crossing into the bootstrap spec's territory that the ownership model
does not actually record.

## Verification

Each line below is one command: spec 049 §3.2 makes a fence's body line a
command, so a trailing backslash continuation would become its own fragment and
fail. The adopter is materialized once at a fixed path, because each line is its
own shell and a `$(mktemp -d)` would not survive to the next assertion.

Every assertion fails against pre-069 code: the default matched no files, so the
scaffolded adopter's `index check` reported fresh across an edited constitution,
and the emitted config carried the bare `"standards/**"` entry.

```verify:cli
# Self-contained: the commands below invoke the release binary.
cargo build --release --locked
# 3.3 the regression guard, and 3.2's assertion beside the round-trip.
cargo test -p spec-spine-core --test index --locked
cargo test -p spec-spine-core --test scaffold --locked
# 3.6 030's workflow auto-waive test, corrected to the real sequence.
cargo test -p spec-spine-cli --test couple --locked
# 3.4 the guide keeps the trap warning and 3.5 puts the migration note beside it.
grep -q 'Watch the glob form' docs/adoption-guide.md
grep -q 'Upgrading across spec 069' docs/adoption-guide.md
# Materialize a fresh adopter on the shipped default.
rm -rf "${TMPDIR:-/tmp}/ss069" && mkdir -p "${TMPDIR:-/tmp}/ss069" && target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss069" init >/dev/null
# 3.1: the emitted default is the working form...
grep -q 'extra_hashed_inputs = \["standards/\*\*/\*", ".github/workflows/\*\*/\*"\]' "${TMPDIR:-/tmp}/ss069/spec-spine.toml"
# ...and the bare directory form is gone from the file.
! grep -q '"standards/\*\*"' "${TMPDIR:-/tmp}/ss069/spec-spine.toml"
# 3.2: the trap is still named, in the phrase 061's own verification greps for.
grep -q 'matches DIRECTORIES' "${TMPDIR:-/tmp}/ss069/spec-spine.toml"
# 3.2: the emitted config still round-trips through the real loader.
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss069" config show >/dev/null
# 3.1: a fresh adopter's ledger starts clean...
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss069" compile >/dev/null
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss069" index >/dev/null
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss069" index check
# ...and an edit to the constitution now stales it, which is the whole spec.
printf '\nA governed edit.\n' >> "${TMPDIR:-/tmp}/ss069/standards/spec/constitution.md"
! target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss069" index check
# The same holds for the workflow directory, the default's other half.
mkdir -p "${TMPDIR:-/tmp}/ss069/.github/workflows" && printf 'name: ci\n' > "${TMPDIR:-/tmp}/ss069/.github/workflows/ci.yml"
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss069" index >/dev/null
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss069" index check
printf 'name: ci2\n' > "${TMPDIR:-/tmp}/ss069/.github/workflows/ci.yml"
! target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss069" index check
rm -rf "${TMPDIR:-/tmp}/ss069"
```
