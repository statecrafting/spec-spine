//! The launcher's top level: the `launcher` namespace, and the one path every
//! other invocation takes (find, read, resolve, verify, execute).

use std::ffi::OsString;
use std::io::Write;
use std::path::PathBuf;

use serde_json::json;

use crate::acquire::{self, Installed, Policy};
use crate::cli::{self, Parsed};
use crate::engine::{self, Found, Req, Selection};
use crate::envelope;
use crate::exec;
use crate::failure::{Failure, Res};
use crate::paths;
use crate::repo::{self, Lock, Repo};
use crate::target;
use crate::{NAME, VERSION};

const HELP: &str = "\
spec-spine launcher: runs the engine release a repository pins.

Usage:
  spec-spine [--repo DIR] [--acquire never|auto] <engine arguments...>
  spec-spine launcher resolve [--json]   answer which engine would run
  spec-spine launcher install [--build]  prepare the pinned engine
  spec-spine launcher lock               write spec-spine.lock
  spec-spine launcher --version

Every other invocation is passed to the pinned engine unchanged.
Outside a repository only --version and --help are answered.
See docs/launcher.md.
";

pub fn run(args: Vec<OsString>) -> u8 {
    match run_inner(args) {
        Ok(code) => code,
        Err(f) => {
            eprintln!("{NAME}: {}", f.message);
            f.code
        }
    }
}

fn run_inner(args: Vec<OsString>) -> Res<u8> {
    let parsed = cli::parse(args)?;
    if let Some(words) = cli::launcher_words(&parsed.own_args) {
        return launcher_verb(&parsed, &words);
    }
    let cwd = cwd()?;
    let found = repo::discover(&cwd, parsed.repo_flag.as_deref(), env_repo());
    let repo = match found {
        Ok(r) => r,
        Err(f) => {
            return match cli::is_info(&parsed.own_args) {
                Some("help") => {
                    print!("{HELP}");
                    Ok(0)
                }
                Some(_) => {
                    println!("{NAME} {VERSION}");
                    Ok(0)
                }
                None => Err(f),
            };
        }
    };
    execute(&parsed, &cwd, repo)
}

fn cwd() -> Res<PathBuf> {
    std::env::current_dir()
        .map_err(|e| Failure::io(format!("cannot read the working directory: {e}")))
}

fn env_repo() -> Option<PathBuf> {
    std::env::var_os("SPEC_SPINE_REPO")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
}

fn env_engine() -> Option<PathBuf> {
    std::env::var_os("SPEC_SPINE_ENGINE")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
}

/// Everything resolution needs, read once.
struct Prepared {
    repo: Repo,
    release: String,
    target: String,
    lock: Option<Lock>,
    data_root: PathBuf,
}

impl Prepared {
    fn new(repo: Repo) -> Res<Prepared> {
        let release = repo::read_pin(&repo.root)?;
        let lock = repo::read_lock(&repo.root, Some(&release))?;
        Ok(Prepared {
            release,
            target: target::host()?,
            lock,
            data_root: paths::data_root()?,
            repo,
        })
    }

    fn req(&self, quarantine: bool) -> Req<'_> {
        Req {
            repo: &self.repo,
            release: &self.release,
            target: &self.target,
            lock: self.lock.as_ref(),
            data_root: &self.data_root,
            override_path: env_engine(),
            quarantine,
        }
    }
}

fn not_installed(p: &Prepared) -> String {
    format!(
        "engine {} for {} is not installed ({}); run `spec-spine launcher install`",
        p.release,
        p.target,
        if p.lock.is_some() {
            "spec-spine.lock binds its digest"
        } else {
            "no spec-spine.lock binds it"
        }
    )
}

fn execute(parsed: &Parsed, cwd: &std::path::Path, repo: Repo) -> Res<u8> {
    if let Some(v) = std::env::var_os("SPEC_SPINE_LAUNCHED") {
        return Err(Failure::internal(format!(
            "SPEC_SPINE_LAUNCHED is already set ({}); the launcher refuses to run from inside an engine it launched",
            v.to_string_lossy()
        )));
    }
    // The engine defaults its repository to the working directory. When the
    // launcher found the repository elsewhere (an ancestor, or
    // SPEC_SPINE_REPO), name it so the engine judges the repository the
    // launcher selected the engine for (D-13).
    let inject = !repo.named_by_flag && repo.root != cwd;
    let root = repo.root.clone();
    let p = Prepared::new(repo)?;
    let selection = match engine::resolve(&p.req(true))? {
        Found::Engine(s) => s,
        Found::Missing => match acquire::policy(parsed.acquire_flag.as_deref())? {
            Policy::Never => return Err(Failure::refused(not_installed(&p))),
            Policy::Auto => match acquire::install(&p.req(true))? {
                Installed::Placed(s) | Installed::Present(s) => s,
            },
        },
    };
    let mut args = parsed.engine_args.clone();
    if inject {
        args.insert(0, root.into_os_string());
        args.insert(0, OsString::from("--repo"));
    }
    exec::run(&selection.path, &args)
}

fn launcher_verb(parsed: &Parsed, words: &[String]) -> Res<u8> {
    let json = words.iter().any(|w| w == "--json");
    let rest: Vec<&str> = words
        .iter()
        .map(String::as_str)
        .filter(|w| *w != "--json")
        .collect();
    match rest.as_slice() {
        ["--version" | "-V"] => {
            println!("{NAME} {VERSION}");
            Ok(0)
        }
        ["--help" | "-h"] => {
            print!("{HELP}");
            Ok(0)
        }
        ["resolve"] => {
            let out = resolve_verb(parsed);
            resolve_output(out, json)
        }
        ["install"] if !json => install_verb(parsed),
        ["install", "--build"] if !json => Err(Failure::refused(
            "`launcher install --build` is not supported yet; install the published release instead",
        )),
        ["lock"] if !json => lock_verb(parsed),
        _ => {
            eprint!("{HELP}");
            Err(Failure::usage(format!(
                "unknown launcher invocation: launcher {}",
                words.join(" ")
            )))
        }
    }
}

fn discover(parsed: &Parsed) -> Res<Repo> {
    repo::discover(&cwd()?, parsed.repo_flag.as_deref(), env_repo())
}

struct Resolution {
    repo: PathBuf,
    release: String,
    target: String,
    lock: bool,
    selection: Selection,
}

/// Writes nothing and downloads nothing, whatever the policy.
fn resolve_verb(parsed: &Parsed) -> Res<Resolution> {
    let p = Prepared::new(discover(parsed)?)?;
    match engine::resolve(&p.req(false))? {
        Found::Engine(selection) => Ok(Resolution {
            repo: p.repo.root.clone(),
            release: p.release.clone(),
            target: p.target.clone(),
            lock: p.lock.is_some(),
            selection,
        }),
        Found::Missing => Err(Failure::not_found(not_installed(&p))),
    }
}

fn resolve_output(out: Res<Resolution>, json: bool) -> Res<u8> {
    if json {
        let (text, code) = match out {
            Ok(r) => {
                let report = json!({
                    "repo": r.repo.to_string_lossy(),
                    "release": r.release,
                    "target": r.target,
                    "path": r.selection.path.to_string_lossy(),
                    "digest": format!("sha256:{}", r.selection.digest),
                    "rule": r.selection.rule.token(),
                    "lock": if r.lock { "matched" } else { "absent" },
                    "trust": if r.lock { "lock" } else { "published-digest" },
                });
                let summary = format!(
                    "engine {} resolved by {}",
                    r.release,
                    r.selection.rule.token()
                );
                (
                    envelope::report(envelope::VERB_RESOLVE, &summary, report),
                    0,
                )
            }
            Err(f) => (envelope::failure(envelope::VERB_RESOLVE, &f), f.code),
        };
        print!("{text}");
        let _ = std::io::stdout().flush();
        return Ok(code);
    }
    let r = out?;
    println!("path: {}", r.selection.path.display());
    println!("release: {}", r.release);
    println!("target: {}", r.target);
    println!("digest: sha256:{}", r.selection.digest);
    println!("rule: {}", r.selection.rule.token());
    println!("lock: {}", if r.lock { "matched" } else { "absent" });
    println!(
        "trust: {}",
        if r.lock { "lock" } else { "published-digest" }
    );
    Ok(0)
}

fn install_verb(parsed: &Parsed) -> Res<u8> {
    let p = Prepared::new(discover(parsed)?)?;
    match acquire::install(&p.req(true))? {
        Installed::Placed(s) => println!("installed {}", s.path.display()),
        Installed::Present(s) => println!("present {}", s.path.display()),
    }
    Ok(0)
}

fn lock_verb(parsed: &Parsed) -> Res<u8> {
    let repo = discover(parsed)?;
    let release = repo::read_pin(&repo.root)?;
    let digests = acquire::compute_digests(&release)?;
    let text = repo::render_lock(&release, &digests);
    let dest = repo.root.join(repo::LOCK_FILE);
    let tmp = repo
        .root
        .join(format!("{}.tmp-{}", repo::LOCK_FILE, std::process::id()));
    std::fs::write(&tmp, text)
        .and_then(|()| std::fs::rename(&tmp, &dest))
        .map_err(|e| {
            let _ = std::fs::remove_file(&tmp);
            Failure::io(format!("cannot write {}: {e}", dest.display()))
        })?;
    println!(
        "wrote {} (release {release}, {} targets)",
        dest.display(),
        digests.len()
    );
    Ok(0)
}
