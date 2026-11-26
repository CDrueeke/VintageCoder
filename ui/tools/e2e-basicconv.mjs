// SPDX-License-Identifier: GPL-3.0-or-later
// Ablauftest BASIC umwandeln im Editor: In Labels umwandeln, In Nummern umwandeln, Neu nummerieren (Dialog mit Vorschau, Anwenden = ein Rückgängig-Schritt),
// Modus-Anzeige, Fehler bei berechnetem Sprungziel, Bereich, Einbindungen, Kontextmenü, Englisch.
// Aufruf: node tools/e2e-basicconv.mjs <Projektkopie examples/c64/basic_native> [Bilder]
//   Kern mit HOME=<leer> VC_TESTBRIDGE=17420 VC_TEST_NO_SPAWN=1 VC_NO_SAVE=1 (Vite: VC_VITE, Brücke: VC_BRIDGE)
import { webkit } from "playwright";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const [proj, shots] = process.argv.slice(2);
if (!proj) { console.error("Aufruf: node tools/e2e-basicconv.mjs <Projekt> [Bilder]"); process.exit(2); }
if (shots) mkdirSync(shots, { recursive: true });
const browser = await webkit.launch();
const page = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
const fails = []; let n = 0; const csp = [];
const ok = (c, m) => { n++; if (!c) { fails.push(m); console.log("FEHLER:", m); } else console.log("ok:", m); };
page.on("pageerror", (e) => { fails.push("Seitenfehler " + e.message); console.log("Seitenfehler", e.message); });
page.on("console", (m) => { if (/Content Security Policy|Refused to/i.test(m.text())) csp.push(m.text()); });
const snap = async (name) => { if (shots) await page.screenshot({ path: join(shots, name + ".png") }); };
await page.goto(`http://127.0.0.1:${process.env.VC_VITE ?? 1420}/?bridge=${process.env.VC_BRIDGE ?? 17420}`); await page.waitForFunction(() => window.__vc && window.__vc.ui.ready);
await page.evaluate(async () => { await window.__vc.setLanguage("de"); }); await page.waitForTimeout(400);
await page.evaluate(async (p) => { await window.__vc.openProject(p); }, proj); await page.waitForTimeout(600);

const oeffnen = async (rel) => { await page.evaluate(async (r) => { window.__vc.ui.extra = null; const t = await window.__vc.call("tree_reload"); window.__vc.ui.project.tree = t.tree; await window.__vc.openFile(r); }, rel); await page.waitForSelector(".cm-editor .cm-content"); await page.waitForTimeout(300); };
const editorText = () => page.locator(".cm-content").innerText();
const befehl = (id) => page.evaluate((i) => { void window.__vc.runCommand(i); }, id);
const dlg = page.getByTestId("basicconv");
const zu = async () => { await page.getByTestId("basicconv-abbrechen").click().catch(() => {}); await page.waitForTimeout(200); };

// 1 Nummern → Labels
const orig = "10 PRINT \"START\"\n20 GOTO 50\n30 PRINT \"UEBERSPRUNGEN\"\n40 END\n50 PRINT \"ZIEL\"\n60 GOTO 40\n";
writeFileSync(join(proj, "nummern.bas"), orig);
await oeffnen("nummern.bas");
await befehl("basic_in_labels"); await dlg.waitFor();
await page.waitForSelector("[data-testid=basicconv-zeilen]");
ok(/Zeilennummern/.test(await page.getByTestId("basicconv-modus").innerText()), "Dialog zeigt den Modus „Zeilennummern“: " + (await page.getByTestId("basicconv-modus").innerText()).replace(/\s+/g, " "));
ok(/Zeilen ändern sich/.test(await page.getByTestId("basicconv-summe").innerText()), "Vorschau nennt die Zahl der Zeilen: " + await page.getByTestId("basicconv-summe").innerText());
ok(await page.getByTestId("basicconv-zeilen").locator("li").count() >= 2, "Vorschau listet geänderte Zeilen");
await snap("conv-1-vorschau");
await page.getByTestId("basicconv-anwenden").click(); await page.waitForTimeout(500);
let t1 = await editorText();
ok(/@/.test(t1) && !/GOTO\s*50/.test(t1), "Anwenden: Labels im Editor, GOTO 50 ist ersetzt: " + t1.replace(/\s+/g, " ").slice(0, 100));
ok(await dlg.count() === 0, "Dialog schließt nach Anwenden");
// ein Rückgängig-Schritt stellt alles wieder her
await befehl("rueckgaengig"); await page.waitForTimeout(300);
const t2 = (await editorText()).replace(/ /g, " ");
ok(t2.replace(/\s+/g, " ").trim() === orig.replace(/\s+/g, " ").trim(), "ein Rückgängig-Schritt stellt den Ausgangstext her");

// 2 Labels → Nummern
await befehl("wiederholen"); await page.waitForTimeout(300);
await befehl("basic_in_nummern"); await dlg.waitFor(); await page.waitForSelector("[data-testid=basicconv-zeilen]");
ok(/Labels/.test(await page.getByTestId("basicconv-modus").innerText()), "Modus „Labels“ erkannt");
await page.getByTestId("basicconv-anwenden").click(); await page.waitForTimeout(500);
const t3 = await editorText();
ok(/^\s*10 /m.test(t3) && !/GOTO\s*@/.test(t3), "In Nummern umwandeln: Zeilennummern, keine Labels in Sprüngen: " + t3.replace(/\s+/g, " ").slice(0, 100));

// 3 Neu nummerieren mit Bereich: nur „Von“ → Hinweis, Anwenden aus
await befehl("basic_neu_nummerieren"); await dlg.waitFor(); await page.waitForSelector("[data-testid=basicconv-modus]");
await page.getByTestId("basicconv-von").fill("30");
ok(await page.getByTestId("basicconv-anwenden").isDisabled(), "Bereich nur mit „Von“: Anwenden ist aus");
await page.getByTestId("basicconv-bis").fill("50"); await page.getByTestId("basicconv-bis").blur(); await page.waitForTimeout(600);
ok((await page.getByTestId("basicconv-summe").count()) + (await page.getByTestId("basicconv-meldungen").count()) > 0, "Bereich 30–50: Vorschau oder Meldung des Kerns (Reihenfolge) erscheint");
await zu();

// 4 berechnetes Sprungziel: Fehler B161/B162, Anwenden aus
writeFileSync(join(proj, "rechnen.bas"), "10 A=1\n20 GOTO A*10\n30 END\n");
await oeffnen("rechnen.bas"); await befehl("basic_in_labels"); await dlg.waitFor(); await page.waitForSelector("[data-testid=basicconv-meldungen]");
ok(/B161/.test(await page.getByTestId("basicconv-meldungen").innerText()) && /B162/.test(await page.getByTestId("basicconv-meldungen").innerText()), "berechnetes Sprungziel: B161 und B162 in den Meldungen");
ok(await page.getByTestId("basicconv-anwenden").isDisabled(), "… und Anwenden ist aus");
await snap("conv-2-fehler"); await zu();

// 5 gemischt: Modus und Warnung B160
writeFileSync(join(proj, "gemischt.bas"), "10 PRINT 1\n@x: PRINT 2\n");
await oeffnen("gemischt.bas"); await befehl("basic_in_labels"); await dlg.waitFor(); await page.waitForSelector("[data-testid=basicconv-modus]");
ok(/gemischt/.test(await page.getByTestId("basicconv-modus").innerText()), "gemischtes Programm: Modus „gemischt“");
await page.waitForSelector("[data-testid=basicconv-meldungen]").catch(() => {});
ok(/B160/.test(await page.getByTestId("basicconv-meldungen").innerText().catch(() => "")), "gemischtes Programm: Warnung B160 steht im Dialog"); await zu();
// Einzahl/Mehrzahl
writeFileSync(join(proj, "eins.bas"), "5 PRINT 1\n");
await oeffnen("eins.bas"); await befehl("basic_neu_nummerieren"); await dlg.waitFor(); await page.waitForSelector("[data-testid=basicconv-modus]"); await page.waitForTimeout(400);
const sum1 = await page.getByTestId("basicconv-summe").innerText().catch(() => "");
ok(/^1 Zeile ändert sich, keine neuen Labels\.$/.test(sum1.trim()), "Einzahl: „1 Zeile ändert sich, keine neuen Labels.“ (" + sum1 + ")");
await zu();

// 6 Einbindungen
writeFileSync(join(proj, "lib.bas"), "100 PRINT \"LIB\"\n110 RETURN\n");
writeFileSync(join(proj, "haupt.bas"), "10 GOSUB 100\n20 END\n#include \"lib.bas\"\n");
await oeffnen("haupt.bas"); await befehl("basic_in_labels"); await dlg.waitFor(); await page.waitForSelector("[data-testid=basicconv-includes]");
ok(await page.getByTestId("basicconv-includes").isChecked(), "Option „Eingebundene Dateien mitumwandeln“ ist vorgewählt");
await snap("conv-3-include");
await page.getByTestId("basicconv-anwenden").click(); await page.waitForTimeout(700);
ok(!/^100 /m.test(readFileSync(join(proj, "lib.bas"), "utf8")), "eingebundene Datei wurde mitumgewandelt und geschrieben");

// 6b Medium im Include: wird nicht verändert, Dialog nennt es
mkdirSync(join(proj, "graphics"), { recursive: true });
writeFileSync(join(proj, "graphics/hero.vcsprite"), readFileSync(process.env.VC_SPRITE ?? join(proj, "../../../examples/c64/sprite_demo/graphics/hero.vcsprite")));
const spriteVorher = readFileSync(join(proj, "graphics/hero.vcsprite"));
writeFileSync(join(proj, "medium.bas"), "10 RESTORE\n20 READ A\n30 GOTO 20\n#include \"graphics/hero.vcsprite\"\n");
await oeffnen("medium.bas"); await befehl("basic_in_labels"); await dlg.waitFor(); await page.waitForSelector("[data-testid=basicconv-medien]");
ok(/Medien: 1/.test(await page.getByTestId("basicconv-medien").innerText()), "Dialog nennt das eingebundene Medium");
await page.getByTestId("basicconv-anwenden").click(); await page.waitForTimeout(600);
ok(readFileSync(join(proj, "graphics/hero.vcsprite")).equals(spriteVorher), "das Medium bleibt unverändert");

// 7 Kontextmenü und Englisch
await oeffnen("nummern.bas");
await page.locator(".cm-content").click({ button: "right" }); await page.waitForSelector(".vc-fixmenu");
ok((await page.locator(".vc-fixmenu button").allInnerTexts()).some((x) => /In Labels umwandeln/.test(x)), "Kontextmenü des Editors bietet die Umwandlung an");
await page.keyboard.press("Escape");
await page.evaluate(async () => { await window.__vc.setLanguage("en"); }); await page.waitForTimeout(400);
await befehl("basic_neu_nummerieren"); await dlg.waitFor();
ok(/Renumber/.test(await page.getByRole("dialog").innerText()) && /Preview/.test(await page.getByRole("dialog").innerText()), "Dialog auf Englisch");
await zu();
ok(csp.length === 0, "Keine Verstöße gegen die Inhaltsrichtlinie (" + csp.length + ")");
console.log(`${n} Prüfungen, ${fails.length} Fehler`);
await browser.close();
process.exit(fails.length ? 1 : 0);
