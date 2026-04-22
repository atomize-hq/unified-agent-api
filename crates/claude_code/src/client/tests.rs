use std::process::Stdio;
use std::time::Duration;

use super::{wait_for_child_exit, ChildExit};

#[cfg(unix)]
#[tokio::test]
async fn wait_for_child_exit_returns_status_when_deadline_has_elapsed() {
    let mut child = tokio::process::Command::new("sh")
        .args(["-c", "exit 0"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn child");
    tokio::time::sleep(Duration::from_millis(50)).await;

    let outcome = wait_for_child_exit(
        &mut child,
        Some(Duration::from_millis(1)),
        Some(tokio::time::Instant::now()),
    )
    .await
    .expect("wait helper succeeds");

    match outcome {
        ChildExit::Exited(status) => assert!(status.success()),
        ChildExit::TimedOut => panic!("expected exited status, got timeout"),
    }
}
