use std::path::PathBuf;

use clap::{ArgGroup, Parser};

use super::{args_from_request, Args, Error};

#[derive(Debug, Parser)]
#[command(group(
    ArgGroup::new("mode")
        .required(true)
        .args(["dry_run", "write"])
        .multiple(false)
))]
pub struct Cli {
    #[arg(long, conflicts_with_all = ["agent", "current_version", "latest_stable", "target_version", "opened_from", "detected_by", "dispatch_kind", "dispatch_workflow", "branch_name", "request_recorded_at", "request_commit"])]
    from_request: Option<PathBuf>,
    #[arg(long, required_unless_present = "from_request")]
    agent: Option<String>,
    #[arg(long, required_unless_present = "from_request")]
    current_version: Option<String>,
    #[arg(long, required_unless_present = "from_request")]
    latest_stable: Option<String>,
    #[arg(long, required_unless_present = "from_request")]
    target_version: Option<String>,
    #[arg(long, required_unless_present = "from_request")]
    opened_from: Option<PathBuf>,
    #[arg(long, required_unless_present = "from_request")]
    detected_by: Option<String>,
    #[arg(long, required_unless_present = "from_request")]
    dispatch_kind: Option<String>,
    #[arg(long)]
    dispatch_workflow: Option<String>,
    #[arg(long, required_unless_present = "from_request")]
    branch_name: Option<String>,
    #[arg(long, required_unless_present = "from_request")]
    request_recorded_at: Option<String>,
    #[arg(long, required_unless_present = "from_request")]
    request_commit: Option<String>,
    #[arg(long)]
    dry_run: bool,
    #[arg(long)]
    write: bool,
}

impl Cli {
    pub fn into_args(self) -> Result<Args, Error> {
        if let Some(path) = self.from_request {
            return args_from_request(&path, self.dry_run, self.write);
        }

        Ok(Args {
            agent: self.agent.expect("clap requires --agent"),
            current_version: self
                .current_version
                .expect("clap requires --current-version"),
            latest_stable: self.latest_stable.expect("clap requires --latest-stable"),
            target_version: self.target_version.expect("clap requires --target-version"),
            opened_from: self.opened_from.expect("clap requires --opened-from"),
            detected_by: self.detected_by.expect("clap requires --detected-by"),
            dispatch_kind: self.dispatch_kind.expect("clap requires --dispatch-kind"),
            dispatch_workflow: self.dispatch_workflow,
            branch_name: self.branch_name.expect("clap requires --branch-name"),
            request_recorded_at: self
                .request_recorded_at
                .expect("clap requires --request-recorded-at"),
            request_commit: self.request_commit.expect("clap requires --request-commit"),
            dry_run: self.dry_run,
            write: self.write,
        })
    }
}
