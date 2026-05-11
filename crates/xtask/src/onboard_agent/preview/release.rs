use std::{fs, path::Path};

use toml_edit::DocumentMut;

use super::render::release_touchpoint_lines;
use super::{
    DraftEntry, Error, ReleasePreview, TextMutationPlan, RELEASE_DOC_END_MARKER, RELEASE_DOC_PATH,
    RELEASE_DOC_START_MARKER, ROOT_AGENT_API_PACKAGE_NAME, WRAPPER_EVENTS_PACKAGE_NAME,
};

pub(super) fn build_release_preview(
    workspace_root: &Path,
    draft: &DraftEntry,
) -> Result<ReleasePreview, Error> {
    let cargo_toml_path = workspace_root.join("Cargo.toml");
    let cargo_toml_text = fs::read_to_string(&cargo_toml_path)
        .map_err(|err| Error::Internal(format!("read {}: {err}", cargo_toml_path.display())))?;
    let workspace_manifest =
        build_workspace_manifest_mutation(&cargo_toml_text, &draft.crate_path)?;

    let release_doc_path = workspace_root.join(RELEASE_DOC_PATH);
    let release_doc_text = fs::read_to_string(&release_doc_path)
        .map_err(|err| Error::Internal(format!("read {}: {err}", release_doc_path.display())))?;
    let publishable_packages =
        publishable_release_packages(workspace_root, draft, &workspace_manifest.desired_after)?;
    let release_doc_block = render_release_doc_block(&publishable_packages);
    let desired_release_doc = splice_release_doc_block(&release_doc_text, &release_doc_block);

    Ok(ReleasePreview {
        lines: release_touchpoint_lines(draft),
        workspace_manifest,
        release_doc: TextMutationPlan {
            path: RELEASE_DOC_PATH.to_string(),
            expected_before: release_doc_text,
            desired_after: desired_release_doc,
        },
    })
}

fn build_workspace_manifest_mutation(
    cargo_toml_text: &str,
    crate_path: &str,
) -> Result<TextMutationPlan, Error> {
    let mut doc = cargo_toml_text
        .parse::<DocumentMut>()
        .map_err(|err| Error::Internal(format!("parse Cargo.toml: {err}")))?;
    let members = doc["workspace"]["members"]
        .as_array_mut()
        .ok_or_else(|| Error::Internal("workspace.members must be an array".to_string()))?;
    let already_present = members
        .iter()
        .any(|member| member.as_str() == Some(crate_path));
    if !already_present {
        let insert_index = members
            .iter()
            .position(|member| member.as_str() == Some("crates/wrapper_events"))
            .or_else(|| {
                members
                    .iter()
                    .position(|member| member.as_str() == Some("crates/xtask"))
            })
            .unwrap_or(members.len());
        members.insert(insert_index, toml_edit::Value::from(crate_path));
    }
    let mut desired_after = doc.to_string();
    if !desired_after.ends_with('\n') {
        desired_after.push('\n');
    }

    Ok(TextMutationPlan {
        path: "Cargo.toml".to_string(),
        expected_before: cargo_toml_text.to_string(),
        desired_after,
    })
}

fn publishable_release_packages(
    workspace_root: &Path,
    draft: &DraftEntry,
    desired_workspace_manifest: &str,
) -> Result<Vec<String>, Error> {
    let doc = desired_workspace_manifest
        .parse::<DocumentMut>()
        .map_err(|err| Error::Internal(format!("parse desired Cargo.toml: {err}")))?;
    let members = doc["workspace"]["members"]
        .as_array()
        .ok_or_else(|| Error::Internal("workspace.members must be an array".to_string()))?;

    let mut leaf_packages = Vec::new();
    let mut wrapper_events = None;
    let mut root_agent_api = None;

    for member in members {
        let Some(member_path) = member.as_str() else {
            return Err(Error::Internal(
                "workspace.members entries must be strings".to_string(),
            ));
        };

        let package_name = if member_path == draft.crate_path {
            draft.package_name.clone()
        } else {
            let manifest_path = workspace_root.join(member_path).join("Cargo.toml");
            let manifest_text = match fs::read_to_string(&manifest_path) {
                Ok(text) => text,
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => continue,
                Err(err) => {
                    return Err(Error::Internal(format!(
                        "read {}: {err}",
                        manifest_path.display()
                    )));
                }
            };
            let manifest = manifest_text.parse::<DocumentMut>().map_err(|err| {
                Error::Internal(format!("parse {}: {err}", manifest_path.display()))
            })?;
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
            package_name.to_string()
        };

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

fn package_publish_disabled(doc: &DocumentMut) -> bool {
    doc.get("package")
        .and_then(toml_edit::Item::as_table_like)
        .and_then(|package| package.get("publish"))
        .and_then(toml_edit::Item::as_bool)
        == Some(false)
}

fn render_release_doc_block(packages: &[String]) -> String {
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

fn splice_release_doc_block(existing: &str, block: &str) -> String {
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
