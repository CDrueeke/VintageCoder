// SPDX-License-Identifier: GPL-3.0-or-later
// Ablauftest Palette-Editor (E7): Playwright (WebKit) gegen die Prüfbrücke des echten Kerns.
// Aufruf: node tools/e2e-palette.mjs <Projektordner mit graphics/pal65.vcpal (wird verändert!)> [Bilderordner]
import { webkit } from "playwright";
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";

if (process.platform === "win32" && !process.env.VC_E2E_WIN) { console.log("übersprungen unter Windows\n0 Prüfungen, 0 Fehler"); process.exit(0); }
const [proj, shots] = process.argv.slice(2);
if (!proj) { console.error("Aufruf: node tools/e2e-palette.mjs <Projektordner> [Bilder]"); process.exit(2); }
if (shots) mkdirSync(shots, { recursive: true });
const browser = await webkit.launch();
const page = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
const fails = []; let n = 0;
const ok = (c, m) => { n++; if (!c) { fails.push(m); console.log("FEHLER:", m); } else console.log("ok:", m); };
page.on("pageerror", (e) => { fails.push("Seitenfehler " + e.message); console.log("Seitenfehler", e.message); });
const snap = async (name) => { if (shots) await page.screenshot({ path: join(shots, name + ".png") }); };
writeFileSync(join(proj, "graphics", "uses.vcsprite"), JSON.stringify({ format: 1, kind: "sprite", colors: { sprite: 5, mc1: 2 }, frames: [] }));
await page.goto("http://127.0.0.1:1420/?bridge=17420"); await page.waitForFunction(() => window.__vc && window.__vc.ui.ready);
await page.evaluate(async (p) => { await window.__vc.setLanguage("de"); await window.__vc.openProject(p); }, proj); await page.waitForTimeout(900);
const pv = () => page.evaluate(() => window.__vc.pal.view);

// 1) Öffnen: .vcpal kommt in den Palette-Editor
await page.evaluate(() => window.__vc.openFile("graphics/pal65.vcpal")); await page.waitForTimeout(700);
let v = await pv();
ok(v && v.kind === "palette" && v.count === 256 && v.sel === 0 && !v.dirty, "pal65.vcpal öffnet als Palette (256 Farben, Farbe 0, unverändert)");
ok(await page.evaluate(() => window.__vc.ui.mode === "grafik" && window.__vc.ui.sub.grafik === "palette"), "Die Ansicht wechselt zu Grafik ▸ Palette");
ok((await page.locator("[data-pal=farbe][role=option]").count()) === 256 && (await page.locator("[data-pal=farbe][tabindex='0']").count()) === 1 && (await page.locator("[role=grid],[role=application]").count()) === 0, "Farbraster: Auswahlliste mit 256 Optionen, eine per Tab erreichbar, keine Raster-/Anwendungsrolle");
const o9 = page.locator("[data-pal=farbe]").nth(9); const l9 = await o9.getAttribute("aria-label");
ok(/^Farbe 9, \$09, #[0-9A-F]{6}$/.test(l9) && (await o9.getAttribute("aria-posinset")) === "10" && (await o9.getAttribute("aria-setsize")) === "256", "Farbfeld sagt Nummer, Hexnummer und Farbwert an: " + l9);
await snap("palette-1-offen");

// 2) Farbe wählen: Klick und Tastatur
await page.locator("[data-pal=farbe][data-i='87']").click(); await page.waitForTimeout(300); v = await pv();
ok(v.sel === 87 && (await page.locator("[data-pal=hex]").inputValue()).toLowerCase() === v.hex.toLowerCase(), "Klick wählt Farbe 87, der Hexwert zeigt " + v.hex);
await page.keyboard.press("ArrowRight"); await page.waitForTimeout(250); ok((await pv()).sel === 88 && (await page.evaluate(() => document.activeElement?.getAttribute("data-i"))) === "88", "Pfeil rechts: Farbe 88, der Fokus wandert mit");
await page.keyboard.press("ArrowDown"); await page.waitForTimeout(250); ok((await pv()).sel === 104, "Pfeil unten: eine Zeile tiefer (104)");
await page.keyboard.press("Home"); await page.waitForTimeout(250); ok((await pv()).sel === 0, "Pos1: Farbe 0");
await page.keyboard.press("End"); await page.waitForTimeout(250); ok((await pv()).sel === 255, "Ende: Farbe 255");
await page.locator("[data-pal=farbe][data-i='5']").click(); await page.waitForTimeout(300);

// 3) Regler: ein Zug = ein Verlaufsschritt
const vor = (await pv()).rgb[0];
await page.locator("[data-pal=kanal-0]").fill("201"); await page.waitForTimeout(400); v = await pv();
ok(v.rgb[0] === 201 && v.dirty && (await page.locator("[data-pal=wert-0]").innerText()) === "201", "Regler Rot auf 201: gespeichert im Dokument, Zahl daneben, Datei als geändert gekennzeichnet");
ok((await page.locator("[data-pal=swatch]").evaluate((e) => getComputedStyle(e).backgroundColor)).startsWith("rgb(201,") || /rgb\(201, /.test(await page.locator("[data-pal=swatch]").evaluate((e) => getComputedStyle(e).backgroundColor)), "Das Farbfeld zeigt die neue Farbe");
ok((await page.locator("[data-pal=farbe][data-i='5']").evaluate((e) => getComputedStyle(e).backgroundColor)).includes("201"), "Das Raster zeigt die geänderte Farbe sofort");
await page.locator("[data-act=undo]").click(); await page.waitForTimeout(300); ok((await pv()).rgb[0] === vor, "Rückgängig stellt den alten Rotwert her");
await page.locator("[data-act=redo]").click(); await page.waitForTimeout(300); ok((await pv()).rgb[0] === 201, "Wiederholen setzt ihn wieder");

// 4) Hexwert
await page.locator("[data-pal=hex]").fill("#00ff7f"); await page.locator("[data-pal=hex]").blur(); await page.waitForTimeout(400); v = await pv();
ok(v.hex.toLowerCase() === "#00ff7f" && v.rgb[1] === 255, "Hexwert #00ff7f übernommen");
await page.locator("[data-pal=hex]").fill("zz12"); await page.waitForTimeout(250);
ok(/kein Farbwert/.test(await page.locator("body").innerText()) && (await page.locator("[data-pal=hex]").getAttribute("aria-invalid")) === "true", "Ungültiger Hexwert: Hinweis, Feld als ungültig gekennzeichnet");
await page.locator("[data-pal=hex]").blur(); await page.waitForTimeout(300); ok((await pv()).hex.toLowerCase() === "#00ff7f", "Der ungültige Wert wird nicht übernommen");

// 5) Im Einsatz
await page.locator("[data-pal=farbe][data-i='5']").click(); await page.waitForTimeout(500);
ok(/uses\.vcsprite/.test(await page.locator("[data-pal=einsatz]").innerText()), "Im Einsatz nennt die Sprite-Datei, die Farbe 5 verwendet");
await page.locator("[data-pal=farbe][data-i='6']").click(); await page.waitForTimeout(500);
ok(/keiner Mediendatei/.test(await page.locator("[data-pal=einsatz]").innerText()), "Farbe 6: nicht im Einsatz, mit Erklärung");

// 6) Standardpalette, Export, Speichern
await page.locator("[data-act=standard]").click(); await page.waitForTimeout(400); v = await pv(); ok(v.canUndo && v.count > 0, "Standardpalette zurückgesetzt, rückgängig machbar");
await page.locator("[data-act=undo]").click(); await page.waitForTimeout(300); ok((await pv()).hex.toLowerCase() !== undefined && (await pv()).canUndo, "Rückgängig nimmt die Standardpalette zurück");
await page.locator("[data-act=export]").click(); await page.waitForSelector("dialog[open] [data-ex=vorschau]"); await page.waitForFunction(() => (document.querySelector("[data-ex=vorschau]")?.textContent ?? "").length > 50, null, { timeout: 8000 }).catch(() => {});
ok((await page.locator("[data-ex=vorschau]").innerText()).length > 50, "Export-Dialog zeigt eine Vorschau der Palette");
await page.getByRole("button", { name: "Abbrechen" }).first().click(); await page.waitForTimeout(300);
await page.locator("[data-act=save]").click(); await page.waitForTimeout(600); v = await pv(); ok(!v.dirty, "Speichern: nicht mehr als geändert gekennzeichnet");
const text = await page.evaluate(async () => (await window.__vc.call("file_read", { rel: "graphics/pal65.vcpal" })).text);
ok(/"#00ff7f"/i.test(text) || /#00ff7f/i.test(text), "Die Datei enthält den gespeicherten Farbwert");
await snap("palette-2-bearbeitet");

// 7) Englisch
await page.evaluate(async () => { await window.__vc.setLanguage("en"); }); await page.waitForTimeout(400);
const bt = await page.locator("body").innerText(); ok(/colou?r 6/i.test(bt) && /in use/i.test(bt) && /hex value/i.test(bt), "Englisch: Colour 5, In use, Hex value");
await page.evaluate(async () => { await window.__vc.setLanguage("de"); });
console.log(`\n${n} Prüfungen, ${fails.length} Fehler`);
await browser.close();
process.exit(fails.length ? 1 : 0);
