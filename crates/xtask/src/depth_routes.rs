//! Explicit refusal for writers without an owning depth-admission path. Read-only callers do
//! not use these guards. Retained records matter even after a declaration/request is replaced.
use crate::{
    agent_registry::{AgentRegistry, REGISTRY_RELATIVE_PATH},
    runtime_advertising::source::Source,
};
use std::path::Path;

pub fn refuse_workspace(root: &Path, route: &str) -> Result<(), String> {
    let source = Source::workspace(root)?;
    if let Some(raw) = source.optional(REGISTRY_RELATIVE_PATH)? {
        let registry = AgentRegistry::parse_unenforced(&raw).map_err(|e| e.to_string())?;
        if registry.agents.iter().any(|a| a.support_depth.is_some()) {
            return Err(format!(
                "{route}: unsupported depth write (declared enrollment)"
            ));
        }
    }
    if source
        .paths("cli_manifests")?
        .iter()
        .any(|p| p.ends_with("/depth-record.json"))
    {
        return Err(format!(
            "{route}: unsupported depth write (retained record)"
        ));
    }
    Ok(())
}

pub fn refuse_manifest(root: &Path, route: &str) -> Result<(), String> {
    let source = Source::workspace(root)?;
    if source
        .paths("reports")?
        .iter()
        .any(|p| p.ends_with("/depth-record.json"))
    {
        return Err(format!(
            "{route}: unsupported depth write (retained record)"
        ));
    }
    for parent in root.canonicalize().map_err(|e| e.to_string())?.ancestors() {
        if parent.join(REGISTRY_RELATIVE_PATH).exists() || parent.join("cli_manifests").is_dir() {
            return refuse_workspace(parent, route);
        }
    }
    Ok(())
}

/// Resolve an output directory that may not exist yet without following a symlink.
pub fn refuse_destination(path: &Path, route: &str) -> Result<(), String> {
    let full = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join(path)
    };
    for ancestor in full.ancestors() {
        match std::fs::symlink_metadata(ancestor) {
            Ok(m) if m.is_dir() && !m.file_type().is_symlink() => {
                return refuse_manifest(ancestor, route)
            }
            Ok(_) => return Err("unsupported output ancestor type".into()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.to_string()),
        }
    }
    Err("unresolved output destination".into())
}
