//! Declared consumer contracts (spec 170 3.4): every invocation a consumer
//! declares under `tests/consumers/<consumer>/contract.json` runs against the
//! in-tree binary and its fixture, and every member it lists must be there.
//!
//! A change that breaks a declared surface fails here, naming the consumer,
//! the invocation and the member (170 3.6), and is expected to update the
//! contract and add a dated entry to its `decisions` in the same change.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde_json::Value;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_spec-spine")
}

fn consumers_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/consumers")
}

fn git(repo: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .expect("git runs");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).unwrap();
        }
    }
}

fn run_ok(repo: &Path, args: &[&str]) {
    let out = Command::new(bin())
        .args(args)
        .current_dir(repo)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "setup `spec-spine {args:?}` failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Copy the fixture, derive its ledger, and commit it as the base; apply the
/// contract's `change` and commit it as the head. Returns `(base, head)`.
fn prepare(dir: &Path, contract: &Value, repo: &Path) -> (String, String) {
    let fixture = contract["fixture"]
        .as_str()
        .expect("contract names a fixture");
    copy_dir(&dir.join(fixture), repo);
    run_ok(repo, &["compile"]);
    run_ok(repo, &["index"]);
    git(repo, &["init", "-q", "-b", "main"]);
    git(repo, &["config", "user.name", "Fixture"]);
    git(repo, &["config", "user.email", "fixture@example.invalid"]);
    git(repo, &["config", "commit.gpgsign", "false"]);
    git(repo, &["add", "-A"]);
    git(repo, &["commit", "-qm", "base"]);
    let base = git(repo, &["rev-parse", "HEAD"]);
    if let Some(writes) = contract["change"]["write"].as_object() {
        for (rel, text) in writes {
            let path = repo.join(rel);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, text.as_str().unwrap()).unwrap();
        }
        git(repo, &["add", "-A"]);
        git(repo, &["commit", "-qm", "head"]);
    }
    let head = git(repo, &["rev-parse", "HEAD"]);
    (base, head)
}

fn type_of(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// `version` is admitted by `prefix` when it equals it or continues it at a
/// `.` boundary: `0.2` admits `0.2.0`, not `0.20.0`.
fn admits(prefix: &str, version: &str) -> bool {
    version == prefix
        || version
            .strip_prefix(prefix)
            .is_some_and(|rest| rest.starts_with('.'))
}

/// Check one invocation; every problem is collected, so one run names them all.
fn check_invocation(
    consumer: &str,
    inv: &Value,
    repo: &Path,
    base: &str,
    head: &str,
    problems: &mut Vec<String>,
) {
    let name = inv["name"].as_str().unwrap_or("?");
    let at = format!("{consumer}: `{name}`");
    let args: Vec<String> = inv["args"]
        .as_array()
        .expect("args")
        .iter()
        .map(|a| {
            a.as_str()
                .unwrap()
                .replace("{base}", base)
                .replace("{head}", head)
        })
        .collect();
    let mut child = Command::new(bin())
        .args(&args)
        .current_dir(repo)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    {
        use std::io::Write;
        let mut stdin = child.stdin.take().unwrap();
        if let Some(text) = inv["stdin"].as_str() {
            stdin.write_all(text.as_bytes()).unwrap();
        }
    }
    let out = child.wait_with_output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let code = out.status.code();

    let codes: Vec<i64> = inv["exit"]
        .as_array()
        .expect("exit")
        .iter()
        .filter_map(Value::as_i64)
        .collect();
    if !code.is_some_and(|c| codes.contains(&i64::from(c))) {
        problems.push(format!(
            "{at}: exit {code:?} is none of the declared {codes:?}; stderr: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
        return;
    }
    if let Some(needle) = inv["stdoutContains"].as_str()
        && !stdout.contains(needle)
    {
        problems.push(format!(
            "{at}: stdout does not contain {needle:?}, which the consumer locates the document by"
        ));
    }

    match inv["format"].as_str() {
        Some("version-token") => {
            let token = stdout
                .lines()
                .next()
                .and_then(|l| l.split_whitespace().last())
                .unwrap_or("");
            let parts: Vec<&str> = token.split('.').collect();
            if parts.len() != 3 || parts.iter().any(|p| p.parse::<u64>().is_err()) {
                problems.push(format!(
                    "{at}: the last token of the first line, {token:?}, is not a version"
                ));
            }
        }
        Some("text") => {}
        Some("json") => check_json(&at, inv, &stdout, problems),
        other => problems.push(format!("{at}: unknown format {other:?}")),
    }
}

fn check_json(at: &str, inv: &Value, stdout: &str, problems: &mut Vec<String>) {
    let doc: Value = match serde_json::from_str(stdout) {
        Ok(v) => v,
        Err(e) => {
            problems.push(format!("{at}: stdout is not one JSON document: {e}"));
            return;
        }
    };
    if let Some(schema) = inv.get("schema") {
        let axis = schema["axis"].as_str().unwrap_or("?");
        let pointer = schema["pointer"].as_str().unwrap();
        let versions: Vec<&str> = schema["versions"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(Value::as_str)
            .collect();
        match doc.pointer(pointer).and_then(Value::as_str) {
            None => problems.push(format!("{at}: the {axis} axis at {pointer} is absent")),
            Some(found) if !versions.iter().any(|p| admits(p, found)) => problems.push(format!(
                "{at}: the {axis} axis is {found}, and the contract declares {versions:?}"
            )),
            Some(_) => {}
        }
    }
    for member in inv["members"].as_array().into_iter().flatten() {
        let pointer = member["pointer"].as_str().unwrap();
        let want = member["type"].as_str().unwrap();
        let optional = member["optional"].as_bool().unwrap_or(false);
        match doc.pointer(pointer) {
            None if optional => {}
            None => problems.push(format!("{at}: member {pointer} is absent")),
            Some(v) if type_of(v) != want => problems.push(format!(
                "{at}: member {pointer} is a {}, the contract declares {want}",
                type_of(v)
            )),
            Some(v) => {
                if let Some(expected) = member.get("equals")
                    && v != expected
                {
                    problems.push(format!(
                        "{at}: member {pointer} is {v}, the consumer requires {expected}"
                    ));
                }
            }
        }
    }
}

fn contracts() -> Vec<(String, PathBuf, Value)> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(consumers_dir()).expect("tests/consumers exists") {
        let dir = entry.unwrap().path();
        if !dir.is_dir() {
            continue;
        }
        let consumer = dir.file_name().unwrap().to_string_lossy().into_owned();
        let text = std::fs::read_to_string(dir.join("contract.json"))
            .unwrap_or_else(|e| panic!("{consumer}: contract.json: {e}"));
        let contract: Value =
            serde_json::from_str(&text).unwrap_or_else(|e| panic!("{consumer}: {e}"));
        out.push((consumer, dir, contract));
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

#[test]
fn every_declared_consumer_contract_holds() {
    let all = contracts();
    assert!(
        all.iter().any(|(c, _, _)| c == "statecraft-cli"),
        "the first declared consumer is statecraft-cli (spec 170 3.4)"
    );
    let mut problems = Vec::new();
    for (consumer, dir, contract) in &all {
        assert_eq!(
            contract["consumer"].as_str(),
            Some(consumer.as_str()),
            "{consumer}: the contract names the directory it sits in"
        );
        assert!(
            contract["decisions"]
                .as_array()
                .is_some_and(|d| !d.is_empty()),
            "{consumer}: a contract carries its dated decisions"
        );
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path().join("repo");
        let (base, head) = prepare(dir, contract, &repo);
        for inv in contract["invocations"].as_array().expect("invocations") {
            check_invocation(consumer, inv, &repo, &base, &head, &mut problems);
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// The statecraft-cli contract covers every invocation spec 170 3.4's table
/// names, so dropping one from the file is not a way to stop checking it.
#[test]
fn the_statecraft_cli_contract_lists_the_tabled_invocations() {
    let (_, _, contract) = contracts()
        .into_iter()
        .find(|(c, _, _)| c == "statecraft-cli")
        .expect("statecraft-cli is declared");
    let argv: Vec<String> = contract["invocations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| {
            i["args"]
                .as_array()
                .unwrap()
                .iter()
                .map(|a| a.as_str().unwrap())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect();
    for want in [
        "--version",
        "check",
        "registry list --json",
        "registry plan --json",
        "registry closure --request - --json",
        "verify 001-a --json",
        "delta --base {base} --head {head} --json",
    ] {
        assert!(argv.iter().any(|a| a == want), "missing invocation: {want}");
    }
}

/// 170 3.6: a member the contract lists and the binary does not emit fails,
/// naming the consumer, the invocation and the member; so does an axis
/// version the contract does not declare. Run against the real binary with a
/// contract edited to demand what it does not give.
#[test]
fn a_broken_surface_fails_naming_consumer_invocation_and_member() {
    let (consumer, dir, mut contract) = contracts()
        .into_iter()
        .find(|(c, _, _)| c == "statecraft-cli")
        .unwrap();
    let invocations = contract["invocations"].as_array_mut().unwrap();
    let plan = invocations
        .iter_mut()
        .find(|i| i["name"] == "registry plan")
        .unwrap();
    plan["members"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({ "pointer": "/ready/0/noSuchMember", "type": "string" }));
    plan["schema"]["versions"] = serde_json::json!(["9"]);
    let plan = plan.clone();
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path().join("repo");
    let (base, head) = prepare(&dir, &contract, &repo);
    let mut problems = Vec::new();
    check_invocation(&consumer, &plan, &repo, &base, &head, &mut problems);
    assert!(
        problems
            .iter()
            .any(|p| p == "statecraft-cli: `registry plan`: member /ready/0/noSuchMember is absent"),
        "{problems:?}"
    );
    assert!(
        problems.iter().any(|p| p
            .starts_with("statecraft-cli: `registry plan`: the read axis is 0.")
            && p.ends_with("and the contract declares [\"9\"]")),
        "{problems:?}"
    );
}

#[test]
fn a_version_prefix_stops_at_a_component() {
    assert!(admits("0", "0.10.0"));
    assert!(admits("0.2", "0.2.0"));
    assert!(!admits("0.2", "0.20.0"));
    assert!(!admits("1", "10.0.0"));
    assert!(admits("1.1.0", "1.1.0"));
}
