use super::*;
use crate::{
    depth_evidence::payload::{Package, PackageOrigin},
    git_inputs::GitInputs,
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn resolve(
    root: &Path,
    view: &mut GitInputs,
    metadata: &Value,
    compiled: &BTreeSet<String>,
) -> Result<DependencyGraph, String> {
    let packages = metadata["packages"]
        .as_array()
        .ok_or("missing Cargo packages")?;
    let nodes = metadata["resolve"]["nodes"]
        .as_array()
        .ok_or("missing Cargo resolved graph")?;
    let root_package = packages
        .iter()
        .find(|p| p["name"] == "unified-agent-api-wrapper-events")
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
    if &seen != compiled {
        return Err("resolved/compiled dependency populations disagree".into());
    }
    if node_by_id[&root_id.as_str()]["features"]
        .as_array()
        .ok_or("missing root features")?
        .iter()
        .any(|f| f != "default")
    {
        return Err("unsupported root features".into());
    }
    let lock_raw = view.required("Cargo.lock")?;
    let lock: Value =
        toml_edit::de::from_str(std::str::from_utf8(&lock_raw).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    let lock = serde_json::to_value(lock).map_err(|e| e.to_string())?;
    let mut result = DependencyGraph {
        schema_version: 1,
        roots: vec![root_id],
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
        let dependencies = node_by_id[id.as_str()]["dependencies"]
            .as_array()
            .unwrap()
            .iter()
            .map(|d| d.as_str().unwrap().to_owned())
            .collect();
        result.packages.push(Package {
            id: id.clone(),
            origin,
            dependencies,
        });
    }
    for path in ["Cargo.toml", "Cargo.lock", CONTRACT, FIXTURE] {
        view.required(path)?;
        result.inputs.push(view.reference(path)?);
    }
    result
        .scopes
        .push("crates/xtask/tests/fixtures/depth-ci".into());
    for path in view.files_under("crates/xtask/tests/fixtures/depth-ci")? {
        result.inputs.push(view.reference(&path)?);
    }
    result.inputs.sort();
    result.inputs.dedup();
    result.scopes.sort();
    result.scopes.dedup();
    DependencyGraph::parse(&serde_json::to_string(&result).map_err(|e| e.to_string())?)
}
