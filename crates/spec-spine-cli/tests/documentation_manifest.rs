//! Spec 160: `documentation manifest validate|freshness` bind one verified Git
//! tree and answer with a document, a finding envelope or a refusal envelope.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::{Value, json};

const OUT_TEXT: &str = "# Alpha\nGenerated from alpha.\nA human wrote this.\n";

fn git(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

fn bin(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_spec-spine"))
        .arg("--repo")
        .arg(root)
        .args(args)
        .output()
        .unwrap()
}

fn sha(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let hex: String = Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    format!("sha256:{hex}")
}

/// A one-spec corpus with a committed ledger, one input and one output.
fn fixture() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("specs/001-a");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("spec.md"),
        "---\nid: \"001-a\"\ntitle: \"T\"\nstatus: draft\ncreated: \"2026-10-10\"\n\
         summary: \"s\"\n---\n# 001\n\nThe body.\n",
    )
    .unwrap();
    fs::create_dir_all(tmp.path().join("docs")).unwrap();
    fs::write(tmp.path().join("alpha.txt"), "alpha\n").unwrap();
    fs::write(tmp.path().join("docs/out.md"), OUT_TEXT).unwrap();
    let compiled = bin(tmp.path(), &["compile"]);
    assert_eq!(compiled.status.code(), Some(0));
    git(tmp.path(), &["init", "-q"]);
    git(
        tmp.path(),
        &["config", "user.email", "test@example.invalid"],
    );
    git(tmp.path(), &["config", "user.name", "Test"]);
    git(tmp.path(), &["config", "commit.gpgsign", "false"]);
    fs::write(tmp.path().join(".gitignore"), "build-meta.json\n").unwrap();
    commit(tmp.path(), "fixture");
    tmp
}

fn commit(root: &Path, message: &str) {
    git(root, &["add", "-A"]);
    git(root, &["commit", "-qm", message]);
}

/// Write `text` into a fresh scratch file outside the repository.
fn scratch(dir: &Path, name: &str, text: &str) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, text).unwrap();
    path
}

/// The single packet page for the fixture's request, written to `dir`.
fn packet(root: &Path, dir: &Path, name: &str) -> (PathBuf, Value) {
    let request = scratch(
        dir,
        "request.json",
        &json!({
            "closure": { "root": "001" },
            "members": [
                { "selector": { "kind": "file", "path": "alpha.txt" }, "requirement": "required" }
            ],
            "consumerSchemaVersion": "1.0.0"
        })
        .to_string(),
    );
    let out = bin(
        root,
        &[
            "context",
            "packet",
            "--request",
            request.to_str().unwrap(),
            "--repository",
            "example/repo",
            "--json",
        ],
    );
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = String::from_utf8(out.stdout).unwrap();
    (
        scratch(dir, name, &text),
        serde_json::from_str(&text).unwrap(),
    )
}

/// A sealed manifest declaring `alpha.txt` and `docs/out.md` against `page`.
fn manifest(page: &Value) -> Value {
    let alpha = page["members"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["identity"] == "file:alpha.txt")
        .unwrap();
    let lines: Vec<&str> = OUT_TEXT.split_inclusive('\n').collect();
    let mut m = json!({
        "schemaVersion": "1.0.0",
        "packet": {
            "schemaVersion": page["schemaVersion"],
            "snapshot": page["snapshot"],
            "requestDigest": page["requestDigest"],
            "closureDigest": page["closureDigest"],
            "pageDigests": [page["packetDigest"]],
            "pageCompleteness": ["complete"]
        },
        "inputs": [{
            "inputId": "alpha",
            "identity": "file:alpha.txt",
            "projection": "full",
            "schemaVersion": alpha["item"]["schemaVersion"],
            "digest": alpha["item"]["digest"],
            "requirement": "required",
            "uses": ["out"]
        }],
        "interfaces": [],
        "externalReferences": [],
        "generator": {
            "name": "docgen",
            "version": "1.0.0",
            "buildDigest": sha(b"docgen"),
            "arguments": ["--all"],
            "configurationDigest": sha(b"config")
        },
        "outputs": [{
            "outputId": "out",
            "path": "docs/out.md",
            "mediaType": "text/markdown",
            "digest": sha(OUT_TEXT.as_bytes()),
            "byteLength": OUT_TEXT.len(),
            "regions": [
                {
                    "regionId": "generated",
                    "kind": "generated",
                    "span": { "startLine": 1, "endLine": 2 },
                    "digest": sha(format!("{}{}", lines[0], lines[1]).as_bytes()),
                    "inputIds": ["alpha"]
                },
                {
                    "regionId": "human",
                    "kind": "human-authored",
                    "span": { "startLine": 3, "endLine": 3 },
                    "digest": sha(lines[2].as_bytes()),
                    "inputIds": []
                }
            ],
            "examples": []
        }],
        "omissions": [],
        "manifestDigest": format!("sha256:{}", "0".repeat(64))
    });
    let typed = spec_spine_core::parse_documentation_manifest(&m.to_string()).unwrap();
    m["manifestDigest"] = json!(spec_spine_core::manifest_digest(&typed).unwrap());
    m
}

fn run(root: &Path, verb: &str, manifest: &Path, page: &Path, extra: &[&str]) -> Output {
    let mut args = vec![
        "documentation",
        "manifest",
        verb,
        "--manifest",
        manifest.to_str().unwrap(),
        "--packet",
        page.to_str().unwrap(),
        "--repository",
        "example/repo",
        "--json",
    ];
    args.extend_from_slice(extra);
    bin(root, &args)
}

fn stdout_json(out: &Output) -> Value {
    serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!(
            "{e}: {}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        )
    })
}

#[test]
fn validate_prints_the_canonical_manifest_and_freshness_is_fresh() {
    let repo = fixture();
    let dir = tempfile::tempdir().unwrap();
    let (page_path, page) = packet(repo.path(), dir.path(), "page.json");
    let m = manifest(&page);
    let manifest_path = scratch(dir.path(), "manifest.json", &m.to_string());

    let out = run(repo.path(), "validate", &manifest_path, &page_path, &[]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.ends_with("}\n"));
    assert_eq!(
        stdout_json(&run(
            repo.path(),
            "validate",
            &manifest_path,
            &page_path,
            &[]
        ))["manifestDigest"],
        m["manifestDigest"]
    );

    let out = run(
        repo.path(),
        "freshness",
        &manifest_path,
        &page_path,
        &["--output-root", "docs"],
    );
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
    let v = stdout_json(&out);
    assert_eq!(v["status"], "fresh");
    assert_eq!(v["repositoryTreeChanged"], false);
    assert_eq!(v["findings"], json!([]));
}

#[test]
fn a_changed_input_is_a_finding_in_the_envelope_without_content() {
    let repo = fixture();
    let dir = tempfile::tempdir().unwrap();
    let (_, page) = packet(repo.path(), dir.path(), "page.json");
    let manifest_path = scratch(dir.path(), "manifest.json", &manifest(&page).to_string());
    fs::write(repo.path().join("alpha.txt"), "PRIVATE-TEXT\n").unwrap();
    commit(repo.path(), "move alpha");
    let (current, _) = packet(repo.path(), dir.path(), "current.json");

    let out = run(repo.path(), "freshness", &manifest_path, &current, &[]);
    assert_eq!(out.status.code(), Some(1));
    let v = stdout_json(&out);
    assert_eq!(v["verb"], "documentation.manifest.freshness");
    assert_eq!(v["outcome"], "finding");
    let report = &v["report"];
    assert_eq!(report["status"], "stale");
    assert_eq!(report["repositoryTreeChanged"], true);
    assert_eq!(report["findings"][0]["kind"], "changed");
    assert_eq!(report["findings"][0]["identity"], "input:alpha");
    assert_eq!(report["findings"][0]["affects"], json!(["out"]));
    assert!(!String::from_utf8_lossy(&out.stdout).contains("PRIVATE-TEXT"));

    // The recorded snapshot no longer validates: the tree moved.
    let (old_page, _) = packet(repo.path(), dir.path(), "again.json");
    let out = run(repo.path(), "validate", &manifest_path, &old_page, &[]);
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(stdout_json(&out)["verb"], "documentation.manifest.validate");
}

#[test]
fn refusals_and_usage_errors_answer_with_the_envelope() {
    let repo = fixture();
    let dir = tempfile::tempdir().unwrap();
    let (page_path, page) = packet(repo.path(), dir.path(), "page.json");
    let m = manifest(&page);
    let manifest_path = scratch(dir.path(), "manifest.json", &m.to_string());

    // An output outside the named output root.
    let out = run(
        repo.path(),
        "validate",
        &manifest_path,
        &page_path,
        &["--output-root", "specs"],
    );
    assert_eq!(out.status.code(), Some(2));
    let v = stdout_json(&out);
    assert_eq!(v["error"]["kind"], "refused");
    assert!(
        v["error"]["message"]
            .as_str()
            .unwrap()
            .contains("path-escape")
    );

    // An unknown member is usage.
    let mut extra = m.clone();
    extra["extensions"] = json!({});
    let bad = scratch(dir.path(), "bad.json", &extra.to_string());
    let out = run(repo.path(), "validate", &bad, &page_path, &[]);
    assert_eq!(out.status.code(), Some(3));
    assert_eq!(stdout_json(&out)["error"]["kind"], "usage");

    // A dirty working tree is refused before anything is read.
    fs::write(repo.path().join("alpha.txt"), "dirty\n").unwrap();
    let out = run(repo.path(), "freshness", &manifest_path, &page_path, &[]);
    assert_eq!(out.status.code(), Some(2));
    let v = stdout_json(&out);
    assert_eq!(v["verb"], "documentation.manifest.freshness");
    assert_eq!(v["error"]["kind"], "refused");
}

#[test]
fn interface_exports_and_external_resolutions_are_read_from_named_files() {
    let repo = fixture();
    let dir = tempfile::tempdir().unwrap();
    let (page_path, page) = packet(repo.path(), dir.path(), "page.json");
    let mut m = manifest(&page);
    m["externalReferences"] = json!([{
        "inputId": "policy",
        "namespace": "statecraft",
        "schemaVersion": "1.0.0",
        "identity": "policy/docs",
        "digest": sha(b"policy"),
        "requirement": "required",
        "uses": ["out"],
        "purpose": "policy"
    }]);
    m["outputs"][0]["regions"][1]["inputIds"] = json!(["policy"]);
    let typed = spec_spine_core::parse_documentation_manifest(&m.to_string()).unwrap();
    m["manifestDigest"] = json!(spec_spine_core::manifest_digest(&typed).unwrap());
    let manifest_path = scratch(dir.path(), "manifest.json", &m.to_string());

    let out = run(repo.path(), "freshness", &manifest_path, &page_path, &[]);
    assert_eq!(out.status.code(), Some(1));
    let report = &stdout_json(&out)["report"];
    assert_eq!(report["status"], "unverified");
    assert_eq!(report["findings"][0]["kind"], "unverified");

    let resolutions = scratch(
        dir.path(),
        "resolutions.json",
        &json!([{
            "namespace": "statecraft",
            "schemaVersion": "1.0.0",
            "identity": "policy/docs",
            "digest": sha(b"policy")
        }])
        .to_string(),
    );
    let out = run(
        repo.path(),
        "freshness",
        &manifest_path,
        &page_path,
        &["--resolutions", resolutions.to_str().unwrap()],
    );
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
    assert_eq!(stdout_json(&out)["status"], "fresh");

    let out = run(
        repo.path(),
        "freshness",
        &manifest_path,
        &page_path,
        &["--export", "nocorpus"],
    );
    assert_eq!(out.status.code(), Some(3));
}
