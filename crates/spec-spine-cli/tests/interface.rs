// Spec: specs/110-an-interface-reference-is-digest-pinned/spec.md
//! Spec 110 §3.3: `interface verify` through the shipped binary. The core
//! tests prove each outcome; this proves the verb a consumer calls: every exit
//! code, the read document, the text form, the `--export` usage refusals, and
//! that nothing is written. Every pin is copied from the exporter's own
//! `registry show --json`, the route §3.1 tells an author to take.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const EXPORTER_ID: &str = "002-environment-lifecycle";
const ANCHOR: &str = "3-13-the-managed-instruction-file";
const BODY: &str = "# 002\n\n## 3. Behavior\n\n### 3.13 The managed instruction file\n\nThe bridge line is inserted once.\n\n### 3.14 Unrelated\n\nOther text.\n";

fn run_in(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_spec-spine"))
        .arg("--repo")
        .arg(root)
        .args(args)
        .output()
        .expect("spawn spec-spine")
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

fn write_spec(root: &Path, id: &str, extra: &str, body: &str) {
    let d = root.join("specs").join(id);
    fs::create_dir_all(&d).unwrap();
    fs::write(
        d.join("spec.md"),
        format!(
            "---\nid: \"{id}\"\ntitle: \"T\"\nstatus: draft\ncreated: \"2026-09-22\"\n\
             summary: \"s\"\n{extra}---\n{body}"
        ),
    )
    .unwrap();
}

/// An exporter corpus, compiled, and the pin its own registry reports.
fn exporter() -> (tempfile::TempDir, String, String) {
    let t = tempfile::tempdir().unwrap();
    write_spec(t.path(), EXPORTER_ID, "", BODY);
    let o = run_in(t.path(), &["compile"]);
    assert_eq!(code(&o), 0, "{}", text(&o));
    let o = run_in(t.path(), &["registry", "show", EXPORTER_ID, "--json"]);
    assert_eq!(code(&o), 0, "{}", text(&o));
    let doc: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    let digest = format!("sha256:{}", doc["contentHash"].as_str().unwrap());
    let section = format!("sha256:{}", doc["sectionDigests"][ANCHOR].as_str().unwrap());
    (t, digest, section)
}

fn importer(refs: &str) -> tempfile::TempDir {
    let t = tempfile::tempdir().unwrap();
    write_spec(
        t.path(),
        "001-bridge",
        &format!("interface_references:\n{refs}"),
        "# 001\n\n## 3. Behavior\n\nText.\n",
    );
    let o = run_in(t.path(), &["compile"]);
    assert_eq!(code(&o), 0, "{}", text(&o));
    t
}

fn whole(digest: &str) -> String {
    format!(
        "  - corpus: \"statecraft-cli\"\n    spec: \"{EXPORTER_ID}\"\n    digest: \"{digest}\"\n    obtained: \"2026-09-22\"\n"
    )
}

fn sectioned(digest: &str, section: &str) -> String {
    format!(
        "{}    sections:\n      - anchor: \"{ANCHOR}\"\n        digest: \"{section}\"\n",
        whole(digest)
    )
}

fn export_arg(dir: &Path) -> String {
    format!("statecraft-cli={}", dir.display())
}

fn verify(imp: &Path, exp: &Path, extra: &[&str]) -> Output {
    let e = export_arg(exp);
    let mut args = vec!["interface", "verify", "--export", e.as_str()];
    args.extend_from_slice(extra);
    run_in(imp, &args)
}

#[test]
fn a_current_pin_exits_0_with_a_read_document() {
    let (exp, digest, _) = exporter();
    let imp = importer(&whole(&digest));
    let o = verify(imp.path(), exp.path(), &["--json"]);
    assert_eq!(code(&o), 0, "{}", text(&o));
    let doc: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(doc["schemaVersion"], spec_spine_types::READ_SCHEMA_VERSION);
    assert_eq!(doc["references"][0]["outcome"], "current");
    assert_eq!(doc["references"][0]["observedDigest"], digest.as_str());
    assert_eq!(doc["summary"]["current"], 1);
}

#[test]
fn an_edit_elsewhere_is_sections_current_at_exit_0_and_stale_at_exit_1_unsectioned() {
    let (exp, digest, section) = exporter();
    write_spec(
        exp.path(),
        EXPORTER_ID,
        "",
        &BODY.replace("Other text.", "Other text, rewritten."),
    );

    let imp = importer(&sectioned(&digest, &section));
    let o = verify(imp.path(), exp.path(), &[]);
    assert_eq!(code(&o), 0, "{}", text(&o));
    let t = text(&o);
    assert!(t.contains("sections-current"), "{t}");
    assert!(t.contains("observed sha256:"), "{t}");

    let imp = importer(&whole(&digest));
    let o = verify(imp.path(), exp.path(), &[]);
    assert_eq!(code(&o), 1, "{}", text(&o));
    let t = text(&o);
    assert!(t.contains("stale  001-bridge -> statecraft-cli:"), "{t}");
    assert!(t.contains(&format!("pinned   {digest}")), "{t}");
}

#[test]
fn an_edit_to_the_pinned_section_is_stale_and_names_the_section() {
    let (exp, digest, section) = exporter();
    write_spec(
        exp.path(),
        EXPORTER_ID,
        "",
        &BODY.replace("inserted once", "inserted twice"),
    );
    let imp = importer(&sectioned(&digest, &section));
    let o = verify(imp.path(), exp.path(), &[]);
    assert_eq!(code(&o), 1, "{}", text(&o));
    assert!(
        text(&o).contains(&format!("section {ANCHOR}: stale, pinned {section}")),
        "{}",
        text(&o)
    );
}

#[test]
fn missing_and_unverified_both_refuse_at_exit_1() {
    let (_, digest, _) = exporter();
    let imp = importer(&whole(&digest));

    // No --export at all: a citation nobody could check did not hold.
    let o = run_in(imp.path(), &["interface", "verify", "--json"]);
    assert_eq!(code(&o), 1, "{}", text(&o));
    let doc: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(doc["references"][0]["outcome"], "unverified");

    // An export that lacks the spec.
    let empty = tempfile::tempdir().unwrap();
    fs::create_dir_all(empty.path().join("specs")).unwrap();
    let o = verify(imp.path(), empty.path(), &[]);
    assert_eq!(code(&o), 1, "{}", text(&o));
    assert!(text(&o).contains("missing"), "{}", text(&o));
}

#[test]
fn a_stale_ledger_exits_2_before_the_export_is_read() {
    let (exp, digest, _) = exporter();
    let imp = importer(&whole(&digest));
    let spec = imp.path().join("specs/001-bridge/spec.md");
    let s = fs::read_to_string(&spec).unwrap();
    fs::write(&spec, s.replace("Text.", "Text, edited.")).unwrap();
    // The export points nowhere: reading it first would be exit 3.
    let o = verify(imp.path(), &exp.path().join("absent"), &[]);
    assert_eq!(code(&o), 2, "{}", text(&o));
}

#[test]
fn malformed_exports_and_an_unreadable_directory_are_exit_3() {
    let (exp, digest, _) = exporter();
    let imp = importer(&whole(&digest));
    for bad in ["=x", "noequals", "statecraft-cli="] {
        let o = run_in(imp.path(), &["interface", "verify", "--export", bad]);
        assert_eq!(code(&o), 3, "{bad}: {}", text(&o));
    }
    let e = export_arg(exp.path());
    let o = run_in(
        imp.path(),
        &["interface", "verify", "--export", &e, "--export", &e],
    );
    assert_eq!(code(&o), 3, "{}", text(&o));
    assert!(text(&o).contains("twice"), "{}", text(&o));

    let o = verify(imp.path(), &exp.path().join("absent"), &[]);
    assert_eq!(code(&o), 3, "{}", text(&o));
}

#[test]
fn an_unknown_spec_exits_1_and_the_short_form_is_accepted() {
    let (exp, digest, _) = exporter();
    let imp = importer(&whole(&digest));
    let o = verify(imp.path(), exp.path(), &["--spec", "999"]);
    assert_eq!(code(&o), 1, "{}", text(&o));
    let a = verify(imp.path(), exp.path(), &["--spec", "001", "--json"]);
    let b = verify(imp.path(), exp.path(), &["--spec", "001-bridge", "--json"]);
    assert_eq!(code(&a), 0, "{}", text(&a));
    assert_eq!(a.stdout, b.stdout);
}

#[test]
fn a_corpus_with_no_references_answers_empty_at_exit_0() {
    let t = tempfile::tempdir().unwrap();
    write_spec(t.path(), "001-a", "", "# 001\n");
    assert_eq!(code(&run_in(t.path(), &["compile"])), 0);
    let o = run_in(t.path(), &["interface", "verify", "--json"]);
    assert_eq!(code(&o), 0, "{}", text(&o));
    let doc: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(doc["references"], serde_json::json!([]));
}

#[test]
fn the_verb_writes_nothing_in_either_tree() {
    let (exp, digest, _) = exporter();
    let imp = importer(&whole(&digest));
    let (a, b) = (tree(imp.path()), tree(exp.path()));
    let o = verify(imp.path(), exp.path(), &["--json"]);
    assert_eq!(code(&o), 0, "{}", text(&o));
    assert_eq!(tree(imp.path()), a);
    assert_eq!(tree(exp.path()), b);
}

fn tree(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in fs::read_dir(&d).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                stack.push(p);
            } else {
                let b = fs::read(&p).unwrap();
                out.push((p, b));
            }
        }
    }
    out.sort();
    out
}
