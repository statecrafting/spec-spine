// Spec: specs/046-kit-hooks-read-never-write/spec.md
//! Kit hook tests (spec 046): the Claude Code hooks the kit ships observe the
//! tree, they do not repair it. Three of the four originally ran a writing
//! subcommand (`compile`, `index`) from a context that could not commit the
//! result, and one of those writes stalled an adopter's orchestrator for
//! eleven hours on a tree it had dirtied itself. These tests read the shipped
//! `kit/settings.json` and refuse a mutating `spec-spine` invocation in any
//! hook other than the one sanctioned write.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

const KIT_SETTINGS: &str = include_str!("../../../kit/settings.json");

/// Every hook command body in the kit, keyed by event name.
fn hook_bodies() -> BTreeMap<String, Vec<String>> {
    let v: serde_json::Value =
        serde_json::from_str(KIT_SETTINGS).expect("kit/settings.json parses");
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
        Some("couple") => true,
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
            "kit/settings.json must ship a {event} hook"
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

/// Spec 046 3.4: pushing to the default branch is refused, by branch and by
/// refspec, before the PR gate runs.
#[test]
fn pre_tool_use_refuses_a_push_to_main() {
    let body = hook_bodies()["PreToolUse"].join("\n");
    assert!(body.contains("git push"), "no push gate");
    for form in ["origin main", "HEAD:main", "origin +main"] {
        assert!(
            body.contains(form),
            "push gate does not match the `{form}` refspec form"
        );
    }
    assert!(
        body.contains("if [ \"$br\" = main ]; then"),
        "push gate does not refuse by current branch"
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
        // Only the first push is walked argument by argument, so a chained
        // second one is refused outright on the default branch rather than
        // waved through unexamined. The same chain off main stays allowed.
        ("git push origin feat && git push origin HEAD", "main", true),
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

/// Run the shipped `PreToolUse` body against `cmd`, in a throwaway repository
/// checked out on `branch`, and return its exit code.
fn run_pre_tool_use(cmd: &str, branch: &str) -> i32 {
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

    let body = hook_bodies()["PreToolUse"].join("\n");
    let payload = serde_json::json!({
        "tool_input": { "command": cmd },
        "cwd": root.to_str().unwrap(),
    })
    .to_string();

    let mut child = Command::new("sh")
        .arg("-c")
        .arg(&body)
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("sh runs");
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

/// Spec 051 3.6: this repository runs the hooks it ships. `kit_hooks.rs` used to
/// assert properties of a file nothing executed; the governance halves of the
/// two configurations must now be identical.
#[test]
fn this_repository_runs_the_hooks_it_ships() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let own: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(root.join(".claude/settings.json")).unwrap())
            .expect(".claude/settings.json parses");
    let kit: serde_json::Value = serde_json::from_str(KIT_SETTINGS).unwrap();

    assert_eq!(
        own["hooks"], kit["hooks"],
        "this repository's hooks must be the ones the kit ships"
    );
    assert_eq!(
        own["permissions"]["deny"], kit["permissions"]["deny"],
        "the destructive-command refusals are governance, not machine preference"
    );
}
