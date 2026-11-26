// SPDX-License-Identifier: GPL-3.0-or-later
// Ablauftest Bildschirm-Editor (E9): Playwright (WebKit) gegen die Prüfbrücke des echten Kerns.
// Aufruf: node tools/e2e-bildschirm.mjs <Projektordner (Kopie von examples/c64/gfx_demo; wird verändert!)> [Bilderordner]
import { webkit } from "playwright";
import { mkdirSync } from "node:fs";
import { join } from "node:path";

if (process.platform === "win32" && !process.env.VC_E2E_WIN) { console.log("übersprungen unter Windows\n0 Prüfungen, 0 Fehler"); process.exit(0); }
const [proj, shots] = process.argv.slice(2);
if (!proj) { console.error("Aufruf: node tools/e2e-bildschirm.mjs <Projektordner> [Bilder]"); process.exit(2); }
if (shots) mkdirSync(shots, { recursive: true });
const browser = await webkit.launch();
const page = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
const fails = []; let n = 0;
const ok = (c, m) => { n++; if (!c) { fails.push(m); console.log("FEHLER:", m); } else console.log("ok:", m); };
page.on("pageerror", (e) => { fails.push("Seitenfehler " + e.message); console.log("Seitenfehler", e.message); });
const snap = async (name) => { if (shots) await page.screenshot({ path: join(shots, name + ".png") }); };
await page.goto("http://127.0.0.1:1420/?bridge=17420"); await page.waitForFunction(() => window.__vc && window.__vc.ui.ready);
await page.evaluate(async (p) => { await window.__vc.setLanguage("de"); await window.__vc.openProject(p); }, proj); await page.waitForTimeout(900);
const bv = () => page.evaluate(() => window.__vc.bs.view); const warte = () => page.waitForTimeout(350);
const zelle = (x, y) => page.locator(`[data-bs=zelle][data-k='${y * 40 + x}']`);
const code = async (x, y) => (await bv()).codes[y * 40 + x];

// 1) Öffnen
await page.evaluate(() => window.__vc.openFile("graphics/title.vcscreen")); await page.waitForTimeout(1000);
let v = await bv();
ok(v && v.kind === "screen" && v.w === 40 && v.h === 25 && v.farbRam && !v.dirty, "title.vcscreen öffnet als Textbildschirm (40 × 25, Farbe je Zelle)");
ok(await page.evaluate(() => window.__vc.ui.mode === "grafik" && window.__vc.ui.sub.grafik === "bildschirm"), "Die Ansicht wechselt zu Grafik ▸ Bildschirm");
ok((await page.locator("[data-bs=zelle][role=option]").count()) === 1000 && (await page.locator("[data-bs=zeichen][role=option]").count()) === 256 && (await page.locator("[data-bs=buehne] [role=grid], [data-bs=tafel] [role=grid], [role=application]").count()) === 0, "Zellen und Zeichentafel sind Auswahllisten mit Optionen (keine Raster- oder Anwendungsrolle)");
ok(/Spalte 1, Zeile 1: Zeichen \d+, \$[0-9A-F]{2}/.test(await zelle(0, 0).getAttribute("aria-label")), "Zelle sagt Spalte, Zeile und Zeichen an: " + (await zelle(0, 0).getAttribute("aria-label")));
ok((await page.locator("[data-bs=buehne] img").getAttribute("src")).startsWith("data:image/png"), "Der Bildschirm wird als Bild vom Kern gezeigt");
await snap("bildschirm-1-offen");

// 2) Zeichen setzen: Klick, rechte Taste
await page.locator("[data-bs=zeichen][data-c='66']").click(); await warte(); ok((await bv()).brush.code === 66, "Zeichen 66 aus der Tafel gewählt");
await zelle(5, 5).click(); await warte(); ok((await code(5, 5)) === 66 && (await bv()).dirty, "Klick setzt das Zeichen in die Zelle; Datei als geändert gekennzeichnet");
await zelle(5, 5).click({ button: "right" }); await warte(); ok((await code(5, 5)) === 32, "Rechte Maustaste löscht (Leerzeichen)");
// Zug: ein Rückgängig-Schritt
const bb = await Promise.all([5, 6, 7, 8].map((x) => zelle(x, 7).boundingBox())); const mid = (b) => [b.x + b.width / 2, b.y + b.height / 2];
await page.mouse.move(...mid(bb[0])); await page.mouse.down(); await page.mouse.move(...mid(bb[1]), { steps: 3 }); await page.mouse.move(...mid(bb[3]), { steps: 3 }); await page.mouse.up(); await warte();
ok((await code(5, 7)) === 66 && (await code(7, 7)) === 66 && (await code(8, 7)) === 66, "Ziehen setzt das Zeichen in allen überstrichenen Zellen");
await page.locator("[data-act=undo]").click(); await warte(); ok((await code(5, 7)) !== 66 && (await code(8, 7)) !== 66, "Ein Rückgängig nimmt den ganzen Zug zurück");
await page.locator("[data-act=redo]").click(); await warte();
// Farbe der neuen Zeichen
await page.locator("[aria-label='Farbe der neuen Zeichen'] [data-idx='5']").click(); await warte(); await zelle(9, 9).click(); await warte(); v = await bv(); ok(v.brush.color === 5 && v.farben[9 * 40 + 9] === 5, "Zeichenfarbe 5 gewählt und mit dem Zeichen in die Zelle geschrieben");

// 3) Werkzeuge: Rahmen, Füllen, Pipette, Löschen, Text
await page.locator("[data-tool=rect]").click(); await warte(); const r0 = await zelle(20, 12).boundingBox(), r1 = await zelle(23, 14).boundingBox();
await page.mouse.move(...mid(r0)); await page.mouse.down(); await page.mouse.move(...mid(r1), { steps: 5 }); await page.mouse.up(); await warte();
ok((await code(20, 12)) === 66 && (await code(23, 14)) === 66 && (await code(23, 12)) === 66 && (await code(21, 13)) !== 66, "Rahmen: die Randzellen tragen das Zeichen, das Innere bleibt");
await page.locator("[data-tool=pick]").click(); await warte(); await zelle(20, 12).click(); await warte(); v = await bv(); ok(v.brush.code === 66 && v.tool === "char", "Pipette nimmt Zeichen und Farbe auf und kehrt zum Zeichenwerkzeug zurück");
await page.locator("[data-bs=zeichen][data-c='81']").click(); await page.locator("[data-tool=fill]").click(); await warte(); await zelle(30, 3).click(); await warte(); ok((await code(39, 24)) === 81, "Füllen füllt die zusammenhängende Fläche");
await page.locator("[data-tool=erase]").click(); await warte(); await zelle(30, 3).click(); await warte(); ok((await code(30, 3)) === 32, "Löschen setzt ein Leerzeichen");
await page.locator("[data-tool=text]").click(); await warte(); await zelle(2, 20).click(); await warte(); await page.keyboard.type("HI"); await page.waitForTimeout(900); v = await bv();
ok((await code(2, 20)) === 8 && (await code(3, 20)) === 9 && v.cursor[0] === 4, "Text tippen: H und I in Bildschirmcodes, der Cursor rückt vor");
await page.locator("[data-act=undo]").click(); await warte(); ok((await code(2, 20)) !== 8 && (await code(3, 20)) !== 9, "Der getippte Text ist ein Rückgängig-Schritt");

// 4) Tastatur: Pfeile bewegen den Cursor, Enter wendet das Werkzeug an
await page.locator("[data-tool=char]").click(); await warte(); await page.locator("[data-bs=zeichen][data-c='70']").click(); await warte(); await zelle(10, 10).focus(); await page.keyboard.press("ArrowRight"); await page.keyboard.press("ArrowDown"); await page.waitForTimeout(900);
ok((await bv()).cursor[0] === 11 && (await bv()).cursor[1] === 11 && (await page.evaluate(() => document.activeElement?.getAttribute("data-k"))) === String(11 * 40 + 11), "Pfeiltasten bewegen den Cursor und den Fokus (Zelle 11, 11)");
await page.keyboard.press("Enter"); await warte(); ok((await code(11, 11)) === 70, "Enter setzt das Zeichen in die Cursorzelle");
ok(/Spalte 12 Zeile 12/.test(await page.locator("[data-bs=zellinfo]").innerText()), "Die Fußzeile nennt Spalte und Zeile: " + (await page.locator("[data-bs=zellinfo]").innerText()));

// 5) Farben des Systems, Zeichensatz, Ansicht
await page.locator("[aria-label='Rahmen'] [data-idx='2']").click(); await warte(); ok((await bv()).roles.border === 2, "Rahmenfarbe 2 gewählt");
await page.locator("[data-bs=zeichensatz]").selectOption(""); await warte(); ok((await bv()).charset === null, "Zeichensatz „Ohne“ gewählt (Bildschirm ohne Zeichen)");
await page.locator("[data-act=undo]").click(); await warte(); ok((await bv()).charset !== null, "Rückgängig stellt den Zeichensatz wieder her");
await snap("bildschirm-2-bearbeitet");

// 6) Zerlegen in Kacheln
await page.locator("[data-act=save]").click(); await warte();
await page.locator("[data-act=zerlegen]").click(); await page.waitForSelector("[data-bs=zerlegen-vorschau]"); await page.waitForFunction(() => /Kacheln/.test(document.querySelector("[data-bs=zerlegen-vorschau]")?.textContent ?? ""), null, { timeout: 8000 }).catch(() => {});
ok(/Kacheln/.test(await page.locator("[data-bs=zerlegen-vorschau]").innerText()), "Vorschau des Zerlegens: " + (await page.locator("[data-bs=zerlegen-vorschau]").innerText()));
await page.locator("[data-bs=zerlegen-los]").click(); await page.waitForTimeout(1500);
ok(await page.evaluate(() => /_tiles\.vctiles$/.test(window.__vc.kc.view?.rel ?? "") && window.__vc.ui.sub.grafik === "kacheln"), "Zerlegen schreibt Kachelsatz und Bank und öffnet den Kachelsatz im Kachel-Editor");

// 7) Bank-Datei ist kein einfacher Textbildschirm: öffnet als Text mit Hinweis
await page.evaluate(() => window.__vc.openFile("graphics/level.vcscreen")); await page.waitForTimeout(900);
// Bank mit Kacheln und mehreren Karten: öffnet im Karten-Editor (E10), nicht als Fehler und nicht als Text
ok(await page.evaluate(() => window.__vc.ui.mode === "grafik" && window.__vc.ui.sub.grafik === "karten") && await page.locator("[data-ka=editor]").count() === 1, "Eine Bank (Kacheln und Karten) öffnet im Karten-Editor");
ok(!/lässt sich nicht öffnen/.test(await page.locator(".toasts").innerText().catch(() => "")), "… und keine Fehlermeldung");
await page.evaluate(() => window.__vc.runCommand("nav:grafik:bildschirm")); await warte();

// 8) Export, Englisch
await page.evaluate(() => window.__vc.openFile("graphics/title.vcscreen")); await page.waitForTimeout(900);
await page.locator("[data-act=export]").click(); await page.waitForSelector("dialog[open] [data-ex=vorschau]"); await page.waitForFunction(() => (document.querySelector("[data-ex=vorschau]")?.textContent ?? "").length > 50, null, { timeout: 8000 }).catch(() => {});
ok((await page.locator("[data-ex=vorschau]").innerText()).length > 50, "Export-Dialog zeigt eine Vorschau des Bildschirms"); await page.getByRole("button", { name: "Abbrechen" }).first().click(); await warte();
await page.evaluate(async () => { await window.__vc.setLanguage("en"); }); await warte(); const bt = await page.locator("body").innerText(); ok(/system colou?rs/i.test(bt) && /charset/i.test(bt) && /split into tiles/i.test(bt), "Englisch: System colours, Charset, Split into tiles");
// Handtest: Zellen erscheinen schon beim Ziehen (Hervorhebung), Linie nach dem Loslassen
await page.evaluate(() => window.__vc.openFile("graphics/title.vcscreen")); await page.waitForTimeout(800);
{ const a = await zelle(10, 20).boundingBox(), b = await zelle(14, 20).boundingBox();
  await page.mouse.move(a.x + a.width / 2, a.y + a.height / 2); await page.mouse.down();
  await page.mouse.move(b.x + b.width / 2, b.y + b.height / 2, { steps: 6 }); await page.waitForTimeout(150);
  ok(await page.locator(".zl.zug").count() >= 4, "Beim Ziehen sind die berührten Zellen sofort hervorgehoben (" + await page.locator(".zl.zug").count() + ")");
  await page.mouse.up(); await warte(); await page.waitForTimeout(300);
  ok(await page.locator(".zl.zug").count() === 0, "Nach dem Loslassen ist die Hervorhebung weg");
  const c = (await bv()).codes; ok([10, 11, 12, 13, 14].every((x) => c[20 * 40 + x] === c[20 * 40 + 10]), "Die Linie ist nach dem Loslassen durchgehend"); }
await page.evaluate(async () => { await window.__vc.setLanguage("de"); });
console.log(`\n${n} Prüfungen, ${fails.length} Fehler`);
await browser.close();
process.exit(fails.length ? 1 : 0);
