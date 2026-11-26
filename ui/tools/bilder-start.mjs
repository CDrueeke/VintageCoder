// SPDX-License-Identifier: GPL-3.0-or-later
// Bilderstrecke für die Abnahme (Startseite): Playwright (WebKit), Prüfbrücke. Aufruf: node tools/bilder-start.mjs <Kopie c64/jumprun> <Kopie c64/music_demo> <Bild.png für das Projektbild> <Ausgabe>
import { webkit } from "playwright";
import { copyFileSync, mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
const [a, b, bild_png, out] = process.argv.slice(2);
if (!out) { console.error("Aufruf: node tools/bilder-start.mjs <jumprun> <music_demo> <Bild.png> <Ausgabe>"); process.exit(2); }
mkdirSync(out, { recursive: true }); copyFileSync(bild_png, join(a, "projektbild.png"));
const browser = await webkit.launch(); const page = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
const liste = []; const bild = async (name, punkt, soll, voll = false) => { await page.waitForTimeout(500); await page.screenshot({ path: join(out, name + ".png"), fullPage: voll }); liste.push([punkt, name + ".png", soll]); };
const V = (js, x) => page.evaluate(js, x); const w = (ms = 500) => page.waitForTimeout(ms);
await page.goto("http://127.0.0.1:1420/?bridge=17420"); await page.waitForFunction(() => window.__vc && window.__vc.ui.ready);
await V(async () => { await window.__vc.setLanguage("de"); window.__vc.setTheme("dark"); }); await w(700);
await bild("start-01-leer", "S-1", "Erststart ohne zuletzt geöffnete Projekte: Kopf mit Logo, drei Aktionskarten, Hinweis, Beispielkarten, erkannte Emulatoren und Tastenkürzel");
await V(async (p) => { await window.__vc.openProject(p); }, b); await w(500); await V(async (p) => { await window.__vc.openProject(p); }, a); await w(700);
await V(() => window.__vc.runCommand("projekt_schliessen")); await w(1000);
await bild("start-02-zuletzt-dunkel-breit", "S-3", "Zuletzt geöffnet als Karten: mit Projektbild (jumprun) bzw. Ersatzkachel (music_demo), System, Alter, Stern und Menü");
await bild("start-03-komplett", "S-1…S-4", "Ganze Seite", true);
await page.setViewportSize({ width: 640, height: 900 }); await bild("start-04-schmal-640", "S-5", "Schmal (640 px): Karten untereinander", true); await page.setViewportSize({ width: 1440, height: 900 });
await V(() => window.__vc.setTheme("light")); await bild("start-05-hell", "S-5", "Helles Farbschema"); await V(async () => { window.__vc.setTheme("dark"); await window.__vc.setLanguage("en"); }); await w(800); await bild("start-06-englisch", "S-5", "Englisch (maschinell, nicht abgenommen)"); await V(async () => { await window.__vc.setLanguage("de"); });
writeFileSync(join(out, "LIESMICH.md"), "# Bilderstrecke Startseite (Entwurf A; de, dunkel, wenn nicht anders genannt)\n\nErzeugt mit `node ui/tools/bilder-start.mjs` (Playwright/WebKit, Prüfbrücke, echter Kern). Beispielbilder gibt es nur im Programmpaket (nicht im Quelltext): in dieser Strecke zeigen die Beispielkarten die Ersatzkachel; im Paket stehen die echten Aufnahmen.\n\n| Handprüfpunkt | Bild | Was zu sehen sein muss |\n|---|---|---|\n" + liste.map(([p, f, s]) => `| ${p} | ${f} | ${s} |`).join("\n") + "\n");
console.log(liste.length + " Bilder"); await browser.close();
