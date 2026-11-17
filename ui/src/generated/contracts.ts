export type Profile = { screen: Screen, touch: Touch, runtime: Timing, };

export type Screen = { width: number, height: number, board: [number, number, number, number], slots: [[number, number, number, number], [number, number, number, number], [number, number, number, number]], preview_pitch: number, cell_majority: number, };

export type Touch = { gain: number, lift: number, hold: number, drop: number, gap: number, risk_max: number, };

export type Timing = { tray_wait: number, retry_wait: number, tray_attempts: number, command_timeout: number, };

export type CalibrationAction = { "type": "read", } | { "type": "capture", serial: string, } | { "type": "preview", captureId: number, profile: Profile, } | { "type": "autoLayout", captureId: number, profile: Profile, } | { "type": "export", profile: Profile, } | { "type": "testDrag", captureId: number, profile: Profile, slot: number, row: number, column: number, installHelper: boolean, };

export type CalibrationRequest = { commandId: string, expectedRevision: number, action: CalibrationAction, };

export type CalibrationView = { layoutDetected: boolean | null, unclearCells: Array<number>, anchors: Array<SlotAnchor>, movement: string | null, revision: number, profile: Profile, toml: string, captureId: number | null, image: string | null, board: [string, string, string, string, string, string, string, string] | null, boardUnclear: boolean, pieces: Array<PieceView>, unknownSlots: Array<number>, message: string, };

export type DeviceInfoView = { serial: string, state: string, model: string, };

export type VisualCycleView = { id: number, cycle: number, revision: number, board: [string, string, string, string, string, string, string, string], pieces: Array<PieceView>, plan: PlanView, completed: number, done: boolean, };

export type SlotAnchor = { slot: number, x: number, y: number, };

export type CaptureView = { id: number, board: [string, string, string, string, string, string, string, string] | null, pieces: Array<PieceView>, unknownSlots: Array<number>, slotIssues: [string | null, string | null, string | null], slotCrops: Array<string>, slotSizes: Array<[number, number]>, pitch: number, anchors: Array<SlotAnchor>, };

export type DeviceView = { revision: number, state: string, profileId: string, profileIds: Array<string>, message: string, error: string | null, busy: boolean, serial: string | null, devices: Array<DeviceInfoView>, completed: number, needsScan: boolean, cycleDone: boolean, provenance: string, visual: VisualCycleView | null, queuedVisuals: number, droppedVisuals: number, capture: CaptureView | null, readiness: string, };

export type DeviceAction = { "type": "discover", } | { "type": "wireless", address: string, code: string | null, } | { "type": "connect", serial: string, installHelper: boolean, } | { "type": "disconnect", } | { "type": "scan", workspaceRevision: number, } | { "type": "captureForEdit", workspaceRevision: number, } | { "type": "nextCycle", workspaceRevision: number, } | { "type": "start", workspaceRevision: number, through: number | null, } | { "type": "stop", } | { "type": "forceStop", } | { "type": "acceptWarning", } | { "type": "profile", id: string, toml: string, } | { "type": "selectProfile", id: string, } | { "type": "acknowledgeVisual", cycleId: number, };

export type DeviceRequest = { commandId: string, expectedRevision: number, action: DeviceAction, };

export type SavedShape = { id: string, name: string, cells: Array<[number, number]>, };

export type ScoreWeights = { points: number, combo: number, flexibility: number, fragmentation: number, };

export type AutomationSettings = { moveDevice: boolean, continuous: boolean, checkAfterMove: boolean, checkAfterCycle: boolean, cycleLimit: number, };

export type DeviceSettings = { serial: string, wirelessAddress: string, method: string, profileId: string, };

export type Preferences = { schemaVersion: number, revision: number, shapes: Array<SavedShape>, automation: AutomationSettings, connection: DeviceSettings, weights: ScoreWeights, };

export type HistoryEntry = { timestamp: string, source: string, cycle: number, status: string, message: string, board: [string, string, string, string, string, string, string, string], };

export type PieceView = { slot: number, catalogId: number | null, name: string, cells: Array<[number, number]>, };

export type WeightsView = { points: number, combo: number, flexibility: number, fragmentation: number, };

export type ComboView = { count: number, misses: number, producerRemaining: number, savedCount: number, checking: boolean, waiting: number, recovered: boolean, };

export type MoveView = { piece: number, row: number, column: number, };

export type StepView = { movement: MoveView, slot: number, before: [string, string, string, string, string, string, string, string], placed: [string, string, string, string, string, string, string, string], after: [string, string, string, string, string, string, string, string], clearedRows: Array<number>, clearedColumns: Array<number>, possibleMoves: number, statesBefore: [ComboView, ComboView, ComboView], statesAfter: [ComboView, ComboView, ComboView], modelPoints: [number, number, number], };

export type MetricsView = { value: number, potentialPoints: number, comboAlive: boolean, flexibility: number, fragmentation: number, shapeCoverage: number, totalPositions: number, normalized: [number | null, number | null, number | null], };

export type PlanView = { complete: boolean, requestedPieces: number, steps: Array<StepView>, finalBoard: [string, string, string, string, string, string, string, string], metrics: MetricsView, modelPoints: [number, number, number], normalizationMaxima: [number, number, number], finalStates: [ComboView, ComboView, ComboView], visited: number, };

export type JobStatus = "running" | "completed" | "cancelled" | "failed";

export type JobView = { id: number, kind: string, status: JobStatus, visited: number, message: string | null, };

export type WorkspaceAction = { "type": "pieces", expectedRevision: number, pieces: Array<PieceView>, } | { "type": "search", expectedRevision: number, maxNodes: number | null, } | { "type": "cancel", expectedRevision: number, } | { "type": "nextCycle", expectedRevision: number, } | { "type": "applyInput", expectedRevision: number, rows: [string, string, string, string, string, string, string, string], pieces: Array<PieceView>, anchors: Array<SlotAnchor>, } | { "type": "updateInput", expectedRevision: number, rows: [string, string, string, string, string, string, string, string], pieces: Array<PieceView>, anchors: Array<SlotAnchor>, acceptedSlots: Array<number>, acceptBoard: boolean, } | { "type": "explore", expectedRevision: number, completed: number, maxNodes: number | null, } | { "type": "select", expectedRevision: number, candidateId: number, completed: number, } | { "type": "restore", expectedRevision: number, completed: number, } | { "type": "weights", expectedRevision: number, weights: WeightsView, };

export type ExplorerRequest = { expectedRevision: number, order: Array<number>, positions: Array<[number, number] | null>, page: number, pageSize: number, };

export type CandidateView = { id: number, moves: Array<MoveView>, score: number | null, clears: number, };

export type ExplorerView = { complete: boolean, total: number, allTotal: number, page: number, pages: number, candidates: Array<CandidateView>, nextCounts: [number, number, number], positionOptions: Array<Array<[number, number]>>, };

export type CandidateRequest = { expectedRevision: number, candidateId: number, };

export type Capabilities = { boardInput: boolean, pieceInput: boolean, solver: boolean, device: boolean, };

export type WorkspaceView = { schemaVersion: number, revision: number, completed: number, checkpoint: number, inputKnown: boolean, consumedSlots: Array<number>, source: string, currentBoard: [string, string, string, string, string, string, string, string], board: [string, string, string, string, string, string, string, string], occupied: number, pieces: Array<PieceView>, cycle: number, phase: string, plan: PlanView | null, weights: WeightsView, job: JobView | null, explorerReady: boolean, capabilities: Capabilities, device: DeviceView, };

export type ReplaceBoardRequest = { expectedRevision: number, rows: [string, string, string, string, string, string, string, string], };

export type ErrorCode = "INVALID_INPUT" | "STALE_REVISION" | "READ_ONLY_BOARD" | "BUSY" | "NO_PLAN" | "INCOMPLETE_PLAN" | "PREFIX_CHANGED" | "UNAVAILABLE" | "DEVICE" | "STORAGE" | "INTERNAL";

export type CommandError = { code: ErrorCode, message: string, };
