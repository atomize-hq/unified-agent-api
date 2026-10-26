//! Read-only interpretation of the fixed pilot. This module installs no writer profile.
mod context;
mod routes;
pub use routes::{decode_route, PilotRoute, PilotTransport};

use super::{freeze, FreezePlan, FreezeRequest};
use crate::agent_registry::support_depth::{
    DepthEnrollmentDeclaration, LifecyclePath, ModeSelector, ObligationTemplate as T, SharedMapping,
};
use std::path::Path;

pub const VERSION: &str = "0.156.1";
pub const TARGET: &str = "x86_64-unknown-linux-musl";

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

// These are finite boundary semantics, not an alternate inventory of approved surface rows.
const MODES: [(&str, &str, &str, &str); 5] = [
    (
        "initial-json",
        "codex exec",
        "adapter-initial-json",
        "run-initial",
    ),
    (
        "resume-last-json",
        "codex exec resume",
        "adapter-resume-last-json",
        "run-resume",
    ),
    (
        "resume-id-json",
        "codex exec resume",
        "adapter-resume-id-json",
        "run-resume",
    ),
    (
        "fork-last-stdio",
        "codex app-server",
        "adapter-fork-last-stdio",
        "run-fork",
    ),
    (
        "fork-id-stdio",
        "codex app-server",
        "adapter-fork-id-stdio",
        "run-fork",
    ),
];

pub(super) fn selection(agent: &str, d: &DepthEnrollmentDeclaration) -> Result<(), String> {
    if agent != "codex"
        || d.version != VERSION
        || d.lifecycle_path != LifecyclePath::Maintenance
        || d.targets != [TARGET]
        || d.modes.len() != 5
        || d.operations.len() != 3
        || d.capabilities.len() != 1
        || d.capabilities[0].id != "agent_api.run"
        || d.capabilities[0].operations != ["run-initial", "run-resume", "run-fork"]
    {
        return Err(
            "pilot requires the exact complete three-operation/five-selector mapping".into(),
        );
    }
    for (selector, path, label, _) in MODES {
        let m = d
            .modes
            .iter()
            .find(|m| m.selector == selector)
            .ok_or("missing pilot selector")?;
        if m.command_path != path
            || m.mode != label
            || m.targets.as_ref().is_some_and(|ts| ts != &[TARGET])
            || !m.predicates.is_empty()
        {
            return Err(
                "pilot mode requires its exact typed discriminator, without arbitrary predicates"
                    .into(),
            );
        }
    }
    for (id, templates) in [
        ("run-initial", vec![T::Core, T::Admin, T::Managed]),
        (
            "run-resume",
            vec![T::Core, T::Session, T::Admin, T::Managed],
        ),
        ("run-fork", vec![T::Core, T::Session, T::Managed, T::Admin]),
    ] {
        let op = d
            .operations
            .iter()
            .find(|o| o.id == id)
            .ok_or("missing pilot operation")?;
        let served: Vec<_> = MODES
            .iter()
            .filter(|m| m.3 == id)
            .map(|m| m.0.to_string())
            .collect();
        let actual: std::collections::BTreeSet<_> = op.templates.iter().collect();
        if op.serves != served
            || actual != templates.iter().collect()
            || op.shared_mapping != Some(SharedMapping::Mapped)
            || op.forwarding != Some(false)
            || !op.not_applicable_permitted.is_empty()
        {
            return Err(
                "pilot operation has incomplete effects, mapping or selector coverage".into(),
            );
        }
    }
    Ok(())
}

pub(super) fn paths(mode: &ModeSelector) -> Vec<&str> {
    if mode.command_path == "codex exec resume" {
        vec!["codex exec", "codex exec resume"]
    } else {
        vec![mode.command_path.as_str()]
    }
}

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
