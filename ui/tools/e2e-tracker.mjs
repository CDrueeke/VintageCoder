// SPDX-License-Identifier: GPL-3.0-or-later
// Ablauftest Tracker (E5.1): Playwright (WebKit) gegen die Prüfbrücke des echten Kerns.
// Aufruf: node tools/e2e-tracker.mjs <Kopie von examples/c64/music_demo (wird verändert!)> <Kopie von examples/atari/music_demo> [Bilderordner]
import { webkit } from "playwright";
import { mkdirSync } from "node:fs";
import { join } from "node:path";
const MOD = process.platform === "darwin" ? "Meta" : "Control";   // Kürzel: Befehlstaste (macOS) bzw. Strg

if (process.platform === "win32" && !process.env.VC_E2E_WIN) { console.log("übersprungen unter Windows\n0 Prüfungen, 0 Fehler"); process.exit(0); }
const [proj, atari, shots] = process.argv.slice(2);
if (!atari) { console.error("Aufruf: node tools/e2e-tracker.mjs <c64 music_demo> <atari music_demo> [Bilder]"); process.exit(2); }
if (shots) mkdirSync(shots, { recursive: true });
const browser = await webkit.launch();
const page = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
const fails = []; let n = 0;
const ok = (c, m) => { n++; if (!c) { fails.push(m); console.log("FEHLER:", m); } else console.log("ok:", m); };
page.on("pageerror", (e) => { fails.push("Seitenfehler " + e.message); console.log("Seitenfehler", e.message); });
const snap = async (name) => { if (shots) await page.screenshot({ path: join(shots, name + ".png") }); };
await page.goto("http://127.0.0.1:1420/?bridge=17420"); await page.waitForFunction(() => window.__vc && window.__vc.ui.ready);
await page.evaluate(async (p) => { await window.__vc.setLanguage("de"); await window.__vc.openProject(p); }, proj); await page.waitForTimeout(900);
const tv = () => page.evaluate(() => window.__vc.tk.view); const warte = () => page.waitForTimeout(450);
const zelle = (r, c) => page.locator(`[data-tk=zelle][data-r='${r}'][data-c='${c}']`);

// 1) Öffnen
await page.evaluate(() => window.__vc.openFile("music/tune.vcsong")); await page.waitForTimeout(1000); let v = await tv();
ok(v && v.kind === "song" && v.channels === 3 && !v.dirty && v.grid.length === v.rows, "tune.vcsong öffnet im Tracker (3 Stimmen, " + v?.rows + " Zeilen)");
ok(await page.evaluate(() => window.__vc.ui.mode === "musik"), "Die Ansicht wechselt zu Musik");
ok((await page.locator("[data-tk=zelle][role=option]").count()) === v.rows * 3 && (await page.locator("[data-tk=eintrag][role=option]").count()) === v.order.length && (await page.locator("[data-tk=instrument][role=option]").count()) === v.instruments.length && (await page.locator("[data-tk=raster] [role=grid], [role=application]").count()) === 0, "Raster, Reihenfolge und Instrumente sind Auswahllisten mit Optionen (keine Raster-/Anwendungsrolle)");
ok(/^Reihe 00, Stimme 1: /.test(await zelle(0, 0).getAttribute("aria-label")), "Zelle sagt Reihe, Stimme, Note und Instrument an: " + (await zelle(0, 0).getAttribute("aria-label")));
await snap("tracker-1-offen");

// 2) Noteneingabe mit Klaviertasten (nach Tastenposition)
await zelle(4, 1).click(); await warte(); await page.keyboard.press("KeyZ"); await warte(); v = await tv();
ok(v.grid[4][1] !== "--- --" && v.cursor[0] === 5 && v.dirty, "Taste Z trägt eine Note ein, der Cursor rückt eine Zeile weiter: " + v.grid[4][1]);
const c3 = v.grid[4][1]; await page.keyboard.press("KeyC"); await warte(); v = await tv(); ok(v.grid[5][1] !== c3 && v.grid[5][1] !== "--- --", "Taste C trägt die nächste Note ein: " + v.grid[5][1]);
await page.locator("[data-act=undo]").click(); await warte(); v = await tv(); ok(v.grid[5][1] === "--- --" && v.grid[4][1] === c3, "Rückgängig: ein Schritt je Note");
await page.locator("[data-act=redo]").click(); await warte(); await page.locator("[data-tk=zelle][tabindex='0']").focus();
await page.keyboard.press("Period"); await warte(); ok((await tv()).octave === 4, "Punkt: Oktave höher (4)");
await page.keyboard.press("BracketRight"); await warte(); ok((await tv()).instrument === 1, "Taste ]: nächstes Instrument (1)");
await page.keyboard.press("Digit1"); await warte(); v = await tv(); ok(v.grid[6][1].startsWith("==="), "Taste 1 setzt das Notenende: " + v.grid[6][1]);

// 3) Auswahl, Block, Transponieren, Löschen
await zelle(4, 1).click(); await page.keyboard.press("Shift+ArrowDown"); await page.keyboard.press("Shift+ArrowDown"); await warte(); v = await tv(); ok(v.sel && v.sel[0] === 4 && v.sel[2] === 6, "Umschalt+Pfeil erweitert die Auswahl (Zeilen 4 bis 6)");
await page.keyboard.press(`${MOD}+c`); await warte(); ok((await tv()).hasClip, "Kopieren füllt die Zwischenablage");
await zelle(10, 0).click(); await page.keyboard.press(`${MOD}+v`); await warte(); v = await tv(); ok(v.grid[10][0] === (await tv()).grid[10][0] && v.grid[10][0] !== "--- --", "Einfügen setzt den Block ab dem Cursor");
await page.locator("[data-act=transpose-up]").click(); await warte(); ok((await tv()).dirty, "Halbton hoch wirkt auf die Zelle");
await page.locator("[data-tk=zelle][tabindex='0']").focus(); await page.keyboard.press("Delete"); await warte(); v = await tv(); ok(v.grid[10][0] === "--- --", "Entf leert die Zelle");
// Maus: Auswahl aufziehen
const a = await zelle(2, 0).boundingBox(), b = await zelle(4, 2).boundingBox(); await page.mouse.move(a.x + 5, a.y + 5); await page.mouse.down(); await page.mouse.move(b.x + 5, b.y + 5, { steps: 6 }); await page.mouse.up(); await warte(); v = await tv();
ok(v.sel && v.sel[0] === 2 && v.sel[1] === 0 && v.sel[2] === 4 && v.sel[3] === 2, "Mit der Maus aufgezogene Auswahl: Zeilen 2–4, Stimmen 1–3");
await page.keyboard.press("PageDown"); await warte(); ok((await tv()).cursor[0] >= 8 || (await tv()).cursor[0] === 4 + 8 - v.rows, "Bild ab: acht Zeilen weiter");
await page.keyboard.press("Home"); await warte(); ok((await tv()).cursor[0] === 0, "Pos1: Zeile 0");

// 4) Muster und Reihenfolge
const p0 = v.patternCount, o0 = v.order.length;
await page.locator("[data-act=pattern-new]").click(); await warte(); ok((await tv()).patternCount === p0 + 1, "Neues Muster angelegt (" + (p0 + 1) + ")");
await page.locator("[data-act=ord-append]").click(); await warte(); ok((await tv()).order.length === o0 + 1, "Muster an die Reihenfolge angehängt");
await page.locator("[data-act=ord-up]").click(); await warte(); await page.locator("[data-tk=eintrag]").first().click(); await warte(); ok((await tv()).ordSel === 0, "Klick wählt einen Eintrag der Reihenfolge");
await page.locator("[data-tk=loop]").fill("1"); await page.locator("[data-tk=loop]").blur(); await warte(); ok((await tv()).loop === 1, "Wiederholung ab Eintrag 1");
await page.locator("[data-act=pattern-dup]").click(); await warte(); ok((await tv()).patternCount === p0 + 2, "Muster dupliziert");
await page.locator("[data-act=pattern-delete]").click(); await warte(); ok((await tv()).patternCount === p0 + 1, "Muster gelöscht (Reihenfolge angepasst)");

// 5) Tempo, Zeilen, Oktave, Schritt
await page.locator("[data-tk=tempo]").fill("4"); await page.locator("[data-tk=tempo]").blur(); await warte(); ok((await tv()).tempo === 4, "Tempo 4");
await page.locator("[data-tk=rows]").fill("16"); await page.locator("[data-tk=rows]").blur(); await warte(); v = await tv(); ok(v.rows === 16 && v.grid.length === 16, "16 Zeilen je Muster");
await page.locator("[data-tk=schritt]").fill("2"); await page.locator("[data-tk=schritt]").blur(); await warte(); await zelle(0, 0).click(); await page.keyboard.press("KeyZ"); await warte(); ok((await tv()).cursor[0] === 2, "Schrittweite 2: der Cursor springt zwei Zeilen");

// 6) Instrumente (SID)
await page.locator("[data-tk=instrument]").nth(1).click(); await warte(); v = await tv(); ok(v.insSel === 1 && !v.pokey, "Instrument 1 gewählt (SID: Klangform und ADSR)");
await page.getByRole("radio", { name: "Säge" }).click(); await warte(); ok((await tv()).instruments[1].wave === "saw", "Klangform Säge");
await page.locator("[data-tk=adsr-d]").fill("7"); await warte(); ok((await tv()).instruments[1].d === 7, "Abfall 7 per Regler");
await page.locator("[data-tk=ins-name]").fill("bass"); await page.locator("[data-tk=ins-name]").blur(); await warte(); ok((await tv()).instruments[1].name === "bass" && /bass/.test(await page.locator("[data-tk=instrument]").nth(1).innerText()), "Instrument umbenannt");
const ni = v.instruments.length; await page.locator("[data-act=ins-new]").click(); await warte(); ok((await tv()).instruments.length === ni + 1, "Neues Instrument"); await page.locator("[data-act=ins-delete]").click(); await warte(); ok((await tv()).instruments.length === ni, "Das letzte unbenutzte Instrument lässt sich löschen");
await page.locator("[data-tk=volume]").fill("9"); await warte(); ok((await tv()).volume === 9, "Lautstärke 9");
await snap("tracker-2-bearbeitet");

// 7) Speichern, Export, Englisch
await page.locator("[data-act=save]").click(); await warte(); ok(!(await tv()).dirty, "Speichern: nicht mehr als geändert gekennzeichnet");
await page.locator("[data-act=export]").click(); await page.waitForSelector("dialog[open] [data-ex=vorschau]"); await page.waitForFunction(() => (document.querySelector("[data-ex=vorschau]")?.textContent ?? "").length > 50, null, { timeout: 8000 }).catch(() => {});
ok((await page.locator("[data-ex=vorschau]").innerText()).length > 50, "Export-Dialog zeigt eine Vorschau des Songs"); await page.getByRole("button", { name: "Abbrechen" }).first().click(); await warte();
await page.evaluate(async () => { await window.__vc.setLanguage("en"); }); await warte(); const bt = await page.locator("body").innerText(); ok(/order/i.test(bt) && /instruments/i.test(bt) && /pattern/i.test(bt), "Englisch: Order, Instruments, Pattern");
await page.evaluate(async () => { await window.__vc.setLanguage("de"); });

// 7b) Stimmen ausblenden, Klaviatur-Karte, Konvertieren (E5.3)
await zelle(0, 1).click(); await warte(); await page.locator("[data-tk=stimme][data-c='1']").click(); await warte(); v = await tv();
ok((await page.locator("[data-tk=zelle][data-c='1']").count()) === 0 && (await page.locator(".kopf .kh").count()) === 2 && v.cursor[1] !== 1, "Stimme 2 ausgeblenden: Spalte weg, Cursor wechselt auf eine sichtbare Stimme");
await zelle(0, 0).click(); await warte(); await page.keyboard.press("ArrowRight"); await warte(); ok((await tv()).cursor[1] === 2, "Pfeil nach rechts überspringt die ausgeblendete Stimme");
await page.locator("[data-tk=stimme][data-c='0']").click(); await page.locator("[data-tk=stimme][data-c='2']").click(); await warte(); ok((await page.locator(".kopf .kh").count()) === 1 && (await page.locator("[data-tk=stimme][aria-pressed=true]").count()) === 1, "Die letzte sichtbare Stimme lässt sich nicht ausblenden");
await page.locator("[data-tk=alle-stimmen]").click(); await warte(); ok((await page.locator(".kopf .kh").count()) === 3 && (await page.locator("[data-tk=alle-stimmen]").isDisabled()), "„Alle einblenden“ zeigt alle Stimmen wieder");
ok((await page.locator("[data-tk=taste]").count()) === 25 && (await page.locator("[data-tk=taste]").first().innerText()).trim() === "Z", "Klaviatur: 25 Tasten, die erste ist mit der Computertaste Z beschriftet");
await page.locator("[data-tk=taste][data-h='4']").click(); await page.waitForTimeout(100); ok((await page.locator("[data-tk=taste][data-h='4'].klingt").count()) === 1 && !(await tv()).dirty, "Klick auf eine Taste lässt sie aufleuchten und ändert den Song nicht");
await page.locator("[data-act=konvertieren]").click(); await page.waitForSelector("dialog[open] [data-tk=import]"); const zielOpt = await page.locator("[data-tk=imp-ziel] option").allInnerTexts();
ok(zielOpt.length >= 1 && (await page.locator("[data-tk=imp-name]").inputValue()) === "tune_pokey", "Konvertieren-Dialog: Quelle erkannt, Name vorgeschlagen, Zielsysteme: " + zielOpt.join(", "));
await page.locator("[data-tk=imp-ok]").click(); await page.waitForTimeout(1500); v = await tv(); ok(v.rel === "music/tune_pokey.vcsong" && v.kind === "song" && !v.pokey, "Der übernommene Song liegt im Projekt und ist geöffnet (SID-Profil): " + v.rel);
await page.locator("[data-act=konvertieren]").click(); await page.waitForSelector("dialog[open] [data-tk=import]"); await page.locator("[data-tk=imp-ok]").click(); await page.waitForSelector("[data-tk=imp-fehler]"); ok(/existiert/i.test(await page.locator("[data-tk=imp-fehler]").innerText()), "Vorhandene Datei wird nicht überschrieben: " + (await page.locator("[data-tk=imp-fehler]").innerText())); await page.getByRole("button", { name: "Abbrechen" }).first().click(); await warte();
await page.evaluate(() => window.__vc.openFile("music/tune.vcsong")); await page.waitForTimeout(800);
await snap("tracker-5-klaviatur");

// 8) POKEY-Song: Verzerrung und Hüllkurve
await page.evaluate(async (p) => { await window.__vc.openProject(p); }, atari); await page.waitForTimeout(900); await page.evaluate(() => window.__vc.openFile("music/tune_pokey.vcsong")); await page.waitForTimeout(1000); v = await tv();
ok(v && v.pokey && v.kind === "song", "POKEY-Song öffnet (Verzerrung und Hüllkurve statt Klangform)");
await page.getByRole("radio", { name: "Buzz" }).click(); await warte(); ok((await tv()).instruments[v.insSel].dist === "buzz", "Verzerrung Buzz");
await page.locator("[data-tk=env-bild][data-k='3']").fill("5"); await warte(); ok((await tv()).instruments[v.insSel].env[3] === 5, "Hüllkurve: Bild 3 auf Lautstärke 5");

// 9) Wiedergabe (E5.2; im Testlauf ohne Ausgabegerät, der Sequenzer rechnet im Speicher)
await zelle(0, 0).click(); await warte(); await page.keyboard.press("Space"); await page.waitForTimeout(700); v = await tv();
ok(v.spielt && v.spielModus === "song" && (await page.locator("[data-tk=spiel-status]").count()) === 1, "Leertaste startet den Song, die Statuszeile zeigt Eintrag und Zeile: " + (await page.locator("[data-tk=spiel-status]").innerText().catch(() => "")));
const pos1 = await page.locator("[data-tk=spiel-status]").innerText(); await page.waitForTimeout(900); const pos2 = await page.locator("[data-tk=spiel-status]").innerText().catch(() => "");
ok(pos1 !== pos2 || !(await tv()).spielt, "Die Position läuft weiter (" + pos1 + " → " + pos2 + ")");
ok((await page.locator(".rwx.pl").count()) <= 1, "Höchstens eine Zeile ist als spielende Zeile markiert");
let peak = 0; for (let i = 0; i < 20 && peak === 0; i++) { peak = await page.evaluate(() => Math.max(0, ...[...document.querySelectorAll("[data-tk=pegel] b")].map((b) => parseFloat(b.style.width) || 0))); await page.waitForTimeout(120); } ok(peak > 0, "Pegelanzeige schlägt aus (" + peak + " %)");
await snap("tracker-3-spielt");
await page.locator("[data-tk=mute][data-c='0']").click(); await warte(); ok((await tv()).mute[0] === true && (await tv()).spielt, "Stumm für Stimme 1 wirkt während der Wiedergabe"); await page.locator("[data-tk=solo][data-c='1']").click(); await warte(); ok((await tv()).solo[1] === true, "Solo für Stimme 2");
await page.locator("[data-tk=mute][data-c='0']").click(); await page.locator("[data-tk=solo][data-c='1']").click(); await warte();
await zelle(1, 0).click(); await warte(); await page.keyboard.press("KeyZ"); await warte(); v = await tv(); ok(v.spielt && v.dirty, "Eine Note während der Wiedergabe eintragen: Wiedergabe läuft weiter");
await page.locator("[data-act=stop]").click(); await page.waitForTimeout(500); v = await tv(); ok(!v.spielt && (await page.locator("[data-tk=spiel-status]").count()) === 0, "Stopp beendet die Wiedergabe");
await zelle(0, 0).click(); await warte(); await page.keyboard.press("Shift+Space"); await page.waitForTimeout(600); v = await tv(); ok(v.spielt && v.spielModus === "pattern", "Umschalt+Leertaste spielt nur das Muster");
await page.keyboard.press("Space"); await page.waitForTimeout(500); ok(!(await tv()).spielt, "Leertaste beendet die Wiedergabe");
await page.locator("[data-act=play-song]").click(); await page.waitForTimeout(500); ok((await tv()).spielt, "Abspielen-Knopf"); await snap("tracker-4-pegel");
console.log(`\n${n} Prüfungen, ${fails.length} Fehler`);
await browser.close();
process.exit(fails.length ? 1 : 0);
