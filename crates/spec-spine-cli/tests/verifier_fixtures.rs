//! Spec 103: the verifier fixture set, executed against the shipped verifier.
//!
//! The fixtures under `crates/spec-spine-core/fixtures/verifier/` are this
//! repository's written claim about how its verifier answers a tampered or
//! cross-version payload, offered to the three consumers note 04 §7 names. A
//! committed claim about behavior that nothing executes is a comment, so this
//! walks the set and runs `spec-spine verify-attestation --recompute --json`
//! over every case.
//!
//! It is a CLI test rather than a core one deliberately (spec 103 §2, D-5):
//! the decision sequence the fixtures record (the loose `schemaVersion` read,
//! the MAJOR gate, the strict parse, the recompute, the byte comparison, and
//! the exit code each produces) is assembled in the CLI. Reassembling it from
//! library calls here would assert a reimplementation.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("spec-spine-core/fixtures/verifier")
}

fn read_json(path: &Path) -> serde_json::Value {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_slice(&bytes).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

fn str_at<'a>(v: &'a serde_json::Value, key: &str, ctx: &str) -> &'a str {
    v.get(key)
        .and_then(|x| x.as_str())
        .unwrap_or_else(|| panic!("{ctx}: missing string member `{key}`"))
}

/// The version this binary was built at, which is what `verify_recompute`
/// compares a payload's `tool.version` against (spec 021 FR-005).
fn tool_version() -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_spec-spine"))
        .arg("--version")
        .output()
        .expect("spawn spec-spine --version");
    String::from_utf8_lossy(&out.stdout)
        .split_whitespace()
        .nth(1)
        .expect("version token")
        .to_string()
}

/// Run one case's payload through the shipped verifier, in a scratch copy of
/// the case's own corpus so nothing reaches this repository's tree.
fn verify(case_dir: &Path, needs_corpus: bool) -> (i32, serde_json::Value) {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    if needs_corpus {
        copy_tree(&case_dir.join("corpus"), root);
        // The recompute reads the committed registry's inputs, so the corpus
        // is compiled here exactly as a consumer reproducing the case would.
        let out = Command::new(env!("CARGO_BIN_EXE_spec-spine"))
            .arg("--repo")
            .arg(root)
            .arg("compile")
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "compiling {}'s corpus: {}",
            case_dir.display(),
            String::from_utf8_lossy(&out.stderr)
        );
    }
    let out = Command::new(env!("CARGO_BIN_EXE_spec-spine"))
        .arg("--repo")
        .arg(root)
        .args([
            "verify-attestation",
            "--recompute",
            "--json",
            "--attestation",
        ])
        .arg(case_dir.join("payload.json"))
        .output()
        .unwrap();
    let code = out.status.code().unwrap_or(-1);
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!(
            "{}: verdict envelope did not parse ({e}); stdout: {} stderr: {}",
            case_dir.display(),
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        )
    });
    (code, json)
}

fn copy_tree(src: &Path, dst: &Path) {
    for entry in std::fs::read_dir(src).unwrap_or_else(|e| panic!("read {}: {e}", src.display())) {
        let entry = entry.unwrap();
        let to = dst.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            std::fs::create_dir_all(&to).unwrap();
            copy_tree(&entry.path(), &to);
        } else {
            std::fs::copy(entry.path(), &to).unwrap();
        }
    }
}

/// What the envelope actually said, reduced to this set's reason vocabulary.
///
/// The mapping is the fixtures' contract with a consumer: it is how an
/// external verifier learns which of its own code paths a case is about.
fn observed_reason(json: &serde_json::Value) -> String {
    if let Some(err) = json.get("error") {
        let kind = err.get("kind").and_then(|k| k.as_str()).unwrap_or("");
        let msg = err.get("message").and_then(|m| m.as_str()).unwrap_or("");
        return match kind {
            "schema" if msg.contains("no string schemaVersion") => "missing-schema-version",
            "schema" => "unsupported-major",
            "parse" if msg.contains("duplicate field") => "duplicate-key",
            "parse" if msg.contains("unknown field") => "unknown-member",
            "parse" => "unreadable-json",
            other => other,
        }
        .to_string();
    }
    let report = json.get("report").expect("report or error");
    match report.get("outcome").and_then(|o| o.as_str()) {
        Some("match") => "accepted".to_string(),
        Some("versionMismatch") => "version-mismatch".to_string(),
        Some("contentMismatch") => {
            let non_canonical = report
                .get("differences")
                .and_then(|d| d.as_array())
                .is_some_and(|d| {
                    d.len() == 1 && d[0] == "bytes are not the canonical serialization"
                });
            if non_canonical {
                "non-canonical-bytes".to_string()
            } else {
                "content-mismatch".to_string()
            }
        }
        other => panic!("unexpected outcome {other:?} in {json}"),
    }
}

#[test]
fn the_fixture_set_describes_the_shipped_verifier() {
    let dir = fixtures_dir();
    let index = read_json(&dir.join("index.json"));

    // ---- §3.8.1: the index is understood -----------------------------------
    let index_version = str_at(&index, "schemaVersion", "index.json");
    assert!(
        index_version.starts_with("0."),
        "index schemaVersion MAJOR {index_version} is not one this test understands"
    );

    let listed: Vec<String> = index["cases"]
        .as_array()
        .expect("index.cases")
        .iter()
        .map(|c| c.as_str().expect("case id").to_string())
        .collect();
    let closed_reasons: BTreeSet<String> = index["reasons"]
        .as_array()
        .expect("index.reasons")
        .iter()
        .map(|r| r.as_str().expect("reason").to_string())
        .collect();

    // ---- §3.8.2: non-empty -------------------------------------------------
    assert!(
        !listed.is_empty(),
        "the fixture index lists no cases; an empty walk asserts nothing"
    );

    // ---- §3.8.3: the listing and the disk are the same set -----------------
    let on_disk: BTreeSet<String> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| {
            let e = e.unwrap();
            e.file_type()
                .unwrap()
                .is_dir()
                .then(|| e.file_name().to_string_lossy().to_string())
        })
        .collect();
    let listed_set: BTreeSet<String> = listed.iter().cloned().collect();
    assert_eq!(
        listed_set, on_disk,
        "the index and the fixture directories disagree; a case added without \
         being listed would otherwise be silently skipped"
    );

    // ---- §3.8.4: the control is present and is producer output -------------
    assert!(
        listed_set.contains("control-untampered"),
        "the control case is missing; a suite of refusals passes for a verifier \
         that refuses everything"
    );

    let build_version = tool_version();
    let mut executed = 0usize;
    let mut reasons_seen: BTreeSet<String> = BTreeSet::new();

    for id in &listed {
        let case_dir = dir.join(id);
        let case = read_json(&case_dir.join("case.json"));
        let ctx = format!("case {id}");
        assert_eq!(
            str_at(&case, "id", &ctx),
            id,
            "{ctx}: id disagrees with its directory"
        );

        // §3.2: the byte-provenance vocabulary, and what each kind obliges.
        let kind = str_at(&case, "bytes", &ctx);
        assert!(
            matches!(kind, "producer" | "mutated" | "authored"),
            "{ctx}: unknown bytes kind {kind:?}"
        );
        if kind == "mutated" {
            assert!(
                case.get("derivedFrom").is_some() && case.get("mutation").is_some(),
                "{ctx}: a mutated payload must record what it derives from and the change applied"
            );
        } else {
            assert!(
                case.get("derivedFrom").is_none() && case.get("mutation").is_none(),
                "{ctx}: only a mutated payload carries derivedFrom/mutation"
            );
        }
        if id == "control-untampered" {
            assert_eq!(
                kind, "producer",
                "the control must be verbatim producer output, or it proves nothing \
                 about what the producer emits"
            );
        }

        // §3.4: the subject is a digest or an explicit none, never a local path.
        let subject = case
            .get("subject")
            .unwrap_or_else(|| panic!("{ctx}: no subject"));
        let skind = str_at(subject, "kind", &ctx);
        assert!(
            matches!(skind, "corpus" | "spec" | "none"),
            "{ctx}: unknown subject kind {skind:?}"
        );
        if skind == "corpus" {
            assert!(
                str_at(subject, "attestationHash", &ctx).starts_with("sha256:"),
                "{ctx}: a corpus subject is identified by digest"
            );
        }

        let expect = case
            .get("expect")
            .unwrap_or_else(|| panic!("{ctx}: no expect"));
        let want_outcome = str_at(expect, "outcome", &ctx);
        let want_reason = expect.get("reason").and_then(|r| r.as_str());
        let want_exit = expect
            .get("exit")
            .and_then(|e| e.as_i64())
            .unwrap_or_else(|| panic!("{ctx}: no expect.exit")) as i32;

        // §3.4: the reason vocabulary is closed and declared in the index.
        if let Some(r) = want_reason {
            assert!(
                closed_reasons.contains(r),
                "{ctx}: reason {r:?} is not in the index's closed set"
            );
            reasons_seen.insert(r.to_string());
        }
        assert_eq!(
            want_outcome == "refused",
            want_reason.is_some(),
            "{ctx}: a refusal carries a reason and an acceptance does not"
        );

        let needs_corpus = case
            .get("needsCorpus")
            .and_then(|n| n.as_bool())
            .unwrap_or_else(|| panic!("{ctx}: no needsCorpus"));
        assert_eq!(
            needs_corpus,
            case_dir.join("corpus").is_dir(),
            "{ctx}: corpus/ is present exactly when needsCorpus is true (§3.3)"
        );

        let (code, json) = verify(&case_dir, needs_corpus);
        let observed = observed_reason(&json);
        executed += 1;

        // §3.7: the version rule. Both branches assert; neither skips.
        let case_tool = str_at(&case, "toolVersion", &ctx);
        let version_bound = needs_corpus && case_tool != build_version;
        if version_bound {
            assert_eq!(
                observed, "version-mismatch",
                "{ctx}: attested under {case_tool}, verified by {build_version}; \
                 FR-005 requires a named version outcome, never a false content \
                 mismatch and never a skip-as-pass. Got {json}"
            );
            let report = &json["report"];
            assert_eq!(report["expected"], case_tool, "{ctx}: {json}");
            assert_eq!(report["actual"], build_version, "{ctx}: {json}");
            assert_eq!(code, 1, "{ctx}: {json}");
        } else {
            let want = want_reason.unwrap_or("accepted");
            assert_eq!(
                observed, want,
                "{ctx}: expected {want}, observed {observed}. {json}"
            );
            assert_eq!(code, want_exit, "{ctx}: exit code. {json}");
        }
    }

    // ---- §3.8.5: every declared reason is exercised ------------------------
    let unexercised: Vec<&String> = closed_reasons.difference(&reasons_seen).collect();
    assert!(
        unexercised.is_empty(),
        "the index declares reasons no case exercises: {unexercised:?}. A closed \
         set with an unreachable member is a vocabulary nobody tested."
    );

    // ---- §3.8.6: the loop actually ran ------------------------------------
    assert_eq!(
        executed,
        listed.len(),
        "executed {executed} of {} listed cases",
        listed.len()
    );
}
