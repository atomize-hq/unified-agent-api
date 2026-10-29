//! Untrusted revision-2 native forms. Parsing never establishes host or execution authority.
use super::*;
use serde::{Deserialize, Serialize};
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
