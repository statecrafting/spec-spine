// Spec: specs/152-a-refusal-says-what-it-is-in-every-form/spec.md
//! Spec 152: a refusal says what it is in every form.
//!
//! §3.1: the `--json` index freshness report answers drift alone and carries
//! each unresolved claim in `unresolvedClaims`. §3.2: every read that takes
//! `--json` answers a failure with the envelope on stdout and nothing on
//! stderr. §3.3: the compact refusal names spec 144's rule. §3.4: verdict
//! `1.1.0`.

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

fn json(out: &Output) -> serde_json::Value {
    serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!(
            "stdout is one JSON document: {e}\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&out.stdout),
            stderr(out)
        )
    })
}

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, content).unwrap();
}

fn write_spec(root: &Path, id: &str, claim: &str) {
    write(
        root,
        &format!("specs/{id}/spec.md"),
        &format!(
            "---\nid: \"{id}\"\ntitle: \"T\"\nstatus: approved\ncreated: \"2026-09-25\"\n\
             implementation: complete\nsummary: \"s\"\nestablishes:\n  - \"{claim}\"\n---\n# {id}\n"
        ),
    );
}

/// A built corpus. With `missing`, its one spec claims `src/missing.rs`, which
/// does not exist: an unresolved claim and nothing else.
fn corpus(missing: bool) -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    write(tmp.path(), "src/a.rs", "pub fn a() {}\n");
    write_spec(
        tmp.path(),
        "001-a",
        if missing {
            "src/missing.rs"
        } else {
            "src/a.rs"
        },
    );
    for verb in ["compile", "index"] {
        run_in(tmp.path(), &[verb]);
    }
    tmp
}

/// The `index` report of `check --json` and the `index check --json` report.
fn index_reports(root: &Path) -> [(i32, serde_json::Value); 2] {
    let c = run_in(root, &["check", "--json"]);
    let i = run_in(root, &["index", "check", "--json"]);
    [
        (code(&c), json(&c)["report"]["index"].clone()),
        (code(&i), json(&i)["report"].clone()),
    ]
}

/// §3.1: an unresolved claim alone reads `fresh`, with no drift line, and the
/// claim is in `unresolvedClaims` in exactly the words the validation refusal
/// of every other reader uses. The exit code is still 1.
#[test]
fn an_unresolved_claim_alone_is_fresh_with_the_claim_beside_it() {
    let tmp = corpus(true);
    // The words the verbs' validation refusal uses (spec 145 §3.1).
    let coverage = run_in(tmp.path(), &["index", "coverage", "--json"]);
    let refusal = json(&coverage)["error"]["violations"].clone();
    assert_eq!(refusal.as_array().map(Vec::len), Some(1), "{refusal}");

    for (exit, report) in index_reports(tmp.path()) {
        assert_eq!(exit, 1, "{report}");
        assert_eq!(report["fresh"], true, "{report}");
        assert!(report.get("expected").is_none(), "{report}");
        assert!(report.get("actual").is_none(), "{report}");
        assert!(!report.to_string().contains("stale shard"), "{report}");
        assert!(
            !report.to_string().contains("blocking-diagnostics"),
            "{report}"
        );
        assert_eq!(report["unresolvedClaims"], refusal, "{report}");
        assert_eq!(report["unresolvedClaims"][0]["code"], "I-004", "{report}");
        assert_eq!(
            report["unresolvedClaims"][0]["path"], "src/missing.rs",
            "{report}"
        );
        assert_eq!(report["unresolvedClaims"][0]["severity"], "error");
    }
}

/// §3.1: a drifted shard is still `fresh: false`, naming the shard; with a
/// claim as well, both are reported; and a corpus with no claim carries no
/// `unresolvedClaims` member at all.
#[test]
fn drift_is_still_stale_and_no_claim_means_no_member() {
    let tmp = corpus(false);
    write_spec(tmp.path(), "002-b", "src/a.rs");
    for (exit, report) in index_reports(tmp.path()) {
        assert_eq!(exit, 1, "{report}");
        assert_eq!(report["fresh"], false, "{report}");
        assert!(
            report["actual"]
                .as_str()
                .is_some_and(|a| a.contains("by-spec/002-b.json")),
            "{report}"
        );
        assert!(report.get("unresolvedClaims").is_none(), "{report}");
    }

    let tmp = corpus(true);
    write_spec(tmp.path(), "002-b", "src/a.rs");
    for (exit, report) in index_reports(tmp.path()) {
        assert_eq!(exit, 1, "{report}");
        assert_eq!(report["fresh"], false, "{report}");
        assert!(!report.to_string().contains("blocking-diagnostics"));
        assert_eq!(report["unresolvedClaims"][0]["code"], "I-004", "{report}");
    }

    // The control: a fresh corpus without a claim.
    let tmp = corpus(false);
    for (exit, report) in index_reports(tmp.path()) {
        assert_eq!(exit, 0, "{report}");
        assert_eq!(report["fresh"], true, "{report}");
        assert!(report.get("unresolvedClaims").is_none(), "{report}");
    }
}

/// §3.2: every read that takes `--json`, refused by a configuration it cannot
/// load, writes the envelope on stdout naming its dotted verb, with the
/// process's own exit code, and nothing on stderr.
#[test]
fn every_json_read_answers_a_failure_with_an_envelope() {
    let tmp = tempfile::tempdir().unwrap();
    write_spec(tmp.path(), "001-a", "src/a.rs");
    // Refused by spec 144's layout rule: exit 2, kind `config`.
    write(
        tmp.path(),
        "spec-spine.toml",
        "[layout]\nderived_dir = \"../outside\"\n",
    );
    let request = tmp.path().join("request.json");
    let request = request.to_str().unwrap();
    let a = tmp.path().join("a.json");
    let a = a.to_str().unwrap();
    let cases: &[(&str, &[&str])] = &[
        ("registry.list", &["registry", "list", "--json"]),
        ("registry.show", &["registry", "show", "001", "--json"]),
        (
            "registry.status-report",
            &["registry", "status-report", "--json"],
        ),
        (
            "registry.relationships",
            &["registry", "relationships", "001", "--json"],
        ),
        (
            "registry.obligation",
            &["registry", "obligation", "001", "--json"],
        ),
        (
            "registry.closure",
            &["registry", "closure", "--request", request, "--json"],
        ),
        ("registry.impacts", &["registry", "impacts", "--json"]),
        ("registry.moves", &["registry", "moves", "--json"]),
        ("registry.plan", &["registry", "plan", "--json"]),
        ("index.orphans", &["index", "orphans", "--json"]),
        ("index.diagnostics", &["index", "diagnostics", "--json"]),
        ("index.owner", &["index", "owner", "src/a.rs", "--json"]),
        ("index.coverage", &["index", "coverage", "--json"]),
        ("config.show", &["config", "show", "--json"]),
        ("interface.verify", &["interface", "verify", "--json"]),
        (
            "scope.evaluate",
            &["scope", "evaluate", "--scope", a, "--json"],
        ),
        ("scope.compare", &["scope", "compare", a, a, "--json"]),
    ];
    for (verb, args) in cases {
        let out = run_in(tmp.path(), args);
        let rc = code(&out);
        assert_ne!(rc, 0, "{verb} must fail here");
        let v = json(&out);
        assert_eq!(v["verb"], *verb, "{v}");
        assert_eq!(v["exitCode"], rc, "{verb}: {v}");
        assert_eq!(v["schemaVersion"], "1.1.0", "{verb}: {v}");
        assert_eq!(v["tool"], "spec-spine", "{verb}: {v}");
        assert!(v["error"]["kind"].is_string(), "{verb}: {v}");
        assert!(v.get("report").is_none(), "{verb}: {v}");
        assert_eq!(stderr(&out), "", "{verb}: nothing on stderr");
    }
}

/// §3.2, the four reads of §1.2 on an unresolved claim: each exits 1 with a
/// `validation` (or `not-found`) envelope; and a successful read is still the
/// bare read document, not an envelope.
#[test]
fn the_four_measured_reads_answer_with_an_envelope() {
    let tmp = corpus(true);
    let scope = tmp.path().join("scope.json");
    fs::write(
        &scope,
        "{\"id\": \"W-1\", \"ownSpec\": \"001\", \"mutable\": [\"src/missing.rs\"]}\n",
    )
    .unwrap();
    let scope = scope.to_str().unwrap();
    for (verb, kind, args) in [
        (
            "index.coverage",
            "validation",
            &["index", "coverage", "--json"][..],
        ),
        (
            "index.owner",
            "validation",
            &["index", "owner", "src/missing.rs", "--json"],
        ),
        (
            "scope.evaluate",
            "validation",
            &["scope", "evaluate", "--scope", scope, "--json"],
        ),
        (
            "registry.show",
            "not-found",
            &["registry", "show", "999", "--json"],
        ),
    ] {
        let out = run_in(tmp.path(), args);
        assert_eq!(code(&out), 1, "{verb}: {}", stderr(&out));
        let v = json(&out);
        assert_eq!(v["verb"], verb, "{v}");
        assert_eq!(v["outcome"], "finding", "{v}");
        assert_eq!(v["error"]["kind"], kind, "{v}");
        assert_eq!(stderr(&out), "", "{verb}");
    }
    // Success is unchanged: the bare read document.
    let out = run_in(tmp.path(), &["registry", "show", "001", "--json"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let v = json(&out);
    assert!(v.get("verb").is_none() && v.get("outcome").is_none(), "{v}");
}

/// §3.3: a compact plan path spec 144's rule refuses is refused with that
/// rule's reason; a `.` segment keeps compact's own. Both exit 2.
#[test]
fn the_compact_refusal_names_the_rule() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write(root, "rules/one.md", "r\n");
    write(root, "docs/note.md", "see `rules/one.md`\n");
    write(
        root,
        "specs/000-a/spec.md",
        "---\nid: \"000-a\"\ntitle: \"A\"\nstatus: approved\ncreated: \"2026-09-25\"\n\
         implementation: complete\nsummary: \"s\"\n---\n# a\n\nBody.\n",
    );
    let git = |args: &[&str]| {
        let st = Command::new("git")
            .arg("-C")
            .arg(root)
            .args([
                "-c",
                "user.name=v",
                "-c",
                "user.email=v@v",
                "-c",
                "commit.gpgsign=false",
            ])
            .args(args)
            .status()
            .unwrap();
        assert!(st.success(), "git {args:?}");
    };
    git(&["init", "-q"]);
    git(&["add", "-A"]);
    git(&["commit", "-qm", "base"]);
    for (path, needle, not) in [
        (
            "C:rules/one.md",
            "`C:rules/one.md`, which contains a ':', a drive, drive-relative or stream form \
             on Windows (spec 144)",
            "no leading slash",
        ),
        (
            "rules/./one.md",
            "with no `.` or `..` component",
            "(spec 144)",
        ),
    ] {
        let plan = root.join("plan.yaml");
        fs::write(
            &plan,
            format!(
                "retire:\n  - path: \"{path}\"\n    kind: file\n    forms:\n      citation: \"x\"\n"
            ),
        )
        .unwrap();
        let out = run_in(
            root,
            &["compact", "--plan-file", plan.to_str().unwrap(), "--plan"],
        );
        let err = stderr(&out);
        assert_eq!(code(&out), 2, "{path}: {err}");
        assert!(err.contains(needle), "{path}: {err}");
        assert!(!err.contains(not), "{path}: {err}");
    }
}
