//! The family envelope (spec 034, spec 132 section 3.4) for `launcher resolve
//! --json`, on the launcher's own schema axis.
//!
//! Header members: `schemaVersion`, `tool`, `verb`, `outcome`, `exitCode`,
//! `summary`, then exactly one of `report` and `error`. Canonical JSON as the
//! family emits it: sorted keys (serde_json maps are ordered), 2-space indent,
//! LF, trailing newline.

use serde_json::{Map, Value, json};

use crate::failure::Failure;

/// The launcher's own envelope schema axis (spec 188 section 3.7).
pub const SCHEMA_VERSION: &str = "0.1.0";
pub const VERB_RESOLVE: &str = "launcher.resolve";

fn outcome(code: u8) -> &'static str {
    match code {
        0 => "ok",
        1 => "finding",
        2 => "refused",
        3 => "usage",
        _ => "failed",
    }
}

fn header(verb: &str, code: u8, summary: &str) -> Map<String, Value> {
    let mut m = Map::new();
    m.insert("schemaVersion".into(), json!(SCHEMA_VERSION));
    m.insert("tool".into(), json!(crate::NAME));
    m.insert("verb".into(), json!(verb));
    m.insert("outcome".into(), json!(outcome(code)));
    m.insert("exitCode".into(), json!(code));
    m.insert("summary".into(), json!(summary));
    m
}

fn render(m: Map<String, Value>) -> String {
    let mut s = serde_json::to_string_pretty(&Value::Object(m)).unwrap_or_default();
    s.push('\n');
    s
}

pub fn report(verb: &str, summary: &str, report: Value) -> String {
    let mut m = header(verb, 0, summary);
    m.insert("report".into(), report);
    render(m)
}

pub fn failure(verb: &str, f: &Failure) -> String {
    let mut m = header(verb, f.code, &f.message);
    m.insert(
        "error".into(),
        json!({ "kind": f.kind, "message": f.message }),
    );
    render(m)
}
