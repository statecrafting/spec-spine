// Spec: specs/169-declared-obligation-traceability/spec.md
//! Spec 169 §3.9: `registry traceability` through the shipped binary. The
//! core tests prove the rules; this proves the verb a consumer calls: the
//! read document and its axes, the filters, the human form, and the exit
//! contract.

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn run_in(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_spec-spine"))
        .arg("--repo")
        .arg(root)
        .args(args)
        .output()
        .expect("spawn spec-spine")
}

fn ok(out: &Output) {
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn write(root: &Path, id: &str, extra: &str) {
    let dir = root.join("specs").join(id);
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("spec.md"),
        format!(
            "---\nid: \"{id}\"\ntitle: \"T\"\nstatus: draft\ncreated: \"2026-10-10\"\nsummary: \"s\"\n{extra}\
             ---\n# {id}\n\n## 3. Behavior\n\n### 3.1 The rule\n\nText.\n\n## Verification\n\nChecks.\n"
        ),
    )
    .unwrap();
}

/// A two-spec corpus: 001-a claims `src/lib.rs` and declares `R-1`; 002-b
/// declares three relations against it, one resolving, one unsupported, one
/// unresolved. Compiled and indexed through the binary.
fn corpus() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir_all(tmp.path().join("src")).unwrap();
    fs::write(tmp.path().join("src/lib.rs"), "pub fn f() {}\n").unwrap();
    write(
        tmp.path(),
        "001-a",
        "establishes:\n  - \"src/lib.rs\"\nobligations:\n  - id: \"R-1\"\n    kind: requirement\n    text: \"The rule holds.\"\n    anchor: \"3-1-the-rule\"\n",
    );
    write(
        tmp.path(),
        "002-b",
        "traceability:\n\
         \x20 - id: \"t-test\"\n    obligation: \"001#R-1\"\n    relation: tested-by\n    target: { kind: test, selector: \"demo::tests::t\" }\n\
         \x20 - id: \"t-impl\"\n    obligation: \"001#R-1\"\n    relation: implemented-by\n    target: { kind: unit, spec: \"001\", unit: \"src/lib.rs\" }\n\
         \x20 - id: \"t-doc\"\n    obligation: \"001#R-1\"\n    relation: documented-by\n    target: { kind: documentation, selector: { kind: file, path: \"docs/none.md\" } }\n",
    );
    ok(&run_in(tmp.path(), &["compile"]));
    ok(&run_in(tmp.path(), &["index"]));
    tmp
}

fn doc(out: &Output) -> serde_json::Value {
    serde_json::from_slice(&out.stdout).unwrap()
}

#[test]
fn the_verb_answers_a_read_document_with_both_axes() {
    let tmp = corpus();
    let out = run_in(tmp.path(), &["registry", "traceability", "--json"]);
    ok(&out);
    let d = doc(&out);
    assert_eq!(d["schemaVersion"], spec_spine_types::READ_SCHEMA_VERSION);
    assert_eq!(
        d["traceabilityVersion"],
        spec_spine_types::TRACEABILITY_SCHEMA_VERSION
    );
    let states: Vec<(&str, &str)> = d["relations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| (r["id"].as_str().unwrap(), r["state"].as_str().unwrap()))
        .collect();
    assert_eq!(
        states,
        [
            ("t-doc", "unresolved"),
            ("t-impl", "resolved"),
            ("t-test", "unsupported")
        ]
    );
    assert_eq!(d["summary"]["resolved"], 1);
    assert_eq!(d["summary"]["unknown"], 0);
    assert_eq!(d["relations"][1]["obligation"], "001-a#R-1");
    // The CLI and the JSON facade emit the same bytes for the same inputs.
    let facade = spec_spine_core::traceability_json(
        &serde_json::to_string(&spec_spine_types::Config::default()).unwrap(),
        tmp.path().to_str().unwrap(),
        "{}",
    )
    .unwrap();
    assert_eq!(String::from_utf8(out.stdout).unwrap(), facade);
}

#[test]
fn filters_intersect_and_the_human_form_sorts_the_same() {
    let tmp = corpus();
    let out = run_in(
        tmp.path(),
        &[
            "registry",
            "traceability",
            "--declared-by",
            "002",
            "--obligation",
            "001#R-1",
            "--state",
            "resolved",
            "--json",
        ],
    );
    ok(&out);
    let d = doc(&out);
    assert_eq!(d["relations"].as_array().unwrap().len(), 1);
    assert_eq!(d["relations"][0]["id"], "t-impl");

    let human = run_in(tmp.path(), &["registry", "traceability"]);
    ok(&human);
    let text = String::from_utf8(human.stdout).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 3);
    assert!(lines[0].starts_with("unresolved") && lines[0].contains("002-b#trace:t-doc"));
    assert!(lines[1].starts_with("resolved") && lines[1].contains("002-b#trace:t-impl"));
    assert!(lines[2].starts_with("unsupported") && lines[2].contains("002-b#trace:t-test"));
    assert!(lines[1].contains("implemented-by"));
}

#[test]
fn a_valid_read_exits_zero_whatever_the_states_and_a_bad_request_does_not() {
    let tmp = corpus();
    ok(&run_in(
        tmp.path(),
        &["registry", "traceability", "--state", "unresolved"],
    ));
    let bad_state = run_in(
        tmp.path(),
        &["registry", "traceability", "--state", "maybe"],
    );
    assert_eq!(bad_state.status.code(), Some(3));
    let unqualified = run_in(
        tmp.path(),
        &["registry", "traceability", "--obligation", "R-1"],
    );
    assert_eq!(unqualified.status.code(), Some(3));
    let unknown = run_in(
        tmp.path(),
        &["registry", "traceability", "--declared-by", "999"],
    );
    assert_eq!(unknown.status.code(), Some(1));
}

#[test]
fn an_empty_corpus_reports_no_declarations() {
    let tmp = tempfile::tempdir().unwrap();
    write(tmp.path(), "001-a", "");
    ok(&run_in(tmp.path(), &["compile"]));
    let out = run_in(tmp.path(), &["registry", "traceability"]);
    ok(&out);
    assert_eq!(
        String::from_utf8(out.stdout).unwrap(),
        "(no declarations)\n"
    );
}

#[test]
fn a_stale_index_makes_a_unit_target_unknown_not_resolved() {
    let tmp = corpus();
    // A spec added after `index` stales the committed index; the unit target
    // is then unknown, and the selector targets are answered as before.
    write(tmp.path(), "003-c", "establishes:\n  - \"src/lib.rs\"\n");
    ok(&run_in(tmp.path(), &["compile"]));
    let out = run_in(tmp.path(), &["registry", "traceability", "--json"]);
    ok(&out);
    let d = doc(&out);
    let impl_ = d["relations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == "t-impl")
        .unwrap();
    assert_eq!(impl_["state"], "unknown", "{d}");
    let doc_ = d["relations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == "t-doc")
        .unwrap();
    assert_eq!(doc_["state"], "unresolved");

    // With no index at all, likewise unknown, and still exit 0.
    fs::remove_dir_all(tmp.path().join(".derived/codebase-index")).unwrap();
    let out = run_in(tmp.path(), &["registry", "traceability", "--json"]);
    ok(&out);
    assert_eq!(doc(&out)["summary"]["unknown"], 1);
}

#[test]
fn capabilities_name_the_verb_and_its_axes() {
    let out = Command::new(env!("CARGO_BIN_EXE_spec-spine"))
        .args(["capabilities", "--json"])
        .output()
        .unwrap();
    ok(&out);
    let d = doc(&out);
    let verb = d["verbs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["path"] == "registry traceability")
        .expect("registry traceability is listed");
    let axes: Vec<&str> = verb["json"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["axis"].as_str().unwrap())
        .collect();
    assert_eq!(axes, ["read", "traceability"]);
}
