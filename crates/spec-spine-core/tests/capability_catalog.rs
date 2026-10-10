//! Spec 162: the capability catalog is canonical, validates against its
//! embedded schema, and is held to the facade, the verdict verbs, the managed
//! gate and the committed registry (§3.10 items 2, 3, 4 and 6). The clap
//! census lives in the binary crate (`main.rs::capability_census`), and the
//! examples and the effect audit in `spec-spine-cli/tests/capability_catalog.rs`.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use sha2::{Digest, Sha256};
use spec_spine_core::{
    capability_catalog, capability_catalog_json, capability_verify, capability_verify_json,
};
use spec_spine_types::{
    CAPABILITY_CATALOG_SCHEMA, CATALOG_SCHEMA_VERSION, CapabilityCatalog, CapabilityVerifyRequest,
    Error, Operation, error_kind, verdict,
};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn catalog() -> CapabilityCatalog {
    capability_catalog().expect("the catalog builds")
}

fn op<'a>(c: &'a CapabilityCatalog, name: &str) -> &'a Operation {
    c.operations
        .iter()
        .find(|o| o.name == name)
        .unwrap_or_else(|| panic!("no operation {name}"))
}

/// The digest construction, restated without the core's helpers: canonical
/// JSON (sorted keys, two-space pretty, trailing LF) with `member` removed.
fn independent_digest(value: &Value, member: &str) -> String {
    let mut v = value.clone();
    v.as_object_mut().unwrap().remove(member);
    let mut bytes = serde_json::to_string_pretty(&v).unwrap();
    bytes.push('\n');
    let digest = Sha256::digest(bytes.as_bytes());
    let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
    format!("sha256:{hex}")
}

#[test]
fn the_catalog_is_canonical_and_byte_identical_across_calls() {
    let a = capability_catalog_json().unwrap();
    let b = capability_catalog_json().unwrap();
    assert_eq!(a, b, "two calls differ");
    assert!(a.ends_with("}\n") && !a.ends_with("\n\n"));
    assert!(!a.contains('\r'));
    let v: Value = serde_json::from_str(&a).unwrap();
    let mut recanon = serde_json::to_string_pretty(&v).unwrap();
    recanon.push('\n');
    assert_eq!(a, recanon, "not canonical: keys unsorted or layout differs");
    assert_eq!(v["schemaVersion"], CATALOG_SCHEMA_VERSION);
    assert_eq!(v["tool"], "spec-spine");
    assert_eq!(v["toolVersion"], env!("CARGO_PKG_VERSION"));
}

#[test]
fn every_digest_matches_an_independent_construction() {
    let v: Value = serde_json::from_str(&capability_catalog_json().unwrap()).unwrap();
    assert_eq!(v["catalogDigest"], independent_digest(&v, "catalogDigest"));
    for o in v["operations"].as_array().unwrap() {
        assert_eq!(
            o["operationDigest"],
            independent_digest(o, "operationDigest"),
            "{}",
            o["name"]
        );
    }
}

#[test]
fn an_operation_digest_excludes_the_tool_version() {
    // 162 D-7: a pin survives an upgrade that leaves the description alone.
    let v: Value = serde_json::from_str(&capability_catalog_json().unwrap()).unwrap();
    for o in v["operations"].as_array().unwrap() {
        assert!(!o.to_string().contains("toolVersion"), "{}", o["name"]);
    }
}

#[test]
fn the_catalog_validates_against_its_embedded_schema() {
    let schema: Value = serde_json::from_str(CAPABILITY_CATALOG_SCHEMA).unwrap();
    let validator = jsonschema::validator_for(&schema).expect("schema compiles");
    let doc: Value = serde_json::from_str(&capability_catalog_json().unwrap()).unwrap();
    let errors: Vec<String> = validator
        .iter_errors(&doc)
        .map(|e| format!("{e} at {}", e.instance_path()))
        .collect();
    assert!(errors.is_empty(), "{errors:#?}");

    // The schema is not vacuous: an unknown effect token is refused.
    let mut bad = doc.clone();
    bad["operations"][0]["effects"]["reads"] = serde_json::json!(["the-network"]);
    assert!(!validator.is_valid(&bad), "an unknown read token validated");
    let mut bad = doc;
    bad["operations"][0]["surprise"] = serde_json::json!(true);
    assert!(!validator.is_valid(&bad), "an unknown member validated");
}

#[test]
fn operations_are_unique_sorted_and_well_formed() {
    let c = catalog();
    let names: Vec<&str> = c.operations.iter().map(|o| o.name.as_str()).collect();
    let mut sorted = names.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(names, sorted, "operations are unsorted or repeat a name");
    for o in &c.operations {
        assert!(!o.summary.is_empty(), "{}", o.name);
        assert!(
            !o.examples.is_empty(),
            "{} has no example (162 §3.9)",
            o.name
        );
        let mut ids: Vec<&str> = o.examples.iter().map(|e| e.id.as_str()).collect();
        let n = ids.len();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), n, "{} repeats an example id", o.name);
        match &o.cli {
            None => {
                assert!(o.name.starts_with("library."), "{}", o.name);
                assert!(
                    o.examples
                        .iter()
                        .all(|e| e.argv.is_empty() && e.stdin.is_some())
                );
            }
            Some(cli) => assert!(
                o.examples.iter().all(|e| e.argv.starts_with(&cli.argv)),
                "{}",
                o.name
            ),
        }
    }
}

/// 162 §3.4's companion rule, and `network: none` for everything else.
#[test]
fn effects_follow_the_vocabulary_rules() {
    for o in &catalog().operations {
        let e = &o.effects;
        if e.executes.iter().any(|x| x == "declared-commands") {
            assert!(e.writes.contains(&"delegated".to_string()), "{}", o.name);
            assert_eq!(e.network, "delegated", "{}", o.name);
            assert!(
                e.authority.contains(&"code-execution".to_string()),
                "{}",
                o.name
            );
        } else {
            assert_eq!(e.network, "none", "{}", o.name);
        }
        for list in [
            &e.reads,
            &e.writes,
            &e.executes,
            &e.environment,
            &e.authority,
        ] {
            let mut s = list.clone();
            s.sort();
            s.dedup();
            assert_eq!(
                &s, list,
                "{}: an effect list is unsorted or repeats",
                o.name
            );
        }
        if o.cli.is_none() {
            assert!(
                e.reads.is_empty() && e.writes.is_empty() && e.executes.is_empty(),
                "{}",
                o.name
            );
        }
    }
}

/// 162 §3.5: every outcome is one of spec 132's pairs, and its kinds map to
/// its code under `Error::exit_code`.
#[test]
fn outcomes_follow_the_exit_contract() {
    let kind_code: BTreeMap<&str, u8> = [
        ("validation", 1),
        ("stale", 1),
        ("not-found", 1),
        ("drift", 1),
        ("refused", 2),
        ("config", 2),
        ("usage", 3),
        ("io", 4),
        ("schema", 4),
        ("internal", 4),
    ]
    .into_iter()
    .collect();
    // Every kind in the closed set is mapped, so this table cannot fall behind.
    let all: BTreeSet<&str> = verdict::ERROR_KINDS.iter().copied().collect();
    assert_eq!(all, kind_code.keys().copied().collect());
    // And the mapping is the one `Error` itself applies, for every variant
    // that carries a kind (`drift` is couple's report, exit 1, spec 132).
    for e in [
        Error::Validation(Vec::new()),
        Error::NotFound(String::new()),
        Error::Stale {
            expected: String::new(),
            actual: String::new(),
        },
        Error::Parse(String::new()),
        Error::Config(String::new()),
        Error::Refused(String::new()),
        Error::Usage(String::new()),
        Error::Io(String::new()),
        Error::Schema(String::new()),
        Error::Internal(String::new()),
    ] {
        assert_eq!(kind_code[error_kind(&e)], e.exit_code(), "{e}");
    }

    for o in &catalog().operations {
        let codes: Vec<u8> = o.outcomes.iter().map(|x| x.exit_code).collect();
        assert!(codes.contains(&0), "{} declares no success", o.name);
        if o.cli.is_some() {
            assert!(
                codes.contains(&3) && codes.contains(&4),
                "{} lacks 3 or 4",
                o.name
            );
            let pinned = o.preconditions.iter().any(|p| p.name == "version-pin-met");
            assert_eq!(
                pinned,
                codes.contains(&2),
                "{}: pin precondition and exit 2 disagree",
                o.name
            );
        }
        for x in &o.outcomes {
            assert_eq!(
                x.outcome,
                spec_spine_types::outcome::of(x.exit_code),
                "{}",
                o.name
            );
            for k in &x.kinds {
                assert_eq!(
                    kind_code[k.as_str()],
                    x.exit_code,
                    "{}: kind {k} on {}",
                    o.name,
                    x.exit_code
                );
            }
        }
        for p in &o.preconditions {
            for c in &p.exit_codes {
                assert!(
                    codes.contains(c),
                    "{}: precondition {} exits {c}, undeclared",
                    o.name,
                    p.name
                );
            }
        }
        for ex in &o.examples {
            assert!(
                codes.contains(&ex.exit_code),
                "{}: example {} exits {}",
                o.name,
                ex.id,
                ex.exit_code
            );
        }
        // 162 §3.9: an error outcome other than usage and I/O is exercised.
        if codes.iter().any(|c| matches!(c, 1 | 2)) {
            assert!(
                o.examples.iter().any(|e| e.exit_code != 0),
                "{} declares a finding or refusal and no example exercises one",
                o.name
            );
        }
    }
}

/// 162 §3.10 item 2: the facade census.
#[test]
fn every_facade_function_is_cataloged_and_nothing_else_is() {
    let lib = fs::read_to_string(repo_root().join("crates/spec-spine-core/src/lib.rs")).unwrap();
    // Re-exported `_json` names that are not entry points, each with why.
    let excluded: BTreeMap<&str, &str> = [(
        "is_package_json",
        "a predicate on a path (spec 030), not a JSON-in/JSON-out function",
    )]
    .into_iter()
    .collect();

    let mut defined: BTreeMap<String, bool> = BTreeMap::new();
    let chunks: Vec<&str> = lib.split("\npub fn ").skip(1).collect();
    for chunk in &chunks {
        let name: String = chunk
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if !name.ends_with("_json") {
            continue;
        }
        // The body ends at the next item at column zero.
        let body = chunk.split("\n}\n").next().unwrap_or(chunk);
        let signature = body.split('{').next().unwrap_or("");
        let reads = signature.contains("repo_root")
            || body.contains("repo_root: String")
            || body.contains("base_root: String");
        defined.insert(name, reads);
    }
    // `pub use` re-exports of `_json` functions defined elsewhere.
    let reexports = lib
        .split("pub use ")
        .skip(1)
        .flat_map(|item| {
            let item = item.split(';').next().unwrap_or("");
            item.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .filter(|w| w.ends_with("_json"))
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .filter(|w| !excluded.contains_key(w.as_str()))
        .collect::<BTreeSet<String>>();
    // A re-exported entry point takes `repo_root` when its definition does.
    for name in reexports {
        defined.entry(name).or_insert(true);
    }

    let c = catalog();
    let mut cataloged: BTreeMap<String, bool> = BTreeMap::new();
    for o in &c.operations {
        for f in &o.facade {
            if let Some(prev) = cataloged.insert(f.function.clone(), f.reads_repository) {
                assert_eq!(
                    prev, f.reads_repository,
                    "{} disagrees with itself",
                    f.function
                );
            }
        }
    }
    let defined_names: BTreeSet<&String> = defined.keys().collect();
    let cataloged_names: BTreeSet<&String> = cataloged.keys().collect();
    assert_eq!(
        defined_names, cataloged_names,
        "the facade and the catalog's facade bindings differ (162 §3.10 item 2)"
    );
    for (name, reads) in &defined {
        assert_eq!(cataloged[name], *reads, "{name}: readsRepository");
    }
    assert!(
        defined.contains_key("selected_content_json"),
        "the re-export was missed"
    );
    assert!(!defined.contains_key("is_package_json"));
}

/// 162 §3.10 item 3: the verb census.
#[test]
fn every_verdict_verb_names_one_cataloged_operation() {
    let src =
        fs::read_to_string(repo_root().join("crates/spec-spine-types/src/verdict.rs")).unwrap();
    let module = src
        .split("pub mod verb {")
        .nth(1)
        .unwrap()
        .split("\n}\n")
        .next()
        .unwrap();
    let verbs: Vec<String> = module
        .lines()
        .filter_map(|l| l.trim().strip_prefix("pub const "))
        .map(|l| l.split('"').nth(1).unwrap().to_string())
        .collect();
    assert!(verbs.len() > 20, "the verb module was not read: {verbs:?}");
    let c = catalog();
    for v in &verbs {
        let hits: Vec<&Operation> = c.operations.iter().filter(|o| &o.name == v).collect();
        assert_eq!(hits.len(), 1, "verb {v} names {} operations", hits.len());
        let output = hits[0].cli.as_ref().map(|c| c.output.as_str());
        assert!(
            matches!(output, Some("verdict-envelope" | "read-document")),
            "verb {v} is bound to output {output:?}"
        );
    }
}

/// 162 §3.10 item 4 and D-9: `gate-verdict` marks exactly the operations the
/// managed gate runs through its `spec_spine` function.
#[test]
fn gate_verdict_marks_exactly_what_the_managed_gate_runs() {
    let gate = fs::read_to_string(repo_root().join("scripts/statecraft/gate.sh")).unwrap();
    let invoked: BTreeSet<String> = gate
        .lines()
        .filter_map(|l| l.trim().strip_prefix("spec_spine "))
        .map(|cmd| {
            let words: Vec<&str> = cmd
                .split_whitespace()
                .take_while(|w| {
                    !w.starts_with('-')
                        && !w.starts_with('"')
                        && !w.starts_with('$')
                        && !w.starts_with('>')
                })
                .collect();
            words.join(".")
        })
        .collect();
    assert!(
        invoked.contains("couple") && invoked.contains("check"),
        "{invoked:?}"
    );
    let marked: BTreeSet<String> = catalog()
        .operations
        .iter()
        .filter(|o| o.effects.authority.iter().any(|a| a == "gate-verdict"))
        .map(|o| o.name.clone())
        .collect();
    assert_eq!(
        marked, invoked,
        "gate-verdict and gate.sh's spec_spine lines differ"
    );
}

/// 162 §3.8 and §3.10 item 6, against the committed registry.
#[test]
fn stability_follows_the_committed_registry() {
    let dir = repo_root().join(".statecraft/derived/spec-registry/by-spec");
    let mut lifecycle: BTreeMap<String, (String, String)> = BTreeMap::new();
    for entry in fs::read_dir(&dir).unwrap() {
        let v: Value =
            serde_json::from_str(&fs::read_to_string(entry.unwrap().path()).unwrap()).unwrap();
        let spec = &v["record"];
        lifecycle.insert(
            spec["id"].as_str().unwrap().to_string(),
            (
                spec["status"].as_str().unwrap().to_string(),
                spec["implementation"].as_str().unwrap_or("").to_string(),
            ),
        );
    }
    assert!(lifecycle.len() > 100, "the registry was not read");
    for o in &catalog().operations {
        assert!(
            !o.governed_by.is_empty(),
            "{} names no governing spec",
            o.name
        );
        for id in &o.governed_by {
            assert!(
                lifecycle.contains_key(id),
                "{}: {id} is not in the registry",
                o.name
            );
        }
        match o.stability.as_str() {
            "stable" => {
                for id in &o.governed_by {
                    let (status, implementation) = &lifecycle[id];
                    assert!(
                        status == "approved" && implementation == "complete",
                        "{} is stable but {id} is {status}/{implementation}",
                        o.name
                    );
                }
                assert!(o.deprecation.is_none(), "{}", o.name);
            }
            "experimental" => assert!(o.deprecation.is_none(), "{}", o.name),
            "deprecated" => assert!(o.deprecation.is_some(), "{}", o.name),
            other => panic!("{}: stability {other}", o.name),
        }
    }
}

#[test]
fn the_compatibility_map_names_every_declared_axis() {
    let c = catalog();
    let src = fs::read_to_string(repo_root().join("crates/spec-spine-types/src/lib.rs")).unwrap();
    let block = src
        .split("pub use version::{")
        .nth(1)
        .unwrap()
        .split("};")
        .next()
        .unwrap();
    let constants: BTreeSet<&str> = block
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .filter(|w| w.ends_with("_VERSION") && *w != "parse_semver")
        .collect();
    // One axis per declared constant, plus the attestation axis declared
    // beside its DTO in `attest.rs`.
    assert_eq!(
        c.compatibility.len(),
        constants.len() + 1,
        "{constants:?} vs {:?}",
        c.compatibility
    );
    assert_eq!(c.compatibility["catalog"], CATALOG_SCHEMA_VERSION);
    for o in &c.operations {
        for r in &o.response {
            if r.axis == "unversioned" {
                assert!(r.version.is_empty(), "{}", o.name);
            } else {
                assert_eq!(
                    c.compatibility.get(&r.axis),
                    Some(&r.version),
                    "{}: axis {}",
                    o.name,
                    r.axis
                );
            }
        }
    }
}

#[test]
fn verify_answers_current_changed_and_missing() {
    let c = catalog();
    let target = op(&c, "capabilities.verify");
    let mut expect = BTreeMap::new();
    expect.insert(
        "capabilities.verify".to_string(),
        target.operation_digest.clone(),
    );
    expect.insert("check".to_string(), format!("sha256:{}", "0".repeat(64)));
    expect.insert(
        "no.such-operation".to_string(),
        format!("sha256:{}", "1".repeat(64)),
    );
    let report = capability_verify(&CapabilityVerifyRequest { expect }).unwrap();
    let outcomes: Vec<(&str, &str)> = report
        .results
        .iter()
        .map(|r| (r.name.as_str(), r.outcome.as_str()))
        .collect();
    assert_eq!(
        outcomes,
        [
            ("capabilities.verify", "current"),
            ("check", "changed"),
            ("no.such-operation", "missing")
        ]
    );
    assert_eq!(report.exit_code(), 1);
    assert_eq!(
        report.results[1].observed.as_deref(),
        Some(op(&c, "check").operation_digest.as_str())
    );
    assert_eq!(report.results[2].observed, None);

    let mut only_current = BTreeMap::new();
    only_current.insert(
        "capabilities.verify".to_string(),
        target.operation_digest.clone(),
    );
    assert_eq!(
        capability_verify(&CapabilityVerifyRequest {
            expect: only_current
        })
        .unwrap()
        .exit_code(),
        0
    );
}

#[test]
fn verify_refuses_malformed_requests_as_usage() {
    for bad in [
        r#"{"expect":{}}"#,
        r#"{"expect":{"check":"sha256:abc"}}"#,
        r#"{"expect":{"check":"sha256:ABCDEF0000000000000000000000000000000000000000000000000000000000"}}"#,
        r#"{"expect":{"check":"md5:00"}}"#,
        r#"{"pins":{}}"#,
        "not json",
    ] {
        let err = capability_verify_json(bad).unwrap_err();
        assert_eq!(err.exit_code(), 3, "{bad}: {err}");
    }
}

/// 162 §3.9: a facade-only example runs the function it binds.
#[test]
fn facade_only_examples_run() {
    for o in catalog().operations.iter().filter(|o| o.cli.is_none()) {
        for ex in &o.examples {
            let stdin = ex.stdin.as_deref().unwrap();
            let function = o.facade[0].function.as_str();
            let result = match function {
                "load_config_json" => spec_spine_core::load_config_json(stdin),
                "scaffold_init_json" => spec_spine_core::scaffold_init_json(stdin),
                "scaffold_init_opts_json" => {
                    let v: Value = serde_json::from_str(stdin).unwrap();
                    spec_spine_core::scaffold_init_opts_json(
                        &v["config"].to_string(),
                        &v["options"].to_string(),
                    )
                }
                other => panic!("no runner for facade-only function {other}"),
            };
            let (code, out) = match result {
                Ok(s) => (0, s),
                Err(e) => (e.exit_code(), String::new()),
            };
            assert_eq!(code, ex.exit_code, "{}", ex.id);
            for needle in &ex.stdout_includes {
                assert!(out.contains(needle), "{}: missing {needle:?}", ex.id);
            }
        }
    }
}
