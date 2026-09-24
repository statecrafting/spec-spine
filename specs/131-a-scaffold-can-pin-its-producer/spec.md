---
id: "131-a-scaffold-can-pin-its-producer"
title: "A scaffold can pin its producer"
status: draft
kind: "tooling"
created: "2026-09-24"
summary: >
  Statecraft writes the `spec-spine.toml` that `scaffold_init_json` returns
  byte for byte, and that file pins nothing: the `[meta]` header and the key
  are both commented out (055 §3.4), and the example is a caret range. A
  repository Statecraft initializes is therefore judged by whatever binary a
  hook finds. This spec adds an opt-in scaffold option, `pinExactVersion`,
  that emits an active `[meta]` table with `required_version =
  "=<this producer's version>"`, through a new facade entry
  `scaffold_init_with_options_json(config_json, options_json)` and its Rust
  form `scaffold_init_with_options`. Without the option the output is
  byte-identical to `scaffold_init_json`'s, measured against 0.25.0 for
  Statecraft's configuration and the default. The configuration is validated
  exactly as 129 requires, and an unknown option is refused rather than
  silently producing an unpinned file.
implementation: complete
owner: "The spec-spine Authors"
risk: low
depends_on:
  - "055-a-version-pin-the-cli-can-check"
  - "092-the-engine-ships-governance-not-an-environment"
  - "129-a-json-config-obeys-the-loaders-rules"
# 3.1: 055 3.4 says `config_toml` MUST emit the key commented out. Under the
# option it is emitted active and exact.
amends: ["055-a-version-pin-the-cli-can-check"]
amends_sections: ["3.4"]
establishes:
  - { kind: file, path: "crates/spec-spine-core/tests/scaffold_pin.rs" }
extends:
  # 3.1, 3.2: the option and the block it swaps.
  - { spec: "092-the-engine-ships-governance-not-an-environment", unit: "crates/spec-spine-core/src/scaffold.rs", nature: additive }
  # 3.3: the facade entry and the re-exports.
  - { spec: "001-compile-registry", unit: "crates/spec-spine-core/src/lib.rs", nature: additive }
  # 3.3: the new entry joins 129's enumeration.
  - { spec: "129-a-json-config-obeys-the-loaders-rules", unit: "crates/spec-spine-core/tests/config_facade.rs", nature: additive }
  # 3.4: the facade documentation.
  - { spec: "129-a-json-config-obeys-the-loaders-rules", unit: "docs/api.md", nature: additive }
references:
  - { unit: { kind: file, path: "specs/055-a-version-pin-the-cli-can-check/spec.md" }, role: context }
intent:
  goal: "a consumer that writes the scaffold byte for byte can ask for a repository pinned exactly to the producer that wrote it, and a consumer that does not ask gets the same bytes as before"
  non_goals:
    - "a caller-supplied version: the pin names this producer, whose version is compiled in (4)"
    - "pinning by digest or checking artifact identity (4)"
    - "changing the default output, or 092 §3.2's retained entry and its signature (3.2)"
---

# 131: A scaffold can pin its producer

## 1. Purpose

Statecraft's `init` asks the linked library for the governance scaffold and
writes `spec-spine.toml` byte for byte (its spec 002 §3.15). Its call, read
from `statecraft-cli` at `fa000c6` (`crates/statecraft-home/src/producer.rs`),
is one line:

```rust
spec_spine_core::scaffold_init_json(config_json)
```

with `config_json` a layout-only object (`specs`, `.statecraft/derived`,
`standards/spec`, `standards/schemas`, `Cargo.toml`, the two npm probes, empty
standalone lists, `.statecraft/state`), against `spec-spine-core =0.23.0`.

The file it gets back pins nothing. 055 §3.4 requires the key commented out,
with the running version as a caret example:

```toml
# [meta]
# ...
# Cargo semantics: a bare version is a caret range, `=` is exact.
# required_version = "0.25.0"
```

Turning that into an exact pin takes two edited lines and an added `=`, which
a consumer that promises to write the producer's bytes unchanged cannot do
without recording a modification of its own. Statecraft's session drafted
exactly that workaround (its option E1) and named this change as the clean
alternative (E3).

055's reason for the default stands: a scaffold that pinned every new
repository by accident would create the stale-pin problem on day one. So the
pin is opt-in, requested by a consumer that has decided it wants it.

## 2. Territory

`crates/spec-spine-core/tests/scaffold_pin.rs` is new and established here.
The option lives in `scaffold.rs`, established by 092; the facade entry and
re-exports in `lib.rs`, established by 001; the new entry joins 129's
enumeration in `tests/config_facade.rs` and its documentation in
`docs/api.md`.

**This spec `amends` 055 §3.4**, which says `config_toml` MUST emit the key
commented out. Under the option it is emitted active and exact.

**It does not amend 092.** §3.2 requires `scaffold_init_json` retained with its
present signature and response shape, and pure; both hold, and the new entry
is additive and equally pure (its only version input is compiled in, as the
commented example already was). §3.2 also removed a `scaffold_init_with`
function that selected the kit, which is why the new Rust function has a
different name. §3.3's file set is unchanged.

## 3. Behavior

### 3.1 The option

`ScaffoldOptions` MUST carry one field, `pin_exact_version` (JSON
`pinExactVersion`), default false. When true, the returned `spec-spine.toml`
MUST contain an active `[meta]` table whose `required_version` is
`"=<the producing library's own package version>"`, followed by comments
saying the pin is exact and names the producing release.

The file MUST load through `load_config`, and the producer's own version MUST
satisfy the requirement while any other version does not.

### 3.2 The default is unchanged

With default options, `"{}"`, or `{"pinExactVersion": false}`, the output MUST
be byte-identical to `scaffold_init_json`'s for the same configuration.
`scaffold_init_json` and `scaffold_init` MUST return what they returned before
this spec. With the option set, only the `[meta]` block of `spec-spine.toml`
differs; every other line of that file and every other file is the same.

### 3.3 The facade entry

`scaffold_init_with_options_json(config_json, options_json)` MUST validate the
configuration exactly as 129 requires of every entry, with the same message as
`scaffold_init_json` for the same configuration. `options_json` MUST be a JSON
object matching `ScaffoldOptions`; an unknown key or text that is not JSON MUST
be `Error::Config`, so a consumer asking for a pin never receives an unpinned
file because it misspelled the option.

The Rust form is `scaffold_init_with_options(&Config, &ScaffoldOptions)`, and
`scaffold_init(cfg)` is that function with default options.

Statecraft's call shape with the option:

```rust
spec_spine_core::scaffold_init_with_options_json(config_json, r#"{"pinExactVersion":true}"#)
```

### 3.4 Documentation

`docs/api.md` MUST list the entry and the option beside `scaffold_init_json`.

## 4. Out of scope

**A caller-supplied version.** The pin names the producer that wrote the file,
whose version is a compile-time constant. A consumer that wants to pin a
different executable (Statecraft's "qualified executable") edits its own
configuration; accepting an arbitrary version here would let the producer
write a pin it cannot vouch for.

**Artifact identity.** A version requirement is not a digest, and a matching
`--version` does not identify a build. That remains
`scripts/reader-identity.sh`'s job (spec 124 §3.4).

**The interaction with a prerelease `main`.** A development build emits its
own version in the pin, which is correct: the file names its producer. Whether
`main` should carry a prerelease version is a separate decision.

## 5. Resolved decisions

**D-1 (2026-09-24): an options argument, not a configuration key.** `Config`
already has `meta.required_version`, and the scaffold could have echoed it.
It does not today (the example is always the running version), and making it
echo a caller's value is the caller-supplied version §4 declines. A separate
options object also keeps `config_json` a description of the repository
rather than of how to produce files for it.

**D-2 (2026-09-24): replace the block, keep one source.** The unpinned
template interpolates the commented block from the same function the pinned
path replaces, so the replacement always finds its target, and the unpinned
bytes were compared with 0.25.0's: identical for Statecraft's configuration
(`sha256 2b9aa03b...a195`) and for `{}` (`bedd3c93...4d7a`).

**D-3 (2026-09-24): refuse an unknown option.** `deny_unknown_fields`, exit 3
through the CLI's mapping. The failure being prevented is a consumer that
believes it asked for a pin and got none.

## Verification

```verify:cli
# 3.1 to 3.3: the pin is active, exact and loads; the default is unchanged;
# only the meta block differs; unknown options and invalid configs refuse.
cargo test -p spec-spine-core --test scaffold_pin --locked
# 3.3: the new entry runs 129's rules with the loader's message.
cargo test -p spec-spine-core --test config_facade --locked
# 054's round-trip and 092's producer boundary still hold.
cargo test -p spec-spine-core --test scaffold --locked
```
