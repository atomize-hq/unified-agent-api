//! The lifecycle state's depth fields: the `support_depth` object of decision OF-P2 and the
//! `written_paths_sha256` of OF-P9 check (i), in
//! `docs/agents/lifecycle/support-depth-onboarding-freeze-decisions.md`.
//!
//! Only the onboarding depth freeze may write or replace the object, and only a passing
//! depth-enrolled write run may record the digest. Neither is installed. Each field is read as a
//! [`Preserved`] value, which a writer writes back in the exact bytes it was read in, so every
//! lifecycle writer preserves both byte for byte whatever their formatting; the lifecycle writer
//! also refuses any write that would change those bytes.

use std::{fmt, ops::Deref};

use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::value::RawValue;

use super::{validation::validate_sha256, LifecycleError, LifecycleStage, LifecycleState};

const OBJECT: &str = "support_depth";
const WRITTEN_PATHS: &str = "written_paths_sha256";

/// A value together with the exact JSON bytes it was read in. It is written back in those bytes,
/// and two are equal only when their bytes are.
#[derive(Clone)]
pub struct Preserved<T> {
    value: T,
    raw: Box<RawValue>,
}

impl<T: Serialize> Preserved<T> {
    /// A new value, in the form a lifecycle writer writes a root member: pretty JSON indented
    /// for that depth.
    pub fn new(value: T) -> Result<Self, serde_json::Error> {
        let pretty = serde_json::to_string_pretty(&value)?.replace('\n', "\n  ");
        Ok(Self {
            raw: RawValue::from_string(pretty)?,
            value,
        })
    }
}

impl<T> Preserved<T> {
    /// The exact bytes the value is held in.
    pub fn raw(&self) -> &str {
        self.raw.get()
    }
}

impl<T> Deref for Preserved<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T> PartialEq for Preserved<T> {
    fn eq(&self, other: &Self) -> bool {
        self.raw.get() == other.raw.get()
    }
}

impl<T> Eq for Preserved<T> {}

impl<T: fmt::Debug> fmt::Debug for Preserved<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl<'de, T: DeserializeOwned> Deserialize<'de> for Preserved<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = Box::<RawValue>::deserialize(deserializer)?;
        // A real parse of the held bytes: escapes are decoded, and a repeated or unknown member
        // of a typed object is refused.
        let value = serde_json::from_str(raw.get()).map_err(serde::de::Error::custom)?;
        Ok(Self { value, raw })
    }
}

impl<T> Serialize for Preserved<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.raw.serialize(serializer)
    }
}

pub(super) fn validate_state_fields(state: &LifecycleState) -> Result<(), LifecycleError> {
    if let Some(object) = &state.support_depth {
        object.validate().map_err(LifecycleError::Validation)?;
    }
    if let Some(digest) = &state.written_paths_sha256 {
        validate_sha256(WRITTEN_PATHS, digest)?;
        if state.support_depth.is_none() {
            return Err(LifecycleError::Validation(format!(
                "{WRITTEN_PATHS} is recorded only for a depth-enrolled generation, whose lifecycle \
                 state states `{OBJECT}`"
            )));
        }
        if state.lifecycle_stage < LifecycleStage::RuntimeIntegrated {
            return Err(LifecycleError::Validation(format!(
                "{WRITTEN_PATHS} is recorded by a passing write run, so it is valid only at \
                 `runtime_integrated` and later stages"
            )));
        }
    }
    Ok(())
}

/// The root depth members of a lifecycle state, by their exact bytes, read by a real JSON parser:
/// member names are decoded, a repeated depth member is refused, and other members are ignored.
#[derive(Deserialize)]
struct DepthBytes {
    #[serde(default)]
    support_depth: Option<Box<RawValue>>,
    #[serde(default)]
    written_paths_sha256: Option<Box<RawValue>>,
}

impl DepthBytes {
    fn read(bytes: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(bytes).map_err(|err| err.to_string())
    }

    fn fields(&self) -> [(&'static str, Option<&str>); 2] {
        [
            (OBJECT, self.support_depth.as_deref().map(RawValue::get)),
            (
                WRITTEN_PATHS,
                self.written_paths_sha256.as_deref().map(RawValue::get),
            ),
        ]
    }
}

/// Refuses a lifecycle write that adds, changes or removes a depth field's bytes, compared with
/// the state already at the destination. `previous` is the destination's bytes, `None` when there
/// is no file; a destination whose depth members cannot be read is refused, since what it states
/// cannot then be preserved.
pub(super) fn require_preserved(
    previous: Option<&[u8]>,
    next: &LifecycleState,
) -> Result<(), LifecycleError> {
    let refuse = |why: String| LifecycleError::Validation(format!("lifecycle writer: {why}"));
    let next = serde_json::to_vec_pretty(next)
        .map_err(|err| LifecycleError::Internal(format!("serialize lifecycle state: {err}")))?;
    let next = DepthBytes::read(&next).map_err(LifecycleError::Internal)?;
    let previous = previous.map(DepthBytes::read).transpose().map_err(|err| {
        refuse(format!(
            "the lifecycle state at the destination cannot be read, so its depth fields \
                 cannot be preserved: {err}"
        ))
    })?;
    let before = previous.as_ref().map(DepthBytes::fields);
    for (index, (field, after)) in next.fields().into_iter().enumerate() {
        let before = before.as_ref().and_then(|fields| fields[index].1);
        if before != after {
            return Err(refuse(format!(
                "`{field}` must be preserved byte for byte; only the onboarding depth freeze \
                 writes the depth object and only a depth-enrolled write run records the \
                 written-paths digest, and neither is installed"
            )));
        }
    }
    Ok(())
}
