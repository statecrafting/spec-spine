---
id: "083-an-attestation-covers-the-territory-it-claims"
title: "An attestation covers the territory it claims"
status: draft
kind: "tooling"
created: "2026-09-10"
implementation: pending
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "017-directory-crate-module-units"
  - "023-ledger-seal"
  - "042-per-spec-attestation"
extends:
  # 3.1 to 3.4: a directory-resolved location is walked, not opened.
  - { spec: "023-ledger-seal", unit: "crates/spec-spine-core/src/attest.rs", nature: additive }
  # 3.5: the regression guards, one per affected unit kind.
  - { spec: "023-ledger-seal", unit: "crates/spec-spine-core/tests/attest.rs", nature: additive }
  # 3.2: the shared walker gains an unfiltered mode, so the exclusion policy
  # stays in one place instead of being copied into attest.rs.
  - { spec: "004-codebase-index", unit: "crates/spec-spine-core/src/index.rs", nature: additive }
  # 3.5: the end-to-end exit-code guard.
  - { spec: "001-compile-registry", unit: "crates/spec-spine-cli/tests/cli.rs", nature: additive }
references:
  - { unit: { kind: file, path: "specs/042-per-spec-attestation/spec.md" }, role: context }
summary: >
  `attest_spec` hashes a unit by opening each resolved location with
  `read_to_string`. Several unit spellings resolve to a directory rather than to
  a file, so the read fails with `Is a directory` and the verb exits 3: fourteen
  of this corpus's eighty-three specs cannot be attested at all, and the failure
  arrives as an I/O error rather than as a verdict. The dominant spelling is not
  the obvious one: thirteen of the fourteen are plain `file` units whose path is
  a subtree (`npm/`, `.claude/skills/`), which spec 000 4.2 defines as resolving
  identically to an explicit `directory` unit. Spec 042 3.1 already states what should happen, that `units` carries
  "the content hash of what it resolved to" for every owning unit, without
  qualifying by kind, so this spec changes nothing 042 requires and supplies the
  implementation that sentence never had. A directory-resolved location is
  walked instead of opened: every file beneath it, pruned by
  `index.resolver_exclusions` and `layout.state_dir` through the indexer's
  existing walker, folded into the same path-sorted `hash::content_hash`
  construction the file units already use. The walk is deliberately not
  extension-filtered, because the coverage universe's source extensions would
  hash zero files for a markdown-only directory such as `.claude/agents/` and
  emit a confident hash over nothing. A directory that walks to no files at all
  hashes its own path as a single piece, so an empty claim cannot attest as
  SHA-256 of the empty input, a constant every empty claim would otherwise
  share. No previously emitted payload changes value, because every spec this
  reaches could not produce one.
---

# 083: An attestation covers the territory it claims

## 1. Purpose

`spec-spine attest --spec <id>` cannot attest fourteen of this corpus's
eighty-three specs. It does not report a bad verdict for them. It fails to
produce a payload at all:

```
$ spec-spine attest --spec 048-kit-ships-the-governed-loop-skills
spec-spine: io error: read /.../.claude/agents/ for spec
'048-kit-ships-the-governed-loop-skills' unit
Directory { path: ".claude/agents/", planned: false }: Is a directory (os error 21)
$ echo $?
3
```

`attest.rs::attest_spec` hashes a unit by opening every resolved location with
`fs::read_to_string`. That is correct for a location that is a file and wrong
for a location that is a directory, and the defect is best read at the location
rather than at the declared kind, because three different spellings land in the
same place:

- a **`file` unit whose path is a subtree**, the trailing-slash shorthand.
  `resolve_unit` admits it on a bare `abs.exists()`, which is true of a
  directory, and returns the directory as the location.
- an explicit **`{ kind: directory, path }` unit**, which resolves to the
  directory itself (`I-007` when it is missing).
- a **`{ kind: crate, id }` unit**, which resolves to the discovered package's
  root directory.

Spec 000 4.2 already states that the first two are one concept: a directory unit
is "a subtree named explicitly; resolution is identical to a trailing-slash file
unit". A fix keyed to the declared kind rather than to the resolved location
would therefore repair one spelling and leave the other broken.

Measured across the corpus, the specs that cannot be attested are 007, 008,
020, 029, 048, 051, 064, 068, 072, 074, 075, 077, 081 and 082. That is not a
random seventeen percent. It is the distribution shims, the kit, and the harness
specs: the territory an adopter is most likely to want evidence about, and the
territory whose claims are most often expressed as a subtree because that is
what a shipped directory of skills or agents is.

Measured by the shape each one trips on, the breakdown is the opposite of what
the vocabulary suggests: **thirteen fail on a `file` unit** whose path is a
subtree (`npm/` on 007, `.claude/skills/` on 082), **one on an explicit
`directory` unit** (048, `.claude/agents/`), and **none on a `crate` unit**,
because this corpus declares no owning crate unit at all. The crate path is
still specified below, since the code reaches it and an adopter's corpus will
exercise it; it is simply not exercised here.

Spec 082 is on the list, which is how the defect surfaced. A reviewer asked for
a per-spec attestation as evidence on 082's ratify PR, and the verb could not
produce one for that spec.

Two properties break, and the second is the worse of the two.

**The evidence is unavailable exactly where it was designed to be used.** Spec
042's purpose is answering "was this spec's territory sound when its work was
declared done" for one spec. A verb that answers that question for sixty-nine
specs and errors for fourteen has not been reduced in scope by anyone's
decision; the scope reduction is invisible until someone runs it.

**The failure is an I/O error, not a verdict.** Spec 042 3.1 is deliberate that
a failing verdict never suppresses the payload, since "an attestation that
refused to exist when the news was bad would be worth nothing as evidence".
Exit 3 is the case that rule does not reach, because the payload is never built
at all. The verb has a considered answer for bad news and no answer for a unit
kind it cannot read.

## 2. Territory

No new files. Four units, all owned elsewhere and all `extends`-ed here:

| Unit | Owner | What changes |
|---|---|---|
| `crates/spec-spine-core/src/attest.rs` | 023 | a directory-resolved location is walked, not opened |
| `crates/spec-spine-core/src/index.rs` | 004 | the shared walker gains an unfiltered mode |
| `crates/spec-spine-core/tests/attest.rs` | 023 | the regression guards for 3.1 to 3.5 |
| `crates/spec-spine-cli/tests/cli.rs` | 001 | the end-to-end exit-code guard |

This spec changes no stated requirement of spec 042, so it declares no `amends`
edge. Spec 042 3.1 already says `units` lists owning units "each with the
content hash of what it resolved to", with no qualification by kind, and spec
017 predates 042, so directory and crate units existed in this corpus when that
sentence was written. What lands here is conformance to the owning spec, which
is the one kind of change an implementing spec may make inside another's
territory without amending it. See 5, D-1.

## 3. Behavior

### 3.1 Every owning unit that resolves MUST attest a content hash

`attest_spec` MUST NOT return `Err` because a resolved location is a directory.
For every owning unit the indexer resolved to at least one location, the emitted
`AttestedUnit` MUST carry `contentHash: Some(_)`.

The rule is keyed to the **resolved location**, never to the declared kind. A
`file` unit whose path is a subtree MUST take the directory path of 3.2 exactly
as an explicit `directory` unit does. An implementation that matched on
`Unit::Directory` and `Unit::Crate` would leave thirteen of the fourteen specs
named in 1 still failing.

The existing contract for a unit that did **not** resolve is unchanged:
`contentHash: null` and `resolution.ok: false`, per spec 042 3.1. This spec
distinguishes "resolved to something this code cannot open" from "did not
resolve", which are currently the same crash and were never the same fact.

A read failure on a location that should be readable MUST still propagate as an
error, exactly as spec 042 3.1 requires. Silently skipping an unreadable file
would emit `Some(hash)` over the files that happened to open, with
`resolution.ok` left true: an attestation reporting a resolved unit whose files
are gone, which is the one thing that payload exists to make impossible.

### 3.2 A directory-resolved location is walked, under one exclusion policy

A location that is a directory MUST be expanded to the files beneath it, at any
depth, and those files folded into the unit's content hash using the project's
standing `hash::content_hash` construction: repo-relative POSIX path, NUL, the
BOM-stripped and LF-normalized bytes, sorted by path. That is the same
construction a `file` unit already uses, so a directory unit's hash is
comparable with every other hash in the payload rather than a second dialect.

The walk MUST prune `index.resolver_exclusions` and `layout.state_dir`. The
second is not optional and is not expressible as the first: spec 039 3.5 makes
the state root its own decision, which no `resolver_exclusions` entry can
express (that list matches directory names, not path prefixes) and none may
cancel.

A symlink encountered by the walk MUST NOT be traversed or dereferenced. It
contributes exactly one piece, its own repo-relative path with the link target
text as the content, and the walk does not descend into it even when it points
at a directory.

Three things follow, and the third is why this clause is normative rather than
an implementation note. A symlink cycle inside a claimed subtree cannot make the
walk diverge, which the shared walker's `path.is_dir()` recursion would
otherwise permit, since `is_dir` follows links. A symlink pointing outside the
repository cannot pull arbitrary filesystem content into a payload whose whole
purpose is to attest what this repository contains. And a symlink that is added,
removed or retargeted still moves the hash, as 3.4 requires of anything under a
claimed directory, because its target text is what was hashed. See 5, D-4.

The walk MUST NOT filter by file extension. The coverage universe's
`SOURCE_EXTS` is the wrong instrument here: `.claude/agents/` holds only
markdown, so an extension-filtered walk would find zero files and emit a
confident hash over nothing while reporting `resolution.ok: true`. A unit that
claims a directory claims what is in it, not the subset of it that happens to
compile. See 5, D-3.

The exclusion policy MUST remain single-sourced. The walk belongs in
`index.rs`, beside `walk_source`, and is called from `attest.rs`; it MUST NOT be
reimplemented locally. `spec-spine-core` already carries three separate
implementations of one short-id policy (`compile.rs::resolve_spec_dir` and
`verify.rs::resolve_spec_id` resolve against the filesystem,
`index.rs::resolve_id` against a set of ids; spec 016 2 calls the relationship a
local mirror rather than a shared call), and a fourth implementation of a
policy that **determines a hash** is worse than a fourth lookup: a
divergence between the two walkers would silently change what an attestation
covers, and nothing would fail.

### 3.3 A directory that walks to no files hashes its own path

When the walk yields no files, whether the directory is empty or everything in
it was pruned, the unit's content hash MUST be computed over a single piece
whose path is the unit's resolved directory path and whose content is empty.

The alternative is `hash::content_hash(vec![])`, which is SHA-256 of the empty
input, `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`. That
constant is identical for every empty directory in every repository. It would
appear in the payload as a sixty-four character hex string indistinguishable
from evidence, and it would compare equal between two unrelated empty claims.
Folding the path keeps an empty claim honest and distinct without inventing a
sentinel the schema does not have.

`resolution.ok` is unaffected by this case. The directory exists, so the unit
resolved; the indexer already refuses a missing directory with `I-007`.

### 3.4 Determinism, and what a recompute mismatch means

The payload stays a pure function of `(config, file contents)`. A file added,
removed or edited anywhere beneath a claimed directory is a change to that
directory's contents and MUST change the unit's hash. That is the correct
reading of a subtree claim: the unit says everything here is mine, so a new file
here is new territory, and an attestation that did not move would be asserting
otherwise.

The consequence is that an untracked local file inside a claimed directory
changes the hash, and `verify-attestation --recompute` on another machine will
report a `ContentMismatch` naming that unit. This is a true statement about two
trees that differ, not a determinism defect, and it is safe here in a way it is
not for `[index] extra_hashed_inputs`: that list feeds **committed** shard
hashes, where machine dependence would poison the ledger and the CI gate, while
an attestation is on demand, gitignored, and never committed, so the difference
surfaces at recompute as a named field rather than being baked into the
repository. See 5, D-2.

### 3.5 What must keep working

A unit whose locations are all **regular files** MUST attest byte-identically to
its pre-083 payload. The declared kind is not a safe way to state this rule: a
`file` unit is affected whenever its path is a subtree, which is the majority
case in 1, so "file units are unchanged" would be false. What is unchanged is
every location that is a file, however its unit was spelled, together with
`section` and `symbol`, which carry spans into files, and `module`, which
resolves through `modules.resolve` to real files. None of those reach the new
path.

`SPEC_ATTESTATION_SCHEMA_VERSION` MUST NOT change. This spec adds no field and
changes no field's meaning; it populates a field that previously could not be
reached whenever a unit resolved to a directory.

The regression guards in `tests/attest.rs` MUST cover a trailing-slash `file`
unit, an explicit `directory` unit, a `crate` unit, an empty directory, a
pruned-to-empty directory, a symlink (including one whose target is a directory,
which must not be descended into), and a spec whose units are all regular files,
so that a future refactor cannot reintroduce the crash, the hash over nothing,
or a walk that diverges.
The first of those is the shape the corpus actually exercises and is therefore
the guard that must not be dropped as redundant.

## 4. Out of scope

**The short id at `attest --spec`.** `attest --spec 070` is refused while
`verify 070` and `compile --spec 070` accept it. That looks like a sibling of
this defect and is not one. `registry show 070` and `registry relationships 070`
refuse it too, so the invariant that specs 049 3.2 and 056 3.1 each assert in
prose, that a spec id argument accepts the short form "as everywhere else",
holds for two verbs out of five and is enforced nowhere. Fixing it inside this
spec would fix it for `attest` alone and leave the same false sentence standing
in two approved specs. It is a corpus-wide argument about every verb that takes
a spec id, and it belongs in its own spec, filed next as 084.

**Committing per-spec attestations.** `.derived/attestation/` stays gitignored.
Spec 042 3.3, which 042 5 then cites, settled this: a committed per-spec bundle
would restale on every edit to any claimed unit and would need its own freshness
gate, buying churn rather than assurance. This spec makes the on-demand payload correct; it does not
change where it lives.

**Signing.** The seal is a key-only post-pass in the CLI and is untouched.

**A span for directory units.** A directory unit resolves to one location with
no span, and this spec keeps it that way. Whether a subtree claim should record
the file list it hashed, so a consumer can see the covered set without
re-walking, is a real question about the payload's shape and a different spec's.

**The corpus-scoped `attest`.** Spec 023's whole-corpus verb never reads unit
locations and is not affected.

**The existing walker's own symlink behaviour.** `walk_source` recurses on
`path.is_dir()`, which follows links, so the resolution and coverage walks it
already serves can descend through a symlink today. 3.2 constrains only the walk
this spec introduces for hashing, where following a link is a trust question
rather than a resolution one. Whether the resolution walk should change is a
question about spec 004's territory and its own defect, if it is one, and
belongs to a spec that has measured it rather than to this one.

## 5. Resolved decisions

**D-1 (2026-09-10): no `amends` edge on spec 042.** Considered and rejected. An
`amends` edge says the amending spec changes what the amended spec requires
(spec 040). Spec 042 3.1 requires a content hash for every owning unit and is
unchanged by this work; the sentence was right and the code never implemented
it. Declaring `amends` here would record a contradiction that does not exist and
would make the amendment history less trustworthy, not more. The reading under
which this would be an amendment is that 042 silently meant "file units only",
which its text does not say and which spec 017's presence in the corpus at the
time argues against.

**D-2 (2026-09-10): an untracked file inside a claimed directory legitimately
changes the hash.** The alternative considered was pruning a fixed junk list
(`.DS_Store` and friends) so that two visually equal trees attest equal.
Rejected on two grounds. It is not expressible in the existing exclusion
vocabulary, which matches directory names rather than file names, so it would
introduce a second exclusion mechanism whose only purpose is to hide files from
a hash. And it is a lie of the useful kind only until it is not: the file is in
the claimed directory, so the honest answer to "what does this unit cover" is
that it covers the file. `CLAUDE.md` warns against exactly this machine
dependence for `[index] extra_hashed_inputs`, and the warning does not transfer,
because that list feeds committed shard hashes while an attestation is on demand
and never committed.

**D-3 (2026-09-10): the walk is not extension-filtered.** Reusing
`coverage.rs::SOURCE_EXTS` was the first design and is wrong for this payload.
Of the fourteen specs the defect reaches, the claimed subtrees are dominated by
markdown, JSON, TOML and YAML: `.claude/agents/` is markdown only. An
extension-filtered walk would hash zero files there, emit a hash anyway, and
report `resolution.ok: true`, which is a vacuous pass in the exact shape this
corpus has been burned by before. The coverage universe filters by extension
because it is answering which **source files** a spec claims; an attestation is
answering what a unit covers, and those are different questions about the same
directory.

**D-4 (2026-09-10): a symlink is hashed as its target text, not followed.**
Three options were considered. Following links is what the shared walker does
today via `path.is_dir()`, and it is the one option that can hang: a cycle
inside a claimed subtree recurses without bound, and a link out of the tree
hashes content this repository does not own into a record asserting what it
does. Skipping links entirely is safe but contradicts 3.4, since adding or
retargeting a link inside claimed territory would leave the hash unmoved and the
change invisible. Hashing the link's own path and target text keeps both
properties and matches how git stores a symlink, as a blob whose content is the
target path, so the attestation records the same fact the repository does.

This is stated here because the walk is new surface. The pre-existing
`walk_source` behaviour is out of scope (4); this spec constrains the walk it
introduces for hashing, where a divergent or out-of-tree read is a correctness
and trust question rather than a resolution one.

## Verification

Each line below is one command: spec 049 3.2 makes a fence's body line a
command, so no line may depend on a variable another line set. The scratch
corpus is materialized at a fixed path for that reason, and the multi-step
comparisons run inside a single `sh -c`.

Every assertion for 3.1 to 3.4 fails against pre-083 code, most of them because
`attest --spec` exits 3 and prints no payload. The two comparison assertions are
written to require a non-empty reading first, because `test "$A" = "$B"` between
two empty strings is a pass that proves nothing and against pre-083 code both
readings are empty.

The 3.5 lines split, and the difference is worth stating rather than leaving to
be inferred. `cargo test --test attest` fails against pre-083 code once the
guards 3.5 requires are written, because failing there is their job. The last
two lines pass before and after: they assert that a spec of regular files still
attests and that the schema constant has not moved, which is what a regression
guard is for and why neither line may be read as evidence that the defect is
fixed.

The scratch corpus claims one subtree in each spelling, `"d1/"` as a
trailing-slash `file` unit and `d2/` as an explicit `directory` unit, so the
3.3 assertion also proves 3.1's rule that the two are treated identically. It
carries its own `spec-spine.toml` naming `resolver_exclusions`, so the 3.2
assertion tests the stated rule rather than whatever the binary's compiled-in
default happens to be, and it still asserts something if that default changes.

Spec 049 3.2 forbids a line depending on a variable another line set, and the
lines here share a filesystem rather than a shell. That is the point of the
fixed path, but it is only safe while each mutation is undone by the line that
made it: the 3.2 assertion removes its own `target/` directory before comparing,
so the 3.4 baseline reads the same tree 3.3 did. A line that left state behind
would make its successor pass or fail for a reason its own text does not
describe.

```verify:cli
# Self-contained: the assertions below drive the release binary.
cargo build --release --locked
# 3.5 the regression guards, which fail against pre-083 code.
cargo test -p spec-spine-core --test attest --locked
# 3.1 the spec the defect was found on attests, at exit 0 rather than exit 3.
target/release/spec-spine attest --spec 048-kit-ships-the-governed-loop-skills >/dev/null
# 3.1 its payload carries at least one real hash, not an error envelope.
sh -c 'test $(target/release/spec-spine attest --spec 048-kit-ships-the-governed-loop-skills --json | grep -c "\"contentHash\": \"") -ge 1'
# 3.1 and every spec in the corpus attests, which was false for 14 of 83.
sh -c 'for id in $(target/release/spec-spine registry list --ids-only); do target/release/spec-spine attest --spec "$id" >/dev/null || { echo "unattestable: $id" >&2; exit 1; }; done'
# A scratch corpus whose one spec claims two empty directories.
rm -rf "${TMPDIR:-/tmp}/ss083" && mkdir -p "${TMPDIR:-/tmp}/ss083/specs/001-dirs" "${TMPDIR:-/tmp}/ss083/d1" "${TMPDIR:-/tmp}/ss083/d2"
# The corpus states its own exclusion policy rather than inheriting the binary's default.
printf -- '[index]\nresolver_exclusions = ["target"]\n' > "${TMPDIR:-/tmp}/ss083/spec-spine.toml"
printf -- '---\nid: "001-dirs"\ntitle: "t"\nstatus: draft\ncreated: "2026-09-10"\nsummary: "s"\nestablishes:\n  - "d1/"\n  - { kind: directory, path: "d2/" }\n---\n\n# t\n' > "${TMPDIR:-/tmp}/ss083/specs/001-dirs/spec.md"
# 3.3 the two spellings both hash, distinctly, never both as SHA-256 of nothing.
sh -c 'test $(target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss083" attest --spec 001-dirs --json | grep "\"contentHash\"" | sort -u | wc -l) -eq 2'
# 3.2 a file under an excluded directory name does not enter the hash.
sh -c 'A=$(target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss083" attest --spec 001-dirs --json | grep "\"contentHash\""); test -n "$A" || exit 1; mkdir -p "${TMPDIR:-/tmp}/ss083/d1/target"; echo junk > "${TMPDIR:-/tmp}/ss083/d1/target/j.txt"; B=$(target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss083" attest --spec 001-dirs --json | grep "\"contentHash\""); rm -rf "${TMPDIR:-/tmp}/ss083/d1/target"; test "$A" = "$B"'
# 3.4 a real file beneath a claimed directory does enter the hash.
sh -c 'A=$(target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss083" attest --spec 001-dirs --json | grep "\"contentHash\""); test -n "$A" || exit 1; echo hello > "${TMPDIR:-/tmp}/ss083/d1/f.txt"; B=$(target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss083" attest --spec 001-dirs --json | grep "\"contentHash\""); test "$A" != "$B"'
rm -rf "${TMPDIR:-/tmp}/ss083"
# 3.5 a file-only spec still attests, and the schema constant is unmoved.
target/release/spec-spine attest --spec 070-a-malformed-id-is-refused-not-a-panic >/dev/null
target/release/spec-spine attest --spec 070-a-malformed-id-is-refused-not-a-panic --json | grep -q '"schemaVersion": "0.1.0"'
```
