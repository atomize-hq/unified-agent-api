//! Finite structural semantics for the read-only Codex pilot. No policy rows are installed.
use super::{
    DepthEnrollmentDeclaration, LifecyclePath, ModeSelector, ObligationTemplate as T, SharedMapping,
};

pub const VERSION: &str = "0.156.1";
pub const TARGET: &str = "x86_64-unknown-linux-musl";

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

pub(crate) fn selection(agent: &str, d: &DepthEnrollmentDeclaration) -> Result<(), String> {
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

pub(crate) fn paths(mode: &ModeSelector) -> Vec<&str> {
    if mode.command_path == "codex exec resume" {
        vec!["codex exec", "codex exec resume", "codex app-server"]
    } else if mode.command_path == "codex exec" {
        vec!["codex exec", "codex app-server"]
    } else {
        vec![mode.command_path.as_str()]
    }
}

/// Structural ownership only. The ordinary resolver still refuses these overlapping modes,
/// and acquisition must independently find each exact identity in the complete bound population.
pub(super) fn owns_supplemental_surface(
    agent: &str,
    declaration: &DepthEnrollmentDeclaration,
    operation: &super::OperationDeclaration,
    surface: &crate::agent_maintenance::support_audit::SurfaceIdentity,
) -> bool {
    selection(agent, declaration).is_ok()
        && declaration
            .modes
            .iter()
            .filter(|mode| operation.serves.contains(&mode.selector))
            .any(|mode| paths(mode).contains(&surface.command_path.as_str()))
}
