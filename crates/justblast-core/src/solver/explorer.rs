use super::*;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct ExplorerIndex {
    pub paths: Vec<(u32, usize)>,
    pub finals: Vec<(Candidate, Metrics)>,
    pub ordered: Vec<usize>,
    pub maxima: [u64; 3],
    pub complete: bool,
    pub visited: u64,
    pub length: usize,
    pub prefix: Vec<Move>,
}
#[derive(Clone, Debug, Default)]
pub struct ExplorerFilter {
    pub order: Vec<u8>,
    pub positions: Vec<Option<(u8, u8)>>,
    pub page: usize,
    pub page_size: usize,
}
#[derive(Clone, Debug)]
pub struct ExplorerEntry {
    pub id: usize,
    pub path: Vec<Move>,
    pub score: Option<f64>,
    pub clears: u32,
}
#[derive(Clone, Debug)]
pub struct ExplorerPage {
    pub entries: Vec<ExplorerEntry>,
    pub total: usize,
    pub all_total: usize,
    pub page: usize,
    pub pages: usize,
    pub complete: bool,
    pub next_counts: [usize; 3],
    pub position_options: Vec<Vec<(u8, u8)>>,
}
fn pack(path: &[Move]) -> u32 {
    path.iter().fold(0, |value, m| {
        value << 8 | u32::from(m.piece) << 6 | u32::from(m.row) << 3 | u32::from(m.column)
    })
}
fn unpack(value: u32, length: usize) -> Vec<Move> {
    (0..length)
        .rev()
        .map(|offset| Move {
            piece: ((value >> (8 * offset + 6)) & 3) as u8,
            row: ((value >> (8 * offset + 3)) & 7) as u8,
            column: ((value >> (8 * offset)) & 7) as u8,
        })
        .collect()
}
struct Enumerator<'a, F> {
    game: &'a Game,
    checkpoint: F,
    max_nodes: u64,
    visited: u64,
    complete: bool,
    paths: Vec<(u32, usize)>,
    finals: Vec<Candidate>,
    seen: HashMap<(u64, [ComboState; 3], [u64; 3]), usize>,
}
impl<F: FnMut(u64) -> bool> Enumerator<'_, F> {
    fn visit(&mut self, current: Candidate) -> Result<(), &'static str> {
        if self.visited >= self.max_nodes {
            self.complete = false;
            return Ok(());
        }
        if self.visited.is_multiple_of(256) && !(self.checkpoint)(self.visited) {
            return Err("Search cancelled");
        }
        self.visited += 1;
        if current.path.len() == self.game.pieces.len() {
            let key = (current.board.bits(), current.states, current.gains);
            if self.paths.len() >= 250_000
                || (!self.seen.contains_key(&key) && self.finals.len() >= 50_000)
            {
                self.complete = false;
                return Ok(());
            }
            let terminal = if let Some(&index) = self.seen.get(&key) {
                index
            } else {
                let index = self.finals.len();
                self.seen.insert(key, index);
                let mut final_state = current.clone();
                final_state.path.clear();
                self.finals.push(final_state);
                index
            };
            self.paths.push((pack(&current.path), terminal));
            return Ok(());
        }
        for index in 0..self.game.pieces.len() {
            if current.path.iter().any(|m| usize::from(m.piece) == index) {
                continue;
            }
            for (row, column, mask) in self.game.pieces[index].shape.placements() {
                if current.board.bits() & mask != 0 {
                    continue;
                }
                let next = advance_candidate(
                    self.game,
                    &current,
                    Move {
                        piece: index as u8,
                        row,
                        column,
                    },
                )?;
                self.visit(next)?;
                if !self.complete {
                    return Ok(());
                }
            }
        }
        Ok(())
    }
}
pub fn enumerate(
    game: &Game,
    shapes: &[Shape],
    prefix: &[Move],
    options: SearchOptions,
    mut checkpoint: impl FnMut(u64) -> bool,
) -> Result<ExplorerIndex, &'static str> {
    if options.max_nodes == 0 {
        return Err("Search budget must be positive");
    }
    let (initial, _) = replay(game, prefix)?;
    let evaluator = evaluation::Evaluator::new(shapes)?;
    let mut enumeration = Enumerator {
        game,
        checkpoint: &mut checkpoint,
        max_nodes: options.max_nodes,
        visited: 0,
        complete: true,
        paths: Vec::new(),
        finals: Vec::new(),
        seen: HashMap::new(),
    };
    enumeration.visit(initial)?;
    let maxima = std::array::from_fn(|i| {
        enumeration
            .finals
            .iter()
            .map(|c| c.gains[i])
            .max()
            .unwrap_or(0)
    });
    let mut finals = Vec::new();
    let mut cache = HashMap::new();
    for (index, candidate) in enumeration.finals.into_iter().enumerate() {
        if index.is_multiple_of(128) && !(enumeration.checkpoint)(enumeration.visited) {
            return Err("Search cancelled");
        }
        let geometry = *cache
            .entry(candidate.board.bits())
            .or_insert_with(|| evaluator.evaluate(candidate.board.bits()));
        let metrics = assess(&candidate, maxima, geometry, game.weights);
        finals.push((candidate, metrics));
    }
    let mut ordered: Vec<usize> = (0..enumeration.paths.len()).collect();
    ordered.sort_unstable_by(|&a, &b| {
        let (pa, ta) = enumeration.paths[a];
        let (pb, tb) = enumeration.paths[b];
        compare(&finals[ta].1, &[], &finals[tb].1, &[]).then_with(|| pa.cmp(&pb))
    });
    if !(enumeration.checkpoint)(enumeration.visited) {
        return Err("Search cancelled");
    }
    Ok(ExplorerIndex {
        paths: enumeration.paths,
        finals,
        ordered,
        maxima,
        complete: enumeration.complete,
        visited: enumeration.visited,
        length: game.pieces.len(),
        prefix: prefix.to_vec(),
    })
}
impl ExplorerIndex {
    pub fn path(&self, id: usize) -> Option<Vec<Move>> {
        self.paths
            .get(id.checked_sub(1)?)
            .map(|&(value, _)| unpack(value, self.length))
    }
    pub fn page(&self, game: &Game, filter: &ExplorerFilter) -> Result<ExplorerPage, &'static str> {
        if filter.positions.len() > self.length
            || filter
                .positions
                .iter()
                .flatten()
                .any(|&(r, c)| r > 7 || c > 7)
            || filter.order.iter().enumerate().any(|(i, s)| {
                filter.order[..i].contains(s) || !game.pieces.iter().any(|p| p.slot == *s)
            })
            || filter.order.len() > self.length
            || filter.page_size == 0
            || filter.page_size > 100
            || filter.order.iter().any(|&s| s > 2)
        {
            return Err("Invalid explorer filter");
        }
        let matches = |path: &[Move], upto: usize, skip: Option<usize>| {
            (0..upto).all(|i| {
                game.pieces[usize::from(path[i].piece)].slot == filter.order[i]
                    && (skip == Some(i)
                        || filter
                            .positions
                            .get(i)
                            .is_none_or(|p| p.is_none_or(|p| p == (path[i].row, path[i].column))))
            })
        };
        let mut filtered = Vec::new();
        let mut next_counts = [0; 3];
        let mut options = vec![Vec::new(); filter.order.len()];
        for &ordinal in &self.ordered {
            let path = unpack(self.paths[ordinal].0, self.length);
            for (i, group) in options.iter_mut().enumerate() {
                if matches(&path, i + 1, Some(i)) {
                    let p = (path[i].row, path[i].column);
                    if !group.contains(&p) {
                        group.push(p);
                    }
                }
            }
            if !matches(&path, filter.order.len(), None) {
                continue;
            }
            filtered.push(ordinal);
            if filter.order.len() < self.length {
                next_counts
                    [usize::from(game.pieces[usize::from(path[filter.order.len()].piece)].slot)] +=
                    1;
            }
        }
        for group in &mut options {
            group.sort_unstable();
        }
        let pages = filtered.len().div_ceil(filter.page_size).max(1);
        let page = filter.page.min(pages - 1);
        let mut entries = Vec::new();
        for &ordinal in filtered
            .iter()
            .skip(page * filter.page_size)
            .take(filter.page_size)
        {
            let path = unpack(self.paths[ordinal].0, self.length);
            let (_, steps) = replay(game, &path)?;
            entries.push(ExplorerEntry {
                id: ordinal + 1,
                path,
                score: self
                    .complete
                    .then_some(self.finals[self.paths[ordinal].1].1.value),
                clears: steps.iter().map(|s| s.result.lines()).sum(),
            });
        }
        Ok(ExplorerPage {
            entries,
            total: filtered.len(),
            all_total: self.paths.len(),
            page,
            pages,
            complete: self.complete,
            next_counts,
            position_options: options,
        })
    }
}
