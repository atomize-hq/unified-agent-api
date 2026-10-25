use super::*;
use crate::{
    depth_evidence::payload::{Package, PackageOrigin},
    git_inputs::GitInputs,
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn normalize(id: &str) -> String {
    if let Some((_, package)) = id
        .strip_prefix("path+file:")
        .and_then(|s| s.split_once('#'))
    {
        format!(
            "workspace:crates/{}#{package}",
            match package.split('@').next().unwrap() {
                "unified-agent-api" => "agent_api",
                "unified-agent-api-codex" => "codex",
                "unified-agent-api-wrapper-events" => "wrapper_events",
                "unified-agent-api-claude-code" => "claude_code",
                "unified-agent-api-aider" => "aider",
                "unified-agent-api-gemini-cli" => "gemini_cli",
                "unified-agent-api-opencode" => "opencode",
                other => other,
            }
        )
    } else {
        id.into()
    }
}

pub(super) fn resolve(
    root: &Path,
    view: &mut GitInputs,
    metadata: &Value,
    compiled: &BTreeSet<String>,
    profile: profile::Profile,
) -> Result<DependencyGraph, String> {
    let packages = metadata["packages"]
        .as_array()
        .ok_or("missing Cargo packages")?;
    let nodes = metadata["resolve"]["nodes"]
        .as_array()
        .ok_or("missing Cargo resolved graph")?;
    let root_package = packages
        .iter()
        .find(|p| p["name"] == profile.package())
        .ok_or("missing affected crate")?;
    let root_id = root_package["id"]
        .as_str()
        .ok_or("missing package id")?
        .to_owned();
    let by_id: BTreeMap<_, _> = packages
        .iter()
        .map(|p| Ok((p["id"].as_str().ok_or("invalid package id")?, p)))
        .collect::<Result<_, String>>()?;
    let node_by_id: BTreeMap<_, _> = nodes
        .iter()
        .map(|n| Ok((n["id"].as_str().ok_or("invalid node id")?, n)))
        .collect::<Result<_, String>>()?;
    let seen = expected(metadata, profile)?;
    if &seen != compiled {
        return Err("resolved/compiled dependency populations disagree".into());
    }
    let lock_raw = view.required("Cargo.lock")?;
    let lock: Value =
        toml_edit::de::from_str(std::str::from_utf8(&lock_raw).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    let lock = serde_json::to_value(lock).map_err(|e| e.to_string())?;
    let mut result = DependencyGraph {
        schema_version: 1,
        roots: vec![normalize(&root_id)],
        packages: vec![],
        inputs: vec![],
        scopes: vec![],
    };
    for id in &seen {
        let p = by_id.get(id.as_str()).ok_or("unknown compiled package")?;
        let manifest = Path::new(
            p["manifest_path"]
                .as_str()
                .ok_or("missing package manifest")?,
        );
        let directory = manifest.parent().ok_or("missing package directory")?;
        let origin = if p["source"].is_null() {
            let relative = directory
                .strip_prefix(root)
                .map_err(|_| "external path dependency is unsupported")?
                .to_str()
                .ok_or("non-UTF8 package path")?
                .to_owned();
            result.scopes.push(relative.clone());
            for path in view.files_under(&relative)? {
                result.inputs.push(view.reference(&path)?);
            }
            PackageOrigin::Local {
                directory: relative,
            }
        } else {
            let source = p["source"].as_str().ok_or("invalid package source")?;
            if !source.starts_with("registry+") {
                return Err("only pinned registry dependencies are supported".into());
            }
            let package = lock["package"]
                .as_array()
                .ok_or("invalid Cargo lock")?
                .iter()
                .find(|l| {
                    l["name"] == p["name"]
                        && l["version"] == p["version"]
                        && l["source"] == p["source"]
                })
                .ok_or("package missing from lock")?;
            let checksum = package["checksum"]
                .as_str()
                .ok_or("missing locked archive checksum")?;
            let cargo_home = std::env::var_os("CARGO_HOME")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| {
                    std::path::PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
                        .join(".cargo")
                });
            let cache = cargo_home.join("registry/cache");
            let archive_name = format!(
                "{}-{}.crate",
                p["name"].as_str().unwrap(),
                p["version"].as_str().unwrap()
            );
            let archives = std::fs::read_dir(cache)
                .map_err(|e| e.to_string())?
                .filter_map(Result::ok)
                .map(|d| d.path().join(&archive_name))
                .filter(|a| a.is_file())
                .collect::<Vec<_>>();
            if archives.len() != 1 {
                return Err("registry archive cannot be uniquely resolved".into());
            }
            output(
                Command::new("python3")
                    .arg(root.join(ARCHIVE))
                    .arg("registry")
                    .arg(&archives[0])
                    .arg(directory)
                    .arg(checksum),
            )?;
            PackageOrigin::Registry {
                source: source.into(),
                checksum: checksum.into(),
            }
        };
        let dependencies: Vec<String> = node_by_id[id.as_str()]["dependencies"]
            .as_array()
            .unwrap()
            .iter()
            .map(|d| d.as_str().unwrap().to_owned())
            .collect();
        result.packages.push(Package {
            id: normalize(id),
            origin,
            dependencies: dependencies
                .into_iter()
                .map(|id: String| normalize(&id))
                .collect(),
        });
    }
    for path in [
        "Cargo.toml",
        "Cargo.lock",
        profile.contract(),
        profile.fixture(),
    ] {
        view.required(path)?;
        result.inputs.push(view.reference(path)?);
    }
    result.scopes.push(profile.directory().into());
    for path in view.files_under(profile.directory())? {
        result.inputs.push(view.reference(&path)?);
    }
    if profile == profile::Profile::Core {
        let record = profile.record(
            std::str::from_utf8(&view.required(profile.fixture())?).map_err(|e| e.to_string())?,
        )?;
        for input in record.obligations.unwrap().acquired_inputs {
            result.inputs.push(view.reference(&input.path)?);
        }
    }
    result.inputs.sort();
    result.inputs.dedup();
    result.scopes.sort();
    result.scopes.dedup();
    result.packages.sort_by(|a, b| a.id.cmp(&b.id));
    let profile: Value = serde_json::from_slice(&view.required(&profile.build_profile())?)
        .map_err(|e| e.to_string())?;
    if profile["schema_version"] != 1
        || serde_json::to_value(&result.packages).map_err(|e| e.to_string())? != profile["packages"]
    {
        return Err("resolved package closure is outside the reviewed profile".into());
    }
    DependencyGraph::parse(&serde_json::to_string(&result).map_err(|e| e.to_string())?)
}

/// Resolve the complete selected closure before running any registry build script or proc macro.
pub(super) fn expected(
    metadata: &Value,
    profile: profile::Profile,
) -> Result<BTreeSet<String>, String> {
    let packages = metadata["packages"]
        .as_array()
        .ok_or("missing Cargo packages")?;
    let nodes = metadata["resolve"]["nodes"]
        .as_array()
        .ok_or("missing Cargo graph")?;
    let root_id = packages
        .iter()
        .find(|p| p["name"] == profile.package())
        .and_then(|p| p["id"].as_str())
        .ok_or("missing affected root")?
        .to_owned();
    let node_by_id: BTreeMap<_, _> = nodes
        .iter()
        .map(|n| Ok((n["id"].as_str().ok_or("invalid node")?, n)))
        .collect::<Result<_, String>>()?;
    let mut seen = BTreeSet::new();
    let mut todo = vec![root_id.clone()];
    while let Some(id) = todo.pop() {
        if seen.insert(id.clone()) {
            let node = node_by_id.get(id.as_str()).ok_or("unresolved Cargo node")?;
            for d in node["dependencies"]
                .as_array()
                .ok_or("missing dependencies")?
            {
                todo.push(d.as_str().ok_or("invalid dependency")?.to_owned());
            }
        }
    }
    Ok(seen)
}
