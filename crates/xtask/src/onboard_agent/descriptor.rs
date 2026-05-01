use std::collections::BTreeMap;

use crate::{approval_artifact, capability_projection::requires_explicit_publication_target};

use super::{
    validate_gate_scalar, ApprovalProvenance, Args, ConfigGate, DraftDescriptorInput, Error,
    TargetGate,
};

impl DraftDescriptorInput {
    pub(super) fn from_raw_args(args: Args) -> Result<Self, Error> {
        Ok(Self {
            agent_id: required_arg(args.agent_id, "--agent-id")?,
            display_name: required_arg(args.display_name, "--display-name")?,
            crate_path: required_arg(args.crate_path, "--crate-path")?,
            backend_module: required_arg(args.backend_module, "--backend-module")?,
            manifest_root: required_arg(args.manifest_root, "--manifest-root")?,
            package_name: required_arg(args.package_name, "--package-name")?,
            canonical_targets: args.canonical_targets,
            wrapper_coverage_binding_kind: required_arg(
                args.wrapper_coverage_binding_kind,
                "--wrapper-coverage-binding-kind",
            )?,
            wrapper_coverage_source_path: required_arg(
                args.wrapper_coverage_source_path,
                "--wrapper-coverage-source-path",
            )?,
            always_on_capabilities: args.always_on_capabilities,
            target_gated_capabilities: args.target_gated_capabilities,
            config_gated_capabilities: args.config_gated_capabilities,
            backend_extensions: args.backend_extensions,
            support_matrix_enabled: required_bool(
                args.support_matrix_enabled,
                "--support-matrix-enabled",
            )?,
            capability_matrix_enabled: required_bool(
                args.capability_matrix_enabled,
                "--capability-matrix-enabled",
            )?,
            capability_matrix_target: args.capability_matrix_target,
            docs_release_track: required_arg(args.docs_release_track, "--docs-release-track")?,
            onboarding_pack_prefix: required_arg(
                args.onboarding_pack_prefix,
                "--onboarding-pack-prefix",
            )?,
            approval_provenance: None,
        })
    }
}

impl From<approval_artifact::ApprovalArtifact> for DraftDescriptorInput {
    fn from(artifact: approval_artifact::ApprovalArtifact) -> Self {
        let descriptor = artifact.descriptor;
        Self {
            agent_id: descriptor.agent_id,
            display_name: descriptor.display_name,
            crate_path: descriptor.crate_path,
            backend_module: descriptor.backend_module,
            manifest_root: descriptor.manifest_root,
            package_name: descriptor.package_name,
            canonical_targets: descriptor.canonical_targets,
            wrapper_coverage_binding_kind: descriptor.wrapper_coverage_binding_kind,
            wrapper_coverage_source_path: descriptor.wrapper_coverage_source_path,
            always_on_capabilities: descriptor.always_on_capabilities,
            target_gated_capabilities: descriptor.target_gated_capabilities,
            config_gated_capabilities: descriptor.config_gated_capabilities,
            backend_extensions: descriptor.backend_extensions,
            support_matrix_enabled: descriptor.support_matrix_enabled,
            capability_matrix_enabled: descriptor.capability_matrix_enabled,
            capability_matrix_target: descriptor.capability_matrix_target,
            docs_release_track: descriptor.docs_release_track,
            onboarding_pack_prefix: descriptor.onboarding_pack_prefix,
            approval_provenance: Some(ApprovalProvenance {
                artifact_path: artifact.relative_path,
                artifact_sha256: artifact.sha256,
                approval_recorded_at: String::new(),
            }),
        }
    }
}

pub(super) fn normalize_capability_matrix_target(
    value: Option<String>,
    canonical_targets: &[String],
    canonical_index: &BTreeMap<String, usize>,
    capability_matrix_enabled: bool,
    target_gated_capabilities: &[TargetGate],
    config_gated_capabilities: &[ConfigGate],
    allow_legacy_single_target_fallback: bool,
) -> Result<Option<String>, Error> {
    let value = value
        .map(|target| validate_gate_scalar(&target, "--capability-matrix-target", &target))
        .transpose()?;

    if let Some(target) = value.as_deref() {
        if !canonical_index.contains_key(target) {
            return Err(Error::Validation(format!(
                "--capability-matrix-target `{target}` is not present in --canonical-target"
            )));
        }
    }

    let explicit_required = requires_explicit_publication_target(
        capability_matrix_enabled,
        !target_gated_capabilities.is_empty(),
        config_gated_capabilities.iter().any(|gate| {
            gate.targets
                .as_ref()
                .is_some_and(|targets| !targets.is_empty())
        }),
    );

    if explicit_required && value.is_none() {
        if allow_legacy_single_target_fallback && canonical_targets.len() == 1 {
            return Ok(Some(canonical_targets[0].clone()));
        }

        return Err(Error::Validation(
            "--capability-matrix-target is required when capability-matrix publication uses target-scoped declarations".to_string(),
        ));
    }

    Ok(value)
}

pub(super) fn canonical_index(canonical_targets: &[String]) -> BTreeMap<String, usize> {
    canonical_targets
        .iter()
        .enumerate()
        .map(|(index, target)| (target.clone(), index))
        .collect()
}

fn required_arg(value: Option<String>, flag_name: &str) -> Result<String, Error> {
    let value = value.ok_or_else(|| {
        Error::Validation(format!(
            "{flag_name} must be provided when --approval is not used"
        ))
    })?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(Error::Validation(format!(
            "{flag_name} must not be empty when --approval is not used"
        )));
    }
    Ok(trimmed.to_string())
}

fn required_bool(value: Option<bool>, flag_name: &str) -> Result<bool, Error> {
    value.ok_or_else(|| {
        Error::Validation(format!(
            "{flag_name} must be provided when --approval is not used"
        ))
    })
}
