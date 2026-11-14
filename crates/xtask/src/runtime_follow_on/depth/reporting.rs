//! The freeze's reporting proposal is checked across all roots before lower delivery.
use super::*;
use crate::workspace_mutation::PlannedMutation;

pub(super) fn append(
    workspace: &Path,
    freeze: &Freeze,
    record_path: &str,
    mutations: &mut Vec<PlannedMutation>,
) -> Result<(), Error> {
    let registry = freeze.captured.registry();
    let bundle =
        crate::support_matrix::preview_preparation(workspace, registry, &freeze.plan.record)
            .map_err(Error::Validation)?;
    let mut proposed = std::collections::BTreeMap::from([
        (
            record_path.into(),
            serde_json::to_string_pretty(&freeze.plan.record)
                .map_err(|e| Error::Internal(e.to_string()))?,
        ),
        (
            freeze.state_path.clone(),
            serde_json::to_string_pretty(&freeze.state)
                .map_err(|e| Error::Internal(e.to_string()))?,
        ),
    ]);
    for (path, after) in [
        (crate::support_matrix::JSON_OUTPUT_PATH, bundle.json),
        (crate::support_matrix::MARKDOWN_OUTPUT_PATH, bundle.markdown),
        (
            crate::support_matrix::AGENT_API_RUNTIME_SUPPORT_DATA_OUTPUT_PATH,
            bundle.runtime_support_data,
        ),
    ] {
        let before = Source::workspace(workspace)
            .map_err(Error::Validation)?
            .optional_bytes(path)
            .map_err(Error::Validation)?;
        if before.as_deref() != Some(after.as_bytes()) {
            mutations.push(match before {
                Some(before) => PlannedMutation::replace(path, before, after.as_bytes().to_vec()),
                None => PlannedMutation::create(path, after.as_bytes().to_vec()),
            });
        }
        proposed.insert(path.into(), after);
    }
    crate::depth_acceptance::check_reporting_preparation(workspace, registry, proposed)
        .map_err(Error::Validation)
}
