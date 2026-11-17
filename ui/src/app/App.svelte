<script lang="ts">
  import { BOARD_MOTION_MS } from '../lib/presentation/motion'
  import { onMount } from 'svelte'
  import Brand from '../components/ui/Brand.svelte'
  import Icon from '../components/ui/Icon.svelte'
  import IconControl from '../components/ui/IconControl.svelte'
  import BoardInput from '../components/board/BoardInput.svelte'
  import BoardDisplay from '../components/board/BoardDisplay.svelte'
  import PieceTray from '../components/board/PieceTray.svelte'
  import SettingsDialog from '../features/settings/SettingsDialog.svelte'
  import PieceEditor from '../features/manual/PieceEditor.svelte'
  import PlanInspector from '../features/plans/PlanInspector.svelte'
  import Explorer from '../features/plans/Explorer.svelte'
  import { createWorkspace } from '../features/workspace/controller.svelte'
  import type { PieceView, WorkspaceView, SlotAnchor } from '../generated/contracts'
  const workspace = createWorkspace()
  let settings: SettingsDialog
  let savingMoveDevice = $state(false)
  async function toggleMoveDevice() {
    if (!workspace.preferences || savingMoveDevice || deviceBusy || workspace.pending) return
    savingMoveDevice = true
    try {
      const next = structuredClone($state.snapshot(workspace.preferences))
      next.automation.moveDevice = !next.automation.moveDevice
      await workspace.savePreferences(next)
    } catch {
    } finally {
      savingMoveDevice = false
    }
  }
  let editingSlot = $state<number | null>(null)
  let editingAnchor = $state<SlotAnchor | null>(null)
  let plan = $derived(workspace.plan)
  let showPlan = $derived(!!plan?.steps.length)
  let searching = $derived(workspace.snapshot?.job?.status === 'running')
  let device = $derived(workspace.snapshot?.device)
  let deviceBusy = $derived(!!device?.busy)
  let consumed = $derived(plan?.steps.slice(0, workspace.completed).map((s) => s.slot) ?? [])
  let actualConsumed = $derived(workspace.snapshot?.consumedSlots ?? [])
  let remainingPieces = $derived(workspace.pieces.filter((p) => !consumed.includes(p.slot)))
  let completePlan = $derived(!!plan?.complete && plan.steps.length === plan.requestedPieces)
  let nextReady = $derived(
    !deviceBusy &&
      completePlan &&
      workspace.completed === plan?.steps.length &&
      workspace.phase === 'idle',
  )
  let canPreview = $derived(
    showPlan &&
      !workspace.preview &&
      !workspace.exploring &&
      !deviceBusy &&
      !workspace.pending &&
      workspace.phase === 'idle' &&
      workspace.completed < (plan?.steps.length ?? 0),
  )
  let phoneReason = $derived(
    !workspace.physical
      ? 'Enable Move device in Settings.'
      : workspace.preview
        ? 'Replace or cancel this alternative first.'
        : workspace.exploring
          ? 'Close alternatives before moving.'
          : device?.readiness || '',
  )
  let message = $derived(
    workspace.error ??
      (!workspace.snapshot
        ? 'Opening your workspace…'
        : searching
          ? `${workspace.snapshot.job?.kind === 'explore' ? 'Finding alternatives' : 'Searching'}… ${(workspace.snapshot.job?.visited ?? 0).toLocaleString('en-US')} nodes`
          : workspace.snapshot.phase === 'no_solution'
            ? 'No legal move. Review the board or pieces.'
            : (device?.error ??
              (device?.serial
                ? device.message
                : 'Connect your phone and Scan, or enter the board and pieces yourself.'))),
  )
  let activityError = $derived(workspace.error ?? device?.error ?? null)
  let activityTitle = $derived(
    activityError
      ? 'Needs attention'
      : device?.state === 'awaiting_confirmation'
        ? 'Waiting for your confirmation'
        : deviceBusy
          ? 'Phone activity'
          : searching
            ? 'Calculating'
            : workspace.running
              ? 'Local preview'
              : workspace.previewActive
                ? workspace.completed === plan?.steps.length
                  ? 'Preview complete'
                  : 'Preview paused'
                : workspace.exploring
                  ? 'Exploring alternatives'
                  : device?.state === 'cycle_completed'
                    ? 'Cycle completed'
                    : device?.state === 'needs_correction'
                      ? 'Input needs confirmation'
                      : showPlan
                        ? 'Ready'
                        : 'Workspace status',
  )
  let activityMessage = $derived(
    activityError ??
      (deviceBusy
        ? device?.message || 'Working with your phone…'
        : searching
          ? message
          : workspace.running
            ? `Previewing step ${Math.min(workspace.completed + 1, plan?.steps.length ?? 0)} of ${plan?.steps.length ?? 0}. Your phone stays still.`
            : workspace.previewActive
              ? `${workspace.completed} of ${plan?.steps.length ?? 0} moves previewed locally. Your phone stays still.`
              : workspace.exploring
                ? 'Choose an order or position, or preview a solution on the board.'
                : !workspace.snapshot
                  ? 'Opening your workspace…'
                  : workspace.snapshot.phase === 'no_solution'
                    ? 'No legal move. Check the board or pieces.'
                    : workspace.pending
                      ? 'Updating the workspace…'
                      : device?.state === 'cycle_completed' || device?.state === 'needs_correction'
                        ? device.message
                        : showPlan
                          ? `${plan?.steps.length ?? 0} moves planned. ${workspace.actualCompleted} played on phone.`
                          : message),
  )
  onMount(() => {
    void workspace.refresh()
    return () => workspace.dispose()
  })
  async function editSlot(slot: number) {
    editingAnchor = null
    editingSlot = slot
  }
  async function editRows(rows: WorkspaceView['board']) {
    try {
      await workspace.updateBoard(rows)
    } catch {}
  }
  async function savePiece(piece: PieceView | null) {
    if (editingSlot !== null) await workspace.updatePiece(editingSlot, piece, editingAnchor)
  }
  function toggleBoardCell(row: number, column: number) {
    const rows = [...(workspace.snapshot?.currentBoard ?? [])] as WorkspaceView['board']
    const line = rows[row]
    if (!line) return
    rows[row] = line.slice(0, column) + (line[column] === '#' ? '.' : '#') + line.slice(column + 1)
    void editRows(rows)
  }
</script>

<div
  class={`dashboard unified-dashboard${showPlan ? '' : ' manual-dashboard'}`}
  style:--board-motion-duration={`${BOARD_MOTION_MS}ms`}
>
  <header class="app-header">
    <Brand />
  </header>
  <main class="workspace">
    {#if showPlan && plan && workspace.displayBoard}
      <BoardDisplay
        rows={workspace.displayBoard}
        {plan}
        pieces={workspace.pieces}
        selected={workspace.selectedStep}
        completed={workspace.completed}
        phase={workspace.phase}
        exploring={workspace.exploring}
        alternative={workspace.hoverPlan}
        alternativeCount={workspace.hoverCount}
        onedit={toggleBoardCell}
        editDisabled={workspace.pending || deviceBusy || workspace.running}
      />
    {:else}
      <BoardInput
        snapshot={workspace.snapshot}
        unknownBoard={!!device?.capture && device.capture.board === null}
        disabled={workspace.pending || deviceBusy}
        onreplace={editRows}
      />
    {/if}
    <aside
      class={`inspector${showPlan ? '' : ' manual-inspector'}${workspace.exploring ? ' is-exploring' : ''}`}
      aria-label="Pieces and plan"
    >
      <div class="inspector-content">
        <PieceTray
          allSlots
          pieces={workspace.pieces}
          {consumed}
          lockedSlots={actualConsumed}
          unknownSlots={!workspace.snapshot?.inputKnown
            ? (device?.capture?.unknownSlots ?? [])
            : []}
          onaccept={(slots) => {
            void workspace.acceptSlots(slots).catch(() => {})
          }}
          issues={device?.capture?.slotIssues ?? []}
          crops={device?.capture?.slotCrops ?? []}
          active={workspace.phase === 'placing' || workspace.phase === 'breaking'
            ? (plan?.steps[workspace.completed]?.slot ?? null)
            : null}
          onedit={editSlot}
          disabled={workspace.pending || !workspace.snapshot || deviceBusy}
        />
        {#if device?.capture?.board === null && !workspace.snapshot?.inputKnown}
          <div class="input-warning" role="status">
            <span>Board read is unclear. Edit the cells or accept the displayed board.</span><button
              class="button"
              disabled={workspace.pending || deviceBusy}
              onclick={() => {
                void workspace.acceptBoard().catch(() => {})
              }}>Accept board</button
            >
          </div>
        {/if}
        {#if showPlan && plan}
          <div id="plan-panel">
            {#if workspace.exploring}<div class="explorer-host">
                <Explorer
                  data={workspace.explorer}
                  pieces={remainingPieces}
                  completed={workspace.completed}
                  order={workspace.order}
                  positions={workspace.positions}
                  pageSize={workspace.pageSize}
                  loading={searching || !workspace.explorer}
                  onfilter={(order, positions) => void workspace.filter(order, positions)}
                  onpage={(page) => void workspace.page(page)}
                  onsize={(size) => void workspace.setPageSize(size)}
                  onchoose={(id) => void workspace.previewCandidate(id)}
                  onclose={workspace.closeExplorer}
                  onhover={(id) => void workspace.hoverCandidate(id)}
                  onpositionpreview={(index, position) =>
                    void workspace.hoverPosition(index, position)}
                />
              </div>
            {:else}<PlanInspector
                {plan}
                pieces={workspace.pieces}
                completed={workspace.completed}
                selected={workspace.selectedStep}
                running={workspace.running}
                phase={workspace.phase}
                candidateId={workspace.candidateId}
                previewId={workspace.previewId}
                onreplace={() => void workspace.replacePlan()}
                oncancel={workspace.cancelPreview}
                onrestore={() => void workspace.restore()}
                onexplore={() => void workspace.explore()}
                onplay={workspace.playThrough}
                onphone={(index) => void workspace.execute(index)}
                phoneDisabled={!!phoneReason || deviceBusy || workspace.displayBehind}
                physical={workspace.physical}
                actualCompleted={workspace.actualCompleted}
                onselect={workspace.selectStep}
                busy={workspace.pending || searching || deviceBusy || workspace.displayBehind}
              />{/if}
          </div>
        {/if}
      </div>
      <div class="controller-stack">
        <section
          class="activity-panel"
          class:has-error={!!activityError}
          aria-label="Current activity"
          aria-live="polite"
          aria-atomic="true"
        >
          <div class="activity-heading"><Icon name="info" /><strong>{activityTitle}</strong></div>
          <p role={activityError ? 'alert' : undefined}>{activityMessage}</p>
          {#if device?.state === 'awaiting_confirmation'}
            <div class="activity-actions">
              <button
                class="button button-primary"
                disabled={workspace.pending}
                onclick={() => void workspace.acceptWarning()}>Accept and continue</button
              ><button class="button" onclick={() => void workspace.stop()}>Stop</button>
            </div>
          {/if}
        </section>
        <section class="console-panel" aria-label="Controls">
          <div class="control-status" class:is-connected={!!device?.serial} role="status">
            <span class="control-status-dot" aria-hidden="true"></span>{device?.serial
              ? 'Connected'
              : 'Disconnected'}
            {#if device?.serial}<button
                class="move-device-shortcut"
                role="switch"
                aria-checked={workspace.physical}
                aria-label="Move device"
                disabled={!workspace.preferences ||
                  workspace.pending ||
                  deviceBusy ||
                  savingMoveDevice}
                onclick={() => void toggleMoveDevice()}
              >
                <span>Move device</span>
                <span class="move-device-track" aria-hidden="true"></span>
              </button>{/if}
          </div>
          <div
            class="console-controls unified-primary-controls"
            role="group"
            aria-label="Workspace actions"
          >
            <IconControl
              icon="scan"
              label="Scan"
              hint={device?.serial
                ? 'Read the board and pieces from your phone.'
                : 'Connect your phone to scan its board and pieces.'}
              disabled={workspace.pending || !workspace.snapshot || deviceBusy || workspace.running}
              onpress={() => {
                if (!device?.serial) settings.show('connection')
                else void workspace.scan()
              }}
            />
            <IconControl
              icon={searching ? 'stop' : 'search'}
              label={searching ? 'Cancel search' : 'Search'}
              hint={searching
                ? 'Stop calculating this plan.'
                : !workspace.snapshot?.inputKnown
                  ? 'Scan or correct the board and pieces first.'
                  : 'Calculate a plan for the current input.'}
              disabled={workspace.pending ||
                !workspace.snapshot ||
                deviceBusy ||
                workspace.running ||
                (!searching && !workspace.snapshot.inputKnown)}
              onpress={() => (searching ? void workspace.cancel() : void workspace.search())}
            />
            <IconControl
              primary
              icon={deviceBusy ? 'stop' : workspace.running ? 'pause' : 'play'}
              label={deviceBusy
                ? 'Stop'
                : workspace.physical
                  ? 'Play on phone'
                  : workspace.running
                    ? 'Pause preview'
                    : 'Preview'}
              hint={deviceBusy
                ? 'Finish the active movement and stop.'
                : workspace.physical
                  ? phoneReason || 'Execute the remaining moves on your phone.'
                  : workspace.running
                    ? 'Pause the local preview.'
                    : !showPlan
                      ? 'Calculate a plan first to preview its moves.'
                      : 'Preview the plan locally without moving your phone.'}
              disabled={workspace.pending ||
                !workspace.snapshot ||
                (!deviceBusy &&
                  (workspace.physical
                    ? !!phoneReason ||
                      !showPlan ||
                      workspace.displayBehind ||
                      workspace.running ||
                      device?.cycleDone
                    : !workspace.running && !canPreview))}
              onpress={() => {
                if (deviceBusy) void workspace.stop()
                else if (workspace.physical) void workspace.execute()
                else if (workspace.running) workspace.pause()
                else workspace.playThrough((plan?.steps.length ?? 1) - 1)
              }}
            />
            {#if workspace.previewActive || workspace.completed > 0}<IconControl
                icon={workspace.physical || !workspace.previewActive ? 'lock' : 'reset'}
                label={workspace.physical || !workspace.previewActive
                  ? 'Phone moves locked'
                  : workspace.actualCompleted > 0
                    ? 'Reset remaining preview'
                    : 'Reset preview'}
                hint={workspace.physical || !workspace.previewActive
                  ? 'Moves already played on your phone cannot be undone or replayed. Switching Move device off does not unlock them.'
                  : workspace.actualCompleted > 0
                    ? 'Clear only the local preview of remaining moves. Completed phone moves stay locked.'
                    : 'Clear the local preview and return to the starting board. No phone movement is undone.'}
                disabled={workspace.pending ||
                  deviceBusy ||
                  workspace.physical ||
                  !workspace.previewActive}
                onpress={workspace.reset}
              />{/if}
            {#if nextReady}<IconControl
                icon="arrow"
                label={device?.cycleDone ? 'Next cycle' : 'Continue simulation'}
                hint={device?.cycleDone
                  ? 'Read the next phone tray without moving.'
                  : 'Keep the simulated board and clear the tray for the next cycle.'}
                disabled={workspace.pending ||
                  (device?.cycleDone && (!device.serial || device.needsScan))}
                onpress={() => void workspace.nextCycle()}
              />{/if}
            {#if deviceBusy}<IconControl
                icon="force-stop"
                label="Force stop"
                danger
                hint="Interrupt immediately. Scan to recover before moving again."
                disabled={workspace.pending}
                onpress={() => void workspace.stop(true)}
              />{/if}
            <IconControl
              icon="settings"
              label="Settings"
              hint="Connection, Move device and other preferences."
              popup="dialog"
              alignEnd
              disabled={workspace.pending || !workspace.snapshot}
              onpress={() => settings.show()}
            />
          </div>
        </section>
      </div>
    </aside>
  </main>
  <SettingsDialog
    bind:this={settings}
    disabled={workspace.pending || !workspace.snapshot}
    {device}
    preferences={workspace.preferences}
    onpreferences={workspace.savePreferences}
    ondevice={workspace.deviceAction}
    weights={workspace.snapshot?.weights}
    onweights={workspace.weights}
  />
  {#if editingSlot !== null}<PieceEditor
      slot={editingSlot}
      initial={workspace.snapshot?.pieces.find((p) => p.slot === editingSlot)}
      capture={device?.capture}
      anchor={editingAnchor ?? undefined}
      onanchor={(anchor) => {
        editingAnchor = anchor
      }}
      onsave={savePiece}
      onclose={() => {
        editingSlot = null
      }}
    />{/if}
</div>
