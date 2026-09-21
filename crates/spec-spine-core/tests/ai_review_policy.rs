// Spec: specs/091-an-unclassified-review-failure-blocks-the-merge/spec.md
//! Spec 091: an unclassified review failure blocks the merge.
//!
//! The AI review job's failure classifier used to fall through to a pass: a
//! non-zero invocation whose diagnostics matched no auth token was treated as
//! a provider outage, given a warning, and let through. On PR #267 an
//! organization-access refusal arrived as prose, matched nothing, and a PR
//! merged with no review. Spec 091 inverts the default and this file is what
//! holds the inversion in place.
//!
//! The rule of this suite (091 3.8): it runs **the workflow's own `run:`
//! scalars**, never a copy of their patterns. A test that rebuilt the
//! classifier would assert a property of its own copy, drift from the file
//! that actually runs in CI, and could not observe the three things this
//! defect lives in: the step's exit status, its outputs, and whether
//! publication happened.
//!
//! It is deliberately **not** a GitHub Actions emulator (091 3.8 rule 5,
//! D-11). It supports the `${{ }}` and `if:` forms this one job contains and
//! panics, naming the step and the expression, on anything else. A silent
//! default here would let a new condition go unevaluated while the suite
//! stayed green.
//!
//! Hermetic by construction: `claude` and `gh` are stubs on a `PATH` this
//! harness controls, every path is inside a per-case `TempDir`, the only
//! credential-shaped value is a literal that is not a credential, and no case
//! makes a provider, GitHub or network call.

use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The file under test. 119 AC-2: the suite names the real workflow.
const WORKFLOW: &str = ".github/workflows/ai-pr-review.yml";
const JOB: &str = "ai-review";
const CLASSIFY_STEP: &str = "Run AI Review";

/// A literal shaped like a token and holding no secret. The class 2 fixture
/// uses the empty string, which is what Actions puts in the environment for an
/// unset secret.
const FIXTURE_TOKEN: &str = "fixture-not-a-real-token";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

// ---------------------------------------------------------------------------
// Workflow model
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct Step {
    name: String,
    id: Option<String>,
    cond: Option<String>,
    run: Option<String>,
    shell: Option<String>,
    env: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
struct Job {
    env: BTreeMap<String, String>,
    steps: Vec<Step>,
}

fn scalar_map(v: Option<&serde_yaml::Value>) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let Some(m) = v.and_then(|v| v.as_mapping()) else {
        return out;
    };
    for (k, val) in m {
        let (Some(k), Some(val)) = (k.as_str(), val.as_str()) else {
            continue;
        };
        out.insert(k.to_string(), val.to_string());
    }
    out
}

fn parse_job() -> Job {
    let path = repo_root().join(WORKFLOW);
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let doc: serde_yaml::Value =
        serde_yaml::from_str(&text).unwrap_or_else(|e| panic!("{WORKFLOW} parses as YAML: {e}"));
    let job = doc
        .get("jobs")
        .and_then(|j| j.get(JOB))
        .unwrap_or_else(|| panic!("{WORKFLOW} declares a `{JOB}` job"));
    let steps: Vec<Step> = job
        .get("steps")
        .and_then(|s| s.as_sequence())
        .unwrap_or_else(|| panic!("the `{JOB}` job declares steps"))
        .iter()
        .map(|s| Step {
            name: s
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("<unnamed>")
                .to_string(),
            id: s.get("id").and_then(|v| v.as_str()).map(str::to_string),
            cond: s.get("if").and_then(|v| v.as_str()).map(str::to_string),
            run: s.get("run").and_then(|v| v.as_str()).map(str::to_string),
            shell: s.get("shell").and_then(|v| v.as_str()).map(str::to_string),
            env: scalar_map(s.get("env")),
        })
        .collect();
    assert!(
        steps.iter().any(|s| s.name == CLASSIFY_STEP),
        "{WORKFLOW} no longer has a `{CLASSIFY_STEP}` step, so this suite asserts nothing"
    );
    Job {
        env: scalar_map(doc.get("env")),
        steps,
    }
}

fn step_index(job: &Job, name: &str) -> usize {
    job.steps
        .iter()
        .position(|s| s.name == name)
        .unwrap_or_else(|| panic!("{WORKFLOW} has a step named {name:?}"))
}

/// The steps a fixture can reach: the classifier and everything after it. The
/// earlier steps set up the diff and are not part of the classification policy.
fn publication_steps(job: &Job) -> Vec<Step> {
    let i = step_index(job, CLASSIFY_STEP);
    job.steps[i + 1..]
        .iter()
        .filter(|s| s.run.is_some())
        .cloned()
        .collect()
}

// ---------------------------------------------------------------------------
// Expression resolution (091 3.8 rules 4 and 5): bounded, and loud past the bound
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Clone)]
struct Ctx {
    /// `github.*` paths, keyed without the `github.` prefix.
    github: BTreeMap<String, String>,
    secrets: BTreeMap<String, String>,
    /// step id -> that step's actual outputs, as written to `GITHUB_OUTPUT`.
    outputs: BTreeMap<String, BTreeMap<String, String>>,
    /// Whether every step so far succeeded, which is what `success()` reports
    /// and what a condition carrying no status function implies.
    success: bool,
}

impl Ctx {
    fn output(&self, step_id: &str, name: &str) -> String {
        // An output a step did not write is the empty string in Actions. That
        // is not an unsupported form, it is the unset case the policy relies
        // on: a blocked classification writes no `review_status`.
        self.outputs
            .get(step_id)
            .and_then(|m| m.get(name))
            .cloned()
            .unwrap_or_default()
    }

    /// One `${{ }}` expression, in the forms this job contains.
    fn resolve(&self, expr: &str, whence: &str) -> String {
        let e = expr.trim();
        if let Some(name) = e.strip_prefix("secrets.") {
            return self.secrets.get(name).cloned().unwrap_or_else(|| {
                panic!(
                    "{whence}: no fixture value for secret {name:?}. Add one rather than guessing."
                )
            });
        }
        if let Some(path) = e.strip_prefix("github.") {
            return self.github.get(path).cloned().unwrap_or_else(|| {
                panic!(
                    "{whence}: no fixture value for `github.{path}`. Add one rather than guessing."
                )
            });
        }
        if let Some(rest) = e.strip_prefix("steps.") {
            let parts: Vec<&str> = rest.split('.').collect();
            if parts.len() == 3 && parts[1] == "outputs" {
                return self.output(parts[0], parts[2]);
            }
        }
        panic!(
            "{whence}: unsupported expression `${{{{ {e} }}}}`. This harness is bounded to the \
             forms the ai-review job contains (spec 091 3.8 rule 5); it refuses rather than \
             guessing, and it is not to be grown into an Actions emulator."
        );
    }

    /// A whole `env:` value: literal text with zero or more `${{ }}` spans.
    fn expand(&self, raw: &str, whence: &str) -> String {
        let mut out = String::new();
        let mut rest = raw;
        while let Some(start) = rest.find("${{") {
            out.push_str(&rest[..start]);
            let after = &rest[start + 3..];
            let end = after.unwrap_or_panic_find("}}", whence);
            out.push_str(&self.resolve(&after[..end], whence));
            rest = &after[end + 2..];
        }
        out.push_str(rest);
        out
    }
}

trait FindOrPanic {
    fn unwrap_or_panic_find(&self, needle: &str, whence: &str) -> usize;
}
impl FindOrPanic for str {
    fn unwrap_or_panic_find(&self, needle: &str, whence: &str) -> usize {
        self.find(needle)
            .unwrap_or_else(|| panic!("{whence}: unterminated `${{{{` expression"))
    }
}

const STATUS_FNS: [&str; 4] = ["success()", "failure()", "always()", "cancelled()"];

/// A step's `if:`, in the forms this job contains: `success()` and
/// `<operand> ==|!= '<literal>'`, joined by `&&`.
///
/// A condition that names no status function carries an implied `success()`,
/// which is Actions' own rule and is load-bearing here: the transient notice's
/// condition tests only `review_status`, and it must not run after a
/// classification step that failed.
fn eval_if(step: &Step, ctx: &Ctx) -> bool {
    let Some(cond) = step.cond.as_deref() else {
        return ctx.success;
    };
    let whence = format!("{WORKFLOW} step {:?} `if:`", step.name);
    let has_status_fn = STATUS_FNS.iter().any(|f| cond.contains(f));
    let mut value = if has_status_fn { true } else { ctx.success };
    for conjunct in cond.split("&&") {
        let c = conjunct.trim();
        if c.is_empty() {
            panic!("{whence}: empty conjunct in {cond:?}");
        }
        if c.contains("||") || c.contains('(') && !STATUS_FNS.contains(&c) {
            panic!(
                "{whence}: unsupported form {c:?}. Supported: `success()` and \
                 `steps.<id>.outputs.<name>` or `github.<path>` compared with ==/!= to a \
                 single-quoted literal, joined by `&&` (spec 091 3.8 rule 5)."
            );
        }
        value &= eval_conjunct(c, ctx, &whence);
    }
    value
}

fn eval_conjunct(c: &str, ctx: &Ctx, whence: &str) -> bool {
    match c {
        "success()" => return ctx.success,
        "always()" => return true,
        "failure()" | "cancelled()" => {
            panic!(
                "{whence}: {c} is not modelled by this harness; add it deliberately or remove it"
            )
        }
        _ => {}
    }
    let (operand, want_eq, literal) = if let Some((l, r)) = c.split_once("==") {
        (l.trim(), true, r.trim())
    } else if let Some((l, r)) = c.split_once("!=") {
        (l.trim(), false, r.trim())
    } else {
        panic!(
            "{whence}: unsupported form {c:?}; this harness evaluates only ==/!= comparisons \
             against a single-quoted literal (spec 091 3.8 rule 5)."
        );
    };
    let literal = literal
        .strip_prefix('\'')
        .and_then(|l| l.strip_suffix('\''))
        .unwrap_or_else(|| panic!("{whence}: right-hand side of {c:?} is not a quoted literal"));
    let actual = ctx.resolve(operand, whence);
    (actual == literal) == want_eq
}

// ---------------------------------------------------------------------------
// Execution under the runner's own shell semantics (091 3.8 rule 3)
// ---------------------------------------------------------------------------

struct Ran {
    status: i32,
    stdout: String,
    outputs: BTreeMap<String, String>,
}

impl Ran {
    fn annotations(&self) -> (bool, bool) {
        (
            self.stdout.contains("::error::"),
            self.stdout.contains("::warning::"),
        )
    }
}

/// The flags the runner applies. A GitHub-hosted Linux step declaring no
/// `shell:` runs as `bash -e {0}`; `shell: bash` adds `-o pipefail`. The `-e`
/// is part of what is under test: a branch that leaves a non-zero status
/// uncaught fails the step on the runner, so it must fail it here.
fn shell_argv(step: &Step) -> Vec<&'static str> {
    match step.shell.as_deref() {
        None => vec!["-e"],
        Some("bash") => vec!["--noprofile", "--norc", "-e", "-o", "pipefail"],
        Some(other) => panic!(
            "{WORKFLOW} step {:?} declares `shell: {other}`, which this harness does not \
             implement. It refuses rather than silently substituting a default \
             (spec 091 3.8 rule 5).",
            step.name
        ),
    }
}

fn run_script(step: &Step, script: &str, dir: &Path, env: &BTreeMap<String, String>) -> Ran {
    let script_path = dir.join(format!("step-{}.sh", sanitize(&step.name)));
    fs::write(&script_path, script).unwrap();
    let out_file = dir.join(format!("github_output-{}", sanitize(&step.name)));
    fs::write(&out_file, "").unwrap();
    let env_file = dir.join(format!("github_env-{}", sanitize(&step.name)));
    fs::write(&env_file, "").unwrap();

    let mut cmd = Command::new("bash");
    cmd.args(shell_argv(step))
        .arg(&script_path)
        .current_dir(dir)
        .env_clear()
        .env("PATH", env.get("PATH").expect("the harness sets PATH"))
        .env("HOME", dir)
        .env("GITHUB_OUTPUT", &out_file)
        .env("GITHUB_ENV", &env_file);
    for (k, v) in env {
        if k != "PATH" {
            cmd.env(k, v);
        }
    }
    let out = cmd.output().expect("bash runs");
    // 091 3.8 rule 5: this harness models `GITHUB_OUTPUT` and not `GITHUB_ENV`,
    // because no step of this job writes one. A step that started to would have
    // its value silently dropped and the suite would stay green, which is the
    // failure mode rule 5 exists to prevent, so the unsupported form refuses
    // loudly instead of going unevaluated.
    let env_written = fs::read_to_string(&env_file).unwrap();
    assert!(
        env_written.trim().is_empty(),
        "{WORKFLOW} step {:?} wrote to `GITHUB_ENV`, which this harness does not model \
         (spec 091 3.8 rule 5). It refuses rather than silently dropping the value:\n{env_written}",
        step.name
    );
    let mut outputs = BTreeMap::new();
    for line in fs::read_to_string(&out_file).unwrap().lines() {
        if let Some((k, v)) = line.split_once('=') {
            outputs.insert(k.to_string(), v.to_string());
        }
    }
    Ran {
        status: out.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned()
            + &String::from_utf8_lossy(&out.stderr),
        outputs,
    }
}

fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

// ---------------------------------------------------------------------------
// Stubs
// ---------------------------------------------------------------------------

fn write_exec(path: &Path, body: &str) {
    fs::write(path, body).unwrap();
    let mut perms = fs::metadata(path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms).unwrap();
}

/// `claude` and `gh`, and nothing else, on a `PATH` prefix. The real binaries
/// are shadowed, so a case that reached one would fail rather than escape.
fn install_stubs(bin: &Path) {
    fs::create_dir_all(bin).unwrap();
    write_exec(
        &bin.join("claude"),
        "#!/bin/sh\n\
         printf 'claude %s\\n' \"$*\" >> \"$STUB_CLAUDE_LOG\"\n\
         cat \"$STUB_CLAUDE_BODY\"\n\
         cat \"$STUB_CLAUDE_DIAG\" >&2\n\
         exit \"$STUB_CLAUDE_RC\"\n",
    );
    write_exec(
        &bin.join("gh"),
        "#!/bin/sh\n\
         printf 'gh %s\\n' \"$*\" >> \"$STUB_GH_LOG\"\n\
         prev=\n\
         for a in \"$@\"; do\n\
         \x20 if [ \"$prev\" = \"--body-file\" ]; then cat \"$a\" >> \"$STUB_GH_BODY\"; fi\n\
         \x20 prev=\"$a\"\n\
         done\n\
         exit \"$STUB_GH_RC\"\n",
    );
}

// ---------------------------------------------------------------------------
// Fixtures (091 3.8.1)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Annot {
    Error,
    Warning,
    None,
}

#[derive(Debug, Clone)]
struct Case {
    row: u32,
    what: &'static str,
    rc: i32,
    body: &'static str,
    diag: &'static str,
    token_set: bool,
    gh_rc: i32,
    class: u8,
    clf_exit: i32,
    review_status: Option<&'static str>,
    /// The publication step expected to be selected, by name fragment.
    publication: Option<&'static str>,
    pub_exit: i32,
    job_passes: bool,
    annot: Annot,
}

const REVIEW_PUB: &str = "Post review comment";
const NOTICE_PUB: &str = "Post API-failure notice";

impl Case {
    /// The default is class 6: an unclassified failure that blocks, publishes
    /// nothing and annotates `::error::`. Every other class is a deliberate
    /// departure from it, which is the shape of the policy itself.
    fn blocked(row: u32, what: &'static str, diag: &'static str) -> Self {
        Case {
            row,
            what,
            rc: 1,
            body: "",
            diag,
            token_set: true,
            gh_rc: 0,
            class: 6,
            clf_exit: 1,
            review_status: None,
            publication: None,
            pub_exit: 0,
            job_passes: false,
            annot: Annot::Error,
        }
    }

    fn rc(mut self, rc: i32) -> Self {
        self.rc = rc;
        self
    }

    /// An empty or whitespace-only review body at `rc == 0`: class 6 by the
    /// other door (3.7).
    fn empty_review(mut self, body: &'static str) -> Self {
        self.rc = 0;
        self.body = body;
        self.diag = "";
        self
    }

    fn refusal(mut self) -> Self {
        self.class = 4;
        self
    }

    fn transient(mut self) -> Self {
        self.class = 5;
        self.clf_exit = 0;
        self.review_status = Some("api_failure");
        self.publication = Some(NOTICE_PUB);
        self.job_passes = true;
        self.annot = Annot::Warning;
        self
    }

    fn reviewed(mut self, body: &'static str) -> Self {
        self.rc = 0;
        self.body = body;
        self.diag = "";
        self.class = 3;
        self.clf_exit = 0;
        self.review_status = Some("ok");
        self.publication = Some(REVIEW_PUB);
        self.job_passes = true;
        self.annot = Annot::None;
        self
    }

    /// The publication step's `gh pr comment` fails. Classification is
    /// untouched; the job outcome is not (D-12).
    fn publication_fails(mut self) -> Self {
        self.gh_rc = 1;
        self.pub_exit = 1;
        self.job_passes = false;
        self
    }

    fn secret_unset(mut self) -> Self {
        self.token_set = false;
        self.rc = 0;
        self.class = 2;
        self
    }
}

fn matrix() -> Vec<Case> {
    let b = Case::blocked;
    vec![
        b(1, "the PR #267 prose refusal, verbatim", "Your organization does not have access to Claude. Please login again or contact your administrator.").refusal(),
        b(2, "401 with an authentication_error type", "API Error: 401 {\"type\":\"authentication_error\",\"message\":\"invalid x-api-key\"}").refusal(),
        b(3, "403 with a permission_error type", "API Error: 403 {\"type\":\"permission_error\",\"message\":\"...\"}").refusal(),
        b(4, "an explicit credit refusal", "Credit balance is too low").refusal(),
        b(5, "529 overloaded_error", "API Error: 529 {\"type\":\"overloaded_error\",\"message\":\"Overloaded\"}").transient(),
        b(6, "429 rate_limit_error", "API Error: 429 {\"type\":\"rate_limit_error\",\"message\":\"...\"}").transient(),
        b(7, "503 in the CLI's own status form", "API Error: 503 Service Unavailable").transient(),
        b(8, "an errno with transport context on one line", "fetch failed: ETIMEDOUT connecting to api.anthropic.com").transient(),
        b(9, "a bare `fetch failed` names no failure", "fetch failed"),
        b(10, "a bare three-digit number is a line number somewhere", "529"),
        b(11, "bare `quota exceeded` settles nothing (D-2)", "quota exceeded"),
        b(12, "`quota exceeded` plus a recognized signal is classified by the signal", "quota exceeded\nAPI Error: 429 {\"type\":\"rate_limit_error\"}").transient(),
        b(13, "refusal outranks transient (3.9 rule 4)", "API Error: 529 overloaded_error\nYour organization does not have access").refusal(),
        b(14, "an unrecognized crash", "Segmentation fault"),
        b(15, "no evidence is not evidence of an outage", ""),
        b(16, "whitespace-only diagnostics", "   \n\t\n"),
        b(17, "timeout-sounding prose is text, not evidence", "The request timed out"),
        b(18, "exit 124 is a status a child can return on its own (D-10)", "").rc(124),
        b(19, "exit 137 is 128+SIGKILL, not proof of a bound (D-10)", "").rc(137),
        b(20, "124 plus timeout prose is still two non-signals", "The operation timed out after 300 seconds").rc(124),
        b(28, "a bare errno with no transport context is not a transient signal", "ETIMEDOUT"),
        b(29, "an errno and a transport phrase on separate lines are not a conjunction", "ETIMEDOUT\nfetch failed"),
        b(21, "a review was produced and published", "").reviewed("## Findings\n\nNone.\n"),
        b(22, "rc 0 with an empty body publishes nothing", "").empty_review(""),
        b(23, "rc 0 with a whitespace-only body publishes nothing", "").empty_review("   \n  \n"),
        b(24, "review prose about 403s and OAuth cannot be reclassified (3.1)", "").reviewed("## Findings\n\nThe new handler returns 403 on an expired OAuth token and the billing path is unauthorized without a seat check."),
        b(25, "a transient skip whose notice failed to publish still fails the job", "API Error: 529 {\"type\":\"overloaded_error\"}").transient().publication_fails(),
        b(26, "a review whose comment failed to publish still fails the job", "").reviewed("## Findings\n\nOne note about the parser.\n").publication_fails(),
        b(27, "the OAuth secret is unset", "").secret_unset(),
    ]
}

// ---------------------------------------------------------------------------
// The runner
// ---------------------------------------------------------------------------

struct Outcome {
    clf: Ran,
    selected: Option<String>,
    pub_status: i32,
    gh_log: String,
    gh_body: String,
}

impl Outcome {
    fn job_passes(&self) -> bool {
        self.clf.status == 0 && self.pub_status == 0
    }
}

fn fixture_ctx() -> Ctx {
    let mut github = BTreeMap::new();
    for (k, v) in [
        ("event.pull_request.number", "267"),
        (
            "event.pull_request.base.sha",
            "0000000000000000000000000000000000000000",
        ),
        (
            "event.pull_request.head.sha",
            "1111111111111111111111111111111111111111",
        ),
        (
            "event.pull_request.head.repo.full_name",
            "spec-spine/spec-spine",
        ),
        ("event.pull_request.body", "fixture PR body"),
        ("repository", "spec-spine/spec-spine"),
        ("actor", "a-human"),
    ] {
        github.insert(k.to_string(), v.to_string());
    }
    let mut secrets = BTreeMap::new();
    secrets.insert("GITHUB_TOKEN".into(), "fixture-not-a-real-token".into());
    secrets.insert("CLAUDE_CODE_OAUTH_TOKEN".into(), FIXTURE_TOKEN.into());

    let mut outputs: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    outputs.insert(
        "diff".into(),
        [("skip", "false"), ("lines", "120")]
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
    );
    outputs.insert(
        "ctx".into(),
        [("fork", "false"), ("dependabot", "false")]
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
    );
    Ctx {
        github,
        secrets,
        outputs,
        success: true,
    }
}

fn step_env(
    job: &Job,
    step: &Step,
    ctx: &Ctx,
    base: &BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    let mut env = base.clone();
    for (k, v) in job.env.iter().chain(step.env.iter()) {
        let whence = format!("{WORKFLOW} step {:?} env `{k}`", step.name);
        env.insert(k.clone(), ctx.expand(v, &whence));
    }
    env
}

/// One fixture, end to end: the classification step, then whichever
/// publication step its actual outputs select.
fn run_case(job: &Job, case: &Case, classify_script: Option<&str>) -> Outcome {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    let bin = d.join("bin");
    install_stubs(&bin);
    let tmpd = d.join("scratch");
    fs::create_dir_all(&tmpd).unwrap();
    let runner_temp = d.join("runner-temp");
    fs::create_dir_all(&runner_temp).unwrap();
    fs::write(tmpd.join("review-input.txt"), "===== PR DIFF =====\n").unwrap();

    let body = d.join("stub-body");
    let diagf = d.join("stub-diag");
    fs::write(&body, case.body).unwrap();
    fs::write(&diagf, case.diag).unwrap();
    let gh_log = d.join("gh.log");
    let gh_body = d.join("gh-body.txt");
    let claude_log = d.join("claude.log");
    for f in [&gh_log, &gh_body, &claude_log] {
        fs::write(f, "").unwrap();
    }

    let path = format!(
        "{}:{}",
        bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let mut base: BTreeMap<String, String> = BTreeMap::new();
    for (k, v) in [
        ("PATH", path),
        ("AI_REVIEW_TMP", tmpd.display().to_string()),
        ("RUNNER_TEMP", runner_temp.display().to_string()),
        ("STUB_CLAUDE_RC", case.rc.to_string()),
        ("STUB_CLAUDE_BODY", body.display().to_string()),
        ("STUB_CLAUDE_DIAG", diagf.display().to_string()),
        ("STUB_CLAUDE_LOG", claude_log.display().to_string()),
        ("STUB_GH_RC", case.gh_rc.to_string()),
        ("STUB_GH_LOG", gh_log.display().to_string()),
        ("STUB_GH_BODY", gh_body.display().to_string()),
    ] {
        base.insert(k.to_string(), v);
    }

    let mut ctx = fixture_ctx();
    if !case.token_set {
        // Actions puts the empty string in the environment for a secret that is
        // not available, which is the state the class 2 guard is written for.
        ctx.secrets
            .insert("CLAUDE_CODE_OAUTH_TOKEN".into(), String::new());
    }

    let classify = job.steps[step_index(job, CLASSIFY_STEP)].clone();
    assert!(
        eval_if(&classify, &ctx),
        "the fixture context must reach the classification step"
    );
    let script = classify_script.map(str::to_string).unwrap_or_else(|| {
        classify
            .run
            .clone()
            .expect("the classification step has a `run:`")
    });
    let env = step_env(job, &classify, &ctx, &base);
    let clf = run_script(&classify, &script, d, &env);

    // The publication step is selected from the outputs the classifier
    // *actually* wrote, never from what the fixture expects it to have written.
    ctx.outputs.insert(
        classify
            .id
            .clone()
            .expect("the classification step has an `id:`"),
        clf.outputs.clone(),
    );
    ctx.success = clf.status == 0;

    let selected: Vec<Step> = publication_steps(job)
        .into_iter()
        .filter(|s| eval_if(s, &ctx))
        .collect();
    assert!(
        selected.len() <= 1,
        "more than one publication step selected: {:?}",
        selected.iter().map(|s| &s.name).collect::<Vec<_>>()
    );

    let mut pub_status = 0;
    let mut selected_name = None;
    if let Some(s) = selected.first() {
        selected_name = Some(s.name.clone());
        let env = step_env(job, s, &ctx, &base);
        let ran = run_script(s, s.run.as_ref().unwrap(), d, &env);
        pub_status = ran.status;
    }

    Outcome {
        clf,
        selected: selected_name,
        pub_status,
        gh_log: fs::read_to_string(&gh_log).unwrap(),
        gh_body: fs::read_to_string(&gh_body).unwrap(),
    }
}

/// The distinguishing phrase of each class's annotation, read as a substring of
/// the workflow's own message. Class 3 says nothing, which the annotation
/// assertion already covers.
fn expected_diagnostic(case: &Case) -> Option<&'static str> {
    match (case.class, case.rc) {
        (2, _) => Some("CLAUDE_CODE_OAUTH_TOKEN secret is unset"),
        (4, _) => Some("The provider refused this AI review invocation"),
        (5, _) => Some("recognized transient"),
        (6, 0) => Some("wrote no review text"),
        (6, _) => Some("no recognized transient signal"),
        _ => None,
    }
}

fn assert_case(job: &Job, case: &Case) {
    let o = run_case(job, case, None);
    let label = format!("row {} (class {}): {}", case.row, case.class, case.what);

    assert_eq!(
        o.clf.status, case.clf_exit,
        "{label}: classification step exit status. stdout:\n{}",
        o.clf.stdout
    );
    match case.review_status {
        Some(want) => assert_eq!(
            o.clf.outputs.get("review_status").map(String::as_str),
            Some(want),
            "{label}: review_status output. outputs: {:?}",
            o.clf.outputs
        ),
        None => assert!(
            !o.clf.outputs.contains_key("review_status"),
            "{label}: review_status must be unset, got {:?}",
            o.clf.outputs
        ),
    }
    if case.review_status == Some("api_failure") {
        assert_eq!(
            o.clf.outputs.get("api_rc").map(String::as_str),
            Some(case.rc.to_string().as_str()),
            "{label}: api_rc output"
        );
        assert!(
            o.clf.outputs.contains_key("api_excerpt"),
            "{label}: api_excerpt output is set"
        );
    }

    // Class 4 and class 6 are observationally identical in exit status,
    // annotation level and publication: both block silently-greenless. The
    // only thing that separates them is what the operator is told, so the
    // diagnostic text is asserted. Without this, deleting every refusal
    // pattern would leave rows 1 to 4 green, because they would simply fall
    // through to class 6 and block for the other reason.
    if let Some(want) = expected_diagnostic(case) {
        assert!(
            o.clf.stdout.contains(want),
            "{label}: the annotation must say {want:?}, so class 4 and class 6 are \
             distinguishable to the operator. stdout:\n{}",
            o.clf.stdout
        );
    }

    let (err, warn) = o.clf.annotations();
    match case.annot {
        Annot::Error => assert!(
            err && !warn,
            "{label}: expected ::error::, stdout:\n{}",
            o.clf.stdout
        ),
        Annot::Warning => assert!(
            warn && !err,
            "{label}: expected ::warning::, stdout:\n{}",
            o.clf.stdout
        ),
        Annot::None => assert!(
            !err && !warn,
            "{label}: expected no annotation, stdout:\n{}",
            o.clf.stdout
        ),
    }

    match case.publication {
        Some(want) => {
            let got = o.selected.as_deref().unwrap_or("<none>");
            assert!(
                got.starts_with(want),
                "{label}: expected publication step starting {want:?}, got {got:?}"
            );
            assert!(
                o.gh_log.contains("gh pr comment"),
                "{label}: the gh stub was not invoked"
            );
            assert!(
                !o.gh_body.trim().is_empty(),
                "{label}: the published body is empty"
            );
        }
        None => {
            assert!(
                o.selected.is_none(),
                "{label}: no publication expected, got {:?}",
                o.selected
            );
            assert!(
                o.gh_log.is_empty(),
                "{label}: a blocked class must post nothing, gh log:\n{}",
                o.gh_log
            );
        }
    }
    assert_eq!(
        o.pub_status, case.pub_exit,
        "{label}: publication step exit status"
    );
    assert_eq!(
        o.job_passes(),
        case.job_passes,
        "{label}: job outcome (clf={}, pub={})",
        o.clf.status,
        o.pub_status
    );
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// 091 3.8.1: the whole fixture matrix, against the workflow's own scripts.
#[test]
fn policy_matrix_holds_against_the_workflows_own_scripts() {
    let job = parse_job();
    let cases = matrix();
    // The row set, not just its size: a count alone passes a matrix that
    // duplicated one row and dropped another, which is the way a fixture goes
    // missing in an edit. 3.8.1 numbers its rows 1 to 29 and every one of them
    // is an assertion some other row does not make.
    let rows: std::collections::BTreeSet<u32> = cases.iter().map(|c| c.row).collect();
    assert_eq!(
        rows,
        (1..=29).collect::<std::collections::BTreeSet<u32>>(),
        "every row of 3.8.1 is present exactly once"
    );
    assert_eq!(cases.len(), rows.len(), "no row number appears twice");
    for case in &cases {
        assert_case(&job, case);
    }
}

/// 091 3.8.1 rows 25 and 26, named so the acceptance block can run them alone:
/// classification succeeded and the job still fails, because publication did
/// not happen. This is the distinction the old single-column matrix hid.
#[test]
fn publication_failure_fails_the_job_even_when_classification_succeeded() {
    let job = parse_job();
    let rows: Vec<Case> = matrix()
        .into_iter()
        .filter(|c| c.row == 25 || c.row == 26)
        .collect();
    assert_eq!(rows.len(), 2, "both publication-failure rows are present");
    for case in &rows {
        assert_eq!(
            case.clf_exit, 0,
            "row {}: classification succeeds",
            case.row
        );
        assert!(!case.job_passes, "row {}: the job fails", case.row);
        assert_case(&job, case);
    }
}

/// 091 3.8.2: a passing verdict substituted into the class 4 branch is
/// detected even though every regex byte is untouched.
#[test]
fn inversion_of_the_refusal_branch_is_detected() {
    let job = parse_job();
    let classify = job.steps[step_index(&job, CLASSIFY_STEP)].clone();
    let run = classify.run.clone().unwrap();

    let mut lines: Vec<String> = run.lines().map(String::from).collect();
    let marker = lines
        .iter()
        .position(|l| l.contains("The provider refused this AI review invocation"))
        .expect("the class 4 branch announces itself with ::error::");
    let terminator = lines[marker + 1..]
        .iter()
        .position(|l| l.trim() == "exit 1")
        .expect("the class 4 branch terminates in a failing exit")
        + marker
        + 1;
    lines[terminator] = lines[terminator].replace("exit 1", "exit 0");
    let mutant = lines.join("\n") + "\n";
    assert_ne!(mutant, run, "the mutation changed something");
    for re in [
        "REFUSAL_RE",
        "TRANSIENT_RE",
        "NET_ERRNO_RE",
        "NET_CONTEXT_RE",
    ] {
        assert_eq!(
            run.matches(re).count(),
            mutant.matches(re).count(),
            "the mutation must leave every pattern byte untouched; {re} changed"
        );
    }

    let row1 = matrix().into_iter().find(|c| c.row == 1).unwrap();

    // The case is only meaningful if the unmutated script actually blocks.
    let honest = run_case(&job, &row1, None);
    assert_eq!(
        honest.clf.status, 1,
        "the real class 4 branch must block, else this case asserts nothing. stdout:\n{}",
        honest.clf.stdout
    );

    let inverted = run_case(&job, &row1, Some(&mutant));
    assert_ne!(
        inverted.clf.status, row1.clf_exit,
        "the inverted branch was not detected: the suite would pass a mutant that turns a \
         refusal into a green gate. Its refusal must be anchored on observed exit status, \
         not on the presence of a pattern."
    );
    assert_eq!(
        inverted.clf.status, 0,
        "the mutant passes, which is the point"
    );
}

/// 091 3.8 rule 5: every `if:` a case can reach is evaluated by the harness,
/// and an unsupported form panics rather than silently going unevaluated.
#[test]
fn every_reachable_condition_and_env_expression_is_evaluated() {
    let job = parse_job();
    let ctx = fixture_ctx();
    let base: BTreeMap<String, String> = [("PATH".to_string(), "/usr/bin".to_string())]
        .into_iter()
        .collect();
    let mut reached = 0;
    let first = step_index(&job, CLASSIFY_STEP);
    for step in &job.steps[first..] {
        if step.run.is_none() {
            continue;
        }
        let _ = eval_if(step, &ctx);
        let _ = step_env(&job, step, &ctx, &base);
        let _ = shell_argv(step);
        reached += 1;
    }
    assert!(
        reached >= 5,
        "expected the classification step plus its publication steps, saw {reached}"
    );
}

/// 091 3.2 and D-8: the structural skips keep their annotation fallbacks, and
/// 3.6 removes the one on the transient notice. AC-4 and AC-5, asserted from
/// the parsed steps rather than from a grep over the whole file.
#[test]
fn structural_skip_fallbacks_are_preserved_and_the_transient_swallow_is_gone() {
    let job = parse_job();
    for (name_fragment, want_fallback) in [
        ("Post API-failure notice", false),
        ("Skip notice (fork PR", true),
        ("Skip notice (Dependabot PR", true),
    ] {
        let step = job
            .steps
            .iter()
            .find(|s| s.name.starts_with(name_fragment))
            .unwrap_or_else(|| panic!("a step named {name_fragment:?} exists"));
        let run = step.run.as_deref().unwrap_or("");
        let swallowed =
            run.contains("gh pr comment") && run.contains("|| echo \"::warning::could not post");
        assert_eq!(
            swallowed, want_fallback,
            "{}: publication-failure fallback presence",
            step.name
        );
    }
}

/// 091 3.6.1 and D-10: no per-invocation timeout wrapper is introduced, and
/// the job-level bound that already existed is preserved.
#[test]
fn no_timeout_wrapper_is_introduced_and_the_job_bound_is_preserved() {
    let text = fs::read_to_string(repo_root().join(WORKFLOW)).unwrap();
    assert!(
        !text.contains("AI_REVIEW_TIMEOUT"),
        "spec 091 D-10 withdrew the AI_REVIEW_TIMEOUT bound"
    );
    assert!(
        text.contains("timeout-minutes: 10"),
        "the job-level bound, and its blocking `cancelled` outcome, are preserved"
    );
}

/// 091 3.4 and D-16: the post step's `review_nonempty` conjunct is the second
/// half of the empty-review guard, and no fixture can reach it while the
/// classifier's own guard holds. This case reconstructs the pre-119 classifier
/// in memory (`review_status=ok` from exit status alone, no `review_nonempty`)
/// and asserts the condition still refuses to publish. Dropping the conjunct
/// turns this red; every other case stays green, which is why it is here.
#[test]
fn the_post_step_condition_refuses_an_empty_review_the_classifier_let_through() {
    let job = parse_job();
    let run = job.steps[step_index(&job, CLASSIFY_STEP)]
        .run
        .clone()
        .unwrap();

    let mut lines: Vec<String> = run.lines().map(String::from).collect();
    let guard = lines
        .iter()
        .position(|l| l.contains("tr -d '[:space:]'") && l.contains("review.md"))
        .expect("the classifier guards on a non-whitespace review body (3.4)");
    let indent: String = lines[guard]
        .chars()
        .take_while(|c| c.is_whitespace())
        .collect();
    lines[guard] = format!("{indent}if true; then");
    let flag = lines
        .iter()
        .position(|l| l.contains("review_nonempty=true"))
        .expect("the classifier writes the review_nonempty output (3.4)");
    let indent: String = lines[flag]
        .chars()
        .take_while(|c| c.is_whitespace())
        .collect();
    lines[flag] = format!("{indent}true");
    let mutant = lines.join("\n") + "\n";
    assert_ne!(mutant, run, "the mutation changed something");
    assert!(
        !mutant.contains("review_nonempty"),
        "the mutant must not write the output the post condition tests"
    );

    let row22 = matrix().into_iter().find(|c| c.row == 22).unwrap();
    let o = run_case(&job, &row22, Some(&mutant));

    assert_eq!(
        o.clf.status, 0,
        "the reconstructed pre-119 classifier passes an empty review, else this case \
         asserts nothing. stdout:\n{}",
        o.clf.stdout
    );
    assert_eq!(
        o.clf.outputs.get("review_status").map(String::as_str),
        Some("ok"),
        "the reconstructed classifier sets review_status=ok"
    );
    assert_eq!(
        o.selected, None,
        "the post step's `review_nonempty` conjunct must refuse an empty review even when \
         the classifier let it through (3.4); it selected {:?}",
        o.selected
    );
    assert!(
        o.gh_log.is_empty(),
        "nothing may be published, gh log:\n{}",
        o.gh_log
    );
}
