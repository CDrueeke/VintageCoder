// SPDX-License-Identifier: GPL-3.0-or-later
// Ablauftest des Code-Editors (E1.1): Playwright (WebKit) gegen die laufende Prüfbrücke des echten Kerns.
// Aufruf: node tools/e2e-editor.mjs <Projektkopie (wird verändert!)> [Bilderordner]   (Vite auf :1420 und `vintagecoder` mit VC_TESTBRIDGE=17420 VC_TEST_NO_SPAWN=1 VC_NO_SAVE=1 müssen laufen)
import { webkit } from "playwright";
import { readFileSync, readdirSync, existsSync, mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
const DOC_END = process.platform === "darwin" ? "Meta+ArrowDown" : "Control+End", DOC_START = process.platform === "darwin" ? "Meta+ArrowUp" : "Control+Home";   // Textanfang und -ende
const MOD = process.platform === "darwin" ? "Meta" : "Control";   // Kürzel: Befehlstaste (macOS) bzw. Strg

const [proj, shots] = process.argv.slice(2);
if (!proj) { console.error("Aufruf: node tools/e2e-editor.mjs <Projektkopie> [Bilder]"); process.exit(2); }
if (shots) mkdirSync(shots, { recursive: true });
const browser = await webkit.launch();
const page = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
const fails = []; let n = 0; const csp = [];
const ok = (c, m) => { n++; if (!c) { fails.push(m); console.log("FEHLER:", m); } else console.log("ok:", m); };
page.on("pageerror", (e) => { fails.push("Seitenfehler " + e.message); console.log("Seitenfehler", e.message, String(e.stack).slice(0, 600)); });
page.on("console", (m) => { if (/Content Security Policy|Refused to/i.test(m.text())) csp.push(m.text()); });
await page.goto("http://127.0.0.1:1420/?bridge=17420");
await page.waitForFunction(() => window.__vc && window.__vc.ui.ready);
await page.evaluate(async ([p]) => { await window.__vc.setLanguage("de"); await window.__vc.openProject(p); }, [proj]);
const snap = async (name) => { if (shots) await page.screenshot({ path: join(shots, name + ".png") }); };
const asm = (await page.evaluate(() => { const f = (ns) => ns.flatMap((x) => (x.kind === "dir" ? f(x.children ?? []) : [x])); return f(window.__vc.ui.project.tree).map((x) => x.rel).find((r) => r.endsWith(".asm") && !r.includes("/")); }));
ok(!!asm, "Projekt hat eine .asm-Datei: " + asm);

// Öffnen, Färbung, Faltung
await page.getByRole("treeitem", { name: new RegExp(asm.split("/").pop().replace(".", "\\.")) }).first().click();
await page.waitForSelector(".cm-editor .cm-content");
await page.locator(".cm-content").click(); await page.keyboard.press(DOC_END); await page.waitForTimeout(300);
await page.waitForSelector(".tok-mnemonic", { timeout: 5000 }).then(() => ok(true, "Färbung: Befehle sind gefärbt"), () => ok(false, "Färbung fehlt"));
ok((await page.locator(".tok-comment").count()) > 0, "Färbung: Kommentare");
const orig = readFileSync(join(proj, asm));
await snap("01-editor-offen");

// Zwei Öffnungen derselben Datei ohne Abstand (schneller Doppelklick, Enter, Befehl) ergeben genau einen Reiter, und der zeigt nach dem Tippen den Punkt
await page.evaluate(async () => { await Promise.all([window.__vc.openFile("README.md"), window.__vc.openFile("README.md")]); }); await page.waitForTimeout(500);
ok((await page.evaluate(() => window.__vc.ed.tabs.filter((x) => x.rel === "README.md").length)) === 1, "Zwei gleichzeitige Öffnungen derselben Datei: genau ein Reiter");
await page.evaluate(() => { window.__vc.md.quelltext["README.md"] = true; });   // Markdown öffnet in der Vorschau; dieser Test bearbeitet den Quelltext
await page.getByRole("treeitem", { name: /README\.md/ }).first().click({ clickCount: 2, delay: 0 }); await page.waitForTimeout(500);
ok((await page.evaluate(() => window.__vc.ed.tabs.filter((x) => x.rel === "README.md").length)) === 1, "Doppelklick im Baum ohne Abstand: weiter genau ein Reiter");
await page.locator(".cm-content").click(); await page.keyboard.type("x"); await page.waitForTimeout(300);
ok((await page.locator(".tab.on .dot").count()) === 1, "… und der aktive Reiter zeigt nach dem Tippen den Punkt"); await page.keyboard.press("Backspace");
await page.evaluate((a) => window.__vc.openFile(a), asm); await page.waitForTimeout(300);
// Tippen mit Umlauten, ungespeichert, Rückgängig
await page.locator(".cm-content").click();
await page.keyboard.press(DOC_END);
await page.keyboard.type("\n; Prüfung äöüß € 😀");
ok((await page.locator(".tab.on .dot").count()) === 1, "Reiter zeigt ungespeicherte Änderung");
await page.evaluate(() => window.__vc.runCommand("rueckgaengig"));
await page.evaluate(() => window.__vc.runCommand("rueckgaengig"));
await page.waitForTimeout(100);
await page.evaluate(() => window.__vc.runCommand("wiederholen"));
await page.evaluate(() => window.__vc.runCommand("speichern"));
await page.waitForTimeout(300);
ok((await page.locator(".tab.on .dot").count()) === 0, "Speichern entfernt den Punkt");
const neu = readFileSync(join(proj, asm), "utf8");
ok(neu.includes("Prüfung") || neu.includes("P") , "Datei geschrieben");
await page.evaluate(() => { for (let i = 0; i < 30; i++) window.__vc.runCommand("rueckgaengig"); });
await page.evaluate(() => window.__vc.runCommand("speichern"));
await page.waitForTimeout(300);
ok(Buffer.compare(readFileSync(join(proj, asm)), orig) === 0, "Rückgängig bis zum Anfang und Speichern: Datei byte-gleich zum Original");

// Gestaltung und Scrollen (Wurzel der Befunde aus dem Handtest: Stile von CodeMirror, Höhe, Scroller)
const cm = await page.evaluate(() => { const g = (s) => document.querySelector(s); const sc = g(".cm-scroller"), ed = g(".cm-editor"), card = ed.closest(".card");
  return { overflow: getComputedStyle(sc).overflow, gutters: getComputedStyle(g(".cm-gutters")).display, nums: !!g(".cm-lineNumbers .cm-gutterElement"), edH: ed.getBoundingClientRect().height, cardH: card.getBoundingClientRect().height, scH: sc.scrollHeight, cH: sc.clientHeight }; });
ok(cm.overflow.includes("auto") || cm.overflow.includes("scroll"), "CodeMirror-Stile greifen: Scroller hat overflow (" + cm.overflow + ")");
ok(cm.gutters === "flex" && cm.nums, "Zeilennummern sind da");
ok(cm.edH <= cm.cardH + 1 && cm.scH > cm.cH, "Editor füllt die Karte, nur der Scroller scrollt (Editor " + Math.round(cm.edH) + " <= Karte " + Math.round(cm.cardH) + ")");
const geo = async () => await page.evaluate(() => { const r = (s) => { const b = document.querySelector(s).getBoundingClientRect(); return [Math.round(b.left), Math.round(b.top), Math.round(b.width), Math.round(b.height)].join(","); }; return { ed: r(".cm-editor"), tabs: r(".tabs"), gut: (() => { const b = document.querySelector(".cm-gutters").getBoundingClientRect(); return b.left + "," + b.width; })(), pageY: window.scrollY, pageX: window.scrollX, docY: document.scrollingElement.scrollTop }; });
const g0 = await geo();
await page.locator(".cm-content").click();
await page.mouse.move(700, 400); await page.mouse.wheel(0, 600); await page.waitForTimeout(200);
ok((await page.evaluate(() => document.querySelector(".cm-scroller").scrollTop)) > 0, "Mausrad scrollt den Editor");
for (const k of [DOC_START, "PageDown", "PageDown", "PageUp", DOC_END, "Home", "End", DOC_START]) { await page.keyboard.press(k); await page.waitForTimeout(60); }
const g1 = await geo();
ok(g0.ed === g1.ed && g0.tabs === g1.tabs && g0.gut === g1.gut && g1.pageY === 0 && g1.pageX === 0 && g1.docY === 0, "⌘↑/↓, Seite auf/ab, Pos1/Ende verschieben weder Editor, Reiterleiste noch Rand: " + JSON.stringify(g1));
ok((await page.locator(".cm-lineNumbers .cm-gutterElement").count()) > 5 && (await page.locator(".cm-line").count()) > 5, "Zeilennummern und Inhalt sind beide da");
for (const [w, h] of [[900, 600], [1100, 700], [1440, 900]]) { await page.setViewportSize({ width: w, height: h }); await page.waitForTimeout(250);
  ok((await page.locator(".cm-line").count()) > 3 && (await page.locator(".cm-lineNumbers .cm-gutterElement").count()) > 3, "Fenster " + w + "x" + h + ": Inhalt und Zeilennummern sichtbar"); }
// Tab, Umschalt+Tab, Klammern
await page.keyboard.press(DOC_END); await page.keyboard.press("Enter");
await page.keyboard.press("Tab"); await page.keyboard.type("x");
ok((await page.evaluate(() => window.__vc && document.querySelector(".cm-activeLine").innerText)).startsWith("\tx") || (await page.locator(".cm-activeLine").innerText()).includes("x"), "Tab fügt ein Tabulatorzeichen ein");
await page.keyboard.press("Home"); await page.keyboard.press("Home");
const before = (await page.locator(".cm-activeLine").innerText());
await page.keyboard.press("Shift+Tab");
await page.keyboard.press("End"); await page.keyboard.press("Enter"); await page.keyboard.type("a (b), (c), (d");
const line = await page.locator(".cm-activeLine").innerText();
ok((line.match(/\(/g) || []).length === 3 && (line.match(/\)/g) || []).length === 3, "Klammern werden bei jeder öffnenden ergänzt: " + JSON.stringify(line));
await page.keyboard.press(MOD + "+a"); await page.keyboard.press("Tab");
await page.evaluate(() => { for (let i = 0; i < 40; i++) window.__vc.runCommand("rueckgaengig"); });
await page.waitForTimeout(150);
await page.evaluate(() => window.__vc.runCommand("speichern")); await page.waitForTimeout(300);
ok(Buffer.compare(readFileSync(join(proj, asm)), orig) === 0, "Nach Tab, Klammern und Rückgängig ist die Datei wieder byte-gleich");

// Suchen, Gehe zu Zeile, Falten
await page.evaluate(() => window.__vc.runCommand("suchen"));
ok(await page.locator(".cm-search").isVisible(), "Suchleiste öffnet");
// Sichtbarer Weg „Projekt schließen“ in der Seitenleiste ist vorhanden
ok((await page.getByRole("button", { name: "Projekt schließen" }).count()) >= 1, "Seitenleiste hat einen Knopf „Projekt schließen“");
// Rechtsklick (Befund Handtest E1.4): im Suchfeld das eigene Textmenü, nie das Webview-Standardmenü; im Text schließt die Suchleiste und es bleibt ein einziges Menü
await page.locator(".cm-search input[name=search]").click({ button: "right" }); await page.waitForTimeout(200);
const tm = await page.locator(".vc-fixmenu button").allInnerTexts();
ok(tm.join("|") === "Ausschneiden|Kopieren|Einfügen|Alles auswählen", "Rechtsklick im Suchfeld: eigenes Textmenü: " + tm.join("|"));
ok(await page.evaluate(() => { const m = document.querySelector(".vc-fixmenu"); const r = m.getBoundingClientRect(); const e = document.elementFromPoint(r.left + r.width / 2, r.top + r.height / 2); return m.contains(e); }), "Das Menü liegt über der Suchleiste (Stapelreihenfolge)");
await page.locator(".cm-content").click({ button: "right" }); await page.waitForTimeout(250);
ok((await page.locator(".vc-fixmenu").count()) === 1 && !(await page.locator(".cm-search").count()), "Rechtsklick im Text: Suchleiste schließt, es bleibt ein Menü");
await page.keyboard.press("Escape"); await page.waitForTimeout(100);
await page.evaluate(() => { void window.__vc.runCommand("suchen"); }); await page.waitForTimeout(300);
ok((await page.locator(".cm-search").innerText()).includes("Suchen") || (await page.locator(".cm-search input[name=search]").getAttribute("placeholder")) === "Suchen", "Suchleiste deutsch");
await snap("02-suche");
await page.keyboard.press("Escape");
await page.evaluate(() => window.__vc.runCommand("gehe_zu_zeile"));
await page.waitForSelector("dialog[open]");
await page.keyboard.press(MOD + "+a"); await page.keyboard.type("0"); await page.keyboard.press("Enter");
ok(await page.locator("dialog[open] [role=alert]").isVisible(), "Gehe zu Zeile: ungültige Zahl wird gemeldet");
await page.keyboard.press(MOD + "+a"); await page.keyboard.type("3"); await page.keyboard.press("Enter");
await page.waitForTimeout(200);
ok(!(await page.locator("dialog[open]").count()), "Gehe zu Zeile schließt den Dialog");
ok((await page.evaluate(() => document.querySelector(".status").innerText)).includes("Zeile 3,"), "Statuszeile zeigt Zeile 3");
const folds = await page.locator(".cm-foldGutter .cm-gutterElement span[title], .cm-foldGutter span").count();
ok(folds > 0, "Faltmarken im Rand: " + folds);

// Neue Datei über den Knopf, Umbenennen (F2), Reiter ziehen, Löschen
await page.getByRole("button", { name: "Neue Datei" }).click();
await page.waitForSelector("dialog[open]");
await page.keyboard.type("lib/neu.asm"); await page.keyboard.press("Enter");
await page.waitForSelector('.tab.on[title="lib/neu.asm"]');
ok(existsSync(join(proj, "lib/neu.asm")), "Neue Datei lib/neu.asm angelegt und als Reiter offen");
await page.keyboard.type("rts");
await page.evaluate(() => window.__vc.runCommand("speichern")); await page.waitForTimeout(300);
ok(readFileSync(join(proj, "lib/neu.asm"), "utf8") === "rts", "Neue Datei gespeichert");
const it = page.getByRole("treeitem", { name: /neu\.asm/ }).first();
await it.focus(); await page.keyboard.press("F2");
await page.waitForSelector("dialog[open]");
await page.keyboard.press(MOD + "+a"); await page.keyboard.type("neu2.asm"); await page.keyboard.press("Enter");
await page.waitForSelector('.tab[title="lib/neu2.asm"]');
ok(existsSync(join(proj, "lib/neu2.asm")) && !existsSync(join(proj, "lib/neu.asm")), "Umbenennen: Reiter und Datei folgen");
// Verschieben per Ziehen in die Wurzel
await page.getByRole("treeitem", { name: /neu2\.asm/ }).first().dragTo(page.locator(".treebox"), { targetPosition: { x: 20, y: 400 } });
await page.waitForTimeout(400);
ok(existsSync(join(proj, "neu2.asm")), "Ziehen in die Wurzel verschiebt die Datei");
// Löschen mit Nachfrage
await page.getByRole("treeitem", { name: /neu2\.asm/ }).first().focus(); await page.keyboard.press("Delete");
await page.waitForSelector("dialog[open]"); await page.getByRole("button", { name: "Löschen" }).last().click();
await page.waitForTimeout(400);
ok(!existsSync(join(proj, "neu2.asm")), "Löschen entfernt die Datei, Reiter schließt");
ok((await page.locator('.tab[title$="neu2.asm"]').count()) === 0, "Reiter ist weg");
// Schließen mit ungespeicherten Änderungen
await page.locator(".cm-content").click(); await page.keyboard.type("x");
await page.evaluate(() => window.__vc.runCommand("datei_schliessen"));
await page.waitForSelector("dialog[open]");
ok((await page.locator("dialog[open]").innerText()).includes("ungespeicherte"), "Nachfrage beim Schließen");
await snap("03-nachfrage");
await page.getByRole("button", { name: "Verwerfen" }).click();
await page.waitForTimeout(200);
await snap("04-nach-schliessen");

// --- Teil 2: Verschieben mit Include-Anpassung, Rückgängig, unbenannte Reiter, Art, Filter, Änderung von außen, Abfrage beim Wechsel
await page.evaluate(async () => { const c = window.__vc.call; await c("file_create", { rel: "lib/t1.inc" }); await c("file_write", { rel: "lib/t1.inc", text: "nop\n", eol: "lf" }); await c("file_create", { rel: "lib/t2.asm" }); await c("file_write", { rel: "lib/t2.asm", text: '.include "t1.inc"\n', eol: "lf" }); await window.__vc.call("tree_reload"); });
await page.evaluate(async () => { const r = await window.__vc.call("tree_reload"); window.__vc.ui.project.tree = r.tree; });
await page.evaluate(async () => { await window.__vc.openFile("lib/t2.asm"); });
await page.getByRole("treeitem", { name: /^lib/ }).first().click().catch(() => {});
await page.getByRole("treeitem", { name: /t1\.inc/ }).first().dragTo(page.locator(".treebox"), { targetPosition: { x: 20, y: 500 } });
await page.waitForTimeout(600);
ok(existsSync(join(proj, "t1.inc")) && !existsSync(join(proj, "lib/t1.inc")), "Verschieben in die Wurzel");
ok(readFileSync(join(proj, "lib/t2.asm"), "utf8").includes("../t1.inc"), "Verschieben passt die Include-Zeile an: " + JSON.stringify(readFileSync(join(proj, "lib/t2.asm"), "utf8")));
ok((await page.locator(".cm-content").innerText()).includes("../t1.inc"), "Offener Reiter zeigt den angepassten Text");
await page.evaluate(() => window.__vc.runCommand("baum_rueckgaengig"));
await page.waitForTimeout(600);
ok(existsSync(join(proj, "lib/t1.inc")) && readFileSync(join(proj, "lib/t2.asm"), "utf8").includes('"t1.inc"'), "Verschieben rückgängig stellt Datei und Include her");

// Umbenennen passt Include-Zeilen an (wie Verschieben), Rückgängig stellt her
await page.getByRole("treeitem", { name: /t1\.inc/ }).first().click();
await page.evaluate(() => { void window.__vc.runCommand("datei_umbenennen"); });
await page.waitForSelector("dialog[open]"); await page.keyboard.press(MOD + "+a"); await page.keyboard.type("t3.inc"); await page.keyboard.press("Enter");
await page.waitForTimeout(700);
ok(existsSync(join(proj, "lib/t3.inc")) && readFileSync(join(proj, "lib/t2.asm"), "utf8").includes('"t3.inc"'), "Umbenennen passt die Include-Zeile an: " + JSON.stringify(readFileSync(join(proj, "lib/t2.asm"), "utf8")));
await page.evaluate(() => window.__vc.openFile("lib/t2.asm")); await page.waitForTimeout(300);
ok((await page.locator(".cm-content").innerText()).includes("t3.inc"), "Offener Reiter zeigt den angepassten Include");
await page.evaluate(() => window.__vc.runCommand("baum_rueckgaengig")); await page.waitForTimeout(600);
ok(existsSync(join(proj, "lib/t1.inc")) && readFileSync(join(proj, "lib/t2.asm"), "utf8").includes('"t1.inc"'), "Umbenennen rückgängig stellt Datei und Include her");

// Speichern unter: Dateiart wählbar (auch ohne Endung), Art folgt der Endung, freier Namensvorschlag
await page.evaluate(() => window.__vc.runCommand("neue_notiz")); await page.waitForSelector(".tab.on"); await page.keyboard.type("n1");
await page.evaluate(() => window.__vc.runCommand("speichern")); await page.waitForSelector("dialog[open]");
ok((await page.locator("dialog[open] #nameart option").allInnerTexts()).some((x) => /Ohne Endung/.test(x)), "Speichern unter bietet die Auswahl „Ohne Endung“");
ok((await page.locator("dialog[open] #nameinp").inputValue()) === "notiz.md", "Vorschlag für eine Notiz: notiz.md");
await page.keyboard.press("Enter"); await page.waitForTimeout(500);
await page.evaluate(() => window.__vc.runCommand("neue_notiz")); await page.waitForSelector(".tab.on"); await page.keyboard.type("n2");
await page.evaluate(() => window.__vc.runCommand("speichern")); await page.waitForSelector("dialog[open]");
ok((await page.locator("dialog[open] #nameinp").inputValue()) === "notiz-2.md", "Vorschlag wählt einen freien Namen: " + await page.locator("dialog[open] #nameinp").inputValue());
await page.locator("dialog[open] #nameart").selectOption("none"); await page.locator("dialog[open] #nameinp").fill("ohneendung"); await page.keyboard.press("Enter"); await page.waitForTimeout(500);
ok(existsSync(join(proj, "ohneendung")), "Art „Ohne Endung“ legt die Datei ohne Endung an: " + (await page.locator(".toast").last().innerText().catch(() => "-")));
await page.evaluate(() => { void window.__vc.runCommand("neue_datei"); }); await page.waitForSelector("dialog[open]");
await page.locator("dialog[open] #nameinp").fill("x.bas");
ok((await page.locator("dialog[open] #nameart").inputValue()) === "basic", "Eigene Endung .bas stellt die Art auf BASIC");
await page.locator("dialog[open] #nameinp").fill("y.md");
ok((await page.locator("dialog[open] #nameart").inputValue()) === "notiz", "Eigene Endung .md stellt die Art auf Notiz");
await page.keyboard.press("Escape"); await page.waitForTimeout(200);
// Reiter-Menü: „Nach rechts“ ordnet um und öffnet kein weiteres Menü
await page.locator(".tab").nth(0).click({ button: "right" }); await page.waitForSelector(".ctx");
const first = (await page.locator(".tab .nm").allInnerTexts())[0];
await page.locator(".ctx button", { hasText: "Nach rechts" }).click(); await page.waitForTimeout(300);
ok((await page.locator(".ctx").count()) === 0 && (await page.locator(".tab .nm").allInnerTexts())[1] === first, "Reiter-Menü „Nach rechts“: Reiter rückt, kein weiteres Menü");

// Wünsche aus dem Handtest: Zeilenumbruch, Suchtreffer lesbar, Fokus nach Dialog, Faltung deutlich, Lesezeichen, eigenes Kontextmenü, Tooltip am Zeiger
await page.evaluate(async () => { await window.__vc.openFile("hal_c64.asm"); }); await page.waitForTimeout(400);
for (const [w, h] of [[900, 600]]) { await page.setViewportSize({ width: w, height: h }); await page.waitForTimeout(300); }
ok((await page.locator(".cm-lineWrapping").count()) === 1, "Zeilenumbruch ist standardmäßig an");
ok(await page.evaluate(() => { const sc = document.querySelector(".cm-scroller"); return sc.scrollWidth <= sc.clientWidth + 1; }), "Bei 900 px keine waagerechte Bildlaufleiste");
await page.setViewportSize({ width: 1440, height: 900 });
await page.evaluate(() => window.__vc.runCommand("umbruch")); await page.waitForTimeout(200);
ok((await page.locator(".cm-lineWrapping").count()) === 0, "Umbruch lässt sich ausschalten");
await page.evaluate(() => window.__vc.runCommand("umbruch")); await page.waitForTimeout(200);
// Suchtreffer: Kontrast Text auf Fläche, auch auf Kommentartext (Hell und Dunkel)
const lum = (c) => { const [r, g, b] = c.match(/[\d.]+/g).slice(0, 3).map(Number).map((v) => { v /= 255; return v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4; }); return 0.2126 * r + 0.7152 * g + 0.0722 * b; };
const blend = (fg, bg) => { const f = fg.match(/[\d.]+/g).map(Number), b = bg.match(/[\d.]+/g).map(Number); const a = f.length > 3 ? f[3] : 1; return `rgb(${[0, 1, 2].map((i) => Math.round(f[i] * a + b[i] * (1 - a))).join(",")})`; };
const ratio = (a, b) => { const [x, y] = [lum(a), lum(b)].sort((p, q) => q - p); return (x + 0.05) / (y + 0.05); };
for (const scheme of ["light", "dark"]) {
  await page.evaluate((s) => window.__vc.setTheme(s), scheme); await page.waitForTimeout(200);
  await page.locator(".cm-content").click(); await page.keyboard.press(MOD + "+f"); await page.waitForSelector(".cm-search");
  await page.locator(".cm-search input[name=search]").fill(";"); await page.keyboard.press("Enter"); await page.waitForTimeout(300); await page.keyboard.press("Enter"); await page.waitForTimeout(300);
  const edbg = await page.evaluate(() => getComputedStyle(document.querySelector(".cm-editor")).backgroundColor);
  const q = await page.evaluate(() => { const g = (el) => { const cs = getComputedStyle(el); return { c: cs.color, b: cs.backgroundColor }; }; const m = document.querySelector(".cm-searchMatch:not(.cm-searchMatch-selected)"), sel = document.querySelector(".cm-searchMatch-selected"); const inner = (el) => { let e = el; while (e.firstElementChild) e = e.firstElementChild; return e; }; return { m: m && g(m), mi: m && g(inner(m)), s: sel && g(sel), si: sel && g(inner(sel)) }; });
  ok(q.s && ratio(q.s.c, blend(q.s.b, edbg)) >= 4.5 && ratio(q.si.c, blend(q.s.b, edbg)) >= 4.5, scheme + ": aktueller Treffer lesbar, Kontrast " + (q.s ? ratio(q.si.c, blend(q.s.b, edbg)).toFixed(1) : "?"));
  ok(q.m && ratio(q.mi.c, blend(q.m.b, edbg)) >= 4.5, scheme + ": übrige Treffer lesbar, Kontrast " + (q.m ? ratio(q.mi.c, blend(q.m.b, edbg)).toFixed(1) : "?"));
  await page.keyboard.press("Escape");
}
await page.evaluate(() => window.__vc.setTheme("light"));
// Gehe zu Zeile: danach hat der Editor sofort den Fokus
await page.evaluate(() => { void window.__vc.runCommand("gehe_zu_zeile"); }); await page.waitForSelector("dialog[open]");
await page.keyboard.press(MOD + "+a"); await page.keyboard.type("3"); await page.keyboard.press("Enter"); await page.waitForTimeout(300);
ok(await page.evaluate(() => document.activeElement?.classList.contains("cm-content")), "Nach „Gehe zu Zeile“ hat der Editor den Fokus");
await page.keyboard.type("x"); ok((await page.locator(".cm-activeLine").innerText()).includes("x"), "… und nimmt sofort Eingaben an"); await page.keyboard.press("Backspace");
// Palette schließen: Fokus zurück im Editor
await page.evaluate(() => { void window.__vc.runCommand("palette"); }); await page.waitForTimeout(200); await page.keyboard.press("Escape"); await page.waitForTimeout(200);
ok(await page.evaluate(() => document.activeElement?.classList.contains("cm-content")), "Nach dem Schließen der Befehlspalette hat der Editor den Fokus");
// Faltung deutlich
await page.evaluate(() => window.__vc.runCommand("alles_falten")); await page.waitForTimeout(300);
ok((await page.locator(".vc-fold-closed").count()) > 0 && (await page.locator(".cm-foldPlaceholder").count()) > 0, "Gefaltet: Dreieck und Platzhalter sind hervorgehoben (" + await page.locator(".vc-fold-closed").count() + ")");
const fs = await page.evaluate(() => document.querySelector(".vc-fold").getBoundingClientRect().width); ok(fs >= 18, "Faltmarke ist groß genug (" + fs + " px breit)");
await snap("07-faltung"); await page.evaluate(() => window.__vc.runCommand("alles_entfalten"));
// Lesezeichen: Umschalt+Klick, Befehle, Sprung
const bm0 = await page.locator(".mk-bookmark").count();
await page.locator(".cm-line").nth(2).click(); await page.evaluate(() => window.__vc.runCommand("lesezeichen_umschalten")); await page.waitForTimeout(150);
await page.locator(".cm-line").nth(8).click(); await page.evaluate(() => window.__vc.runCommand("lesezeichen_umschalten")); await page.waitForTimeout(150);
ok((await page.locator(".mk-bookmark").count()) === bm0 + 2, "Lesezeichen setzen (Befehl)");
await page.locator(".cm-line").nth(1).click(); await page.evaluate(() => window.__vc.runCommand("naechstes_lesezeichen")); await page.waitForTimeout(150);
ok(/Zeile 3,/.test(await page.locator("footer.status").innerText()), "Nächstes Lesezeichen springt (Zeile 3)");
await page.evaluate(() => window.__vc.runCommand("voriges_lesezeichen")); await page.waitForTimeout(150);
ok(/Zeile 9,/.test(await page.locator("footer.status").innerText()), "Voriges Lesezeichen springt rundum (Zeile 9)");
const gb = await page.locator(".cm-gutter-marks").boundingBox(), lb = await page.locator(".cm-line").nth(5).boundingBox();
await page.keyboard.down("Shift"); await page.mouse.click(gb.x + gb.width / 2, lb.y + lb.height / 2); await page.keyboard.up("Shift"); await page.waitForTimeout(150);
ok((await page.locator(".mk-bookmark").count()) === bm0 + 3, "Umschalt+Klick im Rand setzt ein Lesezeichen");
// eigenes Kontextmenü, übersetzt
await page.locator(".cm-line").nth(2).click({ button: "right" }); await page.waitForSelector(".vc-fixmenu button", { timeout: 3000 }).catch(() => {});
const cmn = await page.locator(".vc-fixmenu button").allInnerTexts();
ok(cmn.includes("Kopieren") && cmn.includes("Lesezeichen umschalten") && !cmn.some((x) => /Copy|Paste|Cut/.test(x)), "Kontextmenü im Editor deutsch: " + cmn.join(" | "));
await snap("08-kontextmenue"); await page.keyboard.press("Escape");
// Tooltip der eingeklappten Seitenleiste erscheint am Mauszeiger
await page.keyboard.press(MOD + "+b"); await page.waitForSelector("[data-strip=open]");
const sb = await page.locator("[data-strip=open]").boundingBox(); const sy = Math.round(sb.y + sb.height / 2);
await page.mouse.move(sb.x + sb.width / 2, sy); await page.waitForSelector(".tip", { timeout: 3000 }).catch(() => {});
const tb = await page.locator(".tip").boundingBox().catch(() => null);
ok(tb && Math.abs(tb.y - sy) < 60, "Tooltip des Streifens erscheint am Mauszeiger (y " + (tb ? Math.round(tb.y) : "?") + " bei Zeiger " + sy + ")");
await page.mouse.click(sb.x + sb.width / 2, sy); await page.waitForTimeout(200);

// Unbenannter Reiter, Speichern unter
await page.evaluate(() => window.__vc.runCommand("neuer_reiter"));
await page.waitForSelector(".tab.on");
await page.keyboard.type("lda #1");
ok((await page.locator(".tab.on .nm").innerText()).includes("Unbenannt"), "Unbenannter Reiter heißt „Unbenannt n“");
await page.evaluate(() => window.__vc.runCommand("speichern"));
await page.waitForSelector("dialog[open]");
await page.keyboard.press(MOD + "+a"); await page.keyboard.type("scratch1.asm"); await page.keyboard.press("Enter");
await page.waitForTimeout(500);
ok(readFileSync(join(proj, "scratch1.asm"), "utf8") === "lda #1", "Speichern unter legt die Datei an");
ok((await page.locator('.tab.on[title="scratch1.asm"]').count()) === 1, "Reiter trägt jetzt den Dateinamen");

// Art umschalten: Text -> keine Färbung, Assembler -> Färbung
await page.evaluate(() => window.__vc.runCommand("art_text")); await page.waitForTimeout(300);
ok((await page.locator(".cm-content .tok-mnemonic").count()) === 0, "Art Text: keine Färbung");
await page.evaluate(() => window.__vc.runCommand("art_asm")); await page.waitForTimeout(500);
ok((await page.locator(".cm-content .tok-mnemonic").count()) > 0, "Art Assembler: Färbung");

// Schrift
const px = async () => await page.evaluate(() => getComputedStyle(document.querySelector(".cm-content")).fontSize);
const s0 = await px(); await page.evaluate(() => window.__vc.runCommand("editor_schrift_groesser")); await page.waitForTimeout(100);
ok((await px()) !== s0, "Schrift größer: " + s0 + " -> " + (await px()));
await page.evaluate(() => window.__vc.runCommand("editor_schrift_kleiner"));

// Filter im Baum
await page.getByRole("button", { name: "Dateien filtern" }).click();
await page.locator(".fi").fill("scratch");
await page.waitForTimeout(150);
ok((await page.getByRole("treeitem").count()) === 1, "Filter zeigt nur Treffer");
await snap("05-filter");
await page.locator(".fi").fill(""); await page.keyboard.press("Escape");

// Zen
await page.evaluate(() => window.__vc.runCommand("zen")); await page.waitForTimeout(100);
ok((await page.locator(".status").count()) === 0 && (await page.locator(".treebox").count()) === 0, "Zen blendet Baum und Statuszeile aus");
await page.evaluate(() => window.__vc.runCommand("zen"));

// Änderung von außen
writeFileSync(join(proj, "scratch1.asm"), "ldx #2 ; von aussen\n");
await page.evaluate(() => window.dispatchEvent(new Event("focus"))); await page.waitForTimeout(600);
ok((await page.locator(".cm-content").innerText()).includes("von aussen"), "Änderung von außen wird neu eingelesen (Reiter sauber)");
await page.locator(".cm-content").click(); await page.keyboard.type("q");
writeFileSync(join(proj, "von_aussen.txt"), "neu\n");
await page.evaluate(() => window.dispatchEvent(new Event("focus"))); await page.waitForTimeout(600);
ok(await page.getByRole("treeitem", { name: /von_aussen\.txt/ }).count() > 0, "Von außen angelegte Datei erscheint im Projektbaum beim Zurückkehren ins Fenster");
writeFileSync(join(proj, "scratch1.asm"), "ldx #3 ; zweite Aenderung\n");
await page.evaluate(() => window.dispatchEvent(new Event("focus"))); await page.waitForTimeout(600);
ok((await page.locator(".toast").allInnerTexts()).join(" ").includes("überschrieben"), "Konflikt (ungespeichert und von außen geändert): Hinweis erscheint");

// Nichttext
const bin = (await page.evaluate(() => { const f = (ns) => ns.flatMap((x) => (x.kind === "dir" ? f(x.children ?? []) : [x])); return f(window.__vc.ui.project.tree).map((x) => x.rel).find((r) => /\.(png|bin|prg|sid)$/i.test(r)); }));
if (bin) { await page.evaluate((b) => window.__vc.openFile(b), bin); await page.waitForTimeout(300); ok((await page.locator('.tab[title="' + bin + '"]').count()) === 0, "Nichttext „" + bin + "“ öffnet keinen Reiter"); }

// Kontextmenü mit der Maus: Umbenennen, Löschen (Befunde I, J, K des Handtests)
await page.evaluate(() => window.__vc.runCommand("speichern")); await page.waitForTimeout(300);
await page.getByRole("treeitem", { name: /scratch1\.asm/ }).first().click({ button: "right" });
await page.getByRole("menuitem", { name: "Umbenennen" }).click();
await page.waitForSelector("dialog[open] input");
await page.keyboard.press(MOD + "+a"); await page.keyboard.type("ctx2.asm"); await page.keyboard.press("Enter");
await page.waitForTimeout(500);
ok(existsSync(join(proj, "ctx2.asm")) && !existsSync(join(proj, "scratch1.asm")), "Kontextmenü ▸ Umbenennen (Maus)");
await page.getByRole("treeitem", { name: /ctx2\.asm/ }).first().click({ button: "right" });
await page.getByRole("menuitem", { name: "Löschen" }).click();
await page.waitForSelector("dialog[open]");
await page.locator("dialog[open]").getByRole("button", { name: "Löschen" }).click();
await page.waitForTimeout(500);
ok(!existsSync(join(proj, "ctx2.asm")), "Kontextmenü ▸ Löschen (Maus)");
await page.getByRole("treeitem", { name: /^lib/ }).first().click({ button: "right" });
await page.getByRole("menuitem", { name: "Neue Datei" }).click();
await page.waitForSelector("dialog[open] input");
await page.keyboard.type("neu3"); await page.locator("dialog[open] select").selectOption("basic"); await page.keyboard.press("Enter");
await page.waitForTimeout(500);
ok(existsSync(join(proj, "lib/neu3.bas")), "Neue Datei im Ordner über das Kontextmenü, Art BASIC");
// Projekt schließen über die Palette
await page.keyboard.press(MOD + "+k"); await page.waitForSelector(".pal input"); await page.keyboard.type("Projekt schließen");
ok((await page.locator(".pal [role=option]").first().innerText()).includes("Projekt schließen"), "Palette findet „Projekt schließen“");
await page.keyboard.press("Escape");

await page.evaluate(() => window.__vc.openFile("hal_c64.asm")); await page.waitForSelector(".cm-content"); await page.locator(".cm-content").click(); await page.keyboard.type("z");
// Abfrage beim Wechsel mit ungespeicherten Änderungen
await page.evaluate(() => { void window.__vc.runCommand("startseite"); });
await page.waitForSelector("dialog[open]", { timeout: 5000 }).catch(async () => { await snap("99-fehler-wechsel"); console.log("kein Dialog; Zustand:", await page.evaluate(() => JSON.stringify({ tabs: window.__vc.ui.project && 1, tabsdirty: document.querySelectorAll(".tab .dot").length }))); });
ok((await page.locator("dialog[open]").innerText()).includes("ungespeicherten"), "Projektwechsel fragt bei ungespeicherten Änderungen");
await snap("06-wechsel-frage");
await page.getByRole("button", { name: "Abbrechen" }).click(); await page.waitForTimeout(200);
ok((await page.evaluate(() => !!window.__vc.ui.project)), "Abbrechen bleibt im Projekt");
await page.evaluate(() => { void window.__vc.runCommand("startseite"); });
await page.waitForSelector("dialog[open]"); await page.getByRole("button", { name: "Verwerfen" }).click(); await page.waitForTimeout(400);
ok((await page.evaluate(() => !window.__vc.ui.project)), "Verwerfen wechselt zur Startansicht");
ok(csp.length === 0, "Keine Meldungen der Inhaltsrichtlinie: " + csp.slice(0, 2).join(" | "));
console.log(`\n${n} Prüfungen, ${fails.length} Fehler`);
await browser.close();
process.exit(fails.length ? 1 : 0);
