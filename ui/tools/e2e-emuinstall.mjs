// SPDX-License-Identifier: GPL-3.0-or-later
// Ablauftest „Emulator portabel installieren“ (Einstellungen ▸ Ziele und Hardware) gegen eine lokale Attrappe (file://, nie echtes Netz).
// Aufruf: node tools/e2e-emuinstall.mjs <Emulator-Ordner (VC_EMU_DIR des Kerns)> <Ordner der Attrappe>
// Kern mit HOME=<temp> VC_EMU_DIR=<Emulator-Ordner> VC_TEST_EMU_PKG="atari800|9.9.9|file://<Attrappe>/fake.zip|<sha256>|zip|atari800" (Vorbereitung: siehe e2e-alle.sh)
import { webkit } from "playwright";
import { existsSync } from "node:fs";
import { join } from "node:path";
const [emudir, fake] = process.argv.slice(2);
if (!emudir || !fake) { console.error("Aufruf: node tools/e2e-emuinstall.mjs <Emulator-Ordner> <Attrappe>"); process.exit(2); }
const browser = await webkit.launch(); const page = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
const fails = []; let n = 0; const ok = (c, m) => { n++; if (!c) { fails.push(m); console.log("FEHLER:", m); } else console.log("ok:", m); };
page.on("pageerror", (e) => { fails.push("Seitenfehler " + e.message); console.log("Seitenfehler", e.message); });
await page.goto("http://127.0.0.1:1420/?bridge=17420"); await page.waitForFunction(() => window.__vc && window.__vc.ui.ready);
await page.evaluate(async () => { await window.__vc.setLanguage("de"); });
await page.locator("[data-act=einstellungen]").click(); await page.waitForTimeout(500);
await page.locator("[data-es=pagebtn][data-s=ziele]").click(); await page.waitForTimeout(700);
await page.locator("[data-zl=system][data-s=atari8]").click().catch(() => {}); await page.waitForTimeout(300);
if (!(await page.locator("[data-zl=emu]").count())) { await page.locator("[data-zl=neue-art]").selectOption({ index: 1 }); await page.locator("[data-zl=emu-hinzu]").click(); await page.waitForTimeout(500); }
const emuAtari = page.locator("[data-zl=emu]").filter({ has: page.locator("select.art") });
// Art atari800 wählen, damit die Karte die Attrappe kennt
await emuAtari.first().locator("select.art").selectOption("atari800"); await page.waitForTimeout(800);
await page.locator("[data-zl=inst-start]").first().waitFor({ timeout: 8000 });
ok(await page.locator("[data-zl=inst-start]").first().innerText().then((x) => /9\.9\.9/.test(x)), "„Portabel installieren (9.9.9)“ wird angeboten");
ok(!existsSync(emudir) || !existsSync(join(emudir, "atari800")), "Vor dem Klick ist nichts geladen");
await page.locator("[data-zl=inst-start]").first().click(); await page.waitForTimeout(300);
const frage = await page.locator("[data-zl=inst-frage]").innerText();
ok(/9\.9\.9/.test(frage) && /Jetzt laden/.test(await page.locator("[data-zl=inst-ja]").innerText()), "Rückfrage nennt Version, Quelle und Ziel: " + frage.replace(/\s+/g, " ").slice(0, 140));
await page.locator("[data-zl=inst-nein]").click(); await page.waitForTimeout(200);
ok(await page.locator("[data-zl=inst-frage]").count() === 0 && !existsSync(join(emudir, "atari800")), "Abbrechen lädt nichts");
await page.locator("[data-zl=inst-start]").first().click(); await page.locator("[data-zl=inst-ja]").click();
await page.waitForSelector("[data-zl=inst-meldung]", { timeout: 20000 });
ok(/Installiert/.test(await page.locator("[data-zl=inst-meldung]").first().innerText()), "Nach „Jetzt laden“: Meldung „Installiert“");
ok(existsSync(join(emudir, "atari800", "9.9.9", "atari800")), "Das Programm liegt im eigenen Emulator-Ordner");
await page.waitForTimeout(500);
const rest = await page.locator("[data-zl=inst-start]").allInnerTexts();
ok(!rest.some((x) => /9\.9\.9/.test(x)), "Danach wird dieselbe Version nicht noch einmal angeboten");
console.log("\n" + n + " Prüfungen, " + fails.length + " Fehler"); await browser.close(); process.exit(fails.length ? 1 : 0);
