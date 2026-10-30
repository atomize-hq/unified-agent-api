//! Producer-only local checks. Never read, inventory, hash or copy authentication stores.
use super::environment_known;
mod assets;
use std::{
    collections::BTreeMap,
    ffi::OsString,
    path::{Component, Path, PathBuf},
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    Environment,
    PathRole,
    ForeignLoader,
    Config,
    InputDrift,
}
impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "native context refused: {}",
            match self {
                Self::Environment => "environment",
                Self::PathRole => "path_role",
                Self::ForeignLoader => "foreign_loader",
                Self::Config => "config",
                Self::InputDrift => "input_drift",
            }
        )
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Stamp {
    len: u64,
    readonly: bool,
    identity: (u64, u64, u32),
}
fn stamp(meta: &std::fs::Metadata) -> Stamp {
    #[cfg(unix)]
    let identity = {
        use std::os::unix::fs::MetadataExt;
        (meta.dev(), meta.ino(), meta.mode())
    };
    #[cfg(not(unix))]
    let identity = (0, 0, 0);
    Stamp {
        len: meta.len(),
        readonly: meta.permissions().readonly(),
        identity,
    }
}
fn directory_stamp(path: &Path) -> Result<Stamp, Refusal> {
    let mut value = stamp(&std::fs::symlink_metadata(path).map_err(|_| Refusal::PathRole)?);
    value.len = 0; // Native state creation changes directory size, not its identity.
    Ok(value)
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalGuard {
    home: PathBuf,
    project: PathBuf,
    system: PathBuf,
    config: Option<(serde_json::Value, Stamp)>,
    home_stamp: Stamp,
    project_stamp: Stamp,
}
pub fn role(path: &Path) -> Result<PathBuf, Refusal> {
    if !path.is_absolute()
        || path
            .components()
            .any(|c| matches!(c, Component::CurDir | Component::ParentDir))
    {
        return Err(Refusal::PathRole);
    }
    // components() removes interior './'; lexical equality catches those aliases too.
    let rebuilt: PathBuf = path.components().collect();
    if rebuilt.as_os_str() != path.as_os_str() {
        return Err(Refusal::PathRole);
    }
    let mut prefix = PathBuf::new();
    for c in path.components() {
        prefix.push(c);
        let m = std::fs::symlink_metadata(&prefix).map_err(|_| Refusal::PathRole)?;
        if m.is_symlink() {
            return Err(Refusal::PathRole);
        }
    }
    let canonical = path.canonicalize().map_err(|_| Refusal::PathRole)?;
    if canonical != path || !canonical.is_dir() {
        return Err(Refusal::PathRole);
    }
    Ok(canonical)
}
fn config(home: &Path) -> Result<Option<(serde_json::Value, Stamp)>, Refusal> {
    use std::io::Read;
    let path = home.join("config.toml");
    let meta = match std::fs::symlink_metadata(&path) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(Refusal::Config),
    };
    if !meta.is_file() || meta.is_symlink() || meta.len() > 1_048_576 {
        return Err(Refusal::Config);
    }
    let mut file = std::fs::File::open(&path).map_err(|_| Refusal::Config)?;
    let before = stamp(&meta);
    if stamp(&file.metadata().map_err(|_| Refusal::Config)?) != before {
        return Err(Refusal::InputDrift);
    }
    let mut bytes = String::new();
    file.by_ref()
        .take(1_048_577)
        .read_to_string(&mut bytes)
        .map_err(|_| Refusal::Config)?;
    if stamp(&file.metadata().map_err(|_| Refusal::Config)?) != before
        || stamp(&std::fs::symlink_metadata(&path).map_err(|_| Refusal::Config)?) != before
    {
        return Err(Refusal::InputDrift);
    }
    if bytes.len() > 1_048_576 {
        return Err(Refusal::Config);
    }
    let v: serde_json::Value = toml_edit::de::from_str(&bytes).map_err(|_| Refusal::Config)?;
    let o = v.as_object().ok_or(Refusal::Config)?;
    for (key, value) in o {
        let allowed = match key.as_str() {
            "model" => value == "gpt-5",
            "model_provider" => value == "openai",
            "approval_policy" => matches!(
                value.as_str(),
                Some("never" | "on-request" | "untrusted" | "on-failure")
            ),
            "sandbox_mode" => matches!(
                value.as_str(),
                Some("read-only" | "workspace-write" | "danger-full-access")
            ),
            "web_search" => value == "disabled",
            "features" => assets::features(value),
            "analytics" => value == &serde_json::json!({"enabled":false}),
            // Explicitly bound auth store choice is a nonsecret intent, not resolver proof.
            "cli_auth_credentials_store" => matches!(
                value.as_str(),
                Some("file" | "keyring" | "auto" | "ephemeral")
            ),
            _ => false,
        };
        if !allowed {
            return Err(Refusal::Config);
        }
    }
    Ok(Some((v, before)))
}
fn loaders(
    home: &Path,
    project: &Path,
    system: &Path,
    env: &BTreeMap<OsString, OsString>,
) -> Result<(), Refusal> {
    let present = |p: PathBuf| match std::fs::symlink_metadata(p) {
        Ok(_) => true,
        Err(e) => e.kind() != std::io::ErrorKind::NotFound,
    };
    for dir in project.ancestors().chain(home.ancestors()) {
        for name in ["AGENTS.md", "AGENTS.override.md", ".codex", ".agents"] {
            let p = dir.join(name);
            if p == home {
                continue;
            }
            if present(p) {
                return Err(Refusal::ForeignLoader);
            }
        }
    }
    for name in [
        "AGENTS.md",
        "AGENTS.override.md",
        "hooks",
        "plugins",
        "mcp.json",
    ] {
        if present(home.join(name)) {
            return Err(Refusal::ForeignLoader);
        }
    }
    // CODEX_HOME does not imply that OS-home/XDG loader inputs are absent.
    for key in ["HOME", "XDG_CONFIG_HOME", "XDG_DATA_HOME"] {
        if let Some(root) = env.get(std::ffi::OsStr::new(key)).map(PathBuf::from) {
            for name in ["AGENTS.md", "AGENTS.override.md", ".agents", "codex"] {
                if present(root.join(name)) {
                    return Err(Refusal::ForeignLoader);
                }
            }
            let native = root.join(".codex");
            if native != home && present(native) {
                return Err(Refusal::ForeignLoader);
            }
        }
    }
    assets::bundled(home)?;
    if present(system.to_path_buf()) {
        return Err(Refusal::ForeignLoader);
    }
    Ok(())
}
pub fn pinned_binary_matches(digest: &str) -> bool {
    assets::binary_matches(digest)
}
impl LocalGuard {
    pub fn inspect(
        home: &Path,
        project: &Path,
        env: &BTreeMap<OsString, OsString>,
    ) -> Result<Self, Refusal> {
        Self::inspect_with_system(home, project, env, Path::new("/etc/codex"))
    }
    pub(super) fn inspect_with_system(
        home: &Path,
        project: &Path,
        env: &BTreeMap<OsString, OsString>,
        system: &Path,
    ) -> Result<Self, Refusal> {
        if !environment_known(env) {
            return Err(Refusal::Environment);
        }
        let home = role(home)?;
        let project = role(project)?;
        if home.starts_with(&project) || project.starts_with(&home) {
            return Err(Refusal::PathRole);
        }
        loaders(&home, &project, system, env)?;
        let config = config(&home)?;
        let home_stamp = directory_stamp(&home)?;
        let project_stamp = directory_stamp(&project)?;
        Ok(Self {
            home,
            project,
            system: system.into(),
            config,
            home_stamp,
            project_stamp,
        })
    }
    pub fn home(&self) -> &Path {
        &self.home
    }
    pub fn recheck(&self, env: &BTreeMap<OsString, OsString>) -> Result<(), Refusal> {
        if *self != Self::inspect_with_system(&self.home, &self.project, env, &self.system)? {
            return Err(Refusal::InputDrift);
        }
        Ok(())
    }
}
