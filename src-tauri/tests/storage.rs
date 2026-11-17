use justblast_core::domain::Board;
use justblast_desktop::{
    api::{NativeState, models::*},
    device::runtime::{Journal, lease},
    preferences::{Preferences, Store},
};
fn directory() -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!(
        "justblast-storage-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).unwrap();
    root
}
#[test]
fn preferences_are_durable_but_startup_never_restores_game_or_phone_execution() {
    let root = directory();
    let store = Store::at(root.clone()).unwrap();
    let state = NativeState::with_store(store.clone()).unwrap();
    let mut prefs = state.preferences().unwrap();
    prefs.automation.move_device = true;
    prefs.automation.continuous = true;
    prefs.shapes.push(SavedShape {
        id: "custom-test".to_owned(),
        name: "Saved corner".to_owned(),
        cells: vec![(0, 0), (1, 0), (1, 1)],
    });
    state.save_preferences(prefs).unwrap();
    let revision = state.snapshot().unwrap().revision;
    state
        .action(WorkspaceAction::Weights {
            expected_revision: revision,
            weights: WeightsView {
                points: 50.0,
                combo: 20.0,
                flexibility: 25.0,
                fragmentation: 5.0,
            },
        })
        .unwrap();
    drop(state);
    let next = NativeState::with_store(store).unwrap();
    assert_eq!(next.preferences().unwrap().shapes.len(), 1);
    assert_eq!(next.preferences().unwrap().weights.points, 50.0);
    let view = next.snapshot().unwrap();
    assert_eq!(view.occupied, 0);
    assert!(view.pieces.is_empty());
    assert!(view.plan.is_none());
    assert!(!view.device.busy);
    assert!(view.device.serial.is_none());
    assert_eq!(view.weights.points, 50.0);
    drop(next);
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn failed_and_invalid_preferences_do_not_change_the_accepted_copy() {
    let root = directory();
    let store = Store::at(root.clone()).unwrap();
    let state = NativeState::with_store(store).unwrap();
    let mut invalid = Preferences::default();
    invalid.shapes.push(SavedShape {
        id: "custom-bad".to_owned(),
        name: "Bad".to_owned(),
        cells: vec![(0, 0), (0, 0)],
    });
    assert!(state.save_preferences(invalid).is_err());
    assert!(state.preferences().unwrap().shapes.is_empty());
    std::fs::create_dir(root.join("preferences.json")).unwrap();
    let mut prefs = Preferences::default();
    prefs.automation.continuous = true;
    assert!(state.save_preferences(prefs).is_err());
    assert!(!state.preferences().unwrap().automation.continuous);
    drop(state);
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn journal_recovery_is_durable_and_a_second_instance_cannot_own_the_phone() {
    let root = directory();
    let store = Store::at(root.clone()).unwrap();
    let mut journal = Journal::at(&store).unwrap();
    journal
        .intent("offline-phone", 3, 1, 0, 1, Board::default())
        .unwrap();
    drop(journal);
    let mut journal = Journal::at(&store).unwrap();
    assert!(journal.uncertain("offline-phone"));
    assert!(
        journal
            .intent("offline-phone", 3, 1, 0, 1, Board::default())
            .is_err()
    );
    journal.resynchronize("offline-phone").unwrap();
    assert!(!Journal::at(&store).unwrap().uncertain("offline-phone"));
    let first = lease(&store, "offline-phone").unwrap();
    assert!(lease(&store, "offline-phone").is_err());
    drop(first);
    assert!(lease(&store, "offline-phone").is_ok());
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn duplicate_application_start_is_rejected_until_the_original_state_closes() {
    let root = directory();
    let state = NativeState::with_store(Store::at(root.clone()).unwrap()).unwrap();
    assert!(NativeState::with_store(Store::at(root.clone()).unwrap()).is_err());
    drop(state);
    let next = NativeState::with_store(Store::at(root.clone()).unwrap()).unwrap();
    drop(next);
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn legacy_modes_migrate_to_one_empty_workspace_without_losing_controls() {
    for mode in ["manual", "auto"] {
        let root = directory();
        let mut old = serde_json::to_value(Preferences::default()).unwrap();
        old["schemaVersion"] = serde_json::json!(1);
        old["mode"] = serde_json::json!(mode);
        old["automation"]["moveDevice"] = serde_json::json!(true);
        old["automation"]["checkAfterCycle"] = serde_json::json!(false);
        let bytes = serde_json::to_vec(&old).unwrap();
        std::fs::write(root.join("preferences.json"), &bytes).unwrap();
        let state = NativeState::with_store(Store::at(root.clone()).unwrap()).unwrap();
        let snapshot = state.snapshot().unwrap();
        assert_eq!(snapshot.occupied, 0);
        assert!(!snapshot.input_known);
        assert!(!snapshot.device.busy);
        assert!(snapshot.device.serial.is_none());
        let prefs = state.preferences().unwrap();
        assert_eq!(prefs.schema_version, 2);
        assert!(prefs.automation.move_device);
        assert!(!prefs.automation.check_after_cycle);
        assert_eq!(
            std::fs::read(root.join("preferences.v1.backup.json")).unwrap(),
            bytes
        );
        let saved: serde_json::Value =
            serde_json::from_slice(&std::fs::read(root.join("preferences.json")).unwrap()).unwrap();
        assert!(saved.get("mode").is_none());
        drop(state);
        std::fs::remove_dir_all(root).unwrap();
    }
}
#[test]
fn renamed_builtin_profile_migrates_preferences_without_losing_user_settings() {
    let root = directory();
    let store = Store::at(root.clone()).unwrap();
    let mut saved = Preferences::default();
    saved.connection.profile_id = "samsung-a73".to_owned();
    saved.connection.serial = "offline-phone".to_owned();
    saved.automation.check_after_move = true;
    saved.weights.points = 51.0;
    store.save_preferences(&saved).unwrap();

    let state = NativeState::with_store(store.clone()).unwrap();
    let preferences = state.preferences().unwrap();
    assert_eq!(preferences.connection.profile_id, "default");
    assert_eq!(preferences.connection.serial, "offline-phone");
    assert!(preferences.automation.check_after_move);
    assert_eq!(preferences.weights.points, 51.0);
    let view = state.snapshot().unwrap();
    assert_eq!(view.device.profile_id, "default");
    assert_eq!(view.device.profile_ids, vec!["default".to_owned()]);
    assert!(view.device.serial.is_none());
    assert!(!view.device.busy);
    assert_eq!(
        store.load_preferences().unwrap().connection.profile_id,
        "default"
    );
    let stored: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("preferences.json")).unwrap()).unwrap();
    assert_eq!(stored["connection"]["profileId"], "default");
    drop(state);
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn imported_calibration_values_and_selection_are_durable_and_paths_are_narrow() {
    let root = directory();
    let state = NativeState::with_store(Store::at(root.clone()).unwrap()).unwrap();
    let raw =
        include_str!("../../resources/default-profile.toml").replace("gain = 1.4", "gain = 1.5");
    let view = state.snapshot().unwrap();
    state
        .device_request(DeviceRequest {
            command_id: "import-profile".to_owned(),
            expected_revision: view.device.revision,
            action: DeviceAction::Profile {
                id: "my-phone".to_owned(),
                toml: raw.clone(),
            },
        })
        .unwrap();
    assert_eq!(state.snapshot().unwrap().device.profile_id, "my-phone");
    assert!(root.join("profiles/my-phone.toml").is_file());
    drop(state);
    let next = NativeState::with_store(Store::at(root.clone()).unwrap()).unwrap();
    let view = next.snapshot().unwrap();
    assert_eq!(view.device.profile_id, "my-phone");
    assert!(view.device.profile_ids.contains(&"my-phone".to_owned()));
    assert!(
        next.device_request(DeviceRequest {
            command_id: "bad-path".to_owned(),
            expected_revision: view.device.revision,
            action: DeviceAction::Profile {
                id: "../escape".to_owned(),
                toml: raw
            }
        })
        .is_err()
    );
    drop(next);
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn corrupt_execution_history_is_preserved_and_does_not_disable_manual_input() {
    let root = directory();
    std::fs::write(root.join("execution.json"), b"incomplete journal record").unwrap();
    let state = NativeState::with_store(Store::at(root.clone()).unwrap()).unwrap();
    let snapshot = state.snapshot().unwrap();
    assert!(
        snapshot
            .device
            .error
            .unwrap()
            .contains("Preserve and repair")
    );
    assert_eq!(snapshot.occupied, 0);
    assert!(state.snapshot().unwrap().capabilities.board_input);
    assert_eq!(
        std::fs::read(root.join("execution.json")).unwrap(),
        b"incomplete journal record"
    );
    drop(state);
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn stale_preferences_cannot_overwrite_newly_saved_shapes_or_controls() {
    let state = NativeState::default();
    let old = state.preferences().unwrap();
    let mut next = old.clone();
    next.shapes.push(SavedShape {
        id: "custom-first".to_owned(),
        name: "First".to_owned(),
        cells: vec![(0, 0)],
    });
    let accepted = state.save_preferences(next).unwrap();
    assert!(accepted.revision > old.revision);
    assert!(state.save_preferences(old).is_err());
    assert_eq!(state.preferences().unwrap().shapes.len(), 1);
}

#[test]
fn retired_animation_preferences_are_ignored_and_removed_on_save() {
    for animation in [false, true] {
        let root = directory();
        let store = Store::at(root.clone()).unwrap();
        let mut old = serde_json::to_value(Preferences::default()).unwrap();
        old["automation"]["animation"] = serde_json::json!(animation);
        old["automation"]["confirmMove"] = serde_json::json!(true);
        old["automation"]["confirmCycle"] = serde_json::json!(true);
        std::fs::write(
            root.join("preferences.json"),
            serde_json::to_vec(&old).unwrap(),
        )
        .unwrap();
        let prefs = store.load_preferences().unwrap();

        store.save_preferences(&prefs).unwrap();
        let saved: serde_json::Value =
            serde_json::from_slice(&std::fs::read(root.join("preferences.json")).unwrap()).unwrap();
        for retired in ["animation", "confirmMove", "confirmCycle"] {
            assert!(saved["automation"].get(retired).is_none());
        }
        old["automation"]["confirmMoove"] = serde_json::json!(true);
        std::fs::write(
            root.join("preferences.json"),
            serde_json::to_vec(&old).unwrap(),
        )
        .unwrap();
        assert!(store.load_preferences().is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn legacy_history_is_readable_and_new_records_do_not_write_workspace_modes() {
    let root = directory();
    let store = Store::at(root.clone()).unwrap();
    let old = serde_json::json!([{"timestamp":"1","mode":"manual","cycle":1,"status":"completed","message":"Old cycle","board":vec!["........";8]}]);
    std::fs::write(root.join("history.json"), serde_json::to_vec(&old).unwrap()).unwrap();
    let mut entries = store.load_history().unwrap();
    assert_eq!(entries[0].source, "manual");
    entries[0].source = "simulated".to_owned();
    store.save_history(&entries).unwrap();
    let saved: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("history.json")).unwrap()).unwrap();
    assert!(saved[0].get("mode").is_none());
    assert_eq!(saved[0]["source"], "simulated");
    std::fs::remove_dir_all(root).unwrap();
}
