mod preview;

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use crate::agent_registry::{AgentRegistry, AgentRegistryError, REGISTRY_RELATIVE_PATH};
use clap::Parser;
use thiserror::Error;

use self::preview::{
    build_docs_preview, build_manifest_preview, build_manual_follow_up, build_release_preview,
    render_registry_entry_preview, write_docs_preview, write_input_summary, write_manifest_preview,
    write_manual_follow_up, write_registry_preview, write_release_preview,
};

const OWNERSHIP_MARKER: &str = "<!-- generated-by: xtask onboard-agent; owner: control-plane -->";
const DOCS_NEXT_ROOT: &str = "docs/project_management/next";
const RELEASE_DOC_PATH: &str = "docs/crates-io-release.md";
const PUBLISH_WORKFLOW_PATH: &str = ".github/workflows/publish-crates.yml";
const PUBLISH_SCRIPT_PATH: &str = "scripts/publish_crates.py";
const VALIDATE_PUBLISH_SCRIPT_PATH: &str = "scripts/validate_publish_versions.py";
const CHECK_PUBLISH_READINESS_SCRIPT_PATH: &str = "scripts/check_publish_readiness.py";

#[derive(Debug, Parser, Clone)]
pub struct Args {
    /// Preview-only mode. M1 requires this flag and performs no filesystem writes.
    #[arg(long, required = true)]
    pub dry_run: bool,

    /// Control-plane agent identifier.
    #[arg(long)]
    pub agent_id: String,

    /// Human-facing agent name.
    #[arg(long)]
    pub display_name: String,

    /// Repo-relative wrapper crate root.
    #[arg(long)]
    pub crate_path: String,

    /// Repo-relative backend module path under `crates/agent_api/src/backends/`.
    #[arg(long)]
    pub backend_module: String,

    /// Repo-relative manifest root under `cli_manifests/`.
    #[arg(long)]
    pub manifest_root: String,

    /// Publishable crate package name.
    #[arg(long)]
    pub package_name: String,

    /// Canonical target triple used for capability projection and scaffolding.
    #[arg(long = "canonical-target")]
    pub canonical_targets: Vec<String>,

    /// Wrapper coverage binding kind.
    #[arg(long = "wrapper-coverage-binding-kind")]
    pub wrapper_coverage_binding_kind: String,

    /// Repo-relative wrapper coverage source path.
    #[arg(long = "wrapper-coverage-source-path")]
    pub wrapper_coverage_source_path: String,

    /// Always-advertised capability id.
    #[arg(long = "always-on-capability")]
    pub always_on_capabilities: Vec<String>,

    /// Capability gated by one or more canonical targets: `<capability-id>:<target>[,<target>...]`.
    #[arg(long = "target-gated-capability")]
    pub target_gated_capabilities: Vec<String>,

    /// Capability gated by config key and optional targets: `<capability-id>:<config-key>[:<target>[,<target>...]]`.
    #[arg(long = "config-gated-capability")]
    pub config_gated_capabilities: Vec<String>,

    /// Backend-owned extension id.
    #[arg(long = "backend-extension")]
    pub backend_extensions: Vec<String>,

    /// Whether the agent is enrolled in support-matrix publication.
    #[arg(long = "support-matrix-enabled", action = clap::ArgAction::Set)]
    pub support_matrix_enabled: bool,

    /// Whether the agent is enrolled in capability-matrix publication.
    #[arg(long = "capability-matrix-enabled", action = clap::ArgAction::Set)]
    pub capability_matrix_enabled: bool,

    /// Release documentation track name.
    #[arg(long = "docs-release-track")]
    pub docs_release_track: String,

    /// Docs onboarding pack directory prefix under `docs/project_management/next/`.
    #[arg(long = "onboarding-pack-prefix")]
    pub onboarding_pack_prefix: String,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    Internal(String),
}

impl Error {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Validation(_) => 2,
            Self::Internal(_) => 1,
        }
    }
}

#[derive(Debug, Clone)]
struct DraftEntry {
    agent_id: String,
    display_name: String,
    crate_path: String,
    backend_module: String,
    manifest_root: String,
    package_name: String,
    canonical_targets: Vec<String>,
    wrapper_coverage_binding_kind: String,
    wrapper_coverage_source_path: String,
    always_on_capabilities: Vec<String>,
    target_gated_capabilities: Vec<TargetGate>,
    config_gated_capabilities: Vec<ConfigGate>,
    backend_extensions: Vec<String>,
    support_matrix_enabled: bool,
    capability_matrix_enabled: bool,
    docs_release_track: String,
    onboarding_pack_prefix: String,
}

#[derive(Debug, Clone)]
struct TargetGate {
    capability_id: String,
    targets: Vec<String>,
}

#[derive(Debug, Clone)]
struct ConfigGate {
    capability_id: String,
    config_key: String,
    targets: Option<Vec<String>>,
}

pub fn run(args: Args) -> Result<(), Error> {
    let workspace_root = resolve_workspace_root()?;
    let mut stdout = io::stdout();
    run_in_workspace(&workspace_root, args, &mut stdout)
}

pub fn run_in_workspace<W: Write>(
    workspace_root: &Path,
    args: Args,
    writer: &mut W,
) -> Result<(), Error> {
    if !args.dry_run {
        return Err(Error::Validation(
            "--dry-run is required in M1; mutation mode is not available".to_string(),
        ));
    }

    let registry = AgentRegistry::load(workspace_root).map_err(map_registry_load_error)?;
    let registry_text = fs::read_to_string(workspace_root.join(REGISTRY_RELATIVE_PATH))
        .map_err(|err| Error::Internal(format!("read {REGISTRY_RELATIVE_PATH}: {err}")))?;
    let draft = DraftEntry::from_args(args)?;

    validate_registry_conflicts(&registry, &draft)?;
    validate_filesystem_conflicts(workspace_root, &draft)?;

    let registry_entry_preview = render_registry_entry_preview(&draft);
    validate_candidate_registry(&registry_text, &registry_entry_preview)?;

    let release_preview = build_release_preview(workspace_root, &registry, &draft)?;
    let docs_preview = build_docs_preview(&draft, &release_preview);
    let manifest_preview = build_manifest_preview(&draft);
    let manual_follow_up = build_manual_follow_up(&draft);

    writeln!(writer, "== ONBOARD-AGENT DRY RUN ==")
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(
        writer,
        "M1 preview-only mode; no filesystem writes performed."
    )
    .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer).map_err(|err| Error::Internal(format!("write stdout: {err}")))?;

    write_input_summary(writer, &draft)?;
    write_registry_preview(writer, &registry_entry_preview)?;
    write_docs_preview(writer, &docs_preview)?;
    write_manifest_preview(writer, &manifest_preview)?;
    write_release_preview(writer, &release_preview)?;
    write_manual_follow_up(writer, &manual_follow_up)?;

    writeln!(writer, "== RESULT ==")
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "OK: onboard-agent dry-run preview complete.")
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "No files were written.")
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;

    Ok(())
}

impl DraftEntry {
    fn from_args(args: Args) -> Result<Self, Error> {
        let canonical_targets =
            normalize_ordered_unique(args.canonical_targets, "--canonical-target", true)?;
        let canonical_index = canonical_index(&canonical_targets);
        let always_on_capabilities =
            normalize_sorted_unique(args.always_on_capabilities, "--always-on-capability")?;
        let backend_extensions =
            normalize_sorted_unique(args.backend_extensions, "--backend-extension")?;
        let mut target_gated_capabilities = parse_target_gates(
            args.target_gated_capabilities,
            "--target-gated-capability",
            &canonical_index,
        )?;
        target_gated_capabilities.sort_by(|left, right| {
            left.capability_id
                .cmp(&right.capability_id)
                .then_with(|| left.targets.cmp(&right.targets))
        });
        let mut config_gated_capabilities = parse_config_gates(
            args.config_gated_capabilities,
            "--config-gated-capability",
            &canonical_index,
        )?;
        config_gated_capabilities.sort_by(|left, right| {
            left.capability_id
                .cmp(&right.capability_id)
                .then_with(|| left.config_key.cmp(&right.config_key))
                .then_with(|| left.targets.cmp(&right.targets))
        });

        Ok(Self {
            agent_id: args.agent_id,
            display_name: args.display_name,
            crate_path: args.crate_path,
            backend_module: args.backend_module,
            manifest_root: args.manifest_root,
            package_name: args.package_name,
            canonical_targets,
            wrapper_coverage_binding_kind: args.wrapper_coverage_binding_kind,
            wrapper_coverage_source_path: args.wrapper_coverage_source_path,
            always_on_capabilities,
            target_gated_capabilities,
            config_gated_capabilities,
            backend_extensions,
            support_matrix_enabled: args.support_matrix_enabled,
            capability_matrix_enabled: args.capability_matrix_enabled,
            docs_release_track: args.docs_release_track,
            onboarding_pack_prefix: args.onboarding_pack_prefix,
        })
    }

    fn docs_pack_root(&self) -> PathBuf {
        Path::new(DOCS_NEXT_ROOT).join(&self.onboarding_pack_prefix)
    }
}

fn resolve_workspace_root() -> Result<PathBuf, Error> {
    let current_dir = std::env::current_dir()
        .map_err(|err| Error::Internal(format!("resolve current directory: {err}")))?;
    for candidate in current_dir.ancestors() {
        let cargo_toml = candidate.join("Cargo.toml");
        let Ok(text) = fs::read_to_string(&cargo_toml) else {
            continue;
        };
        if text.contains("[workspace]") {
            return Ok(candidate.to_path_buf());
        }
    }

    Err(Error::Internal(format!(
        "could not resolve workspace root from {}",
        current_dir.display()
    )))
}

fn map_registry_load_error(err: AgentRegistryError) -> Error {
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

fn validate_registry_conflicts(registry: &AgentRegistry, draft: &DraftEntry) -> Result<(), Error> {
    if registry.find(&draft.agent_id).is_some() {
        return Err(Error::Validation(format!(
            "agent_id `{}` already exists in {REGISTRY_RELATIVE_PATH}",
            draft.agent_id
        )));
    }

    for entry in &registry.agents {
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
        if entry.scaffold.onboarding_pack_prefix == draft.onboarding_pack_prefix {
            return Err(Error::Validation(format!(
                "onboarding_pack_prefix `{}` is already owned by agent `{}`",
                draft.onboarding_pack_prefix, entry.agent_id
            )));
        }
    }

    Ok(())
}

fn validate_filesystem_conflicts(workspace_root: &Path, draft: &DraftEntry) -> Result<(), Error> {
    validate_runtime_owned_path_absent(workspace_root, &draft.crate_path, "crate_path")?;
    validate_runtime_owned_path_absent(workspace_root, &draft.backend_module, "backend_module")?;

    let docs_pack_root = draft.docs_pack_root();
    let docs_pack_path = workspace_root.join(&docs_pack_root);
    if docs_pack_path.exists() {
        return Err(Error::Validation(format!(
            "docs scaffold path `{}` already exists; ownership is ambiguous",
            docs_pack_root.display()
        )));
    }

    let manifest_root_path = workspace_root.join(&draft.manifest_root);
    if !manifest_root_path.exists() {
        return Ok(());
    }

    let current_json_path = manifest_root_path.join("current.json");
    if current_json_path.exists() {
        let text = fs::read_to_string(&current_json_path).map_err(|err| {
            Error::Internal(format!("read {}: {err}", current_json_path.display()))
        })?;
        let value: serde_json::Value = serde_json::from_str(&text).map_err(|err| {
            Error::Validation(format!(
                "parse {}: {err}",
                current_json_path
                    .strip_prefix(workspace_root)
                    .unwrap_or(&current_json_path)
                    .display()
            ))
        })?;
        if let Some(existing_targets) = value
            .get("expected_targets")
            .and_then(|value| value.as_array())
        {
            let existing_targets = existing_targets
                .iter()
                .map(|value| {
                    value.as_str().ok_or_else(|| {
                        Error::Validation(format!(
                            "{} contains non-string expected_targets entries",
                            current_json_path
                                .strip_prefix(workspace_root)
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
        }
    }

    let mut conflicting_targets = Vec::new();
    for target in &draft.canonical_targets {
        let supported_pointer = manifest_root_path
            .join("pointers/latest_supported")
            .join(format!("{target}.txt"));
        let validated_pointer = manifest_root_path
            .join("pointers/latest_validated")
            .join(format!("{target}.txt"));
        let coverage_suffix = format!("coverage.{target}.json");
        if supported_pointer.exists() {
            conflicting_targets.push(format!(
                "{}/pointers/latest_supported/{}.txt",
                draft.manifest_root, target
            ));
        }
        if validated_pointer.exists() {
            conflicting_targets.push(format!(
                "{}/pointers/latest_validated/{}.txt",
                draft.manifest_root, target
            ));
        }
        if manifest_root_contains_report(&manifest_root_path.join("reports"), &coverage_suffix)? {
            conflicting_targets.push(format!(
                "{}/reports/**/{}",
                draft.manifest_root, coverage_suffix
            ));
        }
    }

    if !conflicting_targets.is_empty() {
        conflicting_targets.sort();
        conflicting_targets.dedup();
        return Err(Error::Validation(format!(
            "pre-existing target artifacts conflict with proposed canonical_targets: {}",
            conflicting_targets.join(", ")
        )));
    }

    Err(Error::Validation(format!(
        "manifest_root `{}` already exists; ownership is ambiguous",
        draft.manifest_root
    )))
}

fn validate_runtime_owned_path_absent(
    workspace_root: &Path,
    relative_path: &str,
    field_name: &str,
) -> Result<(), Error> {
    if workspace_root.join(relative_path).exists() {
        return Err(Error::Validation(format!(
            "{field_name} `{relative_path}` already exists on disk; ownership is ambiguous"
        )));
    }
    Ok(())
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
        } else if file_type.is_file()
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

fn validate_candidate_registry(
    registry_text: &str,
    registry_entry_preview: &str,
) -> Result<(), Error> {
    let mut combined = registry_text.trim_end().to_string();
    combined.push_str("\n\n");
    combined.push_str(registry_entry_preview);
    AgentRegistry::parse(&combined).map_err(map_registry_load_error)?;
    Ok(())
}

fn normalize_ordered_unique(
    values: Vec<String>,
    flag_name: &str,
    require_non_empty: bool,
) -> Result<Vec<String>, Error> {
    if require_non_empty && values.is_empty() {
        return Err(Error::Validation(format!(
            "{flag_name} must be provided at least once"
        )));
    }
    let mut seen = BTreeSet::new();
    let mut out = Vec::with_capacity(values.len());
    for value in values {
        let trimmed = value.trim().to_string();
        if trimmed.is_empty() {
            return Err(Error::Validation(format!(
                "{flag_name} must not contain empty values"
            )));
        }
        if !seen.insert(trimmed.clone()) {
            return Err(Error::Validation(format!(
                "{flag_name} contains duplicate value `{trimmed}`"
            )));
        }
        out.push(trimmed);
    }
    Ok(out)
}

fn normalize_sorted_unique(values: Vec<String>, flag_name: &str) -> Result<Vec<String>, Error> {
    let mut out = normalize_ordered_unique(values, flag_name, false)?;
    out.sort();
    Ok(out)
}

fn parse_target_gates(
    values: Vec<String>,
    flag_name: &str,
    canonical_index: &BTreeMap<String, usize>,
) -> Result<Vec<TargetGate>, Error> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::with_capacity(values.len());
    for value in values {
        let (capability_id, raw_targets) = value.split_once(':').ok_or_else(|| {
            Error::Validation(format!(
                "{flag_name} must be formatted as <capability-id>:<target>[,<target>...] (got `{value}`)"
            ))
        })?;
        let capability_id = validate_gate_scalar(capability_id, flag_name, &value)?;
        let mut targets = parse_gate_targets(raw_targets, flag_name, &value, canonical_index)?;
        if !seen.insert((capability_id.clone(), targets.clone())) {
            return Err(Error::Validation(format!(
                "{flag_name} contains duplicate entry `{value}`"
            )));
        }
        targets.sort_by_key(|target| canonical_index[target]);
        out.push(TargetGate {
            capability_id,
            targets,
        });
    }
    Ok(out)
}

fn parse_config_gates(
    values: Vec<String>,
    flag_name: &str,
    canonical_index: &BTreeMap<String, usize>,
) -> Result<Vec<ConfigGate>, Error> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::with_capacity(values.len());
    for value in values {
        let mut parts = value.splitn(3, ':');
        let capability_id =
            validate_gate_scalar(parts.next().unwrap_or_default(), flag_name, &value)?;
        let config_key = validate_gate_scalar(parts.next().unwrap_or_default(), flag_name, &value)?;
        let targets = match parts.next() {
            Some(raw_targets) => {
                let mut targets =
                    parse_gate_targets(raw_targets, flag_name, &value, canonical_index)?;
                targets.sort_by_key(|target| canonical_index[target]);
                Some(targets)
            }
            None => None,
        };
        let signature = format!(
            "{}:{}:{}",
            capability_id,
            config_key,
            targets
                .as_ref()
                .map(|targets| targets.join(","))
                .unwrap_or_default()
        );
        if !seen.insert(signature) {
            return Err(Error::Validation(format!(
                "{flag_name} contains duplicate entry `{value}`"
            )));
        }
        out.push(ConfigGate {
            capability_id,
            config_key,
            targets,
        });
    }
    Ok(out)
}

fn validate_gate_scalar(value: &str, flag_name: &str, original: &str) -> Result<String, Error> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(Error::Validation(format!(
            "{flag_name} contains an empty field in `{original}`"
        )));
    }
    Ok(trimmed.to_string())
}

fn parse_gate_targets(
    raw_targets: &str,
    flag_name: &str,
    original: &str,
    canonical_index: &BTreeMap<String, usize>,
) -> Result<Vec<String>, Error> {
    let targets = raw_targets
        .split(',')
        .map(str::trim)
        .filter(|target| !target.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    if targets.is_empty() {
        return Err(Error::Validation(format!(
            "{flag_name} must declare at least one target in `{original}`"
        )));
    }

    let mut seen = BTreeSet::new();
    for target in &targets {
        if !seen.insert(target.clone()) {
            return Err(Error::Validation(format!(
                "{flag_name} contains duplicate target `{target}` in `{original}`"
            )));
        }
        if !canonical_index.contains_key(target) {
            return Err(Error::Validation(format!(
                "{flag_name} target `{target}` is not present in --canonical-target"
            )));
        }
    }
    Ok(targets)
}

fn canonical_index(canonical_targets: &[String]) -> BTreeMap<String, usize> {
    canonical_targets
        .iter()
        .enumerate()
        .map(|(index, target)| (target.clone(), index))
        .collect()
}
