mod support;
use justblast_desktop::{
    api::{NativeState, models::*},
    device::profile::Profile,
    preferences::Store,
};
use std::sync::{Arc, Mutex};
fn request(
    state: &NativeState,
    action: CalibrationAction,
) -> Result<CalibrationView, CommandError> {
    let revision = state.snapshot().unwrap().device.revision;
    state.calibration_request(CalibrationRequest {
        command_id: format!("calibration-{revision}"),
        expected_revision: revision,
        action,
    })
}
#[test]
fn old_profiles_import_hardware_only_and_export_round_trips() {
    let raw = format!(
        "{}\n[weights]\npoints = 999\ncombo = 0\nflexibility = 0\nfragmentation = 0\n[combo_seed]\ncount = 123\n[rule_context]\nall_clear_bonus = 999\n",
        Profile::builtin().export().unwrap()
    );
    let profile = Profile::parse(&raw).unwrap();
    let output = profile.export().unwrap();
    assert!(!output.contains("weights"));
    assert!(!output.contains("combo"));
    assert!(!output.contains("rule_context"));
    assert_eq!(
        Profile::parse(&output).unwrap().screen.board,
        profile.screen.board
    );
    assert!(Profile::parse(&(output + "\n[unknown]\nx=1\n")).is_err());
}
#[test]
fn capture_and_preview_do_not_move_and_test_drag_is_one_shot() {
    let phone = Arc::new(Mutex::new(support::Phone::default()));
    let state =
        NativeState::with_transport(Store::default(), Box::new(support::Fake(phone.clone())))
            .unwrap();
    let capture = request(
        &state,
        CalibrationAction::Capture {
            serial: "offline-phone".to_owned(),
        },
    )
    .unwrap();
    assert_eq!(capture.profile.screen.width, 1080);
    assert!(capture.image.unwrap().starts_with("data:image/png;base64,"));
    assert_eq!(phone.lock().unwrap().calls, 0);
    let id = capture.capture_id.unwrap();
    let profile = capture.profile;
    let preview = request(
        &state,
        CalibrationAction::Preview {
            capture_id: id,
            profile: profile.clone(),
        },
    )
    .unwrap();
    assert!(!preview.board_unclear);
    assert!(preview.unknown_slots.is_empty());
    assert_eq!(phone.lock().unwrap().calls, 0);
    request(
        &state,
        CalibrationAction::TestDrag {
            capture_id: id,
            profile: profile.clone(),
            slot: 0,
            row: 1,
            column: 0,
            install_helper: false,
        },
    )
    .unwrap();
    assert_eq!(phone.lock().unwrap().calls, 1);
    assert_eq!(phone.lock().unwrap().batches, vec![1]);
    assert!(
        request(
            &state,
            CalibrationAction::TestDrag {
                capture_id: id,
                profile,
                slot: 0,
                row: 1,
                column: 0,
                install_helper: false
            }
        )
        .is_err()
    );
    assert_eq!(phone.lock().unwrap().calls, 1);
    let snapshot = state.snapshot().unwrap();
    assert_eq!(snapshot.completed, 0);
    assert!(!snapshot.input_known);
    assert!(!snapshot.device.busy);
}
#[test]
fn stale_capture_and_changed_game_cannot_move() {
    let phone = Arc::new(Mutex::new(support::Phone::default()));
    let state =
        NativeState::with_transport(Store::default(), Box::new(support::Fake(phone.clone())))
            .unwrap();
    let capture = request(
        &state,
        CalibrationAction::Capture {
            serial: "offline-phone".to_owned(),
        },
    )
    .unwrap();
    phone.lock().unwrap().slots[0] = Some(support::custom_ring());
    assert!(
        request(
            &state,
            CalibrationAction::TestDrag {
                capture_id: capture.capture_id.unwrap(),
                profile: capture.profile,
                slot: 0,
                row: 1,
                column: 0,
                install_helper: false
            }
        )
        .is_err()
    );
    assert_eq!(phone.lock().unwrap().calls, 0);
    assert!(!state.snapshot().unwrap().device.busy);
}
#[test]
fn importing_or_selecting_a_profile_preserves_app_preferences_and_weights() {
    let root = std::env::temp_dir().join(format!("justblast-hardware-only-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let state = NativeState::with_store(Store::at(root.clone()).unwrap()).unwrap();
    let mut prefs = state.preferences().unwrap();
    prefs.weights.points = 99.0;
    prefs.automation.continuous = true;
    state.save_preferences(prefs).unwrap();
    let legacy = format!(
        "{}\n[weights]\npoints=123\ncombo=1\nflexibility=1\nfragmentation=1\n",
        Profile::builtin().export().unwrap()
    );
    for (index, action) in [
        DeviceAction::Profile {
            id: "my-phone".to_owned(),
            toml: legacy,
        },
        DeviceAction::SelectProfile {
            id: "default".to_owned(),
        },
    ]
    .into_iter()
    .enumerate()
    {
        let revision = state.snapshot().unwrap().device.revision;
        state
            .device_request(DeviceRequest {
                command_id: format!("profile-{index}"),
                expected_revision: revision,
                action,
            })
            .unwrap();
        assert_eq!(state.preferences().unwrap().weights.points, 99.0);
        assert!(state.preferences().unwrap().automation.continuous);
        assert_eq!(state.snapshot().unwrap().weights.points, 99.0);
    }
    assert!(
        !std::fs::read_to_string(root.join("profiles/my-phone.toml"))
            .unwrap()
            .contains("weights")
    );
    drop(state);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn calibration_takes_over_a_connected_workspace_without_user_disconnect() {
    let phone = Arc::new(Mutex::new(support::Phone::default()));
    let state =
        NativeState::with_transport(Store::default(), Box::new(support::Fake(phone.clone())))
            .unwrap();
    let revision = state.snapshot().unwrap().device.revision;
    state
        .device_request(DeviceRequest {
            command_id: "connect-calibration".to_owned(),
            expected_revision: revision,
            action: DeviceAction::Connect {
                serial: "offline-phone".to_owned(),
                install_helper: false,
            },
        })
        .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while state.snapshot().unwrap().device.busy {
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let capture = request(
        &state,
        CalibrationAction::Capture {
            serial: "offline-phone".to_owned(),
        },
    )
    .unwrap();
    assert!(capture.capture_id.is_some());
    assert!(state.snapshot().unwrap().device.serial.is_none());
    assert_eq!(phone.lock().unwrap().calls, 0);
}

#[test]
fn measured_wrong_landing_adjusts_touch_and_requires_another_verified_test() {
    let phone = Arc::new(Mutex::new(support::Phone::default()));
    let state =
        NativeState::with_transport(Store::default(), Box::new(support::Fake(phone.clone())))
            .unwrap();
    let capture = request(
        &state,
        CalibrationAction::Capture {
            serial: "offline-phone".to_owned(),
        },
    )
    .unwrap();
    let mut profile = capture.profile;
    profile.touch.lift = 360.0;
    let result = request(
        &state,
        CalibrationAction::TestDrag {
            capture_id: capture.capture_id.unwrap(),
            profile,
            slot: 0,
            row: 4,
            column: 5,
            install_helper: false,
        },
    )
    .unwrap();
    assert_eq!(result.movement.as_deref(), Some("adjusted"));
    assert!(
        (result.profile.touch.lift - 269.0).abs() < 40.0,
        "{}",
        result.profile.touch.lift
    );
    let verified = request(
        &state,
        CalibrationAction::TestDrag {
            capture_id: result.capture_id.unwrap(),
            profile: result.profile,
            slot: 1,
            row: 2,
            column: 0,
            install_helper: false,
        },
    )
    .unwrap();
    assert_eq!(verified.movement.as_deref(), Some("verified"));
    assert_eq!(phone.lock().unwrap().calls, 2);
}

#[test]
fn unexplained_landing_stops_and_preserves_recovery_instead_of_retrying() {
    let phone = Arc::new(Mutex::new(support::Phone {
        mismatch_after_send: true,
        ..Default::default()
    }));
    let state =
        NativeState::with_transport(Store::default(), Box::new(support::Fake(phone.clone())))
            .unwrap();
    let capture = request(
        &state,
        CalibrationAction::Capture {
            serial: "offline-phone".to_owned(),
        },
    )
    .unwrap();
    assert!(
        request(
            &state,
            CalibrationAction::TestDrag {
                capture_id: capture.capture_id.unwrap(),
                profile: capture.profile,
                slot: 0,
                row: 2,
                column: 0,
                install_helper: false
            }
        )
        .is_err()
    );
    let capture = request(
        &state,
        CalibrationAction::Capture {
            serial: "offline-phone".to_owned(),
        },
    )
    .unwrap();
    assert!(
        request(
            &state,
            CalibrationAction::TestDrag {
                capture_id: capture.capture_id.unwrap(),
                profile: capture.profile,
                slot: 1,
                row: 3,
                column: 0,
                install_helper: false
            }
        )
        .is_err()
    );
    assert_eq!(phone.lock().unwrap().calls, 1);
}
