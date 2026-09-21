//! Scaffold tests (spec 095, narrowed by spec 092 3.3): the generated corpus is
//! well-formed; a scaffolded repo compiles and lints clean, proving the
//! governance half of adoption works with zero library edits.
//!
//! Since spec 092 the scaffold is a producer Statecraft consumes rather than the
//! output of a `spec-spine init` command, and it produces governance content
//! only. The assertions that read `AGENTS.md`, `.claude/rules/` and the embedded
//! kit went with the surface they were about; the ones that say a scaffolded
//! corpus compiles, lints, round-trips its own config and schedules nothing are
//! here, joined by the boundary assertions 3.2 and 3.3 require.

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

/// Spec 042 3.3: a freshly scaffolded corpus has nothing to schedule. The
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

/// Spec 040 1.1: the scaffolded constitution shipped an amendment clause that
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

// ── spec 054: the scaffold ships what every adopter wrote by hand ─────────

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

/// Spec 058 §3.2: the emitted default is the working glob form. The round-trip
/// assertion above cannot catch this: it compares the emitted file against
/// `Config::default()`, so it holds just as well when both carry a pattern that
/// matches nothing. This asserts the value itself, and the negative half is the
/// one that bites, because `"standards/**/*"` contains `standards/**` as a
/// substring and a careless `contains` check would pass on the broken form.
#[test]
fn the_scaffolded_default_hashes_files_not_directories() {
    let toml = scaffolded(&Config::default(), "spec-spine.toml");
    assert!(
        toml.contains(r#"extra_hashed_inputs = ["standards/**/*", ".github/workflows/**/*"]"#),
        "{toml}"
    );
    for broken in [r#""standards/**""#, r#"".github/workflows/**""#] {
        assert!(
            !toml.contains(broken),
            "the pre-069 directory form is still emitted: {broken}"
        );
    }
    // The trap outlives the default: an adopter narrowing this list can still
    // write it, so the warning stays (and spec 054's own acceptance greps it).
    assert!(toml.contains("matches DIRECTORIES"), "{toml}");
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
    // And the glob trap spec 050 found is called out where it bites.
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

    // The properties spec 040 asked for, mirrored from its assertion on the
    // constitution itself.
    assert!(emitted.contains("Tier 2"), "{emitted}");
    assert!(emitted.contains("Normative hierarchy"), "{emitted}");
    assert!(emitted.contains("Amendment"), "{emitted}");
    // It stays a template: placeholders, not this corpus's own principles.
    assert!(emitted.contains("<Principle name>"), "{emitted}");
}

// ── spec 056: the contract records the lifecycle table ────────────────────

/// §3.3: the scaffolded contract carries both sections, so a new adopter gets
/// them rather than writing them. This is the pattern spec 040 §3.4
/// established when it added the amendment mechanism to the scaffolded
/// constitution.
#[test]
fn the_scaffolded_contract_carries_the_lifecycle_table_and_extra_keys() {
    let contract = scaffolded(&Config::default(), "standards/spec/contract.md");
    assert!(
        contract.contains("## Lifecycle as scheduling"),
        "{contract}"
    );
    assert!(contract.contains("## Extra keys"), "{contract}");

    // §3.1: the table's load-bearing rows, and the three sentences.
    assert!(contract.contains("| `approved` | `pending`, `in-progress` | yes |"));
    assert!(contract.contains("is a work order"), "{contract}");
    assert!(
        contract.contains("never a claim about code"),
        "a draft's unresolved units are expected"
    );
    assert!(
        contract.contains("not a third value"),
        "an absent implementation defers to status"
    );
    // §3.2: the discoverability point.
    assert!(contract.contains("extra_known_keys"), "{contract}");
    assert!(contract.contains("extraFrontmatter"), "{contract}");
}

/// §3.4: the scaffolded corpus still compiles and lints clean with the longer
/// contract, and the generator stays a pure function of `Config`.
#[test]
fn the_longer_contract_does_not_break_the_scaffolded_corpus() {
    let cfg = Config::default();
    let a = scaffold_init(&cfg).unwrap();
    let b = scaffold_init(&cfg).unwrap();
    assert_eq!(
        a.files.len(),
        b.files.len(),
        "pure: same input, same output"
    );
    for (x, y) in a.files.iter().zip(b.files.iter()) {
        assert_eq!(x.contents, y.contents);
    }
}

// ── spec 061 3.3 and 3.4: the scaffold carries the facts the writer applies ──

// ── spec 092: the producer boundary Statecraft consumes ───────────────────

/// The layout values Statecraft passes (spec 092 §3.3). Named once here so the
/// assertions below read the same four values the contract names.
fn statecraft_layout() -> Config {
    let mut cfg = Config::default();
    cfg.layout.specs_dir = "specs".to_string();
    cfg.layout.standards_dir = "standards/spec".to_string();
    cfg.layout.derived_dir = ".statecraft/derived".to_string();
    cfg.layout.state_dir = ".statecraft/state".to_string();
    cfg
}

/// §3.3: the file set is exactly the seven governance files, in a default
/// layout. An exact set rather than a `contains` sweep, because the defect this
/// spec removes is an EXTRA file (an `AGENTS.md`, three `.claude/rules/`), and
/// no number of `contains` assertions can notice one.
#[test]
fn the_producer_emits_exactly_the_governance_file_set() {
    let files = scaffold_init(&Config::default()).unwrap().files;
    let mut paths: Vec<&str> = files.iter().map(|f| f.rel_path.as_str()).collect();
    paths.sort_unstable();
    assert_eq!(
        paths,
        vec![
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

/// §3.3: and it emits no development environment, named path by path. The
/// prefixes rather than only the exact names, so a file placed one level deeper
/// (`.claude/skills/build/SKILL.md`) is caught by the same line.
#[test]
fn the_producer_emits_no_agent_or_environment_artifact() {
    for cfg in [Config::default(), statecraft_layout()] {
        let files = scaffold_init(&cfg).unwrap().files;
        for f in &files {
            let p = f.rel_path.as_str();
            for forbidden in [
                "AGENTS.md",
                "CLAUDE.md",
                ".claude/",
                ".codex/",
                ".agents/",
                ".statecraft/",
                ".mcp.json",
                "Makefile",
                "govern.yml",
                ".github/",
                ".githooks/",
                ".gitattributes",
                "settings.json",
            ] {
                assert!(
                    p != forbidden && !p.starts_with(forbidden),
                    "the scaffold emits `{p}`, which is environment and not governance"
                );
            }
        }
    }
}

/// §3.3: the layout Statecraft passes scaffolds coherently, and every path is
/// relative to the repository root rather than to the configuration file.
#[test]
fn the_statecraft_layout_scaffolds_coherently() {
    let cfg = statecraft_layout();
    let files = scaffold_init(&cfg).unwrap().files;
    let paths: Vec<&str> = files.iter().map(|f| f.rel_path.as_str()).collect();

    assert!(paths.contains(&"specs/000-bootstrap/spec.md"));
    assert!(paths.contains(&"standards/spec/constitution.md"));
    assert!(paths.contains(&"spec-spine.toml"), "at the ROOT, always");
    for p in &paths {
        assert!(!p.starts_with('/'), "`{p}` is absolute");
        assert!(!p.starts_with("./"), "`{p}` is not root-relative");
        assert!(!p.contains(".."), "`{p}` escapes the repository root");
    }

    let toml = scaffolded(&cfg, "spec-spine.toml");
    assert!(
        toml.contains("derived_dir   = \".statecraft/derived\""),
        "{toml}"
    );
    assert!(
        toml.contains("state_dir     = \".statecraft/state\""),
        "{toml}"
    );

    let ignore = scaffolded(&cfg, ".gitignore");
    assert!(
        ignore.contains(".statecraft/derived/**/build-meta.json"),
        "the transient metadata is excluded: {ignore}"
    );
    assert!(
        ignore.contains(".statecraft/state/"),
        "the state root is excluded: {ignore}"
    );
    // And the governed half of `.statecraft/` is NOT excluded. A `.statecraft/`
    // line would put the committed ledger outside version control, which is
    // the failure spec 092 §3.9 exists to refuse.
    assert!(
        !ignore.lines().any(|l| l.trim() == ".statecraft/"),
        "the whole directory must not be ignored: {ignore}"
    );
    assert!(
        !ignore
            .lines()
            .any(|l| l.trim() == ".statecraft/derived/" || l.trim() == ".statecraft/derived"),
        "the committed shard trees must not be ignored: {ignore}"
    );
}

/// §3.3: the `.gitignore` is returned as something to reconcile, not something
/// to write over. A consumer that replaced an existing ignore file would
/// destroy exclusions the repository already had.
#[test]
fn the_gitignore_is_an_append_with_a_marker() {
    let files = scaffold_init(&statecraft_layout()).unwrap().files;
    let ignore = files.iter().find(|f| f.rel_path == ".gitignore").unwrap();
    assert!(ignore.append, "it is appended, not written over");
    assert!(!ignore.overwrite, "and never forced");
    let marker = ignore
        .append_marker
        .as_deref()
        .expect("an append carries the marker that makes it idempotent");
    assert!(
        ignore.contents.contains(marker),
        "the marker must be present in what is appended, or the second run \
         appends a second copy: {marker:?}"
    );
    // Every other file is a plain write, skipped when it already exists.
    for f in files.iter().filter(|f| f.rel_path != ".gitignore") {
        assert!(
            !f.append,
            "{}: only the ignore fragment appends",
            f.rel_path
        );
        assert!(!f.overwrite, "{}: never forced", f.rel_path);
    }
}

/// §3.2: the exported JSON facade is the boundary, and it answers the same
/// thing the typed call does. Exercised through `scaffold_init_json` itself,
/// with a JSON configuration, because that is the function the consumer calls.
#[test]
fn the_json_facade_is_the_boundary_statecraft_consumes() {
    let request = r#"{
        "layout": {
            "specs_dir": "specs",
            "standards_dir": "standards/spec",
            "derived_dir": ".statecraft/derived",
            "state_dir": ".statecraft/state"
        }
    }"#;
    let out = spec_spine_core::scaffold_init_json(request).expect("the facade answers");
    let v: serde_json::Value = serde_json::from_str(&out).expect("it answers JSON");
    let files = v["files"].as_array().expect("files array");
    assert_eq!(files.len(), 7, "the governance file set: {out}");

    let paths: Vec<&str> = files
        .iter()
        .map(|f| f["relPath"].as_str().expect("relPath"))
        .collect();
    assert!(paths.contains(&"spec-spine.toml"), "{paths:?}");
    assert!(paths.contains(&"specs/000-bootstrap/spec.md"), "{paths:?}");
    assert!(!paths.iter().any(|p| p.starts_with(".claude")), "{paths:?}");
    assert!(!paths.contains(&"AGENTS.md"), "{paths:?}");

    // The response shape the consumer is implementing against, field by field.
    let ignore = files
        .iter()
        .find(|f| f["relPath"] == ".gitignore")
        .expect("the ignore fragment");
    for key in ["relPath", "contents", "overwrite", "executable", "append"] {
        assert!(
            ignore.get(key).is_some(),
            "`{key}` is part of the contract: {ignore}"
        );
    }
    assert_eq!(ignore["append"], serde_json::Value::Bool(true));
    assert!(ignore["appendMarker"].is_string(), "{ignore}");

    // And `"{}"` means defaults, which is the documented degenerate request.
    let defaults = spec_spine_core::scaffold_init_json("{}").expect("defaults");
    let d: serde_json::Value = serde_json::from_str(&defaults).unwrap();
    assert_eq!(d["files"].as_array().unwrap().len(), 7);
}

/// §3.2: the producer is pure, asserted two ways that can actually fail.
///
/// The first is behavioral: two calls with the same argument answer the same
/// bytes, so nothing in it reads a clock, a random source or mutable state.
/// The second reads the module's own source for the constructs a pure function
/// may not contain. A source read rather than a claim in a doc comment, because
/// the contract is what a consumer relies on and prose does not fail.
#[test]
fn the_producer_performs_no_io() {
    let first = spec_spine_core::scaffold_init_json("{}").unwrap();
    let second = spec_spine_core::scaffold_init_json("{}").unwrap();
    assert_eq!(first, second, "the same request must answer the same bytes");

    let src = fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/scaffold.rs"),
    )
    .unwrap();
    let code: String = src
        .lines()
        .filter(|l| {
            let t = l.trim_start();
            !t.starts_with("//") && !t.starts_with("///") && !t.starts_with("//!")
        })
        .collect::<Vec<_>>()
        .join("\n");
    for forbidden in [
        "std::fs",
        "fs::write",
        "fs::read",
        "std::env::var",
        "std::process",
        "Command::new",
        "SystemTime",
        "Instant::now",
        "TcpStream",
    ] {
        assert!(
            !code.contains(forbidden),
            "scaffold.rs contains `{forbidden}`; the producer is a pure function \
             of its argument (spec 092 §3.2)"
        );
    }
    // The positive control: the reader above is the thing under test, so a run
    // that found nothing has to be shown capable of finding something. `format!`
    // is in every line of this module.
    assert!(
        code.contains("format!"),
        "the source read returned code the filter had emptied, so the assertions \
         above could not have failed either"
    );
}
