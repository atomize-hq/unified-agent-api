use super::{
    projection::AGENTS,
    source::{self, Source},
};
use quote::ToTokens;
use syn::{Item, ItemImpl};

// Fixed supported executable semantics, not values read from candidate data or include_str!.
const EVALUATOR: &str = "b19816053d4b6d6efce356ca5a13e5f364fb6916816e16e5114ba7a752c2ab76";
const ADAPTER: &str = "8e186919375728c3d57dbddf34c86da100f1c8d57cdb2f29c7a58b16c281658f";

// The sole approved builder wraps the application callback and returns the same
// backend. It cannot replace config or select different advertising flags.
const CODEX_HANDLER_BUILDER: &str = r#"
    pub fn with_interaction_handler<F, Fut>(mut self, handler: F) -> Self
    where
        F: Fn(CodexInteractionRequest) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<CodexInteractionReply, CodexInteractionError>>
            + Send + 'static,
    {
        self.interaction_handler = Some(interaction::Handler(std::sync::Arc::new(move |request| {
            Box::pin(handler(request))
        })));
        self
    }
"#;

pub(super) fn check(source: &Source) -> Result<(), String> {
    for (path, expected) in [
        (
            "crates/agent_api/src/runtime_advertising/evaluator.rs",
            EVALUATOR,
        ),
        ("crates/agent_api/src/runtime_advertising/mod.rs", ADAPTER),
    ] {
        let actual = source::fingerprint(&source.read(path)?)?;
        if actual != expected {
            return Err(format!("{path}: unsupported semantics ({actual})"));
        }
    }
    let lib = parse(source, "crates/agent_api/src/lib.rs")?;
    module(
        &lib,
        "runtime_advertising",
        "#[allow(dead_code)] mod runtime_advertising;",
    )?;
    module(&lib, "backends", "pub mod backends;")?;
    let backends = parse(source, "crates/agent_api/src/backends/mod.rs")?;
    for agent in AGENTS {
        module(
            &backends,
            agent,
            &format!("#[cfg(feature = {agent:?})] pub mod {agent};"),
        )?;
        backend(source, agent)?;
    }
    super::graph::check(source)
}

pub(super) fn parse(source: &Source, path: &str) -> Result<syn::File, String> {
    let file = syn::parse_file(&source.read(path)?).map_err(|e| format!("{path}: {e}"))?;
    for attr in &file.attrs {
        if !["doc", "allow", "warn", "deny", "forbid"]
            .iter()
            .any(|name| attr.path().is_ident(name))
        {
            return Err(format!("{path}: unsupported crate attribute"));
        }
    }
    for item in &file.items {
        match item {
            Item::ExternCrate(_) => return Err(format!("{path}: unsupported extern crate")),
            Item::Use(import) => {
                // Wildcards/custom standard derives could change resolution in guarded descendants.
                let mut bad = false;
                fn scan(tree: &syn::UseTree, bad: &mut bool) {
                    match tree {
                        syn::UseTree::Glob(_) => *bad = true,
                        syn::UseTree::Rename(r) => {
                            if ["Default", "Clone", "Debug", "std"]
                                .contains(&r.rename.to_string().as_str())
                            {
                                *bad = true;
                            }
                        }
                        syn::UseTree::Name(n) => {
                            if ["Default", "Clone", "Debug", "std"]
                                .contains(&n.ident.to_string().as_str())
                            {
                                *bad = true;
                            }
                        }
                        syn::UseTree::Path(p) => scan(&p.tree, bad),
                        syn::UseTree::Group(g) => {
                            for t in &g.items {
                                scan(t, bad);
                            }
                        }
                    }
                }
                scan(&import.tree, &mut bad);
                if bad {
                    return Err(format!("{path}: unsupported advertising name resolution"));
                }
            }
            _ => {}
        }
    }
    Ok(file)
}

fn module(file: &syn::File, name: &str, expected: &str) -> Result<(), String> {
    let found: Vec<_> = file
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Mod(m) if m.ident == name => Some(m),
            _ => None,
        })
        .collect();
    if found.len() != 1 {
        return Err(format!("{name}: missing or ambiguous module"));
    }
    source::same(found[0], expected, name)
}

fn type_name(agent: &str) -> &'static str {
    match agent {
        "codex" => "CodexBackend",
        "claude_code" => "ClaudeCodeBackend",
        "aider" => "AiderBackend",
        "gemini_cli" => "GeminiCliBackend",
        "opencode" => "OpencodeBackend",
        _ => unreachable!(),
    }
}

fn no_attrs(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().all(|a| a.path().is_ident("doc"))
}

fn binding(file: &syn::File, name: &str, expected: &str) -> Result<(), String> {
    fn walk(tree: &syn::UseTree, prefix: &str, found: &mut Vec<(String, String)>) {
        match tree {
            syn::UseTree::Path(p) => walk(&p.tree, &format!("{prefix}{}::", p.ident), found),
            syn::UseTree::Name(n) => {
                found.push((n.ident.to_string(), format!("{prefix}{}", n.ident)))
            }
            syn::UseTree::Rename(r) => {
                found.push((r.rename.to_string(), format!("{prefix}{}", r.ident)))
            }
            syn::UseTree::Group(g) => {
                for item in &g.items {
                    walk(item, prefix, found);
                }
            }
            syn::UseTree::Glob(_) => {}
        }
    }
    let mut imports = Vec::new();
    for item in &file.items {
        if let Item::Use(item) = item {
            walk(&item.tree, "", &mut imports);
        }
    }
    let paths: Vec<_> = imports
        .iter()
        .filter(|(bound, _)| bound == name)
        .map(|(_, path)| path.as_str())
        .collect();
    if paths != [expected] {
        return Err(format!("{name}: unsupported identity/type import binding"));
    }
    Ok(())
}

fn backend(source: &Source, agent: &str) -> Result<(), String> {
    let name = type_name(agent);
    let parent = parse(
        source,
        &format!("crates/agent_api/src/backends/{agent}/mod.rs"),
    )?;
    module(&parent, "backend", "mod backend;")?;
    // Select between two finite supported Codex shapes. Both still undergo exact
    // storage/constructor checks; unknown extra fields or methods remain refused.
    let codex_handler = agent == "codex" && parent.items.iter().any(|item| {
        matches!(item, Item::Struct(s) if s.ident == name && s.fields.iter().any(|field| field.ident.as_ref().is_some_and(|id| id == "interaction_handler")))
    });
    let config = format!("{name}Config");
    let fields=match agent {
        "codex"=>"pub binary: Option<PathBuf>, pub codex_home: Option<PathBuf>, pub model: Option<String>, pub default_timeout: Option<Duration>, pub default_working_dir: Option<PathBuf>, pub env: BTreeMap<String,String>, pub allow_mcp_write: bool, pub allow_external_sandbox_exec: bool,",
        "claude_code"=>"pub binary: Option<PathBuf>, pub claude_home: Option<PathBuf>, pub default_timeout: Option<Duration>, pub default_working_dir: Option<PathBuf>, pub env: BTreeMap<String,String>, pub allow_mcp_write: bool, pub allow_external_sandbox_exec: bool,",
        _=>"pub binary: Option<PathBuf>, pub default_timeout: Option<Duration>, pub env: BTreeMap<String,String>,",
    };
    let mut config_count = 0;
    let mut storage_count = 0;
    let mut constructor_count = 0;
    let mut builder_count = 0;
    for item in &parent.items {
        if let Item::Struct(s) = item {
            if s.ident == config {
                config_count += 1;
                source::same(
                    s,
                    &format!("#[derive(Clone,Debug,Default)] pub struct {config} {{ {fields} }}"),
                    &config,
                )?;
            }
            if s.ident == name {
                storage_count += 1;
                let (derive, extra) = match agent {
                    "claude_code" => ("", ", allow_flag_preflight: Arc<OnceCell<bool>>"),
                    "codex" if codex_handler => {
                        ("", ", interaction_handler: Option<interaction::Handler>")
                    }
                    "codex" => ("", ""),
                    _ => ("#[derive(Clone,Debug)]", ""),
                };
                source::same(
                    s,
                    &format!("{derive} pub struct {name} {{ config: {config}{extra}, }}"),
                    name,
                )?;
            }
        }
        if let Item::Impl(imp) = item {
            if imp.self_ty.to_token_stream().to_string() == name && imp.trait_.is_none() {
                if !no_attrs(&imp.attrs) || !imp.generics.params.is_empty() {
                    return Err(format!("{name}: unsupported constructor scope"));
                }
                for item in &imp.items {
                    if let syn::ImplItem::Fn(method) = item {
                        if codex_handler && method.sig.ident == "with_interaction_handler" {
                            builder_count += 1;
                            source::same(method, CODEX_HANDLER_BUILDER, "handler builder")?;
                            continue;
                        }
                        // Every accepted producer preserves the advertising config exactly.
                        if method.sig.ident != "new" {
                            return Err(format!("{name}: unsupported inherent method"));
                        }
                        constructor_count += 1;
                        let storage = if agent == "claude_code" {
                            "Self { config, allow_flag_preflight: Arc::new(OnceCell::new()), }"
                        } else if codex_handler {
                            "Self { config, interaction_handler: None, }"
                        } else {
                            "Self { config }"
                        };
                        source::same(
                            method,
                            &format!("pub fn new(config: {config}) -> Self {{ {storage} }}"),
                            "constructor",
                        )?;
                    } else {
                        return Err(format!("{name}: unsupported inherent item"));
                    }
                }
            }
        }
    }
    if builder_count != usize::from(codex_handler) {
        return Err(format!("{name}: missing or ambiguous handler builder"));
    }
    if (config_count, storage_count, constructor_count) != (1, 1, 1) {
        return Err(format!(
            "{name}: missing or ambiguous config/storage/constructor"
        ));
    }
    let implementation = parse(
        source,
        &format!("crates/agent_api/src/backends/{agent}/backend.rs"),
    )?;
    for name in [
        "AgentWrapperBackend",
        "AgentWrapperCapabilities",
        "AgentWrapperKind",
    ] {
        binding(&implementation, name, &format!("crate::{name}"))?;
    }
    binding(&implementation, name, &format!("super::{name}"))?;
    if agent == "claude_code" {
        binding(&implementation, "AGENT_KIND", "super::AGENT_KIND")?;
    }
    let implementations: Vec<&ItemImpl> = implementation
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Impl(i)
                if i.self_ty.to_token_stream().to_string() == name
                    && i.trait_.as_ref().is_some_and(|(_, path, _)| {
                        path.to_token_stream().to_string() == "AgentWrapperBackend"
                    }) =>
            {
                Some(i)
            }
            _ => None,
        })
        .collect();
    if implementations.len() != 1 {
        return Err(format!(
            "{name}: missing or ambiguous runtime trait implementation"
        ));
    }
    let imp = implementations[0];
    if !no_attrs(&imp.attrs) || !imp.generics.params.is_empty() || imp.unsafety.is_some() {
        return Err(format!("{name}: unsupported trait scope"));
    }
    let mut caps = 0;
    let mut kinds = 0;
    for item in &imp.items {
        if let syn::ImplItem::Fn(method) = item {
            if method.sig.ident == "capabilities" {
                caps += 1;
                let args = if matches!(agent, "codex" | "claude_code") {
                    "crate::runtime_advertising::evaluator::ConfigValues { allow_mcp_write: self.config.allow_mcp_write, allow_external_sandbox_exec: self.config.allow_external_sandbox_exec, }"
                } else {
                    "crate::runtime_advertising::evaluator::ConfigValues::default()"
                };
                source::same(method,&format!("fn capabilities(&self) -> AgentWrapperCapabilities {{ AgentWrapperCapabilities {{ ids: crate::runtime_advertising::evaluator::evaluate(&crate::runtime_advertising::data::{}, crate::runtime_advertising::host(), {args},), }} }}",agent.to_uppercase()),"capabilities delegate")?;
            }
            if method.sig.ident == "kind" {
                kinds += 1;
                let value = match agent {
                    "codex" => format!("{agent:?}"),
                    "claude_code" => "AGENT_KIND".into(),
                    _ => "super::AGENT_KIND".into(),
                };
                source::same(method,&format!("fn kind(&self) -> AgentWrapperKind {{ AgentWrapperKind({value}.to_string()) }}"),"runtime kind")?;
                if agent != "codex" {
                    let constants: Vec<_> = parent
                        .items
                        .iter()
                        .filter_map(|item| match item {
                            Item::Const(c) if c.ident == "AGENT_KIND" => Some(c),
                            _ => None,
                        })
                        .collect();
                    if constants.len() != 1 {
                        return Err("missing or ambiguous runtime identity".into());
                    }
                    source::same(
                        constants[0],
                        &format!("const AGENT_KIND: &str = {agent:?};"),
                        "runtime identity",
                    )?;
                }
            }
        } else {
            return Err(format!("{name}: unsupported runtime trait item"));
        }
    }
    if (caps, kinds) != (1, 1) {
        return Err(format!("{name}: missing guarded runtime method"));
    }
    Ok(())
}

#[cfg(test)]
mod native_bridge_tests {
    use super::{check, Source};
    use std::{collections::BTreeMap, path::Path};
    #[test]
    fn approved_native_handler_shape_preserves_real_source_capture() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        check(&Source::workspace(root).unwrap()).unwrap();
    }
    #[test]
    fn legacy_shape_and_only_matching_constructor_builder_combinations_are_supported() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let path = "crates/agent_api/src/backends/codex/mod.rs";
        let raw = std::fs::read_to_string(root.join(path)).unwrap();
        let start = raw.find("    /// Register an application handler").unwrap();
        let end = raw.find("\n}\n\npub use interaction").unwrap();
        let mut without_builder = raw.clone();
        without_builder.replace_range(start..end, "");
        let source = Source::proposed(
            root,
            BTreeMap::from([(path.into(), without_builder.clone())]),
        )
        .unwrap();
        assert!(check(&source).unwrap_err().contains("handler builder"));
        let legacy = without_builder
            .replace(
                "    interaction_handler: Option<interaction::Handler>,\n",
                "",
            )
            .replace(
                "Self {\n            config,\n            interaction_handler: None,\n        }",
                "Self { config }",
            );
        assert!(!legacy.contains("interaction_handler:"));
        check(&Source::proposed(root, BTreeMap::from([(path.into(), legacy)])).unwrap()).unwrap();
        let mismatched = raw.replace(
            "    interaction_handler: Option<interaction::Handler>,\n",
            "",
        );
        assert!(check(
            &Source::proposed(root, BTreeMap::from([(path.into(), mismatched)])).unwrap()
        )
        .is_err());
    }

    #[test]
    fn altered_native_handler_storage_constructor_and_builder_are_refused() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let path = "crates/agent_api/src/backends/codex/mod.rs";
        let raw = std::fs::read_to_string(root.join(path)).unwrap();
        for (old, new, expected) in [
            (
                "interaction_handler: Option<interaction::Handler>",
                "interaction_handler: Option<String>",
                "CodexBackend",
            ),
            (
                "interaction_handler: None",
                "interaction_handler: Some(make_handler())",
                "constructor",
            ),
            (
                "            config,",
                "            config: CodexBackendConfig::default(),",
                "constructor",
            ),
            (
                "Box::pin(handler(request))",
                "Box::pin(handler(alter(request)))",
                "handler builder",
            ),
            (
                "        self\n    }\n}",
                "        Self::new(CodexBackendConfig::default())\n    }\n}",
                "handler builder",
            ),
            (
                "    config: CodexBackendConfig,",
                "    config: CodexBackendConfig, unauthorized: bool,",
                "CodexBackend",
            ),
            (
                "impl CodexBackend {",
                "impl CodexBackend { pub fn unauthorized() -> Self { todo!() }",
                "inherent method",
            ),
        ] {
            assert!(raw.contains(old), "missing discriminator {old}");
            let source = Source::proposed(
                root,
                BTreeMap::from([(path.into(), raw.replacen(old, new, 1))]),
            )
            .unwrap();
            let error = check(&source).unwrap_err();
            assert!(error.contains(expected), "{old}: {error}");
        }
    }
}
