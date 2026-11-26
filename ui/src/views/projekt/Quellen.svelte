<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { pj, setzeProjekt } from "../../lib/projekt.svelte";
  import { runCommand } from "../../lib/commands";
  import Icon from "../../lib/components/Icon.svelte";
  import { t } from "../../lib/i18n.svelte";
  import BasicCard from "../../lib/components/BasicCard.svelte";
  import { loadBasic } from "../../lib/basic.svelte";
  import { ui } from "../../lib/state.svelte";
  // Quellen: die Programme des Projekts in Baureihenfolge, je Quellgruppe. Fehlende Dateien stehen mit Wort und Farbe, das Start-Programm mit Stern. Verschoben wird innerhalb der Gruppe.
  const i = $derived(pj.info);
  const start = $derived(i?.mainEffective ?? null);
  // Gruppen bearbeiten (Art, Dialekt, Ziele), Dateien hinzufügen und entfernen: der Kern prüft Pfade und Doppelte; Gespeichert wird sofort
  let neueDatei = $state<Record<number, string>>({});
  const dateien = $derived.by(() => { const o: string[] = []; const f = (ns: typeof ui.project extends null ? never : NonNullable<typeof ui.project>["tree"]) => { for (const n of ns) { if (n.kind === "dir") f(n.children ?? []); else if (/\.(asm|inc|bas|s)$/i.test(n.rel) && !i?.sources.some((q) => q.file === n.rel)) o.push(n.rel); } }; if (ui.project) f(ui.project.tree); return o; });
  const zielWahl = (g: { targets: string[] }, id: string, an: boolean) => (an ? [...new Set([...g.targets, id])] : g.targets.filter((x) => x !== id));
  $effect(() => { void ui.project?.root; void loadBasic(); });
</script>

{#if i}
  <div class="q" data-pj="quellen">
    <p class="small dim">{t("projekt-quellen-hinweis")}</p>
    {#each i.sourceGroups as g, gi (gi)}
      <section class="k" data-pj-gruppe={gi} aria-label={t("projekt-gruppe", { n: gi + 1 })}>
        <h3>{t("projekt-gruppe", { n: gi + 1 })} <span class="dim">· {g.kind === "basic" ? `BASIC${g.dialect ? " (" + g.dialect + ")" : ""}` : g.kind.toUpperCase()}{g.targets.length ? " · " + g.targets.join(", ") : ""}</span></h3>
        <div class="row gk" data-pj-gruppe-kopf={gi}>
          <label class="small dim">{t("projekt-gruppe-art")}<select class="inp" data-pj="gruppe-art" aria-label={t("projekt-gruppe-art")} value={g.kind} onchange={(e) => void setzeProjekt({ groupSet: { i: gi, kind: e.currentTarget.value } })}><option value="asm">{t("projekt-art-asm")}</option><option value="basic">{t("projekt-art-basic")}</option></select></label>
          {#if g.kind === "basic"}<label class="small dim">{t("projekt-gruppe-dialekt")}<input class="inp" data-pj="gruppe-dialekt" value={g.dialect} onchange={(e) => void setzeProjekt({ groupSet: { i: gi, dialect: e.currentTarget.value } })} /></label>{/if}
          <fieldset class="zf"><legend class="small dim">{t("projekt-gruppe-ziele")}</legend><span class="zl">{#each i.targets as z (z.id)}<label class="small"><input type="checkbox" data-pj="gruppe-ziel" data-z={z.id} checked={g.targets.includes(z.id)} onchange={(e) => void setzeProjekt({ groupSet: { i: gi, targets: zielWahl(g, z.id, e.currentTarget.checked) } })} />{z.system.toUpperCase()}</label>{/each}</span></fieldset>
          <button type="button" class="btn sm" data-pj="gruppe-weg" onclick={() => void setzeProjekt({ groupRemove: gi })}>{t("projekt-gruppe-entfernen")}</button>
        </div>
        <ol>
          {#each g.files as f, fi (f.file)}
            <li class:miss={f.missing} data-pj-quelle={f.file}>
              <span class="n">{fi + 1}</span>
              <button type="button" class="nm mono" onclick={() => void runCommand(`file:${f.file}`)} title={t("projekt-datei-oeffnen")}>{f.file}</button>
              {#if f.file === start}<span class="st" title={t("projekt-start-programm")}>★ {t("projekt-start-programm")}</span>{/if}
              {#if f.missing}<span class="mw">{t("projekt-fehlt")}</span>{/if}
              <span class="sp"></span>
              {#if f.file !== start && !f.missing}<button type="button" class="btn sm" data-pj-start={f.file} onclick={() => void setzeProjekt({ main: f.file })}>{t("projekt-als-start")}</button>{/if}
              <button type="button" class="ibtn" data-pj-weg={f.file} aria-label={t("projekt-quelle-entfernen", { f: f.file })} title={t("projekt-quelle-entfernen", { f: f.file })} onclick={() => void setzeProjekt({ fileRemove: f.file })}><Icon name="close" size={14} /></button>
              <button type="button" class="ibtn" data-pj-hoch={f.file} aria-label={t("projekt-nach-oben", { f: f.file })} disabled={fi === 0} onclick={() => void setzeProjekt({ move: { file: f.file, dir: -1 } })}><Icon name="arru" size={15} /></button>
              <button type="button" class="ibtn" data-pj-runter={f.file} aria-label={t("projekt-nach-unten", { f: f.file })} disabled={fi === g.files.length - 1} onclick={() => void setzeProjekt({ move: { file: f.file, dir: 1 } })}><Icon name="arrd" size={15} /></button>
            </li>
          {/each}
        </ol>
        <div class="row"><input class="inp grow" data-pj="datei-neu" list={`pj-dateien-${gi}`} placeholder={t("projekt-datei-hinzu")} aria-label={t("projekt-datei-hinzu")} value={neueDatei[gi] ?? ""} oninput={(e) => (neueDatei[gi] = e.currentTarget.value)} /><datalist id={`pj-dateien-${gi}`}>{#each dateien as d (d)}<option value={d}></option>{/each}</datalist>
          <button type="button" class="btn sm" data-pj="datei-hinzu" disabled={!(neueDatei[gi] ?? "").trim()} onclick={async () => { if (await setzeProjekt({ fileAdd: { group: gi, file: (neueDatei[gi] ?? "").trim() } })) neueDatei[gi] = ""; }}>{t("projekt-hinzufuegen")}</button></div>
      </section>
    {:else}<p class="dim" data-pj="keine-quellen">{t("projekt-keine-quellen")}</p>{/each}
    <div class="row"><button type="button" class="btn sm" data-pj="gruppe-neu-asm" onclick={() => void setzeProjekt({ groupAdd: { kind: "asm" } })}>{t("projekt-gruppe-neu-asm")}</button><button type="button" class="btn sm" data-pj="gruppe-neu-basic" onclick={() => void setzeProjekt({ groupAdd: { kind: "basic" } })}>{t("projekt-gruppe-neu-basic")}</button></div>
    <BasicCard />
  </div>
{/if}

<style>
  .q { display: flex; flex-direction: column; gap: 12px; max-width: 760px; } .small { font-size: 12px; margin: 0; } .dim { color: var(--dim); font-weight: normal; }
  .k { display: flex; flex-direction: column; gap: 6px; padding: 12px 14px; border-radius: 12px; border: 1px solid var(--line2); background: var(--card); } h3 { margin: 0; font-size: 13px; }
  ol { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 3px; } li { display: flex; align-items: center; gap: 8px; padding: 3px 4px; border-radius: 8px; } li:hover { background: var(--hover); }
  .n { color: var(--dim); width: 20px; text-align: right; font-size: 12px; } .nm { background: none; border: 0; color: var(--text); padding: 0; text-align: left; cursor: pointer; overflow-wrap: anywhere; } .nm:hover { text-decoration: underline; }
  .st { font-size: 11px; color: var(--accent2); } .mw { font-size: 11px; color: var(--err, #f85149); font-weight: 600; } li.miss .nm { color: var(--err, #f85149); } .sp { flex: 1; }
  .row { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; } .gk { align-items: flex-end; } .gk > label { display: flex; flex-direction: column; gap: 3px; } .zf label { display: inline-flex; flex-direction: row; align-items: center; gap: 4px; } .zf { flex-direction: column; align-items: flex-start; gap: 3px; } .zf .zl { display: flex; gap: 10px; } .grow { flex: 1; min-width: 160px; } .zf { border: 0; margin: 0; padding: 0; display: flex; gap: 10px; align-items: center; } .zf legend { padding: 0; }
  .inp { height: 28px; padding: 0 8px; border-radius: var(--r-ctl); border: 1px solid var(--line2); background: var(--raise); color: var(--text); font: inherit; }
</style>
