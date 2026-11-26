// SPDX-License-Identifier: GPL-3.0-or-later
// Ablauftest „Absprung aus dem Code-Editor auf Include-Pfade“ (Include-von-Medien Teil E): Doppelklick, Kontextmenü, Altpfad, fehlende Datei.
// Aufruf: node tools/e2e-include.mjs <Kopie von examples/c64/sprite_demo (wird verändert!)>   (Vite :1420, vintagecoder mit VC_TESTBRIDGE=17420 VC_TEST_NO_SPAWN=1 VC_NO_SAVE=1)
import { webkit } from "playwright";
import { mkdirSync, writeFileSync, appendFileSync } from "node:fs";
import { join } from "node:path";
const [proj] = process.argv.slice(2);
if (!proj) { console.error("Aufruf: node tools/e2e-include.mjs <Projektkopie>"); process.exit(2); }
mkdirSync(join(proj, "lib"), { recursive: true });
writeFileSync(join(proj, "lib", "hilfe.inc"), "; Hilfsdatei\n");
appendFileSync(join(proj, "sprite_demo.asm"), '\n.include "lib/hilfe.inc"\n.include "lib/fehlt.inc"\n.include "build/export/alt.inc"\n');
const browser = await webkit.launch(); const page = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
const fails = []; let n = 0;
const ok = (c, m) => { n++; if (!c) { fails.push(m); console.log("FEHLER:", m); } else console.log("ok:", m); };
page.on("pageerror", (e) => { fails.push("Seitenfehler " + e.message); console.log("Seitenfehler", e.message); });
await page.goto("http://127.0.0.1:1420/?bridge=17420"); await page.waitForFunction(() => window.__vc && window.__vc.ui.ready);
await page.evaluate(async (p) => { await window.__vc.setLanguage("de"); await window.__vc.openProject(p); await window.__vc.openFile("sprite_demo.asm"); }, proj);
await page.waitForSelector(".cm-editor .cm-content"); await page.waitForTimeout(500);
const zeile = (text) => page.locator(".cm-line", { hasText: text }).first();
// Mitte eines Textstücks in der Zeile (die Zeile ist breiter als ihr Text): Bildschirmkoordinaten
const punkt = async (zeilentext, stueck) => {
  const suche = () => page.evaluate(([z, st]) => {
    const l = [...document.querySelectorAll(".cm-line")].find((x) => x.textContent.includes(z)); if (!l) return null; l.scrollIntoView({ block: "center" });
    const w = document.createTreeWalker(l, NodeFilter.SHOW_TEXT); let nd; while ((nd = w.nextNode())) { const i = nd.textContent.indexOf(st); if (i >= 0) { const r = document.createRange(); r.setStart(nd, i); r.setEnd(nd, i + st.length); const b = r.getBoundingClientRect(); return { x: b.x + b.width / 2, y: b.y + b.height / 2 }; } }
    return null; }, [zeilentext, stueck]);
  for (const ziel of [null, 0, 0.25, 0.5, 0.75, 1]) {
    if (ziel !== null) { await page.evaluate((z) => { for (const s of document.querySelectorAll(".cm-scroller")) s.scrollTop = z * s.scrollHeight; }, ziel); await page.waitForTimeout(250); }
    const p = await suche(); if (p) return p;
  }
  throw new Error("Text nicht sichtbar: " + zeilentext);
};
const dbl = async (z, st) => { const p = await punkt(z, st); await page.mouse.dblclick(p.x, p.y); };
const toasts = () => page.locator(".toasts").innerText().catch(() => "");
const tabs = () => page.locator(".tab .nm").allInnerTexts();
// Doppelklick auf den Pfad der Mediendatei öffnet den Sprite-Editor
await dbl('.include "graphics/hero.vcsprite"', "hero"); await page.waitForTimeout(800);
ok(await page.evaluate(() => window.__vc.ui.mode === "grafik"), "Doppelklick auf den Pfad eines Sprites öffnet den Sprite-Editor");
// zurück zum Code; Doppelklick auf eine Quelltext-Include öffnet einen Reiter
await page.evaluate(async () => { await window.__vc.runCommand("nav:code"); await window.__vc.openFile("sprite_demo.asm"); }); await page.waitForTimeout(600);
await dbl('.include "lib/hilfe.inc"', "hilfe.inc"); await page.waitForTimeout(700);
ok((await tabs()).includes("hilfe.inc"), "Doppelklick auf eine Quelltext-Include öffnet die Datei als Reiter");
// Doppelklick auf das Schlüsselwort wählt wie bisher das Wort (kein Absprung)
await page.evaluate(async () => { await window.__vc.openFile("sprite_demo.asm"); }); await page.waitForTimeout(500);
const vorher = (await tabs()).length;
await dbl('.include "lib/hilfe.inc"', ".include"); await page.waitForTimeout(400);
ok((await tabs()).length === vorher, "Doppelklick auf andere Wörter wirkt wie bisher (kein Absprung)");
// fehlende Datei: Meldung statt Öffnen
await dbl('.include "lib/fehlt.inc"', "fehlt.inc"); await page.waitForTimeout(600);
ok(/Datei nicht gefunden/.test(await toasts()), "Fehlende Datei: Meldung „Datei nicht gefunden“");
// Altpfad
await dbl('.include "build/export/alt.inc"', "export"); await page.waitForTimeout(600);
ok(/Altpfad/.test(await toasts()), "Altpfad build/export: Hinweis (W214)");
// Kontextmenü: Öffnen und Im Projektbaum zeigen
const p5 = await punkt('.include "graphics/hero.vcsprite"', "hero");
await page.mouse.click(p5.x, p5.y, { button: "right" }); await page.waitForTimeout(300);
ok(await page.getByRole("menuitem", { name: "Eingebundene Datei öffnen" }).count() === 1 && await page.getByRole("menuitem", { name: "Im Projektbaum zeigen" }).count() === 1, "Kontextmenü auf dem Pfad bietet „Eingebundene Datei öffnen“ und „Im Projektbaum zeigen“");
await page.getByRole("menuitem", { name: "Im Projektbaum zeigen" }).click(); await page.waitForTimeout(300);
ok(await page.evaluate(() => window.__vc.ui.selected === "graphics/hero.vcsprite"), "Im Projektbaum zeigen wählt die Datei im Baum");
// Rechtsklick außerhalb eines Pfads: kein Include-Eintrag
await page.evaluate(async () => { await window.__vc.runCommand("nav:code"); await window.__vc.openFile("sprite_demo.asm"); }); await page.waitForTimeout(500);
const p6 = await punkt("sprite_demo.asm", "sprite_demo.asm");
await page.mouse.click(p6.x, p6.y, { button: "right" }); await page.waitForTimeout(300);
ok(await page.getByRole("menuitem", { name: "Eingebundene Datei öffnen" }).count() === 0, "Kontextmenü außerhalb eines Pfads ohne Include-Einträge");
await page.keyboard.press("Escape");
// Haltepunkte und Ansicht (Handtest Block B): Klick auf die Zeilennummer, Haltepunkt bleibt nach Schließen/Öffnen, Cursor und Bildlauf bleiben beim Bereichswechsel
await page.evaluate(async () => { await window.__vc.runCommand("nav:code"); await window.__vc.openFile("sprite_demo.asm"); }); await page.waitForTimeout(600);
const nummer = (n) => page.locator(".cm-lineNumbers .cm-gutterElement", { hasText: new RegExp("^" + n + "$") }).first();
const bp0 = await page.locator(".mk-breakpoint").count();
await nummer(4).click(); await page.waitForTimeout(250);
ok(await page.locator(".mk-breakpoint").count() === bp0 + 1, "Klick auf die Zeilennummer setzt einen Haltepunkt");
await nummer(4).click(); await page.waitForTimeout(250);
ok(await page.locator(".mk-breakpoint").count() === bp0, "Zweiter Klick auf die Zeilennummer entfernt ihn wieder");
await nummer(4).click(); await page.waitForTimeout(250);
await page.evaluate(async () => { await window.__vc.runCommand("datei_schliessen"); }); await page.waitForTimeout(400);
await page.evaluate(async () => { await window.__vc.openFile("sprite_demo.asm"); }); await page.waitForTimeout(700);
ok(await page.locator(".mk-breakpoint").count() === bp0 + 1, "Haltepunkt bleibt nach Schließen und erneutem Öffnen der Datei");
// Cursor und Bildlauf: weit nach unten, Bereich wechseln, zurück
await page.evaluate(() => { const v = window.__vc.getView?.(); });
const stand = () => page.evaluate(() => { const sc = document.querySelector(".cm-scroller"); const sel = window.getSelection(); return { top: sc ? Math.round(sc.scrollTop) : -1, sel: window.__vc.ed.cursor?.line }; });
await page.evaluate(() => { const sc = document.querySelector(".cm-scroller"); sc.scrollTop = sc.scrollHeight; }); await page.waitForTimeout(300);
const letzte = await page.locator(".cm-line").last().click({ position: { x: 5, y: 5 } }).then(() => stand());
await page.waitForTimeout(200);
ok(letzte.top > 0, "Vorbereitung: Ansicht nach unten gescrollt (" + letzte.top + ")");
await page.evaluate(async () => { await window.__vc.runCommand("nav:grafik"); }); await page.waitForTimeout(500);
await page.evaluate(async () => { await window.__vc.runCommand("nav:code"); }); await page.waitForTimeout(700);
const danach = await stand();
ok(danach.sel === letzte.sel && Math.abs(danach.top - letzte.top) < 40, "Nach Bereichswechsel: Cursorzeile " + danach.sel + " (vorher " + letzte.sel + ") und Bildlauf " + danach.top + " (vorher " + letzte.top + ") bleiben erhalten");
console.log("\n" + n + " Prüfungen, " + fails.length + " Fehler"); await browser.close(); process.exit(fails.length ? 1 : 0);
