//! `spec-spine config show`: the effective configuration as a governed read
//! (spec 054).

use std::fs;
use std::path::Path;
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_spec-spine"))
}

fn code(out: &std::process::Output) -> i32 {
    out.status.code().unwrap_or(-1)
}

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, content).unwrap();
}

fn show(root: &Path, extra: &[&str]) -> std::process::Output {
    let mut cmd = bin();
    cmd.arg("--repo").arg(root).args(["config", "show"]);
    cmd.args(extra);
    cmd.output().unwrap()
}

fn show_json(root: &Path) -> serde_json::Value {
    let out = show(root, &["--json"]);
    assert_eq!(code(&out), 0, "{}", String::from_utf8_lossy(&out.stderr));
    serde_json::from_slice(&out.stdout).expect("a single JSON object")
}

/// §3.1: it reads no committed artifact, so it answers on a repository that has
/// never compiled. That is the state an adopter is in when they most need to
/// see what their configuration resolved to.
#[test]
fn it_reports_on_a_repository_that_never_compiled() {
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir_all(tmp.path().join("specs")).unwrap();
    let out = show(tmp.path(), &[]);
    assert_eq!(code(&out), 0, "{}", String::from_utf8_lossy(&out.stderr));
    assert!(String::from_utf8_lossy(&out.stdout).contains("config_version"));
}

/// §3.1: `config` is a read. An absent `spec-spine.toml` is defaults, not a
/// file to create; scaffolding is `init`'s job and has been since spec 006.
#[test]
fn it_never_writes_a_config_file() {
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir_all(tmp.path().join("specs")).unwrap();
    assert_eq!(code(&show(tmp.path(), &[])), 0);
    assert!(
        !tmp.path().join("spec-spine.toml").exists(),
        "a read verb must not scaffold"
    );
}

/// §3.2: the floor is reported merged and attributed, in the order the gate
/// evaluates: built-in first, then the adopter's, each in declared order. This
/// is the fact claude-observatory recorded as unknowable (its spec 016 D-3).
#[test]
fn the_bypass_floor_is_merged_ordered_and_attributed() {
    let tmp = tempfile::tempdir().unwrap();
    write(
        tmp.path(),
        "spec-spine.toml",
        "[coupling]\nbypass_prefixes = [\"vendor/\", \"**/README.md\"]\n",
    );
    let v = show_json(tmp.path());
    let entries = v["coupling"]["bypass_prefixes"].as_array().unwrap();

    let built_in: Vec<&str> = spec_spine_core::DEFAULT_BYPASS_PREFIXES.to_vec();
    assert!(
        entries.len() > built_in.len(),
        "the adopter's entries join the floor"
    );
    // The floor comes first, in its own order.
    for (i, prefix) in built_in.iter().enumerate() {
        assert_eq!(entries[i]["prefix"], *prefix, "floor order at {i}");
        assert_eq!(entries[i]["sources"][0], "built-in");
    }
    // Then the adopter's, in declared order.
    let tail: Vec<&str> = entries[built_in.len()..]
        .iter()
        .map(|e| e["prefix"].as_str().unwrap())
        .collect();
    assert_eq!(tail, vec!["vendor/", "**/README.md"]);
    for e in &entries[built_in.len()..] {
        assert_eq!(e["sources"][0], "spec-spine.toml");
    }
}

/// §3.2: a prefix declared in both lists is reported once, attributed to both.
/// The match is an `or`, so the duplicate is harmless; reporting it twice would
/// suggest something to clean up that is not there.
#[test]
fn a_prefix_in_both_lists_appears_once_with_both_sources() {
    let tmp = tempfile::tempdir().unwrap();
    write(
        tmp.path(),
        "spec-spine.toml",
        "[coupling]\nbypass_prefixes = [\"docs/\"]\n",
    );
    let v = show_json(tmp.path());
    let entries = v["coupling"]["bypass_prefixes"].as_array().unwrap();

    let docs: Vec<&serde_json::Value> = entries.iter().filter(|e| e["prefix"] == "docs/").collect();
    assert_eq!(docs.len(), 1, "reported once: {docs:?}");
    let sources: Vec<&str> = docs[0]["sources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap())
        .collect();
    assert_eq!(sources, vec!["built-in", "spec-spine.toml"]);
}

/// §3.3: the JSON form is the object, never the spec 037 verdict envelope. An
/// envelope carries `ok` and `exitCode`, and this verb decides nothing.
#[test]
fn the_json_form_is_not_a_verdict_envelope() {
    let tmp = tempfile::tempdir().unwrap();
    let v = show_json(tmp.path());
    assert!(v.get("ok").is_none(), "no verdict members: {v}");
    assert!(v.get("exitCode").is_none(), "{v}");
    assert!(v.get("verb").is_none(), "{v}");
    assert_eq!(v["config_version"], "0.1.0", "§3.4: the shape's version");
}

/// §3.4: no field is omitted for being defaulted. An empty config reports every
/// table, filled in; that exhaustiveness is the whole reason the verb is worth
/// having.
#[test]
fn every_default_is_resolved_and_reported() {
    let tmp = tempfile::tempdir().unwrap();
    let v = show_json(tmp.path());
    for table in [
        "manifest",
        "domains",
        "kind",
        "layout",
        "index",
        "branding",
        "coupling",
        "provenance",
        "frontmatter",
        "lint",
    ] {
        assert!(v.get(table).is_some(), "missing table '{table}': {v}");
    }
    // Values nobody wrote down, which is exactly the point.
    assert_eq!(v["layout"]["cargo_workspace"], "Cargo.toml");
    assert_eq!(v["index"]["resolver_exclusions"][0], "target");
    assert_eq!(v["manifest"]["metadata_namespace"], "spec-spine");
}

/// §3.4: the report mirrors `Config`'s tables one for one. A table added to
/// `Config` and forgotten in `EffectiveConfig` fails here rather than silently
/// going unreported.
#[test]
fn the_report_covers_every_config_table() {
    let cfg = serde_json::to_value(spec_spine_types::Config::default()).unwrap();
    let effective = serde_json::to_value(spec_spine_types::EffectiveConfig::new(
        &spec_spine_types::Config::default(),
        Vec::new(),
    ))
    .unwrap();

    let mut want: Vec<&str> = cfg
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    want.sort_unstable();
    let mut got: Vec<&str> = effective
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .filter(|k| *k != "config_version")
        .collect();
    got.sort_unstable();
    assert_eq!(got, want, "every Config table must be reported");
}

/// §3.5: a pure function of the config file and the binary. The same repository
/// and the same binary produce byte-identical bytes.
#[test]
fn the_output_is_deterministic() {
    let tmp = tempfile::tempdir().unwrap();
    write(
        tmp.path(),
        "spec-spine.toml",
        "[coupling]\nbypass_prefixes = [\"vendor/\"]\n",
    );
    let a = show(tmp.path(), &["--json"]).stdout;
    let b = show(tmp.path(), &["--json"]).stdout;
    assert_eq!(a, b);
    assert!(a.ends_with(b"\n"), "trailing newline");
}

/// §3.1: a malformed `spec-spine.toml` is the existing config error, exit 3.
/// This verb reports a configuration, and a file that is not one is the same
/// failure it has always been.
#[test]
fn a_malformed_config_is_exit_3() {
    let tmp = tempfile::tempdir().unwrap();
    write(tmp.path(), "spec-spine.toml", "[coupling]\nnot_a_key = 1\n");
    assert_eq!(code(&show(tmp.path(), &[])), 3);
}
