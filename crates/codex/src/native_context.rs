//! Passive finite launch diagnostics; no SDK dispatch/validation changes.
mod projection;
mod replies;
pub(crate) use projection::*;
use replies::layers;
pub(crate) use replies::{thread_reply, turn_reply, ReplyRoles};
use serde::Serialize;
use std::{collections::BTreeMap, ffi::OsString, path::Path};
use tokio::process::Command;
pub(crate) fn new_owner() -> u64 {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    if tracing::enabled!(target: TARGET,tracing::Level::TRACE) {
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    } else {
        0
    }
}
pub(crate) fn effective_environment(command: &Command) -> BTreeMap<OsString, OsString> {
    let mut env: BTreeMap<_, _> = std::env::vars_os().collect();
    for (key, value) in command.as_std().get_envs() {
        match value {
            Some(value) => {
                env.insert(key.into(), value.into());
            }
            None => {
                env.remove(key);
            }
        }
    }
    env
}
pub(crate) fn launch(
    command: &Command,
    binary: &Path,
    home: Option<&Path>,
    cwd: &Path,
    route: Route,
    spawned: bool,
    owner: u64,
) {
    if !tracing::enabled!(target: TARGET, tracing::Level::TRACE) {
        return;
    }
    let env = effective_environment(command);
    let native_home = home
        .map(Path::to_path_buf)
        .or_else(|| std::env::var_os("CODEX_HOME").map(std::path::PathBuf::from));
    let actual_home = env
        .get(std::ffi::OsStr::new("CODEX_HOME"))
        .map(std::path::PathBuf::from);
    let projection = Launch {
        route,
        spawned,
        binary_matches: command.as_std().get_program() == binary.as_os_str()
            && env
                .get(std::ffi::OsStr::new("CODEX_BINARY"))
                .is_some_and(|p| p == binary.as_os_str()),
        cwd_matches: command.as_std().get_current_dir() == Some(cwd),
        home_matches: native_home == actual_home,
        environment_known: environment_known(&env),
    };
    emit("launch", &projection, owner);
}
fn emit(kind: &'static str, value: &impl Serialize, owner: u64) {
    if let Ok(projection) = serde_json::to_string(value) {
        tracing::trace!(target: TARGET, kind, owner, projection=projection.as_str());
    }
}
#[cfg(test)]
mod tests;

/// Native home as the pinned resolver selects it from the child's effective environment.
pub(crate) fn native_home(command: &Command) -> Option<std::path::PathBuf> {
    if !tracing::enabled!(target: TARGET, tracing::Level::TRACE) {
        return None;
    }
    let env = effective_environment(command);
    env.get(std::ffi::OsStr::new("CODEX_HOME"))
        .map(std::path::PathBuf::from)
        .or_else(|| {
            env.get(std::ffi::OsStr::new("HOME"))
                .map(|h| Path::new(h).join(".codex"))
        })
}
pub(crate) fn config_reply(value: &serde_json::Value, roles: Option<&ReplyRoles>, owner: u64) {
    if !tracing::enabled!(target: TARGET, tracing::Level::TRACE) {
        return;
    }
    let c = &value["config"];
    let projection = Config {
        model_gpt5: c["model"] == "gpt-5",
        provider_openai: c["model_provider"] == "openai",
        approval: match c["approval_policy"].as_str() {
            Some("never") => Some(Approval::Never),
            Some("on-request") => Some(Approval::OnRequest),
            Some("untrusted") => Some(Approval::Untrusted),
            Some("on-failure") => Some(Approval::OnFailure),
            _ => None,
        },
        sandbox: match c["sandbox_mode"].as_str() {
            Some("read-only") => Some(Sandbox::ReadOnly),
            Some("workspace-write") => Some(Sandbox::WorkspaceWrite),
            Some("danger-full-access") => Some(Sandbox::DangerFullAccess),
            _ => None,
        },
        origins_present: value["origins"].is_object(),
        layers: layers(value, roles),
        requirements_empty: false,
    };
    emit("config", &projection, owner);
}
pub(crate) fn requirements_reply(value: &serde_json::Value, owner: u64) {
    if !tracing::enabled!(target: TARGET, tracing::Level::TRACE) {
        return;
    }
    emit(
        "requirements",
        &value.as_object().is_some_and(|o| {
            o.len() == 1 && o.get("requirements") == Some(&serde_json::Value::Null)
        }),
        owner,
    );
}
