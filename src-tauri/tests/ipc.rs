use justblast_desktop::api::NativeState;
use justblast_desktop::api::models::{ErrorCode, ReplaceBoardRequest};

#[test]
fn wire_contract_uses_camel_case_and_rows_instead_of_u64() {
    let state = NativeState::default();
    let view = serde_json::to_value(state.snapshot().unwrap()).unwrap();
    assert_eq!(view["schemaVersion"], 2);
    assert!(view.get("mode").is_none());
    assert_eq!(view["completed"], 0);
    assert_eq!(view["board"], serde_json::json!(vec!["........"; 8]));
    assert_eq!(view["capabilities"]["solver"], true);
    assert_eq!(view["capabilities"]["device"], true);
    assert!(view.get("bits").is_none());
}

#[test]
fn malformed_input_and_stale_revisions_leave_native_state_unchanged() {
    let state = NativeState::default();
    let mut rows = std::array::from_fn(|_| "........".to_owned());
    rows[7] = ".......x".to_owned();
    let error = state
        .replace_board(ReplaceBoardRequest {
            expected_revision: 1,
            rows,
        })
        .unwrap_err();
    assert!(matches!(error.code, ErrorCode::InvalidInput));
    assert_eq!(state.snapshot().unwrap().revision, 1);
    assert_eq!(state.snapshot().unwrap().occupied, 0);
    let error = state
        .replace_board(ReplaceBoardRequest {
            expected_revision: 0,
            rows: std::array::from_fn(|_| "........".to_owned()),
        })
        .unwrap_err();
    assert!(matches!(error.code, ErrorCode::StaleRevision));
}
#[test]
fn retired_mode_commands_and_unknown_fields_are_rejected() {
    assert!(
        serde_json::from_value::<ReplaceBoardRequest>(
            serde_json::json!({"expectedRevision":1,"rows":vec!["........";8],"mode":"manual"})
        )
        .is_err()
    );
}

#[test]
fn action_contracts_reject_unknown_fields_and_invalid_narrow_values() {
    use justblast_desktop::api::models::{DeviceRequest, WorkspaceAction};
    assert!(serde_json::from_value::<WorkspaceAction>(serde_json::json!({"type":"search","expectedRevision":1,"maxNodes":1,"shell":"anything"})).is_err());
    assert!(serde_json::from_value::<DeviceRequest>(serde_json::json!({"commandId":"bad","expectedRevision":1,"action":{"type":"discover","shell":"anything"}})).is_err());
    assert!(serde_json::from_value::<DeviceRequest>(serde_json::json!({"commandId":"bad","expectedRevision":1,"action":{"type":"start","workspaceRevision":1,"through":256}})).is_err());
}
