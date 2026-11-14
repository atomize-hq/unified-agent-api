//! Installed ON2 dispatch; a captured-route failure never falls back to production.
use super::*;

pub(in crate::runtime_follow_on) fn run(
    root: &Path,
    args: Args,
    route: crate::isolated_maintenance::Route,
    writer: &mut impl Write,
) -> Result<(), Error> {
    if args.dry_run == args.write {
        return Err(Error::Validation(
            "choose exactly one of --dry-run or --write".into(),
        ));
    }
    if let Some(run_id) = &args.run_id {
        crate::runtime_evidence_run::validate_run_id(run_id).map_err(Error::Validation)?;
    }
    validate_args(&args, root)?;
    let captured = route.capture(root).map_err(Error::Validation)?;
    // Includes the bootstrap case: unresolved enrollment is an error before any output.
    let raw = Source::workspace(root)
        .map_err(Error::Validation)?
        .optional_bytes(&args.approval)
        .map_err(Error::Validation)?
        .ok_or_else(|| Error::Validation("missing onboarding approval".into()))?;
    let approval =
        approval::execution(root, &raw, &args.approval, &captured).map_err(Error::Validation)?;
    captured.recheck(root).map_err(Error::Validation)?;
    if args.dry_run {
        return dry_run(root, args, writer);
    }
    let (path, state) = load_enrolled_lifecycle_state(root, &approval)?;
    let context = context::assemble(
        root,
        &args,
        approval,
        captured.registry(),
        path,
        state,
        true,
    )?;
    let prepared = load_json::<InputContract>(&context.run_dir.join(INPUT_CONTRACT_FILE_NAME))?;
    prepared.check_support_depth(root, &context.approval, &context.lifecycle_state)?;
    let prompt = read_string(&context.run_dir.join(PROMPT_FILE_NAME))?;
    write_header(writer, &context, true)?;
    depth_candidate::execute(root, &context, &args, &prepared, &prompt, writer)
}

pub(in crate::runtime_follow_on) fn wrapper_prompt(context: &RuntimeContext) -> String {
    format!("# Wrapper-only implementation\n\nRead {run}/input-contract.json. Implement and test the approved wrapper in `{crate_path}`. Preserve the frozen support_depth identities and all acquired files, including generated coverage. No backend or agent_api test is due.\n\n- Allowed source paths: {crate_path}/**\n- Update `{run}/handoff.json` so it passes validation with:\n  agent_id = {agent}; manifest_root = {root}; runtime_lane_complete = true; publication_refresh_required = true; blockers = [].\n- required_commands must match input-contract.json exactly.\n- Deliver substantive wrapper Rust source and wrapper tests; controller diagnostics are not implementation evidence.\n", run=context.run_dir.display(), crate_path=context.approval.descriptor.crate_path, agent=context.approval.descriptor.agent_id, root=context.approval.descriptor.manifest_root)
}

/// Check implementation surfaces actually due to this wrapper-only scope. Qualification of
/// obligations still requires the independent producer; source presence asserts no depth E.
pub(in crate::runtime_follow_on) fn check_implementation(
    root: &Path,
    context: &RuntimeContext,
    written: &[String],
    checks: &mut Vec<ValidationCheck>,
    errors: &mut Vec<String>,
) -> Result<(), Error> {
    let crate_path = &context.approval.descriptor.crate_path;
    let source = Source::workspace(root).map_err(Error::Validation)?;
    let mut implementation = false;
    for path in written {
        if path.starts_with(&format!("{crate_path}/src/")) && path.ends_with(".rs") {
            implementation |= source
                .optional(path)
                .map_err(Error::Validation)?
                .is_some_and(|s| !s.trim().is_empty());
        }
    }
    let mut tests = false;
    for path in source
        .paths(&format!("{crate_path}/tests"))
        .map_err(Error::Validation)?
    {
        if path.ends_with(".rs") {
            tests |= source
                .optional(&path)
                .map_err(Error::Validation)?
                .is_some_and(|s| !s.trim().is_empty());
        }
    }
    let ok = implementation && tests;
    let message =
        "wrapper-only implementation requires changed Rust source and nonempty wrapper tests"
            .to_string();
    if !ok {
        errors.push(message.clone());
    }
    checks.push(ValidationCheck {
        name: "wrapper_implementation".into(),
        ok,
        message,
    });
    Ok(())
}
