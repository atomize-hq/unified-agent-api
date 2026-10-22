//! Preparation-local binding, not write authority or a cross-route lock.
use super::*;
use crate::{agent_registry::REGISTRY_RELATIVE_PATH, runtime_advertising::source::Source};
use std::{collections::BTreeMap, fs, process::Command};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Context {
    pub args: Args,
    pub workspace: PathBuf,
    pub inputs: BTreeMap<String, Option<String>>,
    pub index: Option<Vec<u8>>,
    pub before: Vec<Option<Vec<u8>>>,
}

pub(super) fn refuse_retained_depth(root: &Path, registry: &AgentRegistry) -> Result<(), Error> {
    let source = Source::workspace(root).map_err(Error::Validation)?;
    for entry in &registry.agents {
        for path in source
            .paths(&format!("{}/reports", entry.manifest_root))
            .map_err(Error::Validation)?
        {
            if path.ends_with("/depth-record.json") {
                return Err(Error::Validation(format!(
                    "production depth preparation is not enabled: retained record {path}"
                )));
            }
        }
    }
    Ok(())
}

impl Context {
    pub fn capture(root: &Path, registry: &AgentRegistry, args: &Args) -> Result<Self, Error> {
        let workspace = fs::canonicalize(root).map_err(|e| Error::Internal(e.to_string()))?;
        let source = Source::workspace(root).map_err(Error::Validation)?;
        let mut inputs = BTreeMap::new();
        let depth = registry.agents.iter().any(|e| e.support_depth.is_some());
        let mut prefixes = vec![
            args.opened_from.display().to_string(),
            REGISTRY_RELATIVE_PATH.to_owned(),
            super::super::support_audit::NON_TUI_SUPPORT_DEBT_PATH.to_owned(),
        ];
        // Reports include old debt evidence and retained records; directory membership affects
        // coverage selection. Depth publication also reads every published root, not just --agent.
        for entry in &registry.agents {
            if entry.agent_id != args.agent && !depth {
                continue;
            }
            let base = &entry.manifest_root;
            prefixes.extend([
                format!("{base}/RULES.json"),
                format!("{base}/SCHEMA.json"),
                format!("{base}/wrapper_coverage.json"),
                format!("{base}/latest_validated.txt"),
                format!("{base}/current.json"),
            ]);
            for dir in [
                format!("{base}/reports"),
                format!("{base}/versions"),
                format!("{base}/pointers"),
                format!("{base}/snapshots/{}", args.target_version),
                format!(
                    "docs/agents/lifecycle/{}-maintenance/governance",
                    entry.agent_id
                ),
                format!(
                    "docs/agents/lifecycle/{}/governance",
                    entry.scaffold.onboarding_pack_prefix
                ),
            ] {
                let paths = source.paths(&dir).map_err(Error::Validation)?;
                inputs.insert(format!("directory:{dir}"), Some(paths.join("\n")));
                for path in paths {
                    if root.join(&path).is_file() {
                        prefixes.push(path);
                    }
                }
            }
        }
        if depth {
            for entry in &registry.agents {
                for (version, path) in
                    crate::depth_record::record_files(&root.join(&entry.manifest_root))
                {
                    let raw =
                        fs::read_to_string(path).map_err(|e| Error::Validation(e.to_string()))?;
                    let record =
                        crate::depth_record::DepthRecord::parse(&raw, &entry.agent_id, &version)
                            .map_err(Error::Validation)?;
                    prefixes.extend(
                        record
                            .execution
                            .iter()
                            .flat_map(|e| &e.results)
                            .flat_map(|r| &r.evidence)
                            .map(|file| file.path.clone()),
                    );
                }
            }
        }
        for path in &prefixes {
            let bytes = source.optional(path).map_err(Error::Validation)?;
            inputs.insert(
                path.clone(),
                bytes.map(|s| hex::encode(Sha256::digest(s.as_bytes()))),
            );
        }
        if depth {
            for path in [
                crate::support_matrix::JSON_OUTPUT_PATH,
                crate::support_matrix::MARKDOWN_OUTPUT_PATH,
                crate::support_matrix::AGENT_API_RUNTIME_SUPPORT_DATA_OUTPUT_PATH,
                "docs/specs/unified-agent-api/capability-matrix.md",
                "Cargo.toml",
                "crates/agent_api/Cargo.toml",
                "crates/agent_api/build.rs",
                ".cargo/config",
                ".cargo/config.toml",
                "crates/agent_api/.cargo/config",
                "crates/agent_api/.cargo/config.toml",
            ] {
                inputs.insert(
                    path.into(),
                    source
                        .optional(path)
                        .map_err(Error::Validation)?
                        .map(|s| hex::encode(Sha256::digest(s.as_bytes()))),
                );
            }
            for path in source
                .paths("crates/agent_api/src")
                .map_err(Error::Validation)?
            {
                if root.join(&path).is_file() {
                    inputs.insert(
                        path.clone(),
                        source
                            .optional(&path)
                            .map_err(Error::Validation)?
                            .map(|s| hex::encode(Sha256::digest(s.as_bytes()))),
                    );
                }
            }
        }
        let output = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .map_err(|e| Error::Internal(e.to_string()))?;
        let index = if output.status.success() {
            let output = Command::new("git")
                .arg("-C")
                .arg(root)
                .args(["--literal-pathspecs", "ls-files", "--stage", "-z", "--"])
                .args(inputs.keys().filter(|p| !p.starts_with("directory:")))
                .output()
                .map_err(|e| Error::Internal(e.to_string()))?;
            if !output.status.success() {
                return Err(Error::Validation(
                    String::from_utf8_lossy(&output.stderr).into_owned(),
                ));
            }
            Some(output.stdout)
        } else {
            None
        };
        Ok(Self {
            args: args.clone(),
            workspace,
            inputs,
            index,
            before: Vec::new(),
        })
    }

    pub fn bind_outputs(&mut self, root: &Path, files: &[PreparedFile]) -> Result<(), Error> {
        let jail = WorkspacePathJail::new(root)?;
        self.before = files
            .iter()
            .map(|file| {
                let path = jail.resolve(Path::new(&file.relative_path))?;
                match fs::read(&path) {
                    Ok(bytes) => Ok(Some(bytes)),
                    Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
                    Err(e) => Err(Error::Validation(format!("{}: {e}", path.display()))),
                }
            })
            .collect::<Result<_, _>>()?;
        Ok(())
    }
}

pub(super) fn apply(
    root: &Path,
    plan: &PreparePlan,
    registry: &AgentRegistry,
) -> Result<ApplySummary, Error> {
    if fs::canonicalize(root).map_err(|e| Error::Internal(e.to_string()))? != plan.context.workspace
    {
        return Err(Error::Validation(
            "prepare plan belongs to another workspace".into(),
        ));
    }
    // Rebuild through the same pure path. Public fields never authorize output paths or bytes.
    let current = super::build_with_registry(root, &plan.context.args, registry)?;
    if current != *plan {
        return Err(Error::Validation(
            "stale or modified prepare plan; rebuild before applying".into(),
        ));
    }
    let mutations = plan
        .files
        .iter()
        .zip(&plan.context.before)
        .map(|(file, before)| match before {
            Some(before) => crate::workspace_mutation::PlannedMutation::replace(
                &file.relative_path,
                before.clone(),
                file.contents.clone(),
            ),
            None => crate::workspace_mutation::PlannedMutation::create(
                &file.relative_path,
                file.contents.clone(),
            ),
        })
        .collect::<Vec<_>>();
    apply_mutations(root, &mutations).map_err(Into::into)
}
