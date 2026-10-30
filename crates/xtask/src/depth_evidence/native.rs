//! Untrusted revision-2 native forms. Parsing never establishes host or execution authority.
use super::*;
use serde::{Deserialize, Serialize};
pub mod context;
mod validate;
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Repository {
    pub name: String,
    pub id: u64,
    pub owner_id: u64,
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CommentRef {
    pub id: u64,
    pub body_sha256: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    MaintainerNative,
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeProducer {
    pub kind: Kind,
    pub repository: Repository,
    pub pull_request: u64,
    pub registration: CommentRef,
    pub completion: CommentRef,
    pub collector: FileRef,
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeReceipt {
    pub schema_version: u64,
    pub scope: Scope,
    pub producer: NativeProducer,
    pub build: Build,
    pub provenance: Vec<Provenance>,
    pub cases: Cases,
    pub adequacy: Adequacy,
}
impl NativeReceipt {
    pub fn parse(raw: &str) -> Result<Self, String> {
        let r: Self = payload::parse(raw)?;
        validate::receipt(&r)?;
        Ok(r)
    }
    pub(super) fn claims(&self) -> Claims<'_> {
        Claims {
            scope: &self.scope,
            build: &self.build,
            provenance: &self.provenance,
            cases: &self.cases,
            adequacy: &self.adequacy,
            collector: &self.producer.collector,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Start {
    pub schema_version: u64,
    pub profile: String,
    pub repository: Repository,
    pub pull_request: u64,
    pub packet_head: String,
    pub scope: Scope,
    pub plan: FileRef,
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Finish {
    pub schema_version: u64,
    pub registration: CommentRef,
    pub evidence_commit: String,
    pub bundle: FileRef,
    pub executor_identity: String,
    pub population_through_id: u64,
    pub registrations: Vec<u64>,
    pub execution_attested: bool,
    pub outcomes_attested: bool,
    pub registration_order_attested: bool,
    pub completeness_attested: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Runtime {
    pub version: String,
    pub target: String,
    pub executable_sha256: String,
    pub acquisition: FileRef,
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    pub schema_version: u64,
    pub scope: Scope,
    pub packet_head: String,
    pub build: Build,
    pub native: Runtime,
    pub context: FileRef,
    pub collector: FileRef,
    pub requested: Vec<CaseIdentity>,
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Bundle {
    pub schema_version: u64,
    pub observed_context: FileRef,
    pub plan: FileRef,
    pub claims: Vec<FileRef>,
    pub files: Vec<FileRef>,
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Claim {
    pub scope: Scope,
    pub build: Build,
    pub provenance: Vec<Provenance>,
    pub cases: Cases,
    pub mapping: AdequacyMapping,
}
macro_rules! parsed {
    ($($t:ty => $validate:ident),* $(,)?) => {$(impl $t {
        pub fn parse(raw:&str)->Result<Self,String>{
            let r:Self=payload::parse(raw)?; validate::$validate(&r)?; Ok(r)
        }
    })*};
}
parsed!(Start=>start, Finish=>finish, Plan=>plan, Bundle=>bundle, Claim=>claim);

/// Explicit foreground invocation options. None of these values selects an admission profile.
#[derive(Debug, clap::Args)]
pub struct CollectArgs {
    /// Acknowledge use of the caller's native authentication and usage allowance.
    #[arg(long, required = true)]
    pub live: bool,
    /// Authorize exactly one pre-execution start-registration POST; no finish is posted.
    #[arg(long, required = true)]
    pub register: bool,
    #[arg(long)]
    pub binary: std::path::PathBuf,
    /// Disposable native project, separate from this source checkout.
    #[arg(long)]
    pub working_dir: std::path::PathBuf,
    /// Forward an existing native home without reading or copying credentials.
    #[arg(long)]
    pub codex_home: Option<std::path::PathBuf>,
    /// Fresh storage leaf, chosen before registration; never used to select trust.
    #[arg(long)]
    pub attempt: String,
}
/// Run the standalone compiled producer. An absent profile refuses before any I/O or probe.
/// An active invocation installs process-lifetime SIGINT/SIGTERM handlers; use the xtask command.
pub fn collect(workspace: &std::path::Path, args: CollectArgs) -> Result<(), String> {
    super::verdict::collect_native(workspace, args)
}

/// Finite diagnostic journal shared by the existing runner and the packet producer.
/// These are observations, never authenticated execution or adequacy facts.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeObservation {
    pub session_id: Option<String>,
    pub consistent: bool,
    pub completion_success: bool,
    pub reply_matches: bool,
    pub cancel_requested: bool,
    pub cancelled: bool,
    pub unexpected_item: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeFailure {
    RequestRefused,
    Cancelled,
    CompletionFailed,
    ContextRefused,
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeStep {
    pub case: CaseIdentity,
    pub observation: NativeObservation,
    pub outcome: CaseOutcome,
    pub elapsed_ms: u128,
    pub failure: Option<NativeFailure>,
}

pub fn native_session_id(id: &str) -> bool {
    id.len() == 36
        && id.bytes().enumerate().all(|(i, c)| {
            if [8, 13, 18, 23].contains(&i) {
                c == b'-'
            } else {
                c.is_ascii_hexdigit() && !c.is_ascii_uppercase()
            }
        })
        && id != "11111111-1111-4111-8111-111111111111"
        && id != "00000000-0000-0000-0000-000000000000"
}

/// Prepare an unposted template from committed proof. All human attestations start false.
#[derive(Debug, clap::Args)]
pub struct PrepareFinishArgs {
    #[arg(long)]
    pub attempt: String,
    #[arg(long)]
    pub evidence_commit: String,
}
pub fn prepare_finish(
    workspace: &std::path::Path,
    args: PrepareFinishArgs,
) -> Result<String, String> {
    super::verdict::prepare_native_finish(workspace, args)
}
