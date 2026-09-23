//! A disposable consumer of the expansion wave (specs 102, 103, 106, 107, 109,
//! 110), written only against supported surfaces: the `spec-spine` binary
//! writes each corpus's committed ledger, as it would in an adopting
//! repository, and every read goes through the packaged library's JSON facade.
//!
//! Usage: expansion-consumer <spec-spine binary> <fixtures/verifier dir> <scratch dir>
//!
//! Prints one `ok:` line per contract and exits non-zero on the first that
//! does not hold.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{Value, json};
use spec_spine_core::{
    closure_json, compile_json, interface_verify_json, query_json, verify_attestation_json,
};

const RULE: &str = "3-1-the-rule";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    assert_eq!(args.len(), 4, "usage: {} <bin> <fixtures> <scratch>", args[0]);
    let bin = PathBuf::from(&args[1]);
    let fixtures = PathBuf::from(&args[2]);
    let scratch = PathBuf::from(&args[3]);

    let exporter = scratch.join("exporter");
    let importer = scratch.join("importer");
    write_exporter(&exporter, "The rule text.", "Other text.");
    cli(&bin, &exporter, &["compile"]);

    // ---- 102: a ready entry carries its status; readiness is not approval.
    let registry = compile_json("{}", s(&exporter)).unwrap();
    let reg: Value = serde_json::from_str(&registry).unwrap();
    assert_eq!(reg["validation"]["passed"], true, "{}", reg["validation"]);
    let plan = query(&registry, json!({ "op": "plan" }));
    let ready: Vec<(String, String)> = plan["ready"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| (str_of(&r["id"]), str_of(&r["status"])))
        .collect();
    assert!(ready.contains(&("001-a".into(), "draft".into())), "{ready:?}");
    assert!(ready.contains(&("003-c".into(), "approved".into())), "{ready:?}");
    println!("ok: 102 plan entries carry status (a draft is ready; approval is the consumer's rule)");

    // ---- 106: one obligation, one section identity across two reads.
    let ob = query(&registry, json!({ "op": "obligation", "id": "001#R-1" }));
    let show = query(&registry, json!({ "op": "show", "id": "001" }));
    assert_eq!(ob["sectionDigest"], show["sectionDigests"][RULE]);
    let err = query_json(
        &json!({ "registry": registry, "op": "obligation", "id": "R-1" }).to_string(),
    )
    .unwrap_err();
    assert_eq!(err.exit_code(), 3, "an unqualified obligation id: {err}");
    println!("ok: 106 obligation resolves; its section digest equals show's; unqualified id exits 3");

    // ---- 109: declared impacts, inverted, withdrawn target named.
    let imp = query(&registry, json!({ "op": "impacts", "target": "001-a" }));
    let targets: Vec<(String, bool)> = imp["impacts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| (str_of(&i["target"]), i["targetWithdrawn"].as_bool().unwrap()))
        .collect();
    assert_eq!(
        targets,
        vec![("001-a#R-1".into(), false), ("001-a#R-2".into(), true)]
    );
    println!("ok: 109 impacts inverted onto 001-a, the withdrawn target reported as withdrawn");

    // ---- 107: a closure is normalized, content-addressed, and refuses gaps.
    let d1 = closure(&exporter, json!({ "obligations": ["001#R-1"], "specs": ["003"] }));
    let d1b = closure(
        &exporter,
        json!({ "specs": ["003-c", "003"], "obligations": ["001-a#R-1", "001#R-1"] }),
    );
    assert_eq!(d1["digest"], d1b["digest"], "order, duplicates, short ids");
    let err = closure_json(
        "{}",
        s(&exporter),
        &json!({ "obligations": ["001#R-9"] }).to_string(),
    )
    .unwrap_err();
    assert_eq!(err.exit_code(), 1, "a missing member: {err}");
    println!("ok: 107 closure digest normalized; a missing member exits 1 with no digest");

    // ---- 110: pin 001-a from the exporter's own identities, then verify.
    // The whole-spec identity is the closure member's contentHash; the
    // section identity is the registry's sectionDigests entry.
    let whole = closure(&exporter, json!({ "specs": ["001"] }));
    let content = str_of(&whole["members"][0]["contentHash"]);
    let section = str_of(&show["sectionDigests"][RULE]);
    write_importer(&importer, &content, &section);
    cli(&bin, &importer, &["compile"]);
    let imp_registry = compile_json("{}", s(&importer)).unwrap();
    let verify = |label: &str| -> Vec<String> {
        let text = fs::read_to_string(exporter.join("specs/001-a/spec.md")).unwrap();
        let req = json!({
            "registry": imp_registry,
            "exports": { "upstream": { "001-a": { "path": "specs/001-a/spec.md", "text": text } } },
        });
        let doc: Value =
            serde_json::from_str(&interface_verify_json(&req.to_string()).unwrap()).unwrap();
        let o: Vec<String> = doc["references"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| str_of(&r["outcome"]))
            .collect();
        println!("   110 {label}: {o:?}");
        o
    };
    assert_eq!(verify("unchanged"), vec!["current", "current"]);

    // An edit outside the pinned section.
    write_exporter(&exporter, "The rule text.", "Other text, rewritten.");
    assert_eq!(verify("3.2 edited"), vec!["stale", "sections-current"]);

    // An edit inside it; the obligation's closure moves with it.
    write_exporter(&exporter, "The rule, amended.", "Other text, rewritten.");
    assert_eq!(verify("3.1 edited"), vec!["stale", "stale"]);
    cli(&bin, &exporter, &["compile"]);
    let d2 = closure(&exporter, json!({ "obligations": ["001#R-1"], "specs": ["003"] }));
    assert_ne!(d1["digest"], d2["digest"]);

    // No export supplied: unverified, never a pass.
    let req = json!({ "registry": imp_registry });
    let doc: Value =
        serde_json::from_str(&interface_verify_json(&req.to_string()).unwrap()).unwrap();
    assert_eq!(doc["summary"]["unverified"], 2);
    println!("ok: 110 current / sections-current / stale / unverified, and the closure moved with the section");

    // ---- 103: replay the published fixture set through the facade.
    let index: Value =
        serde_json::from_str(&fs::read_to_string(fixtures.join("index.json")).unwrap()).unwrap();
    let cases = index["cases"].as_array().unwrap();
    assert!(!cases.is_empty());
    for c in cases {
        replay(&fixtures, c.as_str().unwrap());
    }
    println!("ok: 103 all {} fixture cases reproduce their recorded outcome", cases.len());
}

fn replay(fixtures: &Path, id: &str) {
    let dir = fixtures.join(id);
    let case: Value =
        serde_json::from_str(&fs::read_to_string(dir.join("case.json")).unwrap()).unwrap();
    let expect = &case["expect"];
    let corpus = if dir.join("corpus").is_dir() {
        dir.join("corpus")
    } else {
        fixtures.join("control-untampered/corpus")
    };
    let bytes = fs::read(dir.join("payload.json")).unwrap();
    let Ok(text) = String::from_utf8(bytes) else {
        // A facade takes text; bytes that are not UTF-8 cannot reach it, which
        // is the same refusal class the CLI reports.
        assert_eq!(expect["exit"], 3, "{id}");
        return;
    };
    let req = json!({ "repoRoot": s(&corpus), "attestationText": text }).to_string();
    match verify_attestation_json(&req) {
        Ok(out) => {
            let v: Value = serde_json::from_str(&out).unwrap();
            let got = str_of(&v["outcome"]);
            let want = match (str_of(&expect["outcome"]).as_str(), expect["reason"].as_str()) {
                ("accepted", _) => "match",
                (_, Some("version-mismatch")) => "versionMismatch",
                (_, Some("content-mismatch" | "non-canonical-bytes")) => "contentMismatch",
                other => panic!("{id}: answered {got}, recorded {other:?}"),
            };
            assert_eq!(got, want, "{id}");
        }
        Err(e) => assert_eq!(i64::from(e.exit_code()), expect["exit"].as_i64().unwrap(), "{id}: {e}"),
    }
}

fn query(registry: &str, mut req: Value) -> Value {
    req["registry"] = json!(registry);
    serde_json::from_str(&query_json(&req.to_string()).unwrap()).unwrap()
}

fn closure(root: &Path, req: Value) -> Value {
    serde_json::from_str(&closure_json("{}", s(root), &req.to_string()).unwrap()).unwrap()
}

fn cli(bin: &Path, root: &Path, args: &[&str]) {
    let st = Command::new(bin)
        .arg("--repo")
        .arg(root)
        .args(args)
        .status()
        .unwrap();
    assert!(st.success(), "spec-spine {args:?} in {}", root.display());
}

fn s(p: &Path) -> &str {
    p.to_str().unwrap()
}

fn str_of(v: &Value) -> String {
    v.as_str().unwrap_or_else(|| panic!("not a string: {v}")).to_string()
}

fn spec(root: &Path, id: &str, head: &str, body: &str) {
    let d = root.join("specs").join(id);
    fs::create_dir_all(&d).unwrap();
    fs::write(
        d.join("spec.md"),
        format!(
            "---\nid: \"{id}\"\ntitle: \"{id}\"\ncreated: \"2026-09-22\"\nsummary: \"s\"\nimplementation: pending\n{head}---\n{body}"
        ),
    )
    .unwrap();
}

fn write_exporter(root: &Path, rule: &str, other: &str) {
    spec(
        root,
        "001-a",
        "status: draft\nobligations:\n  - { id: \"R-1\", kind: requirement, text: \"The rule holds.\", anchor: \"3-1-the-rule\" }\n  - { id: \"R-2\", kind: requirement, text: \"Old.\", anchor: \"3-2-the-other-rule\", withdrawn: true }\n",
        &format!("# 001\n\n## 3. Behavior\n\n### 3.1 The rule\n\n{rule}\n\n### 3.2 The other rule\n\n{other}\n"),
    );
    spec(
        root,
        "002-b",
        "status: draft\ndepends_on: [\"001-a\"]\nimpacts:\n  - { obligation: \"001#R-1\", nature: refines }\n  - { obligation: \"001#R-2\", nature: informs }\n",
        "# 002\n",
    );
    spec(root, "003-c", "status: approved\n", "# 003\n");
}

fn write_importer(root: &Path, content: &str, section: &str) {
    for (id, extra) in [
        ("001-whole", String::new()),
        (
            "002-sectioned",
            format!("    sections:\n      - {{ anchor: \"{RULE}\", digest: \"sha256:{section}\" }}\n"),
        ),
    ] {
        spec(
            root,
            id,
            &format!(
                "status: draft\ninterface_references:\n  - corpus: \"upstream\"\n    spec: \"001-a\"\n    digest: \"sha256:{content}\"\n    obtained: \"2026-09-22\"\n{extra}"
            ),
            "# x\n",
        );
    }
}
