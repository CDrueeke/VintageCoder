// SPDX-License-Identifier: GPL-3.0-or-later
// Rückfrage mit „Abbrechen“ als Vorgabe (Fokus und Esc): für Handlungen, die der Kern nur nach ausdrücklichem Ja ausführt (endgültig löschen, Programm öffnen).
export const rf = $state({ open: false, title: "", text: "", ok: "", pending: null as null | ((ja: boolean) => void) });
export function fragen(title: string, text: string, ok: string): Promise<boolean> {
  if (rf.pending) rf.pending(false);
  return new Promise((resolve) => { Object.assign(rf, { open: true, title, text, ok, pending: resolve }); });
}
export function antwort(ja: boolean): void { const p = rf.pending; rf.pending = null; rf.open = false; p?.(ja); }
