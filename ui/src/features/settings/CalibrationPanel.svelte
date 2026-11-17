<script lang="ts">
  import type {
    DeviceAction,
    DeviceView,
    Profile,
    CalibrationView,
  } from '../../generated/contracts'
  import { calibrationApi, validHardwareProfile } from '../../lib/native/calibration'
  import { workspaceApi } from '../../lib/native/workspace'
  import TrayGridEditor from './TrayGridEditor.svelte'
  import PieceShape from '../../components/board/PieceShape.svelte'
  let {
    device,
    serial,
    ondevice,
    onbusy,
    onsaved,
    onconnection,
    onclose,
  }: {
    device: DeviceView | undefined
    serial: string
    ondevice: (action: DeviceAction) => Promise<void>
    onbusy: (busy: boolean) => void
    onclose: () => void
    onconnection: () => void
    onsaved: () => Promise<void>
  } = $props()
  const steps = ['Phone', 'Board', 'Pieces', 'Test', 'Save']
  const titles = [
    'Prepare your phone',
    'Does the board line up?',
    'Do the pieces line up?',
    'Check a test move',
    'Save calibration',
  ]
  const hints = [
    'Open the game in portrait. Use the default game theme for best results.',
    'The grid lines should follow the board edges.',
    'Each grid should follow the piece underneath it.',
    'We will place up to 3 pieces and check where they land.',
    'Your phone will reconnect automatically.',
  ]
  let panel = $state<HTMLDivElement>(null!)
  let saved = $state(false)
  let boardEstimated = $state(false)
  let verifiedKey = $state('')
  let phoneChoice = $state('')
  let activeSerial = $derived(phoneChoice || serial)
  let progress = $state('')
  let step = $state(0)
  let profile = $state<Profile | null>(null)
  let image = $state('')
  let captureId = $state<number | null>(null)
  let capturedSerial = $state('')
  let result = $state<CalibrationView | null>(null)
  let pending = $state(false)
  let message = $state('')
  let error = $state('')
  let name = $state('my-phone')
  let slot = $state(0)
  let row = $state(0)
  let column = $state(0)
  let installHelper = $state(false)
  let zoomBoard = $state(true)
  let boardViewport = $state<[number, number, number, number]>([0, 0, 1, 1])
  let viewport = $derived<[number, number, number, number]>(
    profile && step === 1 && zoomBoard
      ? boardViewport
      : [0, 0, profile?.screen.width ?? 1, profile?.screen.height ?? 1],
  )
  let source = $state<HTMLDivElement>(null!)
  let drag: {
    start: [number, number]
    rect: [number, number, number, number]
    mode: 'draw' | 'move' | 'resize'
  } | null = null
  let area = $derived(
    profile ? (step === 1 ? profile.screen.board : profile.screen.slots[slot]!) : null,
  )
  let ready = $derived(!!profile && validHardwareProfile(profile))
  let movementVerified = $derived(
    !!profile &&
      verifiedKey ===
        JSON.stringify({ serial: activeSerial, screen: profile.screen, touch: profile.touch }),
  )
  let saveReady = $derived(
    ready && !boardEstimated && /^[A-Za-z0-9_-]{1,80}$/.test(name) && name !== 'default',
  )
  let nameError = $derived(
    !name
      ? 'Enter a profile name.'
      : name === 'default'
        ? 'Choose a different profile name.'
        : !/^[A-Za-z0-9_-]{1,80}$/.test(name)
          ? 'Use letters, numbers, hyphens or underscores.'
          : '',
  )
  let blocked = $derived(pending || !!device?.busy)
  let previewCurrent = $derived(
    !!result &&
      !!profile &&
      JSON.stringify(result.profile.screen) === JSON.stringify(profile.screen),
  )
  const panelId = $props.id()
  const testHintId = `${panelId}-test-drag-hint`
  let readingBlockReason = $derived.by(() => {
    if (boardEstimated) return 'Check the full board edges before testing.'
    if (capturedSerial !== activeSerial)
      return 'The selected phone changed. Click Capture again, then Preview recognition.'
    if (!captureId) return 'Click Capture again, then Preview recognition before running a test.'
    if (!ready)
      return 'Some calibration values are invalid. Check screen areas, cell size, gain, lift, and advanced timing.'
    if (!result || !previewCurrent)
      return 'Click Preview recognition to check the current screen calibration.'
    if (result.boardUnclear)
      return 'The board reading is unclear. Adjust the board grid, then click Preview recognition.'
    if (result.unknownSlots.length)
      return `Tray slots ${result.unknownSlots.map((n) => n + 1).join(', ')} are unclear. Adjust their grids, then click Preview recognition.`
    return ''
  })
  let automaticTarget = $derived.by(() => {
    if (readingBlockReason || !result?.board) return null
    let best: { slot: number; row: number; column: number; score: number } | null = null
    for (const piece of result.pieces)
      for (let r = 0; r < 8; r++)
        for (let c = 0; c < 8; c++) {
          if (
            piece.cells.some(
              ([dr, dc]) => r + dr > 7 || c + dc > 7 || result!.board![r + dr]?.[c + dc] !== '.',
            )
          )
            continue
          const rows = result.board.map((line) => [...line])
          for (const [dr, dc] of piece.cells) rows[r + dr]![c + dc] = '#'
          const clears =
            rows.some((line) => line.every((cell) => cell === '#')) ||
            [0, 1, 2, 3, 4, 5, 6, 7].some((col) => rows.every((line) => line[col] === '#'))
          const centre =
            ((profile!.screen.slots[piece.slot]![0] +
              profile!.screen.slots[piece.slot]![2] / 2 -
              profile!.screen.board[0]) *
              8) /
            profile!.screen.board[2]
          const score = (clears ? -100 : 0) + Math.abs(c + 0.5 - centre)
          if (!best || score > best.score)
            best = { slot: piece.slot, row: r + 1, column: c + 1, score }
        }
    return best
  })
  let automaticBlockReason = $derived(
    (pending || device?.busy ? 'Wait for the current phone operation to finish.' : '') ||
      readingBlockReason ||
      (automaticTarget
        ? ''
        : 'No recognized shape fits the empty cells. Open a playable game and capture again.'),
  )
  let testBlockReason = $derived.by(() => {
    if (pending || device?.busy) return 'Wait for the current phone operation to finish.'
    if (readingBlockReason) return readingBlockReason
    const piece = result!.pieces.find((p) => p.slot === slot)
    if (!piece) return `Slot ${slot + 1} is empty. Choose a slot containing a detected shape.`
    if (
      !Number.isInteger(row) ||
      !Number.isInteger(column) ||
      row < 1 ||
      row > 8 ||
      column < 1 ||
      column > 8
    )
      return 'Fill Target row and Target column with numbers from 1 to 8.'
    if (
      !result!.board ||
      piece.cells.some(
        ([r, c]) =>
          row - 1 + r > 7 ||
          column - 1 + c > 7 ||
          result!.board![row - 1 + r]?.[column - 1 + c] !== '.',
      )
    )
      return 'The shape does not fit at that target. Choose empty cells with room for the entire shape.'
    return ''
  })
  async function run(task: () => Promise<void>) {
    if (pending) return
    pending = true
    onbusy(true)
    message = ''
    error = ''
    try {
      await task()
    } catch (e) {
      error = e instanceof Error ? e.message : 'Calibration failed.'
    } finally {
      pending = false
      onbusy(false)
    }
  }
  async function capture(automatic = true) {
    await run(async () => {
      progress = 'Capturing the game and locating board and tray grids… Please wait.'
      saved = false
      if (automatic) verifiedKey = ''
      result = null
      captureId = null
      const next = await calibrationApi.request({ type: 'capture', serial: activeSerial })
      if (
        !profile ||
        capturedSerial !== activeSerial ||
        profile.screen.width !== next.profile.screen.width ||
        profile.screen.height !== next.profile.screen.height
      )
        profile = next.profile
      capturedSerial = activeSerial
      image = next.image ?? ''
      captureId = next.captureId
      const detected = await calibrationApi.request({
        type: automatic ? 'autoLayout' : 'preview',
        captureId: next.captureId!,
        profile: $state.snapshot(profile!),
      })
      profile = detected.profile
      if (automatic) boardEstimated = detected.layoutDetected === false
      result = detected
      chooseTarget()
      if (automatic) openBoardEditor()
      message =
        detected.message ||
        'Screenshot refreshed. Recognition and automatic target are ready to check.'
    })
  }
  async function readProfile() {
    await run(async () => {
      const next = await calibrationApi.request({ type: 'read' })
      saved = false
      verifiedKey = ''
      boardEstimated = false
      profile = next.profile
      image = ''
      captureId = null
      result = null
      name = device?.profileId === 'default' ? 'my-phone' : (device?.profileId ?? 'my-phone')
      message = 'Current hardware profile loaded. Capture your phone to calibrate visually.'
    })
  }
  async function preview() {
    if (!profile || !captureId) return
    const draft = $state.snapshot(profile)
    await run(async () => {
      progress = 'Checking the alignment…'
      result = await calibrationApi.request({
        type: 'preview',
        profile: draft,
        captureId: captureId!,
      })
      chooseTarget()
      message = 'Check the detected board and shapes below before testing movement.'
    })
  }
  async function awaitDevice(action: DeviceAction) {
    await ondevice(action)
    const deadline = Date.now() + 180_000
    for (;;) {
      const snapshot = await workspaceApi.snapshot()
      if (!snapshot.device.busy) {
        if (snapshot.device.error) throw new Error(snapshot.device.error)
        return snapshot
      }
      if (Date.now() >= deadline)
        throw new Error('Phone operation is taking too long. Stop and check the phone connection.')
      await new Promise((resolve) => setTimeout(resolve, 100))
    }
  }
  async function exportProfile(save: boolean) {
    if (!profile) return
    const draft = $state.snapshot(profile)
    await run(async () => {
      const exported = await calibrationApi.request({ type: 'export', profile: draft })
      if (save) {
        const beforeSave = await workspaceApi.snapshot()
        if (beforeSave.device.serial) await awaitDevice({ type: 'disconnect' })
        await ondevice({ type: 'profile', id: name, toml: exported.toml })
        const next = await workspaceApi.snapshot()
        if (next.device.profileId !== name || next.device.error)
          throw new Error(next.device.error ?? 'Profile could not be saved.')
        await onsaved()
        if (!capturedSerial && !activeSerial) {
          saved = true
          message = 'Hardware profile saved and selected.'
          return
        }
        progress = 'Profile saved. Reconnecting your phone…'
        await awaitDevice({
          type: 'connect',
          serial: capturedSerial || activeSerial,
          installHelper: true,
        })
        const connected = await workspaceApi.snapshot()
        if (!connected.device.serial || connected.device.error)
          throw new Error(
            `Profile saved. ${connected.device.error ?? 'Could not reconnect the phone.'}`,
          )
        progress = 'Reading the calibrated game…'
        await awaitDevice({ type: 'scan', workspaceRevision: connected.revision })
        saved = true
        const scanned = await workspaceApi.snapshot()
        message = scanned.device.error
          ? `Calibration saved and phone reconnected. ${scanned.device.error}`
          : 'Calibration saved. Phone connected and game scanned; ready to play.'
      } else {
        const url = URL.createObjectURL(new Blob([exported.toml], { type: 'application/toml' }))
        const anchor = document.createElement('a')
        anchor.href = url
        anchor.download = `${name || 'phone-calibration'}.toml`
        anchor.click()
        setTimeout(() => URL.revokeObjectURL(url), 1000)
        message = 'Exported hardware calibration only. App preferences and scoring are excluded.'
      }
    })
  }
  function chooseTarget() {
    if (!automaticTarget) return false
    slot = automaticTarget.slot
    row = automaticTarget.row
    column = automaticTarget.column
    return true
  }
  async function nextStep() {
    if ((step === 1 || step === 2) && captureId) {
      await preview()
      if (error) return
    }
    if (step === 1) boardEstimated = false
    goToStep(step + 1)
  }
  async function testDrag(automatic = true) {
    if (!profile || !captureId || (automatic ? automaticBlockReason : testBlockReason)) return
    await run(async () => {
      for (let attempt = 0; attempt < (automatic ? 3 : 1); attempt++) {
        if (automatic && !chooseTarget())
          throw new Error(
            'No readable shape fits the empty board cells. Open a playable game and capture again.',
          )
        progress = `Checking movement on phone (${attempt + 1} of ${automatic ? 3 : 1})… Please wait.`
        const id = captureId!
        const draft = $state.snapshot(profile!)
        captureId = null
        result = null
        const next = await calibrationApi.request({
          type: 'testDrag',
          captureId: id,
          profile: draft,
          slot,
          row: row - 1,
          column: column - 1,
          installHelper: automatic || installHelper,
        })
        profile = next.profile
        captureId = next.captureId
        image = next.image ?? image
        result = next
        message = next.message
        if (next.movement === 'verified')
          verifiedKey = JSON.stringify({
            serial: capturedSerial,
            screen: next.profile.screen,
            touch: next.profile.touch,
          })
        if (next.movement === 'verified' || !automatic) return
      }
      message =
        'Movement was adjusted, but is not verified yet. Check the phone and run another check or fine-tune the advanced settings.'
    })
  }
  function fitBoard() {
    if (!profile) return
    const [x, y, w, h] = profile.screen.board
    const pad = Math.max(w, h) * 0.12
    const left = Math.max(0, x - pad),
      top = Math.max(0, y - pad)
    boardViewport = [
      left,
      top,
      Math.max(1, Math.min(profile.screen.width, x + w + pad) - left),
      Math.max(1, Math.min(profile.screen.height, y + h + pad) - top),
    ]
  }
  function showAdjustments() {
    const options = panel.querySelector<HTMLDetailsElement>(
      step === 1 ? '[data-adjust-board]' : '[data-adjust-pieces]',
    )
    if (!options) return
    options.open = true
    options.scrollIntoView({ block: 'nearest' })
  }
  function goToStep(next: number) {
    if (next === 1) openBoardEditor()
    else step = next
    panel?.closest('.settings-body')?.scrollTo({ top: 0 })
  }
  function openBoardEditor() {
    step = 1
    zoomBoard = true
    fitBoard()
    panel?.closest('.settings-body')?.scrollTo({ top: 0 })
  }
  async function finishBoardDrag() {
    if (!drag) return
    drag = null
    await preview()
  }
  function nudgeBoard(event: KeyboardEvent) {
    if (
      blocked ||
      !profile ||
      !['ArrowLeft', 'ArrowRight', 'ArrowUp', 'ArrowDown'].includes(event.key)
    )
      return
    event.preventDefault()
    const amount = event.shiftKey ? 10 : 1
    const [x, y, w, h] = profile.screen.board
    setArea([
      Math.max(
        0,
        Math.min(
          profile.screen.width - w,
          x + (event.key === 'ArrowLeft' ? -amount : event.key === 'ArrowRight' ? amount : 0),
        ),
      ),
      Math.max(
        0,
        Math.min(
          profile.screen.height - h,
          y + (event.key === 'ArrowUp' ? -amount : event.key === 'ArrowDown' ? amount : 0),
        ),
      ),
      w,
      h,
    ])
  }
  function setArea(rect: [number, number, number, number]) {
    if (!profile) return
    if (step === 1) {
      profile.screen.board = rect
      boardEstimated = false
    } else profile.screen.slots[slot] = rect
  }
  function point(event: PointerEvent): [number, number] {
    const bounds = source.getBoundingClientRect()
    return [
      Math.round(
        Math.max(
          0,
          Math.min(
            profile!.screen.width - 1,
            viewport[0] +
              Math.max(0, Math.min(1, (event.clientX - bounds.left) / bounds.width)) * viewport[2],
          ),
        ),
      ),
      Math.round(
        Math.max(
          0,
          Math.min(
            profile!.screen.height - 1,
            viewport[1] +
              Math.max(0, Math.min(1, (event.clientY - bounds.top) / bounds.height)) * viewport[3],
          ),
        ),
      ),
    ]
  }

  function startDrag(event: PointerEvent, mode: 'draw' | 'move' | 'resize') {
    if (!profile || !area || blocked || step !== 1 || event.button !== 0) return
    event.preventDefault()
    event.stopPropagation()
    source.setPointerCapture(event.pointerId)
    drag = { start: point(event), rect: [...area], mode }
  }
  function moveDrag(event: PointerEvent) {
    if (!drag || !profile) return
    const [x, y] = point(event),
      [sx, sy] = drag.start,
      [rx, ry, rw, rh] = drag.rect
    if (drag.mode === 'draw')
      setArea([
        Math.min(sx, x),
        Math.min(sy, y),
        Math.max(1, Math.abs(x - sx)),
        Math.max(1, Math.abs(y - sy)),
      ])
    else if (drag.mode === 'move')
      setArea([
        Math.max(0, Math.min(profile.screen.width - rw, rx + x - sx)),
        Math.max(0, Math.min(profile.screen.height - rh, ry + y - sy)),
        rw,
        rh,
      ])
    else
      setArea([
        rx,
        ry,
        Math.max(1, Math.min(profile.screen.width - rx, rw + x - sx)),
        Math.max(1, Math.min(profile.screen.height - ry, rh + y - sy)),
      ])
  }
</script>

<div class="calibration-panel calibration-wizard" bind:this={panel}>
  <ol class="calibration-steps" aria-label="Calibration steps">
    {#each steps as label, index}<li>
        <button
          type="button"
          aria-label={`${index + 1}. ${label}`}
          aria-current={step === index ? 'step' : undefined}
          disabled={blocked || (index > 0 && !profile) || saved}
          onclick={() => goToStep(index)}><span>{index + 1}</span>{label}</button
        >
      </li>{/each}
  </ol>
  <header class="calibration-heading">
    <h3>{saved ? 'You’re ready' : titles[step]}</h3>
    <p>{saved ? 'Calibration saved. You can return to your game.' : hints[step]}</p>
  </header>
  {#if pending}<div class="calibration-working" role="status">
      <span class="calibration-spinner"></span><span>{progress || 'Please wait…'}</span>
      <button class="calibration-link" onclick={() => void ondevice({ type: 'forceStop' })}
        >Stop</button
      >
    </div>{/if}
  {#if error}<p class="calibration-error" role="alert">{error}</p>{/if}
  <div class="calibration-step-content">
    {#if saved}
      <div class="calibration-complete" aria-label="Calibration complete">
        <span aria-hidden="true">✓</span><strong>{name}</strong>
        <button
          class="calibration-link"
          onclick={() => {
            saved = false
            verifiedKey = ''
            goToStep(0)
          }}>Calibrate again</button
        >
      </div>
    {:else if step === 0}
      <div class="calibration-preparation">
        <svg class="calibration-phone-icon" viewBox="0 0 40 64" fill="none" aria-hidden="true"
          ><rect
            x="5"
            y="1"
            width="30"
            height="62"
            rx="5"
            stroke="currentColor"
            stroke-width="1.5"
          /><path
            d="M16 6h8M17 57h6"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
          /></svg
        >
        <p>Keep the game open on your phone.<br />We’ll find the board and pieces for you.</p>
      </div>
      {#if activeSerial}
        <div class="calibration-phone-status">
          <span class="calibration-status-dot"></span><strong
            >{device?.devices.find((d) => d.serial === activeSerial)?.model ||
              'Phone selected'}</strong
          >
        </div>
      {:else}
        <p class="calibration-help">
          Connect by USB and allow USB debugging, then find your phone.
        </p>
        <button
          class="button"
          disabled={blocked}
          onclick={() => void ondevice({ type: 'discover' })}>Find phone</button
        >
      {/if}
      <details class="calibration-options">
        <summary
          >{activeSerial
            ? 'Change phone or connection'
            : 'Choose a phone / wireless setup'}</summary
        >
        {#if device?.devices.length}<label class="settings-field"
            ><span>Phone</span><select
              class="settings-select"
              aria-label="Phone to calibrate"
              bind:value={phoneChoice}
              disabled={blocked}
            >
              <option value="">{serial || 'Choose a phone'}</option
              >{#each device.devices as phone}<option
                  value={phone.serial}
                  disabled={phone.state !== 'device'}
                  >{phone.model || phone.serial} · {phone.state}</option
                >{/each}
            </select></label
          >{/if}
        <div class="calibration-actions">
          <button
            class="button"
            disabled={blocked}
            onclick={() => void ondevice({ type: 'discover' })}>Find phone</button
          ><button class="button" disabled={blocked} onclick={onconnection}>Phone connection</button
          >
        </div>
      </details>
      <details class="calibration-options">
        <summary>Use a saved profile</summary><button
          class="button"
          disabled={blocked}
          onclick={() => void readProfile()}>Load current profile</button
        >
        {#if profile}<p class="calibration-resolution">
            {profile.screen.width} × {profile.screen.height} px
          </p>{/if}
      </details>
    {:else if profile}
      {#if step === 1}
        {#if image && profile && step === 1}
          <div
            class="calibration-image"
            class:calibration-board-zoom={step === 1 && zoomBoard}
            style={step === 1 && zoomBoard
              ? `aspect-ratio:${viewport[2]}/${viewport[3]}`
              : undefined}
            bind:this={source}
            role="group"
            aria-label="Screenshot calibration canvas"
            onpointerdown={(event) => startDrag(event, 'draw')}
            onpointermove={moveDrag}
            onpointerup={() => void finishBoardDrag()}
            onpointercancel={() => (drag = null)}
          >
            <img
              src={image}
              alt="Current phone screenshot for calibration"
              draggable="false"
              style={step === 1 && zoomBoard
                ? `position:absolute;max-width:none;width:${(profile.screen.width / viewport[2]!) * 100}%;left:${(-viewport[0]! / viewport[2]!) * 100}%;top:${(-viewport[1]! / viewport[3]!) * 100}%;`
                : undefined}
            />
            {#each step === 1 ? [profile.screen.board] : [] as rect, index}
              <div
                class:calibration-grid={index === 0}
                class:active={index === (step === 1 ? 0 : slot + 1)}
                class="calibration-overlay"
                style={`left:${((rect[0] - viewport[0]!) / viewport[2]!) * 100}%;top:${((rect[1] - viewport[1]!) / viewport[3]!) * 100}%;width:${(rect[2] / viewport[2]!) * 100}%;height:${(rect[3] / viewport[3]!) * 100}%;background-size:${index === 0 ? 12.5 : (profile.screen.preview_pitch / rect[2]) * 100}% ${index === 0 ? 12.5 : (profile.screen.preview_pitch / rect[3]) * 100}%;`}
                role="button"
                tabindex={blocked ? -1 : 0}
                onkeydown={nudgeBoard}
                onkeyup={(event) => {
                  if (event.key.startsWith('Arrow')) void preview()
                }}
                aria-label={index === 0 ? 'Board area' : `Slot ${index} area`}
                onpointerdown={(event) => {
                  if (index === (step === 1 ? 0 : slot + 1)) startDrag(event, 'move')
                }}
              >
                <span>{index === 0 ? 'Board 8×8' : `Slot ${index}`}</span>
              </div>
              {#if [1, 2].includes(step) && index === (step === 1 ? 0 : slot + 1)}<button
                  class="calibration-resize"
                  style={`left:${((rect[0] + rect[2] - viewport[0]) / viewport[2]) * 100}%;top:${((rect[1] + rect[3] - viewport[1]) / viewport[3]) * 100}%;right:auto;bottom:auto;transform:translate(-50%,-50%);`}
                  aria-label="Resize selected area"
                  onpointerdown={(event) => startDrag(event, 'resize')}
                  type="button"
                ></button>{/if}
            {/each}
          </div>
        {/if}

        <p class="calibration-gesture">Drag to move · corner to resize</p>
        {#if boardEstimated}<p class="calibration-inline-status" role="status">
            Board edges weren’t clear. Align all 8 rows and 8 columns before continuing.
          </p>{/if}
        {#if result?.boardUnclear && previewCurrent}<p class="calibration-inline-status">
            Some cells need a clearer alignment. Adjust the grid, then check again.
          </p>{/if}
        {#if !previewCurrent && !pending}<button
            class="button"
            disabled={blocked || !captureId || !ready}
            onclick={() => void preview()}>Re-read board</button
          >{/if}
        <details class="calibration-options" data-adjust-board>
          <summary>Adjust board</summary>
          <p class="calibration-help">Arrow keys move 1 px. Hold Shift for 10 px.</p>
          {#if area}<div class="calibration-coordinates">
              {#each ['X', 'Y', 'Width', 'Height'] as label, index}<label class="settings-field"
                  ><span>{label}</span><input
                    class="settings-input"
                    type="number"
                    min={index > 1 ? 1 : 0}
                    step="1"
                    value={area[index]}
                    aria-label={`Area ${label}`}
                    disabled={pending}
                    oninput={(event) => {
                      const next: [number, number, number, number] = [...area!]
                      next[index] = Number(event.currentTarget.value)
                      setArea(next)
                    }}
                  /></label
                >{/each}
            </div>{/if}
          <div class="calibration-actions">
            <button
              class="button"
              disabled={blocked}
              onclick={() => {
                zoomBoard = true
                fitBoard()
              }}>Zoom board</button
            ><button class="button" disabled={blocked} onclick={() => (zoomBoard = false)}
              >Full screenshot</button
            >
            <button
              class="button"
              disabled={blocked || !captureId || !ready}
              onclick={() => void preview()}>Re-read board</button
            >
          </div>
        </details>
      {:else if step === 2}
        <TrayGridEditor
          {profile}
          {image}
          {captureId}
          disabled={blocked}
          onchange={(value) => (profile = value)}
        />
      {:else if step === 3}
        <div class="calibration-test-state" class:is-verified={movementVerified} role="status">
          <span aria-hidden="true">{movementVerified ? '✓' : '↗'}</span><strong
            >{movementVerified
              ? 'Movement looks good'
              : pending
                ? 'Checking…'
                : automaticBlockReason
                  ? 'Check the screen'
                  : 'Ready for a test'}</strong
          >
          <p>
            {movementVerified
              ? 'The piece landed in the expected position.'
              : !pending && automaticBlockReason
                ? 'Fix the reading below before testing.'
                : 'Keep the game open. We’ll handle the rest.'}
          </p>
        </div>
        {#if !movementVerified && !pending && automaticBlockReason}<p
            class="calibration-inline-status"
            id={testHintId}
          >
            {result?.boardUnclear
              ? 'The board needs adjusting before a test.'
              : result?.unknownSlots.length
                ? 'A piece needs adjusting before a test.'
                : !captureId || !previewCurrent
                  ? 'Refresh the screen before testing.'
                  : automaticBlockReason}
          </p>
          <div class="calibration-actions">
            <button class="button" disabled={blocked} onclick={openBoardEditor}
              >Adjust board alignment</button
            ><button class="button" disabled={blocked} onclick={() => goToStep(2)}
              >Adjust tray alignment</button
            ><button
              class="button"
              disabled={blocked || !activeSerial}
              onclick={() => void capture(false)}>Refresh screen</button
            >
          </div>{/if}
        <details class="calibration-options">
          <summary>Manual test & adjustments</summary>
          <div>
            <div class="calibration-coordinates">
              <label class="settings-field"
                ><span>Drag gain</span><input
                  class="settings-input"
                  aria-label="Drag gain"
                  type="number"
                  min="0.1"
                  step="0.05"
                  bind:value={profile.touch.gain}
                  disabled={pending}
                /></label
              >
              <label class="settings-field"
                ><span>Drag lift (px)</span><input
                  class="settings-input"
                  aria-label="Drag lift"
                  type="number"
                  min="0"
                  max="600"
                  step="1"
                  bind:value={profile.touch.lift}
                  disabled={pending}
                /></label
              >
            </div>
            <p class="calibration-help">
              Gain controls how far the shape moves for each drag. Lift compensates for the shape
              appearing above your finger. The test uses the same drag calculation as normal play.
            </p>
            <div class="calibration-coordinates">
              <label class="settings-field"
                ><span>Shape</span><select
                  class="settings-select"
                  aria-label="Test slot"
                  bind:value={slot}
                  disabled={pending}
                  >{#each [0, 1, 2] as index}<option value={index}>Slot {index + 1}</option
                    >{/each}</select
                ></label
              >
              <label class="settings-field"
                ><span>Target row</span><input
                  class="settings-input"
                  aria-label="Target row"
                  type="number"
                  min="1"
                  max="8"
                  bind:value={row}
                  disabled={pending}
                /></label
              >
              <label class="settings-field"
                ><span>Target column</span><input
                  class="settings-input"
                  aria-label="Target column"
                  type="number"
                  min="1"
                  max="8"
                  bind:value={column}
                  disabled={pending}
                /></label
              >
            </div>
            <label class="settings-helper"
              ><input type="checkbox" bind:checked={installHelper} disabled={pending} /><span
                class="settings-copy"
                ><strong>Install capture helper for test drag</strong><small
                  >Enable once if it is not installed on this phone.</small
                ></span
              ></label
            >
            <button class="button" disabled={!!testBlockReason} onclick={() => void testDrag(false)}
              >Run one manual test drag</button
            >
          </div>

          <p class="calibration-help">{testBlockReason || 'Ready for one manual test.'}</p>
          <button
            class="button"
            disabled={blocked || !captureId || !ready}
            onclick={() => void preview()}>Preview recognition</button
          >
          <button
            class="button"
            disabled={blocked || !activeSerial}
            onclick={() => void capture(false)}>Capture again</button
          >
          <div class="calibration-actions">
            <button class="button" disabled={blocked} onclick={openBoardEditor}
              >Adjust board alignment</button
            ><button class="button" disabled={blocked} onclick={() => goToStep(2)}
              >Adjust tray alignment</button
            >
          </div>
        </details>
      {:else if step === 4}
        {#if boardEstimated}
          <p class="calibration-inline-status">Review the full board edges before saving.</p>
          <button class="button" disabled={blocked} onclick={openBoardEditor}>Review board</button>
        {:else}
          <div class="calibration-save-summary">
            <span aria-hidden="true">✓</span>
            <p>
              {image ? 'Board and pieces aligned' : 'Profile ready to save'}{movementVerified
                ? ' · movement checked'
                : ''}.
            </p>
          </div>
        {/if}
        <label class="settings-field"
          ><span>Profile name</span><input
            class="settings-input"
            aria-label="Hardware profile name"
            aria-invalid={!!nameError}
            bind:value={name}
            maxlength="80"
            disabled={pending}
          /></label
        >
        {#if nameError}<p class="calibration-inline-status">{nameError}</p>{/if}
        {#if !movementVerified}<p class="calibration-help">
            Movement hasn’t been tested. <button
              class="calibration-link"
              disabled={blocked}
              onclick={() => goToStep(3)}>Run a test</button
            >
          </p>{/if}
        <details class="calibration-options">
          <summary>Profile options</summary>
          <p class="calibration-help">Use a different name to keep an existing profile.</p>
          <button
            class="button"
            disabled={pending || !ready}
            onclick={() => void exportProfile(false)}>Export HP profile</button
          >
          <details class="settings-advanced calibration-timing">
            <summary>Advanced detection & timing</summary>
            <div class="settings-advanced-body">
              <label class="settings-field"
                ><span>Cell majority (0–1)</span><input
                  class="settings-input"
                  type="number"
                  min="0.01"
                  max="1"
                  step="0.01"
                  bind:value={profile.screen.cell_majority}
                  disabled={pending}
                /></label
              >
              {#each ['hold', 'drop', 'gap', 'risk_max'] as key}<label class="settings-field"
                  ><span>Touch {key} (seconds)</span><input
                    class="settings-input"
                    type="number"
                    min="0"
                    max="1"
                    step="0.01"
                    bind:value={profile.touch[key as keyof Profile['touch']]}
                    disabled={pending}
                  /></label
                >{/each}
              {#each ['tray_wait', 'retry_wait', 'command_timeout', 'tray_attempts'] as key}<label
                  class="settings-field"
                  ><span>{key.replaceAll('_', ' ')}</span><input
                    class="settings-input"
                    type="number"
                    min={key === 'tray_attempts' ? 1 : 0.01}
                    max={key === 'tray_attempts' ? 20 : 120}
                    step={key === 'tray_attempts' ? 1 : 0.01}
                    bind:value={profile.runtime[key as keyof Profile['runtime']]}
                    disabled={pending}
                  /></label
                >{/each}
            </div>
          </details>
        </details>
      {/if}
      {#if (step === 1 || step === 3) && result && previewCurrent}
        <details class="calibration-options">
          <summary>Recognition details</summary>
          <p class="calibration-help">
            Board: {result.boardUnclear ? 'needs adjustment' : 'readable'} · Pieces: {result
              .unknownSlots.length
              ? 'needs adjustment'
              : 'readable'}
          </p>
          <div class="calibration-preview">
            {#if result.board}<div class="calibration-board" aria-label="Detected board">
                {#each result.board as line, r}{#each [...line] as cell, c}<i
                      class:filled={cell === '#'}
                      class:unclear={result.unclearCells.includes(r * 8 + c)}
                      title={`Row ${r + 1}, column ${c + 1}: ${result.unclearCells.includes(r * 8 + c) ? 'unclear' : cell === '#' ? 'filled' : 'empty'}`}
                    ></i>{/each}{/each}
              </div>{/if}
            <div class="calibration-pieces">
              {#each [0, 1, 2] as index}{@const piece = result.pieces.find((p) => p.slot === index)}
                <div>
                  <span>Slot {index + 1}</span>{#if piece}<PieceShape
                      cells={piece.cells}
                    />{:else}<small>Empty / unreadable</small>{/if}
                </div>{/each}
            </div>
          </div>
        </details>
      {/if}
    {/if}
  </div>
  <footer class="calibration-wizard-footer">
    {#if saved}<button class="button button-primary" onclick={onclose}>Done</button>
    {:else}
      {#if step > 0}<button
          class="calibration-link"
          disabled={blocked}
          onclick={() => goToStep(step - 1)}>Back</button
        >{:else}<span></span>{/if}
      <div class="calibration-footer-actions">
        {#if step === 1 || step === 2}<button
            class="calibration-link"
            disabled={blocked}
            onclick={showAdjustments}>Adjust</button
          >{/if}
        {#if step === 3 && !movementVerified}<button
            class="calibration-link"
            disabled={blocked}
            onclick={() => goToStep(4)}>Skip test</button
          >{/if}
        {#if step === 0}<button
            class="button button-primary"
            disabled={blocked || !activeSerial}
            onclick={() => void capture()}>Start calibration</button
          >
        {:else if step === 1 || step === 2}<button
            class="button button-primary"
            disabled={blocked || !ready}
            onclick={() => void nextStep()}
            >{step === 1 ? 'Board looks right' : 'Pieces look right'}</button
          >
        {:else if step === 3}<button
            class="button button-primary"
            disabled={blocked || (!movementVerified && !!automaticBlockReason)}
            onclick={() => {
              if (movementVerified) goToStep(4)
              else void testDrag()
            }}>{movementVerified ? 'Continue' : 'Test movement'}</button
          >
        {:else}<button
            class="button button-primary"
            disabled={blocked || !saveReady}
            onclick={() => void exportProfile(true)}>Save calibration</button
          >{/if}
      </div>
    {/if}
  </footer>
</div>
