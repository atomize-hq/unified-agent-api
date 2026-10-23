//! Workflow delivery data, emitted only after successful guarded planning/apply.
use super::*;
use std::fs::{self, OpenOptions};

pub(super) fn destination(workspace: &Path, path: &Path) -> Result<PathBuf, Error> {
    let parent = path.parent().unwrap_or(Path::new("."));
    let parent = fs::canonicalize(parent)
        .map_err(|e| Error::Validation(format!("delivery report parent: {e}")))?;
    let workspace =
        fs::canonicalize(workspace).map_err(|e| Error::Validation(format!("workspace: {e}")))?;
    if parent.starts_with(workspace) {
        return Err(Error::Validation(
            "delivery report must be outside the workspace".into(),
        ));
    }
    let name = path
        .file_name()
        .ok_or_else(|| Error::Validation("delivery report requires a filename".into()))?;
    let path = parent.join(name);
    if fs::symlink_metadata(&path).is_ok() {
        return Err(Error::Validation(
            "delivery report must be a new file".into(),
        ));
    }
    Ok(path)
}

pub(super) fn write(path: &Path, plan: &PreparePlan, applied: bool) -> Result<(), Error> {
    let files: Vec<_> = plan
        .files
        .iter()
        .map(|f| {
            serde_json::json!({
                "path": f.relative_path,
                "sha256": hex::encode(Sha256::digest(&f.contents)),
            })
        })
        .collect();
    let value = serde_json::json!({"applied": applied, "files": files});
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| Error::Internal(format!("create delivery report: {e}")))?;
    writeln!(
        file,
        "{}",
        serde_json::to_string_pretty(&value)
            .map_err(|e| Error::Internal(format!("serialize delivery report: {e}")))?
    )
    .map_err(|e| Error::Internal(format!("write delivery report: {e}")))
}
