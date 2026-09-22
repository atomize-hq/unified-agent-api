//! Has automation lost authority over a packet generation?
//!
//! The nightly watcher re-dispatches `agent-maintenance-open-pr` for every agent whose validated
//! pointer still trails upstream, and the pointer only advances at promotion — so an open packet
//! reads as needed every night for as long as it stays open. Each dispatch regenerates the request
//! with a fresh `request_recorded_at` and lets `peter-evans/create-pull-request` reset the packet
//! branch to base, re-apply the packet, and force-push. Anything a maintainer committed during the
//! day is gone, and a closeout that was never committed stops validating anyway, because
//! `request.rs` digests the whole request file and `closeout/validate.rs` rejects a mismatch.
//!
//! This command answers the one question every destructive step needs answered first. It is a
//! read-only ownership predicate, deliberately *not* a validity check: a packet is protected
//! because a maintainer declared it protected, never because it currently holds a closeout that
//! validates. The late reading would still permit erasing half-finished categorization work, a
//! partially authored closeout, or a valid one during an intentional edit.
//!
//! Four design points are load-bearing and easy to get wrong:
//!
//! - **Markers are a sibling of the request, not a field in it.** Declaring a stand-down must not
//!   change the request's bytes, or the act of protecting a closeout would invalidate it.
//! - **One file per frozen generation, named for it.** Two packet PRs for one agent can be open at
//!   once — supersession exists because of it — so two freezes must be able to coexist. A single
//!   per-agent file would make freezing the newer packet silently revoke the older one's
//!   protection, with no diff signal beyond a one-line edit.
//! - **A directory that cannot be read in full stands automation down.** An entry that is
//!   malformed, misnamed, or written for another agent means the freeze set is unknown, and an
//!   unknown freeze set is not an authorization. This is what keeps a typo from reading as
//!   permission: `v0.155.0.toml` is rejected rather than quietly never matching.
//! - **The marker directory is read from base, never from the packet branch.** The packet branch is
//!   inside the `add-paths` tree that the force-push replaces, so a marker carried there could be
//!   destroyed by the operation it exists to block. Base is never written by the nightly run.
//!   `--from-ref` exists so a caller can re-read base *fresh* immediately before mutating, rather
//!   than trusting a checkout taken minutes earlier; the callers, not this module, choose when that
//!   matters.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use clap::Parser;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use thiserror::Error;
use toml_edit::de::from_str;

use crate::agent_registry::AgentRegistry;

const EXIT_INTERNAL: i32 = 1;
const EXIT_VALIDATION: i32 = 2;

/// Exit code meaning automation has lost authority over the packet generation asked about.
///
/// This is a result, not a failure — the same split `maintenance-audit-status` draws with
/// `EXIT_UPLIFTS_REQUIRED`. A nightly run that stands down did its job. A caller must not collapse
/// it with the other non-zero exits, which mean the guard could not answer: standing down is
/// correct for both, but only one of them is a healthy outcome.
pub const EXIT_STOOD_DOWN: i32 = 3;

/// Marker directory, relative to the agent's maintenance root. One `<target_version>.toml` per
/// frozen generation.
pub const STAND_DOWN_RELATIVE_DIR: &str = "governance/automation-stand-down";

/// The branch a marker has to be committed to.
///
/// It is the branch the packet PR targets, and the only one the nightly force-push does not
/// replace. The packet workflow hardcodes the same value as its `base:`; `c4_spec_ci_wiring` binds
/// the two together, because a rename that moved one without the other would send every declared
/// freeze to a branch nothing reads — a false authorization wearing the shape of a declaration.
pub const BASE_BRANCH: &str = "staging";

const REQUEST_RELATIVE_PATH: &str = "governance/maintenance-request.toml";
const SCHEMA_VERSION: u32 = 1;

/// Where a marker lives, given the packet's own maintenance root.
///
/// Takes the root rather than the agent id so a caller holding a request uses *that* packet's
/// root instead of re-deriving one, and so this module keeps a single definition of the layout.
pub fn marker_relative_path(maintenance_root: &str, target_version: &str) -> String {
    format!("{maintenance_root}/{STAND_DOWN_RELATIVE_DIR}/{target_version}.toml")
}

/// Render a paste-ready marker for one packet generation.
///
/// Every producer of this text goes through here. There are two — the acquisition step in the
/// generated `HANDOFF.md`, and the admission gate's refusal output under `uaa-0039` — and two
/// hand-written copies of the schema is precisely how an instruction drifts away from the parser
/// that has to accept it. One producer, one consumer (`validate_entry`), round-tripped in tests.
///
/// `reason` is a default, not a fact: whoever pastes this is expected to say what they are doing.
/// Any reason is accepted as long as it is non-empty, which is the only thing the parser can
/// meaningfully check.
pub fn render_marker_toml(agent: &str, target_version: &str, request_recorded_at: &str) -> String {
    format!(
        "schema_version = {SCHEMA_VERSION}\n\
         agent_id = \"{agent}\"\n\
         target_version = \"{target_version}\"\n\
         reason = \"closeout in progress\"\n\
         request_recorded_at = \"{request_recorded_at}\"\n"
    )
}

#[derive(Debug, Parser, Clone)]
pub struct Args {
    /// Agent whose packet is being asked about.
    #[arg(long)]
    pub agent: String,

    /// The packet generation being asked about.
    ///
    /// A marker naming a different version does not protect this one: opening `0.156.0` while
    /// `0.155.0` is frozen is a legitimate lifecycle step. Supersession passes the *candidate's*
    /// version here rather than the run's, which is what keeps a newer run from closing a packet
    /// a maintainer is mid-closeout on.
    #[arg(long)]
    pub target_version: String,

    /// Read the marker directory from this git ref instead of the working tree.
    ///
    /// A workflow that checked out base minutes ago is answering from a snapshot, so a marker a
    /// maintainer lands mid-run is invisible to it. Pass a freshly fetched ref here at the boundary
    /// that actually mutates the remote. Any git failure is an internal error, never an absence.
    #[arg(long)]
    pub from_ref: Option<String>,

    /// Workspace root holding the committed packet (default: cwd).
    #[arg(long)]
    pub workspace_root: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StandDownOutcome {
    /// No marker claims this packet generation; automation may mutate it.
    Authorized,
    /// A marker claims it; automation must not mutate it.
    StoodDown,
}

impl StandDownOutcome {
    pub fn exit_code(self) -> i32 {
        match self {
            Self::Authorized => 0,
            Self::StoodDown => EXIT_STOOD_DOWN,
        }
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    Internal(String),
}

impl Error {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Validation(_) => EXIT_VALIDATION,
            Self::Internal(_) => EXIT_INTERNAL,
        }
    }
}

/// A declared stand-down.
///
/// `deny_unknown_fields` is not pedantry here. Someone who writes a field expecting it to widen
/// the protection has to find out at once; silently ignoring it would grant a narrower protection
/// than the author believed they were declaring.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawStandDown {
    schema_version: u32,
    agent_id: String,
    /// Must equal the file's own name. Redundant on purpose: it is what catches a marker copied
    /// from another generation and then only half-edited.
    target_version: String,
    reason: String,
    /// Copy of `request_recorded_at` from the request this stand-down froze. Either this or
    /// `request_sha256` is required: without one, the marker names a packet but not a generation.
    #[serde(default)]
    request_recorded_at: Option<String>,
    /// Digest of the frozen request file, as `request.rs` computes it.
    #[serde(default)]
    request_sha256: Option<String>,
    #[serde(default)]
    declared_at: Option<String>,
    #[serde(default)]
    declared_by: Option<String>,
}

/// Just enough of a request to compare the frozen generation against. Unknown fields are ignored
/// deliberately: this must not start failing because the request schema grew a field.
#[derive(Debug, Deserialize)]
struct RequestStamp {
    request_recorded_at: String,
}

pub fn run(args: Args) -> Result<StandDownOutcome, Error> {
    let workspace_root = match args.workspace_root.clone() {
        Some(root) => root,
        None => std::env::current_dir()
            .map_err(|err| Error::Internal(format!("could not resolve workspace root: {err}")))?,
    };

    // An unknown agent must not read as "no marker, proceed". Resolving the marker path by string
    // formatting alone would turn a typo into a false authorization, which is the one direction
    // this command may never fail in.
    let registry = AgentRegistry::load(&workspace_root)
        .map_err(|err| Error::Validation(format!("could not load the agent registry: {err}")))?;
    if registry.find(&args.agent).is_none() {
        return Err(Error::Validation(format!(
            "unknown agent `{}`: it is not in the agent registry, so no packet of that name exists",
            args.agent
        )));
    }

    let maintenance_root = format!("docs/agents/lifecycle/{}-maintenance", args.agent);
    let marker_dir = format!("{maintenance_root}/{STAND_DOWN_RELATIVE_DIR}");

    let entries = match args.from_ref.as_deref() {
        Some(git_ref) => read_marker_dir_from_ref(&workspace_root, git_ref, &marker_dir)?,
        None => read_marker_dir_from_tree(&workspace_root.join(&marker_dir))?,
    };

    // Every entry is validated, not just the one that might match. An entry this command cannot
    // read means the freeze set is unknown, and an unknown freeze set is not an authorization.
    let mut frozen = BTreeMap::new();
    for (file_name, text) in &entries {
        let marker = validate_entry(&args.agent, &marker_dir, file_name, text)?;
        frozen.insert(marker.target_version.clone(), marker);
    }

    let source = args
        .from_ref
        .as_deref()
        .map(|r| format!("{marker_dir} at {r}"))
        .unwrap_or_else(|| marker_dir.clone());

    let Some(marker) = frozen.get(&args.target_version) else {
        if frozen.is_empty() {
            println!(
                "No stand-down marker in {source}. Automation retains authority over {} {}.",
                args.agent, args.target_version
            );
        } else {
            println!(
                "{source} freezes {} {}, not {}. Automation retains authority over {}.",
                args.agent,
                frozen.keys().cloned().collect::<Vec<_>>().join(", "),
                args.target_version,
                args.target_version
            );
        }
        return Ok(StandDownOutcome::Authorized);
    };

    println!(
        "Automation has stood down from {} {}.",
        args.agent, args.target_version
    );
    println!(
        "  marker: {}",
        marker_relative_path(&maintenance_root, &args.target_version)
    );
    println!("  reason: {}", marker.reason);
    if let Some(declared_by) = marker.declared_by.as_deref() {
        println!("  declared by: {declared_by}");
    }
    if let Some(declared_at) = marker.declared_at.as_deref() {
        println!("  declared at: {declared_at}");
    }
    report_generation_binding(
        marker,
        &workspace_root.join(format!("{maintenance_root}/{REQUEST_RELATIVE_PATH}")),
        &workspace_root,
    );

    Ok(StandDownOutcome::StoodDown)
}

fn read_marker_dir_from_tree(dir: &Path) -> Result<Vec<(String, String)>, Error> {
    let read_dir = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => {
            return Err(Error::Internal(format!(
                "could not read {}: {err}",
                dir.display()
            )))
        }
    };

    let mut out = Vec::new();
    for entry in read_dir {
        let entry = entry
            .map_err(|err| Error::Internal(format!("could not read {}: {err}", dir.display())))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let text = fs::read_to_string(entry.path()).map_err(|err| {
            Error::Internal(format!("could not read {}: {err}", entry.path().display()))
        })?;
        out.push((name, text));
    }
    out.sort();
    Ok(out)
}

/// Read the marker directory out of a git ref without touching the working tree.
///
/// `prepare-agent-maintenance` has already rewritten the packet in the tree by the time the second
/// boundary asks, so checking the files out would mix a fresh marker into a prepared packet. Both
/// git calls fail into `Error::Internal`, which stands automation down: an unreadable ref is not an
/// empty one.
fn read_marker_dir_from_ref(
    workspace_root: &Path,
    git_ref: &str,
    marker_dir: &str,
) -> Result<Vec<(String, String)>, Error> {
    let listing = git(
        workspace_root,
        &["ls-tree", "-r", "--name-only", git_ref, "--", marker_dir],
    )?;

    let mut out = Vec::new();
    for path in listing.lines().map(str::trim).filter(|l| !l.is_empty()) {
        let name = path
            .rsplit('/')
            .next()
            .expect("rsplit always yields at least one segment")
            .to_string();
        out.push((
            name,
            git(workspace_root, &["show", &format!("{git_ref}:{path}")])?,
        ));
    }
    out.sort();
    Ok(out)
}

fn git(workspace_root: &Path, args: &[&str]) -> Result<String, Error> {
    let output = Command::new("git")
        .current_dir(workspace_root)
        .args(args)
        .output()
        .map_err(|err| Error::Internal(format!("could not run `git {}`: {err}", args.join(" "))))?;
    if !output.status.success() {
        return Err(Error::Internal(format!(
            "`git {}` failed ({}): {}",
            args.join(" "),
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    String::from_utf8(output.stdout).map_err(|err| {
        Error::Internal(format!("`git {}` emitted non-UTF-8: {err}", args.join(" ")))
    })
}

fn validate_entry(
    agent: &str,
    marker_dir: &str,
    file_name: &str,
    text: &str,
) -> Result<RawStandDown, Error> {
    let rendered = format!("{marker_dir}/{file_name}");

    // The file name is the match key, so a name this command cannot recognise is a marker that
    // could never match — which is a typo presenting as protection, not a marker for some other
    // generation. Strict `MAJOR.MINOR.PATCH` mirrors what the workflow already enforces on every
    // version it acts on, so nothing outside that set is ever a real generation.
    let Some(version) = file_name
        .strip_suffix(".toml")
        .filter(|v| is_bare_semver(v))
    else {
        return Err(Error::Validation(format!(
            "{rendered} is not named for a packet generation: expected \
             `<MAJOR.MINOR.PATCH>.toml`\n{EXPECTED_SHAPE}"
        )));
    };

    let marker: RawStandDown = from_str(text).map_err(|err| {
        Error::Validation(format!(
            "{rendered} is not a usable stand-down marker: {err}\n{EXPECTED_SHAPE}"
        ))
    })?;

    if marker.schema_version != SCHEMA_VERSION {
        return Err(Error::Validation(format!(
            "{rendered} declares schema_version {} but this build understands {SCHEMA_VERSION}",
            marker.schema_version
        )));
    }

    // The directory already names the agent, so a mismatch means the file was copied from another
    // packet. Reading it as authoritative for whichever agent it happens to sit under would
    // silently apply one packet's freeze to a different one.
    if marker.agent_id != agent {
        return Err(Error::Validation(format!(
            "{rendered} declares agent_id `{}` but sits in {agent}'s maintenance root",
            marker.agent_id
        )));
    }

    if marker.target_version != version {
        return Err(Error::Validation(format!(
            "{rendered} declares target_version `{}`, which is not the generation its file name \
             claims",
            marker.target_version
        )));
    }

    if marker.reason.trim().is_empty() {
        return Err(Error::Validation(format!(
            "{rendered} has an empty `reason`"
        )));
    }

    if marker.request_recorded_at.is_none() && marker.request_sha256.is_none() {
        return Err(Error::Validation(format!(
            "{rendered} must carry `request_recorded_at` or `request_sha256`: without one it names \
             a packet but not the generation of it that was frozen\n{EXPECTED_SHAPE}"
        )));
    }

    Ok(marker)
}

/// Mirror of the `^[0-9]+\.[0-9]+\.[0-9]+$` the packet workflow enforces on every version it acts
/// on. Deliberately stricter than `semver::Version::parse`, which accepts prereleases and build
/// metadata that can never be a packet generation here.
fn is_bare_semver(value: &str) -> bool {
    let mut parts = value.split('.');
    matches!(
        (parts.next(), parts.next(), parts.next(), parts.next()),
        (Some(major), Some(minor), Some(patch), None)
            if [major, minor, patch]
                .iter()
                .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
    )
}

/// Say whether the frozen generation is still the one on disk.
///
/// Reported, never decided on. A stand-down that no longer matches the live request is exactly the
/// case where a maintainer most needs automation to keep its hands off — the request has already
/// moved under them — so a mismatch must not read as permission to proceed.
fn report_generation_binding(marker: &RawStandDown, request_path: &Path, workspace_root: &Path) {
    let rendered = request_path
        .strip_prefix(workspace_root)
        .unwrap_or(request_path)
        .display()
        .to_string();

    let bytes = match fs::read(request_path) {
        Ok(bytes) => bytes,
        Err(err) => {
            if let Some(recorded_at) = marker.request_recorded_at.as_deref() {
                println!("  frozen request_recorded_at: {recorded_at}");
            }
            if let Some(sha) = marker.request_sha256.as_deref() {
                println!("  frozen request_sha256: {sha}");
            }
            println!("  note: could not read {rendered} to compare ({err})");
            return;
        }
    };

    if let Some(expected) = marker.request_recorded_at.as_deref() {
        println!("  frozen request_recorded_at: {expected}");
        match from_str::<RequestStamp>(&String::from_utf8_lossy(&bytes)) {
            Ok(stamp) => report_match(expected, &stamp.request_recorded_at, &rendered),
            Err(err) => println!("  note: could not read {rendered}'s own timestamp ({err})"),
        }
    }

    if let Some(expected) = marker.request_sha256.as_deref() {
        println!("  frozen request_sha256: {expected}");
        report_match(expected, &hex::encode(Sha256::digest(&bytes)), &rendered);
    }
}

fn report_match(expected: &str, actual: &str, rendered: &str) {
    if expected == actual {
        println!("    the frozen request is still the one on this branch");
    } else {
        println!("    note: {rendered} now reads {actual}, so the frozen generation is no longer the one on this branch");
    }
}

const EXPECTED_SHAPE: &str = "\
Expected `<maintenance root>/governance/automation-stand-down/<MAJOR.MINOR.PATCH>.toml`, e.g.
`docs/agents/lifecycle/codex-maintenance/governance/automation-stand-down/0.155.0.toml`:

    schema_version = 1
    agent_id = \"codex\"
    target_version = \"0.155.0\"       # must equal the file name
    reason = \"closeout in progress\"
    request_recorded_at = \"2026-09-19T08:03:11Z\"  # or request_sha256; at least one
    declared_by = \"@spenquatch\"                   # optional
    declared_at = \"2026-09-21T14:02:00Z\"          # optional";

#[cfg(test)]
#[path = "stand_down/tests.rs"]
mod tests;
