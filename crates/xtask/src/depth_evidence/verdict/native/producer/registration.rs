use super::*;
use std::{io::Write, process::Command};
pub(in crate::depth_evidence::verdict) trait StartApi:
    host::Api
{
    // Exactly one POST. An error includes an ambiguous response: callers never retry it.
    fn post_start(&self, repository: &form::Repository, pr: u64, body: &str)
        -> Result<u64, String>;
}
impl StartApi for host::Github {
    fn post_start(
        &self,
        repository: &form::Repository,
        pr: u64,
        body: &str,
    ) -> Result<u64, String> {
        let mut input =
            tempfile::NamedTempFile::new().map_err(|_| "cannot prepare start registration")?;
        let bytes = serde_json::to_vec(&serde_json::json!({"body":body}))
            .map_err(|_| "cannot encode registration")?;
        if bytes.len() > 65_536 {
            return Err("start registration exceeds bounds".into());
        }
        input
            .write_all(&bytes)
            .and_then(|()| input.flush())
            .map_err(|_| "cannot prepare registration input")?;
        let mut c = Command::new("/usr/bin/timeout");
        c.args([
            "--kill-after=5",
            "30",
            "/usr/bin/gh",
            "api",
            "--hostname",
            "github.com",
            "--method",
            "POST",
            "-H",
            "X-GitHub-Api-Version: 2026-03-10",
            "--input",
        ])
        .arg(input.path())
        .arg(format!("repos/{}/issues/{pr}/comments", repository.name));
        let raw=super::super::super::github::transport::capture(&mut c,1_048_576).map_err(|_| "registration failed or outcome is ambiguous; inspect the packet PR, do not retry blindly")?;
        let value: serde_json::Value = serde_json::from_slice(&raw)
            .map_err(|_| "ambiguous registration response; inspect packet PR")?;
        value["id"]
            .as_u64()
            .filter(|id| *id > 0)
            .ok_or_else(|| "registration id unavailable; inspect packet PR before any retry".into())
    }
}
pub(super) fn register(
    api: &impl StartApi,
    p: &Profile,
    pr: u64,
    start: &form::Start,
) -> Result<population::Comment, String> {
    let body = fence("depth-native-start-v1", start)?;
    let expected = crate::depth_ci::reference("", body.as_bytes()).sha256;
    // No native probe precedes this single POST and independent authoritative re-read.
    let id = api.post_start(&p.repository, pr, &body)?;
    let bound = BTreeSet::from([id]);
    let comments = host::comments(api, p, pr, &bound)?;
    let comment = comments
        .into_iter()
        .find(|c| c.id == id)
        .ok_or("posted registration is unavailable; native execution refused")?;
    let direct = api.json(&p.repository, &format!("issues/comments/{id}"))?;
    if !comment.authorized
        || comment.reference().body_sha256 != expected
        || direct["id"].as_u64() != Some(id)
        || direct["body"] != body
        || direct["user"]["id"].as_u64() != Some(comment.author_id)
        || direct["user"]["login"] != comment.login
    {
        return Err(
            "start registration body/author/authority differs; native execution refused".into(),
        );
    }
    Ok(comment)
}
