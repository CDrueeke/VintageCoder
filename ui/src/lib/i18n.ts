// SPDX-License-Identifier: GPL-3.0-or-later
// Umschlag um @fluent/bundle (Konzept 6.2): Der Kern liefert die `.ftl`-Quellen, hier wird lokal formatiert.
// Rückfall: gewählte Sprache → Deutsch → sichtbarer Hinweis `⟦schlüssel⟧` (nie ein Absturz). Pseudo-Sprache `xx` entsteht aus dem Deutschen.
import { FluentBundle, FluentResource, type FluentVariable } from "@fluent/bundle";
import { guard, pseudo } from "./pseudo";
import type { LangCode } from "./api";

export type Args = Record<string, string | number>;

function bundle(locale: string, src: string): FluentBundle {
  const b = new FluentBundle(locale, { useIsolating: false });
  const errors = b.addResource(new FluentResource(src), { allowOverrides: true });
  if (errors.length > 0 && typeof console !== "undefined") console.warn(`i18n: ${errors.length} Fehler beim Lesen des Katalogs ${locale}`);
  return b;
}

export function missingText(key: string): string { return `⟦${key}⟧`; }

export class Translator {
  readonly lang: LangCode;
  private primary: FluentBundle | null;
  private de: FluentBundle;
  /** Schlüssel, die in keinem Katalog standen (für Anzeige und Prüfung) */
  readonly missing = new Set<string>();

  constructor(lang: LangCode, deSrc: string, enSrc: string | null) {
    this.lang = lang;
    this.de = bundle("de", deSrc);
    this.primary = lang === "en" && enSrc !== null ? bundle("en", enSrc) : null;
  }

  has(key: string): boolean { return (this.primary ?? this.de).hasMessage(key) || this.de.hasMessage(key); }

  t(key: string, args?: Args): string {
    const pseudoMode = this.lang === "xx";
    const a: Record<string, FluentVariable> = {};
    for (const [k, v] of Object.entries(args ?? {})) a[k] = pseudoMode && typeof v === "string" ? guard(v) : v;
    const order = this.primary ? [this.primary, this.de] : [this.de];
    for (const b of order) {
      const m = b.getMessage(key);
      if (m?.value) {
        const s = b.formatPattern(m.value, a);
        return pseudoMode ? pseudo(s) : s;
      }
    }
    this.missing.add(key);
    return missingText(key);
  }
}
