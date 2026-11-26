<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import type { Snippet } from "svelte";
  // Dialog auf Grundlage des Elements <dialog>: modal, Fokus bleibt innen, Esc schließt und gibt den Fokus zurück
  let { open = $bindable(false), title, children, footer, wide = false }: { open?: boolean; title: string; children: Snippet; footer?: Snippet; wide?: boolean } = $props();
  let el: HTMLDialogElement;
  const id = `dlg-${Math.random().toString(36).slice(2, 8)}`;
  $effect(() => {
    if (open && !el.open) el.showModal();
    if (!open && el.open) el.close();
  });
</script>

<dialog bind:this={el} class:wide aria-labelledby={id} onclose={() => (open = false)}>
  <div class="dh" {id}>{title}</div>
  <div class="db">{@render children()}</div>
  {#if footer}<div class="df">{@render footer()}</div>{/if}
</dialog>
