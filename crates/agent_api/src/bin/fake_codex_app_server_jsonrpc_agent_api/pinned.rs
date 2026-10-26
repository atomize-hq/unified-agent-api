//! Pinned 0.156.1 protocol fixture: source b412ff32c417f855c2b2d1581b77058eed87c84b.
//! This exercises the production client; it is not a real model run.
use super::*;

pub(super) fn run(scenario: &str) -> io::Result<()> {
    let stdin = io::stdin();
    let mut out = io::stdout().lock();
    let mut page = 0;
    let mut approval = false;
    #[cfg(unix)]
    let mut exit_gate = None;
    for line in stdin.lock().lines() {
        let message: Value = serde_json::from_str(&line?)?;
        let method = message["method"].as_str().unwrap_or_default();
        let id = &message["id"];
        let params = &message["params"];
        maybe_log_request_method(method);
        #[cfg(unix)]
        if env::var("FAKE_CODEX_PAUSE_AT").ok().as_deref() == Some(method) {
            let mut stream =
                std::os::unix::net::UnixStream::connect(env::var("FAKE_CODEX_GATE").unwrap())?;
            writeln!(stream, "stage {}", std::process::id())?;
            exit_gate = Some(stream);
            continue; // Keep reading stdin, deliberately leave this setup RPC unanswered.
        }
        if method.is_empty() {
            assert!(approval, "unexpected response");
            assert_eq!(id, "approval-1");
            assert!(
                message.get("error").is_some(),
                "must never authorize approval"
            );
            continue;
        }
        if matches!(method, "thread/fork" | "turn/start") {
            if let Ok(expected) = env::var("FAKE_CODEX_APP_SERVER_EXPECT_POLICY") {
                assert_eq!(
                    params.get("approvalPolicy").and_then(Value::as_str),
                    if expected == "absent" {
                        None
                    } else {
                        Some(expected.as_str())
                    }
                );
            }
        }
        match method {
            "initialize" => write_result(&mut out, id, json!({"userAgent":"codex/0.156.1"}))?,
            "initialized" => {}
            "thread/list" => {
                assert_eq!(
                    params["sourceKinds"],
                    json!([
                        "cli",
                        "vscode",
                        "exec",
                        "appServer",
                        "subAgent",
                        "subAgentReview",
                        "subAgentCompact",
                        "subAgentThreadSpawn",
                        "subAgentOther",
                        "unknown"
                    ])
                );
                assert_eq!(params["modelProviders"], json!([]));
                assert_eq!(params["archived"], false);
                assert_eq!(params["sortKey"], "updated_at");
                assert_eq!(params["limit"], 100);
                assert_eq!(
                    params["cwd"].as_str(),
                    env::var("FAKE_CODEX_APP_SERVER_EXPECT_CWD").ok().as_deref()
                );
                page += 1;
                if scenario == "pinned_empty" {
                    write_result(&mut out, id, json!({"data":[],"nextCursor":null}))?;
                } else {
                    assert_eq!(
                        params["cursor"],
                        if page == 1 {
                            Value::Null
                        } else {
                            json!("page-2")
                        }
                    );
                    write_result(
                        &mut out,
                        id,
                        json!({"data":[{"id":if page == 1 {"a"} else {"z"},"createdAt":1,"updatedAt":2,"cwd":params["cwd"]}],"nextCursor":if page == 1 || scenario == "pinned_cursor" {json!("page-2")} else {Value::Null}}),
                    )?;
                }
            }
            "thread/fork" => {
                if params["approvalPolicy"] == "on-failure" {
                    write_error(&mut out, id, -32600, "unknown variant SECRET", None)?;
                    continue;
                }
                if scenario == "pinned_last" {
                    assert_eq!(params["threadId"], "z");
                }
                if scenario == "pinned_fork_approval" {
                    approval = true;
                    write_json(
                        &mut out,
                        &json!({"id":"approval-1","method":"item/fileChange/requestApproval","params":{"threadId":"forked-1","turnId":"turn-1","itemId":"item-1","startedAtMs":0,"reason":"SECRET"}}),
                    )?;
                    continue;
                }
                write_result(&mut out, id, json!({"thread":{"id":"forked-1"}}))?;
            }
            "turn/start" => {
                assert_eq!(params["threadId"], "forked-1");
                assert_eq!(
                    params["input"],
                    json!([{"type":"text","text":"hello","text_elements":[]}])
                );
                if scenario == "pinned_wrong_response" {
                    write_result(
                        &mut out,
                        &json!(9999),
                        json!({"turn":{"id":"turn-1","status":"inProgress","items":[]}}),
                    )?;
                    continue;
                }
                if scenario == "pinned_malformed_acceptance" {
                    write_result(&mut out, id, json!({"turnId":"turn-1"}))?;
                    continue;
                }
                write_result(
                    &mut out,
                    id,
                    json!({"turn":{"id":"turn-1","status":"inProgress","items":[],"error":null}}),
                )?;
                notify(
                    &mut out,
                    "turn/started",
                    json!({"threadId":"forked-1","turn":{"id":"turn-1","status":"inProgress","items":[],"error":null}}),
                )?;
                if scenario == "pinned_approval" {
                    approval = true;
                    write_json(
                        &mut out,
                        &json!({"id":"approval-1","method":"item/commandExecution/requestApproval","params":{"threadId":"forked-1","turnId":"turn-1","itemId":"item-1","startedAtMs":0,"command":"SECRET"}}),
                    )?;
                    continue;
                }
                if scenario == "pinned_disconnect" {
                    if let Ok(path) = env::var("FAKE_CODEX_APP_SERVER_EXIT_MARKER") {
                        std::fs::write(path, "disconnected")?;
                    }
                    return Ok(());
                }
                if scenario == "pinned_error" {
                    notify(
                        &mut out,
                        "error",
                        json!({"threadId":"forked-1","turnId":"turn-1","willRetry":false,"error":{"message":"SECRET","codexErrorInfo":null,"additionalDetails":null}}),
                    )?;
                    continue;
                }
                if scenario == "pinned_malformed_terminal" {
                    notify(
                        &mut out,
                        "turn/completed",
                        json!({"threadId":"forked-1","turn":{"id":"turn-1","status":"completed"}}),
                    )?;
                    continue;
                }
                if scenario == "pinned_retry_secret" {
                    notify(
                        &mut out,
                        "error",
                        json!({"threadId":"forked-1","turnId":"turn-1","willRetry":true,"error":{"message":"CREDENTIAL-SECRET-RETRY","codexErrorInfo":null,"additionalDetails":"CREDENTIAL-SECRET-DETAIL"}}),
                    )?;
                }
                if scenario.starts_with("pinned_terminal_") {
                    let mut turn = json!({"status":"completed","items":[],"error":null});
                    match scenario {
                        "pinned_terminal_blank" => turn["id"] = json!(" "),
                        "pinned_terminal_conflict" => turn["id"] = json!("other"),
                        "pinned_terminal_absent" => {}
                        _ => unreachable!(),
                    }
                    notify(
                        &mut out,
                        "turn/completed",
                        json!({"threadId":"forked-1","turnId":"turn-1","turn":turn}),
                    )?;
                    continue;
                }
                if scenario == "pinned_block" {
                    continue;
                }
                notify(
                    &mut out,
                    "future/unknown",
                    json!({"threadId":"forked-1","turnId":"turn-1","SECRET":"ignored"}),
                )?;
                notify(
                    &mut out,
                    "item/agentMessage/delta",
                    json!({"threadId":"forked-1","turnId":"turn-1","delta":42}),
                )?;
                // Wrong-thread and wrong-turn completion must not complete this run.
                complete(&mut out, "other", "turn-1", "completed")?;
                complete(&mut out, "forked-1", "other", "completed")?;
                notify(
                    &mut out,
                    "item/agentMessage/delta",
                    json!({"threadId":"other","turnId":"turn-1","itemId":"message-1","delta":"FOREIGN"}),
                )?;
                notify(
                    &mut out,
                    "item/agentMessage/delta",
                    json!({"threadId":"forked-1","turnId":"turn-1","itemId":"message-1","delta":"final output"}),
                )?;
                notify(
                    &mut out,
                    "item/reasoning/textDelta",
                    json!({"threadId":"forked-1","turnId":"turn-1","itemId":"reason-1","delta":"reason","contentIndex":0}),
                )?;
                notify(
                    &mut out,
                    "item/completed",
                    json!({"threadId":"forked-1","turnId":"turn-1","item":{"id":"message-1","type":"agentMessage","text":"final output"},"completedAtMs":0}),
                )?;
                complete(
                    &mut out,
                    "forked-1",
                    "turn-1",
                    if scenario == "pinned_failed" {
                        "failed"
                    } else {
                        "completed"
                    },
                )?;
            }
            "turn/interrupt" => {
                assert_eq!(params["threadId"], "forked-1");
                assert_eq!(params["turnId"], "turn-1");
                write_result(&mut out, id, json!({}))?;
                complete(&mut out, "forked-1", "turn-1", "interrupted")?;
            }
            _ => panic!("unsupported pinned method: {method}"),
        }
        out.flush()?;
    }
    #[cfg(unix)]
    if let Some(mut stream) = exit_gate {
        use std::io::Read;
        writeln!(stream, "eof")?;
        let mut release = [0];
        stream.read_exact(&mut release)?;
    }
    if let Ok(path) = env::var("FAKE_CODEX_APP_SERVER_EXIT_MARKER") {
        std::fs::write(path, "reaped after EOF")?;
    }
    Ok(())
}

fn notify(out: &mut impl Write, method: &str, params: Value) -> io::Result<()> {
    write_json(out, &json!({"method":method,"params":params}))
}

fn complete(out: &mut impl Write, thread: &str, turn: &str, status: &str) -> io::Result<()> {
    notify(
        out,
        "turn/completed",
        json!({"threadId":thread,"turn":{"id":turn,"status":status,"items":[],"error":if status == "failed" {json!({"message":"SECRET","codexErrorInfo":null,"additionalDetails":null})} else {Value::Null}}}),
    )
}
