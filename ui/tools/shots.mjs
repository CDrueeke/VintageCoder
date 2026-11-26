// SPDX-License-Identifier: GPL-3.0-or-later
// Bilder und Layoutprüfung der Oberfläche: Playwright (WebKit) gegen die laufende Prüfbrücke des echten Kerns.
// Aufruf: node tools/shots.mjs <Ausgabeordner> <Projektordner (Kopie!)>   (Vite auf :1420 und `vintagecoder` mit VC_TESTBRIDGE=17420 müssen laufen)
import { webkit } from "playwright";
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const [out, project] = process.argv.slice(2);
if (!out || !project) { console.error("Aufruf: node tools/shots.mjs <Ausgabe> <Projektkopie>"); process.exit(2); }
const URL = "http://127.0.0.1:1420/?bridge=17420";
const VIEWS = [
  { id: "01-start", project: false },
  { id: "02-code", mode: "code" },
  { id: "03-sprites", mode: "grafik", sub: "sprites" },
  { id: "04-palette", mode: "grafik", sub: "palette" },
  { id: "05-musik", mode: "musik" },
  { id: "06-disk", mode: "daten", sub: "disk" },
  { id: "07-debug", mode: "debug" },
  { id: "08-bibliothek", extra: "gallery" },
  { id: "09-messung", extra: "measure" },
  { id: "10-befehlspalette", palette: true },
  { id: "11-sprachmenue", menu: "Language|Sprache" },
  { id: "12-menue-farbschema", menu: "Color scheme|Farbschema|Pseudo" },
  { id: "13-editor", mode: "code", file: "hal_c64.asm", end: true },
  { id: "14-suche", mode: "code", file: "hal_c64.asm", cmd: "suchen" },
];
const WIDTHS = [[1440, 900], [1100, 700], [900, 600]];
const browser = await webkit.launch();
const befunde = [];

for (const lang of ["de", "en", "xx"]) {
  for (const theme of ["dark", "light"]) {
    const ctx = await browser.newContext({ viewport: { width: 1440, height: 900 }, colorScheme: theme });
    const page = await ctx.newPage();
    await page.goto(URL);
    await page.waitForFunction(() => window.__vc && window.__vc.ui.ready);
    await page.evaluate(async ([l, th]) => { await window.__vc.setLanguage(l); window.__vc.setTheme(th); }, [lang, theme]);
    const dir = join(out, lang); mkdirSync(dir, { recursive: true });
    for (const v of VIEWS) {
      await page.evaluate(async ([v, project]) => {
        const c = window.__vc; c.ui.extra = null; c.ui.palette = false;
        if (v.project === false) { await c.call("project_close"); c.ui.project = null; }
        else if (!c.ui.project) await c.openProject(project);
        if (v.mode) { c.ui.mode = v.mode; if (v.sub) c.ui.sub[v.mode] = v.sub; }
        if (v.extra) c.ui.extra = v.extra;
        if (v.palette) c.ui.palette = true;
        if (v.file) await c.openFile(v.file);
        if (v.cmd) await c.runCommand(v.cmd);
      }, [v, project]);
      if (v.end) { await page.locator(".cm-content").click(); await page.keyboard.press("Meta+ArrowDown"); }
      if (v.menu) { await page.locator(`header button[aria-haspopup="menu"]`).nth(v.menu.startsWith("Color") ? 1 : 0).click(); }
      await page.waitForTimeout(150);
      await page.screenshot({ path: join(dir, `${v.id}-${theme}.png`) });
      if (v.menu || v.cmd) await page.keyboard.press("Escape");
      // Layoutprüfung: abgeschnittene oder überlaufende Texte in drei Fensterbreiten (nur Dunkel, ein Durchlauf je Sprache)
      if (theme === "dark" && !v.menu && !v.palette) for (const [w, h] of WIDTHS) {
        await page.setViewportSize({ width: w, height: h });
        const bad = await page.evaluate(() => [...document.querySelectorAll("button,.chip,.seg *,.status span,.tb *,h2,p,.kbd")].filter((e) => {
          const s = getComputedStyle(e); if (s.display === "inline" || s.overflow === "auto" || e.classList.contains("sr-only") || e.hasAttribute("data-ellipsis")) return false;   // data-ellipsis: bewusst mit „…“ gekürzt (Projektname, Pfad, Zielname)
          const card = e.closest(".card"); const r = e.getBoundingClientRect(), cr = card ? card.getBoundingClientRect() : null;
          if (cr && r.width > 0 && (r.right > cr.right + 1 || r.left < cr.left - 1)) return true;
          return e.scrollWidth > e.clientWidth + 1 && e.clientWidth > 0;
        }).map((e) => `${e.tagName.toLowerCase()}.${e.className.toString().slice(0, 20)} "${(e.textContent || "").trim().slice(0, 30)}"`));
        for (const b of new Set(bad)) befunde.push(`${lang} ${v.id} ${w}x${h}: Überlauf ${b}`);
        await page.setViewportSize({ width: 1440, height: 900 });
      }
    }
    await ctx.close();
  }
}
writeFileSync(join(out, "layout-befunde.txt"), befunde.length ? befunde.join("\n") + "\n" : "keine Befunde\n");
console.log(`fertig, ${befunde.length} Layout-Befunde`);
await browser.close();
