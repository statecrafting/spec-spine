// Spec: specs/207-the-registry-lists-what-verify-would-run/spec.md
//! Spec 207 through the shipped binary: `registry list --implementation` and
//! `--has-acceptance` over a fixture corpus, their composition with `--status`
//! and `--ids-only`, the unchanged output without them, and the compiled
//! `declaresAcceptance` field (§3.1 to §3.5, acceptance criteria 1 to 7).

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_spec-spine"))
}

fn code(out: &Output) -> i32 {
    out.status.code().unwrap_or(-1)
}

fn run_in(root: &Path, args: &[&str]) -> Output {
    bin().arg("--repo").arg(root).args(args).output().unwrap()
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// One command, so `verify` would run something.
const ONE_COMMAND: &str = "```verify:cli\n# a comment first\ntrue\n```\n";
/// A `verify:cli` fence holding only comments and blank lines.
const ONLY_COMMENTS: &str = "```verify:cli\n# nothing to run\n\n   # indented\n```\n";
/// A fence of another tag: counted and declined by `verify`.
const OTHER_TAG: &str = "```verify:manual\ntrue\n```\n";
/// Prose and no fence at all.
const NO_FENCE: &str = "Checked by hand.\n";

/// A spec with the given status, optional implementation, extra frontmatter
/// and Verification body.
fn write_spec(root: &Path, id: &str, status: &str, imp: Option<&str>, extra: &str, block: &str) {
    let imp = imp
        .map(|i| format!("implementation: {i}\n"))
        .unwrap_or_default();
    let p = root.join("specs").join(id).join("spec.md");
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(
        p,
        format!(
            "---\nid: \"{id}\"\ntitle: \"Title {id}\"\nstatus: {status}\n{imp}created: \"2026-10-27\"\n\
             summary: \"s\"\n{extra}---\n# {id}\n\n## 1. Purpose\n\nWhy.\n\n## Verification\n\n{block}"
        ),
    )
    .unwrap();
}

/// `amends` plus `amends_verification` on one target, as V-018 requires.
fn replaces(target: &str) -> String {
    format!("amends:\n  - \"{target}\"\namends_verification:\n  - \"{target}\"\n")
}

/// The fixture corpus. What each spec answers:
///
/// | id     | status   | implementation | own block      | verify would run |
/// |--------|----------|----------------|----------------|------------------|
/// | 001-a  | approved | pending        | one command    | yes              |
/// | 002-b  | approved | in-progress    | only comments  | no               |
/// | 003-c  | approved | complete       | no fence       | no               |
/// | 004-d  | approved | n-a            | other tag      | no               |
/// | 005-e  | draft    | deferred       | one command    | yes              |
/// | 006-f  | draft    | (absent)       | one command    | yes              |
/// | 007-g  | approved | complete       | no fence       | yes, from 008-h  |
/// | 008-h  | approved | complete       | one command    | yes (replaces 7) |
/// | 009-i  | approved | complete       | one command    | no, from 010-j   |
/// | 010-j  | approved | complete       | only comments  | no (replaces 9)  |
/// | 011-k  | approved | complete       | no fence       | yes, from 013-m  |
/// | 012-l  | approved | in-progress    | no fence       | yes, from 013-m  |
/// | 013-m  | draft    | complete       | one command    | yes (replaces 12)|
fn corpus() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    let r = tmp.path();
    write_spec(r, "001-a", "approved", Some("pending"), "", ONE_COMMAND);
    write_spec(
        r,
        "002-b",
        "approved",
        Some("in-progress"),
        "",
        ONLY_COMMENTS,
    );
    write_spec(r, "003-c", "approved", Some("complete"), "", NO_FENCE);
    write_spec(r, "004-d", "approved", Some("n-a"), "", OTHER_TAG);
    write_spec(r, "005-e", "draft", Some("deferred"), "", ONE_COMMAND);
    write_spec(r, "006-f", "draft", None, "", ONE_COMMAND);
    write_spec(r, "007-g", "approved", Some("complete"), "", NO_FENCE);
    write_spec(
        r,
        "008-h",
        "approved",
        Some("complete"),
        &replaces("007-g"),
        ONE_COMMAND,
    );
    write_spec(r, "009-i", "approved", Some("complete"), "", ONE_COMMAND);
    write_spec(
        r,
        "010-j",
        "approved",
        Some("complete"),
        &replaces("009-i"),
        ONLY_COMMENTS,
    );
    write_spec(r, "011-k", "approved", Some("complete"), "", NO_FENCE);
    write_spec(
        r,
        "012-l",
        "approved",
        Some("in-progress"),
        &replaces("011-k"),
        NO_FENCE,
    );
    write_spec(
        r,
        "013-m",
        "draft",
        Some("complete"),
        &replaces("012-l"),
        ONE_COMMAND,
    );
    let out = run_in(r, &["compile"]);
    assert_eq!(code(&out), 0, "{}", String::from_utf8_lossy(&out.stderr));
    tmp
}

/// `registry list <args> --ids-only`, as a vector of ids.
fn ids(root: &Path, args: &[&str]) -> Vec<String> {
    let mut argv = vec!["registry", "list"];
    argv.extend_from_slice(args);
    argv.push("--ids-only");
    let out = run_in(root, &argv);
    assert_eq!(code(&out), 0, "{}", String::from_utf8_lossy(&out.stderr));
    stdout(&out).lines().map(str::to_string).collect()
}

const ALL: [&str; 13] = [
    "001-a", "002-b", "003-c", "004-d", "005-e", "006-f", "007-g", "008-h", "009-i", "010-j",
    "011-k", "012-l", "013-m",
];

// ===== criterion 1: --implementation ========================================

#[test]
fn each_implementation_token_selects_exactly_its_specs() {
    let tmp = corpus();
    let r = tmp.path();
    for (token, want) in [
        ("pending", vec!["001-a"]),
        ("in-progress", vec!["002-b", "012-l"]),
        (
            "complete",
            vec![
                "003-c", "007-g", "008-h", "009-i", "010-j", "011-k", "013-m",
            ],
        ),
        ("n-a", vec!["004-d"]),
        ("deferred", vec!["005-e"]),
    ] {
        assert_eq!(ids(r, &["--implementation", token]), want, "{token}");
    }
    // 006-f carries no value and matches none of the five.
    let mut seen: Vec<String> = ["pending", "in-progress", "complete", "n-a", "deferred"]
        .iter()
        .flat_map(|t| ids(r, &["--implementation", t]))
        .collect();
    seen.sort();
    assert!(!seen.contains(&"006-f".to_string()));
    assert_eq!(seen.len(), ALL.len() - 1);
}

#[test]
fn an_unknown_implementation_token_is_a_usage_error_naming_the_five() {
    let tmp = corpus();
    for bad in ["done", "n/a", "Complete", ""] {
        let out = run_in(tmp.path(), &["registry", "list", "--implementation", bad]);
        assert_eq!(
            code(&out),
            3,
            "{bad}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let err = String::from_utf8_lossy(&out.stderr);
        for token in ["pending", "in-progress", "complete", "n-a", "deferred"] {
            assert!(err.contains(token), "{bad}: {err}");
        }
        assert!(out.stdout.is_empty(), "{bad}");
    }
}

// ===== criteria 2 and 3: --has-acceptance ===================================

#[test]
fn has_acceptance_keeps_a_command_and_drops_comments_no_fence_and_other_tags() {
    let tmp = corpus();
    let kept = ids(tmp.path(), &["--has-acceptance"]);
    assert_eq!(
        kept,
        [
            "001-a", "005-e", "006-f", "007-g", "008-h", "011-k", "012-l", "013-m"
        ]
    );
    for dropped in ["002-b", "003-c", "004-d"] {
        assert!(!kept.contains(&dropped.to_string()), "{dropped}");
    }
}

#[test]
fn an_amended_block_answers_for_the_spec_it_replaces() {
    let tmp = corpus();
    let kept = ids(tmp.path(), &["--has-acceptance"]);
    // Replaced by a block with a command: kept, though its own has none.
    assert!(kept.contains(&"007-g".to_string()));
    // Replaced by a block holding only comments: dropped, though its own has
    // a command, because that block is not the one `verify` runs.
    assert!(!kept.contains(&"009-i".to_string()));
    assert!(!kept.contains(&"010-j".to_string()));
    // A two-step chain resolves to its end: 011-k <- 012-l <- 013-m.
    assert!(kept.contains(&"011-k".to_string()));
    assert!(kept.contains(&"012-l".to_string()));

    let out = run_in(
        tmp.path(),
        &["registry", "list", "--has-acceptance", "--json"],
    );
    assert_eq!(code(&out), 0, "{}", String::from_utf8_lossy(&out.stderr));
    let doc: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let from = |id: &str| {
        doc["items"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap_or_else(|| panic!("{id} not listed"))
            .get("acceptanceFrom")
            .cloned()
    };
    assert_eq!(from("007-g"), Some(serde_json::json!("008-h")));
    assert_eq!(from("011-k"), Some(serde_json::json!("013-m")));
    assert_eq!(from("012-l"), Some(serde_json::json!("013-m")));
    // Present only when it differs from the record's own id.
    for own in ["001-a", "005-e", "006-f", "008-h", "013-m"] {
        assert_eq!(from(own), None, "{own}");
    }
}

#[test]
fn the_answer_matches_what_verify_plans() {
    // §3.2 is "exactly as `verify` resolves": hold the read to the verb.
    let tmp = corpus();
    let kept = ids(tmp.path(), &["--has-acceptance"]);
    for id in ALL {
        let out = run_in(tmp.path(), &["verify", id, "--plan", "--json"]);
        assert_eq!(
            code(&out),
            0,
            "{id}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let plan: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
        // The verify envelope (spec 118) carries the plan under `report`.
        let runs = plan["report"]["commands"]
            .as_array()
            .expect("the plan lists its commands")
            .iter()
            .next()
            .is_some();
        assert_eq!(runs, kept.contains(&id.to_string()), "{id}: {plan}");
    }
}

// ===== criteria 4 and 5: composition, order and projection ==================

#[test]
fn the_filters_intersect_with_each_other_and_with_status_in_id_order() {
    let tmp = corpus();
    let r = tmp.path();
    assert_eq!(
        ids(r, &["--implementation", "complete", "--has-acceptance"]),
        ["007-g", "008-h", "011-k", "013-m"]
    );
    assert_eq!(
        ids(r, &["--status", "draft", "--has-acceptance"]),
        ["005-e", "006-f", "013-m"]
    );
    assert_eq!(
        ids(
            r,
            &[
                "--status",
                "approved",
                "--implementation",
                "complete",
                "--has-acceptance"
            ]
        ),
        ["007-g", "008-h", "011-k"]
    );
    // An empty intersection: nothing under --ids-only, `(no specs)` otherwise,
    // and exit 0 both ways.
    assert!(ids(r, &["--status", "approved", "--implementation", "deferred"]).is_empty());
    let out = run_in(
        r,
        &[
            "registry",
            "list",
            "--status",
            "approved",
            "--implementation",
            "deferred",
        ],
    );
    assert_eq!(code(&out), 0);
    assert_eq!(stdout(&out), "(no specs)\n");
}

#[test]
fn ids_only_projects_the_filtered_set_as_spec_009_defines() {
    let tmp = corpus();
    let args = [
        "registry",
        "list",
        "--implementation",
        "complete",
        "--has-acceptance",
        "--ids-only",
    ];
    let out = run_in(tmp.path(), &args);
    assert_eq!(code(&out), 0);
    assert_eq!(stdout(&out), "007-g\n008-h\n011-k\n013-m\n");

    let mut json_args = args.to_vec();
    json_args.push("--json");
    let out = run_in(tmp.path(), &json_args);
    assert_eq!(code(&out), 0);
    let doc: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(
        doc["items"],
        serde_json::json!(["007-g", "008-h", "011-k", "013-m"])
    );
    assert_eq!(doc["schemaVersion"], spec_spine_types::READ_SCHEMA_VERSION);
}

// ===== criterion 6: no new flag, no change ==================================

#[test]
fn without_the_new_flags_the_outputs_are_unchanged() {
    let tmp = corpus();
    let out = run_in(tmp.path(), &["registry", "list"]);
    assert_eq!(code(&out), 0);
    let mut want = String::new();
    for id in ALL {
        let status = if ["005-e", "006-f", "013-m"].contains(&id) {
            "draft"
        } else {
            "approved"
        };
        want.push_str(&format!("{id}  {status:<11}  Title {id}\n"));
    }
    assert_eq!(stdout(&out), want);
    assert_eq!(ids(tmp.path(), &[]), ALL);

    // `acceptanceFrom` is the flag's: a plain --json list carries none.
    let out = run_in(tmp.path(), &["registry", "list", "--json"]);
    assert_eq!(code(&out), 0);
    assert!(!stdout(&out).contains("acceptanceFrom"));
}

// ===== criterion 7: the compiled field ======================================

#[test]
fn declares_acceptance_is_compiled_exactly_when_the_own_section_holds_a_command() {
    let tmp = corpus();
    let declaring = ["001-a", "005-e", "006-f", "008-h", "009-i", "013-m"];
    for id in ALL {
        let path = tmp
            .path()
            .join(".derived/spec-registry/by-spec")
            .join(format!("{id}.json"));
        let shard: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(
            shard["specVersion"],
            spec_spine_types::REGISTRY_SCHEMA_VERSION
        );
        let field = shard["record"].get("declaresAcceptance");
        if declaring.contains(&id) {
            assert_eq!(field, Some(&serde_json::json!(true)), "{id}");
        } else {
            // The spec's own text only: 007-g, 011-k and 012-l are kept by
            // --has-acceptance and still compile without the field (D-2).
            assert_eq!(field, None, "{id}");
        }
    }
    // The shard schema names the field as the boolean the compiler writes.
    let schema: serde_json::Value =
        serde_json::from_str(spec_spine_types::REGISTRY_SPEC_SHARD_SCHEMA).unwrap();
    let prop = &schema["$defs"]["specRecord"]["properties"]["declaresAcceptance"];
    assert_eq!(prop["type"], "boolean", "{prop}");
}
