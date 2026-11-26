// SPDX-License-Identifier: GPL-3.0-or-later
// Ablauftest Startseite (E1.4): Zuletzt-Liste (anheften, entfernen, fehlender Ordner), Beispiele, Projekt kopieren, Projektdatei wählen, erkannte Emulatoren.
// Aufruf: node tools/e2e-start.mjs <Projektkopie A> <Projektkopie B> <Zielordner für Kopien (leer)> [Bilder]
//   Kern mit HOME=<leer> VC_TEST_DIR=<Zielordner> VC_TEST_PICK_FILE=<.vcprj von Projektkopie B> VC_TESTBRIDGE=17420 VC_TEST_NO_SPAWN=1 VC_NO_SAVE=1
import { webkit } from "playwright";
import { existsSync, mkdirSync, readFileSync, renameSync } from "node:fs";
import { join } from "node:path";

const [projA, projB, ziel, shots] = process.argv.slice(2);
if (!projA || !projB || !ziel) { console.error("Aufruf: node tools/e2e-start.mjs <A> <B> <Zielordner> [Bilder]"); process.exit(2); }
if (shots) mkdirSync(shots, { recursive: true });
import { writeFileSync as schreibe } from "node:fs";
schreibe(process.argv[2] + "/projektbild.png", Buffer.from("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==", "base64"));   // Projektbild für die Karte
const browser = await webkit.launch();
const page = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
const fails = []; let n = 0; const csp = [];
const ok = (c, m) => { n++; if (!c) { fails.push(m); console.log("FEHLER:", m); } else console.log("ok:", m); };
page.on("pageerror", (e) => { fails.push("Seitenfehler " + e.message); console.log("Seitenfehler", e.message); });
page.on("console", (m) => { if (/Content Security Policy|Refused to/i.test(m.text())) csp.push(m.text()); });
const snap = async (name) => { if (shots) await page.screenshot({ path: join(shots, name + ".png") }); };
const start = async () => { await page.goto("http://127.0.0.1:1420/?bridge=17420"); await page.waitForFunction(() => window.__vc && window.__vc.ui.ready); await page.evaluate(async () => { await window.__vc.setLanguage("de"); }); await page.waitForTimeout(500); };
await start();
const dlg = page.getByRole("dialog");
const nameA = projA.split("/").pop();
// Willkommenskarte (erste Einrichtung fehlt): statt der Werkzeugliste; „Später“ blendet sie dauerhaft aus
ok(await page.locator("[data-start=welcome-card]").count() === 1, "Willkommenskarte erscheint ohne Einstellungen");
ok(await page.getByRole("button", { name: "Einrichtung starten" }).count() === 1 && await page.getByRole("button", { name: "Später" }).count() === 1, "… mit „Einrichtung starten“ und „Später“");
await page.locator("[data-start=welcome-later]").click(); await page.waitForTimeout(200);
ok(await page.locator("[data-start=welcome-card]").count() === 0, "„Später“ blendet die Karte aus");
await start();
ok(await page.locator("[data-start=welcome-card]").count() === 0, "… auch nach Neuladen");
// Hauptnavigation: „Startseite“ steht ganz links und ist auf der Startseite hervorgehoben; schmal nur Symbol für die anderen
ok(await page.locator("nav .seg button[aria-current=page]").innerText().then((x) => /Startseite/.test(x)) && /Startseite/.test(await page.locator("nav .seg button").first().innerText()), "Startseite: „Startseite“ ist ganz links und in der Hauptnavigation hervorgehoben");
await page.setViewportSize({ width: 800, height: 700 }); await page.waitForTimeout(200);
const sichtbar = await page.locator("nav .seg button:not([aria-current=page]) .nl").first().evaluate((el) => el.getBoundingClientRect().width);
ok(sichtbar <= 2, "Schmal (800 px): nicht aktive Ziele zeigen nur das Symbol (Breite " + sichtbar + ")");
ok(await page.locator("body").evaluate((b) => b.scrollWidth <= window.innerWidth + 1), "Schmal: kein waagerechtes Scrollen");
await page.setViewportSize({ width: 1280, height: 800 });
// Startbild: nur mit ?splash=1 in der Prüfbrücke; steht etwa drei Sekunden, schließt von selbst; Klick oder Taste beendet es früher
await page.goto("http://127.0.0.1:1420/?bridge=17420&splash=1"); await page.waitForTimeout(300);
ok(await page.locator("[data-splash]").count() === 1, "Startbild erscheint");
await page.waitForTimeout(1800);
ok(await page.locator("[data-splash]").count() === 1, "… und steht nach 2 Sekunden noch (Dauer etwa drei Sekunden)");
await page.waitForTimeout(1500);
ok(await page.locator("[data-splash]").count() === 0, "Startbild schließt nach etwa drei Sekunden von selbst");
await page.goto("http://127.0.0.1:1420/?bridge=17420&splash=1"); await page.waitForTimeout(400);
await page.mouse.click(300, 300); await page.waitForTimeout(300);
ok(await page.locator("[data-splash]").count() === 0, "Klick beendet das Startbild früher");
await page.goto("http://127.0.0.1:1420/?bridge=17420&splash=1"); await page.waitForTimeout(400);
await page.keyboard.press("a"); await page.waitForTimeout(300);
ok(await page.locator("[data-splash]").count() === 0, "Eine beliebige Taste beendet das Startbild früher");
await start();
ok(await page.getByRole("button", { name: "Neues Projekt" }).count() === 1 && await page.getByRole("button", { name: "Projekt öffnen" }).count() === 1, "Startseite: Neues Projekt und Projekt öffnen");
ok(await page.getByRole("button", { name: /Projektdatei/ }).count() === 1, "… und Projektdatei wählen");
const tools = await page.getByRole("region", { name: "Toolchain und Hardware" }).locator("li").allInnerTexts();
ok(tools.length >= 3 && tools.every((x) => /bereit|nicht gefunden/.test(x)), "Erkannte Emulatoren mit Zustand: " + tools.map((x) => x.replace(/\s+/g, " ")).slice(0, 4).join(" | "));
const keys = await page.getByRole("region", { name: "Schnellreferenz" }).locator("li").allInnerTexts();
ok(keys.length >= 5, "Schnellreferenz zeigt Tastenkürzel (" + keys.length + ")");
ok(await page.getByText("Noch kein Projekt geöffnet").count() === 1, "Leere Zuletzt-Liste erklärt sich");
await snap("start-1-leer");

// Zuletzt-Liste füllen: zwei Projekte öffnen und schließen
for (const p of [projA, projB]) { await page.evaluate(async (p) => { await window.__vc.openProject(p); }, p); await page.waitForTimeout(400); await page.evaluate(() => { void window.__vc.runCommand("projekt_schliessen"); }); await page.waitForTimeout(500); }
await start();
const liste = () => page.getByRole("region", { name: "Zuletzt geöffnete Projekte" }).locator("li.proj");
ok(await liste().count() === 2, "Zuletzt geöffnet: beide Projekte (" + await liste().count() + ")");
ok(/jumprun/.test(await liste().first().innerText()) && /C64/i.test(await liste().first().innerText()), "Eintrag zeigt Name und Zielsystem: " + (await liste().first().innerText()).replace(/\s+/g, " ").slice(0, 80));
// Anheften: der ältere Eintrag rückt nach vorn
const zweiter = await liste().nth(1).locator(".name").innerText();
await liste().nth(1).getByRole("button", { name: "Projekt anheften" }).click(); await page.waitForTimeout(400);
ok(await liste().first().locator(".name").innerText() === zweiter && await liste().first().getByRole("button", { name: "Anheften lösen" }).getAttribute("aria-pressed") === "true", "Anheften: der Eintrag steht vorn und ist als angeheftet gekennzeichnet");
await start(); ok(await liste().first().locator(".name").innerText() === zweiter, "… und das bleibt nach einem Neustart so");
const hz = await liste().first().evaluate((e) => e.getBoundingClientRect().height);
ok((await page.locator("li.proj img.pic").count()) >= 1, "Die Karte zeigt das Projektbild (projektbild.png), sonst eine Ersatzkachel: " + (await page.locator("li.proj .pic.ph").count()) + " Ersatzkacheln");
ok(hz <= 300 && hz >= 120, "Projekt als Karte mit Bild oder Ersatzkachel: " + Math.round(hz) + " px");
ok(/angeheftet/i.test(await page.getByRole("region", { name: "Zuletzt geöffnete Projekte" }).innerText()), "Angeheftete haben eine eigene Überschrift");
const pf = await liste().first().locator(".path").innerText();
ok(pf.length <= 60 && !/\n/.test(pf), "Pfad steht in einer Zeile (in der Mitte gekürzt, wenn lang): " + pf);
await liste().first().locator(".main").focus(); await page.keyboard.press("ArrowDown");
ok(await page.evaluate(() => document.activeElement?.classList.contains("main") && document.activeElement !== document.querySelector(".proj .main")), "Pfeil nach unten springt zum nächsten Eintrag");
await snap("start-2-liste");
// Fehlender Ordner
renameSync(projB, projB + "_weg"); await start();
ok((await page.getByText("Ordner fehlt").count()) === 1 && await page.locator(".proj.missing .main").isDisabled(), "Fehlender Ordner: Hinweis und Eintrag gesperrt");
renameSync(projB + "_weg", projB);
// Entfernen
const vor = await liste().count(); await liste().first().getByRole("button", { name: /Weitere Aktionen zu/ }).click(); await page.waitForTimeout(200);
const mi = await page.getByRole("menuitem").allInnerTexts();
ok(mi.join("|") === "Öffnen|Anheften lösen|Kopieren …|Im Ordner zeigen|Aus der Liste entfernen", "Menü „…“ je Eintrag: " + mi.join(" | "));
await page.keyboard.press("Escape"); await page.waitForTimeout(200);
// Rechtsklick auf den Eintrag öffnet dasselbe Menü (Handtest-Befund)
await liste().first().locator(".name").click({ button: "right" }); await page.waitForTimeout(250);
ok((await page.getByRole("menuitem").allInnerTexts()).join("|") === mi.join("|"), "Rechtsklick auf einen Eintrag zeigt das Menü mit Anheften, Im Ordner zeigen, Entfernen");
const ganz = () => page.evaluate(() => [...document.querySelectorAll("[role=menuitem]")].every((el) => { const r = el.getBoundingClientRect(); const e = document.elementFromPoint(r.x + r.width / 2, r.y + r.height / 2); return r.width > 0 && r.left >= 0 && r.top >= 0 && r.right <= innerWidth && r.bottom <= innerHeight && (e === el || el.contains(e)); }));
ok(await ganz(), "Das Rechtsklick-Menü liegt ganz im Fenster, alle Einträge sind erreichbar (nicht von der Karte abgeschnitten)");
await page.keyboard.press("Escape"); await page.waitForTimeout(200);
await liste().last().getByRole("button", { name: /Weitere Aktionen zu/ }).click(); await page.waitForTimeout(300);
ok((await page.getByRole("menuitem").count()) >= 4 && await ganz(), "Das „…“-Menü des letzten Eintrags liegt ebenfalls ganz sichtbar über der Karte");
await page.keyboard.press("Escape"); await page.waitForTimeout(200);
await liste().first().locator(".name").click({ button: "right" }); await page.waitForTimeout(250);
await page.getByRole("menuitem", { name: "Aus der Liste entfernen" }).click(); await page.waitForTimeout(400);
ok(await liste().count() === vor - 1, "Entfernen nimmt den Eintrag aus der Liste");
await start(); ok(await liste().count() === vor - 1, "… auch nach einem Neustart");
// Eintrag öffnet das Projekt
await liste().first().locator(".name").click(); await page.waitForTimeout(600);
ok(await page.evaluate(() => !!window.__vc.ui.project), "Klick auf den Namen öffnet das Projekt");
// „Startseite“ ganz links in der Hauptnavigation: mit offenem Projekt erst Rückfrage
await page.locator("nav .seg button", { hasText: "Startseite" }).click(); await page.waitForSelector("dialog[open]");
await page.locator("dialog[open]").getByRole("button", { name: "Abbrechen" }).first().click(); await page.waitForTimeout(300);
ok(await page.evaluate(() => !!window.__vc.ui.project), "Navigation „Startseite“ mit Projekt: „Abbrechen“ lässt das Projekt offen");
await page.locator("nav .seg button", { hasText: "Startseite" }).click(); await page.waitForSelector("dialog[open]");
await page.locator("dialog[open]").getByRole("button", { name: "Projekt schließen" }).click(); await page.waitForTimeout(600);
ok(await page.evaluate(() => !window.__vc.ui.project && window.__vc.ui.mode === "code") && await page.locator("[data-start=welcome-card], [data-start=willkommen], .abschnitt").first().count() > 0, "… „Projekt schließen“ zeigt die Startseite");
await page.evaluate(() => { void window.__vc.runCommand("projekt_schliessen"); }); await page.waitForTimeout(500);
// Zuletzt geöffnete Projekte als Treffer in der Befehlspalette
await page.evaluate(() => { window.__vc.ui.palette = true; }); await page.waitForTimeout(400);
await page.getByRole("combobox").fill("Projekt öffnen: C64"); await page.waitForTimeout(300);
const pt = await page.getByRole("option").allInnerTexts();
ok(pt.some((x) => /Projekt öffnen: C64: jumprun \(/.test(x)), "Die Palette bietet zuletzt geöffnete Projekte an: " + pt.slice(0, 2).join(" | ").replace(/\s+/g, " ").slice(0, 100));
await page.keyboard.press("Enter"); await page.waitForTimeout(700);
ok(await page.evaluate(() => !!window.__vc.ui.project), "Enter in der Palette öffnet das Projekt");
await page.evaluate(() => { void window.__vc.runCommand("projekt_schliessen"); }); await page.waitForTimeout(500);
// Projektdatei wählen (Prüfbrücke liefert VC_TEST_PICK_FILE)
await page.getByRole("button", { name: /Projektdatei/ }).click(); await page.waitForTimeout(700);
ok(await page.evaluate(() => !!window.__vc.ui.project), "Projektdatei wählen öffnet das Projekt");
await page.evaluate(() => { void window.__vc.runCommand("projekt_schliessen"); }); await page.waitForTimeout(500);
// Beispiele: Startseite zeigt nur wenige empfohlene (Zuletzt-Liste steht darüber), „Alle Beispiele“ ist ein Dialog mit Suche und Gruppen
const bsp = page.getByRole("region", { name: "Beispielprojekte" });
const nBsp = await bsp.locator("li").count();
ok(nBsp >= 1 && nBsp <= 8 && /jumprun/.test(await bsp.innerText()) && /Hugo/.test(await bsp.innerText()), "Startseite zeigt höchstens acht Beispielkarten mit Beschreibung: " + nBsp);
const yZ = (await page.getByRole("region", { name: "Zuletzt geöffnete Projekte" }).boundingBox())?.y ?? 9e9, yB = (await bsp.boundingBox())?.y ?? 0;
ok(yZ < yB, "Die Zuletzt-Liste steht über den Beispielen");
await page.getByRole("button", { name: "Alle Beispiele …" }).click(); await page.waitForSelector("dialog[open]");
const ad = page.locator("dialog[open]");
const nGr = await ad.locator("details").count();
ok(nGr >= 5 && (await ad.locator("details[open]").count()) === 0, "„Alle Beispiele“: nach Zielsystem gruppiert, Gruppen zugeklappt: " + nGr + " Gruppen");
await ad.getByRole("searchbox").fill("mega65"); await page.waitForTimeout(200);
ok((await ad.locator("details").count()) === 1 && (await ad.locator("details[open]").count()) === 1 && /MEGA65/.test(await ad.locator("summary").first().innerText()), "Suche „mega65“ zeigt nur diese Gruppe, aufgeklappt");
await ad.getByRole("searchbox").fill("zzzz-nichts"); await page.waitForTimeout(200);
ok(/Keine Beispiele gefunden/.test(await ad.innerText()), "Ohne Treffer steht ein Hinweis");
await ad.getByRole("searchbox").fill("jumprun"); await page.waitForTimeout(200);
await snap("start-3-beispiele");
await page.keyboard.press("Escape"); await page.waitForTimeout(300);

await bsp.locator("li", { hasText: "jumprun" }).first().getByRole("button", { name: "Kopieren …" }).click(); await page.waitForSelector("dialog[open]");
ok(/Projekt kopieren/.test(await dlg.innerText()) && (await dlg.getByLabel("Name der Kopie").inputValue()) === "jumprun Kopie", "Kopierdialog schlägt „jumprun Kopie“ vor (ist: „" + (await dlg.getByLabel("Name der Kopie").inputValue()) + "“)");
await dlg.getByRole("button", { name: "Wählen …" }).click(); await page.waitForTimeout(400);
ok(/\d+ Dateien/.test(await dlg.innerText()), "Der Dialog nennt Anzahl und Größe: " + (await dlg.innerText()).match(/\d+ Dateien[^\n]*/)?.[0]);
await snap("start-4-kopieren");
await dlg.getByRole("button", { name: "Kopieren" }).click(); await page.waitForFunction(() => window.__vc.ui.project, null, { timeout: 20000 }).catch(() => {}); await page.waitForTimeout(500);
ok(existsSync(join(ziel, "jumprun Kopie")) && existsSync(join(ziel, "jumprun Kopie", "jumprun.asm")), "Die Kopie liegt im Zielordner");
ok(await page.evaluate(() => window.__vc.ui.project?.root.endsWith("jumprun Kopie")), "Danach ist die Kopie geöffnet");
// Kopie für ein anderes System (Entwurf): Normalziel wird das gewählte System, Quellen bleiben unverändert, Hinweis sichtbar
await page.evaluate(() => { void window.__vc.runCommand("projekt_kopieren"); }); await page.waitForSelector("dialog[open]");
await dlg.getByRole("button", { name: "Wählen …" }).click(); await page.waitForTimeout(300);
await dlg.getByLabel("Name der Kopie").fill("jumprun Atari"); await page.waitForTimeout(300);
await dlg.locator("[data-kop=system]").selectOption("c64"); await page.waitForTimeout(200);
ok(/Code wird nicht umgesetzt/.test(await dlg.locator("[data-kop=system-hinweis]").innerText()), "Anderes System: Hinweis, dass der Code nicht umgesetzt wird");
await dlg.getByRole("button", { name: "Kopieren" }).click(); await page.waitForFunction(() => window.__vc.ui.project?.root.endsWith("jumprun Atari"), null, { timeout: 20000 }).catch(() => {}); await page.waitForTimeout(500);
const prj = readFileSync(join(ziel, "jumprun Atari", "jumprun Atari.vcprj"), "utf8");
ok(JSON.parse(prj).targets[0].id === "c64" && JSON.parse(prj).targets.length === 2, "Normalziel der Kopie ist c64 (erster Eintrag, altes Ziel bleibt)");
ok(existsSync(join(ziel, "jumprun Atari", "jumprun.asm")) && readFileSync(join(ziel, "jumprun Atari", "jumprun.asm"), "utf8") === readFileSync(join(ziel, "jumprun Kopie", "jumprun.asm"), "utf8"), "Quellen der System-Kopie sind unverändert");
// Zweite Kopie mit gleichem Namen: Problem sichtbar, Kopieren gesperrt
await page.evaluate(() => { void window.__vc.runCommand("projekt_kopieren"); }); await page.waitForSelector("dialog[open]");
await dlg.getByRole("button", { name: "Wählen …" }).click(); await page.waitForTimeout(300);
await dlg.getByLabel("Name der Kopie").fill("jumprun Kopie"); await page.waitForTimeout(400);
ok(await dlg.getByRole("alert").count() >= 1 && await dlg.getByRole("button", { name: "Kopieren" }).isDisabled(), "Vorhandenes Ziel: Hinweis „gibt es schon“, Kopieren gesperrt: " + (await dlg.getByRole("alert").first().innerText()));
await dlg.getByRole("button", { name: "Abbrechen" }).click();
// Karten heben sich von der Seite ab (hell und dunkel), Text bleibt AA, Eintrag wird beim Überfahren heller
await start();
for (const th of ["light", "dark"]) {
  await page.evaluate(async (th) => { await window.__vc.setTheme(th); }, th); await page.waitForTimeout(300);
  const m = await page.evaluate(() => {
    const rgb = (c) => c.match(/[\d.]+/g).slice(0, 3).map(Number);
    const lum = (c) => { const [r, g, b] = rgb(c).map((v) => { v /= 255; return v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4; }); return 0.2126 * r + 0.7152 * g + 0.0722 * b; };
    const cr = (a, b) => { const x = lum(a), y = lum(b); return (Math.max(x, y) + 0.05) / (Math.min(x, y) + 0.05); };
    const page = getComputedStyle(document.querySelector(".start")).backgroundColor, card = getComputedStyle(document.querySelector(".tafel")).backgroundColor;
    const row = document.querySelector(".proj"); const rb = getComputedStyle(row).backgroundColor, nm = getComputedStyle(row.querySelector(".name")).color, pa = getComputedStyle(row.querySelector(".path")).color;
    return { karte: cr(page, card), name: cr(nm, rb), pfad: cr(pa, rb), page, card, rb };
  });
  ok(m.page !== m.card, th + ": Karte hebt sich farblich von der Seite ab (" + m.page + " / " + m.card + ", Kontrast " + m.karte.toFixed(2) + ")");
  ok(m.name >= 4.5 && m.pfad >= 4.5, th + ": Text auf der Zeile AA (Name " + m.name.toFixed(1) + ", Pfad " + m.pfad.toFixed(1) + ")");
  await page.mouse.move(2, 2); await page.waitForTimeout(150);
  const vorher = await page.locator(".proj").first().evaluate((e) => getComputedStyle(e).backgroundColor);
  await page.locator(".proj").first().hover(); await page.waitForTimeout(150);
  const hover = await page.locator(".proj").first().evaluate((e) => getComputedStyle(e).backgroundColor);
  ok(vorher !== hover, th + ": Eintrag beim Überfahren eine Stufe heller/anders: " + vorher + " → " + hover);
  await snap("start-5-" + th);
}
await page.evaluate(async () => { await window.__vc.setTheme("dark"); });
ok(csp.length === 0, "Keine Meldungen der Inhaltsrichtlinie: " + csp.slice(0, 2).join(" | "));
console.log(`\n${n} Prüfungen, ${fails.length} Fehler`);
await browser.close();
process.exit(fails.length ? 1 : 0);
