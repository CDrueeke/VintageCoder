// SPDX-License-Identifier: GPL-3.0-or-later
// Text aus dem Katalog der Prüfsprache (VC_E2E_LANG = de | en, Vorgabe de) für Ablauftests: so suchen Tests Beschriftungen nicht mehr mit festem deutschem Text.
import { readdirSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
export const LANG = process.env.VC_E2E_LANG === "en" ? "en" : "de";
const dir = resolve(import.meta.dirname, `../../crates/vc-i18n/catalog/${LANG}`);
const KAT = {};
for (const f of readdirSync(dir).filter((x) => x.endsWith(".ftl"))) {
  let cur = null;
  for (const z of readFileSync(resolve(dir, f), "utf8").split(/\r?\n/)) {
    const m = z.match(/^([a-z0-9_.-]+)\s*=\s*(.*)$/);
    if (m) { cur = m[1]; KAT[cur] = m[2]; } else if (cur && z.startsWith(" ")) KAT[cur] += " " + z.trim();
  }
}
/** Katalogtext zu `key`; `{ $name }` wird durch args.name ersetzt */
export const T = (key, args = {}) => (KAT[key] ?? `⟦${key}⟧`).replace(/\{\s*\$(\w+)\s*\}/g, (_, n) => String(args[n] ?? ""));
/** Regulärer Ausdruck, der den Text trifft (Sonderzeichen maskiert) */
export const R = (key, args) => new RegExp(T(key, args).replace(/[.*+?^${}()|[\]\\]/g, "\\$&"), "i");   // Groß-/Kleinschreibung egal (Überschriften erscheinen großgeschrieben)
