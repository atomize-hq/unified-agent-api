use crate::{depth_ci::REPOSITORY, runtime_advertising::source::Source};
use serde_json::Value;
use std::{
    collections::BTreeSet,
    io::Read,
    path::Path,
    process::{Command, Stdio},
};

pub(super) trait Api {
    /// The one GitHub repository this reader is bound to: compiled source, never a receipt claim.
    fn repository(&self) -> &str {
        REPOSITORY
    }
    fn endpoint(&self, path: &str) -> String {
        format!("repos/{}/{path}", self.repository())
    }
    /// The repository object itself, for id and owner corroboration.
    fn identity_endpoint(&self) -> String {
        format!("repos/{}", self.repository())
    }
    fn bytes(&self, endpoint: &str, limit: usize) -> Result<Vec<u8>, String>;
    fn json(&self, endpoint: &str) -> Result<Value, String> {
        serde_json::from_slice(&self.bytes(endpoint, 16_777_216)?)
            .map_err(|_| "invalid GitHub response".into())
    }
}
pub(super) struct Github {
    repository: &'static str,
}
/// The canonical repository reader for the channel and CORE profiles.
pub(super) const CANONICAL: Github = Github {
    repository: REPOSITORY,
};
impl Github {
    /// The isolated repository, only as an authenticated IR-P1 binding names it.
    pub(super) fn isolated(host: &crate::isolated_maintenance::EvidenceHost) -> Self {
        Self {
            repository: host.repository(),
        }
    }
}
impl Api for Github {
    fn repository(&self) -> &str {
        self.repository
    }
    fn bytes(&self, endpoint: &str, limit: usize) -> Result<Vec<u8>, String> {
        // Fixed host and repository; never accept a URL/credential/host from a receipt.
        let own = format!("repos/{}", self.repository);
        if !(endpoint == own || endpoint.starts_with(&format!("{own}/")))
            || endpoint.contains(['\n', '\r'])
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
/// The reader for the isolated repository an authenticated IR-P1 binding names. Unit tests may
/// substitute fake host responses for the calling thread; production cannot.
pub(super) struct Isolated {
    github: Github,
    #[cfg(test)]
    fake: Option<test_reader::Reader>,
}
impl Isolated {
    pub(super) fn new(host: &crate::isolated_maintenance::EvidenceHost) -> Self {
        Self {
            github: Github::isolated(host),
            #[cfg(test)]
            fake: test_reader::installed(),
        }
    }
}
impl Api for Isolated {
    fn repository(&self) -> &str {
        self.github.repository
    }
    fn bytes(&self, endpoint: &str, limit: usize) -> Result<Vec<u8>, String> {
        // A unit test never reaches the network: without a fake host its reads are unavailable.
        #[cfg(test)]
        return match &self.fake {
            Some(fake) => fake(endpoint, limit),
            None => Err("no fake isolated host is installed for this test".into()),
        };
        #[cfg(not(test))]
        self.github.bytes(endpoint, limit)
    }
}
pub(in crate::depth_evidence::verdict) fn capture(
    command: &mut Command,
    limit: usize,
) -> Result<Vec<u8>, String> {
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

/// Exhaust the endpoint; refuse truncation, changing totals and duplicate identities.
pub(super) fn list(api: &impl Api, path: &str, member: Option<&str>) -> Result<Vec<Value>, String> {
    let mut rows = vec![];
    let mut seen = BTreeSet::new();
    let mut total = None;
    for page in 1..=10 {
        let join = if path.contains('?') { '&' } else { '?' };
        let value = api.json(&api.endpoint(&format!("{path}{join}per_page=100&page={page}")))?;
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
pub(super) fn source(root: &Path, repository: &str, commit: &str) -> Result<Source, String> {
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
                &format!("https://github.com/{repository}.git"),
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
#[cfg(test)]
pub(crate) mod test_reader {
    use std::{cell::RefCell, sync::Arc};
    pub(crate) type Reader = Arc<dyn Fn(&str, usize) -> Result<Vec<u8>, String>>;
    thread_local!(static READER: RefCell<Option<Reader>> = const { RefCell::new(None) });
    /// Uninstalls on drop, so a failing test never leaks its fake host.
    pub(crate) struct Installed;
    impl Drop for Installed {
        fn drop(&mut self) {
            READER.with(|r| *r.borrow_mut() = None);
        }
    }
    pub(crate) fn install(reader: Reader) -> Installed {
        READER.with(|r| *r.borrow_mut() = Some(reader));
        Installed
    }
    pub(super) fn installed() -> Option<Reader> {
        READER.with(|r| r.borrow().clone())
    }
}
