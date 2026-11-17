use super::profile::Screen;
use justblast_core::domain::{Board, Shape, catalog};
use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct Image {
    pub width: usize,
    pub height: usize,
    pub rgb: Vec<[u8; 3]>,
}
impl Image {
    pub fn new(width: usize, height: usize, rgb: Vec<[u8; 3]>) -> Result<Self, String> {
        if width == 0
            || height == 0
            || width > 8192
            || height > 8192
            || width.checked_mul(height) != Some(rgb.len())
            || rgb.len() > 6_250_000
        {
            return Err("Invalid frame dimensions".to_owned());
        }
        Ok(Self { width, height, rgb })
    }
    pub fn crop(&self, rect: [u32; 4]) -> Result<Self, String> {
        let [x, y, w, h] = rect.map(|v| v as usize);
        if x + w > self.width || y + h > self.height {
            return Err("Crop outside frame".to_owned());
        }
        Self::new(
            w,
            h,
            (y..y + h)
                .flat_map(|r| {
                    self.rgb[r * self.width + x..r * self.width + x + w]
                        .iter()
                        .copied()
                })
                .collect(),
        )
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct ObservedPiece {
    pub slot: u8,
    pub catalog_id: u32,
    pub shape: Shape,
    pub bbox: [i32; 4],
    pub quality: f64,
}
#[derive(Clone, Debug, PartialEq)]
pub enum Slot {
    Empty,
    Recognized(ObservedPiece),
    Uncertain {
        reason: String,
        candidate: Option<ObservedPiece>,
    },
}
impl Slot {
    pub fn candidate(&self) -> Option<&ObservedPiece> {
        match self {
            Self::Recognized(p)
            | Self::Uncertain {
                candidate: Some(p), ..
            } => Some(p),
            _ => None,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SlotSignature {
    Empty,
    Shape(Shape),
    Unknown,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Tray {
    pub slots: [Slot; 3],
}
impl Tray {
    pub fn readable(&self) -> bool {
        self.slots
            .iter()
            .all(|s| !matches!(s, Slot::Uncertain { .. }))
    }
    pub fn complete(&self) -> bool {
        self.slots.iter().all(|s| matches!(s, Slot::Recognized(_)))
    }
    pub fn pieces(&self) -> Vec<ObservedPiece> {
        self.slots
            .iter()
            .filter_map(|s| {
                if let Slot::Recognized(p) = s {
                    Some(p.clone())
                } else {
                    None
                }
            })
            .collect()
    }
    pub fn editable_pieces(&self) -> Vec<ObservedPiece> {
        self.slots
            .iter()
            .filter_map(|s| s.candidate().cloned())
            .collect()
    }
    pub fn signature(&self) -> Vec<SlotSignature> {
        self.slots
            .iter()
            .map(|s| match s {
                Slot::Empty => SlotSignature::Empty,
                Slot::Recognized(p) => SlotSignature::Shape(p.shape.clone()),
                Slot::Uncertain { .. } => SlotSignature::Unknown,
            })
            .collect()
    }
}
#[derive(Clone, Debug)]
pub struct Observation {
    pub board: Board,
    pub tray: Tray,
}
#[derive(Clone)]
struct Lab {
    width: usize,
    height: usize,
    pixels: Vec<[u8; 3]>,
}
fn percentile(values: &mut [f64], percent: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.sort_unstable_by(f64::total_cmp);
    let point = (values.len() - 1) as f64 * percent;
    let low = point.floor() as usize;
    let high = point.ceil() as usize;
    values[low] + (values[high] - values[low]) * (point - low as f64)
}
fn median(values: impl Iterator<Item = f64>) -> f64 {
    percentile(&mut values.collect::<Vec<_>>(), 0.5)
}
fn colour_median(values: &[[f64; 3]]) -> [f64; 3] {
    std::array::from_fn(|i| median(values.iter().map(|v| v[i])))
}
fn distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    (0..3).map(|i| (a[i] - b[i]).abs()).fold(0.0, f64::max)
}
impl Lab {
    fn from_image(image: &Image) -> Self {
        let gamma: [f64; 256] = std::array::from_fn(|i| {
            let x = i as f64 / 255.0;
            if x <= 0.04045 {
                x / 12.92
            } else {
                ((x + 0.055) / 1.055).powf(2.4)
            }
        });
        let f = |x: f64| {
            if x > 0.008856 {
                x.cbrt()
            } else {
                7.787 * x + 16.0 / 116.0
            }
        };
        let pixels = image
            .rgb
            .iter()
            .map(|p| {
                let r = gamma[usize::from(p[0])];
                let g = gamma[usize::from(p[1])];
                let b = gamma[usize::from(p[2])];
                let x = f((0.412453 * r + 0.357580 * g + 0.180423 * b) / 0.950456);
                let y = f(0.212671 * r + 0.715160 * g + 0.072169 * b);
                let z = f((0.019334 * r + 0.119193 * g + 0.950227 * b) / 1.088754);
                [
                    ((116.0 * y - 16.0) * 2.55).round().clamp(0.0, 255.0) as u8,
                    (500.0 * (x - y) + 128.0).round().clamp(0.0, 255.0) as u8,
                    (200.0 * (y - z) + 128.0).round().clamp(0.0, 255.0) as u8,
                ]
            })
            .collect();
        Self {
            width: image.width,
            height: image.height,
            pixels,
        }
    }
    fn at(&self, x: usize, y: usize) -> [f64; 3] {
        self.pixels[y * self.width + x].map(f64::from)
    }
    fn crop(&self, rect: [u32; 4]) -> Result<Self, String> {
        let [x, y, w, h] = rect.map(|v| v as usize);
        if x + w > self.width || y + h > self.height {
            return Err("Calibration outside frame".to_owned());
        }
        Ok(Self {
            width: w,
            height: h,
            pixels: (y..y + h)
                .flat_map(|r| {
                    self.pixels[r * self.width + x..r * self.width + x + w]
                        .iter()
                        .copied()
                })
                .collect(),
        })
    }
    fn resized(&self, width: usize, height: usize) -> Self {
        let mut pixels = Vec::with_capacity(width * height);
        for y in 0..height {
            for x in 0..width {
                let xx = ((x as f64 + 0.5) * self.width as f64 / width as f64 - 0.5).max(0.0);
                let yy = ((y as f64 + 0.5) * self.height as f64 / height as f64 - 0.5).max(0.0);
                let x0 = (xx.floor() as usize).min(self.width - 1);
                let y0 = (yy.floor() as usize).min(self.height - 1);
                let x1 = (x0 + 1).min(self.width - 1);
                let y1 = (y0 + 1).min(self.height - 1);
                let fx = xx - xx.floor();
                let fy = yy - yy.floor();
                pixels.push(std::array::from_fn(|c| {
                    ((self.at(x0, y0)[c] * (1.0 - fx) + self.at(x1, y0)[c] * fx) * (1.0 - fy)
                        + (self.at(x0, y1)[c] * (1.0 - fx) + self.at(x1, y1)[c] * fx) * fy)
                        .round() as u8
                }));
            }
        }
        Self {
            width,
            height,
            pixels,
        }
    }
}
fn foreground(lab: &Lab) -> Vec<bool> {
    let margin = 32.min(lab.height / 10).max(2);
    let w = lab.width;
    let h = lab.height;
    let top: Vec<[f64; 3]> = (0..w)
        .map(|x| std::array::from_fn(|c| median((0..margin).map(|y| lab.at(x, y)[c]))))
        .collect();
    let bottom: Vec<[f64; 3]> = (0..w)
        .map(|x| std::array::from_fn(|c| median((h - margin..h).map(|y| lab.at(x, y)[c]))))
        .collect();
    let mut kernel: Vec<f64> = (-7..=7)
        .map(|i| (-(f64::from(i) * f64::from(i)) / (2.0 * 2.6 * 2.6)).exp())
        .collect();
    let sum: f64 = kernel.iter().sum();
    for v in &mut kernel {
        *v /= sum;
    }
    let smooth = |values: &[[f64; 3]]| -> Vec<[f64; 3]> {
        (0..w)
            .map(|x| {
                std::array::from_fn(|c| {
                    kernel
                        .iter()
                        .enumerate()
                        .map(|(i, k)| {
                            let mut xx = x as isize + i as isize - 7;
                            if xx < 0 {
                                xx = -xx;
                            }
                            if xx >= w as isize {
                                xx = 2 * w as isize - xx - 2;
                            }
                            values[xx.clamp(0, w as isize - 1) as usize][c] * k
                        })
                        .sum()
                })
            })
            .collect()
    };
    let top = smooth(&top);
    let bottom = smooth(&bottom);
    let mut differences = Vec::with_capacity(w * h);
    let mut border = Vec::new();
    for y in 0..h {
        let weight = y as f64 / (h - 1) as f64;
        for x in 0..w {
            let background =
                std::array::from_fn(|c| top[x][c] * (1.0 - weight) + bottom[x][c] * weight);
            let d = distance(lab.at(x, y), background);
            differences.push(d);
            if y < margin || y >= h - margin {
                border.push(d);
            }
        }
    }
    let threshold = (percentile(&mut border, 0.95) * 3.0).clamp(18.0, 40.0);
    differences.into_iter().map(|d| d > threshold).collect()
}
#[derive(Clone)]
struct Component {
    pixels: Vec<usize>,
    bounds: [usize; 4],
}
fn components(mask: &[bool], w: usize, h: usize) -> Vec<Component> {
    let mut visited = vec![false; mask.len()];
    let mut result = Vec::new();
    for start in 0..mask.len() {
        if !mask[start] || visited[start] {
            continue;
        }
        visited[start] = true;
        let mut queue = VecDeque::from([start]);
        let mut pixels = Vec::new();
        let (mut xmin, mut ymin, mut xmax, mut ymax) = (start % w, start / w, start % w, start / w);
        while let Some(p) = queue.pop_front() {
            pixels.push(p);
            let x = p % w;
            let y = p / w;
            xmin = xmin.min(x);
            xmax = xmax.max(x);
            ymin = ymin.min(y);
            ymax = ymax.max(y);
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let xx = x as isize + dx;
                    let yy = y as isize + dy;
                    if xx >= 0 && xx < w as isize && yy >= 0 && yy < h as isize {
                        let n = yy as usize * w + xx as usize;
                        if mask[n] && !visited[n] {
                            visited[n] = true;
                            queue.push_back(n);
                        }
                    }
                }
            }
        }
        result.push(Component {
            pixels,
            bounds: [xmin, ymin, xmax - xmin + 1, ymax - ymin + 1],
        });
    }
    result
}
fn edge_maps(lab: &Lab) -> Vec<Vec<bool>> {
    let w = lab.width;
    let h = lab.height;
    let mut combined = vec![false; w * h];
    let mut maps = Vec::new();
    for channel in 0..3 {
        let mut gradients = vec![(0.0f64, 0.0f64, 0.0f64); w * h];
        for y in 1..h - 1 {
            for x in 1..w - 1 {
                let p = |dx: isize, dy: isize| {
                    lab.at((x as isize + dx) as usize, (y as isize + dy) as usize)[channel]
                };
                let gx = p(1, -1) + 2.0 * p(1, 0) + p(1, 1) - p(-1, -1) - 2.0 * p(-1, 0) - p(-1, 1);
                let gy = p(-1, 1) + 2.0 * p(0, 1) + p(1, 1) - p(-1, -1) - 2.0 * p(0, -1) - p(1, -1);
                gradients[y * w + x] = (gx, gy, gx.abs() + gy.abs());
            }
        }
        let mut weak = vec![false; w * h];
        let mut queue = VecDeque::new();
        let mut selected = vec![false; w * h];
        for y in 1..h - 1 {
            for x in 1..w - 1 {
                let i = y * w + x;
                let (gx, gy, m) = gradients[i];
                if m <= 10.0 {
                    continue;
                }
                let angle = gy.atan2(gx).to_degrees().rem_euclid(180.0);
                let (a, b) = if !(22.5..157.5).contains(&angle) {
                    (i - 1, i + 1)
                } else if angle < 67.5 {
                    (i - w - 1, i + w + 1)
                } else if angle < 112.5 {
                    (i - w, i + w)
                } else {
                    (i - w + 1, i + w - 1)
                };
                if m >= gradients[a].2 && m >= gradients[b].2 {
                    weak[i] = true;
                    if m > 25.0 {
                        selected[i] = true;
                        queue.push_back(i);
                    }
                }
            }
        }
        while let Some(i) = queue.pop_front() {
            let x = i % w;
            let y = i / w;
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let xx = x as isize + dx;
                    let yy = y as isize + dy;
                    if xx > 0 && xx < (w - 1) as isize && yy > 0 && yy < (h - 1) as isize {
                        let n = yy as usize * w + xx as usize;
                        if weak[n] && !selected[n] {
                            selected[n] = true;
                            queue.push_back(n);
                        }
                    }
                }
            }
        }
        for (i, &edge) in selected.iter().enumerate() {
            combined[i] |= edge;
        }
        maps.push(selected);
    }
    maps.insert(0, combined);
    maps
}
fn extent(mask: &[bool], w: usize, h: usize, pitch: f64, substantial: bool) -> Option<[f64; 2]> {
    let columns: Vec<usize> = (0..w)
        .filter(|&x| {
            let count = (0..h).filter(|&y| mask[y * w + x]).count();
            if substantial {
                count as f64 >= 0.5 * pitch
            } else {
                count > 0
            }
        })
        .collect();
    let rows: Vec<usize> = (0..h)
        .filter(|&y| {
            let count = mask[y * w..(y + 1) * w].iter().filter(|&&v| v).count();
            if substantial {
                count as f64 >= 0.5 * pitch
            } else {
                count > 0
            }
        })
        .collect();
    let x = *columns.first()? as f64;
    let y = *rows.first()? as f64;
    let width = *columns.last()? as f64 - x + 1.0;
    let height = *rows.last()? as f64 - y + 1.0;
    let cols = (width / pitch).round_ties_even();
    let rows = (height / pitch).round_ties_even();
    if !(1.0..=5.0).contains(&cols)
        || !(1.0..=5.0).contains(&rows)
        || (width - cols * pitch)
            .abs()
            .max((height - rows * pitch).abs())
            > 0.3 * pitch
    {
        return None;
    }
    Some([
        x + width / 2.0 - cols * pitch / 2.0 + pitch / 2.0,
        y + height / 2.0 - rows * pitch / 2.0 + pitch / 2.0,
    ])
}
fn border_support(edges: &[bool], w: usize, h: usize, cx: f64, cy: f64, pitch: f64) -> u32 {
    let half = (0.25 * pitch).round_ties_even() as i32;
    let offset = (0.47 * pitch).round_ties_even() as i32;
    let search = (0.13 * pitch).round_ties_even().max(2.0) as i32;
    let cx = cx.round_ties_even() as i32;
    let cy = cy.round_ties_even() as i32;
    let edge = |x: i32, y: i32| {
        x >= 0 && y >= 0 && x < w as i32 && y < h as i32 && edges[y as usize * w + x as usize]
    };
    let mut count = 0;
    for sign in [-1, 1] {
        let x = cx + sign * offset;
        let y = cy + sign * offset;
        let vertical = (-search..=search)
            .map(|dx| (-half..=half).filter(|&dy| edge(x + dx, cy + dy)).count())
            .max()
            .unwrap_or(0) as f64
            / (2 * half + 1) as f64;
        let horizontal = (-search..=search)
            .map(|dy| (-half..=half).filter(|&dx| edge(cx + dx, y + dy)).count())
            .max()
            .unwrap_or(0) as f64
            / (2 * half + 1) as f64;
        count += u32::from(vertical >= 0.3) + u32::from(horizontal >= 0.3);
    }
    count
}
fn occupancy(
    mask: &[bool],
    edges: &[bool],
    w: usize,
    h: usize,
    first: [f64; 2],
    pitch: f64,
    strict: bool,
) -> Option<(Shape, [f64; 2])> {
    let radius = (0.32 * pitch).round_ties_even().max(2.0) as i32;
    let mut sites = Vec::new();
    for row in -6..=6 {
        for col in -6..=6 {
            let cx = first[0] + f64::from(col) * pitch;
            let cy = first[1] + f64::from(row) * pitch;
            if cx < pitch * 0.35
                || cx >= w as f64 - pitch * 0.35
                || cy < pitch * 0.35
                || cy >= h as f64 - pitch * 0.35
            {
                continue;
            }
            let xx = cx.round_ties_even() as i32;
            let yy = cy.round_ties_even() as i32;
            let mut total = 0;
            let mut count = 0;
            for y in yy - radius..=yy + radius {
                for x in xx - radius..=xx + radius {
                    if x >= 0 && y >= 0 && x < w as i32 && y < h as i32 {
                        total += 1;
                        count += u32::from(mask[y as usize * w + x as usize]);
                    }
                }
            }
            let ratio = if total > 0 {
                f64::from(count) / f64::from(total)
            } else {
                0.0
            };
            if !strict {
                if ratio >= 0.3 || (ratio > 0.15 && border_support(edges, w, h, cx, cy, pitch) >= 2)
                {
                    sites.push((row, col));
                }
            } else if ratio >= 0.7 {
                if border_support(edges, w, h, cx, cy, pitch) < 2 {
                    return None;
                }
                sites.push((row, col));
            } else if ratio > 0.15 {
                return None;
            }
        }
    }
    let r0 = sites.iter().map(|c| c.0).min()?;
    let c0 = sites.iter().map(|c| c.1).min()?;
    let cells: Vec<(u8, u8)> = sites
        .iter()
        .map(|&(r, c)| ((r - r0) as u8, (c - c0) as u8))
        .collect();
    if cells.iter().any(|&(r, c)| r >= 5 || c >= 5) {
        return None;
    }
    Some((
        Shape::new(cells).ok()?,
        [
            first[0] + f64::from(c0) * pitch,
            first[1] + f64::from(r0) * pitch,
        ],
    ))
}
fn read_piece(lab: &Lab, slot: u8, screen: &Screen) -> Slot {
    let pitch = screen.preview_pitch;
    let mask = foreground(lab);
    let groups = components(&mask, lab.width, lab.height);
    let valid: Vec<_> = groups
        .iter()
        .filter(|c| c.pixels.len() as f64 >= 0.12 * pitch * pitch)
        .collect();
    let maps = edge_maps(lab);
    let edges = &maps[0];
    let mut squares = Vec::new();
    for edge_map in &maps {
        for invert in [false, true] {
            let mask: Vec<bool> = edge_map.iter().map(|&v| v != invert).collect();
            for component in components(&mask, lab.width, lab.height) {
                let [x, y, w, h] = component.bounds;
                if invert && (x == 0 || y == 0 || x + w == lab.width || y + h == lab.height) {
                    continue;
                }
                if (0.72 * pitch..=1.25 * pitch).contains(&(w as f64))
                    && (0.72 * pitch..=1.25 * pitch).contains(&(h as f64))
                {
                    let area = if invert {
                        component.pixels.len() as f64
                    } else {
                        let points: std::collections::HashSet<_> =
                            component.pixels.iter().copied().collect();
                        let mut outside = std::collections::HashSet::new();
                        let mut queue = VecDeque::new();
                        for yy in y..y + h {
                            for xx in x..x + w {
                                let i = yy * lab.width + xx;
                                if (yy == y || yy == y + h - 1 || xx == x || xx == x + w - 1)
                                    && !points.contains(&i)
                                    && outside.insert(i)
                                {
                                    queue.push_back(i);
                                }
                            }
                        }
                        while let Some(i) = queue.pop_front() {
                            let xx = i % lab.width;
                            let yy = i / lab.width;
                            for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                                let nx = xx as isize + dx;
                                let ny = yy as isize + dy;
                                if nx >= x as isize
                                    && ny >= y as isize
                                    && nx < (x + w) as isize
                                    && ny < (y + h) as isize
                                {
                                    let n = ny as usize * lab.width + nx as usize;
                                    if !points.contains(&n) && outside.insert(n) {
                                        queue.push_back(n);
                                    }
                                }
                            }
                        }
                        (w * h - outside.len()) as f64 - component.pixels.len() as f64 / 2.0
                    };
                    if area >= 0.5 * (w * h) as f64 {
                        squares.push([x as f64 + w as f64 / 2.0, y as f64 + h as f64 / 2.0]);
                    }
                }
            }
        }
    }
    let fragments: Vec<_> = groups
        .iter()
        .filter(|c| c.pixels.len() as f64 >= 0.03 * pitch * pitch)
        .collect();
    if fragments.is_empty() && squares.is_empty() {
        return Slot::Empty;
    }
    let mut foreground = vec![false; mask.len()];
    for group in if valid.is_empty() { &fragments } else { &valid } {
        for &p in &group.pixels {
            foreground[p] = true;
        }
    }
    let mut phases = Vec::new();
    for anchor in &squares {
        let mut selected = Vec::new();
        let mut sites = Vec::new();
        for c in &squares {
            let x = ((c[0] - anchor[0]) / pitch).round_ties_even();
            let y = ((c[1] - anchor[1]) / pitch).round_ties_even();
            let error = (c[0] - (anchor[0] + x * pitch))
                .abs()
                .max((c[1] - (anchor[1] + y * pitch)).abs());
            if error <= 0.13 * pitch {
                selected.push([c[0] - x * pitch, c[1] - y * pitch]);
                sites.push((x as i32, y as i32));
            }
        }
        if let (Some(x0), Some(y0)) = (
            sites.iter().map(|s| s.0).min(),
            sites.iter().map(|s| s.1).min(),
        ) {
            if sites.iter().any(|&(x, y)| x - x0 >= 5 || y - y0 >= 5) {
                continue;
            }
            phases.push([
                median(selected.iter().map(|s| s[0])) + f64::from(x0) * pitch,
                median(selected.iter().map(|s| s[1])) + f64::from(y0) * pitch,
            ]);
        }
    }
    if let Some(phase) = extent(&foreground, lab.width, lab.height, pitch, false) {
        phases.push(phase);
    }
    if phases.is_empty()
        && let Some(phase) = extent(&foreground, lab.width, lab.height, pitch, true)
    {
        phases.push(phase);
    }
    let observed = |shape: &Shape, first: [f64; 2], quality: f64| {
        let [x, y, _, _] = screen.slots[usize::from(slot)];
        ObservedPiece {
            slot,
            catalog_id: 0,
            shape: shape.clone(),
            bbox: [
                (f64::from(x) + first[0] - pitch / 2.0).round_ties_even() as i32,
                (f64::from(y) + first[1] - pitch / 2.0).round_ties_even() as i32,
                (f64::from(shape.width()) * pitch).round_ties_even() as i32,
                (f64::from(shape.height()) * pitch).round_ties_even() as i32,
            ],
            quality,
        }
    };
    let mut draft_phases = phases.clone();
    if let Some(group) = fragments.iter().max_by_key(|c| c.pixels.len()) {
        let [x, y, _, _] = group.bounds;
        draft_phases.push([x as f64 + pitch / 2.0, y as f64 + pitch / 2.0]);
    }
    let draft = || {
        draft_phases
            .iter()
            .copied()
            .filter_map(|phase| {
                occupancy(
                    &foreground,
                    edges,
                    lab.width,
                    lab.height,
                    phase,
                    pitch,
                    false,
                )
            })
            .max_by_key(|(shape, _)| shape.cells().len())
            .map(|(shape, first)| observed(&shape, first, 0.0))
    };
    let uncertain = |reason: &str, candidate: Option<ObservedPiece>| Slot::Uncertain {
        reason: reason.to_owned(),
        candidate,
    };
    let fits: Vec<_> = phases
        .into_iter()
        .filter_map(|phase| {
            occupancy(
                &foreground,
                edges,
                lab.width,
                lab.height,
                phase,
                pitch,
                true,
            )
        })
        .collect();
    let Some((shape, first)) = fits.first() else {
        return uncertain("lattice_cell_or_border_ambiguous", draft());
    };
    if fits.iter().any(|f| f.0 != *shape) {
        return uncertain("conflicting_lattice_phases", draft());
    }
    let margin = (0.13 * pitch).round_ties_even().max(1.0);
    let mut rectangles = Vec::new();
    for &(r, c) in shape.cells() {
        let cx = first[0] + f64::from(c) * pitch;
        let cy = first[1] + f64::from(r) * pitch;
        let left = (cx - pitch / 2.0).round_ties_even();
        let top = (cy - pitch / 2.0).round_ties_even();
        let right = (cx + pitch / 2.0).round_ties_even();
        let bottom = (cy + pitch / 2.0).round_ties_even();
        if left < -0.15 * pitch
            || top < -0.15 * pitch
            || right > lab.width as f64 + 0.15 * pitch
            || bottom > lab.height as f64 + 0.15 * pitch
        {
            return uncertain(
                "piece_touches_slot_boundary",
                Some(observed(shape, *first, 0.0)),
            );
        }
        rectangles.push([
            left.max(0.0) - margin,
            top.max(0.0) - margin,
            right + margin,
            bottom + margin,
        ]);
    }
    let total = foreground.iter().filter(|&&v| v).count();
    let covered = foreground
        .iter()
        .enumerate()
        .filter(|&(i, v)| {
            *v && rectangles.iter().any(|r| {
                let x = (i % lab.width) as f64;
                let y = (i / lab.width) as f64;
                x >= r[0] && y >= r[1] && x <= r[2] && y <= r[3]
            })
        })
        .count();
    let coverage = if total == 0 {
        0.0
    } else {
        covered as f64 / total as f64
    };
    if coverage < 0.85 {
        return uncertain(
            "lattice_foreground_disagreement",
            Some(observed(shape, *first, coverage)),
        );
    }
    let Some((id, _)) = catalog::catalog().into_iter().find(|(_, s)| s == shape) else {
        return uncertain(
            "shape_not_in_catalogue",
            Some(observed(shape, *first, coverage)),
        );
    };
    let [x, y, _, _] = screen.slots[usize::from(slot)];
    Slot::Recognized(ObservedPiece {
        slot,
        catalog_id: id,
        shape: shape.clone(),
        bbox: [
            (f64::from(x) + first[0] - pitch / 2.0).round_ties_even() as i32,
            (f64::from(y) + first[1] - pitch / 2.0).round_ties_even() as i32,
            (f64::from(shape.width()) * pitch).round_ties_even() as i32,
            (f64::from(shape.height()) * pitch).round_ties_even() as i32,
        ],
        quality: coverage,
    })
}
#[derive(Clone, Debug)]
pub struct CellEvidence {
    pub state: char,
    pub surface: f64,
    pub centre: f64,
    pub lower: f64,
    pub foreground: f64,
    pub background: f64,
}
type BoardEvidence = (Vec<CellEvidence>, Vec<Vec<[f64; 3]>>);
fn board_evidence(lab: &Lab, screen: &Screen) -> Result<BoardEvidence, String> {
    let [x, y, w, h] = screen.board;
    let mut joints = [[[0.0; 3]; 9]; 9];
    for (row, line) in joints.iter_mut().enumerate() {
        for (col, joint) in line.iter_mut().enumerate() {
            let xx = (f64::from(x) + col as f64 * f64::from(w) / 8.0).round_ties_even() as usize;
            let yy = (f64::from(y) + row as f64 * f64::from(h) / 8.0).round_ties_even() as usize;
            let xx = xx.min(lab.width - 1);
            let yy = yy.min(lab.height - 1);
            let mut cross = Vec::new();
            for py in yy.saturating_sub(2)..(yy + 3).min(lab.height) {
                cross.push(lab.at(xx, py));
            }
            for px in xx.saturating_sub(2)..(xx + 3).min(lab.width) {
                cross.push(lab.at(px, yy));
            }
            *joint = colour_median(&cross);
        }
    }
    let grid = lab.crop(screen.board)?.resized(384, 384);
    let mut contrasts = Vec::new();
    let mut faces = Vec::new();
    let mut centres = Vec::new();
    for row in 0..8 {
        for col in 0..8 {
            let base = colour_median(&[
                joints[row][col],
                joints[row + 1][col],
                joints[row][col + 1],
                joints[row + 1][col + 1],
            ]);
            let face: Vec<_> = (7..41)
                .flat_map(|r| (7..41).map(move |c| (r, c)))
                .map(|(r, c)| grid.at(col * 48 + c, row * 48 + r))
                .collect();
            let mut distances: Vec<_> = face.iter().map(|&p| distance(p, base)).collect();
            let surface = percentile(&mut distances, 0.5);
            let lower = percentile(&mut distances, 0.1);
            let centre = median(
                (19..29)
                    .flat_map(|r| (19..29).map(move |c| (r, c)))
                    .map(|(r, c)| distance(grid.at(col * 48 + c, row * 48 + r), base)),
            );
            contrasts.push((surface, centre, lower));
            centres.push(colour_median(&face));
            faces.push(face);
        }
    }
    let mut sorted: Vec<_> = contrasts.iter().map(|v| v.0).collect();
    sorted.sort_unstable_by(f64::total_cmp);
    let boundary = sorted
        .windows(2)
        .position(|w| w[1] - w[0] >= 14.0 && w[0] <= 48.0 && w[1] >= 26.0);
    let empty_limit = boundary.map_or(24.0, |i| sorted[i] + 3.0);
    let filled_limit = boundary.map_or(28.0, |i| sorted[i + 1] - 3.0);
    let references: Vec<_> = centres
        .iter()
        .zip(&contrasts)
        .filter(|(_, v)| v.0 <= empty_limit && v.1 <= empty_limit)
        .map(|(c, _)| *c)
        .collect();
    let mut cells = Vec::new();
    for i in 0..64 {
        let (surface, centre, lower) = contrasts[i];
        let mut background = 0.0;
        let mut foreground = 0.0;
        let mut centre_background = 0.0;
        let mut centre_foreground = 0.0;
        if let Some(reference) = references
            .iter()
            .min_by(|&&a, &&b| distance(a, centres[i]).total_cmp(&distance(b, centres[i])))
        {
            let differences: Vec<_> = faces[i].iter().map(|&p| distance(p, *reference)).collect();
            background = differences.iter().filter(|&&d| d <= 18.0).count() as f64
                / differences.len() as f64;
            foreground = 1.0 - background;
            centre_background = (12..22)
                .flat_map(|r| (12..22).map(move |c| (r, c)))
                .filter(|&(r, c)| differences[r * 34 + c] <= 18.0)
                .count() as f64
                / 100.0;
            centre_foreground = 1.0 - centre_background;
        }
        let empty = background >= screen.cell_majority && centre_background >= screen.cell_majority;
        let textured =
            foreground >= screen.cell_majority && centre_foreground >= screen.cell_majority;
        let filled = !empty
            && surface >= filled_limit
            && centre > empty_limit
            && (lower >= empty_limit || textured);
        cells.push(CellEvidence {
            state: if filled {
                '#'
            } else if empty {
                '.'
            } else {
                '?'
            },
            surface,
            centre,
            lower,
            foreground,
            background,
        });
    }
    refine_with_texture(&mut cells, &faces, screen.cell_majority);
    Ok((cells, faces))
}

fn refine_with_texture(cells: &mut [CellEvidence], faces: &[Vec<[f64; 3]>], majority: f64) {
    let texture = |face: &[[f64; 3]]| {
        let mean = face.iter().map(|p| p[0]).sum::<f64>() / face.len() as f64;
        (face.iter().map(|p| (p[0] - mean).powi(2)).sum::<f64>() / face.len() as f64).sqrt()
    };
    let references: Vec<_> = cells
        .iter()
        .zip(faces)
        .filter(|(cell, face)| cell.state == '.' && texture(face) <= 3.0)
        .map(|(_, face)| colour_median(face))
        .collect();
    if references.len() < 3 {
        return;
    }
    let empty = colour_median(&references);
    if references
        .iter()
        .any(|&colour| distance(colour, empty) > 8.0)
    {
        return;
    }
    for (cell, face) in cells.iter_mut().zip(faces) {
        if cell.state != '?' || texture(face) < 6.0 {
            continue;
        }
        let different = |p: &[f64; 3]| distance(*p, empty) > 22.0;
        let surface = face.iter().filter(|p| different(p)).count() as f64 / face.len() as f64;
        let centre = (12..22)
            .flat_map(|r| (12..22).map(move |c| r * 34 + c))
            .filter(|&i| different(&face[i]))
            .count() as f64
            / 100.0;
        if surface >= majority && centre >= majority {
            cell.state = '#';
        }
    }
}
fn board_from_evidence(cells: &[CellEvidence]) -> Result<Board, String> {
    let rows: Vec<String> = cells
        .chunks(8)
        .map(|r| r.iter().map(|c| c.state).collect())
        .collect();
    Board::from_rows(&rows).map_err(|_| {
        format!(
            "Board reading is ambiguous in {} cells",
            cells.iter().filter(|c| c.state == '?').count()
        )
    })
}
pub fn draft_board(cells: &[CellEvidence]) -> Board {
    let rows: Vec<String> = cells
        .chunks(8)
        .map(|row| {
            row.iter()
                .map(|cell| match cell.state {
                    '#' => '#',
                    '.' => '.',
                    _ if cell.foreground > cell.background
                        || (cell.surface >= 26.0 && cell.centre >= 26.0) =>
                    {
                        '#'
                    }
                    _ => '.',
                })
                .collect()
        })
        .collect();
    Board::from_rows(&rows).expect("Board evidence contains an eight by eight grid")
}
fn tray_colour(lab: &Lab, tray: &Tray) -> Option<[f64; 3]> {
    let mut centres = Vec::new();
    let mut spreads = Vec::new();
    for piece in tray.pieces() {
        let [x, y, w, h] = piece.bbox;
        if x < 0 || y < 0 || x + w > lab.width as i32 || y + h > lab.height as i32 {
            continue;
        }
        let px = f64::from(w) / f64::from(piece.shape.width());
        let py = f64::from(h) / f64::from(piece.shape.height());
        for &(r, c) in piece.shape.cells() {
            let x0 = (f64::from(x) + (f64::from(c) + 0.2) * px).round_ties_even() as usize;
            let x1 = (f64::from(x) + (f64::from(c) + 0.8) * px).round_ties_even() as usize;
            let y0 = (f64::from(y) + (f64::from(r) + 0.2) * py).round_ties_even() as usize;
            let y1 = (f64::from(y) + (f64::from(r) + 0.8) * py).round_ties_even() as usize;
            let pixels: Vec<_> = (y0..y1)
                .flat_map(|r| (x0..x1).map(move |c| (c, r)))
                .map(|(x, y)| lab.at(x, y))
                .collect();
            if pixels.is_empty() {
                continue;
            }
            let centre = colour_median(&pixels);
            spreads.push(percentile(
                &mut pixels
                    .iter()
                    .map(|&p| distance(p, centre))
                    .collect::<Vec<_>>(),
                0.9,
            ));
            centres.push(centre);
        }
    }
    if centres.is_empty() {
        return None;
    }
    let colour = colour_median(&centres);
    if centres.iter().any(|&c| distance(c, colour) > 18.0) || spreads.iter().any(|&s| s > 24.0) {
        None
    } else {
        Some(colour)
    }
}
pub fn read_tray(image: &Image, screen: &Screen, origin: (u32, u32)) -> Result<Tray, String> {
    let lab = Lab::from_image(image);
    read_tray_lab(&lab, screen, origin)
}
fn read_tray_lab(lab: &Lab, screen: &Screen, origin: (u32, u32)) -> Result<Tray, String> {
    let mut slots = Vec::new();
    for (slot, &[x, y, w, h]) in screen.slots.iter().enumerate() {
        let x = x.checked_sub(origin.0).ok_or("Invalid tray origin")?;
        let y = y.checked_sub(origin.1).ok_or("Invalid tray origin")?;
        slots.push(read_piece(&lab.crop([x, y, w, h])?, slot as u8, screen));
    }
    Ok(Tray {
        slots: slots.try_into().map_err(|_| "Missing slots")?,
    })
}
pub fn read_board(
    image: &Image,
    screen: &Screen,
) -> Result<(Option<Board>, Vec<CellEvidence>), String> {
    let lab = Lab::from_image(image);
    let (cells, _) = board_evidence(&lab, screen)?;
    Ok((board_from_evidence(&cells).ok(), cells))
}
#[derive(Clone, Debug)]
pub struct RecognitionError {
    pub message: String,
    pub diagnostics: Option<String>,
}
impl From<String> for RecognitionError {
    fn from(message: String) -> Self {
        Self {
            message,
            diagnostics: None,
        }
    }
}
impl std::fmt::Display for RecognitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for RecognitionError {}
pub struct FrameReading {
    pub board: Option<Board>,
    pub cells: Vec<CellEvidence>,
    pub tray: Tray,
}
pub fn read_frame(image: &Image, screen: &Screen) -> Result<FrameReading, String> {
    if image.width != screen.width as usize || image.height != screen.height as usize {
        return Err("Screen resolution does not match calibration".to_owned());
    }
    let lab = Lab::from_image(image);
    let tray = read_tray_lab(&lab, screen, (0, 0))?;
    let (mut cells, faces) = board_evidence(&lab, screen)?;
    if let Some(colour) = tray_colour(&lab, &tray) {
        let empty: Vec<_> = faces
            .iter()
            .zip(&cells)
            .filter(|(_, c)| c.state == '.')
            .map(|(face, _)| colour_median(face))
            .collect();
        let collision = empty.iter().any(|&c| distance(c, colour) <= 24.0);
        for (cell, face) in cells.iter_mut().zip(&faces) {
            let coverage = face
                .iter()
                .filter(|&&p| distance(p, colour) <= 22.0)
                .count() as f64
                / face.len() as f64;
            let centre = (12..22)
                .flat_map(|r| (12..22).map(move |c| (r, c)))
                .filter(|&(r, c)| distance(face[r * 34 + c], colour) <= 22.0)
                .count() as f64
                / 100.0;
            if !collision && coverage >= 0.75 && centre >= 0.75 {
                if cell.surface >= 26.0 && cell.centre >= 26.0 && cell.state != '.' {
                    cell.state = '#';
                } else if cell.state == '.' {
                    cell.state = '?';
                }
            }
        }
    }
    Ok(FrameReading {
        board: board_from_evidence(&cells).ok(),
        cells,
        tray,
    })
}
pub fn read_snapshot(image: &Image, screen: &Screen) -> Result<Observation, RecognitionError> {
    let FrameReading {
        board: _,
        cells,
        tray,
    } = read_frame(image, screen)?;
    if !tray.readable() || tray.pieces().is_empty() {
        let unclear = tray
            .slots
            .iter()
            .enumerate()
            .filter_map(|(slot, status)| {
                matches!(status, Slot::Uncertain { .. }).then_some((slot + 1).to_string())
            })
            .collect::<Vec<_>>();
        return Err(RecognitionError {
            message: if unclear.is_empty() {
                "No pieces were recognized. Open the game, then Scan again.".to_owned()
            } else {
                format!(
                    "Tray slot {} could not be read reliably. Wait for the game to settle or check calibration, then Scan again.",
                    unclear.join(", ")
                )
            },
            diagnostics: Some(format!("{:?}", tray.slots)),
        });
    }
    let board = board_from_evidence(&cells).map_err(|message| RecognitionError {
        message: format!(
            "{message}. Wait for the game to settle or check calibration, then Scan again."
        ),
        diagnostics: Some(format!("cells={cells:?}; tray={:?}", tray.slots)),
    })?;
    Ok(Observation { board, tray })
}

fn calibration_tray_score(lab: &Lab, screen: &Screen, tray: &Tray) -> f64 {
    tray.pieces()
        .iter()
        .map(|piece| {
            let rect = screen.slots[usize::from(piece.slot)];
            let Ok(crop) = lab.crop(rect) else {
                return 0.0;
            };
            let mask = foreground(&crop);
            let groups = components(&mask, crop.width, crop.height);
            let points: Vec<_> = groups
                .iter()
                .filter(|c| c.pixels.len() as f64 > screen.preview_pitch.powi(2) * 0.12)
                .flat_map(|c| c.pixels.iter().copied())
                .collect();
            if points.is_empty() {
                return 0.0;
            }
            let left = points.iter().map(|p| p % crop.width).min().unwrap() as f64;
            let right = points.iter().map(|p| p % crop.width).max().unwrap() as f64 + 1.0;
            let top = points.iter().map(|p| p / crop.width).min().unwrap() as f64;
            let bottom = points.iter().map(|p| p / crop.width).max().unwrap() as f64 + 1.0;
            let [x, y, w, h] = piece.bbox.map(f64::from);
            let error = (x - f64::from(rect[0]) - left).abs()
                + (y - f64::from(rect[1]) - top).abs()
                + (x + w - f64::from(rect[0]) - right).abs()
                + (y + h - f64::from(rect[1]) - bottom).abs();
            10.0 + piece.quality - 2.0 * error / screen.preview_pitch
        })
        .sum()
}

fn board_grid_support(lab: &Lab, [x, y, w, h]: [f64; 4]) -> Option<f64> {
    if x < 0.0 || y < 0.0 || x + w >= lab.width as f64 || y + h >= lab.height as f64 {
        return None;
    }
    let pitch = w / 8.0;
    let radius = (pitch * 0.065).round().max(1.0) as isize;
    let sample = |xx: f64, yy: f64| {
        lab.at(
            xx.round().clamp(0.0, (lab.width - 1) as f64) as usize,
            yy.round().clamp(0.0, (lab.height - 1) as f64) as usize,
        )
    };
    let dark_near = |cx: f64, cy: f64| {
        let mut dark = sample(cx, cy);
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let pixel = sample(cx + dx as f64, cy + dy as f64);
                if pixel[0] < dark[0] {
                    dark = pixel;
                }
            }
        }
        dark
    };
    let mut joints = [[[0.0; 3]; 9]; 9];
    for (r, line) in joints.iter_mut().enumerate() {
        for (c, joint) in line.iter_mut().enumerate() {
            *joint = dark_near(x + c as f64 * w / 8.0, y + r as f64 * h / 8.0);
        }
    }
    let mut rows = [0usize; 8];
    let mut cols = [0usize; 8];
    let mut quality = 0.0;
    for r in 0..8 {
        for (c, col) in cols.iter_mut().enumerate() {
            let centre = sample(
                x + (c as f64 + 0.5) * w / 8.0,
                y + (r as f64 + 0.5) * h / 8.0,
            );
            let base = colour_median(&[
                joints[r][c],
                joints[r][c + 1],
                joints[r + 1][c],
                joints[r + 1][c + 1],
            ]);
            let horizontal = colour_median(&[
                dark_near(x + c as f64 * w / 8.0, y + (r as f64 + 0.5) * h / 8.0),
                dark_near(
                    x + (c as f64 + 1.0) * w / 8.0,
                    y + (r as f64 + 0.5) * h / 8.0,
                ),
            ]);
            let vertical = colour_median(&[
                dark_near(x + (c as f64 + 0.5) * w / 8.0, y + r as f64 * h / 8.0),
                dark_near(
                    x + (c as f64 + 0.5) * w / 8.0,
                    y + (r as f64 + 1.0) * h / 8.0,
                ),
            ]);
            let contrast = distance(centre, base)
                .min(distance(centre, horizontal))
                .min(distance(centre, vertical));
            if contrast >= 2.0 {
                rows[r] += 1;
                *col += 1;
            }
            let edge_points = [
                (x + c as f64 * w / 8.0, y + (r as f64 + 0.5) * h / 8.0),
                (
                    x + (c as f64 + 1.0) * w / 8.0,
                    y + (r as f64 + 0.5) * h / 8.0,
                ),
                (x + (c as f64 + 0.5) * w / 8.0, y + r as f64 * h / 8.0),
                (
                    x + (c as f64 + 0.5) * w / 8.0,
                    y + (r as f64 + 1.0) * h / 8.0,
                ),
            ];
            let alignment = edge_points
                .into_iter()
                .map(|(cx, cy)| (distance(sample(cx, cy), dark_near(cx, cy)) / 12.0).min(1.0))
                .sum::<f64>()
                / 4.0;
            quality += ((contrast - 1.0) / 8.0).clamp(0.0, 1.0) - alignment;
        }
    }
    let weakest = *rows.iter().chain(&cols).min()?;
    if weakest < 4 {
        return None;
    }
    Some(quality / 64.0 + weakest as f64 / 8.0)
}

fn locate_board_lattice(
    lab: &Lab,
    squares: &[([f64; 2], f64)],
    frames: &[[f64; 4]],
) -> Option<[f64; 4]> {
    let board_cells: Vec<_> = squares
        .iter()
        .filter(|(p, w)| *w >= 25.0 && p[1] < lab.height as f64 * 0.78)
        .collect();
    let mut proposals = Vec::new();
    for &[x, y, w, h] in frames {
        for inset in 0..=16 {
            let d = inset as f64 / 2.0;
            proposals.push([x + d, y + d, w - 2.0 * d, h - 2.0 * d]);
        }
    }
    for &(anchor, width) in &board_cells {
        for other in &board_cells {
            let pitch = (other.0[0] - anchor[0]).abs();
            if (other.0[1] - anchor[1]).abs() > 3.0
                || !(width * 0.95..=width * 1.22).contains(&pitch)
                || !(28.0..=44.5).contains(&pitch)
            {
                continue;
            }
            let size = pitch * 8.0;
            for col in 0..8 {
                let x = anchor[0] - (col as f64 + 0.5) * pitch;
                if x < 0.0
                    || x + size > lab.width as f64
                    || (x + size / 2.0 - lab.width as f64 / 2.0).abs() > 18.0
                {
                    continue;
                }
                for row in 0..8 {
                    let y = anchor[1] - (row as f64 + 0.5) * pitch;
                    if y < lab.height as f64 * 0.1 || y + size > lab.height as f64 * 0.83 {
                        continue;
                    }
                    proposals.push([x, y, size, size]);
                }
            }
        }
    }
    let mut seen = std::collections::HashSet::new();
    let mut best: Option<(f64, [f64; 4])> = None;
    for rect in proposals {
        if !seen.insert(rect.map(|v| (v * 2.0).round() as i32)) {
            continue;
        }
        let Some(support) = board_grid_support(lab, rect) else {
            continue;
        };
        let [x, y, w, h] = rect;
        let count = board_cells
            .iter()
            .filter(|(p, _)| {
                let c = (p[0] - x) * 8.0 / w - 0.5;
                let r = (p[1] - y) * 8.0 / h - 0.5;
                (-0.08..=7.08).contains(&c)
                    && (-0.08..=7.08).contains(&r)
                    && (c - c.round()).abs() < 0.08
                    && (r - r.round()).abs() < 0.08
            })
            .count();
        let score = support + count as f64 / 640.0;
        if best.as_ref().is_none_or(|b| score > b.0) {
            best = Some((score, rect));
        }
    }
    best.map(|(_, rect)| rect)
}

struct LayoutEvidence {
    scale: f64,
    lab: Lab,
    squares: Vec<([f64; 2], f64)>,
    board: Option<[f64; 4]>,
}
fn calibration_layout_evidence(image: &Image) -> Result<LayoutEvidence, String> {
    let scale = image.width as f64 / 360.0;
    let small = Image::new(
        360,
        (image.height as f64 / scale).round() as usize,
        (0..(image.height as f64 / scale).round() as usize)
            .flat_map(|y| {
                (0..360).map(move |x| {
                    image.rgb[((y as f64 * scale) as usize).min(image.height - 1) * image.width
                        + ((x as f64 * scale) as usize).min(image.width - 1)]
                })
            })
            .collect(),
    )?;
    let lab = Lab::from_image(&small);
    let mut squares = Vec::new();
    let mut frames = Vec::new();
    for edges in edge_maps(&lab) {
        for c in components(&edges, lab.width, lab.height) {
            let [x, y, w, h] = c.bounds;
            if w as f64 >= lab.width as f64 * 0.7
                && w < lab.width
                && (w as f64 / h as f64 - 1.0).abs() < 0.04
                && y as f64 > lab.height as f64 * 0.1
                && ((y + h) as f64) < lab.height as f64 * 0.83
            {
                frames.push([x as f64, y as f64, w as f64, h as f64]);
            }
        }
        let inside: Vec<_> = edges.iter().map(|v| !v).collect();
        for c in components(&inside, lab.width, lab.height) {
            let [x, y, w, h] = c.bounds;
            if x == 0 || y == 0 || x + w == lab.width || y + h == lab.height {
                continue;
            }
            if (8..=48).contains(&w)
                && (w as f64 / h as f64 - 1.0).abs() < 0.16
                && c.pixels.len() as f64 > (w * h) as f64 * 0.65
            {
                let centre = [x as f64 + w as f64 / 2.0, y as f64 + h as f64 / 2.0];
                if !squares.iter().any(|(p, _): &([f64; 2], f64)| {
                    distance([p[0], p[1], 0.0], [centre[0], centre[1], 0.0]) < 3.0
                }) {
                    squares.push((centre, (w + h) as f64 / 2.0));
                }
            }
        }
    }
    let board = locate_board_lattice(&lab, &squares, &frames);
    Ok(LayoutEvidence {
        scale,
        lab,
        squares,
        board,
    })
}

pub fn locate_board(image: &Image) -> Result<Option<[u32; 4]>, String> {
    let LayoutEvidence { scale, board, .. } = calibration_layout_evidence(image)?;
    Ok(board.map(|r| r.map(|v| (v * scale).round() as u32)))
}

pub fn auto_layout(image: &Image, screen: &Screen) -> Result<(Screen, bool), String> {
    let LayoutEvidence {
        scale,
        lab,
        squares,
        board,
    } = calibration_layout_evidence(image)?;
    let mut next = screen.clone();
    let found = board.is_some();
    if let Some(board) = board {
        let located = board.map(|v| (v * scale).round() as u32);
        let close = located
            .iter()
            .zip(screen.board)
            .all(|(&a, b)| a.abs_diff(b) as f64 <= f64::from(screen.board[2]) / 8.0 * 0.08);
        next.board = if close && read_snapshot(image, screen).is_ok() {
            screen.board
        } else {
            located
        };
    }
    let bottom = (next.board[1] + next.board[3]) as f64 / scale;
    let tray_cells: Vec<_> = squares
        .iter()
        .filter(|(p, w)| {
            *w < 27.0
                && p[1] > bottom + 8.0
                && p[1] < (bottom + 220.0).min(lab.height as f64 - 12.0)
        })
        .collect();
    if !tray_cells.is_empty() {
        let cy = median(tray_cells.iter().map(|(p, _)| p[1]));
        let top = ((cy - 85.0).max(bottom + 4.0) * scale).round() as u32;
        let height = ((170.0 * scale).round() as u32).min(screen.height - top);
        let mut candidate = next.clone();
        candidate.slots = std::array::from_fn(|i| {
            let left = i as u32 * screen.width / 3;
            [left, top, (i as u32 + 1) * screen.width / 3 - left, height]
        });
        let mut small_screen = candidate.clone();
        small_screen.width = lab.width as u32;
        small_screen.height = lab.height as u32;
        small_screen.slots = candidate
            .slots
            .map(|r| r.map(|v| (f64::from(v) / scale).floor() as u32));

        let mut hypotheses = Vec::new();
        let cell = f64::from(next.board[2]) / 8.0 / scale;
        for index in 0..=36 {
            let pitch = cell * (0.28 + f64::from(index) * 0.012);
            small_screen.preview_pitch = pitch;
            let tray = read_tray_lab(&lab, &small_screen, (0, 0))?;
            hypotheses.push((
                calibration_tray_score(&lab, &small_screen, &tray),
                pitch * scale,
            ));
        }
        hypotheses.sort_by(|a, b| b.0.total_cmp(&a.0));
        let original_lab = Lab::from_image(image);
        let mut spacings = Vec::new();
        for rect in candidate.slots {
            let crop = original_lab.crop(rect)?;
            let centres: Vec<_> = components(&foreground(&crop), crop.width, crop.height)
                .into_iter()
                .filter(|c| {
                    let [_, _, w, h] = c.bounds;
                    (w as f64 / h as f64 - 1.0).abs() < 0.12
                        && w as f64 > cell * scale * 0.28
                        && (w as f64) < cell * scale * 0.72
                })
                .map(|c| {
                    let [x, y, w, h] = c.bounds;
                    [x as f64 + w as f64 / 2.0, y as f64 + h as f64 / 2.0]
                })
                .collect();
            for a in &centres {
                for b in &centres {
                    let dx = (a[0] - b[0]).abs();
                    let dy = (a[1] - b[1]).abs();
                    if dy < 2.0 && (cell * scale * 0.3..=cell * scale * 0.7).contains(&dx) {
                        spacings.push(dx);
                    }
                    if dx < 2.0 && (cell * scale * 0.3..=cell * scale * 0.7).contains(&dy) {
                        spacings.push(dy);
                    }
                }
            }
        }
        let measured_pitch = (!spacings.is_empty()).then(|| median(spacings.into_iter()));
        let mut native_pitches = measured_pitch.into_iter().collect::<Vec<_>>();
        for &(score, pitch) in hypotheses.iter().take(3) {
            if score == 0.0 {
                continue;
            }
            for offset in [-1.5, 0.0, 1.5] {
                let refined = pitch + offset;
                if !native_pitches
                    .iter()
                    .any(|p: &f64| (*p - refined).abs() < 0.5)
                {
                    native_pitches.push(refined);
                }
            }
        }
        let mut quality = 0.0;
        for pitch in native_pitches {
            candidate.preview_pitch = pitch;
            let tray = read_tray_lab(&original_lab, &candidate, (0, 0))?;
            let quality_score = calibration_tray_score(&original_lab, &candidate, &tray)
                - measured_pitch.map_or(0.0, |p| 8.0 * (candidate.preview_pitch - p).abs() / p);
            if quality_score > quality {
                quality = quality_score;
                next.preview_pitch = candidate.preview_pitch;
                next.slots = candidate.slots;
                for piece in tray.pieces() {
                    let [x, y, w, h] = piece.bbox;
                    let width = (candidate.preview_pitch * 6.0)
                        .round()
                        .min(f64::from(screen.width) / 3.0) as u32;
                    let height = (candidate.preview_pitch * 6.0)
                        .round()
                        .min(f64::from(screen.height)) as u32;
                    let left = (f64::from(x) + f64::from(w) / 2.0 - f64::from(width) / 2.0)
                        .round()
                        .clamp(0.0, f64::from(screen.width - width))
                        as u32;
                    let top = (f64::from(y) + f64::from(h) / 2.0 - f64::from(height) / 2.0)
                        .round()
                        .clamp(0.0, f64::from(screen.height - height))
                        as u32;
                    next.slots[usize::from(piece.slot)] = [left, top, width, height];
                }
            }
        }
    }
    next.validate()?;
    Ok((next, found))
}

#[cfg(test)]
mod texture_tests {
    use super::*;

    fn cell(state: char) -> CellEvidence {
        CellEvidence {
            state,
            surface: 0.0,
            centre: 0.0,
            lower: 0.0,
            foreground: 0.0,
            background: 0.0,
        }
    }

    fn flat(lightness: f64) -> Vec<[f64; 3]> {
        vec![[lightness, 0.0, 0.0]; 34 * 34]
    }

    fn shaded(lightness: f64) -> Vec<[f64; 3]> {
        (0..34 * 34)
            .map(|i| [lightness + if i % 2 == 0 { 8.0 } else { -8.0 }, 0.0, 0.0])
            .collect()
    }

    #[test]
    fn shading_recovers_ambiguous_tile_without_overriding_accepted_cells() {
        let mut cells = vec![cell('.'), cell('.'), cell('.'), cell('?'), cell('.')];
        let faces = vec![
            flat(20.0),
            flat(20.0),
            flat(20.0),
            shaded(60.0),
            shaded(60.0),
        ];
        refine_with_texture(&mut cells, &faces, 0.9);
        assert_eq!(cells.iter().map(|c| c.state).collect::<String>(), "...#.");
    }

    #[test]
    fn flat_coloured_cells_and_texture_matching_empty_colour_remain_ambiguous() {
        let mut cells = vec![cell('.'), cell('.'), cell('.'), cell('?'), cell('?')];
        let faces = vec![flat(20.0), flat(20.0), flat(20.0), flat(60.0), shaded(20.0)];
        refine_with_texture(&mut cells, &faces, 0.9);
        assert_eq!(cells[3].state, '?');
        assert_eq!(cells[4].state, '?');
    }

    #[test]
    fn missing_or_inconsistent_empty_references_never_imply_full_board() {
        for empty_colours in [vec![], vec![20.0, 20.0], vec![20.0, 20.0, 45.0]] {
            let mut cells: Vec<_> = empty_colours.iter().map(|_| cell('.')).collect();
            let mut faces: Vec<_> = empty_colours.into_iter().map(flat).collect();
            cells.push(cell('?'));
            faces.push(shaded(70.0));
            refine_with_texture(&mut cells, &faces, 0.9);
            assert_eq!(cells.last().unwrap().state, '?');
        }
    }

    #[test]
    fn small_centre_mark_does_not_count_as_an_occupied_cell() {
        let mut cells = vec![cell('.'), cell('.'), cell('.'), cell('?')];
        let mut mark = flat(20.0);
        for r in 12..22 {
            for c in 12..22 {
                mark[r * 34 + c] = [70.0, 0.0, 0.0];
            }
        }
        let faces = vec![flat(20.0), flat(20.0), flat(20.0), mark];
        refine_with_texture(&mut cells, &faces, 0.9);
        assert_eq!(cells[3].state, '?');
    }
}
