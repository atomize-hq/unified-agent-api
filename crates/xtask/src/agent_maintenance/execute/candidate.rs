//! MA4 evidence policy over the neutral independent candidate boundary.
use super::{
    tree::{self, Node, Tree},
    types::Context,
    Error,
};
use crate::candidate_execution::Candidate as Independent;
use std::path::Path;
pub(super) struct Candidate {
    independent: Independent,
    pub context: Context,
    pub staged_evidence: Option<std::collections::BTreeMap<String, String>>,
}
impl std::ops::Deref for Candidate {
    type Target = Independent;
    fn deref(&self) -> &Self::Target {
        &self.independent
    }
}
impl Candidate {
    pub fn prepare(caller: &Path, context: &Context) -> Result<Self, Error> {
        let independent = Independent::prepare(caller, &context.run_dir_rel, vec![])?;
        let root = &independent.root;
        let mut ctx = context.clone();
        ctx.run_dir = root.join(&ctx.run_dir_rel);
        if ctx.codex_binary.contains('/') || ctx.codex_binary.contains('\\') {
            let binary = Path::new(&ctx.codex_binary);
            let full = if binary.is_absolute() {
                binary.to_owned()
            } else {
                caller.join(binary)
            };
            ctx.codex_binary = if let Ok(relative) = full.strip_prefix(caller) {
                root.join(relative)
            } else {
                full
            }
            .to_string_lossy()
            .into_owned();
        }
        Ok(Self {
            independent,
            context: ctx,
            staged_evidence: None,
        })
    }
    pub fn observe(&self) -> Result<Tree, Error> {
        self.independent.observe().map_err(Into::into)
    }
    pub fn check(&self, after: &Tree) -> Result<(), Error> {
        self.independent.check(after)?;
        let paths: Vec<_> = tree::changed(&self.before, after)
            .into_iter()
            .filter(|p| {
                matches!(after.get(p), Some(Node::File(..)))
                    && !p.starts_with(&format!("{}/", self.context.run_dir_rel))
            })
            .collect();
        let mut checks = Vec::new();
        let mut errors = Vec::new();
        super::validate::validate_written_paths(
            &self.root,
            &self.context,
            &paths,
            "relay",
            &mut checks,
            &mut errors,
        )?;
        if !errors.is_empty() {
            return Err(Error::Validation(errors.join("\n")));
        }
        // The host-side byte binding, by path class: every added, changed or removed evidence
        // file (evidence destinations and everything an E binds, transitively) is the
        // host-staged bytes, or the candidate is refused.
        super::delivery_index::check_staged_closure(
            &self.before,
            after,
            &self.context.run_dir_rel,
            self.staged_evidence.as_ref(),
        )?;
        let indexed = super::delivery_index::bound_new_evidence(
            &self.before,
            after,
            &self.context.run_dir_rel,
            self.staged_evidence.as_ref(),
        );
        self.independent.validate_index(&indexed, || {
            crate::depth_acceptance::check_captured_relay(
                tree::source(&self.root, &self.before)?,
                tree::source(&self.root, after)?,
                &self.root,
            )
            .map_err(crate::candidate_execution::Error::Validation)
        })?;
        // Live publisher validators ran above. Refuse if any executor-visible file changed during them.
        if &self.observe()? != after {
            return Err(Error::Validation(
                "candidate changed during relay validation".into(),
            ));
        }
        Ok(())
    }
    pub fn deliver(&self, caller: &Path, after: &Tree, passed: bool) -> Result<Vec<String>, Error> {
        self.independent
            .deliver(caller, after, passed, self.staged_evidence.as_ref())
            .map_err(Into::into)
    }
}

impl From<crate::candidate_execution::Error> for Error {
    fn from(error: crate::candidate_execution::Error) -> Self {
        match error {
            crate::candidate_execution::Error::Validation(s) => Self::Validation(s),
            crate::candidate_execution::Error::Internal(s) => Self::Internal(s),
        }
    }
}
