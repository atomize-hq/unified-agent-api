//! Pinned native protocol fixture. No model, network or credentials are used.
use serde_json::{json, Value};
use std::{
    env,
    io::{self, BufRead, Read, Write},
};
fn emit(out: &mut impl Write, value: Value) {
    writeln!(out, "{value}").unwrap();
    out.flush().unwrap();
}
fn context(id: &str) -> Value {
    json!({"id":id,"type":"fileChange","status":"inProgress","changes":[{"path":"秘密/é.txt","kind":{"type":"update","move_path":"新.txt"},"diff":"@@ -1 +1 @@\n-old\n+新\n"},{"path":"second","kind":{"type":"add"},"diff":"+line\n"}]})
}
fn notification(out: &mut impl Write, method: &str, params: Value) {
    emit(out, json!({"method":method,"params":params}));
}
fn complete(out: &mut impl Write) {
    notification(
        out,
        "turn/completed",
        json!({"threadId":"owned","turn":{"id":"turn","status":"completed","items":[]}}),
    );
}
fn request(out: &mut impl Write, id: Value, method: &str, params: Value) {
    emit(out, json!({"id":id,"method":method,"params":params}));
}
fn mcp(scenario: &str) -> Value {
    let mut params = json!({"threadId":"owned","serverName":"example","mode":"url","message":"Native question","elicitationId":"e","url":"https://example.invalid/approve"});
    if scenario.contains("null") {
        params["turnId"] = Value::Null;
    }
    if scenario.contains("scoped") {
        params["turnId"] = json!("turn");
    }
    if scenario.contains("wrong") {
        params["threadId"] = json!("other");
    }
    params
}
fn main() {
    assert!(
        env::args().any(|arg| arg == "app-server"),
        "native route required"
    );
    let scenario = env::var("NATIVE_SCENARIO").unwrap_or_default();
    let mut out = io::stdout().lock();
    let mut turn_ack = None;
    let mut responses = 0;
    let mut input = io::stdin().lock();
    let mut line = String::new();
    while input.read_line(&mut line).unwrap() != 0 {
        let msg: Value = serde_json::from_str(&line).unwrap();
        line.clear();
        let method = msg["method"].as_str().unwrap_or("");
        let id = &msg["id"];
        let params = &msg["params"];
        match method {
            "initialize" => emit(&mut out, json!({"id":id,"result":{"ready":true}})),
            "config/read" => {
                assert_eq!(params["includeLayers"], false);
                if let Ok(expected) = env::var("NATIVE_CWD") {
                    assert_eq!(params["cwd"], expected);
                    assert_eq!(
                        env::current_dir().unwrap().to_str(),
                        Some(expected.as_str())
                    );
                }
                let profile = env::var("NATIVE_PROFILE").ok();
                emit(
                    &mut out,
                    json!({"id":id,"result":{"config":{"default_permissions":profile,"sandbox_workspace_write":{"writable_roots":["/ambient"]}},"origins":{}}}),
                );
            }
            "configRequirements/read" => emit(
                &mut out,
                json!({"id":id,"result":{"requirements":if env::var("NATIVE_ALLOWED_PROFILE").is_ok(){json!({"allowedPermissionProfiles":{"workspace":true}})}else{Value::Null}}}),
            ),
            "thread/list" => emit(
                &mut out,
                json!({"id":id,"result":{"data":[{"id":"source","cwd":params["cwd"],"createdAt":1,"updatedAt":2}],"nextCursor":null}}),
            ),
            "thread/start" | "thread/resume" | "thread/fork" => {
                if method != "thread/start" {
                    assert_eq!(
                        params["threadId"],
                        env::var("NATIVE_SOURCE").unwrap_or("source".into())
                    );
                }
                if let Ok(expected) = env::var("NATIVE_ROOTS") {
                    assert_eq!(
                        params["runtimeWorkspaceRoots"],
                        serde_json::from_str::<Value>(&expected).unwrap()
                    );
                }
                if let Ok(expected) = env::var("NATIVE_SANDBOX") {
                    assert_eq!(params["sandbox"], expected);
                }
                if let Ok(expected) = env::var("NATIVE_EXPECT_HOME") {
                    assert_eq!(env::var("CODEX_HOME").unwrap(), expected);
                    assert!(
                        std::path::Path::new(&expected).is_dir(),
                        "native requires an existing configured home"
                    );
                }
                if let Ok(expected) = env::var("NATIVE_EXPECT_ENV") {
                    assert_eq!(env::var("ENV_PROBE").unwrap(), expected);
                }
                assert!(params.get("approvalsReviewer").is_none());
                if let Ok(path) = env::var("NATIVE_POLICY_LOG") {
                    std::fs::write(path, params["approvalPolicy"].to_string()).unwrap();
                }
                if scenario == "policy_reject" {
                    emit(
                        &mut out,
                        json!({"id":id,"error":{"code":-32600,"message":"unknown variant on-failure SECRET"}}),
                    );
                    continue;
                }
                let expected = env::var("NATIVE_POLICY")
                    .ok()
                    .map(|value| serde_json::from_str::<Value>(&value).unwrap())
                    .unwrap_or(json!("on-request"));
                assert_eq!(
                    params.get("approvalPolicy"),
                    if expected.is_null() {
                        None
                    } else {
                        Some(&expected)
                    }
                );
                if let Ok(model) = env::var("NATIVE_MODEL") {
                    assert_eq!(params["model"], model);
                    assert_eq!(params["config"]["model_reasoning_effort"], "medium");
                }
                if scenario.contains("mcp_early") {
                    request(
                        &mut out,
                        json!("mcp"),
                        "mcpServer/elicitation/request",
                        mcp(&scenario),
                    );
                }
                if scenario.contains("thread_fail") {
                    emit(
                        &mut out,
                        json!({"id":id,"error":{"code":-32000,"message":"private"}}),
                    );
                } else {
                    emit(
                        &mut out,
                        json!({"id":id,"result":{"thread":{"id":"owned"}}}),
                    );
                }
            }
            "turn/start" => {
                assert_eq!(params["threadId"], "owned");
                let ack = json!({"id":id,"result":{"turn":{"id":"turn","status":"inProgress","items":[]}}});
                if scenario.contains("mcp") {
                    turn_ack = Some(ack);
                    if !scenario.contains("early") {
                        request(
                            &mut out,
                            json!("mcp"),
                            "mcpServer/elicitation/request",
                            mcp(&scenario),
                        );
                    }
                    if scenario.contains("resolved")
                        || scenario.contains("terminal")
                        || scenario.contains("closed")
                        || scenario.contains("replacement")
                        || scenario.contains("replacement")
                        || scenario.contains("disconnect")
                    {
                        gate();
                        if scenario.contains("closed") {
                            notification(&mut out, "thread/closed", json!({"threadId":"owned"}));
                        } else if scenario.contains("disconnect") {
                            break;
                        } else {
                            if scenario.contains("resolved") {
                                notification(
                                    &mut out,
                                    "serverRequest/resolved",
                                    json!({"threadId":"owned","requestId":"mcp"}),
                                );
                            }
                            emit(&mut out, turn_ack.take().unwrap());
                            complete(&mut out);
                        }
                    }
                    if scenario.contains("scoped") {
                        emit(&mut out, turn_ack.take().unwrap());
                    }
                } else if scenario == "queued_completion" {
                    emit(&mut out, ack);
                    queued_completion(&mut out, &mut input);
                } else if scenario.starts_with("file") {
                    if scenario.contains("output") {
                        emit(&mut out, ack.clone());
                        for _ in 0..if scenario.contains("overflow") {
                            1000
                        } else {
                            48
                        } {
                            notification(
                                &mut out,
                                "item/agentMessage/delta",
                                json!({"threadId":"owned","turnId":"turn","delta":"text"}),
                            );
                        }
                    }
                    let mut item = context("file");
                    if scenario.contains("combined") {
                        item["changes"][0]["diff"] = json!("");
                        let delivery = json!({"method":"item/fileChange/requestApproval","params":{"threadId":"owned","turnId":"turn","itemId":"file","startedAtMs":0},"item_context":item});
                        let length = 1024 * 1024 - delivery.to_string().len()
                            + usize::from(scenario.contains("over"));
                        item["changes"][0]["diff"] = json!("a".repeat(length));
                    }
                    if scenario.contains("count") {
                        let count = if scenario.contains("17") { 17 } else { 16 };
                        for index in 1..count {
                            notification(
                                &mut out,
                                "item/started",
                                json!({"threadId":"owned","turnId":"turn","item":context(&format!("extra-{index}"))}),
                            );
                        }
                    }
                    if scenario.contains("aggregate") {
                        item["changes"][0]["diff"] = json!("a".repeat(1024 * 1024 - 1024));
                        for index in 1..16 {
                            let mut extra = item.clone();
                            extra["id"] = json!(format!("extra-{index}"));
                            notification(
                                &mut out,
                                "item/started",
                                json!({"threadId":"owned","turnId":"turn","item":extra}),
                            );
                        }
                    }
                    if scenario.contains("oversize") {
                        item["changes"][0]["diff"] = json!("s".repeat(1024 * 1024));
                    }
                    if scenario.contains("malformed") {
                        item["changes"][0]["kind"] = json!("update");
                    }
                    if !scenario.contains("missing") {
                        notification(
                            &mut out,
                            "item/started",
                            json!({"threadId":if scenario.contains("context_thread"){"other"}else{"owned"},"turnId":if scenario.contains("context_turn"){"old"}else{"turn"},"item":item}),
                        );
                    }
                    if scenario.contains("prior_completed") {
                        notification(
                            &mut out,
                            "item/completed",
                            json!({"threadId":"owned","turnId":"turn","item":context("file")}),
                        );
                    }
                    request(
                        &mut out,
                        json!(7),
                        "item/fileChange/requestApproval",
                        json!({"threadId":"owned","turnId":if scenario.contains("wrong_turn"){"other"}else{"turn"},"itemId":if scenario.contains("wrong_item"){"other"}else{"file"},"startedAtMs":0}),
                    );
                    if scenario.contains("missing_late") {
                        notification(
                            &mut out,
                            "item/started",
                            json!({"threadId":"owned","turnId":"turn","item":context("file")}),
                        );
                    }
                    if scenario == "file_two" {
                        notification(
                            &mut out,
                            "item/started",
                            json!({"threadId":"owned","turnId":"turn","item":context("second")}),
                        );
                        request(
                            &mut out,
                            json!("7"),
                            "item/fileChange/requestApproval",
                            json!({"threadId":"owned","turnId":"turn","itemId":"second","startedAtMs":0}),
                        );
                    }
                    emit(&mut out, ack);
                    if scenario.contains("drift")
                        || scenario.contains("completed")
                        || scenario.contains("resolved")
                        || scenario.contains("identical")
                        || scenario.contains("terminal")
                        || scenario.contains("disconnect")
                        || scenario.contains("closed")
                        || scenario.contains("replacement")
                    {
                        gate();
                        if scenario.contains("replacement") {
                            let mut replacement = context("file");
                            replacement["changes"][0]["diff"] = json!("changed");
                            notification(
                                &mut out,
                                "item/started",
                                json!({"threadId":"owned","turnId":"turn","item":replacement}),
                            );
                        }
                        if scenario.contains("drift") {
                            notification(
                                &mut out,
                                "item/fileChange/patchUpdated",
                                json!({"threadId":"owned","turnId":"turn","itemId":"file","changes":[]}),
                            );
                        }
                        if scenario.contains("completed") {
                            notification(
                                &mut out,
                                "item/completed",
                                json!({"threadId":"owned","turnId":"turn","item":context("file")}),
                            );
                        }
                        if scenario.contains("identical") {
                            notification(
                                &mut out,
                                "item/started",
                                json!({"threadId":"owned","turnId":"turn","item":context("file")}),
                            );
                        }
                        if scenario.contains("terminal") {
                            complete(&mut out);
                        }
                        if scenario.contains("disconnect") {
                            break;
                        }
                        if scenario.contains("closed") {
                            notification(&mut out, "thread/closed", json!({"threadId":"owned"}));
                        }
                        if scenario.contains("identical") {
                            request(
                                &mut out,
                                json!("gate"),
                                "mcpServer/elicitation/request",
                                mcp("mcp_null"),
                            );
                        }
                        if scenario.contains("resolved") {
                            notification(
                                &mut out,
                                "serverRequest/resolved",
                                json!({"threadId":"owned","requestId":7}),
                            );
                            complete(&mut out);
                        }
                    }
                } else if matches!(scenario.as_str(), "command" | "permissions" | "user_input") {
                    emit(&mut out, ack);
                    let mut parameters =
                        json!({"threadId":"owned","turnId":"turn","itemId":"item","startedAtMs":0});
                    let method = match scenario.as_str() {
                        "command" => {
                            parameters["command"] = json!("printf secret");
                            "item/commandExecution/requestApproval"
                        }
                        "permissions" => {
                            parameters["cwd"] = json!("/workspace");
                            parameters["permissions"] = json!({"network":{"enabled":true}});
                            "item/permissions/requestApproval"
                        }
                        _ => {
                            parameters.as_object_mut().unwrap().remove("startedAtMs");
                            parameters["isBlocking"] = json!(true);
                            parameters["questions"] =
                                json!([{"id":"q","header":"Choose","question":"Which?"}]);
                            "item/tool/requestUserInput"
                        }
                    };
                    request(&mut out, json!(7), method, parameters);
                } else {
                    emit(&mut out, ack);
                    notification(
                        &mut out,
                        "item/agentMessage/delta",
                        json!({"threadId":"owned","turnId":"turn","delta":"native final"}),
                    );
                    notification(
                        &mut out,
                        "item/completed",
                        json!({"threadId":"owned","turnId":"turn","item":{"id":"answer","type":"agentMessage","phase":if scenario=="phase_commentary"{json!("commentary")}else if scenario=="phase_legacy"{Value::Null}else{json!("final_answer")},"text":"native final"}}),
                    );
                    complete(&mut out);
                }
            }
            "" => {
                if let Ok(path) = env::var("NATIVE_REPLY_LOG") {
                    let mut log = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(path)
                        .unwrap();
                    writeln!(log, "{msg}").unwrap();
                }
                if id == &json!("gate") {
                    assert_eq!(msg["result"], json!({"action":"decline"}));
                    continue;
                }
                if msg.get("error").is_some() {
                    continue;
                }
                responses += 1;
                if scenario.contains("mcp") {
                    assert_eq!(id, &json!("mcp"));
                    assert_eq!(msg["result"], json!({"action":"decline"}));
                    if let Some(ack) = turn_ack.take() {
                        emit(&mut out, ack);
                    }
                } else if scenario == "permissions" {
                    assert_eq!(
                        msg["result"],
                        json!({"permissions":{"network":{"enabled":false}},"scope":"turn"})
                    );
                } else if scenario == "user_input" {
                    assert_eq!(
                        msg["result"],
                        json!({"answers":{"q":{"answers":["a","b"]}}})
                    );
                } else {
                    assert!(id == &json!(7) || id == &json!("7"));
                    assert_eq!(msg["result"], json!({"decision":"acceptForSession"}));
                }
                if scenario == "file_written" {
                    notification(
                        &mut out,
                        "item/completed",
                        json!({"threadId":"owned","turnId":"turn","item":context("file")}),
                    );
                }
                if scenario != "file_two" || responses == 2 {
                    complete(&mut out);
                }
            }
            "turn/interrupt" | "shutdown" => emit(&mut out, json!({"id":id,"result":{}})),
            "exit" => break,
            "initialized" => {}
            other => panic!("unexpected method {other}"),
        }
    }
}
#[cfg(unix)]
fn gate() {
    use std::{io::Read, os::unix::net::UnixStream};
    let mut stream = UnixStream::connect(env::var("NATIVE_GATE").unwrap()).unwrap();
    stream.write_all(b"ready").unwrap();
    let mut byte = [0];
    stream.read_exact(&mut byte).unwrap();
}
#[cfg(not(unix))]
fn gate() {
    panic!("gate fixtures require Unix");
}

#[cfg(unix)]
fn queued_completion(out: &mut impl Write, input: &mut impl BufRead) {
    use std::os::unix::net::UnixStream;
    request(
        out,
        json!("large"),
        "mcpServer/elicitation/request",
        mcp(""),
    );
    let mut prefix = [0];
    // A >700KiB result has begun writing. Stop consuming stdin, so the one writer
    // cannot reach the subsequent file decision until the two socket gates open.
    input.read_exact(&mut prefix).unwrap();
    assert_eq!(prefix[0], b'{');
    notification(
        out,
        "item/started",
        json!({"threadId":"owned","turnId":"turn","item":context("file")}),
    );
    request(
        out,
        json!(7),
        "item/fileChange/requestApproval",
        json!({"threadId":"owned","turnId":"turn","itemId":"file","startedAtMs":0}),
    );
    let mut gate = UnixStream::connect(env::var("NATIVE_GATE").unwrap()).unwrap();
    gate.write_all(b"ready").unwrap();
    let mut release = [0];
    gate.read_exact(&mut release).unwrap();
    notification(
        out,
        "item/completed",
        json!({"threadId":"owned","turnId":"turn","item":context("file")}),
    );
    request(
        out,
        json!("processed"),
        "mcpServer/elicitation/request",
        mcp(""),
    );
    gate.read_exact(&mut release).unwrap();
    let mut rest = String::new();
    input.read_line(&mut rest).unwrap();
    let large: Value = serde_json::from_str(&format!("{{{rest}")).unwrap();
    assert_eq!(large["id"], "large");
    assert_eq!(
        large["result"]["content"]["opaque"].as_str().unwrap().len(),
        720_000
    );
    rest.clear();
    input.read_line(&mut rest).unwrap();
    let next: Value = serde_json::from_str(&rest).unwrap();
    std::fs::write(env::var("NATIVE_REPLY_LOG").unwrap(), next.to_string()).unwrap();
    if next.get("method").is_some() {
        emit(out, json!({"id":next["id"],"result":{}}));
    } else {
        assert!(next["id"] == json!(7) || next["id"] == json!("processed"));
        complete(out);
    }
}
#[cfg(not(unix))]
fn queued_completion(_: &mut impl Write, _: &mut impl BufRead) {
    panic!("unix-only synchronization fixture")
}
