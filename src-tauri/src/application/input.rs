use super::*;
use crate::device::{
    correction::{self, Capture, Corrections},
    vision::{Observation, Slot},
};
use justblast_core::solver::Piece;

impl Data {
    pub(super) fn capture_view(&self, capture: &Capture) -> CaptureView {
        CaptureView {
            id: capture.id,
            board: if self.device.board_accepted {
                Some(self.workspace.current_board().rows())
            } else {
                capture.board.map(|b| b.rows())
            },
            pieces: capture
                .tray
                .editable_pieces()
                .into_iter()
                .map(|p| PieceView {
                    slot: p.slot,
                    catalog_id: (p.catalog_id > 0).then_some(p.catalog_id),
                    name: format!("Piece {}", p.slot + 1),
                    cells: p.shape.cells().to_vec(),
                })
                .collect(),
            unknown_slots: capture
                .tray
                .slots
                .iter()
                .enumerate()
                .filter_map(|(i, s)| {
                    (matches!(s, Slot::Uncertain { .. }) && !self.device.accepted_slots[i])
                        .then_some(i as u8)
                })
                .collect(),
            slot_issues: std::array::from_fn(|i| match &capture.tray.slots[i] {
                Slot::Uncertain { reason, .. } if !self.device.accepted_slots[i] => {
                    Some(reason.clone())
                }
                _ => None,
            }),
            pitch: self.device.profile.screen.preview_pitch,
            slot_crops: capture.crops.clone(),
            slot_sizes: self
                .device
                .profile
                .screen
                .slots
                .iter()
                .map(|r| (r[2], r[3]))
                .collect(),
            anchors: capture
                .tray
                .editable_pieces()
                .iter()
                .map(|p| {
                    let r = self.device.profile.screen.slots[usize::from(p.slot)];
                    SlotAnchor {
                        slot: p.slot,
                        x: (p.bbox[0] - r[0] as i32).max(0) as u32,
                        y: (p.bbox[1] - r[1] as i32).max(0) as u32,
                    }
                })
                .collect(),
        }
    }
    pub(super) fn readiness(&self) -> String {
        if self.device.serial.is_none() {
            return "Connect a phone, then Scan to link this input.".to_owned();
        }
        if self
            .device
            .serial
            .as_deref()
            .is_some_and(|s| self.device.uncertain(s))
            || self.device.provenance == "uncertain"
        {
            return "Recovery required. Run a stable Scan before moving.".to_owned();
        }
        if self.device.observation.is_none() || self.device.needs_scan {
            return self.device.link_error.clone();
        }
        let snapshot = self.workspace.snapshot();
        if !snapshot.input_known {
            return "Accept or edit the highlighted board or slots.".to_owned();
        }
        if snapshot.pieces.is_empty() {
            return "Tray is empty. Wait for new pieces, then Scan.".to_owned();
        }
        if snapshot
            .plan
            .as_ref()
            .is_none_or(|p| !p.complete || p.steps.len() != snapshot.pieces.len())
        {
            return "Calculate a complete plan for all remaining pieces.".to_owned();
        }
        if self.device.busy {
            return "Wait for the active phone operation.".to_owned();
        }
        String::new()
    }
    pub(super) fn refresh_device_readiness(&mut self) {
        let snapshot = self.workspace.snapshot();
        self.device.cycle_done &= snapshot.completed > 0
            && snapshot
                .plan
                .as_ref()
                .is_some_and(|p| p.complete && usize::from(snapshot.completed) == p.steps.len());
        if snapshot
            .plan
            .as_ref()
            .is_some_and(|p| p.complete && p.steps.len() == snapshot.pieces.len())
            && self.device.observation.is_some()
        {
            self.device.needs_scan = false;
            self.device.link_error.clear();
            if self.device.cycle_done {
                self.device.set(
                    "cycle_completed",
                    "Cycle completed. Next cycle reads the next tray without moving.",
                );
            } else {
                self.device
                    .set("ready", "Plan ready. Preview locally or Play on phone.");
            }
        }
    }
}
impl NativeState {
    pub(super) fn apply_input_locked(
        &self,
        mut data: MutexGuard<'_, Data>,
        expected: u32,
        rows: [String; 8],
        pieces: Vec<PieceView>,
        anchors: Vec<SlotAnchor>,
        acceptance: Option<([bool; 3], bool)>,
    ) -> Result<WorkspaceView, CommandError> {
        data.workspace.check_revision(expected)?;
        if data.device.provenance == "uncertain"
            || data
                .device
                .serial
                .as_deref()
                .is_some_and(|s| data.device.uncertain(s))
        {
            return Err(CommandError::invalid(
                "Run a stable recovery Scan before applying corrections.",
            ));
        }
        if anchors.len() > 3
            || anchors
                .iter()
                .enumerate()
                .any(|(i, a)| a.slot > 2 || anchors[..i].iter().any(|b| b.slot == a.slot))
        {
            return Err(CommandError::invalid("Invalid slot origins"));
        }
        let board = parse_board(&rows)?;
        let pieces = pieces
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<Piece>, _>>()?;
        let accepted = acceptance.map_or([true; 3], |a| a.0);
        let board_accepted = acceptance.is_none_or(|a| a.1);
        let previous = data.workspace.snapshot();
        let capture = data.device.capture.clone();
        let mut observation = None;
        let mut corrections = None;
        let mut link_error = "Scan to link this input to your phone.".to_owned();
        if let Some(capture) = capture {
            let screen = &data.device.profile.screen;
            let mut tray = capture.tray.clone();
            let consumed: Vec<_> = data
                .workspace
                .game()
                .checkpoint
                .as_ref()
                .map_or_else(Vec::new, |c| c.steps.iter().map(|s| s.slot).collect());
            let mut slots: [Option<Slot>; 3] = std::array::from_fn(|_| None);
            let mut valid = capture.board.is_some() || board_accepted;
            for slot in 0..3 {
                if consumed.contains(&(slot as u8)) {
                    tray.slots[slot] = Slot::Empty;
                    continue;
                }
                if let Some(piece) = pieces.iter().find(|p| usize::from(p.slot) == slot) {
                    let raw = &capture.tray.slots[slot];
                    if matches!(raw, Slot::Uncertain { .. }) && !accepted[slot] {
                        valid = false;
                        link_error = "Accept or edit the highlighted slots.".to_owned();
                        continue;
                    }
                    let anchor = anchors
                        .iter()
                        .find(|a| usize::from(a.slot) == slot)
                        .map(|a| (a.x, a.y))
                        .or_else(|| {
                            let previous = data.device.corrections.as_ref()?;
                            let Some(Slot::Recognized(mapped)) = &previous.slots[slot] else {
                                return None;
                            };
                            if mapped.shape != piece.shape
                                || !correction::same_region(
                                    &previous.capture.image,
                                    &capture.image,
                                    screen.slots[slot],
                                )
                            {
                                return None;
                            }
                            let rect = screen.slots[slot];
                            Some((
                                (mapped.bbox[0] as u32).saturating_sub(rect[0]),
                                (mapped.bbox[1] as u32).saturating_sub(rect[1]),
                            ))
                        });
                    let unchanged = matches!(raw,Slot::Recognized(p) if p.shape==piece.shape)
                        && anchor.is_none();
                    if !unchanged {
                        match correction::accepted_piece(
                            &capture,
                            screen,
                            slot as u8,
                            piece.shape.clone(),
                            anchor,
                        ) {
                            Ok(mut mapped) => {
                                mapped.catalog_id = piece.catalog_id.unwrap_or(0);
                                let mapped = Slot::Recognized(mapped);
                                tray.slots[slot] = mapped.clone();
                                slots[slot] = Some(mapped);
                            }
                            Err(error) => {
                                valid = false;
                                link_error = error;
                            }
                        }
                    }
                } else {
                    tray.slots[slot] = Slot::Empty;
                    if !matches!(capture.tray.slots[slot], Slot::Empty) {
                        slots[slot] = Some(Slot::Empty);
                    }
                }
            }
            let raw = capture.board.map(|b| b.rows());
            let cells = (0..8)
                .flat_map(|r| (0..8).map(move |c| (r, c)))
                .filter_map(|(r, c)| {
                    let value = rows[r].as_bytes()[c];
                    (raw.as_ref().is_none_or(|b| b[r].as_bytes()[c] != value)).then_some((
                        r as u8,
                        c as u8,
                        value == b'#',
                    ))
                })
                .collect();
            if valid && data.device.serial.is_some() {
                observation = Some(Observation { board, tray });
                corrections = Some(Corrections {
                    capture,
                    cells,
                    slots,
                });
                link_error.clear();
            }
        }
        let source = if data.device.capture.is_some() {
            "corrected"
        } else {
            "entered"
        };
        data.workspace
            .apply_input(expected, board, pieces, source)?;
        let input_known = capture_is_accepted(&data, &accepted, board_accepted);
        if !input_known {
            data.workspace.require_input_review();
        }
        data.device.accepted_slots = accepted;
        data.device.board_accepted = board_accepted;
        data.invalidate();
        data.device.visuals.clear();
        data.device.observation = observation;
        data.device.corrections = corrections;
        data.device.needs_scan = data.device.observation.is_none();
        data.device.link_error = link_error;
        data.device.error = None;
        data.device.completed = previous.completed;
        if previous.completed == 0 {
            data.device.cycle_done = false;
        }
        if previous.completed > 0
            && data.workspace.snapshot().pieces.len() == usize::from(previous.completed)
            && !data.device.cycle_done
            && data.device.observation.is_some()
        {
            let mut history = data.history.clone();
            history.push(HistoryEntry {
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
                    .to_string(),
                source: "phone".to_owned(),
                cycle: previous.cycle,
                status: "reconciled".to_owned(),
                message: format!(
                    "{} acknowledged placements; remaining slots corrected to empty",
                    previous.completed
                ),
                board: board.rows(),
            });
            if history.len() > 200 {
                history.drain(..history.len() - 200);
            }
            data.store
                .save_history(&history)
                .map_err(|message| CommandError {
                    code: ErrorCode::Storage,
                    message,
                })?;
            data.history = history;
            data.device.cycle_done = true;
        }
        let revision = data.workspace.snapshot().revision;
        let has_remaining = !data.workspace.snapshot().pieces.is_empty();
        if !input_known {
            data.device.set(
                "needs_correction",
                "Accept or edit the highlighted input to continue.",
            );
            return Ok(data.view());
        }
        data.device.set(
            "connected",
            "Input applied. Calculating the remaining plan…",
        );
        if !has_remaining {
            return Ok(data.view());
        }
        drop(data);
        self.start_job(revision, 0, None, false)
    }
    pub(super) fn accept_capture(&self, capture: Capture, stop: &AtomicBool) -> Result<(), String> {
        let known = capture.board.is_some() && capture.tray.readable();
        let board = capture.board.unwrap_or(capture.board_draft);
        let pieces: Vec<Piece> = capture
            .tray
            .editable_pieces()
            .into_iter()
            .map(|p| Piece {
                slot: p.slot,
                catalog_id: (p.catalog_id > 0).then_some(p.catalog_id),
                name: format!("Piece {}", p.slot + 1),
                shape: p.shape,
            })
            .collect();
        let mut data = self.data().map_err(|e| e.message)?;
        let previous = data.workspace.snapshot();
        let changed_executed_slot =
            data.workspace
                .game()
                .checkpoint
                .as_ref()
                .is_some_and(|checkpoint| {
                    checkpoint.steps.iter().any(|step| {
                        !matches!(capture.tray.slots[usize::from(step.slot)], Slot::Empty)
                    })
                });
        let new_checkpoint = previous.completed > 0
            && (data.device.provenance == "uncertain" || changed_executed_slot);
        if new_checkpoint {
            let mut history = data.history.clone();
            history.push(HistoryEntry {timestamp:std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs().to_string(),source:"phone".to_owned(),cycle:previous.cycle,status:"reconciled".to_owned(),message:format!("{} acknowledged placements archived before a new scanned checkpoint; no movements replayed",previous.completed),board:data.workspace.current_board().rows()});
            if history.len() > 200 {
                history.drain(..history.len() - 200);
            }
            data.store.save_history(&history)?;
            data.history = history;
            data.device.cycle_done = false;
        }
        if previous.completed > 0 && !new_checkpoint {
            let mut merged = pieces;
            for step in data
                .workspace
                .game()
                .checkpoint
                .as_ref()
                .map_or(&[][..], |c| c.steps.as_slice())
            {
                if merged.iter().any(|p| p.slot == step.slot) {
                    return Err("An executed slot is present again. Recover the changed game before continuing.".to_owned());
                }
                if let Some(piece) = previous.pieces.iter().find(|p| p.slot == step.slot) {
                    merged.push(piece.clone());
                }
            }
            data.workspace
                .apply_input(previous.revision, board, merged, "observed")
                .map_err(|e| format!("Scan reconciliation rejected: {e:?}"))?;
        } else {
            data.workspace
                .accept_observation(
                    previous.revision,
                    board,
                    pieces,
                    [justblast_core::domain::combo::ComboState::default(); 3],
                    justblast_core::domain::combo::RuleContext::default(),
                    known,
                )
                .map_err(|e| format!("Scan input rejected: {e:?}"))?;
            data.device.completed = 0;
            data.device.cycle_done = false;
        }
        if !known {
            data.workspace.require_input_review();
        }
        data.device.observation = if known { capture.observation() } else { None };
        data.device.position_overridden = false;
        data.device.accepted_slots = [false; 3];
        data.device.board_accepted = false;
        data.device.capture = Some(capture);
        data.device.corrections = None;
        data.device.provenance = "observed";
        data.device.visuals.clear();
        data.device.needs_scan = !known;
        data.device.link_error = if known {
            String::new()
        } else {
            "Accept or edit the highlighted board or slots.".to_owned()
        };
        data.invalidate();
        if !known {
            data.device.set(
                "needs_correction",
                "Scan loaded. Edit any unreadable board cells or piece slots to correct them.",
            );
            return Ok(());
        }
        if data.workspace.snapshot().pieces.is_empty() {
            data.device.set(
                "empty_tray",
                "Tray is empty. Wait for new pieces, then Scan.",
            );
            return Ok(());
        }
        let revision = data.workspace.snapshot().revision;
        drop(data);
        if stop.load(Ordering::Acquire) {
            return Err("Stopped before calculation".to_owned());
        }
        self.start_job_during_scan(revision)
    }
    fn start_job_during_scan(&self, revision: u32) -> Result<(), String> {
        self.start_computation(revision, 0, None, false, true)
            .map(|_| ())
            .map_err(|e| e.message)
    }
}

fn capture_is_accepted(data: &Data, slots: &[bool; 3], board: bool) -> bool {
    data.device.capture.as_ref().is_none_or(|c| {
        (c.board.is_some() || board)
            && c.tray
                .slots
                .iter()
                .enumerate()
                .all(|(i, slot)| !matches!(slot, Slot::Uncertain { .. }) || slots[i])
    })
}
