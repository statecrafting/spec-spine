//! Spec 167 §3.11 items 3, 4 and 5: the read-only tool projection against the
//! built binary.
//!
//! The reference mapper below is test code only (§3.11): it reads nothing but
//! the projection document, turns a call into one invocation of the verb as
//! §3.5 says, and raises the §3.7 errors. It is the proof that the
//! descriptors suffice to build an adapter, not an adapter this repository
//! ships. Examples run in fresh copies of spec 162's fixtures.

use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde_json::{Map, Value, json};
use spec_spine_types::{Example, Operation, Verdict, outcome};

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_spec-spine")
}

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn catalog_fixtures() -> PathBuf {
    crate_dir().join("tests/fixtures/capability-catalog")
}

/// A command with no inherited git or verify state.
fn command(program: &str, dir: &Path) -> Command {
    let mut c = Command::new(program);
    c.current_dir(dir);
    for var in [
        "GIT_DIR",
        "GIT_INDEX_FILE",
        "GIT_WORK_TREE",
        "GIT_OBJECT_DIRECTORY",
        "GIT_ALTERNATE_OBJECT_DIRECTORIES",
        "SPEC_SPINE_PR_BODY",
        "SPEC_SPINE_VERIFY_STACK",
    ] {
        c.env_remove(var);
    }
    c
}

fn git(repo: &Path, args: &[&str]) {
    let out = command("git", repo).args(args).output().expect("git runs");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name().to_string_lossy().into_owned();
        let target = to.join(name.strip_suffix(".fixture").unwrap_or(&name));
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::write(&target, fs::read(entry.path()).unwrap()).unwrap();
        }
    }
}

fn commit(repo: &Path, message: &str) {
    for verb in ["compile", "index"] {
        let out = command(bin(), repo).arg(verb).output().unwrap();
        assert!(
            out.status.success(),
            "{verb}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    git(repo, &["add", "-A"]);
    git(repo, &["commit", "-qm", message]);
}

/// A fresh repository for a spec-162 example fixture, prepared as that
/// suite prepares it, or an empty directory for none.
fn prepare(fixture: Option<&str>) -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();
    let Some(name) = fixture else { return tmp };
    let src = catalog_fixtures().join(name);
    let layered = src.join("base").is_dir();
    copy_dir(
        &if layered {
            src.join("base")
        } else {
            src.clone()
        },
        repo,
    );
    if name == "pinned" {
        return tmp;
    }
    git(repo, &["init", "-q", "-b", "main"]);
    git(repo, &["config", "user.name", "Fixture"]);
    git(repo, &["config", "user.email", "fixture@example.invalid"]);
    git(repo, &["config", "commit.gpgsign", "false"]);
    commit(repo, "base");
    if layered {
        copy_dir(&src.join("head"), repo);
        commit(repo, "head");
    }
    tmp
}

fn projection() -> Value {
    let out = Command::new(bin())
        .args(["capabilities", "--projection", "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    serde_json::from_slice(&out.stdout).unwrap()
}

fn operations() -> Vec<Operation> {
    spec_spine_core::capability_catalog().unwrap().operations
}

// ---------------------------------------------------------------- the mapper

/// The §3.7 envelope a conforming adapter raises itself: the family envelope
/// with verb `projection`, the code as its kind, and a message naming the
/// tool and the reason, never argument or answer content.
fn adapter_error(doc: &Value, code: &str, tool: &str, reason: &str) -> Value {
    let exit_code = doc["errors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["code"] == code)
        .unwrap_or_else(|| panic!("no error code {code}"))["exitCode"]
        .as_u64()
        .unwrap() as u8;
    let message = format!("{tool}: {reason}");
    json!({
        "schemaVersion": spec_spine_types::VERDICT_SCHEMA_VERSION,
        "tool": "spec-spine",
        "verb": "projection",
        "outcome": outcome::of(exit_code),
        "exitCode": exit_code,
        "summary": message,
        "error": { "kind": code, "message": message },
    })
}

/// The tool's descriptor, or the `unknown-tool` error.
fn find_tool<'a>(doc: &'a Value, name: &str) -> Result<&'a Value, Value> {
    doc["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == name)
        .ok_or_else(|| adapter_error(doc, "unknown-tool", name, "no such tool"))
}

/// The closed input schemas use only these keywords; anything else fails
/// loudly so the check cannot pass by ignoring a rule.
fn check_value(schema: &Value, value: &Value) -> Result<(), String> {
    let ok = match schema["type"].as_str().unwrap() {
        "boolean" => value.is_boolean(),
        "object" => value.is_object(),
        "string" => match value.as_str() {
            Some(s) => {
                assert_eq!(schema["pattern"], "^[^-]", "unknown pattern");
                assert_eq!(schema["minLength"], 1, "unknown minLength");
                !s.is_empty() && !s.starts_with('-')
            }
            None => false,
        },
        "array" => match value.as_array() {
            Some(items) => items
                .iter()
                .all(|i| check_value(&schema["items"], i).is_ok()),
            None => false,
        },
        other => panic!("unknown schema type {other}"),
    };
    if ok {
        Ok(())
    } else {
        Err("a value does not match its property".to_string())
    }
}

fn validate(tool: &Value, args: &Map<String, Value>) -> Result<(), String> {
    let schema = &tool["inputSchema"];
    assert_eq!(schema["additionalProperties"], false);
    let props = schema["properties"].as_object().unwrap();
    for key in args.keys() {
        if !props.contains_key(key) {
            return Err("an unknown property".to_string());
        }
    }
    for required in schema["required"].as_array().into_iter().flatten() {
        if !args.contains_key(required.as_str().unwrap()) {
            return Err("a required property is missing".to_string());
        }
    }
    for (key, value) in args {
        check_value(&props[key], value)?;
    }
    Ok(())
}

/// §3.5: one argv and the stdin bytes, or the `invalid-arguments` error.
fn to_invocation(
    doc: &Value,
    tool: &Value,
    repo: &Path,
    args: &Map<String, Value>,
) -> Result<(Vec<String>, Option<String>), Value> {
    let name = tool["name"].as_str().unwrap();
    validate(tool, args).map_err(|why| adapter_error(doc, "invalid-arguments", name, &why))?;
    let inv = &tool["invocation"];
    let mut argv = vec!["--repo".to_string(), repo.display().to_string()];
    argv.extend(
        inv["argv"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t.as_str().unwrap().to_string()),
    );
    let mut stdin = None;
    for m in inv["flags"].as_array().unwrap() {
        let flag = m["flag"].as_str().unwrap().to_string();
        let value = m["property"].as_str().and_then(|p| args.get(p));
        match m["kind"].as_str().unwrap() {
            "fixed" => argv.push(flag),
            "switch" => {
                if value == Some(&Value::Bool(true)) {
                    argv.push(flag);
                }
            }
            "value" => {
                if let Some(v) = value {
                    argv.push(flag);
                    argv.push(v.as_str().unwrap().to_string());
                }
            }
            "repeated" => {
                for v in value.and_then(Value::as_array).into_iter().flatten() {
                    argv.push(flag.clone());
                    argv.push(v.as_str().unwrap().to_string());
                }
            }
            "positional" => {
                if let Some(v) = value {
                    argv.push(v.as_str().unwrap().to_string());
                }
            }
            "stdin" => {
                argv.push(flag);
                argv.push("-".to_string());
                let mut bytes = serde_json::to_string_pretty(value.unwrap()).unwrap();
                bytes.push('\n');
                stdin = Some(bytes);
            }
            other => panic!("unknown mapping kind {other}"),
        }
    }
    argv.push("--json".to_string());
    Ok((argv, stdin))
}

/// Run one argv; the `engine-unavailable` error when nothing can be started.
fn run(
    doc: &Value,
    program: &str,
    tool: &str,
    repo: &Path,
    argv: &[String],
    stdin: Option<&str>,
) -> Result<(i32, Vec<u8>), Value> {
    let mut child = command(program, repo)
        .args(argv)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| adapter_error(doc, "engine-unavailable", tool, "no admitted binary runs"))?;
    {
        let mut pipe = child.stdin.take().unwrap();
        if let Some(input) = stdin {
            pipe.write_all(input.as_bytes()).unwrap();
        }
    }
    let out = child.wait_with_output().unwrap();
    Ok((out.status.code().unwrap_or(-1), out.stdout))
}

/// §3.5 and §3.6: exit 0 and 1 are answers, 2 to 4 are tool errors carrying
/// the verb's envelope, and an answer over the limit is replaced whatever its
/// code.
fn present(
    doc: &Value,
    tool: &str,
    code: i32,
    stdout: Vec<u8>,
    max: usize,
) -> Result<(i32, Vec<u8>), Value> {
    if stdout.len() > max {
        return Err(adapter_error(
            doc,
            "result-too-large",
            tool,
            "the answer exceeds the limit",
        ));
    }
    Ok((code, stdout))
}

// ------------------------------------------------- example argv to arguments

/// One parsed argument of an example's argv.
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone)]
enum Arg {
    Opt(String, Option<String>),
    Pos(usize, String),
}

/// Parse the tokens after the command path against the catalog flags, so a
/// round trip compares what clap would parse, not token order.
fn parse(op: &Operation, tokens: &[String], repo: &Path, stdin: Option<&str>) -> Vec<Arg> {
    let cli = op.cli.as_ref().unwrap();
    let rest = &tokens[cli.argv.len()..];
    assert_eq!(&tokens[..cli.argv.len()], cli.argv.as_slice());
    let mut out = Vec::new();
    let mut pos = 0;
    let mut i = 0;
    while i < rest.len() {
        let t = &rest[i];
        if let Some(long) = t.strip_prefix("--") {
            let (name, inline) = match long.split_once('=') {
                Some((n, v)) => (format!("--{n}"), Some(v.to_string())),
                None => (t.clone(), None),
            };
            let flag = cli
                .flags
                .iter()
                .find(|f| f.name == name)
                .unwrap_or_else(|| panic!("{}: unknown flag {name}", op.name));
            let mut value = if flag.value_name.is_some() {
                inline.or_else(|| {
                    i += 1;
                    rest.get(i).cloned()
                })
            } else {
                None
            };
            // A document request compares by content, file or stdin alike.
            // A `pinned` example refuses before reading its request, and
            // supplies none; it reads as the empty object.
            if op.request.kind == "document"
                && flag.value_name.is_some()
                && name != "--repository"
                && let Some(v) = &value
            {
                let text = if v == "-" {
                    stdin.unwrap_or("{}").to_string()
                } else {
                    fs::read_to_string(repo.join(v)).unwrap_or_else(|_| "{}".to_string())
                };
                let doc: Value = serde_json::from_str(&text).unwrap();
                value = Some(doc.to_string());
            }
            out.push(Arg::Opt(name, value));
        } else {
            out.push(Arg::Pos(pos, t.clone()));
            pos += 1;
        }
        i += 1;
    }
    out.sort();
    out
}

/// The tool arguments an example's argv expresses, or the reason it cannot.
fn arguments(tool: &Value, parsed: &[Arg]) -> Result<Map<String, Value>, String> {
    let mappings = tool["invocation"]["flags"].as_array().unwrap();
    let positionals: Vec<&Value> = mappings
        .iter()
        .filter(|m| m["kind"] == "positional")
        .collect();
    let mut args = Map::new();
    for a in parsed {
        match a {
            Arg::Opt(name, _) if name == "--json" => {}
            Arg::Opt(name, value) => {
                let m = mappings
                    .iter()
                    .find(|m| m["flag"] == name.as_str())
                    .ok_or_else(|| format!("{name} is not a tool argument"))?;
                let prop = m["property"].as_str();
                match m["kind"].as_str().unwrap() {
                    "fixed" => {}
                    "switch" => {
                        args.insert(prop.unwrap().to_string(), Value::Bool(true));
                    }
                    "value" => {
                        args.insert(
                            prop.unwrap().to_string(),
                            Value::from(value.clone().unwrap()),
                        );
                    }
                    "repeated" => {
                        let slot = args
                            .entry(prop.unwrap().to_string())
                            .or_insert_with(|| Value::Array(Vec::new()));
                        slot.as_array_mut()
                            .unwrap()
                            .push(Value::from(value.clone().unwrap()));
                    }
                    "stdin" => {
                        let doc: Value = serde_json::from_str(value.as_ref().unwrap()).unwrap();
                        args.insert(prop.unwrap().to_string(), doc);
                    }
                    other => panic!("unexpected kind {other}"),
                }
            }
            Arg::Pos(i, value) => {
                let m = positionals.get(*i).ok_or("too many positionals")?;
                args.insert(
                    m["property"].as_str().unwrap().to_string(),
                    Value::from(value.clone()),
                );
            }
        }
    }
    Ok(args)
}

struct Case {
    op: Operation,
    tool: Value,
    example: Example,
}

/// Every example of every projected operation.
fn cases(doc: &Value) -> Vec<Case> {
    let mut out = Vec::new();
    for op in operations() {
        let Some(tool) = doc["tools"]
            .as_array()
            .unwrap()
            .iter()
            .find(|t| t["operation"] == op.name.as_str())
        else {
            continue;
        };
        for example in &op.examples {
            out.push(Case {
                op: op.clone(),
                tool: tool.clone(),
                example: example.clone(),
            });
        }
    }
    out
}

fn with_json(argv: &[String]) -> Vec<String> {
    let mut v = argv.to_vec();
    if !v.iter().any(|t| t == "--json") {
        v.push("--json".to_string());
    }
    v
}

// ------------------------------------------------------------------- tests

#[test]
fn the_projection_is_served_byte_identically() {
    // Acceptance 4: repeated runs agree, and the verb's bytes are the facade's.
    let a = Command::new(bin())
        .args(["capabilities", "--projection", "--json"])
        .output()
        .unwrap();
    let b = Command::new(bin())
        .args(["capabilities", "--projection", "--json"])
        .output()
        .unwrap();
    assert_eq!(a.status.code(), Some(0));
    assert_eq!(a.stdout, b.stdout);
    assert_eq!(
        String::from_utf8(a.stdout).unwrap(),
        spec_spine_core::tool_projection_json().unwrap()
    );
    // The human form names each tool and its operation, one per line.
    let human = Command::new(bin())
        .args(["capabilities", "--projection"])
        .output()
        .unwrap();
    let text = String::from_utf8(human.stdout).unwrap();
    let doc = projection();
    let tools = doc["tools"].as_array().unwrap();
    assert_eq!(text.lines().count(), tools.len());
    for (line, t) in text.lines().zip(tools) {
        assert_eq!(
            line,
            format!(
                "{}  {}",
                t["name"].as_str().unwrap(),
                t["operation"].as_str().unwrap()
            )
        );
    }
    // It reads no repository: it answers outside one, and refuses a mix.
    let tmp = tempfile::tempdir().unwrap();
    let out = command(bin(), tmp.path())
        .args(["capabilities", "--projection", "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    for extra in [
        &["--operation", "check"][..],
        &["verify", "--expect", "x=y"][..],
    ] {
        let out = Command::new(bin())
            .args(["capabilities", "--projection"])
            .args(extra)
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(3), "{extra:?}");
    }
}

#[test]
fn every_example_round_trips_through_its_tool() {
    // §3.11 item 3.
    let doc = projection();
    let mut checked = 0;
    let mut failures = Vec::new();
    for case in cases(&doc) {
        let ex = &case.example;
        let tmp = prepare(ex.fixture.as_deref());
        let repo = tmp.path();
        let stdin = ex.stdin.as_deref();
        let direct = parse(&case.op, &with_json(&ex.argv), repo, stdin);
        let args = match arguments(&case.tool, &direct) {
            Ok(a) => a,
            Err(why) => {
                failures.push(format!("{}: not expressible: {why}", ex.id));
                continue;
            }
        };
        let (argv, mapped_stdin) = to_invocation(&doc, &case.tool, repo, &args)
            .unwrap_or_else(|e| panic!("{}: {e}", ex.id));
        assert_eq!(argv[0], "--repo");
        let mapped = parse(&case.op, &argv[2..], repo, mapped_stdin.as_deref());
        if mapped != direct {
            failures.push(format!("{}: {direct:?} became {mapped:?}", ex.id));
        }
        checked += 1;
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    assert!(checked >= 50, "only {checked} examples round-tripped");
}

#[test]
fn a_tool_call_answers_the_verbs_bytes() {
    // §3.11 item 4 and acceptance 3: the mapper's invocation and the direct
    // verb, against one fresh copy of the example's fixture, give the same stdout bytes
    // and exit code.
    let doc = projection();
    let mut failures = Vec::new();
    let mut checked = 0;
    for case in cases(&doc) {
        let ex = &case.example;
        let name = case.tool["name"].as_str().unwrap();
        let tmp = prepare(ex.fixture.as_deref());
        let repo = tmp.path();
        let mut direct_argv = with_json(&ex.argv);
        direct_argv.push("--repo".to_string());
        direct_argv.push(repo.display().to_string());
        let direct = run(&doc, bin(), name, repo, &direct_argv, ex.stdin.as_deref()).unwrap();

        let parsed = parse(&case.op, &with_json(&ex.argv), repo, ex.stdin.as_deref());
        let args = arguments(&case.tool, &parsed).unwrap();
        let (argv, stdin) = to_invocation(&doc, &case.tool, repo, &args).unwrap();
        let mapped = run(&doc, bin(), name, repo, &argv, stdin.as_deref()).unwrap();
        if mapped != direct {
            failures.push(format!(
                "{}: direct exit {} {:?}\nmapped exit {} {:?}",
                ex.id,
                direct.0,
                String::from_utf8_lossy(&direct.1)
                    .chars()
                    .take(300)
                    .collect::<String>(),
                mapped.0,
                String::from_utf8_lossy(&mapped.1)
                    .chars()
                    .take(300)
                    .collect::<String>()
            ));
        }
        checked += 1;
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
    assert!(checked >= 50, "only {checked} examples compared");
}

/// The envelope is the family's (spec 152): it parses as a `Verdict` with no
/// unknown member, its outcome is its exit code's, and the exit code is the
/// one the projection lists for the code.
fn assert_envelope(doc: &Value, envelope: &Value, code: &str) {
    let verdict: Verdict = serde_json::from_value(envelope.clone()).expect("a family envelope");
    let listed = doc["errors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["code"] == code)
        .unwrap();
    assert_eq!(
        u64::from(verdict.exit_code),
        listed["exitCode"].as_u64().unwrap(),
        "{code}"
    );
    assert_eq!(verdict.outcome, outcome::of(verdict.exit_code), "{code}");
    assert_eq!(verdict.verb, "projection");
    assert_eq!(verdict.error.as_ref().unwrap().kind, code);
    assert!(verdict.report.is_none());
}

#[test]
fn each_adapter_error_is_a_family_envelope() {
    // §3.11 item 5 and acceptance 5.
    let doc = projection();
    let tmp = prepare(Some("corpus"));
    let repo = tmp.path();

    let e = find_tool(&doc, "spec_spine_no_such_tool").unwrap_err();
    assert_envelope(&doc, &e, "unknown-tool");

    let calls: Value = serde_json::from_str(
        &fs::read_to_string(crate_dir().join("tests/fixtures/projection/invalid-calls.json"))
            .unwrap(),
    )
    .unwrap();
    for call in calls.as_array().unwrap() {
        let tool = find_tool(&doc, call["tool"].as_str().unwrap()).unwrap();
        let args = call["arguments"].as_object().unwrap();
        let e = to_invocation(&doc, tool, repo, args).expect_err(call["why"].as_str().unwrap());
        assert_envelope(&doc, &e, "invalid-arguments");
        // The message names the tool and the reason, never an argument.
        let message = e["error"]["message"].as_str().unwrap();
        assert!(message.starts_with(tool["name"].as_str().unwrap()));
        for v in args.values() {
            if let Some(s) = v.as_str() {
                assert!(!message.contains(s), "{message} echoes {s}");
            }
        }
    }

    let tool = find_tool(&doc, "spec_spine_registry_list").unwrap();
    let (argv, stdin) = to_invocation(&doc, tool, repo, &Map::new()).unwrap();
    let e = run(
        &doc,
        "/nonexistent/spec-spine",
        "spec_spine_registry_list",
        repo,
        &argv,
        None,
    )
    .unwrap_err();
    assert_envelope(&doc, &e, "engine-unavailable");

    let (code, stdout) = run(
        &doc,
        bin(),
        "spec_spine_registry_list",
        repo,
        &argv,
        stdin.as_deref(),
    )
    .unwrap();
    assert_eq!(code, 0);
    let e = present(&doc, "spec_spine_registry_list", code, stdout.clone(), 16).unwrap_err();
    assert_envelope(&doc, &e, "result-too-large");
    // Under the limit, the same answer passes through untouched.
    assert_eq!(
        present(&doc, "x", code, stdout.clone(), usize::MAX)
            .unwrap()
            .1,
        stdout
    );
}

#[test]
fn a_finding_is_an_answer_not_an_error() {
    // Acceptance 5: exit 1 is presented with the verb's own envelope.
    let doc = projection();
    let tmp = prepare(Some("corpus"));
    let repo = tmp.path();
    let tool = find_tool(&doc, "spec_spine_compile_spec").unwrap();
    let mut args = Map::new();
    args.insert("spec".to_string(), Value::from("999"));
    let (argv, stdin) = to_invocation(&doc, tool, repo, &args).unwrap();
    assert!(argv.contains(&"--spec".to_string()));
    let (code, stdout) = run(
        &doc,
        bin(),
        "spec_spine_compile_spec",
        repo,
        &argv,
        stdin.as_deref(),
    )
    .unwrap();
    let (code, stdout) =
        present(&doc, "spec_spine_compile_spec", code, stdout, usize::MAX).unwrap();
    assert_eq!(code, 1);
    let envelope: Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(envelope["outcome"], "finding");
    assert_eq!(envelope["verb"], "compile.spec");
}
