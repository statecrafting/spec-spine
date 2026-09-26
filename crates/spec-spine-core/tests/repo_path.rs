//! Spec 144: a repository path is one validated type.
//!
//! The configuration's layout roots (§3.2) and the read-side link rule (§3.4).
//! Plan paths (§3.3) are in `retire.rs`, beside the WF-8 cases they extend.

use std::fs;
use std::path::Path;

use spec_spine_core::{check_index_freshness, compile, index};
use spec_spine_types::{Config, load_config};

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, content).unwrap();
}

/// §3.2: every layout root refuses a path that leaves the repository or is a
/// Windows hazard, with exit 2, and accepts a plain relative path.
#[test]
fn every_layout_root_follows_the_one_rule() {
    for key in ["specs_dir", "standards_dir", "derived_dir"] {
        for bad in [
            "../x",
            "/etc",
            "C:foo",
            "a\\b",
            "con/x",
            "a/nul.txt",
            "x./y",
        ] {
            let toml = format!("[layout]\n{key} = '{bad}'\n");
            let err = load_config(&toml).unwrap_err();
            assert_eq!(err.exit_code(), 2, "{key} = {bad}: {err}");
            assert!(format!("{err}").contains(key), "{key} = {bad}: {err}");
        }
        let ok = format!("[layout]\n{key} = 'docs/governed'\n");
        assert!(load_config(&ok).is_ok(), "{key}");
    }
    // state_dir gains the Windows forms too.
    for bad in ["C:state", "st\\ate", "prn"] {
        let err = load_config(&format!("[layout]\nstate_dir = '{bad}'\n")).unwrap_err();
        assert_eq!(err.exit_code(), 2, "state_dir = {bad}: {err}");
    }
}

fn corpus(root: &Path) {
    write(
        root,
        "specs/001-a/spec.md",
        "---\nid: \"001-a\"\ntitle: \"A\"\nstatus: approved\ncreated: \"2026-09-25\"\nimplementation: complete\nsummary: \"s\"\nestablishes:\n  - \"src/a.rs\"\n---\n# 001-a\n",
    );
    write(root, "src/a.rs", "pub fn a() {}\n");
}

/// §3.4: a link resolving outside the repository refuses compile and index,
/// exit 2, naming the link; a link inside it, and a dangling one, do not.
#[cfg(unix)]
#[test]
fn a_link_leaving_the_repository_refuses_the_read() {
    use std::os::unix::fs::symlink;
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path().join("repo");
    let outside = tmp.path().join("outside");
    corpus(&repo);
    write(&outside, "secret.rs", "pub fn secret() {}\n");
    let cfg = Config::default();

    symlink(repo.join("src/a.rs"), repo.join("src/inside.rs")).unwrap();
    symlink(repo.join("src/gone.rs"), repo.join("src/dangling.rs")).unwrap();
    assert!(
        compile(&cfg, &repo).is_ok(),
        "a link inside, and a dangling one, are fine"
    );
    assert!(index(&cfg, &repo).is_ok());

    symlink(outside.join("secret.rs"), repo.join("src/leak.rs")).unwrap();
    for err in [
        compile(&cfg, &repo).err().expect("compile refuses"),
        index(&cfg, &repo).err().expect("index refuses"),
    ] {
        assert_eq!(err.exit_code(), 2, "{err}");
        assert!(format!("{err}").contains("src/leak.rs"), "{err}");
    }
    // A directory link is checked without being followed.
    fs::remove_file(repo.join("src/leak.rs")).unwrap();
    symlink(&outside, repo.join("specs/elsewhere")).unwrap();
    let err = compile(&cfg, &repo)
        .err()
        .expect("a directory link refuses too");
    assert!(format!("{err}").contains("specs/elsewhere"), "{err}");
}

/// §3.4: a repository reached through a link still works, and a link under
/// the derived root is spec 127's to judge, not this rule's.
#[cfg(unix)]
#[test]
fn a_linked_root_and_the_derived_tree_are_not_this_rules_business() {
    use std::os::unix::fs::symlink;
    let tmp = tempfile::tempdir().unwrap();
    let real = tmp.path().join("real");
    corpus(&real);
    let alias = tmp.path().join("alias");
    symlink(&real, &alias).unwrap();
    let cfg = Config::default();
    assert!(compile(&cfg, &alias).is_ok());
    assert!(index(&cfg, &alias).is_ok());

    let outside = tmp.path().join("outside");
    fs::create_dir_all(&outside).unwrap();
    fs::create_dir_all(real.join(".derived")).unwrap();
    symlink(&outside, real.join(".derived/codebase-index")).unwrap();
    assert!(compile(&cfg, &real).is_ok());
    // The index read of that tree is 127's refusal or a plain missing tree,
    // never this rule's message.
    if let Err(e) = check_index_freshness(&cfg, &real) {
        assert!(!format!("{e}").contains("spec 144"), "{e}");
    }
}

/// Spec 147 §3.1, §3.2: a link AT a derived or state root, or at an ancestor
/// of one, is checked on read like any other link. One resolving outside the
/// repository refuses every read, exit 2, naming the link and saying the
/// derived tree is read through it; one resolving inside is still read.
#[cfg(unix)]
#[test]
fn a_linked_ancestor_of_the_derived_root_is_checked_on_read() {
    use std::os::unix::fs::symlink;
    for (toml, link, below) in [
        // The managed layout: `.statecraft` above `.statecraft/derived`.
        (
            "[layout]\nderived_dir = \".statecraft/derived\"\n",
            ".statecraft",
            "derived",
        ),
        // A deeper ancestor, two levels above the root.
        (
            "[layout]\nderived_dir = \"gov/out/derived\"\n",
            "gov",
            "out/derived",
        ),
        // The default root itself, which is also a resolver exclusion by name.
        ("", ".derived", ""),
        // The state root.
        (
            "[layout]\nstate_dir = \".statecraft/state\"\n",
            ".statecraft",
            "state",
        ),
    ] {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path().join("repo");
        corpus(&repo);
        let cfg = load_config(toml).unwrap();
        // Outside: the refusal.
        let away = tmp.path().join("away");
        fs::create_dir_all(away.join(below)).unwrap();
        write(&away, "notes.md", "n\n");
        symlink(&away, repo.join(link)).unwrap();
        let errs = [
            compile(&cfg, &repo).err().expect("compile refuses"),
            index(&cfg, &repo).err().expect("index refuses"),
            check_index_freshness(&cfg, &repo).expect_err("the freshness read refuses"),
        ];
        for err in errs {
            let msg = format!("{err}");
            assert_eq!(err.exit_code(), 2, "{link}: {msg}");
            assert!(msg.starts_with("refused: "), "{link}: {msg}");
            assert!(
                msg.contains(&format!("'{link}' is a link")),
                "{link}: {msg}"
            );
            assert!(
                msg.contains(
                    "the derived tree or a governed file beside it is read through that link"
                ),
                "{link}: {msg}"
            );
            assert!(msg.contains("spec 147"), "{link}: {msg}");
        }
        // Inside: the walk accepts it, and does not enter it.
        fs::remove_file(repo.join(link)).unwrap();
        let store = repo.join("store/sc");
        fs::create_dir_all(store.join(below)).unwrap();
        symlink(Path::new("store/sc"), repo.join(link)).unwrap();
        assert!(
            compile(&cfg, &repo).is_ok(),
            "{link}: a link inside is read"
        );
        assert!(index(&cfg, &repo).is_ok(), "{link}: a link inside is read");
        // The freshness read reaches the walk too: whatever it answers about
        // an unbuilt tree, it is not this refusal.
        if let Err(e) = check_index_freshness(&cfg, &repo) {
            assert!(!format!("{e}").contains("spec 147"), "{link}: {e}");
        }
    }
}

/// Spec 148: create a directory junction at `link` pointing to `target`, with
/// `mklink /J`, the only junction creator the standard library does not wrap.
/// A junction needs no privilege, so a failure here is a test failure.
///
/// `cmd` reads a `/` inside an argument as a switch (`src/inside` is `src`
/// and the switch `/inside`), so both paths are handed over with `\`.
#[cfg(windows)]
fn junction(target: &Path, link: &Path) {
    let win = |p: &Path| p.to_string_lossy().replace('/', "\\");
    let out = std::process::Command::new("cmd")
        .args(["/C", "mklink", "/J"])
        .arg(win(link))
        .arg(win(target))
        .output()
        .expect("cmd runs");
    assert!(
        out.status.success(),
        "mklink /J {} {}: {}",
        link.display(),
        target.display(),
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Spec 148 §3.1: the cases the Unix tests assert, for one Windows link kind.
/// `make` creates a directory link of that kind at its second argument,
/// pointing to its first.
#[cfg(windows)]
fn the_link_rule_holds_for(kind: &str, make: &dyn Fn(&Path, &Path)) {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path().join("repo");
    let outside = tmp.path().join("outside");
    corpus(&repo);
    write(&outside, "secret.rs", "pub fn secret() {}\n");
    let cfg = Config::default();

    // Inside: a link to another directory of the repository is read.
    write(&repo, "lib/b.rs", "pub fn b() {}\n");
    make(&repo.join("lib"), &repo.join("src/inside"));
    assert!(
        compile(&cfg, &repo).is_ok(),
        "{kind}: a link inside is read"
    );
    assert!(index(&cfg, &repo).is_ok(), "{kind}: a link inside is read");

    // Dangling: a link whose target is gone reads nothing and is fine.
    let gone = tmp.path().join("gone");
    fs::create_dir_all(&gone).unwrap();
    make(&gone, &repo.join("src/dangling"));
    fs::remove_dir(&gone).unwrap();
    assert!(
        compile(&cfg, &repo).is_ok(),
        "{kind}: a dangling link is read"
    );
    assert!(
        index(&cfg, &repo).is_ok(),
        "{kind}: a dangling link is read"
    );

    // Outside: compile and index refuse, exit 2, naming the link.
    make(&outside, &repo.join("src/leak"));
    for (verb, res) in [
        ("compile", compile(&cfg, &repo).err()),
        ("index", index(&cfg, &repo).err()),
    ] {
        let err = res.unwrap_or_else(|| panic!("{kind}: {verb} refuses a link outside"));
        assert_eq!(err.exit_code(), 2, "{kind}: {verb}: {err}");
        assert!(
            format!("{err}").contains("src/leak"),
            "{kind}: {verb}: {err}"
        );
    }
    fs::remove_dir(repo.join("src/leak")).unwrap();

    // A repository reached through a link of this kind works.
    let alias = tmp.path().join("alias");
    make(&repo, &alias);
    assert!(
        compile(&cfg, &alias).is_ok(),
        "{kind}: a linked repository works"
    );
    assert!(
        index(&cfg, &alias).is_ok(),
        "{kind}: a linked repository works"
    );
}

/// Spec 148 §3.1: a directory junction follows spec 144's link rule.
#[cfg(windows)]
#[test]
fn a_junction_follows_the_link_rule() {
    the_link_rule_holds_for("junction", &|target, link| junction(target, link));
}

/// Spec 148 §3.1, §3.2: a directory symbolic link follows spec 144's link
/// rule. Creating one needs a privilege (or Developer Mode) a runner may lack:
/// `ERROR_PRIVILEGE_NOT_HELD` (1314) is reported as not run, never passed, and
/// under `CI=true` it fails, so the required job cannot go green without
/// having created a link.
#[cfg(windows)]
#[test]
fn a_windows_symbolic_link_follows_the_link_rule() {
    const ERROR_PRIVILEGE_NOT_HELD: i32 = 1314;
    let probe = tempfile::tempdir().unwrap();
    if let Err(e) = std::os::windows::fs::symlink_dir(probe.path(), probe.path().join("probe")) {
        if e.raw_os_error() == Some(ERROR_PRIVILEGE_NOT_HELD) {
            let why = format!(
                "a_windows_symbolic_link_follows_the_link_rule: NOT RUN: creating a \
                 symbolic link failed with ERROR_PRIVILEGE_NOT_HELD (1314): {e}"
            );
            if std::env::var("CI").as_deref() == Ok("true") {
                panic!("{why}; CI=true, so this fails rather than skips (spec 148 3.2)");
            }
            eprintln!("{why}");
            return;
        }
        panic!("creating a symbolic link failed: {e}");
    }
    the_link_rule_holds_for("symbolic link", &|target, link| {
        std::os::windows::fs::symlink_dir(target, link).unwrap()
    });
}
