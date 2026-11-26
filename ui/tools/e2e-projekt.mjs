// SPDX-License-Identifier: GPL-3.0-or-later
// Ablauftest Bereich „Projekt“ (P1): erster Hauptbereich, zweite Reihe mit sieben Seiten, Befehl „Projekt-Einstellungen“, Befehlspalette, Englisch, ohne Projekt.
// Aufruf: node tools/e2e-projekt.mjs <Projektordner> [Bilderordner]
import { webkit } from "playwright";
import { mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const [proj, shots] = process.argv.slice(2);
if (!proj) { console.error("Aufruf: node tools/e2e-projekt.mjs <Projektordner> [Bilder]"); process.exit(2); }
if (shots) mkdirSync(shots, { recursive: true });
const browser = await webkit.launch();
const page = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
const fails = []; let n = 0;
const ok = (c, m) => { n++; if (!c) { fails.push(m); console.log("FEHLER:", m); } else console.log("ok:", m); };
page.on("pageerror", (e) => { fails.push("Seitenfehler " + e.message); console.log("Seitenfehler", e.message); });
const snap = async (name) => { if (shots) await page.screenshot({ path: join(shots, name + ".png") }); };
await page.goto("http://127.0.0.1:1420/?bridge=17420"); await page.waitForFunction(() => window.__vc && window.__vc.ui.ready);
await page.evaluate(async () => { await window.__vc.setLanguage("de"); });
const hauptNav = () => page.locator("nav[aria-label] .seg button").allInnerTexts();
const subNav = () => page.locator("[role=group][aria-label='Unterbereiche'] button, [role=group][aria-label='Subsections'] button").allInnerTexts();
const titel = () => page.locator("[data-projekt-titel]").innerText();

// 1) Ohne Projekt: der Bereich ist da und erklärt sich
ok((await hauptNav())[0]?.trim() === "Startseite" && (await hauptNav())[1]?.trim() === "Projekt" && (await hauptNav()).length === 7, "Hauptnavigation: „Startseite“ steht ganz links, danach „Projekt“ und die sechs Bereiche: " + (await hauptNav()).map((x) => x.trim()).join(" | "));
await page.evaluate(() => window.__vc.runCommand("nav:projekt")); await page.waitForTimeout(300);
ok((await page.locator("[data-projekt=leer]").count()) === 1 && /Öffne zuerst ein Projekt/.test(await page.locator("[data-projekt=leer]").innerText()), "Ohne Projekt erklärt die Seite, was zu tun ist (nichts bleibt leer)");
ok((await subNav()).length === 7, "Zweite Reihe: sieben Seiten: " + (await subNav()).map((x) => x.trim()).join(" | "));

// 2) Mit Projekt: Seiten wechseln
await page.evaluate(async (p) => { await window.__vc.openProject(p); }, proj); await page.waitForTimeout(900);
await page.evaluate(() => window.__vc.runCommand("nav:projekt")); await page.waitForTimeout(300);
ok(await page.evaluate(() => window.__vc.ui.mode === "projekt"), "Bereich Projekt ist aktiv");
ok(/Übersicht/.test(await titel()), "Erste Seite: Übersicht (" + (await titel()) + ")");
ok((await page.locator("[role=group][aria-label='Unterbereiche'] button[aria-current=page]").innerText()).trim() === "Übersicht", "Die Übersicht ist in der zweiten Reihe als aktuelle Seite gekennzeichnet");
for (const [id, text] of [["allgemein", "Allgemein"], ["ziele", "Ziele"], ["quellen", "Quellen"], ["bauen", "Bauen und Starten"], ["diskette", "Diskette"], ["weitergeben", "Weitergeben"]]) {
  await page.locator("[role=group][aria-label='Unterbereiche'] button", { hasText: new RegExp("^\\s*" + text + "\\s*$") }).click(); await page.waitForTimeout(250);
  ok((await page.locator("[data-projekt]").first().getAttribute("data-projekt")) === id && (await titel()).includes(text), "Seite „" + text + "“ öffnet (" + (await titel()) + ")");
}
await page.evaluate(() => window.__vc.runCommand("nav:projekt:diskette")); await page.waitForTimeout(400);
ok(/ohne Turbo|Turbo/i.test(await page.locator("[data-projekt=diskette]").innerText()), "Seite Diskette: Platz der Turbo-Karte (Hinweis oder Karte)");

// 2b) Übersicht, Allgemein, Ziele mit echten Angaben (P2)
const info = () => page.evaluate(() => window.__vc.call("projekt_info"));
await page.evaluate(() => window.__vc.runCommand("nav:projekt:uebersicht")); await page.waitForSelector("[data-pj=uebersicht]"); await page.waitForTimeout(300);
const i0 = await info();
ok((await page.locator("[data-pj-karte]").count()) === 4, "Übersicht: vier Karten (Allgemein, Ziele, Quellen, Diskette)");
ok((await page.locator("[data-pj-karte=allgemein]").innerText()).includes(i0.name) && /C64/.test(await page.locator("[data-pj-karte=ziele]").innerText()), "Karten nennen Projektname und Zielsystem: " + i0.name);
ok(/in Ordnung|Hinweis|Angabe fehlt|nicht eingeschaltet/.test(await page.locator("[data-pj-karte=quellen] h3").innerText()), "Der Zustand steht als Wort in der Kartenüberschrift: " + (await page.locator("[data-pj-karte=quellen] h3").innerText()).replace(/\n/g, " "));
await page.locator("[data-pj-aendern=ziele]").click(); await page.waitForTimeout(300);
ok(await page.evaluate(() => window.__vc.ui.sub.projekt === "ziele"), "„Ändern ›“ springt zur Seite Ziele");
const z0 = i0.targets[0];
await page.locator(`[data-pj-ziel='${z0.id}'] button`, { hasText: /^\s*NTSC\s*$/ }).click(); await page.waitForTimeout(500);
ok((await info()).targets[0].variant === "ntsc", "Variante NTSC gewählt und in der Projektdatei gespeichert");
await page.locator(`[data-pj-ziel='${z0.id}'] button`, { hasText: /^\s*PAL\s*$/ }).click(); await page.waitForTimeout(500);
ok((await info()).targets[0].variant === "pal", "Variante PAL zurück gespeichert");
ok(await page.locator("[data-pj=ziel-weg]").first().isDisabled(), "Das einzige Ziel lässt sich nicht entfernen");
await page.locator("[data-pj=ziel-neu]").selectOption("atari8"); await page.locator("[data-pj=ziel-hinzu]").click(); await page.waitForTimeout(600);
ok((await info()).targets.length === 2 && (await page.locator("[data-pj-ziel='atari8']").count()) === 1, "Zielsystem Atari hinzufügen: steht in der Projektdatei und auf der Seite");
await page.locator("[data-pj-ziel='atari8'] [data-pj=ziel-weg]").click(); await page.waitForTimeout(600);
ok((await info()).targets.length === 1 && (await page.locator("[data-pj-ziel='atari8']").count()) === 0, "Zielsystem wieder entfernen");
await page.evaluate(() => window.__vc.runCommand("nav:projekt:allgemein")); await page.waitForSelector("[data-pj=allgemein]"); await page.waitForTimeout(300);
await page.locator("[data-pj-feld=name]").fill("Probe Projekt"); await page.locator("[data-pj-feld=name]").blur(); await page.waitForTimeout(600);
ok((await info()).name === "Probe Projekt" && /Gespeichert/.test(await page.locator("[data-pj=status]").innerText()), "Name geändert: gespeichert, Statuszeile meldet es");
ok((await page.locator(".pname b").innerText()).includes("Probe Projekt"), "Der Projektname in der Kopfzeile folgt sofort");
await page.locator("[data-pj-feld=name]").fill("   "); await page.locator("[data-pj-feld=name]").blur(); await page.waitForTimeout(700);
ok((await info()).name === "Probe Projekt" && (await page.locator("[data-pj-feld=name]").inputValue()) === "Probe Projekt", "Leerer Name wird abgelehnt: Datei und Anzeige behalten den alten Namen");
await page.locator("[data-pj-feld=beschreibung]").fill("Eine Beschreibung"); await page.locator("[data-pj-feld=beschreibung]").blur(); await page.waitForTimeout(500);
ok((await info()).description === "Eine Beschreibung", "Beschreibung gespeichert");
const progs = (await info()).sources.map((q) => q.file);
await page.locator("[data-pj-feld=main]").selectOption(progs[progs.length - 1]); await page.waitForTimeout(500);
ok((await info()).main === progs[progs.length - 1], "Start-Programm gewählt und gespeichert: " + progs[progs.length - 1]);
await page.locator("[data-pj-feld=main]").selectOption(""); await page.waitForTimeout(400); ok((await info()).main === null, "Wieder „Automatisch“: kein main in der Datei");
await page.locator("[data-pj-feld=name]").fill(i0.name); await page.locator("[data-pj-feld=name]").blur(); await page.waitForTimeout(400);
await page.locator("[data-pj-feld=beschreibung]").fill(i0.description); await page.locator("[data-pj-feld=beschreibung]").blur(); await page.waitForTimeout(400);
await snap("projekt-2-allgemein");

// 2c) Quellen, Bauen und Starten, Weitergeben (P3)
await page.evaluate(() => window.__vc.runCommand("nav:projekt:quellen")); await page.waitForSelector("[data-pj=quellen]"); await page.waitForTimeout(400);
const i1 = await info(); const start = i1.mainEffective;
ok((await page.locator("[data-pj-gruppe]").count()) >= 1 && (await page.locator("[data-pj-quelle]").count()) === i1.sources.length, "Quellen: Gruppen und alle Programme stehen in Baureihenfolge (" + i1.sources.length + ")");
ok(/★/.test(await page.locator(`[data-pj-quelle='${start}']`).innerText()), "Das Start-Programm trägt den Stern: " + start);
ok((await page.locator(".mw").count()) === 0, "Keine fehlenden Dateien gemeldet");
await page.locator(`[data-pj-quelle='${start}'] button.nm`).click(); await page.waitForTimeout(600);
ok(await page.evaluate(() => window.__vc.ui.mode === "code"), "Klick auf den Dateinamen öffnet die Datei im Code-Editor");
await page.evaluate(() => window.__vc.runCommand("nav:projekt:bauen")); await page.waitForSelector("[data-pj=bauen]"); await page.waitForTimeout(500);
ok((await page.locator("[data-pj-karte]").count()) === 4 && /C64/.test(await page.locator("[data-pj-ziel-emu]").first().innerText()), "Bauen und Starten: Karten Letzter Bau, Emulator je Ziel, Diskettenabbild, Pakete; das Ziel steht da");
ok(/Pakete/.test(await page.locator("[data-pj-karte=pakete]").innerText()), "Karte Pakete (Registry) ist da");
await page.locator("[data-pj=bauen-knopf]").click(); await page.waitForFunction(() => /Gebaut um/.test(document.querySelector("[data-pj=bau-ergebnis]")?.textContent ?? ""), null, { timeout: 60000 }).catch(() => {});
ok(/Gebaut um/.test(await page.locator("[data-pj=bau-ergebnis]").innerText()), "Bauen aus der Seite: das Ergebnis steht da: " + (await page.locator("[data-pj=bau-ergebnis]").innerText()));
await page.evaluate(() => window.__vc.runCommand("nav:projekt:weitergeben")); await page.waitForSelector("[data-pj=weitergeben]"); await page.waitForTimeout(400);
ok((await page.locator("[data-pj-karte]").count()) === 3, "Weitergeben: Karten Projekt kopieren, Export, Zuletzt geöffnet");
await page.locator("[data-pj=kopieren-knopf]").click(); await page.waitForTimeout(600);
ok((await page.locator("dialog[open]").count()) === 1, "„Projekt kopieren …“ öffnet den Kopieren-Dialog");
await page.keyboard.press("Escape"); await page.waitForTimeout(300);

// 3) Befehl „Projekt-Einstellungen …“ und Befehlspalette, Start mit --view
await page.evaluate(() => window.__vc.runCommand("nav:code")); await page.waitForTimeout(250);
await page.evaluate(() => { void window.__vc.runCommand("projekt_einstellungen"); }); await page.waitForTimeout(300);
ok(await page.evaluate(() => window.__vc.ui.mode === "projekt" && window.__vc.ui.sub.projekt === "weitergeben"), "Befehl „Projekt-Einstellungen …“ öffnet den Bereich bei der zuletzt benutzten Seite (Weitergeben)");
await page.evaluate(() => window.__vc.runCommand("nav:projekt:quellen")); await page.waitForTimeout(250);
ok((await page.locator("[data-projekt]").first().getAttribute("data-projekt")) === "quellen", "nav:projekt:quellen (wie --view projekt:quellen) öffnet die Seite Quellen");
await page.evaluate(() => window.__vc.runCommand("nav:code")); await page.waitForTimeout(250);
await page.keyboard.press("Control+Alt+0"); await page.keyboard.press("Meta+Alt+0"); await page.waitForTimeout(350);
ok(await page.evaluate(() => window.__vc.ui.mode === "projekt"), "Kürzel Cmd/Strg+Alt+0 öffnet den Bereich Projekt");
await page.keyboard.press("Control+k"); await page.keyboard.press("Meta+k"); await page.waitForTimeout(300);
await page.keyboard.type("Projekt"); await page.waitForTimeout(300);
ok(/Weitergeben/.test(await page.locator("body").innerText()), "Befehlspalette findet die Seiten des Bereichs Projekt");
await page.keyboard.press("Escape"); await page.waitForTimeout(200);

// 4) Tastatur: Pfeile in der zweiten Reihe, Englisch
await page.evaluate(() => window.__vc.runCommand("nav:projekt:uebersicht")); await page.waitForTimeout(300);
await page.locator("[role=group][aria-label='Unterbereiche'] button[aria-current=page]").click(); await page.keyboard.press("ArrowRight"); await page.waitForTimeout(300);   // nach einem Klick (WebKit fokussiert Knöpfe dabei nicht) wechseln die Pfeiltasten die Seite
ok(await page.evaluate(() => window.__vc.ui.sub.projekt) === "allgemein", "Pfeil rechts nach Klick auf einen Reiter wechselt zur nächsten Seite");
await page.keyboard.press("ArrowLeft"); await page.waitForTimeout(300);
await snap("projekt-1");
await page.evaluate(async () => { await window.__vc.setLanguage("en"); }); await page.waitForTimeout(400);
ok((await hauptNav())[0]?.trim() === "Home" && (await hauptNav())[1]?.trim() === "Project" && (await subNav()).map((x) => x.trim()).includes("Build and run"), "Englisch: Project, Build and run: " + (await subNav()).map((x) => x.trim()).join(" | "));
ok(!(await page.locator("body").innerText()).includes("projekt-"), "Keine Schlüssel im Text");
// P4: Quellgruppen, Ziele und Diskettenabbild bearbeiten (gespeichert wird sofort; der Kern prüft Pfade)
const pfile = () => JSON.parse(readFileSync(join(proj, readdirSync(proj).find((f) => f.endsWith(".vcprj"))), "utf8"));
await page.evaluate(() => window.__vc.runCommand("nav:projekt:quellen")); await page.waitForSelector("[data-pj=quellen]"); await page.waitForTimeout(400);
const gruppen0 = await page.locator("[data-pj-gruppe]").count();
await page.locator("[data-pj=gruppe-neu-asm]").click(); await page.waitForTimeout(500);
ok(await page.locator("[data-pj-gruppe]").count() === gruppen0 + 1 && pfile().sources.length === gruppen0 + 1, "Neue Gruppe (Assembler) erscheint und steht in der Projektdatei");
writeFileSync(join(proj, "p4_neu.asm"), "rts\n"); await page.evaluate(async () => { await window.__vc.call("tree_reload").catch(() => {}); });
const letzte = page.locator("[data-pj-gruppe]").last();
await letzte.locator("[data-pj=datei-neu]").fill("fehlt.asm"); await letzte.locator("[data-pj=datei-hinzu]").click(); await page.waitForTimeout(500);
ok(await page.locator(".toasts").innerText().then((x) => /fehlt/.test(x)), "Fehlende Datei wird abgelehnt (Meldung)");
await letzte.locator("[data-pj=datei-neu]").fill("p4_neu.asm"); await letzte.locator("[data-pj=datei-hinzu]").click(); await page.waitForTimeout(500);
ok(await page.locator("[data-pj-quelle='p4_neu.asm']").count() === 1 && pfile().sources.at(-1).files.includes("p4_neu.asm"), "Datei wird der Gruppe hinzugefügt und gespeichert");
await page.locator("[data-pj-weg='p4_neu.asm']").click(); await page.waitForTimeout(400);
ok(await page.locator("[data-pj-quelle='p4_neu.asm']").count() === 0, "Datei lässt sich aus den Quellen entfernen");
await letzte.locator("[data-pj=gruppe-art]").selectOption("basic"); await page.waitForTimeout(400);
ok(pfile().sources.at(-1).kind === "basic", "Art der Gruppe auf BASIC gestellt");
await letzte.locator("[data-pj=gruppe-weg]").click(); await page.waitForTimeout(400);
ok(await page.locator("[data-pj-gruppe]").count() === gruppen0, "Gruppe lässt sich entfernen");
await page.evaluate(() => window.__vc.runCommand("nav:projekt:ziele")); await page.waitForSelector("[data-pj=ziele]"); await page.waitForTimeout(400);
await page.locator("[data-pj=illegale]").first().selectOption("deny"); await page.waitForTimeout(400);
ok(pfile().targets[0].illegals === "deny", "Undokumentierte Befehle: Fehler eingestellt");
await page.locator("[data-pj=startziel]").first().selectOption("@emulator"); await page.waitForTimeout(400);
ok(pfile().targets[0].start_on === "@emulator", "Startziel des Projekts: Emulator");
await page.locator("[data-pj=illegale]").first().selectOption(""); await page.locator("[data-pj=startziel]").first().selectOption(""); await page.waitForTimeout(400);
ok(!pfile().targets[0].illegals && !pfile().targets[0].start_on, "Zurück auf Standard entfernt die Einträge");
await page.evaluate(() => window.__vc.runCommand("nav:projekt:bauen")); await page.waitForSelector("[data-pj=bauen]"); await page.waitForTimeout(400);
if (await page.locator("[data-pj=abbild-weg]").count()) { await page.locator("[data-pj=abbild-weg]").click(); await page.waitForTimeout(300); }
await page.locator("[data-pj=abbild-neu][data-f=d81]").click(); await page.waitForTimeout(500);
ok(pfile().disk?.format === "d81", "Diskettenabbild D81 einrichten");
await page.locator("[data-pj=abbild-name]").fill("MEINE"); await page.locator("[data-pj=abbild-name]").blur(); await page.waitForTimeout(400);
ok(pfile().disk?.name === "MEINE", "Name auf der Diskette gespeichert");
await page.locator("[data-pj=abbild-weg]").click(); await page.waitForTimeout(400);
ok(!pfile().disk, "„Kein Abbild“ entfernt den Abschnitt");
// Zoom größer auch mit „+“ und „=“ (deutsche Tastatur)
{ const z0 = await page.evaluate(() => window.__vc.ui.zoom); await page.evaluate(() => window.dispatchEvent(new KeyboardEvent("keydown", { key: "+", code: "BracketRight", metaKey: true, bubbles: true, cancelable: true }))); await page.waitForTimeout(250);
  const z1 = await page.evaluate(() => window.__vc.ui.zoom); ok(z1 > z0, "Cmd und „+“ vergrößern den Zoom (" + z0 + " → " + z1 + ")"); await page.evaluate(() => window.__vc.runCommand("zoom_null")); }
await page.evaluate(async () => { await window.__vc.setLanguage("de"); });
console.log(`\n${n} Prüfungen, ${fails.length} Fehler`);
await browser.close();
process.exit(fails.length ? 1 : 0);
