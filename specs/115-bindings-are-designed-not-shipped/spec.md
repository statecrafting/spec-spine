---
id: "115-bindings-are-designed-not-shipped"
title: "Bindings are designed, not shipped"
status: draft
kind: "governance"
created: "2026-09-21"
implementation: deferred
owner: "The spec-spine Authors"
depends_on:
  - "001-compile-registry"
summary: >
  `docs/bindings-plan.md` is design-only by its own header: no binding code in
  this repository. That mandate is not in the corpus, so nothing refuses a
  binding crate landing here. This states the mandate as a contract, and states
  the obligations the facade keeps so a binding remains buildable elsewhere.
  Deferred by mandate: this is the one item whose deferral is the decision.
references:
  - unit: { kind: file, path: "docs/bindings-plan.md" }
    role: "context"
  - unit: { kind: file, path: "crates/spec-spine-core/src/lib.rs" }
    role: "context"
---

# 115: Bindings are designed, not shipped

## 1. Purpose

`docs/bindings-plan.md` designs napi, pyo3 and cgo bindings and opens by
saying no binding code belongs in this repository. That is a real decision and
it lives only in a document header, so it binds a reader and nothing else.

Every other deferred item in this wave is deferred **pending a consumer**.
This one is different: its deferral is the decision, and a consumer naming a
need does not lift it. What a consumer gets is a maintained seam, not a
binding.

### 1.1 Why the dependency is spec 001

The seam is the JSON facade in `crates/spec-spine-core/src/lib.rs`, which
spec 001 owns. That is the semantic dependency: everything this spec requires
is an obligation on that file's public surface.

## 2. Territory

None, on spec 106 §2's terms. This spec adds no code and, by §3.1, is the one
that says no code is added.

## 3. Behavior

### 3.1 No binding code in this repository

This repository MUST NOT contain a napi, pyo3, cgo, wasm-bindgen or
equivalent binding crate, a build script that produces one, or a workspace
member whose purpose is to expose the library to another runtime.

The reason is the one spec 092 gives for the initializer: a binding is a
distribution surface with its own release cadence, its own platform matrix,
its own toolchain and its own breakage modes. Carrying one here makes every
release of the engine a release of the binding, and makes the determinism
gate's four-triple matrix a cross-language problem.

The npm and PyPI directories are **not** bindings and are unaffected. They
ship the prebuilt CLI binary per platform (specs 006, 007); they expose no
library surface.

### 3.2 The facade is the seam, and what that obliges

A binding built elsewhere wraps the JSON-in / JSON-out facade. The obligations
that keep it buildable, all of which hold today:

1. **`&str -> Result<String, Error>`.** Every facade entry keeps that shape.
   No lifetimes, no generics, no trait objects at the boundary.
2. **Additive only.** A facade request gains optional members; it does not
   gain required ones, lose members or change a member's type. Spec 100 §3.8
   is the worked example: `couple_json` gained `priorRoots` as optional and
   absent behaves exactly as before.
3. **Owned, serde DTOs.** The types crate stays plain data, which is what
   makes the same types back both the engine and a future binding.
4. **Versioned answers.** Every read document and every verdict carries a
   `schemaVersion`, so a binding pins a contract rather than a build.
5. **One error mapping.** `Error` and its exit codes are the whole failure
   surface, and the `error.kind` set in the verdict envelope is closed and
   versioned, so a binding branches on a token.

A change that breaks any of the five is a change to what a binding can be
built against, and MUST be declared as such in the spec making it, whether or
not a binding exists.

### 3.3 The mandate is reviewable, not enforced

This spec declares no gate. There is no check that refuses a binding crate,
and adding one would mean the corpus refusing a directory by name, which is a
blunt instrument for a decision a reviewer can make.

What the spec buys is that the decision is **in the corpus**: a pull request
adding a binding crate is now contradicting an approved spec rather than a
document's header, and the contradiction is the thing a reviewer cites.

### 3.4 What this establishes, and what it does not

It establishes that this repository has decided not to ship bindings, and
what it maintains instead.

It does not establish that a binding built elsewhere is correct, supported, or
tested against this engine. `docs/bindings-plan.md` is a design, not a
contract with an implementer, and no binding is endorsed by this repository.

## 4. Out of scope

- **Writing a binding.** §3.1.
- **Any enforcement.** §3.3.
- **Endorsing or supporting an external binding.** §3.4.
- **Changing the facade.** This spec constrains how it may change; it does not
  change it.
- **npm and PyPI.** §3.1: they are binary distribution, not bindings.

## 5. Open design questions

1. **Does the fixture set generalize?** Spec 103 emits verifier fixtures a
   consumer in any language can test against. The same idea for the facade,
   a set of request and response pairs, would let an external binding prove it
   wraps the seam correctly. That is the smallest useful thing this repository
   could offer a binding author and it is not designed here.
2. **Where would a binding live?** A sibling repository is the obvious answer
   and it needs an owner, which is a family decision rather than this
   repository's.

## Acceptance when built

There is nothing to build, which is the point. No `verify:cli` block: an
acceptance block here would assert the absence of code, and a `test ! -d`
against a directory nobody created passes on the day it is written and every
day after, which is the vacuous pass this repository has met before.

The obligations in §3.2 are the ones with real content, and each is already
asserted by an existing test of the spec that owns it: the facade's shape by
`docs/api.md`'s binding-readiness section and the conformance suite, the
additive rule by each schema's MINOR policy, the closed `error.kind` set by
the exhaustive match in `verdict.rs` that fails the build when a variant is
added. This spec adds no assertion because it adds no behavior; it records a
decision that was previously only in a document header.
