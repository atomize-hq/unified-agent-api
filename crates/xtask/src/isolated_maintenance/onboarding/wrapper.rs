//! Check wrapper claims independently of acquisition/report availability using shared intake.
use super::*;
use std::collections::BTreeMap;

pub(super) fn validate(source: &Source, adapter: &Adapter) -> Result<(), String> {
    let path = format!("{}/wrapper_coverage.json", adapter.scaffold.manifest_root);
    let Some(raw) = source.optional(&path)? else {
        return Ok(());
    };
    let wrapper: Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    // Host placeholders are irrelevant to wrapper semantics; this compiled profile template
    // is also available during scope loading, before any repository-bound adapter exists.
    let rules = adapter.wrapper_rules;
    // This empty upstream is private validation context, never retained acquisition evidence.
    // Expected targets/platforms come solely from the compiled governing profile. Shared intake
    // validates all wrapper units, including claims absent from any observed upstream surface.
    let union = serde_json::json!({
        "snapshot_schema_version":2,"mode":"union","complete":true,
        "expected_targets":adapter.scaffold.canonical_targets,
        "inputs":adapter.scaffold.canonical_targets.iter().map(|t| serde_json::json!({"target_triple":t})).collect::<Vec<_>>(),
        "commands":[]
    });
    let temporary = tempfile::tempdir().map_err(|e| e.to_string())?;
    let root = temporary.path();
    let snapshots = root.join("snapshots").join(adapter.version);
    fs::create_dir_all(&snapshots).map_err(|e| e.to_string())?;
    fs::write(root.join("RULES.json"), rules).map_err(|e| e.to_string())?;
    fs::write(root.join("wrapper_coverage.json"), &raw).map_err(|e| e.to_string())?;
    fs::write(snapshots.join("union.json"), union.to_string()).map_err(|e| e.to_string())?;
    let args = crate::manifest_report::Args {
        root: Some(root.to_owned()),
        rules: None,
        version: adapter.version.into(),
    };
    #[cfg(test)]
    let generated = crate::manifest_report::run_at(args, "1970-01-01T00:00:00Z");
    #[cfg(not(test))]
    let generated = crate::manifest_report::run(args);
    generated.map_err(|e| e.to_string())?;
    let mut reports = BTreeMap::new();
    for entry in
        fs::read_dir(root.join("reports").join(adapter.version)).map_err(|e| e.to_string())?
    {
        let entry = entry.map_err(|e| e.to_string())?;
        reports.insert(
            entry
                .file_name()
                .into_string()
                .map_err(|_| "invalid report filename")?,
            serde_json::from_slice(&fs::read(entry.path()).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?,
        );
    }
    crate::manifest_report::intake::validate(
        &adapter.scaffold.agent_id,
        rules.as_bytes(),
        &union,
        &wrapper,
        adapter.version,
        &reports,
    )?;
    Ok(())
}
