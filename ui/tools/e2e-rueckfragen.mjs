// SPDX-License-Identifier: GPL-3.0-or-later
// Ablauftest der Rückfragen „Papierkorb nicht verfügbar → endgültig löschen?“ und „Diese Datei ist ein Programm → trotzdem öffnen?“ (Vorgabe jeweils Abbrechen).
// Kern mit VC_TEST_NO_SPAWN=1: kein Papierkorb (Finder/gio) und kein Programmstart. Gelöscht wird nur in der Projektkopie.
// Aufruf: node tools/e2e-rueckfragen.mjs <Projektkopie (wird verändert!)>
import { webkit } from "playwright";
import { existsSync, mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
const [proj, shots] = process.argv.slice(2);
if (!proj) { console.error("Aufruf: node tools/e2e-rueckfragen.mjs <Projektkopie>"); process.exit(2); }
writeFileSync(join(proj, "weg.txt"), "weg\n"); writeFileSync(join(proj, "start.sh"), "#!/bin/sh\necho x\n"); writeFileSync(join(proj, "info.txt"), "i\n");
const browser = await webkit.launch(); const page = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
const fails = []; let n = 0; const ok = (c, m) => { n++; if (!c) { fails.push(m); console.log("FEHLER:", m); } else console.log("ok:", m); };
page.on("pageerror", (e) => { fails.push("Seitenfehler " + e.message); console.log("Seitenfehler", e.message); });
const warte = (ms = 400) => page.waitForTimeout(ms);
if (shots) mkdirSync(shots, { recursive: true });
const snap = async (name) => { if (shots) await page.screenshot({ path: join(shots, name + ".png") }); };
await page.goto("http://127.0.0.1:1420/?bridge=17420"); await page.waitForFunction(() => window.__vc && window.__vc.ui.ready);
await page.evaluate(async (p) => { await window.__vc.setLanguage("de"); await window.__vc.openProject(p); await window.__vc.call("tree_reload"); }, proj); await warte(800);
const dlg = page.locator("dialog[open]");
// 1) Papierkorb nicht verfügbar
await page.evaluate(() => { void window.__vc.deleteFile("weg.txt"); }); await page.waitForSelector("dialog[open] [data-rf=text]"); await warte(200);
ok(/Papierkorb/.test(await dlg.innerText()) && /nicht gelöscht/.test(await dlg.innerText()), "Rückfrage nennt: Papierkorb nicht verfügbar, Datei nicht gelöscht");
ok(existsSync(join(proj, "weg.txt")), "Bis zur Antwort ist die Datei noch da"); await snap("rf-1-papierkorb");
ok(await page.evaluate(() => document.activeElement?.getAttribute("data-rf")) === "cancel", "Fokus liegt auf „Abbrechen“ (Vorgabe)");
await page.keyboard.press("Escape"); await warte(300);
ok(await dlg.count() === 0 && existsSync(join(proj, "weg.txt")), "Esc bricht ab: nichts gelöscht, kein Fehler");
await page.evaluate(() => { void window.__vc.deleteFile("weg.txt"); }); await page.waitForSelector("dialog[open] [data-rf=cancel]"); await page.locator("[data-rf=cancel]").click(); await warte(300);
ok(existsSync(join(proj, "weg.txt")), "„Abbrechen“ löscht nichts");
await page.evaluate(() => { void window.__vc.deleteFile("weg.txt"); }); await page.waitForSelector("dialog[open] [data-rf=confirm]");
ok(/Endgültig löschen/.test(await page.locator("[data-rf=confirm]").innerText()), "Knopf heißt „Endgültig löschen“");
await page.locator("[data-rf=confirm]").focus(); ok(await page.evaluate(() => document.activeElement?.getAttribute("data-rf")) === "confirm", "„Endgültig löschen“ ist fokussierbar (Tastatur; WebKit überspringt Knöpfe bei Tab)");
await page.keyboard.press("Enter"); await warte(700);
ok(!existsSync(join(proj, "weg.txt")), "Nach „Endgültig löschen“ ist die Datei weg");
// 2) Programm öffnen
await page.evaluate(() => { void window.__vc.oeffneExtern("start.sh"); }); await page.waitForSelector("dialog[open] [data-rf=text]"); await warte(200);
await warte(150); await snap("rf-2-programm"); ok(/Programm/.test(await dlg.innerText()) && /Trotzdem öffnen/.test(await page.locator("[data-rf=confirm]").innerText()), "Rückfrage: Datei ist ein Programm, „Trotzdem öffnen“");
ok(await page.evaluate(() => document.activeElement?.getAttribute("data-rf")) === "cancel", "Fokus liegt auf „Abbrechen“");
await page.keyboard.press("Escape"); await warte(300);
ok(await dlg.count() === 0 && await page.locator(".toasts").innerText().then((x) => !/Fehler|fehlgeschlagen/i.test(x)), "Abbrechen: kein Start, keine Fehlermeldung");
await page.evaluate(() => { void window.__vc.oeffneExtern("start.sh"); }); await page.waitForSelector("dialog[open] [data-rf=confirm]"); await page.locator("[data-rf=confirm]").click(); await warte(500);
ok(await dlg.count() === 0, "Nach „Trotzdem öffnen“ geht der Aufruf mit Bestätigung durch (im Test ohne Start)");
// 3) normale Datei: keine Rückfrage
await page.evaluate(() => { void window.__vc.oeffneExtern("info.txt"); }); await warte(500);
ok(await dlg.count() === 0, "Normale Datei: keine Rückfrage");
await page.evaluate(async () => { await window.__vc.setLanguage("en"); }); await page.evaluate(() => { void window.__vc.oeffneExtern("start.sh"); }); await page.waitForSelector("dialog[open] [data-rf=text]");
ok(/program/i.test(await dlg.innerText()) && !(await dlg.innerText()).includes("rf-"), "Englisch: Text vorhanden: " + (await dlg.innerText()).replace(/\s+/g, " ").slice(0, 80));
await page.keyboard.press("Escape"); await page.evaluate(async () => { await window.__vc.setLanguage("de"); });
console.log("\n" + n + " Prüfungen, " + fails.length + " Fehler"); await browser.close(); process.exit(fails.length ? 1 : 0);
