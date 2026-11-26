<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { pj, setzeProjekt } from "../../lib/projekt.svelte";
  import Switch from "../../lib/components/Switch.svelte";
  import { t } from "../../lib/i18n.svelte";
  // Allgemein: Name, Beschreibung, Start-Programm, Reiter-Wiederherstellung. Jede Änderung wird sofort geprüft und gespeichert; ein abgelehnter Wert wird gemeldet und die Anzeige springt auf die Datei zurück.
  const i = $derived(pj.info);
  let name = $state(""); let beschr = $state(""); let main = $state(""); let wieder = $state(false); let status = $state(""); let ok = $state(true);
  $effect(() => { if (i) { name = i.name; beschr = i.description; main = i.main ?? ""; wieder = i.restoreTabs; } });
  async function speichern(a: Parameters<typeof setzeProjekt>[0]) { ok = await setzeProjekt(a); status = ok ? t("projekt-gespeichert") : ""; }
</script>

{#if i}
  <div class="f" data-pj="allgemein">
    <label class="fld"><span class="lbl">{t("projekt-name")}</span>
      <input class="inp" data-pj-feld="name" bind:value={name} aria-invalid={!name.trim()} onchange={(e) => void speichern({ name: e.currentTarget.value })} /></label>
    {#if !name.trim()}<p class="small bad" role="status">{t("projekt-name-fehlt")}</p>{/if}
    <label class="fld"><span class="lbl">{t("projekt-beschreibung")}</span>
      <textarea class="inp" rows="3" data-pj-feld="beschreibung" bind:value={beschr} onchange={(e) => void speichern({ description: e.currentTarget.value })}></textarea></label>
    <label class="fld"><span class="lbl">{t("projekt-startdatei")}</span>
      <select class="inp" data-pj-feld="main" bind:value={main} onchange={(e) => void speichern({ main: e.currentTarget.value || null })}>
        <option value="">{t("projekt-startdatei-auto", { f: i.sources[0]?.file ?? t("projekt-kein-programm") })}</option>
        {#each i.sources as q (q.file)}<option value={q.file}>{q.file}</option>{/each}
      </select></label>
    <p class="small dim">{t("projekt-startdatei-hinweis")}</p>
    <div class="fld"><Switch bind:checked={wieder} label={t("projekt-reiter-wiederherstellen")} /></div>
    {#if wieder !== i.restoreTabs}<button type="button" class="btn sm" data-pj-feld="reiter" onclick={() => void speichern({ restoreTabs: wieder })}>{t("projekt-uebernehmen")}</button>{/if}
    <p class="small" class:bad={!ok} role="status" aria-live="polite" data-pj="status">{status}</p>
  </div>
{/if}

<style>
  .f { display: flex; flex-direction: column; gap: 6px; max-width: 620px; } .fld { display: flex; flex-direction: column; gap: 4px; margin: 6px 0; } .lbl { color: var(--dim); font-size: 11px; letter-spacing: .05em; text-transform: uppercase; }
  .inp { width: 100%; } .small { font-size: 12px; margin: 0; } .dim { color: var(--dim); } .bad { color: var(--err, #f85149); }
</style>
