//! Shared check-mode entry points for authenticated proposal consumers.
use super::*;

/// Read-only shared validation inside an already authenticated proposal/publication scope.
pub(crate) fn check_root(root: PathBuf) -> Result<(), String> {
    let description = root.display().to_string();
    let violations = run_inner(
        Args {
            root: Some(root),
            rules: None,
            schema: None,
            version_schema: None,
            mode: Mode::Check,
            json: false,
            tip: None,
        },
        None,
    )
    .map_err(|e| format!("manifest validation {description}: {e}"))?;
    if violations.is_empty() {
        return Ok(());
    }
    Err(violations
        .iter()
        .map(Violation::to_human_line)
        .collect::<Vec<_>>()
        .join("\n"))
}

/// Acquisition schema/pointer/report checks. The owning freeze separately derives and checks
/// depth records and complete aggregate publication, which do not exist before re-freeze.
pub(crate) fn check_acquisition(root: PathBuf) -> Result<(), String> {
    let args = Args {
        root: Some(root),
        rules: None,
        schema: None,
        version_schema: None,
        mode: Mode::Check,
        json: false,
        tip: None,
    };
    let violations = run_checks(args, None, false).map_err(|e| e.to_string())?;
    if !violations.is_empty() {
        return Err("manifest acquisition validation refused".into());
    }
    Ok(())
}
