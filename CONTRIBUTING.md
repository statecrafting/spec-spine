# Contributing to spec-spine

spec-spine governs itself: every change is judged by the gate it ships.

## The loop

`AGENTS.md` is the authority for how work is done here, for people and agents
alike. In short:

1. **A change has a spec.** Behavior changes are specified first: a new
   `specs/NNN-slug/spec.md` at the next free ordinal, born `status: draft`. It
   declares every edge it needs, including `amends` on any approved spec whose
   stated behavior it changes. An approved spec is never edited to make code
   pass.
2. **One spec per pull request.** Build on a branch named after the spec id.
3. **Run the gate before every commit:** `make gate`, then
   `cargo test --workspace --locked`,
   `cargo clippy --workspace --all-targets --locked -- -D warnings`, and
   `cargo fmt --all --check`. Commit the regenerated `.statecraft/derived/`
   shards with the change that made them stale.
4. **Ship, then ratify.** The build PR merges with the spec still `draft`;
   ratification (`draft` to `approved`) is a separate PR merged by a
   maintainer.

## What CI requires

`ci-gate` aggregates every job: the Linux build, test, clippy and format run;
the Windows test run (`test (windows)`, spec 134); a build and test at the
declared `rust-version` (spec 135); `cargo deny check` (spec 135);
self-governance; cross-triple determinism; and the AI review. Commits must be
signed.

## Things only a maintainer does

A `Spec-Drift-Waiver:` line in a PR body, ratification, and releases. See the
"Owner delegation" section of `AGENTS.md` once it lands.

## Reporting a vulnerability

Do not open a public issue. A private reporting channel is being set up and
will be named in `SECURITY.md`; until then, contact a maintainer privately
through GitHub.

## Toolchain

The channel is pinned in `rust-toolchain.toml`. Always pass `--locked`; the
committed `Cargo.lock` is part of the determinism contract.
