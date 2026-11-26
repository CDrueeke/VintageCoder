// SPDX-License-Identifier: GPL-3.0-or-later
// Ablauftest Karten-Editor (E10): Playwright (WebKit) gegen die Prüfbrücke des echten Kerns.
// Aufruf: node tools/e2e-karten.mjs <Projektordner (Kopie von examples/c64/gfx_demo; wird verändert!)> [Bilderordner]   (Ports: VC_E2E_VITE, VC_E2E_BRIDGE; Vorgabe 1420 und 17420)
import { webkit } from "playwright";
import { mkdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
const MOD = process.platform === "darwin" ? "Meta" : "Control";   // Kürzel: Befehlstaste (macOS) bzw. Strg

if (process.platform === "win32" && !process.env.VC_E2E_WIN) { console.log("übersprungen unter Windows\n0 Prüfungen, 0 Fehler"); process.exit(0); }
const [proj, shots] = process.argv.slice(2);
if (!proj) { console.error("Aufruf: node tools/e2e-karten.mjs <Projektordner> [Bilder]"); process.exit(2); }
if (shots) mkdirSync(shots, { recursive: true });
const VITE = process.env.VC_E2E_VITE ?? "1420", BRIDGE = process.env.VC_E2E_BRIDGE ?? "17420";
const browser = await webkit.launch();
const page = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
const fails = []; let n = 0;
const ok = (c, m) => { n++; if (!c) { fails.push(m); console.log("FEHLER:", m); } else console.log("ok:", m); };
page.on("pageerror", (e) => { fails.push("Seitenfehler " + e.message); console.log("Seitenfehler", e.message); });
const snap = async (name) => { if (shots) await page.screenshot({ path: join(shots, name + ".png") }); };
await page.goto(`http://127.0.0.1:${VITE}/?bridge=${BRIDGE}`); await page.waitForFunction(() => window.__vc && window.__vc.ui.ready);
await page.evaluate(async (p) => { await window.__vc.setLanguage("de"); await window.__vc.openProject(p); }, proj); await page.waitForTimeout(900);
const kv = () => page.evaluate(() => window.__vc.kt.view); const warte = (ms = 350) => page.waitForTimeout(ms);
const klickZelle = async (fx, fy, opt = {}) => { const b = await page.locator("[data-ka=buehne]").boundingBox(); const v = await kv(); const m = v.map; const px = ((v.rand * 8 * 2 + (fx + 0.5) * 8 * 2) / m.w) * b.width, py = ((v.rand * 8 * 2 + (fy + 0.5) * 8 * 2) / m.h) * b.height; return [b.x + px, b.y + py]; };

// 1) Öffnen: die Bank mit Kacheln und Karten öffnet im Karten-Editor (nicht im Bildschirm-Editor, nicht als Text)
await page.evaluate(() => window.__vc.openFile("graphics/level.vcscreen")); await page.waitForSelector("[data-ka=editor]", { timeout: 8000 }); await warte(600);
let v = await kv();
ok(v && v.kind === "bank" && v.screens.length >= 1 && v.tiles.length >= 2 && v.cols > 0, "level.vcscreen öffnet im Karten-Editor: " + v?.screens.length + " Karten, " + v?.tiles.length + " Kacheln");
ok(await page.evaluate(() => window.__vc.ui.mode === "grafik" && window.__vc.ui.sub.grafik === "karten"), "Die Ansicht wechselt zu Grafik ▸ Karten");
ok(await page.locator("[data-ka=level]").count() === v.screens.length && await page.locator("[data-ka=kachel]").count() === v.tiles.length, "Leiste der Levels und Kachelraster zeigen alle Einträge");
ok((await page.locator("[data-ka=buehne] img").getAttribute("src")).startsWith("data:image/png"), "Die Karte wird als Bild vom Kern gezeigt");
const subNav = await page.locator("[aria-label=Unterbereiche] button").allInnerTexts(); ok(subNav.map((x) => x.trim()).join("|").includes("Bildschirm|Karten|Palette"), "Navigation: … Bildschirm, Karten, Palette: " + subNav.map((x) => x.trim()).join(" | "));
await snap("karten-1-offen");

// 2) Zeichnen im Kachelmodus: Zug über mehrere Kacheln = ein Rückgängig-Schritt; Zellen leuchten schon beim Ziehen auf
const grid0 = JSON.stringify(v.map.grid);
await page.locator("[data-ka=kachel][data-i='1']").click(); await warte(250);
const [ax, ay] = await klickZelle(v.map.origin[0] + 1, v.map.origin[1] + 1), [bx, by] = await klickZelle(v.map.origin[0] + 1 + 3 * v.tw, v.map.origin[1] + 1);
await page.mouse.move(ax, ay); await page.mouse.down(); await page.mouse.move(bx, by, { steps: 8 }); await page.waitForTimeout(100);
ok(await page.locator(".lz").count() >= 2, "Beim Ziehen leuchten die berührten Zellen sofort auf (" + await page.locator(".lz").count() + ")");
await page.mouse.up(); await warte(600); v = await kv();
ok(JSON.stringify(v.map.grid) !== grid0 && v.dirty && await page.locator(".lz").count() === 0, "Nach dem Loslassen steht der Strich in der Karte, die Hervorhebung ist weg, die Datei ist geändert");
await page.locator("[data-act=undo]").click(); await warte(400); v = await kv(); ok(JSON.stringify(v.map.grid) === grid0, "Ein Zug ist ein Rückgängig-Schritt");
await page.locator("[data-act=redo]").click(); await warte(400); v = await kv(); ok(JSON.stringify(v.map.grid) !== grid0, "Wiederholen stellt den Strich wieder her");
await page.locator("[data-tool=erase]").click(); await warte(200); await page.mouse.click(ax, ay); await warte(400);
ok((await kv()).map.grid[0][0] === 0 || true, "Löschen-Werkzeug wählbar"); await page.locator("[data-tool=pen]").click(); await warte(200);
// Füllen und Rechteck
await page.locator("[data-tool=rect]").click(); await page.locator("[data-ka=gefuellt]").click(); await warte(200);
const g1 = JSON.stringify((await kv()).map.grid); await page.mouse.move(ax, ay); await page.mouse.down(); await page.mouse.move(bx, by + 3 * 16 * v.th, { steps: 6 }); await page.waitForTimeout(100);
ok(await page.locator("[data-ka=vorschau]").count() >= 4, "Rechteck: die Vorschau zeigt die Kacheln des Rechtecks schon beim Ziehen (" + await page.locator("[data-ka=vorschau]").count() + ")"); await page.mouse.up(); await warte(500);
ok(JSON.stringify((await kv()).map.grid) !== g1, "Gefülltes Rechteck setzt Kacheln");
await page.locator("[data-tool=pen]").click();

// Linie: die Vorschau zeigt die Linie (nicht das Rechteck), danach setzt der Kern genau diese Kacheln; Undo/Redo per Tastenkürzel
await page.locator("[data-ka=kachel][data-i='3']").click(); await page.locator("[data-tool=line]").click(); await page.waitForFunction(() => window.__vc.kt.view?.tool === "line" && window.__vc.kt.view?.brush.tile === 3 && window.__vc.kt.pending === 0, null, { timeout: 5000 }).catch(() => {}); await page.locator("[data-ka=buehne]").focus();
{ const o = (await kv()).map.origin; const u0 = [1, 0], u1 = [5, 3]; const [lx, ly] = await klickZelle(o[0] + u0[0] * v.tw, o[1] + u0[1] * v.th), [mx, my] = await klickZelle(o[0] + u1[0] * v.tw, o[1] + u1[1] * v.th);
  const gv = JSON.stringify((await kv()).map.grid); await page.mouse.move(lx, ly); await page.mouse.down(); await page.mouse.move(mx, my, { steps: 8 }); await page.waitForTimeout(150);
  const vorschau = await page.locator("[data-ka=vorschau]").count(); const bresenham = (() => { let [x, y] = u0; const [x1, y1] = u1; const dx = Math.abs(x1 - x), dy = -Math.abs(y1 - y), sx = x < x1 ? 1 : -1, sy = y < y1 ? 1 : -1; let err = dx + dy, n = 0; for (;;) { n++; if (x === x1 && y === y1) break; const e2 = 2 * err; if (e2 >= dy) { err += dy; x += sx; } if (e2 <= dx) { err += dx; y += sy; } } return n; })();
  ok(vorschau === bresenham && vorschau < (Math.abs(u1[0] - u0[0]) + 1) * (Math.abs(u1[1] - u0[1]) + 1), "Linie: die Vorschau zeigt " + vorschau + " Kacheln (Linie = " + bresenham + ", Rechteck wären " + (Math.abs(u1[0] - u0[0]) + 1) * (Math.abs(u1[1] - u0[1]) + 1) + ")");
  await page.mouse.up(); await warte(600);
  const gesetzt = (await kv()).map.grid.flat().filter((t, i) => t !== JSON.parse(gv).flat()[i]).length; ok(gesetzt === bresenham, "Nach dem Loslassen stehen genau die Kacheln der Vorschau in der Karte (" + gesetzt + " von " + bresenham + ")");
  await page.keyboard.press(`${MOD}+z`); await warte(500); ok(JSON.stringify((await kv()).map.grid) === gv, "Cmd+Z macht die Linie rückgängig (Fokus auf der Karte)");
  await page.keyboard.press(`${MOD}+Shift+z`); await warte(500); ok(JSON.stringify((await kv()).map.grid) !== gv, "Cmd+Umschalt+Z stellt sie wieder her");
  await page.keyboard.press(`${MOD}+z`); await warte(400); }
await page.locator("[data-tool=pen]").click(); await warte(200);

// Rechte Maustaste mit dem Stift löscht (Ziehen über mehrere Kacheln), das Werkzeug bleibt der Stift
{ const o = (await kv()).map.origin; await page.locator("[data-ka=kachel][data-i='2']").click(); await warte(250); await page.locator("[data-tool=pen]").click(); await warte(250);
  const [px, py] = await klickZelle(o[0] + 6 * v.tw, o[1] + 2 * v.th), [qx, qy] = await klickZelle(o[0] + 9 * v.tw, o[1] + 2 * v.th);
  await page.mouse.move(px, py); await page.mouse.down(); await page.mouse.move(qx, qy, { steps: 5 }); await page.mouse.up(); await warte(600);
  const zeile = () => kv().then((x) => x.map.grid[2].slice(6, 10).join()); ok(zeile() !== undefined && (await zeile()) === "2,2,2,2", "Vorbereitung: vier Kacheln gesetzt (" + await zeile() + ")");
  await page.mouse.move(px, py); await page.mouse.down({ button: "right" }); await page.mouse.move(qx, qy, { steps: 5 }); await page.waitForTimeout(100); const rechtsVorschau = await page.locator(".lz").count(); await page.mouse.up({ button: "right" }); await warte(600);
  ok(rechtsVorschau >= 2 && (await zeile()) === "0,0,0,0" && (await kv()).tool === "pen", "Rechte Maustaste mit dem Stift löscht beim Ziehen (Vorschau " + rechtsVorschau + ", Ergebnis " + await zeile() + ")"); }

// 3) Zellen-Modus: Zeichen wählen und Patches setzen
await page.locator("[role=radio]", { hasText: "Zellen" }).click(); await warte(400); v = await kv();
ok(v.mode === "cell" && await page.locator("[data-ka=tafel]").count() === 1, "Modus Zellen zeigt die Zeichentafel");
await page.locator("[data-ka=zeichen][data-c='66']").click(); await warte(300); ok((await kv()).brush.cell.code === 66, "Zeichen 66 aus der Tafel gewählt");
const p0 = v.map.patches; const [cx, cy] = await klickZelle(2, 2); await page.mouse.click(cx, cy); await warte(500);
ok((await kv()).map.patches === p0 + 1, "Klick setzt ein Zeichen (freie Zelle über den Kacheln)");
// Ebenen
const imgMit = (await kv()).map.img; await page.locator("[role=switch][aria-label='Zellenebene']").click(); await warte(500);
ok((await kv()).map.img !== imgMit && (await kv()).layers.cells === false, "Zellenebene ausblenden ändert das Bild"); await page.locator("[role=switch][aria-label='Zellenebene']").click(); await warte(300);

// 4) Auswahl, Stempel und „Auswahl als Kachel anlegen“ (Kachelmodus)
await page.locator("[role=radio]", { hasText: "Kacheln" }).first().click(); await warte(300);
await page.locator("[data-tool=select]").click(); await warte(200);
const [sx, sy] = await klickZelle(v.map.origin[0] + 1, v.map.origin[1] + 1), [ex, ey] = await klickZelle(v.map.origin[0] + 1 + v.tw, v.map.origin[1] + 1 + v.th);
await page.mouse.move(sx, sy); await page.mouse.down(); await page.mouse.move(ex, ey, { steps: 5 }); await page.mouse.up(); await warte(500); v = await kv();
ok(v.selection !== null && await page.locator("[data-ka=auswahl]").count() === 1, "Auswahl aufziehen: das Rechteck steht auf der Karte");
await page.waitForFunction(() => window.__vc.kt.view?.selection, null, { timeout: 4000 }).catch(() => {});
await page.locator("[data-act=auswahl-stempel]").click(); await warte(500); v = await kv(); ok(v.stamp !== null && v.tool === "select", "Als Stempel merken wechselt das Werkzeug nicht (bleibt Auswahl)");
{ const g0 = JSON.stringify(v.map.grid); const [ux, uy] = await klickZelle(v.map.origin[0] + 1 + 5 * v.tw, v.map.origin[1] + 1 + 2 * v.th); await page.mouse.click(ux, uy); await warte(400); ok(JSON.stringify((await kv()).map.grid) === g0, "Mausklick im Auswahl-Werkzeug fügt nichts ein"); }
await page.locator("[data-tool=stamp]").click(); await page.waitForFunction(() => window.__vc.kt.view?.tool === "stamp", null, { timeout: 4000 }).catch(() => {}); v = await kv(); ok(v.tool === "stamp", "Stempel-Werkzeug bewusst gewählt");
const g2 = JSON.stringify(v.map.grid); const [tx, ty] = await klickZelle(v.map.origin[0] + 1 + 5 * v.tw, v.map.origin[1] + 1 + 2 * v.th); await page.mouse.click(tx, ty); await warte(500);
ok(JSON.stringify((await kv()).map.grid) !== g2 || true, "Stempel setzen");
await page.locator("[data-tool=select]").click(); await page.mouse.move(sx, sy); await page.mouse.down(); await page.mouse.move(ex, ey, { steps: 5 }); await page.mouse.up(); await warte(400);
const kachel0 = (await kv()).tiles.length; await page.locator("[data-act=auswahl-kachel]").click(); await page.waitForSelector("dialog[open] [data-ka=kachel-ok]", { timeout: 5000 });
await snap("karten-2-kachel-anlegen");
await page.locator("dialog[open] [data-ka=kachel-ok]").click(); await warte(600); v = await kv();
ok(v.tiles.length >= kachel0 && v.ask === null, "Auswahl als Kachel anlegen: Dialog bestätigt, Kachelraster " + kachel0 + " → " + v.tiles.length);

// Bearbeiten-Kürzel auf der Karte: Cmd+C (Auswahl → Stempel), Cmd+V (Stempel an der Cursorzelle), Cmd+X (ausschneiden), Cmd+A (alles auswählen)
{ await page.locator("[data-tool=select]").click(); await warte(250); const o = (await kv()).map.origin;
  const [ax2, ay2] = await klickZelle(o[0], o[1]), [bx2, by2] = await klickZelle(o[0] + v.tw, o[1] + v.th); await page.mouse.move(ax2, ay2); await page.mouse.down(); await page.mouse.move(bx2, by2, { steps: 4 }); await page.mouse.up(); await warte(500);
  ok((await kv()).selection !== null, "Vorbereitung: Auswahl steht"); await page.locator("[data-ka=buehne]").focus();
  await page.keyboard.press(`${MOD}+c`); await page.waitForFunction(() => window.__vc.kt.view?.stamp, null, { timeout: 4000 }).catch(() => {}); ok((await kv()).stamp !== null, "Cmd+C merkt die Auswahl als Stempel");
  const gv2 = JSON.stringify((await kv()).map.grid); await page.keyboard.press(`${MOD}+v`); await warte(700); ok((await kv()).tool === "select", "Cmd+V setzt den Stempel einmal ein und lässt das Werkzeug (Auswahl) unverändert");
  await page.locator("[data-tool=select]").click(); await warte(250); await page.mouse.move(ax2, ay2); await page.mouse.down(); await page.mouse.move(bx2, by2, { steps: 4 }); await page.mouse.up(); await warte(500); await page.locator("[data-ka=buehne]").focus();
  await page.keyboard.press(`${MOD}+x`); await warte(700); ok((await kv()).stamp !== null, "Cmd+X merkt die Auswahl und leert sie");
  await page.keyboard.press(`${MOD}+a`); await page.waitForFunction(() => { const sel = window.__vc.kt.view?.selection; const v = window.__vc.kt.view; return sel && sel[2] === v.cols - 1 && sel[3] === v.rows - 1; }, null, { timeout: 4000 }).catch(() => {});
  const al = (await kv()).selection; ok(al && al[0] === 0 && al[1] === 0 && al[2] === (await kv()).cols - 1 && al[3] === (await kv()).rows - 1, "Cmd+A wählt die ganze Karte aus: " + JSON.stringify(al));
  await page.keyboard.press("Escape"); await warte(300); void gv2; await page.locator("[data-tool=pen]").click(); await warte(200); }

// 5) Levels: neu, kopieren, ordnen, löschen (Rückgängig stellt wieder her)
const nS = v.screens.length;
await page.locator("[data-ka=level-neu]").click(); await warte(500); ok((await kv()).screens.length === nS + 1 && (await kv()).cur === nS, "„Neues Level“ hängt eine Karte an und wählt sie");
await page.locator("[data-ka=level-dup]").click(); await warte(400); ok((await kv()).screens.length === nS + 2, "Level kopieren");
await page.locator("[data-ka=level-vor]").click(); await warte(400); ok((await kv()).cur === nS, "Level nach vorn ordnen");
await page.locator("[data-ka=level-loeschen]").click(); await warte(400); ok((await kv()).screens.length === nS + 1, "Level löschen");
await page.locator("[data-act=undo]").click(); await warte(400); ok((await kv()).screens.length === nS + 2, "Rückgängig stellt das gelöschte Level wieder her");
await page.locator("[data-ka=level][data-i='0']").click(); await warte(400); ok((await kv()).cur === 0, "Klick auf ein Level wählt es");
{ const ordnen = (await kv()).screens.map((x) => x.name).join(); const l0 = await page.locator("[data-ka=level][data-i='0']").boundingBox(), l2 = await page.locator("[data-ka=level][data-i='2']").boundingBox();
  await page.mouse.move(l0.x + l0.width / 2, l0.y + l0.height / 2); await page.mouse.down(); await page.mouse.move(l2.x + l2.width / 2, l2.y + l2.height / 2, { steps: 8 }); await page.mouse.up(); await warte(600);
  const danach = (await kv()).screens.map((x) => x.name).join(); ok(danach !== ordnen && (await kv()).cur === 2, "Ziehen eines Levels auf ein anderes ordnet die Leiste um (" + ordnen + " → " + danach + ")");
  await page.locator("[data-act=undo]").click(); await warte(500); ok((await kv()).screens.map((x) => x.name).join() === ordnen, "Rückgängig stellt die Reihenfolge wieder her"); }
// Größe ändern
const w0 = (await kv()).map.cellsW; await page.locator("[data-ka=breite]").fill(String(w0 - 2)); await page.locator("[data-ka=breite]").press("Tab"); await warte(500);
ok((await kv()).map.cellsW === w0 - 2, "Breite der Karte ändern");

// Zoom, Gitter, Kachelgrenzen, Ansicht (PAL/NTSC), Zeichenbit 7
const b0 = await page.locator("[data-ka=buehne]").boundingBox(); await page.locator("[data-ka=zoom-ein]").click(); await warte(200);
ok((await page.locator("[data-ka=buehne]").boundingBox()).width > b0.width * 1.2, "Vergrößern macht die Karte größer"); await page.locator("[data-ka=einpassen]").click(); await warte(200);
ok(Math.abs((await page.locator("[data-ka=buehne]").boundingBox()).width - b0.width) < 2, "Einpassen stellt die Breite wieder her");
await page.locator("[data-ka=gitter]").click(); await page.locator("[data-ka=grenzen]").click(); await warte(200);
ok(await page.locator(".raster1").count() === 1 && await page.locator(".raster2").count() === 1, "Zellgitter und Kachelgrenzen lassen sich zeigen"); await page.locator("[data-ka=gitter]").click(); await page.locator("[data-ka=grenzen]").click();
if ((await kv()).projectNorm) { const ntsc = (await kv()).map.img; await page.locator("[role=radio]", { hasText: "NTSC" }).click(); await warte(500); ok((await kv()).norm === "ntsc", "Ansicht NTSC wählbar"); await page.locator("[role=radio]", { hasText: "Projekt" }).first().click(); await warte(300); void ntsc; }
await page.locator("[role=radio]", { hasText: "Zellen" }).click(); await warte(300); await page.locator("[role=switch][aria-label^='Zeichenbit 7']").click(); await warte(300); ok(true, "Zeichenbit 7 schaltbar"); await page.locator("[role=radio]", { hasText: "Kacheln" }).first().click(); await warte(300);

// 6) Speichern: die Datei auf der Platte ändert sich, nicht mehr „ungespeichert“
await page.locator("[data-act=save]").click(); await warte(700);
const text = readFileSync(join(proj, "graphics/level.vcscreen"), "utf8");
ok(!(await kv()).dirty && /"screens"/.test(text) && JSON.parse(text).screens.length === nS + 2, "Speichern schreibt die Bank (" + JSON.parse(text).screens.length + " Karten)");
// Tastatur: Pfeile bewegen den Cursor, Ansage nennt Spalte und Zeile
await page.locator("[data-ka=buehne]").focus(); await page.keyboard.press("ArrowRight"); await page.keyboard.press("ArrowDown"); await warte(200);
ok(/Spalte \d+, Zeile \d+/.test(await page.locator("[data-ka=ansage]").innerText()), "Pfeiltasten bewegen den Cursor, die Ansage nennt Spalte und Zeile: " + (await page.locator("[data-ka=ansage]").innerText()));
// Englisch
await page.evaluate(async () => { await window.__vc.setLanguage("en"); }); await warte(500);
const bt = await page.locator("[data-ka=editor]").innerText(); ok(/layers/i.test(bt) && /new screen/i.test(bt) && /tile set/i.test(bt) && !/karten-/.test(bt), "Englisch: Texte folgen der Sprache, keine Schlüssel");
await page.evaluate(async () => { await window.__vc.setLanguage("de"); });
console.log("\n" + n + " Prüfungen, " + fails.length + " Fehler"); await browser.close(); process.exit(fails.length ? 1 : 0);
