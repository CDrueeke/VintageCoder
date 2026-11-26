<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { pj, setzeProjekt } from "../../lib/projekt.svelte";
  import Segment from "../../lib/components/Segment.svelte";
  import { call } from "../../lib/api";
  import { currentLang, t } from "../../lib/i18n.svelte";
  // Ziele: die Zielsysteme des Projekts mit wählbarer Variante (PAL/NTSC); Emulator und Startziel stehen in den Einstellungen. Zielsysteme hinzufügen oder entfernen folgt.
  const i = $derived(pj.info);
  let systeme = $state<{ id: string; name: string }[]>([]); let neu = $state("");
  $effect(() => { void currentLang(); if (i) void call("ziele_info").then((r) => (systeme = r.systems.map((x) => ({ id: x.id, name: x.name })))).catch(() => {}); });
  let maschinen = $state<{ id: string; name: string }[]>([]);
  $effect(() => { if (i && !maschinen.length) void call("settings_get").then((r) => { maschinen = ((r.settings as { machines?: { id: string; name?: string; kind?: string }[] }).machines ?? []).map((m) => ({ id: m.id, name: m.name || m.id })); }).catch(() => {}); });
  const frei = $derived(systeme.filter((x) => !i?.targets.some((z) => z.system === x.id || z.id === x.id)));
</script>

{#if i}
  <div class="l" data-pj="ziele">
    {#each i.targets as z (z.id)}
      <section class="k" data-pj-ziel={z.id} aria-label={z.system.toUpperCase()}>
        <h3>{z.system.toUpperCase()}{#if z.id !== z.system} <span class="dim">({z.id})</span>{/if}</h3>
        {#if z.variants.length}
          <Segment small label={t("projekt-variante-von", { s: z.system.toUpperCase() })} items={z.variants.map((v) => ({ id: v, label: v.toUpperCase() }))} value={z.variant || z.variants[0]!} onselect={(v) => void setzeProjekt({ variant: { id: z.id, variant: v } })} />
          <p class="small dim">{t("projekt-variante-hinweis")}</p>
        {:else}<p class="small dim">{t("projekt-keine-variante")}</p>{/if}
        <div class="row"><label class="small dim">{t("projekt-illegale")}<select class="inp" data-pj="illegale" aria-label={t("projekt-illegale")} value={z.illegals} onchange={(e) => void setzeProjekt({ targetSet: { id: z.id, illegals: e.currentTarget.value } })}><option value="">{t("projekt-illegale-auto")}</option><option value="deny">{t("projekt-illegale-deny")}</option><option value="warn">{t("projekt-illegale-warn")}</option><option value="allow">{t("projekt-illegale-allow")}</option></select></label>
          <label class="small dim">{t("projekt-startziel")}<select class="inp" data-pj="startziel" aria-label={t("projekt-startziel")} value={z.startOn} onchange={(e) => void setzeProjekt({ targetSet: { id: z.id, startOn: e.currentTarget.value } })}><option value="">{t("projekt-startziel-global")}</option><option value="@emulator">{t("projekt-startziel-emulator")}</option>{#each maschinen as m (m.id)}<option value={m.id}>{m.name}</option>{/each}</select></label></div>
        {#if z.emulator}<p class="small dim">{t("projekt-emulator-dieses-ziels", { e: z.emulator })}</p>{/if}
        <div class="row"><button type="button" class="btn sm" data-pj="ziel-weg" disabled={i.targets.length < 2} title={i.targets.length < 2 ? t("projekt-letztes-ziel") : ""} onclick={() => void setzeProjekt({ removeTarget: z.id })}>{t("projekt-ziel-entfernen")}</button></div>
      </section>
    {:else}<p class="dim">{t("projekt-kein-ziel")}</p>{/each}
    {#if frei.length}<div class="row"><select class="inp" data-pj="ziel-neu" aria-label={t("projekt-ziel-hinzu")} bind:value={neu}><option value="">{t("projekt-ziel-hinzu")}</option>{#each frei as x (x.id)}<option value={x.id}>{x.name}</option>{/each}</select><button type="button" class="btn sm" data-pj="ziel-hinzu" disabled={!neu} onclick={() => { void setzeProjekt({ addTarget: neu }); neu = ""; }}>{t("projekt-ziel-hinzufuegen")}</button></div><p class="small dim">{t("projekt-ziel-neu-hinweis")}</p>{/if}
  </div>
{/if}

<style>
  .inp { height: 28px; padding: 0 8px; border-radius: var(--r-ctl); border: 1px solid var(--line2); background: var(--raise); color: var(--text); font: inherit; } label { display: flex; flex-direction: column; gap: 3px; }
  .l { display: flex; flex-direction: column; gap: 12px; max-width: 620px; } .k { display: flex; flex-direction: column; gap: 6px; padding: 12px 14px; border-radius: 12px; border: 1px solid var(--line2); background: var(--card); }
  .row { display: flex; gap: 8px; align-items: center; } h3 { margin: 0; font-size: 13px; } .small { font-size: 12px; margin: 0; } .dim { color: var(--dim); font-weight: normal; }
</style>
