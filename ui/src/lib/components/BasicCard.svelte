<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { t } from "../i18n.svelte";
  import { bs, saveBasic } from "../basic.svelte";
  import Card from "./Card.svelte";
  import Switch from "./Switch.svelte";
  // Karte „BASIC“ in den Projekt-Einstellungen: Start und Schrittweite der Zeilennummern, Bauoption „kompakt“, Syntaxprüfung (ein/aus, abgeschaltete Meldungen)
  const s = $derived(bs.state);
  let start = $state(10), step = $state(10), compact = $state(false), check = $state(true), disabled = $state("");
  // Felder aus dem Zustand übernehmen, solange das Formular noch dem zuletzt übernommenen Stand entspricht (nichts vom Anwender geändert, auch nicht am Schalter)
  let loaded = "";
  const form = () => JSON.stringify([start, step, compact, check, disabled.split(/[,\s]+/).map((c) => c.trim().toUpperCase()).filter(Boolean)]);
  $effect(() => {
    const st = s; if (!st) return;
    const sig = JSON.stringify([st.start, st.step, st.compact, st.check, st.disabled]);
    if (sig === loaded) return;
    if (loaded === "" || form() === loaded || form() === sig) { start = st.start; step = st.step; compact = st.compact; check = st.check; disabled = st.disabled.join(", "); loaded = sig; }
  });
  const codes = $derived(disabled.split(/[,\s]+/).map((c) => c.trim().toUpperCase()).filter(Boolean));
  const rangeBad = $derived(!Number.isInteger(start) || !Number.isInteger(step) || step < 1 || start < 0 || start > (s?.maxLine ?? 63999));
  const dirty = $derived(!!s && (start !== s.start || step !== s.step || compact !== s.compact || check !== s.check || JSON.stringify(codes) !== JSON.stringify(s.disabled)));
  async function save() { if (rangeBad) return; await saveBasic({ start, step, compact, check, disabled: codes }); }
</script>

{#if s?.present}
  <Card title={t("basicui-karte-titel")} class="bcard">
    <div class="body" data-testid="basic-card">
      <h3>{t("basicui-nummerierung-titel")}</h3>
      <p class="small dim">{t("basicui-nummerierung-hinweis")}</p>
      <div class="row">
        <label class="lbl">{t("basicui-feld-start")}<input class="inp mono" type="number" min="0" max={s.maxLine} bind:value={start} aria-invalid={rangeBad} data-testid="basic-start" /></label>
        <label class="lbl">{t("basicui-feld-schritt")}<input class="inp mono" type="number" min="1" bind:value={step} aria-invalid={rangeBad} data-testid="basic-step" /></label>
      </div>
      {#if rangeBad}<p class="small err" role="alert" data-testid="basic-fehler">{t("basicui-fehler-bereich", { max: s.maxLine })}</p>{/if}

      <div class="swrow" data-testid="basic-kompakt"><span><Switch bind:checked={compact} label={t("basicui-kompakt")} /></span></div>
      <p class="small dim">{t("basicui-kompakt-hinweis")}</p>

      <div class="swrow" data-testid="basic-pruefung"><span><Switch bind:checked={check} label={t("basicui-pruefung")} /></span></div>
      <p class="small dim">{t("basicui-pruefung-hinweis")}</p>
      <label class="lbl">{t("basicui-abgeschaltet")}<input class="inp mono" bind:value={disabled} disabled={!check} aria-describedby="basic-abg-h" data-testid="basic-abgeschaltet" /></label>
      <p class="small dim" id="basic-abg-h">{t("basicui-abgeschaltet-hinweis")}</p>

      <div class="row save"><button type="button" class="btn pri" disabled={!dirty || rangeBad} onclick={() => void save()} data-testid="basic-speichern">{t("basicui-speichern")}</button></div>
    </div>
  </Card>
{/if}

<style>
  .body { display: flex; flex-direction: column; gap: 8px; padding: 10px 14px 14px; } h3 { margin: 6px 0 0; font-size: 13px; }
  .row { display: flex; gap: 14px; align-items: end; } .lbl { display: flex; flex-direction: column; gap: 4px; font-size: 12px; } .inp { width: 110px; }
  .lbl:last-of-type .inp { width: auto; }
  .small { font-size: 12px; margin: 0; } .dim { color: var(--dim); } .err { color: var(--err, #d33); } .save { margin-top: 6px; } .swrow { margin-top: 6px; } .swrow :global(label) { white-space: nowrap; }
</style>
