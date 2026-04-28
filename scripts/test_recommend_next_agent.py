from __future__ import annotations

from datetime import datetime, timedelta, timezone
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent))

import recommend_next_agent as rna


def candidate_seed_text() -> str:
    return """[defaults.descriptor]
canonical_targets = ["darwin-arm64"]
wrapper_coverage_binding_kind = "generated_from_wrapper_crate"
always_on_capabilities = ["agent_api.config.model.v1", "agent_api.events", "agent_api.events.live", "agent_api.run"]
target_gated_capabilities = []
config_gated_capabilities = []
backend_extensions = []
support_matrix_enabled = true
capability_matrix_enabled = true
capability_matrix_target = ""
docs_release_track = "crates-io"

[candidate.opencode]
display_name = "OpenCode"
research_urls = ["https://research.local/opencode/repo", "https://research.local/opencode/docs", "https://research.local/opencode/pkg"]
install_channels = ["curl -fsSL https://opencode.ai/install | bash", "npm install -g opencode-ai", "brew install sst/tap/opencode"]
auth_notes = "Provider auth required for realistic evaluation."

[candidate.gemini_cli]
display_name = "Gemini CLI"
research_urls = ["https://research.local/gemini/repo", "https://research.local/gemini/docs", "https://research.local/gemini/pkg"]
install_channels = ["npm install -g @google/gemini-cli", "npx @google/gemini-cli"]
auth_notes = "Google account auth required for realistic evaluation."

[candidate.aider]
display_name = "aider"
research_urls = ["https://research.local/aider/repo", "https://research.local/aider/docs", "https://research.local/aider/pkg"]
install_channels = ["python -m pip install aider-install", "python -m pip install aider-chat"]
auth_notes = "Provider credentials may be required for full model-backed evaluation."
"""


def source_record(url: str, *, kind: str, summary: dict[str, object]) -> rna.SourceRecord:
    return rna.SourceRecord(
        url=url,
        kind=kind,
        fetched_at="2026-04-27T18:00:00Z",
        final_url=url,
        summary=summary,
    )


def recent_iso(days_ago: int) -> str:
    value = datetime.now(timezone.utc) - timedelta(days=days_ago)
    return value.replace(microsecond=0).isoformat().replace("+00:00", "Z")


def fake_fetcher(url: str) -> rna.SourceRecord:
    mapping: dict[str, rna.SourceRecord] = {
        "https://research.local/opencode/repo": source_record(
            "https://research.local/opencode/repo",
            kind="github_repo",
            summary={
                "repo": "sst/opencode",
                "description": "CLI run serve json agent subagent automation server headless tool workflow",
                "stars": 28000,
                "pushed_at": recent_iso(4),
                "updated_at": recent_iso(4),
                "topics": ["cli", "agent", "automation", "json", "server"],
                "latest_release_name": "v1.4.0",
            },
        ),
        "https://research.local/opencode/docs": source_record(
            "https://research.local/opencode/docs",
            kind="generic_page",
            summary={
                "title": "OpenCode CLI docs",
                "snippet": "run serve session fork subagent automation json api server workflow",
            },
        ),
        "https://research.local/opencode/pkg": source_record(
            "https://research.local/opencode/pkg",
            kind="npm_package",
            summary={
                "package_name": "opencode-ai",
                "latest_version": "1.4.7",
                "modified": recent_iso(5),
                "created": recent_iso(200),
                "version_count": 140,
                "description": "OpenCode package",
            },
        ),
        "https://research.local/gemini/repo": source_record(
            "https://research.local/gemini/repo",
            kind="github_repo",
            summary={
                "repo": "google-gemini/gemini-cli",
                "description": "CLI agent run terminal model automation workflow",
                "stars": 33000,
                "pushed_at": recent_iso(6),
                "updated_at": recent_iso(6),
                "topics": ["cli", "agent", "model"],
                "latest_release_name": "v0.38.1",
            },
        ),
        "https://research.local/gemini/docs": source_record(
            "https://research.local/gemini/docs",
            kind="generic_page",
            summary={
                "title": "Gemini CLI docs",
                "snippet": "run terminal model auth cli workflow",
            },
        ),
        "https://research.local/gemini/pkg": source_record(
            "https://research.local/gemini/pkg",
            kind="npm_package",
            summary={
                "package_name": "@google/gemini-cli",
                "latest_version": "0.38.1",
                "modified": recent_iso(7),
                "created": recent_iso(120),
                "version_count": 40,
                "description": "Gemini CLI package",
            },
        ),
        "https://research.local/aider/repo": source_record(
            "https://research.local/aider/repo",
            kind="github_repo",
            summary={
                "repo": "Aider-AI/aider",
                "description": "CLI terminal coding assistant agent",
                "stars": 29000,
                "pushed_at": recent_iso(3),
                "updated_at": recent_iso(3),
                "topics": ["cli", "coding", "assistant"],
                "latest_release_name": "v0.86.2",
            },
        ),
        "https://research.local/aider/docs": source_record(
            "https://research.local/aider/docs",
            kind="generic_page",
            summary={
                "title": "aider docs",
                "snippet": "terminal coding assistant workflow",
            },
        ),
        "https://research.local/aider/pkg": source_record(
            "https://research.local/aider/pkg",
            kind="pypi_package",
            summary={
                "package_name": "aider-chat",
                "latest_version": "0.86.2",
                "release_count": 220,
                "latest_upload_time": recent_iso(2),
                "description": "aider package",
            },
        ),
    }
    return mapping[url]


class RecommendationRunnerTests(unittest.TestCase):
    maxDiff = None

    def repo_fixture(self) -> tuple[tempfile.TemporaryDirectory[str], Path, Path, Path]:
        tmpdir = tempfile.TemporaryDirectory()
        root = Path(tmpdir.name)
        seed_path = root / "docs/agents/selection/candidate-seed.toml"
        seed_path.parent.mkdir(parents=True, exist_ok=True)
        seed_path.write_text(candidate_seed_text(), encoding="utf-8")
        canonical = root / rna.CANONICAL_PACKET_REL
        canonical.parent.mkdir(parents=True, exist_ok=True)
        canonical.write_text("ORIGINAL PACKET\n", encoding="utf-8")
        scratch_root = Path(f"{tmpdir.name}-scratch")
        scratch_root.mkdir(parents=True, exist_ok=True)
        return tmpdir, root, seed_path, scratch_root

    def test_seed_parsing_defaults(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            seed_path = Path(tmp) / "candidate-seed.toml"
            seed_path.write_text(candidate_seed_text(), encoding="utf-8")
            seed = rna.parse_seed_file(seed_path)
        self.assertEqual(seed.defaults.canonical_targets, [rna.DEFAULT_TARGET])
        self.assertEqual(seed.candidate_by_id("opencode").derived_descriptor(seed.defaults)["crate_path"], "crates/opencode")
        self.assertEqual(
            seed.candidate_by_id("gemini_cli").derived_descriptor(seed.defaults)["package_name"],
            "unified-agent-api-gemini-cli",
        )

    def test_exact_cli_flags_accepted_and_aliases_rejected(self) -> None:
        parser = rna.build_parser()
        parsed = parser.parse_args(
            [
                "generate",
                "--seed-file",
                "docs/agents/selection/candidate-seed.toml",
                "--run-id",
                "20260427-opencode",
                "--scratch-root",
                "~/.gstack/projects/repo/recommend-next-agent-runs",
            ]
        )
        self.assertEqual(parsed.command, "generate")
        with self.assertRaises(SystemExit):
            parser.parse_args(["generate", "--seed", "x", "--run-id", "y", "--scratch-root", "z"])
        with self.assertRaises(SystemExit):
            parser.parse_args(["promote", "--run", "x", "--repo-run-root", "y", "--approved-agent-id", "z", "--onboarding-pack-prefix", "p"])
        with self.assertRaises(SystemExit):
            parser.parse_args(["promote", "--run-dir", "x", "--repo-root", "y", "--approved-agent-id", "z", "--onboarding-pack-prefix", "p"])

    def test_missing_required_cli_flags_fail(self) -> None:
        parser = rna.build_parser()
        with self.assertRaises(SystemExit):
            parser.parse_args(["generate", "--seed-file", "x", "--run-id", "y"])
        with self.assertRaises(SystemExit):
            parser.parse_args(["promote", "--run-dir", "x", "--repo-run-root", "y", "--approved-agent-id", "z"])

    def test_generate_writes_full_scratch_artifact_set_and_exactly_three_dossiers(self) -> None:
        tmpdir, root, seed_path, scratch_root = self.repo_fixture()
        try:
            before = {path.relative_to(root).as_posix(): path.read_bytes() for path in root.rglob("*") if path.is_file()}
            run_dir = rna.generate_recommendation(
                seed_file=seed_path,
                run_id="20260427-opencode",
                scratch_root=scratch_root,
                fetcher=fake_fetcher,
                now_fn=lambda: "2026-04-27T18:00:00Z",
            )
            after = {path.relative_to(root).as_posix(): path.read_bytes() for path in root.rglob("*") if path.is_file()}
            self.assertEqual(before, after)
            for artifact in rna.SCRATCH_ARTIFACT_FILES:
                self.assertTrue((run_dir / artifact).exists(), artifact)
            dossiers = sorted(path.name for path in (run_dir / "candidate-dossiers").glob("*.json"))
            self.assertEqual(dossiers, ["aider.json", "gemini_cli.json", "opencode.json"])
        finally:
            rna.remove_path(scratch_root)
            tmpdir.cleanup()

    def test_generate_failure_with_fewer_than_three_eligible_candidates(self) -> None:
        tmpdir, _, seed_path, scratch_root = self.repo_fixture()
        bad_records = {
            "https://research.local/gemini/repo": source_record(
                "https://research.local/gemini/repo",
                kind="generic_page",
                summary={"title": "Gemini repo mirror", "snippet": ""},
            ),
            "https://research.local/gemini/docs": source_record(
                "https://research.local/gemini/docs",
                kind="generic_page",
                summary={"title": "Gemini CLI docs", "snippet": ""},
            ),
            "https://research.local/aider/repo": source_record(
                "https://research.local/aider/repo",
                kind="generic_page",
                summary={"title": "aider repo mirror", "snippet": ""},
            ),
            "https://research.local/aider/docs": source_record(
                "https://research.local/aider/docs",
                kind="generic_page",
                summary={"title": "aider docs", "snippet": ""},
            ),
        }

        def failing_fetcher(url: str) -> rna.SourceRecord:
            record = fake_fetcher(url)
            return bad_records.get(url, record)

        try:
            with self.assertRaises(rna.RecommendationError):
                rna.generate_recommendation(
                    seed_file=seed_path,
                    run_id="20260427-fail",
                    scratch_root=scratch_root,
                    fetcher=failing_fetcher,
                    now_fn=lambda: "2026-04-27T18:00:00Z",
                )
            run_dir = scratch_root / "20260427-fail"
            self.assertFalse((run_dir / "comparison.generated.md").exists())
            self.assertFalse((run_dir / "approval-draft.generated.toml").exists())
        finally:
            rna.remove_path(scratch_root)
            tmpdir.cleanup()

    def test_promote_writes_full_review_artifact_set_and_byte_copies(self) -> None:
        tmpdir, root, seed_path, scratch_root = self.repo_fixture()
        try:
            run_dir = rna.generate_recommendation(
                seed_file=seed_path,
                run_id="20260427-opencode",
                scratch_root=scratch_root,
                fetcher=fake_fetcher,
                now_fn=lambda: "2026-04-27T18:00:00Z",
            )
            review_dir = rna.promote_recommendation(
                run_dir=run_dir,
                repo_run_root_rel="docs/agents/selection/runs",
                approved_agent_id="opencode",
                onboarding_pack_prefix="opencode-onboarding",
                override_reason=None,
                repo_root=root,
                now_fn=lambda: "2026-04-27T19:00:00Z",
                git_head_fn=lambda _: "deadbeef",
                validator=lambda *_: None,
            )
            for artifact in rna.COPY_OWNED_REVIEW_FILES + rna.RENDERED_REVIEW_FILES:
                self.assertTrue((review_dir / artifact).exists(), artifact)
            self.assertEqual(
                (run_dir / "comparison.generated.md").read_bytes(),
                (review_dir / "comparison.generated.md").read_bytes(),
            )
            self.assertEqual(
                (run_dir / "comparison.generated.md").read_bytes(),
                (root / rna.CANONICAL_PACKET_REL).read_bytes(),
            )
            self.assertEqual(
                (review_dir / "comparison.generated.md").read_bytes(),
                (root / rna.CANONICAL_PACKET_REL).read_bytes(),
            )
            for artifact in (
                "candidate-pool.json",
                "eligible-candidates.json",
                "scorecard.json",
                "sources.lock.json",
            ):
                self.assertEqual((run_dir / artifact).read_bytes(), (review_dir / artifact).read_bytes())
            for dossier in ("aider.json", "gemini_cli.json", "opencode.json"):
                self.assertEqual(
                    (run_dir / "candidate-dossiers" / dossier).read_bytes(),
                    (review_dir / "candidate-dossiers" / dossier).read_bytes(),
                )
            final_approval = root / "docs/agents/lifecycle/opencode-onboarding/governance/approved-agent.toml"
            self.assertEqual(
                (review_dir / "approval-draft.generated.toml").read_text(encoding="utf-8"),
                final_approval.read_text(encoding="utf-8"),
            )
            self.assertNotEqual(
                (run_dir / "approval-draft.generated.toml").read_text(encoding="utf-8"),
                (review_dir / "approval-draft.generated.toml").read_text(encoding="utf-8"),
            )
        finally:
            rna.remove_path(scratch_root)
            tmpdir.cleanup()

    def test_promote_override_rerenders_approval_with_cli_owned_inputs(self) -> None:
        tmpdir, root, seed_path, scratch_root = self.repo_fixture()
        try:
            run_dir = rna.generate_recommendation(
                seed_file=seed_path,
                run_id="20260427-opencode",
                scratch_root=scratch_root,
                fetcher=fake_fetcher,
                now_fn=lambda: "2026-04-27T18:00:00Z",
            )
            review_dir = rna.promote_recommendation(
                run_dir=run_dir,
                repo_run_root_rel="docs/agents/selection/runs",
                approved_agent_id="gemini_cli",
                onboarding_pack_prefix="gemini-cli-onboarding",
                override_reason="Maintain the current proving-run lane.",
                repo_root=root,
                now_fn=lambda: "2026-04-27T19:00:00Z",
                git_head_fn=lambda _: "feedface",
                validator=lambda *_: None,
            )
            final_text = (root / "docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml").read_text(
                encoding="utf-8"
            )
            review_text = (review_dir / "approval-draft.generated.toml").read_text(encoding="utf-8")
            self.assertEqual(review_text, final_text)
            self.assertIn('approved_agent_id = "gemini_cli"', final_text)
            self.assertIn('recommended_agent_id = "opencode"', final_text)
            self.assertIn('override_reason = "Maintain the current proving-run lane."', final_text)
            self.assertIn('onboarding_pack_prefix = "gemini-cli-onboarding"', final_text)
        finally:
            rna.remove_path(scratch_root)
            tmpdir.cleanup()

    def test_promote_fails_for_shortlist_and_artifact_guards(self) -> None:
        tmpdir, root, seed_path, scratch_root = self.repo_fixture()
        try:
            run_dir = rna.generate_recommendation(
                seed_file=seed_path,
                run_id="20260427-opencode",
                scratch_root=scratch_root,
                fetcher=fake_fetcher,
                now_fn=lambda: "2026-04-27T18:00:00Z",
            )
            with self.assertRaises(rna.RecommendationError):
                rna.promote_recommendation(
                    run_dir=run_dir,
                    repo_run_root_rel="docs/agents/selection/runs",
                    approved_agent_id="cursor",
                    onboarding_pack_prefix="cursor-onboarding",
                    override_reason=None,
                    repo_root=root,
                    now_fn=lambda: "2026-04-27T19:00:00Z",
                    git_head_fn=lambda _: "deadbeef",
                    validator=lambda *_: None,
                )
            (run_dir / "sources.lock.json").unlink()
            with self.assertRaises(rna.RecommendationError):
                rna.promote_recommendation(
                    run_dir=run_dir,
                    repo_run_root_rel="docs/agents/selection/runs",
                    approved_agent_id="opencode",
                    onboarding_pack_prefix="opencode-onboarding",
                    override_reason=None,
                    repo_root=root,
                    now_fn=lambda: "2026-04-27T19:00:00Z",
                    git_head_fn=lambda _: "deadbeef",
                    validator=lambda *_: None,
                )
        finally:
            rna.remove_path(scratch_root)
            tmpdir.cleanup()

    def test_promote_rejects_existing_review_dir(self) -> None:
        tmpdir, root, seed_path, scratch_root = self.repo_fixture()
        try:
            run_dir = rna.generate_recommendation(
                seed_file=seed_path,
                run_id="20260427-opencode",
                scratch_root=scratch_root,
                fetcher=fake_fetcher,
                now_fn=lambda: "2026-04-27T18:00:00Z",
            )
            target = root / "docs/agents/selection/runs/20260427-opencode"
            target.mkdir(parents=True, exist_ok=True)
            with self.assertRaises(rna.RecommendationError):
                rna.promote_recommendation(
                    run_dir=run_dir,
                    repo_run_root_rel="docs/agents/selection/runs",
                    approved_agent_id="opencode",
                    onboarding_pack_prefix="opencode-onboarding",
                    override_reason=None,
                    repo_root=root,
                    now_fn=lambda: "2026-04-27T19:00:00Z",
                    git_head_fn=lambda _: "deadbeef",
                    validator=lambda *_: None,
                )
        finally:
            rna.remove_path(scratch_root)
            tmpdir.cleanup()

    def test_promote_validation_failure_preserves_canonical_and_final_approval(self) -> None:
        tmpdir, root, seed_path, scratch_root = self.repo_fixture()
        final_approval_path = root / "docs/agents/lifecycle/opencode-onboarding/governance/approved-agent.toml"
        final_approval_path.parent.mkdir(parents=True, exist_ok=True)
        final_approval_path.write_text("ORIGINAL APPROVAL\n", encoding="utf-8")
        try:
            run_dir = rna.generate_recommendation(
                seed_file=seed_path,
                run_id="20260427-opencode",
                scratch_root=scratch_root,
                fetcher=fake_fetcher,
                now_fn=lambda: "2026-04-27T18:00:00Z",
            )

            def failing_validator(*_: object) -> None:
                raise rna.RecommendationError("validation failed")

            with self.assertRaises(rna.RecommendationError):
                rna.promote_recommendation(
                    run_dir=run_dir,
                    repo_run_root_rel="docs/agents/selection/runs",
                    approved_agent_id="opencode",
                    onboarding_pack_prefix="opencode-onboarding",
                    override_reason=None,
                    repo_root=root,
                    now_fn=lambda: "2026-04-27T19:00:00Z",
                    git_head_fn=lambda _: "deadbeef",
                    validator=failing_validator,
                )
            self.assertEqual((root / rna.CANONICAL_PACKET_REL).read_text(encoding="utf-8"), "ORIGINAL PACKET\n")
            self.assertEqual(final_approval_path.read_text(encoding="utf-8"), "ORIGINAL APPROVAL\n")
            self.assertFalse((root / "docs/agents/selection/runs/20260427-opencode").exists())
        finally:
            rna.remove_path(scratch_root)
            tmpdir.cleanup()


if __name__ == "__main__":
    unittest.main()
