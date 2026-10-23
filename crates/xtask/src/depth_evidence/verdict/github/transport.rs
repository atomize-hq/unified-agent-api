use crate::{depth_ci::REPOSITORY, runtime_advertising::source::Source};
use serde_json::Value;
use std::{
    collections::BTreeSet,
    io::Read,
    path::Path,
    process::{Command, Stdio},
};

pub(super) trait Api {
    fn bytes(&self, endpoint: &str, limit: usize) -> Result<Vec<u8>, String>;
    fn json(&self, endpoint: &str) -> Result<Value, String> {
        serde_json::from_slice(&self.bytes(endpoint, 16_777_216)?)
            .map_err(|_| "invalid GitHub response".into())
    }
}
pub(super) struct Github;
impl Api for Github {
    fn bytes(&self, endpoint: &str, limit: usize) -> Result<Vec<u8>, String> {
        // Fixed host and repository; never accept a URL/credential/host from a receipt.
        if !endpoint.starts_with(&format!("repos/{REPOSITORY}/")) || endpoint.contains(['\n', '\r'])
        {
            return Err("unsupported GitHub endpoint".into());
        }
        let mut command = Command::new("/usr/bin/timeout");
        command.args([
            "--kill-after=5",
            "60",
            "gh",
            "api",
            "--hostname",
            "github.com",
            "--method",
            "GET",
            "-H",
            "X-GitHub-Api-Version: 2026-03-10",
            endpoint,
        ]);
        capture(&mut command, limit)
    }
}
fn capture(command: &mut Command, limit: usize) -> Result<Vec<u8>, String> {
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| "cannot start host read")?;
    let mut raw = Vec::new();
    child
        .stdout
        .take()
        .ok_or("missing host stdout")?
        .take(limit as u64 + 1)
        .read_to_end(&mut raw)
        .map_err(|_| "cannot read host response")?;
    if raw.len() > limit {
        let _ = child.kill();
        let _ = child.wait();
        return Err("host response bounds exceeded".into());
    }
    if !child.wait().map_err(|_| "host read wait failed")?.success() {
        return Err(
            "host read unavailable (credential, permission, expiry or network); no proof granted"
                .into(),
        );
    }
    Ok(raw)
}
pub(super) fn endpoint(path: &str) -> String {
    format!("repos/{REPOSITORY}/{path}")
}

/// Exhaust the endpoint; refuse truncation, changing totals and duplicate identities.
pub(super) fn list(api: &impl Api, path: &str, member: Option<&str>) -> Result<Vec<Value>, String> {
    let mut rows = vec![];
    let mut seen = BTreeSet::new();
    let mut total = None;
    for page in 1..=10 {
        let join = if path.contains('?') { '&' } else { '?' };
        let value = api.json(&endpoint(&format!("{path}{join}per_page=100&page={page}")))?;
        let batch = if let Some(key) = member {
            let n = value["total_count"]
                .as_u64()
                .ok_or("missing population total")?;
            if total.replace(n).is_some_and(|old| old != n) || n >= 1000 {
                return Err("unstable or saturated host population".into());
            }
            value[key].as_array().ok_or("missing population rows")?
        } else {
            value.as_array().ok_or("missing host list")?
        };
        if batch.len() > 100 {
            return Err("host pagination exceeds page bound".into());
        }
        for row in batch {
            let id = row["id"]
                .as_u64()
                .filter(|n| *n > 0)
                .ok_or("missing positive host id")?;
            if !seen.insert(id) {
                return Err("duplicate host population member".into());
            }
            rows.push(row.clone());
        }
        if batch.len() < 100 {
            if total.is_some_and(|n| n != rows.len() as u64) {
                return Err("incomplete host population".into());
            }
            return Ok(rows);
        }
    }
    Err("host pagination bound reached".into())
}
pub(super) fn sha(raw: &str) -> Result<&str, String> {
    if raw.len() == 40
        && raw
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        Ok(raw)
    } else {
        Err("invalid host commit identity".into())
    }
}
pub(super) fn source(root: &Path, commit: &str) -> Result<Source, String> {
    sha(commit)?;
    if let Ok(source) = Source::tip(root, commit) {
        return Ok(source);
    }
    // Only object acquisition, no refs/working files/FETCH_HEAD. Never use local remote URLs.
    capture(
        Command::new("/usr/bin/timeout")
            .args(["60", "git", "--no-replace-objects", "-C"])
            .arg(root)
            .args([
                "fetch",
                "--no-tags",
                "--no-write-fetch-head",
                "--no-recurse-submodules",
                &format!("https://github.com/{REPOSITORY}.git"),
                commit,
            ]),
        1_048_576,
    )?;
    Source::tip(root, commit)
}
pub(super) fn required_string<'a>(v: &'a Value, key: &str) -> Result<&'a str, String> {
    v[key]
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| format!("missing host {key}"))
}
