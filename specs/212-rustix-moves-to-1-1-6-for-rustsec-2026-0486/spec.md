---
id: "212-rustix-moves-to-1-1-6-for-rustsec-2026-0486"
title: "rustix moves to 1.1.6 for RUSTSEC-2026-0486"
status: draft
kind: "tooling"
created: "2026-11-27"
implementation: complete
owner: "The spec-spine Authors"
risk: low
summary: >
  `cargo audit` in the required `supply-chain` job refuses `main`'s locked
  tree: RUSTSEC-2026-0486 names `rustix` 1.1.4, which `tempfile` pulls in as
  a dev-dependency. Every pull request fails that job until the lock moves.
  This spec moves the lock to `rustix` 1.1.6, the semver-compatible fix, and
  moves the crate's `cargo vet` exemption to the same version, as spec 205
  did for its three crates.
extends:
  # 3.2: the rustix exemption moves to the new version; the counts do not change.
  - { spec: "200-the-dependency-policy-has-an-owner-and-records", unit: { kind: file, path: "supply-chain/config.toml" }, nature: additive }
references:
  - { unit: { kind: file, path: "Cargo.lock" }, role: "the locked tree the advisory is read against" }
---

# 212: rustix moves to 1.1.6 for RUSTSEC-2026-0486

## 1. Purpose

On 2026-11-26 `cargo audit` began refusing `main` with RUSTSEC-2026-0486
against `rustix` 1.1.4. The `supply-chain` job is required, so every open
pull request failed it, including two Dependabot groups and spec 211's
profile render. `rustix` reaches the tree only through `tempfile`, a
dev-dependency of `spec-spine-core`, `spec-spine-cli` and
`spec-spine-launcher`; no published binary links it.

## 2. Territory

- **`Cargo.lock`**: the one `rustix` entry.
- **`supply-chain/config.toml`** (200's): one exemption version.

## 3. Behavior

### 3.1 The lock

`Cargo.lock` carries `rustix` 1.1.6 and no other version of it. No
`Cargo.toml` changes: `tempfile`'s requirement already admits 1.1.6.
`cargo audit` reports no advisory against the locked tree.

### 3.2 The vet exemption follows the version

Spec 200 pins each exemption to a version, so the move would leave `rustix`
unvetted. `supply-chain/config.toml` exempts `rustix` at 1.1.6 instead of
1.1.4, at the same `safe-to-run` criteria. `cargo vet` reports 50 crates
fully audited and 125 exempted, the counts spec 200 measured.

## 4. Out of scope

- Replacing the `rustix` exemption with an audit (200 §4: each one removed is
  its own change).
- Any other lock movement. The Dependabot cargo group proposes its own.

## 5. Resolved decisions

**D-1 (2026-11-27): the exemption moves rather than an audit being added.**
The change is a patch release of a crate already trusted by exemption at the
same criteria; 205 §3.3 set the precedent for moving an exemption with its
version.

## Verification

```verify:cli
# 3.1: the lock carries rustix 1.1.6 only.
test "$(grep -c '^name = "rustix"$' Cargo.lock)" = 1
grep -A1 '^name = "rustix"$' Cargo.lock | grep -qxF 'version = "1.1.6"'
# 3.2: the exemption names the new version at the same criteria.
grep -A2 '^\[\[exemptions\.rustix\]\]$' supply-chain/config.toml | grep -qxF 'version = "1.1.6"'
grep -A2 '^\[\[exemptions\.rustix\]\]$' supply-chain/config.toml | grep -qxF 'criteria = "safe-to-run"'
```
