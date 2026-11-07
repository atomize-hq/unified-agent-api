//! Exercise the complete public ordinary planner/renderer; the synthetic subject has no backend.
use super::*;
use clap::Parser;

pub(super) fn create(root: &Path) {
    let temporary = tempfile::tempdir().unwrap();
    for path in [
        REGISTRY_RELATIVE_PATH,
        "Cargo.toml",
        "docs/crates-io-release.md",
        APPROVAL,
    ] {
        write(temporary.path(), path, fs::read(root.join(path)).unwrap());
    }
    let args = args();
    crate::onboard_agent::run_in_workspace(temporary.path(), args, &mut Vec::new()).unwrap();
    let inventory = format!("docs/agents/lifecycle/{PACK}/governance/written-paths.json");
    let paths: Vec<String> =
        serde_json::from_slice(&fs::read(temporary.path().join(inventory)).unwrap()).unwrap();
    assert!(paths.contains(&format!("{ROOT}/versions/.gitkeep")));
    assert!(paths.contains(&format!("{ROOT}/reports/.gitkeep")));
    for path in paths {
        let mut bytes = fs::read(temporary.path().join(&path)).unwrap();
        if path == REGISTRY_RELATIVE_PATH {
            // The only descriptor projection: the compiled non-runtime entry has no backend.
            bytes = String::from_utf8(bytes)
                .unwrap()
                .replace(
                    "backend_module = \"crates/agent_api/src/backends/depth_fixture\"\n",
                    "",
                )
                .into_bytes();
        }
        write(root, &path, bytes);
    }
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
