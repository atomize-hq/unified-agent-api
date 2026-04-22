use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use crate::{
    agent_registry::AgentRegistryEntry,
    support_matrix::{
        BackendSupportState, ManifestSupportState, PointerPromotionState, SupportRow,
        UaaSupportState,
    },
};
use agent_api::{
    backends::{
        claude_code::{ClaudeCodeBackend, ClaudeCodeBackendConfig},
        codex::{CodexBackend, CodexBackendConfig},
        gemini_cli::{GeminiCliBackend, GeminiCliBackendConfig},
        opencode::{OpencodeBackend, OpencodeBackendConfig},
    },
    AgentWrapperBackend,
};
use serde::Deserialize;

use super::{
    CAPABILITY_EXTERNAL_SANDBOX_V1, CAPABILITY_MCP_ADD_V1, CAPABILITY_MCP_GET_V1,
    CAPABILITY_MCP_LIST_V1, CAPABILITY_MCP_REMOVE_V1, RELEASE_DOC_END_MARKER,
    RELEASE_DOC_START_MARKER, SUPPORT_MARKDOWN_END_MARKER, SUPPORT_MARKDOWN_START_MARKER,
    SUPPORT_MATRIX_MARKDOWN_PATH,
};

pub(super) fn collect_capability_truth(
    entry: &AgentRegistryEntry,
    workspace_root: &Path,
) -> Result<BTreeSet<String>, String> {
    let manifest_path = workspace_root
        .join(&entry.manifest_root)
        .join("current.json");
    let manifest: ManifestCurrent = read_json(&manifest_path)?;
    let mut capabilities = runtime_capabilities(&entry.agent_id)?;

    match entry.agent_id.as_str() {
        "codex" | "claude_code" => {
            capabilities.insert(CAPABILITY_EXTERNAL_SANDBOX_V1.to_string());
            capabilities.insert(CAPABILITY_MCP_ADD_V1.to_string());
            capabilities.insert(CAPABILITY_MCP_REMOVE_V1.to_string());
        }
        "gemini_cli" | "opencode" => {}
        other => {
            return Err(format!(
                "capability truth does not support backend `{other}`"
            ));
        }
    }

    if known_mcp_capability_ids()
        .iter()
        .any(|capability_id| declared_mcp_projection(entry, capability_id).is_some())
    {
        let primary_target = entry.canonical_targets.first().ok_or_else(|| {
            format!(
                "registry entry `{}` has no canonical targets",
                entry.agent_id
            )
        })?;
        if !manifest
            .expected_targets
            .iter()
            .any(|target| target == primary_target)
        {
            return Err(format!(
                "{} missing primary canonical target `{primary_target}` in expected_targets",
                manifest_path.display()
            ));
        }

        for capability_id in known_mcp_capability_ids() {
            let declaration = declared_mcp_projection(entry, capability_id);
            let available_on_primary_target = declaration.as_ref().is_some_and(|projection| {
                projection.supports_target(primary_target)
                    && command_available_on_target(
                        &manifest,
                        mcp_command_path(capability_id),
                        primary_target,
                    )
            });
            let advertise_when_available = declaration
                .as_ref()
                .is_some_and(|projection| projection.advertise_when_available);

            sync_capability_from_manifest(
                &mut capabilities,
                capability_id,
                available_on_primary_target,
                advertise_when_available,
            );
        }
    }

    Ok(capabilities)
}

pub(super) fn parse_capability_matrix_agent_support(
    path: &Path,
    agent_id: &str,
) -> Result<BTreeSet<String>, String> {
    let text =
        fs::read_to_string(path).map_err(|err| format!("read({}): {err}", path.display()))?;
    let mut supported = BTreeSet::new();
    let mut found_agent_column = false;
    let mut lines = text.lines().peekable();

    while let Some(line) = lines.next() {
        if !line.starts_with("| capability id |") {
            continue;
        }

        let headers = parse_markdown_cells(line);
        let agent_column = headers.iter().position(|header| *header == agent_id);
        let Some(agent_column) = agent_column else {
            let _ = lines.next();
            while let Some(candidate) = lines.peek() {
                if !candidate.starts_with('|') {
                    break;
                }
                let _ = lines.next();
            }
            continue;
        };
        found_agent_column = true;

        let _ = lines.next();
        while let Some(candidate) = lines.peek() {
            if !candidate.starts_with('|') {
                break;
            }
            let row = parse_markdown_cells(candidate);
            if row.len() <= agent_column {
                return Err(format!(
                    "{} contains a malformed capability row for `{agent_id}`",
                    path.display()
                ));
            }
            if row[agent_column].contains('✅') {
                supported.insert(row[0].to_string());
            }
            let _ = lines.next();
        }
    }

    if found_agent_column {
        Ok(supported)
    } else {
        Err(format!(
            "{} does not publish a capability column for `{agent_id}`",
            path.display()
        ))
    }
}

pub(super) fn render_support_markdown_section(rows: &[SupportRow]) -> String {
    if rows.is_empty() {
        return String::new();
    }

    let mut out = String::new();
    out.push_str(&format!("### `{}`\n\n", rows[0].agent));
    out.push_str("| agent | version | target | manifest_support | backend_support | uaa_support | pointer_promotion | evidence_notes |\n");
    out.push_str("|---|---|---|---|---|---|---|---|\n");
    for row in rows {
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
            manifest_support_str(row.manifest_support),
            backend_support_str(row.backend_support),
            uaa_support_str(row.uaa_support),
            pointer_promotion_str(row.pointer_promotion),
            notes
        ));
    }
    out
}

pub(super) fn extract_support_markdown_section(
    markdown: &str,
    agent_id: &str,
) -> Result<String, String> {
    let start = markdown
        .find(SUPPORT_MARKDOWN_START_MARKER)
        .ok_or_else(|| {
            format!(
            "missing support-matrix generated block start marker ({SUPPORT_MARKDOWN_START_MARKER})"
        )
        })?;
    let end = markdown.find(SUPPORT_MARKDOWN_END_MARKER).ok_or_else(|| {
        format!("missing support-matrix generated block end marker ({SUPPORT_MARKDOWN_END_MARKER})")
    })?;
    let generated_block = &markdown[start + SUPPORT_MARKDOWN_START_MARKER.len()..end];
    let header = format!("### `{agent_id}`\n");
    let Some(section_start_offset) = generated_block.find(&header) else {
        return Err(format!(
            "{} does not publish a generated section for `{agent_id}`",
            SUPPORT_MATRIX_MARKDOWN_PATH
        ));
    };
    let rest = &generated_block[section_start_offset..];
    let section_end = rest
        .find("\n### `")
        .map(|offset| section_start_offset + offset + 1)
        .unwrap_or_else(|| generated_block.len());
    Ok(generated_block[section_start_offset..section_end]
        .trim_start_matches('\n')
        .to_string())
}

pub(super) fn parse_release_doc_packages(text: &str) -> Result<ReleaseDocPackages, String> {
    let start = text
        .find(RELEASE_DOC_START_MARKER)
        .ok_or_else(|| format!("missing release doc start marker ({RELEASE_DOC_START_MARKER})"))?;
    let end = text
        .find(RELEASE_DOC_END_MARKER)
        .ok_or_else(|| format!("missing release doc end marker ({RELEASE_DOC_END_MARKER})"))?;
    let block = &text[start + RELEASE_DOC_START_MARKER.len()..end];

    let published_crates = extract_markdown_code_list(block, "## Published crates")?;
    let publish_order = extract_markdown_code_list(block, "## Publish order")?;

    Ok(ReleaseDocPackages {
        published_crates,
        publish_order,
    })
}

pub(super) fn extract_marked_block(
    text: &str,
    start_marker: &str,
    end_marker: &str,
) -> Result<String, String> {
    let start = text
        .find(start_marker)
        .ok_or_else(|| format!("missing governance block start marker ({start_marker})"))?;
    let rest = &text[start + start_marker.len()..];
    let end = rest
        .find(end_marker)
        .ok_or_else(|| format!("missing governance block end marker ({end_marker})"))?;
    Ok(rest[..end].trim().to_string())
}

pub(super) fn inline_code_ids(text: &str) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    let mut remaining = text;
    while let Some(start) = remaining.find('`') {
        remaining = &remaining[start + 1..];
        let Some(end) = remaining.find('`') else {
            break;
        };
        let candidate = &remaining[..end];
        if candidate.contains('.') {
            ids.insert(candidate.to_string());
        }
        remaining = &remaining[end + 1..];
    }
    ids
}

pub(super) fn parse_support_state_lines(text: &str) -> Result<BTreeMap<String, String>, String> {
    let mut states = BTreeMap::new();
    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            return Err(format!(
                "governance support block must use `key = value` lines (got `{line}`)"
            ));
        };
        let key = key.trim();
        let value = value.trim();
        if key.is_empty() || value.is_empty() {
            return Err(format!(
                "governance support block must not contain blank keys or values (got `{line}`)"
            ));
        }
        if states.insert(key.to_string(), value.to_string()).is_some() {
            return Err(format!(
                "governance support block contains duplicate key `{key}`"
            ));
        }
    }

    if states.is_empty() {
        return Err("governance support block must not be empty".to_string());
    }

    Ok(states)
}

pub(super) fn build_surfaces<const N: usize>(
    workspace_root: &Path,
    paths: [PathBuf; N],
) -> Vec<String> {
    paths
        .iter()
        .map(|path| path_to_repo_relative(workspace_root, path))
        .collect()
}

pub(super) fn path_to_repo_relative(workspace_root: &Path, path: impl AsRef<Path>) -> String {
    let path = path.as_ref();
    path.strip_prefix(workspace_root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

pub(super) fn read_json<T>(path: &Path) -> Result<T, String>
where
    T: for<'de> Deserialize<'de>,
{
    let text =
        fs::read_to_string(path).map_err(|err| format!("read({}): {err}", path.display()))?;
    serde_json::from_str(&text).map_err(|err| format!("parse({}): {err}", path.display()))
}

#[derive(Debug, Deserialize)]
pub(super) struct ManifestCurrent {
    #[serde(default)]
    pub expected_targets: Vec<String>,
    #[serde(default)]
    pub commands: Vec<ManifestCommand>,
}

#[derive(Debug, Deserialize)]
pub(super) struct ManifestCommand {
    pub path: Vec<String>,
    #[serde(default)]
    pub available_on: Vec<String>,
}

#[derive(Debug)]
pub(super) struct ReleaseDocPackages {
    pub published_crates: Vec<String>,
    pub publish_order: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
struct DeclaredMcpProjection<'a> {
    targets: Option<&'a [String]>,
    advertise_when_available: bool,
}

impl DeclaredMcpProjection<'_> {
    fn supports_target(&self, target: &str) -> bool {
        match self.targets {
            Some(targets) => targets.iter().any(|candidate| candidate == target),
            None => true,
        }
    }
}

fn runtime_capabilities(agent_id: &str) -> Result<BTreeSet<String>, String> {
    let ids = match agent_id {
        "codex" => {
            CodexBackend::new(CodexBackendConfig::default())
                .capabilities()
                .ids
        }
        "claude_code" => {
            ClaudeCodeBackend::new(ClaudeCodeBackendConfig::default())
                .capabilities()
                .ids
        }
        "gemini_cli" => {
            GeminiCliBackend::new(GeminiCliBackendConfig::default())
                .capabilities()
                .ids
        }
        "opencode" => {
            OpencodeBackend::new(OpencodeBackendConfig::default())
                .capabilities()
                .ids
        }
        other => {
            return Err(format!(
                "capability truth does not support backend `{other}`"
            ));
        }
    };

    Ok(ids.into_iter().collect())
}

fn declared_mcp_projection<'a>(
    entry: &'a AgentRegistryEntry,
    capability_id: &str,
) -> Option<DeclaredMcpProjection<'a>> {
    if let Some(target_gated) = entry
        .capability_declaration
        .target_gated
        .iter()
        .find(|declaration| declaration.capability_id == capability_id)
    {
        return Some(DeclaredMcpProjection {
            targets: Some(target_gated.targets.as_slice()),
            advertise_when_available: true,
        });
    }

    entry
        .capability_declaration
        .config_gated
        .iter()
        .find(|declaration| declaration.capability_id == capability_id)
        .map(|config_gated| DeclaredMcpProjection {
            targets: config_gated.targets.as_deref(),
            advertise_when_available: false,
        })
}

fn command_available_on_target(manifest: &ManifestCurrent, path: &[&str], target: &str) -> bool {
    manifest
        .commands
        .iter()
        .find(|command| {
            command
                .path
                .iter()
                .map(String::as_str)
                .eq(path.iter().copied())
        })
        .is_some_and(|command| command.available_on.iter().any(|item| item == target))
}

fn sync_capability_from_manifest(
    capabilities: &mut BTreeSet<String>,
    capability_id: &str,
    available_on_target: bool,
    advertise_when_available: bool,
) {
    if available_on_target {
        if advertise_when_available {
            capabilities.insert(capability_id.to_string());
        }
        return;
    }

    capabilities.remove(capability_id);
}

fn manifest_support_str(value: ManifestSupportState) -> &'static str {
    match value {
        ManifestSupportState::Supported => "supported",
        ManifestSupportState::Unsupported => "unsupported",
    }
}

fn backend_support_str(value: BackendSupportState) -> &'static str {
    match value {
        BackendSupportState::Supported => "supported",
        BackendSupportState::Partial => "partial",
        BackendSupportState::Unsupported => "unsupported",
    }
}

fn uaa_support_str(value: UaaSupportState) -> &'static str {
    match value {
        UaaSupportState::Supported => "supported",
        UaaSupportState::Partial => "partial",
        UaaSupportState::Unsupported => "unsupported",
    }
}

fn pointer_promotion_str(value: PointerPromotionState) -> &'static str {
    match value {
        PointerPromotionState::None => "none",
        PointerPromotionState::LatestSupported => "latest_supported",
        PointerPromotionState::LatestValidated => "latest_validated",
        PointerPromotionState::LatestSupportedAndValidated => "latest_supported_and_validated",
    }
}

fn known_mcp_capability_ids() -> [&'static str; 4] {
    [
        CAPABILITY_MCP_LIST_V1,
        CAPABILITY_MCP_GET_V1,
        CAPABILITY_MCP_ADD_V1,
        CAPABILITY_MCP_REMOVE_V1,
    ]
}

fn mcp_command_path(capability_id: &str) -> &'static [&'static str] {
    match capability_id {
        CAPABILITY_MCP_LIST_V1 => &["mcp", "list"],
        CAPABILITY_MCP_GET_V1 => &["mcp", "get"],
        CAPABILITY_MCP_ADD_V1 => &["mcp", "add"],
        CAPABILITY_MCP_REMOVE_V1 => &["mcp", "remove"],
        _ => &[],
    }
}

fn extract_markdown_code_list(block: &str, heading: &str) -> Result<Vec<String>, String> {
    let heading_index = block
        .find(heading)
        .ok_or_else(|| format!("missing `{heading}` in release guide generated block"))?;
    let after_heading = &block[heading_index + heading.len()..];
    let next_heading = after_heading.find("\n## ").unwrap_or(after_heading.len());
    let section = &after_heading[..next_heading];

    let mut values = Vec::new();
    for line in section.lines() {
        let trimmed = line.trim();
        if !(trimmed.starts_with("- `")
            || trimmed.starts_with("1. `")
            || trimmed.starts_with("2. `")
            || trimmed.starts_with("3. `")
            || trimmed.starts_with("4. `")
            || trimmed.starts_with("5. `")
            || trimmed.starts_with("6. `")
            || trimmed.starts_with("7. `")
            || trimmed.starts_with("8. `")
            || trimmed.starts_with("9. `"))
        {
            continue;
        }

        if let Some(code) = first_inline_code(trimmed) {
            values.push(code);
        }
    }

    if values.is_empty() {
        Err(format!(
            "release guide generated block does not list any packages under `{heading}`"
        ))
    } else {
        Ok(values)
    }
}

fn first_inline_code(line: &str) -> Option<String> {
    let start = line.find('`')?;
    let rest = &line[start + 1..];
    let end = rest.find('`')?;
    Some(rest[..end].to_string())
}

fn parse_markdown_cells(line: &str) -> Vec<&str> {
    line.trim_matches('|')
        .split('|')
        .map(|cell| cell.trim().trim_matches('`'))
        .collect()
}
