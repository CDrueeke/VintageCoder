// SPDX-License-Identifier: GPL-3.0-or-later
// Tastatur: Eingaben im Editor gehen nach `KeyboardEvent.code` (physische Taste; Lehre aus dem Spike: Y/Z, B/E/F, [ ]).
// Die Beschriftung richtet sich nach dem erkannten Layout (aus echten Tastenereignissen gelernt, sonst QWERTY).

export const isMac = typeof navigator !== "undefined" && /Mac|iPhone|iPad/.test(navigator.platform || navigator.userAgent);

/** Beschriftung je physischer Taste, gelernt aus Ereignissen mit Buchstaben/Ziffern (`code` → `key`) */
const learned = new Map<string, string>();

export function learnLayout(e: { code: string; key: string }): void {
  if (/^Key[A-Z]$/.test(e.code) && e.key.length === 1 && /\p{L}/u.test(e.key)) learned.set(e.code, e.key.toUpperCase());
}

export function codeLabel(code: string): string {
  const m = /^Key([A-Z])$/.exec(code);
  if (m) return learned.get(code) ?? m[1]!;
  const d = /^Digit(\d)$/.exec(code);
  if (d) return d[1]!;
  const names: Record<string, string> = {
    Space: "␣", Enter: "↩", Tab: "⇥", Escape: "Esc", Backspace: "⌫", Delete: "⌦", ArrowUp: "↑", ArrowDown: "↓", ArrowLeft: "←", ArrowRight: "→",
    Comma: ",", Period: ".", Slash: "/", Semicolon: ";", Quote: "'", BracketLeft: "[", BracketRight: "]", Backslash: "\\", Minus: "−", Equal: "=", Backquote: "`",
    PageUp: "PgUp", PageDown: "PgDn", Home: "Home", End: "End",
  };
  return names[code] ?? code;
}

const MODIFIER_CODES = new Set(["ShiftLeft", "ShiftRight", "ControlLeft", "ControlRight", "AltLeft", "AltRight", "MetaLeft", "MetaRight"]);

export interface Combo { code: string; cmd: boolean; shift: boolean; alt: boolean }

/** Kürzel aus einem Tastenereignis; nur Modifikatoren allein ergeben nichts */
export function comboFromEvent(e: { code: string; metaKey: boolean; ctrlKey: boolean; shiftKey: boolean; altKey: boolean }, mac = isMac): Combo | null {
  if (MODIFIER_CODES.has(e.code)) return null;
  return { code: e.code, cmd: mac ? e.metaKey : e.ctrlKey, shift: e.shiftKey, alt: e.altKey };
}

/** Anzeigeteile eines Kürzels (`["⌘", "⇧", "S"]` auf dem Mac, `["Strg", "Umschalt", "S"]` sonst) */
export function comboParts(c: Combo, names: { ctrl: string; shift: string; alt: string }, mac = isMac): string[] {
  const p: string[] = [];
  if (mac) { if (c.alt) p.push("⌥"); if (c.shift) p.push("⇧"); if (c.cmd) p.push("⌘"); }
  else { if (c.cmd) p.push(names.ctrl); if (c.alt) p.push(names.alt); if (c.shift) p.push(names.shift); }
  p.push(codeLabel(c.code));
  return p;
}

/** Kürzel der Befehlstabelle („Cmd+Umschalt+S“, „F7“, „Cmd+Komma“) in Teile zerlegen */
export function parseSpec(spec: string): { cmd: boolean; shift: boolean; alt: boolean; key: string } | null {
  if (!spec) return null;
  const parts = spec.split("+");
  const key = parts.pop()!;
  return { cmd: parts.includes("Cmd"), shift: parts.includes("Umschalt"), alt: parts.includes("Alt"), key };
}

const SPEC_KEYS: Record<string, string> = { Plus: "Equal", Minus: "Minus", Komma: "Comma", Rechts: "ArrowRight", Links: "ArrowLeft", Enter: "Enter" };

/** Passt das Tastenereignis auf ein Kürzel der Befehlstabelle? (nur für den Betrieb ohne natives Menü, z. B. im Browser) */
export function matchesSpec(e: { code: string; metaKey: boolean; ctrlKey: boolean; shiftKey: boolean; altKey: boolean }, spec: string, mac = isMac): boolean {
  const s = parseSpec(spec);
  const c = comboFromEvent(e, mac);
  if (!s || !c) return false;
  const want = SPEC_KEYS[s.key] ?? (/^[A-Z]$/.test(s.key) ? `Key${s.key}` : /^\d$/.test(s.key) ? `Digit${s.key}` : s.key);
  return c.cmd === s.cmd && c.shift === s.shift && c.alt === s.alt && c.code === want;
}
