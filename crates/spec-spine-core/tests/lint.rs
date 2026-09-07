//! Spec 053: `L-007`, the opt-in ordinal-monotonicity check on `depends_on`.
//!
//! The corpus had no dedicated lint test file before this spec: `L-006`'s
//! acceptance lives in `tests/coverage.rs` and the scaffold's in
//! `tests/scaffold.rs`. This is that file, and spec 053 claims it.

use std::fs;
use std::path::Path;

use spec_spine_types::{Config, Severity, load_config};

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, content).unwrap();
}

/// A minimal well-formed spec. `establishes` keeps `L-001` quiet so the
/// assertions below read against `L-007` alone.
fn spec(id: &str, depends_on: &[&str]) -> String {
    let deps = if depends_on.is_empty() {
        String::new()
    } else {
        let items: String = depends_on
            .iter()
            .map(|d| format!("  - \"{d}\"\n"))
            .collect();
        format!("depends_on:\n{items}")
    };
    format!(
        "---\nid: \"{id}\"\ntitle: \"T\"\nstatus: approved\ncreated: \"2026-09-07\"\n\
         summary: \"s\"\nestablishes:\n  - \"src/{id}.rs\"\n{deps}---\n# {id}\n## body\n"
    )
}

/// The knob on, everything else default.
fn opted_in() -> Config {
    load_config("[lint]\nrequire_ordinal_monotonic_depends_on = true\n").unwrap()
}

fn l007(cfg: &Config, root: &Path) -> Vec<spec_spine_types::Violation> {
    spec_spine_core::lint(cfg, root)
        .unwrap()
        .violations
        .into_iter()
        .filter(|v| v.code == "L-007")
        .collect()
}

/// A corpus with one forward edge (`012 -> 044`) and one backward one
/// (`044 -> 012` would be a cycle, so `044 -> 001`).
fn forward_corpus(root: &Path) {
    write(root, "specs/001-base/spec.md", &spec("001-base", &[]));
    write(
        root,
        "specs/012-early/spec.md",
        &spec("012-early", &["044-late"]),
    );
    write(
        root,
        "specs/044-late/spec.md",
        &spec("044-late", &["001-base"]),
    );
}

/// §3.1: the knob defaults off, and off means *not emitted*, not
/// emitted-and-filtered. A corpus that has not opted in sees byte-identical
/// lint output before and after this spec.
#[test]
fn the_knob_defaults_off_and_off_emits_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    forward_corpus(tmp.path());
    assert!(
        Config::default()
            .lint
            .require_ordinal_monotonic_depends_on
            .eq(&false),
        "the knob defaults off"
    );
    assert!(
        l007(&Config::default(), tmp.path()).is_empty(),
        "a corpus that did not opt in is not told about its edges"
    );
}

/// §3.2: with the knob on, a forward edge is one error-tier `L-007` naming
/// both ids, and the backward edge in the same corpus is silent.
#[test]
fn a_forward_dependency_is_an_error_and_a_backward_one_is_silent() {
    let tmp = tempfile::tempdir().unwrap();
    forward_corpus(tmp.path());
    let found = l007(&opted_in(), tmp.path());

    assert_eq!(found.len(), 1, "only the forward edge fires: {found:?}");
    let v = &found[0];
    assert_eq!(v.severity, Severity::Error, "the knob alone decides");
    assert!(v.message.contains("012-early"), "{}", v.message);
    assert!(v.message.contains("044-late"), "{}", v.message);
    assert!(
        v.message.contains("points backward in filing order"),
        "{}",
        v.message
    );
    // §3.2: the path is the *declaring* spec, so an editor jumping to the
    // diagnostic lands on the file that has to change.
    assert_eq!(v.path.as_deref(), Some("specs/012-early/spec.md"));
}

/// §3.3: equality is a violation, not a pass. Two specs cannot share an
/// ordinal (`V-004` refuses it), so an entry whose ordinal equals the
/// declaring spec's is a spec depending on itself.
#[test]
fn a_self_dependency_is_caught_by_the_equality_arm() {
    let tmp = tempfile::tempdir().unwrap();
    write(
        tmp.path(),
        "specs/007-self/spec.md",
        &spec("007-self", &["007-self"]),
    );
    let found = l007(&opted_in(), tmp.path());
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0].message.contains("007 >= 007"),
        "{}",
        found[0].message
    );
}

/// §3.3: an id with no leading digit run has no ordinal, and silence is the
/// honest answer when the order is undefined. Checked in both positions: the
/// declaring spec and the target.
#[test]
fn an_id_without_an_ordinal_is_silence_in_either_position() {
    let tmp = tempfile::tempdir().unwrap();
    let r = tmp.path();
    write(r, "specs/auth-login/spec.md", &spec("auth-login", &[]));
    // Declaring spec has no ordinal.
    write(
        r,
        "specs/auth-logout/spec.md",
        &spec("auth-logout", &["auth-login"]),
    );
    // Target has no ordinal, declaring spec does.
    write(
        r,
        "specs/003-numbered/spec.md",
        &spec("003-numbered", &["auth-login"]),
    );
    assert!(
        l007(&opted_in(), r).is_empty(),
        "a corpus that never claimed the convention is not held to it"
    );
}

/// §3.3: the comparison is numeric, not lexical, so a corpus that outgrows
/// three digits orders `1001` above `999` instead of below it.
#[test]
fn the_comparison_is_numeric_not_lexical() {
    let tmp = tempfile::tempdir().unwrap();
    let r = tmp.path();
    write(r, "specs/999-nine/spec.md", &spec("999-nine", &[]));
    // 1001 -> 999 decreases numerically; lexically "1001" < "999" would have
    // called this forward and fired.
    write(
        r,
        "specs/1001-thousand/spec.md",
        &spec("1001-thousand", &["999-nine"]),
    );
    assert!(
        l007(&opted_in(), r).is_empty(),
        "1001 -> 999 points backward"
    );

    // And the genuine inversion still fires.
    let tmp2 = tempfile::tempdir().unwrap();
    let r2 = tmp2.path();
    write(
        r2,
        "specs/1001-thousand/spec.md",
        &spec("1001-thousand", &[]),
    );
    write(
        r2,
        "specs/999-nine/spec.md",
        &spec("999-nine", &["1001-thousand"]),
    );
    assert_eq!(l007(&opted_in(), r2).len(), 1);
}

/// §3.4: `L-007` is a lint and never a `V-` code. The corpus above compiles,
/// its shard is valid, and `compile` reports no violation of its own.
#[test]
fn a_forward_dependency_does_not_fail_compile() {
    let tmp = tempfile::tempdir().unwrap();
    forward_corpus(tmp.path());
    let outcome = spec_spine_core::compile(&opted_in(), tmp.path()).unwrap();
    assert!(
        outcome.registry.validation.passed,
        "a convention the corpus may not hold is not a structural defect: {:?}",
        outcome.registry.validation.violations
    );
    assert!(
        !outcome
            .registry
            .validation
            .violations
            .iter()
            .any(|v| v.code == "L-007"),
        "the lint's code never appears in compile's report"
    );
}
