//! The `init` scaffolder (spec 006): generate a new adopter's starter corpus as
//! **files-as-data**. Pure function of `(config)`: no filesystem writes happen
//! here; the CLI ([`cmd_init`]) writes the returned [`ScaffoldFile`]s. This keeps
//! core IO-light, unit-testable, and FFI-friendly (`scaffold_init_json`).
//!
//! Generated paths honor `config.layout` (`specs_dir`, `standards_dir`) and
//! `config.manifest.metadata_namespace`, so a non-default config scaffolds a
//! coherent non-default layout (the adoption definition-of-done, prompt §8).

use serde::{Deserialize, Serialize};
use spec_spine_types::{Config, Error};

/// A scaffolded file: repo-relative path, contents, and whether `init` should
/// overwrite an existing file (the default generator sets this `false`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScaffoldFile {
    pub rel_path: String,
    pub contents: String,
    pub overwrite: bool,
}

/// The full set of files `spec-spine init` writes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Scaffold {
    pub files: Vec<ScaffoldFile>,
}

/// Generate the adopter scaffold for `cfg`. Pure; performs no IO.
pub fn scaffold_init(cfg: &Config) -> Result<Scaffold, Error> {
    let ns = &cfg.manifest.metadata_namespace;
    let specs = cfg.layout.specs_dir.trim_end_matches('/');
    let standards = cfg.layout.standards_dir.trim_end_matches('/');

    let file = |rel_path: String, contents: String| ScaffoldFile {
        rel_path,
        contents,
        overwrite: false,
    };

    let files = vec![
        file("spec-spine.toml".to_string(), config_toml(cfg)),
        file(
            format!("{standards}/constitution.md"),
            CONSTITUTION.to_string(),
        ),
        file(format!("{standards}/contract.md"), CONTRACT.to_string()),
        file(
            format!("{standards}/templates/spec-template.md"),
            spec_template(ns),
        ),
        file(
            format!("{standards}/templates/constitution-template.md"),
            CONSTITUTION_TEMPLATE.to_string(),
        ),
        file(format!("{specs}/000-bootstrap/spec.md"), bootstrap_spec(ns)),
        file(
            ".claude/rules/orchestrator-rules.md".to_string(),
            ORCHESTRATOR_RULES.to_string(),
        ),
        file(
            ".claude/rules/governed-artifact-reads.md".to_string(),
            GOVERNED_READS.to_string(),
        ),
        file(
            ".claude/rules/adversarial-prompt-refusal.md".to_string(),
            REFUSAL_RULE.to_string(),
        ),
        file(".gitignore".to_string(), gitignore(cfg)),
    ];

    Ok(Scaffold { files })
}

// ===== templates =====

/// A documented starter `spec-spine.toml`, config-aware so a non-default
/// namespace / layout scaffolds coherently.
///
/// Spec 061 §3.2: every table and every key, each at its actual default and
/// each with a line saying what it does and, where one exists, which diagnostic
/// code it drives. The scaffold emitted five knobs and adopters needed
/// thirteen, so each of them read the source or derived the name by experiment;
/// aicortex annotated its own with the code the knob drives, which is the shape
/// copied here.
///
/// A key whose default is the right starting value is emitted at that value. A
/// key that only means something once the adopter has content for it is emitted
/// **commented out** with an example, so they uncomment rather than invent
/// syntax and are not handed empty tables that look meaningful.
///
/// `tests/scaffold.rs` asserts this parses back to `Config::default()` modulo
/// the values substituted from `cfg`: a documented config that has drifted from
/// the defaults it documents is worse than none.
fn config_toml(cfg: &Config) -> String {
    format!(
        "# spec-spine.toml governs this repository. Every key is optional; an\n\
         # absent file behaves as the defaults for a single-Cargo-workspace repo.\n\
         # Every key below is set to its default, so deleting one changes nothing.\n\
         # `spec-spine config show` prints the effective configuration, including\n\
         # the built-in bypass floor this file cannot see.\n\
         \n\
         # [meta]\n\
         # Pin the spec-spine version this repository is governed by. Uncomment to\n\
         # refuse a binary that does not satisfy it. A pin is not only about\n\
         # features: the coupling gate\x27s built-in bypass floor is compiled into\n\
         # the binary, so two versions can judge the same diff differently.\n\
         # Cargo semantics: a bare version is a caret range, `=` is exact.\n\
         # required_version = \"{running_version}\"\n\
         \n\
         [manifest]\n\
         # Drives the Cargo `[package.metadata.{ns}].spec` and package.json `\"{ns}\".spec` reads.\n\
         metadata_namespace = \"{ns}\"\n\
         \n\
         [domains]\n\
         # L-002: warn on a spec with no `domain` once this list is non-empty.\n\
         # Empty means the taxonomy is disabled, not that every spec is unclassified.\n\
         allowed = []\n\
         \n\
         [kind]\n\
         # L-003, symmetric with [domains].\n\
         allowed = []\n\
         \n\
         [layout]\n\
         # Where authored truth lives.\n\
         specs_dir     = \"{specs}\"\n\
         # Where the compiler and indexer write. See .gitignore for whether the\n\
         # shard trees belong in version control.\n\
         derived_dir   = \"{derived}\"\n\
         standards_dir = \"{standards}\"\n\
         schemas_dir   = \"{schemas}\"\n\
         # An ungoverned root for a tool's own working files (spec 039): excluded\n\
         # from every content hash, and bypassed by the coupling gate. L-006 if a\n\
         # spec claims a unit inside it. Empty means no such root is declared.\n\
         state_dir     = \"{state}\"\n\
         # The workspace manifest the Rust package discovery starts from.\n\
         cargo_workspace = \"{cargo_workspace}\"\n\
         # Files probed for npm workspace globs.\n\
         npm_workspaces = [{npm_workspaces}]\n\
         # Packages outside any workspace, named one by one.\n\
         # standalone_rust_workspaces = [\"tools/thing\"]\n\
         # standalone_npm_packages    = [\"npm\"]\n\
         \n\
         [index]\n\
         # Extra files folded into the global-inputs hash, so a change to one\n\
         # stales every shard.\n\
         #\n\
         # WATCH THE GLOB FORM. `dir/**` matches DIRECTORIES and therefore no\n\
         # files; you want `dir/**/*`. The default below carries the broken form\n\
         # and so hashes nothing (spec 057 found this in spec-spine\x27s own\n\
         # config, where it had silently held since the key was added). It is\n\
         # left as-is here because changing the default would restale every\n\
         # existing adopter\x27s index; fix it in your own file:\n\
         #\n\
         #   extra_hashed_inputs = [\"{standards}/**/*\", \".github/workflows/**/*\"]\n\
         extra_hashed_inputs = [{extra_hashed_inputs}]\n\
         # Directories the symbol resolver and the coverage walk skip.\n\
         resolver_exclusions = [{resolver_exclusions}]\n\
         # Named path sets `index check --slice <name>` can gate on their own.\n\
         # [index.slices]\n\
         # api = [\"crates/*/src/lib.rs\"]\n\
         \n\
         [branding]\n\
         # Recorded in every emitted artifact's `build` block.\n\
         compiler_id = \"{compiler_id}\"\n\
         indexer_id  = \"{indexer_id}\"\n\
         \n\
         [coupling]\n\
         # The PR-body waiver keyword (the reason follows the colon). A waiver is\n\
         # a human instrument: it needs explicit human approval, and an agent\n\
         # never writes one on its own authority.\n\
         waiver_keyword = \"{waiver}\"\n\
         # ADDITIVE to the built-in generic floor; it cannot remove an entry.\n\
         # `spec-spine config show` prints the merged list the gate matches on.\n\
         bypass_prefixes = []\n\
         # C-002: refuse a changed source file that no spec specifically claims.\n\
         # Off to start: a corpus turns this on once its coverage debt is retired.\n\
         # `spec-spine index coverage` reports where you stand.\n\
         require_ownership = {require_ownership}\n\
         # Clear a diff whose every non-bypassed path is a dependency-only\n\
         # manifest edit. Off to start; turn it on if a bot opens bump PRs, which\n\
         # cannot add a waiver line to their own body.\n\
         auto_waive_dependency_only = {auto_waive}\n\
         \n\
         [lint]\n\
         # L-007: refuse a `depends_on` entry that does not name a lower ordinal.\n\
         # Off to start: a corpus that files by domain rather than by date holds\n\
         # a coherent position this would spam.\n\
         require_ordinal_monotonic_depends_on = {ordinal_monotonic}\n\
         # L-008 suppression: claimed paths deliberately in no content hash.\n\
         # `index check` still reports the count, so an exception stays explicit.\n\
         # unwitnessed_allowed = [\"crates/**/*.rs\"]\n\
         \n\
         # [provenance.uri_schemes]\n\
         # Named URI prefixes a `references` provenance value may use. The\n\
         # header is commented too: an empty table here would OVERRIDE the\n\
         # built-in schemes rather than add to them.\n\
         # knowledge        = \"knowledge://\"\n\
         # code-fingerprint = \"fingerprint://\"\n\
         \n\
         [frontmatter]\n\
         # Keys this corpus recognizes beyond the grammar, so the unknown-key\n\
         # lint stays quiet about them. They still land in `extraFrontmatter`.\n\
         # extra_known_keys = [\"owner\", \"risk\"]\n",
        running_version = env!("CARGO_PKG_VERSION"),
        ns = cfg.manifest.metadata_namespace,
        specs = cfg.layout.specs_dir,
        derived = cfg.layout.derived_dir,
        standards = cfg.layout.standards_dir,
        schemas = cfg.layout.schemas_dir,
        state = cfg.layout.state_dir,
        cargo_workspace = cfg.layout.cargo_workspace,
        npm_workspaces = quoted(&cfg.layout.npm_workspaces),
        extra_hashed_inputs = quoted(&cfg.index.extra_hashed_inputs),
        resolver_exclusions = quoted(&cfg.index.resolver_exclusions),
        compiler_id = cfg.branding.compiler_id,
        indexer_id = cfg.branding.indexer_id,
        waiver = cfg.coupling.waiver_keyword,
        require_ownership = cfg.coupling.require_ownership,
        auto_waive = cfg.coupling.auto_waive_dependency_only,
        ordinal_monotonic = cfg.lint.require_ordinal_monotonic_depends_on,
    )
}

/// A TOML string array body: `"a", "b"`.
fn quoted(values: &[String]) -> String {
    values
        .iter()
        .map(|v| format!("\"{v}\""))
        .collect::<Vec<_>>()
        .join(", ")
}

/// The scaffolded `.gitignore` (spec 061 §3.1).
///
/// Every adopter independently learned that `build-meta.json` carries a wall
/// clock and dirties the tree, and spec 039's `state_dir` had the same missing
/// half: the live failure it fixed was a permanently dirty tree, not a
/// classification. Both paths come from `Config`, so a non-default
/// `derived_dir` or `state_dir` scaffolds coherently.
///
/// It deliberately does **not** ignore the shard trees. Whether they are
/// committed is the adopter's decision and both answers are legitimate;
/// ignoring them by default would silently opt every new adopter out of the
/// freshness gate, which is one of the system's two central mechanisms. The
/// file says so and names both options.
fn gitignore(cfg: &Config) -> String {
    let derived = cfg.layout.derived_dir.trim_end_matches('/');
    let mut out = format!(
        "# spec-spine writes wall-clock metadata here. It is the one\n\
         # non-deterministic artifact and is excluded from every determinism and\n\
         # golden check, so it must never be committed.\n\
         {derived}/**/build-meta.json\n"
    );
    let state = cfg.layout.state_dir.trim_end_matches('/');
    if !state.is_empty() {
        out.push_str(&format!(
            "\n# The declared state root (spec 039): a tool's own working files,\n\
             # ungoverned and outside every content hash.\n\
             {state}/\n"
        ));
    }
    out.push_str(&format!(
        "\n# NOT ignored, deliberately: {derived}/spec-registry/ and\n\
         # {derived}/codebase-index/ are the committed shard trees.\n\
         #\n\
         # Committing them is what makes `compile --check` and `index check` a\n\
         # freshness gate on a pull request: CI compares the corpus against what\n\
         # the branch committed. Not committing them means regenerating in CI\n\
         # before every gate step instead. Both are legitimate; the scaffold\n\
         # takes neither, because opting you out of the gate silently would be a\n\
         # decision made on your behalf. To choose the second, add:\n\
         #\n\
         #   {derived}/\n"
    ));
    out
}

fn bootstrap_spec(ns: &str) -> String {
    format!(
        "---\n\
         id: \"000-bootstrap\"\n\
         title: \"Bootstrap spec system\"\n\
         status: approved\n\
         # This spec defines what a spec is; it owns no code, so there is nothing\n\
         # to implement. `n-a` keeps `registry plan` from offering it (spec 045).\n\
         implementation: n-a\n\
         created: \"REPLACE-WITH-DATE\"\n\
         summary: >\n\
         \u{20}\u{20}Foundational contract: authored truth lives only in markdown (+ YAML\n\
         \u{20}\u{20}frontmatter); machine-consumable truth is compiler-emitted JSON only;\n\
         \u{20}\u{20}every artifact is a deterministic function of (config, file contents);\n\
         \u{20}\u{20}a typed authority graph governs who-owns-what.\n\
         origin:\n\
         \u{20}\u{20}retroactive: true   # authority held since before the graph existed\n\
         unamendable:\n\
         \u{20}\u{20}- \"markdown-truth-boundary\"\n\
         \u{20}\u{20}- \"json-truth-boundary\"\n\
         \u{20}\u{20}- \"determinism-requirement\"\n\
         \u{20}\u{20}- \"typed-authority-graph\"\n\
         \u{20}\u{20}- \"refusal-rule\"\n\
         ---\n\
         \n\
         # 000: Bootstrap spec system\n\
         \n\
         This is the spec that defines what a spec *is*. Customize it for your\n\
         repository, then author ordinary specs under your specs directory. Each\n\
         compilation unit links back here (or to a more specific spec) via\n\
         `[package.metadata.{ns}].spec` in its manifest, a `// Spec:` comment\n\
         header, or a spec's ownership edge.\n\
         \n\
         ## 1. The authoring / derived boundary\n\
         \n\
         Humans author markdown; the compiler owns the JSON. Never hand-edit a\n\
         derived artifact.\n\
         \n\
         ## 2. The typed authority graph\n\
         \n\
         Specs declare typed edges (`establishes`, `extends`, `refines`,\n\
         `supersedes`, `amends`, `co_authority`, `constrains`, `references`) and\n\
         the units they own (file / section / symbol / directory / crate / module).\n\
         Authority is derived by walking the graph.\n",
        ns = ns
    )
}

fn spec_template(ns: &str) -> String {
    format!(
        "---\n\
         id: \"NNN-slug\"                 # must equal the directory name\n\
         title: \"\"\n\
         status: draft                  # draft | approved | superseded | retired\n\
         implementation: pending        # pending | in-progress | complete | n-a | deferred\n\
         created: \"YYYY-MM-DD\"\n\
         summary: >\n\
         \u{20}\u{20}One paragraph: what this spec governs and why.\n\
         # Ownership edges (declare the units this spec owns):\n\
         establishes:\n\
         \u{20}\u{20}- \"path/to/file.rs\"                              # a file unit\n\
         \u{20}\u{20}# - {{ kind: section, file: \"Makefile\", anchor: \"build\" }}\n\
         \u{20}\u{20}# - {{ kind: symbol, id: \"my_crate::my_fn\" }}\n\
         \u{20}\u{20}# - {{ kind: directory, path: \"crates/my-crate/\" }}\n\
         \u{20}\u{20}# - {{ kind: crate, id: \"my-crate\" }}\n\
         \u{20}\u{20}# - {{ kind: module, id: \"my_crate::serialization\" }}\n\
         # depends_on:\n\
         #   - \"000-bootstrap\"\n\
         ---\n\
         \n\
         # NNN: Title\n\
         \n\
         Link a compilation unit to this spec via `[package.metadata.{ns}].spec`\n\
         in its manifest, a `// Spec:` header, or the edges above.\n\
         \n\
         ## 1. Purpose\n\
         ## 2. Territory\n\
         ## 3. Behavior\n\
         ## 4. Out of scope\n",
        ns = ns
    )
}

// Long enough that the escaped-continuation style of the shorter consts above
// would be a per-line hazard, so this one is a raw string at column 0.
const CONSTITUTION: &str = r#"# Constitution (tier 2)

Durable principles that govern this corpus. This document is **tier 2**: it is
subordinate to the bootstrap spec, whose `unamendable` anchors it may not
contradict, and it governs all ordinary specs.

**Normative hierarchy (highest wins):**

1. the bootstrap spec (`000`): non-overridable.
2. this constitution.
3. the contract: a normative summary of the bootstrap spec.
4. ordinary specs: feature-level claims within this envelope.

When two specs conflict, resolve in this order, then by the typed authority
graph.

---

## I. Markdown-only authored truth

Authored truth lives only in markdown with YAML frontmatter. If a fact governs
the system, it is written in a `spec.md` (or a standards document), never in a
derived artifact.

## II. Compiler-owned JSON machine truth

Machine-consumable truth is emitted by the compiler into the derived tree and is
read only through `spec-spine` subcommands. Hand-editing a derived artifact is a
workflow violation; ad-hoc parsing of one (`jq`/`awk`/`sed`) is equally
forbidden, because a typed read fails at the deserializer instead of silently
encoding a stale assumption.

## III. Spec-first development

A change to behavior begins with a change to a spec: the spec declares the units
it owns and the typed edges to its neighbours before the code is written. The
coupling gate enforces this at PR time. The escape valve is a named, scoped
waiver in the PR body, never a silent edit to an owner spec.

## IV. Determinism and validation

Every artifact-producing function is a pure function of (config, file contents):
the same inputs produce byte-identical output. Validation is mechanical, so
staleness is detectable by content-hash comparison alone.

## V. Legacy as evidence

Code that predates a governing spec is evidence, not a violation: a spec
claiming it declares `origin.retroactive: true` rather than masquerading as a
fresh `establishes` claim. Code adopted from outside the corpus is specced **as
found**, and the behavior the adopting spec would not have chosen is recorded
under a `## Known defects` heading. A defect recorded there is not thereby
blessed: it is what a later spec is written against.

---

## VI onward: the principles of the system you are specifying

Principles I through V govern the corpus and come from spec-spine. Number your
own from VI. They govern the system your corpus describes, and they bind every
spec equally. Keep them few. Freeze the ones you could not recover from by
naming their anchors in the bootstrap spec's `unamendable` list.

Replace this section with your first principle.

---

## Amendment

This constitution is changed by an ordinary spec that is `approved`, **claims
the affected text as an authority unit**, and contradicts no `unamendable`
anchor of the bootstrap spec.

The claim uses the ordinary ownership vocabulary over a section unit of this
file: `establishes` for a principle the spec adds, `refines` (with a named
`aspect`) for one it tightens, `co_authority` for one genuinely shared.

```yaml
refines:
  - aspect: "legacy-as-evidence"
    unit: { kind: section, file: "standards/spec/constitution.md", anchor: "v-legacy-as-evidence" }
```

The anchor is the heading slug, so `## V. Legacy as evidence` is
`v-legacy-as-evidence`. `amends` is **not** the instrument: its targets are spec
ids, and this file is not a spec.

Unlike an amended `spec.md`, which is a record of what the corpus held when it
was ratified and is therefore never edited to mention its successors, this
document is a standing statement of what is true now. It is edited in place, and
its history lives in the specs that claimed each section, and in git.
"#;

const CONTRACT: &str = "# Contract: normative summary\n\
\n\
- Specs live under the configured specs directory, one `NNN-slug/spec.md` each;\n\
  the directory name equals the frontmatter `id`.\n\
- `spec-spine compile` emits the registry; `spec-spine index` emits the codebase\n\
  index; `spec-spine lint` checks corpus conformance; `spec-spine couple` is the\n\
  PR-time gate.\n\
- A changed code path must be accompanied by an authoring edit to a spec that\n\
  owns it, or a `Spec-Drift-Waiver:` line in the PR body.\n\
- Read derived artifacts only through `spec-spine` subcommands; never parse the\n\
  JSON ad hoc.\n\
- An `amends` edge is declared once, in the amending spec's frontmatter; the\n\
  amended `spec.md` is not edited to record it.\n\
- The constitution is not amended by `amends` (its targets are spec ids). An\n\
  approved spec changes it by claiming the affected heading as a section unit\n\
  of that file; see the constitution's own Amendment section.\n";

/// The adopter-facing constitution template (spec 061 §3.3).
///
/// The two-bullet stub spec 043 complained about survived that spec, because
/// 043 §3.4 updated `CONSTITUTION` (the scaffolded document) and left the
/// **template** behind. Two adopters deleted what they were given. This is the
/// real thirty-four-line document: the tier statement, the normative hierarchy,
/// the amendment clause 043 made writable, and the seam saying which principles
/// are the adopter's own.
///
/// A literal rather than `include_str!` of the checked-in file, because that
/// file lives outside this crate and a published crate must be self-contained.
/// `tests/scaffold.rs` asserts the two agree, the same shape as the conformance
/// test that pins DTOs against the embedded schemas.
const CONSTITUTION_TEMPLATE: &str = "# <project> constitution\n\
\n\
Durable principles that govern this corpus. **Tier 2**: subordinate to the\n\
bootstrap spec (`specs/000-*/spec.md`) and governing all ordinary specs.\n\
\n\
**Normative hierarchy (highest wins):**\n\
\n\
1. `specs/000-*/spec.md`: the bootstrap spec. Non-overridable.\n\
2. `standards/spec/constitution.md`: this document.\n\
3. `standards/spec/contract.md`: normative summary of the bootstrap spec.\n\
4. Ordinary specs (`001`+).\n\
\n\
---\n\
\n\
## I. <Principle name>\n\
\n\
<One paragraph. State the principle as a durable rule, and cite the bootstrap\n\
anchor it rests on, if any.>\n\
\n\
## II. <Principle name>\n\
\n\
<...>\n\
\n\
## III. <Principle name>\n\
\n\
<...>\n\
\n\
---\n\
\n\
## Amendment\n\
\n\
This constitution may be amended by an ordinary spec that `amends` it and is\n\
approved, provided the amendment does not contradict a `specs/000` `unamendable`\n\
anchor.\n";

const ORCHESTRATOR_RULES: &str = "# Orchestrator rules\n\
\n\
- Execute phased work in order; stop at human checkpoints.\n\
- Write output files where the spec says; do not invent locations.\n\
- Keep the working tree green; never leave the coupling gate red.\n\
- Recompute derived artifacts (`spec-spine compile`, `spec-spine index`)\n\
\u{20}\u{20}before opening a PR, and commit the regenerated shards with the change that\n\
\u{20}\u{20}made them stale. A shard left uncommitted dirties the tree for whoever comes\n\
\u{20}\u{20}next.\n\
- One session, one spec: follow `AGENTS.md` \"Working the backlog\", then stop.\n";

const GOVERNED_READS: &str = "# Governed artifact reads\n\
\n\
The compiled artifacts under the derived directory are read **only** through\n\
`spec-spine` subcommands (`registry`, `index`), never via ad-hoc `jq`, `grep`,\n\
`python`, `awk`, or `sed` over the JSON. Typed reads make schema drift fail at\n\
the deserializer with a clean error instead of silently encoding stale\n\
assumptions.\n\
\n\
Parsing the *output* of a `spec-spine` subcommand (for example\n\
`spec-spine registry plan --json`, or the `--json` verdict envelope any gate\n\
verb emits) is a typed read and is allowed: the tool has already deserialized\n\
the shards and is answering in a contract it versions. The rule is about the\n\
shard files, not about the CLI's answers.\n";

const REFUSAL_RULE: &str = "# Adversarial prompt refusal (the coherence guard)\n\
\n\
If the coupling gate fails because code and its owning spec disagree, do **not**\n\
resolve it by editing the spec to match the code you just wrote. Surface the\n\
contradiction and let a human (or an agent with explicit authority recorded in\n\
the spec) decide. Never amend an owning spec purely to satisfy a mechanical\n\
refresh; waive instead, with a cited `Spec-Drift-Waiver:` line. A waiver is a\n\
human instrument: it needs explicit human approval, and an agent never writes\n\
one on its own authority.\n\
\n\
Two edits are always legitimate for the spec you are implementing: adding a\n\
file you created to its `establishes` list (the ownership ratchet refuses an\n\
unclaimed file, and the claim belongs in the same change), and recording a\n\
dated decision entry for a choice the spec was silent on. Changing what the\n\
spec *requires* is never yours to do mid-build. If the code needs to touch a\n\
unit another spec owns, declare an `extends` edge naming that spec and unit;\n\
that amends nobody.\n";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scaffolds_the_documented_file_set() {
        let s = scaffold_init(&Config::default()).unwrap();
        let paths: Vec<&str> = s.files.iter().map(|f| f.rel_path.as_str()).collect();
        assert!(paths.contains(&"spec-spine.toml"));
        assert!(paths.contains(&"standards/spec/constitution.md"));
        assert!(paths.contains(&"specs/000-bootstrap/spec.md"));
        assert!(paths.contains(&".claude/rules/adversarial-prompt-refusal.md"));
        // Default generator never forces an overwrite.
        assert!(s.files.iter().all(|f| !f.overwrite));
    }

    #[test]
    fn honors_non_default_layout_and_namespace() {
        let mut cfg = Config::default();
        cfg.manifest.metadata_namespace = "acme".to_string();
        cfg.layout.specs_dir = "contracts".to_string();
        let s = scaffold_init(&cfg).unwrap();
        let paths: Vec<&str> = s.files.iter().map(|f| f.rel_path.as_str()).collect();
        assert!(paths.contains(&"contracts/000-bootstrap/spec.md"));
        let toml = &s
            .files
            .iter()
            .find(|f| f.rel_path == "spec-spine.toml")
            .unwrap()
            .contents;
        assert!(toml.contains("metadata_namespace = \"acme\""));
    }
}
