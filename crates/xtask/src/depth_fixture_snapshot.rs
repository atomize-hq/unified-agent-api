//! Snapshot adapter for the synthetic `depth_fixture` CLI (SA-P1).
//!
//! The fixture publishes its own surface through `--surface-json`, so this adapter does not parse
//! help text. It accepts the argument shape `parity-acquire.yml` passes to every snapshot command.
//! Only per-target `--out-file` mode exists: `<root>/snapshots/<version>/<target>.json`.

use std::{
    fs, io,
    path::{Path, PathBuf},
    process::Command,
};

use clap::Parser;
use sha2::{Digest, Sha256};
use thiserror::Error;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use crate::manifest_snapshot_schema::{
    BinaryPlatform, BinarySnapshot, CommandSnapshot, SnapshotV1,
};

#[derive(Debug, Parser)]
pub struct Args {
    /// Path to the `depth_fixture` binary to snapshot.
    #[arg(long)]
    pub depth_fixture_binary: PathBuf,
    /// Output snapshot file: `<root>/snapshots/<version>/<target>.json`.
    #[arg(long)]
    pub out_file: PathBuf,
    /// Also write raw `--help` output under `<root>/raw_help/<version>/<target>/`.
    #[arg(long)]
    pub capture_raw_help: bool,
    /// Target for raw help capture. Must equal the out-file's target when given.
    #[arg(long)]
    pub raw_help_target: Option<String>,
    /// Accepted for the shared argument shape; the fixture has no supplement.
    #[arg(long)]
    pub supplement: Option<PathBuf>,
    /// Override `collected_at` (RFC3339). Otherwise `SOURCE_DATE_EPOCH`, then now.
    #[arg(long)]
    pub collected_at: Option<String>,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid depth_fixture binary path: {0}")]
    InvalidBinary(PathBuf),
    #[error("invalid --out-file layout: {0}")]
    Layout(String),
    #[error("depth_fixture has no supplement; remove {0}")]
    Supplement(PathBuf),
    #[error("`{0}` failed: {1}")]
    CommandFailed(String, String),
    #[error("unexpected version output: {0:?}")]
    Version(String),
    #[error("invalid collected_at (must be RFC3339): {0}")]
    CollectedAt(String),
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("failed to parse JSON: {0}")]
    Json(#[from] serde_json::Error),
}

pub fn run(args: Args) -> Result<(), Error> {
    if let Some(path) = args.supplement {
        return Err(Error::Supplement(path));
    }
    let binary = fs::canonicalize(&args.depth_fixture_binary)
        .ok()
        .filter(|p| p.is_file())
        .ok_or_else(|| Error::InvalidBinary(args.depth_fixture_binary.clone()))?;

    let version_output = invoke(&binary, &["--version"])?.trim().to_owned();
    let version = version_output
        .strip_prefix("depth_fixture ")
        .filter(|v| semver::Version::parse(v).is_ok())
        .ok_or_else(|| Error::Version(version_output.clone()))?
        .to_owned();
    let (root, target) = layout(&args.out_file, &version)?;
    if args.raw_help_target.as_deref().is_some_and(|t| t != target) {
        return Err(Error::Layout(format!(
            "--raw-help-target differs from the out-file target `{target}`"
        )));
    }

    let mut commands: Vec<CommandSnapshot> =
        serde_json::from_str(&invoke(&binary, &["--surface-json"])?)?;
    commands.sort_by(|a, b| a.path.cmp(&b.path));
    if args.capture_raw_help {
        let dir = root.join("raw_help").join(&version).join(&target);
        fs::create_dir_all(&dir)?;
        fs::write(dir.join("help.txt"), invoke(&binary, &["--help"])?)?;
        fs::write(
            dir.join("run.help.txt"),
            invoke(&binary, &["run", "--help"])?,
        )?;
    }

    let bytes = fs::read(&binary)?;
    let snapshot = SnapshotV1 {
        snapshot_schema_version: 1,
        tool: "depth-fixture-cli".to_owned(),
        collected_at: collected_at(args.collected_at)?,
        binary: BinarySnapshot {
            sha256: hex::encode(Sha256::digest(&bytes)),
            size_bytes: bytes.len() as u64,
            platform: BinaryPlatform {
                os: std::env::consts::OS.to_owned(),
                arch: std::env::consts::ARCH.to_owned(),
            },
            target_triple: target,
            version_output,
            semantic_version: Some(version),
            channel: None,
            commit: None,
        },
        commands,
        features: None,
        known_omissions: None,
    };
    if let Some(parent) = args.out_file.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        &args.out_file,
        format!("{}\n", serde_json::to_string_pretty(&snapshot)?),
    )?;
    Ok(())
}

/// The producer's complete raw-help inventory and writer, also used by bounded admission.
pub(crate) fn write_raw_help(dir: &Path, help: &str, run_help: &str) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    fs::write(dir.join("help.txt"), help)?;
    fs::write(dir.join("run.help.txt"), run_help)
}

fn invoke(binary: &Path, args: &[&str]) -> Result<String, Error> {
    let output = Command::new(binary)
        .args(args)
        .env_clear()
        .env("NO_COLOR", "1")
        .output()?;
    let shown = format!("depth_fixture {}", args.join(" "));
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        return Err(Error::CommandFailed(shown, stderr));
    }
    String::from_utf8(output.stdout)
        .map_err(|_| Error::CommandFailed(shown, "stdout is not UTF-8".to_owned()))
}

fn name(path: Option<&Path>) -> Option<&str> {
    path.and_then(Path::file_name).and_then(|n| n.to_str())
}

/// Returns the manifest root and target from `<root>/snapshots/<version>/<target>.json`.
fn layout(out_file: &Path, version: &str) -> Result<(PathBuf, String), Error> {
    let version_dir = out_file.parent();
    let snapshots_dir = version_dir.and_then(Path::parent);
    if name(version_dir) != Some(version) || name(snapshots_dir) != Some("snapshots") {
        return Err(Error::Layout(format!(
            "expected <root>/snapshots/{version}/<target>.json, got {}",
            out_file.display()
        )));
    }
    let target = name(Some(out_file))
        .and_then(|n| n.strip_suffix(".json"))
        .filter(|t| !t.is_empty())
        .ok_or_else(|| Error::Layout("expected <target>.json".to_owned()))?;
    let root = snapshots_dir
        .and_then(Path::parent)
        .unwrap_or(Path::new("."));
    Ok((root.to_path_buf(), target.to_owned()))
}

fn collected_at(explicit: Option<String>) -> Result<String, Error> {
    if let Some(value) = explicit {
        OffsetDateTime::parse(&value, &Rfc3339).map_err(|_| Error::CollectedAt(value.clone()))?;
        return Ok(value);
    }
    let now = std::env::var("SOURCE_DATE_EPOCH")
        .ok()
        .and_then(|v| v.parse::<i64>().ok())
        .and_then(|secs| OffsetDateTime::from_unix_timestamp(secs).ok())
        .unwrap_or_else(OffsetDateTime::now_utc);
    Ok(now
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_owned()))
}
