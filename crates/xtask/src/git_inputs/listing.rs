//! Index-name snapshots bracket each traversal/recheck; directories are checked in order.
use super::*;
use std::sync::OnceLock;

#[derive(Default)]
pub(super) struct Listing(OnceLock<Option<Vec<Vec<u8>>>>);

impl Listing {
    pub(super) fn from_paths(paths: Option<Vec<Vec<u8>>>) -> Self {
        Self(OnceLock::from(paths))
    }

    fn paths(&self, root: &Path) -> Option<&Vec<Vec<u8>>> {
        self.0
            .get_or_init(|| {
                // Failures fall back to the original per-directory command for exact diagnostics.
                git(root, &["--literal-pathspecs", "ls-files", "-z"])
                    .ok()
                    .map(|raw| {
                        raw.split(|b| *b == 0)
                            .filter(|p| !p.is_empty())
                            .map(Vec::from)
                            .collect()
                    })
            })
            .as_ref()
    }

    pub(super) fn tracked(&self, root: &Path, prefix: &str) -> Option<Vec<u8>> {
        let paths = self.paths(root)?;
        let mut result = Vec::new();
        for path in paths.iter().filter(|p| p.starts_with(prefix.as_bytes())) {
            result.extend(path);
            result.push(0);
        }
        Some(result)
    }
}

pub(super) fn verify(
    root: &Path,
    borrowed: Option<&(PathBuf, String)>,
    files: &BTreeMap<String, Input>,
    expected: &BTreeMap<String, Vec<String>>,
    observed: Vec<(String, Result<Vec<String>, String>)>,
    session: &batch::Session<'_>,
    start: &Listing,
) -> Result<(), String> {
    // Finish the filesystem pass BEFORE capturing the ending index. Store errors so file
    // identity changes retain precedence over directory changes, in original recorded order.
    let final_names: Vec<_> = observed
        .iter()
        .map(|(path, _)| list_observed(root, path, borrowed, Some(start)))
        .collect();
    let (end, index_changed) = session.finish_index(files)?;
    for ((path, start_names), end_names) in observed.into_iter().zip(final_names) {
        let start_names = start_names?;
        let (final_names, mut working_names) = end_names?;
        if borrowed.is_none() {
            let tracked = end
                .tracked(root, &format!("{path}/"))
                .ok_or("missing ending Git index")?;
            extend_tracked_names(&mut working_names, &tracked, &format!("{path}/"))?;
        } else {
            working_names.extend(final_names.iter().cloned());
        }
        let end_names: Vec<_> = working_names.into_iter().collect();
        if expected.get(&path) != Some(&start_names)
            || start_names != final_names
            || start_names != end_names
        {
            return Err(format!(
                "acquisition directory changed during freeze: {path}"
            ));
        }
    }
    // Even unrelated index churn invalidates the complete pass snapshot. Prefer the first
    // differing recorded path above; otherwise attribute it to the first recorded input.
    if index_changed {
        if let Some(path) = files.keys().next() {
            return Err(format!("bound input changed during freeze: {path}"));
        }
        if let Some(path) = expected.keys().next() {
            return Err(format!(
                "acquisition directory changed during freeze: {path}"
            ));
        }
    }
    session.check_objects()
}
