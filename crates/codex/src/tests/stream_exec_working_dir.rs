use super::*;

use std::{env, path::Path, path::PathBuf, time::Duration};

struct RestoreCurrentDir {
    original: PathBuf,
}

impl RestoreCurrentDir {
    fn capture() -> Self {
        Self {
            original: env::current_dir().unwrap(),
        }
    }
}

impl Drop for RestoreCurrentDir {
    fn drop(&mut self) {
        env::set_current_dir(&self.original).unwrap();
    }
}

fn write_relative_probe_sensitive_codex(root: &Path, log_path: &Path) {
    let bin_dir = root.join("bin");
    std_fs::create_dir_all(&bin_dir).unwrap();
    write_fake_codex(
        &bin_dir,
        &format!(
            r#"#!/usr/bin/env bash
set -euo pipefail

log="{log}"
if [[ "$1" == "--version" ]]; then
  echo "codex 1.2.3"
elif [[ "$1" == "features" && "$2" == "list" && "$3" == "--json" ]]; then
  echo '{{"features":["add_dir"]}}'
elif [[ "$1" == "features" && "$2" == "list" ]]; then
  echo "add_dir"
elif [[ "$1" == "--help" ]]; then
  echo "Usage: codex --add-dir"
elif [[ "$1" == "exec" ]]; then
  echo "$@" >> "$log"
  has_add_dir=0
  out=""
  for ((i=1; i<=$#; i++)); do
    arg="${{!i}}"
    if [[ "$arg" == "--add-dir" ]]; then
      has_add_dir=1
    elif [[ "$arg" == "--output-last-message" ]]; then
      j=$((i+1))
      out="${{!j}}"
    fi
  done

  if [[ "$has_add_dir" -ne 1 ]]; then
    echo "missing --add-dir" >&2
    exit 9
  fi

  if [[ -n "$out" ]]; then
    mkdir -p "$(dirname "$out")"
    printf 'final message\n' > "$out"
    echo '{{"type":"thread.started","thread_id":"thread-1"}}'
  else
    echo "ok"
  fi
else
  echo "unexpected args: $*" >&2
  exit 10
fi
"#,
            log = log_path.display()
        ),
    );
}

fn relative_binary_client(working_dir: &Path) -> CodexClient {
    CodexClient::builder()
        .binary("./bin/codex")
        .working_dir(working_dir)
        .timeout(Duration::from_secs(5))
        .add_dir("src")
        .mirror_stdout(false)
        .quiet(true)
        .build()
}

#[cfg(unix)]
#[tokio::test]
async fn send_prompt_probes_relative_binary_from_effective_working_dir() {
    let _guard = env_guard_async().await;
    clear_capability_cache();

    let ambient = tempfile::tempdir().unwrap();
    let working = tempfile::tempdir().unwrap();
    let _restore_cwd = RestoreCurrentDir::capture();
    env::set_current_dir(ambient.path()).unwrap();

    let log_path = working.path().join("exec.log");
    write_relative_probe_sensitive_codex(working.path(), &log_path);

    let client = relative_binary_client(working.path());
    let response = client.send_prompt("hello").await.unwrap();
    assert_eq!(response.trim(), "ok");

    let logged = std_fs::read_to_string(&log_path).unwrap();
    assert!(logged.contains("--add-dir"));
    assert!(logged.contains("src"));
}

#[cfg(unix)]
#[tokio::test]
async fn stream_exec_probes_relative_binary_from_effective_working_dir() {
    let _guard = env_guard_async().await;
    clear_capability_cache();

    let ambient = tempfile::tempdir().unwrap();
    let working = tempfile::tempdir().unwrap();
    let _restore_cwd = RestoreCurrentDir::capture();
    env::set_current_dir(ambient.path()).unwrap();

    let log_path = working.path().join("stream-exec.log");
    write_relative_probe_sensitive_codex(working.path(), &log_path);

    let client = relative_binary_client(working.path());
    let stream = client
        .stream_exec(ExecStreamRequest {
            prompt: "hello".to_string(),
            idle_timeout: None,
            output_last_message: None,
            output_schema: None,
            json_event_log: None,
        })
        .await
        .unwrap();

    let completion = stream.completion.await.unwrap();
    assert_eq!(completion.last_message.as_deref(), Some("final message\n"));

    let logged = std_fs::read_to_string(&log_path).unwrap();
    assert!(logged.contains("--add-dir"));
    assert!(logged.contains("src"));
}

#[cfg(unix)]
#[tokio::test]
async fn stream_resume_probes_relative_binary_from_effective_working_dir() {
    let _guard = env_guard_async().await;
    clear_capability_cache();

    let ambient = tempfile::tempdir().unwrap();
    let working = tempfile::tempdir().unwrap();
    let _restore_cwd = RestoreCurrentDir::capture();
    env::set_current_dir(ambient.path()).unwrap();

    let log_path = working.path().join("stream-resume.log");
    write_relative_probe_sensitive_codex(working.path(), &log_path);

    let client = relative_binary_client(working.path());
    let stream = client
        .stream_resume(ResumeRequest::last().prompt("hello"))
        .await
        .unwrap();

    let completion = stream.completion.await.unwrap();
    assert_eq!(completion.last_message.as_deref(), Some("final message\n"));

    let logged = std_fs::read_to_string(&log_path).unwrap();
    assert!(logged.contains("--add-dir"));
    assert!(logged.contains("src"));
    assert!(logged.contains("resume"));
}
