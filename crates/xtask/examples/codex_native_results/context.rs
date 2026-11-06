//! Exclusive finite subscriber in the standalone collector; no raw tracing output.
use std::{
    collections::BTreeMap,
    ffi::OsString,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use tracing::{
    field::{Field, Visit},
    span::{Attributes, Record},
    Event, Id, Metadata, Subscriber,
};
use xtask::depth_evidence::{
    native::context::{
        self, guard::LocalGuard, CaseContext, Config, Launch, Route, ThreadSettings,
    },
    CaseIdentity,
};
#[derive(Default)]
struct State {
    current: Option<CaseContext>,
    requirements_seen: bool,
    owner: Option<u64>,
}
#[derive(Clone, Default)]
pub struct Capture(Arc<Mutex<State>>);
#[derive(Default)]
struct Fields {
    kind: Option<String>,
    projection: Option<String>,
    valid: bool,
    owner: Option<u64>,
}
impl Visit for Fields {
    fn record_str(&mut self, field: &Field, value: &str) {
        if value.len() > 4096 {
            self.valid = false;
            return;
        }
        match field.name() {
            "kind" => self.kind = Some(value.into()),
            "projection" => self.projection = Some(value.into()),
            _ => self.valid = false,
        }
    }
    fn record_u64(&mut self, field: &Field, value: u64) {
        if field.name() == "owner" && value > 0 && self.owner.is_none() {
            self.owner = Some(value);
        } else {
            self.valid = false;
        }
    }
    fn record_debug(&mut self, _: &Field, _: &dyn std::fmt::Debug) {
        self.valid = false;
    }
}
impl Subscriber for Capture {
    fn enabled(&self, m: &Metadata<'_>) -> bool {
        m.target() == context::TRACE_TARGET
    }
    fn new_span(&self, _: &Attributes<'_>) -> Id {
        Id::from_u64(1)
    }
    fn record(&self, _: &Id, _: &Record<'_>) {}
    fn record_follows_from(&self, _: &Id, _: &Id) {}
    fn enter(&self, _: &Id) {}
    fn exit(&self, _: &Id) {}
    fn event(&self, event: &Event<'_>) {
        if !self.enabled(event.metadata()) {
            return;
        }
        let mut fields = Fields {
            valid: true,
            ..Default::default()
        };
        event.record(&mut fields);
        let Ok(mut state) = self.0.lock() else { return };
        let owner_matches = match (state.owner, fields.owner) {
            (None, Some(owner)) if fields.kind.as_deref() == Some("launch") => {
                state.owner = Some(owner);
                true
            }
            (Some(current), Some(owner)) => current == owner,
            _ => false,
        };
        let duplicate_requirements = fields.kind.as_deref() == Some("requirements")
            && std::mem::replace(&mut state.requirements_seen, true);
        let Some(c) = state.current.as_mut() else {
            return;
        };
        let raw = fields.projection.as_deref().unwrap_or("");
        if duplicate_requirements || !owner_matches {
            c.diagnostics_valid = false;
            return;
        }
        match fields.kind.as_deref() {
            Some("launch") if fields.valid && c.launches.len() < 2 => {
                match serde_json::from_str::<Launch>(raw) {
                    Ok(l) => c.launches.push(l),
                    Err(_) => c.diagnostics_valid = false,
                }
            }
            Some("config") if fields.valid && c.config.is_none() => {
                match serde_json::from_str::<Config>(raw) {
                    Ok(p) => c.config = Some(p),
                    Err(_) => c.diagnostics_valid = false,
                }
            }
            Some("thread")
                if fields.valid
                    && c.route == Route::AppServer
                    && c.launches.len() == 2
                    && c.thread_settings.is_none()
                    && c.turn_accepted.is_none() =>
            {
                match serde_json::from_str::<ThreadSettings>(raw) {
                    Ok(p) => c.thread_settings = Some(p),
                    Err(_) => c.diagnostics_valid = false,
                }
            }
            Some("turn")
                if fields.valid && c.thread_settings.is_some() && c.turn_accepted.is_none() =>
            {
                match serde_json::from_str::<bool>(raw) {
                    Ok(accepted) => c.turn_accepted = Some(accepted),
                    Err(_) => c.diagnostics_valid = false,
                }
            }
            Some("requirements") if fields.valid => match serde_json::from_str::<bool>(raw) {
                Ok(empty) if c.config.is_some() => {
                    c.config.as_mut().unwrap().requirements_empty = empty
                }
                _ => c.diagnostics_valid = false,
            },
            _ => c.diagnostics_valid = false,
        }
    }
}
impl Capture {
    pub fn install() -> Result<Self, String> {
        let capture = Self::default();
        tracing::subscriber::set_global_default(capture.clone())
            .map_err(|_| "native context subscriber unavailable")?;
        Ok(capture)
    }
    pub fn begin(&self, case: CaseIdentity, route: Route, before: bool) {
        let mut state = self.0.lock().unwrap();
        state.requirements_seen = false;
        state.owner = None;
        state.current = Some(CaseContext {
            case,
            route,
            local_before: before,
            local_after: false,
            launches: vec![],
            config: None,
            thread_settings: None,
            turn_accepted: None,
            project_unchanged: None,
            bindings: vec![],
            missing: context::Fact::ALL.to_vec(),
            derived_defaults: vec![],
            diagnostics_valid: true,
        });
    }
    pub fn finish(&self, after: bool) -> CaseContext {
        let mut state = self.0.lock().unwrap();
        let mut c = state.current.take().expect("owned current case");
        c.local_after = after;
        if c.config.is_some() && !state.requirements_seen {
            c.diagnostics_valid = false;
        }
        c.diagnostics_valid &= c.launches.len() == 2
            && !c.launches[0].spawned
            && c.launches[1].spawned
            && c.launches.iter().all(|l| {
                l.route == c.route
                    && l.binary_matches
                    && l.cwd_matches
                    && l.home_matches
                    && l.environment_known
            });
        c
    }
}
pub fn route(interactive: bool, step: &str) -> Route {
    let step = xtask::depth_evidence::native::cases::operation(step);
    if interactive || step.starts_with("fork") {
        Route::AppServer
    } else if step.starts_with("resume") {
        Route::ResumeExec
    } else {
        Route::InitialExec
    }
}
pub fn guard(home: Option<&Path>, project: &Path) -> Result<LocalGuard, String> {
    let home = home
        .map(Path::to_path_buf)
        .or_else(|| std::env::var_os("CODEX_HOME").map(PathBuf::from))
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".codex")))
        .ok_or("native context refused: home")?;
    LocalGuard::inspect(&home, project, &environment()).map_err(|e| e.to_string())
}
pub fn environment() -> BTreeMap<OsString, OsString> {
    std::env::vars_os().collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    fn case() -> CaseIdentity {
        CaseIdentity {
            name: "test".into(),
            executable_sha256: "a".repeat(64),
        }
    }
    fn pair(owner: u64) {
        for spawned in [false, true] {
            let projection = serde_json::to_string(&Launch {
                route: Route::AppServer,
                spawned,
                binary_matches: true,
                cwd_matches: true,
                home_matches: true,
                environment_known: true,
            })
            .unwrap();
            tracing::trace!(target:context::TRACE_TARGET,kind="launch",owner,projection=projection.as_str());
        }
    }
    #[test]
    fn duplicate_missing_and_secret_diagnostics_never_become_valid_context() {
        let capture = Capture::default();
        let dispatch = tracing::Dispatch::new(capture.clone());
        tracing::dispatcher::with_default(&dispatch, || {
            capture.begin(case(), Route::AppServer, true);
            pair(7);
            assert!(capture.finish(true).diagnostics_valid);
            capture.begin(case(), Route::AppServer, true);
            pair(8);
            tracing::trace!(target:context::TRACE_TARGET,kind="config",owner=8u64,projection="{\"unknown\":\"sk-sensitive\"}");
            let c = capture.finish(true);
            assert!(!c.diagnostics_valid);
            assert!(!serde_json::to_string(&c).unwrap().contains("sensitive"));
            capture.begin(case(), Route::InitialExec, true);
            assert!(!capture.finish(true).diagnostics_valid);
            capture.begin(case(), Route::AppServer, true);
            pair(9);
            pair(9);
            assert!(!capture.finish(true).diagnostics_valid);
        });
    }
    #[test]
    fn late_foreign_connection_cannot_supply_current_case_configuration() {
        let capture = Capture::default();
        let dispatch = tracing::Dispatch::new(capture.clone());
        tracing::dispatcher::with_default(&dispatch, || {
            capture.begin(case(), Route::AppServer, true);
            pair(11);
            let projection = serde_json::to_string(&Config {
                model_gpt5: true,
                provider_openai: true,
                approval: Some(context::Approval::Never),
                sandbox: Some(context::Sandbox::WorkspaceWrite),
                origins_present: true,
                layers: None,
                requirements_empty: false,
            })
            .unwrap();
            tracing::trace!(target:context::TRACE_TARGET,kind="config",owner=10u64,projection=projection.as_str());
            assert!(!capture.finish(true).diagnostics_valid);
            capture.begin(case(), Route::AppServer, true);
            pair(12);
            tracing::trace!(target:context::TRACE_TARGET,kind="config",owner=12u64,projection=projection.as_str());
            for _ in 0..2 {
                tracing::trace!(target:context::TRACE_TARGET,kind="requirements",owner=12u64,projection="true");
            }
            assert!(!capture.finish(true).diagnostics_valid);
        });
    }
}

#[cfg(test)]
#[path = "context/process_tests.rs"]
mod process_tests;
