// SPDX-License-Identifier: GPL-3.0-or-later
// Handbuchbilder aus der neuen Oberfläche: Playwright (WebKit) über die Prüfbrücke gegen den echten Kern, 1600×1007, Deutsch, dunkel, ohne Marken.
// Jede Szene hat die Kennung aus docs/handbuch/bilder/manifest.json; geschrieben wird `<Ausgabe>/<kennung>.png` (nie in docs/ – die Übernahme ins Handbuch ist eine eigene Entscheidung).
// Aufruf: node tools/handbuchbilder.mjs <Ausgabe> <Arbeitsordner> [kennung …]
//   Arbeitsordner: wird angelegt und mit Kopien der Beispielprojekte gefüllt. Kern mit eigenem HOME, VC_NO_WIZARD=1, VC_TEST_FAKE_DEBUG=1 (Debug-Bilder sind Beispielansichten ohne Emulator),
//   VC_TESTBRIDGE=17420 VC_TEST_NO_SPAWN=1 VC_NO_SAVE=1; Vite mit VC_CSP=release auf Port 1420. Siehe ui/tools/handbuchbilder.sh.
import { webkit } from "playwright";
import { execFileSync } from "node:child_process";
import { cpSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
const [out, arbeit, ...nur] = process.argv.slice(2);
// Sprache der Bilder: HB_SPRACHE=de (Vorgabe) oder en; Beschriftungen, nach denen Szenen suchen, kommen aus dem Katalog der gewählten Sprache
const SPRACHE = process.env.HB_SPRACHE === "en" ? "en" : "de";
const KATALOG = Object.fromEntries(readFileSync(resolve(import.meta.dirname, `../../crates/vc-i18n/catalog/${SPRACHE}/ui.ftl`), "utf8").split(/\r?\n/).map((z) => z.match(/^([a-z0-9_.-]+)\s*=\s*(.*)$/)).filter(Boolean).map((m) => [m[1], m[2]]));
const T = (k) => KATALOG[k] ?? k;
if (!out || !arbeit) { console.error("Aufruf: node tools/handbuchbilder.mjs <Ausgabe> <Arbeitsordner> [kennung …]"); process.exit(2); }
mkdirSync(out, { recursive: true });
const EX = resolve(import.meta.dirname, "../../examples"); const P = (n) => join(arbeit, n); const VORLAGEN = resolve(import.meta.dirname, "../../tools/paket-vorlagen");
if (existsSync(arbeit)) rmSync(arbeit, { recursive: true, force: true });
for (const [von, nach] of [["c64/tests_demo", "tests"], ["c64/jumprun", "jumprun"], ["c64/music_demo", "music"], ["c64/sprite_demo", "sprites"], ["c64/gfx_demo", "gfx"], ["mega65/pal_demo", "pal65"], ["c64/basic_native", "basic"], ["atari/music_demo", "amusic"], ["atari/pmg_ship", "pmg"], ["mega65/play_demo", "play"], ["mega65/hello", "hello65"], ["atari/colorbars", "acolor"], ["c64/border", "border"], ["c64/basic_langenamen", "langnamen"], ["c64/troxx", "troxx"], ["atari/turbo_basic", "turbo"], ["atari/gfx_demo", "agfx"]]) cpSync(join(EX, von), P(nach), { recursive: true });
cpSync(join(EX, "c64/basic_native/sterne.bas"), P("sprites/sterne.bas"));
cpSync(join(VORLAGEN, "c64/basic_test"), P("basictest"), { recursive: true });
const browser = await webkit.launch(); const ctx = await browser.newContext({ viewport: { width: 1600, height: 1007 } }); const page = await ctx.newPage();
const ergebnis = []; const w = (ms = 500) => page.waitForTimeout(ms);
const V = (f, x) => Promise.race([page.evaluate(f, x), new Promise((_, rej) => setTimeout(() => rej(new Error("Zeitüberschreitung in page.evaluate: " + String(f).slice(0, 120))), 90000))]);
const L = SPRACHE;
const zeigen = async (id, f) => {
  if (nur.length && !nur.includes(id)) return;
  try { const r = await f(); if (!r?.fertig) { await w(450); await V(() => { window.__vc.ui.toasts.splice(0); }); await w(100); await (r?.crop ?? page).screenshot({ path: join(out, id + ".png") }); } ergebnis.push([id, "ok"]); console.log("ok:", id); }
  catch (e) { ergebnis.push([id, "FEHLER " + String(e.message).split("\n")[0]]); console.log("FEHLER:", id, String(e.message).split("\n")[0]); }
  if (process.env.HB_MODUS !== "assistent") await page.keyboard.press("Escape").catch(() => {});   // im Assistenten würde Esc ihn abbrechen
  await w(150);
};
const start = async () => { await page.goto("http://127.0.0.1:1420/?bridge=17420"); await page.waitForFunction(() => window.__vc && window.__vc.ui.ready); await V(async (l) => { await window.__vc.setLanguage(l); window.__vc.setTheme("dark"); }, L); await w(500); };
const projekt = async (n, datei) => {
  let fertig = false; const pr = V(async ([p, f]) => { await window.__vc.openProject(p); if (f) await window.__vc.openFile(f); }, [P(n), datei]).finally(() => { fertig = true; });
  pr.catch(() => {}); let zn = 0;
  while (!fertig) { await w(400); if (++zn === 40) { console.log("HAENGER projekt", zn, await page.locator("dialog[open]").allInnerTexts().catch(() => "?")); await page.screenshot({ path: join(out, "_haenger.png") }).catch(() => {}); } const d = page.locator("dialog[open] button, [data-ask=discard]").filter({ hasText: /^Verwerfen$|^Nicht speichern$|Änderungen verwerfen/ }); if (await d.count().catch(() => 0)) await d.first().click().catch(() => {}); }
  await pr; await w(900); };
const nav = (id) => V(async (i) => { await window.__vc.runCommand(i); }, id).then(() => w(500));
const zeile = async (text) => { const l = page.locator(".cm-line", { hasText: text }).first(); await l.scrollIntoViewIfNeeded(); return l; };

if (process.env.HB_MODUS === "assistent") {
  // Erstkonfiguration in einem Kern ohne Einstellungen: Startbild und die Schritte des Einrichtungsassistenten
  await zeigen("start-splash", async () => { await page.goto("http://127.0.0.1:1420/?bridge=17420&splash=1"); await page.waitForSelector("[data-splash]"); await w(200); });
  await start(); await w(1200);   // der Assistent öffnet sich nicht von selbst: die Willkommenskarte der Startseite startet ihn
  await page.locator("[data-start=welcome-setup]").click({ timeout: 15000 }); await w(800);
  await page.waitForSelector("[data-wz=assistent]", { timeout: 20000 });
  const THEMEN = [["assistent-design", "design"], ["assistent-systeme", "systems"], ["assistent-toolchain", "toolchain"], ["assistent-projekte", "projects"], ["assistent-zusammenfassung", "summary"]];
  const weg = page.locator("[data-wz=weg-schnell]"); await zeigen("assistent-weg-waehlen", async () => { await w(300); });
  if (await weg.count()) { await weg.click(); await w(400); }
  for (const [id, thema] of THEMEN) {
    for (let i = 0; i < 8 && !(await page.locator(`[data-wz=thema-${thema}]`).isVisible().catch(() => false)); i++) { await page.locator("[data-wz=weiter]").click().catch(() => {}); await w(500); }
    await zeigen(id, async () => { await w(400); });
  }
  await page.locator("[data-wz=uebernehmen]").click().catch(() => {}); await page.waitForSelector("[data-wz=angewandt]", { timeout: 20000 }).catch(() => {});
  await page.locator("[data-wz=weiter]").click().catch(() => {}); await w(500);
  await zeigen("assistent-fertig", async () => { await w(300); });
} else {
await start();
await zeigen("willkommen-seite", async () => { await w(300); });
await zeigen("neues-projekt-dialog", async () => { await nav("projekt_neu"); await w(900); });
await page.keyboard.press("Escape"); await w(200);
await projekt("jumprun", "jumprun.asm");
await zeigen("editor-syntax", async () => { await V(() => { window.__vc.ed.dock = null; }); });
await zeigen("editor-hilfe-f1", async () => { await V(async () => { await window.__vc.runCommand("nav:code"); await window.__vc.openFile("jumprun.asm"); window.__vc.ed.dock = null; }); await w(600); await page.locator(".cm-content").click(); await page.keyboard.press("Meta+ArrowUp"); await w(300); const mn = page.locator(".tok-mnemonic", { hasText: "jsr" }).first(); await mn.scrollIntoViewIfNeeded(); await mn.click(); await w(300); await V(async () => { await window.__vc.runCommand("hilfe"); }); await w(1200); await page.mouse.move(5, 5); await w(300); });   // F1 = Befehl „hilfe“: öffnet die Hilfeseite zum Wort am Cursor
await page.mouse.move(5, 5); await w(300);
await zeigen("editor-suchen", async () => { await page.locator(".cm-content").click(); await page.keyboard.press("Meta+f"); await page.keyboard.type("jsr"); await w(400); });
await zeigen("editor-gehe-zu", async () => { await nav("gehe_zu_zeile"); });
await projekt("basic", "sterne.bas"); await zeigen("editor-kontextmenue", async () => { const l = page.locator(".cm-line").nth(3); await l.click({ button: "right" }); await w(400); });
await page.keyboard.press("Escape"); await projekt("jumprun", "jumprun.asm");
await zeigen("editor-vervollstaendigung", async () => { await page.locator(".cm-content").click(); await page.keyboard.press("Meta+ArrowDown"); await page.keyboard.type("\n  lda"); await w(900); });
await page.keyboard.press("Escape"); for (let i = 0; i < 8; i++) await page.keyboard.press("Meta+z"); await w(300);   // die Szene darf den Text nicht verändern
await zeigen("befehlspalette", async () => { await V(() => { window.__vc.ui.palette = true; }); await w(400); });
await V(() => { window.__vc.ui.palette = false; });
await zeigen("bauprotokoll", async () => { await nav("bauen"); await page.waitForFunction(() => !window.__vc.bs?.running, null, { timeout: 60000 }).catch(() => {}); await w(1500); await V(() => { window.__vc.ed.dock = "output"; }); await w(600); });
await zeigen("ui-gesamt", async () => { await V(() => { window.__vc.ui.panel = true; window.__vc.ed.dock = "output"; }); await w(700); });   // mit Ablage (Ausgabe) und Panel Maschine
await zeigen("panel-maschine", async () => ({ crop: page.locator("[data-area=panel]") }));
await zeigen("probleme-panel", async () => { await V(() => { window.__vc.ed.dock = "problems"; }); });
await V(() => { window.__vc.ed.dock = null; });
await zeigen("projekt-uebersicht", async () => { await nav("nav:projekt:uebersicht"); });
await zeigen("projekt-einstellungen", async () => { await nav("nav:projekt:allgemein"); });
await zeigen("einstellungen", async () => { await page.locator("[data-act=einstellungen]").click(); await w(600); await page.locator("[data-es=pagebtn][data-s=allgemein]").click(); });
await zeigen("einstellungen-emulator", async () => { await page.locator("[data-es=pagebtn][data-s=ziele]").click(); await w(600); await page.locator("[data-zl=system]").first().click().catch(() => {}); });
await zeigen("argumente", async () => { await page.getByText(T("ziele-args-titel")).first().click(); await w(700); });
await zeigen("einstellungen-roms", async () => { await page.locator("[data-es=pagebtn][data-s=roms]").click(); await w(900); });
await zeigen("einstellungen-sicherung", async () => { await page.locator("[data-es=pagebtn][data-s=sicherung]").click(); await w(700); });
await zeigen("einstellungen-info", async () => { await page.locator("[data-es=pagebtn][data-s=info]").click(); await w(900); });
await zeigen("einstellungen-update", async () => { if (!(await page.locator("[data-es=pagebtn]").count())) { await page.locator("[data-act=einstellungen]").click(); await w(600); } await page.locator("[data-es=pagebtn][data-s=allgemein]").click(); await w(600); return { crop: page.locator("h3", { hasText: T("einst-updates") }).first().locator("xpath=..") }; });
await zeigen("feedback-dialog", async () => { await page.locator("[data-info=feedback]").click(); await w(900); });
await page.keyboard.press("Escape"); await w(300);
await V(async () => { await window.__vc.runCommand("startseite"); }); await start();
await projekt("sprites", "graphics/hero.vcsprite");
await zeigen("sprite-c64", async () => { await w(500); });
await projekt("gfx", "graphics/font.vcchar");
await zeigen("zeichensatz-ueberblick", async () => { await w(300); });
await V(async () => { await window.__vc.openFile("graphics/title.vcscreen"); }); await w(800);
await zeigen("bildschirm-editor", async () => { await w(300); });
await V(async () => { await window.__vc.openFile("graphics/level_tiles.vctiles"); }); await w(800);
await zeigen("kachel-editor", async () => { await w(300); });
await projekt("pal65", "graphics/pal65.vcpal");
await zeigen("palette-mega65", async () => { await w(300); });
await projekt("music", "music/tune.vcsong");
await zeigen("tracker-ueberblick", async () => { await w(300); });
await zeigen("tracker-muster", async () => { await page.locator("[data-tk=zelle]").nth(5).click().catch(() => {}); });
// --- zweite Batch: Dialoge, Fehler, Tracker, Disk, Tests, Export ---
await projekt("jumprun", "jumprun.asm");
await zeigen("datei-neu-dialog", async () => { await V(async () => { await window.__vc.runCommand("neue_datei"); }); await w(500); });
await page.keyboard.press("Escape"); await w(200);
await zeigen("datei-umbenennen-dialog", async () => { await V(() => { window.__vc.ui.selected = "hal_c64.asm"; }); await V(async () => { await window.__vc.runCommand("datei_umbenennen"); }); await w(500); });
await page.keyboard.press("Escape"); await w(200);
await V(async () => { await window.__vc.openFile("jumprun.asm"); await window.__vc.runCommand("nav:code"); }); await w(500);
await zeigen("asm-fehler", async () => { await page.locator(".cm-content").click(); await page.keyboard.press("Meta+ArrowDown"); await page.keyboard.type("\n  lda #$zz\n  xyz 5\n"); await w(1500); await V(() => { window.__vc.ed.dock = "problems"; }); await w(500); });
await zeigen("editor-fehlerzeile", async () => { await V(() => { window.__vc.ed.dock = null; }); await w(300); });
await page.keyboard.press("Escape"); for (let i = 0; i < 12; i++) await page.keyboard.press("Meta+z"); await w(400);
await V(async () => { await window.__vc.openFile("graphics/hero.vcsprite").catch(() => {}); }); await projekt("sprites", "graphics/hero.vcsprite");
await zeigen("sprite-animation", async () => { await page.locator("[data-frame='0']").click(); await page.locator("[data-frame='1']").click({ modifiers: ["Shift"] }); await w(300); await page.locator("[data-act=anim-from]").click(); await page.waitForSelector("dialog[open] [data-anim-from=name]"); await page.locator("[data-anim-from=name]").fill("laufen"); await page.locator("[data-anim-from=create]").click(); await w(1200); });   // Animation aus den markierten Bildern, Vorschau läuft
await zeigen("grafik-exportieren", async () => { await page.getByRole("button", { name: new RegExp(T("export-titel")) }).first().click(); await w(900); });
await page.keyboard.press("Escape"); await w(200);
await projekt("music", "music/tune.vcsong");
await zeigen("tracker-instrument", async () => { await page.locator("[data-tk=instrumente] [role=option]").nth(1).click(); await w(600); });   // Bass: Sägezahn, zeigt die Instrumentkarte (anders als tracker-muster)
await zeigen("tracker-huellkurve", async () => { await w(300); });
await zeigen("tracker-hoerprobe", async () => { await page.locator("[data-tk=zelle]").nth(3).click(); await page.keyboard.press("KeyZ"); await w(120); await page.screenshot({ path: join(out, "tracker-hoerprobe.png") }); return { fertig: true }; });   // Taste leuchtet nur kurz
await zeigen("zeichensatz-standard", async () => { await projekt("gfx", "graphics/font.vcchar"); });
await zeigen("kachel-karte", async () => { await projekt("gfx", "graphics/level.vcscreen"); await V(async () => { window.__vc.ui.toasts.splice(0); await window.__vc.openFile("graphics/level.vcscreen"); window.__vc.ui.toasts.splice(0); }); await w(900); });   // Karten-Editor (Bank mit Levels), ohne Meldungen früherer Szenen
// Disk: Abbild mit vcc erzeugen und im Disk-Editor öffnen
try { const vcc = resolve(import.meta.dirname, "../../target/debug/vcc"); cpSync(P("jumprun"), P("diskproj"), { recursive: true }); const arg = ["disk", "d64", "-o", P("diskproj/beispiel.d64"), "--name", "HUGO", "--id", "01"];
  for (const [i, n] of ["HUGO", "GUMBO", "MUSIK"].entries()) { writeFileSync(P(`diskproj/d${i}.bin`), Buffer.alloc(700 * (i + 1), i + 1)); arg.push("--add", `${P(`diskproj/d${i}.bin`)}:${n}`); }
  execFileSync(vcc, arg, { stdio: "ignore" }); } catch { /* ohne vcc keine Disk-Bilder */ }
await projekt("diskproj", "beispiel.d64");
await zeigen("disk-studio", async () => { await w(500); });
await projekt("tests", "tests_demo.asm"); await nav("nav:code");
await zeigen("tests-panel", async () => { await nav("bauen"); await page.waitForFunction(() => !window.__vc.bs?.running, null, { timeout: 60000 }).catch(() => {}); await w(2500); await V(async () => { await window.__vc.runCommand("tests_ausfuehren"); }); await w(4000); await V(() => { window.__vc.ed.dock = "tests"; }); await w(800); });   // Tests nach einem Lauf: Liste mit Ergebnissen
await zeigen("tests-ergebnis", async () => { await V(() => { window.__vc.ed.dock = "tests"; }); const z = page.getByText(/tests_demo: Zyklen next_color ≤ 18/).first(); if (await z.count()) { await z.scrollIntoViewIfNeeded(); await z.click(); } else await page.getByText(/tests_demo: Zyklen irq/).first().click(); await w(800); });   // ein Test gewählt: Erwartung und Register rechts
await projekt("jumprun", "jumprun.asm");
await zeigen("debug-ueberblick", async () => { await nav("nav:debug"); await page.locator("[data-dbg=start]").click(); await page.waitForSelector("[data-dbg=register]", { timeout: 90000 }); await w(1200); });
await zeigen("debug-register", async () => ({ crop: page.locator("[data-dbg=register]").locator("xpath=ancestor::section[1]") }));
await zeigen("debug-disassembler", async () => ({ crop: page.locator("[data-dbg=disasm]") }));
await zeigen("debug-speicher", async () => ({ crop: page.locator("[data-dbg=speicher]").locator("xpath=ancestor::section[1]") }));
await V(async () => { await window.__vc.beobachtenSetzen(["a", "x", "$d020"]); }); await w(1200);
await zeigen("debug-beobachten", async () => ({ crop: page.locator("[data-dbg=beobachten]").locator("xpath=ancestor::section[1]") }));
await page.locator("[data-dbg=ende]").click().catch(() => {}); await w(700);
await nav("nav:daten:speicher"); await zeigen("memory-map", async () => { await w(900); });
await nav("nav:daten:hex"); await zeigen("hex-ansicht", async () => { await page.locator("[data-hx=file]", { hasText: "jumprun.prg" }).click(); await w(900); });
await projekt("basictest", "test.bas"); await V(() => { window.__vc.ed.dock = null; window.__vc.ui.panel = false; });
await zeigen("basic-asm-direktive", async () => { await page.locator(".cm-content").click(); await page.keyboard.press("Meta+ArrowDown"); await page.keyboard.type("\n\n\n\n\n\n#asm \"code.asm\""); await w(600); const l = page.locator(".cm-line", { hasText: "#asm" }).last(); await l.scrollIntoViewIfNeeded(); const b = await l.boundingBox(); await page.mouse.move(b.x + 14, b.y + b.height / 2); await page.waitForSelector(".cm-tooltip .vc-card", { timeout: 6000 }).catch(() => {}); await w(500); });   // #asm am Dateiende nach Leerzeilen: die Karte verdeckt nur leere Zeilen
await page.mouse.move(5, 5); await w(300);
await page.mouse.move(5, 5); await page.keyboard.press("Escape"); for (let i = 0; i < 12; i++) await page.keyboard.press("Meta+z"); await w(300);
await zeigen("basic-umwandeln-dialog", async () => { await V(async () => { await window.__vc.runCommand("basic_in_nummern"); }); await w(900); });
// --- dritte Batch (Paket 5): Oberfläche, Editor, Debug ---
const code = async (n, f) => { await projekt(n, f); await V(async () => { await window.__vc.runCommand("nav:code"); window.__vc.ed.dock = null; window.__vc.ui.panel = false; window.__vc.ui.sidebar = true; }); await w(500); };
const einst = async (seite) => { if (!(await page.locator("[data-es=pagebtn]").count())) { await page.locator("[data-act=einstellungen]").click(); await w(600); } await page.locator(`[data-es=pagebtn][data-s=${seite}]`).click(); await w(700); };
await zeigen("editor-falten", async () => { await code("jumprun", "jumprun.asm"); await page.locator(".cm-content").click(); await page.keyboard.press("Meta+ArrowUp"); await page.keyboard.press("Meta+Alt+["); await w(500); await page.mouse.move(5, 5); });
await zeigen("ui-zonen", async () => { await code("jumprun", "jumprun.asm"); await V(() => { window.__vc.ui.sidebar = false; window.__vc.ui.panel = false; window.__vc.ed.dock = null; }); await w(600); });
await V(() => { window.__vc.ui.sidebar = true; });
await zeigen("ui-reiter", async () => { await code("jumprun", "jumprun.asm"); await V(async () => { await window.__vc.openFile("hal_c64.asm"); await window.__vc.openFile("README.md"); await window.__vc.openFile("jumprun.asm"); }); await w(600); await page.locator(".cm-content").click(); await page.keyboard.type(" "); await w(400); return { crop: page.locator(".tbar").first() }; });
await page.keyboard.press("Meta+z"); await w(300);
await zeigen("projektbaum", async () => { await code("jumprun", "jumprun.asm"); const lib = page.locator("[data-area=seitenleiste]").getByText("lib", { exact: true }).first(); await lib.click().catch(() => {}); await w(500); return { crop: page.locator("section[data-area=seitenleiste]").first() }; });
await zeigen("ui-modus-code", async () => { await code("jumprun", "jumprun.asm"); await nav("bauen"); await page.waitForFunction(() => !window.__vc.bs?.running, null, { timeout: 60000 }).catch(() => {}); await w(1500); await V(() => { window.__vc.ui.panel = true; window.__vc.ed.dock = "output"; }); await w(700); });
await zeigen("segmente-baenke", async () => { await V(() => { window.__vc.ui.panel = true; }); await w(500); return { crop: page.locator("[data-area=panel]") }; });
await zeigen("ausgabe-panel", async () => { await V(() => { window.__vc.ed.dock = "output"; }); await w(500); return { crop: page.locator("[data-area=schublade]") }; });
await zeigen("editor-ersetzen", async () => { await code("jumprun", "jumprun.asm"); await page.locator(".cm-content").click(); await V(async () => { await window.__vc.runCommand("ersetzen"); }); await w(500); await page.locator(".cm-search input[name=search]").fill("jsr"); await page.locator(".cm-search input[name=replace]").fill("call_"); await w(400); });
await zeigen("theme-hell-dunkel", async () => {
  await code("jumprun", "jumprun.asm"); await V(() => { window.__vc.ui.panel = true; window.__vc.ed.dock = "output"; }); await V(async () => { await window.__vc.setTheme("dark"); }); await w(700);
  const d = (await page.screenshot()).toString("base64"); await V(async () => { await window.__vc.setTheme("light"); }); await w(900); const h = (await page.screenshot()).toString("base64");
  await V(async () => { await window.__vc.setTheme("dark"); }); await w(500);
  const p2 = await ctx.newPage(); await p2.setViewportSize({ width: 1600, height: 1007 });
  await p2.setContent(`<body style="margin:0"><canvas id=c width=1600 height=1007></canvas></body>`);
  await p2.evaluate(async ([d, h]) => { const li = (s) => new Promise((r) => { const i = new Image(); i.onload = () => r(i); i.src = "data:image/png;base64," + s; }); const c = document.getElementById("c").getContext("2d"); const a = await li(d), b = await li(h); c.drawImage(a, 0, 0, 1600, 1007); c.save(); c.beginPath(); c.moveTo(1000, 0); c.lineTo(1600, 0); c.lineTo(1600, 1007); c.lineTo(600, 1007); c.closePath(); c.clip(); c.drawImage(b, 0, 0, 1600, 1007); c.restore(); c.strokeStyle = "#8b7cf6"; c.lineWidth = 3; c.beginPath(); c.moveTo(1000, 0); c.lineTo(600, 1007); c.stroke(); }, [d, h]);
  await p2.locator("#c").screenshot({ path: join(out, "theme-hell-dunkel.png") }); await p2.close(); return { fertig: true }; });
const orteWeg = () => page.evaluate(() => { document.querySelectorAll("[data-rm=orte]").forEach((e) => { (e.closest(".card") ?? e).style.display = "none"; }); });   // Suchpfade enthalten den Ordner des Entwicklungsrechners: nicht ins Bild
await zeigen("rom-verzeichnis", async () => { await einst("roms"); await orteWeg(); await w(300); });
await zeigen("roms-einrichten", async () => { await einst("allgemein"); await page.getByRole("button", { name: T("wiz-teil-roms") }).click(); await page.waitForSelector("[data-wz=assistent]", { timeout: 20000 }); await page.locator("[data-wz=weiter]").click(); await w(1200); await orteWeg(); await w(300); });
const debugStart = async () => { await code("tests", "tests_demo.asm"); await nav("bauen"); await page.waitForFunction(() => !window.__vc.bs?.running, null, { timeout: 60000 }).catch(() => {}); await w(1500); await nav("nav:debug"); await page.locator("[data-dbg=start]").click(); await page.waitForSelector("[data-dbg=register]", { timeout: 90000 }); await w(1200); };
const karteVon = (sel) => page.locator(sel).first().locator("xpath=ancestor::section[1]");
await zeigen("ui-modus-debug", async () => { await debugStart(); });
await zeigen("debug-pc-zeile", async () => ({ crop: page.locator("[data-dbg=disasm]") }));
await zeigen("debug-bedingung", async () => { await page.locator("[data-dbg=adr]").fill("$080D"); await page.locator("[data-dbg=bed]").fill("($fb) == 9"); await page.locator("[data-dbg=punkt-neu]").click(); await w(800); await page.locator("[data-dbg=adr]").fill(""); return { crop: karteVon("[data-dbg=adr]") }; });
await zeigen("debug-datenhaltepunkt", async () => { await page.locator("[data-dbg=art]").evaluate((e) => e.querySelectorAll("option").forEach((o) => { o.disabled = false; })); await page.locator("[data-dbg=art]").selectOption("write"); await page.locator("[data-dbg=adr]").fill("$fb"); await page.locator("[data-dbg=bed]").fill(""); await w(500); return { crop: karteVon("[data-dbg=adr]") }; });
await zeigen("debug-momentaufnahme", async () => { await page.locator("[data-dbg=erkennen]").click(); await w(1200); return { crop: karteVon("[data-dbg=erkennen]") }; });
await zeigen("emulator-reiter", async () => { await nav("nav:code"); await V(() => { window.__vc.ui.panel = true; }); await w(500); await page.locator("[data-mp=bild]").click().catch(() => {}); await w(600); return { crop: page.locator("[data-area=panel]") }; });
await page.locator("[data-dbg=ende]").click().catch(() => {}); await w(700);
await zeigen("debug-haltepunkt-gutter", async () => { await code("jumprun", "jumprun.asm"); const l = page.locator(".cm-gutter.cm-lineNumbers .cm-gutterElement", { hasText: /^48$/ }).first(); await l.scrollIntoViewIfNeeded(); await l.click(); await w(500); await page.mouse.move(5, 5); await w(300); });
await zeigen("debug-test-sitzung", async () => { await projekt("tests", "tests_demo.asm"); await nav("nav:code"); await nav("bauen"); await page.waitForFunction(() => !window.__vc.bs?.running, null, { timeout: 60000 }).catch(() => {}); await w(2500); await V(async () => { await window.__vc.runCommand("tests_ausfuehren"); }); await w(4000); await V(() => { window.__vc.ed.dock = "tests"; }); await w(800); const z = page.getByText(/tests_demo: Zyklen next_color ≤ 18/).first(); if (await z.count()) await z.click(); else await page.getByText(/tests_demo: Zyklen irq/).first().click(); await w(500); await page.locator("[data-tests=debuggen]").click(); await page.waitForSelector("[data-dbg=test-sitzung]", { timeout: 90000 }); await w(1200); });
await page.locator("[data-dbg=ende]").click().catch(() => {}); await w(700);
// --- Grafik, Musik, Disk, Projekt ---
await zeigen("zeichensatz-standard", async () => { await projekt("gfx", "graphics/font.vcchar"); await V(() => { document.querySelector("[data-cs=vorlage]")?.scrollIntoView({ block: "center" }); }); await w(400); });
await zeigen("zeichensatz-multicolor", async () => { await projekt("jumprun", "graphics/chars.vcchar"); await w(500); });
await zeigen("sprite-c64-multicolor", async () => { await projekt("sprites", "graphics/hero.vcsprite"); await w(500); const m = page.locator("[data-flag=mc]").first(); await m.scrollIntoViewIfNeeded().catch(() => {}); return { crop: m.locator("xpath=ancestor::*[contains(@class,'card') or self::aside or self::section][1]") }; });
await zeigen("sprite-atari-pmg", async () => { await projekt("pmg", "graphics/ship.vcpmg"); await w(800); });
await zeigen("palette-atari", async () => { await projekt("pmg", "graphics/ship.vcpmg"); await w(800); return { crop: page.locator("[data-sp=palette]").first().locator("xpath=ancestor::aside[1] | ancestor::section[1]") }; });
await zeigen("kachel-bank", async () => { await projekt("gfx", "graphics/level_tiles.vctiles"); await w(800); });
await zeigen("tracker-pokey", async () => { await projekt("amusic", "music/tune_pokey.vcsong"); await w(600); await page.locator("[data-tk=instrumente] [role=option]").first().click(); await w(600); });
await zeigen("play-tracker", async () => { await projekt("play", "music/tune_play.vcplay"); await w(900); });
await zeigen("tracker-sid-pokey", async () => { await projekt("music", "music/tune.vcsong"); await w(500); await page.locator("[data-act=konvertieren]").first().click(); await w(900); });
await zeigen("disk-neu", async () => { await projekt("diskproj", "beispiel.d64"); await w(600); await page.getByRole("button", { name: /Neue Diskette/ }).first().click(); await w(800); });
await zeigen("disk-validieren", async () => { await projekt("diskproj", "beispiel.d64"); await w(600); await page.getByRole("button", { name: /prüf/i }).first().click().catch(() => {}); await w(800); });
await zeigen("disk-editor-uebernahme", async () => { await projekt("diskproj", "beispiel.d64"); await w(600); await page.locator("input[type=checkbox]").first().check().catch(() => {}); await w(300); await page.getByRole("button", { name: /Ins Projekt übernehmen/ }).first().click().catch(() => {}); await w(1200); });
await zeigen("basic-marken", async () => { await projekt("basic", "sterne.bas"); await V(() => { window.__vc.ed.dock = null; window.__vc.ui.panel = false; }); await w(500); });
await zeigen("basic-langenamen", async () => { await projekt("langnamen", "punkte.bas"); await w(500); });
await zeigen("asm-makro", async () => { await projekt("troxx", "hud.asm"); await V(() => { window.__vc.ed.dock = null; }); const l = page.locator(".cm-line", { hasText: /\.macro/ }).first(); await l.scrollIntoViewIfNeeded(); await w(500); });
await zeigen("system-vergleich-c64", async () => { await code("border", "border.asm"); });
await zeigen("system-vergleich-atari", async () => { await code("acolor", "colorbars.asm"); });
await zeigen("system-vergleich-mega65", async () => { await code("hello65", "hello.asm"); });
// --- Dialoge, Ziehen, Projekt ---
const dialogKarte = async (id, von, bis) => { const b = await page.locator("dialog[open]").first().boundingBox(); const x0 = b.x + b.width * von, x1 = b.x + b.width * bis; await page.screenshot({ path: join(out, id + ".png"), clip: { x: x0, y: b.y, width: x1 - x0, height: b.height } }); return { fertig: true }; };
await zeigen("wizard-system", async () => { await nav("projekt_neu"); await w(900); return dialogKarte("wizard-system", 0, 0.232); });
await page.keyboard.press("Escape"); await w(200);
await zeigen("wizard-sprache", async () => { await nav("projekt_neu"); await w(900); return dialogKarte("wizard-sprache", 0.236, 0.562); });
await page.keyboard.press("Escape"); await w(200);
await zeigen("wizard-details", async () => { await nav("projekt_neu"); await w(900); return dialogKarte("wizard-details", 0.565, 1); });
await page.keyboard.press("Escape"); await w(200);
await zeigen("drag-drop-rueckfrage", async () => { await code("jumprun", "jumprun.asm"); await V((a) => { void window.__vc.handleDrop(a); }, [P("gfx/graphics/font.vcchar"), P("sprites/graphics/hero.vcsprite")]); await w(1200); });
await zeigen("bauen-knopf", async () => { await code("jumprun", "jumprun.asm"); await page.screenshot({ path: join(out, "bauen-knopf.png"), clip: { x: 1235, y: 0, width: 200, height: 56 } }); return { fertig: true }; });
await zeigen("editor-zweiter-bereich", async () => { await code("jumprun", "lib/music.inc"); await V(async () => { await window.__vc.runCommand("teilen"); }); await w(700); await V(async () => { await window.__vc.openFile("jumprun.asm"); }); await w(900); });
await zeigen("basic-karte-nummerierung", async () => { await projekt("basic", "sterne.bas"); await nav("nav:projekt:quellen"); await w(900); return { crop: page.locator("[data-testid=basic-card]").first().locator("xpath=..") }; });
await zeigen("basic-karte-pruefung", async () => { await nav("nav:projekt:quellen"); await w(700); return { crop: page.locator("[data-pj=quellen]").first() }; });
await zeigen("turbo-karte-diskette", async () => { await projekt("turbo", "balken.tur"); await nav("nav:projekt:diskette"); await w(1200); });
await zeigen("turbo-diskette-assistent", async () => { await V(async () => { await window.__vc.runCommand("turbo_bereitstellen"); }); await w(1000); });
await zeigen("projekt-neu-ergebnis", async () => { await nav("projekt_neu"); await w(900); await page.getByLabel(/Projektname/).fill("hallo"); await page.getByLabel(/Speicherort/).fill(P("")).catch(() => {}); await page.getByRole("button", { name: /Projekt erstellen/ }).click(); await w(2500); await V(async () => { await window.__vc.runCommand("nav:code"); await window.__vc.openFile("src/c64/main.asm"); window.__vc.ui.panel = false; window.__vc.ed.dock = "output"; }); await w(1200); });
await zeigen("asm-erstes-programm", async () => { await V(async () => { window.__vc.ed.dock = null; await window.__vc.openFile("src/c64/main.asm"); }); await w(900); });
await zeigen("assistent-wiederaufruf", async () => { await einst("allgemein"); await page.locator("[data-es=wiz-schnell]").click(); await page.waitForSelector("[data-wz=assistent]", { timeout: 20000 }); await page.locator("[data-wz=weg-schnell]").click().catch(() => {}); await w(400); for (let i = 0; i < 9 && !(await page.locator("[data-wz=thema-summary]").isVisible().catch(() => false)); i++) { await page.locator("[data-wz=weiter]").click().catch(() => {}); await w(500); } await w(500); });
await zeigen("basic-medium-include", async () => { await projekt("sprites", "sterne.bas"); await V(async () => { await window.__vc.openFile("graphics/hero.vcsprite"); }); await w(800); await page.getByRole("button", { name: new RegExp(T("export-titel")) }).first().click(); await w(900); await page.locator("dialog[open]").getByText("BASIC DATA", { exact: true }).first().click(); await w(1000); });
}
writeFileSync(join(out, "STAND.md"), "# Handbuchbilder aus der neuen Oberfläche (Stand dieses Laufs, Sprache " + SPRACHE + ")\n\n" + ergebnis.map(([i, e]) => `- ${i}: ${e}`).join("\n") + "\n");
console.log(ergebnis.filter((x) => x[1] === "ok").length + " von " + ergebnis.length + " Bildern"); await browser.close();
