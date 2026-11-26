// SPDX-License-Identifier: GPL-3.0-or-later
// Ablauftest Einstellungen (E12.1): Playwright (WebKit) gegen die Prüfbrücke des echten Kerns (Einstellungsdatei im eigenen HOME).
// Aufruf: node tools/e2e-einstellungen.mjs <Ordner für Export/Import-Dateien> [Bilderordner]  (App mit VC_TEST_KONFIG_OUT=<Ordner>/out.json VC_TEST_KONFIG_IN=<Ordner>/in.json)
import { webkit } from "playwright";
import { mkdirSync, readFileSync, writeFileSync, existsSync } from "node:fs";
import { join } from "node:path";

if (process.platform === "win32" && !process.env.VC_E2E_WIN) { console.log("übersprungen unter Windows\n0 Prüfungen, 0 Fehler"); process.exit(0); }
const [dir, shots] = process.argv.slice(2);
if (!dir) { console.error("Aufruf: node tools/e2e-einstellungen.mjs <Ordner> [Bilder]"); process.exit(2); }
mkdirSync(dir, { recursive: true }); if (shots) mkdirSync(shots, { recursive: true });
writeFileSync(join(dir, "in.json"), JSON.stringify({ projects_dir: "/tmp/vc-aus-import", machines: [{ id: "m1", kind: "ultimate64", host: "203.0.113.5", password: "" }] }));
const browser = await webkit.launch();
const page = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
const fails = []; let n = 0;
const ok = (c, m) => { n++; if (!c) { fails.push(m); console.log("FEHLER:", m); } else console.log("ok:", m); };
page.on("pageerror", (e) => { fails.push("Seitenfehler " + e.message); console.log("Seitenfehler", e.message); });
const snap = async (name) => { if (shots) await page.screenshot({ path: join(shots, name + ".png") }); };
const warte = (ms = 400) => page.waitForTimeout(ms);
await page.goto("http://127.0.0.1:1420/?bridge=17420"); await page.waitForFunction(() => window.__vc && window.__vc.ui.ready);
await page.evaluate(async () => { await window.__vc.setLanguage("de"); window.__vc.setTheme("dark"); }); await warte();

// 1) Öffnen über Kopfzeile und Befehl
await page.locator("[data-act=einstellungen]").click(); await warte(600);
ok(await page.locator("[data-es=page-allgemein]").isVisible(), "Das Zahnrad öffnet die Einstellungen (Seite Allgemein)");
ok((await page.locator("[data-es=pagebtn]").count()) === 9, "Neun Seiten: Allgemein, Ziele, ROMs, Aussehen, Editor, Toolchain, Tastenkürzel, Sicherung, Info");
await snap("einst-1-allgemein");
// 2) Allgemein: Entwurf, Speichern, Verwerfen
ok(await page.locator("[data-es=save]").isDisabled(), "Speichern ist gesperrt, solange nichts geändert ist");
await page.getByRole("radio", { name: "Täglich" }).click(); await warte(200); ok(!(await page.locator("[data-es=save]").isDisabled()), "Eine Änderung gibt Speichern frei");
await page.locator("[data-es=discard]").click(); await warte(200); ok(await page.locator("[data-es=save]").isDisabled() && (await page.getByRole("radio", { name: "Täglich" }).getAttribute("aria-checked")) !== "true", "Verwerfen stellt den gespeicherten Stand wieder her");
await page.getByRole("radio", { name: "Wöchentlich" }).click(); await page.locator("[data-es=upd-url]").fill("https://example.invalid/liste.json"); await page.locator("[data-es=save]").click(); await warte(500);
let c = await page.evaluate(() => window.__vc.cfg.saved); ok(c.updates?.check === "weekly" && c.updates?.manifest_url === "https://example.invalid/liste.json", "Speichern schreibt Update-Wahl und Adresse in die Einstellungsdatei");
await page.locator("[data-es=page-allgemein]").evaluate(() => 0); 
// Neu laden zeigt den gespeicherten Stand
await page.evaluate(() => window.__vc.ui.extra = null); await page.locator("[data-act=einstellungen]").click(); await warte(500); ok((await page.getByRole("radio", { name: "Wöchentlich" }).getAttribute("aria-checked")) === "true", "Nach erneutem Öffnen steht der gespeicherte Stand da");
// Debug-Protokoll speichert sofort
await page.getByRole("switch", { name: /Debug-Protokoll/ }).click(); await warte(500); c = await page.evaluate(() => window.__vc.cfg.saved); ok(c.debug_log === true && (await page.locator("[data-es=save]").isDisabled()), "Der Schalter Debug-Protokoll speichert sofort");
await page.getByRole("switch", { name: /Debug-Protokoll/ }).click(); await warte(300);
// Sprache wirkt sofort
ok(await page.evaluate(() => document.documentElement.lang) === "de", "Deutsch: das lang-Attribut der Seite ist „de“ (VoiceOver wählt die Stimme danach)");
await page.getByRole("radio", { name: "English" }).click(); await warte(700); ok(/Settings/.test(await page.locator("body").innerText()), "Sprache English wirkt sofort"); ok(await page.evaluate(() => document.documentElement.lang) === "en", "English: das lang-Attribut der Seite folgt auf „en“"); await page.getByRole("radio", { name: "Deutsch" }).click(); await warte(700);
// 3) Aussehen
await page.locator("[data-es=pagebtn][data-s=aussehen]").click(); await warte(200);
await page.getByRole("radio", { name: "Hoher Kontrast" }).click(); await warte(200); ok((await page.evaluate(() => document.documentElement.getAttribute("data-theme"))) === "contrast", "Farbschema Hoher Kontrast setzt das Thema");
await snap("einst-2-kontrast"); await page.getByRole("radio", { name: "Dunkel" }).click(); await warte(200);
await page.locator("[data-es=zoom-plus]").click(); await page.locator("[data-es=zoom-plus]").click(); await warte(200); ok(/120 %/.test(await page.locator("[data-es=zoom]").innerText()), "Zoom 120 % nach zweimal größer");
await page.locator("[data-es=zoom-null]").click(); await warte(200); ok(/100 %/.test(await page.locator("[data-es=zoom]").innerText()), "100 % stellt den Zoom zurück");
await page.evaluate(() => window.__vc.runCommand("zoom_plus")); await warte(200); ok(/110 %/.test(await page.locator("[data-es=zoom]").innerText()), "Der Befehl Zoom größer wirkt ebenso"); await page.locator("[data-es=darstellung-zurueck]").click(); await warte(300);
ok(/100 %/.test(await page.locator("[data-es=zoom]").innerText()) && (await page.evaluate(() => window.__vc.ui.theme)) === "system", "Darstellung zurücksetzen: Zoom 100 %, Farbschema System");
// 4) Editor
await page.locator("[data-es=pagebtn][data-s=editor]").click(); await warte(200); const s0 = await page.locator("[data-es=schrift]").innerText();
await page.locator("[data-es=schrift-plus]").click(); await warte(150); ok((await page.locator("[data-es=schrift]").innerText()) !== s0, "Editor-Schrift größer: " + s0 + " → " + (await page.locator("[data-es=schrift]").innerText())); await page.locator("[data-es=schrift-minus]").click();
const w0 = await page.evaluate(() => window.__vc.ed.wrap); await page.getByRole("switch", { name: "Zeilenumbruch" }).click(); await warte(200); ok((await page.evaluate(() => window.__vc.ed.wrap)) !== w0, "Zeilenumbruch umschalten"); await page.getByRole("switch", { name: "Zeilenumbruch" }).click();
// 5) Toolchain
await page.locator("[data-es=pagebtn][data-s=toolchain]").click(); await warte(300); ok((await page.locator("[data-es=dialekte] li").count()) >= 2, "Toolchain: BASIC-Dialekt je Zielsystem aufgelistet (" + (await page.locator("[data-es=dialekte] li").count()) + ")");
await page.locator("[data-args=asm] [data-args=toggle]").click(); await warte(500); ok((await page.locator("[data-args=asm] [data-args=gruppe]").count()) >= 1 || (await page.locator("[data-args=asm] [data-args=free]").count()) === 1, "Parametereditor des Assemblers zeigt Katalog und freies Feld"); await page.locator("[data-args=asm] [data-args=free]").fill("-I extra/inc"); await warte(500); ok(/-I/.test(await page.locator("[data-args=asm] [data-args=vorschau]").innerText()), "Die Vorschau zeigt die Kommandozeile mit -I"); await page.locator("[data-es=save]").click(); await warte(500); c = await page.evaluate(() => window.__vc.cfg.saved); ok(c.assembler?.args === "-I extra/inc", "Weitere Assembler-Schalter werden gespeichert"); await snap("einst-3-toolchain");
// 5b) Ziele und Hardware
await page.locator("[data-es=pagebtn][data-s=ziele]").click(); await warte(600);
ok((await page.locator("[data-zl=system]").count()) >= 3, "Ziele: die Systeme aus der Vorgabe stehen links (" + (await page.locator("[data-zl=system]").count()) + ")");
await page.locator("[data-zl=system][data-s=c64]").click(); await warte(200); await page.locator("[data-zl=system-weg]").click(); await warte(400);
ok((await page.locator("[data-zl=system][data-s=c64]").count()) === 0, "System entfernen: C64 verschwindet aus der Liste");
await page.locator("[data-zl=neues-system]").selectOption("c64"); await page.locator("[data-zl=system-hinzu]").click(); await warte(500);
ok((await page.locator("[data-zl=system][data-s=c64]").count()) === 1, "System C64 wieder hinzufügen");
const n0 = await page.locator("[data-zl=emu]").count();
await page.locator("[data-zl=neue-art]").selectOption({ index: 1 }); await page.locator("[data-zl=emu-hinzu]").click(); await warte(600);
ok((await page.locator("[data-zl=emu]").count()) === n0 + 1 && (await page.locator("[data-zl=standard]:checked").count()) === 1, "Emulator hinzufügen (" + n0 + " → " + (n0 + 1) + "), genau einer ist Standard");
ok((await page.locator("[data-zl=anwesend]").count()) >= 1, "Der Emulator zeigt, ob sein Programm gefunden wurde: " + (await page.locator("[data-zl=anwesend]").first().innerText()));
await page.locator("[data-zl=pfad]").last().fill("/gibt/es/nicht/x64sc"); await page.locator("[data-zl=pfad]").last().blur(); await warte(600); ok(/✗/.test(await page.locator("[data-zl=anwesend]").last().innerText()), "Ein falscher Pfad wird als nicht gefunden gemeldet");
await page.locator("[data-zl=testen]").last().click(); await warte(500); ok((await page.locator("[data-zl=probe]").count()) === 1, "Testen meldet ein Ergebnis (im Testlauf ohne Start)");
await page.locator("[data-zl=hw-hinzu][data-h=ultimate64]").click(); await warte(500); ok((await page.locator("[data-zl=maschine]").count()) === 1, "Ultimate anbinden: Karte erscheint");
await page.locator("[data-zl=host]").fill("192.168.1.50"); await page.locator("[data-zl=passwort]").fill("geheim"); ok((await page.locator("[data-zl=passwort]").getAttribute("type")) === "password", "Passwort ist verdeckt"); await page.locator("[data-zl=pw-zeigen]").click(); ok((await page.locator("[data-zl=passwort]").getAttribute("type")) === "text", "Zeigen macht das Passwort sichtbar");
await page.locator("[data-zl=hw-testen]").click(); await warte(500); ok((await page.locator("[data-zl=hw-probe]").count()) === 1, "Verbindung testen meldet ein Ergebnis");
await page.locator("[data-zl=start-maschine]").check(); await page.locator("[data-es=save]").click(); await warte(600); c = await page.evaluate(() => window.__vc.cfg.saved);
const tz = c.targets.find((x) => x.system === "c64"); ok(tz && tz.emulators.length === n0 + 1 && tz.start_on === "m1" && c.machines?.[0]?.host === "192.168.1.50" && c.machines[0].password === "geheim", "Speichern: Ziel, Emulator, Startziel Maschine, Host und Passwort stehen in der Datei");
await snap("einst-6-ziele");
await page.locator("[data-zl=maschine-weg]").click(); await warte(400); ok((await page.locator("[data-zl=maschine]").count()) === 0, "Maschine entfernen");
// 5c) ROMs
await page.locator("[data-es=pagebtn][data-s=roms]").click(); await warte(900); ok((await page.locator("[data-rm=system]").count()) >= 3 && (await page.locator("[data-rm=rolle]").count()) >= 1, "ROMs: Systeme und Rollen mit Status (" + (await page.locator("[data-rm=system]").count()) + " Systeme)");
ok((await page.locator("[data-rm=orte] li").count()) >= 1, "ROMs: durchsuchte Orte sind aufgelistet");
await page.locator("[data-rm=system][data-s=c64]").click(); await warte(200); ok((await page.locator("[data-rm=status]").first().innerText()).length > 0, "Status der ROM-Rolle: " + (await page.locator("[data-rm=status]").first().innerText()));
await page.getByRole("switch", { name: /ROMs anderer Programme/ }).click(); await warte(300); ok((await page.evaluate(() => window.__vc.cfg.draft.roms.use_foreign)) === true, "Fremde ROMs mitbenutzen schaltet den Entwurf"); await page.getByRole("switch", { name: /ROMs anderer Programme/ }).click(); await snap("einst-7-roms");
// 7a) Info und Update (Versionsliste aus einer lokalen Datei, VC_UPDATE_URL)
await page.locator("[data-es=pagebtn][data-s=info]").click(); await warte(500); ok(/\d+\.\d+/.test(await page.locator("[data-info=version]").innerText()), "Info: Version " + (await page.locator("[data-info=version]").innerText()));
{ const ab = await page.locator("[data-es=about]").innerText();
  ok(/Copyright © Christian Drüeke/.test(ab) && /GPL-3\.0-or-later/.test(ab) && /LICENSE/.test(ab) && /keine Gewährleistung/.test(ab), "Über-Karte: Copyright, Lizenzname, Hinweis auf LICENSE und Gewährleistungssatz");
  const dk = await page.locator("[data-es=danksagungen]").innerText();
  ok(/Raster Music Tracker/.test(dk) && /Radek Štěrba/.test(dk) && /Vin Samuel und Peter Dell/.test(dk), "Danksagungen nennen den Raster Music Tracker mit Urhebern");
  ok(/GoatTracker 2/.test(dk) && /Lasse Öörni/.test(dk) && /High Voltage SID Collection/.test(dk) && /SpritePad/.test(dk) && /Subchrist Software/.test(dk), "Danksagungen nennen GoatTracker 2, das SID-Format mit der HVSC und SpritePad");
  const rmt = page.locator("[data-credit=rmt] [data-credit-link]").first();
  ok((await rmt.getAttribute("title")) === "https://github.com/raster-atari-org/RASTER-Music-Tracker" && !/raster\.atari\.org/.test(dk), "RMT-Link zeigt auf das Repository raster-atari-org (nicht raster.atari.org)");
  const url0 = page.url(); await rmt.click(); await warte(300);
  ok(page.url() === url0 && !(await page.locator(".toasts").innerText().catch(() => "")).trim(), "Klick auf einen Danksagungs-Link öffnet nur über den Kern (im Test gesperrt), das Fenster bleibt und keine Fehlermeldung");
  ok(await page.locator("[data-credit-link]").count() >= 8 && (await page.locator("[data-credit-link]").evaluateAll((els) => els.every((e) => e.getAttribute("title").startsWith("https://")))), "Alle Links der Danksagungen sind https-Adressen"); }
await page.locator("[data-info=notices]").click(); await warte(200); ok((await page.locator("[data-info=notices-text]").innerText()).length > 100, "Hinweise zu Drittanbietern lassen sich aufklappen");
await page.locator("[data-info=check]").click(); await warte(1500); ok(await page.locator("[data-info=fresh]").isVisible() && /9\.9\.9/.test(await page.locator("[data-info=fresh]").innerText()), "Jetzt prüfen: neue Version 9.9.9 mit Änderungen angeboten");
ok(/Update 9\.9\.9/.test(await page.locator("footer.status").innerText()), "Hinweis „Update 9.9.9 verfügbar“ in der Statusleiste");
await page.locator("[data-info=later]").click(); await warte(500); ok((await page.locator("[data-status=update]").count()) === 0, "Später erinnern blendet den Hinweis aus");
await page.locator("[data-info=skip]").click(); await warte(500); c = await page.evaluate(() => window.__vc.inf.upd); ok(!c.badge, "Version überspringen: kein Hinweis mehr");
await snap("einst-8-info");
await page.locator("[data-info=feedback]").click(); await page.waitForSelector("[data-fb=dialog]"); ok(/Version/i.test(await page.locator("[data-fb=text]").inputValue()), "Feedback: Text mit technischen Angaben vorbereitet"); await page.locator("[data-fb=mail]").click(); await warte(300); ok(/Mailprogramm/.test(await page.locator("[data-fb=msg]").innerText()), "Mail vorbereiten meldet den Schritt (im Testlauf ohne Start)"); await page.getByRole("button", { name: "Abbrechen" }).first().click(); await warte(200);
// 6) Kürzel
await page.locator("[data-es=pagebtn][data-s=kuerzel]").click(); await warte(300); const alle = await page.locator("[data-es=kuerzel] li").count(); ok(alle > 20, "Tastenkürzel: " + alle + " Befehle mit Kürzel");
await page.locator("[data-es=kuerzel-suche]").fill("zoom"); await warte(200); const z = await page.locator("[data-es=kuerzel] li").count(); ok(z >= 1 && z < alle, "Suche nach „zoom“ filtert (" + z + ")");
await page.locator("[data-es=kuerzel-suche]").fill(""); await snap("einst-4-kuerzel");
// Einzeltasten, Tastenbeschriftung der Klaviatur, Startbild (Oberflächenwahl, gemerkt wie Farbschema und Zoom)
const sw = (name) => page.getByRole("switch", { name });
await sw("Einzeltasten der Editoren").click(); await warte(150);
ok(await page.evaluate(() => window.__vc.ui.singleKeys) === false, "Einzeltasten der Editoren lassen sich ausschalten");
await sw("Computertasten auf der Klaviatur anzeigen").click(); await warte(150);
ok(await page.evaluate(() => window.__vc.ui.keyHints) === false, "Tastenbeschriftung der Klaviatur lässt sich ausschalten");
await sw("Einzeltasten der Editoren").click(); await sw("Computertasten auf der Klaviatur anzeigen").click(); await warte(150);
ok(await page.evaluate(() => window.__vc.ui.singleKeys && window.__vc.ui.keyHints), "… und wieder einschalten");
await page.locator("[data-es=pagebtn][data-s=allgemein]").click(); await warte(200);
await sw("Startbild beim Programmstart zeigen").click(); await warte(150);
ok(await page.evaluate(() => window.__vc.ui.splash) === false && await page.evaluate(() => localStorage.getItem("splash")) === "0", "Startbild ausschalten wird gemerkt");
await sw("Startbild beim Programmstart zeigen").click(); await warte(150);

// 7) Sicherung
await page.locator("[data-es=pagebtn][data-s=sicherung]").click(); await warte(200);
await page.locator("[data-es=export]").click(); await warte(600); ok(existsSync(join(dir, "out.json")) && /weekly/.test(readFileSync(join(dir, "out.json"), "utf8")), "Exportieren schreibt die Einstellungsdatei");
await page.locator("[data-es=import]").click(); await page.waitForSelector("dialog[open] [data-es=import-ja]"); ok(/1 Maschinen|1 Maschine/.test(await page.locator("dialog[open]").innerText()), "Importieren fragt mit Zusammenfassung vorher"); await snap("einst-5-import");
await page.locator("[data-es=import-ja]").click(); await warte(600); c = await page.evaluate(() => window.__vc.cfg.saved); ok(c.projects_dir === "/tmp/vc-aus-import" && c.machines?.length === 1, "Übernehmen ersetzt die Einstellungen");
await page.evaluate(() => window.__vc.setTheme("dark")); await page.locator("[data-es=reset]").click(); await page.waitForSelector("dialog[open] [data-es=reset-ja]"); await page.locator("[data-es=reset-ja]").click(); await warte(600); c = await page.evaluate(() => window.__vc.cfg.saved); ok(!c.projects_dir && !(c.machines?.length), "Zurücksetzen stellt die Vorgabe her (nach Rückfrage)");
ok(await page.evaluate(() => window.__vc.ui.theme) === "system", "Zurücksetzen stellt auch das Farbschema (hell/dunkel) auf „System“");
console.log(`\n${n} Prüfungen, ${fails.length} Fehler`);
await browser.close();
process.exit(fails.length ? 1 : 0);
