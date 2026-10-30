//! Project only actual owning-child replies; never issue diagnostic RPCs or retain raw values.
use super::*;
use serde_json::Value;
use std::path::PathBuf;
pub(crate) struct ReplyRoles {
    cwd: Option<PathBuf>,
    roots: Option<Vec<PathBuf>>,
}
impl ReplyRoles {
    pub(crate) fn from_params(params: &Value) -> Option<Self> {
        if !tracing::enabled!(target: TARGET, tracing::Level::TRACE) {
            return None;
        }
        let cwd = params["cwd"]
            .as_str()
            .filter(|p| p.len() <= 4096)
            .map(PathBuf::from);
        let roots = params["runtimeWorkspaceRoots"]
            .as_array()
            .filter(|r| r.len() <= 64)
            .and_then(|r| {
                r.iter()
                    .map(|p| p.as_str().filter(|p| p.len() <= 4096).map(PathBuf::from))
                    .collect()
            });
        Some(Self { cwd, roots })
    }
}
fn matches_path(value: &Value, expected: &Path) -> bool {
    // The producer owns canonical role validation. Keep the SDK projection passive:
    // compare in memory, without a filesystem read or normalization of native replies.
    let rebuilt: PathBuf = expected.components().collect();
    value
        .as_str()
        .is_some_and(|p| Path::new(p).as_os_str() == expected.as_os_str())
        && expected.is_absolute()
        && rebuilt.as_os_str() == expected.as_os_str()
        && !expected
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
}

fn approval(value: &Value) -> Option<ThreadApproval> {
    match value.as_str() {
        Some("never") => return Some(ThreadApproval::Never),
        Some("on-request") => return Some(ThreadApproval::OnRequest),
        Some("untrusted") => return Some(ThreadApproval::Untrusted),
        Some("on-failure") => return Some(ThreadApproval::OnFailure),
        _ => (),
    }
    let outer = value.as_object()?;
    if outer.len() != 1 {
        return None;
    }
    let fields = outer.get("granular")?.as_object()?;
    if fields.len() != 5 {
        return None;
    }
    Some(ThreadApproval::Granular {
        sandbox_approval: fields.get("sandbox_approval")?.as_bool()?,
        rules: fields.get("rules")?.as_bool()?,
        mcp_elicitations: fields.get("mcp_elicitations")?.as_bool()?,
        request_permissions: fields.get("request_permissions")?.as_bool()?,
        skill_approval: fields.get("skill_approval")?.as_bool()?,
    })
}
pub(crate) fn thread_reply(
    value: &Value,
    operation: ThreadOperation,
    roles: Option<&ReplyRoles>,
    owner: u64,
) {
    if !tracing::enabled!(target: TARGET, tracing::Level::TRACE) {
        return;
    }
    let projection = ThreadSettings {
        operation,
        model_gpt5: value["model"] == "gpt-5",
        provider_openai: value["modelProvider"] == "openai",
        service_tier_default: value.get("serviceTier") == Some(&Value::Null),
        approval: approval(&value["approvalPolicy"]),
        reviewer_user: value["approvalsReviewer"] == "user",
        sandbox_mode: match value["sandbox"]["type"].as_str() {
            Some("readOnly") => Some(Sandbox::ReadOnly),
            Some("workspaceWrite") => Some(Sandbox::WorkspaceWrite),
            Some("dangerFullAccess") => Some(Sandbox::DangerFullAccess),
            _ => None,
        },
        cwd_matches: roles
            .and_then(|r| r.cwd.as_deref())
            .is_some_and(|cwd| matches_path(&value["cwd"], cwd)),
        runtime_roots_match: roles
            .and_then(|r| r.roots.as_ref())
            .zip(value["runtimeWorkspaceRoots"].as_array())
            .is_some_and(|(expected, observed)| {
                expected.len() == observed.len()
                    && expected
                        .iter()
                        .zip(observed)
                        .all(|(p, v)| matches_path(v, p))
            }),
        instructions_empty: value["instructionSources"]
            .as_array()
            .is_some_and(Vec::is_empty),
    };
    emit("thread", &projection, owner);
}
pub(crate) fn turn_reply(value: &Value, owner: u64) {
    if !tracing::enabled!(target: TARGET, tracing::Level::TRACE) {
        return;
    }
    let accepted = value["turn"]["status"] == "inProgress"
        && value["turn"]["id"]
            .as_str()
            .is_some_and(|id| !id.is_empty() && id.len() <= 128);
    emit("turn", &accepted, owner);
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn assigned_path_comparison_rejects_lexical_aliases_without_filesystem_reads() {
        let root = Path::new("/source-owned-role/does-not-have-to-exist");
        assert!(matches_path(
            &Value::String(root.display().to_string()),
            root
        ));
        assert!(!matches_path(
            &Value::String(format!("{}/.", root.display())),
            root
        ));
        assert!(!matches_path(
            &Value::String(root.display().to_string()),
            &root.join(".")
        ));
        assert!(!matches_path(
            &Value::String("/root/../work".into()),
            Path::new("/root/../work")
        ));
    }
    #[test]
    fn granular_projection_requires_all_flags_and_rejects_unknown_fields() {
        let value = serde_json::json!({"granular":{"sandbox_approval":true,"rules":false,"mcp_elicitations":true,"request_permissions":false,"skill_approval":true}});
        let full = approval(&value).unwrap();
        assert_ne!(full, ThreadApproval::OnRequest);
        for key in [
            "sandbox_approval",
            "rules",
            "mcp_elicitations",
            "request_permissions",
            "skill_approval",
        ] {
            let mut missing = value.clone();
            missing["granular"].as_object_mut().unwrap().remove(key);
            assert!(approval(&missing).is_none());
        }
        let mut extra = value;
        extra["granular"]["secret"] = Value::String("sentinel".into());
        assert!(approval(&extra).is_none());
    }
}
