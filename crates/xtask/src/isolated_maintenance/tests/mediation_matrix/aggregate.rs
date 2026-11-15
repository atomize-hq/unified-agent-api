//! Actual ON4 proposal gates introduce aggregate effects beyond the named approval scope.
use super::*;
use clap::Parser;

fn foreign_agent(root: &Path, _: &[String]) -> Result<(), String> {
    write(root, "cli_manifests/codex/min_supported.txt", "0.3.0\n");
    Ok(())
}
fn later_version(root: &Path, _: &[String]) -> Result<(), String> {
    write(
        root,
        "cli_manifests/depth_fixture/pointers/latest_supported/x86_64-unknown-linux-gnu.txt",
        "0.3.0\n",
    );
    Ok(())
}
fn other_target(root: &Path, _: &[String]) -> Result<(), String> {
    write(
        root,
        "cli_manifests/depth_fixture/pointers/latest_supported/aarch64-unknown-linux-musl.txt",
        "0.2.0\n",
    );
    Ok(())
}

pub(super) fn check(root: &Path) {
    type Gate = fn(&Path, &[String]) -> Result<(), String>;
    for (gate, expected) in [
        (foreign_agent as Gate, "onboarding cannot change pointer file cli_manifests/codex/min_supported.txt"),
        (later_version as Gate, "onboarding batch has acceptance outside its owned generation"),
        (other_target as Gate, "onboarding cannot change pointer file cli_manifests/depth_fixture/pointers/latest_supported/aarch64-unknown-linux-musl.txt"),
    ] {
        crate::publication_refresh::set_test_gate_runner(Some(gate));
        unchanged(root, expected, || {
            crate::publication_refresh::run_in_workspace(root, crate::publication_refresh::Args::parse_from([
                "refresh-publication", "--write", "--approval",
                "docs/agents/lifecycle/depth-fixture-cli-onboarding/governance/approved-agent.toml",
            ]), &mut Vec::new()).unwrap_err().to_string()
        });
    }
    crate::publication_refresh::set_test_gate_runner(None);
}
