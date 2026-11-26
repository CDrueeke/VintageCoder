<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { openExport } from "../lib/export.svelte";
  import { alleEinblenden, answerAsk, imp, importAusfuehren, importOeffnen, kl, klingt, KLAVIER, naechsteStimme, sicht, stimmeUmschalten, vorspielen, openSong, pickSong, saveSong, songDateien, songRedo, songUndo, spielen, stoppen, tk, trackerOp as op, umschaltenSpielen, wg, wiedergabeAbfrage } from "../lib/tracker.svelte";
  import Dialog from "../lib/components/Dialog.svelte";
  import Icon from "../lib/components/Icon.svelte";
  import Switch from "../lib/components/Switch.svelte";
  import Segment from "../lib/components/Segment.svelte";
  import Tooltip from "../lib/components/Tooltip.svelte";
  import { singleKeyOk, ui } from "../lib/state.svelte";
  import { t } from "../lib/i18n.svelte";
  // Tracker (E5.1, nach Mockup neu-5): links Reihenfolge der Muster und Instrumente, in der Mitte das Raster des gewählten Musters (Zellen „Note Instrument“), rechts der Instrumenteneditor.
  // Das Raster ist eine Auswahlliste: Pfeiltasten bewegen den Cursor, Umschalt erweitert die Auswahl, Klaviertasten tragen Noten ein (nach der Position der Taste). Die Wiedergabe folgt mit E5.2.
  const v = $derived(tk.view);
  const dateien = $derived(songDateien(ui.project?.tree));
  const hex2 = (i: number) => i.toString(16).toUpperCase().padStart(2, "0");
  let askOpen = $state(false);
  $effect(() => { askOpen = tk.ask !== null; });
  $effect(() => { if (!askOpen && tk.ask) void answerAsk("cancel"); });
  const farben = ["#8c7dff", "#7dd3fc", "#fbbf24", "#f87171", "#4ade80", "#f0abfc", "#fdba74", "#94a3b8"];

  // Wiedergabe (E5.2): Abfrage läuft nur, solange der Song spielt; die spielende Zeile wird markiert und bei „Mitscrollen“ sichtbar gehalten
  $effect(() => { wiedergabeAbfrage(!!v?.spielt); });
  // Vorhörton beim Eintragen: Taste der Klaviatur leuchtet kurz
  $effect(() => { const a = v?.audition; if (a && v) klingt(a.note - 1 - v.octave * 12); });
  const stand = $derived(v?.spielt ? wg.stand : null);
  const spielZeile = $derived(v && stand?.playing && (v.kind === "play" || v.spielModus === "pattern" || v.order[stand.ord] === v.pattern) ? stand.row : -1);
  $effect(() => { if (spielZeile >= 0 && wg.folgen) document.querySelector(`[data-tk=raster] [data-tk=zelle][data-r="${spielZeile}"][data-c="0"]`)?.scrollIntoView({ block: "nearest" }); });
  const pegel = (c: number) => Math.min(100, Math.round((stand?.peaks[c] ?? 0) * 140));
  let zfokus = $state(0); let zz: HTMLElement[] = []; let zug: [number, number] | null = null;
  const sichtbar = $derived(Array.from({ length: v?.channels ?? 0 }, (_, c) => c).filter((c) => !sicht.versteckt.includes(c)));
  const spalten = $derived(`40px repeat(${sichtbar.length}, minmax(96px, 1fr))`);
  const idx = (r: number, c: number) => r * (v?.channels ?? 1) + c;
  $effect(() => { if (v) zfokus = idx(v.cursor[0], v.cursor[1]); });
  const inSel = (r: number, c: number) => !!v?.sel && r >= v.sel[0] && r <= v.sel[2] && c >= v.sel[1] && c <= v.sel[3];
  function fokus(r: number, c: number) { zfokus = idx(r, c); zz[zfokus]?.focus(); }
  const zelleAn = (e: PointerEvent): [number, number] | null => { const el = document.elementFromPoint(e.clientX, e.clientY) as HTMLElement | null; const k = el?.closest("[data-r]"); if (!k) return null; return [Number(k.getAttribute("data-r")), Number(k.getAttribute("data-c"))]; };
  function down(e: PointerEvent, r: number, c: number) { if (e.button !== 0) return; zug = [r, c]; fokus(r, c); void op(e.shiftKey && v ? { kind: "select", r0: v.cursor[0], c0: v.cursor[1], r1: r, c1: c } : { kind: "cursor", r, c }); }
  function move(e: PointerEvent) { if (!zug) return; const p = zelleAn(e); if (p && (p[0] !== zug[0] || p[1] !== zug[1]) && e.buttons) { void op({ kind: "select", r0: zug[0], c0: zug[1], r1: p[0], c1: p[1] }); } }
  function up() { zug = null; }
  function taste(e: KeyboardEvent) {
    if (!v) return; const mod = e.metaKey || e.ctrlKey;
    const pfeil: [number, number] | null = e.key === "ArrowUp" ? [-1, 0] : e.key === "ArrowDown" ? [1, 0] : e.key === "ArrowLeft" ? [0, -1] : e.key === "ArrowRight" ? [0, 1] : null;
    const stop = () => { e.preventDefault(); e.stopPropagation(); };
    if (pfeil) { stop(); if (e.altKey && pfeil[1] === 0) { void op({ kind: "cell_instrument", dx: -pfeil[0] }); return; } void op({ kind: "move", dr: pfeil[0], dc: pfeil[1] === 0 ? 0 : naechsteStimme(v.cursor[1], pfeil[1], v.channels), extend: e.shiftKey }).then(() => fokus(tk.view!.cursor[0], tk.view!.cursor[1])); return; }
    if (e.key === "PageDown" || e.key === "PageUp") { stop(); void op({ kind: "move", dr: e.key === "PageDown" ? 8 : -8, dc: 0, extend: e.shiftKey }).then(() => fokus(tk.view!.cursor[0], tk.view!.cursor[1])); return; }
    if (e.key === "Home" || e.key === "End") { stop(); void op({ kind: "jump", r: e.key === "Home" ? 0 : v.rows - 1 }).then(() => fokus(tk.view!.cursor[0], tk.view!.cursor[1])); return; }
    if (e.key === "Delete" || e.key === "Backspace") { stop(); void op({ kind: "clear" }); return; }
    if (mod) { const k = e.key.toLowerCase(); if (k === "c") { stop(); void op({ kind: "copy" }); } else if (k === "x") { stop(); void op({ kind: "cut" }); } else if (k === "v") { stop(); void op({ kind: "paste" }); } else if (k === "a") { stop(); void op({ kind: "select_all" }); } return; }
    if (!singleKeyOk(e)) return;
    if (e.key === "[" ) { stop(); void op({ kind: "instrument_cur", d: -1 }); return; } if (e.key === "]") { stop(); void op({ kind: "instrument_cur", d: 1 }); return; }
    if (e.key === ",") { stop(); void op({ kind: "octave", d: -1 }); return; } if (e.key === ".") { stop(); void op({ kind: "octave", d: 1 }); return; }
    if (e.key === "1") { stop(); void op({ kind: "note_end" }); return; }
    if (e.code === "Space") { stop(); void umschaltenSpielen(e.shiftKey ? "pattern" : "song"); return; }
    const h = KLAVIER[e.code]; if (h !== undefined && !e.altKey) { stop(); void op({ kind: "note", halbton: h }).then(() => fokus(tk.view!.cursor[0], tk.view!.cursor[1])); }
  }
  const noteTxt = (z: string) => z.slice(0, 3); const insTxt = (z: string) => z.slice(4);
  const wellen = $derived([{ id: "triangle", label: t("tracker-welle-dreieck") }, { id: "saw", label: t("tracker-welle-saege") }, { id: "pulse", label: t("tracker-welle-rechteck") }, { id: "noise", label: t("tracker-welle-rauschen") }]);
  const dists = $derived([{ id: "pure", label: t("tracker-dist-rein") }, { id: "bass", label: t("tracker-dist-bass") }, { id: "buzz", label: t("tracker-dist-buzz") }, { id: "gritty", label: t("tracker-dist-gritty") }]);
  const ins = $derived(v?.instruments[v.insSel]);
  const adsr = $derived(ins ? [{ k: "a", label: t("tracker-anschlag"), w: ins.a ?? 0 }, { k: "d", label: t("tracker-abfall"), w: ins.d ?? 0 }, { k: "s", label: t("tracker-halten"), w: ins.s ?? 0 }, { k: "r", label: t("tracker-ausklang"), w: ins.r ?? 0 }] : []);
  // Klaviatur: zwei Oktaven plus Schlusston ab der Oktave der Eingabe; Beschriftung = Computertaste (nach Tastenposition)
  const NOTEN = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
  const taste2 = (h: number) => { const code = Object.entries(KLAVIER).find(([, x]) => x === h)?.[0] ?? ""; return code.replace(/^Key|^Digit/, ""); };
  const tasten = $derived.by(() => { let w = 0; return Array.from({ length: 25 }, (_, h) => { const schwarz = [1, 3, 6, 8, 10].includes(h % 12); if (!schwarz) w++; return { h, schwarz, links: (w / 15) * 100 - 2.6, label: ui.keyHints ? taste2(h) : "", name: `${NOTEN[h % 12]}${(v?.octave ?? 0) + Math.floor(h / 12)}` }; }); });
  let ersterZug = true; const regler = (key: string, value: number, first: boolean) => void op({ kind: "ins_value", key, value, first });
  let nameText = $state(""); $effect(() => { if (ins) nameText = ins.name; });
</script>

{#if !v}
  <div class="blank">
    <Icon name="music" size={28} />
    <h2>{t("tracker-leer-titel")}</h2>
    {#if !ui.project}<p>{t("tracker-leer-kein-projekt")}</p>
    {:else}
      <p>{t("tracker-leer-text")}</p>
      {#if dateien.length}<ul class="files" aria-label={t("tracker-dateien")}>{#each dateien as f (f)}<li><button type="button" class="btn" data-tk="open" onclick={() => void openSong(f)}>{f}</button></li>{/each}</ul>
      {:else}<p class="dim small">{t("tracker-keine-dateien")}</p>{/if}
      <div class="row"><button type="button" class="btn pri" onclick={() => void pickSong()}>{t("sprite-oeffnen")}</button></div>
    {/if}
  </div>
{:else}
  <div class="tr" aria-label={t("view-title-musik")}>
    <section class="col l" aria-label={t("tracker-liste-titel")}>
      <h3>{v.name || v.title.replace(/\.(vcsong|vcplay)$/i, "")}<Tooltip text={t("tracker-muster-neu")}><button type="button" class="ibtn plus" data-act="pattern-new" aria-label={t("tracker-muster-neu")} onclick={() => void op({ kind: "pattern_new" })}><Icon name="plus" size={16} /></button></Tooltip></h3>
      {#if v.kind === "song"}
        <h3>{t("tracker-reihenfolge")}</h3>
        <div class="liste" role="listbox" aria-label={t("tracker-reihenfolge-label", { n: v.order.length })} data-tk="reihenfolge" tabindex="-1">
          {#each v.order as p, i (i)}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <div role="option" class="oz" class:on={i === v.ordSel} tabindex={i === v.ordSel ? 0 : -1} aria-selected={i === v.ordSel} aria-setsize={v.order.length} aria-posinset={i + 1} aria-label={t("tracker-eintrag-ansage", { i: hex2(i), p: hex2(p) })} data-tk="eintrag" data-i={i}
              onclick={() => void op({ kind: "ord_select", i })} onkeydown={(e) => { if (e.key === "ArrowDown") { e.preventDefault(); void op({ kind: "ord_select", i: Math.min(v.order.length - 1, i + 1) }); } else if (e.key === "ArrowUp") { e.preventDefault(); void op({ kind: "ord_select", i: Math.max(0, i - 1) }); } }}>
              <span class="mono dim">{hex2(i)}</span><span>{t("tracker-muster-nr", { n: p + 1 })}</span>{#if i === v.loop}<span class="schl" title={t("tracker-schleife")}>↻</span>{/if}</div>
          {/each}
        </div>
        <div class="row">
          <button type="button" class="btn sm" data-act="ord-append" onclick={() => void op({ kind: "ord_append" })}>{t("tracker-anhaengen")}</button>
          <button type="button" class="btn sm" data-act="ord-set" onclick={() => void op({ kind: "ord_set" })}>{t("tracker-setzen")}</button>
          <button type="button" class="btn sm" data-act="ord-remove" disabled={v.order.length < 2} onclick={() => void op({ kind: "ord_remove" })}>{t("tracker-entfernen")}</button>
          <button type="button" class="ibtn" data-act="ord-up" aria-label={t("tracker-nach-oben")} onclick={() => void op({ kind: "ord_move", d: -1 })}><Icon name="arru" size={15} /></button>
          <button type="button" class="ibtn" data-act="ord-down" aria-label={t("tracker-nach-unten")} onclick={() => void op({ kind: "ord_move", d: 1 })}><Icon name="arrd" size={15} /></button>
        </div>
        <label class="small dim">{t("tracker-schleife-ab")} <input class="inp num" type="number" min="0" max={v.order.length - 1} data-tk="loop" value={v.loop} onchange={(e) => void op({ kind: "loop", l: Number(e.currentTarget.value) })} /></label>
        <div class="row"><button type="button" class="btn sm" data-act="pattern-dup" onclick={() => void op({ kind: "pattern_dup" })}>{t("tracker-muster-dup")}</button><button type="button" class="btn sm" data-act="pattern-delete" disabled={v.patternCount < 2} onclick={() => void op({ kind: "pattern_delete" })}>{t("tracker-muster-loeschen")}</button></div>
      {:else}<p class="small dim">{t("tracker-play-hinweis")}</p>{/if}
      <h3>{t("tracker-instrumente")}</h3>
      <div class="liste" role="listbox" aria-label={t("tracker-instrumente-label", { n: v.instruments.length })} data-tk="instrumente" tabindex="-1">
        {#each v.instruments as x, i (i)}
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <div role="option" class="oz" class:on={i === v.insSel} tabindex={i === v.insSel ? 0 : -1} aria-selected={i === v.insSel} aria-setsize={v.instruments.length} aria-posinset={i + 1} aria-label={t("tracker-instrument-ansage", { i: hex2(i), name: x.name })} data-tk="instrument" data-i={i}
            onclick={() => void op({ kind: "ins_select", i })} onkeydown={(e) => { if (e.key === "ArrowDown") { e.preventDefault(); void op({ kind: "ins_select", i: Math.min(v.instruments.length - 1, i + 1) }); } else if (e.key === "ArrowUp") { e.preventDefault(); void op({ kind: "ins_select", i: Math.max(0, i - 1) }); } }}>
            <span class="mono dim">{hex2(i)}</span><i class="pkt" style={`background:${farben[i % farben.length]}`}></i><span class="nm">{x.name}</span><span class="dim small">{x.wave ?? x.dist ?? ""}</span></div>
        {/each}
      </div>
      <div class="row"><button type="button" class="btn sm" data-act="ins-new" disabled={v.instruments.length >= v.maxInstruments} onclick={() => void op({ kind: "ins_new" })}>{t("tracker-instrument-neu")}</button>
        <button type="button" class="btn sm" data-act="ins-dup" disabled={v.instruments.length >= v.maxInstruments} onclick={() => void op({ kind: "ins_dup" })}>{t("tracker-muster-dup")}</button>
        <button type="button" class="btn sm" data-act="ins-delete" disabled={v.instruments.length < 2 || v.insSel + 1 !== v.instruments.length || !!ins?.used} title={t("tracker-instrument-loeschen-hinweis")} onclick={() => void op({ kind: "ins_delete" })}>{t("tracker-muster-loeschen")}</button></div>
      {#if dateien.length > 1}<h3>{t("tracker-dateien")}</h3>
        <ul class="files" data-tk="files">{#each dateien as f (f)}<li><button type="button" class="fbtn" class:on={f === v.rel} aria-current={f === v.rel ? "true" : undefined} onclick={() => void openSong(f)} title={f}><Icon name="music" size={14} /><span class="nm">{f.split("/").pop()}</span>{#if f === v.rel && v.dirty}<span class="udot" title={t("sprite-ungespeichert")}>●</span>{/if}</button></li>{/each}</ul>{/if}
    </section>

    <section class="col m" aria-label={t("tracker-raster-label")}>
      <div class="bar" role="toolbar" aria-label={t("tracker-werkzeuge")}>
        <Tooltip text={t("cmd-rueckgaengig")}><button type="button" class="ibtn" disabled={!v.canUndo} aria-label={t("cmd-rueckgaengig")} data-act="undo" onclick={() => void songUndo()}><Icon name="undo" size={17} /></button></Tooltip>
        <Tooltip text={t("cmd-wiederholen")}><button type="button" class="ibtn" disabled={!v.canRedo} aria-label={t("cmd-wiederholen")} data-act="redo" onclick={() => void songRedo()}><Icon name="redo" size={17} /></button></Tooltip>
        <span class="sep"></span>
        <Tooltip text={t("tracker-abspielen-song")}><button type="button" class="ibtn" class:aktiv={v.spielt && v.spielModus === "song"} disabled={v.spielt && v.spielModus === "song"} aria-label={t("tracker-abspielen-song")} data-act="play-song" onclick={() => void spielen("song")}><Icon name="play" size={17} /></button></Tooltip>
        <Tooltip text={t("tracker-abspielen-muster")}><button type="button" class="btn sm" class:aktiv={v.spielt && v.spielModus === "pattern"} aria-label={t("tracker-abspielen-muster")} data-act="play-pattern" onclick={() => void spielen("pattern")}>{t("tracker-muster-kurz")}</button></Tooltip>
        <Tooltip text={t("tracker-stopp")}><button type="button" class="ibtn" disabled={!v.spielt} aria-label={t("tracker-stopp")} data-act="stop" onclick={() => void stoppen()}><Icon name="stop" size={17} /></button></Tooltip>
        <Switch bind:checked={wg.folgen} label={t("tracker-mitscrollen")} />
        <span class="sep"></span>
        <label class="zf"><span class="lbl">{t("tracker-tempo")}</span><input class="inp num" type="number" min="1" max="255" data-tk="tempo" value={v.tempo} onchange={(e) => void op({ kind: "tempo", t: Number(e.currentTarget.value) })} /></label>
        <label class="zf"><span class="lbl">{t("tracker-zeilen")}</span><input class="inp num" type="number" min="1" max="128" data-tk="rows" value={v.rows} onchange={(e) => void op({ kind: "rows", n: Number(e.currentTarget.value) })} /></label>
        <label class="zf"><span class="lbl">{t("tracker-oktave")}</span><input class="inp num" type="number" min="0" max="7" data-tk="oktave" value={v.octave} onchange={(e) => void op({ kind: "octave", d: Number(e.currentTarget.value) - v.octave })} /></label>
        <label class="zf"><span class="lbl">{t("tracker-schritt")}</span><input class="inp num" type="number" min="0" max="16" data-tk="schritt" value={v.step} onchange={(e) => void op({ kind: "step", n: Number(e.currentTarget.value) })} /></label>
        <span class="sep"></span>
        <button type="button" class="btn sm" data-act="transpose-up" onclick={() => void op({ kind: "transpose", n: 1 })}>{t("tracker-halbton-hoch")}</button>
        <button type="button" class="btn sm" data-act="transpose-down" onclick={() => void op({ kind: "transpose", n: -1 })}>{t("tracker-halbton-runter")}</button>
        <button type="button" class="btn sm" data-act="apply-ins" onclick={() => void op({ kind: "apply_instrument" })}>{t("tracker-instrument-anwenden")}</button>
        <span class="grow"></span>
        {#if v.kind === "song"}<button type="button" class="btn sm" data-act="konvertieren" onclick={() => void importOeffnen(false)}>{t("tracker-konvertieren")} …</button>{#if v.pokey}<button type="button" class="btn sm" data-act="rmt-import" onclick={() => void importOeffnen(true)}>{t("tracker-rmt-import")} …</button>{/if}{/if}
        <button type="button" class="btn sm" data-act="export" onclick={() => tk.docId !== null && void openExport(tk.docId)}>{t("export-titel")} …</button>
        <button type="button" class="btn sm" disabled={!v.dirty} data-act="save" onclick={() => void saveSong()}>{t("cmd-speichern")}</button>
      </div>
      {#if v.channels > 1}<div class="muster stimmen" role="group" aria-label={t("tracker-stimmen-sichtbar")} data-tk="stimmen"><span class="lbl">{t("tracker-stimmen")}</span>{#each Array.from({ length: v.channels }, (_, c) => c) as c (c)}<button type="button" class="btn sm" class:on={!sicht.versteckt.includes(c)} aria-pressed={!sicht.versteckt.includes(c)} data-tk="stimme" data-c={c} onclick={() => stimmeUmschalten(c)}>{c + 1}</button>{/each}<button type="button" class="btn sm" data-tk="alle-stimmen" disabled={!sicht.versteckt.length} onclick={alleEinblenden}>{t("tracker-alle-einblenden")}</button></div>{/if}
      {#if v.kind === "song"}<div class="muster" role="group" aria-label={t("tracker-muster-wahl")}>{#each Array.from({ length: v.patternCount }, (_, i) => i) as p (p)}<button type="button" class="btn sm" class:on={p === v.pattern} aria-pressed={p === v.pattern} data-tk="muster" data-p={p} onclick={() => void op({ kind: "pattern", p })}>{t("tracker-muster-nr", { n: p + 1 })}</button>{/each}</div>{/if}
      <div class="rasterbox" data-tk="raster">
        <div class="kopf" style={`grid-template-columns:${spalten}`}><span></span>{#each sichtbar as c (c)}<span class="kh"><i class="pkt" style={`background:${farben[c % farben.length]}`}></i>{t("tracker-stimme", { n: c + 1 })}
          <button type="button" class="ms" class:an={v.mute[c]} aria-pressed={!!v.mute[c]} aria-label={t("tracker-stumm", { n: c + 1 })} data-tk="mute" data-c={c} onclick={() => void op({ kind: "mute", c })}>M</button>
          <button type="button" class="ms" class:an={v.solo[c]} aria-pressed={!!v.solo[c]} aria-label={t("tracker-solo", { n: c + 1 })} data-tk="solo" data-c={c} onclick={() => void op({ kind: "solo", c })}>S</button>
          <i class="vu" aria-hidden="true" data-tk="pegel" data-c={c}><b style={`width:${pegel(c)}%`}></b></i></span>{/each}</div>
        <div class="rws" role="listbox" aria-label={t("tracker-raster-liste", { r: v.rows, c: v.channels })} tabindex="-1" onkeydown={taste} onpointermove={move} onpointerup={up}>
          {#each v.grid as zeile, r (r)}
            <div class="rwx" class:beat={r % 4 === 0} class:pl={r === spielZeile} style={`grid-template-columns:${spalten}`}><span class="rn mono">{hex2(r)}</span>
              {#each zeile as z, c (c)}{#if !sicht.versteckt.includes(c)}
                <div role="option" class="zl mono" class:cur={v.cursor[0] === r && v.cursor[1] === c} class:sel={inSel(r, c)} class:leer={z === "--- --"} bind:this={zz[idx(r, c)]} tabindex={idx(r, c) === zfokus ? 0 : -1} aria-selected={v.cursor[0] === r && v.cursor[1] === c} aria-setsize={v.rows * v.channels} aria-posinset={idx(r, c) + 1}
                  aria-label={t("tracker-zelle-ansage", { r: hex2(r), c: c + 1, note: noteTxt(z), ins: insTxt(z) })} data-tk="zelle" data-r={r} data-c={c} onfocus={() => (zfokus = idx(r, c))} onpointerdown={(e) => down(e, r, c)}>
                  <span class="nt">{z === "--- --" ? "···" : noteTxt(z)}</span><span class="ii">{z === "--- --" ? "··" : insTxt(z)}</span></div>{/if}
              {/each}</div>
          {/each}
        </div>
      </div>
      <p class="foot small dim" data-tk="fuss">{#if stand?.playing}<span class="ok" data-tk="spiel-status">▶ {t("tracker-spielt", { i: hex2(stand.ord), r: hex2(stand.row) })} · </span>{/if}{#if wg.stand?.error}<span class="warn" role="alert">{wg.stand.error} · </span>{/if}{#if v.dirty}<span class="warn">● {t("sprite-ungespeichert")} · </span>{/if}{v.title} · {t("tracker-muster-nr", { n: v.pattern + 1 })} · {t("tracker-reihe", { r: hex2(v.cursor[0]) })} · {t("tracker-kanal", { c: v.cursor[1] + 1 })} · <span class="mono">{t("tracker-tasten-hinweis")}</span></p>
    </section>

    <section class="col r" aria-label={t("tracker-instrument-titel")}>
      {#if ins}
        <h3>{t("tracker-instrument-titel-n", { n: hex2(v.insSel), name: ins.name })}</h3>
        <input class="inp" data-tk="ins-name" aria-label={t("tracker-instrument-name")} bind:value={nameText} onchange={(e) => regler("name", e.currentTarget.value as unknown as number, true)} />
        {#if v.pokey}
          <h3>{t("tracker-verzerrung")}</h3><Segment small label={t("tracker-verzerrung")} items={dists} value={ins.dist ?? "pure"} onselect={(id) => void op({ kind: "ins_value", key: "dist", value: id })} />
          <h3>{t("tracker-huellkurve")}</h3>
          <div class="env" role="group" aria-label={t("tracker-huellkurve-label")} data-tk="env">
            {#each (ins.env ?? []) as w, k (k)}<div class="eb"><input type="range" min="0" max="15" value={w} data-tk="env-bild" data-k={k} aria-label={t("tracker-env-bild", { k: k + 1, w })} oninput={(e) => void op({ kind: "envelope", k, v: Number(e.currentTarget.value), first: false })} onchange={() => void op({ kind: "ins_select", i: v.insSel })} /><span class="mono dim">{w}</span></div>{/each}
          </div>
          <label class="small dim">{t("tracker-env-schleife")} <input class="inp num" type="number" min="0" max="15" data-tk="env-loop" value={ins.loop ?? 15} onchange={(e) => void op({ kind: "ins_value", key: "loop", value: Number(e.currentTarget.value) })} /></label>
        {:else}
          <h3>{t("tracker-klangform")}</h3><Segment small label={t("tracker-klangform")} items={wellen} value={ins.wave ?? "pulse"} onselect={(id) => void op({ kind: "ins_value", key: "wave", value: id })} />
          {#each adsr as x (x.k)}
            <label class="rg"><span class="lbl">{x.label}</span><input type="range" min="0" max="15" value={x.w} data-tk={`adsr-${x.k}`} aria-label={x.label} oninput={(e) => { regler(x.k, Number(e.currentTarget.value), ersterZug); ersterZug = false; }} onchange={() => (ersterZug = true)} /><span class="wert mono">{x.w}</span></label>
          {/each}
          {#if (ins.wave ?? "pulse") === "pulse"}<label class="rg"><span class="lbl">{t("tracker-pulsbreite")}</span><input type="range" min="0" max="4095" value={ins.pw ?? 2048} data-tk="pw" aria-label={t("tracker-pulsbreite")} oninput={(e) => { regler("pw", Number(e.currentTarget.value), ersterZug); ersterZug = false; }} onchange={() => (ersterZug = true)} /><span class="wert mono">{Math.round(((ins.pw ?? 2048) / 4096) * 100)} %</span></label>{/if}
        {/if}
        <h3>{t("tracker-klaviatur")}</h3>
        <div class="klav" role="group" aria-label={t("tracker-klaviatur-label", { o: v.octave })} data-tk="klaviatur">
          {#each tasten as k (k.h)}<button type="button" class="kt" class:schwarz={k.schwarz} class:klingt={kl.halbton === k.h || (stand?.notes ?? []).some((n) => n != null && n - 1 - v.octave * 12 === k.h)} style={k.schwarz ? `left:${k.links}%` : ""} data-tk="taste" data-h={k.h} aria-label={t("tracker-taste-ansage", { note: k.name })} onpointerdown={(e) => { e.preventDefault(); void vorspielen(k.h); }}><span class="tl">{k.label}</span></button>{/each}
        </div>
        <p class="small dim">{t("tracker-klaviatur-hinweis")}</p>
        <h3>{t("tracker-lautstaerke")}</h3>
        <label class="rg"><span class="lbl">{t("tracker-lautstaerke")}</span><input type="range" min="0" max="15" value={v.volume} data-tk="volume" aria-label={t("tracker-lautstaerke")} onchange={(e) => void op({ kind: "volume", v: Number(e.currentTarget.value) })} /><span class="wert mono">{v.volume}</span></label>
      {/if}
    </section>
  </div>
{/if}

{#if imp.open}
  <Dialog bind:open={imp.open} title={imp.rmt ? t("tracker-rmt-import") : t("tracker-konvertieren-titel")}>
    {#snippet children()}
      <div class="imp" data-tk="import">
        <p class="small"><b>{imp.q?.file}</b> · {imp.q?.art ?? ""}</p>
        {#if imp.q?.sid}<p class="small dim" data-tk="sid-info">{imp.q.sid.info}</p>{/if}
        {#if imp.q?.error}<p class="warn small" role="alert">{imp.q.error}</p>{:else}
          <label class="small dim">{t("tracker-import-name")}<input class="inp" data-tk="imp-name" bind:value={imp.name} /></label>
          {#if !imp.rmt}<label class="small dim">{t("tracker-import-ziel")}<select class="inp" data-tk="imp-ziel" bind:value={imp.ziel}>{#each imp.ziele as p (p.id)}<option value={p.id}>{p.name}</option>{/each}</select></label>{/if}
          {#if imp.q?.sid}
            <label class="small dim">{t("tracker-import-subtune", { n: imp.q.sid.songs })}<input class="inp num" type="number" min="1" max={imp.q.sid.songs} data-tk="imp-sub" bind:value={imp.subtune} /></label>
            <label class="small dim">{t("tracker-import-sekunden")}<input class="inp num" type="number" min="1" max="600" data-tk="imp-sek" bind:value={imp.sekunden} /></label>
          {/if}
          {#if imp.fehler}<p class="warn small" role="alert" data-tk="imp-fehler">{imp.fehler}</p>{/if}
        {/if}
      </div>
    {/snippet}
    {#snippet footer()}
      <button type="button" class="btn" onclick={() => (imp.open = false)}>{t("dialog-abbrechen")}</button>
      <button type="button" class="btn pri" data-tk="imp-ok" disabled={imp.busy || !!imp.q?.error || !imp.name.trim()} onclick={() => void importAusfuehren()}>{imp.busy ? t("tracker-import-laeuft") : imp.rmt ? t("tracker-rmt-import") : t("tracker-konvertieren")}</button>
    {/snippet}
  </Dialog>
{/if}

{#if askOpen}
  <Dialog bind:open={askOpen} title={t("tracker-frage-titel")}>
    {#snippet children()}<p>{t("sprite-frage-text", { name: v?.title ?? "" })}</p>{/snippet}
    {#snippet footer()}
      <button type="button" class="btn" onclick={() => void answerAsk("cancel")}>{t("dialog-abbrechen")}</button>
      <button type="button" class="btn" data-ask="discard" onclick={() => void answerAsk("discard")}>{t("close-verwerfen")}</button>
      <button type="button" class="btn pri" data-ask="save" onclick={() => void answerAsk("save")}>{t("close-speichern")}</button>
    {/snippet}
  </Dialog>
{/if}

<style>
  .blank { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 8px; color: var(--dim); text-align: center; padding: 20px; }
  .blank h2 { color: var(--text); font-size: 18px; } .blank p { max-width: 420px; } .row { display: flex; gap: 6px; flex-wrap: wrap; align-items: center; } .files { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 4px; }
  .tr { flex: 1; min-height: 0; display: grid; grid-template-columns: minmax(230px, 280px) minmax(0, 1fr) 290px; gap: 10px; padding: 10px; overflow: auto; }
  @media (max-width: 1150px) { .tr { grid-template-columns: 250px minmax(0, 1fr); grid-auto-rows: min-content; } .col.r { grid-column: 1 / -1; } .col { overflow: visible; } } @media (max-width: 760px) { .tr { grid-template-columns: 1fr; } }
  .col { background: var(--card); border: 1px solid var(--line2); border-radius: 12px; padding: 12px; min-width: 0; display: flex; flex-direction: column; gap: 8px; min-height: 0; overflow: auto; } .col.m { min-height: 460px; }
  h3 { margin: 4px 0 0; font-size: 11px; letter-spacing: 0.07em; text-transform: uppercase; color: var(--dim); display: flex; align-items: center; justify-content: space-between; gap: 6px; } .plus { width: 24px; height: 24px; }
  .liste { display: flex; flex-direction: column; gap: 2px; } .oz { display: flex; align-items: center; gap: 8px; padding: 6px 8px; border-radius: 8px; cursor: pointer; } .oz:hover { background: var(--hover); } .oz.on { background: var(--accentbg); color: var(--accent2); } .oz:focus-visible { outline: 2px solid var(--accent2); }
  .pkt { width: 9px; height: 9px; border-radius: 50%; flex: none; display: inline-block; } .nm { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; } .schl { margin-left: auto; color: var(--accent2); }
  .fbtn { display: flex; align-items: center; gap: 7px; width: 100%; padding: 6px 8px; border-radius: 8px; color: var(--text); text-align: left; } .fbtn:hover { background: var(--hover); } .fbtn.on { background: var(--accentbg); color: var(--accent2); } .udot, .warn { color: var(--warn); }
  .bar { display: flex; align-items: center; gap: 6px; flex-wrap: wrap; } .grow { flex: 1; } .sep { width: 1px; height: 20px; background: var(--line2); margin: 0 4px; }
  .zf { display: flex; flex-direction: column; font-size: 10px; } .lbl { color: var(--dim); font-size: 10px; letter-spacing: 0.06em; text-transform: uppercase; } .inp.num { width: 62px; }
  .muster { display: flex; gap: 4px; flex-wrap: wrap; } .muster .on { border-color: var(--accent); background: var(--accentbg); }
  .rasterbox { background: var(--code_bg, var(--raise)); border-radius: 10px; padding: 6px 8px; overflow: auto; flex: 1; min-height: 320px; max-height: 62vh; }
  .kopf { display: grid; gap: 0; position: sticky; top: 0; background: var(--card); z-index: 1; padding-bottom: 4px; } .kh { display: flex; align-items: center; gap: 6px; font-size: 12px; font-weight: 600; padding: 2px 8px; }
  .rwx { display: grid; align-items: center; } .rwx.beat { background: rgba(255, 255, 255, 0.04); } .rn { color: var(--dim); font-size: 12px; text-align: right; padding-right: 8px; }
  .zl { display: flex; gap: 8px; padding: 2px 8px; font-size: 13px; cursor: cell; border-radius: 4px; } .zl.leer { color: var(--faint); } .nt { color: var(--text); font-weight: 600; width: 3ch; } .ii { color: var(--warn); } .zl.leer .nt, .zl.leer .ii { color: var(--faint); font-weight: 400; }
  .rwx.pl { background: rgba(125, 211, 252, 0.16); box-shadow: inset 3px 0 0 var(--accent2); } .ok { color: var(--accent2); }
  .ms { font-size: 10px; font-weight: 700; padding: 0 5px; border-radius: 4px; border: 1px solid var(--line2); color: var(--dim); margin-left: 2px; } .ms.an { background: var(--accent); color: #fff; border-color: var(--accent); }
  .vu { display: block; flex: 1; min-width: 20px; height: 6px; border-radius: 3px; background: var(--raise); overflow: hidden; } .vu b { display: block; height: 100%; background: var(--accent2); transition: width 60ms linear; }
  .aktiv { border-color: var(--accent); background: var(--accentbg); }
  .stimmen { align-items: center; } .imp { display: flex; flex-direction: column; gap: 10px; min-width: 320px; } .imp label { display: flex; flex-direction: column; gap: 4px; }
  .klav { position: relative; display: flex; height: 92px; user-select: none; } .kt { flex: 1; position: relative; background: #f3f3f6; border: 1px solid #888; border-top: 0; border-radius: 0 0 4px 4px; color: #333; font-size: 9px; padding: 0; display: flex; align-items: flex-end; justify-content: center; min-width: 0; }
  .kt.schwarz { position: absolute; top: 0; width: 5.2%; height: 58%; flex: none; background: #1a1b22; border-color: #000; color: #ddd; z-index: 1; } .kt.klingt { background: var(--accent); color: #fff; } .tl { pointer-events: none; padding-bottom: 2px; }
  .kt:focus-visible { outline: 2px solid var(--accent2); outline-offset: -2px; }
  .zl.sel { background: var(--accentbg); } .zl.cur { outline: 2px solid var(--accent); outline-offset: -2px; background: var(--accentbg); } .zl:focus-visible { outline: 3px solid var(--accent2); outline-offset: -3px; }
  .foot > span { margin-right: 0.4em; } .foot { margin: 0; margin-top: auto; } .small { font-size: 12px; margin: 0; } .dim { color: var(--dim); } .inp { width: 100%; } .inp.num { width: 62px; }
  .rg { display: grid; grid-template-columns: 74px minmax(0, 1fr) 40px; align-items: center; gap: 8px; } .rg input[type="range"] { width: 100%; min-width: 0; } .wert { text-align: right; font-size: 12px; }
  .env { display: grid; grid-template-columns: repeat(16, 1fr); gap: 2px; } .eb { display: flex; flex-direction: column; align-items: center; gap: 2px; font-size: 10px; } .eb input[type="range"] { writing-mode: vertical-lr; direction: rtl; height: 80px; width: 14px; }
</style>
