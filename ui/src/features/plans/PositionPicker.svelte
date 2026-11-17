<script lang="ts">
  import { onMount } from 'svelte'
  import Icon from '../../components/ui/Icon.svelte'
  let {
    label,
    value,
    options,
    disabled,
    onchange,
    onpreview,
  }: {
    label: string
    value: [number, number] | null
    options: [number, number][]
    disabled: boolean
    onchange: (value: [number, number] | null) => void
    onpreview: (value: [number, number] | null) => void
  } = $props()
  let trigger: HTMLButtonElement
  let menu = $state<HTMLDivElement>()
  let open = $state(false),
    active = $state(0)
  let layout = $state({ top: 0, left: 0, width: 208, maxHeight: 240 })
  const id = $props.id()
  const key = (p: [number, number] | null) => p?.join(':') ?? ''
  const name = (p: [number, number] | null) => (p ? `R${p[0] + 1} C${p[1] + 1}` : 'Any position')
  let choices = $derived([null, ...options])
  function portal(node: HTMLElement) {
    document.body.appendChild(node)
    return { destroy: () => node.remove() }
  }
  function close() {
    open = false
    onpreview(null)
  }
  function place() {
    if (!open || !trigger) return
    const rect = trigger.getBoundingClientRect()
    const below = innerHeight - rect.bottom - 12,
      above = rect.top - 12
    const maxHeight = Math.min(240, Math.max(80, below >= 160 || below >= above ? below : above))
    const width = Math.min(Math.max(rect.width, 208), innerWidth - 16)
    layout = {
      top: below >= 160 || below >= above ? rect.bottom + 6 : rect.top - maxHeight - 6,
      left: Math.max(8, Math.min(rect.right - width, innerWidth - width - 8)),
      width,
      maxHeight,
    }
  }
  function show() {
    active = Math.max(
      0,
      choices.findIndex((p) => key(p) === key(value)),
    )
    open = true
    place()
  }
  onMount(() => {
    function outside(event: PointerEvent) {
      if (
        open &&
        event.target instanceof Node &&
        !trigger?.contains(event.target) &&
        !menu?.contains(event.target)
      )
        close()
    }
    window.addEventListener('pointerdown', outside)
    window.addEventListener('resize', place)
    window.addEventListener('scroll', place, true)
    return () => {
      window.removeEventListener('pointerdown', outside)
      window.removeEventListener('resize', place)
      window.removeEventListener('scroll', place, true)
      if (open) onpreview(null)
    }
  })
  $effect(() => {
    if (open) {
      const option = menu?.querySelectorAll('[role=option]')[active]
      option?.scrollIntoView({ block: 'nearest' })
    }
  })
  function keydown(event: KeyboardEvent) {
    if (event.key === 'Escape' && open) {
      event.preventDefault()
      event.stopPropagation()
      close()
      return
    }
    if (event.key === 'Tab') {
      if (open) close()
      return
    }
    if (['ArrowDown', 'ArrowUp', 'Home', 'End'].includes(event.key)) {
      event.preventDefault()
      if (!open) {
        show()
        return
      }
      active =
        event.key === 'Home'
          ? 0
          : event.key === 'End'
            ? choices.length - 1
            : (active + (event.key === 'ArrowDown' ? 1 : -1) + choices.length) % choices.length
      onpreview(choices[active] ?? null)
    } else if ((event.key === 'Enter' || event.key === ' ') && open) {
      event.preventDefault()
      const next = choices[active] ?? null
      close()
      onchange(next)
    }
  }
</script>

<div class="move-position-picker">
  <button
    bind:this={trigger}
    type="button"
    class="position-trigger"
    role="combobox"
    aria-label={label}
    aria-haspopup="listbox"
    aria-expanded={open}
    aria-controls={open ? id : undefined}
    aria-activedescendant={open ? `${id}-${active}` : undefined}
    {disabled}
    onclick={() => (open ? close() : show())}
    onkeydown={keydown}
    ><span>{name(value)}</span><span class="position-chevron" aria-hidden="true"></span></button
  >
  {#if open}<div
      use:portal
      bind:this={menu}
      {id}
      class="position-menu"
      style={`top:${layout.top}px;left:${layout.left}px;width:${layout.width}px;max-height:${layout.maxHeight}px`}
      role="listbox"
      tabindex="-1"
      aria-label={label}
      onmouseleave={() => onpreview(null)}
    >
      <div class="position-menu-caption" role="presentation">
        Position <span>Hover to preview</span>
      </div>
      {#each choices as option, index (key(option))}<button
          id={`${id}-${index}`}
          type="button"
          role="option"
          tabindex="-1"
          class={`position-option${active === index ? ' is-highlighted' : ''}`}
          aria-selected={key(option) === key(value)}
          onmouseenter={() => {
            active = index
            onpreview(option)
          }}
          onclick={() => {
            close()
            onchange(option)
          }}
          ><span>{name(option)}</span>{#if key(option) === key(value)}<Icon
              name="check"
            />{/if}</button
        >{/each}
    </div>{/if}
</div>
