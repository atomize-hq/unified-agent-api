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
    let mut child = Command::new(binary)
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .map_err(|_| "cannot probe selected Codex executable")?;
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
        use std::{os::unix::fs::PermissionsExt, time::Instant};
        let dir = tempfile::tempdir().unwrap();
        let binary = dir.path().join("launcher");
        // This finite synthetic descendant holds stdout after its launcher exits.
        // Wait for it below so the test leaves no test child running.
        fs::write(&binary, "#!/bin/sh\nsleep 1 &\necho 'codex-cli 0.162.1'\n").unwrap();
        fs::set_permissions(&binary, fs::Permissions::from_mode(0o700)).unwrap();
        let started = Instant::now();
        let result = version_with_timeout(&binary, Duration::from_millis(50)).await;
        let elapsed = started.elapsed();
        tokio::time::sleep(Duration::from_secs(1)).await;
        assert!(
            result.is_err(),
            "a retained pipe must exhaust the probe deadline"
        );
        assert!(elapsed < Duration::from_millis(500), "{elapsed:?}");
    }
}
