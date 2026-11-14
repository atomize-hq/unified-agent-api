//! Only a captured source-installed route can create the protected publication boundary.
use super::*;
use crate::depth_acceptance::admission::{self, emitter::Emitter, Authority, Policy, Publication};
use std::collections::BTreeMap;

impl Context {
    pub(crate) fn admission_publication(&self) -> Result<Publication, String> {
        let host = self.api.admissions()?;
        let anchor = admission::emitter::object(&self.root, &self.profile.baseline)?;
        // The source-installed installation commit is the authenticated protected policy
        // anchor, independent of both the candidate and producer-population discovery.
        let closure =
            admission::policy::closure(&self.root, &anchor, self.profile.scopes.iter().copied())?;
        let workflow_path = ".github/workflows/ci.yml";
        let policy = Policy {
            implementation: admission::policy::identity(&closure)?,
            workflow_path: workflow_path.into(),
            workflow_commit: self.profile.baseline.clone(),
            workflow_id: 71,
            job_key: "protected-in1".into(),
            app_id: 15368,
            closure,
        };
        Ok(Publication {
            authority: Authority {
                repository: admission::Repository {
                    name: self.profile.isolated.name.into(),
                    id: self.profile.isolated.id,
                    owner: self.profile.isolated.owner,
                },
                anchor,
                policy,
            },
            host,
        })
    }
    pub(crate) fn admission_emitter(&self) -> Result<Emitter, String> {
        let mut inputs = BTreeMap::new();
        let subject = self.profile.subject;
        let entry = self.registry.find(subject.agent_id);
        let state = entry.map(crate::agent_lifecycle::lifecycle_state_path_for_entry);
        let approval = entry.map(crate::agent_lifecycle::approval_artifact_path_for_entry);
        let record = format!(
            "{}/reports/{}/depth-record.json",
            entry.map_or_else(
                || format!("cli_manifests/{}", subject.agent_id),
                |e| e.manifest_root.clone()
            ),
            subject.version
        );
        for path in [
            Some(crate::agent_registry::REGISTRY_RELATIVE_PATH.to_owned()),
            state.clone(),
            approval,
            Some(record.clone()),
            Some(crate::agent_lifecycle::maintenance_request_path(
                subject.agent_id,
            )),
        ]
        .into_iter()
        .flatten()
        {
            inputs.insert(path, ());
        }
        Ok(Emitter {
            context: self.clone(),
            parent: None,
            prepared: None,
            publication: self.admission_publication()?,
            protected_tip: self.host.isolated_tip.clone(),
            canonical_tip: self.host.canonical_tip().into(),
            registry_blob: self.host.registry.clone(),
            candidate: self.workspace.head.clone(),
            state_path: state,
            record_path: record,
            agent: subject.agent_id.into(),
            version: subject.version.into(),
            input_paths: inputs.into_keys().collect(),
        })
    }
}
impl EvidenceHost {
    /// Independent from producer discovery and from any installed thread-local reader.
    pub(crate) fn admission_history(
        &self,
        root: &Path,
        candidates: &[String],
    ) -> Result<Option<admission::history::History>, String> {
        self.admission
            .as_ref()
            .map(|publication| {
                publication.resolve(
                    root,
                    &std::iter::once(self.tip.clone())
                        .chain(candidates.iter().cloned())
                        .collect::<Vec<_>>(),
                )
            })
            .transpose()
    }
}
