// SPDX-License-Identifier: GPL-3.0-or-later
// Ablauftest Registry (R0): Befehle aus einem Erweiterungspaket (Entwicklerschalter VC_EXTENSIONS_DEV), Release-Auswahl (VC_RELEASE) für Befehlspalette und Navigation.
// Aufruf: node tools/e2e-registry.mjs <Projektkopie (wird verändert!)>   (die App läuft mit VC_EXTENSIONS_DEV=<Ordner> und VC_RELEASE=<release.json>)
import { webkit } from "playwright";
import { join } from "node:path";
import { existsSync, writeFileSync } from "node:fs";

const [proj] = process.argv.slice(2);
if (!proj) { console.error("Aufruf: node tools/e2e-registry.mjs <Projektkopie>"); process.exit(2); }
const MOD = process.platform === "darwin" ? "Meta" : "Control";
const fails = []; let n = 0;
const ok = (c, m) => { n++; if (!c) { fails.push(m); console.log("FEHLER:", m); } else console.log("ok:", m); };
const browser = await webkit.launch();
const page = await (await browser.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
page.on("pageerror", (e) => { fails.push("Seitenfehler " + e.message); console.log("Seitenfehler", e.message); });
await page.goto("http://127.0.0.1:1420/?bridge=17420"); await page.waitForFunction(() => window.__vc && window.__vc.ui.ready);
// Start mit Pfaden: die App wurde mit `<Projekt> --open hal_c64.asm --view disk` gestartet; die Oberfläche öffnet das Projekt direkt
await page.waitForFunction(() => window.__vc.ui.project && window.__vc.ed.tabs.length > 0, null, { timeout: 8000 }).catch(() => {}); await page.waitForTimeout(800);
ok(await page.evaluate(() => window.__vc.ui.project?.root) === proj && (await page.evaluate(() => window.__vc.ed.tabs.map((x) => x.rel))).includes("hal_c64.asm"), "Start mit Pfad: Projekt offen, Datei aus --open als Reiter: " + await page.evaluate(() => window.__vc.ui.project?.root));
ok((await page.evaluate(() => window.__vc.ui.mode + "/" + window.__vc.ui.sub.daten)) === "daten/disk", "--view disk zeigt den Disk-Editor");
await page.evaluate(async () => { await window.__vc.setLanguage("de"); });
const call = (c, a) => page.evaluate(([c, a]) => window.__vc.call(c, a), [c, a ?? null]);

// Kern: Befehlstabelle, Release, Auskunft
const tab = await call("commands_table"); const ext = tab.find((c) => c.id === "ext:autor.demo:hallo");
ok(ext && ext.source === "autor.demo" && ext.title === "Hallo sagen" && ext.menu === "Projekt" && ext.submenu === "Erweiterungen" && ext.needs === "none", "Befehl aus dem Paket steht in der Tabelle mit Herkunft, Titel und Untermenü: " + JSON.stringify(ext));
ok(!tab.some((c) => c.id === "ext:autor.demo:sprite_nur"), "Befehl für einen abgewählten Editor (sprite) fehlt");
ok(!tab.some((c) => c.id === "png_laden") && tab.some((c) => c.id === "tracker_noten") && tab.some((c) => c.id === "speichern"), "Release nur Tracker und Disk: Sprite-Befehl weg, Tracker- und allgemeine Befehle da");
const f = await call("features_get"); ok(!f.all && f.studios.find((s) => s.id === "sprite").on === false && f.studios.find((s) => s.id === "disk").on === true, "features_get nennt die Auswahl");
const info = await call("registry_info");
ok(info.hasExtensions && info.packages.some((p) => p.id === "autor.demo" && p.version === "1.2.3") && info.sources.some((s) => s.kind === "program") && info.sources.some((s) => s.kind === "user"), "registry_info: Programm und Benutzerquelle, Paket mit Version");
ok(info.problems.some((p) => /passt nicht|ohne|json|expected/i.test(p.text)), "registry_info meldet das kaputte Paket: " + info.problems.map((p) => p.origin + ": " + p.text).join(" | ").slice(0, 120));
ok(info.commands.some((c) => c.id === "ext:autor.demo:hallo"), "Auskunft nennt die Erweiterungsbefehle");

// Beispiele und Projektstrukturen aus dem Paket (R0.5)
const bsp = await call("examples_list"); const b = bsp.find((x) => x.title === "Demo-Beispiel");
ok(b && b.source === "autor.demo" && b.readonly === true && bsp.some((x) => !x.source), "Beispielliste: Beispiel aus dem Paket mit Herkunft (nur Kopieren), Beispiele des Programms ohne");
const np = await call("newproj_info"); ok(np.structures.some((x) => x.id === "demo-struktur" && x.name === "Demo-Struktur") && np.structures.some((x) => x.id === "simple" || x.id === "einfach"), "Neues Projekt: Struktur aus dem Paket steht neben denen des Programms");

// Oberfläche: Palette
await page.evaluate(async (p) => { await window.__vc.openProject(p); }, proj); await page.waitForTimeout(800);
await page.keyboard.press(MOD + "+k"); await page.waitForSelector("#pal-list"); await page.keyboard.type("hallo"); await page.waitForTimeout(300);
const eintrag = page.locator("#pal-list [role=option]").first();
ok(/Hallo sagen/.test(await eintrag.innerText()) && /autor\.demo/.test(await eintrag.innerText()), "Palette zeigt den Erweiterungsbefehl mit seiner Herkunft: " + (await eintrag.innerText()).replace(/\s+/g, " "));
await page.keyboard.press("Enter"); await page.waitForTimeout(500);
ok(/Erweiterungen|extensions|Werkzeuge/i.test(await page.locator(".toasts").innerText()), "Ausführen meldet, dass die Werkzeuge folgen");
await page.keyboard.press(MOD + "+k"); await page.waitForSelector("#pal-list"); await page.keyboard.type("PNG"); await page.waitForTimeout(300);
ok(!(await page.locator("#pal-list").innerText()).includes("PNG-Bild in Sprite"), "Palette bietet „PNG-Bild in Sprite laden“ nicht an (Sprite-Editor nicht im Release)"); await page.keyboard.press("Escape");

// Navigation: Grafik fehlt ganz (keine ihrer Unterseiten im Release); Daten ohne Speicher
const modi = await page.locator("nav button, nav [role=radio], nav [role=tab]").allInnerTexts();
ok(!modi.some((t) => /Grafik/.test(t)) && modi.some((t) => /Daten/.test(t)) && modi.some((t) => /Musik/.test(t)), "Hauptbereich Grafik fehlt (kein Editor davon im Release), Musik und Daten sind da: " + modi.join(", ").replace(/\n/g, " "));
await page.evaluate(() => window.__vc.runCommand("nav:daten")); await page.waitForTimeout(300);
const subs = await page.locator("[aria-label*=Unter], nav + * button").allInnerTexts().catch(() => []);
ok(await page.evaluate(() => window.__vc.ui.mode) === "daten", "Daten lässt sich öffnen");

// Datei eines abgewählten Editors öffnet als Text mit Hinweis
const sprite = join(proj, "hero.vcsprite"); if (!existsSync(sprite)) writeFileSync(sprite, JSON.stringify({ format: 1, kind: "sprite", profile: "c64.sprite", name: "hero", frames: [] }));
await page.evaluate(async () => { const r = await window.__vc.call("tree_reload"); window.__vc.ui.project.tree = r.tree; await window.__vc.runCommand("nav:code"); }); await page.waitForTimeout(300);
await page.evaluate(() => window.__vc.openFile("hero.vcsprite")); await page.waitForTimeout(700);
ok(/Text/.test(await page.locator(".toasts").innerText()) && (await page.locator(".tab .nm").allInnerTexts()).includes("hero.vcsprite") && !(await page.evaluate(() => window.__vc.sp.view)), "Sprite-Datei öffnet als Text mit Hinweis, nicht im Sprite-Editor");
console.log(`\n${n} Prüfungen, ${fails.length} Fehler`);
await browser.close();
process.exit(fails.length ? 1 : 0);
