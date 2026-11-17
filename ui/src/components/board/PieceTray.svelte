<script lang="ts">
  import type { PieceView } from '../../generated/contracts'
  import PieceShape from './PieceShape.svelte'
  let {
    pieces = [],
    consumed = [],
    lockedSlots = consumed,
    active = null,
    onedit,
    disabled = false,
    allSlots = true,
    unknownSlots = [],
    issues = [],
    crops = [],
    onaccept,
  }: {
    pieces?: PieceView[]
    consumed?: number[]
    lockedSlots?: number[]
    active?: number | null
    onedit?: (slot: number) => void
    disabled?: boolean
    allSlots?: boolean
    unknownSlots?: number[]
    issues?: (string | null)[]
    crops?: string[]
    onaccept?: (slots: number[]) => void
  } = $props()
  let slots = $derived(allSlots ? [0, 1, 2] : pieces.map((p) => p.slot))
</script>

<section
  class="piece-tray"
  aria-label="Game pieces"
  style={`grid-template-columns: repeat(${Math.max(1, slots.length)}, minmax(0, 1fr));`}
>
  {#each slots as slot (slot)}
    {@const piece = pieces.find((p) => p.slot === slot)}
    {@const done = consumed.includes(slot)}
    {@const uncertain = !done && unknownSlots.includes(slot)}
    {@const name = uncertain
      ? issues[slot] === 'shape_not_in_catalogue'
        ? 'Not in library'
        : 'Partial read'
      : (piece?.name ?? 'Empty slot')}
    {@const hint = uncertain
      ? piece
        ? 'Check the detected shape'
        : 'Shape unclear · edit or clear'
      : undefined}
    {@const label = `Slot ${slot + 1}, ${name}${done ? ', already placed' : ''}`}
    {@const tileClass = `piece-tile piece-color-${slot}${done ? ' is-consumed' : ''}${active === slot ? ' is-placing' : ''}${uncertain ? ' is-uncertain' : ''}`}
    {#snippet content()}
      <span class="piece-slot">Slot {slot + 1}</span>
      {#if piece}<PieceShape cells={piece.cells} />
      {:else if uncertain && crops[slot]}<span class="piece-shape-area"
          ><img
            class="uncertain-piece-crop"
            src={crops[slot]}
            alt="Unclear piece captured from phone"
          /></span
        >
      {:else}<span class="piece-shape-area" aria-hidden="true"></span>{/if}
      <span class="piece-name">{uncertain ? '⚠ ' : ''}{done ? 'Placed ✓' : name}</span>
      {#if hint}<span class="piece-read-hint">{hint}</span>{/if}
    {/snippet}
    <div class="piece-slot-item">
      {#if onedit}<button
          type="button"
          class={tileClass}
          aria-label={`Edit ${label}`}
          title={hint}
          aria-haspopup="dialog"
          disabled={disabled || lockedSlots.includes(slot)}
          onclick={() => onedit?.(slot)}>{@render content()}</button
        >
      {:else}<figure class={tileClass} aria-label={label}>{@render content()}</figure>{/if}
      {#if uncertain && onaccept}<button
          class="button slot-accept"
          disabled={disabled || !piece}
          onclick={() => onaccept?.([slot])}>Accept slot {slot + 1}</button
        >{/if}
    </div>
  {/each}
  {#if onaccept && unknownSlots.length > 0}<button
      class="button accept-all-slots"
      disabled={disabled || !unknownSlots.some((slot) => pieces.some((p) => p.slot === slot))}
      onclick={() => onaccept?.(unknownSlots.filter((slot) => pieces.some((p) => p.slot === slot)))}
      >Accept all detected pieces</button
    >{/if}
</section>
