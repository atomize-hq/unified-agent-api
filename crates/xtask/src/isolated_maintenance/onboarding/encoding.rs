//! Full writer encoding prevents duplicate JSON keys or unvalidated trailing bytes.
use super::*;

pub(super) fn check<T: serde::de::DeserializeOwned + serde::Serialize>(
    source: &Source,
    path: &str,
) -> Result<(), String> {
    let Some(bytes) = source.optional_bytes(path)? else {
        return Ok(());
    };
    let value: T = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let expected = format!(
        "{}\n",
        serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?
    );
    if bytes != expected.as_bytes() {
        return Err(format!(
            "bootstrap artifact differs from complete writer encoding: {path}"
        ));
    }
    Ok(())
}
