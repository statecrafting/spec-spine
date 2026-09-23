---
id: "115-bindings-are-designed-not-shipped"
title: "Bindings are designed, not shipped"
status: draft
kind: "governance"
created: "2026-09-21"
implementation: n-a
owner: "The spec-spine Authors"
depends_on:
  - "001-compile-registry"
summary: >
  `docs/bindings-plan.md` designs napi, pyo3 and cgo bindings and says no
  binding code belongs in this repository. That mandate lived only in a
  document header. This records it in the corpus as a policy, and states the
  obligations the JSON facade keeps so that a binding remains buildable
  elsewhere, each as it measurably holds today. It builds nothing, endorses
  no external binding, and is reviewable rather than enforced. Its
  implementation is `n-a`: there is no binding to implement, and this spec is
  the decision not to.
establishes:
  # The policy's own document. A claimed governance file, so L-008 puts it in
  # a content hash.
  - { kind: file, path: "docs/bindings-plan.md" }
extends:
  # D-2: the claim puts the document into `[index] extra_hashed_inputs`.
  - { spec: "092-the-engine-ships-governance-not-an-environment", unit: { kind: file, path: "spec-spine.toml" }, nature: additive }
references:
  - unit: { kind: file, path: "crates/spec-spine-core/src/lib.rs" }
    role: "context"
  - unit: { kind: file, path: "docs/api.md" }
    role: "context"
obligations:
  - id: "R-1"
    kind: requirement
    text: "This repository contains no binding crate, binding build script or workspace member whose purpose is to expose the library to another runtime."
    anchor: "3-1-no-binding-code-in-this-repository"
  - id: "R-2"
    kind: requirement
    text: "A change that breaks a facade obligation declares that it does, in the spec making it, whether or not a binding exists."
    anchor: "3-2-the-facade-is-the-seam-and-what-that-obliges"
  - id: "V-1"
    kind: verification
    text: "The existing evidence for the testable obligations: every emitted document validates against its versioned schema, and the error-kind set is closed with its exit-code mapping."
    anchor: "verification"
    inputs:
      - "crates/spec-spine-core/tests/conformance.rs"
      - "crates/spec-spine-types/src/verdict.rs"
---

# 115: Bindings are designed, not shipped

## 1. Purpose

`docs/bindings-plan.md` designs napi, pyo3 and cgo bindings and opens by
saying no binding code belongs in this repository. That is a real decision,
and until this spec it lived only in a document header, so it bound a reader
and nothing else.

This spec's deferral is the decision itself. A consumer naming a need does
not lift it, and design note 09 D-7 (opportunity-led expansion) does not
change it (note 09 §10.2, row P10). What a consumer gets is a maintained seam,
not a binding. Whether an external binding is built, and in which repository,
is a separate product decision this spec does not make.

### 1.1 Why the dependency is spec 001

The seam is the JSON facade in `crates/spec-spine-core/src/lib.rs`, which
spec 001 owns. Every obligation in §3.2 is on that file's public surface.

## 2. Territory

`docs/bindings-plan.md`, the design document whose header this spec turns
into policy. No code: by §3.1, this is the spec that says none is added. The
facade file is referenced, not claimed; its owners stay its owners, and this
spec adds no coupling obligation to every edit of it.

## 3. Behavior

### 3.1 No binding code in this repository

This repository MUST NOT contain a napi, pyo3, cgo, wasm-bindgen or
equivalent binding crate, a build script that produces one, or a workspace
member whose purpose is to expose the library to another runtime.

A binding is a distribution surface with its own release cadence, platform
matrix, toolchain and breakage modes. Carrying one here makes every engine
release a binding release, and makes the four-triple determinism gate a
cross-language problem.

The npm and PyPI directories are **not** bindings and are unaffected. They
ship the prebuilt CLI binary per platform (specs 006, 007) and expose no
library surface.

### 3.2 The facade is the seam, and what that obliges

A binding built elsewhere wraps the JSON-in / JSON-out facade. The
obligations that keep it buildable, as measured on 2026-09-23 over the 27
public `*_json` functions in `lib.rs`:

1. **Plain values in, a string out.** Every facade entry returns
   `Result<String, Error>` and takes only plain scalar arguments: `&str`, and
   in exactly one case a `bool` (`attest_json`'s `with_coupling`). No
   lifetimes beyond the borrowed `&str`, no generics, no trait objects, no
   callbacks at the boundary.
2. **Additive only.** A facade request gains optional members; it does not
   gain required ones, lose members or change a member's type. Spec 100 §3.8
   is the worked example: `couple_json` gained `priorRoots` as optional, and
   absent behaves exactly as before.
3. **Owned, serde DTOs.** The types crate stays plain data, so the same types
   back both the engine and a binding.
4. **Versioned answers.** Every read document and verdict carries a
   `schemaVersion`, so a binding pins a contract rather than a build.
5. **One error mapping.** `Error` and its exit codes are the whole failure
   surface, and the verdict envelope's `error.kind` set is closed and
   versioned, so a binding branches on a token.

A change that breaks any of the five is a change to what a binding can be
built against, and MUST be declared as such in the spec making it, whether or
not a binding exists.

### 3.3 The mandate is reviewable, not enforced

This spec declares no gate. Nothing refuses a binding crate by directory
name, which would be a blunt instrument for a decision a reviewer can make,
and an assertion that a directory nobody created is absent would pass on the
day it was written and every day after.

What the spec buys is that the decision is **in the corpus**: a pull request
adding a binding crate contradicts a spec rather than a document header, and
the contradiction is what a reviewer cites.

### 3.4 What this establishes, and what it does not

It establishes that this repository has decided not to ship bindings, and
what it maintains instead. It does not establish that a binding built
elsewhere is correct, supported or tested against this engine, and it does
not claim any binding is implemented. `docs/bindings-plan.md` is a design,
not a contract with an implementer.

## 4. Out of scope

- **Writing a binding.** §3.1.
- **Any enforcement.** §3.3.
- **Endorsing, supporting or locating an external binding.** §1, §3.4.
- **Changing the facade.** This spec constrains how it may change.
- **npm and PyPI.** §3.1.

## 5. Resolved decisions

**D-1 (2026-09-23, finalized: the lifecycle representation).** The reserved
draft was `implementation: deferred`, which would read as "a build is owed
and not scheduled". Nothing is owed: the spec is a decision and a set of
obligations on an existing surface. `n-a` is the corpus's value for a record
that has nothing to implement (`standards/spec/contract.md`), and it keeps the
spec out of the ready set. `complete` would imply a binding implementation
exists, which it does not.

**D-2 (2026-09-23, finalized: territory).** A spec with no ownership edge
raises `L-001`, and `n-a` is not exempt (spec 116 §3.3). The truthful claim is
the document this spec governs, `docs/bindings-plan.md`, which had no owner.
Claiming `lib.rs` through `constrains` was rejected: it would make this spec
an owner of every facade edit for the coupling gate, which is enforcement
this spec disclaims (§3.3).

**D-3 (2026-09-23, finalized: the first obligation as measured).** The draft
stated every facade entry as `&str -> Result<String, Error>` and said all five
obligations "hold today". Measured, `attest_json` takes a `bool`. A `bool`
crosses every binding layer the plan names, so the obligation is restated as
plain scalar arguments rather than the facade being changed to match a
sentence.

**D-4 (2026-09-23, finalized: the verification is existing evidence).** The
draft declared no verification block. A ratified `n-a` spec is judged by the
release sweep, where an undeclared block is not an acceptable outcome, and
the obligations with testable content already have tests. The block below
runs them. It asserts no absence (§3.3) and adds no test of its own; it is
the evidence the obligations already stand on, and it fails if that evidence
does.

## Verification

Obligations 4 and 5 are the testable ones. Obligations 1 to 3 are properties
of the surface's shape that a reviewer checks on a change (§3.2's last
paragraph), and `unsafe_code = "forbid"` plus the types crate's owned DTOs are
already held by the workspace lints and the DTO suite.

```verify:cli
# 3.2.4: every emitted document validates against the embedded schema of the
# version it declares.
cargo test -p spec-spine-core --test conformance --locked
# 3.2.5: the error-kind token set is closed (an exhaustive match that fails
# the build on a new variant), and each kind carries its own exit code.
cargo test -p spec-spine-types --lib --locked verdict::
```
