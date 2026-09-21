//! Gate tests (spec 064, retargeted by spec 120 3.6): this repository has one
//! definition of the governed loop, it is the root `Makefile`, and CI calls it
//! rather than restating it.
//!
//! The chain used to be defined in `kit/Makefile`, the copy the repository
//! distributed and then ran on itself, with `kit/govern.yml` as the shipped
//! workflow calling it. The kit is gone; the two rules it was held to are not,
//! and they are what these assertions are: a gate never writes, and the chain
//! has one definition. The subjects are now `Makefile` and
//! `.github/workflows/ci.yml`.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use spec_spine_core::scaffold_init;
use spec_spine_types::Config;

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
/// gate files explain themselves at length, and prose naming a verb is not an
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
            // Skip a cargo target SELECTOR: `cargo build --bin spec-spine
            // --locked` names the binary, it does not invoke it, and the word
            // after it is a cargo flag rather than a verb. Decided on the token
            // BEFORE the name, so an actual invocation whose first word happens
            // to be a flag is still read and still refused.
            if expanded[..i].trim_end().ends_with("--bin") {
                continue;
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

/// The verbs this binary has. A gate file naming anything else is broken.
const VERBS: &[&str] = &[
    // Spec 075: the composed freshness read the protocol and the gate call.
    "check",
    "compile",
    "index",
    "registry",
    "lint",
    "couple",
    "verify",
    // Spec 120 3.1 removed `init`; a gate file naming it is now broken, which
    // is what this list is for.
    "attest",
    "verify-attestation",
    "config",
];

/// Read-only forms, by the same rule `tests/harness_hooks.rs` applies to the hooks
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

/// §3.4: every invocation names a verb this binary has. A gate file naming a
/// renamed verb is exactly the drift spec 048 found in the skills.
#[test]
fn every_gate_invocation_names_a_real_verb() {
    for rel in ["Makefile", ".github/workflows/ci.yml"] {
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
    let makefile = read("Makefile");
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

/// §3.4: the chain in `Makefile` is the chain `AGENTS.md` lists, in order.
/// The same assertion spec 051 made for the skills: a step CI enforces and the
/// gate omits is a step every adopter skips.
#[test]
fn the_gate_chain_follows_agents_md() {
    let listed: Vec<String> = agents_md_gate_commands();
    let gate = invocations(&target_body(&read("Makefile"), "gate"));

    // `compile --check` stands for `compile`: the kit's gate is read-only and
    // CI substitutes the same way, because a gate must never repair the tree it
    // is judging. The writing `index` has the same standing and lives in
    // `refresh`.
    let normalize = |c: &str| c.replace(" --check", "").trim().to_string();
    let listed_heads: Vec<String> = listed.iter().map(|c| normalize(c)).collect();

    let mut cursor = 0usize;
    for cmd in &gate {
        // Spec 114 3.2: `config show` is the ownership guard's PROBE, a
        // configuration read that decides whether the next step runs. It is not
        // a step of the governed chain and `AGENTS.md` does not list it, so the
        // in-order walk skips it. It is not thereby unasserted: it still has to
        // name a real, read-only verb (the two tests above), and
        // `the_gate_reads_the_effective_config_before_asserting_ownership`
        // below refuses a gate that stopped making it.
        if normalize(cmd) == "config show" {
            continue;
        }
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

// ── spec 113: how a document's gate list is read ─────────────────────────
//
// Spec 113 §3.2 compared the gate list in two documents step for step: the one
// the kit shipped and the one `scaffold.rs` generated. Spec 120 §3.3 and §3.4
// removed both, so the comparison has no second document. The PARSER stays and
// becomes the one `agents_md_gate_commands` uses, because what it carries is
// not the comparison: it is the drop guard below, which refuses a fence line
// that reads as an invocation and parses to nothing. A list this parser
// silently dropped a step from would satisfy every assertion made over it.

/// One step of a fenced gate list. `conditional` records whether the document
/// renders the step commented out, which is how a protocol writes a step that
/// belongs in the gate only under a given configuration: dropping that
/// distinction would let an unconditional assertion match a conditional one.
#[derive(Debug, PartialEq, Eq)]
struct GateStep {
    command: String,
    conditional: bool,
}

/// The fenced `sh` block under a document's "Run the gate before every commit"
/// step, as ordered `spec-spine` steps. A leading `# ` marks a conditional step
/// and is stripped; a trailing ` # …` is that condition's prose, not part of the
/// invocation.
///
/// Deliberately generic over the text rather than reading one path. It was
/// written to run over two documents, the one the kit shipped and the one the
/// scaffold generated; spec 120 3.3 and 3.4 removed both, and the parser stays
/// generic because the property it reads is a property of a gate list, not of a
/// filename.
fn gate_steps(text: &str) -> Vec<GateStep> {
    let start = text
        .find("Run the gate before every commit")
        .expect("the document names the gate step");
    let tail = &text[start..];
    let open = tail
        .find("```sh")
        .expect("the gate list is a fenced sh block");
    let body = &tail[open + "```sh".len()..];
    // The close is a fence on its own line, possibly indented: `AGENTS.md`
    // carries the block inside a numbered list. A bare `find("```")` would also
    // match a backtick run inside the block and truncate it, and the drop guard
    // below could not see that, because it counts over the same truncated slice.
    let close = body
        .match_indices("```")
        .find(|(i, _)| {
            body[..*i]
                .rsplit('\n')
                .next()
                .is_some_and(|indent| indent.chars().all(char::is_whitespace))
        })
        .map(|(i, _)| i)
        .expect("the fence closes on a line of its own");
    let fence = &body[..close];
    let steps: Vec<GateStep> = fence
        .lines()
        .filter_map(|l| {
            let t = l.trim();
            let (conditional, t) = match t.strip_prefix("# ") {
                Some(rest) => (true, rest.trim()),
                None => (false, t),
            };
            let cmd = t.strip_prefix("spec-spine ")?;
            // One space, not two: `AGENTS.md` is hand-maintained and is not
            // held to the generator's spacing. Splitting on the wider separator
            // would fold a single-spaced condition into the command, and two
            // documents spelling it the same wrong way would compare equal.
            let cmd = cmd.split(" #").next().unwrap_or(cmd).trim();
            Some(GateStep {
                command: cmd.to_string(),
                conditional,
            })
        })
        .collect();

    // A line this parser drops is a step neither list would carry, so parity
    // would hold over a gate with a missing step. Every line that reads as an
    // invocation has to become one.
    //
    // Lenient about the comment marker where the parser is strict, which is
    // exactly the gap being guarded: `#spec-spine check`, written without the
    // space the parser requires, counts here and parses to nothing. Prose that
    // merely mentions the binary ("# this replaces spec-spine compile --check")
    // is an invocation under neither reading and is counted by neither, so the
    // guard does not fire on a comment a future maintainer adds.
    let named = fence
        .lines()
        .filter(|l| {
            l.trim()
                .trim_start_matches('#')
                .trim_start()
                .starts_with("spec-spine ")
        })
        .count();
    assert_eq!(
        steps.len(),
        named,
        "{named} fence line(s) name spec-spine and {} parsed as steps; a gate \
         line this parser drops is invisible to the parity assertion",
        steps.len()
    );
    steps
}

/// `AGENTS.md`'s gate list, unconditional steps only.
///
/// Read through [`gate_steps`] rather than with a second, simpler parser: the
/// two used to coexist and the simpler one had no drop guard, so a fence line
/// it failed to recognise was invisible to every assertion built on it. A
/// conditional step is excluded here because the chain being compared is the
/// one that always runs.
fn agents_md_gate_commands() -> Vec<String> {
    gate_steps(&read("AGENTS.md"))
        .into_iter()
        .filter(|s| !s.conditional)
        .map(|s| s.command)
        .collect()
}

/// §3.1: the language targets are guarded on a MANIFEST probe, not a command
/// probe. A tree with cargo installed and no `Cargo.toml` is the specify-first
/// case, which is three of the four governed repositories.
#[test]
fn language_targets_probe_for_a_manifest_not_a_tool() {
    let makefile = read("Makefile");
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

/// §3.2: the PR body reaches the gate as a FILE.
///
/// Spec 073's other half, the `has_cargo` job-output probe, was a property of
/// the workflow the kit shipped to a repository that might have no Cargo
/// manifest. That workflow is gone (spec 120 3.4) and this repository is a
/// Cargo workspace, so the probe has no subject here; the `Makefile`'s guarded
/// language targets, which are the same finding in the file that survived, are
/// asserted two tests above.
#[test]
fn the_pr_body_reaches_the_gate_as_a_file() {
    let wf = read(".github/workflows/ci.yml");
    // §3.2: the PR body reaches the gate through a file, not through shell
    // quoting, because a body carrying a waiver line has no safe quoting.
    //
    // Since spec 114 §3.4 the workflow reaches `--pr-body` by handing the file
    // to the one gate definition, which names the flag; asserting the flag's
    // spelling in the workflow would now refuse the consolidation 064's own
    // header comment asks for. The property is unchanged and is asserted in
    // both halves: the workflow writes the file and passes its path, and the
    // target turns that path into `--pr-body`.
    assert!(wf.contains("RUNNER_TEMP"), "{wf}");
    assert!(
        wf.contains("PR_BODY="),
        "the file's path is handed to the gate: {wf}"
    );
    let makefile = read("Makefile");
    assert!(
        makefile.contains("--pr-body"),
        "the one gate definition is where the flag is spelled: {makefile}"
    );
}

/// Spec 020 3.3, held through spec 120 3.7: the merge driver is registered on
/// the shard globs this repository **actually commits**, which are derived from
/// the configured `derived_dir` and not from the default it used to be.
///
/// This is the assertion that catches a half-done relocation: a `.gitattributes`
/// still naming `.derived/` registers the driver on files nothing writes, and a
/// same-shard conflict then arrives with conflict markers in a committed
/// artifact, which is exactly what spec 020 exists to prevent. Read out of the
/// effective configuration rather than restated, so the two cannot drift again.
#[test]
fn the_gitattributes_registers_the_driver_on_the_configured_shard_globs() {
    let cfg: Config = {
        let src = read("spec-spine.toml");
        spec_spine_types::load_config(&src).expect("spec-spine.toml parses")
    };
    let derived = cfg.layout.derived_dir.trim_end_matches('/');
    let ours = read(".gitattributes");
    for glob in [
        format!("{derived}/spec-registry/by-spec/*.json"),
        format!("{derived}/codebase-index/by-spec/*.json"),
        format!("{derived}/codebase-index/by-package/*.json"),
        format!("{derived}/codebase-index/slices.json"),
    ] {
        assert!(
            ours.lines().any(|l| {
                let l = l.trim();
                l.starts_with(&glob) && l.contains("merge=spec-spine-derived-regen")
            }),
            "`{glob}` is a committed shard glob and the merge driver is not \
             registered on it; .gitattributes reads:\n{ours}"
        );
    }
    // And no registration survives on a path this repository does not write.
    for line in ours
        .lines()
        .filter(|l| l.contains("merge=spec-spine-derived-regen"))
    {
        assert!(
            line.trim_start().starts_with(derived),
            "the driver is registered on `{}`, which is not under the configured \
             derived directory `{derived}`",
            line.trim()
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
    let makefile = read("Makefile");
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
    let makefile = read("Makefile");
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

// ── spec 114: one gate definition, and both legs of the workflow call it ──

/// One executable step of a workflow: the event condition it runs under and the
/// script it runs, read out of the parsed document.
///
/// Parsed rather than searched, because §3.4's property is about which commands
/// run and a text search cannot tell a command from a sentence about one.
/// `.github/workflows/ci.yml` names `make gate` twice and only one of
/// those is a step; the other is the header comment saying the workflow has one
/// gate definition while the pull-request leg restated it (114 D-5).
#[derive(Debug)]
struct WorkflowStep {
    name: Option<String>,
    cond: Option<String>,
    run: Option<String>,
}

/// Every step of every job, in document order.
fn workflow_steps(yaml: &str) -> Vec<WorkflowStep> {
    let doc: serde_yaml::Value =
        serde_yaml::from_str(yaml).unwrap_or_else(|e| panic!("the workflow parses as YAML: {e}"));
    let mut out = Vec::new();
    let Some(jobs) = doc.get("jobs").and_then(|j| j.as_mapping()) else {
        return out;
    };
    for (_, job) in jobs {
        let Some(steps) = job.get("steps").and_then(|s| s.as_sequence()) else {
            continue;
        };
        for step in steps {
            out.push(WorkflowStep {
                name: step
                    .get("name")
                    .and_then(|v| v.as_str())
                    .map(str::to_string),
                cond: step.get("if").and_then(|v| v.as_str()).map(str::to_string),
                run: step.get("run").and_then(|v| v.as_str()).map(str::to_string),
            });
        }
    }
    out
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Leg {
    Push,
    PullRequest,
}

/// Which event leg a step runs on, read from its `if:` expression and from
/// nothing else (114 §3.4). A step *named* "Governed loop (pull request)" that
/// carries a push condition is a push step, and a leg identified by its name
/// would be identified by the half of the file a maintainer forgets to update.
fn leg(cond: Option<&str>) -> Option<Leg> {
    let c = cond?;
    if c.contains("github.event_name == 'pull_request'") {
        Some(Leg::PullRequest)
    } else if c.contains("github.event_name != 'pull_request'") {
        Some(Leg::Push)
    } else {
        None
    }
}

/// Every `${{ … }}` expression replaced by one opaque word. GitHub substitutes
/// these before the shell sees the script, so an expression is a single token
/// and never a shell operator: without this the `||` inside
/// `${{ github.base_ref || 'main' }}` would read as a command separator and
/// split one `make` invocation into three commands.
fn mask_expressions(run: &str) -> String {
    let mut out = String::with_capacity(run.len());
    let mut rest = run;
    while let Some(open) = rest.find("${{") {
        out.push_str(&rest[..open]);
        match rest[open..].find("}}") {
            Some(close) => {
                out.push_str("GITHUB_EXPR");
                rest = &rest[open + close + 2..];
            }
            None => {
                out.push_str("GITHUB_EXPR");
                rest = "";
            }
        }
    }
    out.push_str(rest);
    out
}

/// One command of a `run:` script: the words it executes, with the shell
/// quoting removed, and the targets of its output redirections.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct ShellCommand {
    words: Vec<String>,
    redirects: Vec<String>,
}

/// What the word now being read is the target of.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pending {
    /// An ordinary word of the command.
    Word,
    /// The file an output redirection writes.
    Out,
    /// The file an input redirection reads, which nothing here asks about.
    In,
}

/// A reader for the subset of `sh` a `run:` script in this kit is allowed to
/// use, over text GitHub's `${{ … }}` expressions have already been masked out
/// of.
///
/// It is **not** a shell parser, and it does not pretend to be one. It knows
/// quoting, backslash escapes, comments, the separators `; | || & && newline`,
/// and redirections, because those are what tell a command from a mention. Every
/// other construct it can recognise but not model, a command substitution, a
/// here-document, a subshell, a shell group, a `case` arm terminator or an
/// input descriptor duplication, it **refuses**, and
/// `script_commands` turns that refusal into a panic. A test helper that cannot
/// read a script must fail the test, not guess at it: guessing is exactly how
/// `echo 'text; make gate COUPLE=0 ; more text'` came to be read as an
/// invocation of the gate target (114 D-17).
struct ScriptReader {
    src: Vec<char>,
    i: usize,
    cmds: Vec<ShellCommand>,
    cur: ShellCommand,
    /// The word being read, with its quoting already removed.
    word: String,
    /// A word is open, which an empty `""` also makes true.
    open: bool,
    /// Some part of the open word came from inside quotes or from an escape, so
    /// its characters are literal whatever they spell.
    quoted: bool,
    pending: Pending,
}

impl ScriptReader {
    fn new(script: &str) -> Self {
        Self {
            src: script.chars().collect(),
            i: 0,
            cmds: Vec::new(),
            cur: ShellCommand::default(),
            word: String::new(),
            open: false,
            quoted: false,
            pending: Pending::Word,
        }
    }

    fn at(&self, off: usize) -> Option<char> {
        self.src.get(self.i + off).copied()
    }

    fn finish_word(&mut self) -> Result<(), String> {
        if !self.open {
            return Ok(());
        }
        let w = std::mem::take(&mut self.word);
        self.open = false;
        let was_quoted = std::mem::replace(&mut self.quoted, false);
        if !was_quoted && (w == "{" || w == "}") {
            return Err(format!("a shell group (`{w}`) is not supported"));
        }
        match std::mem::replace(&mut self.pending, Pending::Word) {
            Pending::Word => self.cur.words.push(w),
            Pending::Out => self.cur.redirects.push(w),
            Pending::In => {}
        }
        Ok(())
    }

    fn finish_cmd(&mut self) -> Result<(), String> {
        self.finish_word()?;
        if self.pending != Pending::Word {
            return Err("a redirection with no target".to_string());
        }
        let c = std::mem::take(&mut self.cur);
        if !c.words.is_empty() || !c.redirects.is_empty() {
            self.cmds.push(c);
        }
        Ok(())
    }

    /// Open a redirection. A bare run of digits immediately before the operator
    /// is a file descriptor (`2> log`) and not a word of the command.
    fn open_redirect(&mut self, kind: Pending) -> Result<(), String> {
        if self.open
            && !self.quoted
            && !self.word.is_empty()
            && self.word.chars().all(|c| c.is_ascii_digit())
        {
            self.word.clear();
            self.open = false;
            // Already false, because the branch guard requires it. Reset beside
            // `open` anyway, so the word's state is cleared in one place rather
            // than left correct by a condition a reader has to re-derive.
            self.quoted = false;
        } else {
            self.finish_word()?;
        }
        if self.pending != Pending::Word {
            return Err("two redirection operators in a row".to_string());
        }
        self.pending = kind;
        Ok(())
    }

    /// Read the operand of an output descriptor duplication: the `2` of `>&2`,
    /// with `>&` already consumed.
    ///
    /// Two operands are supported, which are the two POSIX spells: a run of one
    /// or more ASCII digits duplicates that descriptor, and a bare `-` closes
    /// the redirected one. Neither names a file, so a well-formed duplication
    /// leaves the command exactly as it found it, and the redirection it opened
    /// takes no target word.
    ///
    /// Everything else is refused: an operand that is missing, one that carries
    /// a tail (`>&2abc`, `>&-2`), and the `>&word` redirect-to-file that only
    /// some shells accept. The operand is read to the next token boundary
    /// first, so a tail is seen rather than left behind as a word of the
    /// command. Consuming a run of zero descriptor characters as if it were
    /// `>&2` is what let `make gate COUPLE=0 >&`, a syntax error to every shell
    /// the kit runs under, read as a clean invocation of the gate target
    /// (114 D-18).
    fn output_duplication(&mut self) -> Result<(), String> {
        let mut operand = String::new();
        while let Some(c) = self.at(0) {
            if c.is_whitespace() || matches!(c, ';' | '|' | '&' | '<' | '>' | '(' | ')') {
                break;
            }
            operand.push(c);
            self.i += 1;
        }
        if operand.is_empty() {
            return Err("an output descriptor duplication (`>&`) with no descriptor".to_string());
        }
        if operand != "-" && !operand.chars().all(|c| c.is_ascii_digit()) {
            return Err(format!(
                "an output descriptor duplication operand (`{operand}`) that is neither a descriptor number nor `-`"
            ));
        }
        self.pending = Pending::Word;
        Ok(())
    }

    fn single_quoted(&mut self) -> Result<(), String> {
        self.i += 1;
        loop {
            match self.at(0) {
                None => return Err("an unterminated single quote".to_string()),
                Some('\'') => {
                    self.i += 1;
                    break;
                }
                Some(c) => {
                    self.word.push(c);
                    self.i += 1;
                }
            }
        }
        self.open = true;
        self.quoted = true;
        Ok(())
    }

    fn double_quoted(&mut self) -> Result<(), String> {
        self.i += 1;
        loop {
            match self.at(0) {
                None => return Err("an unterminated double quote".to_string()),
                Some('"') => {
                    self.i += 1;
                    break;
                }
                Some('`') => {
                    return Err("a backtick command substitution is not supported".to_string());
                }
                Some('$') if self.at(1) == Some('(') => {
                    return Err("a command substitution (`$(`) is not supported".to_string());
                }
                Some('\\') => match self.at(1) {
                    None => return Err("an unterminated double quote".to_string()),
                    Some('\n') => self.i += 2,
                    Some(n @ ('$' | '`' | '"' | '\\')) => {
                        self.word.push(n);
                        self.i += 2;
                    }
                    Some(_) => {
                        self.word.push('\\');
                        self.i += 1;
                    }
                },
                Some(c) => {
                    self.word.push(c);
                    self.i += 1;
                }
            }
        }
        self.open = true;
        self.quoted = true;
        Ok(())
    }

    fn parse(mut self) -> Result<Vec<ShellCommand>, String> {
        while let Some(c) = self.at(0) {
            match c {
                '\\' => match self.at(1) {
                    None => return Err("a trailing backslash".to_string()),
                    // A line continuation joins the two lines into one command.
                    Some('\n') => self.i += 2,
                    // Anything else is one literal character: `\;` is a
                    // semicolon the shell does not read as a separator.
                    Some(n) => {
                        self.word.push(n);
                        self.open = true;
                        self.quoted = true;
                        self.i += 2;
                    }
                },
                '\'' => self.single_quoted()?,
                '"' => self.double_quoted()?,
                '`' => return Err("a backtick command substitution is not supported".to_string()),
                '$' if self.at(1) == Some('(') => {
                    return Err("a command substitution (`$(`) is not supported".to_string());
                }
                '(' | ')' => {
                    return Err(format!(
                        "a subshell or process substitution (`{c}`) is not supported"
                    ));
                }
                // A `#` opens a comment only at the start of a word, so
                // `refs/heads#1` is one word and not a comment.
                '#' if !self.open => {
                    while matches!(self.at(0), Some(ch) if ch != '\n') {
                        self.i += 1;
                    }
                }
                ' ' | '\t' | '\r' => {
                    self.finish_word()?;
                    self.i += 1;
                }
                // `;;` terminates a `case` arm and means nothing outside one:
                // `/bin/sh` and `dash` both call `echo a;; echo b` a syntax
                // error. A `case` is already refused, because its arms carry
                // `)`; refusing the terminator too means the reader never
                // accepts a construct it has not modelled (114 D-17).
                ';' if self.at(1) == Some(';') => {
                    return Err("a `case` arm terminator (`;;`) is not supported".to_string());
                }
                '\n' | ';' => {
                    self.finish_cmd()?;
                    self.i += 1;
                }
                '|' => {
                    self.finish_cmd()?;
                    self.i += 1;
                    if self.at(0) == Some('|') {
                        self.i += 1;
                    }
                }
                '&' if self.at(1) == Some('>') => {
                    self.open_redirect(Pending::Out)?;
                    self.i += 2;
                    if self.at(0) == Some('>') {
                        self.i += 1;
                    }
                }
                '&' => {
                    self.finish_cmd()?;
                    self.i += 1;
                    if self.at(0) == Some('&') {
                        self.i += 1;
                    }
                }
                '>' => {
                    self.open_redirect(Pending::Out)?;
                    self.i += 1;
                    let appending = self.at(0) == Some('>');
                    if appending {
                        self.i += 1;
                    }
                    // `>&2` duplicates a descriptor; it names no file. Its
                    // operand decides that, so the operand is read and checked
                    // rather than assumed (114 D-18).
                    if self.at(0) == Some('&') {
                        if appending {
                            return Err(
                                "an appending descriptor duplication (`>>&`) is not supported"
                                    .to_string(),
                            );
                        }
                        self.i += 1;
                        self.output_duplication()?;
                    }
                }
                '<' if self.at(1) == Some('<') => {
                    return Err("a here-document (`<<`) is not supported".to_string());
                }
                '<' if self.at(1) == Some('(') => {
                    return Err("a process substitution (`<(`) is not supported".to_string());
                }
                // `>&2` is consumed above, because it names no file and changes
                // no command. `<&` is refused instead of mirrored: nothing here
                // needs it, and reaching it through the generic path reported
                // "a redirection with no target", which is not what is wrong.
                '<' if self.at(1) == Some('&') => {
                    return Err(
                        "an input descriptor duplication (`<&`) is not supported".to_string()
                    );
                }
                '<' => {
                    self.open_redirect(Pending::In)?;
                    self.i += 1;
                }
                ch => {
                    self.word.push(ch);
                    self.open = true;
                    self.i += 1;
                }
            }
        }
        self.finish_cmd()?;
        Ok(self.cmds)
    }
}

/// The commands a `run:` script executes: comments dropped, quoting removed, and
/// the script split at the separators a shell would split it at **and at no
/// others**. Whatever survives here is something the runner runs; a mention in a
/// comment, or inside a quoted string, does not.
///
/// This reads the workflow's SOURCE text. A `$RUNNER_TEMP` in it is the literal
/// eight characters, never the runner's expansion of them, so what a variable
/// expands to at job time cannot change how a step tokenises here. GitHub's own
/// `${{ … }}` expressions ARE substituted before the shell sees them, which is
/// why they are masked to one word first.
///
/// A script using a form [`ScriptReader`] does not model is a panic, not a
/// guess: the shipped workflow uses none of them, and a future one that does
/// must be read by something that understands it rather than mis-read by this.
fn script_commands(run: &str) -> Vec<ShellCommand> {
    let masked = mask_expressions(run);
    ScriptReader::new(&masked).parse().unwrap_or_else(|e| {
        panic!("a `run:` script uses a shell form this reader does not support ({e}), so it is refused rather than guessed at: {run}")
    })
}

/// The files a `run:` script's own commands redirect output into.
fn redirect_targets(run: &str) -> BTreeSet<String> {
    script_commands(run)
        .into_iter()
        .flat_map(|c| c.redirects)
        .collect()
}

/// The `make` targets a `run:` script actually invokes, and the variables each
/// invocation sets.
///
/// A target named in a comment, echoed as text, or written into the step's
/// `name:` is not an invocation: only the words of a command whose head is
/// `make` count. That distinction is the whole of §3.4, and 114 D-10 is why it
/// is exercised on fixtures rather than assumed.
fn make_invocations(run: &str) -> Vec<(Vec<String>, BTreeMap<String, String>)> {
    let mut out = Vec::new();
    for cmd in script_commands(run) {
        let mut toks = cmd.words.iter().map(String::as_str);
        let Some(head) = toks.next() else { continue };
        if head != "make" && !head.ends_with("/make") {
            continue;
        }
        let (mut targets, mut vars) = (Vec::new(), BTreeMap::new());
        let mut skip_next = false;
        for t in toks {
            if skip_next {
                skip_next = false;
                continue;
            }
            if t == "-f" || t == "-C" {
                skip_next = true;
                continue;
            }
            if t.starts_with('-') {
                continue;
            }
            match t.split_once('=') {
                // The reader has already removed the quoting, so
                // `PR_BODY="$RUNNER_TEMP/pr-body.txt"` arrives as its path.
                Some((k, v)) => {
                    vars.insert(k.to_string(), v.to_string());
                }
                None => targets.push(t.to_string()),
            }
        }
        out.push((targets, vars));
    }
    out
}

/// The `make gate` invocation of a step, if the step has one.
fn gate_invocation(step: &WorkflowStep) -> Option<BTreeMap<String, String>> {
    let run = step.run.as_deref()?;
    make_invocations(run)
        .into_iter()
        .find(|(targets, _)| targets.iter().any(|t| t == "gate"))
        .map(|(_, vars)| vars)
}

/// The `spec-spine` verbs a `run:` script invokes, as `verb` or `verb subverb`.
fn spec_spine_verbs(run: &str) -> Vec<String> {
    let mut out = Vec::new();
    for cmd in script_commands(run) {
        let mut toks = cmd.words.iter().map(String::as_str);
        let Some(head) = toks.next() else { continue };
        if head != "spec-spine" && !head.ends_with("/spec-spine") {
            continue;
        }
        let Some(verb) = toks.next() else { continue };
        let mut v = verb.to_string();
        if let Some(sub) = toks.next() {
            if !sub.starts_with('-') {
                v.push(' ');
                v.push_str(sub);
            }
        }
        out.push(v);
    }
    out
}

/// The verbs the one gate definition runs, read off `Makefile`'s `gate`
/// target rather than listed here. A step added to the target is a step the
/// workflow may not restate, without anyone remembering to extend a constant.
fn one_gate_definition_verbs() -> BTreeSet<String> {
    invocations(&target_body(&read("Makefile"), "gate"))
        .iter()
        .map(|c| {
            let mut toks = c.split_whitespace();
            let verb = toks.next().unwrap_or("").to_string();
            match toks.next() {
                Some(sub) if !sub.starts_with('-') => format!("{verb} {sub}"),
                _ => verb,
            }
        })
        .collect()
}

/// The steps of `.github/workflows/ci.yml` running on one event leg.
fn legs_of(steps: &[WorkflowStep], want: Leg) -> Vec<&WorkflowStep> {
    steps
        .iter()
        .filter(|s| leg(s.cond.as_deref()) == Some(want))
        .collect()
}

/// §3.4: both legs reach the chain by calling the one gate definition. Each leg
/// is found by its event condition, which is the only place a workflow says what
/// it runs on.
#[test]
fn both_workflow_legs_invoke_the_one_gate_definition() {
    let steps = workflow_steps(&read(".github/workflows/ci.yml"));
    assert!(
        !steps.is_empty(),
        ".github/workflows/ci.yml parsed to no steps, so this test asserts nothing"
    );
    for want in [Leg::Push, Leg::PullRequest] {
        let calling: Vec<_> = legs_of(&steps, want)
            .into_iter()
            .filter(|s| gate_invocation(s).is_some())
            .collect();
        assert_eq!(
            calling.len(),
            1,
            "expected exactly one {want:?} step invoking the `gate` target, got {}. \
             The workflow's own header says the gate has one definition; a leg that \
             restates the chain instead is the drift spec 114 §3.4 closes. Steps: {:#?}",
            calling.len(),
            steps
        );
    }
}

/// §3.4: and no step restates a verb that definition already runs. The header
/// comment is not a step, and a verb named in one is not an invocation.
#[test]
fn no_workflow_step_restates_a_verb_the_one_gate_definition_runs() {
    let chain = one_gate_definition_verbs();
    assert!(
        chain.contains("couple") && chain.contains("check"),
        "the gate target's verbs parsed as {chain:?}, which is not the chain"
    );
    let mut restated: Vec<String> = Vec::new();
    for step in workflow_steps(&read(".github/workflows/ci.yml")) {
        let Some(run) = step.run.as_deref() else {
            continue;
        };
        for v in spec_spine_verbs(run) {
            if chain.contains(&v) {
                restated.push(format!("{:?} runs `spec-spine {v}`", step.name));
            }
        }
    }
    assert!(
        restated.is_empty(),
        "the workflow restates the chain instead of calling the one definition: {restated:?}"
    );
}

/// §3.3 + §3.4: the two legs differ by an argument, not by a second chain. The
/// push leg turns coupling off, because spec 064 §3.2 says `couple` runs on
/// `pull_request` only; the pull-request leg leaves it on and hands over the
/// body file. Neither is inferred from the other: the control is explicit.
#[test]
fn the_one_gate_definition_serves_both_legs_through_explicit_controls() {
    let steps = workflow_steps(&read(".github/workflows/ci.yml"));

    let push = legs_of(&steps, Leg::Push)
        .into_iter()
        .find_map(gate_invocation)
        .expect("a push step invoking the gate target");
    assert_eq!(
        push.get("COUPLE").map(String::as_str),
        Some("0"),
        "spec 064 §3.2: the shipped workflow runs `couple` on pull_request only, \
         so the push leg must spend the explicit control: {push:?}"
    );

    let pr = legs_of(&steps, Leg::PullRequest)
        .into_iter()
        .find_map(gate_invocation)
        .expect("a pull-request step invoking the gate target");
    assert_ne!(
        pr.get("COUPLE").map(String::as_str),
        Some("0"),
        "the pull-request leg is the one that must couple: {pr:?}"
    );
    let body = pr
        .get("PR_BODY")
        .expect("the pull-request leg hands the body file to the gate");
    assert!(
        !body.is_empty(),
        "the body reaches the gate as a path: {pr:?}"
    );

    // And it is the path the step just WROTE, not merely a path-shaped string.
    // Tied to the step's own redirect rather than to the file's name, so
    // renaming `pr-body.txt` cannot quietly turn this into an assertion about a
    // filename; what spec 064 §3.2 requires is that the body travel as a file
    // and that the gate be handed that file.
    //
    // Read off the parsed commands rather than searched for in the text. `> "x"`,
    // `>"x"` and `1> "x"` all write the same file and all reduce to the same
    // target here, while a `>` inside a quoted string writes nothing and is not
    // one (114 D-17).
    let run = legs_of(&steps, Leg::PullRequest)
        .into_iter()
        .find(|s| gate_invocation(s).is_some())
        .and_then(|s| s.run.clone())
        .expect("the pull-request step has a script");
    let written = redirect_targets(&run);
    assert!(
        written.contains(body),
        "PR_BODY must name a file this step writes; it writes {written:?}: {run}"
    );
}

/// §3.2: the ownership guard reads the effective configuration, and reads it in
/// a form whose failure is a failure. `config show | grep -q` would report
/// grep's status and discard the read's, so every way the read can fail would
/// produce an honest-sounding skip and a green gate (114 D-12).
#[test]
fn the_gate_reads_the_effective_config_before_asserting_ownership() {
    let gate = target_body(&read("Makefile"), "gate");
    assert!(
        invocations(&gate).iter().any(|c| c == "config show"),
        "the guard must read the effective config through the CLI: {gate}"
    );
    assert!(
        gate.contains("require_ownership"),
        "on the condition AGENTS.md already publishes: {gate}"
    );
    for line in gate.lines() {
        assert!(
            !(line.contains("config show") && line.contains("| grep")),
            "the probe must not be a pipeline, whose status is grep's: {line}"
        );
    }
}

/// §3.4 + D-10: the detector refuses a mention that is not an invocation, in
/// each of the three forms a mention arrives in. Built as fixtures so the
/// refusals are exercised on every run, rather than shipping a broken workflow
/// to prove them.
#[test]
fn the_one_gate_definition_detector_refuses_a_mention_that_is_not_an_invocation() {
    // A positive control first, so a detector that answered "no" to everything
    // could not pass this test.
    let real = r#"
jobs:
  govern:
    steps:
      - name: Governed loop
        if: github.event_name != 'pull_request'
        run: make gate BASE=origin/${{ github.base_ref || 'main' }} COUPLE=0
"#;
    let steps = workflow_steps(real);
    let vars = legs_of(&steps, Leg::Push)
        .into_iter()
        .find_map(gate_invocation)
        .expect("a real invocation is detected");
    assert_eq!(vars.get("COUPLE").map(String::as_str), Some("0"));
    assert_eq!(
        vars.get("BASE").map(String::as_str),
        Some("origin/GITHUB_EXPR"),
        "the `||` inside a GitHub expression is not a command separator"
    );

    // 1. The step's NAME says `make gate`; the script restates the chain.
    let named = r#"
jobs:
  govern:
    steps:
      - name: Governed loop (make gate)
        if: github.event_name != 'pull_request'
        run: spec-spine check --fail-on-unresolved --fail-on-warn
"#;
    let steps = workflow_steps(named);
    assert!(
        legs_of(&steps, Leg::Push)
            .into_iter()
            .all(|s| gate_invocation(s).is_none()),
        "a target named in a step's `name:` is not an invocation of it"
    );
    assert_eq!(
        spec_spine_verbs(steps[0].run.as_deref().unwrap()),
        vec!["check".to_string()],
        "and the restatement it hides is still counted"
    );

    // 2. A COMMENT inside the script says `make gate`.
    let commented = r#"
jobs:
  govern:
    steps:
      - name: Governed loop (pull request)
        if: github.event_name == 'pull_request'
        run: |
          # make gate would run the whole chain here
          spec-spine lint --fail-on-warn
"#;
    let steps = workflow_steps(commented);
    assert!(
        legs_of(&steps, Leg::PullRequest)
            .into_iter()
            .all(|s| gate_invocation(s).is_none()),
        "a target named in a script comment is not an invocation of it"
    );

    // 3. The script ECHOES the words.
    let echoed = r#"
jobs:
  govern:
    steps:
      - name: Governed loop (pull request)
        if: github.event_name == 'pull_request'
        run: |
          echo "this job runs make gate"
          spec-spine couple --base origin/main --head HEAD
"#;
    let steps = workflow_steps(echoed);
    assert!(
        legs_of(&steps, Leg::PullRequest)
            .into_iter()
            .all(|s| gate_invocation(s).is_none()),
        "a target echoed as text is not an invocation of it"
    );
    assert_eq!(
        spec_spine_verbs(steps[0].run.as_deref().unwrap()),
        vec!["couple".to_string()]
    );

    // 4. A leg is its CONDITION, not its name: a step called "(pull request)"
    //    that runs on a push is a push step.
    let mislabelled = r#"
jobs:
  govern:
    steps:
      - name: Governed loop (pull request)
        if: github.event_name != 'pull_request'
        run: make gate COUPLE=0
"#;
    let steps = workflow_steps(mislabelled);
    assert!(
        legs_of(&steps, Leg::PullRequest).is_empty(),
        "the leg is read from the event condition, not from the step's name"
    );
    assert_eq!(legs_of(&steps, Leg::Push).len(), 1);
}

/// §3.4 + D-17: the detector reads the script's shell quoting, so text a command
/// carries cannot manufacture a command. Splitting on `;`, `|` and `&` wherever
/// they appeared was the defect: it turned the single `echo` below into three
/// commands, the middle one an invocation of the gate target with `COUPLE=0`,
/// from a step that runs no `make` at all.
///
/// Quoted text is the same class of mention as a comment or a step `name:`, and
/// §3.4 already refuses those. This is that requirement holding for the form
/// that got past it.
#[test]
fn the_one_gate_definition_detector_reads_shell_quoting_not_raw_separators() {
    // Positive controls first, one per separator, so a detector that answered
    // "no" to every fixture below could not pass this test. Each of these is a
    // real invocation that follows a real separator.
    for real in [
        "echo hi ; make gate COUPLE=0",
        "echo hi && make gate COUPLE=0",
        "echo hi | cat ; make gate COUPLE=0",
        "echo hi\nmake gate COUPLE=0",
        "make gate COUPLE=0 ; echo done",
    ] {
        let step = WorkflowStep {
            name: None,
            cond: Some("github.event_name != 'pull_request'".to_string()),
            run: Some(real.to_string()),
        };
        assert_eq!(
            gate_invocation(&step).and_then(|v| v.get("COUPLE").cloned()),
            Some("0".to_string()),
            "a real invocation after a real separator is still detected: {real}"
        );
    }

    // And the same text, quoted. Every one of these is a single `echo`.
    let mentions = [
        // The reported reproduction, single-quoted.
        "echo 'text; make gate COUPLE=0 ; more text'",
        // Double-quoted, which the same split treated the same way.
        "echo \"text; make gate COUPLE=0 ; more text\"",
        // The other two separators, in both quotings.
        "echo 'a | make gate COUPLE=0 | b'",
        "echo \"a && make gate COUPLE=0 && b\"",
        // A separator escaped rather than quoted is also not a separator.
        r"echo text\; make gate COUPLE=0",
        // Quoted inside a longer, otherwise ordinary script.
        "set -e\necho 'ran: make gate COUPLE=0; ok'\nspec-spine lint --fail-on-warn",
    ];
    for run in mentions {
        let step = WorkflowStep {
            name: None,
            cond: Some("github.event_name != 'pull_request'".to_string()),
            run: Some(run.to_string()),
        };
        assert_eq!(
            gate_invocation(&step),
            None,
            "quoted or escaped text is not an invocation of the target: {run}"
        );
    }

    // The same fixture through the whole §3.4 path, parsed from YAML and read by
    // leg, because that is where the property is asserted and a helper can be
    // right while the path through it is not.
    let quoted = r#"
jobs:
  govern:
    steps:
      - name: Governed loop
        if: github.event_name != 'pull_request'
        run: |
          echo 'text; make gate COUPLE=0 ; more text'
      - name: Governed loop (pull request)
        if: github.event_name == 'pull_request'
        run: |
          echo "text; make gate PR_BODY=/tmp/pr-body.txt ; more text"
"#;
    let steps = workflow_steps(quoted);
    assert_eq!(steps.len(), 2, "the fixture parsed to no steps");
    for want in [Leg::Push, Leg::PullRequest] {
        assert!(
            legs_of(&steps, want)
                .into_iter()
                .all(|s| gate_invocation(s).is_none()),
            "a {want:?} step that only echoes the target does not invoke it"
        );
    }

    // The restatement detector shares the reader, so it shares the correction:
    // a quoted verb was counted as a restatement, which would have refused a
    // workflow that delegates correctly.
    assert!(
        spec_spine_verbs("echo 'a; spec-spine check --fail-on-unresolved --fail-on-warn'")
            .is_empty(),
        "a verb inside a quoted string is not an invocation of it"
    );
    assert_eq!(
        spec_spine_verbs("echo 'a; spec-spine check' ; spec-spine index coverage"),
        vec!["index coverage".to_string()],
        "and the real invocation beside it is still counted"
    );

    // And so does the PR-body redirection: a `>` inside quotes writes no file.
    assert!(
        redirect_targets("echo \"wrote > $RUNNER_TEMP/pr-body.txt\"").is_empty(),
        "a redirection operator inside a quoted string is text, not a redirect"
    );
    assert!(
        redirect_targets("printf '%s' \"$PR_BODY_TEXT\" > \"$RUNNER_TEMP/pr-body.txt\"")
            .contains("$RUNNER_TEMP/pr-body.txt"),
        "and a real redirect still names the file it writes"
    );
    // Spelled three ways, one target: the assertion is about which file is
    // written, not about a workflow's spacing.
    for spelling in [
        "printf x > out.txt",
        "printf x >out.txt",
        "printf x 1> out.txt",
    ] {
        assert!(
            redirect_targets(spelling).contains("out.txt"),
            "redirect spelling changes nothing: {spelling}"
        );
    }
}

/// §3.4 + D-17: and a script the reader cannot model is refused, not guessed at.
///
/// The reader knows quoting, escapes, comments, separators and redirections. A
/// command substitution or a here-document can carry a command it would read as
/// text, so the honest answer is that the script was not understood. A helper
/// that guessed here would be the same defect one construct further along.
#[test]
fn the_one_gate_definition_detector_refuses_a_script_it_cannot_read() {
    for (run, want) in [
        ("echo $(make gate)", "command substitution"),
        ("echo `make gate`", "backtick"),
        ("echo \"$(make gate)\"", "command substitution"),
        ("( make gate )", "subshell"),
        ("{ make gate; }", "shell group"),
        ("cat <<EOF\nmake gate\nEOF", "here-document"),
        // `;;` is a syntax error outside a `case`, and a `case` is refused by
        // its `)`. Refusing both means nothing unmodelled is accepted.
        ("echo a;; echo b", "`;;`"),
        // `>&2` is consumed; its input twin is refused, and the refusal says so
        // rather than reporting a missing target.
        ("cat 0<&1", "`<&`"),
        ("diff <(make gate) b", "process substitution"),
        ("echo 'unterminated", "unterminated single quote"),
        ("echo \"unterminated", "unterminated double quote"),
        ("make gate >", "redirection with no target"),
    ] {
        let err = ScriptReader::new(run)
            .parse()
            .expect_err(&format!("`{run}` must be refused, not parsed"));
        assert!(
            err.contains(want),
            "`{run}` was refused as {err:?}, which does not name {want}"
        );
    }

    // And the refusal reaches the caller as a failure rather than an empty
    // answer, which would read as "this step invokes nothing".
    let caught = std::panic::catch_unwind(|| script_commands("echo $(make gate)"));
    assert!(
        caught.is_err(),
        "script_commands must fail on a script it cannot read, not return {caught:?}"
    );

    // The shipped workflow uses none of those forms, so the refusal costs it
    // nothing: every one of its steps reads.
    for step in workflow_steps(&read(".github/workflows/ci.yml")) {
        if let Some(run) = step.run.as_deref() {
            ScriptReader::new(&mask_expressions(run))
                .parse()
                .unwrap_or_else(|e| {
                    panic!(
                        ".github/workflows/ci.yml step {:?} does not read: {e}",
                        step.name
                    )
                });
        }
    }
}

/// §3.4 + D-18: an output descriptor duplication is read by its operand, and an
/// operand the reader does not model is refused rather than dropped.
///
/// `>&` was consumed as "duplicates a descriptor, names no file" before its
/// operand was looked at, so a run of **zero** descriptor characters cleared the
/// pending redirection just as `>&2` does. `make gate COUPLE=0 >&` is a syntax
/// error to `/bin/sh`, `dash` and `zsh` alike, and the reader read it as a
/// clean invocation of the gate target: a script no runner can run satisfied the
/// assertion that a leg invokes the one gate definition.
///
/// The supported operands are the two POSIX spells: a run of one or more ASCII
/// digits (`>&2`, duplicate that descriptor) and exactly `-` (`>&-`, close).
/// Everything else, missing, malformed, or a form only some shells accept, is
/// refused.
#[test]
fn the_one_gate_definition_detector_refuses_an_unmodelled_descriptor_duplication() {
    // A positive control first, in both supported spells and against both
    // adjacent boundaries, so a reader that refused everything could not pass.
    for supported in [
        "echo x >&2",
        "echo x >&2\n",
        "echo x >&2 ; make gate COUPLE=0",
        "echo x >&2;make gate COUPLE=0",
        "echo x >&2|cat",
        "echo x 2>&1",
        "echo x 2>&1 ; make gate COUPLE=0",
        "echo x >&10",
        "echo x >&-",
        "echo x >&- ; make gate COUPLE=0",
    ] {
        let cmds = ScriptReader::new(supported).parse().unwrap_or_else(|e| {
            panic!("a supported duplication must still read: {supported}: {e}")
        });
        assert!(
            cmds.iter().all(|c| c.redirects.is_empty()),
            "a duplication names no file: {supported} read as {cmds:?}"
        );
        assert_eq!(
            cmds[0].words,
            vec!["echo".to_string(), "x".to_string()],
            "and it is not a word of the command either: {supported}"
        );
    }

    // The reported reproduction, plus the malformed and unsupported operands
    // beside it. Each names what is wrong with it rather than reporting a
    // missing target, which is a different defect.
    for (run, want) in [
        // Missing: the operand runs out at end of input, at a newline, and at a
        // separator. The first is the reproduction; the others are the same
        // hole one token boundary along, so the fix is not an end-of-input
        // special case.
        ("make gate COUPLE=0 >&", "no descriptor"),
        ("make gate COUPLE=0 >&\necho done", "no descriptor"),
        ("make gate COUPLE=0 >& ; echo done", "no descriptor"),
        ("make gate COUPLE=0 >&| cat", "no descriptor"),
        // A space before the operand is `>&word`, which only some shells read
        // as a redirection to a file. It is not modelled either way.
        ("make gate COUPLE=0 >& out.txt", "no descriptor"),
        // Malformed: digits with a tail, `-` with a tail, and a bare word.
        ("make gate COUPLE=0 >&2abc", "`2abc`"),
        ("make gate COUPLE=0 >&-2", "`-2`"),
        ("make gate COUPLE=0 >&2-", "`2-`"),
        ("make gate COUPLE=0 >&out.txt", "`out.txt`"),
        // `>>&` is a syntax error to `dash` and to `bash` alike; the reader
        // accepted it because it reached the same `&` branch as `>&`.
        ("make gate COUPLE=0 >>&2", "`>>&`"),
    ] {
        let err = ScriptReader::new(run)
            .parse()
            .expect_err(&format!("`{run}` must be refused, not parsed"));
        assert!(
            err.contains(want),
            "`{run}` was refused as {err:?}, which does not name {want}"
        );
    }

    // And through the whole §3.4 path, parsed from YAML and read by leg: the
    // refusal has to reach the assertion that a leg invokes the gate, not just
    // the helper under it. `script_commands` turns the refusal into a panic, so
    // the step answers neither "invokes" nor "does not invoke".
    let unrunnable = r#"
jobs:
  govern:
    steps:
      - name: Governed loop
        if: github.event_name != 'pull_request'
        run: |
          make gate COUPLE=0 >&
"#;
    let steps = workflow_steps(unrunnable);
    assert_eq!(steps.len(), 1, "the fixture parsed to no steps");
    assert_eq!(
        legs_of(&steps, Leg::Push).len(),
        1,
        "the fixture's step is a push step"
    );
    let caught = std::panic::catch_unwind(move || {
        legs_of(&steps, Leg::Push)
            .into_iter()
            .find_map(gate_invocation)
    });
    assert!(
        caught.is_err(),
        "a script no shell can run must not satisfy the invocation assertion; it answered {caught:?}"
    );
}

/// Spec 115 §3.1, held through spec 120 §3.3: no file the scaffold produces
/// carries a line the claim scanner recognizes as a claim attempt.
///
/// 115 measured this against the kit's shell scripts, whose `# Spec:` headers
/// named ids that exist in no adopter's corpus and shadowed a valid header
/// added below them. The kit is gone and the scaffold writes no scripts; the
/// property is asserted over what it does write, because the failure mode is a
/// property of delivered bytes and not of the kit.
///
/// The recognizer is applied rather than imitated: `near_miss_headers_in` is
/// the scanner's own reader, and against an **empty** id set nothing can
/// resolve, so every attempt `index.rs::header_attempt` recognizes is reported
/// as `unknown-spec`. A substring search for the claim token is not the same
/// question (spec 094 §1 measured eleven files against the recognizer's one),
/// and it would also hit the provenance wording §3.2 keeps.
///
/// `//!` lines are `doc-comment-marker` misses, not attempts, and are not
/// asserted on here: they claim nothing in any corpus.
#[test]
fn no_claim_header_reaches_an_adopters_tree() {
    use spec_spine_types::NearMissReason;

    let files = scaffold_init(&Config::default()).unwrap().files;
    assert!(
        files.len() >= 7,
        "the scaffold produced {} files; the assertion below would be vacuous",
        files.len()
    );
    let empty = BTreeSet::new();
    let mut offenders = Vec::new();
    for f in &files {
        for miss in spec_spine_core::index::near_miss_headers_in(&f.rel_path, &f.contents, &empty) {
            if miss.reason == NearMissReason::UnknownSpec {
                offenders.push(format!("{}:{}", f.rel_path, miss.line));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "these delivered files carry a claim header that resolves in no adopter's corpus, \
         and shadow a valid one added below it (spec 115 §1.3): {offenders:?}"
    );

    // The positive control: the reader above is the thing under test, so a run
    // that reported nothing has to be shown capable of reporting. The bytes are
    // the header this spec removed, and the assertion is that this very call
    // still names it. Without this line an empty `offenders` is also what a
    // reader that stopped recognizing claim headers would produce, which is the
    // vacuous pass spec 106 D-7 names.
    let control = "#!/usr/bin/env bash\n# Spec: specs/090-a-hook-bound-to-a-tool-route-misses-the-work/spec.md\n";
    let seen = spec_spine_core::index::near_miss_headers_in("control.sh", control, &empty);
    assert!(
        seen.iter().any(|m| m.reason == NearMissReason::UnknownSpec),
        "the recognizer reported nothing for a file that carries the header this spec \
         removed, so the assertion above could not have failed either: {seen:?}"
    );
}
