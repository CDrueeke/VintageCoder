// SPDX-License-Identifier: GPL-3.0-or-later
// Ablauftest Turbo BASIC XL (E3): Start ohne Diskette (Banner), Assistent „Diskette bereitstellen“ (Prüfung, Verkleinern, Kopie, .gitignore), Karte in den Projekt-Einstellungen,
// Projektbaum-Marke, Starten mit Diskette, Debuggen ausgegraut, Rückfrage bei nicht erkannter Diskette, Befehlspalette.
// Aufruf: node tools/e2e-turbo.mjs <Projektkopie turbo_basic> <Turbo-Diskette (Kopie)> <fremde Diskette (nicht Turbo)> [Bilder]
//   Kern mit HOME=<leer> VC_TEST_PICK_ATR=<Turbo-Diskette> VC_TESTBRIDGE=17420 VC_TEST_NO_SPAWN=1 VC_NO_SAVE=1
import { webkit } from "playwright";
import { existsSync, mkdirSync, readFileSync } from "node:fs";
import { join } from "node:path";

const [proj, disk, fremd, shots] = process.argv.slice(2);
if (!proj || !disk || !fremd) { console.error("Aufruf: node tools/e2e-turbo.mjs <Projekt> <Turbo-Diskette> <fremde Diskette> [Bilder]"); process.exit(2); }
if (shots) mkdirSync(shots, { recursive: true });
const browser = await webkit.launch();
const page = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
const fails = []; let n = 0; const csp = [];
const ok = (c, m) => { n++; if (!c) { fails.push(m); console.log("FEHLER:", m); } else console.log("ok:", m); };
page.on("pageerror", (e) => { fails.push("Seitenfehler " + e.message); console.log("Seitenfehler", e.message); });
page.on("console", (m) => { if (/Content Security Policy|Refused to/i.test(m.text())) csp.push(m.text()); });
const snap = async (name) => { if (shots) await page.screenshot({ path: join(shots, name + ".png") }); };
await page.goto(`http://127.0.0.1:${process.env.VC_VITE ?? 1420}/?bridge=${process.env.VC_BRIDGE ?? 17420}`); await page.waitForFunction(() => window.__vc && window.__vc.ui.ready);
await page.evaluate(async () => { await window.__vc.setLanguage("de"); }); await page.waitForTimeout(400);
await page.evaluate(async (p) => { await window.__vc.openProject(p); }, proj); await page.waitForTimeout(600);
const status = () => page.locator("footer.status").innerText();
const dlg = page.getByRole("dialog");

// 1 Zustand ohne Diskette, Karte, Debuggen
ok(await page.evaluate(() => window.__vc.ui.project.turbo === true), "Projekt als Turbo-BASIC-XL-Projekt erkannt");
ok(/Turbo-Diskette fehlt/.test(await status()), "Statuszeile: Turbo-Diskette fehlt");
await page.evaluate(() => { void window.__vc.runCommand("nav:projekt:diskette"); }); await page.waitForSelector("[data-testid=turbo-card]");
ok(/diskette fehlt/i.test(await page.getByTestId("turbo-zustand").innerText()), "Karte: Zustand „Diskette fehlt“");
ok(await page.getByTestId("turbo-card").getByText(/gehört nicht zum Lieferumfang von VintageCoder/).count() >= 1, "Karte: Hinweis „nicht im Lieferumfang“ (Rechtstext)");
ok(await page.getByTestId("turbo-pruefen").isDisabled() && await page.getByTestId("turbo-diskeditor").isDisabled(), "Prüfen und Disk-Editor sind ohne Diskette aus");
await snap("turbo-1-karte-leer");
await page.evaluate(() => { window.__vc.ui.extra = null; });

// 2 Starten ohne Diskette: Banner + Protokoll, Bau läuft nicht durch
await page.evaluate(() => { void window.__vc.runCommand("starten"); });
await page.waitForSelector("[data-testid=turbo-banner]", { timeout: 30000 });
ok(/Turbo-BASIC-XL-Diskette/.test(await page.getByTestId("turbo-banner").innerText()), "Starten ohne Diskette: Banner „Diskette fehlt“ mit Hinweis");
ok(await page.getByTestId("turbo-banner-bereitstellen").count() === 1, "… mit Knopf „Bereitstellen …“");
await snap("turbo-2-banner");
await page.getByTestId("turbo-banner").getByRole("button", { name: /Banner schließen/ }).click();
ok(await page.getByTestId("turbo-banner").count() === 0, "Banner lässt sich schließen");
await page.evaluate(() => { void window.__vc.runCommand("starten"); }); await page.waitForSelector("[data-testid=turbo-banner]");

// 3 Assistent über das Banner
await page.getByTestId("turbo-banner-bereitstellen").click(); await dlg.waitFor();
ok(/Turbo-BASIC-Diskette bereitstellen/.test(await dlg.innerText()), "Assistent öffnet");
ok(/gehört nicht zum Lieferumfang von VintageCoder\. Besorge dir eine Diskette/.test(await dlg.innerText()), "… zeigt den Rechtstext aus dem Katalog");
ok(await dlg.getByRole("button", { name: "Kopieren" }).isDisabled(), "Kopieren ist erst nach der Prüfung möglich");
await dlg.getByRole("button", { name: /Datei wählen/ }).click(); await page.getByTestId("provide-check").waitFor();
const befund = await page.getByTestId("provide-check").innerText();
ok(/erkannt/.test(befund) && /Sektoren frei/.test(befund) && /turbo\/Turbo_Basic_XL\.atr/.test(befund) && /Original bleibt unverändert/.test(befund), "Prüfung: erkannt, freie Sektoren, Ziel, Original unverändert: " + befund.replace(/\s+/g, " ").slice(0, 160));
ok(await dlg.getByText(/Nur in der Kopie im Projekt entfallen/).count() === 1, "Einmalige Bestätigung der Verkleinerung (nur Projektkopie) wird erklärt");
ok(await dlg.getByRole("radio").count() === 2 && await dlg.getByRole("radio").first().isChecked(), "… Wahl Verkleinern ja/nein, Vorgabe „ja“");
await snap("turbo-3-assistent");
await dlg.getByRole("button", { name: "Kopieren" }).click(); await page.getByTestId("provide-done").waitFor();
ok(/liegt im Projekt/.test(await page.getByTestId("provide-done").innerText()) && /\.gitignore/.test(await dlg.innerText()), "Ergebnis: Kopie im Projekt, .gitignore ergänzt");
ok(existsSync(join(proj, "turbo", "Turbo_Basic_XL.atr")), "Kopie liegt in turbo/ mit dem Originalnamen");
ok(/turbo\/\*\.atr/.test(readFileSync(join(proj, ".gitignore"), "utf8")), ".gitignore enthält turbo/*.atr");
ok(/"turbo"/.test(readFileSync(join(proj, "turbo_basic.vcprj"), "utf8")) && /"reduced": true/.test(readFileSync(join(proj, "turbo_basic.vcprj"), "utf8")), "Projektdatei trägt turbo mit reduced");
ok(readFileSync(join(proj, "turbo", "Turbo_Basic_XL.atr")).length === readFileSync(disk).length, "Kopie hat die Größe des Abbilds");
await snap("turbo-4-fertig");
await dlg.getByRole("button", { name: "Schließen" }).click(); await page.waitForTimeout(300);
ok(await page.getByTestId("turbo-banner").count() === 0, "Banner verschwindet nach dem Bereitstellen");
ok(/Turbo-Diskette: Turbo BASIC XL.*1\.5.*Sektoren frei/.test(await status()), "Statuszeile: " + (await status()).replace(/\s+/g, " ").slice(0, 90));

// 3b Projekt kopieren: Hinweis „nicht jeder darf weitergeben“ (Kern meldet turboDisk)
const kop = await page.evaluate(async (p) => window.__vc.call("copy_run", { src: p, dir: p + "-kopien", name: "mit_diskette", withBuild: false, withGit: false }), proj).catch((e) => ({ fehler: String(e) }));
ok(kop.turboDisk === true && existsSync(join(proj + "-kopien", "mit_diskette", "turbo", "Turbo_Basic_XL.atr")), "Kopie eines Projekts mit Diskette meldet turboDisk (Export-Hinweis)");
// 4 Projektbaum-Marke
const marke = page.locator("[data-mark=turbo-disk]");
await page.evaluate(() => { window.__vc.ui.sidebar = true; });
await page.locator('[role=treeitem][data-rel=turbo]').click().catch(() => {}); await page.waitForTimeout(300);
ok(await marke.count() === 1 && /Diskette \(Anwender\)/.test(await marke.innerText()), "Projektbaum: Marke „Diskette (Anwender)“ an turbo/*.atr");
ok(/Nicht einchecken/.test(await marke.getAttribute("title")), "… mit Hinweis „nicht einchecken“");
await snap("turbo-5-baum");

// 5 Karte mit Diskette
await page.evaluate(() => { void window.__vc.runCommand("nav:projekt:diskette"); }); await page.waitForSelector("[data-testid=turbo-card]");
ok(/diskette bereit/i.test(await page.getByTestId("turbo-zustand").innerText()), "Karte: Zustand „Diskette bereit“");
const karte = await page.getByTestId("turbo-card").innerText();
ok(/Turbo BASIC XL/.test(karte) && /1\.5/.test(karte) && /Sektoren frei/.test(karte) && /Verkleinert auf die Systemdateien/.test(karte), "Karte: Name, Version, freie Sektoren, verkleinert ja: " + karte.replace(/\s+/g, " ").slice(80, 260));
ok(await page.getByTestId("turbo-verkleinern").isDisabled(), "Verkleinern ist nach dem Verkleinern aus");
await page.getByTestId("turbo-pruefen").click(); await page.getByTestId("turbo-note").waitFor();
ok(/Diskette geprüft/.test(await page.getByTestId("turbo-note").innerText()), "Prüfen: Rückmeldung auf der Karte");
ok(await page.getByTestId("turbo-bereitstellen").innerText().then((x) => /Ersetzen/.test(x)), "Knopf heißt jetzt „Ersetzen …“");
// Programmname und Zusatzdateien
await page.getByTestId("turbo-programm").fill("zu langer name");
ok(await page.getByTestId("turbo-speichern").isDisabled() && (await page.locator(".err[role=alert]").count()) === 1, "Ungültiger Programmname wird gemeldet, Speichern gesperrt");
await page.getByTestId("turbo-programm").fill("start.bas");
await page.getByTestId("turbo-zusatz-neu").fill("daten/hallo.dat:HALLO.DAT"); await page.getByTestId("turbo-zusatz-neu").press("Enter");
ok(/HALLO\.DAT/.test(await page.getByTestId("turbo-zusatz").innerText()), "Zusatzdatei erscheint in der Liste");
await page.getByTestId("turbo-speichern").click(); await page.waitForTimeout(500);
const vc = readFileSync(join(proj, "turbo_basic.vcprj"), "utf8");
ok(/"program": "START\.BAS"/.test(vc) && /daten\/hallo\.dat:HALLO\.DAT/.test(vc), "Einstellungen stehen in der Projektdatei");
await snap("turbo-6-karte");
await page.getByTestId("turbo-programm").fill("AUTORUN.BAS"); await page.getByTestId("turbo-zusatz").getByRole("button", { name: /Entfernen/ }).click(); await page.getByTestId("turbo-speichern").click(); await page.waitForTimeout(400);
ok(!/"program"/.test(readFileSync(join(proj, "turbo_basic.vcprj"), "utf8")), "Standardname wird nicht gespeichert");
await page.evaluate(() => { window.__vc.ui.extra = null; });

// 6 Starten mit Diskette
await page.evaluate(() => { void window.__vc.runCommand("bauen"); });
await page.waitForFunction(() => /gebaut/.test(document.querySelector("footer.status")?.innerText ?? ""), null, { timeout: 60000 }).catch(() => {});
await page.evaluate(() => { window.__vc.ui.mode = "code"; }); await page.locator("footer.status button", { hasText: /gebaut|Bau/ }).first().click(); await page.waitForSelector(".out .l");
const log = (await page.locator(".out .l").allInnerTexts()).join("\n");
ok(/Diskette: build\/.*\.atr \(Turbo BASIC XL 1\.5\)/.test(log), "Bauen: Protokoll nennt die Diskette: " + log.split("\n").filter((l) => /Diskette/.test(l)).join(" | ").slice(0, 120));
ok(existsSync(join(proj, "build", "balken.atr")), "build/balken.atr liegt vor");
ok(/Turbo-Diskette bereitgestellt/.test(log), "Protokoll: Bereitstellung steht drin");
await page.evaluate(() => { void window.__vc.runCommand("starten"); });
await page.waitForFunction(() => document.querySelector(".out")?.innerText.includes("Gestartet") || document.querySelector(".out")?.innerText.includes("Testlauf") || document.querySelector(".out")?.innerText.includes("Emulator"), null, { timeout: 30000 }).catch(() => {});
ok(await page.getByTestId("turbo-banner").count() === 0, "Starten mit Diskette: kein Banner");
await snap("turbo-7-start");

// 7 Debuggen ausgegraut
await page.evaluate(() => { void window.__vc.runCommand("debuggen"); }); await page.waitForTimeout(300);
ok((await page.locator(".toast").allInnerTexts()).some((x) => /nicht schrittweise|noch nicht schrittweise/.test(x)), "Debuggen im Turbo-Projekt: Hinweis mit Begründung");
await page.evaluate(() => { window.__vc.ui.palette = true; }); await page.locator("[role=combobox]").fill("debuggen"); await page.waitForTimeout(200);
const optDebug = page.locator("[role=option]", { hasText: "Startprogramm debuggen" });
ok(await optDebug.getAttribute("aria-disabled") === "true" && /schrittweise/.test(await optDebug.innerText()), "Palette: „Debuggen“ ausgegraut mit Begründung");
await snap("turbo-8-palette-debug");
// 8 Befehlspalette: bereitstellen, prüfen, im Disk-Editor öffnen
await page.locator("[role=combobox]").fill("bereitstellen"); await page.waitForTimeout(200);
ok(await page.locator("[role=option]", { hasText: "Turbo-BASIC-Diskette bereitstellen" }).count() === 1, "Palette: „bereitstellen“ findet den Befehl");
await page.locator("[role=combobox]").fill("prüfen"); await page.waitForTimeout(200);
ok(await page.locator("[role=option]", { hasText: "Turbo-BASIC-Diskette prüfen" }).count() === 1, "Palette: „prüfen“ findet den Befehl");
await page.locator("[role=combobox]").fill("disk-editor"); await page.waitForTimeout(200);
ok(await page.locator("[role=option]", { hasText: "im Disk-Editor öffnen" }).count() === 1, "Palette: „im Disk-Editor öffnen“ findet den Befehl");
await page.keyboard.press("Escape");
await page.evaluate(() => { void window.__vc.runCommand("turbo_pruefen"); }); await page.waitForTimeout(400);
ok((await page.locator(".toast").allInnerTexts()).some((x) => /Diskette geprüft/.test(x)), "Befehl „prüfen“ meldet das Ergebnis");
await page.evaluate(() => { void window.__vc.runCommand("turbo_im_disk_editor"); }); await page.waitForTimeout(300);
ok(await page.evaluate(() => window.__vc.ui.mode === "daten" && window.__vc.ui.sub.daten === "disk" && window.__vc.tb.diskPending === "turbo/Turbo_Basic_XL.atr"), "Befehl „im Disk-Editor öffnen“ wechselt zu Daten ▸ Disk und merkt die Datei");
await page.evaluate(() => { window.__vc.ui.mode = "code"; });

// 9 Nicht erkannte Diskette: Rückfrage
await page.evaluate(() => { void window.__vc.runCommand("turbo_bereitstellen"); }); await dlg.waitFor();
await page.evaluate(async (f) => { await window.__vc.checkSource(f); }, fremd); await page.getByTestId("provide-check").waitFor();
const fr = await page.getByTestId("provide-check").innerText();
ok(/Trotzdem verwenden/.test(fr) && /sieht nicht nach Turbo BASIC XL aus|kein DOS-2|DOS-2/i.test(fr), "Nicht erkannt: Rückfrage „Trotzdem verwenden?“: " + fr.replace(/\s+/g, " ").slice(0, 140));
ok(await dlg.getByRole("button", { name: "Kopieren" }).isDisabled(), "… Kopieren bleibt gesperrt bis zur Entscheidung");
await snap("turbo-9-nicht-erkannt");
await dlg.getByRole("button", { name: /Ja, trotzdem verwenden/ }).click(); await page.waitForTimeout(200);
ok(await dlg.getByRole("button", { name: "Kopieren" }).isEnabled(), "… nach „Ja, trotzdem“ ist Kopieren frei");
await dlg.getByRole("button", { name: "Abbrechen" }).click(); await page.waitForTimeout(200);
ok(await dlg.count() === 0 || !(await dlg.isVisible()), "Abbrechen schließt den Assistenten ohne Änderung");
ok(!existsSync(join(proj, "turbo", "x.atr")), "Es wurde nichts kopiert");

// 10 Englisch
await page.evaluate(async () => { await window.__vc.setLanguage("en"); }); await page.waitForTimeout(500);
await page.evaluate(() => { void window.__vc.runCommand("nav:projekt:diskette"); }); await page.waitForSelector("[data-testid=turbo-card]");
const en = await page.getByTestId("turbo-card").innerText();
console.log(en.replace(/\s+/g, " ").slice(0, 900));
ok(/Turbo BASIC XL is not part of VintageCoder/.test(en) && /sectors free/.test(en) && !/Sektoren|gehört/.test(en), "Englisch: Karte vollständig englisch");
await snap("turbo-10-englisch");
await page.evaluate(async () => { await window.__vc.setLanguage("de"); });
// 11 Schmales Fenster und Hoher Kontrast: nichts läuft über, Dialog passt
await page.setViewportSize({ width: 520, height: 800 });
await page.evaluate(() => { void window.__vc.runCommand("nav:projekt:diskette"); }); await page.waitForSelector("[data-testid=turbo-card]"); await page.waitForTimeout(300);
ok(await page.evaluate(() => { const c = document.querySelector("[data-testid=projekt-einstellungen]"); return c.scrollWidth <= c.clientWidth + 1; }), "Schmal (520 px): Projekt-Einstellungen ohne waagerechtes Scrollen");
ok(await page.evaluate(() => { const c = document.querySelector("[data-testid=turbo-card]").getBoundingClientRect(); return c.right <= window.innerWidth + 1; }), "Schmal: Karte passt in die Breite");
await snap("turbo-11-schmal");
await page.emulateMedia({ forcedColors: "active" }); await page.waitForTimeout(200); await snap("turbo-12-hoher-kontrast");
ok(await page.getByTestId("turbo-card").isVisible(), "Hoher Kontrast: Karte sichtbar");
await page.emulateMedia({ forcedColors: "none" });
await page.evaluate(() => { void window.__vc.runCommand("turbo_bereitstellen"); }); await dlg.waitFor();
ok(await page.evaluate(() => { const r = document.querySelector("dialog[open]").getBoundingClientRect(); return r.left >= -1 && r.right <= window.innerWidth + 1; }), "Schmal: Assistent passt in die Breite");
await snap("turbo-13-schmal-assistent");
// Tastatur: Fokus im Dialog beginnt beim Knopf „Datei wählen“, Esc schließt
ok(await page.evaluate(() => /Datei wählen/.test(document.activeElement?.textContent ?? "")), "Tastatur: Fokus beginnt auf „Datei wählen …“");
await page.keyboard.press("Escape"); await page.waitForTimeout(200);
ok(await page.locator("dialog[open]").count() === 0, "Tastatur: Esc schließt den Assistenten");
await page.setViewportSize({ width: 1440, height: 900 });
ok(csp.length === 0, "Keine Verstöße gegen die Inhaltsrichtlinie (" + csp.length + ")");
console.log(`${n} Prüfungen, ${fails.length} Fehler`);
await browser.close();
process.exit(fails.length ? 1 : 0);
