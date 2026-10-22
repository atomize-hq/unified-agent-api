use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotV1 {
    pub snapshot_schema_version: u32,
    pub tool: String,
    pub collected_at: String,
    pub binary: BinarySnapshot,
    pub commands: Vec<CommandSnapshot>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub features: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub known_omissions: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinarySnapshot {
    pub sha256: String,
    pub size_bytes: u64,
    pub platform: BinaryPlatform,
    pub target_triple: String,
    pub version_output: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semantic_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryPlatform {
    pub os: String,
    pub arch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandSnapshot {
    pub path: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub about: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stability: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platforms: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<ArgSnapshot>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<Vec<FlagSnapshot>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgSnapshot {
    pub name: String,
    pub required: bool,
    pub variadic: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagSnapshot {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub long: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short: Option<String>,
    pub takes_value: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeatable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stability: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platforms: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SupplementV1 {
    pub version: u32,
    pub commands: Vec<SupplementCommand>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SupplementCommand {
    pub path: Vec<String>,
    #[serde(default)]
    pub platforms: Option<Vec<String>>,
    pub note: String,
}
