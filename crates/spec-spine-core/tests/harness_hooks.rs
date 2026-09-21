// Spec: specs/046-kit-hooks-read-never-write/spec.md
//! Harness hook tests (spec 046, retargeted by spec 120 3.5): the Claude Code
//! hooks THIS repository runs observe the tree, they do not repair it. Three of
//! the four originally ran a writing subcommand (`compile`, `index`) from a
//! context that could not commit the result, and one of those writes stalled an
//! adopter's orchestrator for eleven hours on a tree it had dirtied itself.
//! These tests read `.claude/settings.json` and refuse a mutating `spec-spine`
//! invocation in any hook other than the one sanctioned write.
//!
//! Until spec 120 the subject was `kit/settings.json`, the copy the repository
//! distributed, and a separate assertion held this repository's copy equal to
//! it. The kit is gone and the distribution is Statecraft's; what is left is the
//! file a session here actually loads, which is the one these assertions now
//! read. Every behavioral assertion is the one that was there, because the two
//! files' hook bodies were asserted byte-equal up to the moment one of them was
//! removed.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

const SETTINGS: &str = include_str!("../../../.claude/settings.json");

/// Every hook command body in the kit, keyed by event name.
fn hook_bodies() -> BTreeMap<String, Vec<String>> {
    let v: serde_json::Value =
        serde_json::from_str(SETTINGS).expect(".claude/settings.json parses");
    let hooks = v["hooks"].as_object().expect("hooks object");
    hooks
        .iter()
        .map(|(event, matchers)| {
            let bodies = matchers
                .as_array()
                .expect("matcher list")
                .iter()
                .flat_map(|m| m["hooks"].as_array().expect("hook list").iter())
                .map(|h| h["command"].as_str().expect("command string").to_string())
                .collect();
            (event.clone(), bodies)
        })
        .collect()
}

/// Every `spec-spine ...` invocation in a hook body, as the verb words that
/// follow the binary (after an optional `--repo <dir>`), up to the first shell
/// metacharacter. `"$sc"` is the kit's alias for the binary and is expanded.
fn spec_spine_invocations(body: &str) -> Vec<Vec<String>> {
    let expanded = body.replace("\"$sc\"", "spec-spine");
    let mut out = Vec::new();
    for line in expanded.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            continue;
        }
        let mut rest = line;
        while let Some(pos) = rest.find("spec-spine") {
            let after = &rest[pos + "spec-spine".len()..];
            // Skip the `sc=spec-spine` assignment and `command -v` probes, and
            // any prose mention (the binary name inside an echo string).
            let before = &rest[..pos];
            let is_probe = before.trim_end().ends_with("sc=")
                || before.ends_with('\'')
                || before.ends_with('"')
                || before.contains("command -v")
                || before.contains("echo")
                || before.contains("run ");
            if !is_probe && after.starts_with(' ') {
                let words: Vec<String> = after
                    .split([';', '|', '&', ')', '>', '<', '\n', '\''])
                    .next()
                    .unwrap_or("")
                    .split_whitespace()
                    .map(str::to_string)
                    .collect();
                // This scanner rewrites the literal `"$sc"` to `spec-spine`
                // above, so the guard `[ -n "$sc" ]` reads here as
                // `[ -n spec-spine ]`. At runtime `$sc` holds a resolved path,
                // and either way the line is a shell test, not a call. A real
                // invocation's first word is a subcommand or a flag (051 3.5).
                let is_call = words.first().is_some_and(|w| {
                    w.starts_with('-') || w.starts_with(|c: char| c.is_ascii_alphabetic())
                });
                let mut words = words.into_iter().peekable();
                if words.peek().map(String::as_str) == Some("--repo") {
                    words.next();
                    words.next();
                }
                let stripped: Vec<String> = words.collect();
                // `"$sc" --repo "$root"` with no subcommand strips to nothing.
                // An empty vec is not an invocation, and recording one would
                // hand every downstream assertion a verb with no first word.
                if is_call && !stripped.is_empty() {
                    out.push(stripped);
                }
            }
            rest = after;
        }
    }
    out
}

/// Whether an invocation only reads: `compile --check`, `index check`, and
/// `couple` are the read verbs the hooks need. `compile` and `index` without
/// their check flag write committed shards.
fn is_read_only(verb: &[String]) -> bool {
    match verb.first().map(String::as_str) {
        Some("compile") => verb.iter().any(|w| w == "--check"),
        Some("index") => verb.get(1).map(String::as_str) == Some("check"),
        // Spec 075 3.2: the composed freshness verb carries the never-writes
        // contract of both primitives it calls, which is precisely why a hook
        // may run it. A `check` that could repair the tree would make a stale
        // committed ledger invisible on the branch that carries it.
        Some("check") | Some("couple") => true,
        // Spec 090 3.2: the commit-boundary hook asks the tool where the
        // derived directory is rather than hardcoding `.derived`. `config`
        // has one subcommand and it prints.
        Some("config") => true,
        // Spec 063 §3.2: the hooks ask `--version` before believing an exit
        // code. Named explicitly rather than folded into a "flags are safe"
        // rule, because this predicate denies by default on purpose and the
        // exemptions should be countable.
        Some("--version" | "--help") => true,
        Some(_) => false,
        None => false,
    }
}

#[test]
fn kit_ships_all_four_hook_events() {
    let bodies = hook_bodies();
    for event in ["SessionStart", "PostToolUse", "PreToolUse", "Stop"] {
        assert!(
            bodies.get(event).is_some_and(|b| !b.is_empty()),
            ".claude/settings.json must carry a {event} hook"
        );
    }
}

#[test]
fn every_hook_invokes_spec_spine_at_least_once() {
    // The parser is only meaningful if it sees invocations; a hook that the
    // scanner reads as spec-spine-free would pass the write test vacuously.
    for (event, bodies) in hook_bodies() {
        let n: usize = bodies.iter().map(|b| spec_spine_invocations(b).len()).sum();
        assert!(n > 0, "{event} hook body has no recognised spec-spine call");
    }
}

/// Spec 046 3.1: no hook writes into the tree it observes, except the live
/// session's recompile after a spec edit (3.2), which is the one place the
/// actor can commit what it wrote.
#[test]
fn hooks_read_and_never_write() {
    for (event, bodies) in hook_bodies() {
        for body in bodies {
            for verb in spec_spine_invocations(&body) {
                let sanctioned = event == "PostToolUse" && verb.as_slice() == ["compile"];
                assert!(
                    sanctioned || is_read_only(&verb),
                    "{event} hook runs a writing spec-spine subcommand: spec-spine {}",
                    verb.join(" ")
                );
            }
        }
    }
}

/// Spec 046 3.3: the PR gate and the post-edit check act on the repository the
/// action targets, not on the session's project directory.
#[test]
fn action_hooks_are_scoped_to_the_target_repo() {
    let bodies = hook_bodies();
    for event in ["PostToolUse", "PreToolUse"] {
        let body = bodies[event].join("\n");
        assert!(
            body.contains("rev-parse --show-toplevel"),
            "{event} must derive the repo root from the action, not CLAUDE_PROJECT_DIR"
        );
        assert!(
            body.contains("--repo \"$root\""),
            "{event} must pass the derived root to spec-spine via --repo"
        );
        assert!(
            body.contains("\"$root/specs\""),
            "{event} must no-op when the target is not a spec-spine corpus"
        );
        assert!(
            !body.contains("CLAUDE_PROJECT_DIR"),
            "{event} must not bind to the session project"
        );
    }
}

/// Spec 046 3.4, as spec 072 3.2 amends it: pushing to the default branch is
/// refused, by branch and by refspec, before the PR gate runs. The name is now
/// resolved rather than spelled, so this guard reads the resolved variable in
/// each of the four sites instead of the literal `main` 046 named.
#[test]
fn pre_tool_use_refuses_a_push_to_the_default_branch() {
    let body = hook_bodies()["PreToolUse"].join("\n");
    assert!(body.contains("git push"), "no push gate");
    // Spec 072 3.1: the three resolution steps, in order.
    let env = body
        .find("SPEC_SPINE_DEFAULT_BRANCH")
        .expect("resolver must honour $SPEC_SPINE_DEFAULT_BRANCH");
    let remote = body
        .find("refs/remotes/origin/HEAD")
        .expect("resolver must consult the remote's own HEAD");
    assert!(env < remote, "the override must be read before the remote");
    // 3.2: every refspec form is BUILT from the resolved name, and quoted, so
    // a `$SPEC_SPINE_DEFAULT_BRANCH` carrying a glob character cannot widen
    // the pattern. An unquoted `$def` here would match literally today and
    // silently become a wildcard for the one adopter who sets a bad value.
    for form in [r#"*"origin $def""#, r#"*"HEAD:$def""#, r#"*"origin +$def""#] {
        assert!(
            body.contains(form),
            "push gate does not build the `{form}` refspec form from the resolved name"
        );
    }
    assert!(
        body.contains("if [ \"$br\" = \"$def\" ]; then"),
        "push gate does not refuse by current branch against the resolved name"
    );
    assert!(
        body.contains(r#"[ "$last" = "$def" ]"#),
        "push gate does not test the trailing argument against the resolved name"
    );
    // The literal comparisons 072 replaced must be gone, not merely joined:
    // one left behind would keep `main` privileged on a repository that has
    // no such branch, which is the defect rather than a leftover.
    for stale in [r#""$br" = main"#, r#""$last" = main"#] {
        assert!(
            !body.contains(stale),
            "push gate still compares against the literal name: {stale}"
        );
    }
    assert!(
        body.contains("update $def"),
        "the refusal message must name the branch it resolved (072 3.2)"
    );
    // Spec 071 3.1: anchored on the command that invokes the push verb. A
    // substring match over the whole command refused any command merely
    // containing the text, this test file among them.
    assert!(
        body.contains("case \"$cmd\" in 'git push'*|*'&& git push'*|*'; git push'*)"),
        "push gate outer match is not anchored on the push verb"
    );
}

/// Spec 071 3.3: the push gate is asserted by **running** it.
///
/// Spec 046 gave this gate a substring assertion, and a substring assertion
/// passes identically on the over-broad body and the corrected one: it cannot
/// see behavior, only text. That is why an over-broad match shipped to every
/// adopter and survived four releases, and it is why these cases execute the
/// shipped body against a synthesized hook payload and read its exit code.
/// Exit 2 is a refusal; exit 0 is a pass.
#[test]
fn the_push_gate_refuses_only_what_would_update_main() {
    if std::process::Command::new("jq")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_err()
    {
        // Spec 046 3.5: a check that cannot run says so rather than passing
        // quietly. CI runs this job on ubuntu-latest, where jq is present.
        eprintln!("SKIPPED the_push_gate_refuses_only_what_would_update_main: jq absent");
        return;
    }

    // (command, current branch, must be refused)
    let cases: &[(&str, &str, bool)] = &[
        // Would update main: the gate's whole purpose.
        ("git push", "main", true),
        ("git push origin", "main", true),
        ("git push origin main", "main", true),
        ("git push origin HEAD", "main", true),
        ("git push --force origin main", "feat", true),
        ("git push origin HEAD:main", "feat", true),
        ("git push origin +main", "feat", true),
        ("git push origin main:main", "feat", true),
        // Updates no branch. The first is the tag push docs/releasing.md
        // tells a maintainer to run from main once the release PR merges.
        ("git push origin v1.2.3", "main", false),
        ("cd . && git push origin v1.2.3", "main", false),
        // The arguments walked are the ones after the ANCHORED invocation, not
        // after the first mention of the verb anywhere in the string. Reading
        // the echo's words as a refspec used to refuse this tag push.
        (
            "echo \"mentions git push\" && git push origin v1.2.3",
            "main",
            false,
        ),
        // Only the anchored push is walked argument by argument, so a chained
        // second one is refused outright on the default branch rather than
        // waved through unexamined. The same chain off main stays allowed.
        ("git push origin feat && git push origin HEAD", "main", true),
        // The same holds when the chained push is a tag: refused outright on
        // the default branch, because the rule is about what went unexamined,
        // not about what the second refspec happens to name.
        (
            "git push origin feat && git push origin v1.2.3",
            "main",
            true,
        ),
        (
            "git push origin feat && git push origin v1.2.3",
            "feat",
            false,
        ),
        ("git push -u origin feat/x", "feat", false),
        ("git push", "feat", false),
        // Not a push at all: the second defect spec 071 fixes. Both of these
        // merely mention the gate, and both were refused before.
        ("grep -n 'origin main' settings.json", "main", false),
        ("sed -n 's/git push origin main//p' f", "main", false),
    ];

    for (cmd, branch, want_refused) in cases {
        let code = run_pre_tool_use(cmd, branch);
        let refused = code == 2;
        assert_eq!(
            refused, *want_refused,
            "`{cmd}` on branch {branch}: exit {code}, refused={refused}, want refused={want_refused}"
        );
    }
}

/// How the throwaway repository declares its default branch (spec 072 3.1).
///
/// `Floor` is a repository that answers neither way, which is the common CI
/// checkout and the case that must keep behaving exactly as it did before 072.
enum Declared {
    /// No override and no `origin/HEAD`: resolution reaches the `main` floor.
    Floor,
    /// The remote's own HEAD, which a clone sets, named by branch.
    OriginHead(&'static str),
    /// `$SPEC_SPINE_DEFAULT_BRANCH`, which outranks both.
    Env(&'static str),
}

/// Spec 072 3.5: the same gate, on a repository whose default branch is not
/// `main`.
///
/// This is the row that would have caught the defect. A gate asserted only
/// against a repository named the way the gate assumes cannot tell a resolved
/// name from a hardcoded one: every case in the matrix above passes
/// identically on the pre-072 body, because there the constant and the
/// repository agree by construction. Here they do not.
#[test]
fn the_push_gate_protects_the_branch_the_repository_actually_has() {
    if std::process::Command::new("jq")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_err()
    {
        eprintln!(
            "SKIPPED the_push_gate_protects_the_branch_the_repository_actually_has: jq absent"
        );
        return;
    }

    // (declaration, command, current branch, must be refused)
    let cases: &[(Declared, &str, &str, bool)] = &[
        // ── the remote's own HEAD says `trunk` ─────────────────────────────
        // A push that would update the resolved branch: refused, which is the
        // protection this spec exists to restore. Against the pre-072 body
        // every one of these three passes unrefused.
        (Declared::OriginHead("trunk"), "git push", "trunk", true),
        (
            Declared::OriginHead("trunk"),
            "git push origin trunk",
            "feat",
            true,
        ),
        (
            Declared::OriginHead("trunk"),
            "git push origin HEAD:trunk",
            "feat",
            true,
        ),
        // A tag push made from it: allowed. Spec 071's release step, preserved
        // on a repository 071 could not describe.
        (
            Declared::OriginHead("trunk"),
            "git push origin v1.2.3",
            "trunk",
            false,
        ),
        // A push naming `main`, which on this repository updates an ordinary
        // branch: allowed. `main` is not privileged here, and a gate that
        // refused it would be protecting a name rather than a branch.
        (
            Declared::OriginHead("trunk"),
            "git push origin main",
            "feat",
            false,
        ),
        (
            Declared::OriginHead("trunk"),
            "git push origin main",
            "trunk",
            false,
        ),
        // ── the environment override outranks both ────────────────────────
        (
            Declared::Env("release"),
            "git push origin release",
            "feat",
            true,
        ),
        (Declared::Env("release"), "git push", "release", true),
        // Checked out on `main`, with the override naming another branch: the
        // push is allowed, because `main` is an ordinary branch here too. This
        // is the case that separates "resolved" from "resolved, then main
        // privileged anyway".
        (
            Declared::Env("release"),
            "git push origin main",
            "main",
            false,
        ),
        // ── the floor, which is every adopter who was already on main ─────
        // Kept beside the rows above rather than left to the matrix in the
        // previous test: what these prove together is that no adopter on
        // `main` regressed while the gate stopped assuming them.
        (Declared::Floor, "git push", "main", true),
        (Declared::Floor, "git push origin main", "feat", true),
        (Declared::Floor, "git push origin v1.2.3", "main", false),
        (Declared::Floor, "git push", "feat", false),
    ];

    for (declared, cmd, branch, want_refused) in cases {
        let code = run_pre_tool_use_on(cmd, branch, declared);
        let refused = code == 2;
        let how = match declared {
            Declared::Floor => "undeclared".to_string(),
            Declared::OriginHead(b) => format!("origin/HEAD={b}"),
            Declared::Env(b) => format!("$SPEC_SPINE_DEFAULT_BRANCH={b}"),
        };
        assert_eq!(
            refused, *want_refused,
            "[{how}] `{cmd}` on branch {branch}: exit {code}, refused={refused}, want refused={want_refused}"
        );
    }
}

/// Run the shipped `PreToolUse` body against `cmd`, in a throwaway repository
/// checked out on `branch` whose default branch is undeclared, and return its
/// exit code.
fn run_pre_tool_use(cmd: &str, branch: &str) -> i32 {
    run_pre_tool_use_on(cmd, branch, &Declared::Floor)
}

/// The same, against a repository that declares its default branch.
fn run_pre_tool_use_on(cmd: &str, branch: &str, declared: &Declared) -> i32 {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let git = |args: &[&str]| {
        Command::new("git")
            .args(args)
            .current_dir(root)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("git runs")
    };
    git(&["init", "-q", "-b", branch]);
    fs::write(root.join("f"), "x").unwrap();
    git(&["add", "."]);
    git(&[
        "-c",
        "user.email=t@example.invalid",
        "-c",
        "user.name=t",
        "commit",
        "-qm",
        "c",
    ]);

    // `git symbolic-ref` writes the symref without requiring the target to
    // exist, so a repository can declare a default branch it has no remote
    // for. That is the shape a fresh clone has and the shape the resolver
    // reads, which is what makes this a test of resolution rather than of git.
    if let Declared::OriginHead(b) = declared {
        git(&[
            "symbolic-ref",
            "refs/remotes/origin/HEAD",
            &format!("refs/remotes/origin/{b}"),
        ]);
    }

    let body = hook_bodies()["PreToolUse"].join("\n");
    let payload = serde_json::json!({
        "tool_input": { "command": cmd },
        "cwd": root.to_str().unwrap(),
    })
    .to_string();

    let mut spawn = Command::new("sh");
    spawn
        .arg("-c")
        .arg(&body)
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    // The variable must be absent, not empty, in every other case: the
    // resolver tests it with `-n`, so an inherited value from the developer's
    // own shell would silently decide these cases.
    match declared {
        Declared::Env(b) => spawn.env("SPEC_SPINE_DEFAULT_BRANCH", b),
        _ => spawn.env_remove("SPEC_SPINE_DEFAULT_BRANCH"),
    };
    let mut child = spawn.spawn().expect("sh runs");
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(payload.as_bytes())
        .expect("payload written");
    child.wait().expect("hook exits").code().unwrap_or(-1)
}

/// Spec 046 3.5: a hook that cannot do its job says so instead of exiting
/// quietly. Every body names the absent-tool condition it skipped on.
#[test]
fn hooks_report_when_they_skip() {
    for (event, bodies) in hook_bodies() {
        let body = bodies.join("\n");
        assert!(
            body.contains("absent"),
            "{event} hook exits silently when spec-spine is absent"
        );
    }
}

#[test]
fn the_write_scanner_recognises_writes() {
    // Pin the scanner itself, so a future hook cannot slip a write past it by
    // matching a shape the scanner ignores.
    let body = "cd x; spec-spine index >/dev/null 2>&1; spec-spine --repo \"$r\" compile; \"$sc\" index check";
    let verbs = spec_spine_invocations(body);
    assert_eq!(verbs.len(), 3, "{verbs:?}");
    assert!(!is_read_only(&verbs[0]), "{:?}", verbs[0]);
    assert!(!is_read_only(&verbs[1]), "{:?}", verbs[1]);
    assert!(is_read_only(&verbs[2]), "{:?}", verbs[2]);
    let orig = "\"$sc\" compile --check >/dev/null 2>&1; c=$?";
    let v = spec_spine_invocations(orig);
    assert_eq!(v.len(), 1);
    assert!(is_read_only(&v[0]));
}

/// Spec 051 3.5: a hook resolves the binary the project declares before it
/// falls back to whatever `PATH` happens to hold. A repository that builds its
/// own binary must be governed by the one it builds. The machine that motivated
/// this spec had `spec-spine` on `PATH` one release behind the checkout, which
/// would have gated a 0.15.0 corpus with a 0.14.0 judgement.
#[test]
fn hooks_resolve_the_projects_binary_before_path() {
    for (event, bodies) in hook_bodies() {
        for body in bodies {
            assert!(
                body.contains("spec_spine_bin"),
                "{event} hook must resolve the binary through spec_spine_bin"
            );
            let env = body
                .find("SPEC_SPINE_BIN")
                .unwrap_or_else(|| panic!("{event}: resolver must honour $SPEC_SPINE_BIN"));
            let built = body
                .find("target/release/spec-spine")
                .unwrap_or_else(|| panic!("{event}: resolver must prefer the repo's own build"));
            let from_path = body
                .find("command -v spec-spine")
                .unwrap_or_else(|| panic!("{event}: resolver must fall back to PATH"));
            assert!(
                env < built && built < from_path,
                "{event}: order must be $SPEC_SPINE_BIN, then the repo's build, then PATH"
            );
        }
    }
}

// ── spec 080: a gate that cannot ask says so ─────────────────────────────────

/// Run the shipped `PreToolUse` body against `gh pr create` with
/// `$SPEC_SPINE_BIN` pointing at a stand-in that exits `check_exit` for
/// `check`, `0` for `couple`, and answers `--version` with `version`. Returns
/// the hook's exit code and its stderr.
fn run_pr_gate_with_stand_in(check_exit: i32, version: &str) -> (i32, String) {
    // A binary that carries the verb: `check --help` succeeds.
    run_pr_gate_with_stand_in_help(check_exit, 0, version)
}

/// As [`run_pr_gate_with_stand_in`], with the `check --help` probe's exit code
/// chosen by the test as well (spec 104 3.3).
///
/// `help_exit` non-zero is a binary that does not carry the `check` verb, where
/// clap spends exit 2 on the unrecognised subcommand: the case spec 104 3.1
/// separates from staleness.
fn run_pr_gate_with_stand_in_help(check_exit: i32, help_exit: i32, version: &str) -> (i32, String) {
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;
    use std::process::{Command, Stdio};

    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let git = |args: &[&str]| {
        Command::new("git")
            .args(args)
            .current_dir(root)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("git runs")
    };
    git(&["init", "-q", "-b", "feat"]);
    fs::create_dir_all(root.join("specs")).unwrap();
    fs::write(root.join("specs/.keep"), "").unwrap();
    git(&["add", "."]);
    git(&[
        "-c",
        "user.email=t@example.invalid",
        "-c",
        "user.name=t",
        "commit",
        "-qm",
        "c",
    ]);

    let stand_in = root.join("stand-in-spec-spine");
    // `check --help` is matched BEFORE `check`, because the gate's probe and the
    // gate's read differ only by that argument and the whole point of spec 104
    // is that they are different questions.
    fs::write(
        &stand_in,
        format!(
            "#!/bin/sh\ncase \"$*\" in\n  *--version*) echo '{version}'; exit 0 ;;\n\
             \x20 *'check --help'*) exit {help_exit} ;;\n  *check*) exit {check_exit} ;;\n\
             \x20 *couple*) exit 0 ;;\nesac\nexit 0\n"
        ),
    )
    .unwrap();
    fs::set_permissions(&stand_in, fs::Permissions::from_mode(0o755)).unwrap();

    let body = hook_bodies()["PreToolUse"].join("\n");
    let payload = serde_json::json!({
        "tool_input": { "command": "gh pr create --title t --body b" },
        "cwd": root.to_str().unwrap(),
    })
    .to_string();
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(&body)
        .current_dir(root)
        .env("SPEC_SPINE_BIN", &stand_in)
        .env_remove("SPEC_SPINE_DEFAULT_BRANCH")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("sh runs");
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(payload.as_bytes())
        .expect("payload written");
    let out = child.wait_with_output().expect("hook exits");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// Spec 080 3.1 and 3.2: exit 3 is a read that was not performed. The gate
/// still refuses, says so in those terms, names the binary and what it
/// answers to `--version`, and never calls the tree stale.
#[test]
fn the_pr_gate_reports_a_read_it_could_not_perform_as_that_and_not_as_stale() {
    let (code, err) = run_pr_gate_with_stand_in(3, "spec-spine 0.17.0-stand-in");
    assert_eq!(
        code, 2,
        "a gate whose check did not run is not green: {err}"
    );
    assert!(err.contains("not performed"), "{err}");
    assert!(
        err.contains("stand-in-spec-spine"),
        "names the binary: {err}"
    );
    assert!(
        err.contains("spec-spine 0.17.0-stand-in"),
        "relays what --version answered (3.2): {err}"
    );
    assert!(
        err.contains("0.18.0"),
        "names the floor the verb needs: {err}"
    );
    assert!(
        !err.to_lowercase().contains("stale"),
        "an old binary is not a stale tree: {err}"
    );
}

/// The other half of the pair: exit 2 is still reported as stale, with the
/// remedy unchanged, so the distinction is pinned rather than the refusal.
#[test]
fn the_pr_gate_still_reports_a_stale_tree_as_stale() {
    let (code, err) = run_pr_gate_with_stand_in(2, "spec-spine 0.18.0-stand-in");
    assert_eq!(code, 2, "{err}");
    assert!(err.contains("stale"), "{err}");
    assert!(err.contains("compile and index"), "{err}");
    assert!(!err.contains("not performed"), "{err}");
    assert!(
        !err.contains("0.18.0-stand-in"),
        "--version is not asked on an answered exit (3.2): {err}"
    );
}

/// Spec 080 3.1: exit 1 is a corpus that does not validate, which is neither
/// stale nor a missing read.
#[test]
fn the_pr_gate_reports_a_corpus_that_does_not_validate() {
    let (code, err) = run_pr_gate_with_stand_in(1, "spec-spine 0.18.0-stand-in");
    assert_eq!(code, 2, "{err}");
    assert!(err.contains("does not validate"), "{err}");
    assert!(!err.contains("is stale"), "{err}");
}

// --- spec 090: the commit boundary ------------------------------------------

/// The hook that fires whatever wrote the bytes. Read as a file rather than
/// out of `.claude/settings.json`, because this one is a git hook and not a
/// Claude Code hook; the property asserted over it is the same.
fn pre_commit_body() -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    fs::read_to_string(root.join(".githooks/pre-commit")).expect(".githooks/pre-commit exists")
}

/// Spec 090 3.1: the one hook whose actor IS the committer still refuses
/// rather than repairs. A pre-commit hook that regenerated the derived tree
/// would collapse "was never stale" and "was stale until the hook fixed it"
/// into the same commit, and telling those apart is the whole content of
/// `check`.
#[test]
fn the_commit_boundary_hook_reads_and_never_repairs() {
    let body = pre_commit_body();
    let verbs = spec_spine_invocations(&body);
    assert!(
        !verbs.is_empty(),
        "no recognised spec-spine call in .githooks/pre-commit; a hook the \
         scanner reads as call-free would pass this test vacuously"
    );
    for verb in &verbs {
        assert!(
            is_read_only(verb),
            ".githooks/pre-commit runs a writing verb: {verb:?}"
        );
    }
}

/// Spec 090 3.1: and it stages nothing. Refusing while quietly adding the
/// regenerated shards to the index is the same repair wearing a refusal's
/// exit code.
#[test]
fn the_commit_boundary_hook_stages_nothing() {
    for line in pre_commit_body().lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            continue;
        }
        assert!(
            !trimmed.contains("git add"),
            ".githooks/pre-commit stages a file: {line}"
        );
    }
}

/// Spec 090 3.2: no coupling verdict at this boundary. `couple` builds its
/// diff from `git diff base...head`, a range of commits, which cannot contain
/// the change being committed. A verdict there would be about the previous
/// commit wearing this one's name.
#[test]
fn the_commit_boundary_hook_runs_no_coupling_verdict() {
    for verb in spec_spine_invocations(&pre_commit_body()) {
        assert_ne!(
            verb.first().map(String::as_str),
            Some("couple"),
            ".githooks/pre-commit asks for a coupling verdict it cannot get"
        );
    }
}

/// Spec 090 3.3: a refusal that hides its own escape hatch produces a
/// contributor who disables the hook entirely.
#[test]
fn the_commit_boundary_hook_names_its_own_escape() {
    assert!(
        pre_commit_body().contains("--no-verify"),
        "the refusal must name the standard bypass"
    );
}

/// Spec 090 3.4: registration is per clone, and the enabler is the only thing
/// that turns the hook on. Until it runs, the hook is inert bytes in the tree.
#[test]
fn the_enabler_registers_the_hooks_path_and_says_how_to_undo_it() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let sh = fs::read_to_string(root.join(".githooks/enable-hooks.sh")).unwrap();
    assert!(sh.contains("git config core.hooksPath"), "{sh}");
    assert!(sh.contains("--unset core.hooksPath"), "{sh}");
}

// ── spec 099: the session hooks report the verdict, not a guess ──────────────

/// `check`'s report for a corpus whose committed shards are byte-exact but
/// whose spec claims a unit that does not resolve. Copied from a 0.19.0 run
/// against a scratch corpus, because the strings the hooks match on are the
/// ones the verb actually prints.
const BLOCKING_ONLY: &str = "spec-registry: fresh\n\
codebase-index: UNRESOLVED CLAIM: 1 unresolved claim(s) over 1 spec(s), which is not staleness\n  \
I-004 001-missing-territory: spec '001-missing-territory' file unit 'src/nothing.rs' does not exist\n  \
regenerating the index does not clear this: each diagnostic is recomputed from the corpus on every run\n";

const STALE_ONLY: &str = "spec-registry: fresh\n\
codebase-index: STALE (run `spec-spine index`)\n1 stale shard(s):\n  missing by-spec/002-second.json\n";

const MIXED: &str = "spec-registry: fresh\n\
codebase-index: STALE (run `spec-spine index`)\n1 stale shard(s):\n  missing by-spec/002-second.json\n\
codebase-index: UNRESOLVED CLAIM: 1 unresolved claim(s) over 1 spec(s), which is not staleness\n  \
I-004 001-missing-territory: spec '001-missing-territory' file unit 'src/nothing.rs' does not exist\n  \
regenerating addresses the stale shard(s) only, not the unresolved claim(s)\n";

const INVALID: &str = "spec-registry: INVALID: the corpus fails validation, so staleness was not \
computed (run `spec-spine compile --check` for the violations)\n";

const NOT_READ: &str =
    "spec-spine: config error: TOML parse error at line 121\nunknown field `nonsense`\n";

const HEALTHY: &str = "spec-registry: fresh\ncodebase-index: fresh\n";

/// Run one shipped session-hook body against a stand-in binary whose `check`
/// exit code and report are both chosen by the caller: the shape spec 080 3.4
/// established for the PR gate, which is why the assertions below can pin the
/// distinction between verdicts rather than the mere presence of a message.
///
/// `help_exit` is what the stand-in spends on `check --help`. Non-zero stands
/// for a binary predating the verb, which clap answers with exit 2, the same
/// code this tool spends on staleness (spec 063 3.1).
///
/// The body is read out of `.claude/settings.json`, never restated here. A test
/// that asserted against its own copy of the script would pass while the
/// shipped one stayed wrong.
fn run_session_hook(
    event: &str,
    check_exit: i32,
    report: &str,
    version: &str,
    help_exit: i32,
) -> (i32, String) {
    use std::os::unix::fs::PermissionsExt;
    use std::process::{Command, Stdio};

    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join("specs")).unwrap();

    let report_file = root.join("report.txt");
    fs::write(&report_file, report).unwrap();

    let stand_in = root.join("stand-in-spec-spine");
    fs::write(
        &stand_in,
        format!(
            "#!/bin/sh\ncase \"$*\" in\n  *--version*) echo '{version}'; exit 0 ;;\n  \
             *--help*) exit {help_exit} ;;\nesac\ncat '{}'\nexit {check_exit}\n",
            report_file.display()
        ),
    )
    .unwrap();
    fs::set_permissions(&stand_in, fs::Permissions::from_mode(0o755)).unwrap();

    let body = hook_bodies()[event].join("\n");
    let out = Command::new("sh")
        .arg("-c")
        .arg(&body)
        .current_dir(root)
        .env("CLAUDE_PROJECT_DIR", root)
        .env("SPEC_SPINE_BIN", &stand_in)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("sh runs");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    )
}

fn run_stop(check_exit: i32, report: &str) -> (i32, String) {
    run_session_hook("Stop", check_exit, report, "spec-spine 0.19.0-stand-in", 0)
}

fn run_session_start(check_exit: i32, report: &str) -> (i32, String) {
    run_session_hook(
        "SessionStart",
        check_exit,
        report,
        "spec-spine 0.19.0-stand-in",
        0,
    )
}

/// Spec 099 3.1: an unresolved claim is reported as itself. The remedy the
/// staleness line carries is regeneration, and regeneration provably does not
/// clear a claim on a unit that does not exist (spec 098 1.2): `index` exits
/// 0, writes the same bytes, and the next read refuses identically.
#[test]
fn the_stop_hook_reports_an_unresolved_claim_as_itself() {
    let (code, out) = run_stop(2, BLOCKING_ONLY);
    assert_eq!(code, 0, "the hook advises and never refuses (1.4): {out}");
    assert!(out.contains("UNRESOLVED CLAIM"), "{out}");
    assert!(
        !out.contains("[freshness] STALE"),
        "an unresolved claim is not staleness: {out}"
    );
    assert!(
        !out.contains("spec-spine compile"),
        "regeneration is not the remedy here, so it must not be named: {out}"
    );
}

/// The other half of the pair: staleness is still staleness, with the wording
/// untouched, so the distinction is what this spec adds rather than a rewrite
/// of a message adopters already read (3.1, D-3).
#[test]
fn the_stop_hook_still_reports_a_stale_tree_as_stale() {
    let (code, out) = run_stop(2, STALE_ONLY);
    assert_eq!(code, 0, "{out}");
    assert!(
        out.contains("[freshness] STALE: run `spec-spine compile` and `index`"),
        "the stale wording is unchanged: {out}"
    );
    assert!(out.contains("Not regenerated here"), "{out}");
    assert!(!out.contains("UNRESOLVED CLAIM"), "{out}");
}

/// Spec 099 3.1: in the mixed case both are reported and neither is elided.
/// Reporting only the stale half is what the pre-099 body did through the
/// `SessionStart` banner, and it named regeneration for a refusal half of
/// which regeneration does not touch.
#[test]
fn the_stop_hook_reports_both_halves_of_a_mixed_verdict() {
    let (code, out) = run_stop(2, MIXED);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("[freshness] STALE"), "{out}");
    assert!(
        out.contains("UNRESOLVED CLAIM"),
        "the half regeneration does not fix must not be dropped: {out}"
    );
}

/// Spec 099 3.1: exit 1 is a corpus that does not validate, which is neither
/// stale nor cleared by regenerating.
#[test]
fn the_stop_hook_reports_a_corpus_that_does_not_validate() {
    let (code, out) = run_stop(1, INVALID);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("INVALID"), "{out}");
    assert!(!out.contains("[freshness] STALE"), "{out}");
    assert!(!out.contains("spec-spine compile"), "{out}");
}

/// Spec 099 3.1 and 3.2, on spec 080's rule: exit 3 is a read that was not
/// performed. The hook says so, names the binary and what it answers to
/// `--version`, and never calls the tree stale.
#[test]
fn the_stop_hook_reports_a_read_it_could_not_perform() {
    let (code, out) = run_stop(3, NOT_READ);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("NOT READ"), "{out}");
    assert!(
        out.contains("stand-in-spec-spine"),
        "names the binary: {out}"
    );
    assert!(
        out.contains("0.19.0-stand-in"),
        "relays what --version answered (3.2): {out}"
    );
    assert!(!out.contains("[freshness] STALE"), "{out}");
    assert!(!out.contains("spec-spine compile"), "{out}");
}

/// Spec 099 3.2: clap spends exit 2 on an unrecognised subcommand and this
/// tool spends exit 2 on staleness, so a binary predating `check` hands back
/// the staleness code without having read anything. The probe runs first and
/// the code is never believed on its own.
#[test]
fn the_stop_hook_reports_a_binary_that_predates_the_verb() {
    let (code, out) = run_session_hook("Stop", 2, STALE_ONLY, "spec-spine 0.17.0-stand-in", 2);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("NOT READ"), "{out}");
    assert!(out.contains("0.17.0-stand-in"), "{out}");
    assert!(
        out.contains("0.18.0"),
        "names the floor the verb needs: {out}"
    );
    assert!(
        !out.contains("[freshness] STALE"),
        "a binary that cannot answer is not a stale tree: {out}"
    );
}

/// Spec 099 3.1: a report the hook does not recognise is said to be exactly
/// that. Guessing either remedy is the defect one level down.
#[test]
fn the_stop_hook_guesses_no_remedy_for_a_report_it_cannot_read() {
    let (code, out) = run_stop(2, "a shape this hook has never seen\n");
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("REFUSED"), "{out}");
    assert!(!out.contains("[freshness] STALE"), "{out}");
    assert!(!out.contains("UNRESOLVED CLAIM"), "{out}");
}

#[test]
fn the_stop_hook_says_nothing_when_both_trees_are_fresh() {
    let (code, out) = run_stop(0, HEALTHY);
    assert_eq!(code, 0);
    assert!(out.trim().is_empty(), "a fresh tree gets no banner: {out}");
}

/// Spec 099 1.4: the hook advises and does not refuse, in every branch. Design
/// note 06's H-6 asks whether that should change; pinning the posture here
/// means a later answer to it is a deliberate edit rather than a side effect
/// of one, which is exactly what this spec was not allowed to decide.
#[test]
fn the_stop_hook_advises_and_never_refuses() {
    for (exit, report) in [
        (0, HEALTHY),
        (1, INVALID),
        (2, BLOCKING_ONLY),
        (2, MIXED),
        (3, NOT_READ),
        (7, "an exit code this hook does not know\n"),
    ] {
        let (code, out) = run_stop(exit, report);
        assert_eq!(
            code, 0,
            "check exit {exit} must still advise, not refuse: {out}"
        );
    }
}

/// Spec 099 3.3: after spec 098 the blocking-only verdict stopped matching
/// `codebase-index: STALE` and fell to the `unknown (check exit 2)` fallback.
/// "Unknown" was no longer false, which is why 098 accepted it as a trade, and
/// it told a session nothing it could act on.
#[test]
fn the_session_banner_reports_an_unresolved_claim_as_itself() {
    let (code, out) = run_session_start(2, BLOCKING_ONLY);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("codebase index: UNRESOLVED CLAIM"), "{out}");
    assert!(
        !out.contains("unknown (check exit"),
        "a verdict the verb documents is not unknown: {out}"
    );
}

/// The stale-only banner is unchanged, wording included, for spec 098 3.3's
/// reason: a caller reading staleness today reads it after.
#[test]
fn the_session_banner_keeps_the_stale_only_wording() {
    let (code, out) = run_session_start(2, STALE_ONLY);
    assert_eq!(code, 0, "{out}");
    assert!(
        out.contains("codebase index: STALE, run spec-spine index"),
        "{out}"
    );
    assert!(!out.contains("UNRESOLVED CLAIM"), "{out}");
}

/// Spec 099 3.3: the mixed banner reported the stale half and dropped the
/// unresolved one, so a reader regenerated, watched the refusal survive, and
/// had been told nothing about why.
#[test]
fn the_session_banner_reports_both_halves_of_a_mixed_verdict() {
    let (code, out) = run_session_start(2, MIXED);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("STALE"), "{out}");
    assert!(
        out.contains("UNRESOLVED CLAIM"),
        "the mixed banner dropped this half before spec 099: {out}"
    );
}

#[test]
fn the_session_banner_still_reports_a_healthy_tree() {
    let (code, out) = run_session_start(0, HEALTHY);
    assert_eq!(code, 0, "{out}");
    assert!(
        out.contains("spec registry: fresh; codebase index: fresh"),
        "{out}"
    );
}

// ===== spec 104: every hook reads the exit code the same way =====

/// Spec 104 §3.1, §3.3: exit 2 from a binary that carries the verb is
/// staleness, and the message is unchanged.
#[test]
fn spec104_exit_2_from_a_current_binary_is_still_stale() {
    let (code, err) = run_pr_gate_with_stand_in_help(2, 0, "spec-spine 0.19.0-stand-in");
    assert_eq!(code, 2, "{err}");
    assert!(err.contains("a committed shard tree is stale"), "{err}");
    assert!(err.contains("spec-spine compile and index"), "{err}");
}

/// Spec 104 §3.1: exit 2 from a binary that does NOT carry the verb is clap's
/// unrecognised-subcommand code, not this tool's staleness code.
#[test]
fn spec104_exit_2_from_a_binary_without_the_verb_is_not_stale() {
    let (code, err) = run_pr_gate_with_stand_in_help(2, 127, "spec-spine 0.17.0-stand-in");
    assert_eq!(code, 2, "every non-zero code still refuses: {err}");
    // The defect: an adopter on 0.17.0 was told to regenerate correct shards.
    assert!(
        !err.contains("is stale"),
        "a tree that was never read is not known to be stale: {err}"
    );
    assert!(
        !err.contains("spec-spine compile and index"),
        "the remedy that does not work must not be named: {err}"
    );
    assert!(err.contains("does not carry the check verb"), "{err}");
    assert!(
        err.contains("0.17.0-stand-in"),
        "the binary is named: {err}"
    );
    assert!(err.contains("0.18.0"), "the floor is named: {err}");
}

/// Spec 104 §1.3, §3.3: the probe costs nothing on the happy path. A binary
/// whose `check --help` fails but whose `check` succeeds still passes the
/// gate, which pins that the probe is reached only from the exit-2 arm.
#[test]
fn spec104_the_probe_does_not_run_on_the_happy_path() {
    let (code, err) = run_pr_gate_with_stand_in_help(0, 127, "spec-spine 0.19.0-stand-in");
    assert_eq!(code, 0, "exit 0 is an answer and needs no probe: {err}");
    assert!(!err.contains("does not carry the check verb"), "{err}");
}

/// Spec 104 §3.2: `SessionStart` reports an unperformed read as one, naming
/// the binary, rather than as a shape it does not recognise.
#[test]
fn spec104_session_start_reports_exit_3_as_a_read_not_performed() {
    let (code, out) = run_session_hook(
        "SessionStart",
        3,
        "spec-spine: config error: unknown field `nope`\n",
        "spec-spine 0.19.0-stand-in",
        0,
    );
    assert_eq!(code, 0, "the banner advises and never refuses: {out}");
    assert!(
        !out.contains("unknown (check exit 3)"),
        "exit 3 is a shape the verb documents, not an unrecognised one: {out}"
    );
    assert!(out.contains("NOT READ"), "{out}");
    assert!(
        out.contains("0.19.0-stand-in"),
        "the binary is named, as the Stop hook has named it since spec 099: {out}"
    );
    // Both halves, since neither tree was judged.
    assert_eq!(
        out.matches("NOT READ").count(),
        2,
        "a read that did not happen did not happen for either tree: {out}"
    );
}

/// Spec 104 §3.2: spec 099 §3.3's fallback stays for a code neither hook knows.
#[test]
fn spec104_an_unrecognised_code_still_falls_back() {
    let (code, out) = run_session_hook(
        "SessionStart",
        7,
        "something nobody has seen\n",
        "spec-spine 0.19.0-stand-in",
        0,
    );
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("unknown (check exit 7)"), "{out}");
    assert!(!out.contains("NOT READ"), "{out}");
}
