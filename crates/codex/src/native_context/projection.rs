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
    /// Native high-to-low layer stack from the owning `config/read`; null when not observed.
    #[serde(deserialize_with = "required_nullable")]
    pub layers: Option<Layers>,
    pub requirements_empty: bool,
}
/// Wire `ConfigLayerSource` type at the pinned revision; any other spelling is `Unknown`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayerClass {
    PackagedDefaults,
    Mdm,
    System,
    EnterpriseManaged,
    User,
    Project,
    SessionFlags,
    LegacyManagedFile,
    LegacyManagedMdm,
    Unknown,
}
/// One native layer without its path, version or values. `role_matches` compares the
/// layer's path with its assigned role in memory: system `/etc/codex/config.toml`, user
/// `<native home>/config.toml` with no profile, project `<cwd>/.codex`; flags have no path.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Layer {
    pub class: LayerClass,
    pub enabled: bool,
    pub empty: bool,
    pub role_matches: bool,
}
/// Finite layer/origin correspondence. This is the managed `config/read` input stack before
/// thread/turn overrides; it is not an effective-turn, loader or exec/fork-route observation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Layers {
    pub entries: Vec<Layer>,
    pub origin_classes: Vec<LayerClass>,
    pub origins_enabled: bool,
}
pub(crate) const MAX_LAYERS: usize = 16;
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThreadOperation {
    Start,
    Resume,
    Fork,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub enum ThreadApproval {
    Never,
    OnRequest,
    Untrusted,
    OnFailure,
    Granular {
        sandbox_approval: bool,
        rules: bool,
        mcp_elicitations: bool,
        request_permissions: bool,
        skill_approval: bool,
    },
}
/// Partial thread response, not an effective-turn, layer, resolver or loader verdict.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThreadSettings {
    pub operation: ThreadOperation,
    pub model_gpt5: bool,
    pub provider_openai: bool,
    pub service_tier_default: bool,
    #[serde(deserialize_with = "required_nullable")]
    pub approval: Option<ThreadApproval>,
    pub reviewer_user: bool,
    #[serde(deserialize_with = "required_nullable")]
    pub sandbox_mode: Option<Sandbox>,
    pub cwd_matches: bool,
    pub runtime_roots_match: bool,
    pub instructions_empty: bool,
}
