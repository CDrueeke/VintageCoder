// SPDX-License-Identifier: GPL-3.0-or-later
// Bilderstrecke für die Abnahme (E5 Tracker): Playwright (WebKit), Prüfbrücke. Aufruf: node tools/bilder-e5.mjs <Kopie c64/music_demo> <Kopie atari/music_demo> <Ausgabeordner>
import { webkit } from "playwright";
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
const [c64, atari, out] = process.argv.slice(2);
if (!out) { console.error("Aufruf: node tools/bilder-e5.mjs <c64 music_demo> <atari music_demo> <Ausgabe>"); process.exit(2); }
mkdirSync(out, { recursive: true });
const browser = await webkit.launch(); const page = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
const liste = []; const bild = async (name, punkt, soll) => { await page.waitForTimeout(500); await page.screenshot({ path: join(out, name + ".png") }); liste.push([punkt, name + ".png", soll]); };
const V = (js, a) => page.evaluate(js, a); const z = (r, c) => page.locator(`[data-tk=zelle][data-r='${r}'][data-c='${c}']`);
await page.goto("http://127.0.0.1:1420/?bridge=17420"); await page.waitForFunction(() => window.__vc && window.__vc.ui.ready);
await V(async () => { await window.__vc.setLanguage("de"); window.__vc.setTheme("dark"); });
await V(async (p) => { await window.__vc.openProject(p); }, c64); await page.waitForTimeout(900);
await V(() => window.__vc.runCommand("nav:musik")); await page.waitForTimeout(500);
await bild("e5-01-leer", "E5-1 (ohne geöffneten Song)", "Hinweis „Kein Song geöffnet“, Liste der Songs des Projekts, Knopf Öffnen");
await V(() => window.__vc.openFile("music/tune.vcsong")); await page.waitForTimeout(1000);
await bild("e5-02-song-dunkel-breit", "E5-1", "Links Reihenfolge (Muster, Wiederholungszeichen) und Instrumente, Mitte Raster mit drei Stimmen, rechts Instrumenteneditor (Klangform, ADSR, Pulsbreite)");
await z(4, 1).click(); await page.keyboard.press("KeyZ"); await page.keyboard.press("KeyC"); await page.keyboard.press("KeyB"); await bild("e5-03-noten-eingegeben", "E5-3", "Drei Noten von oben nach unten eingetragen (Z, C, B), Cursor in der vierten Zeile, Datei „ungespeichert“");
await z(2, 0).click(); await page.keyboard.press("Shift+ArrowDown"); await page.keyboard.press("Shift+ArrowRight"); await page.keyboard.press("Shift+ArrowDown"); await bild("e5-04-auswahl", "E5-5", "Rechteckige Auswahl über zwei Stimmen und vier Zeilen");
await page.locator("[data-act=pattern-new]").click(); await page.locator("[data-act=ord-append]").click(); await bild("e5-05-muster-reihenfolge", "E5-8", "Neues Muster angelegt und an die Reihenfolge gehängt (Muster 3 unten in der Liste)");
await page.locator("[data-tk=instrument]").nth(1).click(); await page.getByRole("radio", { name: "Säge" }).click(); await bild("e5-06-instrument-saege", "E5-9", "Instrument „Bass“ mit Klangform Säge, ADSR-Regler");
await page.locator("[data-act=save]").click(); await page.waitForTimeout(500);
await V(async (p) => { await window.__vc.openProject(p); }, atari); await page.waitForTimeout(900);
await V(() => window.__vc.openFile("music/tune_pokey.vcsong")); await page.waitForTimeout(1000);
await bild("e5-08-pokey", "E5-9", "POKEY-Song: Verzerrung (Reiner Ton, Bass, Buzz, Gritty) und Hüllkurve je Bild als 16 Regler, vier Stimmen");
await z(0, 0).click(); await page.keyboard.press("Space"); await page.waitForTimeout(1300); await bild("e5-07-wiedergabe", "E5-13/14", "Song spielt: Zeile blau hinterlegt, Pegel je Stimme (Balken im Spaltenkopf), Statuszeile „Wiedergabe: Eintrag …, Zeile …“, Knopf Abspielen gedrückt, Stopp frei");
await page.locator("[data-tk=mute][data-c='0']").click(); await page.locator("[data-tk=solo][data-c='1']").click(); await page.waitForTimeout(500); await bild("e5-07b-stumm-solo", "E5-15", "Stimme 1 stumm (M gefüllt), Stimme 2 solo (S gefüllt)"); await page.locator("[data-tk=mute][data-c='0']").click(); await page.locator("[data-tk=solo][data-c='1']").click(); await page.locator("[data-act=stop]").click(); await page.waitForTimeout(400);
await page.locator("[data-tk=stimme][data-c='1']").click(); await bild("e5-07c-stimme-aus", "E5-23", "Stimme 2 ausgeblendet: Spalte fehlt, Knopf „2“ nicht gedrückt, „Alle einblenden“ frei"); await page.locator("[data-tk=alle-stimmen]").click();
await page.locator("[data-act=konvertieren]").click(); await page.waitForSelector("dialog[open] [data-tk=import]"); await bild("e5-07d-konvertieren", "E5-24", "Dialog „Song aus anderem System übernehmen“: Datei und Art, Name, Zielsystem"); await page.getByRole("button", { name: "Abbrechen" }).first().click(); await page.waitForTimeout(300);
await page.setViewportSize({ width: 640, height: 900 }); await bild("e5-09-schmal-640", "E5-1", "Schmal (640 px): Listen, Raster und Instrument untereinander"); await page.setViewportSize({ width: 1440, height: 900 });
await V(() => window.__vc.setTheme("light")); await bild("e5-10-hell", "E5-1", "Helles Farbschema"); await V(async () => { window.__vc.setTheme("dark"); await window.__vc.setLanguage("en"); }); await bild("e5-11-englisch", "E5-1", "Englisch (maschinell, nicht abgenommen)"); await V(async () => { await window.__vc.setLanguage("de"); });
writeFileSync(join(out, "LIESMICH.md"), "# Bilderstrecke E5 Tracker (E5.1, E5.2) (de, dunkel, wenn nicht anders genannt)\n\nErzeugt mit `node ui/tools/bilder-e5.mjs` (Playwright/WebKit, Prüfbrücke, echter Kern).\n\n| Handprüfpunkt | Bild | Was zu sehen sein muss |\n|---|---|---|\n" + liste.map(([p, f, s]) => `| ${p} | ${f} | ${s} |`).join("\n") + "\n\nNur von Hand zu prüfen: Klaviertasten (Tastenposition, auch bei anderem Tastaturlayout), Maus-Auswahl, Tastatur, VoiceOver, Gefühl beim Eingeben.\n");
console.log(liste.length + " Bilder"); await browser.close();
