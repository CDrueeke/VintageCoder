<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  // Kürzelanzeige: je Taste ein <kbd>; für die Sprachausgabe als Ganzes „Befehl+S“ lesbar (Text steht schon in den Teilen)
  let { parts }: { parts: string[] } = $props();
</script>

{#if parts.length > 0}
  <span class="kbdgroup">{#each parts as p, i (i)}<kbd class="kbd">{p}</kbd>{/each}</span>
{/if}
