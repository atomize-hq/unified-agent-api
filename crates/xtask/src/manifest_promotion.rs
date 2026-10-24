//! Owning promotion proposal: render in an invocation-local candidate, admit every effect,
//! and deliver one expected-before batch. This is not final integration authority or a lock.
use crate::{
    agent_maintenance::closeout_proposal::Proposal,
    agent_registry::AgentRegistry,
    runtime_advertising::source::Source,
    workspace_mutation::{apply_admitted_mutations as apply_mutations, ApplySummary},
};
use clap::Parser;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Parser)]
pub struct Args {
    #[arg(long)]
    pub agent: String,
    #[arg(long)]
    pub version: String,
    /// Targets whose validation jobs succeeded against validation-commit (never inferred from union membership).
    #[arg(long = "passed-target", required = true)]
    pub passed_targets: Vec<String>,
    #[arg(long)]
    pub validation_commit: String,
    /// Default is a read-only proposal; delivery requires this flag.
    #[arg(long)]
    pub write: bool,
}

pub struct PromotionPlan {
    root: PathBuf,
    observation: Proposal,
    files: Vec<(PathBuf, Option<Vec<u8>>)>,
    validation_commit: String,
}
impl PromotionPlan {
    pub fn paths(&self) -> Vec<&Path> {
        self.files.iter().map(|(p, _)| p.as_path()).collect()
    }
    pub fn apply(self, root: &Path) -> Result<ApplySummary, String> {
        if root.canonicalize().map_err(|e| e.to_string())? != self.root {
            return Err("promotion workspace changed".into());
        }
        require_revision(root, &self.validation_commit)?;
        let plans = self.observation.plan_changes(
            root,
            self.files,
            crate::depth_acceptance::plan_promotion,
        )?;
        require_revision(root, &self.validation_commit)?;
        apply_mutations(root, &plans).map_err(|e| e.to_string())
    }
}
fn git(root: &Path, revision: &str) -> Result<String, String> {
    let out = crate::git_inputs::command(root)
        .args(["rev-parse", "--verify", "--end-of-options", revision])
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err("promotion requires a resolved validation commit and integration tip".into());
    }
    String::from_utf8(out.stdout)
        .map(|s| s.trim().into())
        .map_err(|e| e.to_string())
}
fn require_revision(root: &Path, revision: &str) -> Result<(), String> {
    if revision.len() != 40
        || !revision.bytes().all(|b| b.is_ascii_hexdigit())
        || git(root, "HEAD^{commit}")? != revision
        || git(root, "origin/staging^{commit}")? != revision
    {
        return Err("promotion validation revision no longer matches HEAD/integration tip".into());
    }
    let status = crate::git_inputs::command(root)
        .args([
            "status",
            "--porcelain=v1",
            "--untracked-files=all",
            "--",
            "crates",
            "docs",
            "cli_manifests",
            ".github",
            ".cargo",
            "Cargo.toml",
            "Cargo.lock",
            "rust-toolchain",
            "rust-toolchain.toml",
        ])
        .output()
        .map_err(|e| e.to_string())?;
    if !status.status.success() || !status.stdout.is_empty() {
        return Err("promotion validation inputs differ from the validated commit".into());
    }
    Ok(())
}
pub fn build_plan(root: &Path, args: &Args) -> Result<PromotionPlan, String> {
    semver::Version::parse(&args.version).map_err(|e| e.to_string())?;
    require_revision(root, &args.validation_commit)?;
    let observation = Proposal::capture(root)?;
    let scratch = tempfile::tempdir().map_err(|e| e.to_string())?;
    observation.materialize(scratch.path())?;
    let registry = AgentRegistry::load(scratch.path()).map_err(|e| e.to_string())?;
    let entry = registry
        .find(&args.agent)
        .ok_or("unknown promotion agent")?;
    let base = &entry.manifest_root;
    let rules: Value = serde_json::from_slice(
        &fs::read(scratch.path().join(base).join("RULES.json")).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let union_path = format!("{base}/snapshots/{}/union.json", args.version);
    let union_bytes = fs::read(scratch.path().join(&union_path)).map_err(|e| e.to_string())?;
    let union: Value = serde_json::from_slice(&union_bytes).map_err(|e| e.to_string())?;
    if union["complete"] != true
        && rules["union"]["promotion_policy"]["allow_promote_when_incomplete"] != true
    {
        return Err("incomplete union promotion is not permitted".into());
    }
    let required = rules["union"]["required_target"]
        .as_str()
        .ok_or("missing required target")?;
    let passed: BTreeSet<_> = args.passed_targets.iter().cloned().collect();
    if passed.len() != args.passed_targets.len() || !passed.contains(required) {
        return Err("promotion requires unique passed targets including required target".into());
    }
    let acquired: BTreeSet<_> = union["inputs"]
        .as_array()
        .ok_or("missing acquired targets")?
        .iter()
        .map(|v| {
            v["target_triple"]
                .as_str()
                .ok_or("invalid acquired target")
                .map(str::to_owned)
        })
        .collect::<Result<_, _>>()?;
    let expected: BTreeSet<_> = rules["union"]["expected_targets"]
        .as_array()
        .ok_or("missing expected targets")?
        .iter()
        .map(|v| {
            v.as_str()
                .ok_or("invalid expected target")
                .map(str::to_owned)
        })
        .collect::<Result<_, _>>()?;
    if !passed.is_subset(&acquired) || !passed.is_subset(&expected) {
        return Err("passed target is not an acquired policy target".into());
    }
    let (_, metadata) = crate::manifest_version_metadata::plan(
        crate::manifest_version_metadata::Args {
            root: Some(scratch.path().join(base)),
            rules: None,
            version: args.version.clone(),
            status: crate::manifest_version_metadata::Status::Validated,
            passed_targets: passed.iter().cloned().collect(),
            failed_targets: vec![],
            skipped_targets: vec![],
        },
        None,
    )
    .map_err(|e| e.to_string())?;
    let meta: Value = serde_json::from_slice(&metadata).map_err(|e| e.to_string())?;
    let mut files: BTreeMap<String, Option<Vec<u8>>> = BTreeMap::new();
    files.insert(
        format!("{base}/versions/{}.json", args.version),
        Some(metadata),
    );
    files.insert(
        format!("{base}/latest_validated.txt"),
        Some(format!("{}\n", args.version).into_bytes()),
    );
    files.insert(format!("{base}/current.json"), Some(union_bytes));
    for target in &passed {
        files.insert(
            format!("{base}/pointers/latest_validated/{target}.txt"),
            Some(format!("{}\n", args.version).into_bytes()),
        );
    }
    for target in meta["coverage"]["supported_targets"]
        .as_array()
        .ok_or("missing supported targets")?
    {
        let target = target.as_str().ok_or("invalid supported target")?;
        if passed.contains(target) {
            files.insert(
                format!("{base}/pointers/latest_supported/{target}.txt"),
                Some(format!("{}\n", args.version).into_bytes()),
            );
        }
    }
    let marker = crate::agent_maintenance::stand_down::marker_relative_path(
        &format!("docs/agents/lifecycle/{}-maintenance", args.agent),
        &args.version,
    );
    if Source::workspace(scratch.path())?
        .optional(&marker)?
        .is_some()
    {
        files.insert(marker, None);
    }
    for (p, b) in &files {
        let full = scratch.path().join(p);
        if let Some(b) = b {
            fs::create_dir_all(full.parent().ok_or("invalid output parent")?)
                .map_err(|e| e.to_string())?;
            fs::write(full, b).map_err(|e| e.to_string())?;
        } else {
            fs::remove_file(full).map_err(|e| e.to_string())?;
        }
    }
    let bundle = crate::support_matrix::generate_publication_artifacts(scratch.path())?;
    for (p, b) in [
        (crate::support_matrix::JSON_OUTPUT_PATH, bundle.json),
        (crate::support_matrix::MARKDOWN_OUTPUT_PATH, bundle.markdown),
        (
            crate::support_matrix::AGENT_API_RUNTIME_SUPPORT_DATA_OUTPUT_PATH,
            bundle.runtime_support_data,
        ),
        (
            crate::capability_matrix::DEFAULT_OUT_PATH,
            crate::capability_matrix::generate_markdown_for_workspace(scratch.path())?,
        ),
    ] {
        files.insert(p.into(), Some(b.into_bytes()));
    }
    // The final candidate, including all publication outputs, passes the canonical validator
    // before any caller bytes are changed. Scratch is a rendering view, never delivery.
    for (path, bytes) in &files {
        if let Some(bytes) = bytes {
            let full = scratch.path().join(path);
            fs::create_dir_all(full.parent().ok_or("invalid output parent")?)
                .map_err(|e| e.to_string())?;
            fs::write(full, bytes).map_err(|e| e.to_string())?;
        }
    }
    if crate::manifest_validate::run(crate::manifest_validate::Args {
        root: Some(scratch.path().join(base)),
        rules: None,
        schema: None,
        version_schema: None,
        mode: crate::manifest_validate::Mode::Check,
        json: false,
        tip: None,
    }) != 0
    {
        return Err("promotion candidate fails canonical manifest validation".into());
    }
    let files = files
        .into_iter()
        .map(|(p, b)| (p.into(), b))
        .collect::<Vec<_>>();
    observation.plan_changes(root, files.clone(), crate::depth_acceptance::plan_promotion)?;
    require_revision(root, &args.validation_commit)?;
    Ok(PromotionPlan {
        root: root.canonicalize().map_err(|e| e.to_string())?,
        observation,
        files,
        validation_commit: args.validation_commit.clone(),
    })
}
pub fn run(args: Args) -> Result<(), String> {
    let root = crate::runtime_advertising::workspace_root()?;
    let plan = build_plan(&root, &args)?;
    for path in plan.paths() {
        println!("{}", path.display());
    }
    if args.write {
        plan.apply(&root)?;
    }
    Ok(())
}
