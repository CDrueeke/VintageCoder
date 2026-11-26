// SPDX-License-Identifier: GPL-3.0-or-later
// Pseudo-Sprache `xx`: dieselbe Verfremdung wie `crates/vc-i18n/src/pseudo.rs` (Buchstaben mit Akzent, etwa 40 % Füllung, Klammern […]).
// Eingesetzte Parameterwerte bleiben unverändert (Schutzzeichen), damit feste Texte im Code auffallen und Felder auf Überlauf geprüft werden.
export const OPEN = "";
export const CLOSE = "";

const FROM = "abcdefghijklmnopqrstuvwxyzäöüßABCDEFGHIJKLMNOPQRSTUVWXYZÄÖÜ";
const TO = "áƀçðéƒĝĥíĵķļɱñóþǫŕšţúṽŵẋýžæœůƞÁƁÇÐÉƑĜĤÍĴĶĻṀÑÓÞǪŔŠŢÚṼŴẊÝŽÆŒŮ";
const MAP = new Map<string, string>([...FROM].map((c, i) => [c, [...TO][i] as string]));

export function guard(v: string): string { return `${OPEN}${v}${CLOSE}`; }

export function pseudo(s: string): string {
  let out = "[";
  let guarded = false;
  let letters = 0;
  for (const c of s) {
    if (c === OPEN) { guarded = true; continue; }
    if (c === CLOSE) { guarded = false; continue; }
    if (guarded) { out += c; continue; }
    if (/\p{L}/u.test(c)) letters++;
    out += MAP.get(c) ?? c;
  }
  return out + "~".repeat(Math.ceil((letters * 2) / 5)) + "]";
}
