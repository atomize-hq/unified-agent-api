//! Finite semantic facts from the already validated acquisition; never a second policy store.
use super::{acquisition::Surfaces, canonical_set};
use crate::{
    agent_maintenance::support_audit::{surface_identity, ReportRowShape, SurfaceIdentity},
    depth_record::part_identity,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct ClassificationBasis {
    pub operation: String,
    pub sha256: String,
    /// Observable facts for reviewing a registry row set. This is diagnostic, not freeze authority.
    pub facts: Value,
}

pub(super) struct AcquisitionFacts<'a> {
    pub agent: &'a str,
    pub version: &'a str,
    pub union: &'a Value,
    pub shards: &'a BTreeMap<String, Value>,
    pub wrapper: &'a Value,
    pub surfaces: &'a Surfaces,
    pub rules_sha256: &'a str,
    pub schema_sha256: &'a str,
}

impl AcquisitionFacts<'_> {
    pub fn basis(
        &self,
        operation: &str,
        target: &str,
        owned: &[SurfaceIdentity],
    ) -> Result<ClassificationBasis, String> {
        let shard = &self.shards[target];
        let original = rows(self.agent, shard, "commands", false)?;
        let union = rows(self.agent, self.union, "commands", false)?;
        let claims = rows(self.agent, self.wrapper, "coverage", true)?;
        let units = owned
            .iter()
            .map(|surface| {
                json!({
                    "surface": surface,
                    "applicable_targets": self.surfaces[surface],
                    "original_shard": original.get(surface),
                    "normalized_union": union.get(surface),
                    "wrapper_claims": claims.get(surface),
                })
            })
            .collect();
        let binary = &shard["binary"];
        let mut facts = json!({
            "agent": self.agent, "version": self.version, "target": target,
            "binary": {"semantic_version": binary["semantic_version"], "target_triple": binary["target_triple"], "sha256": binary["sha256"]},
            "rules_sha256": self.rules_sha256, "schema_sha256": self.schema_sha256,
            "units": canonical_set::<Value>(units)?,
        });
        // Presence is itself a fact: absent is distinct from an explicitly empty regime.
        for key in ["features", "known_omissions"] {
            if let Some(value) = shard.get(key) {
                facts[key] = value.clone();
            }
        }
        Ok(ClassificationBasis {
            operation: operation.into(),
            sha256: part_identity(&facts)?,
            facts,
        })
    }
}

fn rows(
    agent: &str,
    value: &Value,
    member: &str,
    wrapper: bool,
) -> Result<BTreeMap<SurfaceIdentity, Vec<Value>>, String> {
    let mut result = BTreeMap::<_, Vec<_>>::new();
    for command in value[member]
        .as_array()
        .ok_or("missing semantic commands")?
    {
        let path: Vec<String> =
            serde_json::from_value(command["path"].clone()).map_err(|e| e.to_string())?;
        let mut add = |row: &Value, shape: ReportRowShape, leaf: &str| {
            let payload = if wrapper {
                // Child claim scope is intersected with the owning command scope by validation.
                json!({"row": row, "owning_command": command})
            } else {
                row.clone()
            };
            result
                .entry(surface_identity(agent, &path, shape, leaf))
                .or_default()
                .push(payload);
        };
        add(command, ReportRowShape::Command, "");
        for flag in command["flags"].as_array().into_iter().flatten() {
            let key = flag
                .get("key")
                .or_else(|| flag.get("long"))
                .or_else(|| flag.get("short"))
                .and_then(Value::as_str)
                .ok_or("semantic flag lacks identity")?;
            add(flag, ReportRowShape::Flag, key);
        }
        for arg in command["args"].as_array().into_iter().flatten() {
            add(
                arg,
                ReportRowShape::Arg,
                arg["name"].as_str().ok_or("semantic arg lacks identity")?,
            );
        }
    }
    for rows in result.values_mut() {
        *rows = canonical_set(std::mem::take(rows))?;
    }
    Ok(result)
}

pub(super) fn supported_shard(value: &Value) -> Result<(), String> {
    let typed: crate::manifest_snapshot_schema::SnapshotV1 =
        serde_json::from_value(value.clone()).map_err(|e| e.to_string())?;
    if serde_json::to_value(typed).map_err(|e| e.to_string())? != *value {
        return Err("unknown or unprojectable acquisition snapshot field".into());
    }
    Ok(())
}

fn fields(value: &Value, supported: &[&str]) -> Result<(), String> {
    if value
        .as_object()
        .ok_or("semantic row must be an object")?
        .keys()
        .any(|key| !supported.contains(&key.as_str()))
    {
        return Err("unknown or unprojectable acquisition row field".into());
    }
    if let Some(scope) = value.get("scope") {
        fields(scope, &["platforms", "target_triples"])?;
    }
    Ok(())
}
pub(super) fn supported_wrapper(value: &Value) -> Result<(), String> {
    fields(
        value,
        &[
            "schema_version",
            "generated_at",
            "wrapper_version",
            "coverage",
        ],
    )?;
    for command in value["coverage"]
        .as_array()
        .ok_or("missing claim coverage")?
    {
        fields(
            command,
            &["path", "level", "note", "scope", "flags", "args"],
        )?;
        for flag in command["flags"].as_array().into_iter().flatten() {
            fields(flag, &["key", "level", "note", "scope"])?;
        }
        for arg in command["args"].as_array().into_iter().flatten() {
            fields(arg, &["name", "level", "note", "scope"])?;
        }
    }
    Ok(())
}

// The union engine recomputes every semantic field, but deliberately ignores optional raw-help
// provenance during equality checking. That provenance is retained in the basis, and new fields
// there must still refuse even if a widened schema would permit them.
pub(super) fn supported_union(value: &Value) -> Result<(), String> {
    for command in value["commands"]
        .as_array()
        .ok_or("missing union commands")?
    {
        for conflict in command["conflicts"].as_array().into_iter().flatten() {
            if let Some(evidence) = conflict.get("evidence") {
                fields(evidence, &["help_ref_by_target", "help_sha256_by_target"])?;
            }
        }
    }
    Ok(())
}
