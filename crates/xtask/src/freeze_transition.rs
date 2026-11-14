//! Exact freeze effects. Input acquisition authority is deliberately empty-only in FR2a.
use std::collections::BTreeMap;

#[path = "freeze_transition/acquisition.rs"]
pub(crate) mod acquisition;
#[path = "freeze_transition/delivery.rs"]
pub(crate) mod delivery;
#[path = "freeze_transition/tree.rs"]
pub(crate) mod tree;
pub(crate) use acquisition::ValidatedAcquisitionDelta;
pub(crate) use tree::{Entry, Tree};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Operation {
    Create(Entry),
    Replace { before: Entry, after: Entry },
    Delete(Entry),
    Preserve(Entry),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Plan {
    pub operations: BTreeMap<String, Operation>,
    pub expected: Tree,
}

/// Functional core shared by delivery and IN1. Output bytes come from the owning approved
/// renderers. Every untouched leaf, including symlinks and gitlinks, is explicitly preserved.
/// No filesystem, clock, environment, policy selection or executable dispatch occurs here.
pub(crate) fn operations(
    predecessor: &Tree,
    _acquisition: &ValidatedAcquisitionDelta,
    rendered: impl IntoIterator<Item = (String, Option<Vec<u8>>)>,
) -> Result<Plan, String> {
    let mut expected = predecessor.clone();
    let mut operations = predecessor
        .iter()
        .map(|(path, entry)| (path.clone(), Operation::Preserve(entry.clone())))
        .collect::<BTreeMap<_, _>>();
    let mut seen = std::collections::BTreeSet::new();
    for (path, bytes) in rendered {
        tree::validate_path(&path)?;
        if !seen.insert(path.clone()) {
            return Err("duplicate freeze output".into());
        }
        let before = predecessor.get(&path);
        let operation = match bytes {
            Some(bytes) => {
                if before.is_some_and(|entry| !entry.regular()) {
                    return Err("freeze output replaces a nonregular object".into());
                }
                let after = Entry::file(before.map_or("100644", |e| &e.mode), bytes);
                expected.insert(path.clone(), after.clone());
                match before {
                    Some(before) if *before == after => Operation::Preserve(after),
                    Some(before) => Operation::Replace {
                        before: before.clone(),
                        after,
                    },
                    None => Operation::Create(after),
                }
            }
            None => {
                let before = before.ok_or("freeze deletion lacks a predecessor")?;
                expected.remove(&path);
                Operation::Delete(before.clone())
            }
        };
        operations.insert(path, operation);
    }
    tree::validate_inventory(&expected)?;
    Ok(Plan {
        operations,
        expected,
    })
}

impl Plan {
    pub(crate) fn require_equal(&self, candidate: &Tree) -> Result<(), String> {
        if self.expected != *candidate {
            return Err("freeze complete tree mismatch".into());
        }
        Ok(())
    }
}

/// Implemented by the owning approved derivation modules. Implementations render frozen
/// data only. Their input constructors remain private to those modules.
pub(crate) trait WriterInputs {
    type Request;
    fn render(&self) -> Result<Rendered<Self::Request>, String>;
}
pub(crate) struct Rendered<R> {
    pub request: Option<R>,
    pub files: Vec<(String, Vec<u8>)>,
}
pub(crate) struct Planned<R> {
    pub rendered: Rendered<R>,
    pub transition: Plan,
}
/// Shared pure freeze planner. Acquisition construction and writer-input derivation occur at
/// the caller's authenticated boundary. This function renders every effect and preserves the
/// entire supplied predecessor without reading a clock, environment, worktree or Git ref.
pub(crate) fn plan<I: WriterInputs>(
    predecessor: &Tree,
    acquisition: &ValidatedAcquisitionDelta,
    inputs: &I,
) -> Result<Planned<I::Request>, String> {
    let rendered = inputs.render()?;
    let transition = operations(
        predecessor,
        acquisition,
        rendered
            .files
            .iter()
            .map(|(path, bytes)| (path.clone(), Some(bytes.clone()))),
    )?;
    Ok(Planned {
        rendered,
        transition,
    })
}

#[cfg(test)]
#[path = "freeze_transition/tests.rs"]
mod tests;
