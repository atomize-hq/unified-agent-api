//! Only finite, nonsecret diagnostics are retained. No auth/config/environment file reads.
use sha2::{Digest, Sha256};
use std::{fs, io::Read, path::Path, process::Stdio, time::Duration};
use tokio::{io::AsyncReadExt, process::Command};

pub fn fingerprint(path: &Path) -> Result<String, String> {
    let mut f = fs::File::open(path).map_err(|_| "cannot read executable identity")?;
    if !f
        .metadata()
        .map_err(|_| "cannot inspect executable")?
        .is_file()
    {
        return Err("executable must be a regular file".into());
    }
    let mut hash = Sha256::new();
    let mut total = 0usize;
    let mut chunk = [0; 65536];
    loop {
        let n = f
            .read(&mut chunk)
            .map_err(|_| "cannot read executable identity")?;
        if n == 0 {
            break;
        }
        total += n;
        if total > 512 * 1024 * 1024 {
            return Err("executable exceeds identity limit".into());
        }
        hash.update(&chunk[..n]);
    }
    Ok(hex::encode(hash.finalize()))
}
pub async fn version(binary: &Path) -> Result<String, String> {
    version_with_timeout(binary, Duration::from_secs(5)).await
}
async fn version_with_timeout(binary: &Path, timeout: Duration) -> Result<String, String> {
    let mut command = Command::new(binary);
    command
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    let mut child = spawn_probe(|| command.spawn()).await?;
    let mut stdout = child
        .stdout
        .take()
        .ok_or("missing version output")?
        .take(4097);
    let mut bytes = Vec::new();
    // Waiting and draining share one deadline, including when a launcher exits
    // but a descendant retains stdout. Dropping the read future closes the pipe.
    let probe = async { tokio::try_join!(child.wait(), stdout.read_to_end(&mut bytes)) };
    let status = match tokio::time::timeout(timeout, probe).await {
        Ok(Ok((status, _))) => status,
        _ => {
            let _ = child.kill().await;
            let _ = child.wait().await;
            return Err("Codex version probe failed or timed out".into());
        }
    };
    if !status.success() || bytes.len() > 4096 {
        return Err("Codex version probe failed or exceeded bounds".into());
    }
    let raw = std::str::from_utf8(&bytes)
        .map_err(|_| "invalid version response")?
        .trim();
    let parsed = raw
        .strip_prefix("codex-cli ")
        .and_then(|s| semver::Version::parse(s).ok())
        .ok_or("unrecognized Codex version response (details withheld)")?;
    Ok(parsed.to_string())
}
// Match the wrapper's bounded handling of a transient pre-exec ETXTBSY. No process
// has started on this error; never retry an executed version probe or another spawn failure.
async fn spawn_probe<T>(mut spawn: impl FnMut() -> std::io::Result<T>) -> Result<T, String> {
    for attempt in 0..5 {
        match spawn() {
            Ok(child) => return Ok(child),
            Err(error) if cfg!(unix) && error.raw_os_error() == Some(26) && attempt < 4 => {
                tokio::time::sleep(Duration::from_millis(2 << attempt)).await;
            }
            Err(_) => return Err("cannot probe selected Codex executable".into()),
        }
    }
    unreachable!("bounded spawn returns on its final attempt")
}
pub fn output_path(raw: &str) -> Result<(), String> {
    if raw.len() > 4096
        || raw.contains(['\\', '\0'])
        || raw
            .split('/')
            .any(|p| p.is_empty() || matches!(p, "." | ".." | ".git"))
    {
        return Err("output directory must be a safe repository-relative path".into());
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn output_references_cannot_escape_or_name_git_metadata() {
        for path in [
            "",
            "/tmp/out",
            "a//b",
            "a/../b",
            "a/./b",
            ".git/proof",
            "a\\b",
            "a/",
            "a\0b",
        ] {
            assert!(output_path(path).is_err(), "{path:?}");
        }
        assert!(output_path("target/native-results/first").is_ok());
    }
    #[cfg(unix)]
    #[tokio::test]
    async fn probe_retries_only_pre_exec_busy_and_stops_at_its_bound() {
        let mut attempts = 0;
        let value = spawn_probe(|| {
            attempts += 1;
            if attempts < 3 {
                Err(std::io::Error::from_raw_os_error(26))
            } else {
                Ok(7)
            }
        })
        .await
        .unwrap();
        assert_eq!((value, attempts), (7, 3));
        for (errno, expected) in [(26, 5), (2, 1), (13, 1)] {
            let mut attempts = 0;
            let result: Result<(), String> = spawn_probe(|| {
                attempts += 1;
                Err(std::io::Error::from_raw_os_error(errno))
            })
            .await;
            assert_eq!(
                result.unwrap_err(),
                "cannot probe selected Codex executable"
            );
            assert_eq!(attempts, expected);
        }
    }
    #[cfg(unix)]
    #[tokio::test]
    async fn version_probe_withholds_foreign_stdout_and_stderr() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let binary = dir.path().join("native");
        fs::write(
            &binary,
            "#!/bin/sh\necho sk-sensitive-native-output\necho secret-error >&2\n",
        )
        .unwrap();
        fs::set_permissions(&binary, fs::Permissions::from_mode(0o700)).unwrap();
        let error = version(&binary).await.unwrap_err();
        assert!(!error.contains("sensitive"));
        assert!(!error.contains("secret-error"));
        fs::write(&binary, "#!/bin/sh\necho 'codex-cli 0.162.1'\n").unwrap();
        assert_eq!(version(&binary).await.unwrap(), "0.162.1");
    }
    #[cfg(unix)]
    #[tokio::test]
    async fn version_deadline_includes_stdout_held_by_a_launcher_descendant() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let binary = dir.path().join("launcher");
        // This finite synthetic descendant holds stdout after its launcher exits.
        // Wait for it below so the test leaves no test child running.
        fs::write(&binary, "#!/bin/sh\nsleep 1 &\necho 'codex-cli 0.162.1'\n").unwrap();
        fs::set_permissions(&binary, fs::Permissions::from_mode(0o700)).unwrap();
        let result = version_with_timeout(&binary, Duration::from_millis(50)).await;
        tokio::time::sleep(Duration::from_secs(1)).await;
        let error = result.unwrap_err();
        assert!(error.contains("timed out"), "unexpected refusal: {error}");
        // Without a deadline on draining, this finite descendant yields a valid
        // version instead of a timeout. Wall-clock scheduling latency is not the predicate.
    }
}
