---
id: "200-the-dependency-policy-has-an-owner-and-records"
title: "The dependency policy has an owner, and its audit and vet records are committed"
status: approved
kind: "tooling"
created: "2026-10-21"
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "135-the-toolchain-floor-and-dependencies-are-checked"
summary: >
  `deny.toml` is the dependency gate CI runs (spec 135), and no spec owns it:
  135 names it only as a reference. This spec claims it, adds
  `.cargo/audit.toml` so `cargo audit` reads the same policy, and commits the
  `cargo vet` records under `supply-chain/` with six published audit sets
  imported. Each file enters the content hash and the governed scope, so an
  edit to the dependency policy stales the ledger and needs an owner.
establishes:
  - "deny.toml"
  - ".cargo/audit.toml"
  - "supply-chain/"
extends:
  # [index] extra_hashed_inputs and [coverage] governed_scope gain the four
  # policy paths (L-008: a claimed file must be in some content hash).
  - { spec: "061-shipped-is-not-the-same-as-working", unit: { kind: file, path: "spec-spine.toml" }, nature: additive }
references:
  - { unit: { kind: file, path: "Cargo.lock" }, role: "the locked tree both records describe" }
  - { unit: { kind: file, path: ".github/workflows/spec-spine-required.yml" }, role: "runs cargo-deny against deny.toml (managed; unchanged)" }
---

# 200: The dependency policy has an owner, and its audit and vet records are committed

## 1. Purpose

Spec 135 added `cargo deny check` to CI against a committed `deny.toml`, and
listed `deny.toml` under `references`, which owns nothing. The file that
decides which licenses, sources and advisories this repository accepts was
therefore unowned and outside the content hash: an edit to it staled no shard
and needed no spec.

Two further records were missing:

- `cargo audit` had no configuration, so a contributor running it locally got
  the tool's defaults rather than this repository's policy.
- No `cargo vet` record existed, so there was no committed statement of which
  dependencies a published audit covers and which are trusted by exemption.

## 2. Territory

- **`deny.toml`**: claimed as it stands. Its content does not change here.
- **`.cargo/audit.toml`**: new.
- **`supply-chain/`**: new; `config.toml`, `audits.toml` and `imports.lock`
  as `cargo vet` writes them.
- **`spec-spine.toml`** (additive): four entries in each of `[index]
  extra_hashed_inputs` and `[coverage] governed_scope`.

## 3. Behavior

### 3.1 `deny.toml` stays the gate

The managed `spec-spine-required.yml` keeps running `cargo deny check`
against `deny.toml`. Nothing here changes what CI refuses.

### 3.2 `cargo audit` reads the same policy

`.cargo/audit.toml` ignores no advisory, denies `unsound` and `yanked`
findings, and reports `unmaintained` as a warning, which matches
`deny.toml` (`yanked = "deny"`, no ignore list). An advisory ignored in one
file is ignored in the other in the same change, with its id and reason
beside it in both.

### 3.3 `cargo vet` records

`supply-chain/config.toml` imports the published audit sets of the Bytecode
Alliance, Embark Studios, Google, ISRG, Mozilla and Zcash. Every dependency
those sets do not cover carries an exemption written by `cargo vet init` and
reduced by `cargo vet prune`. Measured on 2026-10-21 against the locked tree:
50 crates fully audited, 125 exempted.

The three published workspace crates (`spec-spine-cli`, `spec-spine-core`,
`spec-spine-types`) are first-party (`audit-as-crates-io = false`), with no
exemption pinned to a version. An exemption pinned to `0.31.0` would fail
`cargo vet` on the next version bump, and `scripts/bump_version.py` would have
to learn about `supply-chain/` to avoid that. The fourth member,
`spec-spine-launcher`, sets `publish = false` and is not on crates.io, so
`cargo vet` already treats it as first-party and it needs no policy entry.

### 3.4 Each file is hashed and governed

The four paths are in `[index] extra_hashed_inputs`, so an edit restales the
inputs record, and in `[coverage] governed_scope`, so an unclaimed change to
one is a `C-002` refusal. This spec's `establishes` is the claim.

## 4. Out of scope

- Running `cargo vet` or `cargo audit` in CI. A required job belongs in the
  managed workflow, which changes only through the Statecraft profile.
- Replacing exemptions with audits. Each one removed is its own change.
- Any edit to `deny.toml`'s policy.

## 5. Resolved decisions

**2026-10-27: ratified by the owner.** The owner confirmed every decision
below as written and directed ratification in this pull request; the owner
exception approved on its run is the ratification of record.

**D-1 (2026-10-21): workspace crates are first-party for `cargo vet`.**
`cargo vet init` marks the three published workspace crates
`audit-as-crates-io = true` and exempts each at `0.31.0`. That makes every
release bump a `cargo vet` failure. They are this repository's own source and
are reviewed through its own gate, so they are first-party.

**D-2 (2026-10-21): this spec claims `deny.toml`.** Spec 135 is approved and
can claim nothing new without an amendment, and claiming a file is not an
amendment of 135's behavior. The owner's queue named 135 as the likely owner;
the ownership query showed no owner at all.

## Verification

```verify:cli
# 3.2: the audit policy exists and ignores nothing.
test -f .cargo/audit.toml
grep -qE '^ignore = \[\]$' .cargo/audit.toml
grep -qE '^deny = \["unsound", "yanked"\]$' .cargo/audit.toml
sh -c '! grep -qE "^[[:space:]]*ignore[[:space:]]*=[[:space:]]*\[[^]]" deny.toml'
# 3.3: the vet records exist, import the six sets, and pin no first-party exemption.
test -f supply-chain/config.toml
test -f supply-chain/audits.toml
test -f supply-chain/imports.lock
test "$(grep -c '^\[imports\.' supply-chain/config.toml)" = 6
test "$(grep -c '^audit-as-crates-io = false$' supply-chain/config.toml)" = 3
sh -c '! grep -q "^\[\[exemptions\.spec-spine-" supply-chain/config.toml'
# 3.4: each path is hashed and governed.
grep -qF '"deny.toml",' spec-spine.toml
grep -qF '".cargo/audit.toml",' spec-spine.toml
grep -qF '"supply-chain/*.toml",' spec-spine.toml
test "$(grep -cF '"supply-chain/imports.lock",' spec-spine.toml)" = 2
```
