//! Own only fresh empty no-match projects; keep the original context bound throughout.
use super::{context, Path};
use sha2::{Digest, Sha256};
use xtask::depth_evidence::native::context::guard::{LocalGuard, ProjectGuard};
pub struct Scenario {
    project: Option<tempfile::TempDir>,
    absent_id: Option<String>,
    local: Option<LocalGuard>,
    effects: Option<ProjectGuard>,
}
impl Scenario {
    pub fn new(step: &str, baseline: Option<&LocalGuard>) -> Result<Self, String> {
        let project = if step.starts_with("no-match-") {
            Some(
                tempfile::Builder::new()
                    .prefix("uaa-native-no-match-")
                    .tempdir()
                    .map_err(|_| "cannot create empty native scenario")?,
            )
        } else {
            None
        };
        // Disposable, random directory identity gives the unknown-id control a fresh UUID.
        // No source/credential bytes or path strings enter the retained finite observation.
        let absent_id = project.as_ref().map(|p| {
            let d = hex::encode(Sha256::digest(p.path().as_os_str().as_encoded_bytes()));
            format!(
                "{}-{}-4{}-8{}-{}",
                &d[..8],
                &d[8..12],
                &d[13..16],
                &d[17..20],
                &d[20..32]
            )
        });
        let local = project
            .as_ref()
            .zip(baseline)
            .map(|(p, g)| context::guard(Some(g.home()), p.path()))
            .transpose()?;
        let effects = project
            .as_ref()
            .filter(|_| baseline.is_some())
            .map(|p| ProjectGuard::inspect(p.path()).map_err(|e| e.to_string()))
            .transpose()?;
        Ok(Self {
            project,
            absent_id,
            local,
            effects,
        })
    }
    pub fn absent_id(&self) -> Option<&str> {
        self.absent_id.as_deref()
    }
    pub fn project(&self) -> Option<&Path> {
        self.project.as_ref().map(|p| p.path())
    }
    pub fn unchanged(&self, baseline: Option<&ProjectGuard>) -> Option<bool> {
        baseline
            .map(|p| p.unchanged() && self.effects.as_ref().map(|p| p.unchanged()).unwrap_or(true))
    }
    pub fn eligible(&self, baseline: Option<&LocalGuard>, effects: Option<&ProjectGuard>) -> bool {
        baseline
            .into_iter()
            .chain(self.local.as_ref())
            .all(|g| g.recheck(&context::environment()).is_ok())
            && self.unchanged(effects).unwrap_or(true)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_controls_own_distinct_empty_projects_and_valid_fresh_ids() {
        let a = Scenario::new("no-match-resume-id", None).unwrap();
        let b = Scenario::new("no-match-fork-last", None).unwrap();
        assert_ne!(a.project(), b.project());
        assert_ne!(a.absent_id(), b.absent_id());
        assert!(xtask::depth_evidence::native::native_session_id(
            a.absent_id().unwrap()
        ));
        assert_eq!(std::fs::read_dir(a.project().unwrap()).unwrap().count(), 0);
        assert!(a.eligible(None, None));
        assert!(a.unchanged(None).is_none());
        let normal = Scenario::new("initial", None).unwrap();
        assert!(normal.project().is_none());
    }
    #[test]
    fn empty_project_control_cannot_reset_original_project_effect_baseline() {
        let original = tempfile::tempdir().unwrap();
        let guard = ProjectGuard::inspect(original.path()).unwrap();
        let scenario = Scenario::new("no-match-resume-last", None).unwrap();
        assert!(scenario.eligible(None, Some(&guard)));
        std::fs::write(original.path().join("unexpected"), b"changed").unwrap();
        assert!(!scenario.eligible(None, Some(&guard)));
        assert_eq!(scenario.unchanged(Some(&guard)), Some(false));
    }
}
