---
id: "NNN-slug"                 # MUST equal the directory name; NNN = unique 3-digit ordinal
title: "Short imperative title"
status: draft                  # draft | approved | superseded | retired
created: "YYYY-MM-DD"
summary: >
  One short paragraph: what territory this spec claims and why it exists.
# --- optional descriptive keys ---
# owner: "name"
# authors: ["name"]
# risk: medium                 # low | medium | high | critical
implementation: pending        # pending | in-progress | complete | n-a | deferred
# depends_on: ["NNN-other"]
# domain: "..."                # only if domains.allowed is configured non-empty
# kind: "..."                  # only if kind.allowed is configured non-empty
# --- accepted, but read by nothing today ---
# Both keys parse, validate and compile to the registry record, and no verb
# consumes either. Write one only to carry a fact for a consumer you are also
# writing; giving them meaning in the tool is a deliberate decision, not an
# accretion (docs/design/02-agentic-builder-substrate.md).
# code_aliases: ["legacy-name"]      # other names this spec's code has gone by
# feature_branch: "NNN-slug"         # the branch this spec is built on
# --- typed edges (declare territory + relationships) ---
# Eight edges. `references` is the only non-owning one: the coupling gate
# ignores it, so it names context without claiming it.
# establishes:
#   - { kind: file, path: "src/thing.rs" }
#   - "src/whole_file.rs"      # bare string == { kind: file, path: ... }
#   - "src/subtree/"           # trailing slash == the subtree rooted there
#   - { kind: symbol, id: "crate::module::function" }
#   - { kind: section, file: "Makefile", anchor: "build-target" }
#   - { kind: directory, path: "crates/my-crate/" }
#   - { kind: crate, id: "my-crate" }
#   - { kind: module, id: "my_crate::serialization" }
#   # `planned: true` on any unit declares territory this spec owns and has not
#   # written yet. A declared state, not a diagnostic: an unresolved planned
#   # unit raises no `W-001`, so a corpus running `check --fail-on-unresolved`
#   # can still carry a spec mid-build. Omit it once the file exists; a written
#   # `planned: false` normalizes to absent.
#   - { kind: file, path: "src/not_written_yet.rs", planned: true }
# extends:
#   # Adds surface to a predecessor. The unit does NOT have to appear in the
#   # target spec's territory: an extends-carried unit is a first-class claim,
#   # and it is how a spec touches a file another spec owns without amending it.
#   - { spec: "NNN-predecessor", unit: { kind: file, path: "src/added.rs" }, nature: additive }
#   - { spec: "NNN-predecessor", paths: ["a.rs", "b.rs"] }   # sugar for N file units
# refines:
#   - { aspect: "error-handling", unit: { kind: symbol, id: "crate::run" } }
# supersedes: ["NNN-predecessor"]
# amends: ["NNN-predecessor"]
# co_authority:
#   - { unit: { kind: section, file: "Cargo.toml", anchor: "workspace-deps" }, with_specs: ["NNN-other"] }
# constrains:
#   - { unit: { kind: file, path: "src/api.rs" }, note: "public API is frozen" }
# references:
#   - { unit: { kind: file, path: "docs/notes.md" }, role: "context" }
# --- lifecycle / amendment (as applicable) ---
# superseded_by: "NNN-successor"     # required when status: superseded
# retirement_rationale: "why"        # required when status: retired
# amends_sections: ["anchor"]        # which anchors of the amended spec change
# unamendable: ["anchor"]            # anchors of THIS spec no later spec may amend
# amendment_record: "NNN-successor"  # a spec that records amendments to this
#                                    # one; the coupling gate adds it to this
#                                    # spec.md's owner set, so editing this file
#                                    # clears against that spec too.
#
# `amends_verification`: the amended specs whose `## Verification` block THIS
# spec's block replaces, so `spec-spine verify <amended-id>` runs the commands
# below instead of the ones in the amended file (spec 082 §3.2).
#
# Reach for it when an approved spec's acceptance line has gone wrong: it
# asserts more than the spec requires, or it pinned an output a later approved
# spec legitimately moved. Spec 037 forbids editing the amended file and
# `verify` executes that file, so a replacement declared here is the only route
# to a red block on an approved spec.
#
#   - Every entry MUST also appear in `amends` (`V-018`). Replacing what a spec
#     accepts is an amendment of it, so the edge must be there to be read.
#   - Two live specs naming the same id is `V-019`, and a cycle is `V-020`. The
#     corpus refuses rather than picking one; a human decides which stands.
#   - Resolution follows the chain, and skips a `superseded` or `retired`
#     amender.
#   - `verify` prints which spec's block it ran, so the substitution is never
#     silent. `registry show <id> --json` carries `amendsVerification`.
#   - Carry the amended block in FULL with the defect corrected, not a patch.
#     A reader asking what that spec accepts today should find one block that
#     answers.
#   - Assert, in the replacement, that the amended file still carries the
#     superseded form. The line then goes red if anyone ever resolves this by
#     editing the amended spec instead.
#
# amends: ["NNN-predecessor"]
# amends_verification: ["NNN-predecessor"]
# --- declared constraints (spec 106) ---
# `obligations` names this spec's requirements so they can be cited as
# `<spec-id>#<id>` instead of by a section number that moves. Optional; a spec
# may declare none. Each entry has exactly these members:
#   - `id`: unique within this spec, `^[A-Za-z][A-Za-z0-9]*(-[A-Za-z0-9]+)*$`
#     (so never a `#`). A bad or duplicate id is `V-021`.
#   - `kind`: `requirement`, `invariant` or `verification`. There is no fourth.
#   - `text`: the one sentence the obligation asserts. Empty text is `V-024`.
#   - `anchor`: the slug of a heading in THIS spec's body (the anchor
#     `index owner` computes for a section). It must name exactly one heading;
#     a dangling or ambiguous anchor is `V-022`.
#   - `inputs`: required on a `verification` (test files or commands), and
#     forbidden on every other kind (`V-023`). Declared, never inferred.
#   - `withdrawn: true`: retire an obligation IN PLACE, keeping its id, rather
#     than deleting it, so the id can never be reused for something else.
# An obligation declares; it does not prove its verification passes, and the
# list is what the author wrote, not a proof nothing else is required. No gate
# reads it. `spec-spine registry obligation <spec-id>#<id>` resolves one.
# obligations:
#   - { id: "R-1", kind: requirement, text: "The rule holds.", anchor: "3-1-the-rule" }
#   - { id: "V-1", kind: verification, text: "It is checked.", anchor: "verification", inputs: ["tests/rule.rs"] }
# --- declared impact and conflict (spec 109) ---
# `impacts` and `conflicts` name this spec's relation to another spec's
# obligation (spec 106): a qualified `<spec-id>#<obligation-id>` reference,
# which MUST resolve (an unqualified reference is `V-025`, never resolved
# locally, and a dangling or self-targeting one is `V-028`/`V-029`). Both are
# optional; a spec may declare neither. Nothing computes either, and no gate
# reads them: they record what the author said, not a proof it is complete or
# correct.
#   - `impacts[].nature`: `refines`, `extends`, `supersedes` or `informs`.
#     `supersedes` MUST name, in `successor`, a non-withdrawn obligation THIS
#     spec declares (`V-026`); `successor` on any other nature is refused.
#   - `conflicts[].reason`: non-empty, why the divergence exists (`V-027`).
#   - `conflicts[].resolution`: `deliberate`, `unresolved` or `pending`.
#     `unresolved` is reported by `lint` (`L-014`), once per entry, warning
#     tier. `settled_by`, a spec id, is required exactly when `resolution` is
#     `pending` and refused otherwise (`V-031`).
# The same obligation named twice in one spec's `impacts`, or twice in its
# `conflicts`, is refused (`V-030`). `spec-spine registry impacts [--target
# <ref>] [--declared-by <spec>]` inverts every declaration in the corpus.
# impacts:
#   - { obligation: "NNN-other#R-1", nature: refines, note: "why" }
#   - { obligation: "NNN-other#R-2", nature: supersedes, successor: "R-4" }
# conflicts:
#   - { obligation: "NNN-other#R-3", reason: "why", resolution: deliberate }
#   - { obligation: "NNN-other#R-5", reason: "why", resolution: pending, settled_by: "NNN-later" }
# --- bootstrap marker (NOT an edge) ---
# `origin.retroactive` declares authority held since before the graph existed:
# code that predates its governing spec is evidence, not a violation, and a
# spec claiming it says so rather than posing as a fresh `establishes`.
# origin:
#   retroactive: true
#   paths: ["src/"]
---

# NNN: Title

## 1. Purpose

What problem this spec solves and what it owns.

## 2. Territory

The units this spec claims authority over (mirrors the frontmatter edges, in prose).

## 3. Behavior

What the governed code must do. Use MUST/SHOULD/MAY.

## 4. Out of scope

What this spec deliberately does not cover.

## 5. Resolved decisions

Dated entries (`D-1 (YYYY-MM-DD, what was decided)`) for choices this spec was
silent on and a build had to make. Recording one is always legitimate;
changing what the spec *requires* mid-build is not.

## Verification

The acceptance, as commands. `spec-spine verify <id>` runs this block; each
line is one command and no shell variable survives to the next. Write lines
that fail against the tree this spec is built on and pass after: a block that
is green before the work asserts nothing.

```verify:cli
# one command per line
```
