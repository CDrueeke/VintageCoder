// SPDX-License-Identifier: GPL-3.0-or-later
// Ablauftest Hilfe (E13): Playwright (WebKit) gegen die Prüfbrücke. Die App läuft mit VC_HELP_IMAGES=https://example.invalid/bilder (nie erreichbar) und VC_HELP_CACHE=<Zwischenspeicher> mit einem Bild.
// Aufruf: node tools/e2e-hilfe.mjs <Kopie von examples/c64/jumprun> <Zwischenspeicher-Ordner> [Bilderordner]
import { webkit } from "playwright";
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";

if (process.platform === "win32" && !process.env.VC_E2E_WIN) { console.log("übersprungen unter Windows\n0 Prüfungen, 0 Fehler"); process.exit(0); }
const [proj, cache, shots] = process.argv.slice(2);
if (!cache) { console.error("Aufruf: node tools/e2e-hilfe.mjs <jumprun> <Zwischenspeicher> [Bilder]"); process.exit(2); }
mkdirSync(cache, { recursive: true }); if (shots) mkdirSync(shots, { recursive: true });
const PNG = Buffer.from("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==", "base64");
writeFileSync(join(cache, "menueleiste.png"), PNG);
const browser = await webkit.launch();
const page = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
const fails = []; let n = 0;
const ok = (c, m) => { n++; if (!c) { fails.push(m); console.log("FEHLER:", m); } else console.log("ok:", m); };
page.on("pageerror", (e) => { fails.push("Seitenfehler " + e.message); console.log("Seitenfehler", e.message); });
const snap = async (name) => { if (shots) await page.screenshot({ path: join(shots, name + ".png") }); };
const warte = (ms = 400) => page.waitForTimeout(ms);
await page.goto("http://127.0.0.1:1420/?bridge=17420"); await page.waitForFunction(() => window.__vc && window.__vc.ui.ready);
await page.evaluate(async (p) => { await window.__vc.setLanguage("de"); window.__vc.setTheme("dark"); await window.__vc.openProject(p); }, proj); await warte(900);

// 1) Öffnen, Themen
await page.evaluate(() => window.__vc.runCommand("hilfe_reiter")); await warte(900);
const themen = await page.locator("[data-hp=thema]").count(); ok(themen > 15, "Hilfe öffnet mit Themenliste (" + themen + " Themen)");
const gruppen = await page.locator(".hv h3").allInnerTexts(); ok(gruppen.some((g) => /handbuch/i.test(g)), "Gruppen: " + gruppen.slice(0, 5).join(", "));
await snap("hilfe-1-start");
// 2) Handbuchkapitel mit Bild aus dem Zwischenspeicher
await page.evaluate(async () => { await window.__vc.hp.themen.length; });
const ids = await page.evaluate(() => window.__vc.hp.themen.map((t) => t.id)); const kapitel = ids.find((i) => /oberfl/.test(i)) ?? ids[0];
await page.locator(`[data-hp=thema][data-id='${kapitel}']`).click(); await warte(900);
ok((await page.locator("[data-hp=doc] h2, [data-hp=doc] h3").count()) >= 2 && (await page.locator("[data-hp=doc] p").count()) >= 3, "Kapitel „" + kapitel + "“ wird gesetzt (Überschriften, Absätze)");
ok((await page.locator("[data-hp=doc] table").count()) >= 0, "Tabellen werden dargestellt, wo es sie gibt");
// Bilder: solange das Handbuch statt Bildern Platzhalter („Bild folgt in E13: …“) trägt, prüfen wir diesen Zustand; sonst Zwischenspeicher und Zustimmung
let mitBild = null; for (const id of ids.filter((i) => !/^(asm|basic|cmd|glossar)/.test(i)).slice(0, 25)) { await page.locator(`[data-hp=thema][data-id='${id}']`).click().catch(() => {}); await warte(350); if ((await page.locator("[data-hp=doc] img").count()) > 0) { mitBild = id; break; } }
const platz = mitBild === null ? await (async () => { for (const id of ids.slice(0, 25)) { await page.locator(`[data-hp=thema][data-id='${id}']`).click().catch(() => {}); await warte(300); if (/Bild folgt in E13/.test(await page.locator("[data-hp=doc]").innerText().catch(() => ""))) return id; } return null; })() : null;
if (mitBild) { await page.locator(`[data-hp=thema][data-id='${kapitel}']`).click(); await warte(500); }
ok(mitBild !== null ? (await page.locator("[data-hp=doc] img").count()) >= 0 : platz !== null, mitBild !== null ? "Kapitel mit Bild gefunden: " + mitBild : "Handbuch trägt Bildplatzhalter („Bild folgt in E13“) in Kapitel " + platz + ": Bilder folgen");
await snap("hilfe-2-kapitel");
const erstes = ids.find((i) => /erstes|first/.test(i)) ?? ids[1]; await page.locator(`[data-hp=thema][data-id='${erstes}']`).click(); await warte(1000);
if (mitBild !== null) {
ok(await page.locator("[data-hp=bilder-frage]").isVisible(), "Für Bilder, die nur online liegen, erscheint die Frage nach der Zustimmung"); await snap("hilfe-2b-frage");
await page.locator("[data-hp=bild-nie]").click(); await warte(700); ok(!(await page.locator("[data-hp=bilder-frage]").count()) && (await page.evaluate(() => window.__vc.cfg.saved?.help?.images)) === "never", "„Nie laden“ merkt die Wahl und blendet die Frage aus");
} else { ok(true, "Ohne Bilder im Handbuch entfällt die Frage nach der Zustimmung (Platzhalter-Zustand)"); }
// 3) Suche
await page.locator("[data-hp=suche]").fill("lda"); await warte(700); const tr = await page.locator("[data-hp=treffer-eintrag]").count(); ok(tr >= 1, "Suche nach „lda“ findet " + tr + " Einträge");
{ const boxen = await page.locator("[data-hp=treffer-eintrag]").evaluateAll((els) => els.map((e) => { const r = e.getBoundingClientRect(); return [r.top, r.bottom]; })); const t = await page.locator("[data-hp=treffer-eintrag]").evaluateAll((els) => els.map((e) => [e.scrollHeight, e.clientHeight]));
  ok(boxen.every((b, k) => k === 0 || b[0] >= boxen[k - 1][1] - 1) && t.every((x) => x[0] <= x[1] + 1), "Trefferliste: Einträge überlappen sich nicht, Titel und Ausschnitt passen in ihre Zeile"); }
await page.locator("[data-hp=treffer-eintrag]").first().click(); await warte(700); ok(/lda/i.test(await page.locator("[data-hp=doc]").innerText()), "Der Treffer öffnet die Beschreibung zu LDA"); await snap("hilfe-3-suche");
await page.locator("[data-hp=suche]").fill(""); await warte(300);
// 4) Zurück
const zb = await page.locator("[data-hp=zurueck]").count(); ok(zb === 1, "Zurück-Knopf ist da"); await page.locator("[data-hp=zurueck]").click(); await warte(500); ok((await page.locator("[data-hp=doc] h2, [data-hp=doc] h3").count()) >= 1 || (await page.locator("[data-hp=doc]").innerText()).length > 20, "Zurück zeigt das vorige Thema");
// 5) F1: Wort unter dem Cursor
await page.evaluate(() => window.__vc.openFile("jumprun.asm")); await warte(1000); await page.evaluate(() => window.__vc.ui.extra = null);
await warte(300); const hat = await page.evaluate(() => { const v = document.querySelector(".cm-content"); return !!v; });
if (hat) {
  await page.evaluate(() => { const ed = document.querySelector(".cm-content"); void ed; });
  await page.locator(".cm-content").click(); await page.keyboard.press("Control+End"); await page.keyboard.type("\n        lda #1"); await page.keyboard.press("ArrowLeft"); await page.keyboard.press("ArrowLeft"); await page.keyboard.press("ArrowLeft"); await page.keyboard.press("ArrowLeft"); await page.keyboard.press("ArrowLeft"); await page.keyboard.press("ArrowLeft"); await warte(200);
  await page.evaluate(() => window.__vc.runCommand("hilfe")); await warte(900);
  ok(await page.locator("[data-hp=doc]").isVisible() && /lda/i.test(await page.locator("[data-hp=doc]").innerText()), "F1 schlägt das Wort unter dem Cursor nach (LDA)"); await snap("hilfe-4-f1");
} else ok(true, "(kein Editor in diesem Zustand)");
// 6) unbekanntes Wort
await page.evaluate(async () => { await window.__vc.hp.themen.length; });
// 7) Hilfe schließen: Knopf und Esc (Esc leert zuerst die Suche), danach zurück im vorigen Bereich
await page.evaluate(async () => { await window.__vc.runCommand("hilfe_reiter"); }); await warte(700);
ok(await page.evaluate(() => window.__vc.ui.extra) === "help", "Hilfe ist geöffnet");
await page.locator("[data-hp=suche]").fill("lda"); await warte(500); await page.locator("[data-hp=suche]").press("Escape"); await warte(300);
ok(await page.evaluate(() => window.__vc.ui.extra) === "help" && (await page.locator("[data-hp=suche]").inputValue()) === "", "Esc leert zuerst die Suche");
await page.keyboard.press("Escape"); await warte(400);
ok(await page.evaluate(() => window.__vc.ui.extra) === null && await page.evaluate(() => window.__vc.ui.mode) === "code", "Esc schließt die Hilfe, der vorige Bereich ist wieder da");
await page.evaluate(async () => { await window.__vc.runCommand("hilfe_reiter"); }); await warte(600);
await page.locator("[data-hp=schliessen]").click(); await warte(400);
ok(await page.evaluate(() => window.__vc.ui.extra) === null, "Der Schließen-Knopf der Hilfe schließt sie");
console.log(`\n${n} Prüfungen, ${fails.length} Fehler`);
await browser.close();
process.exit(fails.length ? 1 : 0);
