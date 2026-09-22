#!/usr/bin/env python3
# Spec: specs/121-the-acceptance-workflow-judges-what-was-built/spec.md
"""Render a release-mode sweep report into a GitHub Actions run.

    acceptance-report.py --report SWEEP_JSON --sweep-exit CODE [--summary FILE]

Writes both verdicts and the sweep's own sweep.md to the job summary (FILE,
default $GITHUB_STEP_SUMMARY, else stdout), and one workflow annotation per
row that needs a reader: `error` for a release failure, `warning` for a
pending block that did not pass, `notice` for a pending block that did.

Exit 0 when the report is present, readable, release-mode, and agrees with the
sweep's exit code; the sweep step decides the job. Exit 1 when the evidence is
missing or disagrees with itself (spec 121 3.4).
"""

import argparse
import json
import os
from pathlib import Path
import sys

BAD = ("failed", "not-declared", "not-run")
REQUIRED = ("verdicts", "releaseCounts", "counts", "specs")


def escape_data(text):
    # GitHub workflow-command escaping: a message cannot carry a raw newline.
    return str(text).replace("%", "%25").replace("\r", "%0D").replace("\n", "%0A")


def escape_property(text):
    return escape_data(text).replace(":", "%3A").replace(",", "%2C")


def annotate(level, title, message):
    print(f"::{level} title={escape_property(title)}::{escape_data(message)}")


class Summary:
    def __init__(self, path):
        self.path = path
        self.lines = []

    def add(self, *lines):
        self.lines.extend(lines)

    def flush(self):
        text = "\n".join(self.lines) + "\n"
        if self.path:
            with open(self.path, "a", encoding="utf-8") as handle:
                handle.write(text)
        else:
            sys.stdout.write(text)


def refuse(summary, why):
    annotate("error", "acceptance report", why)
    summary.add("## Acceptance: no verdict this run can vouch for", "", why, "")
    summary.flush()
    return 1


def lifecycle(row):
    lc = row.get("lifecycle") or {}
    return (f"status {lc.get('status') or 'absent'}, implementation "
            f"{lc.get('implementation') or 'absent'}, {lc.get('class') or 'unknown'}")


def main(argv):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--report", required=True, type=Path)
    parser.add_argument("--sweep-exit", required=True)
    parser.add_argument("--summary", default=os.environ.get("GITHUB_STEP_SUMMARY"))
    args = parser.parse_args(argv)
    summary = Summary(args.summary)

    # The code first: a sweep that refused or was interrupted may still have
    # left a report from an earlier stage, and that report is not this run's.
    try:
        code = int(args.sweep_exit.strip())
    except ValueError:
        return refuse(summary, f"the sweep step recorded no exit code ({args.sweep_exit!r}); "
                               "it was killed or cancelled before it finished")
    if code not in (0, 1):
        kind = "was interrupted" if code in (130, 143) else "refused"
        return refuse(summary, f"the sweep {kind} (exit {code}); its report, if any, is not a verdict")

    try:
        report = json.loads(args.report.read_text(encoding="utf-8"))
    except FileNotFoundError:
        return refuse(summary, f"the sweep exited {code} and wrote no report at {args.report}")
    except (OSError, UnicodeDecodeError, ValueError) as err:
        return refuse(summary, f"the report at {args.report} is unreadable: {err}")
    if not isinstance(report, dict):
        return refuse(summary, f"the report at {args.report} is not a JSON object")

    version = str(report.get("schemaVersion", ""))
    if version.split(".")[0] != "1" or any(k not in report for k in REQUIRED):
        return refuse(summary, f"the report's schema ({version or 'absent'}) is not a 1.x "
                               "report carrying both verdicts (1.1.0 or later)")
    if report.get("mode") != "release":
        return refuse(summary, f"the report's mode is {report.get('mode')!r}, not 'release': "
                               "the workflow must exit on the release verdict")

    verdicts = report["verdicts"]
    release_clean = verdicts.get("release") == "clean"
    if release_clean != (code == 0):
        return refuse(summary, f"the sweep exited {code} but its report's release verdict is "
                               f"{verdicts.get('release')!r}")

    counts = report["counts"]
    rcounts = report["releaseCounts"]
    short = report.get("revisionShort") or report.get("revision", "?")
    summary.add(
        f"## Acceptance at `{short}`",
        "",
        f"- **Release verdict: {'CLEAN' if release_clean else 'NOT CLEAN'}**. "
        "This decided the job.",
        "  " + ", ".join(f"{k} {rcounts.get(k, 0)}" for k in
                         ("passed", "failed", "not-declared", "exempt", "not-run", "pending")),
        f"- Corpus verdict (raw, every selected block): "
        f"**{'CLEAN' if verdicts.get('corpus') == 'clean' else 'NOT CLEAN'}**. "
        "Reported, not deciding.",
        "  " + ", ".join(f"{k} {counts.get(k, 0)}" for k in
                         ("passed", "failed", "not-declared", "exempt", "not-run")),
        "",
    )

    for row in report["specs"]:
        rid, outcome, rel = row.get("id"), row.get("outcome"), row.get("releaseOutcome")
        where = f" (log {row['log']})" if row.get("log") else ""
        failure = f": {row['failure']}" if row.get("failure") else ""
        if rel in BAD:
            annotate("error", f"acceptance {rid}",
                     f"{rel} for the release ({lifecycle(row)}){failure}{where}")
        elif rel == "pending" and outcome != "passed":
            annotate("warning", f"pending acceptance {rid}",
                     f"block {outcome} ({lifecycle(row)}); pending work, not a release "
                     f"obligation{failure}{where}")
        elif rel == "pending":
            annotate("notice", f"pending acceptance {rid}",
                     f"block passed before its build is recorded ({lifecycle(row)}); "
                     "a fail-first block that passes early is a finding")

    md = args.report.with_name("sweep.md")
    try:
        summary.add(md.read_text(encoding="utf-8"))
    except OSError as err:
        summary.add(f"_sweep.md could not be read: {err}_")
    summary.flush()
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
