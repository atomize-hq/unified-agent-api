//! Compiled bootstrap inputs shared by public ON1 and independent creation replay.
use super::*;

pub(crate) struct Scaffold {
    pub(crate) entry: AgentRegistryEntry,
    pub(crate) approval_path: String,
    pub(crate) approval_bytes: Vec<u8>,
}

impl Adapter {
    pub(super) fn scaffold(&self, source: &Source) -> Result<Scaffold, String> {
        let (approval_bytes, _) = self.read_approval(source)?;
        Ok(Scaffold {
            entry: self.scaffold.clone(),
            approval_path: self.approval_path(),
            approval_bytes,
        })
    }
}

impl CapturedRegistry {
    pub(crate) fn onboarding_scaffold(&self, root: &Path) -> Result<Option<Scaffold>, String> {
        self.recheck(root)?;
        let context = self.route.context();
        let Some(adapter) = context.profile.onboarding()? else {
            return Ok(None);
        };
        if !adapter.undeclared(&context.registry) {
            return Ok(None);
        }
        let scaffold = adapter.scaffold(&Source::workspace(root)?)?;
        self.recheck(root)?;
        Ok(Some(scaffold))
    }
}
