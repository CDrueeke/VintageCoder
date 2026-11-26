// SPDX-License-Identifier: GPL-3.0-or-later
// Navigation (Konzept Anhang A): fünf Hauptbereiche, zweite Reihe bei Grafik und Daten. `stage` ist die Etappe, in der der Bereich gebaut wird.
export type Mode = "projekt" | "code" | "grafik" | "musik" | "daten" | "debug";

export interface Section { id: string; label: string; title: string; stage: string; /** Editor-Kennung der Release-Auswahl (`sprite`, `tracker` …): ohne diesen Editor im Release fehlt der Eintrag */ feature?: string; /** noch nicht Teil dieser Version: kein Eintrag in der Navigation (der Bereich steht in einer späteren Version) */ spaeter?: boolean }
export interface ModeDef extends Section { id: Mode; icon: string }

export const MODES: ModeDef[] = [
  { id: "projekt", icon: "folder", label: "nav-main-projekt", title: "nav-main-projekt", stage: "P1" },
  { id: "code", icon: "code", label: "nav-main-code", title: "view-title-code", stage: "E1.1" },
  { id: "grafik", icon: "sprite", label: "nav-main-grafik", title: "nav-main-grafik", stage: "E3" },
  { id: "musik", icon: "music", label: "nav-main-musik", title: "view-title-musik", stage: "E5", feature: "tracker" },
  { id: "daten", icon: "memory", label: "nav-main-daten", title: "nav-main-daten", stage: "E2" },
  { id: "debug", icon: "bug", label: "nav-main-debug", title: "view-title-debug", stage: "E6" },
];

export const SUBS: Partial<Record<Mode, Section[]>> = {
  projekt: [
    { id: "uebersicht", label: "nav-projekt-uebersicht", title: "view-title-projekt-uebersicht", stage: "P2" },
    { id: "allgemein", label: "nav-projekt-allgemein", title: "view-title-projekt-allgemein", stage: "P2" },
    { id: "ziele", label: "nav-projekt-ziele", title: "view-title-projekt-ziele", stage: "P2" },
    { id: "quellen", label: "nav-projekt-quellen", title: "view-title-projekt-quellen", stage: "P3" },
    { id: "bauen", label: "nav-projekt-bauen", title: "view-title-projekt-bauen", stage: "P3" },
    { id: "diskette", label: "nav-projekt-diskette", title: "view-title-projekt-diskette", stage: "P3" },
    { id: "weitergeben", label: "nav-projekt-weitergeben", title: "view-title-projekt-weitergeben", stage: "P3" },
  ],
  grafik: [
    { id: "sprites", label: "nav-grafik-sprites", title: "view-title-sprites", stage: "E3", feature: "sprite" },
    { id: "zeichensatz", label: "nav-grafik-zeichensatz", title: "view-title-zeichensatz", stage: "E4", feature: "char" },
    { id: "kacheln", label: "nav-grafik-kacheln", title: "view-title-kacheln", stage: "E8", feature: "tiles" },
    { id: "bildschirm", label: "nav-grafik-bildschirm", title: "view-title-bildschirm", stage: "E9", feature: "screen" },
    { id: "karten", label: "nav-grafik-karten", title: "view-title-karten", stage: "E10", feature: "tiles" },
    { id: "palette", label: "nav-grafik-palette", title: "view-title-palette", stage: "E7", feature: "palette" },
  ],
  daten: [
    { id: "disk", label: "nav-daten-disk", title: "view-title-disk", stage: "E2", feature: "disk" },
    { id: "hex", label: "nav-daten-hex", title: "view-title-hex", stage: "E11" },
    { id: "speicher", label: "nav-daten-speicher", title: "view-title-speicher", stage: "E11", feature: "memmap" },
  ],
};

export function section(mode: Mode, sub: string | undefined): Section {
  const m = MODES.find((x) => x.id === mode)!;
  return SUBS[mode]?.find((s) => s.id === sub) ?? m;
}
