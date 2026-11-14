//! The freeze's reporting proposal is checked across all roots before lower delivery.
use super::*;

pub(super) fn render(
    record: &crate::depth_record::DepthRecord,
    state: &LifecycleState,
    state_path: &str,
    record_path: &str,
    bundle: crate::support_matrix::PublicationBundle,
) -> Result<Vec<(String, Vec<u8>)>, Error> {
    let proposed = std::collections::BTreeMap::from([
        (
            record_path.into(),
            serde_json::to_string_pretty(record).map_err(|e| Error::Internal(e.to_string()))?,
        ),
        (
            state_path.into(),
            serde_json::to_string_pretty(state).map_err(|e| Error::Internal(e.to_string()))?,
        ),
        (crate::support_matrix::JSON_OUTPUT_PATH.into(), bundle.json),
        (
            crate::support_matrix::MARKDOWN_OUTPUT_PATH.into(),
            bundle.markdown,
        ),
        (
            crate::support_matrix::AGENT_API_RUNTIME_SUPPORT_DATA_OUTPUT_PATH.into(),
            bundle.runtime_support_data,
        ),
    ]);
    Ok(proposed
        .into_iter()
        .map(|(path, bytes)| (path, bytes.into_bytes()))
        .collect())
}
