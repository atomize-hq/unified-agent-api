use std::fs;

use serde_json::Value;

use super::{is_union_snapshot, rel_path, schema, ValidateCtx, Violation};

pub(super) fn validate_current_json(
    ctx: &mut ValidateCtx,
    violations: &mut Vec<Violation>,
    latest_validated: Option<&str>,
) {
    let current_path = ctx.root.join("current.json");
    let current_value = match schema::read_json_file(
        &ctx.root,
        &current_path,
        violations,
        "CURRENT_INVALID_JSON",
    ) {
        Some(v) => {
            if latest_validated.is_none()
                && ctx.root_pointers_allow_none
                && is_bootstrap_pointer(&v, &ctx.expected_targets)
            {
                // Bootstrap mode: the pointer `onboard-agent` writes before any version is
                // promoted names the expected targets and no inputs. The support matrix reads it
                // as a root with no current version. Promotion replaces it with a union snapshot.
                return;
            }
            schema::schema_validate(
                ctx,
                violations,
                &ctx.schema,
                &v,
                &current_path,
                "CURRENT_SCHEMA_INVALID",
            );
            if !is_union_snapshot(&v) {
                violations.push(Violation {
                    code: "CURRENT_WRONG_KIND",
                    path: rel_path(&ctx.root, &current_path),
                    json_pointer: Some("/snapshot_schema_version".to_string()),
                    message: "current.json must be an UpstreamSnapshotUnionV2 (snapshot_schema_version=2, mode=union)".to_string(),
                    unit: Some("current_json"),
                    command_path: None,
                    key_or_name: None,
                    field: Some("current"),
                    target_triple: None,
                    details: None,
                });
            }
            Some(v)
        }
        None => {
            if current_path.exists() {
                return;
            }
            if latest_validated.is_none() && ctx.root_pointers_allow_none {
                // Bootstrap mode: allow the parity root to exist without any promoted version yet.
                return;
            }
            violations.push(Violation {
                code: "CURRENT_MISSING",
                path: rel_path(&ctx.root, &current_path),
                json_pointer: None,
                message: "missing required file: current.json".to_string(),
                unit: Some("current_json"),
                command_path: None,
                key_or_name: None,
                field: Some("current"),
                target_triple: None,
                details: None,
            });
            None
        }
    };

    let Some(latest_validated) = latest_validated else {
        return;
    };
    let union_path = ctx
        .root
        .join("snapshots")
        .join(latest_validated)
        .join("union.json");

    if current_path.is_file() && union_path.is_file() {
        if let (Ok(a), Ok(b)) = (fs::read(&current_path), fs::read(&union_path)) {
            if a != b {
                violations.push(Violation {
                    code: "CURRENT_JSON_NOT_EQUAL_UNION",
                    path: rel_path(&ctx.root, &current_path),
                    json_pointer: None,
                    message: format!(
                        "current.json must be byte-for-byte identical to snapshots/{latest_validated}/union.json"
                    ),
                    unit: Some("current_json"),
                    command_path: None,
                    key_or_name: Some(latest_validated.to_string()),
                    field: Some("identity"),
                    target_triple: None,
                    details: None,
                });
            }
        }
    }

    // current.json semantic version invariants use the required target's input.binary.semantic_version.
    let Some(current_value) = current_value else {
        return;
    };
    let required_target = ctx.required_target.clone();
    let required_input = current_value
        .get("inputs")
        .and_then(Value::as_array)
        .and_then(|inputs| {
            inputs.iter().find(|i| {
                i.get("target_triple")
                    .and_then(Value::as_str)
                    .is_some_and(|t| t == required_target.as_str())
            })
        });
    let Some(required_input) = required_input else {
        violations.push(Violation {
            code: "CURRENT_JSON_MISSING_REQUIRED_TARGET",
            path: rel_path(&ctx.root, &current_path),
            json_pointer: Some("/inputs".to_string()),
            message: format!("current.json.inputs[] missing required_target={required_target}"),
            unit: Some("current_json"),
            command_path: None,
            key_or_name: Some(required_target.clone()),
            field: Some("inputs"),
            target_triple: Some(required_target),
            details: None,
        });
        return;
    };
    let semantic_version = required_input
        .get("binary")
        .and_then(|b| b.get("semantic_version"))
        .and_then(Value::as_str);
    if semantic_version != Some(latest_validated) {
        violations.push(Violation {
            code: "CURRENT_JSON_SEMVER_MISMATCH",
            path: rel_path(&ctx.root, &current_path),
            json_pointer: Some("/inputs/*/binary/semantic_version".to_string()),
            message: format!(
                "current.json required_target binary.semantic_version must equal latest_validated.txt (expected {latest_validated}, got {})",
                semantic_version.unwrap_or("<missing>")
            ),
            unit: Some("current_json"),
            command_path: None,
            key_or_name: Some(required_target.clone()),
            field: Some("semantic_version"),
            target_triple: Some(required_target),
            details: None,
        });
    }
}

/// `{"expected_targets": [...], "inputs": []}` with exactly RULES' expected targets, in order.
fn is_bootstrap_pointer(value: &Value, expected_targets: &[String]) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };
    object.len() == 2
        && object
            .get("inputs")
            .and_then(Value::as_array)
            .is_some_and(Vec::is_empty)
        && object
            .get("expected_targets")
            .and_then(Value::as_array)
            .is_some_and(|targets| {
                targets.len() == expected_targets.len()
                    && targets
                        .iter()
                        .zip(expected_targets)
                        .all(|(target, expected)| target.as_str() == Some(expected.as_str()))
            })
}

#[cfg(test)]
mod tests {
    use super::is_bootstrap_pointer;
    use serde_json::json;

    #[test]
    fn only_the_exact_onboarding_pointer_is_a_bootstrap_pointer() {
        let targets = vec!["linux-x64".to_string()];
        assert!(is_bootstrap_pointer(
            &json!({"expected_targets": ["linux-x64"], "inputs": []}),
            &targets
        ));
        for value in [
            json!({"expected_targets": ["darwin-arm64"], "inputs": []}),
            json!({"expected_targets": [], "inputs": []}),
            json!({"expected_targets": ["linux-x64"], "inputs": [{}]}),
            json!({"expected_targets": ["linux-x64"], "inputs": [], "mode": "union"}),
            json!({"expected_targets": ["linux-x64"]}),
            json!(["linux-x64"]),
        ] {
            assert!(!is_bootstrap_pointer(&value, &targets), "{value}");
        }
    }
}
