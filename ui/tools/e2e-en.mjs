// SPDX-License-Identifier: GPL-3.0-or-later
// Ablauftest in englischer Oberfläche (0.3.0 ist zweisprachig): Startseite, Projekt anlegen, Bauen, Sprite öffnen, Sprache zur Laufzeit wechseln, Hilfe öffnen.
// Beschriftungen kommen aus dem Katalog (katalog.mjs, Sprache en), nicht aus festem Text. Aufruf: VC_E2E_LANG=en node tools/e2e-en.mjs <Kopie c64/jumprun> <Kopie c64/sprite_demo> <Ordner für neue Projekte> <Kopie c64/gfx_demo> <Kopie mega65/pal_demo> <Kopie c64/music_demo>
import { webkit } from "playwright";
import { existsSync, mkdirSync } from "node:fs";
import { join } from "node:path";
import { T, R } from "./katalog.mjs";
const [proj, sprites, neuOrdner, gfx, pal, musik] = process.argv.slice(2);
if (!proj || !sprites || !neuOrdner || !gfx || !pal || !musik) { console.error("Aufruf: node tools/e2e-en.mjs <jumprun> <sprite_demo> <Ordner>"); process.exit(2); }
mkdirSync(neuOrdner, { recursive: true });
const browser = await webkit.launch(); const page = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
const fails = []; let n = 0; const ok = (c, m) => { n++; if (!c) { fails.push(m); console.log("FEHLER:", m); } else console.log("ok:", m); };
page.on("pageerror", (e) => { fails.push("Seitenfehler " + e.message); console.log("Seitenfehler", e.message); });
const warte = (ms = 450) => page.waitForTimeout(ms);
const text = () => page.locator("body").innerText();
const DEUTSCH = /Neues Projekt|Projekt öffnen|Beispiel öffnen|Zuletzt geöffnet|Einstellungen|Datei speichern|Bauen und Starten|Wählen …/;
await page.goto("http://127.0.0.1:1420/?bridge=17420"); await page.waitForFunction(() => window.__vc && window.__vc.ui.ready);
await page.evaluate(async () => { await window.__vc.setLanguage("en"); }); await warte(800);
// 1) Startseite
ok(await page.getByRole("button", { name: T("startui-neues-projekt") }).count() >= 1 && await page.getByRole("button", { name: T("startui-projekt-oeffnen") }).count() >= 1, "Startseite englisch: " + T("startui-neues-projekt") + " / " + T("startui-projekt-oeffnen"));
ok(!DEUTSCH.test(await text()), "Startseite: keine deutschen Hauptbeschriftungen");
ok(await page.evaluate(() => document.documentElement.lang) === "en", "Dokumentsprache ist en");
// 2) Projekt anlegen (Dialog englisch)
await page.evaluate(() => { void window.__vc.runCommand("projekt_neu"); }); await page.waitForSelector("dialog[open]"); await warte(500);
const dlg = await page.locator("dialog[open]").innerText();
ok(R("np-zielsystem").test(dlg) && R("np-vorlage").test(dlg), "Neues-Projekt-Dialog englisch: " + T("np-zielsystem") + ", " + T("np-vorlage"));
ok(!/Zielsystem|Vorlage|Dateistruktur/.test(dlg), "Dialog ohne deutsche Beschriftungen");
await page.keyboard.press("Escape"); await warte(300);
// 3) Projekt öffnen und bauen
await page.evaluate(async (p) => { await window.__vc.openProject(p); await window.__vc.openFile("jumprun.asm"); }, proj); await warte(900);
await page.evaluate(() => { void window.__vc.runCommand("bauen"); }); await page.waitForFunction(() => !window.__vc.bs?.running, null, { timeout: 90000 }).catch(() => {}); await warte(2000);
await page.evaluate(() => { window.__vc.ed.dock = "output"; }); await warte(500);
const ausgabe = await text();
ok(/Done: \d+ bytes|\d+ Bytes? in build/i.test(ausgabe) || R("bar-bauen").test(ausgabe), "Bauen: Ausgabe/Statusleiste englisch");
ok(!/^\s*Fertig:|^\s*Warnung \S+:\d+|\d+ zu weite Sprünge/m.test(ausgabe), "Bauprotokoll ohne deutsche Meldungen");
await page.evaluate(() => { window.__vc.ed.dock = null; });
// 4) Sprite öffnen
await page.evaluate(async (p) => { await window.__vc.openProject(p); await window.__vc.openFile("graphics/hero.vcsprite"); }, sprites); await warte(1000);
const sp = await text();
ok(R("sprite-reiter-anim").test(sp), "Sprite-Editor englisch: Reiter „" + T("sprite-reiter-anim") + "“");
ok(!/Sprite-Farbe|Mehrfarbig|Doppelt breit|Hinter dem Hintergrund/.test(sp), "Sprite-Editor: Farbrollen und Schalter nicht deutsch");
// 5) Sprache zur Laufzeit wechseln
await page.evaluate(async () => { await window.__vc.setLanguage("de"); }); await warte(900);
const spDe = await text();
ok(/Bilder|Animationen/.test(spDe) && /Sprite-Farbe/.test(spDe), "Wechsel auf Deutsch ohne Neustart: Sprite-Editor deutsch (inkl. Farbrollen aus dem Kern)");
await page.evaluate(async () => { await window.__vc.setLanguage("en"); }); await warte(900);
ok(!/Sprite-Farbe/.test(await text()) && await page.evaluate(() => document.documentElement.lang) === "en", "Zurück auf Englisch: auch Kern-Texte folgen");
// Startseite und Zuletzt-Liste folgen dem Wechsel
await page.evaluate(async () => { await window.__vc.runCommand("startseite"); }); await warte(900);
await page.evaluate(async () => { await window.__vc.setLanguage("de"); }); await warte(900);
ok(/Neues Projekt/.test(await text()) && /Zuletzt/.test(await text()), "Startseite wechselt zur Laufzeit auf Deutsch");
await page.evaluate(async () => { await window.__vc.setLanguage("en"); }); await warte(900);
ok(R("startui-neues-projekt").test(await text()) && !/vor \d+ (Min|Std)|Beispielprojekte/.test(await text()), "…und zurück: Beispiele und Zuletzt-Liste englisch");
// 6) Hilfe
await page.evaluate(async () => { await window.__vc.runCommand("hilfe_reiter"); }); await page.waitForSelector("[data-hp=themen]", { timeout: 10000 }); await warte(700);
ok(R("hilfe-themen").test(await text()) || (await page.locator("[data-hp=themen] li").count()) > 3, "Hilfe öffnet: Themenliste vorhanden");
// 7) Editoren holen ihre Kern-Texte beim Sprachwechsel sofort neu (nicht erst beim nächsten Öffnen): Ansicht wird ersetzt
const editoren = [["bs", gfx, "graphics/title.vcscreen"], ["kc", gfx, "graphics/level_tiles.vctiles"], ["pal", pal, "graphics/pal65.vcpal"], ["tk", musik, "music/tune.vcsong"], ["sp", sprites, "graphics/hero.vcsprite"]];
for (const [name, dir, datei] of editoren) {
  await page.evaluate(async ([d, f]) => { await window.__vc.setLanguage("en"); await window.__vc.openProject(d); await window.__vc.openFile(f); }, [dir, datei]); await warte(1000);
  const vorher = await page.evaluate((nm) => { const o = window.__vc[nm]; if (!o?.view) return false; o.view.__marke = 1; return true; }, name);
  await page.evaluate(async () => { await window.__vc.setLanguage("de"); }); await warte(1200);
  const nachher = await page.evaluate((nm) => { const v = window.__vc[nm]?.view; return !!v && v.__marke !== 1; }, name);
  ok(vorher && nachher, "Editor „" + name + "“: Ansicht wird beim Sprachwechsel neu vom Kern geholt");
}
console.log("\n" + n + " Prüfungen, " + fails.length + " Fehler"); await browser.close(); process.exit(fails.length ? 1 : 0);
