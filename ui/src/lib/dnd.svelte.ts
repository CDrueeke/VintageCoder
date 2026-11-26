// SPDX-License-Identifier: GPL-3.0-or-later
// Ziehen innerhalb des Fensters mit Zeigerereignissen statt HTML5-Ziehen. Grund: Damit Dateien aus dem Dateimanager in das Fenster gezogen werden können,
// muss Tauris natives Drop-Ereignis eingeschaltet sein; es ersetzt das HTML5-Ziehen im Fenster (auf allen Systemen). Die Quellen (`dragSource`) und
// Ziele (`dropTarget`) hier arbeiten mit `pointerdown/move/up`, haben dieselbe Wirkung und laufen auch in den Ablauftests mit echten Mausfolgen.

export interface DragPayload { kind: string; id: string }
export const dnd = $state({ active: false, payload: null as DragPayload | null, x: 0, y: 0, label: "" });

interface Source { kind: string; id: string; label?: string; onstart?: () => void; onend?: () => void }
interface Target { accept: (p: DragPayload) => boolean; enter?: () => void; leave?: () => void; drop: (p: DragPayload) => void }
const targets = new WeakMap<Element, () => Target>();

let cur: Element | null = null;   // Ziel unter dem Zeiger
const THRESHOLD = 5;

/** Element als Ziehquelle: erst nach 5 Pixel Bewegung beginnt das Ziehen (ein Klick bleibt ein Klick) */
export function dragSource(node: HTMLElement, getOpts: () => Source): void {
  let down: { x: number; y: number; id: number } | null = null; let src: Source | null = null;
  const move = (e: PointerEvent) => {
    if (!down) return;
    if (!dnd.active) { if (Math.hypot(e.clientX - down.x, e.clientY - down.y) < THRESHOLD) return; src = getOpts(); dnd.active = true; dnd.payload = { kind: src.kind, id: src.id }; dnd.label = src.label ?? ""; document.body.classList.add("vc-dragging"); src.onstart?.(); }
    dnd.x = e.clientX; dnd.y = e.clientY; hit(e.clientX, e.clientY);
  };
  const finish = (cancel: boolean) => {
    document.removeEventListener("pointermove", move, true); document.removeEventListener("pointerup", up, true); document.removeEventListener("pointercancel", cancelUp, true); document.removeEventListener("keydown", esc, true);
    const was = dnd.active; const p = dnd.payload; const t = cur ? targets.get(cur)?.() : null;
    if (was) {
      if (!cancel && p && t && t.accept(p)) t.drop(p);
      if (cur) targets.get(cur)?.().leave?.();
      cur = null; dnd.active = false; dnd.payload = null; document.body.classList.remove("vc-dragging"); src?.onend?.();
      const swallow = (ev: Event) => { ev.stopPropagation(); ev.preventDefault(); }; document.addEventListener("click", swallow, { capture: true, once: true }); setTimeout(() => document.removeEventListener("click", swallow, true), 0);   // das Loslassen ist kein Klick
    }
    down = null; src = null;
  };
  const up = () => finish(false); const cancelUp = () => finish(true);
  const esc = (e: KeyboardEvent) => { if (e.key === "Escape") finish(true); };
  node.addEventListener("pointerdown", (e) => {
    const blocked = (e.target as HTMLElement).closest("input, select, textarea, [data-nodrag]");
    if (e.button !== 0 || (blocked && blocked !== node)) return;   // Eingabefelder und ausgenommene Knöpfe (z. B. Schließen im Reiter) starten kein Ziehen
    down = { x: e.clientX, y: e.clientY, id: e.pointerId };
    document.addEventListener("pointermove", move, true); document.addEventListener("pointerup", up, true); document.addEventListener("pointercancel", cancelUp, true); document.addEventListener("keydown", esc, true);
  });
}

/** Element als Ziel: `accept` sagt, ob die gezogene Sache hier abgelegt werden darf (das innerste annehmende Ziel gewinnt) */
export function dropTarget(node: HTMLElement, getOpts: () => Target): void { targets.set(node, getOpts); }

function hit(x: number, y: number): void {
  const p = dnd.payload; let found: Element | null = null;
  if (p) for (let el = document.elementFromPoint(x, y); el; el = el.parentElement) { const g = targets.get(el); if (g && g().accept(p)) { found = el; break; } }
  if (found === cur) return;
  if (cur) targets.get(cur)?.().leave?.();
  cur = found; if (cur) targets.get(cur)?.().enter?.();
}
