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
    // A caller-supplied inner directory cannot establish a stopping boundary. Inspect every
    // enclosing retained scope, including standalone manifests and outer registered workspaces.
    for parent in root.ancestors().skip(1) {
        if parent.join(REGISTRY_RELATIVE_PATH).exists()
            || parent.join("cli_manifests").is_dir()
            || parent.join("reports").is_dir()
        {
            refuse_here(parent, route)?;
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

/// A file output can belong to a different enclosing scope or propose a new depth record.
/// Observe its destination without changing the path the owning writer will use.
pub fn refuse_file_destination(path: &Path, bytes: &[u8], route: &str) -> Result<(), String> {
    let mut full = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join(path)
    };
    // Inspect file aliases as well as directory aliases, including a dangling output link.
    for links in 0..=40 {
        let metadata = match std::fs::symlink_metadata(&full) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => break,
            Err(error) => return Err(error.to_string()),
        };
        if !metadata.file_type().is_symlink() {
            break;
        }
        if links == 40 {
            return Err("too many output symlinks".into());
        }
        let target = std::fs::read_link(&full).map_err(|e| e.to_string())?;
        full = if target.is_absolute() {
            target
        } else {
            full.parent().ok_or("output has no parent")?.join(target)
        };
    }
    let parent = full.parent().ok_or("output has no parent")?;
    let ancestor = parent
        .ancestors()
        .find(|p| p.is_dir())
        .ok_or("unresolved output destination")?;
    let observed_root = ancestor.canonicalize().map_err(|e| e.to_string())?;
    let relative = full.strip_prefix(ancestor).map_err(|e| e.to_string())?;
    let observed_parent = observed_root.join(relative.parent().ok_or("output has no parent")?);
    refuse_destination(&observed_parent, route)?;
    if full
        .file_name()
        .is_some_and(|name| name == "depth-record.json")
        || full.ends_with(REGISTRY_RELATIVE_PATH)
    {
        refuse_output(&observed_root, relative, bytes, route)?;
    }
    Ok(())
}
