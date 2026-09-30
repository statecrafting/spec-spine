//! Resolution (spec 188 section 3.4): override, project tool directory, user
//! store, and nothing else. `PATH` is never consulted.
//!
//! The first candidate that answers decides, and none falls through to a
//! different release. Every candidate is digested before it is trusted; when a
//! lock exists the digest is compared with the lock's, and a store entry is
//! also compared with the name it is stored under.

use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::Duration;

use crate::failure::{Failure, Res};
use crate::hash;
use crate::repo::{self, Lock, Repo};
use crate::target;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rule {
    Override,
    ToolDir,
    Store,
}

impl Rule {
    pub fn token(self) -> &'static str {
        match self {
            Rule::Override => "override",
            Rule::ToolDir => "tool-dir",
            Rule::Store => "store",
        }
    }
}

#[derive(Debug)]
pub struct Selection {
    pub path: PathBuf,
    /// Lowercase hex SHA-256 of the file, verified at resolution time.
    pub digest: String,
    pub rule: Rule,
}

pub enum Found {
    Engine(Selection),
    /// No candidate answered: the pinned engine is not installed.
    Missing,
}

pub struct Req<'a> {
    pub repo: &'a Repo,
    pub release: &'a str,
    pub target: &'a str,
    pub lock: Option<&'a Lock>,
    /// The directory holding `engines/` (see `paths::data_root`).
    pub data_root: &'a Path,
    pub override_path: Option<PathBuf>,
    /// Quarantine a corrupted store entry. Off for `launcher resolve`, which
    /// writes nothing.
    pub quarantine: bool,
}

/// `<data root>/engines/<release>/<target>`.
pub fn store_dir(data_root: &Path, release: &str, target: &str) -> PathBuf {
    data_root.join("engines").join(release).join(target)
}

fn run_probe(path: &Path, args: &[&str]) -> io::Result<Output> {
    let mut last = None;
    for _ in 0..20 {
        match Command::new(path)
            .args(args)
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output()
        {
            // A file written a moment ago can still be open for writing in a
            // forked sibling; the condition clears on its own.
            Err(e) if e.kind() == io::ErrorKind::ExecutableFileBusy => {
                last = Some(e);
                std::thread::sleep(Duration::from_millis(25));
            }
            other => return other,
        }
    }
    Err(last.unwrap_or_else(|| io::Error::other("probe failed")))
}

/// Whether `path` is itself a launcher: it answers `launcher --version`.
pub fn is_launcher(path: &Path) -> bool {
    run_probe(path, &["launcher", "--version"]).is_ok_and(|o| {
        o.status.success()
            && String::from_utf8_lossy(&o.stdout)
                .trim_start()
                .starts_with(crate::NAME)
    })
}

/// The release an engine reports for `--version` (the last word of line one).
fn reported_release(path: &Path) -> Res<String> {
    let out = run_probe(path, &["--version"])
        .map_err(|e| Failure::refused(format!("cannot run {} --version: {e}", path.display())))?;
    if !out.status.success() {
        return Err(Failure::refused(format!(
            "{} --version did not succeed",
            path.display()
        )));
    }
    let text = String::from_utf8_lossy(&out.stdout);
    Ok(text
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().last())
        .unwrap_or("")
        .to_string())
}

fn refuse_launcher(path: &Path) -> Failure {
    Failure::internal(format!(
        "{} is itself a launcher (it answers `launcher --version`); the launcher never executes a launcher",
        path.display()
    ))
}

fn digest_file(path: &Path) -> Res<String> {
    hash::file(path).map_err(|e| Failure::io(format!("cannot read {}: {e}", path.display())))
}

fn mismatch(path: &Path, expected: &str, actual: &str, source: &str) -> Failure {
    Failure::refused(format!(
        "{} does not match {source}: expected sha256:{expected}, found sha256:{actual}",
        path.display()
    ))
}

/// A candidate outside the store (override or tool directory): digested first,
/// so a locked repository never executes a file the lock does not name.
fn check_external(
    path: &Path,
    rule: Rule,
    release: &str,
    expected: Option<&str>,
) -> Res<Selection> {
    let digest = digest_file(path)?;
    if let Some(want) = expected
        && want != digest
    {
        return Err(mismatch(
            path,
            want,
            &digest,
            "the lock's digest for this target",
        ));
    }
    if is_launcher(path) {
        return Err(refuse_launcher(path));
    }
    let reported = reported_release(path)?;
    if reported != release {
        return Err(Failure::refused(format!(
            "{} reports release {reported:?} but the pin is {release}; no other release is substituted",
            path.display()
        )));
    }
    Ok(Selection {
        path: path.to_path_buf(),
        digest,
        rule,
    })
}

pub fn resolve(req: &Req<'_>) -> Res<Found> {
    let expected: Option<&str> = match req.lock {
        None => None,
        Some(lock) => Some(
            lock.digests
                .get(req.target)
                .map(String::as_str)
                .ok_or_else(|| {
                    Failure::config(format!(
                        "spec-spine.lock records no digest for target {}; run `spec-spine launcher lock`",
                        req.target
                    ))
                })?,
        ),
    };

    if let Some(path) = &req.override_path {
        if !path.is_absolute() || !path.is_file() {
            return Err(Failure::refused(format!(
                "SPEC_SPINE_ENGINE must name an existing file by absolute path, got {}",
                path.display()
            )));
        }
        let s = check_external(path, Rule::Override, req.release, expected)?;
        return Ok(Found::Engine(s));
    }

    let path = req
        .repo
        .root
        .join(repo::TOOL_DIR)
        .join(target::exe_name(req.target));
    if path.is_file() {
        let s = check_external(&path, Rule::ToolDir, req.release, expected)?;
        return Ok(Found::Engine(s));
    }

    resolve_store(req, expected)
}

fn resolve_store(req: &Req<'_>, expected: Option<&str>) -> Res<Found> {
    let dir = store_dir(req.data_root, req.release, req.target);
    let name = match expected {
        Some(hex) => hex.to_string(),
        None => {
            let mut entries = Vec::new();
            if let Ok(rd) = std::fs::read_dir(&dir) {
                for e in rd.flatten() {
                    let n = e.file_name().to_string_lossy().into_owned();
                    if hash::is_hex_digest(&n) {
                        entries.push(n);
                    }
                }
            }
            entries.sort();
            match entries.len() {
                0 => return Ok(Found::Missing),
                1 => entries.remove(0),
                _ => {
                    return Err(Failure::refused(format!(
                        "{} holds {} engine entries for {} and no spec-spine.lock says which is meant; \
                         run `spec-spine launcher lock` to bind one",
                        dir.display(),
                        entries.len(),
                        req.release
                    )));
                }
            }
        }
    };
    let entry = dir.join(&name);
    let path = entry.join(target::exe_name(req.target));
    if !path.is_file() {
        return Ok(Found::Missing);
    }
    let digest = digest_file(&path)?;
    if digest != name {
        let mut msg = format!(
            "store entry {} does not match its name: expected sha256:{name}, found sha256:{digest}",
            path.display()
        );
        if req.quarantine {
            let moved = quarantine(&dir, &name)?;
            msg.push_str(&format!("; quarantined as {}", moved.display()));
        } else {
            msg.push_str("; it was left in place because this read writes nothing");
        }
        return Err(Failure::refused(msg));
    }
    if is_launcher(&path) {
        return Err(refuse_launcher(&path));
    }
    Ok(Found::Engine(Selection {
        path,
        digest,
        rule: Rule::Store,
    }))
}

/// Rename a bad entry to a sibling `<sha256>.quarantined-<n>`.
fn quarantine(dir: &Path, name: &str) -> Res<PathBuf> {
    for n in 1..1000 {
        let to = dir.join(format!("{name}.quarantined-{n}"));
        if !to.exists() {
            std::fs::rename(dir.join(name), &to).map_err(|e| {
                Failure::io(format!(
                    "cannot quarantine {}: {e}",
                    dir.join(name).display()
                ))
            })?;
            return Ok(to);
        }
    }
    Err(Failure::io(format!(
        "too many quarantined copies of {name} in {}",
        dir.display()
    )))
}
