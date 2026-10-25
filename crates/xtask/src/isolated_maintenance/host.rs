use super::{Api, Profile, Repository};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    io::Read,
    process::{Command, Stdio},
};

pub(super) struct Github;
impl Api for Github {
    fn json(&self, endpoint: &str) -> Result<Value, String> {
        // An installation must source-bind these exact repositories; credentials or GH_REPO
        // cannot select another host. The live profile is intentionally absent in this slice.
        let profile = super::LIVE_PROFILE.ok_or("isolated maintenance profile is not installed")?;
        if endpoint.contains(['\n', '\r', '\0'])
            || endpoint.len() > 2048
            || ![&profile.isolated, &profile.canonical].iter().any(|repo| {
                endpoint == format!("repos/{}", repo.name)
                    || endpoint.starts_with(&format!("repos/{}/", repo.name))
            })
        {
            return Err("unsupported isolated host endpoint".into());
        }
        let mut child = Command::new("/usr/bin/timeout")
            .args([
                "--kill-after=5",
                "30",
                "/usr/bin/gh",
                "api",
                "--hostname",
                "github.com",
                "--method",
                "GET",
                "-H",
                "Accept: application/vnd.github+json",
                endpoint,
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| "isolated authenticated host read unavailable")?;
        let mut raw = Vec::new();
        let read = child
            .stdout
            .take()
            .ok_or("missing isolated host stdout")?
            .take(16_777_217)
            .read_to_end(&mut raw);
        if read.is_err() || raw.len() > 16_777_216 {
            let _ = child.kill();
            let _ = child.wait();
            return Err("isolated host response unreadable or exceeds bound".into());
        }
        if !child
            .wait()
            .map_err(|_| "isolated host read wait failed")?
            .success()
        {
            return Err("isolated authenticated host read unavailable".into());
        }
        serde_json::from_slice(&raw).map_err(|_| "invalid isolated host response".into())
    }
}
#[derive(PartialEq, Eq)]
pub(super) struct Observation {
    pub(super) isolated_tip: String,
    canonical_tip: String,
    pub(super) source: BTreeMap<String, (String, String)>,
    pub(super) registry: String,
}
fn repository(api: &dyn Api, repository: &Repository) -> Result<String, String> {
    let prefix = format!("repos/{}", repository.name);
    let identity = api.json(&prefix)?;
    if identity["id"].as_u64() != Some(repository.id)
        || identity["owner"]["id"].as_u64() != Some(repository.owner)
        || identity["full_name"].as_str() != Some(repository.name)
    {
        return Err("isolated repository id, owner id or full name differs".into());
    }
    let branch = api.json(&format!("{prefix}/branches/staging"))?;
    if branch["name"] != "staging" || branch["protected"] != true {
        return Err("isolated binding requires protected staging".into());
    }
    let tip = branch["commit"]["sha"]
        .as_str()
        .ok_or("missing protected staging commit")?;
    oid(tip)?;
    Ok(tip.into())
}
pub(super) fn observe(api: &dyn Api, profile: &Profile, head: &str) -> Result<Observation, String> {
    let isolated_tip = repository(api, &profile.isolated)?;
    let canonical_tip = repository(api, &profile.canonical)?;
    let base = format!("repos/{}", profile.isolated.name);
    for (ancestor, target) in [
        (profile.baseline.as_str(), isolated_tip.as_str()),
        (profile.baseline.as_str(), head),
        (isolated_tip.as_str(), head),
    ] {
        let relation = api.json(&format!("{base}/compare/{ancestor}...{target}"))?;
        if !matches!(relation["status"].as_str(), Some("ahead" | "identical"))
            || relation["merge_base_commit"]["sha"].as_str() != Some(ancestor)
        {
            return Err("isolated original installation baseline ancestry differs".into());
        }
    }
    let commit = api.json(&format!("{base}/commits/{head}"))?;
    if commit["sha"].as_str() != Some(head) {
        return Err("candidate is not present in fixed isolated repository".into());
    }
    let registry = api.json(&format!(
        "{base}/contents/{}?ref={isolated_tip}",
        crate::agent_registry::REGISTRY_RELATIVE_PATH
    ))?;
    if registry["path"].as_str() != Some(crate::agent_registry::REGISTRY_RELATIVE_PATH)
        || registry["type"] != "file"
    {
        return Err("isolated protected governing registry is not a regular file".into());
    }
    let registry = registry["sha"]
        .as_str()
        .ok_or("missing protected governing registry identity")?
        .to_owned();
    oid(&registry)?;
    let tree = api.json(&format!(
        "repos/{}/git/trees/{canonical_tip}?recursive=1",
        profile.canonical.name
    ))?;
    if tree["truncated"] != false {
        return Err("canonical producer tree unavailable or truncated".into());
    }
    let rows = tree["tree"]
        .as_array()
        .ok_or("missing canonical producer tree")?;
    if rows.len() > 50_000 {
        return Err("canonical producer inventory exceeds bound".into());
    }
    let mut source = BTreeMap::new();
    for row in rows {
        let path = row["path"].as_str().ok_or("missing canonical tree path")?;
        if !profile
            .scopes
            .iter()
            .any(|prefix| path == *prefix || path.starts_with(&format!("{prefix}/")))
        {
            continue;
        }
        if row["type"] == "tree" {
            continue;
        }
        let sha = row["sha"]
            .as_str()
            .ok_or("missing canonical blob identity")?;
        oid(sha)?;
        let mode = row["mode"].as_str().ok_or("missing canonical blob mode")?;
        if row["type"] != "blob"
            || !matches!(mode, "100644" | "100755")
            || source
                .insert(path.into(), (sha.into(), mode.into()))
                .is_some()
        {
            return Err("unsupported or duplicate canonical producer file".into());
        }
    }
    let expected: std::collections::BTreeSet<_> =
        profile.closure.iter().map(|(p, _, _)| *p).collect();
    if source
        .keys()
        .map(String::as_str)
        .collect::<std::collections::BTreeSet<_>>()
        != expected
    {
        return Err("canonical producer source inventory differs".into());
    }
    // Closing identity/tip reads reject instability within this observation.
    if repository(api, &profile.isolated)? != isolated_tip
        || repository(api, &profile.canonical)? != canonical_tip
    {
        return Err("isolated or canonical protected source moved during observation".into());
    }
    Ok(Observation {
        isolated_tip,
        canonical_tip,
        source,
        registry,
    })
}
pub(super) fn oid(value: &str) -> Result<(), String> {
    if value.len() != 40
        || !value
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        return Err("isolated binding requires a full lowercase Git object id".into());
    }
    Ok(())
}
