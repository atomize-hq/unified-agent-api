//! ON1's finite bootstrap plan. Route absence continues through the ordinary loader.
use super::*;
use crate::isolated_maintenance::onboarding::Scaffold;

pub(super) fn validate_args(args: &Args, scaffold: &Scaffold) -> Result<(), Error> {
    if let Some(path) = &args.approval {
        if path != &scaffold.approval_path || args.has_semantic_descriptor_flags() {
            return Err(Error::Validation(
                "bootstrap requires the exact approval path without semantic descriptor flags"
                    .into(),
            ));
        }
    } else {
        let draft = DraftEntry::from_descriptor_input(DraftDescriptorInput::from_bootstrap_args(
            args.clone(),
        )?)?;
        if !validation::registry_entry_matches_draft(&scaffold.entry, &draft) {
            return Err(Error::Validation(
                "bootstrap descriptor differs from the compiled isolated subject".into(),
            ));
        }
    }
    Ok(())
}

pub(super) fn plan(
    root: &Path,
    scaffold: &Scaffold,
) -> Result<(DraftEntry, OnboardingPlan), Error> {
    let draft = DraftEntry::from_descriptor_input(approval::scaffold_descriptor(scaffold)?)?;
    // The bound approval states a version, but ON1 writes only its ordinary scaffold.
    // NE3 owns truthful metadata after acquisition; no declaration is rendered here.
    if draft.canonical_targets.len() != 1 {
        return Err(Error::Validation(
            "bootstrap requires one canonical target".into(),
        ));
    }
    execution::build_plan(root, || Ok(draft))
}

/// IN1 replays exactly the public plan against every regular blob of its pinned parent.
/// Its adapter has authenticated these compiled inputs; scratch resolves no route of its own.
pub(crate) fn write_scaffold(root: &Path, scaffold: &Scaffold) -> Result<(), Error> {
    let (_, plan) = plan(root, scaffold)?;
    apply_mutations(root, &plan.mutations)?;
    Ok(())
}
