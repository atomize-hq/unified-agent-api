//! Offline observer driver. Use only through scripts/observe_codex_adapter_children.py.
//! Native execution failures are observations, never successful obligation verdicts.
#[path = "codex_native_context/fixture.rs"]
mod fixture;

use agent_api::{
    backends::codex::{CodexBackend, CodexBackendConfig},
    AgentWrapperBackend, AgentWrapperRunRequest,
};
use futures_util::StreamExt;
use serde_json::{json, Value};
use std::{collections::BTreeMap, time::Duration};
use xtask::support_depth::codex_pilot::{decode_route, PilotTransport};

fn extensions(
    selector: &str,
    variant: &str,
    session: &str,
) -> Result<BTreeMap<String, Value>, String> {
    let mut result = BTreeMap::new();
    match selector {
        "initial-json" => {}
        "resume-last-json" | "resume-id-json" | "fork-last-stdio" | "fork-id-stdio" => {
            let key = if selector.starts_with("resume") {
                "agent_api.session.resume.v1"
            } else {
                "agent_api.session.fork.v1"
            };
            let value = if selector.contains("-last-") {
                json!({"selector":"last"})
            } else {
                json!({"selector":"id","id":session})
            };
            result.insert(key.into(), value);
        }
        _ => return Err("unknown fixed pilot selector".into()),
    }
    match variant {
        "unattended" => {}
        "interactive" => {
            result.insert("agent_api.exec.non_interactive".into(), json!(false));
            result.insert(
                "backend.codex.exec.approval_policy".into(),
                json!("on-request"),
            );
        }
        "external" => {
            result.insert("agent_api.exec.external_sandbox.v1".into(), json!(true));
        }
        _ => return Err("unknown fixed pilot safety variant".into()),
    }
    decode_route(&result, true)?;
    Ok(result)
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err("expected fixed selector and safety variant".into());
    }
    // The outer collector, not an arbitrary CLI caller, supplies the isolated namespace.
    if std::env::var("CODEX_HOME").as_deref() != Ok("/pilot/home")
        || std::env::current_dir().map_err(|e| e.to_string())?
            != std::path::Path::new("/pilot/work")
    {
        return Err("requires private producer context".into());
    }
    let result = if args[0] == "session-chain" {
        fixture::run(&args[1]).await?
    } else {
        let cancel = args[0] == "cancel-initial-json";
        let selector = if cancel { "initial-json" } else { &args[0] };
        observe(
            selector,
            &args[1],
            "11111111-1111-4111-8111-111111111111",
            cancel,
            Duration::from_secs(3),
        )
        .await?
    };
    println!("{result}");
    Ok(())
}

async fn observe(
    selector: &str,
    variant: &str,
    session: &str,
    cancel_on_session: bool,
    timeout: Duration,
) -> Result<Value, String> {
    let extensions = extensions(selector, variant, session)?;
    let route = decode_route(&extensions, true)?;
    let backend = CodexBackend::new(CodexBackendConfig {
        binary: Some("/bin/codex".into()),
        codex_home: Some("/pilot/home".into()),
        default_working_dir: Some("/pilot/work".into()),
        allow_external_sandbox_exec: true,
        ..Default::default()
    });
    let request = AgentWrapperRunRequest {
        prompt: "Reply with exactly the text: ok.".into(),
        timeout: Some(timeout),
        extensions,
        ..Default::default()
    };
    let mut event_count = 0;
    let mut session_id = None;
    let mut consistent = true;
    let mut cancel_requested = false;
    let mut status_success = false;
    let mut expected_reply = false;
    let run = if cancel_on_session {
        backend
            .run_control(request)
            .await
            .map(|control| (control.handle, Some(control.cancel)))
    } else {
        backend.run(request).await.map(|handle| (handle, None))
    };
    let outcome = match run {
        Err(error) => json!({"phase":"request","error":error.to_string()}),
        Ok((run, cancel)) => {
            let events = async {
                let mut stream = run.events;
                while let Some(event) = stream.next().await {
                    event_count += 1;
                    bind_session(event.data.as_ref(), &mut session_id, &mut consistent);
                    if cancel_on_session && session_id.is_some() && !cancel_requested {
                        cancel.as_ref().expect("owned cancellation handle").cancel();
                        cancel_requested = true;
                    }
                }
            };
            let (_, completion) = tokio::join!(events, run.completion);
            match completion {
                Ok(completion) => {
                    bind_session(completion.data.as_ref(), &mut session_id, &mut consistent);
                    status_success = completion.status.success();
                    expected_reply = completion.final_text.as_deref().map(str::trim) == Some("ok.");
                    json!({"phase":"completion","status_success":completion.status.success()})
                }
                Err(error) => json!({"phase":"completion","error":error.to_string()}),
            }
        }
    };
    Ok(json!({"selector":route.selector,"variant":variant,
        "transport":match route.transport {PilotTransport::Exec=>"exec",PilotTransport::AppServer=>"app-server"},
        "event_count":event_count,"outcome":outcome,"status_success":status_success,
        "session_id":session_id,"session_identity_consistent":consistent,"expected_reply":expected_reply,
        "cancel_requested":cancel_requested,"timeout_seconds":timeout.as_secs(),"execution_verified":false}))
}

fn bind_session(data: Option<&Value>, session: &mut Option<String>, consistent: &mut bool) {
    let Some(data) = data.filter(|data| {
        data.get("schema").and_then(Value::as_str) == Some("agent_api.session.handle.v1")
    }) else {
        return;
    };
    let id = data
        .get("session")
        .and_then(|value| value.get("id"))
        .and_then(Value::as_str);
    match id.filter(|id| fixture::valid_id(id)) {
        Some(id) if session.as_deref().map_or(true, |old| old == id) => {
            *session = Some(id.into());
        }
        _ => {
            *consistent = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn session_facets_require_consistent_native_ids() {
        let mut id = None;
        let mut consistent = true;
        let facet = |id: &str| json!({"schema":"agent_api.session.handle.v1","session":{"id":id}});
        bind_session(
            Some(&facet("aaaaaaaa-1111-4111-8111-111111111111")),
            &mut id,
            &mut consistent,
        );
        assert!(consistent);
        assert!(id.is_some());
        bind_session(
            Some(&facet("bbbbbbbb-1111-4111-8111-111111111111")),
            &mut id,
            &mut consistent,
        );
        assert!(!consistent);
        let mut missing = None;
        let mut valid = true;
        bind_session(Some(&facet("not-a-session")), &mut missing, &mut valid);
        assert!(!valid);
        assert!(missing.is_none());
    }
    #[test]
    fn fixed_requests_match_all_production_route_variants_and_reject_unknowns() {
        for selector in [
            "initial-json",
            "resume-last-json",
            "resume-id-json",
            "fork-last-stdio",
            "fork-id-stdio",
        ] {
            for variant in ["unattended", "interactive", "external"] {
                let route = decode_route(
                    &extensions(selector, variant, "11111111-1111-4111-8111-111111111111").unwrap(),
                    true,
                )
                .unwrap();
                assert_eq!(route.selector, selector);
                assert_eq!(
                    route.transport == PilotTransport::AppServer,
                    variant == "interactive" || selector.starts_with("fork")
                );
            }
        }
        assert!(extensions("unknown", "unattended", "unused").is_err());
        assert!(extensions("initial-json", "unknown", "unused").is_err());
    }
}
