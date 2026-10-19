//! Spec 166: `reference generate | check | model` through the CLI.

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../spec-spine-core/tests/fixtures/reference"
);

fn bin(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_spec-spine"))
        .arg("--repo")
        .arg(root)
        .args(args)
        .output()
        .unwrap()
}

fn code(out: &Output) -> i32 {
    out.status.code().unwrap()
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for e in fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let target = to.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            copy_dir(&e.path(), &target);
        } else {
            fs::copy(e.path(), target).unwrap();
        }
    }
}

/// The fixture corpus with its ledger compiled and indexed by the binary,
/// and a request without traceability or a catalog.
fn corpus() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    copy_dir(&Path::new(FIXTURE).join("corpus"), tmp.path());
    for verb in ["compile", "index"] {
        let out = bin(tmp.path(), &[verb]);
        assert_eq!(code(&out), 0, "{}", String::from_utf8_lossy(&out.stderr));
    }
    fs::write(
        tmp.path().join("reference-request.json"),
        r#"{"schemaVersion":"1.0.0","outputDir":"docs/reference","specs":{"select":"all"}}"#,
    )
    .unwrap();
    tmp
}

fn run(root: &Path, verb: &str, json: bool) -> Output {
    let mut args = vec!["reference", verb, "--request", "reference-request.json"];
    if json {
        args.push("--json");
    }
    Command::new(env!("CARGO_BIN_EXE_spec-spine"))
        .current_dir(root)
        .arg("--repo")
        .arg(root)
        .args(&args)
        .output()
        .unwrap()
}

#[test]
fn generate_writes_one_page_per_spec_and_check_then_passes() {
    let repo = corpus();
    let root = repo.path();
    assert_eq!(
        code(&run(root, "check", false)),
        1,
        "pages are missing before generation"
    );
    let out = run(root, "generate", true);
    assert_eq!(code(&out), 0, "{}", String::from_utf8_lossy(&out.stderr));
    let doc: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let names: Vec<String> = fs::read_dir(root.join("docs/reference"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names.len(), 5);
    assert_eq!(doc["written"].as_array().unwrap().len(), 5);
    assert!(doc["unchanged"].as_array().unwrap().is_empty());
    let first = fs::read(root.join("docs/reference/001-alpha.md")).unwrap();
    // A rerun writes nothing, and the JSON says so (D-21: written paths only).
    let again = run(root, "generate", true);
    assert_eq!(code(&again), 0);
    let doc: serde_json::Value = serde_json::from_slice(&again.stdout).unwrap();
    assert!(doc["written"].as_array().unwrap().is_empty());
    assert_eq!(doc["unchanged"].as_array().unwrap().len(), 5);
    assert_eq!(code(&run(root, "generate", false)), 0);
    assert_eq!(
        fs::read(root.join("docs/reference/001-alpha.md")).unwrap(),
        first
    );
    let check = run(root, "check", true);
    assert_eq!(code(&check), 0);
    let doc: serde_json::Value = serde_json::from_slice(&check.stdout).unwrap();
    assert!(doc["findings"].as_array().unwrap().is_empty());
}

#[test]
fn a_finding_exits_one_with_the_check_as_the_envelope_report_and_writes_nothing() {
    let repo = corpus();
    let root = repo.path();
    assert_eq!(code(&run(root, "generate", false)), 0);
    let path = root.join("docs/reference/002-beta.md");
    let edited = fs::read_to_string(&path)
        .unwrap()
        .replace("Beta extends alpha.", "Edited.");
    fs::write(&path, &edited).unwrap();
    let out = run(root, "check", true);
    assert_eq!(code(&out), 1);
    let envelope: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(envelope["verb"], "reference.check");
    assert_eq!(envelope["report"]["findings"][0]["kind"], "page-changed");
    assert_eq!(envelope["report"]["findings"][0]["regions"][0], "header");
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        edited,
        "the check never writes"
    );
    let human = run(root, "check", false);
    assert_eq!(code(&human), 1);
    assert!(
        String::from_utf8_lossy(&human.stdout)
            .contains("page-changed: docs/reference/002-beta.md [header]")
    );
}

#[test]
fn model_conforms_and_refusals_carry_their_codes() {
    let repo = corpus();
    let root = repo.path();
    let out = run(root, "model", true);
    assert_eq!(code(&out), 0, "{}", String::from_utf8_lossy(&out.stderr));
    let doc: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(
        doc["schemaVersion"],
        spec_spine_types::REFERENCE_PAGE_SCHEMA_VERSION
    );
    assert_eq!(doc["pages"].as_array().unwrap().len(), 5);
    assert_eq!(doc["pages"][0]["path"], "docs/reference/001-alpha.md");

    let write_request = |body: &str| fs::write(root.join("reference-request.json"), body).unwrap();
    write_request(
        r#"{"schemaVersion":"1.0.0","outputDir":"docs/reference","specs":{"ids":["099-none"]}}"#,
    );
    assert_eq!(code(&run(root, "generate", false)), 1, "an unknown spec");
    write_request(r#"{"schemaVersion":"1.0.0","outputDir":"../out","specs":{"select":"all"}}"#);
    assert_eq!(
        code(&run(root, "generate", false)),
        3,
        "a malformed request"
    );
    write_request(
        r#"{"schemaVersion":"1.0.0","outputDir":"docs/reference","specs":{"select":"all"},"extra":1}"#,
    );
    let out = run(root, "generate", true);
    assert_eq!(code(&out), 3);
    let envelope: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(envelope["verb"], "reference.generate");

    write_request(
        r#"{"schemaVersion":"1.0.0","outputDir":"docs/reference","specs":{"select":"all"}}"#,
    );
    assert_eq!(code(&run(root, "generate", false)), 0);
    let orphan = fs::read_to_string(root.join("docs/reference/002-beta.md"))
        .unwrap()
        .replace(
            "<!-- spec-spine:begin authored id=\"notes\" -->\n",
            "<!-- spec-spine:begin authored id=\"notes\" -->\nkeep\n",
        );
    fs::write(root.join("docs/reference/009-gone.md"), orphan).unwrap();
    assert_eq!(
        code(&run(root, "generate", false)),
        2,
        "an orphan with notes"
    );
    fs::remove_file(root.join("docs/reference/009-gone.md")).unwrap();

    fs::write(
        root.join("specs/002-beta/spec.md"),
        "---\nid: \"002-beta\"\n---\n",
    )
    .unwrap();
    let before = fs::read(root.join("docs/reference/001-alpha.md")).unwrap();
    assert_eq!(code(&run(root, "generate", false)), 1, "a stale ledger");
    assert_eq!(
        fs::read(root.join("docs/reference/001-alpha.md")).unwrap(),
        before
    );
    let stale = run(root, "check", true);
    assert_eq!(code(&stale), 1);
    let envelope: serde_json::Value = serde_json::from_slice(&stale.stdout).unwrap();
    assert_eq!(envelope["report"]["findings"][0]["kind"], "ledger-stale");
}

#[test]
fn every_verb_documents_itself() {
    let repo = corpus();
    for verb in ["generate", "check", "model"] {
        let out = bin(repo.path(), &["reference", verb, "--help"]);
        assert_eq!(code(&out), 0);
        assert!(String::from_utf8_lossy(&out.stdout).contains("--request"));
    }
}
