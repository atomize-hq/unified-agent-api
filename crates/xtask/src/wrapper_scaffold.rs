use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use clap::{ArgGroup, Parser};
use thiserror::Error;

use crate::{
    agent_registry::{AgentRegistry, AgentRegistryEntry, AgentRegistryError},
    workspace_mutation::{
        apply_mutations, ApplySummary, PlannedMutation, WorkspaceMutationError, WorkspacePathJail,
    },
};

const ROOT_MANIFEST_PATH: &str = "Cargo.toml";
const APACHE_LICENSE_PATH: &str = "LICENSE-APACHE";
const MIT_LICENSE_PATH: &str = "LICENSE-MIT";
const REPOSITORY_URL: &str = "https://github.com/atomize-hq/unified-agent-api";
const HOMEPAGE_URL: &str = "https://github.com/atomize-hq/unified-agent-api";
const FILE_SEQUENCE: [&str; 5] = [
    "Cargo.toml",
    "README.md",
    "LICENSE-APACHE",
    "LICENSE-MIT",
    "src/lib.rs",
];

#[derive(Debug, Parser, Clone)]
#[command(group(
    ArgGroup::new("mode")
        .required(true)
        .args(["dry_run", "write"])
        .multiple(false)
))]
pub struct Args {
    /// Agent identifier from crates/xtask/data/agent_registry.toml.
    #[arg(long = "agent")]
    pub agent_id: String,

    /// Preview the scaffold plan without mutating the workspace.
    #[arg(long)]
    pub dry_run: bool,

    /// Apply the scaffold plan to the workspace.
    #[arg(long)]
    pub write: bool,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    Internal(String),
}

#[derive(Debug, Clone)]
struct WrapperScaffoldPlan {
    agent_id: String,
    display_name: String,
    crate_path: String,
    package_name: String,
    files: Vec<PlannedFile>,
    mutations: Vec<PlannedMutation>,
}

#[derive(Debug, Clone)]
struct PlannedFile {
    relative_path: String,
    contents: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PreviewState {
    Create,
    Identical,
    Divergent,
}

impl Error {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Validation(_) => 2,
            Self::Internal(_) => 1,
        }
    }
}

impl From<WorkspaceMutationError> for Error {
    fn from(err: WorkspaceMutationError) -> Self {
        match err {
            WorkspaceMutationError::Validation(message) => Self::Validation(message),
            WorkspaceMutationError::Internal(message) => Self::Internal(message),
        }
    }
}

pub fn run(args: Args) -> Result<(), Error> {
    let workspace_root = resolve_workspace_root()?;
    let mut stdout = io::stdout();
    run_in_workspace(&workspace_root, args, &mut stdout)
}

pub fn run_in_workspace<W: Write>(
    workspace_root: &Path,
    args: Args,
    writer: &mut W,
) -> Result<(), Error> {
    let jail = WorkspacePathJail::new(workspace_root)?;
    let registry = AgentRegistry::load(workspace_root).map_err(map_registry_load_error)?;
    let agent = registry.find(&args.agent_id).ok_or_else(|| {
        Error::Validation(format!(
            "unknown agent `{}` in agent registry",
            args.agent_id
        ))
    })?;
    let licenses = load_root_licenses(&jail)?;
    let plan = build_plan(agent, licenses)?;
    let preview = collect_preview_states(&jail, &plan)?;

    write_plan_preview(writer, &plan, &preview, args.write)?;
    fail_on_divergence(&preview)?;

    if args.write {
        let summary = apply_mutations(workspace_root, &plan.mutations)?;
        write_apply_result(writer, summary)
    } else {
        writeln!(writer, "== RESULT ==")
            .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
        writeln!(
            writer,
            "OK: scaffold-wrapper-crate dry-run preview complete."
        )
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
        writeln!(writer, "No files were written.")
            .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
        Ok(())
    }
}

fn map_registry_load_error(err: AgentRegistryError) -> Error {
    match err {
        AgentRegistryError::Read { path, source } => {
            Error::Internal(format!("read {path}: {source}"))
        }
        AgentRegistryError::Toml(err) => {
            Error::Validation(format!("parse agent registry TOML: {err}"))
        }
        AgentRegistryError::Validation(message) => Error::Validation(message),
    }
}

fn resolve_workspace_root() -> Result<PathBuf, Error> {
    let current_dir = std::env::current_dir()
        .map_err(|err| Error::Internal(format!("resolve current directory: {err}")))?;
    for candidate in current_dir.ancestors() {
        let cargo_toml = candidate.join(ROOT_MANIFEST_PATH);
        let Ok(text) = fs::read_to_string(&cargo_toml) else {
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

fn load_root_licenses(jail: &WorkspacePathJail) -> Result<(Vec<u8>, Vec<u8>), Error> {
    let apache_path = jail.resolve(Path::new(APACHE_LICENSE_PATH))?;
    let mit_path = jail.resolve(Path::new(MIT_LICENSE_PATH))?;
    let apache = fs::read(&apache_path)
        .map_err(|err| Error::Internal(format!("read {}: {err}", apache_path.display())))?;
    let mit = fs::read(&mit_path)
        .map_err(|err| Error::Internal(format!("read {}: {err}", mit_path.display())))?;
    Ok((apache, mit))
}

fn build_plan(
    agent: &AgentRegistryEntry,
    licenses: (Vec<u8>, Vec<u8>),
) -> Result<WrapperScaffoldPlan, Error> {
    let crate_dir = Path::new(&agent.crate_path)
        .file_name()
        .and_then(|part| part.to_str())
        .ok_or_else(|| {
            Error::Validation(format!(
                "agent `{}` has invalid crate path `{}`",
                agent.agent_id, agent.crate_path
            ))
        })?;
    let primary_keyword = primary_keyword(&agent.display_name)?;
    let cargo_toml = render_cargo_toml(
        &agent.display_name,
        &agent.package_name,
        crate_dir,
        &primary_keyword,
    );
    let readme = render_readme(&agent.display_name, &agent.package_name, crate_dir);
    let lib_rs = render_lib_rs(&agent.display_name);

    let mut files = Vec::with_capacity(FILE_SEQUENCE.len());
    let base = Path::new(&agent.crate_path);
    let (apache_license, mit_license) = licenses;
    for relative_name in FILE_SEQUENCE {
        let relative_path = base.join(relative_name);
        let contents = match relative_name {
            "Cargo.toml" => cargo_toml.clone().into_bytes(),
            "README.md" => readme.clone().into_bytes(),
            "LICENSE-APACHE" => apache_license.clone(),
            "LICENSE-MIT" => mit_license.clone(),
            "src/lib.rs" => lib_rs.clone().into_bytes(),
            _ => {
                return Err(Error::Internal(format!(
                    "unhandled scaffold file template `{relative_name}`"
                )));
            }
        };
        files.push(PlannedFile {
            relative_path: relative_path.to_string_lossy().into_owned(),
            contents,
        });
    }

    let mutations = files
        .iter()
        .map(|file| PlannedMutation::create(&file.relative_path, file.contents.clone()))
        .collect();

    Ok(WrapperScaffoldPlan {
        agent_id: agent.agent_id.clone(),
        display_name: agent.display_name.clone(),
        crate_path: agent.crate_path.clone(),
        package_name: agent.package_name.clone(),
        files,
        mutations,
    })
}

fn primary_keyword(display_name: &str) -> Result<String, Error> {
    let keyword = display_name
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '-')
        .flat_map(char::to_lowercase)
        .collect::<String>();
    if keyword.is_empty() {
        return Err(Error::Validation(format!(
            "display_name `{display_name}` does not yield a valid keyword"
        )));
    }
    Ok(keyword)
}

fn render_cargo_toml(
    display_name: &str,
    package_name: &str,
    lib_name: &str,
    primary_keyword: &str,
) -> String {
    format!(
        concat!(
            "[package]\n",
            "name = \"{package_name}\"\n",
            "version.workspace = true\n",
            "edition = \"2021\"\n",
            "rust-version = \"1.78\"\n",
            "description = \"Wrapper scaffold for {display_name}; runtime/backend implementation pending\"\n",
            "license = \"MIT OR Apache-2.0\"\n",
            "repository = \"{repository}\"\n",
            "homepage = \"{homepage}\"\n",
            "documentation = \"https://docs.rs/{package_name}\"\n",
            "keywords = [\"{primary_keyword}\", \"cli\", \"wrapper\", \"agent\"]\n",
            "categories = [\"api-bindings\", \"command-line-interface\"]\n",
            "readme = \"README.md\"\n",
            "\n",
            "[lib]\n",
            "name = \"{lib_name}\"\n",
            "\n",
            "[dependencies]\n",
        ),
        package_name = package_name,
        display_name = display_name,
        repository = REPOSITORY_URL,
        homepage = HOMEPAGE_URL,
        primary_keyword = primary_keyword,
        lib_name = lib_name,
    )
}

fn render_readme(display_name: &str, package_name: &str, lib_name: &str) -> String {
    format!(
        concat!(
            "# {display_name} Rust Wrapper\n\n",
            "- crates.io package: `{package_name}`\n",
            "- Rust library crate: `{lib_name}`\n\n",
            "This crate is a publishable scaffold. Runtime/backend implementation is pending.\n",
        ),
        display_name = display_name,
        package_name = package_name,
        lib_name = lib_name,
    )
}

fn render_lib_rs(display_name: &str) -> String {
    format!(
        concat!(
            "#![forbid(unsafe_code)]\n",
            "//! Publishable scaffold for the {display_name} wrapper crate.\n",
            "//!\n",
            "//! Runtime/backend implementation is pending.\n",
        ),
        display_name = display_name,
    )
}

fn collect_preview_states(
    jail: &WorkspacePathJail,
    plan: &WrapperScaffoldPlan,
) -> Result<Vec<(String, PreviewState)>, Error> {
    let mut preview = Vec::with_capacity(plan.files.len());
    for file in &plan.files {
        let absolute_path = jail.resolve(Path::new(&file.relative_path))?;
        let state = match fs::symlink_metadata(&absolute_path) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() || !metadata.is_file() {
                    PreviewState::Divergent
                } else {
                    let current = fs::read(&absolute_path).map_err(|err| {
                        Error::Internal(format!("read {}: {err}", absolute_path.display()))
                    })?;
                    if current == file.contents {
                        PreviewState::Identical
                    } else {
                        PreviewState::Divergent
                    }
                }
            }
            Err(err) if err.kind() == io::ErrorKind::NotFound => PreviewState::Create,
            Err(err) => {
                return Err(Error::Internal(format!(
                    "stat {}: {err}",
                    absolute_path.display()
                )));
            }
        };
        preview.push((file.relative_path.clone(), state));
    }
    Ok(preview)
}

fn write_plan_preview<W: Write>(
    writer: &mut W,
    plan: &WrapperScaffoldPlan,
    preview: &[(String, PreviewState)],
    write_mode: bool,
) -> Result<(), Error> {
    writeln!(
        writer,
        "== SCAFFOLD-WRAPPER-CRATE {} ==",
        if write_mode { "WRITE" } else { "DRY RUN" }
    )
    .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "agent_id: {}", plan.agent_id)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "display_name: {}", plan.display_name)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "crate_path: {}", plan.crate_path)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "package_name: {}", plan.package_name)
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "planned_files:")
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    for (path, state) in preview {
        writeln!(writer, "- {} [{}]", path, preview_state_label(*state))
            .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    }
    Ok(())
}

fn preview_state_label(state: PreviewState) -> &'static str {
    match state {
        PreviewState::Create => "create",
        PreviewState::Identical => "identical",
        PreviewState::Divergent => "divergent",
    }
}

fn fail_on_divergence(preview: &[(String, PreviewState)]) -> Result<(), Error> {
    if let Some((path, _)) = preview
        .iter()
        .find(|(_, state)| *state == PreviewState::Divergent)
    {
        return Err(Error::Validation(format!(
            "planned write `{path}` is divergent; refusing to overwrite unexpected contents"
        )));
    }
    Ok(())
}

fn write_apply_result<W: Write>(writer: &mut W, summary: ApplySummary) -> Result<(), Error> {
    writeln!(writer, "== RESULT ==")
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(writer, "OK: scaffold-wrapper-crate write complete.")
        .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    writeln!(
        writer,
        "Mutation summary: {} written, {} identical, {} total planned.",
        summary.written, summary.identical, summary.total
    )
    .map_err(|err| Error::Internal(format!("write stdout: {err}")))?;
    Ok(())
}
