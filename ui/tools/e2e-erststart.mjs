// SPDX-License-Identifier: GPL-3.0-or-later
// Ablauftest Erststart (ohne VC_NO_WIZARD, leeres HOME): der Assistent öffnet sich NICHT von selbst; Startseite mit Willkommenskarte → „Einrichtung starten“ öffnet den Assistenten wieder
// (nie den zuletzt benutzten Bereich); Start ohne Projekt zeigt immer die Startseite, auch wenn zuvor ein anderer Bereich gemerkt wurde; „Rückgängig“ im Assistenten stellt auch den Erststart-Zustand her.
// Aufruf: node tools/e2e-erststart.mjs <HOME des Kerns (leer, ohne Einstellungen)> <Ordner für Beispiele> <Projektkopie>
import { webkit } from "playwright";
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
const [home, proj, projekt] = process.argv.slice(2);
if (!home || !proj || !projekt) { console.error("Aufruf: node tools/e2e-erststart.mjs <HOME> <Ordner für Beispiele> <Projektkopie>"); process.exit(2); }
const browser = await webkit.launch(); const ctx = await browser.newContext({ viewport: { width: 1440, height: 900 } }); const page = await ctx.newPage();
const fails = []; let n = 0; const ok = (c, m) => { n++; if (!c) { fails.push(m); console.log("FEHLER:", m); } else console.log("ok:", m); };
page.on("pageerror", (e) => { fails.push("Seitenfehler " + e.message); console.log("Seitenfehler", e.message); });
const warte = (ms = 500) => page.waitForTimeout(ms);
const laden = async () => { await page.goto("http://127.0.0.1:1420/?bridge=17420"); await page.waitForFunction(() => window.__vc && window.__vc.ui.ready); await page.evaluate(async () => { await window.__vc.setLanguage("de"); }); await warte(1200); };
const wizFile = join(home, ".config", "vintagecoder", "wizard.json");
// Ein früherer Besuch im Bereich Musik bleibt im Webview-Speicher gemerkt (wie im Handtest)
await page.goto("http://127.0.0.1:1420/?bridge=17420"); await page.evaluate(() => { localStorage.setItem("mode", "musik"); localStorage.setItem("sub-grafik", "kacheln"); });
await laden();
// Erststart: der Assistent öffnet sich NICHT von selbst; die Startseite zeigt die Willkommenskarte (Entscheidung des Inhabers)
ok(await page.locator("[data-wz=assistent]").count() === 0, "Erststart: der Assistent öffnet sich nicht von selbst");
ok(await page.locator("[data-start=welcome-card]").count() === 1 && /Willkommen bei VintageCoder/.test(await page.locator("[data-start=welcome-card]").innerText()) && await page.locator("[data-start=welcome-setup]").count() === 1, "Die Startseite zeigt die Willkommenskarte mit „Einrichtung starten“ (trotz gemerktem Bereich Musik)");
ok(await page.evaluate(() => window.__vc.ui.mode) === "code", "Ohne Projekt steht der Bereich auf Code (Startseite), nicht auf dem gemerkten");
await page.locator("[data-start=welcome-setup]").click(); await warte(900);
ok(await page.locator("[data-wz=assistent]").count() === 1 && await page.evaluate(() => window.__vc.ui.extra) === "wizard", "„Einrichtung starten“ öffnet den Assistenten (nicht „Kein Song geöffnet“)");
// Schnelleinrichtung übernehmen und rückgängig machen (frischer Erststart): wizard.json verschwindet wieder, die Karte kommt beim nächsten Start wieder
await page.evaluate(async (p) => { const d = await window.__vc.call("wiz_set", { examples: { copy: true, dest: p } }); window.__vc.wz.data = d; }, proj); await warte(400);
await page.evaluate(async () => { await window.__vc.call("wiz_apply", { deselected: [] }); }); await warte(800);
const nachher = existsSync(wizFile) ? JSON.parse(readFileSync(wizFile, "utf8")) : {};
ok(nachher.first_run_done === true, "Nach „Übernehmen“: wizard.json hat first_run_done" + ((nachher.examples_copied ?? []).length ? " und die kopierten Beispiele" : ""));
await page.evaluate(async () => { await window.__vc.call("wiz_undo", { deselected: [] }); }); await warte(800);
ok(!existsSync(wizFile), "Nach „Rückgängig“ beim Erststart gibt es wizard.json nicht mehr (Stand wie vor „Übernehmen“)");
ok(!existsSync(proj) || readdirSync(proj).length === 0, "Rückgängig räumt kopierte Beispiele und leere Ordner weg");
await laden();   // Neustart (Fenster neu laden), ohne den Assistenten zu schließen: Schließen würde den Erststart als erledigt merken
ok(await page.locator("[data-wz=assistent]").count() === 0 && await page.locator("[data-start=welcome-card]").count() === 1, "Neustart nach Rückgängig: wieder die Willkommenskarte, der Assistent öffnet sich nicht");
// Mit Projekt: kein Überlagern-Dialog, dafür ein Hinweis in der Statuszeile
await page.evaluate(async (p) => { await window.__vc.openProject(p); }, projekt); await warte(1200);
ok(await page.locator("[data-wz=assistent]").count() === 0, "Start mit Projekt: der Assistent öffnet sich nicht von selbst");
ok(await page.locator("[data-status=einrichtung]").count() === 1, "… statt dessen steht ein Hinweis in der Statuszeile");
await page.locator("[data-status=einrichtung]").click(); await warte(900);
ok(await page.locator("[data-wz=assistent]").count() === 1, "Der Hinweis in der Statuszeile öffnet den Assistenten");
await page.locator("[data-wz=spaeter]").click(); await warte(600);
await page.evaluate(async () => { await window.__vc.runCommand("startseite"); }); await warte(800);
// Karte „Später“ blendet sie dauerhaft aus
ok(await page.locator("[data-start=welcome-card]").count() === 1, "Zurück auf der Startseite steht die Willkommenskarte noch");
await page.locator("[data-start=welcome-later]").click(); await warte(400);
ok(await page.locator("[data-start=welcome-card]").count() === 0, "„Später“ auf der Karte blendet sie aus");
await laden();
ok(await page.locator("[data-start=welcome-card]").count() === 0 && await page.evaluate(() => window.__vc.ui.mode) === "code", "Nach dem Neustart bleibt die Karte weg, ohne Projekt gilt Startseite (nicht der zuletzt benutzte Bereich)");
console.log("\n" + n + " Prüfungen, " + fails.length + " Fehler"); await browser.close(); process.exit(fails.length ? 1 : 0);
