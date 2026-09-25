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
        for bad in ["../x", "/etc", "C:foo", "a\\b", "con/x", "a/nul.txt", "x./y"] {
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
    assert!(compile(&cfg, &repo).is_ok(), "a link inside, and a dangling one, are fine");
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
    let err = compile(&cfg, &repo).err().expect("a directory link refuses too");
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
