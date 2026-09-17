// Spec: specs/000-spec-spine-bootstrap/spec.md
//! Dogfood: this repo's own bootstrap spec must parse through the types crate.
//!
//! The authored frontmatter of `specs/000` must conform to the grammar this
//! crate defines; a parse failure here means the corpus and the types drifted
//! apart (this guards the grammar independently of the full compile pipeline).

use spec_spine_types::{KNOWN_KEYS, Status, parse_frontmatter};

fn read_repo_file(rel: &str) -> String {
    let path = format!("{}/../../{}", env!("CARGO_MANIFEST_DIR"), rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"))
}

#[test]
fn bootstrap_spec_000_parses() {
    let src = read_repo_file("specs/000-spec-spine-bootstrap/spec.md");
    let fm = parse_frontmatter(&src).expect("specs/000 frontmatter must parse");

    assert_eq!(fm.id, "000-spec-spine-bootstrap");
    assert_eq!(fm.status, Status::Approved);

    // It is the retroactive bootstrap root.
    let origin = fm.origin.expect("000 must declare origin");
    assert!(origin.retroactive, "000 must be origin.retroactive");

    // Its constitutional freeze surface is present and includes the core anchors.
    for anchor in [
        "markdown-truth-boundary",
        "json-truth-boundary",
        "determinism-requirement",
        "directory-name-equals-id",
        "typed-authority-graph",
        "refusal-rule",
    ] {
        assert!(
            fm.unamendable.iter().any(|a| a == anchor),
            "000 unamendable must include {anchor}"
        );
    }

    // The root declares no relationship edges (it establishes nothing via the graph).
    assert!(fm.establishes.is_empty());
    assert!(fm.supersedes.is_empty());
    assert!(fm.amends.is_empty());

    // No unknown keys leaked into the overflow.
    assert!(
        fm.extra_frontmatter.is_empty(),
        "unexpected extra frontmatter: {:?}",
        fm.extra_frontmatter.keys().collect::<Vec<_>>()
    );
}

/// Spec 111 §3.5: the authoring template must document every key the parser
/// accepts, checked against [`KNOWN_KEYS`] itself rather than a list
/// transcribed here. A transcribed list is a third copy of the grammar and
/// goes stale exactly the way the template did: spec 103 added
/// `amends_verification` to the parser and the template never gained it, which
/// five specs recorded in their own §4 and none could close.
///
/// Substring-anchored rather than parsed, because the template's frontmatter is
/// mostly commented examples: `parse_frontmatter` would see only the six live
/// keys and say nothing about the 23 commented ones, which are the subject
/// (spec 111 D-5).
#[test]
fn authoring_template_documents_every_frontmatter_key() {
    let tpl = read_repo_file("standards/spec/templates/spec-template.md");

    // A key counts as documented when it opens a line, allowing indentation and
    // one leading comment marker. The anchoring is what keeps prose from
    // satisfying this: `kind` appears inside `{ kind: file, path: ... }` on
    // several example lines, and none of those is a top-level `kind:`.
    let documents = |key: &str| {
        let want = format!("{key}:");
        tpl.lines().any(|line| {
            let line = line.trim_start();
            let line = line.strip_prefix("# ").unwrap_or(line);
            line.starts_with(&want)
        })
    };

    let missing: Vec<&str> = KNOWN_KEYS
        .iter()
        .copied()
        .filter(|key| !documents(key))
        .collect();
    assert!(
        missing.is_empty(),
        "standards/spec/templates/spec-template.md documents {}/{} frontmatter \
         keys; missing: {missing:?}. A key the parser accepts and the template \
         omits is a key no author can find (spec 111 §3.2).",
        KNOWN_KEYS.len() - missing.len(),
        KNOWN_KEYS.len()
    );

    // §3.2: the two keys nothing reads are named as such, so the check above
    // stays total instead of carrying an exception list a future key could hide
    // in (spec 111 D-3).
    assert!(
        tpl.contains("read by nothing today"),
        "`code_aliases` and `feature_branch` must be documented as inert"
    );

    // §3.1: a key name alone satisfies the loop above while teaching nothing,
    // so the guidance that makes `amends_verification` usable is asserted
    // clause by clause.
    for clause in [
        "# amends_verification: [\"NNN-predecessor\"]",
        "Every entry MUST also appear in `amends` (`V-018`)",
        "`V-019`",
        "`V-020`",
        "skips a `superseded` or `retired`",
        "prints which spec's block it ran",
        "the amended file still carries the",
        "amendsVerification",
        "in FULL with the defect corrected, not a patch",
    ] {
        assert!(
            tpl.contains(clause),
            "the `amends_verification` guidance must state: {clause}"
        );
    }

    // §3.3: the nested forms the corpus actually writes. Neither is a key in
    // `KNOWN_KEYS`, so neither is reachable by the loop above.
    for nested in ["nature: additive", "planned: true", "raises no `W-001`"] {
        assert!(tpl.contains(nested), "the template must show: {nested}");
    }

    // §3.3: all six unit granularities.
    for kind in [
        "kind: file",
        "kind: section",
        "kind: symbol",
        "kind: directory",
        "kind: crate",
        "kind: module",
    ] {
        assert!(tpl.contains(kind), "the template must show: {kind}");
    }

    // §3.4: the two sections the template stopped short of. `verify` runs the
    // fenced block and the template never mentioned it.
    for section in [
        "## 5. Resolved decisions",
        "## Verification",
        "```verify:cli",
    ] {
        assert!(tpl.contains(section), "the template must carry: {section}");
    }
}
