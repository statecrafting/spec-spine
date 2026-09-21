// Spec: specs/067-a-short-id-names-the-same-spec-at-every-verb/spec.md
//! Spec 067 §3.4 and §3.5: the one spec-id policy, the library entry points
//! that call it, and the JSON facade over them.
//!
//! The matrix in `crates/spec-spine-cli/tests/spec_id.rs` drives the six
//! arguments through the binary; this file holds the policy's own boundaries
//! and the library halves of §3.3, which the CLI cannot see from outside.

use std::fs;
use std::path::Path;

use spec_spine_core::{
    SpecIdMatch, attest_spec, compile, match_spec_id, query_json, relationships, resolve_spec_id,
    resolve_spec_ref, show, spec_dir_ids, verify_plan,
};
use spec_spine_types::{Config, Error};

fn write_spec(root: &Path, id: &str, extra: &str) {
    let spec_dir = root.join("specs").join(id);
    fs::create_dir_all(&spec_dir).unwrap();
    let body = format!(
        "---\nid: \"{id}\"\ntitle: \"Title {id}\"\nstatus: approved\ncreated: \"2026-09-11\"\nsummary: \"s\"\n{extra}---\n# {id}\n"
    );
    fs::write(spec_dir.join("spec.md"), body).unwrap();
}

// ===== §3.1 steps 1-4: the policy itself =====

/// Step 1. An exact id wins whatever else is in the set, and whatever order the
/// set iterates in: a corpus holding both `070` and `070-slug` resolves `070`
/// to itself rather than calling it ambiguous.
#[test]
fn an_exact_id_beats_a_segment_match_in_either_order() {
    for ids in [vec!["070", "070-slug"], vec!["070-slug", "070"]] {
        assert_eq!(
            match_spec_id("070", &ids),
            SpecIdMatch::Resolved("070".to_string()),
            "{ids:?}"
        );
    }
}

/// Step 2, and the two non-matches spec 015 §3.1 names by hand: a partial
/// ordinal is not an ordinal, and a wrong full slug does not snap to a
/// neighbour.
#[test]
fn the_whole_leading_segment_must_match() {
    let ids = ["070-a-malformed-id", "071-other"];
    assert_eq!(
        match_spec_id("070", ids),
        SpecIdMatch::Resolved("070-a-malformed-id".to_string())
    );
    assert_eq!(match_spec_id("70", ids), SpecIdMatch::NoMatch);
    assert_eq!(match_spec_id("070-typo", ids), SpecIdMatch::NoMatch);
    assert_eq!(match_spec_id("", ids), SpecIdMatch::NoMatch);
}

/// Step 3. Never guessed, and the candidates are sorted, so the refusal does
/// not depend on a `read_dir` order.
#[test]
fn a_shared_ordinal_is_ambiguous_with_sorted_candidates() {
    let ids = ["001-b", "001-a", "002-c"];
    assert_eq!(
        match_spec_id("001", ids),
        SpecIdMatch::Ambiguous(vec!["001-a".to_string(), "001-b".to_string()])
    );
}

/// §3.2, D-6: membership is a string comparison, so a path-shaped argument
/// matches nothing. This is the whole of what closes §1.2 in the policy; the
/// CLI matrix asserts the exit codes that follow from it.
#[test]
fn a_path_is_not_an_id() {
    let ids = ["070-a-malformed-id"];
    for arg in [
        "../specs/070-a-malformed-id",
        "./070-a-malformed-id",
        "070-a-malformed-id/",
        "specs/070-a-malformed-id",
    ] {
        assert_eq!(match_spec_id(arg, ids), SpecIdMatch::NoMatch, "{arg}");
    }
}

/// §3.1: the two refusals are fixed in one place, so a reader cannot tell which
/// verb produced them. The CLI matrix proves they are identical across the six
/// arguments; this pins what they say.
#[test]
fn the_strict_form_maps_both_failures_to_not_found() {
    let ids = ["001-a", "001-b", "002-c"];
    let err = resolve_spec_id("001", ids).unwrap_err();
    assert!(matches!(err, Error::NotFound(_)), "{err:?}");
    let msg = err.to_string();
    assert!(msg.contains("ambiguous"), "{msg}");
    assert!(msg.contains("001-a") && msg.contains("001-b"), "{msg}");

    let err = resolve_spec_id("999", ids).unwrap_err();
    assert!(matches!(err, Error::NotFound(_)), "{err:?}");
    assert_eq!(err.to_string(), "not found: spec '999'");
}

/// §3.4: the lenient adapters keep their contract, which is what leaves every
/// committed shard byte-identical. `V-008` and `V-010` still see the raw string
/// and still name a dangling reference.
#[test]
fn the_lenient_form_keeps_the_raw_string_on_both_failures() {
    let ids = ["001-a", "001-b", "070-slug"];
    assert_eq!(resolve_spec_ref("070", ids), "070-slug");
    assert_eq!(resolve_spec_ref("070-slug", ids), "070-slug");
    assert_eq!(resolve_spec_ref("001", ids), "001");
    assert_eq!(resolve_spec_ref("999", ids), "999");
}

/// The one set this module supplies: entries holding a `spec.md`, sorted, and
/// nothing else.
#[test]
fn spec_dir_ids_lists_only_directories_holding_a_spec_md() {
    let t = tempfile::tempdir().unwrap();
    write_spec(t.path(), "002-beta", "");
    write_spec(t.path(), "001-alpha", "");
    fs::create_dir_all(t.path().join("specs").join("003-empty")).unwrap();
    fs::write(t.path().join("specs").join("loose.md"), "x").unwrap();
    assert_eq!(
        spec_dir_ids(&t.path().join("specs")).unwrap(),
        vec!["001-alpha".to_string(), "002-beta".to_string()]
    );
}

#[test]
fn spec_dir_ids_on_a_missing_directory_is_an_io_error() {
    let t = tempfile::tempdir().unwrap();
    let err = spec_dir_ids(&t.path().join("nope")).unwrap_err();
    assert!(matches!(err, Error::Io(_)), "{err:?}");
}

// ===== §3.3: resolve once, then use only the resolved id =====

fn corpus() -> (tempfile::TempDir, spec_spine_types::Registry) {
    let t = tempfile::tempdir().unwrap();
    write_spec(t.path(), "016-short", "");
    write_spec(t.path(), "049-verify", "depends_on: [\"016-short\"]\n");
    write_spec(t.path(), "056-compile", "amends: [\"016-short\"]\n");
    let registry = compile(&Config::default(), t.path()).unwrap().registry;
    (t, registry)
}

#[test]
fn show_accepts_the_short_form_and_returns_the_same_record() {
    let (_t, registry) = corpus();
    assert_eq!(show(&registry, "016").unwrap().id, "016-short");
    assert_eq!(
        show(&registry, "016").unwrap(),
        show(&registry, "016-short").unwrap()
    );
}

/// The regression §1.3 describes: `relationships` computed its incoming edges
/// against the raw argument, so resolving inside `show` alone would print an
/// empty `depended_on_by` at exit 0. A wrong answer is worse than a refusal,
/// which is why this assertion names the edge rather than only comparing.
#[test]
fn relationships_under_the_short_form_keeps_its_incoming_edges() {
    let (_t, registry) = corpus();
    let short = relationships(&registry, "016").unwrap();
    let full = relationships(&registry, "016-short").unwrap();
    assert_eq!(short.id, "016-short");
    assert_eq!(short.depended_on_by, vec!["049-verify".to_string()]);
    assert_eq!(short.amended_by, vec!["056-compile".to_string()]);
    assert_eq!(short, full);
}

/// §3.3: the payload's `specId` is the resolved id, and the units are the
/// resolved spec's. A short form that selected the record but not the index
/// mapping would emit a confident attestation over zero units at exit 0.
#[test]
fn attest_spec_under_the_short_form_is_the_same_payload() {
    let t = tempfile::tempdir().unwrap();
    write_spec(
        t.path(),
        "070-attested",
        "establishes:\n  - { kind: file, path: \"src/a.rs\" }\n",
    );
    fs::create_dir_all(t.path().join("src")).unwrap();
    fs::write(t.path().join("src").join("a.rs"), "// a\n").unwrap();

    let cfg = Config::default();
    let short = attest_spec(&cfg, t.path(), "070").unwrap();
    let full = attest_spec(&cfg, t.path(), "070-attested").unwrap();
    assert_eq!(short.attestation.spec_id, "070-attested");
    assert!(
        short
            .attestation
            .units
            .iter()
            .any(|u| u.content_hash.is_some()),
        "a short-form attestation over zero units is the partial fix of 1.3"
    );
    assert_eq!(short.json, full.json);
    assert_eq!(short.attestation_hash, full.attestation_hash);
}

/// §3.3: resolution lives in the library entry point, so the facade a binding
/// wraps accepts the short form with no work of its own.
#[test]
fn the_facade_accepts_the_short_form_for_free() {
    let (_t, registry) = corpus();
    let registry_json = serde_json::to_string(&registry).unwrap();
    let req = |id: &str| {
        serde_json::json!({ "registry": registry_json, "op": "show", "id": id }).to_string()
    };
    let short = query_json(&req("016")).unwrap();
    let full = query_json(&req("016-short")).unwrap();
    assert_eq!(short, full);
    assert!(short.contains("016-short"), "{short}");
}

#[test]
fn verify_plan_accepts_the_short_form_and_refuses_a_path() {
    let t = tempfile::tempdir().unwrap();
    let spec_dir = t.path().join("specs").join("049-verify");
    fs::create_dir_all(&spec_dir).unwrap();
    fs::write(
        spec_dir.join("spec.md"),
        "---\nid: \"049-verify\"\ntitle: \"t\"\nstatus: approved\ncreated: \"2026-09-11\"\nsummary: \"s\"\n---\n\n## Verification\n\n```verify:cli\ntrue\n```\n",
    )
    .unwrap();

    let cfg = Config::default();
    assert_eq!(
        verify_plan(&cfg, t.path(), "049").unwrap().spec_id,
        "049-verify"
    );
    // §1.2: this resolved through `Path::join` before 084.
    let err = verify_plan(&cfg, t.path(), "../specs/049-verify").unwrap_err();
    assert!(matches!(err, Error::NotFound(_)), "{err:?}");
}

/// §3.6: no emitted byte moves. The lenient adapters return what the mirrors
/// returned, so a corpus whose frontmatter uses short references compiles to
/// the same registry it always did.
#[test]
fn a_short_reference_in_frontmatter_still_resolves_the_way_it_did() {
    let t = tempfile::tempdir().unwrap();
    write_spec(t.path(), "016-short", "");
    write_spec(t.path(), "049-verify", "depends_on: [\"016\"]\n");
    let registry = compile(&Config::default(), t.path()).unwrap().registry;
    let rec = show(&registry, "049-verify").unwrap();
    assert_eq!(rec.depends_on, vec!["016-short".to_string()]);
}

/// And an unresolvable one is still carried through raw, so `V-008` names it.
#[test]
fn a_dangling_reference_is_still_carried_through_raw() {
    let t = tempfile::tempdir().unwrap();
    write_spec(t.path(), "049-verify", "depends_on: [\"999\"]\n");
    let registry = compile(&Config::default(), t.path()).unwrap().registry;
    let rec = show(&registry, "049-verify").unwrap();
    assert_eq!(rec.depends_on, vec!["999".to_string()]);
}
