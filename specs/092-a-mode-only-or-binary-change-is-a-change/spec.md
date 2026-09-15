---
id: "092-a-mode-only-or-binary-change-is-a-change"
title: "A mode-only or binary change is a change"
status: draft
kind: "tooling"
created: "2026-09-14"
summary: >
  `couple` builds its `DiffInput` by parsing `git diff --no-color -U0
  --no-renames`, and registers a path only when it meets a `---` or `+++`
  header. Git prints neither for a mode-only change (`old mode` / `new mode`)
  nor for a binary change (`Binary files a/x and b/x differ`), so both arrive
  at the gate as no change at all: an owned binary can change without its
  spec and `C-001` stays silent, and a mode flip on an unclaimed source file
  is invisible to `C-002`. Spec 088 met the same blindness while building
  `delta`, worked around it with a `--name-only` helper, and recorded in D-9
  that `couple` is spec 005's territory. This spec unions the name list into
  the parsed diff: a path git reports and the hunk parser did not register
  enters `DiffInput` as a whole-file change with no spans, deleted when the
  status letter says so, and is then judged by the same clearance rules as
  every other path.
implementation: pending
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "005-coupling-gate"
  - "032-ownership-coverage"
  - "088-a-change-is-classified-under-the-bases-rules"
extends:
  # 3.1 to 3.4: the parser and the union, in the CLI's diff adapter.
  - { spec: "005-coupling-gate", unit: "crates/spec-spine-cli/src/cmd_couple.rs", nature: corrective }
  # 3.5: the helper 088 added for `delta` grows status letters and is shared.
  - { spec: "088-a-change-is-classified-under-the-bases-rules", unit: "crates/spec-spine-cli/src/cmd_couple.rs", nature: additive }
  # 3.6: the ownership ratchet's input universe gains the paths it could not see.
  - { spec: "032-ownership-coverage", unit: "crates/spec-spine-cli/src/cmd_couple.rs", nature: additive }
  # 3.7: the fail-first tests, beside the existing real-git-diff cases.
  - { spec: "005-coupling-gate", unit: "crates/spec-spine-cli/tests/couple.rs", nature: additive }
references:
  - { unit: { kind: file, path: "docs/design/05-remaining-waves-2026-09.md" }, role: context }
---

# 092: A mode-only or binary change is a change

## 1. Purpose

### 1.1 The parser reads spans and infers membership from them

`cmd_couple.rs::parse_unified_diff` walks the diff line by line. It sets the
current path from a `+++` header (or from the preceding `---` when the new side
is `/dev/null`, which is how a deletion is recognized), and attaches every
following `@@` hunk to that path. Membership in `DiffInput` is therefore a
by-product of having seen a header, and the header is a by-product of git having
printed a textual hunk.

### 1.2 Git prints no header for two real change shapes

Measured on 2026-09-14 with the exact invocation `couple` runs
(`git diff --no-color -U0 --no-renames`):

```
diff --git a/a.sh b/a.sh          diff --git a/b.bin b/b.bin
old mode 100644                   index 8352675..1592e5c 100644
new mode 100755                   Binary files a/b.bin and b/b.bin differ
```

A binary add prints `new file mode` and `Binary files /dev/null and b/c.bin
differ`; a binary delete prints `deleted file mode` and `Binary files a/c.bin
and /dev/null differ`. None of the four carries `---` or `+++`.

So `parse_unified_diff` returns an empty `files` list for a pull request that
changes only a mode or only a binary, and `couple` reports that it checked no
paths and found no drift. Both refusals the gate exists to raise are absent:

- **`C-001`.** A spec that owns a binary unit (a fixture, an icon, a golden
  archive) cannot be held to it. The bytes change, the owning `spec.md` does
  not, and the gate passes.
- **`C-002`.** With `[coupling] require_ownership` on, an unclaimed source file
  is a refusal when it changes. A mode flip on an unclaimed `.sh` is a change to
  a tracked source file that the ratchet never sees. `.sh` is in
  `coverage.rs::SOURCE_EXTS`, so the file is in the ratchet's universe; only the
  diff hid it.

### 1.3 The workaround already in the tree

Spec 088 needed the same path list for `delta` and could not use the parser, so
it added `changed_path_names` in the same file: `git diff --name-only -z
--no-renames --end-of-options`, with `core.quotepath=false` so a non-ASCII path
arrives as bytes rather than as an escape sequence. Its D-9 records that the
parser was left alone because `cmd_couple.rs`'s diff adapter is spec 005's
territory and a coupling change belongs in a coupling spec. `delta` therefore
classifies a binary change today and `couple` does not, which is the state this
spec ends.

### 1.4 Why the name list is the authority for membership

`git diff --name-status` answers exactly the question `DiffInput` needs
answered, for every change shape, in one invocation that is already understood
in this file. The hunk parser stays the authority for **spans**, which the name
list cannot supply and which `C-001`'s section and symbol granularity needs.
The union is therefore not a second parser: it is the membership half of the
same question, taken from the source that answers it completely.

## 2. Territory

This spec changes `crates/spec-spine-cli/src/cmd_couple.rs` (the diff adapter
and the helper 088 added there) and adds cases to
`crates/spec-spine-cli/tests/couple.rs`. It changes no core function: the
library already accepts a whole-file `DiffFile` with an empty `hunks` list,
which is the shape `--paths-from` has always produced. No verdict payload, no
schema and no exit-code mapping changes.

## 3. Behavior

### 3.1 The union

After parsing the diff text, the CLI MUST ask git for the changed paths and
their status letters, and MUST add to `DiffInput` every path the parser did not
register. An added entry carries an empty `hunks` list, which the library
already reads as a whole-file change.

A path present in both sources MUST keep the spans the parser found. The name
list contributes membership only; it never replaces or truncates a hunk list.

### 3.2 Deletion is read from the status letter

A `D` status MUST set `deleted: true` on the added entry, so the spec 032
ownership ratchet leaves it alone exactly as it does for a path the parser
recognized through `+++ /dev/null`. Every other status letter (`A`, `M`, `T`,
`C`, `U`) MUST produce `deleted: false`.

`--no-renames` is already passed, so a rename arrives as a delete plus an add
and needs no mapping. This spec does not infer that the two are the same file:
note 04's A07 records why a similarity score is not authority, and 088 reports
the pair rather than the identity for the same reason.

### 3.3 One git invocation shape

The status query MUST use the invocation `changed_path_names` established
(spec 088): `-c core.quotepath=false`, `--no-renames`, `--end-of-options`, and
`-z` so paths are NUL-separated and never quoted. The existing helper SHOULD be
generalized to return `(status, path)` pairs with the name-only form expressed
in terms of it, so `couple` and `delta` cannot disagree about which paths
changed.

A path git reports that the parser also registered is not queried twice: the
union is computed once, in memory, after both answers are in hand.

### 3.4 What a whole-file entry means to the clearance rules

Nothing in `couple.rs` changes. A whole-file entry resolves ownership at file
granularity, so:

- a section or symbol unit whose file is reported whole-file MUST be treated as
  the library already treats a whole-file change to that file, which is how
  `--paths-from` input has always been judged;
- the bypass floor, `[coupling] bypass_prefixes`, the dependency-only waiver
  (specs 005 and 030) and the `Spec-Drift-Waiver:` path all apply unchanged. A
  binary under a bypassed prefix stays bypassed.

### 3.5 `--paths-from` is unaffected

`couple --paths-from` supplies paths without spans and does not run git. It
already produces exactly the shape §3.1 adds, and this spec does not change it.
An orchestrator that adjudicates outside a pull request keeps working as before.

### 3.6 The refusal this adds

After this spec, a pull request whose only change is a mode flip or a binary
edit is judged. Where the path is owned and no owning `spec.md` is in the diff,
`couple` exits 1 with `C-001`; where `require_ownership` is on and the path is
an unclaimed source file, it exits 1 with `C-002`. This is a widening of what
the gate refuses, it is the point of the spec, and §5 D-1 records it as the
decision a reviewer is accepting.

### 3.7 The tests

`crates/spec-spine-cli/tests/couple.rs` already builds real git repositories
(`git_in`, `couple_git`, `real_git_diff_detects_drift`). The new cases MUST use
that machinery and MUST assert through the CLI, since the defect is in the CLI's
diff adapter and a library-level assertion cannot see it:

1. a mode-only change (`chmod +x`) to a path an owning spec claims, with that
   spec's `spec.md` absent from the diff, exits 1;
2. a binary content change to a claimed path, same shape, exits 1;
3. a binary **delete** of a claimed path is judged as a deletion: the ownership
   ratchet does not raise `C-002` for a file that is gone;
4. a mode-only change whose owning `spec.md` is in the same diff clears, so the
   union did not break the ordinary clearance path;
5. a text change to the same path still reports its hunk spans, so the union did
   not flatten a path the parser handled.

## 4. Out of scope

**Rename and copy detection.** `--no-renames` stays. A reviewed move mapping is
note 04's A07 and needs authored data, not a similarity score.

**`couple --head HEAD` reading `base...head`.** Spec 090 §4 named this
separately: the documented pre-commit coupling check cannot see the change being
committed. That is a different defect in the same verb and is not fixed here.

**Widening `SOURCE_EXTS` or `C-002`'s universe.** A tracked file outside the
source extension list is still outside the ratchet, whatever the diff shape.
That hole is real, it is recorded in design note 05 §3, and it is a spec with
adopter blast radius rather than a rider on this one.

**`delta`'s classification.** 088 already sees these paths. Generalizing the
shared helper (§3.3) must not change what `delta` reports, and the test suite
for 088 is the guard.

## 5. Resolved decisions

**D-1 (2026-09-14). The refusal widens, deliberately.** Filing this spec
accepts that an owned binary and a mode flip become governed changes. The
alternative considered was to report them as a warning tier first; it was
rejected because the gate has no warning tier for a coupling finding, and
inventing one for two change shapes would mean two answers to "is this path
governed" depending on how git chose to print it.

**D-2 (2026-09-14). Membership from the name list, spans from the parser.**
Rather than teaching `parse_unified_diff` to recognize `old mode`, `new mode`
and `Binary files ... differ` lines, the path list comes from git's own
complete answer. The recognizer approach was rejected because it would grow a
second grammar for every future diff shape git can print, and the name list has
no such tail.

## Verification

Each line is one command. Two lines fail against pre-092 code and are the
fail-first evidence: `--name-status` appears nowhere in the diff adapter today,
and neither does a status-letter comparison. The `core.quotepath` and
`--no-renames` lines already pass, because they pin the invocation shape spec
088 established and this spec must not change. The `cargo test` line is **not**
fail-first: at the parent commit the cases in §3.7 do not exist, so the suite
passes vacuously. It is listed to keep those cases from being deleted later.

```verify:cli
# 3.1, 3.3: the status query, with the invocation 088 established.
grep -qF -- '--name-status' crates/spec-spine-cli/src/cmd_couple.rs
grep -qF -- 'core.quotepath=false' crates/spec-spine-cli/src/cmd_couple.rs
# 3.2: the status letter, not a second diff parse, decides deletion.
grep -qF -- '"D"' crates/spec-spine-cli/src/cmd_couple.rs
# 3.7: the five cases, asserted through the CLI against real git repositories.
cargo test -p spec-spine-cli --test couple --locked
# 4: renames stay undetected, so a move is still a delete plus an add.
grep -qF -- '--no-renames' crates/spec-spine-cli/src/cmd_couple.rs
```
