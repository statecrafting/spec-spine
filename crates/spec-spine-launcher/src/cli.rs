//! Argument handling. The launcher owns two flags, `--repo` (which it also
//! passes on, the engine having the same global) and `--acquire`, and one verb
//! namespace, `launcher`. Everything else belongs to the engine.

use std::ffi::OsString;
use std::path::PathBuf;

use crate::failure::{Failure, Res};

pub struct Parsed {
    pub repo_flag: Option<PathBuf>,
    pub acquire_flag: Option<String>,
    /// The arguments as given, minus `--acquire`. Handed to the engine.
    pub engine_args: Vec<OsString>,
    /// The arguments minus `--repo` and `--acquire`. What the launcher reads.
    pub own_args: Vec<OsString>,
}

pub fn parse(args: Vec<OsString>) -> Res<Parsed> {
    let mut p = Parsed {
        repo_flag: None,
        acquire_flag: None,
        engine_args: Vec::new(),
        own_args: Vec::new(),
    };
    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        if a == "--" {
            p.engine_args.push(a.clone());
            p.own_args.push(a);
            for rest in it.by_ref() {
                p.engine_args.push(rest.clone());
                p.own_args.push(rest);
            }
            break;
        }
        let text = a.to_string_lossy().into_owned();
        if text == "--repo" {
            let v = it
                .next()
                .ok_or_else(|| Failure::usage("--repo needs a directory"))?;
            p.repo_flag = Some(PathBuf::from(&v));
            p.engine_args.push(a);
            p.engine_args.push(v);
        } else if let Some(v) = text.strip_prefix("--repo=") {
            p.repo_flag = Some(PathBuf::from(v));
            p.engine_args.push(a);
        } else if text == "--acquire" {
            let v = it
                .next()
                .ok_or_else(|| Failure::usage("--acquire takes `never` or `auto`"))?;
            p.acquire_flag = Some(v.to_string_lossy().into_owned());
        } else if let Some(v) = text.strip_prefix("--acquire=") {
            p.acquire_flag = Some(v.to_string());
        } else {
            p.engine_args.push(a.clone());
            p.own_args.push(a);
        }
    }
    Ok(p)
}

/// The words after the `launcher` namespace, when the first positional is it.
pub fn launcher_words(own_args: &[OsString]) -> Option<Vec<String>> {
    let first = own_args
        .iter()
        .position(|a| !a.to_string_lossy().starts_with('-'))?;
    if own_args[first] != "launcher" {
        return None;
    }
    Some(
        own_args[first + 1..]
            .iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect(),
    )
}

/// Whether the invocation is the launcher's own `--version` or `--help`.
pub fn is_info(own_args: &[OsString]) -> Option<&'static str> {
    match own_args.first().and_then(|a| a.to_str()) {
        Some("--version" | "-V") => Some("version"),
        Some("--help" | "-h") => Some("help"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn os(v: &[&str]) -> Vec<OsString> {
        v.iter().map(OsString::from).collect()
    }

    #[test]
    fn acquire_is_the_launchers_and_repo_is_shared() {
        let p = parse(os(&["--acquire=auto", "--repo", "/r", "check", "--json"])).unwrap();
        assert_eq!(p.acquire_flag.as_deref(), Some("auto"));
        assert_eq!(p.repo_flag.as_deref(), Some(std::path::Path::new("/r")));
        assert_eq!(p.engine_args, os(&["--repo", "/r", "check", "--json"]));
        assert_eq!(p.own_args, os(&["check", "--json"]));
    }

    #[test]
    fn the_launcher_namespace_is_the_first_positional_only() {
        let a = os(&["launcher", "resolve", "--json"]);
        assert_eq!(launcher_words(&a).unwrap(), ["resolve", "--json"]);
        assert!(launcher_words(&os(&["check", "launcher"])).is_none());
        assert!(launcher_words(&os(&["--version"])).is_none());
    }
}
