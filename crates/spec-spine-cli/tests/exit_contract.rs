// Spec: specs/132-one-exit-contract-for-the-family/spec.md
//! Spec 132: the family exit contract, one remapped case per test.
//!
//! `0` ok, `1` finding, `2` refused, `3` usage, `4` failed. Each case asserts
//! the exit code and, where the verb speaks JSON, the envelope's `outcome`,
//! `exitCode` and `error.kind`, because a consumer branches on those and the
//! code alone cannot tell `config` from `refused`.

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn run_in(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_spec-spine"))
        .arg("--repo")
        .arg(root)
        .args(args)
        .output()
        .expect("spawn spec-spine")
}

fn code(out: &Output) -> i32 {
    out.status.code().unwrap_or(-1)
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

fn envelope(out: &Output) -> serde_json::Value {
    serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!(
            "stdout is one JSON envelope: {e}\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&out.stdout),
            stderr(out)
        )
    })
}

fn write_spec(root: &Path, id: &str) {
    let dir = root.join("specs").join(id);
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("spec.md"),
        format!(
            "---\nid: \"{id}\"\ntitle: \"T\"\nstatus: approved\ncreated: \"2026-09-24\"\n\
             summary: \"s\"\n---\n# {id}\n"
        ),
    )
    .unwrap();
}

/// A corpus whose committed shards are current.
fn fresh_corpus() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    write_spec(tmp.path(), "001-a");
    for verb in ["compile", "index"] {
        assert_eq!(code(&run_in(tmp.path(), &[verb])), 0, "fixture {verb}");
    }
    tmp
}

/// The envelope's header, asserted for every JSON case: exactly the members
/// §3.4 names, the tool, and an outcome that agrees with the exit code.
fn assert_header(v: &serde_json::Value, verb: &str, exit: i32, outcome: &str) {
    let obj = v.as_object().expect("an object");
    let mut keys: Vec<&str> = obj.keys().map(String::as_str).collect();
    keys.sort_unstable();
    let payload = if obj.contains_key("report") {
        "report"
    } else {
        "error"
    };
    let mut want = vec![
        "exitCode",
        payload,
        "outcome",
        "schemaVersion",
        "summary",
        "tool",
        "verb",
    ];
    want.sort_unstable();
    assert_eq!(keys, want, "{v}");
    assert_eq!(v["schemaVersion"], "1.0.0", "{v}");
    assert_eq!(v["tool"], "spec-spine", "{v}");
    assert_eq!(v["verb"], verb, "{v}");
    assert_eq!(v["exitCode"], exit, "{v}");
    assert_eq!(v["outcome"], outcome, "{v}");
    assert!(v["summary"].as_str().is_some_and(|s| !s.is_empty()), "{v}");
}

// ── 1: a finding ─────────────────────────────────────────────────────────

/// §3.2: a stale tree is a finding, exit 1, `kind: stale`, at every verb that
/// reads freshness. It was exit 2.
#[test]
fn stale_is_a_finding() {
    let tmp = fresh_corpus();
    write_spec(tmp.path(), "002-b");
    for args in [
        &["check"][..],
        &["index", "check"][..],
        &["compile", "--check"][..],
    ] {
        let out = run_in(tmp.path(), args);
        assert_eq!(code(&out), 1, "{args:?}: {}", stderr(&out));
    }
    let out = run_in(tmp.path(), &["index", "check", "--json"]);
    assert_eq!(code(&out), 1, "{}", stderr(&out));
    let v = envelope(&out);
    assert_header(&v, "index.check", 1, "finding");
    assert_eq!(v["report"]["fresh"], false, "the report says why: {v}");
    // A verb guarded by freshness refuses the same way.
    let out = run_in(tmp.path(), &["index", "coverage"]);
    assert_eq!(code(&out), 1, "{}", stderr(&out));
}

/// §3.2: a spec file whose frontmatter does not parse is a finding about the
/// corpus, exit 1, never a failure of the tool.
#[test]
fn malformed_authored_content_is_a_finding() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("specs/001-a");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("spec.md"), "---\nid: [unclosed\n---\n# x\n").unwrap();
    let out = run_in(tmp.path(), &["compile"]);
    assert_eq!(code(&out), 1, "{}", stderr(&out));
}

// ── 2: refused ───────────────────────────────────────────────────────────

/// §3.2: a configuration the loader refuses is exit 2, `kind: config`.
#[test]
fn an_invalid_config_is_refused() {
    let tmp = fresh_corpus();
    fs::write(
        tmp.path().join("spec-spine.toml"),
        "[layout]\nderived_dir = \"../x\"\n",
    )
    .unwrap();
    let out = run_in(tmp.path(), &["check"]);
    assert_eq!(code(&out), 2, "{}", stderr(&out));
    let out = run_in(tmp.path(), &["check", "--json"]);
    assert_eq!(code(&out), 2, "{}", stderr(&out));
    let v = envelope(&out);
    assert_header(&v, "check", 2, "refused");
    assert_eq!(v["error"]["kind"], "config", "{v}");

    fs::write(tmp.path().join("spec-spine.toml"), "not = [toml").unwrap();
    let out = run_in(tmp.path(), &["lint"]);
    assert_eq!(code(&out), 2, "unparseable config: {}", stderr(&out));
}

/// §3.2: a version pin the binary does not satisfy is exit 2, `kind:
/// refused`, and under `--json` it is an envelope like any other refusal.
#[test]
fn a_pin_mismatch_is_refused() {
    let tmp = fresh_corpus();
    fs::write(
        tmp.path().join("spec-spine.toml"),
        "[meta]\nrequired_version = \"=0.0.1\"\n",
    )
    .unwrap();
    let out = run_in(tmp.path(), &["lint"]);
    assert_eq!(code(&out), 2, "{}", stderr(&out));
    assert!(
        stderr(&out).contains("required_version"),
        "{}",
        stderr(&out)
    );
    let out = run_in(tmp.path(), &["lint", "--json"]);
    assert_eq!(code(&out), 2, "{}", stderr(&out));
    let v = envelope(&out);
    assert_header(&v, "lint", 2, "refused");
    assert_eq!(v["error"]["kind"], "refused", "{v}");
}

/// §3.2: a containment refusal (specs 126 to 128) is exit 2, not an I/O
/// failure: nothing went wrong, the tool declined to write outside its tree.
#[test]
fn a_containment_refusal_is_refused_not_io() {
    let tmp = tempfile::tempdir().unwrap();
    write_spec(tmp.path(), "CON");
    let out = run_in(tmp.path(), &["compile"]);
    assert_eq!(code(&out), 2, "{}", stderr(&out));
    assert!(stderr(&out).contains("refused"), "{}", stderr(&out));
}

// ── 3: usage ─────────────────────────────────────────────────────────────

/// §3.2: an argument combination the verb rejects is usage, exit 3, like an
/// argument clap rejects.
#[test]
fn a_rejected_flag_combination_is_usage() {
    let tmp = fresh_corpus();
    let out = run_in(tmp.path(), &["no-such-verb"]);
    assert_eq!(code(&out), 3, "{}", stderr(&out));
    let out = run_in(tmp.path(), &["compile", "--json"]);
    assert_eq!(code(&out), 3, "{}", stderr(&out));
    let v = envelope(&out);
    assert_header(&v, "compile.check", 3, "usage");
    assert_eq!(v["error"]["kind"], "usage", "{v}");
}

// ── 4: failed ────────────────────────────────────────────────────────────

/// §3.2: a read the tool could not perform is exit 4, `kind: io`.
#[test]
fn an_io_failure_is_failed() {
    let tmp = fresh_corpus();
    fs::create_dir(tmp.path().join("spec-spine.toml")).unwrap();
    let out = run_in(tmp.path(), &["lint", "--json"]);
    assert_eq!(code(&out), 4, "{}", stderr(&out));
    let v = envelope(&out);
    assert_header(&v, "lint", 4, "failed");
    assert_eq!(v["error"]["kind"], "io", "{v}");
}

/// §3.2: a tool-produced artifact that does not parse is exit 4, `kind:
/// schema`: the tool wrote it, so the corpus is not at fault.
#[test]
fn a_corrupt_committed_shard_is_failed() {
    let tmp = fresh_corpus();
    let shard = tmp.path().join(".derived/spec-registry/by-spec/001-a.json");
    assert!(shard.is_file(), "fixture shard at {}", shard.display());
    fs::write(&shard, "{ not json").unwrap();
    let out = run_in(tmp.path(), &["registry", "list"]);
    assert_eq!(code(&out), 4, "{}", stderr(&out));
}

// ── the fold and the envelope ────────────────────────────────────────────

/// §3.3: `check` folds its two halves by the higher code, so a failed read
/// outranks a finding and a clean tree is 0. Unix only: the failed read is a
/// directory with no permissions.
#[cfg(unix)]
#[test]
fn check_folds_by_the_higher_code() {
    let tmp = fresh_corpus();
    let out = run_in(tmp.path(), &["check", "--json"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert_header(&envelope(&out), "check", 0, "ok");

    // A stale registry and an index the tool cannot read: the failure wins.
    // (A corrupt shard is not this case: `index check` byte-compares, so it
    // is a stale shard, a finding.)
    write_spec(tmp.path(), "002-b");
    let by_spec = tmp.path().join(".derived/codebase-index/by-spec");
    set_mode(&by_spec, 0o000);
    let out = run_in(tmp.path(), &["check"]);
    set_mode(&by_spec, 0o755);
    assert_eq!(code(&out), 4, "{}", stderr(&out));
}

#[cfg(unix)]
fn set_mode(p: &Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(p, fs::Permissions::from_mode(mode)).unwrap();
}

#[cfg(not(unix))]
fn set_mode(_: &Path, _: u32) {}

/// §3.4: a successful report carries the same header.
#[test]
fn a_report_envelope_carries_the_header() {
    let tmp = fresh_corpus();
    let out = run_in(tmp.path(), &["lint", "--json"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let v = envelope(&out);
    assert_header(&v, "lint", 0, "ok");
    assert!(v.get("report").is_some());
}
