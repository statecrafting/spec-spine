//! Acquisition (spec 188 section 3.5): the policy that permits it, and the
//! install that performs it.
//!
//! Acquisition follows `install.sh`: the archive and its `.sha256` sidecar are
//! downloaded with `curl` (or copied when the release source is a local
//! directory), the archive is checked against the sidecar, and it is extracted
//! with `tar`. The engine inside is then digested; the lock, when present, must
//! name that digest (D-5). Nothing is placed in the store until every check has
//! passed, and placement is one rename under a per-artifact lock file (D-9).

use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant, SystemTime};

use crate::engine::{self, Found, Req, Selection};
use crate::failure::{Failure, Res};
use crate::hash;
use crate::paths;
use crate::target;

pub const DEFAULT_RELEASE_BASE: &str =
    "https://github.com/statecrafting/spec-spine/releases/download";

/// A lock file older than this belongs to a process that died.
const STALE_LOCK: Duration = Duration::from_secs(600);
/// How long a waiting installer polls before giving up.
const LOCK_WAIT: Duration = Duration::from_secs(180);
const POLL: Duration = Duration::from_millis(50);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Policy {
    Never,
    Auto,
}

fn parse_policy(value: &str) -> Option<Policy> {
    match value {
        "never" => Some(Policy::Never),
        "auto" => Some(Policy::Auto),
        _ => None,
    }
}

/// The acquisition policy: `SPEC_SPINE_FROZEN=1` first and above every other
/// source, then the flag, the environment, the user's configuration, and
/// `never`. Nothing in the repository is consulted.
pub fn policy(flag: Option<&str>) -> Res<Policy> {
    if std::env::var("SPEC_SPINE_FROZEN").as_deref() == Ok("1") {
        return Ok(Policy::Never);
    }
    if let Some(v) = flag {
        return parse_policy(v).ok_or_else(|| {
            Failure::usage(format!("--acquire takes `never` or `auto`, got {v:?}"))
        });
    }
    if let Some(v) = std::env::var_os("SPEC_SPINE_ACQUIRE").filter(|v| !v.is_empty()) {
        let v = v.to_string_lossy();
        return parse_policy(&v).ok_or_else(|| {
            Failure::config(format!(
                "SPEC_SPINE_ACQUIRE takes `never` or `auto`, got {v:?}"
            ))
        });
    }
    let file = paths::config_file()?;
    match fs::read_to_string(&file) {
        Ok(text) => {
            #[derive(serde::Deserialize, Default)]
            struct UserConfig {
                acquire: Option<String>,
            }
            let cfg: UserConfig = toml::from_str(&text)
                .map_err(|e| Failure::config(format!("{} is not readable: {e}", file.display())))?;
            match cfg.acquire {
                None => Ok(Policy::Never),
                Some(v) => parse_policy(&v).ok_or_else(|| {
                    Failure::config(format!(
                        "{}: acquire takes `never` or `auto`, got {v:?}",
                        file.display()
                    ))
                }),
            }
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Policy::Never),
        Err(e) => Err(Failure::io(format!("cannot read {}: {e}", file.display()))),
    }
}

/// Where release archives come from.
enum Source {
    Dir(PathBuf),
    Url(String),
}

enum Fetched {
    Got,
    NotPublished,
}

impl Source {
    fn from_env() -> Source {
        let base = std::env::var("SPEC_SPINE_RELEASE_BASE")
            .ok()
            .filter(|b| !b.is_empty())
            .unwrap_or_else(|| DEFAULT_RELEASE_BASE.to_string());
        if base.starts_with("http://") || base.starts_with("https://") {
            return Source::Url(base.trim_end_matches('/').to_string());
        }
        let mut path = base.strip_prefix("file://").unwrap_or(&base).to_string();
        if cfg!(windows) {
            let b = path.as_bytes();
            if b.len() > 2 && b[0] == b'/' && b[2] == b':' {
                path.remove(0);
            }
        }
        Source::Dir(PathBuf::from(path))
    }

    /// Copy or download `<base>/v<release>/<name>` to `dest`.
    fn fetch(&self, release: &str, name: &str, dest: &Path) -> Res<Fetched> {
        match self {
            Source::Dir(base) => {
                let from = base.join(format!("v{release}")).join(name);
                if !from.is_file() {
                    return Ok(Fetched::NotPublished);
                }
                fs::copy(&from, dest)
                    .map_err(|e| Failure::io(format!("cannot copy {}: {e}", from.display())))?;
                Ok(Fetched::Got)
            }
            Source::Url(base) => {
                let url = format!("{base}/v{release}/{name}");
                let status = Command::new("curl")
                    .args(["-fsSL", "-o"])
                    .arg(dest)
                    .arg(&url)
                    .stdin(Stdio::null())
                    .status()
                    .map_err(|e| {
                        Failure::io(format!("cannot run curl (needed to download {url}): {e}"))
                    })?;
                match status.code() {
                    Some(0) => Ok(Fetched::Got),
                    // curl -f: the server answered with an HTTP error.
                    Some(22) => Ok(Fetched::NotPublished),
                    _ => Err(Failure::io(format!("download failed: {url} ({status})"))),
                }
            }
        }
    }
}

/// A scratch directory removed when dropped.
struct Scratch(PathBuf);

impl Scratch {
    fn create(parent: &Path, prefix: &str) -> Res<Scratch> {
        let nanos = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let dir = parent.join(format!("{prefix}{}-{nanos}", std::process::id()));
        fs::create_dir_all(&dir)
            .map_err(|e| Failure::io(format!("cannot create {}: {e}", dir.display())))?;
        Ok(Scratch(dir))
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct Extracted {
    /// The engine executable, extracted and not yet placed anywhere.
    path: PathBuf,
    digest: String,
}

fn extract(archive: &Path, into: &Path, target: &str) -> Res<()> {
    fs::create_dir_all(into)
        .map_err(|e| Failure::io(format!("cannot create {}: {e}", into.display())))?;
    let run = |cmd: &mut Command| {
        cmd.stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
    };
    let ok = if target::is_windows(target) {
        // bsdtar reads zip; where tar cannot, `unzip` is the fallback.
        let via_tar = run(Command::new("tar")
            .arg("-xf")
            .arg(archive)
            .arg("-C")
            .arg(into));
        via_tar.is_ok_and(|s| s.success())
            || run(Command::new("unzip")
                .args(["-q", "-o"])
                .arg(archive)
                .arg("-d")
                .arg(into))
            .is_ok_and(|s| s.success())
    } else {
        run(Command::new("tar")
            .arg("-xzf")
            .arg(archive)
            .arg("-C")
            .arg(into))
        .map_err(|e| Failure::io(format!("cannot run tar: {e}")))?
        .success()
    };
    if ok {
        Ok(())
    } else if target::is_windows(target) {
        Err(Failure::refused(format!(
            "cannot extract {}: zip archives need a tar that reads zip, or unzip, on PATH; \
             Windows acquisition is not supported without one",
            archive.display()
        )))
    } else {
        Err(Failure::io(format!(
            "extract failed: {}",
            archive.display()
        )))
    }
}

/// Fetch, verify against the published sidecar, and extract one target's
/// engine. `None` when the release does not publish the target's archive.
fn fetch_engine(
    source: &Source,
    release: &str,
    target: &str,
    work: &Path,
) -> Res<Option<Extracted>> {
    let name = target::archive_name(release, target);
    let archive = work.join(&name);
    if matches!(
        source.fetch(release, &name, &archive)?,
        Fetched::NotPublished
    ) {
        return Ok(None);
    }
    let sidecar = work.join(format!("{name}.sha256"));
    if matches!(
        source.fetch(release, &format!("{name}.sha256"), &sidecar)?,
        Fetched::NotPublished
    ) {
        return Err(Failure::refused(format!(
            "release {release} publishes {name} without its .sha256 sidecar; nothing was installed"
        )));
    }
    let text = fs::read_to_string(&sidecar)
        .map_err(|e| Failure::io(format!("cannot read {}: {e}", sidecar.display())))?;
    let published = text
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    if !hash::is_hex_digest(&published) {
        return Err(Failure::refused(format!(
            "{name}.sha256 holds no SHA-256 digest"
        )));
    }
    let actual = hash::file(&archive)
        .map_err(|e| Failure::io(format!("cannot read {}: {e}", archive.display())))?;
    if actual != published {
        return Err(Failure::refused(format!(
            "{name} does not match its published digest: expected sha256:{published}, found sha256:{actual}; nothing was installed"
        )));
    }
    let x = work.join(format!("x-{target}"));
    extract(&archive, &x, target)?;
    let path = x.join(target::exe_name(target));
    let regular = fs::symlink_metadata(&path).is_ok_and(|m| m.is_file());
    if !regular {
        return Err(Failure::refused(format!(
            "{name} did not contain a regular file {}",
            target::exe_name(target)
        )));
    }
    let digest = hash::file(&path)
        .map_err(|e| Failure::io(format!("cannot read {}: {e}", path.display())))?;
    Ok(Some(Extracted { path, digest }))
}

/// What an install did.
pub enum Installed {
    /// This process placed the entry.
    Placed(Selection),
    /// The engine was already resolvable (possibly placed by a concurrent install).
    Present(Selection),
}

/// A per-artifact install lock: an exclusive create beside the entries.
struct InstallLock(PathBuf);

impl Drop for InstallLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn try_lock(path: &Path) -> Res<Option<InstallLock>> {
    match OpenOptions::new().write(true).create_new(true).open(path) {
        Ok(mut f) => {
            let _ = writeln!(f, "{}", std::process::id());
            Ok(Some(InstallLock(path.to_path_buf())))
        }
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
            let stale = fs::metadata(path)
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| t.elapsed().ok())
                .is_some_and(|age| age > STALE_LOCK);
            if stale {
                let _ = fs::remove_file(path);
            }
            Ok(None)
        }
        Err(e) => Err(Failure::io(format!(
            "cannot create {}: {e}",
            path.display()
        ))),
    }
}

/// Prepare the pinned engine: resolve it, and if nothing answers, download,
/// verify and place it. Concurrent installs of one artifact never write the
/// same path: one holds the lock and writes; the others wait and use its entry.
pub fn install(req: &Req<'_>) -> Res<Installed> {
    let dir = engine::store_dir(req.data_root, req.release, req.target);
    fs::create_dir_all(&dir)
        .map_err(|e| Failure::io(format!("cannot create {}: {e}", dir.display())))?;
    let lock_path = dir.join(".install.lock");
    let deadline = Instant::now() + LOCK_WAIT;
    loop {
        if let Found::Engine(s) = engine::resolve(req)? {
            return Ok(Installed::Present(s));
        }
        if let Some(_guard) = try_lock(&lock_path)? {
            // The winner of a race may have finished between the resolve above
            // and the lock: look again before writing.
            if let Found::Engine(s) = engine::resolve(req)? {
                return Ok(Installed::Present(s));
            }
            place(req, &dir)?;
            return match engine::resolve(req)? {
                Found::Engine(s) => Ok(Installed::Placed(s)),
                Found::Missing => Err(Failure::internal(
                    "the engine was placed but does not resolve".to_string(),
                )),
            };
        }
        if Instant::now() >= deadline {
            return Err(Failure::io(format!(
                "timed out waiting for another install to finish ({})",
                lock_path.display()
            )));
        }
        std::thread::sleep(POLL);
    }
}

fn place(req: &Req<'_>, dir: &Path) -> Res<()> {
    let source = Source::from_env();
    let scratch = Scratch::create(dir, ".tmp-")?;
    let got = fetch_engine(&source, req.release, req.target, &scratch.0)?.ok_or_else(|| {
        Failure::refused(format!(
            "release {} does not publish target {}; supported targets: {}. \
                 Building from source is only ever an explicit `launcher install --build`",
            req.release,
            req.target,
            target::TARGETS.join(", ")
        ))
    })?;
    if let Some(lock) = req.lock {
        let want = lock.digests.get(req.target).map_or("", String::as_str);
        if want != got.digest {
            return Err(Failure::refused(format!(
                "spec-spine.lock names sha256:{want} for {} but the downloaded archive holds an engine with sha256:{}; nothing was installed",
                req.target, got.digest
            )));
        }
    }
    let entry_tmp = scratch.0.join("entry");
    fs::create_dir_all(&entry_tmp)
        .map_err(|e| Failure::io(format!("cannot create {}: {e}", entry_tmp.display())))?;
    let exe = entry_tmp.join(target::exe_name(req.target));
    fs::rename(&got.path, &exe)
        .map_err(|e| Failure::io(format!("cannot stage {}: {e}", exe.display())))?;
    make_executable(&exe)?;
    let entry = dir.join(&got.digest);
    match fs::rename(&entry_tmp, &entry) {
        Ok(()) => Ok(()),
        // An entry of that name already exists; resolution verifies it, and a
        // bad one is quarantined by the resolve that follows.
        Err(_) if entry.exists() => Ok(()),
        Err(e) => Err(Failure::io(format!(
            "cannot place {}: {e}",
            entry.display()
        ))),
    }
}

#[cfg(unix)]
fn make_executable(path: &Path) -> Res<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755))
        .map_err(|e| Failure::io(format!("cannot chmod {}: {e}", path.display())))
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> Res<()> {
    Ok(())
}

/// `launcher lock`: digest the engine of every target the release publishes.
/// Fetches each target's archive into a scratch directory and writes nothing
/// to the store.
pub fn compute_digests(release: &str) -> Res<BTreeMap<String, String>> {
    let source = Source::from_env();
    let scratch = Scratch::create(&std::env::temp_dir(), "spec-spine-lock-")?;
    let mut digests = BTreeMap::new();
    for t in target::TARGETS {
        if let Some(got) = fetch_engine(&source, release, t, &scratch.0)? {
            digests.insert(t.to_string(), got.digest);
        }
    }
    if digests.is_empty() {
        return Err(Failure::refused(format!(
            "release {release} publishes no archive for any known target ({}); no lock was written",
            target::TARGETS.join(", ")
        )));
    }
    Ok(digests)
}
