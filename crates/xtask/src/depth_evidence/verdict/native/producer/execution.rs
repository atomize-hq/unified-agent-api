//! Invoke the closed-built existing collector once. Native authentication remains caller-owned.
use super::*;
use crate::depth_evidence::{CaseIdentity, CaseOutcome};
use serde_json::{json, Value};
use std::{
    io::Read,
    process::{Command, Stdio},
    time::Duration,
};
use wait_timeout::ChildExt;
pub(super) fn fingerprint(path: &Path) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    let mut f = fs::File::open(path).map_err(|_| "native executable unavailable")?;
    if !f
        .metadata()
        .is_ok_and(|m| m.is_file() && m.len() <= 512 * 1024 * 1024)
    {
        return Err("invalid native executable type/size".into());
    }
    let mut h = Sha256::new();
    let mut buf = [0; 65536];
    let mut total = 0;
    loop {
        let n = f
            .read(&mut buf)
            .map_err(|_| "cannot measure native executable")?;
        if n == 0 {
            break;
        }
        total += n;
        if total > 512 * 1024 * 1024 {
            return Err("native executable exceeds bound".into());
        }
        h.update(&buf[..n]);
    }
    Ok(hex::encode(h.finalize()))
}
pub(in crate::depth_evidence::verdict) fn requested(sha: &str) -> Vec<CaseIdentity> {
    ["unattended", "interactive"]
        .iter()
        .flat_map(|v| {
            [
                "initial",
                "resume-last",
                "resume-id",
                "fork-last",
                "fork-id",
                "cancel",
            ]
            .map(|step| CaseIdentity {
                name: format!("codex-native/{v}/{step}"),
                executable_sha256: sha.into(),
            })
        })
        .collect()
}
pub(in crate::depth_evidence::verdict) struct Execution {
    pub cases: Cases,
    pub context: form::context::Observation,
    pub observations: Value,
    pub complete: bool,
}
pub(super) fn incomplete(plan: &form::Plan) -> Execution {
    Execution {
        cases: Cases {
            requested: plan.requested.clone(),
            executed: vec![],
            raw_output: crate::depth_ci::reference("unavailable", b""),
        },
        context: form::context::Observation::incomplete(),
        observations: json!({"schema_version":1,"status":"unavailable_or_interrupted","unrun":plan.requested}),
        complete: false,
    }
}
fn bounded(root: &Path, path: &str) -> Result<Vec<u8>, String> {
    let mut full = root.to_path_buf();
    for part in path.split('/') {
        full.push(part);
        let m = fs::symlink_metadata(&full).map_err(|_| "native output unavailable")?;
        if m.is_symlink() {
            return Err("native output symlink refused".into());
        }
    }
    let f = fs::File::open(&full).map_err(|_| "native output unavailable")?;
    if !f
        .metadata()
        .is_ok_and(|m| m.is_file() && m.len() <= 1_048_576)
    {
        return Err("native output type/size refused".into());
    }
    let mut bytes = Vec::new();
    f.take(1_048_577)
        .read_to_end(&mut bytes)
        .map_err(|_| "native output unreadable")?;
    if bytes.len() > 1_048_576 {
        return Err("native output bound exceeded".into());
    }
    Ok(bytes)
}
struct Owned(std::process::Child);
impl Drop for Owned {
    fn drop(&mut self) {
        #[cfg(unix)]
        {
            let _ = Command::new("/usr/bin/kill")
                .env_clear()
                .args(["--signal", "KILL", "--"])
                .arg(format!("-{}", self.0.id()))
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
fn wait_owned(
    mut command: Command,
    cancel: &std::sync::atomic::AtomicBool,
    timeout: Duration,
) -> Result<Option<std::process::ExitStatus>, String> {
    super::control::check(cancel)?;
    let mut child = Owned(
        command
            .spawn()
            .map_err(|_| "cannot start registered native collector")?,
    );
    let deadline = std::time::Instant::now() + timeout;
    let status = loop {
        if super::control::check(cancel).is_err() {
            break None;
        }
        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
        if remaining.is_zero() {
            break None;
        }
        if let Some(status) = child
            .0
            .wait_timeout(remaining.min(Duration::from_millis(25)))
            .map_err(|_| "cannot wait for native collector")?
        {
            break Some(status);
        }
    };
    // Also kills descendants when the direct collector exited first. Always reap the direct child.
    drop(child);
    Ok(status)
}
pub(super) fn run(
    root: &Path,
    args: &CollectArgs,
    binary: &Path,
    built: &crate::depth_ci::native::Built,
    plan: &form::Plan,
    cancel: &std::sync::atomic::AtomicBool,
) -> Result<Execution, String> {
    super::control::check(cancel)?;
    if fingerprint(binary)? != plan.native.executable_sha256
        || fingerprint(&built.executable)? != built.executable_sha256
    {
        return Err("native/collector executable changed after registration".into());
    }
    let work = args
        .working_dir
        .canonicalize()
        .map_err(|_| "native working directory unavailable")?;
    if !work.is_dir() || work == root || work.starts_with(root) {
        return Err("native project must be separate from producer source".into());
    }
    let output = PathBuf::from(&plan.context.path)
        .parent()
        .ok_or("missing attempt root")?
        .join("execution")
        .to_string_lossy()
        .into_owned();
    let mut c = Command::new(&built.executable);
    c.args(["--live", "--context-guard", "--workspace"])
        .arg(root)
        .arg("--binary")
        .arg(binary)
        .arg("--working-dir")
        .arg(work)
        .arg("--expected-version")
        .arg(&plan.native.version)
        .arg("--output-dir")
        .arg(&output)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if let Some(home) = &args.codex_home {
        c.arg("--codex-home").arg(home);
    }
    // Inherit the caller's normal authentication/configuration; never reuse BuildEnv.command.
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        c.process_group(0);
    }
    if !cfg!(unix) {
        return Err("owned native process cleanup requires Unix".into());
    }
    let status = wait_owned(c, cancel, Duration::from_secs(420))?;
    // Completed finite journals survive timeout, a missing final file, or a failed final probe.
    let (steps, journal_valid) = read_steps(root, &output, &plan.requested);
    let cases = project(&plan.requested, &output, &steps)?;
    let final_matches = (|| -> Result<bool, String> {
        let raw = bounded(root, &format!("{output}/observations.json"))?;
        let cases_bytes = bounded(root, &format!("{output}/cases.json"))?;
        let final_cases: Cases =
            serde_json::from_slice(&cases_bytes).map_err(|_| "invalid native cases")?;
        crate::depth_evidence::validate::cases(&final_cases)?;
        let observations: Value =
            serde_json::from_slice(&raw).map_err(|_| "invalid native observations")?;
        Ok(final_cases.requested == plan.requested
            && final_cases.executed == cases.executed
            && final_cases.raw_output
                == crate::depth_ci::reference(format!("{output}/observations.json"), &raw)
            && observations["runner_sha256"] == built.executable_sha256
            && observations["selected_binary_sha256"] == plan.native.executable_sha256
            && observations["runtime_version"] == plan.native.version)
    })()
    .unwrap_or(false);
    // Copy only typed finite fields, never arbitrary child JSON, stdout, stderr or error text.
    let observations = json!({"schema_version":1,"collector_finished":status.is_some(),"final_identity_matches":final_matches,"journal_valid":journal_valid,"results":steps});
    let unchanged =
        fingerprint(binary).ok().as_deref() == Some(plan.native.executable_sha256.as_str());
    // The ordinary native context is not closed by an intent or a successful response.
    let mut context = form::context::Observation::incomplete();
    for (i, step) in steps.iter().enumerate() {
        let c = (|| -> Result<form::context::CaseContext, String> {
            let raw = bounded(root, &format!("{output}/context-{i}.json"))?;
            let c: form::context::CaseContext =
                serde_json::from_slice(&raw).map_err(|_| "invalid finite context journal")?;
            if c.case != step.case
                || c.launches.len() > 2
                || !c.bindings.is_empty()
                || !c.derived_defaults.is_empty()
            {
                return Err("native context journal differs".into());
            }
            Ok(c)
        })();
        if let Ok(c) = c {
            context.cases.push(c);
        }
    }
    // Layer/auth/turn/runtime/effect closure and native corroborating cases are still due.
    // Neither a child-provided complete flag nor successful smoke clears these facts.
    let complete = super::control::check(cancel).is_ok()
        && status.is_some_and(|s| s.success())
        && final_matches
        && context.complete
        && journal_valid
        && unchanged
        && cases.executed.len() == cases.requested.len()
        && cases
            .executed
            .iter()
            .all(|r| r.outcome == CaseOutcome::Passed);
    Ok(Execution {
        cases,
        context,
        observations,
        complete,
    })
}

#[cfg(test)]
pub(super) fn journal(
    root: &Path,
    output: &str,
    requested: &[CaseIdentity],
) -> Result<Cases, String> {
    let (steps, valid) = read_steps(root, output, requested);
    if !valid {
        return Err("native journal contains unavailable or malformed cases".into());
    }
    project(requested, output, &steps)
}
fn project(
    requested: &[CaseIdentity],
    output: &str,
    steps: &[form::NativeStep],
) -> Result<Cases, String> {
    Ok(Cases {
        requested: requested.into(),
        executed: steps
            .iter()
            .map(|s| crate::depth_evidence::CaseResult {
                case: s.case.clone(),
                outcome: s.outcome.clone(),
            })
            .collect(),
        raw_output: crate::depth_ci::reference(
            format!("{output}/journal.json"),
            &canonical(&steps)?,
        ),
    })
}
fn read_steps(
    root: &Path,
    output: &str,
    requested: &[CaseIdentity],
) -> (Vec<form::NativeStep>, bool) {
    let mut steps = Vec::new();
    let mut missing = false;
    let mut valid = true;
    for (i, case) in requested.iter().enumerate() {
        let path = format!("{output}/case-{i}.json");
        match fs::symlink_metadata(root.join(&path)) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                missing = true;
                continue;
            }
            Err(_) => {
                valid = false;
                continue;
            }
            Ok(_) => {}
        }
        if missing {
            valid = false;
        }
        let step = (|| -> Result<form::NativeStep, String> {
            let bytes = bounded(root, &path)?;
            let s: form::NativeStep =
                serde_json::from_slice(&bytes).map_err(|_| "invalid finite native journal")?;
            if s.case != *case
                || s.elapsed_ms > 420_000
                || s.observation
                    .session_id
                    .as_deref()
                    .is_some_and(|id| !form::native_session_id(id))
            {
                return Err("native journal identity/observation differs".into());
            }
            Ok(s)
        })();
        match step {
            Ok(s) => steps.push(s),
            Err(_) => {
                valid = false;
            }
        }
    }
    (steps, valid)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn truncated_case_does_not_erase_an_earlier_finite_failure() {
        let tmp = tempfile::tempdir().unwrap();
        let requested = requested(&"a".repeat(64));
        fs::create_dir(tmp.path().join("execution")).unwrap();
        let step = form::NativeStep {
            case: requested[0].clone(),
            observation: form::NativeObservation::default(),
            outcome: CaseOutcome::Failed,
            elapsed_ms: 1,
            failure: Some(form::NativeFailure::RequestRefused),
        };
        fs::write(
            tmp.path().join("execution/case-0.json"),
            canonical(&step).unwrap(),
        )
        .unwrap();
        fs::write(tmp.path().join("execution/case-1.json"), b"{truncated").unwrap();
        let (steps, valid) = read_steps(tmp.path(), "execution", &requested);
        assert!(!valid);
        assert_eq!(steps, vec![step]);
    }
}

#[cfg(all(test, target_os = "linux"))]
mod interruption_tests;
#[cfg(all(test, target_os = "linux"))]
mod process_tests;
