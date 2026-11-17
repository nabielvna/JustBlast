use justblast_core::{
    domain::{
        Board, catalog,
        combo::{ComboState, RuleContext},
    },
    solver::*,
};
use std::time::Instant;
fn main() {
    let selection = std::env::args().nth(1);
    let shapes = catalog::evaluation_shapes();
    if selection.as_deref().is_none_or(|s| s == "demo") {
        let board = Board::from_rows(&[
            "...##...".to_owned(),
            "....#...".to_owned(),
            "##......".to_owned(),
            ".##.....".to_owned(),
            "...#....".to_owned(),
            "##......".to_owned(),
            "###.....".to_owned(),
            "..##....".to_owned(),
        ])
        .unwrap();
        let pieces = vec![
            Piece {
                slot: 0,
                catalog_id: Some(5),
                name: "Piece".to_owned(),
                shape: catalog::catalog().into_iter().find(|s| s.0 == 5).unwrap().1,
            },
            Piece {
                slot: 1,
                catalog_id: Some(9),
                name: "Piece".to_owned(),
                shape: catalog::catalog().into_iter().find(|s| s.0 == 9).unwrap().1,
            },
            Piece {
                slot: 2,
                catalog_id: Some(27),
                name: "Piece".to_owned(),
                shape: catalog::catalog()
                    .into_iter()
                    .find(|s| s.0 == 27)
                    .unwrap()
                    .1,
            },
        ];
        let states = [
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
        let game = Game {
            checkpoint: None,
            board,
            pieces,
            states,
            context: RuleContext::default(),
            weights: Weights::default(),
        };
        let started = Instant::now();
        let result = solve(&game, &shapes, SearchOptions::default(), |_| true).unwrap();
        println!(
            "demo\t{:.6}\t{}\t{}\t{}\t{}\t{:.12}\t{:?}\t{:?}",
            started.elapsed().as_secs_f64() * 1000.0,
            result.stats.visited,
            result.stats.cache_hits,
            result.stats.candidates,
            result.complete,
            result.metrics.value,
            result.candidate.path,
            result.maxima
        );
        if std::env::args().any(|arg| arg == "explore") {
            let started = Instant::now();
            let index = enumerate(&game, &shapes, &[], SearchOptions::default(), |_| true).unwrap();
            println!(
                "explore\t{:.6}\t{}\t{}\t{}\t{}",
                started.elapsed().as_secs_f64() * 1000.0,
                index.visited,
                index.paths.len(),
                index.finals.len(),
                index.complete
            );
        }
    }
    if selection.as_deref().is_none_or(|s| s == "cycle-001") {
        let board = Board::from_rows(&[
            "########".to_owned(),
            "#..####.".to_owned(),
            ".#.#...#".to_owned(),
            "###..###".to_owned(),
            "######.#".to_owned(),
            "#..#.###".to_owned(),
            "..##..##".to_owned(),
            "####.###".to_owned(),
        ])
        .unwrap();
        let pieces = vec![
            Piece {
                slot: 0,
                catalog_id: Some(14),
                name: "Piece".to_owned(),
                shape: catalog::catalog()
                    .into_iter()
                    .find(|s| s.0 == 14)
                    .unwrap()
                    .1,
            },
            Piece {
                slot: 1,
                catalog_id: Some(14),
                name: "Piece".to_owned(),
                shape: catalog::catalog()
                    .into_iter()
                    .find(|s| s.0 == 14)
                    .unwrap()
                    .1,
            },
            Piece {
                slot: 2,
                catalog_id: Some(20),
                name: "Piece".to_owned(),
                shape: catalog::catalog()
                    .into_iter()
                    .find(|s| s.0 == 20)
                    .unwrap()
                    .1,
            },
        ];
        let states = [ComboState::default(); 3];
        let game = Game {
            checkpoint: None,
            board,
            pieces,
            states,
            context: RuleContext::default(),
            weights: Weights::default(),
        };
        let started = Instant::now();
        let result = solve(&game, &shapes, SearchOptions::default(), |_| true).unwrap();
        println!(
            "cycle-001\t{:.6}\t{}\t{}\t{}\t{}\t{:.12}\t{:?}\t{:?}",
            started.elapsed().as_secs_f64() * 1000.0,
            result.stats.visited,
            result.stats.cache_hits,
            result.stats.candidates,
            result.complete,
            result.metrics.value,
            result.candidate.path,
            result.maxima
        );
        if std::env::args().any(|arg| arg == "explore") {
            let started = Instant::now();
            let index = enumerate(&game, &shapes, &[], SearchOptions::default(), |_| true).unwrap();
            println!(
                "explore\t{:.6}\t{}\t{}\t{}\t{}",
                started.elapsed().as_secs_f64() * 1000.0,
                index.visited,
                index.paths.len(),
                index.finals.len(),
                index.complete
            );
        }
    }
    if selection.as_deref().is_none_or(|s| s == "sparse-mixed") {
        let board = Board::from_rows(&[
            "...##...".to_owned(),
            "....#...".to_owned(),
            "##......".to_owned(),
            ".##.....".to_owned(),
            "...#....".to_owned(),
            "##......".to_owned(),
            "###.....".to_owned(),
            "..##....".to_owned(),
        ])
        .unwrap();
        let pieces = vec![
            Piece {
                slot: 0,
                catalog_id: Some(5),
                name: "Piece".to_owned(),
                shape: catalog::catalog().into_iter().find(|s| s.0 == 5).unwrap().1,
            },
            Piece {
                slot: 1,
                catalog_id: Some(9),
                name: "Piece".to_owned(),
                shape: catalog::catalog().into_iter().find(|s| s.0 == 9).unwrap().1,
            },
            Piece {
                slot: 2,
                catalog_id: Some(6),
                name: "Piece".to_owned(),
                shape: catalog::catalog().into_iter().find(|s| s.0 == 6).unwrap().1,
            },
        ];
        let states = [ComboState::default(); 3];
        let game = Game {
            checkpoint: None,
            board,
            pieces,
            states,
            context: RuleContext::default(),
            weights: Weights::default(),
        };
        let started = Instant::now();
        let result = solve(&game, &shapes, SearchOptions::default(), |_| true).unwrap();
        println!(
            "sparse-mixed\t{:.6}\t{}\t{}\t{}\t{}\t{:.12}\t{:?}\t{:?}",
            started.elapsed().as_secs_f64() * 1000.0,
            result.stats.visited,
            result.stats.cache_hits,
            result.stats.candidates,
            result.complete,
            result.metrics.value,
            result.candidate.path,
            result.maxima
        );
        if std::env::args().any(|arg| arg == "explore") {
            let started = Instant::now();
            let index = enumerate(&game, &shapes, &[], SearchOptions::default(), |_| true).unwrap();
            println!(
                "explore\t{:.6}\t{}\t{}\t{}\t{}",
                started.elapsed().as_secs_f64() * 1000.0,
                index.visited,
                index.paths.len(),
                index.finals.len(),
                index.complete
            );
        }
    }
    if selection
        .as_deref()
        .is_none_or(|s| s == "empty-three-singles")
    {
        let board = Board::from_rows(&[
            "........".to_owned(),
            "........".to_owned(),
            "........".to_owned(),
            "........".to_owned(),
            "........".to_owned(),
            "........".to_owned(),
            "........".to_owned(),
            "........".to_owned(),
        ])
        .unwrap();
        let pieces = vec![
            Piece {
                slot: 0,
                catalog_id: Some(1),
                name: "Piece".to_owned(),
                shape: catalog::catalog().into_iter().find(|s| s.0 == 1).unwrap().1,
            },
            Piece {
                slot: 1,
                catalog_id: Some(1),
                name: "Piece".to_owned(),
                shape: catalog::catalog().into_iter().find(|s| s.0 == 1).unwrap().1,
            },
            Piece {
                slot: 2,
                catalog_id: Some(1),
                name: "Piece".to_owned(),
                shape: catalog::catalog().into_iter().find(|s| s.0 == 1).unwrap().1,
            },
        ];
        let states = [ComboState::default(); 3];
        let game = Game {
            checkpoint: None,
            board,
            pieces,
            states,
            context: RuleContext::default(),
            weights: Weights::default(),
        };
        let started = Instant::now();
        let result = solve(&game, &shapes, SearchOptions::default(), |_| true).unwrap();
        println!(
            "empty-three-singles\t{:.6}\t{}\t{}\t{}\t{}\t{:.12}\t{:?}\t{:?}",
            started.elapsed().as_secs_f64() * 1000.0,
            result.stats.visited,
            result.stats.cache_hits,
            result.stats.candidates,
            result.complete,
            result.metrics.value,
            result.candidate.path,
            result.maxima
        );
        if std::env::args().any(|arg| arg == "explore") {
            let started = Instant::now();
            let index = enumerate(&game, &shapes, &[], SearchOptions::default(), |_| true).unwrap();
            println!(
                "explore\t{:.6}\t{}\t{}\t{}\t{}",
                started.elapsed().as_secs_f64() * 1000.0,
                index.visited,
                index.paths.len(),
                index.finals.len(),
                index.complete
            );
        }
    }
}
