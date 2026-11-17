use std::fmt;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Shape {
    cells: Vec<(u8, u8)>,
    width: u8,
    height: u8,
    mask: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidShape;
impl fmt::Display for InvalidShape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Shape cells must be unique, within 0..7, and start at row/column zero")
    }
}
impl std::error::Error for InvalidShape {}

impl Shape {
    pub fn new(mut cells: Vec<(u8, u8)>) -> Result<Self, InvalidShape> {
        cells.sort_unstable();
        if cells.is_empty()
            || cells.iter().any(|&(r, c)| r >= 8 || c >= 8)
            || cells.windows(2).any(|w| w[0] == w[1])
            || cells.iter().map(|c| c.0).min() != Some(0)
            || cells.iter().map(|c| c.1).min() != Some(0)
        {
            return Err(InvalidShape);
        }
        let width = cells.iter().map(|c| c.1).max().ok_or(InvalidShape)? + 1;
        let height = cells.iter().map(|c| c.0).max().ok_or(InvalidShape)? + 1;
        let mask = cells
            .iter()
            .fold(0, |mask, &(r, c)| mask | (1u64 << (r * 8 + c)));
        Ok(Self {
            cells,
            width,
            height,
            mask,
        })
    }
    pub fn cells(&self) -> &[(u8, u8)] {
        &self.cells
    }
    pub fn width(&self) -> u8 {
        self.width
    }
    pub fn height(&self) -> u8 {
        self.height
    }
    pub fn mask_at(&self, row: u8, column: u8) -> Option<u64> {
        if row > 8 - self.height || column > 8 - self.width {
            return None;
        }
        Some(self.mask << (row * 8 + column))
    }
    pub fn placements(&self) -> Vec<(u8, u8, u64)> {
        (0..=8 - self.height)
            .flat_map(|r| (0..=8 - self.width).map(move |c| (r, c, self.mask << (r * 8 + c))))
            .collect()
    }
}
