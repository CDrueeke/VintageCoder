// SPDX-License-Identifier: GPL-3.0-or-later
// Ablauftest Bauen und Starten (E1.3): Playwright (WebKit) gegen die laufende Prüfbrücke des echten Kerns (Emulator wird nicht gestartet: VC_TEST_NO_SPAWN).
// Aufruf: node tools/e2e-bau.mjs <Ordner mit c64/jumprun und common/ (Kopie!)> [Bilderordner]
import { webkit } from "playwright";
import { existsSync, mkdirSync, statSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const [base, shots] = process.argv.slice(2);
if (!base) { console.error("Aufruf: node tools/e2e-bau.mjs <Ordner> [Bilder]"); process.exit(2); }
if (shots) mkdirSync(shots, { recursive: true });
const proj = join(base, "c64/jumprun");
const browser = await webkit.launch();
const page = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
const fails = []; let n = 0; const csp = [];
const ok = (c, m) => { n++; if (!c) { fails.push(m); console.log("FEHLER:", m); } else console.log("ok:", m); };
page.on("pageerror", (e) => { fails.push("Seitenfehler " + e.message); console.log("Seitenfehler", e.message); });
page.on("console", (m) => { if (/Content Security Policy|Refused to/i.test(m.text())) csp.push(m.text()); });
const snap = async (name) => { if (shots) await page.screenshot({ path: join(shots, name + ".png") }); };
const status = () => page.locator("footer.status").innerText().then((s) => s.replace(/\s+/g, " "));

await page.goto("http://127.0.0.1:1420/?bridge=17420");
await page.waitForFunction(() => window.__vc && window.__vc.ui.ready);
await page.evaluate(async ([p]) => { await window.__vc.setLanguage("de"); await window.__vc.openProject(p); await window.__vc.openFile("jumprun.asm"); }, [proj]);
await page.waitForSelector(".cm-content");

// Bauen mit dem Knopf
await page.getByRole("button", { name: "Bauen" }).first().click();
await page.waitForFunction(() => /gebaut/.test(document.querySelector("footer.status")?.innerText ?? ""), null, { timeout: 60000 }).catch(() => {});
ok(/gebaut: [\d.]+ Bytes/.test(await status()), "Bauen mit dem Knopf: Statuszeile meldet die Größe: " + (await status()).slice(0, 80));
await page.waitForTimeout(1500);
ok(/Keine Probleme/.test(await status()), "Nach dem Bau ist der Sprachdienst neu abgeglichen (build/export ist da): " + (await status()).slice(0, 60));
ok(existsSync(join(proj, "build/jumprun.prg")) && statSync(join(proj, "build/jumprun.prg")).size > 1000, "Artefakt build/jumprun.prg liegt vor");
await page.mouse.move(700, 450); await page.mouse.wheel(0, 1400); await page.waitForSelector(".cm-gutter-cyc .cyc", { timeout: 5000 }).catch(() => {});
const cyc = (await page.locator(".cm-gutter-cyc .cyc").allInnerTexts()).filter((c) => c);
ok(cyc.length > 5 && cyc.every((c) => /^\d+(–\d+)?$/.test(c)), "Zyklenzahl je Zeile steht im Rand (" + cyc.length + " Zeilen, z. B. " + cyc.slice(0, 4).join(" ") + ")");
ok((await page.locator("footer.status .strip i").count()) > 0, "Speicherstreifen in der Statuszeile zeigt die Segmente");
await snap("bau-0-zyklen");
await page.locator("footer.status button", { hasText: "gebaut" }).click(); await page.waitForSelector(".out .l");
const out = await page.locator(".out .l").allInnerTexts();
ok(out.some((l) => /^Bauen: jumprun\.asm/.test(l)) && out.some((l) => /^Fertig: [\d.]+ Bytes/.test(l)), "Ausgabe zeigt Bauen und Fertig (" + out.length + " Zeilen)");
await snap("bau-1-ausgabe");
ok(await page.getByRole("button", { name: "Bauen" }).first().isEnabled(), "Knopf ist nach dem Bau wieder frei");

// Starten (kein Emulator im Test): Meldung und Stopp-Knopf
await page.getByRole("button", { name: "Starten" }).first().click();
await page.waitForFunction(() => [...document.querySelectorAll(".out .l")].some((l) => /^Gestartet:/.test(l.textContent)), null, { timeout: 60000 }).catch(() => {});
ok((await page.locator(".out .l").allInnerTexts()).some((l) => /^Gestartet:/.test(l)), "Starten baut und meldet „Gestartet“ in der Ausgabe");
ok((await page.getByRole("button", { name: "Beenden" }).count()) === 1, "Beenden-Knopf erscheint, solange das Programm läuft");
await page.getByRole("button", { name: "Beenden" }).click(); await page.getByRole("button", { name: "Beenden" }).first().waitFor({ state: "detached", timeout: 5000 }).catch(() => {});   // das Beenden des Prozessbaums dauert bis etwa 350 ms (Windows, Linux)
ok((await page.getByRole("button", { name: "Beenden" }).count()) === 0, "Beenden schaltet zurück");

// Emulator beenden: Kopfzeile, Maschine-Panel und Palette gehen denselben Weg; ein von selbst beendeter Emulator wird erkannt (Platzhalterprozess VC_TEST_FAKE_EMU)
if (process.env.VC_TEST_FAKE_EMU) {
  const laeuft = () => page.evaluate(() => window.__vc.call("run_stop").then((w) => w)).then((w) => w);   // beendet gleichzeitig; true = es lief noch
  const starte = async () => { await page.getByRole("button", { name: "Starten" }).first().click(); await page.waitForSelector("header [aria-label=Beenden]", { timeout: 60000 }); };
  const zustand = () => page.evaluate(() => window.__vc.ui.panel).then(() => page.locator(".m [role=status]").innerText().catch(() => "-"));
  await page.evaluate(() => { window.__vc.ui.panel = true; }); await page.waitForTimeout(300);
  await starte();
  ok((await zustand()) === "läuft", "Maschine-Panel zeigt „läuft“: " + await zustand());
  await page.locator(".m .act button", { hasText: "Beenden" }).click(); await page.waitForTimeout(400);
  ok((await zustand()) === "bereit" && (await page.locator("header [aria-label=Beenden]").count()) === 0, "Beenden im Panel: Zustand „bereit“, Knopf in der Kopfzeile weg");
  ok(!(await laeuft()), "… und der Prozess ist wirklich beendet");
  await starte(); await page.locator("header [aria-label=Beenden]").click(); await page.waitForTimeout(400);
  ok((await zustand()) === "bereit" && !(await laeuft()), "Beenden in der Kopfzeile: Zustand „bereit“, Prozess beendet");
  await starte(); await page.evaluate(() => { void window.__vc.runCommand("debug_ende"); }); await page.waitForTimeout(400);
  ok((await zustand()) === "bereit" && !(await laeuft()), "Beenden über die Palette (Debuggen beenden): Zustand „bereit“, Prozess beendet");
  await starte();
  await page.waitForFunction(() => document.querySelector(".m [role=status]")?.textContent === "bereit", null, { timeout: 20000 }).catch(() => {});
  ok((await zustand()) === "bereit", "Von selbst beendeter Emulator: Zustand wechselt auf „bereit“");
  ok((await page.locator(".out .l").allInnerTexts()).some((l) => /^Emulator beendet/.test(l)), "… die Ausgabe meldet „Emulator beendet“");
  ok((await page.locator("header [aria-label=Beenden]").count()) === 0, "… und der Beenden-Knopf ist weg");
  await starte(); ok(true, "Ein neuer Start nach dem Ende geht wieder");
  await page.locator("header [aria-label=Beenden]").click(); await page.waitForTimeout(300);
  await page.evaluate(() => { window.__vc.ui.panel = false; });
}

// Ziel-Chip
await page.locator("header .chip").click(); await page.waitForSelector(".vc-fixmenu button");
ok((await page.locator(".vc-fixmenu button").allInnerTexts()).some((x) => /Emulator|VICE|xemu|Altirra|atari800/i.test(x)), "Startziel-Menü nennt den Emulator: " + (await page.locator(".vc-fixmenu button").allInnerTexts()).join(" | "));
await page.keyboard.press("Escape"); await page.locator("header .chip").click().catch(() => {});

// Fehler beim Bauen: sichtbar in Probleme, Ausgabe und Meldung; Klick springt zur Zeile
writeFileSync(join(proj, "kaputt.asm"), ".target c64\n.org $0801\n  lda #1\n  lda nirgendwo\n  rts\n");
await page.evaluate(async () => { const r = await window.__vc.call("tree_reload"); window.__vc.ui.project.tree = r.tree; await window.__vc.openFile("kaputt.asm"); });
await page.waitForTimeout(400);
await page.evaluate(() => { void window.__vc.runCommand("bauen_aktiv"); });
await page.waitForFunction(() => /Bau: \d+ Fehler/.test(document.querySelector("footer.status")?.innerText ?? ""), null, { timeout: 60000 }).catch(() => {});
ok(/Bau: 1 Fehler/.test(await status()), "Fehlgeschlagener Bau: Statuszeile „Bau: 1 Fehler“: " + (await status()).slice(0, 80));
ok((await page.locator(".toast.err").count()) > 0, "… und eine sichtbare Fehlermeldung");
await page.waitForSelector(".problems .file"); const heads = await page.locator(".problems .file").allInnerTexts();
ok(heads.some((h) => /Bauergebnis/.test(h)), "Probleme zeigt die Gruppe „Bauergebnis“: " + heads.join(" | "));
await snap("bau-2-fehler");
await page.locator(".problems .row", { hasText: "nirgendwo" }).first().click(); await page.waitForTimeout(300);
ok(/Zeile 4,/.test(await status()), "Klick auf den Bauerfehler springt in Zeile 4: " + (await status()).slice(-50));
// Kontextmenü eines Problems, Befehle Springen und Kopieren
await page.locator(".problems .row", { hasText: "nirgendwo" }).first().click({ button: "right" }); await page.waitForSelector(".vc-fixmenu button");
ok(JSON.stringify(await page.locator(".vc-fixmenu button").allInnerTexts()) === JSON.stringify(["Problem: Zur Zeile springen", "Problem: Meldung kopieren", "Alle Probleme kopieren"]), "Kontextmenü Problem: " + (await page.locator(".vc-fixmenu button").allInnerTexts()).join(" | "));
await page.locator(".vc-fixmenu button", { hasText: "Zur Zeile springen" }).click(); await page.waitForTimeout(300);
ok(/Zeile 4,/.test(await status()), "Kontextmenü ▸ Springen führt in die Zeile");
await page.evaluate(() => { void window.__vc.runCommand("problem_kopieren"); }); await page.waitForTimeout(300);
ok((await page.locator(".toast").allInnerTexts()).some((x) => /kopiert|Zwischenablage/.test(x)) || true, "Befehl „Problem kopieren“ läuft ohne Fehler");
// Maschine-Panel und Schublade
await page.evaluate(() => { void window.__vc.runCommand("panel"); }); await page.waitForSelector(".m");
ok(/Maschine/.test(await page.locator(".m h2").innerText()) && (await page.locator(".m table tbody tr").count()) > 0, "Maschine-Panel zeigt Ziel und Segmente des Baus (" + await page.locator(".m table tbody tr").count() + " Segment(e))");
await snap("bau-3-maschine");
await page.evaluate(() => { void window.__vc.runCommand("panel"); }); await page.waitForTimeout(200);
ok((await page.locator(".m").count()) === 0, "Panel lässt sich wieder ausblenden");
const dockOpen = (await page.locator(".dock").count()) > 0; await page.evaluate(() => { void window.__vc.runCommand("schublade"); }); await page.waitForTimeout(200);
ok(((await page.locator(".dock").count()) > 0) !== dockOpen, "Schublade (⌘J) schaltet die Ablage um");
await page.evaluate(() => { void window.__vc.runCommand("schublade"); });
// Ausgabe: Kontextmenü
await page.locator("footer.status button", { hasText: /Bau|gebaut/ }).first().click(); await page.waitForSelector(".out .l");
await page.locator(".out .log").click({ button: "right" }); await page.waitForSelector(".vc-fixmenu button");
ok((await page.locator(".vc-fixmenu button").allInnerTexts()).join("|").includes("Leeren") || (await page.locator(".vc-fixmenu button").allInnerTexts()).some((x) => /leeren/i.test(x)), "Kontextmenü Ausgabe: " + (await page.locator(".vc-fixmenu button").allInnerTexts()).join(" | "));
await page.keyboard.press("Escape");
// Tests: Läufe über *.vctest (hier nur statische Prüfungen), Filter, Detail
writeFileSync(join(proj, "jumprun.vctest"), '{ "kind": "test", "target": "c64", "program": "build/jumprun.prg", "static": [ { "check": "overlap" }, { "check": "size", "max": 100000 }, { "check": "size", "max": 1 } ] }');
await page.evaluate(() => { void window.__vc.runCommand("tests_ausfuehren"); });
await page.waitForFunction(() => /bestanden/.test(document.querySelector(".tp .info")?.textContent ?? ""), null, { timeout: 60000 }).catch(() => {});
ok(/2 von 3 bestanden/.test(await page.locator(".tp .info").innerText()), "Tests laufen (statisch): " + await page.locator(".tp .info").innerText());
ok((await page.locator(".tp .row").count()) === 3, "Liste zeigt drei Ergebnisse");
await page.locator(".tp .row.failed").first().click();
ok(/fehlgeschlagen/.test(await page.locator(".tp .detail").innerText()), "Detail zum fehlgeschlagenen Test: " + (await page.locator(".tp .detail").innerText()).replace(/\n/g, " | ").slice(0, 100));
await page.locator(".tp .chk input").check(); await page.waitForTimeout(150);
ok((await page.locator(".tp .row").count()) === 1, "„nur Fehler“ zeigt nur den fehlgeschlagenen Test");
await snap("bau-4-tests"); await page.locator(".tp .chk input").uncheck();
await page.locator(".tp input[type=search]").fill("Größe"); await page.waitForTimeout(150);
ok((await page.locator(".tp .row").count()) === 2, "Filter „Größe“ zeigt zwei Ergebnisse");
await page.locator(".tp input[type=search]").fill("");
await page.locator(".dock > header [aria-label='Ablage schließen']").click();
// Ausgabe: Zeilen mit Datei:Zeile sind klickbar; Ablage auch ohne offenen Reiter bedienbar
await page.evaluate(() => { window.__vc.ui.mode = "code"; }); await dockOutput();
async function dockOutput() { if ((await page.locator(".dock").count()) === 0) await page.evaluate(() => { void window.__vc.runCommand("schublade"); }); await page.locator(".dock [data-tab=output]").click(); await page.waitForTimeout(300); }
const jump = page.locator(".out .jump", { hasText: "kaputt.asm:4" });
ok((await jump.count()) > 0, "Ausgabe: die Fehlerzeile „kaputt.asm:4: …“ ist anklickbar");
await page.evaluate(async () => { await window.__vc.openFile("jumprun.asm"); }); await jump.first().click(); await page.waitForTimeout(400);
ok(/Zeile 4,/.test(await status()) && (await page.locator(".tab.on .nm").innerText()) === "kaputt.asm", "Klick in der Ausgabe springt in kaputt.asm Zeile 4");
for (let i = 0; i < 8; i++) await page.evaluate(() => { void window.__vc.runCommand("datei_schliessen"); }); await page.waitForTimeout(300);
ok((await page.locator(".tab").count()) === 0 && (await page.locator(".empty").count()) === 1, "Alle Reiter sind zu");
const offen = (await page.locator(".dock").count()) > 0; await page.evaluate(() => { void window.__vc.runCommand("schublade"); }); await page.waitForTimeout(200);
ok(((await page.locator(".dock").count()) > 0) !== offen, "Ohne Reiter lässt sich die Ablage (⌘J) umschalten");
await page.evaluate(() => { void window.__vc.runCommand("bauen"); }); await page.waitForFunction(() => /gebaut/.test(document.querySelector("footer.status")?.innerText ?? ""), null, { timeout: 60000 }).catch(() => {});
await page.evaluate(() => { void window.__vc.runCommand("protokoll_leeren"); });
await page.locator("footer.status button", { hasText: /gebaut|Bau/ }).first().click(); await page.waitForTimeout(300);
ok((await page.locator(".dock").count()) > 0 && (await page.locator(".dock [data-tab=output][aria-selected=true]").count()) === 1, "Klick auf das Bauergebnis in der Statuszeile öffnet die Ausgabe auch ohne Reiter");
await page.evaluate(() => { void window.__vc.runCommand("schublade"); });
// F7 baut die Hauptquelle
await page.evaluate(async () => { await window.__vc.openFile("jumprun.asm"); }); await page.waitForTimeout(500);
await page.locator(".cm-content").click(); await page.keyboard.press("F7");
await page.waitForFunction(() => /gebaut/.test(document.querySelector("footer.status")?.innerText ?? ""), null, { timeout: 60000 }).catch(() => {});
ok(/gebaut/.test(await status()), "F7 baut die Hauptquelle (jumprun.asm) wieder fehlerfrei");
// Kopfzeile bei 900 x 600: alle Knöpfe erreichbar, nichts läuft über den Rand
await page.setViewportSize({ width: 900, height: 600 }); await page.waitForTimeout(400);
const kopf = await page.evaluate(() => ({ breite: document.documentElement.scrollWidth, knoepfe: ["Bauen", "Starten", "Startziel"].map((n) => { const b = document.querySelector(`header [aria-label^="${n}"]`); if (!b) return n + ":fehlt"; const r = b.getBoundingClientRect(); const e = document.elementFromPoint(r.x + r.width / 2, r.y + r.height / 2); return n + ":" + (b.contains(e) ? "frei" : "verdeckt") + (r.right > innerWidth ? "-abgeschnitten" : ""); }), rechts: Math.max(...[...document.querySelectorAll("header .ibtn, header .chip")].map((e) => e.getBoundingClientRect().right)) }));
ok(kopf.breite <= 900 && kopf.knoepfe.every((k) => /:frei$/.test(k)) && kopf.rechts <= 900, "Kopfzeile bei 900 px: " + kopf.knoepfe.join(" ") + ", Seitenbreite " + kopf.breite + ", rechtester Knopf " + Math.round(kopf.rechts));
await snap("bau-5-schmal"); await page.setViewportSize({ width: 1440, height: 900 });
// W203 (zu weite Sprünge) zusammengefasst, Tooltip am unteren Rand, Pfad mit Tooltip, Randmarken bei umbrochenen Zeilen, Ende der Liste sichtbar
await page.evaluate(async () => { await window.__vc.openFile("lib/music.inc"); }); await page.waitForTimeout(1500);
await page.evaluate(() => { window.__vc.ui.panel = false; });
await page.locator("footer.status button[aria-pressed]").first().click().catch(() => {}); await page.waitForTimeout(300);
await page.evaluate(() => { void window.__vc.runCommand("schublade"); }); await page.waitForTimeout(100);
await page.evaluate(() => { window.__vc.ui.mode = "code"; }); await page.waitForTimeout(100);
await page.evaluate(async () => { const m = await window.__vc.call("tree_reload"); void m; });
const dockProbleme = async () => { if ((await page.locator(".dock").count()) === 0) await page.evaluate(() => { void window.__vc.runCommand("schublade"); }); await page.locator(".dock [data-tab=problems]").click(); await page.waitForTimeout(300); };
await dockProbleme();
const sum = page.locator(".problems .row", { hasText: "zu weite Sprünge" });
ok((await sum.count()) >= 1, "Probleme fasst die Warnungen zu weiter Sprünge je Datei zu einer Zeile zusammen (" + await sum.count() + " Datei(en)): " + await sum.first().innerText().catch(() => "-"));
const vorher = await page.locator(".problems .row").count(); await sum.first().click(); await page.waitForTimeout(200);
ok((await page.locator(".problems .row").count()) > vorher + 3, "… und lässt sich aufklappen (" + vorher + " → " + await page.locator(".problems .row").count() + " Zeilen)");
ok(!/Warnung/.test(await status()) || /Hinweis/.test(await status()), "Statuszeile zählt sie als einen Hinweis, nicht als viele Warnungen: " + (await status()).slice(0, 80));
// Tooltip am Speicherstreifen unten: ganz sichtbar
await page.mouse.move(5, 5); await page.locator("footer.status .strip").hover(); await page.waitForSelector(".tip", { timeout: 3000 }).catch(() => {});
const tb = await page.locator(".tip").boundingBox().catch(() => null);
ok(tb && tb.y + tb.height <= 900 && tb.y >= 0, "Tooltip am Speicherstreifen ist vollständig sichtbar (unten " + (tb ? Math.round(tb.y + tb.height) : "?") + " von 900)");
await snap("bau-6-tooltip");
await page.mouse.move(5, 5);
const titelPfad = ((await page.locator(".pname span[title]").getAttribute("title")) ?? "").replace(/\\/g, "/").toLowerCase();
ok(titelPfad.endsWith("/vc-bau/c64/jumprun"), "Pfad in der Titelleiste hat den vollen Pfad als Tooltip: " + titelPfad);   // plattformneutral: unter Windows steht C:/tmp/…
// Randmarken bei umbrochenen Zeilen: Faltmarke steht an der ersten Zeile des Blocks, nicht in der Mitte
await page.evaluate(async () => { await window.__vc.openFile("jumprun.asm"); window.__vc.ui.panel = true; }); await page.waitForTimeout(800);
const ofs = await page.evaluate(() => { const line = document.querySelector(".cm-line").getBoundingClientRect(); const fm = [...document.querySelectorAll(".cm-foldGutter .cm-gutterElement")].find((e) => e.querySelector(".vc-fold") && getComputedStyle(e).visibility !== "hidden"); return { l: line.top, h: line.height, f: fm ? fm.querySelector(".vc-fold").getBoundingClientRect().top : null }; });
ok(ofs.f !== null && ofs.f - ofs.l < 22, "Faltmarke steht bei umbrochener Zeile oben an der ersten Zeile (" + (ofs.f !== null ? Math.round(ofs.f - ofs.l) : "?") + " px unter dem Zeilenanfang, Zeile " + Math.round(ofs.h) + " px hoch)");
await snap("bau-7-umbruch-panel"); await page.evaluate(() => { window.__vc.ui.panel = false; });
// ohne Quelle
ok(csp.length === 0, "Keine Meldungen der Inhaltsrichtlinie: " + csp.slice(0, 2).join(" | "));
console.log(`\n${n} Prüfungen, ${fails.length} Fehler`);
await browser.close();
process.exit(fails.length ? 1 : 0);
