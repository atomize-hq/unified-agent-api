use serde::Deserialize;

/// The standing request, read only for what checks 1 and 3 compare. The validating loader
/// refuses a `[support_depth]` table while no depth enrollment can be declared, and refuses a
/// first-freeze placeholder, so this reader fails only on a file that is not TOML.
#[derive(Default, Deserialize)]
#[serde(default)]
pub struct StandingRequest {
    pub request_commit: String,
    pub request_recorded_at: String,
    pub trigger_kind: String,
    pub opened_from: String,
    pub detected_release: Option<DetectedRelease>,
    pub support_depth: Option<RequestTable>,
}

#[derive(Deserialize)]
pub struct DetectedRelease {
    pub target_version: Option<String>,
}

#[derive(Deserialize)]
pub struct RequestTable {
    pub policy_identity: Option<String>,
    pub obligations_identity: Option<String>,
}
