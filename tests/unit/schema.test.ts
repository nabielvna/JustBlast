import assert from 'node:assert/strict'
import { test } from 'node:test'
import { nativeError, parseWorkspace } from '../../ui/src/lib/native/schema.ts'

const valid = () => ({
  schemaVersion: 2,
  revision: 1,
  completed: 0,
  checkpoint: 1,
  inputKnown: false,
  source: 'entered',
  consumedSlots: [],
  currentBoard: Array<string>(8).fill('........'),
  board: Array<string>(8).fill('........'),
  occupied: 0,
  device: {
    revision: 1,
    state: 'disconnected',
    profileId: 'default',
    profileIds: ['default'],
    message: 'Select a phone',
    error: null,
    busy: false,
    serial: null,
    devices: [],
    completed: 0,
    needsScan: true,
    cycleDone: false,
    provenance: 'none',
    visual: null,
    queuedVisuals: 0,
    droppedVisuals: 0,
    readiness: 'Scan first',
    capture: null,
  },
  pieces: [],
  cycle: 1,
  phase: 'awaiting_input',
  plan: null,
  weights: { points: 45, combo: 25, flexibility: 25, fragmentation: 5 },
  job: null,
  explorerReady: false,
  capabilities: { boardInput: true, pieceInput: false, solver: false, device: false },
})

test('rejects malformed or incompatible native responses before they reach the UI', () => {
  for (const value of [
    null,
    {},
    { ...valid(), schemaVersion: 1 },
    { ...valid(), revision: -1 },
    { ...valid(), board: ['........'] },
    { ...valid(), source: 'demo' },
    { ...valid(), occupied: 64 },
    { ...valid(), capabilities: { device: true } },
  ]) {
    assert.throws(() => parseWorkspace(value))
  }
  const value = valid()
  value.board[7] = '.......#'
  value.occupied = 1
  assert.equal(parseWorkspace(value).board[7], '.......#')
})

test('preserves structured native errors for stale-state recovery', () => {
  const failure = nativeError({ code: 'STALE_REVISION', message: 'Refresh your workspace.' })
  assert.equal(failure.code, 'STALE_REVISION')
  assert.equal(failure.message, 'Refresh your workspace.')
  assert.equal(nativeError({ secret: 'unexpected' }).message.includes('unexpected'), false)
})

test('nested plans and device progress reject malformed geometry, non-finite scores and impossible acknowledgements', async () => {
  const { parsePlan, parseExplorer } = await import('../../ui/src/lib/native/schema.ts')
  const board = Array(8).fill('........')
  const combo = {
    count: 0,
    misses: 0,
    producerRemaining: 0,
    savedCount: 0,
    checking: false,
    waiting: 0,
    recovered: false,
  }
  const plan = {
    complete: true,
    requestedPieces: 1,
    steps: [
      {
        movement: { piece: 0, row: 0, column: 0 },
        slot: 0,
        before: board,
        placed: ['#.......', ...board.slice(1)],
        after: ['#.......', ...board.slice(1)],
        clearedRows: [],
        clearedColumns: [],
        possibleMoves: 64,
        statesBefore: [combo, combo, combo],
        statesAfter: [combo, combo, combo],
        modelPoints: [1, 1, 1],
      },
    ],
    finalBoard: ['#.......', ...board.slice(1)],
    metrics: {
      value: 45,
      potentialPoints: 1,
      comboAlive: false,
      flexibility: 1,
      fragmentation: 0,
      shapeCoverage: 35,
      totalPositions: 100,
      normalized: [1, 1, 1],
    },
    modelPoints: [1, 1, 1],
    normalizationMaxima: [1, 1, 1],
    finalStates: [combo, combo, combo],
    visited: 65,
  }
  assert.equal(parsePlan(plan).steps.length, 1)
  for (const bad of [
    { ...plan, metrics: { ...plan.metrics, value: NaN } },
    { ...plan, steps: [{ ...plan.steps[0], movement: { piece: 0, row: 8, column: 0 } }] },
    { ...plan, steps: [{ ...plan.steps[0], clearedRows: [0, 0] }] },
    { ...plan, modelPoints: [1, 1, Infinity] },
  ])
    assert.throws(() => parsePlan(bad))
  assert.throws(() => parseWorkspace({ ...valid(), device: { ...valid().device, completed: 4 } }))
  assert.throws(() =>
    parseExplorer({
      complete: true,
      total: 1,
      allTotal: 1,
      page: 1,
      pages: 1,
      candidates: [],
      nextCounts: [0, 0, 0],
      positionOptions: [],
    }),
  )
})

test('capture issues distinguish uncertain slots from empty ones at the IPC boundary', () => {
  const capture = {
    id: 1,
    board: Array(8).fill('........'),
    pieces: [{ slot: 0, catalogId: null, name: 'Partial shape', cells: [[0, 0]] }],
    unknownSlots: [0],
    slotIssues: ['shape_not_in_catalogue', null, null],
    slotCrops: Array(3).fill('data:image/png;base64,AA=='),
    slotSizes: Array(3).fill([300, 300]),
    pitch: 54,
    anchors: [{ slot: 0, x: 0, y: 0 }],
  }
  const response = (value: unknown) => ({
    ...valid(),
    device: { ...valid().device, capture: value },
  })
  assert.deepEqual(parseWorkspace(response(capture)).device.capture?.unknownSlots, [0])
  assert.throws(() => parseWorkspace(response({ ...capture, unknownSlots: [] })))
  assert.throws(() => parseWorkspace(response({ ...capture, unknownSlots: [0, 0] })))
  assert.throws(() => parseWorkspace(response({ ...capture, slotIssues: [null, null, null] })))
  const empty = {
    ...capture,
    pieces: [],
    unknownSlots: [],
    slotIssues: [null, null, null],
    anchors: [],
  }
  assert.deepEqual(parseWorkspace(response(empty)).device.capture?.pieces, [])
})
