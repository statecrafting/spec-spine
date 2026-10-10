//! `spec-spine capabilities` (spec 170 3.5, 3.6): the document lists exactly
//! the verbs the binary wires, each `--json` verb names its schema axes at the
//! versions this build emits, and the verb reads no repository.
//!
//! The wired set is read from the binary's own `--help` pages, walked from the
//! top, rather than from the code that builds the document, so the two are
//! independent: a verb the document omitted would still be found here.

use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

use serde_json::Value;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_spec-spine")
}

fn run(dir: &Path, args: &[&str]) -> (Option<i32>, String) {
    let out = Command::new(bin())
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    )
}

fn capabilities(dir: &Path) -> Value {
    let (code, stdout) = run(dir, &["capabilities", "--json"]);
    assert_eq!(code, Some(0), "{stdout}");
    serde_json::from_str(&stdout).expect("one JSON document")
}

/// The subcommand names a `--help` page lists under `Commands:`, without
/// clap's `help`, and whether the page's usage line makes a subcommand
/// optional (`[COMMAND]`), which means the command runs on its own too.
fn help_page(path: &[String]) -> (Vec<String>, bool) {
    let mut args: Vec<&str> = path.iter().map(String::as_str).collect();
    args.push("--help");
    let tmp = tempfile::tempdir().unwrap();
    let (code, text) = run(tmp.path(), &args);
    assert_eq!(code, Some(0), "{args:?}");
    let runs_alone = text
        .lines()
        .find(|l| l.starts_with("Usage:"))
        .is_some_and(|l| !l.contains("<COMMAND>"));
    let mut subs = Vec::new();
    let mut in_commands = false;
    for line in text.lines() {
        if line.trim_end() == "Commands:" {
            in_commands = true;
            continue;
        }
        if in_commands {
            if !line.starts_with("  ") {
                break;
            }
            let name = line.split_whitespace().next().unwrap_or("");
            // A wrapped description line is indented further than a name.
            if line.starts_with("   ") || name.is_empty() || name == "help" {
                continue;
            }
            subs.push(name.to_string());
        }
    }
    (subs, runs_alone)
}

fn wired(path: &mut Vec<String>, out: &mut BTreeSet<String>) {
    let (subs, runs_alone) = help_page(path);
    if !path.is_empty() && (subs.is_empty() || runs_alone) {
        out.insert(path.join(" "));
    }
    for sub in subs {
        path.push(sub);
        wired(path, out);
        path.pop();
    }
}

#[test]
fn the_listed_verbs_are_the_wired_verbs() {
    let tmp = tempfile::tempdir().unwrap();
    let doc = capabilities(tmp.path());
    let listed: BTreeSet<String> = doc["verbs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["path"].as_str().unwrap().to_string())
        .collect();
    let mut wired_set = BTreeSet::new();
    wired(&mut Vec::new(), &mut wired_set);
    let missing: Vec<_> = wired_set.difference(&listed).collect();
    let extra: Vec<_> = listed.difference(&wired_set).collect();
    assert!(
        missing.is_empty(),
        "wired but absent from capabilities: {missing:?}"
    );
    assert!(extra.is_empty(), "listed but not wired: {extra:?}");
    // The verbs a consumer reaches for are among them.
    for verb in [
        "capabilities",
        "check",
        "delta",
        "registry closure",
        "registry list",
        "registry plan",
        "verify",
    ] {
        assert!(listed.contains(verb), "{verb}");
    }
}

#[test]
fn every_json_verb_names_its_schema_axes() {
    let tmp = tempfile::tempdir().unwrap();
    let doc = capabilities(tmp.path());
    let versions = [
        ("verdict", spec_spine_types::VERDICT_SCHEMA_VERSION),
        ("read", spec_spine_types::READ_SCHEMA_VERSION),
        ("delta", spec_spine_types::DELTA_SCHEMA_VERSION),
        ("config", spec_spine_types::CONFIG_VERSION),
        (
            "capabilities",
            spec_spine_types::CAPABILITIES_SCHEMA_VERSION,
        ),
        (
            "traceability",
            spec_spine_types::TRACEABILITY_SCHEMA_VERSION,
        ),
    ];
    let mut unnamed = Vec::new();
    for verb in doc["verbs"].as_array().unwrap() {
        let path = verb["path"].as_str().unwrap();
        let flags: Vec<&str> = verb["flags"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| f.as_str().unwrap())
            .collect();
        let axes = verb["json"].as_array().unwrap();
        if flags.contains(&"json") && axes.is_empty() {
            unnamed.push(path.to_string());
        }
        if !flags.contains(&"json") {
            assert!(axes.is_empty(), "{path} takes no --json but names an axis");
        }
        for a in axes {
            let name = a["axis"].as_str().unwrap();
            let (_, want) = versions
                .iter()
                .find(|(n, _)| *n == name)
                .unwrap_or_else(|| panic!("{path}: unknown axis {name}"));
            assert_eq!(a["version"], *want, "{path}: {name}");
        }
    }
    assert!(
        unnamed.is_empty(),
        "verbs that take --json and name no schema axis: {unnamed:?}"
    );
}

#[test]
fn the_document_names_the_binary_and_its_own_axis() {
    let tmp = tempfile::tempdir().unwrap();
    let doc = capabilities(tmp.path());
    assert_eq!(
        doc["schemaVersion"],
        spec_spine_types::CAPABILITIES_SCHEMA_VERSION
    );
    assert_eq!(spec_spine_types::CAPABILITIES_SCHEMA_VERSION, "0.1.0");
    let (_, version) = run(tmp.path(), &["--version"]);
    assert_eq!(
        version.split_whitespace().last(),
        doc["version"].as_str(),
        "the version --version prints"
    );
    // Sorted by path, so the bytes do not depend on declaration order.
    let paths: Vec<&str> = doc["verbs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["path"].as_str().unwrap())
        .collect();
    let mut sorted = paths.clone();
    sorted.sort_unstable();
    assert_eq!(paths, sorted);
    let verify = doc["verbs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["path"] == "verify")
        .unwrap();
    for flag in ["json", "plan"] {
        assert!(
            verify["flags"]
                .as_array()
                .unwrap()
                .iter()
                .any(|f| f == flag),
            "verify --{flag}"
        );
    }
}

/// 170 3.6: outside any repository it answers, and it reads no repository: a
/// configuration this binary cannot load, or a pin it does not meet, does not
/// refuse it. It writes nothing.
#[test]
fn it_reads_no_repository_and_writes_nothing() {
    let empty = tempfile::tempdir().unwrap();
    capabilities(empty.path());
    assert_eq!(std::fs::read_dir(empty.path()).unwrap().count(), 0);

    let pinned = tempfile::tempdir().unwrap();
    std::fs::write(
        pinned.path().join("spec-spine.toml"),
        "[meta]\nrequired_version = \"=0.0.1\"\n",
    )
    .unwrap();
    let (code, _) = run(pinned.path(), &["registry", "list", "--json"]);
    assert_eq!(code, Some(2), "the pin refuses an ordinary verb here");
    capabilities(pinned.path());

    let broken = tempfile::tempdir().unwrap();
    std::fs::write(broken.path().join("spec-spine.toml"), "not toml = = =\n").unwrap();
    capabilities(broken.path());
    assert_eq!(std::fs::read_dir(broken.path()).unwrap().count(), 1);
}

#[test]
fn without_json_it_prints_one_line_per_verb() {
    let tmp = tempfile::tempdir().unwrap();
    let (code, text) = run(tmp.path(), &["capabilities"]);
    assert_eq!(code, Some(0));
    let doc = capabilities(tmp.path());
    let verbs = doc["verbs"].as_array().unwrap().len();
    assert_eq!(text.lines().count(), verbs + 1, "{text}");
    assert!(text.lines().next().unwrap().starts_with("spec-spine "));
}
