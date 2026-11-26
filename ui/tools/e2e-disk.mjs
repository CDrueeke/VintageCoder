// SPDX-License-Identifier: GPL-3.0-or-later
// Ablauftest Disk-Editor (E2.1): Playwright (WebKit) gegen die Prüfbrücke des echten Kerns.
// Aufruf: node tools/e2e-disk.mjs <Projektordner mit test.d64, test.d81, kaputt.d64> [Bilderordner]
import { webkit } from "playwright";
import { existsSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
const MOD = process.platform === "darwin" ? "Meta" : "Control";   // Kürzel: Befehlstaste (macOS) bzw. Strg

const [proj, shots] = process.argv.slice(2);
if (!proj) { console.error("Aufruf: node tools/e2e-disk.mjs <Projektordner> [Bilder]"); process.exit(2); }
if (shots) mkdirSync(shots, { recursive: true });
const browser = await webkit.launch();
const page = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
const fails = []; let n = 0; const csp = [];
const ok = (c, m) => { n++; if (!c) { fails.push(m); console.log("FEHLER:", m); } else console.log("ok:", m); };
page.on("pageerror", (e) => { fails.push("Seitenfehler " + e.message); console.log("Seitenfehler", e.message); });
page.on("console", (m) => { if (/Content Security Policy|Refused to/i.test(m.text())) csp.push(m.text()); });
const snap = async (name) => { if (shots) await page.screenshot({ path: join(shots, name + ".png") }); };
await page.goto("http://127.0.0.1:1420/?bridge=17420"); await page.waitForFunction(() => window.__vc && window.__vc.ui.ready);
await page.evaluate(async (p) => { await window.__vc.setLanguage("de"); await window.__vc.openProject(p); }, proj); await page.waitForTimeout(900);
const dv = () => page.evaluate(() => window.__vc.dk.view);

// 1) D64 aus dem Projektbaum öffnen
await page.evaluate(() => window.__vc.openFile("test.d64")); await page.waitForTimeout(700);
let v = await dv();
ok(v && v.format === "D64" && v.files.length === 3 && v.cells.length === 35, "test.d64 öffnet im Disk-Editor: " + v?.files.map((f) => f.name).join(", "));
ok(await page.evaluate(() => window.__vc.ui.mode === "daten" && window.__vc.ui.sub.daten === "disk"), "Die Ansicht wechselt zu Daten ▸ Disk");
ok((await page.locator(".fmt.on").innerText()) === "D64" && (await page.locator("tbody tr").count()) === 3, "Format D64 gekennzeichnet, drei Zeilen im Verzeichnis");
ok(/18\/0/.test(await page.locator(".col.r h3").innerText()) && /Kopf/.test(await page.locator(".col.r").innerText()), "Zu Anfang zeigt der Inspektor Kopf/BAM (18/0): " + (await page.locator(".col.r h3").innerText()));
ok(await page.locator(".hex .hr").count() === 32, "Sektor als Hex: 32 Zeilen zu 8 Bytes");
await snap("disk-1-d64");

// 1b) Verzeichnisreihenfolge: Ziehen mit Einfügemarke, Knöpfe, Alt+Pfeil, Rückgängig (Cmd+Z), Speichern
{
  const namen = async () => (await dv()).files.map((f) => f.name).join(","); const o0 = await namen(); const [a, b, c] = o0.split(",");
  const zeile = (i) => page.locator("tbody tr[data-dk=row]").nth(i);
  ok(!(await page.locator("[data-dk=hoch]").isEnabled()) && (await page.locator("[data-act=undo]").isDisabled()), "Ohne Auswahl: Nach oben gesperrt, Rückgängig leer");
  // Ziehen: erste Zeile auf die dritte; währenddessen steht die Einfügemarke unter der Zielzeile
  const r0 = await zeile(0).locator("td.nm").boundingBox(), r2 = await zeile(2).locator("td.nm").boundingBox();
  await page.mouse.move(r0.x + 20, r0.y + r0.height / 2); await page.mouse.down(); await page.mouse.move(r2.x + 20, r2.y + r2.height / 2, { steps: 8 }); await page.waitForTimeout(150);
  ok(await page.locator("tr.ueber-unten").count() === 1 && (await page.locator("tr.ueber-unten").getAttribute("data-i")) === "2", "Beim Ziehen von oben nach unten zeigt eine Marke unter der Zielzeile die Stelle");
  await page.mouse.up(); await page.waitForTimeout(500);
  ok((await namen()) === [b, c, a].join(",") && (await page.locator("tr.ueber-unten, tr.ueber-oben").count()) === 0, "Ziehen ordnet das Verzeichnis um und löscht die Marke: " + await namen());
  v = await dv(); ok(v.dirty && v.canUndo && v.selFile === null, "Die Diskette gilt als geändert, Rückgängig ist frei");
  await page.keyboard.press(`${MOD}+z`); await page.waitForTimeout(400);
  ok((await namen()) === o0 && !(await dv()).dirty, "Cmd+Z macht den Zug in einem Schritt rückgängig (wieder sauber)");
  await page.keyboard.press(`${MOD}+Shift+z`); await page.waitForTimeout(400); ok((await namen()) === [b, c, a].join(","), "Cmd+Umschalt+Z stellt ihn wieder her");
  await page.locator("[data-act=undo]").click(); await page.waitForTimeout(300); ok((await namen()) === o0, "Der Knopf Rückgängig geht ebenso");
  // Knöpfe und Tastatur
  await zeile(0).click(); await page.waitForTimeout(200); await page.locator("[data-dk=runter]").click(); await page.waitForTimeout(400);
  ok((await namen()) === [b, a, c].join(",") && (await dv()).selFile === 1, "„Nach unten“ rückt die gewählte Datei eine Zeile weiter, die Auswahl folgt");
  await zeile(1).focus(); await page.keyboard.press("Alt+ArrowDown"); await page.waitForTimeout(400); ok((await namen()) === [b, c, a].join(",") && (await dv()).selFile === 2, "Alt+Pfeil runter ordnet per Tastatur um");
  await page.keyboard.press("Alt+ArrowUp"); await page.waitForTimeout(400); ok((await namen()) === [b, a, c].join(","), "Alt+Pfeil hoch ordnet zurück");
  ok(await page.locator("[data-dk=hoch]").isEnabled(), "Nach oben ist frei, solange die Auswahl nicht in der ersten Zeile steht");
  for (let i = 0; i < 3; i++) { await page.keyboard.press(`${MOD}+z`); await page.waitForTimeout(250); }
  ok((await namen()) === o0 && !(await dv()).dirty, "Alle Züge lassen sich einzeln zurücknehmen: " + await namen());
  // Vormerkung wandert mit
  await page.locator("tbody tr").nth(0).locator("input[type=checkbox]").check(); await page.waitForTimeout(250);
  await zeile(0).click(); await page.locator("[data-dk=runter]").click(); await page.waitForTimeout(400);
  ok((await dv()).files[1].checked === true && (await dv()).files[0].checked === false, "Die Vormerkung bleibt bei der Datei");
  await page.keyboard.press(`${MOD}+z`); await page.waitForTimeout(300); await page.locator("tbody tr").nth(0).locator("input[type=checkbox]").uncheck(); await page.waitForTimeout(250);
  v = await dv(); ok(!v.dirty && !v.canRedo === false && (await namen()) === o0, "Danach ist die Diskette wieder im Ausgangszustand (dirty " + v.dirty + ", " + await namen() + ")");
  // Speichern nach dem Ziehen: Cmd+S, Befehl „speichern“ und Knopf schreiben dieselbe Datei (Reihenfolge auf der Platte prüfen)
  { const datei = join(proj, "test.d64"); const vorher = readFileSync(datei);
    const ziehen = async () => { const q0 = await zeile(0).locator("td.nm").boundingBox(), q2 = await zeile(2).locator("td.nm").boundingBox(); await page.mouse.move(q0.x + 20, q0.y + q0.height / 2); await page.mouse.down(); await page.mouse.move(q2.x + 20, q2.y + q2.height / 2, { steps: 8 }); await page.mouse.up(); await page.waitForTimeout(500); };
    await ziehen(); ok((await dv()).dirty, "Nach dem Ziehen: ungespeichert");
    await page.keyboard.press(`${MOD}+s`); await page.waitForTimeout(700);
    const nach = readFileSync(datei); ok(!(await dv()).dirty && !nach.equals(vorher), "Cmd+S direkt nach dem Ziehen schreibt die Diskette (Datei geändert, Hinweis „ungespeichert“ weg)");
    await page.keyboard.press(`${MOD}+z`); await page.waitForTimeout(400); ok((await dv()).dirty, "Rückgängig nach dem Speichern macht die Diskette wieder „geändert“");
    await page.evaluate(() => window.__vc.runCommand("speichern")); await page.waitForTimeout(600); ok(!(await dv()).dirty && readFileSync(datei).equals(vorher), "Der Befehl „Speichern“ (Menü) schreibt ebenso, die Datei ist wieder im Ausgangszustand");
    await ziehen(); await page.getByRole("button", { name: "Speichern", exact: true }).click(); await page.waitForTimeout(600); ok(!(await dv()).dirty && !readFileSync(datei).equals(vorher), "Der Knopf „Speichern“ schreibt ebenso");
    await page.keyboard.press(`${MOD}+z`); await page.waitForTimeout(300); await page.keyboard.press(`${MOD}+s`); await page.waitForTimeout(600); ok(readFileSync(datei).equals(vorher) && !(await dv()).dirty, "Zurückgenommen und gespeichert: Datei wieder wie zu Anfang");
  }
  await snap("disk-1b-ordnen");
}

// 2) Datei wählen: Kette, Block n von m
await page.locator("tbody tr").nth(1).click(); await page.waitForTimeout(300);
v = await dv();
ok(v.selFile === 1 && v.chain.length >= 2 && v.chainPos?.[0] === 1, "Datei wählen: Kette von " + v.chain.length + " Sektoren, Inspektor am ersten Sektor");
ok(/Block 1 von \d+/.test(await page.locator(".col.r").innerText()), "Inspektor nennt „Block 1 von n“");
// 3) Folgen und Zurück
const vorFolgen = v.cur.join("/");
await page.getByRole("button", { name: /^Folgen ▶/ }).click(); await page.waitForTimeout(250);
v = await dv(); ok(v.cur.join("/") !== vorFolgen && v.canBack, "Folgen springt zum Folgesektor: " + vorFolgen + " → " + v.cur.join("/"));
await page.getByRole("button", { name: "Zurück" }).click(); await page.waitForTimeout(250);
v = await dv(); ok(v.cur.join("/") === vorFolgen, "Zurück kehrt zurück");
// Kopf des Sektor-Inspektors bricht nie um: Titel und beide Knöpfe in einer Zeile, Knöpfe bleiben beim Weiterspringen an ihrer Stelle
const kopf = () => page.evaluate(() => { const h = document.querySelector(".col.r .hd"), t = h.querySelector("h3").getBoundingClientRect(), b = [...h.querySelectorAll("button")].map((x) => x.getBoundingClientRect()); return { h: h.getBoundingClientRect().height, dy: Math.max(Math.abs(t.top + t.height / 2 - b[0].top - b[0].height / 2), Math.abs(b[1].top - b[0].top)), zx: b[0].left, fx: b[1].left, fw: b[1].width }; });
await page.setViewportSize({ width: 1000, height: 700 }); await page.waitForTimeout(300);
const k1 = await kopf(); await page.getByRole("button", { name: /^Folgen ▶/ }).click(); await page.waitForTimeout(250); const k2 = await kopf();
ok(k1.h < 60 && k2.h < 60 && k1.dy < 8 && k2.dy < 8, "Sektor-Kopf bleibt bei schmalem Fenster einzeilig (" + Math.round(k1.h) + " / " + Math.round(k2.h) + " px hoch)");
ok(Math.abs(k1.fx - k2.fx) < 2 && Math.abs(k1.fw - k2.fw) < 2, "Die Knöpfe behalten ihre Stelle und Breite beim Weiterspringen (" + Math.round(k1.fx) + " → " + Math.round(k2.fx) + ")");
await page.getByRole("button", { name: "Zurück" }).click(); await page.waitForTimeout(250); await page.setViewportSize({ width: 1280, height: 800 });
// 4) Klick in die Belegungskarte
const box = await page.locator(".map canvas").boundingBox();
await page.mouse.click(box.x + 3, box.y + 3); await page.waitForTimeout(300);
v = await dv(); ok(v.cur.join("/") === "1/0", "Klick auf das erste Kästchen wählt Spur 1, Sektor 0: " + v.cur.join("/"));
// 5) Pfeiltasten und Ansage
await page.locator(".map .wrap").focus(); await page.keyboard.press("ArrowRight"); await page.keyboard.press("ArrowDown"); await page.waitForTimeout(400);
v = await dv(); ok(v.cur.join("/") === "2/1", "Pfeiltasten bewegen den Sektor: " + v.cur.join("/"));
ok(/Spur 2, Sektor 1/.test(await page.locator(".map .sr").innerText()), "Die Auswahl wird angesagt: " + (await page.locator(".map .sr").innerText()));
// 6) Kette ausblenden
await page.getByLabel("Kette zeigen").uncheck(); await page.waitForTimeout(250);
ok(!(await dv()).chainOn, "Kette zeigen lässt sich ausschalten");
await page.getByLabel("Kette zeigen").check();
// 7) Zeilen im Verzeichnis per Tastatur
await page.locator("tbody tr").first().focus(); await page.keyboard.press("ArrowDown"); await page.waitForTimeout(300);
ok((await dv()).selFile === 1, "Pfeil nach unten im Verzeichnis wählt die nächste Datei");
await snap("disk-2-gewaehlt");

// 8) D81: zwei Blöcke, 80 Spuren
await page.evaluate(() => window.__vc.openFile("test.d81")); await page.waitForTimeout(700);
v = await dv(); ok(v.format === "D81" && v.cells.length === 80, "test.d81: 80 Spuren");
const b2 = await page.locator(".map canvas").boundingBox(); ok(b2.height < 40 * 15 + 10, "Große Diskette steht in zwei Spurblöcken (Höhe " + Math.round(b2.height) + " px)");
await snap("disk-3-d81");
// 8b) Atari MyDOS (256-Byte-Sektoren, echtes Abbild, wenn VC_ATR_ECHT gesetzt war): Erkennung, Anzeige, Dateien, Prüfung
if (existsSync(join(proj, "mydos.atr"))) {
  await page.evaluate(() => window.__vc.openFile("mydos.atr")); await page.waitForTimeout(800); v = await dv();
  ok(v.format === "ATR" && /MyDOS/.test(v.geometry) && /720/.test(v.geometry) && /256/.test(v.geometry), "mydos.atr: Diskettenspalte nennt „" + v.geometry + "“");
  ok(v.files.length >= 2 && v.files.some((f) => /DOS\.SYS/.test(f.name)) && v.problems.length === 0, "MyDOS: Dateien DOS.SYS und DUP.SYS stehen im Verzeichnis, keine Prüfmeldungen");
  ok(v.hex && v.hex.length === 32, "MyDOS: der Sektor zeigt 256 Byte (" + (v.hex ? v.hex.length : 0) + " Zeilen zu 8)");
  await page.evaluate(() => window.__vc.openFile("test.d81")); await page.waitForTimeout(700);
}
// 9) Kaputtes Abbild: Meldung, die vorige Diskette bleibt
await page.evaluate(() => window.__vc.openFile("kaputt.d64")); await page.waitForTimeout(600);
ok(/lässt sich nicht lesen/.test(await page.locator(".toasts").innerText()) && (await dv()).format === "D81", "Unlesbares Abbild: Meldung, die geöffnete Diskette bleibt");
// E2.2) Prüfen, Vormerken, Übernehmen
await page.evaluate(() => window.__vc.openFile("test.d64")); await page.waitForTimeout(700);
await page.getByRole("button", { name: "Prüfen" }).click(); await page.waitForTimeout(300);
ok(/Keine Probleme gefunden/.test(await page.locator(".col.l").innerText()), "Prüfen: Bericht „Keine Probleme gefunden“");
await page.getByRole("button", { name: "Alle", exact: true }).click(); await page.waitForTimeout(250);
ok((await dv()).checkedCount === 3 && (await page.locator("tbody input[type=checkbox]:checked").count()) === 3, "Vormerken „Alle“: drei Dateien");
await page.getByRole("button", { name: "Keine", exact: true }).click(); await page.waitForTimeout(250);
await page.getByLabel("Vormerken: ZWEI").check(); await page.waitForTimeout(250);
ok((await dv()).checkedCount === 1, "Einzelne Datei vormerken (Schalter in der Zeile, wählt die Datei nicht)");
await page.getByRole("button", { name: "Programme", exact: true }).click(); await page.waitForTimeout(250);
ok((await dv()).checkedCount === 3, "Vormerken „Programme“: alle drei sind PRG");
await page.getByLabel("Zielordner im Projekt").fill("../raus"); await page.getByLabel("Zielordner im Projekt").press("Tab"); await page.waitForTimeout(300);
ok(/innerhalb des Projekts|im Projekt liegen/.test(await page.locator(".col.l").innerText()) && await page.getByRole("button", { name: /Ins Projekt übernehmen/ }).isDisabled(), "Zielordner außerhalb des Projekts: Hinweis, Übernehmen gesperrt");
await page.getByLabel("Zielordner im Projekt").fill("aus_disk"); await page.getByLabel("Zielordner im Projekt").press("Tab"); await page.waitForTimeout(300);
await page.getByRole("button", { name: /Ins Projekt übernehmen/ }).click(); await page.waitForTimeout(1200);
ok(((await dv()).importResult ?? []).filter((x) => x.ok).length >= 3 && existsSync(join(proj, "aus_disk")) && readdirSync(join(proj, "aus_disk")).length >= 3, "Übernehmen legt die Dateien im Zielordner an: " + (existsSync(join(proj, "aus_disk")) ? readdirSync(join(proj, "aus_disk")).join(", ") : "–"));
ok(await page.evaluate(() => JSON.stringify(window.__vc.ui.project.tree).includes("aus_disk")), "Der Dateibaum kennt den neuen Ordner");
await snap("disk-5-uebernommen");
// E2.3) Ablegen, Speichern unter, Speichern
ok(!/Image ungespeichert/.test(await page.locator(".col.l").innerText()), "Frisch geöffnet: nicht „ungespeichert“");
await page.getByRole("button", { name: "Dateien ablegen …" }).click(); await page.waitForTimeout(600);
v = await dv(); ok(v.files.length === 4 && v.dirty, "Dateien ablegen: vierte Datei, Image ungespeichert");
// Handtest: Datei aus dem Finder auf die offene Diskette ziehen fügt einen Eintrag hinzu
{ const dropme = join(proj, "DROPME.PRG"); writeFileSync(dropme, Buffer.from([0x01, 0x08, 0x00, 0x00, 0x60])); const vor = (await dv()).files.length;
  await page.evaluate((pp) => window.__vc.handleDrop([pp]), dropme); await page.waitForTimeout(700);
  ok((await dv()).files.length === vor + 1 && !/nicht öffnet/.test(await page.locator(".toasts").innerText().catch(() => "")), "Ziehen einer Datei auf die offene Diskette fügt einen Eintrag hinzu"); }
ok(/Image ungespeichert/.test(await page.locator(".col.l").innerText()) && /● test\.d64/.test(await page.locator(".img").innerText()), "Der Zustand ist sichtbar (Hinweis und Punkt vor dem Namen)");
await page.getByRole("button", { name: "Speichern", exact: true }).click(); await page.waitForTimeout(500);
ok(!(await dv()).dirty, "Speichern: nicht mehr ungespeichert");
await page.getByRole("button", { name: "Dateien ablegen …" }).click(); await page.waitForTimeout(500);
await page.getByRole("button", { name: "Speichern unter …" }).click(); await page.waitForTimeout(600);
v = await dv(); ok(!v.dirty && /saved\.d64$/.test(v.path), "Speichern unter: Kopie geschrieben, Pfad übernommen: " + v.path);
ok(existsSync(join(proj, "saved.d64")), "saved.d64 liegt im Projekt");
// Neue Diskette; Rückfrage bei ungespeicherter Diskette
await page.getByRole("button", { name: "Neue Diskette …" }).click(); await page.waitForSelector("dialog[open]");
await page.getByLabel("Name der Diskette").fill("meine"); await page.getByRole("button", { name: "Anlegen" }).click(); await page.waitForTimeout(600);
v = await dv(); ok(v.format === "D64" && v.files.length === 0 && v.dirty && v.path === null, "Neue Diskette: leer, ungespeichert, ohne Pfad");
await page.evaluate(() => { void window.__vc.runCommand("projekt_schliessen"); }); await page.waitForTimeout(400);
ok(/Diskette|D64/.test(await page.getByRole("dialog").first().innerText()) && /MEINE|meine|D64/.test(await page.getByRole("dialog").first().innerText()), "Beenden-Liste nennt die ungespeicherte Diskette: " + (await page.getByRole("dialog").first().locator("ul.leave li").allInnerTexts()).join(" | "));
await page.getByRole("dialog").first().getByRole("button", { name: "Abbrechen" }).click(); await page.waitForTimeout(300);
await page.getByRole("button", { name: "Diskette schließen" }).click(); await page.waitForTimeout(400);
ok(/Ungespeicherte Diskette/.test(await page.getByRole("dialog").first().innerText()), "Diskette schließen mit Änderungen fragt nach");
await page.getByRole("dialog").first().getByRole("button", { name: "Abbrechen" }).click(); await page.waitForTimeout(300);
ok(!!(await dv()), "Abbrechen: die Diskette bleibt");
await page.getByRole("button", { name: "Speichern unter …" }).click(); await page.waitForTimeout(500);
ok(!(await dv()).dirty, "Neue Diskette lässt sich speichern (Dialog in der Prüfbrücke liefert den Pfad)");
// E2.4) Einlegen im Emulator (in der Prüfbrücke wird nichts gestartet)
ok(!(await page.getByRole("button", { name: "Im Emulator einlegen" }).isDisabled()), "Gespeicherte Diskette lässt sich einlegen");
await page.getByRole("button", { name: "Im Emulator einlegen" }).click(); await page.waitForTimeout(400);
ok(/Eingelegt/.test(await page.locator(".toasts").innerText()), "Einlegen meldet „Eingelegt …“");
await page.getByRole("button", { name: "Neue Diskette …" }).click(); await page.waitForSelector("dialog[open]"); await page.getByRole("button", { name: "Anlegen" }).click(); await page.waitForTimeout(500);
ok(await page.getByRole("button", { name: "Im Emulator einlegen" }).isDisabled(), "Ungespeicherte Diskette: Einlegen gesperrt („Erst speichern“)");
await page.getByRole("button", { name: "Speichern unter …" }).click(); await page.waitForTimeout(500);
// 10) Schließen, leerer Zustand, Öffnen über die Befehlspalette (Prüfbrücke liefert VC_TEST_DISK)
await page.getByRole("button", { name: "Diskette schließen" }).click(); await page.waitForTimeout(300);
ok(/Keine Diskette geöffnet/.test(await page.locator(".blank").innerText()), "Ohne Diskette erklärt sich die Ansicht");
await page.evaluate(() => { void window.__vc.runCommand("disk_oeffnen"); }); await page.waitForTimeout(800);
ok((await dv())?.format === "D64", "Befehl „Diskette öffnen …“ öffnet die gewählte Datei");
await page.evaluate(() => { window.__vc.ui.palette = true; }); await page.waitForTimeout(400);
await page.getByRole("combobox").fill("Diskette öffnen"); await page.waitForTimeout(300);
ok((await page.getByRole("option").allInnerTexts()).some((x) => /Diskette öffnen/.test(x)), "Die Befehlspalette bietet „Diskette öffnen …“ an");
await page.keyboard.press("Escape");
// 11) Dunkel und hell
for (const th of ["light", "dark"]) { await page.evaluate(async (th) => { await window.__vc.setTheme(th); }, th); await page.waitForTimeout(300); await snap("disk-4-" + th); }
// 12) Englisch
await page.evaluate(async () => { await window.__vc.setLanguage("en"); }); await page.waitForTimeout(400);
await page.setViewportSize({ width: 1000, height: 700 }); await page.waitForTimeout(300); { const k = await kopf(); ok(k.h < 60 && k.dy < 8, "Englisch, schmales Fenster: Sektor-Kopf einzeilig (" + Math.round(k.h) + " px)"); } await page.setViewportSize({ width: 1280, height: 800 });
ok(/directory/i.test(await page.locator(".col.m").innerText()) && /allocation map/i.test(await page.locator(".col.m").innerText()), "Englisch: Verzeichnis und Belegungskarte übersetzt");
ok(csp.length === 0, "Keine Meldungen der Inhaltsrichtlinie: " + csp.slice(0, 2).join(" | "));
console.log(`\n${n} Prüfungen, ${fails.length} Fehler`);
await browser.close();
process.exit(fails.length ? 1 : 0);
