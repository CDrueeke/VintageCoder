// SPDX-License-Identifier: GPL-3.0-or-later
// Ablauftest Sitzung, Zwischenkopie und Beenden mit Liste (E1.4): Playwright (WebKit) gegen die Prüfbrücke des echten Kerns.
// Aufruf: HOME=<leerer Ordner> muss beim Start von `vintagecoder` gesetzt sein (Konfiguration landet dort):
//   node tools/e2e-sitzung.mjs <Projektkopie (wird verändert!)> <HOME des Kerns> [Bilderordner]
import { webkit } from "playwright";
import { existsSync, mkdirSync, readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";

const [proj, home, shots] = process.argv.slice(2);
if (!proj || !home) { console.error("Aufruf: node tools/e2e-sitzung.mjs <Projektkopie> <HOME> [Bilder]"); process.exit(2); }
if (shots) mkdirSync(shots, { recursive: true });
const cfg = join(home, ".config/vintagecoder");
const lies = (n) => { try { return JSON.parse(readFileSync(join(cfg, n), "utf8")); } catch { return null; } };
const browser = await webkit.launch();
const page = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
const fails = []; let n = 0; const csp = [];
const ok = (c, m) => { n++; if (!c) { fails.push(m); console.log("FEHLER:", m); } else console.log("ok:", m); };
page.on("pageerror", (e) => { fails.push("Seitenfehler " + e.message); console.log("Seitenfehler", e.message); });
page.on("console", (m) => { if (/Content Security Policy|Refused to/i.test(m.text())) csp.push(m.text()); });
const snap = async (name) => { if (shots) await page.screenshot({ path: join(shots, name + ".png") }); };
const start = async () => { await page.goto("http://127.0.0.1:1420/?bridge=17420"); await page.waitForFunction(() => window.__vc && window.__vc.ui.ready); await page.evaluate(async () => { await window.__vc.setLanguage("de"); }); };
const oeffne = async () => { await page.evaluate(async ([p]) => { await window.__vc.openProject(p); }, [proj]); await page.waitForTimeout(900); };
const tabs = () => page.locator(".tab .nm").allInnerTexts();
const dateien = await (async () => { await start(); await oeffne(); return page.evaluate(() => { const f = (ns) => ns.flatMap((x) => (x.kind === "dir" ? f(x.children ?? []) : [x])); return f(window.__vc.ui.project.tree).map((x) => x.rel).filter((r) => /\.(asm|inc)$/.test(r)); }); })();
ok(dateien.length >= 3, "Projekt hat mindestens drei Dateien: " + dateien.slice(0, 3).join(", "));
const [a, b, c] = dateien;

// 1) Sitzung wird gemerkt (auch wenn das Wiederherstellen aus ist)
for (const r of [a, b, c]) await page.evaluate(async (r) => { await window.__vc.openFile(r); }, r);
await page.locator(`.tab`, { hasText: b.split("/").pop() }).first().click(); await page.waitForTimeout(700);
const s1 = lies("session.json")?.projects?.[proj];
ok(s1 && JSON.stringify(s1.open) === JSON.stringify([a, b, c]) && s1.active === b, "session.json merkt offene Reiter in Reihenfolge und die aktive Datei: " + JSON.stringify(s1));

// 2) Ohne Schalter wird nichts wiederhergestellt
await start(); await oeffne();
ok((await tabs()).length === 0, "Ohne Schalter öffnet das Projekt ohne Reiter");
ok(lies("session.json")?.projects?.[proj]?.open?.length === 3, "… und die gemerkte Sitzung bleibt dabei erhalten (wird nicht mit „leer“ überschrieben)");

// 3) Mit Schalter: Reiter wiederhergestellt
await page.evaluate(async () => { await window.__vc.call("restore_tabs_set", { on: true }); });
await start(); await oeffne();
const t3 = await tabs();
ok(t3.length === 3 && t3.join("|") === [a, b, c].map((x) => x.split("/").pop()).join("|"), "Mit Schalter kommen die drei Reiter in der alten Reihenfolge zurück: " + t3.join(", "));
ok((await page.locator(".tab.on .nm").innerText()) === b.split("/").pop(), "… und die aktive Datei ist wieder aktiv");
ok(/"restore_tabs": true/.test(readFileSync(join(proj, readdirSync(proj).find((x) => x.endsWith(".vcprj"))), "utf8")), "Der Schalter steht in der .vcprj (restore_tabs)");
await snap("sitzung-1-wiederhergestellt");
// Nur die Reiter schließen: die Sitzung folgt
await page.evaluate(() => { void window.__vc.runCommand("datei_schliessen"); }); await page.waitForTimeout(800);
ok(lies("session.json")?.projects?.[proj]?.open?.length === 2, "Schließen eines Reiters aktualisiert die Sitzung");

// 4) Zwischenkopie unbenannter Reiter
await page.evaluate(() => { void window.__vc.runCommand("neuer_reiter"); }); await page.waitForTimeout(300);
await page.locator(".cm-content").click(); await page.keyboard.type("lda #$42\n sta $d020"); await page.waitForTimeout(2300);
const sc = lies("scratch.json");
ok(sc?.entries?.length === 1 && /lda #\$42/.test(sc.entries[0].text), "Unbenannter Reiter mit Text wird in scratch.json gesichert: " + JSON.stringify(sc?.entries?.map((e) => e.name)));
await snap("sitzung-2-unbenannt");
// „Absturz“: Seite neu laden, ohne zu beenden
await start(); await oeffne();
await page.waitForSelector("[role=dialog]", { timeout: 5000 }).catch(() => {});
const dlg = await page.getByRole("dialog").first().innerText().catch(() => "");
ok(/Unbenannte Reiter wiederherstellen/.test(dlg) && /Unbenannt 1/.test(dlg), "Beim nächsten Start fragt ein Dialog nach den gesicherten Reitern: " + dlg.replace(/\s+/g, " ").slice(0, 100));
await snap("sitzung-3-dialog");
await page.getByRole("button", { name: "Wiederherstellen" }).click(); await page.waitForTimeout(400);
ok((await tabs()).some((x) => /Unbenannt/.test(x)), "Wiederherstellen legt den Reiter wieder an");
ok(/lda #\$42/.test(await page.locator(".cm-content").innerText()), "… mit seinem Text");
// 5) Beenden/Projektwechsel mit Liste
await page.evaluate(async (r) => { await window.__vc.openFile(r); }, a); await page.locator(".cm-content").click(); await page.keyboard.type(" "); await page.waitForTimeout(300);
await page.evaluate(() => { void window.__vc.runCommand("beenden"); }); await page.waitForTimeout(400);
const lv = await page.getByRole("dialog").first();
const items = await lv.locator("ul.leave li").allInnerTexts();
ok(items.length === 2 && items.some((x) => /Unbenannt/.test(x) && /geht ohne Speichern verloren/.test(x)) && items.some((x) => x.includes(a.split("/").pop())), "Beenden listet jeden ungespeicherten Reiter einzeln auf, unbenannte mit Hinweis: " + items.join(" | "));
await snap("sitzung-4-beenden-liste");
await lv.getByRole("button", { name: "Abbrechen" }).click(); await page.waitForTimeout(300);
ok((await page.getByRole("dialog").count()) === 0 && (await tabs()).length >= 2, "Abbrechen bleibt im Programm, nichts geht verloren");
// 6) Verwerfen löscht die Zwischenkopie
await page.evaluate(() => { void window.__vc.runCommand("projekt_schliessen"); }); await page.waitForTimeout(400);
await page.getByRole("dialog").first().getByRole("button", { name: "Verwerfen" }).click(); await page.waitForTimeout(800);
ok(!existsSync(join(cfg, "scratch.json")) || (lies("scratch.json")?.entries ?? []).length === 0, "Verwerfen löscht die Zwischenkopie");
// 7) Absturzsicherung abschaltbar
await page.evaluate(async () => { await window.__vc.call("scratch_pref", { on: false }); });
await oeffne(); await page.evaluate(() => { void window.__vc.runCommand("neuer_reiter"); }); await page.waitForTimeout(300);
await page.locator(".cm-content").click(); await page.keyboard.type("nop"); await page.waitForTimeout(2300);
ok(!existsSync(join(cfg, "scratch.json")), "Ausgeschaltet: keine Zwischenkopie");
await page.evaluate(async () => { await window.__vc.call("scratch_pref", { on: true }); });
// 8) „Alle speichern“ beim Schließen: Namensabfrage für den unbenannten Reiter, danach wird wirklich geschlossen (Befund Handtest E1.4)
await oeffne(); await page.evaluate(() => { void window.__vc.runCommand("neuer_reiter"); }); await page.waitForTimeout(300);
await page.locator(".cm-content").click(); await page.keyboard.type("nop"); await page.waitForTimeout(200);
await page.evaluate(async (r) => { await window.__vc.openFile(r); }, a); await page.locator(".cm-content").click(); await page.keyboard.type(" "); await page.waitForTimeout(300);
await page.evaluate(() => { void window.__vc.runCommand("projekt_schliessen"); }); await page.waitForTimeout(400);
await page.getByRole("dialog").first().getByRole("button", { name: "Alle speichern" }).click(); await page.waitForTimeout(500);
const nd = page.getByRole("dialog").first();
ok(/Speichern unter/.test(await nd.innerText().catch(() => "")), "Für den unbenannten Reiter kommt die Namensabfrage");
await page.locator("#nameinp").fill("zzz-quit.asm"); await page.locator("#nameinp").press("Enter"); await page.waitForTimeout(700);
// weitere unbenannte Reiter fragen der Reihe nach
for (let i = 0; i < 4 && (await page.locator("#nameinp").count()); i++) { await page.locator("#nameinp").fill(`zzz-quit-${i}.asm`); await page.locator("#nameinp").press("Enter"); await page.waitForTimeout(700); }
await page.waitForTimeout(500);
ok(await page.evaluate(() => !window.__vc.ui.project), "Nach „Alle speichern“ und Namen wird das Projekt geschlossen: " + await page.evaluate(() => JSON.stringify({ t: window.__vc.ed.tabs.map((x) => [x.rel, x.dirty, x.scratch]), l: !!window.__vc.ed.leaving, p: !!window.__vc.ed.prompt })));
ok(existsSync(join(proj, "zzz-quit.asm")) && /nop/.test(readFileSync(join(proj, "zzz-quit.asm"), "utf8")), "… und die Datei ist geschrieben");
// 9) Abbruch der Namensabfrage bricht das Schließen ab
await oeffne(); await page.evaluate(() => { void window.__vc.runCommand("neuer_reiter"); }); await page.waitForTimeout(300);
await page.locator(".cm-content").click(); await page.keyboard.type("nop"); await page.waitForTimeout(200);
await page.evaluate(() => { void window.__vc.runCommand("projekt_schliessen"); }); await page.waitForTimeout(400);
await page.getByRole("dialog").first().getByRole("button", { name: "Alle speichern" }).click(); await page.waitForTimeout(500);
await page.keyboard.press("Escape"); await page.waitForTimeout(500);
ok(await page.evaluate(() => !!window.__vc.ui.project) && (await tabs()).some((x) => /Unbenannt/.test(x)), "Abbrechen der Namensabfrage: Projekt bleibt offen, Reiter bleibt");
ok(csp.length === 0, "Keine Meldungen der Inhaltsrichtlinie: " + csp.slice(0, 2).join(" | "));
console.log(`\n${n} Prüfungen, ${fails.length} Fehler`);
await browser.close();
process.exit(fails.length ? 1 : 0);
