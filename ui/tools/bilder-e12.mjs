// SPDX-License-Identifier: GPL-3.0-or-later
// Bilderstrecke für die Abnahme (E12.1 Einstellungen): Playwright (WebKit), Prüfbrücke. Aufruf: node tools/bilder-e12.mjs <Ordner für Testdateien> <Ausgabeordner>  (App mit VC_TEST_KONFIG_OUT/IN wie bei e2e-einstellungen)
import { webkit } from "playwright";
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
const [dir, out] = process.argv.slice(2);
if (!out) { console.error("Aufruf: node tools/bilder-e12.mjs <Ordner> <Ausgabe>"); process.exit(2); }
mkdirSync(out, { recursive: true }); mkdirSync(dir, { recursive: true });
writeFileSync(join(dir, "in.json"), JSON.stringify({ projects_dir: "/tmp/vc-aus-import", machines: [{ id: "m1", kind: "ultimate64", host: "203.0.113.5", password: "" }] }));
const browser = await webkit.launch(); const page = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
const liste = []; const bild = async (name, punkt, soll) => { await page.waitForTimeout(500); await page.screenshot({ path: join(out, name + ".png") }); liste.push([punkt, name + ".png", soll]); };
const V = (js, a) => page.evaluate(js, a); const w = (ms = 400) => page.waitForTimeout(ms); const seite = (s) => page.locator(`[data-es=pagebtn][data-s=${s}]`).click();
await page.goto("http://127.0.0.1:1420/?bridge=17420"); await page.waitForFunction(() => window.__vc && window.__vc.ui.ready);
await V(async () => { await window.__vc.setLanguage("de"); window.__vc.setTheme("dark"); }); await w();
await page.locator("[data-act=einstellungen]").click(); await w(700);
await bild("e12-01-allgemein-dunkel-breit", "E12-1/2", "Einstellungen ▸ Allgemein: Sprache, Updates, Hilfe, Projektordner, Fehlersuche, Einstellungsdatei; Speichern erst nach einer Änderung");
await page.getByRole("radio", { name: "Täglich" }).click(); await w(200);
await bild("e12-02-entwurf", "E12-3", "Eine Änderung: „Ungespeicherte Änderungen“, Speichern und Verwerfen frei");
await page.locator("[data-es=discard]").click();
await seite("ziele"); await w(900); await bild("e12-02b-ziele", "E12-11/12", "Ziele und Hardware: links die Zielsysteme, rechts Startziel, Emulatoren (Pfad, Suchen, Testen, Standard) und Maschinen");
await page.locator("[data-zl=hw-hinzu][data-h=ultimate64]").first().scrollIntoViewIfNeeded().catch(() => {}); await page.locator("[data-zl=system][data-s=mega65]").click(); await w(500); await bild("e12-02c-ziele-mega65", "E12-13", "MEGA65: Xemu und echte Hardware (USB oder Ethernet; Ethernet mit Hinweis „nur gegen Attrappen getestet“)");
await seite("roms"); await w(1200); await bild("e12-02d-roms", "E12-14", "ROMs: Stand je System und Rolle (erkannt, Ersatz, fehlt) mit Prüfsummen, durchsuchte Orte, ROM-Ordner öffnen, fremde ROMs mitbenutzen");
await seite("aussehen"); await w(200); await bild("e12-03-aussehen", "E12-4", "Aussehen: Farbschema (System, Dunkel, Hell, Hoher Kontrast), Zoom, Darstellung zurücksetzen");
await page.getByRole("radio", { name: "Hoher Kontrast" }).click(); await w(300); await bild("e12-04-hoher-kontrast", "E12-4", "Hoher Kontrast: schwarzer Grund, weiße Schrift, gelber Akzent, kräftige Linien");
await page.getByRole("radio", { name: "Dunkel" }).click(); await page.locator("[data-es=zoom-plus]").click(); await page.locator("[data-es=zoom-plus]").click(); await w(300); await bild("e12-05-zoom-120", "E12-5", "Zoom 120 %: die ganze Oberfläche ist größer"); await page.locator("[data-es=zoom-null]").click();
await seite("editor"); await w(200); await bild("e12-06-editor", "E12-6", "Editor: Schriftgröße, Zeilenumbruch, Formatieren beim Verlassen");
await seite("toolchain"); await w(300); await bild("e12-07-toolchain", "E12-7", "Toolchain: eingebauter Assembler, weitere Schalter, Include-Suche, BASIC-Dialekt je Zielsystem");
await seite("kuerzel"); await w(300); await bild("e12-08-kuerzel", "E12-8", "Tastenkürzel: Suche und Liste aller Befehle mit Kürzel");
await seite("sicherung"); await w(200); await page.locator("[data-es=import]").click(); await page.waitForSelector("dialog[open] [data-es=import-ja]"); await bild("e12-09-import-frage", "E12-9", "Konfiguration importieren: Rückfrage mit Zusammenfassung"); await page.getByRole("button", { name: "Abbrechen" }).first().click(); await w(200);
await page.locator("[data-es=reset]").click(); await page.waitForSelector("dialog[open] [data-es=reset-ja]"); await bild("e12-10-reset-frage", "E12-9", "Zurücksetzen: Rückfrage, nicht rückgängig zu machen"); await page.getByRole("button", { name: "Abbrechen" }).first().click(); await w(200);
await seite("info"); await w(500); await page.locator("[data-info=check]").click(); await w(1500); await bild("e12-10b-info-update", "E12-18", "Info und Update: Version, Lizenz, Prüfen; eine neue Version mit Änderungen, Später erinnern, Überspringen, Änderungen ansehen (Versionsliste aus einer lokalen Testdatei)");
await page.locator("[data-info=feedback]").click(); await page.waitForSelector("[data-fb=dialog]"); await bild("e12-10c-feedback", "E12-19", "Feedback: vorbereitete Mail mit technischen Angaben, Kopieren, Mail vorbereiten"); await page.getByRole("button", { name: "Abbrechen" }).first().click(); await w(200);
await seite("allgemein"); await page.setViewportSize({ width: 640, height: 900 }); await bild("e12-11-schmal-640", "E12-10", "Schmal (640 px): Seitenliste über dem Inhalt"); await page.setViewportSize({ width: 1440, height: 900 });
await V(() => window.__vc.setTheme("light")); await bild("e12-12-hell", "E12-10", "Helles Farbschema"); await V(async () => { window.__vc.setTheme("dark"); await window.__vc.setLanguage("en"); }); await bild("e12-13-englisch", "E12-10", "Englisch (maschinell, nicht abgenommen)"); await V(async () => { await window.__vc.setLanguage("de"); });
writeFileSync(join(out, "LIESMICH.md"), "# Bilderstrecke E12.1 Einstellungen (de, dunkel, wenn nicht anders genannt)\n\nErzeugt mit `node ui/tools/bilder-e12.mjs` (Playwright/WebKit, Prüfbrücke, echter Kern, Einstellungsdatei im eigenen Testordner).\n\n| Handprüfpunkt | Bild | Was zu sehen sein muss |\n|---|---|---|\n" + liste.map(([p, f, s]) => `| ${p} | ${f} | ${s} |`).join("\n") + "\n\nNur von Hand zu prüfen: Dateidialoge (Export, Import, Ordner wählen), „Im Ordner zeigen“, Zoom mit Tastatur, VoiceOver.\n");
console.log(liste.length + " Bilder"); await browser.close();
