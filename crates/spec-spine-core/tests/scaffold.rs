//! Scaffold tests (spec 006): the generated corpus is well-formed; a scaffolded
//! repo compiles and lints clean, proving the adoption loop works with zero
//! library edits.

use std::fs;

use spec_spine_core::{compile, lint, plan, scaffold_init};
use spec_spine_types::{Config, load_config};

/// Write a [`Scaffold`] to a temp dir as the CLI would.
fn materialize(cfg: &Config) -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    let scaffold = scaffold_init(cfg).unwrap();
    for f in &scaffold.files {
        let abs = tmp.path().join(&f.rel_path);
        fs::create_dir_all(abs.parent().unwrap()).unwrap();
        fs::write(&abs, &f.contents).unwrap();
    }
    tmp
}

#[test]
fn scaffolded_corpus_compiles_and_lints_clean() {
    let cfg = Config::default();
    let repo = materialize(&cfg);

    let outcome = compile(&cfg, repo.path()).unwrap();
    assert!(
        outcome.registry.validation.passed,
        "scaffolded corpus must compile clean: {:?}",
        outcome.registry.validation.violations
    );
    assert!(
        outcome
            .registry
            .specs
            .iter()
            .any(|s| s.id == "000-bootstrap"),
        "the bootstrap spec is present"
    );

    // The bootstrap spec is retroactive, so it raises no L-001 (no-edge) warning.
    let report = lint(&cfg, repo.path()).unwrap();
    assert!(
        !report.violations.iter().any(|v| v.code == "L-001"),
        "retroactive bootstrap should not trip L-001: {:?}",
        report.violations
    );
}

/// Spec 045 3.3: a freshly scaffolded corpus has nothing to schedule. The
/// bootstrap spec used to carry no `implementation` key, which `plan` read as
/// `pending`, so every `init` adopter's ready set was the bootstrap spec,
/// forever. It now declares `n-a`, and the plan of a scaffold is empty.
#[test]
fn scaffolded_corpus_has_nothing_ready_to_schedule() {
    let cfg = Config::default();
    let repo = materialize(&cfg);
    let outcome = compile(&cfg, repo.path()).unwrap();
    let plan = plan(&outcome.registry).unwrap();
    assert!(plan.ready.is_empty(), "{plan:?}");
    assert!(plan.blocked.is_empty(), "{plan:?}");

    let bootstrap = fs::read_to_string(repo.path().join("specs/000-bootstrap/spec.md")).unwrap();
    assert!(bootstrap.contains("implementation: n-a"), "{bootstrap}");
    let template = fs::read_to_string(
        repo.path()
            .join("standards/spec/templates/spec-template.md"),
    )
    .unwrap();
    assert!(
        template.contains("\nimplementation: pending"),
        "the template states the key rather than commenting it out: {template}"
    );
}

/// Spec 047: the three scaffolded rules carry the clarifications every adopter
/// that rewrote them added by hand, and the kit ships the same text.
#[test]
fn scaffolded_rules_carry_the_047_clarifications() {
    let cfg = Config::default();
    let repo = materialize(&cfg);
    let read = |rel: &str| fs::read_to_string(repo.path().join(rel)).unwrap();

    let reads = read(".claude/rules/governed-artifact-reads.md");
    assert!(reads.contains("is a typed read and is allowed"), "{reads}");

    let refusal = read(".claude/rules/adversarial-prompt-refusal.md");
    assert!(
        refusal.contains("Two edits are always legitimate"),
        "{refusal}"
    );
    assert!(refusal.contains("`establishes` list"), "{refusal}");
    assert!(refusal.contains("human instrument"), "{refusal}");
    assert!(refusal.contains("`extends` edge"), "{refusal}");

    let orch = read(".claude/rules/orchestrator-rules.md");
    assert!(orch.contains("commit the regenerated shards"), "{orch}");
    assert!(orch.contains("One session, one spec"), "{orch}");

    // The kit's copies are byte-identical to what the scaffold writes, so an
    // adopter who ran `init` and one who copied `kit/` read the same rule.
    for name in [
        "governed-artifact-reads",
        "adversarial-prompt-refusal",
        "orchestrator-rules",
    ] {
        let kit = fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../kit/.claude/rules")
                .join(format!("{name}.md")),
        )
        .unwrap();
        assert_eq!(
            kit,
            read(&format!(".claude/rules/{name}.md")),
            "kit/{name} drifted"
        );
    }
}

/// Spec 043 1.1: the scaffolded constitution shipped an amendment clause that
/// named an edge an adopter could not write. It now states the mechanism, and
/// this asserts the defect cannot return silently.
#[test]
fn scaffolded_constitution_states_an_executable_amendment_mechanism() {
    let scaffold = scaffold_init(&Config::default()).unwrap();
    let constitution = &scaffold
        .files
        .iter()
        .find(|f| f.rel_path == "standards/spec/constitution.md")
        .expect("the scaffold writes a constitution")
        .contents;

    assert!(constitution.contains("## Amendment"), "{constitution}");
    // The instrument is a section unit of this file, explicitly not `amends`.
    assert!(constitution.contains("kind: section"), "{constitution}");
    assert!(
        constitution.contains("`amends` is **not** the instrument"),
        "{constitution}"
    );
    // The seam that tells an adopter where their own principles belong.
    assert!(constitution.contains("## VI onward"), "{constitution}");
}

#[test]
fn non_default_namespace_scaffolds_coherently() {
    let mut cfg = Config::default();
    cfg.manifest.metadata_namespace = "acme".to_string();
    cfg.layout.specs_dir = "contracts".to_string();
    let repo = materialize(&cfg);

    // The bootstrap spec landed under the configured specs dir and compiles.
    assert!(
        repo.path()
            .join("contracts/000-bootstrap/spec.md")
            .is_file()
    );
    let outcome = compile(&cfg, repo.path()).unwrap();
    assert!(outcome.registry.validation.passed);
}

// ── spec 061: the scaffold ships what every adopter wrote by hand ─────────

fn scaffolded(cfg: &Config, rel: &str) -> String {
    scaffold_init(cfg)
        .unwrap()
        .files
        .into_iter()
        .find(|f| f.rel_path == rel)
        .unwrap_or_else(|| panic!("scaffold has no {rel}"))
        .contents
}

/// §3.1: the one non-deterministic artifact is ignored, so an adopter does not
/// independently learn that `build-meta.json` carries a wall clock by watching
/// their tree go dirty.
#[test]
fn the_scaffold_ignores_the_wall_clock_artifact() {
    let ignore = scaffolded(&Config::default(), ".gitignore");
    assert!(ignore.contains(".derived/**/build-meta.json"), "{ignore}");
}

/// §3.1: both ignored paths come from `Config`, so a non-default layout
/// scaffolds coherently rather than emitting the defaults as literals.
#[test]
fn the_gitignore_follows_the_configured_layout() {
    let cfg =
        load_config("[layout]\nderived_dir = \"build/derived\"\nstate_dir = \"tool-state\"\n")
            .unwrap();
    let ignore = scaffolded(&cfg, ".gitignore");
    assert!(
        ignore.contains("build/derived/**/build-meta.json"),
        "{ignore}"
    );
    assert!(
        ignore.contains("tool-state/"),
        "the declared state root: {ignore}"
    );
    assert!(
        !ignore.contains(".derived"),
        "no default leaked in as a literal: {ignore}"
    );
}

/// §3.1: an undeclared state root contributes no line. There is nothing to
/// ignore, and an empty path would ignore the repository.
#[test]
fn no_state_root_means_no_state_line() {
    let ignore = scaffolded(&Config::default(), ".gitignore");
    for line in ignore.lines().filter(|l| !l.trim_start().starts_with('#')) {
        assert!(
            line.trim().is_empty() || line.contains("build-meta.json"),
            "unexpected ignore entry: {line}"
        );
    }
}

/// §3.1: the shard trees are NOT ignored, and the file says why. Ignoring them
/// by default would silently opt every new adopter out of the freshness gate.
#[test]
fn the_shard_trees_are_not_ignored_and_the_choice_is_explained() {
    let ignore = scaffolded(&Config::default(), ".gitignore");
    for line in ignore.lines().filter(|l| !l.trim_start().starts_with('#')) {
        assert!(
            !line.trim().eq(".derived/"),
            "the scaffold must not take this decision: {ignore}"
        );
    }
    assert!(ignore.contains("spec-registry/"), "{ignore}");
    assert!(ignore.contains("freshness gate"), "{ignore}");
}

/// §3.1: it does not clobber an existing file, like every other scaffolded one.
#[test]
fn the_gitignore_is_not_marked_overwrite() {
    let f = scaffold_init(&Config::default())
        .unwrap()
        .files
        .into_iter()
        .find(|f| f.rel_path == ".gitignore")
        .unwrap();
    assert!(!f.overwrite);
}

/// §3.2: the emitted config parses back to the configuration it documents. A
/// documented config that has drifted from its own defaults is worse than none,
/// and this is the check that keeps the comments honest as `Config` grows.
#[test]
fn the_scaffolded_config_round_trips_to_its_own_defaults() {
    let toml = scaffolded(&Config::default(), "spec-spine.toml");
    let parsed = load_config(&toml).expect("the scaffolded config must parse");
    assert_eq!(
        parsed,
        Config::default(),
        "every emitted key is at its actual default"
    );
}

/// §3.2: and it round-trips a non-default configuration too, so the file an
/// adopter is handed describes the repository they asked for.
#[test]
fn the_scaffolded_config_round_trips_a_non_default_configuration() {
    let cfg = load_config(
        "[manifest]\nmetadata_namespace = \"acme\"\n\
         [layout]\nspecs_dir = \"corpus\"\nstate_dir = \"var\"\n\
         [coupling]\nrequire_ownership = true\n",
    )
    .unwrap();
    let toml = scaffolded(&cfg, "spec-spine.toml");
    assert_eq!(load_config(&toml).unwrap(), cfg);
}

/// §3.2: the knobs adopters reached for are named, each with the code it drives
/// where one exists. These are the thirteen the audit counted, not the five the
/// scaffold used to emit.
#[test]
fn the_scaffolded_config_names_the_knobs_adopters_needed() {
    let toml = scaffolded(&Config::default(), "spec-spine.toml");
    for knob in [
        "require_ownership",
        "auto_waive_dependency_only",
        "resolver_exclusions",
        "extra_hashed_inputs",
        "state_dir",
        "cargo_workspace",
        "standalone_rust_workspaces",
        "standalone_npm_packages",
        "extra_known_keys",
        "schemas_dir",
        "require_ordinal_monotonic_depends_on",
        "unwitnessed_allowed",
        "uri_schemes",
    ] {
        assert!(toml.contains(knob), "no mention of {knob}");
    }
    // Each of the refusal knobs names the code it drives.
    assert!(toml.contains("C-002"), "{toml}");
    assert!(toml.contains("L-006"), "{toml}");
    assert!(toml.contains("L-007"), "{toml}");
    assert!(toml.contains("L-008"), "{toml}");
    // And the glob trap spec 057 found is called out where it bites.
    assert!(toml.contains("`dir/**` matches DIRECTORIES"), "{toml}");
}

/// §3.3 + §3.4: the emitted template is the checked-in document, and a test
/// pins them together so they cannot drift, which is how the stub survived the
/// spec that diagnosed it.
#[test]
fn the_constitution_template_is_the_real_one_and_cannot_drift() {
    let emitted = scaffolded(
        &Config::default(),
        "standards/spec/templates/constitution-template.md",
    );
    let checked_in =
        std::fs::read_to_string("../../standards/spec/templates/constitution-template.md")
            .expect("the checked-in template");
    assert_eq!(emitted, checked_in, "the constant and the file must agree");

    // The properties spec 043 asked for, mirrored from its assertion on the
    // constitution itself.
    assert!(emitted.contains("Tier 2"), "{emitted}");
    assert!(emitted.contains("Normative hierarchy"), "{emitted}");
    assert!(emitted.contains("Amendment"), "{emitted}");
    // It stays a template: placeholders, not this corpus's own principles.
    assert!(emitted.contains("<Principle name>"), "{emitted}");
}
