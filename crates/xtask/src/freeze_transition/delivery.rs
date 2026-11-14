//! Convert the pure plan to the existing checked atomic delivery boundary.
use super::*;
use crate::workspace_mutation::{PlannedMutation, WorkspacePathJail};
use std::path::Path;

pub(crate) fn plan<I: WriterInputs>(
    workspace: &Path,
    inputs: &I,
) -> Result<Planned<I::Request>, String> {
    let outputs = inputs.render()?;
    let before = observe_outputs(workspace, &outputs.files)?;
    super::plan(&before, &ValidatedAcquisitionDelta::empty(), inputs)
}

fn observe_outputs(workspace: &Path, files: &[(String, Vec<u8>)]) -> Result<Tree, String> {
    let jail = WorkspacePathJail::new(workspace).map_err(|e| e.to_string())?;
    let mut before = Tree::new();
    for (path, _) in files {
        let full = jail.resolve(Path::new(path)).map_err(|e| e.to_string())?;
        let bytes = match std::fs::read(full) {
            Ok(bytes) => Some(bytes),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(error.to_string()),
        };
        if let Some(bytes) = bytes {
            #[cfg(unix)]
            let executable = {
                use std::os::unix::fs::PermissionsExt;
                std::fs::metadata(workspace.join(path))
                    .map_err(|e| e.to_string())?
                    .permissions()
                    .mode()
                    & 0o100
                    != 0
            };
            #[cfg(not(unix))]
            let executable = false;
            before.insert(
                path.clone(),
                Entry::file(if executable { "100755" } else { "100644" }, bytes),
            );
        }
    }
    Ok(before)
}

pub(crate) fn from_plan(plan: &Plan) -> Result<Vec<PlannedMutation>, String> {
    plan.operations
        .iter()
        .map(|(path, operation)| {
            let (before, after) = match operation {
                Operation::Create(after) => (None, Some(after)),
                Operation::Replace { before, after } => (Some(before), Some(after)),
                Operation::Preserve(entry) => (Some(entry), Some(entry)),
                Operation::Delete(before) => (Some(before), None),
            };
            // The filesystem writer admits only regular blobs. Never erase an object-kind or
            // executable-bit precondition while converting the exact planner entries.
            for entry in before.into_iter().chain(after) {
                if !entry.regular() {
                    return Err("freeze delivery requires regular blob outputs".into());
                }
            }
            if let (Some(before), Some(after)) = (before, after) {
                if before.mode != after.mode {
                    return Err("freeze delivery cannot change planned modes".into());
                }
            }
            let entry = after.or(before).ok_or("empty freeze operation")?;
            let mode = if entry.mode == "100755" { 0o755 } else { 0o644 };
            let mutation = match (before, after) {
                (None, Some(after)) => PlannedMutation::create(path, after.bytes.clone()),
                (Some(before), Some(after)) => {
                    PlannedMutation::replace(path, before.bytes.clone(), after.bytes.clone())
                }
                (Some(before), None) => PlannedMutation::remove(path, before.bytes.clone()),
                (None, None) => unreachable!(),
            };
            Ok(mutation.with_git_mode(mode))
        })
        .collect()
}

pub(crate) fn apply(workspace: &Path, plan: &Plan) -> Result<(), String> {
    let mutations = from_plan(plan)?;
    crate::workspace_mutation::apply_admitted_mutations(workspace, &mutations)
        .map_err(|e| e.to_string())?;
    let files = plan
        .operations
        .keys()
        .map(|path| (path.clone(), Vec::new()))
        .collect::<Vec<_>>();
    plan.require_equal(&observe_outputs(workspace, &files)?)
}
