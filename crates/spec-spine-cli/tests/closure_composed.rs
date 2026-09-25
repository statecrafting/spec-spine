// Spec: specs/107-a-context-closure-is-declared/spec.md
//! Spec 107 D-9: the expansion wave's contracts asserted together, through
//! the shipped binary and the library facade, over one disposable corpus.
//!
//! Each branch tested its own contract. These cases exist because the
//! contracts meet: a readiness entry's `status` (spec 102) and a closure's
//! digest both read `spec.md`; an obligation (spec 106) is answered by three
//! surfaces that must agree; and a read that tolerates missing or stale data
//! (`registry obligation`, like `show`) sits beside a resolver that must not.

use std::fs;
use std::path::Path;
use std::process::{Command, Output, Stdio};

use serde_json::Value;

fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_spec-spine"))
        .arg("--repo")
        .arg(root)
        .args(args)
        .stdin(Stdio::null())
        .output()
        .expect("spawn spec-spine")
}

fn ok_json(root: &Path, args: &[&str]) -> Value {
    let out = run(root, args);
    assert_eq!(out.status.code(), Some(0), "{args:?}: {}", text(&out));
    serde_json::from_slice(&out.stdout).unwrap()
}

fn text(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

const A_HEAD: &str = "---\nid: \"001-a\"\ntitle: \"A\"\nstatus: STATUS\nimplementation: pending\ncreated: \"2026-09-22\"\nsummary: \"s\"\n";
const A_OBLIGATIONS: &str = "obligations:\n  - id: \"R-1\"\n    kind: requirement\n    text: \"The rule holds.\"\n    anchor: \"3-1-the-rule\"\n  - id: \"V-1\"\n    kind: verification\n    text: \"It is checked.\"\n    anchor: \"3-2-the-check\"\n    inputs: [\"tests/a.rs\"]\n  - id: \"R-2\"\n    kind: requirement\n    text: \"The old rule.\"\n    anchor: \"3-1-the-rule\"\n    withdrawn: true\n";
const A_BODY: &str =
    "---\n# 001\n\n## 3. Behavior\n\n### 3.1 The rule\n\nRULE\n\n### 3.2 The check\n\nCHECK\n";

struct Corpus {
    tmp: tempfile::TempDir,
    status: &'static str,
    obligations: String,
    rule: &'static str,
    check: &'static str,
}

impl Corpus {
    fn new() -> Self {
        let c = Corpus {
            tmp: tempfile::tempdir().unwrap(),
            status: "draft",
            obligations: A_OBLIGATIONS.to_string(),
            rule: "The rule text.",
            check: "The check text.",
        };
        for (id, status, deps) in [
            ("002-b", "approved", "depends_on: [\"001-a\"]\n"),
            ("003-c", "approved", ""),
        ] {
            let dir = c.root().join("specs").join(id);
            fs::create_dir_all(&dir).unwrap();
            fs::write(
                dir.join("spec.md"),
                format!(
                    "---\nid: \"{id}\"\ntitle: \"{id}\"\nstatus: {status}\nimplementation: pending\ncreated: \"2026-09-22\"\nsummary: \"s\"\n{deps}---\n# {id}\n\n## 1. Purpose\n\nP.\n"
                ),
            )
            .unwrap();
        }
        c.write_a();
        c.compile();
        c
    }

    fn root(&self) -> &Path {
        self.tmp.path()
    }

    fn write_a(&self) {
        let dir = self.root().join("specs/001-a");
        fs::create_dir_all(&dir).unwrap();
        let body = A_BODY
            .replace("RULE", self.rule)
            .replace("CHECK", self.check);
        fs::write(
            dir.join("spec.md"),
            format!(
                "{}{}{}",
                A_HEAD.replace("STATUS", self.status),
                self.obligations,
                body
            ),
        )
        .unwrap();
    }

    fn compile(&self) {
        let out = run(self.root(), &["compile"]);
        assert_eq!(out.status.code(), Some(0), "compile: {}", text(&out));
    }

    fn closure(&self, request: &str) -> Output {
        let path = self.root().join("request.json");
        fs::write(&path, request).unwrap();
        run(
            self.root(),
            &[
                "registry",
                "closure",
                "--request",
                path.to_str().unwrap(),
                "--json",
            ],
        )
    }

    fn digest(&self, request: &str) -> String {
        let out = self.closure(request);
        assert_eq!(out.status.code(), Some(0), "{request}: {}", text(&out));
        let doc: Value = serde_json::from_slice(&out.stdout).unwrap();
        doc["digest"].as_str().unwrap().to_string()
    }
}

const WHOLE_SPEC: &str = r#"{"specs":["001"]}"#;
const PARTS: &str =
    r#"{"sections":[{"spec":"001","anchor":"3-2-the-check"}],"obligations":["001#R-1"]}"#;

/// Spec 102 with 106 and 107: ratifying a spec leaves the ready set's
/// membership and order alone and changes only the reported `status`; a
/// closure that names the whole spec moves, because the flip is an edit to
/// `spec.md`, and a closure that names only its sections and obligations does
/// not. Approval is visible, never consulted, and identity is exactly as
/// coarse as what was named.
#[test]
fn ratification_moves_status_and_whole_spec_identity_and_nothing_else() {
    let mut c = Corpus::new();
    let plan_before = ok_json(c.root(), &["registry", "plan", "--json"]);
    let whole_before = c.digest(WHOLE_SPEC);
    let parts_before = c.digest(PARTS);

    c.status = "approved";
    c.write_a();
    c.compile();
    let plan_after = ok_json(c.root(), &["registry", "plan", "--json"]);

    let ids = |p: &Value| -> Vec<String> {
        p["ready"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e["id"].as_str().unwrap().to_string())
            .collect()
    };
    let status_of = |p: &Value, id: &str| -> String {
        p["ready"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["id"] == id)
            .map(|e| e["status"].as_str().unwrap().to_string())
            .unwrap()
    };
    assert_eq!(
        ids(&plan_before),
        ids(&plan_after),
        "membership or order moved"
    );
    assert!(ids(&plan_before).contains(&"001-a".to_string()));
    assert_eq!(status_of(&plan_before, "001-a"), "draft");
    assert_eq!(status_of(&plan_after, "001-a"), "approved");
    assert_eq!(plan_before["blocked"], plan_after["blocked"]);

    assert_ne!(
        whole_before,
        c.digest(WHOLE_SPEC),
        "a whole-spec closure ignored an edit"
    );
    assert_eq!(
        parts_before,
        c.digest(PARTS),
        "a closure moved on a member it did not name"
    );
}

/// Spec 106 through three surfaces: the CLI read, the `query_json` facade and
/// a resolved closure member say the same thing about one obligation.
#[test]
fn an_obligation_reads_the_same_through_every_supported_surface() {
    let c = Corpus::new();
    let cli = ok_json(c.root(), &["registry", "obligation", "001#V-1", "--json"]);

    let cfg = spec_spine_types::Config::default();
    let registry = spec_spine_core::load_committed_registry(&cfg, c.root()).unwrap();
    let request = serde_json::json!({
        "registry": serde_json::to_string(&registry).unwrap(),
        "op": "obligation",
        "id": "001-a#V-1",
    });
    let facade: Value =
        serde_json::from_str(&spec_spine_core::query_json(&request.to_string()).unwrap()).unwrap();

    let closure: Value =
        serde_json::from_slice(&c.closure(r#"{"obligations":["001#V-1"]}"#).stdout).unwrap();
    let member = &closure["members"][0];

    for (surface, ob, digest) in [
        ("cli", &cli["obligation"], &cli["sectionDigest"]),
        ("facade", &facade["obligation"], &facade["sectionDigest"]),
    ] {
        assert_eq!(ob["text"], member["text"], "{surface}");
        assert_eq!(ob["anchor"], member["anchor"], "{surface}");
        assert_eq!(ob["kind"], member["obligationKind"], "{surface}");
        assert_eq!(ob["inputs"], member["inputs"], "{surface}");
        assert_eq!(digest, &member["sectionDigest"], "{surface}");
    }
    assert_eq!(cli["spec"], "001-a");
    assert_eq!(facade["spec"], "001-a");
    assert_eq!(cli["schemaVersion"], facade["schemaVersion"]);
    assert_eq!(closure["schemaVersion"], cli["schemaVersion"]);
    // Only the CLI reads a committed shard, so only it can add the spec's
    // content hash (106 §3.6); the facade's registry text has none.
    assert!(cli["contentHash"].is_string());
    assert!(facade.get("contentHash").is_none());
}

/// Spec 106's tombstone with 107: a withdrawn obligation still resolves, as
/// withdrawn; declaring its id live again beside the tombstone is refused at
/// compile; and replacing the tombstone with a new text under the same id,
/// which compile cannot see without history, moves every closure that named
/// it, so a consumer holding the old digest detects the reuse.
#[test]
fn a_withdrawn_obligation_keeps_its_identity_and_a_reuse_is_detectable() {
    let mut c = Corpus::new();
    let out = c.closure(r#"{"obligations":["001#R-2"]}"#);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    let doc: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(doc["members"][0]["withdrawn"], true);
    let withdrawn_digest = doc["digest"].as_str().unwrap().to_string();

    // A live R-2 beside the tombstone.
    c.obligations = format!(
        "{A_OBLIGATIONS}  - id: \"R-2\"\n    kind: requirement\n    text: \"A new rule.\"\n    anchor: \"3-1-the-rule\"\n"
    );
    c.write_a();
    let out = run(c.root(), &["compile"]);
    assert_eq!(
        out.status.code(),
        Some(1),
        "a reused id compiled: {}",
        text(&out)
    );
    assert!(text(&out).contains("R-2"), "{}", text(&out));

    // The tombstone deleted and the id re-declared with a new text.
    c.obligations = A_OBLIGATIONS.replace(
        "    text: \"The old rule.\"\n    anchor: \"3-1-the-rule\"\n    withdrawn: true\n",
        "    text: \"A new rule.\"\n    anchor: \"3-1-the-rule\"\n",
    );
    assert_ne!(c.obligations, A_OBLIGATIONS);
    c.write_a();
    c.compile();
    assert_ne!(
        withdrawn_digest,
        c.digest(r#"{"obligations":["001#R-2"]}"#),
        "a reused obligation id kept the digest a consumer recorded"
    );
}

/// Spec 106's section digests through 107: editing a section's prose moves
/// the closures that name it or an obligation anchored in it, and no other.
#[test]
fn a_section_edit_moves_exactly_the_closures_that_depend_on_it() {
    let mut c = Corpus::new();
    let by_obligation = r#"{"obligations":["001#R-1"]}"#;
    let by_other_section = r#"{"sections":[{"spec":"001-a","anchor":"3-2-the-check"}]}"#;
    let before_ob = c.digest(by_obligation);
    let before_other = c.digest(by_other_section);
    let digest_before =
        ok_json(c.root(), &["registry", "obligation", "001#R-1", "--json"])["sectionDigest"]
            .clone();

    c.rule = "The rule text, amended.";
    c.write_a();
    c.compile();

    let digest_after =
        ok_json(c.root(), &["registry", "obligation", "001#R-1", "--json"])["sectionDigest"]
            .clone();
    assert_ne!(
        digest_before, digest_after,
        "the section digest did not move"
    );
    assert_ne!(before_ob, c.digest(by_obligation));
    assert_eq!(before_other, c.digest(by_other_section));
}

/// Spec 107 §3.4: order, repetition and short ids do not move the digest.
#[test]
fn a_closure_digest_is_normalized_over_order_duplicates_and_short_ids() {
    let c = Corpus::new();
    let a = c.digest(
        r#"{"specs":["001-a","003"],"obligations":["001#R-1","001-a#V-1"],"sections":[{"spec":"001","anchor":"3-1-the-rule"}]}"#,
    );
    let b = c.digest(
        r#"{"sections":[{"spec":"001-a","anchor":"3-1-the-rule"},{"spec":"001","anchor":"3-1-the-rule"}],"obligations":["001-a#V-1","001#R-1","001-a#R-1"],"specs":["003-c","001","001-a"],"rationale":"another"}"#,
    );
    assert_eq!(a, b);
}

/// The boundary between a tolerant read and the resolver. `registry
/// obligation` answers from whatever ledger is committed, stale or not (106
/// §3.6 keeps `show`'s degrade); a closure refuses a stale ledger, a missing
/// member and an obligation whose section digest is absent, so none of those
/// can enter a digest that looks complete.
#[test]
fn what_a_tolerant_read_accepts_cannot_enter_a_closure() {
    let mut c = Corpus::new();

    // A missing member is named, and nothing is digested.
    let out = c.closure(r#"{"specs":["001"],"obligations":["001#R-9"],"sections":[{"spec":"001","anchor":"no-such"}]}"#);
    assert_eq!(out.status.code(), Some(1), "{}", text(&out));
    let said = text(&out);
    assert!(
        said.contains("001-a#R-9") || said.contains("001#R-9"),
        "{said}"
    );
    assert!(said.contains("no-such"), "{said}");
    assert!(!said.contains("\"digest\""), "{said}");

    // A stale ledger: the inspection still answers, with the old text; the
    // closure refuses.
    c.rule = "Edited without recompiling.";
    c.write_a();
    let stale_read = run(c.root(), &["registry", "obligation", "001#R-1", "--json"]);
    assert_eq!(stale_read.status.code(), Some(0), "{}", text(&stale_read));
    let out = c.closure(r#"{"obligations":["001#R-1"]}"#);
    assert_eq!(
        out.status.code(),
        Some(1),
        "a stale ledger was digested: {}",
        text(&out)
    );
    c.compile();

    // An obligation whose section digest is absent from the committed shard,
    // edited by hand. The tolerant read still answers and says the digest is
    // missing (`null`); the closure refuses. Through the binary the refusal is
    // the freshness read's (exit 2), which byte-compares every shard before
    // anything is resolved; the resolver's own absent-digest guard, reachable
    // only through the library, is asserted in `tests/closure.rs`.
    let shard_path = c.root().join(".derived/spec-registry/by-spec/001-a.json");
    let mut shard: Value = serde_json::from_slice(&fs::read(&shard_path).unwrap()).unwrap();
    shard["record"]["sectionDigests"]
        .as_object_mut()
        .unwrap()
        .remove("3-1-the-rule")
        .expect("the fixture's digest exists");
    fs::write(&shard_path, serde_json::to_vec_pretty(&shard).unwrap()).unwrap();

    let read = ok_json(c.root(), &["registry", "obligation", "001#R-1", "--json"]);
    assert!(read["sectionDigest"].is_null(), "{read}");
    assert_eq!(read["obligation"]["text"], "The rule holds.");
    let out = c.closure(r#"{"obligations":["001#R-1"]}"#);
    assert_eq!(
        out.status.code(),
        Some(1),
        "an absent digest entered a closure: {}",
        text(&out)
    );
    assert!(!text(&out).contains("\"digest\""), "{}", text(&out));
}
