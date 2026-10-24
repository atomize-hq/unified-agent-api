//! Explicit refusal for writers without an owning depth-admission path. Read-only callers do
//! not use these guards. Retained records matter even after a declaration/request is replaced.
use crate::{
    agent_registry::{AgentRegistry, REGISTRY_RELATIVE_PATH},
    runtime_advertising::source::Source,
};
use std::path::Path;

pub fn refuse_workspace(root: &Path, route: &str) -> Result<(), String> {
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    refuse_here(&root, route)?;
    // Public writers accept a directory, not proof that it is the authority root. Standalone
    // manifest reports also anchor retained scope. Stop at a real enclosing workspace so unrelated
    // directories above that workspace cannot supply authority or hide its records.
    let anchored =
        |p: &Path| p.join(REGISTRY_RELATIVE_PATH).exists() || p.join("cli_manifests").is_dir();
    if anchored(&root) {
        return Ok(());
    }
    for parent in root.ancestors().skip(1) {
        if anchored(parent) || parent.join("reports").is_dir() {
            refuse_here(parent, route)?;
        }
        if anchored(parent) {
            break;
        }
    }
    Ok(())
}
fn refuse_here(root: &Path, route: &str) -> Result<(), String> {
    let source = Source::workspace(root)?;
    if let Some(raw) = source.optional(REGISTRY_RELATIVE_PATH)? {
        refuse_registry(&raw, route)?;
    }
    for prefix in ["cli_manifests", "reports"] {
        if source
            .paths(prefix)?
            .iter()
            .any(|p| p.ends_with("/depth-record.json"))
        {
            return Err(format!(
                "{route}: unsupported depth write (retained record)"
            ));
        }
    }
    Ok(())
}
fn refuse_registry(raw: &str, route: &str) -> Result<(), String> {
    let registry = AgentRegistry::parse_unenforced(raw).map_err(|e| e.to_string())?;
    if registry.agents.iter().any(|a| a.support_depth.is_some()) {
        return Err(format!(
            "{route}: unsupported depth write (declared enrollment)"
        ));
    }
    Ok(())
}
pub fn refuse_manifest(root: &Path, route: &str) -> Result<(), String> {
    refuse_workspace(root, route)
}

/// Also inspect destinations and candidate authority before a neutral writer creates any file.
pub fn refuse_output(
    root: &Path,
    relative: &Path,
    bytes: &[u8],
    route: &str,
) -> Result<(), String> {
    let full = crate::workspace_mutation::WorkspacePathJail::new(root)
        .map_err(|e| e.to_string())?
        .resolve(relative)
        .map_err(|e| e.to_string())?;
    if full.file_name().is_some_and(|n| n == "depth-record.json")
        && full
            .parent()
            .and_then(Path::parent)
            .and_then(Path::file_name)
            .is_some_and(|n| n == "reports")
    {
        return Err(format!(
            "{route}: unsupported depth write (proposed depth record)"
        ));
    }
    if full.ends_with(REGISTRY_RELATIVE_PATH) {
        refuse_registry(
            std::str::from_utf8(bytes).map_err(|e| e.to_string())?,
            route,
        )?;
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
