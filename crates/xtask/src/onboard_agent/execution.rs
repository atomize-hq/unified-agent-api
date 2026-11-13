//! Shared preview and expected-before delivery for ordinary and routed ON1.
use super::*;

pub(super) fn run<W: Write>(
    workspace_root: &Path,
    args: Args,
    writer: &mut W,
) -> Result<(), Error> {
    let captured = crate::isolated_maintenance::route(workspace_root)
        .map_err(Error::Validation)?
        .map(|route| route.capture(workspace_root))
        .transpose()
        .map_err(Error::Validation)?;
    let dry_run_mode = args.dry_run;
    let (draft, plan) = if let Some(captured) = &captured {
        let scaffold = captured
            .onboarding_scaffold(workspace_root)
            .map_err(Error::Validation)?;
        let Some(scaffold) = scaffold else {
            xtask::depth_routes::refuse_workspace(workspace_root, "onboard_agent")
                .map_err(Error::Validation)?;
            return Err(Error::Validation(
                "ON1 requires onboarding bootstrap".into(),
            ));
        };
        routing::validate_args(&args, &scaffold)?;
        routing::plan(workspace_root, &scaffold)?
    } else {
        if !args.dry_run {
            xtask::depth_routes::refuse_workspace(workspace_root, "onboard_agent")
                .map_err(Error::Validation)?;
        }
        build_plan(workspace_root, || {
            let draft = DraftEntry::from_args(args, workspace_root)?;
            preview::refuse_unscaffoldable_manifest_root(&draft)?;
            Ok(draft)
        })?
    };

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
    write_lifecycle_state_preview(writer, &plan.lifecycle_state_preview)?;
    write_manifest_preview(writer, &plan.manifest_preview)?;
    write_release_preview(writer, &plan.release_preview)?;
    write_manual_follow_up(writer, &plan.manual_follow_up)?;

    if let Some(captured) = &captured {
        captured
            .recheck(workspace_root)
            .map_err(Error::Validation)?;
    }
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

pub(super) fn build_plan(
    workspace_root: &Path,
    descriptor: impl FnOnce() -> Result<DraftEntry, Error>,
) -> Result<(DraftEntry, OnboardingPlan), Error> {
    let jail = WorkspacePathJail::new(workspace_root)?;
    let registry_path = jail.resolve(Path::new(REGISTRY_RELATIVE_PATH))?;
    let registry_text = fs::read_to_string(&registry_path)
        .map_err(|err| Error::Internal(format!("read {REGISTRY_RELATIVE_PATH}: {err}")))?;
    let registry = AgentRegistry::parse(&registry_text).map_err(map_registry_load_error)?;
    let draft = descriptor()?;
    validate_registry_conflicts(&registry, &draft)?;
    validate_workspace_package_name_conflicts(&draft, &jail)?;
    validate_filesystem_conflicts(&draft, &jail)?;
    let plan = OnboardingPlan::build(workspace_root, &registry, &draft, &registry_text)?;
    Ok((draft, plan))
}
