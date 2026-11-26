// SPDX-License-Identifier: GPL-3.0-or-later
// Kopieren, Ausschneiden, Einfügen als Befehle (Menü und Befehlspalette) außerhalb des Sprite-Editors: im Code-Editor und in Eingabefeldern.
// Die Tastenkürzel (Cmd/Strg+C, X, V) erledigt das Fenster selbst; hier geht es um den Aufruf über die Palette.
import { getView } from "./editor.svelte";
import { toast } from "./state.svelte";
import { t } from "./i18n.svelte";

export async function zwischenablage(was: "kopieren" | "ausschneiden" | "einfuegen"): Promise<void> {
  const v = getView(); const el = document.activeElement as HTMLElement | null;
  const imFeld = !!el && (el instanceof HTMLInputElement || el instanceof HTMLTextAreaElement);
  if (!imFeld && v) v.focus();
  if (was !== "einfuegen") {
    const ok = document.execCommand(was === "kopieren" ? "copy" : "cut");
    if (!ok) toast(t("zwischenablage-nichts-gewaehlt"), true);
    return;
  }
  try {
    const text = await navigator.clipboard.readText();
    if (imFeld) { const f = el as HTMLInputElement; f.setRangeText(text, f.selectionStart ?? 0, f.selectionEnd ?? 0, "end"); f.dispatchEvent(new Event("input", { bubbles: true })); }
    else if (v) v.dispatch(v.state.replaceSelection(text), { scrollIntoView: true, userEvent: "input.paste" });
    else toast(t("zwischenablage-kein-ziel"), true);
  } catch { toast(t("zwischenablage-einfuegen-hinweis"), true); }
}
