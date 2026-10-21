use super::{effect, input::View, manifest, Effect};
use crate::runtime_advertising::{revision::Revision, source::normalized};
use crate::support_matrix::{
    render_agent_api_runtime_support_data_from_projection, RuntimeSupportProjectionRecord,
};
use quote::ToTokens;
use std::collections::{BTreeMap, BTreeSet};

fn families(view: &View) -> Result<Vec<String>, String> {
    let raw = view
        .source
        .read("crates/agent_api/src/runtime_support.rs")?;
    let file = syn::parse_file(&raw).map_err(|e| e.to_string())?;
    let functions: Vec<_> = file
        .items
        .iter()
        .filter_map(|i| {
            if let syn::Item::Fn(f) = i {
                (f.sig.ident == "runtime_family_records").then_some(f)
            } else {
                None
            }
        })
        .collect();
    if functions.len() != 1 {
        return Err("unsupported embedded runtime family selector".into());
    }
    let function = functions[0];
    let [syn::Stmt::Expr(syn::Expr::Match(m), None)] = function.block.stmts.as_slice() else {
        return Err("unsupported embedded family selector body".into());
    };
    let mut names = BTreeSet::new();
    for arm in &m.arms {
        match &arm.pat {
            syn::Pat::Lit(syn::ExprLit {
                lit: syn::Lit::Str(s),
                ..
            }) => {
                if !names.insert(s.value()) {
                    return Err("duplicate embedded family".into());
                }
            }
            syn::Pat::Wild(_) => {}
            _ => return Err("unsupported embedded family predicate".into()),
        }
    }
    let mut arms = String::new();
    for name in &names {
        if view.registry.find(name).is_none() {
            return Err("unknown embedded runtime family".into());
        }
        arms.push_str(&format!(
            "{name:?} => Some({}_RUNTIME_SUPPORT),",
            name.to_uppercase()
        ));
    }
    let expected=format!("fn runtime_family_records(runtime_family: &str) -> Option<&'static [EmbeddedRuntimeSupportRecord]> {{ match runtime_family {{ {arms} _ => None, }} }}");
    if normalized(&function.to_token_stream().to_string())? != normalized(&expected)? {
        return Err("unsupported embedded runtime family selector semantics".into());
    }
    let includes=file.items.iter().filter(|i|matches!(i,syn::Item::Macro(m) if m.to_token_stream().to_string()=="include ! (\"runtime_support_data.rs\") ;")).count();
    if includes!=1 || file.items.iter().any(|i|matches!(i,syn::Item::Const(c) if c.ident.to_string().ends_with("_RUNTIME_SUPPORT"))) {return Err("unsupported embedded runtime data binding".into());}
    Ok(names.into_iter().collect())
}
fn projection(view: &View) -> Result<BTreeMap<(String, String), Option<String>>, String> {
    let families = families(view)?;
    let mut projection = Vec::new();
    let mut values = BTreeMap::new();
    for family in &families {
        let root = &view
            .registry
            .find(family)
            .ok_or("missing runtime family root")?
            .manifest_root;
        for target in manifest::targets(view, root)? {
            let pointer =
                manifest::pointer_path(view, root, "latest_validated_by_target", Some(&target))?;
            let version = manifest::pointer(view, &pointer)?;
            if let Some(version) = &version {
                let path = format!("{root}/versions/{version}.json");
                let metadata = view.json(&path)?;
                if metadata["semantic_version"] != *version || metadata["schema_version"] != 1 {
                    return Err(format!("{path}: invalid embedded runtime version"));
                }
            }
            values.insert((family.clone(), target.clone()), version.clone());
            projection.push(RuntimeSupportProjectionRecord {
                runtime_family: family.clone(),
                target_triple: target,
                latest_validated: version,
            });
        }
    }
    projection.sort_by(|a, b| {
        (&a.runtime_family, &a.target_triple).cmp(&(&b.runtime_family, &b.target_triple))
    });
    let names: Vec<_> = families.iter().map(String::as_str).collect();
    let expected = render_agent_api_runtime_support_data_from_projection(&projection, &names);
    let path = crate::support_matrix::AGENT_API_RUNTIME_SUPPORT_DATA_OUTPUT_PATH;
    if normalized(&view.source.read(path)?)? != normalized(&expected)? {
        return Err(format!(
            "{path}: differs from its revision's runtime projection"
        ));
    }
    Ok(values)
}
pub(super) fn support(
    before: &View,
    after: &View,
    effects: &mut BTreeSet<Effect>,
) -> Result<(), String> {
    let old = projection(before)?;
    let now = projection(after)?;
    for ((agent, target), version) in now {
        if agent != after.entry.agent_id {
            continue;
        }
        if let Some(version) = version {
            if old.get(&(agent, target.clone())).and_then(Option::as_ref) != Some(&version) {
                effect(
                    effects,
                    &version,
                    Some(&target),
                    crate::support_matrix::AGENT_API_RUNTIME_SUPPORT_DATA_OUTPUT_PATH,
                    "embedded runtime support",
                );
            }
        }
    }
    Ok(())
}
/// Recorded target labels are interpreted explicitly; gated-label grammar remains D5's.
fn host(target: &str) -> Result<(&str, &str), String> {
    match target {
        "x86_64-unknown-linux-musl" | "x86_64-unknown-linux-gnu" | "linux-x64" => {
            Ok(("linux", "x86_64"))
        }
        "aarch64-unknown-linux-musl" | "aarch64-unknown-linux-gnu" => Ok(("linux", "aarch64")),
        "aarch64-apple-darwin" | "darwin-arm64" => Ok(("macos", "aarch64")),
        "x86_64-pc-windows-msvc" | "win32-x64" => Ok(("windows", "x86_64")),
        _ => Err(format!("unresolved advertising target label {target}")),
    }
}
pub(super) fn advertising(
    before: &View,
    after: &View,
    effects: &mut BTreeSet<Effect>,
) -> Result<(), String> {
    let old = Revision::load(&before.source)?;
    let now = Revision::load(&after.source)?;
    for added in now.additions(&old)? {
        if added.agent != after.entry.agent_id {
            continue;
        }
        for (version, record) in &after.records {
            if !record
                .policy
                .selection
                .enrollment
                .capabilities
                .iter()
                .any(|c| c.id == added.capability)
            {
                continue;
            }
            for target in crate::support_matrix::depth::covered_targets(record) {
                if host(target)? == (added.os.as_str(), added.arch.as_str()) {
                    effect(
                        effects,
                        version,
                        Some(target),
                        "crates/agent_api/src/runtime_advertising/data.rs",
                        &format!("runtime advertising {}", added.capability),
                    );
                }
            }
        }
    }
    Ok(())
}
