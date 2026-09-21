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
    // Slices live in their own sidecar since spec 022 (no monolithic index.json).
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
    // Sharded committed form (spec 022): one file per spec, no monolithic registry.json.
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

/// Lowercase hex SHA-256, computed here and not by the tool (spec 077 D-4).
fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::Digest;
    sha2::Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Spec 077 §3.4: one spec's `contentHash` is the path-framed construction, is
/// not the digest of its bytes, and the prose line says so; `attest --spec`'s
/// `specSourceHash` is the unframed one. The inequality separates the two
/// constructions, and the prose assertion is the one that fails on the gloss
/// the defect was (`(sha256 of this spec.md)`), since no value here changed.
#[test]
fn content_hash_is_path_framed_and_spec_source_hash_is_not() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    verdict_fixture(root);
    // CRLF and a BOM, so the pin covers normalization and not only framing.
    let rel = "specs/001-a/spec.md";
    let text = fs::read_to_string(root.join(rel)).unwrap();
    let raw = format!("\u{feff}{}", text.replace('\n', "\r\n"));
    fs::write(root.join(rel), &raw).unwrap();
    for verb in ["compile", "index"] {
        assert_eq!(code(&run_in(root, &[verb])), 0, "{verb}");
    }
    let normalized = text.as_bytes();
    let mut framed = rel.as_bytes().to_vec();
    framed.push(0);
    framed.extend_from_slice(normalized);

    let show = envelope(&run_in(root, &["registry", "show", "001-a", "--json"]));
    let content_hash = show["contentHash"]
        .as_str()
        .expect("contentHash")
        .to_string();
    assert_eq!(content_hash, sha256_hex(&framed), "(1) framed by the path");
    assert_ne!(
        content_hash,
        sha256_hex(normalized),
        "(2) not the bare digest"
    );

    let attest = envelope(&run_in(root, &["attest", "--spec", "001-a", "--json"]));
    assert_eq!(
        attest["report"]["attestation"]["specSourceHash"],
        sha256_hex(normalized),
        "(3) specSourceHash is the unframed digest"
    );

    let prose = run_in(root, &["registry", "show", "001-a"]);
    let stdout = String::from_utf8_lossy(&prose.stdout);
    let line = stdout
        .lines()
        .find(|l| l.starts_with("contentHash:"))
        .unwrap_or_else(|| panic!("a contentHash line: {stdout}"));
    assert!(line.contains(&content_hash), "{line}");
    assert!(
        !line.contains("sha256 of this spec.md"),
        "(4) the pre-096 gloss named the wrong construction: {line}"
    );
    assert!(
        line.contains("path") && line.contains("NUL"),
        "(4) names the framing: {line}"
    );
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

    // JSON form: the id strings, same order, under `items` in a versioned read
    // document (spec 094 §3.6, amending 010 §3.1).
    let json = bin()
        .arg("--repo")
        .arg(tmp.path())
        .args(["registry", "list", "--ids-only", "--json"])
        .output()
        .unwrap();
    assert_eq!(code(&json), 0);
    let doc: serde_json::Value = serde_json::from_slice(&json.stdout).unwrap();
    let ids: Vec<String> = serde_json::from_value(doc["items"].clone()).unwrap();
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
    let doc: serde_json::Value = serde_json::from_slice(&filtered_json.stdout).unwrap();
    let none: Vec<String> = serde_json::from_value(doc["items"].clone()).unwrap();
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
    // Sharded committed form (spec 022): per-spec shard, no monolithic index.json.
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
    // Spec 052 §3.1: two named groups, not one flat list. This fixture indexes
    // without compiling, so no registry is committed and every orphan reads as
    // in flight, which is the "cannot say otherwise" rule rather than a
    // failure: a read verb that answered from the index alone must not start
    // failing because a different artifact is missing.
    let orphans_out = String::from_utf8_lossy(&orphans_text.stdout);
    assert!(orphans_out.contains("in flight"), "{orphans_out}");
    assert!(orphans_out.contains("002-b"), "{orphans_out}");

    let orphans_json = bin()
        .arg("--repo")
        .arg(tmp.path())
        .args(["index", "orphans", "--json"])
        .output()
        .unwrap();
    assert_eq!(code(&orphans_json), 0);
    let partitioned: serde_json::Value = serde_json::from_slice(&orphans_json.stdout).unwrap();
    assert_eq!(partitioned["orphaned"], serde_json::json!([]));
    assert_eq!(partitioned["inFlight"], serde_json::json!(["002-b"]));

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
    // Spec 052 §3.1: with both groups empty the verb stays silent, as it did
    // before the partition.
    assert!(
        none.stdout.is_empty(),
        "{}",
        String::from_utf8_lossy(&none.stdout)
    );
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
    // Spec 028 3.2: 0 fresh, 1 validation failed, 2 stale. Validation outranks
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
    // Spec 029: `index coverage` is a freshness-guarded read verb over the
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

// ===== spec 032: a reader that stops early is not an error =====

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
            // Stderr keeps the panicking macros by design (spec 032 §3.3).
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

/// Spec 032 §3.5(3). The block path (`index render`, `index coverage`) cannot be
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
        "CLI stdout must go through out.rs, not a panicking macro (spec 032):\n{}",
        offenders.join("\n")
    );
}

// ===== spec 034: machine-readable verdicts =====

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

/// Spec 034 3.1: one envelope shape across all six adjudicating verbs, with a
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
        // Spec 043 took the envelope to 0.2.0 by adding the `verify` verb. The
        // assertion tracks the constant rather than a literal so that a future
        // additive verb does not read as a change to these six verbs.
        assert_eq!(
            v["schemaVersion"],
            spec_spine_types::VERDICT_SCHEMA_VERSION,
            "{verb}"
        );
        assert_eq!(v["verb"], verb);
        assert_eq!(v["ok"], true, "{verb}");
        assert_eq!(v["exitCode"], 0, "{verb}");
        assert!(v.get("report").is_some(), "{verb} must carry a report");
        assert!(v.get("error").is_none(), "{verb} must carry no error");
    }
}

/// Spec 034 3.2: `--json` changes what is written, never what is decided. Every
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

/// Spec 034 3.1: `report` is the facade's payload, not a second CLI spelling of
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
    // counterpart and contributes an additive `signature` member (spec 034 3.2
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

/// Spec 034 3.3: a failure is an envelope on stdout with the mapped exit code,
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

    // A validation failure is the one error class with a structured payload,
    // and it must survive the generic error path: a consumer that reads
    // `lint --json`'s violation array on exit 1 gets the same data here rather
    // than a bare sentence and a fallback to parsing stderr.
    fs::write(
        root.join("specs/001-a/spec.md"),
        "---\nid: \"999-mismatched\"\ntitle: \"A\"\nstatus: approved\n\
         created: \"2026-06-09\"\nsummary: \"s\"\n---\n# x\n",
    )
    .unwrap();
    let prose = run_in(root, &["compile", "--check"]);
    let out = run_in(root, &["compile", "--check", "--json"]);
    assert_eq!(code(&prose), 1, "id must match the directory name");
    assert_eq!(code(&out), code(&prose), "the flag does not move the code");
    let v = envelope(&out);
    assert_eq!(v["error"]["kind"], "validation");
    let violations = v["error"]["violations"].as_array().unwrap();
    assert!(!violations.is_empty(), "{v}");
    assert!(
        violations[0]["code"].as_str().unwrap().starts_with("V-"),
        "{v}"
    );
    // ...and stdout is the only channel written, since the envelope already
    // carries what the prose form puts on stderr.
    assert!(
        out.stderr.is_empty(),
        "no second channel under --json: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !prose.stderr.is_empty(),
        "the prose form still reports each violation on stderr"
    );
}

/// Spec 034 3.5: the envelope goes through the closed-reader write, on every
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

/// Spec 034 3.3: without the flag nothing moves. The prose forms keep their
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
    // Spec 050 §3.3 adds one line under the verdict when the ledger has a gap,
    // so the assertion is on the verdict line rather than on the whole stream.
    // The fixture has one claimed-but-unwitnessed path, which is what that line
    // reports; the verdict itself is untouched, which is what 037 §3.3 is about.
    let index_out = String::from_utf8_lossy(&index.stdout);
    assert_eq!(
        index_out.lines().next(),
        Some("index is fresh"),
        "{index_out}"
    );
    assert!(index_out.contains("unwitnessed claims: 1"), "{index_out}");
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

/// Spec 034 D-2: `--signature` is the mode the facade does not model, and the
/// additive `signature` member is the only payload shape in this spec with no
/// facade counterpart to pin it. Exercised end to end so a rename is caught.
///
/// The public key is recovered from the seal's own `keyId`, which spec 021
/// defines as the hex public key, so the test needs no key derivation of its
/// own and stays a pure round-trip through the two commands.
#[test]
fn json_verify_attestation_reports_the_signature_mode() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    verdict_fixture(root);

    let seed = root.join("signing.key");
    fs::write(&seed, [7u8; 32]).unwrap();
    let signed = run_in(root, &["attest", "--sign", "--key", seed.to_str().unwrap()]);
    assert_eq!(
        code(&signed),
        0,
        "{}",
        String::from_utf8_lossy(&signed.stderr)
    );

    let seal: serde_json::Value = serde_json::from_slice(
        &fs::read(root.join(".derived/attestation/attestation.sig")).unwrap(),
    )
    .unwrap();
    let key_id = seal["keyId"].as_str().unwrap().to_string();
    let public = root.join("public.hex");
    fs::write(&public, &key_id).unwrap();

    // Both modes at once: the report carries the facade's `outcome` and the
    // additive `signature` member side by side.
    let out = run_in(
        root,
        &[
            "verify-attestation",
            "--recompute",
            "--signature",
            "--public-key",
            public.to_str().unwrap(),
            "--json",
        ],
    );
    assert_eq!(code(&out), 0, "{}", String::from_utf8_lossy(&out.stderr));
    let v = envelope(&out);
    assert_eq!(v["ok"], true);
    assert_eq!(v["report"]["outcome"], "match");
    assert_eq!(v["report"]["signature"]["valid"], true);
    assert_eq!(v["report"]["signature"]["keyId"], key_id.as_str());

    // Signature-only: no `outcome`, because recompute did not run.
    let out = run_in(
        root,
        &[
            "verify-attestation",
            "--signature",
            "--public-key",
            public.to_str().unwrap(),
            "--json",
        ],
    );
    assert_eq!(code(&out), 0);
    let v = envelope(&out);
    assert!(v["report"].get("outcome").is_none(), "{v}");
    assert_eq!(v["report"]["signature"]["valid"], true);

    // A wrong key is a failed verification: exit 1, and the envelope says so
    // rather than merely omitting the good news.
    fs::write(&public, "00".repeat(32)).unwrap();
    let out = run_in(
        root,
        &[
            "verify-attestation",
            "--signature",
            "--public-key",
            public.to_str().unwrap(),
            "--json",
        ],
    );
    assert_eq!(code(&out), 1);
    let v = envelope(&out);
    assert_eq!(v["ok"], false);
    assert_eq!(v["exitCode"], 1);
    assert_eq!(v["report"]["signature"]["valid"], false);
}

/// Spec 034 3.3: a `verify-attestation` with no mode selected is a config
/// error, not an affirmative `ok: true` over an empty report.
#[test]
fn json_verify_attestation_with_no_mode_is_an_error_envelope() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    verdict_fixture(root);
    let out = run_in(root, &["verify-attestation", "--json"]);
    assert_eq!(code(&out), 3);
    let v = envelope(&out);
    assert_eq!(v["ok"], false);
    assert_eq!(v["error"]["kind"], "config");
    assert!(v.get("report").is_none());
}

// ===== spec 035: `registry plan` =====

/// The scheduling projection, end to end: prose lists the ready set and counts
/// the rest, `--json` carries every blocker with the state that made it one.
///
/// Emitted **bare**, like every other `registry` projection: spec 034's verdict
/// envelope wraps the adjudicating verbs, and 037 4 keeps it off the read verbs,
/// so `plan` joins them rather than splitting the group into two output shapes.
#[test]
fn registry_plan_partitions_the_corpus() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let spec = |id: &str, body: &str| {
        let dir = root.join("specs").join(id);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("spec.md"),
            format!(
                "---\nid: \"{id}\"\ntitle: \"T\"\nstatus: approved\ncreated: \"2026-09-06\"\n\
                 summary: \"s\"\n{body}---\n# {id}\n"
            ),
        )
        .unwrap();
    };
    spec("001-done", "implementation: complete\n");
    spec(
        "002-now",
        "implementation: pending\ndepends_on: [\"001-done\"]\n",
    );
    spec(
        "003-later",
        "implementation: pending\ndepends_on: [\"002-now\"]\n",
    );
    assert_eq!(code(&run_in(root, &["compile"])), 0);

    let prose = run_in(root, &["registry", "plan"]);
    assert_eq!(
        code(&prose),
        0,
        "{}",
        String::from_utf8_lossy(&prose.stderr)
    );
    let text = String::from_utf8_lossy(&prose.stdout);
    // Spec 053 §3.1: the prose renders what the structure holds. Titles on both
    // sets, each blocked spec's reasons rather than a count, and the
    // not-schedulable remainder so the figures add up to the corpus.
    assert!(text.contains("ready (1):"), "{text}");
    assert!(text.contains("002-now  T"), "{text}");
    assert!(text.contains("blocked (1):"), "{text}");
    assert!(text.contains("blocked by 002-now (pending)"), "{text}");
    assert!(
        text.contains("3 specs: 1 ready, 1 blocked, 1 not schedulable"),
        "{text}"
    );
    assert!(
        !text.contains("001-done"),
        "a finished spec is not offered: {text}"
    );

    let out = run_in(root, &["registry", "plan", "--json"]);
    assert_eq!(code(&out), 0);
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    // A read document, not a spec 034 envelope: the report's members at the top
    // level, versioned on the read axis (spec 094), with no `ok` or `report`.
    assert_eq!(
        v["schemaVersion"],
        spec_spine_types::READ_SCHEMA_VERSION,
        "{v}"
    );
    assert!(v.get("report").is_none() && v.get("ok").is_none(), "{v}");
    // Spec 053 §3.3: ready entries are objects carrying the title, and blocked
    // entries gain one additively. The breaking half is deliberate: a parallel
    // titles array to be zipped by position is the shape that generates the
    // join code this spec exists to delete.
    assert_eq!(
        v["ready"],
        serde_json::json!([{ "id": "002-now", "title": "T" }])
    );
    assert_eq!(
        v["blocked"],
        serde_json::json!([
            {
                "id": "003-later",
                "title": "T",
                "blockedBy": [{ "id": "002-now", "state": "pending" }]
            }
        ])
    );
    assert_eq!(v["notSchedulable"], 1);

    // §3.2, as spec 094 §3.6 amends it: `--next` is the single pick, the object
    // rather than a one-element array, under a named `next` member.
    let next = run_in(root, &["registry", "plan", "--next", "--json"]);
    assert_eq!(code(&next), 0);
    let n: serde_json::Value = serde_json::from_slice(&next.stdout).unwrap();
    assert_eq!(
        n,
        serde_json::json!({
            "next": { "id": "002-now", "title": "T" },
            "schemaVersion": spec_spine_types::READ_SCHEMA_VERSION,
        })
    );

    // A corpus with nothing schedulable says so rather than printing an empty
    // page: the prose form has a reader, and "(nothing ready)" is an answer.
    let empty = tempfile::tempdir().unwrap();
    let dir = empty.path().join("specs/001-done");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("spec.md"),
        "---\nid: \"001-done\"\ntitle: \"T\"\nstatus: approved\ncreated: \"2026-09-06\"\n\
         summary: \"s\"\nimplementation: complete\n---\n# 001-done\n",
    )
    .unwrap();
    assert_eq!(code(&run_in(empty.path(), &["compile"])), 0);
    let out = run_in(empty.path(), &["registry", "plan"]);
    assert_eq!(code(&out), 0);
    let text = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        text.trim(),
        "(nothing ready), blocked: 0\n\n1 specs: 0 ready, 0 blocked, 1 not schedulable",
        "the `(nothing ready)` line is unchanged (spec 053 §3.1) and the \
         remainder follows it: on a finished corpus that figure is the whole \
         answer, and without it `blocked: 0` reads as though the specs vanished"
    );
}
// ===== spec 039: per-spec attestation =====

/// `attest --spec` writes `by-spec/<id>.json`, `--sign` seals it beside itself,
/// and `verify-attestation --spec` checks both modes back.
#[test]
fn attest_spec_writes_signs_and_verifies_one_spec() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    verdict_fixture(root);

    let out = run_in(root, &["attest", "--spec", "001-a"]);
    assert_eq!(code(&out), 0, "{}", String::from_utf8_lossy(&out.stderr));
    let path = root.join(".derived/attestation/by-spec/001-a.json");
    assert!(path.is_file(), "the payload lands under by-spec/");
    // The corpus-scoped artifact is untouched: the two scopes do not collide.
    assert!(!root.join(".derived/attestation/attestation.json").exists());

    let payload: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(payload["specId"], "001-a");
    assert_eq!(payload["schemaVersion"], "0.1.0");
    assert_eq!(payload["lifecycle"]["status"], "approved");
    assert_eq!(payload["verdicts"]["resolution"]["ok"], true);
    assert_eq!(
        payload["units"][0]["unit"]["path"], "crate-a/src/lib.rs",
        "the owning unit, with the hash of what it resolved to"
    );
    assert!(payload["units"][0]["contentHash"].is_string());

    // Sign, then verify both modes. The public key is the seal's own keyId.
    let seed = root.join("signing.key");
    fs::write(&seed, [3u8; 32]).unwrap();
    let signed = run_in(
        root,
        &[
            "attest",
            "--spec",
            "001-a",
            "--sign",
            "--key",
            seed.to_str().unwrap(),
        ],
    );
    assert_eq!(
        code(&signed),
        0,
        "{}",
        String::from_utf8_lossy(&signed.stderr)
    );
    let seal_path = root.join(".derived/attestation/by-spec/001-a.sig");
    assert!(seal_path.is_file(), "the seal is the payload's sibling");

    let seal: serde_json::Value = serde_json::from_slice(&fs::read(&seal_path).unwrap()).unwrap();
    let public = root.join("public.hex");
    fs::write(&public, seal["keyId"].as_str().unwrap()).unwrap();
    let verified = run_in(
        root,
        &[
            "verify-attestation",
            "--spec",
            "001-a",
            "--recompute",
            "--signature",
            "--public-key",
            public.to_str().unwrap(),
        ],
    );
    assert_eq!(
        code(&verified),
        0,
        "{}",
        String::from_utf8_lossy(&verified.stderr)
    );
    let text = String::from_utf8_lossy(&verified.stdout);
    assert!(text.contains("recompute: MATCH"), "{text}");
    assert!(text.contains("signature: VALID"), "{text}");

    // Editing an owned file breaks recompute with a named outcome, not silently.
    fs::write(
        root.join("crate-a/src/lib.rs"),
        "pub fn a() {}\npub fn b() {}\n",
    )
    .unwrap();
    let stale = run_in(
        root,
        &["verify-attestation", "--spec", "001-a", "--recompute"],
    );
    assert_eq!(code(&stale), 1);
    assert!(
        String::from_utf8_lossy(&stale.stderr).contains("MISMATCH"),
        "{}",
        String::from_utf8_lossy(&stale.stderr)
    );
}

/// Spec 039 3.1: **`attest`'s exit code is not a verdict.** `0` means an
/// attestation was written and nothing about what it says, for both scopes.
///
/// This is the one verb in the tool where that is true, so it is tested rather
/// than assumed: `lint`, `couple`, `index check` and `compile --check` all put
/// their verdict in the exit code. The rule reaches the corpus-scoped verb by
/// this spec's amendment to 023, so both are exercised.
#[test]
fn attest_exits_zero_on_a_false_verdict_in_both_scopes() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    verdict_fixture(root);

    // A spec claiming a file that was never written: resolution.ok is false.
    fs::write(
        root.join("specs/001-a/spec.md"),
        "---\nid: \"001-a\"\ntitle: \"A\"\nstatus: approved\ncreated: \"2026-06-09\"\n\
         summary: \"s\"\nestablishes:\n  - \"crate-a/src/lib.rs\"\n  - \"crate-a/src/never.rs\"\n\
         ---\n# 001-a\n## body\n",
    )
    .unwrap();
    assert_eq!(code(&run_in(root, &["compile"])), 0);
    assert_eq!(code(&run_in(root, &["index"])), 0);

    let scoped = run_in(root, &["attest", "--spec", "001-a"]);
    assert_eq!(
        code(&scoped),
        0,
        "a record is written whatever it says: {}",
        String::from_utf8_lossy(&scoped.stderr)
    );
    let payload: serde_json::Value = serde_json::from_slice(
        &fs::read(root.join(".derived/attestation/by-spec/001-a.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        payload["verdicts"]["resolution"]["ok"], false,
        "the false verdict is recorded, not suppressed"
    );

    // The corpus scope too, which is 023's territory and reaches this rule by
    // amendment: it exits 0 even with a failing verdict inside.
    let corpus = run_in(root, &["attest"]);
    assert_eq!(
        code(&corpus),
        0,
        "{}",
        String::from_utf8_lossy(&corpus.stderr)
    );
}

/// An unknown spec id is `NotFound` (exit 1) and writes nothing, rather than a
/// payload over a spec that does not exist.
#[test]
fn attest_spec_refuses_an_unknown_id() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    verdict_fixture(root);
    let out = run_in(root, &["attest", "--spec", "999-nope"]);
    assert_eq!(code(&out), 1);
    assert!(!root.join(".derived/attestation/by-spec").exists());
}

/// `attest --spec --json` carries the same `{ attestation, attestationHash }`
/// payload the facade returns, inside spec 034's envelope.
#[test]
fn attest_spec_json_rides_in_the_verdict_envelope() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    verdict_fixture(root);

    let out = run_in(root, &["attest", "--spec", "001-a", "--json"]);
    assert_eq!(code(&out), 0);
    let v = envelope(&out);
    assert_eq!(v["verb"], "attest");
    assert_eq!(v["ok"], true);
    assert_eq!(v["report"]["attestation"]["specId"], "001-a");
    assert!(v["report"]["attestationHash"].is_string());

    // The library call passes `"{}"` (so, `Config::default()`) while the CLI
    // loads whatever config is on disk. They agree only because the fixture
    // writes none; asserted rather than assumed, so extending `verdict_fixture`
    // with a `spec-spine.toml` fails here saying why instead of as a puzzling
    // payload mismatch.
    assert!(
        !root.join("spec-spine.toml").exists(),
        "this comparison assumes the fixture is on the default config"
    );
    let expected: serde_json::Value = serde_json::from_str(
        &spec_spine_core::attest_spec_json("{}", root.to_str().unwrap(), "001-a").unwrap(),
    )
    .unwrap();
    assert_eq!(v["report"], expected, "one payload shape per verb");
}

/// Spec 039 3.1: there is no per-spec coupling verdict, so `--with-coupling`
/// combined with `--spec` is refused rather than accepted and ignored.
///
/// Accepting it would return exit 0 and a payload silently missing the verdict
/// the caller asked for, which is the skip-as-pass shape spec 021 FR-006 rules
/// out for every mode in this verb.
#[test]
fn attest_refuses_with_coupling_scoped_to_one_spec() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    verdict_fixture(root);

    let out = run_in(root, &["attest", "--spec", "001-a", "--with-coupling"]);
    assert_eq!(code(&out), 3, "a mode that cannot run fails visibly");
    let message = String::from_utf8_lossy(&out.stderr);
    assert!(message.contains("--with-coupling"), "{message}");
    assert!(message.contains("--spec"), "{message}");
    assert!(
        !root.join(".derived/attestation/by-spec").exists(),
        "and writes nothing"
    );

    // Each flag alone still works.
    assert_eq!(code(&run_in(root, &["attest", "--spec", "001-a"])), 0);
    assert_eq!(code(&run_in(root, &["attest", "--with-coupling"])), 0);
}

/// A `--spec` id is one path segment, so it cannot walk out of the attestation
/// directory into an unrelated file.
#[test]
fn verify_attestation_refuses_a_traversing_spec_id() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    verdict_fixture(root);

    for id in ["../../etc/passwd", "..", "a/b", ""] {
        let out = run_in(root, &["verify-attestation", "--spec", id, "--recompute"]);
        assert_eq!(code(&out), 3, "id {id:?} must be refused");
        assert!(
            String::from_utf8_lossy(&out.stderr).contains("not a spec id"),
            "id {id:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    // A real id still works, so the guard is not simply refusing everything.
    assert_eq!(code(&run_in(root, &["attest", "--spec", "001-a"])), 0);
    assert_eq!(
        code(&run_in(
            root,
            &["verify-attestation", "--spec", "001-a", "--recompute"]
        )),
        0
    );
}

/// The seal is derived from the attestation's own filename, not a fixed name.
///
/// Spec 021 resolved a missing `--seal` to `attestation.sig` beside the payload.
/// A per-spec attestation lives at `by-spec/<id>.json`, so a fixed name would
/// give every spec in a corpus the same seal path and each signing would
/// overwrite the last (spec 039 D-5).
#[test]
fn the_seal_path_follows_the_attestation_it_signs() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    verdict_fixture(root);
    let seed = root.join("signing.key");
    fs::write(&seed, [5u8; 32]).unwrap();
    let key = seed.to_str().unwrap();

    // The corpus default is unchanged: attestation.json -> attestation.sig.
    assert_eq!(code(&run_in(root, &["attest", "--sign", "--key", key])), 0);
    assert!(root.join(".derived/attestation/attestation.sig").is_file());

    // Two specs seal to two distinct paths rather than overwriting each other.
    fs::write(root.join("crate-a/src/other.rs"), "pub fn b() {}\n").unwrap();
    fs::create_dir_all(root.join("specs/002-b")).unwrap();
    fs::write(
        root.join("specs/002-b/spec.md"),
        "---\nid: \"002-b\"\ntitle: \"B\"\nstatus: approved\ncreated: \"2026-06-09\"\n\
         summary: \"s\"\nestablishes:\n  - \"crate-a/src/other.rs\"\n---\n# 002-b\n## body\n",
    )
    .unwrap();
    for id in ["001-a", "002-b"] {
        assert_eq!(
            code(&run_in(
                root,
                &["attest", "--spec", id, "--sign", "--key", key]
            )),
            0,
            "sign {id}"
        );
        assert!(
            root.join(format!(".derived/attestation/by-spec/{id}.sig"))
                .is_file(),
            "{id} seals beside its own payload"
        );
    }
}

// --- `verify` (spec 043) --------------------------------------------------

/// Write a spec whose `## Verification` section holds `section` verbatim.
fn write_verify_spec(root: &Path, dir: &str, section: &str) {
    let spec_dir = root.join("specs").join(dir);
    fs::create_dir_all(&spec_dir).unwrap();
    let body = format!(
        "---\nid: \"{dir}\"\ntitle: \"T\"\nstatus: approved\ncreated: \"2026-09-06\"\nsummary: \"s\"\n---\n# {dir}\n\n## Verification\n\n{section}\n"
    );
    fs::write(spec_dir.join("spec.md"), body).unwrap();
}

#[test]
fn verify_runs_commands_and_reports_outcomes() {
    let tmp = tempfile::tempdir().unwrap();
    write_verify_spec(tmp.path(), "001-pass", "```verify:cli\ntrue\ntrue\n```");
    write_verify_spec(
        tmp.path(),
        "002-fail",
        "```verify:cli\ntrue\nexit 7\ntrue\n```",
    );
    write_verify_spec(tmp.path(), "003-prose", "- a prose bullet only");
    let run = |args: &[&str]| {
        bin()
            .arg("--repo")
            .arg(tmp.path())
            .args(args)
            .output()
            .unwrap()
    };

    // Every command exits 0 -> passed, exit 0.
    let out = run(&["verify", "001-pass"]);
    assert_eq!(code(&out), 0);
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("passed (2 command(s))"),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );

    // A failure is exit 1, NOT the command's own 7 (spec 043 3.3): the
    // documented exit contract has no entry for 7.
    let out = run(&["verify", "002-fail"]);
    assert_eq!(code(&out), 1, "a failing command is a drift-tier 1");

    // Nothing declared is an honest zero.
    assert_eq!(code(&run(&["verify", "003-prose"])), 0);

    // A missing spec is 1 (not found), never 2 (stale).
    assert_eq!(code(&run(&["verify", "404-gone"])), 1);
}

#[test]
fn verify_json_is_a_verdict_envelope_that_agrees_with_the_exit_code() {
    let tmp = tempfile::tempdir().unwrap();
    // The command writes to BOTH of its streams (spec 090 §3.5). With `true`
    // here, as this fixture read until spec 090, the `expect` below could not
    // fail: nothing was ever in front of the envelope for it to trip on. The
    // markers are assembled by the child so they appear in its output and not in
    // the command text the envelope echoes.
    write_verify_spec(
        tmp.path(),
        "001-pass",
        "```verify:cli\nprintf 'O%sT-F\\n' U; printf 'E%sR-F\\n' R >&2\n```",
    );
    write_verify_spec(
        tmp.path(),
        "002-fail",
        "```verify:cli\ntrue\nexit 7\n```\n\n```verify:browser\nclick\n```",
    );
    write_verify_spec(tmp.path(), "003-prose", "- prose");
    let json = |id: &str| -> (i32, serde_json::Value) {
        let out = bin()
            .arg("--repo")
            .arg(tmp.path())
            .args(["verify", id, "--json"])
            .output()
            .unwrap();
        (
            code(&out),
            serde_json::from_slice(&out.stdout).expect("stdout is one JSON envelope"),
        )
    };

    let (c, v) = json("001-pass");
    assert_eq!(c, 0);
    assert_eq!(v["verb"], "verify");
    // The constant, not a literal: an additive verb elsewhere is not a change
    // to this one's envelope (spec 049 bumped it to 0.3.0 for `compile.spec`).
    assert_eq!(v["schemaVersion"], spec_spine_types::VERDICT_SCHEMA_VERSION);
    assert_eq!(v["ok"], true);
    assert_eq!(v["exitCode"], 0);
    assert_eq!(v["report"]["outcome"], "passed");
    assert_eq!(v["report"]["declared"], true);
    assert_eq!(v["report"]["ran"], 1);
    assert_eq!(v["report"]["total"], 1);

    // The failing command's own code lives in the payload, and `ran` stops at it.
    let (c, v) = json("002-fail");
    assert_eq!(c, 1);
    assert_eq!(v["exitCode"], 1, "the envelope never advertises 7");
    assert_eq!(
        v["report"]["failure"]["exitCode"], 7,
        "but the payload keeps it"
    );
    assert_eq!(v["report"]["failure"]["index"], 2);
    assert_eq!(v["report"]["failure"]["command"], "exit 7");
    assert_eq!(v["report"]["ran"], 2);
    assert_eq!(v["report"]["total"], 2);
    assert_eq!(v["report"]["skipped"][0]["tag"], "verify:browser");

    // not-declared is distinguishable from a pass without parsing prose.
    let (c, v) = json("003-prose");
    assert_eq!(c, 0);
    assert_eq!(v["ok"], true);
    assert_eq!(v["report"]["outcome"], "not-declared");
    assert_eq!(v["report"]["declared"], false);
}

#[test]
fn verify_runs_from_the_repo_root_and_stops_at_the_first_failure() {
    let tmp = tempfile::tempdir().unwrap();
    // Each command appends to a file, so the transcript is checkable on disk.
    write_verify_spec(
        tmp.path(),
        "001-order",
        "```verify:cli\nprintf a >> log.txt\nfalse\nprintf c >> log.txt\n```",
    );
    let out = bin()
        .arg("--repo")
        .arg(tmp.path())
        .args(["verify", "001-order"])
        .output()
        .unwrap();
    assert_eq!(code(&out), 1);
    // `a` ran (relative path resolved against the repo root); `c` never did.
    let log = fs::read_to_string(tmp.path().join("log.txt")).unwrap();
    assert_eq!(log, "a", "later commands must not run after a failure");
}

#[test]
fn verify_plan_reads_without_running() {
    let tmp = tempfile::tempdir().unwrap();
    write_verify_spec(
        tmp.path(),
        "001-p",
        "```verify:cli\nprintf ran >> side_effect.txt\n# a comment\nsecond\n```",
    );
    let out = bin()
        .arg("--repo")
        .arg(tmp.path())
        .args(["verify", "001-p", "--plan"])
        .output()
        .unwrap();
    assert_eq!(code(&out), 0);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        stdout.lines().collect::<Vec<_>>(),
        ["printf ran >> side_effect.txt", "second"],
        "comments are stripped, both commands listed"
    );
    assert!(
        !tmp.path().join("side_effect.txt").exists(),
        "--plan must run nothing"
    );
}

#[test]
fn verify_refuses_to_re_enter_itself() {
    let tmp = tempfile::tempdir().unwrap();
    // The spec's own block runs `verify` on itself: without the spec 043 3.7
    // guard this forks without bound.
    write_verify_spec(
        tmp.path(),
        "001-loop",
        "```verify:cli\nSELF --repo REPO verify 001-loop\n```",
    );
    let spec = tmp.path().join("specs/001-loop/spec.md");
    let body = fs::read_to_string(&spec)
        .unwrap()
        .replace("SELF", env!("CARGO_BIN_EXE_spec-spine"))
        .replace("REPO", tmp.path().to_str().unwrap());
    fs::write(&spec, body).unwrap();

    let out = bin()
        .arg("--repo")
        .arg(tmp.path())
        .args(["verify", "001-loop"])
        .output()
        .unwrap();
    // The inner call is refused, so the outer command exits non-zero and the
    // outer run reports a failure. The point is that it terminates at all.
    assert_eq!(code(&out), 1);

    // The refusal itself, observed directly: an id already on the stack.
    let out = bin()
        .arg("--repo")
        .arg(tmp.path())
        .args(["verify", "001-loop"])
        .env("SPEC_SPINE_VERIFY_STACK", "001-loop")
        .output()
        .unwrap();
    assert_eq!(code(&out), 1);
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("validation"),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    // A stack that does not contain this id runs normally.
    let out = bin()
        .arg("--repo")
        .arg(tmp.path())
        .args(["verify", "001-loop", "--plan"])
        .env("SPEC_SPINE_VERIFY_STACK", "002-other")
        .output()
        .unwrap();
    assert_eq!(code(&out), 0, "only a cycle is refused, not any depth");
}

// --- `index check` diagnostics + `index diagnostics` (spec 044) -----------

/// A corpus whose only spec is in flight and claims one file that exists and
/// one that does not, so the committed index records exactly one `W-001`.
fn write_unresolved_corpus(root: &Path) {
    fs::create_dir_all(root.join("crates/a/src")).unwrap();
    fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/a\"]\n",
    )
    .unwrap();
    fs::write(
        root.join("crates/a/Cargo.toml"),
        "[package]\nname = \"a\"\nversion = \"0.1.0\"\n\n[package.metadata.spec-spine]\nspec = \"001-flight\"\n",
    )
    .unwrap();
    fs::write(
        root.join("crates/a/src/lib.rs"),
        "// Spec: specs/001-flight/spec.md\npub fn a() {}\n",
    )
    .unwrap();
    let spec_dir = root.join("specs/001-flight");
    fs::create_dir_all(&spec_dir).unwrap();
    fs::write(
        spec_dir.join("spec.md"),
        "---\nid: \"001-flight\"\ntitle: \"T\"\nstatus: draft\ncreated: \"2026-09-06\"\nimplementation: pending\nsummary: \"s\"\nestablishes:\n  - \"crates/a/src/lib.rs\"\n  - \"crates/a/src/not_yet.rs\"\n---\n# 001\n",
    )
    .unwrap();
}

#[test]
fn index_check_reports_diagnostics_and_fails_only_when_asked() {
    let tmp = tempfile::tempdir().unwrap();
    write_unresolved_corpus(tmp.path());
    let run = |args: &[&str]| {
        bin()
            .arg("--repo")
            .arg(tmp.path())
            .args(args)
            .output()
            .unwrap()
    };
    assert_eq!(code(&run(&["index"])), 0);

    // Fresh, but the ledger records an unresolved unit: reported, not refused.
    let out = run(&["index", "check"]);
    assert_eq!(code(&out), 0, "a warning must not fail the default gate");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("is fresh"), "{stdout}");
    assert!(stdout.contains("1 W-001"), "{stdout}");

    // Opt in, and the same tree is refused with 1.
    let out = run(&["index", "check", "--fail-on-unresolved"]);
    assert_eq!(code(&out), 1);

    // The refusal is said once, on one stream. Asserted because it was not:
    // an earlier round printed the reason on stdout and repeated it on stderr,
    // and these tests checked only exit codes, so nothing caught it.
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stdout.contains("--fail-on-unresolved refuses"), "{stdout}");
    assert!(
        stderr.trim().is_empty(),
        "the refusal belongs on one stream, got stderr: {stderr}"
    );
    // And a line that reads as a pass must not be the whole of what is said
    // when the process exits 1.
    assert!(
        stdout.lines().all(|l| l.trim() != "index is fresh"),
        "a bare pass line while exiting 1: {stdout}"
    );
}

#[test]
fn a_clean_corpus_keeps_the_bare_verdict_line() {
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
    assert_eq!(code(&run(&["index"])), 0);

    let out = run(&["index", "check"]);
    assert_eq!(code(&out), 0);
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim(),
        "index is fresh",
        "no diagnostics -> the line reads exactly as it did before spec 044"
    );
    // And the strict flag passes, because there is nothing unresolved.
    assert_eq!(code(&run(&["index", "check", "--fail-on-unresolved"])), 0);
}

#[test]
fn staleness_outranks_unresolution() {
    let tmp = tempfile::tempdir().unwrap();
    write_unresolved_corpus(tmp.path());
    let run = |args: &[&str]| {
        bin()
            .arg("--repo")
            .arg(tmp.path())
            .args(args)
            .output()
            .unwrap()
    };
    assert_eq!(code(&run(&["index"])), 0);
    assert_eq!(code(&run(&["index", "check", "--fail-on-unresolved"])), 1);

    // Make the ledger stale. The spec's own `spec.md` is a hashed shard input;
    // editing the claimed *source* file would not restale it, because only
    // section/symbol/module units put their backing file in the shard hash
    // (`index.rs::span_files_for_mapping`). The title changes, the units do
    // not, so the `W-001` survives and both conditions hold at once.
    let spec = tmp.path().join("specs/001-flight/spec.md");
    let body = fs::read_to_string(&spec)
        .unwrap()
        .replace("\"T\"", "\"T2\"");
    fs::write(&spec, body).unwrap();

    // Spec 044 3.3: 2, not 1. A stale ledger's warnings describe a tree that no
    // longer exists, so refusing for them would name the wrong problem.
    assert_eq!(
        code(&run(&["index", "check", "--fail-on-unresolved"])),
        2,
        "staleness must outrank unresolution"
    );
}

#[test]
fn index_check_json_carries_counts_without_disturbing_the_freshness_shape() {
    let tmp = tempfile::tempdir().unwrap();
    write_unresolved_corpus(tmp.path());
    let run = |args: &[&str]| {
        bin()
            .arg("--repo")
            .arg(tmp.path())
            .args(args)
            .output()
            .unwrap()
    };
    assert_eq!(code(&run(&["index"])), 0);

    let out = run(&["index", "check", "--json"]);
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["verb"], "index.check");
    assert_eq!(
        v["report"]["fresh"], true,
        "the freshness member is untouched"
    );
    assert_eq!(v["report"]["diagnostics"]["warnings"], 1);
    assert_eq!(v["report"]["diagnostics"]["errors"], 0);
    assert_eq!(v["report"]["diagnostics"]["byCode"]["W-001"], 1);

    // Spec 044 3.6: a payload addition does not move the envelope version.
    assert_eq!(v["schemaVersion"], spec_spine_types::VERDICT_SCHEMA_VERSION);

    // Spec 044 3.1: `compile --check` shares `freshness_report` and must not
    // have acquired a permanently-zero diagnostics member.
    let out = run(&["compile", "--check", "--json"]);
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["verb"], "compile.check");
    assert!(
        v["report"].get("diagnostics").is_none(),
        "the registry verdict must not carry index diagnostics: {}",
        v["report"]
    );
}

#[test]
fn index_diagnostics_lists_them_and_never_refuses() {
    let tmp = tempfile::tempdir().unwrap();
    write_unresolved_corpus(tmp.path());
    let run = |args: &[&str]| {
        bin()
            .arg("--repo")
            .arg(tmp.path())
            .args(args)
            .output()
            .unwrap()
    };
    assert_eq!(code(&run(&["index"])), 0);

    let out = run(&["index", "diagnostics", "--json"]);
    assert_eq!(code(&out), 0);
    let doc: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    // Spec 094 §3.6: the listing sits under `items` in a versioned object.
    let v = &doc["items"];
    assert_eq!(v.as_array().unwrap().len(), 1);
    assert_eq!(v[0]["code"], "W-001");
    assert_eq!(v[0]["specId"], "001-flight", "attributed to its spec");
    assert_eq!(v[0]["severity"], "warning");

    // The read verb reports; it does not gate, even for the strict case.
    let out = run(&["index", "diagnostics"]);
    assert_eq!(code(&out), 0);
    assert!(String::from_utf8_lossy(&out.stdout).contains("W-001"));
}

// ── spec 093: a stale binary is not a stale ledger ────────────────────────

/// §3.1: a command line clap cannot parse is exit 3, never 2. Clap's default
/// is 2, which this tool spends on staleness, so an unknown flag used to be
/// indistinguishable from a stale ledger except by matching clap's English.
#[test]
fn a_usage_error_is_exit_three_not_stale() {
    let tmp = tempfile::tempdir().unwrap();
    for args in [
        vec!["compile", "--no-such-flag"],
        vec!["no-such-verb"],
        vec!["registry", "show"],          // a required argument is missing
        vec!["index", "check", "--slice"], // a flag with no value
    ] {
        let out = run_in(tmp.path(), &args);
        assert_eq!(
            code(&out),
            3,
            "{args:?} must be a usage error, not staleness: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

/// §3.1: an incomplete invocation is a usage error too. It prints help, but
/// nobody asked for help, and a script that dropped its verb must still fail.
#[test]
fn a_missing_subcommand_is_a_usage_error() {
    let tmp = tempfile::tempdir().unwrap();
    let out = run_in(tmp.path(), &[]);
    assert_eq!(code(&out), 3, "nothing was asked for, so nothing succeeded");
}

/// §3.1: help and version are asked for, so they succeed, on stdout.
#[test]
fn help_and_version_stay_exit_zero_on_stdout() {
    let tmp = tempfile::tempdir().unwrap();
    for args in [vec!["--help"], vec!["--version"], vec!["compile", "--help"]] {
        let out = run_in(tmp.path(), &args);
        assert_eq!(code(&out), 0, "{args:?}");
        assert!(!out.stdout.is_empty(), "{args:?} writes to stdout");
    }
}

/// §3.1: and exit 2 still means staleness, from the verb that means it. The
/// property this spec buys is that 2 now means only that.
#[test]
fn exit_two_still_means_a_stale_ledger() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let dir = root.join("specs/001-a");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("spec.md"),
        "---\nid: \"001-a\"\ntitle: \"T\"\nstatus: approved\ncreated: \"2026-09-07\"\n\
         summary: \"s\"\nestablishes:\n  - \"specs/001-a/spec.md\"\n---\n# 001-a\n## body\n",
    )
    .unwrap();
    assert_eq!(code(&run_in(root, &["compile"])), 0);
    // Mutate the hashed input without recompiling.
    fs::write(
        dir.join("spec.md"),
        "---\nid: \"001-a\"\ntitle: \"T2\"\nstatus: approved\ncreated: \"2026-09-07\"\n\
         summary: \"s\"\nestablishes:\n  - \"specs/001-a/spec.md\"\n---\n# 001-a\n## body\n",
    )
    .unwrap();
    assert_eq!(
        code(&run_in(root, &["compile", "--check"])),
        2,
        "the one condition exit 2 is for"
    );
}

// ── spec 062: `spec-spine check`, both freshness reads in one verb ──────────

/// A repository with one approved spec whose committed shards are current.
fn fresh_repo() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    write_spec(tmp.path(), "001-a", "001-a", "approved");
    for verb in [&["compile"][..], &["index"][..]] {
        let out = bin()
            .arg("--repo")
            .arg(tmp.path())
            .args(verb)
            .output()
            .unwrap();
        assert_eq!(code(&out), 0, "{}", String::from_utf8_lossy(&out.stderr));
    }
    tmp
}

fn check(root: &Path, args: &[&str]) -> std::process::Output {
    bin()
        .arg("--repo")
        .arg(root)
        .arg("check")
        .args(args)
        .output()
        .unwrap()
}

/// Spec 062 3.2: the verb answers for both committed trees in one call, which
/// is the whole point. Before it, the protocol had to know two spellings to ask
/// one question: `compile --check` is a flag where `index check` is a
/// subcommand.
#[test]
fn check_reports_both_trees_and_exits_zero_when_both_are_fresh() {
    let tmp = fresh_repo();
    let out = check(tmp.path(), &[]);
    assert_eq!(code(&out), 0, "{}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("spec-registry: fresh"), "{stdout}");
    assert!(stdout.contains("codebase-index: fresh"), "{stdout}");
}

/// Spec 062 3.2: it MUST never write. A verb the protocol calls to read the
/// committed state cannot repair that state as a side effect of reading it,
/// which is how the spec 016/021 drift reached the default branch looking like
/// a local edit rather than a defect already on the branch.
#[test]
fn check_never_writes_even_when_the_tree_is_stale() {
    let tmp = fresh_repo();
    // Stale both trees by adding a spec and NOT recompiling.
    write_spec(tmp.path(), "002-b", "002-b", "approved");

    let derived = tmp.path().join(".derived");
    let snapshot = |dir: &Path| -> Vec<(String, String)> {
        let mut out = Vec::new();
        let mut stack = vec![dir.to_path_buf()];
        while let Some(d) = stack.pop() {
            for e in fs::read_dir(&d).unwrap().filter_map(Result::ok) {
                let p = e.path();
                if p.is_dir() {
                    stack.push(p);
                } else if p.file_name().unwrap() != "build-meta.json" {
                    out.push((
                        p.strip_prefix(dir).unwrap().display().to_string(),
                        fs::read_to_string(&p).unwrap_or_default(),
                    ));
                }
            }
        }
        out.sort();
        out
    };
    let before = snapshot(&derived);
    assert!(!before.is_empty(), "the fixture must have committed shards");

    let out = check(tmp.path(), &[]);
    assert_eq!(code(&out), 2, "a stale tree is exit 2");
    assert_eq!(
        before,
        snapshot(&derived),
        "`check` repaired the tree it was asked to judge"
    );
}

/// Spec 062 3.4: each tree's report reaches stderr attributed to its tree. Spec
/// 031 3.3 makes the registry stale report's structure contractual precisely
/// because the session protocol reads the drifted shard names back, and exit 2
/// alone cannot say which shard moved.
#[test]
fn check_attributes_staleness_to_the_tree_it_belongs_to() {
    let tmp = fresh_repo();
    write_spec(tmp.path(), "002-b", "002-b", "approved");
    let out = check(tmp.path(), &[]);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("spec-registry: STALE"), "{stderr}");
    assert!(
        stderr.contains("002-b"),
        "the drifted shard must still be named: {stderr}"
    );
    assert!(stderr.contains("codebase-index: STALE"), "{stderr}");
}

/// Spec 062 3.3: `1` outranks `2`, because staleness is not meaningful against
/// a corpus that does not validate. Observed here end to end rather than only
/// in the fold's unit test, which cannot see that a real invalid corpus takes
/// the intended branch.
#[test]
fn check_reports_validation_failure_ahead_of_staleness() {
    let tmp = fresh_repo();
    // A duplicate ordinal: `V-004`, an error-tier violation. It also stales the
    // committed shards, so both conditions hold at once and the fold decides.
    write_spec(tmp.path(), "001-dup", "001-dup", "approved");
    let out = check(tmp.path(), &[]);
    assert_eq!(
        code(&out),
        1,
        "validation failure must outrank staleness: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("spec-registry: INVALID"), "{stderr}");
}

/// Spec 062 3.3: `--json` changes what is written, never what is decided, and
/// the envelope carries both trees under its own verb.
#[test]
fn check_json_carries_both_halves_and_decides_nothing_differently() {
    let tmp = fresh_repo();
    write_spec(tmp.path(), "002-b", "002-b", "approved");

    let plain = check(tmp.path(), &[]);
    let json = check(tmp.path(), &["--json"]);
    assert_eq!(
        code(&plain),
        code(&json),
        "the flag must not change the exit code"
    );

    let v: serde_json::Value = serde_json::from_slice(&json.stdout).expect("an envelope");
    assert_eq!(v["verb"], "check");
    assert_eq!(v["exitCode"], 2);
    assert_eq!(v["ok"], false);
    // Both halves, each keeping its own shape rather than being flattened.
    assert_eq!(v["report"]["registry"]["fresh"], false);
    assert_eq!(v["report"]["registry"]["validationPassed"], true);
    assert!(
        v["report"]["registry"]["actual"]
            .as_str()
            .is_some_and(|s| s.contains("002-b")),
        "the registry half keeps its stale report: {v}"
    );
    assert!(
        v["report"]["index"]["diagnostics"].is_object(),
        "the index half keeps the shape `index check` emits: {v}"
    );
}

/// Spec 062 3.2: `--fail-on-unresolved` is forwarded to the index half.
/// Without it the composite gate could not adopt the verb, and the protocol
/// would still be spelling the primitive.
#[test]
fn check_forwards_fail_on_unresolved_to_the_index_half() {
    let tmp = tempfile::tempdir().unwrap();
    write_unresolved_corpus(tmp.path());
    for verb in [&["compile"][..], &["index"][..]] {
        assert_eq!(
            code(
                &bin()
                    .arg("--repo")
                    .arg(tmp.path())
                    .args(verb)
                    .output()
                    .unwrap()
            ),
            0
        );
    }
    assert_eq!(
        code(&check(tmp.path(), &[])),
        0,
        "fresh, and silent by default"
    );
    assert_eq!(
        code(&check(tmp.path(), &["--fail-on-unresolved"])),
        1,
        "an unresolved unit must refuse under the flag"
    );
}

/// Spec 064 §3.2 and §3.3: the flag turns a warning into exit 1 on every form
/// of `compile`, and reaches the composed verb, which is the only form CI runs.
///
/// The corpus carries one dangling `depends_on`, so it is valid (spec 001 §3.2)
/// and warns (`V-010`). Every assertion below is a pair: the same command with
/// and without the flag, so the flag's effect is isolated from the verb's.
#[test]
fn fail_on_warn_refuses_a_warning_on_compile_and_check() {
    let tmp = tempfile::tempdir().unwrap();
    write_spec(tmp.path(), "001-a", "001-a", "approved");
    // 002 names a spec nobody filed: V-010, warning tier.
    let dir = tmp.path().join("specs/002-b");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("spec.md"),
        "---\nid: \"002-b\"\ntitle: \"T\"\nstatus: approved\ncreated: \"2026-06-08\"\n\
         depends_on: [\"099-absent\"]\nsummary: \"s\"\n---\n# 002-b\n",
    )
    .unwrap();
    let run = |args: &[&str]| {
        bin()
            .arg("--repo")
            .arg(tmp.path())
            .args(args)
            .output()
            .unwrap()
    };

    // Writing form: unflagged 0, flagged 1. The shards are written either way.
    assert_eq!(code(&run(&["compile"])), 0);
    assert_eq!(code(&run(&["compile", "--fail-on-warn"])), 1);

    // §3.2: byte-identical emission. The flag decides an exit code, not output.
    let shard = tmp.path().join(".derived/spec-registry/by-spec/002-b.json");
    let after_plain = fs::read(&shard).unwrap();
    assert_eq!(code(&run(&["compile", "--fail-on-warn"])), 1);
    assert_eq!(
        after_plain,
        fs::read(&shard).unwrap(),
        "a refused compile must write the same bytes as an accepted one"
    );

    // --check form: fresh, so unflagged 0; flagged 1 even though nothing stales.
    assert_eq!(code(&run(&["compile", "--check"])), 0);
    assert_eq!(code(&run(&["compile", "--check", "--fail-on-warn"])), 1);

    // §3.3: the forward into the compile half of the composed verb.
    assert_eq!(code(&run(&["index"])), 0);
    assert_eq!(code(&run(&["check"])), 0);
    assert_eq!(code(&run(&["check", "--fail-on-warn"])), 1);
    // Independent flags: --fail-on-unresolved alone must not refuse a warning.
    assert_eq!(code(&run(&["check", "--fail-on-unresolved"])), 0);

    // §3.3 + spec 034: --json changes what is written, never what is decided.
    let plain = run(&["check", "--fail-on-warn"]);
    let jsonic = run(&["check", "--fail-on-warn", "--json"]);
    assert_eq!(code(&plain), code(&jsonic));
    let v: serde_json::Value = serde_json::from_slice(&jsonic.stdout).unwrap();
    assert_eq!(v["exitCode"], 1);
    assert_eq!(v["ok"], false);
    // §3.4: the tally is in the envelope, attributed to the registry half.
    assert_eq!(v["report"]["registry"]["warnings"], 1);
    assert_eq!(v["report"]["registry"]["validationPassed"], true);

    // §3.1: a clean corpus is unaffected by the flag on either verb.
    let clean = tempfile::tempdir().unwrap();
    write_spec(clean.path(), "001-a", "001-a", "approved");
    let run_clean = |args: &[&str]| {
        bin()
            .arg("--repo")
            .arg(clean.path())
            .args(args)
            .output()
            .unwrap()
    };
    assert_eq!(code(&run_clean(&["compile", "--fail-on-warn"])), 0);
    assert_eq!(code(&run_clean(&["index"])), 0);
    assert_eq!(code(&run_clean(&["check", "--fail-on-warn"])), 0);
}

/// Spec 066: `attest --spec` used to exit 3 for any spec whose unit resolved to
/// a directory, which was fourteen of this corpus's eighty-three specs. The
/// end-to-end guard is the exit code, since that is what an operator and a CI
/// job actually see.
#[test]
fn attest_spec_walks_a_claimed_subtree_instead_of_exiting_three() {
    let tmp = tempfile::tempdir().unwrap();
    let spec_dir = tmp.path().join("specs/001-t");
    fs::create_dir_all(&spec_dir).unwrap();
    // The trailing-slash `file` shorthand: the shape thirteen of the fourteen
    // affected specs use, and the one a fix keyed to `Unit::Directory` misses.
    fs::write(
        spec_dir.join("spec.md"),
        "---\nid: \"001-t\"\ntitle: \"T\"\nstatus: approved\ncreated: \"2026-09-10\"\nsummary: \"s\"\nestablishes:\n  - \"sub/\"\n---\n# 001-t\n",
    )
    .unwrap();
    fs::create_dir_all(tmp.path().join("sub/nested")).unwrap();
    fs::write(tmp.path().join("sub/a.txt"), "a\n").unwrap();
    fs::write(tmp.path().join("sub/nested/b.txt"), "b\n").unwrap();

    let out = bin()
        .arg("--repo")
        .arg(tmp.path())
        .args(["attest", "--spec", "001-t", "--json"])
        .output()
        .unwrap();

    assert_eq!(code(&out), 0, "a claimed subtree must attest, not exit 3");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.contains("\"contentHash\": null"),
        "the resolved subtree must carry a hash: {stdout}"
    );
    assert!(
        stdout.contains("\"ok\": true"),
        "the envelope reports success: {stdout}"
    );
}

// ===== spec 071: a change classified under the base's rules =====

/// `git` in `root`, with an identity and signing off, so the fixture commits
/// whatever the running user's global configuration says.
fn git088(root: &Path, args: &[&str]) {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args([
            "-c",
            "commit.gpgsign=false",
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@t",
        ])
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// The §1 scratch repository: `001-a` owns `src/a.rs` and declares two
/// acceptance commands; `main` holds it compiled and indexed.
fn delta_repo(root: &Path) {
    let w = |rel: &str, content: &[u8]| {
        let p = root.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, content).unwrap();
    };
    w(
        "specs/001-a/spec.md",
        b"---\nid: \"001-a\"\ntitle: \"a\"\nstatus: approved\ncreated: \"2026-09-11\"\nimplementation: complete\nsummary: \"s\"\nestablishes:\n  - \"src/\"\n---\n\n# a\n\n## Verification\n\n```verify:cli\ntest -f src/a.rs\n```\n",
    );
    w("src/a.rs", b"pub fn a() {}\n");
    // Claimed by nobody, so a bypass prefix decides it: `src/` is claimed, and
    // a claim outranks a prefix (spec 008), so only this path can show whose
    // configuration classified.
    w("tools/x.sh", b"echo x\n");
    w(".gitignore", b".derived/**/build-meta.json\n");
    for verb in ["compile", "index"] {
        assert_eq!(code(&run_in(root, &[verb])), 0, "fixture {verb}");
    }
    git088(root, &["init", "-q", "-b", "main"]);
    git088(root, &["add", "-A"]);
    git088(root, &["commit", "-q", "-m", "base"]);
}

fn delta_json_in(root: &Path, head: &str, tmpdir: &Path) -> std::process::Output {
    bin()
        .arg("--repo")
        .arg(root)
        .args(["delta", "--base", "main", "--head", head, "--json"])
        .env("TMPDIR", tmpdir)
        .output()
        .unwrap()
}

fn change_classes(report: &serde_json::Value, path: &str) -> Vec<String> {
    report["changes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["path"] == path)
        .unwrap_or_else(|| panic!("no change for {path}: {report:#}"))["classes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c.as_str().unwrap().to_string())
        .collect()
}

/// §3.1 and D-1 end to end. The working tree is checked out at a head that
/// widens the bypass floor over its own diff, so a verb reading the working
/// tree's configuration would see the candidate's rules; `delta` reads the
/// merge base's, from the exported base tree.
#[test]
fn delta_classifies_under_the_merge_base_not_the_checked_out_head() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("repo");
    fs::create_dir_all(&root).unwrap();
    let exports = tmp.path().join("tmp");
    fs::create_dir_all(&exports).unwrap();
    delta_repo(&root);

    git088(&root, &["switch", "-qc", "policy"]);
    fs::write(
        root.join("spec-spine.toml"),
        "[coupling]\nbypass_prefixes = [\"src/\", \"tools/\"]\n",
    )
    .unwrap();
    fs::write(root.join("src/a.rs"), "pub fn a() { unreviewed(); }\n").unwrap();
    fs::write(root.join("tools/x.sh"), "echo unreviewed\n").unwrap();
    // A binary file: `git diff -U0` prints no `+++` header for it, so a
    // unified-diff parser would drop it from the change (D-9).
    fs::write(root.join("src/blob.bin"), [0u8, 159, 146, 150, 0]).unwrap();
    git088(&root, &["add", "-A"]);
    git088(&root, &["commit", "-q", "-m", "policy"]);

    let out = delta_json_in(&root, "HEAD", &exports);
    assert_eq!(code(&out), 0, "{}", String::from_utf8_lossy(&out.stderr));
    let v = envelope(&out);
    assert_eq!(v["verb"], "delta");
    assert_eq!(v["schemaVersion"], spec_spine_types::VERDICT_SCHEMA_VERSION);
    assert_eq!(v["ok"], true);
    let report = &v["report"];
    assert_eq!(
        report["schemaVersion"],
        spec_spine_types::DELTA_SCHEMA_VERSION
    );
    assert_eq!(report["classifiedUnder"], "base");

    assert_eq!(change_classes(report, "spec-spine.toml"), ["policy"]);
    assert_eq!(
        change_classes(report, "src/a.rs"),
        ["implementation"],
        "the candidate's own bypass prefix does not apply to its own diff"
    );
    assert_eq!(
        change_classes(report, "tools/x.sh"),
        ["unowned"],
        "under the candidate's configuration this path would read bypassed"
    );
    assert_eq!(change_classes(report, "src/blob.bin"), ["implementation"]);
    assert_eq!(report["priorPolicy"]["required"], true);

    // The three commits are the repository's, not invented.
    let rev = |r: &str| {
        let o = Command::new("git")
            .arg("-C")
            .arg(&root)
            .args(["rev-parse", r])
            .output()
            .unwrap();
        String::from_utf8_lossy(&o.stdout).trim().to_string()
    };
    assert_eq!(report["base"], rev("main"));
    assert_eq!(report["mergeBase"], rev("main"));
    assert_eq!(report["head"], rev("HEAD"));

    // Nothing the verb did is left behind: not in the repository's index or
    // working tree, and not in the temporary directory it exported into.
    let status = Command::new("git")
        .arg("-C")
        .arg(&root)
        .args(["status", "--porcelain"])
        .output()
        .unwrap();
    assert!(
        status.stdout.is_empty(),
        "{}",
        String::from_utf8_lossy(&status.stdout)
    );
    assert_eq!(
        fs::read_dir(&exports).unwrap().count(),
        0,
        "the exported trees are removed"
    );
}

/// §3.1 and §3.5: an unchanged range is a report, exit 0, and the prose says
/// what `required: false` does not mean.
#[test]
fn delta_prose_says_what_not_required_does_not_mean() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    delta_repo(root);
    let out = run_in(root, &["delta", "--base", "main", "--head", "main"]);
    assert_eq!(code(&out), 0);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("0 path(s) changed"), "{stdout}");
    assert!(
        stdout.contains("does not mean the change is safe, correct or approved"),
        "{stdout}"
    );
}

/// §3.1: git trouble is exit 3, as an envelope under `--json`; a stale merge
/// base index is exit 2, the code this tool spends on staleness (D-6).
#[test]
fn delta_failures_keep_the_exit_code_contract() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("repo");
    fs::create_dir_all(&root).unwrap();
    delta_repo(&root);

    let out = delta_json_in(&root, "no-such-ref", tmp.path());
    assert_eq!(code(&out), 3);
    let v = envelope(&out);
    assert_eq!(v["verb"], "delta");
    assert_eq!(v["error"]["kind"], "io");

    // A base whose committed index no longer matches its tree.
    git088(&root, &["switch", "-qc", "stale-base"]);
    let spec = fs::read_to_string(root.join("specs/001-a/spec.md")).unwrap();
    fs::write(
        root.join("specs/001-a/spec.md"),
        spec.replace("# a\n", "# a, edited\n"),
    )
    .unwrap();
    git088(&root, &["commit", "-qam", "stale"]);
    let out = bin()
        .arg("--repo")
        .arg(&root)
        .args(["delta", "--base", "stale-base", "--head", "stale-base"])
        .output()
        .unwrap();
    assert_eq!(code(&out), 2, "{}", String::from_utf8_lossy(&out.stderr));
}

// ── spec 076: a stray shard is orphaned at the verbs ─────────────────────

/// `verdict_fixture`'s index shard directory, `by-spec` or `by-package`.
fn index_shard_dir(root: &Path, which: &str) -> std::path::PathBuf {
    root.join(".derived/codebase-index").join(which)
}

/// The three judging reads of §3.6, each asserted to exit 2 with `named` on a
/// drift line of stderr, and the `check --json` index payload returned.
fn assert_judged_stale(root: &Path, named: &str) -> serde_json::Value {
    for args in [&["index", "check"][..], &["check"][..]] {
        let out = run_in(root, args);
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert_eq!(
            code(&out),
            2,
            "{args:?} must judge, not fail to read: {stderr}"
        );
        assert!(stderr.contains(named), "{args:?} names {named}: {stderr}");
    }
    let out = run_in(root, &["check", "--json"]);
    assert_eq!(code(&out), 2, "{}", String::from_utf8_lossy(&out.stdout));
    let v = envelope(&out);
    assert_eq!(v["exitCode"], 2, "{v}");
    assert_eq!(v["report"]["index"]["fresh"], false, "{v}");
    v["report"]["index"].clone()
}

/// §3.6 case 1: an unexpected stray that does not parse is `orphaned` at exit 2,
/// named as unreadable, and counted as skipped. Before 095 both verbs exited 3
/// with a parse error, having already computed the verdict.
#[test]
fn an_unparseable_stray_is_orphaned_at_the_verbs() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    verdict_fixture(root);
    fs::write(
        index_shard_dir(root, "by-spec").join("999-stray.json"),
        "{\"nope\": 1}\n",
    )
    .unwrap();

    let index = assert_judged_stale(root, "orphaned by-spec/999-stray.json");
    assert_eq!(index["skippedShards"], 1, "{index}");
    let stderr = String::from_utf8_lossy(&run_in(root, &["check"]).stderr).into_owned();
    assert!(
        stderr.contains("orphaned by-spec/999-stray.json (unreadable"),
        "the prose names the skipped file on its drift line: {stderr}"
    );
    // The facade half answers the same payload (§3.4, spec 050 §3.3's pairing).
    let facade: serde_json::Value = serde_json::from_str(
        &spec_spine_core::check_freshness_json("{}", root.to_str().unwrap()).unwrap(),
    )
    .unwrap();
    let cli = envelope(&run_in(root, &["index", "check", "--json"]));
    assert_eq!(cli["report"], facade, "index check and its facade agree");
    assert_eq!(facade["skippedShards"], 1, "{facade}");
}

/// §3.6 case 2: a stray that parses is `orphaned` at exit 2, unchanged, and
/// nothing is skipped because the tally could read it.
#[test]
fn a_parseable_stray_is_orphaned_and_nothing_is_skipped() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    verdict_fixture(root);
    let dir = index_shard_dir(root, "by-spec");
    fs::copy(dir.join("001-a.json"), dir.join("999-copy.json")).unwrap();

    let index = assert_judged_stale(root, "orphaned by-spec/999-copy.json");
    assert!(index.get("skippedShards").is_none(), "{index}");
}

/// §3.6 case 3: the same in the other tree.
#[test]
fn an_unparseable_stray_in_by_package_is_orphaned_at_the_verbs() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    verdict_fixture(root);
    let dir = index_shard_dir(root, "by-package");
    assert!(
        fs::read_dir(&dir).unwrap().count() > 0,
        "the fixture has a package shard, so by-package is a real tree"
    );
    fs::write(dir.join("zzz-stray.json"), "[]\n").unwrap();

    let index = assert_judged_stale(root, "orphaned by-package/zzz-stray.json");
    assert_eq!(index["skippedShards"], 1, "{index}");
}

/// §3.6 case 4: an **expected** shard corrupted in place is stale by 086's byte
/// comparison, named `modified` rather than `orphaned`. A fix that caught the
/// parse error only where strays are enumerated would leave exit 3 here.
#[test]
fn an_expected_shard_corrupted_in_place_is_modified_at_the_verbs() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    verdict_fixture(root);
    fs::write(
        index_shard_dir(root, "by-spec").join("001-a.json"),
        "{ truncated by a bad merge",
    )
    .unwrap();

    let index = assert_judged_stale(root, "modified by-spec/001-a.json");
    assert_eq!(index["skippedShards"], 1, "{index}");
    let stderr = String::from_utf8_lossy(&run_in(root, &["index", "check"]).stderr).into_owned();
    assert!(
        !stderr.contains("orphaned"),
        "an expected path is not an orphan: {stderr}"
    );
}

/// §3.6 case 5: a fresh tree exits 0, skips nothing, and its payload carries no
/// new member, so it is the pre-095 payload byte for byte (§3.3's omission).
#[test]
fn a_fresh_tree_payload_gains_no_member() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    verdict_fixture(root);

    for args in [&["index", "check", "--json"][..], &["check", "--json"][..]] {
        let out = run_in(root, args);
        assert_eq!(
            code(&out),
            0,
            "{args:?}: {}",
            String::from_utf8_lossy(&out.stdout)
        );
    }
    let index = envelope(&run_in(root, &["index", "check", "--json"]))["report"].clone();
    let keys: Vec<&str> = index
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(keys, vec!["diagnostics", "fresh", "unwitnessed"], "{index}");
    let composed = envelope(&run_in(root, &["check", "--json"]))["report"]["index"].clone();
    assert_eq!(composed, index, "both verbs carry the one index shape");
    let stdout = String::from_utf8_lossy(&run_in(root, &["check", "--json"]).stdout).into_owned();
    assert!(!stdout.contains("skippedShards"), "{stdout}");
}

/// §3.6 case 6: the consumer half is not loosened. On case 1's tree the reads
/// that consume the ledger still refuse to proceed: `index owner` at its
/// freshness guard (exit 2, as before this spec), and `index render`, which
/// reads the shards with no guard in front, still at exit 3 (095 D-5).
#[test]
fn the_consumer_verbs_still_refuse_an_unparseable_stray() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    verdict_fixture(root);
    fs::write(
        index_shard_dir(root, "by-spec").join("999-stray.json"),
        "{\"nope\": 1}\n",
    )
    .unwrap();

    let owner = run_in(root, &["index", "owner", "crate-a/src/lib.rs"]);
    assert_eq!(
        code(&owner),
        2,
        "{}",
        String::from_utf8_lossy(&owner.stderr)
    );
    let render = run_in(root, &["index", "render"]);
    assert_eq!(
        code(&render),
        3,
        "{}",
        String::from_utf8_lossy(&render.stderr)
    );
    assert!(
        String::from_utf8_lossy(&render.stderr).contains("999-stray.json"),
        "{}",
        String::from_utf8_lossy(&render.stderr)
    );
}

// ── spec 094: a governed read names its version ──────────────────────────

/// `document` parsed as an object whose top-level keys are sorted, returned for
/// further assertions. Key order is read from the bytes, since a parsed map
/// sorts regardless.
fn sorted_object(label: &str, out: &std::process::Output) -> serde_json::Value {
    assert_eq!(
        code(out),
        0,
        "{label}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    let v: serde_json::Value =
        serde_json::from_str(&text).unwrap_or_else(|e| panic!("{label} is JSON ({e}): {text}"));
    let obj = v
        .as_object()
        .unwrap_or_else(|| panic!("{label} is an object: {text}"));
    // Top-level members are the lines indented by exactly two spaces.
    let order: Vec<String> = text
        .lines()
        .filter(|l| l.starts_with("  \"") && !l.starts_with("   "))
        .map(|l| l[3..l[3..].find('"').unwrap() + 3].to_string())
        .collect();
    assert_eq!(order.len(), obj.len(), "{label}: {text}");
    let mut sorted = order.clone();
    sorted.sort();
    assert_eq!(order, sorted, "{label}: top-level keys in sorted order");
    v
}

/// §3.8: each of the thirteen read documents is an object, carries its version
/// member, and emits its top-level keys sorted. Per document, because the
/// defect was never in a shared function: it was call sites that did not use
/// one, and a projection flag is a call site.
#[test]
fn every_read_document_is_a_sorted_versioned_object() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    verdict_fixture(root);
    let version = spec_spine_types::READ_SCHEMA_VERSION;

    let stamped: [(&str, &[&str]); 12] = [
        ("registry list", &["registry", "list", "--json"]),
        (
            "registry list --ids-only",
            &["registry", "list", "--ids-only", "--json"],
        ),
        ("registry show", &["registry", "show", "001-a", "--json"]),
        (
            "registry status-report",
            &["registry", "status-report", "--json"],
        ),
        (
            "registry status-report --nonzero-only",
            &["registry", "status-report", "--nonzero-only", "--json"],
        ),
        (
            "registry relationships",
            &["registry", "relationships", "001-a", "--json"],
        ),
        ("registry plan", &["registry", "plan", "--json"]),
        (
            "registry plan --next",
            &["registry", "plan", "--next", "--json"],
        ),
        (
            "index owner",
            &["index", "owner", "crate-a/src/lib.rs", "--json"],
        ),
        ("index coverage", &["index", "coverage", "--json"]),
        ("index diagnostics", &["index", "diagnostics", "--json"]),
        ("index orphans", &["index", "orphans", "--json"]),
    ];
    for (label, args) in stamped {
        let v = sorted_object(label, &run_in(root, args));
        assert_eq!(v["schemaVersion"], version, "{label}: {v}");
        assert!(
            v.get("config_version").is_none(),
            "{label}: one version member"
        );
    }

    // §3.7: `config show` is sorted, keeps 054's member, and gains no second one.
    let v = sorted_object("config show", &run_in(root, &["config", "show", "--json"]));
    assert!(v.get("config_version").is_some(), "{v}");
    assert!(v.get("schemaVersion").is_none(), "{v}");

    // §3.6: the three arrays sit under `items`, and `--ids-only` still projects
    // ids rather than records.
    for args in [
        &["registry", "list", "--json"][..],
        &["registry", "list", "--ids-only", "--json"][..],
        &["index", "diagnostics", "--json"][..],
    ] {
        assert!(
            envelope(&run_in(root, args))["items"].is_array(),
            "{args:?}"
        );
    }
    let ids = envelope(&run_in(root, &["registry", "list", "--ids-only", "--json"]));
    assert_eq!(ids["items"], serde_json::json!(["001-a"]), "{ids}");
    let records = envelope(&run_in(root, &["registry", "list", "--json"]));
    assert_eq!(records["items"][0]["id"], "001-a", "{records}");

    // §3.3: the facades emit the documents the CLI does.
    let registry_text = {
        let cfg = spec_spine_types::Config::default();
        spec_spine_core::compile(&cfg, root).unwrap().json
    };
    let query = |op: &str, extra: serde_json::Value| {
        let mut req = serde_json::json!({ "registry": registry_text, "op": op });
        req.as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        serde_json::from_str::<serde_json::Value>(
            &spec_spine_core::query_json(&req.to_string()).unwrap(),
        )
        .unwrap()
    };
    assert_eq!(
        query("list", serde_json::json!({ "idsOnly": true })),
        ids,
        "query_json list --ids-only"
    );
    assert_eq!(
        query("plan", serde_json::json!({})),
        envelope(&run_in(root, &["registry", "plan", "--json"]))
    );
    let coverage: serde_json::Value = serde_json::from_str(
        &spec_spine_core::coverage_json("{}", root.to_str().unwrap()).unwrap(),
    )
    .unwrap();
    assert_eq!(
        coverage,
        envelope(&run_in(root, &["index", "coverage", "--json"]))
    );
}

/// §3.8: `plan --next --json` on an empty ready set is `next: null` at exit 0,
/// the same shape as the populated answer, never a bare `null` and never a
/// missing member.
#[test]
fn plan_next_on_an_empty_ready_set_is_a_present_null() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("specs/001-done");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("spec.md"),
        "---\nid: \"001-done\"\ntitle: \"T\"\nstatus: approved\ncreated: \"2026-09-06\"\n\
         summary: \"s\"\nimplementation: complete\n---\n# 001-done\n",
    )
    .unwrap();
    assert_eq!(code(&run_in(tmp.path(), &["compile"])), 0);

    let out = run_in(tmp.path(), &["registry", "plan", "--next", "--json"]);
    let v = sorted_object("plan --next (empty)", &out);
    assert_eq!(
        v,
        serde_json::json!({
            "next": null,
            "schemaVersion": spec_spine_types::READ_SCHEMA_VERSION,
        })
    );
}

// ── spec 070: the authority snapshot ─────────────────────────────────────

/// §3.5: `attest --snapshot` writes `snapshot.json`, reports it in the same
/// `{ attestation, attestationHash }` envelope as the other scopes (equal to the
/// facade's payload), seals it beside itself, and `verify-attestation
/// --snapshot` checks both modes back, naming a moved member on recompute.
#[test]
fn attest_snapshot_writes_seals_and_verifies() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    verdict_fixture(root);

    let out = run_in(root, &["attest", "--snapshot", "--json"]);
    assert_eq!(code(&out), 0, "{}", String::from_utf8_lossy(&out.stderr));
    let v = envelope(&out);
    assert_eq!(v["verb"], "attest");
    let path = root.join(".derived/attestation/snapshot.json");
    assert!(path.is_file(), "the payload lands at snapshot.json");
    assert!(!root.join(".derived/attestation/attestation.json").exists());
    let stored: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(v["report"]["attestation"], stored);
    assert_eq!(
        stored["schemaVersion"],
        spec_spine_types::SNAPSHOT_SCHEMA_VERSION
    );
    assert_eq!(stored["digest"], "frame/1");
    assert_eq!(stored["committed"]["registry"]["matchesRecompute"], true);
    assert_eq!(stored["committed"]["index"]["matchesRecompute"], true);
    assert_eq!(stored["specs"][0]["id"], "001-a");
    assert!(stored["specs"][0]["territoryDigest"].is_string());
    assert!(stored["specs"][0]["specAttestationHash"].is_string());

    let facade: serde_json::Value = serde_json::from_str(
        &spec_spine_core::attest_snapshot_json("{}", root.to_str().unwrap()).unwrap(),
    )
    .unwrap();
    assert_eq!(v["report"], facade, "one payload shape per verb");

    let seed = root.join("signing.key");
    fs::write(&seed, [5u8; 32]).unwrap();
    let signed = run_in(
        root,
        &[
            "attest",
            "--snapshot",
            "--sign",
            "--key",
            seed.to_str().unwrap(),
        ],
    );
    assert_eq!(
        code(&signed),
        0,
        "{}",
        String::from_utf8_lossy(&signed.stderr)
    );
    let seal_path = root.join(".derived/attestation/snapshot.sig");
    assert!(seal_path.is_file(), "the seal is the payload's sibling");
    let seal: serde_json::Value = serde_json::from_slice(&fs::read(&seal_path).unwrap()).unwrap();
    let public = root.join("public.hex");
    fs::write(&public, seal["keyId"].as_str().unwrap()).unwrap();

    let verified = run_in(
        root,
        &[
            "verify-attestation",
            "--snapshot",
            "--recompute",
            "--signature",
            "--public-key",
            public.to_str().unwrap(),
        ],
    );
    assert_eq!(
        code(&verified),
        0,
        "{}",
        String::from_utf8_lossy(&verified.stderr)
    );
    let text = String::from_utf8_lossy(&verified.stdout);
    assert!(text.contains("recompute: MATCH"), "{text}");
    assert!(text.contains("signature: VALID"), "{text}");

    let request = serde_json::json!({
        "repoRoot": root.to_str().unwrap(),
        "attestationText": fs::read_to_string(&path).unwrap(),
    });
    assert_eq!(
        spec_spine_core::verify_snapshot_attestation_json(&request.to_string()).unwrap(),
        "{\"outcome\":\"match\"}"
    );

    fs::write(
        root.join("crate-a/src/lib.rs"),
        "pub fn a() {}\npub fn b() {}\n",
    )
    .unwrap();
    let stale = run_in(root, &["verify-attestation", "--snapshot", "--recompute"]);
    assert_eq!(code(&stale), 1);
    let err = String::from_utf8_lossy(&stale.stderr);
    assert!(err.contains("CONTENT MISMATCH"), "{err}");
    assert!(
        err.contains("specs[001-a]"),
        "the moved member is named: {err}"
    );
}

/// §3.5: each flag names a different scope, so combining them is refused at
/// exit 3 in words a usage error does not contain.
#[test]
fn attest_snapshot_refuses_another_scope() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    verdict_fixture(root);
    for other in [&["--spec", "001-a"][..], &["--with-coupling"][..]] {
        let mut args = vec!["attest", "--snapshot"];
        args.extend_from_slice(other);
        let out = run_in(root, &args);
        assert_eq!(code(&out), 3, "{args:?}");
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(err.contains("cannot combine"), "{args:?}: {err}");
        assert!(!root.join(".derived/attestation/snapshot.json").exists());
    }
    let out = run_in(
        root,
        &[
            "verify-attestation",
            "--snapshot",
            "--spec",
            "001-a",
            "--recompute",
        ],
    );
    assert_eq!(code(&out), 3);
    assert!(String::from_utf8_lossy(&out.stderr).contains("cannot combine"));
}

/// §3.7: the other two scopes are untouched by the snapshot's existence, and
/// the snapshot's join hash is exactly what `attest --spec` emits.
#[test]
fn the_snapshot_join_hash_is_what_attest_spec_emits() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    verdict_fixture(root);
    let per_spec = envelope(&run_in(root, &["attest", "--spec", "001-a", "--json"]));
    let snap = envelope(&run_in(root, &["attest", "--snapshot", "--json"]));
    assert_eq!(
        snap["report"]["attestation"]["specs"][0]["specAttestationHash"],
        per_spec["report"]["attestationHash"]
    );
}

// ── spec 078: governed scope, enumerated by the CLI ──────────────────────

/// A corpus with a declared scope over `scripts/*`, compiled and indexed, with
/// `scripts/run.sh` present and unclaimed.
fn scope_repo(root: &Path) {
    let w = |rel: &str, content: &str| {
        let p = root.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, content).unwrap();
    };
    w("Cargo.toml", "[workspace]\nmembers = [\"crate-a\"]\n");
    w(
        "crate-a/Cargo.toml",
        "[package]\nname = \"crate-a\"\nversion = \"0.1.0\"\n",
    );
    w("crate-a/src/lib.rs", "pub fn a() {}\n");
    w("scripts/run.sh", "echo hi\n");
    w(
        "specs/001-a/spec.md",
        "---\nid: \"001-a\"\ntitle: \"A\"\nstatus: approved\ncreated: \"2026-09-15\"\n\
         summary: \"s\"\nestablishes:\n  - \"crate-a/src/lib.rs\"\n---\n# a\n",
    );
    w(
        "spec-spine.toml",
        "[coverage]\ngoverned_scope = [\"scripts/*\"]\n",
    );
    for verb in ["compile", "index"] {
        let out = run_in(root, &[verb]);
        assert_eq!(
            code(&out),
            0,
            "{verb}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

fn git097(root: &Path, args: &[&str]) {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["-c", "commit.gpgsign=false"])
        .args(args)
        .env("GIT_AUTHOR_NAME", "t")
        .env("GIT_AUTHOR_EMAIL", "t@t")
        .env("GIT_COMMITTER_NAME", "t")
        .env("GIT_COMMITTER_EMAIL", "t@t")
        .output()
        .unwrap();
    assert!(out.status.success(), "git {args:?}: {out:?}");
}

fn coverage_doc(root: &Path, extra: &[&str]) -> serde_json::Value {
    let mut args = vec!["index", "coverage", "--json"];
    args.extend_from_slice(extra);
    let out = run_in(root, &args);
    assert_eq!(code(&out), 0, "{}", String::from_utf8_lossy(&out.stderr));
    envelope(&out)
}

/// §3.6: `--paths-from` is the inventory, named `supplied`, and the facade
/// given the same inventory answers the same report.
#[test]
fn coverage_paths_from_is_the_supplied_inventory_and_matches_the_facade() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    scope_repo(root);
    let list = root.join("inventory.txt");
    fs::write(&list, "scripts/run.sh\n").unwrap();

    let doc = coverage_doc(root, &["--paths-from", list.to_str().unwrap()]);
    assert_eq!(doc["enumeration"], "supplied", "{doc}");
    assert_eq!(
        doc["declaredScopeFiles"],
        serde_json::json!(["scripts/run.sh"])
    );
    assert_eq!(doc["unclaimedFiles"], serde_json::json!(["scripts/run.sh"]));

    let request = serde_json::json!({
        "repoRoot": root.to_str().unwrap(),
        "config": { "coverage": { "governed_scope": ["scripts/*"] } },
        "inventory": { "provenance": "supplied", "paths": ["scripts/run.sh"] },
    });
    let facade: serde_json::Value = serde_json::from_str(
        &spec_spine_core::coverage_inventory_json(&request.to_string())
            .unwrap_or_else(|e| panic!("facade: {e}")),
    )
    .unwrap();
    assert_eq!(facade, doc, "the facade and the CLI answer one report");

    // An empty list is an answer, not a request to enumerate.
    fs::write(&list, "").unwrap();
    let empty = coverage_doc(root, &["--paths-from", list.to_str().unwrap()]);
    assert_eq!(empty["declaredScopeFiles"], serde_json::json!([]));
    assert_eq!(empty["enumeration"], "supplied");
}

/// §3.6: with a scope set and no `--paths-from`, a git failure is exit 3 naming
/// the remedy, never a silent walk.
#[test]
fn coverage_git_failure_is_exit_3_not_a_walk() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    scope_repo(root);
    let out = run_in(root, &["index", "coverage"]);
    assert_eq!(code(&out), 3, "{}", String::from_utf8_lossy(&out.stdout));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("--paths-from"), "{err}");
    assert!(!String::from_utf8_lossy(&out.stdout).contains("declared scope"));
}

/// §3.6: the git enumeration's four membership cases. A tracked file an ignore
/// rule matches is retained; an untracked ignored file is excluded; an unstaged
/// untracked addition is included; a tracked file missing from the tree is
/// dropped.
#[test]
fn the_git_inventory_keeps_tracked_and_new_files_and_drops_ignored_and_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    scope_repo(root);
    let w = |rel: &str| fs::write(root.join(rel), "echo x\n").unwrap();
    w("scripts/tracked_ignored.sh");
    w("scripts/gone.sh");
    git097(root, &["init", "-q"]);
    git097(root, &["add", "-A"]);
    git097(root, &["commit", "-q", "-m", "base"]);
    // Ignore rules added after tracking: the tracked file stays tracked.
    fs::write(root.join(".gitignore"), "scripts/*ignored*.sh\n").unwrap();
    w("scripts/untracked_ignored.sh");
    w("scripts/new.sh");
    fs::remove_file(root.join("scripts/gone.sh")).unwrap();

    let doc = coverage_doc(root, &[]);
    assert_eq!(doc["enumeration"], "tracked", "{doc}");
    assert_eq!(
        doc["declaredScopeFiles"],
        serde_json::json!([
            "scripts/new.sh",
            "scripts/run.sh",
            "scripts/tracked_ignored.sh"
        ]),
        "{doc}"
    );
}

/// §3.1, §3.7: `config show` prints both keys, and a scaffolded
/// `spec-spine.toml` carries them as a commented default.
///
/// The scaffold half used to run `spec-spine init` and read the file off disk.
/// Spec 092 §3.1 removed that verb; the producer it called is still exported,
/// so the half is asserted through the library instead of dropped. What is
/// being tested here is the CONTENT of the scaffolded configuration, which is
/// the same fact either way.
#[test]
fn config_show_and_the_scaffold_carry_the_scope_keys() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let show = run_in(root, &["config", "show"]);
    assert_eq!(code(&show), 0, "{}", String::from_utf8_lossy(&show.stderr));
    let text = String::from_utf8_lossy(&show.stdout);
    assert!(text.contains("[coverage]"), "{text}");
    assert!(text.contains("governed_scope = []"), "{text}");
    assert!(text.contains("governed_scope_exclusions = []"), "{text}");
    let json = envelope(&run_in(root, &["config", "show", "--json"]));
    assert_eq!(
        json["coverage"]["governed_scope"],
        serde_json::json!([]),
        "{json}"
    );

    let scaffold = spec_spine_core::scaffold_init(&spec_spine_types::Config::default()).unwrap();
    let toml = scaffold
        .files
        .iter()
        .find(|f| f.rel_path == "spec-spine.toml")
        .expect("the scaffold produces the config")
        .contents
        .clone();
    assert!(toml.contains("[coverage]"), "{toml}");
    assert!(toml.contains("# governed_scope = ["), "{toml}");
    assert!(toml.contains("dir/**/*"), "the glob trap is named: {toml}");
    // The scaffold still parses, with the scope empty.
    let cfg = spec_spine_types::load_config(&toml).unwrap();
    assert!(cfg.coverage.governed_scope.is_empty());
}

// --- spec 079: a blocking claim is not a stale shard, at the verbs -----------

/// A corpus whose committed shards are byte-exact and whose spec claims a unit
/// that does not exist.
///
/// `status` and `implementation` decide the tier: spec 038's table makes
/// `approved` + `complete` blocking, and anything in flight a `W-001` warning
/// (spec 079 D-5).
fn blocking_corpus(root: &Path, status: &str, implementation: &str) {
    let dir = root.join("specs/001-missing");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("spec.md"),
        format!(
            "---\nid: \"001-missing\"\ntitle: \"T\"\nstatus: {status}\ncreated: \"2026-09-16\"\n\
             implementation: {implementation}\nsummary: \"s\"\nestablishes:\n  - \"src/gone.rs\"\n\
             ---\n\n# 001-missing\n"
        ),
    )
    .unwrap();
    assert_eq!(code(&run_in(root, &["compile"])), 0);
    assert_eq!(code(&run_in(root, &["index"])), 0);
}

fn stderr(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

#[test]
fn check_reports_an_unresolved_claim_as_itself() {
    // Spec 079 AC-1 / FR-003 / FR-004 / FR-005. The refusal and the exit code
    // are unchanged (§3.1); what changes is that the verb no longer calls a
    // byte-exact tree stale and no longer prescribes a command that provably
    // does not work.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    blocking_corpus(root, "approved", "complete");

    let out = run_in(root, &["check"]);
    // Spec 080 §3.1 moved this from 2 to 1: an unresolved claim is a validation
    // failure, not staleness. Spec 079 held the code still deliberately and
    // filed the question forward as design note 05 R-4.
    assert_eq!(code(&out), 1, "{}", stderr(&out));
    let err = stderr(&out);
    let index_line = err
        .lines()
        .find(|l| l.starts_with("codebase-index:"))
        .expect("the index half is attributed to its tree (spec 062 §3.4)");
    assert!(!index_line.contains("STALE"), "{index_line}");
    assert!(!index_line.contains("spec-spine index"), "{index_line}");
    assert!(err.contains("I-004"), "{err}");
    assert!(err.contains("001-missing"), "{err}");
    assert!(err.contains("src/gone.rs"), "{err}");
    assert!(
        err.contains("regenerating the index does not clear this"),
        "{err}"
    );
    // FR-007 / AC-6: the contradiction is named.
    assert!(err.contains("`implementation: complete`"), "{err}");
    // AC-7: and no way out of it is offered.
    assert!(!err.contains("planned: true"), "{err}");
    // The registry half is untouched.
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("spec-registry: fresh"),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );

    // The same at the primitive verb, in its own words and with the same code
    // (spec 080 §3.2: a caller must not have to know which verb it invoked to
    // know what a code means).
    let idx = run_in(root, &["index", "check"]);
    assert_eq!(code(&idx), 1);
    let ierr = stderr(&idx);
    assert!(ierr.contains("I-004"), "{ierr}");
    assert!(!ierr.contains("to refresh"), "{ierr}");
}

#[test]
fn regenerating_leaves_the_message_accurate() {
    // Spec 079 AC-2, the regression. `index` exits 0 and repairs nothing, and
    // the verb still refuses without claiming the tree is stale or that
    // anything was fixed. This is the case the pre-098 output got wrong, so it
    // is asserted directly rather than inferred from AC-1.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    blocking_corpus(root, "approved", "complete");

    assert_eq!(
        code(&run_in(root, &["index"])),
        0,
        "the named remedy exits 0"
    );
    let out = run_in(root, &["check"]);
    assert_eq!(code(&out), 1, "and the refusal stands (spec 080 §3.1)");
    let err = stderr(&out);
    assert!(err.contains("I-004"), "{err}");
    assert!(
        !err.lines()
            .any(|l| l.starts_with("codebase-index:") && l.contains("STALE")),
        "{err}"
    );
}

#[test]
fn check_json_is_unchanged_by_the_message_fix() {
    // Spec 079 FR-009 / AC-8 / D-1: the `--json` envelope keeps its version,
    // members and nesting. A JSON consumer could already separate the two
    // refusals through `report.index.diagnostics.byCode`, so the surface that
    // needed correcting was the text and this one is held still.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    blocking_corpus(root, "approved", "complete");

    let out = run_in(root, &["check", "--json"]);
    // Spec 080 §3.3: `exitCode` carries the new code, which is the point. Every
    // other member, the nesting and the version are what spec 079 left them.
    assert_eq!(code(&out), 1);
    let json = envelope(&out);
    assert_eq!(json["schemaVersion"], "0.4.0", "{json}");
    assert_eq!(json["exitCode"], 1, "{json}");
    assert_eq!(json["ok"], false, "{json}");
    let members: Vec<&str> = json
        .as_object()
        .unwrap()
        .keys()
        .map(|k| k.as_str())
        .collect();
    assert_eq!(
        members,
        ["exitCode", "ok", "report", "schemaVersion", "verb"],
        "{json}"
    );
    let index = &json["report"]["index"];
    let index_members: Vec<&str> = index
        .as_object()
        .unwrap()
        .keys()
        .map(|k| k.as_str())
        .collect();
    assert_eq!(
        index_members,
        ["actual", "diagnostics", "expected", "fresh", "unwitnessed"],
        "{json}"
    );
    assert_eq!(index["diagnostics"]["byCode"]["I-004"], 1, "{json}");
    assert_eq!(
        index["actual"], "1 stale shard(s):\n  blocking-diagnostics by-spec/001-missing.json",
        "the payload text is the one this spec deliberately does not move: {json}"
    );
}

#[test]
fn a_stale_shard_still_reads_as_staleness() {
    // Spec 079 FR-008 / AC-3: unchanged, wording included, for a corpus with no
    // blocking diagnostic. A caller that reads staleness today reads it after.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write_spec(root, "001-a", "001-a", "approved");
    assert_eq!(code(&run_in(root, &["index"])), 0);
    write_spec(root, "001-a", "001-a", "draft"); // a hashed input moves

    let out = run_in(root, &["check"]);
    assert_eq!(code(&out), 2);
    let err = stderr(&out);
    assert!(
        err.contains("codebase-index: STALE (run `spec-spine index`)"),
        "{err}"
    );
    assert!(err.contains("stale shard(s):"), "{err}");
    assert!(!err.contains("I-004"), "{err}");
    assert!(!err.contains("UNRESOLVED CLAIM"), "{err}");

    let idx = run_in(root, &["index", "check"]);
    assert_eq!(code(&idx), 2);
    assert!(
        stderr(&idx).contains("index is STALE (run `spec-spine index` to refresh)"),
        "{}",
        stderr(&idx)
    );
}

#[test]
fn a_mixed_tree_names_both_halves_at_the_verbs() {
    // Spec 079 AC-4 / FR-006: neither half elided, and regeneration attributed
    // to the stale half alone.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write_spec(root, "002-b", "002-b", "approved");
    blocking_corpus(root, "approved", "complete");
    write_spec(root, "002-b", "002-b", "draft"); // 002-b's shard is now behind

    let out = run_in(root, &["check"]);
    // Spec 080 §3.1: a tree holding both refusals exits 1, under spec 062
    // §3.3's order. Both halves are still named below, which is spec 079's
    // requirement and is what this test is really about.
    assert_eq!(code(&out), 1);
    let err = stderr(&out);
    assert!(
        err.contains("codebase-index: STALE (run `spec-spine index`)"),
        "{err}"
    );
    assert!(err.contains("modified by-spec/002-b.json"), "{err}");
    assert!(err.contains("I-004"), "{err}");
    assert!(
        err.contains("regenerating addresses the stale shard(s) only, not the unresolved claim(s)"),
        "{err}"
    );

    // After regenerating, the stale half is gone and the blocking half stands.
    assert_eq!(code(&run_in(root, &["index"])), 0);
    let after = stderr(&run_in(root, &["check"]));
    assert!(after.contains("I-004"), "{after}");
    assert!(
        !after
            .lines()
            .any(|l| l.starts_with("codebase-index:") && l.contains("STALE")),
        "{after}"
    );
}

#[test]
fn a_spec_that_claims_no_completion_is_not_accused_of_one() {
    // Spec 079 AC-6 / D-5. `approved` + `deferred` is not in flight (spec 038's
    // table), so it blocks with the same code and declares no completion; the
    // in-flight pairing blocks nothing at all, which is why it cannot be the
    // negative.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    blocking_corpus(root, "approved", "deferred");
    let err = stderr(&run_in(root, &["check"]));
    assert!(err.contains("I-004"), "{err}");
    assert!(!err.contains("complete"), "{err}");

    let tmp2 = tempfile::tempdir().unwrap();
    let root2 = tmp2.path();
    blocking_corpus(root2, "draft", "in-progress");
    let out = run_in(root2, &["check"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(!stderr(&out).contains("I-004"), "{}", stderr(&out));
}

// ---------------------------------------------------------------------------
// Spec 080: an unresolved claim exits as a validation failure.
//
// The rule these pin is spec 069 §3.1's closing sentence as spec 080 §3.1
// amends it: drift alone exits 2, a blocking diagnostic exits 1, and a tree
// holding both exits 1 under spec 062 §3.3's order. Every message is spec
// 098's and is asserted unchanged, because the value of this change is that it
// moves one code and nothing else.
// ---------------------------------------------------------------------------

/// Spec 080 §3.1, §3.4: `check` spends the validation code on a blocking claim.
#[test]
fn spec101_check_exits_1_on_an_unresolved_claim() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    blocking_corpus(root, "approved", "complete");

    let out = run_in(root, &["check"]);
    assert_eq!(code(&out), 1, "{}", stderr(&out));
    let err = stderr(&out);
    // Spec 080 §3.3: the report is spec 079's, to the word.
    assert!(err.contains("UNRESOLVED CLAIM"), "{err}");
    let index_line = err
        .lines()
        .find(|l| l.starts_with("codebase-index:"))
        .expect("the index half is attributed to its tree");
    assert!(!index_line.contains("STALE"), "{index_line}");
    assert!(!index_line.contains("spec-spine index"), "{index_line}");
}

/// Spec 080 §3.2: the primitive spends the same code on the same fact.
#[test]
fn spec101_index_check_exits_1_on_an_unresolved_claim() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    blocking_corpus(root, "approved", "complete");

    let out = run_in(root, &["index", "check"]);
    assert_eq!(code(&out), 1, "{}", stderr(&out));
    assert!(stderr(&out).contains("I-004"), "{}", stderr(&out));
}

/// Spec 080 §3.1: validation dominates staleness, and both halves survive.
#[test]
fn spec101_a_blocking_claim_and_a_stale_shard_exit_1() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write_spec(root, "002-b", "002-b", "approved");
    blocking_corpus(root, "approved", "complete");
    write_spec(root, "002-b", "002-b", "draft"); // 002-b's shard falls behind

    let out = run_in(root, &["check"]);
    assert_eq!(code(&out), 1, "{}", stderr(&out));
    let err = stderr(&out);
    // Spec 079 §3.3 / FR-006: neither half elided, regeneration attributed to
    // the stale one alone. Moving the code must not cost the report.
    assert!(err.contains("STALE"), "{err}");
    assert!(err.contains("UNRESOLVED CLAIM"), "{err}");
    assert!(
        err.contains("regenerating addresses the stale shard(s) only"),
        "{err}"
    );

    // Spec 080 §3.2 at the primitive, on the SAME corpus: a caller must not
    // have to know which verb it invoked to know what a code means, and the
    // mixed case is the one where the two folds could most easily disagree.
    let idx = run_in(root, &["index", "check"]);
    assert_eq!(code(&idx), 1, "{}", stderr(&idx));
    assert!(stderr(&idx).contains("I-004"), "{}", stderr(&idx));
}

/// Spec 080 §3.3: the regression. Drift alone is still staleness, still 2.
#[test]
fn spec101_a_stale_shard_alone_still_exits_2() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write_spec(root, "001-a", "001-a", "approved");
    assert_eq!(code(&run_in(root, &["index"])), 0);
    write_spec(root, "001-a", "001-a", "draft"); // a hashed input moves

    let out = run_in(root, &["check"]);
    assert_eq!(code(&out), 2, "{}", stderr(&out));
    assert!(
        stderr(&out).contains("codebase-index: STALE (run `spec-spine index`)"),
        "{}",
        stderr(&out)
    );
    assert_eq!(code(&run_in(root, &["index", "check"])), 2);
}

/// Spec 080 §3.3: `--json` carries the new code and nothing else moves.
#[test]
fn spec101_json_carries_the_new_code_and_keeps_its_shape() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    blocking_corpus(root, "approved", "complete");

    let out = run_in(root, &["check", "--json"]);
    assert_eq!(code(&out), 1);
    let json = envelope(&out);
    assert_eq!(json["exitCode"], 1, "{json}");
    assert_eq!(json["ok"], false, "{json}");
    assert_eq!(json["schemaVersion"], "0.4.0", "{json}");
    let members: Vec<&str> = json
        .as_object()
        .unwrap()
        .keys()
        .map(|k| k.as_str())
        .collect();
    assert_eq!(
        members,
        ["exitCode", "ok", "report", "schemaVersion", "verb"],
        "{json}"
    );
    // Spec 080 §4: `actual` stays spec 079 FR-009's deliberate hold, and
    // `byCode` remains the discriminator a JSON consumer already had.
    assert_eq!(
        json["report"]["index"]["diagnostics"]["byCode"]["I-004"], 1,
        "{json}"
    );
    assert_eq!(json["report"]["index"]["fresh"], false, "{json}");
}

/// Spec 080 §3.3: `--fail-on-unresolved` is a different axis and is untouched.
#[test]
fn spec101_the_unresolved_flag_axis_is_unchanged() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    // A draft spec owning a unit that does not resolve is W-001, warning tier
    // (specs 023, 044): work in flight, not a blocking claim.
    blocking_corpus(root, "draft", "in-progress");

    assert_eq!(
        code(&run_in(root, &["check"])),
        0,
        "{}",
        stderr(&run_in(root, &["check"]))
    );
    assert_eq!(code(&run_in(root, &["check", "--fail-on-unresolved"])), 1);
}

// ===== spec 081: the coupling gate can see the change being committed =====

/// The §1.1 scratch repository: `001-a` owns `src/`, committed on `main`, with
/// a second spec `002-b` owning nothing so a later edit to `src/a.rs` has no
/// authoring edit and must refuse.
fn couple102_repo(root: &Path) {
    let w = |rel: &str, content: &[u8]| {
        let p = root.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, content).unwrap();
    };
    w(
        "specs/001-a/spec.md",
        b"---\nid: \"001-a\"\ntitle: \"a\"\nstatus: approved\ncreated: \"2026-09-16\"\nimplementation: complete\nsummary: \"s\"\nestablishes:\n  - \"src/\"\n---\n\n# a\n",
    );
    w("src/a.rs", b"pub fn a() {}\n");
    // A second owner, so a test can dirty a path whose spec is edited nowhere
    // in the union. Without it, an authoring edit in the committed range clears
    // the working-tree change too, which is correct and therefore useless as a
    // negative case.
    w(
        "specs/002-b/spec.md",
        b"---\nid: \"002-b\"\ntitle: \"b\"\nstatus: approved\ncreated: \"2026-09-16\"\nimplementation: complete\nsummary: \"s\"\nestablishes:\n  - \"lib/\"\n---\n\n# b\n",
    );
    w("lib/b.rs", b"pub fn b() {}\n");
    w(".gitignore", b".derived/**/build-meta.json\n");
    for verb in ["compile", "index"] {
        assert_eq!(code(&run_in(root, &[verb])), 0, "fixture {verb}");
    }
    git088(root, &["init", "-q", "-b", "main"]);
    git088(root, &["add", "-A"]);
    git088(root, &["commit", "-q", "-m", "base"]);
}

fn couple102(root: &Path, extra: &[&str]) -> std::process::Output {
    let mut args = vec!["couple", "--base", "main", "--head", "HEAD"];
    args.extend_from_slice(extra);
    run_in(root, &args)
}

/// Spec 081 §1.1, §3.5: a staged edit is invisible to the committed range, and
/// the unflagged verb reports a pass over an empty set.
#[test]
fn spec102_a_staged_edit_is_invisible_without_the_flag() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    couple102_repo(root);
    fs::write(root.join("src/a.rs"), b"pub fn a() { /* drift */ }\n").unwrap();
    git088(root, &["add", "-A"]);

    let bare = couple102(root, &[]);
    assert_eq!(code(&bare), 0, "{}", stderr(&bare));
    assert!(
        String::from_utf8_lossy(&bare.stdout).contains("0 path(s) checked"),
        "the defect: a pass over nothing. {}",
        String::from_utf8_lossy(&bare.stdout)
    );

    let out = couple102(root, &["--include-uncommitted"]);
    assert_eq!(code(&out), 1, "{}", stderr(&out));
    // Violations are written to stderr; the OK line is the stdout half.
    assert!(stderr(&out).contains("src/a.rs"), "{}", stderr(&out));
}

/// Spec 081 §3.1: unstaged changes count too. `git diff HEAD` covers both.
#[test]
fn spec102_an_unstaged_edit_is_judged_under_the_flag() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    couple102_repo(root);
    fs::write(root.join("src/a.rs"), b"pub fn a() { /* drift */ }\n").unwrap();

    assert_eq!(code(&couple102(root, &[])), 0);
    assert_eq!(code(&couple102(root, &["--include-uncommitted"])), 1);
}

/// Spec 081 §3.1, D-2: a path in both views is judged once, not twice.
#[test]
fn spec102_a_path_in_both_views_is_checked_once() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    couple102_repo(root);
    // Committed on a branch off main, then edited again in the working tree.
    git088(root, &["switch", "-q", "-c", "feature"]);
    fs::write(root.join("src/a.rs"), b"pub fn a() { /* one */ }\n").unwrap();
    git088(root, &["add", "-A"]);
    git088(root, &["commit", "-q", "-m", "one"]);
    fs::write(root.join("src/a.rs"), b"pub fn a() { /* one and two */ }\n").unwrap();

    let out = couple102(root, &["--include-uncommitted"]);
    let err = stderr(&out);
    assert_eq!(
        err.matches("C-001 'src/a.rs'").count(),
        1,
        "one violation per path, not one per view: {err}"
    );
}

/// Spec 081 §3.3: the flag is meaningful only against `HEAD`.
#[test]
fn spec102_a_non_head_head_is_refused() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    couple102_repo(root);
    git088(root, &["switch", "-q", "-c", "feature"]);
    fs::write(root.join("src/a.rs"), b"pub fn a() { /* one */ }\n").unwrap();
    git088(root, &["add", "-A"]);
    git088(root, &["commit", "-q", "-m", "one"]);

    // `main` is a real ref and a different commit from HEAD.
    let out = run_in(
        root,
        &[
            "couple",
            "--base",
            "main",
            "--head",
            "main",
            "--include-uncommitted",
        ],
    );
    assert_eq!(code(&out), 3, "{}", stderr(&out));
    assert!(
        stderr(&out).contains("--include-uncommitted"),
        "{}",
        stderr(&out)
    );
}

/// Spec 081 §3.3: `--paths-from` carries no history to union with.
#[test]
fn spec102_paths_from_and_the_flag_are_refused_together() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    couple102_repo(root);
    let list = root.join("paths.txt");
    fs::write(&list, b"src/a.rs\n").unwrap();

    let out = run_in(
        root,
        &[
            "couple",
            "--base",
            "main",
            "--head",
            "HEAD",
            "--paths-from",
            list.to_str().unwrap(),
            "--include-uncommitted",
        ],
    );
    assert_eq!(code(&out), 3, "{}", stderr(&out));
}

/// Spec 081 §3.4: the default does not move. A dirty tree cannot change the
/// verdict of an unflagged run, which is what keeps CI reproducible.
#[test]
fn spec102_the_unflagged_verdict_ignores_the_working_tree() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    couple102_repo(root);
    git088(root, &["switch", "-q", "-c", "feature"]);
    // A committed, properly authored change: 001-a owns src/ and is edited.
    fs::write(root.join("src/a.rs"), b"pub fn a() { /* one */ }\n").unwrap();
    let spec = root.join("specs/001-a/spec.md");
    let body = fs::read_to_string(&spec).unwrap();
    fs::write(&spec, body.replace("# a\n", "# a\n\nAn authoring edit.\n")).unwrap();
    for verb in ["compile", "index"] {
        assert_eq!(code(&run_in(root, &[verb])), 0);
    }
    git088(root, &["add", "-A"]);
    git088(root, &["commit", "-q", "-m", "authored"]);

    let clean = couple102(root, &[]);
    assert_eq!(
        code(&clean),
        0,
        "{}",
        String::from_utf8_lossy(&clean.stdout)
    );

    // Now dirty the tree with an edit whose owning spec (002-b) is edited
    // nowhere in the union. Editing `src/a.rs` again would NOT do: 001-a's
    // authoring edit is in the committed range, so the unioned diff carries it
    // and the gate clears, correctly.
    fs::write(root.join("lib/b.rs"), b"pub fn b() { /* unauthored */ }\n").unwrap();
    let still = couple102(root, &[]);
    assert_eq!(
        code(&still),
        0,
        "a dirty working tree must not change an unflagged verdict: {}",
        String::from_utf8_lossy(&still.stdout)
    );
    let flagged = couple102(root, &["--include-uncommitted"]);
    assert_eq!(code(&flagged), 1, "{}", stderr(&flagged));
    assert!(
        stderr(&flagged).contains("lib/b.rs"),
        "{}",
        stderr(&flagged)
    );
}

// ===== spec 082: an amended acceptance is the one that runs =====

/// A spec document for the 103 fixtures.
fn spec103_doc(id: &str, status: &str, extra: &str, command: &str) -> String {
    format!(
        "---\nid: \"{id}\"\ntitle: \"t\"\nstatus: {status}\ncreated: \"2026-09-16\"\n\
         summary: \"s\"\nimplementation: complete\n{extra}---\n\n# {id}\n\n\
         ## Verification\n\n```verify:cli\n{command}\n```\n"
    )
}

fn spec103_write(root: &Path, id: &str, body: &str) {
    let dir = root.join("specs").join(id);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("spec.md"), body).unwrap();
}

/// Spec 082 §3.1, `V-018`: an entry that is not also in `amends`.
#[test]
fn spec103_amends_verification_outside_amends_is_v018() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    spec103_write(root, "093-a", &spec103_doc("093-a", "approved", "", "a"));
    spec103_write(
        root,
        "103-b",
        // `amends_verification` without the matching `amends`.
        &spec103_doc(
            "103-b",
            "approved",
            "amends_verification: [\"093-a\"]\n",
            "b",
        ),
    );

    let out = run_in(root, &["compile"]);
    assert_eq!(code(&out), 1, "{}", stderr(&out));
    let err = stderr(&out);
    assert!(err.contains("V-018"), "{err}");
    assert!(err.contains("not in amends"), "{err}");
}

/// Spec 082 §3.3, `V-019`: two live specs claiming one acceptance.
#[test]
fn spec103_two_specs_claiming_one_acceptance_is_v019() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    spec103_write(root, "093-a", &spec103_doc("093-a", "approved", "", "a"));
    let extra = "amends: [\"093-a\"]\namends_verification: [\"093-a\"]\n";
    spec103_write(root, "103-b", &spec103_doc("103-b", "approved", extra, "b"));
    spec103_write(root, "104-c", &spec103_doc("104-c", "approved", extra, "c"));

    let out = run_in(root, &["compile"]);
    assert_eq!(code(&out), 1, "{}", stderr(&out));
    assert!(stderr(&out).contains("V-019"), "{}", stderr(&out));

    // D-5: withdrawing one of the two clears the fork rather than needing a
    // tie-break, because a withdrawn spec holds nothing.
    let withdrawn = format!("{extra}retirement_rationale: \"withdrawn\"\n");
    spec103_write(
        root,
        "104-c",
        &spec103_doc("104-c", "retired", &withdrawn, "c"),
    );
    let out = run_in(root, &["compile"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
}

/// Spec 082 §3.3, `V-020`: a cycle resolves to no block.
#[test]
fn spec103_a_cycle_in_the_chain_is_v020() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    spec103_write(
        root,
        "093-a",
        &spec103_doc(
            "093-a",
            "approved",
            "amends: [\"103-b\"]\namends_verification: [\"103-b\"]\n",
            "a",
        ),
    );
    spec103_write(
        root,
        "103-b",
        &spec103_doc(
            "103-b",
            "approved",
            "amends: [\"093-a\"]\namends_verification: [\"093-a\"]\n",
            "b",
        ),
    );

    let out = run_in(root, &["compile"]);
    assert_eq!(code(&out), 1, "{}", stderr(&out));
    let err = stderr(&out);
    assert!(err.contains("V-020"), "{err}");
    // One report per cycle, the convention `detect_dependency_cycle` sets.
    // Every node of a cycle is a start, so an unguarded walk reports a two-node
    // cycle twice.
    assert_eq!(
        err.matches("V-020").count(),
        1,
        "one cycle, one diagnostic: {err}"
    );
    // The refusal is 3.3's, and the message says so.
    assert!(err.contains("spec 082 3.3"), "{err}");
}

/// Spec 082 §3.4: `verify` says whose block it ran, and runs it.
#[test]
fn spec103_verify_states_the_substitution() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    spec103_write(
        root,
        "093-a",
        &spec103_doc("093-a", "approved", "", "false"),
    );
    spec103_write(
        root,
        "103-b",
        &spec103_doc(
            "103-b",
            "approved",
            "amends: [\"093-a\"]\namends_verification: [\"093-a\"]\n",
            "true",
        ),
    );

    let out = run_in(root, &["verify", "093-a"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("acceptance amended by 103-b"),
        "the substitution must never be silent: {stdout}"
    );
    // 093-a's own block is `false`, which would exit 1. The amender's is `true`.
    assert_eq!(code(&out), 0, "{stdout}");
    assert!(stdout.contains("$ true"), "{stdout}");
    assert!(!stdout.contains("$ false"), "{stdout}");

    // The amender under its own name prints no attribution line.
    let own = run_in(root, &["verify", "103-b"]);
    assert!(
        !String::from_utf8_lossy(&own.stdout).contains("acceptance amended by"),
        "{}",
        String::from_utf8_lossy(&own.stdout)
    );
}

/// Spec 082 §3.4: the fact is answerable from the ledger without running
/// anything.
#[test]
fn spec103_registry_show_carries_amends_verification() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    spec103_write(root, "093-a", &spec103_doc("093-a", "approved", "", "a"));
    spec103_write(
        root,
        "103-b",
        &spec103_doc(
            "103-b",
            "approved",
            "amends: [\"093-a\"]\namends_verification: [\"093-a\"]\n",
            "b",
        ),
    );
    assert_eq!(code(&run_in(root, &["compile"])), 0);

    let out = run_in(root, &["registry", "show", "103-b", "--json"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let json: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("registry show emits JSON");
    assert_eq!(
        json["amendsVerification"],
        serde_json::json!(["093-a"]),
        "{json}"
    );
    // `registry show`'s `schemaVersion` is the READ axis (spec 094), not the
    // registry's; the registry MINOR is asserted on the emitted shard instead.
    let shard: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join(".derived/spec-registry/by-spec/103-b.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(shard["specVersion"], "1.3.0", "{shard}");
    assert_eq!(
        shard["record"]["amendsVerification"],
        serde_json::json!(["093-a"]),
        "{shard}"
    );
}

// ── spec 092 §3.7: the relocated derived tree, end to end ─────────────────

/// A corpus whose derived tree is at the managed layout's path compiles,
/// indexes, and is judged there: fresh, stale, missing and orphaned all get the
/// answer they get at the default path.
///
/// Built from nothing on every run, so an empty or default corpus cannot
/// produce the green: the first assertion is that the shards landed under
/// `.statecraft/derived/` and that `.derived/` was never created.
#[test]
fn statecraft_derived_layout_compiles_indexes_and_is_judged() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fs::write(
        root.join("spec-spine.toml"),
        "[layout]\nderived_dir = \".statecraft/derived\"\nstate_dir = \".statecraft/state\"\n",
    )
    .unwrap();
    write_spec(root, "001-a", "001-a", "approved");
    write_spec(root, "002-b", "002-b", "approved");
    let run = |args: &[&str]| bin().arg("--repo").arg(root).args(args).output().unwrap();

    assert_eq!(code(&run(&["compile"])), 0);
    assert_eq!(code(&run(&["index"])), 0);

    let shard = root.join(".statecraft/derived/spec-registry/by-spec/001-a.json");
    assert!(
        shard.is_file(),
        "the registry shard landed at the configured path"
    );
    assert!(
        root.join(".statecraft/derived/codebase-index/by-spec/001-a.json")
            .is_file(),
        "and so did the index shard"
    );
    assert!(
        !root.join(".derived").exists(),
        "nothing was written at the default path"
    );

    // Fresh.
    assert_eq!(code(&run(&["check"])), 0, "both trees are current");

    // Stale: edit a spec without recomputing.
    let spec_md = root.join("specs/002-b/spec.md");
    let body = fs::read_to_string(&spec_md).unwrap();
    fs::write(
        &spec_md,
        body.replace("summary: \"s\"", "summary: \"changed\""),
    )
    .unwrap();
    assert_eq!(code(&run(&["check"])), 2, "a stale relocated tree is stale");
    assert_eq!(code(&run(&["compile"])), 0);
    assert_eq!(code(&run(&["index"])), 0);
    assert_eq!(code(&run(&["check"])), 0, "and recomputing clears it");

    // Missing: delete a committed shard.
    let missing = root.join(".statecraft/derived/spec-registry/by-spec/002-b.json");
    fs::remove_file(&missing).unwrap();
    assert_eq!(code(&run(&["check"])), 2, "a missing shard is staleness");
    assert!(
        !missing.exists(),
        "the gate did not repair the tree it judged"
    );
    assert_eq!(code(&run(&["compile"])), 0);
    assert!(missing.is_file(), "and a writing compile did");

    // Orphaned: a committed shard whose spec the corpus no longer has (spec
    // 095). Produced the way it happens in life, by removing the spec and
    // leaving the shard, rather than by inventing a file: a hand-written stray
    // is a content mismatch, which is staleness and a different answer.
    write_spec(root, "003-c", "003-c", "approved");
    assert_eq!(code(&run(&["compile"])), 0);
    assert_eq!(code(&run(&["index"])), 0);
    assert_eq!(code(&run(&["check"])), 0);
    fs::remove_dir_all(root.join("specs/003-c")).unwrap();
    let orphaned = run(&["check"]);
    assert_eq!(
        code(&orphaned),
        2,
        "a stray shard at the relocated path is refused, as it is at the default \
         (spec 076: exit 2, named as orphaned)"
    );
    let report = format!(
        "{}{}",
        String::from_utf8_lossy(&orphaned.stdout),
        String::from_utf8_lossy(&orphaned.stderr)
    );
    assert!(report.contains("003-c"), "the orphan is named: {report}");
    fs::remove_file(root.join(".statecraft/derived/spec-registry/by-spec/003-c.json")).unwrap();
    fs::remove_file(root.join(".statecraft/derived/codebase-index/by-spec/003-c.json")).unwrap();
    assert_eq!(code(&run(&["check"])), 0);

    // The effective configuration reports the relocated roots, and the gate's
    // bypass floor carries the configured derived root (spec 092 §3.8).
    let cfg = run(&["config", "show"]);
    assert_eq!(code(&cfg), 0);
    let text = String::from_utf8_lossy(&cfg.stdout);
    assert!(
        text.contains("derived_dir = \".statecraft/derived\""),
        "{text}"
    );
    assert!(text.contains("state_dir = \".statecraft/state\""), "{text}");
    assert!(
        text.contains(".statecraft/derived/"),
        "the bypass floor names the configured derived root: {text}"
    );
}
