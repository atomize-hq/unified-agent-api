//! Reconcile frozen acquisition bytes using the union engine, with no raw-help filesystem reads.
use super::{
    assert_supported_globals, assert_supported_sorting, build_union_commands,
    normalize_union_commands, RulesFile, SnapshotUnionV2, UnionInputV2,
};
use crate::manifest_snapshot_schema::SnapshotV1;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn validate(
    rules: &[u8],
    version: &str,
    union: &Value,
    shards: &BTreeMap<String, Value>,
    selected_paths: &BTreeSet<Vec<String>>,
) -> Result<(), String> {
    let rules: RulesFile = serde_json::from_slice(rules).map_err(|e| e.to_string())?;
    assert_supported_sorting(&rules.sorting).map_err(|e| e.to_string())?;
    assert_supported_globals(&rules.globals).map_err(|e| e.to_string())?;
    let targets = &rules.union.expected_targets;
    let mut snapshots = BTreeMap::new();
    for (target, value) in shards {
        let snapshot: SnapshotV1 =
            serde_json::from_value(value.clone()).map_err(|e| e.to_string())?;
        if snapshot.snapshot_schema_version != 1
            || snapshot.tool != rules.union.tool_name
            || snapshot.binary.semantic_version.as_deref() != Some(version)
            || snapshot.binary.target_triple != *target
            || !targets.contains(target)
        {
            return Err(format!("snapshot identity/schema mismatch: {target}"));
        }
        validate_units(
            &snapshot,
            rules.globals.effective_flags_model.enabled,
            selected_paths,
        )?;
        snapshots.insert(target.clone(), snapshot);
    }
    if !snapshots.contains_key(&rules.union.required_target) {
        return Err("required target snapshot is missing".into());
    }
    let present: Vec<_> = targets
        .iter()
        .filter(|t| snapshots.contains_key(*t))
        .cloned()
        .collect();
    let missing: Vec<_> = targets
        .iter()
        .filter(|t| !snapshots.contains_key(*t))
        .cloned()
        .collect();
    let inputs = present
        .iter()
        .map(|target| {
            let s = &snapshots[target];
            UnionInputV2 {
                target_triple: target.clone(),
                collected_at: Some(s.collected_at.clone()),
                binary: s.binary.clone(),
                features: s.features.clone(),
                known_omissions: s.known_omissions.clone(),
            }
        })
        .collect();
    let mut commands = build_union_commands(
        targets,
        &rules.union.required_target,
        version,
        None,
        rules.union.raw_help_layout,
        &present,
        &snapshots,
    );
    normalize_union_commands(&mut commands, &rules.globals.effective_flags_model);
    let expected = SnapshotUnionV2 {
        snapshot_schema_version: 2,
        tool: rules.union.tool_name,
        mode: "union".into(),
        collected_at: union["collected_at"]
            .as_str()
            .ok_or("missing union collected_at")?
            .into(),
        expected_targets: targets.clone(),
        complete: missing.is_empty(),
        missing_targets: (!missing.is_empty()).then_some(missing),
        inputs,
        commands,
    };
    let mut actual = union.clone();
    // Optional raw-help provenance is not consumed for O. Conflict facts themselves must match
    // reconstructed snapshots; help strings never certify effects or classification.
    if let Some(commands) = actual["commands"].as_array_mut() {
        for command in commands {
            if let Some(conflicts) = command.get_mut("conflicts").and_then(Value::as_array_mut) {
                for conflict in conflicts {
                    conflict
                        .as_object_mut()
                        .ok_or("invalid conflict")?
                        .remove("evidence");
                }
            }
        }
    }
    if actual != serde_json::to_value(expected).map_err(|e| e.to_string())? {
        return Err("union does not match its per-target snapshots and current RULES".into());
    }
    Ok(())
}

// Duplicate paths/names and ambiguous flag aliases cannot be hidden by the merger's maps.
fn validate_units(
    snapshot: &SnapshotV1,
    globals: bool,
    selected_paths: &BTreeSet<Vec<String>>,
) -> Result<(), String> {
    let aliases = |command: &crate::manifest_snapshot_schema::CommandSnapshot| -> Result<BTreeMap<String,String>,String> {
        let mut result=BTreeMap::new();
        let mut keys=BTreeSet::new();
        for flag in command.flags.iter().flatten() {
            let key=flag.long.as_ref().or(flag.short.as_ref()).ok_or("flag has no name")?;
            if !keys.insert(key) { return Err("duplicate snapshot flag".into()); }
            for alias in flag.long.iter().chain(&flag.short) {
                if result.insert(alias.clone(),key.clone()).is_some() { return Err("ambiguous snapshot flag alias".into()); }
            }
        }
        Ok(result)
    };
    let root = snapshot
        .commands
        .iter()
        .find(|c| c.path.is_empty())
        .map(aliases)
        .transpose()?
        .unwrap_or_default();
    let mut paths = BTreeSet::new();
    for command in &snapshot.commands {
        if command
            .path
            .iter()
            .any(|p| p.is_empty() || p.chars().any(char::is_whitespace))
        {
            return Err("command paths must have unambiguous tokens".into());
        }
        if !paths.insert(&command.path) {
            return Err("duplicate snapshot command path".into());
        }
        let current = aliases(command)?;
        // A command may legally own a local alias that another command uses globally. Only
        // the exact selected invocation scopes need inheritance interpretation; never apply a
        // parent-prefix assumption to an unrelated child command.
        if globals
            && selected_paths.contains(&command.path)
            && !command.path.is_empty()
            && current
                .iter()
                .any(|(alias, key)| root.get(alias).is_some_and(|r| r != key))
        {
            return Err("command flag alias conflicts with an inherited global flag".into());
        }
        let mut names = BTreeSet::new();
        if command
            .args
            .iter()
            .flatten()
            .any(|a| !names.insert(&a.name))
        {
            return Err("duplicate snapshot argument".into());
        }
    }
    Ok(())
}
