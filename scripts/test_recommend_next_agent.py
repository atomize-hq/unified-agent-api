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
TEMP_ROOT = Path(rna.RECOMMENDATION_TEMP_ROOT_REL)
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

        scratch_root = root / rna.RECOMMENDATION_RUNS_ROOT_REL
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

    def discovery_summary_text(self, *, run_id: str = RUN_ID) -> str:
        lines = [
            f"# Discovery Summary {run_id}",
            "",
            f"Discovery run id: {run_id}",
            "Discovery pass number: 1",
            "Queries used: best AI coding CLI; AI agent CLI tools; developer agent command line",
            "Source classes consulted: web_search_result, official_doc, github, package_registry",
            "Hints file used: none",
            "Web frontier nominations: alpha, beta, gamma, delta",
            "Official-source nominations: alpha, beta, gamma, delta",
            "",
        ]
        for agent_id, display_name in CANDIDATE_ORDER:
            lines.extend(
                [
                    f"## {agent_id} - {display_name}",
                    f"Why it entered the pool: {display_name} appeared in bounded discovery results.",
                    f"First source: https://sources.example.test/{agent_id}/search",
                    "Policy effect: default inclusion policy only.",
                    f"Known caveat: {display_name} still needs research freeze validation.",
                    "",
                ]
            )
        return "\n".join(lines)

    def discovery_sources_lock(self, *, run_id: str = RUN_ID) -> dict[str, object]:
        sources: list[dict[str, object]] = []
        for index, (agent_id, display_name) in enumerate(CANDIDATE_ORDER, start=1):
            web_entry = {
                "candidate_id": agent_id,
                "source_kind": "web_search_result",
                "url": f"https://search.example.test/{agent_id}",
                "title": f"{display_name} search result",
                "captured_at": GENERATED_AT,
                "role": "frontier_signal",
                "query": "best AI coding CLI",
                "rank": index,
            }
            web_entry["sha256"] = rna.sha256_bytes(rna.canonical_json_bytes(rna.canonical_discovery_source_entry(web_entry)))
            sources.append(web_entry)

            doc_entry = {
                "candidate_id": agent_id,
                "source_kind": "official_doc",
                "url": f"https://docs.example.test/{agent_id}",
                "title": f"{display_name} docs",
                "captured_at": GENERATED_AT,
                "role": "docs_surface",
            }
            doc_entry["sha256"] = rna.sha256_bytes(rna.canonical_json_bytes(rna.canonical_discovery_source_entry(doc_entry)))
            sources.append(doc_entry)
        return {"run_id": run_id, "sources": sources}

    def discovery_dir(
        self,
        root: Path,
        *,
        run_id: str = RUN_ID,
        seed_text: str | None = None,
        summary_text: str | None = None,
        sources_lock: dict[str, object] | None = None,
    ) -> Path:
        discovery_dir = root / rna.RECOMMENDATION_TEMP_ROOT_REL / "discovery" / run_id
        discovery_dir.mkdir(parents=True, exist_ok=True)
        rna.write_text(rna.discovery_seed_path(discovery_dir), seed_text or self.seed_text())
        rna.write_text(rna.discovery_summary_path(discovery_dir), summary_text or self.discovery_summary_text(run_id=run_id))
        rna.write_json(rna.discovery_sources_lock_path(discovery_dir), sources_lock or self.discovery_sources_lock(run_id=run_id))
        return discovery_dir

    def research_dir(
        self,
        root: Path,
        *,
        run_id: str = RUN_ID,
        dirname: str | None = None,
        metadata_run_id: str | None = None,
        metadata_override: dict[str, object] | None = None,
        dossier_mutator: Callable[[str, dict[str, object]], dict[str, object]] | None = None,
        include_discovery: bool = True,
    ) -> Path:
        actual_dirname = dirname or run_id
        research_dir = root / rna.RECOMMENDATION_RESEARCH_ROOT_REL / actual_dirname
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
        if include_discovery:
            discovery_input_dir = rna.research_discovery_input_dir(research_dir)
            rna.write_text(rna.discovery_seed_path(discovery_input_dir), self.seed_text())
            rna.write_text(rna.discovery_summary_path(discovery_input_dir), self.discovery_summary_text(run_id=run_id))
            rna.write_json(rna.discovery_sources_lock_path(discovery_input_dir), self.discovery_sources_lock(run_id=run_id))

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
                    evidence_ids=[evidence_ids["doc"], evidence_ids["pkg"]],
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
                    evidence_ids=[evidence_ids["repo"]],
                ),
                "crate_first_fit": self.claim_payload(
                    state=crate,
                    summary=f"{display_name} crate-first fit",
                    evidence_ids=[evidence_ids["repo"]],
                ),
                "reproducibility": self.claim_payload(
                    state=reproducibility,
                    summary=f"{display_name} reproducibility",
                    evidence_ids=[evidence_ids["pkg"], evidence_ids["doc"]],
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

    def claim_payload(
        self,
        *,
        state: str,
        summary: str,
        evidence_ids: list[str],
        blocked_by: list[str] | None = None,
    ) -> dict[str, object]:
        payload: dict[str, object] = {
            "state": state,
            "summary": summary,
            "evidence_ids": evidence_ids,
            "notes": "none",
        }
        if blocked_by is not None:
            payload["blocked_by"] = blocked_by
        return payload

    def load_json(self, path: Path) -> object:
        return json.loads(path.read_text(encoding="utf-8"))

    def file_bytes(self, root: Path) -> dict[str, bytes]:
        return {
            path.relative_to(root).as_posix(): path.read_bytes()
            for path in root.rglob("*")
            if path.is_file()
        }

    def decision_surface_bundle(
        self,
    ) -> tuple[tempfile.TemporaryDirectory[str], rna.SeedConfig, dict[str, dict[str, object]], dict[str, rna.CandidateScore], rna.DecisionSurface]:
        tmpdir, root, _, _, _ = self.repo_fixture()
        research_dir = self.research_dir(root)
        seed = rna.parse_seed_file(research_dir / "seed.snapshot.toml")
        dossiers = {
            agent_id: self.load_json(research_dir / "dossiers" / f"{agent_id}.json")
            for agent_id, _ in CANDIDATE_ORDER
        }
        scores = {
            agent_id: rna.score_candidate(dossier, probe_results=[])
            for agent_id, dossier in dossiers.items()
        }
        surface = rna.build_decision_surface(
            seed=seed,
            shortlist_ids=["alpha", "beta", "gamma"],
            recommended_agent_id="alpha",
            scores={agent_id: scores[agent_id] for agent_id in ("alpha", "beta", "gamma")},
            dossiers=dossiers,
        )
        return tmpdir, seed, dossiers, scores, surface

    def assert_packet_matches_template_provenance(self, packet: str) -> None:
        for index, heading in enumerate(rna.PACKET_SECTION_HEADINGS):
            next_heading = (
                rna.PACKET_SECTION_HEADINGS[index + 1]
                if index + 1 < len(rna.PACKET_SECTION_HEADINGS)
                else None
            )
            self.assertIn(
                rna.packet_template_provenance_line(heading),
                rna.packet_section_slice(packet, heading, next_heading),
                heading,
            )

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
                "workflow_version",
                "next_action",
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
        for gate_key, gate in payload["hard_gate_results"].items():
            self.assertEqual(
                set(gate.keys()),
                {"status", "rule_id", "rejection_reason", "evidence_ids", "notes"},
            )
            self.assertEqual(gate["rule_id"], rna.HARD_GATE_RULES[gate_key].rule_id)

    def selection_staging_root(self, root: Path, run_id: str) -> Path:
        return root / "docs/agents/selection/.staging" / run_id

    def lifecycle_staging_root(self, root: Path, run_id: str) -> Path:
        return root / "docs/agents/lifecycle/.staging" / run_id

    def test_parser_requires_frozen_cli_args(self) -> None:
        parser = rna.build_parser()

        parsed = parser.parse_args(
            [
                "generate",
                "--research-dir",
                "docs/agents/.uaa-temp/recommend-next-agent/research/20260427-frozen-contract",
                "--run-id",
                RUN_ID,
                "--scratch-root",
                "docs/agents/.uaa-temp/recommend-next-agent/runs",
            ]
        )
        self.assertEqual(parsed.command, "generate")
        self.assertEqual(
            parsed.research_dir,
            "docs/agents/.uaa-temp/recommend-next-agent/research/20260427-frozen-contract",
        )

        with self.assertRaises(SystemExit):
            parser.parse_args(
                [
                    "generate",
                    "--run-id",
                    RUN_ID,
                    "--scratch-root",
                    "docs/agents/.uaa-temp/recommend-next-agent/runs",
                ]
            )
        with self.assertRaises(SystemExit):
            parser.parse_args(
                [
                    "generate",
                    "--research-dir",
                    "docs/agents/.uaa-temp/recommend-next-agent/research/20260427-frozen-contract",
                    "--run-id",
                    RUN_ID,
                    "--scratch-root",
                    "docs/agents/.uaa-temp/recommend-next-agent/runs",
                    "--seed-file",
                    "docs/agents/selection/candidate-seed.toml",
                ]
            )
        with self.assertRaises(SystemExit):
            parser.parse_args(
                [
                    "promote",
                    "--run-dir",
                    "docs/agents/.uaa-temp/recommend-next-agent/runs/run",
                    "--repo-run-root",
                    "docs/agents/selection/runs",
                    "--approved-agent-id",
                    "alpha",
                ]
            )

    def test_freeze_discovery_accepts_valid_sources_lock_hashes(self) -> None:
        tmpdir, root, _, _, registry_path = self.repo_fixture()
        try:
            discovery_dir = self.discovery_dir(root)
            research_dir = root / rna.RECOMMENDATION_RESEARCH_ROOT_REL / RUN_ID
            rna.freeze_discovery(discovery_dir=discovery_dir, research_dir=research_dir, registry_path=registry_path)
            self.assertEqual(
                self.load_json(rna.discovery_sources_lock_path(rna.research_discovery_input_dir(research_dir))),
                self.discovery_sources_lock(),
            )
        finally:
            tmpdir.cleanup()

    def test_freeze_discovery_rejects_mismatched_sources_lock_hash(self) -> None:
        tmpdir, root, _, _, registry_path = self.repo_fixture()
        try:
            sources_lock = self.discovery_sources_lock()
            sources_lock["sources"][0]["sha256"] = hex64("wrong-hash")
            discovery_dir = self.discovery_dir(root, sources_lock=sources_lock)
            research_dir = root / rna.RECOMMENDATION_RESEARCH_ROOT_REL / RUN_ID
            with self.assertRaisesRegex(rna.RecommendationError, "sha256 does not match the canonical entry object"):
                rna.freeze_discovery(discovery_dir=discovery_dir, research_dir=research_dir, registry_path=registry_path)
        finally:
            tmpdir.cleanup()

    def test_freeze_discovery_rejects_duplicate_candidate_ids(self) -> None:
        tmpdir, root, _, _, registry_path = self.repo_fixture()
        try:
            duplicate_seed = self.seed_text() + '\n[candidate.alpha]\ndisplay_name = "Alpha Duplicate"\nresearch_urls = ["https://dupe.example"]\ninstall_channels = ["brew install alpha"]\nauth_notes = "dupe"\n'
            discovery_dir = self.discovery_dir(root, seed_text=duplicate_seed)
            research_dir = root / rna.RECOMMENDATION_RESEARCH_ROOT_REL / RUN_ID
            with self.assertRaisesRegex(rna.RecommendationError, "duplicate candidate ids"):
                rna.freeze_discovery(discovery_dir=discovery_dir, research_dir=research_dir, registry_path=registry_path)
        finally:
            tmpdir.cleanup()

    def test_freeze_discovery_rejects_onboarded_agent_ids(self) -> None:
        tmpdir, root, _, _, registry_path = self.repo_fixture()
        try:
            onboarded_seed = self.seed_text().replace("[candidate.delta]", "[candidate.codex]")
            onboarded_seed = onboarded_seed.replace('display_name = "Delta CLI"', 'display_name = "Codex CLI"')
            onboarded_seed = onboarded_seed.replace("delta", "codex")
            onboarded_summary = self.discovery_summary_text().replace("delta", "codex").replace("Delta CLI", "Codex CLI")
            onboarded_sources = self.discovery_sources_lock()
            for entry in onboarded_sources["sources"]:
                if entry["candidate_id"] == "delta":
                    entry["candidate_id"] = "codex"
                    entry["url"] = str(entry["url"]).replace("delta", "codex")
                    entry["title"] = str(entry["title"]).replace("Delta CLI", "Codex CLI")
                    if entry["source_kind"] == "web_search_result":
                        entry["sha256"] = rna.sha256_bytes(rna.canonical_json_bytes(rna.canonical_discovery_source_entry(entry)))
                    else:
                        entry["sha256"] = rna.sha256_bytes(rna.canonical_json_bytes(rna.canonical_discovery_source_entry(entry)))
            discovery_dir = self.discovery_dir(
                root,
                seed_text=onboarded_seed,
                summary_text=onboarded_summary,
                sources_lock=onboarded_sources,
            )
            research_dir = root / rna.RECOMMENDATION_RESEARCH_ROOT_REL / RUN_ID
            with self.assertRaisesRegex(rna.RecommendationError, "already onboarded agent ids: codex"):
                rna.freeze_discovery(discovery_dir=discovery_dir, research_dir=research_dir, registry_path=registry_path)
        finally:
            tmpdir.cleanup()

    def test_freeze_discovery_rejects_unsupported_source_kind(self) -> None:
        tmpdir, root, _, _, registry_path = self.repo_fixture()
        try:
            sources_lock = self.discovery_sources_lock()
            sources_lock["sources"][0]["source_kind"] = "reddit"
            sources_lock["sources"][0].pop("query")
            sources_lock["sources"][0].pop("rank")
            discovery_dir = self.discovery_dir(root, sources_lock=sources_lock)
            research_dir = root / rna.RECOMMENDATION_RESEARCH_ROOT_REL / RUN_ID
            with self.assertRaisesRegex(rna.RecommendationError, "unsupported source_kind `reddit`"):
                rna.freeze_discovery(discovery_dir=discovery_dir, research_dir=research_dir, registry_path=registry_path)
        finally:
            tmpdir.cleanup()

    def test_freeze_discovery_copies_reviewed_seed_and_provenance(self) -> None:
        tmpdir, root, _, _, registry_path = self.repo_fixture()
        try:
            discovery_dir = self.discovery_dir(root)
            research_dir = root / rna.RECOMMENDATION_RESEARCH_ROOT_REL / RUN_ID
            rna.freeze_discovery(discovery_dir=discovery_dir, research_dir=research_dir, registry_path=registry_path)
            self.assertEqual(
                rna.seed_snapshot_path(research_dir).read_bytes(),
                rna.discovery_seed_path(discovery_dir).read_bytes(),
            )
            for filename in rna.DISCOVERY_REQUIRED_FILENAMES:
                self.assertEqual(
                    (rna.research_discovery_input_dir(research_dir) / filename).read_bytes(),
                    (discovery_dir / filename).read_bytes(),
                )
        finally:
            tmpdir.cleanup()

    def test_generate_reads_research_seed_snapshot_instead_of_live_candidate_seed(self) -> None:
        tmpdir, root, live_seed, scratch_root, registry_path = self.repo_fixture()
        try:
            live_seed.write_text("not valid toml\n", encoding="utf-8")
            research_dir = self.research_dir(root)
            run_dir = rna.generate_recommendation(
                research_dir=research_dir,
                run_id=RUN_ID,
                scratch_root=scratch_root,
                registry_path=registry_path,
                now_fn=lambda: GENERATED_AT,
            )
            self.assertEqual(self.load_json(run_dir / "run-status.json")["status"], "success")
        finally:
            tmpdir.cleanup()

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

    def test_generate_surfaces_seed_snapshot_sha_mismatch_as_candidate_error(self) -> None:
        tmpdir, root, live_seed, scratch_root, registry_path = self.repo_fixture()
        try:
            research_dir = self.research_dir(
                root,
                dossier_mutator=lambda agent_id, dossier: (
                    {**dossier, "seed_snapshot_sha256": hex64("wrong-snapshot")} if agent_id == "delta" else dossier
                ),
            )
            run_dir = rna.generate_recommendation(
                research_dir=research_dir,
                run_id=RUN_ID,
                scratch_root=scratch_root,
                registry_path=registry_path,
                now_fn=lambda: GENERATED_AT,
            )

            delta_validation = self.load_json(run_dir / "candidate-validation-results" / "delta.json")
            self.assertEqual(delta_validation["status"], "candidate_error")
            self.assertIn("seed_snapshot_sha256 does not match the run snapshot", delta_validation["error_reasons"][0])
        finally:
            tmpdir.cleanup()

    def test_research_metadata_rejects_run_dir_basename_mismatch(self) -> None:
        tmpdir, root, _, _, _ = self.repo_fixture()
        try:
            research_dir = self.research_dir(root)
            with self.assertRaisesRegex(
                rna.RecommendationError,
                "run directory basename must equal CLI --run-id",
            ):
                rna.validate_research_metadata(
                    research_metadata_path=research_dir / "research-metadata.json",
                    expected_run_id=RUN_ID,
                    research_dir=research_dir,
                    run_dir=root / "scratch" / "wrong-run-dir",
                )
        finally:
            tmpdir.cleanup()

    def test_template_parity_lock_covers_all_packet_sections(self) -> None:
        provenance_lines = rna.packet_template_provenance_lines()
        self.assertEqual(tuple(provenance_lines.keys()), rna.PACKET_SECTION_HEADINGS)
        self.assertEqual(
            provenance_lines["## 1. Candidate Summary"],
            "Provenance: `<committed repo evidence | dated external snapshot evidence | maintainer inference>`",
        )
        self.assertEqual(
            provenance_lines["## 5. Recommendation"],
            "Provenance: `maintainer inference grounded in the comparison table`",
        )

    def test_packet_contract_rejects_missing_provenance_line(self) -> None:
        tmpdir, root, live_seed, scratch_root, registry_path = self.repo_fixture()
        try:
            research_dir = self.research_dir(root)
            run_dir = rna.generate_recommendation(
                research_dir=research_dir,
                run_id=RUN_ID,
                scratch_root=scratch_root,
                registry_path=registry_path,
                now_fn=lambda: GENERATED_AT,
            )
            packet = (run_dir / "comparison.generated.md").read_text(encoding="utf-8")
            tampered = packet.replace(
                rna.packet_template_provenance_line("## 5. Recommendation") + "\n",
                "",
                1,
            )
            with self.assertRaisesRegex(
                rna.RecommendationError,
                "missing required provenance line",
            ):
                rna.validate_packet_contract(
                    packet=tampered,
                    shortlist_ids=["alpha", "beta", "gamma"],
                    seeded_ids=[agent_id for agent_id, _ in CANDIDATE_ORDER],
                    candidate_results={
                        agent_id: rna.build_placeholder_candidate_result(agent_id)
                        for agent_id, _ in CANDIDATE_ORDER
                    },
                )
        finally:
            tmpdir.cleanup()

    def test_generate_emits_template_provenance_lines_for_all_sections(self) -> None:
        tmpdir, root, live_seed, scratch_root, registry_path = self.repo_fixture()
        try:
            research_dir = self.research_dir(root)
            run_dir = rna.generate_recommendation(
                research_dir=research_dir,
                run_id=RUN_ID,
                scratch_root=scratch_root,
                registry_path=registry_path,
                now_fn=lambda: GENERATED_AT,
            )
            packet = (run_dir / "comparison.generated.md").read_text(encoding="utf-8")
            self.assert_packet_matches_template_provenance(packet)
        finally:
            tmpdir.cleanup()

    def test_hard_gate_rejects_inferred_non_interactive_execution(self) -> None:
        dossier = self.dossier_payload(
            agent_id="alpha",
            display_name="Alpha CLI",
            snapshot_sha=hex64("snapshot"),
            non_interactive="inferred",
            offline="verified",
            observable="verified",
            redaction="verified",
            crate="verified",
            reproducibility="verified",
            future="inferred",
            blocked_steps=[],
        )
        gate_result, failed, evidence_sufficient = rna.evaluate_hard_gate(
            dossier=dossier,
            gate_key="non_interactive_execution",
            probe_results=[],
        )

        self.assertTrue(failed)
        self.assertFalse(evidence_sufficient)
        self.assertEqual(gate_result["status"], "fail")
        self.assertEqual(
            gate_result["rule_id"],
            "hard_gate.non_interactive_execution.verified_doc_and_package_or_probe",
        )
        self.assertEqual(
            gate_result["rejection_reason"],
            "claim state `inferred` does not satisfy rule `hard_gate.non_interactive_execution.verified_doc_and_package_or_probe`",
        )

    def test_hard_gate_rejects_reproducibility_without_doc_and_package_evidence(self) -> None:
        dossier = self.dossier_payload(
            agent_id="alpha",
            display_name="Alpha CLI",
            snapshot_sha=hex64("snapshot"),
            non_interactive="verified",
            offline="verified",
            observable="verified",
            redaction="verified",
            crate="verified",
            reproducibility="verified",
            future="inferred",
            blocked_steps=[],
        )
        dossier["claims"]["reproducibility"]["evidence_ids"] = ["alpha-pkg"]

        gate_result, failed, evidence_sufficient = rna.evaluate_hard_gate(
            dossier=dossier,
            gate_key="reproducibility",
            probe_results=[],
        )

        self.assertTrue(failed)
        self.assertFalse(evidence_sufficient)
        self.assertEqual(gate_result["status"], "fail")
        self.assertEqual(
            gate_result["rule_id"],
            "hard_gate.reproducibility.doc_and_package",
        )
        self.assertEqual(
            gate_result["rejection_reason"],
            "missing required evidence kinds: official_doc",
        )

    def test_hard_gate_rejects_inferred_allowed_claim_when_blocked_by_is_present(self) -> None:
        dossier = self.dossier_payload(
            agent_id="gamma",
            display_name="Gamma CLI",
            snapshot_sha=hex64("snapshot"),
            non_interactive="verified",
            offline="inferred",
            observable="verified",
            redaction="verified",
            crate="verified",
            reproducibility="verified",
            future="inferred",
            blocked_steps=[],
        )
        dossier["claims"]["offline_strategy"]["blocked_by"] = ["paid hosted mode review"]

        gate_result, failed, evidence_sufficient = rna.evaluate_hard_gate(
            dossier=dossier,
            gate_key="offline_strategy",
            probe_results=[],
        )

        self.assertTrue(failed)
        self.assertFalse(evidence_sufficient)
        self.assertEqual(gate_result["status"], "blocked")
        self.assertEqual(
            gate_result["rule_id"],
            "hard_gate.offline_strategy.doc_or_repo_backed",
        )
        self.assertEqual(
            gate_result["rejection_reason"],
            "claim is blocked by dossier dependencies: paid hosted mode review",
        )

    def test_generate_handles_required_probes_under_existing_schema(self) -> None:
        tmpdir, root, live_seed, scratch_root, registry_path = self.repo_fixture()
        try:
            research_dir = self.research_dir(
                root,
                dossier_mutator=lambda agent_id, dossier: (
                    {
                        **dossier,
                        "probe_requests": [
                            {"probe_kind": "version", "binary": "python3", "required_for_gate": True},
                            {"probe_kind": "version", "binary": "python3", "required_for_gate": False},
                        ],
                    }
                    if agent_id == "alpha"
                    else (
                        {
                            **dossier,
                            "claims": {
                                **dossier["claims"],
                                "observable_cli_surface": {
                                    **dossier["claims"]["observable_cli_surface"],
                                    "evidence_ids": [],
                                },
                            },
                        }
                        if agent_id == "delta"
                        else dossier
                    )
                ),
            )
            run_dir = rna.generate_recommendation(
                research_dir=research_dir,
                run_id=RUN_ID,
                scratch_root=scratch_root,
                registry_path=registry_path,
                now_fn=lambda: GENERATED_AT,
            )

            alpha_validation = self.load_json(run_dir / "candidate-validation-results" / "alpha.json")
            self.assertEqual(alpha_validation["status"], "eligible")
            self.assertEqual(len(alpha_validation["probe_results"]), 2)
            self.assertEqual(alpha_validation["probe_results"][0]["status"], "passed")
            self.assertTrue(alpha_validation["probe_results"][0]["required_for_gate"])
            self.assertEqual(alpha_validation["probe_results"][1]["status"], "passed")
            self.assertIn(
                "alpha:version:0",
                alpha_validation["hard_gate_results"]["observable_cli_surface"]["evidence_ids"],
            )
            self.assertIn(
                "alpha:version:1",
                alpha_validation["hard_gate_results"]["observable_cli_surface"]["evidence_ids"],
            )
        finally:
            tmpdir.cleanup()

    def test_validate_decision_surface_rejects_missing_loser_rationale(self) -> None:
        tmpdir, _, _, _, surface = self.decision_surface_bundle()
        try:
            invalid = rna.DecisionSurface(
                winner_agent_id=surface.winner_agent_id,
                winner_display_name=surface.winner_display_name,
                winner_rationale=surface.winner_rationale,
                loser_rationales={"gamma": surface.loser_rationales["gamma"]},
                section6_reproducible_now=surface.section6_reproducible_now,
                section6_blocked_until_later=surface.section6_blocked_until_later,
                section7=surface.section7,
                section8=surface.section8,
                section9=surface.section9,
            )
            with self.assertRaisesRegex(
                rna.RecommendationError,
                "missing explicit loser rationale for shortlisted non-winner `beta`",
            ):
                rna.validate_decision_surface(
                    invalid,
                    shortlist_ids=["alpha", "beta", "gamma"],
                    recommended_agent_id="alpha",
                )
        finally:
            tmpdir.cleanup()

    def test_validate_decision_surface_rejects_section6_without_substance(self) -> None:
        tmpdir, _, _, _, surface = self.decision_surface_bundle()
        try:
            cases = (
                (
                    "commands",
                    rna.DecisionSurface(
                        winner_agent_id=surface.winner_agent_id,
                        winner_display_name=surface.winner_display_name,
                        winner_rationale=surface.winner_rationale,
                        loser_rationales=surface.loser_rationales,
                        section6_reproducible_now={
                            **surface.section6_reproducible_now,
                            "runnable commands": [],
                        },
                        section6_blocked_until_later=surface.section6_blocked_until_later,
                        section7=surface.section7,
                        section8=surface.section8,
                        section9=surface.section9,
                    ),
                    "section 6 must include real commands",
                ),
                (
                    "evidence",
                    rna.DecisionSurface(
                        winner_agent_id=surface.winner_agent_id,
                        winner_display_name=surface.winner_display_name,
                        winner_rationale=surface.winner_rationale,
                        loser_rationales=surface.loser_rationales,
                        section6_reproducible_now={
                            **surface.section6_reproducible_now,
                            "evidence gatherable without paid or elevated access": [],
                        },
                        section6_blocked_until_later=surface.section6_blocked_until_later,
                        section7=surface.section7,
                        section8=surface.section8,
                        section9=surface.section9,
                    ),
                    "section 6 must include gatherable evidence",
                ),
                (
                    "blockers",
                    rna.DecisionSurface(
                        winner_agent_id=surface.winner_agent_id,
                        winner_display_name=surface.winner_display_name,
                        winner_rationale=surface.winner_rationale,
                        loser_rationales=surface.loser_rationales,
                        section6_reproducible_now=surface.section6_reproducible_now,
                        section6_blocked_until_later=[],
                        section7=surface.section7,
                        section8=surface.section8,
                        section9=surface.section9,
                    ),
                    "section 6 must include blocked steps",
                ),
            )
            for name, invalid_surface, expected_message in cases:
                with self.subTest(case=name):
                    with self.assertRaisesRegex(rna.RecommendationError, expected_message):
                        rna.validate_decision_surface(
                            invalid_surface,
                            shortlist_ids=["alpha", "beta", "gamma"],
                            recommended_agent_id="alpha",
                        )
        finally:
            tmpdir.cleanup()

    def test_section7_8_9_locked_labels_and_nonempty_content_are_enforced(self) -> None:
        tmpdir, root, live_seed, scratch_root, registry_path = self.repo_fixture()
        try:
            research_dir = self.research_dir(root)
            run_dir = rna.generate_recommendation(
                research_dir=research_dir,
                run_id=RUN_ID,
                scratch_root=scratch_root,
                registry_path=registry_path,
                now_fn=lambda: GENERATED_AT,
            )
            packet = (run_dir / "comparison.generated.md").read_text(encoding="utf-8")
            exact_label = "Manifest root expectations:"
            self.assertIn(exact_label, packet)
            tampered_label = packet.replace(exact_label, "Manifest-root expectations:", 1)
            with self.assertRaisesRegex(
                rna.RecommendationError,
                "comparison packet section 7 labels do not match the locked template",
            ):
                rna.validate_packet_contract(
                    packet=tampered_label,
                    shortlist_ids=["alpha", "beta", "gamma"],
                    seeded_ids=[agent_id for agent_id, _ in CANDIDATE_ORDER],
                    candidate_results={
                        agent_id: rna.build_placeholder_candidate_result(agent_id)
                        for agent_id, _ in CANDIDATE_ORDER
                    },
                )

            emptied = packet.replace(
                "Required deliverables:\n- approved comparison packet and governance artifact\n- wrapper crate code, tests, and manifest outputs\n- backend adapter code, tests, and updated repo evidence\n",
                "Required deliverables:\n",
                1,
            )
            with self.assertRaisesRegex(
                rna.RecommendationError,
                "comparison packet section 9 subsection `Required deliverables` must not be empty",
            ):
                rna.validate_packet_contract(
                    packet=emptied,
                    shortlist_ids=["alpha", "beta", "gamma"],
                    seeded_ids=[agent_id for agent_id, _ in CANDIDATE_ORDER],
                    candidate_results={
                        agent_id: rna.build_placeholder_candidate_result(agent_id)
                        for agent_id, _ in CANDIDATE_ORDER
                    },
                )
        finally:
            tmpdir.cleanup()

    def test_generate_writes_frozen_artifacts_and_deterministic_ordering(self) -> None:
        tmpdir, root, live_seed, scratch_root, registry_path = self.repo_fixture()
        try:
            research_dir = self.research_dir(root)
            run_dir = rna.generate_recommendation(
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
                        "discovery",
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
            self.assertEqual(run_status["workflow_version"], rna.WORKFLOW_VERSION_V2_DISCOVERY)
            self.assertIsNone(run_status["next_action"])
            self.assertEqual(run_status["eligible_candidate_ids"], ["alpha", "beta", "gamma"])
            self.assertEqual(run_status["shortlist_ids"], ["alpha", "beta", "gamma"])
            self.assertEqual(run_status["recommended_agent_id"], "alpha")
            self.assertEqual(
                run_status["candidate_status_counts"],
                {"eligible": 3, "candidate_rejected": 1, "candidate_error": 0},
            )
            for key in rna.APPROVAL_DEPENDENT_METRIC_KEYS:
                self.assertIsNone(run_status["metrics"][key])
            self.assertEqual(run_status["metrics"]["rejected_before_scoring_count"], 1)
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
                        "status": "candidate_rejected",
                        "rejection_reasons": [
                            "hard_gate.observable_cli_surface.verified_doc_or_repo_or_probe: claim state `inferred` does not satisfy rule `hard_gate.observable_cli_surface.verified_doc_or_repo_or_probe`",
                        ],
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
                ["alpha", "beta", "gamma"],
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
            for filename in rna.DISCOVERY_REQUIRED_FILENAMES:
                self.assertEqual(
                    (run_dir / rna.DISCOVERY_DIRNAME / filename).read_bytes(),
                    (rna.research_discovery_input_dir(research_dir) / filename).read_bytes(),
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
                self.assertEqual(gate["rejection_reason"], "")

            self.assertEqual(
                (run_dir / "run-summary.md").read_text(encoding="utf-8"),
                """# Recommendation Run Summary

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
  - rejected_before_scoring_count: 1
  - evidence_collection_time_seconds: 321
  - fetched_source_count: 12
""",
            )

            approval_draft = (run_dir / "approval-draft.generated.toml").read_text(encoding="utf-8")
            self.assertIn('recommended_agent_id = "alpha"', approval_draft)
            self.assertIn('approved_agent_id = "alpha"', approval_draft)
            self.assertIn('approval_commit = "0000000"', approval_draft)

            comparison_packet = (run_dir / "comparison.generated.md").read_text(encoding="utf-8")
            self.assertIn("# Packet - CLI Agent Selection Packet", comparison_packet)
            self.assertIn("Owner(s): wrappers team / deterministic runner", comparison_packet)
            self.assert_packet_matches_template_provenance(comparison_packet)
            self.assertIn("| `alpha` | 3 | 3 | 3 | 3 | 3 | 2 | refs=alpha-repo,alpha-pkg,alpha-doc |", comparison_packet)
            self.assertIn("Approve recommended agent", comparison_packet)
            self.assertIn("Override to shortlisted alternative", comparison_packet)
            self.assertIn("Stop and expand research", comparison_packet)
            self.assertIn("reproducible now:", comparison_packet)
            self.assertIn("- auth / account / billing prerequisites:", comparison_packet)
            self.assertIn("blocked until later:", comparison_packet)
            self.assertIn("### Strategic Contenders", comparison_packet)
        finally:
            tmpdir.cleanup()

    def test_generate_emits_next_action_expand_discovery_on_insufficiency(self) -> None:
        tmpdir, root, _, scratch_root, registry_path = self.repo_fixture()
        try:
            research_dir = self.research_dir(
                root,
                dossier_mutator=lambda agent_id, dossier: (
                    {
                        **dossier,
                        "claims": {
                            **dossier["claims"],
                            "observable_cli_surface": {
                                **dossier["claims"]["observable_cli_surface"],
                                "state": "inferred",
                            },
                        },
                    }
                    if agent_id in {"beta", "gamma"}
                    else dossier
                ),
            )
            with self.assertRaisesRegex(rna.RecommendationError, "fewer than 3 eligible candidates remain after gating"):
                rna.generate_recommendation(
                    research_dir=research_dir,
                    run_id=RUN_ID,
                    scratch_root=scratch_root,
                    registry_path=registry_path,
                    now_fn=lambda: GENERATED_AT,
                )
            run_status = self.load_json(scratch_root / RUN_ID / "run-status.json")
            self.assertEqual(run_status["status"], "insufficient_eligible_candidates")
            self.assertEqual(run_status["next_action"], "expand_discovery")
            self.assertEqual(run_status["workflow_version"], rna.WORKFLOW_VERSION_V2_DISCOVERY)
        finally:
            tmpdir.cleanup()

    def test_generate_emits_next_action_stop_on_pass2_insufficiency(self) -> None:
        pass2_run_id = f"{RUN_ID}-pass2"
        tmpdir, root, _, scratch_root, registry_path = self.repo_fixture()
        try:
            research_dir = self.research_dir(
                root,
                run_id=pass2_run_id,
                dossier_mutator=lambda agent_id, dossier: (
                    {
                        **dossier,
                        "claims": {
                            **dossier["claims"],
                            "observable_cli_surface": {
                                **dossier["claims"]["observable_cli_surface"],
                                "state": "inferred",
                            },
                        },
                    }
                    if agent_id in {"beta", "gamma"}
                    else dossier
                ),
            )
            with self.assertRaisesRegex(rna.RecommendationError, "fewer than 3 eligible candidates remain after gating"):
                rna.generate_recommendation(
                    research_dir=research_dir,
                    run_id=pass2_run_id,
                    scratch_root=scratch_root,
                    registry_path=registry_path,
                    now_fn=lambda: GENERATED_AT,
                )
            run_status = self.load_json(scratch_root / pass2_run_id / "run-status.json")
            self.assertEqual(run_status["status"], "insufficient_eligible_candidates")
            self.assertEqual(run_status["next_action"], "stop")
        finally:
            tmpdir.cleanup()

    def test_generate_copies_discovery_provenance_into_scratch_run_on_insufficiency(self) -> None:
        tmpdir, root, _, scratch_root, registry_path = self.repo_fixture()
        try:
            research_dir = self.research_dir(
                root,
                dossier_mutator=lambda agent_id, dossier: (
                    {
                        **dossier,
                        "claims": {
                            **dossier["claims"],
                            "observable_cli_surface": {
                                **dossier["claims"]["observable_cli_surface"],
                                "state": "inferred",
                            },
                        },
                    }
                    if agent_id in {"beta", "gamma"}
                    else dossier
                ),
            )
            with self.assertRaises(rna.RecommendationError):
                rna.generate_recommendation(
                    research_dir=research_dir,
                    run_id=RUN_ID,
                    scratch_root=scratch_root,
                    registry_path=registry_path,
                    now_fn=lambda: GENERATED_AT,
                )
            run_dir = scratch_root / RUN_ID
            for filename in rna.DISCOVERY_REQUIRED_FILENAMES:
                self.assertEqual(
                    (run_dir / rna.DISCOVERY_DIRNAME / filename).read_bytes(),
                    (rna.research_discovery_input_dir(research_dir) / filename).read_bytes(),
                )
        finally:
            tmpdir.cleanup()

    def test_insufficiency_run_writes_required_and_omits_forbidden_artifacts(self) -> None:
        tmpdir, root, _, scratch_root, registry_path = self.repo_fixture()
        try:
            research_dir = self.research_dir(
                root,
                dossier_mutator=lambda agent_id, dossier: (
                    {
                        **dossier,
                        "claims": {
                            **dossier["claims"],
                            "observable_cli_surface": {
                                **dossier["claims"]["observable_cli_surface"],
                                "state": "inferred",
                            },
                        },
                    }
                    if agent_id in {"beta", "gamma"}
                    else dossier
                ),
            )
            with self.assertRaises(rna.RecommendationError):
                rna.generate_recommendation(
                    research_dir=research_dir,
                    run_id=RUN_ID,
                    scratch_root=scratch_root,
                    registry_path=registry_path,
                    now_fn=lambda: GENERATED_AT,
                )
            run_dir = scratch_root / RUN_ID
            for required in (
                "run-status.json",
                "seed.snapshot.toml",
                "candidate-pool.json",
                "eligible-candidates.json",
                "run-summary.md",
            ):
                self.assertTrue((run_dir / required).exists(), required)
            self.assertTrue((run_dir / "candidate-validation-results").is_dir())
            self.assertTrue((run_dir / "candidate-dossiers").is_dir())
            self.assertTrue((run_dir / rna.DISCOVERY_DIRNAME).is_dir())
            for agent_id, _ in CANDIDATE_ORDER:
                self.assertTrue((run_dir / "candidate-validation-results" / f"{agent_id}.json").exists())
                self.assertTrue((run_dir / "candidate-dossiers" / f"{agent_id}.json").exists())
            for forbidden in (
                "scorecard.json",
                "sources.lock.json",
                "comparison.generated.md",
                "approval-draft.generated.toml",
            ):
                self.assertFalse((run_dir / forbidden).exists(), forbidden)
            summary = (run_dir / "run-summary.md").read_text(encoding="utf-8")
            self.assertIn("- next_action: expand_discovery", summary)
            self.assertIn("already_onboarded: none", summary)
            self.assertIn("missing_public_cli_surface: beta, delta, gamma", summary)
        finally:
            tmpdir.cleanup()

    def test_promote_copies_discovery_provenance_into_committed_review_run(self) -> None:
        tmpdir, root, _, scratch_root, registry_path = self.repo_fixture()
        try:
            research_dir = self.research_dir(root)
            run_dir = rna.generate_recommendation(
                research_dir=research_dir,
                run_id=RUN_ID,
                scratch_root=scratch_root,
                registry_path=registry_path,
                now_fn=lambda: GENERATED_AT,
            )
            review_dir = rna.promote_recommendation(
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
            for filename in rna.DISCOVERY_REQUIRED_FILENAMES:
                self.assertEqual(
                    (review_dir / rna.DISCOVERY_DIRNAME / filename).read_bytes(),
                    (run_dir / rna.DISCOVERY_DIRNAME / filename).read_bytes(),
                )
        finally:
            tmpdir.cleanup()

    def test_promote_fails_for_v2_marked_run_missing_discovery(self) -> None:
        tmpdir, root, _, scratch_root, registry_path = self.repo_fixture()
        try:
            research_dir = self.research_dir(root)
            run_dir = rna.generate_recommendation(
                research_dir=research_dir,
                run_id=RUN_ID,
                scratch_root=scratch_root,
                registry_path=registry_path,
                now_fn=lambda: GENERATED_AT,
            )
            rna.remove_path(run_dir / rna.DISCOVERY_DIRNAME)
            with self.assertRaisesRegex(rna.RecommendationError, "discovery artifact directory"):
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
            tmpdir.cleanup()

    def test_legacy_run_without_v2_marker_remains_promotable(self) -> None:
        tmpdir, root, _, scratch_root, registry_path = self.repo_fixture()
        try:
            research_dir = self.research_dir(root, include_discovery=False)
            run_dir = rna.generate_recommendation(
                research_dir=research_dir,
                run_id=RUN_ID,
                scratch_root=scratch_root,
                registry_path=registry_path,
                now_fn=lambda: GENERATED_AT,
            )
            run_status = self.load_json(run_dir / "run-status.json")
            self.assertIsNone(run_status["workflow_version"])
            self.assertFalse((run_dir / rna.DISCOVERY_DIRNAME).exists())
            review_dir = rna.promote_recommendation(
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
            self.assertFalse((review_dir / rna.DISCOVERY_DIRNAME).exists())
        finally:
            tmpdir.cleanup()

    def test_promote_updates_only_allowed_review_fields_and_preserves_other_bytes(self) -> None:
        tmpdir, root, live_seed, scratch_root, registry_path = self.repo_fixture()
        try:
            research_dir = self.research_dir(root)
            run_dir = rna.generate_recommendation(
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
            for filename in rna.DISCOVERY_REQUIRED_FILENAMES:
                rel = f"{rna.DISCOVERY_DIRNAME}/{filename}"
                self.assertEqual((review_dir / rel).read_bytes(), scratch_bytes[rel], rel)
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
                "workflow_version",
                "next_action",
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
  - rejected_before_scoring_count: 1
  - evidence_collection_time_seconds: 321
  - fetched_source_count: 12
""",
            )
            self.assertEqual(
                (review_dir / "run-summary.md").read_text(encoding="utf-8"),
                """# Recommendation Run Summary

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
  - rejected_before_scoring_count: 1
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
                    "run artifact set does not match the frozen contract",
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
