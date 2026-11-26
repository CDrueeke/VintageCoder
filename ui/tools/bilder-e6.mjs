// SPDX-License-Identifier: GPL-3.0-or-later
// Bilderstrecke für die Abnahme (E6 Debug): Playwright (WebKit), Prüfbrücke mit Attrappe (VC_TEST_FAKE_DEBUG=1). Aufruf: node tools/bilder-e6.mjs <Kopie c64/jumprun> <Ausgabeordner>
import { webkit } from "playwright";
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
const [proj, out] = process.argv.slice(2);
if (!out) { console.error("Aufruf: node tools/bilder-e6.mjs <jumprun> <Ausgabe>"); process.exit(2); }
mkdirSync(out, { recursive: true });
const browser = await webkit.launch(); const page = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
const liste = []; const bild = async (name, punkt, soll) => { await page.waitForTimeout(500); await page.screenshot({ path: join(out, name + ".png") }); liste.push([punkt, name + ".png", soll]); };
const V = (js, a) => page.evaluate(js, a); const st = () => V(() => window.__vc.dbg.s); const w = (ms = 500) => page.waitForTimeout(ms);
await page.goto("http://127.0.0.1:1420/?bridge=17420"); await page.waitForFunction(() => window.__vc && window.__vc.ui.ready);
await V(async () => { await window.__vc.setLanguage("de"); window.__vc.setTheme("dark"); });
await V(async (p) => { await window.__vc.openProject(p); }, proj); await page.waitForTimeout(900);
await V(() => window.__vc.runCommand("nav:debug")); await w();
await bild("e6-01-leer", "E6-1", "Keine Sitzung: Hinweis und Knopf „Debuggen starten“");
await page.locator("[data-dbg=start]").click(); await page.waitForSelector("[data-dbg=register]", { timeout: 60000 }); await w(800);
await bild("e6-02-halt-dunkel-breit", "E6-2", "Sitzung angehalten: links Haltepunkte, Beobachten, Momentaufnahme; Mitte Symbolleiste, Disassembler (aktuelle Zeile hervorgehoben, Bytes, Zyklen), Speicher; rechts Register, Flags, Stapel, Bildschirm");
await page.locator("[data-dbg=hinein]").click(); await w(600);
await bild("e6-03-schritt", "E6-3", "Nach einem Schritt: geänderte Register umrandet, geänderte Flags markiert");
const s = await st(); const k = s.disasm.filter((d) => d.addr > s.regs.pc && d.line && /\.asm$/.test(d.line.file))[2] ?? s.disasm[4];
await page.locator(`[data-dbg=row][data-a='${k.addr}'] [data-dbg=bp]`).click(); await w();
await page.locator("[data-dbg=adr]").fill("$0900"); await page.locator("[data-dbg=bed]").fill("a == 5"); await page.locator("[data-dbg=punkt-neu]").click(); await w();
await page.locator("[data-dbg=beob]").fill("a"); await page.locator("[data-dbg=beob-neu]").click(); await w(300); await page.locator("[data-dbg=beob]").fill("d020"); await page.locator("[data-dbg=beob-neu]").click(); await w(300); await page.locator("[data-dbg=beob]").fill("$0801"); await page.locator("[data-dbg=beob-neu]").click(); await w();
await bild("e6-04-punkte-beobachten", "E6-4/6", "Haltepunkt im Disassembler (roter Punkt), Haltepunkt mit Bedingung, Beobachten mit Wert (hex · dezimal) und dem Hinweis „Meinst du $d020?“");
await page.locator("[data-dbg=weiter]").click(); await w(1500);
await bild("e6-05-treffer", "E6-5", "Weiter: das Programm hielt am Haltepunkt; Status nennt Adresse und Quelltextzeile");
await page.locator("[data-dbg=mem]").fill("0801"); await page.locator("[data-dbg=mem-los]").click(); await w();
await page.locator("[data-dbg=erkennen]").click(); await w(800); await page.locator("[data-dbg=bild-neu]").click(); await w(900);
await bild("e6-06-speicher-moment-bild", "E6-7/8/9", "Speicher ab $0801, Momentaufnahme (Erkennen), Bild des Emulators (Attrappe: Farbverlauf)");
await V(() => window.__vc.runCommand("nav:code")); await w(); await V(() => window.__vc.openFile("jumprun.asm")); await w(900);
await bild("e6-07-editor-zeile", "E6-10", "Im Code-Bereich: die Ausführungszeile ist gelb hinterlegt");
await V(() => window.__vc.runCommand("nav:debug")); await w();
await page.setViewportSize({ width: 640, height: 900 }); await bild("e6-08-schmal-640", "E6-11", "Schmal (640 px): Spalten untereinander"); await page.setViewportSize({ width: 1440, height: 900 });
await V(() => window.__vc.setTheme("light")); await bild("e6-09-hell", "E6-11", "Helles Farbschema"); await V(async () => { window.__vc.setTheme("dark"); await window.__vc.setLanguage("en"); }); await bild("e6-10-englisch", "E6-11", "Englisch (maschinell, nicht abgenommen)"); await V(async () => { await window.__vc.setLanguage("de"); });
await page.locator("[data-dbg=ende]").click(); await w(600);
writeFileSync(join(out, "LIESMICH.md"), "# Bilderstrecke E6 Debug (de, dunkel, wenn nicht anders genannt)\n\nErzeugt mit `node ui/tools/bilder-e6.mjs` (Playwright/WebKit, Prüfbrücke, echter Kern, **Attrappe statt Emulator**: Register und Disassembler sind Platzhalterwerte, das Verhalten (Schritt, Haltepunkt, Beobachten, Speicher) ist echt).\n\n| Handprüfpunkt | Bild | Was zu sehen sein muss |\n|---|---|---|\n" + liste.map(([p, f, s]) => `| ${p} | ${f} | ${s} |`).join("\n") + "\n\nNur von Hand mit echtem Emulator zu prüfen: Haltepunkt trifft im echten Programm, Schrittfolge, Bild des Emulators (Zeitlupe), Beenden ohne hängenden Emulator, Hardware (Ultimate, MEGA65 per USB).\n");
console.log(liste.length + " Bilder"); await browser.close();
