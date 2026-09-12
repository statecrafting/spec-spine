//! Spec 085: `verify-attestation` decides on the bytes it was given.
//!
//! The tamper and version matrix of spec 085 1, end to end through the binary,
//! in both the corpus and the per-spec scope. Every case here verified as
//! `match` and `valid` at exit 0 before this spec, which is why the guard is at
//! the binary rather than at the library: the defect was in what the CLI handed
//! to the check, not in what the check then did.

use std::fs;
use std::path::Path;
use std::process::Command;

const SPEC_ID: &str = "001-a";

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_spec-spine"))
}

fn code(out: &std::process::Output) -> i32 {
    out.status.code().unwrap_or(-1)
}

fn run(root: &Path, args: &[&str]) -> std::process::Output {
    bin()
        .arg("--repo")
        .arg(root)
        .args(args)
        .output()
        .expect("spawn spec-spine")
}

fn stderr(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

fn stdout(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// A one-crate, one-spec corpus with a signed attestation in both scopes.
///
/// Returns the repository root, the signing key path and the public key path.
/// The public key is read back out of the seal's default `key_id`, which is the
/// hex public key, so the fixture needs no key-derivation of its own.
struct Fixture {
    _tmp: tempfile::TempDir,
    root: std::path::PathBuf,
    key: std::path::PathBuf,
    public_key: std::path::PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();
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
            "---\nid: \"001-a\"\ntitle: \"A\"\nstatus: approved\ncreated: \"2026-09-11\"\n\
             summary: \"s\"\nestablishes:\n  - \"crate-a/src/lib.rs\"\n---\n# 001-a\n## body\n",
        );
        for verb in ["compile", "index"] {
            assert_eq!(code(&run(&root, &[verb])), 0, "fixture {verb}");
        }

        let key = root.join("signing.key");
        fs::write(&key, [7u8; 32]).unwrap();
        let key_arg = key.to_str().unwrap().to_string();
        for args in [
            vec!["attest", "--sign", "--key", &key_arg],
            vec!["attest", "--spec", SPEC_ID, "--sign", "--key", &key_arg],
        ] {
            assert_eq!(code(&run(&root, &args)), 0, "fixture {args:?}");
        }

        // The default key_id is the hex public key, and `--public-key` accepts
        // hex, so the seal itself supplies the verifier's key.
        let seal: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(root.join(".derived/attestation/attestation.sig")).unwrap(),
        )
        .unwrap();
        let public_key = root.join("public.key");
        fs::write(&public_key, seal["keyId"].as_str().unwrap()).unwrap();

        Fixture {
            _tmp: tmp,
            root,
            key,
            public_key,
        }
    }

    fn corpus_attestation(&self) -> std::path::PathBuf {
        self.root.join(".derived/attestation/attestation.json")
    }

    fn corpus_seal(&self) -> std::path::PathBuf {
        self.root.join(".derived/attestation/attestation.sig")
    }

    fn spec_attestation(&self) -> std::path::PathBuf {
        self.root
            .join(format!(".derived/attestation/by-spec/{SPEC_ID}.json"))
    }

    fn spec_seal(&self) -> std::path::PathBuf {
        self.root
            .join(format!(".derived/attestation/by-spec/{SPEC_ID}.sig"))
    }

    /// Write `content` to a scratch file inside the fixture and return its path.
    fn scratch(&self, name: &str, content: &str) -> std::path::PathBuf {
        let p = self.root.join(name);
        fs::write(&p, content).unwrap();
        p
    }
}

/// Insert `member` as the payload's first top-level member.
fn inject_top_level(original: &str, member: &str) -> String {
    let (head, rest) = original
        .split_once('\n')
        .expect("canonical JSON is multi-line");
    format!("{head}\n  {member}\n{rest}")
}

// --- 3.2: an unknown member is refused ------------------------------------

/// A member this build does not know is a claim it cannot evaluate, so the file
/// fails to load (exit 3) naming the member, before either mode runs.
///
/// Before spec 085 serde dropped it and both modes reported on the smaller
/// object that remained: `match` and `valid`, at exit 0, while a consumer using
/// its own parser went on to read the member spec-spine never saw.
#[test]
fn an_unknown_member_is_refused_in_every_payload_and_both_scopes() {
    let f = Fixture::new();
    let corpus = fs::read_to_string(f.corpus_attestation()).unwrap();
    let spec = fs::read_to_string(f.spec_attestation()).unwrap();
    let seal = fs::read_to_string(f.corpus_seal()).unwrap();

    // (file under test, the member's name, the arguments that read it)
    let corpus_top = f.scratch(
        "top.json",
        &inject_top_level(&corpus, "\"prCouple\": {\"ok\": true},"),
    );
    let corpus_nested = f.scratch(
        "nested.json",
        &corpus.replace(
            "\"compile\": {",
            "\"compile\": {\n      \"ignoredWarnings\": 12,",
        ),
    );
    let spec_unit = f.scratch(
        "unit.json",
        &spec.replacen(
            "\"contentHash\":",
            "\"coveredBy\": \"review\",\n        \"contentHash\":",
            1,
        ),
    );
    let tampered_seal = f.scratch(
        "seal.sig",
        &inject_top_level(&seal, "\"revokedAt\": \"2026-01-01\","),
    );

    let public_key = f.public_key.to_str().unwrap();
    let corpus_seal = f.corpus_seal();
    let spec_seal = f.spec_seal();
    let cases: Vec<(&str, &str, Vec<&str>)> = vec![
        (
            "a top-level member",
            "prCouple",
            vec![
                "verify-attestation",
                "--attestation",
                corpus_top.to_str().unwrap(),
                "--seal",
                corpus_seal.to_str().unwrap(),
                "--recompute",
                "--signature",
                "--public-key",
                public_key,
            ],
        ),
        (
            "a nested member",
            "ignoredWarnings",
            vec![
                "verify-attestation",
                "--attestation",
                corpus_nested.to_str().unwrap(),
                "--recompute",
            ],
        ),
        (
            "a member on a per-spec attestation's unit",
            "coveredBy",
            vec![
                "verify-attestation",
                "--spec",
                SPEC_ID,
                "--attestation",
                spec_unit.to_str().unwrap(),
                "--seal",
                spec_seal.to_str().unwrap(),
                "--recompute",
                "--signature",
                "--public-key",
                public_key,
            ],
        ),
        (
            "a member on the seal",
            "revokedAt",
            vec![
                "verify-attestation",
                "--seal",
                tampered_seal.to_str().unwrap(),
                "--signature",
                "--public-key",
                public_key,
            ],
        ),
    ];

    for (what, member, args) in cases {
        let out = run(&f.root, &args);
        assert_eq!(code(&out), 3, "{what}: must fail to load, not verify");
        // Naming the member is what keeps this from passing on a missing file,
        // which is also exit 3.
        assert!(
            stderr(&out).contains(member),
            "{what}: the refusal must name `{member}`: {}",
            stderr(&out)
        );
    }
}

// --- 3.3: an unknown MAJOR is refused, and the recompute compares the version

/// A MAJOR this build does not understand is refused at exit 3, in the words the
/// registry and index loaders use, before either mode runs.
#[test]
fn an_unknown_schema_major_is_refused_in_both_scopes() {
    let f = Fixture::new();
    let corpus = fs::read_to_string(f.corpus_attestation()).unwrap();
    let spec = fs::read_to_string(f.spec_attestation()).unwrap();
    let bumped = |s: &str| {
        s.replace(
            "\"schemaVersion\": \"0.1.0\"",
            "\"schemaVersion\": \"9.0.0\"",
        )
    };

    let corpus_file = f.scratch("major.json", &bumped(&corpus));
    let out = run(
        &f.root,
        &[
            "verify-attestation",
            "--attestation",
            corpus_file.to_str().unwrap(),
            "--recompute",
        ],
    );
    assert_eq!(code(&out), 3, "corpus: a MAJOR 9 payload is unreadable");
    assert!(
        stderr(&out)
            .contains("attestation schema MAJOR 9 is unsupported (this build understands 0.x)"),
        "{}",
        stderr(&out)
    );

    let spec_file = f.scratch("major-spec.json", &bumped(&spec));
    let out = run(
        &f.root,
        &[
            "verify-attestation",
            "--spec",
            SPEC_ID,
            "--attestation",
            spec_file.to_str().unwrap(),
            "--recompute",
        ],
    );
    assert_eq!(code(&out), 3, "per-spec: a MAJOR 9 payload is unreadable");
    assert!(
        stderr(&out).contains("MAJOR 9 is unsupported"),
        "{}",
        stderr(&out)
    );

    // A schemaVersion that is not semver is equally unreadable, and says so
    // rather than reporting a content difference.
    let junk = f.scratch(
        "junk.json",
        &corpus.replace(
            "\"schemaVersion\": \"0.1.0\"",
            "\"schemaVersion\": \"zero\"",
        ),
    );
    let out = run(
        &f.root,
        &[
            "verify-attestation",
            "--attestation",
            junk.to_str().unwrap(),
            "--recompute",
        ],
    );
    assert_eq!(code(&out), 3);
    assert!(stderr(&out).contains("not semver"), "{}", stderr(&out));
}

/// A same-MAJOR version difference is readable, so it is a content mismatch, and
/// the report names the field rather than saying "tool.name or schemaVersion".
///
/// The corpus recompute skipped `schemaVersion` entirely before spec 085, so a
/// `0.2.0` payload recomputed as `match` at exit 0.
#[test]
fn the_recompute_compares_schema_version_and_names_it() {
    let f = Fixture::new();
    let corpus = fs::read_to_string(f.corpus_attestation()).unwrap();
    let spec = fs::read_to_string(f.spec_attestation()).unwrap();
    let bumped = |s: &str| {
        s.replace(
            "\"schemaVersion\": \"0.1.0\"",
            "\"schemaVersion\": \"0.2.0\"",
        )
    };

    for (scope, file, args) in [
        (
            "corpus",
            f.scratch("minor.json", &bumped(&corpus)),
            vec!["verify-attestation"],
        ),
        (
            "per-spec",
            f.scratch("minor-spec.json", &bumped(&spec)),
            vec!["verify-attestation", "--spec", SPEC_ID],
        ),
    ] {
        let mut argv = args;
        argv.extend([
            "--attestation",
            file.to_str().unwrap(),
            "--recompute",
            "--json",
        ]);
        let out = run(&f.root, &argv);
        assert_eq!(code(&out), 1, "{scope}: a readable difference is exit 1");
        let v: serde_json::Value = serde_json::from_str(&stdout(&out)).unwrap();
        assert_eq!(v["report"]["outcome"], "contentMismatch", "{scope}");
        let differences = v["report"]["differences"].to_string();
        assert!(
            differences.contains("schemaVersion (0.2.0 -> 0.1.0)"),
            "{scope}: the report must name the field: {differences}"
        );
        assert!(
            !differences.contains("tool.name or schemaVersion"),
            "{scope}: and must not send the reader off to diff two fields by hand"
        );
    }
}

// --- 3.1: both modes decide on the bytes ----------------------------------

/// Values that parse equal and bytes that are not the canonical serialization:
/// the signature is invalid and the recompute is a content mismatch naming the
/// bytes. Both were `valid` and `match` at exit 0 before spec 085.
#[test]
fn reformatted_bytes_fail_both_modes_in_both_scopes() {
    let f = Fixture::new();
    let squash = |p: &Path| fs::read_to_string(p).unwrap().replace('\n', "");

    let cases: Vec<(&str, std::path::PathBuf, std::path::PathBuf, Vec<&str>)> = vec![
        (
            "corpus",
            f.scratch("flat.json", &squash(&f.corpus_attestation())),
            f.corpus_seal(),
            vec!["verify-attestation"],
        ),
        (
            "per-spec",
            f.scratch("flat-spec.json", &squash(&f.spec_attestation())),
            f.spec_seal(),
            vec!["verify-attestation", "--spec", SPEC_ID],
        ),
    ];

    for (scope, file, seal, base) in cases {
        let mut signature = base.clone();
        signature.extend([
            "--attestation",
            file.to_str().unwrap(),
            "--seal",
            seal.to_str().unwrap(),
            "--signature",
            "--public-key",
            f.public_key.to_str().unwrap(),
        ]);
        let out = run(&f.root, &signature);
        assert_eq!(
            code(&out),
            1,
            "{scope}: a seal covers the stored bytes, not a re-serialization of their parse"
        );
        assert!(stderr(&out).contains("INVALID"), "{}", stderr(&out));

        let mut recompute = base;
        recompute.extend([
            "--attestation",
            file.to_str().unwrap(),
            "--recompute",
            "--json",
        ]);
        let out = run(&f.root, &recompute);
        assert_eq!(
            code(&out),
            1,
            "{scope}: read and failed verification is exit 1"
        );
        let v: serde_json::Value = serde_json::from_str(&stdout(&out)).unwrap();
        assert_eq!(v["report"]["outcome"], "contentMismatch", "{scope}");
        assert!(
            v["report"]["differences"]
                .to_string()
                .contains("bytes are not the canonical serialization"),
            "{scope}: and the difference says which of the two diverged: {}",
            v["report"]["differences"]
        );
    }
}

/// 3.4 closing the loop with spec 037 3.1: handed the same bytes, the facade
/// and the CLI report the same thing.
///
/// The existing parity test (`cli.rs::json_report_equals_the_facade_payload`)
/// sends the facade a parsed value read from a canonical file, so it cannot see
/// this: the two agree there whether or not the byte rule exists. The case that
/// can diverge is a file whose bytes are not canonical, and `attestationText` is
/// the form that lets the facade be asked about it at all.
#[test]
fn the_facade_and_the_cli_agree_on_bytes_that_are_not_canonical() {
    let f = Fixture::new();
    let bytes = fs::read_to_string(f.corpus_attestation())
        .unwrap()
        .replace('\n', "");
    let file = f.scratch("flat-parity.json", &bytes);

    let out = run(
        &f.root,
        &[
            "verify-attestation",
            "--attestation",
            file.to_str().unwrap(),
            "--recompute",
            "--json",
        ],
    );
    assert_eq!(code(&out), 1);
    let cli: serde_json::Value = serde_json::from_str(&stdout(&out)).unwrap();

    let request = serde_json::json!({
        "repoRoot": f.root.to_str().unwrap(),
        "attestationText": bytes,
    });
    let facade: serde_json::Value = serde_json::from_str(
        &spec_spine_core::verify_attestation_json(&request.to_string()).unwrap(),
    )
    .unwrap();

    assert_eq!(
        cli["report"], facade,
        "the envelope's report is the facade's payload for the same bytes"
    );
}

// --- 3.6: what must keep working ------------------------------------------

/// An untouched pair verifies in both scopes and in both modes, and the bytes
/// `attest` wrote are the ones its own `attestationHash` covers.
///
/// The second half is what makes the first half's byte comparison a guard
/// rather than a tautology: if `attest` wrote anything but the canonical bytes,
/// every seal it has ever produced would stop verifying under 3.1.
#[test]
fn an_untouched_pair_still_verifies_and_the_written_bytes_are_the_attested_ones() {
    let f = Fixture::new();

    for (scope, args) in [
        ("corpus", vec!["verify-attestation"]),
        ("per-spec", vec!["verify-attestation", "--spec", SPEC_ID]),
    ] {
        let mut argv = args;
        argv.extend([
            "--recompute",
            "--signature",
            "--public-key",
            f.public_key.to_str().unwrap(),
        ]);
        let out = run(&f.root, &argv);
        assert_eq!(code(&out), 0, "{scope}: {}", stderr(&out));
        assert!(stdout(&out).contains("MATCH"), "{scope}: {}", stdout(&out));
        assert!(stdout(&out).contains("VALID"), "{scope}: {}", stdout(&out));
    }

    // `attest --json` reports the hash a seal signs; the file on disk must hash
    // to it, byte for byte.
    for (scope, args, path) in [
        ("corpus", vec!["attest", "--json"], f.corpus_attestation()),
        (
            "per-spec",
            vec!["attest", "--spec", SPEC_ID, "--json"],
            f.spec_attestation(),
        ),
    ] {
        let out = run(&f.root, &args);
        assert_eq!(code(&out), 0, "{scope}");
        let v: serde_json::Value = serde_json::from_str(&stdout(&out)).unwrap();
        let reported = v["report"]["attestationHash"].as_str().unwrap();
        // `attestationHash` is SHA-256 over the canonical serialization;
        // `stored_bytes_hash` is SHA-256 over whatever is on disk. Comparing
        // them asks the question this spec turns on: are those the same bytes?
        let stored = spec_spine_core::stored_bytes_hash(&fs::read(&path).unwrap());
        assert_eq!(
            reported, stored,
            "{scope}: the written bytes must be the ones attestationHash covers"
        );
    }

    // The signing key is still where the fixture put it; a re-seal of the
    // unchanged payload verifies too, so the guard is not passing because
    // nothing was signed.
    assert!(f.key.is_file());
}
