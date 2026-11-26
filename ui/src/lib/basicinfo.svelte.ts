// SPDX-License-Identifier: GPL-3.0-or-later
// Sprachdienste für natives BASIC: Dialekt der Datei (Statuszeile), Gliederung (Marken und #define), durch `#if` ausgeschlossene Zeilen (grau).
// Das Fachliche rechnet der Kern (`vc_ide::basicinfo`, Befehl `basic_info`); hier steht nur der Anzeigezustand je Reiter.
import type { BasicInfo } from "./api";

/** Letzte Auskunft des Kerns je Reiter (Pfad → Info); leer für Nicht-BASIC-Reiter */
export const bi = $state<{ byRel: Record<string, BasicInfo> }>({ byRel: {} });
export const setBasicInfo = (rel: string, info: BasicInfo | null): void => { if (info) bi.byRel[rel] = info; else delete bi.byRel[rel]; };
