//! The release targets, the archive names and the host's own target.
//!
//! The archive names are what `.github/workflows/release.yml` produces:
//! `spec-spine-v<release>-<triple>.tar.gz` (`.zip` on Windows), each with a
//! `<archive>.sha256` sidecar, all under `.../releases/download/v<release>/`.

use crate::failure::{Failure, Res};

/// The five targets a release publishes.
pub const TARGETS: [&str; 5] = [
    "aarch64-apple-darwin",
    "aarch64-unknown-linux-gnu",
    "x86_64-apple-darwin",
    "x86_64-pc-windows-msvc",
    "x86_64-unknown-linux-gnu",
];

/// The host's target triple, or a refusal naming the supported ones.
pub fn host() -> Res<String> {
    let arch = std::env::consts::ARCH;
    let triple = match (std::env::consts::OS, arch, cfg!(target_env = "musl")) {
        ("linux", "x86_64", false) => Some("x86_64-unknown-linux-gnu"),
        ("linux", "aarch64", false) => Some("aarch64-unknown-linux-gnu"),
        ("macos", "x86_64", _) => Some("x86_64-apple-darwin"),
        ("macos", "aarch64", _) => Some("aarch64-apple-darwin"),
        ("windows", "x86_64", _) => Some("x86_64-pc-windows-msvc"),
        _ => None,
    };
    triple.map(str::to_string).ok_or_else(|| {
        Failure::refused(format!(
            "unsupported target {arch}-{}; supported targets: {}",
            std::env::consts::OS,
            TARGETS.join(", ")
        ))
    })
}

pub fn is_windows(target: &str) -> bool {
    target.contains("windows")
}

pub fn archive_name(release: &str, target: &str) -> String {
    let ext = if is_windows(target) { "zip" } else { "tar.gz" };
    format!("spec-spine-v{release}-{target}.{ext}")
}

/// The engine executable's file name inside an archive and in the store.
pub fn exe_name(target: &str) -> &'static str {
    if is_windows(target) {
        "spec-spine.exe"
    } else {
        "spec-spine"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_match_the_release_workflow() {
        assert_eq!(
            archive_name("0.29.0", "x86_64-unknown-linux-gnu"),
            "spec-spine-v0.29.0-x86_64-unknown-linux-gnu.tar.gz"
        );
        assert_eq!(
            archive_name("0.29.0", "x86_64-pc-windows-msvc"),
            "spec-spine-v0.29.0-x86_64-pc-windows-msvc.zip"
        );
    }
}
