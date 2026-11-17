<script lang="ts">
  import type { WorkspaceView, PlanView, PieceView } from '../../generated/contracts'
  import BoardLegend from './BoardLegend.svelte'
  let {
    rows,
    plan,
    pieces,
    selected,
    completed,
    phase,
    exploring = false,
    alternative = null,
    alternativeCount = null,
    onedit,
    editDisabled = false,
  }: {
    rows: WorkspaceView['board']
    plan: PlanView
    pieces: PieceView[]
    selected: number
    completed: number
    phase: 'idle' | 'placing' | 'breaking' | 'settling'
    exploring?: boolean
    alternative?: PlanView | null
    alternativeCount?: number | null
    onedit?: (row: number, column: number) => void
    editDisabled?: boolean
  } = $props()
  let step = $derived(
    plan.steps[
      phase === 'settling' ? Math.max(0, completed - 1) : phase === 'idle' ? selected : completed
    ],
  )
  let planned = $derived(!exploring && phase === 'idle' && selected >= completed)
  let displayRows = $derived(planned ? (step?.before ?? rows) : rows)
  let piece = $derived(pieces.find((p) => p.slot === step?.slot))
  let footprint = $derived(
    new Set(
      piece?.cells.map(
        ([r, c]) => `${r + (step?.movement.row ?? 0)}:${c + (step?.movement.column ?? 0)}`,
      ) ?? [],
    ),
  )
  let clearRows = $derived(step?.clearedRows ?? [])
  let clearCols = $derived(step?.clearedColumns ?? [])
  let showClear = $derived(!exploring && (planned || phase === 'placing' || phase === 'breaking'))
  let previous = $derived.by(() => {
    const labels = new Map<string, number>()
    if (exploring) return labels
    const count = planned ? selected : completed
    for (let i = 0; i < count; i++) {
      const s = plan.steps[i]
      const p = pieces.find((p) => p.slot === s?.slot)
      if (!s || !p) continue
      for (const [r, c] of p.cells)
        labels.set(`${r + s.movement.row}:${c + s.movement.column}`, i + 1)
      for (const key of labels.keys()) {
        const [r, c] = key.split(':').map(Number)
        if (s.after[r!]?.[c!] !== '#') labels.delete(key)
      }
    }
    return labels
  })
  let alternativeCells = $derived.by(() => {
    const result = new Map<string, { slot: number; step: number }>()
    if (exploring && alternative)
      for (
        let i = completed;
        i < Math.min(alternative.steps.length, alternativeCount ?? alternative.steps.length);
        i++
      ) {
        const s = alternative.steps[i]!,
          p = pieces.find((p) => p.slot === s.slot)
        for (const [r, c] of p?.cells ?? [])
          result.set(`${r + s.movement.row}:${c + s.movement.column}`, {
            slot: s.slot,
            step: i + 1,
          })
      }
    return result
  })
  let caption = $derived(
    exploring
      ? alternative
        ? `Preview · ${alternative.steps
            .slice(completed, alternativeCount ?? alternative.steps.length)
            .map((s) => `Slot ${s.slot + 1}`)
            .join(' → ')}`
        : 'Hover a position to preview a block'
      : step
        ? `Slot ${step.slot + 1} · row ${step.movement.row + 1}, column ${step.movement.column + 1}`
        : 'All moves completed',
  )
</script>

<section
  class={`board-section${exploring ? ' is-exploring' : ''}`}
  aria-label="Game board"
  data-motion-phase={phase}
>
  <span class="sr-only" role="status"
    >{phase === 'idle'
      ? 'Ready'
      : phase === 'breaking'
        ? 'Clearing'
        : phase === 'placing'
          ? 'Placing'
          : 'Placement complete'}</span
  >
  <div class="board-area">
    <div class="board-layout">
      <div class="board-columns" aria-hidden="true">
        {#each displayRows as _, i (i)}<span
            class={`axis-label${showClear && clearCols.includes(i) ? ' is-break-target' : ''}`}
            >{i + 1}</span
          >{/each}
      </div>
      <div class="board-rows" aria-hidden="true">
        {#each displayRows as _, i (i)}<span
            class={`axis-label${showClear && clearRows.includes(i) ? ' is-break-target' : ''}`}
            >{i + 1}</span
          >{/each}
      </div>
      <div
        class="board"
        role="group"
        aria-label={`8 by 8 board, ${displayRows.join('').split('#').length - 1} filled cells`}
      >
        {#if showClear && (clearRows.length || clearCols.length)}<div
            class={`break-background${phase === 'settling' ? ' is-cleared' : ''}`}
            aria-hidden="true"
          >
            {#each clearRows as row (row)}<span
                class="break-band"
                style={`grid-row:${row + 1};grid-column:1 / -1`}
              ></span>{/each}{#each clearCols as col (col)}<span
                class="break-band"
                style={`grid-column:${col + 1};grid-row:1 / -1`}
              ></span>{/each}
          </div>{/if}
        {#each displayRows as row, r (r)}{#each [...row] as cell, c (c)}
            {@const key = `${r}:${c}`}{@const landing = footprint.has(key)}{@const breaking =
              phase === 'breaking' && (clearRows.includes(r) || clearCols.includes(c))}{@const alt =
              alternativeCells.get(key)}{@const ghost = exploring
              ? !!alt
              : planned && landing && cell !== '#'}
            {@const colorSlot =
              alt?.slot ??
              (ghost || (cell === '#' && phase !== 'idle' && landing)
                ? step?.slot
                : previous.has(key)
                  ? plan.steps[(previous.get(key) ?? 1) - 1]?.slot
                  : undefined)}
            <button
              type="button"
              disabled={editDisabled || !onedit}
              onclick={() => onedit?.(r, c)}
              aria-label={`Board row ${r + 1}, column ${c + 1}`}
              aria-pressed={cell === '#'}
              class={`board-cell${cell === '#' ? ' is-filled' : ''}${cell === '#' && (previous.has(key) || (phase !== 'idle' && landing)) ? ' is-previous' : ''}${ghost ? ' is-ghost' : ''}${colorSlot !== undefined ? ` piece-color-${colorSlot}` : ''}${(phase === 'placing' || phase === 'settling') && landing && cell === '#' ? ' is-landing' : ''}${breaking ? ' is-breaking' : ''}`}
              ><span class="board-cell-label"
                >{ghost
                  ? (alt?.step ?? selected + 1)
                  : (previous.get(key) ??
                    (cell === '#' && phase !== 'idle' && landing
                      ? phase === 'settling'
                        ? completed
                        : completed + 1
                      : ''))}</span
              ></button
            >
          {/each}{/each}
      </div>
    </div>
    <div class="board-meta">
      <span>{caption}</span>
      <div class="board-meta-count">
        <span>{displayRows.join('').split('#').length - 1}/64 filled</span><BoardLegend />
      </div>
    </div>
  </div>
</section>
