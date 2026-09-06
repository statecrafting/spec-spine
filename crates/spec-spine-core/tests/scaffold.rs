//! Scaffold tests (spec 006): the generated corpus is well-formed; a scaffolded
//! repo compiles and lints clean, proving the adoption loop works with zero
//! library edits.

use std::fs;

use spec_spine_core::{compile, lint, plan, scaffold_init};
use spec_spine_types::Config;

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
