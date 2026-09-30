//! Where the launcher keeps its per-user state (spec 188 D-7).
//!
//! `SPEC_SPINE_HOME` overrides both roots. Otherwise the platform's per-user
//! directories are computed by hand, under a `spec-spine` subdirectory, so no
//! directories crate is needed.

use std::ffi::OsString;
use std::path::PathBuf;

use crate::failure::{Failure, Res};

fn var(name: &str) -> Option<OsString> {
    std::env::var_os(name).filter(|v| !v.is_empty())
}

fn absolute_var(name: &str) -> Option<PathBuf> {
    var(name).map(PathBuf::from).filter(|p| p.is_absolute())
}

fn need(name: &str) -> Res<PathBuf> {
    absolute_var(name).ok_or_else(|| {
        Failure::refused(format!(
            "cannot place per-user state: {name} is unset or not absolute; set SPEC_SPINE_HOME to choose a directory"
        ))
    })
}

/// The directory holding `engines/`.
pub fn data_root() -> Res<PathBuf> {
    if let Some(h) = var("SPEC_SPINE_HOME") {
        return Ok(PathBuf::from(h));
    }
    let base = if cfg!(windows) {
        need("LOCALAPPDATA")?
    } else if cfg!(target_os = "macos") {
        need("HOME")?.join("Library").join("Application Support")
    } else if let Some(x) = absolute_var("XDG_DATA_HOME") {
        x
    } else {
        need("HOME")?.join(".local").join("share")
    };
    Ok(base.join("spec-spine"))
}

/// The user's launcher configuration file (it may not exist).
pub fn config_file() -> Res<PathBuf> {
    if let Some(h) = var("SPEC_SPINE_HOME") {
        return Ok(PathBuf::from(h).join("launcher.toml"));
    }
    let base = if cfg!(windows) {
        need("APPDATA")?
    } else if cfg!(target_os = "macos") {
        need("HOME")?.join("Library").join("Application Support")
    } else if let Some(x) = absolute_var("XDG_CONFIG_HOME") {
        x
    } else {
        need("HOME")?.join(".config")
    };
    Ok(base.join("spec-spine").join("launcher.toml"))
}
