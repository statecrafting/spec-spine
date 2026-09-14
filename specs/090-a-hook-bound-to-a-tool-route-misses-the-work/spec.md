---
id: "090-a-hook-bound-to-a-tool-route-misses-the-work"
title: "A hook bound to a tool route misses the work"
status: approved
kind: "tooling"
created: "2026-09-13"
summary: >
  Every hook the kit ships is bound to a Claude Code tool name. The
  `PostToolUse` pair matches `Edit|Write` and reads `tool_input.file_path`;
  spec 046's one sanctioned write and the freshness read after a governance
  edit both hang off that matcher. A session steered to write through `Bash`
  produces the same bytes in the same file through a route that supplies
  `command` and no `file_path`, so neither fires, and nothing in the session
  says so. The steer is selected by the session's permission mode, which a
  repository cannot set and cannot observe, so no repository-side setting
  closes the gap. This spec adds the one boundary no tool route can bypass:
  a git `pre-commit` hook that refuses a commit which would land a stale or
  incompletely staged derived tree. It refuses and never repairs, because a
  pre-commit hook that regenerated and staged would make a stale committed
  tree indistinguishable from a fresh one, which is the defect spec 046 and
  the 017/021 drift both name. It is opt-in per clone through an enabler that
  follows spec 020's registration pattern, and inert until registered.
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "020-derived-artifact-merge-driver"
  - "046-kit-hooks-read-never-write"
  - "064-the-kit-ships-the-composite-gate"
  - "065-init-and-the-kit-are-one-adoption"
  - "075-one-name-one-freshness-verb"
extends:
  # 3.2, 3.4: the refusing hook and its per-clone enabler, in this
  # repository's own `.githooks/` (spec 020's directory).
  - { spec: "020-derived-artifact-merge-driver", unit: ".githooks/pre-commit", nature: additive }
  - { spec: "020-derived-artifact-merge-driver", unit: ".githooks/enable-hooks.sh", nature: additive }
  # 3.2, 3.4: the kit's copy, which is the source of both.
  - { spec: "064-the-kit-ships-the-composite-gate", unit: "kit/.githooks/pre-commit", nature: additive }
  - { spec: "064-the-kit-ships-the-composite-gate", unit: "kit/.githooks/enable-hooks.sh", nature: additive }
  # 3.5: the read-never-write property, asserted over a hook 046 did not have.
  - { spec: "046-kit-hooks-read-never-write", unit: "crates/spec-spine-core/tests/kit_hooks.rs", nature: additive }
  # 3.5: the kit/`.githooks/` parity list gains the two new names.
  - { spec: "064-the-kit-ships-the-composite-gate", unit: "crates/spec-spine-core/tests/kit_gate.rs", nature: additive }
  # 3.6: both new files are governance files, so they join the hashed set.
  - { spec: "064-the-kit-ships-the-composite-gate", unit: "spec-spine.toml", nature: additive }
  # 3.6: the embedded copy `init --with-kit` writes, regenerated.
  - { spec: "065-init-and-the-kit-are-one-adoption", unit: "crates/spec-spine-core/src/kit_embedded.rs", nature: corrective }
references:
  - { unit: { kind: file, path: "docs/design/03-adopter-audit-2026-09.md" }, role: context }
---

# 090: A hook bound to a tool route misses the work

## 1. Purpose

### 1.1 The matcher is a condition on the agent, not on the change

`kit/settings.json` registers two `PostToolUse` hooks under the matcher
`Edit|Write`. The body's first act is:

```sh
fp=$(jq -r '.tool_input.file_path // empty' 2>/dev/null)
[ -n "$fp" ] || exit 0
```

Everything spec 046 built rests on that line. The one sanctioned write (046
section 3.2), which recompiles after a `specs/*/spec.md` edit so `compile
--check` stays green in CI without a separate step, runs only when the edit
arrived through `Edit` or `Write`. The freshness read over the governance glob
list runs under the same condition.

A session that writes the same bytes into the same file with a shell heredoc,
`sed -i` or a generator script takes a route that supplies `command` and no
`file_path`. The matcher does not select it. Adding `Bash` to the matcher does
not help: the body exits at the line above, because a `Bash` payload has no
`file_path` to read. The hook is not weakened by the shell route; it is absent
from it.

### 1.2 The condition is set outside the repository

Claude Code emits a runtime instruction preferring shell operations for reads,
searches and file changes. It is selected by the session's permission mode. A
repository cannot select that mode: project and project-local settings cannot
activate `bypassPermissions`, and no setting a repository ships reports which
steer a session received. So the repository's guarantee is conditional on a
fact the repository can neither set, read, nor test.

This is the shape spec 046 section 1.1 already refused once, in a different
place: a kit that contradicted itself, where the prose described a guard the
implementation did not provide. Here the implementation provides the guard for
one tool route and the prose describes it as a property of the change.

### 1.3 What a missed hook costs

Nothing fails. `compile` is not run, so the committed registry drifts from the
corpus; the freshness read is not run, so the session is never told. The tree
looks tended. The defect surfaces at the next `check`, which in the ordinary
case is CI on a pull request that is already open, and in the bad case is a
later session that regenerates a large diff it did not cause and cannot
explain. Spec 031 was filed for exactly this failure reaching the default
branch once already.

### 1.4 The boundary that is not a tool

A commit is made by one command whatever wrote the bytes. `git commit` runs
`pre-commit` from `core.hooksPath` for a heredoc, an `Edit` call, a generator,
an IDE and a rebase alike. It is the earliest point in this repository's loop
that is a property of the change rather than of the actor.

## 2. Territory

This spec adds two files to this repository's `.githooks/` (spec 020's
directory) and the same two to `kit/.githooks/` (spec 064's), extends two test
files with the assertions that pin them, adds both names to
`[index] extra_hashed_inputs`, and regenerates the embedded kit. It changes no
hook body in `kit/settings.json` and retires nothing: the `PostToolUse` pair
keeps working on the route it matches, where it is the faster feedback.

## 3. Behavior

### 3.1 The hook refuses; it does not repair

`.githooks/pre-commit` MUST NOT run a `spec-spine` subcommand that writes into
the repository, and MUST NOT stage a file. Spec 046 section 1.2 states the rule
for a hook that cannot commit. This hook is the first in the kit whose actor is
the committer, so that argument does not reach it, and the rule holds anyway
for a different reason: a pre-commit hook that regenerated the derived tree and
added it to the index would make a commit that was never stale and a commit
that was stale until the hook fixed it into the same commit. The distinction is
the whole content of `check`, and spec 046's `SessionStart` case, the 017/021
drift, and `AGENTS.md`'s standing refusal to substitute `compile` for `check`
are the same argument in three other places.

The remedy the hook prints is `make refresh`, run by the committer, followed by
the commit again.

### 3.2 What it checks

In order, stopping at the first refusal:

1. **Resolve.** The repository is the one the commit is being made in
   (`git rev-parse --show-toplevel`). A tree with no `specs/` directory is not
   governed: exit 0 silently.
2. **The binary.** Resolved in spec 051's order: `$SPEC_SPINE_BIN`, then
   `./target/release/spec-spine`, then `PATH`. When none is found the hook
   prints that it skipped and why, and exits 0. A contributor who has not built
   the binary MUST NOT be unable to commit (spec 046 section 3.5).
3. **Freshness.** `spec-spine check`. Exit `0` passes. Exit `2` refuses and
   names the stale tree and `make refresh`. Exit `1` refuses and prints the
   validation output. Exit `3` refuses: the binary is present and has answered
   that it cannot perform the read, and an unanswered freshness question MUST
   NOT be absorbed silently into a durable act. Any other code refuses as
   unknown.
4. **Staged completeness.** A working tree that compiles clean still lands
   stale if the regenerated shards are not in the index. The hook refuses when
   `git diff --name-only -- <derived_dir>` or
   `git ls-files --others --exclude-standard -- <derived_dir>` is non-empty,
   and names the files. The derived directory comes from
   `spec-spine config show --json`, read as `layout.derived_dir`, never from a
   hardcoded `.derived` and never by parsing `spec-spine.toml` by hand.

The hook MUST NOT run `couple`. `couple` builds its diff from
`git diff base...head`, which is a range of commits and cannot contain the
change being committed, so a coupling verdict at this boundary would be a
verdict about the previous commit wearing this one's name.

### 3.3 The escape is the committer's

`git commit --no-verify` bypasses the hook. That is the standard instrument and
this spec does not shadow it with a second one. The hook's refusal message
names it, because a refusal that hides its own escape hatch produces a
contributor who disables the hook entirely.

### 3.4 Registration is per clone and opt-in

`.githooks/enable-hooks.sh` sets `core.hooksPath` to `.githooks` in this
clone's `.git/config`, is idempotent, and prints the one command that undoes
it. It follows `enable-merge-driver.sh` (spec 020) in shape and for the same
reason: the registration lives in per-clone config, which is not committed, so
each clone enables it once, and worktrees created off that clone inherit it.

Until it is registered, both files are inert bytes in the tree. This spec does
not enable the hook for anyone.

### 3.5 The tests

`tests/kit_hooks.rs` gains the assertion that `kit/.githooks/pre-commit` runs
no writing `spec-spine` verb and contains no `git add`, over the same
vocabulary spec 046 section 3.7 scans the settings hooks with.
`tests/kit_gate.rs`'s kit/repository parity list gains both names, so the two
copies cannot diverge silently.

### 3.6 The kit ships it

Both files live in `kit/.githooks/` as the source and are mirrored into this
repository's `.githooks/`. `.githooks/pre-commit` and `kit/.githooks/pre-commit`
join `[index] extra_hashed_inputs`: the existing `.githooks/*.sh` and
`kit/.githooks/*.sh` patterns do not match a file a git hook must name exactly.
`scripts/gen-kit-embedded.py` walks `kit/`, so regenerating carries both into
`init --with-kit`.

## 4. Out of scope

**Changing `kit/settings.json`.** The `PostToolUse` pair is correct on the
route it matches and is faster feedback than a commit. This spec adds a
boundary; it removes none.

**A staged-diff mode for `couple`.** `couple --head HEAD` reading
`base...head` means the documented pre-commit coupling check cannot see the
change being committed. That is a real defect in a different verb and is filed
separately; section 3.2 refuses to paper over it here.

**Enabling the hook in CI or for contributors.** Registration is a per-clone
act (3.4). CI already runs the whole gate and does not need it.

**A hook for agents without hooks.** Unchanged from spec 046 section 4: a git
hook is the answer for every agent, which is part of why this boundary was
chosen, but nothing here adds a `PreToolUse` equivalent for a harness that has
none.

## Verification

Each line is one command. Every line naming `pre-commit` or `enable-hooks.sh`
fails against pre-090 code, because neither file exists; those lines are the
fail-first evidence. The two `cargo test` lines are **not**: at the parent
commit the assertions they carry do not exist yet, so the suites pass
vacuously. They are listed to keep the assertions from being deleted later,
not as proof that anything changed.

```verify:cli
cargo test -p spec-spine-core --test kit_hooks --locked
cargo test -p spec-spine-core --test kit_gate --locked
# 3.1: the hook refuses and never repairs. No bare writing verb, no staging.
! grep -qE '\$sc" --repo "\$root" (compile|index)( |$)' kit/.githooks/pre-commit
! grep -qE '^[^#]*git add' kit/.githooks/pre-commit
# 3.2: the derived directory is a typed read, not a hardcoded path.
grep -qF 'config show --json' kit/.githooks/pre-commit
# 3.2: no coupling verdict at this boundary.
! grep -qE '^[^#]*couple' kit/.githooks/pre-commit
# 3.3: the refusal names its own escape.
grep -qF -- '--no-verify' kit/.githooks/pre-commit
# 3.4: the enabler sets the hooks path and is not the hook.
grep -qF 'core.hooksPath' kit/.githooks/enable-hooks.sh
# 3.6: both copies agree and both parse.
sh -c 'cmp -s kit/.githooks/pre-commit .githooks/pre-commit'
sh -c 'cmp -s kit/.githooks/enable-hooks.sh .githooks/enable-hooks.sh'
sh -n kit/.githooks/pre-commit
sh -n kit/.githooks/enable-hooks.sh
# 3.6: the embedded copy adopters receive carries both.
grep -qF '.githooks/pre-commit' crates/spec-spine-core/src/kit_embedded.rs
grep -qF '.githooks/enable-hooks.sh' crates/spec-spine-core/src/kit_embedded.rs
```
