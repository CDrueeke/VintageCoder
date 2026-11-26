// SPDX-License-Identifier: GPL-3.0-or-later
// Ablauftest BASIC im Bau und in den Projekt-Einstellungen: Karte „BASIC“ (Start, Schrittweite, kompakt, Syntaxprüfung), Wirkung auf den Bau
// (Zeilennummern, Warnungen B15x in der Problemliste), Wertebereichsprüfung, Speichern in der Projektdatei.
// Aufruf: node tools/e2e-basic.mjs <Projektkopie examples/c64/basic_native> [Bilder]
//   Kern mit HOME=<leer> VC_TESTBRIDGE=17420 VC_TEST_NO_SPAWN=1 VC_NO_SAVE=1 (Vite: VC_VITE, Brücke: VC_BRIDGE)
import { webkit } from "playwright";
import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const [proj, shots] = process.argv.slice(2);
if (!proj) { console.error("Aufruf: node tools/e2e-basic.mjs <Projekt> [Bilder]"); process.exit(2); }
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
const vcprj = () => JSON.parse(readFileSync(join(proj, "basic_native.vcprj"), "utf8"));
const bauen = async (rel) => {
  const out = join(proj, "build", rel.replace(/\.bas$/, ".classic.txt")); rmSync(out, { force: true });
  await page.evaluate(async (r) => { window.__vc.ui.extra = null; window.__vc.ui.mode = "code"; const t = await window.__vc.call("tree_reload"); window.__vc.ui.project.tree = t.tree; await window.__vc.openFile(r); }, rel); await page.waitForTimeout(300);
  await page.evaluate(() => { void window.__vc.runCommand("bauen"); });
  for (let i = 0; i < 100 && !existsSync(out); i++) await page.waitForTimeout(200);
  await page.waitForTimeout(800);
};

// 1 Karte in den Projekt-Einstellungen
await page.evaluate(() => { void window.__vc.runCommand("nav:projekt:quellen"); }); await page.waitForSelector("[data-testid=basic-card]");
ok(await page.getByTestId("basic-start").inputValue() === "10" && await page.getByTestId("basic-step").inputValue() === "10", "Karte BASIC: Vorgabe Start 10, Schrittweite 10");
ok(await page.getByTestId("basic-speichern").isDisabled(), "Speichern ist ohne Änderung aus");
await snap("basic-1-karte");

// 2 Wertebereich: Schrittweite 0 wird abgelehnt
await page.getByTestId("basic-step").fill("0");
ok(await page.getByTestId("basic-fehler").count() === 1 && await page.getByTestId("basic-speichern").isDisabled(), "Schrittweite 0: Hinweis sichtbar, Speichern aus");

// 3 Speichern: Projektdatei bekommt den Abschnitt basic
await page.getByTestId("basic-start").fill("100"); await page.getByTestId("basic-step").fill("5");
await page.getByTestId("basic-kompakt").locator("button[role=switch]").click();
await page.getByTestId("basic-abgeschaltet").fill("b152, b154");
await page.getByTestId("basic-speichern").click(); await page.waitForTimeout(500);
const b = vcprj().basic;
ok(b && b.start === 100 && b.step === 5 && b.compact === true && JSON.stringify(b.disabled) === JSON.stringify(["B152", "B154"]), "Projektdatei: " + JSON.stringify(b));
ok(vcprj().targets?.length === 1 && vcprj().main === "sterne.bas", "übrige Felder der Projektdatei bleiben erhalten");
ok(await page.getByTestId("basic-speichern").isDisabled(), "nach dem Speichern wieder aus");
await snap("basic-2-gespeichert");

// 4 Wirkung auf den Bau: Nummerierung ab 100 in Schritten von 5
writeFileSync(join(proj, "zahlen.bas"), "PRINT \"A\"\nPRINT \"B\"\n");
await bauen("zahlen.bas");
const l = readFileSync(join(proj, "build/zahlen.classic.txt"), "utf8").toUpperCase();
ok(/100 ?PRINT/.test(l) && /105 ?PRINT/.test(l), "Bau nimmt Start 100 / Schritt 5 aus dem Projekt: " + l.split("\n").slice(0, 2).join(" | "));

// 5 Syntaxprüfung: FOR ohne NEXT erscheint in der Problemliste; abgeschaltete Codes nicht
writeFileSync(join(proj, "pruef.bas"), "FOR I=1 TO 3\nSCORE=SCORE+I\nPRINT SCORE\nEND\n");
await bauen("pruef.bas");
await page.evaluate(() => { void window.__vc.runCommand("schublade"); }); await page.waitForTimeout(300);
await page.locator(".dock [data-tab=problems]").click().catch(() => {}); await page.waitForTimeout(300);
const txt = await page.locator(".problems").innerText().catch(() => "");
ok(/NEXT/i.test(txt), "Problemliste zeigt die Warnung zu FOR ohne NEXT (B151): " + txt.replace(/\s+/g, " ").slice(0, 120));
ok(/B151/.test(txt), "Problemliste zeigt die Fehlernummer (B151)");
ok(((await page.locator(".problems .row", { hasText: "SCORE" }).count())) === 1 && /auch in Zeile/.test(txt), "B150 (SCORE) nur einmal, weitere Zeilen in derselben Meldung");
await snap("basic-3-warnung");

// 5b Erklärtext zum Code: Überfahren zeigt ihn als Tooltip, Klick klappt ihn unter der Zeile auf
const chip = page.locator(".problems [data-testid=problem-code]", { hasText: "B151" }).first();
ok(/FOR|NEXT/.test((await chip.getAttribute("title")) ?? ""), "Code B151 hat den Erklärtext als Tooltip: " + ((await chip.getAttribute("title")) ?? "-").slice(0, 60));
await chip.click(); await page.waitForTimeout(300);
const erkl = await page.locator(".problems [data-testid=problem-erklaerung]").first().innerText().catch(() => "");
ok(/FOR/.test(erkl) && /abschalten/.test(erkl), "Klick auf den Code klappt die Erklärung auf: " + erkl.replace(/\s+/g, " ").slice(0, 100));
await snap("basic-3b-erklaerung");
await chip.click(); await page.waitForTimeout(200);
ok((await page.locator(".problems [data-testid=problem-erklaerung]").count()) === 0, "zweiter Klick klappt die Erklärung wieder zu");

// 6 Prüfung aus: keine Warnung mehr
await page.evaluate(() => { void window.__vc.runCommand("nav:projekt:quellen"); }); await page.waitForSelector("[data-testid=basic-card]");
await page.getByTestId("basic-pruefung").locator("button[role=switch]").click();
await page.getByTestId("basic-speichern").click(); await page.waitForTimeout(500);
ok(vcprj().basic.check === false, "Prüfung ausgeschaltet gespeichert");
await bauen("pruef.bas");
const txt2 = await page.locator(".problems").innerText().catch(() => "");
ok(!/NEXT/i.test(txt2), "ohne Prüfung keine Warnung mehr");

// 6b Karte: 100/5 speichern, dann 10/10 plus „Kompakt“ in einem Zug speichern (Handtest: compact fehlte beim ersten Speichern)
await page.evaluate(() => { void window.__vc.runCommand("nav:projekt:quellen"); }); await page.waitForSelector("[data-testid=basic-card]");
await page.getByTestId("basic-pruefung").locator("button[role=switch]").click();
await page.getByTestId("basic-kompakt").locator("button[role=switch]").click();   // kompakt war an (Schritt 3): jetzt aus
await page.getByTestId("basic-start").fill("100"); await page.getByTestId("basic-step").fill("5");
await page.getByTestId("basic-speichern").click(); await page.waitForTimeout(500);
await page.getByTestId("basic-start").fill("10"); await page.getByTestId("basic-step").fill("10");
await page.getByTestId("basic-kompakt").locator("button[role=switch]").click();
await page.getByTestId("basic-speichern").click(); await page.waitForTimeout(500);
ok(vcprj().basic?.compact === true && vcprj().basic?.start === undefined, "erstes Speichern nach 100/5 → 10/10 + Kompakt schreibt compact: " + JSON.stringify(vcprj().basic));
await page.getByTestId("basic-kompakt").locator("button[role=switch]").click(); await page.getByTestId("basic-speichern").click(); await page.waitForTimeout(500);

// 7 Zurück auf Standard: Abschnitt verschwindet
await page.evaluate(() => { void window.__vc.runCommand("nav:projekt:quellen"); }); await page.waitForSelector("[data-testid=basic-card]");
await page.getByTestId("basic-start").fill("10"); await page.getByTestId("basic-step").fill("10");
await page.getByTestId("basic-abgeschaltet").fill("");
await page.getByTestId("basic-speichern").click(); await page.waitForTimeout(500);
ok(vcprj().basic === undefined, "Standardwerte: Abschnitt basic wird entfernt");

// 8 Englisch
await page.evaluate(async () => { await window.__vc.setLanguage("en"); }); await page.waitForTimeout(400);
await page.evaluate(() => { void window.__vc.runCommand("nav:projekt:quellen"); }); await page.waitForSelector("[data-testid=basic-card]");
ok(/Line numbers/.test(await page.getByTestId("basic-card").innerText()) && /Build compact/.test(await page.getByTestId("basic-card").innerText()), "Karte auf Englisch");
ok(csp.length === 0, "Keine Verstöße gegen die Inhaltsrichtlinie (" + csp.length + ")");
console.log(`${n} Prüfungen, ${fails.length} Fehler`);
await browser.close();
process.exit(fails.length ? 1 : 0);
