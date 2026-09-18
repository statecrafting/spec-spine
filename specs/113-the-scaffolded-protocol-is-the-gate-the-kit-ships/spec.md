---
id: "113-the-scaffolded-protocol-is-the-gate-the-kit-ships"
title: "The scaffolded protocol is the gate the kit ships"
status: approved
kind: "tooling"
created: "2026-09-17"
summary: >
  `init` writes an `AGENTS.md` whose gate is the shape this repository ran in
  June, and `kit/AGENTS.md`, which the same command copies into the same tree
  under `--with-kit`, carries the shape it runs today. They disagree on four
  lines: the freshness verb, the coupling base, whether the ownership assertion
  is conditional, and what a session reads at startup. Both invocations still
  work, so nothing is broken and an adopter simply gets the older gate, told in
  two voices by one command. This spec makes the generated protocol render the
  gate the kit documents. Spec 065's acceptance greps the scaffolded file for
  `spec-spine compile --check`, so the change falsifies an assertion narrower
  than the rule it stands for, and this spec declares 065's acceptance replaced
  rather than editing 065.
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "006-init-scaffold"
  - "064-the-kit-ships-the-composite-gate"
  - "065-init-and-the-kit-are-one-adoption"
  - "072-the-default-branch-is-configured-not-assumed"
  - "075-one-name-one-freshness-verb"
  - "103-an-amended-acceptance-is-the-one-that-runs"
extends:
  # The hand-maintained string literal, and the two suites that assert it.
  - { spec: "006-init-scaffold", unit: "crates/spec-spine-core/src/scaffold.rs", nature: additive }
  - { spec: "006-init-scaffold", unit: "crates/spec-spine-core/tests/scaffold.rs", nature: additive }
  - { spec: "006-init-scaffold", unit: "crates/spec-spine-cli/tests/init.rs", nature: additive }
  # 3.2's parity test lands beside the gate-chain assertions it reuses, in a
  # file spec 064 established. Without this edge the test the spec requires is
  # an unclaimed edit and the coupling gate refuses the build that writes it.
  - { spec: "064-the-kit-ships-the-composite-gate", unit: "crates/spec-spine-core/tests/kit_gate.rs", nature: additive }
amends: ["065-init-and-the-kit-are-one-adoption"]
# 3.5: this spec's `## Verification` block IS 065's acceptance from now on.
# 065's file is not edited (spec 040 3.1). What changes is one assertion in it,
# which pinned the verb the scaffold emitted rather than the property 065 3.1
# states.
amends_verification: ["065-init-and-the-kit-are-one-adoption"]
references:
  - { unit: { kind: file, path: "docs/design/05-remaining-waves-2026-09.md" }, role: context }
---

# 113: The scaffolded protocol is the gate the kit ships

## 1. Purpose

### 1.1 One command, two gates

`spec-spine init --with-kit` writes two documents into one tree. It generates
`AGENTS.md` from a string literal in `crates/spec-spine-core/src/scaffold.rs`,
and it copies `kit/AGENTS.md` nowhere (065 §3.2's decision: the scaffold emits a
config-aware protocol instead), but the kit's own `AGENTS.md` is what an adopter
reads on the repository page and in the docs. The two disagree.

Measured on 2026-09-17 against `7a7a8b6` with the 0.20.0 binary:

| | Generated (`scaffold.rs`) | `kit/AGENTS.md` |
|---|---|---|
| Freshness verb in the gate | `spec-spine index check --fail-on-unresolved` | `spec-spine check` (spec 075: both trees, one verb) |
| Coupling base | `couple --base origin/main` | `--base "$(git symbolic-ref --short refs/remotes/origin/HEAD 2>/dev/null \|\| echo origin/main)"` (spec 072) |
| Ownership assertion | `index coverage --fail-on-untraced`, unconditional | commented, conditional on `[coupling] require_ownership` |
| Startup freshness read | `spec-spine compile --check` | `spec-spine check` |

Every invocation on the left still works. `index check` did not go away when
spec 075 composed `check`, `origin/main` is the third step of the very fallback
spec 072 wrote, and `compile --check` still reads the registry. So this is not a
broken scaffold; it is an older one, and the defect is that one command tells an
adopter two different things about the same gate.

### 1.2 The drift is invisible to every test that watches this file

`tests/scaffold.rs` asserts the generated tree against expectations written
beside each change, and `kit_gate.rs` asserts that `kit/Makefile`'s chain is the
chain **this repository's** `AGENTS.md` lists, in order
(`the_kit_gate_chain_follows_agents_md`). Nothing compares the **generated**
protocol against the kit's. Spec 075 moved this repository's `AGENTS.md` and
`kit/AGENTS.md` to the composed verb, and asserted both:

```
! grep -qF 'compile --check' AGENTS.md
! grep -qF 'index check' AGENTS.md
```

Both lines read the repository's own file. The scaffolded literal, which is a
third copy of the same protocol living inside a Rust source file, was not in
scope and did not move.

### 1.3 The scaffold is hand-maintained, and heavily co-owned

`kit_embedded.rs` is generated from `kit/` by `scripts/gen-kit-embedded.py`, so
a kit edit propagates. `scaffold.rs` is not generated from anything: the
protocol is a string literal, established by spec 006 and extended by ten specs
since (043, 045, 047, 061, 062, 065, 066, 069, 074, 097). Each of those edits
was a targeted insertion, and none had reason to re-read the gate block.

## 2. Territory

This spec establishes no new file. It claims three units another spec owns,
through `extends`:

| Path | Why |
|---|---|
| `crates/spec-spine-core/src/scaffold.rs` | the literal itself |
| `crates/spec-spine-core/tests/scaffold.rs` | where the scaffolded text is asserted |
| `crates/spec-spine-cli/tests/init.rs` | the `--with-kit` path's assertions |
| `crates/spec-spine-core/tests/kit_gate.rs` | where §3.2's parity test lands, beside the gate-chain assertions it reuses |

and one claim about another spec's file: that spec 065's `## Verification` block
is no longer the one that runs (§3.5).

No schema constant moves, no CLI surface changes, and no verb's behavior
changes. What changes is four lines of text a generator emits, and the
assertions that read them.

## 3. Behavior

### 3.1 The generated gate is the kit's gate

The gate block in the `AGENTS.md` that `scaffold.rs` emits MUST name the same
verbs, with the same flags, in the same order, as the fenced gate list in
`kit/AGENTS.md`, allowing for the configured paths the scaffold substitutes.
Specifically:

- the freshness read MUST be `spec-spine check` (spec 075 §3.1), not
  `spec-spine index check --fail-on-unresolved`;
- the coupling base MUST be resolved from the repository rather than written as
  the literal `origin/main`, and MUST be the same resolution `kit/AGENTS.md`
  renders: `git symbolic-ref --short refs/remotes/origin/HEAD`, falling back to
  `origin/main`. The `$SPEC_SPINE_DEFAULT_BRANCH` override is carried the way
  the kit carries it, in the sentence under the block, which names the two
  places that read the variable (the push gate and `kit/Makefile`). D-3;
- the ownership assertion MUST carry the same condition `kit/AGENTS.md` states,
  namely that it belongs in the gate when `[coupling] require_ownership` is on;
- the startup freshness read in the New Sessions protocol MUST be
  `spec-spine check`, not `spec-spine compile --check`.

### 3.2 The two protocols are compared by a test, not by a reader

A test MUST assert that the generated gate list and `kit/AGENTS.md`'s gate list
name the same verbs in the same order. Reusing `kit_gate.rs`'s existing
`invocations` and `agents_md_gate_commands` helpers is the obvious shape and is
not required; what is required is that a future edit to either list, without the
other, fails a test rather than shipping.

This is the assertion §1.2 says does not exist, and it is the half of this spec
that keeps the defect from recurring. Fixing four lines without it buys one
release.

### 3.3 The scaffold stays config-aware

065 §3.1 requires the emitted `AGENTS.md` to render the adopter's own corpus and
derived paths, which is why it is generated rather than copied. That MUST NOT
change: the verbs and flags become the kit's, the paths stay the adopter's, and
a corpus configured with a non-default `[layout] specs_dir` MUST still read its
own directory in the protocol it is given.

Config-awareness MUST be asserted against a **non-default** configuration. A
corpus left at the default renders `specs/` whether the generator substitutes
anything or not, so the default path cannot tell a config-aware generator from a
hard-coded one. The assertion MUST also be a whole-path one: a configured
`governance/specs` **contains** the default `specs/` as a substring, so a
substring search for the default matches a correctly configured tree and proves
nothing either way. D-4.

### 3.4 `init` without `--with-kit` is unchanged in shape

Plain `init` writes the protocol and not the harness (065 §3.2). That split MUST
hold: this spec changes what the protocol says, never which files either mode
writes.

### 3.5 What spec 065's acceptance now is

This spec's `## Verification` block MUST replace spec 065's in full, through
`amends_verification` (spec 103 §3.1), and 065's file MUST NOT be edited.

The line that moves is 065's

```
grep -q 'spec-spine compile --check' "${TMPDIR:-/tmp}/ss065/AGENTS.md"
```

which sits under the comment "3.1: the protocol is there, and names the
non-writing reads". 065 §3.1's requirement is that the scaffolded protocol names
reads that do not write; `compile --check` was a true rendering of that on
2026-09-07 and `check` is the true rendering of it now, and spec 075 is the
approved spec that made the second one true. The assertion pinned the spelling,
not the property: the same finding spec 108 §3.6 records for 093 and 059, and
spec 110 §1.2 for 071 and 072.

The replacement block MUST assert the property instead: that the scaffolded
protocol names the composed read, and that it does **not** name the superseded
one. The negative matters for the same reason spec 110 §3.2 gives: a literal
left behind on another branch of the same generator would satisfy a positive
grep while shipping two protocols that disagree.

Carrying the block in full rather than a patch follows spec 103 §3.5 and spec
105 D-1.

### 3.6 Filing this spec moves spec 065's acceptance before the fix exists

`amends_verification` takes effect the moment this spec's frontmatter is in the
corpus. `resolve_acceptance_source` skips only a `superseded` or `retired`
holder (spec 103 §3.2, D-5), so a `draft` holder is live: with this file merged
and unimplemented, `spec-spine verify 065 --plan` selects **this**
block, and `make verify SPEC=065` runs it.

Measured on 2026-09-18 at `270157b`: `verify 065 --plan` already resolves
here, on the unmerged filing branch.

That is spec 103 working as designed, and this spec does **not** change it. The
consequence is about **when this file lands**, not about the rule:

- merged before the build, spec 065's acceptance is this block, and this block
  fails, because it asserts the behavior the build has yet to write. An approved
  spec's acceptance would be red on the default branch, and `verify 065`
  would report the absence of a fix rather than the presence of a regression;
- merged **with** the build, the acceptance it replaces is green on arrival and
  the replacement is never worse than what it replaced.

So this spec is filed and built in one change, or filed on a branch that is not
merged until the build is on it. It is not merged as documentation. The
`amends_verification` declaration is required by §3.5 and is not removed
to make a filing-only merge safe: removing it would leave spec 065's stale
assertion live against changed behavior, which is the defect this spec exists to
resolve. D-5.

## 4. Out of scope

- **Editing spec 065, or spec 075.** 065 keeps the block it was ratified with,
  which is the record spec 040 §3.2 protects; 075's text is true as written and
  this spec is the consequence of it reaching a third copy.
- **Generating `scaffold.rs` from `kit/AGENTS.md`.** It is the obvious end
  state and it is a different change: 065 §3.1 requires a config-aware
  document, so a generator would have to template the paths out of the kit's
  file, which means editing `kit/AGENTS.md` into a template and changing what
  every adopter reads. D-1.
- **The rest of the scaffolded protocol.** Only the four lines §1.1 measures
  move. The scaffold's prose about lifecycle, waivers and the derived boundary
  is not compared to the kit's here, and §3.2's test is scoped to the gate list
  for that reason. D-2.
- **Making a documented shell command honour `$SPEC_SPINE_DEFAULT_BRANCH`.**
  Neither `kit/AGENTS.md` nor this repository's `AGENTS.md` does today, and 072
  does not ask them to; changing that is a change to 072's territory and to what
  every adopter reads, not a generator fix. D-3.
- **`docs/adoption-guide.md`.** Checked on 2026-09-17: it already says
  `spec-spine check`, so the guide is ahead of the scaffold, not behind it.
- **The empty-corpus refusal the scaffolded gate inherits.** Making the
  ownership assertion conditional here removes it from the generated protocol's
  unconditional list; the same refusal reaching `kit/Makefile`'s `gate` target
  is spec 114's, and this spec does not touch that file.

## 5. Resolved decisions

D-1 (2026-09-17, why the literal is corrected rather than generated from the
kit). Generating one from the other is the only shape that cannot drift again,
and it is blocked on a decision this spec has no standing to take: 065 §3.1
requires the adopter's paths in the emitted document, so the kit's `AGENTS.md`
would have to become a template, and what every adopter reads on the repository
page would change to accommodate a generator. §3.2's parity test buys the same
protection against the same failure at a fraction of the blast radius, and
leaves the generator available as a later spec.

D-3 (2026-09-18, the parity requirement and the branch override, which as
originally drafted could not both hold). §3.1's opening sentence requires the
generated gate list to name the same verbs **with the same flags** as
`kit/AGENTS.md`'s, and its second bullet as drafted required the base to be
resolved in spec 072 §3.1's three steps, beginning with
`$SPEC_SPINE_DEFAULT_BRANCH`. `kit/AGENTS.md` renders two steps
(`git symbolic-ref ... || echo origin/main`), and so does this repository's own
`AGENTS.md`. A generator satisfying the bullet would therefore fail the parity
test §3.2 requires, and one satisfying parity would fail the bullet.

The conflict is resolved toward parity, because the bullet was the half that
overstated its source. 072 §3.1's three steps are a requirement on **the push
gate hook**, and 072 §3.3 explicitly permits a build file to compute the value
"rather than repeating the hook's three steps inline". 072 §3.4, which governs
"the documented commands", requires only that they stop naming a branch that may
not exist. Nothing in 072 asks a documented shell command to read
`$SPEC_SPINE_DEFAULT_BRANCH`, and the kit's prose is accurate about the scope it
claims: it says the variable overrides "the branch the push gate protects and
`kit/Makefile` compares against", which is exactly the two places that read it.

So the generated protocol renders the kit's two-step command and the kit's
sentence beneath it. An adopter who wants a documented command that honours the
variable is asking for a change to `kit/AGENTS.md` and to this repository's
`AGENTS.md` as well, which is a different spec against 072's territory, not a
generator fix. §4 records it as out of scope.

D-4 (2026-09-18, why §3.3's assertion needs a non-default corpus and a
whole-path match). Measured on 2026-09-18 at `270157b`: `init` on a tree
carrying `[layout] specs_dir = "governance/specs"` emits an `AGENTS.md` naming
`governance/specs/`, and `load_repo_config` is read before the scaffold is
built, so the generator is genuinely config-aware today. Both halves of the
assertion are needed to show it. A default corpus renders `specs/` from a
hard-coded literal just as readily as from a substitution, and
`grep -F 'specs/'` matches `governance/specs/` as a substring, so the drafted
line was green against a config-aware generator, a hard-coded one, and a
configured tree alike. The replacement configures a non-default directory and
asserts the configured path is present **and** that no bare default path is
rendered.

D-2 (2026-09-17, why only the gate list is compared). A test that compared the
two protocols in full would fail on the first sentence: one is written for an
adopter installing the kit and the other for a repository that already has it,
and they differ deliberately in almost every paragraph. The gate list is the
part that is meant to be the same thing said twice, which is why §1.1 measures
exactly that and §3.2 asserts exactly that.

D-5 (2026-09-18, why this spec is not merged as a filing on its own).
Recorded because the obvious way to clear a backlog is to merge the drafts and
build them later, and for a spec carrying `amends_verification` that is not a
neutral act. §3.6 measures what it costs: spec 065's acceptance becomes this
block as soon as the frontmatter is in the corpus, because spec 103 §3.2's
resolver excludes only `superseded` and `retired` holders and a `draft` holder
resolves like any other.

The alternative considered and rejected was excluding `draft` holders from
resolution. It would make filing free, and it would also mean an amendment's
acceptance does not take effect until ratification, which in this repository's
producer lifecycle happens **after** the build has merged (`AGENTS.md`, "Working
the backlog"). The replacement would then be inert over exactly the window it is
written for, and the stale assertion it replaces would run against the new
behavior. That is a change to spec 103's governed behavior with its own
trade-offs, and it is not this spec's to take: it belongs in a spec against 103,
deliberately, or nowhere.

The cheap fix is scheduling: file and build in one change. This spec does that.

## Verification

Each line is one command, run independently: no shell variable survives to the
next line. This block is spec 065's acceptance (§3.5) as well as this spec's
own, so it is read in two halves and labelled as such.

**Fail-first evidence.** Measured against the scaffolded tree at the parent
commit, which carries `compile --check`, `index check --fail-on-unresolved` and
`couple --base origin/main`, and carries neither `symbolic-ref` nor
`require_ownership`:

| Line | At the parent |
|---|---|
| `grep 'spec-spine check'` in the scaffolded `AGENTS.md` | red |
| `! grep 'spec-spine compile --check'` | red |
| `grep 'symbolic-ref'`, `grep 'require_ownership'` | red, both absent |
| `! grep 'couple --base origin/main'`, `! grep 'index check --fail-on-unresolved'` | red |
| `registry show 113` | red, not found, exit 1 |
| the 3.2 parity test | does not exist, so the line fails to compile |

The 065 half's other lines are green at the parent and stay green: this spec
weakens none of them, and §3.5 moves exactly one.

**Green at the parent, and preservation rather than evidence.** §3.3's four
lines are labelled here rather than left to look like evidence, because §3.3 is
a "MUST NOT change" requirement and its assertions are therefore green on both
sides by design. Measured on 2026-09-18 at `270157b`: a tree carrying
`specs_dir = "governance/specs"` already gets an `AGENTS.md` naming
`governance/specs/`, already carries no bare `specs/` path, and already has its
bootstrap spec written under the configured directory. What those lines defend
is the build: a generator that hard-coded paths while rewriting the gate block
would satisfy every red line in the table above and turn all four of these red.
That is the failure they exist to catch, and it is a failure this spec's own
change is the most likely cause of.

The drafted line they replace, `grep -qF 'specs/'` against a default corpus,
could not catch it: it was green against a config-aware generator, a hard-coded
one, and a configured tree alike (D-4).

```verify:cli
# --- spec 065's acceptance, which this block now holds (3.5) ---
# Self-contained: the commands below invoke the release binary.
cargo build --release --locked
cargo test -p spec-spine-core --test scaffold --locked
cargo test -p spec-spine-cli --test init --locked
# 065 3.2: scaffold a fresh adopter with the harness.
rm -rf "${TMPDIR:-/tmp}/ss065" && mkdir -p "${TMPDIR:-/tmp}/ss065" && target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss065" init --with-kit >/dev/null
# 065 3.1: the protocol is there, and names the non-writing reads. Since spec
# 075 the composed read is `spec-spine check`, which is what 065 3.1 requires a
# protocol to name; `compile --check` was that rendering in September and is
# not any more (113 3.5).
test -f "${TMPDIR:-/tmp}/ss065/AGENTS.md"
grep -qF 'spec-spine check' "${TMPDIR:-/tmp}/ss065/AGENTS.md"
# And the superseded spelling is gone, not merely joined: a generator emitting
# both would satisfy the line above while shipping two protocols that disagree.
! grep -qF 'spec-spine compile --check' "${TMPDIR:-/tmp}/ss065/AGENTS.md"
# 065 3.2: the harness landed at the adopter's own paths, not under `kit/`.
test -f "${TMPDIR:-/tmp}/ss065/.claude/skills/build/SKILL.md"
test -f "${TMPDIR:-/tmp}/ss065/.claude/settings.json"
test -f "${TMPDIR:-/tmp}/ss065/.github/workflows/govern.yml"
test ! -d "${TMPDIR:-/tmp}/ss065/kit"
# 065 3.4: and the scaffolded repository satisfies its own gate on the first run.
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss065" compile >/dev/null
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss065" index >/dev/null
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss065" lint --fail-on-warn
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss065" index check
# 065 3.2: plain `init` writes the protocol and not the harness.
rm -rf "${TMPDIR:-/tmp}/ss065b" && mkdir -p "${TMPDIR:-/tmp}/ss065b" && target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss065b" init >/dev/null && test -f "${TMPDIR:-/tmp}/ss065b/AGENTS.md" && test ! -d "${TMPDIR:-/tmp}/ss065b/.claude/skills"
# 065 3.5: the README says how it is installed.
grep -q 'init --with-kit' kit/README.md
# --- spec 113's own requirements ---
# 3.1: the coupling base is resolved, not the literal `origin/main`.
grep -qF 'symbolic-ref' "${TMPDIR:-/tmp}/ss065/AGENTS.md"
! grep -qF 'couple --base origin/main' "${TMPDIR:-/tmp}/ss065/AGENTS.md"
# 3.1: the ownership assertion carries the condition the kit states.
grep -qF 'require_ownership' "${TMPDIR:-/tmp}/ss065/AGENTS.md"
# 3.1: and the superseded freshness verb is gone from the gate list too.
! grep -qF 'index check --fail-on-unresolved' "${TMPDIR:-/tmp}/ss065/AGENTS.md"
# 3.3: the emitted protocol is still config-aware, asserted against a
# NON-DEFAULT corpus directory. A default tree renders `specs/` from a hard-coded
# literal exactly as readily as from a substitution, so it cannot tell the two
# apart (D-4).
rm -rf "${TMPDIR:-/tmp}/ss113" && mkdir -p "${TMPDIR:-/tmp}/ss113" && printf '[meta]\nrequired_version = ">=0.20.0"\n[layout]\nspecs_dir = "governance/specs"\n' > "${TMPDIR:-/tmp}/ss113/spec-spine.toml"
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss113" init >/dev/null
grep -qF 'governance/specs/' "${TMPDIR:-/tmp}/ss113/AGENTS.md"
# And no BARE default path is rendered beside it. `grep -F 'specs/'` alone is
# green here either way, because `governance/specs/` contains it: the match has
# to be anchored so the configured prefix is required (D-4).
! grep -qE '(^|[^/[:alnum:]_-])specs/' "${TMPDIR:-/tmp}/ss113/AGENTS.md"
# The configured corpus is also where `init` actually wrote the bootstrap spec,
# so the document and the tree agree rather than only the document being right.
test -d "${TMPDIR:-/tmp}/ss113/governance/specs" && test ! -d "${TMPDIR:-/tmp}/ss113/specs"
# 3.2: the parity test exists and passes, and its summary names a non-zero pass
# count: a name filter that matches nothing exits 0, so the bare line would stay
# green while asserting nothing (spec 106 D-7).
cargo test -p spec-spine-core --test kit_gate --locked scaffolded_protocol > "${TMPDIR:-/tmp}/ss113-parity.txt" 2>&1
grep -qE 'test result: ok\. [1-9][0-9]* passed' "${TMPDIR:-/tmp}/ss113-parity.txt"
# The replacement is declared, read through the CLI rather than off the shard.
# Redirected, not piped: at the parent commit this verb exits 1 and prints
# nothing, and a pipeline would report that as a JSON decode error naming the
# wrong defect (spec 107 D-4).
target/release/spec-spine registry show 113 --json > "${TMPDIR:-/tmp}/ss113-show.json"
python3 -c "import json; d=json.load(open('${TMPDIR:-/tmp}/ss113-show.json')); assert d['amendsVerification'] == ['065-init-and-the-kit-are-one-adoption'], d; assert d['amends'] == ['065-init-and-the-kit-are-one-adoption'], d"
# Spec 065's file is not edited (spec 040 3.1): its own block still carries the
# superseded assertion. This goes red the moment someone resolves this by
# editing 065 instead.
grep -qF "grep -q 'spec-spine compile --check'" specs/065-init-and-the-kit-are-one-adoption/spec.md
rm -rf "${TMPDIR:-/tmp}/ss065" "${TMPDIR:-/tmp}/ss065b" "${TMPDIR:-/tmp}/ss113"
rm -f "${TMPDIR:-/tmp}/ss113-parity.txt" "${TMPDIR:-/tmp}/ss113-show.json"
```
