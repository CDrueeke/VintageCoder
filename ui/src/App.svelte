<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { fragen } from "./lib/rueckfrage.svelte";
  import { onMount } from "svelte";
  import { t, currentLang } from "./lib/i18n.svelte";
  import { MODES, type Mode } from "./lib/nav";
  import { runCommand } from "./lib/commands";
  import { init, openProject, setMode, setSub, setLanguage, setTheme, toggleTheme, resolvedTheme, ui, visibleModes, visibleSubs } from "./lib/state.svelte";
  import { call, on, transportKind } from "./lib/api";
  import CopyDialog from "./lib/components/CopyDialog.svelte";
  import NewProjectDialog from "./lib/components/NewProjectDialog.svelte";
  import Splitter from "./lib/components/Splitter.svelte";
  import { lay, overlay, setSize, show, stacked, startLayout, watchWidth } from "./lib/layout.svelte";
  import { SIZES } from "./lib/layout";
  import { checkScratch, restoreSession, sess, startSitzung, flushSession } from "./lib/sitzung.svelte";
  import { startKontext } from "./lib/kontext";
  import { checkExternal, confirmLeave, diagEditor, ed, splitOn, hasDirty, initEditor, deleteFile, oeffneExtern, openFile, resetEditor, startSprachdienste, markierungenLaden } from "./lib/editor.svelte";
  import { comboParts, isMac, learnLayout, matchesSpec } from "./lib/keys";
  import CommandPalette from "./lib/components/CommandPalette.svelte";
  import CodeEditor from "./lib/components/CodeEditor.svelte";
  import { bau, buildRun, loadTargets, runStop, setTarget } from "./lib/bau.svelte";
  import MachinePanel from "./lib/components/MachinePanel.svelte";
  import MemStrip from "./lib/components/MemStrip.svelte";
  import Dock from "./lib/components/Dock.svelte";
  import { counts } from "./lib/lsp.svelte";
  import EditorDialogs from "./lib/components/EditorDialogs.svelte";
  import { bi } from "./lib/basicinfo.svelte";
  import ProjectTree from "./lib/components/ProjectTree.svelte";
  import TabBar from "./lib/components/TabBar.svelte";
  import TurboBanner from "./lib/components/TurboBanner.svelte";
  import TurboDialog from "./lib/components/TurboDialog.svelte";
  import ProjektView from "./views/ProjektView.svelte";
  import { checkSource, dlg, loadTurbo, startTurbo, statusText as turboStatus, tb } from "./lib/turbo.svelte";
  import Icon from "./lib/components/Icon.svelte";
  import Kbd from "./lib/components/Kbd.svelte";
  import Menu from "./lib/components/Menu.svelte";
  import Segment from "./lib/components/Segment.svelte";
  import Rueckfrage from "./lib/components/Rueckfrage.svelte";
  import Splash from "./lib/components/Splash.svelte";
  import Toasts from "./lib/components/Toasts.svelte";
  import DragGhost from "./lib/components/DragGhost.svelte";
  import DropLayer from "./lib/components/DropLayer.svelte";
  import { handleDrop, sperreDateiDrop, startDrop, startPfade } from "./lib/ablegen.svelte";
  import Tooltip from "./lib/components/Tooltip.svelte";
  import GalleryView from "./views/GalleryView.svelte";
  import MeasureView from "./views/MeasureView.svelte";
  import PlaceholderView from "./views/PlaceholderView.svelte";
  import StartView from "./views/StartView.svelte";
  import DiskView from "./views/DiskView.svelte";
  import SpriteView from "./views/SpriteView.svelte";
  import ZeichensatzView from "./views/ZeichensatzView.svelte";
  import PaletteView from "./views/PaletteView.svelte";
  import KachelView from "./views/KachelView.svelte";
  import KartenView from "./views/KartenView.svelte";
  import { kt } from "./lib/karten.svelte";
  import BildschirmView from "./views/BildschirmView.svelte";
  import TrackerView from "./views/TrackerView.svelte";
  import DebugView from "./views/DebugView.svelte";
  import HexView from "./views/HexView.svelte";
  import SettingsView from "./views/SettingsView.svelte";
  import WizardView from "./views/WizardView.svelte";
  import HelpView from "./views/HelpView.svelte";
  import FeedbackDialog from "./lib/components/FeedbackDialog.svelte";
  import ZahlenDialog from "./lib/components/ZahlenDialog.svelte";
  import SpeicherView from "./views/SpeicherView.svelte";
  import { beobachtenSetzen, dbg, debugTest } from "./lib/debug.svelte";
  import { md } from "./lib/markdown.svelte";
  import { hx, spk } from "./lib/daten.svelte";
  import { cfg, einstellungenOeffnen, inf, ri, rm, updateAutoStart } from "./lib/settings.svelte";
  import { wizAutoStart, wizStarten, wz, willkommen } from "./lib/wiz.svelte";
  import { hp } from "./lib/hilfe.svelte";
  import { tk } from "./lib/tracker.svelte";
  import { bs } from "./lib/bildschirm.svelte";
  import { pal } from "./lib/palette.svelte";
  import { kc } from "./lib/kachel.svelte";
  import { sp } from "./lib/sprite.svelte";
  import ExportDialog from "./lib/components/ExportDialog.svelte";
  import { dk } from "./lib/disk.svelte";

  let tauri = $state(true);
  let treeOpen = $state<Record<string, boolean>>({});
  const subs = $derived(visibleSubs(ui.mode));
  const paletteKeys = $derived(comboParts({ code: "KeyK", cmd: true, shift: false, alt: false }, { ctrl: t("key-ctrl"), shift: t("key-shift"), alt: "Alt" }));
  const langItems = $derived([
    { id: "auto", label: t("lang-choice-auto"), checked: ui.choice === "auto" && currentLang() !== "xx" },
    { id: "de", label: "Deutsch", checked: ui.choice === "de" && currentLang() === "de" },
    { id: "en", label: "English", checked: ui.choice === "en" && currentLang() === "en" },
    { id: "xx", label: t("lang-choice-pseudo"), checked: currentLang() === "xx" },
  ]);
  const themeItems = $derived([
    { id: "system", label: t("theme-system"), checked: ui.theme === "system" },
    { id: "light", label: t("theme-light"), checked: ui.theme === "light" },
    { id: "dark", label: t("theme-dark"), checked: ui.theme === "dark" },
  ]);

  // Projektwechsel: offene Reiter gehören zum alten Projekt
  let lastRoot: string | null = null;
  $effect(() => { const r = ui.project?.root ?? null; if (r !== lastRoot) { lastRoot = r; resetEditor(); if (r) void restoreSession(r); else sess.ready = false; void loadTurbo(); } });
  // Menüeinträge je nach Lage (Projekt offen, Editor aktiv) ein- oder ausschalten
  const spriteOn = $derived(ui.mode === "grafik" && (ui.sub.grafik === "sprites" || ui.sub.grafik === "zeichensatz") && sp.view !== null);
  /** Ein Dokument-Editor (Sprite, Zeichensatz, Kacheln, Karten, Bildschirm, Palette, Tracker, Disk) ist offen und sichtbar: dann gelten Rückgängig, Speichern & Co. des Menüs (sonst sind sie gesperrt und das Kürzel erreicht den Editor nicht) */
  const docOn = $derived(spriteOn || (ui.mode === "grafik" && ((ui.sub.grafik === "kacheln" && kc.view !== null) || (ui.sub.grafik === "karten" && kt.view !== null) || (ui.sub.grafik === "bildschirm" && bs.view !== null) || (ui.sub.grafik === "palette" && pal.view !== null))) || (ui.mode === "musik" && tk.view !== null) || (ui.mode === "daten" && ui.sub.daten === "disk" && dk.view !== null));
  const editorActive = $derived(ui.mode === "code" && ui.extra === null && ed.active !== null);
  $effect(() => { const p = !!ui.project, e = editorActive; void docOn; const tu = !!tb.state?.turbo; if (ui.ready) call("menu_context", { project: p, editor: e || docOn, turbo: tu }).catch(() => {}); });

  /** Ziehen mit gedrückter Maustaste auf leerer Fläche startet im WebKit eine „Auswahl“ und scrollt dabei überlaufende Bereiche (Disk-Editor) von selbst: dort Standardverhalten abstellen.
   *  Eingabefelder, Editor, Knöpfe, ziehbare Elemente und Bildlaufleisten bleiben unberührt (Ziehen aus dem Finder und Zeilen-Umordnen laufen über eigene Wege). */
  function mausKeinAutoscroll(e: MouseEvent) {
    if (e.button !== 0 || e.defaultPrevented) return; const el = e.target as HTMLElement | null; if (!el?.closest) return;
    if (el.closest('input,textarea,select,option,button,a,label,summary,[contenteditable],[draggable="true"],.cm-editor,[data-drag],[data-allow-select]')) return;
    if ((el.clientWidth && e.offsetX >= el.clientWidth && el.offsetWidth > el.clientWidth) || (el.clientHeight && e.offsetY >= el.clientHeight && el.offsetHeight > el.clientHeight)) return;   // Bildlaufleiste
    e.preventDefault(); (el.closest('[tabindex]') as HTMLElement | null)?.focus({ preventScroll: true });   // Fokus wie bei einem normalen Klick (Tastenkürzel auf der Karte, Listen)
  }
  onMount(async () => {
    await init(); initEditor(); startLayout(); startSitzung(); startKontext(); void startTurbo(); void startSprachdienste(); void checkScratch(); setTimeout(() => void updateAutoStart(), 4000); setTimeout(() => void wizAutoStart(), 800);
    await on("menu:cmd", (p) => { const id = (p as { id: string }).id; if (once(id)) void runCommand(id); });
    const kind = await transportKind();
    if (import.meta.env.VITE_SELFTEST) { const { selftest } = await import("./lib/selftest"); void selftest(); }
    tauri = kind === "tauri"; void startDrop(kind); sperreDateiDrop(); await on("app:oeffnen", () => void startPfade(kind)); void startPfade(kind);
    try { if (localStorage.getItem("vcDiag") === "1") (window as unknown as Record<string, unknown>).__vcDiag = diagEditor; } catch { /* ohne Speicher */ }   // Diagnose im echten Fenster: localStorage.vcDiag = "1", neu laden, dann window.__vcDiag()
    // Fenster schließen mit ungespeicherten Änderungen: erst nachfragen
    if (tauri) { const { getCurrentWindow } = await import("@tauri-apps/api/window"); const w = getCurrentWindow(); await w.onCloseRequested(async (e) => { await flushSession(); if (hasDirty()) { e.preventDefault(); if (await confirmLeave()) await w.destroy(); } }); }
    // Nur mit Prüfbrücke: Steuerung für Bildaufnahmen und Playwright (im Auslieferungsbau gibt es keine Brücke)
    if (import.meta.env.DEV && kind === "bridge") (window as unknown as Record<string, unknown>).__vc = { call, ui, ed, dk, sp, lay, runCommand, openProject, pal, kc, bs, tk, dbg, hx, spk, cfg, inf, wz, hp, setTheme, setLanguage, openFile, handleDrop, tb, dlg, checkSource, debugTest, deleteFile, oeffneExtern, md, markierungenLaden, rm, ri, kt, beobachtenSetzen };
  });

  // Ohne natives Menü (Browser mit Prüfbrücke) übernimmt die Oberfläche die Kürzel der Befehlstabelle selbst
  // Kürzel können als Menüereignis und als Tastenereignis ankommen (oder nur als Tastenereignis): derselbe Befehl läuft innerhalb von 150 ms nur einmal
  const lastRun = new Map<string, number>();
  function once(id: string): boolean { const n = Date.now(); if (n - (lastRun.get(id) ?? 0) < 150) return false; lastRun.set(id, n); return true; }
  function keydown(e: KeyboardEvent) {
    learnLayout(e);
    if (e.defaultPrevented) return;   // schon vom Editor behandelt (sonst doppelt)
    // Zoom größer auch mit „+“ und „=“ (auf der deutschen Tastatur ist „+“ eine eigene Taste, deren Code nicht der des Menükürzels ist)
    if ((e.metaKey || e.ctrlKey) && !e.altKey && (e.key === "+" || e.key === "=")) { e.preventDefault(); if (once("zoom_plus")) void runCommand("zoom_plus"); return; }
    for (const c of ui.commands) {
      if (!c.keys || !matchesSpec(e, c.keys)) continue;
      // Im echten Fenster gehören Kürzel dem nativen Menü; als Rückfall laufen nur Befehle, die immer erlaubt sind (das Menü sperrt die anderen je nach Lage)
      if (tauri && c.needs !== "none") {
        // Speichern ist wiederholbar ohne Schaden: es läuft auch von hier, falls das native Menü die Taste nicht auslöst (unter Windows nicht jeder Weg zum Menü funktioniert) oder „Speichern“ dort gesperrt steht
        const lage = c.needs === "project" ? !!ui.project : ed.active !== null || docOn;
        if ((c.id === "speichern" || c.id === "alle_speichern") && lage) { e.preventDefault(); if (once(c.id)) void runCommand(c.id); }
        return;
      }
      e.preventDefault(); if (once(c.id)) void runCommand(c.id); return;
    }
  }
  void call; void toggleTheme; void resolvedTheme; void isMac;

  /** Zähler der Statuszeile für die Datei im Reiter: Fehler und Warnungen, dazu Hinweise (Schweregrad 3, z. B. Namen, die nur im Kontext einer einbindenden Datei gelten) */
  function problemText(): string {
    const c = counts(ed.active ? [ed.active] : []); const all = counts(); const others = all.errors + all.warnings + all.hints - (c.errors + c.warnings + c.hints);
    const h = c.hints > 0 ? t("status-hinweise", { h: c.hints }) : ""; const rest = others > 0 ? ` · ${t("status-andere-dateien", { n: others })}` : "";   // Die Zeile gilt der Datei im Reiter; Meldungen anderer Dateien (auch Hinweise) nennt sie ehrlich dazu
    if (c.errors + c.warnings === 0) return (h || t("status-problems", { e: 0, w: 0 })) + rest;
    return t("status-problems", { e: c.errors, w: c.warnings }) + (h ? ` · ${h}` : "") + rest;
  }

  /** Pfad in der Mitte kürzen (Anfang und Ende bleiben erkennbar); der volle Pfad steht im Tooltip */
  function midShort(p: string, max: number): string { if (p.length <= max) return p; const tail = Math.ceil(max * 0.6), head = max - tail - 1; return `${p.slice(0, head)}…${p.slice(p.length - tail)}`; }
  let targetMenu = $state(false);
  /** Anzeige des Baus in der Statuszeile */
  const bauText = $derived(bau.running ? t("bar-baut") : bau.last ? (bau.last.ok ? t("bar-gebaut", { size: bau.last.info?.size ?? 0 }) : t("bar-fehler", { n: bau.last.errors })) : "");

  const mode = $derived(ui.mode as Mode);
  const projectName = $derived(ui.project?.name ?? t("titlebar-no-project"));
  const sideKeys = $derived(comboParts({ code: "KeyB", cmd: true, shift: false, alt: false }, { ctrl: t("key-ctrl"), shift: t("key-shift"), alt: "Alt" }).join(isMac ? "" : "+"));
  const codeView = $derived(ui.mode === "code" && !!ui.project && ui.extra === null && !ui.zen);
  const sideShow = $derived(show("seitenleiste")), panelShow = $derived(show("panel")), dockShow = $derived(show("schublade")), docShow = $derived(show("dokument"));
  const showTree = $derived(codeView && sideShow !== "min" && (ui.sidebar || lay.max === "seitenleiste"));
  const showStrip = $derived(codeView && (!showTree || overlay()));
  const showPanel = $derived(codeView && panelShow !== "min" && (ui.panel || lay.max === "panel"));
  /** „Startseite“ in der Hauptnavigation: ohne Projekt sofort, mit Projekt erst nach Rückfrage (das Projekt wird geschlossen) */
  async function zurStartseite(): Promise<void> {
    if (ui.project) { if (!(await fragen(t("nav-start-frage-titel"), t("nav-start-frage-text", { name: ui.project.name ?? "" }), t("nav-start-frage-ok")))) return; }
    const { runCommand } = await import("./lib/commands"); await runCommand("startseite");
  }
</script>

<svelte:window onkeydown={keydown} onmousedown={mausKeinAutoscroll} onfocus={() => void checkExternal()} />

<div class="app">
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <header class="tb" data-tauri-drag-region>
    <Tooltip text={t("cmd-seitenleiste")}><button type="button" class="ibtn" class:on={ui.sidebar} aria-pressed={ui.sidebar} aria-label={t("cmd-seitenleiste")} onclick={() => (ui.sidebar = !ui.sidebar)}><Icon name="side" size={18} /></button></Tooltip>
    <div class="pname" style="display:flex;flex-direction:column;line-height:1.15;pointer-events:none;min-width:0;max-width:340px"><b data-ellipsis style="font-size:13px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis">{projectName}</b>
      <span title={ui.project?.pretty ?? ""} style="font-size:11px;color:var(--dim);white-space:nowrap;pointer-events:auto" class="mono">{ui.project ? midShort(ui.project.pretty, 46) : "VintageCoder"}</span></div>
    <span class="sp"></span>
    <nav aria-label={t("nav-main-label")}>
      <Segment kind="nav" label={t("nav-main-label")} items={[{ id: "start", label: t("nav-main-start"), icon: "home" }, ...visibleModes().map((m) => ({ id: m.id, label: t(m.label), icon: m.icon }))]} value={ui.mode === "code" && !ui.project ? "start" : ui.mode} onselect={(id) => void (id === "start" ? zurStartseite() : setMode(id as Mode))}>
        {#snippet item(it)}<Icon name={it.icon ?? "code"} size={15} /><span class="nl">{it.label}</span>{/snippet}
      </Segment>
    </nav>
    <span class="sp"></span>
    {#if ui.project}
      <Tooltip text={`${t("bar-bauen")} (F7)`}><button type="button" class="ibtn" aria-label={t("bar-bauen")} disabled={bau.running} onclick={() => void buildRun()}><Icon name="build" size={17} /></button></Tooltip>
      <Tooltip text={`${t("bar-starten")} (F5)`}><button type="button" class="ibtn" aria-label={t("bar-starten")} disabled={bau.running || bau.starting} onclick={() => void buildRun({ then: "run" })}><Icon name="play" size={17} /></button></Tooltip>
      {#if bau.started}<Tooltip text={t("bar-stopp")}><button type="button" class="ibtn" aria-label={t("bar-stopp")} onclick={() => void runStop()}><Icon name="stop" size={15} /></button></Tooltip>{/if}
      <div class="tgt">
        <button type="button" class="chip" aria-haspopup="menu" aria-expanded={targetMenu} aria-label={t("bar-ziel")} onclick={() => { void loadTargets(); targetMenu = !targetMenu; }}><Icon name="chip" size={14} /><b>{ui.project.systems.join(" · ")}</b>{#if bau.targets}<span class="tgtname" data-ellipsis>· {bau.targets.name}</span>{/if}</button>
        {#if targetMenu && bau.targets}
          <div class="vc-fixmenu" style="position:absolute;right:0;top:34px" role="menu" aria-label={t("bar-ziel")}>
            {#each bau.targets.targets as z (z.id)}<button type="button" role="menuitem" onclick={() => { targetMenu = false; void setTarget(z.id); }}>{z.id === bau.targets.current ? "✓ " : ""}{z.name}</button>{/each}
          </div>
        {/if}
      </div>
    {/if}
    <Menu label={t("titlebar-language")} radio align="right" items={langItems} onselect={(id) => setLanguage(id as "auto" | "de" | "en" | "xx")}>{#snippet button()}<Icon name="globe" size={17} />{/snippet}</Menu>
    <Menu label={t("titlebar-theme")} radio align="right" items={themeItems} onselect={(id) => setTheme(id as "system" | "light" | "dark" | "contrast")}>{#snippet button()}<Icon name={resolvedTheme() === "dark" ? "moon" : "sun"} size={17} />{/snippet}</Menu>
    <Tooltip text={t("cmd-einstellungen")}><button type="button" class="ibtn" aria-label={t("cmd-einstellungen")} data-act="einstellungen" onclick={() => void einstellungenOeffnen()}><Icon name="gear" size={17} /></button></Tooltip>
    <Tooltip text={t("cmd-palette")}><button type="button" class="ibtn" aria-label={t("cmd-palette")} onclick={() => (ui.palette = true)}><Icon name="search" size={17} /></button></Tooltip>
  </header>

  <div class="subbar">
    {#if subs.length > 0 && ui.extra === null}
      <Segment small kind="nav" label={t("nav-sub-label")} items={subs.map((s) => ({ id: s.id, label: t(s.label) }))} value={ui.sub[ui.mode] ?? ""} onselect={(id) => setSub(mode, id)} />
    {/if}
  </div>

  <main class="body" use:watchWidth={(w) => (lay.mainW = w)}>
    {#if showTree && ui.project}
      <section class="card" class:ovl={overlay()} style={lay.max === "seitenleiste" ? "flex:1" : `width:${lay.sideW}px;flex:none`} data-area="seitenleiste" aria-label={t("tree-heading")}>
        <ProjectTree bind:open={treeOpen} />
      </section>
      {#if lay.max !== "seitenleiste" && !overlay()}<Splitter value={lay.sideW} min={SIZES.sideW.min} max={SIZES.sideW.max} label={t("splitter-seite")} dir="right" reset={SIZES.sideW.def} onchange={(v) => setSize("sideW", v)} />{/if}
    {:else if showStrip}
      <div class="sidestrip card" data-area="seitenleiste" role="toolbar" aria-orientation="vertical" aria-label={t("tree-heading")}>
        <Tooltip text={`${t("cmd-seitenleiste")} (${sideKeys})`}><button type="button" class="ibtn" aria-label={t("cmd-seitenleiste")} data-strip="open" onclick={() => { lay.max = null; ui.sidebar = true; }}><Icon name="chevr" size={14} /></button></Tooltip>
        <Tooltip text={t("tree-heading")}><button type="button" class="ibtn" aria-label={t("tree-heading")} data-strip="files" onclick={() => { lay.max = null; ui.sidebar = true; }}><Icon name="folder" size={17} /></button></Tooltip>
        <Tooltip text={t("dock-search")}><button type="button" class="ibtn" aria-label={t("dock-search")} data-strip="search" onclick={() => { if (lay.max && lay.max !== "schublade") lay.max = null; ed.dock = "search"; }}><Icon name="search" size={17} /></button></Tooltip>
      </div>
    {/if}
    {#if docShow !== "min" || lay.max === "schublade"}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <section class="card" style="flex:1" data-area-wrap="dokument" onpointerdowncapture={() => { if (overlay() && (ui.sidebar || ui.panel)) { ui.sidebar = false; ui.panel = false; } }} aria-label={ui.extra ? t(ui.extra === "gallery" ? "gallery-title" : "measure-title") : t(MODES.find((m) => m.id === ui.mode)!.title)}>
      {#if ui.extra === "gallery"}<GalleryView />
      {:else if ui.extra === "settings"}<SettingsView />
      {:else if ui.extra === "wizard"}<WizardView />
      {:else if ui.extra === "help"}<HelpView />
      {:else if ui.extra === "measure"}<MeasureView />
      {:else if ui.mode === "daten" && ui.sub.daten === "disk"}<DiskView />
      {:else if ui.mode === "daten" && ui.sub.daten === "hex"}<HexView />
      {:else if ui.mode === "daten" && ui.sub.daten === "speicher"}<SpeicherView />
      {:else if ui.mode === "grafik" && ui.sub.grafik === "sprites"}<SpriteView />
      {:else if ui.mode === "grafik" && ui.sub.grafik === "zeichensatz"}<ZeichensatzView />
      {:else if ui.mode === "grafik" && ui.sub.grafik === "palette"}<PaletteView />
      {:else if ui.mode === "grafik" && ui.sub.grafik === "kacheln"}<KachelView />
      {:else if ui.mode === "grafik" && ui.sub.grafik === "karten"}<KartenView />
      {:else if ui.mode === "grafik" && ui.sub.grafik === "bildschirm"}<BildschirmView />
      {:else if ui.mode === "projekt"}<ProjektView />
      {:else if ui.mode === "musik"}<TrackerView />
      {:else if ui.mode === "debug"}<DebugView />
      {:else if ui.mode === "code" && !ui.project}<StartView />
      {:else if ui.mode === "code" && ui.project}
        <TurboBanner />
        <div class="docs" class:split={splitOn()} class:stack={splitOn() && stacked()} class:fold={lay.max === "schublade"} use:watchWidth={(w) => (lay.docsW = w)}>
          {#each splitOn() ? [0, 1] as const : [0] as const as g (g)}
            <div class="grp" class:second={g === 1} role="group" aria-label={splitOn() ? t(g === 0 ? "split-gruppe-links" : "split-gruppe-rechts") : t("split-gruppe-dokument")}>
              <TabBar group={g} />
              {#if lay.max !== "schublade"}{#if ed.activeBy[g]}<CodeEditor group={g} />{:else}<div class="empty"><p style="max-width:380px">{t("editor-leer")}</p></div>{/if}{/if}
            </div>
          {/each}
        </div>
        {#if !ui.zen}<Dock />{/if}
      {:else}<PlaceholderView {mode} />{/if}
    </section>
    {:else if codeView}
      <button type="button" class="strip" aria-label={t("bereich-wiederherstellen")} data-max="restore" onclick={() => (lay.max = null)}><Icon name="restore" size={12} /></button>
    {/if}
    {#if showPanel}
      {#if lay.max !== "panel" && !overlay()}<Splitter value={lay.panelW} min={SIZES.panelW.min} max={SIZES.panelW.max} label={t("splitter-panel")} dir="left" reset={SIZES.panelW.def} onchange={(v) => setSize("panelW", v)} />{/if}
      <section class="card" class:ovl={overlay()} class:ovr={overlay()} style={lay.max === "panel" ? "flex:1" : `width:${lay.panelW}px;flex:none`} data-area="panel" aria-label={t("machine-title")}><MachinePanel /></section>
    {/if}
  </main>

  {#if !ui.zen}<footer class="status">
    <span>{ui.project ? `${ui.project.name}` : t("tree-no-project")}</span>
    {#if editorActive}<span style="margin-left:14px">{t(`status-lang-${ed.tabs.find((x) => x.rel === ed.active)?.lang ?? "plain"}`)}{#if ed.active && bi.byRel[ed.active]?.dialect}{" · "}<span data-testid="status-dialekt">{bi.byRel[ed.active]?.dialect?.name}</span>{/if} · {ed.tabs.find((x) => x.rel === ed.active)?.eol === "crlf" ? "CRLF" : "LF"}</span><button type="button" class="btn" style="margin-left:10px;border:0;background:none;color:var(--dim)" aria-pressed={ed.dock === "problems"} onclick={() => (ed.dock = ed.dock === "problems" ? null : "problems")}>{problemText()}</button><span style="margin-left:14px" class="mono">{t("status-cursor", { line: ed.cursor.line, col: ed.cursor.col })}{#if ed.cursor.selected} · {t("status-selected", { n: ed.cursor.selected })}{/if}</span>{/if}
    {#if dbg.s?.active}<button type="button" class="btn" style="margin-left:10px;border:0;background:none;color:var(--accent2)" data-status="debug-pc" onclick={() => setMode("debug")}>{dbg.s.running ? t("dbgui-status-laeuft") : dbg.s.regs ? t("status-debug-pc", { pc: dbg.s.regs.pc.toString(16).toUpperCase().padStart(4, "0") }) : t("dbgui-status-startet")}</button>{/if}
    {#if turboStatus()}<span style="margin-left:14px" data-testid="turbo-status">{turboStatus()}</span>{/if}
    {#if bauText}<button type="button" class="btn" style="margin-left:10px;border:0;background:none;color:var(--dim)" onclick={() => (ed.dock = "output")}>{bauText}{#if bau.started} · {t("bar-laeuft")}{/if}</button><MemStrip />{/if}
    {#if wz.hinweis && ui.project && !wz.open && !willkommen.weg}<button type="button" class="btn" style="margin-left:10px;border:0;background:none;color:var(--accent2)" data-status="einrichtung" onclick={() => void wizStarten("quick")}>{t("status-einrichtung-hinweis")}</button>{/if}
    {#if inf.upd?.badge}<button type="button" class="btn" style="margin-left:10px;border:0;background:none;color:var(--accent2)" data-status="update" onclick={() => void einstellungenOeffnen("info")}>{t("info-update-hinweis", { v: inf.upd.badge })}</button>{/if}
    <span class="sp"></span>
    <span class="hint"><Kbd parts={paletteKeys} /><button type="button" class="btn" style="border:0;background:none;color:var(--dim)" onclick={() => (ui.palette = true)}>{t("status-keys-hint")}</button></span>
  </footer>{/if}
</div>
<CommandPalette />
<FeedbackDialog />
<ZahlenDialog />
<EditorDialogs />
<NewProjectDialog />
<CopyDialog />
<ExportDialog />
<TurboDialog />
<Toasts />
<Rueckfrage />
<Splash />
<DragGhost />
<DropLayer />
