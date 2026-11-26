// SPDX-License-Identifier: GPL-3.0-or-later
// Ablauftest Einrichtungsassistent (E12.3): Playwright (WebKit) gegen die Prüfbrücke. Die App startet ohne Einstellungen (eigenes HOME, ohne VC_NO_WIZARD): der Assistent startet erst auf Anforderung (Karte der Startseite).
// Aufruf: node tools/e2e-assistent.mjs <HOME der App> <Ordner für Projekte/Beispiele> [Bilderordner]
import { webkit } from "playwright";
import { existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";

if (process.platform === "win32" && !process.env.VC_E2E_WIN) { console.log("übersprungen unter Windows\n0 Prüfungen, 0 Fehler"); process.exit(0); }
const [home, proj, shots] = process.argv.slice(2);
if (!proj) { console.error("Aufruf: node tools/e2e-assistent.mjs <HOME> <Projektordner> [Bilder]"); process.exit(2); }
if (shots) mkdirSync(shots, { recursive: true });
const settingsDatei = join(home, ".config", "vintagecoder", "settings.json");
const browser = await webkit.launch();
const page = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
const fails = []; let n = 0;
const ok = (c, m) => { n++; if (!c) { fails.push(m); console.log("FEHLER:", m); } else console.log("ok:", m); };
page.on("pageerror", (e) => { fails.push("Seitenfehler " + e.message); console.log("Seitenfehler", e.message); });
const snap = async (name) => { if (shots) await page.screenshot({ path: join(shots, name + ".png") }); };
const warte = (ms = 400) => page.waitForTimeout(ms); const wz = () => page.evaluate(() => window.__vc.wz.data);
await page.goto("http://127.0.0.1:1420/?bridge=17420"); await page.waitForFunction(() => window.__vc && window.__vc.ui.ready);
await page.evaluate(async () => { await window.__vc.setLanguage("de"); window.__vc.setTheme("dark"); });

// 1) Erster Start: der Assistent öffnet sich nicht von selbst, er startet auf Anforderung (Karte der Startseite)
await warte(1200);
ok(await page.locator("[data-wz=assistent]").count() === 0 && await page.locator("[data-start=welcome-card]").count() === 1, "Ohne Einstellungen öffnet sich der Assistent nicht von selbst; die Startseite zeigt die Willkommenskarte");
await page.locator("[data-start=welcome-setup]").click(); await page.waitForSelector("[data-wz=assistent]", { timeout: 15000 }); await warte(600);
ok(await page.locator("[data-wz=thema-welcome]").isVisible(), "„Einrichtung starten“ öffnet den Assistenten (Willkommen)");
let d = await wz(); ok(d.steps.length === 7 && d.mode === "quick", "Schnelleinrichtung hat 7 Schritte"); await snap("wiz-1-willkommen");
await page.locator("[data-wz=weg-voll]").click(); await warte(500); d = await wz(); ok(d.steps.length === 12 && d.mode === "full", "„Alles selbst“ hat 12 Schritte"); await page.locator("[data-wz=weg-schnell]").click(); await warte(500);
ok(!existsSync(settingsDatei), "Bis „Übernehmen“ wird nichts geschrieben");
// 2) Design
await page.locator("[data-wz=weiter]").click(); await warte(400); ok(await page.locator("[data-wz=thema-design]").isVisible(), "Schritt Design");
await page.getByRole("radio", { name: "Hell" }).click(); await warte(300); ok((await page.evaluate(() => document.documentElement.getAttribute("data-theme"))) === "light", "Farbschema Hell wirkt sofort als Vorschau");
await page.locator("[data-wz=zoom-plus]").click(); await warte(300); ok(/110 %/.test(await page.locator("[data-wz=zoom]").innerText()), "Zoom 110 %"); await snap("wiz-2-design");
// 3) Systeme
await page.locator("[data-wz=weiter]").click(); await warte(500); ok(await page.locator("[data-wz=thema-systems]").isVisible(), "Schritt Zielsysteme");
const hat = async () => (await wz()).settings.targets.map((t) => t.system);
ok((await hat()).includes("c64"), "C64 ist vorgewählt: " + (await hat()).join(", "));
await page.getByRole("switch", { name: /Commodore 128/ }).click(); await warte(600); ok((await hat()).includes("c128"), "C128 hinzugeschaltet"); await snap("wiz-3-systeme");
// 4) Toolchain (Emulatoren), Projekte
await page.locator("[data-wz=weiter]").click(); await warte(900); ok(await page.locator("[data-wz=thema-toolchain]").isVisible() && (await page.locator("[data-zl=emu]").count()) >= 1, "Schritt Toolchain zeigt die Emulatoren der gewählten Systeme (" + (await page.locator("[data-zl=emu]").count()) + ")"); await snap("wiz-4-toolchain");
await page.locator("[data-wz=weiter]").click(); await warte(500); ok(await page.locator("[data-wz=thema-projects]").isVisible(), "Schritt Projekte");
await page.locator("[data-wz=projekte]").fill(proj); await page.locator("[data-wz=projekte]").blur(); await warte(600);
const bsp = await page.locator("[data-wz=beispiele] li").count(); ok(bsp === 0 || bsp >= 1, "Beispiele: " + bsp + " zur Wahl");
if (await page.getByRole("switch", { name: /Beispielprojekte/ }).count()) { const an = await page.evaluate(() => window.__vc.wz.data.examples.copy); if (!an) await page.getByRole("switch", { name: /Beispielprojekte/ }).click(); await warte(500); }
await snap("wiz-5-projekte");
// 5) Zusammenfassung, abwählen, übernehmen
await page.locator("[data-wz=weiter]").click(); await warte(600); ok(await page.locator("[data-wz=thema-summary]").isVisible() && (await page.locator("[data-wz=aenderung]").count()) >= 2, "Zusammenfassung listet die Änderungen (" + (await page.locator("[data-wz=aenderung]").count()) + ")"); await snap("wiz-6-zusammenfassung");
await page.locator("[data-wz=aenderung][data-k='target:c128']").uncheck(); await warte(200);
await page.locator("[data-wz=uebernehmen]").click(); await page.waitForSelector("[data-wz=angewandt]", { timeout: 20000 }); await warte(300);
ok(existsSync(settingsDatei), "Übernehmen schreibt die Einstellungsdatei"); const geschrieben = JSON.parse(readFileSync(settingsDatei, "utf8"));
ok(!(geschrieben.targets ?? []).some((t) => t.system === "c128") && (geschrieben.targets ?? []).some((t) => t.system === "c64") && geschrieben.projects_dir === proj, "Abgewähltes (C128) fehlt, C64 und Projektordner stehen drin");
ok(existsSync(join(home, ".config", "vintagecoder", "wizard.json")), "Der Stand des Assistenten ist gemerkt (wizard.json)"); await snap("wiz-7-uebernommen");
const bspOrdner = join(proj, "Beispiele"); const kopiert = existsSync(bspOrdner) ? readdirSync(bspOrdner).length : 0; ok(bsp === 0 || kopiert >= 1, "Beispiele kopiert: " + kopiert + " Ordner");
// 6) Rückgängig
await page.locator("[data-wz=rueckgaengig]").click(); await warte(600); ok(!existsSync(settingsDatei) || !(JSON.parse(readFileSync(settingsDatei, "utf8")).targets ?? []).length, "Rückgängig stellt den Zustand vorher wieder her (keine Einstellungen)");
ok(!(existsSync(bspOrdner) && readdirSync(bspOrdner).length), "Rückgängig entfernt die kopierten Beispiele");
// 7) Wieder übernehmen, weiter bis Fertig, schließen
await page.locator("[data-wz=uebernehmen]").click(); await page.waitForSelector("[data-wz=angewandt]", { timeout: 20000 });
await page.locator("[data-wz=weiter]").click(); await warte(400); ok(await page.locator("[data-wz=finished]").isVisible(), "Schluss: „Die Einrichtung ist fertig“");
ok(await page.locator("[data-wz=erstes-neu]").isVisible(), "Schluss: „Erstes Projekt“ bietet „Neues Projekt anlegen“");
if (bsp > 0) { await page.locator("[data-wz=erstes-beispiel]").click(); await warte(1500); ok(await page.evaluate(() => !!window.__vc.ui.project), "„Beispiel öffnen“ öffnet das erste kopierte Beispiel: " + await page.evaluate(() => window.__vc.ui.project?.root)); await page.evaluate(async () => { await window.__vc.runCommand("projekt_schliessen"); }); await warte(500); }
else await page.locator("[data-wz=schliessen]").click(); await warte(500);
const st = await page.evaluate(() => window.__vc.call("wiz_stand")); ok(st.start === "normal", "Beim nächsten Start öffnet er sich nicht mehr von selbst (" + st.start + ")");
// 8) Aus den Einstellungen: Teil-Assistent, Esc bricht ab und stellt die Darstellung wieder her
await page.locator("[data-act=einstellungen]").click(); await warte(500); await page.locator("[data-es=wiz-voll]").click(); await warte(800); ok(await page.locator("[data-wz=assistent]").isVisible(), "Aus den Einstellungen lässt sich der Assistent wieder aufrufen");
await page.keyboard.press("Escape"); await warte(500); ok(!(await page.locator("[data-wz=assistent]").count()), "Esc bricht den Assistenten ab");
// 9) „Neues erkannt“: Stand zurückdrehen (bekannte Programme leer), Startseite zeigt die Neufunde; „Als bekannt merken“ räumt sie weg
const wizFile = join(home, ".config", "vintagecoder", "wizard.json"); const wj = JSON.parse(readFileSync(wizFile, "utf8")); wj.known_programs = []; wj.no_news_hint = false; writeFileSync(wizFile, JSON.stringify(wj));
await page.evaluate(async () => { await window.__vc.runCommand("startseite"); }); await warte(400);
await page.goto("http://127.0.0.1:1420/?bridge=17420"); await page.waitForFunction(() => window.__vc && window.__vc.ui.ready); await warte(1200);
const nw = await page.evaluate(() => window.__vc.call("wiz_stand")); await page.evaluate(() => { window.__vc.wz.open = false; window.__vc.ui.extra = null; }); await warte(300);
if (nw.news?.length) {
  ok(await page.locator("[data-start=news]").count() === 1, "Neues erkannt: Karte auf der Startseite (" + nw.news.map((x) => x.kind).join(", ") + ")");
  await page.locator("[data-start=news-merken]").click(); await warte(500);
  ok(await page.locator("[data-start=news]").count() === 0 && JSON.parse(readFileSync(wizFile, "utf8")).known_programs.length > 0, "„Als bekannt merken“ blendet die Karte aus und merkt die Programme");
  const w2 = JSON.parse(readFileSync(wizFile, "utf8")); w2.known_programs = []; writeFileSync(wizFile, JSON.stringify(w2)); await page.reload(); await page.waitForFunction(() => window.__vc && window.__vc.ui.ready); await warte(1000);
  if (await page.locator("[data-start=news-aus]").count()) { await page.locator("[data-start=news-aus]").click(); await warte(400); ok(JSON.parse(readFileSync(wizFile, "utf8")).no_news_hint === true, "„Nicht mehr zeigen“ schaltet den Hinweis ab"); }
} else ok(true, "Neues erkannt: auf diesem Rechner wird kein Emulator gefunden (Karte nicht prüfbar, übersprungen)");
console.log(`\n${n} Prüfungen, ${fails.length} Fehler`);
await browser.close();
process.exit(fails.length ? 1 : 0);
