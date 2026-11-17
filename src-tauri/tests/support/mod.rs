use justblast_core::domain::{Board, Shape, catalog};
use justblast_desktop::device::{
    profile::{Profile, Screen},
    touch::TouchAction,
    transport::{DeviceInfo, Transport},
    vision::Image,
};
use std::{
    path::Path,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

pub struct Phone {
    pub board: Board,
    pub slots: [Option<Shape>; 3],
    pub calls: usize,
    pub captures: usize,
    pub batches: Vec<usize>,
    pub fail_after_send: bool,
    pub slow: bool,
    pub next_tray: bool,
    pub closed: usize,
    pub force_seen: bool,
    pub mismatch_after_send: bool,
}
impl Default for Phone {
    fn default() -> Self {
        Self {
            board: Board::from_rows(
                &[
                    "#######.", "........", "........", "........", "........", "........",
                    "........", "........",
                ]
                .map(str::to_owned),
            )
            .unwrap(),
            slots: [Some(shape(1)), Some(shape(3)), None],
            calls: 0,
            captures: 0,
            batches: Vec::new(),
            fail_after_send: false,
            slow: false,
            next_tray: true,
            closed: 0,
            force_seen: false,
            mismatch_after_send: false,
        }
    }
}
pub fn shape(id: u32) -> Shape {
    catalog::catalog()
        .into_iter()
        .find(|x| x.0 == id)
        .unwrap()
        .1
}
fn rect(image: &mut Image, x: usize, y: usize, w: usize, h: usize, colour: [u8; 3]) {
    for r in y..(y + h).min(image.height) {
        for c in x..(x + w).min(image.width) {
            image.rgb[r * image.width + c] = colour;
        }
    }
}
pub fn render(phone: &Phone, screen: &Screen) -> Image {
    let mut image = Image::new(
        screen.width as usize,
        screen.height as usize,
        vec![[40, 70, 100]; screen.width as usize * screen.height as usize],
    )
    .unwrap();
    let [x, y, w, h] = screen.board;
    let rows = phone.board.rows();
    for (r, line) in rows.iter().enumerate() {
        for (c, cell) in line.bytes().enumerate() {
            if cell == b'#' {
                let left = x as usize + c * w as usize / 8;
                let right = x as usize + (c + 1) * w as usize / 8;
                let top = y as usize + r * h as usize / 8;
                let bottom = y as usize + (r + 1) * h as usize / 8;
                rect(
                    &mut image,
                    left + 3,
                    top + 3,
                    right - left - 6,
                    bottom - top - 6,
                    [220, 100, 60],
                );
            }
        }
    }
    for (slot, shape) in phone.slots.iter().enumerate() {
        if let Some(shape) = shape {
            let [x, y, w, h] = screen.slots[slot];
            let pitch = screen.preview_pitch as usize;
            let left = x as usize + (w as usize - usize::from(shape.width()) * pitch) / 2;
            let top = y as usize + (h as usize - usize::from(shape.height()) * pitch) / 2;
            for &(r, c) in shape.cells() {
                rect(
                    &mut image,
                    left + usize::from(c) * pitch + 1,
                    top + usize::from(r) * pitch + 1,
                    pitch - 2,
                    pitch - 2,
                    [180, 70, 40],
                );
                rect(
                    &mut image,
                    left + usize::from(c) * pitch + 4,
                    top + usize::from(r) * pitch + 4,
                    pitch - 8,
                    pitch - 8,
                    [220, 100, 60],
                );
            }
        }
    }
    image
}
pub struct Fake(pub Arc<Mutex<Phone>>);
impl Transport for Fake {
    fn discover(&mut self, _: &AtomicBool) -> Result<Vec<DeviceInfo>, String> {
        Ok(vec![DeviceInfo {
            serial: "offline-phone".to_owned(),
            state: "device".to_owned(),
            model: "Offline phone".to_owned(),
        }])
    }
    fn wireless(&mut self, _: &str, _: Option<&str>, _: &AtomicBool) -> Result<(), String> {
        Ok(())
    }
    fn connect(
        &mut self,
        _: &str,
        _: &Screen,
        _: Option<&Path>,
        _: &AtomicBool,
    ) -> Result<(), String> {
        Ok(())
    }
    fn calibration_capture(&mut self, _: &str, cancel: &AtomicBool) -> Result<Image, String> {
        self.capture(
            &justblast_desktop::device::profile::Profile::builtin().screen,
            false,
            cancel,
        )
    }
    fn capture(
        &mut self,
        screen: &Screen,
        tray_only: bool,
        cancel: &AtomicBool,
    ) -> Result<Image, String> {
        if cancel.load(Ordering::Acquire) {
            return Err("Cancelled capture".to_owned());
        }
        let mut phone = self.0.lock().unwrap();
        phone.captures += 1;
        let image = render(&phone, screen);
        if tray_only {
            image.crop(screen.tray())
        } else {
            Ok(image)
        }
    }
    fn execute(
        &mut self,
        actions: &[TouchAction],
        _: &[f64],
        force: &AtomicBool,
        on_move: &mut dyn FnMut(usize),
    ) -> Result<(), String> {
        let profile = Profile::builtin();
        {
            let mut phone = self.0.lock().unwrap();
            phone.calls += 1;
            phone.batches.push(actions.len() / 3);
        }
        let slow = self.0.lock().unwrap().slow;
        if slow {
            for _ in 0..100 {
                if force.load(Ordering::Acquire) {
                    self.0.lock().unwrap().force_seen = true;
                    return Err("Uncertain forced result".to_owned());
                }
                std::thread::sleep(Duration::from_millis(10));
            }
        }
        for (index, actions) in actions.chunks(3).enumerate() {
            let mut phone = self.0.lock().unwrap();
            let slot = profile
                .screen
                .slots
                .iter()
                .position(|r| actions[0].x >= r[0] as i32 && actions[0].x < (r[0] + r[2]) as i32)
                .ok_or("Touch did not start in a slot")?;
            let shape = phone.slots[slot].take().ok_or("Executed empty slot")?;
            let [x, y, w, h] = profile.screen.slots[slot];
            let pitch = profile.screen.preview_pitch;
            let width = f64::from(shape.width()) * pitch;
            let height = f64::from(shape.height()) * pitch;
            let left = f64::from(x) + (f64::from(w) - width) / 2.0;
            let top = f64::from(y) + (f64::from(h) - height) / 2.0;
            let centre = [left + width / 2.0, top + height / 2.0];
            let target = [
                f64::from(actions[1].x - actions[0].x) * profile.touch.gain + centre[0],
                f64::from(actions[1].y - actions[0].y) * profile.touch.gain + centre[1]
                    - profile.touch.lift,
            ];
            let [bx, by, bw, bh] = profile.screen.board;
            let col = ((target[0] - f64::from(bx)) * 8.0 / f64::from(bw)
                - f64::from(shape.width()) / 2.0)
                .round() as u8;
            let row = ((target[1] - f64::from(by)) * 8.0 / f64::from(bh)
                - f64::from(shape.height()) / 2.0)
                .round() as u8;
            phone.board = phone
                .board
                .place(&shape, row, col)
                .ok_or("Illegal physical placement")?
                .board;
            drop(phone);
            on_move(index + 1);
        }
        let mut phone = self.0.lock().unwrap();
        if phone.slots.iter().all(Option::is_none) && phone.next_tray {
            phone.slots = [Some(shape(9)), Some(shape(5)), Some(shape(6))];
        }
        if phone.mismatch_after_send {
            let mut rows = phone.board.rows();
            let value = if rows[7].as_bytes()[7] == b'#' {
                "."
            } else {
                "#"
            };
            rows[7].replace_range(7..8, value);
            phone.board = Board::from_rows(&rows).unwrap();
        }
        if phone.fail_after_send {
            return Err("Input result uncertain: acknowledgement lost".to_owned());
        }
        Ok(())
    }
    fn close(&mut self) {
        self.0.lock().unwrap().closed += 1;
    }
}

pub fn custom_ring() -> Shape {
    Shape::new(
        (0..4)
            .flat_map(|r| {
                (0..4).filter_map(move |c| (r == 0 || r == 3 || c == 0 || c == 3).then_some((r, c)))
            })
            .collect(),
    )
    .unwrap()
}
