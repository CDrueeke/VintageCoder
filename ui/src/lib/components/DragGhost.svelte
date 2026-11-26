<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { dnd } from "../dnd.svelte";
  // Kleine Marke am Zeiger, solange etwas im Fenster gezogen wird (Reiter, Datei, Schritt)
</script>

{#if dnd.active && dnd.label}<div class="ghost" aria-hidden="true" style={`left:${dnd.x + 14}px;top:${dnd.y + 14}px`}>{dnd.label}</div>{/if}

<style>
  .ghost { position: fixed; z-index: 9999; pointer-events: none; padding: 3px 9px; border-radius: 8px; background: var(--card); color: var(--text); border: 1px solid var(--accent); font-size: 12px; box-shadow: 0 6px 18px rgb(0 0 0 / 0.3); max-width: 240px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  :global(body.vc-dragging) { user-select: none; -webkit-user-select: none; cursor: grabbing; }
  :global(body.vc-dragging *) { cursor: grabbing !important; }
</style>
