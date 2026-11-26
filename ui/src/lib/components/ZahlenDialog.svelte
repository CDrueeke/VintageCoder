<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { zahlenBit, zahlenBreite, zahlenEinsetzen, zahlenKopieren, zahlenText, zw } from "../zahlen.svelte";
  import Dialog from "./Dialog.svelte";
  import { t } from "../i18n.svelte";
  // Zahlenwerkzeuge: Zahl oder Ausdruck eingeben, alle Schreibweisen ansehen, Bits per Klick umschalten, kopieren oder an der Schreibmarke einsetzen
  let offen = $state(false);
  $effect(() => { offen = zw.offen; }); $effect(() => { if (!offen && zw.offen) zw.offen = false; });
  const w = $derived(zw.werte);
  const arten = [["dec", "zahlen-dezimal"], ["hex", "zahlen-hexadezimal"], ["bin", "zahlen-binaer"]] as const;
</script>
{#if offen}
  <Dialog bind:open={offen} title={t("zahlen-zahlenwerkzeuge")}>
    {#snippet children()}
      <div class="zw" data-zw="dialog">
        <label class="small dim">{t("zahlen-zahl-oder-ausdruck-d020-0xff-1010")}
          <input class="inp mono" data-zw="eingabe" value={zw.text} oninput={(e) => zahlenText(e.currentTarget.value)} /></label>
        {#if w && !w.ok}<p class="small err" role="alert" data-zw="err">{w.error}</p>{/if}
        {#if w?.ok}
          <dl class="kv" data-zw="werte">
            {#each arten as [art, schl] (art)}
              <div class="row"><dt>{t(schl)}</dt><dd class="mono" data-zw={art}>{w[art]}</dd>
                <button type="button" class="btn sm" data-zw={`kopieren-${art}`} onclick={() => void zahlenKopieren(art)}>{t("zahlen-kopieren")}</button>
                <button type="button" class="btn sm" data-zw={`einsetzen-${art}`} title={t("zahlen-in-den-editor-an-der-schreibmarke")} onclick={() => zahlenEinsetzen(art)}>{t("zahlen-einsetzen")}</button></div>
            {/each}
            <div class="row"><dt>{t("zahlen-mit-vorzeichen-b-bit", { b: w.bits ?? 8 })}</dt><dd class="mono" data-zw="signed">{w.signed}</dd></div>
            <div class="row"><dt>{t("zahlen-lo-hi-byte")}</dt><dd class="mono" data-zw="bytes">{w.lo} / {w.hi}</dd></div>
            {#if w.char}<div class="row"><dt>{t("zahlen-zeichen-ascii")}</dt><dd class="mono" data-zw="zeichen">{w.char}</dd></div>{/if}
          </dl>
          <div class="bits" role="group" aria-label={t("zahlen-bit-editor")}>
            <span class="small dim">{t("zahlen-bit-editor")}</span>
            {#each w.bitList ?? [] as b, i (i)}
              {@const n = (w.bits ?? 8) - 1 - i}
              <button type="button" class="bit" class:an={b === 1} aria-pressed={b === 1} aria-label={t("zahlen-bit-n-umschalten", { n })} data-zw="bit" data-n={n} onclick={() => zahlenBit(n)}>{b}</button>
            {/each}
            <span class="breite" role="radiogroup" aria-label={t("zahlen-breite")}>
              <button type="button" class="btn sm" class:pri={zw.bits === 8} role="radio" aria-checked={zw.bits === 8} data-zw="b8" onclick={() => zahlenBreite(8)}>{t("zahlen-8-bit")}</button>
              <button type="button" class="btn sm" class:pri={zw.bits === 16} role="radio" aria-checked={zw.bits === 16} data-zw="b16" onclick={() => zahlenBreite(16)}>{t("zahlen-16-bit")}</button>
            </span>
          </div>
        {/if}
      </div>
    {/snippet}
    {#snippet footer()}<button type="button" class="btn" data-zw="zu" onclick={() => (zw.offen = false)}>{t("zahlen-schliessen")}</button>{/snippet}
  </Dialog>
{/if}
<style>
  .zw { display: flex; flex-direction: column; gap: 10px; min-width: 460px; } label { display: flex; flex-direction: column; gap: 4px; } .inp { width: 100%; font: inherit; } .mono { font-family: var(--font-mono, monospace); }
  .kv { margin: 0; display: flex; flex-direction: column; gap: 4px; } .row { display: flex; align-items: center; gap: 8px; } dt { width: 190px; color: var(--dim); font-size: 12px; } dd { margin: 0; flex: 1; min-width: 0; overflow-wrap: anywhere; }
  .bits { display: flex; align-items: center; gap: 4px; flex-wrap: wrap; } .bit { width: 26px; height: 28px; border-radius: 6px; border: 1px solid var(--line2); background: var(--card); font-family: var(--font-mono, monospace); } .bit.an { background: var(--accent); border-color: var(--accent); color: #fff; }
  .breite { margin-left: auto; display: flex; gap: 4px; } .small { font-size: 12px; margin: 0; } .dim { color: var(--dim); } .err { color: var(--err); }
</style>
