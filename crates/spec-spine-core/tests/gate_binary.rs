//! Spec 117 §3.5: which binary `make` actually governs with.
//!
//! The defect this asserts against was invisible for as long as it was: the
//! root `Makefile` documented `$SPEC_SPINE`, then `./target/release`, then
//! `PATH`, and assigned `SPEC_SPINE ?= spec-spine`, which is the third step
//! alone. Nothing read the two together, because reading a variable is not
//! running a gate.
//!
//! So these tests do not read the variable. Each case builds a tree with stub
//! binaries that announce a distinct identity and a distinct version, runs the
//! REAL root `Makefile` (copied, never paraphrased) in it, and asserts on which
//! stub printed. The two refusal rows additionally assert that NO stub printed:
//! a non-zero exit is on its own compatible with the gate having run the wrong
//! binary and that binary having failed, which is the exact failure being
//! refused.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

/// A stub `spec-spine`. `--version` answers like the real binary; anything
/// else echoes the identity and the argv it was handed.
fn write_stub(path: &Path, id: &str, version: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        path,
        format!(
            "#!/bin/sh\n\
             case \"$1\" in\n\
             \x20 --version) echo \"spec-spine {version} (stub {id})\" ;;\n\
             \x20 *) echo \"STUB-RAN {id} argv: $*\" ;;\n\
             esac\n\
             exit 0\n"
        ),
    )
    .unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

struct Tree {
    dir: tempfile::TempDir,
}

impl Tree {
    /// A copy of the real `Makefile`, a `PATH` directory, and optionally the
    /// in-tree build. Nothing else: the cases here never reach a corpus.
    fn new(with_in_tree: bool) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::copy(repo_root().join("Makefile"), root.join("Makefile")).unwrap();
        write_stub(&root.join("pathdir/spec-spine"), "PATH", "0.20.0");
        if with_in_tree {
            write_stub(&root.join("target/release/spec-spine"), "INTREE", "0.22.0");
        }
        write_stub(&root.join("elsewhere/ss-override"), "OVERRIDE", "9.9.9");
        // A regular file that is NOT executable: the invalid-override case.
        fs::create_dir_all(root.join("broken")).unwrap();
        fs::write(root.join("broken/spec-spine"), "not a binary\n").unwrap();
        fs::set_permissions(
            root.join("broken/spec-spine"),
            fs::Permissions::from_mode(0o644),
        )
        .unwrap();
        Tree { dir }
    }

    fn path(&self, rel: &str) -> String {
        self.dir.path().join(rel).to_string_lossy().into_owned()
    }

    /// Run a target with the stub `PATH` in front of a minimal system one.
    fn make(&self, args: &[&str]) -> Output {
        let path = format!("{}:/usr/bin:/bin:/usr/sbin:/sbin", self.path("pathdir"));
        Command::new("make")
            .arg("-C")
            .arg(self.dir.path())
            .args(args)
            .env("PATH", path)
            .env_remove("SPEC_SPINE")
            .env_remove("SPEC_SPINE_BIN")
            .output()
            .expect("make is required to run the gate tests")
    }
}

fn out(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}
fn err(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

/// Which stub, if any, the target actually executed, read out of the marker rather than matched against a list
/// of known identities. A fixed list is a footgun for the negative assertions
/// that use this: a stub added later and forgotten here would make `ran`
/// answer `None` while a binary had in fact run, so "no substitute may run"
/// would pass by not looking. Any `STUB-RAN <id>` is a stub running.
fn ran(o: &Output) -> Option<String> {
    let combined = format!("{}{}", out(o), err(o));
    let i = combined.find("STUB-RAN ")?;
    let rest = &combined[i + "STUB-RAN ".len()..];
    let id: String = rest.chars().take_while(|c| !c.is_whitespace()).collect();
    if id.is_empty() { None } else { Some(id) }
}

// --- §3.5 row 1: an explicit override wins over everything present ---------

#[test]
fn explicit_override_is_used_even_when_the_in_tree_binary_exists() {
    let t = Tree::new(true);
    let o = t.make(&[
        "verify",
        "SPEC=probe",
        &format!("SPEC_SPINE={}", t.path("elsewhere/ss-override")),
    ]);
    assert!(o.status.success(), "stdout={} stderr={}", out(&o), err(&o));
    assert_eq!(ran(&o).as_deref(), Some("OVERRIDE"));
    assert!(
        err(&o).contains("an explicit SPEC_SPINE override"),
        "the announcement must say where the binary came from: {}",
        err(&o)
    );
    assert!(err(&o).contains("9.9.9"), "{}", err(&o));
}

// --- §3.5 row 2: the in-tree binary beats an installed one ----------------

#[test]
fn the_in_tree_binary_is_preferred_over_path() {
    let t = Tree::new(true);
    let o = t.make(&["verify", "SPEC=probe"]);
    assert!(o.status.success(), "stdout={} stderr={}", out(&o), err(&o));
    assert_eq!(
        ran(&o).as_deref(),
        Some("INTREE"),
        "with both present the gate must run the binary this checkout builds; \
         stdout={} stderr={}",
        out(&o),
        err(&o)
    );
    assert!(
        err(&o).contains("./target/release/spec-spine"),
        "{}",
        err(&o)
    );
}

// --- §3.5 row 3: no in-tree build, so the documented fallback -------------

#[test]
fn path_is_the_fallback_when_the_in_tree_binary_is_absent() {
    let t = Tree::new(false);
    let o = t.make(&["verify", "SPEC=probe"]);
    assert!(o.status.success(), "stdout={} stderr={}", out(&o), err(&o));
    assert_eq!(ran(&o).as_deref(), Some("PATH"));
    assert!(
        err(&o).contains("PATH fallback"),
        "the fallback must announce itself as one: {}",
        err(&o)
    );
    assert!(
        err(&o).contains("may predate this corpus"),
        "spec 117 §3.3 requires the PATH case to say so: {}",
        err(&o)
    );
}

// --- §3.5 row 4: an OLDER binary on PATH, which is the measured defect ----

#[test]
fn an_older_path_binary_does_not_govern_a_built_checkout() {
    let t = Tree::new(true);
    let o = t.make(&["gate", "COUPLE=0", "OWNERSHIP=0"]);
    assert_eq!(
        ran(&o).as_deref(),
        Some("INTREE"),
        "stdout={} stderr={}",
        out(&o),
        err(&o)
    );
    let e = err(&o);
    assert!(
        e.contains("0.22.0") && !e.contains("0.20.0"),
        "the announced version must be the in-tree one, not the older install: {e}"
    );
    // The chain still ran through the same binary, not just the preflight.
    assert!(
        out(&o).contains("STUB-RAN INTREE argv: check"),
        "{}",
        out(&o)
    );
    assert!(
        out(&o).contains("STUB-RAN INTREE argv: lint"),
        "{}",
        out(&o)
    );
}

// --- §3.5 row 5: an unusable explicit override is refused, not replaced ---

#[test]
fn an_unusable_explicit_override_refuses_and_runs_nothing() {
    let t = Tree::new(true);
    let o = t.make(&[
        "verify",
        "SPEC=probe",
        &format!("SPEC_SPINE={}", t.path("broken/spec-spine")),
    ]);
    assert!(!o.status.success(), "stdout={}", out(&o));
    let e = err(&o);
    assert!(
        e.contains(&t.path("broken/spec-spine")),
        "the refusal must name the value it was given: {e}"
    );
    assert!(e.contains("names nothing executable"), "{e}");
    assert_eq!(
        ran(&o),
        None,
        "no substitute may run; a non-zero exit alone does not prove that. \
         stdout={} stderr={e}",
        out(&o)
    );
    assert!(
        !e.contains("STUB-RAN"),
        "and the substitute must not have been announced either: {e}"
    );
}

// --- §3.5 row 5b: an override naming a directory is the same refusal ------

#[test]
fn an_override_naming_a_directory_refuses_and_runs_nothing() {
    // The sibling of row 5, and the one that pins the mechanism rather than
    // the fixture. `command -v` disagrees across shells on a value containing
    // a slash: bash checks the executable bit, dash hands the string back. A
    // directory is executable-bit-set and is not a program, so a guard built
    // on `command -v` alone admits it under dash and the caller then meets a
    // message about the binary not running instead of one about the value.
    let t = Tree::new(true);
    let o = t.make(&[
        "verify",
        "SPEC=probe",
        &format!("SPEC_SPINE={}", t.path("broken")),
    ]);
    assert!(!o.status.success(), "stdout={}", out(&o));
    let e = err(&o);
    assert!(
        e.contains("names nothing executable"),
        "a directory is not a binary, whatever the shell says: {e}"
    );
    assert_eq!(ran(&o), None, "no substitute may run. stderr={e}");
}

// --- §3.5 row 6: an empty explicit override is the same refusal -----------

#[test]
fn an_empty_explicit_override_refuses_and_runs_nothing() {
    let t = Tree::new(true);
    let o = t.make(&["verify", "SPEC=probe", "SPEC_SPINE="]);
    assert!(!o.status.success(), "stdout={}", out(&o));
    assert!(err(&o).contains("set and empty"), "{}", err(&o));
    assert_eq!(ran(&o), None, "stdout={} stderr={}", out(&o), err(&o));
}

// --- §3.1: nothing found at all is a refusal, not a silent empty command --

#[test]
fn no_binary_anywhere_refuses_by_name() {
    let t = Tree::new(false);
    fs::remove_file(t.dir.path().join("pathdir/spec-spine")).unwrap();
    let o = t.make(&["verify", "SPEC=probe"]);
    assert!(!o.status.success(), "stdout={}", out(&o));
    let e = err(&o);
    assert!(
        e.contains("could not resolve a binary to govern with"),
        "{e}"
    );
    assert!(e.contains("./target/release/spec-spine"), "{e}");
}

// --- §3.4: resolving the binary must not build it ------------------------

#[test]
fn the_preflight_does_not_build_anything() {
    let t = Tree::new(false);
    let o = t.make(&["spec-spine-binary"]);
    assert!(o.status.success(), "stderr={}", err(&o));
    assert!(
        !t.dir.path().join("target/release/spec-spine").exists(),
        "the gate must not create the binary it judges with"
    );
    assert!(
        !format!("{}{}", out(&o), err(&o)).contains("cargo "),
        "{}",
        err(&o)
    );
}
