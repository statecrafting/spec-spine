// Spec: specs/129-a-json-config-obeys-the-loaders-rules/spec.md
//! A configuration passed as JSON obeys the loader's rules (spec 129).
//!
//! `load_config` refuses a `[layout] derived_dir` that leaves the repository
//! (128), a `state_dir` that overlaps a governed root or names the whole
//! repository (036), and a malformed `[index.slices]` entry (011). Every facade
//! entry that takes a configuration as JSON used to run none of those rules, so
//! a binding could hand the engine a configuration no `spec-spine.toml` can
//! hold. Each case below names every such entry, so an entry added later
//! without the rule fails here rather than in an adopter.

use spec_spine_core::{
    attest_json, attest_snapshot_json, attest_spec_json, check_freshness_json, check_json,
    check_registry_freshness_json, closure_json, compact_json, compile_json, couple_json,
    coverage_inventory_json, coverage_json, delta_json, index_json, lint_json, render_json,
    scaffold_init, scaffold_init_json, scope_json, verify_attestation_json, verify_plan_json,
    verify_snapshot_attestation_json, verify_spec_attestation_json,
};
use spec_spine_types::{Config, Error, load_config};

/// A repository root nothing is ever read from: every refusal below must come
/// before the entry touches the filesystem.
const ROOT: &str = "/nonexistent/spec-spine-129";

/// Every facade entry that takes a configuration, called with `config` (a JSON
/// object). The request-shaped entries embed it; the others take it as their
/// first argument. Each entry's other inputs are well-formed, so a refusal can
/// only be about the configuration.
fn entries(config: &serde_json::Value) -> Vec<(&'static str, Result<String, Error>)> {
    let c = config.to_string();
    let commits = serde_json::json!({ "base": "b", "mergeBase": "m", "head": "h" });
    let request = |extra: serde_json::Value| {
        let mut r = serde_json::json!({ "config": config, "repoRoot": ROOT });
        for (k, v) in extra.as_object().unwrap() {
            r[k] = v.clone();
        }
        r.to_string()
    };
    vec![
        ("compile_json", compile_json(&c, ROOT)),
        ("closure_json", closure_json(&c, ROOT, "{}")),
        ("scope_json", scope_json(&c, ROOT, r#"{"ownSpec":"001-a"}"#)),
        ("index_json", index_json(&c, ROOT)),
        ("lint_json", lint_json(&c, ROOT)),
        ("check_json", check_json(&c, ROOT)),
        ("check_freshness_json", check_freshness_json(&c, ROOT)),
        (
            "check_registry_freshness_json",
            check_registry_freshness_json(&c, ROOT),
        ),
        ("coverage_json", coverage_json(&c, ROOT)),
        ("verify_plan_json", verify_plan_json(&c, ROOT, "001-a")),
        ("render_json", render_json(&c, "{}")),
        ("scaffold_init_json", scaffold_init_json(&c)),
        ("compact_json", compact_json(&c, ROOT, "remove: []\n")),
        ("attest_json", attest_json(&c, ROOT, false)),
        ("attest_spec_json", attest_spec_json(&c, ROOT, "001-a")),
        ("attest_snapshot_json", attest_snapshot_json(&c, ROOT)),
        (
            "coverage_inventory_json",
            coverage_inventory_json(&request(serde_json::json!({}))),
        ),
        (
            "couple_json",
            couple_json(&request(serde_json::json!({ "diff": { "files": [] } }))),
        ),
        (
            "delta_json",
            delta_json(
                &serde_json::json!({
                    "config": config,
                    "baseRoot": ROOT,
                    "headRoot": ROOT,
                    "changed": [],
                    "commits": commits,
                })
                .to_string(),
            ),
        ),
        (
            "verify_snapshot_attestation_json",
            verify_snapshot_attestation_json(&request(
                serde_json::json!({ "attestationText": "{}" }),
            )),
        ),
        (
            "verify_spec_attestation_json",
            verify_spec_attestation_json(&request(serde_json::json!({ "attestationText": "{}" }))),
        ),
        (
            "verify_attestation_json",
            verify_attestation_json(&request(serde_json::json!({ "attestationText": "{}" }))),
        ),
    ]
}

/// The message `load_config` gives for the same value written as TOML. The
/// facade must refuse with exactly this, so there is one rule and one wording.
fn loader_message(toml: &str) -> String {
    match load_config(toml) {
        Err(Error::Config(m)) => m,
        other => panic!("the loader must refuse {toml:?}, got {other:?}"),
    }
}

fn assert_every_entry_refuses(config: serde_json::Value, toml: &str) {
    let expected = loader_message(toml);
    let mut wrong = Vec::new();
    for (name, result) in entries(&config) {
        match result {
            Err(Error::Config(m)) if m == expected => {}
            other => wrong.push(format!("{name}: {other:?}")),
        }
    }
    assert!(
        wrong.is_empty(),
        "every entry must refuse {config} with the loader's message\n  {expected}\n\
         these did not:\n  {}",
        wrong.join("\n  ")
    );
}

// ===== 3.1: the loader's rules, at every entry =====

#[test]
fn an_escaping_derived_dir_is_refused_at_every_entry() {
    for (json, toml) in [
        ("../outside", "derived_dir = '../outside'"),
        ("/abs/outside", "derived_dir = '/abs/outside'"),
        ("x/../../outside", "derived_dir = 'x/../../outside'"),
        ("..", "derived_dir = '..'"),
        ("..\\outside", "derived_dir = '..\\outside'"),
        ("C:outside", "derived_dir = 'C:outside'"),
    ] {
        assert_every_entry_refuses(
            serde_json::json!({ "layout": { "derived_dir": json } }),
            &format!("[layout]\n{toml}\n"),
        );
    }
}

#[test]
fn a_state_dir_the_loader_refuses_is_refused_at_every_entry() {
    for value in [".", "/", "specs", ".derived/state", "../state", "/abs"] {
        assert_every_entry_refuses(
            serde_json::json!({ "layout": { "state_dir": value } }),
            &format!("[layout]\nstate_dir = '{value}'\n"),
        );
    }
    // The comparison is against the resolved roots, not their defaults (036).
    assert_every_entry_refuses(
        serde_json::json!({ "layout": { "specs_dir": "corpus", "state_dir": "corpus/state" } }),
        "[layout]\nspecs_dir = 'corpus'\nstate_dir = 'corpus/state'\n",
    );
}

#[test]
fn a_malformed_slice_is_refused_at_every_entry() {
    assert_every_entry_refuses(
        serde_json::json!({ "index": { "slices": { "Api": ["src/**/*"] } } }),
        "[index.slices]\nApi = ['src/**/*']\n",
    );
    assert_every_entry_refuses(
        serde_json::json!({ "index": { "slices": { "api": [] } } }),
        "[index.slices]\napi = []\n",
    );
}

// ===== 3.2: what stays accepted =====

/// No entry refuses these as a configuration. Most still fail, because
/// [`ROOT`] does not exist; the assertion is only that the failure is not a
/// configuration refusal.
#[test]
fn accepted_configurations_are_not_refused_at_any_entry() {
    for config in [
        serde_json::json!({}),
        serde_json::json!({ "layout": { "derived_dir": ".derived" } }),
        serde_json::json!({ "layout": { "derived_dir": ".statecraft/derived" } }),
        serde_json::json!({ "layout": { "derived_dir": "./.derived/" } }),
        serde_json::json!({ "layout": { "derived_dir": "" } }),
        serde_json::json!({ "layout": { "derived_dir": "." } }),
        serde_json::json!({ "layout": { "derived_dir": "a..b" } }),
        serde_json::json!({ "layout": { "state_dir": ".statecraft/state" } }),
        serde_json::json!({ "index": { "slices": { "api": ["src/**/*"] } } }),
        statecraft_config(),
    ] {
        assert!(
            load_config(&toml_of(&config)).is_ok(),
            "control: the loader accepts {config}"
        );
        for (name, result) in entries(&config) {
            assert!(
                !matches!(result, Err(Error::Config(_))),
                "{name} must accept {config}, got {result:?}"
            );
        }
    }
}

/// The configuration the Statecraft CLI passes to [`scaffold_init_json`],
/// copied from its `statecraft-home/src/producer.rs` `config_json()` as
/// measured at statecraft-cli `fa000c6`. Every layout key is given
/// explicitly, never defaulted.
fn statecraft_config() -> serde_json::Value {
    serde_json::json!({
        "layout": {
            "specs_dir": "specs",
            "derived_dir": ".statecraft/derived",
            "standards_dir": "standards/spec",
            "schemas_dir": "standards/schemas",
            "cargo_workspace": "Cargo.toml",
            "npm_workspaces": ["package.json", "pnpm-workspace.yaml"],
            "standalone_rust_workspaces": [],
            "standalone_npm_packages": [],
            "state_dir": ".statecraft/state"
        }
    })
}

/// The same configuration as TOML, for the loader control.
fn toml_of(config: &serde_json::Value) -> String {
    let parsed: Config = serde_json::from_value(config.clone()).unwrap();
    toml::to_string(&parsed).unwrap()
}

#[test]
fn the_statecraft_call_shape_scaffolds_unchanged() {
    let out = scaffold_init_json(&statecraft_config().to_string()).unwrap();
    let scaffold: serde_json::Value = serde_json::from_str(&out).unwrap();
    let files = scaffold["files"].as_array().unwrap();
    let toml = files
        .iter()
        .find(|f| f["relPath"] == "spec-spine.toml")
        .unwrap()["contents"]
        .as_str()
        .unwrap();
    let reloaded = load_config(toml).unwrap();
    assert_eq!(reloaded.layout.derived_dir, ".statecraft/derived");
    assert_eq!(reloaded.layout.state_dir, ".statecraft/state");
    let mut paths: Vec<&str> = files
        .iter()
        .map(|f| f["relPath"].as_str().unwrap())
        .collect();
    paths.sort_unstable();
    assert_eq!(
        paths,
        [
            ".gitignore",
            "spec-spine.toml",
            "specs/000-bootstrap/spec.md",
            "standards/spec/constitution.md",
            "standards/spec/contract.md",
            "standards/spec/templates/constitution-template.md",
            "standards/spec/templates/spec-template.md",
        ]
    );
}

// ===== 3.3: the scaffold emits a configuration its loader reads back =====

/// A value the facade accepts must reach the scaffolded `spec-spine.toml`
/// intact: the file loads, and every key the scaffold writes reads back as
/// given. A `"` used to end the TOML string early, and a `\` was read back as
/// an escape.
#[test]
fn every_scaffolded_value_reads_back_as_given() {
    for value in [
        "has\"quote",
        "back\\slash",
        "tab\tand\u{7f}del",
        "line\nbreak",
        "plain",
    ] {
        let mut cfg = Config::default();
        cfg.manifest.metadata_namespace = value.to_string();
        cfg.layout.specs_dir = format!("specs-{value}");
        cfg.layout.standards_dir = format!("std-{value}");
        cfg.layout.schemas_dir = format!("schemas-{value}");
        cfg.layout.cargo_workspace = format!("{value}/Cargo.toml");
        cfg.layout.npm_workspaces = vec![value.to_string()];
        cfg.index.extra_hashed_inputs = vec![format!("{value}/**/*")];
        cfg.index.resolver_exclusions = vec![value.to_string()];
        cfg.branding.compiler_id = value.to_string();
        cfg.branding.indexer_id = value.to_string();
        cfg.coupling.waiver_keyword = value.to_string();
        let scaffold = scaffold_init(&cfg).unwrap();
        let toml = &scaffold
            .files
            .iter()
            .find(|f| f.rel_path == "spec-spine.toml")
            .unwrap()
            .contents;
        let back = load_config(toml)
            .unwrap_or_else(|e| panic!("the scaffold for {value:?} must load, got {e:?}"));
        assert_eq!(back.manifest, cfg.manifest, "{value:?}");
        assert_eq!(back.layout.specs_dir, cfg.layout.specs_dir, "{value:?}");
        assert_eq!(back.layout.standards_dir, cfg.layout.standards_dir);
        assert_eq!(back.layout.schemas_dir, cfg.layout.schemas_dir);
        assert_eq!(back.layout.cargo_workspace, cfg.layout.cargo_workspace);
        assert_eq!(back.layout.npm_workspaces, cfg.layout.npm_workspaces);
        assert_eq!(
            back.index.extra_hashed_inputs,
            cfg.index.extra_hashed_inputs
        );
        assert_eq!(
            back.index.resolver_exclusions,
            cfg.index.resolver_exclusions
        );
        assert_eq!(back.branding, cfg.branding);
        assert_eq!(back.coupling.waiver_keyword, cfg.coupling.waiver_keyword);
    }
}

/// The public Rust producer refuses what the JSON producer refuses: both
/// return a configuration file, and neither may return one its loader
/// refuses.
#[test]
fn the_rust_scaffold_refuses_what_the_loader_refuses() {
    let mut cfg = Config::default();
    cfg.layout.derived_dir = "../outside".to_string();
    let expected = loader_message("[layout]\nderived_dir = '../outside'\n");
    match scaffold_init(&cfg) {
        Err(Error::Config(m)) => assert_eq!(m, expected),
        other => panic!("scaffold_init must refuse, got {other:?}"),
    }
}
