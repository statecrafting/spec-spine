---
id: "089-a-skip-and-a-failure-are-different-answers"
title: "A skip and a failure are different answers"
status: draft
kind: "tooling"
created: "2026-09-11"
summary: >
  The kit's Makefile guards its language targets on a manifest probe, written
  as `test -f M && cmd || echo skipping`. That is not an if/else: the `||`
  branch fires when the probe is false OR when the command fails, so on a
  repository that HAS the manifest a failing command exits 0 having printed
  "no manifest, skipping". Any adopter who copies `kit/govern.yml` unmodified
  into a repository with a `Cargo.toml` gets a build job that cannot fail:
  a broken `cargo test`, a `cargo fmt --check` diff and a `clippy -D warnings`
  hit are all reported as a skip and the check goes green. Spec 064's
  acceptance could not catch it, because it asserts the probe is a manifest
  probe and that is true of the broken shape and the fixed one alike. This
  spec makes the guard an explicit `if`, and adds the acceptance that tells
  the two apart: the shipped recipe lines are run in both states.
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "064-the-kit-ships-the-composite-gate"
  - "065-init-and-the-kit-are-one-adoption"
extends:
  # 3.1: the five guarded recipe lines.
  - { spec: "064-the-kit-ships-the-composite-gate", unit: "kit/Makefile", nature: corrective }
  # 3.2: the acceptance that runs the shipped lines in both states.
  - { spec: "064-the-kit-ships-the-composite-gate", unit: "crates/spec-spine-core/tests/kit_gate.rs", nature: additive }
  # 3.3: the embedded copy `init --with-kit` writes.
  - { spec: "065-init-and-the-kit-are-one-adoption", unit: "crates/spec-spine-core/src/kit_embedded.rs", nature: corrective }
---

# 089: A skip and a failure are different answers

## 1. Purpose

`kit/Makefile` guards each language target so the whole gate is a clean no-op
on a corpus with no code. That intent is right and spec 064 section 3.1 argues
it well: a machine with `cargo` installed and no `Cargo.toml` is the
specify-first case, three of the four governed repositories, and probing for
the tool answers the wrong question.

The implementation of that intent conflates two different answers:

```make
	@test -f Cargo.toml && cargo test --workspace --locked || echo "no Cargo.toml, skipping"
```

`cmd_a && cmd_b || cmd_c` is not an if/else. `cmd_c` runs when **either**
`cmd_a` or `cmd_b` fails, so it is only safe when `cmd_b` cannot fail. Here
`cmd_b` is the entire test suite. On a repository that has the manifest, a
failing `cargo test` falls into the same branch as an absent `Cargo.toml`, the
target prints `no Cargo.toml, skipping` (a false statement about a file that
is right there) and exits 0.

Reproduced with the probe true and the command forced to fail:

| form | output | exit |
|---|---|---|
| `test -f Cargo.toml && false \|\| echo "no Cargo.toml, skipping"` | `no Cargo.toml, skipping` | 0 |
| `if test -f Cargo.toml; then false; else echo "skipping"; fi` | nothing | 1 |

**Where it bites.** `kit/govern.yml` line 81 is `- run: make build test fmt
clippy`, gated on the `has_cargo` job output. An adopter who copies that
workflow unmodified into a repository with a `Cargo.toml` gets a Rust job that
cannot fail. That is the worst shape a gate can have: the repository looks
defended and is not, and nothing in the run says otherwise.

**What it costs today: nothing, and that is luck rather than design.** Every
current copy dodges it by accident. This repository's `ci.yml` runs
`make -f kit/Makefile gate`, and `gate` has no guarded line; the cargo stack
runs directly in the same workflow. `canonical-keysort-json` deleted the probe
and build jobs when it copied `govern.yml`, to avoid running every cargo
command twice per pull request. `statecrafting-profile` runs the four targets
but has no manifest at all, so the job is gated off. `statecrafting` replaced
the guarded `test:` target with a plain `npm test`. The defect is latent, and
a latent defect in a shipped template is a defect with an audience of everyone
who has not adopted yet.

Found on 2026-09-11 while adopting the 0.18.0 kit into `statecraft.ing`, which
patched the two `package.json` lines locally under its own spec 005 and
recorded the divergence as a decision. That patch is the adopter-side workaround.
This is the fix.

## 2. Territory

- **Owns nothing new.** Three units, all extended from their owners: the five
  guarded recipe lines in `kit/Makefile` and the test file that holds the kit
  gate's acceptance (spec 064), and the generated module `init --with-kit`
  writes from the kit (spec 065).
- **Does not touch `gate`, `refresh` or `verify`.** They carry no guarded line
  and no manifest probe. The read-only contract spec 064 section 3.1 places on
  `gate` is unchanged and still asserted.
- **Does not touch `kit/govern.yml`.** The workflow is correct; it calls
  targets that were not. Fixing the Makefile fixes the job.
- **Does not touch this repository's own gate.** `ci.yml` calls
  `make -f kit/Makefile gate` and runs cargo directly, so nothing here changes
  what this repository's CI enforces.

## 3. Behavior

### 3.1 The guard is an explicit conditional

Each of the five guarded recipe lines MUST be written as:

```make
	@if test -f <manifest>; then <command>; else echo "no <manifest>, skipping"; fi
```

`test -f` is kept as the probe rather than `[ -f ]` so spec 064's
`language_targets_probe_for_a_manifest_not_a_tool` keeps passing unchanged:
the probe it asserts is still there, and this spec changes only what happens
to the command's exit status. The no-op property that guard exists for is
unchanged: with the manifest absent, the command never runs, the line prints
the skip and exits 0.

The header comment states the rule next to the manifest-probe rule it
qualifies, because the `||` form is the one a reader reaches for and the next
person to touch this file needs to know why it is not used.

### 3.2 The acceptance runs the shipped lines, in both states

Spec 064's `language_targets_probe_for_a_manifest_not_a_tool` asserts each
target's body contains `test -f Cargo.toml` or `test -f package.json`. Both
the broken and the fixed shape satisfy it. An acceptance that never forces a
command to fail cannot distinguish a guard from a swallow, so this spec adds
two tests to `crates/spec-spine-core/tests/kit_gate.rs`:

- **static**: no guarded line combines `&&` with `|| echo`, and every guarded
  line is the `if test -f ...; then ...; else ...; fi` form. This refuses the
  shape at review time rather than only at run time.
- **behavioral**: each guarded line is extracted from the shipped
  `kit/Makefile` and executed, twice. With no manifest present it must exit 0
  and print the skip. With the manifest present and its command replaced by
  `false`, it must exit non-zero and must not print the skip.

The behavioral test runs the **shipped bytes**, not a hand-written copy of
them, which is the only form that would have caught the original defect. It
runs each line under `sh -c` rather than through `make`: make runs every
recipe line in its own shell, so the line's exit status is the target's
verdict for that step, and testing the line directly removes a dependency on
`make` being installed wherever the suite runs.

Both tests fail against pre-089 code and pass after it. Spec 064's own test
passes in both states, which is the point.

### 3.3 The embedded copy is regenerated

`crates/spec-spine-core/src/kit_embedded.rs` is generated from `kit/` by
`scripts/gen-kit-embedded.py`, and `init --with-kit` writes the embedded bytes
rather than reading `kit/`. A fix to `kit/Makefile` that does not regenerate
the module fixes the file in this repository and ships the broken one to every
new adopter. The module is regenerated in the same change, and the existing
test asserting the two agree holds it there.

## 4. Out of scope

- **A lint.** The conflating form is a shell idiom, not a corpus property, and
  a `spec-spine` lint over adopter Makefiles is a different tool than the one
  this repository builds. The static test in 3.2 covers the file that ships.
- **Auditing every adopter's own Makefile.** Adopters customize the language
  targets (`statecrafting` already replaced `test:` outright), so their copies
  are theirs. The release note is the right instrument: adopters re-copy
  `kit/Makefile`.
- **`kit/govern.yml`.** Correct as written; see section 2.
- **The `||` idiom elsewhere in the kit.** The hooks in `kit/settings.json`
  use `||` for control flow in several places, all of them where the left side
  is a probe that cannot meaningfully "fail" in the swallow sense. Re-reading
  them is worthwhile and is not this change.

## 5. Resolved decisions

- **2026-09-11: `test -f` is kept, `[ -f ]` rejected.** Both are POSIX and
  equivalent. `test -f` keeps spec 064's acceptance passing without editing
  it, so the diff stays about the defect. Editing another spec's assertion to
  accommodate a fix that did not need to would be noise in exactly the place
  where noise is expensive.
- **2026-09-11: the behavioral test runs `sh -c`, not `make`.** Running the
  real `make` would also exercise make's own failure propagation, which is not
  in doubt and is not what broke. `sh -c` on the recipe line tests the thing
  that was wrong, needs no `make` on the test host, and runs in milliseconds.
- **2026-09-11: the cargo lines are fixed here even though no repository
  currently runs them through this Makefile.** The adopter that found this
  patched only its two `package.json` lines, correctly, because it has no
  `Cargo.toml`. The kit is the template, and a template is fixed for the
  adopter who has not arrived yet.
- **2026-09-11: `nature: corrective` on `kit/Makefile` and the embedded
  module.** The edges to those two units change behavior the owning specs
  described, rather than adding to it. The edge on the test file is
  `additive`: spec 064's tests all stay and two join them.

## Verification

Each line is one command. The first three fail against pre-089 code: the two
new tests do not exist, and the shipped Makefile carries the conflating form.

```verify:cli
cargo test -p spec-spine-core --test kit_gate --locked
# 3.1: no guarded line conflates a false probe with a failed command.
! grep -qE '^	@test -f .+ && .+ \|\| echo' kit/Makefile
# 3.1: all five guarded lines are the explicit conditional.
sh -c 'test "$(grep -cE "^	@if test -f " kit/Makefile)" = 5'
# 3.3: the embedded copy adopters receive carries the fixed bytes.
grep -qF 'if test -f Cargo.toml; then cargo test --workspace --locked' crates/spec-spine-core/src/kit_embedded.rs
grep -qF 'if test -f package.json; then npm test --if-present' crates/spec-spine-core/src/kit_embedded.rs
```
