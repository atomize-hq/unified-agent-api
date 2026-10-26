//! Bind acquired schemas and the current implementation without treating either as runtime proof.
use crate::{
    depth_record::{part_identity, FileRef},
    git_inputs::GitInputs,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

const PROTOCOL: &str = "cli_manifests/codex/reports/0.156.1/pilot-protocol-facts.json";
const PROTOCOL_SHA256: &str = "964601de43f695ef07f1d992c4c14a7418346bdd8dbe90c4ac9123ec0a5e1fb2";

pub(in crate::support_depth) fn load(
    view: &mut GitInputs,
) -> Result<(Value, Vec<FileRef>), String> {
    let raw = view.required(PROTOCOL)?;
    if hex::encode(Sha256::digest(&raw)) != PROTOCOL_SHA256 {
        return Err(
            "pilot protocol/config acquisition is incomplete, incompatible or changed".into(),
        );
    }
    let report: Value = serde_json::from_slice(&raw).map_err(|e| e.to_string())?;
    // Keep numeric schema payloads in the raw/content digest; B.7 does not project JSON numbers.
    let mut refs = vec![view.reference(PROTOCOL)?];
    let mut source = Vec::new();
    for scope in [
        "crates/codex/src",
        "crates/agent_api/src",
        "crates/wrapper_events/src",
        "crates/xtask/src/support_depth",
    ] {
        for path in view.files_under(scope)? {
            let reference = view.reference(&path)?;
            source.push(json!({"input":reference,"executable":view.executable(&path)?}));
            refs.push(reference);
        }
    }
    for path in [
        "crates/xtask/src/support_depth.rs",
        "crates/xtask/src/git_inputs.rs",
        "crates/xtask/src/manifest_union.rs",
        "crates/xtask/src/manifest_report.rs",
        "crates/xtask/src/manifest_snapshot_schema.rs",
        "Cargo.toml",
        "Cargo.lock",
        "crates/codex/Cargo.toml",
        "crates/agent_api/Cargo.toml",
        "crates/wrapper_events/Cargo.toml",
        "crates/xtask/Cargo.toml",
        "crates/xtask/build.rs",
        "scripts/acquire_codex_pilot_protocol.py",
        "docs/specs/codex-interaction-contract.md",
        "docs/specs/codex-streaming-exec-contract.md",
        "docs/specs/codex-app-server-jsonrpc-contract.md",
        "docs/specs/unified-agent-api/extensions-spec.md",
    ] {
        let bytes = view.optional(path)?;
        if bytes.is_some() {
            let reference = view.reference(path)?;
            source.push(json!({"input":reference,"executable":view.executable(path)?}));
            refs.push(reference);
        } else if path != "crates/xtask/build.rs" {
            return Err(format!("missing pilot source/config fact: {path}"));
        }
    }
    source.sort_by_key(Value::to_string);
    let features = report["config_schema"]["properties"]["features"]["properties"]
        .as_object()
        .ok_or("pinned configuration schema has no complete feature set")?;
    let mut config = String::from("model_provider = \"openai\"\nmodel = \"gpt-5\"\napproval_policy = \"never\"\nsandbox_mode = \"workspace-write\"\nweb_search = \"disabled\"\n[analytics]\nenabled = false\n[features]\n");
    let schema =
        jsonschema::JSONSchema::compile(&report["config_schema"]).map_err(|e| e.to_string())?;
    let mut unbound = Vec::new();
    for key in features.keys() {
        if !schema.is_valid(&json!({"features":{key:false}})) {
            unbound.push(key.clone());
            continue;
        }
        config.push_str(&format!("{key} = false\n"));
    }
    let typed: Value = toml_edit::de::from_str(&config).map_err(|e| e.to_string())?;
    if !schema.is_valid(&typed) {
        return Err("controlled config does not match pinned schema".into());
    }
    let facts = json!({
        "revision":"codex-pilot-routes-v1", "protocol_sha256":PROTOCOL_SHA256,
        "binary_sha256":report["binary_sha256"], "unbound_object_feature_defaults":unbound,
        "source_sha256":part_identity(&source)?, "source":source,
        "expected_config_toml":config,
        "context_observation":"not_acquired",
        "routes": [
            {"operation":"run-initial","selectors":["initial-json"],"exec":true,"managed":true},
            {"operation":"run-resume","selectors":["resume-last-json","resume-id-json"],"exec":true,"managed":true},
            {"operation":"run-fork","selectors":["fork-last-stdio","fork-id-stdio"],"exec":false,"managed":true}
        ],
        "managed_initial_resume":["config/read","configRequirements/read","thread/start|thread/resume","turn/start"],
        "managed_fork":["thread/list (last only)","thread/fork","turn/start"],
        "child_environment":"canonical CommandEnvironment on every route; normalized overlay last",
        "runtime_roots":"cwd + requested roots + eligible ambient roots; effective native config and requirements",
        "persistence":"exec temporary last-message output; native thread/session artifacts including partial failure",
        "ownership":"native handler, correlated item/thread/turn context, bounded frames and retained JSON; owned deadline/cancel/cleanup",
        "controlled_context":"private pinned binary/home/cwd/roots; cleared producer environment; no unbound loaders, provider, features, services or credentials; runtime proof required"
    });
    Ok((facts, refs))
}
