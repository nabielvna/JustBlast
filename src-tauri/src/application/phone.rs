use super::*;
use crate::device::{
    profile::Profile,
    runtime::{self},
    touch,
    transport::Transport,
    vision::{self, Observation, SlotSignature, Tray},
};
use justblast_core::{
    domain::combo::ComboState,
    services::Phase,
    solver::{Piece, SearchResult},
};
use std::{
    hash::{Hash, Hasher},
    sync::atomic::AtomicBool,
    time::{SystemTime, UNIX_EPOCH},
};

impl Data {
    pub(super) fn device_view(&self) -> DeviceView {
        let d = &self.device;
        DeviceView {
            revision: d.revision,
            state: d.state.to_owned(),
            profile_id: d.profile_id.clone(),
            profile_ids: d.profile_ids.clone(),
            message: d.message.clone(),
            error: d.error.clone(),
            busy: d.busy,
            serial: d.serial.clone(),
            devices: d
                .devices
                .iter()
                .map(|d| DeviceInfoView {
                    serial: d.serial.clone(),
                    state: d.state.clone(),
                    model: d.model.clone(),
                })
                .collect(),
            completed: d.completed,
            needs_scan: d.needs_scan,
            cycle_done: d.cycle_done,
            provenance: d.provenance.to_owned(),
            visual: d.visuals.front().and_then(|v| {
                v.snapshot.plan.as_ref().map(|p| VisualCycleView {
                    id: v.id,
                    cycle: v.snapshot.cycle,
                    revision: v.snapshot.revision,
                    board: v.snapshot.board.rows(),
                    pieces: v.snapshot.pieces.iter().cloned().map(Into::into).collect(),
                    plan: PlanView::from_result(p, v.snapshot.pieces.len()),
                    completed: v.completed,
                    done: v.done,
                })
            }),
            queued_visuals: d.visuals.len() as u32,
            dropped_visuals: d.dropped_visuals,
            capture: d.capture.as_ref().map(|c| self.capture_view(c)),
            readiness: self.readiness(),
        }
    }
}
fn device_error(message: impl Into<String>) -> CommandError {
    CommandError {
        code: ErrorCode::Device,
        message: message.into(),
    }
}
impl NativeState {
    pub fn with_transport(
        store: crate::preferences::Store,
        transport: Box<dyn Transport>,
    ) -> Result<Self, String> {
        let state = Self::with_store(store)?;
        state.1.lock().map_err(|e| e.to_string())?.transport = transport;
        Ok(state)
    }
    pub fn device_request(&self, request: DeviceRequest) -> Result<WorkspaceView, CommandError> {
        if request.command_id.is_empty() || request.command_id.len() > 100 {
            return Err(CommandError::invalid("Invalid command identity"));
        }
        let mut hash = std::collections::hash_map::DefaultHasher::new();
        serde_json::to_string(&request)
            .map_err(|_| CommandError::invalid("Invalid device request"))?
            .hash(&mut hash);
        let payload = hash.finish();
        let mut data = self.data()?;
        if let Some(&previous) = data.device.commands.get(&request.command_id) {
            if previous != payload {
                return Err(CommandError::invalid(
                    "Command identity reused with different input",
                ));
            }
            return Ok(data.view());
        }
        if data.device.commands.len() >= 4096 {
            return Err(CommandError::invalid(
                "Restart JustBlast to clear command history",
            ));
        }
        if data.device.shutdown {
            return Err(device_error("JustBlast is shutting down"));
        }
        let control = matches!(
            request.action,
            DeviceAction::Stop {}
                | DeviceAction::ForceStop {}
                | DeviceAction::AcknowledgeVisual { .. }
        );
        if !control && request.expected_revision != data.device.revision {
            return Err(CommandError {
                code: ErrorCode::StaleRevision,
                message: "The device state changed. Refresh before trying again.".to_owned(),
            });
        }
        match &request.action {
            DeviceAction::AcceptWarning {} => {
                if !data.device.busy || data.device.state != "awaiting_confirmation" {
                    return Err(device_error("There is no position warning to accept"));
                }
                data.device.warning_accepted = true;
                data.device.set(
                    "verifying",
                    "Warning accepted. Continuing the remaining moves…",
                );
                data.device.commands.insert(request.command_id, payload);
                return Ok(data.view());
            }
            DeviceAction::Stop {} | DeviceAction::ForceStop {} => {
                data.device.stop.store(true, Ordering::Release);
                if matches!(request.action, DeviceAction::ForceStop {}) {
                    data.device.force.store(true, Ordering::Release);
                    data.device.needs_scan = true;
                }
                if data.device.busy {
                    data.device.set(
                        "stopping",
                        "Stopping; waiting for the active operation to finish.",
                    );
                }
                data.device.commands.insert(request.command_id, payload);
                return Ok(data.view());
            }
            DeviceAction::AcknowledgeVisual { cycle_id } => {
                if data
                    .device
                    .visuals
                    .front()
                    .is_some_and(|v| v.id == *cycle_id && v.done)
                {
                    data.device.visuals.pop_front();
                    data.device.revision += 1;
                }
                return Ok(data.view());
            }
            _ => {}
        }
        if data.device.busy {
            return Err(CommandError {
                code: ErrorCode::Busy,
                message: "Wait for the active device operation or press Stop".to_owned(),
            });
        }
        match &request.action {
            DeviceAction::NextCycle { workspace_revision } => {
                data.workspace.check_revision(*workspace_revision)?;
                if !data.device.cycle_done || data.device.observation.is_none() {
                    return Err(device_error("Complete the current phone cycle first"));
                }
            }
            DeviceAction::Profile { id, toml } => {
                if data.device.serial.is_some() {
                    return Err(device_error("Disconnect before changing calibration"));
                }
                if !crate::device::profile::valid_id(id) || id == "default" || toml.len() > 65536 {
                    return Err(CommandError::invalid(
                        "Invalid calibration identity or file size",
                    ));
                }
                let profile = Profile::parse(toml).map_err(CommandError::invalid)?;
                if data.device.profile_ids.len() >= 65 && !data.device.profile_ids.contains(id) {
                    return Err(CommandError::invalid(
                        "At most 64 custom profiles are supported",
                    ));
                }
                let root = data.store.root().ok_or_else(|| {
                    device_error("Calibration import requires application storage")
                })?;
                let directory = root.join("profiles");
                std::fs::create_dir_all(&directory).map_err(|e| device_error(e.to_string()))?;
                crate::preferences::atomic_write(
                    &directory.join(format!("{id}.toml")),
                    profile.export().map_err(CommandError::invalid)?.as_bytes(),
                )
                .map_err(device_error)?;
                let mut prefs = data.preferences.clone();
                prefs.connection.profile_id = id.clone();
                prefs.advance().map_err(CommandError::invalid)?;
                data.store.save_preferences(&prefs).map_err(device_error)?;
                data.preferences = prefs;
                data.invalidate();
                data.device.needs_scan = true;
                data.device.profile = profile;
                data.device.profile_id = id.clone();
                if !data.device.profile_ids.contains(id) {
                    data.device.profile_ids.push(id.clone());
                }
                data.device.commands.insert(request.command_id, payload);
                data.device.set(
                    "disconnected",
                    format!("Calibration {id} imported. Connect your phone."),
                );
                return Ok(data.view());
            }
            DeviceAction::SelectProfile { id } => {
                if data.device.serial.is_some() {
                    return Err(device_error("Disconnect before changing calibration"));
                }
                let profile =
                    crate::device::profile::load(&data.store, id).map_err(CommandError::invalid)?;
                let mut prefs = data.preferences.clone();
                prefs.connection.profile_id = id.clone();
                prefs.advance().map_err(CommandError::invalid)?;
                data.store.save_preferences(&prefs).map_err(device_error)?;
                data.preferences = prefs;
                data.invalidate();
                data.device.needs_scan = true;
                data.device.profile = profile;
                data.device.profile_id = id.clone();
                data.device.commands.insert(request.command_id, payload);
                data.device.set(
                    "disconnected",
                    format!("Calibration {id} selected. Connect your phone."),
                );
                return Ok(data.view());
            }
            DeviceAction::Scan {
                workspace_revision, ..
            }
            | DeviceAction::CaptureForEdit { workspace_revision } => {
                data.workspace.check_revision(*workspace_revision)?;
                if data.device.serial.is_none() {
                    return Err(device_error("Connect a phone first"));
                }
            }
            DeviceAction::Start {
                workspace_revision,
                through,
            } => {
                data.workspace.check_revision(*workspace_revision)?;
                if !data.preferences.automation.move_device {
                    return Err(device_error(
                        "Enable Move device before sending phone movements",
                    ));
                }
                let snapshot = data.workspace.snapshot();
                let plan = snapshot
                    .plan
                    .as_ref()
                    .ok_or_else(|| device_error("Scan and calculate a complete plan first"))?;
                if !data.readiness().is_empty()
                    || data.device.cycle_done
                    || data.device.state != "ready"
                    || data.device.needs_scan
                    || !plan.complete
                    || plan.steps.len() != snapshot.pieces.len()
                {
                    return Err(device_error(
                        "Scan and calculate a fresh complete plan first",
                    ));
                }
                if let Some(index) = through
                    && (usize::from(*index) >= plan.steps.len() || *index < data.device.completed)
                {
                    return Err(device_error("This move is unknown or has already executed"));
                }
                if usize::from(data.device.completed) >= plan.steps.len() {
                    return Err(device_error("These phone movements have already executed"));
                }
            }
            DeviceAction::Connect { serial, .. } => {
                if serial.is_empty() || serial.len() > 200 {
                    return Err(CommandError::invalid("Invalid device serial"));
                }
            }
            DeviceAction::Discover {}
            | DeviceAction::Wireless { .. }
            | DeviceAction::Disconnect {} => {}
            _ => return Err(CommandError::invalid("Invalid device action")),
        }
        if data.workspace.snapshot().phase == Phase::Searching {
            let revision = data.workspace.snapshot().revision;
            data.workspace.cancel_search(revision)?;
        }
        data.invalidate();
        data.device.reset_flags();
        data.device.busy = true;
        data.device.commands.insert(request.command_id, payload);
        data.device.set(
            match request.action {
                DeviceAction::Discover {} => "discovering",
                DeviceAction::Wireless { .. } => "connecting",
                DeviceAction::Connect { .. } => "connecting",
                DeviceAction::Disconnect {} => "disconnecting",
                DeviceAction::Scan { .. }
                | DeviceAction::NextCycle { .. }
                | DeviceAction::CaptureForEdit { .. } => "scanning",
                DeviceAction::Start { .. } => "checking",
                _ => "connected",
            },
            "Working…",
        );
        let state = self.clone();
        let action = request.action;
        if let Err(error) = std::thread::Builder::new()
            .name("justblast-phone".to_owned())
            .spawn(move || {
                let result = state.phone_operation(action);
                let Ok(mut data) = state.data() else {
                    return;
                };
                if let Err(message) = result {
                    let uncertain = data.device.force.load(Ordering::Acquire)
                        || data
                            .device
                            .serial
                            .as_deref()
                            .is_some_and(|serial| data.device.uncertain(serial));
                    let stopped = data.device.stop.load(Ordering::Acquire);
                    data.device.needs_scan = true;
                    data.device.error = if stopped && !uncertain {
                        None
                    } else {
                        Some(message.clone())
                    };
                    if let Some(visual) = data.device.visuals.back_mut() {
                        visual.done = true;
                    }
                    if data.workspace.snapshot().phase == Phase::Searching {
                        let revision = data.workspace.snapshot().revision;
                        let _ = data.workspace.cancel_search(revision);
                    }
                    data.device.provenance = if uncertain {
                        "uncertain"
                    } else {
                        data.device.provenance
                    };
                    data.device.set(
                        if uncertain {
                            "uncertain"
                        } else if stopped {
                            "stopped"
                        } else {
                            "error"
                        },
                        message,
                    );
                    if uncertain {
                        drop(data);
                        if let Ok(mut connection) = state.1.lock() {
                            connection.transport.close();
                        }
                        let Ok(next) = state.data() else {
                            return;
                        };
                        data = next;
                    }
                }
                data.device.busy = false;
                data.device.revision += 1;
            })
        {
            data.device.busy = false;
            return Err(device_error(error.to_string()));
        }
        Ok(data.view())
    }
    fn phone_operation(&self, action: DeviceAction) -> Result<(), String> {
        let mut connection = self
            .1
            .lock()
            .map_err(|_| "Device transport is unavailable")?;
        let (stop, profile) = {
            let data = self.data().map_err(|e| e.message)?;
            (data.device.stop.clone(), data.device.profile.clone())
        };
        connection
            .transport
            .configure_timeout(Duration::from_secs_f64(profile.runtime.command_timeout));
        match action {
            DeviceAction::Discover {} => {
                let devices = connection.transport.discover(&stop)?;
                let mut data = self.data().map_err(|e| e.message)?;
                data.device.devices = devices;
                let state = if data.device.serial.is_some() {
                    if !data.device.needs_scan && data.workspace.snapshot().plan.is_some() {
                        "ready"
                    } else {
                        "connected"
                    }
                } else {
                    "disconnected"
                };
                data.device
                    .set(state, "Discovery finished. Select an authorized phone.");
            }
            DeviceAction::Wireless { address, code } => {
                connection
                    .transport
                    .wireless(&address, code.as_deref(), &stop)?;
                let devices = connection.transport.discover(&stop)?;
                let mut data = self.data().map_err(|e| e.message)?;
                data.device.devices = devices;
                let status = if data.device.serial.is_some() {
                    if !data.device.needs_scan && data.workspace.snapshot().plan.is_some() {
                        "ready"
                    } else {
                        "connected"
                    }
                } else {
                    "disconnected"
                };
                data.device.set(
                    status,
                    if code.is_some() {
                        "Paired. Connect using the debugging port."
                    } else {
                        "Wireless connected. Select the phone, then Connect."
                    },
                );
            }
            DeviceAction::Connect {
                serial,
                install_helper,
            } => {
                connection.transport.close();
                connection.lease = None;
                {
                    let mut data = self.data().map_err(|e| e.message)?;
                    data.device.serial = None;
                    data.device.observation = None;
                    data.device.capture = None;
                    data.device.corrections = None;
                    data.device.needs_scan = true;
                }
                let (helper, lease) = {
                    let data = self.data().map_err(|e| e.message)?;
                    let lease = runtime::lease(&data.store, &serial)?;
                    let helper = if install_helper {
                        let root = data.store.root().ok_or(
                            "Capture helper installation requires the desktop data directory",
                        )?;
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
                    (helper, lease)
                };
                connection
                    .transport
                    .connect(&serial, &profile.screen, helper.as_deref(), &stop)?;
                connection.lease = lease;
                let mut data = self.data().map_err(|e| e.message)?;
                data.device.serial = Some(serial.clone());
                data.device.needs_scan = true;

                data.device.observation = None;
                data.device.provenance = if data.device.uncertain(&serial) {
                    "uncertain"
                } else {
                    "none"
                };
                data.device.set(
                    "connected",
                    "Connected. Open the game, then Scan. No touch commands sent.",
                );
            }
            DeviceAction::Disconnect {} => {
                connection.transport.close();
                connection.lease = None;
                let mut data = self.data().map_err(|e| e.message)?;
                data.device.serial = None;
                data.device.observation = None;
                data.device.capture = None;
                data.device.corrections = None;
                data.device.needs_scan = true;
                data.device.provenance = "none";
                data.device.set("disconnected", "Disconnected.");
            }
            DeviceAction::Scan { .. } => {
                let stable = {
                    let data = self.data().map_err(|e| e.message)?;
                    data.device
                        .serial
                        .as_deref()
                        .is_some_and(|s| data.device.uncertain(s))
                        || data.device.provenance == "uncertain"
                };
                let mut capture = self.raw_capture(&mut *connection.transport, &profile, &stop)?;
                if stable {
                    let mut matched = false;
                    for _ in 0..profile.runtime.tray_attempts {
                        runtime::wait(Duration::from_secs_f64(profile.runtime.retry_wait), &stop)?;
                        let next = self.raw_capture(&mut *connection.transport, &profile, &stop)?;
                        let semantic_match = capture.board.is_some()
                            && capture.board == next.board
                            && capture.tray.readable()
                            && next.tray.readable()
                            && capture.tray.signature() == next.tray.signature();
                        let image_match = crate::device::correction::same_region(
                            &capture.image,
                            &next.image,
                            profile.screen.board,
                        ) && profile.screen.slots.iter().all(|&r| {
                            crate::device::correction::same_region(&capture.image, &next.image, r)
                        });
                        if semantic_match || image_match {
                            capture = next;
                            matched = true;
                            break;
                        }
                        capture = next;
                    }
                    if !matched {
                        return Err("Recovery needs consecutive matching screen observations. Wait, then Scan again.".to_owned());
                    }
                }
                let (serial, journal) = {
                    let data = self.data().map_err(|e| e.message)?;
                    (
                        data.device.serial.clone().ok_or("Connect first")?,
                        data.device.journal.clone(),
                    )
                };
                journal
                    .lock()
                    .map_err(|_| "Execution journal is unavailable")?
                    .resynchronize(&serial)?;
                self.accept_capture(capture, &stop)?;
            }
            DeviceAction::CaptureForEdit { .. } => {
                let capture = self.raw_capture(&mut *connection.transport, &profile, &stop)?;
                let mut data = self.data().map_err(|e| e.message)?;
                data.device.capture = Some(capture);
                data.device.set(
                    "connected",
                    "Phone image refreshed for input correction. No movements sent.",
                );
                data.refresh_device_readiness();
            }
            DeviceAction::NextCycle { .. } => {
                let (snapshot, previous) = {
                    let data = self.data().map_err(|e| e.message)?;
                    (
                        data.workspace.snapshot(),
                        data.device.observation.clone().ok_or("Scan first")?,
                    )
                };
                let plan = snapshot
                    .plan
                    .as_ref()
                    .ok_or("Complete the current plan first")?;
                let (next, observed) = self.next_tray(
                    &mut *connection.transport,
                    &profile,
                    &previous.tray,
                    plan.candidate.board,
                    &stop,
                )?;
                {
                    let mut data = self.data().map_err(|e| e.message)?;
                    data.workspace
                        .next_cycle(snapshot.revision)
                        .map_err(|_| "Cycle changed")?;
                    data.device.cycle_done = false;
                    data.device.capture = None;
                    data.device.corrections = None;
                }
                self.scan_solve(next, Some(plan.candidate.states), observed, &stop)?;
            }
            DeviceAction::Start { through, .. } => {
                self.play(&mut *connection.transport, through)?
            }
            _ => return Err("Unsupported device operation".to_owned()),
        }
        Ok(())
    }
    fn raw_capture(
        &self,
        transport: &mut dyn Transport,
        profile: &Profile,
        stop: &AtomicBool,
    ) -> Result<crate::device::correction::Capture, String> {
        let image = transport.capture(&profile.screen, false, stop)?;
        let id = {
            let mut data = self.data().map_err(|e| e.message)?;
            data.device.next_capture += 1;
            data.device.next_capture
        };
        let capture = crate::device::correction::Capture::read(id, image, &profile.screen)?;
        if capture.board.is_none() || !capture.tray.readable() {
            let store = self.data().map_err(|e| e.message)?.store.clone();
            if let Err(error) = crate::preferences::save_diagnostic(
                &store,
                &capture.image,
                &format!(
                    "Partial input; board readable={}; tray={:?}",
                    capture.board.is_some(),
                    capture.tray.slots
                ),
            ) {
                self.data().map_err(|e| e.message)?.device.message =
                    format!("Partial input retained; diagnostics could not be saved: {error}");
            }
        }
        Ok(capture)
    }
    fn capture(
        &self,
        transport: &mut dyn Transport,
        profile: &Profile,
        stop: &AtomicBool,
    ) -> Result<Observation, String> {
        let (expected, signature) = {
            let data = self.data().map_err(|e| e.message)?;
            let s = data.workspace.snapshot();
            let signature = data.device.observation.as_ref().map(|o| {
                s.plan.as_ref().map_or_else(
                    || o.tray.signature(),
                    |p| Self::expected_signature(&o.tray, p, usize::from(s.completed)),
                )
            });
            (data.workspace.current_board(), signature)
        };
        self.capture_expected(transport, profile, stop, expected, signature.as_deref())
    }
    fn capture_expected(
        &self,
        transport: &mut dyn Transport,
        profile: &Profile,
        stop: &AtomicBool,
        expected: justblast_core::domain::Board,
        signature: Option<&[SlotSignature]>,
    ) -> Result<Observation, String> {
        let capture = self
            .raw_capture(transport, profile, stop)
            .map_err(|e| format!("Capture failed: {e}"))?;
        let corrections = self
            .data()
            .map_err(|e| e.message)?
            .device
            .corrections
            .clone();
        self.data().map_err(|e| e.message)?.device.capture = Some(capture.clone());
        let observation = if let Some(c) = corrections {
            c.resolve(&capture, &profile.screen, expected, signature)?
        } else {
            let observation = capture
                .observation()
                .ok_or("Board is unreadable. Scan and correct it.")?;
            if !observation.tray.readable() {
                return Err("A tray slot is unreadable. Scan and correct it.".to_owned());
            }
            observation
        };
        self.data().map_err(|e| e.message)?.device.capture = Some(capture);
        Ok(observation)
    }
    fn scan_solve(
        &self,
        observation: Observation,
        states: Option<[ComboState; 3]>,
        observed: bool,
        stop: &AtomicBool,
    ) -> Result<(), String> {
        let (revision, game) = {
            let mut data = self.data().map_err(|e| e.message)?;
            let snapshot = data.workspace.snapshot();
            let pieces = observation
                .tray
                .pieces()
                .into_iter()
                .map(|p| Piece {
                    slot: p.slot,
                    catalog_id: (p.catalog_id > 0).then_some(p.catalog_id),
                    name: format!("Piece {}", p.slot + 1),
                    shape: p.shape,
                })
                .collect();
            let states =
                states.unwrap_or([justblast_core::domain::combo::ComboState::default(); 3]);
            let accepted = if observed {
                data.workspace.accept_observation(
                    snapshot.revision,
                    observation.board,
                    pieces,
                    states,
                    justblast_core::domain::combo::RuleContext::default(),
                    true,
                )
            } else {
                data.workspace.accept_prediction(
                    snapshot.revision,
                    observation.board,
                    pieces,
                    states,
                    justblast_core::domain::combo::RuleContext::default(),
                )
            };
            accepted.map_err(|e| format!("Scan input was rejected: {e:?}"))?;
            let revision = data.workspace.snapshot().revision;
            let (revision, game) = data
                .workspace
                .begin_search(revision)
                .map_err(|e| format!("Search input was rejected: {e:?}"))?;
            data.device.observation = Some(observation);
            data.device.corrections = None;
            data.device.capture = None;
            data.device.cycle_done = false;
            data.device.completed = 0;
            data.device.provenance = if observed { "observed" } else { "predicted" };
            data.device.set("solving", "Calculating a complete plan…");
            data.invalidate();
            (revision, game)
        };
        let started = Instant::now();
        let result = solver::solve(
            &game,
            &evaluation_shapes(),
            SearchOptions::default(),
            |_| !stop.load(Ordering::Acquire) && started.elapsed() < Duration::from_secs(30),
        )
        .map_err(str::to_owned)?;
        if stop.load(Ordering::Acquire) {
            return Err("Stopped. Scan before starting again.".to_owned());
        }
        let complete = result.complete && result.steps.len() == game.pieces.len();
        let mut data = self.data().map_err(|e| e.message)?;
        data.workspace
            .accept_search(revision, result)
            .map_err(|_| "The workspace changed during device search")?;
        data.device.needs_scan = !complete;
        data.device.set(
            if complete { "ready" } else { "no_solution" },
            if complete {
                "Plan ready. Play will recheck the screen before moving."
            } else {
                "No complete plan accepted. No movement sent; check the game and Scan again."
            },
        );
        Ok(())
    }
    fn verify(
        &self,
        transport: &mut dyn Transport,
        profile: &Profile,
        stop: &AtomicBool,
        board: justblast_core::domain::Board,
        signature: Option<&[SlotSignature]>,
    ) -> Result<Observation, String> {
        let mut reason = "Phone position differs from the plan.".to_owned();
        for attempt in 0..profile.runtime.tray_attempts {
            if attempt > 0 {
                runtime::wait(Duration::from_secs_f64(profile.runtime.retry_wait), stop)?;
            }
            match self.capture_expected(transport, profile, stop, board, signature) {
                Ok(observation)
                    if observation.board == board
                        && signature.is_none_or(|s| observation.tray.signature() == s) =>
                {
                    return Ok(observation);
                }
                Ok(_) => {}
                Err(error) => {
                    if stop.load(Ordering::Acquire) || error.starts_with("Capture failed:") {
                        return Err(error);
                    }
                    reason = error;
                }
            }
        }
        self.confirm_position_warning(profile, stop, board, signature, &reason)
    }
    fn confirm_position_warning(
        &self,
        profile: &Profile,
        stop: &AtomicBool,
        board: justblast_core::domain::Board,
        signature: Option<&[SlotSignature]>,
        reason: &str,
    ) -> Result<Observation, String> {
        {
            let mut data = self.data().map_err(|e| e.message)?;
            data.device.warning_accepted = false;
            data.device.set("awaiting_confirmation", format!("{reason} Accept the planned position and continue, or Stop to edit/Scan. Completed moves will not repeat."));
        }
        loop {
            runtime::wait(Duration::from_millis(50), stop)?;
            let mut data = self.data().map_err(|e| e.message)?;
            if data.device.stop.load(Ordering::Acquire) {
                return Err("Stopped at position warning. No remaining moves sent.".to_owned());
            }
            if !data.device.warning_accepted {
                continue;
            }
            data.device.warning_accepted = false;
            let capture = data
                .device
                .capture
                .clone()
                .ok_or("No screen captured. Scan again.")?;
            let mut tray = capture.tray.clone();
            for slot in 0..3 {
                tray.slots[slot] = match signature.and_then(|s| s.get(slot)) {
                    Some(SlotSignature::Empty) => vision::Slot::Empty,
                    Some(SlotSignature::Shape(shape)) => {
                        let baseline = data
                            .device
                            .observation
                            .as_ref()
                            .and_then(|o| o.tray.slots[slot].candidate())
                            .filter(|p| &p.shape == shape)
                            .ok_or(
                                "The expected piece has no drag origin. Stop and edit this slot.",
                            )?;
                        vision::Slot::Recognized(baseline.clone())
                    }
                    None | Some(SlotSignature::Unknown) => match &tray.slots[slot] {
                        vision::Slot::Uncertain {
                            candidate: Some(piece),
                            ..
                        } => vision::Slot::Recognized(piece.clone()),
                        vision::Slot::Uncertain {
                            candidate: None, ..
                        } => return Err(
                            "No shape detected in a new slot. Edit or clear it before continuing."
                                .to_owned(),
                        ),
                        other => other.clone(),
                    },
                };
            }
            let cells = (0..8)
                .flat_map(|r| (0..8).map(move |c| (r, c)))
                .map(|(r, c)| (r as u8, c as u8, board.rows()[r].as_bytes()[c] == b'#'))
                .collect();
            data.device.corrections = Some(crate::device::correction::Corrections {
                capture,
                cells,
                slots: std::array::from_fn(|i| Some(tray.slots[i].clone())),
            });
            data.device.position_overridden = true;
            data.device.provenance = "predicted";
            data.device.set(
                "running",
                "Position warning accepted. Continuing without replaying completed moves.",
            );
            let _ = profile;
            return Ok(Observation { board, tray });
        }
    }

    fn expected_signature(
        tray: &Tray,
        plan: &SearchResult,
        completed: usize,
    ) -> Vec<SlotSignature> {
        let mut expected = tray.signature();
        for step in &plan.steps[..completed] {
            expected[usize::from(step.slot)] = SlotSignature::Empty;
        }
        expected
    }
    fn send(
        &self,
        transport: &mut dyn Transport,
        game: &Game,
        plan: &SearchResult,
        tray: &Tray,
        range: std::ops::Range<usize>,
    ) -> Result<(), String> {
        let (profile, force) = {
            let data = self.data().map_err(|e| e.message)?;
            (data.device.profile.clone(), data.device.force.clone())
        };
        let (actions, delays) = touch::prepare_range(game, plan, tray, &profile, range.clone())?;
        let first = range.start;
        let last = range.end;
        let (serial, snapshot, journal) = {
            let data = self.data().map_err(|e| e.message)?;
            if data.device.stop.load(Ordering::Acquire) {
                return Err("Stopped before sending movement. Scan again.".to_owned());
            }
            (
                data.device.serial.clone().ok_or("Connect first")?,
                data.workspace.snapshot(),
                data.device.journal.clone(),
            )
        };
        journal
            .lock()
            .map_err(|_| "Execution journal is unavailable")?
            .intent(
                &serial,
                snapshot.revision,
                snapshot.cycle,
                first as u8,
                (last - 1) as u8,
                plan.steps[last - 1].result.board,
            )?;
        {
            let mut data = self.data().map_err(|e| e.message)?;
            if data.device.stop.load(Ordering::Acquire) {
                return Err("Stopped before sending movement. Scan again.".to_owned());
            }
            data.device.provenance = "predicted";
            data.device.set(
                "running",
                format!("Sending moves {}–{last} as one batch…", first + 1),
            );
        }
        let mut progress_error = None;
        transport.execute(&actions, &delays, &force, &mut |count| {
            if first + count > last {
                progress_error = Some("Device acknowledgement exceeds the authorized batch");
                return;
            }
            match self.data() {
                Ok(mut data) => {
                    if data.workspace.acknowledge((first + count) as u8).is_err() {
                        progress_error = Some("Acknowledged prefix was rejected");
                    } else {
                        data.device.progress((first + count) as u8);
                    }
                }
                Err(_) => progress_error = Some("Acknowledgement could not be published"),
            }
        })?;
        if let Some(error) = progress_error {
            return Err(error.to_owned());
        }
        journal
            .lock()
            .map_err(|_| "Execution journal is unavailable")?
            .acknowledge(&serial)?;
        {
            let mut data = self.data().map_err(|e| e.message)?;
            data.device.progress(last as u8);
            data.workspace
                .acknowledge(last as u8)
                .map_err(|_| "Acknowledged prefix was rejected")?;
        }
        if touch::needs_clear_wait(&plan.steps[last - 1].result) {
            runtime::wait(Duration::from_secs_f64(touch::CLEAR_SETTLE_SECONDS), &force)?;
        }
        Ok(())
    }
    fn next_tray(
        &self,
        transport: &mut dyn Transport,
        profile: &Profile,
        previous: &Tray,
        board: justblast_core::domain::Board,
        stop: &AtomicBool,
    ) -> Result<(Observation, bool), String> {
        for attempt in 0..profile.runtime.tray_attempts {
            if attempt > 0 {
                runtime::wait(Duration::from_secs_f64(profile.runtime.retry_wait), stop)?;
            }
            let image = transport.capture(&profile.screen, true, stop)?;
            let rect = profile.screen.tray();
            let tray = vision::read_tray(&image, &profile.screen, (rect[0], rect[1]))?;
            if tray.complete() && tray.signature() != previous.signature() {
                return Ok((Observation { board, tray }, false));
            }
        }
        Ok((self.capture(transport, profile, stop)?, true))
    }
    fn play(&self, transport: &mut dyn Transport, through: Option<u8>) -> Result<(), String> {
        let (profile, options, stop, force) = {
            let data = self.data().map_err(|e| e.message)?;
            (
                data.device.profile.clone(),
                data.preferences.automation.clone(),
                data.device.stop.clone(),
                data.device.force.clone(),
            )
        };
        let (expected_board, expected_slots) = {
            let data = self.data().map_err(|e| e.message)?;
            let snapshot = data.workspace.snapshot();
            let plan = snapshot.plan.as_ref().ok_or("Calculate a plan first")?;
            let observation = data.device.observation.as_ref().ok_or("Scan first")?;
            (
                data.workspace.current_board(),
                Self::expected_signature(
                    &observation.tray,
                    plan,
                    usize::from(data.device.completed),
                ),
            )
        };
        let mut fresh = self.verify(
            transport,
            &profile,
            &stop,
            expected_board,
            Some(&expected_slots),
        )?;
        let mut cycles = 0;
        loop {
            let (snapshot, game, observation, first) = {
                let data = self.data().map_err(|e| e.message)?;
                (
                    data.workspace.snapshot(),
                    data.workspace.game(),
                    data.device.observation.clone().ok_or("Scan first")?,
                    usize::from(data.device.completed),
                )
            };
            let plan = snapshot.plan.as_ref().ok_or("Scan first")?;
            let expected_board = game.initial().board;
            let expected_signature = Self::expected_signature(&observation.tray, plan, first);
            if fresh.board != expected_board || fresh.tray.signature() != expected_signature {
                if first > 0 {
                    return Err("Phone changed after an executed move. Scan to recover; no move was repeated.".to_owned());
                }
                self.scan_solve(fresh, None, true, &stop)?;
                self.data().map_err(|e| e.message)?.device.set(
                    "ready",
                    "The phone changed. A new plan is ready; press Play again.",
                );
                return Ok(());
            }
            {
                let mut data = self.data().map_err(|e| e.message)?;
                if data
                    .device
                    .visuals
                    .back()
                    .is_none_or(|v| v.snapshot.revision != snapshot.revision)
                {
                    data.device.queue(snapshot.clone())?;
                }
            }
            let end = through.map_or(plan.steps.len(), |i| usize::from(i) + 1);
            if !options.check_after_move {
                self.send(transport, &game, plan, &fresh.tray, first..end)?;
            } else {
                for index in first..end {
                    if index > first {
                        runtime::wait(Duration::from_secs_f64(profile.runtime.tray_wait), &stop)?;
                    }
                    let expected = Self::expected_signature(&observation.tray, plan, index);
                    let current = self.verify(
                        transport,
                        &profile,
                        &stop,
                        plan.steps[index].before,
                        Some(&expected),
                    )?;
                    self.send(transport, &game, plan, &current.tray, index..index + 1)?;
                    if options.check_after_move && index + 1 < plan.steps.len() {
                        runtime::wait(Duration::from_secs_f64(profile.runtime.tray_wait), &stop)?;
                        let expected = Self::expected_signature(&observation.tray, plan, index + 1);
                        self.verify(
                            transport,
                            &profile,
                            &stop,
                            plan.steps[index].result.board,
                            Some(&expected),
                        )?;
                    }
                }
            }
            if end < plan.steps.len() {
                let mut data = self.data().map_err(|e| e.message)?;
                data.device.set(
                    "ready",
                    format!("Move {end} acknowledged. Executed moves are locked."),
                );
                return Ok(());
            }
            if force.load(Ordering::Acquire) {
                return Err("Forced stop. Device result is uncertain; Scan again.".to_owned());
            }
            let finishing = force.clone();
            runtime::wait(
                Duration::from_secs_f64(profile.runtime.tray_wait),
                &finishing,
            )?;
            let check = options.check_after_move || options.check_after_cycle;
            self.data().map_err(|e| e.message)?.device.set(
                if check { "verifying" } else { "scanning" },
                if check {
                    "Verifying the resulting board…"
                } else {
                    "Reading the tray for the next cycle…"
                },
            );
            let should_continue = options.continuous
                && through.is_none()
                && (options.cycle_limit == 0 || cycles + 1 < options.cycle_limit)
                && !stop.load(Ordering::Acquire);
            let (next, observed) = if check {
                (
                    self.verify(transport, &profile, &finishing, plan.candidate.board, None)?,
                    true,
                )
            } else if should_continue {
                self.next_tray(
                    transport,
                    &profile,
                    &observation.tray,
                    plan.candidate.board,
                    &finishing,
                )?
            } else {
                (
                    Observation {
                        board: plan.candidate.board,
                        tray: Tray {
                            slots: std::array::from_fn(|_| vision::Slot::Empty),
                        },
                    },
                    false,
                )
            };
            if force.load(Ordering::Acquire) {
                return Err("Forced stop. Device result is uncertain; Scan again.".to_owned());
            }
            let states = if next.board == plan.candidate.board {
                plan.candidate.states
            } else if next.board == game.board {
                game.states
            } else {
                [justblast_core::domain::combo::ComboState::default(); 3]
            };
            {
                let mut data = self.data().map_err(|e| e.message)?;
                let mut history = data.history.clone();
                history.push(HistoryEntry {
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                        .to_string(),
                    source: "phone".to_owned(),
                    cycle: snapshot.cycle,
                    status: if data.device.position_overridden {
                        "accepted_warning"
                    } else if check {
                        "verified"
                    } else {
                        "acknowledged"
                    }
                    .to_owned(),
                    message: format!(
                        "{} placements; board {}",
                        plan.steps.len(),
                        if observed { "observed" } else { "predicted" }
                    ),
                    board: next.board.rows(),
                });
                if history.len() > 200 {
                    history.drain(..history.len() - 200);
                }
                data.store.save_history(&history)?;
                data.history = history;
                data.device.cycle_done = true;
                if should_continue {
                    data.workspace
                        .next_cycle(snapshot.revision)
                        .map_err(|_| "Cycle changed during execution")?;
                    data.device.capture = None;
                    data.device.corrections = None;
                }
                if let Some(visual) = data.device.visuals.back_mut() {
                    visual.done = true;
                }
            }
            cycles += 1;
            if !should_continue {
                let mut data = self.data().map_err(|e| e.message)?;
                data.device.set(
                    "cycle_completed",
                    "Cycle completed. Next cycle reads the next tray without moving.",
                );
                return Ok(());
            }
            if stop.load(Ordering::Acquire) {
                let mut data = self.data().map_err(|e| e.message)?;
                data.device.needs_scan = true;
                data.device
                    .set("connected", "Stopped after the cycle. Scan when ready.");
                return Ok(());
            }
            let observed = observed
                && !self
                    .data()
                    .map_err(|e| e.message)?
                    .device
                    .position_overridden;
            self.scan_solve(next.clone(), Some(states), observed, &stop)?;
            self.data()
                .map_err(|e| e.message)?
                .device
                .position_overridden = false;
            fresh = next;
            if through.is_some()
                || !options.continuous
                || (options.cycle_limit > 0 && cycles >= options.cycle_limit)
                || self.data().map_err(|e| e.message)?.device.state != "ready"
            {
                return Ok(());
            }
        }
    }
    pub fn shutdown(&self) {
        if let Ok(mut data) = self.data() {
            data.device.shutdown = true;
            data.device.stop.store(true, Ordering::Release);
            data.device.force.store(true, Ordering::Release);
            data.invalidate();
        }
        let started = Instant::now();
        while started.elapsed() < Duration::from_secs(3) {
            if let Ok(mut connection) = self.1.try_lock() {
                connection.transport.close();
                connection.lease = None;
                return;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}
