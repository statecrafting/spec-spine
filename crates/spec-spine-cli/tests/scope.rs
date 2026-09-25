// Spec: specs/108-a-work-scope-is-declared/spec.md
//! Spec 108 §3.6: `spec-spine scope evaluate` and `spec-spine scope compare`
//! through the shipped binary. The core tests prove the resolution and
//! comparison logic; this proves the verbs a consumer calls: stdin and a
//! file agree, every exit code, and that evaluating and comparing exit 0
//! whether or not they found anything (§3.3, §3.5, §3.7).

use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Output, Stdio};

fn run(root: &Path, args: &[&str], stdin: Option<&str>) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_spec-spine"))
        .arg("--repo")
        .arg(root)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn spec-spine");
    if let Some(text) = stdin {
        child
            .stdin
            .take()
            .unwrap()
            .write_all(text.as_bytes())
            .unwrap();
    } else {
        drop(child.stdin.take());
    }
    child.wait_with_output().unwrap()
}

fn code(o: &Output) -> i32 {
    o.status.code().unwrap_or(-1)
}

fn text(o: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    )
}

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, content).unwrap();
}

/// `100-a` owns `crates/a/src/lib.rs`, `200-b` owns `crates/b/src/lib.rs`,
/// committed registry + index.
fn corpus() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    let r = tmp.path();
    write(r, "crates/a/src/lib.rs", "pub fn a() {}\n");
    write(r, "crates/b/src/lib.rs", "pub fn b() {}\n");
    write(
        r,
        "specs/100-a/spec.md",
        "---\nid: \"100-a\"\ntitle: \"A\"\nstatus: draft\ncreated: \"2026-09-22\"\n\
         summary: \"s\"\nestablishes:\n  - \"crates/a/src/lib.rs\"\n---\n# a\n",
    );
    write(
        r,
        "specs/200-b/spec.md",
        "---\nid: \"200-b\"\ntitle: \"B\"\nstatus: draft\ncreated: \"2026-09-22\"\n\
         summary: \"s\"\nestablishes:\n  - \"crates/b/src/lib.rs\"\n---\n# b\n",
    );
    let o = run(r, &["compile"], None);
    assert_eq!(code(&o), 0, "{}", text(&o));
    let o = run(r, &["index"], None);
    assert_eq!(code(&o), 0, "{}", text(&o));
    tmp
}

const SELF_OWNED: &str = r#"{"ownSpec":"100","mutable":["crates/a/src/lib.rs"]}"#;

// ---- evaluate: stdin and a file agree --------------------------------------

#[test]
fn evaluate_stdin_and_a_file_give_the_same_read_document() {
    let tmp = corpus();
    let from_stdin = run(
        tmp.path(),
        &["scope", "evaluate", "--scope", "-", "--json"],
        Some(SELF_OWNED),
    );
    assert_eq!(code(&from_stdin), 0, "{}", text(&from_stdin));

    let f = tmp.path().join("scope.json");
    fs::write(&f, SELF_OWNED).unwrap();
    let from_file = run(
        tmp.path(),
        &[
            "scope",
            "evaluate",
            "--scope",
            f.to_str().unwrap(),
            "--json",
        ],
        None,
    );
    assert_eq!(from_file.stdout, from_stdin.stdout);

    let doc: serde_json::Value = serde_json::from_slice(&from_stdin.stdout).unwrap();
    assert_eq!(doc["ownSpec"], "100-a");
    assert_eq!(doc["schemaVersion"], spec_spine_types::READ_SCHEMA_VERSION);
    assert_eq!(spec_spine_types::READ_SCHEMA_VERSION, "0.8.0");
    assert_eq!(doc["findings"].as_array().unwrap().len(), 0);
    assert!(!doc["indexHash"].as_str().unwrap().is_empty());

    // The text form names the path.
    let text_out = run(
        tmp.path(),
        &["scope", "evaluate", "--scope", "-"],
        Some(SELF_OWNED),
    );
    assert!(
        String::from_utf8_lossy(&text_out.stdout).contains("crates/a/src/lib.rs"),
        "{}",
        text(&text_out)
    );
}

// ---- evaluate: findings reach the CLI --------------------------------------

#[test]
fn a_crossing_is_reported_as_s002_naming_both() {
    let tmp = corpus();
    let req = r#"{"ownSpec":"100","mutable":["crates/b/src/lib.rs"]}"#;
    let o = run(
        tmp.path(),
        &["scope", "evaluate", "--scope", "-", "--json"],
        Some(req),
    );
    assert_eq!(code(&o), 0, "{}", text(&o));
    let doc: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(doc["findings"][0]["code"], "S-002");
    assert_eq!(doc["findings"][0]["ownSpec"], "100-a");
    assert_eq!(doc["findings"][0]["owners"][0], "200-b");
}

#[test]
fn an_unowned_mutable_path_is_reported_as_s001() {
    let tmp = corpus();
    let req = r#"{"ownSpec":"100","mutable":["crates/nowhere.rs"]}"#;
    let o = run(
        tmp.path(),
        &["scope", "evaluate", "--scope", "-", "--json"],
        Some(req),
    );
    assert_eq!(code(&o), 0, "{}", text(&o));
    let doc: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(doc["findings"][0]["code"], "S-001");
}

// ---- evaluate: every refusal has its exit code -----------------------------

#[test]
fn each_evaluate_refusal_has_its_exit_code() {
    let tmp = corpus();
    let code_for = |stdin: &str| {
        code(&run(
            tmp.path(),
            &["scope", "evaluate", "--scope", "-"],
            Some(stdin),
        ))
    };
    assert_eq!(code_for("{}"), 3, "no ownSpec, and no path at all");
    assert_eq!(
        code_for(r#"{"ownSpec":"100"}"#),
        3,
        "no path across any role"
    );
    assert_eq!(
        code_for(r#"{"ownSpec":"100","mutable":["/etc/passwd"]}"#),
        3,
        "an absolute path"
    );
    assert_eq!(
        code_for(r#"{"ownSpec":"100","mutable":["a/../b"]}"#),
        3,
        "a '..' segment"
    );
    assert_eq!(
        code_for(r#"{"ownSpec":"100","other":true,"mutable":["a"]}"#),
        3,
        "an unknown member"
    );
    assert_eq!(
        code_for(
            r#"{"ownSpec":"100","mutable":["crates/a/src/lib.rs"],"readOnly":["crates/a/src/lib.rs"]}"#
        ),
        3,
        "one path under two roles"
    );
    assert_eq!(code_for("not json"), 3, "unparseable");
    assert_eq!(
        code_for(r#"{"ownSpec":"999","mutable":["crates/a/src/lib.rs"]}"#),
        1,
        "an unknown ownSpec"
    );
    let unknown = run(
        tmp.path(),
        &["scope", "evaluate", "--scope", "-"],
        Some(r#"{"ownSpec":"999","shared":[{"path":"crates/a/src/lib.rs","with":["888"]}]}"#),
    );
    assert_eq!(code(&unknown), 1);
    let msg = text(&unknown);
    assert!(msg.contains("999"), "{msg}");
    assert!(msg.contains("888"), "{msg}");

    // Edited and not recompiled: the index no longer vouches for the corpus.
    let spec = tmp.path().join("specs/100-a/spec.md");
    let src = fs::read_to_string(&spec).unwrap();
    fs::write(&spec, src.replace("# a", "# a, edited")).unwrap();
    assert_eq!(code_for(SELF_OWNED), 1, "stale");
}

#[test]
fn evaluate_exits_zero_whether_or_not_it_found_anything() {
    let tmp = corpus();
    let clean = run(
        tmp.path(),
        &["scope", "evaluate", "--scope", "-"],
        Some(SELF_OWNED),
    );
    assert_eq!(code(&clean), 0);
    let crossing = run(
        tmp.path(),
        &["scope", "evaluate", "--scope", "-"],
        Some(r#"{"ownSpec":"100","mutable":["crates/b/src/lib.rs"]}"#),
    );
    assert_eq!(code(&crossing), 0, "a report, not a gate");
}

// ---- compare: stdin, a file, and each refusal ------------------------------

#[test]
fn compare_stdin_and_a_file_agree_and_report_a_conflict() {
    let tmp = corpus();
    let a = r#"{"ownSpec":"100","mutable":["crates/x.rs"]}"#;
    let b_path = tmp.path().join("b.json");
    fs::write(&b_path, r#"{"ownSpec":"200","mutable":["crates/x.rs"]}"#).unwrap();

    let o = run(
        tmp.path(),
        &["scope", "compare", "-", b_path.to_str().unwrap(), "--json"],
        Some(a),
    );
    assert_eq!(code(&o), 0, "{}", text(&o));
    let doc: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(doc["schemaVersion"], spec_spine_types::READ_SCHEMA_VERSION);
    assert_eq!(doc["conflicts"][0]["kind"], "both-mutable");
    assert_eq!(doc["a"]["ownSpec"], "100");
    assert_eq!(doc["b"]["ownSpec"], "200");
}

#[test]
fn compare_with_no_overlap_exits_zero_with_no_conflicts() {
    let tmp = corpus();
    let a = r#"{"ownSpec":"100","mutable":["crates/x.rs"]}"#;
    let b = r#"{"ownSpec":"200","mutable":["crates/y.rs"]}"#;
    let a_path = tmp.path().join("a.json");
    fs::write(&a_path, a).unwrap();
    let o = run(
        tmp.path(),
        &["scope", "compare", a_path.to_str().unwrap(), "-", "--json"],
        Some(b),
    );
    assert_eq!(code(&o), 0, "no conflict is still exit 0");
    let doc: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(doc["conflicts"].as_array().unwrap().len(), 0);
}

#[test]
fn compare_refuses_both_arguments_reading_stdin() {
    let tmp = corpus();
    let o = run(tmp.path(), &["scope", "compare", "-", "-"], Some("{}"));
    assert_eq!(code(&o), 3, "{}", text(&o));
}

#[test]
fn compare_never_touches_the_committed_index() {
    // Compare reads no ledger (§3.5): it works even against a repo with no
    // committed registry or index at all.
    let tmp = tempfile::tempdir().unwrap();
    let a = r#"{"ownSpec":"anything","mutable":["x"]}"#;
    let b = r#"{"ownSpec":"else","mutable":["x"]}"#;
    let a_path = tmp.path().join("a.json");
    let b_path = tmp.path().join("b.json");
    fs::write(&a_path, a).unwrap();
    fs::write(&b_path, b).unwrap();
    let o = run(
        tmp.path(),
        &[
            "scope",
            "compare",
            a_path.to_str().unwrap(),
            b_path.to_str().unwrap(),
            "--json",
        ],
        None,
    );
    assert_eq!(code(&o), 0, "{}", text(&o));
}
