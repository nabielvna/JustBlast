use justblast_desktop::api::{NativeState, models::*};
use std::time::{Duration, Instant};
fn input() -> NativeState {
    let state = NativeState::default();
    state
        .action(WorkspaceAction::Pieces {
            expected_revision: 1,
            pieces: (0..3)
                .map(|slot| PieceView {
                    slot,
                    catalog_id: None,
                    name: "Single".to_owned(),
                    cells: vec![(0, 0)],
                })
                .collect(),
        })
        .unwrap();
    state
}
#[test]
fn search_cancellation_and_edits_reject_late_results_without_holding_the_state_mutex() {
    let state = input();
    let revision = state.snapshot().unwrap().revision;
    state
        .action(WorkspaceAction::Search {
            expected_revision: revision,
            max_nodes: None,
        })
        .unwrap();
    let searched = state.snapshot().unwrap();
    let started = Instant::now();
    for _ in 0..20 {
        state.snapshot().unwrap();
    }
    assert!(started.elapsed() < Duration::from_millis(200));
    let mut rows = std::array::from_fn(|_| "........".to_owned());
    rows[7] = ".......#".to_owned();
    let next = state
        .replace_board(ReplaceBoardRequest {
            expected_revision: searched.revision,
            rows,
        })
        .unwrap();
    assert_eq!(next.occupied, 1);
    assert!(next.plan.is_none());
    std::thread::sleep(Duration::from_millis(100));
    let after = state.snapshot().unwrap();
    assert_eq!(after.revision, next.revision);
    assert_eq!(after.occupied, 1);
    assert!(after.plan.is_none());
}
#[test]
fn partial_search_never_commits_a_cycle_and_repeated_next_does_not_advance_twice() {
    let state = input();
    let revision = state.snapshot().unwrap().revision;
    state
        .action(WorkspaceAction::Search {
            expected_revision: revision,
            max_nodes: Some(2),
        })
        .unwrap();
    let started = Instant::now();
    loop {
        if state
            .snapshot()
            .unwrap()
            .job
            .is_some_and(|j| !matches!(j.status, JobStatus::Running))
        {
            break;
        }
        assert!(started.elapsed() < Duration::from_secs(5));
        std::thread::sleep(Duration::from_millis(5));
    }
    let view = state.snapshot().unwrap();
    assert!(!view.plan.unwrap().complete);
    assert!(
        state
            .action(WorkspaceAction::NextCycle {
                expected_revision: view.revision
            })
            .is_err()
    );
    assert_eq!(state.snapshot().unwrap().cycle, 1);
    let revision = state.snapshot().unwrap().revision;
    state
        .action(WorkspaceAction::Pieces {
            expected_revision: revision,
            pieces: vec![PieceView {
                slot: 1,
                catalog_id: None,
                name: "Single".to_owned(),
                cells: vec![(0, 0)],
            }],
        })
        .unwrap();
    let revision = state.snapshot().unwrap().revision;
    state
        .action(WorkspaceAction::Search {
            expected_revision: revision,
            max_nodes: None,
        })
        .unwrap();
    while state
        .snapshot()
        .unwrap()
        .job
        .is_some_and(|j| matches!(j.status, JobStatus::Running))
    {
        assert!(started.elapsed() < Duration::from_secs(5));
        std::thread::sleep(Duration::from_millis(5));
    }
    let revision = state.snapshot().unwrap().revision;
    state
        .action(WorkspaceAction::NextCycle {
            expected_revision: revision,
        })
        .unwrap();
    assert!(
        state
            .action(WorkspaceAction::NextCycle {
                expected_revision: revision
            })
            .is_err()
    );
    assert_eq!(state.snapshot().unwrap().cycle, 2);
}
#[test]
fn a_budget_exhausted_before_the_first_move_does_not_claim_no_legal_moves() {
    let state = input();
    let revision = state.snapshot().unwrap().revision;
    state
        .action(WorkspaceAction::Search {
            expected_revision: revision,
            max_nodes: Some(1),
        })
        .unwrap();
    let started = Instant::now();
    while state
        .snapshot()
        .unwrap()
        .job
        .is_some_and(|j| matches!(j.status, JobStatus::Running))
    {
        assert!(started.elapsed() < Duration::from_secs(5));
        std::thread::sleep(Duration::from_millis(5));
    }
    let view = state.snapshot().unwrap();
    assert_eq!(view.phase, "awaiting_input");
    assert!(!view.plan.unwrap().complete);
}
