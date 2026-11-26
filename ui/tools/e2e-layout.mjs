// SPDX-License-Identifier: GPL-3.0-or-later
// Ablauftest Layout (E1.4): Größen verstellen, Layout merken und zurücksetzen, Bereiche maximieren und wiederherstellen, Fokus-Maximieren.
// Aufruf: node tools/e2e-layout.mjs <Projektkopie> [Bilderordner]   (Vite auf :1420 und `vintagecoder` mit VC_TESTBRIDGE=17420 VC_TEST_NO_SPAWN=1 VC_NO_SAVE=1 müssen laufen)
import { webkit } from "playwright";
import { mkdirSync } from "node:fs";
import { join } from "node:path";
const MOD = process.platform === "darwin" ? "Meta" : "Control";   // Kürzel: Befehlstaste (macOS) bzw. Strg

const [proj, shots] = process.argv.slice(2);
if (!proj) { console.error("Aufruf: node tools/e2e-layout.mjs <Projektkopie> [Bilder]"); process.exit(2); }
if (shots) mkdirSync(shots, { recursive: true });
const browser = await webkit.launch();
const page = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
const fails = []; let n = 0; const csp = [];
const ok = (c, m) => { n++; if (!c) { fails.push(m); console.log("FEHLER:", m); } else console.log("ok:", m); };
page.on("pageerror", (e) => { fails.push("Seitenfehler " + e.message); console.log("Seitenfehler", e.message); });
page.on("console", (m) => { if (/Content Security Policy|Refused to/i.test(m.text())) csp.push(m.text()); });
const snap = async (name) => { if (shots) await page.screenshot({ path: join(shots, name + ".png") }); };
const start = async () => { await page.goto("http://127.0.0.1:1420/?bridge=17420"); await page.waitForFunction(() => window.__vc && window.__vc.ui.ready); await page.evaluate(async () => { await window.__vc.setLanguage("de"); }); };
const oeffne = async () => { await page.evaluate(async ([p]) => { await window.__vc.openProject(p); }, [proj]); await page.waitForTimeout(700); };
const box = (sel) => page.locator(sel).first().boundingBox();
const cmd = async (id) => { await page.evaluate((i) => { void window.__vc.runCommand(i); }, id); await page.waitForTimeout(700); };
const has = async (sel) => (await page.locator(sel).count()) > 0;
await start(); await page.evaluate(() => { localStorage.removeItem("layout"); }); await start(); await oeffne();
await page.evaluate(async () => { await window.__vc.openFile("jumprun.asm"); }); await page.waitForTimeout(500);

// Vorgabe
ok(await has("section[data-area=seitenleiste]") && !(await has(".dock")) && !(await has("[data-area=panel]")), "Vorgabe: Seitenleiste offen, Ablage und Panel zu");
// Ablage öffnen und Höhe verstellen (Tastatur und Maus)
await cmd("schublade");
const h0 = (await box(".dock")).height;
ok(Math.abs(h0 - 210) < 3, "Ablage öffnet mit der Standardhöhe (" + Math.round(h0) + " px)");
await page.locator("[role=separator][aria-label='Höhe der Ablage']").focus(); await page.keyboard.press("ArrowUp"); await page.keyboard.press("ArrowUp"); await page.keyboard.press("Shift+ArrowUp");
const h1 = (await box(".dock")).height;
ok(Math.abs(h1 - (h0 + 70)) < 3, "Pfeil hoch vergrößert die Ablage (10+10+50 px): " + Math.round(h0) + " → " + Math.round(h1));
const sp = await box("[role=separator][aria-label='Höhe der Ablage']");
await page.mouse.move(sp.x + sp.width / 2, sp.y + sp.height / 2); await page.mouse.down(); await page.mouse.move(sp.x + 10, sp.y - 60, { steps: 6 }); await page.mouse.up(); await page.waitForTimeout(200);
const h2 = (await box(".dock")).height;
ok(h2 > h1 + 40, "Ziehen mit der Maus vergrößert die Ablage: " + Math.round(h1) + " → " + Math.round(h2));
await page.locator("[role=separator][aria-label='Höhe der Ablage']").focus(); await page.keyboard.press("Home");
ok(Math.abs((await box(".dock")).height - 90) < 3, "Pos1 setzt die kleinste Höhe (90 px)");
await page.locator("[role=separator][aria-label='Höhe der Ablage']").dblclick(); await page.waitForTimeout(150);
ok(Math.abs((await box(".dock")).height - 210) < 3, "Doppelklick auf die Trennlinie stellt die Standardhöhe her");
// Seitenleisten- und Panelbreite
await page.locator("[role=separator][aria-label='Breite der Seitenleiste']").focus(); await page.keyboard.press("ArrowRight"); await page.keyboard.press("Shift+ArrowRight");
const sw = (await box("section[data-area=seitenleiste]")).width;
ok(Math.abs(sw - (252 + 60)) < 3, "Seitenleiste wird breiter (" + Math.round(sw) + " px)");
await page.evaluate(() => { window.__vc.ui.panel = true; }); await page.waitForTimeout(200);
await page.locator("[role=separator][aria-label='Breite des Maschine-Panels']").focus(); await page.keyboard.press("Shift+ArrowLeft");
ok(Math.abs((await box("[data-area=panel]")).width - 350) < 3, "Panel wird breiter (350 px)");
// Merken: neu laden
await page.waitForTimeout(500);
const gemerkt = await page.evaluate(() => JSON.parse(localStorage.getItem("layout") ?? "null"));
ok(gemerkt && gemerkt.dock === "problems" && gemerkt.panel === true && gemerkt.panelW === 350 && gemerkt.sideW === 312, "Das Layout steht in der Oberfläche: " + JSON.stringify(gemerkt));
await start(); await oeffne(); await page.evaluate(async () => { await window.__vc.openFile("jumprun.asm"); }); await page.waitForTimeout(500);
ok(await has(".dock") && await has("[data-area=panel]") && Math.abs((await box("[data-area=panel]")).width - 350) < 3 && Math.abs((await box("section[data-area=seitenleiste]")).width - 312) < 3, "Nach dem Neustart: Ablage, Panel und Breiten wie zuletzt");
// Seitenleiste zu (Streifen mit Symbolen)
await cmd("seitenleiste");
ok(await has("[data-strip=files]") && await has("[data-strip=search]") && await has("[data-strip=open]") && !(await has("section[data-area=seitenleiste]")), "Seitenleiste zu: Streifen mit Symbolen für Dateien und Suche");
await page.locator("[data-strip=search]").click(); await page.waitForTimeout(200);
ok(await page.locator(".dock [data-tab=search][aria-selected=true]").count() === 1, "Symbol Suche öffnet die Suche in der Ablage");
await snap("layout-1-streifen");
await page.locator("[data-strip=files]").click(); await page.waitForTimeout(200);
ok(await has("section[data-area=seitenleiste]"), "Symbol Dateien klappt die Seitenleiste auf");

// Zurücksetzen
await cmd("layout_zuruecksetzen");
ok(!(await has(".dock")) && !(await has("[data-area=panel]")) && Math.abs((await box("section[data-area=seitenleiste]")).width - 252) < 3, "Layout zurücksetzen: Ablage und Panel zu, Seitenleiste in Standardbreite");
const nachReset = await page.evaluate(() => JSON.parse(localStorage.getItem("layout") ?? "null"));
await page.waitForTimeout(500);
const nachReset2 = await page.evaluate(() => JSON.parse(localStorage.getItem("layout") ?? "null"));
ok(!nachReset2 || (nachReset2.dock === null && nachReset2.panel === false && nachReset2.sideW === 252 && nachReset2.panelW === 300 && nachReset2.dockH === 210), "… und das Gemerkte ist auf der Vorgabe: " + JSON.stringify(nachReset2 ?? nachReset));

// Maximieren
const maxOf = (a) => page.evaluate((x) => window.__vc.lay.max === x, a);
await cmd("schublade_max");
const d = await box(".dock"); const main = await box("main.body");
ok(await maxOf("schublade") && d.height > main.height - 120, "Ablage maximieren: füllt die Fläche (" + Math.round(d.height) + " von " + Math.round(main.height) + " px)");
ok(!(await has(".cm-editor")) && await has(".tabs") && await has("[data-strip=open]"), "… Editor ist weg, Reiterleiste und Seitenleistenstreifen bleiben");
await snap("layout-2-ablage-max");
ok(await page.locator("[data-max=schublade][aria-pressed=true]").count() === 1, "Knopf im Kopf der Ablage zeigt „wiederherstellen“");
await page.locator("[data-max=schublade]").click(); await page.waitForTimeout(700);
ok(!(await maxOf("schublade")) && await has(".cm-editor") && await has("section[data-area=seitenleiste]"), "Wiederherstellen bringt Editor und Seitenleiste zurück");
// Doppelklick auf den Kopf
await page.locator(".dock header").dblclick({ position: { x: 600, y: 14 } }); await page.waitForTimeout(700);
ok(await maxOf("schublade"), "Doppelklick auf den Kopf der Ablage maximiert");
await page.locator(".dock header").dblclick({ position: { x: 600, y: 14 } }); await page.waitForTimeout(700);
ok(!(await maxOf("schublade")), "… und stellt wieder her");
// Seitenleiste, Dokument, Panel
await cmd("spalte_max"); const sb = await box("section[data-area=seitenleiste]");
ok(await maxOf("seitenleiste") && sb.width > 1200, "Seitenleiste maximieren: füllt die Breite (" + Math.round(sb.width) + " px)"); await cmd("spalte_max");
await cmd("dokument_max");
ok(await maxOf("dokument") && !(await has("section[data-area=seitenleiste]")) && (await has("[data-strip=open]")), "Dokument maximieren: Seitenleiste wird zum Streifen"); await cmd("dokument_max");
await cmd("panel_max");
ok(await maxOf("panel") && (await box("[data-area=panel]")).width > 1200 && !(await has(".cm-editor")), "Panel maximieren: füllt die Breite, Editor ist weg"); await snap("layout-3-panel-max");
await page.locator("[data-max=restore]").click(); await page.waitForTimeout(700);
ok(!(await maxOf("panel")) && await has(".cm-editor"), "Der Streifen „Wiederherstellen“ bringt das Layout zurück");
ok(!(await has("[data-area=panel]")), "… das Panel war vorher zu und ist wieder zu (Maximieren ändert das Gemerkte nicht)");
// Anderen Bereich öffnen beendet die Maximierung
await cmd("schublade_max"); await cmd("seitenleiste"); ok(await maxOf("schublade"), "Eine Seitenleiste, die zugeht, lässt die Maximierung der Ablage bestehen"); await cmd("seitenleiste");
ok(!(await maxOf("schublade")), "Eine Seitenleiste, die aufgeht, beendet die Maximierung der Ablage"); await cmd("seitenleiste");
// Fokus-Maximieren
await page.evaluate(() => { window.__vc.ed.dock = "problems"; }); await page.waitForTimeout(200);
await page.locator(".cm-content").click(); await page.keyboard.press(MOD + "+Shift+Enter"); await page.waitForTimeout(700);
ok(await maxOf("dokument"), "⌘⇧Enter im Editor maximiert das Dokumentenfeld");
await page.keyboard.press(MOD + "+Shift+Enter"); await page.waitForTimeout(700);
ok(!(await maxOf("dokument")), "… und stellt wieder her");
await page.locator(".dock [data-tab=output]").focus(); await page.keyboard.press(MOD + "+Shift+Enter"); await page.waitForTimeout(700);
ok(await maxOf("schublade"), "⌘⇧Enter in der Ablage maximiert die Ablage");
await page.locator(".dock [data-tab=output]").focus(); await page.keyboard.press(MOD + "+Shift+Enter"); await page.waitForTimeout(700);
// Zen beendet die Maximierung
await cmd("dokument_max"); await cmd("zen");
ok(!(await maxOf("dokument")) && !(await has(".dock")) , "Zen beendet eine Maximierung"); await cmd("zen");
// Breitenklassen: schmal legt Seitenleiste und Panel über die Fläche; geteilte Ansicht steht darunter untereinander
await cmd("layout_zuruecksetzen"); await page.evaluate(async () => { await window.__vc.openFile("jumprun.asm"); await window.__vc.openFile("hal_c64.asm"); }); await page.waitForTimeout(400);
await page.setViewportSize({ width: 1440, height: 900 }); await page.waitForTimeout(400);
ok(!(await page.locator("section[data-area=seitenleiste].ovl").count()), "Breit: die Seitenleiste verdrängt die Fläche (kein Überlagern)");
await cmd("teilen"); let a1 = await box(".grp >> nth=0"), a2 = await box(".grp >> nth=1");
ok(a2.x > a1.x + 200 && Math.abs(a2.y - a1.y) < 3, "Breit: die Hälften stehen nebeneinander");
await page.setViewportSize({ width: 1100, height: 800 }); await page.waitForTimeout(500);
a1 = await box(".grp >> nth=0"); a2 = await box(".grp >> nth=1");
ok(a2.y > a1.y + 100 && Math.abs(a2.x - a1.x) < 3, "Unter 1000 Punkten Dokumentenbreite stehen die Hälften untereinander");
await snap("layout-4-untereinander");
await cmd("teilen");
await page.setViewportSize({ width: 900, height: 700 }); await page.waitForTimeout(500);
ok(await page.locator("section[data-area=seitenleiste].ovl").count() === 1, "Schmal (900 px): die Seitenleiste liegt über der Fläche");
const dk = await box("section[data-area-wrap=dokument]"); ok(dk.x < 120 && dk.width > 700, "… die Dokumentenfläche behält ihre Breite (" + Math.round(dk.width) + " px ab x " + Math.round(dk.x) + ")");
await snap("layout-5-schmal");
await page.locator(".cm-content").click(); await page.waitForTimeout(300);
ok(!(await page.locator("section[data-area=seitenleiste]").count()), "Ein Klick in die Fläche schließt die überlagernde Seitenleiste");
ok(await page.locator("[data-strip=open]").count() === 1, "… zurück bleibt der Streifen");
await page.locator("[data-strip=open]").click(); await page.waitForTimeout(300);
ok(await page.locator("section[data-area=seitenleiste].ovl").count() === 1, "Der Streifen klappt sie wieder über die Fläche");
await page.setViewportSize({ width: 1440, height: 900 }); await page.waitForTimeout(400);
ok(!(await page.locator("section[data-area=seitenleiste].ovl").count()), "Wieder breit: die Seitenleiste verdrängt wieder");
// Bildschirmleser: Sprache der Seite und benannte Bereiche
ok((await page.evaluate(() => document.documentElement.lang)) === "de", "Seitensprache ist gesetzt (lang=de)");
const regionen = await page.locator("section[aria-label], [role=group][aria-label], nav[aria-label], [role=separator][aria-label]").evaluateAll((els) => els.map((e) => e.getAttribute("aria-label")));
ok(regionen.length >= 4 && regionen.every((r) => r && r.length > 2), "Alle Bereiche und Trennlinien haben einen Namen: " + regionen.slice(0, 6).join(" | "));
ok(csp.length === 0, "Keine Meldungen der Inhaltsrichtlinie: " + csp.slice(0, 2).join(" | "));
console.log(`\n${n} Prüfungen, ${fails.length} Fehler`);
await browser.close();
process.exit(fails.length ? 1 : 0);
