// SPDX-License-Identifier: GPL-3.0-or-later
// „Als Include einfügen“ (Projektbaum): schreibt `.include "<Pfad relativ zur Quelldatei>"` in den Quelltext des aktiven Assembler-Reiters, vor die Zeile mit der Schreibmarke.
import { ed, getView } from "./editor.svelte";
import { toast } from "./state.svelte";
import { t } from "./i18n.svelte";

/** Endungen, die sich nicht in Quelltext einbinden lassen (Projektdateien und Diskettenabbilder) */
const NICHT = [".vcprj", ".d64", ".d71", ".d81", ".atr", ".prg", ".crt"];

/** Pfad von der Datei `von` (Projektpfad) zur Datei `nach`, mit `/` getrennt */
export function relativ(von: string, nach: string): string {
  const a = von.split("/").slice(0, -1), b = nach.split("/"); let i = 0;
  while (i < a.length && i < b.length - 1 && a[i] === b[i]) i++;
  return [...a.slice(i).map(() => ".."), ...b.slice(i)].join("/");
}

/** Meldungstext, warum „Als Include einfügen“ gerade nicht geht (null = geht), für den ausgegrauten Menüpunkt */
export function includeGrund(rel: string, istDatei: boolean): string | null {
  if (!istDatei) return t("include-keine-datei");
  if (NICHT.some((e) => rel.toLowerCase().endsWith(e))) return t("include-nicht-moeglich", { name: rel.split("/").pop() ?? rel });
  const tab = ed.tabs.find((x) => x.rel === ed.active);
  if (!tab || tab.scratch || tab.lang !== "asm") return t("include-kein-asm");
  return null;
}

export function alsInclude(rel: string | null | undefined, istDatei: boolean): void {
  if (!rel || !istDatei) { toast(t("include-keine-datei"), true); return; }
  if (NICHT.some((e) => rel.toLowerCase().endsWith(e))) { toast(t("include-nicht-moeglich", { name: rel.split("/").pop() ?? rel }), true); return; }
  const tab = ed.tabs.find((x) => x.rel === ed.active); const v = getView();
  if (!tab || tab.scratch || tab.lang !== "asm" || !v) { toast(t("include-kein-asm"), true); return; }
  const pfad = relativ(tab.rel, rel); if (pfad === tab.rel.split("/").pop() && rel === tab.rel) { toast(t("include-nicht-selbst"), true); return; }
  const text = v.state.doc.toString();
  if (new RegExp(`^\\s*\\.include\\s+"${pfad.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}"`, "mi").test(text)) { toast(t("include-schon-da", { name: pfad })); return; }
  const zeile = `.include "${pfad}"`; const l = v.state.doc.lineAt(v.state.selection.main.head);
  v.dispatch({ changes: { from: l.from, insert: zeile + "\n" }, selection: { anchor: l.from + zeile.length + 1 }, scrollIntoView: true, userEvent: "input" }); v.focus();
  toast(t("include-eingefuegt-zeile", { zeile }));
}
