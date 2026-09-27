#[cfg(unix)]
mod unix {
    use std::fs;

    use claude_code::{
        ClaudeClient, ClaudeCommandRequest, ClaudeNonTuiCommand, ClaudeNonTuiCommandRequest,
    };
    use tempfile::TempDir;

    #[tokio::test]
    async fn forwards_packet_command_global_and_command_arguments_to_stub() {
        use std::os::unix::fs::PermissionsExt;

        let dir = TempDir::new().expect("temp dir");
        let output = dir.path().join("argv");
        let script_path = dir.path().join("fake-claude");
        fs::write(
            &script_path,
            format!("#!/bin/sh\nprintf '%s\\n' \"$@\" > {}\n", output.display()),
        )
        .expect("write stub");
        let mut permissions = fs::metadata(&script_path).expect("metadata").permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&script_path, permissions).expect("chmod stub");

        let client = ClaudeClient::builder().binary(&script_path).build();
        let result = client
            .run_non_tui_command(
                ClaudeNonTuiCommandRequest::new(ClaudeNonTuiCommand::McpLogin)
                    .global_arg("--worktree")
                    .args(["--no-browser", "packet-server"]),
            )
            .await
            .expect("run stub");
        assert!(result.status.success());
        assert_eq!(
            fs::read_to_string(output).expect("read argv"),
            "--worktree\nmcp\nlogin\n--no-browser\npacket-server\n"
        );
    }

    #[tokio::test]
    async fn forwards_existing_mcp_extension_through_the_generic_command_runner() {
        use std::os::unix::fs::PermissionsExt;

        let dir = TempDir::new().expect("temp dir");
        let output = dir.path().join("argv");
        let script_path = dir.path().join("fake-claude");
        fs::write(
            &script_path,
            format!("#!/bin/sh\nprintf '%s\\n' \"$@\" > {}\n", output.display()),
        )
        .expect("write stub");
        let mut permissions = fs::metadata(&script_path).expect("metadata").permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&script_path, permissions).expect("chmod stub");

        let client = ClaudeClient::builder().binary(&script_path).build();
        let result = client
            .run_command(ClaudeCommandRequest::root().args([
                "--worktree",
                "mcp",
                "add",
                "--client-secret",
                "packet-secret",
                "packet-server",
            ]))
            .await
            .expect("run stub");
        assert!(result.status.success());
        assert_eq!(
            fs::read_to_string(output).expect("read argv"),
            "--worktree\nmcp\nadd\n--client-secret\npacket-secret\npacket-server\n"
        );
    }

    #[test]
    fn each_admitted_command_keeps_its_fixed_path_before_forwarded_args() {
        for command in ClaudeNonTuiCommand::all() {
            let argv = ClaudeNonTuiCommandRequest::new(*command)
                .arg("packet-argument")
                .into_command()
                .argv();
            assert_eq!(
                argv[..command.path().len()]
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>(),
                command.path(),
                "fixed path must lead argv for {command:?}"
            );
            assert_eq!(argv.last().map(String::as_str), Some("packet-argument"));
        }
    }
}
