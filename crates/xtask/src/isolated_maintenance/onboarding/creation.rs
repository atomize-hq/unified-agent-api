//! Re-render ON1 from authenticated parent inputs, never a candidate-supplied inventory.
use super::*;
use clap::Parser;
use std::collections::BTreeSet;

pub(super) fn validate(
    workspace: &Path,
    parent: &str,
    before: &Source,
    after: &Source,
    adapter: &Adapter,
) -> Result<BTreeSet<String>, String> {
    // Ordinary ON1 must see every pinned parent path, including non-workspace files and
    // all inputs its planner reads. A selected-input copy silently drops preconditions.
    let parent_view = view::View::pinned(workspace, parent)?;
    let root = parent_view.root();
    let entry = &adapter.scaffold;
    // Public approval loading/routing is slice 2. Reuse today's ordinary raw-descriptor
    // renderer here, then project only its backend field onto the compiled non-runtime entry.
    let backend = format!("crates/agent_api/src/backends/{}", entry.agent_id);
    let args = crate::onboard_agent::Args::try_parse_from([
        "onboard-agent",
        "--write",
        "--agent-id",
        &entry.agent_id,
        "--display-name",
        &entry.display_name,
        "--crate-path",
        &entry.crate_path,
        "--backend-module",
        &backend,
        "--manifest-root",
        &entry.manifest_root,
        "--package-name",
        &entry.package_name,
        "--canonical-target",
        adapter.subject.target,
        "--wrapper-coverage-binding-kind",
        &entry.wrapper_coverage.binding_kind,
        "--wrapper-coverage-source-path",
        &entry.wrapper_coverage.source_path,
        "--support-matrix-enabled",
        if entry.publication.support_matrix_enabled {
            "true"
        } else {
            "false"
        },
        "--capability-matrix-enabled",
        "false",
        "--docs-release-track",
        &entry.release.docs_release_track,
        "--onboarding-pack-prefix",
        &entry.scaffold.onboarding_pack_prefix,
    ])
    .map_err(|e| e.to_string())?;
    crate::onboard_agent::run_in_workspace(root, args, &mut Vec::new())
        .map_err(|e| e.to_string())?;
    let rendered = Source::workspace(root)?;
    let inventory = format!(
        "docs/agents/lifecycle/{}/governance/written-paths.json",
        entry.scaffold.onboarding_pack_prefix
    );
    let paths: BTreeSet<String> =
        serde_json::from_str(&rendered.read(&inventory)?).map_err(|e| e.to_string())?;
    for path in &paths {
        let mut expected = rendered.read(path)?;
        if path == REGISTRY_RELATIVE_PATH {
            let mut registry: toml_edit::DocumentMut = expected
                .parse::<toml_edit::DocumentMut>()
                .map_err(|e| e.to_string())?;
            let entry = registry["agents"]
                .as_array_of_tables_mut()
                .ok_or("invalid ON1 registry")?
                .iter_mut()
                .find(|e| e["agent_id"].as_str() == Some(adapter.scaffold.agent_id.as_str()))
                .ok_or("missing ON1 entry")?;
            entry.remove("backend_module");
            expected = registry.to_string();
        }
        if ![
            REGISTRY_RELATIVE_PATH,
            "Cargo.toml",
            "docs/crates-io-release.md",
        ]
        .contains(&path.as_str())
            && before
                .optional_bytes(path)?
                .is_some_and(|bytes| bytes != expected.as_bytes())
        {
            return Err(format!(
                "ON1 creation would overwrite a divergent scaffold: {path}"
            ));
        }
        if after.optional_bytes(path)?.as_deref() != Some(expected.as_bytes())
            || after.executable(path)?
        {
            return Err(format!(
                "ON1 creation differs from ordinary rendered scaffold: {path}"
            ));
        }
    }
    Ok(paths)
}
