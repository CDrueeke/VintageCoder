// SPDX-License-Identifier: GPL-3.0-or-later
// Bilderstrecke für die Abnahme (E13 Hilfe): Playwright (WebKit), Prüfbrücke (App mit VC_HELP_IMAGES/VC_HELP_CACHE wie bei e2e-hilfe). Aufruf: node tools/bilder-e13.mjs <jumprun> <Zwischenspeicher> <Ausgabe>
import { webkit } from "playwright";
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
const [proj, cache, out] = process.argv.slice(2);
if (!out) { console.error("Aufruf: node tools/bilder-e13.mjs <jumprun> <Zwischenspeicher> <Ausgabe>"); process.exit(2); }
mkdirSync(out, { recursive: true }); mkdirSync(cache, { recursive: true });
writeFileSync(join(cache, "menueleiste.png"), Buffer.from("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==", "base64"));
const browser = await webkit.launch(); const page = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
const liste = []; const bild = async (name, punkt, soll) => { await page.waitForTimeout(500); await page.screenshot({ path: join(out, name + ".png") }); liste.push([punkt, name + ".png", soll]); };
const V = (js, a) => page.evaluate(js, a); const w = (ms = 500) => page.waitForTimeout(ms);
await page.goto("http://127.0.0.1:1420/?bridge=17420"); await page.waitForFunction(() => window.__vc && window.__vc.ui.ready);
await V(async (p) => { await window.__vc.setLanguage("de"); window.__vc.setTheme("dark"); await window.__vc.openProject(p); }, proj); await w(900);
await V(() => window.__vc.runCommand("hilfe_reiter")); await w(900);
await bild("e13-01-start", "E13-1", "Hilfe: links Suche und Themen nach Gruppen (Handbuch zuerst), rechts Willkommen");
const ids = await V(() => window.__vc.hp.themen.map((t) => t.id)); const kap = (re) => ids.find((i) => re.test(i)) ?? ids[0];
await page.locator(`[data-hp=thema][data-id='${kap(/oberfl/)}']`).click(); await w(1000); await bild("e13-02-kapitel-dunkel-breit", "E13-2", "Handbuchkapitel: Überschriften, Absätze, Liste, Tabelle; Bild aus dem Zwischenspeicher");
await page.locator(`[data-hp=thema][data-id='${kap(/erstes|first/)}']`).click(); await w(1000); await bild("e13-03-bilder-frage", "E13-7", "Kapitel mit Online-Bildern: Frage „Handbuchbilder laden?“ (Nur dieses Mal, Immer, Nie)");
await page.locator("[data-hp=suche]").fill("lda"); await w(800); await bild("e13-04-suche", "E13-4", "Suche nach „lda“: Treffer mit Auszug");
await page.locator("[data-hp=treffer-eintrag]").first().click(); await w(800); await bild("e13-05-treffer", "E13-4", "Beschreibung des Befehls LDA");
await page.locator("[data-hp=suche]").fill(""); await page.setViewportSize({ width: 640, height: 900 }); await bild("e13-06-schmal-640", "E13-9", "Schmal (640 px): Liste über dem Text"); await page.setViewportSize({ width: 1440, height: 900 });
await V(() => window.__vc.setTheme("light")); await bild("e13-07-hell", "E13-9", "Helles Farbschema"); await V(async () => { window.__vc.setTheme("dark"); await window.__vc.setLanguage("en"); }); await w(1000); await bild("e13-08-englisch", "E13-9", "Englisch (maschinell, nicht abgenommen)"); await V(async () => { await window.__vc.setLanguage("de"); });
writeFileSync(join(out, "LIESMICH.md"), "# Bilderstrecke E13 Hilfe (de, dunkel, wenn nicht anders genannt)\n\nErzeugt mit `node ui/tools/bilder-e13.mjs` (Playwright/WebKit, Prüfbrücke, echter Kern; Handbuchbilder nur aus einem Test-Zwischenspeicher).\n\n| Handprüfpunkt | Bild | Was zu sehen sein muss |\n|---|---|---|\n" + liste.map(([p, f, s]) => `| ${p} | ${f} | ${s} |`).join("\n") + "\n\nNur von Hand zu prüfen: F1 im Editor, echte Handbuchbilder aus dem Netz, VoiceOver. Die Kapiteltexte beschreiben teils noch die bisherige Oberfläche.\n");
console.log(liste.length + " Bilder"); await browser.close();
