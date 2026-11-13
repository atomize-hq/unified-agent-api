//! Shared bootstrap planner for fixture setup; public ON1 is exercised in `public_on1`.
use super::*;
use clap::Parser;

pub(super) fn create(root: &Path) {
    let scaffold = crate::isolated_maintenance::onboarding::Scaffold {
        entry: AgentRegistry::parse_unenforced(SCAFFOLD)
            .unwrap()
            .agents
            .remove(0),
        approval_path: APPROVAL.into(),
        approval_bytes: fs::read(root.join(APPROVAL)).unwrap(),
    };
    crate::onboard_agent::routing::write_scaffold(root, &scaffold).unwrap();
}

pub(super) fn args() -> crate::onboard_agent::Args {
    crate::onboard_agent::Args::parse_from([
        "onboard-agent",
        "--write",
        "--agent-id",
        "depth_fixture",
        "--display-name",
        "Depth Fixture",
        "--crate-path",
        "crates/depth_fixture",
        "--backend-module",
        "crates/agent_api/src/backends/depth_fixture",
        "--manifest-root",
        ROOT,
        "--package-name",
        "unified-agent-api-depth-fixture",
        "--canonical-target",
        "x86_64-unknown-linux-gnu",
        "--wrapper-coverage-binding-kind",
        "generated_from_wrapper_crate",
        "--wrapper-coverage-source-path",
        "crates/depth_fixture",
        "--support-matrix-enabled",
        "true",
        "--capability-matrix-enabled",
        "false",
        "--docs-release-track",
        "unpublished",
        "--onboarding-pack-prefix",
        PACK,
    ])
}
