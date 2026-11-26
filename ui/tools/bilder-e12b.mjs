// SPDX-License-Identifier: GPL-3.0-or-later
// Bilderstrecke E12-Reste (ROM-Import, Emulator portabel installieren, Erstes Projekt, Neues erkannt). Aufruf: node tools/bilder-e12b.mjs <Ausgabe> roms|assistent <Ordner>
// roms: Kern mit VC_ROMS, VC_EMU_DIR, VC_TEST_ROM_PICK, VC_TEST_EMU_PKG (wie e2e-alle.sh); assistent: Kern ohne Einstellungen (HOME leer), Ordner = HOME.
import { webkit } from "playwright";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
const [out, modus, dir] = process.argv.slice(2);
if (!out || !modus) { console.error("Aufruf: node tools/bilder-e12b.mjs <Ausgabe> roms|assistent <Ordner>"); process.exit(2); }
mkdirSync(out, { recursive: true });
const browser = await webkit.launch(); const page = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
const merk = join(out, ".liste.json"); const alt = existsSync(merk) ? JSON.parse(readFileSync(merk, "utf8")) : []; const liste = [];
const w = (ms = 500) => page.waitForTimeout(ms);
const bild = async (name, punkt, soll) => { await w(400); await page.screenshot({ path: join(out, name + ".png") }); liste.push([punkt, name + ".png", soll]); };
await page.goto("http://127.0.0.1:1420/?bridge=17420"); await page.waitForFunction(() => window.__vc && window.__vc.ui.ready);
await page.evaluate(async () => { await window.__vc.setLanguage("de"); window.__vc.setTheme("dark"); }); await w(600);
if (modus === "roms") {
  await page.locator("[data-act=einstellungen]").click(); await w(500); await page.locator("[data-es=pagebtn][data-s=roms]").click(); await w(800);
  await bild("r-01-roms-seite", "R-1", "ROMs: neue Karte „ROMs übernehmen“ mit drei Knöpfen und Hinweis, darunter Stand je System");
  await page.locator("[data-rm=pick]").click(); await page.waitForSelector("[data-rm=plan]"); await bild("r-02-plan-nicht-erkannt", "R-2", "Plan für eine unbekannte Datei und ein kaputtes Zip: Gründe stehen da, „Übernehmen“ ist gesperrt, nichts wird kopiert");
  await page.locator("[data-rm=cancel]").click(); await page.locator("[data-rm=foreign]").click(); await page.waitForSelector("[data-rm=plan], [data-rm=msg]", { timeout: 15000 }); await bild("r-03-fremde", "R-3", "ROMs anderer Programme (z. B. VICE): erkannte Dateien mit Haken, nichts kopiert bis „Übernehmen“");
  await page.locator("[data-rm=cancel]").click().catch(() => {});
  await page.locator("[data-es=pagebtn][data-s=ziele]").click(); await w(700);
  await page.locator("[data-zl=system][data-s=atari8]").click().catch(() => {}); await w(300);
  if (!(await page.locator("[data-zl=emu]").count())) { await page.locator("[data-zl=neue-art]").selectOption({ index: 1 }); await page.locator("[data-zl=emu-hinzu]").click(); await w(500); }
  await page.locator("[data-zl=emu]").first().locator("select.art").selectOption("atari800"); await w(800);
  await bild("r-04-emu-angebot", "R-4", "Ziele: unter dem Emulator „Portabel installieren (Version) …“, geladen wird noch nichts");
  await page.locator("[data-zl=inst-start]").first().click(); await bild("r-05-emu-frage", "R-5", "Rückfrage nennt Version, Größe, Quelle und Zielordner; erst „Jetzt laden“ lädt");
  await page.locator("[data-zl=inst-ja]").click(); await page.waitForSelector("[data-zl=inst-meldung]", { timeout: 20000 }); await bild("r-06-emu-fertig", "R-6", "Nach dem Laden: „Installiert: …“ (hier eine lokale Attrappe, kein Netz)");
} else {
  await w(800);
  const spaeter = page.getByRole("button", { name: /Später fortsetzen/ });
  await page.locator("[data-wz=weiter]").click().catch(() => {});
  await page.evaluate(async () => { await window.__vc.call("wiz_start", { mode: "quick" }); });
  await page.evaluate(() => window.__vc.wz.step = 0);
  // bis zur Zusammenfassung, übernehmen, weiter bis Fertig
  for (let i = 0; i < 12 && !(await page.locator("[data-wz=uebernehmen]").count()); i++) { await page.locator("[data-wz=weiter]").click(); await w(350); }
  await page.locator("[data-wz=uebernehmen]").click(); await page.waitForSelector("[data-wz=angewandt]", { timeout: 20000 });
  await page.locator("[data-wz=weiter]").click(); await w(500);
  await bild("a-01-erstes-projekt", "A-1", "Letzter Schritt: „Erstes Projekt“ mit „Neues Projekt anlegen …“ und „Beispiel öffnen: …“");
  await page.locator("[data-wz=schliessen]").click(); await w(400);
  const wf = join(dir, ".config", "vintagecoder", "wizard.json"); const wj = JSON.parse(readFileSync(wf, "utf8")); wj.known_programs = []; wj.no_news_hint = false; writeFileSync(wf, JSON.stringify(wj));
  await page.goto("http://127.0.0.1:1420/?bridge=17420"); await page.waitForFunction(() => window.__vc && window.__vc.ui.ready); await w(1500);
  await bild("a-02-neues-erkannt", "A-2", "Startseite: Karte „Neues erkannt“ mit „Emulatoren einrichten“, „Als bekannt merken“, „Nicht mehr zeigen“ (setzt voraus, dass auf dem Rechner Emulatoren gefunden werden)");
}
const neu = new Set(liste.map((l) => l[1])); const alle = [...alt.filter((l) => !neu.has(l[1])), ...liste].sort((a, b) => a[1].localeCompare(b[1])); writeFileSync(merk, JSON.stringify(alle));
writeFileSync(join(out, "LIESMICH.md"), "# Bilderstrecke E12-Reste (de, dunkel)\n\nErzeugt mit `node ui/tools/bilder-e12b.mjs` (Playwright/WebKit, Prüfbrücke, echter Kern, eigenes HOME, Attrappen; kein Netz, keine echten ROMs).\n\n| Punkt | Bild | Erwartet |\n|---|---|---|\n" + alle.map((l) => `| ${l[0]} | ${l[1]} | ${l[2]} |`).join("\n") + "\n");
console.log(liste.length + " Bilder"); await browser.close();
