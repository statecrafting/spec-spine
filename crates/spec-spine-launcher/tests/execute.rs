// Spec: specs/188-a-repository-pin-selects-the-engine/spec.md
//! V-1, execution: arguments, working directory and exit status reach the
//! engine and come back unchanged, and the launcher never runs itself.

mod common;

use std::fs;

use common::*;

const ARGS: [&str; 4] = ["check", "--json", "a b", "--fail-on-warn"];

fn engine_world() -> (World, std::path::PathBuf) {
    let w = World::new();
    let r = w.repo("r", "=0.28.0");
    w.store_engine("0.28.0", &stub("0.28.0", "x"));
    (w, r)
}

#[test]
fn arguments_working_directory_and_exit_status_are_preserved() {
    let (w, r) = engine_world();
    let out = w.path("out.txt");
    let o = w
        .cmd(&r)
        .env("STUB_OUT", &out)
        .env("STUB_EXIT", "1")
        .args(ARGS)
        .output()
        .unwrap();
    assert_eq!(code(&o), 1, "the engine exits 1, so the invocation exits 1");
    assert_eq!(stdout(&o), "engine 0.28.0\n");

    let seen = read_kv(&out);
    let want: Vec<String> = ARGS.iter().map(|a| format!("arg={a}")).collect();
    assert_eq!(&seen[3..], want.as_slice(), "{seen:?}");
    let cwd = fs::canonicalize(&r).unwrap();
    assert_eq!(seen[1], format!("cwd={}", cwd.display()));
}

#[test]
fn other_exit_statuses_pass_through_too() {
    let (w, r) = engine_world();
    for status in ["0", "2", "3", "4", "77"] {
        let o = w
            .cmd(&r)
            .env("STUB_EXIT", status)
            .arg("check")
            .output()
            .unwrap();
        assert_eq!(code(&o).to_string(), status);
    }
}

#[test]
fn a_repository_found_in_an_ancestor_is_named_to_the_engine_and_the_cwd_is_kept() {
    let (w, r) = engine_world();
    let sub = r.join("crates/deep");
    fs::create_dir_all(&sub).unwrap();
    let out = w.path("out.txt");
    let o = w
        .cmd(&sub)
        .env("STUB_OUT", &out)
        .args(ARGS)
        .output()
        .unwrap();
    assert_eq!(code(&o), 0, "{}", stderr(&o));
    let seen = read_kv(&out);
    assert_eq!(seen[3], "arg=--repo");
    assert_eq!(seen[4], format!("arg={}", r.display()));
    assert_eq!(seen[5], "arg=check");
    assert_eq!(
        seen[1],
        format!("cwd={}", fs::canonicalize(&sub).unwrap().display())
    );
}

#[test]
fn an_explicit_repo_flag_is_passed_through_untouched() {
    let (w, r) = engine_world();
    let elsewhere = w.path("elsewhere");
    fs::create_dir_all(&elsewhere).unwrap();
    let out = w.path("out.txt");
    let o = w
        .cmd(&elsewhere)
        .env("STUB_OUT", &out)
        .arg("--repo")
        .arg(&r)
        .arg("check")
        .output()
        .unwrap();
    assert_eq!(code(&o), 0, "{}", stderr(&o));
    let seen = read_kv(&out);
    assert_eq!(
        &seen[3..],
        [
            "arg=--repo".to_string(),
            format!("arg={}", r.display()),
            "arg=check".into()
        ]
    );
}

#[test]
fn the_child_is_told_it_was_launched() {
    let (w, r) = engine_world();
    let out = w.path("out.txt");
    let v = w.cmd(&r).args(["launcher", "--version"]).output().unwrap();
    let version = stdout(&v).trim().rsplit(' ').next().unwrap().to_string();
    w.cmd(&r)
        .env("STUB_OUT", &out)
        .arg("check")
        .output()
        .unwrap();
    assert_eq!(read_kv(&out)[2], format!("launched={version}"));
}

#[test]
fn recursion_is_refused_with_exit_4() {
    let (w, r) = engine_world();
    let out = w.path("out.txt");
    let o = w
        .cmd(&r)
        .env("SPEC_SPINE_LAUNCHED", "0.1.0")
        .env("STUB_OUT", &out)
        .arg("check")
        .output()
        .unwrap();
    assert_eq!(code(&o), 4, "{}", stderr(&o));
    assert!(stderr(&o).contains("SPEC_SPINE_LAUNCHED"), "{}", stderr(&o));
    assert!(!out.exists(), "the engine must not run");
}

#[test]
fn an_override_naming_a_launcher_is_refused_with_exit_4() {
    let (w, r) = engine_world();
    let out = w.path("out.txt");
    // The real launcher binary.
    let o = w
        .cmd(&r)
        .env("SPEC_SPINE_ENGINE", LAUNCHER)
        .env("STUB_OUT", &out)
        .arg("check")
        .output()
        .unwrap();
    assert_eq!(code(&o), 4, "{}", stderr(&o));
    assert!(stderr(&o).contains("launcher"), "{}", stderr(&o));

    // And anything that answers `launcher --version` as one.
    let fake = w.path("fake/spec-spine");
    write_exec(
        &fake,
        "#!/bin/sh\nif [ \"$1\" = \"launcher\" ]; then echo 'spec-spine-launcher 9.9.9'; exit 0; fi\necho 'spec-spine 0.28.0'\n",
    );
    let o = w
        .cmd(&r)
        .env("SPEC_SPINE_ENGINE", &fake)
        .env("STUB_OUT", &out)
        .arg("check")
        .output()
        .unwrap();
    assert_eq!(code(&o), 4, "{}", stderr(&o));
    assert!(!out.exists());
}

#[test]
fn a_store_entry_that_is_a_launcher_is_refused_with_exit_4() {
    let w = World::new();
    let r = w.repo("r", "=0.28.0");
    let fake = "#!/bin/sh\nif [ \"$1\" = \"launcher\" ]; then echo 'spec-spine-launcher 9.9.9'; exit 0; fi\necho 'spec-spine 0.28.0'\n";
    w.store_engine("0.28.0", fake);
    let o = w.cmd(&r).arg("check").output().unwrap();
    assert_eq!(code(&o), 4, "{}", stderr(&o));
}
