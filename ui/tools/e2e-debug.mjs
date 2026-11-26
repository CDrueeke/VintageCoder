// SPDX-License-Identifier: GPL-3.0-or-later
// Ablauftest Debug (E6): Playwright (WebKit) gegen die Prüfbrücke des echten Kerns. Statt eines Emulators läuft die Attrappe (VC_TEST_FAKE_DEBUG=1).
// Aufruf: node tools/e2e-debug.mjs <Kopie von examples/c64/jumprun> [Bilderordner]
import { webkit } from "playwright";
import { mkdirSync } from "node:fs";
import { join } from "node:path";

if (process.platform === "win32" && !process.env.VC_E2E_WIN) { console.log("übersprungen unter Windows\n0 Prüfungen, 0 Fehler"); process.exit(0); }
const [proj, shots] = process.argv.slice(2);
if (!proj) { console.error("Aufruf: node tools/e2e-debug.mjs <c64 jumprun> [Bilder]"); process.exit(2); }
if (shots) mkdirSync(shots, { recursive: true });
const browser = await webkit.launch();
const page = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
const fails = []; let n = 0;
const ok = (c, m) => { n++; if (!c) { fails.push(m); console.log("FEHLER:", m); } else console.log("ok:", m); };
page.on("pageerror", (e) => { fails.push("Seitenfehler " + e.message); console.log("Seitenfehler", e.message); });
const snap = async (name) => { if (shots) await page.screenshot({ path: join(shots, name + ".png") }); };
await page.goto("http://127.0.0.1:1420/?bridge=17420"); await page.waitForFunction(() => window.__vc && window.__vc.ui.ready);
await page.evaluate(async (p) => { await window.__vc.setLanguage("de"); await window.__vc.openProject(p); }, proj); await page.waitForTimeout(900);
const st = () => page.evaluate(() => window.__vc.dbg.s); const warte = (ms = 450) => page.waitForTimeout(ms);
const pcText = async () => (await page.locator("[data-dbg=reg-pc]").innerText()).trim();

// 1) Leerer Zustand und Start
await page.evaluate(() => window.__vc.runCommand("nav:debug")); await warte();
ok(await page.locator("[data-dbg=empty]").isVisible(), "Debug ohne Sitzung zeigt den Hinweis und den Startknopf");
await snap("debug-1-leer");
await page.locator("[data-dbg=start]").click(); await page.waitForSelector("[data-dbg=register]", { timeout: 60000 }); await warte(); let s = await st();
ok(s.active && !s.running && s.haveRegs && s.backend === "Attrappe", "Debuggen baut und startet die Sitzung, das Programm ist am Anfang angehalten");
ok(/Angehalten bei \$[0-9A-F]{4}/.test(await page.locator("[data-dbg=status]").innerText()), "Status nennt den Haltepunkt: " + (await page.locator("[data-dbg=status]").innerText()));
ok((await page.locator("[data-dbg=ueber]").innerText()).trim().length > 0 && (await page.locator("[data-dbg=weiter]").innerText()).trim().length > 0 && (await page.locator("[data-dbg=hinein]").innerText()).trim().length > 0 && (await page.locator("[data-dbg=ende]").innerText()).trim().length > 0, "Schaltflächen Weiter, Schritt über/hinein und Stopp tragen Text");
const pc0 = await pcText(); ok(pc0 === "$" + s.regs.pc.toString(16).toUpperCase().padStart(4, "0"), "Register PC = " + pc0);
const zeilen = await page.locator("[data-dbg=row]").count(); ok(zeilen >= 10 && (await page.locator("[data-dbg=row].cur").count()) === 1, "Disassembler zeigt " + zeilen + " Zeilen, die aktuelle ist hervorgehoben");
ok((await page.locator("[data-dbg=flags] .fl").count()) === 8, "Flags: acht Bits");
await snap("debug-2-halt");

// 2) Einzelschritt, Änderungen markiert
await page.locator("[data-dbg=hinein]").click(); await warte(); const pc1 = await pcText(); ok(pc1 !== pc0, "Schritt hinein: PC " + pc0 + " → " + pc1);
ok((await page.locator(".rg.chg").count()) >= 1, "Geänderte Register sind markiert");
await page.locator("[data-dbg=ueber]").click(); await warte(); ok((await pcText()) !== pc1, "Schritt über bewegt den PC");

// 3) Haltepunkt im Disassembler, Weiter, Treffer
s = await st(); const pcJetzt = s.regs.pc; const kand = s.disasm.filter((d) => d.addr > pcJetzt && d.line && /\.asm$/.test(d.line.file)); const adrZ = (kand[2] ?? kand[0] ?? s.disasm[s.disasm.length - 1]).addr;
await page.locator(`[data-dbg=row][data-a='${adrZ}'] [data-dbg=bp]`).click(); await warte(); s = await st();
ok(s.points.some((p) => p.kind === "exec" && p.start === adrZ), "Klick auf den Rand setzt einen Haltepunkt bei $" + adrZ.toString(16));
ok((await page.locator("[data-dbg=punkte] .pt").count()) >= 1, "Die Haltepunkt-Liste zeigt ihn");
await page.locator("[data-dbg=weiter]").click(); await warte(1200); s = await st();
ok(!s.running && s.regs.pc === adrZ, "Weiter: das Programm hält am Haltepunkt ($" + s.regs.pc.toString(16) + ")");
await snap("debug-3-haltepunkt");

// 4) Bedingung und Fehler, Punkt löschen
await page.locator("[data-dbg=adr]").fill("$0900"); await page.locator("[data-dbg=bed]").fill("a == 5"); await page.locator("[data-dbg=punkt-neu]").click(); await warte(); s = await st();
ok(s.points.some((p) => p.start === 0x900 && p.cond === "a == 5"), "Haltepunkt mit Bedingung");
await page.locator("[data-dbg=adr]").fill("xyz"); await page.locator("[data-dbg=punkt-neu]").click(); await warte(150); ok((await page.locator(".form [role=alert]").count()) === 1, "Ungültige Adresse: Hinweis mit Beispielen");
const np = s.points.length; await page.locator("[data-dbg=punkt-weg]").first().click(); await warte(); ok((await st()).points.length === np - 1, "Punkt löschen");

// 5) Beobachten
await page.locator("[data-dbg=beob]").fill("a"); await page.locator("[data-dbg=beob-neu]").click(); await warte(); ok((await page.locator("[data-dbg=beob-wert]").count()) === 1, "Beobachten: Register a zeigt einen Wert: " + (await page.locator("[data-dbg=beob-wert]").innerText().catch(() => "")));
await page.locator("[data-dbg=beob]").fill("d020"); await page.locator("[data-dbg=beob-neu]").click(); await warte(); ok(/Meinst du \$d020/.test(await page.locator("[data-dbg=beob-fehler]").innerText().catch(() => "")), "Beobachten: „d020“ liefert „Meinst du $d020?“");
await page.locator("[data-dbg=beob-weg]").last().click(); await warte();
await page.locator("[data-dbg=beob]").fill("$0801"); await page.locator("[data-dbg=beob-neu]").click(); await warte(); ok((await page.locator("[data-dbg=beob-wert]").count()) === 2, "Beobachten: eine Adresse zeigt ihr Byte");

// 6) Speicher
await page.locator("[data-dbg=mem]").fill("0801"); await page.locator("[data-dbg=mem-los]").click(); await warte(); s = await st(); ok(s.mem.addr === 0x801 && (await page.locator("[data-dbg=speicher] .mz").count()) === 16, "Speicher ab $0801: 16 Zeilen zu 16 Bytes");
await page.locator("[data-dbg=mem]").fill("zzz"); await page.locator("[data-dbg=mem-los]").click(); await warte(200); ok(/keine Adresse/i.test(await page.locator("body").innerText()), "Ungültige Speicheradresse: verständliche Meldung");

// 7) Momentaufnahme
await page.locator("[data-dbg=erkennen]").click(); await warte(700); s = await st(); ok(!!s.detected, "Momentaufnahme: Erkennen liefert " + (s.detected?.error ? "eine Meldung" : "ein Ergebnis"));

// 8) Bild
await page.locator("[data-dbg=bild-neu]").click(); await warte(900); ok((await page.locator("[data-dbg=bild] img").count()) === 1, "Bild des Emulators wird angezeigt");
await snap("debug-4-bild");

// 9) Quelltextzeile im Editor
s = await st(); if (s.ort) { await page.evaluate(() => window.__vc.runCommand("nav:code")); await warte(); await page.evaluate((f) => window.__vc.openFile(f), s.ort.file); await warte(700); ok((await page.locator(".cm-debugline").count()) === 1, "Im Editor ist die Ausführungszeile markiert (" + s.ort.file + ":" + s.ort.line + ")"); await snap("debug-5-editor"); await page.evaluate(() => window.__vc.runCommand("nav:debug")); await warte(); } else ok(true, "(kein Zeilenbezug in dieser Adresse)");

// 10) Schritt hinaus (kann dauern: der Kern wartet bis zum Rücksprung), danach beenden
await page.locator("[data-dbg=raus]").click(); await warte(300);
await page.locator("[data-dbg=ende]").click(); await warte(600); ok(!(await st())?.active && await page.locator("[data-dbg=empty]").isVisible() && (await page.locator(".cm-debugline").count()) === 0, "Beenden: Hinweis wieder da, Ausführungszeile im Editor entfernt");
// 11) Chip-Fenster, PC in der Statusleiste, Maschine-Panel, Test-Sitzung
await page.evaluate(() => window.__vc.runCommand("nav:debug")); await warte();
await page.locator("[data-dbg=start]").click(); await page.waitForSelector("[data-dbg=register]", { timeout: 60000 }); await warte(700);
ok(await page.locator("[data-dbg=chip-an]").count() >= 2, "Chip-Fenster: Chips des Zielsystems stehen zur Wahl (" + await page.locator("[data-dbg=chip-an]").count() + ")");
await page.locator("[data-dbg=chip-an]").first().check(); await warte(1200);
ok(await page.locator("[data-dbg=chip]").count() === 1 && (await page.locator("[data-dbg=chip-wert]").allInnerTexts()).some((x) => /^\$[0-9A-F]{2}$/.test(x)), "Gewählter Chip zeigt seine Register mit gelesenen Werten");
await snap("debug-11-chips"); await page.locator("[data-dbg=chip-an]").first().uncheck(); await warte(300);
ok(await page.locator("[data-dbg=chip]").count() === 0, "Abwählen blendet das Fenster aus");
ok(/angehalten bei \$[0-9A-F]{4}/i.test(await page.locator("[data-status=debug-pc]").innerText()), "Statusleiste nennt den PC der Sitzung: " + await page.locator("[data-status=debug-pc]").innerText());
await page.evaluate(async () => { await window.__vc.runCommand("nav:code"); window.__vc.ui.panel = true; }); await warte(600);
ok(await page.locator("[data-mp=regs]").isVisible(), "Maschine-Panel zeigt Register und Flags der Sitzung");
await snap("debug-12-panel");
if (await page.locator("[data-mp=bild-holen]").count()) { await page.locator("[data-mp=bild-holen]").click(); await warte(900); ok(await page.locator("[data-mp=bild] img").count() === 1, "Panel: „Bild aktualisieren“ holt ein Einzelbild"); }
await page.evaluate(async () => { window.__vc.ui.panel = false; await window.__vc.runCommand("nav:debug"); }); await warte(500);
await page.locator("[data-dbg=ende]").click(); await warte(600);
await page.evaluate(async () => { await window.__vc.debugTest("jumprun.asm", "beispiel"); }); await warte(1200);
await snap("debug-13-test-sitzung"); s = await st(); ok(s.active && s.test === true && await page.locator("[data-dbg=test-sitzung]").isVisible(), "Test-Sitzung: angehalten am Anfang, als Test-Sitzung gekennzeichnet");
await page.locator("[data-dbg=ende]").click(); await warte(500); ok(!(await st())?.active, "Test-Sitzung lässt sich beenden");
console.log(`\n${n} Prüfungen, ${fails.length} Fehler`);
await browser.close();
process.exit(fails.length ? 1 : 0);
