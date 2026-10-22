---
id: "205-the-yaml-reader-takes-serde-saphyr-2"
title: "The YAML reader takes serde-saphyr 2.0"
status: approved
kind: "tooling"
created: "2026-10-21"
implementation: complete
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "196-yaml-parsing-moves-off-unsafe-libyaml"
summary: >
  Spec 196 pinned `serde-saphyr` at `=1.3.0` and declined 2.0.0 because it
  had been published the day before. Dependabot #504 proposes 2.0.0 (with
  `granit-parser` 2.0.0 and `sha2` 0.11.1) and fails to compile: 2.0
  deprecates the option 196's reader sets to refuse a non-finite float. This
  spec takes 2.0.0, sets the option's replacement to the same refusal, and
  moves the three `cargo vet` exemptions to the new versions. Every parity row
  196 pinned holds unchanged.
amends:
  # 196 D-1 pinned 1.3.0 and declined 2.0.0; 3.1 below replaces that pin.
  - "196-yaml-parsing-moves-off-unsafe-libyaml"
extends:
  # 3.2: the reader's non-finite option moves to its 2.0 name.
  - { spec: "196-yaml-parsing-moves-off-unsafe-libyaml", unit: { kind: file, path: "crates/spec-spine-types/src/yaml.rs" }, nature: corrective }
  # 3.3: the three exemptions move to the new versions; the counts do not change.
  - { spec: "200-the-dependency-policy-has-an-owner-and-records", unit: { kind: file, path: "supply-chain/config.toml" }, nature: additive }
references:
  - { unit: { kind: file, path: "Cargo.toml" }, role: "the workspace dependency pin" }
  - { unit: { kind: file, path: "Cargo.lock" }, role: "the locked tree the pin resolves to" }
---

# 205: The YAML reader takes serde-saphyr 2.0

## 1. Purpose

Spec 196 D-1 pinned `serde-saphyr` exactly at `1.3.0` and declined 2.0.0,
published on 2026-10-13, because a release one day old had had no time to
draw an advisory. Eight days later Dependabot's #504 proposes 2.0.0. It fails
to build under `-D warnings`: 2.0 deprecates
`Options::reject_non_finite_typeless_float`, which 196's reader sets to keep
D-4's refusal of `.inf` and `.nan`.

## 2. Territory

- **`crates/spec-spine-types/src/yaml.rs`** (196's): one option line.
- **`Cargo.toml`, `Cargo.lock`**: the pin and the locked tree.
- **`supply-chain/config.toml`** (200's): three exemption versions.

## 3. Behavior

### 3.1 The pin

The workspace pins `serde-saphyr = "=2.0.0"`, default features off and
`deserialize` on, as 196 D-1 did for 1.3.0. Its parser is `granit-parser`
2.0.0. Both crates still declare `#![forbid(unsafe_code)]`; `serde-saphyr`'s
MSRV is still 1.89, under the workspace's 1.90.

### 3.2 The non-finite refusal is set by its new name

The reader sets `Options::non_finite_float_policy` to
`NonFiniteFloatPolicy::Reject` in place of the deprecated boolean. 2.0's
changelog states the boolean `true` and `Reject` behave the same, so 196 D-4
is unchanged: `.inf` and `.nan` refuse the document as V-002. Every row of
`yaml_parity.rs` and `yaml_workflow_parity.rs` holds without an edit.

### 3.3 The vet exemptions follow the versions

`supply-chain/config.toml` exempts `granit-parser` and `serde-saphyr` at
2.0.0 and `sha2` at 0.11.1 instead of 1.3.0, 1.3.0 and 0.11.0. `cargo vet`
reports 50 crates fully audited and 125 exempted, as spec 200 measured.

## 4. Out of scope

- Any other change to 196's reader configuration.
- `sha2` 0.11.1 beyond its lock entry and exemption: the workspace requires
  `"0.11"`, so the patch release needs no manifest edit.

## 5. Resolved decisions

**D-1 (2026-10-21): 2.0.0 is taken eight days after its release.** 196 D-1's
reason for declining it was its age alone. On 2026-10-21 `cargo deny check
advisories` reports nothing against it, and both crates keep the properties
196 chose `serde-saphyr` for. 2.0's other changes (composite-key comparison,
comment handling, multi-document entry points, `Error::AliasError`'s new
field) touch no path the reader uses: it parses one document through
`from_str_with_options` and reads errors only through `Display`.

## Verification

```verify:cli
# 3.1: the pin is exact at 2.0.0 and the locked tree carries 2.0.0 only.
grep -qF 'serde-saphyr = { version = "=2.0.0", default-features = false, features = ["deserialize"] }' Cargo.toml
test "$(grep -c '^name = "serde-saphyr"$' Cargo.lock)" = 1
grep -A1 '^name = "serde-saphyr"$' Cargo.lock | grep -qxF 'version = "2.0.0"'
# 3.2: the reader sets the policy, not the deprecated boolean.
grep -qF 'options.non_finite_float_policy = serde_saphyr::NonFiniteFloatPolicy::Reject;' crates/spec-spine-types/src/yaml.rs
sh -c '! grep -qF reject_non_finite_typeless_float crates/spec-spine-types/src/yaml.rs'
cargo test -p spec-spine-types --locked --test yaml_parity
# 3.3: the exemptions name the new versions.
grep -A1 '^\[\[exemptions\.serde-saphyr\]\]$' supply-chain/config.toml | grep -qxF 'version = "2.0.0"'
grep -A1 '^\[\[exemptions\.granit-parser\]\]$' supply-chain/config.toml | grep -qxF 'version = "2.0.0"'
grep -A1 '^\[\[exemptions\.sha2\]\]$' supply-chain/config.toml | grep -qxF 'version = "0.11.1"'
```
