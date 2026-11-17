<script lang="ts">
  import { onMount } from 'svelte'
  import type { Profile } from '../../generated/contracts'
  import { calibrationApi } from '../../lib/native/calibration'
  let {
    profile,
    image,
    captureId,
    disabled,
    onchange,
  }: {
    profile: Profile
    image: string
    captureId: number | null
    disabled: boolean
    onchange: (profile: Profile) => void
  } = $props()
  let selected = $state(0)
  let dimensions = $state<[number, number][]>([
    [3, 3],
    [3, 3],
    [3, 3],
  ])
  let offsets = $state<[number, number][]>([
    [0, 0],
    [0, 0],
    [0, 0],
  ])
  let unclear = $state<number[]>([])
  let locating = $state(false)
  let adjusting = $state(false)
  let emptySlots = $state<number[]>([])
  let canvas = $state<SVGSVGElement>(null!)
  let viewport = $state<[number, number, number, number]>([0, 0, 1, 1])
  let active = $state(false)
  let dragging: {
    index: number
    mode: 'move' | 'size'
    start: [number, number]
    profile: Profile
    grid: [number, number, number, number]
  } | null = null
  function fit() {
    const pad = profile.screen.preview_pitch * 1.5
    const top = Math.max(0, Math.min(...profile.screen.slots.map((r) => r[1])) - pad)
    const bottom = Math.min(
      profile.screen.height,
      Math.max(...profile.screen.slots.map((r) => r[1] + r[3])) + pad,
    )
    viewport = [0, top, profile.screen.width, Math.max(1, bottom - top)]
  }
  function zoomSlot(index: number) {
    const [x, y, w, h] = profile.screen.slots[index]!
    const pad = profile.screen.preview_pitch
    const left = Math.max(0, x - pad),
      top = Math.max(0, y - pad)
    const right = Math.min(profile.screen.width, x + w + pad),
      bottom = Math.min(profile.screen.height, y + h + pad)
    viewport = [left, top, right - left, bottom - top]
    selected = index
  }
  onMount(() => {
    fit()
    let disposed = false
    if (captureId) {
      locating = true
      void calibrationApi
        .request({ type: 'preview', captureId, profile: $state.snapshot(profile) })
        .then((result) => {
          if (disposed || active) return
          unclear = result.unknownSlots
          emptySlots = [0, 1, 2].filter(
            (i) => !result.pieces.some((p) => p.slot === i) && !result.unknownSlots.includes(i),
          )
          for (const piece of result.pieces) {
            const anchor = result.anchors.find((a) => a.slot === piece.slot)
            if (!anchor) continue
            dimensions[piece.slot] = [
              Math.max(...piece.cells.map((c) => c[1])) + 1,
              Math.max(...piece.cells.map((c) => c[0])) + 1,
            ]
            const area = profile.screen.slots[piece.slot]!
            offsets[piece.slot] = [
              anchor.x +
                (dimensions[piece.slot]![0] * profile.screen.preview_pitch) / 2 -
                area[0] -
                area[2] / 2,
              anchor.y +
                (dimensions[piece.slot]![1] * profile.screen.preview_pitch) / 2 -
                area[1] -
                area[3] / 2,
            ]
          }
        })
        .catch(() => {
          if (!disposed) unclear = [0, 1, 2]
        })
        .finally(() => {
          if (!disposed) locating = false
        })
    }
    return () => {
      disposed = true
    }
  })
  function grid(index: number, value = profile): [number, number, number, number] {
    const [x, y, w, h] = value.screen.slots[index]!
    const [cols, rows] = dimensions[index]!
    const pitch = value.screen.preview_pitch
    return [
      x + w / 2 + offsets[index]![0] - (cols * pitch) / 2,
      y + h / 2 + offsets[index]![1] - (rows * pitch) / 2,
      cols * pitch,
      rows * pitch,
    ]
  }
  function point(event: PointerEvent): [number, number] {
    const bounds = canvas.getBoundingClientRect()
    return [
      viewport[0] + ((event.clientX - bounds.left) / bounds.width) * viewport[2],
      viewport[1] + ((event.clientY - bounds.top) / bounds.height) * viewport[3],
    ]
  }
  function centerRect(
    rect: [number, number, number, number],
    cx: number,
    cy: number,
    width = rect[2],
    height = rect[3],
  ): [number, number, number, number] {
    const w = Math.min(profile.screen.width, Math.max(1, Math.round(width)))
    const h = Math.min(profile.screen.height, Math.max(1, Math.round(height)))
    return [
      Math.max(0, Math.min(profile.screen.width - w, Math.round(cx - w / 2))),
      Math.max(0, Math.min(profile.screen.height - h, Math.round(cy - h / 2))),
      w,
      h,
    ]
  }
  function move(index: number, dx: number, dy: number, value = profile) {
    const slots: Profile['screen']['slots'] = value.screen.slots.map((r) => [
      ...r,
    ]) as Profile['screen']['slots']
    const rect = slots[index]!
    slots[index] = centerRect(rect, rect[0] + rect[2] / 2 + dx, rect[1] + rect[3] / 2 + dy)
    onchange({ ...value, screen: { ...value.screen, slots } })
  }
  function resize(pitch: number, value = profile, anchor?: [number, number, number]) {
    if (!Number.isFinite(pitch)) return
    pitch =
      Math.round(
        Math.max(4, Math.min(Math.min(value.screen.width, value.screen.height) / 6, pitch)) * 10,
      ) / 10
    const slots = value.screen.slots.map((rect, index) => {
      let cx = rect[0] + rect[2] / 2,
        cy = rect[1] + rect[3] / 2
      if (anchor && index === anchor[0]) {
        cx = anchor[1] + (dimensions[index]![0] * pitch) / 2 - offsets[index]![0]
        cy = anchor[2] + (dimensions[index]![1] * pitch) / 2 - offsets[index]![1]
      }
      return centerRect(rect, cx, cy, 6 * pitch, 6 * pitch)
    }) as Profile['screen']['slots']
    onchange({ ...value, screen: { ...value.screen, preview_pitch: pitch, slots } })
  }
  function start(event: PointerEvent, index: number, mode: 'move' | 'size') {
    if (disabled || event.button !== 0) return
    event.preventDefault()
    event.stopPropagation()
    selected = index
    active = true
    canvas.setPointerCapture(event.pointerId)
    dragging = {
      index,
      mode,
      start: point(event),
      profile: $state.snapshot(profile),
      grid: grid(index),
    }
  }
  function drag(event: PointerEvent) {
    if (!dragging || disabled) return
    const [x, y] = point(event)
    if (dragging.mode === 'move')
      move(dragging.index, x - dragging.start[0], y - dragging.start[1], dragging.profile)
    else {
      const delta = (x - dragging.start[0] + (y - dragging.start[1])) / 2
      resize(dragging.profile.screen.preview_pitch + delta, dragging.profile, [
        dragging.index,
        dragging.grid[0],
        dragging.grid[1],
      ])
    }
  }
  function keyboard(event: KeyboardEvent, index: number, mode: 'move' | 'size') {
    if (disabled || !['ArrowLeft', 'ArrowRight', 'ArrowUp', 'ArrowDown'].includes(event.key)) return
    event.preventDefault()
    event.stopPropagation()
    active = true
    selected = index
    const delta = event.shiftKey ? 10 : 1
    if (mode === 'size')
      resize(
        profile.screen.preview_pitch +
          (['ArrowLeft', 'ArrowUp'].includes(event.key) ? -delta : delta),
      )
    else
      move(
        index,
        event.key === 'ArrowLeft' ? -delta : event.key === 'ArrowRight' ? delta : 0,
        event.key === 'ArrowUp' ? -delta : event.key === 'ArrowDown' ? delta : 0,
      )
  }
</script>

<div class="tray-grid-editor">
  {#if locating}<p class="settings-note" role="status">
      Aligning tray grids to detected blocks… Please wait.
    </p>{/if}
  {#if unclear.length}<p class="settings-note" role="status">
      Check pieces {unclear.map((i) => i + 1).join(', ')}. Their grids are estimates.
    </p>{/if}
  {#if image}
    <svg
      class="tray-grid-canvas"
      bind:this={canvas}
      viewBox={viewport.join(' ')}
      role="group"
      aria-label="Drag tray grids over shapes"
      onpointermove={drag}
      onpointerup={() => (dragging = null)}
      onpointercancel={() => (dragging = null)}
      onlostpointercapture={() => (dragging = null)}
    >
      <image href={image} width={profile.screen.width} height={profile.screen.height} />
      {#each locating ? [] : [0, 1, 2].filter((i) => adjusting || !emptySlots.includes(i)) as index}
        {@const rect = grid(index)}
        {@const pitch = profile.screen.preview_pitch}
        {@const counts = dimensions[index]!}
        <g
          class:active={selected === index}
          class="tray-cell-grid"
          role="button"
          tabindex={disabled ? -1 : 0}
          aria-disabled={disabled}
          aria-label={`Move slot ${index + 1} grid`}
          onpointerdown={(event) => start(event, index, 'move')}
          onkeydown={(event) => keyboard(event, index, 'move')}
        >
          <text x={rect[0]} y={rect[1] - 16}>Slot {index + 1}</text>
          {#each Array(counts[1]) as _, row}{#each Array(counts[0]) as _, col}<rect
                class="tray-grid-cell"
                x={rect[0] + col * pitch}
                y={rect[1] + row * pitch}
                width={pitch}
                height={pitch}
              />{/each}{/each}
        </g>
        {#if selected === index}<g
            class="tray-grid-size-handle"
            role="button"
            tabindex={disabled ? -1 : 0}
            aria-disabled={disabled}
            aria-label={`Resize slot ${index + 1} cells`}
            onpointerdown={(event) => start(event, index, 'size')}
            onkeydown={(event) => keyboard(event, index, 'size')}
          >
            <circle class="tray-grid-handle-hit" cx={rect[0] + pitch} cy={rect[1] + pitch} r="28" />
            <circle cx={rect[0] + pitch} cy={rect[1] + pitch} r="10" />
          </g>{/if}
      {/each}
    </svg>
  {:else}<p class="settings-note">
      Capture a screenshot in Screen to position the grids visually.
    </p>{/if}
  <p class="calibration-gesture">Drag a grid to move it · round handle to resize</p>
  <details class="calibration-options" data-adjust-pieces bind:open={adjusting}>
    <summary>Adjust pieces</summary>
    <div class="calibration-actions" aria-label="Tray slot selection">
      {#each [0, 1, 2] as index}<button
          class="button"
          aria-pressed={selected === index}
          {disabled}
          onclick={() => zoomSlot(index)}>Slot {index + 1}</button
        >{/each}
      <button class="button" {disabled} onclick={() => fit()}>Fit tray</button>
    </div>
    <div class="tray-grid-controls">
      <label class="settings-field"
        ><span>Grid columns</span><select
          class="settings-select"
          aria-label="Grid columns"
          value={dimensions[selected]![0]}
          {disabled}
          onchange={(event) => {
            active = true
            dimensions[selected]![0] = Number(event.currentTarget.value)
          }}
          >{#each [1, 2, 3, 4, 5] as n}<option value={n}>{n}</option>{/each}</select
        ></label
      >
      <label class="settings-field"
        ><span>Grid rows</span><select
          class="settings-select"
          aria-label="Grid rows"
          value={dimensions[selected]![1]}
          {disabled}
          onchange={(event) => {
            active = true
            dimensions[selected]![1] = Number(event.currentTarget.value)
          }}
          >{#each [1, 2, 3, 4, 5] as n}<option value={n}>{n}</option>{/each}</select
        ></label
      >
      <label class="settings-field"
        ><span>Cell size (px)</span><input
          class="settings-input"
          type="number"
          aria-label="Tray cell size"
          min="4"
          step="0.1"
          value={profile.screen.preview_pitch}
          {disabled}
          onchange={(event) => {
            active = true
            resize(Number(event.currentTarget.value))
          }}
        /></label
      >
    </div>
    <p class="calibration-help">
      Cell size applies to all pieces. Arrow keys move 1 px; Shift moves 10 px.
    </p>
    <details class="settings-advanced">
      <summary>Advanced reading area · Slot {selected + 1}</summary>
      <p class="calibration-help">
        The reading area includes space for larger shapes in later trays. Only adjust these bounds
        if recognition still misses a shape.
      </p>
      <div class="calibration-coordinates">
        {#each ['X', 'Y', 'Width', 'Height'] as label, index}<label class="settings-field"
            ><span>{label}</span><input
              class="settings-input"
              type="number"
              aria-label={`Slot area ${label}`}
              min={index > 1 ? 1 : 0}
              value={profile.screen.slots[selected]![index]}
              {disabled}
              oninput={(event) => {
                const slots = profile.screen.slots.map((r) => [...r]) as Profile['screen']['slots']
                slots[selected]![index] = Number(event.currentTarget.value)
                onchange({ ...profile, screen: { ...profile.screen, slots } })
              }}
            /></label
          >{/each}
      </div>
    </details>
  </details>
</div>
