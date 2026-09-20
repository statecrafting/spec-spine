---
id: "115-the-kit-ships-no-claim-an-adopter-cannot-resolve"
title: "The kit ships no claim an adopter cannot resolve"
status: draft
kind: "tooling"
created: "2026-09-17"
summary: >
  Three of the shell scripts `init --with-kit` writes carry a `# Spec:` claim
  header naming a spec id that exists in this repository's corpus and in no
  adopter's. Here the headers are inert, because `.githooks/` falls outside every
  discovered package and the scanner never reads them, which is what spec 097
  measured. In an adopter whose package is the repository root they are read,
  resolve to nothing, and are reported as `unknown-spec` near misses; under
  `[coupling] require_ownership` the same three files are unclaimed sources. The
  sharp end is shadowing: the scanner takes the first claim attempt in the window
  whether it resolves or not, so an adopter who adds their own valid header below
  the kit's gets a file that still reads as unowned. This spec makes the kit ship
  provenance rather than a claim.
implementation: in-progress
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "032-ownership-coverage"
  - "064-the-kit-ships-the-composite-gate"
  - "065-init-and-the-kit-are-one-adoption"
  - "094-a-claim-below-the-header-window-is-not-silent"
  - "097-governed-scope-is-declared-not-inferred"
  # 3.1's acceptance runs the composite gate on a scratch adopter that has a
  # package, which is red for 114's reason until 114 is built. A record
  # dependency on the gate this spec's measurement runs, not on a behavior.
  - "114-one-gate-definition-that-holds-on-a-code-free-corpus"
extends:
  - { spec: "064-the-kit-ships-the-composite-gate", unit: "kit/.githooks/enable-hooks.sh", nature: additive }
  - { spec: "064-the-kit-ships-the-composite-gate", unit: "kit/.githooks/enable-merge-driver.sh", nature: additive }
  - { spec: "064-the-kit-ships-the-composite-gate", unit: "kit/.githooks/merge-derived-index.sh", nature: additive }
  # Spec 064 3.3 asserts the two trees are equal, so the edit lands in both.
  - { spec: "020-derived-artifact-merge-driver", paths: [".githooks/enable-hooks.sh", ".githooks/enable-merge-driver.sh", ".githooks/merge-derived-index.sh"] }
  - { spec: "064-the-kit-ships-the-composite-gate", unit: "crates/spec-spine-core/tests/kit_gate.rs", nature: additive }
  - { spec: "065-init-and-the-kit-are-one-adoption", unit: "crates/spec-spine-core/src/kit_embedded.rs", nature: additive }
references:
  - { unit: { kind: file, path: "docs/design/05-remaining-waves-2026-09.md" }, role: context }
---

# 115: The kit ships no claim an adopter cannot resolve

## 1. Purpose

### 1.1 What a fresh adoption is handed

Measured on 2026-09-17 against `7a7a8b6` with the 0.20.0 binary. A scratch
directory, `git init`, `spec-spine init --with-kit`, one crate at the root
(`Cargo.toml` plus `src/main.rs`), `require_ownership = true`, then the
ownership read:

```
$ spec-spine index coverage --fail-on-untraced
coverage: 0/4 source files specifically claimed (0.0%); 0 floor-only, 4 unclaimed
  near-miss comment headers (claimed nothing): 3
    .githooks/enable-hooks.sh:2 unknown-spec
    .githooks/enable-merge-driver.sh:2 unknown-spec
    .githooks/merge-derived-index.sh:2 unknown-spec
  . (no floor): 0/4 claimed, 0 floor-only, 4 unclaimed

unclaimed (no owning spec):
  .githooks/enable-hooks.sh
  .githooks/enable-merge-driver.sh
  .githooks/merge-derived-index.sh
  src/main.rs
$ echo $?
1
```

`src/main.rs` is the adopter's own file and belongs on that list. The other
three are files the tool wrote sixty seconds earlier, and the headers they carry
are:

```
.githooks/enable-hooks.sh        # Spec: specs/090-a-hook-bound-to-a-tool-route-misses-the-work/spec.md
.githooks/enable-merge-driver.sh # Spec: 020-derived-artifact-merge-driver
.githooks/merge-derived-index.sh # Spec: 020-derived-artifact-merge-driver
```

Specs 020 and 090 are this repository's. They resolve here and nowhere else.

### 1.2 Why nobody saw it, stated exactly

Spec 097 §1.2 measured these same headers and recorded them as **inert**: seven
files in this repository carry a valid `// Spec:` header that nothing reads,
three of them the `.githooks/` scripts and three more their `kit/` originals.
That measurement is correct and this spec does not revise it. What makes them
inert here is the scanner's scope: `index.rs::scan_comment_headers` is handed
`&discovered.packages` and walks source files inside them. This repository's
packages are `crates/spec-spine-cli`, `crates/spec-spine-core`,
`crates/spec-spine-types` and `npm`, so `.githooks/` and `kit/` fall outside
every one of them and the headers are never read.

An adopter with a single crate at the repository root has one package, `.`, and
it contains everything. The same bytes that are unreadable here are read there.
The producer's corpus is the one shape in which this defect cannot appear.

The fourth hook is invisible for a different reason. `.githooks/pre-commit`
carries the same header and has no extension, so it is not in
`coverage.rs::SOURCE_EXTS` and is neither scanned nor counted. Three files are
reported, not four.

### 1.3 The shadowing is the part that is not cosmetic

Spec 094 §3.2: the first claim attempt inside the window decides, resolving or
not, so an unresolvable attempt shadows a valid header below it. Reproduced in
the same scratch corpus, with the adopter's own claim added on line 3:

```
#!/usr/bin/env bash
# Spec: specs/090-a-hook-bound-to-a-tool-route-misses-the-work/spec.md
# Spec: specs/000-bootstrap/spec.md
```

```
$ spec-spine index && spec-spine index owner .githooks/enable-hooks.sh
.githooks/enable-hooks.sh
  (no spec owns this path)
```

The adopter did the documented thing, in the documented place, and the kit's
line stops it working. A diagnostic an adopter can ignore is noise; a shipped
byte that defeats the documented remedy for that noise is a defect.

### 1.4 What the header was for

Provenance. `.githooks/enable-hooks.sh` is governed by spec 090 in this
repository, and saying so at the top of the file is useful to a reader here. The
header form was the convenient way to say it, and it carries a second meaning the
author did not need: it is the claim syntax. Only the second meaning travels.

## 2. Territory

This spec establishes no new file. It claims, through `extends`:

| Path | Why |
|---|---|
| `kit/.githooks/enable-hooks.sh` | the header, in the kit's copy |
| `kit/.githooks/enable-merge-driver.sh` | as above |
| `kit/.githooks/merge-derived-index.sh` | as above |
| `.githooks/` (the same three names) | spec 064 §3.3 asserts the two trees are byte-equal, so the edit lands in both or the test goes red |
| `crates/spec-spine-core/tests/kit_gate.rs` | where the kit's shipped files are asserted |
| `crates/spec-spine-core/src/kit_embedded.rs` | generated from `kit/`, restamped by the edit |

`.githooks/pre-commit` and `kit/.githooks/pre-commit` carry the same header and
are **in** the territory: §1.2's reason for excluding them from the measurement
is that the scanner cannot see them, which is a fact about today's
`SOURCE_EXTS` and not a promise. Leaving a claim header in a file because
nothing currently reads it is the arrangement this spec exists to end.

## 3. Behavior

### 3.1 No delivered file carries a claim header

No file `spec-spine init --with-kit` writes into an adopter's tree MAY carry a
line that `index.rs::header_attempt` recognizes as a claim attempt, namely a
line whose first non-space content is `//` or `#` followed by `Spec:`.

A test MUST assert this over the kit tree, by applying the scanner's own
recognizer rather than a substring search for `Spec:`. Spec 094 §1 measured the
difference: a substring search hits eleven files in this repository and the
recognizer hits one.

### 3.2 Provenance is kept, in a form that is not a claim

Each file that loses a header MUST keep a comment naming the spec that governs
it in this repository, worded so the recognizer does not match it. The
information is worth keeping (§1.4) and the syntax is what travels badly.

The governing spec ids MUST be preserved exactly: `020` for the merge driver and
its enabler, `090` for the commit-boundary hook and its enabler.

### 3.3 The producer's own claims are unchanged

`.githooks/` is claimed here by spec 020's `establishes` on the directory, and
two of its files additionally by spec 090's `extends`. Those frontmatter claims
are the ones that have ever been read (§1.2), and this spec changes none of
them. `spec-spine index owner` MUST return the same owner set for each of the
seven paths after this spec as before it.

### 3.4 The two trees stay equal

Spec 064 §3.3 makes `kit/.githooks/` the source and this repository's
`.githooks/` a copy asserted equal to it. That assertion MUST still pass:
the edit is made in `kit/` and mirrored, never in one tree alone.

### 3.5 A fresh adoption reports no near miss it was handed

On the corpus of §1.1 (a scratch `init --with-kit` with one root crate),
`index coverage` MUST report no near-miss comment header and no `unknown-spec`,
and `index owner` MUST answer with the adopter's spec for a hook the adopter has
claimed in the documented way.

**What this does not do, stated because the obvious reading of the title is
wrong.** The three hooks remain **unclaimed** after this spec, and correctly so.
Removing a header that claims nothing does not claim anything; a file the
adopter has not decided about belongs in the unclaimed list beside their own
undecided sources, which is what `index coverage` is for. Under
`[coupling] require_ownership` they are still a `C-002` on the first pull
request that edits one, and the remedy is the ordinary one: claim them.

What changes is that the remedy now works. Before this spec an adopter who
followed it got the shadowing of §1.3, and the diagnostic pointing at the file
named a spec id they had never heard of. D-4.

## 4. Out of scope

- **Widening the claim scanner to files outside every package.** That is the
  change spec 097 §3.4 declined and spec 094 §4 reserves, and it would move
  ownership in every adopter corpus at once. This spec removes three bytes'
  worth of claim; it does not touch what claims.
- **Enabling `[coverage] governed_scope` in this repository.** 097 shipped the
  mechanism and left the adoption open (note 05 R-7, a human decision). Nothing
  here depends on it, and after this spec the three files carry no claim to
  become live if it is ever turned on.
- **Changing the shadowing rule.** 094 §3.2 makes the first attempt decide, and
  that is the right rule: an unresolvable header is a mistake worth surfacing,
  not a line to skip past. This spec removes the mistake from the shipped bytes.
- **`py/scripts/smoke_test.sh`, the seventh inert header 097 counted.** It is
  not delivered to anyone: it lives in this repository's PyPI shim. D-2.
- **The `.githooks/` files being unclaimed in an adopter's corpus at all.**
  After this spec they are unclaimed without a near miss and without shadowing,
  which is the correct state for a file the adopter has not yet decided about.
  Whether the kit should also ship guidance on claiming them is documentation,
  and D-3 records why it is not bundled here.

## 5. Resolved decisions

D-1 (2026-09-17, why the header is reworded rather than deleted). Deleting the
line loses the fact that spec 090 governs the hook, which is true, useful to a
reader of this repository, and recorded nowhere else in the file. The recognizer
is narrow enough that keeping the sentence costs nothing: it strips one `//` or
`#` and then requires the literal `Spec:`, so any other wording is not an
attempt. Provenance and claim were conflated by accident, and separating them
keeps both.

D-2 (2026-09-17, why `py/scripts/smoke_test.sh` stays as it is). Spec 097
counted seven inert headers. Six are in scope here because they are either
shipped to adopters or byte-equal to something that is. The seventh is in this
repository's PyPI distribution shim, reaches no adopter's tree, and would be a
change to spec 008's territory for no measured effect. It stays inert, and stays
recorded as inert by 097.

D-3 (2026-09-17, why no guidance is added to `kit/README.md`). An adopter now
sees three unclaimed files with no diagnostic attached, which is the same thing
they see for their own unclaimed sources, and the coverage report already names
the remedy in general terms. Adding a passage about claiming the hooks would be
a change to the README's shape from a spec whose measured defect is three lines
of shell, and `kit/README.md` is spec 064 §3.3's territory for the merge-driver
text specifically. It is a fair follow-up and it is not this.

D-4 (2026-09-17, why the scaffolded bootstrap spec does not claim `.githooks/`).
It is the one change that would make the hooks arrive owned, and it contradicts
the scaffold as it stands: `specs/000-bootstrap/spec.md` is written
`implementation: n-a` with the comment "this spec defines what a spec is; it owns
no code, so there is nothing to implement", and giving it an `establishes` on a
directory of shell scripts would make that sentence false in every corpus the
tool creates. An adopter deciding which spec owns their hooks is the adopter
making an authority decision, which is the thing this tool exists to keep in
their hands. §3.5 says so rather than letting the spec's title imply otherwise.

## Verification

Each line is one command, run independently: no shell variable survives to the
next line.

**Fail-first evidence.** Every `! grep` line over a hook fails at the parent
commit, where all eight files carry the header. The scratch-adopter lines fail
there too: `index coverage` reports three `unknown-spec` near misses, and
`index owner` answers "no spec owns this path" for a file carrying the adopter's
own claim on the line below the kit's (§1.3, reproduced). `registry show 115` is
a not-found exit 1 there.

Three lines are **green at the parent and stay green**: the two `index owner`
greps of §3.3, which assert that this repository's frontmatter claims are
untouched, and the coverage line asserting the hooks are still listed as
unclaimed. They are preservation assertions, and §3.5 and D-4 say what each one
would catch.

```verify:cli
cargo build --release --locked
# 3.1: no delivered hook carries a claim attempt. The recognizer is `//` or `#`,
# then `Spec:`; the pattern below is that shape and not a search for `Spec:`,
# which spec 094 1 measured as eleven files against the recognizer's one.
! grep -qE '^[[:space:]]*(//|#)[[:space:]]*Spec:' kit/.githooks/enable-hooks.sh
! grep -qE '^[[:space:]]*(//|#)[[:space:]]*Spec:' kit/.githooks/enable-merge-driver.sh
! grep -qE '^[[:space:]]*(//|#)[[:space:]]*Spec:' kit/.githooks/merge-derived-index.sh
! grep -qE '^[[:space:]]*(//|#)[[:space:]]*Spec:' kit/.githooks/pre-commit
! grep -qE '^[[:space:]]*(//|#)[[:space:]]*Spec:' .githooks/enable-hooks.sh
! grep -qE '^[[:space:]]*(//|#)[[:space:]]*Spec:' .githooks/enable-merge-driver.sh
! grep -qE '^[[:space:]]*(//|#)[[:space:]]*Spec:' .githooks/merge-derived-index.sh
! grep -qE '^[[:space:]]*(//|#)[[:space:]]*Spec:' .githooks/pre-commit
# 3.2: and the provenance survives, with the right spec id in each file. The id
# is matched in its full directory form, not as the bare digits: `020` is a
# substring of any date, ordinal or hash that happens to contain it, and two of
# these four lines were already green at the parent for exactly that reason
# (measured 2026-09-18). The full form is what a reader needs anyway.
grep -qF '020-derived-artifact-merge-driver' kit/.githooks/merge-derived-index.sh
grep -qF '020-derived-artifact-merge-driver' kit/.githooks/enable-merge-driver.sh
grep -qF '090-a-hook-bound-to-a-tool-route-misses-the-work' kit/.githooks/enable-hooks.sh
grep -qF '090-a-hook-bound-to-a-tool-route-misses-the-work' kit/.githooks/pre-commit
# 3.4: the two trees are still equal, and 065's generator is still in step.
cargo test -p spec-spine-core --test kit_gate --locked
cargo test -p spec-spine-core --test scaffold --locked
# 3.1: the recognizer-based assertion over the kit tree exists and ran. The
# summary is asserted to name a non-zero pass count, because a filter matching
# nothing exits 0 (spec 106 D-7).
cargo test -p spec-spine-core --test kit_gate --locked no_claim_header > "${TMPDIR:-/tmp}/ss115-hdr.txt" 2>&1
grep -qE 'test result: ok\. [1-9][0-9]* passed' "${TMPDIR:-/tmp}/ss115-hdr.txt"
# 3.3: this repository's own owner sets are unchanged, read through the CLI and
# not off the shard (.claude/rules/governed-artifact-reads.md).
target/release/spec-spine index owner .githooks/enable-hooks.sh > "${TMPDIR:-/tmp}/ss115-own.txt"
grep -qF '090-a-hook-bound-to-a-tool-route-misses-the-work' "${TMPDIR:-/tmp}/ss115-own.txt"
grep -qF '020-derived-artifact-merge-driver' "${TMPDIR:-/tmp}/ss115-own.txt"
# 3.5: a fresh adoption with one root crate reports no near miss it was handed.
rm -rf "${TMPDIR:-/tmp}/ss115" && mkdir -p "${TMPDIR:-/tmp}/ss115/src" && git -C "${TMPDIR:-/tmp}/ss115" init -q .
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss115" init --with-kit >/dev/null
printf '[package]\nname = "demo"\nversion = "0.1.0"\nedition = "2021"\n' > "${TMPDIR:-/tmp}/ss115/Cargo.toml"
printf 'fn main() {}\n' > "${TMPDIR:-/tmp}/ss115/src/main.rs"
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss115" compile >/dev/null
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss115" index >/dev/null
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss115" index coverage > "${TMPDIR:-/tmp}/ss115-cov.txt"
! grep -qF 'near-miss' "${TMPDIR:-/tmp}/ss115-cov.txt"
! grep -qF 'unknown-spec' "${TMPDIR:-/tmp}/ss115-cov.txt"
# 3.5: and the hooks are still listed as unclaimed, which is the correct state
# for a file the adopter has not decided about. This line is the guard on the
# reading D-4 refuses: a spec that made them arrive owned would break it.
grep -qF '.githooks/enable-hooks.sh' "${TMPDIR:-/tmp}/ss115-cov.txt"
# 1.3: and the adopter's own header, added in the documented place, now claims.
printf '#!/usr/bin/env bash\n# Spec: specs/000-bootstrap/spec.md\n' > "${TMPDIR:-/tmp}/ss115/.githooks/enable-hooks.sh"
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss115" index >/dev/null
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss115" index owner .githooks/enable-hooks.sh | grep -qF '000-bootstrap'
rm -rf "${TMPDIR:-/tmp}/ss115"
rm -f "${TMPDIR:-/tmp}/ss115-hdr.txt" "${TMPDIR:-/tmp}/ss115-own.txt" "${TMPDIR:-/tmp}/ss115-cov.txt"
```
