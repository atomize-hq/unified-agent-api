//! Fixed ordinary registry for production-parser and ordinary behavior controls.
//! Candidate invariants must read their actual workspace through the authenticated route.
pub const fn undeclared_registry() -> &'static str {
    include_str!("../fixtures/undeclared_registry.toml")
}

/// One ordinary capability workspace per test process, shared by immutable readers.
#[allow(dead_code)]
pub fn ordinary_workspace() -> &'static std::path::Path {
    static WORKSPACE: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();
    WORKSPACE
        .get_or_init(|| {
            let directory = tempfile::tempdir().unwrap();
            let root = directory.path();
            let registry_path = root.join("crates/xtask/data/agent_registry.toml");
            std::fs::create_dir_all(registry_path.parent().unwrap()).unwrap();
            std::fs::write(registry_path, undeclared_registry()).unwrap();
            std::fs::write(root.join("Cargo.toml"), "[workspace]\nmembers = []\n").unwrap();
            let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
            let registry =
                xtask::agent_registry::AgentRegistry::parse(undeclared_registry()).unwrap();
            for entry in &registry.agents {
                let relative = format!("{}/current.json", entry.manifest_root);
                let destination = root.join(&relative);
                std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
                std::fs::copy(source.join(relative), destination).unwrap();
            }
            let states: Vec<serde_json::Value> =
                serde_json::from_str(include_str!("../fixtures/undeclared_lifecycle_states.json"))
                    .unwrap();
            for state in states {
                let prefix = state["onboarding_pack_prefix"].as_str().unwrap();
                let destination = root.join(xtask::agent_lifecycle::lifecycle_state_path(prefix));
                std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
                std::fs::write(destination, serde_json::to_vec_pretty(&state).unwrap()).unwrap();
            }
            directory
        })
        .path()
}

/// Execute a control whose production entry resolves its workspace from the process cwd.
/// A child isolates cwd from concurrently running tests and keeps every original assertion.
#[allow(dead_code)]
pub fn run_in_ordinary_workspace(test: &str) -> bool {
    if std::env::var("S10_ORDINARY_TEST").as_deref() == Ok(test) {
        return false;
    }
    #[cfg(target_os = "linux")]
    let executable = std::path::PathBuf::from("/proc/self/exe");
    #[cfg(not(target_os = "linux"))]
    let executable = std::env::current_exe().unwrap();
    let output = std::process::Command::new(executable)
        .args(["--exact", test, "--nocapture"])
        .env("S10_ORDINARY_TEST", test)
        .current_dir(ordinary_workspace())
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("1 passed;"),
        "{output:?}"
    );
    true
}

#[allow(dead_code)]
pub fn ordinary_lifecycle_state(agent_id: &str) -> xtask::agent_lifecycle::LifecycleState {
    let states: Vec<xtask::agent_lifecycle::LifecycleState> =
        serde_json::from_str(include_str!("../fixtures/undeclared_lifecycle_states.json")).unwrap();
    states
        .into_iter()
        .find(|state| state.agent_id == agent_id)
        .expect("ordinary lifecycle fixture")
}
