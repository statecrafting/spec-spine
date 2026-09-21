---
id: "081-coupling-sees-the-change-being-committed"
title: "The coupling gate can see the change being committed"
status: approved
kind: "tooling"
created: "2026-09-16"
summary: >
  `couple --base X --head HEAD` builds a `git diff X...HEAD`, so every path in
  the working tree and the index is invisible to it. The documented pre-commit
  coupling check therefore cannot see the change it is being run to judge: on a
  branch whose work is not yet committed it reports `0 path(s) checked, no
  drift` and exits 0, which reads as a pass. Spec 094 §4 placed the defect out
  of scope and said it "is filed separately"; spec 093 §4 placed it out of scope
  and pointed back at 090. Nothing filed it, and the phrase in 090 §4 has been
  untrue since it was written. This spec files it: `--include-uncommitted`
  unions the committed range with `git diff HEAD`, so the gate judges what a
  commit would contain. The library stays pure and still shells out to nothing;
  the CLI does the git work, as it already does for the range.
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "005-coupling-gate"
  - "073-a-mode-only-or-binary-change-is-a-change"
extends:
  # 3.1, 3.2: the flag and the union. `run_git_diff` and `changed_path_statuses`
  # are the only place this tool runs git, and they stay so.
  - { spec: "005-coupling-gate", unit: "crates/spec-spine-cli/src/cmd_couple.rs", nature: additive }
  # 3.1: the flag's clap definition and its dispatch. Found by running the very
  # gate this spec builds: `couple --include-uncommitted` refused the change on
  # a C-001 for this path, which the unflagged verb reported as 0 paths checked.
  - { spec: "005-coupling-gate", unit: "crates/spec-spine-cli/src/main.rs", nature: additive }
  # 3.4: the acceptance.
  - { spec: "009-registry-query-projection-flags", unit: "crates/spec-spine-cli/tests/cli.rs", nature: additive }
references:
  - { unit: { kind: file, path: "docs/design/05-remaining-waves-2026-09.md" }, role: context }
  - { unit: { kind: file, path: "docs/design/06-harness-and-distribution-2026-09.md" }, role: context }
---
# 102: The coupling gate can see the change being committed

## 1. Purpose

### 1.1 A gate that reports on nothing reports a pass

`build_diff_input` runs `git diff --no-color -U0 --no-renames base...head` and
pairs it with `git diff --name-status` over the same three-dot range. Both
describe history. Neither can see the index or the working tree.

So on a branch where the work is written but not committed, which is exactly
the state a pre-commit check runs in:

```
$ spec-spine couple --base origin/main --head HEAD
spec-spine couple: OK: 0 path(s) checked, no drift.
$ echo $?
0
```

Observed in this repository on 2026-09-16 while building spec 095, with
twenty-two governed paths edited and staged. The gate did not fail to judge
them; it did not know they existed. "0 path(s) checked" is in the output, and
it is the only thing distinguishing that line from a real pass, which is a
distinction a reader looking for a non-zero exit code will not make.

### 1.2 Two out-of-scope clauses pointing at each other

Spec 094 §4:

> **A staged-diff mode for `couple`.** `couple --head HEAD` reading
> `base...head` means the documented pre-commit coupling check cannot see the
> change being committed. That is a real defect in a different verb and is filed
> separately; section 3.2 refuses to paper over it here.

Spec 093 §4:

> **`couple --head HEAD` reading `base...head`.** Spec 094 §4 named this
> separately: the documented pre-commit coupling check cannot see the change
> being committed. That is a different defect in the same verb and is not fixed
> here.

Design note 06 §3.9 checked the corpus at `3bc004b` and found no spec filing
it. Re-checked on 2026-09-16 at `a6ef6e3`, across all 100 specs: still none.
090's "is filed separately" was untrue on the day it was written and stayed
untrue for six specs. 092's sentence is accurate as written, which §4 records.

### 1.3 Why the fix belongs in `couple` and not in the harness

The harness could commit first and check afterwards, and that would make this
repository's own hook honest. It would do nothing for an adopter who runs
`couple` by hand before committing, which the kit's own documentation invites,
and it would leave the verb reporting a pass over an empty set for everyone
else. The defect is that the verb cannot be asked the question; the fix is to
let it be asked.

Making it a **flag** rather than the default is what keeps the change additive.
CI runs `couple` over a pushed range where the working tree is irrelevant and
must stay so: a dirty checkout on a runner must never change a gate verdict.

## 2. Territory

No new file. One flag, one union, one amendment:

| Unit | Edge | Why |
|---|---|---|
| `090` | `amends` | §4's "is filed separately". |
| `crates/spec-spine-cli/src/cmd_couple.rs` | `extends` 005, additive | The flag and the union. |
| `crates/spec-spine-cli/src/main.rs` | `extends` 005, additive | The flag's clap definition. |
| `crates/spec-spine-cli/tests/cli.rs` | `extends` 010, additive | The acceptance. |

`crates/spec-spine-core/**` is untouched. The library is a pure function of
`(Config, file contents)` and runs no git; this spec adds git work to the one
file that already does it.

## 3. Behavior

### 3.1 `--include-uncommitted` adds the working tree to the range

`spec-spine couple` MUST accept `--include-uncommitted`. With it, the
`DiffInput` is the union of:

- `git diff --no-color -U0 --no-renames --end-of-options <base>...<head>`, the
  committed range, unchanged; and
- `git diff --no-color -U0 --no-renames --end-of-options HEAD`, the index and
  the working tree against the commit they sit on.

Both MUST be paired with a `--name-status` read over the same two ranges, so
spec 093's rule holds for the new half as well: the parser is the authority for
spans and the name list for membership, which is what carries a mode-only or a
binary change that git prints no `+++` header for.

A path only the working tree knows enters as a whole-file change. A path **both**
views know keeps the **union** of their hunks: the change a commit would record
touches every line either view touched, and dropping the later view's spans
would let a working-tree edit to a line the committed range never touched go
unjudged. A file is deleted for the gate's purposes when the **later** of the
two views says so, because that is the state a commit would record.

The two rules differ on purpose. Spans accumulate because they describe what
changed, and both views changed something. The deletion verdict does not
accumulate because it describes a final state, and only the later view knows
it.

### 3.2 Untracked files are out, and that is not a gap

`git diff HEAD` covers staged and unstaged changes to tracked files. A file
that has never been `git add`-ed is invisible to it, and MUST stay invisible:
a commit would not contain it either, so judging it would refuse a change that
is not being made. A new file that has been staged **is** in `git diff HEAD`
and is therefore judged, which is the case the ownership ratchet cares about.

### 3.3 The flag is meaningful only against `HEAD`

`--include-uncommitted` compares the working tree with `HEAD`, so it MUST be
refused with exit `3` when `--head` names anything that does not resolve to the
same commit as `HEAD`. Silently unioning a working tree against an unrelated
commit would produce a diff describing no state that ever existed.

It MUST also be refused with exit `3` alongside `--paths-from`, which carries
its own path list and no history to union with.

### 3.4 The default does not move

Without the flag, `couple` behaves exactly as it does today, byte for byte in
its output and identically in its exit code. CI's invocation is unflagged and
its verdict MUST NOT depend on the state of the runner's working tree.

### 3.5 What spec 094 §4 now says

This replaces the second and third sentences of spec 094 §4's staged-diff
clause. Under spec 037 the replacement text lives here, and 090 is not edited.
Where 090 reads:

> That is a real defect in a different verb and is filed separately; section 3.2
> refuses to paper over it here.

it now reads:

> That is a real defect in a different verb. It was not filed when this spec was
> written, despite the sentence that stood here saying it was; spec 081 files
> it and adds `couple --include-uncommitted`. Section 3.2 refuses to paper over
> it here, and still does: this spec's hook is not changed to pass the flag.

The rest of 090 §4 stands, §3.2 included. Nothing about spec 094's own
behavior changes: its hook still runs the unflagged verb.

### 3.6 The acceptance

`crates/spec-spine-cli/tests/cli.rs` MUST assert, in a scratch repository with
real commits:

- a governed file edited and **staged** but not committed: unflagged `couple`
  reports `0 path(s) checked` and exits 0; with `--include-uncommitted` the
  path is checked and the drift refused;
- the same for an edit that is neither staged nor committed;
- a path changed in the committed range **and** again in the working tree is
  checked once, not twice;
- `--include-uncommitted --head <a non-HEAD commit>` exits 3;
- `--include-uncommitted --paths-from <file>` exits 3;
- the unflagged verdict over a range is unchanged by a dirty working tree.

The first two fail against pre-102 code, which does not have the flag: clap
refuses the unknown argument with exit 3 where the assertion expects a drift
refusal. The last is the regression guard for §3.4.

## 4. Out of scope

- **Amending spec 093 §4.** Its sentence says spec 094 "named this separately",
  which is true: 090 did name it. The false claim is 090's "is filed
  separately", and that is the only one amended here. Correcting a sentence
  that is accurate would be noise in the amendment record, and design note 05
  R-3's phrasing ("both say the other filed it") is looser than note 06 §3.9's,
  which isolates the untrue phrase correctly.
- **Making it the default.** §1.3 and §3.4. A gate whose verdict depends on the
  runner's working tree is a gate that cannot be reproduced.
- **Untracked files.** §3.2.
- **The harness adopting the flag.** Whether this repository's `pre-commit`
  hook, `kit/Makefile` or the `PreToolUse` gate pass `--include-uncommitted` is
  a change to those files with its own adopter-facing consequence. This spec
  makes the flag exist and correct; wiring it in is separate work, and spec
  094 §3.2's refusal to paper over the gap is unaffected either way.
- **`git stash`-based approaches.** Spec 068's fail-first probe used a partial
  stash and recorded how easily one loses work. A read-only union of two diffs
  touches nothing.

## 5. Resolved decisions

D-1 (2026-09-16, why two `git diff` invocations rather than one over a range
that spans the working tree). Git has no range syntax naming "the merge base of
X and HEAD, through to the working tree". `git diff <base>` alone compares the
base with the working tree but uses a two-dot comparison, which reports every
change made on the base branch since the fork as though this branch made it.
Keeping the three-dot range and adding `git diff HEAD` preserves the merge-base
semantics the gate has always had.

D-2 (2026-09-16, why the later view wins a deletion disagreement). A file
deleted in the committed range and restored in the working tree is not deleted
by the change a commit would record, and the reverse is a deletion. The gate
judges the state the commit would produce, so the working tree is the later
word.

D-4 (2026-09-16, why hunks union where the deletion verdict overwrites). Added
during the build, from a review finding: §3.1's first draft said a path the
committed range knows "keeps its spans", which describes neither what the code
does nor what the gate needs. A span says what changed and both views changed
something, so the union is the honest answer; a deletion verdict says what the
final state is, and only the later view knows that. The code was right and the
sentence was wrong, which is the direction worth recording: an acceptance that
passes tells you nothing about a clause that describes the wrong mechanism.

D-3 (2026-09-16, why exit 3 rather than a warning for a non-HEAD `--head`). The
result would otherwise be a diff describing no state that ever existed, and a
gate's answer over nonsense input is worse than no answer. Spec 093 maps every
usage error to exit 3, and this is one.

## Verification

Each line is one command (spec 043 §3.2).

The first line is the regression guard of §3.4, run against this repository:
the unflagged verb still answers, and its verdict does not depend on whether
the working tree is clean. It cannot fail on the defect and is here to fail on
an over-broad fix, which is the failure mode adding a diff source has.

The second and third lines are the fail-first evidence. Their filter selects
tests that do not exist at the parent commit, where `cargo test` runs zero of
them and reports `ok`: a filter matching nothing passes, so the count is
asserted before the suite is trusted. Each selected test builds its own git
repository with real commits, because the defect is about what `git diff` can
and cannot see and no fixture without history can reproduce it.

```verify:cli
target/release/spec-spine couple --base "$(git symbolic-ref --short refs/remotes/origin/HEAD 2>/dev/null || echo origin/main)" --head HEAD
sh -c 'n=$(cargo test -p spec-spine-cli --test cli --locked spec102_ 2>&1 | grep -c "^test spec102_"); test "$n" -ge 6 || { echo "expected at least 6 spec102_ tests, ran $n"; exit 1; }'
cargo test -p spec-spine-cli --test cli --locked spec102_
```
