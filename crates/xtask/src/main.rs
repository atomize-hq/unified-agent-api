#![forbid(unsafe_code)]

mod agent_api_backend_type_leak_guard;
mod capability_matrix_audit;
mod claude_snapshot;
mod claude_union;
mod claude_wrapper_coverage;
mod close_proving_run;
mod codex_report;
mod codex_retain;
mod codex_snapshot;
mod codex_union;
mod codex_validate;
mod codex_version_metadata;
mod codex_wrapper_coverage;
mod historical_lifecycle_backfill;
mod version_bump;
mod wrapper_coverage_shared;

use xtask::agent_maintenance::{
    closeout as agent_maintenance_closeout, drift as agent_maintenance_drift,
    refresh as agent_maintenance_refresh,
};
use xtask::capability_matrix;
pub use xtask::onboard_agent;
pub use xtask::prepare_publication;
pub use xtask::runtime_follow_on;
pub use xtask::support_matrix;
pub use xtask::wrapper_scaffold;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "xtask")]
#[command(about = "Project automation tasks")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
#[allow(clippy::enum_variant_names)]
enum Command {
    /// Guard against backend crate types leaking into the public agent_api surface.
    AgentApiBackendTypeLeakGuard(agent_api_backend_type_leak_guard::Args),
    /// Generate a Codex CLI snapshot manifest under `cli_manifests/codex/`.
    CodexSnapshot(codex_snapshot::Args),
    /// Generate a Claude Code CLI snapshot manifest under `cli_manifests/claude_code/`.
    ClaudeSnapshot(claude_snapshot::Args),
    /// Validate a proving-run closeout artifact and refresh the onboarding packet docs.
    CloseProvingRun(close_proving_run::Args),
    /// Merge per-target snapshots into a union snapshot under `cli_manifests/codex/`.
    CodexUnion(codex_union::Args),
    /// Merge per-target snapshots into a union snapshot under `cli_manifests/claude_code/`.
    ClaudeUnion(claude_union::Args),
    /// Generate deterministic coverage reports under `cli_manifests/codex/reports/<version>/`.
    CodexReport(codex_report::Args),
    /// Materialize `cli_manifests/codex/versions/<version>.json` deterministically.
    CodexVersionMetadata(codex_version_metadata::Args),
    /// Deterministically prune out-of-window snapshots/reports directories (dry-run by default).
    CodexRetain(codex_retain::Args),
    /// Generate `cli_manifests/codex/wrapper_coverage.json` from wrapper source of truth.
    CodexWrapperCoverage(codex_wrapper_coverage::CliArgs),
    /// Generate `cli_manifests/claude_code/wrapper_coverage.json` from wrapper source of truth.
    ClaudeWrapperCoverage(claude_wrapper_coverage::CliArgs),
    /// Validate committed Codex parity artifacts under `cli_manifests/codex/`.
    CodexValidate(codex_validate::Args),
    /// Preview the next control-plane onboarding packet without writing files.
    OnboardAgent(Box<onboard_agent::Args>),
    /// Create a publishable wrapper crate shell for an onboarded agent.
    ScaffoldWrapperCrate(wrapper_scaffold::Args),
    /// Prepare or validate the bounded runtime follow-on lane for an onboarded agent.
    RuntimeFollowOn(runtime_follow_on::Args),
    /// Prepare the committed publication handoff from runtime-integrated evidence.
    PreparePublication(prepare_publication::Args),
    /// Generate or verify the universal agent capability matrix markdown.
    CapabilityMatrix(capability_matrix::Args),
    /// Audit the capability matrix for orthogonality invariants.
    CapabilityMatrixAudit(capability_matrix_audit::Args),
    /// Detect maintenance-relevant drift for an already-onboarded agent.
    CheckAgentDrift(agent_maintenance_drift::Args),
    /// Refresh maintenance packet docs and generated publication surfaces from a maintenance request.
    RefreshAgent(agent_maintenance_refresh::Args),
    /// Validate and close an agent maintenance run.
    CloseAgentMaintenance(agent_maintenance_closeout::Args),
    /// Backfill truthful historical lifecycle maintenance artifacts for known malformed baselines.
    HistoricalLifecycleBackfill(historical_lifecycle_backfill::Args),
    /// Generate support publication JSON and Markdown outputs from committed manifest evidence.
    SupportMatrix(support_matrix::Args),
    /// Bump the workspace release version and exact inter-crate publish pins.
    VersionBump(version_bump::Args),
}

fn main() {
    let cli = Cli::parse();

    let exit_code = match cli.command {
        Command::AgentApiBackendTypeLeakGuard(args) => {
            match agent_api_backend_type_leak_guard::run(args) {
                Ok(()) => 0,
                Err(err) => {
                    eprintln!("{err}");
                    1
                }
            }
        }
        Command::CodexSnapshot(args) => match codex_snapshot::run(args) {
            Ok(()) => 0,
            Err(err) => {
                eprintln!("{err}");
                1
            }
        },
        Command::ClaudeSnapshot(args) => match claude_snapshot::run(args) {
            Ok(()) => 0,
            Err(err) => {
                eprintln!("{err}");
                1
            }
        },
        Command::CloseProvingRun(args) => match close_proving_run::run(args) {
            Ok(()) => 0,
            Err(err) => {
                eprintln!("{err}");
                err.exit_code()
            }
        },
        Command::CodexUnion(args) => match codex_union::run(args) {
            Ok(()) => 0,
            Err(err) => {
                eprintln!("{err}");
                1
            }
        },
        Command::ClaudeUnion(args) => match claude_union::run(args) {
            Ok(()) => 0,
            Err(err) => {
                eprintln!("{err}");
                1
            }
        },
        Command::CodexReport(args) => match codex_report::run(args) {
            Ok(()) => 0,
            Err(err) => {
                eprintln!("{err}");
                1
            }
        },
        Command::CodexVersionMetadata(args) => match codex_version_metadata::run(args) {
            Ok(()) => 0,
            Err(err) => {
                eprintln!("{err}");
                1
            }
        },
        Command::CodexRetain(args) => match codex_retain::run(args) {
            Ok(()) => 0,
            Err(err) => {
                eprintln!("{err}");
                1
            }
        },
        Command::CodexWrapperCoverage(args) => match codex_wrapper_coverage::run(args) {
            Ok(()) => 0,
            Err(err) => {
                eprintln!("{err}");
                1
            }
        },
        Command::ClaudeWrapperCoverage(args) => match claude_wrapper_coverage::run(args) {
            Ok(()) => 0,
            Err(err) => {
                eprintln!("{err}");
                1
            }
        },
        Command::CodexValidate(args) => codex_validate::run(args),
        Command::OnboardAgent(args) => match onboard_agent::run(*args) {
            Ok(()) => 0,
            Err(err) => {
                eprintln!("{err}");
                err.exit_code()
            }
        },
        Command::ScaffoldWrapperCrate(args) => match wrapper_scaffold::run(args) {
            Ok(()) => 0,
            Err(err) => {
                eprintln!("{err}");
                err.exit_code()
            }
        },
        Command::RuntimeFollowOn(args) => match runtime_follow_on::run(args) {
            Ok(()) => 0,
            Err(err) => {
                eprintln!("{err}");
                err.exit_code()
            }
        },
        Command::PreparePublication(args) => match prepare_publication::run(args) {
            Ok(()) => 0,
            Err(err) => {
                eprintln!("{err}");
                err.exit_code()
            }
        },
        Command::CapabilityMatrix(args) => match capability_matrix::run(args) {
            Ok(()) => 0,
            Err(err) => {
                eprintln!("{err}");
                1
            }
        },
        Command::CapabilityMatrixAudit(args) => match capability_matrix_audit::run(args) {
            Ok(()) => 0,
            Err(err) => {
                eprintln!("{err}");
                1
            }
        },
        Command::CheckAgentDrift(args) => match agent_maintenance_drift::run(args) {
            Ok(agent_maintenance_drift::DriftCheckOutcome::Clean(_)) => 0,
            Ok(agent_maintenance_drift::DriftCheckOutcome::DriftDetected(_)) => 2,
            Err(err) => {
                eprintln!("{err}");
                err.exit_code()
            }
        },
        Command::RefreshAgent(args) => match agent_maintenance_refresh::run(args) {
            Ok(()) => 0,
            Err(err) => {
                eprintln!("{err}");
                err.exit_code()
            }
        },
        Command::CloseAgentMaintenance(args) => match agent_maintenance_closeout::run(args) {
            Ok(()) => 0,
            Err(err) => {
                eprintln!("{err}");
                err.exit_code()
            }
        },
        Command::HistoricalLifecycleBackfill(args) => {
            match historical_lifecycle_backfill::run(args) {
                Ok(()) => 0,
                Err(err) => {
                    eprintln!("{err}");
                    err.exit_code()
                }
            }
        }
        Command::SupportMatrix(args) => match support_matrix::run(args) {
            Ok(()) => 0,
            Err(err) => {
                eprintln!("{err}");
                1
            }
        },
        Command::VersionBump(args) => match version_bump::run(args) {
            Ok(()) => 0,
            Err(err) => {
                eprintln!("{err}");
                1
            }
        },
    };

    std::process::exit(exit_code);
}
