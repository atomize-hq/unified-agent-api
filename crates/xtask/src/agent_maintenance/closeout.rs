#[path = "closeout/render.rs"]
mod render;
#[path = "closeout/types.rs"]
mod types;
#[path = "closeout/validate.rs"]
mod validate;
#[path = "closeout/write.rs"]
mod write;

use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

pub(super) use self::types::maintenance_pack_root;
pub use self::types::{
    Args, CloseoutWriteSummary, DeferredFindingsTruth, LinkedMaintenanceCloseout,
    LoadedMaintenanceRequest, MaintenanceCloseout, MaintenanceCloseoutError,
    MaintenanceControlPlaneAction, MaintenanceDriftCategory, MaintenanceFinding,
    MaintenanceRequest, MaintenanceTriggerKind, RuntimeFollowupRequired,
};
pub use self::validate::{load_linked_closeout, load_request_artifact};
#[allow(unused_imports)]
pub(crate) use self::validate::{validate_live_drift_report, validate_live_drift_truth};
pub use self::write::{plan_closeout_mutations, write_closeout_outputs};

pub(super) const DOCS_NEXT_ROOT: &str = "docs/agents/lifecycle";
pub(super) const OWNERSHIP_MARKER: &str =
    "<!-- generated-by: xtask close-agent-maintenance; owner: maintenance-control-plane -->";

pub fn run(args: Args) -> Result<(), MaintenanceCloseoutError> {
    let workspace_root = resolve_workspace_root()?;
    let mut stdout = io::stdout();
    run_in_workspace(&workspace_root, args, &mut stdout)
}

pub fn run_in_workspace<W: Write>(
    workspace_root: &Path,
    args: Args,
    writer: &mut W,
) -> Result<(), MaintenanceCloseoutError> {
    let summary = write_closeout_outputs(workspace_root, &args.request, &args.closeout)?;
    writeln!(writer, "OK: close-agent-maintenance write complete.")
        .map_err(|err| MaintenanceCloseoutError::Internal(format!("write stdout: {err}")))?;
    writeln!(
        writer,
        "Refreshed 3 maintenance closeout surfaces for `{}` under `{}`.",
        summary.agent_id, summary.maintenance_pack_prefix
    )
    .map_err(|err| MaintenanceCloseoutError::Internal(format!("write stdout: {err}")))?;
    Ok(())
}

fn resolve_workspace_root() -> Result<PathBuf, MaintenanceCloseoutError> {
    let current_dir = std::env::current_dir()
        .map_err(|err| MaintenanceCloseoutError::Internal(format!("current_dir: {err}")))?;
    for candidate in current_dir.ancestors() {
        let cargo_toml = candidate.join("Cargo.toml");
        let Ok(text) = fs::read_to_string(&cargo_toml) else {
            continue;
        };
        if text.contains("[workspace]") {
            return Ok(candidate.to_path_buf());
        }
    }

    Err(MaintenanceCloseoutError::Internal(format!(
        "could not resolve workspace root from {}",
        current_dir.display()
    )))
}
