//! The create lane's second statement of P and O: the `support_depth` object of an onboarding
//! pack's `governance/lifecycle-state.json` (decision OF-P2 of
//! `docs/agents/lifecycle/support-depth-onboarding-freeze-decisions.md`).
//!
//! The object has the shape of a maintenance request's `[support_depth]` table, and it is compared
//! with the record by the same `DepthRecord::request_mismatch`, with the onboarding Event in place
//! of the request's. Nothing writes the object yet: no onboarding freeze is installed, and every
//! lifecycle writer preserves it byte for byte.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use super::{sha256, DepthRecord, Event, RequestBinding};

/// The lifecycle state's `support_depth` object. It holds exactly `policy_identity` and, once O is
/// frozen, `obligations_identity`, and nothing else.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LifecycleDepth {
    pub policy_identity: String,
    /// Absent while O is not yet frozen. It is left out, never `null`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub obligations_identity: Option<String>,
}

impl LifecycleDepth {
    pub fn validate(&self) -> Result<(), String> {
        sha256("support_depth.policy_identity", &self.policy_identity)?;
        if let Some(identity) = &self.obligations_identity {
            sha256("support_depth.obligations_identity", identity)?;
        }
        Ok(())
    }

    /// What the object froze, stated as a request states it, for `request_mismatch`.
    pub fn binding(&self, event: Event) -> RequestBinding {
        RequestBinding {
            event,
            policy_identity: Some(self.policy_identity.clone()),
            obligations_identity: self.obligations_identity.clone(),
        }
    }
}

/// What an onboarding pack's approval artifact states of its generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnboardingApproval {
    /// The onboarding Event: the approval's `approval_commit` and `approval_recorded_at`.
    pub event: Event,
    /// The version its `[support_depth]` table approves a depth enrollment for, if any.
    pub support_depth_version: Option<String>,
}

#[derive(Deserialize)]
struct ApprovalStamp {
    approval_commit: String,
    approval_recorded_at: String,
}

impl OnboardingApproval {
    /// Reads the approval artifact at `relative_path` for what the cross-check compares. Its
    /// `[support_depth]` table is read by the approval loader's own rules.
    pub fn parse(text: &str, relative_path: &str) -> Result<Self, String> {
        let scope = crate::approval_artifact::parse_approval_attribution(text, relative_path)
            .map_err(|err| err.to_string())?;
        let stamp: ApprovalStamp = toml_edit::de::from_str(text)
            .map_err(|err| format!("approval artifact `{relative_path}`: {err}"))?;
        Ok(Self {
            event: Event::Onboarding {
                approval_commit: stamp.approval_commit,
                approval_recorded_at: stamp.approval_recorded_at,
            },
            support_depth_version: scope.support_depth_version,
        })
    }
}

/// How an onboarding generation stands, once its record and lifecycle object agree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnboardingDepth {
    /// No record and no object: P is not yet frozen. Insufficient depth, and not an error.
    NotFrozen,
    /// The record and the object state this approval's generation.
    Frozen,
    /// The record and the object state an earlier approval's generation, and this one has not
    /// frozen yet. Insufficient depth, and not an error.
    EarlierGeneration,
}

/// One way the record and the object fail to agree. `version` names the record's version, or is
/// `None` when the fault is the object's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingFault {
    pub code: &'static str,
    pub version: Option<String>,
    pub message: String,
}

/// Compares the lifecycle object with the onboarding records of one manifest root, `records` by
/// version. Records whose Event is a maintenance Event are not the object's and are ignored here.
/// A missing object is never read as "not depth-enrolled".
pub fn onboarding_binding(
    object: Option<&LifecycleDepth>,
    approval: &OnboardingApproval,
    records: &BTreeMap<String, DepthRecord>,
) -> Result<OnboardingDepth, Vec<BindingFault>> {
    let version = approval.support_depth_version.as_deref();
    let onboarding = records
        .iter()
        .filter(|(_, record)| matches!(record.event, Event::Onboarding { .. }));
    let mut faults = Vec::new();
    let mut stated = None;
    for (held, record) in onboarding {
        if object.is_some() && Some(held.as_str()) == version {
            stated = Some(record);
            continue;
        }
        let message = match (object, version) {
            (None, _) => "the record states an onboarding generation, but the lifecycle state has \
                          no `support_depth` object: a deleted binding"
                .to_string(),
            (Some(_), Some(version)) => format!(
                "the record states an onboarding generation, but the lifecycle state's \
                 `support_depth` object states version `{version}`, the approval's"
            ),
            (Some(_), None) => "the record states an onboarding generation, but the approval \
                                approves no depth enrollment"
                .to_string(),
        };
        faults.push(fault(
            "DEPTH_LIFECYCLE_BINDING_MISSING",
            Some(held.as_str()),
            message,
        ));
    }

    let state = match (object, stated) {
        (None, _) => Some(OnboardingDepth::NotFrozen),
        (Some(_), None) => {
            let message = match version {
                Some(version) => format!(
                    "the lifecycle state's `support_depth` object states P, but version \
                     `{version}`, the approval's, holds no onboarding depth record: a deleted \
                     binding"
                ),
                None => "the lifecycle state's `support_depth` object states P, but the approval \
                         approves no depth enrollment"
                    .to_string(),
            };
            faults.push(fault("DEPTH_LIFECYCLE_BINDING_ORPHANED", None, message));
            None
        }
        (Some(object), Some(record)) => compare(object, approval, record, version, &mut faults),
    };
    match state {
        Some(state) if faults.is_empty() => Ok(state),
        _ => Err(faults),
    }
}

fn compare(
    object: &LifecycleDepth,
    approval: &OnboardingApproval,
    record: &DepthRecord,
    version: Option<&str>,
    faults: &mut Vec<BindingFault>,
) -> Option<OnboardingDepth> {
    let (state, event) = if record.event == approval.event {
        (OnboardingDepth::Frozen, approval.event.clone())
    } else if earlier(&record.event, &approval.event) {
        (OnboardingDepth::EarlierGeneration, record.event.clone())
    } else {
        let message = "the record's Event is neither the approval's nor an earlier approval's";
        faults.push(fault(
            "DEPTH_RECORD_NOT_THE_LIFECYCLES",
            version,
            message.to_string(),
        ));
        return None;
    };
    if let Some(why) = record.request_mismatch(&object.binding(event)) {
        let message = format!("{why}; the lifecycle state's `support_depth` object froze it");
        faults.push(fault("DEPTH_RECORD_NOT_THE_LIFECYCLES", version, message));
        return None;
    }
    Some(state)
}

/// Whether `then` names an approval recorded strictly before `now`'s. A time that does not read
/// as RFC 3339 is never earlier.
fn earlier(then: &Event, now: &Event) -> bool {
    let recorded = |event: &Event| match event {
        Event::Onboarding {
            approval_recorded_at,
            ..
        } => OffsetDateTime::parse(approval_recorded_at, &Rfc3339).ok(),
        Event::Maintenance { .. } => None,
    };
    matches!((recorded(then), recorded(now)), (Some(then), Some(now)) if then < now)
}

fn fault(code: &'static str, version: Option<&str>, message: String) -> BindingFault {
    BindingFault {
        code,
        version: version.map(str::to_string),
        message,
    }
}
