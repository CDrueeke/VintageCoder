<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { pj } from "../../lib/projekt.svelte";
  import { setSub } from "../../lib/state.svelte";
  import { t } from "../../lib/i18n.svelte";
  // Übersicht: eine Karte je Seite mit Zustandspunkt (Wort und Farbe), Kurzangabe und „Ändern ›“, das zur Seite springt (wie die frühere Projekt-Übersicht)
  const i = $derived(pj.info);
  type Zustand = "ok" | "warn" | "err" | "off";
  const karten = $derived.by(() => {
    if (!i) return [];
    const progs = i.sources.length;
    return [
      { id: "allgemein", z: (i.name.trim() ? "ok" : "err") as Zustand, zeilen: [i.name.trim() || t("projekt-name-fehlt"), t("projekt-start", { f: i.mainEffective ?? t("projekt-kein-programm") })] },
      { id: "ziele", z: (i.targets.length ? "ok" : "err") as Zustand, zeilen: i.targets.length ? i.targets.map((x) => `${x.system.toUpperCase()}${x.variant ? " · " + x.variant.toUpperCase() : ""}`) : [t("projekt-kein-ziel")] },
      { id: "quellen", z: (i.missing.length ? "err" : progs ? "ok" : "warn") as Zustand, zeilen: [t("projekt-quellen-anzahl", { n: progs, g: i.groups }), ...(i.missing.length ? [t("projekt-fehlende-dateien", { n: i.missing.length })] : [])] },
      { id: "diskette", z: (i.disk || i.turbo ? "ok" : "off") as Zustand, zeilen: [i.disk ? t("projekt-diskette-an", { f: i.disk.format.toUpperCase(), n: i.disk.files.length || progs }) : i.turbo ? t("projekt-turbo-an") : t("projekt-diskette-aus")] },
    ];
  });
  const wort = (z: Zustand) => z === "ok" ? t("projekt-zustand-ok") : z === "warn" ? t("projekt-zustand-warn") : z === "err" ? t("projekt-zustand-err") : t("projekt-zustand-off");
</script>

{#if i}
  <div class="grid" data-pj="uebersicht">
    {#each karten as k (k.id)}
      <section class="k" data-pj-karte={k.id} aria-labelledby={`pjk-${k.id}`}>
        <h3 id={`pjk-${k.id}`}><span class={`dot ${k.z}`} aria-hidden="true"></span>{t(`nav-projekt-${k.id}`)} <span class="w">{wort(k.z)}</span></h3>
        {#each k.zeilen as z (z)}<p>{z}</p>{/each}
        <button type="button" class="btn sm" data-pj-aendern={k.id} onclick={() => setSub("projekt", k.id)}>{t("projekt-aendern")}</button>
      </section>
    {/each}
  </div>
  {#if i.errors.length || i.warnings.length}
    <section class="k probs" data-pj="probleme" aria-label={t("projekt-probleme")}><h3>{t("projekt-probleme")}</h3>
      <ul>{#each i.errors as e (e)}<li class="err">● {e}</li>{/each}{#each i.warnings as w (w)}<li class="warn">▲ {w}</li>{/each}</ul></section>
  {/if}
{/if}

<style>
  .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(240px, 1fr)); gap: 12px; max-width: 980px; }
  .k { display: flex; flex-direction: column; gap: 4px; padding: 12px 14px; border-radius: 12px; border: 1px solid var(--line2); background: var(--card); min-width: 0; }
  .k h3 { margin: 0 0 4px; font-size: 13px; display: flex; align-items: center; gap: 7px; } .k p { margin: 0; color: var(--dim); font-size: 12.5px; overflow-wrap: anywhere; } .k .btn { align-self: flex-start; margin-top: 6px; }
  .w { font-weight: normal; color: var(--dim); font-size: 11px; margin-left: auto; }
  .dot { width: 10px; height: 10px; border-radius: 50%; flex: none; background: var(--dim); } .dot.ok { background: var(--ok, #3fb950); } .dot.warn { background: var(--warn, #d29922); } .dot.err { background: var(--err, #f85149); }
  .probs { max-width: 980px; margin-top: 12px; } .probs ul { margin: 0; padding: 0; list-style: none; display: flex; flex-direction: column; gap: 3px; font-size: 12.5px; } .err { color: var(--err, #f85149); } .warn { color: var(--warn, #d29922); }
</style>
