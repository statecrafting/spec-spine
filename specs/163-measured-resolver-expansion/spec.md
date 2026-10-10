---
id: "163-measured-resolver-expansion"
title: "Expand structural resolution only where measurement proves it deterministic"
status: draft
kind: "governance"
created: "2026-09-27"
implementation: in-progress
owner: "The spec-spine Authors"
risk: medium
depends_on:
  - "004-codebase-index"
  - "016-directory-crate-module-units"
  - "025-symbol-resolution-feature-gate"
  - "155-selected-content-accessor"
summary: >
  Publishes a measured construct matrix for the Rust and TypeScript structural
  resolver and adds only the constructs whose identity and span are a pure
  function of the pinned grammars: items inside inline modules, Rust impl and
  trait members, macro_rules definitions, integration-test roots, TypeScript
  class methods and top-level variables, and a Rust test identity bound to its
  #[test] attribute, which lifts spec 155's test-selector reservation. Every
  other construct reports a closed unsupported or unknown reason instead of an
  absence. No identity that resolves today changes meaning; the index schema
  advances one MINOR, staged with the engine pin that first carries it, and
  dependency edges stay out of scope.
amends:
  # 004 §3.3 says "Only top-level items are indexed in v1 (no impl methods, no
  # inline mod bodies)"; §3.2 and §3.3 here replace that sentence.
  - "004-codebase-index"
  # 016 §3.2 resolves only a top-level inline mod block; §3.2 here resolves
  # nested inline modules too.
  - "016-directory-crate-module-units"
  # 155 was a draft when this spec was drafted and is approved now (D-1, as
  # revised 2026-10-10): §3.5 adds an omission reason to its vocabulary and
  # §3.7 changes what its structural projections return.
  - "155-selected-content-accessor"
establishes:
  - { kind: file, path: "crates/spec-spine-core/tests/resolver_matrix.rs" }
  - { kind: file, path: "crates/spec-spine-cli/tests/resolver_matrix.rs" }
extends:
  - { spec: "004-codebase-index", unit: { kind: file, path: "crates/spec-spine-core/src/symbols.rs" }, nature: additive }
  - { spec: "004-codebase-index", unit: { kind: file, path: "crates/spec-spine-core/src/index.rs" }, nature: additive }
  - { spec: "155-selected-content-accessor", unit: { kind: file, path: "crates/spec-spine-core/src/content.rs" }, nature: additive }
  - { spec: "155-selected-content-accessor", unit: { kind: file, path: "crates/spec-spine-types/src/content.rs" }, nature: additive }
  - { spec: "155-selected-content-accessor", unit: { kind: file, path: "crates/spec-spine-core/tests/content.rs" }, nature: corrective }
  - { spec: "155-selected-content-accessor", unit: { kind: file, path: "crates/spec-spine-cli/tests/content.rs" }, nature: additive }
  - { spec: "074-a-governed-read-names-its-version", unit: { kind: file, path: "crates/spec-spine-core/tests/read.rs" }, nature: additive }
  - { spec: "108-a-work-scope-is-declared", unit: { kind: file, path: "crates/spec-spine-core/tests/scope.rs" }, nature: additive }
  - { spec: "108-a-work-scope-is-declared", unit: { kind: file, path: "crates/spec-spine-cli/tests/scope.rs" }, nature: additive }
  - { spec: "022-index-sharding", unit: { kind: file, path: "crates/spec-spine-types/tests/dtos.rs" }, nature: additive }
  - { spec: "074-a-governed-read-names-its-version", unit: { kind: file, path: "crates/spec-spine-types/src/version.rs" }, nature: additive }
  - { spec: "057-the-docs-name-what-adopters-derived", unit: { kind: file, path: "docs/schema-versioning.md" }, nature: additive }
  - { spec: "088-the-template-teaches-the-whole-grammar", unit: { kind: file, path: "standards/spec/templates/spec-template.md" }, nature: additive }
references:
  - { unit: { kind: file, path: "docs/design/09-disposition-2026-09-21.md" }, role: "roadmap (RS-01, row J)" }
  - { unit: { kind: file, path: ".github/workflows/determinism.yml" }, role: "unowned; §3.9 adds one step" }
  - { unit: { kind: file, path: "crates/spec-spine-core/src/manifest.rs" }, role: "dependency-table strip that rules out manifest dependency edges" }
obligations:
  - id: "R-1"
    kind: requirement
    text: "The resolver supports exactly the constructs in the §3.2 matrix, each with the identity grammar and span that section gives, and nothing else."
    anchor: "3-2-supported-constructs-and-identity-grammar"
  - id: "R-2"
    kind: requirement
    text: "A Rust test identity binds one function to a bare #[test] attribute in a src/ file or an integration-test root, and spec 155's test selector resolves exactly those identities."
    anchor: "3-4-test-identity"
  - id: "R-3"
    kind: requirement
    text: "A lookup that does not resolve reports unsupported or unknown with one closed reason whenever §3.5 applies, and plain unresolved only otherwise."
    anchor: "3-5-unsupported-and-unknown-are-reported-not-absent"
  - id: "R-4"
    kind: requirement
    text: "Structural projections obey spec 155's whole-line rule and documentation recognizes every outer doc-comment form in §3.7."
    anchor: "3-7-projections-follow-the-grammar"
  - id: "R-5"
    kind: requirement
    text: "The read schema advances to 0.11.0, the index schema advances to 1.3.0 in the change that moves the engine pin to the first release carrying this resolver, and any later matrix change follows the §3.8 compatibility rule."
    anchor: "3-8-compatibility-gate"
  - id: "I-1"
    kind: invariant
    text: "Every id the 0.28.0 resolver (unchanged through 0.29.0) resolves keeps a byte-identical location list; a new construct never adds a location to it."
    anchor: "3-3-existing-identities-never-change-meaning"
  - id: "I-2"
    kind: invariant
    text: "Resolution is syntactic: no name resolution, macro expansion, type inference, cfg evaluation, import following, or dependency graph."
    anchor: "3-10-no-semantic-claims"
  - id: "V-1"
    kind: verification
    text: "Every matrix row, reason token, legacy golden, projection correction, and version constant is asserted by the resolver_matrix tests on each release triple."
    anchor: "verification"
    inputs:
      - "crates/spec-spine-core/tests/resolver_matrix.rs"
      - "crates/spec-spine-cli/tests/resolver_matrix.rs"
intent:
  goal: "give impl members, nested module items, and tests stable structural identities without changing any identity that resolves today"
  non_goals:
    - "semantic program understanding, macro expansion, or cfg evaluation"
    - "import, call, or package dependency graphs"
    - "a test authority-unit kind in frontmatter"
    - "languages other than Rust and TypeScript"
---

# 163: Expand structural resolution only where measurement proves it deterministic

## 1. Purpose

Specs 004 and 016 resolve `symbol` and `module` units to top-level items and
top-level inline modules. Spec 155 reserves its `test` selector as
`unsupported-selector` "until a resolver binds nested and integration-test
functions together with their test attributes" (155 §3.3, D-6), and draft 169 (declared obligation traceability, drafted as
161) makes every `test` trace target `unsupported` until this spec exists.

### 1.1 Measured matrix (2026-09-27, spec-spine 0.28.0, main `c0c1256d`)

Method: a throwaway repository with one Rust crate (`demo`, with `src/` and
`tests/`) and one npm package (`web`), and one draft spec declaring 50
`symbol` units and 6 `module` units, one per construct. It was run through
`compile`, `index` (unresolved units surface as `W-001`), and `content select`
on a committed tree. Node kinds were confirmed by dumping the pinned grammars
(tree-sitter 0.27.0, tree-sitter-rust 0.24.2, tree-sitter-typescript 0.23.2).

| # | Construct | 0.28.0 result | Evidence |
|---|---|---|---|
| 1 | Rust top-level `fn`, `struct`, `enum`, `union`, `trait`, `const`, `static`, `type` | resolved; span is the item node, attributes and docs excluded | `demo::top_fn` → `lib.rs:5-5` |
| 2 | Bodyless `mod x;` as a symbol | resolved to the declaration line | `demo::sub` → `lib.rs:2-2` |
| 3 | File module as a module unit | resolved, whole file | module `demo::sub` → `sub.rs` |
| 4 | Top-level inline `mod` (symbol and module) | resolved, block span | `demo::inline_mod` → `42-47` |
| 5 | Item inside an inline `mod` | unresolved | `demo::inline_mod::in_inline`, `demo::tests::helper_not_test` |
| 6 | Nested inline `mod` | unresolved as symbol and module | `demo::inline_mod::deeper` |
| 7 | Inherent `impl` fn and associated const, plain and generic | unresolved | `demo::TopStruct::inherent_method`, `::ASSOC`, `demo::Generic::gen_method` |
| 8 | Trait `impl` method | unresolved | `demo::TopStruct::trait_method` |
| 9 | Trait required and provided methods | unresolved | `demo::TopTrait::trait_method`, `::provided` |
| 10 | `fn` nested in a `fn` body | unresolved | `demo::outer::nested` |
| 11 | `macro_rules!` definition, exported or not | unresolved | `demo::make_fn`, `demo::exported_macro` |
| 12 | Item generated by an item-position macro | unresolved, indistinguishable from absent | `demo::macro_generated` |
| 13 | `#[cfg]` twins of one name | resolved with two locations; a `content select` naming it refuses the whole request, exit 2 | `demo::cfg_twin` |
| 14 | `#[test]` fn in `src/` `mod tests` | unresolved as a symbol; the `test` selector returns `unsupported-selector` | `demo::tests::unit_test_in_src` |
| 15 | `#[tokio::test]` fn | unresolved | `demo::tests::tokio_test` |
| 16 | Integration test in `tests/` | unresolved under all three spellings tried; `tests/` is never walked | `demo::integration_test_fn`, `integration::…`, `demo::integration::…` |
| 17 | Rust `///` and multi-line `/** */` docs | `documentation` projection resolved | `doc_then_attr` → `1-1`, attribute skipped |
| 18 | Rust single-line `/** doc */` and TS single-line JSDoc | `missing-content`, "no attached documentation comments" | `demo::docs::block_doc`, `web::src::util::formatDate` |
| 19 | `#[doc = "…"]` | `missing-content`, a false claim of absence | `demo::docs::attr_doc` |
| 20 | Doc comment separated by a blank line | `missing-content` (155's contiguity rule; rustdoc would attach it) | `demo::docs::blank_separated` |
| 21 | Multi-line signature ending `) -> u8 {` | `signature` 20-21 drops the return-type line; `body` 22-24 begins with `) -> u8 {` | `demo::docs::multi_line_sig` |
| 22 | TS `function` (exported, default, or internal), `class`, `interface` | resolved | `web::src::util::internalFn` → `10-10` |
| 23 | TS class method, instance or static | unresolved | `web::src::util::Helper::run`, `::make` |
| 24 | TS `export const f = () => …` | unresolved | `web::src::util::arrowFn` |
| 25 | TS `namespace` | unresolved | `web::src::util::NS`, `::NS::inNs` |
| 26 | `.tsx` | parsed with the TypeScript grammar, not TSX; top-level items resolved by error recovery | `web::src::comp::After` |
| 27 | String-named TS test `it("…")` | no identity exists | `util.ts:13` |
| 28 | Dependency edges | none recorded; `PackageRecord` carries no dependency list, and the manifest projection strips Cargo dependency tables on purpose | `codebase.rs` `PackageRecord`; `manifest.rs` `CARGO_DEP_TABLES` |
| 29 | Four-triple determinism proof | this repository's frontmatter declares zero `symbol` or `module` units, so `determinism.yml` digests no tree-sitter span | 0 `"kind": "symbol"` in `.statecraft/derived/codebase-index/by-spec/` |

Re-measured 2026-10-10 against 0.29.0 (main `3910c483`): neither
`crates/spec-spine-core/src/symbols.rs` nor `crates/spec-spine-core/src/content.rs`
has changed since `c0c1256d`, so every row above stands for 0.29.0 as well.

Adopter exposure is small. Among the local checkouts under `~/DevWork`, only
`rahi` (two top-level symbols in `refines`) and `wire-witness` (three planned
file-module units) declare such units, and all are rows 1 to 3.

### 1.2 What the measurement decides

Rows 5 to 11, 14, 16, 23, and 24 are syntactic facts that the pinned grammars
expose, with a name field and a line span. They can be supported without any
semantic step. Rows 12 and 15 cannot be supported honestly: one needs macro
expansion and the other depends on what a framework's macro means. They must
stop looking like plain absence. Rows 18, 19, and 21 are defects against 155
§3.5 as written. Row 29 means the determinism claim for spans has so far been
asserted and never exercised. Row 28 shows that dependency edges would
reintroduce bytes the index deliberately excludes (§5).

## 2. Territory

- `crates/spec-spine-core/src/symbols.rs` (004): the construct walker, the
  identity grammar, the legacy-set rule, recorded unsupported sites, the test
  index, and the projection corrections.
- `crates/spec-spine-core/src/index.rs` (004): classified reasons on the
  `I-005` and `I-008` messages.
- `crates/spec-spine-core/src/content.rs` and
  `crates/spec-spine-types/src/content.rs` (155): the `test` selector
  resolves, and the omission reason gains `indeterminate-selector`.
- `crates/spec-spine-core/tests/content.rs` (155): its
  `unsupported_test_selector_is_an_explicit_omission` case is rewritten to
  assert an unsupported form under §3.4. It currently asserts that every test
  selector is unsupported.
- `crates/spec-spine-types/src/version.rs`, `docs/schema-versioning.md`, and
  the spec template: the version constants and their documentation, plus the
  new id forms in the template's unit comments.
- The tests that pin the read version literal (`crates/spec-spine-core/tests/`
  `read.rs`, `scope.rs` and `content.rs`; `crates/spec-spine-cli/tests/`
  `scope.rs` and `content.rs`) and, at the pin move (§3.8), the index version
  literal in `crates/spec-spine-types/tests/dtos.rs`.
- The planned `resolver_matrix` tests in core and CLI.
- `.github/workflows/determinism.yml` is unowned and is edited by §3.9.
  Workflow files are outside `SOURCE_EXTS`, so the edit carries no C-002. It
  does restale the inputs record (spec 141).

## 3. Behavior

### 3.1 Scope of the walk

Rust: every `.rs` file under a Rust package's `src/` (unchanged), plus its
integration-test roots: `tests/<stem>.rs` and `tests/<dir>/main.rs`. TypeScript:
unchanged (`.ts`/`.tsx` under the package, `.d.ts` skipped). Existing exclusions
and the state-dir filter apply. Containers (inline modules, impl blocks, trait
bodies, class bodies) are descended to a depth of 32. A container deeper than
that yields §3.5 `unknown` / `nesting-limit` for ids under it.

### 3.2 Supported constructs and identity grammar

`<crate>` is the package name with `-` replaced by `_`. `<pkg>` is the npm
package name. `<mod>` is the file-module path (spec 004 rules), followed by the name of
each enclosing inline module. `<root>` is `<crate>` for `src/` files and
`<crate>/tests/<target>` for an integration-test root, where `<target>` is the
`<stem>` or `<dir>`. `/` cannot occur in a Rust path, so a root id never
collides with a `src/` id. Spans are the named node's line span, with attributes
and comments excluded as today.

| Construct | Id | Kind |
|---|---|---|
| Legacy item kinds (§1.1 rows 1, 2) at any inline-module depth | `<root>::<mod>::<name>` | symbol |
| Inline `mod` block at any depth | `<root>::<mod>::<name>` | symbol and module (block span) |
| Integration-test root file | `<crate>/tests/<target>` | module (whole file) |
| Inherent `impl` member (`fn`, `const`, `type`) | `<root>::<mod>::<Type>::<name>` | symbol |
| Trait `impl` member | `<root>::<mod>::<` `Type` ` as ` `TraitPath` `>::<name>`, for example `demo::<TopStruct as fmt::Display>::fmt` | symbol |
| Trait body member (`fn`, signature-only `fn`, `const`, `type`) | `<root>::<mod>::<Trait>::<name>` | symbol |
| `macro_rules!` definition | `<root>::<mod>::<name>!` | symbol |
| TS class `method_definition` with a plain identifier name | `<pkg>::<mod>::<Class>::<name>` | symbol |
| TS top-level `const`/`let`/`var` declarator binding one identifier | `<pkg>::<mod>::<name>` | symbol (declarator span) |

`<Type>` is the impl's self type only when it is a bare type identifier or a
generic type over one. Generic arguments are dropped, so `impl<T> Generic<T>`
gives `Generic`. `TraitPath` is the trait as written, with all whitespace
removed and generic arguments kept (`fmt::Display`, `From<u8>`). The angle
brackets and the single-space ` as ` around a trait-impl segment are literal,
as in Rust's own qualified path, so a `::` inside `TraitPath` never splits the
id (D-10). The id prefix
is where the impl is written, not where the type is defined. Spelling a trait
two ways yields two ids; nothing is resolved. The trailing `!` keeps macros in
their own namespace, as Rust does.

Several locations for one id remain several locations in the index and an
ambiguity refusal in `content select`, unchanged (row 13). Two impls of one
trait with differing self-type arguments are ambiguous by construction. They
are not disambiguated by inference.

New constructs in a `.tsx` file are not supported (§3.5, `tsx-grammar`). The
legacy `.tsx` result keeps using the TypeScript grammar (D-5).

### 3.3 Existing identities never change meaning

Let L be the id-to-locations map that the 0.28.0 construct set yields for a
tree. That set is legacy item kinds at the top level of each `src/` file, top-level
inline modules, and file modules. For every id in L, the resolver MUST return
exactly L's locations, byte-identical in order and span. A new construct whose id
is already in L contributes no location. It is dropped and recorded as an
unsupported site with reason `legacy-collision`. Such collisions need
non-idiomatic names, such as a file module and a type sharing a name, and the
legacy owner keeps the id. The resolver_matrix test carries a golden of L for its
fixture, captured from 0.28.0 before the build, and asserts it unchanged.

### 3.4 Test identity

A Rust test is a `function_item`. Its contiguous preceding `attribute_item`
siblings must include one whose attribute is the bare path `test` with no
arguments. The item must sit at the top level of, or inside inline modules
within, a `src/` file or an integration-test root (§3.1). Its test id is the
§3.2 symbol id of that function: `<root>::<mod>::<name>`. Its span runs from the
first line of that contiguous attribute run to the function's last line, so
selected content shows the attribute. `#[should_panic]`, `#[ignore]`, and
`#[cfg(...)]` do not affect identity.

Spec 155's `test` selector resolves these ids. Its `projection`, budget,
digest, and ambiguity rules are 155's unchanged. `documentation` is taken from
above the attribute run, `signature` and `body` from the function node. A test
selector needs the fresh committed index, as a `symbol` selector does. A
function that resolves as a symbol but carries no test attribute is
`missing-content` ("not a test function").

A package whose `Cargo.toml` sets `autotests = false` or declares any `[[test]]`
table has none of its `tests/` files treated as roots. Their ids are
`unsupported` / `declared-test-targets`, because the target set is no longer
the directory convention (D-4).

### 3.5 Unsupported and unknown are reported, not absent

The walker records every site it recognizes but does not support, with a
would-be id where one exists. A lookup of a `symbol`, `module`, or `test` id
yields exactly one outcome, with this precedence:

1. `resolved`: one or more locations (§3.2 to §3.4);
2. `unsupported`: the id equals a recorded site's would-be id;
3. `unknown`: the id's scope cannot be enumerated. The scope is the longest
   proper prefix that resolves as a module, type, trait, or class;
4. `unresolved`: none of the above.

Reason tokens are closed:

| Outcome | Reason | Site |
|---|---|---|
| unsupported | `nested-item` | an item inside a `fn` body; would-be `<fn-id>::<name>` |
| unsupported | `impl-type-form` | impl self type that is a reference, scoped path, tuple, array, pointer, or `dyn`/`impl` type |
| unsupported | `re-export` | a named `use`/`export {…}` binding; ids name definitions, never re-exports |
| unsupported | `member` | enum variant, struct or union field, TS class field or interface member |
| unsupported | `foreign-item` | an item in an `extern` block |
| unsupported | `framework-test-attribute` | a fn whose attribute path ends in `test` but is not bare `test` (row 15); test ids only |
| unsupported | `unrooted-test-file` | a `.rs` file under `tests/` that is not a root; would-be `<crate>/tests/<path>` |
| unsupported | `declared-test-targets` | §3.4, last paragraph |
| unsupported | `typescript-test` | any `test` selector whose prefix names an npm package; string-named tests have no identity |
| unsupported | `ts-namespace`, `ts-destructuring`, `ts-computed-name` | TS namespaces, destructuring declarators, computed or private member names |
| unsupported | `tsx-grammar` | a §3.2 new construct in a `.tsx` file |
| unsupported | `legacy-collision` | §3.3 |
| unknown | `item-macro-in-scope` | the scope contains an item-position macro invocation, other than `macro_rules!`, that could define the name |
| unknown | `parse-error-in-scope` | the scope's node contains a grammar error |
| unknown | `nesting-limit` | §3.1 |
| unknown | `resolver-disabled` | built without `symbol-resolution` (spec 025) |

Surfacing:

- `index`: codes `I-005` and `I-008` and their `W-001`/`W-002` downgrade are
  unchanged. For `unsupported` and `unknown`, the message gains the suffix
  ` (<outcome>: <reason>)`. A plain `unresolved` message stays byte-identical
  to 0.28.0.
- `content select`: `unsupported` maps to `unsupported-selector`, `unknown` to
  the new `indeterminate-selector`, and `unresolved` to `missing-content`. Each
  carries its reason token in `message`. Required-selector completeness rules
  are 155's.

No outcome is ever mapped to another for convenience. Nothing falls back to a
containing item or file.

### 3.6 Nothing is walked that is not needed

The symbol and module indexes remain built only when a declared unit or a
selector needs them. The test index is built only when a `test` selector is
present. Unsupported sites are recorded in the same walk and add no
file reads.

### 3.7 Projections follow the grammar

These corrections bring 155's implementation into line with 155 §3.5 as
written.

- `signature` and `body`: when the body node starts on a line that also holds
  signature text before it, the boundary shares a line. Both projections then
  MUST be `unsupported-projection` (row 21), and neither returns a truncated
  span. New constructs get the same treatment. A node with no `body` field
  (declarators, signature-only trait fns, consts) has neither projection.
- `documentation`: recognized forms are contiguous `///` lines, and `/** … */`
  blocks on one line or several (Rust and TS), with attributes between comment
  and item skipped as today. A contiguous `#[doc = …]` attribute yields
  `unsupported-projection` ("attribute documentation"), not `missing-content`
  (row 19). Blank-line separation still ends the run (D-6). Every
  documentation span 0.28.0 returns is returned unchanged.

### 3.8 Compatibility gate

- `INDEX_SCHEMA_VERSION` becomes `1.3.0` (MINOR). The document shape is
  unchanged; the version records the resolver matrix that produced the bytes,
  following spec 023's semantics-only MINOR. Effects:
  - every index file restamps `schemaVersion` once;
  - a unit that newly resolves gains locations, loses its warning, and adds its
    backing file to its shard's `shardHash` inputs;
  - a unit newly classified `unsupported` or `unknown` changes only its message;
  - every other unit is byte-identical by §3.3.
  A 0.28.0 binary reading a 1.3.0 tree reports it stale, which is the
  `required_version` pin's job.
- `INDEX_SCHEMA_VERSION` moves in the change that moves this repository's
  engine pin to the first release carrying this resolver, not before (D-9).
- `READ_SCHEMA_VERSION` becomes `0.11.0` (MINOR): one new omission reason, and
  `test` selectors that resolve. (`0.10.0` was taken by spec 158 after this
  spec was drafted.)
- It is MINOR only because of I-1. For every later matrix change: adding a
  construct is an index MINOR and MUST preserve §3.3 against the then-current
  L. Removing a construct, re-spelling an id grammar, or changing a legacy
  location is MAJOR. Adding a reason token is a read MINOR.
- A newly resolved unit becomes a claim with the existing span semantics. An
  approved spec cannot hold an unresolved symbol unit and still pass `check`
  (`I-005` blocks). Ownership therefore moves only for draft or in-flight
  owners (`W-001`) and non-owning references (`W-002`).
- `docs/schema-versioning.md` records both versions, and the adopter note
  (re-run `index` once).

### 3.9 Determinism across the four release triples

The walk sorts files, containers, and members by path and then start position.
Ids and locations sort as today. Output is a pure function of (config, file
contents). The resolver_matrix core test embeds the full expected map for its
fixture: ids, spans, test ids, and every outcome and reason. It compares that
map byte for byte, so passing on a triple proves that triple emits the golden.
`determinism.yml` gains one step per leg, run after its build:
`cargo test -p spec-spine-core --test resolver_matrix --locked`. This closes
row 29, and all four triples then assert one golden. The fixture has CRLF and
BOM variants of one file, which must yield identical spans (spec 143).

### 3.10 No semantic claims

The resolver MUST NOT resolve names or `use`/import paths, expand macros,
evaluate `cfg`, infer types, follow `#[path]`, or emit dependency, call, or
import edges. An id is a syntactic address and asserts nothing about behavior,
reachability, or test execution. Resolving a test identity never runs it.

## 4. Acceptance criteria

1. Every §1.1 row is a resolver_matrix case with its post-163 result: rows 5 to
   11, 14, 16, 23, and 24 resolve under §3.2 and §3.4; rows 12, 15, and 25 to 27
   report their §3.5 reasons; rows 18, 19, and 21 follow §3.7.
2. The L golden for the fixture is byte-identical, including row 13's two
   locations and row 2's declaration-line span.
3. Every §3.5 reason token has at least one case in core and one surfaced
   through the CLI (`index` message and `content select --json` omission).
4. A plain unresolved unit's diagnostic is byte-identical to 0.28.0.
5. Feature off: `symbol`, `module`, and `test` lookups report
   `unknown`/`resolver-disabled`, and the crate builds without tree-sitter.
6. The two version constants read `1.3.0` and `0.11.0`, and conformance
   passes. The read constant moves with the build; the index constant moves at
   the pin change (D-9).
7. `determinism.yml` runs the resolver_matrix test on all four triples.
8. This repository's committed index changes only by the `schemaVersion`
   restamp, and only at the pin change. Its corpus declares no symbol or module
   unit, so the build itself leaves every committed shard byte-identical.

## 5. Out of scope

- **Dependency edges.** Manifest dependency edges would put back into index
  bytes the Cargo dependency tables that `manifest.rs` strips so a
  Dependabot-class bump does not stale the ledger. Edges from `use`/import
  statements are a guessed graph without name resolution, because re-exports,
  globs, macros, and `cfg` all intervene. Spec 109 already makes impact a
  declared relation. Reopen only with a consumer and a deterministic source.
- A `test` authority-unit kind in frontmatter; 169 targets use 155 selectors.
- Macro-expanded, parameterized (`rstest`, `test_case`), doctest, benchmark,
  example, `build.rs`, and framework-attribute tests.
- Enum variants, fields, closures, and items in `fn` bodies.
- TSX grammar selection, `.vue`, `.d.ts`, JavaScript, and other languages.
- Changing 155's selector, budget, or ambiguity contracts.

## 6. Resolved decisions

**D-1 (2026-09-27, revised 2026-10-10): amend 004, 016 and 155.** 004 §3.3
and 016 §3.2 state the top-level limit as behavior in approved specs, so
widening it is an amendment. 155 was a draft when this was drafted, and its
§3.3 anticipates "later resolver specs", so lifting the test reservation alone
would need no `amends`. 155 was approved on 2026-09-28 (`a86ed246`), and
§3.5's new omission reason and §3.7's projection corrections change what its
approved text returns, so 155 is amended too. Its declared acceptance (the content
tests) keeps passing, so no `amends_verification` is needed.

**D-2 (2026-09-27): legacy set wins collisions.** Letting a new construct add
a location to an existing id would change a committed shard for a unit that
resolves today. That violates I-1 and would make the bump MAJOR.

**D-3 (2026-09-27, draft, owner to confirm): trait-impl ids keep trait
generic arguments and drop self-type arguments.** `From<u8>`/`From<u16>`
impls routinely differ only by trait argument, and inherent impls rarely differ
by self-type argument. The alternative, both kept verbatim, forces authors to
write `Generic<T>::m`.

**D-4 (2026-09-27, draft, owner to confirm): bare `#[test]` only.**
`#[tokio::test]` is common, but recognizing it by the last path segment would
guess at a macro's meaning. A later spec may add a closed, named list.

**D-5 (2026-09-27, draft, owner to confirm): `.tsx` keeps the TypeScript
grammar.** Switching to the TSX grammar could move legacy spans (I-1). New
constructs in `.tsx` stay `unsupported` until a MAJOR decides it.

**D-6 (2026-09-27, draft, owner to confirm): blank lines still end a doc
run.** rustdoc attaches across them. Changing it now would alter results that
155 returns today, so it is left to a 155 amendment if wanted.

**D-7 (2026-09-27): determinism by golden, not by corpus.** Declaring symbol
units in this repository's own specs to exercise the workflow would couple
product specs to a test need. A golden test run on each triple is a
stronger proof.

**D-8 (2026-09-27): matrix lives in this spec, the test, and the template.**
A standalone `docs/resolver-matrix.md` would need an `extra_hashed_inputs`
entry (L-008) and a `spec-spine.toml` edit. It is deferred until the matrix
grows.

**D-9 (2026-10-10): the index restamp moves with the engine pin.** This
repository's managed gate judges the committed index with the exact released
engine its pin names (spec 156), and the self-governance job judges the same
tree with the candidate engine. A `schemaVersion` restamp satisfies one and
fails the other, so it cannot merge with the build. The read constant has no
committed artifact and moves with the build. The index constant, its
`dtos.rs` literal, the `schema-versioning.md` row and the one regenerated index
tree land together in the change that moves the pin to the first release
carrying this resolver (spec 193's pattern). Until then this spec stays
`implementation: in-progress` and its Verification block's `1.3.0` line fails.

**D-10 (2026-10-10): a trait-impl segment is spelled as Rust spells it.** The
draft wrote `<Type as TraitPath>` in the same placeholder notation as `<Type>`,
which left open whether the brackets are literal. They are: Rust's qualified
path is `<Type as Trait>::name`, and without the brackets a `TraitPath` such
as `fmt::Display` would be split by the `::` that §3.5's scope prefix walks.

**D-11 (2026-10-10): two tokens have no CLI surface.** `legacy-collision`
names an id that resolves to its legacy owner, so no lookup can return it, and
`resolver-disabled` needs a binary built without `symbol-resolution`, which the
CLI test does not build. Both are asserted in the core resolver_matrix test
(the recorded-site list and the `--no-default-features` run). Every other
token reaches the CLI test through an `index` message or a `content select`
omission.

**D-12 (2026-10-10): any non-bare attribute ending in `test` is a framework
attribute.** §3.4 binds only a bare `#[test]` with no arguments. An attribute
whose path's last segment is `test` but which is not that form
(`#[tokio::test]`, and `#[test(...)]`) records `framework-test-attribute`; an
attribute whose last segment is anything else (`#[rstest]`, `#[test_case]`)
records nothing, so the function is "not a test function".

**D-13 (2026-10-10): §3.7's documentation grammar runs only where 0.28.0's
found nothing.** The 0.28.0 scan runs first and its span is returned as is;
only when it returns nothing do the one-line `/** … */` block, a multi-line
block whose closing line carries text, and the `#[doc = …]` report apply. That
is the cheapest construction that provably keeps every 0.28.0 span.

**D-14 (2026-10-10): a leading BOM is stripped before parsing.** Removing it
moves no row, so no span changes; measured on the 0.29.0 release, a BOM'd CRLF
file already resolved its legacy items at the LF spans, and the golden keeps
asserting that.

**D-15 (2026-10-10): a test's signature and body come from its function
node.** A test span opens on its attribute run, so the projection finds the
function item ending on the span's last line rather than one starting on its
first.

**D-16 (2026-10-10): the fallback reads attribute structure, not raw
brackets.** Review of the second change found that the 0.28.0 attribute scan
counts every `[` and `]`, so a `[` inside a string literal can leave the depth
unbalanced for a line above to cancel. The 0.28.0 scan is kept as it is
(D-13), but the §3.7 fallback ignores brackets inside string literals, never
lets the depth go negative, and stops at a comment line when it is outside an
attribute.

Status (2026-10-10): built in two changes so each stays reviewable. The
first carries the resolver, the index messages, the determinism step and the
core resolver_matrix lookups and index outcomes. The second carries the
`content select` surface (test selectors, `indeterminate-selector`, the §3.7
projections), the read constant, the template note and the CLI resolver_matrix
test. Both are built. The index constant and its restamp wait for the pin
move (D-9), so `implementation` stays `in-progress` and the Verification
block's `1.3.0` line fails until then.

## Verification

Every line but the last fails before the build: the test targets do not exist and the
constants read `1.2.0` and `0.10.0`.

```verify:cli
cargo test -p spec-spine-core --test resolver_matrix --locked
cargo test -p spec-spine-core --no-default-features --test resolver_matrix --locked
cargo test -p spec-spine-cli --test resolver_matrix --locked
grep -q 'INDEX_SCHEMA_VERSION: &str = "1.3.0"' crates/spec-spine-types/src/version.rs
grep -q 'READ_SCHEMA_VERSION: &str = "0.11.0"' crates/spec-spine-types/src/version.rs
grep -q 'resolver_matrix' .github/workflows/determinism.yml
cargo test --workspace --locked
```
