//! `spec-spine init` + the adoption definition-of-done (prompt §8): scaffold a
//! throwaway repo and run the full compile → index → lint → couple loop against
//! it with a **non-default `manifest.metadata_namespace`** and a **custom
//! `domains.allowed`**, with zero source edits to the library.

use std::fs;
use std::path::Path;
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_spec-spine"))
}

fn code(out: &std::process::Output) -> i32 {
    out.status.code().unwrap_or(-1)
}

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, content).unwrap();
}

fn run(root: &Path, args: &[&str]) -> std::process::Output {
    bin().arg("--repo").arg(root).args(args).output().unwrap()
}

#[test]
fn init_is_idempotent_and_force_overwrites() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    let first = run(root, &["init"]);
    assert_eq!(code(&first), 0);
    assert!(root.join("specs/000-bootstrap/spec.md").is_file());
    assert!(root.join("standards/spec/constitution.md").is_file());
    assert!(
        root.join(".claude/rules/adversarial-prompt-refusal.md")
            .is_file()
    );

    // Second run skips existing files (idempotent, still exit 0).
    let second = run(root, &["init"]);
    assert_eq!(code(&second), 0);
    assert!(String::from_utf8_lossy(&second.stdout).contains("skip (exists)"));

    // --force overwrites in place.
    let forced = run(root, &["init", "--force"]);
    assert_eq!(code(&forced), 0);
    assert!(String::from_utf8_lossy(&forced.stdout).contains("(--force)"));
}

#[test]
fn adoption_loop_with_non_default_namespace_and_domains() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    // Non-default namespace + a custom domain allowlist, written before init so
    // the scaffolder and every command read them.
    write(
        root,
        "spec-spine.toml",
        "[manifest]\nmetadata_namespace = \"acme\"\n\n[domains]\nallowed = [\"tooling\"]\n",
    );

    assert_eq!(code(&run(root, &["init"])), 0);

    // A crate linked back via the NON-DEFAULT namespace key.
    write(root, "Cargo.toml", "[workspace]\nmembers = [\"tool-x\"]\n");
    write(
        root,
        "tool-x/Cargo.toml",
        "[package]\nname = \"tool-x\"\nversion = \"0.1.0\"\n\
         [package.metadata.acme]\nspec = \"010-feature\"\n",
    );
    write(root, "tool-x/src/lib.rs", "pub fn run() {}\n");
    // A spec with a VALID domain from the custom allowlist.
    write(
        root,
        "specs/010-feature/spec.md",
        "---\nid: \"010-feature\"\ntitle: \"Feature\"\nstatus: approved\n\
         created: \"2026-06-09\"\nsummary: \"s\"\ndomain: \"tooling\"\n\
         establishes:\n  - \"tool-x/src/lib.rs\"\n---\n# 010\n## body\n",
    );

    // compile → index → lint, all clean.
    assert_eq!(code(&run(root, &["compile"])), 0, "compile clean");
    let idx = run(root, &["index"]);
    assert_eq!(code(&idx), 0, "index clean");
    // The non-default namespace drove the manifest read (spec 024: the package
    // inventory lives in its per-package shard).
    let pkg_shard =
        fs::read_to_string(root.join(".derived/codebase-index/by-package/tool-x.json")).unwrap();
    assert!(
        pkg_shard.contains("\"specRef\": \"010-feature\""),
        "acme namespace must link tool-x → 010-feature"
    );
    assert_eq!(code(&run(root, &["lint"])), 0, "lint runs");

    // couple: drift when only code changes; cleared when the owning spec is edited.
    write(root, "changed1.txt", "tool-x/src/lib.rs\n");
    let drift = run(
        root,
        &[
            "couple",
            "--paths-from",
            root.join("changed1.txt").to_str().unwrap(),
        ],
    );
    assert_eq!(
        code(&drift),
        1,
        "{}",
        String::from_utf8_lossy(&drift.stderr)
    );

    write(
        root,
        "changed2.txt",
        "tool-x/src/lib.rs\nspecs/010-feature/spec.md\n",
    );
    let cleared = run(
        root,
        &[
            "couple",
            "--paths-from",
            root.join("changed2.txt").to_str().unwrap(),
        ],
    );
    assert_eq!(code(&cleared), 0);
}

#[test]
fn custom_domain_allowlist_is_enforced_at_compile() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write(
        root,
        "spec-spine.toml",
        "[domains]\nallowed = [\"tooling\"]\n",
    );
    write(root, "Cargo.toml", "[workspace]\nmembers = []\n");
    // A domain OUTSIDE the allowlist → compile validation failure (exit 1).
    write(
        root,
        "specs/010-feature/spec.md",
        "---\nid: \"010-feature\"\ntitle: \"F\"\nstatus: approved\ncreated: \"2026-06-09\"\n\
         summary: \"s\"\ndomain: \"not-allowed\"\n---\n# 010\n",
    );
    let out = run(root, &["compile"]);
    assert_eq!(code(&out), 1, "invalid domain must fail compile");
}

// ── spec 065: init and the kit are one adoption ───────────────────────────

/// §3.4: a repository scaffolded with the kit satisfies its own gate on the
/// first run. The kit adds hooks and skills referencing verbs and paths, and a
/// scaffold that produces a repository failing its own gate is worse than none.
#[test]
fn a_kit_scaffolded_repository_is_immediately_governed() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    let init = bin()
        .arg("--repo")
        .arg(root)
        .args(["init", "--with-kit"])
        .output()
        .unwrap();
    assert_eq!(code(&init), 0, "{}", String::from_utf8_lossy(&init.stderr));

    // The harness landed at the adopter's own paths.
    for rel in [
        "AGENTS.md",
        ".claude/settings.json",
        ".claude/skills/build/SKILL.md",
        "Makefile",
        ".github/workflows/govern.yml",
        ".gitignore",
    ] {
        assert!(root.join(rel).is_file(), "missing {rel}");
    }

    // And the whole chain is clean on the first run.
    for args in [
        vec!["compile"],
        vec!["index"],
        vec!["lint", "--fail-on-warn"],
        vec!["index", "check"],
    ] {
        let out = bin().arg("--repo").arg(root).args(&args).output().unwrap();
        assert_eq!(
            code(&out),
            0,
            "{args:?} on a freshly scaffolded repo: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

/// §3.2: plain `init` writes the protocol and not the harness, so an adopter
/// who wants the rules without the skills gets exactly that.
#[test]
fn plain_init_writes_the_protocol_but_not_the_harness() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    assert_eq!(
        code(&bin().arg("--repo").arg(root).arg("init").output().unwrap()),
        0
    );

    assert!(
        root.join("AGENTS.md").is_file(),
        "the protocol is not optional"
    );
    assert!(root.join(".claude/rules/orchestrator-rules.md").is_file());
    assert!(
        !root.join(".claude/skills").exists(),
        "the harness is opt-in"
    );
    assert!(!root.join("Makefile").exists());
}

/// §3.3: it refuses to clobber. An adopter who has written their own
/// `AGENTS.md` is the common case in a repository that has been worked in, and
/// overwriting a cross-agent authority document would destroy project protocol
/// no backup makes obvious.
#[test]
fn with_kit_does_not_clobber_an_existing_agents_md() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fs::write(root.join("AGENTS.md"), "# mine\n").unwrap();

    let out = bin()
        .arg("--repo")
        .arg(root)
        .args(["init", "--with-kit"])
        .output()
        .unwrap();
    assert_eq!(
        code(&out),
        0,
        "skipping is not an error; init is idempotent"
    );
    assert_eq!(
        fs::read_to_string(root.join("AGENTS.md")).unwrap(),
        "# mine\n",
        "the adopter's own protocol survived"
    );
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("skip"),
        "and the skip is reported: {}",
        String::from_utf8_lossy(&out.stdout)
    );

    // `--force` behaves for these files exactly as it does for the other ten.
    let forced = bin()
        .arg("--repo")
        .arg(root)
        .args(["init", "--with-kit", "--force"])
        .output()
        .unwrap();
    assert_eq!(code(&forced), 0);
    assert_ne!(
        fs::read_to_string(root.join("AGENTS.md")).unwrap(),
        "# mine\n"
    );
}

// ── spec 074 3.10: the guards run what was written ──────────────────────────
//
// Every defect this spec fixes was already covered by a test that asserted the
// artifact EXISTED. A test that a file is written passes on a file nobody can
// execute; a test that a key is emitted passes on a key that matches nothing.
// These assertions exercise the artifact instead.

/// Spec 074 3.4: a scaffolded executable arrives executable.
///
/// `init --with-kit` wrote `.githooks/enable-merge-driver.sh` at mode 644 while
/// its own documentation invoked it as `./.githooks/enable-merge-driver.sh`.
#[cfg(unix)]
#[test]
fn scaffolded_shell_scripts_are_executable_on_disk() {
    use std::os::unix::fs::PermissionsExt;
    let tmp = tempfile::tempdir().unwrap();
    let out = run(tmp.path(), &["init", "--with-kit"]);
    assert_eq!(code(&out), 0, "{}", String::from_utf8_lossy(&out.stderr));

    let mut checked = 0;
    for rel in [
        ".githooks/enable-merge-driver.sh",
        ".githooks/merge-derived-index.sh",
    ] {
        let p = tmp.path().join(rel);
        assert!(p.is_file(), "{rel} was not written");
        let mode = fs::metadata(&p).unwrap().permissions().mode();
        assert!(mode & 0o111 != 0, "{rel} is not executable: mode {mode:o}");
        checked += 1;
    }
    assert_eq!(checked, 2, "the guard must actually have found the scripts");

    // And nothing else silently gained the bit.
    let toml = fs::metadata(tmp.path().join("spec-spine.toml"))
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(toml & 0o111, 0, "a non-script must stay non-executable");
}

/// Spec 074 3.3: `--with-kit` writes the binding for the driver it installs.
///
/// Spec 065 3.2 decided the stanza was "a block to append rather than a file to
/// write" and excluded it, which shipped two merge-driver hooks with nothing
/// binding them.
#[test]
fn with_kit_writes_the_merge_driver_binding_and_does_not_duplicate_it() {
    let tmp = tempfile::tempdir().unwrap();
    assert_eq!(code(&run(tmp.path(), &["init", "--with-kit"])), 0);

    let ga = tmp.path().join(".gitattributes");
    let first = fs::read_to_string(&ga).expect(".gitattributes was written");
    assert!(
        first.contains("merge=spec-spine-derived-regen"),
        "the binding is missing: {first}"
    );
    let bindings = |s: &str| s.matches("merge=spec-spine-derived-regen").count();
    let n = bindings(&first);
    assert!(n > 0);

    // A second run must add nothing.
    assert_eq!(code(&run(tmp.path(), &["init", "--with-kit"])), 0);
    let second = fs::read_to_string(&ga).unwrap();
    assert_eq!(
        bindings(&second),
        n,
        "appending must be idempotent, got:\n{second}"
    );
}

/// The same, into a repository that already has a `.gitattributes`, which is
/// most of them. The existing content must survive.
#[test]
fn the_binding_appends_to_an_existing_gitattributes() {
    let tmp = tempfile::tempdir().unwrap();
    write(tmp.path(), ".gitattributes", "* text=auto eol=lf\n");
    assert_eq!(code(&run(tmp.path(), &["init", "--with-kit"])), 0);

    let after = fs::read_to_string(tmp.path().join(".gitattributes")).unwrap();
    assert!(
        after.contains("* text=auto eol=lf"),
        "the adopter's own rules must be preserved: {after}"
    );
    assert!(
        after.contains("merge=spec-spine-derived-regen"),
        "the binding must be appended: {after}"
    );
}

/// Spec 074 3.5: `init` does not create a corpus its own compiler refuses.
///
/// Run beside an existing `000` spec it wrote a second one, reported "0
/// skipped", and left a tree the next `compile` rejects with `V-004`. The check
/// is on the ORDINAL, not the directory name, because that is what V-004
/// collides on.
#[test]
fn init_skips_a_bootstrap_spec_whose_ordinal_is_taken_and_the_result_compiles() {
    let tmp = tempfile::tempdir().unwrap();
    write(
        tmp.path(),
        "specs/000-my-own-bootstrap/spec.md",
        "---\nid: \"000-my-own-bootstrap\"\ntitle: \"B\"\nstatus: approved\n\
         created: \"2026-01-01\"\nsummary: \"s\"\nestablishes:\n  - \"README.md\"\n\
         ---\n# 000\n## body\n",
    );

    let out = run(tmp.path(), &["init"]);
    assert_eq!(code(&out), 0, "{}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("skip (ordinal 000 already used by specs/000-my-own-bootstrap)"),
        "the skip must say why, in the same voice as every other skip: {stdout}"
    );
    assert!(
        !tmp.path().join("specs/000-bootstrap/spec.md").exists(),
        "the colliding spec must not be written"
    );
    // Do not point the adopter at a file this run declined to write.
    assert!(
        !stdout.contains("customize specs/000-bootstrap/spec.md"),
        "the closing hint names a skipped file: {stdout}"
    );

    // The whole point: the corpus `init` produced is one its own compiler accepts.
    let compiled = run(tmp.path(), &["compile"]);
    assert_eq!(
        code(&compiled),
        0,
        "init must not produce a corpus compile refuses: {}",
        String::from_utf8_lossy(&compiled.stderr)
    );
}
