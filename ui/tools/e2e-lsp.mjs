// SPDX-License-Identifier: GPL-3.0-or-later
// Ablauftest der Sprachdienste im Editor (E1.2): Playwright (WebKit) gegen die laufende Prüfbrücke des echten Kerns.
// Aufruf: node tools/e2e-lsp.mjs <Projektkopie (wird verändert!)> [Bilderordner]   (Vite auf :1420 mit VC_CSP=release und `vintagecoder` mit VC_TESTBRIDGE=17420 VC_TEST_NO_SPAWN=1 VC_NO_SAVE=1 müssen laufen)
import { webkit } from "playwright";
import { mkdirSync, writeFileSync, readFileSync } from "node:fs";
import { join } from "node:path";
const DOC_END = process.platform === "darwin" ? "Meta+ArrowDown" : "Control+End", DOC_START = process.platform === "darwin" ? "Meta+ArrowUp" : "Control+Home";   // Textanfang und -ende
const MOD = process.platform === "darwin" ? "Meta" : "Control";   // Kürzel: Befehlstaste (macOS) bzw. Strg

const [proj, shots] = process.argv.slice(2);
if (!proj) { console.error("Aufruf: node tools/e2e-lsp.mjs <Projektkopie> [Bilder]"); process.exit(2); }
if (shots) mkdirSync(shots, { recursive: true });
const browser = await webkit.launch();
const page = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
const fails = []; let n = 0; const csp = [];
const ok = (c, m) => { n++; if (!c) { fails.push(m); console.log("FEHLER:", m); } else console.log("ok:", m); };
page.on("pageerror", (e) => { fails.push("Seitenfehler " + e.message); console.log("Seitenfehler", e.message, String(e.stack).slice(0, 600)); });
page.on("console", (m) => { if (/Content Security Policy|Refused to/i.test(m.text())) csp.push(m.text()); });
// Kontrast (WCAG)
const lum = (c) => { const [r, g, b] = c.match(/[\d.]+/g).slice(0, 3).map(Number).map((v) => { v /= 255; return v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4; }); return 0.2126 * r + 0.7152 * g + 0.0722 * b; };
const rat = (a, b) => { const [x, y] = [lum(a), lum(b)].sort((p, q) => q - p); return (x + 0.05) / (y + 0.05); };
const snap = async (name) => { if (shots) await page.screenshot({ path: join(shots, name + ".png") }); };
const SRC = [".target c64", ".org $0801", "start:   lda #$00", "         sta $d020", "         ldx #8", "loop:    dex", "         bne loop", "         jmp start", ""].join("\n");
writeFileSync(join(proj, "probe.asm"), SRC);
writeFileSync(join(proj, "cpu.asm"), "; Kopf\n; Block\n; weiter\n; noch\nbra start\ninz\nldq $1000\nlda #$00\nnop\n");
writeFileSync(join(proj, "probe.bas"), "10 poke 53280,0\n20 print \"{clr}hallo\"\n30 fori=1to10:next\n");

await page.goto("http://127.0.0.1:1420/?bridge=17420");
await page.waitForFunction(() => window.__vc && window.__vc.ui.ready);
await page.evaluate(async ([p]) => { await window.__vc.setLanguage("de"); await window.__vc.openProject(p); await window.__vc.openFile("probe.asm"); }, [proj]);
await page.waitForSelector(".cm-content");
await page.waitForTimeout(600);
const text = () => page.locator(".cm-content").innerText();
ok((await page.locator(".cm-lintRange-error").count()) === 0, "Fehlerfreier Text: keine Wellenlinie");
ok(/Keine Probleme/.test(await page.locator("footer.status").innerText()), "Statuszeile: Keine Probleme");

// Hover auf einen Befehl: Befehlshilfe
const mn = page.locator(".tok-mnemonic").first(); const box = await mn.boundingBox();
await page.mouse.move(box.x + 4, box.y + 4); await page.waitForSelector(".cm-tooltip .vc-card", { timeout: 4000 }).catch(() => {});
const card = await page.locator(".cm-tooltip .vc-card").first().innerText().catch(() => "");
ok(/LDA/i.test(card) && !/⟦/.test(card), "Hover auf Befehl zeigt die Befehlshilfe: " + card.slice(0, 40).replace(/\n/g, " "));
ok((await page.locator(".cm-tooltip .vc-card table").count()) > 0, "Befehlshilfe enthält die Tabelle der Adressierungsarten");
// Hover-Karte zweispaltig (Variante C): links Titel, Kurzbeschreibung und Flags, rechts Tabelle, unten die Zeile des Cursors
{ const k = page.locator(".cm-tooltip .vc-hv").first();
  ok((await k.locator(".vc-hv-t").innerText()).toUpperCase() === "LDA" && (await k.locator(".vc-hv-k").count()) === 1 && (await k.locator(".vc-hv-l .vc-pill").count()) >= 1, "Hover-Karte: Titel, Kurzbeschreibung und Flags stehen links");
  ok((await k.locator(".vc-hv-r table").count()) === 1 && (await k.locator(".vc-hv-ft").count()) === 1 && /Diese Zeile|This line/.test(await k.locator(".vc-hv-ft").innerText()), "Hover-Karte: Tabelle rechts, Fußzeile „Diese Zeile“ unten");
  ok(/^(Hilfe zu|Help for) /.test(await k.getAttribute("aria-label") ?? "") && (await k.getAttribute("role")) === "group", "Hover-Karte hat eine sprechende Beschriftung");
  const bb = await page.locator(".cm-tooltip:has(.vc-hv)").first().boundingBox(); ok(bb && bb.width <= 560 && bb.height <= 540 && bb.x >= 0 && bb.y >= 0, "Hover-Karte: nicht zu breit, Maximalhöhe, im Fenster (" + Math.round(bb?.width ?? 0) + "×" + Math.round(bb?.height ?? 0) + ")");
  await page.locator(".cm-content").focus(); await page.keyboard.press("Escape"); await page.waitForTimeout(300);
  ok((await page.locator(".cm-tooltip:has(.vc-hv)").count()) === 0, "Esc schließt die Hover-Karte"); }
await page.mouse.move(box.x + 4, box.y + 4); await page.waitForSelector(".cm-tooltip .vc-card", { timeout: 4000 }).catch(() => {}); await snap("lsp-1-hover-befehl");
await page.mouse.move(5, 5); await page.waitForTimeout(300);

// Fehler erzeugen: unbekanntes Symbol
await page.locator(".cm-content").click(); await page.keyboard.press(DOC_END);
await page.keyboard.type("         lda nirgendwo");
await page.waitForSelector(".cm-lintRange-error", { timeout: 5000 }).then(() => ok(true, "Fehler erscheint als Wellenlinie"), () => ok(false, "Keine Wellenlinie für unbekanntes Symbol"));
ok(/1 Fehler/.test(await page.locator("footer.status").innerText()), "Statuszeile zählt den Fehler: " + (await page.locator("footer.status").innerText()).replace(/\s+/g, " ").slice(0, 80));
const eb = await page.locator(".cm-lintRange-error").first().boundingBox();
await page.mouse.move(eb.x + 3, eb.y + 4); await page.waitForSelector(".cm-tooltip .vc-diag", { timeout: 4000 }).catch(() => {});
const dc = await page.locator(".cm-tooltip .vc-diag").first().innerText().catch(() => "");
ok(/nirgendwo/.test(dc) && !/⟦/.test(dc), "Fehlerkarte nennt das Symbol, Text aus dem Katalog: " + dc.replace(/\n/g, " | "));
await snap("lsp-2-fehlerkarte");
await page.mouse.move(5, 5); await page.waitForTimeout(300);
// Probleme-Liste
await page.locator("footer.status button[aria-pressed]").first().click();
await page.waitForSelector(".problems .row");
ok((await page.locator(".problems .row").count()) === 1 && /nirgendwo/.test(await page.locator(".problems .row").innerText()), "Liste Probleme zeigt den Fehler");
await snap("lsp-3-probleme");
await page.locator(".problems .row").click(); await page.waitForTimeout(200);
ok(/Zeile 9/.test(await page.locator("footer.status").innerText()), "Klick in die Liste springt in die Zeile: " + (await page.locator("footer.status").innerText()).replace(/\s+/g, " ").slice(-60));
await page.locator(".dock > header [aria-label='Ablage schließen']").click(); await page.locator(".cm-content").click(); await page.keyboard.press("End");
// Fehler beheben: Wellenlinie verschwindet
for (let i = 0; i < 9; i++) await page.keyboard.press("Backspace");
await page.keyboard.type("start");
await page.waitForFunction(() => document.querySelectorAll(".cm-lintRange-error").length === 0, null, { timeout: 5000 }).then(() => ok(true, "Fehler behoben: Wellenlinie verschwindet"), () => ok(false, "Wellenlinie bleibt stehen"));

// Vervollständigung
await page.keyboard.press("Enter"); await page.keyboard.type("   ld");
await page.waitForSelector(".cm-tooltip-autocomplete li", { timeout: 4000 }).catch(() => {});
const opts = await page.locator(".cm-tooltip-autocomplete li").allInnerTexts().catch(() => []);
ok(opts.some((o) => /^lda/.test(o)) && opts.some((o) => /^ldx/.test(o)), "Vervollständigung bietet Befehle: " + opts.slice(0, 4).join(" / ").replace(/\n/g, " "));
await snap("lsp-4-vervollstaendigung");
await page.keyboard.press("Escape");
for (let i = 0; i < 2; i++) await page.keyboard.press("Backspace");
await page.keyboard.type("jmp lo");
await page.waitForFunction(() => [...document.querySelectorAll(".cm-tooltip-autocomplete li")].some((l) => /^loop/.test(l.textContent)), null, { timeout: 4000 }).catch(() => {});
const o2 = await page.locator(".cm-tooltip-autocomplete li").allInnerTexts().catch(() => []);
ok(o2.some((o) => /^loop/.test(o)), "Vervollständigung bietet Marken des Textes: " + o2.slice(0, 3).join(" / ").replace(/\n/g, " "));
await page.waitForTimeout(200);   // CodeMirror nimmt Enter erst 75 ms nach dem letzten Aufbau der Liste an (Schutz vor Fehleingaben)
await page.keyboard.press("Enter"); await page.waitForTimeout(100);
ok(/jmp loop/.test(await page.locator(".cm-activeLine").innerText()), "Enter übernimmt den Vorschlag: " + JSON.stringify(await page.locator(".cm-activeLine").innerText()));
await page.keyboard.press("Enter"); await page.keyboard.type(".");
await page.waitForSelector(".cm-tooltip-autocomplete li", { timeout: 4000 }).catch(() => {});
const o3 = await page.locator(".cm-tooltip-autocomplete li").allInnerTexts().catch(() => []);
ok(o3.some((o) => /^\.org/.test(o)) , "Nach dem Punkt kommen Direktiven (kanonische Namen): " + o3.slice(0, 3).join(" / ").replace(/\n/g, " "));
await page.keyboard.press("Escape"); await page.keyboard.press("Backspace");
// Gehe zu Definition
await page.keyboard.type("         bne start");
await page.waitForTimeout(500);
await page.evaluate(() => window.__vc.runCommand("gehe_zu_definition"));
await page.waitForTimeout(400);
ok(/Zeile 3,/.test(await page.locator("footer.status").innerText()), "Gehe zu Definition springt zur Marke start (Zeile 3): " + (await page.locator("footer.status").innerText()).replace(/\s+/g, " ").slice(-50));
// ohne Fund: sichtbare Meldung
await page.keyboard.press(DOC_END); await page.keyboard.press("Enter");
await page.evaluate(() => window.__vc.runCommand("gehe_zu_definition"));
await page.waitForTimeout(300);
ok((await page.locator(".toast").count()) > 0 && /Definition/.test(await page.locator(".toast").first().innerText()), "Ohne Fund erscheint eine sichtbare Meldung");

// Gliederung
await page.evaluate(() => window.__vc.runCommand("in_projekt_suchen"));
await page.waitForSelector(".dock [role=tab][aria-selected=true]");
ok((await page.locator(".dock [role=tab][aria-selected=true]").innerText()) === "Suche", "Befehl „In Projekt suchen“ öffnet die Ablage mit der Suche");
await page.locator(".dock [data-tab=outline]").click();
await page.waitForSelector(".dock .o .row", { timeout: 4000 }).catch(() => {});
const ol = await page.locator(".dock .o .row").allInnerTexts();
ok(ol.some((x) => /start/.test(x)) && ol.some((x) => /loop/.test(x)), "Gliederung zeigt die Marken: " + ol.slice(0, 2).join(" | ").replace(/\n/g, " "));
await page.locator(".dock .o .row", { hasText: "loop" }).first().click(); await page.waitForTimeout(300);
ok(/Zeile 6,/.test(await page.locator("footer.status").innerText()), "Klick in die Gliederung springt zur Marke loop (Zeile 6)");
// Suche im Projekt (Hintergrund, mit Treffern aus mehreren Dateien, ungespeicherter Text zählt mit)
await page.locator(".dock [data-tab=search]").click();
await page.locator(".dock input[type=search]").fill("VIC");
await page.waitForFunction(() => /Treffer/.test(document.querySelector(".dock .info")?.textContent ?? "") && document.querySelectorAll(".dock .s .row").length > 3, null, { timeout: 8000 }).catch(() => {});
const sr = await page.locator(".dock .s .file").allInnerTexts();
ok(sr.length >= 1 && (await page.locator(".dock .s .row").count()) > 3, "Suche liefert Treffer, nach Datei gruppiert: " + sr.slice(0, 3).join(", ") + " / " + (await page.locator(".dock .info").innerText()));
await snap("lsp-7-suche");
await page.locator(".dock .s .row").first().click(); await page.waitForTimeout(400);
ok(/Zeile \d+,/.test(await page.locator("footer.status").innerText()), "Klick auf Treffer öffnet die Datei an der Zeile");
await page.locator(".dock input[type=search]").fill("lda start");
await page.waitForSelector('.dock .s .file:has-text("probe.asm")', { timeout: 8000 }).catch(() => {});
ok((await page.locator(".dock .s .file", { hasText: "probe.asm" }).count()) === 1, "Ungespeicherter Text des Reiters wird mit durchsucht (probe.asm)");
await page.locator(".dock input[type=search]").fill("zzzzzzznichtda");
await page.waitForFunction(() => /Keine Treffer/.test(document.querySelector(".dock .info")?.textContent ?? ""), null, { timeout: 8000 }).catch(() => {});
ok(/Keine Treffer/.test(await page.locator(".dock .info").innerText()), "Ohne Treffer: „Keine Treffer“");
await page.locator(".dock > header [aria-label='Ablage schließen']").click();
await page.evaluate(() => window.__vc.openFile("probe.asm")); await page.waitForTimeout(300);
await page.locator(".cm-content").click();
// Prettyprint beim Verlassen der Zeile, Rückgängig, Befehl für die Datei
await page.keyboard.press(DOC_END); await page.keyboard.press("Enter"); await page.waitForTimeout(500);   // die Zeile darüber wird beim Verlassen formatiert; erst danach tippen
await page.keyboard.type("lda   #1      ; ein Wert");
await page.keyboard.press("ArrowUp"); await page.waitForTimeout(600);
const pl = await text();
ok(/\n {8}lda #1 {20,}; ein Wert/.test(pl), "Zeile wird beim Verlassen formatiert (Spalten für Befehl und Kommentar)");
await page.keyboard.press(MOD + "+z"); await page.waitForTimeout(200);
ok(/\n\s*lda {3}#1 {6}; ein Wert/.test(await text()), "Ein Rückgängig nimmt das Formatieren zurück");
await page.evaluate(() => window.__vc.runCommand("datei_formatieren")); await page.waitForTimeout(600);
ok(/\n {8}lda #1 {20,}; ein Wert/.test(await text()), "Befehl „Datei formatieren“ formatiert");
await page.evaluate(() => window.__vc.runCommand("datei_formatieren")); await page.waitForTimeout(300);
ok(/Schon formatiert/.test(await page.locator(".toast").last().innerText()), "Zweites Mal: Meldung „Schon formatiert“ (idempotent)");
await snap("lsp-5-formatiert");
// auch beim Verlassen mit Enter und auch bei Zeilen mit Fehler (hier: unbekanntes Symbol)
await page.keyboard.press(DOC_END); await page.keyboard.press("Enter"); await page.waitForTimeout(500);
await page.keyboard.type("lda   nirgendwohin    ; fehler"); await page.keyboard.press("Enter"); await page.waitForTimeout(900);
ok(/\n {8}lda nirgendwohin {10,}; fehler\n/.test(await text()), "Zeile mit Fehler wird beim Verlassen mit Enter formatiert");
await page.keyboard.type("lda   #2"); await page.keyboard.press("Enter"); await page.waitForTimeout(900);
ok(/\n {8}lda #2\n/.test(await text()), "Fehlerfreie Zeile wird beim Verlassen mit Enter formatiert");
await page.keyboard.press(MOD + "+z"); await page.keyboard.press(MOD + "+z"); await page.waitForTimeout(150);
// Quick-Fix für alten Direktivennamen
await page.locator(".cm-content").click(); await page.keyboard.press(DOC_END); await page.keyboard.press("Enter");
await page.keyboard.type(".pseudopc $c000"); await page.waitForTimeout(600);
await page.keyboard.press(MOD + "+."); await page.waitForSelector(".vc-fixmenu button", { timeout: 4000 }).catch(() => {});
const fx = await page.locator(".vc-fixmenu button").allInnerTexts();
ok(fx.length > 0 && /pseudopc/.test(fx[0]), "⌘. bietet den kanonischen Namen an: " + fx.join(" | "));
await snap("lsp-6-quickfix");
await page.locator(".vc-fixmenu button").first().click(); await page.waitForTimeout(300);
ok(!/\.pseudopc/.test(await page.locator(".cm-activeLine").innerText()), "Quick-Fix ersetzt den Namen: " + (await page.locator(".cm-activeLine").innerText()));
await page.keyboard.press(MOD + "+z"); await page.waitForTimeout(200);
ok(/\.pseudopc/.test(await page.locator(".cm-activeLine").innerText()), "Quick-Fix ist ein Rückgängig-Schritt");
await page.keyboard.press(MOD + "+/"); await page.waitForTimeout(200);
ok(/^\s*;/.test(await page.locator(".cm-activeLine").innerText()), "⌘/ schaltet den Kommentar um");
await page.keyboard.press(MOD + "+/"); await page.waitForTimeout(200);

// Zeichenangaben und BASIC: Karten und Namen kommen aus dem Kern, nicht vom Sprachdienst
const where = (sub, off, lineHas = "") => page.evaluate(([sub, off, lineHas]) => { const w = document.createTreeWalker(document.querySelector(".cm-content"), NodeFilter.SHOW_TEXT); let n; while ((n = w.nextNode())) { const i = n.textContent.indexOf(sub); if (i >= 0 && (!lineHas || n.parentElement.closest(".cm-line").textContent.includes(lineHas))) { const r = document.createRange(); r.setStart(n, i + off); r.setEnd(n, i + off + 1); const b = r.getBoundingClientRect(); return { x: b.x + b.width / 2, y: b.y + b.height / 2 }; } } return null; }, [sub, off, lineHas]);
await page.evaluate(() => window.__vc.openFile("probe.bas")); await page.waitForTimeout(500);
let pt = await where("53280", 2); await page.mouse.move(pt.x, pt.y);
await page.waitForSelector(".cm-tooltip .vc-card", { timeout: 4000 }).catch(() => {});
const bc = await page.locator(".cm-tooltip .vc-card").first().innerText().catch(() => "");
ok(/D020|d020/.test(bc) && !/⟦/.test(bc), "BASIC: Hover auf POKE-Adresse zeigt das Register: " + bc.slice(0, 60).replace(/\n/g, " "));
await snap("lsp-8-basic-hover"); await page.mouse.move(5, 5); await page.waitForTimeout(400);
pt = await where("{clr}", 2); await page.mouse.move(pt.x, pt.y);
await page.waitForSelector(".cm-tooltip .vc-card", { timeout: 4000 }).catch(() => {});
const cc = await page.locator(".cm-tooltip .vc-card").first().innerText().catch(() => "");
ok(/clr/i.test(cc) && !/⟦/.test(cc), "Hover auf {clr} beschreibt das Zeichen: " + cc.slice(0, 60).replace(/\n/g, " "));
const kwTexts = await page.locator(".tok-keyword").allInnerTexts();
ok(kwTexts.filter((x) => /^for$/i.test(x)).length === 1 && kwTexts.filter((x) => /^to$/i.test(x)).length === 1 && kwTexts.filter((x) => /^next$/i.test(x)).length === 1, "BASIC: fori=1to10 ohne Leerzeichen wird als for, to erkannt und gefärbt: " + kwTexts.join(","));
await page.mouse.move(5, 5);
await page.evaluate(() => window.__vc.openFile("probe.asm")); await page.waitForTimeout(400);
await page.locator(".cm-content").click(); await page.keyboard.press(DOC_END); await page.keyboard.press("Enter");
await page.keyboard.type('.text "ab{cl');
await page.waitForFunction(() => [...document.querySelectorAll(".cm-tooltip-autocomplete li")].some((l) => /^clr/.test(l.textContent)), null, { timeout: 5000 }).then(() => ok(true, "Nach { in einer Zeichenkette kommen die Zeichennamen (clr)"), () => ok(false, "Keine Zeichennamen nach {"));
await snap("lsp-9-zeichennamen"); await page.waitForTimeout(200);
await page.keyboard.press("Tab"); await page.waitForTimeout(150);
const clrLine = await page.locator(".cm-activeLine").innerText();
ok(/\.text "ab\{clr\}"/.test(clrLine) && !/\}\}/.test(clrLine), "Zeichenname übernehmen: schließende Klammer genau einmal: " + clrLine.trim());
await page.keyboard.press("Escape");

// Zeilen verschieben und duplizieren (CodeMirror-Standardtasten)
await page.locator(".cm-line").nth(1).click(); await page.keyboard.press("End");
const l1 = (await page.locator(".cm-line").nth(1).innerText()), l0 = (await page.locator(".cm-line").nth(0).innerText());
await page.keyboard.press("Alt+ArrowUp"); await page.waitForTimeout(150);
ok((await page.locator(".cm-line").nth(0).innerText()) === l1 && (await page.locator(".cm-line").nth(1).innerText()) === l0, "Alt+↑ verschiebt die Zeile nach oben");
await page.keyboard.press("Alt+ArrowDown"); await page.waitForTimeout(150);
const cnt0 = await page.locator(".cm-line").count();
await page.keyboard.press("Shift+Alt+ArrowDown"); await page.waitForTimeout(150);
ok((await page.locator(".cm-line").count()) === cnt0 + 1, "Umschalt+Alt+↓ dupliziert die Zeile");
await page.keyboard.press(MOD + "+z"); await page.waitForTimeout(150);

// Kontextmenü des Editors (Maus)
const bp0 = await page.locator(".mk-breakpoint").count();
await page.locator(".cm-line").nth(2).click({ button: "right" });
await page.waitForSelector(".vc-fixmenu button", { timeout: 3000 }).catch(() => {});
const cmn = await page.locator(".vc-fixmenu button").allInnerTexts();
ok(cmn.length === 12 && cmn.includes("Kopieren") && cmn.includes("Gehe zu Definition"), "Kontextmenü im Editor: " + cmn.join(" | "));
await snap("lsp-10-kontextmenue");
await page.locator(".vc-fixmenu button", { hasText: "Haltepunkt umschalten" }).click(); await page.waitForTimeout(300);
ok((await page.locator(".mk-breakpoint").count()) === bp0 + 1, "Kontextmenü ▸ Haltepunkt setzt einen Haltepunkt");
await page.locator(".cm-line").nth(2).click({ button: "right" });
await page.locator(".vc-fixmenu button", { hasText: "Alles auswählen" }).click(); await page.waitForTimeout(300);
ok(/markiert/.test(await page.locator("footer.status").innerText()), "Kontextmenü ▸ Alles auswählen markiert");
await page.keyboard.press("ArrowRight");
await page.evaluate(() => window.__vc.runCommand("haltepunkt_umschalten"));

// Hover auf Zahl und Marke, alter Direktivenname in der Liste, Vervollständigung von Hand
await page.evaluate(() => window.__vc.openFile("probe.asm")); await page.waitForTimeout(400);
let q = await where("$d020", 2); await page.mouse.move(q.x, q.y);
await page.waitForSelector(".cm-tooltip .vc-card", { timeout: 4000 }).catch(() => {});
const hz = await page.locator(".cm-tooltip .vc-card").first().innerText().catch(() => "");
ok(/D020/i.test(hz) && /VIC|Rahmen|Rand|border/i.test(hz) && !/⟦/.test(hz), "Hover auf $d020: Wert und Register: " + hz.slice(0, 70).replace(/\n/g, " "));
await page.mouse.move(5, 5); await page.waitForTimeout(400);
q = await where("loop", 1, "bne"); await page.mouse.move(q.x, q.y);
await page.waitForSelector(".cm-tooltip .vc-card", { timeout: 4000 }).catch(() => {});
const hm = await page.locator(".cm-tooltip .vc-card").first().innerText().catch(() => "");
ok(/loop|\$/i.test(hm) && /Definiert|Defined/i.test(hm), "Hover auf eine Marke: Wert und Fundstelle: " + hm.slice(0, 70).replace(/\n/g, " "));
await page.mouse.move(5, 5);
await page.locator(".cm-content").click(); await page.keyboard.press(DOC_END); await page.keyboard.press("Enter"); await page.keyboard.type(".pseudop");
await page.waitForFunction(() => [...document.querySelectorAll(".cm-tooltip-autocomplete li")].some((l) => /^\.pseudopc/.test(l.textContent)), null, { timeout: 5000 }).then(() => ok(true, "Alter Direktivenname erscheint erst beim Tippen"), () => ok(false, "Alter Name fehlt in der Liste"));
const al = await page.locator(".cm-tooltip-autocomplete li").first().innerText().catch(() => "");
ok(/relocate|kanon|canon/i.test(al), "… mit Hinweis auf den kanonischen Namen: " + al.replace(/\n/g, " "));
await page.keyboard.press("Escape");
for (let i = 0; i < 8; i++) await page.keyboard.press("Backspace");
await page.keyboard.type("   ");
await page.keyboard.press("Control+Space");
await page.waitForSelector(".cm-tooltip-autocomplete li", { timeout: 4000 }).then(() => ok(true, "Strg+Leertaste öffnet die Liste ohne Tippen"), () => ok(false, "Liste per Hand nicht geöffnet"));
await page.keyboard.press("Escape");

// Klammersprung, Zeilen sortieren, Leerzeichen am Zeilenende, Fehlersprung
// (in einer Textdatei: dort formatiert nichts beim Verlassen der Zeile)
await page.evaluate(async () => { await window.__vc.call("file_create", { rel: "trim.txt" }); await window.__vc.call("file_write", { rel: "trim.txt", text: "zz  \naa  ", eol: "lf" }); await window.__vc.openFile("trim.txt"); });
await page.waitForTimeout(500); await page.locator(".cm-content").click(); await page.keyboard.press(MOD + "+a");
await page.evaluate(() => window.__vc.runCommand("zeilen_sortieren")); await page.waitForTimeout(200);
const srt = (await page.locator(".cm-line").allInnerTexts()).slice(0, 2).map((x) => x.trim());
ok(srt[0].startsWith("aa") && srt[1].startsWith("zz"), "Zeilen sortieren: " + srt.join(" | "));
await page.evaluate(() => window.__vc.runCommand("leerzeichen_entfernen")); await page.waitForTimeout(200);
ok((await page.locator(".cm-line").allInnerTexts()).slice(0, 2).every((x) => x === x.trimEnd()), "Leerzeichen am Zeilenende entfernt");
await page.keyboard.press(MOD + "+z"); await page.waitForTimeout(150);
ok((await page.locator(".cm-line").allInnerTexts()).slice(0, 2).some((x) => x !== x.trimEnd()), "… in einem Rückgängig-Schritt");
await page.evaluate(() => window.__vc.openFile("probe.asm")); await page.waitForTimeout(400); await page.locator(".cm-content").click();
await page.keyboard.press("Escape");
await page.keyboard.press(DOC_END); await page.keyboard.type("(x)"); await page.keyboard.press("ArrowLeft"); await page.keyboard.press("ArrowLeft");
const c0 = await page.locator("footer.status .mono").innerText();
await page.keyboard.press("Control+m"); await page.waitForTimeout(100);
ok((await page.locator("footer.status .mono").innerText()) !== c0, "Ctrl+M springt zur passenden Klammer");
await page.keyboard.press(DOC_END); await page.keyboard.type("\n         lda nirgendwo2"); await page.waitForSelector(".cm-lintRange-error", { timeout: 5000 }).catch(() => {});
await page.keyboard.press(DOC_START); await page.keyboard.press("F8"); await page.waitForTimeout(200);
ok(/Zeile \d+/.test(await page.locator("footer.status").innerText()) && !/Zeile 1,/.test(await page.locator("footer.status").innerText()), "F8 springt zum nächsten Problem");
await page.evaluate(() => window.__vc.runCommand("voriges_problem"));

// Dunkles Schema: Fehlerkarte, Vervollständigungsliste und Korrekturmenü lesbar; Statuszeile zählt nur die Datei im Reiter, die Ablage das Projekt
await page.evaluate((s) => window.__vc.setTheme(s), "dark"); await page.waitForTimeout(300);
await page.evaluate(() => window.__vc.openFile("probe.asm")); await page.waitForTimeout(500);
const er = page.locator(".cm-lintRange-error").first(); const eb3 = await er.boundingBox(); await page.mouse.move(eb3.x + 3, eb3.y + 4); await page.waitForSelector(".cm-tooltip .vc-diag-1", { timeout: 4000 }).catch(() => {});
const ec = await page.evaluate(() => { const c = document.querySelector(".cm-tooltip .vc-diag-1 .vc-diag-msg"); if (!c) return null; return { fg: getComputedStyle(c).color, bg: getComputedStyle(c.closest(".cm-tooltip")).backgroundColor }; });
ok(ec && rat(ec.fg, ec.bg) >= 4.5, "Dunkles Schema: Fehlerkarte erscheint und ist lesbar" + (ec ? ", Kontrast " + rat(ec.fg, ec.bg).toFixed(1) : " (keine Karte)"));
await snap("lsp-12-dunkel-fehlerkarte"); await page.mouse.move(5, 5); await page.waitForTimeout(300);
await page.locator(".cm-content").click(); await page.keyboard.press(DOC_END); await page.keyboard.press("Enter"); await page.keyboard.type("ld");
await page.waitForSelector(".cm-tooltip-autocomplete li", { timeout: 4000 }).catch(() => {});
const lc = await page.evaluate(() => { const li = document.querySelector(".cm-tooltip-autocomplete li:not([aria-selected])") ?? document.querySelector(".cm-tooltip-autocomplete li"); if (!li) return null; return { fg: getComputedStyle(li).color, bg: getComputedStyle(li.closest(".cm-tooltip")).backgroundColor }; });
ok(lc && rat(lc.fg, lc.bg) >= 4.5, "Dunkles Schema: Vervollständigungsliste lesbar" + (lc ? ", Kontrast " + rat(lc.fg, lc.bg).toFixed(1) : " (keine Liste)"));
const sc2 = await page.evaluate(() => { const li = document.querySelector(".cm-tooltip-autocomplete li[aria-selected]"); return li ? { fg: getComputedStyle(li).color, bg: getComputedStyle(li).backgroundColor } : null; });
await snap("lsp-13-dunkel-liste"); await page.keyboard.press("Escape"); await page.keyboard.press("Backspace"); await page.keyboard.press("Backspace"); await page.keyboard.type(".pseudopc $c000"); await page.waitForTimeout(500);
await page.keyboard.press(MOD + "+."); await page.waitForSelector(".vc-fixmenu button", { timeout: 3000 }).catch(() => {});
const fm = await page.evaluate(() => { const b = document.querySelector(".vc-fixmenu button"); return b ? { fg: getComputedStyle(b).color, bg: getComputedStyle(b.closest(".vc-fixmenu")).backgroundColor } : null; });
ok(fm && rat(fm.fg, fm.bg) >= 4.5, "Dunkles Schema: Korrekturmenü lesbar" + (fm ? ", Kontrast " + rat(fm.fg, fm.bg).toFixed(1) : " (kein Menü)")); await page.keyboard.press("Escape");
await page.evaluate((s) => window.__vc.setTheme(s), "light");
await page.evaluate(() => window.__vc.openFile("probe.bas")); await page.waitForTimeout(400);
ok(/Keine Probleme/.test(await page.locator("footer.status").innerText()), "Statuszeile zählt nur die Datei im Reiter (probe.bas ohne Meldungen): " + (await page.locator("footer.status").innerText()).replace(/\s+/g, " ").slice(0, 60));
ok(/Keine Probleme · \d+ Meldungen? in anderen Dateien/.test((await page.locator("footer.status").innerText()).replace(/\s+/g, " ")), "Statuszeile nennt Meldungen anderer Dateien ehrlich dazu: " + (await page.locator("footer.status").innerText()).replace(/\s+/g, " ").slice(0, 90));
await page.locator("footer.status button[aria-pressed]").first().click(); await page.waitForSelector(".problems .sum", { timeout: 3000 }).catch(() => {});
ok(/Projekt: \d+ Meldungen in \d+ Dateien?/.test(await page.locator(".problems .sum").innerText().catch(() => "")), "Ablage zeigt die Kurzzeile für das Projekt: " + await page.locator(".problems .sum").innerText().catch(() => "-"));
ok((await page.locator(".problems .file").count()) >= 2, "Ablage gruppiert nach Dateien (" + await page.locator(".problems .file").count() + " Gruppen)");
await page.locator(".dock > header [aria-label='Ablage schließen']").click();

// Tab übernimmt den Vorschlag der Vervollständigung; Kommentar auch mit ⌘⌥C
await page.evaluate(() => window.__vc.openFile("probe.asm")); await page.waitForTimeout(400);
await page.locator(".cm-content").click(); await page.keyboard.press(DOC_END); await page.keyboard.press("Enter"); await page.waitForTimeout(500); await page.keyboard.type("   ld");
await page.waitForSelector(".cm-tooltip-autocomplete li", { timeout: 4000 }).catch(() => {}); await page.waitForTimeout(300);
await page.keyboard.press("Tab"); await page.waitForTimeout(200);
ok(/\bld[axy]\b/.test(await page.locator(".cm-activeLine").innerText()) && !/\t/.test(await page.locator(".cm-activeLine").innerText()), "Tab übernimmt den Vorschlag: " + JSON.stringify(await page.locator(".cm-activeLine").innerText()));
await page.keyboard.press("Alt+" + MOD + "+c"); await page.waitForTimeout(200);
ok(/^\s*;/.test(await page.locator(".cm-activeLine").innerText()), "⌘⌥C schaltet den Kommentar um");
await page.keyboard.press("Alt+" + MOD + "+c");

// Zeilenumbruch und Zeilenende
await page.evaluate(() => window.__vc.runCommand("umbruch")); await page.waitForTimeout(200);
ok((await page.locator(".cm-lineWrapping").count()) === 0, "Zeilenumbruch schaltet aus (Standard ist an)");
await page.evaluate(() => window.__vc.runCommand("umbruch")); await page.waitForTimeout(200);
ok((await page.locator(".cm-lineWrapping").count()) === 1, "Zeilenumbruch schaltet wieder ein");
await page.evaluate(() => window.__vc.runCommand("zeilenende")); await page.waitForTimeout(200);
ok(/CRLF/.test(await page.locator("footer.status").innerText()), "Zeilenende umgeschaltet: Statuszeile zeigt CRLF");
await page.evaluate(() => window.__vc.runCommand("zeilenende"));

// Faltmarken folgen dem Text (Falt-Beginn bleibt am Blockanfang), Hinweise sind überall gleich sichtbar (Zähler, Karte, Liste), Karten im dunklen Schema lesbar
await page.evaluate(() => window.__vc.openFile("cpu.asm")); await page.waitForTimeout(1200);
const foldTops = () => page.evaluate(() => [...document.querySelectorAll(".cm-foldGutter .cm-gutterElement")].filter((e) => e.querySelector(".vc-fold") && getComputedStyle(e).visibility !== "hidden").map((e) => Math.round(e.getBoundingClientRect().top)));
const f0 = await foldTops(); const firstLine = await page.locator(".cm-line").first().boundingBox();
ok(f0.length > 0 && Math.abs(f0[0] - firstLine.y) < 6, "Faltmarke steht am Anfang des Kommentarblocks (nach dem Öffnen): " + JSON.stringify(f0));
await page.locator(".cm-line").nth(2).click(); await page.keyboard.press("Home"); await page.keyboard.type("x"); await page.waitForTimeout(700);
const f1 = await foldTops(); ok(f1.length === 0 || Math.abs(f1[0] - firstLine.y) > 6, "Nach dem Tippen am Zeilenanfang ist der Block kein Kommentarblock mehr (Marke folgt)");
await page.keyboard.press("Backspace"); await page.waitForTimeout(700);
const f2 = await foldTops(); ok(f2.length > 0 && Math.abs(f2[0] - firstLine.y) < 6, "Nach dem Löschen steht die Faltmarke wieder am Blockanfang: " + JSON.stringify(f2));
ok(/Hinweis/.test(await page.locator("footer.status").innerText()) && !/Keine Probleme/.test(await page.locator("footer.status").innerText()), "Hinweise stehen in der Statuszeile: " + (await page.locator("footer.status").innerText()).replace(/\s+/g, " ").slice(0, 70));
await page.locator("footer.status button[aria-pressed]").first().click(); await page.locator(".dock [data-tab=problems]").click(); await page.waitForSelector(".problems .row", { timeout: 3000 }).catch(() => {});
ok((await page.locator(".problems .row").count()) >= 4, "Probleme-Liste enthält die Hinweise (" + await page.locator(".problems .row").count() + ")");
await page.locator(".dock > header [aria-label='Ablage schließen']").click();
await page.evaluate((s) => window.__vc.setTheme(s), "dark"); await page.waitForTimeout(300);
const hr = await page.locator(".cm-lintRange-info").first().boundingBox(); await page.mouse.move(hr.x + 3, hr.y + 4); await page.waitForSelector(".cm-tooltip .vc-card", { timeout: 4000 }).catch(() => {});
const hc = await page.locator(".cm-tooltip .vc-card").first().innerText().catch(() => "");
ok(/E205|unbekannter Befehl/.test(hc), "Karte zum Hinweis mit Code und Meldung: " + hc.replace(/\n/g, " | ").slice(0, 60));
const kc = await page.evaluate(() => { const c = document.querySelector(".cm-tooltip .vc-card"); const tip = c.closest(".cm-tooltip"); return { fg: getComputedStyle(c.querySelector(".vc-diag-msg") ?? c).color, bg: getComputedStyle(tip).backgroundColor }; });
ok(rat(kc.fg, kc.bg) >= 4.5, "Dunkles Schema: Karte lesbar, Kontrast " + rat(kc.fg, kc.bg).toFixed(1) + " (" + kc.fg + " auf " + kc.bg + ")");
await page.mouse.move(5, 5); await page.waitForTimeout(300);
await page.evaluate(() => window.__vc.openFile("probe.asm")); await page.waitForTimeout(400);
let dq = await where(".org", 1); await page.mouse.move(dq.x, dq.y); await page.waitForSelector(".cm-tooltip .vc-card", { timeout: 4000 }).catch(() => {});
const dc3 = await page.evaluate(() => { const c = document.querySelector(".cm-tooltip .vc-card"); if (!c) return null; const tip = c.closest(".cm-tooltip"); const p = c.querySelector("p, strong") ?? c; return { fg: getComputedStyle(p).color, bg: getComputedStyle(tip).backgroundColor }; });
ok(dc3 && rat(dc3.fg, dc3.bg) >= 4.5, "Dunkles Schema: Karte über einer Direktive lesbar" + (dc3 ? ", Kontrast " + rat(dc3.fg, dc3.bg).toFixed(1) : " (keine Karte)"));
await snap("lsp-11-dunkel-karte"); await page.mouse.move(5, 5);
await page.evaluate((s) => window.__vc.setTheme(s), "light");

// Sprache Englisch: Karte und Zähler wechseln mit
await page.evaluate(() => window.__vc.setLanguage("en")); await page.waitForTimeout(300);
await page.keyboard.press(DOC_END); await page.keyboard.type("\n         lda nirgendwo");
await page.waitForSelector(".cm-lintRange-error", { timeout: 5000 }).catch(() => {});
const eb2 = await page.locator(".cm-lintRange-error").first().boundingBox();
await page.mouse.move(eb2.x + 3, eb2.y + 4); await page.waitForSelector(".cm-tooltip .vc-diag", { timeout: 4000 }).catch(() => {});
const dc2 = await page.locator(".cm-tooltip .vc-diag").first().innerText().catch(() => "");
ok(/unknown (symbol|instruction)|unexpected characters|nirgendwo/i.test(dc2) && /Error/.test(dc2) && !/⟦/.test(dc2), "Englisch: Fehlerkarte englisch: " + dc2.replace(/\n/g, " | "));
await page.evaluate(() => window.__vc.setLanguage("de"));
// Include einer erst beim Bauen entstehenden Datei aus build/export/: Hinweis statt Fehler (die Quelle liegt im Projekt)
mkdirSync(join(proj, "graphics"), { recursive: true }); writeFileSync(join(proj, "graphics", "hgprobe.vcsprite"), "{}");
writeFileSync(join(proj, "exp.asm"), ".target c64\n.org $0801\n.include \"build/export/hgprobe.asm\"\nstart: rts\n");
await page.evaluate(async () => { await window.__vc.openFile("exp.asm"); }); await page.waitForTimeout(1200);
ok((await page.locator(".cm-lintRange-error").count()) === 0 && /Hinweis/.test(await page.locator("footer.status").innerText()), "Include aus build/export ohne Datei: Hinweis „entsteht beim Bauen“ statt Fehler: " + (await page.locator("footer.status").innerText()).slice(0, 60).replace(/\n/g, " "));
writeFileSync(join(proj, "exp2.asm"), ".target c64\n.org $0801\n.include \"build/export/gibtsnicht.asm\"\nstart: rts\n");
await page.evaluate(async () => { await window.__vc.openFile("exp2.asm"); }); await page.waitForTimeout(1200);
ok((await page.locator(".cm-lintRange-error").count()) >= 1, "Include einer Datei ohne Quelle bleibt ein Fehler");
ok(csp.length === 0, "Keine Meldungen der Inhaltsrichtlinie: " + csp.join(" | ").slice(0, 200));
console.log(`\n${n} Prüfungen, ${fails.length} Fehler`);
await browser.close();
process.exit(fails.length ? 1 : 0);
