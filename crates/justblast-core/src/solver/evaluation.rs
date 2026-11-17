use crate::domain::Shape;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Geometry {
    pub coverage: u32,
    pub capped_positions: u32,
    pub total_positions: u32,
    pub flexibility: f64,
    pub fragmentation: f64,
}

pub struct Evaluator {
    masks: Vec<Vec<u64>>,
}
impl Evaluator {
    pub fn new(shapes: &[Shape]) -> Result<Self, &'static str> {
        let mut unique = Vec::new();
        for shape in shapes {
            if !unique.contains(&shape) {
                unique.push(shape);
            }
        }
        if unique.is_empty() {
            return Err("Evaluation catalogue must not be empty");
        }
        Ok(Self {
            masks: unique
                .iter()
                .map(|s| s.placements().into_iter().map(|p| p.2).collect())
                .collect(),
        })
    }
    pub fn evaluate(&self, bits: u64) -> Geometry {
        let mut coverage = 0;
        let mut capped_positions = 0;
        let mut total_positions = 0;
        for group in &self.masks {
            let count = group.iter().filter(|&&mask| mask & bits == 0).count() as u32;
            coverage += u32::from(count > 0);
            capped_positions += count.min(4);
            total_positions += count;
        }
        Geometry {
            coverage,
            capped_positions,
            total_positions,
            flexibility: (0.7 * f64::from(coverage) + 0.3 * f64::from(capped_positions) / 4.0)
                / self.masks.len() as f64,
            fragmentation: fragmentation(bits),
        }
    }
}
pub fn fragmentation(bits: u64) -> f64 {
    let mut remaining = !bits;
    let empty = remaining.count_ones();
    if empty == 0 {
        return 1.0;
    }
    let mut small = 0;
    while remaining != 0 {
        let mut component = 1u64 << remaining.trailing_zeros();
        loop {
            let neighbors = ((component & !0x8080808080808080) << 1)
                | ((component & !0x0101010101010101) >> 1)
                | (component << 8)
                | (component >> 8);
            let expanded = component | (neighbors & remaining);
            if component == expanded {
                break;
            }
            component = expanded;
        }
        let size = component.count_ones();
        if size <= 3 {
            small += size;
        }
        remaining &= !component;
    }
    f64::from(small) / f64::from(empty)
}
