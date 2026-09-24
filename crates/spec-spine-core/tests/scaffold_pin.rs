//! Spec 131: the scaffold can pin its producer, exactly.
//!
//! Statecraft writes the `spec-spine.toml` the scaffold returns byte for byte,
//! and that file pins nothing. An opt-in option emits an active `[meta]` table
//! whose `required_version` is `=<this producer's version>`. Without the option
//! the output is unchanged, byte for byte.

use spec_spine_core::{
    ScaffoldOptions, scaffold_init, scaffold_init_json, scaffold_init_with_options,
    scaffold_init_with_options_json,
};
use spec_spine_types::{Config, Error, load_config};

const PRODUCER: &str = env!("CARGO_PKG_VERSION");

/// The layout Statecraft passes today (its `producer::config_json`), verbatim.
const STATECRAFT_CONFIG_JSON: &str = r#"{"layout":{"specs_dir":"specs","derived_dir":".statecraft/derived","standards_dir":"standards/spec","schemas_dir":"standards/schemas","cargo_workspace":"Cargo.toml","npm_workspaces":["package.json","pnpm-workspace.yaml"],"standalone_rust_workspaces":[],"standalone_npm_packages":[],"state_dir":".statecraft/state"}}"#;

fn toml_of(json: &str) -> String {
    let v: serde_json::Value = serde_json::from_str(json).unwrap();
    v["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["relPath"] == "spec-spine.toml")
        .unwrap()["contents"]
        .as_str()
        .unwrap()
        .to_string()
}

fn pinned() -> ScaffoldOptions {
    ScaffoldOptions {
        pin_exact_version: true,
    }
}

/// §3.1: the option emits an active `[meta]` table pinning this producer
/// exactly, and the file loads.
#[test]
fn the_pinned_scaffold_carries_an_active_exact_pin() {
    let out =
        scaffold_init_with_options_json(STATECRAFT_CONFIG_JSON, r#"{"pinExactVersion":true}"#)
            .unwrap();
    let toml = toml_of(&out);
    assert!(
        toml.contains("\n[meta]\n"),
        "an active table header:\n{toml}"
    );
    let line = format!("\nrequired_version = \"={PRODUCER}\"\n");
    assert!(toml.contains(&line), "an active exact pin:\n{toml}");
    let cfg = load_config(&toml).expect("the pinned file loads");
    assert_eq!(
        cfg.meta.required_version.as_deref(),
        Some(format!("={PRODUCER}").as_str())
    );
}

/// §3.1: the pin is exact. The producer's own version satisfies it and its
/// neighbours do not.
#[test]
fn the_pin_admits_only_the_producer_version() {
    let files = scaffold_init_with_options(&Config::default(), &pinned()).unwrap();
    let cfg = load_config(&files.files[0].contents).unwrap();
    cfg.check_required_version(PRODUCER)
        .expect("the producer satisfies its own pin");
    let (maj, rest) = PRODUCER.split_once('.').unwrap();
    let (min, patch) = rest.split_once('.').unwrap();
    let patch: u64 = patch.parse().unwrap();
    let min_n: u64 = min.parse().unwrap();
    for other in [
        format!("{maj}.{min}.{}", patch + 1),
        format!("{maj}.{}.0", min_n + 1),
    ] {
        assert!(
            cfg.check_required_version(&other).is_err(),
            "{other} must not satisfy the exact pin"
        );
    }
}

/// §3.2: without the option, and with the option off, the output is exactly
/// what `scaffold_init_json` returns, which is what it returned before.
#[test]
fn the_default_scaffold_is_unchanged() {
    let plain = scaffold_init_json(STATECRAFT_CONFIG_JSON).unwrap();
    for options in ["{}", r#"{"pinExactVersion":false}"#] {
        assert_eq!(
            scaffold_init_with_options_json(STATECRAFT_CONFIG_JSON, options).unwrap(),
            plain,
            "{options}"
        );
    }
    let toml = toml_of(&plain);
    assert!(toml.contains("\n# [meta]\n"), "{toml}");
    assert!(toml.contains(&format!("\n# required_version = \"{PRODUCER}\"\n")));
    assert!(!toml.contains("\n[meta]\n"));
}

/// §3.2: the pinned file differs from the default only in the `[meta]` block.
/// Every other line, and every other file, is the same bytes.
#[test]
fn the_pin_changes_only_the_meta_block() {
    let cfg = Config::default();
    let plain = scaffold_init(&cfg).unwrap();
    let pin = scaffold_init_with_options(&cfg, &pinned()).unwrap();
    assert_eq!(plain.files.len(), pin.files.len());
    for (a, b) in plain.files.iter().zip(&pin.files).skip(1) {
        assert_eq!(a, b, "{} must not change", a.rel_path);
    }
    let after_meta = |s: &str| s[s.find("\n[manifest]\n").unwrap()..].to_string();
    let (a, b) = (&plain.files[0].contents, &pin.files[0].contents);
    assert_eq!(after_meta(a), after_meta(b));
    let head_a = &a[..a.find("\n# [meta]\n").unwrap()];
    let head_b = &b[..b.find("\n[meta]\n").unwrap()];
    assert_eq!(head_a, head_b, "the header comment is unchanged");
}

/// §3.3: the options are validated like the configuration. An unknown key is a
/// refusal, not a silently unpinned file, and the configuration rules still run.
#[test]
fn unknown_options_and_invalid_configs_are_refused() {
    let err = scaffold_init_with_options_json("{}", r#"{"pinExact":true}"#).unwrap_err();
    assert!(matches!(err, Error::Config(_)), "{err:?}");
    let err = scaffold_init_with_options_json("{}", "not json").unwrap_err();
    assert!(matches!(err, Error::Config(_)), "{err:?}");
    let bad = r#"{"layout":{"derived_dir":"../outside"}}"#;
    assert_eq!(
        format!(
            "{:?}",
            scaffold_init_with_options_json(bad, r#"{"pinExactVersion":true}"#).unwrap_err()
        ),
        format!("{:?}", scaffold_init_json(bad).unwrap_err()),
        "the same refusal as the unpinned entry"
    );
}
