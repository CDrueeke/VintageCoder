// SPDX-License-Identifier: GPL-3.0-or-later
// Ablauftest Kachel-Editor (E8): Playwright (WebKit) gegen die Prüfbrücke des echten Kerns.
// Aufruf: node tools/e2e-kacheln.mjs <Projektordner (Kopie von examples/c64/gfx_demo; wird verändert!)> [Bilderordner]
import { webkit } from "playwright";
import { mkdirSync } from "node:fs";
import { join } from "node:path";

if (process.platform === "win32" && !process.env.VC_E2E_WIN) { console.log("übersprungen unter Windows\n0 Prüfungen, 0 Fehler"); process.exit(0); }
const [proj, shots] = process.argv.slice(2);
if (!proj) { console.error("Aufruf: node tools/e2e-kacheln.mjs <Projektordner> [Bilder]"); process.exit(2); }
if (shots) mkdirSync(shots, { recursive: true });
const browser = await webkit.launch();
const page = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
const fails = []; let n = 0;
const ok = (c, m) => { n++; if (!c) { fails.push(m); console.log("FEHLER:", m); } else console.log("ok:", m); };
page.on("pageerror", (e) => { fails.push("Seitenfehler " + e.message); console.log("Seitenfehler", e.message); });
const snap = async (name) => { if (shots) await page.screenshot({ path: join(shots, name + ".png") }); };
await page.goto("http://127.0.0.1:1420/?bridge=17420"); await page.waitForFunction(() => window.__vc && window.__vc.ui.ready);
await page.evaluate(async (p) => { await window.__vc.setLanguage("de"); await window.__vc.openProject(p); }, proj); await page.waitForTimeout(900);
const kv = () => page.evaluate(() => window.__vc.kc.view);
const warte = () => page.waitForTimeout(350);

// 1) Öffnen
await page.evaluate(() => window.__vc.openFile("graphics/level_tiles.vctiles")); await page.waitForTimeout(900);
let v = await kv();
ok(v && v.kind === "tileset" && v.count === 4 && v.tw === 2 && v.th === 2 && !v.dirty, "level_tiles.vctiles öffnet als Kachelsatz (4 Kacheln, 2 × 2 Zellen)");
ok(await page.evaluate(() => window.__vc.ui.mode === "grafik" && window.__vc.ui.sub.grafik === "kacheln"), "Die Ansicht wechselt zu Grafik ▸ Kacheln");
ok((await page.locator("[data-kc=kachel][role=option]").count()) === 4 && (await page.locator("[data-kc=zelle][role=option]").count()) === 4 && (await page.locator("[data-kc=zeichen][role=option]").count()) === 256 && (await page.locator("[data-kc=liste] [role=grid], [data-kc=gross] [role=grid], [data-kc=tafel] [role=grid]").count()) === 0, "Kachelliste, Zellen und Zeichentafel sind Auswahllisten mit Optionen (keine Raster-Rolle)");
ok(/Kachel 1, stein, \d+ mal verwendet/.test(await page.locator("[data-kc=kachel]").nth(1).getAttribute("aria-label")), "Kachel sagt Nummer, Name und Verwendung an: " + (await page.locator("[data-kc=kachel]").nth(1).getAttribute("aria-label")));
ok((await page.locator("[data-kc=fill]").innerText()).includes("von 256"), "Füllstand des Zielzeichensatzes: " + (await page.locator("[data-kc=fill]").innerText()));
await snap("kacheln-1-offen");

// 2) Kachel wählen: Klick, Pfeile
await page.locator("[data-kc=kachel]").nth(2).click(); await warte(); ok((await kv()).sel === 2, "Klick wählt Kachel 2 (gras)");
await page.keyboard.press("ArrowRight"); await warte(); ok((await kv()).sel === 3 && (await page.evaluate(() => document.activeElement?.getAttribute("data-i"))) === "3", "Pfeil rechts: Kachel 3, der Fokus wandert mit");
await page.keyboard.press("Home"); await warte(); ok((await kv()).sel === 0, "Pos1: Kachel 0");
await page.locator("[data-kc=kachel]").nth(2).click(); await warte();

// 3) Zeichen aus der Tafel in die markierte Zelle
await page.locator("[data-kc=zelle]").nth(1).click(); await warte();
await page.locator("[data-kc=zeichen][data-c='65']").click(); await warte(); v = await kv();
ok(v.brush.code === 65 && v.cells[1].code === 65 && v.dirty, "Zeichen 65 aus der Tafel wählt den Pinsel und setzt die markierte Zelle");
// Zug über mehrere Zellen: ein Rückgängig-Schritt
await page.locator("[data-kc=zeichen][data-c='66']").click(); await warte();
const zb = await Promise.all([0, 1, 2, 3].map((k) => page.locator(`[data-kc=zelle][data-k='${k}']`).boundingBox()));
const mitte = (b) => [b.x + b.width / 2, b.y + b.height / 2];
await page.mouse.move(...mitte(zb[0])); await page.mouse.down(); await page.mouse.move(...mitte(zb[1]), { steps: 4 }); await page.mouse.move(...mitte(zb[3]), { steps: 4 }); await page.mouse.up(); await warte(); v = await kv();
ok(v.cells[0].code === 66 && v.cells[1].code === 66 && v.cells[3].code === 66, "Ziehen über Zellen setzt das Zeichen in allen überstrichenen Zellen");
await page.locator("[data-act=undo]").click(); await warte(); v = await kv(); ok(v.cells[0].code !== 66 && v.cells[3].code !== 66, "Ein Rückgängig nimmt den ganzen Zug zurück");
await page.locator("[data-act=redo]").click(); await warte();
// Pipette (Alt+Klick) und Tastatur in den Zellen
await page.locator("[data-kc=zeichen][data-c='90']").click(); await warte();
await page.locator("[data-kc=zelle]").nth(2).click({ modifiers: ["Alt"] }); await warte(); v = await kv(); ok(v.brush.code === v.cells[2].code, "Alt+Klick nimmt das Zeichen der Zelle als Pinsel auf (Pipette)");
await page.locator("[data-kc=zelle]").nth(0).focus(); await page.keyboard.press("ArrowRight"); await page.keyboard.press("ArrowDown"); await warte(); v = await kv(); ok(v.cur === 3, "Pfeiltasten bewegen den Zellcursor (Zelle 3)");
const vorEnter = v.cells[3].code; await page.locator("[data-kc=zeichen][data-c='70']").click(); await warte(); await page.locator("[data-kc=zelle]").nth(3).focus(); await page.keyboard.press("Enter"); await warte(); v = await kv(); ok(v.cells[3].code === 70, "Enter setzt das Zeichen in die Zelle (vorher " + vorEnter + ")");

// 4) Zellen-Aktionen
const vorher = (await kv()).cells.map((c) => c.code);
await page.locator("[data-act=fliph]").click(); await warte(); const nach = (await kv()).cells.map((c) => c.code); ok(JSON.stringify(nach) === JSON.stringify([vorher[1], vorher[0], vorher[3], vorher[2]]), "Spiegeln waagerecht vertauscht die Zellen links und rechts");
await page.locator("[data-act=fliph]").click(); await warte();
await page.locator("[data-act=copy]").click(); await page.locator("[data-act=clear]").click(); await warte(); ok((await kv()).cells.every((c) => c.code === 0), "Leeren setzt alle Zellen auf Zeichen 0");
await page.locator("[data-act=paste]").click(); await warte(); ok(JSON.stringify((await kv()).cells.map((c) => c.code)) === JSON.stringify(vorher), "Einfügen stellt die kopierten Zellen wieder her");
await page.locator("[data-act=rotate]").click(); await page.locator("[data-act=roll-r]").click(); await warte(); ok((await kv()).dirty, "Drehen und Rollen sind möglich");

// 5) Name, Farbe, Eigenschaften
await page.locator("[data-kc=name]").fill("wiese"); await page.locator("[data-kc=name]").blur(); await warte(); ok((await kv()).tile.name === "wiese" && (await page.locator("[data-kc=kachel]").nth(2).innerText()).includes("wiese"), "Umbenennen: die Liste zeigt den neuen Namen");
await page.locator("[aria-label='Farbe der Kachel'] [data-idx='5']").click(); await warte(); ok((await kv()).tile.col === 5, "Kachelfarbe 5 gewählt");
await page.locator("[data-kc=prop] button[role=switch]").first().click(); await warte(); ok(((await kv()).tile.props & 1) === 1, "Eigenschaft „solid“ eingeschaltet");
await page.locator("[data-kc=prop-neu]").fill("tod"); await page.locator("[data-act=prop-neu]").click(); await warte(); ok((await kv()).propDefs.length === 2 && (await page.locator("[data-kc=prop]").count()) === 2, "Neue Eigenschaft „tod“ angelegt");
await page.getByRole("radio", { name: "Je Zelle" }).click(); await warte(); v = await kv(); ok(v.colorMode === "cell" && (await page.getByRole("radio", { name: "Nur Farbe" }).count()) === 1, "Farbe je Zelle: der Pinselmodus (Zeichen, Farbe, beides) erscheint");
await page.getByRole("radio", { name: "Je Kachel" }).click(); await warte();

// 6) Größe, Rückfrage
await page.locator("[data-kc=gw]").fill("3"); await page.locator("[data-kc=gh]").fill("3"); await page.locator("[data-act=size]").click(); await warte(); v = await kv(); ok(v.tw === 3 && v.th === 3 && v.cells.length === 9, "Größe 3 × 3: aufgefüllt, ohne Rückfrage");
await page.locator("[data-act=undo]").click(); await warte();
await page.locator("[data-kc=gw]").fill("1"); await page.locator("[data-kc=gh]").fill("1"); await page.locator("[data-act=size]").click(); await warte(); ok(/gehen belegte Zellen verloren/.test(await page.locator("[data-kc=frage]").innerText()), "Größe 1 × 1 würde Zellen verlieren: Rückfrage");
await page.locator("[data-ask=no]").click(); await warte(); ok((await kv()).tw === 2, "Abbrechen lässt die Größe");

// 7) Kacheln: neu, duplizieren, zusammenführen, löschen mit Rückfrage
await page.locator("[data-act=tile-new]").click(); await warte(); ok((await kv()).count === 5 && (await kv()).sel === 4, "Neue Kachel angelegt und gewählt");
await page.locator("[data-act=tile-dup]").click(); await warte(); v = await kv(); ok(v.count === 6 && v.tiles[5].double === 4, "Duplikat: erkannt (gleich wie Kachel 4)");
await page.locator("[data-act=tile-merge]").click(); await warte(); ok((await kv()).count === 5, "Zusammenführen entfernt das Duplikat");
await page.locator("[data-kc=kachel]").nth(1).click(); await warte(); await page.locator("[data-act=tile-delete]").click(); await warte();
ok(/mal in Bildschirmbanken verwendet/.test(await page.locator("[data-kc=frage]").innerText()), "Kachel 1 wird in Bänken benutzt: Löschen fragt nach");
await page.locator("[data-ask=no]").click(); await warte(); ok((await kv()).count === 5, "Abbrechen: die Kachel bleibt");

// 8) Speichern, Export (Zusammenführen hat den Satz wegen der Bänke schon gespeichert: erst wieder etwas ändern)
ok(!(await kv()).dirty, "Zusammenführen mit Bank-Anpassung speichert den Satz");
await page.locator("[data-kc=name]").fill("wiese2"); await page.locator("[data-kc=name]").blur(); await warte(); ok((await kv()).dirty, "Eine neue Änderung macht den Satz wieder ungespeichert");
await page.locator("[data-act=save]").click(); await page.waitForTimeout(600); ok(!(await kv()).dirty, "Speichern: nicht mehr als geändert gekennzeichnet");
await page.locator("[data-act=export]").click(); await page.waitForSelector("dialog[open] [data-ex=vorschau]"); await page.waitForFunction(() => (document.querySelector("[data-ex=vorschau]")?.textContent ?? "").length > 50, null, { timeout: 8000 }).catch(() => {});
ok((await page.locator("[data-ex=vorschau]").innerText()).length > 50, "Export-Dialog zeigt eine Vorschau des Kachelsatzes");
await page.getByRole("button", { name: "Abbrechen" }).first().click(); await warte();
const text = await page.evaluate(async () => (await window.__vc.call("file_read", { rel: "graphics/level_tiles.vctiles" })).text); ok(/wiese2/.test(text) && /tod/.test(text), "Die Datei enthält Name und neue Eigenschaft");
await snap("kacheln-2-bearbeitet");

// 9) Englisch
await page.evaluate(async () => { await window.__vc.setLanguage("en"); }); await warte(); const bt = await page.locator("body").innerText();
ok(/tile set/i.test(bt) && /properties/i.test(bt) && /target charset/i.test(bt), "Englisch: Tile set, Properties, Target charset");
await page.evaluate(async () => { await window.__vc.setLanguage("de"); });
console.log(`\n${n} Prüfungen, ${fails.length} Fehler`);
await browser.close();
process.exit(fails.length ? 1 : 0);
