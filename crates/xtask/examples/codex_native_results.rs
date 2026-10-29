//! Explicitly opt-in native diagnostics using the caller's existing Codex configuration/auth.
//! The exported Cases are claims, never authenticated evidence or qualification.
#[path = "codex_native_results/io.rs"]
mod io;
#[path = "codex_native_results/record.rs"]
mod record;
use agent_api::{
    backends::codex::{CodexBackend, CodexBackendConfig},
    AgentWrapperBackend, AgentWrapperError, AgentWrapperRunRequest,
};
use clap::{Parser, ValueEnum};
use futures_util::StreamExt;
use serde::Serialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};
use xtask::{
    depth_evidence::{
        native::{NativeFailure, NativeStep as StepResult},
        CaseIdentity, CaseOutcome, CaseResult, Cases,
    },
    depth_record::FileRef,
};

#[derive(Debug, Clone, Copy, Serialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
enum Variant {
    Unattended,
    Interactive,
}
impl Variant {
    fn name(self) -> &'static str {
        match self {
            Self::Unattended => "unattended",
            Self::Interactive => "interactive",
        }
    }
}
#[derive(Parser)]
#[command(about = "Opt-in Codex/UAA native result collection. Local results remain unverified.")]
struct Args {
    /// Explicit source workspace for a closed-build collector moved to a fresh target directory.
    #[arg(long)]
    workspace: Option<PathBuf>,
    /// Required acknowledgment: native calls use the caller's authentication and usage allowance.
    #[arg(long, required = true)]
    live: bool,
    /// Explicit Codex executable, without an account- or machine-specific default path.
    #[arg(long)]
    binary: PathBuf,
    /// Use a disposable project to avoid last-session selection racing unrelated caller runs.
    #[arg(long)]
    working_dir: PathBuf,
    /// Optional existing Codex home; forwarded to the SDK. Never copied or read by this command.
    #[arg(long)]
    codex_home: Option<PathBuf>,
    /// Optional ordinary SDK model override; otherwise use native caller configuration.
    #[arg(long)]
    model: Option<String>,
    /// Refuse a different installed version before the first SDK run.
    #[arg(long)]
    expected_version: Option<semver::Version>,
    /// One variant, or both when omitted. External-sandbox bypass is never enabled.
    #[arg(long, value_enum)]
    variant: Option<Variant>,
    /// New repository-relative output directory; existing destinations are never overwritten.
    #[arg(long)]
    output_dir: String,
}
const REPLY: &str = "UAA_SMOKE_OK";
fn identity(variant: Variant, step: &str, executable: &str) -> CaseIdentity {
    CaseIdentity {
        name: format!("codex-native/{}/{step}", variant.name()),
        executable_sha256: executable.into(),
    }
}
async fn observe(
    backend: &CodexBackend,
    variant: Variant,
    step: &str,
    seed: Option<&str>,
) -> (record::Observation, Option<NativeFailure>) {
    let mut extensions = BTreeMap::new();
    if matches!(variant, Variant::Interactive) {
        extensions.insert("agent_api.exec.non_interactive".into(), json!(false));
        extensions.insert(
            "backend.codex.exec.approval_policy".into(),
            json!("on-request"),
        );
    }
    if step.starts_with("resume") || step.starts_with("fork") {
        let key = if step.starts_with("resume") {
            "agent_api.session.resume.v1"
        } else {
            "agent_api.session.fork.v1"
        };
        let selector = if step.ends_with("-last") {
            json!({"selector":"last"})
        } else {
            json!({"selector":"id","id":seed})
        };
        extensions.insert(key.into(), selector);
    }
    let request=AgentWrapperRunRequest {
        prompt:format!("Reply with exactly the single token {REPLY} and no other text. Do not run tools or change files."),
        timeout:Some(Duration::from_secs(30)), extensions,..Default::default()
    };
    let mut observation = record::Observation {
        consistent: true,
        ..Default::default()
    };
    let control = match backend.run_control(request).await {
        Ok(c) => c,
        Err(_) => return (observation, Some(NativeFailure::RequestRefused)),
    };
    let events = async {
        let mut stream = control.handle.events;
        let mut count = 0;
        while let Some(event) = stream.next().await {
            count += 1;
            record::bind_session(
                event.data.as_ref(),
                &mut observation.session_id,
                &mut observation.consistent,
            );
            if count > 4096 || record::unexpected_item(&event.kind, event.data.as_ref()) {
                observation.unexpected_item = true;
                control.cancel.cancel();
            }
            if step == "cancel" && observation.session_id.is_some() && !observation.cancel_requested
            {
                control.cancel.cancel();
                observation.cancel_requested = true;
            }
        }
    };
    let (_, completion) = tokio::join!(events, control.handle.completion);
    let failure = match completion {
        Ok(c) => {
            record::bind_session(
                c.data.as_ref(),
                &mut observation.session_id,
                &mut observation.consistent,
            );
            observation.completion_success = c.status.success();
            observation.reply_matches = c.final_text.as_deref() == Some(REPLY);
            None
        }
        Err(AgentWrapperError::Backend { message }) if message == "cancelled" => {
            observation.cancelled = true;
            Some(NativeFailure::Cancelled)
        }
        Err(_) => Some(NativeFailure::CompletionFailed),
    };
    (observation, failure)
}
fn workspace(root: &Path) -> Result<serde_json::Value, String> {
    let git = |args: &[&str]| -> Result<String, String> {
        let out = Command::new("git")
            .args(["--no-replace-objects", "-c", "core.fsmonitor=false", "-C"])
            .arg(root)
            .args(args)
            .output()
            .map_err(|_| "cannot observe workspace Git identity")?;
        if !out.status.success() {
            return Err("requires the repository checkout".into());
        }
        String::from_utf8(out.stdout).map_err(|_| "invalid Git identity".into())
    };
    let commit = git(&["rev-parse", "HEAD"])?;
    let tree = git(&["rev-parse", "HEAD^{tree}"])?;
    if ![&commit, &tree].iter().all(|id| {
        id.trim().len() == 40
            && id
                .trim()
                .bytes()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
    }) {
        return Err("unsupported Git identity".into());
    }
    Ok(
        json!({"observed_commit":commit.trim(),"observed_tree":tree.trim(),"clean":git(&["status","--porcelain"])?.is_empty(),"build_attested":false}),
    )
}
async fn collect(root: &Path, args: Args) -> Result<bool, String> {
    if !args.live {
        return Err("native calls require --live".into());
    }
    io::output_path(&args.output_dir)?;
    let output = root.join(&args.output_dir);
    let parent = output.parent().ok_or("missing output parent")?;
    // Existing parents only: no speculative directory tree or overwrite of an old run.
    let mut ancestor = root.to_path_buf();
    for component in Path::new(&args.output_dir)
        .parent()
        .ok_or("missing output parent")?
        .components()
    {
        ancestor.push(component);
        let metadata = fs::symlink_metadata(&ancestor)
            .map_err(|_| "create the output parent directory first")?;
        if metadata.is_symlink() || !metadata.is_dir() {
            return Err("output ancestors must be regular directories".into());
        }
    }
    let canonical_parent = parent
        .canonicalize()
        .map_err(|_| "create the output parent directory first")?;
    if !canonical_parent.starts_with(root) {
        return Err("output parent must remain in the repository".into());
    }
    if output.symlink_metadata().is_ok() {
        return Err("output directory already exists; preserve the prior attempt".into());
    }
    let binary = args
        .binary
        .canonicalize()
        .map_err(|_| "selected executable is unavailable")?;
    let work = args
        .working_dir
        .canonicalize()
        .map_err(|_| "working directory is unavailable")?;
    if !work.is_dir() {
        return Err("working directory must be a directory".into());
    }
    let home = args
        .codex_home
        .map(|p| {
            p.canonicalize()
                .map_err(|_| "configured Codex home is unavailable")
        })
        .transpose()?;
    let version = io::version(&binary).await?;
    if args
        .expected_version
        .as_ref()
        .is_some_and(|expected| expected.to_string() != version)
    {
        return Err("selected Codex version differs from --expected-version".into());
    }
    let native_sha = io::fingerprint(&binary)?;
    let runner_sha =
        io::fingerprint(&std::env::current_exe().map_err(|_| "cannot identify collector")?)?;
    let source = workspace(root)?;
    fs::create_dir(&output).map_err(|_| "cannot reserve new output directory")?;
    if output
        .canonicalize()
        .map_err(|_| "cannot bind output directory")?
        != canonical_parent.join(output.file_name().ok_or("missing output name")?)
    {
        return Err("output directory changed".into());
    }
    let variants = args
        .variant
        .map(|v| vec![v])
        .unwrap_or_else(|| vec![Variant::Unattended, Variant::Interactive]);
    let requested = variants
        .iter()
        .flat_map(|v| record::STEPS.map(|step| identity(*v, step, &runner_sha)))
        .collect::<Vec<_>>();
    let backend = CodexBackend::new(CodexBackendConfig {
        binary: Some(binary.clone()),
        codex_home: home,
        model: args.model,
        default_working_dir: Some(work),
        ..Default::default()
    });
    let mut results = Vec::new();
    let mut stopped = false;
    for variant in &variants {
        let mut state = record::State::default();
        for step in record::STEPS {
            if stopped {
                break;
            }
            let started = Instant::now();
            let (observation, failure) = observe(&backend, *variant, step, state.seed()).await;
            let outcome = state.record(step, &observation);
            stopped = outcome != CaseOutcome::Passed;
            results.push(StepResult {
                case: identity(*variant, step, &runner_sha),
                observation,
                outcome,
                elapsed_ms: started.elapsed().as_millis(),
                failure,
            });
            // A completed case survives a later crash/timeout. create_new prevents retries from
            // silently replacing outcomes; no raw native text or credential material is journaled.
            let bytes = serde_json::to_vec(results.last().ok_or("missing case result")?)
                .map_err(|_| "cannot encode native case")?;
            if bytes.len() > 1_048_576 {
                return Err("native case exceeds output bound".into());
            }
            use std::io::Write;
            let mut journal = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(output.join(format!("case-{}.json", results.len() - 1)))
                .map_err(|_| "cannot preserve native case journal")?;
            journal
                .write_all(&bytes)
                .and_then(|()| journal.sync_all())
                .map_err(|_| "cannot preserve native case journal")?;
        }
    }
    let unchanged = io::fingerprint(&binary).ok().as_deref() == Some(native_sha.as_str());
    let complete = unchanged
        && results.len() == requested.len()
        && results.iter().all(|r| r.outcome == CaseOutcome::Passed);
    let observations = json!({"schema_version":1,"kind":"codex-native-diagnostic","runtime_version":version,
        "selected_binary_sha256":native_sha,"selected_binary_unchanged":unchanged,"runner_sha256":runner_sha,
        "workspace":source,"variants":variants,"per_request_timeout_seconds":30,"results":results,
        "complete":complete,"evidence_status":"unverified","packet_binding":null,
        "limits":["local result is not authenticated producer evidence","effective native config and loader closure are not observed",
            "selected executable may be a launcher; transitive native/build closure is not attested",
            "tool/approval and external-sandbox regimes are not qualified","no independent adequacy, adoption or packet binding"]});
    let raw =
        serde_json::to_vec_pretty(&observations).map_err(|_| "cannot serialize diagnostics")?;
    if raw.len() > 1_048_576 {
        return Err("native diagnostic exceeds output bound".into());
    }
    let cases = Cases {
        requested,
        executed: results
            .iter()
            .map(|r| CaseResult {
                case: r.case.clone(),
                outcome: r.outcome.clone(),
            })
            .collect(),
        raw_output: FileRef {
            path: format!("{}/observations.json", args.output_dir),
            sha256: hex::encode(Sha256::digest(&raw)),
        },
    };
    for (name, bytes) in [
        ("observations.json", raw),
        (
            "cases.json",
            serde_json::to_vec_pretty(&cases).map_err(|_| "cannot serialize case results")?,
        ),
    ] {
        use std::io::Write;
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(output.join(name))
            .map_err(|_| "cannot create native result file")?;
        f.write_all(&bytes)
            .map_err(|_| "cannot write native result file")?;
    }
    println!("Native diagnostic saved; qualification remains unverified.");
    Ok(complete)
}
#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = Args::parse();
    let root = args
        .workspace
        .clone()
        .map(Ok)
        .unwrap_or_else(std::env::current_dir);
    let result = match root.and_then(|p| p.canonicalize()) {
        Ok(root) => collect(&root, args).await,
        Err(_) => Err("repository checkout is unavailable".into()),
    };
    match result {
        Ok(true) => {}
        Ok(false) => std::process::exit(1),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    fn fake_args(root: &Path) -> (tempfile::TempDir, Args) {
        use std::os::unix::fs::PermissionsExt;
        fs::create_dir_all(root.join("target")).unwrap();
        let tmp = tempfile::Builder::new()
            .prefix("native-test-")
            .tempdir_in(root.join("target"))
            .unwrap();
        let binary = tmp.path().join("fake-codex");
        fs::write(&binary,"#!/bin/sh\nif [ \"$1\" = --version ]; then echo 'codex-cli 0.162.1'; exit 0; fi\necho sk-sensitive-native-error >&2\nexit 1\n").unwrap();
        fs::set_permissions(&binary, fs::Permissions::from_mode(0o700)).unwrap();
        let home = tmp.path().join("empty-native-home");
        fs::create_dir(&home).unwrap();
        let args = Args {
            workspace: None,
            live: true,
            binary,
            working_dir: tmp.path().into(),
            codex_home: Some(home),
            model: None,
            expected_version: None,
            variant: None,
            output_dir: tmp
                .path()
                .join("results")
                .strip_prefix(root)
                .unwrap()
                .to_str()
                .unwrap()
                .into(),
        };
        (tmp, args)
    }
    #[cfg(unix)]
    #[tokio::test]
    async fn failed_native_run_preserves_population_and_binds_sanitized_existing_case_format() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let (_tmp, args) = fake_args(&root);
        let output = root.join(&args.output_dir);
        assert!(!collect(&root, args).await.unwrap());
        let raw = fs::read(output.join("observations.json")).unwrap();
        let cases: Cases =
            serde_json::from_slice(&fs::read(output.join("cases.json")).unwrap()).unwrap();
        assert_eq!(cases.requested.len(), 12);
        assert_eq!(cases.executed.len(), 1);
        assert_eq!(cases.executed[0].outcome, CaseOutcome::Failed);
        assert_eq!(cases.raw_output.sha256, hex::encode(Sha256::digest(&raw)));
        assert_eq!(fs::read(root.join(&cases.raw_output.path)).unwrap(), raw);
        let report: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        assert_eq!(report["evidence_status"], "unverified");
        assert_eq!(report["complete"], false);
        assert!(report["packet_binding"].is_null());
        assert!(!String::from_utf8(raw)
            .unwrap()
            .contains("sk-sensitive-native-error"));
    }
    #[cfg(unix)]
    #[tokio::test]
    async fn wrong_version_and_existing_destination_refuse_before_native_run() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let (_tmp, mut args) = fake_args(&root);
        let output = root.join(&args.output_dir);
        args.expected_version = Some(semver::Version::new(0, 156, 1));
        let error = collect(&root, args).await.unwrap_err();
        assert!(error.contains("differs"), "unexpected refusal: {error}");
        assert!(!output.exists());
        let (_tmp, mut args) = fake_args(&root);
        let output = root.join(&args.output_dir);
        fs::create_dir(&output).unwrap();
        fs::write(output.join("prior"), "keep this attempt").unwrap();
        args.binary = root.join("not-an-executable");
        assert!(collect(&root, args)
            .await
            .unwrap_err()
            .contains("already exists"));
        assert_eq!(
            fs::read_to_string(output.join("prior")).unwrap(),
            "keep this attempt"
        );
    }
    #[cfg(unix)]
    #[tokio::test]
    async fn output_parent_symlink_cannot_alias_git_or_another_location() {
        use std::os::unix::fs::symlink;
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let (tmp, mut args) = fake_args(&root);
        let alias = tmp.path().join("alias");
        symlink(root.join(".git"), &alias).unwrap();
        args.output_dir = alias
            .join("results")
            .strip_prefix(&root)
            .unwrap()
            .to_str()
            .unwrap()
            .into();
        args.binary = root.join("not-an-executable");
        assert!(collect(&root, args)
            .await
            .unwrap_err()
            .contains("regular directories"));
    }
    #[test]
    fn live_acknowledgment_is_required_by_parser_before_any_probe() {
        assert!(Args::try_parse_from([
            "native-results",
            "--binary",
            "/not/invoked",
            "--working-dir",
            "/unused",
            "--output-dir",
            "target/results"
        ])
        .is_err());
        assert!(Args::try_parse_from([
            "native-results",
            "--live",
            "--binary",
            "/not/invoked",
            "--working-dir",
            "/unused",
            "--output-dir",
            "target/results"
        ])
        .is_ok());
    }
}
