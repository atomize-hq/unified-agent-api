use super::*;
use serde_json::Value;
use std::collections::BTreeMap;

// Only private compiled Profile data can supply repositories or approved input expectations.
pub(in crate::depth_evidence::verdict) trait Api {
    fn json(&self, repository: &form::Repository, path: &str) -> Result<Value, String>;
}
pub(super) struct Github;
impl Github {
    fn read(repository: &form::Repository, path: &str) -> Result<Vec<u8>, String> {
        crate::depth_evidence::validate::repository(&repository.name)?;
        if path.contains(['\r', '\n', '\0']) || path.len() > 2048 || path.starts_with('/') {
            return Err("invalid native host resource".into());
        }
        let mut command = std::process::Command::new("/usr/bin/timeout");
        command
            .args([
                "--kill-after=5",
                "30",
                "gh",
                "api",
                "--hostname",
                "github.com",
                "--method",
                "GET",
                "-H",
                "X-GitHub-Api-Version: 2026-03-10",
                "-H",
            ])
            .arg("Accept: application/vnd.github+json")
            .arg(format!("repos/{}/{path}", repository.name));
        super::super::github::transport::capture(&mut command, 16_777_216)
    }
}
impl Api for Github {
    fn json(&self, repo: &form::Repository, path: &str) -> Result<Value, String> {
        serde_json::from_slice(&Self::read(repo, path)?)
            .map_err(|_| "invalid native host JSON".into())
    }
}
pub(super) fn list(
    api: &impl Api,
    repo: &form::Repository,
    path: &str,
) -> Result<Vec<Value>, String> {
    let mut rows = Vec::new();
    let mut ids = BTreeSet::new();
    for page in 1..=10 {
        let join = if path.contains('?') { '&' } else { '?' };
        let value = api.json(repo, &format!("{path}{join}per_page=100&page={page}"))?;
        let batch = value.as_array().ok_or("missing native host page")?;
        if batch.len() > 100 {
            return Err("native host page bound exceeded".into());
        }
        for value in batch {
            let id = value["id"]
                .as_u64()
                .filter(|n| *n > 0)
                .ok_or("missing host identity")?;
            if !ids.insert(id) {
                return Err("duplicate native host identity".into());
            }
            rows.push(value.clone());
        }
        if batch.len() < 100 {
            return Ok(rows);
        }
    }
    Err("native host pagination bound reached".into())
}
pub(super) fn comments(
    api: &impl Api,
    p: &Profile,
    pr: u64,
    bound: &BTreeSet<u64>,
) -> Result<Vec<population::Comment>, String> {
    let rows = list(api, &p.repository, &format!("issues/{pr}/comments"))?;
    let mut out = Vec::new();
    let mut authorities = BTreeMap::new();
    for row in rows {
        let id = row["id"].as_u64().ok_or("missing comment id")?;
        let body = row["body"].as_str().ok_or("missing comment body")?;
        let relevant = body.contains("```depth-native-")
            || body.contains("```depth-adequacy-v1")
            || bound.contains(&id);
        // Ordinary chatter is not asked to authenticate a producer, but its id still bounds snapshots.
        let mut c = population::Comment {
            id,
            body: body.into(),
            author_id: 0,
            login: String::new(),
            authorized: false,
        };
        if relevant {
            c.author_id = row["user"]["id"]
                .as_u64()
                .filter(|id| *id > 0)
                .ok_or("missing host author identity")?;
            c.login = row["user"]["login"]
                .as_str()
                .ok_or("missing host author")?
                .into();
            if c.login.is_empty()
                || !c
                    .login
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-')
            {
                return Err("invalid host author".into());
            }
            if let Some((id, authorized)) = authorities.get(&c.login) {
                if *id != c.author_id {
                    return Err("native host login has inconsistent author ids".into());
                }
                c.authorized = *authorized;
                out.push(c);
                continue;
            }
            let permission = api.json(
                &p.canonical,
                &format!("collaborators/{}/permission", c.login),
            )?;
            if permission["user"]["id"].as_u64() != Some(c.author_id)
                || permission["user"]["login"].as_str() != Some(c.login.as_str())
            {
                return Err("permission lookup author differs or is unavailable".into());
            }
            c.authorized = match permission["permission"].as_str() {
                Some("admin" | "maintain" | "write") => true,
                Some("read" | "none" | "triage") => false,
                _ => return Err("indeterminate native author permission".into()),
            };
            authorities.insert(c.login.clone(), (c.author_id, c.authorized));
        }
        out.push(c);
    }
    Ok(out)
}
#[derive(Clone, PartialEq, Eq)]
pub(super) struct Tree {
    pub commit: String,
    pub oid: String,
    pub entries: BTreeMap<String, (String, String)>,
}
impl Tree {
    pub fn read(api: &impl Api, repo: &form::Repository, commit: &str) -> Result<Self, String> {
        crate::depth_evidence::validate::git_identity(commit)?;
        let value = api.json(repo, &format!("git/commits/{commit}"))?;
        if value["sha"] != commit {
            return Err("immutable host commit differs".into());
        }
        let oid = value["tree"]["sha"]
            .as_str()
            .ok_or("missing host tree")?
            .to_owned();
        crate::depth_evidence::validate::git_identity(&oid)?;
        let value = api.json(repo, &format!("git/trees/{oid}?recursive=1"))?;
        if value["sha"] != oid || value["truncated"] != false {
            return Err("unavailable/truncated native proof tree".into());
        }
        let rows = value["tree"]
            .as_array()
            .filter(|a| a.len() <= 50_000)
            .ok_or("native tree bound exceeded")?;
        let mut entries = BTreeMap::new();
        for row in rows {
            let path = row["path"].as_str().ok_or("missing proof path")?;
            crate::depth_evidence::validate::path(path)?;
            let mode = row["mode"].as_str().ok_or("missing proof mode")?;
            let sha = row["sha"].as_str().ok_or("missing proof blob")?;
            crate::depth_evidence::validate::git_identity(sha)?;
            if entries
                .insert(path.into(), (mode.into(), sha.into()))
                .is_some()
            {
                return Err("duplicate proof tree entry".into());
            }
        }
        Ok(Self {
            commit: commit.into(),
            oid,
            entries,
        })
    }
    pub fn source(
        &self,
        root: &Path,
    ) -> Result<crate::runtime_advertising::source::Source, String> {
        let source = crate::runtime_advertising::source::Source::tip(root, &self.commit)?;
        if source.tree()? != self.oid {
            return Err("cached Git object differs from authenticated host tree".into());
        }
        Ok(source)
    }
    pub fn bytes(&self, root: &Path, path: &str) -> Result<Vec<u8>, String> {
        crate::depth_evidence::validate::path(path)?;
        let (mode, _) = self
            .entries
            .get(path)
            .ok_or("missing immutable proof blob")?;
        if !matches!(mode.as_str(), "100644" | "100755") {
            return Err("non-regular proof blob".into());
        }
        let source = self.source(root)?;
        let bytes = source
            .optional_bytes(path)?
            .ok_or("missing cached immutable proof blob")?;
        if source.executable(path)? != (mode == "100755") || bytes.len() > 1_048_576 {
            return Err("native proof mode or size differs".into());
        }
        Ok(bytes)
    }
    pub fn file(&self, root: &Path, file: &FileRef) -> Result<Vec<u8>, String> {
        let bytes = self.bytes(root, &file.path)?;
        if crate::depth_ci::reference(&file.path, &bytes) != *file {
            return Err("immutable native payload digest differs".into());
        }
        Ok(bytes)
    }
    pub fn inventory(&self, scope: &str) -> BTreeSet<String> {
        let prefix = format!("{scope}/");
        self.entries
            .iter()
            .filter(|(path, (mode, _))| path.starts_with(&prefix) && mode != "040000")
            .map(|(path, _)| path.clone())
            .collect()
    }
}
pub(super) fn ancestor(
    api: &impl Api,
    repo: &form::Repository,
    base: &str,
    head: &str,
) -> Result<(), String> {
    crate::depth_evidence::validate::git_identity(base)?;
    crate::depth_evidence::validate::git_identity(head)?;
    let v = api.json(repo, &format!("compare/{base}...{head}"))?;
    if !matches!(v["status"].as_str(), Some("ahead" | "identical"))
        || v["merge_base_commit"]["sha"] != base
    {
        return Err("native proof ancestry differs".into());
    }
    Ok(())
}
fn tip(api: &impl Api, repo: &form::Repository) -> Result<String, String> {
    let identity = api.json(repo, "")?;
    if identity["id"].as_u64() != Some(repo.id)
        || identity["owner"]["id"].as_u64() != Some(repo.owner_id)
        || identity["full_name"] != repo.name
    {
        return Err("native repository identity differs".into());
    }
    let branch = api.json(repo, "branches/staging")?;
    if branch["name"] != "staging" || branch["protected"] != true {
        return Err("native staging is not protected".into());
    }
    let sha = branch["commit"]["sha"]
        .as_str()
        .ok_or("missing native staging tip")?
        .to_owned();
    crate::depth_evidence::validate::git_identity(&sha)?;
    Ok(sha)
}
#[derive(PartialEq, Eq)]
pub(super) struct Binding {
    pub canonical: Tree,
    pub isolated_tip: String,
    pub pr: u64,
    pub packet_head: String,
}
pub(super) fn bind(api: &impl Api, p: &Profile, current: &Evaluation) -> Result<Binding, String> {
    let canonical = Tree::read(api, &p.canonical, &tip(api, &p.canonical)?)?;
    let isolated_tip = tip(api, &p.repository)?;
    ancestor(api, &p.repository, &p.baseline, &isolated_tip)?;
    let request = match &current.current.event {
        Event::Maintenance { request_commit, .. } => request_commit,
        _ => return Err("native producer supports maintenance only".into()),
    };
    let pulls = list(api, &p.repository, &format!("commits/{request}/pulls"))?;
    let matching: Vec<_> = pulls
        .iter()
        .filter(|v| {
            v["base"]["ref"] == "staging"
                && v["base"]["repo"]["id"].as_u64() == Some(p.repository.id)
                && v["base"]["repo"]["full_name"] == p.repository.name
        })
        .collect();
    if matching.len() != 1 {
        return Err("native generation has no unique authenticated packet PR".into());
    }
    let pr = matching[0]["number"]
        .as_u64()
        .filter(|n| *n > 0)
        .ok_or("missing packet PR")?;
    let pull = api.json(&p.repository, &format!("pulls/{pr}"))?;
    if pull["number"].as_u64() != Some(pr)
        || pull["base"]["ref"] != "staging"
        || pull["base"]["repo"]["id"].as_u64() != Some(p.repository.id)
        || pull["head"]["repo"]["id"].as_u64() != Some(p.repository.id)
    {
        return Err("native packet host differs".into());
    }
    let packet_head = pull["head"]["sha"]
        .as_str()
        .ok_or("missing packet head")?
        .to_owned();
    ancestor(api, &p.repository, &p.baseline, &packet_head)?;
    ancestor(api, &p.repository, request, &packet_head)?;
    Ok(Binding {
        canonical,
        isolated_tip,
        pr,
        packet_head,
    })
}
#[cfg(test)]
mod tests;
