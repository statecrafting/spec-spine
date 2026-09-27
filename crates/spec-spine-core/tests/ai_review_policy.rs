// Spec: specs/091-an-unclassified-review-failure-blocks-the-merge/spec.md
//! Specs 091 and 156: the managed AI review keeps strict blocking semantics.
//!
//! Profile 10 moves the executable policy from an embedded workflow scalar to
//! `scripts/statecraft/ai-review.sh`. These tests inspect the managed caller and
//! execute that exact script for the failure classes whose precedence caused
//! the original defect. Statecraft owns the generated script; this repository
//! owns the proof that the adopted artifact keeps spec 091's invariant.

#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const WORKFLOW: &str = ".github/workflows/statecraft-ai-review.yml";
const SCRIPT: &str = "scripts/statecraft/ai-review.sh";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn run(mut command: Command) -> Output {
    command.output().expect("command starts")
}

fn git(dir: &Path, args: &[&str]) -> String {
    let mut command = Command::new("git");
    command.current_dir(dir).args(args);
    let output = run(command);
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

fn write_executable(path: &Path, body: &str) {
    fs::write(path, body).unwrap();
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
}

#[derive(Clone, Copy)]
struct ReviewCase<'a> {
    diagnostic: &'a str,
    reviewer_exit: i32,
    valid_review: bool,
    credential: bool,
    comment_exit: i32,
}

impl<'a> ReviewCase<'a> {
    fn failure(diagnostic: &'a str, reviewer_exit: i32) -> Self {
        Self {
            diagnostic,
            reviewer_exit,
            valid_review: false,
            credential: true,
            comment_exit: 0,
        }
    }
}

fn run_review(case: ReviewCase<'_>) -> Output {
    run_review_with_script(&repo_root().join(SCRIPT), case)
}

fn run_review_with_script(script: &Path, case: ReviewCase<'_>) -> Output {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("project");
    let bin = temp.path().join("bin");
    let scratch = temp.path().join("scratch");
    fs::create_dir_all(&project).unwrap();
    fs::create_dir_all(&bin).unwrap();
    fs::create_dir_all(&scratch).unwrap();

    git(&project, &["init", "-q"]);
    git(
        &project,
        &["config", "user.email", "fixture@example.invalid"],
    );
    git(&project, &["config", "user.name", "fixture"]);
    fs::write(project.join("subject.txt"), "base\n").unwrap();
    git(&project, &["add", "subject.txt"]);
    git(&project, &["commit", "-qm", "base"]);
    let base = git(&project, &["rev-parse", "HEAD"]);
    fs::write(project.join("subject.txt"), "head\n").unwrap();
    git(&project, &["commit", "-qam", "head"]);
    let head = git(&project, &["rev-parse", "HEAD"]);

    write_executable(&bin.join("npm"), "#!/bin/sh\nexit 0\n");
    write_executable(
        &bin.join("claude"),
        "#!/bin/sh\n\
         if [ \"${FIXTURE_VALID_REVIEW:-false}\" = true ]; then\n\
           printf '```json\\n{\"head\":\"%s\",\"verdict\":\"no-findings\",\"findings\":[]}\\n```\\n' \"$HEAD_SHA\"\n\
         else\n\
           printf '%s\\n' \"${FIXTURE_DIAGNOSTIC:-}\" >&2\n\
         fi\n\
         exit \"${FIXTURE_REVIEWER_EXIT:-0}\"\n",
    );
    write_executable(
        &bin.join("gh"),
        "#!/bin/sh\n\
         if [ \"${1:-}\" = api ]; then\n\
           case \"$*\" in\n\
             *'/pulls/'*) printf '%s\\n' \"$HEAD_SHA\" ;;\n\
             *) : ;;\n\
           esac\n\
           exit 0\n\
         fi\n\
         if [ \"${1:-}\" = pr ] && [ \"${2:-}\" = comment ]; then\n\
           echo fixture-gh-comment >&2\n\
           exit \"${FIXTURE_COMMENT_EXIT:-0}\"\n\
         fi\n\
         exit 1\n",
    );
    write_executable(
        &bin.join("sha256sum"),
        "#!/bin/sh\ncat >/dev/null\nprintf '%064d  -\\n' 0\n",
    );

    let system_path = std::env::var("PATH").unwrap();
    let mut command = Command::new("bash");
    command
        .arg(script)
        .current_dir(&project)
        .env("PATH", format!("{}:{system_path}", bin.display()))
        .env("BASE_SHA", &base)
        .env("HEAD_SHA", &head)
        .env("PR_NUMBER", "156")
        .env("REPO", "statecrafting/spec-spine")
        .env("HEAD_REPO", "statecrafting/spec-spine")
        .env("DIFF_CAP", "3000")
        .env("CLAUDE_CLI_VERSION", "fixture")
        .env("PROFILE_IDENTITY", "fixture-profile")
        .env("FIXTURE_DIAGNOSTIC", case.diagnostic)
        .env("FIXTURE_REVIEWER_EXIT", case.reviewer_exit.to_string())
        .env(
            "FIXTURE_VALID_REVIEW",
            if case.valid_review { "true" } else { "false" },
        )
        .env("FIXTURE_COMMENT_EXIT", case.comment_exit.to_string())
        .env("AI_REVIEW_TMP", &scratch)
        .env("GITHUB_OUTPUT", scratch.join("output"));
    if case.credential {
        command.env("CLAUDE_CODE_OAUTH_TOKEN", "fixture-not-a-secret");
    } else {
        command
            .env_remove("CLAUDE_CODE_OAUTH_TOKEN")
            .env_remove("ANTHROPIC_API_KEY");
    }
    run(command)
}

fn combined(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned() + &String::from_utf8_lossy(&output.stderr)
}

#[test]
fn managed_workflow_calls_the_base_review_script_without_secret_inheritance() {
    let text = fs::read_to_string(repo_root().join(WORKFLOW)).unwrap();
    let caller =
        fs::read_to_string(repo_root().join(".github/workflows/statecraft-ci.yml")).unwrap();
    let workflow: serde_yaml::Value = serde_yaml::from_str(&text).unwrap();
    let review = &workflow["jobs"]["review"];
    assert_eq!(review["timeout-minutes"].as_u64(), Some(15));
    let rendered = serde_yaml::to_string(review).unwrap();
    assert!(rendered.contains("scripts/statecraft/ai-review.sh"));
    assert!(rendered.contains("git show"));
    assert!(rendered.contains("bash \"$script\""));
    assert!(caller.contains("      ANTHROPIC_API_KEY: ${{ secrets.ANTHROPIC_API_KEY }}"));
    assert!(
        caller.contains("      CLAUDE_CODE_OAUTH_TOKEN: ${{ secrets.CLAUDE_CODE_OAUTH_TOKEN }}")
    );
    assert!(!caller.contains("secrets: inherit"));
}

#[test]
fn unclassified_reviewer_failure_blocks() {
    let output = run_review(ReviewCase::failure("unexpected provider response", 17));
    assert_eq!(output.status.code(), Some(4), "{}", combined(&output));
    assert!(
        combined(&output).contains("unclassified failure is not a review"),
        "{}",
        combined(&output)
    );
}

#[test]
fn access_refusal_outranks_other_failure_text() {
    let output = run_review(ReviewCase::failure("API Error: 503; access denied", 17));
    assert_eq!(output.status.code(), Some(2), "{}", combined(&output));
    assert!(
        combined(&output).contains("provider refused the review"),
        "{}",
        combined(&output)
    );
}

#[test]
fn empty_successful_review_blocks() {
    let output = run_review(ReviewCase::failure("", 0));
    assert_eq!(output.status.code(), Some(4), "{}", combined(&output));
    assert!(
        combined(&output).contains("exited 0 and wrote nothing"),
        "{}",
        combined(&output)
    );
}

#[test]
fn missing_credential_refuses() {
    let output = run_review(ReviewCase {
        credential: false,
        ..ReviewCase::failure("unused", 17)
    });
    assert_eq!(output.status.code(), Some(2), "{}", combined(&output));
    assert!(
        combined(&output).contains("neither ANTHROPIC_API_KEY nor CLAUDE_CODE_OAUTH_TOKEN"),
        "{}",
        combined(&output)
    );
}

#[test]
fn transient_failures_are_visible_skips() {
    for diagnostic in [
        "API Error: 429 rate limited",
        "API Error: 503 Service Unavailable",
        "API Error: 529 overloaded_error",
        "fetch failed: ECONNRESET connecting to api.anthropic.com",
        "quota exceeded\nAPI Error: 429 rate_limit_error",
    ] {
        let output = run_review(ReviewCase::failure(diagnostic, 17));
        assert_eq!(output.status.code(), Some(0), "{}", combined(&output));
        assert!(
            combined(&output).contains("skipped:transient"),
            "diagnostic {diagnostic:?}: {}",
            combined(&output)
        );
    }
}

#[test]
fn timeout_and_unscoped_network_text_block() {
    for (diagnostic, code) in [
        ("The operation timed out after 300 seconds", 124),
        ("The process was killed after its timeout", 137),
        ("ECONNRESET", 17),
        ("quota exceeded", 17),
    ] {
        let output = run_review(ReviewCase::failure(diagnostic, code));
        assert_eq!(output.status.code(), Some(4), "{}", combined(&output));
        assert!(
            combined(&output).contains("unclassified failure is not a review"),
            "diagnostic {diagnostic:?}: {}",
            combined(&output)
        );
    }
}

#[test]
fn successful_review_is_published() {
    let output = run_review(ReviewCase {
        valid_review: true,
        ..ReviewCase::failure("", 0)
    });
    assert_eq!(output.status.code(), Some(0), "{}", combined(&output));
    assert!(combined(&output).contains("fixture-gh-comment"));
    assert!(combined(&output).contains("AI review: no-findings"));
}

#[test]
fn publication_failure_blocks() {
    let output = run_review(ReviewCase {
        valid_review: true,
        comment_exit: 1,
        ..ReviewCase::failure("", 0)
    });
    assert_eq!(output.status.code(), Some(4), "{}", combined(&output));
    assert!(
        combined(&output).contains("review could not be posted"),
        "{}",
        combined(&output)
    );
}

#[test]
fn inversion_of_the_refusal_branch_is_detected() {
    let baseline = run_review(ReviewCase::failure("API Error: 529; access denied", 17));
    assert_eq!(baseline.status.code(), Some(2), "{}", combined(&baseline));

    let temp = tempfile::tempdir().unwrap();
    let mutant = temp.path().join("ai-review.sh");
    let source = fs::read_to_string(repo_root().join(SCRIPT)).unwrap();
    let changed = source.replacen(
        "if printf '%s\\n' \"$err\" | grep -qiE \"$REFUSAL_RE\"; then",
        "if false && printf '%s\\n' \"$err\" | grep -qiE \"$REFUSAL_RE\"; then",
        1,
    );
    assert_ne!(source, changed, "the refusal branch mutation must apply");
    write_executable(&mutant, &changed);

    let output = run_review_with_script(
        &mutant,
        ReviewCase::failure("API Error: 529; access denied", 17),
    );
    assert_ne!(
        output.status.code(),
        Some(2),
        "the executed matrix must detect a disabled refusal branch: {}",
        combined(&output)
    );
}
