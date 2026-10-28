//! Only finite, nonsecret diagnostics are retained. No auth/config/environment file reads.
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Read,
    path::Path,
    process::{Command, Stdio},
    time::Duration,
};
use wait_timeout::ChildExt;

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
pub fn version(binary: &Path) -> Result<String, String> {
    let mut child = Command::new(binary)
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| "cannot probe selected Codex executable")?;
    let result = child
        .wait_timeout(Duration::from_secs(5))
        .map_err(|_| "cannot wait for version probe");
    let status = match result {
        Ok(Some(s)) => s,
        _ => {
            let _ = child.kill();
            let _ = child.wait();
            return Err("Codex version probe failed or timed out".into());
        }
    };
    let mut bytes = Vec::new();
    child
        .stdout
        .take()
        .ok_or("missing version output")?
        .take(4097)
        .read_to_end(&mut bytes)
        .map_err(|_| "cannot read version response")?;
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
    #[test]
    fn version_probe_withholds_foreign_stdout_and_stderr() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let binary = dir.path().join("native");
        fs::write(
            &binary,
            "#!/bin/sh\necho sk-sensitive-native-output\necho secret-error >&2\n",
        )
        .unwrap();
        fs::set_permissions(&binary, fs::Permissions::from_mode(0o700)).unwrap();
        let error = version(&binary).unwrap_err();
        assert!(!error.contains("sensitive"));
        assert!(!error.contains("secret-error"));
        fs::write(&binary, "#!/bin/sh\necho 'codex-cli 0.162.1'\n").unwrap();
        assert_eq!(version(&binary).unwrap(), "0.162.1");
    }
}
