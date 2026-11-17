use justblast_core::{
    domain::{Board, Shape, catalog::evaluation_shapes},
    services::{Workspace, WorkspaceError},
    solver::{self, Piece, SearchOptions},
};
fn board(rows: [&str; 8]) -> Board {
    Board::from_rows(&rows.map(String::from)).unwrap()
}
fn pieces() -> Vec<Piece> {
    (0..2)
        .map(|slot| Piece {
            slot,
            catalog_id: None,
            name: format!("Piece {slot}"),
            shape: Shape::new(vec![(0, 0)]).unwrap(),
        })
        .collect()
}
fn solve(w: &mut Workspace) {
    let (revision, game) = w.begin_search(w.snapshot().revision).unwrap();
    let result = solver::solve(
        &game,
        &evaluation_shapes(),
        SearchOptions::default(),
        |_| true,
    )
    .unwrap();
    w.accept_search(revision, result).unwrap();
}
#[test]
fn bitboard_preserves_highest_bit_without_javascript_number_rounding() {
    let rows = [
        "#.......", "........", "........", "........", "........", "........", "........",
        ".......#",
    ];
    let value = board(rows);
    assert_eq!(value.rows(), rows.map(String::from));
    assert_eq!(value.occupied(), 2);
}
#[test]
fn rejects_malformed_boards_including_multibyte_characters() {
    let valid = vec!["........".to_owned(); 8];
    assert!(Board::from_rows(&valid[..7]).is_err());
    for bad in [".........", ".......x", "......é", "......."] {
        let mut rows = valid.clone();
        rows[0] = bad.to_owned();
        assert!(Board::from_rows(&rows).is_err());
    }
}
#[test]
fn one_empty_workspace_accepts_input_and_rejects_stale_atomic_edits() {
    let mut w = Workspace::default();
    assert!(!w.snapshot().input_known);
    assert_eq!(w.snapshot().completed, 0);
    let accepted = w
        .apply_input(1, Board::default(), pieces(), "entered")
        .unwrap();
    assert!(accepted.input_known);
    assert_eq!(
        w.apply_input(1, board(["########"; 8]), vec![], "entered"),
        Err(WorkspaceError::StaleRevision)
    );
    assert_eq!(w.snapshot(), accepted);
}
#[test]
fn correction_after_an_acknowledgement_preserves_history_and_solves_only_the_suffix() {
    let mut w = Workspace::default();
    w.apply_input(1, Board::default(), pieces(), "entered")
        .unwrap();
    solve(&mut w);
    let first = w.snapshot().plan.unwrap().steps[0].clone();
    w.acknowledge(1).unwrap();
    let corrected = board([
        "........", "........", "........", "........", "........", "........", "........",
        ".......#",
    ]);
    let revision = w.snapshot().revision;
    w.apply_input(revision, corrected, pieces(), "corrected")
        .unwrap();
    solve(&mut w);
    let s = w.snapshot();
    let plan = s.plan.unwrap();
    assert_eq!(s.completed, 1);
    assert_eq!(plan.steps[0], first);
    assert_eq!(plan.steps[1].before, corrected);
    assert_ne!(plan.steps[0].slot, plan.steps[1].slot);
    assert_eq!(w.acknowledge(0), Err(WorkspaceError::PrefixChanged));
    let mut changed = pieces();
    changed[usize::from(first.slot)].shape = Shape::new(vec![(0, 0), (0, 1)]).unwrap();
    assert_eq!(
        w.apply_input(s.revision, corrected, changed, "corrected"),
        Err(WorkspaceError::PrefixChanged)
    );
}
#[test]
fn revised_weights_cannot_unlock_completed_steps_and_cycle_commits_once() {
    let mut w = Workspace::default();
    w.apply_input(1, Board::default(), pieces(), "entered")
        .unwrap();
    solve(&mut w);
    w.acknowledge(1).unwrap();
    let mut weights = w.snapshot().weights;
    weights.points += 1.0;
    w.set_weights(w.snapshot().revision, weights).unwrap();
    solve(&mut w);
    assert_eq!(w.snapshot().completed, 1);
    let revision = w.snapshot().revision;
    let next = w.next_cycle(revision).unwrap();
    assert_eq!(next.cycle, 2);
    assert_eq!(next.completed, 0);
    assert!(next.pieces.is_empty());
    assert_eq!(next.source, "predicted");
    assert_eq!(w.next_cycle(revision), Err(WorkspaceError::StaleRevision));
}

#[test]
fn removing_an_unexecuted_slot_preserves_historical_slot_identity_when_ordinals_change() {
    let mut w = Workspace::default();
    w.apply_input(1, Board::default(), pieces(), "entered")
        .unwrap();
    solve(&mut w);
    let plan = w.snapshot().plan.unwrap();
    let shapes = evaluation_shapes();
    let mut path = plan.candidate.path.clone();
    path.reverse();
    let revision = w.snapshot().revision;
    w.select_path(revision, &path, 0, plan.maxima, true, &shapes)
        .unwrap();
    let historical = w.snapshot().plan.unwrap().steps[0].clone();
    assert_eq!(historical.slot, 1);
    w.acknowledge(1).unwrap();
    let kept = pieces().into_iter().filter(|p| p.slot == 1).collect();
    w.apply_input(w.snapshot().revision, w.current_board(), kept, "corrected")
        .unwrap();
    solve(&mut w);
    let snapshot = w.snapshot();
    assert_eq!(snapshot.completed, 1);
    assert_eq!(snapshot.plan.as_ref().unwrap().steps, [historical]);
    assert_eq!(snapshot.plan.unwrap().candidate.path[0].piece, 0);
}

#[test]
fn unknown_reconciliation_cannot_be_searched_as_an_empty_board() {
    let mut w = Workspace::default();
    w.apply_input(1, Board::default(), pieces(), "entered")
        .unwrap();
    solve(&mut w);
    w.acknowledge(1).unwrap();
    w.require_input_review();
    let snapshot = w.snapshot();
    assert!(!snapshot.input_known);
    assert_eq!(snapshot.completed, 1);
    assert_eq!(
        w.begin_search(snapshot.revision).unwrap_err(),
        WorkspaceError::InvalidInput
    );
}
