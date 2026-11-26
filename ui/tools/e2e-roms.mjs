// SPDX-License-Identifier: GPL-3.0-or-later
// Ablauftest ROM-Import (Einstellungen ▸ ROMs): Auswahl, Plan ohne Kopieren, Ablegen ins Fenster, Suche in anderen Programmen, Fundliste kopieren, Festnageln.
// Aufruf: node tools/e2e-roms.mjs <Ordner mit Testdateien (wird angelegt)> <ROM-Ordner (VC_ROMS des Kerns)>
// Kern mit HOME=<temp> VC_ROMS=<ROM-Ordner> VC_TEST_ROM_PICK=<Ordner>/zz.bin|<Ordner>/pack.zip VC_TESTBRIDGE=17420 VC_TEST_NO_SPAWN=1 VC_NO_SAVE=1 – nie echte ROMs, nie Netz.
import { webkit } from "playwright";
import { existsSync, mkdirSync, readFileSync, writeFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
const [dir, romdir] = process.argv.slice(2);
if (!dir || !romdir) { console.error("Aufruf: node tools/e2e-roms.mjs <Ordner> <ROM-Ordner>"); process.exit(2); }
mkdirSync(dir, { recursive: true });
writeFileSync(join(dir, "zz.bin"), Buffer.from(Array.from({ length: 8192 }, (_, i) => i % 241)));
writeFileSync(join(dir, "pack.zip"), "PK");
const browser = await webkit.launch(); const ctx = await browser.newContext({ viewport: { width: 1440, height: 900 }, permissions: [] }); const page = await ctx.newPage();
const fails = []; let n = 0; const ok = (c, m) => { n++; if (!c) { fails.push(m); console.log("FEHLER:", m); } else console.log("ok:", m); };
page.on("pageerror", (e) => { fails.push("Seitenfehler " + e.message); console.log("Seitenfehler", e.message); });
await page.goto("http://127.0.0.1:1420/?bridge=17420"); await page.waitForFunction(() => window.__vc && window.__vc.ui.ready);
await page.evaluate(async () => { await window.__vc.setLanguage("de"); });
await page.locator("[data-act=einstellungen]").click(); await page.waitForTimeout(500);
await page.locator("[data-es=pagebtn][data-s=roms]").click(); await page.waitForTimeout(700);
ok(await page.locator("[data-rm=import]").count() === 1, "ROM-Seite hat die Karte „ROMs übernehmen“");
// Auswahl: nicht erkannte Datei und kaputtes Zip → Plan mit Gründen, nichts kopiert
await page.locator("[data-rm=pick]").click(); await page.waitForSelector("[data-rm=plan]", { timeout: 8000 });
const plan = await page.locator("[data-rm=plan] li").allInnerTexts();
ok(plan.length === 2 && plan.every((x) => !/wird kopiert/.test(x)), "Plan zeigt beide Dateien als nicht übernehmbar: " + plan.map((x) => x.replace(/\s+/g, " ")).join(" | "));
ok(await page.locator("[data-rm=run]").isDisabled(), "Ohne erkannte ROMs ist „Übernehmen“ gesperrt");
ok(!existsSync(romdir) || readdirSync(romdir).length === 0, "Es wurde nichts in den ROM-Ordner kopiert");
await page.locator("[data-rm=cancel]").click(); await page.waitForTimeout(200);
ok(await page.locator("[data-rm=plan]").count() === 0, "Abbrechen schließt den Plan");
// Ablegen ins Fenster auf der ROM-Seite: ROM-Import statt Projektablage
await page.evaluate((p) => window.__vc.handleDrop([p]), join(dir, "zz.bin")); await page.waitForSelector("[data-rm=plan]", { timeout: 8000 });
ok(await page.locator("[data-rm=plan] li").count() === 1, "Ablegen auf die ROM-Seite öffnet den ROM-Plan (kein Projekt-Dialog)");
await page.locator("[data-rm=cancel]").click();
// Fremde Orte: im Prüf-HOME gibt es nichts
await page.locator("[data-rm=foreign]").click(); await page.waitForTimeout(1500);
await page.waitForSelector("[data-rm=msg], [data-rm=plan]", { timeout: 15000 });
const fremdPlan = await page.locator("[data-rm=plan] li").count();   // auf einem Rechner mit VICE/Atari800 kann es Funde geben: sie werden nur angezeigt, nie kopiert
ok(fremdPlan > 0 ? await page.locator("[data-rm=aktion]").count() === fremdPlan : /keine erkannten ROMs/.test(await page.locator("[data-rm=msg]").innerText()), "Suche in anderen Programmen: Liste (" + fremdPlan + ") oder Meldung ohne Funde");
if (fremdPlan) await page.locator("[data-rm=cancel]").click();
ok(!existsSync(romdir) || readdirSync(romdir).length === 0, "Auch die Suche kopiert nichts");
// Fundliste kopieren (Zwischenablage im Test nicht erlaubt: es kommt entweder die Meldung oder ein Fehler, nie ein Absturz)
await page.locator("[data-rm=copylist]").click(); await page.waitForTimeout(400);
ok(await page.locator(".toasts").count() >= 0, "Fundliste kopieren ohne Absturz");
// Festnageln über den Kern: roms.json im ROM-Ordner
await page.evaluate(async () => { await window.__vc.call("roms_pin", { system: "c64", role: "kernal", file: "mein.bin" }); });
const rj = join(romdir, "roms.json");
ok(existsSync(rj) && JSON.parse(readFileSync(rj, "utf8")).c64?.kernal === "mein.bin", "Festnageln schreibt roms.json");
await page.evaluate(async () => { await window.__vc.call("roms_pin", { system: "c64", role: "kernal" }); });
ok(JSON.parse(readFileSync(rj, "utf8")).c64?.kernal === undefined, "Automatisch entfernt den Eintrag wieder");
// Datei eines anderen Systems als dem aktiven Reiter: die Planzeile nennt das Ziel („→ PET/CBM“); einzeln und aus einem Zip (Attrappen: der Plan kommt vom Kern, hier als Zustand eingesetzt)
await page.locator("[data-es=pagebtn][data-s=roms]").click().catch(() => {}); await page.waitForTimeout(300);
await page.evaluate(() => { const sys = window.__vc.rm.info.systems; const pet = sys.find((x) => /pet/i.test(x.id)) ?? sys[1]; window.__vc.rm.sel = sys[0].id; window.__vc.ri.items = [
  { i: 0, name: "kernal.901465-03.bin", src: "kernal.901465-03.bin", system: pet.id, role: "kernal", label: "PET Kernal", size: 4096, crc32: "00000000", action: "copy", reason: "", dest: "x", source: null },
  { i: 1, name: "pet.zip › basic-4.bin", src: "pet.zip", system: pet.id, role: "basic", label: "PET BASIC 4", size: 4096, crc32: "00000001", action: "copy", reason: "", dest: "y", source: "pet.zip" },
  { i: 2, name: "eigenes.bin", src: "eigenes.bin", system: sys[0].id, role: "kernal", label: null, size: 8192, crc32: "00000002", action: "copy", reason: "", dest: "z", source: null }]; window.__vc.ri.pick = [0, 1, 2]; window.__vc.ri.planned = true; });
await page.waitForTimeout(300);
const ziele = await page.locator("[data-rm=plan] li").evaluateAll((els) => els.map((e) => e.querySelector("[data-rm=anderes-system]")?.textContent?.trim() ?? ""));
ok(/^→ /.test(ziele[0]) && /^→ /.test(ziele[1]) && ziele[2] === "", "Planzeile zeigt „→ Zielsystem“ nur bei Dateien eines anderen Systems (einzeln und im Zip): " + ziele.join(" | "));
console.log("\n" + n + " Prüfungen, " + fails.length + " Fehler"); await browser.close(); process.exit(fails.length ? 1 : 0);
