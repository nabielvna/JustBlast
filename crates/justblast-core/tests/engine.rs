use justblast_core::domain::combo::*;
use justblast_core::domain::{Board, Shape, catalog};
use justblast_core::solver::evaluation::{Evaluator, fragmentation};
use justblast_core::solver::*;
fn game(board: Board, pieces: Vec<Piece>) -> Game {
    Game {
        checkpoint: None,
        board,
        pieces,
        states: [ComboState::default(); 3],
        context: RuleContext::default(),
        weights: Weights::default(),
    }
}
fn piece(slot: u8, shape: Shape) -> Piece {
    Piece {
        slot,
        catalog_id: None,
        name: format!("Slot {slot}"),
        shape,
    }
}
fn shape(id: u32) -> Shape {
    catalog::catalog()
        .into_iter()
        .find(|x| x.0 == id)
        .unwrap()
        .1
}
#[test]
fn reference_plan_matches_path_boards_scores_and_normalization() {
    let mut game = game(
        Board::from_rows(&[
            "...##...".to_owned(),
            "....#...".to_owned(),
            "##......".to_owned(),
            ".##.....".to_owned(),
            "...#....".to_owned(),
            "##......".to_owned(),
            "###.....".to_owned(),
            "..##....".to_owned(),
        ])
        .unwrap(),
        vec![piece(0, shape(5)), piece(1, shape(9)), piece(2, shape(27))],
    );
    game.states = [
        ComboState {
            count: 0,
            misses: 0,
            producer_remaining: 0,
            saved_count: 0,
            checking: false,
            waiting: 0,
            recovered: false,
        },
        ComboState {
            count: 0,
            misses: 0,
            producer_remaining: 0,
            saved_count: 0,
            checking: false,
            waiting: 0,
            recovered: false,
        },
        ComboState {
            count: 0,
            misses: 0,
            producer_remaining: 0,
            saved_count: 0,
            checking: false,
            waiting: 0,
            recovered: false,
        },
    ];
    let result = solve(
        &game,
        &catalog::evaluation_shapes(),
        SearchOptions::default(),
        |_| true,
    )
    .unwrap();
    assert!(result.complete);
    assert_eq!(
        result.candidate.path,
        vec![
            Move {
                piece: 0,
                row: 6,
                column: 3
            },
            Move {
                piece: 1,
                row: 6,
                column: 6
            },
            Move {
                piece: 2,
                row: 3,
                column: 0
            }
        ]
    );
    assert_eq!(result.maxima, [20, 20, 20]);
    assert_eq!(result.candidate.gains, [20, 20, 20]);
    assert_eq!(
        result.candidate.board.rows(),
        [
            "...##...".to_owned(),
            "....#...".to_owned(),
            "##......".to_owned(),
            "###.....".to_owned(),
            "##.#....".to_owned(),
            "##......".to_owned(),
            "........".to_owned(),
            "..##..##".to_owned()
        ]
    );
    assert!((result.metrics.value - 95.0).abs() < 1e-12);
    assert!((result.metrics.potential_points - 1.0).abs() < 1e-12);
    assert!((result.metrics.geometry.flexibility - 1.0).abs() < 1e-12);
    assert!((result.metrics.geometry.fragmentation - 0.0).abs() < 1e-12);
}
#[test]
fn extracted_combo_source_cases_match_every_transition() {
    {
        let mut s = ComboState {
            count: 6,
            misses: 0,
            producer_remaining: 2,
            saved_count: 6,
            checking: false,
            waiting: 1,
            recovered: false,
        };
        s = advance_combo(
            s,
            Scenario::Basic,
            3,
            2,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 0,
                producer_remaining: 2,
                saved_count: 6,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            1,
            1,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 0,
                producer_remaining: 2,
                saved_count: 6,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            0,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 0,
                producer_remaining: 2,
                saved_count: 6,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            3,
            2,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 0,
                producer_remaining: 2,
                saved_count: 6,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            1,
            1,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 0,
                producer_remaining: 2,
                saved_count: 6,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            0,
            0,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 1,
                producer_remaining: 2,
                saved_count: 6,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            2,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 0,
                producer_remaining: 2,
                saved_count: 6,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            3,
            1,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 0,
                producer_remaining: 2,
                saved_count: 6,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 11,
            misses: 4,
            producer_remaining: 2,
            saved_count: 3,
            checking: true,
            waiting: 0,
            recovered: true,
        };
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 12,
                misses: 0,
                producer_remaining: 2,
                saved_count: 3,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            3,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 13,
                misses: 0,
                producer_remaining: 2,
                saved_count: 3,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            0,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 13,
                misses: 1,
                producer_remaining: 2,
                saved_count: 3,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            0,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 13,
                misses: 2,
                producer_remaining: 2,
                saved_count: 3,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            2,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 14,
                misses: 0,
                producer_remaining: 2,
                saved_count: 3,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            0,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 14,
                misses: 1,
                producer_remaining: 2,
                saved_count: 3,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            2,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 15,
                misses: 0,
                producer_remaining: 2,
                saved_count: 3,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 16,
                misses: 0,
                producer_remaining: 2,
                saved_count: 3,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
    }
    {
        let mut s = ComboState {
            count: 0,
            misses: 1,
            producer_remaining: 0,
            saved_count: 6,
            checking: false,
            waiting: 1,
            recovered: false,
        };
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 1,
                misses: 0,
                producer_remaining: 0,
                saved_count: 6,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            3,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 2,
                misses: 0,
                producer_remaining: 0,
                saved_count: 6,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 3,
                misses: 0,
                producer_remaining: 0,
                saved_count: 6,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 4,
                misses: 0,
                producer_remaining: 0,
                saved_count: 6,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            2,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 5,
                misses: 0,
                producer_remaining: 0,
                saved_count: 6,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 0,
                producer_remaining: 0,
                saved_count: 6,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            1,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 7,
                misses: 0,
                producer_remaining: 0,
                saved_count: 6,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            1,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 0,
                producer_remaining: 0,
                saved_count: 6,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 5,
            misses: 0,
            producer_remaining: 2,
            saved_count: 1,
            checking: true,
            waiting: 0,
            recovered: true,
        };
        s = advance_combo(
            s,
            Scenario::Basic,
            2,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 0,
                producer_remaining: 2,
                saved_count: 1,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 7,
                misses: 0,
                producer_remaining: 2,
                saved_count: 1,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            3,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 0,
                producer_remaining: 2,
                saved_count: 1,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 9,
                misses: 0,
                producer_remaining: 2,
                saved_count: 1,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 10,
                misses: 0,
                producer_remaining: 2,
                saved_count: 1,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            1,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 11,
                misses: 0,
                producer_remaining: 2,
                saved_count: 1,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            0,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 11,
                misses: 1,
                producer_remaining: 2,
                saved_count: 1,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            1,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 12,
                misses: 0,
                producer_remaining: 2,
                saved_count: 1,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
    }
    {
        let mut s = ComboState {
            count: 2,
            misses: 2,
            producer_remaining: 1,
            saved_count: 1,
            checking: false,
            waiting: 1,
            recovered: false,
        };
        s = advance_combo(
            s,
            Scenario::Basic,
            2,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 3,
                misses: 0,
                producer_remaining: 1,
                saved_count: 1,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            2,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 4,
                misses: 0,
                producer_remaining: 1,
                saved_count: 1,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            2,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 5,
                misses: 0,
                producer_remaining: 1,
                saved_count: 1,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 0,
                producer_remaining: 1,
                saved_count: 1,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            0,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 1,
                producer_remaining: 1,
                saved_count: 1,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 7,
                misses: 0,
                producer_remaining: 1,
                saved_count: 1,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            2,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 0,
                producer_remaining: 1,
                saved_count: 1,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            2,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 9,
                misses: 0,
                producer_remaining: 1,
                saved_count: 1,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 11,
            misses: 4,
            producer_remaining: 1,
            saved_count: 7,
            checking: true,
            waiting: 0,
            recovered: true,
        };
        s = advance_combo(
            s,
            Scenario::Basic,
            3,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 12,
                misses: 0,
                producer_remaining: 1,
                saved_count: 7,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            2,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 13,
                misses: 0,
                producer_remaining: 1,
                saved_count: 7,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 14,
                misses: 0,
                producer_remaining: 1,
                saved_count: 7,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            3,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 15,
                misses: 0,
                producer_remaining: 1,
                saved_count: 7,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 16,
                misses: 0,
                producer_remaining: 1,
                saved_count: 7,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            2,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 17,
                misses: 0,
                producer_remaining: 1,
                saved_count: 7,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            1,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 18,
                misses: 0,
                producer_remaining: 1,
                saved_count: 7,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 19,
                misses: 0,
                producer_remaining: 1,
                saved_count: 7,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
    }
    {
        let mut s = ComboState {
            count: 8,
            misses: 0,
            producer_remaining: 1,
            saved_count: 7,
            checking: false,
            waiting: 1,
            recovered: false,
        };
        s = advance_combo(
            s,
            Scenario::Basic,
            2,
            2,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 0,
                producer_remaining: 1,
                saved_count: 7,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            2,
            1,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 0,
                producer_remaining: 1,
                saved_count: 7,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            0,
            0,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 1,
                producer_remaining: 1,
                saved_count: 7,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            1,
            2,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 0,
                producer_remaining: 1,
                saved_count: 7,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            1,
            1,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 0,
                producer_remaining: 1,
                saved_count: 7,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            0,
            0,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 1,
                producer_remaining: 1,
                saved_count: 7,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            2,
            2,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 0,
                producer_remaining: 1,
                saved_count: 7,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            2,
            1,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 0,
                producer_remaining: 1,
                saved_count: 7,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 5,
            misses: 0,
            producer_remaining: 2,
            saved_count: 1,
            checking: true,
            waiting: 0,
            recovered: true,
        };
        s = advance_combo(
            s,
            Scenario::Basic,
            3,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 0,
                producer_remaining: 2,
                saved_count: 1,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 7,
                misses: 0,
                producer_remaining: 2,
                saved_count: 1,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            0,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 7,
                misses: 1,
                producer_remaining: 2,
                saved_count: 1,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            3,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 0,
                producer_remaining: 2,
                saved_count: 1,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            2,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 9,
                misses: 0,
                producer_remaining: 2,
                saved_count: 1,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            1,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 10,
                misses: 0,
                producer_remaining: 2,
                saved_count: 1,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            1,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 11,
                misses: 0,
                producer_remaining: 2,
                saved_count: 1,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            2,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 12,
                misses: 0,
                producer_remaining: 2,
                saved_count: 1,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
    }
    {
        let mut s = ComboState {
            count: 2,
            misses: 4,
            producer_remaining: 0,
            saved_count: 5,
            checking: false,
            waiting: 1,
            recovered: false,
        };
        s = advance_combo(
            s,
            Scenario::Basic,
            3,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 3,
                misses: 0,
                producer_remaining: 0,
                saved_count: 5,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            0,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 3,
                misses: 1,
                producer_remaining: 0,
                saved_count: 5,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            2,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 4,
                misses: 0,
                producer_remaining: 0,
                saved_count: 5,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            3,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 5,
                misses: 0,
                producer_remaining: 0,
                saved_count: 5,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            2,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 0,
                producer_remaining: 0,
                saved_count: 5,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            0,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 1,
                producer_remaining: 0,
                saved_count: 5,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            0,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 2,
                producer_remaining: 0,
                saved_count: 5,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            2,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 7,
                misses: 0,
                producer_remaining: 0,
                saved_count: 5,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 11,
            misses: 1,
            producer_remaining: 2,
            saved_count: 4,
            checking: true,
            waiting: 0,
            recovered: true,
        };
        s = advance_combo(
            s,
            Scenario::Basic,
            0,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 11,
                misses: 2,
                producer_remaining: 2,
                saved_count: 4,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            2,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 12,
                misses: 0,
                producer_remaining: 2,
                saved_count: 4,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            2,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 13,
                misses: 0,
                producer_remaining: 2,
                saved_count: 4,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            3,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 14,
                misses: 0,
                producer_remaining: 2,
                saved_count: 4,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            3,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 15,
                misses: 0,
                producer_remaining: 2,
                saved_count: 4,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            3,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 16,
                misses: 0,
                producer_remaining: 2,
                saved_count: 4,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            0,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 16,
                misses: 1,
                producer_remaining: 2,
                saved_count: 4,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            1,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 17,
                misses: 0,
                producer_remaining: 2,
                saved_count: 4,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
    }
    {
        let mut s = ComboState {
            count: 0,
            misses: 1,
            producer_remaining: 2,
            saved_count: 4,
            checking: false,
            waiting: 1,
            recovered: false,
        };
        s = advance_combo(
            s,
            Scenario::Basic,
            3,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 1,
                misses: 0,
                producer_remaining: 2,
                saved_count: 4,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            3,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 2,
                misses: 0,
                producer_remaining: 2,
                saved_count: 4,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            2,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 3,
                misses: 0,
                producer_remaining: 2,
                saved_count: 4,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 4,
                misses: 0,
                producer_remaining: 2,
                saved_count: 4,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            3,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 5,
                misses: 0,
                producer_remaining: 2,
                saved_count: 4,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            0,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 5,
                misses: 1,
                producer_remaining: 2,
                saved_count: 4,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 0,
                producer_remaining: 2,
                saved_count: 4,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            3,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 7,
                misses: 0,
                producer_remaining: 2,
                saved_count: 4,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 5,
            misses: 4,
            producer_remaining: 2,
            saved_count: 1,
            checking: true,
            waiting: 0,
            recovered: true,
        };
        s = advance_combo(
            s,
            Scenario::Basic,
            2,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 0,
                producer_remaining: 2,
                saved_count: 1,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            3,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 7,
                misses: 0,
                producer_remaining: 2,
                saved_count: 1,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            2,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 0,
                producer_remaining: 2,
                saved_count: 1,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            3,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 9,
                misses: 0,
                producer_remaining: 2,
                saved_count: 1,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            1,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 10,
                misses: 0,
                producer_remaining: 2,
                saved_count: 1,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 11,
                misses: 0,
                producer_remaining: 2,
                saved_count: 1,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            3,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 12,
                misses: 0,
                producer_remaining: 2,
                saved_count: 1,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            1,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 13,
                misses: 0,
                producer_remaining: 2,
                saved_count: 1,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
    }
    {
        let mut s = ComboState {
            count: 10,
            misses: 3,
            producer_remaining: 0,
            saved_count: 7,
            checking: false,
            waiting: 1,
            recovered: false,
        };
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            2,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 10,
                misses: 0,
                producer_remaining: 0,
                saved_count: 7,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            1,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 10,
                misses: 0,
                producer_remaining: 0,
                saved_count: 7,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            2,
            0,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 10,
                misses: 0,
                producer_remaining: 0,
                saved_count: 7,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            1,
            2,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 10,
                misses: 0,
                producer_remaining: 0,
                saved_count: 7,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            1,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 10,
                misses: 0,
                producer_remaining: 0,
                saved_count: 7,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            2,
            0,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 10,
                misses: 0,
                producer_remaining: 0,
                saved_count: 7,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            2,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 10,
                misses: 0,
                producer_remaining: 0,
                saved_count: 7,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            1,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 10,
                misses: 0,
                producer_remaining: 0,
                saved_count: 7,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 3,
            misses: 3,
            producer_remaining: 2,
            saved_count: 7,
            checking: true,
            waiting: 0,
            recovered: true,
        };
        s = advance_combo(
            s,
            Scenario::Basic,
            3,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 4,
                misses: 0,
                producer_remaining: 2,
                saved_count: 7,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            1,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 5,
                misses: 0,
                producer_remaining: 2,
                saved_count: 7,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            0,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 5,
                misses: 1,
                producer_remaining: 2,
                saved_count: 7,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            1,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 0,
                producer_remaining: 2,
                saved_count: 7,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            2,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 7,
                misses: 0,
                producer_remaining: 2,
                saved_count: 7,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            2,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 0,
                producer_remaining: 2,
                saved_count: 7,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 9,
                misses: 0,
                producer_remaining: 2,
                saved_count: 7,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            0,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 9,
                misses: 1,
                producer_remaining: 2,
                saved_count: 7,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
    }
    {
        let mut s = ComboState {
            count: 4,
            misses: 2,
            producer_remaining: 1,
            saved_count: 3,
            checking: false,
            waiting: 1,
            recovered: false,
        };
        s = advance_combo(
            s,
            Scenario::Basic,
            1,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 5,
                misses: 0,
                producer_remaining: 1,
                saved_count: 3,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 0,
                producer_remaining: 1,
                saved_count: 3,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            0,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 1,
                producer_remaining: 1,
                saved_count: 3,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 7,
                misses: 0,
                producer_remaining: 1,
                saved_count: 3,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            3,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 0,
                producer_remaining: 1,
                saved_count: 3,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 9,
                misses: 0,
                producer_remaining: 1,
                saved_count: 3,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            0,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 9,
                misses: 1,
                producer_remaining: 1,
                saved_count: 3,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 10,
                misses: 0,
                producer_remaining: 1,
                saved_count: 3,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 5,
            misses: 2,
            producer_remaining: 0,
            saved_count: 2,
            checking: true,
            waiting: 0,
            recovered: true,
        };
        s = advance_combo(
            s,
            Scenario::Basic,
            1,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 0,
                producer_remaining: 0,
                saved_count: 2,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            0,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 1,
                producer_remaining: 0,
                saved_count: 2,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            0,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 2,
                producer_remaining: 0,
                saved_count: 2,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 7,
                misses: 0,
                producer_remaining: 0,
                saved_count: 2,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            0,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 7,
                misses: 1,
                producer_remaining: 0,
                saved_count: 2,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            1,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 0,
                producer_remaining: 0,
                saved_count: 2,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            0,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 1,
                producer_remaining: 0,
                saved_count: 2,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            3,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 9,
                misses: 0,
                producer_remaining: 0,
                saved_count: 2,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
    }
    {
        let mut s = ComboState {
            count: 2,
            misses: 4,
            producer_remaining: 0,
            saved_count: 3,
            checking: false,
            waiting: 1,
            recovered: false,
        };
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 3,
                misses: 0,
                producer_remaining: 0,
                saved_count: 3,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            3,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 4,
                misses: 0,
                producer_remaining: 0,
                saved_count: 3,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 5,
                misses: 0,
                producer_remaining: 0,
                saved_count: 3,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            2,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 0,
                producer_remaining: 0,
                saved_count: 3,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            2,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 7,
                misses: 0,
                producer_remaining: 0,
                saved_count: 3,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            2,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 0,
                producer_remaining: 0,
                saved_count: 3,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            0,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 1,
                producer_remaining: 0,
                saved_count: 3,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            1,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 9,
                misses: 0,
                producer_remaining: 0,
                saved_count: 3,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 3,
            misses: 4,
            producer_remaining: 2,
            saved_count: 3,
            checking: true,
            waiting: 0,
            recovered: true,
        };
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 4,
                misses: 0,
                producer_remaining: 2,
                saved_count: 3,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            1,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 5,
                misses: 0,
                producer_remaining: 2,
                saved_count: 3,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 0,
                producer_remaining: 2,
                saved_count: 3,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            1,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 7,
                misses: 0,
                producer_remaining: 2,
                saved_count: 3,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 0,
                producer_remaining: 2,
                saved_count: 3,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            0,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 1,
                producer_remaining: 2,
                saved_count: 3,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 9,
                misses: 0,
                producer_remaining: 2,
                saved_count: 3,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 10,
                misses: 0,
                producer_remaining: 2,
                saved_count: 3,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
    }
    {
        let mut s = ComboState {
            count: 8,
            misses: 2,
            producer_remaining: 1,
            saved_count: 2,
            checking: false,
            waiting: 1,
            recovered: false,
        };
        s = advance_combo(
            s,
            Scenario::Basic,
            2,
            2,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 0,
                producer_remaining: 1,
                saved_count: 2,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            0,
            1,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 1,
                producer_remaining: 1,
                saved_count: 2,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            2,
            0,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 0,
                producer_remaining: 1,
                saved_count: 2,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            2,
            2,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 0,
                producer_remaining: 1,
                saved_count: 2,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            3,
            1,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 0,
                producer_remaining: 1,
                saved_count: 2,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            0,
            0,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 1,
                producer_remaining: 1,
                saved_count: 2,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            2,
            2,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 0,
                producer_remaining: 1,
                saved_count: 2,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            0,
            1,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 1,
                producer_remaining: 1,
                saved_count: 2,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 1,
            misses: 4,
            producer_remaining: 0,
            saved_count: 4,
            checking: true,
            waiting: 0,
            recovered: true,
        };
        s = advance_combo(
            s,
            Scenario::Basic,
            3,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 2,
                misses: 0,
                producer_remaining: 0,
                saved_count: 4,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 3,
                misses: 0,
                producer_remaining: 0,
                saved_count: 4,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 4,
                misses: 0,
                producer_remaining: 0,
                saved_count: 4,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            1,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 5,
                misses: 0,
                producer_remaining: 0,
                saved_count: 4,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            2,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 0,
                producer_remaining: 0,
                saved_count: 4,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 7,
                misses: 0,
                producer_remaining: 0,
                saved_count: 4,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            2,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 0,
                producer_remaining: 0,
                saved_count: 4,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 9,
                misses: 0,
                producer_remaining: 0,
                saved_count: 4,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
    }
    {
        let mut s = ComboState {
            count: 2,
            misses: 2,
            producer_remaining: 2,
            saved_count: 0,
            checking: false,
            waiting: 1,
            recovered: false,
        };
        s = advance_combo(
            s,
            Scenario::Basic,
            0,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 0,
                misses: 3,
                producer_remaining: 2,
                saved_count: 0,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            3,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 1,
                misses: 0,
                producer_remaining: 2,
                saved_count: 0,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            0,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 1,
                misses: 1,
                producer_remaining: 2,
                saved_count: 0,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 2,
                misses: 0,
                producer_remaining: 2,
                saved_count: 0,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            2,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 3,
                misses: 0,
                producer_remaining: 2,
                saved_count: 0,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            0,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 3,
                misses: 1,
                producer_remaining: 2,
                saved_count: 0,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            1,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 4,
                misses: 0,
                producer_remaining: 2,
                saved_count: 0,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            3,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 5,
                misses: 0,
                producer_remaining: 2,
                saved_count: 0,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 11,
            misses: 0,
            producer_remaining: 2,
            saved_count: 5,
            checking: true,
            waiting: 0,
            recovered: true,
        };
        s = advance_combo(
            s,
            Scenario::Basic,
            3,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 12,
                misses: 0,
                producer_remaining: 2,
                saved_count: 5,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 13,
                misses: 0,
                producer_remaining: 2,
                saved_count: 5,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 14,
                misses: 0,
                producer_remaining: 2,
                saved_count: 5,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            2,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 15,
                misses: 0,
                producer_remaining: 2,
                saved_count: 5,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            0,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 15,
                misses: 1,
                producer_remaining: 2,
                saved_count: 5,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 16,
                misses: 0,
                producer_remaining: 2,
                saved_count: 5,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            1,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 17,
                misses: 0,
                producer_remaining: 2,
                saved_count: 5,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 18,
                misses: 0,
                producer_remaining: 2,
                saved_count: 5,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
    }
    {
        let mut s = ComboState {
            count: 8,
            misses: 2,
            producer_remaining: 0,
            saved_count: 5,
            checking: false,
            waiting: 1,
            recovered: false,
        };
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 9,
                misses: 0,
                producer_remaining: 0,
                saved_count: 5,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            1,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 10,
                misses: 0,
                producer_remaining: 0,
                saved_count: 5,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            2,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 11,
                misses: 0,
                producer_remaining: 0,
                saved_count: 5,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            2,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 12,
                misses: 0,
                producer_remaining: 0,
                saved_count: 5,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 13,
                misses: 0,
                producer_remaining: 0,
                saved_count: 5,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            1,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 14,
                misses: 0,
                producer_remaining: 0,
                saved_count: 5,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            1,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 15,
                misses: 0,
                producer_remaining: 0,
                saved_count: 5,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            1,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 16,
                misses: 0,
                producer_remaining: 0,
                saved_count: 5,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 5,
            misses: 4,
            producer_remaining: 2,
            saved_count: 3,
            checking: true,
            waiting: 0,
            recovered: true,
        };
        s = advance_combo(
            s,
            Scenario::Basic,
            1,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 0,
                producer_remaining: 2,
                saved_count: 3,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 7,
                misses: 0,
                producer_remaining: 2,
                saved_count: 3,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            3,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 0,
                producer_remaining: 2,
                saved_count: 3,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            3,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 9,
                misses: 0,
                producer_remaining: 2,
                saved_count: 3,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            4,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 10,
                misses: 0,
                producer_remaining: 2,
                saved_count: 3,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            1,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 11,
                misses: 0,
                producer_remaining: 2,
                saved_count: 3,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            1,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 12,
                misses: 0,
                producer_remaining: 2,
                saved_count: 3,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Basic,
            2,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 13,
                misses: 0,
                producer_remaining: 2,
                saved_count: 3,
                checking: true,
                waiting: 0,
                recovered: true
            }
        );
    }
    {
        let mut s = ComboState {
            count: 6,
            misses: 2,
            producer_remaining: 1,
            saved_count: 6,
            checking: false,
            waiting: 1,
            recovered: false,
        };
        s = advance_combo(
            s,
            Scenario::Continuation,
            2,
            2,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 0,
                producer_remaining: 2,
                saved_count: 6,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            0,
            1,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 1,
                producer_remaining: 2,
                saved_count: 6,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            0,
            0,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 2,
                producer_remaining: 2,
                saved_count: 6,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            0,
            2,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 3,
                producer_remaining: 2,
                saved_count: 6,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            3,
            1,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 0,
                producer_remaining: 1,
                saved_count: 6,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            3,
            0,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 0,
                producer_remaining: 0,
                saved_count: 6,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            4,
            2,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 0,
                producer_remaining: 2,
                saved_count: 6,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            1,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 0,
                producer_remaining: 1,
                saved_count: 6,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 11,
            misses: 2,
            producer_remaining: 0,
            saved_count: 6,
            checking: true,
            waiting: 0,
            recovered: true,
        };
        s = advance_combo(
            s,
            Scenario::Continuation,
            3,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 12,
                misses: 0,
                producer_remaining: 2,
                saved_count: 0,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            3,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 13,
                misses: 0,
                producer_remaining: 1,
                saved_count: 13,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            4,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 14,
                misses: 0,
                producer_remaining: 0,
                saved_count: 14,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 15,
                misses: 0,
                producer_remaining: 2,
                saved_count: 15,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            3,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 16,
                misses: 0,
                producer_remaining: 1,
                saved_count: 16,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            4,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 17,
                misses: 0,
                producer_remaining: 0,
                saved_count: 17,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            2,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 18,
                misses: 0,
                producer_remaining: 2,
                saved_count: 18,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            2,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 19,
                misses: 0,
                producer_remaining: 1,
                saved_count: 19,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 4,
            misses: 3,
            producer_remaining: 2,
            saved_count: 6,
            checking: false,
            waiting: 1,
            recovered: false,
        };
        s = advance_combo(
            s,
            Scenario::Continuation,
            3,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 5,
                misses: 0,
                producer_remaining: 2,
                saved_count: 5,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            2,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 0,
                producer_remaining: 1,
                saved_count: 6,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            4,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 7,
                misses: 0,
                producer_remaining: 0,
                saved_count: 7,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            0,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 7,
                misses: 1,
                producer_remaining: 0,
                saved_count: 7,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            4,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 0,
                producer_remaining: 1,
                saved_count: 8,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            3,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 9,
                misses: 0,
                producer_remaining: 0,
                saved_count: 9,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            3,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 10,
                misses: 0,
                producer_remaining: 2,
                saved_count: 10,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            0,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 10,
                misses: 1,
                producer_remaining: 2,
                saved_count: 10,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 1,
            misses: 3,
            producer_remaining: 0,
            saved_count: 4,
            checking: true,
            waiting: 0,
            recovered: true,
        };
        s = advance_combo(
            s,
            Scenario::Continuation,
            4,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 2,
                misses: 0,
                producer_remaining: 2,
                saved_count: 0,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 3,
                misses: 0,
                producer_remaining: 1,
                saved_count: 3,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            2,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 4,
                misses: 0,
                producer_remaining: 0,
                saved_count: 4,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 5,
                misses: 0,
                producer_remaining: 2,
                saved_count: 5,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            4,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 0,
                producer_remaining: 1,
                saved_count: 6,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            2,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 7,
                misses: 0,
                producer_remaining: 0,
                saved_count: 7,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 0,
                producer_remaining: 2,
                saved_count: 8,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            0,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 1,
                producer_remaining: 2,
                saved_count: 8,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 2,
            misses: 1,
            producer_remaining: 0,
            saved_count: 5,
            checking: false,
            waiting: 1,
            recovered: false,
        };
        s = advance_combo(
            s,
            Scenario::Continuation,
            2,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 3,
                misses: 0,
                producer_remaining: 2,
                saved_count: 3,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            4,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 4,
                misses: 0,
                producer_remaining: 1,
                saved_count: 4,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            0,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 4,
                misses: 1,
                producer_remaining: 1,
                saved_count: 4,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            3,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 5,
                misses: 0,
                producer_remaining: 2,
                saved_count: 5,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 0,
                producer_remaining: 1,
                saved_count: 6,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 7,
                misses: 0,
                producer_remaining: 0,
                saved_count: 7,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            3,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 0,
                producer_remaining: 2,
                saved_count: 8,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            0,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 1,
                producer_remaining: 2,
                saved_count: 8,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 3,
            misses: 4,
            producer_remaining: 1,
            saved_count: 6,
            checking: true,
            waiting: 0,
            recovered: true,
        };
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 4,
                misses: 0,
                producer_remaining: 2,
                saved_count: 0,
                checking: false,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            0,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 4,
                misses: 1,
                producer_remaining: 2,
                saved_count: 4,
                checking: false,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            0,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 4,
                misses: 2,
                producer_remaining: 2,
                saved_count: 4,
                checking: false,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            4,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 5,
                misses: 0,
                producer_remaining: 2,
                saved_count: 5,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 0,
                producer_remaining: 1,
                saved_count: 6,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            3,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 7,
                misses: 0,
                producer_remaining: 0,
                saved_count: 7,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            0,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 7,
                misses: 1,
                producer_remaining: 0,
                saved_count: 7,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            2,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 0,
                producer_remaining: 1,
                saved_count: 8,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 4,
            misses: 3,
            producer_remaining: 2,
            saved_count: 5,
            checking: false,
            waiting: 1,
            recovered: false,
        };
        s = advance_combo(
            s,
            Scenario::Continuation,
            0,
            2,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 4,
                misses: 4,
                producer_remaining: 2,
                saved_count: 4,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            1,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 4,
                misses: 0,
                producer_remaining: 1,
                saved_count: 4,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            2,
            0,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 4,
                misses: 0,
                producer_remaining: 0,
                saved_count: 4,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            4,
            2,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 4,
                misses: 0,
                producer_remaining: 2,
                saved_count: 4,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            4,
            1,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 4,
                misses: 0,
                producer_remaining: 1,
                saved_count: 4,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            0,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 4,
                misses: 0,
                producer_remaining: 0,
                saved_count: 4,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            0,
            2,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 4,
                misses: 1,
                producer_remaining: 0,
                saved_count: 4,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            0,
            1,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 4,
                misses: 2,
                producer_remaining: 0,
                saved_count: 4,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 1,
            misses: 1,
            producer_remaining: 2,
            saved_count: 1,
            checking: true,
            waiting: 0,
            recovered: true,
        };
        s = advance_combo(
            s,
            Scenario::Continuation,
            2,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 2,
                misses: 0,
                producer_remaining: 2,
                saved_count: 0,
                checking: false,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 3,
                misses: 0,
                producer_remaining: 1,
                saved_count: 3,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            2,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 4,
                misses: 0,
                producer_remaining: 0,
                saved_count: 4,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            0,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 4,
                misses: 1,
                producer_remaining: 0,
                saved_count: 4,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            2,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 5,
                misses: 0,
                producer_remaining: 1,
                saved_count: 5,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 0,
                producer_remaining: 0,
                saved_count: 6,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            0,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 1,
                producer_remaining: 0,
                saved_count: 6,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            0,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 2,
                producer_remaining: 0,
                saved_count: 6,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 10,
            misses: 1,
            producer_remaining: 1,
            saved_count: 0,
            checking: false,
            waiting: 1,
            recovered: false,
        };
        s = advance_combo(
            s,
            Scenario::Continuation,
            3,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 11,
                misses: 0,
                producer_remaining: 2,
                saved_count: 11,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 12,
                misses: 0,
                producer_remaining: 1,
                saved_count: 12,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 13,
                misses: 0,
                producer_remaining: 0,
                saved_count: 13,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            4,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 14,
                misses: 0,
                producer_remaining: 2,
                saved_count: 14,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            2,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 15,
                misses: 0,
                producer_remaining: 1,
                saved_count: 15,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            3,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 16,
                misses: 0,
                producer_remaining: 0,
                saved_count: 16,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 17,
                misses: 0,
                producer_remaining: 2,
                saved_count: 17,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            0,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 17,
                misses: 1,
                producer_remaining: 2,
                saved_count: 17,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 7,
            misses: 0,
            producer_remaining: 1,
            saved_count: 1,
            checking: true,
            waiting: 0,
            recovered: true,
        };
        s = advance_combo(
            s,
            Scenario::Continuation,
            3,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 0,
                producer_remaining: 2,
                saved_count: 0,
                checking: false,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            2,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 9,
                misses: 0,
                producer_remaining: 1,
                saved_count: 9,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            4,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 10,
                misses: 0,
                producer_remaining: 0,
                saved_count: 10,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            2,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 11,
                misses: 0,
                producer_remaining: 2,
                saved_count: 11,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 12,
                misses: 0,
                producer_remaining: 1,
                saved_count: 12,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 13,
                misses: 0,
                producer_remaining: 0,
                saved_count: 13,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            0,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 13,
                misses: 1,
                producer_remaining: 0,
                saved_count: 13,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            3,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 14,
                misses: 0,
                producer_remaining: 1,
                saved_count: 14,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 8,
            misses: 3,
            producer_remaining: 2,
            saved_count: 1,
            checking: false,
            waiting: 1,
            recovered: false,
        };
        s = advance_combo(
            s,
            Scenario::Continuation,
            2,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 9,
                misses: 0,
                producer_remaining: 2,
                saved_count: 9,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 10,
                misses: 0,
                producer_remaining: 1,
                saved_count: 10,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            0,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 10,
                misses: 1,
                producer_remaining: 1,
                saved_count: 10,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            3,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 11,
                misses: 0,
                producer_remaining: 2,
                saved_count: 11,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            4,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 12,
                misses: 0,
                producer_remaining: 1,
                saved_count: 12,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            3,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 13,
                misses: 0,
                producer_remaining: 0,
                saved_count: 13,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 14,
                misses: 0,
                producer_remaining: 2,
                saved_count: 14,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            3,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 15,
                misses: 0,
                producer_remaining: 1,
                saved_count: 15,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 5,
            misses: 4,
            producer_remaining: 2,
            saved_count: 2,
            checking: true,
            waiting: 0,
            recovered: true,
        };
        s = advance_combo(
            s,
            Scenario::Continuation,
            0,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 0,
                misses: 5,
                producer_remaining: 2,
                saved_count: 0,
                checking: false,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            4,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 1,
                misses: 0,
                producer_remaining: 1,
                saved_count: 1,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            3,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 2,
                misses: 0,
                producer_remaining: 0,
                saved_count: 2,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            4,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 3,
                misses: 0,
                producer_remaining: 2,
                saved_count: 3,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 4,
                misses: 0,
                producer_remaining: 1,
                saved_count: 4,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            0,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 4,
                misses: 1,
                producer_remaining: 1,
                saved_count: 4,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            2,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 5,
                misses: 0,
                producer_remaining: 2,
                saved_count: 5,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 0,
                producer_remaining: 1,
                saved_count: 6,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 2,
            misses: 2,
            producer_remaining: 0,
            saved_count: 7,
            checking: false,
            waiting: 1,
            recovered: false,
        };
        s = advance_combo(
            s,
            Scenario::Continuation,
            0,
            2,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 2,
                misses: 3,
                producer_remaining: 0,
                saved_count: 2,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            3,
            1,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 2,
                misses: 0,
                producer_remaining: 1,
                saved_count: 2,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            3,
            0,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 2,
                misses: 0,
                producer_remaining: 0,
                saved_count: 2,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            4,
            2,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 2,
                misses: 0,
                producer_remaining: 2,
                saved_count: 2,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            4,
            1,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 2,
                misses: 0,
                producer_remaining: 1,
                saved_count: 2,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            0,
            0,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 2,
                misses: 1,
                producer_remaining: 1,
                saved_count: 2,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            2,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 2,
                misses: 0,
                producer_remaining: 2,
                saved_count: 2,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            2,
            1,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 2,
                misses: 0,
                producer_remaining: 1,
                saved_count: 2,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 3,
            misses: 1,
            producer_remaining: 0,
            saved_count: 4,
            checking: true,
            waiting: 0,
            recovered: true,
        };
        s = advance_combo(
            s,
            Scenario::Continuation,
            3,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 4,
                misses: 0,
                producer_remaining: 2,
                saved_count: 0,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            2,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 5,
                misses: 0,
                producer_remaining: 1,
                saved_count: 5,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            2,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 0,
                producer_remaining: 0,
                saved_count: 6,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 7,
                misses: 0,
                producer_remaining: 2,
                saved_count: 7,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            0,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 7,
                misses: 1,
                producer_remaining: 2,
                saved_count: 7,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            3,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 0,
                producer_remaining: 0,
                saved_count: 8,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            3,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 9,
                misses: 0,
                producer_remaining: 2,
                saved_count: 9,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            3,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 10,
                misses: 0,
                producer_remaining: 1,
                saved_count: 10,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 8,
            misses: 1,
            producer_remaining: 1,
            saved_count: 6,
            checking: false,
            waiting: 1,
            recovered: false,
        };
        s = advance_combo(
            s,
            Scenario::Continuation,
            2,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 9,
                misses: 0,
                producer_remaining: 2,
                saved_count: 9,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            2,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 10,
                misses: 0,
                producer_remaining: 1,
                saved_count: 10,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 11,
                misses: 0,
                producer_remaining: 0,
                saved_count: 11,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            0,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 11,
                misses: 1,
                producer_remaining: 0,
                saved_count: 11,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 12,
                misses: 0,
                producer_remaining: 1,
                saved_count: 12,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            4,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 13,
                misses: 0,
                producer_remaining: 0,
                saved_count: 13,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            4,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 14,
                misses: 0,
                producer_remaining: 2,
                saved_count: 14,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 15,
                misses: 0,
                producer_remaining: 1,
                saved_count: 15,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 1,
            misses: 4,
            producer_remaining: 1,
            saved_count: 5,
            checking: true,
            waiting: 0,
            recovered: true,
        };
        s = advance_combo(
            s,
            Scenario::Continuation,
            4,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 2,
                misses: 0,
                producer_remaining: 2,
                saved_count: 0,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 3,
                misses: 0,
                producer_remaining: 1,
                saved_count: 3,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            3,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 4,
                misses: 0,
                producer_remaining: 0,
                saved_count: 4,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            3,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 5,
                misses: 0,
                producer_remaining: 2,
                saved_count: 5,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            4,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 0,
                producer_remaining: 1,
                saved_count: 6,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            0,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 1,
                producer_remaining: 1,
                saved_count: 6,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 7,
                misses: 0,
                producer_remaining: 2,
                saved_count: 7,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 0,
                producer_remaining: 1,
                saved_count: 8,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 10,
            misses: 0,
            producer_remaining: 0,
            saved_count: 0,
            checking: false,
            waiting: 1,
            recovered: false,
        };
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 11,
                misses: 0,
                producer_remaining: 2,
                saved_count: 11,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            0,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 11,
                misses: 1,
                producer_remaining: 2,
                saved_count: 11,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            4,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 12,
                misses: 0,
                producer_remaining: 0,
                saved_count: 12,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            2,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 13,
                misses: 0,
                producer_remaining: 2,
                saved_count: 13,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            3,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 14,
                misses: 0,
                producer_remaining: 1,
                saved_count: 14,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            0,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 14,
                misses: 1,
                producer_remaining: 1,
                saved_count: 14,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            2,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 15,
                misses: 0,
                producer_remaining: 2,
                saved_count: 15,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 16,
                misses: 0,
                producer_remaining: 1,
                saved_count: 16,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 11,
            misses: 4,
            producer_remaining: 1,
            saved_count: 5,
            checking: true,
            waiting: 0,
            recovered: true,
        };
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 12,
                misses: 0,
                producer_remaining: 2,
                saved_count: 0,
                checking: false,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            0,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 12,
                misses: 1,
                producer_remaining: 2,
                saved_count: 12,
                checking: false,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            4,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 13,
                misses: 0,
                producer_remaining: 0,
                saved_count: 13,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 14,
                misses: 0,
                producer_remaining: 2,
                saved_count: 14,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 15,
                misses: 0,
                producer_remaining: 1,
                saved_count: 15,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 16,
                misses: 0,
                producer_remaining: 0,
                saved_count: 16,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 17,
                misses: 0,
                producer_remaining: 2,
                saved_count: 17,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            2,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 18,
                misses: 0,
                producer_remaining: 1,
                saved_count: 18,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 8,
            misses: 2,
            producer_remaining: 0,
            saved_count: 8,
            checking: false,
            waiting: 1,
            recovered: false,
        };
        s = advance_combo(
            s,
            Scenario::Continuation,
            0,
            2,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 3,
                producer_remaining: 0,
                saved_count: 8,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            0,
            1,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 4,
                producer_remaining: 0,
                saved_count: 8,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            0,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 0,
                producer_remaining: 0,
                saved_count: 8,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            2,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 0,
                producer_remaining: 2,
                saved_count: 8,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            2,
            1,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 0,
                producer_remaining: 1,
                saved_count: 8,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            0,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 0,
                producer_remaining: 0,
                saved_count: 8,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            2,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 0,
                producer_remaining: 2,
                saved_count: 8,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            1,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 0,
                producer_remaining: 1,
                saved_count: 8,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 5,
            misses: 0,
            producer_remaining: 2,
            saved_count: 4,
            checking: true,
            waiting: 0,
            recovered: true,
        };
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 0,
                producer_remaining: 2,
                saved_count: 0,
                checking: false,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 7,
                misses: 0,
                producer_remaining: 1,
                saved_count: 7,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            4,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 0,
                producer_remaining: 0,
                saved_count: 8,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 9,
                misses: 0,
                producer_remaining: 2,
                saved_count: 9,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            3,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 10,
                misses: 0,
                producer_remaining: 1,
                saved_count: 10,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            4,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 11,
                misses: 0,
                producer_remaining: 0,
                saved_count: 11,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            3,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 12,
                misses: 0,
                producer_remaining: 2,
                saved_count: 12,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 13,
                misses: 0,
                producer_remaining: 1,
                saved_count: 13,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 2,
            misses: 4,
            producer_remaining: 1,
            saved_count: 6,
            checking: false,
            waiting: 1,
            recovered: false,
        };
        s = advance_combo(
            s,
            Scenario::Continuation,
            3,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 3,
                misses: 0,
                producer_remaining: 2,
                saved_count: 3,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 4,
                misses: 0,
                producer_remaining: 1,
                saved_count: 4,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            2,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 5,
                misses: 0,
                producer_remaining: 0,
                saved_count: 5,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            3,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 0,
                producer_remaining: 2,
                saved_count: 6,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            3,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 7,
                misses: 0,
                producer_remaining: 1,
                saved_count: 7,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            2,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 0,
                producer_remaining: 0,
                saved_count: 8,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            2,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 9,
                misses: 0,
                producer_remaining: 2,
                saved_count: 9,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 10,
                misses: 0,
                producer_remaining: 1,
                saved_count: 10,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 11,
            misses: 1,
            producer_remaining: 0,
            saved_count: 0,
            checking: true,
            waiting: 0,
            recovered: true,
        };
        s = advance_combo(
            s,
            Scenario::Continuation,
            0,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 11,
                misses: 2,
                producer_remaining: 0,
                saved_count: 0,
                checking: false,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            0,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 0,
                misses: 3,
                producer_remaining: 0,
                saved_count: 0,
                checking: false,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 1,
                misses: 0,
                producer_remaining: 0,
                saved_count: 1,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            2,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 2,
                misses: 0,
                producer_remaining: 2,
                saved_count: 2,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            3,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 3,
                misses: 0,
                producer_remaining: 1,
                saved_count: 3,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            4,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 4,
                misses: 0,
                producer_remaining: 0,
                saved_count: 4,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            3,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 5,
                misses: 0,
                producer_remaining: 2,
                saved_count: 5,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            3,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 0,
                producer_remaining: 1,
                saved_count: 6,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 8,
            misses: 1,
            producer_remaining: 2,
            saved_count: 1,
            checking: false,
            waiting: 1,
            recovered: false,
        };
        s = advance_combo(
            s,
            Scenario::Continuation,
            2,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 9,
                misses: 0,
                producer_remaining: 2,
                saved_count: 9,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 10,
                misses: 0,
                producer_remaining: 1,
                saved_count: 10,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            2,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 11,
                misses: 0,
                producer_remaining: 0,
                saved_count: 11,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            3,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 12,
                misses: 0,
                producer_remaining: 2,
                saved_count: 12,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            3,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 13,
                misses: 0,
                producer_remaining: 1,
                saved_count: 13,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 14,
                misses: 0,
                producer_remaining: 0,
                saved_count: 14,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            2,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 15,
                misses: 0,
                producer_remaining: 2,
                saved_count: 15,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 16,
                misses: 0,
                producer_remaining: 1,
                saved_count: 16,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 9,
            misses: 3,
            producer_remaining: 1,
            saved_count: 1,
            checking: true,
            waiting: 0,
            recovered: true,
        };
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 10,
                misses: 0,
                producer_remaining: 2,
                saved_count: 0,
                checking: false,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            2,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 11,
                misses: 0,
                producer_remaining: 1,
                saved_count: 11,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            4,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 12,
                misses: 0,
                producer_remaining: 0,
                saved_count: 12,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            4,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 13,
                misses: 0,
                producer_remaining: 2,
                saved_count: 13,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            4,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 14,
                misses: 0,
                producer_remaining: 1,
                saved_count: 14,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            4,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 15,
                misses: 0,
                producer_remaining: 0,
                saved_count: 15,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            2,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 16,
                misses: 0,
                producer_remaining: 2,
                saved_count: 16,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::Continuation,
            1,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 17,
                misses: 0,
                producer_remaining: 1,
                saved_count: 17,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 10,
            misses: 0,
            producer_remaining: 2,
            saved_count: 6,
            checking: false,
            waiting: 1,
            recovered: false,
        };
        s = advance_combo(
            s,
            Scenario::MultiLine,
            3,
            2,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 10,
                misses: 0,
                producer_remaining: 2,
                saved_count: 10,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            2,
            1,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 10,
                misses: 0,
                producer_remaining: 1,
                saved_count: 10,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            4,
            0,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 10,
                misses: 0,
                producer_remaining: 0,
                saved_count: 10,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            4,
            2,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 10,
                misses: 0,
                producer_remaining: 2,
                saved_count: 10,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            4,
            1,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 10,
                misses: 0,
                producer_remaining: 1,
                saved_count: 10,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            1,
            0,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 10,
                misses: 0,
                producer_remaining: 0,
                saved_count: 10,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            3,
            2,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 10,
                misses: 0,
                producer_remaining: 2,
                saved_count: 10,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            2,
            1,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 10,
                misses: 0,
                producer_remaining: 1,
                saved_count: 10,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 11,
            misses: 1,
            producer_remaining: 0,
            saved_count: 1,
            checking: true,
            waiting: 0,
            recovered: true,
        };
        s = advance_combo(
            s,
            Scenario::MultiLine,
            1,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 12,
                misses: 0,
                producer_remaining: 2,
                saved_count: 0,
                checking: false,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            2,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 14,
                misses: 0,
                producer_remaining: 1,
                saved_count: 13,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            1,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 15,
                misses: 0,
                producer_remaining: 0,
                saved_count: 15,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            3,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 18,
                misses: 0,
                producer_remaining: 2,
                saved_count: 16,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            1,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 19,
                misses: 0,
                producer_remaining: 1,
                saved_count: 19,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            3,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 22,
                misses: 0,
                producer_remaining: 0,
                saved_count: 20,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            1,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 23,
                misses: 0,
                producer_remaining: 2,
                saved_count: 23,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            4,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 27,
                misses: 0,
                producer_remaining: 1,
                saved_count: 24,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 8,
            misses: 2,
            producer_remaining: 2,
            saved_count: 7,
            checking: false,
            waiting: 1,
            recovered: false,
        };
        s = advance_combo(
            s,
            Scenario::MultiLine,
            1,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 9,
                misses: 0,
                producer_remaining: 2,
                saved_count: 9,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            4,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 13,
                misses: 0,
                producer_remaining: 1,
                saved_count: 10,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            1,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 14,
                misses: 0,
                producer_remaining: 0,
                saved_count: 14,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            2,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 16,
                misses: 0,
                producer_remaining: 2,
                saved_count: 15,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            2,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 18,
                misses: 0,
                producer_remaining: 1,
                saved_count: 17,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            0,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 18,
                misses: 1,
                producer_remaining: 1,
                saved_count: 17,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            2,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 20,
                misses: 0,
                producer_remaining: 2,
                saved_count: 19,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            1,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 21,
                misses: 0,
                producer_remaining: 1,
                saved_count: 21,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 1,
            misses: 3,
            producer_remaining: 1,
            saved_count: 7,
            checking: true,
            waiting: 0,
            recovered: true,
        };
        s = advance_combo(
            s,
            Scenario::MultiLine,
            3,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 2,
                misses: 0,
                producer_remaining: 2,
                saved_count: 0,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            2,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 4,
                misses: 0,
                producer_remaining: 1,
                saved_count: 3,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            1,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 5,
                misses: 0,
                producer_remaining: 0,
                saved_count: 5,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            1,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 0,
                producer_remaining: 2,
                saved_count: 6,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            2,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 0,
                producer_remaining: 1,
                saved_count: 7,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            2,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 10,
                misses: 0,
                producer_remaining: 0,
                saved_count: 9,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            3,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 13,
                misses: 0,
                producer_remaining: 2,
                saved_count: 11,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            0,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 13,
                misses: 1,
                producer_remaining: 2,
                saved_count: 11,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 2,
            misses: 3,
            producer_remaining: 0,
            saved_count: 0,
            checking: false,
            waiting: 1,
            recovered: false,
        };
        s = advance_combo(
            s,
            Scenario::MultiLine,
            1,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 3,
                misses: 0,
                producer_remaining: 2,
                saved_count: 3,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            1,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 4,
                misses: 0,
                producer_remaining: 1,
                saved_count: 4,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            4,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 0,
                producer_remaining: 0,
                saved_count: 5,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            2,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 10,
                misses: 0,
                producer_remaining: 2,
                saved_count: 9,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            2,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 12,
                misses: 0,
                producer_remaining: 1,
                saved_count: 11,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            4,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 16,
                misses: 0,
                producer_remaining: 0,
                saved_count: 13,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            0,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 16,
                misses: 1,
                producer_remaining: 0,
                saved_count: 13,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            0,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 16,
                misses: 2,
                producer_remaining: 0,
                saved_count: 13,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 3,
            misses: 0,
            producer_remaining: 2,
            saved_count: 0,
            checking: true,
            waiting: 0,
            recovered: true,
        };
        s = advance_combo(
            s,
            Scenario::MultiLine,
            4,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 7,
                misses: 0,
                producer_remaining: 2,
                saved_count: 0,
                checking: false,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            2,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 9,
                misses: 0,
                producer_remaining: 1,
                saved_count: 8,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            1,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 10,
                misses: 0,
                producer_remaining: 0,
                saved_count: 10,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            2,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 12,
                misses: 0,
                producer_remaining: 2,
                saved_count: 11,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            2,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 14,
                misses: 0,
                producer_remaining: 1,
                saved_count: 13,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            3,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 17,
                misses: 0,
                producer_remaining: 0,
                saved_count: 15,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            1,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 18,
                misses: 0,
                producer_remaining: 2,
                saved_count: 18,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            3,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 21,
                misses: 0,
                producer_remaining: 1,
                saved_count: 19,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 4,
            misses: 3,
            producer_remaining: 0,
            saved_count: 6,
            checking: false,
            waiting: 1,
            recovered: false,
        };
        s = advance_combo(
            s,
            Scenario::MultiLine,
            3,
            2,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 4,
                misses: 0,
                producer_remaining: 2,
                saved_count: 4,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            4,
            1,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 4,
                misses: 0,
                producer_remaining: 1,
                saved_count: 4,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            2,
            0,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 4,
                misses: 0,
                producer_remaining: 0,
                saved_count: 4,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            1,
            2,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 4,
                misses: 0,
                producer_remaining: 2,
                saved_count: 4,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            0,
            1,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 4,
                misses: 1,
                producer_remaining: 2,
                saved_count: 4,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            4,
            0,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 4,
                misses: 0,
                producer_remaining: 0,
                saved_count: 4,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            3,
            2,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 4,
                misses: 0,
                producer_remaining: 2,
                saved_count: 4,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            2,
            1,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 4,
                misses: 0,
                producer_remaining: 1,
                saved_count: 4,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 9,
            misses: 0,
            producer_remaining: 0,
            saved_count: 0,
            checking: true,
            waiting: 0,
            recovered: true,
        };
        s = advance_combo(
            s,
            Scenario::MultiLine,
            0,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 9,
                misses: 1,
                producer_remaining: 0,
                saved_count: 0,
                checking: false,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            1,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 10,
                misses: 0,
                producer_remaining: 1,
                saved_count: 10,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            3,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 13,
                misses: 0,
                producer_remaining: 0,
                saved_count: 11,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            4,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 17,
                misses: 0,
                producer_remaining: 2,
                saved_count: 14,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            3,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 20,
                misses: 0,
                producer_remaining: 1,
                saved_count: 18,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            1,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 21,
                misses: 0,
                producer_remaining: 0,
                saved_count: 21,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            2,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 23,
                misses: 0,
                producer_remaining: 2,
                saved_count: 22,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            4,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 27,
                misses: 0,
                producer_remaining: 1,
                saved_count: 24,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 2,
            misses: 1,
            producer_remaining: 0,
            saved_count: 2,
            checking: false,
            waiting: 1,
            recovered: false,
        };
        s = advance_combo(
            s,
            Scenario::MultiLine,
            2,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 4,
                misses: 0,
                producer_remaining: 2,
                saved_count: 3,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            2,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 0,
                producer_remaining: 1,
                saved_count: 5,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            4,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 10,
                misses: 0,
                producer_remaining: 0,
                saved_count: 7,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            3,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 13,
                misses: 0,
                producer_remaining: 2,
                saved_count: 11,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            2,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 15,
                misses: 0,
                producer_remaining: 1,
                saved_count: 14,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            4,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 19,
                misses: 0,
                producer_remaining: 0,
                saved_count: 16,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            4,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 23,
                misses: 0,
                producer_remaining: 2,
                saved_count: 20,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            2,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 25,
                misses: 0,
                producer_remaining: 1,
                saved_count: 24,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 7,
            misses: 4,
            producer_remaining: 0,
            saved_count: 8,
            checking: true,
            waiting: 0,
            recovered: true,
        };
        s = advance_combo(
            s,
            Scenario::MultiLine,
            0,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 0,
                misses: 5,
                producer_remaining: 0,
                saved_count: 0,
                checking: false,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            3,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 1,
                misses: 0,
                producer_remaining: 1,
                saved_count: 1,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            1,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 2,
                misses: 0,
                producer_remaining: 0,
                saved_count: 2,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            3,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 5,
                misses: 0,
                producer_remaining: 2,
                saved_count: 3,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            0,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 5,
                misses: 1,
                producer_remaining: 2,
                saved_count: 3,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            1,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 0,
                producer_remaining: 0,
                saved_count: 6,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            1,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 7,
                misses: 0,
                producer_remaining: 2,
                saved_count: 7,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            2,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 9,
                misses: 0,
                producer_remaining: 1,
                saved_count: 8,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 0,
            misses: 3,
            producer_remaining: 2,
            saved_count: 7,
            checking: false,
            waiting: 1,
            recovered: false,
        };
        s = advance_combo(
            s,
            Scenario::MultiLine,
            1,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 1,
                misses: 0,
                producer_remaining: 2,
                saved_count: 1,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            2,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 3,
                misses: 0,
                producer_remaining: 1,
                saved_count: 2,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            2,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 5,
                misses: 0,
                producer_remaining: 0,
                saved_count: 4,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            2,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 7,
                misses: 0,
                producer_remaining: 2,
                saved_count: 6,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            0,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 7,
                misses: 1,
                producer_remaining: 2,
                saved_count: 6,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            0,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 7,
                misses: 2,
                producer_remaining: 2,
                saved_count: 6,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            1,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 0,
                producer_remaining: 2,
                saved_count: 8,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            2,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 10,
                misses: 0,
                producer_remaining: 1,
                saved_count: 9,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 9,
            misses: 4,
            producer_remaining: 2,
            saved_count: 7,
            checking: true,
            waiting: 0,
            recovered: true,
        };
        s = advance_combo(
            s,
            Scenario::MultiLine,
            1,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 10,
                misses: 0,
                producer_remaining: 2,
                saved_count: 0,
                checking: false,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            1,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 11,
                misses: 0,
                producer_remaining: 1,
                saved_count: 11,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            4,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 15,
                misses: 0,
                producer_remaining: 0,
                saved_count: 12,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            3,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 18,
                misses: 0,
                producer_remaining: 2,
                saved_count: 16,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            4,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 22,
                misses: 0,
                producer_remaining: 1,
                saved_count: 19,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            3,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 25,
                misses: 0,
                producer_remaining: 0,
                saved_count: 23,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            3,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 28,
                misses: 0,
                producer_remaining: 2,
                saved_count: 26,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            4,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 32,
                misses: 0,
                producer_remaining: 1,
                saved_count: 29,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 2,
            misses: 1,
            producer_remaining: 0,
            saved_count: 2,
            checking: false,
            waiting: 1,
            recovered: false,
        };
        s = advance_combo(
            s,
            Scenario::MultiLine,
            3,
            2,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 2,
                misses: 0,
                producer_remaining: 2,
                saved_count: 2,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            0,
            1,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 2,
                misses: 1,
                producer_remaining: 2,
                saved_count: 2,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            2,
            0,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 2,
                misses: 0,
                producer_remaining: 0,
                saved_count: 2,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            2,
            2,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 2,
                misses: 0,
                producer_remaining: 2,
                saved_count: 2,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            0,
            1,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 2,
                misses: 1,
                producer_remaining: 2,
                saved_count: 2,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            2,
            0,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 2,
                misses: 0,
                producer_remaining: 0,
                saved_count: 2,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            4,
            2,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 2,
                misses: 0,
                producer_remaining: 2,
                saved_count: 2,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            2,
            1,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 2,
                misses: 0,
                producer_remaining: 1,
                saved_count: 2,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 3,
            misses: 0,
            producer_remaining: 2,
            saved_count: 5,
            checking: true,
            waiting: 0,
            recovered: true,
        };
        s = advance_combo(
            s,
            Scenario::MultiLine,
            0,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 3,
                misses: 1,
                producer_remaining: 2,
                saved_count: 0,
                checking: false,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            4,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 7,
                misses: 0,
                producer_remaining: 1,
                saved_count: 4,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            2,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 9,
                misses: 0,
                producer_remaining: 0,
                saved_count: 8,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            4,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 13,
                misses: 0,
                producer_remaining: 2,
                saved_count: 10,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            3,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 16,
                misses: 0,
                producer_remaining: 1,
                saved_count: 14,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            0,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 16,
                misses: 1,
                producer_remaining: 1,
                saved_count: 14,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            1,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 17,
                misses: 0,
                producer_remaining: 2,
                saved_count: 17,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            4,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 21,
                misses: 0,
                producer_remaining: 1,
                saved_count: 18,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 4,
            misses: 1,
            producer_remaining: 2,
            saved_count: 3,
            checking: false,
            waiting: 1,
            recovered: false,
        };
        s = advance_combo(
            s,
            Scenario::MultiLine,
            1,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 5,
                misses: 0,
                producer_remaining: 2,
                saved_count: 5,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            0,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 5,
                misses: 1,
                producer_remaining: 2,
                saved_count: 5,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            4,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 9,
                misses: 0,
                producer_remaining: 0,
                saved_count: 6,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            4,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 13,
                misses: 0,
                producer_remaining: 2,
                saved_count: 10,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            3,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 16,
                misses: 0,
                producer_remaining: 1,
                saved_count: 14,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            2,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 18,
                misses: 0,
                producer_remaining: 0,
                saved_count: 17,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            2,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 20,
                misses: 0,
                producer_remaining: 2,
                saved_count: 19,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            3,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 23,
                misses: 0,
                producer_remaining: 1,
                saved_count: 21,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 9,
            misses: 4,
            producer_remaining: 2,
            saved_count: 1,
            checking: true,
            waiting: 0,
            recovered: true,
        };
        s = advance_combo(
            s,
            Scenario::MultiLine,
            2,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 10,
                misses: 0,
                producer_remaining: 2,
                saved_count: 0,
                checking: false,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            3,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 13,
                misses: 0,
                producer_remaining: 1,
                saved_count: 11,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            1,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 14,
                misses: 0,
                producer_remaining: 0,
                saved_count: 14,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            1,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 15,
                misses: 0,
                producer_remaining: 2,
                saved_count: 15,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            0,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 15,
                misses: 1,
                producer_remaining: 2,
                saved_count: 15,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            1,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 16,
                misses: 0,
                producer_remaining: 0,
                saved_count: 16,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            0,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 16,
                misses: 1,
                producer_remaining: 0,
                saved_count: 16,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            1,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 17,
                misses: 0,
                producer_remaining: 1,
                saved_count: 17,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 6,
            misses: 1,
            producer_remaining: 2,
            saved_count: 4,
            checking: false,
            waiting: 1,
            recovered: false,
        };
        s = advance_combo(
            s,
            Scenario::MultiLine,
            0,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 2,
                producer_remaining: 2,
                saved_count: 4,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            3,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 9,
                misses: 0,
                producer_remaining: 1,
                saved_count: 7,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            0,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 9,
                misses: 1,
                producer_remaining: 1,
                saved_count: 7,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            4,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 13,
                misses: 0,
                producer_remaining: 2,
                saved_count: 10,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            3,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 16,
                misses: 0,
                producer_remaining: 1,
                saved_count: 14,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            0,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 16,
                misses: 1,
                producer_remaining: 1,
                saved_count: 14,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            3,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 19,
                misses: 0,
                producer_remaining: 2,
                saved_count: 17,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            2,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 21,
                misses: 0,
                producer_remaining: 1,
                saved_count: 20,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 11,
            misses: 0,
            producer_remaining: 0,
            saved_count: 5,
            checking: true,
            waiting: 0,
            recovered: true,
        };
        s = advance_combo(
            s,
            Scenario::MultiLine,
            1,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 12,
                misses: 0,
                producer_remaining: 2,
                saved_count: 0,
                checking: false,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            1,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 13,
                misses: 0,
                producer_remaining: 1,
                saved_count: 13,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            2,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 15,
                misses: 0,
                producer_remaining: 0,
                saved_count: 14,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            1,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 16,
                misses: 0,
                producer_remaining: 2,
                saved_count: 16,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            0,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 16,
                misses: 1,
                producer_remaining: 2,
                saved_count: 16,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            4,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 20,
                misses: 0,
                producer_remaining: 0,
                saved_count: 17,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            0,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 20,
                misses: 1,
                producer_remaining: 0,
                saved_count: 17,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            4,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 24,
                misses: 0,
                producer_remaining: 1,
                saved_count: 21,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 8,
            misses: 0,
            producer_remaining: 0,
            saved_count: 1,
            checking: false,
            waiting: 1,
            recovered: false,
        };
        s = advance_combo(
            s,
            Scenario::MultiLine,
            0,
            2,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 1,
                producer_remaining: 0,
                saved_count: 1,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            4,
            1,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 0,
                producer_remaining: 1,
                saved_count: 8,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            4,
            0,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 0,
                producer_remaining: 0,
                saved_count: 8,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            0,
            2,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 1,
                producer_remaining: 0,
                saved_count: 8,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            0,
            1,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 2,
                producer_remaining: 0,
                saved_count: 8,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            1,
            0,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 0,
                producer_remaining: 0,
                saved_count: 8,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            1,
            2,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 0,
                producer_remaining: 2,
                saved_count: 8,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            1,
            1,
            RuleContext {
                guide_step: 1,
                guide_finished: false,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 0,
                producer_remaining: 1,
                saved_count: 8,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 5,
            misses: 4,
            producer_remaining: 1,
            saved_count: 1,
            checking: true,
            waiting: 0,
            recovered: true,
        };
        s = advance_combo(
            s,
            Scenario::MultiLine,
            1,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 0,
                producer_remaining: 2,
                saved_count: 0,
                checking: false,
                waiting: 0,
                recovered: true
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            2,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 0,
                producer_remaining: 1,
                saved_count: 7,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            4,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 12,
                misses: 0,
                producer_remaining: 0,
                saved_count: 9,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            1,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 13,
                misses: 0,
                producer_remaining: 2,
                saved_count: 13,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            1,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 14,
                misses: 0,
                producer_remaining: 1,
                saved_count: 14,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            1,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 15,
                misses: 0,
                producer_remaining: 0,
                saved_count: 15,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            4,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 19,
                misses: 0,
                producer_remaining: 2,
                saved_count: 16,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            3,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 22,
                misses: 0,
                producer_remaining: 1,
                saved_count: 20,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 2,
            misses: 2,
            producer_remaining: 2,
            saved_count: 5,
            checking: false,
            waiting: 1,
            recovered: false,
        };
        s = advance_combo(
            s,
            Scenario::MultiLine,
            0,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 2,
                misses: 3,
                producer_remaining: 2,
                saved_count: 5,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            2,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 3,
                misses: 0,
                producer_remaining: 1,
                saved_count: 3,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            2,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 5,
                misses: 0,
                producer_remaining: 0,
                saved_count: 4,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            0,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 5,
                misses: 1,
                producer_remaining: 0,
                saved_count: 4,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            0,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 5,
                misses: 2,
                producer_remaining: 0,
                saved_count: 4,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            3,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 8,
                misses: 0,
                producer_remaining: 0,
                saved_count: 6,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            4,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 12,
                misses: 0,
                producer_remaining: 2,
                saved_count: 9,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            0,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 12,
                misses: 1,
                producer_remaining: 2,
                saved_count: 9,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 7,
            misses: 1,
            producer_remaining: 0,
            saved_count: 7,
            checking: true,
            waiting: 0,
            recovered: true,
        };
        s = advance_combo(
            s,
            Scenario::MultiLine,
            3,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 10,
                misses: 0,
                producer_remaining: 2,
                saved_count: 0,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            1,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 11,
                misses: 0,
                producer_remaining: 1,
                saved_count: 11,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            3,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 14,
                misses: 0,
                producer_remaining: 0,
                saved_count: 12,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            0,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 14,
                misses: 1,
                producer_remaining: 0,
                saved_count: 12,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            3,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 17,
                misses: 0,
                producer_remaining: 1,
                saved_count: 15,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            4,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 21,
                misses: 0,
                producer_remaining: 0,
                saved_count: 18,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            3,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 24,
                misses: 0,
                producer_remaining: 2,
                saved_count: 22,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            2,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 26,
                misses: 0,
                producer_remaining: 1,
                saved_count: 25,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 4,
            misses: 1,
            producer_remaining: 0,
            saved_count: 4,
            checking: false,
            waiting: 1,
            recovered: false,
        };
        s = advance_combo(
            s,
            Scenario::MultiLine,
            2,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 0,
                producer_remaining: 2,
                saved_count: 5,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            0,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 1,
                producer_remaining: 2,
                saved_count: 5,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            4,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 10,
                misses: 0,
                producer_remaining: 0,
                saved_count: 7,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            4,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 14,
                misses: 0,
                producer_remaining: 2,
                saved_count: 11,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            3,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 17,
                misses: 0,
                producer_remaining: 1,
                saved_count: 15,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            4,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 21,
                misses: 0,
                producer_remaining: 0,
                saved_count: 18,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            0,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 21,
                misses: 1,
                producer_remaining: 0,
                saved_count: 18,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            2,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 23,
                misses: 0,
                producer_remaining: 1,
                saved_count: 22,
                checking: false,
                waiting: 1,
                recovered: false
            }
        );
    }
    {
        let mut s = ComboState {
            count: 1,
            misses: 1,
            producer_remaining: 2,
            saved_count: 8,
            checking: true,
            waiting: 0,
            recovered: true,
        };
        s = advance_combo(
            s,
            Scenario::MultiLine,
            2,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 3,
                misses: 0,
                producer_remaining: 2,
                saved_count: 0,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            3,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 0,
                producer_remaining: 1,
                saved_count: 4,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            0,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 6,
                misses: 1,
                producer_remaining: 1,
                saved_count: 4,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            1,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 7,
                misses: 0,
                producer_remaining: 2,
                saved_count: 7,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            3,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 10,
                misses: 0,
                producer_remaining: 1,
                saved_count: 8,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            1,
            0,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 11,
                misses: 0,
                producer_remaining: 0,
                saved_count: 11,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            3,
            2,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 14,
                misses: 0,
                producer_remaining: 2,
                saved_count: 12,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
        s = advance_combo(
            s,
            Scenario::MultiLine,
            4,
            1,
            RuleContext {
                guide_step: 2,
                guide_finished: true,
                all_clear_bonus: 0,
            },
        );
        assert_eq!(
            s,
            ComboState {
                count: 18,
                misses: 0,
                producer_remaining: 1,
                saved_count: 15,
                checking: false,
                waiting: 0,
                recovered: false
            }
        );
    }
}

#[test]
fn board_rules_match_independent_grid_oracle() {
    let mut random = 71u64;
    for _ in 0..120 {
        random = random.wrapping_mul(6364136223846793005).wrapping_add(1);
        let board = Board::from_bits(random);
        for (_, shape) in catalog::catalog() {
            for row in 0..8 {
                for col in 0..8 {
                    let legal = shape.cells().iter().all(|&(r, c)| {
                        row + r < 8
                            && col + c < 8
                            && board.rows()[usize::from(row + r)].as_bytes()[usize::from(col + c)]
                                == b'.'
                    });
                    assert_eq!(board.can_place(&shape, row, col), legal);
                    if !legal {
                        continue;
                    }
                    let mut grid = board.rows().map(|s| s.into_bytes());
                    for &(r, c) in shape.cells() {
                        grid[usize::from(row + r)][usize::from(col + c)] = b'#';
                    }
                    let full_rows: Vec<usize> = (0..8)
                        .filter(|&r| grid[r].iter().all(|&x| x == b'#'))
                        .collect();
                    let full_cols: Vec<usize> = (0..8)
                        .filter(|&c| (0..8).all(|r| grid[r][c] == b'#'))
                        .collect();
                    for (r, line) in grid.iter_mut().enumerate() {
                        for (c, cell) in line.iter_mut().enumerate() {
                            if full_rows.contains(&r) || full_cols.contains(&c) {
                                *cell = b'.';
                            }
                        }
                    }
                    let after = board.place(&shape, row, col).unwrap();
                    assert_eq!(
                        after.board.rows(),
                        grid.map(|s| String::from_utf8(s).unwrap())
                    );
                    assert_eq!(after.lines() as usize, full_rows.len() + full_cols.len());
                }
            }
        }
    }
}
#[test]
fn crossing_clear_and_shapes_with_holes() {
    let board = Board::from_bits((0xff | 0x0101010101010101) ^ 1);
    let after = board.place(&shape(1), 0, 0).unwrap();
    assert_eq!(after.lines(), 2);
    assert_eq!(after.removed_mask.count_ones(), 15);
    assert_eq!(after.board.bits(), 0);
    let ring = Shape::new(vec![
        (0, 0),
        (0, 1),
        (0, 2),
        (1, 0),
        (1, 2),
        (2, 0),
        (2, 1),
        (2, 2),
    ])
    .unwrap();
    assert!(Board::from_bits(1 << 9).can_place(&ring, 0, 0));
    assert!(!Board::default().can_place(&shape(3), 0, 7));
    assert!(Shape::new(vec![(0, 0), (0, 0)]).is_err());
    assert!(Shape::new(vec![(1, 1)]).is_err());
}
#[test]
fn geometry_matches_grid_oracle() {
    let shapes = catalog::evaluation_shapes();
    let eval = Evaluator::new(&shapes).unwrap();
    let mut seed = 81u64;
    for _ in 0..100 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let rows = Board::from_bits(seed).rows();
        let mut visited = [[false; 8]; 8];
        let mut small = 0;
        let mut empty = 0;
        for r in 0..8 {
            for c in 0..8 {
                if rows[r].as_bytes()[c] == b'#' {
                    continue;
                }
                empty += 1;
                if visited[r][c] {
                    continue;
                }
                visited[r][c] = true;
                let mut stack = vec![(r, c)];
                let mut size = 0;
                while let Some((rr, cc)) = stack.pop() {
                    size += 1;
                    for (dr, dc) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                        let nr = rr as i32 + dr;
                        let nc = cc as i32 + dc;
                        if (0..8).contains(&nr) && (0..8).contains(&nc) {
                            let a = nr as usize;
                            let b = nc as usize;
                            if !visited[a][b] && rows[a].as_bytes()[b] == b'.' {
                                visited[a][b] = true;
                                stack.push((a, b));
                            }
                        }
                    }
                }
                if size <= 3 {
                    small += size;
                }
            }
        }
        let expected = if empty == 0 {
            1.0
        } else {
            f64::from(small) / f64::from(empty)
        };
        assert_eq!(fragmentation(seed), expected);
        let counts: Vec<u32> = shapes
            .iter()
            .map(|s| {
                (0..=8 - s.height())
                    .flat_map(|r| (0..=8 - s.width()).map(move |c| (r, c)))
                    .filter(|&(r, c)| {
                        s.cells().iter().all(|&(dr, dc)| {
                            rows[usize::from(r + dr)].as_bytes()[usize::from(c + dc)] == b'.'
                        })
                    })
                    .count() as u32
            })
            .collect();
        let geometry = eval.evaluate(seed);
        assert_eq!(geometry.total_positions, counts.iter().sum::<u32>());
        assert_eq!(
            geometry.coverage,
            counts.iter().filter(|&&v| v > 0).count() as u32
        );
    }
    assert_eq!(fragmentation(u64::MAX ^ ((1 << 7) | (1 << 8))), 1.0);
}
#[test]
fn blocked_square_can_be_opened_by_later_slot() {
    let board = Board::from_rows(
        &[
            "##.#####", "........", "........", "#.#.#.#.", "#.#.#.#.", "#.#.#.#.", "#.#.#.#.",
            "#.#.#.#.",
        ]
        .map(str::to_owned),
    )
    .unwrap();
    let g = game(board, vec![piece(0, shape(13)), piece(2, shape(1))]);
    let result = solve(
        &g,
        &catalog::evaluation_shapes(),
        SearchOptions::default(),
        |_| true,
    )
    .unwrap();
    assert!(result.complete);
    assert_eq!(result.steps.len(), 2);
    assert_eq!(result.steps[0].slot, 2);
    assert_eq!(
        (
            result.steps[0].movement.row,
            result.steps[0].movement.column
        ),
        (0, 2)
    );
}
#[test]
fn partial_no_move_and_cancellation_are_distinct() {
    let mut g = game(Board::default(), vec![piece(0, shape(1))]);
    let result = solve(
        &g,
        &catalog::evaluation_shapes(),
        SearchOptions { max_nodes: 1 },
        |_| true,
    )
    .unwrap();
    assert!(!result.complete);
    assert!(result.steps.is_empty());
    assert!(
        solve(
            &g,
            &catalog::evaluation_shapes(),
            SearchOptions::default(),
            |_| false
        )
        .is_err()
    );
    g.board = Board::from_rows(
        &[
            "#.#.#.#.", ".#.#.#.#", "#.#.#.#.", ".#.#.#.#", "#.#.#.#.", ".#.#.#.#", "#.#.#.#.",
            ".#.#.#.#",
        ]
        .map(str::to_owned),
    )
    .unwrap();
    g.pieces[0].shape = shape(9);
    let result = solve(
        &g,
        &catalog::evaluation_shapes(),
        SearchOptions::default(),
        |_| true,
    )
    .unwrap();
    assert!(result.complete);
    assert!(result.steps.is_empty());
}
#[test]
fn identical_slots_keep_deterministic_recommendation_and_all_explorer_orders() {
    let row = Shape::new((0..8).map(|c| (0, c)).collect()).unwrap();
    let g = game(
        Board::default(),
        vec![piece(0, row.clone()), piece(1, row.clone()), piece(2, row)],
    );
    let result = solve(
        &g,
        &catalog::evaluation_shapes(),
        SearchOptions::default(),
        |_| true,
    )
    .unwrap();
    assert_eq!(result.candidate.gains[0], 84);
    assert_eq!(
        result.candidate.path,
        vec![
            Move {
                piece: 0,
                row: 0,
                column: 0
            },
            Move {
                piece: 1,
                row: 0,
                column: 0
            },
            Move {
                piece: 2,
                row: 0,
                column: 0
            }
        ]
    );
    let index = enumerate(
        &g,
        &catalog::evaluation_shapes(),
        &[],
        SearchOptions::default(),
        |_| true,
    )
    .unwrap();
    assert!(index.complete);
    assert_eq!(index.paths.len(), 6 * 8 * 8 * 8);
    let page = index
        .page(
            &g,
            &ExplorerFilter {
                order: vec![2, 0, 1],
                positions: vec![Some((7, 0)), None, None],
                page: 0,
                page_size: 10,
            },
        )
        .unwrap();
    assert_eq!(page.total, 64);
    assert_eq!(
        page.position_options[0],
        (0..8).map(|r| (r, 0)).collect::<Vec<_>>()
    );
    assert!(page.entries.iter().all(|e| e.path[0]
        == Move {
            piece: 2,
            row: 7,
            column: 0
        }));
    let prefix = vec![Move {
        piece: 2,
        row: 7,
        column: 0,
    }];
    let continuation = enumerate(
        &g,
        &catalog::evaluation_shapes(),
        &prefix,
        SearchOptions::default(),
        |_| true,
    )
    .unwrap();
    assert_eq!(continuation.paths.len(), 128);
    assert!(
        continuation
            .paths
            .iter()
            .enumerate()
            .all(|(i, _)| continuation.path(i + 1).unwrap()[0] == prefix[0])
    );
}

#[test]
fn mixed_slot_explorer_counts_match_a_small_independent_exhaustive_search() {
    fn count(board: Board, pieces: &[Piece], used: u8) -> usize {
        if used.count_ones() as usize == pieces.len() {
            return 1;
        }
        let mut total = 0;
        for (index, piece) in pieces.iter().enumerate() {
            if used & (1 << index) != 0 {
                continue;
            }
            for row in 0..8 {
                for column in 0..8 {
                    if let Some(next) = board.place(&piece.shape, row, column) {
                        total += count(next.board, pieces, used | (1 << index));
                    }
                }
            }
        }
        total
    }
    let board = Board::from_rows(
        &[
            "#.#.#.#.", ".#.#.#.#", "#.#.#.#.", ".#.#.#.#", "#.#.#.#.", ".#.#.#.#", "........",
            "........",
        ]
        .map(str::to_owned),
    )
    .unwrap();
    for pieces in [
        vec![piece(0, shape(1)), piece(2, shape(3))],
        vec![piece(1, shape(9)), piece(2, shape(6))],
    ] {
        let g = game(board, pieces);
        let expected = count(board, &g.pieces, 0);
        let index = enumerate(
            &g,
            &catalog::evaluation_shapes(),
            &[],
            SearchOptions::default(),
            |_| true,
        )
        .unwrap();
        assert!(index.complete);
        assert_eq!(index.paths.len(), expected);
        let page = index
            .page(
                &g,
                &ExplorerFilter {
                    order: vec![],
                    positions: vec![],
                    page: 0,
                    page_size: 100,
                },
            )
            .unwrap();
        assert_eq!(page.total, expected);
        assert!(page.entries.windows(2).all(|p| p[0].score >= p[1].score));
    }
}
