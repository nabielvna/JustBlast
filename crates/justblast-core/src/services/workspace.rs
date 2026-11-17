use crate::domain::combo::{ComboState, RuleContext};
use crate::domain::{Board, Shape};
use crate::solver::{
    Game, Move, Piece, SearchResult, Weights, assess, evaluation::Evaluator, replay,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkspaceError {
    StaleRevision,
    InvalidInput,
    Busy,
    NoPlan,
    IncompletePlan,
    PrefixChanged,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Phase {
    AwaitingInput,
    Searching,
    Ready,
    NoSolution,
}
#[derive(Clone, Debug, PartialEq)]
pub struct WorkspaceSnapshot {
    pub revision: u32,
    pub completed: u8,
    pub checkpoint: u32,
    pub input_known: bool,
    pub source: String,
    pub board: Board,
    pub pieces: Vec<Piece>,
    pub cycle: u32,
    pub phase: Phase,
    pub plan: Option<SearchResult>,
    pub weights: Weights,
}
#[derive(Clone)]
struct Session {
    checkpoint: Option<crate::solver::Checkpoint>,
    checkpoint_id: u32,
    input_known: bool,
    source: String,
    board: Board,
    pieces: Vec<Piece>,
    cycle: u32,
    phase: Phase,
    states: [ComboState; 3],
    context: RuleContext,
    plan: Option<SearchResult>,
    recommendation: Option<SearchResult>,
}
impl Default for Session {
    fn default() -> Self {
        Self {
            checkpoint: None,
            checkpoint_id: 1,
            input_known: false,
            source: "entered".to_owned(),
            board: Board::default(),
            pieces: Vec::new(),
            cycle: 1,
            phase: Phase::AwaitingInput,
            states: [ComboState::default(); 3],
            context: RuleContext::default(),
            plan: None,
            recommendation: None,
        }
    }
}
pub struct Workspace {
    revision: u32,
    session: Session,
    weights: Weights,
}
impl Default for Workspace {
    fn default() -> Self {
        Self {
            revision: 1,
            session: Session::default(),
            weights: Weights::default(),
        }
    }
}
impl Workspace {
    fn session(&self) -> &Session {
        &self.session
    }
    fn session_mut(&mut self) -> &mut Session {
        &mut self.session
    }
    pub fn snapshot(&self) -> WorkspaceSnapshot {
        let s = self.session();
        WorkspaceSnapshot {
            revision: self.revision,
            completed: s.checkpoint.as_ref().map_or(0, |c| c.steps.len() as u8),
            checkpoint: s.checkpoint_id,
            input_known: s.input_known,
            source: s.source.clone(),
            board: s.board,
            pieces: s.pieces.clone(),
            cycle: s.cycle,
            phase: s.phase,
            plan: s.plan.clone(),
            weights: self.weights,
        }
    }
    pub fn check_revision(&self, expected: u32) -> Result<(), WorkspaceError> {
        if self.revision == expected {
            Ok(())
        } else {
            Err(WorkspaceError::StaleRevision)
        }
    }
    fn changed_input(&mut self) {
        self.revision += 1;
        let s = self.session_mut();
        s.phase = Phase::AwaitingInput;
        s.plan = None;
        s.recommendation = None;
    }
    pub fn apply_input(
        &mut self,
        expected: u32,
        board: Board,
        mut pieces: Vec<Piece>,
        source: &str,
    ) -> Result<WorkspaceSnapshot, WorkspaceError> {
        self.check_revision(expected)?;
        pieces.sort_by_key(|p| p.slot);
        if pieces.len() > 3
            || pieces
                .iter()
                .any(|p| p.slot > 2 || p.name.trim().is_empty() || p.name.chars().count() > 40)
            || pieces.windows(2).any(|w| w[0].slot == w[1].slot)
        {
            return Err(WorkspaceError::InvalidInput);
        }
        if let Some(checkpoint) = &self.session.checkpoint {
            for step in &checkpoint.steps {
                let previous = self.session.pieces.iter().find(|p| p.slot == step.slot);
                if pieces.iter().find(|p| p.slot == step.slot) != previous {
                    return Err(WorkspaceError::PrefixChanged);
                }
            }
        }
        if let Some(checkpoint) = &mut self.session.checkpoint {
            checkpoint.candidate.board = board;
            checkpoint.candidate.path = checkpoint
                .steps
                .iter()
                .map(|step| Move {
                    piece: pieces
                        .iter()
                        .position(|p| p.slot == step.slot)
                        .expect("Consumed slots validated above") as u8,
                    row: step.movement.row,
                    column: step.movement.column,
                })
                .collect();
        }
        self.session.board = board;
        self.session.pieces = pieces;
        self.session.input_known = true;
        self.session.source = source.to_owned();
        self.session.checkpoint_id += 1;
        self.changed_input();
        Ok(self.snapshot())
    }
    pub fn replace_board(
        &mut self,
        expected: u32,
        board: Board,
    ) -> Result<WorkspaceSnapshot, WorkspaceError> {
        self.apply_input(expected, board, self.session.pieces.clone(), "entered")
    }
    pub fn replace_pieces(
        &mut self,
        expected: u32,
        pieces: Vec<Piece>,
    ) -> Result<WorkspaceSnapshot, WorkspaceError> {
        self.apply_input(expected, self.current_board(), pieces, "entered")
    }
    pub fn acknowledge(&mut self, completed: u8) -> Result<(), WorkspaceError> {
        let previous = self.snapshot().completed;
        if completed == previous {
            return Ok(());
        }
        if completed < previous {
            return Err(WorkspaceError::PrefixChanged);
        }
        let plan = self.session.plan.as_ref().ok_or(WorkspaceError::NoPlan)?;
        let count = usize::from(completed);
        if count == 0 || count > plan.steps.len() {
            return Err(WorkspaceError::InvalidInput);
        }
        let steps = plan.steps[..count].to_vec();
        let last = &steps[count - 1];
        self.session.checkpoint = Some(crate::solver::Checkpoint {
            candidate: crate::solver::Candidate {
                board: last.result.board,
                states: last.states_after,
                gains: std::array::from_fn(|i| steps.iter().map(|s| s.points[i]).sum()),
                path: plan.candidate.path[..count].to_vec(),
            },
            steps,
        });
        Ok(())
    }
    pub fn require_input_review(&mut self) {
        self.session.input_known = false;
        self.changed_input();
    }
    pub fn current_board(&self) -> Board {
        self.session
            .checkpoint
            .as_ref()
            .map_or(self.session.board, |c| c.candidate.board)
    }
    pub fn set_weights(
        &mut self,
        expected: u32,
        weights: Weights,
    ) -> Result<WorkspaceSnapshot, WorkspaceError> {
        self.check_revision(expected)?;
        if !weights.valid() {
            return Err(WorkspaceError::InvalidInput);
        }
        if weights != self.weights {
            self.weights = weights;
            self.changed_input();
        }
        Ok(self.snapshot())
    }
    pub fn game(&self) -> Game {
        let s = self.session();
        Game {
            checkpoint: s.checkpoint.clone(),
            board: s.board,
            pieces: s.pieces.clone(),
            states: s.states,
            context: s.context,
            weights: self.weights,
        }
    }
    pub fn begin_search(&mut self, expected: u32) -> Result<(u32, Game), WorkspaceError> {
        self.check_revision(expected)?;
        if self.session().phase == Phase::Searching {
            return Err(WorkspaceError::Busy);
        }
        if !self.session.input_known {
            return Err(WorkspaceError::InvalidInput);
        }
        let game = self.game();
        game.validate().map_err(|_| WorkspaceError::InvalidInput)?;
        self.session_mut().phase = Phase::Searching;
        self.session_mut().plan = None;
        self.session_mut().recommendation = None;
        self.revision += 1;
        Ok((self.revision, game))
    }
    pub fn accept_search(
        &mut self,
        expected: u32,
        result: SearchResult,
    ) -> Result<(), WorkspaceError> {
        self.check_revision(expected)?;
        if self.session().phase != Phase::Searching {
            return Err(WorkspaceError::StaleRevision);
        }
        let s = self.session_mut();
        s.phase = if result.steps.len() < s.pieces.len()
            && result.steps.len() == s.checkpoint.as_ref().map_or(0, |c| c.steps.len())
        {
            if result.complete {
                Phase::NoSolution
            } else {
                Phase::AwaitingInput
            }
        } else {
            Phase::Ready
        };
        s.recommendation = Some(result.clone());
        s.plan = Some(result);
        self.revision += 1;
        Ok(())
    }
    pub fn cancel_search(&mut self, expected: u32) -> Result<WorkspaceSnapshot, WorkspaceError> {
        self.check_revision(expected)?;
        if self.session().phase == Phase::Searching {
            self.session_mut().phase = Phase::AwaitingInput;
            self.revision += 1;
        }
        Ok(self.snapshot())
    }
    pub fn select_path(
        &mut self,
        expected: u32,
        path: &[Move],
        prefix: usize,
        maxima: [u64; 3],
        complete: bool,
        shapes: &[Shape],
    ) -> Result<WorkspaceSnapshot, WorkspaceError> {
        self.check_revision(expected)?;
        let game = self.game();
        let active = self.session().plan.as_ref().ok_or(WorkspaceError::NoPlan)?;
        if path.len() != game.pieces.len()
            || prefix > active.candidate.path.len()
            || path.get(..prefix) != active.candidate.path.get(..prefix)
        {
            return Err(WorkspaceError::PrefixChanged);
        }
        let (candidate, steps) = replay(&game, path).map_err(|_| WorkspaceError::InvalidInput)?;
        let geometry = Evaluator::new(shapes)
            .map_err(|_| WorkspaceError::InvalidInput)?
            .evaluate(candidate.board.bits());
        let metrics = assess(&candidate, maxima, geometry, self.weights);
        self.session_mut().plan = Some(SearchResult {
            candidate,
            steps,
            metrics,
            maxima,
            complete,
            stats: Default::default(),
        });
        self.session_mut().phase = Phase::Ready;
        self.revision += 1;
        Ok(self.snapshot())
    }
    pub fn restore_plan(
        &mut self,
        expected: u32,
        prefix: usize,
    ) -> Result<WorkspaceSnapshot, WorkspaceError> {
        self.check_revision(expected)?;
        let s = self.session();
        let plan = s.recommendation.clone().ok_or(WorkspaceError::NoPlan)?;
        if prefix > 0 {
            let active = s.plan.as_ref().ok_or(WorkspaceError::NoPlan)?;
            if plan.candidate.path.get(..prefix) != active.candidate.path.get(..prefix) {
                return Err(WorkspaceError::PrefixChanged);
            }
        }
        self.session_mut().plan = Some(plan);
        self.revision += 1;
        Ok(self.snapshot())
    }
    pub fn next_cycle(&mut self, expected: u32) -> Result<WorkspaceSnapshot, WorkspaceError> {
        self.check_revision(expected)?;
        let s = self.session();
        let plan = s.plan.as_ref().ok_or(WorkspaceError::NoPlan)?;
        if !plan.complete || plan.steps.len() != s.pieces.len() {
            return Err(WorkspaceError::IncompletePlan);
        }
        let board = plan.candidate.board;
        let states = plan.candidate.states;
        let s = self.session_mut();
        s.board = board;
        s.states = states;
        s.pieces.clear();
        s.checkpoint = None;
        s.checkpoint_id += 1;
        s.source = "predicted".to_owned();
        s.cycle += 1;
        s.plan = None;
        s.recommendation = None;
        s.phase = Phase::AwaitingInput;
        self.revision += 1;
        Ok(self.snapshot())
    }
    pub fn accept_prediction(
        &mut self,
        expected: u32,
        board: Board,
        pieces: Vec<Piece>,
        states: [ComboState; 3],
        context: RuleContext,
    ) -> Result<WorkspaceSnapshot, WorkspaceError> {
        self.accept_observation(expected, board, pieces, states, context, true)?;
        self.session.source = "predicted".to_owned();
        Ok(self.snapshot())
    }
    pub fn accept_observation(
        &mut self,
        expected: u32,
        board: Board,
        mut pieces: Vec<Piece>,
        states: [ComboState; 3],
        context: RuleContext,
        known: bool,
    ) -> Result<WorkspaceSnapshot, WorkspaceError> {
        self.check_revision(expected)?;
        pieces.sort_by_key(|p| p.slot);
        let game = Game {
            checkpoint: None,
            board,
            pieces: pieces.clone(),
            states,
            context,
            weights: self.weights,
        };
        if !pieces.is_empty() {
            game.validate().map_err(|_| WorkspaceError::InvalidInput)?;
        }
        self.session.board = board;
        self.session.pieces = pieces;
        self.session.states = states;
        self.session.context = context;
        self.session.checkpoint = None;
        self.session.checkpoint_id += 1;
        self.session.input_known = known;
        self.session.source = "observed".to_owned();
        self.changed_input();
        Ok(self.snapshot())
    }
}
