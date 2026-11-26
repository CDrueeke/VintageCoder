// SPDX-License-Identifier: GPL-3.0-or-later
// Rechtsklick (Auslieferungsbau): das Standardmenü der Webview (Neu laden, Autofill, …) erscheint nie; in Textfeldern kommt ein eigenes Menü
// (Ausschneiden, Kopieren, Einfügen, Alles auswählen), sonst nichts. Ein geöffnetes Menü schließt, sobald ein anderes aufgeht.
import { popupMenu } from "./cm-lsp";
import { t } from "./i18n.svelte";

type Feld = HTMLInputElement | HTMLTextAreaElement;
const textfeld = (el: Element | null): Feld | null => {
  const f = el?.closest("input, textarea") as Feld | null; if (!f) return null;
  if (f instanceof HTMLInputElement && !["text", "search", "number", "url", "email", "tel", "password", ""].includes(f.type)) return null;
  return f;
};

function einfuegen(f: Feld, text: string): void {
  f.focus(); const a = f.selectionStart ?? f.value.length, b = f.selectionEnd ?? a;
  f.setRangeText(text, a, b, "end"); f.dispatchEvent(new InputEvent("input", { bubbles: true, inputType: "insertFromPaste", data: text }));
}

export function startKontext(): void {
  window.addEventListener("contextmenu", (e) => {
    document.querySelectorAll(".vc-fixmenu").forEach((m) => m.remove());           // ein altes Menü weicht dem neuen
    window.dispatchEvent(new CustomEvent("vc:closemenus"));                         // auch die Menüs der Oberfläche (Reiter, Baum); das neue öffnet danach
    const el = e.target instanceof Element ? e.target : null;
    if (el?.closest(".cm-content")) return;                                         // der Editor hat sein eigenes Menü
    const f = textfeld(el);
    if (f) {
      e.preventDefault();
      const sel = (f.selectionEnd ?? 0) > (f.selectionStart ?? 0), nurLesen = f.readOnly || f.disabled;
      popupMenu(null, e.clientX, e.clientY, t("editor-kontextmenu"), [
        { label: t("cmd-ausschneiden"), disabled: !sel || nurLesen, run: () => { f.focus(); document.execCommand("cut"); } },
        { label: t("cmd-kopieren"), disabled: !sel, run: () => { f.focus(); document.execCommand("copy"); } },
        { label: t("cmd-einfuegen"), disabled: nurLesen, run: () => { navigator.clipboard.readText().then((x) => einfuegen(f, x), () => {}); } },
        { label: t("cmd-alle_auswaehlen"), run: () => { f.focus(); f.select(); } },
      ]);
      return;
    }
    if (!import.meta.env.DEV) e.preventDefault();                                   // sonst das Standardmenü der Webview unterdrücken (Entwicklung: Inspektor bleibt erreichbar)
  }, true);
}
