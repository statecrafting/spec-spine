// Spec: specs/188-a-repository-pin-selects-the-engine/spec.md
//! V-1, acquisition: a download happens only under a policy the user or CI
//! set, is verified before anything is placed, and concurrent installs of one
//! artifact never write the same path. The release source is a local fixture
//! directory built with `tar`; no network is used.

mod common;

use std::fs;
use std::path::PathBuf;
use std::process::Stdio;

use common::*;

const RELEASE: &str = "0.29.0";

struct Fx {
    w: World,
    repo: PathBuf,
    base: PathBuf,
    engine: String,
    digest: String,
}

fn fx() -> Fx {
    let w = World::new();
    let repo = w.repo("r", &format!("={RELEASE}"));
    let base = w.path("releases");
    let engine = stub(RELEASE, "fixture");
    let digest = w.release_fixture(&base, RELEASE, &engine);
    Fx {
        w,
        repo,
        base,
        engine,
        digest,
    }
}

impl Fx {
    fn cmd(&self) -> std::process::Command {
        let mut c = self.w.cmd(&self.repo);
        c.env("SPEC_SPINE_RELEASE_BASE", &self.base);
        c
    }

    fn nothing_placed(&self) {
        assert!(
            self.w.store_entries(RELEASE).is_empty(),
            "installed: {:?}",
            self.w.store_names(RELEASE)
        );
        assert!(
            !self
                .w
                .store_names(RELEASE)
                .iter()
                .any(|n| n.starts_with(".tmp")),
            "left scratch behind: {:?}",
            self.w.store_names(RELEASE)
        );
    }
}

#[test]
fn policy_never_refuses_and_downloads_nothing() {
    let f = fx();
    for extra in [vec![], vec!["--acquire=never"]] {
        let o = f.cmd().args(&extra).arg("check").output().unwrap();
        assert_eq!(code(&o), 2, "{}", stderr(&o));
        let e = stderr(&o);
        assert!(
            e.contains("launcher install") && e.contains(RELEASE) && e.contains(host_target()),
            "{e}"
        );
        assert!(!stdout(&o).contains("engine"));
    }
    f.nothing_placed();
    assert!(
        !f.w.home.exists(),
        "the default policy touched {}",
        f.w.home.display()
    );
}

#[test]
fn policy_auto_installs_from_the_local_fixture_and_runs_the_engine() {
    let f = fx();
    let out = f.w.path("out.txt");
    let o = f
        .cmd()
        .env("STUB_OUT", &out)
        .args(["--acquire=auto", "check", "--json"])
        .output()
        .unwrap();
    assert_eq!(code(&o), 0, "{}", stderr(&o));
    assert_eq!(stdout(&o), format!("engine {RELEASE}\n"));
    // The launcher's own flag is not the engine's argument.
    assert_eq!(&read_kv(&out)[3..], ["arg=check", "arg=--json"]);

    assert_eq!(f.w.store_entries(RELEASE), vec![f.digest.clone()]);
    let exe = f.w.entry_dir(RELEASE, &f.digest).join("spec-spine");
    assert_eq!(fs::read_to_string(&exe).unwrap(), f.engine);
    assert!(
        !f.w.store_names(RELEASE)
            .iter()
            .any(|n| n.starts_with(".tmp"))
    );

    // The second invocation needs no source at all.
    let o =
        f.w.cmd(&f.repo)
            .env("SPEC_SPINE_RELEASE_BASE", f.w.path("gone"))
            .arg("check")
            .output()
            .unwrap();
    assert_eq!(code(&o), 0, "{}", stderr(&o));
}

#[test]
fn the_environment_and_a_file_url_source_work_and_the_flag_outranks_the_environment() {
    let f = fx();
    let url = format!("file://{}", f.base.display());
    let o =
        f.w.cmd(&f.repo)
            .env("SPEC_SPINE_RELEASE_BASE", &url)
            .env("SPEC_SPINE_ACQUIRE", "auto")
            .args(["--acquire", "never", "check"])
            .output()
            .unwrap();
    assert_eq!(code(&o), 2, "the flag outranks the environment");
    f.nothing_placed();

    let o =
        f.w.cmd(&f.repo)
            .env("SPEC_SPINE_RELEASE_BASE", &url)
            .env("SPEC_SPINE_ACQUIRE", "auto")
            .arg("check")
            .output()
            .unwrap();
    assert_eq!(code(&o), 0, "{}", stderr(&o));
    assert_eq!(f.w.store_entries(RELEASE), vec![f.digest.clone()]);
}

#[test]
fn the_users_configuration_can_enable_acquisition() {
    let f = fx();
    fs::create_dir_all(&f.w.home).unwrap();
    fs::write(
        f.w.home.join("launcher.toml"),
        "acquire = \"auto\"\nfuture_key = 1\n",
    )
    .unwrap();
    let o = f.cmd().arg("check").output().unwrap();
    assert_eq!(code(&o), 0, "{}", stderr(&o));
}

#[test]
fn frozen_beats_auto_from_every_source() {
    let f = fx();
    fs::create_dir_all(&f.w.home).unwrap();
    fs::write(f.w.home.join("launcher.toml"), "acquire = \"auto\"\n").unwrap();
    let o = f
        .cmd()
        .env("SPEC_SPINE_FROZEN", "1")
        .env("SPEC_SPINE_ACQUIRE", "auto")
        .args(["--acquire=auto", "check"])
        .output()
        .unwrap();
    assert_eq!(code(&o), 2, "{}", stderr(&o));
    assert!(stderr(&o).contains("launcher install"));
    f.nothing_placed();
}

#[test]
fn a_repository_file_cannot_enable_acquisition() {
    let w = World::new();
    let base = w.path("releases");
    let digest = w.release_fixture(&base, RELEASE, &stub(RELEASE, "fixture"));
    let r = w.repo_with(
        "r",
        &format!("={RELEASE}"),
        "acquire = \"auto\"\n[launcher]\nacquire = \"auto\"\n[acquire]\npolicy = \"auto\"\n",
    );
    w.write_lock(&r, RELEASE, &digest, "acquire = \"auto\"\n");
    fs::write(r.join("launcher.toml"), "acquire = \"auto\"\n").unwrap();
    let o = w
        .cmd(&r)
        .env("SPEC_SPINE_RELEASE_BASE", &base)
        .arg("check")
        .output()
        .unwrap();
    assert_eq!(code(&o), 2, "{}", stderr(&o));
    assert!(w.store_entries(RELEASE).is_empty());
    assert!(!w.home.exists());
}

#[test]
fn a_lock_digest_that_disagrees_with_the_archive_is_refused_naming_both() {
    let f = fx();
    let locked = "c".repeat(64);
    f.w.write_lock(&f.repo, RELEASE, &locked, "");
    let o = f.cmd().args(["--acquire=auto", "check"]).output().unwrap();
    assert_eq!(code(&o), 2, "{}", stderr(&o));
    let e = stderr(&o);
    assert!(e.contains(&locked), "{e}");
    assert!(e.contains(&f.digest), "{e}");
    assert!(e.contains("nothing was installed"), "{e}");
    f.nothing_placed();
    assert!(!stdout(&o).contains("engine"));

    // The explicit verb refuses the same way.
    let o = f.cmd().args(["launcher", "install"]).output().unwrap();
    assert_eq!(code(&o), 2);
    f.nothing_placed();
}

#[test]
fn an_archive_that_disagrees_with_its_published_digest_installs_nothing() {
    let f = fx();
    let name = format!("spec-spine-v{RELEASE}-{}.tar.gz.sha256", host_target());
    fs::write(
        f.base.join(format!("v{RELEASE}")).join(name),
        format!("{}  x\n", "d".repeat(64)),
    )
    .unwrap();
    let o = f.cmd().args(["launcher", "install"]).output().unwrap();
    assert_eq!(code(&o), 2, "{}", stderr(&o));
    assert!(stderr(&o).contains(&"d".repeat(64)));
    f.nothing_placed();
}

#[test]
fn an_unpublished_release_is_refused_naming_the_targets() {
    let f = fx();
    let r = f.w.repo("other", "=9.9.9");
    let o =
        f.w.cmd(&r)
            .env("SPEC_SPINE_RELEASE_BASE", &f.base)
            .args(["launcher", "install"])
            .output()
            .unwrap();
    assert_eq!(code(&o), 2, "{}", stderr(&o));
    assert!(stderr(&o).contains("supported targets"), "{}", stderr(&o));
}

#[test]
fn install_build_is_not_supported_yet() {
    let f = fx();
    let o = f
        .cmd()
        .args(["launcher", "install", "--build"])
        .output()
        .unwrap();
    assert_eq!(code(&o), 2);
    assert!(stderr(&o).contains("not supported"), "{}", stderr(&o));
}

#[test]
fn two_concurrent_installs_of_one_artifact_one_writes_and_the_other_uses_it() {
    let f = fx();
    let spawn = || {
        f.cmd()
            .args(["launcher", "install"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap()
    };
    let a = spawn();
    let b = spawn();
    let (a, b) = (a.wait_with_output().unwrap(), b.wait_with_output().unwrap());
    assert_eq!(code(&a), 0, "{}", stderr(&a));
    assert_eq!(code(&b), 0, "{}", stderr(&b));

    let lines = [stdout(&a), stdout(&b)];
    let placed = lines.iter().filter(|l| l.starts_with("installed ")).count();
    let present = lines.iter().filter(|l| l.starts_with("present ")).count();
    assert_eq!((placed, present), (1, 1), "{lines:?}");
    assert_eq!(f.w.store_entries(RELEASE), vec![f.digest.clone()]);
    assert_eq!(
        f.w.store_names(RELEASE),
        vec![f.digest.clone()],
        "no lock file or scratch directory is left behind"
    );
    let exe = f.w.entry_dir(RELEASE, &f.digest).join("spec-spine");
    assert_eq!(sha256_file(&exe), f.digest);
}

#[test]
fn lock_records_the_engine_digest_of_every_published_target() {
    let f = fx();
    fs::write(
        f.repo.join("spec-spine.lock"),
        "[engine]\nrelease = \"0.28.0\"\n[engine.digests]\n",
    )
    .unwrap();
    let o = f.cmd().args(["launcher", "lock"]).output().unwrap();
    assert_eq!(code(&o), 0, "{}", stderr(&o));
    let lock = fs::read_to_string(f.repo.join("spec-spine.lock")).unwrap();
    assert!(lock.contains(&format!("release = \"{RELEASE}\"")), "{lock}");
    assert!(!lock.contains("tool_dir"), "{lock}");
    for t in UNIX_TARGETS {
        assert!(
            lock.contains(&format!("\"{t}\" = \"sha256:{}\"", f.digest)),
            "{lock}"
        );
    }
    assert!(
        !lock.contains("windows"),
        "the fixture publishes no Windows archive"
    );
    // Locking is not installing.
    f.nothing_placed();

    // With the lock in place, install and resolve agree with it.
    let o = f.cmd().args(["launcher", "install"]).output().unwrap();
    assert_eq!(code(&o), 0, "{}", stderr(&o));
    let o = f
        .cmd()
        .args(["launcher", "resolve", "--json"])
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["report"]["lock"], "matched");
    assert_eq!(v["report"]["trust"], "lock");
    assert_eq!(v["report"]["digest"], format!("sha256:{}", f.digest));
}
