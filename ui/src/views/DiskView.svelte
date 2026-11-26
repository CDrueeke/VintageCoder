<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { addFiles, answerAsk, askThen, back, closeDisk, diskOp, diskRedo, diskUndo, dk, fileMove, follow, importMarked, insertDisk, pickDisk, saveDisk, saveDiskAs, selectFile, selectSector, setChain, validate } from "../lib/disk.svelte";
  import { dnd, dragSource, dropTarget } from "../lib/dnd.svelte";
  import NewDiskDialog from "../lib/components/NewDiskDialog.svelte";
  import Dialog from "../lib/components/Dialog.svelte";
  import { ui } from "../lib/state.svelte";
  import { t } from "../lib/i18n.svelte";
  import DiskMap from "../lib/components/DiskMap.svelte";
  import Icon from "../lib/components/Icon.svelte";
  // Disk-Editor (E2.1): links Diskette und Belegung, Mitte Verzeichnis und Belegungskarte, rechts Sektor-Inspektor (nach Mockup neu-11)
  const v = $derived(dk.view);
  let neu = $state(false);
  let askOpen = $state(false);
  $effect(() => { askOpen = dk.ask !== null; });
  $effect(() => { if (!askOpen && dk.ask) void answerAsk("cancel"); });
  const hx = (b: number): string => b.toString(16).toUpperCase().padStart(2, "0");
  const asc = (s: string): string => s;
  const formats = ["D64", "D71", "D81", "ATR"];
  const pct = $derived(v && v.total ? Math.round(((v.total - v.free) / v.total) * 100) : 0);
  /** Zeile über der die gezogene Zeile gerade schwebt (Einfügemarke); die Marke liegt unter der Zeile, wenn von oben gezogen wird, sonst darüber */
  let ueber = $state<number | null>(null);
  const markeUnten = $derived(dnd.payload?.kind === "diskfile" && ueber !== null ? Number(dnd.payload.id) < ueber : false);
  async function verschiebe(von: number, nach: number) {
    if (!v || nach < 0 || nach >= v.files.length || von === nach) return;
    if (await fileMove(von, nach)) queueMicrotask(() => document.querySelectorAll<HTMLElement>(".dir tbody tr")[nach]?.focus());
  }
  function rowKey(e: KeyboardEvent, i: number) {
    if (!v) return;
    if (e.altKey && (e.key === "ArrowDown" || e.key === "ArrowUp")) { e.preventDefault(); if (v.canOrder) void verschiebe(i, i + (e.key === "ArrowDown" ? 1 : -1)); return; } const d = e.key === "ArrowDown" ? 1 : e.key === "ArrowUp" ? -1 : 0; if (!d) return; e.preventDefault();
    const j = Math.max(0, Math.min(v.files.length - 1, i + d)); void selectFile(j);
    queueMicrotask(() => document.querySelectorAll<HTMLElement>(".dir tbody tr")[j]?.focus());
  }
</script>

{#if !v}
  <div class="blank">
    <Icon name="memory" size={28} />
    <h2>{t("disk-leer-titel")}</h2>
    <p>{t("disk-leer-text")}</p>
    <div class="row"><button type="button" class="btn pri" onclick={() => void pickDisk()}>{t("disk-oeffnen")}</button><button type="button" class="btn" onclick={() => (neu = true)}>{t("disk-neu")}</button></div>
  </div>
{:else}
  <div class="disk" aria-label={t("view-title-disk")}>
    <section class="col l" aria-label={t("disk-karte-diskette")}>
      <h3>{t("disk-karte-diskette")}</h3>
      <div class="fmts" role="group" aria-label={t("disk-karte-diskette")}>{#each formats as f (f)}<span class="fmt" class:on={f === v.format} aria-current={f === v.format ? "true" : undefined}>{f}</span>{/each}</div>
      <div class="img"><Icon name="file" size={15} /><span class="grow nm" title={v.path ?? v.title}>{v.dirty ? "● " : ""}{v.title}</span>{#if v.size}<span class="dim">{v.size}</span>{/if}</div>
      <p class="dim small">{v.geometry}</p>
      <h3>{t("disk-belegung")}</h3>
      <div class="bar" role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow={pct} aria-label={t("disk-belegung")}><span style={`width:${pct}%`}></span></div>
      <p class="small">{t("disk-belegt-frei", { used: v.used, free: v.free })}</p>
      {#if v.problems.length}<p class="bad small" role="status">{v.problems.length} ⚠</p>{/if}
      {#if v.dirty}<p class="warn small" role="status">{t("disk-ungespeichert")}</p>{/if}
      <h3>{t("disk-aktionen")}</h3>
      <div class="acts">
        <button type="button" class="btn pri" disabled={v.dirty || !v.path} title={v.dirty || !v.path ? t("disk-einlegen-erst") : ""} onclick={() => void insertDisk()}>{t("disk-einlegen")}</button>
        <button type="button" class="btn" disabled={!v.fs} title={v.fs ? "" : t("disk-ohne-dateisystem")} onclick={() => void addFiles()}>{t("disk-ablegen")}</button>
        <button type="button" class="btn" onclick={() => void validate()}>{t("disk-pruefen")}</button>
        <button type="button" class="btn" disabled={!ui.project || v.checkedCount === 0 || v.opts.dirBad} title={!ui.project ? t("disk-projekt-noetig") : ""} onclick={() => void importMarked()}>{t("disk-uebernehmen")}{v.checkedCount ? ` (${v.checkedCount})` : ""}</button>
        <button type="button" class="btn" disabled={!v.dirty} onclick={() => void saveDisk()}>{t("disk-speichern")}</button>
        <button type="button" class="btn" onclick={() => void saveDiskAs()}>{t("disk-speichern-unter")}</button>
        <button type="button" class="btn" onclick={() => void pickDisk()}>{t("disk-oeffnen")}</button>
        <button type="button" class="btn" onclick={() => (neu = true)}>{t("disk-neu")}</button>
        <button type="button" class="btn" onclick={() => askThen(closeDisk)}>{t("disk-schliessen")}</button>
      </div>
      {#if v.checkedCount > 0}
        <h3>{t("disk-uebernahme")}</h3>
        <label class="fld small"><span>{t("disk-ziel")}</span><input class="inp" value={v.opts.dir} aria-invalid={v.opts.dirBad} onchange={(e) => void diskOp({ kind: "opts", dir: e.currentTarget.value })} /></label>
        {#if v.opts.dirBad}<p class="bad small" role="alert">{t("disk-ziel-ungueltig")}</p>{/if}
        <label class="chk small"><input type="checkbox" checked={v.opts.basic} onchange={(e) => void diskOp({ kind: "opts", basic: e.currentTarget.checked })} />{t("disk-opt-basic")}</label>
        <label class="chk small"><input type="checkbox" checked={v.opts.raw} onchange={(e) => void diskOp({ kind: "opts", raw: e.currentTarget.checked })} />{t("disk-opt-raw")}</label>
        <label class="chk small"><input type="checkbox" checked={v.opts.asm} onchange={(e) => void diskOp({ kind: "opts", asm: e.currentTarget.checked })} />{t("disk-opt-asm")}</label>
      {/if}
      {#if v.report}
        <h3>{t("disk-bericht")}</h3>
        {#if v.report.length === 0}<p class="small ok" role="status">{t("disk-bericht-ok")}</p>{:else}<ul class="rep small">{#each v.report as l, i (i)}<li>{l}</li>{/each}</ul>{/if}
      {/if}
      {#if v.importResult}
        <h3>{t("disk-uebernahme")}</h3>
        <ul class="rep small">{#each v.importResult as l, i (i)}<li class:bad={!l.ok}>{l.text}</li>{/each}</ul>
      {/if}
    </section>

    <section class="col m" aria-label={t("disk-verzeichnis")}>
      <div class="hd"><h3>{t("disk-verzeichnis")}</h3><span class="dim small">{t("disk-n-dateien", { n: v.files.length })}</span></div>
      <div class="vm small"><span class="dim">{t("disk-ordnen-hinweis")}</span>
        <button type="button" class="btn sm" data-dk="hoch" disabled={!v.canOrder || v.selFile === null || v.selFile < 1} onclick={() => void verschiebe(v.selFile ?? 0, (v.selFile ?? 0) - 1)}>{t("disk-datei-hoch")}</button>
        <button type="button" class="btn sm" data-dk="runter" disabled={!v.canOrder || v.selFile === null || v.selFile >= v.files.length - 1} onclick={() => void verschiebe(v.selFile ?? 0, (v.selFile ?? 0) + 1)}>{t("disk-datei-runter")}</button>
        <button type="button" class="btn sm" data-act="undo" disabled={!v.canUndo} onclick={() => void diskUndo()}>{t("cmd-rueckgaengig")}</button>
        <button type="button" class="btn sm" data-act="redo" disabled={!v.canRedo} onclick={() => void diskRedo()}>{t("cmd-wiederholen")}</button></div>
      <div class="vm small"><span class="dim">{t("disk-vormerken")}:</span>
        <button type="button" class="btn sm" onclick={() => void diskOp({ kind: "check_all" })}>{t("disk-vm-alle")}</button>
        <button type="button" class="btn sm" onclick={() => void diskOp({ kind: "check_none" })}>{t("disk-vm-keine")}</button>
        <button type="button" class="btn sm" onclick={() => void diskOp({ kind: "check_prg" })}>{t("disk-vm-prg")}</button>
        <button type="button" class="btn sm" onclick={() => void diskOp({ kind: "check_basic" })}>{t("disk-vm-basic")}</button></div>
      <div class="tw"><table class="dir">
        <thead><tr><th class="ck"><span class="sr">{t("disk-vormerken")}</span></th><th>{t("disk-spalte-name")}</th><th>{t("disk-spalte-typ")}</th><th class="r">{t("disk-spalte-bloecke")}</th><th class="r">{t("disk-spalte-groesse")}</th><th class="r">{t("disk-spalte-start")}</th></tr></thead>
        <tbody>{#each v.files as f (f.i)}
          <tr tabindex="0" class:sel={v.selFile === f.i} class:ueber-oben={ueber === f.i && !markeUnten} class:ueber-unten={ueber === f.i && markeUnten} data-dk="row" data-i={f.i} aria-selected={v.selFile === f.i} aria-roledescription={v.canOrder ? t("disk-ordnen-zeile", { name: f.name }) : undefined}
            use:dragSource={() => ({ kind: v.canOrder ? "diskfile" : "none", id: String(f.i), label: f.name })} use:dropTarget={() => ({ accept: (p) => p.kind === "diskfile" && Number(p.id) !== f.i, enter: () => { ueber = f.i; }, leave: () => { if (ueber === f.i) ueber = null; }, drop: (p) => { ueber = null; void verschiebe(Number(p.id), f.i); } })} onclick={() => void selectFile(f.i)} onkeydown={(e) => { if (e.key === "Enter" || e.key === " ") { e.preventDefault(); void selectFile(f.i); } else rowKey(e, f.i); }}>
            <td class="ck"><input type="checkbox" checked={f.checked} aria-label={t("disk-vormerken-datei", { name: f.name })} onclick={(e) => e.stopPropagation()} onchange={(e) => void diskOp({ kind: "check", i: f.i, on: e.currentTarget.checked })} /></td>
            <td class="mono nm">{asc(f.name)}{#if f.locked}<span class="chip" title={t("disk-geschuetzt")}>🔒</span>{/if}{#if f.auto}<span class="chip">{t("disk-autostart")}</span>{/if}</td>
            <td class="mono acc">{f.kind}</td><td class="r mono">{f.blocks}</td><td class="r mono dim">{f.size ?? ""}</td><td class="r mono dim">{f.start}</td>
          </tr>
        {/each}</tbody>
      </table></div>
      <div class="hd"><h3>{t("disk-belegungskarte")}</h3>
        <span class="leg small dim"><i class="d s1"></i>{t("disk-legende-datei")} <i class="d s2"></i>{t("disk-legende-belegt")} <i class="d s3"></i>{t("disk-legende-verz")} <i class="d s4"></i>{t("disk-legende-kopf")}</span>
        <label class="chain small"><input type="checkbox" checked={v.chainOn} onchange={(e) => void setChain(e.currentTarget.checked)} />{t("disk-kette-zeigen")}</label>
      </div>
      <DiskMap view={v} onselect={(tr, s) => void selectSector(tr, s)} />
    </section>

    <section class="col r" aria-label={v.title2}>
      <div class="hd hdsek"><h3>{v.title2}</h3>
        <span class="row"><button type="button" class="btn sm" disabled={!v.canBack} onclick={() => void back()}>{t("disk-zurueck")}</button>
          <button type="button" class="btn sm folgen" disabled={!v.next} onclick={() => void follow()}>{v.next ? t("disk-folgen-zu", { a: v.next.text }) : t("disk-kettenende")}</button></span></div>
      {#if v.chainPos}<p class="small">{t("disk-datei-block", { name: v.selFile !== null ? v.files[v.selFile]?.name ?? "" : "", n: v.chainPos[0], m: v.chainPos[1] })}</p>{:else if v.owner}<p class="small">{v.owner}</p>{/if}
      {#if v.hex}
        <div class="hex mono" role="table" aria-label={v.title2}>{#each v.hex as r (r.off)}
          <div class="hr" role="row"><span class="of dim" role="cell">{hx(r.off)}</span><span class="by" role="cell">{#each r.bytes as b, i (i)}<span class:ptr={b[1]}>{hx(b[0])}</span>{/each}</span><span class="tx dim" role="cell">{r.text}</span></div>
        {/each}</div>
      {/if}
      <dl class="kv small">
        {#if v.pointer}<div><dt>{v.pointer}</dt></div>{/if}
        <div><dt>{t("disk-naechster-sektor")}</dt><dd class="mono">{v.next ? v.next.text : "–"}</dd></div>
        {#if v.bytesInSector !== null}<div><dt>{t("disk-bytes-im-sektor")}</dt><dd class="mono">{v.bytesInSector}</dd></div>{/if}
      </dl>
    </section>
  </div>
{/if}

<NewDiskDialog bind:open={neu} />
{#if dk.ask}
  <Dialog bind:open={askOpen} title={t("disk-frage-titel")}>
    {#snippet children()}<p>{t("disk-frage-text", { name: v?.title ?? "" })}</p>{/snippet}
    {#snippet footer()}
      <button type="button" class="btn" onclick={() => void answerAsk("cancel")}>{t("dialog-abbrechen")}</button>
      <button type="button" class="btn" onclick={() => void answerAsk("discard")}>{t("close-verwerfen")}</button>
      <button type="button" class="btn pri" onclick={() => void answerAsk("save")}>{t("close-speichern")}</button>
    {/snippet}
  </Dialog>
{/if}

<style>
  .blank { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 8px; color: var(--dim); text-align: center; padding: 20px; }
  .blank h2 { color: var(--text); font-size: 18px; } .blank p { max-width: 420px; }
  .disk { flex: 1; min-height: 0; display: grid; grid-template-columns: 240px minmax(0, 1fr) 300px; gap: 10px; padding: 10px; overflow: auto; container-type: inline-size; }
  @media (max-width: 1100px) { .disk { grid-template-columns: 220px minmax(0, 1fr); } .col.r { grid-column: 1 / -1; } }
  @media (max-width: 700px) { .disk { grid-template-columns: 1fr; } }
  .col { background: var(--card); border: 1px solid var(--line2); border-radius: 12px; padding: 12px; min-width: 0; display: flex; flex-direction: column; gap: 8px; }
  h3 { margin: 4px 0 0; font-size: 11px; letter-spacing: 0.07em; text-transform: uppercase; color: var(--dim); }
  /* Kopf des Sektor-Inspektors: eine Zeile, die Knöpfe behalten ihren Platz (der Titel kürzt, nicht die Knöpfe) */
  .hdsek { flex-wrap: nowrap; } .hdsek h3 { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-variant-numeric: tabular-nums; } .hdsek .row { flex: none; white-space: nowrap; }
  .hdsek .btn { white-space: nowrap; font-variant-numeric: tabular-nums; } .hdsek .folgen { min-width: 11ch; text-align: center; }
  .hd { display: flex; align-items: center; justify-content: space-between; gap: 8px; flex-wrap: wrap; } .row { display: flex; gap: 6px; }
  .fmts { display: flex; background: var(--raise); border-radius: 9px; padding: 2px; } .fmt { flex: 1; text-align: center; padding: 4px 0; border-radius: 7px; font-size: 12px; color: var(--dim); } .fmt.on { background: var(--card); color: var(--text); border: 1px solid var(--line2); }
  .img { display: flex; align-items: center; gap: 8px; padding: 7px 9px; background: var(--accentbg); border-radius: 8px; } .grow { flex: 1; min-width: 0; } .nm { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .bar { height: 8px; background: var(--raise); border-radius: 5px; overflow: hidden; } .bar span { display: block; height: 100%; background: var(--accent); }
  .acts { display: flex; flex-direction: column; gap: 6px; } .acts .btn { justify-content: flex-start; }
  .vm { display: flex; align-items: center; gap: 6px; flex-wrap: wrap; } .ck { width: 28px; text-align: center; } .sr { position: absolute; width: 1px; height: 1px; overflow: hidden; clip-path: inset(50%); }
  .fld span { display: block; color: var(--dim); margin-bottom: 3px; } .inp { width: 100%; } .chk { display: flex; gap: 6px; align-items: center; } .warn { color: var(--warn); } .ok { color: var(--ok); }
  .rep { margin: 0; padding-left: 16px; display: flex; flex-direction: column; gap: 2px; } .rep li.bad { color: var(--err); }
  .dir tbody tr.ueber-oben { box-shadow: inset 0 2px 0 var(--accent); } .dir tbody tr.ueber-unten { box-shadow: inset 0 -2px 0 var(--accent); }   /* Einfügemarke beim Ziehen */
  .dim { color: var(--dim); } .small { font-size: 12px; } .bad { color: var(--err); }
  .tw { overflow: auto; max-height: 40vh; border: 1px solid var(--line); border-radius: 8px; }
  table { width: 100%; border-collapse: collapse; font-size: 12.5px; } th { position: sticky; top: 0; background: var(--card); text-align: left; font-size: 10.5px; letter-spacing: 0.06em; text-transform: uppercase; color: var(--dim); padding: 6px 8px; border-bottom: 1px solid var(--line); }
  td { padding: 5px 8px; border-bottom: 1px solid var(--line); } .r { text-align: right; } .acc { color: var(--accent); }
  tbody tr { cursor: pointer; } tbody tr:hover { background: var(--hover); } tbody tr.sel { background: var(--accentbg); } tbody tr:focus-visible { outline: 2px solid var(--accent); outline-offset: -2px; }
  .chip { margin-left: 6px; padding: 0 6px; font-size: 10.5px; border: 1px solid var(--line2); border-radius: 8px; color: var(--dim); }
  .leg { display: flex; gap: 10px; align-items: center; flex-wrap: wrap; } .d { display: inline-block; width: 10px; height: 8px; border-radius: 2px; margin-right: 3px; } .s1 { background: var(--accent); } .s2 { background: var(--faint); } .s3 { background: var(--warn); } .s4 { background: var(--ok); }
  .chain { display: flex; align-items: center; gap: 5px; }
  .hex { display: flex; flex-direction: column; gap: 1px; font-size: 12px; } .hr { display: flex; gap: 10px; } .of { width: 22px; } .by { display: flex; gap: 6px; flex: 1; } .by .ptr { color: var(--warn); } .tx { white-space: pre; }
  .kv { display: flex; flex-direction: column; gap: 4px; margin: 4px 0 0; } .kv > div { display: flex; justify-content: space-between; gap: 8px; border-top: 1px solid var(--line); padding-top: 4px; } dd { margin: 0; }
</style>
