//! The registry remains the declaration inventory; retained records cannot disappear by omission.
use crate::{
    agent_registry::{
        support_depth::{
            DepthEnrollmentDeclaration, LifecyclePath, ObligationTemplate, SharedMapping,
        },
        AgentRegistry, REGISTRY_RELATIVE_PATH,
    },
    depth_record::{DepthRecord, Event},
    git_inputs::GitInputs,
    runtime_advertising::source::Source,
};
use std::{fs, path::Path};

/// One compiled isolated maintenance subject. A profile names exactly one; nothing else selects it.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct Subject {
    pub agent_id: &'static str,
    pub version: &'static str,
    pub target: &'static str,
    pub selector: &'static str,
    pub command_path: &'static str,
    pub flag: &'static str,
    pub shared_mapping: SharedMapping,
    /// The mapped `agent_api` capability; `None` for a wrapper-only operation, which has none.
    pub capability: Option<&'static str>,
}
/// Codex qualification subject (MP-P2): unchanged from the IR-P2 binding.
pub(super) const CODEX: Subject = Subject {
    agent_id: "codex",
    version: "0.156.1",
    target: "x86_64-unknown-linux-musl",
    selector: "run-json",
    command_path: "codex exec",
    flag: "--json",
    shared_mapping: SharedMapping::Mapped,
    capability: Some("agent_api.run"),
};
/// Synthetic machinery-proof subject (MP-P1, SA-P3). It qualifies no real agent.
pub(super) const SYNTHETIC: Subject = Subject {
    agent_id: "depth_fixture",
    version: "0.2.0",
    target: "linux-x64",
    selector: "run-json",
    command_path: "depth_fixture run",
    flag: "--json",
    shared_mapping: SharedMapping::WrapperOnly,
    capability: None,
};
pub(super) fn load(
    root: &Path,
    inputs: &mut GitInputs,
    protected: &Source,
    protected_registry: &str,
    subject: &Subject,
) -> Result<AgentRegistry, String> {
    let bytes = inputs.required(REGISTRY_RELATIVE_PATH)?;
    // Isolated governing data may differ from the ordinary canonical registry. It must already
    // be protected in the fixed isolated repository, and still satisfy this finite scope.
    if super::git_blob(root, &bytes)? != protected_registry
        || inputs.executable(REGISTRY_RELATIVE_PATH)?
        || protected.executable(REGISTRY_RELATIVE_PATH)?
        || protected.optional_bytes(REGISTRY_RELATIVE_PATH)?.as_ref() != Some(&bytes)
    {
        return Err("isolated governing registry differs from protected staging".into());
    }
    let registry =
        AgentRegistry::parse_unenforced(std::str::from_utf8(&bytes).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    let agent = registry
        .find(subject.agent_id)
        .ok_or("missing isolated subject registry entry")?;
    let table = agent
        .support_depth
        .as_ref()
        .ok_or("missing isolated depth declaration")?;
    if table.enabled_paths.as_deref() != Some(&[LifecyclePath::Maintenance]) {
        return Err("isolated scope supports maintenance only".into());
    }
    let declarations = table
        .enrollments
        .as_deref()
        .ok_or("missing isolated enrollment")?;
    if declarations.len() != 1 {
        return Err("extra isolated depth enrollment".into());
    }
    selection(&declarations[0], subject)?;
    for entry in &registry.agents {
        if entry.agent_id != subject.agent_id && entry.support_depth.is_some() {
            return Err("extra registered isolated depth scope".into());
        }
    }
    let mut records = Vec::new();
    let mut count = 0;
    // Bind inventories including every registered root and orphan/secondary roots. Unlike record_files,
    // errors, symlinks and non-regular entries refuse instead of being read as absence.
    for prefix in ["cli_manifests", "reports"] {
        walk(root, inputs, prefix, &mut records, &mut count)?;
    }
    for path in records {
        let prefix = format!("{}/reports/", agent.manifest_root);
        let version = path
            .strip_prefix(&prefix)
            .and_then(|p| p.strip_suffix("/depth-record.json"));
        if version != Some(subject.version) {
            return Err(format!("unsupported retained isolated scope: {path}"));
        }
        let raw = inputs.required(&path)?;
        let record = DepthRecord::parse(
            std::str::from_utf8(&raw).map_err(|e| e.to_string())?,
            subject.agent_id,
            subject.version,
        )?;
        if !matches!(record.event, Event::Maintenance { .. })
            || !record.policy.path_enabled
            || record.policy.selection.enrollment != declarations[0]
            || record
                .obligations
                .as_ref()
                .is_some_and(|o| o.targets != [subject.target])
        {
            return Err("retained isolated selection differs from governing declaration".into());
        }
    }
    Ok(registry)
}
fn selection(declaration: &DepthEnrollmentDeclaration, subject: &Subject) -> Result<(), String> {
    let modes = &declaration.modes;
    let operations = &declaration.operations;
    let mappings = &declaration.capabilities;
    if declaration.lifecycle_path != LifecyclePath::Maintenance
        || declaration.version != subject.version
        || declaration.targets != [subject.target]
        || modes.len() != 1
        || operations.len() != 1
        || mappings.len() != usize::from(subject.capability.is_some())
    {
        return Err("unsupported isolated maintenance selection".into());
    }
    let mode = &modes[0];
    if mode.selector != subject.selector
        || mode.command_path != subject.command_path
        || mode.mode != "headless"
        || mode
            .targets
            .as_ref()
            .is_some_and(|targets| targets != &[subject.target])
        || mode.predicates.len() != 1
        || mode.predicates[0].flag != subject.flag
        || mode.predicates[0].value.is_some()
    {
        return Err("unsupported isolated headless presence predicate scope".into());
    }
    let operation = &operations[0];
    if operation.id != "run"
        || operation.serves != [subject.selector]
        || operation.templates != [ObligationTemplate::Core]
        || operation.classification_basis.is_some()
        || operation.classifications.is_some()
        || operation.shared_mapping != Some(subject.shared_mapping)
        || operation.forwarding != Some(false)
        || !operation.not_applicable_permitted.is_empty()
        || mappings
            .iter()
            .any(|m| Some(m.id.as_str()) != subject.capability || m.operations != ["run"])
    {
        return Err("unsupported isolated operation/capability/forwarding/NA scope".into());
    }
    Ok(())
}
fn walk(
    root: &Path,
    inputs: &mut GitInputs,
    path: &str,
    records: &mut Vec<String>,
    count: &mut usize,
) -> Result<(), String> {
    *count += 1;
    if *count > 50_000 || path.split('/').count() > 64 {
        return Err("isolated retained inventory exceeds bound".into());
    }
    for name in inputs.list(path)? {
        let child = format!("{path}/{name}");
        let metadata = fs::symlink_metadata(root.join(&child)).map_err(|e| e.to_string())?;
        if metadata.file_type().is_symlink() {
            return Err("symlink in isolated retained inventory".into());
        }
        if name == "depth-record.json" {
            records.push(child);
        } else if metadata.is_dir() {
            walk(root, inputs, &child, records, count)?;
        } else if !metadata.is_file() {
            return Err("unsupported isolated retained entry".into());
        }
    }
    Ok(())
}
