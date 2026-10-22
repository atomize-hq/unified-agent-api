//! Request bindings only: neither table presence nor a matching record supplies write authority.
use super::{validate::validate_sha256_value, MaintenanceRequestError};
use crate::{
    agent_registry::{support_depth::LifecyclePath, AgentRegistryEntry},
    depth_record::{DepthRecord, Event, RequestBinding},
    workspace_mutation::WorkspacePathJail,
};
use serde::{Deserialize, Serialize};
use std::{fs, io, path::Path};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SupportDepthBinding {
    pub policy_identity: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub obligations_identity: Option<String>,
}

impl SupportDepthBinding {
    fn validate(&self, path: &Path) -> Result<(), MaintenanceRequestError> {
        validate_sha256_value(path, "support_depth.policy_identity", &self.policy_identity)?;
        if let Some(identity) = &self.obligations_identity {
            validate_sha256_value(path, "support_depth.obligations_identity", identity)?;
        }
        Ok(())
    }

    pub(crate) fn render(&self) -> Result<String, String> {
        self.validate(Path::new("<rendered request>"))
            .map_err(|e| e.to_string())?;
        toml_edit::ser::to_string(self)
            .map(|fields| format!("[support_depth]\n{fields}\n"))
            .map_err(|e| e.to_string())
    }
}

// Shared by both validating loader policies. This checks enrollment presence and frozen
// Event/P/O agreement, not policy currentness, evidence adequacy, execution or write admission.
pub(super) fn validate_binding(
    jail: &WorkspacePathJail,
    path: &Path,
    entry: &AgentRegistryEntry,
    artifact_version: &str,
    version: Option<&str>,
    event: Event,
    table: Option<&SupportDepthBinding>,
) -> Result<(), MaintenanceRequestError> {
    let invalid = |reason: &str| {
        MaintenanceRequestError::Validation(format!(
            "maintenance request `{}` support_depth: {reason}",
            path.display()
        ))
    };
    if let Some(table) = table {
        table.validate(path)?;
    }
    let Some(version) = version else {
        return if table.is_some() {
            Err(invalid("requires a detected target version"))
        } else {
            Ok(())
        };
    };
    let record_path = format!(
        "{}/reports/{version}/depth-record.json",
        entry.manifest_root
    );
    let absolute = jail
        .resolve(Path::new(&record_path))
        .map_err(super::validate::map_jail_error)?;
    let record = match fs::read_to_string(&absolute) {
        Ok(raw) => {
            Some(DepthRecord::parse(&raw, &entry.agent_id, version).map_err(|e| invalid(&e))?)
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => None,
        Err(error) => return Err(invalid(&format!("read {record_path}: {error}"))),
    };
    let declaration = entry
        .support_depth
        .as_ref()
        .and_then(|d| d.enrollments.as_ref())
        .and_then(|enrollments| enrollments.iter().find(|e| e.version == version));
    let Some(declaration) = declaration else {
        if record.is_some() {
            return Err(invalid("missing policy for retained depth record"));
        }
        return if table.is_some() {
            Err(invalid("table must be absent for an ordinary request"))
        } else {
            Ok(())
        };
    };
    if declaration.lifecycle_path != LifecyclePath::Maintenance {
        return Err(invalid(
            "target version is enrolled on another lifecycle path",
        ));
    }
    if artifact_version != "2" {
        return Err(invalid("requires artifact_version 2"));
    }
    let table = table.ok_or_else(|| invalid("table is required for a depth-enrolled request"))?;
    let record = record.ok_or_else(|| invalid("target version's depth record is missing"))?;
    let binding = RequestBinding {
        event,
        policy_identity: Some(table.policy_identity.clone()),
        obligations_identity: table.obligations_identity.clone(),
    };
    if let Some(reason) = record.request_mismatch(&binding) {
        return Err(invalid(reason));
    }
    Ok(())
}
