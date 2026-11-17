use super::{
    profile::Profile,
    transport::{DeviceInfo, Transport},
    vision::Observation,
};
use crate::preferences::{Store, atomic_write};
use justblast_core::{domain::Board, services::WorkspaceSnapshot};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    fs::{self, File},
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

pub struct Connection {
    pub transport: Box<dyn Transport>,
    pub lease: Option<File>,
}
#[derive(Clone)]
pub struct VisualCycle {
    pub id: u32,
    pub snapshot: WorkspaceSnapshot,
    pub completed: u8,
    pub done: bool,
}
pub struct Runtime {
    pub capture: Option<super::correction::Capture>,
    pub corrections: Option<super::correction::Corrections>,
    pub next_capture: u32,
    pub accepted_slots: [bool; 3],
    pub board_accepted: bool,
    pub warning_accepted: bool,
    pub position_overridden: bool,
    pub cycle_done: bool,
    pub link_error: String,
    pub revision: u32,
    pub state: &'static str,
    pub message: String,
    pub error: Option<String>,
    pub busy: bool,
    pub serial: Option<String>,
    pub devices: Vec<DeviceInfo>,
    pub completed: u8,
    pub needs_scan: bool,
    pub provenance: &'static str,
    pub stop: Arc<AtomicBool>,
    pub force: Arc<AtomicBool>,
    pub profile: Profile,
    pub profile_id: String,
    pub profile_ids: Vec<String>,
    pub observation: Option<Observation>,
    pub commands: HashMap<String, u64>,
    pub visuals: VecDeque<VisualCycle>,
    pub next_visual: u32,
    pub dropped_visuals: u32,
    pub journal: Arc<Mutex<Journal>>,
    pub shutdown: bool,
}
impl Default for Runtime {
    fn default() -> Self {
        Self {
            capture: None,
            corrections: None,
            next_capture: 0,
            accepted_slots: [false; 3],
            board_accepted: false,
            warning_accepted: false,
            position_overridden: false,
            cycle_done: false,
            link_error: "Scan to link this input to your phone.".to_owned(),
            revision: 1,
            state: "disconnected",
            message: "Select a phone in Settings. Discovery only runs when requested.".to_owned(),
            error: None,
            busy: false,
            serial: None,
            devices: Vec::new(),
            completed: 0,
            needs_scan: true,
            provenance: "none",
            stop: Arc::new(AtomicBool::new(false)),
            force: Arc::new(AtomicBool::new(false)),
            profile: Profile::builtin(),
            profile_id: "default".to_owned(),
            profile_ids: vec!["default".to_owned()],
            observation: None,
            commands: HashMap::new(),
            visuals: VecDeque::new(),
            next_visual: 0,
            dropped_visuals: 0,
            journal: Arc::new(Mutex::new(Journal::default())),
            shutdown: false,
        }
    }
}
impl Runtime {
    pub fn uncertain(&self, serial: &str) -> bool {
        self.journal
            .lock()
            .map_or(true, |journal| journal.uncertain(serial))
    }

    pub fn set(&mut self, state: &'static str, message: impl Into<String>) {
        self.state = state;
        self.message = message.into();
        self.revision += 1;
    }
    pub fn reset_flags(&mut self) {
        self.warning_accepted = false;
        self.stop.store(false, Ordering::Release);
        self.force.store(false, Ordering::Release);
        self.error = None;
    }
    pub fn queue(&mut self, snapshot: WorkspaceSnapshot) -> Result<u32, String> {
        self.visuals.clear();
        self.next_visual += 1;
        let id = self.next_visual;
        self.visuals.push_back(VisualCycle {
            id,
            snapshot,
            completed: self.completed,
            done: false,
        });
        Ok(id)
    }
    pub fn progress(&mut self, completed: u8) {
        self.completed = completed;
        if let Some(cycle) = self.visuals.back_mut() {
            cycle.completed = completed;
        }
        self.revision += 1;
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Intent {
    serial: String,
    plan_revision: u32,
    cycle: u32,
    first: u8,
    last: u8,
    expected_board: [String; 8],
    pending: bool,
}
#[derive(Default)]
pub struct Journal {
    path: Option<PathBuf>,
    intents: HashMap<String, Intent>,
    corrupt: Option<String>,
}
impl Journal {
    pub fn recovering(store: &Store) -> Self {
        Self::at(store).unwrap_or_else(|error|Self{path:store.root().map(|p|p.join("execution.json")),intents:HashMap::new(),corrupt:Some(format!("Execution history is damaged. Preserve and repair execution.json before moving the phone: {error}"))})
    }
    pub fn error(&self) -> Option<&str> {
        self.corrupt.as_deref()
    }

    pub fn at(store: &Store) -> Result<Self, String> {
        let path = store.root().map(|p| p.join("execution.json"));
        let mut intents = HashMap::new();
        if let Some(path) = &path
            && path.exists()
        {
            if fs::metadata(path).map_err(|e| e.to_string())?.len() > 1_000_000 {
                return Err("Execution journal is too large".to_owned());
            }
            let bytes = fs::read(path).map_err(|e| e.to_string())?;
            if bytes.len() > 1_000_000 {
                return Err(
                    "Execution journal is too large; preserve it and inspect before recovery"
                        .to_owned(),
                );
            }
            intents = serde_json::from_slice(&bytes)
                .map_err(|e| format!("Execution journal could not be read: {e}"))?;
        }
        Ok(Self {
            path,
            intents,
            corrupt: None,
        })
    }
    fn save(&self) -> Result<(), String> {
        if let Some(error) = &self.corrupt {
            return Err(error.clone());
        }
        if let Some(path) = &self.path {
            atomic_write(
                path,
                &serde_json::to_vec(&self.intents).map_err(|e| e.to_string())?,
            )?;
        }
        Ok(())
    }
    pub fn uncertain(&self, serial: &str) -> bool {
        self.corrupt.is_some() || self.intents.get(serial).is_some_and(|i| i.pending)
    }
    pub fn intent(
        &mut self,
        serial: &str,
        revision: u32,
        cycle: u32,
        first: u8,
        last: u8,
        board: Board,
    ) -> Result<(), String> {
        if self.uncertain(serial) {
            return Err("Unresolved execution intent requires a stable Scan".to_owned());
        }
        self.intents.insert(
            serial.to_owned(),
            Intent {
                serial: serial.to_owned(),
                plan_revision: revision,
                cycle,
                first,
                last,
                expected_board: board.rows(),
                pending: true,
            },
        );
        self.save()
            .map_err(|e| format!("Execution intent could not be saved. No movement sent: {e}"))
    }
    pub fn acknowledge(&mut self, serial: &str) -> Result<(), String> {
        if let Some(intent) = self.intents.get_mut(serial) {
            intent.pending = false;
        }
        if let Err(error) = self.save() {
            if let Some(intent) = self.intents.get_mut(serial) {
                intent.pending = true;
            }
            return Err(format!(
                "Acknowledgement could not be saved. Scan before continuing: {error}"
            ));
        }
        Ok(())
    }
    pub fn resynchronize(&mut self, serial: &str) -> Result<(), String> {
        if let Some(error) = &self.corrupt {
            return Err(error.clone());
        }
        let previous = self.intents.remove(serial);
        if let Err(error) = self.save() {
            if let Some(previous) = previous {
                self.intents.insert(serial.to_owned(), previous);
            }
            return Err(error);
        }
        Ok(())
    }
}
pub fn lease(store: &Store, serial: &str) -> Result<Option<File>, String> {
    let Some(root) = store.root() else {
        return Ok(None);
    };
    let name = serial
        .bytes()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(root.join(format!("device-{name}.lock")))
        .map_err(|e| e.to_string())?;
    file.try_lock().map_err(|_| {
        "This phone is owned by another JustBlast instance. Disconnect it there first.".to_owned()
    })?;
    Ok(Some(file))
}
pub fn wait(duration: Duration, stop: &AtomicBool) -> Result<(), String> {
    let started = Instant::now();
    while started.elapsed() < duration {
        if stop.load(Ordering::Acquire) {
            return Err("Stopped. Scan before starting again.".to_owned());
        }
        std::thread::sleep(
            duration
                .saturating_sub(started.elapsed())
                .min(Duration::from_millis(20)),
        );
    }
    Ok(())
}
