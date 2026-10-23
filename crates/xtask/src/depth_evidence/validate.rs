use std::collections::BTreeSet;

use super::*;
pub(super) use crate::depth_record::sha256;
use crate::depth_record::{file_ref, obligation_ref};

impl EvidenceReceipt {
    pub(super) fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err("unsupported depth evidence schema_version".into());
        }
        let scope = &self.scope;
        text(&scope.agent_id)?;
        text(&scope.version)?;
        let version = semver::Version::parse(&scope.version).map_err(|e| e.to_string())?;
        if !version.pre.is_empty()
            || !version.build.is_empty()
            || version.to_string() != scope.version
        {
            return Err("evidence version must be canonical MAJOR.MINOR.PATCH".into());
        }
        match &scope.event {
            Event::Maintenance {
                request_commit,
                request_recorded_at,
                trigger_kind,
                opened_from,
            } => {
                for field in [
                    request_commit,
                    request_recorded_at,
                    trigger_kind,
                    opened_from,
                ] {
                    text(field)?;
                }
            }
            Event::Onboarding {
                approval_commit,
                approval_recorded_at,
            } => {
                text(approval_commit)?;
                text(approval_recorded_at)?;
            }
        }
        obligation(&scope.obligation)?;
        sha256("policy_identity", &scope.policy_identity)?;
        sha256("obligations_identity", &scope.obligations_identity)?;
        for revision in [
            &scope.rules_revision,
            &scope.trust_revision,
            &scope.review_revision,
        ] {
            text(revision)?;
        }
        let producer = &self.producer;
        repository(&producer.repository)?;
        path(&producer.workflow_path)?;
        if !producer.workflow_path.starts_with(".github/workflows/")
            || !(producer.workflow_path.ends_with(".yml")
                || producer.workflow_path.ends_with(".yaml"))
        {
            return Err("producer workflow must name .github/workflows/*.yml or *.yaml".into());
        }
        git_identity(&producer.workflow_commit)?;
        file(&producer.collector)?;
        positive(producer.run_id)?;
        positive(producer.run_attempt)?;
        text(&producer.job)?;
        text(&producer.artifact_name)?;
        let build = &self.build;
        git_identity(&build.source_commit)?;
        git_identity(&build.source_tree)?;
        text(&build.host_target)?;
        file(&build.toolchain)?;
        file(&build.environment)?;
        file(&build.dependency_graph)?;
        strings(&build.features)?;
        files(&build.covered_inputs)?;
        strings(&build.absent_inputs)?;
        for absent in &build.absent_inputs {
            path(absent)?;
            if build
                .covered_inputs
                .iter()
                .chain([
                    &build.toolchain,
                    &build.environment,
                    &build.dependency_graph,
                ])
                .any(|f| &f.path == absent)
            {
                return Err(format!("input is both covered and absent: {absent}"));
            }
        }
        unique(&self.provenance, |p| &p.file.path)?;
        for source in &self.provenance {
            text(&source.origin)?;
            file(&source.file)?;
        }
        unique(&self.cases.requested, |c| c)?;
        for case in &self.cases.requested {
            case_identity(case)?;
        }
        unique(&self.cases.executed, |r| &r.case)?;
        for result in &self.cases.executed {
            case_identity(&result.case)?;
        }
        file(&self.cases.raw_output)?;
        let mapping = &self.adequacy.mapping;
        obligation(&mapping.obligation)?;
        text(&mapping.reviewer_identity)?;
        text(&mapping.executor_identity)?;
        unique(&mapping.assertions, |a| &a.case)?;
        for assertion in &mapping.assertions {
            case_identity(&assertion.case)?;
            file(&assertion.assertion)?;
            files(&assertion.production_inputs)?;
            text(&assertion.counterfactual)?;
        }
        let endorsement = &self.adequacy.endorsement;
        repository(&endorsement.repository)?;
        positive(endorsement.pull_request)?;
        positive(endorsement.id)?;
        text(&endorsement.author)?;
        git_identity(&endorsement.source_commit)?;
        sha256("body_sha256", &endorsement.body_sha256)?;
        sha256("mapping_sha256", &endorsement.mapping_sha256)?;
        if endorsement.mapping_sha256 != mapping.sha256()? {
            return Err("adequacy mapping_sha256 disagrees with mapping content".into());
        }
        Ok(())
    }
}

pub(super) fn text(value: &str) -> Result<(), String> {
    if value.trim().is_empty() || value.len() > 4096 || value.contains('\0') {
        return Err("evidence string must be nonblank, NUL-free and at most 4096 bytes".into());
    }
    Ok(())
}

pub(super) fn path(value: &str) -> Result<(), String> {
    text(value)?;
    if value.contains('\\')
        || value
            .split('/')
            .any(|s| matches!(s, "" | "." | ".." | ".git"))
    {
        return Err(format!("unsafe repository-relative evidence path: {value}"));
    }
    Ok(())
}

pub(super) fn file(value: &FileRef) -> Result<(), String> {
    path(&value.path)?;
    file_ref("depth evidence", value)
}

pub(super) fn unique<'a, T, K: Ord + ?Sized + 'a>(
    values: &'a [T],
    key: impl Fn(&'a T) -> &'a K,
) -> Result<(), String> {
    if values.len() > 4096 || values.iter().map(key).collect::<BTreeSet<_>>().len() != values.len()
    {
        return Err("evidence list exceeds 4096 entries or repeats an identity".into());
    }
    Ok(())
}

pub(super) fn strings(values: &[String]) -> Result<(), String> {
    unique(values, |v| v)?;
    for value in values {
        text(value)?;
    }
    Ok(())
}

pub(super) fn files(values: &[FileRef]) -> Result<(), String> {
    unique(values, |v| &v.path)?;
    for value in values {
        file(value)?;
    }
    Ok(())
}

fn obligation(value: &ObligationRef) -> Result<(), String> {
    obligation_ref("depth evidence", value)?;
    text(&value.operation)?;
    text(&value.target)?;
    if let Some(capability) = &value.capability {
        text(capability)?;
    }
    Ok(())
}

fn case_identity(value: &CaseIdentity) -> Result<(), String> {
    text(&value.name)?;
    sha256("case executable_sha256", &value.executable_sha256)
}

pub(super) fn git_identity(value: &str) -> Result<(), String> {
    if value.len() != 40
        || !value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err("evidence Git identity must be 40 lowercase hexadecimal characters".into());
    }
    Ok(())
}

fn positive(value: u64) -> Result<(), String> {
    if value == 0 {
        return Err("evidence host id/attempt must be positive".into());
    }
    Ok(())
}

fn repository(value: &str) -> Result<(), String> {
    text(value)?;
    let parts: Vec<_> = value.split('/').collect();
    if parts.len() != 2
        || parts.iter().any(|part| {
            part.is_empty()
                || matches!(*part, "." | "..")
                || !part
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
        })
    {
        return Err("evidence repository must be owner/name".into());
    }
    Ok(())
}
