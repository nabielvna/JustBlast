<script lang="ts">
  let { cells }: { cells: [number, number][] } = $props()
  let rows = $derived(Math.max(0, ...cells.map((c) => c[0])) + 1)
  let columns = $derived(Math.max(0, ...cells.map((c) => c[1])) + 1)
  let occupied = $derived(new Set(cells.map((c) => `${c[0]}:${c[1]}`)))
</script>

<span class="piece-shape-area" aria-hidden="true">
  <span
    class="piece-shape"
    style={`--piece-rows:${rows};--piece-columns:${columns};grid-template-columns:repeat(${columns},var(--piece-fit-cell))`}
  >
    {#each Array(rows * columns) as _, i (i)}<i
        class:piece-hole={!occupied.has(`${Math.floor(i / columns)}:${i % columns}`)}
      ></i>{/each}
  </span>
</span>
