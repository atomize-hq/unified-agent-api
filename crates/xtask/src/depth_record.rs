//! The depth record, as Annex B of `docs/specs/support-depth-contract.md` defines it.
//!
//! This module decides only what the annex says a record decides by itself: its form, its three
//! identities, the digest an acceptance entry is made over, and how the coverage of two records
//! is compared, and whether it states what a maintenance request froze. Whether the parts agree
//! with each other or with the registry is depth admission, and is not decided here. Nothing
//! writes a record yet; `manifest-validate` and the support matrix generator read one.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::agent_maintenance::support_audit::{DebtInventoryRow, SurfaceIdentity};
use crate::agent_registry::support_depth::{
    DepthEnrollmentDeclaration, LifecyclePath, ObligationId, ObligationTemplate,
};

pub const DEPTH_RECORD_SCHEMA_VERSION: u64 = 1;
/// The record's file in `reports/<version>/` of its manifest root (Annex B.1).
pub const DEPTH_RECORD_FILE_NAME: &str = "depth-record.json";

/// Every `reports/<version>/depth-record.json` under a manifest root, by version. Anything at the
/// name counts, a dangling symlink or a directory included, so a reader fails on it rather than
/// taking it for no record.
pub fn record_files(root: &Path) -> Vec<(String, PathBuf)> {
    let Ok(entries) = fs::read_dir(root.join("reports")) else {
        return Vec::new();
    };
    let mut files: Vec<_> = entries
        .flatten()
        .map(|entry| entry.path().join(DEPTH_RECORD_FILE_NAME))
        .filter(|path| fs::symlink_metadata(path).is_ok())
        .filter_map(|path| {
            let version = path.parent()?.file_name()?.to_string_lossy().into_owned();
            Some((version, path))
        })
        .collect();
    files.sort();
    files
}

/// SHA-256 identities of evidence held as regular stage-0 files with matching working bytes.
/// Missing paths have no identity. Unsafe, untracked, conflicted or divergent inputs refuse;
/// a working file cannot confer an identity the index does not hold. No proof is admitted here.
pub fn bound_evidence<'a>(
    workspace: &Path,
    records: impl IntoIterator<Item = &'a DepthRecord>,
) -> Result<BTreeMap<String, String>, String> {
    let paths: BTreeSet<&str> = records
        .into_iter()
        .flat_map(|record| &record.execution)
        .flat_map(|execution| &execution.results)
        .flat_map(|result| &result.evidence)
        .map(|file| file.path.as_str())
        .collect();
    if paths.is_empty() {
        return Ok(BTreeMap::new());
    }
    let mut inputs = crate::git_inputs::GitInputs::new(workspace)?;
    let mut identities = BTreeMap::new();
    for path in paths {
        if inputs.optional(path)?.is_some() {
            identities.insert(path.into(), inputs.reference(path)?.sha256);
        }
    }
    inputs.verify()?;
    Ok(identities)
}

/// What a maintenance request states of the generation it froze: its Event, and the P and O
/// identities of its `[support_depth]` table, both `None` when it has none.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestBinding {
    pub event: Event,
    pub policy_identity: Option<String>,
    pub obligations_identity: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DepthRecord {
    pub schema_version: u64,
    pub event: Event,
    pub policy: PolicyPart,
    /// `None` is a part stated as not yet produced. The member itself is never absent.
    #[serde(deserialize_with = "stated")]
    pub obligations: Option<ObligationsPart>,
    #[serde(deserialize_with = "stated")]
    pub execution: Option<ExecutionPart>,
    pub acceptance_entries: Vec<AcceptanceEntry>,
}

fn stated<'de, D: Deserializer<'de>, T: Deserialize<'de>>(d: D) -> Result<Option<T>, D::Error> {
    Option::deserialize(d)
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "lifecycle_path", rename_all = "snake_case", deny_unknown_fields)]
pub enum Event {
    Maintenance {
        request_commit: String,
        request_recorded_at: String,
        trigger_kind: String,
        opened_from: String,
    },
    Onboarding {
        approval_commit: String,
        approval_recorded_at: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyPart {
    pub identity: String,
    pub selection: Selection,
    /// `GOVERNING_RULES_REVISION` as it stood when P was frozen.
    pub rules_revision: String,
    /// The governing rules as P resolved them: BASE, and each template the selection selects.
    pub base: Vec<ObligationId>,
    pub templates: Vec<TemplateRule>,
    pub path_enabled: bool,
    pub debt_baseline: Vec<DebtInventoryRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Selection {
    pub agent_id: String,
    pub enrollment: DepthEnrollmentDeclaration,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TemplateRule {
    pub template: ObligationTemplate,
    pub obligations: Vec<ObligationId>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FileRef {
    pub path: String,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObligationsPart {
    pub identity: String,
    pub policy_identity: String,
    pub targets: Vec<String>,
    pub acquired_inputs: Vec<FileRef>,
    pub operations: Vec<OperationObligations>,
    pub classification_required: Vec<OperationSurface>,
    pub acceptance_work: Vec<AcceptanceWork>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OperationObligations {
    pub id: String,
    pub surfaces: Vec<SurfaceIdentity>,
    /// `M` aside: an `M` obligation follows from a capability mapping and has no id here.
    pub obligations: Vec<ObligationId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OperationSurface {
    pub operation: String,
    pub surface: SurfaceIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AcceptanceWork {
    pub surface: SurfaceIdentity,
    pub targets: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionPart {
    pub identity: String,
    pub policy_identity: String,
    pub obligations_identity: String,
    pub results: Vec<ObligationResult>,
    pub reuse_bindings: Vec<ReuseBinding>,
    pub debt_grants: Vec<DebtInventoryRow>,
    pub debt_retired: Vec<String>,
}

/// One obligation of one operation on one target. It states `id`, or `capability` for the `M`
/// obligation of that capability, and never both.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObligationRef {
    pub operation: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<ObligationId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capability: Option<String>,
    pub target: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObligationResult {
    pub obligation: ObligationRef,
    pub result: DepthResult,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
    pub evidence: Vec<FileRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DepthResult {
    Verified,
    Unverified,
    Failed,
    NotApplicable,
}

/// The P and O that a reused obligation's evidence was produced under.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReuseBinding {
    pub obligation: ObligationRef,
    pub policy_identity: String,
    pub obligations_identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "backs", rename_all = "snake_case", deny_unknown_fields)]
pub enum AcceptanceEntry {
    MaintenanceCloseout {
        closeout: FileRef,
        record_sha256: String,
    },
    ProvingRunCloseout {
        closeout: FileRef,
        record_sha256: String,
    },
    Publication {
        packet: FileRef,
        record_sha256: String,
    },
    Promotion {
        targets: Vec<String>,
        record_sha256: String,
    },
}

impl AcceptanceEntry {
    /// The digest of what the record stated when the entry was made.
    pub fn record_sha256(&self) -> &str {
        match self {
            Self::MaintenanceCloseout { record_sha256, .. }
            | Self::ProvingRunCloseout { record_sha256, .. }
            | Self::Publication { record_sha256, .. }
            | Self::Promotion { record_sha256, .. } => record_sha256,
        }
    }
}

impl DepthRecord {
    /// Reads the record that `agent_id`'s manifest root holds for `version`.
    pub fn parse(raw: &str, agent_id: &str, version: &str) -> Result<Self, String> {
        let value: Value =
            serde_json::from_str(raw).map_err(|err| format!("depth record: parse JSON: {err}"))?;
        // Decided before the typed read, so that another revision is reported as one.
        if value.get("schema_version").and_then(Value::as_u64) != Some(DEPTH_RECORD_SCHEMA_VERSION)
        {
            return Err(format!(
                "depth record: schema_version must be {DEPTH_RECORD_SCHEMA_VERSION}; no other \
                 revision is read"
            ));
        }
        // Read from the text, not from `value`, which has already dropped a repeated member.
        let record: Self =
            serde_json::from_str(raw).map_err(|err| format!("depth record: {err}"))?;
        // One content has one written form, so an optional member is left out and never `null`.
        if json(&record)? != value {
            return Err(
                "depth record: an optional member that states nothing must be left out, every \
                 other member must be written, and a debt row's targets are written in order"
                    .to_string(),
            );
        }
        record
            .validate(agent_id, version)
            .and_then(|()| ordered("", &value))
            .map_err(|err| format!("depth record: {err}"))?;
        Ok(record)
    }

    /// Why the record does not state what `request` froze, under rule 4 of the maintenance request
    /// contract's request fields: the request's Event, its P identity, and its O identity, or no O
    /// while it states none. A request without the table froze no P, so no record states it.
    pub fn request_mismatch(&self, request: &RequestBinding) -> Option<&'static str> {
        if self.event != request.event {
            return Some("the record's Event is not the request's");
        }
        if request.policy_identity.as_ref() != Some(&self.policy.identity) {
            return Some("the record's P is not the one the request froze");
        }
        let obligations = self.obligations.as_ref().map(|part| &part.identity);
        (obligations != request.obligations_identity.as_ref())
            .then_some("the record's O is not the one the request froze")
    }

    /// The digest an acceptance entry is made over: the Event and the three parts, identities
    /// included, acceptance entries left out.
    pub fn stated_sha256(&self) -> Result<String, String> {
        let mut stated = serde_json::Map::new();
        stated.insert("event".to_string(), json(&self.event)?);
        stated.insert("policy".to_string(), json(&self.policy)?);
        stated.insert("obligations".to_string(), json(&self.obligations)?);
        stated.insert("execution".to_string(), json(&self.execution)?);
        sha256_canonical(&Value::Object(stated))
    }

    /// What this record lacks of what `earlier` covers and claims: a covered mode, a target the
    /// mode is covered on, or a claimed capability. Empty when the selection has not shrunk.
    pub fn lacking_from(&self, earlier: &Self) -> Vec<String> {
        let now = &self.policy.selection.enrollment;
        let then = &earlier.policy.selection.enrollment;
        let mut lacking = Vec::new();
        for mode in &then.modes {
            let kept = now.modes.iter().find(|kept| {
                kept.selector == mode.selector
                    && kept.command_path == mode.command_path
                    && kept.mode == mode.mode
                    && kept.predicates.len() == mode.predicates.len()
                    && kept.predicates.iter().all(|p| mode.predicates.contains(p))
            });
            let Some(kept) = kept else {
                lacking.push(format!("mode `{}`", mode.selector));
                continue;
            };
            let kept_targets = kept.targets.as_ref().unwrap_or(&now.targets);
            for target in mode.targets.as_ref().unwrap_or(&then.targets) {
                if !kept_targets.contains(target) {
                    lacking.push(format!("mode `{}` on `{target}`", mode.selector));
                }
            }
        }
        for capability in &then.capabilities {
            if !now.capabilities.iter().any(|c| c.id == capability.id) {
                lacking.push(format!("capability `{}`", capability.id));
            }
        }
        lacking
    }

    fn validate(&self, agent_id: &str, version: &str) -> Result<(), String> {
        let enrollment = &self.policy.selection.enrollment;
        if self.policy.selection.agent_id != agent_id || enrollment.version != version {
            return Err(format!(
                "policy.selection must name agent `{agent_id}` and version `{version}`, whose \
                 directory holds the record"
            ));
        }
        enrollment
            .validate(agent_id)
            .map_err(|err| format!("policy.selection.enrollment: {err}"))?;

        let (path, fields) = match &self.event {
            Event::Maintenance {
                request_commit,
                request_recorded_at,
                trigger_kind,
                opened_from,
            } => (
                LifecyclePath::Maintenance,
                vec![
                    request_commit,
                    request_recorded_at,
                    trigger_kind,
                    opened_from,
                ],
            ),
            Event::Onboarding {
                approval_commit,
                approval_recorded_at,
            } => (
                LifecyclePath::Onboarding,
                vec![approval_commit, approval_recorded_at],
            ),
        };
        if path != enrollment.lifecycle_path {
            return Err("event.lifecycle_path must be the selection's lifecycle path".to_string());
        }
        if fields.iter().any(|field| field.trim().is_empty()) {
            return Err("event must state every field".to_string());
        }

        // Policy part.
        identity("policy", &self.policy.identity, &self.policy)?;
        let selected: BTreeSet<_> = enrollment
            .operations
            .iter()
            .flat_map(|operation| &operation.templates)
            .collect();
        let stated: BTreeSet<_> = self.policy.templates.iter().map(|r| &r.template).collect();
        if stated != selected || stated.len() != self.policy.templates.len() {
            return Err(
                "policy.templates must state each template the selection selects, once".to_string(),
            );
        }
        if self.policy.rules_revision.trim().is_empty() {
            return Err("policy.rules_revision must name a revision".to_string());
        }
        unique("policy.base", self.policy.base.iter())?;
        unique(
            "policy.debt_baseline",
            debt_rows("policy.debt_baseline", &self.policy.debt_baseline, agent_id)?,
        )?;

        if let Some(part) = &self.obligations {
            identity("obligations", &part.identity, part)?;
            sha256("obligations.policy_identity", &part.policy_identity)?;
            unique("obligations.targets", part.targets.iter())?;
            unique(
                "obligations.operations",
                part.operations.iter().map(|o| &o.id),
            )?;
            for input in &part.acquired_inputs {
                file_ref("obligations.acquired_inputs", input)?;
            }
            // One file and one name identity each have one entry, or one content has two forms.
            unique(
                "obligations.acquired_inputs",
                part.acquired_inputs.iter().map(|input| &input.path),
            )?;
            unique(
                "obligations.acceptance_work",
                part.acceptance_work.iter().map(|work| &work.surface),
            )?;
        }

        if let Some(part) = &self.execution {
            identity("execution", &part.identity, part)?;
            sha256("execution.policy_identity", &part.policy_identity)?;
            sha256("execution.obligations_identity", &part.obligations_identity)?;
            unique(
                "execution.results",
                part.results.iter().map(|r| &r.obligation),
            )?;
            unique(
                "execution.reuse_bindings",
                part.reuse_bindings.iter().map(|b| &b.obligation),
            )?;
            for result in &part.results {
                obligation_ref("execution.results", &result.obligation)?;
                let at = format!(
                    "execution.results[{} on {}]",
                    result.obligation.operation, result.obligation.target
                );
                let rationale = result.rationale.as_deref().map(str::trim);
                if (result.result == DepthResult::NotApplicable) != rationale.is_some()
                    || rationale == Some("")
                {
                    return Err(format!(
                        "{at}: a `not_applicable` result states a rationale, and no other does"
                    ));
                }
                if result.result != DepthResult::Verified && !result.evidence.is_empty() {
                    return Err(format!("{at}: only a `verified` result binds evidence"));
                }
                for evidence in &result.evidence {
                    file_ref(&at, evidence)?;
                }
                unique(&at, result.evidence.iter().map(|evidence| &evidence.path))?;
            }
            for binding in &part.reuse_bindings {
                obligation_ref("execution.reuse_bindings", &binding.obligation)?;
                sha256(
                    "execution.reuse_bindings.policy_identity",
                    &binding.policy_identity,
                )?;
                sha256(
                    "execution.reuse_bindings.obligations_identity",
                    &binding.obligations_identity,
                )?;
            }
            unique(
                "execution.debt_grants",
                debt_rows("execution.debt_grants", &part.debt_grants, agent_id)?,
            )?;
            if part
                .debt_grants
                .iter()
                .any(|row| row.authorized_at_version != version)
            {
                return Err(format!(
                    "execution.debt_grants must authorize version `{version}` only"
                ));
            }
            unique("execution.debt_retired", part.debt_retired.iter())?;
            if row_ids(&part.debt_grants).any(|id| part.debt_retired.contains(id)) {
                return Err("execution: a debt row is granted or retired, never both".to_string());
            }
        }

        for entry in &self.acceptance_entries {
            sha256("acceptance_entries.record_sha256", entry.record_sha256())?;
            match entry {
                AcceptanceEntry::MaintenanceCloseout { closeout, .. }
                | AcceptanceEntry::ProvingRunCloseout { closeout, .. } => {
                    file_ref("acceptance_entries.closeout", closeout)?
                }
                AcceptanceEntry::Publication { packet, .. } => {
                    file_ref("acceptance_entries.packet", packet)?
                }
                AcceptanceEntry::Promotion { targets, .. } => {
                    if targets.is_empty() {
                        return Err("acceptance_entries.targets must name a target".to_string());
                    }
                    unique("acceptance_entries.targets", targets.iter())?
                }
            }
        }
        Ok(())
    }
}

/// The identity of a part: the SHA-256 digest of the part with its `identity` member left out.
pub fn part_identity<T: Serialize>(part: &T) -> Result<String, String> {
    let mut value = json(part)?;
    if let Value::Object(members) = &mut value {
        members.remove("identity");
    }
    sha256_canonical(&value)
}

fn json<T: Serialize>(value: &T) -> Result<Value, String> {
    serde_json::to_value(value).map_err(|err| format!("depth record: serialize: {err}"))
}

fn sha256_canonical(value: &Value) -> Result<String, String> {
    let mut bytes = Vec::new();
    write_canonical(value, &mut bytes)?;
    Ok(hex::encode(Sha256::digest(bytes)))
}

/// RFC 8785 for the values a part holds. A part holds no number, and its member names are ASCII,
/// so sorting them by byte is the order the RFC requires and `serde_json` writes each string as
/// the RFC does.
fn write_canonical(value: &Value, out: &mut Vec<u8>) -> Result<(), String> {
    match value {
        Value::Null | Value::Bool(_) | Value::String(_) => {
            serde_json::to_writer(out, value).map_err(|err| format!("serialize: {err}"))?
        }
        Value::Number(_) => return Err("a part of a depth record holds no number".to_string()),
        Value::Array(items) => {
            out.push(b'[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.push(b',');
                }
                write_canonical(item, out)?;
            }
            out.push(b']');
        }
        Value::Object(members) => {
            let sorted: BTreeMap<_, _> = members.iter().collect();
            out.push(b'{');
            for (index, (name, member)) in sorted.into_iter().enumerate() {
                if index > 0 {
                    out.push(b',');
                }
                if !name.is_ascii() {
                    return Err(format!("member name `{name}` is not ASCII"));
                }
                write_canonical(&Value::String(name.clone()), out)?;
                out.push(b':');
                write_canonical(member, out)?;
            }
            out.push(b'}');
        }
    }
    Ok(())
}

/// A list the record derives is written in ascending order of its entries' canonical bytes. The
/// enrollment keeps the declaration's order, and acceptance entries the order they were made in.
fn ordered(at: &str, value: &Value) -> Result<(), String> {
    match value {
        Value::Array(items) => {
            let mut previous: Option<Vec<u8>> = None;
            for item in items {
                ordered(at, item)?;
                if at == "acceptance_entries" {
                    continue;
                }
                let mut bytes = Vec::new();
                write_canonical(item, &mut bytes)?;
                if previous.as_ref().is_some_and(|previous| *previous >= bytes) {
                    return Err(format!(
                        "`{at}` must list its entries in ascending order, without repeats"
                    ));
                }
                previous = Some(bytes);
            }
        }
        Value::Object(members) => {
            for (name, member) in members.iter().filter(|(name, _)| *name != "enrollment") {
                ordered(name, member)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn identity<T: Serialize>(part: &str, stated: &str, value: &T) -> Result<(), String> {
    if stated != part_identity(value)? {
        return Err(format!(
            "{part}.identity is not the digest of what the {part} part states"
        ));
    }
    Ok(())
}

pub(crate) fn sha256(field: &str, value: &str) -> Result<(), String> {
    let ok = value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
    if ok {
        Ok(())
    } else {
        Err(format!(
            "{field} must be a SHA-256 digest written as 64 lowercase hexadecimal characters"
        ))
    }
}

pub(crate) fn file_ref(field: &str, file: &FileRef) -> Result<(), String> {
    let path = &file.path;
    // An empty segment covers an empty path, a leading `/`, a trailing `/` and `//`.
    if path.contains('\\') || path.split('/').any(|part| matches!(part, "" | "." | "..")) {
        return Err(format!(
            "{field}: `{path}` must be a repository-relative path"
        ));
    }
    sha256(&format!("{field}: sha256 of `{path}`"), &file.sha256)
}

pub(crate) fn obligation_ref(field: &str, obligation: &ObligationRef) -> Result<(), String> {
    if obligation.id.is_some() == obligation.capability.is_some() {
        return Err(format!(
            "{field}: an obligation of `{}` states exactly one of `id` and `capability`, the \
             second for an `M` obligation",
            obligation.operation
        ));
    }
    Ok(())
}

/// The row ids, once every row is the agent's under B.6 rule 5 and every target of every row
/// is one both orders of B.6 rule 1 agree on.
pub(crate) fn debt_rows<'a>(
    field: &str,
    rows: &'a [DebtInventoryRow],
    agent_id: &str,
) -> Result<impl Iterator<Item = &'a String>, String> {
    // A surface names its agent only by the root of its command path, so both must name it.
    if rows
        .iter()
        .any(|row| row.agent_id != agent_id || row.command_path.split(' ').next() != Some(agent_id))
    {
        return Err(format!(
            "{field} must hold rows of agent `{agent_id}` only, on command paths rooted at it"
        ));
    }
    let plain =
        |target: &String| !target.contains(|c: char| c.is_whitespace() || "\"\\".contains(c));
    if rows
        .iter()
        .any(|row| !row.scope_target_triples.iter().all(plain))
    {
        return Err(format!(
            "{field}: a target holds no whitespace, `\"` or `\\`"
        ));
    }
    Ok(row_ids(rows))
}

fn row_ids(rows: &[DebtInventoryRow]) -> impl Iterator<Item = &String> {
    rows.iter().map(|row| &row.row_id)
}

fn unique<T: Ord>(field: &str, values: impl Iterator<Item = T>) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    for value in values {
        if !seen.insert(value) {
            return Err(format!("{field} must not repeat an entry"));
        }
    }
    Ok(())
}
