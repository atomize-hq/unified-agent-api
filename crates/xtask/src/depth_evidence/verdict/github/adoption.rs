use super::{profile::*, transport::*, *};
use crate::depth_ci::{self, Witness};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Body {
    schema_version: u64,
    source_commit: String,
    mappings: Vec<Entry>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    mapping_sha256: String,
    reviewer_identity: String,
    executor_identity: String,
    independence_attested: bool,
    counterfactual_adopted: bool,
}

pub(super) fn resolve(
    api: &impl Api,
    root: &Path,
    pr: u64,
    w: &Witness,
) -> Result<Option<(AdequacyMapping, Adoption)>, String> {
    let comments = list(api, &format!("issues/{pr}/comments"), None)?;
    let reviews = list(api, &format!("pulls/{pr}/reviews"), None)?;
    let mut adopted = vec![];
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
            if original(&reviewed)? != original(&built)? {
                continue;
            }
            let same_inputs=[depth_ci::ASSERTION,depth_ci::PRODUCTION,depth_ci::CONTRACT].iter().all(|path| matches!((reviewed.optional_bytes(path),built.optional_bytes(path),reviewed.executable(path),built.executable(path)),(Ok(Some(a)),Ok(Some(b)),Ok(am),Ok(bm)) if a==b && am==bm));
            if !same_inputs {
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
                    || entry.executor_identity != EXECUTOR
                    || entry.reviewer_identity == EXECUTOR
                    || entry.reviewer_identity.trim().is_empty()
                {
                    continue;
                }
                let mapping = mapping(w, &entry.reviewer_identity)?;
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
                    mapping_sha256: entry.mapping_sha256,
                };
                adopted.push((
                    mapping,
                    Adoption {
                        endorsement,
                        reviewer: entry.reviewer_identity,
                        executor: entry.executor_identity,
                    },
                ));
            }
        }
    }
    // Deterministic choice among equivalent valid adoptions; never select among execution runs.
    adopted.sort_by_key(|(_, a)| a.endorsement.id);
    Ok(adopted.into_iter().next())
}
fn body(raw: &str) -> Result<Option<Body>, String> {
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
