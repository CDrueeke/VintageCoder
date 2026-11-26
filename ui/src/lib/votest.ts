// SPDX-License-Identifier: GPL-3.0-or-later
// Minimalbeispiel für die Prüfung mit VoiceOver, unabhängig von der Oberfläche: 256 Zeichenzellen mit Roving Focus in drei Bauweisen.
// Aufruf: `?vo-test=a` (Auswahlliste mit Optionen), `?vo-test=b` (Gruppe mit Knöpfen), `?vo-test=c` (nackte Knöpfe ohne Behälter-Rolle). Pfeiltasten wechseln die Zelle.
// Zweck: klären, ob das Präfix „Fenster …“ beim Fokuswechsel vom System kommt oder von unserer Bauweise.
export function voTest(variant: string): void {
  document.title = `VoiceOver-Test ${variant}`;
  const root = document.getElementById("app") ?? document.body; root.textContent = "";
  const h = document.createElement("h1"); h.textContent = `VoiceOver-Test ${variant.toUpperCase()}: Pfeiltasten wechseln die Zelle`; h.style.cssText = "font:600 16px system-ui;margin:12px";
  const nav = document.createElement("p"); nav.style.margin = "12px"; for (const v of ["a", "b", "c"]) { const l = document.createElement("a"); l.href = `?vo-test=${v}`; l.textContent = `Bauweise ${v.toUpperCase()}`; l.style.marginRight = "12px"; nav.append(l); }
  const box = document.createElement("div"); box.style.cssText = "display:grid;grid-template-columns:repeat(16,28px);gap:2px;margin:12px;width:max-content";
  const hex = (i: number) => "$" + i.toString(16).toUpperCase().padStart(2, "0");
  const cells: HTMLElement[] = []; let cur = 0;
  if (variant === "a") { box.setAttribute("role", "listbox"); box.setAttribute("aria-label", "Zeichen"); } else if (variant === "b") { box.setAttribute("role", "group"); box.setAttribute("aria-label", "Zeichen"); }
  for (let i = 0; i < 256; i++) {
    const c = document.createElement(variant === "a" ? "div" : "button"); c.textContent = String.fromCharCode(i >= 32 && i < 127 ? i : 46); c.style.cssText = "width:28px;height:28px;font:12px monospace;border:1px solid #888;background:#fff;color:#000;padding:0";
    c.setAttribute("aria-label", `Zeichen ${i}, ${hex(i)}`); c.tabIndex = i === 0 ? 0 : -1;
    if (variant === "a") { c.setAttribute("role", "option"); c.setAttribute("aria-selected", i === 0 ? "true" : "false"); c.setAttribute("aria-posinset", String(i + 1)); c.setAttribute("aria-setsize", "256"); }
    cells.push(c); box.append(c);
  }
  box.addEventListener("keydown", (e) => {
    const d = e.key === "ArrowRight" ? 1 : e.key === "ArrowLeft" ? -1 : e.key === "ArrowDown" ? 16 : e.key === "ArrowUp" ? -16 : 0; if (!d) return; e.preventDefault();
    const n = Math.max(0, Math.min(255, cur + d)); if (n === cur) return;
    cells[cur]!.tabIndex = -1; cells[cur]!.setAttribute("aria-selected", "false"); cur = n; cells[n]!.tabIndex = 0; if (variant === "a") cells[n]!.setAttribute("aria-selected", "true"); cells[n]!.focus({ preventScroll: true });
  });
  root.append(h, nav, box);
}
