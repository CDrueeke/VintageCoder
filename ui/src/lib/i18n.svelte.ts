// SPDX-License-Identifier: GPL-3.0-or-later
// Reaktive Hülle um den Übersetzer: `t()` in Svelte-Ausdrücken wird beim Sprachwechsel von selbst neu berechnet (ohne Neustart).
import { call, type LangCode } from "./api";
import { Translator, missingText, type Args } from "./i18n";

const state = $state<{ rev: number; lang: LangCode }>({ rev: 0, lang: "de" });
let translator: Translator | null = null;

/** Text zu `key`; liest `state.rev`, damit Svelte beim Sprachwechsel neu zeichnet */
export function t(key: string, args?: Args): string {
  void state.rev;
  return translator ? translator.t(key, args) : missingText(key);
}

export function currentLang(): LangCode { void state.rev; return state.lang; }

/** Sprache laden (Quellen vom Kern holen) und alle Texte der Oberfläche umschalten */
const langHooks: (() => void | Promise<void>)[] = [];
/** Modul meldet an: nach einem Sprachwechsel (nicht beim ersten Laden) vom Kern fertig gelieferte Texte neu holen */
export function onLangChange(fn: () => void | Promise<void>): void { langHooks.push(fn); }

export async function loadLanguage(lang: LangCode): Promise<void> {
  const src = await call("i18n_ftl", { lang });
  const wechsel = translator !== null;
  translator = new Translator(src.lang, src.de, src.en);
  state.lang = src.lang;
  state.rev++;
  if (typeof document !== "undefined") {   // Sprache der Seite für Sprachausgabe (VoiceOver, Orca, NVDA) und Silbentrennung
    const l = src.lang === "xx" ? "de" : src.lang; document.documentElement.lang = l;
    const m = document.querySelector<HTMLMetaElement>('meta[http-equiv="content-language"]'); if (m) m.content = l;
  }
  void meldeDom(src.lang);
  if (wechsel) for (const f of langHooks) { try { void f(); } catch { /* ein Modul darf den Wechsel nicht stören */ } }
}

/** Fehler des Kerns als Text (`err-<code>`, sonst allgemeiner Rückfall) */
export function errorText(code: string, args: Record<string, unknown> = {}): string {
  const flat: Args = {};
  for (const [k, v] of Object.entries(args)) flat[k] = typeof v === "number" ? v : String(v);
  if (translator?.has(`err-${code}`)) return t(`err-${code}`, flat);
  return t("err-unbekannt", { code });
}

export function missingKeys(): string[] { return translator ? [...translator.missing] : []; }

/** Meldung des Kerns (`{key, args}`, I2) als Text: verschachtelte Meldungen werden zuerst formatiert; fehlt der Schlüssel, gilt `fallback` (Text des Kerns) */
export function msgText(key: string, args: Record<string, unknown> | undefined, fallback: string): string {
  void state.rev;
  if (!translator || !key || !translator.has(key)) return fallback;
  const flat: Args = {};
  for (const [k, v] of Object.entries(args ?? {})) {
    if (v !== null && typeof v === "object") { const o = v as { key?: string; args?: Record<string, unknown> }; flat[k] = typeof o.key === "string" ? msgText(o.key, o.args, o.key) : JSON.stringify(v); }
    else flat[k] = typeof v === "number" ? v : String(v ?? "");
  }
  return translator.t(key, flat);
}

/** Prüfhilfe: meldet dem Kern (nur wirksam mit VC_DOM_DUMP), was die Webview als Sprache der Seite sieht – lang, Meta, contentType, Systemsprachen der Webview, Elemente mit eigenem lang */
async function meldeDom(lang: string): Promise<void> {
  if (typeof document === "undefined") return;
  try {
    const el = (s: string) => document.querySelector(s)?.getAttribute("lang") ?? null;
    const info = { app: lang, htmlLang: document.documentElement.lang, meta: document.querySelector('meta[http-equiv="content-language"]')?.getAttribute("content") ?? null, contentType: document.contentType,
      navigator: navigator.language, navigatorLanguages: [...navigator.languages], cmContent: el(".cm-content"), mitLang: [...document.querySelectorAll("[lang]")].map((e) => `${e.tagName.toLowerCase()}[${e.getAttribute("lang")}]`).slice(0, 20) };
    await call("dom_info", { info });
  } catch { /* nur Diagnose */ }
}
