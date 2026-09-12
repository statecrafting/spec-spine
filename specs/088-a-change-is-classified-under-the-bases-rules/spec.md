---
id: "088-a-change-is-classified-under-the-bases-rules"
title: "A change is classified under the base's rules"
status: draft
kind: "tooling"
created: "2026-09-11"
implementation: pending
owner: "The spec-spine Authors"
risk: high
depends_on:
  - "000-spec-spine-bootstrap"
  - "005-coupling-gate"
  - "030-cargo-workflow-dependency-waiver"
  - "037-machine-readable-verdicts"
  - "040-amendment-authoring"
  - "049-verify-declared-acceptance"
  - "055-the-ledger-answers-what-consumers-rebuild"
  - "086-the-committed-index-is-compared-not-trusted"
establishes:
  # Planned (spec 076) until the build writes them; the build drops the flag.
  # 3.6: the report DTO and `DELTA_SCHEMA_VERSION`.
  - { kind: file, path: "crates/spec-spine-types/src/delta.rs", planned: true }
  # 3.2 to 3.5: the pure classifier over two trees.
  - { kind: file, path: "crates/spec-spine-core/src/delta.rs", planned: true }
  # 3.8: the classification matrix.
  - { kind: file, path: "crates/spec-spine-core/tests/delta.rs", planned: true }
  # 3.1: the verb: git, the two exports, and the envelope.
  - { kind: file, path: "crates/spec-spine-cli/src/cmd_delta.rs", planned: true }
extends:
  # 3.1: the verb's declaration, and the envelope's new verb token.
  - { spec: "001-compile-registry", unit: "crates/spec-spine-cli/src/main.rs", nature: additive }
  - { spec: "037-machine-readable-verdicts", unit: "crates/spec-spine-types/src/verdict.rs", nature: additive }
  # 3.1: the merge-base and diff plumbing `couple` already has, shared rather than copied.
  - { spec: "005-coupling-gate", unit: "crates/spec-spine-cli/src/cmd_couple.rs", nature: additive }
  # 3.6: the module, its re-exports, the facade, and the schema constant.
  - { spec: "001-compile-registry", unit: "crates/spec-spine-core/src/lib.rs", nature: additive }
  - { spec: "000-spec-spine-bootstrap", unit: "crates/spec-spine-types/src/lib.rs", nature: additive }
  - { spec: "000-spec-spine-bootstrap", unit: "crates/spec-spine-types/src/version.rs", nature: additive }
  # 3.8: end-to-end coverage of the verb.
  - { spec: "001-compile-registry", unit: "crates/spec-spine-cli/tests/cli.rs", nature: additive }
references:
  - { unit: { kind: file, path: "docs/authority-evidence.md" }, role: context }
  - { unit: { kind: file, path: "docs/design/02-agentic-builder-substrate.md" }, role: context }
  - { unit: { kind: file, path: "docs/design/04-authority-evidence-extension.md" }, role: context }
summary: >
  The coupling gate clears C-001 when any owning spec's `spec.md` is in the
  diff, and reads the owners from the candidate's own index. Measured on a
  scratch repository, a candidate that deletes a command from its own
  `## Verification` block while changing the code that command checked passes
  `couple` at exit 0, and so does a candidate that files a new draft spec with
  an `extends` edge on another spec's unit and rewrites that unit. Both are
  changes a reviewer must weigh under the rules that held before the change,
  and nothing reports them as such. Design note 02 G7 says no refusal should be
  asserted before it is designed, because the naive rule refuses legitimate
  refactors. This spec therefore adds a report, not a gate: `spec-spine delta
  --base B --head H` classifies every changed path into implementation,
  requirement, verification, authority, lifecycle, constitutional, policy,
  derived, bypassed, unowned or unknown, computed under the merge base's own
  configuration and index so a candidate cannot reclassify its change by
  editing `spec-spine.toml`. It records the base and head plan digests for
  every spec whose declared acceptance changed, the owner sets on both sides
  of every path whose ownership moved, and a `priorPolicy` block naming the
  classes a consumer must judge under the base's policy. It interprets no
  prose, infers no moves, and decides nothing.
---

# 088: A change is classified under the base's rules

## 1. Purpose

A consumer that approves or rejects a candidate needs to know what kind of
change it is judging. spec-spine knows the structure: which paths are owned,
by whom, which spec text declares which acceptance. It reports none of it per
change. `couple` answers one question, whether owned code moved without an
owning `spec.md` in the same diff, and the answer is yes for two changes that
should never be approved on the candidate's own say-so. Measured on 2026-09-11
in a scratch repository whose approved spec `001-a` establishes `src/a.rs` and
declares two acceptance commands (`test -f src/a.rs`, `grep -q "fn a"
src/a.rs`):

| Candidate | `couple` | Plan after |
|---|---|---|
| deletes the `grep` line from 001-a's `## Verification` and renames `fn a` | exit 0, no violation | `test -f src/a.rs` only, which passes |
| files draft `002-b` with `extends: {spec: 001-a, unit: src/a.rs}` and rewrites `src/a.rs` | exit 0, no violation | unchanged; `index owner src/a.rs` now names 002-b too |

The first is a candidate weakening the acceptance that judges it. The second
is an authority transfer approved by nobody outside the diff. Both are the
sanctioned shapes for legitimate work too: specs are edited when behavior
changes, and `extends` is how a spec claims a unit it must touch (spec 047).
That is exactly why note 02 G7 refuses to assert a mechanism before a design:
a rule that refuses these shapes refuses refactors.

What can be done without that design is to say what changed, structurally,
under rules the candidate did not write. A reviewer, or an issuer granting a
permit, then decides under the base's policy. statecraft-cli already
approximates the first half locally with a prefix list of policy-sensitive
paths; this is the version that knows what a spec is.

## 2. Territory

- `crates/spec-spine-types/src/delta.rs` and `crates/spec-spine-core/src/delta.rs`:
  the report and the pure classifier.
- `crates/spec-spine-cli/src/cmd_delta.rs`: the verb.
- `cmd_couple.rs`: the merge-base and diff helpers, shared.
- `main.rs`, `verdict.rs`, both `lib.rs`, `version.rs`: declarations, the
  `delta` verb token, the facade, `DELTA_SCHEMA_VERSION`.
- `crates/spec-spine-core/tests/delta.rs` and `crates/spec-spine-cli/tests/cli.rs`.

## 3. Behavior

### 3.1 The verb and the seam

`spec-spine delta --base <ref> --head <ref> [--json]` MUST:

1. resolve `merge-base(base, head)`, as `couple` does, and list the changed
   paths of `merge-base...head` with renames disabled;
2. export the merge-base tree and the head tree to two temporary directories
   (the committed `.derived/` included), and remove them afterwards;
3. pass both roots, the path list and the three commit ids to the core.

The core function is pure over `(base root, head root, changed paths,
commit ids)`: it reads files, never git, never the clock. The commit ids are
echoed, not computed.

The verb exits 0 whenever a report was produced, whatever it says, and 3 for
I/O, git or parse trouble. It is a record, not a gate (spec 042 3.1's rule for
`attest`, for the same reason).

### 3.2 The base's rules classify

Classification MUST use the merge base's `spec-spine.toml` and the merge base's
committed index (read through the loaders and compared under spec 086 before
use). A change to the configuration is itself classified (`policy`); it does
not change how the rest of its own diff is classified. Owner sets are derived
with `owners_for_path` (spec 055) against the base index and against the head
index, whole-file.

### 3.3 Classes

A path carries every class that applies, and at least one.

| Class | Applies when |
|---|---|
| `implementation` | the path has at least one owner at base or head and is none of the kinds below |
| `requirement` | a `spec.md` whose body outside `## Verification` changed, or whose frontmatter changed in a key no other class names |
| `verification` | a `spec.md` whose `verify:cli` plan (spec 049's parser) differs between base and head, including a spec added or removed |
| `authority` | a `spec.md` whose typed edges or `depends_on` changed; or any path whose owner set differs between base and head |
| `lifecycle` | `status`, `implementation`, `superseded_by` or `retirement_rationale` changed |
| `constitutional` | a path under the base's `standards_dir`; or a `spec.md` that declares `unamendable` anchors at base or head |
| `policy` | `spec-spine.toml`, or a path matched by the base's `[index] extra_hashed_inputs` |
| `derived` | a path under the base's `derived_dir` |
| `bypassed` | covered by the base's bypass floor and configured prefixes, and by no class above |
| `unowned` | no owner at base or head, not bypassed |
| `unknown` | a `spec.md` or configuration that fails to parse on either side, or anything the rules above cannot place |

Frontmatter keys spec-spine does not model, including declared
`extra_known_keys`, are `requirement`: an unrecognized change is treated as a
change to what is required, never as nothing. A parse failure is `unknown`,
never `implementation`.

With renames disabled, a move is a removal and an addition. The report never
infers that the two are one file; identity transfer is a reviewed mapping and a
later proposal (design note 04, P2).

### 3.4 Detail carried per class

- `verification`: `basePlanHash` and `headPlanHash`, each SHA-256 of the
  canonical JSON of the plan's `commands` list (absent on the side where the
  spec does not exist), and the counts of commands only in base and only in
  head.
- `authority`: for a `spec.md`, the edges added and removed by type; for any
  path, `baseOwners` and `headOwners`.
- `lifecycle`: each changed key with its base and head value.

### 3.5 Prior policy

`priorPolicy.required` MUST be true when any path carries `requirement`,
`verification`, `authority`, `lifecycle`, `constitutional`, `policy` or
`unknown`, and `priorPolicy.classes` lists which. The report MUST say, in the
verb's help and in `docs/authority-evidence.md`, that `required: false` means
only that no structural class above changed. It does not mean the change is
safe, correct or approved.

spec-spine never evaluates the approval. A consumer judges each listed class
under the base revision's policy and records who approved it.

### 3.6 Report and versioning

The report is canonical JSON: `schemaVersion`, `tool`, `classifiedUnder:
"base"`, `base`, `mergeBase`, `head`, `changes` sorted by path, `counts` per
class, and `priorPolicy`. `DELTA_SCHEMA_VERSION` starts at `0.1.0` on its own
axis. The envelope gains the verb token `delta`, an additive change that moves
`VERDICT_SCHEMA_VERSION` to its next MINOR (spec 037's own rule). The facade
`delta_json(request)` takes `{ config?, baseRoot, headRoot, changed, commits }`
and returns the same report.

### 3.7 What it does not claim

It interprets no prose: any byte change to a spec body outside `## Verification`
is `requirement`, whatever the words say. It cannot see a change to a file a
verification command reads unless that file is declared as the command's input;
today such a change is `implementation`, and design note 04 4.4 proposes the
declaration. It decides nothing and refuses nothing.

### 3.8 Tests (minimum)

The classification matrix covers: a code edit with its owning spec untouched;
a body edit; a `## Verification` command removed (the first row of 1); a new
spec with an `extends` edge on another spec's unit plus an edit to that unit
(the second row of 1); a status flip; a `spec-spine.toml` edit that also tries
to widen the bypass floor for its own diff; an edit under `standards/`; a
`git mv` of an owned file; a `spec.md` that fails to parse at head; and an
unchanged tree, which reports no changes and `required: false`.

## 4. Out of scope

**Refusing anything.** Note 02 G7.

**Recording approvals.** The consumer's.

**Obligation ids and declared verifier inputs.** Design note 04 4.4.

**Similarity-based move detection.** 3.3.

**Semantic comparison of prose.** 3.7.

## 5. Resolved decisions

**D-1 (2026-09-11): the base's configuration and index classify.** A candidate
that edits `spec-spine.toml` to add a bypass prefix would otherwise have its own
diff classified under the prefix it just added. The base is the only side the
candidate did not write.

**D-2 (2026-09-11): two exported trees, not blobs.** The core already reads
trees; handing it two roots keeps it pure and lets it reuse the compiler, the
loaders and `owners_for_path` unchanged, rather than teaching each to read a git
object.

**D-3 (2026-09-11): a report, not a gate.** Recorded above and in note 02 G7.
A later spec may add a refusing mode once a design distinguishes a weakening
from a refactor; this report is the evidence such a design would need.

**D-4 (2026-09-11): conservative defaults.** Unmodeled keys are `requirement`
and parse failures are `unknown`, because a classifier that defaulted to the
harmless class would report exactly the changes it did not understand as the
ones that need no review.

## Verification

Each line runs in its own `sh -c` from the repository root (spec 049 3.5). The
scratch repository lives at a fixed path.

Against pre-088 code every line that calls `delta` fails, since the verb does
not exist; those lines are the evidence. The two `couple` lines pass before
and after: they record that the gate alone admits both candidates in 1, which
is the premise, and they would fail loudly if a later change made `couple`
refuse them, which would be worth knowing.

```verify:cli
cargo build --release --locked
rm -rf "${TMPDIR:-/tmp}/ss088" && mkdir -p "${TMPDIR:-/tmp}/ss088/specs/001-a" "${TMPDIR:-/tmp}/ss088/src"
printf -- '---\nid: "001-a"\ntitle: "a"\nstatus: approved\ncreated: "2026-09-11"\nimplementation: complete\nsummary: "s"\nestablishes:\n  - "src/a.rs"\n---\n\n# a\n\n## Verification\n\n```verify:cli\ntest -f src/a.rs\ngrep -q "fn a" src/a.rs\n```\n' > "${TMPDIR:-/tmp}/ss088/specs/001-a/spec.md"
printf 'pub fn a() {}\n' > "${TMPDIR:-/tmp}/ss088/src/a.rs"
D="${TMPDIR:-/tmp}/ss088"; S="$PWD/target/release/spec-spine"; cd "$D" && "$S" compile >/dev/null && "$S" index >/dev/null && git init -q -b main && git add -A && git -c user.email=t@example.invalid -c user.name=t commit -qm base
D="${TMPDIR:-/tmp}/ss088"; S="$PWD/target/release/spec-spine"; cd "$D" && git switch -qc weaken && sed -i.orig '/grep -q "fn a"/d' specs/001-a/spec.md && rm specs/001-a/spec.md.orig && printf 'pub fn b() {}\n' > src/a.rs && "$S" compile >/dev/null && "$S" index >/dev/null && git add -A && git -c user.email=t@example.invalid -c user.name=t commit -qm weaken && git switch -q main
D="${TMPDIR:-/tmp}/ss088"; S="$PWD/target/release/spec-spine"; cd "$D" && git switch -qc transfer && mkdir -p specs/002-b && printf -- '---\nid: "002-b"\ntitle: "b"\nstatus: draft\ncreated: "2026-09-11"\nimplementation: pending\nsummary: "s"\nextends:\n  - { spec: "001-a", unit: "src/a.rs", nature: additive }\n---\n\n# b\n' > specs/002-b/spec.md && printf 'pub fn a() { rewritten(); }\n' > src/a.rs && "$S" compile >/dev/null && "$S" index >/dev/null && git add -A && git -c user.email=t@example.invalid -c user.name=t commit -qm transfer && git switch -q main
D="${TMPDIR:-/tmp}/ss088"; S="$PWD/target/release/spec-spine"; cd "$D" && git switch -q weaken && "$S" couple --base main --head HEAD; R=$?; git switch -q main; test $R -eq 0
D="${TMPDIR:-/tmp}/ss088"; S="$PWD/target/release/spec-spine"; cd "$D" && git switch -q transfer && "$S" couple --base main --head HEAD; R=$?; git switch -q main; test $R -eq 0
D="${TMPDIR:-/tmp}/ss088"; S="$PWD/target/release/spec-spine"; cd "$D" && "$S" delta --base main --head weaken --json > "$D/w.json" && grep -q '"verification"' "$D/w.json" && grep -q '"headPlanHash"' "$D/w.json" && grep -q '"required": true' "$D/w.json"
D="${TMPDIR:-/tmp}/ss088"; S="$PWD/target/release/spec-spine"; cd "$D" && "$S" delta --base main --head transfer --json > "$D/t.json" && grep -q '"authority"' "$D/t.json" && grep -q '"002-b"' "$D/t.json" && grep -q '"required": true' "$D/t.json"
D="${TMPDIR:-/tmp}/ss088"; S="$PWD/target/release/spec-spine"; cd "$D" && git switch -qc policy && printf -- '[coupling]\nbypass_prefixes = ["src/"]\n' > spec-spine.toml && printf 'pub fn a() { unreviewed(); }\n' > src/a.rs && git add -A && git -c user.email=t@example.invalid -c user.name=t commit -qm policy && "$S" delta --base main --head policy --json > "$D/p.json"; R=$?; git switch -q main; test $R -eq 0 && grep -q '"policy"' "$D/p.json" && grep -q '"implementation"' "$D/p.json"
D="${TMPDIR:-/tmp}/ss088"; S="$PWD/target/release/spec-spine"; cd "$D" && "$S" delta --base main --head main --json > "$D/n.json" && grep -q '"required": false' "$D/n.json"
rm -rf "${TMPDIR:-/tmp}/ss088"
target/release/spec-spine check
```
