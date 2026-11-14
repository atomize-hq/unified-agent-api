//! Producer invariants in addition to complete independently authenticated acquisition bytes.
use crate::{freeze_transition::Tree, manifest_acquisition::AcquisitionPlan};
use serde_json::Value;
use std::path::Path;

pub(super) fn validate(
    root: &Path,
    predecessor: &Tree,
    plan: &AcquisitionPlan,
) -> Result<(), String> {
    let base = root.join(&plan.manifest_root);
    crate::manifest_validate::check_acquisition(base.clone())?;
    let metadata_path = format!("{}/versions/{}.json", plan.manifest_root, plan.version);
    let metadata = std::fs::read(root.join(&metadata_path)).map_err(|e| e.to_string())?;
    let value: Value = serde_json::from_slice(&metadata).map_err(|e| e.to_string())?;
    let stamp = value["updated_at"]
        .as_str()
        .ok_or("missing acquisition metadata clock")?;
    time::OffsetDateTime::parse(stamp, &time::format_description::well_known::Rfc3339)
        .map_err(|e| e.to_string())?;
    let (_, expected) = crate::manifest_version_metadata::plan_at(
        crate::manifest_version_metadata::Args {
            root: Some(base),
            rules: None,
            version: plan.version.clone(),
            status: crate::manifest_version_metadata::Status::Reported,
            passed_targets: vec![],
            failed_targets: vec![],
            skipped_targets: vec![],
        },
        predecessor.get(&metadata_path).map(|e| e.bytes.as_slice()),
        stamp,
    )
    .map_err(|e| e.to_string())?;
    if metadata != expected {
        return Err("acquisition reported metadata differs from producer".into());
    }
    let lock_path = format!("{}/artifacts.lock.json", plan.manifest_root);
    let before: Value = serde_json::from_slice(
        &predecessor
            .get(&lock_path)
            .filter(|entry| entry.regular())
            .ok_or("missing predecessor lockfile")?
            .bytes,
    )
    .map_err(|e| e.to_string())?;
    let after: Value =
        serde_json::from_slice(&std::fs::read(root.join(lock_path)).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    validate_lockfile(&before, &after, plan)
}

fn validate_lockfile(before: &Value, after: &Value, plan: &AcquisitionPlan) -> Result<(), String> {
    let mut preserved = before
        .as_object()
        .ok_or("invalid predecessor lockfile")?
        .clone();
    let mut actual = after
        .as_object()
        .ok_or("invalid acquired lockfile")?
        .clone();
    preserved.remove("artifacts");
    actual.remove("artifacts");
    if preserved != actual {
        return Err("acquisition replaced lockfile top-level content".into());
    }
    let rows = |v: &Value| -> Result<Vec<Value>, String> {
        // The existing jq writer treats missing/null predecessor artifacts as an empty list.
        if v["artifacts"].is_null() {
            return Ok(vec![]);
        }
        Ok(v["artifacts"]
            .as_array()
            .ok_or("invalid lockfile artifacts")?
            .clone())
    };
    let retained = |rows: Vec<Value>| {
        rows.into_iter()
            .filter(|r| r[&plan.lockfile_version_key] != plan.version)
            .collect::<Vec<_>>()
    };
    let sort = |mut rows: Vec<Value>| {
        rows.sort_by_key(|r| r.to_string());
        rows
    };
    if sort(retained(rows(before)?)) != sort(retained(rows(after)?)) {
        return Err("acquisition replaced predecessor lockfile rows".into());
    }
    let acquired = rows(after)?
        .into_iter()
        .filter(|r| r[&plan.lockfile_version_key] == plan.version)
        .collect::<Vec<_>>();
    if acquired.len() != plan.include.len() {
        return Err("acquisition lockfile target population differs".into());
    }
    for target in &plan.include {
        let matches: Vec<_> = acquired
            .iter()
            .filter(|r| r["target_triple"] == target.target_triple)
            .collect();
        if matches.len() != 1 {
            return Err("acquisition lockfile missing/duplicate target".into());
        }
        let row = matches[0];
        if row["asset_name"] != target.asset_name
            || row["download_url"] != target.download_url
            || row["size_bytes"].as_u64().filter(|s| *s > 0).is_none()
            || row["sha256"]
                .as_str()
                .map_or(true, |s| super::super::sha256(s).is_err())
            || target.platform_package.as_ref().is_some_and(|p| {
                row["package_name"] != *p || row["package_version"] != plan.version
            })
        {
            return Err("acquisition lockfile provenance differs from plan".into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn acquisition_lockfile_preserves_rows_top_level_and_exact_target_provenance() {
        let rules: Value = serde_json::from_str(include_str!(
            "../../../../tests/fixtures/depth-synthetic/governing/manifest-root/RULES.json"
        ))
        .unwrap();
        let descriptor: crate::manifest_acquisition::AcquisitionDescriptor =
            serde_json::from_value(rules["acquisition"].clone()).unwrap();
        let target = "x86_64-unknown-linux-gnu".to_owned();
        let plan = crate::manifest_acquisition::resolve(
            "depth_fixture",
            "cli_manifests/depth_fixture",
            "0.2.0",
            &target,
            std::slice::from_ref(&target),
            false,
            &descriptor,
        )
        .unwrap();
        let before = json!({"schema_version":1,"notes":"keep","artifacts":[{"semantic_version":"0.1.0","target_triple":target,"notes":"keep this row"}]});
        let mut after = before.clone();
        after["artifacts"].as_array_mut().unwrap().push(json!({"semantic_version":"0.2.0", "target_triple":target,
            "asset_name":plan.include[0].asset_name,"download_url":plan.include[0].download_url,"sha256":"a".repeat(64),"size_bytes":1}));
        validate_lockfile(&before, &after, &plan).unwrap();
        for empty in [
            json!({"schema_version":1,"notes":"keep"}),
            json!({"schema_version":1,"notes":"keep","artifacts":null}),
        ] {
            let mut acquired = after.clone();
            acquired["artifacts"].as_array_mut().unwrap().remove(0);
            validate_lockfile(&empty, &acquired, &plan).unwrap();
        }
        for mutation in 0..7 {
            let mut changed = after.clone();
            match mutation {
                0 => changed["notes"] = json!("replaced"),
                1 => changed["artifacts"][0]["notes"] = json!("replaced"),
                2 => {
                    changed["artifacts"].as_array_mut().unwrap().remove(0);
                }
                3 => {
                    let row = changed["artifacts"][1].clone();
                    changed["artifacts"].as_array_mut().unwrap().push(row);
                }
                4 => changed["artifacts"][1]["download_url"] = json!("other source"),
                5 => changed["artifacts"][1]["sha256"] = json!("incomplete digest"),
                _ => changed["artifacts"][1]["size_bytes"] = json!(0),
            }
            assert!(
                validate_lockfile(&before, &changed, &plan).is_err(),
                "{mutation}"
            );
        }
    }
}
