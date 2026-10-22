use super::git;
use crate::agent_registry::support_depth::SupportDepth;
use serde_json::{json, Value};
use std::{fs, path::Path};
use tempfile::TempDir;

pub fn write(root: &Path, path: &str, bytes: impl AsRef<[u8]>) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}

pub fn workspace(agent: &str) -> TempDir {
    let root = tempfile::tempdir().unwrap();
    git(root.path(), &["init", "-q"]);
    let mut registry: toml_edit::DocumentMut = include_str!("../../../../data/agent_registry.toml")
        .parse()
        .unwrap();
    let rules = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../../cli_manifests/{agent}/RULES.json")),
    )
    .unwrap();
    let rules_json: Value = serde_json::from_str(&rules).unwrap();
    let target = &rules_json["union"]["required_target"];
    let table: SupportDepth = serde_json::from_value(json!({"enabled_paths":["maintenance"],"enrollments":[{
        "lifecycle_path":"maintenance","version":"1.2.3","targets":[target],
        "modes":[{"selector":"run","command_path":format!("{agent} run"),"mode":"headless","predicates":[]}],
        "operations":[{"id":"run","serves":["run"],"templates":["CORE"],"shared_mapping":"wrapper_only","forwarding":false,
        "promise":{"r1":"r","v1":"v","g1":"g","e1":"e","a1":"a","x1":"x","x2":"x","x3":"x","x4":"x"}}]}]})).unwrap();
    let entry = registry["agents"]
        .as_array_of_tables_mut()
        .unwrap()
        .iter_mut()
        .find(|e| e["agent_id"].as_str() == Some(agent))
        .unwrap();
    entry.insert(
        "support_depth",
        toml_edit::Item::Table(
            toml_edit::ser::to_document(&table)
                .unwrap()
                .as_table()
                .clone(),
        ),
    );
    write(
        root.path(),
        "crates/xtask/data/agent_registry.toml",
        registry.to_string(),
    );
    write(
        root.path(),
        &format!("cli_manifests/{agent}/RULES.json"),
        rules,
    );
    write(
        root.path(),
        "docs/specs/unified-agent-api/non-tui-support-debt.md",
        "### `support-debt-authorization-contract-target-version-v1`\n",
    );
    git(root.path(), &["add", "."]);
    root
}

pub fn acquire(root: &Path, agent: &str) {
    let manifest = root.join(format!("cli_manifests/{agent}"));
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    fs::copy(
        repo.join(format!("cli_manifests/{agent}/SCHEMA.json")),
        manifest.join("SCHEMA.json"),
    )
    .unwrap();
    let rules: Value =
        serde_json::from_slice(&fs::read(manifest.join("RULES.json")).unwrap()).unwrap();
    let target = rules["union"]["required_target"].as_str().unwrap();
    let shard = json!({"snapshot_schema_version":1,"tool":rules["union"]["tool_name"],
        "collected_at":"2026-10-22T00:00:00Z","binary":{"sha256":"a".repeat(64),"size_bytes":1,
        "platform":{"os":"linux","arch":"x86_64"},"target_triple":target,"version_output":"1.2.3","semantic_version":"1.2.3"},
        "commands":[{"path":[],"flags":[{"long":"--global","short":"-g","takes_value":false}]},
        {"path":["run"],"usage":"run [--output FILE] PROMPT","args":[{"name":"PROMPT","required":true,"variadic":false}],
        "flags":[{"long":"--output","short":"-o","takes_value":true,"value_name":"FILE"}]}]});
    write(
        root,
        &format!("cli_manifests/{agent}/snapshots/1.2.3/{target}.json"),
        serde_json::to_vec_pretty(&shard).unwrap(),
    );
    let wrapper = json!({"schema_version":1,"coverage":[{"path":["run"],"level":"passthrough","note":"bounded forwarding fixture","flags":[
        {"key":"--claimed","level":"explicit"},{"key":"--output","level":"passthrough","note":"bounded forwarding fixture"}],"args":[{"name":"PROMPT","level":"unsupported"}]}]});
    write(
        root,
        &format!("cli_manifests/{agent}/wrapper_coverage.json"),
        wrapper.to_string(),
    );
    regenerate(root, agent);
}

pub fn regenerate(root: &Path, agent: &str) {
    let manifest = root.join(format!("cli_manifests/{agent}"));
    for subcommand in ["manifest-union", "manifest-report"] {
        if subcommand == "manifest-union" {
            crate::manifest_union::run(crate::manifest_union::Args {
                root: Some(manifest.clone()),
                rules: None,
                version: "1.2.3".into(),
            })
            .unwrap();
        } else {
            crate::manifest_report::run(crate::manifest_report::Args {
                root: Some(manifest.clone()),
                rules: None,
                version: "1.2.3".into(),
            })
            .unwrap();
        }
    }
    git(root, &["add", "."]);
}
