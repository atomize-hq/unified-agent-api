use std::collections::BTreeMap;

use crate::{mcp::AgentWrapperMcpAddTransport, AgentWrapperError};

use super::super::{
    claude_mcp_add_argv, claude_mcp_get_argv, claude_mcp_list_argv, claude_mcp_remove_argv,
    runner::{
        capture_bounded, classify_manifest_runtime_conflict_text, finalize_claude_mcp_output,
        CapturedClaudeMcpCommandOutput,
    },
    PINNED_MCP_RUNTIME_CONFLICT,
};
use super::support::{exit_status_with_code, success_exit_status, write_all_and_close};

fn sample_stdio_transport() -> AgentWrapperMcpAddTransport {
    AgentWrapperMcpAddTransport::Stdio {
        command: vec!["node".to_string()],
        args: vec!["server.js".to_string()],
        env: BTreeMap::new(),
    }
}

fn assert_runtime_conflict(err: AgentWrapperError) {
    match err {
        AgentWrapperError::Backend { message } => {
            assert_eq!(message, PINNED_MCP_RUNTIME_CONFLICT);
        }
        other => panic!("expected Backend error, got {other:?}"),
    }
}

#[tokio::test]
async fn capture_bounded_preserves_small_streams() {
    let (writer, reader) = tokio::io::duplex(64);
    let writer_task = tokio::spawn(write_all_and_close(writer, b"hello".to_vec()));

    let (captured, saw_more) = capture_bounded(reader, 8).await.expect("capture succeeds");
    writer_task.await.expect("writer completes");

    assert_eq!(captured, b"hello");
    assert!(!saw_more);
}

#[tokio::test]
async fn capture_bounded_retains_only_bound_and_marks_overflow() {
    let (writer, reader) = tokio::io::duplex(64);
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
    let (writer, reader) = tokio::io::duplex(64);
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
    let argv =
        claude_mcp_add_argv("demo", &sample_stdio_transport()).expect("stdio transport should map");

    assert!(classify_manifest_runtime_conflict_text(
        &argv,
        "error: unknown subcommand 'add'"
    ));
}

#[test]
fn classify_manifest_runtime_conflict_detects_add_transport_flag_drift_without_echoed_add() {
    let argv =
        claude_mcp_add_argv("demo", &sample_stdio_transport()).expect("stdio transport should map");

    assert!(classify_manifest_runtime_conflict_text(
        &argv,
        "error: unexpected argument '--transport' found"
    ));
}

#[test]
fn classify_manifest_runtime_conflict_detects_add_env_flag_usage_drift() {
    let transport = AgentWrapperMcpAddTransport::Stdio {
        command: vec!["node".to_string()],
        args: vec!["server.js".to_string()],
        env: BTreeMap::from([("ALPHA_ENV".to_string(), "1".to_string())]),
    };
    let argv = claude_mcp_add_argv("demo", &transport).expect("stdio transport should map");

    assert!(classify_manifest_runtime_conflict_text(
        &argv,
        "error: unexpected argument '--env' found\n\nusage: claude mcp add [options]"
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

    let argv =
        claude_mcp_add_argv("demo", &sample_stdio_transport()).expect("stdio transport should map");
    assert!(!classify_manifest_runtime_conflict_text(
        &argv,
        "error: unexpected argument '--foo' found"
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

    assert_runtime_conflict(err);
}

#[test]
fn finalize_claude_mcp_output_returns_backend_error_for_add_flag_drift() {
    let argv =
        claude_mcp_add_argv("demo", &sample_stdio_transport()).expect("stdio transport should map");

    let err = finalize_claude_mcp_output(
        &argv,
        CapturedClaudeMcpCommandOutput {
            status: exit_status_with_code(2),
            stdout_bytes: b"raw stdout should not leak".to_vec(),
            stdout_saw_more: false,
            stderr_bytes: b"error: unexpected argument '--transport' found".to_vec(),
            stderr_saw_more: false,
        },
    )
    .expect_err("add flag drift should fail closed");

    assert_runtime_conflict(err);
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

    assert_runtime_conflict(err);
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
