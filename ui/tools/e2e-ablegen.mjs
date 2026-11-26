// SPDX-License-Identifier: GPL-3.0-or-later
// Ablauftest „Dateien auf das Fenster ziehen“ (W1): das native Drop-Ereignis gibt es nur in der echten Anwendung; hier wird dieselbe Verarbeitung mit den Pfaden aufgerufen (`__vc.handleDrop`).
// Dazu das Ziehen innerhalb des Fensters mit Zeigerereignissen (Dateibaum, Ordner, Reiter).
// Aufruf: node tools/e2e-ablegen.mjs <Projektkopie (wird verändert!)> <Ordner mit externen Testdateien (wird angelegt)> [Diskettenabbild]
import { webkit } from "playwright";
import { cpSync, existsSync, mkdirSync, readdirSync, rmSync, writeFileSync, readFileSync } from "node:fs";
import { join } from "node:path";

const [proj, ext, disk] = process.argv.slice(2);
if (!proj || !ext) { console.error("Aufruf: node tools/e2e-ablegen.mjs <Projektkopie> <Ordner> [d64]"); process.exit(2); }
mkdirSync(ext, { recursive: true });
const fails = []; let n = 0;
const ok = (c, m) => { n++; if (!c) { fails.push(m); console.log("FEHLER:", m); } else console.log("ok:", m); };
writeFileSync(join(ext, "extern.asm"), ".target c64\n.org $0801\nrts\n"); writeFileSync(join(ext, "seltsam.xyz"), "x");
writeFileSync(join(ext, "held.vcsprite"), readFileSync("../examples/c64/sprite_demo/graphics/hero.vcsprite"));
const browser = await webkit.launch();
const page = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
page.on("pageerror", (e) => { fails.push("Seitenfehler " + e.message); console.log("Seitenfehler", e.message); });
await page.goto("http://127.0.0.1:1420/?bridge=17420"); await page.waitForFunction(() => window.__vc && window.__vc.ui.ready);
const drop = (paths) => page.evaluate((p) => window.__vc.handleDrop(p), paths);
const tabNames = () => page.locator(".tab .nm").allInnerTexts();

// Ohne Projekt: Hinweis statt Absturz
await page.evaluate(async () => { await window.__vc.setLanguage("de"); });
await drop([join(ext, "extern.asm")]); await page.waitForTimeout(500);
ok(/Projekt/.test(await page.locator(".toasts").innerText()), "Ohne Projekt: Hinweis, zuerst ein Projekt zu öffnen");
await page.evaluate(async (p) => { await window.__vc.openProject(p); }, proj); await page.waitForTimeout(900);

// Datei im Projekt: öffnet direkt
const imProjekt = readdirSync(proj).find((f) => f.endsWith(".asm"));
await drop([join(proj, imProjekt)]); await page.waitForTimeout(600);
ok((await tabNames()).some((x) => x === imProjekt), "Datei im Projekt: öffnet sich als Reiter (" + imProjekt + ")");
ok(!(await page.locator("dialog[open]").count()), "… ohne Rückfrage");

// Außerhalb: Rückfrage, Abbrechen kopiert nichts
await drop([join(ext, "extern.asm")]); await page.waitForSelector("dialog[open] [data-drop=yes]");
ok(/extern\.asm/.test(await page.locator("dialog[open]").innerText()), "Datei außerhalb des Projekts: Rückfrage nennt die Datei");
await page.locator("[data-drop=no]").click(); await page.waitForTimeout(300);
ok(!existsSync(join(proj, "extern.asm")), "Abbrechen kopiert nichts");
// … Zustimmen kopiert ins Projekt (nichts überschrieben) und öffnet
await drop([join(ext, "extern.asm")]); await page.waitForSelector("dialog[open] [data-drop=yes]"); await page.locator("[data-drop=yes]").click(); await page.waitForTimeout(900);
ok(existsSync(join(proj, "extern.asm")) && (await tabNames()).includes("extern.asm"), "Zustimmen: Datei im Projekt, als Reiter geöffnet");
await drop([join(ext, "extern.asm")]); await page.waitForSelector("dialog[open] [data-drop=yes]"); await page.locator("[data-drop=yes]").click(); await page.waitForTimeout(900);
ok(existsSync(join(proj, "extern_2.asm")) && readFileSync(join(proj, "extern.asm"), "utf8").includes("rts"), "Zweites Ablegen überschreibt nichts (extern_2.asm)");
// Sprite: kommt nach graphics und öffnet im Sprite-Editor
await drop([join(ext, "held.vcsprite")]); await page.waitForSelector("dialog[open] [data-drop=yes]"); await page.locator("[data-drop=yes]").click(); await page.waitForTimeout(1200);
ok(existsSync(join(proj, "graphics", "held.vcsprite")) && (await page.evaluate(() => window.__vc.sp.view?.rel)) === "graphics/held.vcsprite", "Sprite-Datei: nach graphics kopiert und im Sprite-Editor geöffnet");
// Unbekannter Typ
await drop([join(ext, "seltsam.xyz")]); await page.waitForTimeout(500);
ok(/seltsam\.xyz/.test(await page.locator(".toasts").innerText()) && !(await page.locator("dialog[open]").count()), "Unbekannter Dateityp: Hinweis, keine Rückfrage");
// Diskettenabbild
if (disk && existsSync(disk)) { await drop([disk]); await page.waitForTimeout(900); ok((await page.evaluate(() => window.__vc.dk.view?.format)) === "D64", "Diskettenabbild: öffnet im Disk-Editor"); }

// Während einer Rückfrage ankommende Pfade (Öffnen mit, zweiter Start) gehen nicht verloren
if (disk && existsSync(disk)) {
  await page.evaluate(() => window.__vc.runCommand("nav:code")); await page.waitForTimeout(300);
  await page.evaluate(() => { window.__vc.dk.view = null; });
  await drop([join(ext, "extern.asm")]); await page.waitForSelector("dialog[open] [data-drop=yes]");
  await drop([disk]); await page.waitForTimeout(400);
  ok((await page.locator("dialog[open] [data-drop=yes]").count()) === 1, "Offene Rückfrage bleibt bestehen, das zweite Ereignis wartet");
  await page.locator("[data-drop=no]").click(); await page.waitForTimeout(900);
  ok((await page.evaluate(() => window.__vc.dk.view?.format)) === "D64", "Nach der Antwort wird das wartende Diskettenabbild geöffnet");
}

// Ziehen im Fenster mit Zeigerereignissen (statt HTML5-Ziehen): Reiter ordnen, Datei in Ordner
await page.evaluate(() => window.__vc.runCommand("nav:code")); await page.waitForTimeout(400);
await page.evaluate(() => window.__vc.openFile("extern.asm")); await page.evaluate(() => window.__vc.openFile("extern_2.asm")); await page.waitForTimeout(400);
const names0 = await tabNames(); const a = page.locator(".tab", { hasText: "extern_2.asm" }).first(), b = page.locator(".tab", { hasText: "extern.asm" }).first();
const ba = await a.boundingBox(), bb = await b.boundingBox();
await page.mouse.move(ba.x + ba.width / 2, ba.y + ba.height / 2); await page.mouse.down(); await page.mouse.move(bb.x + 10, bb.y + bb.height / 2, { steps: 6 }); await page.mouse.up(); await page.waitForTimeout(400);
const names1 = await tabNames(); ok(JSON.stringify(names0) !== JSON.stringify(names1) && names1.indexOf("extern_2.asm") < names1.indexOf("extern.asm"), "Reiter ziehen ordnet um: " + names0.join(",") + " → " + names1.join(","));
await page.evaluate(async () => { await window.__vc.call("dir_create", { rel: "ablage" }); const r = await window.__vc.call("tree_reload"); window.__vc.ui.project.tree = r.tree; }); await page.waitForTimeout(400);
const f = page.locator("[role=treeitem][data-rel='extern_2.asm']"), d = page.locator("[role=treeitem][data-rel='ablage']");
const bf = await f.boundingBox(), bd = await d.boundingBox();
await page.mouse.move(bf.x + 30, bf.y + bf.height / 2); await page.mouse.down(); await page.mouse.move(bd.x + 30, bd.y + bd.height / 2, { steps: 6 }); await page.mouse.up(); await page.waitForTimeout(700);
ok(existsSync(join(proj, "ablage", "extern_2.asm")) && !existsSync(join(proj, "extern_2.asm")), "Datei im Baum auf einen Ordner ziehen verschiebt sie");
ok(!(await page.locator(".dragghost, .ghost").count()) && !(await page.evaluate(() => document.body.classList.contains("vc-dragging"))), "Nach dem Ablegen ist das Ziehen beendet");
// Zwei Aufträge fast gleichzeitig (zweiter Aufruf der Anwendung mitten im Öffnen des ersten): sie laufen nacheinander, das zuletzt übergebene Projekt bleibt offen
const zwei = proj + "-zwei"; rmSync(zwei, { recursive: true, force: true }); cpSync(proj, zwei, { recursive: true });
await page.evaluate(async ([a, b]) => { const p1 = window.__vc.handleDrop([a]); const p2 = window.__vc.handleDrop([b]); await Promise.all([p1, p2]); }, [proj, zwei]); await page.waitForTimeout(1500);
ok((await page.evaluate(() => window.__vc.ui.project?.root)) === zwei, "Zwei fast gleichzeitige Aufträge: das zuletzt übergebene Projekt bleibt offen (nicht das zuerst begonnene)");
rmSync(zwei, { recursive: true, force: true });
console.log(`\n${n} Prüfungen, ${fails.length} Fehler`);
await browser.close();
process.exit(fails.length ? 1 : 0);
