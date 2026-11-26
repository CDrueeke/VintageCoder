// SPDX-License-Identifier: GPL-3.0-or-later
// Ablauftest Geteilte Ansicht (E1.4): zwei Reitergruppen, Fokus, Bearbeiten in beiden Hälften, Schieben, Ziehen an den Rand, Teilung aufheben.
// Aufruf: node tools/e2e-split.mjs <Projektkopie (wird verändert!)> [Bilderordner]   (Vite auf :1420 und `vintagecoder` mit VC_TESTBRIDGE=17420 VC_TEST_NO_SPAWN=1 VC_NO_SAVE=1 laufen)
import { webkit } from "playwright";
import { mkdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
const DOC_END = process.platform === "darwin" ? "Meta+ArrowDown" : "Control+End", DOC_START = process.platform === "darwin" ? "Meta+ArrowUp" : "Control+Home";   // Textanfang und -ende
const MOD = process.platform === "darwin" ? "Meta" : "Control";   // Kürzel: Befehlstaste (macOS) bzw. Strg

const [proj, shots] = process.argv.slice(2);
if (!proj) { console.error("Aufruf: node tools/e2e-split.mjs <Projektkopie> [Bilder]"); process.exit(2); }
if (shots) mkdirSync(shots, { recursive: true });
const browser = await webkit.launch();
const page = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
const fails = []; let n = 0; const csp = [];
const ok = (c, m) => { n++; if (!c) { fails.push(m); console.log("FEHLER:", m); } else console.log("ok:", m); };
page.on("pageerror", (e) => { fails.push("Seitenfehler " + e.message); console.log("Seitenfehler", e.message); });
page.on("console", (m) => { if (/Content Security Policy|Refused to/i.test(m.text())) csp.push(m.text()); });
const snap = async (name) => { if (shots) await page.screenshot({ path: join(shots, name + ".png") }); };
await page.goto("http://127.0.0.1:1420/?bridge=17420"); await page.waitForFunction(() => window.__vc && window.__vc.ui.ready);
await page.evaluate(async ([p]) => { await window.__vc.setLanguage("de"); await window.__vc.openProject(p); }, [proj]); await page.waitForTimeout(700);
const dateien = await page.evaluate(() => { const f = (ns) => ns.flatMap((x) => (x.kind === "dir" ? f(x.children ?? []) : [x])); return f(window.__vc.ui.project.tree).map((x) => x.rel).filter((r) => /\.(asm|inc)$/.test(r)); });
const [a, b, c] = dateien; const nm = (r) => r.split("/").pop();
for (const r of [a, b, c]) await page.evaluate(async (r) => { await window.__vc.openFile(r); }, r);
await page.waitForTimeout(500);
const cmd = async (id) => { await page.evaluate((i) => { void window.__vc.runCommand(i); }, id); await page.waitForTimeout(700); };
const grp = (g) => page.locator(".grp").nth(g);
const tabsOf = async (g) => grp(g).locator(".tab .nm").allInnerTexts();
const active = () => page.evaluate(() => ({ a: window.__vc.ed.active, by: window.__vc.ed.activeBy, f: window.__vc.ed.focus }));
ok((await page.locator(".grp").count()) === 1, "Anfangs eine Hälfte mit drei Reitern: " + (await tabsOf(0)).join(", "));

// Teilen
await cmd("teilen");
ok((await page.locator(".grp").count()) === 2, "Teilen (⌘⌥T) zeigt zwei Hälften");
ok(JSON.stringify(await tabsOf(0)) === JSON.stringify([nm(a), nm(b)]) && JSON.stringify(await tabsOf(1)) === JSON.stringify([nm(c)]), "Der aktive Reiter ging nach rechts: links " + (await tabsOf(0)).join(", ") + " | rechts " + (await tabsOf(1)).join(", "));
ok((await page.locator(".cm-editor").count()) === 2, "Beide Hälften haben einen Editor");
{ const x = await active(); ok(x.f === 1 && x.a === c, "Fokus liegt rechts, aktiv ist " + nm(c) + " (ist: Fokus " + x.f + ", aktiv " + x.a + ")"); }
await snap("split-1-geteilt");
// Bearbeiten rechts verändert nur die rechte Datei
const rechts = grp(1).locator(".cm-content"), links = grp(0).locator(".cm-content");
const textL0 = await links.innerText(); const textR0 = await rechts.innerText();
ok(textL0 !== textR0 && textR0.length > 0, "Die Hälften zeigen verschiedene Texte");
await rechts.click(); await page.keyboard.press(DOC_START); await page.keyboard.type("; rechts\n"); await page.waitForTimeout(300);
ok((await rechts.innerText()).startsWith("; rechts") && (await links.innerText()) === textL0, "Tippen rechts ändert nur die rechte Hälfte");
ok(await grp(1).locator(".tab.on .dot").count() === 1 && await grp(0).locator(".tab .dot").count() === 0, "Ungespeichert-Punkt nur am rechten Reiter");
await page.keyboard.press(MOD + "+s"); await page.waitForTimeout(500);
ok(readFileSync(join(proj, c), "utf8").startsWith("; rechts"), "⌘S in der rechten Hälfte speichert die rechte Datei");
// Fokuswechsel per Klick links
await links.click(); await page.waitForTimeout(200);
const af = await active(); ok(af.f === 0 && af.a === b, "Klick in die linke Hälfte setzt den Fokus dorthin (aktiv " + nm(af.a ?? "") + ")");
ok(/Zeile \d+, Spalte \d+/.test(await page.locator("footer.status").innerText()), "Statuszeile zeigt die Lage der fokussierten Hälfte");
await page.keyboard.press(DOC_START); await page.keyboard.type("; links\n"); await page.waitForTimeout(300);
ok((await links.innerText()).startsWith("; links") && (await rechts.innerText()).startsWith("; rechts"), "Tippen links ändert nur die linke Hälfte");
// Reiterwechsel gilt je Hälfte
await page.evaluate(() => { void window.__vc.runCommand("voriger_reiter"); }); await page.waitForTimeout(500);
const w = await active(); ok(w.f === 0 && w.a === a && w.by[1] === c, "Voriger Reiter wechselt nur in der fokussierten Hälfte");
// Reiter der anderen Hälfte anklicken: Fokus wechselt
await grp(1).locator(".tab").first().click(); await page.waitForTimeout(400);
ok((await active()).f === 1, "Klick auf einen Reiter der rechten Hälfte fokussiert sie");
// Schieben
await cmd("reiter_andere_haelfte");
ok(JSON.stringify(await tabsOf(0)).includes(nm(c)) && (await page.locator(".grp").count()) === 2 === false || (await page.locator(".grp").count()) === 1, "Der einzige Reiter rechts nach links geschoben: Teilung endet");
ok((await page.locator(".grp").count()) === 1, "… leere rechte Hälfte verschwindet");
// Teilen per Befehl „andere Hälfte“, dann Schließen des letzten rechten Reiters
await page.locator(".tab", { hasText: nm(b) }).first().click(); await cmd("reiter_andere_haelfte");
ok((await page.locator(".grp").count()) === 2 && (await tabsOf(1)).join() === nm(b), "„In die andere Hälfte“ teilt, wenn noch nicht geteilt");
await cmd("alle_speichern");
await page.evaluate(() => { void window.__vc.runCommand("datei_schliessen"); }); await page.waitForTimeout(500);
ok((await page.locator(".grp").count()) === 1, "Schließen des letzten Reiters rechts hebt die Teilung auf");
// Ziehen an den rechten Rand
const quelle = page.locator(".tab", { hasText: nm(a) }).first();
const q = await quelle.boundingBox();
await page.mouse.move(q.x + q.width / 2, q.y + q.height / 2); await page.mouse.down(); await page.mouse.move(q.x + q.width / 2 + 30, q.y + q.height / 2 + 30, { steps: 4 }); await page.waitForTimeout(200);
ok(await page.locator(".z.edge").count() === 1, "Beim Ziehen erscheint die Zone „Rechts ablegen“");
await snap("split-2-ziehen");
{ const zb = await page.locator(".z.edge").boundingBox(); await page.mouse.move(zb.x + zb.width / 2, zb.y + zb.height / 2, { steps: 4 }); await page.waitForTimeout(150); await page.mouse.up(); await page.waitForTimeout(600); }
ok((await page.locator(".grp").count()) === 2 && (await tabsOf(1)).includes(nm(a)), "Ablegen am rechten Rand teilt und nimmt den Reiter mit");
await page.evaluate(() => { window.__vc.ed.dragRel = null; });
// Teilung aufheben
await cmd("teilen");
ok((await page.locator(".grp").count()) === 1 && (await tabsOf(0)).length === 2, "Teilen erneut vereint beide Hälften (" + (await tabsOf(0)).join(", ") + ")");
// Dokument maximieren und Zen mit Teilung
await cmd("teilen"); await cmd("dokument_max");
ok((await page.locator(".grp").count()) === 2 && (await page.locator(".cm-editor").count()) === 2, "Maximiertes Dokumentenfeld bleibt geteilt");
await cmd("dokument_max"); await cmd("teilen");
// Hover und Färbung in der rechten Hälfte
await page.evaluate(() => { void window.__vc.runCommand("teilen"); }); await page.waitForTimeout(700);
ok((await page.locator(".grp").nth(1).locator(".tok-mnemonic, .tok-comment").count()) > 0, "Die rechte Hälfte ist gefärbt");
ok(csp.length === 0, "Keine Meldungen der Inhaltsrichtlinie: " + csp.slice(0, 2).join(" | "));
console.log(`\n${n} Prüfungen, ${fails.length} Fehler`);
await browser.close();
process.exit(fails.length ? 1 : 0);
