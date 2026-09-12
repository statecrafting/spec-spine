//! Kit gate tests (spec 064): the kit ships a composite gate and a merge
//! driver, and this repository runs what it ships.
//!
//! The kit's session harness was exercised here from spec 046 onward, and its
//! build half was not, because there was no build half: every adopter wrote
//! the same Makefile and the same `has_cargo` workflow by hand. These pin the
//! artifacts that replace that, and hold them to the same two rules the hooks
//! and skills are held to: a gate never writes, and the chain has one
//! definition.

use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn read(rel: &str) -> String {
    let p = repo_root().join(rel);
    fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

/// Every `spec-spine <verb> ...` invocation in a file, with `$(SPEC_SPINE)`
/// and a bare `spec-spine` both recognised. Comment lines are skipped: the
/// kit's files explain themselves at length, and prose naming a verb is not an
/// invocation of it.
fn invocations(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in body.lines() {
        let t = line.trim();
        if t.starts_with('#') || t.starts_with("##") {
            continue;
        }
        let expanded = t.replace("$(SPEC_SPINE)", "spec-spine");
        for (i, _) in expanded.match_indices("spec-spine ") {
            // Skip a mention inside a word (`spec-spine-cli`) or a URL.
            if i > 0 {
                let prev = expanded.as_bytes()[i - 1];
                if !prev.is_ascii_whitespace() && prev != b'"' && prev != b'\'' {
                    continue;
                }
            }
            let after = &expanded[i + "spec-spine ".len()..];
            let cmd: String = after
                .split(['|', '>', ';', '&'])
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            if !cmd.is_empty() {
                out.push(cmd);
            }
        }
    }
    out
}

/// The verbs this binary has. A kit file naming anything else ships broken.
const VERBS: &[&str] = &[
    // Spec 075: the composed freshness read the protocol and the gate call.
    "check",
    "compile",
    "index",
    "registry",
    "lint",
    "couple",
    "verify",
    "init",
    "attest",
    "verify-attestation",
    "config",
];

/// Read-only forms, by the same rule `tests/kit_hooks.rs` applies to the hooks
/// (spec 046): `compile` reads only with `--check`, `index` reads only as a
/// named read action.
fn is_read_only(cmd: &str) -> bool {
    let mut w = cmd.split_whitespace();
    match w.next() {
        Some("compile") => cmd.contains("--check"),
        Some("index") => matches!(
            w.next(),
            Some("check")
                | Some("coverage")
                | Some("orphans")
                | Some("diagnostics")
                | Some("render")
                | Some("owner")
        ),
        // Spec 075 3.2: `check` carries the never-writes contract of the two
        // primitives it composes, which is what lets the gate call it.
        Some("check") | Some("couple") | Some("registry") | Some("lint") | Some("config") => true,
        _ => false,
    }
}

/// §3.4: every invocation names a verb this binary has. A kit that shipped a
/// renamed verb is exactly the drift spec 048 found in the skills.
#[test]
fn every_kit_gate_invocation_names_a_real_verb() {
    for rel in ["kit/Makefile", "kit/govern.yml"] {
        for cmd in invocations(&read(rel)) {
            let head = cmd.split_whitespace().next().unwrap_or("");
            assert!(
                VERBS.contains(&head),
                "{rel} invokes `spec-spine {cmd}`, and `{head}` is not a verb"
            );
        }
    }
}

/// §3.1 + §3.4: the gate path never writes. A gate that writes repairs what it
/// is meant to judge, which is the rule spec 046 established for the hooks and
/// which the build half must hold to as well.
#[test]
fn the_gate_target_is_read_only() {
    let makefile = read("kit/Makefile");
    let gate = target_body(&makefile, "gate");
    for cmd in invocations(&gate) {
        assert!(
            is_read_only(&cmd),
            "the `gate` target runs a writing verb: spec-spine {cmd}"
        );
    }
    // And the writing half is a separate target, so the two cannot be confused.
    let refresh = target_body(&makefile, "refresh");
    assert!(
        invocations(&refresh).iter().any(|c| c == "compile"),
        "`refresh` is where the writing lives: {refresh}"
    );
}

/// The recipe lines of one Makefile target.
fn target_body(makefile: &str, target: &str) -> String {
    let start = makefile
        .find(&format!("\n{target}:"))
        .unwrap_or_else(|| panic!("no `{target}` target"));
    let rest = &makefile[start + 1..];
    rest.lines()
        .skip(1)
        .take_while(|l| l.starts_with('\t') || l.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// §3.4: the chain in `kit/Makefile` is the chain `AGENTS.md` lists, in order.
/// The same assertion spec 051 made for the skills: a step CI enforces and the
/// gate omits is a step every adopter skips.
#[test]
fn the_kit_gate_chain_follows_agents_md() {
    let listed: Vec<String> = agents_md_gate_commands();
    let gate = invocations(&target_body(&read("kit/Makefile"), "gate"));

    // `compile --check` stands for `compile`: the kit's gate is read-only and
    // CI substitutes the same way, because a gate must never repair the tree it
    // is judging. The writing `index` has the same standing and lives in
    // `refresh`.
    let normalize = |c: &str| c.replace(" --check", "").trim().to_string();
    let listed_heads: Vec<String> = listed.iter().map(|c| normalize(c)).collect();

    let mut cursor = 0usize;
    for cmd in &gate {
        let n = normalize(cmd);
        let found = listed_heads[cursor..]
            .iter()
            .position(|l| l.starts_with(n.split(" --base").next().unwrap_or(&n)));
        let Some(pos) = found else {
            panic!(
                "`{cmd}` is not in AGENTS.md's gate list at or after position {cursor}: {listed_heads:?}"
            );
        };
        cursor += pos + 1;
    }
    assert!(!gate.is_empty(), "the gate target must invoke spec-spine");
}

fn agents_md_gate_commands() -> Vec<String> {
    let text = read("AGENTS.md");
    let start = text
        .find("Run the gate before every commit")
        .expect("AGENTS.md names the gate step");
    let tail = &text[start..];
    let open = tail
        .find("```sh")
        .expect("the gate list is a fenced sh block");
    let body = &tail[open + "```sh".len()..];
    let close = body.find("```").expect("the fence closes");
    body[..close]
        .lines()
        .filter_map(|l| l.trim().strip_prefix("spec-spine "))
        .map(|c| c.trim().to_string())
        .collect()
}

/// §3.1: the language targets are guarded on a MANIFEST probe, not a command
/// probe. A tree with cargo installed and no `Cargo.toml` is the specify-first
/// case, which is three of the four governed repositories.
#[test]
fn language_targets_probe_for_a_manifest_not_a_tool() {
    let makefile = read("kit/Makefile");
    for target in ["test", "build", "fmt", "clippy"] {
        let body = target_body(&makefile, target);
        assert!(
            body.contains("test -f Cargo.toml") || body.contains("test -f package.json"),
            "`{target}` must be guarded on a manifest probe: {body}"
        );
        assert!(
            !body.contains("command -v cargo"),
            "`{target}` probes for the tool, which answers the wrong question"
        );
    }
}

/// §3.2: the workflow carries the finding, not only the pattern. A workflow
/// with the shape and no reason gets "simplified" back into the broken form by
/// the next reader.
#[test]
fn the_workflow_records_why_the_probe_is_a_job() {
    let wf = read("kit/govern.yml");
    assert!(wf.contains("has_cargo"), "the probe pattern");
    assert!(wf.contains("GITHUB_OUTPUT"), "published as a job output");
    assert!(
        wf.contains("hashFiles"),
        "the finding: GitHub rejects hashFiles in a job-level if"
    );
    // §3.2: the PR body reaches the gate through a file, not through shell
    // quoting, because a body carrying a waiver line has no safe quoting.
    assert!(wf.contains("--pr-body"), "{wf}");
    assert!(wf.contains("RUNNER_TEMP"), "{wf}");
}

/// §3.3: the kit's copy is the source, and this repository's `.githooks/` is
/// asserted equal to it. Two hand-maintained copies of a script that
/// regenerates committed artifacts is how they diverge silently.
#[test]
fn the_kit_githooks_and_this_repositorys_agree() {
    for name in ["enable-merge-driver.sh", "merge-derived-index.sh"] {
        assert_eq!(
            read(&format!("kit/.githooks/{name}")),
            read(&format!(".githooks/{name}")),
            "kit/.githooks/{name} and .githooks/{name} have diverged"
        );
    }
}

/// §3.3: the README answers rahi's question. An adopter running one spec per PR
/// with committed shards should be able to read it and know whether they need
/// the driver.
#[test]
fn the_readme_explains_the_merge_driver() {
    let readme = read("kit/README.md");
    assert!(readme.contains("merge driver"), "named at all");
    assert!(readme.contains("opt-in per clone"), "{readme}");
    assert!(
        readme.contains("disjoint shard files"),
        "sharding removes the common conflict"
    );
    assert!(
        readme.contains("never replaces the staleness gate")
            || readme.contains("never replaces the"),
        "the driver does not stand in for `index check`"
    );
}

/// §3.3: the stanza registers the driver on the shard globs this repository
/// actually commits, so an adopter copying it is registering it on the right
/// files.
#[test]
fn the_gitattributes_stanza_matches_the_committed_shard_globs() {
    let stanza = read("kit/.gitattributes-stanza");
    let ours = read(".gitattributes");
    for glob in stanza
        .lines()
        .filter(|l| !l.trim_start().starts_with('#') && l.contains("merge="))
    {
        assert!(
            ours.contains(glob.trim()),
            "the kit registers `{}` and this repository does not",
            glob.trim()
        );
    }
}

/// The guarded recipe lines of one Makefile target, `@` stripped, in order.
/// These are shell commands: make runs each recipe line in its own shell, so
/// the line's exit status is the target's verdict for that step.
fn guarded_lines(makefile: &str, target: &str) -> Vec<String> {
    target_body(makefile, target)
        .lines()
        .map(|l| l.trim_start().trim_start_matches('@').to_string())
        .filter(|l| l.contains("test -f "))
        .collect()
}

/// Rewrite a guarded line so its command is `false`, leaving the probe and the
/// else-branch exactly as shipped. Returns `None` for a line that is not an
/// if/else, which the test above already refuses.
fn with_failing_command(line: &str) -> Option<String> {
    let (probe, rest) = line.split_once("; then ")?;
    let (_command, tail) = rest.split_once("; else ")?;
    Some(format!("{probe}; then false; else {tail}"))
}

/// §3.1 (spec 089): a guarded target distinguishes "the manifest is absent"
/// from "the command failed". `test -f M && cmd || echo skipping` does not:
/// `||` fires for either, so a failing command exits 0 printing a false skip.
/// Static half, so the shape is refused at review time and not only at run
/// time.
#[test]
fn a_guarded_target_does_not_conflate_a_skip_with_a_failure() {
    let makefile = read("kit/Makefile");
    for target in ["test", "build", "fmt", "clippy"] {
        let lines = guarded_lines(&makefile, target);
        assert!(!lines.is_empty(), "`{target}` has no guarded line");
        for line in lines {
            assert!(
                !(line.contains("&&") && line.contains("|| echo")),
                "`{target}` guards with `&& ... || echo`, so a failing command \
                 is reported as a skip: {line}"
            );
            assert!(
                line.starts_with("if test -f ")
                    && line.contains("; then ")
                    && line.contains("; else ")
                    && line.ends_with("fi"),
                "`{target}` must guard with an explicit if/else: {line}"
            );
        }
    }
}

/// §3.2 (spec 089): the shipped recipe lines are RUN, in both states, so the
/// two answers are proved rather than asserted about the text.
///
/// This is the acceptance the defect got past. Spec 064's
/// `language_targets_probe_for_a_manifest_not_a_tool` passes for the broken
/// shape and the fixed one alike, because both contain `test -f Cargo.toml`;
/// an acceptance that never forces a command to fail cannot tell them apart.
#[test]
fn a_guarded_recipe_skips_when_absent_and_fails_when_the_command_fails() {
    let makefile = read("kit/Makefile");
    let run = |script: &str, dir: &Path| {
        std::process::Command::new("sh")
            .arg("-c")
            .arg(script)
            .current_dir(dir)
            .output()
            .expect("sh is available")
    };

    for target in ["test", "build", "fmt", "clippy"] {
        for line in guarded_lines(&makefile, target) {
            // The manifest is absent: the command never runs, the line exits 0
            // and says so. This is spec 064 3.1's no-op, still true.
            let empty = tempfile::tempdir().unwrap();
            let out = run(&line, empty.path());
            assert!(
                out.status.success(),
                "`{target}` must be a clean no-op with no manifest: {line}"
            );
            assert!(
                String::from_utf8_lossy(&out.stdout).contains("skipping"),
                "`{target}` must say it skipped: {line}"
            );

            // The manifest is present and the command fails: the line must
            // fail. Under the old shape this printed "skipping" and exited 0.
            let present = tempfile::tempdir().unwrap();
            fs::write(present.path().join("Cargo.toml"), "").unwrap();
            fs::write(present.path().join("package.json"), "{}").unwrap();
            let forced = with_failing_command(&line)
                .unwrap_or_else(|| panic!("`{target}` line is not an if/else: {line}"));
            let out = run(&forced, present.path());
            assert!(
                !out.status.success(),
                "`{target}` reported a failing command as success: {forced}"
            );
            assert!(
                !String::from_utf8_lossy(&out.stdout).contains("skipping"),
                "`{target}` called a failure a skip: {forced}"
            );
        }
    }
}
