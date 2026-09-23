---
id: "123-a-startup-verdict-names-its-reader"
title: "A startup verdict names its reader"
status: draft
kind: "tooling"
created: "2026-09-23"
summary: >
  Three session starts on 2026-09-23 reported "spec registry: INVALID;
  codebase index: STALE" on a corpus that was valid and fresh. The
  reader was the repository's own `target/release/spec-spine`, built from an
  earlier revision and left in place while the checkout fast-forwarded past
  the frontmatter grammar it knows, so an incompatible reader's refusal was
  printed as a finding about the corpus. The hooks that turn `check` into a
  message now name the executable they selected, and when that executable is
  the in-tree build and is older than the source it is built from, they say
  the verdict is an earlier revision's reading and keep what it said. They
  still read only: nothing builds, regenerates or installs.
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "093-the-harness-this-repository-runs"
  - "094-one-gate-and-the-boundaries-it-holds"
# 3.7: one wording rule of 093 §3.9 and §3.10, for the older-reader case only.
amends: ["093-the-harness-this-repository-runs"]
extends:
  # 3.1 to 3.3: the three hooks that report `check`'s verdict.
  - { spec: "093-the-harness-this-repository-runs", unit: { kind: file, path: ".claude/settings.json" }, nature: additive }
  # 3.6: the regressions run the shipped bodies.
  - { spec: "093-the-harness-this-repository-runs", unit: { kind: file, path: "crates/spec-spine-core/tests/harness_hooks.rs" }, nature: additive }
  # 3.4: the commit boundary's freshness refusal.
  - { spec: "094-one-gate-and-the-boundaries-it-holds", unit: { kind: directory, path: ".githooks/" }, nature: additive }
establishes:
  # 3.6: the commit boundary run for real, in a throwaway repository.
  - { kind: file, path: "crates/spec-spine-core/tests/reader_identity.rs" }
references:
  - { unit: { kind: file, path: "docs/release-candidate-0.22.0.md" }, role: context }
---

# 123: A startup verdict names its reader

## 1. Purpose

### 1.1 What happened, measured

The `SessionStart` hook in `.claude/settings.json` printed this on three
session starts, 2026-09-23 01:51Z (a resume), 02:12Z and 04:21Z, each with
`cwd` and `CLAUDE_PROJECT_DIR` set to this repository's main checkout (read
from the hooks' recorded `stdout`, not their command text, which contains the
same words):

```
[session-freshness] spec registry: INVALID, the corpus fails validation (run spec-spine compile for the violations); codebase index: STALE, run spec-spine index
```

Between 02:09Z and 04:11Z the `Stop` hook, which also reads
`CLAUDE_PROJECT_DIR`, printed its `INVALID` line sixteen times. The checkout
was `6e123d2e` at the first banner and `0ca3001d` at the other two. Each tree
was valid and both committed trees were fresh: read by a binary built from it,
each answers `fresh` twice and exits 0.

The session that saw the third warning attributed it to the `spec-spine` on
`PATH` (`~/.cargo/bin`, then `0.20.0`) and replaced that binary. The
attribution was wrong, and the resolution order says why. The hook resolves
its reader as `$SPEC_SPINE_BIN`, then the repository's own
`target/release/spec-spine`, then `PATH` (spec 093 §3.5). At the time:

| Rule | State | Selected |
|---|---|---|
| `$SPEC_SPINE_BIN` | unset (no shell profile, `launchctl` or settings `env` sets it) | no |
| `target/release/spec-spine` | present and executable; it answered `spec-spine 0.22.0` five seconds after the warning, before anything rebuilt it | **yes** |
| `PATH` | `~/.cargo/bin/spec-spine`, `0.20.0` | never consulted |

The hook was the repository's own registration: the session transcript
records it as `SessionStart:startup` (and `SessionStart:resume` for the
first) with the command body from
`.claude/settings.json`. No managed settings file exists on the machine,
`~/.claude/settings.json` registers no `SessionStart` hook, and the untracked
`.codex/hooks.json` in the checkout is a Codex registration that Claude Code
does not load.

The in-tree build was the one last made in the main checkout, at 2026-09-22
04:06 local time (10:06Z) at `3d4f3902`, whose `crates/`, `Cargo.toml` and
`Cargo.lock` are identical to the frozen 0.22.0 candidate's (`f9fa6a8f`); no
build ran in that checkout again until after the last banner. The checkout was
fast-forwarded without a rebuild to `03a204b9`, `45becbbe` (which brought spec
106), `6e123d2e`, `b7c13452` and `0ca3001d`. The frontmatter of specs 106, 107
and 109 declares members that the candidate's grammar reads as malformed extra
frontmatter:

```
V-002 [specs/106-obligations-are-declared-constraints/spec.md] malformed frontmatter: extra-frontmatter lists must contain only strings
V-002 [specs/107-a-context-closure-is-declared/spec.md] malformed frontmatter: extra-frontmatter lists must contain only strings
V-002 [specs/109-impact-and-conflict-are-declared/spec.md] malformed frontmatter: extra-frontmatter lists must contain only strings
```

Reproduced with real binaries: the candidate's build, placed as
`target/release/spec-spine` in a worktree at `0ca3001d` and dated before it,
makes the unchanged hook print the warning above byte for byte; a build from
`0ca3001d` in the same position makes it print `fresh` twice. The control ran
too: a session start at 2026-09-22 22:18Z, while the checkout was still at
`3d4f3902`, reported `fresh` from the same build, because the reader and the
tree matched.

### 1.2 Why the hook could not tell

`check` exits 1 for a corpus that fails validation, and a reader whose grammar
predates the corpus fails validation too. From the exit code and the report
the two are the same answer. The difference is in the reader, and the hook
never said which reader it had asked.

A version comparison would not have separated them either. The build and the
current source both answer `0.22.0`, which a separate change to the package
identity addresses, and within one version every development build
answers the same string. What does separate them, for the one reader that has
a source to compare against, is age: a build older than any file it is built
from was built from a different revision.

## 2. Territory

Extends spec 093's `.claude/settings.json` and its hook test, and spec 094's
`.githooks/`. Establishes one test file.

## 3. Behavior

### 3.1 The reader is named

Every message the `SessionStart`, `Stop` and `PreToolUse` (PR gate) hooks
print from a `check` verdict other than a clean pass MUST name the executable
that produced it by the path the resolver selected, and by what it answers to
`--version` wherever spec 093 §3.9 lets a hook ask. It does not let one ask on
exit 0 or exit 2, which are answers, so a stale verdict names its reader by
path alone; the age comparison of §3.2 spawns nothing and is asked there too.
`SessionStart` names the reader on a clean pass as well, because the banner is
the one place a session learns which reader it is running.

### 3.2 A reader older than its source is not believed

When the selected reader is the repository's own `target/release/spec-spine`
and any file under `crates/`, or `Cargo.toml` or `Cargo.lock`, is newer than
it, the hooks MUST NOT report its verdict as the tree's:

- `SessionStart` prints `NOT JUDGED`, names the reader and the first newer
  source path, and gives the rebuild command.
- `Stop` prints the same qualification before the reader's report.
- The PR gate still refuses (a check that did not pass is not green, spec 093
  §3.1), names the reader, and replaces "fix the violations it names" with the
  rebuild, because the violations may be an older grammar's.

When the reader is current, an `INVALID` verdict is reported as a finding
about the corpus, as before, now with the reader beside it.

A reader selected by `$SPEC_SPINE_BIN` or from `PATH` has no source in this
repository to be compared against. It is named and never called older.

### 3.3 The original diagnostic is kept

The reader's own words survive every qualification: `SessionStart` quotes
both halves it would have printed, `Stop` prints its unchanged lines after the
qualification, and the commit hook prints `check`'s output before refusing.
The qualification is added evidence, never a substitute.

### 3.4 The commit boundary names its reader

`.githooks/pre-commit`'s freshness refusals name the reader the same way, and
a reader older than its source is refused as that, with the rebuild as the
remedy. This is the refusal the 0.22.0 record §14.8 describes, where a fresh
worktree with no build fell back to `PATH` `0.20.0`: the refusal stands, and
now says which binary made it.

### 3.5 Read-only

Nothing here builds, regenerates, installs or edits configuration. The age
comparison reads file modification times. The rebuild is named in a message
and never run: a hook that built would be a hook that writes, in the middle of
whatever the session is doing (spec 093). No global setting and no installed
binary is changed to make a warning go away.

### 3.6 Evidence

Behavioral, through the shipped bodies:

- `harness_hooks.rs` runs each hook from `.claude/settings.json` in a
  repository whose in-tree build and a different `PATH` binary both exist,
  with `$SPEC_SPINE_BIN` unset, which is the resolution that selected the
  reader in §1.1. The build's report is the one captured from the candidate's
  binary. Cases: an older build (`NOT JUDGED`, the in-tree path named, the
  `PATH` binary not named, both original halves kept); a current build (the
  `INVALID` finding, named); no build (the `PATH` binary named, not aged); a
  clean pass (named); `Stop` and the PR gate, older and current.
- `reader_identity.rs` runs the real `.githooks/pre-commit` through `git
  commit`: a `PATH` reader's refusal names it; an older in-tree build is
  refused as older, with its report kept.
- Both files assert that every hook line naming `cargo build` is a message.

### 3.7 What this amends in spec 093

Spec 093 §3.9 says that when the probe succeeds on exit 2 "the message is
unchanged, wording included", and §3.10 fixes the `SessionStart` banner's
verdict wording. Both are amended for one case only, a reader that is the
in-tree build and older than its source:

- the PR gate's stale refusal keeps its first line and replaces the
  `compile` / `index` remedy with the rebuild, because regenerating with an
  earlier revision's reader would commit that revision's shards;
- the `SessionStart` banner reports `NOT JUDGED` and quotes the verdict it
  would have printed, instead of printing it as the tree's.

For a current reader the stale wording is unchanged: the PR gate appends one
line naming the reader's path, and `Stop` prints one before its unchanged
report. `--version` stays off exit 0 and exit 2 in every hook, as §3.9
requires. Nothing else in 093 moves, and its acceptance runs unchanged.

## 4. Out of scope

- **Which reader is selected.** Spec 093's order is unchanged. The defect was
  that the selected reader was not named, not that it was selected.
- **A floor for released readers.** A published binary older than the
  corpus's grammar is refused by `[meta] required_version`, which needs a
  version that distinguishes the readers; that belongs to the change that
  gives the expansion line its own package version.
- **Embedding a source revision in the binary.** §5 D-1.
- **The `PostToolUse` hook.** It prints `check`'s own lines after an edit and
  interprets no verdict.

## 5. Resolved decisions

**D-1 (2026-09-23, age rather than an embedded revision).** A build-time
source revision would need a build script that runs `git`, answers nothing
from a packaged crate (which has no repository), and makes the binary depend
on the commit it was built at. Modification time answers the only question
the hook has, whether this build could be the build of this tree, from what
is on disk, and errs toward "not judged" when a source file was touched after
a build that it did not affect. That error costs a rebuild; the error it
replaces cost a session that believed a valid corpus was invalid.

**D-2 (2026-09-23, build: fail-first measured).** Against the hooks at
`3b67b63d`, all six new `harness_hooks.rs` cases and both behavioral
`reader_identity.rs` cases fail; with this build all pass. The commit-hook
cases are in their own file because spec 122's acceptance pins
`commit_boundary.rs`'s case count.

**D-3 (2026-09-23, review of #317).** The first build named the reader in the
PR gate's invalid arm only, leaving its stale arm and catch-all, and `Stop`'s
stale, unresolved and catch-all reports, unnamed, which §3.1 does not allow.
Every arm now names it. Doing so on exit 2 met spec 093 §3.9, which keeps
`--version` off an answered exit and the stale wording unchanged: the reader
is named there by path, `--version` is not asked, and the one wording change
(the older reader's remedy) is declared as an amendment (§3.7) rather than
made quietly. 093's own pin, that the stale arm asks no `--version`, passes.

**D-4 (2026-09-23, second review of #317).** The commit hook asked
`--version` for every refusal, exit 2 included, where the other hooks keep it
off an answered exit (spec 093 §3.9). It names a stale verdict's reader by path
now, as they do, and `reader_identity.rs` asserts it.

## Verification

Written to fail against the tree this spec is filed on: neither the helper
nor the test file exists.

```verify:cli
# 3.2: the one comparison, in every hook that reports a verdict.
test "$(grep -o 'spec_spine_reader_predates()' .claude/settings.json | wc -l | tr -d ' ')" = 3
grep -qF 'spec_spine_reader_predates()' .githooks/pre-commit
# 3.1 to 3.3, 3.5: behavioral, through the shipped bodies.
cargo test -p spec-spine-core --test harness_hooks --locked spec123_
cargo test -p spec-spine-core --test reader_identity --locked
# 3.5: 093's read-only contract still holds for every hook.
cargo test -p spec-spine-core --test harness_hooks --locked hooks_read_and_never_write
```
