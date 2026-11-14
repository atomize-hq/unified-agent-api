//! Trusted collector dispatcher. No candidate executable or Cargo operation occurs on this path.
use super::*;
use crate::{depth_record::DepthRecord, git_inputs::GitInputs};

pub(crate) fn dispatch(
    root: &Path,
    publication: &Publication,
    candidate: &str,
    record: &DepthRecord,
) -> Result<Option<ContextRecord>, String> {
    let Some(invocation) = publication.host.reporting_invocation(candidate)? else {
        return Ok(None);
    };
    let invocation = invocation.normalized();
    if invocation.substantive_invoked {
        return Err("reporting dispatcher observed substantive execution".into());
    }
    if invocation
        .dispatch_inputs
        .get("depth_mode")
        .map(String::as_str)
        != Some(KIND)
    {
        return Err("reporting dispatcher inputs do not select reporting mode".into());
    }
    let mut objects = admission::objects::Objects::new(root);
    let head = objects.commit(candidate)?;
    if head.parents.len() != 1 {
        return Err("reporting candidate requires one authenticated predecessor".into());
    }
    let predecessor = objects.commit(&head.parents[0])?.object;
    let approved = authority(publication);
    policy::authenticate(root, &approved.policy, &approved.policy)?;
    let protected = publication.host.json("git/ref/heads/staging")?;
    let tip = protected["object"]["sha"]
        .as_str()
        .ok_or("reporting protected tip unavailable")?;
    let history = publication.resolve(root, &[tip.into(), candidate.into()])?;
    let predecessors = history.candidate_predecessors(candidate)?;
    let path = format!(
        "cli_manifests/{}/reports/{}/depth-record.json",
        record.policy.selection.agent_id, record.policy.selection.enrollment.version
    );
    let (candidate_record, bytes) = objects
        .blob(&head.object, &path)?
        .ok_or("reporting candidate lacks canonical generation record")?;
    let committed = DepthRecord::parse(
        std::str::from_utf8(&bytes).map_err(|e| e.to_string())?,
        &record.policy.selection.agent_id,
        &record.policy.selection.enrollment.version,
    )?;
    if Generation::from_record(&committed) != Generation::from_record(record) {
        return Err("reporting generation differs from candidate".into());
    }
    let mut context = ContextRecord {
        schema_version: 1,
        kind: KIND.into(),
        repository: approved.repository,
        generation: Generation::from_record(record),
        enrollment_anchor: approved.anchor,
        candidate: head.object.clone(),
        predecessor: predecessor.clone(),
        predecessor_admissions: predecessors,
        candidate_record,
        policy: approved.policy,
        head_relationship: if invocation.checkout == head.object {
            "exact_head"
        } else {
            "pr_test_merge"
        }
        .into(),
        source_observations: BTreeMap::from([
            ("candidate".into(), head.object.tree),
            ("checkout".into(), invocation.checkout.tree.clone()),
            ("predecessor".into(), predecessor.tree),
        ]),
        indexed_tree: invocation.checkout.tree.clone(),
        worktree_tree: invocation.checkout.tree.clone(),
        invocation,
        selected_mode: KIND.into(),
        enrolled_substantive_path_not_invoked: true,
        reporting_outcome: "success".into(),
        collection_complete: true,
    };
    context.check_objects(root)?;
    // Pin complete checkout/index/worktree observations before the reporting branch executes.
    let checkout =
        crate::depth_ci::output(crate::git_inputs::command(root).args(["rev-parse", "HEAD"]))?;
    if std::str::from_utf8(&checkout)
        .map_err(|e| e.to_string())?
        .trim()
        != context.invocation.checkout.commit
    {
        return Err("reporting actual checkout differs from host invocation".into());
    }
    let tree = freeze::read_tree(root, &context.invocation.checkout)?;
    let indexed = crate::depth_ci::output(
        crate::git_inputs::command(root).args(["ls-files", "--stage", "-z"]),
    )?;
    let paths = indexed
        .split(|byte| *byte == 0)
        .filter(|row| !row.is_empty())
        .map(|row| {
            let row = std::str::from_utf8(row).map_err(|e| e.to_string())?;
            row.split_once('\t')
                .map(|(_, path)| path.to_owned())
                .ok_or("reporting invalid index entry".into())
        })
        .collect::<Result<std::collections::BTreeSet<_>, String>>()?;
    if paths != tree.keys().cloned().collect() {
        return Err("reporting index inventory differs from checkout".into());
    }
    let mut view = GitInputs::new(root)?;
    view.capture_files(tree.keys().map(String::as_str))?;
    for (path, entry) in tree {
        if !entry.regular()
            || view.reference(&path)? != super::super::reference(&path, &entry.bytes)
            || view.executable(&path)? != (entry.mode == "100755")
        {
            return Err("reporting indexed/worktree observations differ from checkout".into());
        }
    }
    view.verify()?;
    history.recheck()?;
    // Selection is complete. The reporting-only branch inspects bounded data exclusively.
    // All identity-bearing inputs above stay pinned even if publication's closing read fails.
    if publication
        .host
        .reporting_invocation(candidate)?
        .map(Invocation::normalized)
        .as_ref()
        != Some(&context.invocation)
    {
        context.collection_complete = false;
        context.reporting_outcome = "failure".into();
    }
    let claims = &context.invocation.claims;
    let mut artifact = publication.host.upload(claims, &context.canonical()?)?;
    if artifact == 0 {
        return Err("reporting upload returned no artifact identity".into());
    }
    let closing = view
        .verify()
        .and_then(|()| history.recheck())
        .and_then(|()| {
            let head = crate::depth_ci::output(
                crate::git_inputs::command(root).args(["rev-parse", "HEAD"]),
            )?;
            if std::str::from_utf8(&head)
                .map_err(|e| e.to_string())?
                .trim()
                != context.invocation.checkout.commit
                || publication.host.reporting_execution(
                    context.invocation.claims.run,
                    context.invocation.claims.attempt,
                )?
                || publication
                    .host
                    .reporting_invocation(candidate)?
                    .map(Invocation::normalized)
                    .as_ref()
                    != Some(&context.invocation)
            {
                return Err("reporting closing checkout/dispatch changed".into());
            }
            Ok(())
        });
    if closing.is_err() {
        context.collection_complete = false;
        context.reporting_outcome = "failure".into();
        artifact = publication.host.upload(claims, &context.canonical()?)?;
        if artifact == 0 {
            return Err("reporting upload returned no artifact identity".into());
        }
    }
    publication.host.commit(
        claims,
        &transport::control_commitment(PREFIX, KIND, &context.identity()?, artifact),
    )?;
    let retained = authenticate(root, publication, claims.run, claims.attempt)?;
    if retained != context || !retained.successful() {
        return Err("reporting collection/publication incomplete".into());
    }
    Ok(Some(context))
}

/// IN1 pins this reference during prepare, before any owning policy predicate.
pub(crate) fn capture(
    root: &Path,
    publication: &Publication,
    candidate: &Object,
    parent: &Object,
    generation: &Generation,
    predecessors: &[String],
) -> Result<Option<ContextRecord>, String> {
    let Some(invocation) = publication.host.reporting_invocation(&candidate.commit)? else {
        return Ok(None);
    };
    let invocation = invocation.normalized();
    let context = authenticate(
        root,
        publication,
        invocation.claims.run,
        invocation.claims.attempt,
    )?;
    let execution = publication
        .host
        .reporting_execution(invocation.claims.run, invocation.claims.attempt);
    observations::unavailable("reporting/execution", &execution);
    let execution = execution?;
    observations::fact(
        "reporting/execution",
        "producer_context",
        &(context.identity()?, execution),
    );
    if execution {
        return Err("reporting context has substantive execution observation".into());
    }
    if context.candidate != *candidate
        || context.predecessor != *parent
        || context.generation != *generation
        || context.predecessor_admissions != predecessors
        || context.invocation != invocation
        || !context.successful()
    {
        return Err("reporting context candidate/predecessor/generation/dispatch mismatch".into());
    }
    Ok(Some(context))
}
