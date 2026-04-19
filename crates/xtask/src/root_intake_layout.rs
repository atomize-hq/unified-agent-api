use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootIntakeLayout {
    root: PathBuf,
}

impl RootIntakeLayout {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn versions_dir(&self) -> PathBuf {
        self.root.join("versions")
    }

    pub fn latest_supported_pointers_dir(&self) -> PathBuf {
        self.root.join("pointers").join("latest_supported")
    }

    pub fn latest_supported_pointer_path(&self, target_triple: &str) -> PathBuf {
        self.latest_supported_pointers_dir()
            .join(format!("{target_triple}.txt"))
    }

    pub fn latest_validated_pointers_dir(&self) -> PathBuf {
        self.root.join("pointers").join("latest_validated")
    }

    pub fn latest_validated_pointer_path(&self, target_triple: &str) -> PathBuf {
        self.latest_validated_pointers_dir()
            .join(format!("{target_triple}.txt"))
    }

    pub fn reports_dir(&self) -> PathBuf {
        self.root.join("reports")
    }

    pub fn reports_version_dir(&self, version: &str) -> PathBuf {
        self.reports_dir().join(version)
    }
}

#[cfg(test)]
mod tests {
    use super::RootIntakeLayout;
    use std::path::PathBuf;

    #[test]
    fn root_intake_layout_is_shape_driven_for_current_agent_roots() {
        for root in ["cli_manifests/codex", "cli_manifests/claude_code"] {
            let layout = RootIntakeLayout::new(root);
            assert_eq!(layout.versions_dir(), PathBuf::from(root).join("versions"));
            assert_eq!(
                layout.latest_supported_pointers_dir(),
                PathBuf::from(root)
                    .join("pointers")
                    .join("latest_supported")
            );
            assert_eq!(
                layout.latest_supported_pointer_path("x86_64-unknown-linux-musl"),
                PathBuf::from(root)
                    .join("pointers")
                    .join("latest_supported")
                    .join("x86_64-unknown-linux-musl.txt")
            );
            assert_eq!(
                layout.latest_validated_pointers_dir(),
                PathBuf::from(root)
                    .join("pointers")
                    .join("latest_validated")
            );
            assert_eq!(
                layout.latest_validated_pointer_path("x86_64-unknown-linux-musl"),
                PathBuf::from(root)
                    .join("pointers")
                    .join("latest_validated")
                    .join("x86_64-unknown-linux-musl.txt")
            );
            assert_eq!(layout.reports_dir(), PathBuf::from(root).join("reports"));
            assert_eq!(
                layout.reports_version_dir("1.2.3"),
                PathBuf::from(root).join("reports").join("1.2.3")
            );
        }
    }
}
