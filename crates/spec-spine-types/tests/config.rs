// Spec: specs/000-spec-spine-bootstrap/spec.md
//! Config tests: absent (defaults), minimal, full, malformed (clean error).

use spec_spine_types::{Config, Error, load_config};

#[test]
fn default_config_has_expected_values() {
    let c = Config::default();
    assert_eq!(c.manifest.metadata_namespace, "spec-spine");
    assert!(c.domains.allowed.is_empty());
    assert!(c.kind.allowed.is_empty());
    assert_eq!(c.layout.specs_dir, "specs");
    assert_eq!(c.layout.derived_dir, ".derived");
    assert_eq!(c.layout.standards_dir, "standards/spec");
    assert_eq!(c.layout.cargo_workspace, "Cargo.toml");
    // The encore-bug fix: the default reads root package.json#workspaces.
    assert!(
        c.layout
            .npm_workspaces
            .contains(&"package.json".to_string())
    );
    assert!(
        c.layout
            .npm_workspaces
            .contains(&"pnpm-workspace.yaml".to_string())
    );
    assert_eq!(c.branding.compiler_id, "spec-spine");
    assert_eq!(c.coupling.waiver_keyword, "Spec-Drift-Waiver:");
    assert_eq!(
        c.provenance
            .uri_schemes
            .get("code-fingerprint")
            .map(String::as_str),
        Some("fingerprint://")
    );
}

#[test]
fn absent_config_equals_default() {
    // An empty document yields a working default for a conventional repo.
    let c = load_config("").expect("empty config must load");
    assert_eq!(c, Config::default());
}

#[test]
fn minimal_config_overrides_one_knob() {
    let c = load_config("[manifest]\nmetadata_namespace = \"oap\"\n").unwrap();
    assert_eq!(c.manifest.metadata_namespace, "oap");
    // Everything else stays at default.
    assert_eq!(c.layout.specs_dir, "specs");
}

#[test]
fn allowlist_semantics() {
    let c = load_config("[domains]\nallowed = [\"app\", \"substrate\"]\n").unwrap();
    assert!(!c.domains.is_disabled());
    assert!(c.domains.permits("app"));
    assert!(!c.domains.permits("platform"));
    // Disabled allowlist permits anything.
    assert!(Config::default().kind.is_disabled());
    assert!(Config::default().kind.permits("anything-goes"));
}

#[test]
fn full_config_round_trips() {
    // Serialize a non-default config to TOML and load it back.
    let mut original = Config::default();
    original.manifest.metadata_namespace = "myns".to_string();
    original.domains.allowed = vec!["app".to_string(), "tooling".to_string()];
    original.kind.allowed = vec!["feature".to_string()];
    let toml_src = toml::to_string(&original).unwrap();
    let reloaded = load_config(&toml_src).unwrap();
    assert_eq!(original, reloaded);
}

#[test]
fn malformed_config_is_clean_error_not_panic() {
    // Unknown top-level section.
    let e = load_config("[bogus_section]\nx = 1\n").unwrap_err();
    assert!(matches!(e, Error::Config(_)));
    assert_eq!(e.exit_code(), 3);

    // Unknown key in a known section (the typo'd-knob failure class).
    let e = load_config("[manifest]\nmetadata_namspace = \"x\"\n").unwrap_err();
    assert!(matches!(e, Error::Config(_)));

    // Wrong type.
    let e = load_config("[domains]\nallowed = \"not-a-list\"\n").unwrap_err();
    assert!(matches!(e, Error::Config(_)));
}

#[test]
fn this_repos_spec_spine_toml_loads() {
    // Dogfood: the committed config for this very repo must parse under
    // deny_unknown_fields (catches struct/TOML drift).
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../spec-spine.toml");
    let src = std::fs::read_to_string(root).expect("repo spec-spine.toml must exist");
    let c = load_config(&src).expect("repo spec-spine.toml must load");
    assert_eq!(c.manifest.metadata_namespace, "spec-spine");
    assert!(c.domains.is_disabled());
    assert!(c.kind.is_disabled());
}

// ===== spec 039: layout.state_dir =====

#[test]
fn state_dir_is_unset_by_default_and_inert() {
    let c = Config::default();
    assert_eq!(c.layout.state_dir, "", "no state root is declared");
    // Every behavior keyed on it is off, so an existing repo is unchanged.
    for path in ["src/lib.rs", "state/journal.db", ".spec-spine/x", ""] {
        assert!(!c.layout.is_state_path(path), "{path}");
    }
}

#[test]
fn state_dir_matching_is_separator_aware() {
    let c = load_config("[layout]\nstate_dir = \"state\"\n").unwrap();
    for inside in ["state", "state/", "state/journal.db", "state/a/b.json"] {
        assert!(c.layout.is_state_path(inside), "{inside} is under the root");
    }
    // A sibling that merely shares a prefix is not inside it: a raw string
    // prefix test here would silently ungovern a real source directory.
    for outside in [
        "stateful",
        "stateful/x.rs",
        "src/state.rs",
        "src/state/x.rs",
    ] {
        assert!(!c.layout.is_state_path(outside), "{outside} is not");
    }
}

#[test]
fn a_trailing_slash_names_the_same_root() {
    let plain = load_config("[layout]\nstate_dir = \"state\"\n").unwrap();
    let slashed = load_config("[layout]\nstate_dir = \"state/\"\n").unwrap();
    for path in ["state", "state/x", "stateful/x"] {
        assert_eq!(
            plain.layout.is_state_path(path),
            slashed.layout.is_state_path(path),
            "{path}"
        );
    }
}

#[test]
fn state_dir_may_not_overlap_a_governed_root_in_either_direction() {
    // Equal, ancestor, and descendant are all refused; a sibling sharing a
    // prefix is accepted, per the separator-aware rule.
    let refused = [
        "specs",       // equals specs_dir
        ".derived",    // equals derived_dir
        ".",           // contains both: would ungovern the whole repository
        "./",          // the same value, spelled with the trailing slash
        "specs/state", // sits inside specs_dir
        ".derived/state",
    ];
    for value in refused {
        let err = load_config(&format!("[layout]\nstate_dir = \"{value}\"\n")).unwrap_err();
        assert!(
            matches!(err, Error::Config(_)),
            "state_dir '{value}' must be refused, got {err:?}"
        );
        assert_eq!(err.exit_code(), 3, "a bad config is exit 3");
    }
    // A value escaping the repo matches no path the gates ever test, so it
    // would declare a root that silences nothing while the config says one is
    // declared. Refused rather than left inert.
    for value in ["..", "../logs", "../../outside", "../"] {
        let err = load_config(&format!("[layout]\nstate_dir = \"{value}\"\n")).unwrap_err();
        assert!(
            matches!(err, Error::Config(_)),
            "state_dir '{value}' must be refused, got {err:?}"
        );
    }

    for value in ["specs-state", "state", "tool/state", ".derived-state"] {
        assert!(
            load_config(&format!("[layout]\nstate_dir = \"{value}\"\n")).is_ok(),
            "state_dir '{value}' is outside both governed roots"
        );
    }
}

/// The overlap check reads the **resolved** layout, never the defaults.
///
/// This is the inverse of the default-layout result and fails against any
/// implementation comparing with a hardcoded `specs`: the same two values swap
/// verdicts when `specs_dir` moves. Getting this wrong would clear a config that
/// makes every `spec.md` ungoverned, with every gate still exiting 0.
#[test]
fn the_overlap_check_reads_the_configured_roots() {
    let refused = load_config("[layout]\nspecs_dir = \"corpus\"\nstate_dir = \"corpus/state\"\n");
    assert!(refused.is_err(), "corpus/state sits inside the specs root");

    let accepted = load_config("[layout]\nspecs_dir = \"corpus\"\nstate_dir = \"specs/state\"\n");
    assert!(
        accepted.is_ok(),
        "specs/ is not a governed root in this layout: {accepted:?}"
    );
}
