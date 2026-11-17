<script lang="ts">
  type Name =
    | 'lock'
    | 'search'
    | 'scan'
    | 'settings'
    | 'play'
    | 'check'
    | 'clear'
    | 'info'
    | 'close'
    | 'grid'
    | 'reset'
    | 'arrow'
    | 'pause'
    | 'stop'
    | 'force-stop'
    | 'chevron'
  let { name, style }: { name: Name; style?: string } = $props()
</script>

<svg
  class="icon"
  {style}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width="1.8"
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
>
  {#if name === 'lock'}
    <rect x="5" y="10" width="14" height="11" rx="2" /><path d="M8 10V7a4 4 0 0 1 8 0v3m-4 5v2" />
  {:else if name === 'search'}
    <circle cx="10.5" cy="10.5" r="6.5" /><path d="m16 16 5 5" />
  {:else if name === 'scan'}
    <path
      d="M8 3H4a1 1 0 0 0-1 1v4m13-5h4a1 1 0 0 1 1 1v4M3 16v4a1 1 0 0 0 1 1h4m8 0h4a1 1 0 0 0 1-1v-4M5 12h14"
    />
  {:else if name === 'settings'}
    <path
      d="M12 3v3m0 12v3M3 12h3m12 0h3M5.6 5.6l2.1 2.1m8.6 8.6 2.1 2.1M5.6 18.4l2.1-2.1m8.6-8.6 2.1-2.1"
    />
    <circle cx="12" cy="12" r="6" /><circle cx="12" cy="12" r="2" />
  {:else if name === 'play'}
    <path d="m9 5 11 7-11 7Z" />
  {:else if name === 'chevron'}
    <path d="m9 6 6 6-6 6" />
  {:else if name === 'reset'}
    <path d="M3 10a9 9 0 1 1 2 8M3 4v6h6" />
  {:else if name === 'arrow'}
    <path d="M5 12h14m-6-6 6 6-6 6" />
  {:else if name === 'pause'}
    <path d="M8 5v14M16 5v14" />
  {:else if name === 'force-stop'}
    <path d="M8 3h8l5 5v8l-5 5H8l-5-5V8Z" /><path d="m9 9 6 6m0-6-6 6" />
  {:else if name === 'stop'}
    <rect x="6" y="6" width="12" height="12" rx="2" />
  {:else if name === 'check'}
    <path d="m5 12 4 4L19 6" />
  {:else if name === 'clear'}
    <path
      d="m4 14 9-10a2 2 0 0 1 3 0l4 4a2 2 0 0 1 0 3l-8 9H8l-4-4a1.5 1.5 0 0 1 0-2ZM10 8l8 8M12 20h9"
    />
  {:else if name === 'info'}
    <circle cx="12" cy="12" r="9" /><path d="M12 11v6m0-10v.01" />
  {:else if name === 'grid'}
    <rect x="4" y="4" width="6" height="6" rx="1.5" /><rect
      x="14"
      y="4"
      width="6"
      height="6"
      rx="1.5"
    />
    <rect x="4" y="14" width="6" height="6" rx="1.5" /><rect
      x="14"
      y="14"
      width="6"
      height="6"
      rx="1.5"
    />
  {:else}
    <path d="m6 6 12 12M6 18 18 6" />
  {/if}
</svg>
