<script lang="ts">
  import Icon from '../ui/Icon.svelte'

  let open = $state(false)
  let root: HTMLDivElement
  const id = $props.id()
  const entries = [
    ['is-filled', 'Existing blocks'],
    ['', 'Empty cells'],
    ['is-previous', 'Previous placements'],
    ['is-ghost', 'Planned placement'],
    ['is-break', 'Row/column will clear'],
    ['is-step', 'Numbers = step order'],
  ]
</script>

<svelte:window
  onpointerdown={(event) => {
    if (event.target instanceof Node && !root.contains(event.target)) open = false
  }}
  onkeydown={(event) => {
    if (event.key === 'Escape') open = false
  }}
/>
<div class="legend-help" bind:this={root}>
  <button
    class="legend-info-button"
    type="button"
    aria-label="Color and symbol guide"
    aria-haspopup="dialog"
    aria-expanded={open}
    aria-controls={open ? id : undefined}
    onclick={() => (open = !open)}><Icon name="info" /></button
  >
  {#if open}
    <div class="legend-popover" {id} role="dialog" aria-labelledby={`${id}-title`} tabindex="-1">
      <div class="legend-popover-header">
        <h3 id={`${id}-title`}>Colors &amp; symbols</h3>
        <button
          class="legend-close-button"
          type="button"
          aria-label="Close guide"
          onclick={() => (open = false)}><Icon name="close" /></button
        >
      </div>
      <ul class="board-legend" aria-label="Board legend">
        {#each entries as [kind, label] (label)}
          <li>
            <i class={`legend-swatch ${kind}`} aria-hidden="true">{kind === 'is-step' ? '1' : ''}</i
            ><span>{label}</span>
          </li>
        {/each}
      </ul>
    </div>
  {/if}
</div>
