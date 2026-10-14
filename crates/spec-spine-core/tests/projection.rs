//! Spec 167 §3.11 items 1, 2 and 6: the read-only tool projection is held to
//! the capability catalog it is derived from. The round trip, byte parity and
//! error envelopes run against the built binary in
//! `spec-spine-cli/tests/projection.rs`.

use std::collections::BTreeSet;

use serde_json::Value;
use sha2::{Digest, Sha256};
use spec_spine_core::{capability_catalog, project, tool_projection, tool_projection_json};
use spec_spine_types::{
    CapabilityCatalog, EXCLUSION_REASONS, Operation, TOOL_PROJECTION_SCHEMA,
    TOOL_PROJECTION_SCHEMA_VERSION, ToolProjection,
};

/// Host-path arguments, restated for the independent rule (167 D-15).
const HOST_PATHS: &[&str] = &[
    "--attestation",
    "--export",
    "--key",
    "--manifest",
    "--map-out",
    "--output-root",
    "--packet",
    "--paths-from",
    "--plan-file",
    "--pr-body",
    "--public-key",
    "--request",
    "--resolutions",
    "--scope",
    "--seal",
];

/// A valued argument that is not a host path, and why (167 D-15). With
/// [`HOST_PATHS`] this classifies every valued argument the catalog binds, so
/// a new one fails `every_valued_argument_is_classified` until it is decided.
const NOT_HOST_PATHS: &[(&str, &str)] = &[
    ("--affected-by", "a revision"),
    ("--base", "a revision"),
    ("--declared-by", "a spec id"),
    ("--expect", "an operation digest pin"),
    ("--head", "a revision"),
    ("--key-id", "a key label, not a key"),
    ("--obligation", "an obligation reference"),
    ("--operation", "an operation name"),
    ("--repo", "the bound repository; never a property"),
    ("--repository", "an opaque repository identity"),
    ("--revision", "a revision"),
    ("--slice", "an index slice name"),
    ("--spec", "a spec id"),
    ("--state", "a traceability state"),
    ("--status", "a lifecycle status"),
    ("--target", "an impact target reference"),
    ("--waiver-as-of", "a date"),
    ("--waiver-uses", "a waiver use count"),
    ("id", "a spec id"),
    (
        "path",
        "a repository-relative path, read under the bound repository",
    ),
    ("reference", "an obligation reference"),
];

fn catalog() -> CapabilityCatalog {
    capability_catalog().expect("the catalog builds")
}

fn forbidden(
    writes: &[String],
    executes: &[String],
    network: &str,
    env: &[String],
    reads: &[String],
    authority: &[String],
) -> Option<&'static str> {
    let bad_authority = [
        "code-execution",
        "corpus-rewrite",
        "ledger-rewrite",
        "signing",
        "signature-verification",
        "waiver-evaluation",
    ];
    if !writes.iter().all(|w| w == "temporary") {
        Some("writes")
    } else if !executes.iter().all(|x| x == "git") {
        Some("executes")
    } else if network != "none" {
        Some("network")
    } else if !env.is_empty() {
        Some("environment")
    } else if reads
        .iter()
        .any(|r| r == "key-material" || r == "attestation")
    {
        Some("key-material")
    } else if authority
        .iter()
        .any(|a| bad_authority.contains(&a.as_str()))
    {
        Some("authority")
    } else {
        None
    }
}

/// §3.2, implemented again from the spec text: `None` for a tool, or the
/// first reason in the closed order.
fn expected_reason(o: &Operation) -> Option<&'static str> {
    let Some(cli) = &o.cli else {
        return Some("no-json-answer");
    };
    if !["verdict-envelope", "read-document"].contains(&cli.output.as_str())
        || !cli.flags.iter().any(|f| f.name == "--json")
    {
        return Some("no-json-answer");
    }
    let mut found: Vec<&'static str> = Vec::new();
    let e = &o.effects;
    found.extend(forbidden(
        &e.writes,
        &e.executes,
        &e.network,
        &e.environment,
        &e.reads,
        &e.authority,
    ));
    for w in &o.effects_when {
        let required = cli.flags.iter().any(|f| f.name == w.flag && f.required);
        let a = &w.effects;
        if required {
            found.extend(forbidden(
                &a.writes,
                &a.executes,
                "none",
                &a.environment,
                &a.reads,
                &a.authority,
            ));
        }
    }
    let stdin_request = matches!(
        (o.name.as_str(), o.request.kind.as_str()),
        ("registry.closure" | "scope.evaluate", "document")
    );
    let host_path_required = cli.flags.iter().any(|f| {
        f.required
            && (HOST_PATHS.contains(&f.name.as_str())
                || (o.name == "scope.compare" && f.positional))
    });
    if !stdin_request && (host_path_required || o.request.kind == "document") {
        found.push("caller-path");
    }
    if o.stability == "deprecated" {
        found.push("deprecated");
    }
    found
        .into_iter()
        .min_by_key(|r| EXCLUSION_REASONS.iter().position(|x| x == r).unwrap())
}

/// The census (§3.11 item 1): `Err` naming every disagreement.
fn census(c: &CapabilityCatalog, p: &ToolProjection) -> Result<(), Vec<String>> {
    let mut problems = Vec::new();
    let mut seen = BTreeSet::new();
    for t in &p.tools {
        if !seen.insert(t.operation.as_str()) {
            problems.push(format!("{} appears twice", t.operation));
        }
    }
    for x in &p.excluded {
        if !seen.insert(x.operation.as_str()) {
            problems.push(format!("{} appears twice", x.operation));
        }
    }
    let names: BTreeSet<&str> = p.tools.iter().map(|t| t.name.as_str()).collect();
    if names.len() != p.tools.len() {
        problems.push("two tools share a name".to_string());
    }
    for o in &c.operations {
        let recorded = match (
            p.tools.iter().find(|t| t.operation == o.name),
            p.excluded.iter().find(|x| x.operation == o.name),
        ) {
            (Some(_), None) => None,
            (None, Some(x)) => Some(x.reason.as_str()),
            _ => {
                problems.push(format!("{} is not listed exactly once", o.name));
                continue;
            }
        };
        let expected = expected_reason(o);
        if recorded != expected {
            problems.push(format!(
                "{}: recorded {recorded:?}, the rule gives {expected:?}",
                o.name
            ));
        }
    }
    if seen.len() != c.operations.len() {
        problems.push("the projection lists an operation the catalog lacks".to_string());
    }
    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems)
    }
}

#[test]
fn every_operation_is_projected_or_excluded_once_by_the_rule() {
    let c = catalog();
    let p = tool_projection().unwrap();
    census(&c, &p).unwrap_or_else(|e| panic!("{e:#?}"));
    assert_eq!(p.catalog_digest, c.catalog_digest);
    for t in &p.tools {
        let o = c.operations.iter().find(|o| o.name == t.operation).unwrap();
        assert_eq!(t.operation_digest, o.operation_digest, "{}", t.name);
        assert_eq!(
            t.name,
            format!("spec_spine_{}", o.name.replace(['.', '-'], "_"))
        );
    }
    for x in &p.excluded {
        let o = c.operations.iter().find(|o| o.name == x.operation).unwrap();
        assert_eq!(x.operation_digest, o.operation_digest, "{}", x.operation);
    }
}

/// The measured projection (167 D-8, recorded as D-16). A change to the
/// catalog that moves an operation in or out fails here until the decision
/// entry moves with it.
#[test]
fn the_projected_set_is_the_recorded_one() {
    let p = tool_projection().unwrap();
    let tools: Vec<&str> = p.tools.iter().map(|t| t.operation.as_str()).collect();
    assert_eq!(
        tools,
        [
            "capabilities",
            "capabilities.verify",
            "check",
            "compile.check",
            "compile.spec",
            "config.show",
            "delta",
            "index.check",
            "index.coverage",
            "index.diagnostics",
            "index.orphans",
            "index.owner",
            "interface.verify",
            "lint",
            "registry.closure",
            "registry.impacts",
            "registry.list",
            "registry.moves",
            "registry.obligation",
            "registry.plan",
            "registry.relationships",
            "registry.show",
            "registry.status-report",
            "registry.traceability",
            "scope.evaluate",
            "verify.affected",
            "verify.plan",
        ]
    );
}

fn planted(name: &str, edit: impl FnOnce(&mut Operation)) -> CapabilityCatalog {
    let mut c = catalog();
    let o = c.operations.iter_mut().find(|o| o.name == name).unwrap();
    edit(o);
    c
}

#[test]
fn a_planted_forbidden_effect_moves_the_operation_out() {
    // Acceptance 1: the census fails on a planted record that writes the
    // corpus, executes declared commands, or reads key material, against the
    // projection computed before the plant; recomputing excludes it.
    let before = tool_projection().unwrap();
    for (edit, reason) in [
        (
            Box::new(|o: &mut Operation| o.effects.writes = vec!["corpus".into()])
                as Box<dyn Fn(&mut Operation)>,
            "writes",
        ),
        (
            Box::new(|o: &mut Operation| o.effects.executes = vec!["declared-commands".into()]),
            "executes",
        ),
        (
            Box::new(|o: &mut Operation| o.effects.reads.push("key-material".into())),
            "key-material",
        ),
    ] {
        let c = planted("verify.plan", |o| edit(o));
        assert!(
            census(&c, &before).is_err(),
            "{reason}: census passed a stale projection"
        );
        let after = project(&c).unwrap();
        census(&c, &after).unwrap_or_else(|e| panic!("{e:#?}"));
        let x = after
            .excluded
            .iter()
            .find(|x| x.operation == "verify.plan")
            .expect("excluded");
        assert_eq!(x.reason, reason);
    }
}

#[test]
fn no_projected_tool_can_write_execute_or_reach_out() {
    // §3.11 item 2.
    let c = catalog();
    for t in tool_projection().unwrap().tools {
        let o = c.operations.iter().find(|o| o.name == t.operation).unwrap();
        let e = &o.effects;
        assert!(e.writes.iter().all(|w| w == "temporary"), "{}", t.name);
        assert!(e.executes.iter().all(|x| x == "git"), "{}", t.name);
        assert_eq!(e.network, "none", "{}", t.name);
        assert!(e.environment.is_empty(), "{}", t.name);
        assert!(
            !e.authority.iter().any(|a| [
                "code-execution",
                "corpus-rewrite",
                "ledger-rewrite",
                "signing",
                "signature-verification",
                "waiver-evaluation"
            ]
            .contains(&a.as_str())),
            "{}",
            t.name
        );
        assert_eq!(t.trust, "untrusted-repository-data");
        assert!(t.hints.read_only && !t.hints.destructive && t.hints.idempotent);
        assert!(!t.hints.open_world);
    }
}

#[test]
fn no_input_schema_names_the_repository_json_or_a_host_path() {
    // Acceptance 2.
    for t in tool_projection().unwrap().tools {
        let schema = &t.input_schema;
        assert_eq!(schema["type"], "object", "{}", t.name);
        assert_eq!(schema["additionalProperties"], false, "{}", t.name);
        let props = schema["properties"].as_object().unwrap();
        for key in props.keys() {
            assert!(key != "repo" && key != "json", "{}: {key}", t.name);
            assert!(
                !HOST_PATHS.contains(&format!("--{key}").as_str()) || key == "request",
                "{}: host path {key}",
                t.name
            );
        }
        for m in &t.invocation.flags {
            assert!(m.flag != "--repo" && m.flag != "--json", "{}", t.name);
            let host = HOST_PATHS.contains(&m.flag.as_str());
            assert!(
                !host || m.kind == "stdin",
                "{}: {} is passed",
                t.name,
                m.flag
            );
        }
    }
}

#[test]
fn a_variant_tool_cannot_select_another_operation() {
    // Spec 167 D-14: `verify.plan` always passes `--plan` and cannot pass
    // `--affected-by`; `verify.affected` requires a base.
    let p = tool_projection().unwrap();
    let tool = |name: &str| p.tools.iter().find(|t| t.operation == name).unwrap();
    let plan = tool("verify.plan");
    assert!(
        plan.invocation
            .flags
            .iter()
            .any(|m| m.flag == "--plan" && m.kind == "fixed")
    );
    assert!(plan.input_schema["properties"].get("affected-by").is_none());
    assert!(plan.input_schema["properties"].get("plan").is_none());
    let affected = tool("verify.affected");
    assert!(
        affected.input_schema["required"]
            .as_array()
            .unwrap()
            .contains(&Value::from("affected-by"))
    );
    let check = tool("compile.check");
    assert!(check.input_schema["properties"].get("spec").is_none());
    let one = tool("compile.spec");
    assert!(one.input_schema["properties"].get("check").is_none());
    assert!(
        one.input_schema["required"]
            .as_array()
            .unwrap()
            .contains(&Value::from("spec"))
    );
}

#[test]
fn every_valued_argument_is_classified() {
    // 167 D-15: the host-path table is exhaustive over valued arguments.
    let decided: BTreeSet<&str> = HOST_PATHS
        .iter()
        .copied()
        .chain(NOT_HOST_PATHS.iter().map(|(a, _)| *a))
        .collect();
    for o in &catalog().operations {
        let Some(cli) = &o.cli else { continue };
        for f in &cli.flags {
            if f.value_name.is_none() || (o.name == "scope.compare" && f.positional) {
                continue;
            }
            assert!(
                decided.contains(f.name.as_str()),
                "{}: `{}` is neither a host path nor recorded as not one",
                o.name,
                f.name
            );
        }
    }
}

fn independent_digest(value: &Value) -> String {
    let mut v = value.clone();
    v.as_object_mut().unwrap().remove("projectionDigest");
    let mut bytes = serde_json::to_string_pretty(&v).unwrap();
    bytes.push('\n');
    let hex: String = Sha256::digest(bytes.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    format!("sha256:{hex}")
}

#[test]
fn the_document_is_canonical_digested_and_valid() {
    // §3.11 item 6.
    let a = tool_projection_json().unwrap();
    assert_eq!(a, tool_projection_json().unwrap(), "two calls differ");
    assert!(a.ends_with("}\n") && !a.ends_with("\n\n") && !a.contains('\r'));
    let v: Value = serde_json::from_str(&a).unwrap();
    let mut recanon = serde_json::to_string_pretty(&v).unwrap();
    recanon.push('\n');
    assert_eq!(a, recanon, "not canonical");
    assert_eq!(v["schemaVersion"], TOOL_PROJECTION_SCHEMA_VERSION);
    assert_eq!(v["projectionDigest"], independent_digest(&v));

    let names: Vec<&str> = v["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted);
    let codes: Vec<&str> = v["errors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["code"].as_str().unwrap())
        .collect();
    assert_eq!(
        codes,
        [
            "engine-unavailable",
            "invalid-arguments",
            "result-too-large",
            "unknown-tool"
        ]
    );

    let schema: Value = serde_json::from_str(TOOL_PROJECTION_SCHEMA).unwrap();
    let validator = jsonschema::validator_for(&schema).expect("schema compiles");
    let errors: Vec<String> = validator
        .iter_errors(&v)
        .map(|e| format!("{e} at {}", e.instance_path()))
        .collect();
    assert!(errors.is_empty(), "{errors:#?}");
    let mut bad = v.clone();
    bad["tools"][0]["hints"]["destructive"] = Value::Bool(true);
    assert!(!validator.is_valid(&bad), "a destructive tool validated");
    let mut bad = v.clone();
    bad["tools"][0]["inputSchema"]["patternProperties"] = serde_json::json!({".*": {}});
    assert!(!validator.is_valid(&bad), "an open inputSchema validated");
    let mut bad = v;
    bad["tools"][0]["inputSchema"]["properties"]["repo"] = serde_json::json!({"type": "string"});
    assert!(!validator.is_valid(&bad), "a repo property validated");
}
