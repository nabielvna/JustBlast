<script lang="ts">
  import { onMount } from 'svelte'
  import type {
    PieceView,
    Preferences,
    SavedShape,
    CaptureView,
    SlotAnchor,
  } from '../../generated/contracts'
  import Icon from '../../components/ui/Icon.svelte'
  import PieceShape from '../../components/board/PieceShape.svelte'
  import {
    normalize,
    presets,
    shapeKey,
    transform,
    parseImportedShapes,
  } from '../../lib/presentation/shapes'
  import { preferencesApi } from '../../lib/native/preferences'
  let {
    slot,
    initial,
    onsave,
    onclose,
    capture,
    anchor,
    onanchor,
  }: {
    slot: number
    initial?: PieceView
    onsave: (piece: PieceView | null) => Promise<void>
    onclose: () => void
    capture?: CaptureView | null
    anchor?: SlotAnchor
    onanchor?: (anchor: SlotAnchor) => void
  } = $props()
  let originX = $state(0),
    originY = $state(0),
    originChanged = $state(false)
  let dialog: HTMLDialogElement
  let panel = $state<'shape' | 'phone'>('shape')
  let showLibrary = $state(false)
  let cells = $state<[number, number][]>([])
  let normalized = $derived(normalize(cells))
  let collection = $state<'presets' | 'custom'>('presets')
  let query = $state(''),
    name = $state(''),
    message = $state('')
  let preferences = $state<Preferences | null>(null)
  let saving = $state(false)
  let custom = $derived(preferences?.shapes ?? [])
  let duplicate = $derived(custom.some((s) => shapeKey(s.cells) === shapeKey(cells)))
  let shapes = $derived(
    (collection === 'presets' ? presets : custom).filter((s) =>
      s.name.toLowerCase().includes(query.trim().toLowerCase()),
    ),
  )
  onMount(() => {
    cells = initial?.cells.map((c) => [...c]) ?? []
    const origin = anchor ?? capture?.anchors.find((a) => a.slot === slot)
    originX = origin?.x ?? 0
    originY = origin?.y ?? 0
    dialog.showModal()
    let alive = true
    void preferencesApi
      .snapshot()
      .then((v) => {
        if (alive) preferences = v
      })
      .catch((e: unknown) => {
        if (alive) message = e instanceof Error ? e.message : 'Could not load saved shapes.'
      })
    return () => {
      alive = false
    }
  })
  function dismiss() {
    dialog.close()
    onclose()
  }
  function toggle(r: number, c: number) {
    cells = cells.some((cell) => cell[0] === r && cell[1] === c)
      ? cells.filter((cell) => cell[0] !== r || cell[1] !== c)
      : [...cells, [r, c]]
  }
  async function persist(shapes: SavedShape[], success: string) {
    if (!preferences || saving) return false
    saving = true
    try {
      preferences = await preferencesApi.save({ ...preferences, shapes })
      message = success
      return true
    } catch (e) {
      message = e instanceof Error ? e.message : 'Could not save changes.'
      try {
        preferences = await preferencesApi.snapshot()
      } catch {}
      return false
    } finally {
      saving = false
    }
  }
  async function importShapes(event: Event) {
    const file = (event.currentTarget as HTMLInputElement).files?.[0]
    if (!file || file.size > 1000000) {
      message = 'Choose a shape JSON file smaller than 1 MB.'
      return
    }
    try {
      const imported = parseImportedShapes(await file.text())
      await persist(
        [...custom, ...imported.filter((s) => !custom.some((c) => c.id === s.id))],
        'Shapes imported.',
      )
      collection = 'custom'
    } catch (e) {
      message = e instanceof Error ? e.message : 'Invalid shape file.'
    }
  }
</script>

<dialog
  class={`control-config piece-editor piece-color-${slot}`}
  bind:this={dialog}
  aria-labelledby="piece-editor-title"
  oncancel={(e) => {
    e.preventDefault()
    dismiss()
  }}
  onclick={(e) => {
    if (e.target === dialog) dismiss()
  }}
>
  <div class="control-config-header">
    <div class="piece-editor-title-group">
      <span class="piece-editor-eyebrow">PIECE EDITOR</span>
      <h2 id="piece-editor-title">
        <span class="piece-editor-slot-dot"></span>Edit Slot {slot + 1}
      </h2>
    </div>
    <button
      type="button"
      class="button control-settings"
      aria-label="Close piece editor"
      onclick={dismiss}><Icon name="close" /></button
    >
  </div>
  <div class="piece-editor-navigation" role="group" aria-label="Editor view">
    <button
      class="button"
      type="button"
      aria-pressed={!showLibrary && panel === 'shape'}
      onclick={() => {
        panel = 'shape'
        showLibrary = false
      }}>Shape</button
    >
    <button
      class="button piece-library-navigation"
      type="button"
      aria-pressed={showLibrary}
      onclick={() => {
        showLibrary = true
      }}>Library</button
    >
    {#if capture}<button
        class="button"
        type="button"
        aria-pressed={!showLibrary && panel === 'phone'}
        onclick={() => {
          panel = 'phone'
          showLibrary = false
        }}>Phone position</button
      >{/if}
  </div>
  <div class="piece-editor-layout" data-view={showLibrary ? 'library' : panel}>
    <div class="piece-editor-drawing" hidden={panel !== 'shape'}>
      <div class="piece-drawing-heading">
        <h3>Draw your piece</h3>
        <span>5 × 5</span>
      </div>
      <p class="piece-editor-help">Click cells to draw, or pick a shape from the library.</p>
      <div
        class="manual-piece-grid manual-grid"
        role="group"
        aria-label={`Edit slot ${slot + 1}, 5 by 5`}
      >
        {#each Array(25) as _, i (i)}{@const r = Math.floor(i / 5)}{@const c = i % 5}<button
            type="button"
            class={`manual-piece-cell${cells.some((cell) => cell[0] === r && cell[1] === c) ? ' is-filled' : ''}`}
            aria-label={`Slot ${slot + 1}, row ${r + 1}, column ${c + 1}`}
            aria-pressed={cells.some((cell) => cell[0] === r && cell[1] === c)}
            onclick={() => toggle(r, c)}
          ></button>{/each}
      </div>
      <div class="piece-editor-meta">
        <span><strong>{cells.length}</strong> / 25 cells filled</span><button
          type="button"
          class="legend-info-button"
          aria-label="Clear piece"
          onclick={() => {
            cells = []
          }}><Icon name="clear" /></button
        >
      </div>
      <div class="piece-editor-transforms">
        <button
          class="button"
          type="button"
          disabled={!cells.length}
          onclick={() => {
            cells = transform(cells, 'rotate')
          }}>Rotate ↻</button
        ><button
          class="button"
          type="button"
          disabled={!cells.length}
          onclick={() => {
            cells = transform(cells, 'flip')
          }}>Flip ↔</button
        >
      </div>
    </div>
    {#if capture}
      <section class="piece-capture" hidden={panel !== 'phone'} aria-label="Phone piece position">
        <h3>Phone piece position</h3>
        <p>
          Click the top-left corner of the piece grid, including empty corners. Fine-tune X/Y below.
        </p>
        <div class="slot-crop-stage">
          <button
            class="slot-crop-origin"
            type="button"
            aria-label={`Set grid origin for slot ${slot + 1}`}
            onclick={(e) => {
              const rect = e.currentTarget.getBoundingClientRect()
              const size = capture!.slotSizes[slot]!
              originX = Math.max(0, Math.round(((e.clientX - rect.left) * size[0]) / rect.width))
              originY = Math.max(0, Math.round(((e.clientY - rect.top) * size[1]) / rect.height))
              originChanged = true
            }}
          >
            <img src={capture.slotCrops[slot]} alt={`Phone crop for slot ${slot + 1}`} />
            {#each normalized as [r, c] (`${r}-${c}`)}<i
                style={`left:${((originX + c * capture.pitch) / capture.slotSizes[slot]![0]) * 100}%;top:${((originY + r * capture.pitch) / capture.slotSizes[slot]![1]) * 100}%;width:${(capture.pitch / capture.slotSizes[slot]![0]) * 100}%;height:${(capture.pitch / capture.slotSizes[slot]![1]) * 100}%`}
              ></i>{/each}
          </button>
        </div>
        <div class="slot-origin-fields">
          <label
            >Origin X<input
              aria-label="Piece origin X"
              type="number"
              min="0"
              max={capture.slotSizes[slot]?.[0]}
              bind:value={originX}
              onchange={() => {
                originChanged = true
              }}
            /></label
          ><label
            >Origin Y<input
              aria-label="Piece origin Y"
              type="number"
              min="0"
              max={capture.slotSizes[slot]?.[1]}
              bind:value={originY}
              onchange={() => {
                originChanged = true
              }}
            /></label
          >
        </div>
      </section>
    {/if}
    <section class="shape-library" aria-label="Shape library">
      <div class="shape-library-heading">
        <h3>Shape library</h3>
        <span>{collection === 'presets' ? presets.length : custom.length} shapes</span>
      </div>
      <div class="shape-library-tabs" role="group" aria-label="Shape collection">
        <button
          class="button"
          type="button"
          aria-pressed={collection === 'presets'}
          onclick={() => {
            collection = 'presets'
          }}>Presets</button
        ><button
          class="button"
          type="button"
          aria-pressed={collection === 'custom'}
          onclick={() => {
            collection = 'custom'
          }}>My shapes</button
        >
      </div>
      <label class="shape-library-search"
        ><Icon name="search" /><input
          aria-label="Search shapes"
          placeholder="Search shapes…"
          bind:value={query}
        /></label
      >
      <div class="shape-library-grid">
        {#each shapes as shape (shape.id)}<div class="shape-library-item">
            <button
              class="shape-library-choice"
              type="button"
              aria-label={`Use ${shape.name}`}
              aria-pressed={shapeKey(shape.cells) === shapeKey(cells)}
              onclick={() => {
                cells = shape.cells.map((c) => [...c])
                message = `${shape.name} loaded. Apply to slot when ready.`
              }}><PieceShape cells={shape.cells} /><span>{shape.name}</span></button
            >{#if collection === 'custom'}<button
                class="shape-library-delete"
                type="button"
                disabled={saving}
                aria-label={`Delete ${shape.name}`}
                onclick={() =>
                  void persist(
                    custom.filter((s) => s.id !== shape.id),
                    `${shape.name} deleted.`,
                  )}><Icon name="close" /></button
              >{/if}
          </div>{/each}
        {#if !shapes.length}<p class="shape-library-empty">
            {query.trim() ? 'No matching shapes.' : 'Save a drawing to start your collection.'}
          </p>{/if}
      </div>
      <form
        class="shape-library-save"
        onsubmit={(e) => {
          e.preventDefault()
          if (!cells.length || duplicate || !name.trim()) return
          void persist(
            [
              ...custom,
              { id: `custom-${crypto.randomUUID()}`, name: name.trim(), cells: normalize(cells) },
            ],
            'Shape saved to My shapes.',
          ).then((saved) => {
            if (!saved) return
            name = ''
            collection = 'custom'
            query = ''
          })
        }}
      >
        <label for="custom-shape-name">Save this drawing</label>
        <div>
          <input
            id="custom-shape-name"
            maxlength="40"
            placeholder="Shape name"
            bind:value={name}
          /><button
            class="button"
            type="submit"
            disabled={!cells.length || duplicate || !name.trim() || !preferences || saving}
            >Save shape</button
          >
        </div>
        <small
          >{duplicate
            ? 'This shape is already in My shapes.'
            : 'My shapes are saved in this application.'}</small
        >
        {#if collection === 'custom'}
          <label class="shape-library-import"
            >Import legacy shapes<input
              type="file"
              accept="application/json,.json"
              disabled={!preferences || saving}
              onchange={(e) => void importShapes(e)}
            /></label
          >
        {/if}
      </form>
    </section>
  </div>
  <p class="shape-library-status" role="status">{message}</p>
  <div class="piece-editor-actions">
    <button type="button" class="button" onclick={dismiss}>Cancel</button><button
      type="button"
      class="button button-primary"
      disabled={saving}
      onclick={() => {
        if (originChanged && capture) {
          if (
            !Number.isInteger(originX) ||
            !Number.isInteger(originY) ||
            originX < 0 ||
            originY < 0
          ) {
            message = 'Choose a valid grid origin.'
            return
          }
          onanchor?.({ slot, x: originX, y: originY })
        }
        void onsave(
          cells.length
            ? {
                slot,
                catalogId: (() => {
                  const preset = presets.find((p) => shapeKey(p.cells) === shapeKey(cells))
                  return preset ? Number(preset.id.slice(5)) : null
                })(),
                name:
                  initial && shapeKey(initial.cells) === shapeKey(cells)
                    ? initial.name
                    : `Custom piece · ${cells.length} cells`,
                cells: normalize(cells),
              }
            : null,
        )
          .then(dismiss)
          .catch((reason: unknown) => {
            message = reason instanceof Error ? reason.message : 'Could not apply this piece.'
          })
      }}>Apply to slot</button
    >
  </div>
</dialog>
