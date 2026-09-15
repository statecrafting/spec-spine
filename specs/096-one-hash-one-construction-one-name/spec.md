---
id: "096-one-hash-one-construction-one-name"
title: "One hash, one construction, one name"
status: draft
kind: "tooling"
created: "2026-09-14"
summary: >
  `registry show` prints `contentHash: <h>  (sha256 of this spec.md)`, and
  approved spec 055 §3.4 says the same thing in prose: "`contentHash` is
  SHA-256 over that spec's `spec.md` alone". It is neither. The value is the
  shard hash, which is SHA-256 over the repo-relative POSIX path, a NUL, and
  the normalized bytes, so for `specs/086-.../spec.md` the printed value is
  `a4a0098235e9...` while the digest of the file's bytes is `ae0144ca7e4a...`,
  the value `attest --spec` reports as `specSourceHash`. 055 §3.3 exists for
  consumers who reimplement the normalization to pin against the ledger, and
  one adopter does exactly that, so a gloss naming the wrong construction is
  the whole defect. This spec amends 055 §3.4 to state the construction the
  code has always used, makes the printed line say it, and adds the test that
  pins both digests so the two cannot drift again.
implementation: in-progress
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "042-per-spec-attestation"
  - "055-the-ledger-answers-what-consumers-rebuild"
amends:
  - "055-the-ledger-answers-what-consumers-rebuild"   # §3.4 states a construction the code never had
extends:
  # 3.2: the printed line, where the wrong gloss lives.
  - { spec: "055-the-ledger-answers-what-consumers-rebuild", unit: "crates/spec-spine-cli/src/cmd_registry.rs", nature: corrective }
  # 3.4: the pin over both constructions, asserted through the CLI.
  - { spec: "055-the-ledger-answers-what-consumers-rebuild", unit: "crates/spec-spine-cli/tests/cli.rs", nature: additive }
references:
  - { unit: { kind: file, path: "docs/design/05-remaining-waves-2026-09.md" }, role: context }
  - { unit: { kind: file, path: "docs/authority-evidence.md" }, role: context }
---

# 096: One hash, one construction, one name

## 1. Purpose

### 1.1 The measurement

For `specs/086-the-committed-index-is-compared-not-trusted/spec.md`, on
2026-09-14 at `0.19.0`:

```
registry show 086 --json  .contentHash    a4a0098235e9...
attest --spec 086         .specSourceHash ae0144ca7e4a...
sha256(normalized bytes)                  ae0144ca7e4a...
sha256(path + NUL + normalized bytes)     a4a0098235e9...
```

`contentHash` is the second construction. `cmd_registry.rs` prints it with the
gloss `(sha256 of this spec.md)`, which names the first.

### 1.2 The approved prose says it too

This is not only a label. Spec 055 §3.4, which is approved, reads:

> `contentHash` is SHA-256 over that spec's `spec.md` alone: the registry's
> only hashed input, unchanged since before sharding.

The sentence's purpose was to distinguish the registry value from the index's
per-spec shard hash, which folds span-backing source files and the global
scalar, and it does that correctly. What it gets wrong is the construction: the
registry's shard hash frames the path, as every content hash in this repository
does (`<repo-relative POSIX path>\0<normalized bytes>`, sorted by path), which
is the property that makes a one-file digest and a whole-tree fold the same
algorithm.

### 1.3 Who the gloss is for, and what it costs them

055 §3.3's stated reason for adding `contentHash` was that a consumer was
reimplementing `hash.rs`'s normalization in TypeScript in order to pin against
the ledger. That consumer reads the gloss, hashes the file's bytes, gets a
different 64 hex characters, and concludes either that the tool is wrong or that
its own normalization is. Both conclusions are wrong and both cost a session.

`docs/authority-evidence.md` §4 already records the discrepancy as measured
fact, and design note 04 carries it as F9, "small fix, unfiled". It is small.
It is also an approved spec stating a behavior the code never had, which makes
it an amendment rather than a patch, and spec 040 says where an amendment is
authored: here, in the amending spec, without editing 055.

## 2. Territory

This spec changes the printed line in `cmd_registry.rs`, adds the pin to
`crates/spec-spine-cli/tests/cli.rs`, and states the replacement text for 055
§3.4. It changes no emitted value, no DTO, no committed artifact and no schema
version: every digest keeps the construction it has, which is the point.

## 3. Behavior

### 3.1 The construction, stated once and correctly

`contentHash`, as `registry show` reports it, is the value the spec's committed
registry shard records as `shardHash`, which is:

```
SHA-256( <repo-relative POSIX path> 0x00 <normalized bytes of spec.md> )
```

where normalization strips a BOM and converts CRLF and CR to LF. It is read
from the committed shard and never recomputed (055 §3.2, unchanged).

This replaces spec 055 §3.4's first sentence. Under spec 040 the replacement
text lives here, and 055 is not edited:

> `contentHash` is the registry shard's own content hash over that spec's
> `spec.md`: SHA-256 over the file's repo-relative POSIX path, a NUL, and the
> file's normalized bytes. It covers that one file and nothing else, which is
> what distinguishes it from the index's per-spec shard hash, and it is framed
> by the path like every other content hash this tool computes, so it is not
> equal to the digest of the file's bytes alone.

The rest of 055 §3.4 stands: a consumer asking "has this spec's text changed"
still wants the registry value, and a consumer asking "has anything this spec
depends on changed" still wants the index value.

### 3.2 The printed line names the construction it prints

The prose form of `registry show` MUST NOT describe the value as the digest of
the file's bytes. It MUST name the framing, briefly, in the same breath as
reporting the value, which is what 055 §3.4 already requires of this line for a
different distinction. A consumer that wants the exact algorithm reads this
spec or `docs/api.md`; the line's job is to stop a reader reproducing the wrong
thing.

### 3.3 The unframed digest has a name already, and keeps it

`attest --spec` reports `specSourceHash`: SHA-256 over the same normalized
bytes, unframed. That is the value a consumer reproduces with an ordinary
digest of the file, and it is the one to document as such.

`registry show` MUST NOT gain it. 055 §3.2 forbids exactly this: `registry` is
the read-side view of what was committed, and a `show` that computed a digest
from `spec.md` would report a value the ledger does not hold, which silently
repairs the staleness `compile --check` exists to reveal. Two names, two
constructions, two verbs, and each verb reports only what its own source of
truth holds.

### 3.4 The pin

`crates/spec-spine-cli/tests/cli.rs` MUST assert, for one spec in the corpus,
all four of:

1. `registry show --json`'s `contentHash` equals SHA-256 of the framed input;
2. it does **not** equal SHA-256 of the normalized bytes alone;
3. `attest --spec`'s `specSourceHash` equals SHA-256 of the normalized bytes
   alone;
4. the **prose** form of `registry show` does not describe its `contentHash`
   as the digest of the file's bytes, and does name the framing, per §3.2.

Asserting the inequality in (2) is the part that separates the two
constructions: a test that only checked (1) would pass under either gloss.

Assertion (4) is the part that catches the actual defect, and it was missing
from the first draft of this section. Every value in this spec is already
correct and stays correct; what was wrong was a **sentence**, in two places.
A suite that pins only digests would have passed unchanged on the day the
wrong gloss was written and would pass unchanged on the day someone writes it
again. The assertion must therefore read the emitted line, and it must be
written so that it fails on the pre-096 wording specifically: the shipped line
ends `(sha256 of this spec.md)`, which is the phrase that says the wrong thing,
so the test asserts that phrase is absent and that the line names the path
framing. A test asserting only that some explanation is present would pass on
the wrong explanation.

### 3.5 Documentation

`website/docs/cli/registry.md`, the adopter-facing reference for
`registry show`, MUST state the construction beside the field. `docs/api.md`
MUST name `specSourceHash` as the unframed digest and the verb that reports it,
so the two names are distinguishable from the document a binding author reads.
`docs/authority-evidence.md` §4 already records the discrepancy as measured
fact and MUST be updated to record that it is closed, with a pointer to this
spec, since it is the document consumers in this family read for digest
provenance.

Neither documentation file is in `[index] extra_hashed_inputs`, so this spec's
documentation duty restales no shard.

## 4. Out of scope

**Renaming `contentHash`.** It is spec 055's field name and an adopter's pin.
A rename to `shardHash` would be more accurate and is a breaking output change
for the one consumer the field exists for.

**Changing any digest construction.** Note 04 §4.1 states the standing rule:
existing payloads keep their constructions forever so historical evidence stays
interpretable. New record types use the framed `frame/1` construction spec 087
defines. Nothing here touches either.

**Adding `specSourceHash` to `registry show`** (§3.3).

**The other unversioned read documents.** Spec 093 covers those; this spec is
about one value's meaning, not about the envelope it arrives in.

## 5. Resolved decisions

**D-1 (2026-09-14). The amendment corrects the prose, not the code.** The
alternative was to make the code match the approved sentence by emitting the
unframed digest as `contentHash`. That was rejected: it would change a value
adopters have pinned since 0.14.0, it would break the identity between this
field and the shard's own `shardHash`, and it would mean a read that recomputes
(§3.3). The code is right and the sentence is wrong, so the sentence is what
this spec changes.

**D-2 (2026-09-14). One amendment, no `amends_sections` narrowing.** The
`amends` edge names 055 as a whole and §3.1 names the section and quotes the
replacement text. The corpus has no spec that narrows an `amends` edge to an
anchor, and inventing the convention here would put the amendment's scope in
two places.

**D-3 (2026-09-15). The acceptance reads the sentence, not only the digests.**
§3.4's fourth assertion. This spec changes no value, so a suite of digest
comparisons is a suite that cannot fail on the defect: it would have passed on
the day the wrong gloss was written. The assertion names the pre-096 phrase it
must not find, rather than checking that some explanation is present, because
"an explanation is present" is true of the wrong explanation.

## Verification

Each line is one command. The first `grep` fails against pre-096 code, because
the wrong gloss is still in the source; it is the fail-first evidence. The
digest comparison lines pass before and after, because no value changes: they
are the pin, and the one asserting inequality is what makes the pair of
constructions explicit. The two lines reading the **prose** form also fail
against pre-096 code, for the same reason as the first `grep` and at the
surface a consumer actually reads. The `cargo test` line is **not** fail-first:
the assertions in §3.4 do not exist at the parent commit.

```verify:cli
# 3.2: the wrong gloss is gone.
! grep -qF 'sha256 of this spec.md' crates/spec-spine-cli/src/cmd_registry.rs
# 3.1: the printed value is the framed construction, not the bare digest.
target/release/spec-spine registry show 096 --json | python3 -c 'import json,sys,hashlib; p="specs/096-one-hash-one-construction-one-name/spec.md"; b=open(p,"rb").read().replace(b"\r\n",b"\n").replace(b"\r",b"\n").lstrip(b"\xef\xbb\xbf"); d=json.load(sys.stdin)["contentHash"]; assert d==hashlib.sha256(p.encode()+b"\x00"+b).hexdigest(), d; assert d!=hashlib.sha256(b).hexdigest()'
# 3.5: the construction is documented where the field is, for both names.
grep -qF 'specSourceHash' docs/api.md
grep -qE 'NUL|0x00|path-framed' website/docs/cli/registry.md
# 3.2: the prose form names the framing rather than the file's bytes.
target/release/spec-spine registry show 096 | grep -i contentHash | grep -qvF 'sha256 of this spec.md'
target/release/spec-spine registry show 096 | grep -i contentHash | grep -qE 'path|framed|NUL'
# 3.4: the four-way pin, through the CLI.
cargo test -p spec-spine-cli --test cli --locked
```
