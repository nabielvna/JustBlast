use super::*;
use crate::device::{correction, profile::Profile, runtime, touch, vision};

pub(super) struct CalibrationCapture {
    id: u32,
    serial: String,
    image: vision::Image,
}

fn view(profile: Profile, revision: u32) -> Result<CalibrationView, String> {
    Ok(CalibrationView {
        layout_detected: None,
        unclear_cells: vec![],
        anchors: vec![],
        movement: None,
        revision,
        toml: profile.export()?,
        profile,
        capture_id: None,
        image: None,
        board: None,
        board_unclear: false,
        pieces: vec![],
        unknown_slots: vec![],
        message: String::new(),
    })
}
fn preview(
    capture: &CalibrationCapture,
    profile: Profile,
    revision: u32,
) -> Result<CalibrationView, String> {
    profile.validate()?;
    if capture.image.width != profile.screen.width as usize
        || capture.image.height != profile.screen.height as usize
    {
        return Err("Calibration resolution must match the captured screenshot".to_owned());
    }
    let vision::FrameReading { board, cells, tray } =
        vision::read_frame(&capture.image, &profile.screen)?;
    let mut result = view(profile, revision)?;
    result.capture_id = Some(capture.id);
    result.board = Some(board.unwrap_or_else(|| vision::draft_board(&cells)).rows());
    result.board_unclear = board.is_none();
    result.unclear_cells = cells
        .iter()
        .enumerate()
        .filter_map(|(i, c)| (c.state == '?').then_some(i as u8))
        .collect();
    result.anchors = tray
        .pieces()
        .iter()
        .filter_map(|p| {
            Some(SlotAnchor {
                slot: p.slot,
                x: u32::try_from(p.bbox[0]).ok()?,
                y: u32::try_from(p.bbox[1]).ok()?,
            })
        })
        .collect();
    result.pieces = tray
        .editable_pieces()
        .into_iter()
        .map(|p| PieceView {
            slot: p.slot,
            catalog_id: (p.catalog_id > 0).then_some(p.catalog_id),
            name: format!("Slot {}", p.slot + 1),
            cells: p.shape.cells().to_vec(),
        })
        .collect();
    result.unknown_slots = tray
        .slots
        .iter()
        .enumerate()
        .filter_map(|(i, s)| matches!(s, vision::Slot::Uncertain { .. }).then_some(i as u8))
        .collect();
    Ok(result)
}
impl NativeState {
    pub fn calibration_request(
        &self,
        request: CalibrationRequest,
    ) -> Result<CalibrationView, CommandError> {
        let mut data = self.data()?;
        if request.expected_revision != data.device.revision {
            return Err(CommandError {
                code: ErrorCode::StaleRevision,
                message: "Phone settings changed. Reload calibration.".to_owned(),
            });
        }
        if data.device.busy || data.device.shutdown {
            return Err(CommandError {
                code: ErrorCode::Busy,
                message: "Wait for the phone operation to finish.".to_owned(),
            });
        }
        match request.action {
            CalibrationAction::Read {} => {
                return view(data.device.profile.clone(), data.device.revision)
                    .map_err(CommandError::invalid);
            }
            CalibrationAction::Export { profile } => {
                return view(profile, data.device.revision).map_err(CommandError::invalid);
            }
            CalibrationAction::AutoLayout {
                capture_id,
                mut profile,
            } => {
                let capture = data
                    .calibration
                    .as_ref()
                    .filter(|c| c.id == capture_id)
                    .ok_or_else(|| CommandError::invalid("Capture a fresh screenshot first."))?;
                profile.validate().map_err(CommandError::invalid)?;
                let (screen, found) = vision::auto_layout(&capture.image, &profile.screen)
                    .map_err(CommandError::invalid)?;
                profile.screen = screen;
                let mut result = preview(capture, profile, data.device.revision)
                    .map_err(CommandError::invalid)?;
                result.layout_detected = Some(found);
                result.message = if found { "Board grid located automatically. Check the grid and tray alignment, then confirm." }
                    else { "Could not confidently locate all board grid lines. The saved layout is an estimate; adjust it before confirming. Use the default game theme." }.to_owned();
                return Ok(result);
            }
            CalibrationAction::Preview {
                capture_id,
                profile,
            } => {
                let capture = data
                    .calibration
                    .as_ref()
                    .filter(|c| c.id == capture_id)
                    .ok_or_else(|| CommandError::invalid("Capture a fresh screenshot first."))?;
                return preview(capture, profile, data.device.revision)
                    .map_err(CommandError::invalid);
            }
            _ => {}
        }
        if request.command_id.is_empty()
            || request.command_id.len() > 100
            || data.calibration_commands.len() >= 4096
            || !data.calibration_commands.insert(request.command_id)
        {
            return Err(CommandError::invalid(
                "Calibration command already used or invalid. Capture again before retrying.",
            ));
        }
        data.device.reset_flags();
        data.device.busy = true;
        data.device.set("calibrating", "Calibrating phone…");
        let stop = data.device.stop.clone();
        let force = data.device.force.clone();
        drop(data);
        let result = (|| -> Result<CalibrationView, String> {
            let mut connection = self.1.lock().map_err(|_| "Phone connection unavailable")?;
            connection.transport.close();
            connection.lease = None;
            {
                let mut data = self.data().map_err(|e| e.message)?;
                data.device.serial = None;
                data.device.observation = None;
                data.device.capture = None;
                data.device.corrections = None;
                data.device.provenance = "none";
                data.device.needs_scan = true;
            }
            match request.action {
                CalibrationAction::Capture { serial } => {
                    let lease = {
                        let data = self.data().map_err(|e| e.message)?;
                        runtime::lease(&data.store, &serial)?
                    };
                    let image = connection.transport.calibration_capture(&serial, &stop)?;
                    drop(lease);
                    if image.width > image.height {
                        return Err("Use portrait orientation while calibrating.".to_owned());
                    }
                    let mut data = self.data().map_err(|e| e.message)?;
                    let mut profile = data.device.profile.clone();
                    let sx = image.width as f64 / f64::from(profile.screen.width);
                    let sy = image.height as f64 / f64::from(profile.screen.height);
                    fn scale(r: [u32; 4], sx: f64, sy: f64) -> [u32; 4] {
                        [
                            (f64::from(r[0]) * sx).floor() as u32,
                            (f64::from(r[1]) * sy).floor() as u32,
                            (f64::from(r[2]) * sx).floor().max(1.0) as u32,
                            (f64::from(r[3]) * sy).floor().max(1.0) as u32,
                        ]
                    }
                    profile.screen.board = scale(profile.screen.board, sx, sy);
                    profile.screen.slots = profile.screen.slots.map(|rect| scale(rect, sx, sy));
                    profile.screen.preview_pitch *= sx;
                    profile.touch.lift = (profile.touch.lift * sy).min(600.0);
                    profile.screen.width = image.width as u32;
                    profile.screen.height = image.height as u32;
                    profile.validate()?;
                    data.device.next_capture = data
                        .device
                        .next_capture
                        .checked_add(1)
                        .ok_or("Capture limit reached")?;
                    let capture = CalibrationCapture {
                        id: data.device.next_capture,
                        serial,
                        image,
                    };
                    let mut result = view(profile, data.device.revision)?;
                    result.image = Some(correction::png_data(&capture.image)?);
                    result.capture_id = Some(capture.id);
                    result.message =
                        "Screenshot captured. Adjust the overlays to match the game.".to_owned();
                    data.calibration = Some(capture);
                    Ok(result)
                }
                CalibrationAction::TestDrag {
                    capture_id,
                    profile,
                    slot,
                    row,
                    column,
                    install_helper,
                } => {
                    profile.validate()?;
                    if slot > 2 || row > 7 || column > 7 {
                        return Err("Choose a slot and board cell.".to_owned());
                    }
                    let (capture, helper, lease, journal, revision) = {
                        let mut data = self.data().map_err(|e| e.message)?;
                        let capture = data
                            .calibration
                            .take()
                            .filter(|c| c.id == capture_id)
                            .ok_or("Capture a fresh screenshot before each test drag.")?;
                        if data.device.uncertain(&capture.serial) {
                            return Err("Resolve the previous phone operation with a stable Scan before testing.".to_owned());
                        }
                        let lease = runtime::lease(&data.store, &capture.serial)?;
                        let helper = if install_helper {
                            let root = data
                                .store
                                .root()
                                .ok_or("Helper installation requires application storage")?;
                            let directory = root.join("native");
                            std::fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
                            let path = directory.join("bb-crop-android");
                            crate::preferences::atomic_write(
                                &path,
                                include_bytes!("../../../resources/native/bb-crop-android"),
                            )?;
                            Some(path)
                        } else {
                            None
                        };
                        (
                            capture,
                            helper,
                            lease,
                            data.device.journal.clone(),
                            data.device.revision,
                        )
                    };
                    connection.lease = lease;
                    let outcome = (|| -> Result<CalibrationView, String> {
                        if capture.image.width != profile.screen.width as usize
                            || capture.image.height != profile.screen.height as usize
                        {
                            return Err("Screenshot resolution changed. Capture again.".to_owned());
                        }
                        connection.transport.connect(
                            &capture.serial,
                            &profile.screen,
                            helper.as_deref(),
                            &stop,
                        )?;
                        let fresh = connection
                            .transport
                            .calibration_capture(&capture.serial, &stop)?;
                        if !correction::same_region(&capture.image, &fresh, profile.screen.board)
                            || !profile
                                .screen
                                .slots
                                .iter()
                                .all(|&rect| correction::same_region(&capture.image, &fresh, rect))
                        {
                            return Err(
                                "Game changed since capture. Capture again before a test drag."
                                    .to_owned(),
                            );
                        }
                        let observation = vision::read_snapshot(&fresh, &profile.screen)
                            .map_err(|e| e.message)?;
                        let vision::Slot::Recognized(piece) =
                            &observation.tray.slots[usize::from(slot)]
                        else {
                            return Err(
                                "Choose a clearly recognized shape before testing drag.".to_owned()
                            );
                        };
                        let after = observation
                            .board
                            .place(&piece.shape, row, column)
                            .ok_or("The chosen placement does not fit the board.")?;
                        let actions = touch::drag_piece(piece, row, column, &profile)?;
                        let delays = [profile.touch.hold, profile.touch.drop, profile.touch.gap];
                        journal.lock().map_err(|_| "Journal unavailable")?.intent(
                            &capture.serial,
                            revision,
                            0,
                            0,
                            0,
                            after.board,
                        )?;
                        if stop.load(Ordering::Acquire) || force.load(Ordering::Acquire) {
                            return Err("Test drag stopped. Capture again.".to_owned());
                        }
                        let mut acknowledged = 0;
                        connection
                            .transport
                            .execute(&actions, &delays, &force, &mut |count| {
                                acknowledged = count
                            })?;
                        if acknowledged != 1 {
                            return Err(
                                "Drag acknowledgement missing. Scan before any more movements."
                                    .to_owned(),
                            );
                        }
                        let mut previous: Option<vision::Observation> = None;
                        let mut stable = None;
                        let mut final_image = None;
                        for _ in 0..profile.runtime.tray_attempts.max(2) {
                            runtime::wait(
                                std::time::Duration::from_secs_f64(profile.runtime.tray_wait),
                                &stop,
                            )?;
                            let image = connection
                                .transport
                                .calibration_capture(&capture.serial, &stop)?;
                            if let Ok(observed) = vision::read_snapshot(&image, &profile.screen) {
                                if previous.as_ref().is_some_and(|p| {
                                    p.board == observed.board
                                        && p.tray.signature() == observed.tray.signature()
                                }) {
                                    stable = Some(observed);
                                    final_image = Some(image);
                                    break;
                                }
                                previous = Some(observed);
                            } else {
                                previous = None;
                            }
                        }
                        let observed = stable.ok_or("Landing could not be read reliably. No further test was sent. Use a stable Scan before trying again.")?;
                        let mut calibrated = profile.clone();
                        let verified = observed.board == after.board;
                        if !verified {
                            let mut locations = Vec::new();
                            for (r, c, _) in piece.shape.placements() {
                                if observation
                                    .board
                                    .place(&piece.shape, r, c)
                                    .is_some_and(|p| p.board == observed.board)
                                {
                                    locations.push((r, c));
                                }
                            }
                            let tray_matches =
                                observation.tray.slots.iter().enumerate().all(|(i, s)| {
                                    if i == usize::from(slot) {
                                        matches!(observed.tray.slots[i], vision::Slot::Empty)
                                    } else {
                                        s.candidate().map(|p| &p.shape)
                                            == observed.tray.slots[i].candidate().map(|p| &p.shape)
                                    }
                                });
                            if locations.len() != 1 || !tray_matches {
                                return Err("The landing is ambiguous. Automatic tests stopped; check the phone and use a stable Scan before trying again.".to_owned());
                            }
                            let (r, c) = locations[0];
                            let [bx, by, bw, bh] = profile.screen.board;
                            let actual = [
                                f64::from(bx)
                                    + (f64::from(c) + f64::from(piece.shape.width()) / 2.0)
                                        * f64::from(bw)
                                        / 8.0,
                                f64::from(by)
                                    + (f64::from(r) + f64::from(piece.shape.height()) / 2.0)
                                        * f64::from(bh)
                                        / 8.0,
                            ];
                            let centre = [
                                f64::from(piece.bbox[0]) + f64::from(piece.bbox[2]) / 2.0,
                                f64::from(piece.bbox[1]) + f64::from(piece.bbox[3]) / 2.0,
                            ];
                            let dx = f64::from(actions[1].x - actions[0].x);
                            let dy = f64::from(actions[1].y - actions[0].y);
                            let gain = if dx.abs() > f64::from(bw) / 8.0 {
                                (actual[0] - centre[0]) / dx
                            } else {
                                profile.touch.gain
                            };
                            let lift = centre[1] + dy * gain - actual[1];
                            if !(0.5..=3.0).contains(&gain) || !(0.0..=600.0).contains(&lift) {
                                return Err("The measured drag correction is outside the supported range. Check grid alignment and adjust movement manually.".to_owned());
                            }
                            calibrated.touch.gain = gain;
                            calibrated.touch.lift = lift;
                            calibrated.validate()?;
                        }
                        journal
                            .lock()
                            .map_err(|_| "Journal unavailable")?
                            .acknowledge(&capture.serial)?;
                        let mut data = self.data().map_err(|e| e.message)?;
                        data.device.next_capture = data
                            .device
                            .next_capture
                            .checked_add(1)
                            .ok_or("Capture limit reached")?;
                        let next_capture = CalibrationCapture {
                            id: data.device.next_capture,
                            serial: capture.serial.clone(),
                            image: final_image.ok_or("Missing verified image")?,
                        };
                        let mut result = preview(&next_capture, calibrated, revision)?;
                        result.image = Some(correction::png_data(&next_capture.image)?);
                        result.movement =
                            Some(if verified { "verified" } else { "adjusted" }.to_owned());
                        result.message = if verified { "Movement verified on the phone. Ready to save." }
                            else { "Landing measured. Gain and lift adjusted automatically; checking with another shape." }.to_owned();
                        data.calibration = Some(next_capture);
                        Ok(result)
                    })();
                    connection.transport.close();
                    connection.lease = None;
                    outcome
                }
                _ => unreachable!(),
            }
        })();
        let mut data = self.data()?;
        data.device.busy = false;
        data.device.needs_scan = true;
        data.device.set(
            "disconnected",
            "Calibration mode. Review the result and save when ready.",
        );
        result
            .map(|mut result| {
                result.revision = data.device.revision;
                result
            })
            .map_err(CommandError::invalid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn calibration_and_scan_share_the_same_board_tray_and_geometry() {
        for bytes in [
            include_bytes!("../../../tests/fixtures/before.png").as_slice(),
            include_bytes!("../../../tests/fixtures/board-textured-stars.png").as_slice(),
        ] {
            let image = crate::device::transport::decode_screenshot(bytes).unwrap();
            let profile = Profile::builtin();
            let scan = vision::read_snapshot(&image, &profile.screen).unwrap();

            let capture = CalibrationCapture {
                id: 1,
                serial: "offline".to_owned(),
                image,
            };
            let calibration = preview(&capture, profile, 1).unwrap();
            assert!(!calibration.board_unclear);
            assert_eq!(calibration.board, Some(scan.board.rows()));
            assert!(calibration.unknown_slots.is_empty());
            for piece in scan.tray.pieces() {
                assert_eq!(
                    calibration
                        .pieces
                        .iter()
                        .find(|p| p.slot == piece.slot)
                        .unwrap()
                        .cells,
                    piece.shape.cells()
                );
                let anchor = calibration
                    .anchors
                    .iter()
                    .find(|a| a.slot == piece.slot)
                    .unwrap();
                assert_eq!(
                    (anchor.x as i32, anchor.y as i32),
                    (piece.bbox[0], piece.bbox[1])
                );
            }
        }
    }
}
