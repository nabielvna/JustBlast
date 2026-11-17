use super::*;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug)]
pub struct SearchOptions {
    pub max_nodes: u64,
}
impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            max_nodes: 2_000_000,
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SearchStats {
    pub visited: u64,
    pub cache_hits: u64,
    pub legal_edges: u64,
    pub candidates: usize,
}
#[derive(Clone, Debug, PartialEq)]
pub struct SearchResult {
    pub candidate: Candidate,
    pub steps: Vec<Step>,
    pub metrics: Metrics,
    pub maxima: [u64; 3],
    pub complete: bool,
    pub stats: SearchStats,
}

type Key = (u64, u8, [ComboState; 3], [u64; 3]);
struct Search<'a, F> {
    game: &'a Game,
    options: SearchOptions,
    checkpoint: F,
    seen: HashSet<Key>,
    terminal_keys: HashSet<(u64, [ComboState; 3], [u64; 3])>,
    candidates: Vec<Candidate>,
    best_depth: usize,
    complete: bool,
    stats: SearchStats,
}
impl<F: FnMut(u64) -> bool> Search<'_, F> {
    fn visit(&mut self, current: Candidate, remaining: u8) -> Result<(), &'static str> {
        if self.stats.visited >= self.options.max_nodes {
            self.complete = false;
            return Ok(());
        }
        if self.stats.visited.is_multiple_of(256) && !(self.checkpoint)(self.stats.visited) {
            return Err("Search cancelled");
        }
        self.stats.visited += 1;
        if !self.seen.insert((
            current.board.bits(),
            remaining,
            current.states,
            current.gains,
        )) {
            self.stats.cache_hits += 1;
            return Ok(());
        }
        let depth = current.path.len();
        if depth > self.best_depth {
            self.best_depth = depth;
            self.candidates.clear();
            self.terminal_keys.clear();
        }
        if depth == self.best_depth
            && self
                .terminal_keys
                .insert((current.board.bits(), current.states, current.gains))
        {
            self.candidates.push(current.clone());
        }
        let mut equivalent = Vec::new();
        for index in 0..self.game.pieces.len() {
            if remaining & (1 << index) == 0 {
                continue;
            }
            let shape = &self.game.pieces[index].shape;
            if equivalent.contains(&shape) {
                continue;
            }
            equivalent.push(shape);
            for (row, column, mask) in shape.placements() {
                if current.board.bits() & mask != 0 {
                    continue;
                }
                self.stats.legal_edges += 1;
                let next = advance_candidate(
                    self.game,
                    &current,
                    Move {
                        piece: index as u8,
                        row,
                        column,
                    },
                )?;
                self.visit(next, remaining & !(1 << index))?;
                if !self.complete {
                    return Ok(());
                }
            }
        }
        Ok(())
    }
}
pub fn solve(
    game: &Game,
    shapes: &[Shape],
    options: SearchOptions,
    mut checkpoint: impl FnMut(u64) -> bool,
) -> Result<SearchResult, &'static str> {
    game.validate()?;
    if options.max_nodes == 0 {
        return Err("Search budget must be positive");
    }
    let evaluator = evaluation::Evaluator::new(shapes)?;
    let mut search = Search {
        game,
        options,
        checkpoint: &mut checkpoint,
        seen: HashSet::new(),
        terminal_keys: HashSet::new(),
        candidates: Vec::new(),
        best_depth: 0,
        complete: true,
        stats: SearchStats::default(),
    };
    let initial = game.initial();
    let used = initial
        .path
        .iter()
        .fold(0u8, |mask, m| mask | (1 << m.piece));
    search.best_depth = initial.path.len();
    search.visit(initial, ((1 << game.pieces.len()) - 1) & !used)?;
    let maxima = std::array::from_fn(|i| {
        search
            .candidates
            .iter()
            .map(|c| c.gains[i])
            .max()
            .unwrap_or(0)
    });
    let mut best: Option<(Candidate, Metrics)> = None;
    let mut geometry_cache = HashMap::new();
    for (index, candidate) in search.candidates.iter().enumerate() {
        if index.is_multiple_of(128) && !(search.checkpoint)(search.stats.visited) {
            return Err("Search cancelled");
        }
        let geometry = *geometry_cache
            .entry(candidate.board.bits())
            .or_insert_with(|| evaluator.evaluate(candidate.board.bits()));
        let metrics = assess(candidate, maxima, geometry, game.weights);
        if best.as_ref().is_none_or(|(old, old_metrics)| {
            compare(&metrics, &candidate.path, old_metrics, &old.path).is_lt()
        }) {
            best = Some((candidate.clone(), metrics));
        }
    }
    search.stats.candidates = search.candidates.len();
    let (candidate, metrics) = best.ok_or("Search produced no state")?;
    let (_, steps) = replay(game, &candidate.path)?;
    Ok(SearchResult {
        candidate,
        steps,
        metrics,
        maxima,
        complete: search.complete,
        stats: search.stats,
    })
}
