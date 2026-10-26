use super::{NotificationHook, PendingRequests, WriteMessage};
use crate::mcp::interactions::{AppServerMessage, Delivery, MAX_FRAME};
use crate::mcp::{
    AppNotification, ApprovalKind, ApprovalRequest, CodexEvent, McpError, RequestId,
    METHOD_CODEX_EVENT,
};
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::ChildStdout,
    sync::{mpsc, Mutex},
};
use tracing::warn;
#[derive(Debug, Deserialize)]
struct RpcResponse {
    id: Value,
    result: Option<Value>,
    error: Option<RpcError>,
}

#[derive(Debug, Deserialize)]
struct RpcNotification {
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct RpcError {
    code: i64,
    message: String,
    data: Option<Value>,
}

pub(super) async fn reader_task(
    stdout: ChildStdout,
    pending: PendingRequests,
    notification_hook: NotificationHook,
    mirror_stdio: bool,
    writer: mpsc::UnboundedSender<WriteMessage>,
) {
    let is_app = matches!(notification_hook, NotificationHook::App { .. });
    let mut reader = BufReader::new(stdout);
    loop {
        let line = match next_line(&mut reader, is_app).await {
            Ok(Some(line)) => line,
            Ok(None) => break,
            Err(_) => {
                fail_interactions(&notification_hook, "invalid or oversized app-server frame");
                break;
            }
        };
        if mirror_stdio && !is_app {
            eprintln!("[mcp stdout] {line}");
        }

        if line.trim().is_empty() {
            continue;
        }

        let mut value: Value = match serde_json::from_str(&line) {
            Ok(value) => value,
            Err(err) => {
                if is_app {
                    fail_interactions(&notification_hook, "malformed app-server JSON");
                    break;
                }
                warn!("failed to parse MCP stdout as JSON: {err}");
                continue;
            }
        };

        // Charge normalized serialized bytes; decoded number/string spelling may differ.
        let frame_bytes = if is_app {
            value.to_string().len()
        } else {
            line.len()
        };
        drop(line);

        // App-server server requests have their own id space (including string ids).
        // Never misclassify one as a response to our request or authorize an action.
        if value.get("method").is_some() && value.get("id").is_some() {
            if let NotificationHook::App {
                sinks,
                interactions,
            } = &notification_hook
            {
                if let Some(delivery) = interactions {
                    let id = value["id"].take();
                    let method = value["method"].as_str().unwrap_or_default().to_owned();
                    let params = value["params"].take();
                    drop(value);
                    let request = delivery
                        .state
                        .request(id.clone(), method, params, frame_bytes);
                    match request {
                        Ok(request) => {
                            drop(id);
                            if delivery
                                .messages
                                .send(Delivery {
                                    message: AppServerMessage::Request(request),
                                    _bytes: None,
                                })
                                .await
                                .is_err()
                            {
                                delivery.state.close();
                                break;
                            }
                        }
                        Err(_) => {
                            let reply = serde_json::json!({"jsonrpc":"2.0","id":id,"error":{"code":-32601,"message":"client interaction unsupported"}});
                            let _ = writer.send(WriteMessage::Line(reply.to_string()));
                            delivery
                                .state
                                .fail("unsupported or malformed native interaction");
                            break;
                        }
                    }
                    continue;
                }
                let method = value["method"].as_str().unwrap_or_default();
                let approval = matches!(
                    method,
                    "item/commandExecution/requestApproval"
                        | "item/fileChange/requestApproval"
                        | "item/permissions/requestApproval"
                        | "execCommandApproval"
                        | "applyPatchApproval"
                );
                let reply = serde_json::json!({"jsonrpc":"2.0","id":value["id"],"error":{"code":-32601,"message":"client interaction unsupported"}});
                let _ = writer.send(WriteMessage::Line(reply.to_string()));
                broadcast_app_event(
                    AppNotification::Raw {
                        method: method.to_string(),
                        params: value["params"].clone(),
                    },
                    sinks,
                )
                .await;
                let mut guard = pending.lock().await;
                for (_, tx) in guard.drain() {
                    let _ = tx.send(Err(McpError::Server(
                        if approval {
                            "approval required"
                        } else {
                            "client interaction required"
                        }
                        .into(),
                    )));
                }
                continue;
            }
        }
        match decode_message(value) {
            Some(Incoming::Response(response)) => {
                handle_response(response, &pending).await;
            }
            Some(Incoming::Notification(notification)) => match &notification_hook {
                NotificationHook::Codex { sinks } => {
                    if notification.method == METHOD_CODEX_EVENT {
                        let params = notification.params.unwrap_or(Value::Null);
                        let event = parse_codex_event(&params).unwrap_or(CodexEvent::Raw {
                            method: METHOD_CODEX_EVENT.to_string(),
                            params,
                        });
                        broadcast_codex_event(event, sinks).await;
                    }
                }
                NotificationHook::App {
                    sinks,
                    interactions,
                } => {
                    let params = notification.params.unwrap_or(Value::Null);
                    if let Some(delivery) = interactions {
                        if notification.method == "serverRequest/resolved" {
                            delivery
                                .state
                                .resolved(&params["requestId"], params["threadId"].as_str());
                        }
                        let bytes = match delivery.state.reserve(frame_bytes) {
                            Ok(bytes) => bytes,
                            Err(_) => {
                                delivery
                                    .state
                                    .fail("native interaction retained-byte limit exceeded");
                                break;
                            }
                        };
                        if delivery
                            .messages
                            .send(Delivery {
                                message: AppServerMessage::Notification {
                                    method: notification.method,
                                    params,
                                },
                                _bytes: Some(bytes),
                            })
                            .await
                            .is_err()
                        {
                            delivery.state.close();
                            break;
                        }
                        continue;
                    }
                    let event = parse_app_notification(&notification.method, &params);
                    broadcast_app_event(event, sinks).await;
                }
            },
            None => {
                if is_app {
                    fail_interactions(&notification_hook, "malformed app-server message");
                    break;
                }
                warn!("received malformed MCP message");
            }
        }
    }

    match &notification_hook {
        NotificationHook::App {
            sinks,
            interactions,
        } => {
            if let Some(delivery) = interactions {
                delivery.state.fail("app-server disconnected");
            }
            sinks.lock().await.clear();
        }
        NotificationHook::Codex { .. } => {}
    }
    let mut guard = pending.lock().await;
    for (_, tx) in guard.drain() {
        let _ = tx.send(Err(McpError::ChannelClosed));
    }
}

fn fail_interactions(hook: &NotificationHook, message: &str) {
    if let NotificationHook::App {
        interactions: Some(delivery),
        ..
    } = hook
    {
        delivery.state.fail(message);
    }
}

// Inspect buffered chunks before growing the frame allocation. MCP retains its legacy framing.
async fn next_line(
    reader: &mut BufReader<ChildStdout>,
    bounded: bool,
) -> std::io::Result<Option<String>> {
    let mut bytes = Vec::new();
    loop {
        let available = reader.fill_buf().await?;
        if available.is_empty() {
            if bytes.is_empty() {
                return Ok(None);
            }
            break;
        }
        let length = available
            .iter()
            .position(|b| *b == b'\n')
            .map_or(available.len(), |n| n + 1);
        let newline = available[length - 1] == b'\n';
        let payload_length = length - usize::from(newline);
        if bounded && bytes.len() + payload_length > MAX_FRAME {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "frame bound",
            ));
        }
        bytes.extend_from_slice(&available[..payload_length]);
        reader.consume(length);
        if newline {
            break;
        }
    }
    if bytes.last() == Some(&b'\r') {
        bytes.pop();
    }
    String::from_utf8(bytes)
        .map(Some)
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "frame encoding"))
}

#[derive(Debug)]
enum Incoming {
    Response(RpcResponse),
    Notification(RpcNotification),
}

fn decode_message(value: Value) -> Option<Incoming> {
    let is_notification = value.get("id").is_none() && value.get("method").is_some();
    if is_notification {
        let notification: RpcNotification = serde_json::from_value(value).ok()?;
        return Some(Incoming::Notification(notification));
    }

    let is_response = value.get("id").is_some();
    if is_response {
        let response: RpcResponse = serde_json::from_value(value).ok()?;
        return Some(Incoming::Response(response));
    }

    None
}

async fn handle_response(response: RpcResponse, pending: &PendingRequests) {
    let Some(id) = parse_request_id(&response.id) else {
        warn!("received response without numeric id");
        return;
    };

    let sender = { pending.lock().await.remove(&id) };
    let Some(tx) = sender else {
        return;
    };

    if let Some(err) = response.error {
        let mapped = if err.code == -32800 {
            McpError::Cancelled
        } else {
            McpError::Rpc {
                code: err.code,
                message: err.message,
                data: err.data,
            }
        };
        let _ = tx.send(Err(mapped));
    } else if let Some(result) = response.result {
        let _ = tx.send(Ok(result));
    } else {
        let _ = tx.send(Err(McpError::Transport(
            "response missing result and error".into(),
        )));
    }
}

fn parse_request_id(value: &Value) -> Option<RequestId> {
    if let Some(num) = value.as_u64() {
        return Some(num);
    }

    value.as_str().and_then(|s| s.parse::<RequestId>().ok())
}

fn parse_codex_event(value: &Value) -> Option<CodexEvent> {
    let payload = value.get("msg").unwrap_or(value);
    let event_type = payload.get("type")?.as_str()?;
    let conversation_id = payload
        .get("thread_id")
        .or_else(|| payload.get("threadId"))
        .or_else(|| payload.get("session_id"))
        .or_else(|| payload.get("conversation_id"))
        .or_else(|| payload.get("conversationId"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    match event_type {
        "task_complete" => Some(CodexEvent::TaskComplete {
            conversation_id,
            result: payload
                .get("result")
                .cloned()
                .unwrap_or_else(|| payload.clone()),
        }),
        "approval_required" | "approval" => {
            let approval_id = payload
                .get("approval_id")
                .or_else(|| payload.get("id"))
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            let raw_kind = payload
                .get("kind")
                .or_else(|| payload.get("approval_kind"))
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_string();

            let kind = match raw_kind.to_lowercase().as_str() {
                "exec" => ApprovalKind::Exec,
                "apply" => ApprovalKind::Apply,
                other => ApprovalKind::Unknown(other.to_string()),
            };

            Some(CodexEvent::ApprovalRequired(ApprovalRequest {
                approval_id,
                kind,
                payload: payload.clone(),
            }))
        }
        "cancelled" | "canceled" => Some(CodexEvent::Cancelled {
            conversation_id: payload
                .get("conversation_id")
                .and_then(Value::as_str)
                .map(|s| s.to_string()),
            reason: payload
                .get("reason")
                .and_then(Value::as_str)
                .map(|s| s.to_string()),
        }),
        "error" => Some(CodexEvent::Error {
            message: payload
                .get("message")
                .or_else(|| payload.get("error"))
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            data: payload.get("data").cloned(),
        }),
        _ => None,
    }
}

async fn broadcast_codex_event(
    event: CodexEvent,
    sinks: &Arc<Mutex<Vec<mpsc::UnboundedSender<CodexEvent>>>>,
) {
    let mut guard = sinks.lock().await;
    guard.retain(|tx| tx.send(event.clone()).is_ok());
}

fn parse_app_notification(method: &str, value: &Value) -> AppNotification {
    let notification_type = value
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_lowercase();

    let thread_id = extract_string(value, &["thread_id", "threadId"]).unwrap_or_default();
    let turn_id = extract_string(value, &["turn_id", "turnId"]);

    match notification_type.as_str() {
        "task_complete" | "taskcomplete" => AppNotification::TaskComplete {
            thread_id,
            turn_id,
            result: value.get("result").cloned().unwrap_or(Value::Null),
        },
        "item" => AppNotification::Item {
            thread_id,
            turn_id,
            item: value.get("item").cloned().unwrap_or_else(|| value.clone()),
        },
        "error" => AppNotification::Error {
            message: value
                .get("message")
                .or_else(|| value.get("error"))
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            data: value.get("data").cloned(),
        },
        _ => AppNotification::Raw {
            method: method.to_string(),
            params: value.clone(),
        },
    }
}

fn extract_string(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| value.get(key).and_then(Value::as_str))
        .map(|s| s.to_string())
}

async fn broadcast_app_event(
    event: AppNotification,
    sinks: &Arc<Mutex<Vec<mpsc::UnboundedSender<AppNotification>>>>,
) {
    let mut guard = sinks.lock().await;
    guard.retain(|tx| tx.send(event.clone()).is_ok());
}
