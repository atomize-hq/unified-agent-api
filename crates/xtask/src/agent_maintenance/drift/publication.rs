use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use crate::{
    agent_registry::{AgentRegistry, AgentRegistryEntry, REGISTRY_RELATIVE_PATH},
    release_doc,
    support_matrix::{validate_publication_consistency, SupportMatrixArtifact, SupportRow},
};

use super::{
    build_finding, shared, DriftCategory, DriftFinding, CAPABILITY_MATRIX_PATH, RELEASE_DOC_PATH,
    SUPPORT_MATRIX_JSON_PATH, SUPPORT_MATRIX_MARKDOWN_PATH,
};

pub(super) fn inspect_registry_manifest(
    entry: &AgentRegistryEntry,
    workspace_root: &Path,
) -> Option<DriftFinding> {
    let current_path = workspace_root
        .join(&entry.manifest_root)
        .join("current.json");
    let mut issues = Vec::new();

    let current = match shared::read_json::<shared::ManifestCurrent>(&current_path) {
        Ok(current) => current,
        Err(err) => {
            issues.push(err);
            return Some(build_finding(
                DriftCategory::RegistryManifest,
                "registry entry no longer matches the committed manifest root.",
                issues,
                shared::build_surfaces(
                    workspace_root,
                    [PathBuf::from(REGISTRY_RELATIVE_PATH), current_path],
                ),
            ));
        }
    };

    if current.expected_targets.is_empty() {
        issues.push(format!(
            "{} no longer declares any expected_targets",
            entry.manifest_root
        ));
    }

    let missing_targets = entry
        .canonical_targets
        .iter()
        .filter(|target| {
            !current
                .expected_targets
                .iter()
                .any(|candidate| candidate == *target)
        })
        .cloned()
        .collect::<Vec<_>>();
    if !missing_targets.is_empty() {
        issues.push(format!(
            "registry canonical targets are absent from {}: {}",
            current_path
                .strip_prefix(workspace_root)
                .unwrap_or(&current_path)
                .display(),
            missing_targets.join(", ")
        ));
    }

    if issues.is_empty() {
        None
    } else {
        Some(build_finding(
            DriftCategory::RegistryManifest,
            "registry entry no longer matches the committed manifest root.",
            issues,
            shared::build_surfaces(
                workspace_root,
                [PathBuf::from(REGISTRY_RELATIVE_PATH), current_path],
            ),
        ))
    }
}

pub(super) fn inspect_capability_publication(
    entry: &AgentRegistryEntry,
    workspace_root: &Path,
    capability_truth: Result<&BTreeSet<String>, &String>,
) -> Option<DriftFinding> {
    if !entry.publication.capability_matrix_enabled {
        return None;
    }

    let mut issues = Vec::new();
    let truth = match capability_truth {
        Ok(truth) => truth,
        Err(err) => {
            issues.push(err.clone());
            return Some(build_finding(
                DriftCategory::CapabilityPublication,
                "published capability inventory no longer matches modeled backend truth.",
                issues,
                shared::build_surfaces(
                    workspace_root,
                    [
                        PathBuf::from(CAPABILITY_MATRIX_PATH),
                        PathBuf::from(&entry.backend_module),
                    ],
                ),
            ));
        }
    };

    let capability_matrix_path = workspace_root.join(CAPABILITY_MATRIX_PATH);
    let published = match shared::parse_capability_matrix_agent_support(
        &capability_matrix_path,
        &entry.agent_id,
    ) {
        Ok(published) => published,
        Err(err) => {
            issues.push(err);
            return Some(build_finding(
                DriftCategory::CapabilityPublication,
                "published capability inventory no longer matches modeled backend truth.",
                issues,
                shared::build_surfaces(
                    workspace_root,
                    [
                        PathBuf::from(CAPABILITY_MATRIX_PATH),
                        PathBuf::from(&entry.backend_module),
                    ],
                ),
            ));
        }
    };

    let missing = truth.difference(&published).cloned().collect::<Vec<_>>();
    if !missing.is_empty() {
        issues.push(format!(
            "published capability matrix is missing {} capability id(s): {}",
            missing.len(),
            missing.join(", ")
        ));
    }

    let unexpected = published.difference(truth).cloned().collect::<Vec<_>>();
    if !unexpected.is_empty() {
        issues.push(format!(
            "published capability matrix overclaims {} capability id(s): {}",
            unexpected.len(),
            unexpected.join(", ")
        ));
    }

    if issues.is_empty() {
        None
    } else {
        Some(build_finding(
            DriftCategory::CapabilityPublication,
            "published capability inventory no longer matches modeled backend truth.",
            issues,
            shared::build_surfaces(
                workspace_root,
                [
                    PathBuf::from(CAPABILITY_MATRIX_PATH),
                    PathBuf::from(&entry.backend_module),
                ],
            ),
        ))
    }
}

pub(super) fn inspect_support_publication(
    entry: &AgentRegistryEntry,
    workspace_root: &Path,
    expected_rows: Result<&Vec<SupportRow>, &String>,
) -> Option<DriftFinding> {
    if !entry.publication.support_matrix_enabled {
        return None;
    }

    let mut issues = Vec::new();
    let expected_rows = match expected_rows {
        Ok(rows) => rows.as_slice(),
        Err(err) => {
            issues.push(err.clone());
            return Some(build_finding(
                DriftCategory::SupportPublication,
                "published support artifacts no longer match committed support truth.",
                issues,
                shared::build_surfaces(
                    workspace_root,
                    [
                        PathBuf::from(&entry.manifest_root),
                        PathBuf::from(SUPPORT_MATRIX_JSON_PATH),
                        PathBuf::from(SUPPORT_MATRIX_MARKDOWN_PATH),
                    ],
                ),
            ));
        }
    };

    let json_path = workspace_root.join(SUPPORT_MATRIX_JSON_PATH);
    let json_artifact = match shared::read_json::<SupportMatrixArtifact>(&json_path) {
        Ok(artifact) => artifact,
        Err(err) => {
            issues.push(err);
            return Some(build_finding(
                DriftCategory::SupportPublication,
                "published support artifacts no longer match committed support truth.",
                issues,
                shared::build_surfaces(
                    workspace_root,
                    [
                        PathBuf::from(SUPPORT_MATRIX_JSON_PATH),
                        PathBuf::from(SUPPORT_MATRIX_MARKDOWN_PATH),
                    ],
                ),
            ));
        }
    };

    if let Err(consistency_issues) =
        validate_publication_consistency(workspace_root, &json_artifact.rows)
    {
        let agent_issues = consistency_issues
            .into_iter()
            .filter(|issue| issue.agent == entry.agent_id)
            .map(|issue| issue.message)
            .collect::<Vec<_>>();
        issues.extend(agent_issues);
    }

    let published_rows = json_artifact
        .rows
        .iter()
        .filter(|row| row.agent == entry.agent_id)
        .cloned()
        .collect::<Vec<_>>();
    if published_rows != expected_rows {
        issues.push(
            "published support-matrix JSON rows do not match derived support rows".to_string(),
        );
    }

    let markdown_path = workspace_root.join(SUPPORT_MATRIX_MARKDOWN_PATH);
    let markdown = match fs::read_to_string(&markdown_path) {
        Ok(markdown) => markdown,
        Err(err) => {
            issues.push(format!("read({}): {err}", markdown_path.display()));
            return Some(build_finding(
                DriftCategory::SupportPublication,
                "published support artifacts no longer match committed support truth.",
                issues,
                shared::build_surfaces(
                    workspace_root,
                    [
                        PathBuf::from(SUPPORT_MATRIX_JSON_PATH),
                        PathBuf::from(SUPPORT_MATRIX_MARKDOWN_PATH),
                    ],
                ),
            ));
        }
    };

    let expected_section = shared::render_support_markdown_section(expected_rows);
    match shared::extract_support_markdown_section(&markdown, &entry.agent_id) {
        Ok(section) if section.trim_end() == expected_section.trim_end() => {}
        Ok(_) => issues.push(
            "published support-matrix Markdown section does not match derived support rows"
                .to_string(),
        ),
        Err(err) => issues.push(err),
    }

    if issues.is_empty() {
        None
    } else {
        Some(build_finding(
            DriftCategory::SupportPublication,
            "published support artifacts no longer match committed support truth.",
            issues,
            shared::build_surfaces(
                workspace_root,
                [
                    PathBuf::from(SUPPORT_MATRIX_JSON_PATH),
                    PathBuf::from(SUPPORT_MATRIX_MARKDOWN_PATH),
                ],
            ),
        ))
    }
}

pub(super) fn inspect_release_doc(
    entry: &AgentRegistryEntry,
    workspace_root: &Path,
    registry: &AgentRegistry,
) -> Option<DriftFinding> {
    if entry.release.docs_release_track != "crates-io" {
        return None;
    }

    let release_path = workspace_root.join(RELEASE_DOC_PATH);
    let text = match fs::read_to_string(&release_path) {
        Ok(text) => text,
        Err(err) => {
            return Some(build_finding(
                DriftCategory::ReleaseDoc,
                "release guide block no longer matches the registry-backed publish order.",
                vec![format!("read({}): {err}", release_path.display())],
                shared::build_surfaces(
                    workspace_root,
                    [
                        PathBuf::from(RELEASE_DOC_PATH),
                        PathBuf::from(REGISTRY_RELATIVE_PATH),
                    ],
                ),
            ));
        }
    };

    let actual = match shared::parse_release_doc_packages(&text) {
        Ok(actual) => actual,
        Err(err) => {
            return Some(build_finding(
                DriftCategory::ReleaseDoc,
                "release guide block no longer matches the registry-backed publish order.",
                vec![err],
                shared::build_surfaces(
                    workspace_root,
                    [
                        PathBuf::from(RELEASE_DOC_PATH),
                        PathBuf::from(REGISTRY_RELATIVE_PATH),
                    ],
                ),
            ));
        }
    };

    let expected =
        release_doc::registry_release_packages(registry, &entry.release.docs_release_track);
    let mut issues = Vec::new();
    compare_release_doc_section(
        "published crates block",
        &actual.published_crates,
        &expected,
        &mut issues,
    );
    compare_release_doc_section(
        "publish order block",
        &actual.publish_order,
        &expected,
        &mut issues,
    );

    if issues.is_empty() {
        None
    } else {
        Some(build_finding(
            DriftCategory::ReleaseDoc,
            "release guide block no longer matches the registry-backed publish order.",
            issues,
            shared::build_surfaces(
                workspace_root,
                [
                    PathBuf::from(RELEASE_DOC_PATH),
                    PathBuf::from(REGISTRY_RELATIVE_PATH),
                ],
            ),
        ))
    }
}

fn compare_release_doc_section(
    section_name: &str,
    actual: &[String],
    expected: &[String],
    issues: &mut Vec<String>,
) {
    if actual == expected {
        return;
    }

    let missing = expected
        .iter()
        .filter(|package| !actual.contains(*package))
        .cloned()
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        issues.push(format!(
            "{section_name} is missing package(s): {}",
            missing.join(", ")
        ));
    }

    let unexpected = actual
        .iter()
        .filter(|package| !expected.contains(*package))
        .cloned()
        .collect::<Vec<_>>();
    if !unexpected.is_empty() {
        issues.push(format!(
            "{section_name} contains unexpected package(s): {}",
            unexpected.join(", ")
        ));
    }

    let duplicates = duplicate_packages(actual);
    if !duplicates.is_empty() {
        issues.push(format!(
            "{section_name} contains duplicate package(s): {}",
            duplicates.join(", ")
        ));
    }

    if missing.is_empty() && unexpected.is_empty() && duplicates.is_empty() {
        issues.push(format!(
            "{section_name} order does not match expected registry-backed publish order"
        ));
    }
}

fn duplicate_packages(packages: &[String]) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut duplicates = BTreeSet::new();
    for package in packages {
        if !seen.insert(package.clone()) {
            duplicates.insert(package.clone());
        }
    }
    duplicates.into_iter().collect()
}
