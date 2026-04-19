from __future__ import annotations

import unittest

from publish_crates import (
    BOOTSTRAP_TOKEN_ENV,
    OIDC_TOKEN_ENV,
    select_registry_token,
)
from publish_planner import (
    PublishStrategy,
    plan_publish_actions,
    topological_publish_order,
    workspace_publishable_packages_from_metadata,
)


def package_fixture(
    *,
    package_id: str,
    name: str,
    version: str,
    manifest_path: str,
    dependencies: list[dict],
    publish: list[str] | None = None,
) -> dict:
    return {
        "id": package_id,
        "name": name,
        "version": version,
        "manifest_path": manifest_path,
        "dependencies": dependencies,
        "publish": publish,
    }


def dep_fixture(name: str, *, internal: bool = True) -> dict:
    return {
        "name": name,
        "source": None if internal else "registry+https://github.com/rust-lang/crates.io-index",
        "req": "=0.2.3",
    }


def metadata_fixture() -> dict:
    workspace_members = [
        "path+file:///repo/crates/agent_api#0.2.3",
        "path+file:///repo/crates/codex#0.2.3",
        "path+file:///repo/crates/claude_code#0.2.3",
        "path+file:///repo/crates/opencode#0.2.3",
        "path+file:///repo/crates/wrapper_events#0.2.3",
        "path+file:///repo/crates/xtask#0.2.3",
    ]
    packages = [
        package_fixture(
            package_id=workspace_members[0],
            name="unified-agent-api",
            version="0.2.3",
            manifest_path="/repo/crates/agent_api/Cargo.toml",
            dependencies=[
                dep_fixture("unified-agent-api-codex"),
                dep_fixture("unified-agent-api-claude-code"),
                dep_fixture("unified-agent-api-opencode"),
                dep_fixture("serde", internal=False),
            ],
        ),
        package_fixture(
            package_id=workspace_members[1],
            name="unified-agent-api-codex",
            version="0.2.3",
            manifest_path="/repo/crates/codex/Cargo.toml",
            dependencies=[dep_fixture("serde", internal=False)],
        ),
        package_fixture(
            package_id=workspace_members[2],
            name="unified-agent-api-claude-code",
            version="0.2.3",
            manifest_path="/repo/crates/claude_code/Cargo.toml",
            dependencies=[dep_fixture("serde", internal=False)],
        ),
        package_fixture(
            package_id=workspace_members[3],
            name="unified-agent-api-opencode",
            version="0.2.3",
            manifest_path="/repo/crates/opencode/Cargo.toml",
            dependencies=[dep_fixture("serde", internal=False)],
        ),
        package_fixture(
            package_id=workspace_members[4],
            name="unified-agent-api-wrapper-events",
            version="0.2.3",
            manifest_path="/repo/crates/wrapper_events/Cargo.toml",
            dependencies=[
                dep_fixture("unified-agent-api-codex"),
                dep_fixture("unified-agent-api-claude-code"),
                dep_fixture("unified-agent-api-opencode"),
                dep_fixture("serde", internal=False),
            ],
        ),
        package_fixture(
            package_id=workspace_members[5],
            name="xtask",
            version="0.2.3",
            manifest_path="/repo/crates/xtask/Cargo.toml",
            dependencies=[],
            publish=[],
        ),
    ]
    return {"workspace_members": workspace_members, "packages": packages}


class FakeRegistryClient:
    def __init__(
        self,
        *,
        existing_crates: set[str] | None = None,
        existing_versions: set[tuple[str, str]] | None = None,
    ) -> None:
        self.existing_crates = existing_crates or set()
        self.existing_versions = existing_versions or set()

    def crate_exists(self, crate: str) -> bool:
        return crate in self.existing_crates

    def crate_version_exists(self, crate: str, version: str) -> bool:
        return (crate, version) in self.existing_versions


class PublishPlannerTests(unittest.TestCase):
    def setUp(self) -> None:
        self.packages = workspace_publishable_packages_from_metadata(metadata_fixture())

    def test_topological_publish_order_uses_workspace_tie_breaker(self) -> None:
        ordered = topological_publish_order(self.packages)
        self.assertEqual(
            [package.name for package in ordered],
            [
                "unified-agent-api-codex",
                "unified-agent-api-claude-code",
                "unified-agent-api-opencode",
                "unified-agent-api-wrapper-events",
                "unified-agent-api",
            ],
        )

    def test_plan_marks_all_existing_versions_as_skip(self) -> None:
        registry = FakeRegistryClient(
            existing_crates={package.name for package in self.packages},
            existing_versions={(package.name, package.version) for package in self.packages},
        )
        planned = plan_publish_actions(
            self.packages,
            registry_client=registry,
            release_version="0.2.3",
        )
        self.assertEqual(
            [item.strategy for item in planned],
            [PublishStrategy.SKIP] * 5,
        )

    def test_plan_handles_new_leaf_crate(self) -> None:
        registry = FakeRegistryClient(
            existing_crates={
                "unified-agent-api-codex",
                "unified-agent-api-claude-code",
                "unified-agent-api-wrapper-events",
                "unified-agent-api",
            },
            existing_versions={
                ("unified-agent-api-codex", "0.2.3"),
                ("unified-agent-api-claude-code", "0.2.3"),
            },
        )
        planned = plan_publish_actions(
            self.packages,
            registry_client=registry,
            release_version="0.2.3",
        )
        strategies = {item.package.name: item.strategy for item in planned}
        self.assertEqual(
            strategies["unified-agent-api-opencode"],
            PublishStrategy.PUBLISH_WITH_BOOTSTRAP_TOKEN,
        )
        self.assertEqual(
            strategies["unified-agent-api-wrapper-events"],
            PublishStrategy.PUBLISH_WITH_OIDC,
        )
        self.assertEqual(
            strategies["unified-agent-api"],
            PublishStrategy.PUBLISH_WITH_OIDC,
        )

    def test_plan_handles_new_dependent_crate(self) -> None:
        registry = FakeRegistryClient(
            existing_crates={
                "unified-agent-api-codex",
                "unified-agent-api-claude-code",
                "unified-agent-api-opencode",
                "unified-agent-api-wrapper-events",
            },
            existing_versions={
                ("unified-agent-api-codex", "0.2.3"),
                ("unified-agent-api-claude-code", "0.2.3"),
                ("unified-agent-api-opencode", "0.2.3"),
                ("unified-agent-api-wrapper-events", "0.2.3"),
            },
        )
        planned = plan_publish_actions(
            self.packages,
            registry_client=registry,
            release_version="0.2.3",
        )
        strategies = {item.package.name: item.strategy for item in planned}
        self.assertEqual(
            strategies["unified-agent-api"],
            PublishStrategy.PUBLISH_WITH_BOOTSTRAP_TOKEN,
        )

    def test_plan_supports_partial_rerun(self) -> None:
        registry = FakeRegistryClient(
            existing_crates={package.name for package in self.packages if package.name != "unified-agent-api-opencode"},
            existing_versions={
                ("unified-agent-api-codex", "0.2.3"),
            },
        )
        planned = plan_publish_actions(
            self.packages,
            registry_client=registry,
            release_version="0.2.3",
        )
        self.assertEqual(planned[0].strategy, PublishStrategy.SKIP)
        self.assertEqual(planned[1].strategy, PublishStrategy.PUBLISH_WITH_OIDC)
        self.assertEqual(planned[2].strategy, PublishStrategy.PUBLISH_WITH_BOOTSTRAP_TOKEN)


class TokenSelectionTests(unittest.TestCase):
    def test_existing_crate_publish_uses_oidc_token(self) -> None:
        token = select_registry_token(
            PublishStrategy.PUBLISH_WITH_OIDC,
            {OIDC_TOKEN_ENV: "oidc-token"},
        )
        self.assertEqual(token, "oidc-token")

    def test_first_publish_uses_bootstrap_token(self) -> None:
        token = select_registry_token(
            PublishStrategy.PUBLISH_WITH_BOOTSTRAP_TOKEN,
            {BOOTSTRAP_TOKEN_ENV: "bootstrap-token"},
        )
        self.assertEqual(token, "bootstrap-token")

    def test_missing_bootstrap_token_fails_clearly(self) -> None:
        with self.assertRaisesRegex(RuntimeError, BOOTSTRAP_TOKEN_ENV):
            select_registry_token(PublishStrategy.PUBLISH_WITH_BOOTSTRAP_TOKEN, {})


if __name__ == "__main__":
    unittest.main()
