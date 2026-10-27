//! Offline observer driver. Use only through scripts/observe_codex_adapter_children.py.
//! Native execution failures are observations, never successful obligation verdicts.
use agent_api::{
    backends::codex::{CodexBackend, CodexBackendConfig},
    AgentWrapperBackend, AgentWrapperRunRequest,
};
use futures_util::StreamExt;
use serde_json::{json, Value};
use std::{collections::BTreeMap, time::Duration};
use xtask::support_depth::codex_pilot::{decode_route, PilotTransport};

fn extensions(selector: &str, variant: &str) -> Result<BTreeMap<String, Value>, String> {
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
                json!({"selector":"id","id":"11111111-1111-4111-8111-111111111111"})
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
    let extensions = extensions(&args[0], &args[1])?;
    let route = decode_route(&extensions, true)?;
    let backend = CodexBackend::new(CodexBackendConfig {
        binary: Some("/bin/codex".into()),
        codex_home: Some("/pilot/home".into()),
        default_working_dir: Some("/pilot/work".into()),
        allow_external_sandbox_exec: true,
        ..Default::default()
    });
    let request = AgentWrapperRunRequest {
        prompt: "Reply with ok.".into(),
        timeout: Some(Duration::from_secs(3)),
        extensions,
        ..Default::default()
    };
    let mut event_count = 0;
    let outcome = match backend.run(request).await {
        Err(error) => json!({"phase":"request","error":error.to_string()}),
        Ok(run) => {
            let events = async {
                let mut stream = run.events;
                while stream.next().await.is_some() {
                    event_count += 1;
                }
            };
            let (_, completion) = tokio::join!(events, run.completion);
            match completion {
                Ok(completion) => {
                    json!({"phase":"completion","status_success":completion.status.success()})
                }
                Err(error) => json!({"phase":"completion","error":error.to_string()}),
            }
        }
    };
    println!(
        "{}",
        json!({"selector":route.selector,"variant":args[1],
        "transport":match route.transport {PilotTransport::Exec=>"exec",PilotTransport::AppServer=>"app-server"},
        "event_count":event_count,"outcome":outcome,"execution_verified":false})
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
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
                let route = decode_route(&extensions(selector, variant).unwrap(), true).unwrap();
                assert_eq!(route.selector, selector);
                assert_eq!(
                    route.transport == PilotTransport::AppServer,
                    variant == "interactive" || selector.starts_with("fork")
                );
            }
        }
        assert!(extensions("unknown", "unattended").is_err());
        assert!(extensions("initial-json", "unknown").is_err());
    }
}
