<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { copyIntoSource, close, ex, includeIntoSource, preview, saveToProject, setFormat, testInEmulator, braucht, ziele } from "../export.svelte";
  import Dialog from "./Dialog.svelte";
  import Segment from "./Segment.svelte";
  import { t } from "../i18n.svelte";
  // Export (Mockup 12): Format, Name, Adresse, Umfang und Vorschau; „Als Include einfügen“ bindet die erzeugte Datei ein, „In Quelltext kopieren“ schreibt den Text selbst
  // an die Cursorzeile (ohne Include). Bei BASIC DATA auf Wunsch mit Einleseroutine.
  let open = $state(false);
  $effect(() => { open = ex.open; });
  $effect(() => { if (!open && ex.open) close(); });
  const formats = $derived(["asm", "inc", "bin", "prg", "data"].map((f) => ({ id: f, label: t(`export-format-${f}`) })));
  const tabs = $derived([{ id: "text", label: t("export-tab-text") }, ...(ex.format === "data" ? [] : [{ id: "hex", label: t("export-tab-hex") }])]);
  const gesamt = $derived(ex.files.reduce((a, f) => a + f[1], 0));
  const ok = $derived(ex.error === null && ex.name.trim() !== "");
  const zieleListe = $derived(ziele());
</script>

{#if ex.open}
  <Dialog bind:open title={t("export-titel")} wide>
    {#snippet children()}
      <p class="small dim mono">{ex.rel}</p>
      <div class="fld"><span class="lbl">{t("export-format")}</span><Segment items={formats} value={ex.format} onselect={(id) => setFormat(id)} label={t("export-format")} small /></div>
      <div class="row2">
        <label class="fld"><span class="lbl">{t("export-name")}</span><input class="inp mono" data-ex="name" bind:value={ex.name} oninput={() => void preview()} /></label>
        {#if braucht(ex.format) || (ex.format === "data" && ex.routine)}
          <label class="fld"><span class="lbl">{ex.format === "prg" ? t("export-ladeadresse") : t("export-adresse")}</span><input class="inp mono" data-ex="addr" bind:value={ex.addr} oninput={() => void preview()} aria-invalid={ex.error !== null && /dresse/.test(ex.error)} /></label>
        {/if}
      </div>
      {#if ex.format === "data"}
        <label class="chk"><input type="checkbox" data-ex="routine" bind:checked={ex.routine} onchange={() => void preview()} /> {t("export-routine")}</label>
        {#if ex.routine}<p class="small dim">{t("export-routine-hinweis")}</p>{/if}
      {/if}
      <div class="tabs"><Segment items={tabs} value={ex.tab} onselect={(id) => (ex.tab = id as "text" | "hex")} label={t("export-vorschau")} small /></div>
      <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
      <pre class="vor mono" data-ex="vorschau" tabindex="0" aria-label={t("export-vorschau")}>{ex.tab === "hex" ? (ex.hex || t("export-keine-binaer")) : ex.text}</pre>
      <p class="small" class:bad={ex.error !== null} data-ex="status" role="status">{ex.error ? `● ${ex.error}` : t("export-bereit", { n: ex.files.length, bytes: gesamt })}</p>
      {#if ex.files.length}<ul class="small dim files">{#each ex.files as f (f[0])}<li class="mono">{f[0]} · {f[1]} B</li>{/each}</ul>{/if}
      <label class="fld"><span class="lbl">{t("export-ziel")}</span>
        {#if zieleListe.length}<select class="inp" data-ex="ziel" bind:value={ex.target}>{#each zieleListe as z (z)}<option value={z}>{z}</option>{/each}</select>
        {:else}<span class="small warn">{t("export-kein-ziel")}</span>{/if}</label>
      {#if ex.message}<p class="small" class:bad={!ex.ok} role="status" data-ex="meldung">{ex.message}</p>{/if}
    {/snippet}
    {#snippet footer()}
      <button type="button" class="btn" onclick={close}>{t("dialog-abbrechen")}</button>
      <button type="button" class="btn" data-ex="test" disabled={!ok} onclick={() => void testInEmulator()}>{t("export-testen")}</button>
      <button type="button" class="btn" data-ex="save" disabled={!ok} onclick={() => void saveToProject()}>{t("export-speichern")}</button>
      {#if ex.format === "asm" || ex.format === "data"}
        <button type="button" class="btn" data-ex="include" disabled={!ok || !zieleListe.length} onclick={() => void includeIntoSource()}>{t("export-include")}</button>
        <button type="button" class="btn pri" data-ex="copy" disabled={!ok || !zieleListe.length} onclick={() => void copyIntoSource()}>{t("export-kopieren")}</button>
      {/if}
    {/snippet}
  </Dialog>
{/if}

<style>
  .fld { display: flex; flex-direction: column; gap: 4px; margin: 8px 0; min-width: 0; } .lbl { color: var(--dim); font-size: 11px; letter-spacing: 0.05em; text-transform: uppercase; }
  .row2 { display: flex; gap: 12px; flex-wrap: wrap; } .row2 .fld { flex: 1; min-width: 140px; } .chk { display: flex; gap: 8px; align-items: center; margin: 6px 0; }
  .inp { width: 100%; } .small { font-size: 12px; margin: 4px 0; } .dim { color: var(--dim); } .bad { color: var(--err); } .warn { color: var(--warn); }
  .tabs { margin: 8px 0 4px; } .vor { margin: 0; background: var(--codebg); border: 1px solid var(--line2); border-radius: 8px; padding: 10px; max-height: 240px; min-height: 90px; overflow: auto; font-size: 12px; white-space: pre; }
  .files { margin: 4px 0; padding-left: 16px; }
</style>
