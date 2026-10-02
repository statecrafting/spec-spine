// Spec: specs/188-a-repository-pin-selects-the-engine/spec.md
//! V-1, resolution and verification: which engine a repository's pin selects,
//! and every refusal on the way. Stub engines in a temporary store; no network.

// The fixtures are shell-script engines and `tar` archives, which exist on the
// Unix hosts only. On Windows the crate and these files still compile; the
// wait-and-return execution path there has no fixture yet.
#![cfg(unix)]

mod common;

use std::fs;

use common::*;
use serde_json::Value;

fn json(o: &std::process::Output) -> Value {
    serde_json::from_slice(&o.stdout).expect("stdout is one JSON document")
}

#[test]
fn two_repositories_pinning_different_releases_resolve_their_own() {
    let w = World::new();
    let a = w.repo("a", "=0.28.0");
    let b = w.repo("b", "=0.29.0");
    let sa = stub("0.28.0", "a");
    let sb = stub("0.29.0", "b");
    let da = w.store_engine("0.28.0", &sa);
    let db = w.store_engine("0.29.0", &sb);

    let oa = w.cmd(&a).arg("check").output().unwrap();
    let ob = w.cmd(&b).arg("check").output().unwrap();
    assert_eq!(stdout(&oa), "engine 0.28.0\n");
    assert_eq!(stdout(&ob), "engine 0.29.0\n");

    // Neither run changed the other's (or its own) store entry.
    assert_eq!(w.store_entries("0.28.0"), vec![da.clone()]);
    assert_eq!(w.store_entries("0.29.0"), vec![db.clone()]);
    assert_eq!(
        sha256_file(&w.entry_dir("0.28.0", &da).join("spec-spine")),
        da
    );
    assert_eq!(
        sha256_file(&w.entry_dir("0.29.0", &db).join("spec-spine")),
        db
    );
}

#[test]
fn a_newer_configuration_key_still_resolves() {
    let w = World::new();
    let extra = "\n[layout]\nspecs_dir = \"specs\"\nkey_from_the_future = { nested = [1, 2, 3] }\n\
                 [table_the_launcher_never_heard_of]\nx = true\n";
    let r = w.repo_with("r", "=0.28.0", extra);
    w.store_engine("0.28.0", &stub("0.28.0", "x"));
    let o = w.cmd(&r).arg("check").output().unwrap();
    assert_eq!(code(&o), 0, "{}", stderr(&o));
    assert_eq!(stdout(&o), "engine 0.28.0\n");
}

#[test]
fn an_engine_on_path_is_never_consulted() {
    let w = World::new();
    let r = w.repo("r", "=0.28.0");
    // A decoy of the very release the pin names, first on PATH.
    write_exec(&w.decoy.join("spec-spine"), &stub("0.28.0", "decoy"));

    let refused = w.cmd(&r).arg("check").output().unwrap();
    assert_eq!(code(&refused), 2, "{}", stderr(&refused));
    assert!(
        !stdout(&refused).contains("engine"),
        "the decoy must not run"
    );
    assert!(
        stderr(&refused).contains("launcher install"),
        "{}",
        stderr(&refused)
    );

    w.store_engine("0.28.0", &stub("0.28.0", "store"));
    let o = w.cmd(&r).arg("check").output().unwrap();
    assert_eq!(code(&o), 0);
    let rj = w
        .cmd(&r)
        .args(["launcher", "resolve", "--json"])
        .output()
        .unwrap();
    let path = json(&rj)["report"]["path"].as_str().unwrap().to_string();
    assert!(path.starts_with(w.home.to_str().unwrap()), "{path}");
}

#[test]
fn an_inexact_pin_is_refused_with_exit_2() {
    let w = World::new();
    w.store_engine("0.28.0", &stub("0.28.0", "x"));
    for pin in ["^0.28", ">=0.28.0", "0.28.0", "=0.28"] {
        let r = w.repo("r", pin);
        let o = w.cmd(&r).arg("check").output().unwrap();
        assert_eq!(code(&o), 2, "{pin}");
        let e = stderr(&o);
        assert!(e.contains(pin) && e.contains("=X.Y.Z"), "{pin}: {e}");
    }
}

#[test]
fn a_lock_for_another_release_is_refused_naming_both() {
    let w = World::new();
    let r = w.repo("r", "=0.28.0");
    let d = w.store_engine("0.28.0", &stub("0.28.0", "x"));
    w.write_lock(&r, "0.27.0", &d, "");
    let o = w.cmd(&r).arg("check").output().unwrap();
    assert_eq!(code(&o), 2);
    let e = stderr(&o);
    assert!(e.contains("0.27.0") && e.contains("0.28.0"), "{e}");
}

#[test]
fn a_digest_that_disagrees_with_the_lock_is_refused_naming_both() {
    let w = World::new();
    let r = w.repo("r", "=0.28.0");
    let engine = w.path("elsewhere/spec-spine");
    let s = stub("0.28.0", "x");
    write_exec(&engine, &s);
    let actual = sha256_bytes(s.as_bytes());
    let locked = "a".repeat(64);
    w.write_lock(&r, "0.28.0", &locked, "");

    let o = w
        .cmd(&r)
        .env("SPEC_SPINE_ENGINE", &engine)
        .arg("check")
        .output()
        .unwrap();
    assert_eq!(code(&o), 2, "{}", stderr(&o));
    let e = stderr(&o);
    assert!(e.contains(&locked) && e.contains(&actual), "{e}");
    assert!(
        !stdout(&o).contains("engine"),
        "a refused override must not run"
    );

    // The override is not silently replaced by a store entry.
    w.store_engine("0.28.0", &s);
    let again = w
        .cmd(&r)
        .env("SPEC_SPINE_ENGINE", &engine)
        .arg("check")
        .output()
        .unwrap();
    assert_eq!(code(&again), 2);
}

#[test]
fn an_override_of_another_release_is_refused_without_fallback() {
    let w = World::new();
    let r = w.repo("r", "=0.28.0");
    w.store_engine("0.28.0", &stub("0.28.0", "store"));
    let engine = w.path("elsewhere/spec-spine");
    write_exec(&engine, &stub("0.27.0", "x"));
    let o = w
        .cmd(&r)
        .env("SPEC_SPINE_ENGINE", &engine)
        .arg("check")
        .output()
        .unwrap();
    assert_eq!(code(&o), 2);
    assert!(stderr(&o).contains("0.27.0"), "{}", stderr(&o));
    assert!(!stdout(&o).contains("engine"));
}

#[test]
fn a_store_entry_whose_bytes_changed_is_quarantined_and_refused() {
    let w = World::new();
    let r = w.repo("r", "=0.28.0");
    let original = stub("0.28.0", "x");
    let name = w.store_engine("0.28.0", &original);
    // Same name, different bytes: still a working script, so only the digest
    // can tell.
    w.store_engine_as("0.28.0", &name, &format!("{original}# tampered\n"));

    let o = w.cmd(&r).arg("check").output().unwrap();
    assert_eq!(code(&o), 2, "{}", stderr(&o));
    assert!(
        !stdout(&o).contains("engine"),
        "a corrupted entry must not run"
    );
    let e = stderr(&o);
    assert!(e.contains(&name) && e.contains("quarantined"), "{e}");

    let names = w.store_names("0.28.0");
    assert_eq!(names, vec![format!("{name}.quarantined-1")]);

    // It is never repaired silently: the next run finds nothing installed.
    let again = w.cmd(&r).arg("check").output().unwrap();
    assert_eq!(code(&again), 2);
    assert!(stderr(&again).contains("launcher install"));
}

#[test]
fn resolve_json_answers_the_family_envelope() {
    let w = World::new();
    let r = w.repo("r", "=0.28.0");
    let d = w.store_engine("0.28.0", &stub("0.28.0", "x"));
    w.write_lock(&r, "0.28.0", &d, "");

    let o = w
        .cmd(&r)
        .args(["launcher", "resolve", "--json"])
        .output()
        .unwrap();
    assert_eq!(code(&o), 0, "{}", stderr(&o));
    let text = stdout(&o);
    assert!(
        text.ends_with("}\n") && text.contains("\n  \"exitCode\""),
        "{text}"
    );
    // Sorted keys: the header members appear in alphabetical order.
    let order: Vec<usize> = [
        "exitCode",
        "outcome",
        "report",
        "schemaVersion",
        "summary",
        "tool",
        "verb",
    ]
    .iter()
    .map(|k| text.find(&format!("\"{k}\"")).unwrap())
    .collect();
    assert!(order.windows(2).all(|p| p[0] < p[1]), "{text}");

    let v = json(&o);
    assert_eq!(v["schemaVersion"], "0.1.0");
    assert_eq!(v["verb"], "launcher.resolve");
    assert_eq!(v["outcome"], "ok");
    assert_eq!(v["exitCode"], 0);
    assert!(v.get("error").is_none());
    let rep = v["report"].as_object().unwrap();
    let mut keys: Vec<&str> = rep.keys().map(String::as_str).collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        [
            "digest", "lock", "path", "release", "repo", "rule", "target", "trust"
        ]
    );
    let path = w.entry_dir("0.28.0", &d).join("spec-spine");
    assert_eq!(rep["path"], path.to_str().unwrap());
    assert_eq!(rep["digest"], format!("sha256:{d}"));
    assert_eq!(rep["release"], "0.28.0");
    assert_eq!(rep["target"], host_target());
    assert_eq!(rep["repo"], fs::canonicalize(&r).unwrap().to_str().unwrap());
    assert_eq!(rep["rule"], "store");
    assert_eq!(rep["lock"], "matched");
    assert_eq!(rep["trust"], "lock");
}

#[test]
fn resolve_json_without_a_lock_reports_the_published_digest_trust() {
    let w = World::new();
    let r = w.repo("r", "=0.28.0");
    w.store_engine("0.28.0", &stub("0.28.0", "x"));
    let o = w
        .cmd(&r)
        .args(["launcher", "resolve", "--json"])
        .output()
        .unwrap();
    let v = json(&o);
    assert_eq!(v["report"]["lock"], "absent");
    assert_eq!(v["report"]["trust"], "published-digest");
}

#[test]
fn resolve_json_exits_1_when_not_installed_and_writes_nothing() {
    let w = World::new();
    let r = w.repo("r", "=0.28.0");
    // Even a policy that would allow acquisition changes nothing here.
    let o = w
        .cmd(&r)
        .env("SPEC_SPINE_ACQUIRE", "auto")
        .env("SPEC_SPINE_RELEASE_BASE", w.path("no-such-release-dir"))
        .args(["launcher", "resolve", "--json"])
        .output()
        .unwrap();
    assert_eq!(code(&o), 1, "{}", stderr(&o));
    let v = json(&o);
    assert_eq!(v["outcome"], "finding");
    assert_eq!(v["exitCode"], 1);
    assert_eq!(v["error"]["kind"], "not-found");
    assert!(
        v["error"]["message"]
            .as_str()
            .unwrap()
            .contains("launcher install")
    );
    assert!(v.get("report").is_none());
    assert!(!w.home.exists(), "resolve created {}", w.home.display());
}

#[test]
fn resolve_json_refusals_and_usage_use_their_own_exit_codes() {
    let w = World::new();
    let bad = w.repo("bad", ">=0.28.0");
    let o = w
        .cmd(&bad)
        .args(["launcher", "resolve", "--json"])
        .output()
        .unwrap();
    assert_eq!(code(&o), 2);
    assert_eq!(json(&o)["outcome"], "refused");

    let o = w
        .cmd(&bad)
        .args(["launcher", "resolve", "--bogus"])
        .output()
        .unwrap();
    assert_eq!(code(&o), 3);
}

#[test]
fn resolve_reads_a_corrupted_entry_without_quarantining_it() {
    let w = World::new();
    let r = w.repo("r", "=0.28.0");
    let s = stub("0.28.0", "x");
    let name = w.store_engine("0.28.0", &s);
    w.store_engine_as("0.28.0", &name, &format!("{s}# tampered\n"));
    let o = w
        .cmd(&r)
        .args(["launcher", "resolve", "--json"])
        .output()
        .unwrap();
    assert_eq!(code(&o), 2);
    assert_eq!(
        w.store_names("0.28.0"),
        vec![name],
        "resolve writes nothing"
    );
}

#[test]
fn the_project_tool_directory_is_a_rule_and_is_verified_against_the_lock() {
    let w = World::new();
    let r = w.repo("r", "=0.28.0");
    let s = stub("0.28.0", "tool");
    write_exec(&r.join(".bin/spec-spine"), &s);

    // With no lock, `.bin/spec-spine` is put to the pin alone.
    let o = w
        .cmd(&r)
        .args(["launcher", "resolve", "--json"])
        .output()
        .unwrap();
    assert_eq!(code(&o), 0, "{}", stderr(&o));
    assert_eq!(json(&o)["report"]["rule"], "tool-dir");

    // With a lock, it must also be the locked executable.
    w.write_lock(&r, "0.28.0", &sha256_bytes(s.as_bytes()), "");
    let o = w
        .cmd(&r)
        .args(["launcher", "resolve", "--json"])
        .output()
        .unwrap();
    assert_eq!(code(&o), 0, "{}", stderr(&o));
    assert_eq!(json(&o)["report"]["rule"], "tool-dir");

    // A tool directory whose engine is not the locked one is refused.
    w.write_lock(&r, "0.28.0", &"b".repeat(64), "");
    let o = w.cmd(&r).arg("check").output().unwrap();
    assert_eq!(code(&o), 2);
    assert!(!stdout(&o).contains("engine"));
}

#[test]
fn a_retired_tool_directory_is_not_a_candidate() {
    let w = World::new();
    let r = w.repo("r", "=0.28.0");
    write_exec(&r.join(".tooling/bin/spec-spine"), &stub("0.28.0", "old"));
    let o = w
        .cmd(&r)
        .args(["launcher", "resolve", "--json"])
        .output()
        .unwrap();
    assert_ne!(json(&o)["report"]["rule"], "tool-dir", "{}", stdout(&o));
}

#[test]
fn the_override_rule_is_reported() {
    let w = World::new();
    let r = w.repo("r", "=0.28.0");
    let engine = w.path("elsewhere/spec-spine");
    write_exec(&engine, &stub("0.28.0", "o"));
    let o = w
        .cmd(&r)
        .env("SPEC_SPINE_ENGINE", &engine)
        .args(["launcher", "resolve", "--json"])
        .output()
        .unwrap();
    assert_eq!(code(&o), 0, "{}", stderr(&o));
    assert_eq!(json(&o)["report"]["rule"], "override");
    // A relative override is refused.
    let o = w
        .cmd(&r)
        .env("SPEC_SPINE_ENGINE", "spec-spine")
        .arg("check")
        .output()
        .unwrap();
    assert_eq!(code(&o), 2);
}

#[test]
fn outside_a_repository_only_version_and_help_are_answered() {
    let w = World::new();
    let nowhere = w.path("nowhere");
    fs::create_dir_all(&nowhere).unwrap();
    let v = w.cmd(&nowhere).arg("--version").output().unwrap();
    assert_eq!(code(&v), 0);
    assert!(
        stdout(&v).starts_with("spec-spine-launcher "),
        "{}",
        stdout(&v)
    );
    let h = w.cmd(&nowhere).arg("--help").output().unwrap();
    assert_eq!(code(&h), 0);
    let o = w.cmd(&nowhere).arg("check").output().unwrap();
    assert_eq!(code(&o), 2);
    assert!(stderr(&o).contains("no engine was run"), "{}", stderr(&o));
    let lv = w
        .cmd(&nowhere)
        .args(["launcher", "--version"])
        .output()
        .unwrap();
    assert_eq!(code(&lv), 0);
    assert_eq!(stdout(&lv), stdout(&v));
}
