//! Read the actual candidate. The declared-checkout leg authenticates in an independent,
//! test-owned library process, where the private fixture profile can be installed.
use std::{path::Path, process::Command};
use xtask::agent_registry::AgentRegistry;

pub fn registry(root: &Path) -> AgentRegistry {
    let Some(reader) = std::env::var_os("S10_DECLARED_REGISTRY_READER") else {
        return xtask::isolated_maintenance::registry(root)
            .expect("authenticated candidate registry");
    };
    let raw = read(&reader, root, "registry");
    // Admission happened in the independently routed child. Validate those exact returned bytes.
    AgentRegistry::parse_unenforced(&raw).expect("validate authenticated registry bytes")
}

#[allow(dead_code)]
pub fn capability_markdown(root: &Path) -> String {
    match std::env::var_os("S10_DECLARED_REGISTRY_READER") {
        Some(reader) => read(&reader, root, "capability"),
        None => xtask::capability_matrix::generate_markdown_for_workspace(root)
            .expect("actual candidate capability matrix"),
    }
}

#[allow(dead_code)]
pub fn validate_claude_lifecycle(root: &Path) {
    match std::env::var_os("S10_DECLARED_REGISTRY_READER") {
        Some(reader) => assert_eq!(read(&reader, root, "claude-lifecycle"), "validated"),
        None => {
            let registry = registry(root);
            let entry = registry.find("claude_code").unwrap();
            xtask::agent_lifecycle::load_lifecycle_state(
                root,
                &xtask::agent_lifecycle::lifecycle_state_path_for_entry(entry),
            )
            .unwrap();
        }
    }
}

#[allow(dead_code)]
pub fn check_runtime_advertising(root: &Path) {
    match std::env::var_os("S10_DECLARED_REGISTRY_READER") {
        Some(reader) => assert_eq!(read(&reader, root, "runtime-check"), "validated"),
        None => xtask::runtime_advertising::run_in_workspace(
            root,
            xtask::runtime_advertising::Args { check: true },
        )
        .unwrap(),
    }
}

#[allow(dead_code)]
pub fn runtime_support_data(root: &Path) -> String {
    match std::env::var_os("S10_DECLARED_REGISTRY_READER") {
        Some(reader) => read(&reader, root, "runtime-support-data"),
        None => xtask::support_matrix::render_agent_api_runtime_support_data(root).unwrap(),
    }
}

fn read(reader: &std::ffi::OsStr, root: &Path, operation: &str) -> String {
    let output = Command::new(reader)
        .args([
            "--exact",
            "isolated_maintenance::tests::declared_test_leg::authenticated_registry_child",
            "--ignored",
            "--nocapture",
        ])
        .env("S10_DECLARED_REGISTRY_ROOT", root)
        .env("S10_DECLARED_REGISTRY_OPERATION", operation)
        .output()
        .expect("launch independent test-owned registry reader");
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    let authenticated = stdout
        .lines()
        .find_map(|line| line.strip_prefix("S10_AUTHENTICATED_RESULT="))
        .expect("authenticated reader returned registry bytes");
    String::from_utf8(hex::decode(authenticated).unwrap()).unwrap()
}
