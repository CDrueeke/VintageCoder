<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { call, type RegistryInfo } from "../../lib/api";
  import { pj, setzeProjekt } from "../../lib/projekt.svelte";
  import { bau, buildRun } from "../../lib/bau.svelte";
  import { fail } from "../../lib/state.svelte";
  import { currentLang, t } from "../../lib/i18n.svelte";
  // Bauen und Starten: letzter Bau, Bau- und Startknöpfe, Emulator je Ziel (Überschreibungen des Projekts; die globale Wahl steht in den Einstellungen) und die Pakete der Registry mit Herkunft.
  const i = $derived(pj.info);
  let reg = $state<RegistryInfo | null>(null);
  $effect(() => { void currentLang(); void call("registry_info").then((r) => (reg = r)).catch(fail); });
  // Diskettenabbild beim Bauen (Abschnitt `disk` der Projektdatei): Format und Name; ohne Angabe wird keines erzeugt
  const diskFormate = ["d64", "d71", "d81", "atr"];
  const wann = (at: number) => new Date(at).toLocaleTimeString();
</script>

{#if i}
  <div class="b" data-pj="bauen">
    <section class="k" data-pj-karte="bau" aria-label={t("projekt-letzter-bau")}>
      <h3>{t("projekt-letzter-bau")}</h3>
      {#if bau.last}<p data-pj="bau-ergebnis">{bau.last.ok ? t("projekt-bau-ok", { w: wann(bau.last.at) }) : t("projekt-bau-fehler", { n: bau.last.errors, w: wann(bau.last.at) })}</p>
      {:else}<p class="dim" data-pj="bau-ergebnis">{t("projekt-noch-nicht-gebaut")}</p>{/if}
      <div class="row"><button type="button" class="btn" data-pj="bauen-knopf" disabled={bau.running} onclick={() => void buildRun()}>{t("bar-bauen")}</button>
        <button type="button" class="btn pri" data-pj="starten-knopf" disabled={bau.running} onclick={() => void buildRun({ then: "run" })}>{t("bar-starten")}</button></div>
    </section>
    <section class="k" data-pj-karte="emulator" aria-label={t("projekt-emulator-je-ziel")}>
      <h3>{t("projekt-emulator-je-ziel")}</h3>
      {#each i.targets as z (z.id)}
        <p data-pj-ziel-emu={z.id}><b>{z.system.toUpperCase()}</b>{z.variant ? ` · ${z.variant.toUpperCase()}` : ""}: {z.emulator ? t("projekt-emulator-projekt", { e: z.emulator }) : t("projekt-emulator-global")}{z.startOn ? " · " + t("projekt-start-auf", { s: z.startOn }) : ""}</p>
      {/each}
      <p class="small dim">{t("projekt-emulator-hinweis")}</p>
    </section>
    <section class="k" data-pj-karte="abbild" aria-label={t("projekt-abbild")}>
      <h3>{t("projekt-abbild")}</h3>
      {#if i.disk}
        <div class="row"><label class="small dim">{t("projekt-abbild-format")}<select class="inp" data-pj="abbild-format" value={i.disk.format} onchange={(e) => void setzeProjekt({ disk: { format: e.currentTarget.value } })}>{#each diskFormate as f (f)}<option value={f}>{f.toUpperCase()}</option>{/each}</select></label>
          <label class="small dim">{t("projekt-abbild-name")}<input class="inp" data-pj="abbild-name" value={i.disk.name} onchange={(e) => void setzeProjekt({ disk: { name: e.currentTarget.value } })} /></label>
          <button type="button" class="btn sm" data-pj="abbild-weg" onclick={() => void setzeProjekt({ disk: null })}>{t("projekt-abbild-weg")}</button></div>
        <p class="small dim">{t("projekt-abbild-hinweis", { datei: i.disk.file || "build/…" })}</p>
      {:else}<p class="dim" data-pj="abbild-aus">{t("projekt-abbild-aus")}</p><div class="row">{#each diskFormate as f (f)}<button type="button" class="btn sm" data-pj="abbild-neu" data-f={f} onclick={() => void setzeProjekt({ disk: { format: f } })}>{t("projekt-abbild-neu", { f: f.toUpperCase() })}</button>{/each}</div>{/if}
    </section>
    <section class="k" data-pj-karte="pakete" aria-label={t("projekt-pakete")}>
      <h3>{t("projekt-pakete")}</h3>
      {#if reg}
        {#each reg.packages as p (p.id)}<p data-pj-paket={p.id}><b>{p.name || p.id}</b> <span class="dim">{p.kind} · {p.version}</span></p>{:else}<p class="dim">{t("projekt-keine-pakete")}</p>{/each}
        {#each reg.sources as s (s.path + s.kind)}<p class="small dim">{t("projekt-paket-herkunft", { k: s.kind, o: s.origin, p: s.path })}</p>{/each}
        {#each reg.problems as pr (pr.file + pr.text)}<p class="small bad">● {pr.file}: {pr.text}</p>{/each}
      {/if}
    </section>
  </div>
{/if}

<style>
  .inp { height: 28px; padding: 0 8px; border-radius: var(--r-ctl); border: 1px solid var(--line2); background: var(--raise); color: var(--text); font: inherit; } .row { display: flex; gap: 8px; align-items: flex-end; flex-wrap: wrap; } label { display: flex; flex-direction: column; gap: 3px; }
  .b { display: flex; flex-direction: column; gap: 12px; max-width: 760px; } .k { display: flex; flex-direction: column; gap: 5px; padding: 12px 14px; border-radius: 12px; border: 1px solid var(--line2); background: var(--card); }
  h3 { margin: 0 0 2px; font-size: 13px; } p { margin: 0; font-size: 13px; overflow-wrap: anywhere; } .small { font-size: 12px; } .dim { color: var(--dim); font-weight: normal; } .bad { color: var(--err, #f85149); } .row { display: flex; gap: 8px; margin-top: 4px; }
</style>
