#!/usr/bin/env python3
"""Check the candidate/base and existing CI results for staging integration."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys


JOBS = frozenset("""pr-purpose repo-version-guards publish-readiness preflight rust-tests
agent-api-all-features loc-check agent-api-backend-type-leak-guard capability-matrix
codex-validate-artifacts claude-validate-artifacts opencode-validate-artifacts
versions claude-versions linux-latest-validated linux-min-supported
linux-claude-latest-validated
""".split())


def require(condition, message):
    if not condition:
        raise ValueError(message)


def git(root, *args):
    result = subprocess.run(
        ["git", "--no-replace-objects", "-C", str(root), *args], capture_output=True,
    )
    require(result.returncode == 0,
            f"git {' '.join(args)} failed: {result.stderr.decode(errors='replace').strip()}")
    return result.stdout.decode("utf-8").removesuffix("\n")


def sha(value):
    require(isinstance(value, str) and re.fullmatch(r"[0-9a-f]{40}", value)
            and value != "0" * 40, f"missing or malformed commit SHA: {value!r}")
    return value


def commit(root, value):
    value = sha(value)
    require(git(root, "rev-parse", "--verify", f"{value}^{{commit}}") == value,
            f"expected a commit, not a tag: {value}")
    return value


def context(root, event_name, event, expected_sha):
    """Bind staging checks to the actual checkout and the event's whole transition."""
    candidate = git(root, "rev-parse", "HEAD")
    require(isinstance(event, dict), "event payload must be an object")
    require(candidate == commit(root, expected_sha), "checkout differs from GITHUB_SHA")
    result = dict(event=event_name, integration=False, candidate=candidate,
                  tree=git(root, "rev-parse", "HEAD^{tree}"), tip="", head="")
    if event_name == "pull_request":
        pr = event.get("pull_request", {})
        require(isinstance(pr, dict) and isinstance(pr.get("base"), dict)
                and isinstance(pr["base"].get("ref"), str) and pr["base"]["ref"],
                "missing pull-request base ref")
        if pr["base"]["ref"] != "staging":
            return result
        base = commit(root, pr["base"].get("sha"))
        require(isinstance(pr.get("head"), dict), "missing pull-request head")
        head = commit(root, pr.get("head", {}).get("sha"))
        parents = git(root, "rev-list", "--parents", "-n", "1", candidate).split()
        require(parents == [candidate, base, head],
                "PR checkout must be the two-parent merge of the event base and head")
        result.update(integration=True, tip=base, head=head)
    elif event_name == "merge_group":
        group = event.get("merge_group", {})
        require(isinstance(group, dict) and event.get("action") == "checks_requested"
                and group.get("base_ref") == "refs/heads/staging",
                "expected checks_requested for a staging merge group")
        base = sha(group.get("base_sha"))
        # Full-history checkout plus this explicit fetch makes the supplied base readable.
        # Never resolve a branch name in place of a missing base or continue without --tip.
        git(root, "fetch", "--no-tags", "origin", base)
        commit(root, base)
        require(candidate == commit(root, group.get("head_sha"))
                == commit(root, expected_sha), "checkout differs from the merge-group SHA")
        git(root, "merge-base", "--is-ancestor", base, candidate)
        result.update(integration=True, tip=base)
    return result


def version_change(root, event_name, event, candidate):
    """Use B->M for both staging events, preserving other release comparisons."""
    if candidate["integration"]:
        before, after = candidate["tip"], candidate["candidate"]
    elif event_name == "pull_request":
        pr = event["pull_request"]
        after = commit(root, pr["head"].get("sha"))
        before = git(root, "merge-base", commit(root, pr["base"].get("sha")), after)
    elif event_name == "push":
        before = event.get("before")
        if not before or before == "0" * 40:
            return "0"
        before, after = commit(root, before), candidate["candidate"]
    else:
        return "0"
    changed = set(git(root, "diff", "--name-only", before, after, "--", "VERSION", "CHANGELOG.md").splitlines())
    require("VERSION" not in changed or "CHANGELOG.md" in changed,
            "VERSION changed without a CHANGELOG.md change")
    return "1" if "VERSION" in changed else "0"


def pointer(root, candidate, agent, name, allow_none=False):
    value = git(root, "show", f"{candidate}:cli_manifests/{agent}/{name}.txt")
    part = r"(?:0|[1-9][0-9]*)"
    require((allow_none and value == "none") or re.fullmatch(rf"{part}\.{part}\.{part}", value),
            f"invalid {agent}/{name} pointer: {value!r}")
    return value


def evaluate(root, candidate, needs, changed):
    require(isinstance(needs, dict) and set(needs) == JOBS,
            "dependency inventory must contain exactly the declared 17 CI jobs")
    require(changed in ("0", "1"), "unknown version-change predicate")
    for job in sorted(JOBS):
        require(isinstance(needs[job], dict), f"missing result for {job}")
        result = needs[job].get("result")
        require(result in ("success", "failure", "cancelled", "skipped"),
                f"invalid result for {job}: {result!r}")
        require(result in ("success", "skipped"), f"{job}: {result}")

    def output(job, key, expected):
        values = needs[job].get("outputs", {})
        require(isinstance(values, dict) and values.get(key) == expected,
                f"{job}.{key} does not match the checked candidate")

    for key, value in {"candidate": candidate["candidate"], "tip": candidate["tip"],
                       "version_changed": changed}.items():
        output("repo-version-guards", key, value)
    latest = pointer(root, candidate["candidate"], "codex", "latest_validated")
    minimum = pointer(root, candidate["candidate"], "codex", "min_supported")
    claude = pointer(root, candidate["candidate"], "claude_code", "latest_validated", True)
    output("versions", "latest_validated", latest)
    output("versions", "min_supported", minimum)
    output("claude-versions", "latest_validated", claude)

    allowed_skips = {}
    if changed == "0":
        allowed_skips["publish-readiness"] = "VERSION unchanged in B->M"
    if minimum == latest:
        allowed_skips["linux-min-supported"] = "min_supported equals latest_validated"
    if claude == "none":
        allowed_skips["linux-claude-latest-validated"] = "Claude latest_validated is none"
    skipped = {}
    for job in sorted(JOBS):
        if needs[job]["result"] == "skipped":
            require(job in allowed_skips, f"unexpected skipped job: {job}")
            skipped[job] = allowed_skips[job]
    return skipped


def report_summary(report):
    text = json.dumps(report, indent=2, sort_keys=True)
    print(text)
    if os.environ.get("GITHUB_STEP_SUMMARY"):
        with open(os.environ["GITHUB_STEP_SUMMARY"], "a", encoding="utf-8") as summary:
            summary.write(f"### Staging integration verdict\n\n```json\n{text}\n```\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=("tip", "version", "gate"))
    args = parser.parse_args()
    root = Path.cwd()
    event_name = os.environ["GITHUB_EVENT_NAME"]
    event = json.loads(Path(os.environ["GITHUB_EVENT_PATH"]).read_text())
    candidate = context(root, event_name, event, os.environ.get("GITHUB_SHA", ""))
    if args.mode == "tip":
        print(candidate["tip"])
        return
    changed = version_change(root, event_name, event, candidate)
    if args.mode == "version":
        outputs = {"candidate": candidate["candidate"], "tip": candidate["tip"],
                   "version_changed": changed}
        with open(os.environ["GITHUB_OUTPUT"], "a", encoding="utf-8") as output:
            output.write("".join(f"{key}={value}\n" for key, value in outputs.items()))
        print(f"VERSION changed: {changed}")
        return
    if not candidate["integration"]:
        print("Non-staging event; the staging integration context is not emitted.")
        return
    needs = json.loads(os.environ["CI_NEEDS"])
    report = candidate | {
        "repository": os.environ.get("GITHUB_REPOSITORY", ""),
        "run_id": os.environ.get("GITHUB_RUN_ID", ""),
        "run_attempt": os.environ.get("GITHUB_RUN_ATTEMPT", ""),
        "evaluator_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        "version_changed": changed,
        "dependency_results": {job: data.get("result") if isinstance(data, dict) else None
                               for job, data in needs.items()} if isinstance(needs, dict) else None,
    }
    try:
        report["skip_reasons"] = evaluate(root, candidate, needs, changed)
        report["verdict"] = "success"
    except ValueError as error:
        report.update(verdict="failure", error=str(error))
        raise
    finally:
        report_summary(report)


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, KeyError, TypeError) as error:
        print(f"staging integration: {error}", file=sys.stderr)
        sys.exit(1)
