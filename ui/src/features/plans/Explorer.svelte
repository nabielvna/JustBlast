<script lang="ts">
  import type { ExplorerView, PieceView } from '../../generated/contracts'
  import PositionPicker from './PositionPicker.svelte'
  import Icon from '../../components/ui/Icon.svelte'
  import PieceShape from '../../components/board/PieceShape.svelte'
  let {
    data,
    pieces,
    completed,
    order,
    positions,
    pageSize,
    loading,
    onfilter,
    onpage,
    onsize,
    onchoose,
    onclose,
    onhover,
    onpositionpreview,
  }: {
    data: ExplorerView | null
    pieces: PieceView[]
    completed: number
    order: number[]
    positions: ([number, number] | null)[]
    pageSize: number
    loading: boolean
    onfilter: (order: number[], positions: ([number, number] | null)[]) => void
    onpage: (page: number) => void
    onsize: (size: number) => void
    onchoose: (id: number) => void
    onclose: () => void
    onhover: (id: number | null) => void
    onpositionpreview: (index: number, position: [number, number] | null) => void
  } = $props()
  let listing = $state(false),
    pageDraft = $state('1')
  $effect(() => {
    pageDraft = String((data?.page ?? 0) + 1)
  })
  function choosePosition(index: number, value: string) {
    const pos = value ? (value.split(':').map(Number) as [number, number]) : null
    onfilter(
      order.slice(0, index + 1),
      Array.from({ length: index + 1 }, (_, i) => (i === index ? pos : (positions[i] ?? null))),
    )
  }
</script>

<section class="plan-inspector moves-explorer" aria-labelledby="moves-explorer-title">
  <header class="explorer-header">
    <div class="explorer-heading">
      <h2 id="moves-explorer-title">
        {completed ? 'Change remaining moves' : 'Explore alternatives'}
      </h2>
      {#if completed}<p class="explorer-intro">
          {completed} placed · {pieces.length} remaining. Placed moves stay unchanged.
        </p>{/if}
      <p class="explorer-total" role="status">
        Solutions available: <strong
          >{loading || !data
            ? '…'
            : `${data.complete ? '' : 'At least '}${data.total.toLocaleString('en-US')}`}</strong
        >
      </p>
    </div>
    <div class="explorer-actions">
      {#if listing}<button
          class="order-clear"
          type="button"
          onclick={() => {
            listing = false
          }}>Edit filters</button
        >{/if}<button
        class="explorer-back"
        type="button"
        aria-label="Back to placement plan"
        onclick={onclose}><Icon name="arrow" />Plan</button
      >
    </div>
  </header>
  <div class={`explorer-body${listing ? ' is-listing' : ''}`}>
    {#if !listing}
      <div class="order-builder" aria-busy={loading}>
        <div class="order-column-heading">
          <h3>Move order</h3>
          <button
            class="order-clear"
            type="button"
            disabled={loading || !order.length}
            onclick={() => onfilter([], [])}>Reset</button
          >
        </div>
        <ol class="move-order">
          {#each pieces as _, index (index)}
            {@const piece = pieces.find((p) => p.slot === order[index])}{@const active =
              index === order.length}{@const automatic =
              order.length === pieces.length && index === pieces.length - 1}
            <li
              class={`order-step${active ? ' is-active' : ''}${piece ? ' is-assigned' : ''}`}
              aria-current={active ? 'step' : undefined}
            >
              <div class="order-position">
                <span class="order-number">{completed + index + 1}</span>
                {#if piece}<span class={`order-piece-preview piece-color-${piece.slot}`}
                    ><PieceShape cells={piece.cells} /></span
                  ><span class="order-piece-name">Slot {piece.slot + 1}</span>
                  <PositionPicker
                    label={`Position for move ${completed + index + 1}, slot ${piece.slot + 1}`}
                    value={positions[index] ?? null}
                    options={data?.positionOptions[index] ?? []}
                    disabled={loading}
                    onchange={(position) => choosePosition(index, position?.join(':') ?? '')}
                    onpreview={(position) => onpositionpreview(index, position)}
                  />
                  {#if automatic}<span class="order-auto">Auto</span>{:else}<button
                      class="order-change"
                      type="button"
                      disabled={loading}
                      aria-label={`Change move ${completed + index + 1}, slot ${piece.slot + 1}, and following moves`}
                      onclick={() => onfilter(order.slice(0, index), positions.slice(0, index))}
                      >Change</button
                    >{/if}
                {:else if active}<h3 class="order-placeholder">Choose a block</h3>{:else}<span
                    class="order-placeholder">Waiting</span
                  >{/if}
              </div>
              {#if active}<div
                  class="slot-choices"
                  role="group"
                  aria-label={`Choose a block for move ${completed + index + 1}`}
                >
                  {#each pieces.filter((p) => !order.includes(p.slot)) as option (option.slot)}{@const available =
                      !!data?.nextCounts[option.slot]}<button
                      class={`slot-choice piece-color-${option.slot}`}
                      type="button"
                      disabled={loading || !available}
                      aria-label={`Slot ${option.slot + 1}${available ? '' : ', no complete solution'}`}
                      onclick={() => onfilter([...order, option.slot], positions)}
                      ><span class="slot-choice-preview"><PieceShape cells={option.cells} /></span
                      ><span class="slot-choice-copy"
                        ><strong>Slot {option.slot + 1}</strong>{#if !available}<small
                            >No solution</small
                          >{/if}</span
                      ></button
                    >{/each}
                </div>{/if}
            </li>
          {/each}
        </ol>
      </div>
      <div class={`order-feedback${data?.total === 0 ? ' is-empty' : ''}`} role="status">
        <span
          >{loading
            ? 'Checking solutions…'
            : !data?.total
              ? 'No complete solution. Try another order or position.'
              : order.length === pieces.length
                ? 'All blocks can fit'
                : 'View all solutions or choose a block to filter.'}</span
        >
      </div>
      <button
        class="button button-primary order-view-solutions"
        type="button"
        disabled={loading || !data?.total}
        onclick={() => {
          listing = true
        }}
        >View {order.length ? '' : 'all '}{data?.total.toLocaleString('en-US')} solutions <Icon
          name="arrow"
        /></button
      >
    {:else}
      {#if data && !data.complete}<p class="explorer-intro" role="status">
          Enumeration reached its limit. Counts are lower bounds; global ranking is not proven.
        </p>{/if}
      <div class="candidate-table" role="region" aria-label="Placement solutions">
        <div class="candidate-columns" aria-hidden="true">
          <span>#</span><span>Placement sequence</span><span title="Algorithm score, highest first"
            >Score ↓</span
          >
        </div>
        <div class="candidate-list" aria-label="Placement solutions" aria-busy={loading}>
          {#each data?.candidates ?? [] as candidate (candidate.id)}<button
              type="button"
              class="candidate-row"
              disabled={loading}
              onmouseenter={() => onhover(candidate.id)}
              onmouseleave={() => onhover(null)}
              onfocus={() => onhover(candidate.id)}
              onblur={() => onhover(null)}
              onclick={() => onchoose(candidate.id)}
              aria-label={`Preview ${candidate.id}, ${candidate.moves
                .slice(completed)
                .map((m) => `slot ${m.piece + 1} row ${m.row + 1} column ${m.column + 1}`)
                .join(', ')}`}
              ><span class="candidate-id">{candidate.id}</span><span class="candidate-steps"
                >{#each candidate.moves.slice(completed) as move, i (move.piece)}{@const piece =
                    pieces.find((p) => p.slot === move.piece) ?? pieces[i]}<span
                    class="candidate-move"
                    >{#if i}<span class="candidate-arrow"><Icon name="arrow" /></span>{/if}<span
                      class={`candidate-position piece-color-${piece?.slot ?? i}`}
                      title={`Slot ${(piece?.slot ?? i) + 1} · ${piece?.name}`}
                      >{#if piece}<PieceShape cells={piece.cells} />{/if}<span
                        class="candidate-coordinate">R{move.row + 1} C{move.column + 1}</span
                      ></span
                    ></span
                  >{/each}</span
              ><span class="candidate-metrics"
                ><strong>{candidate.score?.toFixed(3) ?? '—'}</strong><small
                  >{candidate.clears} breaks</small
                ></span
              ></button
            >{/each}
          {#if !data?.total}<p class="explorer-empty">No solutions for this order.</p>{/if}
        </div>
      </div>
      <footer class="candidate-pagination">
        <select
          class="pagination-size"
          aria-label="Solutions per page"
          value={pageSize}
          disabled={loading}
          onchange={(e) => onsize(Number(e.currentTarget.value))}
          >{#each [5, 10, 15, 20] as size (size)}<option value={size}>{size} / page</option
            >{/each}</select
        >
        <p class="pagination-summary" role="status">
          <strong
            >{data?.total ? data.page * pageSize + 1 : 0}–{Math.min(
              ((data?.page ?? 0) + 1) * pageSize,
              data?.total ?? 0,
            )}</strong
          >
          / {data?.total.toLocaleString('en-US')}
        </p>
        <nav class="pagination-controls" aria-label="Solution pages">
          <button
            class="button"
            type="button"
            aria-label="Previous page"
            disabled={loading || !data?.page}
            onclick={() => onpage((data?.page ?? 0) - 1)}
            ><Icon name="chevron" style="transform:rotate(180deg)" /></button
          >
          <form
            class="pagination-jump"
            onsubmit={(e) => {
              e.preventDefault()
              const next = Number(pageDraft)
              if (Number.isInteger(next))
                onpage(Math.max(0, Math.min((data?.pages ?? 1) - 1, next - 1)))
            }}
          >
            <input
              aria-label="Page number"
              type="text"
              inputmode="numeric"
              pattern="[0-9]*"
              bind:value={pageDraft}
              disabled={loading || !data?.total}
            /><span>/ {data?.pages ?? 1}</span>
          </form>
          <button
            class="button"
            type="button"
            aria-label="Next page"
            disabled={loading || !data || data.page + 1 >= data.pages}
            onclick={() => onpage((data?.page ?? 0) + 1)}><Icon name="chevron" /></button
          >
        </nav>
      </footer>
    {/if}
  </div>
</section>
