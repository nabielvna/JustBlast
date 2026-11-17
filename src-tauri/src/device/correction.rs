use super::{
    profile::Screen,
    vision::{self, Image, Observation, ObservedPiece, Slot, SlotSignature, Tray},
};
use justblast_core::domain::{Board, Shape};
use std::sync::Arc;

#[derive(Clone)]
pub struct Capture {
    pub id: u32,
    pub image: Arc<Image>,
    pub board: Option<Board>,
    pub board_draft: Board,
    pub tray: Tray,
    pub crops: Vec<String>,
}
impl Capture {
    pub fn read(id: u32, image: Image, screen: &Screen) -> Result<Self, String> {
        if image.width != screen.width as usize || image.height != screen.height as usize {
            return Err("Screen resolution does not match calibration".to_owned());
        }
        let vision::FrameReading { board, cells, tray } = vision::read_frame(&image, screen)?;
        let board_draft = board.unwrap_or_else(|| vision::draft_board(&cells));
        let crops = screen
            .slots
            .iter()
            .map(|&rect| png_data(&image.crop(rect)?))
            .collect::<Result<Vec<_>, String>>()?;
        Ok(Self {
            id,
            image: Arc::new(image),
            board,
            board_draft,
            tray,
            crops,
        })
    }
    pub fn observation(&self) -> Option<Observation> {
        Some(Observation {
            board: self.board?,
            tray: self.tray.clone(),
        })
    }
}
#[derive(Clone)]
pub struct Corrections {
    pub capture: Capture,
    pub cells: Vec<(u8, u8, bool)>,
    pub slots: [Option<Slot>; 3],
}
pub fn same_region(a: &Image, b: &Image, rect: [u32; 4]) -> bool {
    if a.width != b.width || a.height != b.height {
        return false;
    }
    let [x, y, w, h] = rect.map(|v| v as usize);
    if w == 0 || h == 0 || x + w > a.width || y + h > a.height {
        return false;
    }
    let mut samples = Vec::new();
    for r in 0..24 {
        for c in 0..24 {
            let px = x + w * (4 + c * 2) / 56;
            let py = y + h * (4 + r * 2) / 56;
            let aa = a.rgb[py * a.width + px];
            let bb = b.rgb[py * b.width + px];
            samples.push(std::array::from_fn::<_, 3, _>(|i| {
                i32::from(bb[i]) - i32::from(aa[i])
            }));
        }
    }
    let offset: [i32; 3] = std::array::from_fn(|i| {
        let mut values: Vec<_> = samples.iter().map(|s| s[i]).collect();
        values.sort_unstable();
        values[values.len() / 2]
    });
    if offset.iter().any(|v| v.abs() > 18) {
        return false;
    }
    let changed = samples
        .iter()
        .filter(|s| (0..3).any(|i| (s[i] - offset[i]).abs() > 24))
        .count();
    changed * 1000 <= samples.len() * 8
}
pub fn board_rect(screen: &Screen, row: u8, col: u8) -> [u32; 4] {
    let [x, y, w, h] = screen.board;
    let left = x + u32::from(col) * w / 8;
    let top = y + u32::from(row) * h / 8;
    [
        left,
        top,
        x + (u32::from(col) + 1) * w / 8 - left,
        y + (u32::from(row) + 1) * h / 8 - top,
    ]
}
impl Corrections {
    pub fn resolve(
        &self,
        capture: &Capture,
        screen: &Screen,
        expected: Board,
        signature: Option<&[SlotSignature]>,
    ) -> Result<Observation, String> {
        let mut rows = capture.board.unwrap_or_default().rows();
        if capture.board.is_none()
            && !same_region(&self.capture.image, &capture.image, screen.board)
        {
            return Err("The corrected board image changed. Scan and review it again.".to_owned());
        }
        if capture.board.is_none() {
            rows = expected.rows();
        }
        let expected_rows = expected.rows();
        for &(r, c, filled) in &self.cells {
            let expected_value = expected_rows[usize::from(r)].as_bytes()[usize::from(c)] == b'#';
            if expected_value == filled {
                if capture.board.is_some()
                    && (rows[usize::from(r)].as_bytes()[usize::from(c)] == b'#') == filled
                {
                    continue;
                }
                if !same_region(
                    &self.capture.image,
                    &capture.image,
                    board_rect(screen, r, c),
                ) {
                    return Err(format!(
                        "Corrected cell R{} C{} changed. Scan and review it.",
                        r + 1,
                        c + 1
                    ));
                }
                rows[usize::from(r)].replace_range(
                    usize::from(c)..usize::from(c) + 1,
                    if filled { "#" } else { "." },
                );
            }
        }
        let mut tray = capture.tray.clone();
        for (slot, corrected) in self.slots.iter().enumerate() {
            if let Some(corrected) = corrected {
                let corrected_shape = if let Slot::Recognized(p) = corrected {
                    SlotSignature::Shape(p.shape.clone())
                } else {
                    SlotSignature::Empty
                };
                if signature.is_none_or(|s| s.get(slot) != Some(&corrected_shape)) {
                    continue;
                }
                if tray.signature().get(slot) == Some(&corrected_shape) {
                    continue;
                }
                if !same_region(&self.capture.image, &capture.image, screen.slots[slot]) {
                    return Err(format!(
                        "Corrected slot {} changed. Scan and review it.",
                        slot + 1
                    ));
                }
                tray.slots[slot] = corrected.clone();
            }
        }
        if !tray.readable() {
            return Err("A remaining slot is unreadable. Scan and correct it.".to_owned());
        }
        Ok(Observation {
            board: Board::from_rows(&rows).map_err(|e| format!("{e:?}"))?,
            tray,
        })
    }
}
pub fn mapped_piece(
    capture: &Capture,
    screen: &Screen,
    slot: u8,
    shape: Shape,
    origin: Option<(u32, u32)>,
) -> Result<ObservedPiece, String> {
    map_piece(capture, screen, slot, shape, origin, false)
}
pub fn accepted_piece(
    capture: &Capture,
    screen: &Screen,
    slot: u8,
    shape: Shape,
    origin: Option<(u32, u32)>,
) -> Result<ObservedPiece, String> {
    map_piece(capture, screen, slot, shape, origin, true)
}
fn map_piece(
    capture: &Capture,
    screen: &Screen,
    slot: u8,
    shape: Shape,
    origin: Option<(u32, u32)>,
    accepted: bool,
) -> Result<ObservedPiece, String> {
    let rect = *screen.slots.get(usize::from(slot)).ok_or("Unknown slot")?;
    let pitch = screen.preview_pitch;
    let (x, y) = if let Some((x, y)) = origin {
        if x > rect[2] || y > rect[3] {
            return Err("The grid origin leaves its slot.".to_owned());
        }
        (f64::from(rect[0] + x), f64::from(rect[1] + y))
    } else if let Some(p) = capture.tray.slots[usize::from(slot)].candidate() {
        (f64::from(p.bbox[0]), f64::from(p.bbox[1]))
    } else if accepted {
        (
            f64::from(rect[0]) + (f64::from(rect[2]) - f64::from(shape.width()) * pitch) / 2.0,
            f64::from(rect[1]) + (f64::from(rect[3]) - f64::from(shape.height()) * pitch) / 2.0,
        )
    } else {
        return Err(format!(
            "Set the grid origin on the crop for slot {}.",
            slot + 1
        ));
    };
    let width = f64::from(shape.width()) * pitch;
    let height = f64::from(shape.height()) * pitch;
    if x < f64::from(rect[0])
        || y < f64::from(rect[1])
        || x + width > f64::from(rect[0] + rect[2]) + 2.0
        || y + height > f64::from(rect[1] + rect[3]) + 2.0
    {
        return Err("The corrected piece grid leaves its slot. Adjust its origin.".to_owned());
    }
    let background =
        capture.image.rgb[(rect[1] as usize + 3) * capture.image.width + rect[0] as usize + 3];
    for &(r, c) in shape.cells().iter().filter(|_| !accepted) {
        let px = (x + (f64::from(c) + 0.5) * pitch).round() as usize;
        let py = (y + (f64::from(r) + 0.5) * pitch).round() as usize;
        if px >= capture.image.width || py >= capture.image.height {
            return Err("Invalid touch origin".to_owned());
        }
        let pixel = capture.image.rgb[py * capture.image.width + px];
        if (0..3)
            .map(|i| (i32::from(pixel[i]) - i32::from(background[i])).abs())
            .sum::<i32>()
            < 45
        {
            return Err(
                "A corrected tile has no visual support at this origin. Adjust the grid origin."
                    .to_owned(),
            );
        }
    }
    Ok(ObservedPiece {
        slot,
        catalog_id: 0,
        shape,
        bbox: [
            x.round() as i32,
            y.round() as i32,
            width.round() as i32,
            height.round() as i32,
        ],
        quality: 1.0,
    })
}
pub fn png_data(image: &Image) -> Result<String, String> {
    let mut bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut bytes, image.width as u32, image.height as u32);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
        writer
            .write_image_data(&image.rgb.iter().flatten().copied().collect::<Vec<_>>())
            .map_err(|e| e.to_string())?;
    }
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut value = String::from("data:image/png;base64,");
    for chunk in bytes.chunks(3) {
        let n = (u32::from(chunk[0]) << 16)
            | (u32::from(*chunk.get(1).unwrap_or(&0)) << 8)
            | u32::from(*chunk.get(2).unwrap_or(&0));
        value.push(ALPHABET[((n >> 18) & 63) as usize] as char);
        value.push(ALPHABET[((n >> 12) & 63) as usize] as char);
        value.push(if chunk.len() > 1 {
            ALPHABET[((n >> 6) & 63) as usize] as char
        } else {
            '='
        });
        value.push(if chunk.len() > 2 {
            ALPHABET[(n & 63) as usize] as char
        } else {
            '='
        });
    }
    Ok(value)
}
