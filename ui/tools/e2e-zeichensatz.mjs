// SPDX-License-Identifier: GPL-3.0-or-later
// Ablauftest Zeichensatz-Editor (E4): Playwright (WebKit) gegen die Prüfbrücke des echten Kerns.
// Aufruf: node tools/e2e-zeichensatz.mjs <Projektordner mit graphics/font.vcchar (wird verändert!)> [Bilderordner]
import { webkit } from "playwright";
import { mkdirSync } from "node:fs";
import { join } from "node:path";

if (process.platform === "win32" && !process.env.VC_E2E_WIN) { console.log("übersprungen unter Windows (Playwright-WebKit verliert Mausfolgen)\n0 Prüfungen, 0 Fehler"); process.exit(0); }
const [proj, shots] = process.argv.slice(2);
if (!proj) { console.error("Aufruf: node tools/e2e-zeichensatz.mjs <Projektordner> [Bilder]"); process.exit(2); }
if (shots) mkdirSync(shots, { recursive: true });
const browser = await webkit.launch();
const page = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
const fails = []; let n = 0;
const ok = (c, m) => { n++; if (!c) { fails.push(m); console.log("FEHLER:", m); } else console.log("ok:", m); };
page.on("pageerror", (e) => { fails.push("Seitenfehler " + e.message); console.log("Seitenfehler", e.message); });
const snap = async (name) => { if (shots) await page.screenshot({ path: join(shots, name + ".png") }); };
await page.goto("http://127.0.0.1:1420/?bridge=17420"); await page.waitForFunction(() => window.__vc && window.__vc.ui.ready);
await page.evaluate(async (p) => { await window.__vc.setLanguage("de"); await window.__vc.openProject(p); }, proj); await page.waitForTimeout(900);
const sv = () => page.evaluate(() => window.__vc.sp.view);
const geo = () => page.evaluate(() => { const c = document.querySelector("canvas[data-cw]"); const r = c.getBoundingClientRect(); return { l: r.left + +c.dataset.ox, t: r.top + +c.dataset.oy, cw: +c.dataset.cw, ch: +c.dataset.ch }; });
const pt = async (x, y) => { const g = await geo(); return [g.l + (x + 0.5) * g.cw, g.t + (y + 0.5) * g.ch]; };

// 1) Öffnen: Zeichensatz-Dateien kommen in den Zeichensatz-Editor
await page.evaluate(() => window.__vc.openFile("graphics/font.vcchar")); await page.waitForTimeout(800);
let v = await sv();
ok(v && v.kind === "charset" && v.frames === 256 && v.width === 8 && v.height === 8, "font.vcchar öffnet als Zeichensatz (256 Zeichen, 8 × 8)");
ok(await page.evaluate(() => window.__vc.ui.mode === "grafik" && window.__vc.ui.sub.grafik === "zeichensatz"), "Die Ansicht wechselt zu Grafik ▸ Zeichensatz");
ok((await page.locator("[data-cs=grid]").count()) === 1 && (await page.locator("[data-cs=vorschau]").count()) === 1, "Übersicht aller Zeichen und Vorschau im Bild sind da");
{ const r = page.locator("[data-cs=grid] [role=listbox]"); const n = await page.locator("[data-cs=zelle][role=option]").count(); const label = await r.getAttribute("aria-label");
  const tab = await page.locator("[data-cs=zelle][tabindex='0']").count(); const o5 = page.locator("[data-cs=zelle]").nth(5);
  const l5 = await o5.getAttribute("aria-label"), pos = await o5.getAttribute("aria-posinset"), size = await o5.getAttribute("aria-setsize");
  ok(n === 256 && label === "Zeichen" && tab === 1 && /^Zeichen 5, \$05$/.test(l5) && pos === "6" && size === "256" && (await page.locator("[data-cs=grid] canvas").getAttribute("aria-hidden")) === "true", "Bildschirmleser: Auswahlliste „Zeichen“ mit 256 Optionen, eine per Tab, Label kurz: „" + l5 + "“, Position " + pos + " von " + size + ", Canvas verborgen");
  ok((await page.locator("[data-cs=grid] [role=grid], [data-cs=grid] [role=row], [data-cs=grid] [role=gridcell], [data-cs=grid] [role=application], [data-cs=grid] table").count()) === 0, "Keine Raster-, Tabellen- oder Anwendungsrolle (löst lange VoiceOver-Hinweise aus)"); }
await snap("zeichensatz-1-offen");

// 2) Zeichen wählen: Klick in die Übersicht, Pfeiltasten
const gb = await page.locator("[data-cs=grid]").boundingBox(); const z = gb.width / 16;
await page.mouse.click(gb.x + 2.5 * z, gb.y + 0.5 * z); await page.waitForTimeout(300); v = await sv();
ok(v.fi === 2, "Klick auf das dritte Feld wählt Zeichen 2 (B)");
ok(/Zeichen 2 · \$02/.test(await page.locator("[data-cs=chip]").innerText()) && /2 · \$02/.test(await page.locator("[data-cs=nummer]").innerText()), "Zeichenmarke und Nummer nennen Zeichen 2 · $02");
await page.keyboard.press("ArrowRight"); await page.waitForTimeout(250); ok((await sv()).fi === 3 && (await page.evaluate(() => document.activeElement?.getAttribute("data-i"))) === "3", "Pfeil rechts: Zeichen 3, der Fokus wandert in die Zelle 3");
await page.keyboard.press("ArrowDown"); await page.waitForTimeout(250); ok((await sv()).fi === 19, "Pfeil unten: eine Zeile tiefer (Zeichen 19)");
await page.keyboard.press("Home"); await page.waitForTimeout(250); ok((await sv()).fi === 0, "Pos1 springt zum ersten Zeichen");
await page.keyboard.press("End"); await page.waitForTimeout(250); ok((await sv()).fi === 255, "Ende springt zum letzten Zeichen");
ok((await page.locator("[data-cs=nachbar]").count()) === 2, "Umgebung am Rand: nur die vorhandenen Nachbarn");
await page.locator("[data-cs=nachbar][data-i='254']").click(); await page.waitForTimeout(250); ok((await sv()).fi === 254 && (await page.locator("[data-cs=nachbar]").count()) === 3, "Klick auf einen Nachbarn wählt ihn, drei Nachbarn in der Mitte");

// 2b) Roving Focus ohne Neuaufbau (VoiceOver liest sonst bei jedem Zeichen den Fensterkontext vor): dieselben Knoten, genau ein Fokuswechsel
await page.locator("[data-cs=zelle]").nth(98).click(); await page.waitForTimeout(300);
await page.evaluate(() => { window.__ev = []; window.__opt = document.querySelector("[data-cs=zelle][data-i='98']"); window.__lb = document.querySelector("[data-cs=grid] [role=listbox]"); window.__nodes = [...document.querySelectorAll("[data-cs=zelle]")];
  for (const t of ["focusin", "focusout"]) document.addEventListener(t, (e) => window.__ev.push(t + ":" + (e.target.getAttribute?.("data-i") ?? e.target.tagName)), true); });
await page.keyboard.press("ArrowRight"); await page.waitForTimeout(500);
{ const r = await page.evaluate(() => ({ ev: window.__ev, opt: window.__opt === document.querySelector("[data-cs=zelle][data-i='98']"), lb: window.__lb === document.querySelector("[data-cs=grid] [role=listbox]"), alle: window.__nodes.every((n, i) => n === document.querySelectorAll("[data-cs=zelle]")[i]) && document.querySelectorAll("[data-cs=zelle]").length === 256 }));
  ok(r.opt && r.lb && r.alle, "Pfeil rechts: Optionen und Listenelement bleiben dieselben DOM-Knoten (nichts wird neu aufgebaut)");
  ok(JSON.stringify(r.ev) === JSON.stringify(["focusout:98", "focusin:99"]), "Genau ein Fokuswechsel von Option 98 auf 99: " + r.ev.join(", ")); }
await page.keyboard.press("ArrowLeft"); await page.waitForTimeout(300);

// 3) Zeichnen im 8×8-Editor, Rückgängig, Speichern
await page.locator("[data-cs=grid]").click({ position: { x: 2.5 * z, y: 0.5 * z } }); await page.waitForTimeout(250);
const vorher = (await sv()).pixels.join("");
const [x0, y0] = await pt(0, 7); await page.mouse.click(x0, y0); await page.waitForTimeout(300); v = await sv();
ok(v.pixels.join("") !== vorher && v.dirty, "Ein Punkt gesetzt: Zeichen geändert, Datei als geändert gekennzeichnet");
ok(v.thumbs[2].pixels.join("") === v.pixels.join(""), "Die Übersicht zeigt das geänderte Zeichen sofort");
await page.locator("[data-act=undo]").click(); await page.waitForTimeout(300); ok((await sv()).pixels.join("") === vorher, "Rückgängig stellt das Zeichen her");
await page.mouse.click(x0, y0); await page.waitForTimeout(300);
await page.locator("[data-act=save]").click(); await page.waitForTimeout(500); ok(!(await sv()).dirty, "Speichern: nicht mehr als geändert gekennzeichnet");
await page.locator("[data-act=undo]").click(); await page.waitForTimeout(300); await page.locator("[data-act=save]").click(); await page.waitForTimeout(400);

// 4) Vorschau im Bild: ändert sich mit dem Text
const cw = () => page.locator("[data-cs=vorschau]").evaluate((c) => c.width);
const w0 = await cw(); await page.locator("[data-cs=text]").fill("ABCDEFGHIJKLMNOPQRSTUVWXYZ ABCDEFG"); await page.waitForTimeout(300);
const w1 = await cw(); ok(w1 > w0, "Ein längerer Text macht die Vorschau breiter (" + w0 + " → " + w1 + " px)");
await snap("zeichensatz-2-bearbeitet");

// 4b) Bereiche: markieren und springen
ok((await page.locator("[data-bereich]").count()) === 5, "Bereiche: Alle, Buchstaben, Ziffern, Satzzeichen, Frei");
await page.locator("[data-bereich=buchstaben]").click(); await page.waitForTimeout(300);
ok((await sv()).fi === 1 && (await page.locator("[data-bereich=buchstaben]").getAttribute("aria-pressed")) === "true", "Bereich Buchstaben: springt zu Zeichen 1 (A) und ist gewählt");
await page.locator("[data-bereich=ziffern]").click(); await page.waitForTimeout(300); ok((await sv()).fi === 48, "Bereich Ziffern: springt zu Zeichen 48 (0)");
await page.locator("[data-bereich=custom]").click(); await page.waitForTimeout(300);
await page.locator("[data-cs=from]").fill("10"); await page.locator("[data-cs=to]").fill("20"); await page.waitForTimeout(200);
ok((await page.locator("[data-cs=from]").count()) === 1 && (await page.locator("[data-bereich=custom]").getAttribute("aria-pressed")) === "true", "Frei: Felder von/bis erscheinen");
await page.locator("[data-bereich=custom]").click(); await page.waitForTimeout(200); ok((await page.locator("[data-bereich=custom]").getAttribute("aria-pressed")) === "false", "Ein zweiter Klick hebt den Bereich auf");

// 4c) Vorlage laden (eigener Satz), Abweichung anzeigen, Zeichen zurücksetzen, Rückgängig
const opts = await page.locator("[data-cs=vorlage-wahl] option").evaluateAll((o) => o.map((x) => ({ v: x.value, d: x.disabled })));
const eig = opts.find((o) => o.v.startsWith("std:") && !o.d);
ok(!!eig, "Die Vorlagenliste bietet einen eigenen Satz (Standard eigen): " + (eig ? eig.v : "–"));
await page.locator("[data-cs=vorlage-wahl]").selectOption(eig.v); await page.locator("[data-act=vorlage]").click(); await page.waitForFunction((r) => window.__vc.sp.view.charset.vorlage === r, eig.v, { timeout: 8000 }).catch(() => {}); v = await sv();
ok(v.charset.vorlage === eig.v && v.charset.geaendert.every((g) => !g) && v.dirty, "Vorlage geladen: Satz entspricht ihr, Datei als geändert gekennzeichnet");
ok((await page.locator("[data-cs=herkunft]").innerText()).length > 10, "Die Herkunftskarte nennt den Satz");
{ await page.evaluate(() => { document.querySelector("[data-cs=grid]").closest(".col").scrollTop = 0; }); await page.waitForTimeout(100); const gb2 = await page.locator("[data-cs=grid]").boundingBox(); await page.mouse.click(gb2.x + 2.5 * (gb2.width / 16), gb2.y + 0.5 * (gb2.width / 16)); await page.waitForTimeout(300); }
ok(/Wie die Vorlage/.test(await page.locator("[data-cs=abweichung-text]").innerText()), "Unveränderte Zeichen: „Wie die Vorlage“");
await page.locator("[data-act=clear]").click(); await page.waitForTimeout(400); v = await sv();
ok(v.charset.geaendert[2] === true && /Gegenüber der Vorlage geändert/.test(await page.locator("[data-cs=abweichung-text]").innerText()), "Zeichen geleert: es weicht ab (Hinweis und Markierung in der Übersicht)");
await page.locator("[data-act=original]").click(); await page.waitForTimeout(300); v = await sv();
ok(v.charset.geaendert[2] === false && v.fi === 2, "Zeichen auf Vorlage zurücksetzen: Zeichen wieder wie die Vorlage");
await page.locator("[data-act=undo]").click(); await page.waitForTimeout(300); ok((await sv()).charset.geaendert[2] === true, "Rückgängig nimmt das Zurücksetzen zurück");
for (let i = 0; i < 6 && (await sv()).canUndo; i++) { await page.locator("[data-act=undo]").click(); await page.waitForTimeout(200); }
ok(!(await sv()).dirty, "Alles rückgängig: Datei wieder unverändert");

// 4d) Neuer Zeichensatz
await page.locator("[data-act=new]").first().click(); await page.waitForTimeout(400);
const pf = await page.locator("[data-new=profil] option").count(); ok(pf >= 1, "Neuer Zeichensatz: Profile zur Wahl (" + pf + ")");
await page.locator("[data-new=name]").fill("probe"); await page.locator("[data-new=anlegen]").click(); await page.waitForTimeout(900); v = await sv();
ok(v && v.kind === "charset" && v.rel === "graphics/probe.vcchar", "Der neue Zeichensatz öffnet im Zeichensatz-Editor: " + (v && v.rel));
await snap("zeichensatz-3-neu");

// 4e) Export (E4.3): derselbe Dialog wie bei Sprites, für 256 Zeichen = 2048 Byte
await page.locator("[data-act=export]").click(); await page.waitForSelector("dialog[open] [data-ex=vorschau]"); await page.getByRole("radio", { name: "Assembler" }).click(); await page.waitForTimeout(700);
{ const vt = await page.locator("[data-ex=vorschau]").innerText(); ok(/probe/.test(vt) && /\.byte|\.db|\$[0-9a-f]{2}/i.test(vt), "Export-Dialog: Assembler-Vorschau des Zeichensatzes (" + vt.split("\n").length + " Zeilen)"); }
await page.locator("[data-ex=save]").click(); await page.waitForTimeout(800);
{ const gesp = await page.evaluate(async () => { try { return (await window.__vc.call("file_read", { rel: "build/export/probe.asm" })).text; } catch { return ""; } }); ok(gesp.length > 500, "In Projekt speichern schreibt build/export/probe.asm (" + gesp.length + " Zeichen)"); }
await page.getByRole("button", { name: "Abbrechen" }).first().click(); await page.waitForTimeout(300);

// 4f) Minimalbeispiel für die VoiceOver-Prüfung (?vo-test): 256 Zellen, Roving Focus
for (const v of ["a", "b", "c"]) {
  await page.goto("http://127.0.0.1:1420/?vo-test=" + v); await page.waitForSelector("[aria-label='Zeichen 0, $00']");
  await page.locator("[aria-label='Zeichen 0, $00']").focus(); await page.keyboard.press("ArrowRight"); await page.keyboard.press("ArrowDown"); await page.waitForTimeout(150);
  ok((await page.evaluate(() => document.activeElement?.getAttribute("aria-label"))) === "Zeichen 17, $11", "Minimalbeispiel " + v.toUpperCase() + ": 256 Zellen, Pfeile bewegen den Fokus (Zeichen 17)");
}
await page.goto("http://127.0.0.1:1420/?bridge=17420"); await page.waitForFunction(() => window.__vc && window.__vc.ui.ready);

// 5) Zurück zu Sprites: der Sprite-Editor zeigt keinen Zeichensatz
await page.evaluate(() => window.__vc.runCommand("nav:grafik:sprites")); await page.waitForTimeout(400);
ok(/Kein Sprite geöffnet/.test(await page.locator("body").innerText()), "Im Sprite-Editor zeigt ein geöffneter Zeichensatz nicht an (Hinweis „Kein Sprite geöffnet“)");
console.log(`\n${n} Prüfungen, ${fails.length} Fehler`);
await browser.close();
process.exit(fails.length ? 1 : 0);
