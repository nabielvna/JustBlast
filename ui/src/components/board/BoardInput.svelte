<script lang="ts">
  import type { WorkspaceView } from '../../generated/contracts'
  import Icon from '../ui/Icon.svelte'
  import BoardLegend from './BoardLegend.svelte'

  let {
    snapshot,
    disabled,
    onreplace,
    inputRows,
    editable = true,
    unknownBoard = false,
  }: {
    snapshot: WorkspaceView | null
    disabled: boolean
    onreplace: (rows: WorkspaceView['board']) => void
    inputRows?: WorkspaceView['board']
    editable?: boolean
    unknownBoard?: boolean
  } = $props()

  const empty = Array<string>(8).fill('........') as WorkspaceView['board']
  let rows = $derived(inputRows ?? snapshot?.currentBoard ?? empty)
  let manual = $derived(editable)

  function toggle(row: number, column: number) {
    const next = [...rows] as WorkspaceView['board']
    const line = next[row]!
    next[row] = line.slice(0, column) + (line[column] === '#' ? '.' : '#') + line.slice(column + 1)
    onreplace(next)
  }
</script>

<section class="board-section" aria-label={manual ? 'Manual board input' : 'Scanned board'}>
  <div class="board-area">
    <div class="board-layout">
      <div class="board-columns" aria-hidden="true">
        {#each rows as _, index (index)}<span class="axis-label">{index + 1}</span>{/each}
      </div>
      <div class="board-rows" aria-hidden="true">
        {#each rows as _, index (index)}<span class="axis-label">{index + 1}</span>{/each}
      </div>
      <div
        class="board manual-grid"
        role="group"
        aria-label={manual ? 'Edit 8 by 8 board' : 'Board awaiting scan'}
      >
        {#each rows as line, row (row)}
          {#each [...line] as cell, column (column)}
            <button
              type="button"
              class={`board-cell${cell === '#' ? ' is-filled' : unknownBoard ? ' is-unknown' : ''}`}
              aria-label={`Board row ${row + 1}, column ${column + 1}`}
              title={unknownBoard ? 'Unreadable board: review this cell' : undefined}
              aria-pressed={cell === '#'}
              disabled={disabled || !manual || !snapshot?.capabilities.boardInput}
              onclick={() => toggle(row, column)}
            ></button>
          {/each}
        {/each}
      </div>
    </div>
    <div class="board-meta">
      <span
        >{unknownBoard
          ? 'Board unreadable · review all cells before Apply'
          : manual
            ? 'Click a cell to fill or clear it'
            : 'Scan your phone to load the board'}</span
      >
      <div class="board-meta-count">
        <span>{rows.join('').split('#').length - 1}/64 filled</span>
        <button
          class="legend-info-button board-clear-button"
          type="button"
          aria-label="Clear board"
          disabled={disabled || !manual}
          onclick={() => onreplace([...empty])}><Icon name="clear" /></button
        >
        <BoardLegend />
      </div>
    </div>
  </div>
</section>
