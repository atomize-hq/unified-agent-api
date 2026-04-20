#!/usr/bin/env python3
"""Shared publish planning helpers for workspace crates."""

from __future__ import annotations

from collections import defaultdict
from dataclasses import dataclass
from enum import Enum
import heapq
import json
from pathlib import Path
import subprocess
import urllib.error
import urllib.request

DEFAULT_REGISTRY_URL = "https://crates.io"
KNOWN_PUBLISH_ORDER = [
    "unified-agent-api-codex",
    "unified-agent-api-claude-code",
    "unified-agent-api-opencode",
    "unified-agent-api-wrapper-events",
    "unified-agent-api",
]
KNOWN_PUBLISH_PRIORITY = {
    package_name: index for index, package_name in enumerate(KNOWN_PUBLISH_ORDER)
}


class PublishStrategy(str, Enum):
    SKIP = "skip"
    PUBLISH_WITH_OIDC = "publish-with-oidc"
    PUBLISH_WITH_BOOTSTRAP_TOKEN = "publish-with-bootstrap-token"


@dataclass(frozen=True)
class WorkspacePackage:
    name: str
    version: str
    manifest_path: Path
    workspace_order: int
    internal_dependencies: tuple[str, ...]

    @property
    def has_internal_dependencies(self) -> bool:
        return bool(self.internal_dependencies)


@dataclass(frozen=True)
class PlannedPublish:
    package: WorkspacePackage
    strategy: PublishStrategy


class RegistryClient:
    """Minimal crates.io API client used by publish planning."""

    def __init__(self, *, base_url: str = DEFAULT_REGISTRY_URL, timeout_seconds: float = 15.0) -> None:
        self.base_url = base_url.rstrip("/")
        self.timeout_seconds = timeout_seconds

    def crate_exists(self, crate: str) -> bool:
        return self._exists(f"/api/v1/crates/{crate}")

    def crate_version_exists(self, crate: str, version: str) -> bool:
        return self._exists(f"/api/v1/crates/{crate}/{version}")

    def _exists(self, path: str) -> bool:
        url = f"{self.base_url}{path}"
        request = urllib.request.Request(
            url,
            headers={"User-Agent": "unified-agent-api publish planner"},
        )
        try:
            with urllib.request.urlopen(request, timeout=self.timeout_seconds):
                return True
        except urllib.error.HTTPError as exc:
            if exc.code == 404:
                return False
            raise


def package_sort_key(package: WorkspacePackage) -> tuple[int, int, str]:
    priority = KNOWN_PUBLISH_PRIORITY.get(
        package.name,
        len(KNOWN_PUBLISH_PRIORITY) + package.workspace_order,
    )
    return (priority, package.workspace_order, package.name)


def run(
    cmd: list[str],
    *,
    cwd: Path,
    capture_output: bool = False,
    check: bool = True,
    env: dict[str, str] | None = None,
) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=cwd,
        check=check,
        capture_output=capture_output,
        text=True,
        env=env,
    )


def load_metadata(root: Path) -> dict:
    result = run(
        ["cargo", "metadata", "--format-version", "1", "--no-deps"],
        cwd=root,
        capture_output=True,
    )
    return json.loads(result.stdout)


def is_crates_io_publishable(package: dict) -> bool:
    """Return whether cargo metadata allows publishing this package to crates.io."""

    publish = package.get("publish")
    if publish is None:
        return True
    if publish is False:
        return False
    if publish == []:
        return False
    return "crates-io" in publish


def load_publishable_packages(root: Path) -> list[WorkspacePackage]:
    metadata = load_metadata(root)
    return workspace_publishable_packages_from_metadata(metadata)


def workspace_publishable_packages_from_metadata(metadata: dict) -> list[WorkspacePackage]:
    workspace_member_ids = list(metadata["workspace_members"])
    workspace_order_by_id = {package_id: index for index, package_id in enumerate(workspace_member_ids)}
    workspace_packages = [
        package
        for package in metadata["packages"]
        if package["id"] in workspace_order_by_id
    ]
    publishable_packages = [
        package
        for package in workspace_packages
        if is_crates_io_publishable(package)
    ]
    publishable_names = {package["name"] for package in publishable_packages}
    order_key_by_name = {
        package["name"]: (
            KNOWN_PUBLISH_PRIORITY.get(
                package["name"],
                len(KNOWN_PUBLISH_PRIORITY) + workspace_order_by_id[package["id"]],
            ),
            workspace_order_by_id[package["id"]],
            package["name"],
        )
        for package in publishable_packages
    }

    normalized: list[WorkspacePackage] = []
    for package in sorted(
        publishable_packages,
        key=lambda item: workspace_order_by_id[item["id"]],
    ):
        internal_dependencies = sorted(
            {
                dep["name"]
                for dep in package["dependencies"]
                if dep["name"] in publishable_names and dep.get("source") is None
            },
            key=lambda name: order_key_by_name[name],
        )
        normalized.append(
            WorkspacePackage(
                name=package["name"],
                version=package["version"],
                manifest_path=Path(package["manifest_path"]).resolve(),
                workspace_order=workspace_order_by_id[package["id"]],
                internal_dependencies=tuple(internal_dependencies),
            )
        )

    return normalized


def package_map(packages: list[WorkspacePackage]) -> dict[str, WorkspacePackage]:
    return {package.name: package for package in packages}


def build_dependents_map(packages: list[WorkspacePackage]) -> dict[str, tuple[str, ...]]:
    package_by_name = package_map(packages)
    dependents: dict[str, list[str]] = defaultdict(list)
    for package in packages:
        for dependency in package.internal_dependencies:
            dependents[dependency].append(package.name)

    return {
        package_name: tuple(
            sorted(names, key=lambda name: package_sort_key(package_by_name[name]))
        )
        for package_name, names in dependents.items()
    }


def topological_publish_order(packages: list[WorkspacePackage]) -> list[WorkspacePackage]:
    package_by_name = package_map(packages)
    dependents = build_dependents_map(packages)
    remaining_dependencies = {
        package.name: len(package.internal_dependencies)
        for package in packages
    }

    ready: list[tuple[int, str]] = [
        (package_sort_key(package)[0], package_sort_key(package)[1], package.name)
        for package in packages
        if remaining_dependencies[package.name] == 0
    ]
    heapq.heapify(ready)

    ordered_names: list[str] = []
    while ready:
        _, _, package_name = heapq.heappop(ready)
        ordered_names.append(package_name)
        for dependent_name in dependents.get(package_name, ()):
            remaining_dependencies[dependent_name] -= 1
            if remaining_dependencies[dependent_name] == 0:
                dependent = package_by_name[dependent_name]
                heapq.heappush(
                    ready,
                    (
                        package_sort_key(dependent)[0],
                        package_sort_key(dependent)[1],
                        dependent_name,
                    ),
                )

    if len(ordered_names) != len(packages):
        unresolved = sorted(
            name for name, count in remaining_dependencies.items() if count > 0
        )
        raise ValueError(
            "Publishable workspace dependency graph contains a cycle: "
            + ", ".join(unresolved)
        )

    return [package_by_name[name] for name in ordered_names]


def validate_release_version(
    packages: list[WorkspacePackage],
    *,
    release_version: str | None = None,
) -> None:
    if release_version is None:
        return

    mismatches = [
        f"{package.name}={package.version}"
        for package in packages
        if package.version != release_version
    ]
    if mismatches:
        raise ValueError(
            "Publish planner expected release version "
            f"{release_version}, but found: {', '.join(mismatches)}"
        )


def determine_strategy(
    package: WorkspacePackage,
    *,
    registry_client: RegistryClient,
) -> PublishStrategy:
    if registry_client.crate_version_exists(package.name, package.version):
        return PublishStrategy.SKIP
    if not registry_client.crate_exists(package.name):
        return PublishStrategy.PUBLISH_WITH_BOOTSTRAP_TOKEN
    return PublishStrategy.PUBLISH_WITH_OIDC


def plan_publish_actions(
    packages: list[WorkspacePackage],
    *,
    registry_client: RegistryClient,
    release_version: str | None = None,
) -> list[PlannedPublish]:
    validate_release_version(packages, release_version=release_version)
    ordered = topological_publish_order(packages)
    return [
        PlannedPublish(
            package=package,
            strategy=determine_strategy(package, registry_client=registry_client),
        )
        for package in ordered
    ]
