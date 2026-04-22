use std::{fs, path::Path};

use toml_edit::DocumentMut;

pub const RELEASE_DOC_PATH: &str = "docs/crates-io-release.md";
pub const RELEASE_DOC_START_MARKER: &str =
    "<!-- generated-by: xtask onboard-agent; section: crates-io-release -->";
pub const RELEASE_DOC_END_MARKER: &str =
    "<!-- /generated-by: xtask onboard-agent; section: crates-io-release -->";

const WORKSPACE_MANIFEST_PATH: &str = "Cargo.toml";
const WRAPPER_EVENTS_PACKAGE_NAME: &str = "unified-agent-api-wrapper-events";
const ROOT_AGENT_API_PACKAGE_NAME: &str = "unified-agent-api";

pub fn render_release_doc(workspace_root: &Path) -> Result<String, String> {
    let existing = fs::read_to_string(workspace_root.join(RELEASE_DOC_PATH)).map_err(|err| {
        format!(
            "read {}: {err}",
            workspace_root.join(RELEASE_DOC_PATH).display()
        )
    })?;
    let packages = publishable_release_packages(workspace_root)?;
    let block = render_release_doc_block(&packages);
    Ok(splice_release_doc_block(&existing, &block))
}

pub fn publishable_release_packages(workspace_root: &Path) -> Result<Vec<String>, String> {
    let workspace_manifest_text = fs::read_to_string(workspace_root.join(WORKSPACE_MANIFEST_PATH))
        .map_err(|err| {
            format!(
                "read {}: {err}",
                workspace_root.join(WORKSPACE_MANIFEST_PATH).display()
            )
        })?;
    let doc = workspace_manifest_text
        .parse::<DocumentMut>()
        .map_err(|err| {
            format!(
                "parse {}: {err}",
                workspace_root.join(WORKSPACE_MANIFEST_PATH).display()
            )
        })?;
    let members = doc["workspace"]["members"]
        .as_array()
        .ok_or_else(|| "workspace.members must be an array".to_string())?;

    let mut leaf_packages = Vec::new();
    let mut wrapper_events = None;
    let mut root_agent_api = None;

    for member in members {
        let Some(member_path) = member.as_str() else {
            return Err("workspace.members entries must be strings".to_string());
        };
        let manifest_path = workspace_root.join(member_path).join("Cargo.toml");
        let manifest_text = match fs::read_to_string(&manifest_path) {
            Ok(text) => text,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => continue,
            Err(err) => return Err(format!("read {}: {err}", manifest_path.display())),
        };
        let manifest = manifest_text
            .parse::<DocumentMut>()
            .map_err(|err| format!("parse {}: {err}", manifest_path.display()))?;
        if package_publish_disabled(&manifest) {
            continue;
        }
        let Some(package_name) = manifest
            .get("package")
            .and_then(toml_edit::Item::as_table_like)
            .and_then(|package| package.get("name"))
            .and_then(toml_edit::Item::as_str)
        else {
            continue;
        };
        if !package_name.starts_with("unified-agent-api") {
            continue;
        }

        let package_name = package_name.to_string();
        if package_name == WRAPPER_EVENTS_PACKAGE_NAME {
            wrapper_events = Some(package_name);
        } else if package_name == ROOT_AGENT_API_PACKAGE_NAME {
            root_agent_api = Some(package_name);
        } else {
            leaf_packages.push(package_name);
        }
    }

    if let Some(package_name) = wrapper_events {
        leaf_packages.push(package_name);
    }
    if let Some(package_name) = root_agent_api {
        leaf_packages.push(package_name);
    }

    Ok(leaf_packages)
}

pub fn render_release_doc_block(packages: &[String]) -> String {
    let list = packages
        .iter()
        .map(|package| format!("- `{package}`"))
        .collect::<Vec<_>>()
        .join("\n");
    let order = packages
        .iter()
        .enumerate()
        .map(|(index, package)| format!("{}. `{package}`", index + 1))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "{RELEASE_DOC_START_MARKER}\n## Published crates\n\nThis repository publishes {} Rust packages for each root `VERSION` bump:\n\n{list}\n\n## Publish order\n\nAlways publish in this order:\n\n{order}\n{RELEASE_DOC_END_MARKER}",
        packages.len()
    )
}

pub fn splice_release_doc_block(existing: &str, block: &str) -> String {
    match (
        existing.find(RELEASE_DOC_START_MARKER),
        existing.find(RELEASE_DOC_END_MARKER),
    ) {
        (Some(start), Some(end)) if start <= end => {
            let before = &existing[..start];
            let after = &existing[end + RELEASE_DOC_END_MARKER.len()..];
            format!("{before}{block}{after}")
        }
        _ => {
            let trimmed = existing.trim_end();
            format!("{trimmed}\n\n{block}\n")
        }
    }
}

fn package_publish_disabled(doc: &DocumentMut) -> bool {
    doc.get("package")
        .and_then(toml_edit::Item::as_table_like)
        .and_then(|package| package.get("publish"))
        .and_then(toml_edit::Item::as_bool)
        == Some(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splice_release_doc_replaces_existing_generated_block() {
        let existing = "# Guide\n\nbefore\n\n<!-- generated-by: xtask onboard-agent; section: crates-io-release -->\nold\n<!-- /generated-by: xtask onboard-agent; section: crates-io-release -->\n\nafter\n";
        let block = "<!-- generated-by: xtask onboard-agent; section: crates-io-release -->\nnew\n<!-- /generated-by: xtask onboard-agent; section: crates-io-release -->";

        let updated = splice_release_doc_block(existing, block);

        assert!(updated.contains("before"));
        assert!(updated.contains("after"));
        assert!(updated.contains("\nnew\n"));
        assert!(!updated.contains("\nold\n"));
    }
}
