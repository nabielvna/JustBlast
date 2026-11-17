import type {
  CommandError,
  CalibrationView,
  Profile,
  WorkspaceView,
  PlanView,
  ExplorerView,
  PieceView,
} from '../../generated/contracts'

export function record(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
}
const integer = (value: unknown, min = 0, max = 0xffff_ffff): value is number =>
  typeof value === 'number' && Number.isInteger(value) && value >= min && value <= max
const finite = (value: unknown): value is number =>
  typeof value === 'number' && Number.isFinite(value)
const board = (value: unknown): boolean =>
  Array.isArray(value) &&
  value.length === 8 &&
  value.every((r: unknown) => typeof r === 'string' && /^[.#]{8}$/.test(r))
const position = (value: unknown): boolean =>
  Array.isArray(value) && value.length === 2 && value.every((n: unknown) => integer(n, 0, 7))
const triple = (value: unknown, check: (v: unknown) => boolean): boolean =>
  Array.isArray(value) && value.length === 3 && value.every(check)
const movement = (v: unknown): boolean =>
  record(v) && integer(v.piece, 0, 2) && integer(v.row, 0, 7) && integer(v.column, 0, 7)
const combo = (v: unknown): boolean =>
  record(v) &&
  ['count', 'misses', 'savedCount', 'waiting'].every((k) => integer(v[k])) &&
  integer(v.producerRemaining, 0, 2) &&
  typeof v.checking === 'boolean' &&
  typeof v.recovered === 'boolean'
export function validPiece(v: unknown): v is PieceView {
  return (
    record(v) &&
    integer(v.slot, 0, 2) &&
    (v.catalogId === null || integer(v.catalogId, 1)) &&
    typeof v.name === 'string' &&
    v.name.trim().length > 0 &&
    v.name.length <= 80 &&
    Array.isArray(v.cells) &&
    v.cells.length > 0 &&
    v.cells.length <= 64 &&
    v.cells.every(position) &&
    new Set(v.cells.map((c) => JSON.stringify(c))).size === v.cells.length &&
    v.cells.some((c) => c[0] === 0) &&
    v.cells.some((c) => c[1] === 0)
  )
}
const lineIndices = (v: unknown): boolean =>
  Array.isArray(v) && v.every((n) => integer(n, 0, 7)) && new Set(v).size === v.length
export function parsePlan(value: unknown): PlanView {
  if (
    !record(value) ||
    typeof value.complete !== 'boolean' ||
    !integer(value.requestedPieces, 1, 3) ||
    !Array.isArray(value.steps) ||
    value.steps.length > value.requestedPieces ||
    !board(value.finalBoard) ||
    !integer(value.visited) ||
    !triple(value.modelPoints, finite) ||
    !triple(value.normalizationMaxima, finite) ||
    !triple(value.finalStates, combo)
  )
    throw new Error('Invalid plan response')
  const m = value.metrics
  if (
    !record(m) ||
    !['value', 'potentialPoints', 'flexibility', 'fragmentation'].every((k) => finite(m[k])) ||
    typeof m.comboAlive !== 'boolean' ||
    !integer(m.shapeCoverage) ||
    !integer(m.totalPositions) ||
    !triple(m.normalized, (n) => n === null || finite(n))
  )
    throw new Error('Invalid plan metrics')
  const used = new Set<number>()
  for (const s of value.steps) {
    if (
      !record(s) ||
      !movement(s.movement) ||
      !integer(s.slot, 0, 2) ||
      used.has(s.slot) ||
      !board(s.before) ||
      !board(s.placed) ||
      !board(s.after) ||
      !lineIndices(s.clearedRows) ||
      !lineIndices(s.clearedColumns) ||
      !integer(s.possibleMoves, 1, 64) ||
      !triple(s.statesBefore, combo) ||
      !triple(s.statesAfter, combo) ||
      !triple(s.modelPoints, finite)
    )
      throw new Error('Invalid plan step')
    used.add(s.slot)
  }
  return value as unknown as PlanView
}
export function parseWorkspace(value: unknown): WorkspaceView {
  if (!record(value)) throw new Error('Invalid workspace response')
  const capabilities = value.capabilities
  if (
    value.schemaVersion !== 2 ||
    !integer(value.revision, 1) ||
    !Array.isArray(value.consumedSlots) ||
    !value.consumedSlots.every((s: unknown) => integer(s, 0, 2)) ||
    !integer(value.completed, 0, 3) ||
    !integer(value.checkpoint, 1) ||
    typeof value.inputKnown !== 'boolean' ||
    !['entered', 'observed', 'corrected', 'predicted'].includes(String(value.source)) ||
    !board(value.currentBoard) ||
    !board(value.board) ||
    !Array.isArray(value.board) ||
    value.occupied !== value.board.join('').split('#').length - 1 ||
    !record(capabilities) ||
    !['boardInput', 'pieceInput', 'solver', 'device'].every(
      (k) => typeof capabilities[k] === 'boolean',
    ) ||
    !Array.isArray(value.pieces) ||
    value.pieces.length > 3 ||
    !value.pieces.every(validPiece) ||
    new Set(value.pieces.map((p) => p.slot)).size !== value.pieces.length ||
    !integer(value.cycle, 1) ||
    !['awaiting_input', 'searching', 'ready', 'no_solution'].includes(String(value.phase)) ||
    typeof value.explorerReady !== 'boolean'
  )
    throw new Error('Invalid workspace response. Restart JustBlast or update the app.')
  const weights = value.weights
  if (
    !record(weights) ||
    !['points', 'combo', 'flexibility', 'fragmentation'].every(
      (k) => finite(weights[k]) && (weights[k] as number) >= 0,
    )
  )
    throw new Error('Invalid scoring weights')
  if (value.plan !== null) parsePlan(value.plan)
  if (value.job !== null) {
    const j = value.job
    if (
      !record(j) ||
      !integer(j.id, 1) ||
      !['solve', 'explore'].includes(String(j.kind)) ||
      !['running', 'completed', 'cancelled', 'failed'].includes(String(j.status)) ||
      !integer(j.visited) ||
      (j.message !== null && typeof j.message !== 'string')
    )
      throw new Error('Invalid search status')
  }
  parseDevice(value.device)
  return value as unknown as WorkspaceView
}
export function parseExplorer(v: unknown): ExplorerView {
  if (
    !record(v) ||
    typeof v.complete !== 'boolean' ||
    !integer(v.total) ||
    !integer(v.allTotal) ||
    !integer(v.page) ||
    !integer(v.pages, 1) ||
    v.page >= v.pages ||
    !Array.isArray(v.candidates) ||
    v.candidates.length > 100 ||
    !triple(v.nextCounts, (n) => integer(n)) ||
    !Array.isArray(v.positionOptions) ||
    !v.positionOptions.every((g) => Array.isArray(g) && g.every(position))
  )
    throw new Error('Invalid alternatives response')
  for (const c of v.candidates) {
    if (
      !record(c) ||
      !integer(c.id, 1) ||
      !integer(c.clears, 0, 48) ||
      (c.score !== null && !finite(c.score)) ||
      !Array.isArray(c.moves) ||
      c.moves.length < 1 ||
      c.moves.length > 3 ||
      !c.moves.every(movement)
    )
      throw new Error('Invalid alternative plan')
  }
  return v as unknown as ExplorerView
}
export function nativeError(reason: unknown): Error & { code?: CommandError['code'] } {
  if (reason instanceof Error) return reason
  const codes: CommandError['code'][] = [
    'INVALID_INPUT',
    'STALE_REVISION',
    'READ_ONLY_BOARD',
    'BUSY',
    'NO_PLAN',
    'INCOMPLETE_PLAN',
    'PREFIX_CHANGED',
    'UNAVAILABLE',
    'DEVICE',
    'STORAGE',
    'INTERNAL',
  ]
  if (
    record(reason) &&
    typeof reason.message === 'string' &&
    codes.includes(reason.code as CommandError['code'])
  )
    return Object.assign(new Error(reason.message), { code: reason.code as CommandError['code'] })
  return new Error('The desktop connection failed. Restart JustBlast and try again.')
}

export function parseDevice(value: unknown): void {
  if (
    !record(value) ||
    !integer(value.revision, 1) ||
    typeof value.state !== 'string' ||
    typeof value.profileId !== 'string' ||
    !Array.isArray(value.profileIds) ||
    !value.profileIds.every((id: unknown) => typeof id === 'string') ||
    typeof value.message !== 'string' ||
    (value.error !== null && typeof value.error !== 'string') ||
    typeof value.busy !== 'boolean' ||
    (value.serial !== null && typeof value.serial !== 'string') ||
    !integer(value.completed, 0, 3) ||
    typeof value.needsScan !== 'boolean' ||
    typeof value.cycleDone !== 'boolean' ||
    typeof value.provenance !== 'string' ||
    typeof value.readiness !== 'string' ||
    !integer(value.queuedVisuals, 0, 64) ||
    !integer(value.droppedVisuals) ||
    !Array.isArray(value.devices) ||
    !value.devices.every(
      (d: unknown) =>
        record(d) &&
        typeof d.serial === 'string' &&
        typeof d.state === 'string' &&
        typeof d.model === 'string',
    )
  )
    throw new Error('Invalid device response')

  for (const c of [value.capture]) {
    if (c === null) continue
    if (
      !record(c) ||
      !finite(c.pitch) ||
      c.pitch <= 0 ||
      !integer(c.id, 1) ||
      (c.board !== null && !board(c.board)) ||
      !Array.isArray(c.pieces) ||
      c.pieces.length > 3 ||
      !c.pieces.every(validPiece) ||
      !Array.isArray(c.unknownSlots) ||
      !c.unknownSlots.every((s: unknown) => integer(s, 0, 2)) ||
      !Array.isArray(c.slotIssues) ||
      c.slotIssues.length !== 3 ||
      !c.slotIssues.every(
        (s: unknown, i: number) =>
          (s === null || typeof s === 'string') &&
          Array.isArray(c.unknownSlots) &&
          (s !== null) === c.unknownSlots.includes(i),
      ) ||
      new Set(c.unknownSlots).size !== c.unknownSlots.length ||
      !Array.isArray(c.slotCrops) ||
      c.slotCrops.length !== 3 ||
      !c.slotCrops.every(
        (s: unknown) =>
          typeof s === 'string' && s.startsWith('data:image/png;base64,') && s.length < 1000000,
      ) ||
      !Array.isArray(c.slotSizes) ||
      c.slotSizes.length !== 3 ||
      !c.slotSizes.every(
        (s: unknown) =>
          Array.isArray(s) && s.length === 2 && s.every((v: unknown) => integer(v, 1, 8192)),
      ) ||
      !Array.isArray(c.anchors) ||
      c.anchors.length > 3 ||
      !c.anchors.every(
        (a: unknown) =>
          record(a) && integer(a.slot, 0, 2) && integer(a.x, 0, 8192) && integer(a.y, 0, 8192),
      )
    )
      throw new Error('Invalid capture evidence')
  }
  if (value.visual !== null) {
    const v = value.visual
    if (
      !record(v) ||
      !integer(v.id, 1) ||
      !integer(v.cycle, 1) ||
      !integer(v.revision, 1) ||
      !board(v.board) ||
      !Array.isArray(v.pieces) ||
      !v.pieces.every(validPiece) ||
      !integer(v.completed, 0, 3) ||
      typeof v.done !== 'boolean'
    )
      throw new Error('Invalid visual progress')
    parsePlan(v.plan)
  }
}

export function validHardwareProfile(v: unknown): v is Profile {
  if (!record(v) || !record(v.screen) || !record(v.touch) || !record(v.runtime)) return false
  const finite = (n: unknown) => typeof n === 'number' && Number.isFinite(n)
  const positive = (n: unknown) => finite(n) && Number(n) > 0
  const dimension = (n: unknown) => Number.isInteger(n) && Number(n) > 0 && Number(n) <= 8192
  if (
    !dimension(v.screen.width) ||
    !dimension(v.screen.height) ||
    Number(v.screen.width) * Number(v.screen.height) * 4 > 25_000_000
  )
    return false
  const rect = (r: unknown) =>
    Array.isArray(r) &&
    r.length === 4 &&
    r.every((n) => Number.isInteger(n) && n >= 0) &&
    r[2] > 0 &&
    r[3] > 0 &&
    r[0] + r[2] <= Number((v.screen as Record<string, unknown>).width) &&
    r[1] + r[3] <= Number((v.screen as Record<string, unknown>).height)
  return (
    rect(v.screen.board) &&
    Array.isArray(v.screen.slots) &&
    v.screen.slots.length === 3 &&
    v.screen.slots.every(rect) &&
    positive(v.screen.preview_pitch) &&
    Number(v.screen.preview_pitch) >= 4 &&
    positive(v.screen.cell_majority) &&
    Number(v.screen.cell_majority) <= 1 &&
    positive(v.touch.gain) &&
    finite(v.touch.lift) &&
    Number(v.touch.lift) >= 0 &&
    Number(v.touch.lift) <= 600 &&
    ['hold', 'drop', 'gap', 'risk_max'].every(
      (k) =>
        finite((v.touch as Record<string, unknown>)[k]) &&
        Number((v.touch as Record<string, unknown>)[k]) >= 0 &&
        Number((v.touch as Record<string, unknown>)[k]) <= 1,
    ) &&
    Number(v.touch.risk_max) >=
      Math.max(Number(v.touch.hold), Number(v.touch.drop), Number(v.touch.gap)) &&
    ['tray_wait', 'retry_wait', 'command_timeout'].every(
      (k) =>
        positive((v.runtime as Record<string, unknown>)[k]) &&
        Number((v.runtime as Record<string, unknown>)[k]) <= 120,
    ) &&
    Number.isInteger(v.runtime.tray_attempts) &&
    Number(v.runtime.tray_attempts) >= 1 &&
    Number(v.runtime.tray_attempts) <= 20
  )
}
export function parseCalibration(v: unknown): CalibrationView {
  if (
    !record(v) ||
    !validHardwareProfile(v.profile) ||
    !Number.isInteger(v.revision) ||
    Number(v.revision) < 1 ||
    typeof v.toml !== 'string' ||
    !(v.captureId === null || (Number.isInteger(v.captureId) && Number(v.captureId) > 0)) ||
    !(
      v.image === null ||
      (typeof v.image === 'string' && /^data:image\/png;base64,[A-Za-z0-9+/=]+$/.test(v.image))
    ) ||
    !(
      v.board === null ||
      (Array.isArray(v.board) &&
        v.board.length === 8 &&
        v.board.every((r) => typeof r === 'string' && /^[.#]{8}$/.test(r)))
    ) ||
    typeof v.boardUnclear !== 'boolean' ||
    !(v.layoutDetected === null || typeof v.layoutDetected === 'boolean') ||
    !Array.isArray(v.unclearCells) ||
    !v.unclearCells.every((n) => integer(n, 0, 63)) ||
    !Array.isArray(v.pieces) ||
    !v.pieces.every(validPiece) ||
    !Array.isArray(v.unknownSlots) ||
    !v.unknownSlots.every((n) => Number.isInteger(n) && n >= 0 && n <= 2) ||
    !Array.isArray(v.anchors) ||
    !v.anchors.every(
      (a) =>
        record(a) &&
        Number.isInteger(a.slot) &&
        Number(a.slot) >= 0 &&
        Number(a.slot) < 3 &&
        Number.isInteger(a.x) &&
        Number(a.x) >= 0 &&
        Number(a.x) < (v.profile as Profile).screen.width &&
        Number.isInteger(a.y) &&
        Number(a.y) >= 0 &&
        Number(a.y) < (v.profile as Profile).screen.height,
    ) ||
    typeof v.message !== 'string' ||
    !(v.movement === null || v.movement === 'verified' || v.movement === 'adjusted')
  )
    throw new Error('Invalid calibration response')
  return v as CalibrationView
}
