<script lang="ts">
  import type { PlanView, PieceView } from '../../generated/contracts'
  import Icon from '../../components/ui/Icon.svelte'
  import PieceShape from '../../components/board/PieceShape.svelte'
  let {
    plan,
    pieces,
    completed,
    selected,
    running,
    phase,
    candidateId,
    previewId,
    onreplace,
    oncancel,
    onrestore,
    onexplore,
    onplay,
    onphone,
    phoneDisabled = true,
    actualCompleted = 0,
    onselect,
    busy = false,
    physical = false,
  }: {
    plan: PlanView
    pieces: PieceView[]
    completed: number
    selected: number
    running: boolean
    phase: string
    candidateId: number | null
    previewId: number | null
    onreplace: () => void
    oncancel: () => void
    onrestore: () => void
    onexplore: () => void
    onplay: (index: number) => void
    onphone: (index: number) => void
    phoneDisabled?: boolean
    actualCompleted?: number
    onselect: (index: number) => void
    busy?: boolean
    physical?: boolean
  } = $props()
  let breaks = $derived(
    plan.steps.reduce((n, s) => n + s.clearedRows.length + s.clearedColumns.length, 0),
  )
</script>

<section class="plan-inspector" aria-label="Placement plan evaluation">
  <header class="plan-header">
    {#if previewId !== null || candidateId !== null}
      <div class={`plan-candidate-reference${previewId !== null ? ' is-preview' : ''}`}>
        <div class="plan-candidate-copy">
          <strong
            >Solution {previewId ?? candidateId}
            <span class="plan-candidate-badge">{previewId !== null ? 'Preview' : 'Active'}</span
            ></strong
          >
          <p role="status" id="plan-choice-help">
            {previewId !== null
              ? completed
                ? `Replace the remaining ${plan.steps.length - completed} moves? The ${completed} completed moves stay unchanged.`
                : 'Replace the algorithm plan with this solution?'
              : 'Solution replaced. You can still return to the algorithm plan.'}
          </p>
        </div>
        <div class="plan-choice-actions">
          <button
            class="button plan-algorithm-button"
            type="button"
            disabled={busy}
            onclick={onrestore}><Icon name="reset" />Algorithm plan</button
          >{#if previewId !== null}{#if completed}<button
                class="button"
                type="button"
                onclick={oncancel}>Keep current plan</button
              >{/if}<button
              class="button plan-replace-button"
              type="button"
              disabled={busy}
              onclick={onreplace}><Icon name="check" />Replace</button
            >{/if}
        </div>
        {#if previewId !== null}<p class="plan-preview-help">
            Replace to enable playback, or keep the current plan.
          </p>{/if}
      </div>
    {/if}
    <div class="plan-heading">
      <h2>Placement plan</h2>
      <p class="plan-subline">
        <span
          title="Weighted plan score under hypothetical combo models; model points are not the actual game score"
          >Score <strong>{plan.complete ? plan.metrics.value.toFixed(3) : 'Provisional'}</strong
          ></span
        ><span class="plan-summary-dot" aria-hidden="true">·</span><span
          >{breaks} {breaks === 1 ? 'break' : 'breaks'}</span
        >
      </p>
    </div>
    <button
      class="plan-explore-button"
      type="button"
      disabled={busy || running || phase !== 'idle' || completed === plan.steps.length}
      aria-label="Explore alternative solutions"
      aria-controls="plan-panel"
      onclick={onexplore}><span>Explore alternatives</span><Icon name="arrow" /></button
    >
  </header>
  {#if !plan.complete}<p class="plan-preview-help" role="status">
      Search reached its limit. This plan is provisional and cannot advance the cycle or move your
      phone.
    </p>{/if}
  {#if plan.steps.length < plan.requestedPieces}<p class="plan-preview-help" role="status">
      Only {plan.steps.length} of {plan.requestedPieces} pieces were placed. Edit the input or Scan again
      for a complete plan.
    </p>{/if}
  {#if actualCompleted > 0}<p class="plan-progress-note" role="status">
      <Icon name="lock" />{actualCompleted} played on phone · locked. Reset only clears local preview.
    </p>{/if}
  <div class="plan-list">
    {#each plan.steps as step, index (step.slot)}
      {@const piece = pieces.find((p) => p.slot === step.slot)}{@const done =
        index < completed}{@const canPlay =
        !busy && previewId === null && !running && phase === 'idle' && !done}
      <div
        class={`plan-row piece-color-${step.slot}${done ? ' is-done' : ''}${!done && selected === index ? ' is-selected' : ''}${done ? ' is-locked' : ''}`}
      >
        <button
          type="button"
          class="plan-row-select"
          disabled={busy || done || running || phase !== 'idle'}
          aria-pressed={!done && selected === index}
          aria-label={`Step ${index + 1}, slot ${step.slot + 1}, ${piece?.name}, row ${step.movement.row + 1}, column ${step.movement.column + 1}${done ? ', placed' : ''}`}
          onclick={() => onselect(index)}
          ><span class="plan-step-number">{done ? '✓' : String(index + 1).padStart(2, '0')}</span
          >{#if piece}<PieceShape cells={piece.cells} />{/if}<span class="plan-piece"
            ><span>Slot {step.slot + 1}</span><small>{piece?.name}</small></span
          ><span class="plan-destination"
            ><span class="coordinate">R{step.movement.row + 1} C{step.movement.column + 1}</span
            ><small
              >{selected === index && phase === 'placing'
                ? 'Placing'
                : selected === index && phase === 'breaking'
                  ? 'Clearing'
                  : done
                    ? index < actualCompleted
                      ? 'Played on phone'
                      : 'Previewed'
                    : selected === index
                      ? index === completed
                        ? 'Next'
                        : 'Selected'
                      : 'Pending'}</small
            ></span
          ></button
        >
        <button
          type="button"
          class="plan-row-action"
          disabled={!canPlay || (physical && (phoneDisabled || index < actualCompleted))}
          aria-label={physical
            ? `Play through step ${index + 1} on phone`
            : `Preview through step ${index + 1}`}
          data-tooltip={done
            ? index < actualCompleted
              ? 'Already played on phone. This move cannot be reset or replayed.'
              : 'Already previewed. Use Reset preview in the console to preview again.'
            : previewId !== null
              ? 'Replace this solution to enable playback'
              : physical
                ? `Execute remaining phone moves through step ${index + 1}`
                : `Preview steps ${completed + 1}–${index + 1} in order`}
          onclick={() => (physical ? onphone(index) : onplay(index))}
          ><Icon name={done && index < actualCompleted ? 'lock' : 'play'} /></button
        >
      </div>
    {/each}
  </div>
</section>
