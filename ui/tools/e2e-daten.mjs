// SPDX-License-Identifier: GPL-3.0-or-later
// Ablauftest Daten (E11): Hex-Ansicht und Speicherkarte, Playwright (WebKit) gegen die Prüfbrücke des echten Kerns.
// Aufruf: node tools/e2e-daten.mjs <Kopie von examples/c64/jumprun (bekommt Testdateien)> [Bilderordner]
import { webkit } from "playwright";
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";

if (process.platform === "win32" && !process.env.VC_E2E_WIN) { console.log("übersprungen unter Windows\n0 Prüfungen, 0 Fehler"); process.exit(0); }
const [proj, shots] = process.argv.slice(2);
if (!proj) { console.error("Aufruf: node tools/e2e-daten.mjs <c64 jumprun> [Bilder]"); process.exit(2); }
if (shots) mkdirSync(shots, { recursive: true });
// Testdateien: 5000 Bytes (zwei Seiten), darin ein Muster zum Suchen; ein PRG mit Ladeadresse $0801
const roh = Buffer.alloc(5000); for (let i = 0; i < roh.length; i++) roh[i] = (i * 7) % 251; roh[6] = 0xA9; roh[7] = 0x00; roh.write("HELLO", 4500, "latin1");
writeFileSync(join(proj, "test.bin"), roh); writeFileSync(join(proj, "x.prg"), Buffer.from([0x01, 0x08, 0x0b, 0x08, 0x0a, 0x00, 0x9e, 0x32, 0x30, 0x36, 0x31, 0x00, 0x00, 0x00, 0x60]));
mkdirSync(join(proj, "build", "c64"), { recursive: true }); writeFileSync(join(proj, "build", "c64", "gebaut.prg"), Buffer.from([0x01, 0x08, 0x00, 0x00, 0x60]));   // gebautes Programm: steht ebenfalls in der Liste
const browser = await webkit.launch();
const page = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
const fails = []; let n = 0;
const ok = (c, m) => { n++; if (!c) { fails.push(m); console.log("FEHLER:", m); } else console.log("ok:", m); };
page.on("pageerror", (e) => { fails.push("Seitenfehler " + e.message); console.log("Seitenfehler", e.message); });
const snap = async (name) => { if (shots) await page.screenshot({ path: join(shots, name + ".png") }); };
const warte = (ms = 400) => page.waitForTimeout(ms); const hx = () => page.evaluate(() => window.__vc.hx);
await page.goto("http://127.0.0.1:1420/?bridge=17420"); await page.waitForFunction(() => window.__vc && window.__vc.ui.ready);
await page.evaluate(async (p) => { await window.__vc.setLanguage("de"); await window.__vc.openProject(p); }, proj); await page.waitForTimeout(900);
const byte = (o) => page.locator(`[data-hx=byte][data-o='${o}']`);

// 1) Hex: Liste und Seite
await page.evaluate(() => window.__vc.runCommand("nav:daten:hex")); await warte(700);
const namen = await page.locator("[data-hx=file]").allInnerTexts(); ok(namen.some((x) => /test\.bin/.test(x)) && namen.some((x) => /x\.prg/.test(x)) && namen.some((x) => /gebaut\.prg/.test(x)), "Binärdateien des Projekts und gebaute Dateien sind aufgelistet: " + namen.map((x) => x.split("\n")[0]).join(", "));
await page.locator("[data-hx=file]", { hasText: "test.bin" }).click(); await warte(); let s = await hx();
ok(s.page.pages === 2 && s.page.rows.length === 256 && (await page.locator("[data-hx=byte]").count()) === 4096, "test.bin: zwei Seiten, 256 Zeilen zu 16 Byte");
ok((await page.locator("[data-hx=seite]").innerText()).trim() === "1 / 2", "Seitenanzeige 1 / 2");
ok(/Nur lesen/.test(await page.locator("body").innerText()), "„Nur lesen“ ist gekennzeichnet");
// 2) Auswahl und Deutung
await byte(6).click(); await warte(150); ok(/169/.test(await page.locator("[data-hx=deut-byte]").innerText()) && /\$A9/.test(await page.locator("[data-hx=deut-byte]").innerText()), "Byte anklicken: Deutung 169 · $A9: " + (await page.locator("[data-hx=deut-byte]").innerText()));
ok(/\$00A9/.test(await page.locator("[data-hx=deut-wort]").innerText()), "Wort (klein vorn) $00A9");
await byte(9).click({ modifiers: ["Shift"] }); await warte(150); ok(/\$0006–\$0009/.test(await page.locator("[data-hx=auswahl]").innerText()), "Umschalt+Klick erweitert die Auswahl: " + (await page.locator("[data-hx=auswahl]").innerText()).replace(/\n/g, " "));
await byte(9).focus(); await page.keyboard.press("ArrowRight"); await warte(150); s = await hx(); ok(s.sel.a === 10 && s.sel.b === 10, "Pfeil nach rechts wählt das nächste Byte");
await page.keyboard.press("ArrowDown"); await warte(100); s = await hx(); ok(s.sel.a === 26, "Pfeil nach unten: eine Zeile weiter (16 Byte)");
// 3) Breite
await page.getByRole("radio", { name: "8 Byte" }).click(); await warte(); s = await hx(); ok(s.page.width === 8 && s.page.rows.length === 512, "8 Byte je Zeile: 512 Zeilen");
await page.getByRole("radio", { name: "16 Byte" }).click(); await warte();
// 4) Suche
await page.locator("[data-hx=suche]").fill("$A9 00"); await page.locator("[data-hx=suchen]").click(); await warte(); s = await hx(); ok(s.sel && s.sel.a === 6 && s.sel.b === 7, "Suche nach Bytes „$A9 00“ trifft Versatz 6");
await page.locator("[data-hx=suche]").fill("HELLO"); await page.locator("[data-hx=suchen]").click(); await warte(); s = await hx(); ok(s.sel && s.sel.a === 4500 && s.page.page === 1, "Suche nach Text „HELLO“ springt auf Seite 2 zum Versatz 4500");
ok(/Gefunden/.test(await page.locator("[data-hx=status]").innerText()), "Status meldet den Fund");
await page.locator("[data-hx=suche]").fill("gibtsnicht"); await page.locator("[data-hx=suchen]").click(); await warte(); ok(/Nichts gefunden/.test(await page.locator("[data-hx=status]").innerText()), "Kein Treffer: „Nichts gefunden“");
await snap("daten-1-hex");
// 5) Seiten
await page.locator("[data-hx=anfang]").click(); await warte(); ok((await page.locator("[data-hx=seite]").innerText()).trim() === "1 / 2", "Zum Anfang");
await page.locator("[data-hx=weiter]").click(); await warte(); ok((await page.locator("[data-hx=seite]").innerText()).trim() === "2 / 2" && (await page.locator("[data-hx=weiter]").isDisabled()), "Nächste Seite; am Ende gesperrt");
await page.locator("[data-hx=zurueck]").click(); await warte(); ok((await page.locator("[data-hx=seite]").innerText()).trim() === "1 / 2", "Vorige Seite");
// 6) Binärdatei aus dem Baum öffnet in der Hex-Ansicht, PRG zeigt die Ladeadresse
await page.evaluate(() => window.__vc.openFile("x.prg")); await warte(800); s = await hx(); ok(s.rel === "x.prg" && /Ladeadresse \$0801/.test(await page.locator("[data-hx=lade]").innerText()), "x.prg öffnet in der Hex-Ansicht mit Ladeadresse $0801");
ok(/sehr|ganze Datei/.test(await page.locator("[data-hx=umfang]").innerText()), "Umfang: ganze Datei geladen");

// 7) Speicherkarte: ohne Bau, dann gebaut
await page.evaluate(() => window.__vc.runCommand("nav:daten:speicher")); await warte(600);
const gebaut = await page.evaluate(() => window.__vc.spk.info?.built); 
if (!gebaut) { ok(await page.locator("[data-spk=empty]").isVisible(), "Speicherkarte ohne Bau: Hinweis „Noch nichts gebaut“"); await page.evaluate(() => { void window.__vc.runCommand("bauen"); }); await page.waitForFunction(() => /gebaut/.test(document.querySelector("footer.status")?.innerText ?? ""), null, { timeout: 120000 }); await warte(1200); }
ok((await page.locator("[data-spk=seite]").count()) === 256, "Speicherkarte: 256 Seiten");
s = await page.evaluate(() => window.__vc.spk.info); ok(s.built && s.program > 0 && /\$/.test(await page.locator("[data-spk=adresse]").innerText()), "Programm belegt " + s.program + " Byte; gewählte Seite " + await page.locator("[data-spk=adresse]").innerText());
ok((await page.locator("[data-spk=pruefungen] li").count()) >= 2, "Prüfungen sind aufgelistet");
await page.locator("[data-spk=seite][data-p='0']").click(); await warte(500); ok(/\$0000–\$00FF/.test(await page.locator("[data-spk=adresse]").innerText()) && /Zeropage/i.test(await page.locator("[data-spk=art]").innerText()), "Seite 0: Zeropage: " + (await page.locator("[data-spk=art]").innerText()));
await page.locator("[data-spk=seite][data-p='0']").focus(); await page.keyboard.press("ArrowRight"); await warte(500); ok(/\$0100–\$01FF/.test(await page.locator("[data-spk=adresse]").innerText()), "Pfeil nach rechts wählt die nächste Seite ($0100)");
const pre = await page.locator("[data-spk=preset]").count(); if (pre > 1) { await page.locator("[data-spk=preset]").nth(1).click(); await warte(600); ok((await page.locator("[data-spk=preset][aria-checked=true]").count()) === 1, "Bankumschaltung: Vorgabe wählbar (" + pre + " Vorgaben)"); } else ok(true, "(keine Bankumschaltung bei diesem System)");
await page.getByRole("radio", { name: "Programm" }).click(); await warte(200); ok((await page.locator("[data-spk=seite].dim").count()) > 0 && (await page.locator("[data-spk=seite]:not(.dim)").count()) > 0, "Filter „Programm“ blendet Seiten ohne Programm ab");
await snap("daten-2-speicher");
console.log(`\n${n} Prüfungen, ${fails.length} Fehler`);
await browser.close();
process.exit(fails.length ? 1 : 0);
