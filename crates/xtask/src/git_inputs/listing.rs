//! Index-name snapshots bracket each traversal/recheck; directories are checked in order.
use super::*;
use std::sync::OnceLock;

#[derive(Default)]
pub(super) struct Listing(OnceLock<Option<Vec<Vec<u8>>>>);

impl Listing {
    pub(super) fn snapshot(root: &Path, borrowed: bool) -> Self {
        let listing = Self::default();
        if !borrowed {
            listing.paths(root);
        }
        listing
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
    expected: &BTreeMap<String, Vec<String>>,
    observed: Vec<(String, Result<Vec<String>, String>)>,
) -> Result<(), String> {
    if observed.is_empty() {
        return Ok(());
    }
    let end = Listing::snapshot(root, borrowed.is_some());
    // Keep the original directory order even if an earlier directory changes after its
    // first observation. A fresh ending index catches staged additions with no working file.
    for (path, start_names) in observed {
        let start_names = start_names?;
        let end_names = list_cached(root, &path, borrowed, Some(&end))?;
        if expected.get(&path) != Some(&start_names) || start_names != end_names {
            return Err(format!(
                "acquisition directory changed during freeze: {path}"
            ));
        }
    }
    Ok(())
}
