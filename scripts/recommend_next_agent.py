#!/usr/bin/env python3
from __future__ import annotations

import argparse
from collections.abc import Callable
from dataclasses import dataclass
from datetime import datetime, timezone
from functools import lru_cache
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tomllib
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[1]
CANONICAL_PACKET_REL = "docs/agents/selection/cli-agent-selection-packet.md"
PACKET_TEMPLATE_REL = "docs/templates/agent-selection/cli-agent-selection-packet-template.md"
LIVE_SEED_REL = "docs/agents/selection/candidate-seed.toml"
REGISTRY_RELATIVE_PATH = "crates/xtask/data/agent_registry.toml"
RECOMMENDATION_TEMP_ROOT_REL = "docs/agents/.uaa-temp/recommend-next-agent"
RECOMMENDATION_RESEARCH_ROOT_REL = f"{RECOMMENDATION_TEMP_ROOT_REL}/research"
RECOMMENDATION_RUNS_ROOT_REL = f"{RECOMMENDATION_TEMP_ROOT_REL}/runs"
DEFAULT_TARGET = "darwin-arm64"
APPROVAL_VERSION = "1"
SELECTION_MODE = "factory_validation"
TIMESTAMP_PATTERN = re.compile(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$")
HEX64_PATTERN = re.compile(r"^[0-9a-f]{64}$")
SAFE_BINARY_PATTERN = re.compile(r"^[A-Za-z0-9._-]+$")
PRIMARY_DIMENSIONS = (
    "Adoption & community pull",
    "CLI product maturity & release activity",
    "Installability & docs quality",
    "Reproducibility & access friction",
)
SECONDARY_DIMENSIONS = (
    "Architecture fit for this repo",
    "Capability expansion / future leverage",
)
DIMENSIONS = PRIMARY_DIMENSIONS + SECONDARY_DIMENSIONS
REQUIRED_DEFAULT_KEYS = {
    "canonical_targets",
    "wrapper_coverage_binding_kind",
    "always_on_capabilities",
    "target_gated_capabilities",
    "config_gated_capabilities",
    "backend_extensions",
    "support_matrix_enabled",
    "capability_matrix_enabled",
    "capability_matrix_target",
    "docs_release_track",
}
REQUIRED_CANDIDATE_KEYS = {
    "display_name",
    "research_urls",
    "install_channels",
    "auth_notes",
}
OPTIONAL_CANDIDATE_KEYS = {
    "crate_path",
    "backend_module",
    "manifest_root",
    "package_name",
    "canonical_targets",
    "wrapper_coverage_binding_kind",
    "wrapper_coverage_source_path",
    "always_on_capabilities",
    "target_gated_capabilities",
    "config_gated_capabilities",
    "backend_extensions",
    "support_matrix_enabled",
    "capability_matrix_enabled",
    "capability_matrix_target",
    "docs_release_track",
}
DOSSIER_REQUIRED_KEYS = {
    "schema_version",
    "agent_id",
    "display_name",
    "generated_at",
    "seed_snapshot_sha256",
    "official_links",
    "install_channels",
    "auth_prerequisites",
    "claims",
    "probe_requests",
    "blocked_steps",
    "normalized_caveats",
    "evidence",
}
CLAIM_KEYS = (
    "non_interactive_execution",
    "offline_strategy",
    "observable_cli_surface",
    "redaction_fit",
    "crate_first_fit",
    "reproducibility",
    "future_leverage",
)
HARD_GATE_KEYS = (
    "non_interactive_execution",
    "offline_strategy",
    "observable_cli_surface",
    "redaction_fit",
    "crate_first_fit",
    "reproducibility",
)
CLAIM_ALLOWED_KEYS = {"state", "summary", "evidence_ids", "blocked_by", "notes"}
EVIDENCE_REQUIRED_KEYS = {
    "evidence_id",
    "kind",
    "title",
    "captured_at",
    "sha256",
    "excerpt",
}
EVIDENCE_ALLOWED_KEYS = EVIDENCE_REQUIRED_KEYS | {"url"}
PROBE_REQUEST_KEYS = {"probe_kind", "binary", "required_for_gate"}
ALLOWED_CLAIM_STATES = {"verified", "blocked", "inferred", "unknown"}
ALLOWED_EVIDENCE_KINDS = {"official_doc", "github", "package_registry", "ancillary", "probe_output"}
ALLOWED_PROBE_KINDS = {"help", "version"}
ALLOWED_CANDIDATE_STATUSES = {"eligible", "candidate_rejected", "candidate_error"}
ALLOWED_RUN_STATUSES = {
    "success",
    "success_with_candidate_errors",
    "insufficient_eligible_candidates",
    "run_fatal",
}
ALLOWED_GATE_RESULTS = {"pass", "fail", "blocked", "unknown"}
ALLOWED_PROBE_RESULTS = {"passed", "failed", "skipped"}
PROBE_ARGS = {"help": "--help", "version": "--version"}
MAX_EVIDENCE_REFS = 12
MAX_OFFICIAL_DOC_REFS = 4
MAX_PACKAGE_REGISTRY_REFS = 2
MAX_GITHUB_REFS = 3
MAX_ANCILLARY_REFS = 3
MAX_BLOCKED_STEPS = 3
MAX_FREEFORM_NOTE_CHARS = 1200
MAX_PROBES_PER_CANDIDATE = 2
PROBE_TIMEOUT_SECONDS = 5
MAX_PROBE_OUTPUT_BYTES = 32768
RUN_STATUS_METRIC_KEYS = (
    "maintainer_time_to_decision_seconds",
    "shortlist_override",
    "predicted_blocker_count",
    "later_discovered_blocker_count",
    "rejected_before_scoring_count",
    "evidence_collection_time_seconds",
    "fetched_source_count",
)
APPROVAL_DEPENDENT_METRIC_KEYS = {
    "maintainer_time_to_decision_seconds",
    "shortlist_override",
    "predicted_blocker_count",
    "later_discovered_blocker_count",
}
RUN_ARTIFACT_FILES = (
    "run-status.json",
    "seed.snapshot.toml",
    "candidate-pool.json",
    "eligible-candidates.json",
    "scorecard.json",
    "sources.lock.json",
    "comparison.generated.md",
    "approval-draft.generated.toml",
    "run-summary.md",
)
RUN_ARTIFACT_DIRS = ("candidate-dossiers", "candidate-validation-results")
PACKET_TOPMATTER_PREFIX = (
    "<!-- generated-by: scripts/recommend_next_agent.py generate -->",
    "# Packet - CLI Agent Selection Packet",
    "",
    "Status: Generated",
)
PACKET_RELATED_DOCS = (
    "- `docs/specs/cli-agent-recommendation-dossier-contract.md`",
    "- `docs/specs/cli-agent-onboarding-charter.md`",
    "- `docs/specs/unified-agent-api/support-matrix.md`",
    "- `docs/specs/**` for any normative contract this packet cites",
)
PACKET_SECTION_HEADINGS = (
    "## 1. Candidate Summary",
    "## 2. What Already Exists",
    "## 3. Selection Rubric",
    "## 4. Fixed 3-Candidate Comparison Table",
    "## 5. Recommendation",
    "## 6. Recommended Agent Evaluation Recipe",
    "## 7. Repo-Fit Analysis",
    "## 8. Required Artifacts",
    "## 9. Workstreams, Deliverables, Risks, And Gates",
    "## 10. Dated Evidence Appendix",
)
PACKET_TABLE_HEADER = "| Candidate | Adoption & community pull | CLI product maturity & release activity | Installability & docs quality | Reproducibility & access friction | Architecture fit for this repo | Capability expansion / future leverage | Notes |"
PACKET_TABLE_DIVIDER = "|---|---:|---:|---:|---:|---:|---:|---|"
SECTION7_LABELS = (
    "Manifest root expectations",
    "Wrapper crate expectations",
    "`agent_api` backend expectations",
    "UAA promotion expectations",
    "Support/publication expectations",
    "Likely seam risks",
)
SECTION8_LABELS = (
    "Manifest-root artifacts",
    "Wrapper-crate artifacts",
    "`agent_api` artifacts",
    "UAA promotion-gate artifacts",
    "Docs/spec artifacts",
    "Evidence/fixture artifacts",
)
SECTION9_LABELS = (
    "Required workstreams",
    "Required deliverables",
    "Blocking risks",
    "Acceptance gates",
)
SECTION6_REPRO_NOW_LABELS = (
    "install paths",
    "auth / account / billing prerequisites",
    "runnable commands",
    "evidence gatherable without paid or elevated access",
    "expected artifacts to save during evaluation",
)


class RecommendationError(Exception):
    pass


class NoAbbrevArgumentParser(argparse.ArgumentParser):
    def __init__(self, *args: Any, **kwargs: Any) -> None:
        kwargs.setdefault("allow_abbrev", False)
        super().__init__(*args, **kwargs)


@dataclass(frozen=True)
class DescriptorDefaults:
    canonical_targets: list[str]
    wrapper_coverage_binding_kind: str
    always_on_capabilities: list[str]
    target_gated_capabilities: list[str]
    config_gated_capabilities: list[str]
    backend_extensions: list[str]
    support_matrix_enabled: bool
    capability_matrix_enabled: bool
    capability_matrix_target: str
    docs_release_track: str


@dataclass(frozen=True)
class CandidateSeed:
    agent_id: str
    display_name: str
    research_urls: list[str]
    install_channels: list[str]
    auth_notes: str
    overrides: dict[str, Any]

    def derived_descriptor(self, defaults: DescriptorDefaults, *, agent_id: str | None = None) -> dict[str, Any]:
        actual_agent_id = agent_id or self.agent_id
        crate_path = self.overrides.get("crate_path", f"crates/{actual_agent_id}")
        descriptor: dict[str, Any] = {
            "agent_id": actual_agent_id,
            "display_name": self.display_name,
            "crate_path": crate_path,
            "backend_module": self.overrides.get(
                "backend_module",
                f"crates/agent_api/src/backends/{actual_agent_id}",
            ),
            "manifest_root": self.overrides.get("manifest_root", f"cli_manifests/{actual_agent_id}"),
            "package_name": self.overrides.get(
                "package_name",
                f"unified-agent-api-{actual_agent_id.replace('_', '-')}",
            ),
            "canonical_targets": self.overrides.get("canonical_targets", defaults.canonical_targets),
            "wrapper_coverage_binding_kind": self.overrides.get(
                "wrapper_coverage_binding_kind",
                defaults.wrapper_coverage_binding_kind,
            ),
            "wrapper_coverage_source_path": self.overrides.get("wrapper_coverage_source_path", crate_path),
            "always_on_capabilities": self.overrides.get(
                "always_on_capabilities",
                defaults.always_on_capabilities,
            ),
            "target_gated_capabilities": self.overrides.get(
                "target_gated_capabilities",
                defaults.target_gated_capabilities,
            ),
            "config_gated_capabilities": self.overrides.get(
                "config_gated_capabilities",
                defaults.config_gated_capabilities,
            ),
            "backend_extensions": self.overrides.get("backend_extensions", defaults.backend_extensions),
            "support_matrix_enabled": self.overrides.get(
                "support_matrix_enabled",
                defaults.support_matrix_enabled,
            ),
            "capability_matrix_enabled": self.overrides.get(
                "capability_matrix_enabled",
                defaults.capability_matrix_enabled,
            ),
            "docs_release_track": self.overrides.get("docs_release_track", defaults.docs_release_track),
        }
        capability_matrix_target = self.overrides.get("capability_matrix_target")
        if capability_matrix_target is None:
            capability_matrix_target = defaults.capability_matrix_target
        if capability_matrix_target:
            descriptor["capability_matrix_target"] = capability_matrix_target
        return descriptor


@dataclass(frozen=True)
class SeedConfig:
    defaults: DescriptorDefaults
    candidates: list[CandidateSeed]

    def candidate_by_id(self, agent_id: str) -> CandidateSeed:
        for candidate in self.candidates:
            if candidate.agent_id == agent_id:
                return candidate
        raise RecommendationError(f"unknown candidate `{agent_id}`")


@dataclass(frozen=True)
class CandidateScore:
    scores: dict[str, int]
    notes: str

    @property
    def primary_sum(self) -> int:
        return sum(self.scores[dimension] for dimension in PRIMARY_DIMENSIONS)

    @property
    def secondary_sum(self) -> int:
        return sum(self.scores[dimension] for dimension in SECONDARY_DIMENSIONS)


@dataclass
class CandidateResult:
    agent_id: str
    status: str
    schema_valid: bool
    hard_gate_results: dict[str, dict[str, Any]]
    probe_results: list[dict[str, Any]]
    rejection_reasons: list[str]
    error_reasons: list[str]
    evidence_ids_used: list[str]
    notes: list[str]
    score: CandidateScore | None = None


@dataclass(frozen=True)
class HardGateRule:
    gate_key: str
    rule_id: str
    passing_states: frozenset[str]
    required_all_of_kinds: frozenset[str] = frozenset()
    required_any_of_kinds: frozenset[str] = frozenset()
    allow_probe_output: bool = False
    reject_on_blocked_by: bool = False
    require_required_probe_pass_if_present: bool = False


@dataclass(frozen=True)
class DecisionSurface:
    winner_agent_id: str
    winner_display_name: str
    winner_rationale: str
    loser_rationales: dict[str, list[str]]
    section6_reproducible_now: dict[str, list[str]]
    section6_blocked_until_later: list[str]
    section7: dict[str, list[str]]
    section8: dict[str, list[str]]
    section9: dict[str, list[str]]


def build_parser() -> argparse.ArgumentParser:
    parser = NoAbbrevArgumentParser(description="Generate and promote the next CLI agent recommendation lane.")
    subparsers = parser.add_subparsers(dest="command", required=True, parser_class=NoAbbrevArgumentParser)

    generate = subparsers.add_parser("generate")
    generate.add_argument("--seed-file", required=True)
    generate.add_argument(
        "--research-dir",
        required=True,
        help=f"repo-local frozen research root, e.g. {RECOMMENDATION_RESEARCH_ROOT_REL}/<run-id>",
    )
    generate.add_argument("--run-id", required=True)
    generate.add_argument(
        "--scratch-root",
        required=True,
        help=f"repo-local scratch run root, e.g. {RECOMMENDATION_RUNS_ROOT_REL}",
    )

    promote = subparsers.add_parser("promote")
    promote.add_argument(
        "--run-dir",
        required=True,
        help=f"repo-local scratch run directory, e.g. {RECOMMENDATION_RUNS_ROOT_REL}/<run-id>",
    )
    promote.add_argument("--repo-run-root", required=True)
    promote.add_argument("--approved-agent-id", required=True)
    promote.add_argument("--onboarding-pack-prefix", required=True)
    promote.add_argument("--override-reason")
    return parser


def utc_now() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def json_dumps(data: Any) -> str:
    return json.dumps(data, indent=2, sort_keys=True, ensure_ascii=True) + "\n"


def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def read_json(path: Path) -> Any:
    return json.loads(read_text(path))


@lru_cache(maxsize=1)
def packet_template_provenance_lines() -> dict[str, str]:
    lines = read_text(REPO_ROOT / PACKET_TEMPLATE_REL).splitlines()
    provenance_lines: dict[str, str] = {}
    for heading in PACKET_SECTION_HEADINGS:
        try:
            heading_index = lines.index(heading)
        except ValueError as exc:
            raise RecommendationError(f"packet template is missing required heading `{heading}`") from exc
        provenance_line = None
        for line in lines[heading_index + 1 :]:
            if line.startswith("## "):
                break
            if line.startswith("Provenance: "):
                provenance_line = line
                break
        if provenance_line is None:
            raise RecommendationError(f"packet template is missing required provenance line for `{heading}`")
        provenance_lines[heading] = provenance_line
    return provenance_lines


def packet_template_provenance_line(heading: str) -> str:
    try:
        return packet_template_provenance_lines()[heading]
    except KeyError as exc:
        raise RecommendationError(f"unknown packet heading `{heading}`") from exc


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def write_json(path: Path, data: Any) -> None:
    write_text(path, json_dumps(data))


def write_bytes(path: Path, contents: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(contents)


def remove_path(path: Path) -> None:
    if not path.exists():
        return
    if path.is_dir():
        shutil.rmtree(path)
    else:
        path.unlink()


def canonical_packet_path(repo_root: Path) -> Path:
    return repo_root / CANONICAL_PACKET_REL


def restore_file(path: Path, previous_bytes: bytes | None, *, repo_root: Path) -> None:
    if previous_bytes is None:
        remove_path(path)
        parent = path.parent
        while parent != repo_root and parent.exists() and not any(parent.iterdir()):
            parent.rmdir()
            parent = parent.parent
        return
    write_bytes(path, previous_bytes)


def sha256_bytes(contents: bytes) -> str:
    return hashlib.sha256(contents).hexdigest()


def sha256_file(path: Path) -> str:
    return sha256_bytes(path.read_bytes())


def parse_timestamp(value: str) -> datetime:
    if not isinstance(value, str) or not TIMESTAMP_PATTERN.match(value):
        raise RecommendationError(f"timestamp `{value}` must be UTC RFC3339 with trailing Z")
    return datetime.fromisoformat(value.replace("Z", "+00:00"))


def ensure_string(value: Any, *, label: str) -> str:
    if not isinstance(value, str) or not value:
        raise RecommendationError(f"{label} must be a non-empty string")
    return value


def ensure_optional_string(value: Any, *, label: str) -> str | None:
    if value is None:
        return None
    if not isinstance(value, str):
        raise RecommendationError(f"{label} must be a string or null")
    return value


def ensure_string_list(value: Any, *, label: str) -> list[str]:
    if not isinstance(value, list):
        raise RecommendationError(f"{label} must be an array")
    result: list[str] = []
    for index, entry in enumerate(value):
        if not isinstance(entry, str):
            raise RecommendationError(f"{label}[{index}] must be a string")
        result.append(entry)
    return result


def ensure_bool(value: Any, *, label: str) -> bool:
    if not isinstance(value, bool):
        raise RecommendationError(f"{label} must be a boolean")
    return value


def ensure_int(value: Any, *, label: str) -> int:
    if not isinstance(value, int):
        raise RecommendationError(f"{label} must be an integer")
    return value


def ensure_hex64(value: Any, *, label: str) -> str:
    if not isinstance(value, str) or not HEX64_PATTERN.match(value):
        raise RecommendationError(f"{label} must be a lowercase 64-char SHA-256 hex string")
    return value


def limit_note(value: str, *, label: str) -> str:
    if len(value) > MAX_FREEFORM_NOTE_CHARS:
        raise RecommendationError(f"{label} exceeds the {MAX_FREEFORM_NOTE_CHARS}-char limit")
    return value


def load_onboarded_agent_ids(registry_path: Path) -> set[str]:
    try:
        data = tomllib.loads(read_text(registry_path))
    except FileNotFoundError as exc:
        raise RecommendationError(f"registry file `{registry_path}` does not exist") from exc
    except tomllib.TOMLDecodeError as exc:
        raise RecommendationError(f"registry file `{registry_path}` is not valid TOML: {exc}") from exc
    agents = data.get("agents")
    if not isinstance(agents, list):
        raise RecommendationError(f"registry file `{registry_path}` must define an `agents` array")
    onboarded_ids: set[str] = set()
    for index, agent in enumerate(agents):
        if not isinstance(agent, dict):
            raise RecommendationError(
                f"registry file `{registry_path}` has a non-table agent entry at index {index}"
            )
        agent_id = agent.get("agent_id")
        if not isinstance(agent_id, str) or not agent_id:
            raise RecommendationError(
                f"registry file `{registry_path}` has an invalid `agent_id` at index {index}"
            )
        onboarded_ids.add(agent_id)
    return onboarded_ids


def parse_seed_file(seed_path: Path) -> SeedConfig:
    try:
        data = tomllib.loads(read_text(seed_path))
    except FileNotFoundError as exc:
        raise RecommendationError(f"seed file `{seed_path}` does not exist") from exc
    except tomllib.TOMLDecodeError as exc:
        raise RecommendationError(f"seed file `{seed_path}` is not valid TOML: {exc}") from exc
    if set(data.keys()) != {"defaults", "candidate"}:
        raise RecommendationError("seed file must contain exactly `defaults` and `candidate` top-level tables")
    defaults_root = data["defaults"]
    if set(defaults_root.keys()) != {"descriptor"}:
        raise RecommendationError("seed file `[defaults]` must contain exactly the `descriptor` table")
    defaults_data = defaults_root["descriptor"]
    if set(defaults_data.keys()) != REQUIRED_DEFAULT_KEYS:
        raise RecommendationError("seed file `[defaults.descriptor]` keys do not match the frozen contract")
    if defaults_data["canonical_targets"] != [DEFAULT_TARGET]:
        raise RecommendationError("seed file `canonical_targets` must be `[\"darwin-arm64\"]` in v1")
    if defaults_data["capability_matrix_target"] != "":
        raise RecommendationError("seed file `capability_matrix_target` must be the empty string by default")
    defaults = DescriptorDefaults(
        canonical_targets=list(defaults_data["canonical_targets"]),
        wrapper_coverage_binding_kind=defaults_data["wrapper_coverage_binding_kind"],
        always_on_capabilities=list(defaults_data["always_on_capabilities"]),
        target_gated_capabilities=list(defaults_data["target_gated_capabilities"]),
        config_gated_capabilities=list(defaults_data["config_gated_capabilities"]),
        backend_extensions=list(defaults_data["backend_extensions"]),
        support_matrix_enabled=bool(defaults_data["support_matrix_enabled"]),
        capability_matrix_enabled=bool(defaults_data["capability_matrix_enabled"]),
        capability_matrix_target=defaults_data["capability_matrix_target"],
        docs_release_track=defaults_data["docs_release_track"],
    )
    candidates: list[CandidateSeed] = []
    for agent_id, candidate_data in data["candidate"].items():
        keys = set(candidate_data.keys())
        missing = REQUIRED_CANDIDATE_KEYS - keys
        if missing:
            raise RecommendationError(
                f"candidate `{agent_id}` is missing required keys: {', '.join(sorted(missing))}"
            )
        unknown = keys - (REQUIRED_CANDIDATE_KEYS | OPTIONAL_CANDIDATE_KEYS)
        if unknown:
            raise RecommendationError(
                f"candidate `{agent_id}` has unsupported keys: {', '.join(sorted(unknown))}"
            )
        candidates.append(
            CandidateSeed(
                agent_id=agent_id,
                display_name=ensure_string(candidate_data["display_name"], label=f"candidate `{agent_id}` display_name"),
                research_urls=list(candidate_data["research_urls"]),
                install_channels=list(candidate_data["install_channels"]),
                auth_notes=ensure_string(candidate_data["auth_notes"], label=f"candidate `{agent_id}` auth_notes"),
                overrides={key: candidate_data[key] for key in OPTIONAL_CANDIDATE_KEYS if key in candidate_data},
            )
        )
    if len(candidates) < 3:
        raise RecommendationError("seed file must define at least 3 candidates")
    return SeedConfig(defaults=defaults, candidates=candidates)


def build_empty_metrics() -> dict[str, Any]:
    return {key: None for key in RUN_STATUS_METRIC_KEYS}


def build_base_run_status(
    *,
    run_id: str,
    generated_at: str,
    research_dir: Path,
    run_dir: Path,
) -> dict[str, Any]:
    return {
        "run_id": run_id,
        "status": "run_fatal",
        "generated_at": generated_at,
        "research_dir": str(research_dir),
        "run_dir": str(run_dir),
        "eligible_candidate_ids": [],
        "shortlist_ids": [],
        "recommended_agent_id": None,
        "candidate_status_counts": {
            "eligible": 0,
            "candidate_rejected": 0,
            "candidate_error": 0,
        },
        "metrics": build_empty_metrics(),
        "errors": [],
        "approved_agent_id": None,
        "approval_recorded_at": None,
        "override_reason": None,
        "committed_review_dir": None,
        "committed_packet_path": None,
        "committed_approval_artifact_path": None,
    }


def candidate_status_counts(candidate_results: dict[str, CandidateResult]) -> dict[str, int]:
    counts = {"eligible": 0, "candidate_rejected": 0, "candidate_error": 0}
    for result in candidate_results.values():
        counts[result.status] += 1
    return counts


def seeded_candidate_ids(seed: SeedConfig) -> list[str]:
    return [candidate.agent_id for candidate in seed.candidates]


def expected_run_artifact_files(seed: SeedConfig) -> list[str]:
    expected = list(RUN_ARTIFACT_FILES)
    expected.extend(f"candidate-dossiers/{agent_id}.json" for agent_id in seeded_candidate_ids(seed))
    expected.extend(f"candidate-validation-results/{agent_id}.json" for agent_id in seeded_candidate_ids(seed))
    return sorted(expected)


def run_artifact_files(root: Path) -> list[str]:
    return sorted(path.relative_to(root).as_posix() for path in root.rglob("*") if path.is_file())


def ensure_run_artifact_set(run_dir: Path, seed: SeedConfig) -> None:
    for dirname in RUN_ARTIFACT_DIRS:
        if not (run_dir / dirname).is_dir():
            raise RecommendationError(f"required run artifact directory `{dirname}` is missing")
    actual = run_artifact_files(run_dir)
    expected = expected_run_artifact_files(seed)
    if actual != expected:
        raise RecommendationError("run artifact set does not match the frozen contract")


def validate_run_status_payload(
    *,
    payload: dict[str, Any],
    expected_run_id: str,
    expected_research_dir: Path,
    expected_run_dir: Path,
    promoted: bool,
) -> None:
    required_keys = {
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
    }
    if set(payload.keys()) != required_keys:
        raise RecommendationError("run-status.json keys do not match the frozen contract")
    if payload["run_id"] != expected_run_id:
        raise RecommendationError("run-status.json run_id does not match the expected run id")
    if payload["research_dir"] != str(expected_research_dir):
        raise RecommendationError("run-status.json research_dir does not match the expected research dir")
    if payload["run_dir"] != str(expected_run_dir):
        raise RecommendationError("run-status.json run_dir does not match the expected run dir")
    if payload["status"] not in ALLOWED_RUN_STATUSES:
        raise RecommendationError("run-status.json has an invalid status")
    parse_timestamp(ensure_string(payload["generated_at"], label="run-status.json generated_at"))
    ensure_string_list(payload["eligible_candidate_ids"], label="run-status.json eligible_candidate_ids")
    ensure_string_list(payload["shortlist_ids"], label="run-status.json shortlist_ids")
    if payload["recommended_agent_id"] is not None:
        ensure_string(payload["recommended_agent_id"], label="run-status.json recommended_agent_id")
    if set(payload["candidate_status_counts"].keys()) != set(ALLOWED_CANDIDATE_STATUSES):
        raise RecommendationError("run-status.json candidate_status_counts keys do not match the frozen contract")
    for key, value in payload["candidate_status_counts"].items():
        ensure_int(value, label=f"run-status.json candidate_status_counts.{key}")
    if set(payload["metrics"].keys()) != set(RUN_STATUS_METRIC_KEYS):
        raise RecommendationError("run-status.json metrics keys do not match the frozen contract")
    for key in RUN_STATUS_METRIC_KEYS:
        value = payload["metrics"][key]
        if key in APPROVAL_DEPENDENT_METRIC_KEYS:
            if promoted:
                if key != "later_discovered_blocker_count" and value is None:
                    raise RecommendationError(f"promoted run-status.json metric `{key}` must be finalized")
            elif value is not None:
                raise RecommendationError(f"scratch run-status.json metric `{key}` must be null")
        elif value is None:
            raise RecommendationError(f"run-status.json metric `{key}` must be concrete in scratch output")
    if not isinstance(payload["errors"], list):
        raise RecommendationError("run-status.json errors must be an array")
    for index, entry in enumerate(payload["errors"]):
        if not isinstance(entry, dict):
            raise RecommendationError(f"run-status.json errors[{index}] must be an object")
        if set(entry.keys()) != {"scope", "agent_id", "code", "message"}:
            raise RecommendationError(f"run-status.json errors[{index}] keys do not match the frozen contract")
        if entry["scope"] not in {"run", "candidate"}:
            raise RecommendationError(f"run-status.json errors[{index}] has an invalid scope")
        ensure_optional_string(entry["agent_id"], label=f"run-status.json errors[{index}] agent_id")
        ensure_string(entry["code"], label=f"run-status.json errors[{index}] code")
        ensure_string(entry["message"], label=f"run-status.json errors[{index}] message")
    bookkeeping_keys = (
        "approved_agent_id",
        "approval_recorded_at",
        "override_reason",
        "committed_review_dir",
        "committed_packet_path",
        "committed_approval_artifact_path",
    )
    if promoted:
        ensure_string(payload["approved_agent_id"], label="run-status.json approved_agent_id")
        parse_timestamp(ensure_string(payload["approval_recorded_at"], label="run-status.json approval_recorded_at"))
        ensure_optional_string(payload["override_reason"], label="run-status.json override_reason")
        ensure_string(payload["committed_review_dir"], label="run-status.json committed_review_dir")
        ensure_string(payload["committed_packet_path"], label="run-status.json committed_packet_path")
        ensure_string(
            payload["committed_approval_artifact_path"],
            label="run-status.json committed_approval_artifact_path",
        )
    else:
        for key in bookkeeping_keys:
            if payload[key] is not None:
                raise RecommendationError(f"scratch run-status.json field `{key}` must be null")


def validate_candidate_validation_payload(payload: dict[str, Any], *, expected_agent_id: str) -> None:
    required_keys = {
        "agent_id",
        "status",
        "schema_valid",
        "hard_gate_results",
        "probe_results",
        "rejection_reasons",
        "error_reasons",
        "evidence_ids_used",
        "notes",
    }
    if set(payload.keys()) != required_keys:
        raise RecommendationError(f"candidate validation payload `{expected_agent_id}` keys do not match the frozen contract")
    if payload["agent_id"] != expected_agent_id:
        raise RecommendationError(f"candidate validation payload `{expected_agent_id}` has the wrong agent_id")
    if payload["status"] not in ALLOWED_CANDIDATE_STATUSES:
        raise RecommendationError(f"candidate validation payload `{expected_agent_id}` has an invalid status")
    ensure_bool(payload["schema_valid"], label=f"candidate validation payload `{expected_agent_id}` schema_valid")
    if set(payload["hard_gate_results"].keys()) != set(HARD_GATE_KEYS):
        raise RecommendationError(f"candidate validation payload `{expected_agent_id}` hard_gate_results keys do not match the frozen contract")
    for gate_key, gate in payload["hard_gate_results"].items():
        if set(gate.keys()) != {"status", "rule_id", "rejection_reason", "evidence_ids", "notes"}:
            raise RecommendationError(f"candidate validation payload `{expected_agent_id}` gate `{gate_key}` keys do not match the frozen contract")
        if gate["status"] not in ALLOWED_GATE_RESULTS:
            raise RecommendationError(f"candidate validation payload `{expected_agent_id}` gate `{gate_key}` has an invalid status")
        ensure_string(gate["rule_id"], label=f"candidate validation payload `{expected_agent_id}` gate `{gate_key}` rule_id")
        if gate["rule_id"] != HARD_GATE_RULES[gate_key].rule_id:
            raise RecommendationError(f"candidate validation payload `{expected_agent_id}` gate `{gate_key}` has the wrong rule_id")
        if not isinstance(gate["rejection_reason"], str):
            raise RecommendationError(f"candidate validation payload `{expected_agent_id}` gate `{gate_key}` rejection_reason must be a string")
        ensure_string_list(gate["evidence_ids"], label=f"candidate validation payload `{expected_agent_id}` gate `{gate_key}` evidence_ids")
        if not isinstance(gate["notes"], str):
            raise RecommendationError(f"candidate validation payload `{expected_agent_id}` gate `{gate_key}` notes must be a string")
    if not isinstance(payload["probe_results"], list):
        raise RecommendationError(f"candidate validation payload `{expected_agent_id}` probe_results must be an array")
    for index, probe in enumerate(payload["probe_results"]):
        if set(probe.keys()) != {"probe_kind", "binary", "required_for_gate", "status", "exit_code", "timed_out", "captured_output_ref", "notes"}:
            raise RecommendationError(f"candidate validation payload `{expected_agent_id}` probe_results[{index}] keys do not match the frozen contract")
        if probe["status"] not in ALLOWED_PROBE_RESULTS:
            raise RecommendationError(f"candidate validation payload `{expected_agent_id}` probe_results[{index}] has an invalid status")
    ensure_string_list(payload["rejection_reasons"], label=f"candidate validation payload `{expected_agent_id}` rejection_reasons")
    ensure_string_list(payload["error_reasons"], label=f"candidate validation payload `{expected_agent_id}` error_reasons")
    ensure_string_list(payload["evidence_ids_used"], label=f"candidate validation payload `{expected_agent_id}` evidence_ids_used")
    ensure_string_list(payload["notes"], label=f"candidate validation payload `{expected_agent_id}` notes")


def packet_section_slice(packet: str, heading: str, next_heading: str | None) -> str:
    start = packet.index(heading)
    end = len(packet) if next_heading is None else packet.index(next_heading, start)
    return packet[start:end]


def validate_packet_contract(
    *,
    packet: str,
    shortlist_ids: list[str],
    seeded_ids: list[str],
    candidate_results: dict[str, CandidateResult],
) -> None:
    lines = packet.splitlines()
    if tuple(lines[:4]) != PACKET_TOPMATTER_PREFIX:
        raise RecommendationError("comparison packet title block shape does not match the frozen template")
    if len(lines) < 11 or not lines[4].startswith("Date (UTC): ") or lines[5] != "Owner(s): wrappers team / deterministic runner" or lines[6] != "Related source docs:" or lines[7:11] != list(PACKET_RELATED_DOCS):
        raise RecommendationError("comparison packet related-doc shape does not match the frozen template")
    previous_index = -1
    for heading in PACKET_SECTION_HEADINGS:
        try:
            current_index = packet.index(heading)
        except ValueError as exc:
            raise RecommendationError(f"comparison packet is missing required heading `{heading}`") from exc
        if current_index <= previous_index:
            raise RecommendationError("comparison packet section order does not match the frozen template")
        previous_index = current_index
        required_provenance = packet_template_provenance_line(heading)
        if required_provenance not in packet_section_slice(
            packet,
            heading,
            PACKET_SECTION_HEADINGS[PACKET_SECTION_HEADINGS.index(heading) + 1] if heading != PACKET_SECTION_HEADINGS[-1] else None,
        ):
            raise RecommendationError(f"comparison packet is missing required provenance line for `{heading}`")

    section4 = packet_section_slice(packet, PACKET_SECTION_HEADINGS[3], PACKET_SECTION_HEADINGS[4])
    section4_lines = section4.splitlines()
    if PACKET_TABLE_HEADER not in section4_lines or PACKET_TABLE_DIVIDER not in section4_lines:
        raise RecommendationError("comparison packet section 4 table shape does not match the frozen template")
    candidate_rows = [line for line in section4_lines if line.startswith("| `")]
    if len(candidate_rows) != 3:
        raise RecommendationError("comparison packet section 4 must contain exactly 3 candidate rows")
    for agent_id in shortlist_ids:
        matches = [line for line in candidate_rows if line.startswith(f"| `{agent_id}` |")]
        if len(matches) != 1 or "refs=" not in matches[0]:
            raise RecommendationError(f"comparison packet section 4 row for `{agent_id}` must cite dossier evidence or probe refs")

    section5 = packet_section_slice(packet, PACKET_SECTION_HEADINGS[4], PACKET_SECTION_HEADINGS[5])
    if "refs=" not in section5 and not re.search(r"`[a-z0-9_]+:(?:help|version):\d+`", section5):
        raise RecommendationError("comparison packet section 5 rationale must cite dossier evidence ids or probe refs")
    tail = [line for line in section5.splitlines() if line.strip()]
    if tail[-3:] != [
        "Approve recommended agent",
        "Override to shortlisted alternative",
        "Stop and expand research",
    ]:
        raise RecommendationError("comparison packet section 5 must end with the three decision options")

    section6 = packet_section_slice(packet, PACKET_SECTION_HEADINGS[5], PACKET_SECTION_HEADINGS[6])
    if "reproducible now:" not in section6 or "blocked until later:" not in section6:
        raise RecommendationError("comparison packet section 6 must be split into reproducible now and blocked until later")
    section7 = packet_section_slice(packet, PACKET_SECTION_HEADINGS[6], PACKET_SECTION_HEADINGS[7])
    exact_nonempty_labeled_section(section7, SECTION7_LABELS, section_heading="comparison packet section 7")
    section8 = packet_section_slice(packet, PACKET_SECTION_HEADINGS[7], PACKET_SECTION_HEADINGS[8])
    exact_nonempty_labeled_section(section8, SECTION8_LABELS, section_heading="comparison packet section 8")
    section9 = packet_section_slice(packet, PACKET_SECTION_HEADINGS[8], PACKET_SECTION_HEADINGS[9])
    exact_nonempty_labeled_section(section9, SECTION9_LABELS, section_heading="comparison packet section 9")

    appendix = packet_section_slice(packet, PACKET_SECTION_HEADINGS[9], None)
    for agent_id in shortlist_ids:
        if f"### `{agent_id}`" not in appendix or "- Loser rationale:" not in appendix:
            raise RecommendationError("comparison packet appendix must include loser rationale for each shortlisted candidate")
    if "captured `" not in appendix:
        raise RecommendationError("comparison packet appendix must include dated evidence provenance")
    strategic_contenders = [
        agent_id for agent_id in seeded_ids if agent_id not in shortlist_ids and candidate_results[agent_id].status != "eligible"
    ]
    if strategic_contenders and "### Strategic Contenders" not in appendix:
        raise RecommendationError("comparison packet appendix must include strategic contenders when they exist")


def validate_run_summary_delta(scratch_summary: str, promoted_summary: str) -> None:
    allowed_prefixes = tuple(f"  - {key}:" for key in APPROVAL_DEPENDENT_METRIC_KEYS)

    def filtered_lines(text: str) -> list[str]:
        filtered: list[str] = []
        skip_override = False
        for line in text.splitlines():
            if line.startswith("- approved_agent_id:"):
                continue
            if line.startswith(allowed_prefixes):
                continue
            if line == "- override_summary:":
                skip_override = True
                continue
            if skip_override:
                if line.startswith("  - "):
                    continue
                skip_override = False
            filtered.append(line)
        return filtered

    if filtered_lines(scratch_summary) != filtered_lines(promoted_summary):
        raise RecommendationError("run-summary.md includes promote-time deltas outside the legal summary classes")


def validate_run_status_delta(scratch_status: dict[str, Any], promoted_status: dict[str, Any]) -> None:
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
        if scratch_status[key] != promoted_status[key]:
            raise RecommendationError(f"run-status.json field `{key}` changed outside the legal promote delta classes")
    for key in (
        "rejected_before_scoring_count",
        "evidence_collection_time_seconds",
        "fetched_source_count",
    ):
        if scratch_status["metrics"][key] != promoted_status["metrics"][key]:
            raise RecommendationError(f"run-status.json metric `{key}` changed outside the legal promote delta classes")


def validate_scratch_outputs(
    *,
    run_dir: Path,
    run_status: dict[str, Any],
    seed: SeedConfig,
    research_dir: Path,
    candidate_pool: dict[str, Any],
    eligible_candidates: dict[str, Any],
    scorecard: dict[str, Any],
    sources_lock: dict[str, Any],
    candidate_results: dict[str, CandidateResult],
) -> None:
    ensure_run_artifact_set(run_dir, seed)
    actual_status = read_json(run_dir / "run-status.json")
    validate_run_status_payload(
        payload=actual_status,
        expected_run_id=run_status["run_id"],
        expected_research_dir=research_dir,
        expected_run_dir=run_dir,
        promoted=False,
    )
    if actual_status != run_status:
        raise RecommendationError("scratch run-status.json does not match the deterministic output")
    if read_json(run_dir / "candidate-pool.json") != candidate_pool:
        raise RecommendationError("candidate-pool.json does not match the deterministic output")
    if read_json(run_dir / "eligible-candidates.json") != eligible_candidates:
        raise RecommendationError("eligible-candidates.json does not match the deterministic output")
    if read_json(run_dir / "scorecard.json") != scorecard:
        raise RecommendationError("scorecard.json does not match the deterministic output")
    if read_json(run_dir / "sources.lock.json") != sources_lock:
        raise RecommendationError("sources.lock.json does not match the deterministic output")
    summary_text = read_text(run_dir / "run-summary.md")
    expected_summary = render_run_summary(
        mode="generate",
        run_id=run_status["run_id"],
        generated_at=run_status["generated_at"],
        recommended_agent_id=run_status["recommended_agent_id"],
        approved_agent_id=None,
        shortlist_ids=run_status["shortlist_ids"],
        metrics=run_status["metrics"],
        override_reason=None,
    )
    if summary_text != expected_summary:
        raise RecommendationError("run-summary.md does not match the deterministic scratch summary")
    packet_text = read_text(run_dir / "comparison.generated.md")
    validate_packet_contract(
        packet=packet_text,
        shortlist_ids=run_status["shortlist_ids"],
        seeded_ids=seeded_candidate_ids(seed),
        candidate_results=candidate_results,
    )
    recommended_agent_id = run_status["recommended_agent_id"] or run_status["shortlist_ids"][0]
    expected_approval_draft = render_approval_toml(
        candidate=seed.candidate_by_id(recommended_agent_id),
        defaults=seed.defaults,
        recommended_agent_id=recommended_agent_id,
        approved_agent_id=recommended_agent_id,
        onboarding_pack_prefix=derived_pack_prefix(recommended_agent_id),
        approval_commit="0000000",
        approval_recorded_at=run_status["generated_at"],
        override_reason=None,
    )
    if read_text(run_dir / "approval-draft.generated.toml") != expected_approval_draft:
        raise RecommendationError("approval-draft.generated.toml does not match the deterministic scratch output")
    for candidate in seed.candidates:
        if (run_dir / "candidate-dossiers" / f"{candidate.agent_id}.json").read_bytes() != dossier_path(research_dir, candidate.agent_id).read_bytes():
            raise RecommendationError(f"candidate dossier `{candidate.agent_id}` is not a byte-copy of the research dossier")
        validation = read_json(run_dir / "candidate-validation-results" / f"{candidate.agent_id}.json")
        validate_candidate_validation_payload(validation, expected_agent_id=candidate.agent_id)
        if validation != serialize_candidate_validation(candidate_results[candidate.agent_id]):
            raise RecommendationError(f"candidate validation result `{candidate.agent_id}` does not match the deterministic output")


def validate_promoted_outputs(
    *,
    scratch_run_dir: Path,
    scratch_status: dict[str, Any],
    final_review_dir: Path,
    final_status: dict[str, Any],
    seed: SeedConfig,
    research_dir: Path,
    canonical_path: Path,
    final_approval_path: Path,
    final_approval_text: str,
) -> None:
    ensure_run_artifact_set(final_review_dir, seed)
    actual_final_status = read_json(final_review_dir / "run-status.json")
    if actual_final_status != final_status:
        raise RecommendationError("promoted run-status.json does not match the deterministic promote output")
    validate_run_status_payload(
        payload=actual_final_status,
        expected_run_id=scratch_status["run_id"],
        expected_research_dir=research_dir,
        expected_run_dir=scratch_run_dir,
        promoted=True,
    )
    validate_run_status_delta(scratch_status, actual_final_status)
    if (final_review_dir / "comparison.generated.md").read_bytes() != (scratch_run_dir / "comparison.generated.md").read_bytes():
        raise RecommendationError("committed comparison packet must be a byte-copy of scratch output")
    if canonical_path.read_bytes() != (scratch_run_dir / "comparison.generated.md").read_bytes():
        raise RecommendationError("canonical packet must be byte-identical to scratch comparison output")
    for artifact in (
        "seed.snapshot.toml",
        "candidate-pool.json",
        "eligible-candidates.json",
        "scorecard.json",
        "sources.lock.json",
        "comparison.generated.md",
        "approval-draft.generated.toml",
    ):
        if (final_review_dir / artifact).read_bytes() != (scratch_run_dir / artifact).read_bytes():
            raise RecommendationError(f"committed review artifact `{artifact}` must be a byte-copy of the scratch run")
    for candidate in seed.candidates:
        rel_paths = (
            f"candidate-dossiers/{candidate.agent_id}.json",
            f"candidate-validation-results/{candidate.agent_id}.json",
        )
        for rel_path in rel_paths:
            if (final_review_dir / rel_path).read_bytes() != (scratch_run_dir / rel_path).read_bytes():
                raise RecommendationError(f"committed review artifact `{rel_path}` must be a byte-copy of the scratch run")
    scratch_summary = read_text(scratch_run_dir / "run-summary.md")
    promoted_summary = read_text(final_review_dir / "run-summary.md")
    validate_run_summary_delta(scratch_summary, promoted_summary)
    expected_summary = render_run_summary(
        mode="promote",
        run_id=actual_final_status["run_id"],
        generated_at=actual_final_status["generated_at"],
        recommended_agent_id=actual_final_status["recommended_agent_id"],
        approved_agent_id=actual_final_status["approved_agent_id"],
        shortlist_ids=actual_final_status["shortlist_ids"],
        metrics=actual_final_status["metrics"],
        override_reason=actual_final_status["override_reason"],
    )
    if promoted_summary != expected_summary:
        raise RecommendationError("promoted run-summary.md does not match the deterministic promote summary")
    promoted_candidate_results = {
        candidate.agent_id: CandidateResult(
            agent_id=candidate.agent_id,
            status=read_json(final_review_dir / "candidate-validation-results" / f"{candidate.agent_id}.json")["status"],
            schema_valid=True,
            hard_gate_results={},
            probe_results=[],
            rejection_reasons=[],
            error_reasons=[],
            evidence_ids_used=[],
            notes=[],
        )
        for candidate in seed.candidates
    }
    validate_packet_contract(
        packet=read_text(final_review_dir / "comparison.generated.md"),
        shortlist_ids=actual_final_status["shortlist_ids"],
        seeded_ids=seeded_candidate_ids(seed),
        candidate_results=promoted_candidate_results,
    )
    if read_text(final_approval_path) != final_approval_text:
        raise RecommendationError("final approval artifact must match the promote-time rendered approval contents")


def build_placeholder_gate_results() -> dict[str, dict[str, Any]]:
    return {
        key: {
            "status": "unknown",
            "rule_id": HARD_GATE_RULES[key].rule_id,
            "rejection_reason": "",
            "evidence_ids": [],
            "notes": "",
        }
        for key in HARD_GATE_KEYS
    }


def build_placeholder_candidate_result(agent_id: str) -> CandidateResult:
    return CandidateResult(
        agent_id=agent_id,
        status="candidate_error",
        schema_valid=False,
        hard_gate_results=build_placeholder_gate_results(),
        probe_results=[],
        rejection_reasons=[],
        error_reasons=[],
        evidence_ids_used=[],
        notes=[],
    )


def build_run_error(*, scope: str, agent_id: str | None, code: str, message: str) -> dict[str, Any]:
    return {
        "scope": scope,
        "agent_id": agent_id,
        "code": code,
        "message": message,
    }


def validate_seed_file_exists(seed_file: Path) -> None:
    if not seed_file.exists():
        raise RecommendationError(f"seed file `{seed_file}` does not exist")


def validate_research_metadata(
    *,
    research_metadata_path: Path,
    expected_run_id: str,
    research_dir: Path,
    run_dir: Path,
) -> dict[str, Any]:
    try:
        data = read_json(research_metadata_path)
    except FileNotFoundError as exc:
        raise RecommendationError(f"research metadata `{research_metadata_path}` does not exist") from exc
    except json.JSONDecodeError as exc:
        raise RecommendationError(f"research metadata `{research_metadata_path}` is not valid JSON: {exc}") from exc
    if not isinstance(data, dict):
        raise RecommendationError("research metadata must be a JSON object")
    if set(data.keys()) != {"run_id", "evidence_collection_time_seconds", "fetched_source_count"}:
        raise RecommendationError("research metadata keys do not match the frozen contract")
    run_id = data["run_id"]
    evidence_collection_time_seconds = data["evidence_collection_time_seconds"]
    fetched_source_count = data["fetched_source_count"]
    if not isinstance(run_id, str):
        raise RecommendationError("research metadata run_id must be a string")
    if not isinstance(evidence_collection_time_seconds, int):
        raise RecommendationError("research metadata evidence_collection_time_seconds must be an integer")
    if not isinstance(fetched_source_count, int):
        raise RecommendationError("research metadata fetched_source_count must be an integer")
    if run_id != expected_run_id:
        raise RecommendationError("research metadata run_id must equal CLI --run-id")
    if research_dir.name != expected_run_id:
        raise RecommendationError("research directory basename must equal CLI --run-id")
    if run_dir.name != expected_run_id:
        raise RecommendationError("run directory basename must equal CLI --run-id")
    if run_id != research_dir.name or run_id != run_dir.name:
        raise RecommendationError("research metadata run_id must match the directory basenames")
    return data


def validate_dossier_top_level(
    dossier: dict[str, Any],
    *,
    agent_id: str,
    snapshot_sha: str,
    seeded_ids: set[str],
) -> None:
    if set(dossier.keys()) != DOSSIER_REQUIRED_KEYS:
        raise RecommendationError(f"dossier `{agent_id}` top-level keys do not match the frozen contract")
    ensure_string(dossier["schema_version"], label=f"dossier `{agent_id}` schema_version")
    actual_agent_id = ensure_string(dossier["agent_id"], label=f"dossier `{agent_id}` agent_id")
    if actual_agent_id != agent_id:
        raise RecommendationError(f"dossier `{agent_id}` agent_id does not match its filename")
    if actual_agent_id not in seeded_ids:
        raise RecommendationError(f"dossier `{agent_id}` does not correspond to a seeded candidate")
    ensure_string(dossier["display_name"], label=f"dossier `{agent_id}` display_name")
    parse_timestamp(ensure_string(dossier["generated_at"], label=f"dossier `{agent_id}` generated_at"))
    if ensure_hex64(dossier["seed_snapshot_sha256"], label=f"dossier `{agent_id}` seed_snapshot_sha256") != snapshot_sha:
        raise RecommendationError(f"dossier `{agent_id}` seed_snapshot_sha256 does not match the run snapshot")
    ensure_string_list(dossier["official_links"], label=f"dossier `{agent_id}` official_links")
    ensure_string_list(dossier["install_channels"], label=f"dossier `{agent_id}` install_channels")
    ensure_string_list(dossier["auth_prerequisites"], label=f"dossier `{agent_id}` auth_prerequisites")
    blocked_steps = ensure_string_list(dossier["blocked_steps"], label=f"dossier `{agent_id}` blocked_steps")
    if len(blocked_steps) > MAX_BLOCKED_STEPS:
        raise RecommendationError(f"dossier `{agent_id}` blocked_steps exceeds the limit")
    for index, entry in enumerate(blocked_steps):
        limit_note(entry, label=f"dossier `{agent_id}` blocked_steps[{index}]")
    normalized_caveats = ensure_string_list(
        dossier["normalized_caveats"],
        label=f"dossier `{agent_id}` normalized_caveats",
    )
    for index, entry in enumerate(normalized_caveats):
        limit_note(entry, label=f"dossier `{agent_id}` normalized_caveats[{index}]")
    validate_claims(dossier["claims"], agent_id=agent_id)
    validate_probe_requests(dossier["probe_requests"], agent_id=agent_id)
    validate_evidence(dossier["evidence"], agent_id=agent_id)


def validate_claims(claims: Any, *, agent_id: str) -> None:
    if not isinstance(claims, dict):
        raise RecommendationError(f"dossier `{agent_id}` claims must be an object")
    if set(claims.keys()) != set(CLAIM_KEYS):
        raise RecommendationError(f"dossier `{agent_id}` claims keys do not match the frozen contract")
    for claim_key in CLAIM_KEYS:
        claim = claims[claim_key]
        if not isinstance(claim, dict):
            raise RecommendationError(f"dossier `{agent_id}` claim `{claim_key}` must be an object")
        if not set(claim.keys()).issubset(CLAIM_ALLOWED_KEYS):
            raise RecommendationError(f"dossier `{agent_id}` claim `{claim_key}` has unsupported keys")
        for required in ("state", "summary", "evidence_ids"):
            if required not in claim:
                raise RecommendationError(f"dossier `{agent_id}` claim `{claim_key}` is missing `{required}`")
        state = ensure_string(claim["state"], label=f"dossier `{agent_id}` claim `{claim_key}` state")
        if state not in ALLOWED_CLAIM_STATES:
            raise RecommendationError(f"dossier `{agent_id}` claim `{claim_key}` has an invalid state")
        limit_note(
            ensure_string(claim["summary"], label=f"dossier `{agent_id}` claim `{claim_key}` summary"),
            label=f"dossier `{agent_id}` claim `{claim_key}` summary",
        )
        ensure_string_list(claim["evidence_ids"], label=f"dossier `{agent_id}` claim `{claim_key}` evidence_ids")
        if "blocked_by" in claim:
            ensure_string_list(claim["blocked_by"], label=f"dossier `{agent_id}` claim `{claim_key}` blocked_by")
        if "notes" in claim:
            limit_note(
                ensure_string(claim["notes"], label=f"dossier `{agent_id}` claim `{claim_key}` notes"),
                label=f"dossier `{agent_id}` claim `{claim_key}` notes",
            )


def validate_probe_requests(probe_requests: Any, *, agent_id: str) -> None:
    if not isinstance(probe_requests, list):
        raise RecommendationError(f"dossier `{agent_id}` probe_requests must be an array")
    if len(probe_requests) > MAX_PROBES_PER_CANDIDATE:
        raise RecommendationError(f"dossier `{agent_id}` probe_requests exceeds the limit")
    for index, entry in enumerate(probe_requests):
        if not isinstance(entry, dict):
            raise RecommendationError(f"dossier `{agent_id}` probe_requests[{index}] must be an object")
        if set(entry.keys()) != PROBE_REQUEST_KEYS:
            raise RecommendationError(f"dossier `{agent_id}` probe_requests[{index}] keys do not match the frozen contract")
        probe_kind = ensure_string(entry["probe_kind"], label=f"dossier `{agent_id}` probe_requests[{index}] probe_kind")
        if probe_kind not in ALLOWED_PROBE_KINDS:
            raise RecommendationError(f"dossier `{agent_id}` probe_requests[{index}] has an invalid probe_kind")
        binary = ensure_string(entry["binary"], label=f"dossier `{agent_id}` probe_requests[{index}] binary")
        if not SAFE_BINARY_PATTERN.match(binary) or "/" in binary:
            raise RecommendationError(f"dossier `{agent_id}` probe_requests[{index}] binary violates the allowlist")
        ensure_bool(entry["required_for_gate"], label=f"dossier `{agent_id}` probe_requests[{index}] required_for_gate")


def validate_evidence(evidence: Any, *, agent_id: str) -> None:
    if not isinstance(evidence, list):
        raise RecommendationError(f"dossier `{agent_id}` evidence must be an array")
    if len(evidence) > MAX_EVIDENCE_REFS:
        raise RecommendationError(f"dossier `{agent_id}` evidence exceeds the limit")
    kind_counts = {kind: 0 for kind in ALLOWED_EVIDENCE_KINDS}
    evidence_ids: set[str] = set()
    for index, entry in enumerate(evidence):
        if not isinstance(entry, dict):
            raise RecommendationError(f"dossier `{agent_id}` evidence[{index}] must be an object")
        if not set(entry.keys()).issubset(EVIDENCE_ALLOWED_KEYS):
            raise RecommendationError(f"dossier `{agent_id}` evidence[{index}] has unsupported keys")
        missing = EVIDENCE_REQUIRED_KEYS - set(entry.keys())
        if missing:
            raise RecommendationError(
                f"dossier `{agent_id}` evidence[{index}] is missing required keys: {', '.join(sorted(missing))}"
            )
        evidence_id = ensure_string(entry["evidence_id"], label=f"dossier `{agent_id}` evidence[{index}] evidence_id")
        if evidence_id in evidence_ids:
            raise RecommendationError(f"dossier `{agent_id}` evidence ids must be unique")
        evidence_ids.add(evidence_id)
        kind = ensure_string(entry["kind"], label=f"dossier `{agent_id}` evidence[{index}] kind")
        if kind not in ALLOWED_EVIDENCE_KINDS:
            raise RecommendationError(f"dossier `{agent_id}` evidence[{index}] has an invalid kind")
        if "url" in entry and entry["url"] is not None:
            ensure_string(entry["url"], label=f"dossier `{agent_id}` evidence[{index}] url")
        ensure_string(entry["title"], label=f"dossier `{agent_id}` evidence[{index}] title")
        parse_timestamp(ensure_string(entry["captured_at"], label=f"dossier `{agent_id}` evidence[{index}] captured_at"))
        ensure_hex64(entry["sha256"], label=f"dossier `{agent_id}` evidence[{index}] sha256")
        limit_note(
            ensure_string(entry["excerpt"], label=f"dossier `{agent_id}` evidence[{index}] excerpt"),
            label=f"dossier `{agent_id}` evidence[{index}] excerpt",
        )
        kind_counts[kind] += 1
    if kind_counts["official_doc"] > MAX_OFFICIAL_DOC_REFS:
        raise RecommendationError(f"dossier `{agent_id}` has too many official_doc refs")
    if kind_counts["package_registry"] > MAX_PACKAGE_REGISTRY_REFS:
        raise RecommendationError(f"dossier `{agent_id}` has too many package_registry refs")
    if kind_counts["github"] > MAX_GITHUB_REFS:
        raise RecommendationError(f"dossier `{agent_id}` has too many github refs")
    if kind_counts["ancillary"] > MAX_ANCILLARY_REFS:
        raise RecommendationError(f"dossier `{agent_id}` has too many ancillary refs")


def validate_claim_evidence_links(dossier: dict[str, Any], *, agent_id: str) -> None:
    evidence_ids = {entry["evidence_id"] for entry in dossier["evidence"]}
    for claim_key, claim in dossier["claims"].items():
        for evidence_id in claim["evidence_ids"]:
            if evidence_id not in evidence_ids:
                raise RecommendationError(f"dossier `{agent_id}` claim `{claim_key}` references unknown evidence_id `{evidence_id}`")


def load_dossier_payload(path: Path) -> dict[str, Any]:
    try:
        data = read_json(path)
    except UnicodeDecodeError as exc:
        raise RecommendationError(f"dossier `{path}` is not valid UTF-8: {exc}") from exc
    except json.JSONDecodeError as exc:
        raise RecommendationError(f"dossier `{path}` is not valid JSON: {exc}") from exc
    if not isinstance(data, dict):
        raise RecommendationError(f"dossier `{path}` must be a JSON object")
    return data


def redacted_text(value: str) -> str:
    value = re.sub(r"(?i)\b(token|api[_-]?key|secret|password)=\S+", r"\1=[REDACTED]", value)
    value = re.sub(r"(?i)\b(bearer)\s+\S+", r"\1 [REDACTED]", value)
    value = re.sub(r"(?:(?:/Users|/home|/tmp|/var|/private|/opt|/Volumes|/Applications|/usr|/etc)[^\s]*)", "[REDACTED_PATH]", value)
    return value


def probe_output_ref(agent_id: str, probe_kind: str, index: int) -> str:
    return f"{agent_id}:{probe_kind}:{index}"


def execute_probe(
    *,
    agent_id: str,
    probe_request: dict[str, Any],
    index: int,
) -> tuple[dict[str, Any], dict[str, Any] | None, str | None]:
    probe_kind = probe_request["probe_kind"]
    binary = probe_request["binary"]
    required_for_gate = probe_request["required_for_gate"]
    result: dict[str, Any] = {
        "probe_kind": probe_kind,
        "binary": binary,
        "required_for_gate": required_for_gate,
        "status": "skipped",
        "exit_code": None,
        "timed_out": False,
        "captured_output_ref": None,
        "notes": "",
    }
    if probe_kind not in ALLOWED_PROBE_KINDS:
        result["status"] = "failed"
        result["notes"] = "invalid probe kind"
        return result, None, "probe kind is not allowlisted"
    if not SAFE_BINARY_PATTERN.match(binary) or "/" in binary:
        result["status"] = "failed"
        result["notes"] = "binary violates allowlist"
        return result, None, "binary violates allowlist"
    env = {key: os.environ[key] for key in ("PATH", "HOME", "TMPDIR") if key in os.environ}
    try:
        completed = subprocess.run(
            [binary, PROBE_ARGS[probe_kind]],
            capture_output=True,
            check=False,
            env=env,
            text=False,
            timeout=PROBE_TIMEOUT_SECONDS,
        )
    except FileNotFoundError:
        result["status"] = "failed"
        result["notes"] = "binary not found"
        return result, None, "binary not found"
    except subprocess.TimeoutExpired as exc:
        result["status"] = "failed"
        result["timed_out"] = True
        captured = (exc.stdout or b"") + (exc.stderr or b"")
        if len(captured) > MAX_PROBE_OUTPUT_BYTES:
            result["notes"] = "timed out and exceeded output byte cap"
        else:
            result["notes"] = "timed out"
        return result, None, result["notes"]
    captured = completed.stdout + completed.stderr
    result["exit_code"] = completed.returncode
    if len(captured) > MAX_PROBE_OUTPUT_BYTES:
        result["status"] = "failed"
        result["notes"] = "captured output exceeded byte cap"
        return result, None, "captured output exceeded byte cap"
    if completed.returncode != 0:
        result["status"] = "failed"
        result["notes"] = f"probe exited with code {completed.returncode}"
        return result, None, result["notes"]
    output_id = probe_output_ref(agent_id, probe_kind, index)
    result["status"] = "passed"
    result["captured_output_ref"] = output_id
    result["notes"] = "probe passed"
    ref = {
        "probe_output_ref": output_id,
        "probe_kind": probe_kind,
        "binary": binary,
        "captured_at": utc_now(),
        "exit_code": completed.returncode,
        "excerpt": redacted_text(captured.decode("utf-8", errors="replace")),
    }
    return result, ref, None


HARD_GATE_RULES = {
    "non_interactive_execution": HardGateRule(
        gate_key="non_interactive_execution",
        rule_id="hard_gate.non_interactive_execution.verified_doc_and_package_or_probe",
        passing_states=frozenset({"verified"}),
        required_all_of_kinds=frozenset({"official_doc"}),
        required_any_of_kinds=frozenset({"package_registry", "probe_output"}),
        allow_probe_output=True,
        require_required_probe_pass_if_present=True,
    ),
    "offline_strategy": HardGateRule(
        gate_key="offline_strategy",
        rule_id="hard_gate.offline_strategy.doc_or_repo_backed",
        passing_states=frozenset({"verified", "inferred"}),
        required_any_of_kinds=frozenset({"official_doc", "github"}),
        reject_on_blocked_by=True,
    ),
    "observable_cli_surface": HardGateRule(
        gate_key="observable_cli_surface",
        rule_id="hard_gate.observable_cli_surface.verified_doc_or_repo_or_probe",
        passing_states=frozenset({"verified"}),
        required_any_of_kinds=frozenset({"official_doc", "github", "probe_output"}),
        allow_probe_output=True,
        require_required_probe_pass_if_present=True,
    ),
    "redaction_fit": HardGateRule(
        gate_key="redaction_fit",
        rule_id="hard_gate.redaction_fit.repo_or_probe_backed",
        passing_states=frozenset({"verified", "inferred"}),
        required_any_of_kinds=frozenset({"github", "probe_output"}),
        allow_probe_output=True,
        reject_on_blocked_by=True,
    ),
    "crate_first_fit": HardGateRule(
        gate_key="crate_first_fit",
        rule_id="hard_gate.crate_first_fit.doc_or_repo_or_package_backed",
        passing_states=frozenset({"verified", "inferred"}),
        required_any_of_kinds=frozenset({"official_doc", "github", "package_registry"}),
        reject_on_blocked_by=True,
    ),
    "reproducibility": HardGateRule(
        gate_key="reproducibility",
        rule_id="hard_gate.reproducibility.doc_and_package",
        passing_states=frozenset({"verified", "inferred"}),
        required_all_of_kinds=frozenset({"official_doc", "package_registry"}),
        reject_on_blocked_by=True,
    ),
}


def evidence_index(dossier: dict[str, Any]) -> dict[str, dict[str, Any]]:
    return {entry["evidence_id"]: entry for entry in dossier["evidence"]}


def evidence_kinds_for_ids(
    evidence_lookup: dict[str, dict[str, Any]],
    evidence_ids: list[str],
) -> set[str]:
    kinds: set[str] = set()
    for evidence_id in evidence_ids:
        if evidence_id in evidence_lookup:
            kinds.add(evidence_lookup[evidence_id]["kind"])
    return kinds


def probe_output_ids_for_gate(
    *,
    rule: HardGateRule,
    probe_results: list[dict[str, Any]],
) -> list[str]:
    if not rule.allow_probe_output:
        return []
    return [
        result["captured_output_ref"]
        for result in probe_results
        if result["status"] == "passed" and result["captured_output_ref"]
    ]


def required_probe_results(probe_results: list[dict[str, Any]]) -> list[dict[str, Any]]:
    return [result for result in probe_results if result["required_for_gate"]]


def effective_evidence_kinds(
    *,
    dossier: dict[str, Any],
    evidence_ids: list[str],
) -> set[str]:
    kinds = evidence_kinds_for_ids(evidence_index(dossier), evidence_ids)
    known_evidence_ids = set(evidence_index(dossier).keys())
    if any(evidence_id not in known_evidence_ids for evidence_id in evidence_ids):
        kinds.add("probe_output")
    return kinds


def missing_required_evidence_message(
    *,
    rule: HardGateRule,
    actual_kinds: set[str],
) -> str | None:
    missing_all = sorted(rule.required_all_of_kinds - actual_kinds)
    if missing_all:
        return "missing required evidence kinds: " + ", ".join(missing_all)
    if rule.required_any_of_kinds and not actual_kinds.intersection(rule.required_any_of_kinds):
        return "missing required evidence kinds: one of " + ", ".join(sorted(rule.required_any_of_kinds))
    return None


def build_gate_result(
    *,
    status: str,
    rule: HardGateRule,
    evidence_ids: list[str],
    notes: str,
    rejection_reason: str = "",
) -> dict[str, Any]:
    return {
        "status": status,
        "rule_id": rule.rule_id,
        "rejection_reason": rejection_reason,
        "evidence_ids": evidence_ids,
        "notes": notes,
    }


def evaluate_hard_gate(
    *,
    dossier: dict[str, Any],
    gate_key: str,
    probe_results: list[dict[str, Any]],
) -> tuple[dict[str, Any], bool, bool]:
    rule = HARD_GATE_RULES[gate_key]
    claim = dossier["claims"][gate_key]
    state = claim["state"]
    notes = claim.get("notes", "")
    blocked_by = list(claim.get("blocked_by", []))
    evidence_ids = list(claim["evidence_ids"])
    evidence_ids.extend(
        probe_id
        for probe_id in probe_output_ids_for_gate(rule=rule, probe_results=probe_results)
        if probe_id not in evidence_ids
    )
    if state == "blocked":
        rejection_reason = "claim state is blocked"
        return build_gate_result(
            status="blocked",
            rule=rule,
            evidence_ids=evidence_ids,
            notes=notes,
            rejection_reason=rejection_reason,
        ), True, False
    if state == "unknown":
        rejection_reason = "claim state is unknown"
        return build_gate_result(
            status="unknown",
            rule=rule,
            evidence_ids=evidence_ids,
            notes=notes,
            rejection_reason=rejection_reason,
        ), True, False
    if rule.reject_on_blocked_by and blocked_by:
        rejection_reason = "claim is blocked by dossier dependencies: " + ", ".join(blocked_by)
        return build_gate_result(
            status="blocked",
            rule=rule,
            evidence_ids=evidence_ids,
            notes=notes,
            rejection_reason=rejection_reason,
        ), True, False
    if state not in rule.passing_states:
        rejection_reason = f"claim state `{state}` does not satisfy rule `{rule.rule_id}`"
        return build_gate_result(
            status="fail",
            rule=rule,
            evidence_ids=evidence_ids,
            notes=notes,
            rejection_reason=rejection_reason,
        ), True, False
    required_results = required_probe_results(probe_results)
    if rule.require_required_probe_pass_if_present and required_results and any(
        result["status"] != "passed" for result in required_results
    ):
        rejection_reason = "required_for_gate probe did not pass"
        return build_gate_result(
            status="fail",
            rule=rule,
            evidence_ids=evidence_ids,
            notes=notes,
            rejection_reason=rejection_reason,
        ), True, False
    actual_kinds = effective_evidence_kinds(dossier=dossier, evidence_ids=evidence_ids)
    missing_message = missing_required_evidence_message(rule=rule, actual_kinds=actual_kinds)
    if missing_message is not None:
        rejection_reason = missing_message
        return build_gate_result(
            status="fail",
            rule=rule,
            evidence_ids=evidence_ids,
            notes=notes,
            rejection_reason=rejection_reason,
        ), True, False
    return build_gate_result(
        status="pass",
        rule=rule,
        evidence_ids=evidence_ids,
        notes=notes,
    ), False, True


def claim_state_rank(state: str) -> int:
    return {
        "verified": 3,
        "inferred": 2,
        "unknown": 1,
        "blocked": 0,
    }[state]


def evidence_kind_counts(dossier: dict[str, Any]) -> dict[str, int]:
    counts = {kind: 0 for kind in ALLOWED_EVIDENCE_KINDS}
    for entry in dossier["evidence"]:
        counts[entry["kind"]] += 1
    return counts


def candidate_note(dossier: dict[str, Any], *, probe_results: list[dict[str, Any]]) -> str:
    cited: list[str] = []
    for claim_key in ("crate_first_fit", "reproducibility", "non_interactive_execution"):
        cited.extend(dossier["claims"][claim_key]["evidence_ids"])
    for result in probe_results:
        if result["captured_output_ref"]:
            cited.append(result["captured_output_ref"])
    seen: list[str] = []
    for value in cited:
        if value not in seen:
            seen.append(value)
    return "refs=" + ",".join(seen[:6]) if seen else "refs=none"


def score_adoption(dossier: dict[str, Any]) -> int:
    counts = evidence_kind_counts(dossier)
    leverage_rank = claim_state_rank(dossier["claims"]["future_leverage"]["state"])
    if counts["github"] >= 1 and counts["package_registry"] >= 1 and leverage_rank >= 2:
        return 3
    if counts["github"] >= 1 and (counts["package_registry"] >= 1 or counts["official_doc"] >= 1):
        return 2
    if counts["github"] >= 1 or counts["package_registry"] >= 1:
        return 1
    return 0


def score_maturity(dossier: dict[str, Any]) -> int:
    observable_rank = claim_state_rank(dossier["claims"]["observable_cli_surface"]["state"])
    execution_rank = claim_state_rank(dossier["claims"]["non_interactive_execution"]["state"])
    counts = evidence_kind_counts(dossier)
    if observable_rank >= 2 and execution_rank >= 2 and counts["package_registry"] >= 1:
        return 3
    if observable_rank >= 2 and (counts["official_doc"] >= 1 or counts["package_registry"] >= 1):
        return 2
    if observable_rank >= 1:
        return 1
    return 0


def score_installability(dossier: dict[str, Any]) -> int:
    install_count = len(dossier["install_channels"])
    links = len(dossier["official_links"])
    state_rank = claim_state_rank(dossier["claims"]["non_interactive_execution"]["state"])
    if install_count >= 2 and links >= 2 and state_rank >= 2:
        return 3
    if install_count >= 1 and links >= 1 and state_rank >= 2:
        return 2
    if install_count >= 1:
        return 1
    return 0


def score_reproducibility(dossier: dict[str, Any]) -> int:
    reproducibility_rank = claim_state_rank(dossier["claims"]["reproducibility"]["state"])
    offline_rank = claim_state_rank(dossier["claims"]["offline_strategy"]["state"])
    if reproducibility_rank == 3 and offline_rank >= 2:
        return 3
    if reproducibility_rank >= 2 and offline_rank >= 2:
        return 2
    if reproducibility_rank >= 1:
        return 1
    return 0


def score_architecture_fit(dossier: dict[str, Any]) -> int:
    crate_rank = claim_state_rank(dossier["claims"]["crate_first_fit"]["state"])
    redaction_rank = claim_state_rank(dossier["claims"]["redaction_fit"]["state"])
    if crate_rank == 3 and redaction_rank == 3:
        return 3
    if crate_rank >= 2 and redaction_rank >= 2:
        return 2
    if crate_rank >= 1 or redaction_rank >= 1:
        return 1
    return 0


def score_future_leverage(dossier: dict[str, Any]) -> int:
    return {
        3: 3,
        2: 2,
        1: 1,
        0: 0,
    }[claim_state_rank(dossier["claims"]["future_leverage"]["state"])]


def score_candidate(dossier: dict[str, Any], *, probe_results: list[dict[str, Any]]) -> CandidateScore:
    scores = {
        "Adoption & community pull": score_adoption(dossier),
        "CLI product maturity & release activity": score_maturity(dossier),
        "Installability & docs quality": score_installability(dossier),
        "Reproducibility & access friction": score_reproducibility(dossier),
        "Architecture fit for this repo": score_architecture_fit(dossier),
        "Capability expansion / future leverage": score_future_leverage(dossier),
    }
    return CandidateScore(scores=scores, notes=candidate_note(dossier, probe_results=probe_results))


def shortlist_sort_key(agent_id: str, score: CandidateScore) -> tuple[int, int, int, int, int, int, str]:
    return (
        -score.primary_sum,
        -score.scores["Architecture fit for this repo"],
        -score.scores["Reproducibility & access friction"],
        -score.secondary_sum,
        -score.scores["CLI product maturity & release activity"],
        -score.scores["Adoption & community pull"],
        agent_id,
    )


def derived_pack_prefix(agent_id: str) -> str:
    return f"{agent_id.replace('_', '-')}-onboarding"


def render_approval_toml(
    *,
    candidate: CandidateSeed,
    defaults: DescriptorDefaults,
    recommended_agent_id: str,
    approved_agent_id: str,
    onboarding_pack_prefix: str,
    approval_commit: str,
    approval_recorded_at: str,
    override_reason: str | None,
) -> str:
    descriptor = candidate.derived_descriptor(defaults, agent_id=approved_agent_id)
    descriptor["onboarding_pack_prefix"] = onboarding_pack_prefix
    lines = [
        f'artifact_version = "{APPROVAL_VERSION}"',
        f'comparison_ref = "{CANONICAL_PACKET_REL}"',
        f'selection_mode = "{SELECTION_MODE}"',
        f'recommended_agent_id = "{recommended_agent_id}"',
        f'approved_agent_id = "{approved_agent_id}"',
        f'approval_commit = "{approval_commit}"',
        f'approval_recorded_at = "{approval_recorded_at}"',
    ]
    if recommended_agent_id != approved_agent_id:
        if not override_reason:
            raise RecommendationError("override_reason is required when approved_agent_id differs from recommended_agent_id")
        lines.append(f'override_reason = "{escape_toml_string(override_reason)}"')
    lines.extend(["", "[descriptor]"])
    ordered_keys = [
        "agent_id",
        "display_name",
        "crate_path",
        "backend_module",
        "manifest_root",
        "package_name",
        "canonical_targets",
        "wrapper_coverage_binding_kind",
        "wrapper_coverage_source_path",
        "always_on_capabilities",
        "backend_extensions",
        "support_matrix_enabled",
        "capability_matrix_enabled",
        "capability_matrix_target",
        "docs_release_track",
        "onboarding_pack_prefix",
    ]
    for key in ordered_keys:
        if key not in descriptor:
            continue
        lines.append(f"{key} = {toml_value(descriptor[key])}")
    for entry in descriptor.get("target_gated_capabilities", []):
        capability_id, targets = parse_target_gate_entry(entry)
        lines.extend(
            [
                "",
                "[[descriptor.target_gated_capabilities]]",
                f'capability_id = "{escape_toml_string(capability_id)}"',
                f"targets = {toml_value(targets)}",
            ]
        )
    for entry in descriptor.get("config_gated_capabilities", []):
        capability_id, config_key, targets = parse_config_gate_entry(entry)
        lines.extend(
            [
                "",
                "[[descriptor.config_gated_capabilities]]",
                f'capability_id = "{escape_toml_string(capability_id)}"',
                f'config_key = "{escape_toml_string(config_key)}"',
            ]
        )
        if targets:
            lines.append(f"targets = {toml_value(targets)}")
    return "\n".join(lines) + "\n"


def escape_toml_string(value: str) -> str:
    return value.replace("\\", "\\\\").replace('"', '\\"')


def parse_target_gate_entry(entry: str) -> tuple[str, list[str]]:
    capability_id, _, targets_raw = entry.partition(":")
    if not capability_id or not targets_raw:
        raise RecommendationError(f"invalid target gated capability entry `{entry}`")
    targets = [value.strip() for value in targets_raw.split(",") if value.strip()]
    if not targets:
        raise RecommendationError(f"invalid target gated capability targets in `{entry}`")
    return capability_id.strip(), targets


def parse_config_gate_entry(entry: str) -> tuple[str, str, list[str]]:
    parts = [part.strip() for part in entry.split(":")]
    if len(parts) < 2:
        raise RecommendationError(f"invalid config gated capability entry `{entry}`")
    capability_id = parts[0]
    config_key = parts[1]
    targets = [value.strip() for value in parts[2].split(",") if value.strip()] if len(parts) > 2 else []
    if not capability_id or not config_key:
        raise RecommendationError(f"invalid config gated capability entry `{entry}`")
    return capability_id, config_key, targets


def toml_value(value: Any) -> str:
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, str):
        return f'"{escape_toml_string(value)}"'
    if isinstance(value, list):
        inner = ", ".join(toml_value(item) for item in value)
        return f"[{inner}]"
    raise RecommendationError(f"unsupported TOML value {value!r}")


def render_metrics_lines(metrics: dict[str, Any]) -> list[str]:
    lines = ["- metrics:"]
    for key in RUN_STATUS_METRIC_KEYS:
        value = metrics[key]
        display = "pending" if value is None else str(value).lower() if isinstance(value, bool) else str(value)
        lines.append(f"  - {key}: {display}")
    return lines


def render_run_summary(
    *,
    mode: str,
    run_id: str,
    generated_at: str,
    recommended_agent_id: str | None,
    approved_agent_id: str | None,
    shortlist_ids: list[str],
    metrics: dict[str, Any],
    override_reason: str | None,
) -> str:
    approved_display = approved_agent_id if approved_agent_id is not None else "pending"
    lines = [
        "# Recommendation Run Summary",
        "",
        f"- run_id: {run_id}",
        f"- generated_at: {generated_at}",
        f"- recommended_agent_id: {recommended_agent_id or 'pending'}",
        f"- approved_agent_id: {approved_display}",
        f"- shortlist_ids: {', '.join(shortlist_ids) if shortlist_ids else 'pending'}",
    ]
    lines.extend(render_metrics_lines(metrics))
    if approved_agent_id and recommended_agent_id and approved_agent_id != recommended_agent_id:
        lines.extend(
            [
                "- override_summary:",
                f"  - approved_agent_id: {approved_agent_id}",
                f"  - recommended_agent_id: {recommended_agent_id}",
                f"  - override_reason: {override_reason or 'pending'}",
            ]
        )
    return "\n".join(lines) + "\n"


def render_labeled_bullet_section(section: dict[str, list[str]], labels: tuple[str, ...]) -> list[str]:
    lines: list[str] = []
    for label in labels:
        lines.append(f"{label}:")
        for entry in section[label]:
            lines.append(f"- {entry}")
        lines.append("")
    return lines


def labeled_section_blocks(section_text: str, labels: tuple[str, ...]) -> dict[str, list[str]]:
    blocks: dict[str, list[str]] = {}
    current_label: str | None = None
    for raw_line in section_text.splitlines():
        line = raw_line.strip()
        if line.startswith("## "):
            continue
        if not line or line.startswith("Provenance: "):
            continue
        matched_label = next((label for label in labels if line == f"{label}:"), None)
        if matched_label is not None:
            current_label = matched_label
            blocks[current_label] = []
            continue
        if current_label is not None:
            blocks[current_label].append(raw_line)
    return blocks


def exact_nonempty_labeled_section(section_text: str, labels: tuple[str, ...], *, section_heading: str) -> None:
    blocks = labeled_section_blocks(section_text, labels)
    if tuple(blocks.keys()) != labels:
        raise RecommendationError(f"{section_heading} labels do not match the locked template")
    for label in labels:
        content = [line for line in blocks[label] if line.strip()]
        if not content:
            raise RecommendationError(f"{section_heading} subsection `{label}` must not be empty")


def shortlist_loser_rationale(
    *,
    agent_id: str,
    recommended_agent_id: str,
    scores: dict[str, CandidateScore],
    dossiers: dict[str, dict[str, Any]],
) -> list[str]:
    loser = scores[agent_id]
    winner = scores[recommended_agent_id]
    dossier = dossiers[agent_id]
    if dossier["blocked_steps"]:
        return [
            f"`{agent_id}` ties or trails `{recommended_agent_id}` on the scorecard and still carries a blocked follow-up step: {dossier['blocked_steps'][0]}",
        ]
    weaker_dimensions = [
        dimension
        for dimension in DIMENSIONS
        if loser.scores[dimension] < winner.scores[dimension]
    ]
    if weaker_dimensions:
        cited = ", ".join(f"`{dimension}`" for dimension in weaker_dimensions[:2])
        return [
            f"`{agent_id}` loses because `{recommended_agent_id}` has stronger evidence-backed coverage on {cited}.",
        ]
    return [
        f"`{agent_id}` loses on the frozen shortlist tie-break chain after matching the score buckets; the winner keeps the cleaner recommendation path for immediate evaluation.",
    ]


def build_decision_surface(
    *,
    seed: SeedConfig,
    shortlist_ids: list[str],
    recommended_agent_id: str,
    scores: dict[str, CandidateScore],
    dossiers: dict[str, dict[str, Any]],
) -> DecisionSurface:
    recommended = seed.candidate_by_id(recommended_agent_id)
    recommended_dossier = dossiers[recommended_agent_id]
    descriptor = recommended.derived_descriptor(seed.defaults)
    probe_requests = recommended_dossier["probe_requests"]
    runnable_commands = [f"`{channel}`" for channel in recommended_dossier["install_channels"]]
    if probe_requests:
        runnable_commands.extend(
            f"`{probe_request['binary']} {PROBE_ARGS[probe_request['probe_kind']]}`"
            for probe_request in probe_requests
        )
    else:
        runnable_commands.extend(
            [
                f"`{recommended_agent_id} --help`",
                f"`{recommended_agent_id} --version`",
            ]
        )
    evidence_lines = [
        f"`{entry['evidence_id']}` (`{entry['kind']}`): {entry['title']}"
        for entry in recommended_dossier["evidence"]
    ]
    blocked_lines = list(recommended_dossier["blocked_steps"])
    if not blocked_lines:
        blocked_lines = [
            "any hosted or provider-only workflow remains blocked until a maintainer validates live account, auth, and billing requirements outside the local install path",
            "any capability that cannot be exercised from the local CLI surface remains blocked until wrapper-first evaluation artifacts are committed",
        ]
    winner_score = scores[recommended_agent_id]
    loser_rationales = {
        agent_id: shortlist_loser_rationale(
            agent_id=agent_id,
            recommended_agent_id=recommended_agent_id,
            scores=scores,
            dossiers=dossiers,
        )
        for agent_id in shortlist_ids
        if agent_id != recommended_agent_id
    }
    return DecisionSurface(
        winner_agent_id=recommended_agent_id,
        winner_display_name=recommended.display_name,
        winner_rationale=(
            f"`{recommended.display_name}` wins because it satisfies the strict hard-gate rules with the strongest immediate repo-fit evidence (`{winner_score.notes}`), "
            f"while preserving the best frozen shortlist position with primary score `{winner_score.primary_sum}` and secondary score `{winner_score.secondary_sum}`."
        ),
        loser_rationales=loser_rationales,
        section6_reproducible_now={
            "install paths": [f"`{channel}`" for channel in recommended_dossier["install_channels"]],
            "auth / account / billing prerequisites": recommended_dossier["auth_prerequisites"] or ["none before local install"],
            "runnable commands": runnable_commands,
            "evidence gatherable without paid or elevated access": evidence_lines,
            "expected artifacts to save during evaluation": [
                "redacted install log",
                "redacted `--help` output capture",
                "redacted `--version` output capture",
                f"notes linking saved artifacts back to `{recommended_agent_id}` dossier evidence ids",
            ],
        },
        section6_blocked_until_later=blocked_lines,
        section7={
            "Manifest root expectations": [
                f"keep generated manifests and review outputs aligned under `{descriptor['manifest_root']}` before backend integration work starts",
                "preserve the canonical comparison packet and approval artifact references while manifest-root surfaces are wired up",
            ],
            "Wrapper crate expectations": [
                f"start with the wrapper crate at `{descriptor['crate_path']}` as the first implementation stage",
                "keep CLI parsing, command execution, and event normalization inside the wrapper seam until behavior is proven",
            ],
            "`agent_api` backend expectations": [
                f"add backend adapter work under `{descriptor['backend_module']}` only after wrapper behavior is reviewable",
                "map wrapper outputs into existing phase-1 seams without widening the current contracts prematurely",
            ],
            "UAA promotion expectations": [
                "treat UAA promotion review as the final stage after wrapper and backend evidence exists",
                "do not treat support or capability matrix publication as a substitute for wrapper-first proof",
            ],
            "Support/publication expectations": [
                f"preserve `docs_release_track = \"{descriptor['docs_release_track']}\"` and the approved descriptor flags as the publication baseline",
                "land support-matrix or capability-matrix updates only when the implementation artifacts justify them",
            ],
            "Likely seam risks": [
                "CLI surface drift can invalidate parser assumptions between saved dossier evidence and real execution",
                "provider-gated or hosted workflows may remain untestable until maintainer access exists, so keep them outside the wrapper-first acceptance path",
            ],
        },
        section8={
            "Manifest-root artifacts": [
                f"committed manifest snapshots and review artifacts under `{descriptor['manifest_root']}`",
                "validation output proving manifest-root paths and packet references stay aligned",
            ],
            "Wrapper-crate artifacts": [
                f"wrapper crate code and tests under `{descriptor['crate_path']}`",
                "fixture-backed help/version captures or parser coverage notes for the approved CLI surface",
            ],
            "`agent_api` artifacts": [
                f"backend adapter code under `{descriptor['backend_module']}`",
                "integration tests or fixtures proving wrapper outputs map cleanly into `agent_api`",
            ],
            "UAA promotion-gate artifacts": [
                "dry-run approval validation via `cargo run -p xtask -- onboard-agent --approval ... --dry-run`",
                "promotion review evidence showing wrapper and backend outputs satisfy the approved packet",
            ],
            "Docs/spec artifacts": [
                "canonical packet, approval artifact, and any required `docs/specs/**` updates for real behavior changes",
                "repo guidance updates that point future maintainers at the approved onboarding seam",
            ],
            "Evidence/fixture artifacts": [
                "saved dossier evidence ids and probe output refs linked through `sources.lock.json`",
                "redacted local evaluation captures, fixtures, and blocker notes required to reproduce acceptance decisions",
            ],
        },
        section9={
            "Required workstreams": [
                "packet closeout and approval artifact review",
                "wrapper crate implementation",
                "`agent_api` backend integration",
                "UAA promotion review and matrix/publication closeout",
            ],
            "Required deliverables": [
                "approved comparison packet and governance artifact",
                "wrapper crate code, tests, and manifest outputs",
                "backend adapter code, tests, and updated repo evidence",
            ],
            "Blocking risks": [
                "provider or account-gated flows may block parity claims after local CLI install succeeds",
                "release drift between saved dossier evidence and current binaries can invalidate planned parsing assumptions",
            ],
            "Acceptance gates": [
                "packet and approval artifacts remain byte-stable except for allowed promote-time deltas",
                "wrapper crate proves the approved help/version/non-interactive surfaces with saved evidence",
                "backend adapter integration does not contradict existing `docs/specs/**` contracts",
                "at least 3 eligible candidates remain after hard gating and exactly 3 shortlisted candidates are documented",
            ],
        },
    )


def validate_decision_surface(
    surface: DecisionSurface,
    *,
    shortlist_ids: list[str],
    recommended_agent_id: str,
) -> None:
    if len(shortlist_ids) != 3:
        raise RecommendationError("DecisionSurface requires exactly 3 shortlisted candidates")
    for agent_id in shortlist_ids:
        if agent_id == recommended_agent_id:
            continue
        if agent_id not in surface.loser_rationales or not any(
            entry.strip() for entry in surface.loser_rationales[agent_id]
        ):
            raise RecommendationError(f"DecisionSurface missing explicit loser rationale for shortlisted non-winner `{agent_id}`")
    if tuple(surface.section6_reproducible_now.keys()) != SECTION6_REPRO_NOW_LABELS:
        raise RecommendationError("DecisionSurface section 6 reproducible-now labels do not match the locked template")
    runnable_commands = surface.section6_reproducible_now["runnable commands"]
    if not runnable_commands or not any("`" in entry and " " in entry for entry in runnable_commands):
        raise RecommendationError("DecisionSurface section 6 must include real commands")
    evidence_lines = surface.section6_reproducible_now["evidence gatherable without paid or elevated access"]
    if not evidence_lines:
        raise RecommendationError("DecisionSurface section 6 must include gatherable evidence")
    if not surface.section6_blocked_until_later:
        raise RecommendationError("DecisionSurface section 6 must include blocked steps")
    for labels, section, name in (
        (SECTION7_LABELS, surface.section7, "section 7"),
        (SECTION8_LABELS, surface.section8, "section 8"),
        (SECTION9_LABELS, surface.section9, "section 9"),
    ):
        if tuple(section.keys()) != labels:
            raise RecommendationError(f"DecisionSurface {name} labels do not match the locked template")
        for label in labels:
            if not section[label]:
                raise RecommendationError(f"DecisionSurface {name} subsection `{label}` must not be empty")


def render_comparison_packet(
    *,
    run_id: str,
    generated_at: str,
    seed: SeedConfig,
    shortlist_ids: list[str],
    recommended_agent_id: str,
    scores: dict[str, CandidateScore],
    dossiers: dict[str, dict[str, Any]],
    candidate_results: dict[str, CandidateResult],
) -> str:
    if len(shortlist_ids) != 3:
        raise RecommendationError("comparison packet requires exactly 3 shortlisted candidates")
    recommended = seed.candidate_by_id(recommended_agent_id)
    snapshot_date = generated_at[:10]
    recommended_dossier = dossiers[recommended_agent_id]
    decision_surface = build_decision_surface(
        seed=seed,
        shortlist_ids=shortlist_ids,
        recommended_agent_id=recommended_agent_id,
        scores=scores,
        dossiers=dossiers,
    )
    validate_decision_surface(
        decision_surface,
        shortlist_ids=shortlist_ids,
        recommended_agent_id=recommended_agent_id,
    )
    lines = [
        "<!-- generated-by: scripts/recommend_next_agent.py generate -->",
        "# Packet - CLI Agent Selection Packet",
        "",
        "Status: Generated",
        f"Date (UTC): {generated_at}",
        "Owner(s): wrappers team / deterministic runner",
        "Related source docs:",
        "- `docs/specs/cli-agent-recommendation-dossier-contract.md`",
        "- `docs/specs/cli-agent-onboarding-charter.md`",
        "- `docs/specs/unified-agent-api/support-matrix.md`",
        "- `docs/specs/**` for any normative contract this packet cites",
        "",
        f"Run id: `{run_id}`",
        "",
        "## 1. Candidate Summary",
        "",
        packet_template_provenance_line("## 1. Candidate Summary"),
        "",
        "Shortlisted candidates:",
    ]
    for agent_id in shortlist_ids:
        lines.append(f"- `{agent_id}`")
    lines.extend(
        [
            "",
            "Why these 3:",
            "- they are the highest-ranked eligible candidates under the frozen shortlist algorithm",
            "",
            "Recommendation in one sentence:",
            f"- `{recommended.display_name}` (`{recommended_agent_id}`) ranks first under the deterministic shortlist contract.",
            "",
            "## 2. What Already Exists",
            "",
            packet_template_provenance_line("## 2. What Already Exists"),
            "",
            "- `docs/specs/cli-agent-onboarding-charter.md`",
            "- `docs/specs/cli-agent-recommendation-dossier-contract.md`",
            "- `docs/templates/agent-selection/cli-agent-selection-packet-template.md`",
            "- `docs/cli-agent-onboarding-factory-operator-guide.md`",
            "- `crates/xtask/src/approval_artifact.rs`",
            "",
            "## 3. Selection Rubric",
            "",
            packet_template_provenance_line("## 3. Selection Rubric"),
            "",
            "This packet preserves the frozen score dimensions, the 0-3 scale, and the deterministic shortlist sort order. Product-value signals remain primary, while architecture fit and future leverage break ties only after the primary comparison is established.",
            "",
            "## 4. Fixed 3-Candidate Comparison Table",
            "",
            packet_template_provenance_line("## 4. Fixed 3-Candidate Comparison Table"),
            "",
            PACKET_TABLE_HEADER,
            PACKET_TABLE_DIVIDER,
        ]
    )
    for agent_id in shortlist_ids:
        candidate = seed.candidate_by_id(agent_id)
        score = scores[agent_id]
        lines.append(
            "| `{agent_id}` | {a} | {m} | {i} | {r} | {fit} | {future} | {notes} |".format(
                agent_id=agent_id,
                a=score.scores["Adoption & community pull"],
                m=score.scores["CLI product maturity & release activity"],
                i=score.scores["Installability & docs quality"],
                r=score.scores["Reproducibility & access friction"],
                fit=score.scores["Architecture fit for this repo"],
                future=score.scores["Capability expansion / future leverage"],
                notes=score.notes,
            )
        )
    lines.extend(
        [
            "",
            "## 5. Recommendation",
            "",
            packet_template_provenance_line("## 5. Recommendation"),
            "",
            f"Recommended winner: `{decision_surface.winner_agent_id}`",
            "",
            decision_surface.winner_rationale,
            "",
        ]
    )
    for agent_id in shortlist_ids:
        if agent_id == recommended_agent_id:
            continue
        for rationale in decision_surface.loser_rationales[agent_id]:
            lines.append(f"- {rationale}")
    lines.extend(
        [
            "",
            "Approve recommended agent",
            "Override to shortlisted alternative",
            "Stop and expand research",
            "",
            "## 6. Recommended Agent Evaluation Recipe",
            "",
            packet_template_provenance_line("## 6. Recommended Agent Evaluation Recipe"),
            "",
            "reproducible now:",
        ]
    )
    for label in SECTION6_REPRO_NOW_LABELS:
        lines.append(f"- {label}:")
        for entry in decision_surface.section6_reproducible_now[label]:
            lines.append(f"  - {entry}")
    lines.extend(["", "blocked until later:"])
    for entry in decision_surface.section6_blocked_until_later:
        lines.append(f"- {entry}")
    lines.extend(
        [
            "",
            "## 7. Repo-Fit Analysis",
            "",
            packet_template_provenance_line("## 7. Repo-Fit Analysis"),
            "",
        ]
    )
    lines.extend(render_labeled_bullet_section(decision_surface.section7, SECTION7_LABELS))
    lines.extend(
        [
            "## 8. Required Artifacts",
            "",
            packet_template_provenance_line("## 8. Required Artifacts"),
            "",
        ]
    )
    lines.extend(render_labeled_bullet_section(decision_surface.section8, SECTION8_LABELS))
    lines.extend(
        [
            "## 9. Workstreams, Deliverables, Risks, And Gates",
            "",
            packet_template_provenance_line("## 9. Workstreams, Deliverables, Risks, And Gates"),
            "",
        ]
    )
    lines.extend(render_labeled_bullet_section(decision_surface.section9, SECTION9_LABELS))
    lines.extend(
        [
            "## 10. Dated Evidence Appendix",
            "",
            packet_template_provenance_line("## 10. Dated Evidence Appendix"),
            "",
        ]
    )
    for agent_id in shortlist_ids:
        dossier = dossiers[agent_id]
        score = scores[agent_id]
        lines.append(f"### `{agent_id}`")
        lines.append("")
        lines.append(f"- Snapshot date: `{snapshot_date}`")
        lines.append("- Official links:")
        for link in dossier["official_links"]:
            lines.append(f"  - `{link}`")
        lines.append("- Install / distribution:")
        for channel in dossier["install_channels"]:
            lines.append(f"  - `{channel}`")
        lines.append("- Adoption / community:")
        lines.append(f"  - refs `{score.notes}`")
        lines.append("- Release activity:")
        for entry in dossier["evidence"]:
            lines.append(f"  - `{entry['evidence_id']}` `{entry['kind']}` captured `{entry['captured_at']}`")
        lines.append("- Access prerequisites:")
        for entry in dossier["auth_prerequisites"] or ["none"]:
            lines.append(f"  - {entry}")
        lines.append("- Normalized notes:")
        for entry in dossier["normalized_caveats"]:
            lines.append(f"  - {entry}")
        if agent_id == recommended_agent_id:
            lines.append("- Loser rationale: winner")
        else:
            lines.append("- Loser rationale: " + " ".join(decision_surface.loser_rationales[agent_id]))
        lines.append("")
    strategic_contenders = [
        candidate.agent_id
        for candidate in seed.candidates
        if candidate_results[candidate.agent_id].status != "eligible" and candidate.agent_id not in shortlist_ids
    ]
    if strategic_contenders:
        lines.append("### Strategic Contenders")
        lines.append("")
        for agent_id in strategic_contenders:
            rejection_reasons = candidate_results[agent_id].rejection_reasons or candidate_results[agent_id].error_reasons
            reason = rejection_reasons[0] if rejection_reasons else "see candidate validation result"
            lines.append(f"- `{agent_id}`: {reason}")
        lines.append("")
    return "\n".join(lines) + "\n"


def git_head(repo_root: Path) -> str:
    result = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=repo_root,
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout.strip()


def validate_approval_artifact(repo_root: Path, approval_path: Path) -> None:
    relative = approval_path.relative_to(repo_root)
    result = subprocess.run(
        ["cargo", "run", "-p", "xtask", "--", "onboard-agent", "--approval", str(relative), "--dry-run"],
        cwd=repo_root,
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        raise RecommendationError("approval artifact validation failed:\n" + result.stdout + result.stderr)


def serialize_candidate_validation(result: CandidateResult) -> dict[str, Any]:
    return {
        "agent_id": result.agent_id,
        "status": result.status,
        "schema_valid": result.schema_valid,
        "hard_gate_results": result.hard_gate_results,
        "probe_results": result.probe_results,
        "rejection_reasons": result.rejection_reasons,
        "error_reasons": result.error_reasons,
        "evidence_ids_used": result.evidence_ids_used,
        "notes": result.notes,
    }


def seed_snapshot_path(research_dir: Path) -> Path:
    return research_dir / "seed.snapshot.toml"


def research_summary_path(research_dir: Path) -> Path:
    return research_dir / "research-summary.md"


def research_metadata_path(research_dir: Path) -> Path:
    return research_dir / "research-metadata.json"


def dossier_path(research_dir: Path, agent_id: str) -> Path:
    return research_dir / "dossiers" / f"{agent_id}.json"


def build_candidate_pool_entry(
    candidate: CandidateSeed,
    *,
    result: CandidateResult,
    shortlisted: bool,
    recommended: bool,
) -> dict[str, Any]:
    return {
        "agent_id": candidate.agent_id,
        "status": result.status,
        "rejection_reasons": result.rejection_reasons,
        "error_reasons": result.error_reasons,
        "shortlisted": shortlisted,
        "recommended": recommended,
    }


def build_initial_run_artifacts(
    *,
    run_id: str,
    seed: SeedConfig,
    candidate_results: dict[str, CandidateResult],
) -> tuple[dict[str, Any], dict[str, Any]]:
    candidate_pool = {
        "run_id": run_id,
        "candidates": [
            build_candidate_pool_entry(
                candidate,
                result=candidate_results[candidate.agent_id],
                shortlisted=False,
                recommended=False,
            )
            for candidate in seed.candidates
        ],
    }
    eligible_candidates = {
        "run_id": run_id,
        "eligible_candidates": [],
    }
    return candidate_pool, eligible_candidates


def default_scorecard() -> dict[str, Any]:
    return {
        "dimensions": list(DIMENSIONS),
        "primary_dimensions": list(PRIMARY_DIMENSIONS),
        "secondary_dimensions": list(SECONDARY_DIMENSIONS),
        "shortlist_order": [],
        "recommended_agent_id": None,
        "candidates": {},
    }


def research_phase_fatal(
    *,
    message: str,
    code: str,
    run_status: dict[str, Any],
    run_dir: Path,
    seed_snapshot_bytes: bytes | None = None,
    candidate_results: dict[str, CandidateResult] | None = None,
    seed: SeedConfig | None = None,
) -> None:
    run_status["status"] = "run_fatal"
    run_status["errors"].append(build_run_error(scope="run", agent_id=None, code=code, message=message))
    if seed_snapshot_bytes is not None:
        write_bytes(run_dir / "seed.snapshot.toml", seed_snapshot_bytes)
    if seed is not None and candidate_results is not None:
        run_status["candidate_status_counts"] = candidate_status_counts(candidate_results)
        write_candidate_artifacts(run_dir=run_dir, seed=seed, candidate_results=candidate_results)
        candidate_pool, eligible_candidates = build_initial_run_artifacts(
            run_id=run_status["run_id"],
            seed=seed,
            candidate_results=candidate_results,
        )
        write_json(run_dir / "candidate-pool.json", candidate_pool)
        write_json(run_dir / "eligible-candidates.json", eligible_candidates)
        write_json(run_dir / "scorecard.json", default_scorecard())
        write_json(
            run_dir / "sources.lock.json",
            {"run_id": run_status["run_id"], "generated_at": run_status["generated_at"], "candidates": []},
        )
    write_json(run_dir / "run-status.json", run_status)
    raise RecommendationError(message)


def write_candidate_artifacts(
    *,
    run_dir: Path,
    seed: SeedConfig,
    candidate_results: dict[str, CandidateResult],
) -> None:
    validation_dir = run_dir / "candidate-validation-results"
    validation_dir.mkdir(parents=True, exist_ok=True)
    for candidate in seed.candidates:
        write_json(validation_dir / f"{candidate.agent_id}.json", serialize_candidate_validation(candidate_results[candidate.agent_id]))


def promote_time_seconds(*, generated_at: str, approval_recorded_at: str) -> int:
    return int((parse_timestamp(approval_recorded_at) - parse_timestamp(generated_at)).total_seconds())


def predicted_blocker_count_for_candidate(dossier: dict[str, Any]) -> int:
    blocked_claims = sum(1 for claim in dossier["claims"].values() if claim["state"] == "blocked")
    return blocked_claims + len(dossier["blocked_steps"])


def generate_recommendation(
    *,
    seed_file: Path,
    research_dir: Path,
    run_id: str,
    scratch_root: Path,
    registry_path: Path | None = None,
    now_fn: Callable[[], str] = utc_now,
) -> Path:
    run_dir = scratch_root / run_id
    remove_path(run_dir)
    run_dir.mkdir(parents=True, exist_ok=True)
    generated_at = now_fn()
    run_status = build_base_run_status(
        run_id=run_id,
        generated_at=generated_at,
        research_dir=research_dir,
        run_dir=run_dir,
    )

    try:
        validate_seed_file_exists(seed_file)
    except RecommendationError as exc:
        research_phase_fatal(
            message=str(exc),
            code="seed_file_missing",
            run_status=run_status,
            run_dir=run_dir,
        )

    if not research_dir.exists():
        research_phase_fatal(
            message=f"research dir `{research_dir}` does not exist",
            code="research_dir_missing",
            run_status=run_status,
            run_dir=run_dir,
        )
    if not research_summary_path(research_dir).exists():
        research_phase_fatal(
            message=f"research summary `{research_summary_path(research_dir)}` does not exist",
            code="research_summary_missing",
            run_status=run_status,
            run_dir=run_dir,
        )

    snapshot = seed_snapshot_path(research_dir)
    if not snapshot.exists():
        research_phase_fatal(
            message=f"seed snapshot `{snapshot}` does not exist",
            code="seed_snapshot_missing",
            run_status=run_status,
            run_dir=run_dir,
        )
    try:
        seed_snapshot_bytes = snapshot.read_bytes()
        seed = parse_seed_file(snapshot)
    except RecommendationError as exc:
        research_phase_fatal(
            message=str(exc),
            code="seed_snapshot_invalid",
            run_status=run_status,
            run_dir=run_dir,
        )
    seed_snapshot_sha = sha256_bytes(seed_snapshot_bytes)
    write_bytes(run_dir / "seed.snapshot.toml", seed_snapshot_bytes)

    actual_registry_path = registry_path or (REPO_ROOT / REGISTRY_RELATIVE_PATH)
    onboarded_agent_ids = load_onboarded_agent_ids(actual_registry_path)

    candidate_results = {
        candidate.agent_id: build_placeholder_candidate_result(candidate.agent_id)
        for candidate in seed.candidates
    }

    metadata_path = research_metadata_path(research_dir)
    try:
        metadata = validate_research_metadata(
            research_metadata_path=metadata_path,
            expected_run_id=run_id,
            research_dir=research_dir,
            run_dir=run_dir,
        )
    except RecommendationError as exc:
        research_phase_fatal(
            message=str(exc),
            code="research_metadata_invalid",
            run_status=run_status,
            run_dir=run_dir,
            seed_snapshot_bytes=seed_snapshot_bytes,
        )
    run_status["metrics"]["evidence_collection_time_seconds"] = metadata["evidence_collection_time_seconds"]
    run_status["metrics"]["fetched_source_count"] = metadata["fetched_source_count"]

    dossiers: dict[str, dict[str, Any]] = {}
    seeded_ids = {candidate.agent_id for candidate in seed.candidates}
    run_dossier_dir = run_dir / "candidate-dossiers"
    run_dossier_dir.mkdir(parents=True, exist_ok=True)
    for candidate in seed.candidates:
        candidate_path = dossier_path(research_dir, candidate.agent_id)
        if not candidate_path.exists():
            candidate_results[candidate.agent_id].error_reasons.append("missing required dossier")
            candidate_results[candidate.agent_id].notes.append("dossier is required for every seeded candidate")
            research_phase_fatal(
                message=f"missing dossier for seeded candidate `{candidate.agent_id}`",
                code="dossier_missing",
                run_status=run_status,
                run_dir=run_dir,
                seed_snapshot_bytes=seed_snapshot_bytes,
                candidate_results=candidate_results,
                seed=seed,
            )
        write_bytes(run_dossier_dir / f"{candidate.agent_id}.json", candidate_path.read_bytes())
        try:
            dossier = load_dossier_payload(candidate_path)
            validate_dossier_top_level(dossier, agent_id=candidate.agent_id, snapshot_sha=seed_snapshot_sha, seeded_ids=seeded_ids)
            validate_claim_evidence_links(dossier, agent_id=candidate.agent_id)
        except RecommendationError as exc:
            result = candidate_results[candidate.agent_id]
            result.status = "candidate_error"
            result.schema_valid = False
            result.error_reasons.append(str(exc))
            result.notes.append("schema validation failed")
            run_status["errors"].append(
                build_run_error(
                    scope="candidate",
                    agent_id=candidate.agent_id,
                    code="dossier_invalid",
                    message=str(exc),
                )
            )
            continue
        dossiers[candidate.agent_id] = dossier
        result = candidate_results[candidate.agent_id]
        result.schema_valid = True
        result.notes.append("schema validation passed")
        evidence_ids = [entry["evidence_id"] for entry in dossier["evidence"]]
        result.evidence_ids_used = evidence_ids
        result.probe_results = []

        probe_output_refs: list[dict[str, Any]] = []
        required_probe_failure = False
        for index, probe_request in enumerate(dossier["probe_requests"]):
            probe_result, probe_ref, failure_message = execute_probe(
                agent_id=candidate.agent_id,
                probe_request=probe_request,
                index=index,
            )
            result.probe_results.append(probe_result)
            if probe_ref is not None:
                probe_output_refs.append(probe_ref)
            if failure_message is not None:
                if probe_request["required_for_gate"]:
                    required_probe_failure = True
                    result.error_reasons.append(failure_message)
                else:
                    result.notes.append(failure_message)

        gate_failed = False
        for gate_key in HARD_GATE_KEYS:
            gate_result, failed, evidence_sufficient = evaluate_hard_gate(
                dossier=dossier,
                gate_key=gate_key,
                probe_results=result.probe_results,
            )
            result.hard_gate_results[gate_key] = gate_result
            if failed:
                gate_failed = True
                rejection_reason = gate_result["rejection_reason"] or f"{gate_key} gate status is {gate_result['status']}"
                result.rejection_reasons.append(
                    f"{gate_result['rule_id']}: {rejection_reason}"
                )
            if evidence_sufficient:
                for evidence_id in gate_result["evidence_ids"]:
                    if evidence_id not in result.evidence_ids_used:
                        result.evidence_ids_used.append(evidence_id)

        if candidate.agent_id in onboarded_agent_ids:
            gate_failed = True
            result.rejection_reasons.append(
                f"agent_id `{candidate.agent_id}` already exists in {REGISTRY_RELATIVE_PATH} and is already onboarded"
            )

        if required_probe_failure:
            result.status = "candidate_error"
            run_status["errors"].append(
                build_run_error(
                    scope="candidate",
                    agent_id=candidate.agent_id,
                    code="required_probe_failed",
                    message="a required gate probe failed",
                )
            )
            continue

        if gate_failed:
            result.status = "candidate_rejected"
            continue

        result.status = "eligible"
        result.score = score_candidate(dossier, probe_results=result.probe_results)

        dossier["_probe_output_refs"] = probe_output_refs

    counts = candidate_status_counts(candidate_results)
    run_status["candidate_status_counts"] = counts
    run_status["metrics"]["rejected_before_scoring_count"] = counts["candidate_rejected"] + counts["candidate_error"]

    eligible_ids = [
        candidate.agent_id
        for candidate in seed.candidates
        if candidate_results[candidate.agent_id].status == "eligible" and candidate_results[candidate.agent_id].score is not None
    ]
    eligible_ids.sort(key=lambda agent_id: shortlist_sort_key(agent_id, candidate_results[agent_id].score or CandidateScore({}, "")))
    shortlist_ids = eligible_ids[:3] if len(eligible_ids) >= 3 else []
    recommended_agent_id = shortlist_ids[0] if shortlist_ids else None

    run_status["eligible_candidate_ids"] = eligible_ids
    run_status["shortlist_ids"] = shortlist_ids
    run_status["recommended_agent_id"] = recommended_agent_id

    candidate_pool = {
        "run_id": run_id,
        "candidates": [
            build_candidate_pool_entry(
                candidate,
                result=candidate_results[candidate.agent_id],
                shortlisted=candidate.agent_id in shortlist_ids,
                recommended=candidate.agent_id == recommended_agent_id,
            )
            for candidate in seed.candidates
        ],
    }
    eligible_candidates = {
        "run_id": run_id,
        "eligible_candidates": [
            {
                "agent_id": agent_id,
                "scores": candidate_results[agent_id].score.scores,
                "primary_sum": candidate_results[agent_id].score.primary_sum,
                "secondary_sum": candidate_results[agent_id].score.secondary_sum,
            }
            for agent_id in eligible_ids
            if candidate_results[agent_id].score is not None
        ],
    }
    scorecard = {
        "dimensions": list(DIMENSIONS),
        "primary_dimensions": list(PRIMARY_DIMENSIONS),
        "secondary_dimensions": list(SECONDARY_DIMENSIONS),
        "shortlist_order": shortlist_ids,
        "recommended_agent_id": recommended_agent_id,
        "candidates": {
            agent_id: {
                "scores": candidate_results[agent_id].score.scores,
                "primary_sum": candidate_results[agent_id].score.primary_sum,
                "secondary_sum": candidate_results[agent_id].score.secondary_sum,
                "notes": candidate_results[agent_id].score.notes,
            }
            for agent_id in eligible_ids
            if candidate_results[agent_id].score is not None
        },
    }
    sources_lock = {
        "run_id": run_id,
        "generated_at": generated_at,
        "candidates": [],
    }
    for candidate in seed.candidates:
        dossier = dossiers.get(candidate.agent_id)
        evidence_refs = []
        probe_output_refs = []
        if dossier:
            evidence_refs = [
                {
                    "evidence_id": entry["evidence_id"],
                    "kind": entry["kind"],
                    **({"url": entry["url"]} if "url" in entry else {}),
                    "title": entry["title"],
                    "captured_at": entry["captured_at"],
                    "sha256": entry["sha256"],
                }
                for entry in dossier["evidence"]
            ]
            probe_output_refs = dossier.get("_probe_output_refs", [])
            dossier.pop("_probe_output_refs", None)
        sources_lock["candidates"].append(
            {
                "agent_id": candidate.agent_id,
                "evidence_refs": evidence_refs,
                "probe_output_refs": probe_output_refs,
            }
        )

    write_candidate_artifacts(run_dir=run_dir, seed=seed, candidate_results=candidate_results)
    write_json(run_dir / "candidate-pool.json", candidate_pool)
    write_json(run_dir / "eligible-candidates.json", eligible_candidates)
    write_json(run_dir / "scorecard.json", scorecard)
    write_json(run_dir / "sources.lock.json", sources_lock)

    if len(eligible_ids) < 3:
        run_status["status"] = "insufficient_eligible_candidates"
        write_json(run_dir / "run-status.json", run_status)
        write_text(
            run_dir / "run-summary.md",
            render_run_summary(
                mode="generate",
                run_id=run_id,
                generated_at=generated_at,
                recommended_agent_id=None,
                approved_agent_id=None,
                shortlist_ids=[],
                metrics=run_status["metrics"],
                override_reason=None,
            ),
        )
        raise RecommendationError("fewer than 3 eligible candidates remain after gating")

    recommended_candidate = seed.candidate_by_id(recommended_agent_id or shortlist_ids[0])
    run_status["status"] = "success_with_candidate_errors" if counts["candidate_error"] > 0 else "success"
    write_json(run_dir / "run-status.json", run_status)
    write_text(
        run_dir / "run-summary.md",
        render_run_summary(
            mode="generate",
            run_id=run_id,
            generated_at=generated_at,
            recommended_agent_id=recommended_agent_id,
            approved_agent_id=None,
            shortlist_ids=shortlist_ids,
            metrics=run_status["metrics"],
            override_reason=None,
        ),
    )
    write_text(
        run_dir / "comparison.generated.md",
        render_comparison_packet(
            run_id=run_id,
            generated_at=generated_at,
            seed=seed,
            shortlist_ids=shortlist_ids,
            recommended_agent_id=recommended_agent_id or shortlist_ids[0],
            scores={agent_id: candidate_results[agent_id].score for agent_id in shortlist_ids if candidate_results[agent_id].score is not None},
            dossiers=dossiers,
            candidate_results=candidate_results,
        ),
    )
    write_text(
        run_dir / "approval-draft.generated.toml",
        render_approval_toml(
            candidate=recommended_candidate,
            defaults=seed.defaults,
            recommended_agent_id=recommended_agent_id or shortlist_ids[0],
            approved_agent_id=recommended_agent_id or shortlist_ids[0],
            onboarding_pack_prefix=derived_pack_prefix(recommended_agent_id or shortlist_ids[0]),
            approval_commit="0000000",
            approval_recorded_at=generated_at,
            override_reason=None,
        ),
    )
    validate_scratch_outputs(
        run_dir=run_dir,
        run_status=run_status,
        seed=seed,
        research_dir=research_dir,
        candidate_pool=candidate_pool,
        eligible_candidates=eligible_candidates,
        scorecard=scorecard,
        sources_lock=sources_lock,
        candidate_results=candidate_results,
    )
    return run_dir


def ensure_promotable_run(run_dir: Path) -> SeedConfig:
    seed = parse_seed_file(run_dir / "seed.snapshot.toml")
    ensure_run_artifact_set(run_dir, seed)
    return seed


def finalize_run_status_for_promote(
    *,
    scratch_status: dict[str, Any],
    approved_agent_id: str,
    approval_recorded_at: str,
    override_reason: str | None,
    committed_review_dir: str,
    committed_packet_path: str,
    committed_approval_artifact_path: str,
    approved_dossier: dict[str, Any],
) -> dict[str, Any]:
    finalized = json.loads(json.dumps(scratch_status))
    finalized["approved_agent_id"] = approved_agent_id
    finalized["approval_recorded_at"] = approval_recorded_at
    finalized["override_reason"] = override_reason
    finalized["committed_review_dir"] = committed_review_dir
    finalized["committed_packet_path"] = committed_packet_path
    finalized["committed_approval_artifact_path"] = committed_approval_artifact_path
    finalized["metrics"]["maintainer_time_to_decision_seconds"] = promote_time_seconds(
        generated_at=finalized["generated_at"],
        approval_recorded_at=approval_recorded_at,
    )
    finalized["metrics"]["shortlist_override"] = approved_agent_id != finalized["recommended_agent_id"]
    finalized["metrics"]["predicted_blocker_count"] = predicted_blocker_count_for_candidate(approved_dossier)
    finalized["metrics"]["later_discovered_blocker_count"] = None
    return finalized


def promote_recommendation(
    *,
    run_dir: Path,
    repo_run_root_rel: str,
    approved_agent_id: str,
    onboarding_pack_prefix: str,
    override_reason: str | None,
    repo_root: Path = REPO_ROOT,
    now_fn: Callable[[], str] = utc_now,
    git_head_fn: Callable[[Path], str] = git_head,
    validator: Callable[[Path, Path], None] = validate_approval_artifact,
    replace_fn: Callable[[Path, Path], None] = os.replace,
) -> Path:
    seed = ensure_promotable_run(run_dir)
    live_seed = repo_root / LIVE_SEED_REL
    if not live_seed.exists():
        raise RecommendationError(f"live seed file `{live_seed}` does not exist")
    scorecard = read_json(run_dir / "scorecard.json")
    shortlist_ids = list(scorecard["shortlist_order"])
    recommended_agent_id = scorecard["recommended_agent_id"]
    if approved_agent_id not in shortlist_ids:
        raise RecommendationError("approved_agent_id must be one of the shortlisted 3 candidates")
    if approved_agent_id != recommended_agent_id and not override_reason:
        raise RecommendationError("override_reason is required when approved_agent_id differs from recommended_agent_id")

    approved_candidate = seed.candidate_by_id(approved_agent_id)
    approved_dossier = read_json(run_dir / "candidate-dossiers" / f"{approved_agent_id}.json")

    review_root = repo_root / repo_run_root_rel
    final_review_dir = review_root / run_dir.name
    if final_review_dir.exists():
        raise RecommendationError(f"review run directory `{final_review_dir}` already exists")

    temp_review_dir = review_root / f".tmp-{run_dir.name}"
    selection_staging_root = repo_root / "docs" / "agents" / "selection" / ".staging" / run_dir.name
    lifecycle_staging_root = repo_root / "docs" / "agents" / "lifecycle" / ".staging" / run_dir.name
    remove_path(temp_review_dir)
    remove_path(selection_staging_root)
    remove_path(lifecycle_staging_root)
    temp_review_dir.mkdir(parents=True, exist_ok=True)

    try:
        for item in run_dir.iterdir():
            if item.is_file():
                shutil.copy2(item, temp_review_dir / item.name)
            elif item.is_dir() and item.name in {"candidate-dossiers", "candidate-validation-results"}:
                shutil.copytree(item, temp_review_dir / item.name)

        approval_commit = git_head_fn(repo_root)
        approval_recorded_at = now_fn()
        final_approval_text = render_approval_toml(
            candidate=approved_candidate,
            defaults=seed.defaults,
            recommended_agent_id=recommended_agent_id,
            approved_agent_id=approved_agent_id,
            onboarding_pack_prefix=onboarding_pack_prefix,
            approval_commit=approval_commit,
            approval_recorded_at=approval_recorded_at,
            override_reason=override_reason,
        )
        canonical_path = canonical_packet_path(repo_root)
        staged_canonical_path = selection_staging_root / "cli-agent-selection-packet.md"
        final_approval_path = repo_root / "docs" / "agents" / "lifecycle" / onboarding_pack_prefix / "governance" / "approved-agent.toml"
        staged_approval_path = lifecycle_staging_root / onboarding_pack_prefix / "governance" / "approved-agent.toml"
        final_approval_path.parent.mkdir(parents=True, exist_ok=True)
        previous_canonical = canonical_path.read_bytes() if canonical_path.exists() else None
        previous_approval = final_approval_path.read_bytes() if final_approval_path.exists() else None

        scratch_status = read_json(run_dir / "run-status.json")
        final_status = finalize_run_status_for_promote(
            scratch_status=scratch_status,
            approved_agent_id=approved_agent_id,
            approval_recorded_at=approval_recorded_at,
            override_reason=override_reason,
            committed_review_dir=str(Path(repo_run_root_rel) / run_dir.name),
            committed_packet_path=CANONICAL_PACKET_REL,
            committed_approval_artifact_path=str(
                Path("docs/agents/lifecycle") / onboarding_pack_prefix / "governance" / "approved-agent.toml"
            ),
            approved_dossier=approved_dossier,
        )
        write_json(temp_review_dir / "run-status.json", final_status)
        write_text(
            temp_review_dir / "run-summary.md",
            render_run_summary(
                mode="promote",
                run_id=final_status["run_id"],
                generated_at=final_status["generated_at"],
                recommended_agent_id=final_status["recommended_agent_id"],
                approved_agent_id=approved_agent_id,
                shortlist_ids=final_status["shortlist_ids"],
                metrics=final_status["metrics"],
                override_reason=override_reason,
            ),
        )

        canonical_bytes = (run_dir / "comparison.generated.md").read_bytes()
        write_bytes(staged_canonical_path, canonical_bytes)
        write_text(staged_approval_path, final_approval_text)
        validator(repo_root, staged_approval_path)

        try:
            replace_fn(staged_canonical_path, canonical_path)
            replace_fn(staged_approval_path, final_approval_path)
            replace_fn(temp_review_dir, final_review_dir)
        except Exception:
            restore_file(canonical_path, previous_canonical, repo_root=repo_root)
            restore_file(final_approval_path, previous_approval, repo_root=repo_root)
            raise
    except Exception:
        remove_path(temp_review_dir)
        remove_path(selection_staging_root)
        remove_path(lifecycle_staging_root)
        raise
    remove_path(selection_staging_root)
    remove_path(lifecycle_staging_root)

    validate_promoted_outputs(
        scratch_run_dir=run_dir,
        scratch_status=scratch_status,
        final_review_dir=final_review_dir,
        final_status=final_status,
        seed=seed,
        research_dir=Path(scratch_status["research_dir"]),
        canonical_path=canonical_path,
        final_approval_path=final_approval_path,
        final_approval_text=final_approval_text,
    )
    return final_review_dir


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    try:
        if args.command == "generate":
            generate_recommendation(
                seed_file=Path(args.seed_file),
                research_dir=Path(os.path.expanduser(args.research_dir)),
                run_id=args.run_id,
                scratch_root=Path(os.path.expanduser(args.scratch_root)),
            )
        else:
            promote_recommendation(
                run_dir=Path(os.path.expanduser(args.run_dir)),
                repo_run_root_rel=args.repo_run_root,
                approved_agent_id=args.approved_agent_id,
                onboarding_pack_prefix=args.onboarding_pack_prefix,
                override_reason=args.override_reason,
            )
    except RecommendationError as exc:
        print(f"ERROR: {exc}")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
