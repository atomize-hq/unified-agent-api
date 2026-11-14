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

pub(crate) fn from_plan(plan: Plan) -> Vec<PlannedMutation> {
    plan.operations
        .into_iter()
        .map(|(path, operation)| match operation {
            Operation::Create(after) => PlannedMutation::create(path, after.bytes),
            Operation::Replace { before, after } => {
                PlannedMutation::replace(path, before.bytes, after.bytes)
            }
            Operation::Preserve(after) => {
                PlannedMutation::replace(path, after.bytes.clone(), after.bytes)
            }
            Operation::Delete(before) => PlannedMutation::remove(path, before.bytes),
        })
        .collect()
}
