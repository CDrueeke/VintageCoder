<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { t } from "../i18n.svelte";
  import { checkCopy, diskName, openInDiskEditor, openProvide, openReduce, saveSettings, tb } from "../turbo.svelte";
  import Card from "./Card.svelte";
  // Karte „Turbo BASIC XL“ in den Projekt-Einstellungen (Konzept 10): Zustand der Diskette, Knöpfe, Programmname, Zusatzdateien, Hinweis „nicht im Lieferumfang“
  const s = $derived(tb.state);
  const r = $derived(s?.report ?? null);
  let program = $state(""), files = $state<string[]>([]), neu = $state(""), loadedFor = $state(""), touched = $state(false);
  // Felder aus dem Zustand übernehmen, solange der Anwender nichts geändert hat
  $effect(() => { const st = s; if (!st) return; const sig = JSON.stringify([st.program, st.files]); if (sig !== loadedFor && !touched) { program = st.program; files = [...st.files]; loadedFor = sig; } });
  const dirty = $derived(!!s && (program.trim().toUpperCase() !== s.program || JSON.stringify(files) !== JSON.stringify(s.files)));
  const validName = (n: string) => /^[A-Z][A-Z0-9_]{0,7}(\.[A-Z0-9_]{1,3})?$/.test(n.trim().toUpperCase());
  const progBad = $derived(!validName(program));
  function add() { const z = neu.trim(); if (!z) return; if (!files.includes(z)) files = [...files, z]; neu = ""; touched = true; }
  async function save() { if (progBad) return; add(); const ok = await saveSettings(program.trim().toUpperCase(), files); if (ok && tb.state) { loadedFor = ""; touched = false; } }
  const missingSrc = (z: string) => (s?.warnings ?? []).some((w) => w.includes(z));
</script>

{#if s?.turbo}
  <Card title={t("turboui-karte-titel")} class="tcard">
    {#snippet header()}<span class="state" class:ok={s.present} class:bad={!s.present} data-testid="turbo-zustand">{s.present ? t("turboui-zustand-bereit") : t("turboui-zustand-fehlt")}</span>{/snippet}
    <div class="body" data-testid="turbo-card">
      <p class="legal">{t("rechtlich-turbo-assistent")}</p>
      {#if s.present && r}
        <ul class="facts" aria-label={t("turboui-pruefung-titel")}>
          <li><b>{diskName(r)}</b></li>
          <li>{t("turboui-zustand-frei", { count: r.free_sectors, kb: r.kb })}</li>
          <li>{s.reduced ? t("turboui-zustand-reduziert-ja") : t("turboui-zustand-reduziert-nein")}</li>
          <li class="mono small">{t("turboui-zustand-kopie", { disk: s.disk })}</li>
          {#if s.changed}<li class="warn">{t("turboui-zustand-veraendert")}</li>{/if}
          {#if !r.turbo}<li class="warn">{t("turboui-zustand-nicht-turbo")}</li>{/if}
          {#each r.problems.slice(0, 3) as p (p)}<li class="warn">{t("turboui-dlg-befund-fehler", { m: p })}</li>{/each}
        </ul>
      {:else if s.disk}
        <p class="warn" role="alert">{t("turboui-zustand-datei-fehlt", { disk: s.disk })}</p>
      {:else}
        <p class="dim">{t("turboui-zustand-fehlt")}</p>
      {/if}
      <div class="btns" role="group" aria-label={t("turboui-karte-titel")}>
        <button type="button" class="btn" class:pri={!s.present} onclick={() => openProvide(s.disk ? "replace" : "provide")} data-testid="turbo-bereitstellen">{s.disk ? t("turboui-knopf-ersetzen") : t("turboui-knopf-bereitstellen")}</button>
        <button type="button" class="btn" disabled={!s.present} onclick={() => void checkCopy()} data-testid="turbo-pruefen">{t("turboui-knopf-pruefen")}</button>
        <button type="button" class="btn" disabled={!s.present || !r?.reduce || s.reduced} onclick={openReduce} data-testid="turbo-verkleinern">{t("turboui-knopf-verkleinern")}</button>
        <button type="button" class="btn" disabled={!s.present} onclick={openInDiskEditor} data-testid="turbo-diskeditor">{t("turboui-knopf-disk-editor")}</button>
      </div>
      {#if tb.note}
        <div class="note" class:okn={tb.note.kind === "ok"} role="status" data-testid="turbo-note"><p>{tb.note.text}</p>
          {#if tb.note.lines.length}<ul class="mono small">{#each tb.note.lines as l (l)}<li>{l}</li>{/each}</ul>{/if}</div>
      {/if}
      {#if r && r.entries.length}
        <details class="files"><summary>{t("turboui-dateien-titel", { count: r.entries.length })}</summary>
          <ul class="mono small">{#each r.entries as e (e.name + e.sectors)}<li>{t("turboui-datei-zeile", { name: e.name, sectors: e.sectors })}{e.locked ? ` · ${t("turboui-datei-gesperrt")}` : ""}</li>{/each}</ul></details>
      {/if}

      <h3>{t("turboui-feld-programm")}</h3>
      <div class="fld"><input class="inp mono" value={program} aria-label={t("turboui-feld-programm")} aria-invalid={progBad} oninput={(e) => { program = e.currentTarget.value; touched = true; }} data-testid="turbo-programm" />
        <p class="small dim">{t("turboui-feld-programm-hinweis")}</p>
        {#if progBad}<p class="small err" role="alert">{t("turboui-feld-programm-fehler", { name: program })}</p>{/if}</div>

      <h3>{t("turboui-zusatz-titel")}</h3>
      <p class="small dim">{t("turboui-zusatz-hinweis")}</p>
      {#if files.length}
        <ul class="zl" data-testid="turbo-zusatz">{#each files as z (z)}
          <li><span class="mono grow">{z}</span>{#if missingSrc(z)}<span class="small warn">{t("turboui-zusatz-fehlt", { zeile: z })}</span>{/if}
            <button type="button" class="btn" aria-label={t("turboui-zusatz-entfernen", { zeile: z })} onclick={() => { files = files.filter((x) => x !== z); touched = true; }}>×</button></li>{/each}</ul>
      {:else}<p class="small dim">{t("turboui-zusatz-leer")}</p>{/if}
      <div class="row"><input class="inp mono grow" bind:value={neu} placeholder={t("turboui-zusatz-neu")} aria-label={t("turboui-zusatz-neu")} onkeydown={(e) => { if (e.key === "Enter") { e.preventDefault(); add(); } }} data-testid="turbo-zusatz-neu" />
        <button type="button" class="btn" onclick={add} disabled={!neu.trim()}>{t("turboui-zusatz-hinzufuegen")}</button></div>
      <div class="row save"><button type="button" class="btn pri" disabled={!dirty && !neu.trim() || progBad} onclick={() => void save()} data-testid="turbo-speichern">{t("turboui-speichern")}</button></div>

      <p class="small dim">{t("turboui-hinweis-kein-lieferumfang")}</p>
      <p class="small dim">{t("turboui-hinweis-gitignore")}</p>
      <p class="small dim">{t("turboui-debuggen-hinweis")}</p>
    </div>
  </Card>
{/if}

<style>
  .body { padding: 4px 14px 14px; color: var(--text); display: flex; flex-direction: column; gap: 6px; font-size: 13px; }
  .state { font-size: 11.5px; padding: 2px 8px; border-radius: 10px; border: 1px solid var(--line2); } .state.ok { color: var(--ok); } .state.bad { color: var(--warn); }
  .legal { margin: 0; padding: 8px 10px; border-radius: 8px; background: var(--raise); border: 1px solid var(--line2); font-size: 12.5px; }
  .facts { margin: 0; padding-left: 18px; } .facts li { margin: 2px 0; }
  .btns, .row { display: flex; gap: 8px; flex-wrap: wrap; align-items: center; } .row.save { margin-top: 6px; }
  h3 { font-size: 11.5px; margin: 12px 0 2px; color: var(--dim); letter-spacing: 0.04em; text-transform: uppercase; }
  .inp { height: 30px; padding: 0 8px; border-radius: var(--r-ctl); border: 1px solid var(--line2); background: var(--raise); color: var(--text); font: inherit; max-width: 480px; } .inp[aria-invalid="true"] { border-color: var(--err); }
  .grow { flex: 1; min-width: 0; } .zl { list-style: none; margin: 0; padding: 0; } .zl li { display: flex; align-items: center; gap: 8px; margin: 3px 0; }
  .small { font-size: 11.5px; margin: 2px 0; } .dim { color: var(--dim); } .warn { color: var(--warn); } .err { color: var(--err); }
  .note { border: 1px solid var(--warn); border-radius: 8px; padding: 6px 10px; } .note.okn { border-color: var(--ok); } .note p { margin: 0; } .note ul { margin: 4px 0 0; padding-left: 18px; }
  .files summary { cursor: pointer; font-size: 12px; } .files ul { margin: 4px 0 0; padding-left: 18px; max-height: 180px; overflow: auto; }
  @media (forced-colors: active) { .legal, .state, .note { border: 1px solid CanvasText; } }
</style>
