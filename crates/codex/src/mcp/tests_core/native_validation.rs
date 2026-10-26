use super::super::interaction_validation::valid;
use serde_json::json;

#[test]
fn pinned_complete_decisions_and_opaque_content_are_preserved() {
    let command = "item/commandExecution/requestApproval";
    for reply in [
        json!({"decision":"accept"}),
        json!({"decision":"acceptForSession"}),
        json!({"decision":"decline"}),
        json!({"decision":"cancel"}),
        json!({"decision":{"acceptWithExecpolicyAmendment":{"execpolicy_amendment":["git","status"]}}}),
        json!({"decision":{"applyNetworkPolicyAmendment":{"network_policy_amendment":{"action":"deny","host":"example.test"}}}}),
    ] {
        assert!(valid(command, &reply, true), "{reply}");
    }
    assert!(!valid(command, &json!({"decision":true}), true));
    assert!(!valid(
        command,
        &json!({"decision":{"acceptWithExecpolicyAmendment":{"execpolicy_amendment":[true]}}}),
        true
    ));
    assert!(!valid(
        command,
        &json!({"decision":{"acceptWithExecpolicyAmendment":{"execpolicy_amendment":[]},"unknown":true}}),
        true
    ));
    // Native additionalProperties defaults to true; do not silently strengthen it.
    assert!(valid(
        command,
        &json!({"decision":"accept","futureMetadata":false}),
        true
    ));
    let permissions = "item/permissions/requestApproval";
    assert!(valid(
        permissions,
        &json!({"permissions":{"fileSystem":{"read":["/a"],"write":[],"entries":[{"access":"deny","path":{"type":"glob_pattern","pattern":"**/secret"}}]},"network":{"enabled":false}},"scope":"session","strictAutoReview":false}),
        true
    ));
    assert!(valid(permissions, &json!({"permissions":{}}), true));
    assert!(!valid(
        permissions,
        &json!({"permissions":{"network":{"enabled":"false"}}}),
        true
    ));
    assert!(!valid(
        permissions,
        &json!({"permissions":{},"scope":"forever"}),
        true
    ));
    assert!(valid(
        "item/tool/requestUserInput",
        &json!({"answers":{"q":{"answers":["Unicode λ","line\n"]}}}),
        true
    ));
    assert!(!valid(
        "item/tool/requestUserInput",
        &json!({"answers":{"q":{"answers":[false]}}}),
        true
    ));
    assert!(valid(
        "mcpServer/elicitation/request",
        &json!({"action":"accept","content":{"nested":[false,null,42]},"_meta":{"custom":"value"}}),
        true
    ));
    assert!(!valid(
        "mcpServer/elicitation/request",
        &json!({"action":"approve"}),
        true
    ));
}

#[test]
fn standalone_mcp_absent_and_null_turn_and_required_fields_follow_pinned_contract() {
    let method = "mcpServer/elicitation/request";
    let mut params = json!({"threadId":"t","serverName":"s","mode":"url","message":"login","url":"https://example.test","elicitationId":"e"});
    assert!(valid(method, &params, false));
    params["turnId"] = json!(null);
    assert!(valid(method, &params, false));
    params["turnId"] = json!("turn");
    assert!(valid(method, &params, false));
    params["turnId"] = json!(42);
    assert!(!valid(method, &params, false));
    assert!(!valid(
        "item/fileChange/requestApproval",
        &json!({"threadId":"t","turnId":"u","itemId":"i"}),
        false
    ));
    assert!(!valid("future/request", &json!({}), false));
}
