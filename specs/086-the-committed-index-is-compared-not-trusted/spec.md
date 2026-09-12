---
id: "086-the-committed-index-is-compared-not-trusted"
title: "The committed index is compared, not trusted"
status: draft
kind: "tooling"
created: "2026-09-11"
implementation: in-progress
owner: "The spec-spine Authors"
risk: high
depends_on:
  - "004-codebase-index"
  - "005-coupling-gate"
  - "024-index-sharding"
  - "031-registry-freshness-check"
  - "075-one-name-one-freshness-verb"
amends:
  # 024 5 accepts that a resolution flip caused purely by a sibling change is
  # not caught by `index check` and is healed by a later `index` run. Comparing
  # bytes catches it. 024's text is not edited (spec 040). See 5, D-1.
  - "024-index-sharding"
establishes:
  # 3.5: the tamper matrix against the committed index, library side.
  - { kind: file, path: "crates/spec-spine-core/tests/index_body.rs" }
extends:
  # 3.1 and 3.2: `check_index_freshness` compares the committed shard bytes
  # with the shards a fresh index emits, and reports the drift classes.
  - { spec: "004-codebase-index", unit: "crates/spec-spine-core/src/index.rs", nature: additive }
  # 3.2: `index check` prints the three drift classes by shard.
  - { spec: "004-codebase-index", unit: "crates/spec-spine-cli/src/cmd_index.rs", nature: additive }
  # 3.2: `check` reports the index half with the same classes.
  - { spec: "075-one-name-one-freshness-verb", unit: "crates/spec-spine-cli/src/cmd_check.rs", nature: additive }
references:
  - { unit: { kind: file, path: "docs/authority-evidence.md" }, role: context }
  - { unit: { kind: file, path: "docs/design/04-authority-evidence-extension.md" }, role: context }
  - { unit: { kind: file, path: "specs/031-registry-freshness-check/spec.md" }, role: context }
summary: >
  `index check` does not compare the committed index with what the corpus
  indexes to. It reads each committed shard's own mapping, derives from that
  mapping which span files to hash, and compares the resulting hash with the
  shard's `shardHash` field. The mapping body itself (which units resolved
  where, which paths implement which spec, the ownership flags) is never
  compared with a fresh resolution, and for a `file`, `directory` or `crate`
  unit, which carries no span, nothing in the hash constrains it. Measured on a
  scratch repository: rewriting a committed shard so its spec owns nothing,
  with `shardHash` left alone, leaves `index check` and `check` reporting fresh,
  and `couple` then derives ownership from the edited body and passes a change
  to code the spec owned, without its `spec.md`. `.derived/` is in the bypass
  floor, so the edit itself trips nothing. AGENTS.md tells every session that a
  fresh `check` means the committed shards are exactly what the corpus compiles
  to; for the index that is not what is checked. Spec 031 3.1 recorded the
  weaker comparison as a cost trade; a full re-index of this repository takes
  0.04 s against 0.03 s for the check. This spec makes `index check` compare
  bytes and set membership exactly as `compile --check` does, so every reader
  of the committed index (the coupling gate, `index owner`, coverage, the
  projections) reads a body equal to the recompute. It amends 024 5, whose
  bounded trade let a sibling-caused resolution flip go unreported.
---

# 086: The committed index is compared, not trusted

## 1. Purpose

Two committed trees back every read verb and the coupling gate: the registry
and the index. Spec 031 made the registry's freshness check byte-exact:
`compile --check` re-emits every shard and compares bytes, which "catches a
stale hash, a hand-edited shard body, and a schema-version restamp alike". The
same section records that the index side is weaker, "because its inputs (code
spans) are too expensive to re-resolve".

What the weaker check actually does is in `index.rs::check_index_freshness`.
For each committed spec shard it reads the committed mapping, asks that mapping
which files back its spans (`span_files_for_mapping(&sh.mapping)`), hashes the
spec's `spec.md`, those files and the global-inputs scalar, and compares the
result with the shard's `shardHash`. The body is trusted to name its own inputs
and is never compared with anything. For a unit with no span (every `file`,
`directory` and `crate` unit, which is 72 of this repository's claims and the
common case in an adopter corpus) the body can say anything at all.

Measured on 2026-09-11 in a scratch repository whose spec `001-a` establishes
`src/a.rs`:

```
$ spec-spine check                          # committed trees as generated
spec-registry: fresh
codebase-index: fresh
# rewrite .derived/codebase-index/by-spec/001-a.json so 001-a owns nothing;
# leave "shardHash" as it was
$ spec-spine index check ; echo $?
index is fresh
0
# commit that, plus an edit to src/a.rs, without touching specs/001-a/spec.md
$ spec-spine couple --base main --head HEAD --json
{ "exitCode": 0, "report": { "checkedPaths": 1, "violations": [] }, ... }
$ spec-spine index owner src/a.rs
src/a.rs
  (no spec owns this path)
```

The gate derived ownership from a body nobody checked. Continuous integration
does not catch it either: the self-governance job runs `check`, and the
determinism job regenerates the trees and compares the four platforms with each
other, never with the committed copy.

The cost argument does not survive measurement here: `spec-spine index` over
this repository's 85 specs and 4 packages takes 0.04 s, against 0.03 s for
`index check`. A corpus heavy in `symbol` units pays for tree-sitter
resolution, which is the case spec 031 had in mind; 3.4 keeps that cost bounded.

## 2. Territory

- `crates/spec-spine-core/src/index.rs`: `check_index_freshness` re-emits the
  shard set in memory and compares it with the committed files.
- `crates/spec-spine-cli/src/cmd_index.rs` and `cmd_check.rs`: the report names
  each drifted shard with its class.
- `crates/spec-spine-core/tests/index_body.rs`: the guards.

No committed shard changes, no emitted byte changes, and no schema moves.

## 3. Behavior

### 3.1 `index check` compares bytes and sets

`index check` MUST index the corpus in memory, without writing, and compare the
result with the committed index tree:

- **modified**: a committed shard file whose bytes differ from the shard the
  in-memory index emits. This covers a stale `shardHash`, a hand-edited body,
  and a schema restamp alike.
- **missing**: a spec or package with no committed shard.
- **orphaned**: a committed shard with no spec or package behind it, including
  a stray file in either shard directory.

The `--slice` form keeps its current sidecar comparison. The existing
blocking-diagnostic refusal stays. The exit code is unchanged: 2 when anything
drifted.

### 3.2 Every reader inherits it

`check`, and the freshness guard in front of `couple` and `index coverage`,
call the same comparison, so after this spec a committed index that reads fresh
is byte-identical to the recompute. The reports name each drifted shard with
its class, as `compile --check` does for the registry.

### 3.3 What the gate's ownership now means

`couple` loads owners from the committed index only after 3.1 has passed. The
owner set a C-001 decision uses is therefore the owner set the corpus resolves
to at that tree, not whatever the committed body says.

This does not settle who may change ownership. A candidate can still claim a
unit through a new `extends` edge in its own `spec.md` and regenerate the index
honestly; that is a legitimate route (spec 047) and a structural change a
reviewer must see. Draft 088 classifies it. This spec only ensures the body the
gate reads is the one the corpus produces.

### 3.4 Cost

The in-memory index is the one `spec-spine index` builds, so the check costs
one index run and one comparison, never a write. A corpus for which that is too
slow keeps the `--slice` form for scoped checks. The measurement in 1 is
recorded so a later reviewer can repeat it rather than restate the premise.

### 3.5 What must keep working

- A tree regenerated by `spec-spine index` reads fresh, and nothing the indexer
  emits changes.
- `check --fail-on-unresolved --fail-on-warn` keeps its current meaning on a
  fresh tree.
- The four-triple determinism gate is unaffected: it compares emitted trees,
  which this spec does not change.

### 3.6 The trade in 024 5 changes

024 5 accepts that a resolution flip caused purely by a sibling change (a unit
that begins resolving because another PR added the file) is not caught by
`index check` and is healed by a later `index` run. A byte comparison catches
it: after both PRs merge, the committed shard differs from the recompute, and
`check` reports it until someone runs `index`. With the merge queue (spec 020)
the speculative build of the second PR sees the flip before it merges, and the
remedy is the one 024 already names. FR-005's guarantee, no textual conflict
between disjoint changes, is untouched: this spec changes what is reported, not
what is written.

## 4. Out of scope

**Deciding who may claim a unit.** 3.3; draft 088.

**Signing or hashing the committed body into a new field.** A digest of the
body stored beside the body can be rewritten with it. Only a comparison with a
fresh resolution establishes anything.

**The registry side.** Spec 031 already compares bytes.

## 5. Resolved decisions

**D-1 (2026-09-11): `amends` 024.** 024 5 is a deliberate statement of what
index staleness does not catch, and this spec changes it. The amendment is
declared here and 024's text is untouched (spec 040). 031 3.1's sentence about
the index side is descriptive of 024's behavior, not a requirement of 031, so no
edge to 031 is needed beyond the reference.

**D-2 (2026-09-11): fix the check, not only the gate.** The alternative was to
make `couple` resolve ownership from a fresh in-memory index and leave `index
check` as it is. That repairs one reader and leaves `index owner`, coverage and
every consumer of the committed tree reading an unchecked body, while `check`
continues to report it fresh. The committed tree is the ledger; the check is
what makes it one.

**D-3 (2026-09-11): no opt-in flag.** A flag would leave the weak comparison as
the default, and every gate chain already written against `check` would keep
it.

**D-4 (2026-09-12): an unbuilt index stays an I/O error, not staleness.** 3.1
mirrors `compile --check`, and 031 3.2 makes an unbuilt *registry* stale rather
than an error, on the reasoning that a tree never built is by definition not
vouching for the corpus. Read straight across, that would make an unbuilt index
stale here too. It is not adopted. `check_index_freshness` is the freshness
guard in front of `couple`, `index coverage` and `index owner`, so the change
would move all three from exit 3 to exit 2 and replace "run `spec-spine index`
first" with "stale", none of which this spec is for. 3.1 describes the
comparison, not the read boundary in front of it, and the registry's answer
belongs to the verb that reads a registry.

**D-5 (2026-09-12): a shard from an unknown schema MAJOR is refused, not called
`modified`.** The comparison classifies without parsing, which is what lets a
stray file be `orphaned` rather than a parse error. Applied without exception it
would also demote the read boundary's MAJOR gate, which
`read_committed_index_shards` has always applied: a committed tree at a MAJOR
this build does not understand would read as `modified`, and the remedy a stale
verdict advises, running `spec-spine index`, would overwrite it with a
downgrade. So a file that already differs is probed for its `schemaVersion`
alone, and only to re-raise `Error::Schema`; a body that will not parse stays
`modified`. No classification changes, and the registry side is left as it
stands: this keeps a refusal that already existed rather than adding one.

## Verification

Each line runs in its own `sh -c` from the repository root (spec 049 3.5). The
scratch repository lives at a fixed path; each tampering line restores the
shard before its final assertion.

Against pre-086 code, the three tamper lines fail (the check and the gate both
report success on a rewritten body) and are the evidence. The regenerate line,
this repository's own `check`, and the setup lines pass before and after. The
tamper rewrites a path inside the body, which cannot move `shardHash` for a
`file` unit, so the line cannot pass by accident of a hash change.

```verify:cli
cargo build --release --locked
rm -rf "${TMPDIR:-/tmp}/ss086" && mkdir -p "${TMPDIR:-/tmp}/ss086/specs/001-a" "${TMPDIR:-/tmp}/ss086/src"
printf -- '---\nid: "001-a"\ntitle: "a"\nstatus: approved\ncreated: "2026-09-11"\nimplementation: complete\nsummary: "s"\nestablishes:\n  - "src/a.rs"\n---\n\n# a\n' > "${TMPDIR:-/tmp}/ss086/specs/001-a/spec.md"
printf 'pub fn a() {}\n' > "${TMPDIR:-/tmp}/ss086/src/a.rs"
D="${TMPDIR:-/tmp}/ss086"; S=target/release/spec-spine; $S --repo "$D" compile >/dev/null && $S --repo "$D" index >/dev/null && $S --repo "$D" check
D="${TMPDIR:-/tmp}/ss086"; S=target/release/spec-spine; F="$D/.derived/codebase-index/by-spec/001-a.json"; cp "$F" "$D/bak"; sed 's#"src/a.rs"#"src/zz.rs"#g' "$D/bak" > "$F"; $S --repo "$D" index check > "$D/out" 2>&1; R=$?; cp "$D/bak" "$F"; test $R -eq 2 && grep -q "001-a" "$D/out"
D="${TMPDIR:-/tmp}/ss086"; S=target/release/spec-spine; F="$D/.derived/codebase-index/by-spec/001-a.json"; cp "$F" "$D/bak"; sed 's#"src/a.rs"#"src/zz.rs"#g' "$D/bak" > "$F"; $S --repo "$D" check >/dev/null 2>&1; R=$?; cp "$D/bak" "$F"; test $R -eq 2
D="${TMPDIR:-/tmp}/ss086"; S="$PWD/target/release/spec-spine"; cd "$D" && git init -q -b main && git -c user.email=t@example.invalid -c user.name=t add -A && git -c user.email=t@example.invalid -c user.name=t commit -qm base && sed -i.orig 's#"src/a.rs"#"src/zz.rs"#g' .derived/codebase-index/by-spec/001-a.json && rm .derived/codebase-index/by-spec/001-a.json.orig && printf 'pub fn a() { edited(); }\n' > src/a.rs && git -c user.email=t@example.invalid -c user.name=t commit -qam tamper && "$S" couple --base main~1 --head HEAD >/dev/null 2>&1; R=$?; git reset -q --hard main~1; test $R -eq 2
D="${TMPDIR:-/tmp}/ss086"; S=target/release/spec-spine; $S --repo "$D" index >/dev/null && $S --repo "$D" index check
target/release/spec-spine check
rm -rf "${TMPDIR:-/tmp}/ss086"
```
