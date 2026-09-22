---
id: "118-the-renumber-is-history-not-a-standing-rule"
title: "The renumber is history, not a standing rule"
status: approved
kind: "governance"
created: "2026-09-22"
summary: >
  Spec 095's acceptance asserts that the live corpus's ordinals are contiguous,
  `o == list(range(len(o)))`. Its requirement (3.3) is narrower: the one-time
  renumber left the survivors contiguous. Its own D-2 says reserved gaps are
  deliberate. Merging spec 117 while 105 to 116 stay reserved made the block red
  with no defect in the corpus. The repository owner ruled that the renumber is
  a historical transformation and not a perpetual contiguity rule
  (docs/release-candidate-0.22.0.md 10.1). This spec carries 095's block in full
  through `amends_verification`. The historical contiguity is read from the map
  that records it, and the live corpus is held to identity rules that survive
  growth, with fixture negatives showing each can still refuse. 095's
  requirements and block are not edited; its file gains only the superseded-
  acceptance note spec 082 3.4 requires.
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "082-an-amended-acceptance-is-the-one-that-runs"
  - "095-the-corpus-describes-what-exists"
amends: ["095-the-corpus-describes-what-exists"]
# 3.1: this spec's `## Verification` block IS 095's acceptance from now on.
# 095's requirements are unchanged; only what it accepts is corrected.
amends_verification: ["095-the-corpus-describes-what-exists"]
references:
  - { unit: { kind: file, path: "docs/corpus-map.md" }, role: context }
  - { unit: { kind: file, path: "docs/release-candidate-0.22.0.md" }, role: context }
---

# 118: The renumber is history, not a standing rule

## 1. Purpose

### 1.1 A red block with no defect behind it

`./scripts/verify-sweep.sh --rev origin/main` at `835dd2e4`, and the nightly
`Acceptance` run at `3d4f3902` (run `35723398403`), both report:

```
FAILED  095-the-corpus-describes-what-exists  FAILED at command 9 (exit 1)
```

Command 9 is:

```python
o=[int(i[:3]) for i in ids]; assert o==sorted(o); assert len(set(o))==len(o); assert o==list(range(len(o)))
```

The last conjunct requires every ordinal from `000` to the highest to be
present, which makes it a standing contiguity rule. Merging spec 117 while 105
to 116 are reserved on an unmerged branch makes it false. At `2d3673db`, the
revision before 117 merged, the same predicate held over 105 ids.

Spec 095 does not require that. §3.3 describes the rewrite that made the
survivors contiguous **once**. D-2 says in terms that "gaps are not a defect on
their own; spec 095's and 117's reserved gaps were deliberate", and it gives
the collapse, not contiguity in general, as the reason for the rewrite. The
block asserts more than the spec says, which is spec 082's case for
`amends_verification`.

### 1.2 The owner's ruling

Recorded in `docs/release-candidate-0.22.0.md` §10.1 before this spec was
built. The renumber requirement describes the historical transformation.
Legitimately reserved later ordinals are permitted. 117 keeps its identity,
105 to 116 are not merged, nothing is renumbered, and the replacement must keep
meaningful verification of everything else 095 accepts.

## 2. Territory

This spec establishes no code. It owns its own `spec.md`, and through
`amends_verification` it owns the fact that 095's `## Verification` block is no
longer the one `verify 095` runs.

It makes one edit to 095's file: the superseded-acceptance note above 095's
fence, which spec 082 3.4 requires of every amended block and
`every_superseded_verification_block_says_so_in_its_own_document` enforces.
The owner's ruling (§1.2) authorizes it. Nothing else in 095 changes: no
requirement, no decision and no command.

## 3. Behavior

### 3.1 The block is replaced in full

This spec's `## Verification` block MUST replace 095's in full (spec 082 3.5),
carrying every one of 095's commands except command 9. Commands 8 and 10,
which wrote and removed a temporary id list only for command 9, go with it.
095's requirements and commands MUST NOT be edited; the only permitted edit is
the note of §2. The block MUST assert that 095 still carries the superseded
predicate, so a later "fix" by editing 095's commands turns this block red.

### 3.2 The historical renumber is read from the record of it

095 §3.4 requires `docs/corpus-map.md` to list every survivor's new id (its §2)
and the three specs the collapse filed (its §3). Those ids are the renumber's
output. The block MUST assert that their ordinals are unique, contiguous from
`000`, and end at `095`, the last spec the collapse filed. It MUST also assert
that every one of them is still a spec in the live corpus. This is exactly the
claim §3.3 makes, and the map is history, so the assertion does not move as the
corpus grows.

The checker MUST be shown able to fail. The same command, run against a copy
of the map with one survivor row removed and against a copy with one row
duplicated, MUST exit non-zero.

### 3.3 The live corpus is held to rules that survive growth

Against the live corpus the block MUST assert that every id matches the id
grammar, that ordinals are strictly increasing in listing order (so none
repeats), and that the spec directories on disk and the registry's ids are the
same set. It MUST NOT assert contiguity, a maximum ordinal, or a count.

### 3.4 The engine still refuses what matters, shown on fixtures

Isolated fixture corpora under the block's own `TMPDIR` MUST show:

1. A corpus with a gap (`000`, `001`, `117`) compiles, exit 0, and lists all
   three ids. A reserved later ordinal is legitimate.
2. Two specs sharing an ordinal (`117-a`, `117-b`) are refused, exit 1, `V-004`.
3. An id outside the grammar (`17-short`) is refused, exit 1, `V-012`.
4. A directory that does not equal its id is refused, exit 1, `V-001`.

The fixtures borrow no local ref and read nothing outside their own directory.

## 4. Out of scope

- **Any change to 095's requirements.** They are unchanged, and so is the rule
  in 095 §3.7 that the renumber is not a precedent.
- **Merging 105 to 116, or renumbering 117.** §9.4 routes 2 and 3, declined by
  the owner.
- **The sweep's treatment of unbuilt drafts.** Spec 119.

## 5. Resolved decisions

D-1 (2026-09-22, the map, not a pinned count). A literal "96 specs from 000 to
095" would describe history correctly, but it states twice what the map
already records. It would also go stale silently if the map were ever
corrected. Contiguity-from-zero plus the `095` endpoint is the shape of the
fact, read from the document 095 made responsible for it.

D-2 (2026-09-22, strictly increasing covers uniqueness and order). `registry
list --ids-only` lists ids sorted. Strictly increasing ordinals in that order
therefore fail on a repeated ordinal as well as on an out-of-order listing, and
never on a gap.

D-3 (2026-09-22, `.agents` and `.codex` are asserted absent from the tracked
tree). 095's `! test -e .agents` and `! test -e .codex` read the working
directory, so they fail in a maintainer checkout that holds untracked local
agent directories. The maintainer's own checkout does, today. The rule they
check is that no removed surface is in the repository, and a sweep worktree
answered that correctly only because it contains no untracked files. The
replacement asserts `git ls-files` returns nothing under `kit`, `.agents` and
`.codex`, which checks the revision in every checkout. `! test -e kit` is kept
as well: no checkout has a reason to hold that directory.

D-4 (2026-09-22, this spec joins the files allowed to name removed ids). 095's
removed-id scan excludes the specs that account for the removal (092 to 096).
This block has to spell the six ids to carry that scan. Without an exclusion
the scan would find this spec's own file and refuse. That is the same reason
096 was added to the class, and the character class is extended by exactly
this ordinal.

## Verification

This block replaces spec 095's (§3.1). The first part is 095's, command by
command, except its command 9 and the two lines that existed only for it.

**Fail-first evidence**, measured on 2026-09-22 at `3d4f3902` (the parent of
this change): `verify 095` runs 095's own block and fails at command 9. With
this spec present, `verify 095` runs this block and passes (41 commands). The
two mutation lines of §3.2 were checked against weakened checkers: with the
contiguity conjunct removed the gap copy passes, so the gap line goes red; with
both the uniqueness and contiguity conjuncts removed the duplicate copy passes
too, so the duplicate line goes red.

```verify:cli
cargo build --release --locked
# --- 095's acceptance, carried ---
# 095 3.1, 3.2: none of the twenty-seven remains, by directory and by registry.
! test -e specs/006-init-scaffold
! test -e specs/029-claude-code-skill-kit
! test -e specs/046-kit-hooks-read-never-write
! test -e specs/064-the-kit-ships-the-composite-gate
! test -e specs/100-one-source-generates-the-agent-trees
! test -e specs/116-shepherd-reads-every-reviewer
# 095 3.3: every reference resolves. A dangling id is a compile error (V-004
# family), and an unresolved unit is what `check` refuses.
target/release/spec-spine check --fail-on-unresolved --fail-on-warn
target/release/spec-spine lint --fail-on-warn
# 095 3.3: no removed id is cited outside the specs that account for it. 118
# joins the class because this block spells the ids (D-4).
sh -c 'for id in 006-init-scaffold 029-claude-code-skill-kit 046-kit-hooks-read-never-write 064-the-kit-ships-the-composite-gate 100-one-source-generates-the-agent-trees 116-shepherd-reads-every-reviewer; do if grep -rlF "$id" specs crates .claude/skills .claude/agents AGENTS.md CLAUDE.md README.md 2>/dev/null | grep -qvE "^specs/(09[23456]|118)-"; then echo "still cited: $id"; exit 1; fi; done; exit 0'
# 095 3.2 and 3.4: every removed spec is named by a successor and is in the map.
test "$(grep -cE '^\| `[0-9]{3}-[a-z0-9-]+` \|' docs/corpus-map.md)" -ge 27
sh -c 'n=0; for f in specs/*/spec.md; do n=$((n + $(grep -cE "^- \`[0-9]{3}-[a-z0-9-]+\`$" "$f"))); done; test "$n" -eq 27 || { echo "predecessors named: $n, want 27"; exit 1; }'
# 095 3.5: the sweep's ledger is live, closed, and every entry names a real spec.
grep -qE '^readonly LEDGER_CLOSED_AT=[0-9]+$' scripts/verify-sweep.sh
bash -n scripts/verify-sweep.sh
# 095 3.5: nothing in the repository still carries the removed product (D-3).
! test -e kit
test -z "$(git ls-files -- kit .agents .codex)"
# And the whole loop, over the corpus.
target/release/spec-spine index coverage --fail-on-untraced
make gate SPEC_SPINE=target/release/spec-spine COUPLE=0
cargo test --workspace --locked
# --- 118 3.1: the superseded form is still in 095's own file ---
grep -qF 'assert o==list(range(len(o)))' specs/095-the-corpus-describes-what-exists/spec.md
# --- 118 3.2: the historical renumber, read from the map ---
python3 -c 'import re,sys; t=open(sys.argv[1]).read(); s2=t.split("\n## 2.")[1].split("\n## 3.")[0]; s3=t.split("\n## 3.")[1].split("\n## 4.")[0]; new=re.findall(r"^\| `[0-9]{3}-[a-z0-9-]+` \| `([0-9]{3}-[a-z0-9-]+)` \|$", s2, re.M)+re.findall(r"^- `([0-9]{3}-[a-z0-9-]+)`$", s3, re.M); o=sorted(int(i[:3]) for i in new); assert len(set(o))==len(o), "a renumbered ordinal repeats"; assert o==list(range(len(o))), "the renumber left a gap"; assert o[-1]==95, o[-1]; print(len(o), "ids after the renumber, contiguous 000 to 095")' docs/corpus-map.md
python3 -c 'import re,subprocess; t=open("docs/corpus-map.md").read(); s2=t.split("\n## 2.")[1].split("\n## 3.")[0]; s3=t.split("\n## 3.")[1].split("\n## 4.")[0]; new=set(re.findall(r"^\| `[0-9]{3}-[a-z0-9-]+` \| `([0-9]{3}-[a-z0-9-]+)` \|$", s2, re.M)+re.findall(r"^- `([0-9]{3}-[a-z0-9-]+)`$", s3, re.M)); ids=set(subprocess.check_output(["target/release/spec-spine","registry","list","--ids-only"], text=True).split()); gone=sorted(new-ids); assert new, "no renumbered ids parsed from docs/corpus-map.md"; assert not gone, ("renumbered ids missing from the corpus", gone)'
# The checker can fail: one survivor row removed leaves a gap...
sh -c 'mkdir -p "${TMPDIR:-/tmp}/ss118" && grep -vF "| \`050-" docs/corpus-map.md > "${TMPDIR:-/tmp}/ss118/map-gap.md" && ! cmp -s docs/corpus-map.md "${TMPDIR:-/tmp}/ss118/map-gap.md"'
! python3 -c 'import re,sys; t=open(sys.argv[1]).read(); s2=t.split("\n## 2.")[1].split("\n## 3.")[0]; s3=t.split("\n## 3.")[1].split("\n## 4.")[0]; new=re.findall(r"^\| `[0-9]{3}-[a-z0-9-]+` \| `([0-9]{3}-[a-z0-9-]+)` \|$", s2, re.M)+re.findall(r"^- `([0-9]{3}-[a-z0-9-]+)`$", s3, re.M); o=sorted(int(i[:3]) for i in new); assert len(set(o))==len(o), "a renumbered ordinal repeats"; assert o==list(range(len(o))), "the renumber left a gap"; assert o[-1]==95, o[-1]' "${TMPDIR:-/tmp}/ss118/map-gap.md"
# ...and one row duplicated repeats an ordinal.
sh -c 'mkdir -p "${TMPDIR:-/tmp}/ss118" && awk "{print} /\\| \`050-/{print}" docs/corpus-map.md > "${TMPDIR:-/tmp}/ss118/map-dup.md" && test "$(grep -c "| \`050-" "${TMPDIR:-/tmp}/ss118/map-dup.md")" -gt "$(grep -c "| \`050-" docs/corpus-map.md)"'
! python3 -c 'import re,sys; t=open(sys.argv[1]).read(); s2=t.split("\n## 2.")[1].split("\n## 3.")[0]; s3=t.split("\n## 3.")[1].split("\n## 4.")[0]; new=re.findall(r"^\| `[0-9]{3}-[a-z0-9-]+` \| `([0-9]{3}-[a-z0-9-]+)` \|$", s2, re.M)+re.findall(r"^- `([0-9]{3}-[a-z0-9-]+)`$", s3, re.M); o=sorted(int(i[:3]) for i in new); assert len(set(o))==len(o), "a renumbered ordinal repeats"; assert o==list(range(len(o))), "the renumber left a gap"; assert o[-1]==95, o[-1]' "${TMPDIR:-/tmp}/ss118/map-dup.md"
# --- 118 3.3: the live corpus, by rules that survive growth ---
python3 -c 'import re,subprocess; ids=subprocess.check_output(["target/release/spec-spine","registry","list","--ids-only"], text=True).split(); bad=[i for i in ids if not re.fullmatch(r"[0-9]{3}-[a-z0-9]+(-[a-z0-9]+)*", i)]; assert ids and not bad, bad; o=[int(i[:3]) for i in ids]; assert all(a<b for a,b in zip(o,o[1:])), "ordinals repeat or are out of order"; print(len(ids), "ids, unique and ordered; gaps permitted")'
test "$(ls -d specs/*/ | sed 's#^specs/##; s#/$##' | sort)" = "$(target/release/spec-spine registry list --ids-only | sort)"
# --- 118 3.4: fixture corpora, each isolated under this block's TMPDIR ---
# 1. A reserved later ordinal is legitimate.
sh -c 'r="${TMPDIR:-/tmp}/ss118/gap"; rm -rf "$r"; for s in 000-first 001-second 117-after-a-reserved-gap; do mkdir -p "$r/specs/$s" && printf -- "---\nid: \"%s\"\ntitle: \"t\"\nstatus: draft\ncreated: \"2026-09-22\"\nsummary: \"s\"\n---\n\n# t\n" "$s" > "$r/specs/$s/spec.md" || exit 1; done'
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss118/gap" compile
test "$(target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss118/gap" registry list --ids-only | tr '\n' ' ')" = "000-first 001-second 117-after-a-reserved-gap "
# 2. Two specs sharing an ordinal are refused.
sh -c 'r="${TMPDIR:-/tmp}/ss118/dup"; rm -rf "$r"; for s in 117-a 117-b; do mkdir -p "$r/specs/$s" && printf -- "---\nid: \"%s\"\ntitle: \"t\"\nstatus: draft\ncreated: \"2026-09-22\"\nsummary: \"s\"\n---\n\n# t\n" "$s" > "$r/specs/$s/spec.md" || exit 1; done'
sh -c 'target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss118/dup" compile >/dev/null 2>&1; test $? -eq 1'
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss118/dup" compile 2>&1 | grep -q "V-004"
# 3. An id outside the grammar is refused.
sh -c 'r="${TMPDIR:-/tmp}/ss118/bad"; rm -rf "$r"; mkdir -p "$r/specs/17-short" && printf -- "---\nid: \"17-short\"\ntitle: \"t\"\nstatus: draft\ncreated: \"2026-09-22\"\nsummary: \"s\"\n---\n\n# t\n" > "$r/specs/17-short/spec.md"'
sh -c 'target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss118/bad" compile >/dev/null 2>&1; test $? -eq 1'
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss118/bad" compile 2>&1 | grep -q "V-012"
# 4. A directory that does not equal its id is refused.
sh -c 'r="${TMPDIR:-/tmp}/ss118/dir"; rm -rf "$r"; mkdir -p "$r/specs/118-x" && printf -- "---\nid: \"118-y\"\ntitle: \"t\"\nstatus: draft\ncreated: \"2026-09-22\"\nsummary: \"s\"\n---\n\n# t\n" > "$r/specs/118-x/spec.md"'
sh -c 'target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss118/dir" compile >/dev/null 2>&1; test $? -eq 1'
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss118/dir" compile 2>&1 | grep -q "V-001"
rm -rf "${TMPDIR:-/tmp}/ss118"
```
