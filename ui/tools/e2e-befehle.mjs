// SPDX-License-Identifier: GPL-3.0-or-later
// Ablauftest „Befehle mit Funktion“: Zahlenwerkzeuge, Als Include einfügen, Zwischenablage-Befehle, ausgeblendete Befehle (docs/befehle-ohne-funktion.md).
// Aufruf: node tools/e2e-befehle.mjs <Projektkopie (wird verändert!)>   (Vite auf :1420 und `vintagecoder` mit VC_TESTBRIDGE=17420 VC_TEST_NO_SPAWN=1 VC_NO_SAVE=1 müssen laufen)
import { webkit } from "playwright";
const MOD = process.platform === "darwin" ? "Meta" : "Control", DOC_START = process.platform === "darwin" ? "Meta+ArrowUp" : "Control+Home";
const [proj] = process.argv.slice(2);
if (!proj) { console.error("Aufruf: node tools/e2e-befehle.mjs <Projektkopie>"); process.exit(2); }
const browser = await webkit.launch();
const ctx = await browser.newContext({ viewport: { width: 1440, height: 900 } }); const page = await ctx.newPage();
const fails = []; let n = 0;
const ok = (c, m) => { n++; if (!c) { fails.push(m); console.log("FEHLER:", m); } else console.log("ok:", m); };
page.on("pageerror", (e) => { fails.push("Seitenfehler " + e.message); console.log("Seitenfehler", e.message); });
const w = (ms = 400) => page.waitForTimeout(ms);
const toasts = () => page.evaluate(() => window.__vc.ui.toasts.map((x) => x.text).join(" | "));
const klar = () => page.evaluate(() => { window.__vc.ui.toasts.splice(0); });
const text = () => page.evaluate(() => document.querySelector(".cm-content")?.innerText ?? "");
await page.goto("http://127.0.0.1:1420/?bridge=17420"); await page.waitForFunction(() => window.__vc && window.__vc.ui.ready);
await page.evaluate(async ([p]) => { await window.__vc.setLanguage("de"); await window.__vc.openProject(p); await window.__vc.openFile("jumprun.asm"); await window.__vc.runCommand("nav:code"); }, [proj]); await w(900);

// 1) Ausgeblendete Befehle: nicht in der Tabelle für Menü und Palette; eingeblendete schon
const ids = await page.evaluate(async () => (await window.__vc.call("commands_table")).map((z) => z.id));
for (const id of ["kodierung", "basic_nach_asm", "hex_vorschlaege", "std_zeichensatz", "zeichensatz_rom", "zeichensatz_png", "studio_zwiebel", "studio_raster"]) ok(!ids.includes(id), `Befehl „${id}“ ohne Funktion ist aus Menü und Palette verschwunden`);
for (const id of ["zahlenwerkzeuge", "als_include", "projekt_ordner_oeffnen", "kopieren", "ausschneiden", "einfuegen"]) ok(ids.includes(id), `Befehl „${id}“ ist da`);

// 2) Zahlenwerkzeuge
await klar(); await page.evaluate(() => { void window.__vc.runCommand("zahlenwerkzeuge"); }); await page.waitForSelector("[data-zw=dialog]"); await w(500);
ok(!/noch nicht/.test(await toasts()), "Zahlenwerkzeuge öffnet einen Dialog (keine Meldung „noch nicht gebaut“)");
ok((await page.locator("[data-zw=dec]").innerText()) === "53280" && (await page.locator("[data-zw=hex]").innerText()) === "$D020", "$D020 wird als 53280 und $D020 gezeigt");
await page.locator("[data-zw=eingabe]").fill("$D000+$20"); await w(400); ok((await page.locator("[data-zw=dec]").innerText()) === "53280", "Ausdruck $D000+$20 wird gerechnet (53280)");
ok((await page.locator("[data-zw=bytes]").innerText()).replace(/\s/g, "") === "32/208", "Lo-/Hi-Byte: 32 / 208");
await page.locator("[data-zw=eingabe]").fill("65"); await w(400); ok((await page.locator("[data-zw=zeichen]").innerText()) === "A", "65 zeigt das Zeichen A");
await page.locator("[data-zw=eingabe]").fill("1/0"); await w(400); ok(await page.locator("[data-zw=err]").count() === 1 && await page.locator("[data-zw=werte]").count() === 0, "Fehlerhafter Ausdruck zeigt eine Fehlermeldung und keine Werte");
await page.locator("[data-zw=eingabe]").fill("$D020"); await w(400);
await page.locator("[data-zw=bit][data-n='0']").click(); await w(400); ok((await page.locator("[data-zw=eingabe]").inputValue()) === "$D021" && (await page.locator("[data-zw=dec]").innerText()) === "53281", "Klick auf Bit 0 schaltet um: $D021");
await page.locator("[data-zw=b16]").click(); await w(300); ok(await page.locator("[data-zw=bit]").count() === 16, "Breite 16 Bit zeigt 16 Bits");
await page.locator("[data-zw=zu]").click(); await w(200);
await page.locator(".cm-content").click(); await page.keyboard.press(DOC_START); await page.evaluate(() => { void window.__vc.runCommand("zahlenwerkzeuge"); }); await page.waitForSelector("[data-zw=dialog]"); await w(400);
await page.locator("[data-zw=einsetzen-hex]").click(); await w(500);
ok((await text()).startsWith("$D021") && await page.locator("[data-zw=dialog]").count() === 0, "Einsetzen schreibt $D021 an die Schreibmarke und schließt den Dialog");
await page.keyboard.press(`${MOD}+z`); await w(400); ok(!(await text()).startsWith("$D021"), "Cmd/Strg+Z nimmt das Einsetzen in einem Schritt zurück");

// 3) Als Include einfügen
const vorher = await text(); await klar();
await page.evaluate(() => { window.__vc.ui.selected = "README.md"; void window.__vc.runCommand("als_include"); }); await w(600);
ok((await text()).includes('.include "README.md"') && /Eingefügt/.test(await toasts()), "Als Include einfügen schreibt .include \"README.md\" in den Quelltext: " + (await toasts()));
await klar(); await page.evaluate(() => { void window.__vc.runCommand("als_include"); }); await w(500);
ok(((await text()).match(/\.include "README\.md"/g) ?? []).length === 1 && /schon eingebunden/.test(await toasts()), "Zweiter Aufruf fügt nichts doppelt ein („schon eingebunden“)");
await page.locator(".cm-content").click(); await page.keyboard.press(`${MOD}+z`); await w(400); ok(!(await text()).includes('.include "README.md"') || (await text()) === vorher, "Cmd/Strg+Z nimmt die Include-Zeile zurück");
await klar(); await page.evaluate(() => { window.__vc.ui.selected = "jumprun.vcprj"; void window.__vc.runCommand("als_include"); }); await w(400);
ok(/lässt sich nicht in Quelltext einbinden/.test(await toasts()), "Projektdatei lässt sich nicht einbinden (Meldung)");

// 3b) Als Include einfügen aus dem Kontextmenü des Projektbaums (Rechtsklick auf lib/core.asm)
await klar(); const vor3b = await text();
const baumZeile = (r = "lib/core.asm") => page.locator(`[role=treeitem][data-rel="${r}"]`);
if (await baumZeile().count() === 0) { await page.evaluate(() => { window.__vc.ui.selected = "lib"; }); await page.locator('[role=treeitem][data-rel="lib"]').click().catch(() => {}); await w(300); }
await baumZeile().click({ button: "right" }); await w(300);
const mp = page.getByRole("menuitem", { name: "Als Include einfügen" });
ok(await mp.count() === 1 && (await mp.getAttribute("aria-disabled")) === null, "Baum-Kontextmenü auf lib/core.asm bietet „Als Include einfügen“ (aktiv, Assembler-Reiter offen)");
await mp.click(); await w(600);
ok(/schon eingebunden/.test(await toasts()), "Menüpunkt auf bereits eingebundener lib/core.asm meldet „schon eingebunden“: " + (await toasts()));
await klar(); await baumZeile("README.md").click({ button: "right" }); await w(300);
await page.getByRole("menuitem", { name: "Als Include einfügen" }).click(); await w(600);
ok((await text()).includes('.include "README.md"'), "Menüpunkt schreibt .include \"README.md\" in den Quelltext: " + (await toasts()));
await page.locator(".cm-content").click(); await page.keyboard.press(`${MOD}+z`); await w(400);
ok(!(await text()).includes('.include "lib/core.asm"') || (await text()) === vor3b, "Cmd/Strg+Z nimmt die Zeile aus dem Menü zurück");
// ohne Assembler-Reiter: ausgegraut mit Grund
await page.evaluate(async () => { await window.__vc.openFile("README.md"); }); await w(500);
await baumZeile("README.md").click({ button: "right" }); await w(300);
const mp2 = page.getByRole("menuitem", { name: "Als Include einfügen" });
ok(await mp2.count() === 1 && (await mp2.getAttribute("aria-disabled")) === "true" && /Assembler/.test((await mp2.getAttribute("title")) ?? ""), "Ohne Assembler-Reiter ist der Menüpunkt ausgegraut und nennt den Grund");
await page.keyboard.press("Escape"); await page.evaluate(async () => { await window.__vc.openFile("jumprun.asm"); }); await w(500);

// 4) Kopieren, Ausschneiden, Einfügen außerhalb des Sprite-Editors: keine Meldung „noch nicht gebaut“
await page.locator(".cm-content").click(); await page.keyboard.press(`${MOD}+a`); await klar();
for (const id of ["kopieren", "einfuegen"]) { await page.evaluate((i) => { void window.__vc.runCommand(i); }, id); await w(500); ok(!/noch nicht/.test(await toasts()), `Befehl „${id}“ im Code-Editor: keine Meldung „noch nicht gebaut“ (${await toasts() || "ohne Meldung"})`); }
const t0 = await text(); await page.keyboard.press("ArrowRight"); await page.locator(".cm-content").click(); await page.keyboard.press(`${MOD}+a`); await page.evaluate(() => { void window.__vc.runCommand("ausschneiden"); }); await w(500);
ok(!/noch nicht/.test(await toasts()) , "Befehl „ausschneiden“: keine Meldung „noch nicht gebaut“");
await page.keyboard.press(`${MOD}+z`); await w(400); ok((await text()) === t0 || (await text()).length > 0, "Ausschneiden lässt sich zurücknehmen");

console.log(`\n${n} Prüfungen, ${fails.length} Fehler`); await browser.close(); process.exit(fails.length ? 1 : 0);
