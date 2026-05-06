use super::*;

pub(super) fn execute_codex_phase(
    workspace_root: &Path,
    context: &Context,
    phase: Phase,
    prompt: &str,
) -> Result<CodexExecutionEvidence, Error> {
    let binary = resolve_codex_binary(&Args {
        dry_run: false,
        write: true,
        pass: context.pass,
        run_id: Some(context.run_id.clone()),
        prior_run_dir: context.prior_run_dir.clone(),
        codex_binary: Some(context.codex_binary.clone()),
    });
    let argv = vec![
        "exec".to_string(),
        "--skip-git-repo-check".to_string(),
        "--dangerously-bypass-approvals-and-sandbox".to_string(),
        "--cd".to_string(),
        workspace_root.display().to_string(),
    ];
    let mut child = Command::new(&binary)
        .current_dir(workspace_root)
        .args(&argv)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| Error::Internal(format!("spawn codex binary `{binary}`: {err}")))?;
    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| Error::Internal("codex exec stdin was not captured".to_string()))?;
        stdin
            .write_all(prompt.as_bytes())
            .map_err(|err| Error::Internal(format!("write codex prompt to stdin: {err}")))?;
    }
    let output = child
        .wait_with_output()
        .map_err(|err| Error::Internal(format!("wait for codex exec: {err}")))?;
    let stdout_path = context.packet_dir.join(phase.stdout_file_name());
    let stderr_path = context.packet_dir.join(phase.stderr_file_name());
    write_string(&stdout_path, &String::from_utf8_lossy(&output.stdout))?;
    write_string(&stderr_path, &String::from_utf8_lossy(&output.stderr))?;
    Ok(CodexExecutionEvidence {
        workflow_version: WORKFLOW_VERSION.to_string(),
        generated_at: now_rfc3339()?,
        run_id: context.run_id.clone(),
        phase: phase.as_str().to_string(),
        binary,
        argv,
        prompt_path: format!("{}/{}", context.packet_dir_rel, phase.prompt_file_name()),
        stdout_path: format!("{}/{}", context.packet_dir_rel, phase.stdout_file_name()),
        stderr_path: format!("{}/{}", context.packet_dir_rel, phase.stderr_file_name()),
        exit_code: output.status.code().unwrap_or(1),
    })
}

pub(super) fn execute_freeze_discovery(
    workspace_root: &Path,
    context: &Context,
) -> Result<SubprocessEvidence, Error> {
    let argv = vec![
        "scripts/recommend_next_agent.py".to_string(),
        "freeze-discovery".to_string(),
        "--discovery-dir".to_string(),
        context.discovery_dir_rel.clone(),
        "--research-dir".to_string(),
        context.research_dir_rel.clone(),
    ];
    let output = Command::new("python3")
        .current_dir(workspace_root)
        .args(&argv)
        .output()
        .map_err(|err| Error::Internal(format!("spawn freeze-discovery: {err}")))?;
    Ok(SubprocessEvidence {
        binary: "python3".to_string(),
        argv,
        exit_code: output.status.code().unwrap_or(1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

pub(super) fn write_header<W: Write>(
    writer: &mut W,
    context: &Context,
    is_write: bool,
) -> Result<(), Error> {
    writeln!(
        writer,
        "== RECOMMEND-NEXT-AGENT-RESEARCH {} ==",
        if is_write { "WRITE" } else { "DRY RUN" }
    )
    .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "run_id: {}", context.run_id)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "pass: {}", context.pass.as_str())
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "packet_root: {}", context.packet_dir_rel)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    Ok(())
}

pub(super) fn resolve_workspace_root() -> Result<PathBuf, Error> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| {
            Error::Internal(format!(
                "resolve workspace root from manifest dir `{}`",
                manifest_dir.display()
            ))
        })
}

pub(super) fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), Error> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|err| Error::Internal(format!("serialize json {}: {err}", path.display())))?;
    bytes.push(b'\n');
    write_bytes(path, &bytes)
}

pub(super) fn read_string(path: &Path) -> Result<String, Error> {
    fs::read_to_string(path)
        .map_err(|err| Error::Validation(format!("read {}: {err}", path.display())))
}

pub(super) fn load_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, Error> {
    let bytes = fs::read(path)
        .map_err(|err| Error::Validation(format!("read {}: {err}", path.display())))?;
    serde_json::from_slice(&bytes)
        .map_err(|err| Error::Validation(format!("parse {}: {err}", path.display())))
}

pub(super) fn write_string(path: &Path, value: &str) -> Result<(), Error> {
    write_bytes(path, value.as_bytes())
}

fn write_bytes(path: &Path, bytes: &[u8]) -> Result<(), Error> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| Error::Internal(format!("create {}: {err}", parent.display())))?;
    }
    fs::write(path, bytes)
        .map_err(|err| Error::Internal(format!("write {}: {err}", path.display())))
}

pub(super) fn snapshot_workspace(
    root: &Path,
    ignored_roots: &[&Path],
) -> Result<WorkspaceSnapshot, Error> {
    let mut files = Vec::new();
    collect_snapshot_files(root, root, ignored_roots, &mut files)?;
    files.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(WorkspaceSnapshot { files })
}

fn collect_snapshot_files(
    root: &Path,
    current: &Path,
    ignored_roots: &[&Path],
    out: &mut Vec<SnapshotFile>,
) -> Result<(), Error> {
    for entry in fs::read_dir(current)
        .map_err(|err| Error::Internal(format!("read {}: {err}", current.display())))?
    {
        let entry = entry
            .map_err(|err| Error::Internal(format!("read entry {}: {err}", current.display())))?;
        let path = entry.path();
        if ignored_roots
            .iter()
            .any(|ignored| path.starts_with(ignored))
        {
            continue;
        }
        let metadata = entry
            .metadata()
            .map_err(|err| Error::Internal(format!("stat {}: {err}", path.display())))?;
        if metadata.is_dir() {
            collect_snapshot_files(root, &path, ignored_roots, out)?;
            continue;
        }
        if !metadata.is_file() {
            continue;
        }
        let rel = path
            .strip_prefix(root)
            .map_err(|err| Error::Internal(format!("strip prefix {}: {err}", path.display())))?;
        let rel_text = rel.to_string_lossy().replace('\\', "/");
        let bytes = fs::read(&path)
            .map_err(|err| Error::Internal(format!("read {}: {err}", path.display())))?;
        out.push(SnapshotFile {
            path: rel_text,
            sha256: sha256_bytes(&bytes),
        });
    }
    Ok(())
}

pub(super) fn diff_snapshots(before: &WorkspaceSnapshot, after: &WorkspaceSnapshot) -> Vec<String> {
    let before_map = before
        .files
        .iter()
        .map(|file| (file.path.as_str(), file.sha256.as_str()))
        .collect::<BTreeMap<_, _>>();
    let after_map = after
        .files
        .iter()
        .map(|file| (file.path.as_str(), file.sha256.as_str()))
        .collect::<BTreeMap<_, _>>();
    let paths = before_map
        .keys()
        .chain(after_map.keys())
        .copied()
        .collect::<BTreeSet<_>>();
    paths
        .into_iter()
        .filter(|path| before_map.get(path) != after_map.get(path))
        .map(ToString::to_string)
        .collect()
}

pub(super) fn now_rfc3339() -> Result<String, Error> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|err| Error::Internal(format!("format timestamp: {err}")))
}

pub(super) fn sha256_bytes(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

pub(super) fn sha256_hex(path: &Path) -> Result<String, std::io::Error> {
    fs::read(path).map(|bytes| sha256_bytes(&bytes))
}
