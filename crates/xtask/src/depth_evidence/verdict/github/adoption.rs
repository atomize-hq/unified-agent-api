use super::{profile::*, transport::*, *};
use crate::depth_ci::profile::Profile;
use crate::depth_ci::{self, Witness};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::depth_evidence::verdict) struct Body {
    pub(in crate::depth_evidence::verdict) schema_version: u64,
    pub(in crate::depth_evidence::verdict) source_commit: String,
    pub(in crate::depth_evidence::verdict) mappings: Vec<Entry>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::depth_evidence::verdict) struct Entry {
    pub(in crate::depth_evidence::verdict) mapping_sha256: String,
    pub(in crate::depth_evidence::verdict) reviewer_identity: String,
    pub(in crate::depth_evidence::verdict) executor_identity: String,
    pub(in crate::depth_evidence::verdict) independence_attested: bool,
    pub(in crate::depth_evidence::verdict) counterfactual_adopted: bool,
}

pub(super) fn resolve(
    api: &impl Api,
    root: &Path,
    pr: u64,
    w: &Witness,
    p: Profile,
) -> Result<Vec<(AdequacyMapping, Adoption)>, String> {
    let comments = list(api, &format!("issues/{pr}/comments"), None)?;
    let reviews = list(api, &format!("pulls/{pr}/reviews"), None)?;
    let original = original_for(&source(root, &w.build.source_commit)?, p)?;
    let obligations = p.obligations(&original);
    let mut adopted = vec![];
    let mut mappings = std::collections::BTreeMap::new();
    for (kind, values) in [
        (EndorsementKind::IssueComment, comments),
        (EndorsementKind::PrReview, reviews),
    ] {
        for value in values {
            if kind == EndorsementKind::PrReview
                && !matches!(value["state"].as_str(), Some("APPROVED" | "COMMENTED"))
            {
                continue;
            }
            let Some(raw) = value["body"].as_str() else {
                continue;
            };
            let Some(body) = body(raw)? else { continue };
            if body.schema_version != 1 || body.mappings.is_empty() {
                continue;
            }
            sha(&body.source_commit)?;
            if kind == EndorsementKind::PrReview && value["commit_id"] != body.source_commit {
                continue;
            }
            let reviewed = source(root, &body.source_commit)?;
            let built = source(root, &w.build.source_commit)?;
            if original_for(&reviewed, p)? != original_for(&built, p)? {
                continue;
            }
            let input_paths: Vec<_> = if p == Profile::Channel {
                vec![
                    depth_ci::ASSERTION,
                    depth_ci::PRODUCTION,
                    depth_ci::CONTRACT,
                ]
            } else {
                w.graph.inputs.iter().map(|f| f.path.as_str()).collect()
            };
            let same_inputs=input_paths.iter().all(|path| matches!((reviewed.optional_bytes(path),built.optional_bytes(path),reviewed.executable(path),built.executable(path)),(Ok(Some(a)),Ok(Some(b)),Ok(am),Ok(bm)) if a==b && am==bm));
            let same_requirement_population = match p.requirements() {
                Some(scope) => reviewed.committed_files(scope)? == built.committed_files(scope)?,
                None => true,
            };
            if !same_inputs || !same_requirement_population {
                continue;
            }
            let author = required_string(&value["user"], "login")?;
            if !author
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-')
            {
                return Err("invalid GitHub author".into());
            }
            let permission = api.json(&endpoint(&format!("collaborators/{author}/permission")))?;
            if !matches!(
                permission["permission"].as_str(),
                Some("admin" | "write" | "maintain")
            ) {
                continue;
            }
            for entry in body.mappings {
                if !entry.independence_attested
                    || !entry.counterfactual_adopted
                    || entry.executor_identity != p.executor()
                    || entry.reviewer_identity == p.executor()
                    || entry.reviewer_identity.trim().is_empty()
                {
                    continue;
                }
                if !mappings.contains_key(&entry.reviewer_identity) {
                    let bound = obligations
                        .iter()
                        .map(|obligation| {
                            mapping_for(root, w, &entry.reviewer_identity, p, obligation)
                        })
                        .collect::<Result<Vec<_>, String>>()?;
                    mappings.insert(entry.reviewer_identity.clone(), bound);
                }
                for mapping in &mappings[&entry.reviewer_identity] {
                    if mapping.sha256()? != entry.mapping_sha256 {
                        continue;
                    }
                    let endorsement = Endorsement {
                        kind: kind.clone(),
                        repository: depth_ci::REPOSITORY.into(),
                        pull_request: pr,
                        id: value["id"].as_u64().ok_or("missing endorsement id")?,
                        author: author.into(),
                        source_commit: body.source_commit.clone(),
                        body_sha256: depth_ci::reference("", raw.as_bytes()).sha256,
                        mapping_sha256: entry.mapping_sha256.clone(),
                    };
                    adopted.push((
                        mapping.clone(),
                        Adoption {
                            endorsement,
                            reviewer: entry.reviewer_identity.clone(),
                            executor: entry.executor_identity.clone(),
                        },
                    ));
                }
            }
        }
    }
    // Deterministic choice among equivalent valid adoptions; never select among execution runs.
    adopted.sort_by_key(|(_, a)| a.endorsement.id);
    let mut complete = vec![];
    for obligation in obligations {
        let Some(i) = adopted.iter().position(|(m, _)| m.obligation == obligation) else {
            return Ok(vec![]);
        };
        complete.push(adopted.remove(i));
    }
    Ok(complete)
}
pub(in crate::depth_evidence::verdict) fn body(raw: &str) -> Result<Option<Body>, String> {
    let marker = "```depth-adequacy-v1\n";
    if !raw.contains(marker) {
        return Ok(None);
    }
    if raw.matches(marker).count() != 1 {
        return Err("ambiguous adequacy adoption block".into());
    }
    let (_, rest) = raw.split_once(marker).unwrap();
    let (json, _) = rest
        .split_once("\n```")
        .ok_or("unterminated adequacy adoption")?;
    if json.len() > 65536 {
        return Err("oversized adequacy adoption".into());
    }
    serde_json::from_str(json)
        .map(Some)
        .map_err(|_| "invalid exact adequacy adoption".into())
}
pub(super) fn proposal(w: &Witness, reviewer: &str) -> Result<Value, String> {
    let m = mapping(w, reviewer)?;
    Ok(
        serde_json::json!({"schema_version":1,"source_commit":w.build.source_commit,"mappings":[{"mapping_sha256":m.sha256()?,"reviewer_identity":reviewer,"executor_identity":EXECUTOR,"independence_attested":true,"counterfactual_adopted":true}]}),
    )
}

pub(super) fn proposal_for(
    root: &Path,
    w: &Witness,
    reviewer: &str,
    p: Profile,
) -> Result<Value, String> {
    if p == Profile::Channel {
        return proposal(w, reviewer);
    }
    let original = original_for(&source(root, &w.build.source_commit)?, p)?;
    let mappings=p.obligations(&original).iter().map(|o|{
        let m=mapping_for(root,w,reviewer,p,o)?;
        Ok(serde_json::json!({"mapping_sha256":m.sha256()?,"reviewer_identity":reviewer,"executor_identity":p.executor(),"independence_attested":true,"counterfactual_adopted":true}))
    }).collect::<Result<Vec<_>,String>>()?;
    Ok(
        serde_json::json!({"schema_version":1,"source_commit":w.build.source_commit,"mappings":mappings}),
    )
}
