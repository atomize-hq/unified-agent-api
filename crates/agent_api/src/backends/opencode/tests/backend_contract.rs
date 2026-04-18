use crate::{backend_harness::BackendHarnessAdapter, AgentWrapperBackend};

use super::support::backend_with_env;

#[test]
fn opencode_adapter_implements_backend_harness_adapter_contract() {
    fn assert_impl<T: BackendHarnessAdapter>() {}
    assert_impl::<crate::backends::opencode::OpencodeBackend>();
}

#[test]
fn opencode_backend_keeps_capabilities_and_extensions_conservative() {
    let backend = backend_with_env(Default::default());

    assert_eq!(
        backend.capabilities().ids,
        std::collections::BTreeSet::from([
            "agent_api.run".to_string(),
            "agent_api.events".to_string(),
            "agent_api.events.live".to_string(),
        ])
    );
    assert!(backend.supported_extension_keys().is_empty());
}
