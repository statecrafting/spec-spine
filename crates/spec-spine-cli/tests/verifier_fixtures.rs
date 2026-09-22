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

use std::collections::{BTreeMap, BTreeSet};
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

/// Copy a case's corpus into `root` and compile it, exactly as a consumer
/// reproducing the case would: the recompute reads the registry's inputs.
fn compiled_corpus(case_dir: &Path, root: &Path) {
    copy_tree(&case_dir.join("corpus"), root);
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

/// What this build's producer emits for a case's corpus, byte for byte.
fn attest_now(case_dir: &Path) -> Vec<u8> {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    compiled_corpus(case_dir, root);
    let out = Command::new(env!("CARGO_BIN_EXE_spec-spine"))
        .arg("--repo")
        .arg(root)
        .arg("attest")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "attesting {}'s corpus: {}",
        case_dir.display(),
        String::from_utf8_lossy(&out.stderr)
    );
    std::fs::read(root.join(".derived/attestation/attestation.json"))
        .expect("attest wrote .derived/attestation/attestation.json")
}

/// The committed bytes with the one `tool.version` literal the set was
/// generated under replaced by this build's (§3.7, D-12). Exactly one
/// occurrence, or the substitution would be a guess.
fn at_version(bytes: &[u8], from: &str, to: &str, ctx: &str) -> Vec<u8> {
    let text = std::str::from_utf8(bytes).unwrap_or_else(|e| panic!("{ctx}: not UTF-8: {e}"));
    // Structural, not only textual: the literal carried is `tool.version`'s.
    let parsed: serde_json::Value =
        serde_json::from_str(text).unwrap_or_else(|e| panic!("{ctx}: carried bytes parse: {e}"));
    assert_eq!(
        parsed["tool"]["version"], from,
        "{ctx}: the version literal to carry is not tool.version"
    );
    let needle = format!("\"version\": \"{from}\"");
    let n = text.matches(&needle).count();
    assert_eq!(
        n, 1,
        "{ctx}: expected exactly one `{needle}` to carry to {to}, found {n}"
    );
    text.replacen(&needle, &format!("\"version\": \"{to}\""), 1)
        .into_bytes()
}

/// Run a payload through the shipped verifier, in a scratch copy of the
/// case's own corpus so nothing reaches this repository's tree. `payload`
/// overrides the committed bytes (the version-carried form, D-12).
fn verify(case_dir: &Path, needs_corpus: bool, payload: Option<&[u8]>) -> (i32, serde_json::Value) {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    if needs_corpus {
        compiled_corpus(case_dir, root);
    }
    let payload_path = match payload {
        Some(bytes) => {
            let path = tmp.path().join("carried-payload.json");
            std::fs::write(&path, bytes).unwrap();
            path
        }
        None => case_dir.join("payload.json"),
    };
    let out = Command::new(env!("CARGO_BIN_EXE_spec-spine"))
        .arg("--repo")
        .arg(root)
        .args([
            "verify-attestation",
            "--recompute",
            "--json",
            "--attestation",
        ])
        .arg(&payload_path)
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
    let payload_types: BTreeSet<String> = index["payloadTypes"]
        .as_array()
        .expect("index.payloadTypes")
        .iter()
        .map(|t| t.as_str().expect("payload type").to_string())
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

    // ---- §3.7 / D-12: the set's version, and whether it is still current ----
    // The control is verbatim producer output, so its `toolVersion` is the
    // build the whole set was generated under.
    let control_dir = dir.join("control-untampered");
    let control = read_json(&control_dir.join("case.json"));
    let set_version = str_at(&control, "toolVersion", "case control-untampered").to_string();
    // The set is current when this build's producer emits the control's
    // bytes exactly, once the one version literal is carried across. A
    // difference anywhere else is content the set no longer describes.
    let committed_control = std::fs::read(control_dir.join("payload.json")).unwrap();
    let expected_now = if set_version == build_version {
        committed_control.clone()
    } else {
        at_version(
            &committed_control,
            &set_version,
            &build_version,
            "control-untampered",
        )
    };
    let produced_now = attest_now(&control_dir);
    assert!(
        produced_now == expected_now,
        "STALE FIXTURES: the producer at {build_version} no longer emits the control's \
         committed bytes (generated at {set_version}) apart from the version literal. \
         Regenerate the set: cargo build --release --locked -p spec-spine-cli && \
         python3 crates/spec-spine-core/fixtures/verifier/generate.py (spec 103 §3.9, D-12). \
         Produced now:\n{}\nCommitted, carried to {build_version}:\n{}",
        String::from_utf8_lossy(&produced_now),
        String::from_utf8_lossy(&expected_now)
    );

    // Every outcome the shipped verifier actually produced, per case. §3.8.5
    // and §3.8.6 are judged over this map, never over what a case.json
    // expected or how many ids the index listed.
    let mut observed_by_case: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut record = |id: &str, reason: &str| {
        observed_by_case
            .entry(id.to_string())
            .or_default()
            .push(reason.to_string());
    };

    for id in &listed {
        let case_dir = dir.join(id);
        let case = read_json(&case_dir.join("case.json"));
        let ctx = format!("case {id}");
        assert_eq!(
            str_at(&case, "id", &ctx),
            id,
            "{ctx}: id disagrees with its directory"
        );

        // §3.4: the payload type is a name from the index's closed vocabulary.
        let payload_type = str_at(&case, "payloadType", &ctx);
        assert!(
            payload_types.contains(payload_type),
            "{ctx}: payloadType {payload_type:?} is not in the index's payloadTypes"
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

        let case_tool = str_at(&case, "toolVersion", &ctx).to_string();
        // A case attested under a version other than the set's is a deliberate
        // version case, and the only outcome it may declare is version-mismatch.
        if case_tool != set_version {
            // A version refusal happens at the recompute, so a deliberate version
            // case needs its corpus; without one it could never observe it.
            assert!(
                needs_corpus,
                "{ctx}: attested under {case_tool}, not the set's {set_version}, so it \
                 is a version case and must carry its corpus (needsCorpus: true)"
            );
            assert_eq!(
                want_reason,
                Some("version-mismatch"),
                "{ctx}: attested under {case_tool}, not the set's {set_version}, so it \
                 must declare version-mismatch"
            );
        }

        // §3.7's first branch: the case's own expectation, observed. A
        // recompute case generated under the set's version is carried to this
        // build (D-12), so every declared outcome is observed at every version.
        let first_branch_payload =
            (needs_corpus && case_tool == set_version && set_version != build_version).then(|| {
                at_version(
                    &std::fs::read(case_dir.join("payload.json")).unwrap(),
                    &set_version,
                    &build_version,
                    &ctx,
                )
            });
        let deliberate_version_case = needs_corpus && case_tool != set_version;
        if deliberate_version_case {
            // The version-mismatch case keeps its committed version: that
            // difference is the case.
            let (code, json) = verify(&case_dir, needs_corpus, None);
            let observed = observed_reason(&json);
            assert_eq!(observed, "version-mismatch", "{ctx}: {json}");
            assert_eq!(
                json["report"]["expected"],
                case_tool.as_str(),
                "{ctx}: {json}"
            );
            assert_eq!(
                json["report"]["actual"],
                build_version.as_str(),
                "{ctx}: {json}"
            );
            assert_eq!(code, want_exit, "{ctx}: exit code. {json}");
            record(id, &observed);
        } else {
            let (code, json) = verify(&case_dir, needs_corpus, first_branch_payload.as_deref());
            let observed = observed_reason(&json);
            let want = want_reason.unwrap_or("accepted");
            assert_eq!(
                observed, want,
                "{ctx}: expected {want}, observed {observed}. {json}"
            );
            assert_eq!(code, want_exit, "{ctx}: exit code. {json}");
            record(id, &observed);
        }

        // §3.7's second branch: the committed bytes, attested under another
        // build, are a named version outcome, never a content mismatch and
        // never a skip.
        if first_branch_payload.is_some() {
            let (code, json) = verify(&case_dir, needs_corpus, None);
            let observed = observed_reason(&json);
            assert_eq!(
                observed, "version-mismatch",
                "{ctx}: attested under {set_version}, verified by {build_version}; \
                 FR-005 requires a named version outcome, never a false content \
                 mismatch and never a skip-as-pass. Got {json}"
            );
            assert_eq!(
                json["report"]["expected"],
                set_version.as_str(),
                "{ctx}: {json}"
            );
            assert_eq!(
                json["report"]["actual"],
                build_version.as_str(),
                "{ctx}: {json}"
            );
            assert_eq!(code, 1, "{ctx}: {json}");
            record(id, &observed);
        }
    }

    // ---- §3.8.6: every listed case produced a verifier outcome -------------
    let observed_ids: BTreeSet<String> = observed_by_case.keys().cloned().collect();
    assert_eq!(
        observed_ids, listed_set,
        "cases with no observed verifier outcome, or outcomes for unlisted cases"
    );

    // ---- §3.8.5: every declared reason was produced by the shipped verifier -
    let reasons_seen: BTreeSet<String> = observed_by_case.values().flatten().cloned().collect();
    let unexercised: Vec<&String> = closed_reasons
        .iter()
        .filter(|r| !reasons_seen.contains(*r))
        .collect();
    assert!(
        unexercised.is_empty(),
        "the shipped verifier ({build_version}) produced no case with these declared \
         reasons: {unexercised:?}. A closed set with an unexercised member is a \
         vocabulary nobody tested."
    );
    assert!(
        reasons_seen.contains("accepted"),
        "no case was observed as accepted under {build_version}: the control did \
         not run on §3.7's first branch, so the suite of refusals asserts nothing \
         about what a valid payload does."
    );
}
