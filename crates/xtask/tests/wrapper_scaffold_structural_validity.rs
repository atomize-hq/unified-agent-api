use std::{fs, process::Command};

#[path = "support/onboard_agent_harness.rs"]
mod harness;

use harness::{
    nested_scaffold_fixture_root, run_xtask, wrapper_scaffold_args, NESTED_GEMINI_CRATE_PATH,
};

#[test]
fn scaffold_wrapper_crate_nested_registry_path_passes_targeted_cargo_check() {
    let fixture = nested_scaffold_fixture_root("wrapper-scaffold-structural-validity");

    let scaffold = run_xtask(&fixture, wrapper_scaffold_args("--write", "gemini_cli"));
    assert_eq!(scaffold.exit_code, 0, "stderr:\n{}", scaffold.stderr);

    let manifest = fs::read_to_string(fixture.join(NESTED_GEMINI_CRATE_PATH).join("Cargo.toml"))
        .expect("read manifest");
    assert!(manifest.contains("version.workspace = true"));
    assert!(manifest.contains("edition = \"2021\""));
    assert!(manifest.contains("rust-version = \"1.78\""));
    assert!(manifest.contains("license = \"MIT OR Apache-2.0\""));
    assert!(manifest.contains("repository = \"https://github.com/atomize-hq/unified-agent-api\""));
    assert!(manifest.contains("homepage = \"https://github.com/atomize-hq/unified-agent-api\""));
    assert!(manifest.contains("readme = \"README.md\""));

    let output = Command::new("cargo")
        .current_dir(&fixture)
        .args(["check", "-p", "unified-agent-api-gemini-cli"])
        .output()
        .expect("run cargo check");

    assert!(
        output.status.success(),
        "cargo check failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
