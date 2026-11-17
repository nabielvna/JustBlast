use super::{
    profile::Profile,
    vision::{Slot, Tray},
};
use justblast_core::{
    domain::{Board, Shape},
    solver::{Game, SearchResult},
};
use std::collections::HashMap;
pub const CLEAR_SETTLE_SECONDS: f64 = 3.2;
pub fn needs_clear_wait(result: &justblast_core::domain::MoveResult) -> bool {
    result.lines() > 0 && result.board.bits() == 0
}
#[derive(Clone, Debug)]
pub struct TouchAction {
    pub kind: &'static str,
    pub x: i32,
    pub y: i32,
}
fn recoverable(
    board: Board,
    remaining: &[Shape],
    cache: &mut HashMap<(u64, Vec<Shape>), bool>,
) -> bool {
    if remaining.is_empty() {
        return true;
    }
    let key = (board.bits(), remaining.to_vec());
    if let Some(&v) = cache.get(&key) {
        return v;
    }
    for (index, shape) in remaining.iter().enumerate() {
        let rest: Vec<_> = remaining
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != index)
            .map(|(_, s)| s.clone())
            .collect();
        for (row, col, mask) in shape.placements() {
            if board.bits() & mask != 0 {
                continue;
            }
            if let Some(after) = board.place(shape, row, col)
                && recoverable(after.board, &rest, cache)
            {
                cache.insert(key, true);
                return true;
            }
        }
    }
    cache.insert(key, false);
    false
}
pub fn prepare(
    game: &Game,
    plan: &SearchResult,
    tray: &Tray,
    profile: &Profile,
) -> Result<(Vec<TouchAction>, Vec<f64>), String> {
    prepare_range(game, plan, tray, profile, 0..plan.steps.len())
}
pub fn prepare_range(
    game: &Game,
    plan: &SearchResult,
    tray: &Tray,
    profile: &Profile,
    range: std::ops::Range<usize>,
) -> Result<(Vec<TouchAction>, Vec<f64>), String> {
    if range.start >= range.end || range.end > plan.steps.len() {
        return Err("Invalid touch range".to_owned());
    }
    let mut actions = Vec::new();
    let mut delays = Vec::new();
    let mut cache = HashMap::new();
    for (index, step) in plan.steps.iter().enumerate() {
        let shape = &game
            .pieces
            .iter()
            .find(|p| p.slot == step.slot)
            .ok_or("Unknown planned slot")?
            .shape;
        if range.contains(&index) {
            let Slot::Recognized(piece) = &tray.slots[usize::from(step.slot)] else {
                return Err("Plan refers to an unreadable or empty tray slot".to_owned());
            };
            if piece.shape != *shape {
                return Err("Tray geometry changed. Scan again.".to_owned());
            }
            actions.extend(drag_piece(
                piece,
                step.movement.row,
                step.movement.column,
                profile,
            )?);
        }
        let remaining: Vec<_> = plan.steps[index + 1..]
            .iter()
            .map(|s| {
                game.pieces
                    .iter()
                    .find(|p| p.slot == s.slot)
                    .map(|p| p.shape.clone())
                    .ok_or("Unknown planned slot")
            })
            .collect::<Result<Vec<_>, _>>()?;
        let (mut wrong, mut fatal) = (0, 0);
        for dr in -1..=1 {
            for dc in -1..=1 {
                if dr == 0 && dc == 0 {
                    continue;
                }
                let r = i32::from(step.movement.row) + dr;
                let c = i32::from(step.movement.column) + dc;
                if !(0..8).contains(&r) || !(0..8).contains(&c) {
                    continue;
                }
                if let Some(result) = step.before.place(shape, r as u8, c as u8) {
                    wrong += 1;
                    if !recoverable(result.board, &remaining, &mut cache) {
                        fatal += 1;
                    }
                }
            }
        }
        let risk = if wrong == 0 {
            0.0
        } else {
            (4.0 * f64::from(fatal) / f64::from(wrong)).ceil() / 4.0
        };
        for base in [profile.touch.hold, profile.touch.drop, profile.touch.gap] {
            delays.push(
                ((base + (profile.touch.risk_max - base) * risk) * 1_000_000.0).round()
                    / 1_000_000.0,
            );
        }
        if needs_clear_wait(&step.result) {
            *delays.last_mut().expect("Touch phases were added") += CLEAR_SETTLE_SECONDS;
        }
    }
    Ok((actions, delays[range.start * 3..range.end * 3].to_vec()))
}

pub fn drag_piece(
    piece: &super::vision::ObservedPiece,
    row_target: u8,
    column: u8,
    profile: &Profile,
) -> Result<Vec<TouchAction>, String> {
    let shape = &piece.shape;
    let mut actions = Vec::new();
    let [x, y, w, h] = piece.bbox;
    let &(row, col) = shape
        .cells()
        .iter()
        .min_by(|&&(r, c), &&(rr, cc)| {
            let value = |r: u8, c: u8| {
                (f64::from(r) + 0.5 - f64::from(shape.height()) / 2.0).powi(2)
                    + (f64::from(c) + 0.5 - f64::from(shape.width()) / 2.0).powi(2)
            };
            value(r, c).total_cmp(&value(rr, cc))
        })
        .ok_or("Empty shape")?;
    let start = [
        f64::from(x) + (f64::from(col) + 0.5) * profile.screen.preview_pitch,
        f64::from(y) + (f64::from(row) + 0.5) * profile.screen.preview_pitch,
    ];
    let centre = [
        f64::from(x) + f64::from(w) / 2.0,
        f64::from(y) + f64::from(h) / 2.0,
    ];
    let [bx, by, bw, bh] = profile.screen.board;
    let target = [
        f64::from(bx) + (f64::from(column) + f64::from(shape.width()) / 2.0) * f64::from(bw) / 8.0,
        f64::from(by)
            + (f64::from(row_target) + f64::from(shape.height()) / 2.0) * f64::from(bh) / 8.0,
    ];
    let end = [
        start[0] + (target[0] - centre[0]) / profile.touch.gain,
        start[1] + (target[1] - (centre[1] - profile.touch.lift)) / profile.touch.gain,
    ];
    for (kind, point) in [("DOWN", start), ("MOVE", end), ("UP", end)] {
        let x = point[0].round_ties_even() as i32;
        let y = point[1].round_ties_even() as i32;
        if x < 0 || y < 0 || x >= profile.screen.width as i32 || y >= profile.screen.height as i32 {
            return Err("Drag leaves calibrated screen".to_owned());
        }
        actions.push(TouchAction { kind, x, y });
    }
    Ok(actions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settling_delay_is_only_for_all_clear_not_ordinary_breaks() {
        let single = Shape::new(vec![(0, 0)]).unwrap();
        let all_clear = Board::from_bits(0x7f).place(&single, 0, 7).unwrap();
        assert!(needs_clear_wait(&all_clear));
        let ordinary_break = Board::from_bits(0x7f | (1 << 16))
            .place(&single, 0, 7)
            .unwrap();
        assert_eq!(ordinary_break.lines(), 1);
        assert!(!needs_clear_wait(&ordinary_break));
        let no_break = Board::from_bits(0).place(&single, 0, 0).unwrap();
        assert!(!needs_clear_wait(&no_break));
    }
}
