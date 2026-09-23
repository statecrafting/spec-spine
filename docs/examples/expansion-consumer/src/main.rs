//! A disposable consumer of the expansion wave (specs 102, 103, 106, 107, 108,
//! 109, 110, 113), written only against supported surfaces: the `spec-spine`
//! binary writes each corpus's committed ledger, as it would in an adopting
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
    closure_json, compile_json, couple_json, interface_verify_json, query_json, scope_compare_json,
    scope_json, verify_attestation_json,
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

    // ---- A governed corpus with code: 001-a owns src/a.rs, 002-b owns
    // src/b.rs, and src/c.rs is nobody's.
    let work = scratch.join("work");
    write_work(&work);
    cli(&bin, &work, &["compile"]);
    cli(&bin, &work, &["index"]);

    // ---- 108: a scope is evaluated against ownership, and compared.
    let ev = scope(&work, json!({ "id": "W-1", "ownSpec": "001", "mutable": ["src/a.rs", "src/b.rs"], "readOnly": ["src/c.rs"] }));
    let found: Vec<(String, String)> = ev["findings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| (str_of(&f["code"]), str_of(&f["path"])))
        .collect();
    assert_eq!(found, vec![("S-002".into(), "src/b.rs".into())], "{ev}");
    assert_eq!(ev["ownSpec"], "001-a");
    assert!(ev["indexHash"].is_string() && ev["schemaVersion"].is_string(), "{ev}");
    let shared = scope(&work, json!({ "ownSpec": "001", "mutable": ["src/a.rs"], "shared": [{ "path": "src/b.rs", "with": ["002"] }] }));
    assert_eq!(shared["findings"], json!([]), "declared sharing is no crossing");
    let err = scope_json("{}", s(&work), &json!({ "ownSpec": "999", "mutable": ["src/a.rs"] }).to_string()).unwrap_err();
    assert_eq!(err.exit_code(), 1, "an unknown ownSpec: {err}");
    let err = scope_json("{}", s(&work), &json!({ "ownSpec": "001", "mutable": ["../x"] }).to_string()).unwrap_err();
    assert_eq!(err.exit_code(), 3, "a path outside the repository: {err}");
    let a = json!({ "ownSpec": "001", "mutable": ["src/b.rs"] });
    let b = json!({ "ownSpec": "002", "mutable": ["src/"] });
    let r = json!({ "ownSpec": "002", "readOnly": ["src/b.rs"] });
    let kinds = |x: &Value, y: &Value| -> Vec<String> {
        let c: Value = serde_json::from_str(&scope_compare_json(&x.to_string(), &y.to_string()).unwrap()).unwrap();
        c["conflicts"].as_array().unwrap().iter().map(|k| str_of(&k["kind"])).collect()
    };
    assert_eq!(kinds(&a, &b), vec!["both-mutable"], "a subtree overlaps a file inside it");
    assert_eq!(kinds(&a, &r), vec!["changed-under-read"]);
    assert_eq!(kinds(&r, &r), Vec::<String>::new(), "two readers do not conflict");
    println!("ok: 108 scope names the crossing, accepts declared sharing, refuses unknown and escaping inputs, and compares");

    // ---- 113: a waiver's lifecycle is evaluated over inputs the caller gives.
    let both = ["src/a.rs", "src/b.rs"];
    let scoped = "Spec-Drift-Waiver: b only\nSpec-Drift-Waiver-Paths: src/b.rs\nSpec-Drift-Waiver-Until: 2026-12-31\n";
    let rep = couple(&work, &both, json!({ "prBody": scoped, "waiverInputs": { "asOf": "2026-10-01" } }));
    assert_eq!(cleared(&rep, 0), vec!["src/b.rs"], "{rep}");
    assert!(rep.get("waiver").is_none(), "src/a.rs still refuses, so the run is not waived");
    let expired = couple(&work, &both, json!({ "prBody": scoped, "waiverInputs": { "asOf": "2027-01-01" } }));
    assert_eq!(expired["waivers"][0]["effective"], false);
    assert_eq!(cleared(&expired, 0), Vec::<String>::new(), "a failed check clears nothing");
    let undated = couple(&work, &both, json!({ "prBody": scoped }));
    assert_eq!(undated["waivers"][0]["checks"][0]["outcome"], "not-evaluated");
    assert_eq!(cleared(&undated, 0), vec!["src/b.rs"], "not evaluated is not failed");
    let once = "Spec-Drift-Waiver: once\nSpec-Drift-Waiver-Max-Uses: 1\n";
    let id = str_of(&couple(&work, &both, json!({ "prBody": once }))["waivers"][0]["id"]);
    let mut uses = serde_json::Map::new();
    uses.insert(id, json!(1));
    let spent = couple(&work, &both, json!({ "prBody": once, "waiverInputs": { "uses": uses } }));
    assert_eq!(spent["waivers"][0]["checks"][0]["outcome"], "failed", "the caller's count reached the limit");
    let none = couple(&work, &both, json!({}));
    assert!(none.get("waivers").is_none(), "no waiver declared, no new member: {none}");
    let two_sources = json!({ "repoRoot": s(&work), "diff": diff(&both), "prBody": scoped, "waiver": { "reason": "x" } });
    assert_eq!(couple_json(&two_sources.to_string()).unwrap_err().exit_code(), 3);
    println!("ok: 113 scoped, expired, not-evaluated and spent waivers each report and clear as declared");

    // ---- Composed: a work order holds a closure and a scope, changes the
    // code, and asks the gate. What the scope called a crossing is exactly
    // what the gate refuses; a waiver scoped to it clears exactly that; and
    // neither the gate nor the waiver moves the closure or the scope.
    let order_closure = json!({ "obligations": ["001#R-1"], "specs": ["001"] });
    let before = closure(&work, order_closure.clone());
    let changed = ["specs/001-a/spec.md", "src/a.rs", "src/b.rs"];
    let gate = couple(&work, &changed, json!({}));
    let refused: Vec<String> = gate["violations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| str_of(&v["path"]))
        .collect();
    let crossings: Vec<String> = ev["findings"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|f| f["code"] == "S-002")
        .map(|f| str_of(&f["path"]))
        .collect();
    assert_eq!(refused, crossings, "the scope predicted the gate's refusal");
    let body = format!(
        "Spec-Drift-Waiver: the declared crossing\nSpec-Drift-Waiver-Paths: {}\nSpec-Drift-Waiver-Until: 2026-12-31\n",
        crossings.join(", ")
    );
    let waived = couple(&work, &changed, json!({ "prBody": body, "waiverInputs": { "asOf": "2026-10-01" } }));
    assert_eq!(waived["waiver"], "the declared crossing", "every refusal cleared: {waived}");
    assert_eq!(cleared(&waived, 0), crossings, "and nothing beyond the crossing");
    assert_eq!(closure(&work, order_closure)["digest"], before["digest"], "the gate moved no closure");
    let again = scope(&work, json!({ "id": "W-1", "ownSpec": "001", "mutable": ["src/a.rs", "src/b.rs"], "readOnly": ["src/c.rs"] }));
    assert_eq!(again, ev, "the gate and the waiver moved no scope evaluation");
    println!("ok: composed: scope crossing == gate refusal; a waiver scoped to it clears exactly it; closure and scope unchanged");
}

fn replay(fixtures: &Path, id: &str) {
    let dir = fixtures.join(id);
    let case: Value =
        serde_json::from_str(&fs::read_to_string(dir.join("case.json")).unwrap()).unwrap();
    let expect = &case["expect"];
    // `needsCorpus` is the case's own statement; the directory must agree.
    let needs = case["needsCorpus"].as_bool().expect("needsCorpus is a bool");
    assert_eq!(needs, dir.join("corpus").is_dir(), "{id}: needsCorpus disagrees with corpus/");
    let corpus = if needs {
        dir.join("corpus")
    } else {
        // Refused before any recompute, so any corpus serves as the root.
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

fn scope(root: &Path, req: Value) -> Value {
    serde_json::from_str(&scope_json("{}", s(root), &req.to_string()).unwrap()).unwrap()
}

fn diff(paths: &[&str]) -> Value {
    json!({ "files": paths.iter().map(|p| json!({ "path": p, "hunks": [], "deleted": false })).collect::<Vec<_>>() })
}

/// `couple_json` over `paths`, with `extra` members merged into the request.
fn couple(root: &Path, paths: &[&str], extra: Value) -> Value {
    let mut req = json!({ "repoRoot": s(root), "diff": diff(paths) });
    for (k, v) in extra.as_object().unwrap() {
        req[k] = v.clone();
    }
    serde_json::from_str(&couple_json(&req.to_string()).unwrap()).unwrap()
}

fn cleared(report: &Value, n: usize) -> Vec<String> {
    report["waivers"][n]["clears"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| str_of(&c["path"]))
        .collect()
}

fn write_work(root: &Path) {
    fs::create_dir_all(root.join("src")).unwrap();
    for f in ["a", "b", "c"] {
        fs::write(root.join(format!("src/{f}.rs")), format!("pub fn {f}() {{}}\n")).unwrap();
    }
    spec(
        root,
        "001-a",
        "status: draft\nestablishes:\n  - \"src/a.rs\"\nobligations:\n  - { id: \"R-1\", kind: requirement, text: \"A holds.\", anchor: \"3-1-the-rule\" }\n",
        "# 001\n\n## 3. Behavior\n\n### 3.1 The rule\n\nA holds.\n",
    );
    spec(root, "002-b", "status: draft\nestablishes:\n  - \"src/b.rs\"\n", "# 002\n");
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
