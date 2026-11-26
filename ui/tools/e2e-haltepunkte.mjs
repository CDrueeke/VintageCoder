// SPDX-License-Identifier: GPL-3.0-or-later
// Ablauftest Haltepunkte und Lesezeichen über Neustart (Handtest): je Projekt in der Sitzung gespeichert, bei geändertem Text wiedergefunden.
// Aufruf: node tools/e2e-haltepunkte.mjs <Projektkopie (wird verändert!)> <HOME des Kerns>
import { webkit } from "playwright";
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
const [proj, home] = process.argv.slice(2);
if (!proj || !home) { console.error("Aufruf: node tools/e2e-haltepunkte.mjs <Projektkopie> <HOME>"); process.exit(2); }
const cfg = join(home, ".config/vintagecoder");
const lies = () => { try { return JSON.parse(readFileSync(join(cfg, "session.json"), "utf8")).projects?.[proj]; } catch { return null; } };
const browser = await webkit.launch(); const page = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
const fails = []; let n = 0;
const ok = (c, m) => { n++; if (!c) { fails.push(m); console.log("FEHLER:", m); } else console.log("ok:", m); };
page.on("pageerror", (e) => { fails.push("Seitenfehler " + e.message); console.log("Seitenfehler", e.message); });
const laden = async () => { await page.goto("http://127.0.0.1:1420/?bridge=17420"); await page.waitForFunction(() => window.__vc && window.__vc.ui.ready); await page.evaluate(async () => { await window.__vc.setLanguage("de"); }); };
const oeffne = async (rel) => { await page.evaluate(async ([p, r]) => { await window.__vc.openProject(p); await window.__vc.openFile(r); }, [proj, rel]); await page.waitForSelector(".cm-editor .cm-content"); await page.waitForTimeout(700); };
const nummer = (k) => page.locator(".cm-lineNumbers .cm-gutterElement", { hasText: new RegExp("^" + k + "$") }).first();
const bps = async () => (await page.locator(".mk-breakpoint").count()) - 1;   // eine Marke ist der unsichtbare Platzhalter des Randes
await laden();
const rel = (await (async () => { await page.evaluate(async (p) => { await window.__vc.openProject(p); }, proj); await page.waitForTimeout(700); return page.evaluate(() => { const f = (ns) => ns.flatMap((x) => (x.kind === "dir" ? f(x.children ?? []) : [x])); return f(window.__vc.ui.project.tree).map((x) => x.rel).find((r) => r === "jumprun.asm"); }); })());
ok(!!rel, "Projekt hat eine .asm-Datei: " + rel);
await oeffne(rel);
await nummer(29).click(); await page.waitForTimeout(250); await nummer(32).click(); await page.waitForTimeout(900);
ok(await bps() === 2, "Zwei Haltepunkte gesetzt (Zeilen 29 und 32)");
const s = lies(); const m = s?.marks?.[rel];
ok(m && m.breakpoints.length === 2 && m.breakpoints.map((x) => x.line).join() === "29,32" && m.breakpoints.every((x) => typeof x.text === "string"), "session.json speichert die Haltepunkte je Datei mit Zeilentext: " + JSON.stringify(m?.breakpoints));
// Neustart: Fenster neu laden, Projekt und Datei öffnen
await laden(); await oeffne(rel);
ok(await bps() === 2, "Nach dem Neustart sind die Haltepunkte wieder da");
// Datei ändern: zwei Zeilen am Anfang eingefügt (außerhalb des Editors), Haltepunkte folgen dem Zeilentext
await page.evaluate(async () => { await window.__vc.runCommand("datei_schliessen"); }); await page.waitForTimeout(400);
const pfad = join(proj, rel); const text = readFileSync(pfad, "utf8"); writeFileSync(pfad, "; neu 1\n; neu 2\n" + text);
await page.evaluate(async (r) => { await window.__vc.openFile(r); }, rel); await page.waitForTimeout(800);
ok(await bps() === 2, "Bei geändertem Text bleiben die Haltepunkte (an ihrer Zeile wiedergefunden)");
await page.evaluate(async () => { await window.__vc.runCommand("datei_schliessen"); }); await page.waitForTimeout(400);
// Zeile gelöscht: Haltepunkt entfällt
writeFileSync(pfad, text.split("\n").filter((_, i) => i !== 28).join("\n"));
await page.evaluate(async (r) => { await window.__vc.openFile(r); }, rel); await page.waitForTimeout(800);
ok(await bps() === 1, "Wurde die Zeile eines Haltepunkts gelöscht, entfällt er (der andere bleibt)");
// Start mit --open: die Datei ist schon offen, wenn die Sitzung gelesen wird: die Marken werden nachträglich eingesetzt (und überschreiben nichts)
await laden(); await oeffne(rel);
await page.evaluate(async () => { await window.__vc.runCommand("datei_schliessen"); window.__vc.markierungenLaden({}); }); await page.waitForTimeout(300);   // gemerkte Marken leeren: die Datei öffnet ohne Haltepunkte, wie beim Start mit --open vor dem Lesen der Sitzung
writeFileSync(pfad, text); await page.evaluate(async (r) => { await window.__vc.openFile(r); }, rel); await page.waitForTimeout(600);
const vorher = await bps();
await page.evaluate(() => window.__vc.markierungenLaden({ "jumprun.asm": { breakpoints: [{ line: 29, text: ".cpu 6510 { illegals = deny }           ; CPU 6510; undokumentierte Befehle sind verboten" }], bookmarks: [] } })); await page.waitForTimeout(500);
ok(await bps() === vorher + 1, "Ist die Datei schon offen (Start mit --open), setzt das spätere Lesen der Sitzung die Haltepunkte nachträglich ein");
console.log(`\n${n} Prüfungen, ${fails.length} Fehler`);
await browser.close(); process.exit(fails.length ? 1 : 0);
