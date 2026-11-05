//! CP-P1: choose admission or observation from authenticated facts, never invocation labels.
use crate::{git_inputs, isolated_maintenance::Context};
use std::path::Path;

pub(super) fn check(root: &Path, revision: &str, context: &Context) -> Result<(), String> {
    check_inner(root, revision, context).map_err(|e| format!("isolated IN1 binding: {e}"))
}

fn check_inner(root: &Path, revision: &str, context: &Context) -> Result<(), String> {
    let tip = context.protected_tip(root)?;
    if super::resolve(root, "origin/staging")? != tip {
        return Err("local integration ref differs from the authenticated protected tip".into());
    }
    let parent = super::resolve(root, revision)?;
    let head = super::resolve(root, "HEAD")?;
    if head == parent {
        return Err("HEAD equals the supplied parent".into());
    }
    if parent == tip && head != tip {
        let status = git_inputs::command(root)
            .args(["merge-base", "--is-ancestor", &parent, &head])
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
        let object = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
        let first = object
            .lines()
            .take_while(|line| !line.is_empty())
            .find_map(|line| line.strip_prefix("parent "));
        if first != Some(parent.as_str()) {
            return Err("observation parent is not the authenticated tip's first parent".into());
        }
    } else {
        return Err("neither admission nor single-first-parent observation is bound".into());
    }
    context.recheck(root)
}
