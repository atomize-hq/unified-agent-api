//! Private, passive diagnostics. Only finite projections reach tracing; never command Debug.
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, ffi::OsString, path::Path};

pub(crate) const TARGET: &str = "uaa_native_context_v1";
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Route {
    InitialExec,
    ResumeExec,
    AppServer,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Launch {
    pub route: Route,
    pub spawned: bool,
    pub binary_matches: bool,
    pub cwd_matches: bool,
    pub home_matches: bool,
    pub environment_known: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub model_gpt5: bool,
    pub provider_openai: bool,
    #[serde(deserialize_with = "required_nullable")]
    pub approval: Option<Approval>,
    #[serde(deserialize_with = "required_nullable")]
    pub sandbox: Option<Sandbox>,
    pub origins_present: bool,
    // Existing API requests includeLayers:false; never invent layer/turn closure.
    pub layers_complete: bool,
    pub requirements_empty: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Approval {
    Never,
    OnRequest,
    Untrusted,
    OnFailure,
    Granular,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Sandbox {
    ReadOnly,
    WorkspaceWrite,
    DangerFullAccess,
}

pub(crate) fn environment_known(env: &BTreeMap<OsString, OsString>) -> bool {
    env.iter().all(|(key, value)| {
        let (Some(key), Some(value)) = (key.to_str(), value.to_str()) else {
            return false;
        };
        match key {
            // Values stay in memory. Presence of an auth channel does not prove resolver identity.
            "OPENAI_API_KEY" | "CODEX_API_KEY" => !value.is_empty(),
            "HOME" | "CODEX_HOME" | "CODEX_BINARY" | "TMPDIR" | "XDG_CONFIG_HOME"
            | "XDG_DATA_HOME" | "XDG_CACHE_HOME" => Path::new(value).is_absolute(),
            "PATH" => !value.is_empty() && std::env::split_paths(value).all(|p| p.is_absolute()),
            "LANG" | "LC_ALL" => matches!(value, "C" | "C.UTF-8" | "en_US.UTF-8"),
            "TZ" => value == "UTC",
            "RUST_LOG" => value == "error",
            // Reviewed non-effect channels: no values or machine-specific identities retained.
            "SHLVL" | "_" | "PWD" | "OLDPWD" | "TERM" | "COLORTERM" | "NO_COLOR" => true,
            _ => false,
        }
    })
}

// A nullable field is still a required member of the finite projection.
pub fn required_nullable<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}
