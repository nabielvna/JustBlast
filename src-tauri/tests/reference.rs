use justblast_core::{
    domain::{
        Board, catalog,
        combo::{ComboState, RuleContext},
    },
    solver::*,
};
use serde_json::Value;
#[test]
fn all_shared_benchmark_cases_match_legacy_paths_scores_boards_and_search_completeness() {
    let inputs: Vec<Value> =
        serde_json::from_str(include_str!("../../tests/fixtures/benchmark-inputs.json")).unwrap();
    let references: Vec<Value> = serde_json::from_str(include_str!(
        "../../tests/fixtures/engine-benchmark-reference.json"
    ))
    .unwrap();
    for (input, reference) in inputs.iter().zip(&references) {
        let board = Board::from_rows(
            &input["board"]
                .as_array()
                .unwrap()
                .iter()
                .map(|s| s.as_str().unwrap().to_owned())
                .collect::<Vec<_>>(),
        )
        .unwrap();
        let pieces = input["pieces"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| {
                let id = p["id"].as_u64().unwrap() as u32;
                Piece {
                    slot: p["slot"].as_u64().unwrap() as u8,
                    catalog_id: Some(id),
                    name: "Piece".to_owned(),
                    shape: catalog::catalog()
                        .into_iter()
                        .find(|s| s.0 == id)
                        .unwrap()
                        .1,
                }
            })
            .collect();
        let mut states = [ComboState::default(); 3];
        if let Some(seeds) = input["states"].as_array() {
            for (s, v) in states.iter_mut().zip(seeds) {
                let n = |key| v[key].as_u64().unwrap() as u32;
                *s = ComboState {
                    count: n("count"),
                    misses: n("misses"),
                    producer_remaining: n("producer_remaining") as u8,
                    saved_count: n("saved_count"),
                    checking: v["checking"].as_bool().unwrap(),
                    waiting: n("waiting"),
                    recovered: v["recovered"].as_bool().unwrap(),
                };
            }
        }
        let game = Game {
            checkpoint: None,
            board,
            pieces,
            states,
            context: RuleContext::default(),
            weights: Weights::default(),
        };
        let result = solve(
            &game,
            &catalog::evaluation_shapes(),
            SearchOptions::default(),
            |_| true,
        )
        .unwrap();
        assert_eq!(result.complete, reference["complete"].as_bool().unwrap());
        assert_eq!(result.stats.visited, reference["visited"].as_u64().unwrap());
        assert_eq!(
            serde_json::to_value(result.candidate.board.rows()).unwrap(),
            reference["finalBoard"]
        );
        assert_eq!(
            serde_json::to_value(result.maxima).unwrap(),
            reference["maxima"]
        );
        assert_eq!(
            serde_json::to_value(result.candidate.gains).unwrap(),
            reference["gains"]
        );
        let steps:Vec<_>=result.steps.iter().map(|s|serde_json::json!({"slot":s.slot,"row":s.movement.row,"column":s.movement.column})).collect();
        assert_eq!(serde_json::to_value(steps).unwrap(), reference["steps"]);
        for (name, value) in [
            ("value", result.metrics.value),
            ("potential_points", result.metrics.potential_points),
            ("flexibility", result.metrics.geometry.flexibility),
            ("fragmentation", result.metrics.geometry.fragmentation),
        ] {
            assert!(
                (value - reference["metrics"][name].as_f64().unwrap()).abs() < 1e-12,
                "{}: {name}",
                input["name"]
            );
        }
    }
}

#[test]
fn every_owned_catalogue_id_cell_and_alias_matches_the_recorded_resource() {
    let value: Value =
        serde_json::from_str(include_str!("../../resources/shapes-v1.json")).unwrap();
    let rows = value["shapes"].as_array().unwrap();
    let catalog = catalog::catalog();
    assert_eq!(catalog.len(), rows.len());
    for ((id, shape), row) in catalog.iter().zip(rows) {
        assert_eq!(u64::from(*id), row["id"].as_u64().unwrap());
        assert_eq!(serde_json::to_value(shape.cells()).unwrap(), row["cells"]);
    }
    assert!(catalog::evaluation_shapes().len() < catalog.len());
}
