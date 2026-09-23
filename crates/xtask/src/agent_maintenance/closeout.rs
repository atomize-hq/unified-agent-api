#[path = "closeout/render.rs"]
mod render;
#[path = "closeout/support_audit_truth.rs"]
mod support_audit_truth;
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

use super::stand_down;

pub use self::support_audit_truth::{WrapperOnlyCategory, WrapperOnlyDisposition};
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
    require_stand_down(workspace_root, &args.request)?;
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

/// The admission gate (`uaa-0039`).
///
/// The nightly watcher regenerates a trailing packet every night, rewriting the request and
/// force-pushing the branch. That destroys committed work and invalidates a closeout bound to the
/// previous request even when the closeout was never committed. `AGENTS.md` and the generated
/// `HANDOFF.md` both instruct a maintainer to declare the freeze before their first adjudication —
/// but skipping `manifest-validate` fails CI while skipping the freeze fails nothing until the cron
/// job destroys the work. This is what converts the second kind of omission into the first.
///
/// Three properties, each of which the obvious implementation gets wrong:
///
/// - The question is whether *acquisition is established*, not whether a marker exists. The marker
///   is read from `origin/staging`, so one that is only in the working tree or only on the packet
///   branch — the branch the force-push replaces — is refused.
/// - A predicate error is never evidence of a declaration. An unreadable ref, a missing registry
///   entry or a malformed marker refuses, exactly as `maintenance-stand-down-check` treats them.
/// - Reuse, never a second parser. `stand_down::run` owns the schema, and its `StoodDown` outcome
///   is this gate's success case — the polarity is inverted because automation asks for permission
///   to mutate while a maintainer asks for protection before judging.
fn require_stand_down(
    workspace_root: &Path,
    request_path: &Path,
) -> Result<(), MaintenanceCloseoutError> {
    let loaded = load_request_artifact(workspace_root, request_path)?;
    // Without a detected release there is no packet generation for a marker to name, and no cron
    // job regenerates it. A manual closeout has nothing to freeze against.
    let Some(target_version) = loaded
        .request
        .detected_release
        .as_ref()
        .map(|release| release.target_version.clone())
    else {
        return Ok(());
    };

    let agent = loaded.request.agent_id.clone();
    let base = stand_down::BASE_BRANCH;
    let outcome = stand_down::run(stand_down::Args {
        agent: agent.clone(),
        target_version: target_version.clone(),
        from_ref: Some(format!("origin/{base}")),
        workspace_root: Some(workspace_root.to_path_buf()),
    })
    .map_err(|err| {
        MaintenanceCloseoutError::Validation(format!(
            "close-agent-maintenance cannot confirm the automation stand-down for `{agent}` \
             {target_version}, so it refuses: {err}"
        ))
    })?;

    if outcome == stand_down::StandDownOutcome::StoodDown {
        return Ok(());
    }

    let maintenance_root = loaded.maintenance_pack_root.display().to_string();
    let marker_dir = format!("{maintenance_root}/{}", stand_down::STAND_DOWN_RELATIVE_DIR);
    let marker_path = stand_down::marker_relative_path(&maintenance_root, &target_version);
    let marker = stand_down::render_marker_toml(
        &agent,
        &target_version,
        &loaded.request.request_recorded_at,
    );
    Err(MaintenanceCloseoutError::Validation(format!(
        "close-agent-maintenance refuses: automation still holds authority over `{agent}` \
         {target_version}. The nightly watcher can regenerate this packet and force-push over the \
         closeout. Declare the freeze on `{base}` first — never on the packet branch, which the \
         force-push replaces:\n\n\
         \x20 git switch {base} && git pull --ff-only\n\
         \x20 mkdir -p {marker_dir}\n\
         \x20 cat > {marker_path} <<'TOML'\n{marker}TOML\n\
         \x20 git add {marker_path}\n\
         \x20 git commit {marker_path} -m 'chore({agent}): stand automation down for {target_version}'\n\
         \x20 git push origin {base}\n\
         \x20 git switch -\n\n\
         If the tree is dirty and `git switch` refuses, use a throwaway worktree instead of \
         stashing packet work:\n\n\
         \x20 git worktree add ../standdown-{agent} {base}\n\
         \x20 cd ../standdown-{agent} && git pull --ff-only\n\
         \x20 mkdir -p {marker_dir} && cat > {marker_path} <<'TOML'\n{marker}TOML\n\
         \x20 git add {marker_path} && git commit {marker_path} -m 'chore({agent}): stand automation down for {target_version}'\n\
         \x20 git push origin {base}\n\
         \x20 cd - && git worktree remove ../standdown-{agent}\n\n\
         Then re-run this command. Confirm with: cargo run -p xtask -- \
         maintenance-stand-down-check --agent {agent} --target-version {target_version} \
         --from-ref origin/{base}"
    )))
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
