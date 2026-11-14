//! Walk immutable parent edges, independent of CI population and API observation order.
use super::*;
use crate::agent_lifecycle::{LifecycleStage, LifecycleState};
use crate::runtime_advertising::source::Source;
use std::collections::BTreeMap;

struct Position {
    first: String,
    stage: LifecycleStage,
}

struct Commit {
    tree: String,
    parents: Vec<String>,
}

/// Invocation-local caches for one authenticated Event/P/O generation. A cached commit has
/// already passed its entire ancestry. Unchanged input blobs share their lifecycle projection.
#[derive(Default)]
pub(super) struct Onboarding {
    sources: BTreeMap<String, Option<Position>>,
    states: BTreeMap<Vec<u8>, Option<LifecycleStage>>,
    first: Option<String>,
}

impl Onboarding {
    /// Both API heads and actual checkouts participate. All parents matter, including the PR
    /// head in a test merge's second parent and staging merged into a PR branch.
    pub(super) fn observe(
        &mut self,
        root: &Path,
        commit: &str,
        source: &Source,
        p: Profile,
        record: &DepthRecord,
    ) -> Result<(), String> {
        if p != Profile::SyntheticOnboarding {
            return Ok(());
        }
        if !self.sources.contains_key(commit) {
            depth_ci::onboarding::check(source, p, record)?;
        }
        // Iterative postorder avoids recursion limits. No path-filtered log simplification
        // may erase a merge parent, a temporary rollback or a deleted/reintroduced binding.
        let mut pending = vec![(commit.to_owned(), None)];
        while let Some((oid, node)) = pending.pop() {
            if self.sources.contains_key(&oid) {
                continue;
            }
            let Some(node) = node else {
                let node = read_commit(root, &oid)?;
                let parents = node.parents.clone();
                pending.push((oid, Some(node)));
                pending.extend(parents.into_iter().map(|parent| (parent, None)));
                continue;
            };
            let stage = self.stage(root, &oid, &node.tree, p, record)?;
            let mut position: Option<Position> = None;
            for parent in &node.parents {
                let Some(prior) = self
                    .sources
                    .get(parent)
                    .ok_or("incomplete onboarding history")?
                else {
                    continue;
                };
                let next =
                    stage.ok_or("onboarding lifecycle history interrupts frozen Event/P/O")?;
                crate::runtime_follow_on::depth::check_progression(prior.stage, next)?;
                if position
                    .as_ref()
                    .is_some_and(|pos| pos.first != prior.first)
                {
                    return Err(
                        "ambiguous onboarding history: multiple first applicable states".into(),
                    );
                }
                position = Some(Position {
                    first: prior.first.clone(),
                    stage: next,
                });
            }
            if position.is_none() {
                position = stage.map(|stage| Position {
                    first: oid.clone(),
                    stage,
                });
            }
            self.sources.insert(oid, position);
        }
        let position = self
            .sources
            .get(commit)
            .and_then(Option::as_ref)
            .ok_or("source history lacks its applicable onboarding state")?;
        if self
            .first
            .as_ref()
            .is_some_and(|first| first != &position.first)
        {
            return Err(
                "ambiguous onboarding history: sources have different generation starts".into(),
            );
        }
        self.first = Some(position.first.clone());
        Ok(())
    }

    fn stage(
        &mut self,
        root: &Path,
        commit: &str,
        tree: &str,
        p: Profile,
        original: &DepthRecord,
    ) -> Result<Option<LifecycleStage>, String> {
        // Include every byte read by onboarding::check, even the release-watch registry.
        // Tree entries bind paths, modes and object ids. Source retains regular-file checks.
        let entries = crate::depth_ci::output(crate::git_inputs::command(root).args([
            "ls-tree",
            "-r",
            "-z",
            "--full-tree",
            tree,
            "--",
            p.fixture(),
            depth_ci::onboarding::STATE,
            depth_ci::onboarding::APPROVAL,
            crate::agent_registry::REGISTRY_RELATIVE_PATH,
        ]))?;
        if let Some(stage) = self.states.get(&entries) {
            return Ok(*stage);
        }
        // Never acquire history through a remote or consult working lifecycle bytes. Missing
        // ancestor objects refuse, including shallow boundaries concealed by revision walkers.
        let at = Source::tip(root, commit)?;
        let candidate = at
            .optional(p.fixture())?
            .map(|raw| DepthRecord::parse(&raw, p.agent_id(), p.version()))
            .transpose()?;
        let stage = if candidate.as_ref().is_some_and(|record| {
            record.event == original.event
                && record.policy == original.policy
                && record.obligations == original.obligations
        }) {
            depth_ci::onboarding::check(&at, p, original)?;
            let state: LifecycleState =
                serde_json::from_str(&at.read(depth_ci::onboarding::STATE)?)
                    .map_err(|e| e.to_string())?;
            Some(state.lifecycle_stage)
        } else {
            None
        };
        self.states.insert(entries, stage);
        Ok(stage)
    }
}

fn read_commit(root: &Path, commit: &str) -> Result<Commit, String> {
    sha(commit)?;
    // Raw object headers retain the real parent list even with shallow/graft metadata. The
    // hardened runner disables replacement objects and implicit promisor-object acquisition.
    let bytes = crate::depth_ci::output(
        crate::git_inputs::command(root).args(["cat-file", "commit", commit]),
    )
    .map_err(|e| format!("cannot read complete onboarding history at {commit}: {e}"))?;
    let headers = bytes
        .split(|byte| *byte == b'\n')
        .take_while(|line| !line.is_empty());
    let mut tree = None;
    let mut parents = vec![];
    for line in headers {
        if let Some(oid) = line.strip_prefix(b"tree ") {
            tree = Some(sha(std::str::from_utf8(oid).map_err(|e| e.to_string())?)?.to_owned());
        }
        if let Some(oid) = line.strip_prefix(b"parent ") {
            parents.push(sha(std::str::from_utf8(oid).map_err(|e| e.to_string())?)?.to_owned());
        }
    }
    Ok(Commit {
        tree: tree.ok_or("onboarding history commit lacks its tree")?,
        parents,
    })
}
