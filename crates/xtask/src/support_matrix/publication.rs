use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

use super::{
    SupportRow, GENERATED_END_MARKER, GENERATED_SECTION_NOTE, GENERATED_SECTION_TITLE,
    GENERATED_START_MARKER, JSON_OUTPUT_PATH, MARKDOWN_OUTPUT_PATH,
};

#[derive(Debug, Clone)]
pub(super) struct PublicationBundle {
    pub(super) json: String,
    pub(super) markdown: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SupportMatrixArtifact {
    pub(crate) schema_version: u8,
    pub(crate) rows: Vec<SupportRow>,
}

pub(super) fn render_publication_bundle(rows: &[SupportRow]) -> Result<PublicationBundle, String> {
    let json = serde_json::to_string_pretty(&SupportMatrixArtifact {
        schema_version: 1,
        rows: rows.to_vec(),
    })
    .map_err(|err| format!("serialize support-matrix json: {err}"))?;
    let markdown = render_markdown_projection(rows);

    Ok(PublicationBundle {
        json: format!("{json}\n"),
        markdown,
    })
}

fn render_markdown_projection(rows: &[SupportRow]) -> String {
    let mut out = String::new();
    let mut current_agent: Option<&str> = None;

    for row in rows {
        if current_agent != Some(row.agent.as_str()) {
            if current_agent.is_some() {
                out.push('\n');
            }
            current_agent = Some(row.agent.as_str());
            out.push_str(&format!("### `{}`\n\n", row.agent));
            out.push_str("| agent | version | target | manifest_support | backend_support | uaa_support | pointer_promotion | evidence_notes |\n");
            out.push_str("|---|---|---|---|---|---|---|---|\n");
        }

        let notes = if row.evidence_notes.is_empty() {
            "—".to_string()
        } else {
            row.evidence_notes.join("; ")
        };

        out.push_str(&format!(
            "| `{}` | `{}` | `{}` | `{}` | `{}` | `{}` | `{}` | {} |\n",
            row.agent,
            row.version,
            row.target,
            row.manifest_support.as_str(),
            row.backend_support.as_str(),
            row.uaa_support.as_str(),
            row.pointer_promotion.as_str(),
            notes
        ));
    }

    out
}

pub(super) fn write_publication_artifacts(
    workspace_root: &Path,
    bundle: &PublicationBundle,
) -> Result<(), String> {
    let json_path = workspace_root.join(JSON_OUTPUT_PATH);
    let markdown_path = workspace_root.join(MARKDOWN_OUTPUT_PATH);
    write_file(&json_path, &bundle.json)?;

    let existing_markdown = fs::read_to_string(&markdown_path)
        .map_err(|err| format!("read({}): {err}", markdown_path.display()))?;
    let updated_markdown = splice_markdown_projection(&existing_markdown, &bundle.markdown);
    write_file(&markdown_path, &updated_markdown)?;
    Ok(())
}

pub(super) fn validate_publication_artifacts(
    workspace_root: &Path,
    bundle: &PublicationBundle,
) -> Result<(), String> {
    let json_path = workspace_root.join(JSON_OUTPUT_PATH);
    let markdown_path = workspace_root.join(MARKDOWN_OUTPUT_PATH);

    let checked_in_json: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&json_path)
            .map_err(|err| format!("read({}): {err}", json_path.display()))?,
    )
    .map_err(|err| format!("parse({}): {err}", json_path.display()))?;
    let generated_json: serde_json::Value = serde_json::from_str(&bundle.json)
        .map_err(|err| format!("parse generated support-matrix json: {err}"))?;
    if checked_in_json != generated_json {
        return Err(format!(
            "{} is stale; regenerate with `cargo run -p xtask -- support-matrix`",
            json_path.display()
        ));
    }

    let checked_in_markdown = fs::read_to_string(&markdown_path)
        .map_err(|err| format!("read({}): {err}", markdown_path.display()))?;
    let checked_in_block = extract_generated_markdown_block(&checked_in_markdown)?;
    let generated_block = format!(
        "{GENERATED_START_MARKER}\n{}{}",
        bundle.markdown, GENERATED_END_MARKER
    );
    if checked_in_block != generated_block {
        return Err(format!(
            "{} generated block is stale; regenerate with `cargo run -p xtask -- support-matrix`",
            markdown_path.display()
        ));
    }

    Ok(())
}

fn extract_generated_markdown_block(existing: &str) -> Result<&str, String> {
    let start = existing.find(GENERATED_START_MARKER).ok_or_else(|| {
        format!("missing support-matrix generated block start marker ({GENERATED_START_MARKER})")
    })?;
    let end = existing[start..]
        .find(GENERATED_END_MARKER)
        .map(|offset| start + offset)
        .ok_or_else(|| {
            format!("missing support-matrix generated block end marker ({GENERATED_END_MARKER})")
        })?;

    if end < start {
        return Err("support-matrix generated block markers are out of order".to_string());
    }

    Ok(&existing[start..end + GENERATED_END_MARKER.len()])
}

fn write_file(path: &Path, contents: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("create_dir_all({}): {err}", parent.display()))?;
    }
    fs::write(path, contents).map_err(|err| format!("write({}): {err}", path.display()))
}

fn splice_markdown_projection(existing: &str, projection: &str) -> String {
    let generated_block = format!("{GENERATED_START_MARKER}\n{projection}{GENERATED_END_MARKER}");

    match (
        existing.find(GENERATED_START_MARKER),
        existing.find(GENERATED_END_MARKER),
    ) {
        (Some(start), Some(end)) if start <= end => {
            let before = &existing[..start];
            let after = &existing[end + GENERATED_END_MARKER.len()..];
            format!("{before}{generated_block}{after}")
        }
        _ => {
            let trimmed = existing.trim_end();
            format!(
                "{trimmed}\n\n{GENERATED_SECTION_TITLE}\n\n{GENERATED_SECTION_NOTE}\n\n{generated_block}\n"
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::support_matrix::{
        BackendSupportState, ManifestSupportState, PointerPromotionState, UaaSupportState,
    };

    fn sample_rows() -> Vec<SupportRow> {
        vec![
            SupportRow {
                agent: "codex".to_string(),
                version: "1.0.0".to_string(),
                target: "linux-x64".to_string(),
                manifest_support: ManifestSupportState::Supported,
                backend_support: BackendSupportState::Partial,
                uaa_support: UaaSupportState::Partial,
                pointer_promotion: PointerPromotionState::LatestValidated,
                evidence_notes: vec![
                    "backend report includes backend-only surface outside unified support"
                        .to_string(),
                ],
            },
            SupportRow {
                agent: "codex".to_string(),
                version: "0.9.0".to_string(),
                target: "darwin-arm64".to_string(),
                manifest_support: ManifestSupportState::Unsupported,
                backend_support: BackendSupportState::Unsupported,
                uaa_support: UaaSupportState::Unsupported,
                pointer_promotion: PointerPromotionState::None,
                evidence_notes: vec!["current root snapshot omits this target".to_string()],
            },
        ]
    }

    #[test]
    fn markdown_splice_preserves_normative_contract_text() {
        let existing = "# Spec\n\n## Purpose\nManual contract.\n";
        let projection = "### `codex`\n\n| agent | version | target | manifest_support | backend_support | uaa_support | pointer_promotion | evidence_notes |\n|---|---|---|---|---|---|---|---|\n";

        let updated = splice_markdown_projection(existing, projection);

        assert!(updated.contains("## Purpose\nManual contract."));
        assert!(updated.contains(GENERATED_SECTION_TITLE));
        assert!(updated.contains(GENERATED_START_MARKER));
        assert!(updated.contains("### `codex`"));
    }

    #[test]
    fn publication_bundle_uses_same_rows_for_json_and_markdown() {
        let rows = sample_rows();
        let bundle = render_publication_bundle(&rows).expect("render publication bundle");

        let artifact: SupportMatrixArtifact =
            serde_json::from_str(&bundle.json).expect("parse generated support-matrix json");
        assert_eq!(artifact.schema_version, 1);
        assert_eq!(artifact.rows, rows);

        let expected_markdown = "\
### `codex`\n\
\n\
| agent | version | target | manifest_support | backend_support | uaa_support | pointer_promotion | evidence_notes |\n\
|---|---|---|---|---|---|---|---|\n\
| `codex` | `1.0.0` | `linux-x64` | `supported` | `partial` | `partial` | `latest_validated` | backend report includes backend-only surface outside unified support |\n\
| `codex` | `0.9.0` | `darwin-arm64` | `unsupported` | `unsupported` | `unsupported` | `none` | current root snapshot omits this target |\n";
        assert_eq!(bundle.markdown, expected_markdown);
    }
}
