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

/// Spec 035 §3.5(3). The block path (`index render`, `index coverage`) cannot be
/// exercised by a pipe-breaking test: its output fits inside a pipe buffer on
/// any corpus small enough to build in one, so such a test could never fail.
/// The guarantee is asserted structurally instead. This is the check that would
/// have caught the two `print!` sites a line-only migration left behind.
#[test]
fn no_panicking_stdout_macro_remains_in_the_cli() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut offenders: Vec<String> = Vec::new();

    for entry in fs::read_dir(&src).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "rs") {
            continue;
        }
        let text = fs::read_to_string(&path).unwrap();
        for (n, raw) in text.lines().enumerate() {
            let line = raw.trim_start();
            // Comments name these macros when explaining why they are not used.
            if line.starts_with("//") {
                continue;
            }
            // `print!(` also occurs inside `eprint!(`, and `println!(` inside
            // `eprintln!(`; stderr keeps the panicking macros by design (§3.3).
            for mac in ["print!(", "println!("] {
                if let Some(at) = line.find(mac) {
                    let is_stderr = at > 0 && line.as_bytes()[at - 1] == b'e';
                    if !is_stderr {
                        offenders.push(format!(
                            "{}:{}: {line}",
                            path.file_name().unwrap().to_string_lossy(),
                            n + 1
                        ));
                    }
                }
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "CLI stdout must go through out.rs, not a panicking macro (spec 035):\n{}",
        offenders.join("\n")
    );
}
