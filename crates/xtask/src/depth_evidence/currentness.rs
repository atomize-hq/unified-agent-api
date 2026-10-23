use super::{payload::DependencyGraph, EvidenceReceipt};
use crate::{git_inputs::GitInputs, runtime_advertising::source::Source};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, path::Path};

pub(super) fn check(
    workspace: &Path,
    view: &mut GitInputs,
    receipt: &EvidenceReceipt,
    graph: &DependencyGraph,
) -> Result<(), String> {
    let source = Source::tip(workspace, &receipt.build.source_commit)?;
    if source.tree()? != receipt.build.source_tree {
        return Err("actual built source tree mismatch".into());
    }
    for file in &graph.inputs {
        let current = view.required(&file.path)?;
        let built = source
            .optional_bytes(&file.path)?
            .ok_or("missing built dependency")?;
        if hex::encode(Sha256::digest(&current)) != file.sha256
            || built != current
            || source.executable(&file.path)? != view.executable(&file.path)?
        {
            return Err(format!("changed built/current dependency: {}", file.path));
        }
    }
    for file in [
        &receipt.build.toolchain,
        &receipt.cases.raw_output,
        &receipt.producer.collector,
    ] {
        let current = view.required(&file.path)?;
        if hex::encode(Sha256::digest(&current)) != file.sha256 {
            return Err(format!("changed referenced evidence: {}", file.path));
        }
    }
    for path in &receipt.build.absent_inputs {
        if view.optional(path)?.is_some() || source.optional_bytes(path)?.is_some() {
            return Err(format!("optional input is present: {path}"));
        }
    }
    for scope in &graph.scopes {
        let prefix = format!("{scope}/");
        let claimed: BTreeSet<_> = graph
            .inputs
            .iter()
            .filter(|f| f.path.starts_with(&prefix))
            .map(|f| f.path.clone())
            .collect();
        let built: BTreeSet<_> = source.committed_files(scope)?.into_iter().collect();
        let current: BTreeSet<_> = view.files_under(scope)?.into_iter().collect();
        if built != claimed || current != claimed {
            return Err(format!("incomplete or changed dependency scope: {scope}"));
        }
    }
    Ok(())
}
