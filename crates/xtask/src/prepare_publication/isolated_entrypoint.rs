//! Test-only entry into the same body, after authenticating the installed isolated route.
//! Instantiated in each entrypoint module to use that module's distinct Args and Error types.
#![allow(dead_code)]
use super::*;

pub(crate) fn run<W: Write>(
    workspace_root: &Path,
    args: Args,
    writer: &mut W,
) -> Result<(), Error> {
    xtask::isolated_maintenance::route(workspace_root)
        .map_err(Error::Validation)?
        .ok_or_else(|| Error::Validation("isolated route required".into()))?;
    run_body(workspace_root, args, writer)
}
