<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import Dialog from "./Dialog.svelte";
  import Segment from "./Segment.svelte";
  import { animFromMarked, sp } from "../sprite.svelte";
  import { t } from "../i18n.svelte";
  // Animation aus den markierten Bildern der Bilderliste: Name, Ablauf, Tempo (Bilder je Sekunde; der Kern speichert Ticks zu 1/50 s)
  const suggestion = $derived.by(() => { const names = new Set((sp.view?.anims ?? []).map((a) => a.name)); let n = "anim", k = 1; while (names.has(n)) { k++; n = `anim${k}`; } return n; });
  let name = $state(""); let mode = $state("loop"); let fps = $state(8);
  $effect(() => { if (sp.animDialog) { name = suggestion; mode = "loop"; fps = 8; } });
  const taken = $derived((sp.view?.anims ?? []).some((a) => a.name === name));
  const valid = $derived(/^[A-Za-z_][A-Za-z0-9_]*$/.test(name) && !taken);
  const ticks = $derived(Math.max(1, Math.min(255, Math.round(50 / Math.max(1, Math.min(50, Number(fps) || 8))))));
  async function create() { if (valid) await animFromMarked(name, mode, Number(fps) || 8); }
</script>

{#if sp.animDialog}
  <Dialog bind:open={sp.animDialog} title={t("sprite-anim-aus-titel", { n: sp.marked.length })}>
    {#snippet children()}
      <label class="fld"><span class="lbl">{t("sprite-anim-name")}</span><input class="inp mono" data-anim-from="name" bind:value={name} aria-invalid={!valid} /></label>
      {#if !valid}<p class="small bad" role="status">{taken ? t("sprite-anim-aus-vergeben") : t("sprite-anim-aus-ungueltig")}</p>{/if}
      <div class="fld"><span class="lbl">{t("sprite-anim-aus-ablauf")}</span>
        <Segment items={[{ id: "loop", label: t("sprite-ablauf-loop") }, { id: "pingpong", label: t("sprite-ablauf-pingpong") }, { id: "once", label: t("sprite-ablauf-once") }]} value={mode} onselect={(id) => (mode = id)} label={t("sprite-ablauf")} small /></div>
      <label class="fld"><span class="lbl">{t("sprite-anim-aus-tempo")}</span><input class="inp" type="number" min="1" max="50" data-anim-from="fps" bind:value={fps} />
        <span class="small dim" data-anim-from="ticks">{t("sprite-anim-aus-ticks", { t: ticks })}</span></label>
      <p class="small dim">{t("sprite-anim-aus-hinweis")}</p>
    {/snippet}
    {#snippet footer()}
      <button type="button" class="btn" onclick={() => (sp.animDialog = false)}>{t("dialog-abbrechen")}</button>
      <button type="button" class="btn pri" data-anim-from="create" disabled={!valid} onclick={() => void create()}>{t("sprite-anim-aus-erzeugen")}</button>
    {/snippet}
  </Dialog>
{/if}

<style> .fld { display: flex; flex-direction: column; gap: 4px; margin: 8px 0; } .lbl { color: var(--dim); font-size: 11px; letter-spacing: .05em; text-transform: uppercase; } .inp { width: 100%; } .small { font-size: 12px; } .bad { color: var(--err); } .dim { color: var(--dim); } </style>
