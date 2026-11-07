//! Record-free reporting must equal a fresh derivation; source and capability changes are
//! otherwise outside the finite checkpoint's allowed paths.
use crate::support_matrix;
use std::{fs, path::Path};

pub(super) fn publication(root: &Path, parent: &str) -> Result<(), String> {
    let before = crate::runtime_advertising::source::Source::tip(root, parent)?;
    let markdown = before.read(support_matrix::MARKDOWN_OUTPUT_PATH)?;
    let bundle = support_matrix::generate_publication_artifacts_from_markdown(root, &markdown)?;
    for (path, expected) in [
        (support_matrix::JSON_OUTPUT_PATH, bundle.json),
        (support_matrix::MARKDOWN_OUTPUT_PATH, bundle.markdown),
        (
            support_matrix::AGENT_API_RUNTIME_SUPPORT_DATA_OUTPUT_PATH,
            bundle.runtime_support_data,
        ),
    ] {
        if fs::read(root.join(path)).map_err(|e| e.to_string())? != expected.as_bytes() {
            return Err(format!(
                "bootstrap reporting publication is incoherent: {path}"
            ));
        }
    }
    let expected = crate::capability_matrix::generate_markdown_for_workspace(root)?;
    if fs::read(root.join(crate::capability_matrix::DEFAULT_OUT_PATH)).map_err(|e| e.to_string())?
        != expected.as_bytes()
    {
        return Err("bootstrap capability publication is incoherent".into());
    }
    Ok(())
}
