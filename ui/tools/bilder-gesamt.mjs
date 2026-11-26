// SPDX-License-Identifier: GPL-3.0-or-later
// Bilderstrecke Gesamtpaket (neu seit dem Einzelpaket Startseite): Willkommenskarte, Startbild, schmale Navigation, Tasten-/Startbild-Einstellungen, Assistent beim Erststart.
// Aufruf: node tools/bilder-gesamt.mjs <Ausgabe> [assistent]  (Prüfbrücke Port 17420, Vite 1420; „assistent“: Kern ohne VC_NO_WIZARD gestartet)
import { webkit } from "playwright";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
const [out, modus] = process.argv.slice(2);
if (!out) { console.error("Aufruf: node tools/bilder-gesamt.mjs <Ausgabe> [assistent]"); process.exit(2); }
mkdirSync(out, { recursive: true });
const browser = await webkit.launch(); const page = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
const merk = join(out, ".liste.json"); const alt = existsSync(merk) ? JSON.parse(readFileSync(merk, "utf8")) : []; const liste = []; const w = (ms = 500) => page.waitForTimeout(ms);
const bild = async (name, punkt, soll, voll = false) => { await w(500); await page.screenshot({ path: join(out, name + ".png"), fullPage: voll }); liste.push([punkt, name + ".png", soll]); };
const V = (js, x) => page.evaluate(js, x);
const laden = async (q = "") => { await page.goto("http://127.0.0.1:1420/?bridge=17420" + q); await page.waitForFunction(() => window.__vc && window.__vc.ui.ready); await V(async () => { await window.__vc.setLanguage("de"); window.__vc.setTheme("dark"); }); await w(700); };
if (modus === "assistent") {
  await laden(); await w(800);
  await bild("g-01-assistent-erststart", "A-1", "Erster Start ohne Einstellungen: der Einrichtungsassistent öffnet sich von selbst; dahinter keine zweite Karte");
  const spaeter = page.getByRole("button", { name: /Später fortsetzen/ }); if (await spaeter.count()) { await spaeter.click(); await w(800); }
  await page.locator("[data-start=welcome-card]").scrollIntoViewIfNeeded().catch(() => {}); await w(300);
  await bild("g-02-nach-spaeter-willkommen", "A-3", "Nach „Später“: Startseite mit Willkommenskarte als Einstieg (statt der Werkzeugliste), unten links");
} else {
  await page.goto("http://127.0.0.1:1420/?bridge=17420&splash=1"); await page.waitForSelector("[data-splash]"); await w(150);
  await page.screenshot({ path: join(out, "g-03-startbild.png") }); liste.push(["A-5", "g-03-startbild.png", "Startbild beim Programmstart (etwa 1 s, Klick oder Taste schließt)"]);
  if (false) await bild("g-03-startbild", "A-5", "Startbild beim Programmstart (etwa 1 s, Klick oder Taste schließt)");
  await w(1800); await laden();
  await bild("g-04-startseite-breit", "A-6", "Startseite 1440 px: „Projekt“ in der Hauptnavigation hervorgehoben");
  await page.setViewportSize({ width: 900, height: 800 }); await bild("g-05-schmal-900", "A-6", "900 px: Navigation mit Symbolen, nur das aktive Ziel mit Text, Kopfzeile ohne Überlauf");
  await page.setViewportSize({ width: 640, height: 800 }); await bild("g-06-schmal-640", "A-6", "640 px: wie 900, Karten untereinander", true);
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.locator("[data-act=einstellungen]").click(); await w(700);
  await page.locator("[data-es=pagebtn][data-s=allgemein]").click(); await bild("g-07-einst-allgemein-startbild", "A-5", "Einstellungen ▸ Allgemein: Schalter „Startbild beim Programmstart zeigen“");
  await page.locator("[data-es=pagebtn][data-s=kuerzel]").click(); await bild("g-08-einst-tasten", "C-20", "Einstellungen ▸ Tastenkürzel: Einzeltasten der Editoren und Tastenbeschriftung der Klaviatur");
  await V(() => window.__vc.setTheme("light")); await bild("g-09-einst-tasten-hell", "H-42", "Dasselbe hell");
  await V(() => window.__vc.setTheme("contrast")); await bild("g-10-einst-tasten-kontrast", "H-42", "Dasselbe im hohen Kontrast");
  await V(() => window.__vc.setTheme("dark"));
}
const neu = new Set(liste.map((l) => l[1])); const alle = [...alt.filter((l) => !neu.has(l[1])), ...liste].sort((a, b) => a[1].localeCompare(b[1])); writeFileSync(merk, JSON.stringify(alle));
writeFileSync(join(out, "LIESMICH.md"), "# Bilderstrecke Gesamtpaket (neu: Erststart, Willkommenskarte, Startbild, Navigation, Tasten)\n\nErzeugt mit `node ui/tools/bilder-gesamt.mjs` (Playwright/WebKit, Prüfbrücke, echter Kern, Deutsch, dunkel wenn nicht anders genannt). Die übrigen Bilder liegen in den Einzelpaketen unter `abnahme-ui/*/bilder`.\n\n| Punkt (RUNDGANG.md) | Bild | Erwartet |\n|---|---|---|\n" + alle.map((l) => `| ${l[0]} | ${l[1]} | ${l[2]} |`).join("\n") + "\n");
console.log(liste.length + " Bilder"); await browser.close();
