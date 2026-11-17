<script lang="ts">
  import { onMount, tick, type ComponentProps } from 'svelte'
  import Icon from './Icon.svelte'
  let {
    icon,
    label,
    hint = '',
    disabled = false,
    primary = false,
    danger = false,
    alignEnd = false,
    popup,
    onpress,
  }: {
    icon: ComponentProps<typeof Icon>['name']
    label: string
    hint?: string
    disabled?: boolean
    primary?: boolean
    danger?: boolean
    alignEnd?: boolean
    popup?: 'dialog'
    onpress: () => void
  } = $props()
  const tooltipId = $props.id()
  let anchor: HTMLSpanElement
  let tooltip = $state<HTMLDivElement>()
  let open = $state(false)
  let placed = $state(false)
  let left = $state(0),
    top = $state(0)
  let timer: ReturnType<typeof setTimeout> | undefined
  function hide() {
    clearTimeout(timer)
    open = false
    placed = false
  }
  function show(immediate = false) {
    clearTimeout(timer)
    const reveal = () => {
      window.dispatchEvent(new CustomEvent('justblast:tooltip', { detail: tooltipId }))
      placed = false
      open = true
    }
    if (immediate) reveal()
    else timer = setTimeout(reveal, 220)
  }
  async function place(labelText = label, hintText = hint) {
    await tick()
    if (!open || !tooltip || labelText !== label || hintText !== hint) return
    const a = anchor.getBoundingClientRect(),
      t = tooltip.getBoundingClientRect()
    if (a.bottom < 0 || a.top > innerHeight) {
      hide()
      return
    }
    left = Math.max(8, Math.min(innerWidth - t.width - 8, a.left + (a.width - t.width) / 2))
    top =
      a.top - t.height - 8 >= 8
        ? a.top - t.height - 8
        : Math.min(innerHeight - t.height - 8, a.bottom + 8)
    placed = true
  }
  function portal(node: HTMLElement) {
    document.body.append(node)
    return {
      destroy() {
        node.remove()
      },
    }
  }
  $effect(() => {
    if (open) {
      void place(label, hint)
    }
  })
  onMount(() => {
    const closeOther = (event: Event) => {
      if ((event as CustomEvent<string>).detail !== tooltipId) hide()
    }
    const reposition = () => {
      if (open) void place()
    }
    window.addEventListener('justblast:tooltip', closeOther)
    window.addEventListener('scroll', reposition, true)
    window.addEventListener('resize', reposition)
    return () => {
      hide()
      window.removeEventListener('justblast:tooltip', closeOther)
      window.removeEventListener('scroll', reposition, true)
      window.removeEventListener('resize', reposition)
    }
  })
</script>

<span
  class="control-tooltip-anchor"
  role="presentation"
  class:push-end={alignEnd}
  bind:this={anchor}
  onpointerenter={() => show()}
  onpointerleave={() => {
    if (!anchor.contains(document.activeElement)) hide()
  }}
  onfocusin={() => show(true)}
  onfocusout={hide}
>
  <button
    type="button"
    class="button icon-control"
    class:button-primary={primary}
    class:is-danger={danger}
    class:is-disabled={disabled}
    aria-disabled={disabled}
    aria-label={label}
    aria-haspopup={popup}
    aria-describedby={open ? tooltipId : undefined}
    onkeydown={(e) => {
      if (e.key === 'Escape') hide()
    }}
    onclick={() => {
      if (disabled) return
      hide()
      onpress()
    }}><Icon name={icon} /></button
  >
</span>
{#if open}<div
    use:portal
    bind:this={tooltip}
    id={tooltipId}
    role="tooltip"
    class="control-tooltip"
    style:left={`${left}px`}
    style:top={`${top}px`}
    style:visibility={placed ? 'visible' : 'hidden'}
  >
    <strong>{label}</strong>{#if hint}<span>{hint}</span>{/if}
  </div>{/if}

<style>
  .control-tooltip-anchor {
    display: inline-flex;
    flex: 0 1 36px;
    min-width: 28px;
  }
  .push-end {
    margin-inline-start: auto;
  }
  .icon-control {
    width: 36px;
    height: 36px;
    min-height: 36px;
    min-width: 0;
    padding: 0;
    flex: 1;
  }
  .icon-control :global(.icon) {
    width: 18px;
    height: 18px;
  }
  .icon-control.is-disabled {
    opacity: 0.4;
    box-shadow: none;
    cursor: default;
    pointer-events: none;
  }
  .is-danger {
    color: var(--danger-soft);
    border-color: var(--danger);
    background: var(--danger);
  }
  .control-tooltip {
    position: fixed;
    z-index: 1100;
    display: grid;
    gap: 4px;
    width: max-content;
    max-width: min(240px, calc(100vw - 16px));
    padding: 9px 12px;
    border: 1px solid var(--rule);
    border-radius: 6px;
    background: var(--surface);
    color: var(--ink);
    box-shadow: 0 4px 16px color-mix(in srgb, var(--shadow) 20%, transparent);
    font:
      500 11px/1.45 'DM Sans',
      sans-serif;
    pointer-events: none;
  }
  .control-tooltip strong {
    font-size: 12px;
    font-weight: 500;
  }
</style>
