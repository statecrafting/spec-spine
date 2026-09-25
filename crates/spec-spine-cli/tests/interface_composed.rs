// Spec: specs/110-an-interface-reference-is-digest-pinned/spec.md
//! Spec 110 composed with the wave it depends on, through the shipped binary.
//!
//! `closure_composed.rs` (spec 107 D-9) asserts 102, 106 and 107 together.
//! These cases add 109 and 110: one exporter corpus declares obligations and
//! an impact against them, and an importer in another directory pins the
//! exporter's spec and one of its sections. What must agree is identity: the
//! section digest one surface reports is the digest every other surface uses,
//! so a change moves each of them together and a status flip moves only the
//! identities that cover the whole file.

use std::fs;
use std::path::Path;
use std::process::{Command, Output, Stdio};

use serde_json::Value;

const RULE_ANCHOR: &str = "3-1-the-rule";

fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_spec-spine"))
        .arg("--repo")
        .arg(root)
        .args(args)
        .stdin(Stdio::null())
        .output()
        .expect("spawn spec-spine")
}

fn text(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

fn json(root: &Path, args: &[&str], want: i32) -> Value {
    let out = run(root, args);
    assert_eq!(out.status.code(), Some(want), "{args:?}: {}", text(&out));
    serde_json::from_slice(&out.stdout).unwrap()
}

fn compile(root: &Path) {
    let out = run(root, &["compile"]);
    assert_eq!(out.status.code(), Some(0), "compile: {}", text(&out));
}

/// The exporter: `001-a` declares `R-1` (live, in 3.1) and `R-2` (withdrawn),
/// and `002-b` declares an impact on each.
struct Exporter {
    tmp: tempfile::TempDir,
    status: &'static str,
    rule: &'static str,
    other: &'static str,
}

impl Exporter {
    fn new() -> Self {
        let e = Exporter {
            tmp: tempfile::tempdir().unwrap(),
            status: "draft",
            rule: "The rule text.",
            other: "Other text.",
        };
        let dir = e.root().join("specs/002-b");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("spec.md"),
            "---\nid: \"002-b\"\ntitle: \"B\"\nstatus: draft\ncreated: \"2026-09-22\"\nsummary: \"s\"\n\
             impacts:\n  - obligation: \"001#R-1\"\n    nature: refines\n  - obligation: \"001#R-2\"\n    nature: informs\n\
             ---\n# 002\n\n## 1. Purpose\n\nP.\n",
        )
        .unwrap();
        e.write();
        e
    }

    fn root(&self) -> &Path {
        self.tmp.path()
    }

    fn write(&self) {
        let dir = self.root().join("specs/001-a");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("spec.md"),
            format!(
                "---\nid: \"001-a\"\ntitle: \"A\"\nstatus: {}\ncreated: \"2026-09-22\"\nsummary: \"s\"\n\
                 obligations:\n  - id: \"R-1\"\n    kind: requirement\n    text: \"The rule holds.\"\n    anchor: \"{RULE_ANCHOR}\"\n\
                 \x20 - id: \"R-2\"\n    kind: requirement\n    text: \"Old.\"\n    anchor: \"3-2-the-other-rule\"\n    withdrawn: true\n\
                 ---\n# 001\n\n## 3. Behavior\n\n### 3.1 The rule\n\n{}\n\n### 3.2 The other rule\n\n{}\n",
                self.status, self.rule, self.other
            ),
        )
        .unwrap();
        compile(self.root());
    }

    fn show(&self) -> Value {
        json(self.root(), &["registry", "show", "001", "--json"], 0)
    }

    fn closure_digest(&self, request: &str) -> String {
        let path = self.root().join("request.json");
        fs::write(&path, request).unwrap();
        let doc = json(
            self.root(),
            &[
                "registry",
                "closure",
                "--request",
                path.to_str().unwrap(),
                "--json",
            ],
            0,
        );
        doc["digest"].as_str().unwrap().to_string()
    }
}

/// An importer pinning `001-a` whole (`whole`) and by its rule section
/// (`sectioned`), in two specs, from the exporter's registry as it is now.
fn importer(exp: &Exporter) -> tempfile::TempDir {
    let show = exp.show();
    let digest = format!("sha256:{}", show["contentHash"].as_str().unwrap());
    let section = format!(
        "sha256:{}",
        show["sectionDigests"][RULE_ANCHOR].as_str().unwrap()
    );
    let t = tempfile::tempdir().unwrap();
    for (id, sections) in [
        ("001-whole", String::new()),
        (
            "002-sectioned",
            format!(
                "    sections:\n      - anchor: \"{RULE_ANCHOR}\"\n        digest: \"{section}\"\n"
            ),
        ),
    ] {
        let dir = t.path().join("specs").join(id);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("spec.md"),
            format!(
                "---\nid: \"{id}\"\ntitle: \"T\"\nstatus: draft\ncreated: \"2026-09-22\"\nsummary: \"s\"\n\
                 interface_references:\n  - corpus: \"upstream\"\n    spec: \"001-a\"\n    digest: \"{digest}\"\n    obtained: \"2026-09-22\"\n{sections}\
                 ---\n# {id}\n"
            ),
        )
        .unwrap();
    }
    compile(t.path());
    t
}

/// Outcome per declaring spec, and the process exit code.
fn outcomes(imp: &Path, exp: &Path) -> (Vec<(String, String)>, i32) {
    let e = format!("upstream={}", exp.display());
    let out = run(imp, &["interface", "verify", "--export", &e, "--json"]);
    let code = out.status.code().unwrap_or(-1);
    assert!(code == 0 || code == 1, "{}", text(&out));
    let doc: Value = serde_json::from_slice(&out.stdout).unwrap();
    let v = doc["references"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            (
                r["declaredBy"].as_str().unwrap().to_string(),
                r["outcome"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    (v, code)
}

fn pair(a: &str, b: &str) -> (String, String) {
    (a.to_string(), b.to_string())
}

/// 106 with 107 and 110: the section digest `registry obligation` reports
/// for `R-1`, the one `registry show` lists for its anchor, and the one an
/// importer pins are the same value, and the importer verifies `current`.
#[test]
fn one_section_identity_across_obligation_show_and_pin() {
    let exp = Exporter::new();
    let ob = json(
        exp.root(),
        &["registry", "obligation", "001#R-1", "--json"],
        0,
    );
    let show = exp.show();
    assert_eq!(ob["sectionDigest"], show["sectionDigests"][RULE_ANCHOR]);
    assert_eq!(ob["contentHash"], show["contentHash"]);

    let imp = importer(&exp);
    let (o, code) = outcomes(imp.path(), exp.root());
    assert_eq!(code, 0);
    assert_eq!(
        o,
        vec![
            pair("001-whole", "current"),
            pair("002-sectioned", "current")
        ]
    );
}

/// 106, 107, 109 and 110: editing the section an obligation is anchored in
/// moves the obligation's closure, stales both pins (the sectioned one names
/// the section), and leaves the declared impact set byte-identical: an impact
/// is what an author said, not something recomputed from the text.
#[test]
fn a_changed_obligation_section_moves_the_closure_and_stales_both_pins() {
    let mut exp = Exporter::new();
    let imp = importer(&exp);
    let by_ob = r#"{"obligations":["001#R-1"]}"#;
    let closure_before = exp.closure_digest(by_ob);
    let impacts_before = run(exp.root(), &["registry", "impacts", "--json"]).stdout;

    exp.rule = "The rule text, amended.";
    exp.write();

    assert_ne!(closure_before, exp.closure_digest(by_ob));
    assert_eq!(
        impacts_before,
        run(exp.root(), &["registry", "impacts", "--json"]).stdout
    );
    let (o, code) = outcomes(imp.path(), exp.root());
    assert_eq!(code, 1);
    assert_eq!(
        o,
        vec![pair("001-whole", "stale"), pair("002-sectioned", "stale")]
    );
}

/// 102 and 110: ratifying the exporter's spec is an edit to `spec.md`, so a
/// whole-spec pin goes stale and a section pin stays verified as
/// `sections-current`, and the same edit leaves the obligation's closure alone.
/// An edit elsewhere in the body behaves the same way.
#[test]
fn a_status_flip_stales_only_whole_file_identities() {
    let mut exp = Exporter::new();
    let imp = importer(&exp);
    let by_ob = r#"{"obligations":["001#R-1"]}"#;
    let closure_before = exp.closure_digest(by_ob);

    exp.status = "approved";
    exp.write();
    assert_eq!(closure_before, exp.closure_digest(by_ob));
    let (o, code) = outcomes(imp.path(), exp.root());
    assert_eq!(code, 1, "the whole-spec pin refuses");
    assert_eq!(
        o,
        vec![
            pair("001-whole", "stale"),
            pair("002-sectioned", "sections-current")
        ]
    );

    exp.status = "draft";
    exp.other = "Other text, rewritten.";
    exp.write();
    assert_eq!(closure_before, exp.closure_digest(by_ob));
    let (o, _) = outcomes(imp.path(), exp.root());
    assert_eq!(o[1], pair("002-sectioned", "sections-current"));
}

/// 106 and 109: an impact on a withdrawn obligation keeps pointing at that id
/// and says it is withdrawn; the live one does not.
#[test]
fn an_impact_on_a_withdrawn_obligation_names_it_withdrawn() {
    let exp = Exporter::new();
    let doc = json(exp.root(), &["registry", "impacts", "--json"], 0);
    let by_target: Vec<(String, bool)> = doc["impacts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| {
            (
                i["target"].as_str().unwrap().to_string(),
                i["targetWithdrawn"].as_bool().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        by_target,
        vec![
            ("001-a#R-1".to_string(), false),
            ("001-a#R-2".to_string(), true)
        ]
    );
}

/// 110 with the tolerant read: the exporter's committed ledger may be stale
/// (its `registry show` still answers the old digest), but the verifier
/// recomputes from `spec.md`, so the importer sees the change regardless.
/// What the importer's own ledger says is refused when stale (exit 2).
#[test]
fn a_stale_exporter_ledger_cannot_hide_a_change_and_a_stale_importer_ledger_refuses() {
    let exp = Exporter::new();
    let imp = importer(&exp);
    // Edit the exporter's rule WITHOUT recompiling it.
    let path = exp.root().join("specs/001-a/spec.md");
    let s = fs::read_to_string(&path).unwrap();
    fs::write(&path, s.replace("The rule text.", "Changed, uncompiled.")).unwrap();
    let (o, code) = outcomes(imp.path(), exp.root());
    assert_eq!(code, 1);
    assert_eq!(
        o,
        vec![pair("001-whole", "stale"), pair("002-sectioned", "stale")]
    );

    let ip = imp.path().join("specs/001-whole/spec.md");
    let s = fs::read_to_string(&ip).unwrap();
    fs::write(&ip, format!("{s}\nEdited.\n")).unwrap();
    let e = format!("upstream={}", exp.root().display());
    let out = run(imp.path(), &["interface", "verify", "--export", &e]);
    assert_eq!(out.status.code(), Some(1), "{}", text(&out));
}
