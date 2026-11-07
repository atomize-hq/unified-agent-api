//! Record-free input checks use paths, subject and approval values, never agent-name waivers.
use super::*;
use crate::agent_lifecycle as life;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn record_free(source: &Source, adapter: &Adapter, absent: bool) -> Result<(), String> {
    for prefix in ["cli_manifests", "reports"] {
        if source
            .paths(prefix)?
            .iter()
            .any(|p| p.ends_with("/depth-record.json"))
        {
            return Err("bootstrap admits no retained depth record".into());
        }
    }
    let pack = &adapter.scaffold.scaffold.onboarding_pack_prefix;
    for name in ["publication-ready.json", "proving-run-closeout.json"] {
        if source
            .optional(&format!("docs/agents/lifecycle/{pack}/governance/{name}"))?
            .is_some()
        {
            return Err("bootstrap admits no publication or closeout packet".into());
        }
    }
    let root = &adapter.scaffold.manifest_root;
    if absent && !source.paths(root)?.is_empty() {
        return Err("absent bootstrap subject has a preexisting manifest root".into());
    }
    if let Some(raw) = source.optional(&format!("{root}/current.json"))? {
        let bootstrap =
            crate::onboard_agent::render_current_json(&adapter.scaffold.canonical_targets);
        if raw != bootstrap {
            return Err("bootstrap current must preserve exact ordinary ON1 renderer bytes".into());
        }
    }
    for path in source.paths(root)? {
        let pointer = path.starts_with(&format!("{root}/pointers/")) && path.ends_with(".txt")
            || [
                format!("{root}/latest_validated.txt"),
                format!("{root}/min_supported.txt"),
            ]
            .contains(&path);
        if pointer {
            let raw = source.read(&path)?;
            crate::manifest_validate::parse_pointer_content(&raw)
                .map_err(|why| format!("invalid bootstrap pointer {path}: {why}"))?;
            if raw.as_bytes() != b"none\n" {
                return Err("bootstrap pointer acceptance is forbidden".into());
            }
        }
        if path.starts_with(&format!("{root}/versions/")) && path.ends_with(".json") {
            if path != format!("{root}/versions/{}.json", adapter.version) {
                return Err("bootstrap metadata selects another version".into());
            }
            let value: Value =
                serde_json::from_str(&source.read(&path)?).map_err(|e| e.to_string())?;
            if value["schema_version"] != 1
                || value["semantic_version"] != adapter.version
                || value["status"] != "snapshotted"
                || value.get("validation").is_some_and(|v| {
                    v.as_object().map_or(true, |m| {
                        m.values()
                            .any(|v| v.as_array().map_or(true, |a| !a.is_empty()))
                    })
                })
                || value
                    .pointer("/promotion/eligible_for_latest_validated")
                    .is_some_and(|v| v != false)
            {
                return Err(
                    "bootstrap metadata must be ordinary snapshotted without acceptance".into(),
                );
            }
            if source
                .optional(&format!("{root}/snapshots/{}/union.json", adapter.version))?
                .is_none()
            {
                return Err("snapshotted metadata lacks acquired union".into());
            }
            acquisition(source, adapter, false)?;
        }
    }
    if let Some(raw) = source.optional(&adapter.state_path())? {
        let state: life::LifecycleState = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
        state.validate().map_err(|e| e.to_string())?;
        let (approval_raw, _) = adapter.read_approval(source)?;
        if absent
            || state.agent_id != adapter.scaffold.agent_id
            || state.onboarding_pack_prefix != adapter.scaffold.scaffold.onboarding_pack_prefix
            || state.approval_artifact_path != adapter.approval_path()
            || state.approval_artifact_sha256 != digest(&approval_raw)
            || state.lifecycle_stage != life::LifecycleStage::Enrolled
            || state.support_tier != life::SupportTier::Bootstrap
            || state.required_evidence
                != life::required_evidence_for_stage(life::LifecycleStage::Enrolled)
            || state.satisfied_evidence
                != life::required_evidence_for_stage(life::LifecycleStage::Enrolled)
            || state.support_depth.is_some()
            || !state.side_states.is_empty()
            || state.active_runtime_evidence_run_id.is_some()
            || state.implementation_summary.is_some()
            || state.publication_packet_path.is_some()
            || state.publication_packet_sha256.is_some()
            || state.closeout_baseline_path.is_some()
            || state.maintenance_readiness_adoption_path.is_some()
            || state.maintenance_readiness_adoption_sha256.is_some()
        {
            return Err("bootstrap lifecycle binding or acceptance changed".into());
        }
    }
    Ok(())
}

pub(super) fn digest(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    hex::encode(Sha256::digest(bytes))
}

/// Verify current acquired bytes with the existing union/report engines. Partial acquisition
/// checkpoints may precede a union; metadata requires a union and declaration requires all O inputs.
pub(super) fn acquisition(
    source: &Source,
    adapter: &Adapter,
    complete: bool,
) -> Result<(), String> {
    raw_help::validate(source, adapter)?;
    let root = &adapter.scaffold.manifest_root;
    let version = adapter.version;
    let rules_path = format!("{root}/RULES.json");
    let read = |p: &str| -> Result<Value, String> {
        serde_json::from_str(&source.read(p)?).map_err(|e| e.to_string())
    };
    let acquired: BTreeSet<_> = adapter.acquired_paths()?.into_iter().collect();
    let mut artifact_paths = Vec::new();
    for directory in ["snapshots", "reports"] {
        for path in source.paths(&format!("{root}/{directory}"))? {
            if path.ends_with(".json") {
                if !acquired.contains(&path) {
                    return Err(format!("foreign bootstrap acquisition artifact: {path}"));
                }
                artifact_paths.push(path);
            }
        }
    }
    let wrapper_path = format!("{root}/wrapper_coverage.json");
    if source.optional(&wrapper_path)?.is_some() {
        artifact_paths.push(wrapper_path.clone());
    }
    if !artifact_paths.is_empty() {
        // The compiled profile's governing schema validates partial objects too. It has only
        // local references; no candidate-provided schema may widen this bounded acquisition.
        let mut schema: Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/depth-synthetic/governing/manifest-root/SCHEMA.json"
        ))
        .map_err(|e| e.to_string())?;
        schema
            .as_object_mut()
            .ok_or("invalid compiled acquisition schema")?
            .remove("$id");
        schema
            .as_object_mut()
            .ok_or("invalid compiled acquisition schema")?
            .remove("oneOf");
        schema["$defs"]["SemverString"]["enum"] = serde_json::json!([version]);
        schema["$defs"]["TargetTriple"]["enum"] =
            serde_json::json!(adapter.scaffold.canonical_targets);
        for path in &artifact_paths {
            let kind = if *path == wrapper_path {
                "WrapperCoverageV1"
            } else if path.ends_with("/union.json") {
                "UpstreamSnapshotUnionV2"
            } else if path.starts_with(&format!("{root}/reports/")) {
                "CoverageReportV1"
            } else {
                "UpstreamSnapshotV1"
            };
            schema["$ref"] = Value::String(format!("#/$defs/{kind}"));
            let validator = crate::support_depth::acquisition_schema::compile(
                &serde_json::to_vec(&schema).map_err(|e| e.to_string())?,
            )?;
            let value = read(path)?;
            crate::support_depth::acquisition_schema::validate(&validator, path, &value)?;
            match kind {
                "WrapperCoverageV1" => crate::support_depth::supported_wrapper(&value)?,
                "UpstreamSnapshotUnionV2" => crate::support_depth::supported_union(&value)?,
                "UpstreamSnapshotV1" => crate::support_depth::supported_shard(&value)?,
                _ => {}
            }
        }
    }
    wrapper::validate(source, adapter)?;
    encoding::check::<depth_fixture::wrapper_coverage_manifest::WrapperCoverageManifestV1>(
        source,
        &wrapper_path,
    )?;
    let mut shards = BTreeMap::new();
    for target in &adapter.scaffold.canonical_targets {
        let path = format!("{root}/snapshots/{version}/{target}.json");
        if source.optional(&path)?.is_some() {
            shards.insert(target.clone(), read(&path)?);
        } else if complete {
            return Err("declaration requires complete acquisition".into());
        }
    }
    let rules = source.optional_bytes(&rules_path)?;
    let paths: BTreeSet<_> = adapter
        .declared
        .support_depth
        .as_ref()
        .unwrap()
        .enrollments
        .as_ref()
        .unwrap()[0]
        .modes
        .iter()
        .map(|m| {
            m.command_path
                .split(' ')
                .skip(1)
                .map(str::to_owned)
                .collect()
        })
        .collect();
    if !shards.is_empty() {
        crate::manifest_union::intake::validate_shards(
            rules.as_deref().ok_or("missing acquired RULES")?,
            version,
            &shards,
            &paths,
        )?;
        for target in shards.keys() {
            encoding::check::<crate::manifest_snapshot_schema::SnapshotV1>(
                source,
                &format!("{root}/snapshots/{version}/{target}.json"),
            )?;
        }
    }
    let union_path = format!("{root}/snapshots/{version}/union.json");
    if source.optional(&union_path)?.is_none() {
        if complete {
            return Err("missing acquired union".into());
        }
        if artifact_paths
            .iter()
            .any(|path| path.starts_with(&format!("{root}/reports/")))
        {
            return Err("bootstrap report requires its acquired union".into());
        }
        return Ok(());
    }
    let rules = rules.ok_or("missing acquired RULES")?;
    let union = read(&union_path)?;
    crate::manifest_union::intake::validate(&rules, version, &union, &shards, &paths)?;
    crate::manifest_union::intake::validate_bytes(
        &rules,
        version,
        &union,
        &shards,
        &paths,
        &source
            .optional_bytes(&union_path)?
            .ok_or("missing union bytes")?,
    )?;
    let mut reports = BTreeMap::new();
    for name in ["any", "all"].into_iter().chain(
        adapter
            .scaffold
            .canonical_targets
            .iter()
            .map(String::as_str),
    ) {
        let path = format!("{root}/reports/{version}/coverage.{name}.json");
        if source.optional(&path)?.is_some() {
            reports.insert(
                format!("coverage.{name}.json"),
                source
                    .optional_bytes(&path)?
                    .ok_or("missing report bytes")?,
            );
        }
    }
    if complete || !reports.is_empty() {
        let wrapper = read(&format!("{root}/wrapper_coverage.json"))?;
        crate::manifest_report::intake::validate_bytes(
            &adapter.scaffold.agent_id,
            &rules,
            &union,
            &wrapper,
            version,
            &reports,
        )?;
    }
    Ok(())
}

/// Compare present governing files with the private adapter's expected scaffold/adoption.
pub(super) fn governing(source: &Source, adapter: &Adapter, complete: bool) -> Result<(), String> {
    for (name, expected) in adapter.governing()? {
        let path = format!("{}/{name}", adapter.scaffold.manifest_root);
        let Some(raw) = source.optional(&path)? else {
            if complete {
                return Err(format!("missing reviewed governing adoption: {path}"));
            }
            continue;
        };
        if raw != expected {
            return Err(format!(
                "bootstrap governing files differ from compiled template: {path}"
            ));
        }
    }
    Ok(())
}

/// NE3's whole output uses only the pinned parent's retained fields and a valid fresh timestamp.
pub(super) fn metadata(
    root: &Path,
    before: &Source,
    source: &Source,
    adapter: &Adapter,
) -> Result<(), String> {
    let path = format!(
        "{}/versions/{}.json",
        adapter.scaffold.manifest_root, adapter.version
    );
    let Some(raw) = source.optional_bytes(&path)? else {
        return Ok(());
    };
    let actual: Value = serde_json::from_slice(&raw).map_err(|e| e.to_string())?;
    let stamp = actual["updated_at"]
        .as_str()
        .ok_or("metadata lacks its NE3 timestamp")?;
    time::OffsetDateTime::parse(stamp, &time::format_description::well_known::Rfc3339)
        .map_err(|e| e.to_string())?;
    let (_, expected) = crate::manifest_version_metadata::plan_at(
        crate::manifest_version_metadata::Args {
            root: Some(root.join(&adapter.scaffold.manifest_root)),
            rules: None,
            version: adapter.version.into(),
            status: crate::manifest_version_metadata::Status::Snapshotted,
            passed_targets: vec![],
            failed_targets: vec![],
            skipped_targets: vec![],
        },
        before.optional_bytes(&path)?.as_deref(),
        stamp,
    )
    .map_err(|e| e.to_string())?;
    if raw != expected {
        return Err("bootstrap metadata differs from complete ordinary NE3 bytes".into());
    }
    Ok(())
}
