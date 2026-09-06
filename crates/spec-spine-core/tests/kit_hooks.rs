// Spec: specs/046-kit-hooks-read-never-write/spec.md
//! Kit hook tests (spec 046): the Claude Code hooks the kit ships observe the
//! tree, they do not repair it. Three of the four originally ran a writing
//! subcommand (`compile`, `index`) from a context that could not commit the
//! result, and one of those writes stalled an adopter's orchestrator for
//! eleven hours on a tree it had dirtied itself. These tests read the shipped
//! `kit/settings.json` and refuse a mutating `spec-spine` invocation in any
//! hook other than the one sanctioned write.

use std::collections::BTreeMap;

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
                let mut words = words.into_iter().peekable();
                if words.peek().map(String::as_str) == Some("--repo") {
                    words.next();
                    words.next();
                }
                out.push(words.collect());
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
        body.contains("[ \"$br\" = main ]"),
        "push gate does not refuse by current branch"
    );
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
