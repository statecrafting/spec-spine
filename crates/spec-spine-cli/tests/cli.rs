//! End-to-end exit-code contract for the `spec-spine` binary.
//!
//! Exit codes: 0 ok, 1 validation failure / not found, 3 I/O / parse / schema.

use std::fs;
use std::path::Path;
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_spec-spine"))
}

fn write_spec(root: &Path, dir: &str, id: &str, status: &str) {
    let spec_dir = root.join("specs").join(dir);
    fs::create_dir_all(&spec_dir).unwrap();
    let body = format!(
        "---\nid: \"{id}\"\ntitle: \"T\"\nstatus: {status}\ncreated: \"2026-06-08\"\nsummary: \"s\"\n---\n# {id}\n"
    );
    fs::write(spec_dir.join("spec.md"), body).unwrap();
}

fn code(out: &std::process::Output) -> i32 {
    out.status.code().unwrap_or(-1)
}

#[test]
fn index_slice_hashes_and_check() {
    let tmp = tempfile::tempdir().unwrap();
    write_spec(tmp.path(), "001-a", "001-a", "approved");
    fs::create_dir_all(tmp.path().join("conf")).unwrap();
    fs::write(tmp.path().join("conf/a.json"), "{\"a\":1}\n").unwrap();
    fs::write(tmp.path().join("conf/b.json"), "{\"b\":2}\n").unwrap();
    let run = |args: &[&str]| {
        let out = bin().arg("--repo").arg(tmp.path()).args(args).output();
        out.unwrap()
    };
    // Slices live in their own sidecar since spec 024 (no monolithic index.json).
    let slices_file = tmp.path().join(".derived/codebase-index/slices.json");

    // No slices configured: no sidecar; --slice is a config error (3).
    assert_eq!(code(&run(&["index"])), 0);
    assert!(
        !slices_file.exists(),
        "no slices configured -> no slices.json sidecar"
    );
    assert_eq!(
        code(&run(&["index", "check", "--slice", "agent-config"])),
        3,
        "unknown slice name -> 3"
    );

    // Slices configured AFTER the committed index: missing entry -> stale.
    fs::write(
        tmp.path().join("spec-spine.toml"),
        "[index.slices]\nzz-last = [\"conf/b.json\"]\nagent-config = [\"conf/a.json\", \"conf/missing.json\"]\n",
    )
    .unwrap();
    assert_eq!(
        code(&run(&["index", "check", "--slice", "agent-config"])),
        2,
        "an index predating the slice config is not vouching for it"
    );

    // Rebuild: entries emitted key-sorted; both slices fresh.
    assert_eq!(code(&run(&["index"])), 0);
    let raw = fs::read_to_string(&slices_file).unwrap();
    assert!(
        raw.find("agent-config").unwrap() < raw.find("zz-last").unwrap(),
        "slice hash keys are sorted"
    );
    assert_eq!(
        code(&run(&["index", "check", "--slice", "agent-config"])),
        0
    );
    assert_eq!(code(&run(&["index", "check", "--slice", "zz-last"])), 0);
    assert_eq!(code(&run(&["index", "check"])), 0);

    // Independence: a slice-only file's edit trips its slice, not the global
    // gate and not the other slice.
    fs::write(tmp.path().join("conf/a.json"), "{\"a\":99}\n").unwrap();
    assert_eq!(code(&run(&["index", "check"])), 0, "global gate unaffected");
    assert_eq!(
        code(&run(&["index", "check", "--slice", "agent-config"])),
        2
    );
    assert_eq!(code(&run(&["index", "check", "--slice", "zz-last"])), 0);

    // ...and vice versa: a global-input edit leaves the slices fresh.
    write_spec(tmp.path(), "001-a", "001-a", "draft");
    assert_eq!(
        code(&run(&["index", "check"])),
        2,
        "spec.md is global input"
    );
    assert_eq!(code(&run(&["index", "check", "--slice", "zz-last"])), 0);

    // Deletion of a guarded file is a hash change, not a config error.
    assert_eq!(code(&run(&["index"])), 0);
    fs::remove_file(tmp.path().join("conf/b.json")).unwrap();
    assert_eq!(code(&run(&["index", "check", "--slice", "zz-last"])), 2);

    // Unknown name with slices configured is still 3.
    assert_eq!(code(&run(&["index", "check", "--slice", "nope"])), 3);
}

#[test]
fn invalid_slice_config_exits_3() {
    let tmp = tempfile::tempdir().unwrap();
    write_spec(tmp.path(), "001-a", "001-a", "approved");

    // Name outside [a-z0-9][a-z0-9-]*.
    fs::write(
        tmp.path().join("spec-spine.toml"),
        "[index.slices]\n\"Bad_Name\" = [\"conf/*.json\"]\n",
    )
    .unwrap();
    assert_eq!(
        code(
            &bin()
                .arg("--repo")
                .arg(tmp.path())
                .arg("index")
                .output()
                .unwrap()
        ),
        3
    );

    // Empty glob list.
    fs::write(
        tmp.path().join("spec-spine.toml"),
        "[index.slices]\nok = []\n",
    )
    .unwrap();
    assert_eq!(
        code(
            &bin()
                .arg("--repo")
                .arg(tmp.path())
                .arg("index")
                .output()
                .unwrap()
        ),
        3
    );
}

#[test]
fn compile_ok_then_queries() {
    let tmp = tempfile::tempdir().unwrap();
    write_spec(tmp.path(), "001-a", "001-a", "approved");
    write_spec(tmp.path(), "002-b", "002-b", "approved");

    let compile = bin()
        .arg("--repo")
        .arg(tmp.path())
        .arg("compile")
        .output()
        .unwrap();
    assert_eq!(code(&compile), 0, "clean compile exits 0");
    // Sharded committed form (spec 024): one file per spec, no monolithic registry.json.
    assert!(
        tmp.path()
            .join(".derived/spec-registry/by-spec/001-a.json")
            .is_file()
    );
    assert!(
        !tmp.path()
            .join(".derived/spec-registry/registry.json")
            .exists()
    );

    let list = bin()
        .arg("--repo")
        .arg(tmp.path())
        .args(["registry", "list"])
        .output()
        .unwrap();
    assert_eq!(code(&list), 0);
    assert!(String::from_utf8_lossy(&list.stdout).contains("001-a"));

    let show_missing = bin()
        .arg("--repo")
        .arg(tmp.path())
        .args(["registry", "show", "999-nope"])
        .output()
        .unwrap();
    assert_eq!(code(&show_missing), 1, "not found exits 1");
}

#[test]
fn registry_list_ids_only_projection() {
    let tmp = tempfile::tempdir().unwrap();
    write_spec(tmp.path(), "001-a", "001-a", "approved");
    write_spec(tmp.path(), "002-b", "002-b", "approved");
    write_spec(tmp.path(), "003-c", "003-c", "draft");
    let compiled = bin()
        .arg("--repo")
        .arg(tmp.path())
        .arg("compile")
        .output()
        .unwrap();
    assert_eq!(code(&compiled), 0);

    // Text form: newline-delimited ids in id order, nothing else.
    let text = bin()
        .arg("--repo")
        .arg(tmp.path())
        .args(["registry", "list", "--ids-only"])
        .output()
        .unwrap();
    assert_eq!(code(&text), 0);
    assert_eq!(
        String::from_utf8_lossy(&text.stdout),
        "001-a\n002-b\n003-c\n"
    );

    // JSON form: an array of id strings, same order.
    let json = bin()
        .arg("--repo")
        .arg(tmp.path())
        .args(["registry", "list", "--ids-only", "--json"])
        .output()
        .unwrap();
    assert_eq!(code(&json), 0);
    let ids: Vec<String> = serde_json::from_slice(&json.stdout).unwrap();
    assert_eq!(ids, ["001-a", "002-b", "003-c"]);

    // --status filters first, then the projection applies.
    let filtered = bin()
        .arg("--repo")
        .arg(tmp.path())
        .args(["registry", "list", "--ids-only", "--status", "approved"])
        .output()
        .unwrap();
    assert_eq!(code(&filtered), 0);
    assert_eq!(String::from_utf8_lossy(&filtered.stdout), "001-a\n002-b\n");

    let filtered_json = bin()
        .arg("--repo")
        .arg(tmp.path())
        .args([
            "registry",
            "list",
            "--ids-only",
            "--status",
            "retired",
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(code(&filtered_json), 0);
    let none: Vec<String> = serde_json::from_slice(&filtered_json.stdout).unwrap();
    assert!(none.is_empty());

    // Empty projection in text mode: empty output (no "(no specs)"), exit 0.
    let empty = bin()
        .arg("--repo")
        .arg(tmp.path())
        .args(["registry", "list", "--ids-only", "--status", "retired"])
        .output()
        .unwrap();
    assert_eq!(code(&empty), 0);
    assert!(empty.stdout.is_empty());
}

#[test]
fn registry_status_report_nonzero_only_projection() {
    let tmp = tempfile::tempdir().unwrap();
    // approved + draft present; superseded + retired are the zero-count rows.
    write_spec(tmp.path(), "001-a", "001-a", "approved");
    write_spec(tmp.path(), "002-b", "002-b", "approved");
    write_spec(tmp.path(), "003-c", "003-c", "draft");
    let compiled = bin()
        .arg("--repo")
        .arg(tmp.path())
        .arg("compile")
        .output()
        .unwrap();
    assert_eq!(code(&compiled), 0);

    // Without the flag, output is byte-identical to pre-010 behavior.
    let plain = bin()
        .arg("--repo")
        .arg(tmp.path())
        .args(["registry", "status-report"])
        .output()
        .unwrap();
    assert_eq!(code(&plain), 0);
    assert_eq!(
        String::from_utf8_lossy(&plain.stdout),
        "total:      3\ndraft:      1\napproved:   2\nsuperseded: 0\nretired:    0\n"
    );

    // Human form: zero-count rows omitted, total unaffected.
    let human = bin()
        .arg("--repo")
        .arg(tmp.path())
        .args(["registry", "status-report", "--nonzero-only"])
        .output()
        .unwrap();
    assert_eq!(code(&human), 0);
    assert_eq!(
        String::from_utf8_lossy(&human.stdout),
        "total:      3\ndraft:      1\napproved:   2\n"
    );

    // JSON form: zero-count keys absent, total present.
    let json = bin()
        .arg("--repo")
        .arg(tmp.path())
        .args(["registry", "status-report", "--nonzero-only", "--json"])
        .output()
        .unwrap();
    assert_eq!(code(&json), 0);
    let report: serde_json::Value = serde_json::from_slice(&json.stdout).unwrap();
    assert_eq!(report["total"], 3);
    assert_eq!(report["draft"], 1);
    assert_eq!(report["approved"], 2);
    assert!(report.get("superseded").is_none());
    assert!(report.get("retired").is_none());
}

#[test]
fn compile_validation_failure_exits_1() {
    let tmp = tempfile::tempdir().unwrap();
    // Directory name != id -> V-001 (error tier).
    write_spec(tmp.path(), "001-folder", "001-mismatch", "approved");
    let out = bin()
        .arg("--repo")
        .arg(tmp.path())
        .arg("compile")
        .output()
        .unwrap();
    assert_eq!(code(&out), 1, "validation failure exits 1");
}

#[test]
fn missing_specs_dir_exits_3() {
    let tmp = tempfile::tempdir().unwrap();
    // No specs/ dir at all -> I/O error.
    let out = bin()
        .arg("--repo")
        .arg(tmp.path())
        .arg("compile")
        .output()
        .unwrap();
    assert_eq!(code(&out), 3, "I/O error exits 3");
}

#[test]
fn registry_query_before_compile_exits_3() {
    let tmp = tempfile::tempdir().unwrap();
    write_spec(tmp.path(), "001-a", "001-a", "approved");
    // No compile yet -> registry.json missing -> I/O error.
    let out = bin()
        .arg("--repo")
        .arg(tmp.path())
        .args(["registry", "list"])
        .output()
        .unwrap();
    assert_eq!(code(&out), 3);
}

#[test]
fn index_then_check_fresh_then_stale() {
    let tmp = tempfile::tempdir().unwrap();
    write_spec(tmp.path(), "001-a", "001-a", "approved");

    let built = bin()
        .arg("--repo")
        .arg(tmp.path())
        .arg("index")
        .output()
        .unwrap();
    assert_eq!(code(&built), 0, "index writes -> 0");
    // Sharded committed form (spec 024): per-spec shard, no monolithic index.json.
    assert!(
        tmp.path()
            .join(".derived/codebase-index/by-spec/001-a.json")
            .is_file()
    );

    let fresh = bin()
        .arg("--repo")
        .arg(tmp.path())
        .args(["index", "check"])
        .output()
        .unwrap();
    assert_eq!(code(&fresh), 0, "fresh -> 0");

    // Mutate a hashed input -> stale.
    write_spec(tmp.path(), "001-a", "001-a", "draft");
    let stale = bin()
        .arg("--repo")
        .arg(tmp.path())
        .args(["index", "check"])
        .output()
        .unwrap();
    assert_eq!(code(&stale), 2, "stale -> 2");
}

#[test]
fn index_render_and_orphans_projections() {
    let tmp = tempfile::tempdir().unwrap();
    let write_claiming_spec = |id: &str, target: &str| {
        let dir = tmp.path().join("specs").join(id);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("spec.md"),
            format!(
                "---\nid: \"{id}\"\ntitle: \"T\"\nstatus: approved\ncreated: \"2026-06-08\"\nsummary: \"s\"\nestablishes:\n  - \"{target}\"\n---\n# {id}\n"
            ),
        )
        .unwrap();
    };
    // 001-a claims a path that resolves -> mapped; 002-b claims a path that
    // resolves nowhere -> orphaned.
    fs::create_dir_all(tmp.path().join("src")).unwrap();
    fs::write(tmp.path().join("src/lib.rs"), "// Spec: 001-a\n").unwrap();
    write_claiming_spec("001-a", "src/lib.rs");
    write_claiming_spec("002-b", "src/missing.rs");

    // Projections before `index` has run: exit 3 (missing artifact).
    let early_render = bin()
        .arg("--repo")
        .arg(tmp.path())
        .args(["index", "render"])
        .output()
        .unwrap();
    assert_eq!(code(&early_render), 3, "render without index -> 3");
    let early_orphans = bin()
        .arg("--repo")
        .arg(tmp.path())
        .args(["index", "orphans"])
        .output()
        .unwrap();
    assert_eq!(code(&early_orphans), 3, "orphans without index -> 3");

    let built = bin()
        .arg("--repo")
        .arg(tmp.path())
        .arg("index")
        .output()
        .unwrap();
    assert_eq!(code(&built), 0);

    // Orphans, text and JSON forms.
    let orphans_text = bin()
        .arg("--repo")
        .arg(tmp.path())
        .args(["index", "orphans"])
        .output()
        .unwrap();
    assert_eq!(code(&orphans_text), 0, "orphans is a query, not a gate");
    assert_eq!(String::from_utf8_lossy(&orphans_text.stdout), "002-b\n");

    let orphans_json = bin()
        .arg("--repo")
        .arg(tmp.path())
        .args(["index", "orphans", "--json"])
        .output()
        .unwrap();
    assert_eq!(code(&orphans_json), 0);
    let ids: Vec<String> = serde_json::from_slice(&orphans_json.stdout).unwrap();
    assert_eq!(ids, ["002-b"]);

    // Render: exit 0 even with diagnostics in the artifact; contractual
    // sections present in order.
    let render = bin()
        .arg("--repo")
        .arg(tmp.path())
        .args(["index", "render"])
        .output()
        .unwrap();
    assert_eq!(code(&render), 0, "diagnostics do not fail a render");
    let md = String::from_utf8_lossy(&render.stdout);
    let positions: Vec<usize> = [
        "# spec-spine codebase index",
        "## Packages",
        "## Traceability",
    ]
    .iter()
    .map(|s| md.find(s).unwrap_or_else(|| panic!("missing section {s}")))
    .collect();
    assert!(positions.windows(2).all(|w| w[0] < w[1]), "section order");
    assert!(md.contains("### Orphaned specs"));
    assert!(md.contains("- 002-b"));
    assert!(md.ends_with('\n'));

    // Empty orphans list -> empty output, still exit 0.
    fs::remove_dir_all(tmp.path().join("specs/002-b")).unwrap();
    let rebuilt = bin()
        .arg("--repo")
        .arg(tmp.path())
        .arg("index")
        .output()
        .unwrap();
    assert_eq!(code(&rebuilt), 0);
    let none = bin()
        .arg("--repo")
        .arg(tmp.path())
        .args(["index", "orphans"])
        .output()
        .unwrap();
    assert_eq!(code(&none), 0);
    assert!(none.stdout.is_empty());
}

#[test]
fn lint_fail_on_warn_gating() {
    let tmp = tempfile::tempdir().unwrap();
    // An ordinary spec with no ownership edge -> L-001 (warning).
    write_spec(tmp.path(), "001-a", "001-a", "approved");

    let lenient = bin()
        .arg("--repo")
        .arg(tmp.path())
        .arg("lint")
        .output()
        .unwrap();
    assert_eq!(code(&lenient), 0, "warnings alone do not fail");

    let strict = bin()
        .arg("--repo")
        .arg(tmp.path())
        .args(["lint", "--fail-on-warn"])
        .output()
        .unwrap();
    assert_eq!(code(&strict), 1, "--fail-on-warn fails on a warning");
}

#[test]
fn compile_check_exit_contract() {
    // Spec 031 3.2: 0 fresh, 1 validation failed, 2 stale. Validation outranks
    // staleness.
    let tmp = tempfile::tempdir().unwrap();
    write_spec(tmp.path(), "001-a", "001-a", "approved");
    let run = |args: &[&str]| {
        bin()
            .arg("--repo")
            .arg(tmp.path())
            .args(args)
            .output()
            .unwrap()
    };

    // Never compiled: the committed registry is not vouching for anything.
    assert_eq!(
        code(&run(&["compile", "--check"])),
        2,
        "unbuilt -> stale (2)"
    );

    assert_eq!(code(&run(&["compile"])), 0);
    assert_eq!(
        code(&run(&["compile", "--check"])),
        0,
        "just compiled -> fresh (0)"
    );

    // --check must not have written anything, so a second check still agrees
    // and no build-meta sidecar was produced by it.
    let meta = tmp.path().join(".derived/spec-registry/build-meta.json");
    let meta_before = fs::read(&meta).unwrap();
    assert_eq!(code(&run(&["compile", "--check"])), 0);
    assert_eq!(
        fs::read(&meta).unwrap(),
        meta_before,
        "--check must not restamp build-meta.json"
    );

    // Edit a spec.md without recompiling: the PR #61 regression.
    let spec_md = tmp.path().join("specs/001-a/spec.md");
    let edited = fs::read_to_string(&spec_md).unwrap() + "\nmore body\n";
    fs::write(&spec_md, edited).unwrap();
    let stale = run(&["compile", "--check"]);
    assert_eq!(code(&stale), 2, "edited spec, stale shard -> 2");
    assert!(
        String::from_utf8_lossy(&stale.stderr).contains("modified 001-a.json"),
        "stale detail belongs on stderr: {}",
        String::from_utf8_lossy(&stale.stderr)
    );

    // Break validation while the shard is ALSO stale: validation wins (1).
    fs::write(
        &spec_md,
        "---\nid: \"mismatched\"\ntitle: \"T\"\nstatus: approved\ncreated: \"2026-06-08\"\nsummary: \"s\"\n---\n",
    )
    .unwrap();
    assert_eq!(
        code(&run(&["compile", "--check"])),
        1,
        "validation outranks staleness"
    );
}

#[test]
fn index_coverage_reports_and_gates() {
    // Spec 032: `index coverage` is a freshness-guarded read verb over the
    // tree and the committed index; `--fail-on-untraced` is the whole-tree
    // "fully specified" assertion.
    let tmp = tempfile::tempdir().unwrap();
    let r = tmp.path();
    let write = |rel: &str, content: &str| {
        let p = r.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, content).unwrap();
    };
    write(
        "Cargo.toml",
        "[package]\nname = \"root\"\nversion = \"0.1.0\"\n",
    );
    write("src/lib.rs", "pub fn a() {}\n");
    write("src/other.rs", "pub fn b() {}\n");
    write(
        "specs/001-a/spec.md",
        "---\nid: \"001-a\"\ntitle: \"T\"\nstatus: approved\ncreated: \"2026-06-08\"\nsummary: \"s\"\nestablishes:\n  - \"src/lib.rs\"\n---\n# 001-a\n",
    );
    let run = |args: &[&str]| bin().arg("--repo").arg(r).args(args).output().unwrap();

    assert_eq!(
        code(&run(&["index", "coverage"])),
        3,
        "no committed index -> artifact missing (3)"
    );
    assert_eq!(code(&run(&["index"])), 0);

    let text = run(&["index", "coverage"]);
    assert_eq!(code(&text), 0, "a report, not a gate");
    let out = String::from_utf8_lossy(&text.stdout);
    assert!(
        out.contains(
            "coverage: 1/2 source files specifically claimed (50.0%); 0 floor-only, 1 unclaimed"
        ),
        "{out}"
    );
    assert!(
        out.contains("unclaimed (no owning spec):\n  src/other.rs"),
        "{out}"
    );

    let json = run(&["index", "coverage", "--json"]);
    assert_eq!(code(&json), 0);
    let report: serde_json::Value = serde_json::from_slice(&json.stdout).unwrap();
    assert_eq!(report["sourceFiles"], 2);
    assert_eq!(report["claimedFiles"], 1);
    assert_eq!(
        report["unclaimedFiles"],
        serde_json::json!(["src/other.rs"])
    );

    assert_eq!(
        code(&run(&["index", "coverage", "--fail-on-untraced"])),
        1,
        "an untraced file fails the assertion"
    );

    // Claim the file, re-index: fully specified.
    write(
        "specs/001-a/spec.md",
        "---\nid: \"001-a\"\ntitle: \"T\"\nstatus: approved\ncreated: \"2026-06-08\"\nsummary: \"s\"\nestablishes:\n  - \"src/\"\n---\n# 001-a\n",
    );
    assert_eq!(
        code(&run(&["index", "coverage"])),
        2,
        "stale index -> 2, never a report over the wrong ledger"
    );
    assert_eq!(code(&run(&["index"])), 0);
    let full = run(&["index", "coverage", "--fail-on-untraced"]);
    assert_eq!(code(&full), 0, "{}", String::from_utf8_lossy(&full.stderr));
    assert!(
        String::from_utf8_lossy(&full.stdout)
            .contains("coverage: 2/2 source files specifically claimed (100.0%)")
    );
}

// ===== spec 035: a reader that stops early is not an error =====

/// `println!` unwraps its write, so a closed reader panicked the process:
/// `spec-spine registry list --json | head` exited **101** with a backtrace,
/// outside the documented 0/1/2/3 contract. Piping into `head` or a pager is
/// ordinary use.
///
/// The fixture is deliberately oversized. The child must still be mid-write
/// when the reader goes away, so the output has to exceed the OS pipe buffer
/// (64 KiB on Linux, smaller on some platforms); 30 specs with an 8 KiB summary
/// each is comfortably past any of them.
#[test]
fn closed_reader_exits_cleanly_rather_than_panicking() {
    use std::io::Read;
    use std::process::Stdio;

    let tmp = tempfile::tempdir().unwrap();
    let filler = "x".repeat(8192);
    for i in 0..30 {
        let id = format!("{i:03}-spec");
        let dir = tmp.path().join("specs").join(&id);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("spec.md"),
            format!(
                "---\nid: \"{id}\"\ntitle: \"T\"\nstatus: approved\ncreated: \"2026-06-08\"\nsummary: \"{filler}\"\n---\n# {id}\n"
            ),
        )
        .unwrap();
    }

    // `registry list` reads the committed shards, so the corpus has to exist.
    let compiled = bin()
        .arg("--repo")
        .arg(tmp.path())
        .arg("compile")
        .output()
        .unwrap();
    assert_eq!(code(&compiled), 0, "fixture must compile");

    let mut child = bin()
        .arg("--repo")
        .arg(tmp.path())
        .args(["registry", "list", "--json"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    // Read a little, then close the pipe: exactly what `| head -c 32` does.
    let mut stdout = child.stdout.take().unwrap();
    let mut buf = [0u8; 32];
    let _ = stdout.read(&mut buf);
    drop(stdout);

    let out = child.wait_with_output().unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);

    assert_ne!(
        out.status.code(),
        Some(101),
        "a closed reader must not panic the process; stderr: {stderr}"
    );
    assert!(
        !stderr.contains("panicked"),
        "no panic should reach stderr; stderr: {stderr}"
    );
    assert_eq!(
        out.status.code(),
        Some(0),
        "a reader that stops early is a normal end; stderr: {stderr}"
    );
}

/// Byte offsets of panicking **stdout** macro calls on one source line.
///
/// Every occurrence is examined, not just the first: `println!(` also occurs
/// inside `eprintln!(`, so a line carrying a stderr call before a real stdout
/// one would otherwise be cleared by its first match and the real call never
/// seen. A gate meant to be permanent proof cannot have a false negative.
fn panicking_stdout_macros(line: &str) -> Vec<usize> {
    let mut hits = Vec::new();
    if line.trim_start().starts_with("//") {
        return hits;
    }
    for mac in ["print!(", "println!("] {
        let mut from = 0;
        while let Some(rel) = line[from..].find(mac) {
            let at = from + rel;
            // Stderr keeps the panicking macros by design (spec 035 §3.3).
            let is_stderr = at > 0 && line.as_bytes()[at - 1] == b'e';
            if !is_stderr {
                hits.push(at);
            }
            from = at + mac.len();
        }
    }
    hits
}

#[test]
fn scanner_does_not_let_a_stderr_call_mask_a_stdout_one() {
    assert!(panicking_stdout_macros(r#"eprintln!("x");"#).is_empty());
    assert!(panicking_stdout_macros(r#"eprint!("x");"#).is_empty());
    assert!(panicking_stdout_macros("// println!(\"a comment\");").is_empty());
    assert!(!panicking_stdout_macros(r#"println!("x");"#).is_empty());
    assert!(!panicking_stdout_macros(r#"print!("x");"#).is_empty());
    // The regression: the stderr call comes first and must not clear the line.
    assert!(!panicking_stdout_macros(r#"eprintln!("{}", x); println!("{}", y);"#).is_empty());
    assert!(!panicking_stdout_macros(r#"eprint!("{}", x); print!("{}", y);"#).is_empty());
}

/// Spec 035 §3.5(3). The block path (`index render`, `index coverage`) cannot be
/// exercised by a pipe-breaking test: its output fits inside a pipe buffer on
/// any corpus small enough to build in one, so such a test could never fail.
/// The guarantee is asserted structurally instead. This is the check that would
/// have caught the two `print!` sites a line-only migration left behind.
#[test]
fn no_panicking_stdout_macro_remains_in_the_cli() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut offenders: Vec<String> = Vec::new();

    // Recursive: `src/` is flat today, but a future `src/util/` must not escape
    // the enforcement by being invisible to it.
    let mut dirs = vec![src.clone()];
    let mut files: Vec<std::path::PathBuf> = Vec::new();
    while let Some(dir) = dirs.pop() {
        for entry in fs::read_dir(&dir).unwrap() {
            let p = entry.unwrap().path();
            if p.is_dir() {
                dirs.push(p);
            } else {
                files.push(p);
            }
        }
    }

    for path in files {
        if path.extension().is_none_or(|e| e != "rs") {
            continue;
        }
        let text = fs::read_to_string(&path).unwrap();
        for (n, raw) in text.lines().enumerate() {
            let line = raw.trim_start();
            if !panicking_stdout_macros(line).is_empty() {
                offenders.push(format!(
                    "{}:{}: {line}",
                    path.strip_prefix(&src).unwrap_or(&path).display(),
                    n + 1
                ));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "CLI stdout must go through out.rs, not a panicking macro (spec 035):\n{}",
        offenders.join("\n")
    );
}

// ===== spec 037: machine-readable verdicts =====

/// A minimal governed repo the six adjudicating verbs all have something to say
/// about: one crate claimed by spec `001-a`, compiled and indexed.
fn verdict_fixture(root: &Path) {
    let w = |rel: &str, content: &str| {
        let p = root.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, content).unwrap();
    };
    w("Cargo.toml", "[workspace]\nmembers = [\"crate-a\"]\n");
    w(
        "crate-a/Cargo.toml",
        "[package]\nname = \"crate-a\"\nversion = \"0.1.0\"\n\
         [package.metadata.spec-spine]\nspec = \"001-a\"\n",
    );
    w("crate-a/src/lib.rs", "pub fn a() {}\n");
    w(
        "specs/001-a/spec.md",
        "---\nid: \"001-a\"\ntitle: \"A\"\nstatus: approved\ncreated: \"2026-06-09\"\n\
         summary: \"s\"\nestablishes:\n  - \"crate-a/src/lib.rs\"\n---\n# 001-a\n## body\n",
    );
    for verb in ["compile", "index"] {
        let out = bin().arg("--repo").arg(root).arg(verb).output().unwrap();
        assert_eq!(code(&out), 0, "fixture {verb}: {:?}", out.status);
    }
}

fn run_in(root: &Path, args: &[&str]) -> std::process::Output {
    bin()
        .arg("--repo")
        .arg(root)
        .args(args)
        .output()
        .expect("spawn spec-spine")
}

/// Write `paths` for `couple --paths-from` and return the argument.
fn changed_paths(root: &Path, paths: &[&str]) -> std::path::PathBuf {
    let p = root.join("changed.txt");
    fs::write(&p, format!("{}\n", paths.join("\n"))).unwrap();
    p
}

fn envelope(out: &std::process::Output) -> serde_json::Value {
    let stdout = String::from_utf8_lossy(&out.stdout);
    serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!(
            "stdout is not one JSON document ({e}); stdout: {stdout}; stderr: {}",
            String::from_utf8_lossy(&out.stderr)
        )
    })
}

/// Spec 037 3.1: one envelope shape across all six adjudicating verbs, with a
/// `report` member and no `error` member on a corpus that passes.
#[test]
fn json_envelope_on_every_adjudicating_verb() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    verdict_fixture(root);
    let paths = changed_paths(root, &["crate-a/src/lib.rs", "specs/001-a/spec.md"]);
    let paths = paths.to_str().unwrap();

    let cases: [(&str, Vec<&str>); 6] = [
        ("compile.check", vec!["compile", "--check", "--json"]),
        ("index.check", vec!["index", "check", "--json"]),
        ("lint", vec!["lint", "--json"]),
        ("couple", vec!["couple", "--paths-from", paths, "--json"]),
        ("attest", vec!["attest", "--json"]),
        (
            "verify-attestation",
            vec!["verify-attestation", "--recompute", "--json"],
        ),
    ];

    for (verb, args) in cases {
        let out = run_in(root, &args);
        assert_eq!(
            code(&out),
            0,
            "{verb}: {:?}",
            String::from_utf8_lossy(&out.stderr)
        );
        let v = envelope(&out);
        assert_eq!(v["schemaVersion"], "0.1.0", "{verb}");
        assert_eq!(v["verb"], verb);
        assert_eq!(v["ok"], true, "{verb}");
        assert_eq!(v["exitCode"], 0, "{verb}");
        assert!(v.get("report").is_some(), "{verb} must carry a report");
        assert!(v.get("error").is_none(), "{verb} must carry no error");
    }
}

/// Spec 037 3.2: `--json` changes what is written, never what is decided. Every
/// failure mode returns the code the prose form returns, and `ok` agrees.
#[test]
fn json_exit_codes_match_the_prose_form() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    verdict_fixture(root);

    // Drift: a claimed path changed without its owning spec (exit 1).
    let paths = changed_paths(root, &["crate-a/src/lib.rs"]);
    let paths = paths.to_str().unwrap();
    let prose = run_in(root, &["couple", "--paths-from", paths]);
    let json = run_in(root, &["couple", "--paths-from", paths, "--json"]);
    assert_eq!(code(&prose), 1);
    assert_eq!(code(&json), code(&prose), "couple drift");
    let v = envelope(&json);
    assert_eq!(v["exitCode"], 1);
    assert_eq!(v["ok"], false);
    assert!(
        !v["report"]["violations"].as_array().unwrap().is_empty(),
        "the reasons ride in the report, not in prose"
    );

    // Staleness: edit a spec (a hashed input to both trees) without rerunning
    // `compile`/`index`, so one edit exercises both freshness gates (exit 2).
    let spec = root.join("specs/001-a/spec.md");
    let body = fs::read_to_string(&spec).unwrap();
    fs::write(&spec, body.replace("## body", "## body edited")).unwrap();

    for args in [
        ["index", "check"].as_slice(),
        ["compile", "--check"].as_slice(),
    ] {
        let prose = run_in(root, args);
        let mut json_args = args.to_vec();
        json_args.push("--json");
        let json = run_in(root, &json_args);
        assert_eq!(code(&prose), 2, "{args:?} prose");
        assert_eq!(code(&json), code(&prose), "{args:?} json");
        let v = envelope(&json);
        assert_eq!(v["ok"], false, "{args:?}");
        assert_eq!(v["report"]["fresh"], false, "{args:?}");
        assert!(
            v["report"]["expected"].is_string(),
            "{args:?}: the stale detail rides in the report"
        );
    }
}

/// Spec 037 3.1: `report` is the facade's payload, not a second CLI spelling of
/// it. Compared as documents rather than as strings: the envelope is canonical
/// (sorted, pretty) while the facade returns compact JSON, so the bytes of the
/// two encodings differ by construction and the claim that can hold, and that
/// the spec's own `"report": { }` example requires, is that the *payload* is one
/// shape. Any divergence in members, spelling or values fails here.
#[test]
fn json_report_equals_the_facade_payload() {
    use spec_spine_core::{
        attest_json, check_freshness_json, check_registry_freshness_json, couple_json, lint_json,
        verify_attestation_json,
    };

    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    verdict_fixture(root);
    let repo = root.to_str().unwrap();
    let parse = |s: String| serde_json::from_str::<serde_json::Value>(&s).unwrap();

    let cases: Vec<(&str, Vec<String>, serde_json::Value)> = vec![
        (
            "lint",
            vec!["lint".into(), "--json".into()],
            parse(lint_json("{}", repo).unwrap()),
        ),
        (
            "index.check",
            vec!["index".into(), "check".into(), "--json".into()],
            parse(check_freshness_json("{}", repo).unwrap()),
        ),
        (
            "compile.check",
            vec!["compile".into(), "--check".into(), "--json".into()],
            parse(check_registry_freshness_json("{}", repo).unwrap()),
        ),
        (
            "attest",
            vec!["attest".into(), "--json".into()],
            parse(attest_json("{}", repo, false).unwrap()),
        ),
    ];
    for (verb, args, expected) in cases {
        let args: Vec<&str> = args.iter().map(String::as_str).collect();
        let out = run_in(root, &args);
        assert_eq!(envelope(&out)["report"], expected, "{verb}");
    }

    // couple: the facade takes the diff in its request, so build the same one.
    let paths = changed_paths(root, &["crate-a/src/lib.rs"]);
    let out = run_in(
        root,
        &["couple", "--paths-from", paths.to_str().unwrap(), "--json"],
    );
    let request = serde_json::json!({
        "repoRoot": repo,
        "diff": { "files": [{ "path": "crate-a/src/lib.rs", "hunks": [], "deleted": false }] },
    });
    let expected = parse(couple_json(&request.to_string()).unwrap());
    assert_eq!(envelope(&out)["report"], expected, "couple");

    // verify-attestation: `--recompute` alone is the mode the facade models, so
    // its report is the facade's payload exactly. `--signature` has no facade
    // counterpart and contributes an additive `signature` member (spec 037 3.2
    // requires the envelope to report every verdict the prose reports).
    let attestation: serde_json::Value = serde_json::from_slice(
        &fs::read(root.join(".derived/attestation/attestation.json")).unwrap(),
    )
    .unwrap();
    let out = run_in(root, &["verify-attestation", "--recompute", "--json"]);
    let request = serde_json::json!({ "repoRoot": repo, "attestation": attestation });
    let expected = parse(verify_attestation_json(&request.to_string()).unwrap());
    assert_eq!(envelope(&out)["report"], expected, "verify-attestation");
}

/// Spec 037 3.3: a failure is an envelope on stdout with the mapped exit code,
/// a stable `kind`, and no `report`; stdout carries nothing else.
#[test]
fn json_error_path_is_an_envelope_on_stdout() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    verdict_fixture(root);

    // A malformed spec-spine.toml is Error::Config -> exit 3.
    fs::write(root.join("spec-spine.toml"), "[layout\n").unwrap();
    let out = run_in(root, &["lint", "--json"]);
    assert_eq!(code(&out), 3);
    let v = envelope(&out);
    assert_eq!(v["verb"], "lint");
    assert_eq!(v["exitCode"], 3);
    assert_eq!(v["ok"], false);
    assert_eq!(v["error"]["kind"], "config");
    assert!(v.get("report").is_none(), "error and report are exclusive");
    assert!(
        v["error"]["message"].as_str().unwrap().len() > 1,
        "the message is human text, present but unpromised"
    );

    // The writing form of `compile` has no machine-readable verdict (3 4), and
    // says so as an envelope rather than as an unparseable sentence.
    fs::remove_file(root.join("spec-spine.toml")).unwrap();
    let out = run_in(root, &["compile", "--json"]);
    assert_eq!(code(&out), 3);
    assert_eq!(envelope(&out)["error"]["kind"], "config");
}

/// Spec 037 3.5: the envelope goes through the closed-reader write, on every
/// verb. `spec-spine <verb> --json | head` is a `0`, not a `101`.
#[test]
fn json_survives_a_closed_reader_on_every_verb() {
    use std::io::Read;
    use std::process::Stdio;

    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    verdict_fixture(root);
    let paths = changed_paths(root, &["crate-a/src/lib.rs"]);
    let paths = paths.to_string_lossy().into_owned();

    let cases: [Vec<&str>; 6] = [
        vec!["compile", "--check", "--json"],
        vec!["index", "check", "--json"],
        vec!["lint", "--json"],
        vec!["couple", "--paths-from", &paths, "--json"],
        vec!["attest", "--json"],
        vec!["verify-attestation", "--recompute", "--json"],
    ];

    for args in cases {
        let mut child = bin()
            .arg("--repo")
            .arg(root)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let mut stdout = child.stdout.take().unwrap();
        let mut buf = [0u8; 8];
        let _ = stdout.read(&mut buf);
        drop(stdout);
        let out = child.wait_with_output().unwrap();
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            !stderr.contains("panicked"),
            "{args:?} panicked on a closed reader: {stderr}"
        );
        assert_ne!(out.status.code(), Some(101), "{args:?}");
    }
}

/// Spec 037 3.3: without the flag nothing moves. The prose forms keep their
/// stdout text, so no existing consumer is disturbed by this spec.
#[test]
fn prose_output_is_unchanged_without_the_flag() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    verdict_fixture(root);

    let compile = run_in(root, &["compile", "--check"]);
    assert!(
        String::from_utf8_lossy(&compile.stdout).starts_with("spec-registry is fresh:"),
        "{}",
        String::from_utf8_lossy(&compile.stdout)
    );
    let index = run_in(root, &["index", "check"]);
    assert_eq!(
        String::from_utf8_lossy(&index.stdout).trim(),
        "index is fresh"
    );
    let lint = run_in(root, &["lint"]);
    assert!(
        String::from_utf8_lossy(&lint.stdout).contains("lint: 0 error(s)"),
        "{}",
        String::from_utf8_lossy(&lint.stdout)
    );
    let paths = changed_paths(root, &["crate-a/src/lib.rs", "specs/001-a/spec.md"]);
    let couple = run_in(root, &["couple", "--paths-from", paths.to_str().unwrap()]);
    assert!(
        String::from_utf8_lossy(&couple.stdout).contains("no drift"),
        "{}",
        String::from_utf8_lossy(&couple.stdout)
    );
    let attest = run_in(root, &["attest"]);
    assert!(
        String::from_utf8_lossy(&attest.stdout).contains("attestationHash:"),
        "{}",
        String::from_utf8_lossy(&attest.stdout)
    );
    let verify = run_in(root, &["verify-attestation", "--recompute"]);
    assert!(
        String::from_utf8_lossy(&verify.stdout).contains("recompute: MATCH"),
        "{}",
        String::from_utf8_lossy(&verify.stdout)
    );
}
