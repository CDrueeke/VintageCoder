<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { call, type RecentProject } from "../../lib/api";
  import { runCommand } from "../../lib/commands";
  import { fail, openProject, ui } from "../../lib/state.svelte";
  import { currentLang, t } from "../../lib/i18n.svelte";
  // Weitergeben: das Projekt kopieren (zum Beispiel für eine Weitergabe oder als Sicherung) und der Verlauf der zuletzt geöffneten Projekte. Export von Grafik, Zeichensätzen und Musik steht in den jeweiligen Editoren.
  let recent = $state<RecentProject[]>([]); let geladen = $state(false);
  $effect(() => { void ui.project?.root; void currentLang(); void call("recent_list").then((r) => { recent = r; geladen = true; }).catch((e) => { fail(e); geladen = true; }); });
</script>

<div class="w" data-pj="weitergeben">
  <section class="k" data-pj-karte="kopieren" aria-label={t("projekt-kopieren-titel")}>
    <h3>{t("projekt-kopieren-titel")}</h3>
    <p class="dim">{t("projekt-kopieren-text")}</p>
    <div class="row"><button type="button" class="btn pri" data-pj="kopieren-knopf" onclick={() => void runCommand("projekt_kopieren")}>{t("projekt-kopieren-knopf")}</button></div>
  </section>
  <section class="k" data-pj-karte="export" aria-label={t("projekt-export-titel")}>
    <h3>{t("projekt-export-titel")}</h3>
    <p class="dim">{t("projekt-export-text")}</p>
  </section>
  <section class="k" data-pj-karte="verlauf" aria-label={t("projekt-verlauf")}>
    <h3>{t("projekt-verlauf")}</h3>
    {#each recent as r (r.path)}
      <div class="r" data-pj-zuletzt={r.name}><span class="nm"><b>{r.name}</b> <span class="dim mono">{r.pretty}</span></span><span class="dim">{r.ago}</span>
        <button type="button" class="btn sm" disabled={r.missing || r.path === ui.project?.root} onclick={() => void openProject(r.path)}>{t("startui-oeffnen")}</button></div>
    {:else}{#if geladen}<p class="dim">{t("startui-noch-kein-projekt-geoeffnet-neues-projekt")}</p>{/if}{/each}
  </section>
</div>

<style>
  .w { display: flex; flex-direction: column; gap: 12px; max-width: 760px; } .k { display: flex; flex-direction: column; gap: 6px; padding: 12px 14px; border-radius: 12px; border: 1px solid var(--line2); background: var(--card); }
  h3 { margin: 0; font-size: 13px; } p { margin: 0; font-size: 13px; } .dim { color: var(--dim); font-weight: normal; } .row { display: flex; gap: 8px; }
  .r { display: flex; align-items: center; gap: 10px; } .nm { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
</style>
