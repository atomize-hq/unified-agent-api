//! The capability matrix's backend set derives from the agent registry: the generator reads only
//! the inputs each registry entry names, and an enrolled agent outside the compiled built-in
//! inventory gets its own column without any code change.

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use xtask::{
    agent_lifecycle::lifecycle_state_path_for_entry,
    agent_registry::{AgentRegistry, BUILT_IN_AGENTS, REGISTRY_RELATIVE_PATH},
    capability_matrix::DEFAULT_OUT_PATH,
};

const NEW_AGENT: &str = "zeta_cli";
const TEMPLATE_AGENT: &str = "aider";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
}

fn write(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

/// A workspace holding the registry and, for every entry, exactly the files the capability
/// matrix reads for it: the root's `current.json` and the lifecycle state, where committed.
fn seed(dir: &Path, registry_text: &str) {
    write(&dir.join("Cargo.toml"), "[workspace]\nmembers = []\n");
    write(&dir.join(REGISTRY_RELATIVE_PATH), registry_text);
    let registry = AgentRegistry::parse(registry_text).unwrap();
    for entry in &registry.agents {
        let lifecycle = lifecycle_state_path_for_entry(entry);
        let manifest = format!("{}/current.json", entry.manifest_root);
        for relative in [lifecycle, manifest] {
            let source = repo_root().join(&relative);
            if source.is_file() {
                write(&dir.join(&relative), &read(&source));
            }
        }
    }
}

fn generate(dir: &Path) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .arg("capability-matrix")
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "capability-matrix failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    read(&dir.join(DEFAULT_OUT_PATH))
}

fn header_backends(markdown: &str) -> Vec<String> {
    let header = markdown
        .lines()
        .find(|line| line.starts_with("| capability id |"))
        .expect("a capability table");
    header
        .split('|')
        .map(str::trim)
        .filter(|cell| cell.starts_with('`'))
        .map(|cell| cell.trim_matches('`').to_string())
        .collect()
}

#[test]
fn registry_inputs_alone_reproduce_the_checked_in_matrix() {
    let temp = tempfile::tempdir().unwrap();
    let registry = read(&repo_root().join(REGISTRY_RELATIVE_PATH));
    seed(temp.path(), &registry);
    let generated = generate(temp.path());
    assert_eq!(
        generated,
        read(&repo_root().join(DEFAULT_OUT_PATH)),
        "the checked-in capability matrix must be exactly what the registry's inputs generate"
    );
    let parsed = AgentRegistry::parse(&registry).unwrap();
    let mut enrolled: Vec<String> = parsed
        .capability_matrix_entries()
        .map(|entry| entry.agent_id.clone())
        .collect();
    enrolled.sort();
    assert_eq!(header_backends(&generated), enrolled);
}

#[test]
fn an_enrolled_agent_outside_the_built_in_inventory_gets_a_column() {
    assert!(!BUILT_IN_AGENTS.contains(&NEW_AGENT));
    let canonical = read(&repo_root().join(REGISTRY_RELATIVE_PATH));
    let template = canonical
        .split("[[agents]]")
        .find(|block| block.contains(&format!("agent_id = \"{TEMPLATE_AGENT}\"")))
        .expect("template entry");
    let entry = format!("[[agents]]{}", template.replace(TEMPLATE_AGENT, NEW_AGENT));
    let registry = format!("{canonical}\n{entry}");

    let temp = tempfile::tempdir().unwrap();
    seed(temp.path(), &registry);
    let parsed = AgentRegistry::parse(&registry).unwrap();
    let new_entry = parsed.find(NEW_AGENT).unwrap();
    let template_entry = parsed.find(TEMPLATE_AGENT).unwrap();
    for (from, to) in [
        (
            lifecycle_state_path_for_entry(template_entry),
            lifecycle_state_path_for_entry(new_entry),
        ),
        (
            format!("{}/current.json", template_entry.manifest_root),
            format!("{}/current.json", new_entry.manifest_root),
        ),
    ] {
        let mut json: serde_json::Value =
            serde_json::from_str(&read(&repo_root().join(from))).unwrap();
        // The runtime template lineage names a compiled template, not this agent, so it keeps
        // the template's id; every other mention of the template agent is renamed.
        let summary = json.get("implementation_summary").cloned();
        json = serde_json::from_str(&json.to_string().replace(TEMPLATE_AGENT, NEW_AGENT)).unwrap();
        if let Some(summary) = summary {
            json["implementation_summary"] = summary;
        }
        write(&temp.path().join(to), &json.to_string());
    }

    let generated = generate(temp.path());
    let backends = header_backends(&generated);
    assert!(backends.contains(&NEW_AGENT.to_string()), "{backends:?}");
    let column = backends.iter().position(|id| id == NEW_AGENT).unwrap() + 2;
    for capability in &new_entry.capability_declaration.always_on {
        let row = generated
            .lines()
            .find(|line| line.starts_with(&format!("| `{capability}` |")))
            .unwrap_or_else(|| panic!("no row for {capability}"));
        assert_eq!(
            row.split('|').nth(column).map(str::trim),
            Some("✅"),
            "{row}"
        );
    }
}
