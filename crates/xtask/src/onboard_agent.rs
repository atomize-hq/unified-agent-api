mod mutation;
mod preview;
mod validation;

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use crate::agent_registry::{AgentRegistry, REGISTRY_RELATIVE_PATH};
use clap::{ArgGroup, Parser};
use thiserror::Error;
use toml_edit::DocumentMut;

use self::mutation::{apply_mutations, ApplySummary, PlannedMutation, WorkspacePathJail};
use self::preview::{
    build_docs_preview, build_manifest_preview, build_manual_follow_up, build_release_preview,
    load_proving_run_metrics, render_registry_entry_preview, write_docs_preview,
    write_input_summary, write_manifest_preview, write_manual_follow_up, write_registry_preview,
    write_release_preview,
};
use self::validation::{
    desired_registry_text, map_registry_load_error, validate_candidate_registry,
    validate_filesystem_conflicts, validate_registry_conflicts,
    validate_workspace_package_name_conflicts,
};

const OWNERSHIP_MARKER: &str = "<!-- generated-by: xtask onboard-agent; owner: control-plane -->";
const DOCS_NEXT_ROOT: &str = "docs/project_management/next";
const RELEASE_DOC_PATH: &str = "docs/crates-io-release.md";
const PUBLISH_WORKFLOW_PATH: &str = ".github/workflows/publish-crates.yml";
const PUBLISH_SCRIPT_PATH: &str = "scripts/publish_crates.py";
const VALIDATE_PUBLISH_SCRIPT_PATH: &str = "scripts/validate_publish_versions.py";
const CHECK_PUBLISH_READINESS_SCRIPT_PATH: &str = "scripts/check_publish_readiness.py";

#[derive(Debug, Parser, Clone)]
#[command(group(
    ArgGroup::new("mode")
        .required(true)
        .args(["dry_run", "write"])
        .multiple(false)
))]
pub struct Args {
    /// Preview the onboarding plan without mutating the workspace.
    #[arg(long)]
    pub dry_run: bool,

    /// Apply the onboarding plan to the workspace.
    #[arg(long)]
    pub write: bool,

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

#[derive(Debug, Clone, PartialEq, Eq)]
struct TargetGate {
    capability_id: String,
    targets: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ConfigGate {
    capability_id: String,
    config_key: String,
    targets: Option<Vec<String>>,
}

#[derive(Debug)]
struct OnboardingPlan {
    registry_entry_preview: String,
    docs_preview: Vec<(String, Option<String>)>,
    manifest_preview: Vec<(String, Option<String>)>,
    release_preview: preview::ReleasePreview,
    manual_follow_up: Vec<String>,
    mutations: Vec<PlannedMutation>,
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
    let jail = WorkspacePathJail::new(workspace_root)?;
    let dry_run_mode = args.dry_run;
    let registry_path = jail.resolve(Path::new(REGISTRY_RELATIVE_PATH))?;
    let registry_text = fs::read_to_string(&registry_path)
        .map_err(|err| Error::Internal(format!("read {REGISTRY_RELATIVE_PATH}: {err}")))?;
    let registry = AgentRegistry::parse(&registry_text).map_err(map_registry_load_error)?;
    let draft = DraftEntry::from_args(args)?;

    validate_registry_conflicts(&registry, &draft)?;
    validate_workspace_package_name_conflicts(&draft, &jail)?;
    validate_filesystem_conflicts(&draft, &jail)?;

    let plan = OnboardingPlan::build(workspace_root, &registry, &draft, &registry_text)?;

    writeln!(
        writer,
        "== ONBOARD-AGENT {} ==",
        if dry_run_mode { "DRY RUN" } else { "WRITE" }
    )
    .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    if dry_run_mode {
        writeln!(
            writer,
            "Shared onboarding plan preview; no filesystem writes performed."
        )
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    } else {
        writeln!(writer, "Shared onboarding plan preview before apply.")
            .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    }
    writeln!(writer).map_err(|err| Error::Internal(format!("write stdout: {err}")))?;

    write_input_summary(writer, &draft)?;
    write_registry_preview(writer, &plan.registry_entry_preview)?;
    write_docs_preview(writer, &plan.docs_preview)?;
    write_manifest_preview(writer, &plan.manifest_preview)?;
    write_release_preview(writer, &plan.release_preview)?;
    write_manual_follow_up(writer, &plan.manual_follow_up)?;

    let apply_summary = if dry_run_mode {
        None
    } else {
        Some(apply_mutations(workspace_root, &plan.mutations)?)
    };

    writeln!(writer, "== RESULT ==")
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    match apply_summary {
        Some(summary) => write_apply_result(writer, summary)?,
        None => {
            writeln!(writer, "OK: onboard-agent dry-run preview complete.")
                .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
            writeln!(writer, "No files were written.")
                .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
        }
    }

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

impl OnboardingPlan {
    fn build(
        workspace_root: &Path,
        registry: &AgentRegistry,
        draft: &DraftEntry,
        registry_text: &str,
    ) -> Result<Self, Error> {
        let registry_entry_preview = render_registry_entry_preview(draft);
        let registry_after =
            desired_registry_text(registry, draft, registry_text, &registry_entry_preview);
        validate_candidate_registry(&registry_after)?;

        let release_preview = build_release_preview(workspace_root, draft)?;
        let proving_run_metrics = load_proving_run_metrics(workspace_root, draft)?;
        let docs_preview =
            build_docs_preview(draft, &release_preview, proving_run_metrics.as_ref());
        let manifest_preview = build_manifest_preview(draft);
        let manual_follow_up = build_manual_follow_up(draft, proving_run_metrics.as_ref());

        let mut mutations = Vec::with_capacity(3 + docs_preview.len() + manifest_preview.len());
        mutations.push(PlannedMutation::replace(
            REGISTRY_RELATIVE_PATH,
            registry_text.as_bytes().to_vec(),
            registry_after.into_bytes(),
        ));
        mutations.push(PlannedMutation::replace(
            &release_preview.workspace_manifest.path,
            release_preview
                .workspace_manifest
                .expected_before
                .clone()
                .into_bytes(),
            release_preview
                .workspace_manifest
                .desired_after
                .clone()
                .into_bytes(),
        ));
        mutations.push(PlannedMutation::replace(
            &release_preview.release_doc.path,
            release_preview
                .release_doc
                .expected_before
                .clone()
                .into_bytes(),
            release_preview
                .release_doc
                .desired_after
                .clone()
                .into_bytes(),
        ));
        // Closeout metadata is packet-local manual state. The onboarding command can read it to
        // render closeout-aware docs, but it must not rewrite or heal that file implicitly.
        mutations.extend(preview_writes(&docs_preview));
        mutations.extend(preview_writes(&manifest_preview));

        Ok(Self {
            registry_entry_preview,
            docs_preview,
            manifest_preview,
            release_preview,
            manual_follow_up,
            mutations,
        })
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

fn preview_writes(previews: &[(String, Option<String>)]) -> Vec<PlannedMutation> {
    previews
        .iter()
        .map(|(path, contents)| {
            PlannedMutation::create(
                path.clone(),
                contents.clone().unwrap_or_default().into_bytes(),
            )
        })
        .collect()
}

fn write_apply_result<W: Write>(writer: &mut W, summary: ApplySummary) -> Result<(), Error> {
    writeln!(writer, "OK: onboard-agent write complete.")
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(
        writer,
        "Mutation summary: {} written, {} identical, {} total planned.",
        summary.written, summary.identical, summary.total
    )
    .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    Ok(())
}

fn read_toml(path: &Path) -> Result<DocumentMut, Error> {
    let text = fs::read_to_string(path)
        .map_err(|err| Error::Internal(format!("read {}: {err}", path.display())))?;
    text.parse::<DocumentMut>()
        .map_err(|err| Error::Internal(format!("parse {}: {err}", path.display())))
}

fn workspace_members(root_doc: &DocumentMut) -> Result<Vec<PathBuf>, Error> {
    let members = root_doc["workspace"]["members"]
        .as_array()
        .ok_or_else(|| Error::Internal("workspace.members must be an array".to_string()))?;
    members
        .iter()
        .map(|member| {
            let member = member.as_str().ok_or_else(|| {
                Error::Internal("workspace.members entries must be strings".to_string())
            })?;
            Ok(PathBuf::from(member))
        })
        .collect()
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
