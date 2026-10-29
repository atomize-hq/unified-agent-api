use super::*;
#[derive(Clone, PartialEq, Eq)]
pub(super) struct Comment {
    pub id: u64,
    pub body: String,
    pub author_id: u64,
    pub login: String,
    pub authorized: bool,
}
impl Comment {
    pub fn reference(&self) -> form::CommentRef {
        form::CommentRef {
            id: self.id,
            body_sha256: crate::depth_ci::reference("", self.body.as_bytes()).sha256,
        }
    }
}
pub(super) struct Attempt {
    pub start: form::Start,
    pub registration: form::CommentRef,
    pub finish: form::Finish,
    pub completion: form::CommentRef,
}
pub(super) fn block<'a>(raw: &'a str, tag: &str) -> Result<Option<&'a str>, String> {
    let marker = format!("```{tag}\n");
    if !raw.contains(&marker) {
        return Ok(None);
    }
    if raw.len() > 1_048_576 || raw.matches(&marker).count() != 1 {
        return Err("ambiguous or oversized native host block".into());
    }
    let rest = raw.split_once(&marker).unwrap().1;
    let json = rest
        .split_once("\n```")
        .ok_or("unterminated native host block")?
        .0;
    if json.len() > 65_536 {
        return Err("native host JSON exceeds bound".into());
    }
    Ok(Some(json))
}
pub(super) fn same_generation(a: &Scope, b: &Scope) -> bool {
    a.agent_id == b.agent_id
        && a.version == b.version
        && a.event == b.event
        && a.enrollment_origin == b.enrollment_origin
        && a.policy_identity == b.policy_identity
        && a.obligations_identity == b.obligations_identity
        && a.rules_revision == b.rules_revision
        && a.trust_revision == b.trust_revision
        && a.review_revision == b.review_revision
        && a.obligation.target == b.obligation.target
}
pub(super) fn discover(
    comments: &[Comment],
    scope: &Scope,
    repo: &form::Repository,
    pr: u64,
    bound: &BTreeSet<u64>,
) -> Result<Vec<Attempt>, String> {
    use std::collections::BTreeMap;
    let mut ids = BTreeSet::new();
    let mut eligible = BTreeSet::new();
    let mut starts = BTreeMap::new();
    let mut all_starts = BTreeMap::new();
    let mut finishes = Vec::new();
    for c in comments {
        if c.id == 0 || !ids.insert(c.id) {
            return Err("duplicate or invalid host comment id".into());
        }
        if !c.authorized {
            if bound.contains(&c.id) {
                return Err("bound native comment lacks authority".into());
            }
            continue;
        }
        eligible.insert(c.id);
        let start = block(&c.body, "depth-native-start-v1")?;
        let finish = block(&c.body, "depth-native-finish-v1")?;
        if start.is_some() && finish.is_some() {
            return Err("start and finish share a host record".into());
        }
        if let Some(raw) = start {
            let s = form::Start::parse(raw)?;
            all_starts.insert(c.id, c.reference());
            if !same_generation(&s.scope, scope) {
                continue;
            }
            if s.profile != "isolated-codex-native-v1"
                || s.repository != *repo
                || s.pull_request != pr
            {
                return Err("native registration host/profile differs".into());
            }
            starts.insert(c.id, (c, s));
        } else if let Some(raw) = finish {
            finishes.push((c, form::Finish::parse(raw)?));
        } else if c.body.contains("```depth-native-") || bound.contains(&c.id) {
            return Err("unknown or changed relevant native host record".into());
        }
    }
    if !bound.is_subset(&eligible) {
        return Err("missing or revoked bound native record".into());
    }
    // A surviving finish still binds a registration even when that start was deleted.
    // Missing history cannot establish that an attempt belongs to an irrelevant generation.
    for (_, finish) in &finishes {
        if all_starts.get(&finish.registration.id) != Some(&finish.registration) {
            return Err(
                "native completion has a missing or changed authoritative registration".into(),
            );
        }
    }
    let mut attempts = Vec::new();
    let mut used = BTreeSet::new();
    for (id, (registration, start)) in &starts {
        let matching: Vec<_> = finishes
            .iter()
            .filter(|(_, f)| f.registration.id == *id)
            .collect();
        if matching.len() != 1 {
            return Err("missing or duplicate native completion".into());
        }
        let (completion, finish) = matching[0];
        if finish.registration != registration.reference()
            || finish.population_through_id >= completion.id
            || finish.population_through_id < *id
            || !ids.contains(&finish.population_through_id)
        {
            return Err("native registration digest/order/snapshot differs".into());
        }
        let expected: BTreeSet<_> = starts
            .keys()
            .filter(|n| **n <= finish.population_through_id)
            .copied()
            .collect();
        if finish
            .registrations
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
            != expected
        {
            return Err("native completion omits or invents an applicable registration".into());
        }
        used.extend([*id, completion.id]);
        attempts.push(Attempt {
            start: start.clone(),
            registration: registration.reference(),
            finish: finish.clone(),
            completion: completion.reference(),
        });
    }
    if !bound.is_subset(&used) {
        return Err("bound native record is outside the applicable population".into());
    }
    Ok(attempts)
}
#[cfg(test)]
mod tests;
