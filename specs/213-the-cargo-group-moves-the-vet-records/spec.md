---
id: "213-the-cargo-group-moves-the-vet-records"
title: "The cargo group moves the vet records and 205's pin"
status: draft
kind: "tooling"
created: "2026-11-27"
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "205-the-yaml-reader-takes-serde-saphyr-2"
summary: >
  Dependabot's cargo group (#537) moved `serde-saphyr` and `granit-parser` to
  2.2.0, `tree-sitter` to 0.27.2, and `clap`, `clap_derive`, `syn`,
  `proc-macro2` and `quote` by patch releases. It merged without moving the
  `cargo vet` records, so `cargo vet --locked` refused eight crates and the
  required `supply-chain` job failed on `main` and on every pull request. It
  also moved the exact `serde-saphyr` pin spec 205 asserts, so 205's
  acceptance failed at its first command. This spec moves the exemptions,
  refreshes the imported audits, and carries 205's acceptance at 2.2.0.
amends:
  # 205 3.1 pinned 2.0.0; 3.1 below records the pin at 2.2.0.
  - "205-the-yaml-reader-takes-serde-saphyr-2"
amends_sections: ["3"]
amends_verification:
  - "205-the-yaml-reader-takes-serde-saphyr-2"
extends:
  # 3.2: the exemptions follow the versions and the imported audits are refreshed.
  - { spec: "200-the-dependency-policy-has-an-owner-and-records", unit: { kind: file, path: "supply-chain/config.toml" }, nature: additive }
  - { spec: "200-the-dependency-policy-has-an-owner-and-records", unit: { kind: file, path: "supply-chain/imports.lock" }, nature: additive }
references:
  - { unit: { kind: file, path: "Cargo.toml" }, role: "the workspace pins #537 moved" }
  - { unit: { kind: file, path: "Cargo.lock" }, role: "the locked tree the records describe" }
  - { unit: { kind: file, path: "crates/spec-spine-types/src/yaml.rs" }, role: "196's reader, unchanged" }
---

# 213: The cargo group moves the vet records and 205's pin

## 1. Purpose

#537 merged on 2026-11-27 with these lock moves:

| Crate | From | To |
|---|---|---|
| `serde-saphyr` (exact pin) | 2.0.0 | 2.2.0 |
| `granit-parser` | 2.0.0 | 2.2.0 |
| `tree-sitter` (exact pin) | 0.27.1 | 0.27.2 |
| `clap`, `clap_derive` | 4.6.7 | 4.6.9 |
| `syn` (3.x) | 3.0.4 | 3.0.10 |
| `proc-macro2` | 1.0.106 | 1.0.107 |
| `quote` | 1.0.45 | 1.0.47 |

Spec 200 pins each `cargo vet` exemption to a version, and the imported
audits covering `proc-macro2` and `quote` named the old versions. `cargo vet
--locked` therefore refused all eight, and the required `supply-chain` job
failed on `main` and on every pull request opened after it. 205's acceptance
asserts the `=2.0.0` pin, so `spec-spine verify 205` failed at command 1.

## 2. Territory

- **`supply-chain/config.toml`** and **`supply-chain/imports.lock`** (200's).
- **`specs/205-the-yaml-reader-takes-serde-saphyr-2/spec.md`**: its
  Verification block is labeled superseded; no command changes.

## 3. Behavior

### 3.1 The pin is 2.2.0 (amends 205 3.1)

The workspace pins `serde-saphyr = "=2.2.0"`, default features off and
`deserialize` on, with `granit-parser` 2.2.0 under it. 205 3.2 is unchanged:
the reader sets `NonFiniteFloatPolicy::Reject`, and every row of
`yaml_parity.rs` and `yaml_workflow_parity.rs` holds without an edit.

### 3.2 The vet records follow the versions (amends 205 3.3)

`supply-chain/config.toml` moves six exemptions to the locked versions, at
unchanged criteria: `clap` and `clap_derive` 4.6.9, `granit-parser` and
`serde-saphyr` 2.2.0, `syn` 3.0.10, `tree-sitter` 0.27.2. `proc-macro2`
1.0.107 gains a `safe-to-deploy` exemption, because no imported audit covers
it. `supply-chain/imports.lock` is refreshed from the six imported sets,
which brings Mozilla's audit of `quote` 1.0.47. `cargo vet --locked` reports
51 crates fully audited and 124 exempted.

## 4. Out of scope

- Removing an exemption the refreshed imports now make redundant (200 §4:
  each one removed is its own change). The `strum` and `strum_macros`
  exemptions stay.
- Replacing any exemption with a local audit.

## 5. Resolved decisions

**D-1 (2026-11-27): the exemptions move rather than audits being
certified.** 205 3.3 set the precedent: a version move of a crate already
trusted by exemption moves the exemption at the same criteria. `cargo vet`
estimates a 14,560-line audit backlog for these deltas, most of it the
`serde-saphyr` and `granit-parser` 2.0 to 2.2 diffs.

**D-2 (2026-11-27): this spec follows the change it records.** #537 moved
the pins and the lock; this spec moves the records and the acceptance that
#537 left behind, so the `serde-saphyr` and `tree-sitter` assertions below
pass on the tree it lands on. The exemption assertions fail on `75fc45ba`,
the `main` #537 produced.

## Verification

```verify:cli
# ---- carried for 205-the-yaml-reader-takes-serde-saphyr-2 (amends_verification); the pin and the three exemptions move to 2.2.0 (213 3.1, 3.2) ----
# 3.1: the pin is exact at 2.2.0 and the locked tree carries 2.2.0 only.
grep -qF 'serde-saphyr = { version = "=2.2.0", default-features = false, features = ["deserialize"] }' Cargo.toml
test "$(grep -c '^name = "serde-saphyr"$' Cargo.lock)" = 1
grep -A1 '^name = "serde-saphyr"$' Cargo.lock | grep -qxF 'version = "2.2.0"'
# 3.2: the reader sets the policy, not the deprecated boolean.
grep -qF 'options.non_finite_float_policy = serde_saphyr::NonFiniteFloatPolicy::Reject;' crates/spec-spine-types/src/yaml.rs
sh -c '! grep -qF reject_non_finite_typeless_float crates/spec-spine-types/src/yaml.rs'
cargo test -p spec-spine-types --locked --test yaml_parity
# 3.3: the exemptions name the new versions.
grep -A1 '^\[\[exemptions\.serde-saphyr\]\]$' supply-chain/config.toml | grep -qxF 'version = "2.2.0"'
grep -A1 '^\[\[exemptions\.granit-parser\]\]$' supply-chain/config.toml | grep -qxF 'version = "2.2.0"'
grep -A1 '^\[\[exemptions\.sha2\]\]$' supply-chain/config.toml | grep -qxF 'version = "0.11.1"'
# ---- 213 ----
# 3.2: the other exemptions name the locked versions (the supply-chain job runs cargo vet itself).
grep -A1 '^\[\[exemptions\.tree-sitter\]\]$' supply-chain/config.toml | grep -qxF 'version = "0.27.2"'
grep -A1 '^\[\[exemptions\.clap\]\]$' supply-chain/config.toml | grep -qxF 'version = "4.6.9"'
grep -A1 '^\[\[exemptions\.clap_derive\]\]$' supply-chain/config.toml | grep -qxF 'version = "4.6.9"'
grep -A1 '^\[\[exemptions\.syn\]\]$' supply-chain/config.toml | grep -qxF 'version = "3.0.10"'
grep -A1 '^\[\[exemptions\.proc-macro2\]\]$' supply-chain/config.toml | grep -qxF 'version = "1.0.107"'
```
