mod support;
use justblast_desktop::{
    api::{NativeState, models::*},
    preferences::Store,
};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use support::{Fake, Phone};
fn state(phone: Arc<Mutex<Phone>>) -> NativeState {
    NativeState::with_transport(Store::default(), Box::new(Fake(phone))).unwrap()
}
fn request(state: &NativeState, action: DeviceAction) -> WorkspaceView {
    let view = state.snapshot().unwrap();
    state
        .device_request(DeviceRequest {
            command_id: format!("request-{}", view.device.revision),
            expected_revision: view.device.revision,
            action,
        })
        .unwrap()
}
fn idle(state: &NativeState) -> WorkspaceView {
    let started = Instant::now();
    loop {
        let snapshot = state.snapshot().unwrap();
        if !snapshot.device.busy
            && snapshot
                .job
                .as_ref()
                .is_none_or(|j| !matches!(j.status, JobStatus::Running))
        {
            return snapshot;
        }
        assert!(
            started.elapsed() < Duration::from_secs(60),
            "{}",
            snapshot.device.message
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}
fn ready(state: &NativeState) {
    request(
        state,
        DeviceAction::Connect {
            serial: "offline-phone".to_owned(),
            install_helper: false,
        },
    );
    assert_eq!(idle(state).device.state, "connected");
    let revision = state.snapshot().unwrap().revision;
    request(
        state,
        DeviceAction::Scan {
            workspace_revision: revision,
        },
    );
    let view = idle(state);
    assert_eq!(view.device.state, "ready", "{:?}", view.device.error);
    assert!(view.plan.unwrap().complete);
}
fn enabled(state: &NativeState) {
    let mut prefs = state.preferences().unwrap();
    prefs.automation.move_device = true;
    prefs.automation.check_after_cycle = false;
    state.save_preferences(prefs).unwrap();
}
fn unique_directory(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "justblast-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&path).unwrap();
    path
}
#[test]
fn construction_and_preview_never_send_movements() {
    let phone = Arc::new(Mutex::new(Phone::default()));
    let state = state(phone.clone());
    assert_eq!(phone.lock().unwrap().captures, 0);
    ready(&state);
    let snapshot = state.snapshot().unwrap();
    let error = state
        .device_request(DeviceRequest {
            command_id: "disabled-start".to_owned(),
            expected_revision: snapshot.device.revision,
            action: DeviceAction::Start {
                workspace_revision: snapshot.revision,
                through: None,
            },
        })
        .unwrap_err();
    assert!(error.message.contains("Enable Move device"));
    assert_eq!(phone.lock().unwrap().calls, 0);
}
#[test]
fn unchecked_cycle_is_one_batch_and_duplicate_commands_never_replay() {
    let phone = Arc::new(Mutex::new(Phone::default()));
    let state = state(phone.clone());
    ready(&state);
    enabled(&state);
    let snapshot = state.snapshot().unwrap();
    let req = || DeviceRequest {
        command_id: "one-start".to_owned(),
        expected_revision: snapshot.device.revision,
        action: DeviceAction::Start {
            workspace_revision: snapshot.revision,
            through: None,
        },
    };
    state.device_request(req()).unwrap();
    state.device_request(req()).unwrap();
    let next = idle(&state);
    assert_eq!(
        next.device.state, "cycle_completed",
        "{:?}",
        next.device.error
    );
    assert_eq!(next.cycle, 1);
    assert_eq!(phone.lock().unwrap().batches, [2]);
    assert_eq!(next.device.visual.unwrap().completed, 2);
    state.device_request(req()).unwrap();
    assert_eq!(phone.lock().unwrap().calls, 1);
}
#[test]
fn step_commands_preserve_backend_progress_and_reject_completed_steps() {
    let phone = Arc::new(Mutex::new(Phone::default()));
    let state = state(phone.clone());
    ready(&state);
    enabled(&state);
    let revision = state.snapshot().unwrap().revision;
    request(
        &state,
        DeviceAction::Start {
            workspace_revision: revision,
            through: Some(0),
        },
    );
    let snapshot = idle(&state);
    assert_eq!(snapshot.device.completed, 1);
    assert_eq!(snapshot.device.state, "ready");
    let duplicate = state.device_request(DeviceRequest {
        command_id: "repeat-step".to_owned(),
        expected_revision: snapshot.device.revision,
        action: DeviceAction::Start {
            workspace_revision: revision,
            through: Some(0),
        },
    });
    assert!(duplicate.is_err());
    request(
        &state,
        DeviceAction::Start {
            workspace_revision: revision,
            through: Some(1),
        },
    );
    let next = idle(&state);
    assert_eq!(next.cycle, 1, "{:?}", next.device.error);
    assert_eq!(phone.lock().unwrap().batches, [1, 1]);
}
#[test]
fn lost_acknowledgement_requires_stable_scan_and_does_not_replay() {
    let phone = Arc::new(Mutex::new(Phone {
        fail_after_send: true,
        ..Phone::default()
    }));
    let state = state(phone.clone());
    ready(&state);
    enabled(&state);
    let revision = state.snapshot().unwrap().revision;
    request(
        &state,
        DeviceAction::Start {
            workspace_revision: revision,
            through: None,
        },
    );
    let snapshot = idle(&state);
    assert_eq!(snapshot.device.state, "uncertain");
    assert!(snapshot.device.needs_scan);
    assert_eq!(phone.lock().unwrap().calls, 1);
    assert!(
        state
            .device_request(DeviceRequest {
                command_id: "unsafe-retry".to_owned(),
                expected_revision: snapshot.device.revision,
                action: DeviceAction::Start {
                    workspace_revision: revision,
                    through: None
                }
            })
            .is_err()
    );
    phone.lock().unwrap().fail_after_send = false;
    request(
        &state,
        DeviceAction::Scan {
            workspace_revision: snapshot.revision,
        },
    );
    let next = idle(&state);
    assert_eq!(next.device.state, "ready", "{:?}", next.device.error);
    assert_eq!(phone.lock().unwrap().calls, 1);
}
#[test]
fn an_execution_intent_write_failure_prevents_any_send_and_survives_restart() {
    let root = unique_directory("journal");
    let phone = Arc::new(Mutex::new(Phone::default()));
    let state = NativeState::with_transport(
        Store::at(root.clone()).unwrap(),
        Box::new(Fake(phone.clone())),
    )
    .unwrap();
    ready(&state);
    enabled(&state);
    std::fs::remove_file(root.join("execution.json")).unwrap();
    std::fs::create_dir(root.join("execution.json")).unwrap();
    let revision = state.snapshot().unwrap().revision;
    request(
        &state,
        DeviceAction::Start {
            workspace_revision: revision,
            through: None,
        },
    );
    let snapshot = idle(&state);
    assert_eq!(phone.lock().unwrap().calls, 0);
    assert_eq!(snapshot.device.state, "uncertain");
    assert!(snapshot.device.error.unwrap().contains("No movement sent"));
    state.shutdown();
    drop(state);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn continuous_execution_keeps_the_latest_cycle_without_waiting_for_presentation() {
    let phone = Arc::new(Mutex::new(Phone::default()));
    let state = state(phone.clone());
    ready(&state);
    let mut prefs = state.preferences().unwrap();
    prefs.automation.move_device = true;
    prefs.automation.check_after_cycle = false;
    prefs.automation.continuous = true;
    prefs.automation.cycle_limit = 2;
    state.save_preferences(prefs).unwrap();
    let revision = state.snapshot().unwrap().revision;
    request(
        &state,
        DeviceAction::Start {
            workspace_revision: revision,
            through: None,
        },
    );
    let next = idle(&state);
    assert_eq!(next.cycle, 2, "{:?}", next.device.error);
    assert_eq!(phone.lock().unwrap().batches, [2, 3]);
    assert_eq!(next.device.queued_visuals, 1);
    let visual = next.device.visual.unwrap();
    assert_eq!(visual.cycle, 2);
    assert_eq!(visual.completed, 3);
    assert!(visual.done);
    let current = request(
        &state,
        DeviceAction::AcknowledgeVisual {
            cycle_id: visual.id - 1,
        },
    );
    assert_eq!(current.device.visual.unwrap().id, visual.id);
}
#[test]
fn checked_moves_send_separate_batches_and_are_verified() {
    let phone = Arc::new(Mutex::new(Phone::default()));
    let state = state(phone.clone());
    ready(&state);
    let mut prefs = state.preferences().unwrap();
    prefs.automation.move_device = true;
    prefs.automation.check_after_cycle = false;
    prefs.automation.check_after_move = true;
    prefs.automation.check_after_cycle = true;
    state.save_preferences(prefs).unwrap();
    let revision = state.snapshot().unwrap().revision;
    request(
        &state,
        DeviceAction::Start {
            workspace_revision: revision,
            through: None,
        },
    );
    let started = Instant::now();
    loop {
        let view = state.snapshot().unwrap();
        if !view.device.busy {
            assert_eq!(view.cycle, 1, "{:?}", view.device.error);
            break;
        }
        assert!(started.elapsed() < Duration::from_secs(60));
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(phone.lock().unwrap().batches, [1, 1]);
}
#[test]
fn force_stop_interrupts_local_acknowledgement_and_prevents_retry() {
    let phone = Arc::new(Mutex::new(Phone {
        slow: true,
        ..Phone::default()
    }));
    let state = state(phone.clone());
    ready(&state);
    enabled(&state);
    let revision = state.snapshot().unwrap().revision;
    request(
        &state,
        DeviceAction::Start {
            workspace_revision: revision,
            through: None,
        },
    );
    let started = Instant::now();
    while phone.lock().unwrap().calls == 0 {
        assert!(started.elapsed() < Duration::from_secs(10));
        std::thread::sleep(Duration::from_millis(10));
    }
    request(&state, DeviceAction::ForceStop {});
    let view = idle(&state);
    assert_eq!(view.device.state, "uncertain");
    assert!(view.device.needs_scan);
    assert!(phone.lock().unwrap().force_seen);
    assert_eq!(phone.lock().unwrap().calls, 1);
}
#[test]
fn graceful_stop_finishes_the_authorized_batch_without_starting_another() {
    let phone = Arc::new(Mutex::new(Phone {
        slow: true,
        ..Phone::default()
    }));
    let state = state(phone.clone());
    ready(&state);
    let mut prefs = state.preferences().unwrap();
    prefs.automation.move_device = true;
    prefs.automation.check_after_cycle = false;
    prefs.automation.continuous = true;
    state.save_preferences(prefs).unwrap();
    let revision = state.snapshot().unwrap().revision;
    request(
        &state,
        DeviceAction::Start {
            workspace_revision: revision,
            through: None,
        },
    );
    let started = Instant::now();
    while phone.lock().unwrap().calls == 0 {
        assert!(started.elapsed() < Duration::from_secs(10));
        std::thread::sleep(Duration::from_millis(10));
    }
    request(&state, DeviceAction::Stop {});
    let view = idle(&state);
    assert_eq!(phone.lock().unwrap().batches, [2]);
    assert_eq!(view.cycle, 1);
    assert_eq!(view.device.state, "cycle_completed");
}

#[test]
fn alternatives_after_a_physical_step_cannot_replace_the_executed_prefix() {
    let phone = Arc::new(Mutex::new(Phone::default()));
    let state = state(phone.clone());
    ready(&state);
    enabled(&state);
    let revision = state.snapshot().unwrap().revision;
    request(
        &state,
        DeviceAction::Start {
            workspace_revision: revision,
            through: Some(0),
        },
    );
    let snapshot = idle(&state);
    let old = snapshot.plan.unwrap().steps[0].movement;
    assert!(
        state
            .action(WorkspaceAction::Explore {
                expected_revision: revision,
                completed: 0,
                max_nodes: None
            })
            .is_err()
    );
    state
        .action(WorkspaceAction::Explore {
            expected_revision: revision,
            completed: 1,
            max_nodes: None,
        })
        .unwrap();
    let started = Instant::now();
    loop {
        let view = state.snapshot().unwrap();
        if view.explorer_ready {
            break;
        }
        assert!(started.elapsed() < Duration::from_secs(10));
        std::thread::sleep(Duration::from_millis(10));
    }
    let page = state
        .explorer_page(ExplorerRequest {
            expected_revision: revision,
            order: vec![],
            positions: vec![],
            page: 0,
            page_size: 10,
        })
        .unwrap();
    let candidate = page.candidates.last().unwrap().id;
    state
        .action(WorkspaceAction::Select {
            expected_revision: revision,
            candidate_id: candidate,
            completed: 1,
        })
        .unwrap();
    let selected = state.snapshot().unwrap();
    let first = selected.plan.as_ref().unwrap().steps[0].movement;
    assert_eq!(
        (first.piece, first.row, first.column),
        (old.piece, old.row, old.column)
    );
    assert_eq!(selected.device.completed, 1);
    assert_eq!(selected.device.queued_visuals, 1);
    request(
        &state,
        DeviceAction::Start {
            workspace_revision: selected.revision,
            through: None,
        },
    );
    let next = idle(&state);
    assert_eq!(next.cycle, 1, "{:?}", next.device.error);
    assert_eq!(phone.lock().unwrap().batches, [1, 1]);
}

#[test]
fn unreadable_custom_piece_can_be_corrected_and_executed_against_the_same_image() {
    let custom = support::custom_ring();
    let phone = Arc::new(Mutex::new(Phone {
        slots: [Some(custom.clone()), Some(support::shape(3)), None],
        ..Phone::default()
    }));
    let state = state(phone.clone());
    request(
        &state,
        DeviceAction::Connect {
            serial: "offline-phone".to_owned(),
            install_helper: false,
        },
    );
    idle(&state);
    let view = state.snapshot().unwrap();
    request(
        &state,
        DeviceAction::Scan {
            workspace_revision: view.revision,
        },
    );
    let view = idle(&state);
    assert!(!view.input_known);
    assert!(
        view.device
            .capture
            .as_ref()
            .unwrap()
            .unknown_slots
            .contains(&0)
    );
    let profile = justblast_desktop::device::profile::Profile::builtin();
    let slot = profile.screen.slots[0];
    assert_eq!(
        view.pieces.iter().find(|p| p.slot == 0).unwrap().cells,
        custom.cells()
    );
    let mut pieces = view.pieces;
    pieces.retain(|p| p.slot != 0);
    pieces.push(PieceView {
        slot: 0,
        catalog_id: None,
        name: "Corrected ring".to_owned(),
        cells: custom.cells().to_vec(),
    });
    let revision = view.revision;
    state
        .action(WorkspaceAction::ApplyInput {
            expected_revision: revision,
            rows: Box::new(view.board),
            pieces,
            anchors: vec![SlotAnchor {
                slot: 0,
                x: (slot[2] - 216) / 2,
                y: (slot[3] - 216) / 2,
            }],
        })
        .unwrap();
    let view = idle(&state);
    assert_eq!(view.source, "corrected");
    assert_eq!(view.device.state, "ready", "{:?}", view.device.readiness);
    assert_eq!(phone.lock().unwrap().calls, 0);
    enabled(&state);
    let view = state.snapshot().unwrap();
    request(
        &state,
        DeviceAction::Start {
            workspace_revision: view.revision,
            through: None,
        },
    );
    let view = idle(&state);
    assert_eq!(
        view.device.state, "cycle_completed",
        "{:?}",
        view.device.error
    );
    assert_eq!(phone.lock().unwrap().batches, [2]);
    assert_eq!(view.completed, 2);
}

#[test]
fn rescan_immediately_replaces_entered_input_and_does_not_move() {
    let phone = Arc::new(Mutex::new(Phone::default()));
    let state = state(phone.clone());
    state
        .replace_board(ReplaceBoardRequest {
            expected_revision: 1,
            rows: std::array::from_fn(|_| "........".to_owned()),
        })
        .unwrap();
    request(
        &state,
        DeviceAction::Connect {
            serial: "offline-phone".to_owned(),
            install_helper: false,
        },
    );
    idle(&state);
    let view = state.snapshot().unwrap();
    request(
        &state,
        DeviceAction::Scan {
            workspace_revision: view.revision,
        },
    );
    let view = idle(&state);
    assert_eq!(view.occupied, 7);
    assert!(view.plan.is_some());
    assert_eq!(phone.lock().unwrap().calls, 0);
}

#[test]
fn next_cycle_reads_the_next_tray_only_after_explicit_request() {
    let phone = Arc::new(Mutex::new(Phone::default()));
    let state = state(phone.clone());
    ready(&state);
    enabled(&state);
    let view = state.snapshot().unwrap();
    request(
        &state,
        DeviceAction::Start {
            workspace_revision: view.revision,
            through: None,
        },
    );
    let done = idle(&state);
    assert_eq!(done.cycle, 1);
    assert_eq!(done.completed, 2);
    assert_eq!(state.history().unwrap().len(), 1);
    request(
        &state,
        DeviceAction::NextCycle {
            workspace_revision: done.revision,
        },
    );
    let next = idle(&state);
    assert_eq!(next.cycle, 2);
    assert_eq!(next.completed, 0);
    assert_eq!(next.plan.unwrap().steps.len(), 3);
    assert_eq!(phone.lock().unwrap().batches, [2]);
}

#[test]
fn correction_at_a_partial_pause_keeps_physical_prefix_across_move_permission_changes() {
    let phone = Arc::new(Mutex::new(Phone::default()));
    let state = state(phone.clone());
    ready(&state);
    enabled(&state);
    let view = state.snapshot().unwrap();
    request(
        &state,
        DeviceAction::Start {
            workspace_revision: view.revision,
            through: Some(0),
        },
    );
    let view = idle(&state);
    let first = serde_json::to_value(&view.plan.as_ref().unwrap().steps[0]).unwrap();
    let calls = phone.lock().unwrap().calls;
    request(
        &state,
        DeviceAction::CaptureForEdit {
            workspace_revision: view.revision,
        },
    );
    let view = idle(&state);
    assert_eq!(phone.lock().unwrap().calls, calls);
    state
        .action(WorkspaceAction::ApplyInput {
            expected_revision: view.revision,
            rows: Box::new(view.current_board),
            pieces: view.pieces,
            anchors: vec![],
        })
        .unwrap();
    let view = idle(&state);
    assert_eq!(view.completed, 1);
    assert_eq!(
        serde_json::to_value(&view.plan.as_ref().unwrap().steps[0]).unwrap(),
        first
    );
    let mut prefs = state.preferences().unwrap();
    prefs.automation.move_device = false;
    state.save_preferences(prefs).unwrap();
    let view = state.snapshot().unwrap();
    assert_eq!(view.completed, 1);
    assert!(
        state
            .action(WorkspaceAction::Explore {
                expected_revision: view.revision,
                completed: 0,
                max_nodes: None
            })
            .is_err()
    );
    enabled(&state);
    let view = state.snapshot().unwrap();
    request(
        &state,
        DeviceAction::Start {
            workspace_revision: view.revision,
            through: None,
        },
    );
    let done = idle(&state);
    assert_eq!(
        done.device.state, "cycle_completed",
        "{:?}",
        done.device.error
    );
    assert_eq!(phone.lock().unwrap().batches, [1, 1]);
    assert_eq!(done.completed, 2);
}

mod correction_tests {
    use super::support;
    use justblast_core::domain::{Board, Shape};
    use justblast_desktop::device::{
        correction::{Capture, Corrections, board_rect, mapped_piece, same_region},
        profile::Profile,
        vision::{Observation, Slot},
    };
    use support::{Phone, render};
    #[test]
    fn repeated_wrong_labels_are_overridden_only_while_corrected_visual_evidence_is_unchanged() {
        let profile = Profile::builtin();
        let phone = Phone::default();
        let mut baseline =
            Capture::read(1, render(&phone, &profile.screen), &profile.screen).unwrap();
        let correct = baseline.board.unwrap();
        baseline.board = Some(Board::default());
        let corrections = Corrections {
            capture: baseline.clone(),
            cells: vec![(0, 0, true)],
            slots: std::array::from_fn(|_| None),
        };
        let fixed = corrections
            .resolve(
                &baseline,
                &profile.screen,
                correct,
                Some(&baseline.tray.signature()),
            )
            .unwrap();
        assert_eq!(fixed.board.rows()[0], "#.......");
        let mut changed_phone = Phone::default();
        changed_phone.board = Board::default();
        let changed =
            Capture::read(2, render(&changed_phone, &profile.screen), &profile.screen).unwrap();
        assert!(
            corrections
                .resolve(
                    &changed,
                    &profile.screen,
                    correct,
                    Some(&baseline.tray.signature())
                )
                .is_err()
        );
    }
    #[test]
    fn image_comparison_tolerates_small_lighting_changes_and_rejects_tile_changes() {
        let screen = Profile::builtin().screen;
        let phone = Phone::default();
        let image = render(&phone, &screen);
        let mut slight = image.clone();
        for pixel in &mut slight.rgb {
            for channel in pixel {
                *channel = channel.saturating_add(3);
            }
        }
        assert!(same_region(&image, &slight, board_rect(&screen, 0, 0)));
        let changed = render(
            &Phone {
                board: Board::default(),
                ..Phone::default()
            },
            &screen,
        );
        assert!(!same_region(&image, &changed, board_rect(&screen, 0, 0)));
    }
    #[test]
    fn corrected_shape_requires_supported_cells_and_a_bounded_origin() {
        let screen = Profile::builtin().screen;
        let capture = Capture::read(1, render(&Phone::default(), &screen), &screen).unwrap();
        let single = Shape::new(vec![(0, 0)]).unwrap();
        assert!(mapped_piece(&capture, &screen, 0, single.clone(), None).is_ok());
        assert!(mapped_piece(&capture, &screen, 0, single, Some((0, 0))).is_err());
        assert!(
            mapped_piece(
                &capture,
                &screen,
                0,
                Shape::new(vec![(0, 0), (0, 1)]).unwrap(),
                Some((330, 360))
            )
            .is_err()
        );
        let wrong = Corrections {
            capture: capture.clone(),
            cells: vec![],
            slots: [Some(Slot::Empty), None, None],
        };
        let Observation { tray, .. } = wrong
            .resolve(&capture, &screen, capture.board.unwrap(), None)
            .unwrap();
        assert!(matches!(tray.slots[0], Slot::Recognized(_)));
    }
}

#[test]
fn stable_recovery_retains_unreadable_input_for_correction_without_retrying_touches() {
    use justblast_core::domain::Shape;
    let phone = Arc::new(Mutex::new(Phone::default()));
    let state = state(phone.clone());
    ready(&state);
    enabled(&state);
    phone.lock().unwrap().fail_after_send = true;
    let view = state.snapshot().unwrap();
    request(
        &state,
        DeviceAction::Start {
            workspace_revision: view.revision,
            through: None,
        },
    );
    let view = idle(&state);
    assert_eq!(view.device.state, "uncertain");
    {
        let mut phone = phone.lock().unwrap();
        phone.fail_after_send = false;
        phone.slots = [
            Some(
                Shape::new(
                    (0..4)
                        .flat_map(|r| {
                            (0..4).filter_map(move |c| {
                                (r == 0 || r == 3 || c == 0 || c == 3).then_some((r, c))
                            })
                        })
                        .collect(),
                )
                .unwrap(),
            ),
            None,
            None,
        ];
    }
    request(
        &state,
        DeviceAction::Scan {
            workspace_revision: view.revision,
        },
    );
    let recovered = idle(&state);
    assert_eq!(
        recovered.device.state, "needs_correction",
        "{:?}",
        recovered.device.error
    );
    assert!(!recovered.input_known);
    assert!(recovered.device.capture.unwrap().unknown_slots.contains(&0));
    assert_eq!(phone.lock().unwrap().calls, 1);
    assert!(
        state
            .action(WorkspaceAction::Search {
                expected_revision: recovered.revision,
                max_nodes: None
            })
            .is_err()
    );
}

#[test]
fn a_scan_of_a_changed_game_immediately_archives_the_old_prefix_and_loads_the_new_slots() {
    let phone = Arc::new(Mutex::new(Phone::default()));
    let state = state(phone.clone());
    ready(&state);
    enabled(&state);
    let view = state.snapshot().unwrap();
    request(
        &state,
        DeviceAction::Start {
            workspace_revision: view.revision,
            through: Some(0),
        },
    );
    let partial = idle(&state);
    assert_eq!(partial.completed, 1);
    {
        let mut phone = phone.lock().unwrap();
        phone.board = justblast_core::domain::Board::default();
        phone.slots = [Some(support::shape(9)), Some(support::shape(5)), None];
    }
    request(
        &state,
        DeviceAction::Scan {
            workspace_revision: partial.revision,
        },
    );
    let recovered = idle(&state);
    assert_eq!(
        recovered.device.state, "ready",
        "{:?}",
        recovered.device.error
    );
    assert_eq!(recovered.completed, 0);
    assert!(recovered.checkpoint > partial.checkpoint);
    assert_eq!(phone.lock().unwrap().batches, [1]);
    let history = state.history().unwrap();
    assert_eq!(history.last().unwrap().status, "reconciled");
    assert!(
        history
            .last()
            .unwrap()
            .message
            .contains("1 acknowledged placements archived")
    );
}

#[test]
fn damaged_phone_history_does_not_disable_unlinked_input_after_disconnect() {
    let root = unique_directory("unlinked-corrupt");
    std::fs::write(root.join("execution.json"), b"damaged history").unwrap();
    let phone = Arc::new(Mutex::new(Phone::default()));
    let state = NativeState::with_transport(
        Store::at(root.clone()).unwrap(),
        Box::new(Fake(phone.clone())),
    )
    .unwrap();
    request(
        &state,
        DeviceAction::Connect {
            serial: "offline-phone".to_owned(),
            install_helper: false,
        },
    );
    assert_eq!(idle(&state).device.provenance, "uncertain");
    request(&state, DeviceAction::Disconnect {});
    let disconnected = idle(&state);
    assert_eq!(disconnected.device.provenance, "none");
    state
        .action(WorkspaceAction::ApplyInput {
            expected_revision: disconnected.revision,
            rows: Box::new(std::array::from_fn(|_| "........".to_owned())),
            pieces: vec![PieceView {
                slot: 0,
                catalog_id: None,
                name: "Single".to_owned(),
                cells: vec![(0, 0)],
            }],
            anchors: vec![],
        })
        .unwrap();
    let view = idle(&state);
    assert!(view.plan.is_some());
    assert!(view.device.serial.is_none());
    assert_eq!(phone.lock().unwrap().calls, 0);
    assert_eq!(
        std::fs::read(root.join("execution.json")).unwrap(),
        b"damaged history"
    );
    drop(state);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn uncertain_shapes_keep_editable_cells_without_becoming_empty_or_trusted() {
    use justblast_desktop::device::vision::SlotSignature;
    use justblast_desktop::device::{
        profile::Profile,
        vision::{self, Slot},
    };
    use support::{Phone, render};
    let screen = Profile::builtin().screen;
    let ring = support::custom_ring();
    let phone = Phone {
        slots: [Some(ring.clone()), Some(support::shape(3)), None],
        ..Phone::default()
    };
    let tray = vision::read_tray(&render(&phone, &screen), &screen, (0, 0)).unwrap();
    let Slot::Uncertain {
        reason,
        candidate: Some(candidate),
    } = &tray.slots[0]
    else {
        panic!("{tray:?}")
    };
    assert_eq!(reason, "shape_not_in_catalogue");
    assert_eq!(candidate.shape, ring);
    assert_eq!(tray.pieces().len(), 1);
    assert_eq!(tray.editable_pieces().len(), 2);
    assert!(!tray.readable());
    assert_eq!(tray.signature()[0], SlotSignature::Unknown);
    assert_eq!(tray.signature()[2], SlotSignature::Empty);
    let empty = Phone {
        slots: [None, Some(support::shape(3)), None],
        ..Phone::default()
    };
    let clear = vision::read_tray(&render(&empty, &screen), &screen, (0, 0)).unwrap();
    assert_ne!(tray.signature(), clear.signature());
    assert!(matches!(clear.slots[0], Slot::Empty));
    assert_eq!(clear.editable_pieces().len(), 1);
}

#[test]
fn damaged_tile_retains_partial_shape_and_small_visible_fragment_is_not_empty() {
    use justblast_desktop::device::{
        profile::Profile,
        vision::{self, Slot},
    };
    use support::{Phone, render};
    let screen = Profile::builtin().screen;
    let phone = Phone {
        slots: [Some(support::shape(3)), None, None],
        ..Phone::default()
    };
    let mut image = render(&phone, &screen);
    let [x, y, w, h] = screen.slots[0].map(|v| v as usize);
    let pitch = screen.preview_pitch as usize;
    let left = x + (w - 2 * pitch) / 2;
    let top = y + (h - pitch) / 2;
    for row in top..top + pitch {
        for col in left + pitch..left + pitch + pitch / 2 {
            image.rgb[row * image.width + col] = [40, 70, 100];
        }
    }
    let tray = vision::read_tray(&image, &screen, (0, 0)).unwrap();
    let Slot::Uncertain {
        candidate: Some(p), ..
    } = &tray.slots[0]
    else {
        panic!("{tray:?}")
    };
    assert!(!p.shape.cells().is_empty());
    assert!(matches!(tray.slots[1], Slot::Empty));
    let mut fragment = render(
        &Phone {
            slots: [None, None, None],
            ..Phone::default()
        },
        &screen,
    );
    for row in top..top + 14 {
        for col in left..left + 14 {
            fragment.rgb[row * fragment.width + col] = [220, 100, 60];
        }
    }
    let tray = vision::read_tray(&fragment, &screen, (0, 0)).unwrap();
    assert!(matches!(tray.slots[0], Slot::Uncertain { .. }), "{tray:?}");
    assert!(matches!(tray.slots[1], Slot::Empty));
}

#[test]
fn rescans_replace_unknown_with_empty_and_empty_with_unknown_without_stale_pieces() {
    let phone = Arc::new(Mutex::new(Phone {
        slots: [Some(support::custom_ring()), None, None],
        ..Phone::default()
    }));
    let state = state(phone.clone());
    request(
        &state,
        DeviceAction::Connect {
            serial: "offline-phone".to_owned(),
            install_helper: false,
        },
    );
    idle(&state);
    for slot in [Some(0), None, Some(2)] {
        phone.lock().unwrap().slots =
            std::array::from_fn(|i| (slot == Some(i)).then(support::custom_ring));
        let view = state.snapshot().unwrap();
        request(
            &state,
            DeviceAction::Scan {
                workspace_revision: view.revision,
            },
        );
        let view = idle(&state);
        let capture = view.device.capture.unwrap();
        assert_eq!(
            capture.unknown_slots,
            slot.into_iter().map(|i| i as u8).collect::<Vec<_>>()
        );
        assert_eq!(view.input_known, slot.is_none());
        if slot.is_none() {
            assert_eq!(view.device.state, "empty_tray");
            assert!(view.device.message.contains("Tray is empty"));
            assert!(view.device.readiness.contains("Tray is empty"));
        } else {
            assert_eq!(view.device.state, "needs_correction");
        }
        assert_eq!(view.pieces.len(), usize::from(slot.is_some()));
        if let Some(slot) = slot {
            assert_eq!(view.pieces[0].slot, slot as u8);
            assert!(view.plan.is_none());
        }
        assert_eq!(phone.lock().unwrap().calls, 0);
    }
}

#[test]
fn accepting_one_slot_keeps_other_uncertain_slots_unaccepted_and_accept_all_enables_phone() {
    let phone = Arc::new(Mutex::new(Phone {
        slots: [
            Some(support::custom_ring()),
            Some(support::custom_ring()),
            None,
        ],
        ..Phone::default()
    }));
    let state = state(phone.clone());
    request(
        &state,
        DeviceAction::Connect {
            serial: "offline-phone".to_owned(),
            install_helper: false,
        },
    );
    idle(&state);
    let revision = state.snapshot().unwrap().revision;
    request(
        &state,
        DeviceAction::Scan {
            workspace_revision: revision,
        },
    );
    let view = idle(&state);
    assert_eq!(
        view.device.capture.as_ref().unwrap().unknown_slots,
        vec![0, 1]
    );
    let updated = state
        .action(WorkspaceAction::UpdateInput {
            expected_revision: view.revision,
            rows: Box::new(view.current_board),
            pieces: view.pieces,
            anchors: vec![],
            accepted_slots: vec![0],
            accept_board: false,
        })
        .unwrap();
    assert!(!updated.input_known);
    assert!(updated.plan.is_none());
    assert_eq!(
        updated.device.capture.as_ref().unwrap().unknown_slots,
        vec![1]
    );
    assert_eq!(phone.lock().unwrap().calls, 0);
    state
        .action(WorkspaceAction::UpdateInput {
            expected_revision: updated.revision,
            rows: Box::new(updated.current_board),
            pieces: updated.pieces,
            anchors: vec![],
            accepted_slots: vec![0, 1],
            accept_board: false,
        })
        .unwrap();
    let ready = idle(&state);
    assert!(ready.input_known);
    assert!(
        ready
            .device
            .capture
            .as_ref()
            .unwrap()
            .unknown_slots
            .is_empty()
    );
    assert!(
        ready.device.readiness.is_empty(),
        "{}",
        ready.device.readiness
    );
    enabled(&state);
    request(
        &state,
        DeviceAction::Start {
            workspace_revision: ready.revision,
            through: Some(0),
        },
    );
    let done = idle(&state);
    assert_eq!(done.completed, 1, "{:?}", done.device.error);
    assert_eq!(phone.lock().unwrap().batches, vec![1]);
}

fn await_warning(state: &NativeState) -> WorkspaceView {
    let started = Instant::now();
    loop {
        let view = state.snapshot().unwrap();
        if view.device.state == "awaiting_confirmation" {
            return view;
        }
        assert!(
            view.device.busy,
            "{} {:?}",
            view.device.state, view.device.error
        );
        assert!(
            started.elapsed() < Duration::from_secs(30),
            "{}",
            view.device.message
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn a_position_warning_waits_for_acceptance_and_continues_only_the_remaining_prefix() {
    let phone = Arc::new(Mutex::new(Phone::default()));
    let state = state(phone.clone());
    ready(&state);
    enabled(&state);
    phone.lock().unwrap().board = justblast_core::domain::Board::default();
    let view = state.snapshot().unwrap();
    request(
        &state,
        DeviceAction::Start {
            workspace_revision: view.revision,
            through: Some(0),
        },
    );
    await_warning(&state);
    assert_eq!(phone.lock().unwrap().calls, 0);
    request(&state, DeviceAction::AcceptWarning {});
    let done = idle(&state);
    assert_eq!(done.completed, 1, "{:?}", done.device.error);
    assert_eq!(phone.lock().unwrap().batches, vec![1]);
    let err = state
        .device_request(DeviceRequest {
            command_id: "no-warning".to_owned(),
            expected_revision: done.device.revision,
            action: DeviceAction::AcceptWarning {},
        })
        .unwrap_err();
    assert!(err.message.contains("no position warning"));
}

#[test]
fn stopping_a_position_warning_sends_no_movements() {
    let phone = Arc::new(Mutex::new(Phone::default()));
    let state = state(phone.clone());
    ready(&state);
    enabled(&state);
    phone.lock().unwrap().board = justblast_core::domain::Board::default();
    let view = state.snapshot().unwrap();
    request(
        &state,
        DeviceAction::Start {
            workspace_revision: view.revision,
            through: None,
        },
    );
    await_warning(&state);
    request(&state, DeviceAction::Stop {});
    assert_eq!(idle(&state).device.state, "stopped");
    assert_eq!(phone.lock().unwrap().calls, 0);
}

#[test]
fn checked_move_and_cycle_mismatches_can_be_accepted_without_replaying_acknowledged_moves() {
    for after_move in [true, false] {
        let phone = Arc::new(Mutex::new(Phone {
            mismatch_after_send: true,
            ..Phone::default()
        }));
        let state = state(phone.clone());
        ready(&state);
        let mut prefs = state.preferences().unwrap();
        prefs.automation.move_device = true;
        prefs.automation.check_after_move = after_move;
        prefs.automation.check_after_cycle = !after_move;
        state.save_preferences(prefs).unwrap();
        let view = state.snapshot().unwrap();
        request(
            &state,
            DeviceAction::Start {
                workspace_revision: view.revision,
                through: if after_move { Some(0) } else { None },
            },
        );
        let warning = await_warning(&state);
        let count = if after_move { 1 } else { 2 };
        assert_eq!(warning.completed, count);
        assert_eq!(phone.lock().unwrap().calls, 1);
        request(&state, DeviceAction::AcceptWarning {});
        let done = idle(&state);
        assert_eq!(done.completed, count);
        assert_eq!(phone.lock().unwrap().calls, 1);
        assert!(done.device.error.is_none(), "{:?}", done.device.error);
    }
}

#[test]
fn accepting_geometry_does_not_require_classifier_pixel_support_but_still_checks_slot_bounds() {
    use justblast_desktop::device::{
        correction::{Capture, accepted_piece},
        profile::Profile,
    };
    let screen = Profile::builtin().screen;
    let capture = Capture::read(1, support::render(&Phone::default(), &screen), &screen).unwrap();
    assert!(accepted_piece(&capture, &screen, 0, support::shape(1), Some((0, 0))).is_ok());
    assert!(
        accepted_piece(
            &capture,
            &screen,
            0,
            support::shape(1),
            Some((screen.slots[0][2], 0))
        )
        .is_err()
    );
}

#[test]
fn stop_also_interrupts_a_warning_during_final_cycle_verification() {
    let phone = Arc::new(Mutex::new(Phone {
        mismatch_after_send: true,
        ..Phone::default()
    }));
    let state = state(phone.clone());
    ready(&state);
    let mut prefs = state.preferences().unwrap();
    prefs.automation.move_device = true;
    prefs.automation.check_after_cycle = true;
    state.save_preferences(prefs).unwrap();
    let view = state.snapshot().unwrap();
    request(
        &state,
        DeviceAction::Start {
            workspace_revision: view.revision,
            through: None,
        },
    );
    let warning = await_warning(&state);
    assert_eq!(warning.completed, 2);
    request(&state, DeviceAction::Stop {});
    assert_eq!(idle(&state).device.state, "stopped");
    assert_eq!(phone.lock().unwrap().batches, vec![2]);
}

#[test]
fn a_new_scan_after_a_simulation_branch_cannot_inherit_cycle_completed() {
    let phone = Arc::new(Mutex::new(Phone::default()));
    let state = state(phone.clone());
    ready(&state);
    enabled(&state);
    let view = state.snapshot().unwrap();
    request(
        &state,
        DeviceAction::Start {
            workspace_revision: view.revision,
            through: None,
        },
    );
    let done = idle(&state);
    assert!(done.device.cycle_done);
    assert_eq!(done.completed, 2);
    let movements = phone.lock().unwrap().calls;
    let mut prefs = state.preferences().unwrap();
    prefs.automation.move_device = false;
    state.save_preferences(prefs).unwrap();
    state
        .action(WorkspaceAction::NextCycle {
            expected_revision: done.revision,
        })
        .unwrap();
    let branched = state.snapshot().unwrap();
    assert_eq!(branched.completed, 0);
    request(
        &state,
        DeviceAction::Scan {
            workspace_revision: branched.revision,
        },
    );
    let fresh = idle(&state);
    assert_eq!(fresh.completed, 0);
    assert_eq!(phone.lock().unwrap().calls, movements);
    assert!(
        !fresh.device.cycle_done,
        "New scan inherited completion: {}",
        fresh.device.message
    );
    assert_eq!(fresh.device.state, "ready");
    assert!(fresh.device.readiness.is_empty());
}
