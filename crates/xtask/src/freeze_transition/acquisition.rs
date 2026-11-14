//! Sole constructor of non-empty acquisition deltas. Callers must authenticate observations
//! through the protected host before supplying the complete observed output inventory.
use super::{Entry, Tree};
use std::collections::BTreeMap;

pub(crate) type Inventory = BTreeMap<String, Option<Entry>>;
#[derive(Debug, Clone)]
pub(crate) struct ValidatedAcquisitionDelta {
    before: Inventory,
    after: Inventory,
}
impl ValidatedAcquisitionDelta {
    pub(crate) fn empty() -> Self {
        Self {
            before: Inventory::new(),
            after: Inventory::new(),
        }
    }
    pub(crate) fn apply(&self, predecessor: &Tree) -> Result<Tree, String> {
        let mut tree = predecessor.clone();
        for (path, entry) in &self.after {
            if predecessor.get(path) != self.before[path].as_ref() {
                return Err("acquisition predecessor changed".into());
            }
            match entry {
                Some(entry) => {
                    tree.insert(path.clone(), entry.clone());
                }
                None => {
                    tree.remove(path);
                }
            }
        }
        super::tree::validate_inventory(&tree)?;
        Ok(tree)
    }
    pub(crate) fn owns(&self, path: &str) -> bool {
        self.after.contains_key(path)
    }
}

/// No observation grants only preservation, including authenticated absence.
pub(crate) fn validate(
    predecessor: &Tree,
    candidate: &Tree,
    output_paths: &[String],
) -> Result<ValidatedAcquisitionDelta, String> {
    if output_paths
        .iter()
        .any(|path| predecessor.get(path) != candidate.get(path))
    {
        return Err("acquisition authority unavailable".into());
    }
    Ok(ValidatedAcquisitionDelta::empty())
}

/// The protected acquisition boundary supplies an independently authenticated complete manifest,
/// never a candidate fingerprint. This comparison binds every byte, absence, mode and kind.
pub(crate) fn authenticated(
    predecessor: &Tree,
    candidate: &Tree,
    output_paths: &[String],
    observed: &Inventory,
) -> Result<ValidatedAcquisitionDelta, String> {
    if observed.len() != output_paths.len()
        || output_paths.iter().any(|p| !observed.contains_key(p))
    {
        return Err("acquisition inventory differs from resolved plan".into());
    }
    let mut before = Inventory::new();
    for path in output_paths {
        super::tree::validate_path(path)?;
        let entry = &observed[path];
        if entry.as_ref() != candidate.get(path) || entry.as_ref().is_some_and(|e| !e.regular()) {
            return Err("acquisition bytes/presence/mode/kind differ from observation".into());
        }
        before.insert(path.clone(), predecessor.get(path).cloned());
    }
    let delta = ValidatedAcquisitionDelta {
        before,
        after: observed.clone(),
    };
    delta.apply(predecessor)?;
    Ok(delta)
}
