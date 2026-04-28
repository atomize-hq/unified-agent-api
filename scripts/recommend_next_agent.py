#!/usr/bin/env python3
from __future__ import annotations

import argparse
from collections.abc import Callable
from dataclasses import asdict, dataclass
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tomllib
from typing import Any
import urllib.error
import urllib.parse
import urllib.request

REPO_ROOT = Path(__file__).resolve().parents[1]
CANONICAL_PACKET_REL = "docs/agents/selection/cli-agent-selection-packet.md"
DEFAULT_TARGET = "darwin-arm64"
APPROVAL_VERSION = "1"
SELECTION_MODE = "factory_validation"
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
SCRATCH_ARTIFACT_FILES = (
    "candidate-pool.json",
    "eligible-candidates.json",
    "scorecard.json",
    "sources.lock.json",
    "comparison.generated.md",
    "approval-draft.generated.toml",
    "run-summary.md",
)
COPY_OWNED_REVIEW_FILES = (
    "candidate-pool.json",
    "eligible-candidates.json",
    "scorecard.json",
    "sources.lock.json",
    "comparison.generated.md",
    "approval-draft.generated.toml",
    "run-summary.md",
)
AUTH_FRICTION_KEYWORDS = (
    "auth",
    "provider",
    "credential",
    "credentials",
    "account",
    "billing",
    "paid",
    "subscription",
    "tier",
)
ARCHITECTURE_KEYWORDS = (
    "run",
    "json",
    "serve",
    "server",
    "headless",
    "stdin",
    "stdout",
    "automation",
    "agent",
    "tool",
    "terminal",
)
LEVERAGE_KEYWORDS = (
    "subagent",
    "session",
    "fork",
    "model",
    "automation",
    "tool",
    "api",
    "server",
    "json",
    "workflow",
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
            "canonical_targets": self.overrides.get(
                "canonical_targets",
                defaults.canonical_targets,
            ),
            "wrapper_coverage_binding_kind": self.overrides.get(
                "wrapper_coverage_binding_kind",
                defaults.wrapper_coverage_binding_kind,
            ),
            "wrapper_coverage_source_path": self.overrides.get(
                "wrapper_coverage_source_path",
                crate_path,
            ),
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
            "backend_extensions": self.overrides.get(
                "backend_extensions",
                defaults.backend_extensions,
            ),
            "support_matrix_enabled": self.overrides.get(
                "support_matrix_enabled",
                defaults.support_matrix_enabled,
            ),
            "capability_matrix_enabled": self.overrides.get(
                "capability_matrix_enabled",
                defaults.capability_matrix_enabled,
            ),
            "docs_release_track": self.overrides.get(
                "docs_release_track",
                defaults.docs_release_track,
            ),
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
class SourceRecord:
    url: str
    kind: str
    fetched_at: str
    final_url: str
    summary: dict[str, Any]


@dataclass(frozen=True)
class CandidateEvidence:
    install_channel_count: int
    docs_source_count: int
    fetched_source_count: int
    auth_friction_hits: int
    github_stars: int
    package_version_count: int
    release_age_days: int | None
    architecture_keyword_hits: int
    leverage_keyword_hits: int
    corpus_excerpt: str


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


def build_parser() -> argparse.ArgumentParser:
    parser = NoAbbrevArgumentParser(description="Generate and promote the next CLI agent recommendation lane.")
    subparsers = parser.add_subparsers(dest="command", required=True, parser_class=NoAbbrevArgumentParser)

    generate = subparsers.add_parser("generate")
    generate.add_argument("--seed-file", required=True)
    generate.add_argument("--run-id", required=True)
    generate.add_argument("--scratch-root", required=True)

    promote = subparsers.add_parser("promote")
    promote.add_argument("--run-dir", required=True)
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


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def write_json(path: Path, data: Any) -> None:
    write_text(path, json_dumps(data))


def remove_path(path: Path) -> None:
    if not path.exists():
        return
    if path.is_dir():
        shutil.rmtree(path)
    else:
        path.unlink()


def canonical_packet_path(repo_root: Path) -> Path:
    return repo_root / CANONICAL_PACKET_REL


def parse_seed_file(seed_path: Path) -> SeedConfig:
    data = tomllib.loads(read_text(seed_path))
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
    for agent_id, candidate_data in sorted(data["candidate"].items()):
        keys = set(candidate_data.keys())
        required_missing = REQUIRED_CANDIDATE_KEYS - keys
        if required_missing:
            raise RecommendationError(
                f"candidate `{agent_id}` is missing required keys: {', '.join(sorted(required_missing))}"
            )
        allowed = REQUIRED_CANDIDATE_KEYS | OPTIONAL_CANDIDATE_KEYS
        unknown = keys - allowed
        if unknown:
            raise RecommendationError(f"candidate `{agent_id}` has unsupported keys: {', '.join(sorted(unknown))}")
        candidates.append(
            CandidateSeed(
                agent_id=agent_id,
                display_name=candidate_data["display_name"],
                research_urls=list(candidate_data["research_urls"]),
                install_channels=list(candidate_data["install_channels"]),
                auth_notes=candidate_data["auth_notes"],
                overrides={key: candidate_data[key] for key in OPTIONAL_CANDIDATE_KEYS if key in candidate_data},
            )
        )
    if len(candidates) < 3:
        raise RecommendationError("seed file must define at least 3 candidates")
    return SeedConfig(defaults=defaults, candidates=candidates)


def fetch_url(url: str) -> SourceRecord:
    fetched_at = utc_now()
    parsed = urllib.parse.urlparse(url)
    if parsed.netloc == "github.com":
        return fetch_github_repo(url, fetched_at)
    if parsed.netloc == "www.npmjs.com":
        return fetch_npm_package(url, fetched_at)
    if parsed.netloc == "pypi.org":
        return fetch_pypi_package(url, fetched_at)
    return fetch_generic_page(url, fetched_at)


def urlopen_json(url: str) -> tuple[Any, str]:
    request = urllib.request.Request(url, headers={"User-Agent": "unified-agent-api recommend-next-agent"})
    with urllib.request.urlopen(request, timeout=20) as response:
        final_url = response.geturl()
        payload = response.read().decode("utf-8")
    return json.loads(payload), final_url


def urlopen_text(url: str) -> tuple[str, str]:
    request = urllib.request.Request(url, headers={"User-Agent": "unified-agent-api recommend-next-agent"})
    with urllib.request.urlopen(request, timeout=20) as response:
        final_url = response.geturl()
        payload = response.read(250_000).decode("utf-8", errors="replace")
    return payload, final_url


def fetch_github_repo(url: str, fetched_at: str) -> SourceRecord:
    parsed = urllib.parse.urlparse(url)
    parts = [part for part in parsed.path.split("/") if part]
    if len(parts) < 2:
        raise RecommendationError(f"unsupported GitHub repo url `{url}`")
    owner, repo = parts[0], parts[1]
    repo_data, final_url = urlopen_json(f"https://api.github.com/repos/{owner}/{repo}")
    release_summary: dict[str, Any] = {}
    try:
        release_data, _ = urlopen_json(f"https://api.github.com/repos/{owner}/{repo}/releases/latest")
        release_summary = {
            "latest_release_name": release_data.get("name") or release_data.get("tag_name"),
            "latest_release_published_at": release_data.get("published_at"),
        }
    except urllib.error.HTTPError as exc:
        if exc.code != 404:
            raise
    summary = {
        "repo": repo_data["full_name"],
        "description": repo_data.get("description"),
        "stars": repo_data.get("stargazers_count", 0),
        "forks": repo_data.get("forks_count", 0),
        "open_issues": repo_data.get("open_issues_count", 0),
        "updated_at": repo_data.get("updated_at"),
        "pushed_at": repo_data.get("pushed_at"),
        "topics": repo_data.get("topics", []),
    }
    summary.update(release_summary)
    return SourceRecord(url=url, kind="github_repo", fetched_at=fetched_at, final_url=final_url, summary=summary)


def fetch_npm_package(url: str, fetched_at: str) -> SourceRecord:
    parsed = urllib.parse.urlparse(url)
    match = re.search(r"/package/(.+)$", parsed.path)
    if not match:
        raise RecommendationError(f"unsupported npm package url `{url}`")
    package_name = urllib.parse.unquote(match.group(1))
    encoded = urllib.parse.quote(package_name, safe="@")
    package_data, final_url = urlopen_json(f"https://registry.npmjs.org/{encoded}")
    times = package_data.get("time", {})
    summary = {
        "package_name": package_name,
        "latest_version": package_data.get("dist-tags", {}).get("latest"),
        "modified": times.get("modified"),
        "created": times.get("created"),
        "version_count": len(package_data.get("versions", {})),
        "description": package_data.get("description"),
    }
    return SourceRecord(url=url, kind="npm_package", fetched_at=fetched_at, final_url=final_url, summary=summary)


def fetch_pypi_package(url: str, fetched_at: str) -> SourceRecord:
    parsed = urllib.parse.urlparse(url)
    match = re.search(r"/project/([^/]+)/?", parsed.path)
    if not match:
        raise RecommendationError(f"unsupported PyPI package url `{url}`")
    package_name = urllib.parse.unquote(match.group(1))
    package_data, final_url = urlopen_json(f"https://pypi.org/pypi/{package_name}/json")
    release_dates = sorted(
        item.get("upload_time_iso_8601")
        for items in package_data.get("releases", {}).values()
        for item in items
        if item.get("upload_time_iso_8601")
    )
    summary = {
        "package_name": package_name,
        "latest_version": package_data.get("info", {}).get("version"),
        "release_count": len(package_data.get("releases", {})),
        "latest_upload_time": release_dates[-1] if release_dates else None,
        "description": package_data.get("info", {}).get("summary"),
    }
    return SourceRecord(url=url, kind="pypi_package", fetched_at=fetched_at, final_url=final_url, summary=summary)


def fetch_generic_page(url: str, fetched_at: str) -> SourceRecord:
    html, final_url = urlopen_text(url)
    title_match = re.search(r"<title[^>]*>(.*?)</title>", html, re.IGNORECASE | re.DOTALL)
    title = html_unescape(strip_tags(title_match.group(1))) if title_match else None
    text = strip_tags(html)
    text = re.sub(r"\s+", " ", text).strip()
    lowered = text.lower()
    summary = {
        "title": title,
        "snippet": text[:500],
        "keyword_hits": {
            "architecture": keyword_hits(lowered, ARCHITECTURE_KEYWORDS),
            "leverage": keyword_hits(lowered, LEVERAGE_KEYWORDS),
        },
    }
    return SourceRecord(url=url, kind="generic_page", fetched_at=fetched_at, final_url=final_url, summary=summary)


def strip_tags(text: str) -> str:
    return re.sub(r"<[^>]+>", " ", text)


def html_unescape(text: str) -> str:
    return (
        text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", '"')
        .replace("&#39;", "'")
    )


def keyword_hits(text: str, terms: tuple[str, ...]) -> int:
    return sum(text.count(term) for term in terms)


def iso_age_days(iso_value: str | None) -> int | None:
    if not iso_value:
        return None
    value = datetime.fromisoformat(iso_value.replace("Z", "+00:00"))
    return (datetime.now(timezone.utc) - value).days


def collect_evidence(candidate: CandidateSeed, records: list[SourceRecord]) -> CandidateEvidence:
    texts: list[str] = [candidate.auth_notes.lower()]
    github_stars = 0
    package_version_count = 0
    release_age_days: int | None = None
    docs_source_count = 0
    for record in records:
        if record.kind == "github_repo":
            github_stars = max(github_stars, int(record.summary.get("stars") or 0))
            pushed_days = iso_age_days(record.summary.get("pushed_at"))
            if pushed_days is not None:
                release_age_days = pushed_days if release_age_days is None else min(release_age_days, pushed_days)
            texts.extend(
                str(value).lower()
                for value in (
                    record.summary.get("description"),
                    " ".join(record.summary.get("topics", [])),
                    record.summary.get("latest_release_name"),
                )
                if value
            )
        elif record.kind == "npm_package":
            package_version_count = max(package_version_count, int(record.summary.get("version_count") or 0))
            modified_days = iso_age_days(record.summary.get("modified"))
            if modified_days is not None:
                release_age_days = modified_days if release_age_days is None else min(release_age_days, modified_days)
            texts.append(str(record.summary.get("description") or "").lower())
        elif record.kind == "pypi_package":
            package_version_count = max(package_version_count, int(record.summary.get("release_count") or 0))
            upload_days = iso_age_days(record.summary.get("latest_upload_time"))
            if upload_days is not None:
                release_age_days = upload_days if release_age_days is None else min(release_age_days, upload_days)
            texts.append(str(record.summary.get("description") or "").lower())
        else:
            docs_source_count += 1
            texts.append(str(record.summary.get("title") or "").lower())
            texts.append(str(record.summary.get("snippet") or "").lower())
    corpus = " ".join(texts)
    return CandidateEvidence(
        install_channel_count=len(candidate.install_channels),
        docs_source_count=docs_source_count,
        fetched_source_count=len(records),
        auth_friction_hits=keyword_hits(candidate.auth_notes.lower(), AUTH_FRICTION_KEYWORDS),
        github_stars=github_stars,
        package_version_count=package_version_count,
        release_age_days=release_age_days,
        architecture_keyword_hits=keyword_hits(corpus, ARCHITECTURE_KEYWORDS),
        leverage_keyword_hits=keyword_hits(corpus, LEVERAGE_KEYWORDS),
        corpus_excerpt=corpus[:500],
    )


def eligibility(candidate: CandidateSeed, evidence: CandidateEvidence, records: list[SourceRecord]) -> list[str]:
    reasons: list[str] = []
    if evidence.fetched_source_count < 2:
        reasons.append("insufficient external evidence was captured")
    if not any(record.kind == "github_repo" for record in records):
        reasons.append("missing inspectable GitHub repo evidence")
    if evidence.install_channel_count == 0:
        reasons.append("no non-interactive install path is declared")
    if evidence.architecture_keyword_hits == 0:
        reasons.append("non-interactive CLI/runtime surface evidence is too weak")
    return reasons


def score_candidate(evidence: CandidateEvidence) -> CandidateScore:
    scores = {
        "Adoption & community pull": score_adoption(evidence),
        "CLI product maturity & release activity": score_maturity(evidence),
        "Installability & docs quality": score_installability(evidence),
        "Reproducibility & access friction": score_reproducibility(evidence),
        "Architecture fit for this repo": score_architecture_fit(evidence),
        "Capability expansion / future leverage": score_future_leverage(evidence),
    }
    notes = (
        f"stars={evidence.github_stars}, installs={evidence.install_channel_count}, "
        f"docs={evidence.docs_source_count}, auth_hits={evidence.auth_friction_hits}, "
        f"architecture_hits={evidence.architecture_keyword_hits}, leverage_hits={evidence.leverage_keyword_hits}"
    )
    return CandidateScore(scores=scores, notes=notes)


def score_adoption(evidence: CandidateEvidence) -> int:
    if evidence.github_stars >= 20_000:
        return 3
    if evidence.github_stars >= 5_000:
        return 2
    if evidence.github_stars >= 1_000:
        return 1
    return 0


def score_maturity(evidence: CandidateEvidence) -> int:
    if evidence.package_version_count <= 0 or evidence.release_age_days is None:
        return 0
    if evidence.release_age_days <= 45:
        return 3
    if evidence.release_age_days <= 180:
        return 2
    if evidence.release_age_days <= 365:
        return 1
    return 0


def score_installability(evidence: CandidateEvidence) -> int:
    if evidence.install_channel_count >= 2 and evidence.docs_source_count >= 1 and evidence.fetched_source_count >= 3:
        return 3
    if evidence.install_channel_count >= 1 and evidence.docs_source_count >= 1:
        return 2
    if evidence.install_channel_count >= 1:
        return 1
    return 0


def score_reproducibility(evidence: CandidateEvidence) -> int:
    if evidence.auth_friction_hits <= 1:
        return 3
    if evidence.auth_friction_hits <= 3:
        return 2
    if evidence.auth_friction_hits <= 5:
        return 1
    return 0


def score_architecture_fit(evidence: CandidateEvidence) -> int:
    if evidence.architecture_keyword_hits >= 16:
        return 3
    if evidence.architecture_keyword_hits >= 8:
        return 2
    if evidence.architecture_keyword_hits >= 2:
        return 1
    return 0


def score_future_leverage(evidence: CandidateEvidence) -> int:
    if evidence.leverage_keyword_hits >= 16:
        return 3
    if evidence.leverage_keyword_hits >= 8:
        return 2
    if evidence.leverage_keyword_hits >= 2:
        return 1
    return 0


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


def build_candidate_pool_entry(
    candidate: CandidateSeed,
    *,
    eligible: bool,
    rejection_reasons: list[str],
    shortlisted: bool,
    recommended: bool,
    score: CandidateScore | None,
) -> dict[str, Any]:
    return {
        "agent_id": candidate.agent_id,
        "display_name": candidate.display_name,
        "eligible": eligible,
        "rejection_reasons": rejection_reasons,
        "shortlisted": shortlisted,
        "recommended": recommended,
        "score": score.scores if score else None,
        "notes": score.notes if score else None,
    }


def render_run_summary(
    *,
    mode: str,
    run_id: str,
    generated_at: str,
    recommended_agent_id: str,
    shortlist_ids: list[str],
    approved_agent_id: str | None = None,
    onboarding_pack_prefix: str | None = None,
    override_reason: str | None = None,
) -> str:
    lines = [
        f"# Recommendation Run Summary",
        "",
        f"- mode: `{mode}`",
        f"- run_id: `{run_id}`",
        f"- generated_at: `{generated_at}`",
        f"- shortlist: {', '.join(f'`{agent_id}`' for agent_id in shortlist_ids)}",
        f"- recommended_agent_id: `{recommended_agent_id}`",
    ]
    if approved_agent_id:
        lines.append(f"- approved_agent_id: `{approved_agent_id}`")
    if onboarding_pack_prefix:
        lines.append(f"- onboarding_pack_prefix: `{onboarding_pack_prefix}`")
    if override_reason:
        lines.append(f"- override_reason: {override_reason}")
    return "\n".join(lines) + "\n"


def render_comparison_packet(
    *,
    run_id: str,
    generated_at: str,
    seed: SeedConfig,
    shortlist_ids: list[str],
    recommended_agent_id: str,
    scores: dict[str, CandidateScore],
    records_by_agent: dict[str, list[SourceRecord]],
) -> str:
    recommended = seed.candidate_by_id(recommended_agent_id)
    lines = [
        "<!-- generated-by: scripts/recommend_next_agent.py generate -->",
        "# Packet — CLI Agent Selection Recommendation",
        "",
        f"Status: Generated",
        f"Date (UTC): {generated_at}",
        f"Run id: `{run_id}`",
        "Related source docs:",
        "- `docs/specs/cli-agent-onboarding-charter.md`",
        "- `docs/templates/agent-selection/cli-agent-selection-packet-template.md`",
        "- `docs/cli-agent-onboarding-factory-operator-guide.md`",
        "",
        "## 1. Candidate Summary",
        "",
        "Provenance: `dated external snapshot evidence + maintainer inference encoded by the deterministic runner`",
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
            "Provenance: `committed repo evidence`",
            "",
            "- `docs/specs/cli-agent-onboarding-charter.md`",
            "- `docs/templates/agent-selection/cli-agent-selection-packet-template.md`",
            "- `docs/cli-agent-onboarding-factory-operator-guide.md`",
            "- `crates/xtask/src/approval_artifact.rs`",
            "",
            "## 3. Selection Rubric",
            "",
            "Provenance: `maintainer inference informed by dated external snapshot evidence`",
            "",
            "This packet uses the frozen score buckets and the deterministic shortlist sort order. It does not publish a weighted total column.",
            "",
            "## 4. Fixed 3-Candidate Comparison Table",
            "",
            "Provenance: `dated external snapshot evidence + deterministic runner scoring`",
            "",
            "| Candidate | Adoption & community pull | CLI product maturity & release activity | Installability & docs quality | Reproducibility & access friction | Architecture fit for this repo | Capability expansion / future leverage | Notes |",
            "|---|---:|---:|---:|---:|---:|---:|---|",
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
            "Provenance: `maintainer inference grounded in the comparison table`",
            "",
            f"Recommended winner: `{recommended_agent_id}`",
            "",
            f"`{recommended.display_name}` ranks first after deterministic tie-break ordering.",
            "",
            "## 6. Recommended Agent Evaluation Recipe",
            "",
            "Provenance: `dated external snapshot evidence + seed inputs`",
            "",
            f"Recommended agent: `{recommended.display_name}`",
            "",
            "Install paths:",
        ]
    )
    for channel in recommended.install_channels:
        lines.append(f"- `{channel}`")
    lines.extend(
        [
            "",
            "Auth / access notes:",
            f"- {recommended.auth_notes}",
            "",
            "## 7. Repo-Fit Analysis",
            "",
            "Provenance: `committed repo evidence + deterministic descriptor derivation`",
            "",
        ]
    )
    descriptor = recommended.derived_descriptor(seed.defaults)
    lines.extend(
        [
            f"- crate path: `{descriptor['crate_path']}`",
            f"- backend module: `{descriptor['backend_module']}`",
            f"- manifest root: `{descriptor['manifest_root']}`",
            f"- package name: `{descriptor['package_name']}`",
            "",
            "## 8. Required Artifacts",
            "",
            "Provenance: `committed repo evidence + maintainer inference`",
            "",
            "- canonical comparison packet",
            "- approval artifact draft",
            "- committed review run artifacts",
            "- wrapper/backend follow-on surfaces after approval",
            "",
            "## 9. Workstreams, Deliverables, Risks, And Gates",
            "",
            "Provenance: `maintainer inference grounded in repo constraints`",
            "",
            "- workstreams: contract, runner, validation, proving, integration",
            "- deliverables: seed file, skill, runner, tests, review run, approval draft",
            "- risks: source drift, insufficient eligible candidates, approval validation failure",
            "- gates: exactly 3 shortlisted candidates, successful approval dry-run, green validation",
            "",
            "## 10. Dated Evidence Appendix",
            "",
            "Provenance: `dated external snapshot evidence`",
            "",
        ]
    )
    for agent_id in shortlist_ids:
        candidate = seed.candidate_by_id(agent_id)
        lines.append(f"### `{agent_id}`")
        lines.append("")
        lines.append(f"- display name: `{candidate.display_name}`")
        for record in records_by_agent[agent_id]:
            lines.append(f"- `{record.kind}` `{record.url}` fetched `{record.fetched_at}`")
        lines.append("")
    lines.extend(
        [
            "## 11. Acceptance Checklist",
            "",
            "Provenance: `deterministic runner output`",
            "",
            "- [x] The packet compares exactly 3 candidates.",
            "- [x] The packet names one deterministic recommendation.",
            "- [x] The appendix preserves dated source provenance for each shortlisted candidate.",
        ]
    )
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
        raise RecommendationError(
            "approval artifact validation failed:\n"
            + result.stdout
            + result.stderr
        )


def generate_recommendation(
    *,
    seed_file: Path,
    run_id: str,
    scratch_root: Path,
    fetcher: Callable[[str], SourceRecord] = fetch_url,
    now_fn: Callable[[], str] = utc_now,
) -> Path:
    seed = parse_seed_file(seed_file)
    run_dir = scratch_root / run_id
    remove_path(run_dir)
    run_dir.mkdir(parents=True, exist_ok=True)
    generated_at = now_fn()
    sources_lock: dict[str, Any] = {
        "run_id": run_id,
        "generated_at": generated_at,
        "candidates": {},
    }
    scores: dict[str, CandidateScore] = {}
    eligibility_by_agent: dict[str, list[str]] = {}
    records_by_agent: dict[str, list[SourceRecord]] = {}
    evidence_by_agent: dict[str, CandidateEvidence] = {}

    for candidate in seed.candidates:
        records: list[SourceRecord] = []
        for url in candidate.research_urls:
            try:
                records.append(fetcher(url))
            except Exception as exc:
                raise RecommendationError(f"source capture failed for `{candidate.agent_id}` url `{url}`: {exc}") from exc
        evidence = collect_evidence(candidate, records)
        rejection_reasons = eligibility(candidate, evidence, records)
        records_by_agent[candidate.agent_id] = records
        evidence_by_agent[candidate.agent_id] = evidence
        eligibility_by_agent[candidate.agent_id] = rejection_reasons
        if not rejection_reasons:
            scores[candidate.agent_id] = score_candidate(evidence)
        sources_lock["candidates"][candidate.agent_id] = {
            "display_name": candidate.display_name,
            "records": [asdict(record) for record in records],
        }

    eligible_ids = sorted(agent_id for agent_id, reasons in eligibility_by_agent.items() if not reasons)
    if len(eligible_ids) < 3:
        write_json(run_dir / "sources.lock.json", sources_lock)
        raise RecommendationError("fewer than 3 eligible candidates remain after gating")

    shortlist_ids = sorted(eligible_ids, key=lambda agent_id: shortlist_sort_key(agent_id, scores[agent_id]))[:3]
    recommended_agent_id = shortlist_ids[0]
    recommended_candidate = seed.candidate_by_id(recommended_agent_id)

    candidate_pool = []
    for candidate in seed.candidates:
        candidate_pool.append(
            build_candidate_pool_entry(
                candidate,
                eligible=not eligibility_by_agent[candidate.agent_id],
                rejection_reasons=eligibility_by_agent[candidate.agent_id],
                shortlisted=candidate.agent_id in shortlist_ids,
                recommended=candidate.agent_id == recommended_agent_id,
                score=scores.get(candidate.agent_id),
            )
        )

    eligible_candidates = [
        {
            "agent_id": agent_id,
            "display_name": seed.candidate_by_id(agent_id).display_name,
            "score": scores[agent_id].scores,
            "primary_sum": scores[agent_id].primary_sum,
            "secondary_sum": scores[agent_id].secondary_sum,
        }
        for agent_id in sorted(eligible_ids, key=lambda candidate_id: shortlist_sort_key(candidate_id, scores[candidate_id]))
    ]

    scorecard = {
        "dimensions": list(DIMENSIONS),
        "primary_dimensions": list(PRIMARY_DIMENSIONS),
        "secondary_dimensions": list(SECONDARY_DIMENSIONS),
        "shortlist_order": shortlist_ids,
        "recommended_agent_id": recommended_agent_id,
        "candidates": {
            agent_id: {
                "scores": scores[agent_id].scores,
                "primary_sum": scores[agent_id].primary_sum,
                "secondary_sum": scores[agent_id].secondary_sum,
                "notes": scores[agent_id].notes,
            }
            for agent_id in sorted(scores)
        },
    }

    provisional_approval = render_approval_toml(
        candidate=recommended_candidate,
        defaults=seed.defaults,
        recommended_agent_id=recommended_agent_id,
        approved_agent_id=recommended_agent_id,
        onboarding_pack_prefix=derived_pack_prefix(recommended_agent_id),
        approval_commit="0000000",
        approval_recorded_at=generated_at,
        override_reason=None,
    )
    comparison_packet = render_comparison_packet(
        run_id=run_id,
        generated_at=generated_at,
        seed=seed,
        shortlist_ids=shortlist_ids,
        recommended_agent_id=recommended_agent_id,
        scores=scores,
        records_by_agent=records_by_agent,
    )
    run_summary = render_run_summary(
        mode="generate",
        run_id=run_id,
        generated_at=generated_at,
        recommended_agent_id=recommended_agent_id,
        shortlist_ids=shortlist_ids,
    )

    write_json(run_dir / "candidate-pool.json", {"run_id": run_id, "candidates": candidate_pool})
    write_json(run_dir / "eligible-candidates.json", {"run_id": run_id, "eligible_candidates": eligible_candidates})
    write_json(run_dir / "scorecard.json", scorecard)
    write_json(run_dir / "sources.lock.json", sources_lock)
    write_text(run_dir / "comparison.generated.md", comparison_packet)
    write_text(run_dir / "approval-draft.generated.toml", provisional_approval)
    write_text(run_dir / "run-summary.md", run_summary)

    dossiers_dir = run_dir / "candidate-dossiers"
    dossiers_dir.mkdir(parents=True, exist_ok=True)
    for agent_id in shortlist_ids:
        candidate = seed.candidate_by_id(agent_id)
        dossier = {
            "agent_id": agent_id,
            "display_name": candidate.display_name,
            "research_urls": candidate.research_urls,
            "install_channels": candidate.install_channels,
            "auth_notes": candidate.auth_notes,
            "descriptor": candidate.derived_descriptor(seed.defaults),
            "evidence": asdict(evidence_by_agent[agent_id]),
            "score": {
                "scores": scores[agent_id].scores,
                "primary_sum": scores[agent_id].primary_sum,
                "secondary_sum": scores[agent_id].secondary_sum,
                "notes": scores[agent_id].notes,
            },
            "sources": [asdict(record) for record in records_by_agent[agent_id]],
        }
        write_json(dossiers_dir / f"{agent_id}.json", dossier)

    ensure_scratch_artifacts_complete(run_dir, shortlist_ids)
    return run_dir


def ensure_scratch_artifacts_complete(run_dir: Path, shortlist_ids: list[str]) -> None:
    for artifact in SCRATCH_ARTIFACT_FILES:
        path = run_dir / artifact
        if not path.exists():
            raise RecommendationError(f"required scratch artifact `{artifact}` is missing")
    dossiers_dir = run_dir / "candidate-dossiers"
    actual_dossiers = sorted(path.name for path in dossiers_dir.glob("*.json"))
    expected_dossiers = sorted(f"{agent_id}.json" for agent_id in shortlist_ids)
    if actual_dossiers != expected_dossiers:
        raise RecommendationError(
            "scratch candidate dossiers do not match the shortlisted candidates"
        )


def load_json(path: Path) -> Any:
    return json.loads(read_text(path))


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
    scorecard = load_json(run_dir / "scorecard.json")
    shortlist_ids = list(scorecard["shortlist_order"])
    recommended_agent_id = scorecard["recommended_agent_id"]
    ensure_scratch_artifacts_complete(run_dir, shortlist_ids)
    if approved_agent_id not in shortlist_ids:
        raise RecommendationError("approved_agent_id must be one of the shortlisted 3 candidates")
    if approved_agent_id != recommended_agent_id and not override_reason:
        raise RecommendationError("override_reason is required when approved_agent_id differs from recommended_agent_id")

    seed = parse_seed_file(repo_root / "docs/agents/selection/candidate-seed.toml")
    approved_candidate = seed.candidate_by_id(approved_agent_id)
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
        for artifact in COPY_OWNED_REVIEW_FILES:
            shutil.copy2(run_dir / artifact, temp_review_dir / artifact)
        source_dossiers = run_dir / "candidate-dossiers"
        review_dossiers = temp_review_dir / "candidate-dossiers"
        review_dossiers.mkdir(parents=True, exist_ok=True)
        for agent_id in shortlist_ids:
            shutil.copy2(source_dossiers / f"{agent_id}.json", review_dossiers / f"{agent_id}.json")

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
        canonical_bytes = (run_dir / "comparison.generated.md").read_bytes()
        staged_canonical_path = selection_staging_root / "cli-agent-selection-packet.md"
        final_approval_path = repo_root / "docs" / "agents" / "lifecycle" / onboarding_pack_prefix / "governance" / "approved-agent.toml"
        staged_approval_path = lifecycle_staging_root / onboarding_pack_prefix / "governance" / "approved-agent.toml"
        final_approval_path.parent.mkdir(parents=True, exist_ok=True)
        previous_canonical = canonical_path.read_bytes() if canonical_path.exists() else None
        previous_approval = final_approval_path.read_bytes() if final_approval_path.exists() else None

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

    if (final_review_dir / "comparison.generated.md").read_bytes() != canonical_path.read_bytes():
        raise RecommendationError("canonical packet must be byte-identical to the review comparison packet")
    for artifact in COPY_OWNED_REVIEW_FILES:
        if (final_review_dir / artifact).read_bytes() != (run_dir / artifact).read_bytes():
            raise RecommendationError(f"committed review artifact `{artifact}` must be a byte-copy of the scratch run")
    if read_text(final_approval_path) != final_approval_text:
        raise RecommendationError("final approval artifact must match the promote-time rendered approval contents")
    return final_review_dir


def write_bytes(path: Path, contents: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(contents)


def restore_file(path: Path, previous_bytes: bytes | None, *, repo_root: Path) -> None:
    if previous_bytes is None:
        remove_path(path)
        parent = path.parent
        while parent != repo_root and parent.exists() and not any(parent.iterdir()):
            parent.rmdir()
            parent = parent.parent
        return
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(previous_bytes)


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    try:
        if args.command == "generate":
            generate_recommendation(
                seed_file=Path(args.seed_file),
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
