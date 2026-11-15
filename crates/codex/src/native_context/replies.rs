//! Project only actual owning-child replies; never issue diagnostic RPCs or retain raw values.
use super::*;
use serde_json::Value;
use std::path::PathBuf;
pub(crate) struct ReplyRoles {
    cwd: Option<PathBuf>,
    roots: Option<Vec<PathBuf>>,
    home: Option<PathBuf>,
}
impl ReplyRoles {
    /// `home` is the native home the owning child was launched with, resolved like the native
    /// resolver (`CODEX_HOME`, else `$HOME/.codex`) from its effective environment.
    pub(crate) fn from_params(params: &Value, home: Option<&Path>) -> Option<Self> {
        if !tracing::enabled!(target: TARGET, tracing::Level::TRACE) {
            return None;
        }
        let cwd = params["cwd"]
            .as_str()
            .filter(|p| p.len() <= 4096)
            .map(PathBuf::from);
        let roots = params["runtimeWorkspaceRoots"]
            .as_array()
            .filter(|r| r.len() <= 64)
            .and_then(|r| {
                r.iter()
                    .map(|p| p.as_str().filter(|p| p.len() <= 4096).map(PathBuf::from))
                    .collect()
            });
        Some(Self {
            cwd,
            roots,
            home: home.map(Path::to_path_buf),
        })
    }
}
fn matches_path(value: &Value, expected: &Path) -> bool {
    // The producer owns canonical role validation. Keep the SDK projection passive:
    // compare in memory, without a filesystem read or normalization of native replies.
    let rebuilt: PathBuf = expected.components().collect();
    value
        .as_str()
        .is_some_and(|p| Path::new(p).as_os_str() == expected.as_os_str())
        && expected.is_absolute()
        && rebuilt.as_os_str() == expected.as_os_str()
        && !expected
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
}

const SYSTEM_CONFIG: &str = "/etc/codex/config.toml";
// Exact member sets of each pinned wire source; extra or missing members are `Unknown`.
fn source(name: &Value, roles: Option<&ReplyRoles>) -> (LayerClass, bool) {
    let Some(o) = name.as_object() else {
        return (LayerClass::Unknown, false);
    };
    let keys = |expected: &[&str]| {
        o.len() == expected.len() && expected.iter().all(|k| o.contains_key(*k))
    };
    let path =
        |key: &str, expected: Option<PathBuf>| expected.is_some_and(|p| matches_path(&o[key], &p));
    let home = roles.and_then(|r| r.home.as_ref());
    let cwd = roles.and_then(|r| r.cwd.as_ref());
    match o.get("type").and_then(Value::as_str) {
        Some("packagedDefaults") if keys(&["type", "file"]) => {
            (LayerClass::PackagedDefaults, false)
        }
        Some("mdm") if keys(&["type", "domain", "key"]) => (LayerClass::Mdm, false),
        Some("system") if keys(&["type", "file"]) => (
            LayerClass::System,
            path("file", Some(PathBuf::from(SYSTEM_CONFIG))),
        ),
        Some("enterpriseManaged") if keys(&["type", "id", "name"]) => {
            (LayerClass::EnterpriseManaged, false)
        }
        Some("user") if keys(&["type", "file"]) || keys(&["type", "file", "profile"]) => (
            LayerClass::User,
            matches!(o.get("profile"), None | Some(Value::Null))
                && path("file", home.map(|h| h.join("config.toml"))),
        ),
        Some("project") if keys(&["type", "dotCodexFolder"]) => (
            LayerClass::Project,
            path("dotCodexFolder", cwd.map(|c| c.join(".codex"))),
        ),
        Some("sessionFlags") if keys(&["type"]) => (LayerClass::SessionFlags, true),
        Some("legacyManagedConfigTomlFromFile") if keys(&["type", "file"]) => {
            (LayerClass::LegacyManagedFile, false)
        }
        Some("legacyManagedConfigTomlFromMdm") if keys(&["type"]) => {
            (LayerClass::LegacyManagedMdm, false)
        }
        _ => (LayerClass::Unknown, false),
    }
}
/// Project the requested native layer stack. Raw layer configs, paths, versions and disabled
/// reasons stay in memory; only classes and booleans are emitted.
pub(crate) fn layers(value: &Value, roles: Option<&ReplyRoles>) -> Option<Layers> {
    let raw = value.get("layers")?.as_array()?;
    let origins = value.get("origins")?.as_object()?;
    if raw.len() > MAX_LAYERS || origins.len() > 4096 {
        return None;
    }
    let mut entries = Vec::with_capacity(raw.len());
    let mut enabled = Vec::new();
    for layer in raw {
        let o = layer.as_object()?;
        let disabled = match o.get("disabledReason") {
            None => false,
            Some(Value::String(_)) => true,
            _ => return None,
        };
        if !o
            .keys()
            .all(|k| matches!(k.as_str(), "name" | "version" | "config" | "disabledReason"))
            || !o.get("version").is_some_and(Value::is_string)
        {
            return None;
        }
        let (class, role_matches) = source(o.get("name")?, roles);
        if !disabled {
            enabled.push((o.get("name")?, o.get("version")?));
        }
        entries.push(Layer {
            class,
            enabled: !disabled,
            empty: o.get("config")?.as_object().is_some_and(|c| c.is_empty()),
            role_matches,
        });
    }
    let mut classes = std::collections::BTreeSet::new();
    let mut origins_enabled = true;
    for metadata in origins.values() {
        let m = metadata.as_object()?;
        let (name, version) = (m.get("name")?, m.get("version")?);
        classes.insert(source(name, roles).0);
        origins_enabled &= m.len() == 2 && enabled.iter().any(|(n, v)| *n == name && *v == version);
    }
    Some(Layers {
        entries,
        origin_classes: classes.into_iter().collect(),
        origins_enabled,
    })
}

fn approval(value: &Value) -> Option<ThreadApproval> {
    match value.as_str() {
        Some("never") => return Some(ThreadApproval::Never),
        Some("on-request") => return Some(ThreadApproval::OnRequest),
        Some("untrusted") => return Some(ThreadApproval::Untrusted),
        Some("on-failure") => return Some(ThreadApproval::OnFailure),
        _ => (),
    }
    let outer = value.as_object()?;
    if outer.len() != 1 {
        return None;
    }
    let fields = outer.get("granular")?.as_object()?;
    if fields.len() != 5 {
        return None;
    }
    Some(ThreadApproval::Granular {
        sandbox_approval: fields.get("sandbox_approval")?.as_bool()?,
        rules: fields.get("rules")?.as_bool()?,
        mcp_elicitations: fields.get("mcp_elicitations")?.as_bool()?,
        request_permissions: fields.get("request_permissions")?.as_bool()?,
        skill_approval: fields.get("skill_approval")?.as_bool()?,
    })
}
pub(crate) fn thread_reply(
    value: &Value,
    operation: ThreadOperation,
    roles: Option<&ReplyRoles>,
    owner: u64,
) {
    if !tracing::enabled!(target: TARGET, tracing::Level::TRACE) {
        return;
    }
    let projection = ThreadSettings {
        operation,
        model_gpt5: value["model"] == "gpt-5",
        provider_openai: value["modelProvider"] == "openai",
        service_tier_default: value.get("serviceTier") == Some(&Value::Null),
        approval: approval(&value["approvalPolicy"]),
        reviewer_user: value["approvalsReviewer"] == "user",
        sandbox_mode: match value["sandbox"]["type"].as_str() {
            Some("readOnly") => Some(Sandbox::ReadOnly),
            Some("workspaceWrite") => Some(Sandbox::WorkspaceWrite),
            Some("dangerFullAccess") => Some(Sandbox::DangerFullAccess),
            _ => None,
        },
        cwd_matches: roles
            .and_then(|r| r.cwd.as_deref())
            .is_some_and(|cwd| matches_path(&value["cwd"], cwd)),
        runtime_roots_match: roles
            .and_then(|r| r.roots.as_ref())
            .zip(value["runtimeWorkspaceRoots"].as_array())
            .is_some_and(|(expected, observed)| {
                expected.len() == observed.len()
                    && expected
                        .iter()
                        .zip(observed)
                        .all(|(p, v)| matches_path(v, p))
            }),
        instructions_empty: value["instructionSources"]
            .as_array()
            .is_some_and(Vec::is_empty),
    };
    emit("thread", &projection, owner);
}
pub(crate) fn turn_reply(value: &Value, owner: u64) {
    if !tracing::enabled!(target: TARGET, tracing::Level::TRACE) {
        return;
    }
    let accepted = value["turn"]["status"] == "inProgress"
        && value["turn"]["id"]
            .as_str()
            .is_some_and(|id| !id.is_empty() && id.len() <= 128);
    emit("turn", &accepted, owner);
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn assigned_path_comparison_rejects_lexical_aliases_without_filesystem_reads() {
        let root = Path::new("/source-owned-role/does-not-have-to-exist");
        assert!(matches_path(
            &Value::String(root.display().to_string()),
            root
        ));
        assert!(!matches_path(
            &Value::String(format!("{}/.", root.display())),
            root
        ));
        assert!(!matches_path(
            &Value::String(root.display().to_string()),
            &root.join(".")
        ));
        assert!(!matches_path(
            &Value::String("/root/../work".into()),
            Path::new("/root/../work")
        ));
    }
    fn roles() -> ReplyRoles {
        ReplyRoles {
            cwd: Some(PathBuf::from("/role/project")),
            roots: None,
            home: Some(PathBuf::from("/role/home")),
        }
    }
    fn stack() -> Value {
        let v = |name: Value| serde_json::json!({"name":name,"version":"sk-sentinel-version"});
        serde_json::json!({
            "config":{},
            "origins":{
                "model":v(serde_json::json!({"type":"user","file":"/role/home/config.toml"})),
                "sandbox_mode":v(serde_json::json!({"type":"sessionFlags"}))
            },
            "layers":[
                {"name":{"type":"sessionFlags"},"version":"sk-sentinel-version","config":{"sandbox_mode":"sk-sentinel-flag"}},
                {"name":{"type":"project","dotCodexFolder":"/role/project/.codex"},"version":"sk-sentinel-version","config":{},"disabledReason":"sk-sentinel-untrusted"},
                {"name":{"type":"user","file":"/role/home/config.toml"},"version":"sk-sentinel-version","config":{"model":"sk-sentinel-user"}},
                {"name":{"type":"system","file":"/etc/codex/config.toml"},"version":"sk-sentinel-version","config":{}}
            ]
        })
    }
    #[test]
    fn layer_stack_projects_classes_roles_and_origins_without_retaining_native_strings() {
        let observed = layers(&stack(), Some(&roles())).unwrap();
        let class = |c, enabled, empty| Layer {
            class: c,
            enabled,
            empty,
            role_matches: true,
        };
        assert_eq!(
            observed,
            Layers {
                entries: vec![
                    class(LayerClass::SessionFlags, true, false),
                    class(LayerClass::Project, false, true),
                    class(LayerClass::User, true, false),
                    class(LayerClass::System, true, true),
                ],
                origin_classes: vec![LayerClass::User, LayerClass::SessionFlags],
                origins_enabled: true,
            }
        );
        let raw = serde_json::to_string(&observed).unwrap();
        assert!(!raw.contains("sk-") && !raw.contains("/role") && !raw.contains("/etc"));
        // Without the requested member, or with an unbounded stack, nothing is claimed.
        let mut absent = stack();
        absent.as_object_mut().unwrap().remove("layers");
        assert!(layers(&absent, Some(&roles())).is_none());
        let mut many = stack();
        let first = many["layers"][0].clone();
        many["layers"] = Value::Array(vec![first; MAX_LAYERS + 1]);
        assert!(layers(&many, Some(&roles())).is_none());
    }
    #[test]
    fn foreign_managed_and_malformed_layers_never_match_their_roles() {
        let at = |i: usize, name: Value| {
            let mut v = stack();
            v["layers"][i]["name"] = name;
            layers(&v, Some(&roles())).unwrap().entries[i].clone()
        };
        use serde_json::json;
        // Wrong user file, a selected profile, an ancestor project and another system file.
        assert!(!at(2, json!({"type":"user","file":"/other/config.toml"})).role_matches);
        let profile = at(
            2,
            json!({"type":"user","file":"/role/home/config.toml","profile":"p"}),
        );
        assert!(profile.class == LayerClass::User && !profile.role_matches);
        assert!(!at(1, json!({"type":"project","dotCodexFolder":"/role/.codex"})).role_matches);
        assert!(
            !at(
                3,
                json!({"type":"system","file":"/etc/codex/../codex/config.toml"})
            )
            .role_matches
        );
        for (name, class) in [
            (
                json!({"type":"enterpriseManaged","id":"i","name":"n"}),
                LayerClass::EnterpriseManaged,
            ),
            (
                json!({"type":"mdm","domain":"d","key":"k"}),
                LayerClass::Mdm,
            ),
            (
                json!({"type":"legacyManagedConfigTomlFromFile","file":"/etc/codex/managed_config.toml"}),
                LayerClass::LegacyManagedFile,
            ),
            (
                json!({"type":"legacyManagedConfigTomlFromMdm"}),
                LayerClass::LegacyManagedMdm,
            ),
            (
                json!({"type":"packagedDefaults","file":"/bin/codex"}),
                LayerClass::PackagedDefaults,
            ),
            (json!({"type":"future"}), LayerClass::Unknown),
            (
                json!({"type":"sessionFlags","secret":"sk"}),
                LayerClass::Unknown,
            ),
            (
                json!({"type":"user","file":"/role/home/config.toml","extra":1}),
                LayerClass::Unknown,
            ),
        ] {
            let l = at(0, name);
            assert!(l.class == class && !l.role_matches);
        }
        // Unknown layer members or a non-string disabled reason refuse the whole projection.
        let mut extra = stack();
        extra["layers"][0]["secret"] = json!("sk");
        assert!(layers(&extra, Some(&roles())).is_none());
        let mut reason = stack();
        reason["layers"][1]["disabledReason"] = json!(true);
        assert!(layers(&reason, Some(&roles())).is_none());
        // Without assigned home/cwd roles only the fixed system path and flags can match.
        let unassigned = layers(&stack(), None).unwrap();
        assert!(unassigned
            .entries
            .iter()
            .all(|l| l.role_matches
                == matches!(l.class, LayerClass::SessionFlags | LayerClass::System)));
    }
    #[test]
    fn origins_must_name_an_enabled_layer_and_version() {
        use serde_json::json;
        for origin in [
            json!({"name":{"type":"project","dotCodexFolder":"/role/project/.codex"},"version":"sk-sentinel-version"}),
            json!({"name":{"type":"user","file":"/role/home/config.toml"},"version":"other"}),
            json!({"name":{"type":"user","file":"/role/home/config.toml"},"version":"sk-sentinel-version","x":1}),
        ] {
            let mut v = stack();
            v["origins"]["model"] = origin;
            assert!(!layers(&v, Some(&roles())).unwrap().origins_enabled);
        }
    }
    #[test]
    fn granular_projection_requires_all_flags_and_rejects_unknown_fields() {
        let value = serde_json::json!({"granular":{"sandbox_approval":true,"rules":false,"mcp_elicitations":true,"request_permissions":false,"skill_approval":true}});
        let full = approval(&value).unwrap();
        assert_ne!(full, ThreadApproval::OnRequest);
        for key in [
            "sandbox_approval",
            "rules",
            "mcp_elicitations",
            "request_permissions",
            "skill_approval",
        ] {
            let mut missing = value.clone();
            missing["granular"].as_object_mut().unwrap().remove(key);
            assert!(approval(&missing).is_none());
        }
        let mut extra = value;
        extra["granular"]["secret"] = Value::String("sentinel".into());
        assert!(approval(&extra).is_none());
    }
}
