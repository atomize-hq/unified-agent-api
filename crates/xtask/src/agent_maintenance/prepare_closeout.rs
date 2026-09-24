//! T6 — `prepare-agent-closeout`.
//!
//! Composes T4's resolved CI evidence and T5's derived findings into a `maintenance-closeout.json`
//! that `close-agent-maintenance` accepts unmodified, and refuses rather than emitting an artifact
//! it cannot stand behind.
//!
//! # What this command does not decide
//!
//! Every field here is either resolved from evidence, derived from committed artifacts, or carried
//! forward from a prior adjudication. None is invented, and the refusals are what make that claim
//! checkable rather than merely stated:
//!
//! - **A red or unresolvable preflight refuses.** The spec's §3 and §6 both say the generator never
//!   emits `preflight_passed: false`, and §9's third criterion says it refuses and says why. T4 is
//!   a *resolver* and legitimately reports `passed: false`; this is the *generator* and converts
//!   that into a refusal. The `bool` in the schema stays meaningful because a hand-authored closeout
//!   may still carry `false` — the generator is simply stricter than the validator.
//! - **An unadjudicated wrapper-only row refuses.** A disposition answers why the wrapper claims a
//!   surface upstream's help does not show, and the four categories are indistinguishable from
//!   repository state alone: a surface upstream hides and a surface upstream removed produce an
//!   identical observation. Nothing here can tell them apart, so nothing here writes one. The
//!   refusal names each row and is the work queue.
//! - **A request with no detected release refuses.** T5's derivation is version-scoped and there is
//!   no version to scope it to.
//!
//! # Why it writes before it validates
//!
//! `load_linked_closeout` is the only public entry into the validator and it is path-addressed:
//! `maintenance_pack_prefix_from_closeout_path` requires exactly six components ending in the
//! literal `maintenance-closeout.json`, so a candidate written under a temporary name or into a
//! temporary directory is refused on its path before its bytes are read. Validating a candidate
//! elsewhere is therefore impossible without widening the validator, which spec §4 forbids — it is
//! the fixed authority T6 must satisfy unmodified.
//!
//! So the canonical path is written first and validated second, and a refusal restores the previous
//! bytes. Where no closeout existed there is nothing to restore, and the error names the file left
//! behind rather than deleting it: a maintainer who can see the rejected artifact can read why it
//! was rejected, and silent removal of a file this command just created is how a generator loses a
//! maintainer's trust. The guarantee that survives is the one that matters — the bytes validated
//! are the bytes on disk, never a second representation that happens to agree.
//!
//! # Why it does not build its own serializer
//!
//! `close-agent-maintenance` rewrites the closeout from `serialize_closeout_json` on every run. A
//! second emitter here would mean the artifact this command writes is silently not the artifact
//! that lands — the maintainer reviews one set of bytes and commits another. One writer, reused.

use std::{
    io::{self, Write},
    path::{Path, PathBuf},
};

use clap::Parser;

use crate::agent_registry::AgentRegistry;

use super::{
    closeout::{
        derive_findings, read_recorded_dispositions, reject_merge_commit, require_commit_binding,
        resolve_preflight_with_fetcher, serialize_closeout_json, DeferredFindingsTruth,
        MaintenanceCloseout, MaintenanceCloseoutError, WrapperOnlyDisposition,
    },
    support_audit::{wrapper_only_baseline, SurfaceIdentity},
};

#[derive(Debug, Parser)]
pub struct Args {
    #[arg(long)]
    pub request: PathBuf,

    /// The revision the CI evidence is resolved against. Must exist in this repository and be
    /// reachable from `HEAD` — the same binding `close-agent-maintenance` enforces.
    #[arg(long)]
    pub commit: String,

    /// RFC3339 UTC. An argument rather than a clock so the artifact is byte-reproducible when the
    /// committed value is re-supplied. `SOURCE_DATE_EPOCH` is deliberately not consulted: the
    /// acquisition workflow derives it from the upstream publish time.
    #[arg(long)]
    pub recorded_at: String,

    /// Write the artifact. Without it the command previews and touches nothing.
    #[arg(long)]
    pub write: bool,
}

pub fn run(args: Args) -> Result<(), MaintenanceCloseoutError> {
    let workspace_root = super::closeout::resolve_workspace_root()?;
    let mut stdout = io::stdout();
    run_in_workspace(&workspace_root, args, &mut stdout)
}

pub fn run_in_workspace<W: Write>(
    workspace_root: &Path,
    args: Args,
    writer: &mut W,
) -> Result<(), MaintenanceCloseoutError> {
    run_in_workspace_with_fetcher(workspace_root, args, writer, |url| {
        super::watch::fetch_text(url)
            .map_err(|err| MaintenanceCloseoutError::Internal(format!("fetch {url}: {err}")))
    })
}

/// The injected-evidence seam, matching `watch.rs` and `resolve_preflight_with_fetcher`.
///
/// Tests drive the whole command from frozen API responses rather than reaching the network, which
/// is what lets the refusal paths be exercised against real packet shapes.
pub fn run_in_workspace_with_fetcher<W: Write, F>(
    workspace_root: &Path,
    args: Args,
    writer: &mut W,
    fetch: F,
) -> Result<(), MaintenanceCloseoutError>
where
    F: FnMut(&str) -> Result<String, MaintenanceCloseoutError>,
{
    let prepared = build_closeout(workspace_root, &args, fetch)?;
    let bytes = serialize_closeout_json(&prepared.closeout)?;

    write_preview(writer, &prepared, args.write)?;
    if !args.write {
        return Ok(());
    }
    write_and_verify(workspace_root, &args.request, &prepared, bytes)?;
    writeln!(
        writer,
        "OK: wrote and validated {}",
        prepared.closeout_path.display()
    )
    .map_err(stdout_error)
}

struct PreparedCloseout {
    closeout: MaintenanceCloseout,
    closeout_path: PathBuf,
    agent_id: String,
    target_version: String,
    live_row_count: usize,
}

fn build_closeout<F>(
    workspace_root: &Path,
    args: &Args,
    fetch: F,
) -> Result<PreparedCloseout, MaintenanceCloseoutError>
where
    F: FnMut(&str) -> Result<String, MaintenanceCloseoutError>,
{
    let loaded = super::closeout::load_request_artifact(workspace_root, &args.request)?;
    let agent_id = loaded.request.agent_id.clone();
    let target_version = loaded
        .request
        .detected_release
        .as_ref()
        .map(|release| release.target_version.clone())
        .ok_or_else(|| {
            MaintenanceCloseoutError::Validation(format!(
                "{}: the request records no detected release, so there is no target version to \
                 derive findings against. A closeout is scoped to a release; this request is not.",
                loaded.request_path.display()
            ))
        })?;

    // Order matters, and it is the cheap-and-specific checks first. The binding runs before the
    // merge-commit shape check because `git rev-list` on a sha this repository has never contained
    // fails as a broken invocation rather than as an answer, which would surface a fabricated sha
    // as an internal error instead of the refusal that names it. Both are local, so both run before
    // the network.
    require_commit_binding(workspace_root, &args.commit)?;
    reject_merge_commit(workspace_root, &args.commit)?;
    let preflight = resolve_preflight_with_fetcher(&args.commit, fetch)?;
    if !preflight.passed {
        return Err(MaintenanceCloseoutError::Validation(format!(
            "CI for `{}` resolved to a failure, so there is no closeout to write. The generator \
             never records `preflight_passed: false` (spec §3): fix the packet and re-resolve \
             against the new head. Runs consulted: {}",
            args.commit,
            describe_runs(&preflight.runs)
        )));
    }

    let findings = derive_findings(workspace_root, &agent_id, &target_version)?;

    let registry = AgentRegistry::load(workspace_root)
        .map_err(|err| MaintenanceCloseoutError::Internal(format!("load agent registry: {err}")))?;
    let entry = registry.find(&agent_id).ok_or_else(|| {
        MaintenanceCloseoutError::Validation(format!("`{agent_id}` is not an enrolled agent"))
    })?;

    let closeout_path = loaded
        .maintenance_pack_root
        .join("governance/maintenance-closeout.json");
    let baseline =
        wrapper_only_baseline(workspace_root, entry, &target_version).map_err(|err| {
            MaintenanceCloseoutError::Validation(format!(
                "live wrapper-only re-derivation failed for `{agent_id}`: {err}"
            ))
        })?;
    let (baseline_ref, live_surfaces) = match baseline {
        Some((report_ref, surfaces)) => (Some(report_ref), surfaces),
        None => (None, Default::default()),
    };
    let dispositions = carry_dispositions(workspace_root, &closeout_path, &live_surfaces)?;

    Ok(PreparedCloseout {
        closeout: MaintenanceCloseout {
            request_ref: repo_relative_string(&loaded.request_path),
            request_sha256: loaded.request_sha256.clone(),
            resolved_findings: findings.resolved_findings,
            deferred_findings: findings.deferred_findings,
            wrapper_only_baseline_ref: baseline_ref,
            wrapper_only_dispositions: dispositions,
            preflight_passed: true,
            recorded_at: args.recorded_at.clone(),
            commit: args.commit.clone(),
        },
        closeout_path,
        agent_id,
        target_version,
        live_row_count: live_surfaces.len(),
    })
}

/// Carry forward the adjudications a prior closeout recorded, and refuse on any live row without
/// one.
///
/// A disposition is a judgement about a surface, so it survives a version change; that is what
/// makes carrying forward correct rather than merely convenient. Rows whose surface is no longer
/// live are dropped, because `check_dispositions_cover_live` rejects an entry for a surface the
/// report no longer carries — a row sorted `obsolete` contracts publication and then vanishes, and
/// keeping its disposition would make the next closeout invalid.
fn carry_dispositions(
    workspace_root: &Path,
    closeout_path: &Path,
    live_surfaces: &std::collections::BTreeSet<SurfaceIdentity>,
) -> Result<Vec<WrapperOnlyDisposition>, MaintenanceCloseoutError> {
    let recorded = read_recorded_dispositions(workspace_root, closeout_path)?;
    let carried: Vec<WrapperOnlyDisposition> = recorded
        .into_iter()
        .filter(|row| live_surfaces.contains(&row.surface))
        .collect();

    let missing: Vec<&SurfaceIdentity> = live_surfaces
        .iter()
        .filter(|surface| !carried.iter().any(|row| &row.surface == *surface))
        .collect();
    if !missing.is_empty() {
        return Err(MaintenanceCloseoutError::Validation(format!(
            "{} wrapper-only row(s) have no recorded disposition, so this closeout cannot be \
             generated. Each one is a judgement about why the wrapper claims a surface upstream's \
             help does not show, and nothing derivable from this repository distinguishes \
             `hidden_upstream_supported` from `obsolete`. Adjudicate each row into \
             `wrapper_only_dispositions` in {}, then re-run:\n{}",
            missing.len(),
            closeout_path.display(),
            missing
                .iter()
                .map(|surface| format!("  - {}", surface.describe()))
                .collect::<Vec<_>>()
                .join("\n")
        )));
    }
    Ok(carried)
}

/// Write the canonical path, validate it through the validator's real input path, and restore the
/// previous bytes if it refuses.
fn write_and_verify(
    workspace_root: &Path,
    request_path: &Path,
    prepared: &PreparedCloseout,
    bytes: Vec<u8>,
) -> Result<(), MaintenanceCloseoutError> {
    let resolved = workspace_root.join(&prepared.closeout_path);
    let previous = std::fs::read(&resolved).ok();
    if let Some(parent) = resolved.parent() {
        std::fs::create_dir_all(parent).map_err(|err| {
            MaintenanceCloseoutError::Internal(format!("create {}: {err}", parent.display()))
        })?;
    }
    std::fs::write(&resolved, &bytes).map_err(|err| {
        MaintenanceCloseoutError::Internal(format!(
            "write {}: {err}",
            prepared.closeout_path.display()
        ))
    })?;

    match super::closeout::load_linked_closeout(
        workspace_root,
        request_path,
        &prepared.closeout_path,
    ) {
        Ok(_) => Ok(()),
        Err(err) => Err(restore_and_report(
            &resolved,
            &prepared.closeout_path,
            previous,
            err,
        )),
    }
}

fn restore_and_report(
    resolved: &Path,
    closeout_path: &Path,
    previous: Option<Vec<u8>>,
    err: MaintenanceCloseoutError,
) -> MaintenanceCloseoutError {
    match previous {
        Some(bytes) => match std::fs::write(resolved, bytes) {
            Ok(()) => MaintenanceCloseoutError::Validation(format!(
                "the generated closeout did not validate, so {} is unchanged: {err}",
                closeout_path.display()
            )),
            Err(restore_err) => MaintenanceCloseoutError::Internal(format!(
                "the generated closeout did not validate ({err}), and restoring {} failed too: \
                 {restore_err}. That file now holds the rejected artifact.",
                closeout_path.display()
            )),
        },
        None => MaintenanceCloseoutError::Validation(format!(
            "the generated closeout did not validate: {err}. No closeout existed before this run, \
             so {} now holds the rejected artifact — it is left in place deliberately, so the \
             refusal can be read against it. Remove it once the cause is fixed.",
            closeout_path.display()
        )),
    }
}

fn write_preview<W: Write>(
    writer: &mut W,
    prepared: &PreparedCloseout,
    will_write: bool,
) -> Result<(), MaintenanceCloseoutError> {
    let closeout = &prepared.closeout;
    writeln!(
        writer,
        "prepare-agent-closeout: {} {} ({})",
        prepared.agent_id,
        prepared.target_version,
        if will_write { "write" } else { "preview" }
    )
    .map_err(stdout_error)?;
    writeln!(
        writer,
        "  path            {}",
        prepared.closeout_path.display()
    )
    .map_err(stdout_error)?;
    writeln!(writer, "  commit          {}", closeout.commit).map_err(stdout_error)?;
    writeln!(writer, "  recorded_at     {}", closeout.recorded_at).map_err(stdout_error)?;
    writeln!(writer, "  preflight       passed").map_err(stdout_error)?;
    writeln!(
        writer,
        "  resolved        {} finding(s)",
        closeout.resolved_findings.len()
    )
    .map_err(stdout_error)?;
    match &closeout.deferred_findings {
        DeferredFindingsTruth::ExplicitNone(reason) => {
            writeln!(writer, "  deferred        none ({reason})").map_err(stdout_error)?
        }
        DeferredFindingsTruth::Findings(findings) => {
            writeln!(writer, "  deferred        {} finding(s)", findings.len())
                .map_err(stdout_error)?
        }
    }
    writeln!(
        writer,
        "  wrapper-only    {} live row(s), {} disposition(s) carried",
        prepared.live_row_count,
        closeout.wrapper_only_dispositions.len()
    )
    .map_err(stdout_error)?;
    if !will_write {
        writeln!(writer, "Nothing written. Re-run with --write to apply.").map_err(stdout_error)?;
    }
    Ok(())
}

fn describe_runs(runs: &[super::closeout::EvidenceRun]) -> String {
    if runs.is_empty() {
        return "none".to_string();
    }
    runs.iter()
        .map(|run| {
            format!(
                "{} (attempt {}) {}",
                run.run_id, run.run_attempt, run.conclusion
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn repo_relative_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn stdout_error(err: io::Error) -> MaintenanceCloseoutError {
    MaintenanceCloseoutError::Internal(format!("write stdout: {err}"))
}
