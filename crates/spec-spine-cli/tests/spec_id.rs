// Spec: specs/084-a-short-id-names-the-same-spec-at-every-verb/spec.md
//! Spec 084 §3.5: the six-argument matrix.
//!
//! 049 §3.2 and 056 §3.1 each asserted the cross-verb rule in prose, and
//! nothing held it: four of the six arguments refused the short form through
//! two ratifications. This file drives all six against one fixture corpus and
//! is the behavioural half of §3.4, since a private copy of the policy would
//! have to reproduce both refusal messages byte for byte to pass.
//!
//! The census in `crates/spec-spine-cli/src/main.rs` is the structural half:
//! the two lists are kept equal by its failure message rather than by linkage,
//! because an integration test cannot import from a binary crate (084 D-5).

use std::fs;
use std::path::Path;
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_spec-spine"))
}

fn code(out: &std::process::Output) -> i32 {
    out.status.code().unwrap_or(-1)
}

fn write_spec(root: &Path, id: &str, extra: &str) {
    let spec_dir = root.join("specs").join(id);
    fs::create_dir_all(&spec_dir).unwrap();
    let body = format!(
        "---\nid: \"{id}\"\ntitle: \"Title {id}\"\nstatus: approved\ncreated: \"2026-09-11\"\nsummary: \"s\"\n{extra}---\n\n# {id}\n\n## Verification\n\n```verify:cli\ntrue\n```\n"
    );
    fs::write(spec_dir.join("spec.md"), body).unwrap();
}

/// One fixture corpus, compiled and indexed, with an attestation on disk for
/// the one spec `verify-attestation` is pointed at.
fn corpus() -> tempfile::TempDir {
    let t = tempfile::tempdir().unwrap();
    write_spec(
        t.path(),
        "016-short",
        "establishes:\n  - { kind: file, path: \"src/a.rs\" }\n",
    );
    write_spec(t.path(), "049-verify", "depends_on: [\"016-short\"]\n");
    write_spec(t.path(), "056-compile", "amends: [\"016-short\"]\n");
    fs::create_dir_all(t.path().join("src")).unwrap();
    fs::write(t.path().join("src").join("a.rs"), "// a\n").unwrap();
    for v in [
        &["compile"][..],
        &["index"][..],
        &["attest", "--spec", "016-short"][..],
    ] {
        let out = run(t.path(), v);
        assert_eq!(code(&out), 0, "fixture setup `{v:?}`: {}", stderr(&out));
    }
    t
}

fn run(repo: &Path, args: &[&str]) -> std::process::Output {
    bin().arg("--repo").arg(repo).args(args).output().unwrap()
}

fn stdout(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// The six arguments of §3.1, as argv tails taking the id last where the verb
/// allows it. `verify` needs `--plan` so the matrix never executes what a
/// fixture corpus declares.
fn six(id: &str) -> Vec<(&'static str, Vec<String>)> {
    vec![
        (
            "registry show",
            vec!["registry".into(), "show".into(), id.into()],
        ),
        (
            "registry relationships",
            vec!["registry".into(), "relationships".into(), id.into()],
        ),
        (
            "compile --spec",
            vec!["compile".into(), "--spec".into(), id.into()],
        ),
        ("verify", vec!["verify".into(), id.into(), "--plan".into()]),
        (
            "attest --spec",
            vec!["attest".into(), "--spec".into(), id.into()],
        ),
        (
            "verify-attestation --spec",
            vec![
                "verify-attestation".into(),
                "--spec".into(),
                id.into(),
                "--recompute".into(),
            ],
        ),
    ]
}

fn argv(v: &[String]) -> Vec<&str> {
    v.iter().map(|s| s.as_str()).collect()
}

// ===== §3.1: the full and short forms agree, at all six =====

/// §3.1 and §3.3: byte-identical stdout and the same exit code. Before 084 four
/// of these six refused the short form outright.
#[test]
fn the_short_and_full_forms_are_byte_identical_at_all_six_arguments() {
    let t = corpus();
    for ((name, full), (_, short)) in six("016-short").into_iter().zip(six("016")) {
        let a = run(t.path(), &argv(&full));
        let b = run(t.path(), &argv(&short));
        assert_eq!(code(&a), 0, "{name} full form: {}", stderr(&a));
        assert_eq!(code(&b), 0, "{name} refused the short form: {}", stderr(&b));
        assert_eq!(stdout(&a), stdout(&b), "{name} stdout differs");
    }
}

/// §3.3, and the regression §1.3 names: `relationships` compared the raw
/// argument to compute its incoming edges, so a partial fix prints a spec's
/// outgoing edges with an empty `depended_on_by` at exit 0. Equality alone
/// cannot see a regression that breaks both forms alike, so the edge is named.
#[test]
fn relationships_keeps_its_incoming_edges_under_the_short_form() {
    let t = corpus();
    let out = run(t.path(), &["registry", "relationships", "016", "--json"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let v: serde_json::Value = serde_json::from_str(&stdout(&out)).unwrap();
    assert_eq!(v["id"], "016-short");
    assert_eq!(v["dependedOnBy"][0], "049-verify");
    assert_eq!(v["amendedBy"][0], "056-compile");
}

/// §3.3: the short form must not produce an attestation over zero units, which
/// is what resolving the record lookup alone yields, at exit 0 with
/// `resolution.ok` still true.
#[test]
fn attest_under_the_short_form_lists_the_same_units() {
    let t = corpus();
    let short = run(t.path(), &["attest", "--spec", "016", "--json"]);
    let full = run(t.path(), &["attest", "--spec", "016-short", "--json"]);
    assert_eq!(code(&short), 0, "{}", stderr(&short));
    let s = stdout(&short);
    assert!(s.contains("\"contentHash\""), "zero-unit payload: {s}");
    assert_eq!(s, stdout(&full));
}

/// §3.3: the file is named by the resolved id, and nothing is named after the
/// argument. Two files for one spec is the shape `verify-attestation --spec
/// <full id>` then fails to read.
#[test]
fn attest_names_the_file_by_the_resolved_id() {
    let t = corpus();
    let dir = t.path().join(".derived/attestation/by-spec");
    fs::remove_file(dir.join("016-short.json")).unwrap();
    let out = run(t.path(), &["attest", "--spec", "016"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        dir.join("016-short.json").is_file(),
        "resolved name missing"
    );
    assert!(
        !dir.join("016.json").exists(),
        "a file named after the argument"
    );
}

// ===== §3.1 steps 3 and 4: one refusal, whichever verb produced it =====

/// Step 3. All six refuse at exit 1 with one message naming both candidates.
///
/// The attestation files for the two candidates are written **empty**: a
/// correct implementation refuses before reading them, and one that read first
/// would fail to parse and exit 3, not 1. That is what makes "resolve before
/// read" observable rather than assumed.
#[test]
fn an_ambiguous_ordinal_is_one_refusal_at_all_six_arguments() {
    let t = tempfile::tempdir().unwrap();
    write_spec(t.path(), "001-a", "");
    write_spec(t.path(), "001-b", "");
    // A corpus with a shared ordinal fails V-004, and `compile` still writes
    // the shards `registry show` reads (spec 001 §3.5).
    let out = run(t.path(), &["compile"]);
    assert_eq!(code(&out), 1, "a shared ordinal must fail V-004");
    assert!(
        t.path()
            .join(".derived/spec-registry/by-spec/001-a.json")
            .is_file(),
        "compile must still write the shards"
    );
    let att = t.path().join(".derived/attestation/by-spec");
    fs::create_dir_all(&att).unwrap();
    fs::write(att.join("001-a.json"), "").unwrap();
    fs::write(att.join("001-b.json"), "").unwrap();

    let mut messages: Vec<String> = Vec::new();
    for (name, args) in six("001") {
        let out = run(t.path(), &argv(&args));
        assert_eq!(code(&out), 1, "{name} did not refuse at exit 1");
        let msg = stderr(&out).trim().to_string();
        assert!(msg.contains("ambiguous"), "{name}: {msg}");
        assert!(
            msg.contains("001-a") && msg.contains("001-b"),
            "{name}: {msg}"
        );
        messages.push(msg);
    }
    assert_eq!(messages.len(), 6, "every argument must write a refusal");
    messages.dedup();
    assert_eq!(
        messages.len(),
        1,
        "the refusal must not name which verb produced it: {messages:?}"
    );
}

/// Step 4, at the five arguments that refuse it. `verify-attestation` is the
/// exception and is asserted separately below.
#[test]
fn no_match_is_one_refusal_at_the_five_arguments_that_refuse_it() {
    let t = corpus();
    let mut messages: Vec<String> = Vec::new();
    for (name, args) in six("999").into_iter().take(5) {
        let out = run(t.path(), &argv(&args));
        assert_eq!(code(&out), 1, "{name}: {}", stderr(&out));
        let msg = stderr(&out).trim().to_string();
        assert!(!msg.is_empty(), "{name} wrote no refusal");
        messages.push(msg);
    }
    assert_eq!(messages.len(), 5);
    messages.dedup();
    assert_eq!(messages.len(), 1, "five different refusals: {messages:?}");
}

/// §3.2 and D-4: at `verify-attestation`, step 4 does not refuse. The argument
/// falls through as given and the read fails exactly as it does today, which
/// spec 042 §3.5 assigns to exit 3.
#[test]
fn verify_attestation_falls_through_to_exit_three_on_no_match() {
    let t = corpus();
    let out = run(
        t.path(),
        &["verify-attestation", "--spec", "999", "--recompute"],
    );
    assert_eq!(code(&out), 3, "{}", stderr(&out));
    assert!(
        stderr(&out).contains("attest --spec"),
        "the hint must survive: {}",
        stderr(&out)
    );
}

// ===== §3.2 and D-6: a path is not an id =====

/// Each of these resolved through the filesystem before 084: the two `verify`
/// forms exited 0, and `compile --spec ./<id>` reported a false `V-001` on a
/// valid spec because the "resolved id" was the path the caller typed.
#[test]
fn a_path_shaped_argument_is_refused_not_walked_to() {
    let t = corpus();
    for arg in ["../specs/016-short", "016-short/", "./016-short"] {
        let out = run(t.path(), &["verify", arg, "--plan"]);
        assert_eq!(code(&out), 1, "verify resolved a path: {arg}");
    }
    let out = run(t.path(), &["compile", "--spec", "./016-short"]);
    assert_eq!(code(&out), 1);
    let msg = format!("{}{}", stdout(&out), stderr(&out));
    assert!(msg.contains("not found"), "{msg}");
    assert!(!msg.contains("V-001"), "a false validation failure: {msg}");
}

/// §3.2: `validate_spec_id` still runs on the **raw** argument, before any
/// resolution, and still refuses at exit 3. It is what keeps step 4's
/// fall-through from building a filename that reads outside `by-spec/`.
#[test]
fn verify_attestation_still_refuses_a_separator_before_resolving() {
    let t = corpus();
    for arg in ["../x", "a/b", ""] {
        let out = run(
            t.path(),
            &["verify-attestation", "--spec", arg, "--recompute"],
        );
        assert_eq!(code(&out), 3, "{arg}: {}", stderr(&out));
    }
}

// ===== §3.6: what must keep working =====

/// An unknown id keeps its exit code, and a partial ordinal is not an ordinal.
#[test]
fn an_unknown_id_and_a_partial_ordinal_keep_their_exit_codes() {
    let t = corpus();
    for arg in ["999", "16", "016-typo"] {
        let out = run(t.path(), &["registry", "show", arg]);
        assert_eq!(code(&out), 1, "{arg}: {}", stderr(&out));
    }
}

/// Every id argument's help text says it accepts the short form, as `verify`'s
/// already did (§3.1).
#[test]
fn every_id_argument_documents_the_short_form() {
    for (verb, needle) in [
        (vec!["registry", "show", "--help"], "short"),
        (vec!["registry", "relationships", "--help"], "short"),
        (vec!["compile", "--help"], "short id"),
        (vec!["verify", "--help"], "short"),
        (vec!["attest", "--help"], "short id"),
        (vec!["verify-attestation", "--help"], "short id"),
    ] {
        let out = bin().args(&verb).output().unwrap();
        let text = format!("{}{}", stdout(&out), stderr(&out));
        assert!(
            text.contains(needle),
            "{verb:?} help does not mention the short form:\n{text}"
        );
    }
}
