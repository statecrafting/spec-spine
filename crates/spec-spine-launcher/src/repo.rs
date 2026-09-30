//! What the launcher reads from a repository: the root, the pin, the lock
//! (spec 188 sections 3.2 and 3.3), and nothing else.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::failure::{Failure, Res};
use crate::hash;

pub const CONFIG_FILE: &str = "spec-spine.toml";
pub const LOCK_FILE: &str = "spec-spine.lock";
/// The project tool directory (spec 188 section 3.4): a repository-local
/// engine, when one is installed, is `.bin/spec-spine`.
pub const TOOL_DIR: &str = ".bin";

#[derive(Debug)]
pub struct Repo {
    pub root: PathBuf,
    /// Whether the root was named by `--repo` (the engine then already knows it).
    pub named_by_flag: bool,
}

/// Find the repository: `--repo`, then `SPEC_SPINE_REPO`, then the nearest
/// ancestor of the working directory holding `spec-spine.toml`.
pub fn discover(cwd: &Path, flag: Option<&Path>, env: Option<PathBuf>) -> Res<Repo> {
    let named_by_flag = flag.is_some();
    if let Some(named) = flag.map(Path::to_path_buf).or(env) {
        let root = if named.is_absolute() {
            named
        } else {
            cwd.join(named)
        };
        if !root.join(CONFIG_FILE).is_file() {
            return Err(Failure::refused(format!(
                "{} holds no {CONFIG_FILE}; no engine was run",
                root.display()
            )));
        }
        return Ok(Repo {
            root,
            named_by_flag,
        });
    }
    for dir in cwd.ancestors() {
        if dir.join(CONFIG_FILE).is_file() {
            return Ok(Repo {
                root: dir.to_path_buf(),
                named_by_flag: false,
            });
        }
    }
    Err(Failure::refused(format!(
        "not inside a spec-spine repository: no {CONFIG_FILE} in {} or any parent; no engine was run",
        cwd.display()
    )))
}

// The tolerant pin reader: it names the one key it needs and ignores every
// other table and key, so a newer configuration never stops it selecting.
#[derive(Deserialize, Default)]
struct RawConfig {
    meta: Option<RawMeta>,
}

#[derive(Deserialize, Default)]
struct RawMeta {
    required_version: Option<String>,
}

/// The pinned release from `[meta] required_version`, which must be `=X.Y.Z`.
pub fn read_pin(root: &Path) -> Res<String> {
    let path = root.join(CONFIG_FILE);
    let text = std::fs::read_to_string(&path)
        .map_err(|e| Failure::io(format!("cannot read {}: {e}", path.display())))?;
    let raw: RawConfig = toml::from_str(&text).map_err(|e| {
        Failure::config(format!(
            "{} is not readable as TOML, so no pin can be read: {e}",
            path.display()
        ))
    })?;
    let Some(pin) = raw.meta.and_then(|m| m.required_version) else {
        return Err(Failure::config(format!(
            "{} has no [meta] required_version; the launcher selects an engine only by an exact pin. \
             Set required_version = \"=X.Y.Z\"",
            path.display()
        )));
    };
    parse_exact(&pin)
}

/// `=X.Y.Z` (an optional `-prerelease`), and no other form.
pub fn parse_exact(pin: &str) -> Res<String> {
    let refuse = |why: &str| {
        Failure::config(format!(
            "[meta] required_version {pin:?} {why}; the launcher selects an engine only by an exact pin. \
             Replace it with \"=X.Y.Z\" naming the release that should judge this repository"
        ))
    };
    let Some(rest) = pin.trim().strip_prefix('=') else {
        return Err(refuse("is not exact"));
    };
    let version = rest.trim();
    let (core, pre) = match version.split_once('-') {
        Some((c, p)) => (c, Some(p)),
        None => (version, None),
    };
    let parts: Vec<&str> = core.split('.').collect();
    let numeric = parts.len() == 3
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()));
    let pre_ok = pre.is_none_or(|p| {
        !p.is_empty()
            && p.bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-')
    });
    if !numeric || !pre_ok {
        return Err(refuse("is not a version of the form X.Y.Z"));
    }
    Ok(version.to_string())
}

#[derive(Deserialize, Default)]
struct RawLock {
    engine: Option<RawEngine>,
}

#[derive(Deserialize, Default)]
struct RawEngine {
    release: Option<String>,
    #[serde(default)]
    digests: BTreeMap<String, String>,
}

#[derive(Debug)]
pub struct Lock {
    /// Target triple to lowercase hex digest (without the `sha256:` prefix).
    pub digests: BTreeMap<String, String>,
}

/// The lock beside `spec-spine.toml`, when present. A lock whose `release`
/// differs from the pin is refused, naming both. `pin` is `None` only for
/// `launcher lock`, which is replacing the lock and so reads it without judging.
pub fn read_lock(root: &Path, pin: Option<&str>) -> Res<Option<Lock>> {
    let path = root.join(LOCK_FILE);
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(Failure::io(format!("cannot read {}: {e}", path.display()))),
    };
    let bad = |why: String| Failure::config(format!("{}: {why}", path.display()));
    let raw: RawLock = toml::from_str(&text).map_err(|e| bad(format!("not readable: {e}")))?;
    let engine = raw
        .engine
        .ok_or_else(|| bad("no [engine] table".to_string()))?;
    let release = engine
        .release
        .ok_or_else(|| bad("[engine] has no release".to_string()))?;
    if let Some(pin) = pin
        && release != pin
    {
        return Err(Failure::config(format!(
            "{} names release {release} but {CONFIG_FILE} pins {pin}; run `spec-spine launcher lock` to rewrite it",
            path.display()
        )));
    }
    let mut digests = BTreeMap::new();
    for (target, value) in engine.digests {
        let hex = value
            .strip_prefix("sha256:")
            .filter(|h| hash::is_hex_digest(h))
            .ok_or_else(|| {
                bad(format!(
                    "digest for {target} is not sha256:<64 lowercase hex digits>"
                ))
            })?;
        digests.insert(target, hex.to_string());
    }
    Ok(Some(Lock { digests }))
}

/// The text of a lock for `release`; keys are sorted so the file is stable.
pub fn render_lock(release: &str, digests: &BTreeMap<String, String>) -> String {
    let mut s = String::from(
        "# Written by `spec-spine launcher lock`. No engine reads this file.\n\
         # Digests are the SHA-256 of the engine executable for each target.\n\
         [engine]\n",
    );
    s.push_str(&format!("release = \"{release}\"\n"));
    s.push_str("[engine.digests]\n");
    for (target, hex) in digests {
        s.push_str(&format!("\"{target}\" = \"sha256:{hex}\"\n"));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_exact_pins_select() {
        assert_eq!(parse_exact("=0.29.0").unwrap(), "0.29.0");
        assert_eq!(parse_exact("= 0.29.0-rc.1").unwrap(), "0.29.0-rc.1");
        for bad in ["^0.29", ">=0.29.0", "0.29.0", "=0.29", "=0.29.x", "=", "*"] {
            assert_eq!(parse_exact(bad).unwrap_err().code, 2, "{bad}");
        }
    }

    #[test]
    fn the_reader_ignores_every_other_key() {
        let raw: RawConfig = toml::from_str(
            "[meta]\nrequired_version = \"=1.2.3\"\nfuture = 1\n[from_the_future]\nx = [1, 2]\n",
        )
        .unwrap();
        assert_eq!(raw.meta.unwrap().required_version.unwrap(), "=1.2.3");
    }

    #[test]
    fn rendered_lock_reads_back() {
        let dir = std::env::temp_dir().join(format!("ss-launcher-lock-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut d = BTreeMap::new();
        d.insert("x86_64-unknown-linux-gnu".to_string(), "a".repeat(64));
        let text = render_lock("0.29.0", &d);
        std::fs::write(dir.join(LOCK_FILE), text).unwrap();
        let lock = read_lock(&dir, Some("0.29.0")).unwrap().unwrap();
        assert_eq!(lock.digests, d);
        assert_eq!(read_lock(&dir, Some("0.30.0")).unwrap_err().code, 2);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
