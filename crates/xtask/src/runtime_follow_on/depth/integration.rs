//! Integration authority uses one authenticated tip, with Source's git_inputs::command runner.
use crate::{
    depth_record::{acquisition, DepthRecord},
    runtime_advertising::source::Source,
    support_matrix::depth::covered_targets,
};

/// Keep the statement reader's complete-directory validation and shared parser, while ensuring
/// both enumeration and blob reads use the same hardened view as bound-input comparisons.
pub(super) fn statement(
    tip: &Source,
    pack: &str,
    agent: &str,
    version: &str,
) -> Result<Option<acquisition::AcquisitionStatement>, String> {
    let directory = acquisition::statement_dir(pack);
    let mut found = None;
    for path in tip.committed_files(&directory)? {
        let name = path
            .strip_prefix(&format!("{directory}/"))
            .filter(|name| !name.contains('/'))
            .ok_or_else(|| {
                format!("{path}: acquisition statement must be a direct directory entry")
            })?;
        if tip.executable(&path)? {
            return Err(format!(
                "{path}: acquisition statement must have mode 100644"
            ));
        }
        let statement = acquisition::AcquisitionStatement::parse(&tip.read(&path)?, agent, name)?;
        if statement.version == version {
            found = Some(statement);
        }
    }
    Ok(found)
}

/// Charter rule 4 adopts the registry targets rule, including coverage already integrated for
/// this version. A record in the onboarding branch alone cannot grant target authority.
pub(super) fn targets(
    tip: &Source,
    agent: &str,
    root: &str,
    version: &str,
    declared: &[String],
    expected: &[String],
) -> Result<(), String> {
    if declared.iter().all(|target| expected.contains(target)) {
        return Ok(());
    }
    let path = format!("{root}/reports/{version}/depth-record.json");
    let record = tip
        .optional(&path)?
        .map(|raw| DepthRecord::parse(&raw, agent, version))
        .transpose()?;
    let covered = record.as_ref().map(covered_targets).unwrap_or_default();
    let missing: Vec<_> = declared
        .iter()
        .filter(|target| !expected.contains(target) && !covered.contains(target.as_str()))
        .cloned()
        .collect();
    if !missing.is_empty() {
        return Err(format!(
            "charter rule 4: targets have neither RULES.json union.expected_targets nor \
             depth-record coverage at the authenticated integration tip: {}",
            missing.join(", ")
        ));
    }
    Ok(())
}
