//! Spec 127 through the shipped binary: `compile`, `index` and `attest` write
//! nothing through a symbolic link below the repository root, refuse a spec id
//! that names a reserved Windows device, and still work in a repository that
//! is itself reached through a link.
//!
//! Every case builds its own fixture at `<tmp>/repo` beside `<tmp>/outside`,
//! so `outside/` stands for any directory the user can write. The whole of
//! `<tmp>` is snapshotted without following a link (a link is recorded as its
//! target), so a write, a prune or a creation anywhere, inside the repository
//! or out of it, shows up as a difference.
//!
//! The link cases are Unix-only: creating a link on Windows needs a privilege
//! CI does not have, and spec 127 claims nothing there (4).
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

fn write_spec(root: &Path, dir: &str, id: &str, summary: &str) {
    write(
        &root.join(format!("specs/{dir}/spec.md")),
        format!(
            "---\nid: \"{id}\"\ntitle: \"T\"\nstatus: approved\ncreated: \"2026-09-23\"\n\
             summary: \"{summary}\"\n---\n# T\n\n## 1. Purpose\n\nWhy.\n"
        )
        .as_bytes(),
    );
}

/// One entry under the snapshotted directory.
#[derive(Debug, PartialEq, Eq)]
enum Entry {
    Dir,
    File(Vec<u8>),
    Link(PathBuf),
}

type Tree = BTreeMap<PathBuf, Entry>;

/// Every entry under `dir`, following no link: a linked directory is recorded
/// as its target and not descended into, so the walk sees each real file once.
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

const OUTSIDE: &[u8] = b"outside the repository\n";

struct Fixture {
    tmp: tempfile::TempDir,
    repo: PathBuf,
    outside: PathBuf,
}

/// A one-spec corpus at `<tmp>/repo`, an `outside/` directory holding
/// `other.json`, and, with `layout`, a `spec-spine.toml` setting that derived
/// directory. With `built`, `compile`, `index` and `attest --spec` have run
/// once, which is also the positive control: a clean tree builds.
fn fixture(built: bool, layout: Option<&str>) -> Fixture {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path().join("repo");
    let outside = tmp.path().join("outside");
    write_spec(&repo, "001-a", "001-a", "one");
    if let Some(dir) = layout {
        write(
            &repo.join("spec-spine.toml"),
            format!("[layout]\nderived_dir = \"{dir}\"\n").as_bytes(),
        );
    }
    write(&outside.join("other.json"), OUTSIDE);
    let f = Fixture { tmp, repo, outside };
    if built {
        f.build();
    }
    f
}

impl Fixture {
    fn derived(&self) -> PathBuf {
        self.repo.join(".derived")
    }

    fn build(&self) {
        for args in [&["compile"][..], &["index"], &["attest", "--spec", "001-a"]] {
            let out = run_in(&self.repo, args);
            assert_eq!(
                code(&out),
                0,
                "a clean tree: `{}` succeeds; stderr: {}",
                args.join(" "),
                String::from_utf8_lossy(&out.stderr)
            );
        }
    }

    /// Change the corpus so the next `compile` and `index` would rewrite the
    /// existing shard: a refused run that still wrote it shows up.
    fn touch_corpus(&self) {
        write_spec(&self.repo, "001-a", "001-a", "two");
    }

    /// Replace `rel` (under the repository) with a link to `target`, removing
    /// whatever was there.
    fn link(&self, rel: &str, target: &Path) {
        let at = self.repo.join(rel);
        if let Ok(md) = fs::symlink_metadata(&at) {
            if md.is_dir() {
                fs::remove_dir_all(&at).unwrap();
            } else {
                fs::remove_file(&at).unwrap();
            }
        }
        fs::create_dir_all(at.parent().unwrap()).unwrap();
        symlink(target, &at).unwrap();
    }

    fn key(&self) -> String {
        let key = self.tmp.path().join("signing.key");
        if !key.exists() {
            write(&key, &[7u8; 32]);
        }
        key.to_string_lossy().into_owned()
    }
}

/// Run `args`, and assert the refusal spec 127 3.2 prescribes: exit 3, the
/// linked (or wrong-kind) path named relative to the repository, nothing
/// written, and not one byte of `<tmp>` changed.
fn assert_refused(f: &Fixture, args: &[&str], rel: &str, what: &str) {
    let before = snapshot(f.tmp.path());
    let out = run_in(&f.repo, args);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let what = format!("{what}: `{}`", args.join(" "));
    assert_eq!(code(&out), 3, "{what}: exit 3; stderr: {stderr}");
    for needle in [rel, "nothing was written", "spec 127"] {
        assert!(
            stderr.contains(needle),
            "{what}: the refusal carries {needle:?}; stderr: {stderr}"
        );
    }
    assert_eq!(
        snapshot(f.tmp.path()),
        before,
        "{what}: the refused run changed the tree"
    );
}

/// B-10 cases 1 and 5: a linked shard or attestation file. Before spec 127 the
/// outside file was overwritten with shard JSON and the verb exited 0.
#[test]
fn a_linked_shard_or_attestation_file_refuses() {
    for (verb, rel) in [
        (&["compile"][..], "spec-registry/by-spec/001-a.json"),
        (&["index"], "codebase-index/by-spec/001-a.json"),
        (
            &["attest", "--spec", "001-a"],
            "attestation/by-spec/001-a.json",
        ),
    ] {
        let f = fixture(true, None);
        f.touch_corpus();
        write(&f.outside.join("victim.json"), OUTSIDE);
        f.link(&format!(".derived/{rel}"), &f.outside.join("victim.json"));
        assert_refused(&f, verb, &format!(".derived/{rel}"), "a linked file");
    }
}

/// B-10 case 6: a linked seal. The attestation beside it is a real file, and
/// the refusal still comes before it is rewritten.
#[test]
fn a_linked_seal_refuses_before_the_attestation_is_written() {
    let f = fixture(true, None);
    f.touch_corpus();
    write(&f.outside.join("seal.txt"), OUTSIDE);
    f.link(
        ".derived/attestation/by-spec/001-a.sig",
        &f.outside.join("seal.txt"),
    );
    let key = f.key();
    assert_refused(
        &f,
        &["attest", "--spec", "001-a", "--sign", "--key", &key],
        ".derived/attestation/by-spec/001-a.sig",
        "a linked seal",
    );
}

/// B-10 cases 2 and 4: a linked `by-spec/` directory. Before spec 127 the
/// shard was written into `outside/` and `outside/other.json` was pruned.
#[test]
fn a_linked_shard_directory_refuses_and_prunes_nothing() {
    for (verb, rel) in [
        (&["compile"][..], ".derived/spec-registry/by-spec"),
        (&["index"], ".derived/codebase-index/by-spec"),
        (&["index"], ".derived/codebase-index/by-package"),
        (
            &["attest", "--spec", "001-a"],
            ".derived/attestation/by-spec",
        ),
    ] {
        let f = fixture(true, None);
        f.touch_corpus();
        f.link(rel, &f.outside);
        assert_refused(&f, verb, rel, "a linked directory");
        assert_eq!(fs::read(f.outside.join("other.json")).unwrap(), OUTSIDE);
    }
}

/// B-10 case 3, and the managed layout's extra ancestor: the derived root, an
/// artifact root, or `.statecraft` above `.statecraft/derived`, is a link.
#[test]
fn a_linked_ancestor_refuses() {
    for (layout, link, verbs) in [
        (
            None,
            ".derived",
            &[&["compile"][..], &["index"], &["attest"]][..],
        ),
        (None, ".derived/spec-registry", &[&["compile"][..]]),
        (None, ".derived/codebase-index", &[&["index"][..]]),
        (
            None,
            ".derived/attestation",
            &[&["attest", "--snapshot"][..]],
        ),
        (
            Some(".statecraft/derived"),
            ".statecraft",
            &[&["compile"][..], &["index"], &["attest", "--spec", "001-a"]],
        ),
    ] {
        for verb in verbs {
            // Built and unbuilt: an existing tree moved behind a link, and a
            // link waiting for a first build.
            for built in [true, false] {
                let f = fixture(built, layout);
                f.touch_corpus();
                if built {
                    fs::rename(f.repo.join(link), f.outside.join("moved")).unwrap();
                    f.link(link, &f.outside.join("moved"));
                } else {
                    f.link(link, &f.outside);
                }
                assert_refused(
                    &f,
                    verb,
                    link,
                    &format!("a linked ancestor (built: {built})"),
                );
            }
        }
    }
}

/// 3.3: the preflight covers every output of the run, so a link at the last
/// thing a verb writes or removes still refuses before the first shard moves.
/// `touch_corpus` makes every shard due for a rewrite, so a partial run shows.
#[test]
fn a_link_at_any_later_output_refuses_before_the_first_write() {
    // compile: build-meta.json and the legacy registry.json it removes.
    for rel in [
        "spec-registry/build-meta.json",
        "spec-registry/registry.json",
    ] {
        let f = fixture(true, None);
        f.touch_corpus();
        write(&f.outside.join("victim.json"), OUTSIDE);
        f.link(&format!(".derived/{rel}"), &f.outside.join("victim.json"));
        assert_refused(&f, &["compile"], &format!(".derived/{rel}"), "compile");
    }
    // index: the legacy index.json it removes, and slices.json, which it
    // writes when slices are configured and removes when they are not.
    for (slices, rel) in [
        (false, "codebase-index/index.json"),
        (true, "codebase-index/slices.json"),
        (false, "codebase-index/slices.json"),
    ] {
        let f = fixture(true, None);
        if slices {
            write(
                &f.repo.join("spec-spine.toml"),
                b"[index.slices]\ncore = [\"specs/**\"]\n",
            );
        }
        f.touch_corpus();
        write(&f.outside.join("victim.json"), OUTSIDE);
        f.link(&format!(".derived/{rel}"), &f.outside.join("victim.json"));
        assert_refused(
            &f,
            &["index"],
            &format!(".derived/{rel}"),
            &format!("index (slices: {slices})"),
        );
    }
    // attest: the corpus attestation and the snapshot.
    for (verb, rel) in [
        (&["attest"][..], "attestation/attestation.json"),
        (&["attest", "--snapshot"], "attestation/snapshot.json"),
    ] {
        let f = fixture(true, None);
        write(&f.outside.join("victim.json"), OUTSIDE);
        f.link(&format!(".derived/{rel}"), &f.outside.join("victim.json"));
        assert_refused(&f, verb, &format!(".derived/{rel}"), "attest");
    }
}

/// 3.3: an entry the run would prune is checked too. A linked stray shard is
/// refused, not removed, so the hostile tree is reported rather than repaired.
#[test]
fn a_linked_entry_due_for_pruning_refuses() {
    for rel in [
        ".derived/spec-registry/by-spec/zzz.json",
        ".derived/codebase-index/by-spec/zzz.json",
    ] {
        let verb = if rel.contains("registry") {
            "compile"
        } else {
            "index"
        };
        let f = fixture(true, None);
        f.touch_corpus();
        f.link(rel, &f.outside.join("other.json"));
        assert_refused(&f, &[verb], rel, "a linked stray shard");
    }
}

/// 3.3: `index` writes two batches, and a link in the second refuses before
/// the first is written: the real `by-spec/` keeps its old shard.
#[test]
fn index_checks_both_batches_before_writing_either() {
    let f = fixture(true, None);
    let shard = f.derived().join("codebase-index/by-spec/001-a.json");
    let old = fs::read(&shard).unwrap();
    f.touch_corpus();
    f.link(".derived/codebase-index/by-package", &f.outside);
    assert_refused(
        &f,
        &["index"],
        ".derived/codebase-index/by-package",
        "mixed tree",
    );
    assert_eq!(fs::read(&shard).unwrap(), old);
}

/// 3.4: an existing component of the wrong kind is refused in the same
/// preflight, so it cannot fail halfway through a run either.
#[test]
fn a_component_of_the_wrong_kind_refuses_before_anything() {
    let f = fixture(true, None);
    f.touch_corpus();
    let meta = f.derived().join("spec-registry/build-meta.json");
    fs::remove_file(&meta).unwrap();
    fs::create_dir(&meta).unwrap();
    assert_refused(
        &f,
        &["compile"],
        ".derived/spec-registry/build-meta.json",
        "a directory where a file is written",
    );

    let f = fixture(true, None);
    f.touch_corpus();
    let by_spec = f.derived().join("codebase-index/by-spec");
    fs::remove_dir_all(&by_spec).unwrap();
    write(&by_spec, b"not a directory\n");
    assert_refused(
        &f,
        &["index"],
        ".derived/codebase-index/by-spec",
        "a file where a directory is expected",
    );
}

/// Positive control: on a tree with no link, a first build creates, a rebuild
/// rewrites, and a removed spec's shards are pruned, exactly as before.
#[test]
fn a_clean_tree_still_builds_rebuilds_and_prunes() {
    let f = fixture(true, None);
    write_spec(&f.repo, "002-b", "002-b", "b");
    f.build();
    for tree in ["spec-registry", "codebase-index"] {
        assert!(f.derived().join(tree).join("by-spec/002-b.json").is_file());
    }
    fs::remove_dir_all(f.repo.join("specs/002-b")).unwrap();
    f.touch_corpus();
    f.build();
    for tree in ["spec-registry", "codebase-index"] {
        assert!(!f.derived().join(tree).join("by-spec/002-b.json").exists());
    }
    let key = f.key();
    let out = run_in(
        &f.repo,
        &["attest", "--spec", "001-a", "--sign", "--key", &key],
    );
    assert_eq!(code(&out), 0, "{}", String::from_utf8_lossy(&out.stderr));
    assert!(f.derived().join("attestation/by-spec/001-a.sig").is_file());
    assert_eq!(
        code(&run_in(&f.repo, &["check"])),
        0,
        "the rebuilt tree is fresh"
    );
}

/// 3.1: the repository root and its ancestors are not checked. A repository
/// reached through a linked path (as `$TMPDIR` is on macOS, and many home
/// directories are) builds, rebuilds and attests as it always did.
#[test]
fn a_repository_reached_through_a_link_still_builds() {
    let f = fixture(false, None);
    let via = f.tmp.path().join("via");
    symlink(&f.repo, &via).unwrap();
    let key = f.key();
    for _ in 0..2 {
        for args in [
            &["compile"][..],
            &["index"],
            &["attest"],
            &["attest", "--spec", "001-a", "--sign", "--key", &key],
        ] {
            let out = run_in(&via, args);
            assert_eq!(
                code(&out),
                0,
                "through a linked root: `{}`; stderr: {}",
                args.join(" "),
                String::from_utf8_lossy(&out.stderr)
            );
        }
        f.touch_corpus();
    }
    assert!(
        f.derived()
            .join("spec-registry/by-spec/001-a.json")
            .is_file()
    );
    assert!(f.derived().join("attestation/by-spec/001-a.sig").is_file());

    // A linked ancestor of the root, one level further up.
    let up = f.tmp.path().join("up");
    symlink(f.tmp.path(), &up).unwrap();
    let out = run_in(&up.join("repo"), &["compile"]);
    assert_eq!(code(&out), 0, "{}", String::from_utf8_lossy(&out.stderr));
}

/// B-13 through the binary (3.5): an id naming a reserved Windows device
/// refuses on every platform, before anything is written, whether or not
/// the tree was built.
#[test]
fn an_id_naming_a_reserved_device_refuses_everywhere() {
    for id in [
        "CON",
        "nul",
        "Aux.tar",
        "COM1",
        "lpt9",
        "CON ",
        "PRN.",
        "COM\u{b9}",
    ] {
        for built in [true, false] {
            let f = fixture(built, None);
            write_spec(&f.repo, "002-b", id, "b");
            let quoted = format!("{:?}", format!("{id}.json"));
            let mut verbs: Vec<Vec<&str>> = vec![vec!["compile"], vec!["index"]];
            if built {
                verbs.push(vec!["attest", "--spec", id]);
            }
            for verb in &verbs {
                let before = snapshot(f.tmp.path());
                let out = run_in(&f.repo, verb);
                let stderr = String::from_utf8_lossy(&out.stderr);
                let what = format!("id {id:?} (built: {built}): `{}`", verb.join(" "));
                assert_eq!(code(&out), 3, "{what}; stderr: {stderr}");
                assert!(stderr.contains(&quoted), "{what}: names {quoted}: {stderr}");
                assert!(stderr.contains("nothing was written"), "{what}: {stderr}");
                assert_eq!(snapshot(f.tmp.path()), before, "{what} changed the tree");
            }
        }
    }
}
