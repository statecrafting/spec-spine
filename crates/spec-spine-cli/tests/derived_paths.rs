// Spec: specs/126-a-derived-file-stays-in-its-directory/spec.md
//! Spec 126 through the shipped binary: a spec id that is not one plain file
//! name cannot make `compile`, `index` or `attest --spec` write, prune or
//! create a file, inside the repository or out of it.
//!
//! Every case builds its own fixture at `<tmp>/repo`, so `<tmp>` itself is
//! outside the repository and a victim placed there stands for any file the
//! user can write. The whole of `<tmp>` is snapshotted, directories included,
//! so an escape from the repository shows up as a difference too. Every
//! absolute id is built from the same `<tmp>` that is snapshotted.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
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

fn write(path: &Path, content: &[u8]) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

/// `id` as a YAML double-quoted scalar, so an id carrying a backslash, a quote
/// or a NUL reaches the parser as exactly those characters.
fn yaml_quoted(id: &str) -> String {
    let mut out = String::from("\"");
    for c in id.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\0' => out.push_str("\\0"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// A spec whose directory is `dir` and whose frontmatter `id` is `id`. The two
/// differ for every hostile id, which is `V-001`: reported, and not a defence.
fn write_spec(root: &Path, dir: &str, id: &str) {
    write(
        &root.join(format!("specs/{dir}/spec.md")),
        format!(
            "---\nid: {}\ntitle: \"T\"\nstatus: approved\ncreated: \"2026-09-23\"\n\
             summary: \"s\"\n---\n# T\n\n## 1. Purpose\n\nWhy.\n",
            yaml_quoted(id)
        )
        .as_bytes(),
    );
}

/// Every entry under `dir`: a file maps to its bytes, a directory to `None`.
type Tree = BTreeMap<PathBuf, Option<Vec<u8>>>;

fn snapshot(dir: &Path) -> Tree {
    let mut out = BTreeMap::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for entry in fs::read_dir(&d).unwrap() {
            let p = entry.unwrap().path();
            let rel = p.strip_prefix(dir).unwrap().to_path_buf();
            if p.is_dir() {
                out.insert(rel, None);
                stack.push(p);
            } else {
                out.insert(rel, Some(fs::read(&p).unwrap()));
            }
        }
    }
    out
}

const VICTIM_IN_REPO: &[u8] = b"{\"name\":\"victim-in-repo\"}\n";
const VICTIM_OUTSIDE: &[u8] = b"{\"victim\":\"outside the repository\"}\n";

/// A one-spec corpus at `<tmp>/repo` with two victims, the repository's own
/// `package.json` and a file beside the repository. With `built`, its three
/// derived trees are already written, which is also the positive case: a valid
/// id compiles, indexes and attests.
fn fixture(built: bool) -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path().join("repo");
    write_spec(&repo, "001-a", "001-a");
    write(&repo.join("package.json"), VICTIM_IN_REPO);
    write(&tmp.path().join("victim.json"), VICTIM_OUTSIDE);
    if built {
        for args in [&["compile"][..], &["index"], &["attest", "--spec", "001-a"]] {
            let out = run_in(&repo, args);
            assert_eq!(
                code(&out),
                0,
                "a valid id: `{}` succeeds; stderr: {}",
                args.join(" "),
                String::from_utf8_lossy(&out.stderr)
            );
        }
        for path in [
            ".derived/spec-registry/by-spec/001-a.json",
            ".derived/codebase-index/by-spec/001-a.json",
            ".derived/attestation/by-spec/001-a.json",
        ] {
            assert!(repo.join(path).is_file(), "a valid id writes {path}");
        }
    }
    (tmp, repo)
}

/// A hostile id and what it reached before spec 126. Each relative one is
/// relative to a `by-spec/` directory three levels below the repository root
/// (`.derived/<tree>/by-spec/`), the same depth for the registry, the index and
/// the attestation.
struct Hostile {
    id: String,
    what: &'static str,
    /// Whether `attest --spec <id>` can name it: an argument cannot carry a
    /// NUL, and the empty argument resolves nothing.
    addressable: bool,
}

fn hostile_ids(outer: &Path) -> Vec<Hostile> {
    let h = |id: String, what, addressable| Hostile {
        id,
        what,
        addressable,
    };
    vec![
        h(
            "../../../package".into(),
            "relative traversal onto the repository's package.json",
            true,
        ),
        h(
            "../../../../victim".into(),
            "relative traversal onto a file outside the repository",
            true,
        ),
        h(
            outer.join("absolute").to_string_lossy().into_owned(),
            "an absolute path outside the repository",
            true,
        ),
        h(String::new(), "an empty id (a hidden `.json`)", false),
        h(".hidden".into(), "a leading dot", true),
        h("a\\b".into(), "a backslash separator", true),
        h("C:evil".into(), "a Windows drive prefix", true),
        h("a:b".into(), "a colon", true),
        h("a\0b".into(), "a NUL", false),
    ]
}

/// The refusal is exit 3, says nothing was written, names the refused file
/// name, and points at the read-only command that reports the id (3.4). The
/// file name is matched as the refusal quotes it, so this also proves the run
/// reached the name check rather than failing earlier for another reason.
fn assert_refused(out: &Output, id: &str, what: &str) {
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(code(out), 2, "{what}: exit 2; stderr: {stderr}");
    let quoted = format!("{:?}", format!("{id}.json"));
    for needle in [
        quoted.as_str(),
        "nothing was written",
        "spec-spine compile --check",
    ] {
        assert!(
            stderr.contains(needle),
            "{what}: the refusal carries {needle:?}; stderr: {stderr}"
        );
    }
}

/// 3.2: `compile` and `index` refuse before touching anything. The tree is
/// already built, so this is also the prune case: the committed shards, the
/// attestation and `build-meta.json` are all in the snapshot, and a refused
/// run that pruned, rewrote or added any of them would change it.
#[test]
fn compile_and_index_refuse_an_id_that_is_not_a_file_name() {
    for verb in ["compile", "index"] {
        for case in 0..hostile_ids(Path::new("/")).len() {
            let (tmp, repo) = fixture(true);
            let h = hostile_ids(tmp.path()).swap_remove(case);
            write_spec(&repo, "002-b", &h.id);
            let before = snapshot(tmp.path());
            let out = run_in(&repo, &[verb]);
            let what = format!("`{verb}` with id {:?} ({})", h.id, h.what);
            assert_refused(&out, &h.id, &what);
            assert_eq!(snapshot(tmp.path()), before, "{what} changed the tree");
            assert_eq!(fs::read(repo.join("package.json")).unwrap(), VICTIM_IN_REPO);
            assert_eq!(
                fs::read(tmp.path().join("victim.json")).unwrap(),
                VICTIM_OUTSIDE
            );
        }
    }
}

/// 3.2 on a tree never built: nothing at all is created, not the `by-spec/`
/// directory `sync_dir` would have made and not the artifact root the CLI used
/// to create before calling it (D-3, closed by D-5). The comparison is exact:
/// the snapshot after the refused run equals the one before it.
#[test]
fn a_refused_first_build_creates_nothing() {
    for verb in ["compile", "index"] {
        for case in 0..hostile_ids(Path::new("/")).len() {
            let (tmp, repo) = fixture(false);
            let h = hostile_ids(tmp.path()).swap_remove(case);
            write_spec(&repo, "002-b", &h.id);
            let before = snapshot(tmp.path());
            let out = run_in(&repo, &[verb]);
            let what = format!("first `{verb}` with id {:?} ({})", h.id, h.what);
            assert_refused(&out, &h.id, &what);
            assert!(!repo.join(".derived").exists(), "{what} created .derived");
            assert_eq!(snapshot(tmp.path()), before, "{what} changed the tree");
        }
    }
}

/// The other side of D-5: with the premature `create_dir_all` gone, a valid
/// first build still creates every directory and file it did before, because
/// `sync_dir` creates the artifact root on its way to the shards.
#[test]
fn a_valid_first_build_still_creates_its_whole_tree() {
    let (_tmp, repo) = fixture(true);
    for dir in [
        ".derived/spec-registry/by-spec",
        ".derived/codebase-index/by-spec",
        ".derived/codebase-index/by-package",
        ".derived/attestation/by-spec",
    ] {
        assert!(repo.join(dir).is_dir(), "a valid first build creates {dir}");
    }
    assert!(
        repo.join(".derived/spec-registry/build-meta.json")
            .is_file(),
        "compile still writes build-meta.json beside the shards"
    );
    // And a second, unchanged run is still clean over the tree it made.
    assert_eq!(
        code(&run_in(&repo, &["check"])),
        0,
        "the first build is fresh"
    );
}

/// 3.3: `attest --spec` resolves a hostile id from the corpus, because the
/// corpus declares it, and must still refuse to write it, in prose and in
/// JSON, and with `--sign` must write no seal either.
#[test]
fn attest_spec_refuses_an_id_that_is_not_a_file_name() {
    for case in 0..hostile_ids(Path::new("/")).len() {
        for built in [true, false] {
            let (tmp, repo) = fixture(built);
            let h = hostile_ids(tmp.path()).swap_remove(case);
            if !h.addressable {
                continue;
            }
            write_spec(&repo, "002-b", &h.id);
            let key = tmp.path().join("signing.key");
            write(&key, &[7u8; 32]);
            let key = key.to_string_lossy().into_owned();
            let before = snapshot(tmp.path());
            let what = format!("`attest --spec {:?}` ({}, built: {built})", h.id, h.what);

            let out = run_in(&repo, &["attest", "--spec", &h.id]);
            assert_refused(&out, &h.id, &what);
            assert_eq!(snapshot(tmp.path()), before, "{what} changed the tree");

            let out = run_in(&repo, &["attest", "--spec", &h.id, "--sign", "--key", &key]);
            assert_refused(&out, &h.id, &format!("{what} --sign"));
            assert_eq!(
                snapshot(tmp.path()),
                before,
                "{what} --sign changed the tree"
            );

            // JSON: the same refusal, as the one envelope on stdout.
            let out = run_in(&repo, &["attest", "--spec", &h.id, "--json"]);
            assert_eq!(code(&out), 2, "{what} --json");
            let v: serde_json::Value = serde_json::from_slice(&out.stdout)
                .unwrap_or_else(|e| panic!("{what} --json: stdout is one envelope: {e}"));
            assert_ne!(v["outcome"], "ok", "{what} --json");
            assert_eq!(v["exitCode"], 2, "{what} --json");
            let message = v["error"]["message"].as_str().unwrap_or_default();
            assert!(
                message.contains("nothing was written")
                    && message.contains(&format!("{:?}", format!("{}.json", h.id))),
                "{what} --json: the envelope carries the prose refusal: {message}"
            );
            assert_eq!(
                snapshot(tmp.path()),
                before,
                "{what} --json changed the tree"
            );
        }
    }
}

/// The guard is not refusing everything: an id that fails `V-012` but is a
/// plain file name still gets its shard and its attestation (3.1), exactly as
/// before, and `compile` still exits 1 for the validation failure.
#[test]
fn an_invalid_but_plain_id_keeps_its_behavior() {
    let (_tmp, repo) = fixture(true);
    write_spec(&repo, "002-b", "002-B");
    let out = run_in(&repo, &["compile"]);
    assert_eq!(code(&out), 1, "V-012 is still exit 1");
    assert!(String::from_utf8_lossy(&out.stderr).contains("V-012"));
    assert!(
        repo.join(".derived/spec-registry/by-spec/002-B.json")
            .is_file()
    );
    assert!(
        repo.join(".derived/spec-registry/by-spec/001-a.json")
            .is_file()
    );
    assert_eq!(code(&run_in(&repo, &["index"])), 0);
    assert!(
        repo.join(".derived/codebase-index/by-spec/002-B.json")
            .is_file()
    );
    assert_eq!(code(&run_in(&repo, &["attest", "--spec", "002-B"])), 0);
    assert!(
        repo.join(".derived/attestation/by-spec/002-B.json")
            .is_file()
    );
}
