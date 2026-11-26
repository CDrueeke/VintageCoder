<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { humanBytes, kop, pickCopyDir, runCopy, setCopy } from "../kopieren.svelte";
  import { t } from "../i18n.svelte";
  import Dialog from "./Dialog.svelte";
  // Projekt kopieren: Name der Kopie, Speicherort, Bauordner und Versionsgeschichte mitkopieren, danach öffnen
  function key(e: KeyboardEvent) { if (e.key === "Enter" && (e.target as HTMLElement).tagName === "INPUT" && !kop.problem) { e.preventDefault(); void runCopy(); } }
</script>

<Dialog bind:open={kop.open} title={t("kopierdlg-projekt-kopieren")}>
  {#snippet children()}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="kp" onkeydown={key}>
      <p class="mono small">{t("kopierdlg-quelle-p1", { p1: kop.src })}</p>
      <label class="fld"><span>{t("kopierdlg-name-der-kopie")}</span><input class="inp" value={kop.name} aria-invalid={!!kop.problem} oninput={(e) => setCopy("name", e.currentTarget.value)} /></label>
      <div class="fld"><span>{t("kopierdlg-speicherort")}</span><div class="row"><span class="mono small grow" title={kop.dir}>{kop.dir}</span><button type="button" class="btn" onclick={() => void pickCopyDir()}>{t("kopierdlg-waehlen")}</button></div></div>
      {#if kop.systems.length}
      <label class="fld"><span>{t("kopierdlg-system")}</span><select class="inp" data-kop="system" value={kop.system} onchange={(e) => setCopy("system", e.currentTarget.value)}><option value="">{t("kopierdlg-system-wie-original")}</option>{#each kop.systems as x (x.id)}<option value={x.id}>{x.name}</option>{/each}</select></label>
      {#if kop.system}<p class="dim small" data-kop="system-hinweis">{t("kopierdlg-system-hinweis-lang")}</p>{/if}
      {/if}
      <label class="chk"><input type="checkbox" checked={kop.withBuild} onchange={(e) => setCopy("withBuild", e.currentTarget.checked)} />{t("kopierdlg-bauordner-build-mitkopieren")}</label>
      <label class="chk"><input type="checkbox" checked={kop.withGit} onchange={(e) => setCopy("withGit", e.currentTarget.checked)} />{t("kopierdlg-versionsgeschichte-git-mitkopieren")}</label>
      <label class="chk"><input type="checkbox" checked={kop.after} onchange={(e) => setCopy("after", e.currentTarget.checked)} />{t("kopierdlg-danach-die-kopie-oeffnen")}</label>
      {#if kop.problem}<p class="err" role="alert">{kop.problem}</p>
      {:else if kop.info}<p class="dim small">{t("kopierdlg-n-dateien-p2-werden-kopiert", { n: kop.info.files, p2: humanBytes(kop.info.bytes) })}</p>{/if}
      {#if kop.error}<p class="err" role="alert">{kop.error}</p>{/if}
    </div>
  {/snippet}
  {#snippet footer()}
    <button type="button" class="btn" disabled={kop.busy} onclick={() => (kop.open = false)}>{t("kopierdlg-abbrechen")}</button>
    <button type="button" class="btn pri" disabled={!!kop.problem || kop.busy} onclick={() => void runCopy()}>{kop.busy ? t("kopierdlg-kopiert") : t("kopierdlg-kopieren")}</button>
  {/snippet}
</Dialog>

<style>
  .kp { color: var(--text); min-width: 440px; } @media (max-width: 560px) { .kp { min-width: 0; } }
  .fld { display: flex; flex-direction: column; gap: 4px; margin: 0 0 10px; } .fld > span { font-size: 11.5px; color: var(--dim); }
  .inp { height: 30px; padding: 0 8px; border-radius: var(--r-ctl); border: 1px solid var(--line2); background: var(--raise); color: var(--text); font: inherit; }
  .row { display: flex; align-items: center; gap: 8px; } .grow { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .chk { display: flex; gap: 8px; align-items: center; margin: 4px 0; font-size: 13px; }
  .small { font-size: 11.5px; margin: 0 0 8px; } .dim { color: var(--dim); } .err { color: var(--err, #c33); margin: 6px 0 0; }
</style>
