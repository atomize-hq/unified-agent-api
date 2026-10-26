//! The one bounded presence predicate. This is declaration interpretation, not argv parsing.
use crate::agent_registry::support_depth::ModeSelector;
use serde_json::Value;

pub(super) fn validate(
    agent: &str,
    mode: &ModeSelector,
    shard: &Value,
    union: &Value,
) -> Result<(), String> {
    if mode.predicates.is_empty() {
        return Ok(());
    }
    let predicate = &mode.predicates[0];
    if mode.predicates.len() != 1 || predicate.value.is_some() {
        return Err("only acquired --json presence is supported".into());
    }
    let path: Vec<_> = mode.command_path.split(' ').skip(1).collect();
    if mode.command_path.split(' ').next() != Some(agent) {
        return Err("foreign predicate command".into());
    }
    let command = |value: &Value| -> Result<Value, String> {
        value["commands"]
            .as_array()
            .ok_or("missing predicate commands")?
            .iter()
            .find(|c| c["path"] == serde_json::json!(path))
            .cloned()
            .ok_or_else(|| "predicate command is not acquired at the exact path".into())
    };
    let original = command(shard)?;
    let normalized = command(union)?;
    if normalized["conflicts"]
        .as_array()
        .into_iter()
        .flatten()
        .any(|conflict| conflict["unit"] == "flag" && conflict["key"] == "--json")
    {
        return Err("--json has an unresolved acquired alias/arity conflict".into());
    }
    let flags = |command: &Value| -> Result<Vec<Value>, String> {
        command["flags"]
            .as_array()
            .cloned()
            .ok_or_else(|| "predicate has no acquired flags".into())
    };
    let originals = flags(&original)?;
    let matching: Vec<_> = originals
        .iter()
        .filter(|f| {
            f["long"].as_str() == Some(&predicate.flag)
                || f["short"].as_str() == Some(&predicate.flag)
        })
        .collect();
    if matching.len() != 1 || matching[0]["long"] != "--json" {
        return Err("predicate is not the acquired canonical --json or an acquired alias".into());
    }
    let original = matching[0];
    let normalized = flags(&normalized)?;
    let matching: Vec<_> = normalized.iter().filter(|f| f["key"] == "--json").collect();
    if matching.len() != 1 {
        return Err("--json has no coherent normalized acquisition".into());
    }
    let normalized = matching[0];
    // A union conflict is not a license to choose an arity or alias regime from one target.
    for row in [original, normalized] {
        if row["takes_value"] != false || row.get("value_name").is_some() {
            return Err("--json predicate must have exact zero arity".into());
        }
    }
    for key in ["long", "short", "repeatable"] {
        if original.get(key) != normalized.get(key) {
            return Err("--json acquired alias/arity regime is incoherent".into());
        }
    }
    Ok(())
}
