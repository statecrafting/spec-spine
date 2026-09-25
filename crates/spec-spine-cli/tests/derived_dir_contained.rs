//! Spec 128 through the shipped binary: a committed `[layout] derived_dir`
//! that leaves the repository is refused before any verb writes, and a
//! derived tree inside it, including one reached through a linked root, still
//! builds.
//!
//! Every case builds its own area: `<tmp>/repo` (one spec and a
//! `spec-spine.toml`) beside `<tmp>/outside`, which holds a sentinel
//! `other.json` in each directory a derived tree would write. The whole of
//! `<tmp>` is snapshotted without following a link, so a write, a prune or a
//! creation anywhere, inside the repository or out of it, shows up as a
//! difference.
//!
//! The rows are Unix-only because the absolute case is spelled as a Unix path
//! and the linked-root control needs a symbolic link; the platform-independent
//! half of the rule is `spec-spine-types`' `tests/config.rs`.
#![cfg(unix)]

use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn code(out: &Output) -> i32 {
    out.status.code().unwrap_or(-1)
}

fn run_in(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_spec-spine"))
        .arg("--repo")
        .arg(root)
        .args(args)
        .output()
        .unwrap()
}

fn write(path: &Path, content: &[u8]) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

/// One entry under the snapshotted directory.
#[derive(Debug, PartialEq, Eq)]
enum Entry {
    Dir,
    File(Vec<u8>),
    Link(PathBuf),
}

type Tree = BTreeMap<PathBuf, Entry>;

/// Every entry under `dir`, following no link.
fn snapshot(dir: &Path) -> Tree {
    let mut out = BTreeMap::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for entry in fs::read_dir(&d).unwrap() {
            let p = entry.unwrap().path();
            let rel = p.strip_prefix(dir).unwrap().to_path_buf();
            let kind = fs::symlink_metadata(&p).unwrap().file_type();
            if kind.is_symlink() {
                out.insert(rel, Entry::Link(fs::read_link(&p).unwrap()));
            } else if kind.is_dir() {
                out.insert(rel, Entry::Dir);
                stack.push(p);
            } else {
                out.insert(rel, Entry::File(fs::read(&p).unwrap()));
            }
        }
    }
    out
}

const SENTINEL: &[u8] = b"outside the repository\n";

/// The directories under a derived root that `compile`, `index` and `attest`
/// write or prune.
const DERIVED_DIRS: &[&str] = &[
    "spec-registry/by-spec",
    "codebase-index/by-spec",
    "codebase-index/by-package",
    "attestation",
];

struct Area {
    tmp: tempfile::TempDir,
    repo: PathBuf,
    outside: PathBuf,
}

/// A one-spec corpus at `<tmp>/repo` whose `spec-spine.toml` sets
/// `derived_dir` to `layout` (with `@OUTSIDE@` replaced by the absolute path
/// of `<tmp>/outside`), and a sentinel `other.json` in each derived directory
/// under `<tmp>/outside`.
fn area(layout: &str) -> Area {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path().join("repo");
    let outside = tmp.path().join("outside");
    write(
        &repo.join("specs/001-a/spec.md"),
        b"---\nid: \"001-a\"\ntitle: \"T\"\nstatus: approved\ncreated: \"2026-09-23\"\n\
          summary: \"one\"\n---\n# T\n\n## 1. Purpose\n\nWhy.\n",
    );
    let value = layout.replace("@OUTSIDE@", outside.to_str().unwrap());
    write(
        &repo.join("spec-spine.toml"),
        format!("[layout]\nderived_dir = '{value}'\n").as_bytes(),
    );
    for dir in DERIVED_DIRS {
        write(&outside.join(dir).join("other.json"), SENTINEL);
    }
    Area { tmp, repo, outside }
}

/// The escaping rows of 1.1 that a Unix host can express.
const ESCAPING: &[&str] = &[
    "../outside",
    "@OUTSIDE@",
    "x/../../outside",
    "specs/../../outside",
];

/// Every verb that writes a derived tree, plus a read, with the arguments it
/// needs.
const VERBS: &[&[&str]] = &[
    &["compile"],
    &["index"],
    &["attest"],
    &["attest", "--spec", "001-a"],
    &["check"],
];

#[test]
fn an_escaping_derived_dir_refuses_every_verb_and_changes_nothing() {
    for layout in ESCAPING {
        for verb in VERBS {
            let a = area(layout);
            let before = snapshot(a.tmp.path());
            let out = run_in(&a.repo, verb);
            let stderr = String::from_utf8_lossy(&out.stderr);
            assert_eq!(
                code(&out),
                2,
                "derived_dir '{layout}', `{}` must exit 3; stderr:\n{stderr}",
                verb.join(" ")
            );
            assert!(
                stderr.contains("layout.derived_dir"),
                "derived_dir '{layout}', `{}`: the refusal names the key; stderr:\n{stderr}",
                verb.join(" ")
            );
            assert_eq!(
                snapshot(a.tmp.path()),
                before,
                "derived_dir '{layout}', `{}` changed the area",
                verb.join(" ")
            );
        }
    }
}

#[test]
fn a_derived_dir_inside_the_repository_builds() {
    for layout in [".derived", ".statecraft/derived", "./build/derived/"] {
        let a = area(layout);
        let outside = snapshot(&a.outside);
        for verb in [&["compile"][..], &["index"], &["attest"], &["check"]] {
            let out = run_in(&a.repo, verb);
            assert_eq!(
                code(&out),
                0,
                "derived_dir '{layout}', `{}`; stderr:\n{}",
                verb.join(" "),
                String::from_utf8_lossy(&out.stderr)
            );
        }
        let derived = a.repo.join(layout.trim_start_matches("./"));
        assert!(
            derived.join("spec-registry/by-spec/001-a.json").is_file(),
            "derived_dir '{layout}': the shard is inside the repository"
        );
        assert_eq!(snapshot(&a.outside), outside, "derived_dir '{layout}'");
    }
}

#[test]
fn a_repository_reached_through_a_link_still_builds() {
    let a = area(".statecraft/derived");
    let link = a.tmp.path().join("linked-repo");
    symlink(&a.repo, &link).unwrap();
    let outside = snapshot(&a.outside);
    for verb in [&["compile"][..], &["index"], &["attest"], &["check"]] {
        let out = run_in(&link, verb);
        assert_eq!(
            code(&out),
            0,
            "`{}` through a linked root; stderr:\n{}",
            verb.join(" "),
            String::from_utf8_lossy(&out.stderr)
        );
    }
    assert!(
        a.repo
            .join(".statecraft/derived/spec-registry/by-spec/001-a.json")
            .is_file()
    );
    assert_eq!(snapshot(&a.outside), outside);
}
