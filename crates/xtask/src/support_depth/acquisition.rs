use super::{
    acquisition_basis::{self, AcquisitionFacts, ClassificationBasis},
    acquisition_modes, canonical_set,
    git_inputs::GitInputs,
};
use crate::{
    agent_maintenance::support_audit::{surface_identity, ReportRowShape, SurfaceIdentity},
    depth_record::{
        part_identity, AcceptanceWork, FileRef, ObligationsPart, OperationObligations,
        OperationSurface, PolicyPart,
    },
    manifest_report, manifest_union,
};
use jsonschema::{Draft, JSONSchema};
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub(super) type Surfaces = BTreeMap<SurfaceIdentity, BTreeSet<String>>;

pub(super) struct Acquired {
    pub obligations: ObligationsPart,
    pub classification_bases: Vec<ClassificationBasis>,
}

pub(super) fn derive(
    view: &mut GitInputs,
    root: &str,
    rules_bytes: &[u8],
    policy: &PolicyPart,
    carried: Vec<AcceptanceWork>,
    required: &BTreeSet<SurfaceIdentity>,
) -> Result<Option<Acquired>, String> {
    derive_with_pilot(view, root, rules_bytes, policy, carried, required, None)
}

pub(super) fn derive_with_pilot(
    view: &mut GitInputs,
    root: &str,
    rules_bytes: &[u8],
    policy: &PolicyPart,
    carried: Vec<AcceptanceWork>,
    required: &BTreeSet<SurfaceIdentity>,
    pilot: Option<&(Value, Vec<FileRef>)>,
) -> Result<Option<Acquired>, String> {
    let selection = &policy.selection;
    let agent = &selection.agent_id;
    let version = &selection.enrollment.version;
    let snapshots = format!("{root}/snapshots/{version}");
    let reports_dir = format!("{root}/reports/{version}");
    let snapshot_files = view.list(&snapshots)?;
    let report_files: Vec<_> = view
        .list(&reports_dir)?
        .into_iter()
        .filter(|p| p.starts_with("coverage.") && p.ends_with(".json"))
        .collect();
    if snapshot_files.is_empty() && report_files.is_empty() {
        if pilot.is_some() {
            return Err("pilot requires complete CLI acquisition".into());
        }
        if policy
            .selection
            .enrollment
            .modes
            .iter()
            .flat_map(|m| &m.predicates)
            .any(|p| p.flag != "--json" || p.value.is_some())
        {
            return Err(
                "without acquisition only canonical --json presence has structural meaning".into(),
            );
        }
        return Ok(None);
    }
    if !snapshot_files.iter().any(|f| f == "union.json") {
        return Err("partial acquisition: missing union".into());
    }
    let rules: Rules = serde_json::from_slice(rules_bytes).map_err(|e| e.to_string())?;
    let mut acquired = vec![view.reference(&format!("{root}/RULES.json"))?];
    let schema_path = format!("{root}/SCHEMA.json");
    let mut schema: Value =
        serde_json::from_slice(&view.required(&schema_path)?).map_err(|e| e.to_string())?;
    local_schema(&schema, true)?;
    schema
        .as_object_mut()
        .ok_or("schema must be an object")?
        .remove("$id");
    let schema = JSONSchema::options()
        .with_draft(Draft::Draft202012)
        .compile(&schema)
        .map_err(|e| e.to_string())?;
    let schema_ref = view.reference(&schema_path)?;
    let rules_sha256 = acquired[0].sha256.clone();
    let schema_sha256 = schema_ref.sha256.clone();
    acquired.push(schema_ref);
    let mut read = |path: &str| -> Result<Value, String> {
        let bytes = view.required(path)?;
        let value: Value = serde_json::from_slice(&bytes).map_err(|e| format!("{path}: {e}"))?;
        if let Err(errors) = schema.validate(&value) {
            return Err(format!(
                "{path}: invalid acquisition schema: {}",
                errors.map(|e| e.to_string()).collect::<Vec<_>>().join("; ")
            ));
        }
        acquired.push(view.reference(path)?);
        Ok(value)
    };
    let union = read(&format!("{snapshots}/union.json"))?;
    acquisition_basis::supported_union(&union)?;
    let mut shards = BTreeMap::new();
    for file in snapshot_files.iter().filter(|f| f.as_str() != "union.json") {
        let target = file
            .strip_suffix(".json")
            .filter(|t| rules.union.expected_targets.iter().any(|a| a == *t))
            .ok_or_else(|| format!("unexpected target snapshot: {file}"))?;
        shards.insert(target.to_owned(), read(&format!("{snapshots}/{file}"))?);
    }
    for shard in shards.values() {
        acquisition_basis::supported_shard(shard)?;
    }
    if let Some((facts, _)) = pilot {
        if shards
            .get(super::codex_pilot::TARGET)
            .map(|s| &s["binary"]["sha256"])
            != facts.get("binary_sha256")
        {
            return Err("pilot protocol acquisition belongs to another target binary".into());
        }
    }
    let selected_paths = selection
        .enrollment
        .modes
        .iter()
        .flat_map(|mode| {
            let paths = if pilot.is_some() {
                super::codex_pilot::paths(mode)
            } else {
                vec![mode.command_path.as_str()]
            };
            paths
                .into_iter()
                .map(|path| path.split(' ').skip(1).map(str::to_owned).collect())
                .collect::<Vec<_>>()
        })
        .collect();
    manifest_union::intake::validate(rules_bytes, version, &union, &shards, &selected_paths)?;
    if selection
        .enrollment
        .targets
        .iter()
        .any(|t| !shards.contains_key(t))
    {
        return Err("selected target has no acquired snapshot".into());
    }
    let wrapper = read(&format!("{root}/wrapper_coverage.json"))?;
    acquisition_basis::supported_wrapper(&wrapper)?;
    let mut reports = BTreeMap::new();
    for file in &report_files {
        reports.insert(file.clone(), read(&format!("{reports_dir}/{file}"))?);
    }
    let mut surfaces =
        manifest_report::intake::validate(agent, rules_bytes, &union, &wrapper, version, &reports)?;
    // Keep original per-target units as well as normalized union names. Global normalization and
    // wrapper coverage levels cannot erase a unit; available_on is never execution evidence.
    let mut observed = Surfaces::new();
    observe(agent, &union, &mut observed, None)?;
    for (target, shard) in &shards {
        observe(agent, shard, &mut observed, Some(target))?;
    }
    for (surface, targets) in &observed {
        surfaces
            .entry(surface.clone())
            .or_default()
            .extend(targets.iter().cloned());
    }
    let mut work: BTreeMap<SurfaceIdentity, BTreeSet<String>> = carried
        .into_iter()
        .map(|w| (w.surface, w.targets.into_iter().collect()))
        .collect();
    for surface in required {
        if let Some(targets) = observed.get(surface).or_else(|| surfaces.get(surface)) {
            work.entry(surface.clone())
                .or_default()
                .extend(targets.iter().cloned());
        } else if !work.contains_key(surface) {
            return Err(format!(
                "required work has no bound target applicability: {}",
                surface.describe()
            ));
        }
    }
    let work = canonical_set(
        work.into_iter()
            .map(|(surface, targets)| {
                Ok(AcceptanceWork {
                    surface,
                    targets: canonical_set(targets.into_iter().collect())?,
                })
            })
            .collect::<Result<Vec<_>, String>>()?,
    )?;
    for mode in &selection.enrollment.modes {
        for target in mode
            .targets
            .as_ref()
            .unwrap_or(&selection.enrollment.targets)
        {
            if pilot.is_some() {
                super::codex_pilot::validate_mode(mode, &shards[target], &union)?;
            } else {
                acquisition_modes::validate(agent, mode, &shards[target], &union)?;
            }
        }
    }
    let facts = AcquisitionFacts {
        agent,
        version,
        union: &union,
        shards: &shards,
        wrapper: &wrapper,
        surfaces: &surfaces,
        rules_sha256: &rules_sha256,
        schema_sha256: &schema_sha256,
    };
    let mut classification_bases = Vec::new();
    let mut operations = Vec::new();
    let mut classification = Vec::new();
    for operation in &selection.enrollment.operations {
        let modes: Vec<_> = selection
            .enrollment
            .modes
            .iter()
            .filter(|m| operation.serves.contains(&m.selector))
            .collect();
        let mut owned = BTreeSet::new();
        for mode in modes {
            let targets = mode
                .targets
                .as_ref()
                .unwrap_or(&selection.enrollment.targets);
            let found = surfaces.iter().any(|(s, ts)| {
                s.command_path == mode.command_path
                    && matches!(s.surface_kind.as_str(), "commands" | "subcommands")
                    && ts.iter().any(|t| targets.contains(t))
            });
            if !found {
                return Err(format!(
                    "selector `{}` has no observed or claimed command",
                    mode.selector
                ));
            }
            for (surface, available) in &surfaces {
                let global = rules.globals.effective_flags_model.enabled
                    && surface.surface_kind == "global_flags"
                    && surface.command_path == *agent;
                let selected_paths = if pilot.is_some() {
                    super::codex_pilot::paths(mode)
                } else {
                    vec![mode.command_path.as_str()]
                };
                if (selected_paths.contains(&surface.command_path.as_str()) || global)
                    && available.iter().any(|t| targets.contains(t))
                {
                    owned.insert(surface.clone());
                }
            }
        }
        let owned: Vec<SurfaceIdentity> = canonical_set(owned.into_iter().collect())?;
        // One exact mode/target is the finite first classified-policy scope. Other existing
        // declarations retain their old fully unresolved O without inventing a combined basis.
        if selection.enrollment.targets.len() == 1
            && (selection.enrollment.modes.len() == 1 || pilot.is_some())
        {
            let mut basis = facts.basis(&operation.id, &selection.enrollment.targets[0], &owned)?;
            if let Some((context, _)) = pilot {
                basis.facts["pilot"] = context.clone();
                basis.facts["modes"] = serde_json::to_value(&modes_for_basis(policy, operation))
                    .map_err(|e| e.to_string())?;
                basis.sha256 = part_identity(&basis.facts)?;
            }
            if operation
                .classification_basis
                .as_ref()
                .is_some_and(|expected| expected != &basis.sha256)
            {
                return Err(format!(
                    "operation `{}` has a stale classification_basis",
                    operation.id
                ));
            }
            classification_bases.push(basis);
        }
        let rows = operation.classifications.as_deref().unwrap_or_default();
        for row in rows {
            if !owned.contains(&row.surface) {
                return Err(format!(
                    "operation `{}` classification is foreign or unowned: {}",
                    operation.id,
                    row.surface.describe()
                ));
            }
        }
        for surface in &owned {
            if !rows.iter().any(|row| row.surface == *surface) {
                classification.push(OperationSurface {
                    operation: operation.id.clone(),
                    surface: surface.clone(),
                });
            }
        }
        operations.push(OperationObligations {
            id: operation.id.clone(),
            surfaces: owned,
            obligations: canonical_set(operation.obligations().into_iter().collect())?,
        });
    }
    if let Some((_, references)) = pilot {
        acquired.extend(references.iter().cloned());
    }
    let mut result = ObligationsPart {
        identity: String::new(),
        policy_identity: policy.identity.clone(),
        targets: canonical_set(selection.enrollment.targets.clone())?,
        acquired_inputs: canonical_set(acquired)?,
        operations: canonical_set(operations)?,
        classification_required: canonical_set(classification)?,
        acceptance_work: work,
    };
    result.identity = part_identity(&result)?;
    Ok(Some(Acquired {
        obligations: result,
        classification_bases,
    }))
}

fn observe(
    agent: &str,
    value: &Value,
    out: &mut Surfaces,
    target: Option<&String>,
) -> Result<(), String> {
    let commands = value["commands"].as_array().ok_or("missing commands")?;
    for command in commands {
        let path: Vec<String> =
            serde_json::from_value(command["path"].clone()).map_err(|e| e.to_string())?;
        let mut add = |row: &Value, shape: ReportRowShape, leaf: &str| -> Result<(), String> {
            let available: Vec<String> = match target {
                Some(t) => vec![t.clone()],
                None => serde_json::from_value(row["available_on"].clone())
                    .map_err(|e| e.to_string())?,
            };
            out.entry(surface_identity(agent, &path, shape, leaf))
                .or_default()
                .extend(available);
            Ok(())
        };
        add(command, ReportRowShape::Command, "")?;
        if let Some(flags) = command["flags"].as_array() {
            for flag in flags {
                let key = flag
                    .get("key")
                    .or_else(|| flag.get("long"))
                    .or_else(|| flag.get("short"))
                    .and_then(Value::as_str)
                    .ok_or("flag lacks canonical key")?;
                add(flag, ReportRowShape::Flag, key)?;
            }
        }
        if let Some(args) = command["args"].as_array() {
            for arg in args {
                add(
                    arg,
                    ReportRowShape::Arg,
                    arg["name"].as_str().ok_or("arg lacks name")?,
                )?;
            }
        }
    }
    Ok(())
}

// Compilation is offline and consumes only this bound schema. External policy/schema inputs
// cannot be fetched invisibly; supporting them later requires binding each referenced file.
fn local_schema(value: &Value, root: bool) -> Result<(), String> {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                if key == "$ref" && !value.as_str().is_some_and(|r| r.starts_with("#/$defs/")) {
                    return Err("external schema references are unsupported".into());
                }
                if (key == "$id" && !root)
                    || matches!(key.as_str(), "$dynamicRef" | "$recursiveRef")
                {
                    return Err(
                        "nested schema identities/dynamic references are unsupported".into(),
                    );
                }
                local_schema(value, false)?;
            }
        }
        Value::Array(items) => {
            for item in items {
                local_schema(item, false)?;
            }
        }
        _ => {}
    }
    Ok(())
}

#[derive(Deserialize)]
struct Rules {
    union: Targets,
    globals: Globals,
}
#[derive(Deserialize)]
struct Targets {
    expected_targets: Vec<String>,
}
#[derive(Deserialize)]
struct Globals {
    effective_flags_model: Flags,
}
#[derive(Deserialize)]
struct Flags {
    enabled: bool,
}

fn modes_for_basis<'a>(
    policy: &'a PolicyPart,
    operation: &crate::agent_registry::support_depth::OperationDeclaration,
) -> Vec<&'a crate::agent_registry::support_depth::ModeSelector> {
    policy
        .selection
        .enrollment
        .modes
        .iter()
        .filter(|m| operation.serves.contains(&m.selector))
        .collect()
}
