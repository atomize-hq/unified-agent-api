//! CP-P1: choose admission or observation from authenticated facts, never invocation labels.
use crate::{git_inputs, isolated_maintenance::Context};
use std::path::Path;

pub(super) struct Binding {
    pub(super) parent: String,
    head: String,
    tip: String,
}

fn error(message: impl std::fmt::Display) -> String {
    format!("isolated IN1 binding: {message}")
}

impl Binding {
    pub(super) fn recheck(&self, root: &Path, context: &Context) -> Result<(), String> {
        // Re-authenticate captured T, then check local refs after the potentially slow host
        // reads. P remains the captured object id even if the caller's ref moves and returns.
        if context.protected_tip(root).map_err(error)? != self.tip
            || super::resolve(root, "HEAD").map_err(error)? != self.head
        {
            return Err(error(
                "integration HEAD or authenticated protected tip moved",
            ));
        }
        self.check_local_tip(root).map_err(error)
    }

    fn check_local_tip(&self, root: &Path) -> Result<(), String> {
        // Exact lookup: `rev-parse` would fall back to a nested tag or branch spelled
        // `refs/remotes/origin/staging` when the tracking ref itself is absent.
        let output = git_inputs::command(root)
            .args([
                "show-ref",
                "--verify",
                "--hash",
                "refs/remotes/origin/staging",
            ])
            .output()
            .map_err(|e| e.to_string())?;
        if !output.status.success() {
            return Err("local integration ref refs/remotes/origin/staging is absent".into());
        }
        if String::from_utf8_lossy(&output.stdout).trim() != self.tip {
            return Err(
                "local integration ref differs from the authenticated protected tip".into(),
            );
        }
        Ok(())
    }
}

pub(super) fn capture(root: &Path, revision: &str, context: &Context) -> Result<Binding, String> {
    capture_inner(root, revision, context).map_err(error)
}

fn capture_inner(root: &Path, revision: &str, context: &Context) -> Result<Binding, String> {
    let tip = context.protected_tip(root)?.to_owned();
    let parent = super::resolve(root, revision)?;
    let head = super::resolve(root, "HEAD")?;
    let binding = Binding { parent, head, tip };
    binding.check_local_tip(root)?;
    let Binding { parent, head, tip } = &binding;
    if head == parent {
        return Err("HEAD equals the supplied parent".into());
    }
    if parent == tip && head != tip {
        let status = git_inputs::command(root)
            .args(["merge-base", "--is-ancestor", parent, head])
            .status()
            .map_err(|e| e.to_string())?;
        if !status.success() {
            return Err("candidate HEAD does not descend from the authenticated tip".into());
        }
    } else if head == tip {
        // Read the commit object named by authenticated T itself. For a merge only its first
        // parent is eligible; a grandparent, second parent or event-supplied claim is not.
        let output = git_inputs::command(root)
            .args(["cat-file", "commit", tip])
            .output()
            .map_err(|e| e.to_string())?;
        if !output.status.success() {
            return Err("authenticated tip commit is unreadable".into());
        }
        let first = output
            .stdout
            .split(|b| *b == b'\n')
            .take_while(|line| !line.is_empty())
            .find_map(|line| line.strip_prefix(b"parent "));
        if first != Some(parent.as_bytes()) {
            return Err("observation parent is not the authenticated tip's first parent".into());
        }
    } else {
        return Err("neither admission nor single-first-parent observation is bound".into());
    }
    context.recheck(root)?;
    Ok(binding)
}
