// Spec: specs/188-a-repository-pin-selects-the-engine/spec.md
//! Shared fixtures for the launcher's V-1 tests: stub engines (small shell
//! scripts), a store to put them in, a local release fixture built with the
//! `tar` command, and a launcher command with a scrubbed environment. No
//! network is used anywhere.
#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use sha2::{Digest, Sha256};
use tempfile::TempDir;

pub const LAUNCHER: &str = env!("CARGO_BIN_EXE_spec-spine-launcher");

/// The targets a release publishes as tar.gz (the Windows zip is left out of
/// the fixture, so it reads as not published).
pub const UNIX_TARGETS: [&str; 4] = [
    "aarch64-apple-darwin",
    "aarch64-unknown-linux-gnu",
    "x86_64-apple-darwin",
    "x86_64-unknown-linux-gnu",
];

pub fn host_target() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
        ("linux", "aarch64") => "aarch64-unknown-linux-gnu",
        ("macos", "x86_64") => "x86_64-apple-darwin",
        ("macos", "aarch64") => "aarch64-apple-darwin",
        other => panic!("these tests run on Unix hosts, not {other:?}"),
    }
}

/// Writing a script and executing it while sibling tests fork can fail with
/// ETXTBSY. Writes are serialised, land under a temporary name and are renamed
/// into place, so the executable is never open for writing.
static WRITE: Mutex<()> = Mutex::new(());
static SEQ: AtomicUsize = AtomicUsize::new(0);

pub fn write_exec(path: &Path, contents: &str) {
    use std::os::unix::fs::PermissionsExt;
    let _guard = WRITE.lock().unwrap_or_else(|e| e.into_inner());
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let tmp = path.with_extension(format!("tmp{}", SEQ.fetch_add(1, Ordering::SeqCst)));
    fs::write(&tmp, contents).unwrap();
    fs::set_permissions(&tmp, fs::Permissions::from_mode(0o755)).unwrap();
    fs::rename(&tmp, path).unwrap();
}

/// A stub engine of `release`. `tag` makes two stubs of one release differ.
pub fn stub(release: &str, tag: &str) -> String {
    format!(
        r#"#!/bin/sh
# stub engine {release} {tag}
if [ "$1" = "--version" ]; then echo "spec-spine {release}"; exit 0; fi
if [ "$1" = "launcher" ]; then exit 3; fi
if [ -n "$STUB_OUT" ]; then
  {{
    echo "release={release}"
    echo "cwd=$(pwd -P)"
    echo "launched=$SPEC_SPINE_LAUNCHED"
    for a in "$@"; do echo "arg=$a"; done
  }} > "$STUB_OUT"
fi
echo "engine {release}"
exit "${{STUB_EXIT:-0}}"
"#
    )
}

pub fn sha256_bytes(b: &[u8]) -> String {
    let d = Sha256::digest(b);
    d.iter().map(|x| format!("{x:02x}")).collect()
}

pub fn sha256_file(p: &Path) -> String {
    sha256_bytes(&fs::read(p).unwrap())
}

/// One scratch world: a home for the store, a decoy directory first on `PATH`,
/// and directories for repositories and release fixtures.
pub struct World {
    pub dir: TempDir,
    pub home: PathBuf,
    pub decoy: PathBuf,
}

impl World {
    pub fn new() -> World {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        let decoy = dir.path().join("decoy");
        fs::create_dir_all(&decoy).unwrap();
        World { dir, home, decoy }
    }

    pub fn path(&self, rel: &str) -> PathBuf {
        self.dir.path().join(rel)
    }

    /// A launcher command in `cwd` with every SPEC_SPINE_* variable cleared.
    pub fn cmd(&self, cwd: &Path) -> Command {
        let mut c = Command::new(LAUNCHER);
        c.env_clear()
            .env("PATH", format!("{}:/usr/bin:/bin", self.decoy.display()))
            .env("HOME", self.dir.path())
            .env("SPEC_SPINE_HOME", &self.home)
            .current_dir(cwd);
        c
    }

    /// A repository at `name` pinning `pin` (the literal `required_version`).
    pub fn repo(&self, name: &str, pin: &str) -> PathBuf {
        self.repo_with(name, pin, "")
    }

    pub fn repo_with(&self, name: &str, pin: &str, extra: &str) -> PathBuf {
        let root = self.path(name);
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("spec-spine.toml"),
            format!("[meta]\nrequired_version = \"{pin}\"\n{extra}"),
        )
        .unwrap();
        root
    }

    pub fn write_lock(&self, repo: &Path, release: &str, digest: &str, extra: &str) {
        fs::write(
            repo.join("spec-spine.lock"),
            format!(
                "[engine]\nrelease = \"{release}\"\n{extra}[engine.digests]\n\"{}\" = \"sha256:{digest}\"\n",
                host_target()
            ),
        )
        .unwrap();
    }

    /// Put `contents` into the store as `release`'s engine for this host,
    /// under its own digest. Returns the digest.
    pub fn store_engine(&self, release: &str, contents: &str) -> String {
        let hex = sha256_bytes(contents.as_bytes());
        self.store_engine_as(release, &hex, contents);
        hex
    }

    pub fn store_engine_as(&self, release: &str, name: &str, contents: &str) {
        let exe = self.entry_dir(release, name).join("spec-spine");
        write_exec(&exe, contents);
    }

    pub fn entry_dir(&self, release: &str, name: &str) -> PathBuf {
        self.home
            .join("engines")
            .join(release)
            .join(host_target())
            .join(name)
    }

    /// Everything under the store for `release`, as names.
    pub fn store_names(&self, release: &str) -> Vec<String> {
        let d = self.home.join("engines").join(release).join(host_target());
        let mut v: Vec<String> = fs::read_dir(d)
            .map(|rd| {
                rd.flatten()
                    .map(|e| e.file_name().to_string_lossy().into_owned())
                    .collect()
            })
            .unwrap_or_default();
        v.sort();
        v
    }

    /// The hex-named entries only.
    pub fn store_entries(&self, release: &str) -> Vec<String> {
        self.store_names(release)
            .into_iter()
            .filter(|n| n.len() == 64 && n.bytes().all(|b| b.is_ascii_hexdigit()))
            .collect()
    }

    /// A release fixture under `base/v<release>/`: a tar.gz per Unix target,
    /// each with the `.sha256` sidecar the release workflow publishes. Returns
    /// the digest of the engine inside (the same in every archive).
    pub fn release_fixture(&self, base: &Path, release: &str, engine: &str) -> String {
        let stage = self.path(&format!("stage-{release}"));
        write_exec(&stage.join("spec-spine"), engine);
        fs::write(stage.join("LICENSE"), "fixture\n").unwrap();
        let out = base.join(format!("v{release}"));
        fs::create_dir_all(&out).unwrap();
        for t in UNIX_TARGETS {
            let name = format!("spec-spine-v{release}-{t}.tar.gz");
            let archive = out.join(&name);
            let status = Command::new("tar")
                .arg("-C")
                .arg(&stage)
                .arg("-czf")
                .arg(&archive)
                .arg(".")
                .status()
                .unwrap();
            assert!(status.success(), "tar failed");
            fs::write(
                out.join(format!("{name}.sha256")),
                format!("{}  {name}\n", sha256_file(&archive)),
            )
            .unwrap();
        }
        sha256_bytes(engine.as_bytes())
    }
}

pub fn read_kv(path: &Path) -> Vec<String> {
    fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(str::to_string)
        .collect()
}

pub fn stdout(o: &std::process::Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

pub fn stderr(o: &std::process::Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

pub fn code(o: &std::process::Output) -> i32 {
    o.status.code().expect("exited by signal")
}
