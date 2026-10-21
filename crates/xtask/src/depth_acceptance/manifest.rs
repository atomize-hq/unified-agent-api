use super::{
    effect,
    input::{strings, text, View},
    Effect,
};
use serde_json::Value;
use std::collections::BTreeSet;

pub(super) fn pointer(view: &View, path: &str) -> Result<Option<String>, String> {
    view.source
        .optional(path)?
        .map(|raw| {
            let value = raw.trim();
            if value == "none" {
                Ok(None)
            } else if semver::Version::parse(value)
                .is_ok_and(|v| v.pre.is_empty() && v.build.is_empty())
            {
                Ok(Some(value.into()))
            } else {
                Err(format!("{path}: invalid pointer"))
            }
        })
        .transpose()
        .map(Option::flatten)
}
pub(super) fn targets(view: &View, root: &str) -> Result<Vec<String>, String> {
    let current = view.json(&format!("{root}/current.json"))?;
    strings(
        current
            .get("expected_targets")
            .ok_or("current lacks expected_targets")?,
    )
}
pub(super) fn pointer_path(
    view: &View,
    root: &str,
    key: &str,
    target: Option<&str>,
) -> Result<String, String> {
    let rules = view.json(&format!("{root}/RULES.json"))?;
    let pointers = rules
        .pointer("/storage/pointers")
        .ok_or("missing pointer declarations")?;
    let path = if key == "compat" {
        text(&pointers["compat"], "latest_validated")?
    } else {
        text(pointers, key)?
    };
    let path = path.replace("<target_triple>", target.unwrap_or(""));
    if !path.starts_with(&format!("{root}/")) || path.contains('<') {
        return Err("invalid root pointer declaration".into());
    }
    Ok(path)
}
pub(super) fn detect(
    before: &View,
    after: &View,
    effects: &mut BTreeSet<Effect>,
) -> Result<(), String> {
    let root = &after.entry.manifest_root;
    let mut covered: BTreeSet<String> = after
        .records
        .values()
        .flat_map(crate::support_matrix::depth::covered_targets)
        .map(str::to_owned)
        .collect();
    covered.extend(targets(before, root)?);
    covered.extend(targets(after, root)?);
    for key in [
        "latest_validated_by_target",
        "latest_supported_by_target",
        "compat",
    ] {
        let scope: Vec<_> = if key == "compat" {
            vec![None]
        } else {
            covered.iter().map(|t| Some(t.as_str())).collect()
        };
        for target in scope {
            let old = pointer_path(before, root, key, target)?;
            let path = pointer_path(after, root, key, target)?;
            let now = pointer(after, &path)?;
            if now != pointer(before, &old)? {
                if let Some(version) = now {
                    effect(effects, &version, target, &path, key);
                }
            }
        }
    }
    for version in after.records.keys() {
        let path = format!("{root}/versions/{version}.json");
        let now = after.json(&path)?;
        let old = before.optional_json(&path)?.unwrap_or(Value::Null);
        if now["schema_version"] != 1 || now["semantic_version"] != *version {
            return Err(format!("{path}: unsupported version metadata"));
        }
        if !old.is_null() && (old["schema_version"] != 1 || old["semantic_version"] != *version) {
            return Err(format!("{path}: unsupported baseline metadata"));
        }
        let status = text(&now, "status")?;
        if ["validated", "supported"].contains(&status) && old["status"] != now["status"] {
            effect(effects, version, None, &path, "status");
        }
        let passed = |v: &Value| -> Result<Vec<String>, String> {
            v.pointer("/validation/passed_targets")
                .map(strings)
                .unwrap_or(Ok(Vec::new()))
        };
        let previous = passed(&old)?;
        for target in passed(&now)? {
            if !previous.contains(&target) {
                effect(
                    effects,
                    version,
                    Some(&target),
                    &path,
                    "validation.passed_targets",
                );
            }
        }
        let marker = format!(
            "docs/agents/lifecycle/{}-maintenance/governance/automation-stand-down/{version}.toml",
            after.entry.agent_id
        );
        if before.source.optional(&marker)?.is_some() && after.source.optional(&marker)?.is_none() {
            effect(effects, version, None, &marker, "marker retirement");
        }
    }
    Ok(())
}
