// SPDX-License-Identifier: GPL-3.0-or-later
// Befehlsausführung der Oberfläche. Menüleiste, Befehlspalette und Kürzel liefern dieselben Kennungen (Tabelle `vc-commands`).
// Befehle, die erst in späteren Etappen gebaut werden, melden das in einer Meldung statt still nichts zu tun.
import { foldAll, unfoldAll } from "@codemirror/language";
import { openSearchPanel } from "@codemirror/search";
import { nextDiagnostic, previousDiagnostic } from "@codemirror/lint";
import { startCompletion } from "@codemirror/autocomplete";
import { redo, selectAll, undo } from "@codemirror/commands";
import { call } from "./api";
import { toggleLineMark } from "./cm";
import { cursorMatchingBracket, toggleLineComment } from "@codemirror/commands";
import { formatFile, formatLine, sortLines, trimTrailing } from "./cm-format";
import { focusMax, lay, resetLayout, toggleMax } from "./layout.svelte";
import { openCopy } from "./kopieren.svelte";
import { openNewProject } from "./neuprojekt.svelte";
import { flushSession } from "./sitzung.svelte";
import { bau, buildRun, clearLog, logText, problemLines, runStop } from "./bau.svelte";
import { lsp } from "./lsp.svelte";
import { runTests } from "./tests.svelte";
import { gotoDefinition, quickFix } from "./cm-lsp";
import { aktiv as diskAktiv, diskRedo, diskUndo, dk, pickDisk, saveDisk } from "./disk.svelte";
import { openExport } from "./export.svelte";
import { openHex } from "./daten.svelte";
import { wizStarten } from "./wiz.svelte";
import { hilfeOeffnen, hilfeZuWort } from "./hilfe.svelte";
import { einstellungenOeffnen, feedbackOeffnen, updatePruefen } from "./settings.svelte";
import { aktiv as debugAktiv, debugEnde, debugStarten, hierher as dbgHierher, hinein as dbgHinein, pause as dbgPause, raus as dbgRaus, ueber as dbgUeber, weiter as dbgWeiter } from "./debug.svelte";
import { aktiv as trackerAktiv, saveSong, songRedo, songUndo, tk, trackerOp } from "./tracker.svelte";
import { aktiv as bildschirmAktiv, bildschirmRedo, bildschirmUndo, bs, saveBildschirm } from "./bildschirm.svelte";
import { aktiv as kachelAktiv, kachelRedo, kachelUndo, kc, saveKacheln } from "./kachel.svelte";
import { aktiv as kartenAktiv, kartenRedo, kartenUndo, kt, saveKarten } from "./karten.svelte";
import { aktiv as paletteAktiv, pal, paletteRedo, paletteUndo, savePalette } from "./palette.svelte";
import { aktiv as spriteAktiv, copySel, loadPng, pasteAt, saveSprite, selectAll as spriteSelectAll, sp, spriteRedo, spriteUndo } from "./sprite.svelte";
import { otherHalf, splitToggle, closeTab, confirmLeave, copyPath, deleteAsk, ed, findNode, fontStep, fmtHooks, getView, toggleEol, toggleWrap, bookmarkStep, bookmarkToggle, gotoLineAsk, jumpTo, lspHooks, newDirIn, newFileIn, newNoteIn, newScratch, openFile, parentOf, renameAsk, revealIn, saveActive, saveAll, setLang, shiftTab, stepTab, treeUndo } from "./editor.svelte";
import { t } from "./i18n.svelte";
import { checkCopy, isTurbo, openInDiskEditor, openProvide } from "./turbo.svelte";
import { search, startSearch } from "./suche.svelte";
import { type Mode } from "./nav";
import { mdAls, mdBefehl, mdBefehle, mdUmschalten } from "./markdown.svelte";
import { openConv } from "./basicconv.svelte";
import { zahlenOeffnen } from "./zahlen.svelte";
import { zwischenablage } from "./zwischenablage";
import { alsInclude } from "./include.svelte";
import { closeProject, fail, openProject, pickProject, setLanguage, setMode, setSub, setTheme, setZoom, toast, toggleTheme, ui, visibleModes, visibleSubs } from "./state.svelte";

function inEditor(f: (v: import("@codemirror/view").EditorView) => unknown): void { const v = getView(); if (v) { f(v); v.focus(); } }
function nodeKind(rel: string): "file" | "dir" { return findNode(ui.project?.tree ?? [], rel)?.kind ?? "file"; }
/** Zielordner für „Neu“: gewählter Ordner, sonst Ordner der gewählten Datei, sonst Wurzel */
function targetFolder(): string { const s = ui.selected; if (!s) return ""; return nodeKind(s) === "dir" ? s : parentOf(s); }

export async function runCommand(id: string): Promise<void> {
  if (id.startsWith("ext:")) { toast(t("ext-werkzeuge-folgen")); return; }   // Befehle aus Erweiterungen: die Werkzeuge folgen mit E2
  if (id.startsWith("nav:")) {
    const [, mode, sub] = id.split(":") as [string, Mode, string | undefined];
    if (sub) setSub(mode, sub); else setMode(mode);
    return;
  }
  if (id.startsWith("file:")) { await openFile(id.slice(5)); ui.mode = "code"; ui.extra = null; return; }
  if (id.startsWith("recent:")) { if (await confirmLeave()) { await flushSession(); await openProject(id.slice(7)); } return; }
  if (id.startsWith("lang:")) { await setLanguage(id.slice(5) as "auto" | "de" | "en" | "xx"); return; }
  if (mdBefehle.includes(id)) { await mdBefehl(id); return; }
  switch (id) {
    case "md_umschalten": mdUmschalten(); return;
    case "md_als": mdAls(); return;
    case "ui:gallery": ui.extra = "gallery"; return;
    case "ui:measure": ui.extra = "measure"; return;
    case "palette": ui.palette = true; return;
    case "hell_dunkel": toggleTheme(); return;
    case "seitenleiste": ui.sidebar = !ui.sidebar; return;
    case "projekt_neu": if (await confirmLeave()) { await flushSession(); await openNewProject(); } return;
    case "disk_oeffnen": await pickDisk(); return;
    case "projekt_kopieren": if (ui.project) await openCopy(ui.project.root); else toast(t("kopierdlg-projekt-kopieren-erst-ein-projekt-oeffnen"), true); return;
    case "zahlenwerkzeuge": zahlenOeffnen(); return;
    case "als_include": alsInclude(ui.selected, !!ui.selected && nodeKind(ui.selected) === "file"); return;
    case "projekt_ordner_oeffnen": if (await confirmLeave()) { await flushSession(); await pickProject(); } return;
    case "projekt_oeffnen": if (await confirmLeave()) { await flushSession(); await pickProject(); } return;
    case "projekt_schliessen": case "startseite": if (await confirmLeave()) { await flushSession(); await closeProject(); setMode("code"); } return;   // ohne Projekt zeigt „Code“ die Startseite
    case "tracker_noten": document.querySelector<HTMLElement>("[data-tk=zelle][tabindex='0']")?.focus(); return;
    case "tracker_instrumente": document.querySelector<HTMLElement>("[data-tk=instrument][tabindex='0']")?.focus(); return;
    case "tracker_ablauf": document.querySelector<HTMLElement>("[data-tk=eintrag][tabindex='0']")?.focus(); return;
    case "projekt_bereich": setMode("projekt"); return;
    case "projekt_einstellungen": setSub("projekt", ui.sub.projekt && ui.sub.projekt !== "uebersicht" ? ui.sub.projekt : "allgemein"); return;
    case "turbo_bereitstellen": openProvide(); return;
    case "turbo_pruefen": await checkCopy(); return;
    case "turbo_im_disk_editor": openInDiskEditor(); return;
    case "debuggen": case "debuggen_aktiv": if (isTurbo()) { toast(t("turboui-debuggen-hinweis"), true); return; } await debugStarten(); return;
    case "debug_weiter": if (debugAktiv()) await dbgWeiter(); else await debugStarten(); return;
    case "debug_pause": await dbgPause(); return;
    case "debug_ueber": await dbgUeber(); return;
    case "debug_hinein": await dbgHinein(); return;
    case "debug_raus": await dbgRaus(); return;
    case "debug_hierher": await dbgHierher(ed.active && !ed.tabs.find((x) => x.rel === ed.active)?.scratch ? ed.active : null, ed.cursor.line); return;
    case "speichern": if (trackerAktiv()) await saveSong(); else if (bildschirmAktiv()) await saveBildschirm(); else if (kartenAktiv()) await saveKarten(); else if (kachelAktiv()) await saveKacheln(); else if (paletteAktiv()) await savePalette(); else if (spriteAktiv()) await saveSprite(); else if (diskAktiv()) await saveDisk(); else await saveActive(); return;
    case "alle_speichern": await saveAll(); if (sp.view?.dirty) await saveSprite(); if (pal.view?.dirty) await savePalette(); if (kc.view?.dirty) await saveKacheln(); if (kt.view?.dirty) await saveKarten(); if (bs.view?.dirty) await saveBildschirm(); if (tk.view?.dirty) await saveSong(); if (dk.view?.dirty && dk.view.path) await saveDisk(); return;
    case "datei_schliessen": if (ed.active) closeTab(ed.active); return;
    case "neue_datei": newFileIn(targetFolder()); return;
    case "neuer_reiter": newScratch(false); ui.mode = "code"; return;
    case "neue_notiz": newScratch(true); ui.mode = "code"; return;
    case "neue_notiz_dialog": newNoteIn(targetFolder()); return;
    case "datei_oeffnen": { const r = await call("file_pick").catch((e) => { fail(e); return null; }); if (r?.rel) await openFile(r.rel); else if (r?.outside) toast(t("editor-draussen", { path: r.outside })); return; }
    case "ziel_oeffnen": if (ui.selected && nodeKind(ui.selected) === "file") await openFile(ui.selected); return;
    case "naechster_reiter": stepTab(1); return;
    case "voriger_reiter": stepTab(-1); return;
    case "reiter_links": shiftTab(-1); return;
    case "reiter_rechts": shiftTab(1); return;
    case "art_text": setLang("plain"); return;
    case "art_asm": setLang("asm"); return;
    case "art_basic": setLang("basic"); return;
    case "zen": ui.zen = !ui.zen; return;
    case "teilen": splitToggle(); return;
    case "reiter_andere_haelfte": otherHalf(); return;
    case "spalte_max": toggleMax("seitenleiste"); return;
    case "dokument_max": toggleMax("dokument"); return;
    case "schublade_max": if (!ed.dock && lay.max !== "schublade") ed.dock = "problems"; toggleMax("schublade"); return;
    case "panel_max": toggleMax("panel"); return;
    case "maximiere_fokus": focusMax(); return;
    case "layout_zuruecksetzen": resetLayout(); toast(t("layout-zurueck")); return;
    case "editor_schrift_groesser": fontStep(1); return;
    case "editor_schrift_kleiner": fontStep(-1); return;
    case "baum_rueckgaengig": await treeUndo(); return;
    case "einstellungen": await einstellungenOeffnen(); return;
    case "hilfe": await hilfeZuWort(); return;
    case "hilfe_reiter": await hilfeOeffnen(); return;
    case "assistent": await wizStarten("quick"); return;
    case "assistent_neues": await wizStarten("roms"); return;
    case "assistent_roms": await wizStarten("roms"); return;
    case "assistent_emulatoren": await wizStarten("emulators"); return;
    case "assistent_hardware": await wizStarten("hardware"); return;
    case "assistent_design": await wizStarten("design"); return;
    case "ueber": await einstellungenOeffnen("info"); return;
    case "feedback": await feedbackOeffnen(); return;
    case "update_pruefen": await einstellungenOeffnen("info"); await updatePruefen(); return;
    case "zoom_plus": setZoom(ui.zoom + 0.1); return;
    case "zoom_minus": setZoom(ui.zoom - 0.1); return;
    case "zoom_null": setZoom(1); return;
    case "hoher_kontrast": setTheme(ui.theme === "contrast" ? "system" : "contrast"); return;
    case "darstellung_zuruecksetzen": setTheme("system"); setZoom(1); return;
    case "hex_ansicht": if (ui.selected) await openHex(ui.selected); return;
    case "pfad_kopieren": if (ui.selected) await copyPath(ui.selected); return;
    case "im_ordner_zeigen": if (ui.selected) await revealIn(ui.selected); return;
    case "neuer_ordner": newDirIn(targetFolder()); return;
    case "datei_umbenennen": if (ui.selected) renameAsk(ui.selected, nodeKind(ui.selected)); return;
    case "datei_loeschen": if (ui.selected && nodeKind(ui.selected) === "file") await deleteAsk(ui.selected); return;
    case "rueckgaengig": if (trackerAktiv()) await songUndo(); else if (bildschirmAktiv()) await bildschirmUndo(); else if (kartenAktiv()) await kartenUndo(); else if (kachelAktiv()) await kachelUndo(); else if (paletteAktiv()) await paletteUndo(); else if (spriteAktiv()) await spriteUndo(); else if (diskAktiv()) await diskUndo(); else inEditor(undo); return;
    case "wiederholen": if (trackerAktiv()) await songRedo(); else if (bildschirmAktiv()) await bildschirmRedo(); else if (kartenAktiv()) await kartenRedo(); else if (kachelAktiv()) await kachelRedo(); else if (paletteAktiv()) await paletteRedo(); else if (spriteAktiv()) await spriteRedo(); else if (diskAktiv()) await diskRedo(); else inEditor(redo); return;
    case "suchen": case "ersetzen": inEditor(openSearchPanel); return;
    case "gehe_zu_zeile": gotoLineAsk(); return;
    case "alles_falten": inEditor(foldAll); return;
    case "alles_entfalten": inEditor(unfoldAll); return;
    case "alle_auswaehlen": if (spriteAktiv()) await spriteSelectAll(); else inEditor(selectAll); return;
    case "gehe_zu_definition": { const v = getView(); if (v) { await gotoDefinition(v, lspHooks); v.focus(); } else toast(t("lsp-keine-dienste"), true); return; }
    case "zeile_formatieren": case "datei_formatieren": {
      const v = getView(); if (!v) return;
      if (!fmtHooks.target()) { toast(t("format-nicht-moeglich"), true); return; }
      const changed = await (id === "zeile_formatieren" ? formatLine(v, fmtHooks) : formatFile(v, fmtHooks));
      toast(t(changed ? "format-fertig" : "format-nichts")); v.focus(); return;
    }
    case "in_projekt_suchen": {
      ui.mode = "code"; ui.extra = null; ed.dock = "search";
      const v = getView(); if (v && !v.state.selection.main.empty) { const q = v.state.sliceDoc(v.state.selection.main.from, v.state.selection.main.to); if (!q.includes("\n")) { search.text = q; void startSearch(); } }
      search.focus++; return;
    }
    case "umbruch": toggleWrap(); return;
    case "lesezeichen_umschalten": bookmarkToggle(); return;
    case "naechstes_lesezeichen": bookmarkStep(1); return;
    case "voriges_lesezeichen": bookmarkStep(-1); return;
    case "zeilenende": toggleEol(); return;
    case "kommentar_umschalten": inEditor(toggleLineComment); return;
    case "zur_klammer": inEditor(cursorMatchingBracket); return;
    case "leerzeichen_entfernen": { const v = getView(); if (v) { toast(t(trimTrailing(v) ? "format-fertig" : "format-nichts")); v.focus(); } return; }
    case "zeilen_sortieren": { const v = getView(); if (!v) return; const r = sortLines(v); toast(r === null ? t("format-sortieren-auswahl") : t(r ? "format-fertig" : "format-nichts"), r === null); v.focus(); return; }
    case "png_laden": if (spriteAktiv() && sp.docId !== null) await loadPng(); else toast(t("sprite-png-nichts"), true); return;
    case "exportieren": if (trackerAktiv() && tk.docId !== null) await openExport(tk.docId); else if (bildschirmAktiv() && bs.docId !== null) await openExport(bs.docId); else if (kartenAktiv() && kt.docId !== null) await openExport(kt.docId); else if (kachelAktiv() && kc.docId !== null) await openExport(kc.docId); else if (paletteAktiv() && pal.docId !== null) await openExport(pal.docId); else if (spriteAktiv() && sp.docId !== null) await openExport(sp.docId); else toast(t("export-nichts"), true); return;
    case "bauen": await buildRun(); return;
    case "bauen_aktiv": await buildRun({ aktiv: true }); return;
    case "starten": await buildRun({ then: "run" }); return;
    case "starten_aktiv": await buildRun({ aktiv: true, then: "run" }); return;
    case "debug_ende": if (debugAktiv()) await debugEnde(); else await runStop(); return;   // Emulator beenden (Debuggen selbst kommt mit E6); derselbe Weg wie Kopfzeile und Maschine-Panel
    case "protokoll_leeren": clearLog(); return;
    case "protokoll_kopieren": try { await navigator.clipboard.writeText(logText()); toast(t("output-kopiert")); } catch (e) { fail(e); } return;
    case "problem_springen": { const p = bau.sel ?? (bau.problems[0] ? { file: bau.problems[0].file, line: bau.problems[0].line, text: "" } : null); if (!p || p.line < 1) { toast(t("problems-none"), true); return; } void jumpTo(p.file, p.line - 1, 0); return; }
    case "problem_kopieren": { const p = bau.sel; if (!p) { toast(t("problems-keine-wahl"), true); return; } try { await navigator.clipboard.writeText(`${p.file}:${p.line}: ${p.text}`); toast(t("problems-kopiert")); } catch (e) { fail(e); } return; }
    case "probleme_alle_kopieren": { const l = problemLines(lsp.diags); if (!l.length) { toast(t("problems-none"), true); return; } try { await navigator.clipboard.writeText(l.join("\n")); toast(t("problems-kopiert")); } catch (e) { fail(e); } return; }
    case "panel": ui.panel = !ui.panel; return;
    case "schublade": ed.dock = ed.dock ? null : "problems"; return;
    case "tests_ausfuehren": await runTests("all"); return;
    case "korrigieren": { const v = getView(); if (v) await quickFix(v, lspHooks); return; }
    case "vervollstaendigen": inEditor(startCompletion); return;
    case "naechstes_problem": inEditor(nextDiagnostic); return;
    case "voriges_problem": inEditor(previousDiagnostic); return;
    case "haltepunkt_umschalten": { const v = getView(); if (v) toggleLineMark(v, "breakpoint"); return; }
    case "beenden": if (!(await confirmLeave())) return; await flushSession(); try { await call("app_quit"); } catch (e) { fail(e); } return;
    case "kopieren": case "ausschneiden": case "einfuegen": if (spriteAktiv()) { if (id === "einfuegen") await pasteAt(); else await copySel(id === "ausschneiden"); return; } await zwischenablage(id); return;
    case "basic_in_labels": void openConv("labels"); return;
    case "basic_in_nummern": void openConv("numbers"); return;
    case "basic_neu_nummerieren": void openConv("renumber"); return;
    default: toast(t("toast-not-yet", { name: t(`cmd-${id}`) }));
  }
}

/** Orte der Navigation für die Befehlspalette: Kennung, Beschriftungsschlüssel */
export function places(): { id: string; label: string }[] {
  const out: { id: string; label: string }[] = [];
  for (const m of visibleModes()) {
    out.push({ id: `nav:${m.id}`, label: m.label });
    for (const s of visibleSubs(m.id)) out.push({ id: `nav:${m.id}:${s.id}`, label: s.label });
  }
  return out;
}
