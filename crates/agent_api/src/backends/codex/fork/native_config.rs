//! Pinned exec-compatible roots; native Codex still evaluates permissions.
use super::super::CodexBackendError;
use codex::mcp::{CodexAppServer, McpError};
use serde_json::Value;
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};
fn invalid() -> CodexBackendError {
    CodexBackendError::AppServer(McpError::Server("invalid native configuration".into()))
}
pub(super) async fn workspace_roots(
    server: &CodexAppServer,
    cwd: &Path,
    add_dirs: &[PathBuf],
) -> Result<Option<Vec<PathBuf>>, CodexBackendError> {
    let config = super::lifecycle::reply(
        server
            .config_read(cwd.to_path_buf())
            .await
            .map_err(CodexBackendError::AppServer)?,
    )
    .await?;
    let requirements = super::lifecycle::reply(
        server
            .config_requirements_read()
            .await
            .map_err(CodexBackendError::AppServer)?,
    )
    .await?;
    merge(cwd, add_dirs, &config, &requirements).map(Some)
}
fn merge(
    cwd: &Path,
    add_dirs: &[PathBuf],
    config: &Value,
    requirements: &Value,
) -> Result<Vec<PathBuf>, CodexBackendError> {
    if config.get("origins").and_then(Value::as_object).is_none()
        || !requirements.is_object()
        || requirements.get("requirements").is_none()
    {
        return Err(invalid());
    }
    let config = config
        .get("config")
        .and_then(Value::as_object)
        .ok_or_else(invalid)?;
    let default_profile = match config.get("default_permissions") {
        None | Some(Value::Null) => false,
        Some(Value::String(_)) => true,
        _ => return Err(invalid()),
    };
    let requirements = match requirements.get("requirements") {
        None | Some(Value::Null) => None,
        Some(Value::Object(object)) => Some(object),
        _ => return Err(invalid()),
    };
    let allowed_profiles =
        match requirements.and_then(|requirements| requirements.get("allowedPermissionProfiles")) {
            None | Some(Value::Null) => false,
            Some(Value::Object(profiles)) if profiles.values().all(Value::is_boolean) => true,
            _ => return Err(invalid()),
        };
    let ambient = match config.get("sandbox_workspace_write") {
        None | Some(Value::Null) => Vec::new(),
        Some(Value::Object(workspace)) => match workspace.get("writable_roots") {
            None => Vec::new(),
            Some(Value::Array(roots)) => roots
                .iter()
                .map(|root| {
                    root.as_str()
                        .map(PathBuf::from)
                        .filter(|path| path.is_absolute())
                        .ok_or_else(invalid)
                })
                .collect::<Result<Vec<_>, _>>()?,
            _ => return Err(invalid()),
        },
        _ => return Err(invalid()),
    };
    let mut roots = Vec::new();
    let mut seen = BTreeSet::new();
    for root in std::iter::once(cwd.to_path_buf())
        .chain(add_dirs.iter().cloned())
        .chain(
            ambient
                .into_iter()
                .filter(|_| !default_profile && !allowed_profiles),
        )
    {
        if seen.insert(root.clone()) {
            roots.push(root);
        }
    }
    Ok(roots)
}
#[cfg(test)]
mod tests {
    use super::merge;
    use serde_json::json;
    use std::path::{Path, PathBuf};
    #[test]
    fn roots_preserve_exec_order_ambient_and_profile_conditions() {
        let cwd = Path::new("/workspace");
        let dirs = vec![PathBuf::from("/extra"), PathBuf::from("/workspace")];
        for default in [false, true] {
            for allowed in [false, true] {
                let mut config = json!({"origins":{},"config":{"sandbox_workspace_write":{"writable_roots":["/ambient","/extra"]}}});
                if default {
                    config["config"]["default_permissions"] = json!("profile");
                }
                let requirements = if allowed {
                    json!({"requirements":{"allowedPermissionProfiles":{"profile":true}}})
                } else {
                    json!({"requirements":null})
                };
                let roots = merge(cwd, &dirs, &config, &requirements).unwrap();
                let expected = if default || allowed {
                    vec!["/workspace", "/extra"]
                } else {
                    vec!["/workspace", "/extra", "/ambient"]
                };
                assert_eq!(
                    roots,
                    expected.into_iter().map(PathBuf::from).collect::<Vec<_>>()
                );
            }
        }
    }
    #[test]
    fn malformed_configuration_cannot_drop_roots() {
        for config in [
            json!({}),
            json!({"origins":{},"config":{"sandbox_workspace_write":{"writable_roots":["relative"]}}}),
            json!({"origins":{},"config":{"default_permissions":true}}),
        ] {
            assert!(merge(
                Path::new("/workspace"),
                &[],
                &config,
                &json!({"requirements":null})
            )
            .is_err());
        }
    }
}
