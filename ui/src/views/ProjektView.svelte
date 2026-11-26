<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { t } from "../lib/i18n.svelte";
  import { loadTurbo, tb } from "../lib/turbo.svelte";
  import { setSub, ui, visibleSubs } from "../lib/state.svelte";
  import { section } from "../lib/nav";
  import TurboCard from "../lib/components/TurboCard.svelte";
  import Icon from "../lib/components/Icon.svelte";
  import { loadProjekt } from "../lib/projekt.svelte";
  import Uebersicht from "./projekt/Uebersicht.svelte";
  import Allgemein from "./projekt/Allgemein.svelte";
  import Ziele from "./projekt/Ziele.svelte";
  import Quellen from "./projekt/Quellen.svelte";
  import Bauen from "./projekt/Bauen.svelte";
  import Weitergeben from "./projekt/Weitergeben.svelte";
  // Bereich „Projekt“: die Projektsicht an einer Stelle, Unterseiten in der zweiten Reihe (Übersicht, Allgemein, Ziele, Quellen, Bauen und Starten, Diskette, Weitergeben).
  // P1: Navigation und Seitenrahmen; P2: Übersicht, Allgemein, Ziele; P3: Quellen, Bauen und Starten, Weitergeben (die Seite Diskette zeigt die Turbo-Karte). Die Turbo-Karte (früher „Projekt-Einstellungen“) liegt auf der Seite Diskette.
  const sub = $derived(ui.sub.projekt ?? "uebersicht");
  const sec = $derived(section("projekt", sub));
  $effect(() => { void ui.project?.root; void sub; if (ui.project) void loadProjekt(); });
  $effect(() => { void ui.project?.root; if (ui.project && sub === "diskette") void loadTurbo(); });
</script>

<div class="ps" data-testid="projekt-einstellungen" data-projekt={sub}>
  <div class="top"><h2 data-projekt-titel>{t(sec.title)}</h2></div>
  {#if !ui.project}
    <p class="dim" style="padding:14px" data-projekt="leer">{t("projekt-ohne-projekt")}</p>
  {:else if sub === "diskette"}
    <div class="cards">
      {#if tb.state?.turbo}<TurboCard />{:else}<p class="dim" style="padding:14px">{t("turboui-projekt-ohne-turbo")}</p>{/if}
    </div>
  {:else if sub === "uebersicht"}<Uebersicht />
  {:else if sub === "allgemein"}<Allgemein />
  {:else if sub === "ziele"}<Ziele />
  {:else if sub === "quellen"}<Quellen />
  {:else if sub === "bauen"}<Bauen />
  {:else if sub === "weitergeben"}<Weitergeben />
  {:else}
    <div class="cards">
      <div class="card" data-projekt="folgt"><Icon name="folder" size={22} /><p>{t("projekt-seite-folgt")}</p>
        <ul class="orte" aria-label={t("nav-sub-label")}>{#each visibleSubs("projekt").filter((s) => s.id !== sub) as s (s.id)}<li><button type="button" class="btn" onclick={() => setSub("projekt", s.id)}>{t(s.label)}</button></li>{/each}</ul></div>
    </div>
  {/if}
</div>

<style>
  .ps { display: flex; flex-direction: column; min-height: 0; flex: 1; overflow: auto; padding: 12px 16px; gap: 10px; }
  .top { display: flex; align-items: center; gap: 10px; } .top h2 { margin: 0; font-size: 15px; }
  .cards { display: flex; flex-direction: column; gap: 12px; max-width: 820px; } .dim { color: var(--dim); }
  .card { display: flex; flex-direction: column; gap: 8px; padding: 14px; border-radius: 12px; border: 1px solid var(--line2); background: var(--card); color: var(--dim); }
  .orte { list-style: none; margin: 0; padding: 0; display: flex; flex-wrap: wrap; gap: 6px; }
  :global(.tcard) { position: static; }
</style>
