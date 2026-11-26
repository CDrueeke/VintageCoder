<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { aktiv, beobachtenSetzen, chipsWaehlen, bildHolen, bildModus, bildQuelle, dbg, debugEnde, debugStarten, hex, hinein, momentErkennen, momentSichern, pause, punktHinzu, punktWeg, raus, speicherAnzeigen, textSenden, ueber, weiter, zumQuelltext } from "../lib/debug.svelte";
  import Icon from "../lib/components/Icon.svelte";
  import Switch from "../lib/components/Switch.svelte";
  import Tooltip from "../lib/components/Tooltip.svelte";
  import { alleHaltepunkte } from "../lib/editor.svelte";
  import { bau } from "../lib/bau.svelte";
  import { t } from "../lib/i18n.svelte";
  import { ui } from "../lib/state.svelte";
  // Debug (E6, nach Mockup neu-14): links Haltepunkte, Beobachten und Momentaufnahme, in der Mitte Symbolleiste, Disassembler und Speicher, rechts Register, Flags, Stapel und das Bild des Emulators.
  const s = $derived(dbg.s);
  /** Tooltip mit Name und Taste des Befehls (am Mac gelten andere Tasten als unter Windows und Linux) */
  const tip = (id: string): string => { const k = ui.commands.find((c) => c.id === id)?.keysDisplay; const n = t(`cmd-${id}`); return k ? t("dbgui-tooltip", { name: n, taste: k }) : n; };
  const halt = $derived(!!s && !s.running && !!s.haveRegs);
  const status = $derived(!s ? "" : s.starting ? t("dbgui-status-startet") : s.running ? t("dbgui-status-laeuft") : s.haveRegs ? t("dbgui-status-halt", { pc: hex(s.regs!.pc, 4), ort: s.ort ? `${s.ort.file}:${s.ort.line}` : (s.disasm?.[0]?.label ?? "") }) : t("dbgui-status-wartet"));
  let memText = $state(""); let beobText = $state(""); let sendText = $state("");
  let pkind = $state("exec"); let pAdr = $state(""); let pEnde = $state(""); let pBed = $state(""); let pIgn = $state(0); let pFehler = $state("");
  let snapName = $state("momentaufnahme"); let snapSprites = $state(true); let snapTiles = $state(false); let snapRom = $state(false);
  const bytes = (b: number[]) => b.map((x) => hex(x, 2)).join(" ");
  const asc = (b: number) => (b >= 0x20 && b < 0x7f ? String.fromCharCode(b) : "·");
  const reg = $derived(s?.regs ? [["PC", s.regs.pc, 4], ["A", s.regs.a, 2], ["X", s.regs.x, 2], ["Y", s.regs.y, 2], ["SP", s.regs.sp, 2]] as [string, number, number][] : []);
  const geaendert = (i: number) => !!s?.regs?.prev && s.regs.prev[i] !== [s.regs.pc, s.regs.a, s.regs.x, s.regs.y, s.regs.sp, s.regs.flags][i];
  const flags = "NV-BDIZC".split("");
  const memZeilen = $derived.by(() => { const m = s?.mem; if (!m) return []; return Array.from({ length: Math.ceil(m.bytes.length / 16) }, (_, r) => ({ addr: m.addr + r * 16, b: m.bytes.slice(r * 16, r * 16 + 16), prev: m.prev.length === m.bytes.length ? m.prev.slice(r * 16, r * 16 + 16) : [] })); });
  const editorBps = $derived(Object.entries(alleHaltepunkte()).flatMap(([f, ls]) => ls.map((l) => ({ f, l }))));
  const adr = (x: string): number | null => { const v = x.trim().replace(/^\$|^0x/i, ""); return /^[0-9a-f]{1,4}$/i.test(v) ? parseInt(v, 16) : null; };
  async function hinzu() {
    pFehler = ""; const a = adr(pAdr); const b = pEnde.trim() ? adr(pEnde) : a;
    if (a === null || b === null) { pFehler = t("dbgui-punkt-adresse"); return; }
    if (await punktHinzu({ pkind, start: a, end: b, cond: pBed, ignore: pIgn })) { pAdr = ""; pEnde = ""; pBed = ""; pIgn = 0; }
  }
  const kindText = (k: string) => ({ exec: t("dbgui-art-exec"), read: t("dbgui-art-read"), write: t("dbgui-art-write"), access: t("dbgui-art-access") })[k] ?? k;
  async function beobHinzu() { const x = beobText.trim(); if (!x) return; await beobachtenSetzen([...dbg.watches, x]); beobText = ""; }
  // Chip-Fenster: Register der gewählten Chips mit dem gelesenen Wert (nur im Halt; im Lauf stehen die letzten Werte da)
  const chipWert = (addr: number): number | null => { for (const w of s?.chips ?? []) if (addr >= w.addr && addr < w.addr + w.bytes.length) return w.bytes[addr - w.addr] ?? null; return null; };
  const chipWahl = (id: string, an: boolean) => void chipsWaehlen(an ? [...new Set([...dbg.chipSel, id])] : dbg.chipSel.filter((x) => x !== id));
  const zahl = (v: number) => `$${hex(v >>> 0 & 0xffff, v > 0xff ? 4 : 2)} · ${v}`;
</script>

{#if !aktiv()}
  <div class="blank" data-dbg="empty">
    <Icon name="bug" size={28} />
    <h2>{t("dbgui-leer-titel")}</h2>
    <p>{t("dbgui-leer-text")}</p>
    {#if bau.last && !bau.last.ok}<p class="warn small">{t("dbgui-bau-fehlt")}</p>{/if}
    <div class="row"><button type="button" class="btn pri" data-dbg="start" disabled={dbg.starting || bau.running} onclick={() => void debugStarten()}>{dbg.starting ? t("dbgui-startet") : t("dbgui-start")}</button></div>
    <p class="small dim">{t("dbgui-leer-hinweis")}</p>
  </div>
{:else if s}
  <div class="db" aria-label={t("view-title-debug")}>
    <section class="col l" aria-label={t("dbgui-links")}>
      <h3>{t("dbgui-haltepunkte")}</h3>
      <ul class="liste" data-dbg="punkte">
        {#each s.points ?? [] as p (p.id)}
          <li class="pt"><span class="art">{kindText(p.kind)}</span><span class="mono">${hex(p.start, 4)}{p.end !== p.start ? `–$${hex(p.end, 4)}` : ""}</span>{#if p.cond}<span class="dim small" title={t("dbgui-bedingung")}>if {p.cond}</span>{/if}{#if p.ignore}<span class="dim small">×{p.ignore}</span>{/if}{#if p.hits}<span class="dim small" title={t("dbgui-treffer")}>{p.hits}×</span>{/if}
            <button type="button" class="ibtn" aria-label={t("dbgui-punkt-loeschen")} data-dbg="punkt-weg" onclick={() => void punktWeg(p.id)}><Icon name="x" size={14} /></button></li>
        {/each}
        {#each editorBps as b (b.f + ":" + b.l)}<li class="pt dim small" data-dbg="editor-bp"><span class="art">{t("dbgui-editor")}</span><span>{b.f}:{b.l}</span></li>{/each}
        {#if !(s.points?.length) && !editorBps.length}<li class="dim small">{t("dbgui-keine-punkte")}</li>{/if}
      </ul>
      {#if s.condError}<p class="warn small" role="alert" data-dbg="bed-fehler">{s.condError}</p>{/if}
      <div class="form">
        <select class="inp" data-dbg="art" aria-label={t("dbgui-art")} bind:value={pkind}><option value="exec">{t("dbgui-art-exec")}</option><option value="read" disabled={!s.caps?.watch}>{t("dbgui-art-read")}</option><option value="write" disabled={!s.caps?.watch}>{t("dbgui-art-write")}</option><option value="access" disabled={!s.caps?.watch}>{t("dbgui-art-access")}</option></select>
        <input class="inp" data-dbg="adr" placeholder={t("dbgui-adresse")} aria-label={t("dbgui-adresse")} bind:value={pAdr} />
        {#if pkind !== "exec"}<input class="inp" data-dbg="ende" placeholder={t("dbgui-ende")} aria-label={t("dbgui-ende")} bind:value={pEnde} />{/if}
        <input class="inp" data-dbg="bed" placeholder={t("dbgui-bedingung")} aria-label={t("dbgui-bedingung")} bind:value={pBed} />
        <label class="small dim">{t("dbgui-ignorieren")} <input class="inp num" type="number" min="0" data-dbg="ign" bind:value={pIgn} /></label>
        <button type="button" class="btn sm" data-dbg="punkt-neu" onclick={() => void hinzu()}>{t("dbgui-hinzufuegen")}</button>
        {#if pFehler}<p class="warn small" role="alert">{pFehler}</p>{/if}
      </div>

      <h3>{t("dbgui-beobachten")}</h3>
      <ul class="liste" data-dbg="beobachten">
        {#each s.watch ?? [] as w, i (w.expr + i)}
          <li class="pt"><span class="nm">{w.expr}</span>{#if w.error}<span class="warn small" data-dbg="beob-fehler">{w.error}</span>{:else if w.value !== undefined}<b class="mono" data-dbg="beob-wert">{zahl(w.value)}</b>{:else}<span class="dim small">…</span>{/if}
            <button type="button" class="ibtn" aria-label={t("dbgui-beob-weg")} data-dbg="beob-weg" onclick={() => void beobachtenSetzen(dbg.watches.filter((_, k) => k !== i))}><Icon name="x" size={14} /></button></li>
        {/each}
        {#if !dbg.watches.length}<li class="dim small">{t("dbgui-keine-beobachtungen")}</li>{/if}
      </ul>
      <div class="row"><input class="inp" data-dbg="beob" placeholder={t("dbgui-beob-eingabe")} aria-label={t("dbgui-beob-eingabe")} bind:value={beobText} onkeydown={(e) => { if (e.key === "Enter") void beobHinzu(); }} /><button type="button" class="btn sm" data-dbg="beob-neu" onclick={() => void beobHinzu()}>{t("dbgui-hinzufuegen")}</button></div>

      <h3>{t("dbgui-moment")}</h3>
      {#if !halt}<p class="dim small">{t("dbgui-moment-halt")}</p>{:else}
        <button type="button" class="btn sm" data-dbg="erkennen" onclick={() => void momentErkennen()}>{t("dbgui-erkennen")}</button>
        {#if s.detected?.error}<p class="warn small" role="alert">{s.detected.error}</p>
        {:else if s.detected}
          <div class="small" data-dbg="erkannt">
            <div class="mono">{s.detected.mode || s.detected.system} · {t("dbgui-bildschirm-bei")} ${hex(s.detected.screen ?? 0, 4)}</div>
            <div class="mono dim">{s.detected.charset != null ? t("dbgui-zeichensatz-bei", { a: hex(s.detected.charset, 4) }) : s.detected.romOff != null ? t("dbgui-zeichensatz-rom") : t("dbgui-zeichensatz-unbekannt")}{s.detected.charsetMulticolor ? " · " + t("dbgui-mehrfarbig") : ""}</div>
            <div class="mono dim">{t("dbgui-sprites-n", { n: s.detected.sprites ?? 0 })}</div>
            {#each s.detected.notes ?? [] as n (n)}<div class="dim">{n}</div>{/each}
          </div>
          <Switch bind:checked={snapSprites} label={t("dbgui-moment-sprites")} />
          <Switch bind:checked={snapTiles} label={t("dbgui-moment-kacheln")} />
          {#if s.detected.romOff != null && s.detected.charset == null}<Switch bind:checked={snapRom} label={t("dbgui-moment-rom")} />{/if}
          <input class="inp" data-dbg="snap-name" aria-label={t("dbgui-moment-name")} bind:value={snapName} />
          <button type="button" class="btn sm" data-dbg="snap-sichern" onclick={() => void momentSichern({ name: snapName, sprites: snapSprites, tiles: snapTiles, rom: snapRom })}>{t("dbgui-moment-sichern")}</button>
        {/if}
      {/if}
      <h3>{t("dbgui-chips")}</h3>
      <div class="chipsw" data-dbg="chips-wahl">{#each dbg.chipInfo as c (c.id)}<label class="small"><input type="checkbox" data-dbg="chip-an" data-c={c.id} checked={dbg.chipSel.includes(c.id)} onchange={(e) => chipWahl(c.id, e.currentTarget.checked)} />{c.name}</label>{:else}<span class="dim small">{t("dbgui-chips-keine")}</span>{/each}</div>
      {#each dbg.chipInfo.filter((c) => dbg.chipSel.includes(c.id)) as c (c.id)}
        <div class="chip-fenster" data-dbg="chip" data-c={c.id} role="table" aria-label={c.name}>
          <div class="small"><b>{c.name}</b></div>
          {#each c.regs as r (r.addr + r.name)}{@const v = chipWert(r.addr)}
            <div class="cr mono" role="row" title={r.doc}><span class="dim">${hex(r.addr, 4)}</span><span class="grow">{r.name}</span><span data-dbg="chip-wert">{v === null ? "–" : `$${hex(v, 2)}`}</span></div>
          {/each}
        </div>
      {/each}
    </section>

    <section class="col m" aria-label={t("dbgui-mitte")}>
      <div class="bar" role="toolbar" aria-label={t("dbgui-werkzeuge")}>
        <Tooltip text={tip("debug_weiter")}><button type="button" class="ibtn go" data-dbg="weiter" aria-label={t("cmd-debug_weiter")} disabled={s.running || s.starting} onclick={() => void weiter()}><Icon name="play" size={17} /><span class="kt">{t("dbgui-knopf-weiter")}</span></button></Tooltip>
        <Tooltip text={tip("debug_pause")}><button type="button" class="ibtn" data-dbg="pause" aria-label={t("cmd-debug_pause")} disabled={!s.running || !s.canPause} onclick={() => void pause()}><Icon name="pause" size={17} /><span class="kt">{t("dbgui-knopf-pause")}</span></button></Tooltip>
        <Tooltip text={tip("debug_ueber")}><button type="button" class="ibtn" data-dbg="ueber" aria-label={t("cmd-debug_ueber")} disabled={!halt} onclick={() => void ueber()}><Icon name="arrr" size={17} /><span class="kt">{t("dbgui-knopf-ueber")}</span></button></Tooltip>
        <Tooltip text={tip("debug_hinein")}><button type="button" class="ibtn" data-dbg="hinein" aria-label={t("cmd-debug_hinein")} disabled={!halt} onclick={() => void hinein()}><Icon name="arrd" size={17} /><span class="kt">{t("dbgui-knopf-hinein")}</span></button></Tooltip>
        <Tooltip text={tip("debug_raus")}><button type="button" class="ibtn" data-dbg="raus" aria-label={t("cmd-debug_raus")} disabled={!halt} onclick={() => void raus()}><Icon name="arru" size={17} /><span class="kt">{t("dbgui-knopf-raus")}</span></button></Tooltip>
        <Tooltip text={tip("debug_ende")}><button type="button" class="ibtn stop" data-dbg="ende" aria-label={t("cmd-debug_ende")} onclick={() => void debugEnde()}><Icon name="stop" size={17} /><span class="kt">{t("dbgui-knopf-ende")}</span></button></Tooltip>
        <span class="sep"></span>
        {#if s.test}<span class="chip" data-dbg="test-sitzung">{t("dbgui-test-sitzung")}</span>{/if}<span class="chip" class:run={s.running} data-dbg="status" role="status">{status}</span>
        <span class="grow"></span>
        <span class="dim small">{s.backend}</span>
      </div>
      {#if s.stepBlock}<p class="warn small" data-dbg="schritt-block">{s.stepBlock}</p>{/if}
      <div class="dis" data-dbg="disasm" role="list" aria-label={t("dbgui-disasm")}>
        {#if !halt}<p class="dim small pad">{s.running ? t("dbgui-laeuft-hinweis") : t("dbgui-status-wartet")}</p>{/if}
        {#each halt ? (s.disasm ?? []) : [] as l (l.addr)}
          {#if l.label}<div class="lab mono" role="listitem">{l.label}:</div>{/if}
          <div class="dz mono" class:cur={l.addr === s.regs?.pc} class:ill={l.illegal} role="listitem" data-dbg="row" data-a={l.addr}>
            <button type="button" class="bp" class:an={l.bp} aria-pressed={l.bp} aria-label={t("dbgui-haltepunkt-bei", { a: hex(l.addr, 4) })} data-dbg="bp" onclick={() => void (l.bp ? punktWeg(s.points?.find((p) => p.kind === "exec" && p.start === l.addr)?.id ?? 0) : punktHinzu({ pkind: "exec", start: l.addr }))}></button>
            <span class="ad">{hex(l.addr, 4)}</span><span class="by dim">{bytes(l.bytes)}</span><span class="tx">{l.text}</span>
            <span class="cy dim">{l.cycles ? `${l.cycles}${l.extra ? "+" : ""}` : ""}</span>
            {#if l.line && l.addr === s.regs?.pc}<button type="button" class="lnk" data-dbg="quelle" onclick={() => void zumQuelltext(l.line!.file, l.line!.line)}>{l.line.file}:{l.line.line}</button>{/if}
          </div>
        {/each}
      </div>
      <div class="bar"><h3>{t("dbgui-speicher")} ${hex(s.mem?.addr ?? 0, 4)}</h3><span class="grow"></span>
        <input class="inp adrf" data-dbg="mem" placeholder={t("dbgui-mem-eingabe")} aria-label={t("dbgui-mem-eingabe")} bind:value={memText} onkeydown={(e) => { if (e.key === "Enter") void speicherAnzeigen(memText); }} /><button type="button" class="btn sm" data-dbg="mem-los" onclick={() => void speicherAnzeigen(memText)}>{t("dbgui-mem-los")}</button></div>
      <div class="mem mono" data-dbg="speicher" role="table" aria-label={t("dbgui-speicher")}>
        {#if !halt}<p class="dim small">{t("dbgui-laeuft-hinweis")}</p>{/if}
        {#each halt ? memZeilen : [] as z (z.addr)}
          <div class="mz" role="row"><span class="ad dim" role="rowheader">{hex(z.addr, 4)}</span>
            <span class="hx">{#each z.b as b, i (i)}<i class:chg={z.prev.length && z.prev[i] !== b}>{hex(b, 2)}</i>{/each}</span>
            <span class="as dim">{z.b.map(asc).join("")}</span></div>
        {/each}
      </div>
    </section>

    <section class="col r" aria-label={t("dbgui-rechts")}>
      <h3>{t("dbgui-register")}<span class="pill" class:run={s.running}>{s.running ? t("dbgui-status-laeuft") : t("dbgui-angehalten")}</span></h3>
      {#if !halt}<p class="dim small">{t("dbgui-laeuft-hinweis")}</p>{:else}
        <div class="regs" data-dbg="register">{#each reg as r, i (r[0])}<div class="rg" class:chg={geaendert(i)}><span class="dim">{r[0]}</span><b class="mono" data-dbg={`reg-${r[0].toLowerCase()}`}>${hex(r[1], r[2])}</b></div>{/each}</div>
        {@const rg = s.regs!}
        <div class="flags" role="group" aria-label={t("dbgui-flags")} data-dbg="flags">{#each flags as f, i (i)}<span class="fl" class:on={f !== "-" && !!((rg.flags >> (7 - i)) & 1)} class:chg={!!rg.prev && !!(((rg.prev[5] ?? 0) ^ rg.flags) >> (7 - i) & 1)} aria-label={`${f} ${((rg.flags >> (7 - i)) & 1)}`}>{f}</span>{/each}</div>
        {#if s.regs!.extra.length}<p class="mono small dim">{s.regs!.extra.map((e) => `${e.name} ${e.value.toString(16).toUpperCase()}`).join("  ")}</p>{/if}
        <h3>{t("dbgui-stapel")}</h3>
        <div class="stack mono" data-dbg="stapel">{#each s.stack ?? [] as b, i (i)}<div><span class="dim">${hex(0x100 + s.regs!.sp + 1 + i, 4)}</span> <b>${hex(b, 2)}</b></div>{:else}<span class="dim small">{t("dbgui-stapel-leer")}</span>{/each}</div>
      {/if}
      <h3>{t("dbgui-bildschirm")}</h3>
      {#if !s.frame?.can}<p class="dim small">{t("dbgui-bild-nicht")}</p>{:else}
        <div class="bild" data-dbg="bild">{#if dbg.frame}<img src={dbg.frame.img} alt={t("dbgui-bild-alt")} width={dbg.frame.w} height={dbg.frame.h} />{:else}<span class="dim small">{t("dbgui-bild-leer")}</span>{/if}</div>
        {#if s.frame!.error}<p class="warn small" role="alert">{s.frame!.error}</p>{/if}
        <div class="row"><button type="button" class="btn sm" data-dbg="bild-neu" onclick={() => void bildHolen()}>{t("dbgui-bild-aktualisieren")}</button></div>
        <Switch checked={dbg.auto} label={t("dbgui-bild-auto")} onchange={(on) => void bildModus(on, dbg.slow)} />
        <Switch checked={dbg.slow} label={t("dbgui-bild-live")} onchange={(on) => void bildModus(dbg.auto, on)} />
        {#if s.frame!.sources.length > 1}<select class="inp" data-dbg="bild-quelle" aria-label={t("dbgui-bild-quelle")} onchange={(e) => void bildQuelle(Number(e.currentTarget.value))}>{#each s.frame!.sources as q, i (q)}<option value={i}>{q}</option>{/each}</select>{/if}
        {#if s.frame!.canText}<div class="row"><input class="inp" data-dbg="text" placeholder={t("dbgui-text-eingabe")} aria-label={t("dbgui-text-eingabe")} bind:value={sendText} onkeydown={(e) => { if (e.key === "Enter") { void textSenden(sendText); sendText = ""; } }} /><button type="button" class="btn sm" data-dbg="text-senden" onclick={() => { void textSenden(sendText); sendText = ""; }}>{t("dbgui-text-senden")}</button></div>{/if}
      {/if}
    </section>
  </div>
{/if}

<style>
  .bar .ibtn:has(.kt) { width: auto; padding: 0 9px; gap: 5px; display: inline-flex; align-items: center; } .kt { font-size: 12px; }
  .blank { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 8px; color: var(--dim); text-align: center; padding: 20px; }
  .blank h2 { color: var(--text); font-size: 18px; } .blank p { max-width: 460px; } .row { display: flex; gap: 6px; flex-wrap: wrap; align-items: center; }
  .db { flex: 1; min-height: 0; display: grid; grid-template-columns: minmax(250px, 300px) minmax(0, 1fr) 290px; gap: 10px; padding: 10px; overflow: auto; }
  @media (max-width: 1150px) { .db { grid-template-columns: 260px minmax(0, 1fr); grid-auto-rows: min-content; } .col.r { grid-column: 1 / -1; } .col { overflow: visible; } } @media (max-width: 760px) { .db { grid-template-columns: 1fr; } }
  .col { background: var(--card); border: 1px solid var(--line2); border-radius: 12px; padding: 12px; min-width: 0; display: flex; flex-direction: column; gap: 8px; min-height: 0; overflow: auto; } .col.m { min-height: 480px; }
  h3 { margin: 4px 0 0; font-size: 11px; letter-spacing: 0.07em; text-transform: uppercase; color: var(--dim); display: flex; align-items: center; justify-content: space-between; gap: 6px; }
  .liste { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 3px; } .pt { display: flex; align-items: center; gap: 8px; padding: 4px 6px; border-radius: 8px; background: var(--raise); } .pt .nm { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; }
  .art { font-size: 10px; text-transform: uppercase; color: var(--accent2); min-width: 54px; } .form { display: flex; flex-direction: column; gap: 6px; } .inp { width: 100%; } .inp.num { width: 70px; } .inp.adrf { width: 150px; }
  .bar { display: flex; align-items: center; gap: 6px; flex-wrap: wrap; } .grow { flex: 1; } .sep { width: 1px; height: 20px; background: var(--line2); margin: 0 4px; } .small { font-size: 12px; margin: 0; } .dim { color: var(--dim); } .warn { color: var(--warn); } .pad { padding: 8px; }
  .go { color: var(--ok, #4ade80); } .stop { color: var(--err, #f87171); } .chip, .pill { font-size: 12px; padding: 3px 10px; border-radius: 12px; border: 1px solid var(--line2); } .chip.run, .pill.run { border-color: var(--ok, #4ade80); color: var(--ok, #4ade80); } .pill { text-transform: none; letter-spacing: 0; background: var(--warn); color: #000; border: 0; font-weight: 700; } .pill.run { background: var(--ok, #4ade80); color: #000; }
  .dis { background: var(--code_bg, var(--raise)); border-radius: 10px; padding: 6px; overflow: auto; flex: 1; min-height: 240px; max-height: 52vh; }
  .dz { display: grid; grid-template-columns: 22px 46px 92px minmax(0, 1fr) 40px auto; align-items: center; gap: 6px; padding: 1px 4px; font-size: 13px; border-radius: 4px; } .dz.cur { background: var(--accentbg); outline: 1px solid var(--accent); } .dz.ill .tx { color: var(--warn); } .ad { color: var(--dim); } .tx { font-weight: 600; } .cy { font-size: 11px; text-align: right; }
  .lab { color: var(--accent2); font-weight: 700; margin: 6px 0 0 28px; font-size: 12px; } .bp { width: 12px; height: 12px; border-radius: 50%; border: 1px solid var(--line2); background: transparent; padding: 0; margin: 0 auto; } .bp.an { background: #f87171; border-color: #f87171; } .bp:hover { border-color: #f87171; }
  .lnk { font-size: 11px; color: var(--accent2); text-decoration: underline; }
  .mem { background: var(--code_bg, var(--raise)); border-radius: 10px; padding: 6px 10px; font-size: 13px; } .mz { display: grid; grid-template-columns: 44px minmax(0, 1fr) 16ch; gap: 10px; } .hx { display: flex; gap: 6px; flex-wrap: wrap; } .hx i { font-style: normal; } .hx i.chg { color: var(--warn); font-weight: 700; } .as { letter-spacing: 0.04em; }
  .regs { display: grid; grid-template-columns: repeat(auto-fill, minmax(72px, 1fr)); gap: 6px; } .rg { display: flex; flex-direction: column; background: var(--raise); border-radius: 8px; padding: 4px 8px; } .rg.chg { outline: 1px solid var(--accent); background: var(--accentbg); } .rg span { font-size: 10px; }
  .flags { display: flex; gap: 4px; } .fl { flex: 1; text-align: center; font-size: 12px; font-weight: 700; padding: 3px 0; border-radius: 6px; background: var(--raise); color: var(--dim); } .fl.on { background: var(--accentbg); color: var(--accent2); } .fl.chg { outline: 1px solid var(--accent); }
  .stack { font-size: 12px; display: flex; flex-direction: column; gap: 1px; } .bild { background: #000; border-radius: 8px; padding: 6px; display: flex; justify-content: center; min-height: 80px; align-items: center; } .bild img { max-width: 100%; height: auto; image-rendering: pixelated; }
  .bp:focus-visible, .lnk:focus-visible { outline: 2px solid var(--accent2); outline-offset: 2px; }
  .chipsw { display: flex; flex-wrap: wrap; gap: 4px 10px; } .chipsw label { display: inline-flex; align-items: center; gap: 4px; } .chip-fenster { max-height: 240px; overflow: auto; border: 1px solid var(--line2); border-radius: 8px; padding: 6px 8px; } .cr { display: flex; gap: 8px; font-size: 12px; line-height: 1.5; } .cr .grow { flex: 1; }
</style>
