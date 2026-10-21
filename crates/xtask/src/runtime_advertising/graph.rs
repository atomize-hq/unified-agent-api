//! Guard declarations across every reachable runtime module, not only the delegate's file.
use super::{conformance, source::Source};
use quote::ToTokens;
use std::{collections::BTreeSet, path::PathBuf};
use syn::{visit::Visit, Attribute, Item};

const TYPES: [&str; 10] = [
    "AiderBackend",
    "AiderBackendConfig",
    "ClaudeCodeBackend",
    "ClaudeCodeBackendConfig",
    "CodexBackend",
    "CodexBackendConfig",
    "GeminiCliBackend",
    "GeminiCliBackendConfig",
    "OpencodeBackend",
    "OpencodeBackendConfig",
];
const MACROS: &[&str] = &[
    "assert",
    "assert_eq",
    "assert_ne",
    "debug_assert",
    "debug_assert_eq",
    "debug_assert_ne",
    "cfg",
    "concat",
    "env",
    "option_env",
    "file",
    "line",
    "column",
    "stringify",
    "format",
    "format_args",
    "vec",
    "matches",
    "panic",
    "unreachable",
    "todo",
    "unimplemented",
    "print",
    "println",
    "eprint",
    "eprintln",
    "write",
    "writeln",
    "include_str",
    "include_bytes",
];

pub(super) fn check(source: &Source) -> Result<(), String> {
    let mut graph = Graph {
        source,
        path: String::new(),
        directory: PathBuf::new(),
        nested: 0,
        visited: BTreeSet::new(),
        error: None,
    };
    graph.file(
        "crates/agent_api/src/lib.rs",
        PathBuf::from("crates/agent_api/src"),
    );
    graph.error.map_or(Ok(()), Err)
}

struct Graph<'a> {
    source: &'a Source,
    path: String,
    directory: PathBuf,
    nested: usize,
    visited: BTreeSet<String>,
    error: Option<String>,
}

fn attributes(item: &Item) -> &[Attribute] {
    match item {
        Item::Const(x) => &x.attrs,
        Item::Enum(x) => &x.attrs,
        Item::ExternCrate(x) => &x.attrs,
        Item::Fn(x) => &x.attrs,
        Item::ForeignMod(x) => &x.attrs,
        Item::Impl(x) => &x.attrs,
        Item::Macro(x) => &x.attrs,
        Item::Mod(x) => &x.attrs,
        Item::Static(x) => &x.attrs,
        Item::Struct(x) => &x.attrs,
        Item::Trait(x) => &x.attrs,
        Item::TraitAlias(x) => &x.attrs,
        Item::Type(x) => &x.attrs,
        Item::Union(x) => &x.attrs,
        Item::Use(x) => &x.attrs,
        _ => &[],
    }
}

fn contains_type(stream: proc_macro2::TokenStream) -> bool {
    stream.into_iter().any(|t| match t {
        proc_macro2::TokenTree::Ident(id) => TYPES.contains(&id.to_string().as_str()),
        proc_macro2::TokenTree::Group(g) => contains_type(g.stream()),
        _ => false,
    })
}

impl Graph<'_> {
    fn fail(&mut self, message: &str) {
        if self.error.is_none() {
            self.error = Some(format!(
                "{}: unsupported runtime graph {message}",
                self.path
            ));
        }
    }
    fn file(&mut self, path: &str, directory: PathBuf) {
        if self.error.is_some() {
            return;
        }
        if !self.visited.insert(path.into()) {
            self.fail("module cycle or duplicate source");
            return;
        }
        let file = match conformance::parse(self.source, path) {
            Ok(file) => file,
            Err(e) => {
                self.error = Some(e);
                return;
            }
        };
        let old_path = std::mem::replace(&mut self.path, path.into());
        let old_directory = std::mem::replace(&mut self.directory, directory);
        let old_nested = std::mem::replace(&mut self.nested, 0);
        self.visit_file(&file);
        self.path = old_path;
        self.directory = old_directory;
        self.nested = old_nested;
    }
    fn alias(&mut self, stream: proc_macro2::TokenStream) {
        if contains_type(stream) {
            self.fail("guarded type alias");
        }
    }
}

impl<'ast> Visit<'ast> for Graph<'_> {
    fn visit_item(&mut self, item: &'ast Item) {
        if self.error.is_some() {
            return;
        }
        // This exact condition is unconditionally excluded from production, independently of host/features.
        if attributes(item)
            .iter()
            .any(|a| a.to_token_stream().to_string() == "# [cfg (test)]")
        {
            return;
        }
        if matches!(item, Item::ExternCrate(_) | Item::Verbatim(_)) {
            self.fail("item expansion");
            return;
        }
        syn::visit::visit_item(self, item);
    }
    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        for attr in &item.attrs {
            self.visit_attribute(attr);
        }
        if ["std", "tokio", "serde", "serde_json", "thiserror"]
            .contains(&item.ident.to_string().as_str())
        {
            self.fail("standard/tooling namespace replacement");
            return;
        }
        let directory = self.directory.join(item.ident.to_string());
        if let Some((_, items)) = &item.content {
            let old = std::mem::replace(&mut self.directory, directory);
            self.nested += 1;
            for item in items {
                self.visit_item(item);
            }
            self.nested -= 1;
            self.directory = old;
        } else {
            let flat = directory
                .with_extension("rs")
                .to_string_lossy()
                .into_owned();
            let nested = directory.join("mod.rs").to_string_lossy().into_owned();
            match (self.source.optional(&flat), self.source.optional(&nested)) {
                (Ok(Some(_)), Ok(None)) => self.file(&flat, directory),
                (Ok(None), Ok(Some(_))) => self.file(&nested, directory),
                (Err(e), _) | (_, Err(e)) => self.error = Some(e),
                _ => self.fail("missing or ambiguous module source"),
            }
        }
    }
    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        if contains_type(item.self_ty.to_token_stream()) {
            let name = item.self_ty.to_token_stream().to_string();
            let agent = match name.as_str() {
                "AiderBackend" => "aider",
                "ClaudeCodeBackend" => "claude_code",
                "CodexBackend" => "codex",
                "GeminiCliBackend" => "gemini_cli",
                "OpencodeBackend" => "opencode",
                _ => "",
            };
            let stem = format!("crates/agent_api/src/backends/{agent}");
            let trait_name = item
                .trait_
                .as_ref()
                .map(|(_, p, _)| p.to_token_stream().to_string());
            let accepted = self.nested == 0
                && !agent.is_empty()
                && match trait_name.as_deref() {
                    None => self.path == format!("{stem}/mod.rs"),
                    Some("AgentWrapperBackend") => self.path == format!("{stem}/backend.rs"),
                    Some("BackendHarnessAdapter") => {
                        ["aider", "gemini_cli", "opencode"].contains(&agent)
                            && self.path == format!("{stem}/harness.rs")
                    }
                    _ => false,
                };
            if !accepted {
                self.fail("guarded type implementation/producer");
                return;
            }
        }
        syn::visit::visit_item_impl(self, item);
    }
    fn visit_item_type(&mut self, item: &'ast syn::ItemType) {
        self.alias(item.ty.to_token_stream());
        syn::visit::visit_item_type(self, item);
    }
    fn visit_impl_item_type(&mut self, item: &'ast syn::ImplItemType) {
        self.alias(item.ty.to_token_stream());
        syn::visit::visit_impl_item_type(self, item);
    }
    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        fn imports(tree: &syn::UseTree, prefix: &str, out: &mut Vec<(String, String, bool)>) {
            match tree {
                syn::UseTree::Path(p) => imports(&p.tree, &format!("{prefix}{}::", p.ident), out),
                syn::UseTree::Name(n) => {
                    out.push((n.ident.to_string(), format!("{prefix}{}", n.ident), false))
                }
                syn::UseTree::Rename(r) => out.push((
                    r.rename.to_string(),
                    format!("{prefix}{}", r.ident),
                    TYPES.contains(&r.ident.to_string().as_str()),
                )),
                syn::UseTree::Group(g) => {
                    for tree in &g.items {
                        imports(tree, prefix, out);
                    }
                }
                syn::UseTree::Glob(_) => out.push(("*".into(), "*".into(), false)),
            }
        }
        let mut bindings = Vec::new();
        imports(&item.tree, "", &mut bindings);
        for (name, path, guarded_alias) in bindings {
            if guarded_alias {
                self.fail("guarded type import alias");
            }
            if (MACROS.contains(&name.as_str())
                || [
                    "*",
                    "Default",
                    "Clone",
                    "Copy",
                    "Debug",
                    "Eq",
                    "PartialEq",
                    "Ord",
                    "PartialOrd",
                    "Hash",
                    "std",
                    "tokio",
                    "serde",
                    "serde_json",
                    "thiserror",
                    "select",
                    "pin",
                    "json",
                ]
                .contains(&name.as_str()))
                && !(name == "env" && path == "std::env")
            {
                self.fail("macro or tooling import rebinding");
            }
        }
        syn::visit::visit_item_use(self, item);
    }
    fn visit_block(&mut self, block: &'ast syn::Block) {
        self.nested += 1;
        syn::visit::visit_block(self, block);
        self.nested -= 1;
    }
    fn visit_item_macro(&mut self, item: &'ast syn::ItemMacro) {
        if self.path == "crates/agent_api/src/runtime_support.rs"
            && item.to_token_stream().to_string() == "include ! (\"runtime_support_data.rs\") ;"
        {
            self.file(
                "crates/agent_api/src/runtime_support_data.rs",
                self.directory.clone(),
            );
        } else {
            self.fail("item macro expansion");
        }
    }
    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        let name = mac.path.to_token_stream().to_string();
        if !MACROS.contains(&name.as_str())
            && !["tokio :: select", "tokio :: pin", "serde_json :: json"].contains(&name.as_str())
        {
            self.fail("macro expansion");
            return;
        }
        fn declaration(tokens: proc_macro2::TokenStream) -> bool {
            use proc_macro2::TokenTree;
            let tokens: Vec<_> = tokens.into_iter().collect();
            for (i, token) in tokens.iter().enumerate() {
                match token {
                    TokenTree::Ident(id)
                        if [
                            "impl",
                            "mod",
                            "use",
                            "extern",
                            "struct",
                            "enum",
                            "union",
                            "trait",
                            "type",
                            "macro_rules",
                            "include",
                        ]
                        .contains(&id.to_string().as_str()) =>
                    {
                        return true
                    }
                    TokenTree::Group(g) if declaration(g.stream()) => return true,
                    TokenTree::Punct(p)
                        if p.as_char() == '!'
                            && matches!(tokens.get(i + 1), Some(TokenTree::Group(_))) =>
                    {
                        let mut start = i;
                        if start > 0 && matches!(&tokens[start - 1], TokenTree::Ident(_)) {
                            start -= 1;
                            while start >= 3
                                && matches!(&tokens[start-1], TokenTree::Punct(p) if p.as_char()==':')
                                && matches!(&tokens[start-2], TokenTree::Punct(p) if p.as_char()==':')
                                && matches!(&tokens[start - 3], TokenTree::Ident(_))
                            {
                                start -= 3;
                            }
                        }
                        let head = tokens[start..i]
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>()
                            .join("");
                        if !head.is_empty()
                            && !MACROS.contains(&head.as_str())
                            && !["tokio::select", "tokio::pin", "serde_json::json"]
                                .contains(&head.as_str())
                        {
                            return true;
                        }
                    }
                    _ => {}
                }
            }
            false
        }
        if declaration(mac.tokens.clone()) {
            self.fail("declaration inside opaque macro");
        }
    }
    fn visit_attribute(&mut self, attr: &'ast Attribute) {
        let name = attr.path().to_token_stream().to_string();
        if name == "derive" {
            let parsed = attr.parse_args_with(
                syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
            );
            let Ok(paths) = parsed else {
                self.fail("derive syntax");
                return;
            };
            for path in paths {
                if ![
                    "Clone",
                    "Copy",
                    "Debug",
                    "Default",
                    "Eq",
                    "PartialEq",
                    "Ord",
                    "PartialOrd",
                    "Hash",
                    "serde :: Serialize",
                    "serde :: Deserialize",
                    "thiserror :: Error",
                ]
                .contains(&path.to_token_stream().to_string().as_str())
                {
                    self.fail("derive expansion");
                }
            }
        } else if ![
            "cfg", "doc", "allow", "warn", "deny", "forbid", "inline", "cold", "must_use", "error",
            "from", "source", "serde",
        ]
        .contains(&name.as_str())
        {
            self.fail("attribute expansion or alternate module path");
        }
    }
}
