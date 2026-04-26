use std::{collections::BTreeSet, fs, path::Path};

use toml_edit::{DocumentMut, Item};

use crate::agent_registry::{AgentRegistry, AgentRegistryEntry, AgentRegistryError};

use super::{
    read_toml, workspace_members, ConfigGate, DraftEntry, Error, TargetGate, WorkspacePathJail,
    REGISTRY_RELATIVE_PATH,
};

pub(super) fn map_registry_load_error(err: AgentRegistryError) -> Error {
    match err {
        AgentRegistryError::Read { path, source } => {
            Error::Internal(format!("read {path}: {source}"))
        }
        AgentRegistryError::Toml(err) => {
            Error::Validation(format!("parse agent registry TOML: {err}"))
        }
        AgentRegistryError::Validation(message) => Error::Validation(message),
    }
}

pub(super) fn validate_registry_conflicts(
    registry: &AgentRegistry,
    draft: &DraftEntry,
) -> Result<(), Error> {
    for entry in &registry.agents {
        if entry.agent_id == draft.agent_id {
            if registry_entry_matches_draft(entry, draft) {
                continue;
            }
            return Err(Error::Validation(format!(
                "agent_id `{}` already exists in {REGISTRY_RELATIVE_PATH}",
                draft.agent_id
            )));
        }
        if entry.crate_path == draft.crate_path {
            return Err(Error::Validation(format!(
                "crate_path `{}` is already owned by agent `{}`",
                draft.crate_path, entry.agent_id
            )));
        }
        if entry.backend_module == draft.backend_module {
            return Err(Error::Validation(format!(
                "backend_module `{}` is already owned by agent `{}`",
                draft.backend_module, entry.agent_id
            )));
        }
        if entry.manifest_root == draft.manifest_root {
            return Err(Error::Validation(format!(
                "manifest_root `{}` is already owned by agent `{}`",
                draft.manifest_root, entry.agent_id
            )));
        }
        if entry.package_name == draft.package_name {
            return Err(Error::Validation(format!(
                "package_name `{}` is already owned by agent `{}`",
                draft.package_name, entry.agent_id
            )));
        }
        if entry.scaffold.onboarding_pack_prefix == draft.onboarding_pack_prefix {
            return Err(Error::Validation(format!(
                "onboarding_pack_prefix `{}` is already owned by agent `{}`",
                draft.onboarding_pack_prefix, entry.agent_id
            )));
        }
    }

    Ok(())
}

pub(super) fn validate_workspace_package_name_conflicts(
    draft: &DraftEntry,
    jail: &WorkspacePathJail,
) -> Result<(), Error> {
    let root_manifest_path = jail.resolve(Path::new("Cargo.toml"))?;
    let root_manifest = read_toml(&root_manifest_path)?;
    let draft_crate_path = Path::new(&draft.crate_path);
    for member in workspace_members(&root_manifest)? {
        let manifest_path = jail.resolve(&member.join("Cargo.toml"))?;
        if member.as_path() == draft_crate_path && !manifest_path.exists() {
            continue;
        }
        let manifest = read_toml(&manifest_path)?;
        let Some(package_name) = package_name(&manifest) else {
            continue;
        };
        if member.as_path() == draft_crate_path && package_name == draft.package_name {
            continue;
        }
        if package_name == draft.package_name {
            return Err(Error::Validation(format!(
                "package_name `{}` already exists in workspace member `{}` ({})",
                draft.package_name,
                member.display(),
                member.join("Cargo.toml").display()
            )));
        }
    }
    Ok(())
}

pub(super) fn validate_filesystem_conflicts(
    draft: &DraftEntry,
    jail: &WorkspacePathJail,
) -> Result<(), Error> {
    validate_runtime_owned_path_shape(jail, &draft.crate_path, "crate_path")?;
    validate_runtime_owned_path_shape(jail, &draft.backend_module, "backend_module")?;

    let docs_pack_root = draft.docs_pack_root();
    let _docs_pack_path = jail.resolve(&docs_pack_root)?;

    let manifest_root_path = jail.resolve(Path::new(&draft.manifest_root))?;
    if !manifest_root_path.exists() {
        return Ok(());
    }

    let has_current_json = manifest_root_path.join("current.json").exists();
    validate_manifest_root_targets(draft, jail, &manifest_root_path)?;
    if !has_current_json {
        validate_manifest_root_artifact_conflicts(draft, &manifest_root_path)?;
    }
    Ok(())
}

pub(super) fn validate_candidate_registry(candidate_registry_text: &str) -> Result<(), Error> {
    AgentRegistry::parse(candidate_registry_text).map_err(map_registry_load_error)?;
    Ok(())
}

pub(super) fn desired_registry_text(
    registry: &AgentRegistry,
    draft: &DraftEntry,
    registry_text: &str,
    registry_entry_preview: &str,
) -> String {
    if registry
        .find(&draft.agent_id)
        .is_some_and(|entry| registry_entry_matches_draft(entry, draft))
    {
        registry_text.to_string()
    } else {
        append_registry_entry(registry_text, registry_entry_preview)
    }
}

fn registry_entry_matches_draft(entry: &AgentRegistryEntry, draft: &DraftEntry) -> bool {
    entry.display_name == draft.display_name
        && entry.crate_path == draft.crate_path
        && entry.backend_module == draft.backend_module
        && entry.manifest_root == draft.manifest_root
        && entry.package_name == draft.package_name
        && entry.canonical_targets == draft.canonical_targets
        && entry.wrapper_coverage.binding_kind == draft.wrapper_coverage_binding_kind
        && entry.wrapper_coverage.source_path == draft.wrapper_coverage_source_path
        && entry.capability_declaration.always_on == draft.always_on_capabilities
        && entry.capability_declaration.backend_extensions == draft.backend_extensions
        && entry
            .capability_declaration
            .target_gated
            .iter()
            .map(|gate| TargetGate {
                capability_id: gate.capability_id.clone(),
                targets: gate.targets.clone(),
            })
            .collect::<Vec<_>>()
            == draft.target_gated_capabilities
        && entry
            .capability_declaration
            .config_gated
            .iter()
            .map(|gate| ConfigGate {
                capability_id: gate.capability_id.clone(),
                config_key: gate.config_key.clone(),
                targets: gate.targets.clone(),
            })
            .collect::<Vec<_>>()
            == draft.config_gated_capabilities
        && entry.publication.support_matrix_enabled == draft.support_matrix_enabled
        && entry.publication.capability_matrix_enabled == draft.capability_matrix_enabled
        && entry.publication.capability_matrix_target == draft.capability_matrix_target
        && entry.release.docs_release_track == draft.docs_release_track
        && entry.scaffold.onboarding_pack_prefix == draft.onboarding_pack_prefix
}

fn package_name(doc: &DocumentMut) -> Option<&str> {
    doc.get("package")
        .and_then(Item::as_table_like)
        .and_then(|package| package.get("name"))
        .and_then(Item::as_str)
}

fn validate_runtime_owned_path_shape(
    jail: &WorkspacePathJail,
    relative_path: &str,
    field_name: &str,
) -> Result<(), Error> {
    let resolved = jail.resolve(Path::new(relative_path))?;
    if resolved.exists() && !resolved.is_dir() {
        return Err(Error::Validation(format!(
            "{field_name} `{relative_path}` already exists on disk but is not a directory"
        )));
    }
    Ok(())
}

fn validate_manifest_root_targets(
    draft: &DraftEntry,
    jail: &WorkspacePathJail,
    manifest_root_path: &Path,
) -> Result<(), Error> {
    let current_json_path = manifest_root_path.join("current.json");
    if !current_json_path.exists() {
        return Ok(());
    }

    let text = fs::read_to_string(&current_json_path)
        .map_err(|err| Error::Internal(format!("read {}: {err}", current_json_path.display())))?;
    let value: serde_json::Value = serde_json::from_str(&text).map_err(|err| {
        Error::Validation(format!(
            "parse {}: {err}",
            current_json_path
                .strip_prefix(jail.root())
                .unwrap_or(&current_json_path)
                .display()
        ))
    })?;
    let Some(existing_targets) = value
        .get("expected_targets")
        .and_then(|value| value.as_array())
    else {
        return Ok(());
    };

    let existing_targets = existing_targets
        .iter()
        .map(|value| {
            value.as_str().ok_or_else(|| {
                Error::Validation(format!(
                    "{} contains non-string expected_targets entries",
                    current_json_path
                        .strip_prefix(jail.root())
                        .unwrap_or(&current_json_path)
                        .display()
                ))
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    if existing_targets
        != draft
            .canonical_targets
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
    {
        return Err(Error::Validation(format!(
            "manifest_root `{}` already contains expected_targets {:?}, which conflicts with proposed canonical_targets {:?}",
            draft.manifest_root,
            existing_targets,
            draft.canonical_targets
        )));
    }

    Ok(())
}

fn validate_manifest_root_artifact_conflicts(
    draft: &DraftEntry,
    manifest_root_path: &Path,
) -> Result<(), Error> {
    let mut conflicting_targets = BTreeSet::new();
    for target in &draft.canonical_targets {
        let supported_pointer = manifest_root_path
            .join("pointers/latest_supported")
            .join(format!("{target}.txt"));
        let validated_pointer = manifest_root_path
            .join("pointers/latest_validated")
            .join(format!("{target}.txt"));
        let coverage_suffix = format!("coverage.{target}.json");
        if supported_pointer.exists() {
            conflicting_targets.insert(format!(
                "{}/pointers/latest_supported/{}.txt",
                draft.manifest_root, target
            ));
        }
        if validated_pointer.exists() {
            conflicting_targets.insert(format!(
                "{}/pointers/latest_validated/{}.txt",
                draft.manifest_root, target
            ));
        }
        if manifest_root_contains_report(&manifest_root_path.join("reports"), &coverage_suffix)? {
            conflicting_targets.insert(format!(
                "{}/reports/**/{}",
                draft.manifest_root, coverage_suffix
            ));
        }
    }

    if conflicting_targets.is_empty() {
        return Ok(());
    }

    Err(Error::Validation(format!(
        "pre-existing target artifacts conflict with proposed canonical_targets: {}",
        conflicting_targets
            .into_iter()
            .collect::<Vec<_>>()
            .join(", ")
    )))
}

fn manifest_root_contains_report(root: &Path, suffix: &str) -> Result<bool, Error> {
    if !root.exists() {
        return Ok(false);
    }
    for entry in fs::read_dir(root)
        .map_err(|err| Error::Internal(format!("read {}: {err}", root.display())))?
    {
        let entry =
            entry.map_err(|err| Error::Internal(format!("read {}: {err}", root.display())))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|err| Error::Internal(format!("read {}: {err}", path.display())))?;
        if file_type.is_dir() {
            if manifest_root_contains_report(&path, suffix)? {
                return Ok(true);
            }
            continue;
        }
        if file_type.is_file()
            && path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name == suffix)
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn append_registry_entry(registry_text: &str, registry_entry_preview: &str) -> String {
    let mut combined = registry_text.trim_end().to_string();
    combined.push_str("\n\n");
    combined.push_str(registry_entry_preview);
    combined
}
