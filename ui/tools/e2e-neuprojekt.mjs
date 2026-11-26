// SPDX-License-Identifier: GPL-3.0-or-later
// Ablauftest Neues Projekt (E1.4): Dialog, Optionen je Zielsystem, Vorschau, Anlegen und Öffnen, Fehler bleiben im Dialog.
// Aufruf: node tools/e2e-neuprojekt.mjs <Ordner für neue Projekte (leer)> [Bilderordner]   (Kern mit VC_TEST_DIR=<derselbe Ordner> VC_TESTBRIDGE=17420 VC_TEST_NO_SPAWN=1 VC_NO_SAVE=1 und HOME=<leer>)
import { webkit } from "playwright";
import { existsSync, mkdirSync, readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";

const [ziel, shots] = process.argv.slice(2);
if (!ziel) { console.error("Aufruf: node tools/e2e-neuprojekt.mjs <Ordner> [Bilder]"); process.exit(2); }
if (shots) mkdirSync(shots, { recursive: true });
const browser = await webkit.launch();
const page = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
const fails = []; let n = 0; const csp = [];
const ok = (c, m) => { n++; if (!c) { fails.push(m); console.log("FEHLER:", m); } else console.log("ok:", m); };
page.on("pageerror", (e) => { fails.push("Seitenfehler " + e.message); console.log("Seitenfehler", e.message); });
page.on("console", (m) => { if (/Content Security Policy|Refused to/i.test(m.text())) csp.push(m.text()); });
const snap = async (name) => { if (shots) await page.screenshot({ path: join(shots, name + ".png") }); };
await page.goto("http://127.0.0.1:1420/?bridge=17420"); await page.waitForFunction(() => window.__vc && window.__vc.ui.ready);
await page.evaluate(async () => { await window.__vc.setLanguage("de"); });
const dlg = page.getByRole("dialog");

// Startseite
ok(await page.getByRole("button", { name: "Neues Projekt" }).count() === 1, "Startseite hat „Neues Projekt“");
await page.getByRole("button", { name: "Neues Projekt" }).click(); await page.waitForSelector("dialog[open]", { timeout: 5000 });
ok(/Neues Projekt erstellen/.test(await dlg.innerText()), "Der Dialog öffnet");
const systeme = await dlg.getByRole("radiogroup", { name: "Zielsystem" }).getByRole("radio").allInnerTexts();
ok(systeme.length >= 5 && systeme.some((s) => /Commodore 64/.test(s)) && systeme.some((s) => /VIC|20/.test(s)), "Zielsysteme sind gelistet: " + systeme.map((s) => s.split("\n")[0]).join(", "));
const karten = () => dlg.getByRole("radiogroup", { name: "Vorlage" }).getByRole("radio");
const k0 = await karten().allInnerTexts();
ok(k0.length >= 1 && /vcasm/.test(k0[0]), "Assembler-Vorlagen für das Zielsystem: " + k0.map((s) => s.split("\n")[0]).join(", "));
ok(await karten().first().getAttribute("aria-checked") === "true", "Die erste Vorlage ist gewählt");
await dlg.getByRole("radio", { name: "BASIC" }).click(); await page.waitForTimeout(300);
const k1 = await karten().allInnerTexts(); ok(k1.length === 0 || /BASIC/.test(k1[0]), "BASIC zeigt BASIC-Vorlagen (" + k1.length + ")");
ok(k1.length === 0 || await dlg.getByText("BASIC-Dialekt").count() > 0, "… und den Dialekt zur Wahl");
await dlg.getByRole("radio", { name: "Assembler" }).click(); await page.waitForTimeout(300);
await snap("neuprojekt-1-dialog");
// Zielsystemwechsel: MEGA65 hat keine Videonorm, VIC-20 hat Speicherausbau
const sysBtn = (txt) => dlg.getByRole("radiogroup", { name: "Zielsystem" }).getByRole("radio", { name: new RegExp(txt) }).first();
if (await sysBtn("MEGA65").count()) { await sysBtn("MEGA65").click(); await page.waitForTimeout(300); ok(await dlg.getByText("Videonorm").count() === 0, "MEGA65: keine Videonorm"); }
if (await sysBtn("VC-20").count()) { await sysBtn("VC-20").click(); await page.waitForTimeout(300); ok(await dlg.getByText("Speicherausbau").count() === 1, "VIC-20: Speicherausbau zur Wahl"); }
await sysBtn("Commodore 64").click(); await page.waitForTimeout(300);
ok(await dlg.getByText("Videonorm").count() === 1, "C64: Videonorm zur Wahl");
// Name prüfen
const name = dlg.getByLabel("Projektname");
await name.fill("a b"); await page.waitForTimeout(150);
ok(await dlg.getByRole("alert").count() >= 1 && await dlg.getByRole("button", { name: "Projekt erstellen und öffnen" }).isDisabled(), "Ungültiger Name: Hinweis sichtbar, Anlegen gesperrt");
await name.fill("probe1"); await page.waitForTimeout(150);
ok(await dlg.getByRole("button", { name: "Projekt erstellen und öffnen" }).isEnabled(), "Gültiger Name: Anlegen frei");
// Speicherort wählen (Prüfbrücke liefert VC_TEST_DIR)
await dlg.getByRole("button", { name: "Wählen …" }).click(); await page.waitForTimeout(300);
ok((await dlg.innerText()).includes(ziel.split("/").pop()), "Speicherort gewählt: " + ziel);
// Vorschau folgt den Optionen
const liste = () => dlg.getByRole("list", { name: "Dateistruktur" }).innerText();
ok(/probe1\.vcprj|probe1\//.test(await liste()) && !/README\.md/.test(await liste()), "Vorschau zeigt das Projekt ohne README");
await dlg.getByLabel("README.md mit Bauhinweisen anlegen").check(); await dlg.getByLabel(/Git-Repository anlegen/).check(); await page.waitForTimeout(400);
ok(/README\.md/.test(await liste()) && /\.gitignore/.test(await liste()), "Vorschau zeigt README.md und .gitignore, sobald gewählt");
await snap("neuprojekt-2-ausgefuellt");
// Anlegen
await dlg.getByRole("button", { name: "Projekt erstellen und öffnen" }).click(); await page.waitForFunction(() => window.__vc.ui.project && window.__vc.ui.project.name, null, { timeout: 15000 }).catch(() => {});
await page.waitForTimeout(600);
const dir = join(ziel, "probe1");
ok(existsSync(join(dir, "probe1.vcprj")) && existsSync(join(dir, "README.md")) && existsSync(join(dir, ".gitignore")), "Projektordner mit .vcprj, README.md und .gitignore angelegt: " + readdirSync(dir).join(", "));
ok((await page.evaluate(() => window.__vc.ui.project?.name)) === "probe1" && (await page.locator("dialog[open]").count()) === 0, "Das Projekt ist geöffnet, der Dialog zu");
ok((await page.getByRole("treeitem").count()) >= 2, "Der Projektbaum zeigt die Dateien");
await snap("neuprojekt-3-geoeffnet");
// Zweites Anlegen am selben Ort: Fehler bleibt im Dialog
await page.evaluate(() => { void window.__vc.runCommand("projekt_neu"); }); await page.waitForSelector("dialog[open]");
await dlg.getByLabel("Projektname").fill("probe1"); await dlg.getByRole("button", { name: "Wählen …" }).click(); await page.waitForTimeout(300);
await dlg.getByRole("button", { name: "Projekt erstellen und öffnen" }).click(); await page.waitForTimeout(800);
const alert = await dlg.getByRole("alert").allInnerTexts();
ok(await page.locator("dialog[open]").count() === 1 && alert.some((a) => a.length > 10), "Existiert der Ordner schon, bleibt der Dialog offen und zeigt den Fehler: " + alert.join(" | ").slice(0, 100));
await snap("neuprojekt-4-fehler");
await dlg.getByRole("button", { name: "Abbrechen" }).click(); await page.waitForTimeout(300);
ok(await page.locator("dialog[open]").count() === 0, "Abbrechen schließt den Dialog");
// Esc schließt ebenfalls und der Dialog lässt sich wieder öffnen
await page.evaluate(() => { void window.__vc.runCommand("projekt_neu"); }); await page.waitForSelector("dialog[open]"); await page.keyboard.press("Escape"); await page.waitForTimeout(300);
await page.evaluate(() => { void window.__vc.runCommand("projekt_neu"); }); await page.waitForSelector("dialog[open]", { timeout: 4000 }).then(() => ok(true, "Nach Esc lässt sich der Dialog erneut öffnen"), () => ok(false, "Dialog öffnet nach Esc nicht mehr"));
await page.keyboard.press("Escape");
ok(csp.length === 0, "Keine Meldungen der Inhaltsrichtlinie: " + csp.slice(0, 2).join(" | "));
console.log(`\n${n} Prüfungen, ${fails.length} Fehler`);
await browser.close();
process.exit(fails.length ? 1 : 0);
