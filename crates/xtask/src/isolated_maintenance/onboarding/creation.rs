//! Re-render ON1 from authenticated parent inputs, never a candidate-supplied inventory.
use super::*;
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
    crate::onboard_agent::routing::write_scaffold(root, &adapter.scaffold(before)?)
        .map_err(|e| e.to_string())?;
    let rendered = Source::workspace(root)?;
    let inventory = format!(
        "docs/agents/lifecycle/{}/governance/written-paths.json",
        adapter.scaffold.scaffold.onboarding_pack_prefix
    );
    let paths: BTreeSet<String> =
        serde_json::from_str(&rendered.read(&inventory)?).map_err(|e| e.to_string())?;
    for path in &paths {
        let expected = rendered.read(path)?;
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
