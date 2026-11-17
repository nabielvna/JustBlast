pub mod evaluation;
mod explorer;
mod search;

use crate::domain::combo::{ComboState, RuleContext, Scenario, advance_combo, model_points};
use crate::domain::{Board, MoveResult, Shape};
use evaluation::Geometry;
pub use explorer::{ExplorerEntry, ExplorerFilter, ExplorerIndex, ExplorerPage, enumerate};
pub use search::{SearchOptions, SearchResult, SearchStats, solve};
use std::cmp::Ordering;

#[derive(Clone, Debug, PartialEq)]
pub struct Piece {
    pub slot: u8,
    pub catalog_id: Option<u32>,
    pub name: String,
    pub shape: Shape,
}
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Move {
    pub piece: u8,
    pub row: u8,
    pub column: u8,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Weights {
    pub points: f64,
    pub combo: f64,
    pub flexibility: f64,
    pub fragmentation: f64,
}
impl Default for Weights {
    fn default() -> Self {
        Self {
            points: 45.0,
            combo: 25.0,
            flexibility: 25.0,
            fragmentation: 5.0,
        }
    }
}
impl Weights {
    pub fn valid(self) -> bool {
        [
            self.points,
            self.combo,
            self.flexibility,
            self.fragmentation,
        ]
        .iter()
        .all(|v| v.is_finite() && *v >= 0.0)
            && (self.points + self.combo + self.flexibility + self.fragmentation).is_finite()
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Checkpoint {
    pub candidate: Candidate,
    pub steps: Vec<Step>,
}
#[derive(Clone, Debug)]
pub struct Game {
    pub checkpoint: Option<Checkpoint>,
    pub board: Board,
    pub pieces: Vec<Piece>,
    pub states: [ComboState; 3],
    pub context: RuleContext,
    pub weights: Weights,
}
impl Game {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.pieces.is_empty() || self.pieces.len() > 3 {
            return Err("Provide one to three pieces");
        }
        let mut slots = 0;
        for piece in &self.pieces {
            if piece.slot > 2 || slots & (1 << piece.slot) != 0 {
                return Err("Piece slots must be distinct and within 0..2");
            }
            slots |= 1 << piece.slot;
        }
        if self.pieces.windows(2).any(|w| w[0].slot > w[1].slot) {
            return Err("Pieces must be sorted by slot");
        }
        if !self.weights.valid() || self.states.iter().any(|s| s.producer_remaining > 2) {
            return Err("Invalid scoring context");
        }
        if let Some(checkpoint) = &self.checkpoint {
            let path = &checkpoint.candidate.path;
            if path.len() != checkpoint.steps.len()
                || path.len() > self.pieces.len()
                || path.iter().enumerate().any(|(i, m)| {
                    let step = &checkpoint.steps[i];
                    self.pieces
                        .get(usize::from(m.piece))
                        .is_none_or(|p| p.slot != step.slot)
                        || path[..i].iter().any(|old| old.piece == m.piece)
                        || m.row != step.movement.row
                        || m.column != step.movement.column
                })
            {
                return Err("Invalid acknowledged checkpoint");
            }
        }
        Ok(())
    }
    pub fn initial(&self) -> Candidate {
        if let Some(checkpoint) = &self.checkpoint {
            return checkpoint.candidate.clone();
        }
        Candidate {
            board: self.board,
            states: self.states,
            gains: [0; 3],
            path: Vec::new(),
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Candidate {
    pub board: Board,
    pub states: [ComboState; 3],
    pub gains: [u64; 3],
    pub path: Vec<Move>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Metrics {
    pub value: f64,
    pub potential_points: f64,
    pub combo_alive: bool,
    pub geometry: Geometry,
    pub normalized: [Option<f64>; 3],
}
pub fn assess(
    candidate: &Candidate,
    maxima: [u64; 3],
    geometry: Geometry,
    weights: Weights,
) -> Metrics {
    let normalized = std::array::from_fn(|i| {
        if maxima[i] == 0 {
            None
        } else {
            Some(candidate.gains[i] as f64 / maxima[i] as f64)
        }
    });
    let points = normalized
        .iter()
        .flatten()
        .copied()
        .reduce(f64::min)
        .unwrap_or(0.0);
    let combo_alive = candidate.states.iter().all(|s| s.count > 0);
    Metrics {
        value: weights.points * points
            + weights.combo * f64::from(combo_alive)
            + weights.flexibility * geometry.flexibility
            - weights.fragmentation * geometry.fragmentation,
        potential_points: points,
        combo_alive,
        geometry,
        normalized,
    }
}
pub fn compare(a: &Metrics, a_path: &[Move], b: &Metrics, b_path: &[Move]) -> Ordering {
    b.value
        .total_cmp(&a.value)
        .then_with(|| b.geometry.coverage.cmp(&a.geometry.coverage))
        .then_with(|| b.geometry.total_positions.cmp(&a.geometry.total_positions))
        .then_with(|| {
            a.geometry
                .fragmentation
                .total_cmp(&b.geometry.fragmentation)
        })
        .then_with(|| a_path.cmp(b_path))
}
#[derive(Clone, Debug, PartialEq)]
pub struct Step {
    pub movement: Move,
    pub slot: u8,
    pub result: MoveResult,
    pub before: Board,
    pub states_before: [ComboState; 3],
    pub states_after: [ComboState; 3],
    pub points: [u64; 3],
    pub possible_moves: usize,
}
pub fn advance_candidate(
    game: &Game,
    current: &Candidate,
    movement: Move,
) -> Result<Candidate, &'static str> {
    let piece = game
        .pieces
        .get(usize::from(movement.piece))
        .ok_or("Unknown piece")?;
    if current.path.iter().any(|m| m.piece == movement.piece) {
        return Err("Piece already used");
    }
    let result = current
        .board
        .place(&piece.shape, movement.row, movement.column)
        .ok_or("Illegal placement")?;
    let remaining = (game.pieces.len() - current.path.len() - 1) as u8;
    let states = std::array::from_fn(|i| {
        advance_combo(
            current.states[i],
            Scenario::ALL[i],
            result.lines(),
            remaining,
            game.context,
        )
    });
    let gains = std::array::from_fn(|i| {
        current.gains[i]
            + model_points(
                piece.shape.cells().len() as u32,
                result.lines(),
                states[i],
                Scenario::ALL[i],
                game.context,
                result.board.bits() == 0,
            )
    });
    let mut path = current.path.clone();
    path.push(movement);
    Ok(Candidate {
        board: result.board,
        states,
        gains,
        path,
    })
}
pub fn transition(
    game: &Game,
    current: &Candidate,
    movement: Move,
) -> Result<(Candidate, Step), &'static str> {
    let next = advance_candidate(game, current, movement)?;
    let piece = &game.pieces[usize::from(movement.piece)];
    let result = current
        .board
        .place(&piece.shape, movement.row, movement.column)
        .ok_or("Illegal placement")?;
    let step = Step {
        movement,
        slot: piece.slot,
        before: current.board,
        possible_moves: piece
            .shape
            .placements()
            .iter()
            .filter(|p| p.2 & current.board.bits() == 0)
            .count(),
        result,
        states_before: current.states,
        states_after: next.states,
        points: std::array::from_fn(|i| next.gains[i] - current.gains[i]),
    };
    Ok((next, step))
}
pub fn replay(game: &Game, path: &[Move]) -> Result<(Candidate, Vec<Step>), &'static str> {
    game.validate()?;
    let mut current = game.initial();
    let mut steps = game
        .checkpoint
        .as_ref()
        .map_or_else(Vec::new, |c| c.steps.clone());
    if !path.starts_with(&current.path) {
        return Err("Completed moves must stay unchanged");
    }
    let prefix = current.path.len();
    for &movement in &path[prefix..] {
        let (next, step) = transition(game, &current, movement)?;
        current = next;
        steps.push(step);
    }
    Ok((current, steps))
}
