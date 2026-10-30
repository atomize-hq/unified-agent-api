//! Build-only reuse for the local native producer: no Actions header, no native execution.
use super::{
    collector::{identity, required, toolchain, BuildEnv},
    *,
};
use crate::git_inputs::GitInputs;
use crate::runtime_advertising::source::Source;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};
pub(crate) struct Built {
    // Owns the fresh closed build home/target for the lifetime of the native child.
    pub _target: tempfile::TempDir,
    pub executable: PathBuf,
    pub executable_sha256: String,
    pub build: Build,
    pub payloads: Vec<(FileRef, Vec<u8>)>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Units {
    features: BTreeMap<String, Value>,
    build_scripts: Vec<Value>,
}
const UNITS: &str = "crates/xtask/data/depth-native-build-units-v1.json";
pub(crate) fn build(
    root: &Path,
    prefix: &str,
    graph: &DependencyGraph,
    environment: &Environment,
    expected_toolchain: &FileRef,
    cancel: &std::sync::atomic::AtomicBool,
) -> Result<Built, String> {
    active(cancel)?;
    if *environment != {
        let mut e = super::collector::environment(profile::Profile::Core);
        e.features.clear();
        e
    } {
        return Err("unsupported native wrapper build target/features".into());
    }
    let mut view = GitInputs::new(root)?;
    for required in ["Cargo.toml", "Cargo.lock", UNITS, ARCHIVE]
        .into_iter()
        .chain(crate::depth_evidence::native::context::INPUTS)
    {
        if !graph.inputs.iter().any(|f| f.path == required) {
            return Err("native build helper/input lacks approved source coverage".into());
        }
    }

    DependencyGraph::parse(&serde_json::to_string(graph).map_err(|_| "invalid graph")?)?;
    let commit = String::from_utf8(output(
        Command::new("git")
            .arg("--no-replace-objects")
            .arg("-C")
            .arg(root)
            .args(["rev-parse", "HEAD"]),
    )?)
    .map_err(|_| "invalid build source")?
    .trim()
    .to_owned();
    let source = Source::tip(root, &commit)?;
    for file in &graph.inputs {
        view.required(&file.path)?;
        if view.reference(&file.path)? != *file
            || source.optional_bytes(&file.path)?.as_deref()
                != Some(view.required(&file.path)?.as_slice())
            || source.executable(&file.path)? != view.executable(&file.path)?
        {
            return Err("native build inputs differ from frozen source".into());
        }
    }
    for scope in &graph.scopes {
        let expected: BTreeSet<_> = graph
            .inputs
            .iter()
            .filter(|f| f.path.starts_with(&format!("{scope}/")))
            .map(|f| f.path.clone())
            .collect();
        if view
            .files_under(scope)?
            .into_iter()
            .collect::<BTreeSet<_>>()
            != expected
        {
            return Err("native build source inventory differs".into());
        }
    }
    if !graph.inputs.iter().any(|f| f.path == UNITS) {
        return Err("native build units lack approved source coverage".into());
    }
    let units: Units = serde_json::from_slice(&view.required(UNITS)?)
        .map_err(|_| "invalid reviewed native build units")?;
    let absent = [
        ".cargo/config",
        ".cargo/config.toml",
        "rust-toolchain",
        "rust-toolchain.toml",
        "crates/xtask/build.rs",
    ]
    .map(str::to_owned)
    .to_vec();
    for path in &absent {
        if view.optional(path)?.is_some() || source.optional_bytes(path)?.is_some() {
            return Err("unsupported native Cargo configuration".into());
        }
    }
    for parent in root.ancestors().skip(1) {
        for path in [".cargo/config", ".cargo/config.toml"] {
            if std::fs::symlink_metadata(parent.join(path)).is_ok() {
                return Err("unsupported ancestor Cargo configuration".into());
            }
        }
    }
    active(cancel)?;
    let env = BuildEnv::new(root, profile::Profile::Core)?;
    let metadata: Value = serde_json::from_str(&required(
        env.command(&env.cargo).args([
            "metadata",
            "--format-version",
            "1",
            "--locked",
            "--offline",
            "--no-default-features",
            "--filter-platform",
            TARGET,
        ]),
        16_777_216,
    )?)
    .map_err(|_| "invalid native build metadata")?;
    active(cancel)?;
    let tree = required(
        env.command(&env.cargo).args([
            "tree",
            "--locked",
            "--offline",
            "--no-default-features",
            "-p",
            "xtask",
            "--target",
            TARGET,
            "--prefix",
            "none",
            "--format",
            "{p}",
            "--edges",
            "normal,build,dev",
        ]),
        16_777_216,
    )?;
    let metadata = super::graph::selected(metadata, &tree)?;
    // Validate locked registry bytes and exact reviewed topology before build-time code runs.
    let expected = super::graph::expected_for(&metadata, "xtask")?;
    let observed = super::graph::resolve_for(root, &mut view, &metadata, &expected, "xtask")?;
    if observed.packages != graph.packages
        || observed.roots != graph.roots
        || observed.inputs.iter().any(|f| !graph.inputs.contains(f))
        || observed.scopes.iter().any(|s| !graph.scopes.contains(s))
    {
        return Err("native Cargo closure differs from reviewed graph".into());
    }
    active(cancel)?;
    let output = required(
        env.command(&env.cargo).args([
            "build",
            "--locked",
            "--offline",
            "-p",
            "xtask",
            "--no-default-features",
            "--target",
            TARGET,
            "--example",
            "codex_native_results",
            "--message-format=json",
        ]),
        16_777_216,
    )?;
    active(cancel)?;
    let mut compiled = BTreeSet::new();
    let mut features = BTreeMap::new();
    let mut scripts = vec![];
    let mut executable = None;
    for line in output.lines() {
        let v: Value = serde_json::from_str(line).map_err(|_| "invalid Cargo artifact record")?;
        if v["reason"] == "compiler-artifact" {
            let id = v["package_id"].as_str().ok_or("missing compiled package")?;
            compiled.insert(id.into());
            if features
                .insert(super::graph::normalize(id), v["features"].clone())
                .is_some_and(|prior| prior != v["features"])
            {
                return Err("unsupported multiple native feature units".into());
            }
            if v["target"]["name"] == "codex_native_results"
                && v["target"]["kind"] == serde_json::json!(["example"])
            {
                executable = Some(PathBuf::from(
                    v["executable"]
                        .as_str()
                        .ok_or("missing native collector executable")?,
                ));
            }
        }
        if v["reason"] == "build-script-executed" {
            scripts.push(v);
        }
    }
    let mut ids: Vec<_> = scripts.iter().map(|s| s["package_id"].clone()).collect();
    ids.sort_by_key(Value::to_string);
    if compiled != expected || features != units.features || ids != units.build_scripts {
        return Err("native compiled units differ from reviewed build".into());
    }
    let executable = executable.ok_or("missing built native collector")?;
    let report = toolchain(&env, &executable, &scripts)?.into_bytes();
    if reference(&expected_toolchain.path, &report) != *expected_toolchain {
        return Err("native build toolchain differs from approved toolchain".into());
    }
    let graph_bytes = serde_json::to_vec(graph).map_err(|_| "invalid build graph")?;
    let env_bytes = serde_json::to_vec(environment).map_err(|_| "invalid build environment")?;
    let graph_ref = reference(format!("{prefix}/graph.json"), &graph_bytes);
    let env_ref = reference(format!("{prefix}/environment.json"), &env_bytes);
    view.verify()?;
    Ok(Built {
        executable_sha256: identity(&executable)?,
        executable,
        build: Build {
            source_commit: commit,
            source_tree: source.tree()?,
            host_target: TARGET.into(),
            toolchain: expected_toolchain.clone(),
            features: vec![],
            environment: env_ref.clone(),
            dependency_graph: graph_ref.clone(),
            covered_inputs: graph.inputs.clone(),
            absent_inputs: absent,
        },
        payloads: vec![
            (graph_ref, graph_bytes),
            (env_ref, env_bytes),
            (expected_toolchain.clone(), report),
        ],
        _target: env.target,
    })
}

fn active(cancel: &std::sync::atomic::AtomicBool) -> Result<(), String> {
    if cancel.load(std::sync::atomic::Ordering::Acquire) {
        Err("foreground native build interrupted".into())
    } else {
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn interrupted_native_build_refuses_before_workspace_or_build_dispatch() {
        let graph = DependencyGraph {
            schema_version: 1,
            roots: vec![],
            packages: vec![],
            inputs: vec![],
            scopes: vec![],
        };
        let env = super::super::collector::environment(profile::Profile::Core);
        let toolchain = reference("toolchain", b"not read");
        let flag = std::sync::atomic::AtomicBool::new(true);
        let error = match build(
            Path::new("/never-read"),
            "not-created",
            &graph,
            &env,
            &toolchain,
            &flag,
        ) {
            Ok(_) => panic!("interrupted build executed"),
            Err(error) => error,
        };
        assert!(error.contains("interrupted"));
    }
}
