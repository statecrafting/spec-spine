//! Spec 116: `L-001` exempts a deferred contract that claims nothing.
//!
//! The exemption is a ratchet, not a hole, so the negative cases carry the
//! weight: scheduling a deferred spec must bring the warning back, `n-a` and an
//! absent `implementation` must not be swept in, and a deferred spec that names
//! a unit must still be diagnosed about that unit. Every case runs on a
//! scratch corpus and changes one field against a fixed base (116 D-2, D-5).

use std::fs;
use std::path::Path;

use spec_spine_types::{Config, Violation};

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, content).unwrap();
}

/// A spec with no ownership edge at all, with an authored `implementation`
/// (or none, for the empty string).
fn territoryless(id: &str, implementation: &str) -> String {
    let impl_line = if implementation.is_empty() {
        String::new()
    } else {
        format!("implementation: {implementation}\n")
    };
    format!(
        "---\nid: \"{id}\"\ntitle: \"T\"\nstatus: draft\ncreated: \"2026-09-21\"\n\
         {impl_line}summary: \"a contract nobody is building\"\n---\n# {id}\n## body\n"
    )
}

fn lint(root: &Path) -> Vec<Violation> {
    spec_spine_core::lint(&Config::default(), root)
        .unwrap()
        .violations
}

fn l001(root: &Path) -> Vec<Violation> {
    lint(root)
        .into_iter()
        .filter(|v| v.code == "L-001")
        .collect()
}

#[test]
fn a_deferred_spec_with_no_territory_is_exempt_from_l001() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write(
        root,
        "specs/001-deferred/spec.md",
        &territoryless("001-deferred", "deferred"),
    );
    assert!(
        l001(root).is_empty(),
        "a deliberately unscheduled contract claims nothing on purpose: {:?}",
        l001(root)
    );
}

#[test]
fn scheduling_a_deferred_spec_re_arms_l001() {
    // Same corpus, same file, one field changed, compared against a fixed base
    // rather than against a remembered count. An implementation that dropped
    // L-001 unconditionally passes the exemption case and fails every one of
    // these.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let path = "specs/001-contract/spec.md";

    write(root, path, &territoryless("001-contract", "deferred"));
    assert!(l001(root).is_empty(), "deferred is exempt");

    for scheduled in ["pending", "in-progress", "complete", ""] {
        write(root, path, &territoryless("001-contract", scheduled));
        let v = l001(root);
        assert_eq!(
            v.len(),
            1,
            "implementation {scheduled:?} must re-arm L-001, got {v:?}"
        );
        assert_eq!(
            v[0].message, "spec '001-contract' declares no ownership edge (claims no territory)",
            "the message of a spec that still raises L-001 is unchanged (116 §3.4)"
        );
    }

    // And back again: nothing remembers the earlier scheduling either.
    write(root, path, &territoryless("001-contract", "deferred"));
    assert!(l001(root).is_empty(), "deferred again is exempt again");
}

#[test]
fn an_n_a_spec_with_no_territory_still_raises_l001() {
    // §3.3: `n-a` is a ratified record that owns nothing on purpose, a
    // different statement from "not scheduled", and is not swept in.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write(
        root,
        "specs/001-na/spec.md",
        &territoryless("001-na", "n-a"),
    );
    assert_eq!(
        l001(root).len(),
        1,
        "n-a must not inherit the deferred exemption"
    );
}

#[test]
fn the_exemption_moves_no_other_diagnostic() {
    // §3.4: the only difference between a deferred and a pending copy of the
    // same territoryless spec is the one L-001 line.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let path = "specs/001-contract/spec.md";

    write(root, path, &territoryless("001-contract", "pending"));
    let mut pending: Vec<(String, String)> = lint(root)
        .into_iter()
        .map(|v| (v.code, v.message))
        .collect();
    write(root, path, &territoryless("001-contract", "deferred"));
    let deferred: Vec<(String, String)> = lint(root)
        .into_iter()
        .map(|v| (v.code, v.message))
        .collect();

    let removed = pending
        .iter()
        .position(|(c, _)| c == "L-001")
        .expect("the pending copy raises L-001");
    pending.remove(removed);
    assert_eq!(pending, deferred);
}

#[test]
fn a_deferred_spec_that_names_a_unit_is_still_diagnosed() {
    // §3.2: the exemption covers the DECLARATION, never a claim. The index's
    // unresolved-unit diagnostic is what holds a claim on a path that does not
    // exist (116 D-3), so that is what is asserted, by code and by path
    // (116 D-6), and compared with the same claim by a pending spec.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let path = "specs/001-deferred/spec.md";
    let claiming = |implementation: &str| {
        format!(
            "---\nid: \"001-deferred\"\ntitle: \"T\"\nstatus: draft\ncreated: \"2026-09-21\"\n\
             implementation: {implementation}\nsummary: \"s\"\nestablishes:\n  - \"src/nowhere.rs\"\n\
             ---\n# 001-deferred\n## body\n"
        )
    };
    let unresolved = |root: &Path| -> Vec<(String, Option<String>)> {
        let outcome = spec_spine_core::index(&Config::default(), root).unwrap();
        let d = outcome.index.diagnostics;
        d.warnings
            .into_iter()
            .chain(d.errors)
            .filter(|d| d.message.contains("src/nowhere.rs"))
            .map(|d| (d.code, d.path))
            .collect()
    };

    write(root, path, &claiming("deferred"));
    assert!(
        l001(root).is_empty(),
        "it declares an ownership edge, so L-001 is not the question"
    );
    let deferred = unresolved(root);
    assert!(
        deferred.iter().any(|(code, _)| code == "W-001"),
        "a deferred spec's claim on a path that does not exist is still an \
         unresolved unit: {deferred:?}"
    );

    write(root, path, &claiming("pending"));
    assert_eq!(
        unresolved(root),
        deferred,
        "the deferral changes nothing about how the claim is diagnosed"
    );
}
