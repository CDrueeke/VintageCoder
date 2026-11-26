// SPDX-License-Identifier: GPL-3.0-or-later
// Bilderstrecke für die Abnahme (E7 Palette, E8 Kacheln): Playwright (WebKit), eigenes Fenster, gegen die Prüfbrücke. Schreibt PNG-Dateien und LIESMICH.md (Punkt → Bild → was zu sehen sein muss).
// Aufruf: node tools/bilder-e7e8.mjs <Ordner Kopie von examples/mega65/pal_demo> <Ordner Kopie von examples/c64/gfx_demo> <Ordner Kopie von examples/atari/gfx_demo> <Ausgabeordner>
import { webkit } from "playwright";
import { mkdirSync, writeFileSync, copyFileSync } from "node:fs";
import { join } from "node:path";
const [pal, kach, atari, out] = process.argv.slice(2);
if (!out) { console.error("Aufruf: node tools/bilder-e7e8.mjs <pal_demo> <gfx_demo> <atari_gfx_demo> <Ausgabe>"); process.exit(2); }
mkdirSync(out, { recursive: true });
const browser = await webkit.launch(); const ctx = await browser.newContext({ viewport: { width: 1440, height: 900 }, deviceScaleFactor: 1 }); const page = await ctx.newPage();
const liste = []; const bild = async (name, punkt, soll) => { await page.waitForTimeout(450); await page.screenshot({ path: join(out, name + ".png") }); liste.push([punkt, name + ".png", soll]); };
const V = (js, arg) => page.evaluate(js, arg);
await page.goto("http://127.0.0.1:1420/?bridge=17420"); await page.waitForFunction(() => window.__vc && window.__vc.ui.ready);
await V(async () => { await window.__vc.setLanguage("de"); window.__vc.setTheme("dark"); });
const projekt = async (p) => { await V(async (p) => { await window.__vc.openProject(p); }, p); await page.waitForTimeout(900); };
const datei = async (r) => { await V((r) => window.__vc.openFile(r), r); await page.waitForTimeout(900); };

// ---- E7 Palette
writeFileSync(join(pal, "graphics", "uses.vcsprite"), JSON.stringify({ format: 1, kind: "sprite", colors: { sprite: 5, mc1: 2 }, frames: [] }));
await projekt(pal);
await V(() => window.__vc.runCommand("nav:grafik:palette")); await page.waitForTimeout(500);
await bild("e7-01-leer", "E7-1 (ohne geöffnete Palette)", "Hinweis „Keine Palette geöffnet“, Liste der Palettendateien (pal65, pal65b) als Knöpfe, Knopf Öffnen");
await datei("graphics/pal65.vcpal");
await bild("e7-02-raster-dunkel-breit", "E7-1/2", "Farbraster 16 × 16 links, Karte „Farbe 0“ rechts, Dateiliste mit zwei Paletten unter dem Raster, Statuszeile mit Dateiname");
await page.locator("[data-pal=farbe][data-i='87']").click(); await bild("e7-03-farbe-87", "E7-2", "Farbe 87 gewählt (Rahmen im Raster), Farbfeld, Regler und Hexwert zeigen dieselbe Farbe");
await page.locator("[data-pal=kanal-0]").fill("201"); await bild("e7-04-regler-rot", "E7-3", "Regler Rot 201: Zahl daneben, Farbfeld und Raster folgen, Speichern-Knopf aktiv, „ungespeichert“ unten");
await page.locator("[data-pal=hex]").fill("zz12"); await bild("e7-05-hexwert-fehler", "E7-4", "Ungültiger Hexwert: Feld rot markiert, Hinweis „kein Farbwert“");
await page.locator("[data-pal=hex]").fill("#00ff7f"); await page.locator("[data-pal=hex]").blur();
await page.locator("[data-pal=farbe][data-i='5']").click(); await bild("e7-06-im-einsatz", "E7-7", "Farbe 5: Karte „Im Einsatz“ nennt uses.vcsprite (Sprite) als Link");
await page.locator("[data-pal=farbe][data-i='6']").click(); await bild("e7-07-nicht-im-einsatz", "E7-7", "Farbe 6: „in keiner Mediendatei … verwendet“");
await page.locator("[data-act=save]").click(); await page.waitForTimeout(500);
await page.locator("[data-act=export]").click(); await page.waitForSelector("dialog[open] [data-ex=vorschau]"); await bild("e7-08-export-dialog", "E7-8", "Export-Dialog mit Vorschau der Palette als Assembler-Daten");
await page.getByRole("button", { name: "Abbrechen" }).first().click(); await page.waitForTimeout(300);
await page.locator("[data-pal=farbe][data-i='9']").click(); await page.locator("[data-pal=kanal-2]").fill("77");
await page.getByRole("button", { name: /pal65b/ }).click(); await bild("e7-09-rueckfrage-ungespeichert", "E7-10", "Wechsel zur zweiten Palette bei ungespeicherter Änderung: Rückfrage Speichern/Verwerfen/Abbrechen");
await page.locator("[data-ask=discard]").click(); await page.waitForTimeout(800);
await bild("e7-10-zweite-palette", "E7-10", "Zweite Palette pal65b geöffnet (Farbe 1 ist gelb), in der Dateiliste hervorgehoben");
await page.setViewportSize({ width: 640, height: 900 }); await bild("e7-11-schmal-640", "E7-1", "Schmal (640 px): Raster und Karte untereinander, nichts ragt heraus");
await page.setViewportSize({ width: 1440, height: 900 });
await V(() => window.__vc.setTheme("light")); await bild("e7-12-hell-breit", "E7-1", "Helles Farbschema: Raster, Regler und Karten gut lesbar");
await V(async () => { window.__vc.setTheme("dark"); await window.__vc.setLanguage("en"); }); await bild("e7-13-englisch", "E7-1", "Englisch: „Colour n“, „In use“, „Hex value“ (Katalog en maschinell, nicht abgenommen)");
await V(async () => { await window.__vc.setLanguage("de"); });

// ---- E8 Kacheln
await projekt(kach);
await V(() => window.__vc.runCommand("nav:grafik:kacheln")); await page.waitForTimeout(500);
await bild("e8-01-leer", "E8-1 (ohne geöffneten Kachelsatz)", "Hinweis „Kein Kachelsatz geöffnet“ mit Liste der Kachelsätze des Projekts");
await datei("graphics/level_tiles.vctiles");
await bild("e8-02-kachelsatz-dunkel-breit", "E8-1", "Links die Kacheln mit Vorschau, Name und Verwendungszähler (n×), Mitte große Kachel (2 × 2 Zellen) und Zeichentafel, rechts Karte Kachel mit Farbe, Eigenschaften, Größe, Zielzeichensatz-Füllstand");
await page.locator("[data-kc=kachel]").nth(1).click(); await bild("e8-03-kachel-stein", "E8-2", "Kachel „stein“ gewählt (Rahmen), große Kachel zeigt sie, Fußzeile „Kachel 2 von 4“");
await page.locator("[data-kc=zelle]").nth(1).click(); await page.locator("[data-kc=zeichen][data-c='65']").click(); await bild("e8-04-zeichen-gesetzt", "E8-3", "Zeichen A aus der Tafel in die markierte Zelle gesetzt; Kacheln links folgen, „ungespeichert“");
const zb = await Promise.all([0, 1, 3].map((k) => page.locator(`[data-kc=zelle][data-k='${k}']`).boundingBox())); const mid = (b) => [b.x + b.width / 2, b.y + b.height / 2];
await page.locator("[data-kc=zeichen][data-c='66']").click(); await page.mouse.move(...mid(zb[0])); await page.mouse.down(); await page.mouse.move(...mid(zb[1]), { steps: 4 }); await page.mouse.move(...mid(zb[2]), { steps: 4 }); await page.mouse.up();
await bild("e8-05-zug-ueber-zellen", "E8-4", "Zug über drei Zellen: alle tragen Zeichen B");
await page.getByRole("radio", { name: "Je Zelle" }).click(); await bild("e8-06-farbe-je-zelle", "E8-8", "Farbe je Zelle: Pinselmodus (Beides/Nur Zeichen/Nur Farbe) und Pinselfarbe erscheinen");
await page.getByRole("radio", { name: "Je Kachel" }).click();
await page.locator("[data-kc=prop-neu]").fill("tod"); await page.locator("[data-act=prop-neu]").click(); await bild("e8-07-eigenschaften", "E8-9", "Zwei Eigenschaften (solid, tod) als Schalter");
await page.locator("[data-kc=gw]").fill("1"); await page.locator("[data-kc=gh]").fill("1"); await page.locator("[data-act=size]").click(); await bild("e8-08-groesse-rueckfrage", "E8-10", "Rückfrage „gehen belegte Zellen verloren“ bei Größe 1 × 1");
await page.locator("[data-ask=no]").click();
await page.locator("[data-act=tile-dup]").click(); await bild("e8-09-duplikat", "E8-11", "Duplikat der Kachel: in der Liste mit „=“ markiert, Knopf Zusammenführen aktiv");
await page.locator("[data-act=tile-merge]").click(); await page.locator("[data-kc=kachel]").nth(1).click(); await page.locator("[data-act=tile-delete]").click(); await bild("e8-10-loeschen-rueckfrage", "E8-11", "Löschen einer in Bildschirmbanken benutzten Kachel: Rückfrage mit Zahl der Verwendungen");
await page.locator("[data-ask=no]").click();
await page.locator("[data-kc=name]").fill("stein2"); await page.locator("[data-kc=name]").blur(); await page.waitForTimeout(500);
if (await page.locator("[data-act=save]").isEnabled()) { await page.locator("[data-act=save]").click(); await page.waitForTimeout(500); }
await page.locator("[data-act=export]").click(); await page.waitForSelector("dialog[open] [data-ex=vorschau]"); await bild("e8-11-export-dialog", "E8-13", "Export-Dialog mit Vorschau des Kachelsatzes");
await page.getByRole("button", { name: "Abbrechen" }).first().click(); await page.waitForTimeout(300);
await page.setViewportSize({ width: 640, height: 900 }); await bild("e8-12-schmal-640", "E8-1", "Schmal (640 px): die drei Bereiche stehen untereinander");
await page.setViewportSize({ width: 1440, height: 900 }); await V(() => window.__vc.setTheme("light")); await bild("e8-13-hell-breit", "E8-1", "Helles Farbschema");
await V(async () => { window.__vc.setTheme("dark"); await window.__vc.setLanguage("en"); }); await bild("e8-14-englisch", "E8-1", "Englisch: Tile set, Properties, Target charset (maschinell, nicht abgenommen)"); await V(async () => { await window.__vc.setLanguage("de"); });
await projekt(atari); await datei("graphics/alevel_tiles.vctiles");
await bild("e8-15-atari", "E8-1", "Atari-Kachelsatz (128 Zeichen in der Tafel, Atari-Palette)");

writeFileSync(join(out, "LIESMICH.md"), "# Bilderstrecke E7 Palette und E8 Kacheln (de, dunkel, wenn nicht anders genannt)\n\nErzeugt mit `node ui/tools/bilder-e7e8.mjs` (Playwright/WebKit, Prüfbrücke, echter Kern).\n\n| Handprüfpunkt | Bild | Was zu sehen sein muss |\n|---|---|---|\n" + liste.map(([p, f, s]) => `| ${p} | ${f} | ${s} |`).join("\n") + "\n\nNur von Hand zu prüfen (Gefühl/Handhabung): Maus und Zug, Tastatur und Fokus, VoiceOver, Pipette mit Alt+Klick.\n");
console.log(liste.length + " Bilder in " + out); await browser.close();
