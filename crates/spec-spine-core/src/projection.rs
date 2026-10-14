//! The read-only tool projection (spec 167), derived from the capability
//! catalog of this build and from nothing else (§3.1).
//!
//! An operation becomes a tool exactly when §3.2's eligibility rule holds over
//! its declared effects; every other operation is excluded with the first
//! reason that applies. A tool's input schema comes from its CLI binding
//! (§3.4), and its invocation says how a call becomes one argv of the
//! existing verb (§3.5). Nothing here runs a verb or computes an answer.
//!
//! The catalog does not say which arguments name a path on the host, or which
//! document requests a verb can read from stdin. Those are this module's two
//! tables, [`HOST_PATH_ARGUMENTS`] and [`STDIN_REQUESTS`], recorded as spec 167
//! D-15 and held by the core test to every valued argument in the catalog.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Value, json};
use spec_spine_types::{
    AdapterError, AddedEffects, ArgumentMapping, CapabilityCatalog, Effects, Error, Exclusion,
    Flag, Invocation, Operation, TOOL_DESCRIPTION_SUFFIX, TOOL_PROJECTION_SCHEMA_VERSION,
    TOOL_TRUST, Tool, ToolAnswer, ToolHints, ToolProjection,
};

use crate::capability::{capability_catalog, digest_without};

/// Authorities no tool may carry (167 §3.2 rule 6).
const FORBIDDEN_AUTHORITY: &[&str] = &[
    "code-execution",
    "corpus-rewrite",
    "ledger-rewrite",
    "signing",
    "signature-verification",
    "waiver-evaluation",
];

/// Arguments whose value is a path on the host (167 §3.4, D-15), as
/// `(operation, argument)`; `*` is every operation. An optional one is left
/// out of the tool; a required one excludes the operation `caller-path`.
/// A positional that names a repository-relative path (`index owner`'s and
/// `registry moves`' `path`) is not here: the verb reads it under the bound
/// repository.
pub const HOST_PATH_ARGUMENTS: &[(&str, &str)] = &[
    ("*", "--attestation"),
    ("*", "--export"),
    ("*", "--key"),
    ("*", "--manifest"),
    ("*", "--map-out"),
    ("*", "--output-root"),
    ("*", "--packet"),
    ("*", "--paths-from"),
    ("*", "--plan-file"),
    ("*", "--pr-body"),
    ("*", "--public-key"),
    ("*", "--resolutions"),
    ("*", "--request"),
    ("*", "--scope"),
    ("*", "--seal"),
    ("scope.compare", "a"),
    ("scope.compare", "b"),
];

/// Document requests a verb reads from stdin when its path argument is `-`
/// (167 §3.4, D-15), as `(operation, argument)`. Such an argument becomes the
/// tool's `request` property instead of a host path.
pub const STDIN_REQUESTS: &[(&str, &str)] = &[
    ("registry.closure", "--request"),
    ("scope.evaluate", "--scope"),
];

/// A string value may not begin with `-`, so no value can be read as a flag
/// (167 D-15).
const VALUE_PATTERN: &str = "^[^-]";

fn listed(table: &[(&str, &str)], operation: &str, argument: &str) -> bool {
    table
        .iter()
        .any(|(op, arg)| (*op == "*" || *op == operation) && *arg == argument)
}

/// The first forbidden effect in §3.2's reason order, if any.
fn forbidden(
    writes: &[String],
    executes: &[String],
    network: &str,
    environment: &[String],
    reads: &[String],
    authority: &[String],
) -> Option<&'static str> {
    if writes.iter().any(|w| w != "temporary") {
        return Some("writes");
    }
    if executes.iter().any(|x| x != "git") {
        return Some("executes");
    }
    if network != "none" {
        return Some("network");
    }
    if !environment.is_empty() {
        return Some("environment");
    }
    if reads
        .iter()
        .any(|r| r == "key-material" || r == "attestation")
    {
        return Some("key-material");
    }
    if authority
        .iter()
        .any(|a| FORBIDDEN_AUTHORITY.contains(&a.as_str()))
    {
        return Some("authority");
    }
    None
}

fn forbidden_base(e: &Effects) -> Option<&'static str> {
    forbidden(
        &e.writes,
        &e.executes,
        &e.network,
        &e.environment,
        &e.reads,
        &e.authority,
    )
}

fn forbidden_added(e: &AddedEffects) -> Option<&'static str> {
    forbidden(
        &e.writes,
        &e.executes,
        "none",
        &e.environment,
        &e.reads,
        &e.authority,
    )
}

/// `spec_spine_` and the operation name with `.` and `-` spelled `_`.
pub fn tool_name(operation: &str) -> String {
    format!("spec_spine_{}", operation.replace(['.', '-'], "_"))
}

fn rank(reason: &str) -> usize {
    spec_spine_types::EXCLUSION_REASONS
        .iter()
        .position(|r| *r == reason)
        .unwrap_or(usize::MAX)
}

/// What a projected operation's arguments become.
struct Arguments {
    mappings: Vec<ArgumentMapping>,
    properties: BTreeMap<String, Value>,
    required: BTreeSet<String>,
    stdin: bool,
}

fn string_schema() -> Value {
    json!({ "type": "string", "minLength": 1, "pattern": VALUE_PATTERN })
}

/// Decide one operation: `Ok` with its arguments, or `Err` with the first
/// exclusion reason (167 §3.2, §3.4).
fn decide(op: &Operation, catalog: &[Operation]) -> Result<Arguments, &'static str> {
    let mut reasons: Vec<&'static str> = Vec::new();
    let Some(cli) = &op.cli else {
        return Err("no-json-answer");
    };
    let answers_json = matches!(cli.output.as_str(), "verdict-envelope" | "read-document")
        && cli.flags.iter().any(|f| f.name == "--json");
    if !answers_json {
        return Err("no-json-answer");
    }
    if let Some(r) = forbidden_base(&op.effects) {
        reasons.push(r);
    }

    // A flag that adds a forbidden effect is dropped when optional and
    // excludes the operation when required (§3.2 rule 7, §3.4).
    let mut dropped: BTreeSet<&str> = BTreeSet::new();
    for when in &op.effects_when {
        if let Some(r) = forbidden_added(&when.effects) {
            let required = cli.flags.iter().any(|f| f.name == when.flag && f.required);
            if required {
                reasons.push(r);
            } else {
                dropped.insert(when.flag.as_str());
            }
        }
    }

    // A sibling variant's selector is never this operation's to pass
    // (167 D-14).
    let own: BTreeSet<&str> = cli.selector.iter().map(String::as_str).collect();
    for sibling in catalog {
        if sibling.name == op.name {
            continue;
        }
        if let Some(s) = sibling.cli.as_ref().filter(|s| s.argv == cli.argv) {
            for f in &s.selector {
                if !own.contains(f.as_str()) {
                    dropped.insert(f.as_str());
                }
            }
        }
    }

    let mut args = Arguments {
        mappings: Vec::new(),
        properties: BTreeMap::new(),
        required: BTreeSet::new(),
        stdin: false,
    };
    let mut positionals: Vec<&Flag> = Vec::new();
    let mut stdin_flag: Option<&Flag> = None;
    for f in &cli.flags {
        if f.name == "--json" || f.name == "--repo" || dropped.contains(f.name.as_str()) {
            continue;
        }
        if listed(STDIN_REQUESTS, &op.name, &f.name) && op.request.kind == "document" {
            stdin_flag = Some(f);
            continue;
        }
        if listed(HOST_PATH_ARGUMENTS, &op.name, &f.name) {
            if f.required {
                reasons.push("caller-path");
            }
            continue;
        }
        if f.positional {
            positionals.push(f);
            continue;
        }
        let property = f.name.trim_start_matches("--").to_string();
        if own.contains(f.name.as_str()) && f.value_name.is_none() {
            args.mappings.push(ArgumentMapping {
                flag: f.name.clone(),
                property: None,
                kind: "fixed".to_string(),
            });
            continue;
        }
        let (kind, schema) = match (&f.value_name, f.repeatable) {
            (None, _) => ("switch", json!({ "type": "boolean" })),
            (Some(_), false) => ("value", string_schema()),
            (Some(_), true) => (
                "repeated",
                json!({ "type": "array", "items": string_schema() }),
            ),
        };
        if f.required || own.contains(f.name.as_str()) {
            args.required.insert(property.clone());
        }
        args.properties.insert(property.clone(), schema);
        args.mappings.push(ArgumentMapping {
            flag: f.name.clone(),
            property: Some(property),
            kind: kind.to_string(),
        });
    }
    for f in positionals {
        args.properties.insert(f.name.clone(), string_schema());
        if f.required {
            args.required.insert(f.name.clone());
        }
        args.mappings.push(ArgumentMapping {
            flag: f.name.clone(),
            property: Some(f.name.clone()),
            kind: "positional".to_string(),
        });
    }
    if let Some(f) = stdin_flag {
        args.properties
            .insert("request".to_string(), json!({ "type": "object" }));
        args.required.insert("request".to_string());
        args.mappings.push(ArgumentMapping {
            flag: f.name.clone(),
            property: Some("request".to_string()),
            kind: "stdin".to_string(),
        });
        args.stdin = true;
    } else if op.request.kind == "document" {
        // A document request the verb cannot read from stdin is a host path.
        reasons.push("caller-path");
    }

    if op.stability == "deprecated" {
        reasons.push("deprecated");
    }
    match reasons.into_iter().min_by_key(|r| rank(r)) {
        Some(r) => Err(r),
        None => Ok(args),
    }
}

fn tool(op: &Operation, args: Arguments) -> Tool {
    let summary = op.summary.trim_end_matches('.');
    let mut input_schema = json!({
        "type": "object",
        "additionalProperties": false,
        "properties": args.properties,
    });
    if !args.required.is_empty() {
        input_schema["required"] = json!(args.required);
    }
    Tool {
        name: tool_name(&op.name),
        operation: op.name.clone(),
        operation_digest: op.operation_digest.clone(),
        description: format!("{summary}. {TOOL_DESCRIPTION_SUFFIX}"),
        input_schema,
        invocation: Invocation {
            argv: op.cli.as_ref().map(|c| c.argv.clone()).unwrap_or_default(),
            flags: args.mappings,
            stdin: args.stdin.then(|| "request".to_string()),
        },
        answer: ToolAnswer {
            response: op.response.clone(),
            outcomes: op.outcomes.clone(),
        },
        budget: op.budget.clone(),
        pagination: op.pagination.clone(),
        bounded: op.budget.is_some(),
        hints: ToolHints {
            read_only: true,
            destructive: false,
            idempotent: true,
            open_world: false,
        },
        trust: TOOL_TRUST.to_string(),
        stability: op.stability.clone(),
    }
}

/// The adapter's own errors (167 §3.7), sorted by code.
fn adapter_errors() -> Vec<AdapterError> {
    [
        (
            "engine-unavailable",
            "no binary the repository's version pin admits can be run",
            2,
        ),
        (
            "invalid-arguments",
            "the arguments fail the tool's inputSchema",
            3,
        ),
        (
            "result-too-large",
            "the answer exceeds the adapter's declared maximum result size",
            4,
        ),
        ("unknown-tool", "the name is not in tools", 3),
    ]
    .into_iter()
    .map(|(code, when, exit_code)| AdapterError {
        code: code.to_string(),
        when: when.to_string(),
        exit_code,
    })
    .collect()
}

/// The projection (167 §3.1): a pure function of this build's catalog.
pub fn tool_projection() -> Result<ToolProjection, Error> {
    project(&capability_catalog()?)
}

/// The projection of `catalog`. [`tool_projection`] passes this build's; a
/// test passes a planted one to show the rule moves with the record.
pub fn project(catalog: &CapabilityCatalog) -> Result<ToolProjection, Error> {
    let mut tools = Vec::new();
    let mut excluded = Vec::new();
    for op in &catalog.operations {
        match decide(op, &catalog.operations) {
            Ok(args) => tools.push(tool(op, args)),
            Err(reason) => excluded.push(Exclusion {
                operation: op.name.clone(),
                operation_digest: op.operation_digest.clone(),
                reason: reason.to_string(),
            }),
        }
    }
    tools.sort_by(|a, b| a.name.cmp(&b.name));
    excluded.sort_by(|a, b| a.operation.cmp(&b.operation));
    let mut seen = BTreeSet::new();
    for t in &tools {
        let valid = t.name.len() <= 64
            && t.name
                .bytes()
                .all(|b: u8| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_');
        if !valid || !seen.insert(t.name.as_str()) {
            return Err(Error::Internal(format!(
                "tool name `{}` is invalid or collides (spec 167 §3.3)",
                t.name
            )));
        }
    }
    let mut projection = ToolProjection {
        schema_version: TOOL_PROJECTION_SCHEMA_VERSION.to_string(),
        tool: "spec-spine".to_string(),
        tool_version: env!("CARGO_PKG_VERSION").to_string(),
        catalog_digest: catalog.catalog_digest.clone(),
        tools,
        excluded,
        errors: adapter_errors(),
        projection_digest: String::new(),
    };
    projection.projection_digest = digest_without(&projection, "projectionDigest")?;
    Ok(projection)
}
