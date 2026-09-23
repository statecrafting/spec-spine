//! A disposable consumer of the expansion wave (specs 102, 103, 106, 107, 108,
//! 109, 110, 113, and since 0.24.0 specs 111, 112, 114 and 116), written only
//! against supported surfaces: the `spec-spine`
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
    closure_json, compile_json, couple_json, interface_verify_json, lint_json, query_json,
    scope_compare_json, scope_json, verify_attestation_json,
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

    since_0_24_0(&bin, &scratch);
}

/// The contracts added in 0.24.0. Each is additive: nothing above changes.
/// `CONSUMER_ONLY=111,116` runs a subset, so a negative control can show each
/// section failing on its own against a release that lacks it.
fn since_0_24_0(bin: &Path, scratch: &Path) {
    let only = std::env::var("CONSUMER_ONLY").ok();
    let want = |id: &str| only.as_deref().is_none_or(|o| o.split(',').any(|x| x == id));
    if want("111") {
        contract_111(bin, scratch);
    }
    if want("112") {
        contract_112(bin, scratch);
    }
    if want("114") {
        contract_114(scratch);
    }
    if want("116") {
        contract_116(scratch);
    }
}

fn contract_111(bin: &Path, scratch: &Path) {
    // ---- 111: a move is a reviewed mapping, looked up and never inferred.
    let mv = scratch.join("moves");
    write_moves(&mv, true);
    cli(bin, &mv, &["compile"]);
    cli(bin, &mv, &["index"]);
    let registry = compile_json("{}", s(&mv)).unwrap();
    let reg: Value = serde_json::from_str(&registry).unwrap();
    assert_eq!(reg["validation"]["passed"], true, "{}", reg["validation"]);
    let look = |path: &str| query(&registry, json!({ "op": "moves", "path": path }));
    let unmapped = look("src/c.rs");
    assert_eq!(unmapped["outcome"], "unmapped", "{unmapped}");
    assert!(unmapped["schemaVersion"].is_string(), "a versioned read document");
    let chain = look("src/a.rs");
    assert_eq!(chain["outcome"], "resolved", "{chain}");
    let hops: Vec<(String, String)> = chain["hops"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| (str_of(&h["from"]), str_of(&h["to"])))
        .collect();
    assert_eq!(
        hops,
        vec![
            ("src/a.rs".into(), "src/a2.rs".into()),
            ("src/a2.rs".into(), "src/a3.rs".into())
        ],
        "a two-hop chain across two specs"
    );
    assert_eq!(chain["terminals"], json!([{ "path": "src/a3.rs" }]), "{chain}");
    let split = look("src/s.rs");
    assert_eq!(split["outcome"], "resolved", "{split}");
    assert_eq!(
        split["terminals"],
        json!([{ "path": "src/s1.rs" }, { "path": "src/s2.rs" }]),
        "a split follows every declared branch"
    );
    let amb = look("src/x.rs");
    assert_eq!(amb["outcome"], "ambiguous", "{amb}");
    let cands: Vec<String> = amb["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| str_of(&c["declaredBy"]))
        .collect();
    assert_eq!(cands, vec!["001-a", "002-b"], "every candidate named, none picked");
    let cyc = look("src/p.rs");
    assert_eq!(cyc["outcome"], "cycle", "{cyc}");
    assert_eq!(cyc["chain"], json!(["src/p.rs", "src/q.rs", "src/p.rs"]), "{cyc}");
    let all = query(&registry, json!({ "op": "moves" }));
    let froms: Vec<String> = all["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| str_of(&i["from"]))
        .collect();
    let mut sorted = froms.clone();
    sorted.sort();
    assert_eq!(froms, sorted, "the flattened map is sorted");
    assert_eq!(froms.len(), 9, "{all}");
    // The CLI and the facade answer from the same ledger; `unmapped` and
    // `resolved` exit 0, `ambiguous` and `cycle` exit 1 (111 §3.4).
    for (path, code) in [("src/a.rs", 0), ("src/c.rs", 0), ("src/s.rs", 0), ("src/x.rs", 1), ("src/p.rs", 1)] {
        let via_cli = cli_json(bin, &mv, &["registry", "moves", path, "--json"], code);
        assert_eq!(via_cli, look(path), "`registry moves {path}` and the facade disagree");
    }
    // `answered_by: "001"` resolves (a short id, spec 084), so no V-041 here;
    // a dangling one below is a warning, never a refusal.
    assert!(!reg["validation"].to_string().contains("V-041"), "{}", reg["validation"]);
    // L-015: a declared `to` absent from the tree is a lint warning.
    let lint_mv: Value = serde_json::from_str(&lint_json("{}", s(&mv)).unwrap()).unwrap();
    assert!(lint_mv.to_string().contains("L-015"), "no L-015 for a missing `to`: {lint_mv}");
    assert!(!lint_mv.to_string().contains("L-016"), "{lint_mv}");

    // A declared move clears no deletion: the same deletion, with and without
    // the mapping, reaches the identical verdict, and it is a refusal.
    let bare = scratch.join("moves-bare");
    write_moves(&bare, false);
    cli(bin, &bare, &["compile"]);
    cli(bin, &bare, &["index"]);
    let deletion = |root: &Path| -> Value {
        let req = json!({
            "repoRoot": s(root),
            "diff": { "files": [{ "path": "src/a.rs", "hunks": [], "deleted": true }] },
        });
        serde_json::from_str(&couple_json(&req.to_string()).unwrap()).unwrap()
    };
    let with_map = deletion(&mv);
    let without = deletion(&bare);
    assert_eq!(with_map["violations"], without["violations"], "the mapping moved the verdict");
    let codes: Vec<String> = with_map["violations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| str_of(&v["code"]))
        .collect();
    assert_eq!(codes, vec!["C-001"], "an unauthored deletion is refused: {with_map}");

    // A malformed declaration is refused at compile (V-040), and lint's tree
    // checks stay inside the repository (111 D-12).
    let bad = scratch.join("moves-bad");
    fs::create_dir_all(&bad).unwrap();
    spec(
        &bad,
        "001-a",
        "status: draft\nestablishes:\n  - \"src/a.rs\"\nmoves:\n  - { from: \"../outside.rs\", to: null, kind: removed }\n",
        "# 001\n",
    );
    fs::write(scratch.join("outside.rs"), "pub fn o() {}\n").unwrap();
    let out: Value = serde_json::from_str(&compile_json("{}", s(&bad)).unwrap()).unwrap();
    assert_eq!(out["validation"]["passed"], false);
    assert!(out["validation"]["violations"].to_string().contains("V-040"), "{}", out["validation"]);
    let lint: Value = serde_json::from_str(&lint_json("{}", s(&bad)).unwrap()).unwrap();
    assert!(!lint.to_string().contains("L-016"), "lint stat a path outside the tree: {lint}");

    // V-041: a dangling `answered_by` is a compile warning, and L-016: a path
    // declared removed that is still in the tree is a lint warning.
    let dangling = scratch.join("moves-dangling");
    fs::create_dir_all(dangling.join("src")).unwrap();
    fs::write(dangling.join("src/old.rs"), "pub fn o() {}\n").unwrap();
    spec(
        &dangling,
        "001-a",
        "status: draft\nestablishes:\n  - \"src/old.rs\"\nmoves:\n  - { from: \"src/old.rs\", to: null, kind: removed, answered_by: \"009\" }\n",
        "# 001\n",
    );
    let out: Value = serde_json::from_str(&compile_json("{}", s(&dangling)).unwrap()).unwrap();
    assert_eq!(out["validation"]["passed"], true, "V-041 is a warning: {}", out["validation"]);
    assert!(out["validation"].to_string().contains("V-041"), "{}", out["validation"]);
    let lint: Value = serde_json::from_str(&lint_json("{}", s(&dangling)).unwrap()).unwrap();
    assert!(lint.to_string().contains("L-016"), "no L-016 for a present removed path: {lint}");
    println!("ok: 111 unmapped / resolved chain / split / ambiguous / cycle; the map is sorted; CLI == facade; a mapping clears no deletion; V-040 refuses an escaping path; V-041, L-015, L-016 warn");
}

fn contract_112(bin: &Path, scratch: &Path) {

    // ---- 112: a declared overlay is transported, scoped to its spec, and
    // read by no verdict.
    let ov = scratch.join("overlay");
    write_overlay(&ov, true);
    let cfg = json!({ "frontmatter": { "extra_known_keys": ["effects"] } }).to_string();
    let with: Value = serde_json::from_str(&compile_json(&cfg, s(&ov)).unwrap()).unwrap();
    assert_eq!(with["validation"]["passed"], true, "{}", with["validation"]);
    let rec = record(&with, "001-a");
    assert_eq!(
        rec["extraFrontmatter"]["effects"],
        json!({ "src/a.rs": { "network": false, "reads": ["the index"] } }),
        "transported as JSON, keys sorted"
    );
    let undeclared: Value = serde_json::from_str(&compile_json("{}", s(&ov)).unwrap()).unwrap();
    assert_eq!(undeclared["validation"]["passed"], false, "an undeclared nested overlay");
    assert!(undeclared["validation"]["violations"].to_string().contains("V-002"));
    let plain = scratch.join("overlay-absent");
    write_overlay(&plain, false);
    let without: Value = serde_json::from_str(&compile_json(&cfg, s(&plain)).unwrap()).unwrap();
    assert_eq!(record(&without, "002-b"), record(&with, "002-b"), "the other spec's record");
    // The hash scope is the declaring spec: the committed ledger's content
    // hash moves for 001-a and for nothing else.
    for root in [&ov, &plain] {
        fs::write(root.join("spec-spine.toml"), "[frontmatter]\nextra_known_keys = [\"effects\"]\n").unwrap();
        cli(bin, root, &["compile"]);
    }
    let hash = |root: &Path, id: &str| -> Value {
        let req = json!({ "specs": [id] }).to_string();
        let c: Value = serde_json::from_str(&closure_json(&cfg, s(root), &req).unwrap()).unwrap();
        c["members"][0]["contentHash"].clone()
    };
    assert!(hash(&ov, "002").is_string());
    assert_eq!(hash(&ov, "002"), hash(&plain, "002"), "the hash scope is the declaring spec");
    assert_ne!(hash(&ov, "001"), hash(&plain, "001"));
    assert_eq!(
        verdicts(&cfg, &ov),
        verdicts(&cfg, &plain),
        "no verdict reads an overlay"
    );
    println!("ok: 112 overlay transported, refused when undeclared, hashed only in its spec, read by no verdict");
}

fn contract_114(scratch: &Path) {

    // ---- 114: intent is a well-formed standing declaration, optional, and
    // an authorization of nothing.
    let it = scratch.join("intent");
    write_intent(&it, "intent:\n  goal: \"a correct removal stops being refused\"\n  non_goals:\n    - \"changing C-001\"\n");
    let valid: Value = serde_json::from_str(&compile_json("{}", s(&it)).unwrap()).unwrap();
    assert_eq!(valid["validation"]["passed"], true, "{}", valid["validation"]);
    assert_eq!(
        record(&valid, "001-a")["intent"],
        json!({ "goal": "a correct removal stops being refused", "nonGoals": ["changing C-001"] })
    );
    let absent = scratch.join("intent-absent");
    write_intent(&absent, "");
    let none: Value = serde_json::from_str(&compile_json("{}", s(&absent)).unwrap()).unwrap();
    assert!(record(&none, "001-a").get("intent").is_none(), "absent is no member");
    assert_eq!(verdicts("{}", &it), verdicts("{}", &absent), "a valid intent moves no verdict");
    for bad in [
        "intent:\n  non_goals: [\"x\"]\n",
        "intent:\n  goal: \"   \"\n",
        "intent:\n  goal: \"g\"\n  approach: \"how\"\n",
        "intent: \"just a string\"\n",
    ] {
        let dir = scratch.join("intent-bad");
        let _ = fs::remove_dir_all(&dir);
        write_intent(&dir, bad);
        let out: Value = serde_json::from_str(&compile_json("{}", s(&dir)).unwrap()).unwrap();
        assert_eq!(out["validation"]["passed"], false, "accepted a malformed intent: {bad}");
    }
    println!("ok: 114 valid intent recorded as {{goal, nonGoals}}; absence is no member; four malformed shapes refused; no verdict moves");
}

fn contract_116(scratch: &Path) {

    // ---- 116: a deferred spec claiming nothing raises no L-001, and the
    // exemption re-arms when it is scheduled.
    let lint_codes = |implementation: &str| -> Vec<String> {
        let dir = scratch.join(format!("deferred-{implementation}"));
        spec(&dir, "001-a", "status: draft\n", "# 001\n");
        let text = fs::read_to_string(dir.join("specs/001-a/spec.md"))
            .unwrap()
            .replace("implementation: pending", &format!("implementation: {implementation}"));
        fs::write(dir.join("specs/001-a/spec.md"), text).unwrap();
        let out: Value = serde_json::from_str(&lint_json("{}", s(&dir)).unwrap()).unwrap();
        out.as_array()
            .unwrap()
            .iter()
            .map(|v| str_of(&v["code"]))
            .collect()
    };
    assert!(!lint_codes("deferred").contains(&"L-001".to_string()), "deferred is exempt");
    assert!(lint_codes("pending").contains(&"L-001".to_string()), "scheduled re-arms L-001");
    assert!(lint_codes("n-a").contains(&"L-001".to_string()), "n-a is deliberately not exempt");
    println!("ok: 116 L-001 exempts a deferred spec only, and re-arms when scheduled");
}

/// Two corpora differing only in `moves`: 001-a owns src/a.rs; with `map`,
/// 001-a and 002-b declare a chain, a split, a disagreement and a loop.
fn write_moves(root: &Path, map: bool) {
    fs::create_dir_all(root.join("src")).unwrap();
    for f in ["a", "a3", "b", "c", "s1", "s2"] {
        fs::write(root.join(format!("src/{f}.rs")), "pub fn f() {}\n").unwrap();
    }
    let (a, b) = if map {
        (
            "moves:\n  - { from: \"src/a.rs\", to: \"src/a2.rs\", kind: relocated }\n  - { from: \"src/s.rs\", to: [\"src/s1.rs\", \"src/s2.rs\"], kind: split }\n  - { from: \"src/x.rs\", to: \"src/y.rs\", kind: relocated }\n  - { from: \"src/p.rs\", to: \"src/q.rs\", kind: relocated }\n",
            "moves:\n  - { from: \"src/a2.rs\", to: \"src/a3.rs\", kind: relocated }\n  - { from: \"src/x.rs\", to: \"src/z.rs\", kind: relocated }\n  - { from: \"src/q.rs\", to: \"src/p.rs\", kind: relocated }\n  - { from: \"src/old.rs\", to: null, kind: removed, answered_by: \"001\" }\n",
        )
    } else {
        ("", "")
    };
    spec(root, "001-a", &format!("status: draft\nestablishes:\n  - \"src/a.rs\"\n{a}"), "# 001\n");
    spec(root, "002-b", &format!("status: draft\nestablishes:\n  - \"src/b.rs\"\n{b}"), "# 002\n");
}

fn write_overlay(root: &Path, overlay: bool) {
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/a.rs"), "pub fn a() {}\n").unwrap();
    fs::write(root.join("src/b.rs"), "pub fn b() {}\n").unwrap();
    let o = if overlay {
        "effects:\n  \"src/a.rs\":\n    reads: [\"the index\"]\n    network: false\n"
    } else {
        ""
    };
    spec(root, "001-a", &format!("status: draft\nestablishes:\n  - \"src/a.rs\"\n{o}"), "# 001\n");
    spec(root, "002-b", "status: draft\nestablishes:\n  - \"src/b.rs\"\n", "# 002\n");
}

fn write_intent(root: &Path, intent: &str) {
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/a.rs"), "pub fn a() {}\n").unwrap();
    spec(root, "001-a", &format!("status: draft\nestablishes:\n  - \"src/a.rs\"\n{intent}"), "# 001\n");
}

/// One spec's record from a `compile_json` answer.
fn record<'a>(compiled: &'a Value, id: &str) -> &'a Value {
    compiled["specs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == id)
        .unwrap_or_else(|| panic!("no record for {id}"))
}

/// The verdicts a gate reads, minus the one thing that legitimately differs
/// between two corpora (their content hashes): compile's validation and
/// lint's violations (`lint_json` answers with the array itself).
fn verdicts(cfg: &str, root: &Path) -> (Value, Value) {
    let c: Value = serde_json::from_str(&compile_json(cfg, s(root)).unwrap()).unwrap();
    let l: Value = serde_json::from_str(&lint_json(cfg, s(root)).unwrap()).unwrap();
    assert!(c["validation"].is_object() && l.is_array(), "{c} {l}");
    (c["validation"].clone(), l)
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

fn cli_json(bin: &Path, root: &Path, args: &[&str], code: i32) -> Value {
    let out = Command::new(bin).arg("--repo").arg(root).args(args).output().unwrap();
    assert_eq!(out.status.code(), Some(code), "spec-spine {args:?} in {}", root.display());
    serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!("spec-spine {args:?}: stdout is not JSON ({e}): {}", String::from_utf8_lossy(&out.stdout))
    })
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
