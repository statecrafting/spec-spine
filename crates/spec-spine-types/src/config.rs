//! The `spec-spine.toml` configuration model.
//!
//! Everything the reference repos had to fork over is a knob here. An absent
//! config yields a working default for a single-Cargo-workspace repo with
//! `specs/` at the root ([`Config::default`]). Every struct is
//! `#[serde(default, deny_unknown_fields)]`: missing keys default, and a
//! *misspelled* knob is a loud [`Error::Config`] rather than a silently-ignored
//! setting: the exact failure class that left template-encore blind to its npm
//! packages. See `docs/design/00-architecture.md` §3.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// The full configuration. All sections are optional. `Default` is derived;
/// each field's own `Default` supplies the conventional value.
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub manifest: ManifestConfig,
    /// Opt-in `domain` taxonomy (empty `allowed` ⇒ free-text/disabled).
    pub domains: AllowlistConfig,
    /// Opt-in `kind` taxonomy, symmetric with `domains` (empty ⇒ disabled).
    pub kind: AllowlistConfig,
    pub layout: LayoutConfig,
    pub index: IndexConfig,
    pub branding: BrandingConfig,
    pub coupling: CouplingConfig,
    pub provenance: ProvenanceConfig,
    pub frontmatter: FrontmatterConfig,
}

/// `[manifest]`: how a manifest links a compilation unit back to its spec.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ManifestConfig {
    /// Drives both `[package.metadata.<ns>].spec` (Cargo) and `"<ns>".spec`
    /// (package.json). OAP used `oap`; aide/encore used `spec`.
    pub metadata_namespace: String,
}

impl Default for ManifestConfig {
    fn default() -> Self {
        ManifestConfig {
            metadata_namespace: "spec-spine".to_string(),
        }
    }
}

/// A reusable opt-in categorical allowlist (used by `[domains]` and `[kind]`).
///
/// Empty ⇒ the field is free-text / disabled (no enum check). Non-empty ⇒ a
/// closed enum: the field value, *when present*, must be a member (a `V`-error
/// otherwise). Field absence is allowed.
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AllowlistConfig {
    pub allowed: Vec<String>,
}

impl AllowlistConfig {
    /// True if this taxonomy is disabled (no allowlist configured).
    pub fn is_disabled(&self) -> bool {
        self.allowed.is_empty()
    }

    /// True if `value` is permitted: always when disabled, else membership.
    pub fn permits(&self, value: &str) -> bool {
        self.is_disabled() || self.allowed.iter().any(|a| a == value)
    }
}

/// `[layout]`: path conventions. Never hardcode `specs/`, `.derived/`, etc.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LayoutConfig {
    pub specs_dir: String,
    pub derived_dir: String,
    pub standards_dir: String,
    pub schemas_dir: String,
    /// Root Cargo workspace manifest (relative to repo root).
    pub cargo_workspace: String,
    /// Manifests that DECLARE npm/pnpm workspace members. The indexer reads
    /// member globs from whichever exists. The default reads root
    /// `package.json#workspaces`, fixing the template-encore bug where a
    /// hardcoded `public/pnpm-workspace.yaml` made all npm packages invisible.
    pub npm_workspaces: Vec<String>,
    /// Crates outside the root Cargo workspace.
    pub standalone_rust_workspaces: Vec<String>,
    /// npm packages outside the declared workspaces.
    pub standalone_npm_packages: Vec<String>,
    /// A declared, ungoverned tool-state root (spec 039). Empty means **no
    /// state root is declared**, and every behavior keyed on it is inert.
    ///
    /// Declared: the gates recognize it, so `couple` bypasses it, `coverage`
    /// excludes it from classification, the resolver does not scan it, and it
    /// contributes to no content hash. Ungoverned: spec-spine never reads its
    /// contents and never writes into it, so the purity invariant holds (a root
    /// read from would be a second input to functions contracted to be pure in
    /// the corpus, and one written to would make a read command mutate the tree).
    ///
    /// A default of `.spec-spine/` was rejected: silently bypassing a real path
    /// in every adopter's repo on upgrade changes what the gate refuses, and a
    /// gate that quietly stops refusing something is the regression this project
    /// can least afford.
    pub state_dir: String,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        LayoutConfig {
            specs_dir: "specs".to_string(),
            derived_dir: ".derived".to_string(),
            standards_dir: "standards/spec".to_string(),
            schemas_dir: "standards/schemas".to_string(),
            cargo_workspace: "Cargo.toml".to_string(),
            npm_workspaces: vec![
                "package.json".to_string(),
                "pnpm-workspace.yaml".to_string(),
            ],
            standalone_rust_workspaces: Vec::new(),
            standalone_npm_packages: Vec::new(),
            state_dir: String::new(),
        }
    }
}

impl LayoutConfig {
    /// Whether `path` (repo-relative, POSIX) lies under the declared state root.
    ///
    /// Always false when no root is declared. Matching is separator-aware, so a
    /// root of `state` covers `state` itself and `state/journal.db` but never
    /// `stateful/x`: a prefix test on the raw string would silently ungovern a
    /// sibling directory that merely shares a name.
    pub fn is_state_path(&self, path: &str) -> bool {
        let root = trim_root(&self.state_dir);
        if root.is_empty() {
            return false;
        }
        let path = trim_root(path);
        path == root || path.strip_prefix(root).is_some_and(|r| r.starts_with('/'))
    }
}

/// A layout root reduced to its comparable form: no trailing slash, and no
/// leading `./`, so `state`, `state/` and `./state` name one root (the handling
/// spec 036 established for `specs_dir`).
fn trim_root(value: &str) -> &str {
    value
        .trim_end_matches('/')
        .strip_prefix("./")
        .unwrap_or_else(|| value.trim_end_matches('/'))
}

/// `[index]`: inputs and exclusions for the codebase indexer.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct IndexConfig {
    /// Globs folded into the content-hash beyond the always-hashed core
    /// (all spec.md + discovered manifests + `spec-spine.toml`).
    pub extra_hashed_inputs: Vec<String>,
    /// Directory names pruned from symbol/section resolution walks.
    pub resolver_exclusions: Vec<String>,
    /// `[index.slices]` (spec 012): named glob groups, each emitted as a
    /// `build.sliceHashes` entry and gated by `index check --slice <name>`.
    /// Names match `[a-z0-9][a-z0-9-]*`; each list is non-empty, with
    /// `extra_hashed_inputs` pattern semantics. Slices are independent of the
    /// global hash: listing a file here does NOT fold it into `contentHash`.
    pub slices: BTreeMap<String, Vec<String>>,
}

impl Default for IndexConfig {
    fn default() -> Self {
        IndexConfig {
            extra_hashed_inputs: vec![
                "standards/**".to_string(),
                ".github/workflows/**".to_string(),
            ],
            slices: BTreeMap::new(),
            resolver_exclusions: vec![
                "target".to_string(),
                "node_modules".to_string(),
                ".derived".to_string(),
                "dist".to_string(),
                "build".to_string(),
                ".next".to_string(),
            ],
        }
    }
}

/// `[branding]`: identifiers stamped into emitted `build` metadata.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BrandingConfig {
    pub compiler_id: String,
    pub indexer_id: String,
}

impl Default for BrandingConfig {
    fn default() -> Self {
        BrandingConfig {
            compiler_id: "spec-spine".to_string(),
            indexer_id: "spec-spine".to_string(),
        }
    }
}

/// `[coupling]`: the PR-time gate's exemptions and waiver keyword.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CouplingConfig {
    /// **Additional** paths exempt from the gate, on top of the always-applied
    /// hardcoded floor (`spec_spine_core::DEFAULT_BYPASS_PREFIXES`). Match rules:
    /// trailing `/` ⇒ dir prefix; leading `**/` ⇒ tail-suffix anywhere; else
    /// exact file. This list is **additive**: it adds to the floor and can never
    /// remove a floor entry. The default is **empty**: the floor is the single
    /// built-in source, so an adopter declares only their own additions rather
    /// than restating (and seeming able to override) the floor.
    pub bypass_prefixes: Vec<String>,
    /// The PR-body waiver keyword; the free-text reason follows the colon.
    pub waiver_keyword: String,
    /// Opt-in mechanical auto-waiver for dependency-only diffs (spec 005
    /// §3.5). When `true` and no PR-body waiver is present, the CLI compares
    /// the parsed base/head JSON of every non-bypassed changed path: if all
    /// are `package.json` manifests whose only differences are version
    /// strings inside the standard dependency tables (same package keys),
    /// the gate self-waives: the path dependabot-class PRs cannot take
    /// (they can edit neither specs nor PR bodies). Anything beyond a
    /// version string (a new package, a `scripts` edit, spec-binding
    /// metadata) refuses the auto-waiver, fail-closed. Default `false`.
    pub auto_waive_dependency_only: bool,
    /// Opt-in ownership ratchet (spec 032). When `true`, a changed source file
    /// inside a discovered package that no spec **specifically** claims is a
    /// `C-002` violation instead of being skipped as "not a coupling concern".
    /// Specific means a resolved ownership-bearing unit (file / section /
    /// symbol / directory / crate / module) or a `// Spec:` comment header;
    /// a manifest floor alone (`[package.metadata.<ns>].spec`, spec 005 §3.6)
    /// does not count, because it covers a whole package regardless of what
    /// anyone has thought about. Default `false`: `C-001` holds for any
    /// corpus the day it is written, but full coverage is a state a repo has
    /// to reach first. `spec-spine index coverage` reports the distance.
    pub require_ownership: bool,
}

impl Default for CouplingConfig {
    fn default() -> Self {
        CouplingConfig {
            // Empty by design: the floor lives in `couple.rs` and is always
            // unioned in; duplicating it here was redundant and misleadingly
            // implied it was overridable.
            bypass_prefixes: Vec::new(),
            waiver_keyword: "Spec-Drift-Waiver:".to_string(),
            auto_waive_dependency_only: false,
            require_ownership: false,
        }
    }
}

/// `[provenance]`: the OPEN provenance-scheme registry (kind → URI scheme).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ProvenanceConfig {
    pub uri_schemes: BTreeMap<String, String>,
}

impl Default for ProvenanceConfig {
    fn default() -> Self {
        let mut uri_schemes = BTreeMap::new();
        uri_schemes.insert("knowledge".to_string(), "knowledge://".to_string());
        uri_schemes.insert("code-fingerprint".to_string(), "fingerprint://".to_string());
        ProvenanceConfig { uri_schemes }
    }
}

/// `[frontmatter]`: recognized-key extensions.
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct FrontmatterConfig {
    /// Keys an adopter recognizes (suppresses the lint's unknown-key warning);
    /// they still overflow into `extra_frontmatter`.
    pub extra_known_keys: Vec<String>,
}

/// Load and validate a `spec-spine.toml` from its source text.
///
/// Returns [`Error::Config`] (mapped to exit code 3) on any malformed or
/// unknown-key error; never panics.
pub fn load_config(toml_src: &str) -> Result<Config> {
    let config: Config = toml::from_str(toml_src).map_err(|e| Error::Config(e.to_string()))?;
    validate_slices(&config)?;
    validate_state_dir(&config)?;
    Ok(config)
}

/// `layout.state_dir` may not overlap `specs_dir` or `derived_dir` in either
/// direction (spec 039 3.2).
///
/// The comparison is against the **resolved** values of those two keys, never
/// against their defaults. Both are configurable, so a check written against the
/// literal `specs` would clear a repo with `specs_dir = "corpus"` and
/// `state_dir = "corpus/state"` and quietly make every `spec.md` under it
/// ungoverned. This is the defect spec 036 fixed in `couple.rs`, and a
/// validation rule is exactly where a default is easiest to hardcode.
///
/// The test is overlap, not equality, because the dangerous values are the ones
/// that *contain* a root: a `state_dir` of `.` ungoverns the whole repository,
/// and every gate then keeps exiting 0 while adjudicating nothing. The
/// descendant direction is refused too, since a path inside a governed root and
/// an ungoverned one at once would need a precedence rule for a situation with
/// no legitimate use, and refusing the configuration is cheaper than specifying
/// which root wins.
fn validate_state_dir(config: &Config) -> Result<()> {
    let state = trim_root(&config.layout.state_dir);
    if state.is_empty() {
        return Ok(());
    }
    // The repo root contains every governed root, so it fails the overlap test
    // below only if that test knows `.` is an ancestor of everything, which a
    // string comparison does not. Refused by name instead: this is the value
    // spec 039 3.2 calls the worst outcome, since every gate would keep exiting
    // 0 while adjudicating nothing at all.
    if state == "." {
        return Err(Error::Config(format!(
            "layout.state_dir '{}' is the repository root: a state root is ungoverned, \
             so declaring the whole repository one would silence every gate",
            config.layout.state_dir
        )));
    }
    // A value escaping the repository is worse than wrong, it is inert: every
    // path the gates test is repo-relative and carries no `..`, so nothing would
    // ever match and the gates would behave as though no root were declared
    // while the config says one is. Silence that claims to be a decision is the
    // failure mode this key exists to prevent.
    if state == ".." || state.starts_with("../") {
        return Err(Error::Config(format!(
            "layout.state_dir '{}' escapes the repository: the value is a repo-relative \
             directory, and one outside it would match no path and silently declare nothing",
            config.layout.state_dir
        )));
    }
    for (key, value) in [
        ("specs_dir", &config.layout.specs_dir),
        ("derived_dir", &config.layout.derived_dir),
    ] {
        let other = trim_root(value);
        if other.is_empty() {
            continue;
        }
        let contains =
            |a: &str, b: &str| a == b || b.strip_prefix(a).is_some_and(|r| r.starts_with('/'));
        if contains(state, other) || contains(other, state) {
            return Err(Error::Config(format!(
                "layout.state_dir '{}' overlaps layout.{key} '{}': a state root is ungoverned, \
                 so it may not equal, contain, or sit inside a governed root",
                config.layout.state_dir, value
            )));
        }
    }
    Ok(())
}

/// `[index.slices]` grammar (spec 012 §3.1): names match
/// `[a-z0-9][a-z0-9-]*`, glob lists are non-empty.
fn validate_slices(config: &Config) -> Result<()> {
    for (name, globs) in &config.index.slices {
        let mut chars = name.chars();
        let head_ok = chars
            .next()
            .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit());
        let tail_ok = chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
        if !(head_ok && tail_ok) {
            return Err(Error::Config(format!(
                "[index.slices] name '{name}' must match [a-z0-9][a-z0-9-]*"
            )));
        }
        if globs.is_empty() {
            return Err(Error::Config(format!(
                "[index.slices] '{name}' must list at least one glob"
            )));
        }
    }
    Ok(())
}
