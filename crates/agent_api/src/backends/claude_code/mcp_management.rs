#![allow(dead_code)]

use std::{
    collections::BTreeMap,
    env,
    ffi::OsString,
    io,
    path::PathBuf,
    process::{ExitStatus, Stdio},
    time::Duration,
};

use claude_code::ClaudeHomeLayout;
use tokio::{
    io::{AsyncRead, AsyncReadExt},
    process::{Child, Command},
    task::JoinHandle,
};

use crate::{
    bounds::{enforce_mcp_output_bound, MCP_STDERR_BOUND_BYTES, MCP_STDOUT_BOUND_BYTES},
    mcp::{
        AgentWrapperMcpAddTransport, AgentWrapperMcpCommandContext, AgentWrapperMcpCommandOutput,
    },
    AgentWrapperError,
};

const CLAUDE_BINARY_ENV: &str = "CLAUDE_BINARY";
const CLAUDE_HOME_ENV: &str = "CLAUDE_HOME";
const DISABLE_AUTOUPDATER_ENV: &str = "DISABLE_AUTOUPDATER";
const HOME_ENV: &str = "HOME";
const XDG_CACHE_HOME_ENV: &str = "XDG_CACHE_HOME";
const XDG_CONFIG_HOME_ENV: &str = "XDG_CONFIG_HOME";
const XDG_DATA_HOME_ENV: &str = "XDG_DATA_HOME";
#[cfg(windows)]
const USERPROFILE_ENV: &str = "USERPROFILE";
#[cfg(windows)]
const APPDATA_ENV: &str = "APPDATA";
#[cfg(windows)]
const LOCALAPPDATA_ENV: &str = "LOCALAPPDATA";

const PINNED_CAPTURE_FAILURE: &str =
    "claude_code backend error: capture (details redacted when unsafe)";
const PINNED_MCP_RUNTIME_CONFLICT: &str =
    "claude_code backend error: installed claude does not support pinned mcp management command shape (details redacted)";
const PINNED_PREPARE_CLAUDE_HOME_FAILURE: &str =
    "claude_code backend error: prepare CLAUDE_HOME (details redacted when unsafe)";
const PINNED_SPAWN_FAILURE: &str =
    "claude_code backend error: spawn (details redacted when unsafe)";
const PINNED_TIMEOUT_FAILURE: &str =
    "claude_code backend error: timeout (details redacted when unsafe)";
const PINNED_WAIT_FAILURE: &str = "claude_code backend error: wait (details redacted when unsafe)";
const PINNED_URL_BEARER_TOKEN_ENV_VAR_UNSUPPORTED: &str =
    "claude mcp add url transport does not support bearer_token_env_var";

#[derive(Clone, Debug, PartialEq, Eq)]
struct ResolvedClaudeMcpCommand {
    binary_path: PathBuf,
    working_dir: Option<PathBuf>,
    timeout: Option<Duration>,
    env: BTreeMap<String, String>,
    materialize_claude_home: Option<ClaudeHomeLayout>,
}

// Keep the captured bytes separate from final bounded strings so S1c can insert
// drift classification without changing the runner shape again.
#[derive(Debug)]
struct CapturedClaudeMcpCommandOutput {
    status: ExitStatus,
    stdout_bytes: Vec<u8>,
    stdout_saw_more: bool,
    stderr_bytes: Vec<u8>,
    stderr_saw_more: bool,
}

pub(super) fn claude_mcp_list_argv() -> Vec<OsString> {
    vec![OsString::from("mcp"), OsString::from("list")]
}

pub(super) fn claude_mcp_get_argv(name: &str) -> Vec<OsString> {
    vec![
        OsString::from("mcp"),
        OsString::from("get"),
        OsString::from(name),
    ]
}

pub(super) fn claude_mcp_remove_argv(name: &str) -> Vec<OsString> {
    vec![
        OsString::from("mcp"),
        OsString::from("remove"),
        OsString::from(name),
    ]
}

pub(super) fn claude_mcp_add_argv(
    name: &str,
    transport: &AgentWrapperMcpAddTransport,
) -> Result<Vec<OsString>, AgentWrapperError> {
    let mut argv = vec![OsString::from("mcp"), OsString::from("add")];

    match transport {
        AgentWrapperMcpAddTransport::Stdio { command, args, env } => {
            argv.push(OsString::from("--transport"));
            argv.push(OsString::from("stdio"));
            for (key, value) in env {
                argv.push(OsString::from("--env"));
                argv.push(OsString::from(format!("{key}={value}")));
            }
            argv.push(OsString::from(name));
            argv.extend(command.iter().map(OsString::from));
            argv.extend(args.iter().map(OsString::from));
            Ok(argv)
        }
        AgentWrapperMcpAddTransport::Url {
            url,
            bearer_token_env_var,
        } => {
            if bearer_token_env_var.is_some() {
                return Err(invalid_request(PINNED_URL_BEARER_TOKEN_ENV_VAR_UNSUPPORTED));
            }

            argv.push(OsString::from("--transport"));
            argv.push(OsString::from("http"));
            argv.push(OsString::from(name));
            argv.push(OsString::from(url));
            Ok(argv)
        }
    }
}

pub(super) async fn run_claude_mcp(
    config: super::ClaudeCodeBackendConfig,
    argv: Vec<OsString>,
    context: AgentWrapperMcpCommandContext,
) -> Result<AgentWrapperMcpCommandOutput, AgentWrapperError> {
    let resolved = resolve_claude_mcp_command(&config, &context);
    let captured = capture_claude_mcp_output(&resolved, &argv).await?;
    finalize_claude_mcp_output(&argv, captured)
}

fn resolve_claude_mcp_command(
    config: &super::ClaudeCodeBackendConfig,
    context: &AgentWrapperMcpCommandContext,
) -> ResolvedClaudeMcpCommand {
    resolve_claude_mcp_command_with_env(config, context, env::var(CLAUDE_BINARY_ENV).ok())
}

fn resolve_claude_mcp_command_with_env(
    config: &super::ClaudeCodeBackendConfig,
    context: &AgentWrapperMcpCommandContext,
    claude_binary_env: Option<String>,
) -> ResolvedClaudeMcpCommand {
    let binary_path = resolve_claude_binary_path(config.binary.as_ref(), claude_binary_env);
    let mut env = config.env.clone();
    env.entry(DISABLE_AUTOUPDATER_ENV.to_string())
        .or_insert_with(|| "1".to_string());

    let materialize_claude_home = config
        .claude_home
        .as_ref()
        .map(|path| ClaudeHomeLayout::new(path.clone()));
    if let Some(layout) = materialize_claude_home.as_ref() {
        inject_claude_home_env(&mut env, layout);
    }

    env.extend(context.env.clone());

    ResolvedClaudeMcpCommand {
        binary_path,
        working_dir: context
            .working_dir
            .clone()
            .or_else(|| config.default_working_dir.clone()),
        timeout: context.timeout.or(config.default_timeout),
        env,
        materialize_claude_home,
    }
}

fn resolve_claude_binary_path(
    config_binary: Option<&PathBuf>,
    claude_binary_env: Option<String>,
) -> PathBuf {
    if let Some(binary) = config_binary {
        return binary.clone();
    }
    if let Some(binary) = claude_binary_env {
        if !binary.trim().is_empty() {
            return PathBuf::from(binary);
        }
    }
    PathBuf::from("claude")
}

fn inject_claude_home_env(env: &mut BTreeMap<String, String>, layout: &ClaudeHomeLayout) {
    let root = layout.root().to_string_lossy().into_owned();
    env.entry(CLAUDE_HOME_ENV.to_string())
        .or_insert_with(|| root.clone());
    env.entry(HOME_ENV.to_string())
        .or_insert_with(|| root.clone());
    env.entry(XDG_CONFIG_HOME_ENV.to_string())
        .or_insert_with(|| layout.xdg_config_home().to_string_lossy().into_owned());
    env.entry(XDG_DATA_HOME_ENV.to_string())
        .or_insert_with(|| layout.xdg_data_home().to_string_lossy().into_owned());
    env.entry(XDG_CACHE_HOME_ENV.to_string())
        .or_insert_with(|| layout.xdg_cache_home().to_string_lossy().into_owned());

    #[cfg(windows)]
    {
        env.entry(USERPROFILE_ENV.to_string())
            .or_insert_with(|| root.clone());
        env.entry(APPDATA_ENV.to_string())
            .or_insert_with(|| layout.appdata_dir().to_string_lossy().into_owned());
        env.entry(LOCALAPPDATA_ENV.to_string())
            .or_insert_with(|| layout.localappdata_dir().to_string_lossy().into_owned());
    }
}

async fn capture_claude_mcp_output(
    resolved: &ResolvedClaudeMcpCommand,
    argv: &[OsString],
) -> Result<CapturedClaudeMcpCommandOutput, AgentWrapperError> {
    if let Some(layout) = resolved.materialize_claude_home.as_ref() {
        layout
            .materialize(true)
            .map_err(|_| backend_error(PINNED_PREPARE_CLAUDE_HOME_FAILURE))?;
    }

    let mut command = Command::new(&resolved.binary_path);
    command
        .args(argv)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .envs(&resolved.env);

    if let Some(working_dir) = resolved.working_dir.as_ref() {
        command.current_dir(working_dir);
    }

    let mut child = command
        .spawn()
        .map_err(|_| backend_error(PINNED_SPAWN_FAILURE))?;

    let Some(stdout) = child.stdout.take() else {
        cleanup_child(&mut child).await;
        return Err(backend_error(PINNED_CAPTURE_FAILURE));
    };
    let Some(stderr) = child.stderr.take() else {
        cleanup_child(&mut child).await;
        return Err(backend_error(PINNED_CAPTURE_FAILURE));
    };

    let stdout_task = tokio::spawn(capture_bounded(stdout, MCP_STDOUT_BOUND_BYTES));
    let stderr_task = tokio::spawn(capture_bounded(stderr, MCP_STDERR_BOUND_BYTES));

    let status = match wait_for_exit(&mut child, resolved.timeout).await {
        Ok(status) => status,
        Err(err) => {
            stdout_task.abort();
            stderr_task.abort();
            return Err(err);
        }
    };

    let (stdout_bytes, stdout_saw_more) = join_capture_task(stdout_task).await?;
    let (stderr_bytes, stderr_saw_more) = join_capture_task(stderr_task).await?;

    Ok(CapturedClaudeMcpCommandOutput {
        status,
        stdout_bytes,
        stdout_saw_more,
        stderr_bytes,
        stderr_saw_more,
    })
}

fn finalize_claude_mcp_output(
    argv: &[OsString],
    captured: CapturedClaudeMcpCommandOutput,
) -> Result<AgentWrapperMcpCommandOutput, AgentWrapperError> {
    if !captured.status.success()
        && is_manifest_runtime_conflict(argv, &captured.stdout_bytes, &captured.stderr_bytes)
    {
        return Err(backend_error(PINNED_MCP_RUNTIME_CONFLICT));
    }

    let (stdout, stdout_truncated) = enforce_mcp_output_bound(
        &captured.stdout_bytes,
        captured.stdout_saw_more,
        MCP_STDOUT_BOUND_BYTES,
    );
    let (stderr, stderr_truncated) = enforce_mcp_output_bound(
        &captured.stderr_bytes,
        captured.stderr_saw_more,
        MCP_STDERR_BOUND_BYTES,
    );

    Ok(AgentWrapperMcpCommandOutput {
        status: captured.status,
        stdout,
        stderr,
        stdout_truncated,
        stderr_truncated,
    })
}

fn is_manifest_runtime_conflict(argv: &[OsString], stdout: &[u8], stderr: &[u8]) -> bool {
    let stderr = String::from_utf8_lossy(stderr);
    let stdout = String::from_utf8_lossy(stdout);
    classify_manifest_runtime_conflict_text(argv, &format!("{stderr}\n{stdout}"))
}

fn classify_manifest_runtime_conflict_text(argv: &[OsString], text: &str) -> bool {
    let text = text.to_ascii_lowercase();

    let unknown_signal = [
        "unknown",
        "unrecognized",
        "unexpected",
        "invalid",
        "no such",
        "not recognized",
    ]
    .iter()
    .any(|signal| text.contains(signal));
    if !unknown_signal {
        return false;
    }

    let syntax_context = ["command", "subcommand", "argument", "option", "flag"]
        .iter()
        .any(|signal| text.contains(signal));
    if !syntax_context {
        return false;
    }

    manifest_conflict_tokens(argv)
        .into_iter()
        .any(|token| text.contains(token))
}

fn manifest_conflict_tokens(argv: &[OsString]) -> Vec<&'static str> {
    let mut tokens = vec!["mcp"];
    match argv.get(1).and_then(|arg| arg.to_str()) {
        Some("list") => tokens.push("list"),
        Some("get") => tokens.push("get"),
        Some("add") => tokens.push("add"),
        Some("remove") => tokens.push("remove"),
        _ => {}
    }
    tokens
}

fn effective_timeout_for_wait(timeout: Option<Duration>) -> Option<Duration> {
    match timeout {
        Some(timeout) if timeout == Duration::ZERO => None,
        other => other,
    }
}

async fn wait_for_exit(
    child: &mut Child,
    timeout: Option<Duration>,
) -> Result<ExitStatus, AgentWrapperError> {
    match effective_timeout_for_wait(timeout) {
        Some(timeout) => match tokio::time::timeout(timeout, child.wait()).await {
            Ok(Ok(status)) => Ok(status),
            Ok(Err(_)) => Err(backend_error(PINNED_WAIT_FAILURE)),
            Err(_) => {
                cleanup_child(child).await;
                Err(backend_error(PINNED_TIMEOUT_FAILURE))
            }
        },
        None => child
            .wait()
            .await
            .map_err(|_| backend_error(PINNED_WAIT_FAILURE)),
    }
}

async fn cleanup_child(child: &mut Child) {
    let _ = child.kill().await;
    let _ = child.wait().await;
}

async fn join_capture_task(
    task: JoinHandle<io::Result<(Vec<u8>, bool)>>,
) -> Result<(Vec<u8>, bool), AgentWrapperError> {
    task.await
        .map_err(|_| backend_error(PINNED_CAPTURE_FAILURE))?
        .map_err(|_| backend_error(PINNED_CAPTURE_FAILURE))
}

pub(super) async fn capture_bounded<R>(
    mut reader: R,
    bound_bytes: usize,
) -> io::Result<(Vec<u8>, bool)>
where
    R: AsyncRead + Unpin,
{
    let retain_bound = bound_bytes.saturating_add(1);
    let mut retained = Vec::with_capacity(retain_bound.min(4096));
    let mut saw_more = false;
    let mut chunk = [0u8; 4096];

    loop {
        let read = reader.read(&mut chunk).await?;
        if read == 0 {
            break;
        }

        if retained.len() < retain_bound {
            let remaining = retain_bound - retained.len();
            let to_copy = remaining.min(read);
            retained.extend_from_slice(&chunk[..to_copy]);
            if to_copy < read {
                saw_more = true;
            }
        } else {
            saw_more = true;
        }
    }

    if retained.len() > bound_bytes {
        retained.truncate(bound_bytes);
        saw_more = true;
    }

    Ok((retained, saw_more))
}

fn backend_error(message: &'static str) -> AgentWrapperError {
    AgentWrapperError::Backend {
        message: message.to_string(),
    }
}

fn invalid_request(message: &'static str) -> AgentWrapperError {
    AgentWrapperError::InvalidRequest {
        message: message.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tokio::io::{duplex, AsyncWriteExt, DuplexStream};

    fn success_exit_status() -> ExitStatus {
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            ExitStatus::from_raw(0)
        }
        #[cfg(windows)]
        {
            use std::os::windows::process::ExitStatusExt;
            ExitStatus::from_raw(0)
        }
    }

    fn exit_status_with_code(code: i32) -> ExitStatus {
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            ExitStatus::from_raw(code << 8)
        }
        #[cfg(windows)]
        {
            use std::os::windows::process::ExitStatusExt;
            ExitStatus::from_raw(code as u32)
        }
    }

    fn sample_config() -> super::super::ClaudeCodeBackendConfig {
        super::super::ClaudeCodeBackendConfig {
            binary: Some(PathBuf::from("/tmp/fake-claude")),
            claude_home: Some(PathBuf::from("/tmp/claude-home")),
            default_timeout: Some(Duration::from_secs(30)),
            default_working_dir: Some(PathBuf::from("default/workdir")),
            env: BTreeMap::from([
                ("CONFIG_ONLY".to_string(), "config-only".to_string()),
                ("OVERRIDE_ME".to_string(), "config".to_string()),
            ]),
            ..Default::default()
        }
    }

    fn sample_context() -> AgentWrapperMcpCommandContext {
        AgentWrapperMcpCommandContext {
            working_dir: Some(PathBuf::from("request/workdir")),
            timeout: Some(Duration::from_secs(5)),
            env: BTreeMap::from([
                ("OVERRIDE_ME".to_string(), "request".to_string()),
                ("REQUEST_ONLY".to_string(), "request-only".to_string()),
            ]),
        }
    }

    async fn write_all_and_close(mut writer: DuplexStream, bytes: Vec<u8>) {
        writer.write_all(&bytes).await.expect("write succeeds");
        writer.shutdown().await.expect("shutdown succeeds");
    }

    #[test]
    fn claude_mcp_list_argv_is_pinned() {
        assert_eq!(
            claude_mcp_list_argv(),
            vec![OsString::from("mcp"), OsString::from("list")]
        );
    }

    #[test]
    fn claude_mcp_get_argv_is_pinned() {
        assert_eq!(
            claude_mcp_get_argv("demo"),
            vec![
                OsString::from("mcp"),
                OsString::from("get"),
                OsString::from("demo"),
            ]
        );
    }

    #[test]
    fn claude_mcp_remove_argv_is_pinned() {
        assert_eq!(
            claude_mcp_remove_argv("demo"),
            vec![
                OsString::from("mcp"),
                OsString::from("remove"),
                OsString::from("demo"),
            ]
        );
    }

    #[test]
    fn claude_mcp_remove_argv_preserves_name_as_single_item() {
        assert_eq!(
            claude_mcp_remove_argv("demo server"),
            vec![
                OsString::from("mcp"),
                OsString::from("remove"),
                OsString::from("demo server"),
            ]
        );
    }

    #[test]
    fn claude_mcp_add_argv_maps_stdio_transport_with_sorted_env_and_no_separator() {
        let transport = AgentWrapperMcpAddTransport::Stdio {
            command: vec!["node".to_string()],
            args: vec!["server.js".to_string(), "--flag".to_string()],
            env: BTreeMap::from([
                ("BETA".to_string(), "two".to_string()),
                ("ALPHA".to_string(), "one".to_string()),
            ]),
        };

        assert_eq!(
            claude_mcp_add_argv("demo", &transport).expect("stdio transport should map"),
            vec![
                OsString::from("mcp"),
                OsString::from("add"),
                OsString::from("--transport"),
                OsString::from("stdio"),
                OsString::from("--env"),
                OsString::from("ALPHA=one"),
                OsString::from("--env"),
                OsString::from("BETA=two"),
                OsString::from("demo"),
                OsString::from("node"),
                OsString::from("server.js"),
                OsString::from("--flag"),
            ]
        );
    }

    #[test]
    fn claude_mcp_add_argv_maps_url_transport_without_bearer_env() {
        let transport = AgentWrapperMcpAddTransport::Url {
            url: "https://example.test/mcp".to_string(),
            bearer_token_env_var: None,
        };

        assert_eq!(
            claude_mcp_add_argv("demo", &transport).expect("url transport should map"),
            vec![
                OsString::from("mcp"),
                OsString::from("add"),
                OsString::from("--transport"),
                OsString::from("http"),
                OsString::from("demo"),
                OsString::from("https://example.test/mcp"),
            ]
        );
    }

    #[test]
    fn claude_mcp_add_argv_rejects_url_transport_with_bearer_env_var() {
        let transport = AgentWrapperMcpAddTransport::Url {
            url: "https://example.test/mcp".to_string(),
            bearer_token_env_var: Some("TOKEN_ENV".to_string()),
        };

        let err = claude_mcp_add_argv("demo", &transport)
            .expect_err("url bearer token env var should be rejected");

        match err {
            AgentWrapperError::InvalidRequest { message } => {
                assert_eq!(message, PINNED_URL_BEARER_TOKEN_ENV_VAR_UNSUPPORTED);
            }
            other => panic!("expected InvalidRequest, got: {other:?}"),
        }
    }

    #[test]
    fn resolve_claude_binary_path_prefers_config_over_env() {
        let resolved = resolve_claude_binary_path(
            Some(&PathBuf::from("/tmp/from-config")),
            Some("/tmp/from-env".to_string()),
        );

        assert_eq!(resolved, PathBuf::from("/tmp/from-config"));
    }

    #[test]
    fn resolve_claude_binary_path_uses_env_when_config_absent() {
        let resolved = resolve_claude_binary_path(None, Some("/tmp/from-env".to_string()));

        assert_eq!(resolved, PathBuf::from("/tmp/from-env"));
    }

    #[test]
    fn resolve_claude_binary_path_ignores_blank_env() {
        let resolved = resolve_claude_binary_path(None, Some("   ".to_string()));

        assert_eq!(resolved, PathBuf::from("claude"));
    }

    #[test]
    fn resolve_claude_mcp_command_applies_precedence_and_home_injection() {
        let resolved = resolve_claude_mcp_command_with_env(
            &sample_config(),
            &sample_context(),
            Some("/tmp/from-env".to_string()),
        );
        let layout = ClaudeHomeLayout::new("/tmp/claude-home");

        assert_eq!(resolved.binary_path, PathBuf::from("/tmp/fake-claude"));
        assert_eq!(resolved.working_dir, Some(PathBuf::from("request/workdir")));
        assert_eq!(resolved.timeout, Some(Duration::from_secs(5)));
        assert_eq!(
            resolved.env.get("CONFIG_ONLY").map(String::as_str),
            Some("config-only")
        );
        assert_eq!(
            resolved.env.get("OVERRIDE_ME").map(String::as_str),
            Some("request")
        );
        assert_eq!(
            resolved.env.get("REQUEST_ONLY").map(String::as_str),
            Some("request-only")
        );
        assert_eq!(
            resolved
                .env
                .get(DISABLE_AUTOUPDATER_ENV)
                .map(String::as_str),
            Some("1")
        );
        assert_eq!(
            resolved.env.get(CLAUDE_HOME_ENV).map(String::as_str),
            Some("/tmp/claude-home")
        );
        assert_eq!(
            resolved.env.get(HOME_ENV).map(String::as_str),
            Some("/tmp/claude-home")
        );
        assert_eq!(
            resolved.env.get(XDG_CONFIG_HOME_ENV).map(String::as_str),
            Some(layout.xdg_config_home().to_string_lossy().as_ref())
        );
        assert_eq!(
            resolved.env.get(XDG_DATA_HOME_ENV).map(String::as_str),
            Some(layout.xdg_data_home().to_string_lossy().as_ref())
        );
        assert_eq!(
            resolved.env.get(XDG_CACHE_HOME_ENV).map(String::as_str),
            Some(layout.xdg_cache_home().to_string_lossy().as_ref())
        );
        assert_eq!(resolved.materialize_claude_home, Some(layout));
    }

    #[test]
    fn resolve_claude_mcp_command_uses_backend_defaults_when_request_values_absent() {
        let resolved = resolve_claude_mcp_command_with_env(
            &sample_config(),
            &AgentWrapperMcpCommandContext::default(),
            None,
        );

        assert_eq!(resolved.working_dir, Some(PathBuf::from("default/workdir")));
        assert_eq!(resolved.timeout, Some(Duration::from_secs(30)));
    }

    #[test]
    fn disable_autoupdater_default_does_not_override_explicit_values() {
        let mut config = sample_config();
        config
            .env
            .insert(DISABLE_AUTOUPDATER_ENV.to_string(), "0".to_string());
        let resolved = resolve_claude_mcp_command_with_env(
            &config,
            &AgentWrapperMcpCommandContext::default(),
            None,
        );
        assert_eq!(
            resolved
                .env
                .get(DISABLE_AUTOUPDATER_ENV)
                .map(String::as_str),
            Some("0")
        );

        let mut context = AgentWrapperMcpCommandContext::default();
        context
            .env
            .insert(DISABLE_AUTOUPDATER_ENV.to_string(), "2".to_string());
        let resolved = resolve_claude_mcp_command_with_env(&config, &context, None);
        assert_eq!(
            resolved
                .env
                .get(DISABLE_AUTOUPDATER_ENV)
                .map(String::as_str),
            Some("2")
        );
    }

    #[test]
    fn request_env_overrides_injected_home_keys() {
        let mut context = AgentWrapperMcpCommandContext::default();
        context
            .env
            .insert(HOME_ENV.to_string(), "/tmp/request-home".to_string());
        context.env.insert(
            XDG_CONFIG_HOME_ENV.to_string(),
            "/tmp/request-xdg-config".to_string(),
        );

        let resolved = resolve_claude_mcp_command_with_env(&sample_config(), &context, None);

        assert_eq!(
            resolved.env.get(HOME_ENV).map(String::as_str),
            Some("/tmp/request-home")
        );
        assert_eq!(
            resolved.env.get(XDG_CONFIG_HOME_ENV).map(String::as_str),
            Some("/tmp/request-xdg-config")
        );
        assert_eq!(
            resolved.env.get(CLAUDE_HOME_ENV).map(String::as_str),
            Some("/tmp/claude-home")
        );
        assert_eq!(
            resolved.materialize_claude_home,
            Some(ClaudeHomeLayout::new("/tmp/claude-home"))
        );
    }

    #[test]
    fn zero_timeout_is_treated_as_no_timeout() {
        assert_eq!(effective_timeout_for_wait(Some(Duration::ZERO)), None);
        assert_eq!(
            effective_timeout_for_wait(Some(Duration::from_secs(3))),
            Some(Duration::from_secs(3))
        );
    }

    #[tokio::test]
    async fn capture_bounded_preserves_small_streams() {
        let (writer, reader) = duplex(64);
        let writer_task = tokio::spawn(write_all_and_close(writer, b"hello".to_vec()));

        let (captured, saw_more) = capture_bounded(reader, 8).await.expect("capture succeeds");
        writer_task.await.expect("writer completes");

        assert_eq!(captured, b"hello");
        assert!(!saw_more);
    }

    #[tokio::test]
    async fn capture_bounded_retains_only_bound_and_marks_overflow() {
        let (writer, reader) = duplex(64);
        let writer_task = tokio::spawn(write_all_and_close(
            writer,
            b"abcdefghijklmnopqrstuvwxyz".to_vec(),
        ));

        let (captured, saw_more) = capture_bounded(reader, 8).await.expect("capture succeeds");
        writer_task.await.expect("writer completes");

        assert_eq!(captured, b"abcdefgh");
        assert!(saw_more);
    }

    #[tokio::test]
    async fn capture_bounded_with_zero_bound_drains_input_and_reports_overflow() {
        let (writer, reader) = duplex(64);
        let writer_task = tokio::spawn(write_all_and_close(writer, b"abcdef".to_vec()));

        let (captured, saw_more) = capture_bounded(reader, 0).await.expect("capture succeeds");
        writer_task.await.expect("writer completes");

        assert!(captured.is_empty());
        assert!(saw_more);
    }

    #[test]
    fn classify_manifest_runtime_conflict_detects_unknown_mcp_command() {
        assert!(classify_manifest_runtime_conflict_text(
            &claude_mcp_list_argv(),
            "error: unrecognized subcommand 'mcp'"
        ));
    }

    #[test]
    fn classify_manifest_runtime_conflict_detects_unknown_get_subcommand() {
        assert!(classify_manifest_runtime_conflict_text(
            &claude_mcp_get_argv("demo"),
            "error: no such subcommand 'get'"
        ));
    }

    #[test]
    fn classify_manifest_runtime_conflict_detects_unknown_add_subcommand() {
        let transport = AgentWrapperMcpAddTransport::Stdio {
            command: vec!["node".to_string()],
            args: vec!["server.js".to_string()],
            env: BTreeMap::new(),
        };
        let argv = claude_mcp_add_argv("demo", &transport).expect("stdio transport should map");

        assert!(classify_manifest_runtime_conflict_text(
            &argv,
            "error: unknown subcommand 'add'"
        ));
    }

    #[test]
    fn classify_manifest_runtime_conflict_detects_unknown_list_subcommand() {
        assert!(classify_manifest_runtime_conflict_text(
            &claude_mcp_list_argv(),
            "error: unknown subcommand 'list'"
        ));
    }

    #[test]
    fn classify_manifest_runtime_conflict_detects_unknown_remove_subcommand() {
        assert!(classify_manifest_runtime_conflict_text(
            &claude_mcp_remove_argv("demo"),
            "error: unrecognized subcommand 'remove'"
        ));
    }

    #[test]
    fn classify_manifest_runtime_conflict_ignores_domain_failures() {
        assert!(!classify_manifest_runtime_conflict_text(
            &claude_mcp_get_argv("demo"),
            "server demo not found"
        ));
        assert!(!classify_manifest_runtime_conflict_text(
            &claude_mcp_get_argv("demo"),
            "unknown server demo"
        ));
        assert!(!classify_manifest_runtime_conflict_text(
            &claude_mcp_get_argv("demo"),
            "permission denied while contacting remote service"
        ));
        assert!(!classify_manifest_runtime_conflict_text(
            &claude_mcp_get_argv("demo"),
            "network error: failed to connect"
        ));
    }

    #[test]
    fn finalize_claude_mcp_output_returns_backend_error_for_drift() {
        let err = finalize_claude_mcp_output(
            &claude_mcp_get_argv("demo"),
            CapturedClaudeMcpCommandOutput {
                status: exit_status_with_code(2),
                stdout_bytes: b"raw stdout should not leak".to_vec(),
                stdout_saw_more: false,
                stderr_bytes: b"error: no such subcommand 'get'".to_vec(),
                stderr_saw_more: false,
            },
        )
        .expect_err("drift should fail closed");

        match err {
            AgentWrapperError::Backend { message } => {
                assert_eq!(message, PINNED_MCP_RUNTIME_CONFLICT);
            }
            other => panic!("expected Backend error, got {other:?}"),
        }
    }

    #[test]
    fn finalize_claude_mcp_output_keeps_normal_non_zero_exits_as_ok() {
        let output = finalize_claude_mcp_output(
            &claude_mcp_get_argv("demo"),
            CapturedClaudeMcpCommandOutput {
                status: exit_status_with_code(3),
                stdout_bytes: b"listed output".to_vec(),
                stdout_saw_more: false,
                stderr_bytes: b"server demo not found".to_vec(),
                stderr_saw_more: false,
            },
        )
        .expect("normal failures should remain Ok(output)");

        assert_eq!(output.status, exit_status_with_code(3));
        assert_eq!(output.stdout, "listed output");
        assert_eq!(output.stderr, "server demo not found");
        assert!(!output.stdout_truncated);
        assert!(!output.stderr_truncated);
    }

    #[test]
    fn finalize_claude_mcp_output_detects_drift_in_stdout_too() {
        let err = finalize_claude_mcp_output(
            &claude_mcp_list_argv(),
            CapturedClaudeMcpCommandOutput {
                status: exit_status_with_code(4),
                stdout_bytes: b"error: unknown subcommand 'list'".to_vec(),
                stdout_saw_more: false,
                stderr_bytes: Vec::new(),
                stderr_saw_more: false,
            },
        )
        .expect_err("stdout drift should fail closed");

        match err {
            AgentWrapperError::Backend { message } => {
                assert_eq!(message, PINNED_MCP_RUNTIME_CONFLICT);
            }
            other => panic!("expected Backend error, got {other:?}"),
        }
    }

    #[test]
    fn success_exit_skips_drift_classification() {
        let output = finalize_claude_mcp_output(
            &claude_mcp_get_argv("demo"),
            CapturedClaudeMcpCommandOutput {
                status: success_exit_status(),
                stdout_bytes: b"error: no such subcommand 'get'".to_vec(),
                stdout_saw_more: false,
                stderr_bytes: Vec::new(),
                stderr_saw_more: false,
            },
        )
        .expect("successful exits should remain Ok(output)");

        assert_eq!(output.status, success_exit_status());
        assert_eq!(output.stdout, "error: no such subcommand 'get'");
        assert!(output.stderr.is_empty());
    }
}
