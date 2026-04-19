#[cfg(unix)]
mod unix {
    use std::{fs, process::Command, time::Duration};

    use futures_util::StreamExt;
    use opencode::{OpencodeClient, OpencodeError, OpencodeRunJsonEvent, OpencodeRunRequest};
    use tempfile::TempDir;
    use tokio::time;

    const STEP_START: &str = include_str!("fixtures/run_json/v1/step_start.jsonl");
    const TEXT: &str = include_str!("fixtures/run_json/v1/text.jsonl");

    fn first_nonempty_line(text: &str) -> &str {
        text.lines()
            .find(|line| !line.chars().all(|ch| ch.is_whitespace()))
            .expect("fixture contains a non-empty line")
    }

    fn pid_exists(pid: i32) -> bool {
        Command::new("kill")
            .arg("-0")
            .arg(pid.to_string())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }

    async fn assert_pid_gone(pid: i32) {
        let deadline = time::Instant::now() + Duration::from_millis(500);
        loop {
            if !pid_exists(pid) {
                return;
            }

            if time::Instant::now() >= deadline {
                panic!("expected pid {pid} to be gone, but it still exists");
            }

            time::sleep(Duration::from_millis(25)).await;
        }
    }

    #[tokio::test]
    async fn run_json_timeout_after_stdout_eof_reaps_child() {
        use std::os::unix::fs::PermissionsExt;

        let dir = TempDir::new().expect("temp dir");
        let pid_file = dir.path().join("pid.txt");
        let script_path = dir.path().join("fake-opencode");
        let step_start = first_nonempty_line(STEP_START);
        let text = first_nonempty_line(TEXT);

        let script = format!(
            r#"#!/bin/sh
set -eu
: "${{PID_FILE:?missing PID_FILE}}"
echo $$ > "$PID_FILE"
printf '%s\n' '{step_start}'
printf '%s\n' '{text}'
exec 1>&-
exec sleep 1000000
"#
        );

        fs::write(&script_path, script).expect("write script");
        let mut perms = fs::metadata(&script_path).expect("metadata").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).expect("chmod");

        let client = OpencodeClient::builder()
            .binary(&script_path)
            .env("PID_FILE", pid_file.to_string_lossy().to_string())
            .timeout(Duration::from_millis(300))
            .build();

        let handle = client
            .run_json(OpencodeRunRequest::new("Reply with OK."))
            .await
            .expect("spawn run-json handle");
        let mut events = handle.events;

        let first = tokio::time::timeout(Duration::from_secs(1), events.next())
            .await
            .expect("first event timeout")
            .expect("stream open")
            .expect("event parses");
        assert!(matches!(first, OpencodeRunJsonEvent::StepStart { .. }));

        let second = tokio::time::timeout(Duration::from_secs(1), events.next())
            .await
            .expect("second event timeout")
            .expect("stream open")
            .expect("event parses");
        match second {
            OpencodeRunJsonEvent::Text { text, .. } => assert_eq!(text, "OK"),
            other => panic!("expected text event, got {other:?}"),
        }

        tokio::time::timeout(Duration::from_secs(1), async {
            while events.next().await.is_some() {}
        })
        .await
        .expect("expected event stream to close after stdout EOF");

        let err = tokio::time::timeout(Duration::from_secs(2), handle.completion)
            .await
            .expect("completion should resolve after timeout triggers")
            .expect_err("completion should surface timeout");
        assert!(
            matches!(err, OpencodeError::Timeout { .. }),
            "expected timeout, got: {err:?}"
        );

        let pid: i32 = {
            let deadline = time::Instant::now() + Duration::from_secs(1);
            loop {
                match fs::read_to_string(&pid_file) {
                    Ok(contents) => break contents,
                    Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                        if time::Instant::now() >= deadline {
                            panic!("pid file was not created before timeout");
                        }
                        time::sleep(Duration::from_millis(25)).await;
                    }
                    Err(err) => panic!("failed to read pid file: {err}"),
                }
            }
        }
        .trim()
        .parse()
        .expect("pid parse");

        assert_pid_gone(pid).await;
    }
}
