import type {
  WorkspaceView,
  WorkspaceAction,
  PieceView,
  PlanView,
  ExplorerView,
  WeightsView,
  Preferences,
  DeviceAction,
  SlotAnchor,
} from '../../generated/contracts'
import { BOARD_MOTION_MS } from '../../lib/presentation/motion'
import { nativeError } from '../../lib/native/schema'
import { preferencesApi } from '../../lib/native/preferences'
import { workspaceApi } from '../../lib/native/workspace'

type Action = WorkspaceAction extends infer A
  ? A extends { expectedRevision: number }
    ? Omit<A, 'expectedRevision'>
    : never
  : never
export function createWorkspace() {
  let snapshot = $state<WorkspaceView | null>(null)
  let preferences = $state<Preferences | null>(null)
  let visualId: number | null = null
  const physical = () => !!preferences?.automation.moveDevice
  let previewActive = $state(false)
  let pending = $state(false)
  let error = $state<string | null>(null)
  let preview = $state<PlanView | null>(null)
  let previewId = $state<number | null>(null)
  let candidateId = $state<number | null>(null)
  let explorer = $state<ExplorerView | null>(null)
  let exploring = $state(false)
  let hoverPlan = $state<PlanView | null>(null)
  let hoverToken = 0
  let hoverCount = $state<number | null>(null)
  let explorerTicket = 0
  let previewToken = 0
  let pageSize = $state(10)
  let order = $state<number[]>([])
  let positions = $state<([number, number] | null)[]>([])
  let completed = $state(0)
  let selectedStep = $state(0)
  let phase = $state<'idle' | 'placing' | 'breaking' | 'settling'>('idle')
  let running = $state(false)
  let locked = $state(false)
  let target = -1
  let generation = 0
  let timer: ReturnType<typeof setTimeout> | undefined
  let disposed = false
  let pollTimer: ReturnType<typeof setTimeout> | undefined
  const plan = () =>
    preview ?? (!previewActive ? snapshot?.device.visual?.plan : null) ?? snapshot?.plan ?? null
  const pieces = () =>
    (!previewActive ? snapshot?.device.visual?.pieces : null) ?? snapshot?.pieces ?? []
  const initialBoard = () =>
    (!previewActive ? snapshot?.device.visual?.board : null) ?? snapshot?.board
  function cancelTimer() {
    generation++
    clearTimeout(timer)
    timer = undefined
  }
  function resetPlayback() {
    cancelTimer()
    previewActive = false
    completed = snapshot?.device.visual?.completed ?? snapshot?.completed ?? 0
    selectedStep = completed
    phase = 'idle'
    running = false
    locked = completed > 0
    target = -1
  }
  function clearPlanPresentation(reset = true) {
    previewToken++
    hoverToken++
    explorerTicket++
    if (reset) resetPlayback()
    preview = null
    previewId = null
    candidateId = null
    explorer = null
    exploring = false
    order = []
    positions = []
  }
  async function run(operation: () => Promise<WorkspaceView>) {
    if (pending || disposed) return
    pending = true
    error = null
    try {
      const next = await operation()
      if (!disposed) acceptSnapshot(next)
    } catch (reason) {
      const failure = nativeError(reason)
      if (!disposed) error = failure.message
      if (!disposed && failure.code === 'STALE_REVISION') {
        clearPlanPresentation()
        try {
          const next = await workspaceApi.snapshot()
          if (!disposed) acceptSnapshot(next)
        } catch {}
      }
    } finally {
      if (!disposed) pending = false
    }
  }
  async function act(action: Action) {
    const accepted = snapshot
    if (!accepted) return
    await run(() =>
      workspaceApi.action({ ...action, expectedRevision: accepted.revision } as WorkspaceAction),
    )
  }
  async function updateInput(
    rows = snapshot?.currentBoard,
    pieces = snapshot?.pieces,
    acceptedSlots: number[] = [],
    acceptBoard = false,
    anchors: SlotAnchor[] = [],
  ) {
    if (!snapshot || !rows || !pieces || pending || snapshot.device.busy) return
    clearPlanPresentation()
    await act({ type: 'updateInput', rows, pieces, anchors, acceptedSlots, acceptBoard })
    visualId = null
    if (error) throw new Error(error)
  }

  async function loadPage(page = 0) {
    const accepted = snapshot
    if (!accepted?.explorerReady || disposed) return
    const ticket = ++explorerTicket
    try {
      const fixed = accepted.plan?.steps.slice(0, completed) ?? []
      const next = await workspaceApi.explorer({
        expectedRevision: accepted.revision,
        order: [...fixed.map((s) => s.slot), ...order],
        positions: [
          ...fixed.map((s) => [s.movement.row, s.movement.column] as [number, number]),
          ...positions,
        ],
        page,
        pageSize,
      })
      next.positionOptions = next.positionOptions.slice(completed)
      if (!disposed && ticket === explorerTicket && snapshot?.revision === accepted.revision)
        explorer = next
    } catch (reason) {
      if (!disposed) error = nativeError(reason).message
    }
  }
  async function poll() {
    if (disposed) return
    const wasRunning = snapshot?.job?.status === 'running' || !!snapshot?.device.busy
    if (wasRunning && !pending) {
      try {
        const next = await workspaceApi.snapshot()
        if (
          !disposed &&
          !pending &&
          next.revision >= (snapshot?.revision ?? 0) &&
          next.device.revision >= (snapshot?.device.revision ?? 0)
        )
          acceptSnapshot(next)
      } catch (reason) {
        if (!disposed) error = nativeError(reason).message
      }
      if (snapshot?.job?.status === 'failed') error = snapshot.job.message
      if (snapshot?.device.error) error = snapshot.device.error
      if (snapshot?.explorerReady && exploring && !explorer) await loadPage()
    }
    const visual = snapshot?.device.visual
    if (
      !previewActive &&
      visual?.done &&
      !pending &&
      snapshot?.plan &&
      snapshot.revision !== visual.revision &&
      snapshot.job?.status !== 'running'
    )
      await deviceAction({ type: 'acknowledgeVisual', cycleId: visual.id })
    if (!disposed) pollTimer = setTimeout(() => void poll(), snapshot?.device.busy ? 50 : 100)
  }
  function schedule(callback: () => void, duration: number) {
    const token = generation
    timer = setTimeout(() => {
      if (!disposed && generation === token) callback()
    }, duration)
  }
  function settle() {
    completed++
    phase = 'settling'
    schedule(() => {
      phase = 'idle'
      selectedStep = completed
      if (running && completed <= target && completed < (plan()?.steps.length ?? 0))
        schedule(advance, 16)
      else running = false
    }, BOARD_MOTION_MS)
  }
  function advance() {
    if (!running) return
    const step = plan()?.steps[completed]
    if (!step || preview || disposed) {
      running = false
      return
    }
    selectedStep = completed
    locked = true
    phase = 'placing'
    schedule(() => {
      if (step.clearedRows.length || step.clearedColumns.length) {
        phase = 'breaking'
        schedule(settle, BOARD_MOTION_MS)
      } else settle()
    }, BOARD_MOTION_MS)
  }
  async function deviceAction(action: DeviceAction) {
    if (!snapshot) return
    const warningRevision = action.type === 'acceptWarning' ? snapshot.device.revision : null
    await run(async () => {
      const current = await workspaceApi.snapshot()
      const requestAction =
        action.type === 'scan' ? { ...action, workspaceRevision: current.revision } : action
      return workspaceApi.device({
        commandId: crypto.randomUUID(),
        expectedRevision: warningRevision ?? current.device.revision,
        action: requestAction,
      })
    })
    if (snapshot?.device.error) error = snapshot.device.error
    if (action.type === 'profile' || action.type === 'selectProfile')
      preferences = await preferencesApi.snapshot()
  }
  function acceptSnapshot(next: WorkspaceView) {
    const old = snapshot
    snapshot = next
    if (
      old &&
      next.source === 'observed' &&
      next.checkpoint !== old.checkpoint &&
      next.device.capture &&
      next.device.capture.id !== old.device.capture?.id
    ) {
      clearPlanPresentation()
      visualId = null
    }
    if (old && old.cycle !== next.cycle) {
      clearPlanPresentation()
      previewActive = false
    }
    syncPhysicalProgress()
  }
  function syncPhysicalProgress() {
    if (disposed || previewActive) return
    const visual = snapshot?.device.visual ?? null
    const changedCycle = (visual?.id ?? null) !== visualId
    if (changedCycle) {
      clearPlanPresentation()
      visualId = visual?.id ?? null
    }
    const acknowledged = visual?.completed ?? snapshot?.device.completed ?? 0
    if (!changedCycle && completed === acknowledged) return
    cancelTimer()
    completed = acknowledged
    selectedStep = completed
    locked = completed > 0
    running = false
    phase = completed > 0 ? 'settling' : 'idle'
    if (phase === 'settling')
      schedule(() => {
        phase = 'idle'
      }, BOARD_MOTION_MS)
  }
  return {
    get preferences() {
      return preferences
    },
    get physical() {
      return physical()
    },
    get pieces() {
      return pieces()
    },
    get displayBehind() {
      return (
        !previewActive &&
        !!snapshot?.device.visual &&
        (snapshot.device.visual.revision !== snapshot.revision ||
          completed < snapshot.device.visual.completed ||
          phase !== 'idle')
      )
    },
    get snapshot() {
      return snapshot
    },
    get pending() {
      return pending
    },
    get error() {
      return error
    },
    get plan() {
      return plan()
    },
    get preview() {
      return preview
    },
    get previewId() {
      return previewId
    },
    get candidateId() {
      return candidateId
    },
    get explorer() {
      return explorer
    },
    get hoverPlan() {
      return hoverPlan
    },
    get hoverCount() {
      return hoverCount
    },
    get pageSize() {
      return pageSize
    },
    get exploring() {
      return exploring
    },
    get order() {
      return order
    },
    get positions() {
      return positions
    },
    get completed() {
      return completed
    },
    get selectedStep() {
      return selectedStep
    },
    get phase() {
      return phase
    },
    get running() {
      return running
    },
    get locked() {
      return locked
    },
    get displayBoard(): WorkspaceView['board'] | undefined {
      const p = plan()
      if (!p) return initialBoard()
      if (exploring) return completed ? p.steps[completed - 1]?.after : initialBoard()
      if (phase === 'placing' || phase === 'breaking') return p.steps[completed]?.placed
      if (completed === (snapshot?.completed ?? 0) && p.steps[completed])
        return p.steps[completed]?.before
      if (phase === 'settling' || completed > 0) return p.steps[completed - 1]?.after
      return p.steps[selectedStep]?.before ?? initialBoard()
    },
    refresh: async () => {
      await run(workspaceApi.snapshot)
      if (snapshot && !preferences) {
        try {
          preferences = await preferencesApi.snapshot()
        } catch (e) {
          error = nativeError(e).message
        }
      }
      syncPhysicalProgress()
      if (!pollTimer) void poll()
    },
    savePreferences: async (next: Preferences) => {
      const wasPhysical = physical()
      try {
        const saved = await preferencesApi.save(next)
        preferences = saved
        if (wasPhysical !== physical()) {
          resetPlayback()
          visualId = null
        }
        syncPhysicalProgress()
        await run(workspaceApi.snapshot)
        return saved
      } catch (e) {
        error = nativeError(e).message
        throw e
      }
    },
    deviceAction,
    scan: async () => {
      if (!snapshot) return
      clearPlanPresentation(false)
      await deviceAction({
        type: 'scan',
        workspaceRevision: snapshot.revision,
      })
    },
    execute: async (through: number | null = null) => {
      if (!snapshot || !physical() || preview || exploring || running) return
      resetPlayback()
      await deviceAction({ type: 'start', workspaceRevision: snapshot.revision, through })
    },
    stop: async (force = false) => {
      await deviceAction({ type: force ? 'forceStop' : 'stop' })
    },
    get previewActive() {
      return previewActive
    },
    get actualCompleted() {
      return (
        (!previewActive ? snapshot?.device.visual?.completed : undefined) ??
        snapshot?.completed ??
        0
      )
    },
    updateBoard: (rows: WorkspaceView['board']) => updateInput(rows, undefined, [], true),
    updatePiece: (slot: number, piece: PieceView | null, anchor: SlotAnchor | null = null) =>
      updateInput(
        undefined,
        [...(snapshot?.pieces ?? []).filter((p) => p.slot !== slot), ...(piece ? [piece] : [])],
        [slot],
        false,
        anchor ? [anchor] : [],
      ),
    acceptSlots: (slots: number[]) => updateInput(undefined, undefined, slots),
    acceptBoard: () => updateInput(undefined, undefined, [], true),
    acceptWarning: () => deviceAction({ type: 'acceptWarning' }),
    search: async () => {
      clearPlanPresentation()
      await act({ type: 'search', maxNodes: null })
    },
    cancel: async () => {
      await act({ type: 'cancel' })
      exploring = false
      explorer = null
    },
    nextCycle: async () => {
      if (snapshot?.device.cycleDone)
        await deviceAction({ type: 'nextCycle', workspaceRevision: snapshot.revision })
      else await act({ type: 'nextCycle' })
      if (!error) clearPlanPresentation()
    },
    weights: async (weights: WeightsView) => {
      clearPlanPresentation()
      await act({ type: 'weights', weights })
      if (!error) {
        preferences = await preferencesApi.snapshot()
        if (snapshot?.pieces.length) await act({ type: 'search', maxNodes: null })
      }
    },
    playThrough: (index: number) => {
      if (preview || running || phase !== 'idle' || index < completed || !plan()?.steps[index])
        return
      cancelTimer()
      previewActive = true
      target = index
      running = true
      advance()
    },
    pause: () => {
      running = false
    },
    reset: () => {
      if (physical() || pending || snapshot?.device.busy) return
      resetPlayback()
    },
    selectStep: (index: number) => {
      if (
        !running &&
        !pending &&
        !snapshot?.device.busy &&
        phase === 'idle' &&
        index >= completed &&
        plan()?.steps[index]
      )
        selectedStep = index
    },
    explore: async () => {
      if (running || phase !== 'idle') return
      exploring = true
      const used = new Set(snapshot?.plan?.steps.slice(0, completed).map((s) => s.slot) ?? [])
      const remaining = snapshot?.pieces.filter((p) => !used.has(p.slot)) ?? []
      order = remaining.length === 1 ? [remaining[0]!.slot] : []
      positions = []
      explorer = null
      await act({ type: 'explore', completed, maxNodes: null })
    },
    closeExplorer: () => {
      previewToken++
      exploring = false
      hoverPlan = null
      hoverToken++
      if (snapshot?.job?.kind === 'explore' && snapshot.job.status === 'running')
        void act({ type: 'cancel' })
    },
    hoverCandidate: async (id: number | null) => {
      hoverCount = null
      const token = ++hoverToken
      hoverPlan = null
      const accepted = snapshot
      if (!accepted || id === null) return
      try {
        const next = await workspaceApi.preview({
          expectedRevision: accepted.revision,
          candidateId: id,
        })
        if (!disposed && token === hoverToken && snapshot?.revision === accepted.revision)
          hoverPlan = next
      } catch {}
    },
    hoverPosition: async (index: number, position: [number, number] | null) => {
      const token = ++hoverToken
      hoverPlan = null
      hoverCount = completed + index + 1
      const accepted = snapshot
      if (!accepted?.explorerReady || position === null) return
      const fixed = accepted.plan?.steps.slice(0, completed) ?? []
      try {
        const nextPositions = Array.from({ length: index + 1 }, (_, i) =>
          i === index ? position : (positions[i] ?? null),
        )
        const page = await workspaceApi.explorer({
          expectedRevision: accepted.revision,
          order: [...fixed.map((s) => s.slot), ...order.slice(0, index + 1)],
          positions: [
            ...fixed.map((s) => [s.movement.row, s.movement.column] as [number, number]),
            ...nextPositions,
          ],
          page: 0,
          pageSize: 1,
        })
        const candidate = page.candidates[0]
        if (!candidate || token !== hoverToken) return
        const next = await workspaceApi.preview({
          expectedRevision: accepted.revision,
          candidateId: candidate.id,
        })
        if (!disposed && token === hoverToken && snapshot?.revision === accepted.revision)
          hoverPlan = next
      } catch {}
    },
    setPageSize: async (size: number) => {
      pageSize = size
      await loadPage()
    },
    filter: async (nextOrder: number[], nextPositions: ([number, number] | null)[]) => {
      const used = new Set(snapshot?.plan?.steps.slice(0, completed).map((s) => s.slot) ?? [])
      const remaining = snapshot?.pieces.filter((p) => !used.has(p.slot)).map((p) => p.slot) ?? []
      const rest = remaining.filter((s) => !nextOrder.includes(s))
      order = rest.length === 1 ? [...nextOrder, rest[0]!] : nextOrder
      positions = nextPositions
      hoverPlan = null
      hoverToken++
      await loadPage()
    },
    page: loadPage,
    previewCandidate: async (id: number) => {
      const token = ++previewToken
      const accepted = snapshot
      if (!accepted || running || phase !== 'idle') return
      try {
        const next = await workspaceApi.preview({
          expectedRevision: accepted.revision,
          candidateId: id,
        })
        if (!disposed && token === previewToken && snapshot?.revision === accepted.revision) {
          preview = next
          previewId = id
          selectedStep = completed
          exploring = false
        }
      } catch (reason) {
        error = nativeError(reason).message
      }
    },
    cancelPreview: () => {
      previewToken++
      preview = null
      previewId = null
      selectedStep = completed
    },
    replacePlan: async () => {
      previewToken++
      if (previewId === null) return
      const id = previewId
      await act({ type: 'select', candidateId: id, completed })
      if (!error) {
        candidateId = id
        preview = null
        previewId = null
        selectedStep = completed
      }
    },
    restore: async () => {
      previewToken++
      await act({ type: 'restore', completed })
      if (!error) {
        preview = null
        previewId = null
        candidateId = null
        selectedStep = completed
      }
    },
    dispose: () => {
      disposed = true
      cancelTimer()
      clearTimeout(pollTimer)
    },
  }
}
