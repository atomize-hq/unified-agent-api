use std::{
    io::{self, Write},
    path::Path,
};

use clap::{ArgGroup, Parser};

#[derive(Debug, Parser, Clone)]
#[command(group(
    ArgGroup::new("mode")
        .required(true)
        .args(["check", "write"])
        .multiple(false)
))]
pub struct Args {
    /// Repo-relative approved onboarding artifact under docs/agents/lifecycle/**/governance/approved-agent.toml.
    #[arg(long)]
    pub approval: String,

    /// Validate that the closeout-preparation contract can be satisfied without rewriting files.
    #[arg(long)]
    pub check: bool,

    /// Materialize the canonical proving-run closeout draft from published lifecycle truth.
    #[arg(long)]
    pub write: bool,
}

#[derive(Debug)]
pub enum Error {
    Validation(String),
    Internal(String),
}

impl Error {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Validation(_) => 2,
            Self::Internal(_) => 1,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Validation(message) | Self::Internal(message) => f.write_str(message),
        }
    }
}

pub fn run(args: Args) -> Result<(), Error> {
    let workspace_root = resolve_workspace_root()?;
    let mut stdout = io::stdout();
    run_in_workspace(&workspace_root, args, &mut stdout)
}

pub fn run_in_workspace<W: Write>(
    _workspace_root: &Path,
    _args: Args,
    _writer: &mut W,
) -> Result<(), Error> {
    Err(Error::Validation(
        "prepare-proving-run-closeout is scaffolded but not implemented yet".to_string(),
    ))
}

fn resolve_workspace_root() -> Result<std::path::PathBuf, Error> {
    let current_dir = std::env::current_dir()
        .map_err(|err| Error::Internal(format!("resolve current directory: {err}")))?;
    for candidate in current_dir.ancestors() {
        let cargo_toml = candidate.join("Cargo.toml");
        let Ok(text) = std::fs::read_to_string(&cargo_toml) else {
            continue;
        };
        if text.contains("[workspace]") {
            return Ok(candidate.to_path_buf());
        }
    }
    Err(Error::Internal(format!(
        "could not resolve workspace root from {}",
        current_dir.display()
    )))
}
