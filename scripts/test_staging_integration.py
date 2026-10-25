import copy
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

import check_staging_integration as gate


class IntegrationTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        runtime = tempfile.TemporaryDirectory()
        self.addCleanup(runtime.cleanup)
        self.runtime = Path(runtime.name)
        self.git("init", "-q", "-b", "staging")
        self.git("config", "user.name", "Test")
        self.git("config", "user.email", "test@example.invalid")
        self.git("config", "commit.gpgsign", "false")
        self.git("config", "core.hooksPath", "/dev/null")
        for path, value in {
            "VERSION": "1.0.0", "CHANGELOG.md": "initial",
            "cli_manifests/codex/latest_validated.txt": "0.61.0",
            "cli_manifests/codex/min_supported.txt": "0.60.0",
            "cli_manifests/claude_code/latest_validated.txt": "2.1.29",
        }.items():
            self.write(path, value + "\n")
        self.initial = self.save("initial")
        self.git("checkout", "-qb", "feature")
        self.write("feature", "new feature\n")
        self.head = self.save("feature")
        self.git("checkout", "-q", "staging")
        self.write("base", "base advance\n")
        self.base = self.save("base")
        self.git("merge", "-q", "--no-ff", "feature", "-m", "candidate")
        self.candidate = self.git("rev-parse", "HEAD")
        self.git("remote", "add", "origin", str(self.root))
        self.event = {"pull_request": {"base": {"ref": "staging", "sha": self.base},
                                       "head": {"sha": self.head}}}

    def git(self, *args):
        return subprocess.run(["git", "-C", str(self.root), *args], check=True,
                              capture_output=True, text=True).stdout.strip()

    def write(self, path, text):
        path = self.root / path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)

    def save(self, message):
        self.git("add", ".")
        self.git("commit", "-qm", message)
        return self.git("rev-parse", "HEAD")

    def context(self, event=None, name="pull_request", expected=None):
        return gate.context(self.root, name, self.event if event is None else event,
                            self.candidate if expected is None else expected)

    def group(self, base=None, candidate=None):
        return {"action": "checks_requested", "merge_group": {
            "base_ref": "refs/heads/staging", "base_sha": base or self.base,
            "head_sha": candidate or self.candidate,
        }}

    def test_pr_binds_exact_two_parents_and_expected_checkout(self):
        result = self.context()
        self.assertTrue(result["integration"])
        self.assertEqual((result["candidate"], result["tip"], result["head"]),
                         (self.candidate, self.base, self.head))
        for path, value in [("base", self.initial), ("head", self.initial)]:
            event = copy.deepcopy(self.event)
            event["pull_request"][path]["sha"] = value
            with self.subTest(path=path), self.assertRaises(ValueError):
                self.context(event)
        with self.assertRaises(ValueError):
            self.context(expected=self.head)
        self.git("checkout", "-q", self.head)
        with self.assertRaises(ValueError):
            self.context(expected=self.head)  # A head checkout cannot replace a merge.

    def test_queue_accepts_whole_group_without_assuming_parent_layout(self):
        for base, candidate in [(self.base, self.candidate), (self.initial, self.head)]:
            self.git("checkout", "-q", candidate)
            result = self.context(self.group(base, candidate), "merge_group", candidate)
            self.assertEqual(result["tip"], base)
            self.assertTrue(result["integration"])
            self.assertEqual(self.git("rev-parse", "FETCH_HEAD"), base)

    def test_queue_refuses_wrong_ref_action_objects_and_checkout(self):
        for field, value in [("base_ref", "refs/heads/main"), ("base_sha", ""),
                             ("base_sha", "0" * 40), ("base_sha", "1" * 40),
                             ("head_sha", self.head), ("head_sha", "HEAD")]:
            event = self.group()
            event["merge_group"][field] = value
            with self.subTest(field=field, value=value), self.assertRaises(ValueError):
                self.context(event, "merge_group")
        event = self.group()
        event["action"] = "destroyed"
        with self.assertRaises(ValueError):
            self.context(event, "merge_group")
        for expected in ["", "HEAD", "1" * 40, self.head]:
            with self.subTest(expected=expected), self.assertRaises(ValueError):
                self.context(self.group(), "merge_group", expected)

    def test_queue_base_must_be_an_ancestor(self):
        self.git("checkout", "-qb", "other", self.initial)
        self.write("other", "unrelated advance\n")
        other = self.save("other")
        self.git("checkout", "-q", self.candidate)
        with self.assertRaises(ValueError):
            self.context(self.group(base=other), "merge_group")

    def test_non_staging_events_keep_no_tip_behavior(self):
        event = copy.deepcopy(self.event)
        event["pull_request"]["base"]["ref"] = "main"
        for name, payload in [("pull_request", event), ("push", {}), ("workflow_dispatch", {})]:
            result = self.context(payload, name)
            self.assertFalse(result["integration"])
            self.assertEqual(result["tip"], "")
        with self.assertRaises(ValueError):
            self.context({})  # Missing metadata is not a non-staging PR.

    def test_dirty_tracked_or_untracked_candidate_is_not_exact_result(self):
        for path in ("VERSION", "docs/uncommitted-proof.json"):
            with self.subTest(path=path):
                self.write(path, "uncommitted candidate input\n")
                with self.assertRaisesRegex(ValueError, "exactly the committed candidate"):
                    self.context()
                if path == "VERSION":
                    self.git("checkout", "--", path)
                else:
                    (self.root / path).unlink()

    def needs(self, candidate):
        result = {job: {"result": "success", "outputs": {}} for job in gate.JOBS}
        result["repo-version-guards"]["outputs"] = {
            key: candidate[key] for key in ("candidate", "tip")
        } | {"version_changed": "0"}
        result["publish-readiness"]["result"] = "skipped"
        for job, agent, names in [
            ("versions", "codex", ("latest_validated", "min_supported")),
            ("claude-versions", "claude_code", ("latest_validated",)),
        ]:
            result[job]["outputs"] = {
                name: self.git("show", f"{candidate['candidate']}:cli_manifests/{agent}/{name}.txt")
                for name in names
            }
        return result

    def test_all_required_jobs_must_succeed_and_optional_jobs_cannot_fail(self):
        candidate = self.context()
        needs = self.needs(candidate)
        self.assertEqual(gate.evaluate(self.root, candidate, needs, "0"),
                         {"publish-readiness": "VERSION unchanged in B->M"})
        for job in gate.JOBS:
            for result in ["failure", "cancelled", "", None, "neutral", "SUCCESS", "skipped"]:
                if job == "publish-readiness" and result == "skipped":
                    continue
                bad = copy.deepcopy(needs)
                bad[job]["result"] = result
                with self.subTest(job=job, result=result), self.assertRaises(ValueError):
                    gate.evaluate(self.root, candidate, bad, "0")

    def test_only_independently_justified_skips_are_accepted(self):
        self.write("cli_manifests/codex/min_supported.txt", "0.61.0\n")
        self.write("cli_manifests/claude_code/latest_validated.txt", "none\n")
        self.candidate = self.save("matching minimum and no Claude validation")
        candidate = self.context(self.group(), "merge_group")
        needs = self.needs(candidate)
        for job in ["linux-min-supported", "linux-claude-latest-validated"]:
            needs[job]["result"] = "skipped"
        self.assertEqual(set(gate.evaluate(self.root, candidate, needs, "0")),
                         {"publish-readiness", "linux-min-supported", "linux-claude-latest-validated"})
        for job in list(needs):
            if needs[job]["result"] == "skipped":
                needs[job]["result"] = "success"
        self.assertEqual(gate.evaluate(self.root, candidate, needs, "0"), {})

    def test_missing_or_unknown_inventory_and_results_are_refused(self):
        candidate = self.context()
        needs = self.needs(candidate)
        for bad in [{}, [], None, needs | {"new-job": {"result": "success"}},
                    {key: value for key, value in needs.items() if key != "preflight"},
                    needs | {"preflight": {}}, needs | {"preflight": None}]:
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                gate.evaluate(self.root, candidate, bad, "0")

    def test_missing_stale_and_contradictory_outputs_are_refused_even_with_success(self):
        candidate = self.context()
        needs = self.needs(candidate)
        for job in ["repo-version-guards", "versions", "claude-versions"]:
            for key in needs[job]["outputs"]:
                for value in [None, "", "contradiction"]:
                    bad = copy.deepcopy(needs)
                    bad[job]["outputs"][key] = value
                    with self.subTest(job=job, key=key, value=value), self.assertRaises(ValueError):
                        gate.evaluate(self.root, candidate, bad, "0")
        bad = copy.deepcopy(needs)
        bad["repo-version-guards"]["outputs"]["tip"] = self.initial
        with self.assertRaises(ValueError):
            gate.evaluate(self.root, candidate, bad, "0")
        needs["publish-readiness"]["result"] = "success"
        needs["repo-version-guards"]["outputs"]["version_changed"] = "1"
        with self.assertRaises(ValueError):
            gate.evaluate(self.root, candidate, needs, "0")

    def test_malformed_candidate_pointers_are_refused_even_with_successful_jobs(self):
        for agent, name, bad in [("codex", "min_supported", ""),
                                 ("codex", "latest_validated", "none"),
                                 ("claude_code", "latest_validated", " nonsense ")]:
            self.write("cli_manifests/codex/min_supported.txt", "0.60.0\n")
            self.write("cli_manifests/codex/latest_validated.txt", "0.61.0\n")
            self.write("cli_manifests/claude_code/latest_validated.txt", "2.1.29\n")
            self.write(f"cli_manifests/{agent}/{name}.txt", bad + "\n")
            self.candidate = self.save("bad pointer")
            candidate = self.context(self.group(), "merge_group")
            needs = self.needs(candidate)
            with self.subTest(agent=agent, name=name), self.assertRaises(ValueError):
                gate.evaluate(self.root, candidate, needs, "0")

    def test_changed_version_requires_changelog_and_readiness_success(self):
        self.write("VERSION", "1.1.0\n")
        self.candidate = self.save("version without changelog")
        candidate = self.context(self.group(), "merge_group")
        with self.assertRaises(ValueError):
            gate.version_change(self.root, "merge_group", self.group(), candidate)
        self.write("CHANGELOG.md", "release notes\n")
        self.candidate = self.save("changelog")
        candidate = self.context(self.group(), "merge_group")
        changed = gate.version_change(self.root, "merge_group", self.group(), candidate)
        self.assertEqual(changed, "1")
        needs = self.needs(candidate)
        needs["repo-version-guards"]["outputs"]["version_changed"] = changed
        with self.assertRaises(ValueError):
            gate.evaluate(self.root, candidate, needs, changed)
        needs["publish-readiness"]["result"] = "success"
        self.assertEqual(gate.evaluate(self.root, candidate, needs, changed), {})

    def test_identical_independent_bumps_use_integration_range_and_preserve_main(self):
        for branch in ["new-base", "new-head"]:
            self.git("checkout", "-qb", branch, self.initial)
            self.write("VERSION", "1.1.0\n")
            self.write("CHANGELOG.md", "same release notes\n")
            if branch == "new-head":
                self.write("other-feature", "unrelated\n")
            revision = self.save("same bump")
            if branch == "new-base":
                self.base = revision
            else:
                self.head = revision
        self.git("checkout", "-q", "new-base")
        self.git("merge", "-q", "--no-ff", "new-head", "-m", "same bump merge")
        self.candidate = self.git("rev-parse", "HEAD")
        self.event["pull_request"]["base"]["sha"] = self.base
        self.event["pull_request"]["head"]["sha"] = self.head
        for name, event in [("pull_request", self.event), ("merge_group", self.group())]:
            candidate = self.context(event, name)
            changed = gate.version_change(self.root, name, event, candidate)
            self.assertEqual(changed, "0")
            self.assertIn("publish-readiness", gate.evaluate(self.root, candidate, self.needs(candidate), changed))
        event = copy.deepcopy(self.event)
        event["pull_request"]["base"]["ref"] = "main"
        candidate = self.context(event)
        self.assertEqual(gate.version_change(self.root, "pull_request", event, candidate), "1")
        for before, changed in [(self.initial, "1"), (self.base, "0"), ("0" * 40, "0"), (None, "0")]:
            candidate = self.context({}, "push")
            self.assertEqual(gate.version_change(self.root, "push", {"before": before}, candidate), changed)
        candidate = self.context({}, "workflow_dispatch")
        self.assertEqual(gate.version_change(self.root, "workflow_dispatch", {}, candidate), "0")

    def cli(self, mode, event=None, name="pull_request", needs=None):
        event_file = self.runtime / "event.json"
        event_file.write_text(json.dumps(self.event if event is None else event))
        env = os.environ | {
            "GITHUB_EVENT_NAME": name, "GITHUB_EVENT_PATH": str(event_file),
            "GITHUB_SHA": self.candidate, "GITHUB_OUTPUT": str(self.runtime / "outputs"),
            "GITHUB_STEP_SUMMARY": str(self.runtime / "summary.md"),
            "GITHUB_REPOSITORY": "example/repo", "GITHUB_RUN_ID": "123", "GITHUB_RUN_ATTEMPT": "2",
            "CI_NEEDS": json.dumps(needs),
        }
        return subprocess.run([sys.executable, gate.__file__, mode], cwd=self.root,
                              env=env, capture_output=True, text=True)

    def test_cli_emits_bound_outputs_and_gate_summary_and_fails_on_errors(self):
        candidate = self.context()
        needs = self.needs(candidate)
        version = self.cli("version")
        self.assertEqual(version.returncode, 0, version.stderr)
        outputs = dict(line.split("=", 1) for line in (self.runtime / "outputs").read_text().splitlines())
        self.assertEqual(outputs, {"candidate": self.candidate, "tip": self.base, "version_changed": "0"})
        result = self.cli("gate", needs=needs)
        self.assertEqual(result.returncode, 0, result.stderr)
        report = json.loads(result.stdout)
        self.assertEqual((report["verdict"], report["candidate"], report["tip"]),
                         ("success", self.candidate, self.base))
        self.assertEqual((report["repository"], report["run_id"], report["run_attempt"]),
                         ("example/repo", "123", "2"))
        self.assertEqual(report["tree"], self.git("rev-parse", "HEAD^{tree}"))
        self.assertEqual(len(report["evaluator_sha256"]), 64)
        self.assertIn('"verdict": "success"', (self.runtime / "summary.md").read_text())
        needs["preflight"]["result"] = "cancelled"
        result = self.cli("gate", needs=needs)
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(json.loads(result.stdout)["verdict"], "failure")
        self.assertIn("preflight", result.stderr)
        self.assertNotIn("Traceback", result.stderr)
        result = self.cli("tip", self.group(), "merge_group")
        self.assertEqual((result.returncode, result.stdout.strip()), (0, self.base), result.stderr)
        result = self.cli("gate", {}, "workflow_dispatch", needs=None)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("Non-staging", result.stdout)


if __name__ == "__main__":
    unittest.main()
