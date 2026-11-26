// SPDX-License-Identifier: GPL-3.0-or-later
// Bilderstrecke für die Abnahme (E9 Bildschirm): Playwright (WebKit) gegen die Prüfbrücke. Aufruf: node tools/bilder-e9.mjs <Kopie von examples/c64/gfx_demo> <Kopie von examples/atari/gfx_demo> <Ausgabeordner>
import { webkit } from "playwright";
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
const [c64, atari, out] = process.argv.slice(2);
if (!out) { console.error("Aufruf: node tools/bilder-e9.mjs <gfx_demo> <atari_gfx_demo> <Ausgabe>"); process.exit(2); }
mkdirSync(out, { recursive: true });
const browser = await webkit.launch(); const page = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
const liste = []; const bild = async (name, punkt, soll) => { await page.waitForTimeout(500); await page.screenshot({ path: join(out, name + ".png") }); liste.push([punkt, name + ".png", soll]); };
const V = (js, a) => page.evaluate(js, a);
await page.goto("http://127.0.0.1:1420/?bridge=17420"); await page.waitForFunction(() => window.__vc && window.__vc.ui.ready);
await V(async () => { await window.__vc.setLanguage("de"); window.__vc.setTheme("dark"); });
const zelle = (x, y, w = 40) => page.locator(`[data-bs=zelle][data-k='${y * w + x}']`); const mid = (b) => [b.x + b.width / 2, b.y + b.height / 2];
await V(async (p) => { await window.__vc.openProject(p); }, c64); await page.waitForTimeout(900);
await V(() => window.__vc.runCommand("nav:grafik:bildschirm")); await page.waitForTimeout(500);
await bild("e9-01-leer", "E9-1 (ohne geöffneten Bildschirm)", "Hinweis „Kein Bildschirm geöffnet“ mit Liste der Bildschirm-Dateien");
await V(() => window.__vc.openFile("graphics/title.vcscreen")); await page.waitForTimeout(1000);
await bild("e9-02-bildschirm-dunkel-breit", "E9-1", "Links Zeichentafel und Zeichenfarbe, Mitte der Bildschirm mit Rand und Hintergrund (Zeichen im Zeichensatz des Projekts), rechts Größe 40 × 25, Farben des Systems, Zeichensatz, Ausgabe");
await page.locator("[data-bs=zeichen][data-c='66']").click(); await zelle(5, 5).click(); await bild("e9-03-zeichen-gesetzt", "E9-2", "Zeichen B in Zelle (5, 5), Datei „ungespeichert“, Fußzeile nennt Spalte und Zeile");
const bb = await Promise.all([5, 8, 12].map((x) => zelle(x, 8).boundingBox())); await page.mouse.move(...mid(bb[0])); await page.mouse.down(); await page.mouse.move(...mid(bb[1]), { steps: 4 }); await page.mouse.move(...mid(bb[2]), { steps: 4 }); await page.mouse.up();
await bild("e9-04-zug", "E9-3", "Zug über sieben Zellen ohne Lücken");
await page.locator("[aria-label='Farbe der neuen Zeichen'] [data-idx='5']").click(); await page.locator("[data-tool=rect]").click(); const r0 = await zelle(20, 12).boundingBox(), r1 = await zelle(30, 16).boundingBox(); await page.mouse.move(...mid(r0)); await page.mouse.down(); await page.mouse.move(...mid(r1), { steps: 5 }); await page.mouse.up();
await bild("e9-05-rahmen-farbe", "E9-4/5", "Grüner Rahmen (Zeichenfarbe 5) von Ecke zu Ecke");
await page.locator("[data-tool=text]").click(); await zelle(3, 20).click(); await page.keyboard.type("HALLO WELT"); await page.waitForTimeout(900); await bild("e9-06-text", "E9-6", "Text „HALLO WELT“ als Bildschirmzeichen, Hinweiszeile zum Textwerkzeug");
await page.locator("[data-tool=char]").click(); await page.locator("[aria-label='Rahmen'] [data-idx='2']").click(); await page.locator("[aria-label='Hintergrund'] [data-idx='0']").click(); await bild("e9-07-farben-system", "E9-8", "Rahmen rot, Hintergrund schwarz");
await page.locator("[data-bs=zeichensatz]").selectOption(""); await bild("e9-08-ohne-zeichensatz", "E9-9", "Ohne Zeichensatz: keine Zeichen sichtbar, Tafel leer");
await page.locator("[data-act=undo]").click();
await page.locator("[data-act=zerlegen]").click(); await page.waitForSelector("[data-bs=zerlegen-vorschau]"); await page.waitForTimeout(800); await bild("e9-09-zerlegen-dialog", "E9-10", "Dialog „In Kacheln zerlegen“ mit Kachelgröße und Vorschau „n Kacheln, m Patches, x → y Byte“");
await page.locator("[data-bs=zerlegen-abbrechen]").click();
await page.setViewportSize({ width: 640, height: 900 }); await bild("e9-10-schmal-640", "E9-1", "Schmal (640 px): Tafel, Bildschirm und Karte untereinander"); await page.setViewportSize({ width: 1440, height: 900 });
await V(() => window.__vc.setTheme("light")); await bild("e9-11-hell", "E9-1", "Helles Farbschema"); await V(async () => { window.__vc.setTheme("dark"); await window.__vc.setLanguage("en"); }); await bild("e9-12-englisch", "E9-1", "Englisch (maschinell, nicht abgenommen)"); await V(async () => { await window.__vc.setLanguage("de"); });
if (await page.locator("[data-act=save]").isEnabled()) { await page.locator("[data-act=save]").click(); await page.waitForTimeout(500); }
await V(async (p) => { await window.__vc.openProject(p); }, atari); await page.waitForTimeout(900); await V(() => window.__vc.openFile("graphics/atitle.vcscreen")); await page.waitForTimeout(1000);
await bild("e9-13-atari", "E9-1", "Atari-Bildschirm (Zeichensatz mit 128 Zeichen, eine Zeichenfarbe: keine Farbwahl je Zeichen)");
writeFileSync(join(out, "LIESMICH.md"), "# Bilderstrecke E9 Bildschirm-Editor (de, dunkel, wenn nicht anders genannt)\n\nErzeugt mit `node ui/tools/bilder-e9.mjs` (Playwright/WebKit, Prüfbrücke, echter Kern).\n\n| Handprüfpunkt | Bild | Was zu sehen sein muss |\n|---|---|---|\n" + liste.map(([p, f, s]) => `| ${p} | ${f} | ${s} |`).join("\n") + "\n\nNur von Hand zu prüfen: Maus und Zug (auch schnell), Tastatur (Pfeile, Enter, D/R/F/T/I/E), VoiceOver, Gefühl beim Tippen.\n");
console.log(liste.length + " Bilder"); await browser.close();
