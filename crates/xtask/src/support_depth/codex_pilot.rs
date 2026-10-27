//! Read-only interpretation of the fixed pilot. This module installs no writer profile.
mod context;
mod routes;
pub use routes::{decode_route, PilotRoute, PilotTransport};

use super::{freeze, FreezePlan, FreezeRequest};
use crate::agent_registry::support_depth::ModeSelector;
use std::path::Path;

pub use crate::agent_registry::support_depth::pilot_scope::{TARGET, VERSION};

#[derive(Debug)]
pub struct PilotReadiness {
    /// Diagnostic candidate P/O only; never write admission or an execution verdict.
    pub plan: FreezePlan,
    pub remaining: Vec<String>,
}

/// Interpret a complete candidate declaration from the coherent registry view. Ordinary freeze,
/// closeout, promotion and the uninstalled isolated writer profile keep their existing refusals.
pub fn readiness(root: &Path, request: &FreezeRequest) -> Result<Option<PilotReadiness>, String> {
    if request.agent != "codex" || request.version != VERSION {
        return Err("pilot readiness requires Codex 0.156.1".into());
    }
    let Some(plan) = freeze::plan_pilot(root, request)? else {
        return Ok(None);
    };
    let o = plan
        .record
        .obligations
        .as_ref()
        .ok_or("pilot requires complete acquisition")?;
    let mut remaining = vec![
        "controlled context and effective defaults require runtime observation".into(),
        "packet-specific producer, native execution and independent adequacy are not established"
            .into(),
        "isolated writer profile and maintenance-chain proof are not installed".into(),
    ];
    for unit in &o.classification_required {
        remaining.push(format!(
            "classification_required: {}: {}",
            unit.operation,
            unit.surface.describe()
        ));
    }
    Ok(Some(PilotReadiness { plan, remaining }))
}

pub(super) use crate::agent_registry::support_depth::pilot_scope::{paths, selection};

pub(super) fn validate_mode(
    mode: &ModeSelector,
    shard: &serde_json::Value,
    union: &serde_json::Value,
) -> Result<(), String> {
    if mode.command_path == "codex app-server" {
        return Ok(());
    }
    // --json is emitted on exec, including when resume is selected. The child --last flag is
    // separately acquired; typed last/id discrimination never guesses an inherited CLI grammar.
    for (path, flag) in std::iter::once(("codex exec", "--json"))
        .chain((mode.selector == "resume-last-json").then_some(("codex exec resume", "--last")))
    {
        super::acquisition_modes::presence("codex", path, flag, flag, shard, union)?;
    }
    Ok(())
}

pub(super) use context::load;
