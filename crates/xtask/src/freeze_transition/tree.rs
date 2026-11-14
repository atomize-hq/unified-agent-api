//! Complete leaf inventories preserve Git modes and object kinds as well as bytes.
use std::collections::BTreeMap;

pub(crate) type Tree = BTreeMap<String, Entry>;
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Entry {
    pub mode: String,
    pub kind: String,
    // Gitlinks carry the named commit id. Blob contents include symlink targets verbatim.
    pub bytes: Vec<u8>,
}
impl Entry {
    pub(crate) fn file(mode: &str, bytes: Vec<u8>) -> Self {
        Self {
            mode: mode.into(),
            kind: "blob".into(),
            bytes,
        }
    }
    pub(crate) fn regular(&self) -> bool {
        self.kind == "blob" && matches!(self.mode.as_str(), "100644" | "100755")
    }
}
pub(crate) fn validate_path(path: &str) -> Result<(), String> {
    if path.contains('\\')
        || path
            .split('/')
            .any(|p| matches!(p, "" | "." | ".." | ".git"))
    {
        return Err("invalid freeze tree path".into());
    }
    Ok(())
}
pub(crate) fn validate_inventory(tree: &Tree) -> Result<(), String> {
    for path in tree.keys() {
        validate_path(path)?;
        let mut parent = std::path::Path::new(path).parent();
        while let Some(path) = parent.filter(|p| !p.as_os_str().is_empty()) {
            if tree.contains_key(path.to_str().ok_or("non-UTF8 freeze path")?) {
                return Err("freeze tree has a leaf directory collision".into());
            }
            parent = path.parent();
        }
    }
    Ok(())
}
