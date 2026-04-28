from __future__ import annotations

from collections.abc import Callable
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent))

import recommend_next_agent as rna


GENERATED_AT = "2026-04-27T18:00:00Z"
APPROVED_AT = "2026-04-27T19:00:00Z"
RUN_ID = "20260427-frozen-contract"
CANDIDATE_ORDER = (
    ("gamma", "Gamma CLI"),
    ("beta", "Beta CLI"),
    ("alpha", "Alpha CLI"),
    ("delta", "Delta CLI"),
)


def registry_text(*agent_ids: str) -> str:
    return "\n".join(f'[[agents]]\nagent_id = "{agent_id}"\n' for agent_id in agent_ids)


def hex64(label: str) -> str:
    return rna.sha256_bytes(label.encode("utf-8"))


class RecommendationRunnerContractTests(unittest.TestCase):
    maxDiff = None

    def repo_fixture(self) -> tuple[tempfile.TemporaryDirectory[str], Path, Path, Path, Path]:
        tmpdir = tempfile.TemporaryDirectory()
        root = Path(tmpdir.name)

        live_seed = root / rna.LIVE_SEED_REL
        live_seed.parent.mkdir(parents=True, exist_ok=True)
        live_seed.write_text(self.seed_text(), encoding="utf-8")

        registry_path = root / rna.REGISTRY_RELATIVE_PATH
        registry_path.parent.mkdir(parents=True, exist_ok=True)
        registry_path.write_text(registry_text("codex", "claude_code"), encoding="utf-8")

        canonical_path = root / rna.CANONICAL_PACKET_REL
        canonical_path.parent.mkdir(parents=True, exist_ok=True)
        canonical_path.write_text("ORIGINAL PACKET\n", encoding="utf-8")

        scratch_root = root / "scratch"
        scratch_root.mkdir(parents=True, exist_ok=True)
        return tmpdir, root, live_seed, scratch_root, registry_path

    def seed_text(self) -> str:
        blocks = [
            '[defaults.descriptor]',
            'canonical_targets = ["darwin-arm64"]',
            'wrapper_coverage_binding_kind = "generated_from_wrapper_crate"',
            'always_on_capabilities = ["agent_api.run", "agent_api.events"]',
            'target_gated_capabilities = []',
            'config_gated_capabilities = []',
            'backend_extensions = []',
            'support_matrix_enabled = true',
            'capability_matrix_enabled = true',
            'capability_matrix_target = ""',
            'docs_release_track = "crates-io"',
        ]
        for agent_id, display_name in CANDIDATE_ORDER:
            blocks.extend(
                [
                    "",
                    f"[candidate.{agent_id}]",
                    f'display_name = "{display_name}"',
                    f'research_urls = ["https://research.local/{agent_id}/repo", "https://research.local/{agent_id}/docs"]',
                    f'install_channels = ["brew install {agent_id}", "npm install -g {agent_id}"]',
                    f'auth_notes = "{display_name} auth notes"',
                ]
            )
        return "\n".join(blocks) + "\n"

    def research_dir(
        self,
        root: Path,
        *,
        run_id: str = RUN_ID,
        dirname: str | None = None,
        metadata_run_id: str | None = None,
        metadata_override: dict[str, object] | None = None,
        dossier_mutator: Callable[[str, dict[str, object]], dict[str, object]] | None = None,
    ) -> Path:
        actual_dirname = dirname or run_id
        research_dir = root / "research" / actual_dirname
        dossier_dir = research_dir / "dossiers"
        dossier_dir.mkdir(parents=True, exist_ok=True)

        snapshot_bytes = self.seed_text().encode("utf-8")
        snapshot_sha = rna.sha256_bytes(snapshot_bytes)
        rna.write_bytes(research_dir / "seed.snapshot.toml", snapshot_bytes)
        rna.write_text(research_dir / "research-summary.md", "# Frozen research summary\n")

        metadata = {
            "run_id": metadata_run_id or run_id,
            "evidence_collection_time_seconds": 321,
            "fetched_source_count": 12,
        }
        if metadata_override is not None:
            metadata = metadata_override
        rna.write_json(research_dir / "research-metadata.json", metadata)

        dossier_specs = {
            "alpha": dict(
                non_interactive="verified",
                offline="verified",
                observable="verified",
                redaction="verified",
                crate="verified",
                reproducibility="verified",
                future="inferred",
                blocked_steps=[],
            ),
            "beta": dict(
                non_interactive="verified",
                offline="verified",
                observable="verified",
                redaction="verified",
                crate="verified",
                reproducibility="verified",
                future="inferred",
                blocked_steps=["Hosted provider mode still needs maintainer follow-up."],
            ),
            "gamma": dict(
                non_interactive="verified",
                offline="inferred",
                observable="verified",
                redaction="inferred",
                crate="verified",
                reproducibility="inferred",
                future="inferred",
                blocked_steps=[],
            ),
            "delta": dict(
                non_interactive="verified",
                offline="inferred",
                observable="inferred",
                redaction="inferred",
                crate="inferred",
                reproducibility="inferred",
                future="unknown",
                blocked_steps=[],
            ),
        }
        for agent_id, display_name in CANDIDATE_ORDER:
            dossier = self.dossier_payload(
                agent_id=agent_id,
                display_name=display_name,
                snapshot_sha=snapshot_sha,
                **dossier_specs[agent_id],
            )
            if dossier_mutator is not None:
                dossier = dossier_mutator(agent_id, dossier)
            rna.write_json(dossier_dir / f"{agent_id}.json", dossier)
        return research_dir

    def dossier_payload(
        self,
        *,
        agent_id: str,
        display_name: str,
        snapshot_sha: str,
        non_interactive: str,
        offline: str,
        observable: str,
        redaction: str,
        crate: str,
        reproducibility: str,
        future: str,
        blocked_steps: list[str],
    ) -> dict[str, object]:
        evidence_ids = {
            "doc": f"{agent_id}-doc",
            "repo": f"{agent_id}-repo",
            "pkg": f"{agent_id}-pkg",
        }
        return {
            "schema_version": "v1",
            "agent_id": agent_id,
            "display_name": display_name,
            "generated_at": GENERATED_AT,
            "seed_snapshot_sha256": snapshot_sha,
            "official_links": [
                f"https://docs.example.test/{agent_id}",
                f"https://install.example.test/{agent_id}",
            ],
            "install_channels": [
                f"brew install {agent_id}",
                f"npm install -g {agent_id}",
            ],
            "auth_prerequisites": [],
            "claims": {
                "non_interactive_execution": self.claim_payload(
                    state=non_interactive,
                    summary=f"{display_name} non-interactive execution",
                    evidence_ids=[evidence_ids["doc"]],
                ),
                "offline_strategy": self.claim_payload(
                    state=offline,
                    summary=f"{display_name} offline strategy",
                    evidence_ids=[evidence_ids["doc"]],
                ),
                "observable_cli_surface": self.claim_payload(
                    state=observable,
                    summary=f"{display_name} observable CLI surface",
                    evidence_ids=[evidence_ids["repo"]],
                ),
                "redaction_fit": self.claim_payload(
                    state=redaction,
                    summary=f"{display_name} redaction fit",
                    evidence_ids=[evidence_ids["doc"]],
                ),
                "crate_first_fit": self.claim_payload(
                    state=crate,
                    summary=f"{display_name} crate-first fit",
                    evidence_ids=[evidence_ids["repo"]],
                ),
                "reproducibility": self.claim_payload(
                    state=reproducibility,
                    summary=f"{display_name} reproducibility",
                    evidence_ids=[evidence_ids["pkg"]],
                ),
                "future_leverage": self.claim_payload(
                    state=future,
                    summary=f"{display_name} future leverage",
                    evidence_ids=[evidence_ids["pkg"]],
                ),
            },
            "probe_requests": [],
            "blocked_steps": blocked_steps,
            "normalized_caveats": [f"{display_name} caveat normalization"],
            "evidence": [
                {
                    "evidence_id": evidence_ids["doc"],
                    "kind": "official_doc",
                    "url": f"https://docs.example.test/{agent_id}",
                    "title": f"{display_name} docs",
                    "captured_at": GENERATED_AT,
                    "sha256": hex64(f"{agent_id}:doc"),
                    "excerpt": f"{display_name} docs excerpt",
                },
                {
                    "evidence_id": evidence_ids["repo"],
                    "kind": "github",
                    "url": f"https://github.com/example/{agent_id}",
                    "title": f"{display_name} repo",
                    "captured_at": GENERATED_AT,
                    "sha256": hex64(f"{agent_id}:repo"),
                    "excerpt": f"{display_name} repo excerpt",
                },
                {
                    "evidence_id": evidence_ids["pkg"],
                    "kind": "package_registry",
                    "url": f"https://packages.example.test/{agent_id}",
                    "title": f"{display_name} package",
                    "captured_at": GENERATED_AT,
                    "sha256": hex64(f"{agent_id}:pkg"),
                    "excerpt": f"{display_name} package excerpt",
                },
            ],
        }

    def claim_payload(self, *, state: str, summary: str, evidence_ids: list[str]) -> dict[str, object]:
        return {
            "state": state,
            "summary": summary,
            "evidence_ids": evidence_ids,
            "notes": "none",
        }

    def load_json(self, path: Path) -> object:
        return json.loads(path.read_text(encoding="utf-8"))

    def file_bytes(self, root: Path) -> dict[str, bytes]:
        return {
            path.relative_to(root).as_posix(): path.read_bytes()
            for path in root.rglob("*")
            if path.is_file()
        }

    def assert_run_status_schema(self, payload: dict[str, object]) -> None:
        self.assertEqual(
            set(payload.keys()),
            {
                "run_id",
                "status",
                "generated_at",
                "research_dir",
                "run_dir",
                "eligible_candidate_ids",
                "shortlist_ids",
                "recommended_agent_id",
                "candidate_status_counts",
                "metrics",
                "errors",
                "approved_agent_id",
                "approval_recorded_at",
                "override_reason",
                "committed_review_dir",
                "committed_packet_path",
                "committed_approval_artifact_path",
            },
        )
        self.assertEqual(
            set(payload["candidate_status_counts"].keys()),
            {"eligible", "candidate_rejected", "candidate_error"},
        )
        self.assertEqual(set(payload["metrics"].keys()), set(rna.RUN_STATUS_METRIC_KEYS))

    def assert_candidate_validation_schema(self, payload: dict[str, object]) -> None:
        self.assertEqual(
            set(payload.keys()),
            {
                "agent_id",
                "status",
                "schema_valid",
                "hard_gate_results",
                "probe_results",
                "rejection_reasons",
                "error_reasons",
                "evidence_ids_used",
                "notes",
            },
        )
        self.assertEqual(set(payload["hard_gate_results"].keys()), set(rna.HARD_GATE_KEYS))
        for gate in payload["hard_gate_results"].values():
            self.assertEqual(set(gate.keys()), {"status", "evidence_ids", "notes"})

    def selection_staging_root(self, root: Path, run_id: str) -> Path:
        return root / "docs/agents/selection/.staging" / run_id

    def lifecycle_staging_root(self, root: Path, run_id: str) -> Path:
        return root / "docs/agents/lifecycle/.staging" / run_id

    def test_parser_requires_frozen_cli_args(self) -> None:
        parser = rna.build_parser()

        parsed = parser.parse_args(
            [
                "generate",
                "--seed-file",
                "docs/agents/selection/candidate-seed.toml",
                "--research-dir",
                "research/20260427-frozen-contract",
                "--run-id",
                RUN_ID,
                "--scratch-root",
                "/tmp/recommend-next-agent-runs",
            ]
        )
        self.assertEqual(parsed.command, "generate")
        self.assertEqual(parsed.research_dir, "research/20260427-frozen-contract")

        with self.assertRaises(SystemExit):
            parser.parse_args(
                [
                    "generate",
                    "--seed-file",
                    "docs/agents/selection/candidate-seed.toml",
                    "--run-id",
                    RUN_ID,
                    "--scratch-root",
                    "/tmp/recommend-next-agent-runs",
                ]
            )
        with self.assertRaises(SystemExit):
            parser.parse_args(
                [
                    "promote",
                    "--run-dir",
                    "/tmp/recommend-next-agent-runs/run",
                    "--repo-run-root",
                    "docs/agents/selection/runs",
                    "--approved-agent-id",
                    "alpha",
                ]
            )

    def test_generate_rejects_missing_and_mismatched_research_inputs(self) -> None:
        cases = (
            (
                "research_dir_missing",
                lambda root: root / "research" / RUN_ID,
                None,
                "research_dir_missing",
                "does not exist",
            ),
            (
                "research_summary_missing",
                lambda root: self.research_dir(root),
                lambda research_dir: (research_dir / "research-summary.md").unlink(),
                "research_summary_missing",
                "research summary",
            ),
            (
                "seed_snapshot_missing",
                lambda root: self.research_dir(root),
                lambda research_dir: (research_dir / "seed.snapshot.toml").unlink(),
                "seed_snapshot_missing",
                "seed snapshot",
            ),
            (
                "research_metadata_run_id_mismatch",
                lambda root: self.research_dir(root, metadata_run_id="different-run-id"),
                None,
                "research_metadata_invalid",
                "research metadata run_id must equal CLI --run-id",
            ),
            (
                "research_dir_basename_mismatch",
                lambda root: self.research_dir(root, dirname="different-dir-name"),
                None,
                "research_metadata_invalid",
                "research directory basename must equal CLI --run-id",
            ),
            (
                "research_metadata_keys_mismatch",
                lambda root: self.research_dir(
                    root,
                    metadata_override={"run_id": RUN_ID, "fetched_source_count": 12},
                ),
                None,
                "research_metadata_invalid",
                "research metadata keys do not match the frozen contract",
            ),
        )

        for name, build_research_dir, mutate, expected_code, expected_message in cases:
            with self.subTest(case=name):
                tmpdir, root, live_seed, scratch_root, registry_path = self.repo_fixture()
                try:
                    research_dir = build_research_dir(root)
                    if mutate is not None:
                        mutate(research_dir)

                    with self.assertRaises(rna.RecommendationError):
                        rna.generate_recommendation(
                            seed_file=live_seed,
                            research_dir=research_dir,
                            run_id=RUN_ID,
                            scratch_root=scratch_root,
                            registry_path=registry_path,
                            now_fn=lambda: GENERATED_AT,
                        )

                    run_dir = scratch_root / RUN_ID
                    run_status = self.load_json(run_dir / "run-status.json")
                    self.assert_run_status_schema(run_status)
                    self.assertEqual(run_status["status"], "run_fatal")
                    self.assertEqual(run_status["errors"][0]["code"], expected_code)
                    self.assertIn(expected_message, run_status["errors"][0]["message"])

                    if expected_code == "research_metadata_invalid":
                        self.assertFalse((run_dir / "candidate-pool.json").exists())
                        self.assertFalse((run_dir / "eligible-candidates.json").exists())
                        self.assertFalse((run_dir / "scorecard.json").exists())
                        self.assertFalse((run_dir / "sources.lock.json").exists())
                        self.assertEqual(sorted(path.name for path in run_dir.iterdir()), ["run-status.json", "seed.snapshot.toml"])
                finally:
                    tmpdir.cleanup()

    def test_generate_surfaces_dossier_identity_mismatch_as_candidate_error(self) -> None:
        tmpdir, root, live_seed, scratch_root, registry_path = self.repo_fixture()
        try:
            research_dir = self.research_dir(
                root,
                dossier_mutator=lambda agent_id, dossier: (
                    {**dossier, "agent_id": "shadow-delta"} if agent_id == "delta" else dossier
                ),
            )
            run_dir = rna.generate_recommendation(
                seed_file=live_seed,
                research_dir=research_dir,
                run_id=RUN_ID,
                scratch_root=scratch_root,
                registry_path=registry_path,
                now_fn=lambda: GENERATED_AT,
            )

            run_status = self.load_json(run_dir / "run-status.json")
            self.assertEqual(run_status["status"], "success_with_candidate_errors")
            self.assertEqual(
                run_status["candidate_status_counts"],
                {"eligible": 3, "candidate_rejected": 0, "candidate_error": 1},
            )
            self.assertEqual(run_status["shortlist_ids"], ["alpha", "beta", "gamma"])
            self.assertEqual(run_status["recommended_agent_id"], "alpha")

            delta_validation = self.load_json(run_dir / "candidate-validation-results" / "delta.json")
            self.assert_candidate_validation_schema(delta_validation)
            self.assertEqual(delta_validation["status"], "candidate_error")
            self.assertFalse(delta_validation["schema_valid"])
            self.assertIn("agent_id does not match its filename", delta_validation["error_reasons"][0])
            self.assertEqual(delta_validation["probe_results"], [])
            self.assertEqual(delta_validation["evidence_ids_used"], [])

            scorecard = self.load_json(run_dir / "scorecard.json")
            self.assertEqual(scorecard["shortlist_order"], ["alpha", "beta", "gamma"])
            self.assertEqual(scorecard["recommended_agent_id"], "alpha")
        finally:
            tmpdir.cleanup()

    def test_generate_writes_frozen_artifacts_and_deterministic_ordering(self) -> None:
        tmpdir, root, live_seed, scratch_root, registry_path = self.repo_fixture()
        try:
            research_dir = self.research_dir(root)
            run_dir = rna.generate_recommendation(
                seed_file=live_seed,
                research_dir=research_dir,
                run_id=RUN_ID,
                scratch_root=scratch_root,
                registry_path=registry_path,
                now_fn=lambda: GENERATED_AT,
            )

            self.assertEqual(
                sorted(path.name for path in run_dir.iterdir()),
                sorted(
                    [
                        "approval-draft.generated.toml",
                        "candidate-dossiers",
                        "candidate-pool.json",
                        "candidate-validation-results",
                        "comparison.generated.md",
                        "eligible-candidates.json",
                        "run-status.json",
                        "run-summary.md",
                        "scorecard.json",
                        "seed.snapshot.toml",
                        "sources.lock.json",
                    ]
                ),
            )

            run_status = self.load_json(run_dir / "run-status.json")
            self.assert_run_status_schema(run_status)
            self.assertEqual(run_status["status"], "success")
            self.assertEqual(run_status["generated_at"], GENERATED_AT)
            self.assertEqual(run_status["research_dir"], str(research_dir))
            self.assertEqual(run_status["run_dir"], str(run_dir))
            self.assertEqual(run_status["eligible_candidate_ids"], ["alpha", "beta", "gamma", "delta"])
            self.assertEqual(run_status["shortlist_ids"], ["alpha", "beta", "gamma"])
            self.assertEqual(run_status["recommended_agent_id"], "alpha")
            self.assertEqual(
                run_status["candidate_status_counts"],
                {"eligible": 4, "candidate_rejected": 0, "candidate_error": 0},
            )
            for key in rna.APPROVAL_DEPENDENT_METRIC_KEYS:
                self.assertIsNone(run_status["metrics"][key])
            self.assertEqual(run_status["metrics"]["rejected_before_scoring_count"], 0)
            self.assertEqual(run_status["metrics"]["evidence_collection_time_seconds"], 321)
            self.assertEqual(run_status["metrics"]["fetched_source_count"], 12)
            self.assertEqual(run_status["approved_agent_id"], None)
            self.assertEqual(run_status["approval_recorded_at"], None)
            self.assertEqual(run_status["override_reason"], None)
            self.assertEqual(run_status["committed_review_dir"], None)
            self.assertEqual(run_status["committed_packet_path"], None)
            self.assertEqual(run_status["committed_approval_artifact_path"], None)
            self.assertEqual(run_status["errors"], [])

            candidate_pool = self.load_json(run_dir / "candidate-pool.json")
            self.assertEqual(set(candidate_pool.keys()), {"run_id", "candidates"})
            self.assertEqual(candidate_pool["run_id"], RUN_ID)
            self.assertEqual(
                [candidate["agent_id"] for candidate in candidate_pool["candidates"]],
                ["gamma", "beta", "alpha", "delta"],
            )
            self.assertEqual(
                candidate_pool["candidates"],
                [
                    {
                        "agent_id": "gamma",
                        "status": "eligible",
                        "rejection_reasons": [],
                        "error_reasons": [],
                        "shortlisted": True,
                        "recommended": False,
                    },
                    {
                        "agent_id": "beta",
                        "status": "eligible",
                        "rejection_reasons": [],
                        "error_reasons": [],
                        "shortlisted": True,
                        "recommended": False,
                    },
                    {
                        "agent_id": "alpha",
                        "status": "eligible",
                        "rejection_reasons": [],
                        "error_reasons": [],
                        "shortlisted": True,
                        "recommended": True,
                    },
                    {
                        "agent_id": "delta",
                        "status": "eligible",
                        "rejection_reasons": [],
                        "error_reasons": [],
                        "shortlisted": False,
                        "recommended": False,
                    },
                ],
            )

            eligible_candidates = self.load_json(run_dir / "eligible-candidates.json")
            self.assertEqual(set(eligible_candidates.keys()), {"run_id", "eligible_candidates"})
            self.assertEqual(
                [candidate["agent_id"] for candidate in eligible_candidates["eligible_candidates"]],
                ["alpha", "beta", "gamma", "delta"],
            )
            self.assertEqual(
                eligible_candidates["eligible_candidates"][0],
                {
                    "agent_id": "alpha",
                    "scores": {
                        "Adoption & community pull": 3,
                        "CLI product maturity & release activity": 3,
                        "Installability & docs quality": 3,
                        "Reproducibility & access friction": 3,
                        "Architecture fit for this repo": 3,
                        "Capability expansion / future leverage": 2,
                    },
                    "primary_sum": 12,
                    "secondary_sum": 5,
                },
            )

            scorecard = self.load_json(run_dir / "scorecard.json")
            self.assertEqual(
                set(scorecard.keys()),
                {
                    "dimensions",
                    "primary_dimensions",
                    "secondary_dimensions",
                    "shortlist_order",
                    "recommended_agent_id",
                    "candidates",
                },
            )
            self.assertEqual(scorecard["dimensions"], list(rna.DIMENSIONS))
            self.assertEqual(scorecard["primary_dimensions"], list(rna.PRIMARY_DIMENSIONS))
            self.assertEqual(scorecard["secondary_dimensions"], list(rna.SECONDARY_DIMENSIONS))
            self.assertEqual(scorecard["shortlist_order"], ["alpha", "beta", "gamma"])
            self.assertEqual(scorecard["recommended_agent_id"], "alpha")
            self.assertEqual(
                scorecard["candidates"]["alpha"]["notes"],
                "refs=alpha-repo,alpha-pkg,alpha-doc",
            )
            self.assertEqual(
                scorecard["candidates"]["beta"]["notes"],
                "refs=beta-repo,beta-pkg,beta-doc",
            )

            sources_lock = self.load_json(run_dir / "sources.lock.json")
            self.assertEqual(set(sources_lock.keys()), {"run_id", "generated_at", "candidates"})
            self.assertEqual(sources_lock["run_id"], RUN_ID)
            self.assertEqual(sources_lock["generated_at"], GENERATED_AT)
            self.assertEqual(
                [candidate["agent_id"] for candidate in sources_lock["candidates"]],
                ["gamma", "beta", "alpha", "delta"],
            )
            self.assertEqual(set(sources_lock["candidates"][0].keys()), {"agent_id", "evidence_refs", "probe_output_refs"})
            self.assertEqual(sources_lock["candidates"][0]["probe_output_refs"], [])
            self.assertEqual(
                set(sources_lock["candidates"][0]["evidence_refs"][0].keys()),
                {"evidence_id", "kind", "url", "title", "captured_at", "sha256"},
            )

            validation_dir = run_dir / "candidate-validation-results"
            self.assertEqual(
                sorted(path.name for path in validation_dir.glob("*.json")),
                ["alpha.json", "beta.json", "delta.json", "gamma.json"],
            )
            alpha_validation = self.load_json(validation_dir / "alpha.json")
            self.assert_candidate_validation_schema(alpha_validation)
            self.assertEqual(alpha_validation["status"], "eligible")
            self.assertTrue(alpha_validation["schema_valid"])
            self.assertEqual(alpha_validation["probe_results"], [])
            self.assertEqual(alpha_validation["rejection_reasons"], [])
            self.assertEqual(alpha_validation["error_reasons"], [])
            self.assertEqual(alpha_validation["notes"], ["schema validation passed"])
            for gate in alpha_validation["hard_gate_results"].values():
                self.assertEqual(gate["status"], "pass")

            self.assertEqual(
                (run_dir / "run-summary.md").read_text(encoding="utf-8"),
                """# Recommendation Run Summary

- mode: generate
- run_id: 20260427-frozen-contract
- generated_at: 2026-04-27T18:00:00Z
- recommended_agent_id: alpha
- approved_agent_id: pending
- shortlist_ids: alpha, beta, gamma
- metrics:
  - maintainer_time_to_decision_seconds: pending
  - shortlist_override: pending
  - predicted_blocker_count: pending
  - later_discovered_blocker_count: pending
  - rejected_before_scoring_count: 0
  - evidence_collection_time_seconds: 321
  - fetched_source_count: 12
""",
            )

            approval_draft = (run_dir / "approval-draft.generated.toml").read_text(encoding="utf-8")
            self.assertIn('recommended_agent_id = "alpha"', approval_draft)
            self.assertIn('approved_agent_id = "alpha"', approval_draft)
            self.assertIn('approval_commit = "0000000"', approval_draft)
        finally:
            tmpdir.cleanup()

    def test_promote_updates_only_allowed_review_fields_and_preserves_other_bytes(self) -> None:
        tmpdir, root, live_seed, scratch_root, registry_path = self.repo_fixture()
        try:
            research_dir = self.research_dir(root)
            run_dir = rna.generate_recommendation(
                seed_file=live_seed,
                research_dir=research_dir,
                run_id=RUN_ID,
                scratch_root=scratch_root,
                registry_path=registry_path,
                now_fn=lambda: GENERATED_AT,
            )
            scratch_bytes = self.file_bytes(run_dir)
            scratch_status = self.load_json(run_dir / "run-status.json")
            scratch_summary = (run_dir / "run-summary.md").read_text(encoding="utf-8")

            review_dir = rna.promote_recommendation(
                run_dir=run_dir,
                repo_run_root_rel="docs/agents/selection/runs",
                approved_agent_id="beta",
                onboarding_pack_prefix="beta-onboarding",
                override_reason="Prefer the reviewed fallback with the narrower rollout plan.",
                repo_root=root,
                now_fn=lambda: APPROVED_AT,
                git_head_fn=lambda _: "deadbeef",
                validator=lambda *_: None,
            )

            self.assertEqual(
                self.file_bytes(run_dir),
                scratch_bytes,
            )
            self.assertEqual(
                (root / rna.CANONICAL_PACKET_REL).read_bytes(),
                scratch_bytes["comparison.generated.md"],
            )
            self.assertEqual(
                (review_dir / "approval-draft.generated.toml").read_bytes(),
                scratch_bytes["approval-draft.generated.toml"],
            )
            self.assertNotEqual(
                (
                    root
                    / "docs/agents/lifecycle/beta-onboarding/governance/approved-agent.toml"
                ).read_bytes(),
                scratch_bytes["approval-draft.generated.toml"],
            )

            for artifact in (
                "seed.snapshot.toml",
                "candidate-pool.json",
                "eligible-candidates.json",
                "scorecard.json",
                "sources.lock.json",
                "comparison.generated.md",
                "approval-draft.generated.toml",
            ):
                self.assertEqual(
                    (review_dir / artifact).read_bytes(),
                    scratch_bytes[artifact],
                    artifact,
                )
            for dirname in ("candidate-dossiers", "candidate-validation-results"):
                for path in (run_dir / dirname).glob("*.json"):
                    rel = path.relative_to(run_dir).as_posix()
                    self.assertEqual((review_dir / rel).read_bytes(), scratch_bytes[rel], rel)

            review_status = self.load_json(review_dir / "run-status.json")
            self.assert_run_status_schema(review_status)
            for key in (
                "run_id",
                "status",
                "generated_at",
                "research_dir",
                "run_dir",
                "eligible_candidate_ids",
                "shortlist_ids",
                "recommended_agent_id",
                "candidate_status_counts",
                "errors",
            ):
                self.assertEqual(review_status[key], scratch_status[key], key)
            for key in (
                "rejected_before_scoring_count",
                "evidence_collection_time_seconds",
                "fetched_source_count",
            ):
                self.assertEqual(review_status["metrics"][key], scratch_status["metrics"][key], key)
            self.assertEqual(review_status["approved_agent_id"], "beta")
            self.assertEqual(review_status["approval_recorded_at"], APPROVED_AT)
            self.assertEqual(
                review_status["override_reason"],
                "Prefer the reviewed fallback with the narrower rollout plan.",
            )
            self.assertEqual(
                review_status["committed_review_dir"],
                f"docs/agents/selection/runs/{RUN_ID}",
            )
            self.assertEqual(review_status["committed_packet_path"], rna.CANONICAL_PACKET_REL)
            self.assertEqual(
                review_status["committed_approval_artifact_path"],
                "docs/agents/lifecycle/beta-onboarding/governance/approved-agent.toml",
            )
            self.assertEqual(review_status["metrics"]["maintainer_time_to_decision_seconds"], 3600)
            self.assertTrue(review_status["metrics"]["shortlist_override"])
            self.assertEqual(review_status["metrics"]["predicted_blocker_count"], 1)
            self.assertIsNone(review_status["metrics"]["later_discovered_blocker_count"])

            self.assertEqual(
                scratch_summary,
                """# Recommendation Run Summary

- mode: generate
- run_id: 20260427-frozen-contract
- generated_at: 2026-04-27T18:00:00Z
- recommended_agent_id: alpha
- approved_agent_id: pending
- shortlist_ids: alpha, beta, gamma
- metrics:
  - maintainer_time_to_decision_seconds: pending
  - shortlist_override: pending
  - predicted_blocker_count: pending
  - later_discovered_blocker_count: pending
  - rejected_before_scoring_count: 0
  - evidence_collection_time_seconds: 321
  - fetched_source_count: 12
""",
            )
            self.assertEqual(
                (review_dir / "run-summary.md").read_text(encoding="utf-8"),
                """# Recommendation Run Summary

- mode: promote
- run_id: 20260427-frozen-contract
- generated_at: 2026-04-27T18:00:00Z
- recommended_agent_id: alpha
- approved_agent_id: beta
- shortlist_ids: alpha, beta, gamma
- metrics:
  - maintainer_time_to_decision_seconds: 3600
  - shortlist_override: true
  - predicted_blocker_count: 1
  - later_discovered_blocker_count: pending
  - rejected_before_scoring_count: 0
  - evidence_collection_time_seconds: 321
  - fetched_source_count: 12
- override_summary:
  - approved_agent_id: beta
  - recommended_agent_id: alpha
  - override_reason: Prefer the reviewed fallback with the narrower rollout plan.
""",
            )

            final_approval = (
                root / "docs/agents/lifecycle/beta-onboarding/governance/approved-agent.toml"
            ).read_text(encoding="utf-8")
            self.assertIn('recommended_agent_id = "alpha"', final_approval)
            self.assertIn('approved_agent_id = "beta"', final_approval)
            self.assertIn('approval_commit = "deadbeef"', final_approval)
            self.assertIn(
                'override_reason = "Prefer the reviewed fallback with the narrower rollout plan."',
                final_approval,
            )

            self.assertFalse(self.selection_staging_root(root, RUN_ID).exists())
            self.assertFalse(self.lifecycle_staging_root(root, RUN_ID).exists())
        finally:
            tmpdir.cleanup()

    def test_promote_guard_and_rollback_coverage(self) -> None:
        tmpdir, root, live_seed, scratch_root, registry_path = self.repo_fixture()
        final_approval_path = root / "docs/agents/lifecycle/alpha-onboarding/governance/approved-agent.toml"
        final_approval_path.parent.mkdir(parents=True, exist_ok=True)
        final_approval_path.write_text("ORIGINAL APPROVAL\n", encoding="utf-8")
        try:
            research_dir = self.research_dir(root)
            run_dir = rna.generate_recommendation(
                seed_file=live_seed,
                research_dir=research_dir,
                run_id=RUN_ID,
                scratch_root=scratch_root,
                registry_path=registry_path,
                now_fn=lambda: GENERATED_AT,
            )

            with self.assertRaisesRegex(
                rna.RecommendationError,
                "approved_agent_id must be one of the shortlisted 3 candidates",
            ):
                rna.promote_recommendation(
                    run_dir=run_dir,
                    repo_run_root_rel="docs/agents/selection/runs",
                    approved_agent_id="delta",
                    onboarding_pack_prefix="delta-onboarding",
                    override_reason=None,
                    repo_root=root,
                    now_fn=lambda: APPROVED_AT,
                    git_head_fn=lambda _: "deadbeef",
                    validator=lambda *_: None,
                )

            with self.assertRaisesRegex(
                rna.RecommendationError,
                "override_reason is required when approved_agent_id differs from recommended_agent_id",
            ):
                rna.promote_recommendation(
                    run_dir=run_dir,
                    repo_run_root_rel="docs/agents/selection/runs",
                    approved_agent_id="beta",
                    onboarding_pack_prefix="beta-onboarding",
                    override_reason=None,
                    repo_root=root,
                    now_fn=lambda: APPROVED_AT,
                    git_head_fn=lambda _: "deadbeef",
                    validator=lambda *_: None,
                )

            missing_sources = run_dir / "sources.lock.json"
            original_sources = missing_sources.read_bytes()
            missing_sources.unlink()
            try:
                with self.assertRaisesRegex(
                    rna.RecommendationError,
                    "required scratch artifact `sources.lock.json` is missing",
                ):
                    rna.promote_recommendation(
                        run_dir=run_dir,
                        repo_run_root_rel="docs/agents/selection/runs",
                        approved_agent_id="alpha",
                        onboarding_pack_prefix="alpha-onboarding",
                        override_reason=None,
                        repo_root=root,
                        now_fn=lambda: APPROVED_AT,
                        git_head_fn=lambda _: "deadbeef",
                        validator=lambda *_: None,
                    )
            finally:
                rna.write_bytes(missing_sources, original_sources)

            def failing_validator(*_: object) -> None:
                raise rna.RecommendationError("approval validation failed")

            with self.assertRaisesRegex(rna.RecommendationError, "approval validation failed"):
                rna.promote_recommendation(
                    run_dir=run_dir,
                    repo_run_root_rel="docs/agents/selection/runs",
                    approved_agent_id="alpha",
                    onboarding_pack_prefix="alpha-onboarding",
                    override_reason=None,
                    repo_root=root,
                    now_fn=lambda: APPROVED_AT,
                    git_head_fn=lambda _: "deadbeef",
                    validator=failing_validator,
                )
            self.assertEqual((root / rna.CANONICAL_PACKET_REL).read_text(encoding="utf-8"), "ORIGINAL PACKET\n")
            self.assertEqual(final_approval_path.read_text(encoding="utf-8"), "ORIGINAL APPROVAL\n")
            self.assertFalse((root / "docs/agents/selection/runs" / RUN_ID).exists())
            self.assertFalse(self.selection_staging_root(root, RUN_ID).exists())
            self.assertFalse(self.lifecycle_staging_root(root, RUN_ID).exists())

            replace_calls: list[tuple[Path, Path]] = []

            def failing_replace(src: Path, dst: Path) -> None:
                replace_calls.append((src, dst))
                if len(replace_calls) == 1:
                    os.replace(src, dst)
                    return
                raise OSError("replace failed")

            with self.assertRaisesRegex(OSError, "replace failed"):
                rna.promote_recommendation(
                    run_dir=run_dir,
                    repo_run_root_rel="docs/agents/selection/runs",
                    approved_agent_id="alpha",
                    onboarding_pack_prefix="alpha-onboarding",
                    override_reason=None,
                    repo_root=root,
                    now_fn=lambda: APPROVED_AT,
                    git_head_fn=lambda _: "deadbeef",
                    validator=lambda *_: None,
                    replace_fn=failing_replace,
                )
            self.assertEqual((root / rna.CANONICAL_PACKET_REL).read_text(encoding="utf-8"), "ORIGINAL PACKET\n")
            self.assertEqual(final_approval_path.read_text(encoding="utf-8"), "ORIGINAL APPROVAL\n")
            self.assertFalse((root / "docs/agents/selection/runs" / RUN_ID).exists())
            self.assertFalse(self.selection_staging_root(root, RUN_ID).exists())
            self.assertFalse(self.lifecycle_staging_root(root, RUN_ID).exists())
        finally:
            tmpdir.cleanup()


if __name__ == "__main__":
    unittest.main()
