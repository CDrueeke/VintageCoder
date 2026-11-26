// SPDX-License-Identifier: GPL-3.0-or-later
// Absprung aus dem Code-Editor auf den Pfad einer eingebundenen Datei (Konzept Include-von-Medien 4.5): Doppelklick, Kontextmenü, Cmd/Strg+Klick.
// Trefferfläche ist nur der Pfadstring in einer Include-Zeile (`.include`, `.incbin`, `#include`, `#asm … with`); der Kern löst den Pfad auf.
import type { EditorView } from "@codemirror/view";

const ZEILE = /(?:\.include|\.incbin|#include|#asm\b[^"\n]*?\bwith)\s+"([^"\n]+)"/gi;

/** Pfad der Include-Zeile, auf dem `pos` liegt (Anführungszeichen eingeschlossen), sonst null */
export function includePfadAn(view: EditorView, pos: number): string | null {
  const l = view.state.doc.lineAt(pos); const off = pos - l.from;
  for (const m of l.text.matchAll(ZEILE)) {
    const pfad = m[1] ?? ""; const ende = (m.index ?? 0) + m[0].length, start = ende - pfad.length - 2;
    if (off >= start && off <= ende) return pfad;
  }
  return null;
}
