// SPDX-License-Identifier: GPL-3.0-or-later
// Bilder zum Dialog „BASIC umwandeln“ (Beleg statt Handtest): gemischtes Programm (Modus + B160), Einbindung (Dateiname in eigener Zeile), Nummernprogramm, Englisch; je hell und dunkel.
// Aufruf: node tools/e2e-basicconv-bilder.mjs <Projektkopie examples/c64/basic_native> <Bilderordner>
import { webkit } from "playwright";
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
const [proj, out] = process.argv.slice(2);
if (!proj || !out) { console.error("Aufruf: node tools/e2e-basicconv-bilder.mjs <Projekt> <Bilderordner>"); process.exit(2); }
mkdirSync(out, { recursive: true });
const browser = await webkit.launch();
const page = await (await browser.newContext({ viewport: { width: 1280, height: 820 } })).newPage();
const fails = []; let n = 0;
const ok = (c, m) => { n++; if (!c) { fails.push(m); console.log("FEHLER:", m); } else console.log("ok:", m); };
page.on("pageerror", (e) => { fails.push("Seitenfehler " + e.message); console.log("Seitenfehler", e.message); });
await page.goto(`http://127.0.0.1:${process.env.VC_VITE ?? 1420}/?bridge=${process.env.VC_BRIDGE ?? 17420}`); await page.waitForFunction(() => window.__vc && window.__vc.ui.ready);
await page.evaluate(async () => { await window.__vc.setLanguage("de"); }); await page.waitForTimeout(300);
await page.evaluate(async (p) => { await window.__vc.openProject(p); }, proj); await page.waitForTimeout(600);
writeFileSync(join(proj, "umwandeln_nummern.bas"), '10 PRINT "START"\n20 GOTO 50\n30 PRINT "UEBERSPRUNGEN"\n40 END\n50 PRINT "ZIEL"\n60 GOTO 40\n');
writeFileSync(join(proj, "umwandeln_gemischt.bas"), "10 PRINT 1\n@x: PRINT 2\n");
writeFileSync(join(proj, "umwandeln_haupt.bas"), '10 GOSUB 100\n20 END\n#include "umwandeln_lib.bas"\n');
writeFileSync(join(proj, "umwandeln_lib.bas"), '100 PRINT "LIB"\n110 RETURN\n');
const oeffnen = async (rel) => { await page.evaluate(async (r) => { window.__vc.ui.extra = null; const t = await window.__vc.call("tree_reload"); window.__vc.ui.project.tree = t.tree; await window.__vc.openFile(r); }, rel); await page.waitForSelector(".cm-editor .cm-content"); await page.waitForTimeout(300); };
const befehl = (id) => page.evaluate((i) => { void window.__vc.runCommand(i); }, id);
const dlg = page.getByTestId("basicconv");
const szene = async (name, rel, cmd, check) => {
  await oeffnen(rel); await befehl(cmd); await dlg.waitFor(); await page.waitForSelector("[data-testid=basicconv-modus]"); await page.waitForTimeout(700);
  for (const th of ["light", "dark"]) {
    await page.evaluate((x) => window.__vc.setTheme(x), th); await page.waitForTimeout(250);
    if (th === "light" && check) await check();
    await page.screenshot({ path: join(out, `${name}-${th === "light" ? "hell" : "dunkel"}.png`) });
  }
  await page.evaluate((x) => window.__vc.setTheme(x), "light");
  await page.getByTestId("basicconv-abbrechen").click(); await page.waitForTimeout(250);
};
const text = () => page.getByTestId("basicconv").innerText();
await szene("1-gemischt-in-labels-umwandeln", "umwandeln_gemischt.bas", "basic_in_labels", async () => {
  const t = await text();
  ok(/Modus: gemischt/.test(t), "Modus: gemischt sichtbar");
  ok(/B160/.test(t), "Warnung B160 sichtbar");
  ok(/1 Zeile ändert sich/.test(t), "Text „1 Zeile ändert sich“ sichtbar: " + (t.match(/\d+ Zeilen? änder\w+ sich[^.]*\./)?.[0] ?? "-"));
});
await szene("2-haupt-mit-include-in-labels-umwandeln", "umwandeln_haupt.bas", "basic_in_labels", async () => {
  const t = await text();
  ok(/Eingebundene Dateien mitumwandeln/.test(t), "Option „Eingebundene Dateien mitumwandeln“ sichtbar");
  // Dateiname steht in eigener Zeile: kein Überlappen mit dem alten Zeilentext (Rechtecke der beiden Elemente schneiden sich nicht)
  const geo = await page.evaluate(() => [...document.querySelectorAll("[data-testid=basicconv-zeilen] li")].map((li) => { const f = li.querySelector(".fl"), o = li.querySelector(".old"); if (!f || !o) return null; const a = f.getBoundingClientRect(), b = o.getBoundingClientRect(); return a.bottom <= b.top + 1 || a.right <= b.left || b.right <= a.left; }).filter((x) => x !== null));
  ok(geo.length > 0 && geo.every(Boolean), "Dateiname überlappt den Zeilentext nicht (" + geo.length + " Zeilen aus der Einbindung)");
});
await szene("3-nummern-in-labels-umwandeln", "umwandeln_nummern.bas", "basic_in_labels", async () => {
  ok(/6 Zeilen ändern sich, 2 neue Labels/.test(await text()), "Vorschau: 6 Zeilen ändern sich, 2 neue Labels");
});
await page.evaluate(async () => { await window.__vc.setLanguage("en"); }); await page.waitForTimeout(400);
await szene("4-englisch-nummern-convert-to-labels", "umwandeln_nummern.bas", "basic_in_labels", async () => {
  const t = await text();
  ok(/Convert to labels/.test(await page.getByRole("dialog").innerText()) && /6 lines change, 2 new labels/.test(t), "Englischer Dialog: „6 lines change, 2 new labels“");
});
await szene("5-englisch-gemischt-mixed", "umwandeln_gemischt.bas", "basic_in_labels", async () => {
  const t = await text();
  ok(/Mode: mixed/.test(t) && /B160/.test(t) && /1 line changes/.test(t), "Englisch, gemischt: Mode: mixed, B160, „1 line changes“");
});
console.log(`${n} Prüfungen, ${fails.length} Fehler`);
await browser.close();
process.exit(fails.length ? 1 : 0);
