use std::fmt;

use super::Shape;

pub const SIZE: usize = 8;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Board(u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidBoard;

impl fmt::Display for InvalidBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Provide eight rows of eight '.' or '#' cells")
    }
}

impl std::error::Error for InvalidBoard {}

impl Board {
    pub fn from_rows(rows: &[String]) -> Result<Self, InvalidBoard> {
        if rows.len() != SIZE {
            return Err(InvalidBoard);
        }
        let mut bits = 0;
        for (row, line) in rows.iter().enumerate() {
            if line.len() != SIZE {
                return Err(InvalidBoard);
            }
            for (column, cell) in line.bytes().enumerate() {
                match cell {
                    b'#' => bits |= 1 << (row * SIZE + column),
                    b'.' => {}
                    _ => return Err(InvalidBoard),
                }
            }
        }
        Ok(Self(bits))
    }

    pub fn rows(self) -> [String; SIZE] {
        std::array::from_fn(|row| {
            (0..SIZE)
                .map(|column| {
                    if self.0 & (1 << (row * SIZE + column)) == 0 {
                        '.'
                    } else {
                        '#'
                    }
                })
                .collect()
        })
    }

    pub fn occupied(self) -> u32 {
        self.0.count_ones()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MoveResult {
    pub board: Board,
    pub placed: Board,
    pub placement_mask: u64,
    pub cleared_rows: Vec<u8>,
    pub cleared_columns: Vec<u8>,
    pub removed_mask: u64,
}
impl MoveResult {
    pub fn lines(&self) -> u32 {
        (self.cleared_rows.len() + self.cleared_columns.len()) as u32
    }
}
impl Board {
    pub fn from_bits(bits: u64) -> Self {
        Self(bits)
    }
    pub fn bits(self) -> u64 {
        self.0
    }
    pub fn can_place(self, shape: &Shape, row: u8, column: u8) -> bool {
        shape
            .mask_at(row, column)
            .is_some_and(|mask| self.0 & mask == 0)
    }
    pub fn place(self, shape: &Shape, row: u8, column: u8) -> Option<MoveResult> {
        let mask = shape.mask_at(row, column)?;
        if self.0 & mask != 0 {
            return None;
        }
        let placed = self.0 | mask;
        let mut removed = 0;
        let mut rows = Vec::new();
        let mut columns = Vec::new();
        for i in 0..8 {
            let row_mask = 0xffu64 << (i * 8);
            let col_mask = 0x0101010101010101u64 << i;
            if placed & row_mask == row_mask {
                rows.push(i);
                removed |= row_mask;
            }
            if placed & col_mask == col_mask {
                columns.push(i);
                removed |= col_mask;
            }
        }
        Some(MoveResult {
            board: Self(placed & !removed),
            placed: Self(placed),
            placement_mask: mask,
            cleared_rows: rows,
            cleared_columns: columns,
            removed_mask: removed,
        })
    }
}
