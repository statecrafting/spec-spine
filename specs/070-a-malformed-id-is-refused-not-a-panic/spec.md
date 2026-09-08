---
id: "070-a-malformed-id-is-refused-not-a-panic"
title: "A malformed id is refused, not a panic"
status: draft
kind: "tooling"
created: "2026-09-08"
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "001-compile-registry"
  - "053-depends-on-ordinal-monotonicity"
extends:
  # 3.1 and 3.2: the V-004 prefix, computed by character and only when numeric.
  - { spec: "001-compile-registry", unit: "crates/spec-spine-core/src/compile.rs", nature: additive }
  # 3.3: the regression guards.
  - { spec: "001-compile-registry", unit: "crates/spec-spine-core/tests/compile.rs", nature: additive }
references:
  - { unit: { kind: file, path: "docs/design/00-architecture.md" }, role: context }
summary: >
  `detect_duplicates` reads a spec id's V-004 prefix as `&id[..id.len().min(3)]`,
  a byte slice. An id whose fourth byte falls inside a multi-byte character
  panics the process instead of diagnosing it, so `a日本-spec` exits 101 with a
  backtrace where the corpus should have been refused at exit 1 by V-012, the
  code that exists to name exactly that malformation. Spec 053 §4 found the
  defect, declined to fix it from inside a lint change, and asked for this spec.
  The fix reads the prefix the way spec 001 §3.2 already describes it, as a
  numeric prefix (`NNN`), using the character-safe leading-digit-run rule spec
  053 established in `lint.rs`.
---

# 070: A malformed id is refused, not a panic

## 1. Purpose

Restore the panic-free-on-user-input invariant to `compile`, and with it the
stable exit-code contract, for the one input that currently breaks both.

`crates/spec-spine-core/src/compile.rs` computes the V-004 duplicate-prefix key
as:

```rust
let prefix = &id[..id.len().min(3)];
```

That is a byte slice at a fixed offset. Rust's `str` indexing panics when the
offset is not a character boundary, so any id whose byte 3 falls inside a
multi-byte character aborts the process:

```
thread 'main' panicked at crates/spec-spine-core/src/compile.rs:353:25:
byte index 3 is not a char boundary; it is inside '日' (bytes 1..4) of `a日本-spec`
```

Two contracts break at once. `CLAUDE.md` states that core is "panic-free on user
input: malformed config/frontmatter yields a clean `Error`, never a panic", and
the corpus publishes `0` / `1` / `2` / `3` as a stable exit-code contract. A
panic spends 101, which is not in that set, and it preempts the diagnosis: the
directory name is already a `V-012` violation, and `V-012` is the code whose
whole job is to say so. The tool crashes on its way to telling the truth.

The trigger is narrow (an id is authored, not attacker-supplied) which is why
this is `risk: medium` rather than higher. It is still the only known input that
makes a governed read abort instead of answering, and a gate that can crash is a
gate an adopter cannot put in CI with confidence.

## 2. Territory

No new files. Two units, both owned by spec 001 and both `extends`-ed here:

| Unit | Owner | What changes |
|---|---|---|
| `crates/spec-spine-core/src/compile.rs` | 001 | `detect_duplicates` reads a character-safe numeric prefix |
| `crates/spec-spine-core/tests/compile.rs` | 001 | the regression guards for §3.1 to §3.3 |

This spec changes no stated requirement of spec 001, so it declares no `amends`
edge. Spec 001 §3.2 already defines the code as "`V-004` duplicate numeric
prefix (`NNN`) across different slugs"; the implementation simply never matched
that sentence. What lands here is conformance to the owning spec, which is the
one kind of change an implementing spec may make inside another's territory
without amending it.

## 3. Behavior

### 3.1 The numeric prefix is read by character, and only when numeric

`detect_duplicates` MUST compute a spec id's numeric prefix as its **leading run
of ASCII decimal digits**, iterating characters, never by byte index. This is
the rule spec 053 already established for `lint.rs::ordinal`, whose doc comment
names this very defect as the thing it does not inherit. The two readings of an
ordinal in this codebase are now the same reading.

For a conformant id the run is exactly the `NNN` that `V-012` requires, so
nothing about a well-formed corpus changes.

### 3.2 An id with no numeric prefix is not a V-004 candidate

An id whose leading digit run is empty MUST NOT participate in V-004 at all: it
has no numeric prefix, so it can share one with nothing.

This is what spec 001 §3.2 says, and the byte-slice implementation contradicted
it in a second, quieter way. Given the legal-on-disk ids `auth-login` and
`auth-logout`, the old code reported:

```
V-004 numeric prefix 'aut' is shared by 'auth-login' and 'auth-logout'
```

`aut` is not a numeric prefix, and the two specs do not collide. The diagnostic
was noise printed beside the `V-012` violations that were the real finding.
Silence is the honest answer when there is no ordinal, which is the same
argument spec 053 made for `ordinal` returning `None`.

### 3.3 What must keep working

V-004 MUST still fire for two distinct slugs sharing a numeric prefix
(`007-alpha` and `007-beta`), which is the check's entire purpose, and V-003
MUST still fire for a genuinely duplicated id. The regression guards in
`tests/compile.rs` cover the panic input, both shapes of id that carry no
ordinal (the one that straddles byte 3 and the one that lands exactly on it),
and the two pre-existing true positives, so a future refactor cannot quietly
reintroduce either half of the defect.

## 4. Out of scope

**Widening the id grammar.** `V-012` continues to require
`^[0-9]{3}-[a-z0-9]+(-[a-z0-9]+)*$`. This spec changes what happens on the way
to that refusal, never the refusal itself: a non-ASCII id is still an invalid
id, it is simply now *reported* as one. Whether the corpus should accept
non-numeric ids is a real question and a different spec's.

**Ordinals wider than three digits.** `lint.rs::ordinal` parses an arbitrary
digit run so that `1001-foo` sorts above `999-bar`, while `V-012` caps the id at
three. That inconsistency is real and predates this spec. Aligning them is a
grammar decision, and §4's first paragraph defers it for the same reason.

**Auditing the rest of the codebase for byte slicing.** This spec's change came
with a sweep of `&str` slicing across `spec-spine-core` and
`spec-spine-types`. Every other site indexes at an offset returned by `find`, or
behind `valid_id`'s guard that byte 3 is `-`, and is a character boundary by
construction. The sweep found one defect and it is fixed here; standing
enforcement (a lint, a clippy rule) is not proposed, because a rule that fires
on every safe `find`-derived slice would cost more than it catches.

## 5. Resolved decisions

**2026-09-08: compare digit runs as strings, not as parsed integers.**
`detect_duplicates` reports the prefix in its message, so it needs the text.
Parsing to `u64` and formatting back would turn `007` into `7` in the
diagnostic, and would make `07` and `007` collide as the same prefix when they
are different (malformed) directory names. The check is about a shared textual
`NNN`; the string is the right key. `lint.rs::ordinal` parses because it needs
to *order*, which is a different question about the same characters.

**2026-09-08: no `amends` edge on spec 001.** Considered and rejected: an
`amends` edge says the amending spec changes what the amended spec requires
(spec 040). Spec 001 §3.2 requires a numeric prefix check and is unchanged by
this work. Declaring `amends` here would record a contradiction that does not
exist and would make the ledger's amendment history less trustworthy, not more.

## Verification

Each line below is one command: spec 049 §3.2 makes a fence's body line a
command, so no line may depend on a variable another line set. The scratch
corpora are materialized at fixed paths for that reason.

Every assertion fails against pre-070 code. The first repro aborts at exit 101
with a char-boundary panic instead of exiting 1, and the second prints the
`V-004` line that §3.2 removes.

```verify:cli
# Self-contained: the assertions below drive the release binary.
cargo build --release --locked
# 3.3 the regression guards, which panic rather than fail against pre-070 code.
cargo test -p spec-spine-core --test compile --locked
# A corpus whose id puts a multi-byte character across byte 3.
rm -rf "${TMPDIR:-/tmp}/ss070a" && mkdir -p "${TMPDIR:-/tmp}/ss070a/specs/a日本-spec"
printf -- '---\nid: "a日本-spec"\ntitle: "t"\nstatus: draft\ncreated: "2026-09-08"\nsummary: "s"\n---\n\n# t\n' > "${TMPDIR:-/tmp}/ss070a/specs/a日本-spec/spec.md"
# 3.1 it is refused by the code that names the real malformation...
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss070a" compile 2>&1 | grep -q 'V-012'
# ...at exit 1, inside the stable contract, never the 101 a panic spends.
sh -c 'target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss070a" compile >/dev/null 2>&1; test $? -eq 1'
# A corpus of two legal-on-disk ids that share three leading letters.
rm -rf "${TMPDIR:-/tmp}/ss070b" && mkdir -p "${TMPDIR:-/tmp}/ss070b/specs/auth-login" "${TMPDIR:-/tmp}/ss070b/specs/auth-logout"
printf -- '---\nid: "auth-login"\ntitle: "t"\nstatus: draft\ncreated: "2026-09-08"\nsummary: "s"\n---\n\n# t\n' > "${TMPDIR:-/tmp}/ss070b/specs/auth-login/spec.md"
printf -- '---\nid: "auth-logout"\ntitle: "t"\nstatus: draft\ncreated: "2026-09-08"\nsummary: "s"\n---\n\n# t\n' > "${TMPDIR:-/tmp}/ss070b/specs/auth-logout/spec.md"
# 3.2 no ordinal means no V-004, and V-012 remains the finding.
! target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss070b" compile 2>&1 | grep -q 'V-004'
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss070b" compile 2>&1 | grep -q 'V-012'
# A corpus of two conformant ids that genuinely share NNN.
rm -rf "${TMPDIR:-/tmp}/ss070c" && mkdir -p "${TMPDIR:-/tmp}/ss070c/specs/007-alpha" "${TMPDIR:-/tmp}/ss070c/specs/007-beta"
printf -- '---\nid: "007-alpha"\ntitle: "t"\nstatus: draft\ncreated: "2026-09-08"\nsummary: "s"\n---\n\n# t\n' > "${TMPDIR:-/tmp}/ss070c/specs/007-alpha/spec.md"
printf -- '---\nid: "007-beta"\ntitle: "t"\nstatus: draft\ncreated: "2026-09-08"\nsummary: "s"\n---\n\n# t\n' > "${TMPDIR:-/tmp}/ss070c/specs/007-beta/spec.md"
# 3.3 the true positive still fires, naming the shared numeric prefix.
target/release/spec-spine --repo "${TMPDIR:-/tmp}/ss070c" compile 2>&1 | grep -q "numeric prefix '007' is shared"
rm -rf "${TMPDIR:-/tmp}/ss070a" "${TMPDIR:-/tmp}/ss070b" "${TMPDIR:-/tmp}/ss070c"
```
