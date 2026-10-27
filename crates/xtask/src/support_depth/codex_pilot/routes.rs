use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PilotTransport {
    Exec,
    AppServer,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PilotRoute {
    pub selector: String,
    pub operation: String,
    pub transport: PilotTransport,
    /// Original id bytes are preserved. Last never becomes an invented id or an initial run.
    pub session_id: Option<String>,
    pub external_sandbox: bool,
    pub noninteractive: bool,
}

#[derive(Deserialize)]
#[serde(tag = "selector", rename_all = "snake_case", deny_unknown_fields)]
enum Session {
    Last,
    Id { id: String },
}

/// Decode only the finite selector/safety context. Remaining request validation belongs to the
/// production adapter; this diagnostic does not authorize invocation or assert full validity.
pub fn decode_route(
    extensions: &BTreeMap<String, Value>,
    external_opt_in: bool,
) -> Result<PilotRoute, String> {
    let known = [
        "agent_api.session.resume.v1",
        "agent_api.session.fork.v1",
        "agent_api.exec.non_interactive",
        "agent_api.exec.external_sandbox.v1",
        "backend.codex.exec.approval_policy",
        "backend.codex.exec.sandbox_mode",
        "agent_api.exec.add_dirs.v1",
        "agent_api.config.model.v1",
    ];
    if extensions.keys().any(|key| !known.contains(&key.as_str())) {
        return Err("unknown pilot request extension".into());
    }
    let resume = extensions.get("agent_api.session.resume.v1");
    let fork = extensions.get("agent_api.session.fork.v1");
    if resume.is_some() && fork.is_some() {
        return Err("resume and fork are mutually exclusive".into());
    }
    let boolean = |key: &str, default| -> Result<bool, String> {
        extensions
            .get(key)
            .map(|v| v.as_bool().ok_or_else(|| format!("{key} must be boolean")))
            .unwrap_or(Ok(default))
    };
    let noninteractive = boolean("agent_api.exec.non_interactive", true)?;
    let external = boolean("agent_api.exec.external_sandbox.v1", false)?;
    if extensions.contains_key("agent_api.exec.external_sandbox.v1") && !external_opt_in {
        return Err("external sandbox extension requires backend opt-in".into());
    }
    if external
        && (!noninteractive
            || extensions
                .keys()
                .any(|k| k.starts_with("backend.codex.exec.")))
    {
        return Err("external sandbox conflicts with interactive or exec policy intent".into());
    }
    if let Some(policy) = extensions.get("backend.codex.exec.approval_policy") {
        let valid = match policy {
            Value::String(s) => matches!(
                s.as_str(),
                "never" | "untrusted" | "on-failure" | "on-request"
            ),
            Value::Object(_) => granular(policy),
            _ => false,
        };
        if !valid || (noninteractive && policy != "never") {
            return Err("invalid approval-policy context".into());
        }
    }
    if let Some(sandbox) = extensions.get("backend.codex.exec.sandbox_mode") {
        if !matches!(
            sandbox.as_str(),
            Some("read-only" | "workspace-write" | "danger-full-access")
        ) {
            return Err("unknown sandbox context".into());
        }
    }
    let session = resume
        .or(fork)
        .map(|value| {
            if value
                .as_object()
                .is_some_and(|o| o.get("selector").is_some_and(|v| v == "last") && o.len() != 1)
            {
                return Err("last selector cannot contain additional fields".to_string());
            }
            serde_json::from_value::<Session>(value.clone())
                .map_err(|_| "invalid session selector".to_string())
        })
        .transpose()?;
    let (variant, session_id) = match session {
        None => ("", None),
        Some(Session::Last) => ("last", None),
        Some(Session::Id { id }) if !id.trim().is_empty() => ("id", Some(id)),
        _ => return Err("blank session id".into()),
    };
    let (selector, operation) = if fork.is_some() {
        (format!("fork-{variant}-stdio"), "run-fork")
    } else if resume.is_some() {
        (format!("resume-{variant}-json"), "run-resume")
    } else {
        ("initial-json".into(), "run-initial")
    };
    let transport = if fork.is_some() || (!noninteractive && !external) {
        PilotTransport::AppServer
    } else {
        PilotTransport::Exec
    };
    Ok(PilotRoute {
        selector,
        operation: operation.into(),
        transport,
        session_id,
        external_sandbox: external,
        noninteractive,
    })
}

fn granular(value: &Value) -> bool {
    let Some(object) = value.as_object().filter(|o| o.len() == 1) else {
        return false;
    };
    let Some(fields) = object.get("granular").and_then(Value::as_object) else {
        return false;
    };
    let required = ["sandbox_approval", "rules", "mcp_elicitations"];
    required
        .iter()
        .all(|k| fields.get(*k).is_some_and(Value::is_boolean))
        && fields.iter().all(|(k, v)| {
            matches!(
                k.as_str(),
                "sandbox_approval"
                    | "rules"
                    | "mcp_elicitations"
                    | "skill_approval"
                    | "request_permissions"
            ) && v.is_boolean()
        })
}
