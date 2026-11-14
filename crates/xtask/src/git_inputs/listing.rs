//! One index-name observation per traversal/recheck; working directories are still read in order.
use super::*;
use std::sync::OnceLock;

#[derive(Default)]
pub(super) struct Listing(OnceLock<Option<Vec<Vec<u8>>>>);

impl Listing {
    pub(super) fn tracked(&self, root: &Path, prefix: &str) -> Option<Vec<u8>> {
        let paths = self
            .0
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
            .as_ref()?;
        let mut result = Vec::new();
        for path in paths.iter().filter(|p| p.starts_with(prefix.as_bytes())) {
            result.extend(path);
            result.push(0);
        }
        Some(result)
    }
}
