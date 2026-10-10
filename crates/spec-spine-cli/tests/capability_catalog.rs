//! Spec 162: the capability catalog against the built binary.
//!
//! Every CLI example runs against `CARGO_BIN_EXE_spec-spine` in a fresh copy
//! of its fixture (§3.9); the effect audit reads each command module for the
//! effects it performs and requires them declared (§3.10 item 5); the
//! catalog's CLI bindings and spec 170's `verbs` name the same commands
//! (§3.10 item 7); and the `capabilities` forms answer as §3.11 and §3.12 say.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde_json::Value;
use spec_spine_types::{CapabilityCatalog, Example, Operation};

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_spec-spine")
}

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixtures() -> PathBuf {
    crate_dir().join("tests/fixtures/capability-catalog")
}

fn catalog() -> CapabilityCatalog {
    spec_spine_core::capability_catalog().unwrap()
}

/// A command with no inherited git or verify state, so a hook or an
/// enclosing `spec-spine verify` cannot change what an example observes.
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
        // `Cargo.toml.fixture` lands as `Cargo.toml`: a manifest committed
        // under this crate would be read by cargo as a package of its own.
        let name = entry.file_name().to_string_lossy().into_owned();
        let target = to.join(name.strip_suffix(".fixture").unwrap_or(&name));
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            // Read and write rather than `fs::copy`, which on macOS keeps the
            // source's mtime: a layered file of the same size and an old mtime
            // looks unchanged to git's stat check and is never committed.
            fs::write(&target, fs::read(entry.path()).unwrap()).unwrap();
        }
    }
}

fn run_ok(repo: &Path, args: &[&str]) {
    let out = command(bin(), repo).args(args).output().unwrap();
    assert!(
        out.status.success(),
        "spec-spine {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Commit the tree as it stands, after compiling and indexing it with the
/// binary under test (162 D-13).
fn commit(repo: &Path, message: &str) {
    run_ok(repo, &["compile"]);
    run_ok(repo, &["index"]);
    git(repo, &["add", "-A"]);
    git(repo, &["commit", "-qm", message]);
}

/// A fresh repository for `fixture`, or an empty directory for none.
fn prepare(fixture: Option<&str>) -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();
    let Some(name) = fixture else { return tmp };
    let src = fixtures().join(name);
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
        // Never compiled: nothing here can be read past the pin.
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

/// The head of a stream, for a failure message.
fn head(text: &str) -> String {
    text.chars().take(600).collect()
}

fn run_example(ex: &Example) -> (i32, String, String) {
    let tmp = prepare(ex.fixture.as_deref());
    let mut c = command(bin(), tmp.path());
    c.args(&ex.argv)
        .arg("--repo")
        .arg(tmp.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = c.spawn().unwrap();
    {
        let mut stdin = child.stdin.take().unwrap();
        if let Some(input) = &ex.stdin {
            stdin.write_all(input.as_bytes()).unwrap();
        }
    }
    let out = child.wait_with_output().unwrap();
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// 162 §3.9 and acceptance 4: every CLI example, against the built binary.
#[test]
fn every_example_exits_as_recorded() {
    let mut failures = Vec::new();
    let mut ran = 0;
    for op in catalog().operations.iter().filter(|o| o.cli.is_some()) {
        let declared: BTreeSet<u8> = op.outcomes.iter().map(|o| o.exit_code).collect();
        for ex in &op.examples {
            ran += 1;
            let (code, stdout, stderr) = run_example(ex);
            if code != i32::from(ex.exit_code) {
                failures.push(format!(
                    "{}: exit {code}, recorded {}\nstdout: {}\nstderr: {}",
                    ex.id,
                    ex.exit_code,
                    head(&stdout),
                    head(&stderr)
                ));
                continue;
            }
            if !declared.contains(&ex.exit_code) {
                failures.push(format!(
                    "{}: exit {code} is not declared by {}",
                    ex.id, op.name
                ));
            }
            for needle in &ex.stdout_includes {
                if !stdout.contains(needle) {
                    failures.push(format!(
                        "{}: stdout lacks {needle:?}\nstdout: {}",
                        ex.id,
                        head(&stdout)
                    ));
                }
            }
        }
    }
    assert!(ran > 60, "only {ran} examples ran");
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

/// The commands a module serves (162 §3.10 item 5), by operation name.
fn module_operations() -> BTreeMap<&'static str, Vec<&'static str>> {
    [
        ("cmd_attest.rs", vec!["attest"]),
        (
            "cmd_capabilities.rs",
            vec!["capabilities", "capabilities.verify"],
        ),
        ("cmd_check.rs", vec!["check"]),
        ("cmd_compact.rs", vec!["compact", "compact.plan"]),
        (
            "cmd_compile.rs",
            vec!["compile", "compile.check", "compile.spec"],
        ),
        ("cmd_config.rs", vec!["config.show"]),
        ("cmd_content.rs", vec!["content.select"]),
        ("cmd_couple.rs", vec!["couple"]),
        ("cmd_delta.rs", vec!["delta"]),
        (
            "cmd_index.rs",
            vec![
                "index.build",
                "index.check",
                "index.coverage",
                "index.diagnostics",
                "index.orphans",
                "index.owner",
                "index.render",
            ],
        ),
        ("cmd_interface.rs", vec!["interface.verify"]),
        ("cmd_lint.rs", vec!["lint"]),
        (
            "cmd_registry.rs",
            vec![
                "registry.closure",
                "registry.impacts",
                "registry.list",
                "registry.moves",
                "registry.obligation",
                "registry.plan",
                "registry.relationships",
                "registry.show",
                "registry.status-report",
            ],
        ),
        ("cmd_scope.rs", vec!["scope.compare", "scope.evaluate"]),
        (
            "cmd_verify.rs",
            vec!["verify", "verify.affected", "verify.plan"],
        ),
        ("seal.rs", vec!["attest", "verify-attestation"]),
        ("verify_attestation.rs", vec!["verify-attestation"]),
    ]
    .into_iter()
    .collect()
}

/// What a module's non-test source performs, as effect tokens: `executes:git`,
/// `reads:clock`, `writes:*` (some write), `writes:temporary`, `env:NAME`.
fn detected_effects(source: &str) -> BTreeSet<String> {
    let code = source.split("#[cfg(test)]").next().unwrap_or(source);
    let mut found = BTreeSet::new();
    if code.contains("Command::new(\"git\")") {
        found.insert("executes:git".to_string());
    }
    if code.contains("now_utc") || code.contains("SystemTime::now") {
        found.insert("reads:clock".to_string());
    }
    if code.contains("temp_dir") {
        found.insert("writes:temporary".to_string());
    }
    for write in [
        "fs::write(",
        "File::create(",
        "create_dir",
        "OpenOptions",
        "fs::rename(",
    ] {
        if code.contains(write) {
            found.insert("writes:*".to_string());
        }
    }
    if code.contains("env::var(") || code.contains("env::var_os(") {
        found.insert("env:*".to_string());
        // Every literal variable name the module mentions must be declared.
        for piece in code.split('"').skip(1).step_by(2) {
            if piece.starts_with("SPEC_SPINE_")
                && piece.chars().all(|c| c.is_ascii_uppercase() || c == '_')
            {
                found.insert(format!("env:{piece}"));
            }
        }
    }
    found
}

fn declares(ops: &[&Operation], effect: &str) -> bool {
    let (kind, token) = effect.split_once(':').unwrap();
    ops.iter().any(|o| {
        let e = &o.effects;
        let added = o.effects_when.iter().map(|w| &w.effects);
        match kind {
            "executes" => e.executes.iter().any(|x| x == token),
            "reads" => {
                e.reads.iter().any(|x| x == token)
                    || added.clone().any(|a| a.reads.iter().any(|x| x == token))
            }
            "writes" if token == "*" => !e.writes.is_empty(),
            "writes" => e.writes.iter().any(|x| x == token),
            "env" if token == "*" => !e.environment.is_empty(),
            "env" => e.environment.iter().any(|x| x == token),
            _ => false,
        }
    })
}

/// 162 §3.10 item 5 and acceptance 3: no module performs an effect none of
/// its operations declares. One-directional by design (D-6).
#[test]
fn the_effect_audit_finds_no_understated_effect() {
    let c = catalog();
    let map = module_operations();
    let src = crate_dir().join("src");
    let mut modules = BTreeSet::new();
    let mut failures = Vec::new();
    for entry in fs::read_dir(&src).unwrap() {
        let name = entry.unwrap().file_name().to_string_lossy().into_owned();
        if !(name.starts_with("cmd_") || name == "seal.rs" || name == "verify_attestation.rs") {
            continue;
        }
        modules.insert(name.clone());
        let Some(op_names) = map.get(name.as_str()) else {
            failures.push(format!("{name} serves no operation in the audit's map"));
            continue;
        };
        let ops: Vec<&Operation> = op_names
            .iter()
            .map(|n| {
                c.operations
                    .iter()
                    .find(|o| o.name == *n)
                    .unwrap_or_else(|| panic!("{n}"))
            })
            .collect();
        let source = fs::read_to_string(src.join(&name)).unwrap();
        for effect in detected_effects(&source) {
            if !declares(&ops, &effect) {
                failures.push(format!(
                    "{name} performs {effect}, which none of {op_names:?} declares"
                ));
            }
        }
    }
    let mapped: BTreeSet<String> = map.keys().map(|k| k.to_string()).collect();
    assert_eq!(
        modules, mapped,
        "the audit map and the command modules differ"
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The audit is not vacuous: it sees each effect it looks for.
#[test]
fn the_effect_audit_detects_each_effect() {
    let found = detected_effects(
        "fn f() { Command::new(\"git\"); OffsetDateTime::now_utc(); std::env::temp_dir(); \
         std::fs::write(p, b); std::env::var(\"SPEC_SPINE_X\"); }\n#[cfg(test)]\nmod t { fn g() { Command::new(\"cargo\"); } }",
    );
    let expected: BTreeSet<String> = [
        "executes:git",
        "reads:clock",
        "writes:temporary",
        "writes:*",
        "env:*",
        "env:SPEC_SPINE_X",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(found, expected);
    let undeclared = Operation {
        effects: Default::default(),
        ..catalog().operations[0].clone()
    };
    assert!(!declares(&[&undeclared], "executes:git"));
}

fn capabilities_doc(dir: &Path) -> Value {
    let out = command(bin(), dir)
        .args(["capabilities", "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    serde_json::from_slice(&out.stdout).unwrap()
}

/// 162 §3.10 item 7 and D-8: the one answer cannot disagree with itself.
#[test]
fn the_catalog_binds_exactly_the_verbs_spec_170_lists() {
    let tmp = tempfile::tempdir().unwrap();
    let doc = capabilities_doc(tmp.path());
    let verbs: BTreeSet<String> = doc["verbs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["path"].as_str().unwrap().to_string())
        .collect();
    let bound: BTreeSet<String> = catalog()
        .operations
        .iter()
        .filter_map(|o| o.cli.as_ref().map(|c| c.argv.join(" ")))
        .collect();
    assert_eq!(verbs, bound);
}

/// 162 §3.12 and acceptance 1: the document carries the catalog exactly as
/// the facade builds it, on the capabilities axis' MINOR.
#[test]
fn the_capabilities_document_carries_the_catalog() {
    let tmp = tempfile::tempdir().unwrap();
    let doc = capabilities_doc(tmp.path());
    let facade: Value =
        serde_json::from_str(&spec_spine_core::capability_catalog_json().unwrap()).unwrap();
    assert_eq!(doc["catalog"], facade);
    assert_eq!(
        doc["schemaVersion"],
        spec_spine_types::CAPABILITIES_SCHEMA_VERSION
    );
    assert_eq!(doc["schemaVersion"], "0.2.0");
    let again = capabilities_doc(tmp.path());
    assert_eq!(doc, again);
    let names: Vec<&str> = doc["catalog"]["operations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| o["name"].as_str().unwrap())
        .collect();
    // 162 §3.15's inventory, and nothing else.
    let inventory = [
        "attest",
        "capabilities",
        "capabilities.verify",
        "check",
        "compact",
        "compact.plan",
        "compile",
        "compile.check",
        "compile.spec",
        "config.show",
        "content.select",
        "couple",
        "delta",
        "index.build",
        "index.check",
        "index.coverage",
        "index.diagnostics",
        "index.orphans",
        "index.owner",
        "index.render",
        "interface.verify",
        "library.load-config",
        "library.scaffold-init",
        "library.scaffold-init-opts",
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
        "scope.compare",
        "scope.evaluate",
        "verify",
        "verify-attestation",
        "verify.affected",
        "verify.plan",
    ];
    assert_eq!(names, inventory);
}

/// 162 §3.11 and acceptance 5, through the CLI: `current` exits 0.
#[test]
fn verify_answers_current_with_exit_zero() {
    let c = catalog();
    let digest = &c
        .operations
        .iter()
        .find(|o| o.name == "check")
        .unwrap()
        .operation_digest;
    let tmp = tempfile::tempdir().unwrap();
    let out = command(bin(), tmp.path())
        .args([
            "capabilities",
            "verify",
            "--expect",
            &format!("check={digest}"),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["verb"], "capabilities.verify");
    assert_eq!(v["report"]["results"][0]["outcome"], "current");

    // A repeated pin is usage.
    let out = command(bin(), tmp.path())
        .args([
            "capabilities",
            "verify",
            "--expect",
            &format!("check={digest}"),
            "--expect",
            &format!("check={digest}"),
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(3));
    // None at all is usage too (clap).
    let out = command(bin(), tmp.path())
        .args(["capabilities", "verify"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(3));
}

/// 162 D-5: the new forms are answered before the pin, like 170's verb.
#[test]
fn the_capabilities_forms_answer_past_an_unmet_pin() {
    let tmp = prepare(Some("pinned"));
    let ok = command(bin(), tmp.path())
        .args(["capabilities", "--operation", "check", "--json"])
        .output()
        .unwrap();
    assert_eq!(
        ok.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&ok.stderr)
    );
    let v: Value = serde_json::from_slice(&ok.stdout).unwrap();
    assert_eq!(v["operation"]["name"], "check");
    assert_eq!(v["schemaVersion"], spec_spine_types::CATALOG_SCHEMA_VERSION);
    let refused = command(bin(), tmp.path()).args(["check"]).output().unwrap();
    assert_eq!(
        refused.status.code(),
        Some(2),
        "the fixture's pin is not unmet"
    );
}

/// 162 §2 and acceptance 6: the help text states spec 132's exit code.
#[test]
fn interface_verify_help_states_exit_one_for_a_stale_registry() {
    let tmp = tempfile::tempdir().unwrap();
    let out = command(bin(), tmp.path())
        .args(["interface", "verify", "--help"])
        .output()
        .unwrap();
    let help = String::from_utf8_lossy(&out.stdout)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        help.contains("1 when the committed registry is stale"),
        "{help}"
    );
    assert!(
        !help.contains("2 when the committed registry is stale"),
        "{help}"
    );
}
