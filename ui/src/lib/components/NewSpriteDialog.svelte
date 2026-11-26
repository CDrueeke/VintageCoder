<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { call } from "../api";
  import Dialog from "./Dialog.svelte";
  import { reloadTree } from "../editor.svelte";
  import { openSprite } from "../sprite.svelte";
  import { fail } from "../state.svelte";
  import { t } from "../i18n.svelte";
  // Neue Sprite-Datei aus einem Profil des Projekts (Sprites und Player/Missile); danach öffnet sie im Sprite-Editor
  let { open = $bindable(false), kinds = ["sprite", "pmg"], title = "sprite-neu-titel", defaultName = "sprite" }: { open?: boolean; kinds?: string[]; title?: string; defaultName?: string } = $props();
  let profiles = $state<{ id: string; name: string; kind: string; target: string }[]>([]);
  let profile = $state(""); let name = $state(""); let dir = $state("graphics");
  const ok = $derived(/^[A-Za-z0-9_-]+$/.test(name) && profile !== "");
  $effect(() => { if (open && name === "") name = defaultName; });
  $effect(() => { if (open) void call("media_profiles", { kinds }).then((p) => { profiles = p; if (!p.some((x) => x.id === profile)) profile = p[0]?.id ?? ""; }).catch(fail); });
  async function create() {
    if (!ok) return;
    try { const r = await call("media_new", { name, profile, dir }); open = false; await reloadTree(); await openSprite(r.path); } catch (e) { fail(e); }
  }
</script>

{#if open}
  <Dialog bind:open title={t(title)}>
    {#snippet children()}
      <label class="fld"><span class="lbl">{t("sprite-neu-profil")}</span><select class="inp" data-new="profil" bind:value={profile}>{#each profiles as p (p.id)}<option value={p.id}>{p.name}</option>{/each}</select></label>
      <label class="fld"><span class="lbl">{t("sprite-neu-name")}</span><input class="inp mono" data-new="name" bind:value={name} aria-invalid={!/^[A-Za-z0-9_-]+$/.test(name)} /></label>
      {#if !/^[A-Za-z0-9_-]+$/.test(name)}<p class="small bad" role="status">{t("sprite-neu-name-ungueltig")}</p>{/if}
      <label class="fld"><span class="lbl">{t("sprite-neu-ordner")}</span><input class="inp mono" data-new="dir" bind:value={dir} /></label>
    {/snippet}
    {#snippet footer()}
      <button type="button" class="btn" onclick={() => (open = false)}>{t("dialog-abbrechen")}</button>
      <button type="button" class="btn pri" data-new="anlegen" disabled={!ok} onclick={() => void create()}>{t("sprite-neu-anlegen")}</button>
    {/snippet}
  </Dialog>
{/if}

<style> .fld { display: flex; flex-direction: column; gap: 4px; margin: 8px 0; } .lbl { color: var(--dim); font-size: 11px; letter-spacing: .05em; text-transform: uppercase; } .inp { width: 100%; } .small { font-size: 12px; } .bad { color: var(--err); } </style>
