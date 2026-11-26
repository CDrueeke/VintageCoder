// SPDX-License-Identifier: GPL-3.0-or-later
// Bilderstrecke Projekt-Rest (P4): Quellgruppen bearbeiten, Ziele (undokumentierte Befehle, Startziel), Diskettenabbild. Aufruf: node tools/bilder-projekt-p4.mjs <Kopie c64/jumprun> <Ausgabe>
import { webkit } from "playwright";
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
const [proj, out] = process.argv.slice(2);
if (!out) { console.error("Aufruf: node tools/bilder-projekt-p4.mjs <Projekt> <Ausgabe>"); process.exit(2); }
mkdirSync(out, { recursive: true });
const browser = await webkit.launch(); const page = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
const liste = []; const w = (ms = 500) => page.waitForTimeout(ms);
const bild = async (n, p, soll) => { await w(400); await page.screenshot({ path: join(out, n + ".png") }); liste.push([p, n + ".png", soll]); };
await page.goto("http://127.0.0.1:1420/?bridge=17420"); await page.waitForFunction(() => window.__vc && window.__vc.ui.ready);
await page.evaluate(async (p) => { await window.__vc.setLanguage("de"); window.__vc.setTheme("dark"); await window.__vc.openProject(p); }, proj); await w(800);
await page.evaluate(() => window.__vc.runCommand("nav:projekt:quellen")); await page.waitForSelector("[data-pj=quellen]");
await bild("p4-01-quellen", "P4-1", "Quellen: je Gruppe Art, Dialekt (bei BASIC), Ziele als Haken, „Gruppe entfernen“; je Datei Start-Knopf, Pfeile und Entfernen; unten „Datei hinzufügen“ und „Neue Gruppe“");
await page.locator("[data-pj=gruppe-neu-basic]").click(); await w(500); await page.locator("[data-pj-gruppe]").last().locator("[data-pj=datei-neu]").fill("fehlt.bas"); await page.locator("[data-pj-gruppe]").last().locator("[data-pj=datei-hinzu]").click(); await w(400);
await bild("p4-02-fehler", "P4-2", "Fehlerfall: Datei fehlt im Projekt → Meldung, nichts gespeichert; die neue Gruppe ist leer (BASIC)");
await page.evaluate(() => window.__vc.runCommand("nav:projekt:ziele")); await page.waitForSelector("[data-pj=ziele]"); await w(300);
await bild("p4-03-ziele", "P4-3", "Ziele: Variante, „Undokumentierte Befehle“, „Startziel (F5)“ je Zielsystem");
await page.evaluate(() => window.__vc.runCommand("nav:projekt:bauen")); await page.waitForSelector("[data-pj=bauen]"); await page.locator("[data-pj=abbild-neu][data-f=d64]").click(); await w(500);
await bild("p4-04-abbild", "P4-4", "Bauen: Karte „Diskettenabbild beim Bauen“ mit Format, Name und Ausgabedatei");
await page.setViewportSize({ width: 640, height: 900 }); await bild("p4-05-schmal", "P4-4", "Schmal (640 px): Karten bleiben lesbar"); await page.setViewportSize({ width: 1440, height: 900 });
await page.evaluate(() => window.__vc.setTheme("light")); await bild("p4-06-hell", "P4-4", "Hell"); await page.evaluate(() => window.__vc.setTheme("dark"));
writeFileSync(join(out, "LIESMICH.md"), "# Bilderstrecke Projekt-Rest (de, dunkel)\n\nErzeugt mit `node ui/tools/bilder-projekt-p4.mjs` (Playwright/WebKit, Prüfbrücke, Arbeitskopie von c64/jumprun).\n\n| Punkt | Bild | Erwartet |\n|---|---|---|\n" + liste.map((l) => `| ${l[0]} | ${l[1]} | ${l[2]} |`).join("\n") + "\n");
console.log(liste.length + " Bilder"); await browser.close();
