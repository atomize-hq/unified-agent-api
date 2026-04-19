#!/usr/bin/env python3
"""Plan and execute crates.io publishes for workspace crates."""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import sys
import time

from publish_planner import (
    DEFAULT_REGISTRY_URL,
    PlannedPublish,
    PublishStrategy,
    RegistryClient,
    WorkspacePackage,
    build_dependents_map,
    load_publishable_packages,
    plan_publish_actions,
    run,
)

OIDC_TOKEN_ENV = "OIDC_CARGO_REGISTRY_TOKEN"
LEGACY_OIDC_TOKEN_ENV = "CARGO_REGISTRY_TOKEN"
BOOTSTRAP_TOKEN_ENV = "BOOTSTRAP_CARGO_REGISTRY_TOKEN"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--plan", action="store_true", help="print publish order and strategy")
    mode.add_argument("--execute", action="store_true", help="publish crates in dependency order")
    parser.add_argument("--root", default=".", help="workspace root")
    parser.add_argument("--release-version", required=True, help="root release version")
    parser.add_argument(
        "--registry-url",
        default=os.environ.get("CRATES_IO_REGISTRY_URL", DEFAULT_REGISTRY_URL),
        help="registry base URL",
    )
    parser.add_argument(
        "--poll-attempts",
        type=int,
        default=40,
        help="max attempts while waiting for registry visibility",
    )
    parser.add_argument(
        "--poll-interval-seconds",
        type=float,
        default=15.0,
        help="sleep between registry visibility checks",
    )
    return parser.parse_args()


def select_registry_token(strategy: PublishStrategy, env: dict[str, str]) -> str | None:
    if strategy is PublishStrategy.SKIP:
        return None

    if strategy is PublishStrategy.PUBLISH_WITH_OIDC:
        token = env.get(OIDC_TOKEN_ENV) or env.get(LEGACY_OIDC_TOKEN_ENV)
        if token:
            return token
        raise RuntimeError(
            "Missing OIDC token for existing-crate publish. Set "
            f"{OIDC_TOKEN_ENV} (or {LEGACY_OIDC_TOKEN_ENV}) before running --execute."
        )

    token = env.get(BOOTSTRAP_TOKEN_ENV)
    if token:
        return token
    raise RuntimeError(
        "Detected first-publish crate but no bootstrap token is available. Set "
        f"{BOOTSTRAP_TOKEN_ENV} from the protected release secret before running --execute."
    )


def cargo_publish_dry_run(root: Path, package: WorkspacePackage) -> tuple[bool, str]:
    result = run(
        ["cargo", "publish", "--dry-run", "--locked", "-p", package.name],
        cwd=root,
        capture_output=True,
        check=False,
    )
    output = result.stderr.strip() or result.stdout.strip()
    return result.returncode == 0, output


def wait_for_package_readiness(
    root: Path,
    package: WorkspacePackage,
    *,
    registry_client: RegistryClient,
    poll_attempts: int,
    poll_interval_seconds: float,
) -> None:
    if not package.has_internal_dependencies:
        return

    if registry_client.crate_version_exists(package.name, package.version):
        print(f"{package.name}@{package.version} already exists; skipping readiness wait.")
        return

    last_output = ""
    for attempt in range(1, poll_attempts + 1):
        success, output = cargo_publish_dry_run(root, package)
        if success:
            print(
                f"{package.name}@{package.version} passed publish dry-run on attempt "
                f"{attempt}/{poll_attempts}."
            )
            return

        last_output = output
        print(
            f"Waiting for registry visibility before publishing {package.name}@{package.version} "
            f"(attempt {attempt}/{poll_attempts})."
        )
        time.sleep(poll_interval_seconds)

    raise RuntimeError(
        f"Timed out waiting for {package.name}@{package.version} to become publishable."
        + (f" Last cargo output: {last_output}" if last_output else "")
    )


def wait_for_downstream_progress(
    root: Path,
    published: PlannedPublish,
    pending: list[PlannedPublish],
    *,
    registry_client: RegistryClient,
    dependents_map: dict[str, tuple[str, ...]],
    poll_attempts: int,
    poll_interval_seconds: float,
) -> None:
    if not pending:
        return

    dependent_names = set(dependents_map.get(published.package.name, ()))
    if not dependent_names:
        return

    downstream = next(
        (
            item.package
            for item in pending
            if item.package.name in dependent_names and item.strategy is not PublishStrategy.SKIP
        ),
        None,
    )
    if downstream is None:
        return

    for attempt in range(1, poll_attempts + 1):
        if registry_client.crate_version_exists(published.package.name, published.package.version):
            print(
                f"{published.package.name}@{published.package.version} is visible in the registry "
                f"after publish (attempt {attempt}/{poll_attempts})."
            )
            return

        success, _ = cargo_publish_dry_run(root, downstream)
        if success:
            print(
                f"{downstream.name}@{downstream.version} is publishable after "
                f"{published.package.name}@{published.package.version} (attempt {attempt}/{poll_attempts})."
            )
            return

        print(
            f"Waiting for downstream readiness after publishing "
            f"{published.package.name}@{published.package.version} (attempt {attempt}/{poll_attempts})."
        )
        time.sleep(poll_interval_seconds)

    raise RuntimeError(
        "Timed out waiting for downstream progress after publishing "
        f"{published.package.name}@{published.package.version}."
    )


def publish_package(
    root: Path,
    planned: PlannedPublish,
    *,
    token: str,
) -> None:
    print(
        f"Publishing {planned.package.name}@{planned.package.version} via "
        f"{planned.strategy.value}."
    )
    publish_env = os.environ.copy()
    publish_env[LEGACY_OIDC_TOKEN_ENV] = token
    run(
        ["cargo", "publish", "--locked", "-p", planned.package.name],
        cwd=root,
        env=publish_env,
    )


def print_plan(planned: list[PlannedPublish]) -> None:
    print("Computed publish plan:")
    for item in planned:
        print(f"- {item.package.name}@{item.package.version}: {item.strategy.value}")


def execute(args: argparse.Namespace) -> int:
    root = Path(args.root).resolve()
    packages = load_publishable_packages(root)
    registry_client = RegistryClient(base_url=args.registry_url)
    planned = plan_publish_actions(
        packages,
        registry_client=registry_client,
        release_version=args.release_version,
    )
    print_plan(planned)

    if args.plan:
        return 0

    dependents_map = build_dependents_map([item.package for item in planned])
    env = dict(os.environ)
    for index, item in enumerate(planned):
        if item.strategy is PublishStrategy.SKIP:
            print(f"Skipping {item.package.name}@{item.package.version}; already published.")
            continue

        wait_for_package_readiness(
            root,
            item.package,
            registry_client=registry_client,
            poll_attempts=args.poll_attempts,
            poll_interval_seconds=args.poll_interval_seconds,
        )
        token = select_registry_token(item.strategy, env)
        publish_package(root, item, token=token)
        wait_for_downstream_progress(
            root,
            item,
            planned[index + 1 :],
            registry_client=registry_client,
            dependents_map=dependents_map,
            poll_attempts=args.poll_attempts,
            poll_interval_seconds=args.poll_interval_seconds,
        )

    return 0


def main() -> int:
    args = parse_args()
    try:
        return execute(args)
    except Exception as exc:
        print(f"publish_crates: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
