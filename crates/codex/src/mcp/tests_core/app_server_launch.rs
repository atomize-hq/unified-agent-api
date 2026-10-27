use super::super::test_support::{prelude::*, *};
use super::super::*;

#[test]
fn app_pool_fixtures_preserve_python_lookup_under_hostile_ambient_path() {
    // Only the spawned test process gets the hostile PATH; the parallel parent suite is intact.
    // These two cases exercise initial starts, reuse, stop/stop_all, and subsequent restarts.
    let output = std::process::Command::new(std::env::current_exe().expect("test executable"))
        .args([
            "mcp::tests_runtime_app::app_runtime_pool_api::",
            "--nocapture",
        ])
        .env("PATH", "/no/python/on/ambient/path")
        .output()
        .expect("run isolated pool fixture tests");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "{stdout}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        stdout.contains("2 passed"),
        "pool tests must actually run: {stdout}"
    );
}

#[tokio::test]
async fn app_server_launch_can_enable_analytics_flag() {
    let (dir, script) = write_fake_app_server();
    let log_path = dir.path().join("argv.json");

    let mut config = test_config(script);
    config.app_server_analytics_default_enabled = true;
    config.env.push((
        OsString::from("ARGV_LOG"),
        OsString::from(log_path.as_os_str()),
    ));

    let client = test_client();
    let server = CodexAppServer::start(config, client)
        .await
        .expect("spawn app server");

    let mut argv_line = None;
    for _ in 0..50 {
        if let Ok(contents) = fs::read_to_string(&log_path) {
            argv_line = contents.lines().next().map(str::to_string);
            break;
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    }

    let argv_line = argv_line.expect("argv log should be written");
    let argv: Vec<String> = serde_json::from_str(&argv_line).expect("argv json");
    assert_eq!(argv, vec!["app-server", "--analytics-default-enabled"]);

    server.shutdown().await.expect("shutdown server");
}
