use crate::api::models::*;
use justblast_core::{
    domain::catalog::evaluation_shapes,
    services::Workspace,
    solver::{self, ExplorerIndex, Game, SearchOptions},
};
use std::sync::{
    Arc, Mutex, MutexGuard,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
use std::time::{Duration, Instant};
mod calibration;
mod input;
mod phone;
use crate::device::{
    runtime::{Connection, Runtime},
    transport::Adb,
};

struct Job {
    id: u32,
    kind: &'static str,
    revision: u32,
    cancelled: Arc<AtomicBool>,
    progress: Arc<AtomicU64>,
    status: JobStatus,
    message: Option<String>,
}
struct Indexed {
    revision: u32,
    game: Game,
    index: Arc<ExplorerIndex>,
}
#[derive(Default)]
struct Data {
    workspace: Workspace,
    calibration: Option<calibration::CalibrationCapture>,
    calibration_commands: std::collections::HashSet<String>,
    job: Option<Job>,
    explorer: Option<Indexed>,
    next_job: u32,
    store: crate::preferences::Store,
    preferences: Preferences,
    history: Vec<HistoryEntry>,
    device: Runtime,
    _instance_lease: Option<std::fs::File>,
}
#[derive(Clone)]
pub struct NativeState(Arc<Mutex<Data>>, Arc<Mutex<Connection>>);
impl Default for NativeState {
    fn default() -> Self {
        Self(
            Arc::new(Mutex::new(Data::default())),
            Arc::new(Mutex::new(Connection {
                transport: Box::new(Adb::default()),
                lease: None,
            })),
        )
    }
}
impl Data {
    fn view(&self) -> WorkspaceView {
        let mut view: WorkspaceView = self.workspace.snapshot().into();
        view.job = self.job.as_ref().map(|j| JobView {
            id: j.id,
            kind: j.kind.to_owned(),
            status: j.status.clone(),
            visited: j.progress.load(Ordering::Relaxed).min(u64::from(u32::MAX)) as u32,
            message: j.message.clone(),
        });
        view.device = self.device_view();
        view.current_board = self.workspace.current_board().rows();
        view.consumed_slots = self
            .workspace
            .game()
            .checkpoint
            .as_ref()
            .map_or_else(Vec::new, |c| c.steps.iter().map(|s| s.slot).collect());
        view.explorer_ready = self
            .explorer
            .as_ref()
            .is_some_and(|e| e.revision == view.revision);
        view
    }
    fn invalidate(&mut self) {
        if let Some(job) = &mut self.job
            && matches!(job.status, JobStatus::Running)
        {
            job.cancelled.store(true, Ordering::Release);
            job.status = JobStatus::Cancelled;
        }
        self.explorer = None;
    }
}
impl NativeState {
    pub fn with_store(store: crate::preferences::Store) -> Result<Self, String> {
        let instance_lease = if let Some(root) = store.root() {
            let file = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(false)
                .open(root.join("justblast.lock"))
                .map_err(|e| e.to_string())?;
            file.try_lock()
                .map_err(|_| "JustBlast is already running. Use the existing window.".to_owned())?;
            Some(file)
        } else {
            None
        };
        let preferences = store.load_preferences()?;
        let history = store.load_history()?;
        let journal = crate::device::runtime::Journal::recovering(&store);
        let journal_error = journal.error().map(str::to_owned);
        let mut device = Runtime {
            journal: Arc::new(Mutex::new(journal)),
            ..Runtime::default()
        };
        if let Some(error) = journal_error {
            device.message = error.clone();
            device.error = Some(error);
        }
        device.profile_ids = crate::device::profile::list(&store)?;
        device.profile_id = preferences.connection.profile_id.clone();
        device.profile = crate::device::profile::load(&store, &device.profile_id)?;
        let mut workspace = Workspace::default();
        let revision = workspace.snapshot().revision;
        workspace
            .set_weights(revision, preferences.weights.clone().into())
            .map_err(|_| "Invalid saved weights")?;
        Ok(Self(
            Arc::new(Mutex::new(Data {
                workspace,
                _instance_lease: instance_lease,
                store,
                preferences,
                history,
                device,
                ..Data::default()
            })),
            Arc::new(Mutex::new(Connection {
                transport: Box::new(Adb::default()),
                lease: None,
            })),
        ))
    }
    pub fn preferences(&self) -> Result<Preferences, CommandError> {
        Ok(self.data()?.preferences.clone())
    }
    pub fn save_preferences(&self, mut prefs: Preferences) -> Result<Preferences, CommandError> {
        prefs.validate().map_err(CommandError::invalid)?;
        let mut data = self.data()?;
        if prefs.revision != data.preferences.revision {
            return Err(CommandError {
                code: ErrorCode::StaleRevision,
                message: "Preferences changed. Reload them before saving.".to_owned(),
            });
        }
        if data.device.busy && prefs.automation != data.preferences.automation {
            return Err(CommandError {
                code: ErrorCode::Busy,
                message: "Stop before changing device controls".to_owned(),
            });
        }
        if prefs.connection.profile_id != data.device.profile_id {
            return Err(CommandError::invalid(
                "Select calibration using the device controls",
            ));
        }
        let changed_weights = data.preferences.weights != prefs.weights;
        if changed_weights && data.device.busy {
            return Err(CommandError {
                code: ErrorCode::Busy,
                message: "Stop before changing scoring weights".to_owned(),
            });
        }
        prefs.advance().map_err(CommandError::invalid)?;
        data.store
            .save_preferences(&prefs)
            .map_err(|message| CommandError {
                code: ErrorCode::Storage,
                message,
            })?;
        if changed_weights {
            let revision = data.workspace.snapshot().revision;
            data.workspace
                .set_weights(revision, prefs.weights.clone().into())?;
            data.invalidate();
        }
        data.preferences = prefs;
        Ok(data.preferences.clone())
    }
    pub fn history(&self) -> Result<Vec<HistoryEntry>, CommandError> {
        Ok(self.data()?.history.clone())
    }

    fn data(&self) -> Result<MutexGuard<'_, Data>, CommandError> {
        self.0.lock().map_err(|_| CommandError {
            code: ErrorCode::Internal,
            message: "The workspace is unavailable. Restart JustBlast.".to_owned(),
        })
    }
    pub fn snapshot(&self) -> Result<WorkspaceView, CommandError> {
        Ok(self.data()?.view())
    }
    pub fn replace_board(
        &self,
        request: ReplaceBoardRequest,
    ) -> Result<WorkspaceView, CommandError> {
        let board = parse_board(&request.rows)?;
        let mut data = self.data()?;
        if data.device.busy {
            return Err(CommandError {
                code: ErrorCode::Busy,
                message: "Stop the active device operation first".to_owned(),
            });
        }
        let before = data.workspace.snapshot().revision;
        data.workspace
            .replace_board(request.expected_revision, board)?;
        if data.workspace.snapshot().revision != before {
            data.invalidate();
            data.device.observation = None;
            data.device.needs_scan = true;
        }
        Ok(data.view())
    }
    pub fn action(&self, action: WorkspaceAction) -> Result<WorkspaceView, CommandError> {
        match action {
            WorkspaceAction::Search {
                expected_revision,
                max_nodes,
            } => return self.start_job(expected_revision, 0, max_nodes, false),
            WorkspaceAction::Explore {
                expected_revision,
                completed,
                max_nodes,
            } => return self.start_job(expected_revision, completed, max_nodes, true),
            _ => {}
        }
        let mut data = self.data()?;
        if data.device.busy {
            return Err(CommandError {
                code: ErrorCode::Busy,
                message: "Stop the active device operation first".to_owned(),
            });
        }
        match action {
            WorkspaceAction::ApplyInput {
                expected_revision,
                rows,
                pieces,
                anchors,
            } => {
                return self.apply_input_locked(
                    data,
                    expected_revision,
                    *rows,
                    pieces,
                    anchors,
                    None,
                );
            }
            WorkspaceAction::UpdateInput {
                expected_revision,
                rows,
                pieces,
                anchors,
                accepted_slots,
                accept_board,
            } => {
                data.workspace.check_revision(expected_revision)?;
                if accepted_slots.iter().any(|&slot| slot > 2) {
                    return Err(CommandError::invalid("Unknown slot"));
                }
                let mut accepted = data.device.accepted_slots;
                for slot in accepted_slots {
                    accepted[usize::from(slot)] = true;
                }
                let board_accepted = accept_board || data.device.board_accepted;
                return self.apply_input_locked(
                    data,
                    expected_revision,
                    *rows,
                    pieces,
                    anchors,
                    Some((accepted, board_accepted)),
                );
            }
            WorkspaceAction::Pieces {
                expected_revision,
                pieces,
            } => {
                let pieces = pieces
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<_>, _>>()?;
                data.workspace.replace_pieces(expected_revision, pieces)?;
                data.invalidate();
                data.device.needs_scan = true;
                data.device.observation = None;
            }
            WorkspaceAction::Cancel { expected_revision } => {
                data.workspace.cancel_search(expected_revision)?;
                data.invalidate();
            }
            WorkspaceAction::NextCycle { expected_revision } => {
                let before = data.workspace.snapshot();
                data.workspace.check_revision(expected_revision)?;
                let plan = before
                    .plan
                    .as_ref()
                    .ok_or_else(|| CommandError::invalid("Search for a plan first"))?;
                if !plan.complete || plan.steps.len() != before.pieces.len() {
                    return Err(justblast_core::services::WorkspaceError::IncompletePlan.into());
                }
                let mut history = data.history.clone();
                if before.completed > 0 && !data.device.cycle_done {
                    history.push(HistoryEntry {
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs()
                            .to_string(),
                        source: "phone".to_owned(),
                        cycle: before.cycle,
                        status: "paused".to_owned(),
                        message: format!(
                            "{} acknowledged placements retained before simulation branch",
                            before.completed
                        ),
                        board: data.workspace.current_board().rows(),
                    });
                }
                history.push(HistoryEntry {
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                        .to_string(),
                    source: "simulated".to_owned(),
                    cycle: before.cycle,
                    status: "simulated".to_owned(),
                    message: format!("{} placements", plan.steps.len()),
                    board: plan.candidate.board.rows(),
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
                data.workspace.next_cycle(expected_revision)?;
                data.device.observation = None;
                data.device.capture = None;
                data.device.corrections = None;
                data.device.needs_scan = true;
                data.device.completed = 0;
                data.device.cycle_done = false;
                data.device.visuals.clear();
                data.device.set(
                    "connected",
                    "Simulated board. Scan before moving your phone.",
                );
                data.history = history;
                data.invalidate();
            }
            WorkspaceAction::Weights {
                expected_revision,
                weights,
            } => {
                data.workspace.check_revision(expected_revision)?;
                let weights: solver::Weights = weights.into();
                if !weights.valid() {
                    return Err(CommandError::invalid("Invalid scoring weights"));
                }
                let mut prefs = data.preferences.clone();
                prefs.weights = weights.into();
                prefs.advance().map_err(CommandError::invalid)?;
                data.store
                    .save_preferences(&prefs)
                    .map_err(|message| CommandError {
                        code: ErrorCode::Storage,
                        message,
                    })?;
                data.workspace.set_weights(expected_revision, weights)?;
                data.preferences = prefs;
                data.invalidate();
            }
            WorkspaceAction::Select {
                expected_revision,
                candidate_id,
                completed,
            } => {
                data.workspace.check_revision(expected_revision)?;
                if completed < data.workspace.snapshot().completed {
                    return Err(CommandError::invalid(
                        "Completed phone moves are authoritative",
                    ));
                }
                let e = data
                    .explorer
                    .as_ref()
                    .filter(|e| e.revision == expected_revision)
                    .ok_or_else(|| CommandError::invalid("Alternatives expired. Explore again."))?;
                let path = e
                    .index
                    .path(candidate_id as usize)
                    .ok_or_else(|| CommandError::invalid("Unknown candidate"))?;
                let maxima = e.index.maxima;
                let complete = e.index.complete;
                data.workspace.select_path(
                    expected_revision,
                    &path,
                    usize::from(completed),
                    maxima,
                    complete,
                    &evaluation_shapes(),
                )?;
                let snapshot = data.workspace.snapshot();
                let revision = snapshot.revision;
                if let Some(visual) = data.device.visuals.back_mut()
                    && visual.snapshot.cycle == snapshot.cycle
                    && !visual.done
                {
                    visual.snapshot = snapshot;
                }
                if let Some(e) = &mut data.explorer {
                    e.revision = revision;
                }
            }
            WorkspaceAction::Restore {
                expected_revision,
                completed,
            } => {
                if completed < data.workspace.snapshot().completed {
                    return Err(CommandError::invalid(
                        "Completed phone moves are authoritative",
                    ));
                }
                data.workspace
                    .restore_plan(expected_revision, usize::from(completed))?;
                let snapshot = data.workspace.snapshot();
                let revision = snapshot.revision;
                if let Some(visual) = data.device.visuals.back_mut()
                    && visual.snapshot.cycle == snapshot.cycle
                    && !visual.done
                {
                    visual.snapshot = snapshot;
                }
                if let Some(e) = &mut data.explorer {
                    e.revision = revision;
                }
            }
            WorkspaceAction::Search { .. } | WorkspaceAction::Explore { .. } => unreachable!(),
        }
        Ok(data.view())
    }
    fn start_job(
        &self,
        expected: u32,
        completed: u8,
        budget: Option<u32>,
        explore: bool,
    ) -> Result<WorkspaceView, CommandError> {
        self.start_computation(expected, completed, budget, explore, false)
    }
    fn start_computation(
        &self,
        expected: u32,
        completed: u8,
        budget: Option<u32>,
        explore: bool,
        during_scan: bool,
    ) -> Result<WorkspaceView, CommandError> {
        let max_nodes = budget.unwrap_or(2_000_000);
        if max_nodes == 0 || max_nodes > 10_000_000 {
            return Err(CommandError::invalid(
                "Search budget must be within 1..10000000",
            ));
        }
        let mut data = self.data()?;
        if data.device.busy && !during_scan {
            return Err(CommandError {
                code: ErrorCode::Busy,
                message: "Stop the active device operation first".to_owned(),
            });
        }
        data.workspace.check_revision(expected)?;
        let (revision, game, prefix) = if explore {
            if completed < data.workspace.snapshot().completed {
                return Err(CommandError::invalid(
                    "Completed phone moves are authoritative",
                ));
            }
            let snapshot = data.workspace.snapshot();
            let plan = snapshot
                .plan
                .ok_or_else(|| CommandError::invalid("Search for a plan first"))?;
            let prefix = plan
                .candidate
                .path
                .get(..usize::from(completed))
                .ok_or_else(|| CommandError::invalid("Invalid completed prefix"))?
                .to_vec();
            (expected, data.workspace.game(), prefix)
        } else {
            let (revision, game) = data.workspace.begin_search(expected)?;
            (revision, game, Vec::new())
        };
        data.invalidate();
        data.next_job = data
            .next_job
            .checked_add(1)
            .ok_or_else(|| CommandError::invalid("Restart JustBlast"))?;
        let id = data.next_job;
        let cancelled = Arc::new(AtomicBool::new(false));
        let progress = Arc::new(AtomicU64::new(0));
        data.job = Some(Job {
            id,
            kind: if explore { "explore" } else { "solve" },
            revision,
            cancelled: cancelled.clone(),
            progress: progress.clone(),
            status: JobStatus::Running,
            message: None,
        });
        let state = self.clone();
        let spawned = std::thread::Builder::new()
            .name(format!("justblast-compute-{id}"))
            .spawn(move || {
                let started = Instant::now();
                let mut timed_out = false;
                let checkpoint = |visited| {
                    progress.store(visited, Ordering::Relaxed);
                    if started.elapsed() > Duration::from_secs(30) {
                        timed_out = true;
                        return false;
                    }
                    !cancelled.load(Ordering::Acquire)
                };
                enum Output {
                    Plan(solver::SearchResult),
                    Index(ExplorerIndex),
                }
                let result = if explore {
                    solver::enumerate(
                        &game,
                        &evaluation_shapes(),
                        &prefix,
                        SearchOptions {
                            max_nodes: u64::from(max_nodes),
                        },
                        checkpoint,
                    )
                    .map(Output::Index)
                } else {
                    solver::solve(
                        &game,
                        &evaluation_shapes(),
                        SearchOptions {
                            max_nodes: u64::from(max_nodes),
                        },
                        checkpoint,
                    )
                    .map(Output::Plan)
                };
                let Ok(mut data) = state.data() else {
                    return;
                };
                if data
                    .job
                    .as_ref()
                    .is_none_or(|j| j.id != id || j.revision != revision)
                    || data.workspace.snapshot().revision != revision
                {
                    return;
                }
                match result {
                    Ok(output) if !cancelled.load(Ordering::Acquire) => {
                        match output {
                            Output::Plan(result) => {
                                if data.workspace.accept_search(revision, result).is_err() {
                                    return;
                                }
                                data.refresh_device_readiness();
                            }
                            Output::Index(index) => {
                                data.explorer = Some(Indexed {
                                    revision,
                                    game,
                                    index: Arc::new(index),
                                });
                            }
                        }
                        if let Some(j) = &mut data.job {
                            j.status = JobStatus::Completed;
                        }
                    }
                    result => {
                        if !explore {
                            let _ = data.workspace.cancel_search(revision);
                        }
                        if let Some(j) = &mut data.job {
                            j.status = if cancelled.load(Ordering::Acquire) {
                                JobStatus::Cancelled
                            } else {
                                JobStatus::Failed
                            };
                            j.message = Some(if timed_out {
                                "Search exceeded 30 seconds. Reduce the input or node budget."
                                    .to_owned()
                            } else {
                                result.err().unwrap_or("Search cancelled").to_owned()
                            });
                        }
                    }
                }
            });
        if let Err(error) = spawned {
            data.invalidate();
            if !explore {
                let _ = data.workspace.cancel_search(revision);
            }
            return Err(CommandError {
                code: ErrorCode::Internal,
                message: error.to_string(),
            });
        }
        Ok(data.view())
    }
    pub fn explorer_page(&self, request: ExplorerRequest) -> Result<ExplorerView, CommandError> {
        let (game, index) = {
            let data = self.data()?;
            data.workspace.check_revision(request.expected_revision)?;
            let e = data
                .explorer
                .as_ref()
                .filter(|e| e.revision == request.expected_revision)
                .ok_or_else(|| CommandError::invalid("Alternatives expired. Explore again."))?;
            (e.game.clone(), e.index.clone())
        };
        let page = index
            .page(
                &game,
                &solver::ExplorerFilter {
                    order: request.order,
                    positions: request.positions,
                    page: request.page as usize,
                    page_size: usize::from(request.page_size),
                },
            )
            .map_err(CommandError::invalid)?;
        self.data()?
            .workspace
            .check_revision(request.expected_revision)?;
        Ok(ExplorerView {
            complete: page.complete,
            total: page.total as u32,
            all_total: page.all_total as u32,
            page: page.page as u32,
            pages: page.pages as u32,
            candidates: page
                .entries
                .into_iter()
                .map(|entry| CandidateView {
                    id: entry.id as u32,
                    moves: entry
                        .path
                        .into_iter()
                        .map(|m| MoveView {
                            piece: game.pieces[usize::from(m.piece)].slot,
                            row: m.row,
                            column: m.column,
                        })
                        .collect(),
                    score: entry.score,
                    clears: entry.clears,
                })
                .collect(),
            next_counts: page.next_counts.map(|n| n as u32),
            position_options: page.position_options,
        })
    }
    pub fn preview_candidate(&self, request: CandidateRequest) -> Result<PlanView, CommandError> {
        let data = self.data()?;
        data.workspace.check_revision(request.expected_revision)?;
        let e = data
            .explorer
            .as_ref()
            .filter(|e| e.revision == request.expected_revision)
            .ok_or_else(|| CommandError::invalid("Alternatives expired. Explore again."))?;
        let path = e
            .index
            .path(request.candidate_id as usize)
            .ok_or_else(|| CommandError::invalid("Unknown candidate"))?;
        let (candidate, steps) = solver::replay(&e.game, &path).map_err(CommandError::invalid)?;
        let geometry = solver::evaluation::Evaluator::new(&evaluation_shapes())
            .map_err(CommandError::invalid)?
            .evaluate(candidate.board.bits());
        let metrics = solver::assess(&candidate, e.index.maxima, geometry, e.game.weights);
        Ok(PlanView::from_result(
            &solver::SearchResult {
                candidate,
                steps,
                metrics,
                maxima: e.index.maxima,
                complete: e.index.complete,
                stats: Default::default(),
            },
            e.game.pieces.len(),
        ))
    }
}
