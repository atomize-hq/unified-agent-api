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
    pub context: Value,
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
        context: json!({"schema_version":1,"complete":false,"effective_native_configuration":"unresolved","collector":"unavailable_or_interrupted"}),
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
pub(super) fn run(
    root: &Path,
    args: &CollectArgs,
    binary: &Path,
    built: &crate::depth_ci::native::Built,
    plan: &form::Plan,
) -> Result<Execution, String> {
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
    c.args(["--live", "--workspace"])
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
    #[cfg(not(unix))]
    {
        return Err("owned native process cleanup requires Unix".into());
    }
    let mut child = Owned(
        c.spawn()
            .map_err(|_| "cannot start registered native collector")?,
    );
    let status = child
        .0
        .wait_timeout(Duration::from_secs(420))
        .map_err(|_| "cannot wait for native collector")?;
    // Drop kills the owned group even if a launcher left a descendant behind.
    drop(child);
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
    let context = json!({"schema_version":1,"complete":false,"effective_native_configuration":"unresolved","runtime_version":plan.native.version,"selected_binary_sha256":plan.native.executable_sha256,"selected_binary_unchanged":unchanged,"credential_source":"caller_owned_not_read_or_copied"});
    let complete = status.is_some_and(|s| s.success())
        && final_matches
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

#[cfg(all(test, unix))]
mod process_tests {
    use super::*;
    #[test]
    fn fake_collector_uses_foreground_child_and_retains_only_finite_journal_fields() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let prefix = "cli_manifests/codex/reports/0.156.1/evidence/native-v1/attempt";
        fs::create_dir_all(root.join(prefix)).unwrap();
        let project = tempfile::tempdir().unwrap();
        let executable = root.join("fake-closed-collector");
        // This fake only produces a finite failed case. Neither it nor the test invokes Codex.
        let sha = "b".repeat(64);
        let step = form::NativeStep {
            case: requested(&sha)[0].clone(),
            observation: form::NativeObservation::default(),
            outcome: CaseOutcome::Failed,
            elapsed_ms: 1,
            failure: Some(form::NativeFailure::RequestRefused),
        };
        // Harness identity is checked before spawn; self-reference is avoided by substituting the
        // actual measured identity into the journal at runtime using the sha256sum system utility.
        let step_json = String::from_utf8(canonical(&step).unwrap()).unwrap();
        let script=format!("#!/bin/sh\nwhile [ $# -gt 0 ]; do\n case \"$1\" in --workspace) shift; workspace=$1;; --output-dir) shift; output=$1;; esac\n shift\ndone\nmkdir \"$workspace/$output\"\ndigest=$(/usr/bin/sha256sum \"$0\"); digest=${{digest%% *}}\ncat <<'FINITE' | /usr/bin/sed \"s/{sha}/$digest/g\" > \"$workspace/$output/case-0.json\"\n{step_json}\nFINITE\necho sk-secret-native-stdout\necho sk-secret-native-error >&2\nexit 1\n");
        fs::write(&executable, script).unwrap();
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o700)).unwrap();
        let digest = fingerprint(&executable).unwrap();
        let binary = root.join("native-not-executed");
        fs::write(&binary, b"native bytes only").unwrap();
        let mut record = crate::depth_ci::profile::Profile::Core
            .record(include_str!(
                "../../../../../tests/fixtures/depth-core/record.json"
            ))
            .unwrap();
        let obligation = crate::depth_record::due(&record)
            .unwrap()
            .into_iter()
            .next()
            .unwrap();
        let scope = Scope {
            agent_id: "codex".into(),
            version: "0.156.1".into(),
            event: record.event.clone(),
            enrollment_origin: EnrollmentOrigin::Isolated,
            obligation,
            policy_identity: record.policy.identity.clone(),
            obligations_identity: record.obligations.take().unwrap().identity,
            rules_revision: "1".into(),
            trust_revision: "maintainer-native-v1".into(),
            review_revision: "maintainer-mapping-v1".into(),
        };
        let file = crate::depth_ci::reference("source", b"source");
        let build = Build {
            source_commit: "a".repeat(40),
            source_tree: "b".repeat(40),
            host_target: crate::depth_ci::TARGET.into(),
            toolchain: file.clone(),
            features: vec![],
            environment: file.clone(),
            dependency_graph: file.clone(),
            covered_inputs: vec![file.clone()],
            absent_inputs: vec![],
        };
        let plan = form::Plan {
            schema_version: 1,
            scope,
            packet_head: "c".repeat(40),
            build: build.clone(),
            native: form::Runtime {
                version: "0.156.1".into(),
                target: "x86_64-unknown-linux-musl".into(),
                executable_sha256: fingerprint(&binary).unwrap(),
                acquisition: file.clone(),
            },
            context: crate::depth_ci::reference(format!("{prefix}/intent.json"), b"intent"),
            collector: file,
            requested: requested(&digest),
        };
        let built = crate::depth_ci::native::Built {
            _target: tempfile::tempdir().unwrap(),
            executable,
            executable_sha256: digest,
            build,
            payloads: vec![],
        };
        let args = CollectArgs {
            live: true,
            register: true,
            binary: binary.clone(),
            working_dir: project.path().into(),
            codex_home: None,
            attempt: "attempt".into(),
        };
        let result = run(root, &args, &binary, &built, &plan).unwrap();
        assert!(!result.complete);
        assert_eq!(result.context["complete"], false);
        assert_eq!(result.cases.requested.len(), 12);
        assert_eq!(result.cases.executed.len(), 1);
        assert_eq!(result.cases.executed[0].outcome, CaseOutcome::Failed);
        let retained = serde_json::to_string(&result.observations).unwrap();
        assert!(!retained.contains("sk-secret"));
    }
}
