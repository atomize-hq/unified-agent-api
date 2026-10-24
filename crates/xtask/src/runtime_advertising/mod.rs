//! Restricted runtime advertising: generated registry projection and fail-closed source views.
use clap::Parser;
use std::{fs, path::PathBuf};

mod conformance;
#[allow(dead_code)]
#[path = "../../../agent_api/src/runtime_advertising/evaluator.rs"]
mod evaluator;
mod features;
mod graph;
pub mod projection;
pub mod revision;
pub mod source;

pub const DATA: &str = "crates/agent_api/src/runtime_advertising/data.rs";

#[derive(Debug, Parser)]
pub struct Args {
    /// Check generated data and runtime conformance without writing.
    #[arg(long)]
    pub check: bool,
}

pub fn workspace_root() -> Result<PathBuf, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    cwd.ancestors()
        .find(|p| fs::read_to_string(p.join("Cargo.toml")).is_ok_and(|s| s.contains("[workspace]")))
        .map(PathBuf::from)
        .ok_or_else(|| "not in a Cargo workspace".into())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Invalid(String),
    #[error("{0}")]
    Operational(String),
}

impl Error {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Invalid(_) => 2,
            Self::Operational(_) => 1,
        }
    }
}

fn prepare(root: &std::path::Path) -> Result<String, String> {
    let source = source::Source::workspace(root)?;
    conformance::check(&source)?;
    features::Features::load(&source)?;
    source.optional(DATA)?;
    let registry = crate::agent_registry::AgentRegistry::parse(
        &source.read(crate::agent_registry::REGISTRY_RELATIVE_PATH)?,
    )
    .map_err(|e| e.to_string())?;
    Ok(projection::render(&projection::project(&registry)?))
}

pub fn run(args: Args) -> Result<(), Error> {
    let root = workspace_root().map_err(Error::Operational)?;
    if !args.check {
        xtask::depth_routes::refuse_workspace(&root, "runtime-advertising-model")
            .map_err(Error::Invalid)?;
    }
    let source = prepare(&root).map_err(Error::Invalid)?;
    if args.check {
        revision::Revision::workspace(&root).map_err(Error::Invalid)?;
    } else {
        fs::create_dir_all(root.join(DATA).parent().unwrap())
            .map_err(|e| Error::Operational(e.to_string()))?;
        fs::write(root.join(DATA), source).map_err(|e| Error::Operational(e.to_string()))?;
    }
    Ok(())
}
