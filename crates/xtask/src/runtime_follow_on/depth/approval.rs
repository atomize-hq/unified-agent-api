//! Preserve the dry-run reader's declaration requirement around the shared finite projection.
use super::*;

pub(super) fn read(
    raw: &[u8],
    path: &str,
    captured: &CapturedRegistry,
) -> Result<OnboardingApproval, String> {
    let document: Value = toml_edit::de::from_str(text(raw)?).map_err(|e| e.to_string())?;
    let approval = OnboardingApproval::parse(text(raw)?, path)?;
    if approval.support_depth_version.is_none() {
        return Err("charter rule 3 unresolved: approval states no depth version".into());
    }
    let entry = captured
        .registry()
        .find(&approval.agent_id)
        .ok_or("unknown approval agent")?;
    if entry.backend_module.is_some() || entry.support_depth.is_none() {
        return Err(
            "private approval reader requires the compiled non-runtime depth subject".into(),
        );
    }
    crate::isolated_maintenance::onboarding::approval::validate(document, approval, entry)
}
