pub use crate::preferences::{
    AutomationSettings, DeviceSettings, HistoryEntry, Preferences, SavedShape, ScoreWeights,
};
use justblast_core::{
    domain::{Board, Shape, combo::ComboState},
    services::{Phase, WorkspaceError, WorkspaceSnapshot},
    solver::{self, Piece, SearchResult, Weights},
};
use serde::{Deserialize, Serialize};

macro_rules! strict_action {
    ($name:ident,$wire:ident,{$($variant:ident $({$($field:ident:$ty:ty),* $(,)?})?),* $(,)?})=>{
        #[derive(Debug,Deserialize,Serialize)]
        #[serde(tag="type",rename_all="camelCase",rename_all_fields="camelCase",deny_unknown_fields)]
        pub enum $name {$($variant $({$($field:$ty),*})?),*}
        #[cfg(feature="export-contracts")]
        #[derive(ts_rs::TS)]
        #[ts(rename=stringify!($name),tag="type",rename_all="camelCase",rename_all_fields="camelCase")]
        pub enum $wire {$($variant $({$($field:$ty),*})?),*}
        #[cfg(feature="export-contracts")]
        impl ts_rs::TS for $name {
            type WithoutGenerics=Self;
            type OptionInnerType=Self;
            const IS_ENUM:bool=true;
            fn name(cfg:&ts_rs::Config)->String{<$wire as ts_rs::TS>::name(cfg)}
            fn inline(cfg:&ts_rs::Config)->String{<$wire as ts_rs::TS>::inline(cfg)}
            fn decl(cfg:&ts_rs::Config)->String{<$wire as ts_rs::TS>::decl(cfg)}
            fn decl_concrete(cfg:&ts_rs::Config)->String{<$wire as ts_rs::TS>::decl_concrete(cfg)}
            fn visit_dependencies(visitor:&mut impl ts_rs::TypeVisitor){<$wire as ts_rs::TS>::visit_dependencies(visitor)}
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "export-contracts", derive(ts_rs::TS))]
pub struct PieceView {
    pub slot: u8,
    pub catalog_id: Option<u32>,
    pub name: String,
    pub cells: Vec<(u8, u8)>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "export-contracts", derive(ts_rs::TS))]
pub struct WeightsView {
    pub points: f64,
    pub combo: f64,
    pub flexibility: f64,
    pub fragmentation: f64,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "export-contracts", derive(ts_rs::TS))]
pub struct ComboView {
    pub count: u32,
    pub misses: u32,
    pub producer_remaining: u8,
    pub saved_count: u32,
    pub checking: bool,
    pub waiting: u32,
    pub recovered: bool,
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "export-contracts", derive(ts_rs::TS))]
pub struct MoveView {
    pub piece: u8,
    pub row: u8,
    pub column: u8,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "export-contracts", derive(ts_rs::TS))]
pub struct StepView {
    pub movement: MoveView,
    pub slot: u8,
    pub before: [String; 8],
    pub placed: [String; 8],
    pub after: [String; 8],
    pub cleared_rows: Vec<u8>,
    pub cleared_columns: Vec<u8>,
    pub possible_moves: u32,
    pub states_before: [ComboView; 3],
    pub states_after: [ComboView; 3],
    pub model_points: [f64; 3],
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "export-contracts", derive(ts_rs::TS))]
pub struct MetricsView {
    pub value: f64,
    pub potential_points: f64,
    pub combo_alive: bool,
    pub flexibility: f64,
    pub fragmentation: f64,
    pub shape_coverage: u32,
    pub total_positions: u32,
    pub normalized: [Option<f64>; 3],
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "export-contracts", derive(ts_rs::TS))]
pub struct PlanView {
    pub complete: bool,
    pub requested_pieces: u8,
    pub steps: Vec<StepView>,
    pub final_board: [String; 8],
    pub metrics: MetricsView,
    pub model_points: [f64; 3],
    pub normalization_maxima: [f64; 3],
    pub final_states: [ComboView; 3],
    pub visited: u32,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "export-contracts", derive(ts_rs::TS))]
pub struct Capabilities {
    pub board_input: bool,
    pub piece_input: bool,
    pub solver: bool,
    pub device: bool,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "export-contracts", derive(ts_rs::TS))]
pub enum JobStatus {
    Running,
    Completed,
    Cancelled,
    Failed,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "export-contracts", derive(ts_rs::TS))]
pub struct JobView {
    pub id: u32,
    pub kind: String,
    pub status: JobStatus,
    pub visited: u32,
    pub message: Option<String>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "export-contracts", derive(ts_rs::TS))]
pub struct WorkspaceView {
    pub schema_version: u8,
    pub revision: u32,
    pub completed: u8,
    pub checkpoint: u32,
    pub input_known: bool,
    pub consumed_slots: Vec<u8>,
    pub source: String,
    pub current_board: [String; 8],
    pub board: [String; 8],
    pub occupied: u32,
    pub pieces: Vec<PieceView>,
    pub cycle: u32,
    pub phase: String,
    pub plan: Option<PlanView>,
    pub weights: WeightsView,
    pub job: Option<JobView>,
    pub explorer_ready: bool,
    pub capabilities: Capabilities,
    pub device: DeviceView,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "export-contracts", derive(ts_rs::TS))]
pub struct ReplaceBoardRequest {
    pub expected_revision: u32,
    pub rows: [String; 8],
}
strict_action!(WorkspaceAction,WorkspaceActionWire,{
    Pieces {
        expected_revision: u32,
        pieces: Vec<PieceView>,
    },
    Search {
        expected_revision: u32,
        max_nodes: Option<u32>,
    },
    Cancel {
        expected_revision: u32,
    },
    NextCycle {
        expected_revision: u32,
    },
    ApplyInput {
        expected_revision: u32,
        rows: Box<[String; 8]>,
        pieces: Vec<PieceView>,
        anchors: Vec<SlotAnchor>,
    },
    UpdateInput {
        expected_revision: u32,
        rows: Box<[String; 8]>,
        pieces: Vec<PieceView>,
        anchors: Vec<SlotAnchor>,
        accepted_slots: Vec<u8>,
        accept_board: bool,
    },
    Explore {
        expected_revision: u32,
        completed: u8,
        max_nodes: Option<u32>,
    },
    Select {
        expected_revision: u32,
        candidate_id: u32,
        completed: u8,
    },
    Restore {
        expected_revision: u32,
        completed: u8,
    },
    Weights {
        expected_revision: u32,
        weights: WeightsView,
    },
});
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "export-contracts", derive(ts_rs::TS))]
pub struct ExplorerRequest {
    pub expected_revision: u32,
    pub order: Vec<u8>,
    pub positions: Vec<Option<(u8, u8)>>,
    pub page: u32,
    pub page_size: u8,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "export-contracts", derive(ts_rs::TS))]
pub struct CandidateView {
    pub id: u32,
    pub moves: Vec<MoveView>,
    pub score: Option<f64>,
    pub clears: u32,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "export-contracts", derive(ts_rs::TS))]
pub struct ExplorerView {
    pub complete: bool,
    pub total: u32,
    pub all_total: u32,
    pub page: u32,
    pub pages: u32,
    pub candidates: Vec<CandidateView>,
    pub next_counts: [u32; 3],
    pub position_options: Vec<Vec<(u8, u8)>>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "export-contracts", derive(ts_rs::TS))]
pub struct CandidateRequest {
    pub expected_revision: u32,
    pub candidate_id: u32,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[cfg_attr(feature = "export-contracts", derive(ts_rs::TS))]
pub enum ErrorCode {
    InvalidInput,
    StaleRevision,
    ReadOnlyBoard,
    Busy,
    NoPlan,
    IncompletePlan,
    PrefixChanged,
    Unavailable,
    Device,
    Storage,
    Internal,
}
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "export-contracts", derive(ts_rs::TS))]
pub struct CommandError {
    pub code: ErrorCode,
    pub message: String,
}
impl CommandError {
    pub fn invalid(message: impl Into<String>) -> Self {
        Self {
            code: ErrorCode::InvalidInput,
            message: message.into(),
        }
    }
}
impl TryFrom<PieceView> for Piece {
    type Error = CommandError;
    fn try_from(p: PieceView) -> Result<Self, Self::Error> {
        if p.slot > 2
            || p.name.trim().is_empty()
            || p.name.chars().count() > 40
            || p.catalog_id == Some(0)
        {
            return Err(CommandError::invalid("Invalid piece"));
        }
        let shape = Shape::new(p.cells).map_err(|e| CommandError::invalid(e.to_string()))?;
        if p.catalog_id.is_some_and(|id| {
            !justblast_core::domain::catalog::catalog()
                .iter()
                .any(|(known, s)| *known == id && *s == shape)
        }) {
            return Err(CommandError::invalid(
                "Catalogue identity does not match the corrected geometry",
            ));
        }
        Ok(Self {
            slot: p.slot,
            catalog_id: p.catalog_id,
            name: p.name.trim().to_owned(),
            shape,
        })
    }
}
impl From<Piece> for PieceView {
    fn from(p: Piece) -> Self {
        Self {
            slot: p.slot,
            catalog_id: p.catalog_id,
            name: p.name,
            cells: p.shape.cells().to_vec(),
        }
    }
}
impl From<Weights> for WeightsView {
    fn from(w: Weights) -> Self {
        Self {
            points: w.points,
            combo: w.combo,
            flexibility: w.flexibility,
            fragmentation: w.fragmentation,
        }
    }
}
impl From<WeightsView> for Weights {
    fn from(w: WeightsView) -> Self {
        Self {
            points: w.points,
            combo: w.combo,
            flexibility: w.flexibility,
            fragmentation: w.fragmentation,
        }
    }
}
impl From<ComboState> for ComboView {
    fn from(s: ComboState) -> Self {
        Self {
            count: s.count,
            misses: s.misses,
            producer_remaining: s.producer_remaining,
            saved_count: s.saved_count,
            checking: s.checking,
            waiting: s.waiting,
            recovered: s.recovered,
        }
    }
}
impl From<solver::Move> for MoveView {
    fn from(m: solver::Move) -> Self {
        Self {
            piece: m.piece,
            row: m.row,
            column: m.column,
        }
    }
}
impl PlanView {
    pub fn from_result(result: &SearchResult, requested: usize) -> Self {
        let m = &result.metrics;
        Self {
            complete: result.complete,
            requested_pieces: requested as u8,
            steps: result
                .steps
                .iter()
                .map(|s| StepView {
                    movement: s.movement.into(),
                    slot: s.slot,
                    before: s.before.rows(),
                    placed: s.result.placed.rows(),
                    after: s.result.board.rows(),
                    cleared_rows: s.result.cleared_rows.clone(),
                    cleared_columns: s.result.cleared_columns.clone(),
                    possible_moves: s.possible_moves as u32,
                    states_before: s.states_before.map(Into::into),
                    states_after: s.states_after.map(Into::into),
                    model_points: s.points.map(|n| n as f64),
                })
                .collect(),
            final_board: result.candidate.board.rows(),
            metrics: MetricsView {
                value: m.value,
                potential_points: m.potential_points,
                combo_alive: m.combo_alive,
                flexibility: m.geometry.flexibility,
                fragmentation: m.geometry.fragmentation,
                shape_coverage: m.geometry.coverage,
                total_positions: m.geometry.total_positions,
                normalized: m.normalized,
            },
            model_points: result.candidate.gains.map(|n| n as f64),
            normalization_maxima: result.maxima.map(|n| n as f64),
            final_states: result.candidate.states.map(Into::into),
            visited: result.stats.visited.min(u64::from(u32::MAX)) as u32,
        }
    }
}
impl From<WorkspaceSnapshot> for WorkspaceView {
    fn from(s: WorkspaceSnapshot) -> Self {
        Self {
            schema_version: 2,
            revision: s.revision,
            completed: s.completed,
            checkpoint: s.checkpoint,
            input_known: s.input_known,
            consumed_slots: Vec::new(),
            source: s.source,
            current_board: s.board.rows(),
            board: s.board.rows(),
            occupied: s.board.occupied(),
            plan: s
                .plan
                .as_ref()
                .map(|p| PlanView::from_result(p, s.pieces.len())),
            pieces: s.pieces.into_iter().map(Into::into).collect(),
            cycle: s.cycle,
            phase: match s.phase {
                Phase::AwaitingInput => "awaiting_input",
                Phase::Searching => "searching",
                Phase::Ready => "ready",
                Phase::NoSolution => "no_solution",
            }
            .to_owned(),
            weights: s.weights.into(),
            job: None,
            explorer_ready: false,
            device: DeviceView::default(),
            capabilities: Capabilities {
                board_input: true,
                piece_input: true,
                solver: true,
                device: true,
            },
        }
    }
}
impl From<WorkspaceError> for CommandError {
    fn from(value: WorkspaceError) -> Self {
        let (code, message) = match value {
            WorkspaceError::StaleRevision => (
                ErrorCode::StaleRevision,
                "The workspace changed. Refresh before trying again.",
            ),
            WorkspaceError::InvalidInput => (
                ErrorCode::InvalidInput,
                "Provide valid pieces and scoring settings.",
            ),
            WorkspaceError::Busy => (ErrorCode::Busy, "A search is already running."),
            WorkspaceError::NoPlan => (ErrorCode::NoPlan, "Search for a plan first."),
            WorkspaceError::IncompletePlan => (
                ErrorCode::IncompletePlan,
                "A complete plan for all pieces is required.",
            ),
            WorkspaceError::PrefixChanged => (
                ErrorCode::PrefixChanged,
                "Completed moves must stay unchanged.",
            ),
        };
        Self {
            code,
            message: message.to_owned(),
        }
    }
}
pub fn parse_board(rows: &[String]) -> Result<Board, CommandError> {
    Board::from_rows(rows).map_err(|e| CommandError::invalid(e.to_string()))
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "export-contracts", derive(ts_rs::TS))]
pub struct DeviceInfoView {
    pub serial: String,
    pub state: String,
    pub model: String,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "export-contracts", derive(ts_rs::TS))]
pub struct VisualCycleView {
    pub id: u32,
    pub cycle: u32,
    pub revision: u32,
    pub board: [String; 8],
    pub pieces: Vec<PieceView>,
    pub plan: PlanView,
    pub completed: u8,
    pub done: bool,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "export-contracts", derive(ts_rs::TS))]
pub struct DeviceView {
    pub revision: u32,
    pub state: String,
    pub profile_id: String,
    pub profile_ids: Vec<String>,
    pub message: String,
    pub error: Option<String>,
    pub busy: bool,
    pub serial: Option<String>,
    pub devices: Vec<DeviceInfoView>,
    pub completed: u8,
    pub needs_scan: bool,
    pub cycle_done: bool,
    pub provenance: String,
    pub visual: Option<VisualCycleView>,
    pub queued_visuals: u32,
    pub dropped_visuals: u32,
    pub capture: Option<CaptureView>,
    pub readiness: String,
}
impl Default for DeviceView {
    fn default() -> Self {
        Self {
            revision: 1,
            state: "disconnected".to_owned(),
            profile_id: "default".to_owned(),
            profile_ids: vec!["default".to_owned()],
            message: "Select a phone in Settings.".to_owned(),
            error: None,
            busy: false,
            serial: None,
            devices: Vec::new(),
            completed: 0,
            needs_scan: true,
            cycle_done: false,
            provenance: "none".to_owned(),
            visual: None,
            queued_visuals: 0,
            dropped_visuals: 0,
            capture: None,
            readiness: "Scan to link this input to your phone.".to_owned(),
        }
    }
}
strict_action!(DeviceAction,DeviceActionWire,{
    Discover {},
    Wireless {
        address: String,
        code: Option<String>,
    },
    Connect {
        serial: String,
        install_helper: bool,
    },
    Disconnect {},
    Scan {
        workspace_revision: u32,
    },
    CaptureForEdit {workspace_revision: u32},
    NextCycle { workspace_revision: u32 },
    Start {
        workspace_revision: u32,
        through: Option<u8>,
    },
    Stop {},
    ForceStop {},
    AcceptWarning {},
    Profile {
        id: String,
        toml: String,
    },
    SelectProfile {
        id: String,
    },
    AcknowledgeVisual {
        cycle_id: u32,
    },
});
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "export-contracts", derive(ts_rs::TS))]
pub struct DeviceRequest {
    pub command_id: String,
    pub expected_revision: u32,
    pub action: DeviceAction,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "export-contracts", derive(ts_rs::TS))]
pub struct SlotAnchor {
    pub slot: u8,
    pub x: u32,
    pub y: u32,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "export-contracts", derive(ts_rs::TS))]
pub struct CaptureView {
    pub id: u32,
    pub board: Option<[String; 8]>,
    pub pieces: Vec<PieceView>,
    pub unknown_slots: Vec<u8>,
    pub slot_issues: [Option<String>; 3],
    pub slot_crops: Vec<String>,
    pub slot_sizes: Vec<(u32, u32)>,
    pub pitch: f64,
    pub anchors: Vec<SlotAnchor>,
}

pub use crate::device::profile::{
    Profile as HardwareProfile, Screen as CalibrationScreen, Timing as CalibrationTiming,
    Touch as CalibrationTouch,
};
strict_action!(CalibrationAction, CalibrationActionWire, {
    Read {},
    Capture { serial: String },
    Preview { capture_id: u32, profile: HardwareProfile },
    AutoLayout { capture_id: u32, profile: HardwareProfile },
    Export { profile: HardwareProfile },
    TestDrag { capture_id: u32, profile: HardwareProfile, slot: u8, row: u8, column: u8, install_helper: bool },
});
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "export-contracts", derive(ts_rs::TS))]
pub struct CalibrationRequest {
    pub command_id: String,
    pub expected_revision: u32,
    pub action: CalibrationAction,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "export-contracts", derive(ts_rs::TS))]
pub struct CalibrationView {
    pub layout_detected: Option<bool>,
    pub unclear_cells: Vec<u8>,
    pub anchors: Vec<SlotAnchor>,
    pub movement: Option<String>,
    pub revision: u32,
    pub profile: HardwareProfile,
    pub toml: String,
    pub capture_id: Option<u32>,
    pub image: Option<String>,
    pub board: Option<[String; 8]>,
    pub board_unclear: bool,
    pub pieces: Vec<PieceView>,
    pub unknown_slots: Vec<u8>,
    pub message: String,
}
