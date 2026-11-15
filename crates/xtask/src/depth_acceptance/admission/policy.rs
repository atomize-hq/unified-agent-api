//! Runtime Git identities of the independently installed protected IN1 policy source.
//! A payload's closure or implementation digest never selects the evaluator's policy anchor.
use super::{objects::Objects, *};
use std::{collections::BTreeMap, path::Path};

const REQUIRED: [&str; 7] = [
    "crates",
    "Cargo.toml",
    "Cargo.lock",
    ".github",
    ".github/workflows/ci.yml",
    "scripts",
    "scripts/depth_ci_archive.py",
];
const OPTIONAL: [&str; 4] = [
    ".cargo/config",
    ".cargo/config.toml",
    "rust-toolchain",
    "rust-toolchain.toml",
];

pub(crate) fn closure<'a>(
    root: &Path,
    source: &Object,
    extra: impl IntoIterator<Item = &'a str>,
) -> Result<BTreeMap<String, String>, String> {
    let mut objects = Objects::new(root);
    objects.require(source)?;
    for path in OPTIONAL {
        if objects.identity(source, path)?.is_some() {
            return Err("IN1 policy requires absent root Cargo/toolchain configuration".into());
        }
    }
    REQUIRED
        .into_iter()
        .chain(extra)
        .map(|path| {
            let id = objects
                .identity(source, path)?
                .ok_or_else(|| format!("IN1 missing approved policy input: {path}"))?;
            Ok((path.to_owned(), id))
        })
        .collect()
}

pub(crate) fn identity(closure: &BTreeMap<String, String>) -> Result<String, String> {
    Ok(digest(
        b"support-depth-in1-git-policy-v1\0",
        &canonical(closure)?,
    ))
}

/// Like producer approval, compare each source tree/blob with independently protected objects.
/// The source-installed authority supplies `approved`; neither candidate nor retained JSON can.
pub(crate) fn authenticate(root: &Path, claimed: &Policy, approved: &Policy) -> Result<(), String> {
    authenticate_inputs(root, claimed, approved).map_err(|failure| match failure {
        AuthenticationFailure::SourceRead(error) | AuthenticationFailure::Mismatch(error) => error,
    })
}

#[derive(Debug)]
pub(crate) enum AuthenticationFailure {
    SourceRead(String),
    Mismatch(String),
}
impl From<String> for AuthenticationFailure {
    fn from(error: String) -> Self {
        Self::SourceRead(error)
    }
}

pub(crate) fn authenticate_inputs(
    root: &Path,
    claimed: &Policy,
    approved: &Policy,
) -> Result<(), AuthenticationFailure> {
    let mut objects = Objects::new(root);
    let source = objects.commit(&claimed.workflow_commit)?.object;
    let anchor = objects.commit(&approved.workflow_commit)?.object;
    let expected = closure(root, &anchor, approved.closure.keys().map(String::as_str))?;
    if approved.closure != expected
        || claimed.closure != expected
        || claimed.implementation != identity(&expected)?
        || approved.implementation != claimed.implementation
    {
        return Err(AuthenticationFailure::Mismatch(
            "IN1 policy identity differs from protected anchor".into(),
        ));
    }
    let evaluated = if source == anchor {
        expected.clone()
    } else {
        closure(root, &source, expected.keys().map(String::as_str))?
    };
    for (path, id) in expected {
        if evaluated.get(&path) != Some(&id) {
            return Err(AuthenticationFailure::Mismatch(format!(
                "IN1 policy closure is not approved on protected staging: {path}"
            )));
        }
    }
    Ok(())
}
