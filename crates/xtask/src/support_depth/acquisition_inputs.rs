//! The acquisition derivation consumes the same bounded inputs in a checkout or immutable head.
use crate::{depth_record::FileRef, git_inputs::GitInputs, runtime_advertising::source::Source};

pub(crate) trait Inputs {
    fn optional(&mut self, path: &str) -> Result<Option<Vec<u8>>, String>;
    fn list(&mut self, path: &str) -> Result<Vec<String>, String>;
    fn required(&mut self, path: &str) -> Result<Vec<u8>, String> {
        self.optional(path)?
            .ok_or_else(|| format!("missing generation input: {path}"))
    }
    fn reference(&mut self, path: &str) -> Result<FileRef, String> {
        Ok(crate::depth_ci::reference(path, &self.required(path)?))
    }
}
impl Inputs for GitInputs {
    fn optional(&mut self, path: &str) -> Result<Option<Vec<u8>>, String> {
        GitInputs::optional(self, path)
    }
    fn list(&mut self, path: &str) -> Result<Vec<String>, String> {
        GitInputs::list(self, path)
    }
}
impl Inputs for Source {
    fn optional(&mut self, path: &str) -> Result<Option<Vec<u8>>, String> {
        self.optional_bytes(path)
    }
    fn list(&mut self, path: &str) -> Result<Vec<String>, String> {
        let prefix = format!("{path}/");
        Ok(self
            .paths(path)?
            .into_iter()
            .filter_map(|p| {
                p.strip_prefix(&prefix)
                    .filter(|p| !p.contains('/'))
                    .map(str::to_owned)
            })
            .collect())
    }
}
