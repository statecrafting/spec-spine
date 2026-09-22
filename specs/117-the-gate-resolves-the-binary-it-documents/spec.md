---
id: "117-the-gate-resolves-the-binary-it-documents"
title: "The gate resolves the binary it documents"
status: draft
kind: "tooling"
created: "2026-09-21"
implementation: complete
owner: "The spec-spine Authors"
depends_on:
  - "093-the-harness-this-repository-runs"
  - "094-one-gate-and-the-boundaries-it-holds"
summary: >
  The root Makefile's header documents a binary-resolution order preferring the
  binary this checkout builds, and its assignment implements none of it:
  `SPEC_SPINE ?= spec-spine` is the PATH fallback and nothing else, so `make
  gate` on a machine with an installed CLI governs with whatever that CLI is.
  Measured here at three releases behind the checkout. The assignment is
  corrected to the order the header states, an unusable explicit override is
  refused rather than silently replaced, and the resolved binary and its version
  are announced before the chain runs.
establishes:
  - { kind: file, path: "crates/spec-spine-core/tests/gate_binary.rs" }
extends:
  - spec: "094-one-gate-and-the-boundaries-it-holds"
    unit: { kind: file, path: "Makefile" }
    nature: additive
references:
  - unit: { kind: file, path: ".githooks/pre-commit" }
    role: "context"
---

# 117: The gate resolves the binary it documents

## 1. Purpose

`make gate` is the executable form of the governed loop (spec 094 §3.1). What
it is not, on any machine with `spec-spine` installed, is a gate run by the
binary this checkout builds.

The header of the root `Makefile` says, in the variable table it offers a
reader:

```
#   SPEC_SPINE  the binary to govern with. A repository that builds its own
#               must point at the one it builds, which is the resolution order
#               spec 093 established: $SPEC_SPINE, then ./target/release, then
#               PATH. Set it once here rather than in every caller.
```

Thirty lines later it assigns:

```
SPEC_SPINE ?= spec-spine
```

`?=` honours an explicit override, so the first step of the documented order is
real. The second does not exist. `spec-spine` is the third step written as if it
were the whole answer, and on a machine with an installed CLI it resolves there
every time.

### 1.1 What it cost, measured

Measured on this machine, 2026-09-21, on the release candidate branch:

```
$ command -v spec-spine && spec-spine --version
/Users/…/.cargo/bin/spec-spine
spec-spine 0.20.0
$ ./target/release/spec-spine --version
spec-spine 0.22.0
```

Three releases behind the checkout, and `make gate` selected it.

It was not caught by a wrong verdict. It was caught while building a **draft
that is not in this corpus** (filed as ordinal 116 on the branch
`specs/deferred-contracts-2026-09-21`, and deliberately not part of this
release): its lint exemption, already built, still reported ten `L-001`
warnings through `make` and none through `./target/release/spec-spine`. That is
the *mild* shape of the failure: two binaries disagreeing loudly enough that
someone looked. The shape that does not announce itself is a gate that agrees
with the corpus by accident and would not have, one commit later.

The ordinal is named only so the trail is followable. Nothing in this spec
depends on that draft, and `registry list` will not resolve 116 until and
unless it is merged: see `docs/corpus-map.md` for how an unresolvable ordinal
is read here.

`[meta] required_version = ">=0.17.0"` does not close this. It is a floor and
was chosen as one on purpose (spec 061 §3.8): 0.20.0 clears it. A binary that
is admitted is not a binary that agrees with the corpus, and finding *a* binary
is not evidence that it can judge *this* one.

### 1.2 Why this is its own spec

The `Makefile` is spec 094's territory. The defect was found during that
draft's build and deliberately not fixed there: correcting another spec's file
under cover of an unrelated build is the drift this corpus exists to refuse. It is
also not spec 093's, which governs the **hooks** (§3.12) and is correct about
them; the Makefile borrowed its sentence without borrowing its behavior.

## 2. Territory

This spec establishes `crates/spec-spine-core/tests/gate_binary.rs`, the
selection test, and extends spec 094 at `Makefile`, the file whose assignment
is wrong.

It changes no verb, no library function, no schema and no exit-code mapping.
Nothing outside the `Makefile` and the new test changes.

## 3. Behavior

### 3.1 The documented order is the implemented order

The `Makefile` MUST select the governing binary in this order, taking the first
that applies:

1. **An explicit `SPEC_SPINE`**, from the command line or the environment. It
   is used as given and is never replaced (§3.2).
2. **`./target/release/spec-spine`**, when that path exists and is an
   executable regular file. A repository that builds its own binary is governed
   by the one it builds.
3. **`spec-spine` on `PATH`**, the documented fallback, which keeps an
   installed CLI working on a checkout that has not been built.

Step 2 MUST test for an executable regular file, not merely for a name: a
directory answers `test -x` and is not a binary.

The selection MUST be made once, at parse time, and MUST be the same value
every recipe in the file uses. A gate whose steps could resolve differently
from one another is a worse answer than the one being replaced.

### 3.2 An unusable explicit override is refused, never substituted

When `SPEC_SPINE` is set explicitly and names nothing executable, the
`Makefile` MUST refuse, with a message naming the value it was given. It MUST
NOT fall back to `./target/release/spec-spine` or to `PATH`.

The recipe exits 3, which is this repository's usage/config code, and `make`
reports the recipe's failure as its own exit 2. That is the shape every other
refusal in this file already has (`OWNERSHIP=yes`, `COUPLE=maybe`, spec 094
§3.3 and §3.4), and spec 094's own acceptance asserts those with `! make ...`
rather than on a number for the same reason. No claim is made here that `make`
exits 3.

An empty explicit `SPEC_SPINE=` is the same refusal. Left to expand, it would
drop the binary from the command line entirely and run the verb name as a
command, which is a different failure wearing the wrong exit code.

This is the rule the `.githooks/pre-commit` resolver does **not** follow: it
tests `[ -x "${SPEC_SPINE_BIN}" ]` and, failing that, falls through to the next
candidate. That is defensible for an advisory hook that must exit 0 whatever
happens (spec 093 §3.10). It is not defensible for the gate, where the whole
question is which binary returned the verdict. The divergence is deliberate and
is recorded here so that nobody reconciles them by making the gate quieter.

A resolution that finds nothing at all is the same refusal, naming both places
that were looked in. No step of the chain runs.

### 3.3 The gate announces the binary it resolved, and its version

Before the first verb runs, `gate`, `refresh` and `verify` MUST print one line
naming the resolved binary, where it came from, and the output of its
`--version`.

Announcement is the whole remedy for §1.1's silent shape. A correct resolution
order still leaves a session free to build nothing and be governed by an old
install; what it must not do is leave that invisible. The `PATH` case MUST say
that this repository builds its own binary and that an installed CLI may
predate the corpus.

The announcement is not a version check and MUST NOT be written as one. The
floor is `[meta] required_version`, it is enforced by the binary on every verb,
and this spec neither moves it nor duplicates it.

### 3.4 What does not change

`gate` stays read-only (spec 094 §3.1). Resolving the binary MUST NOT build it:
a gate that builds the tool it judges with has repaired its own premise. The
`OWNERSHIP`, `COUPLE`, `PR_BODY`, `BASE` and `HEAD` controls, the manifest
guards, and every refusal spec 094 §3.2 to §3.5 requires keep the behavior they
have.

### 3.5 The selection is tested by executing distinguishable binaries

`crates/spec-spine-core/tests/gate_binary.rs` MUST exercise the real root
`Makefile` -- copied, not paraphrased -- against stub binaries that report
distinct identities and distinct versions, and MUST assert on **which stub
ran**, not only on which path a variable held. A test that reads the variable
asserts the assignment; only a test that runs the binary asserts the selection.

The cases it MUST cover:

| case | tree | `SPEC_SPINE` | required outcome |
|---|---|---|---|
| explicit override | in-tree present, PATH present | an override stub | the override stub runs |
| in-tree available | in-tree present, PATH present | unset | the in-tree stub runs |
| in-tree missing | in-tree absent, PATH present | unset | the PATH stub runs |
| older PATH binary | in-tree present, PATH stub reports an older version | unset | the in-tree stub runs, and the announced version is the in-tree one |
| invalid override | in-tree present, PATH present | a path that is not executable | refused; the message names the value, and neither the in-tree nor the PATH stub runs |
| override naming a directory | in-tree present, PATH present | an existing directory | refused as not executable, on every shell; neither stub runs |
| empty override | in-tree present, PATH present | `` (empty) | refused |
| nothing anywhere | in-tree absent, PATH absent | unset | refused, naming both places looked in |

The directory row is the one that pins the mechanism rather than a fixture: a
directory carries the executable bit and is not a program, so a guard built on
`command -v` admits it wherever `command -v` answers for paths at all. See D-9.

The "invalid override", "override naming a directory" and "empty override"
rows MUST assert the **negative**:
that no other stub ran. An exit 3 alone is compatible with the gate having run
the wrong binary and that binary having failed.

## 4. Out of scope

- **Changing the `.githooks/pre-commit` resolver** to refuse an unusable
  `SPEC_SPINE_BIN`. §3.2 records why the two differ; changing the hook is spec
  093's territory and needs its own evidence about advisory exits.
- **Raising `[meta] required_version`.** §3.3. The floor's one-directional
  shape is spec 061 §3.8's decision and is unaffected.
- **Making `gate` build the binary.** §3.4.
- **Any change to the verbs, the chain, or what CI calls.** `.github/workflows/ci.yml`
  calls `make gate` and keeps calling it unchanged.
- **The release candidate.** This spec is built on its own branch and is
  integrated, if at all, by an explicit merge. It does not depend on the
  version bump and the version bump does not depend on it.

## 5. Resolved decisions

**D-1 (2026-09-21, file: the fix is a spec, not a one-line edit).** The
assignment is one line and the corpus's rule about whose file it is does not
have a size exemption. Spec 094 owns `Makefile`; an `extends` edge naming that
unit is the sanctioned door (spec 037) and needs no edit to 094's approved
prose, which has nothing to say about binary resolution and is not made wrong
by this.

**D-2 (2026-09-21, file: refuse an unusable override rather than fall
through).** The alternative is the hook's behavior, and it is the more
forgiving one. It was rejected because the two callers are asking different
questions. The hook asks "can I advise?", and the honest answer when a named
binary is missing is to advise with another one. The gate asks "is this tree
governed?", and a gate that silently answers with a binary the caller did not
name has substituted the subject of the question. §3.2.

**D-3 (2026-09-21, build: the announcement goes to stderr).** `make verify
SPEC=<id>` hands its stdout to the verb, and spec 090 made that channel the
verdict's alone. A `make`-level line on stdout would put prose in front of a
consumer reading a verdict, which is the defect spec 090 removed from the verb
and there is no reason to reintroduce one layer out. stderr is where an
announcement about the run belongs, and the tests read it there.

**D-4 (2026-09-21, build: eight cases, and all eight failed first; a ninth was
added later by D-9).** Run
against the `Makefile` this spec is filed on, every case in §3.5 fails. They do
not all fail for the same reason, and the difference is worth recording:

| case | why it failed on the old `Makefile` |
|---|---|
| explicit override | selection was already right (`?=`); it failed on the announcement |
| in-tree available | **selection**: ran `PATH`, expected in-tree |
| in-tree missing | selection was already right; it failed on the announcement |
| older PATH binary | **selection**: the whole chain ran through the 0.20.0 stub |
| invalid override | **refusal**: no preflight existed, so `make` reached the verb |
| empty override | **refusal**: same |
| nothing anywhere | **refusal**: same |
| does not build | the target did not exist |

Two rows would have passed on selection alone. They are kept because the
announcement is §3.3's remedy for the silent shape, and an untested
announcement is the thing that was wrong with the header in the first place.

**D-5 (2026-09-21, file: announce, do not check).** A tempting second remedy is
for the `Makefile` to compare the resolved binary's version against something.
Rejected: there is exactly one declared floor, `[meta] required_version`, the
binary enforces it on every verb, and a second comparison in `make` would be a
copy that can drift from it -- the failure mode spec 094 exists to remove from
this file. Announcement gives a reader the fact without creating a second
authority. §3.3.

**D-6 (2026-09-21, build: the announcement goes through a shell variable).**
`tests/gate.rs` reads `$(SPEC_SPINE) <word>` in this file as an invocation and
checks `<word>` against the verb list; it is the detector that stops a second
chain growing here, and it caught the first draft of the preflight, whose
messages said `spec-spine binary` and `governing with $(SPEC_SPINE) [`. Both
are prose, neither is an invocation, and the honest fix was to stop feeding the
detector false positives rather than to teach it to ignore a line. The resolved
path is copied into a shell variable once and the messages interpolate that.

**D-7 (2026-09-21, build: the read-only assertion reads the target, not the
file).** The acceptance block first asserted no writing verb anywhere in the
`Makefile`. It failed, correctly: `refresh` is the writing half and runs
`compile` and `index` by design (spec 094 §3.1). A file-wide grep for a writing
verb refuses the correct file. The assertion now runs spec 094's own detector,
which reads the `gate` target.

**D-8 (2026-09-22, review: the value crosses into the shell through the
environment).** Raised on the pull request. The recipe opened with
`b='$(SPEC_SPINE)'`, so a path containing a single quote closes the literal and
the caller meets a `/bin/sh` syntax error instead of this target's own refusal.
Double-quoting only trades that for `$` expansion. A target-specific
`export SPEC_SPINE_VALUE = $(SPEC_SPINE)` hands the bytes to the recipe
verbatim, where neither character is special. It costs one line and removes the
class rather than one member of it.

Extended on review to `SPEC_SPINE_ORIGIN`, which the same recipe expanded
inline. It is an internal variable rather than a documented input, but a
command-line assignment overrides a `:=` one in GNU make, so it is reachable
by the same route; eliminating half a class and leaving the other half is
worse than not having looked.

**D-9 (2026-09-22, CI: usability is `test -x`, not `command -v`, for a value
naming a path).** Found by this spec's own acceptance failing on the Linux
runner while passing locally. `command -v` does not agree across shells on a
value containing a slash: bash checks the executable bit, dash returns the
string unchanged. A non-executable absolute path therefore cleared the guard
under dash and refused one step later with "the resolved binary does not run
here" rather than §3.2's "names nothing executable" -- the right verdict with
the wrong reason, which for a spec whose subject is *which binary answered* is
the wrong verdict. The guard now uses `test -f && test -x` for a value
containing a slash and `command -v` only for a bare name, so the refusal a
caller sees does not depend on the runner's `/bin/sh`. A directory-valued
override is added to the table as the row that pins the mechanism: it carries
the executable bit and is not a program, so it is the case a `command -v` guard
admits on any shell that answers for paths at all.

**D-10 (2026-09-22, review: a resolved path containing a space is a known
limitation, and quoting the invocation is not the fix).** Raised on the pull
request: the preflight validates `"$b"`, which survives a space, while each
chain line invokes `$(SPEC_SPINE)` unquoted, which the shell word-splits. So a
path with a space clears the preflight and then fails as "command not found",
which is the false confidence §3.2 is otherwise written against.

Quoting the invocations is refused, and the reason is specific to this file.
`tests/gate.rs` recognises an invocation by finding `spec-spine ` -- the name
followed by a space -- after expanding `$(SPEC_SPINE)`. Writing
`"$(SPEC_SPINE)" check` produces `"spec-spine" check`, which contains no such
substring, so the detector that keeps this `Makefile` from growing a second,
unreviewed chain of verbs would go blind to **every** line it guards. Trading
spec 094's structural check for an edge case in path naming is a bad trade.

The limitation is therefore recorded rather than fixed: a repository whose
checkout path contains a space cannot be governed through `make` until the
detector is taught to read a quoted invocation, which is spec 094's file and
spec 094's change. The preflight's refusals are unaffected; what it cannot do
is promise that a path it accepted will survive word splitting.

## Verification

Behavioral, and written to fail against the tree this spec is filed on.

On that tree `crates/spec-spine-core/tests/gate_binary.rs` does not exist, so
the test command fails to compile; and the `Makefile` still carries
`SPEC_SPINE ?= spec-spine`, so the in-tree, older-PATH and invalid-override
rows of §3.5 would fail if it did.

```verify:cli
# 3.5: the selection test exists and is the real Makefile's test.
test -f crates/spec-spine-core/tests/gate_binary.rs
# 3.1 to 3.3, 3.5: run it. Every row of the table is one test case, and each
# asserts which stub binary actually ran.
cargo test -p spec-spine-core --test gate_binary --locked
# 3.1: the corrected assignment is conditional on the override, not a bare
# PATH default. The old line is gone.
! grep -qE '^SPEC_SPINE[[:space:]]*\?=[[:space:]]*spec-spine[[:space:]]*$' Makefile
# 3.4: the gate is still read-only. Asserted by spec 094's own detector,
# which reads the `gate` target rather than the whole file: `refresh` is the
# writing half and legitimately runs `compile` and `index`, so a file-wide
# grep for a writing verb refuses the correct Makefile (measured: it did).
cargo test -p spec-spine-core --test gate the_gate_target_is_read_only --locked
# The governed loop over the corpus this spec is part of.
./target/release/spec-spine check --fail-on-unresolved --fail-on-warn
./target/release/spec-spine lint --fail-on-warn
```
