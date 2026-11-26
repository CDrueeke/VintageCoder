// SPDX-License-Identifier: GPL-3.0-or-later
// Ablauftest Markdown (E13): Vorschau, Umschalten, Schreibhilfen, Aufgaben abhaken, Links und Bilder, lange Dokumente blockweise.
// Aufruf: node tools/e2e-markdown.mjs <Projektkopie (wird verändert!)> [Bilderordner]
import { webkit } from "playwright";
import { mkdirSync, writeFileSync, readFileSync } from "node:fs";
import { join } from "node:path";
const [proj, shots] = process.argv.slice(2);
if (!proj) { console.error("Aufruf: node tools/e2e-markdown.mjs <Projektkopie> [Bilder]"); process.exit(2); }
if (shots) mkdirSync(shots, { recursive: true });
const PNG = Buffer.from("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==", "base64");
mkdirSync(join(proj, "doku"), { recursive: true }); writeFileSync(join(proj, "doku", "punkt.png"), PNG);
const lang = ["# Lang\n"]; for (let i = 0; i < 260; i++) lang.push(`Absatz ${i}\n`);
writeFileSync(join(proj, "lang.md"), lang.join("\n"));
writeFileSync(join(proj, "notizen.md"), "# Notizen\n\nText mit **fett**, *kursiv*, `code` und ~~weg~~.\n\n- [ ] eins\n- [x] zwei\n\n1. a\n2. b\n\n> Zitat\n\n| Name | Wert |\n|:--|--:|\n| a | 1 |\n\n```asm\nlda #1\n```\n\n[Web](https://example.org) [Quelle](jumprun.asm:3) [Anker](#notizen) [Böse](javascript:alert(1))\n\n![Punkt](doku/punkt.png) ![Fehlt](doku/nix.png)\n\n<script>alert(1)</script>\n");
const browser = await webkit.launch(); const page = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
const fails = []; let n = 0; const ok = (c, m) => { n++; if (!c) { fails.push(m); console.log("FEHLER:", m); } else console.log("ok:", m); };
page.on("pageerror", (e) => { fails.push("Seitenfehler " + e.message); console.log("Seitenfehler", e.message); });
const snap = async (name) => { if (shots) await page.screenshot({ path: join(shots, name + ".png") }); };
const warte = (ms = 450) => page.waitForTimeout(ms);
await page.goto("http://127.0.0.1:1420/?bridge=17420"); await page.waitForFunction(() => window.__vc && window.__vc.ui.ready);
await page.evaluate(async (p) => { await window.__vc.setLanguage("de"); await window.__vc.openProject(p); await window.__vc.call("tree_reload"); await window.__vc.openFile("notizen.md"); }, proj); await warte(900);
await page.waitForSelector("[data-md=vorschau]"); await warte(300);
ok(await page.locator("[data-md=vorschau] h1").innerText() === "Notizen", "Markdown-Datei öffnet in der Vorschau (Überschrift)");
ok(await page.locator("[data-md=vorschau] strong").innerText() === "fett" && await page.locator("[data-md=vorschau] em").count() === 1 && await page.locator("[data-md=vorschau] s").count() === 1, "Fett, kursiv, durchgestrichen");
ok(await page.locator("[data-md=vorschau] blockquote").count() === 1 && await page.locator("[data-md=vorschau] ol li").count() === 2, "Zitat und nummerierte Liste");
ok(await page.locator("[data-md=tabelle] th").count() === 2 && (await page.locator("[data-md=tabelle] td").nth(1).getAttribute("style") ?? "").includes("right"), "Tabelle mit Ausrichtung");
ok(await page.locator("[data-md=code]").innerText().then((x) => /lda #1/.test(x)), "Codeblock");
ok(await page.locator("[data-md=bild]").count() === 1 && await page.locator("[data-md=bild-fehlt]").count() === 1, "Bild aus dem Projekt wird gezeigt, fehlendes als Platzhalter");
ok(await page.locator("[data-md=html]").innerText().then((x) => /<script>/.test(x)) && await page.locator("[data-md=vorschau] script").count() === 0, "Rohes HTML erscheint als Text und läuft nie");
ok(await page.locator("[data-md=link][data-kind=blocked]").count() === 0 && await page.locator("[data-md=vorschau] .gesperrt").count() === 1, "Gefährliches Linkschema ist gesperrt (kein Link)");
await snap("md-1-vorschau");
// Aufgabe abhaken
await page.locator("[data-md=aufgabe]").first().check(); await warte(500);
const roh = await page.evaluate(() => window.__vc.ed.tabs.find((t) => t.rel === "notizen.md")?.dirty);
ok(roh === true, "Aufgabe abhaken ändert den Text (Reiter ist geändert)");
// Rückgängig: die Vorschau zeichnet sich von selbst neu (Handtest)
{ const haken = () => page.locator("[data-md=aufgabe]").first().isChecked();
  ok(await haken() === true, "Vorbereitung: Aufgabe ist abgehakt");
  await page.evaluate(() => window.__vc.runCommand("rueckgaengig")); await warte(700);
  ok(await haken() === false, "Nach Rückgängig zeigt die Vorschau die Aufgabe wieder offen (ohne Neuladen)");
  await page.evaluate(() => window.__vc.runCommand("wiederholen")); await warte(700);
  ok(await haken() === true, "Wiederholen: Vorschau folgt erneut"); }
// Links
await page.locator("[data-md=link][data-kind=web]").click(); await warte(400);
ok(await page.locator(".toasts").innerText().then((x) => /Browser/.test(x)), "Web-Link öffnet nur auf Klick (im Test ohne Start) und meldet es");
await page.locator("[data-md=link][data-kind=file]").click(); await warte(800);
ok(await page.evaluate(() => window.__vc.ed.active) === "jumprun.asm", "Datei-Link springt in die Quelldatei");
await page.evaluate(async () => { await window.__vc.openFile("notizen.md"); }); await warte(500);
// Umschalten und Schreibhilfen
await page.locator("[data-md=quelltext-an]").click(); await warte(400);
ok(await page.locator(".cm-editor").isVisible() && await page.locator("[data-md=format]").count() >= 10, "Quelltext: Editor und Schreibhilfen");
await warte(500); ok(await page.locator(".tok-md-heading").count() >= 1 && await page.locator(".tok-md-bold").count() >= 1 && await page.locator(".tok-md-code").count() >= 1, "Quelltext ist gefärbt (Überschrift, fett, Code)");
await page.locator(".cm-content").click(); await page.keyboard.press("Meta+ArrowUp"); await page.keyboard.press("Control+Home");
await page.locator("[data-md=format][data-id=md_tiefer]").click(); await warte(300);
const zeile1 = await page.locator(".cm-line").first().innerText(); ok(/^## Notizen/.test(zeile1), "Überschrift tiefer: ## Notizen");
await page.keyboard.press("Meta+z"); await page.keyboard.press("Control+z"); await warte(300);
ok(/^# Notizen/.test(await page.locator(".cm-line").first().innerText()), "Rückgängig macht die Schreibhilfe in einem Schritt rückgängig");
await page.locator(".cm-content").click(); await page.keyboard.press("Meta+ArrowUp"); await page.keyboard.press("Control+Home"); await page.keyboard.press("End");
await page.locator("[data-md=format][data-id=md_fett]").click(); await warte(300);
const fett = await page.locator(".cm-line").first().innerText(); ok(/\*\*Notizen\*\*/.test(fett), "Fett ohne Auswahl umschließt das Wort: " + fett);
await snap("md-2-quelltext");
await page.evaluate(async () => { await window.__vc.runCommand("md_umschalten"); }); await warte(400);
ok(await page.locator("[data-md=vorschau]").isVisible(), "Befehl „Vorschau/Quelltext umschalten“ wechselt zurück");
// Befehl in der Vorschau wechselt in den Quelltext
await page.evaluate(async () => { await window.__vc.runCommand("md_kursiv"); }); await warte(500);
ok(await page.locator(".cm-editor").isVisible(), "Schreibhilfe-Befehl in der Vorschau wechselt in den Quelltext");
// Keine Markdown-Datei: Hinweis
await page.evaluate(async () => { await window.__vc.openFile("jumprun.asm"); }); await warte(500); await page.evaluate(async () => { await window.__vc.runCommand("md_fett"); }); await warte(300);
ok(await page.locator(".toasts").innerText().then((x) => /keine Markdown/.test(x)) && (await page.locator("[data-md=leiste]").count()) === 0, "In einer Assemblerdatei: Hinweis, keine Markdown-Leiste");
// Lange Dokumente blockweise
await page.evaluate(async () => { await window.__vc.openFile("lang.md"); }); await warte(900);
const erst = await page.locator("[data-md=vorschau] p").count(); ok(erst > 0 && erst < 260 && await page.locator("[data-md=mehr]").count() === 1, "Langes Dokument erscheint blockweise (" + erst + " von 260)");
await page.locator("[data-md=mehr]").click(); await warte(800); ok(await page.locator("[data-md=vorschau] p").count() > erst, "„Weitere Blöcke anzeigen“ lädt nach");
await page.setViewportSize({ width: 640, height: 800 }); await warte(300); await snap("md-3-schmal"); await page.setViewportSize({ width: 1440, height: 900 });
console.log("\n" + n + " Prüfungen, " + fails.length + " Fehler"); await browser.close(); process.exit(fails.length ? 1 : 0);
