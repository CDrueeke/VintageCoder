// SPDX-License-Identifier: GPL-3.0-or-later
// Ablauftest Sprachdienste für natives BASIC: Dialekt in der Statuszeile, ausgeschlossene Zeilen (#if) grau, Gliederung (Marken, #define), Hover für #define-Namen und Direktiven.
// Aufruf: node tools/e2e-basicinfo.mjs <Projektkopie examples/c64/basic_native> [Bilderordner]
import { webkit } from "playwright";
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
const [proj, shots] = process.argv.slice(2);
if (!proj) { console.error("Aufruf: node tools/e2e-basicinfo.mjs <Projekt> [Bilder]"); process.exit(2); }
if (shots) mkdirSync(shots, { recursive: true });
const browser = await webkit.launch();
const page = await (await browser.newContext({ viewport: { width: 1280, height: 820 } })).newPage();
const fails = []; let n = 0;
const ok = (c, m) => { n++; if (!c) { fails.push(m); console.log("FEHLER:", m); } else console.log("ok:", m); };
page.on("pageerror", (e) => { fails.push("Seitenfehler " + e.message); console.log("Seitenfehler", e.message); });
const snap = async (name) => { if (!shots) return; for (const th of ["light", "dark"]) { await page.evaluate((x) => window.__vc.setTheme(x), th); await page.waitForTimeout(250); await page.screenshot({ path: join(shots, `${name}-${th === "light" ? "hell" : "dunkel"}.png`) }); } await page.evaluate((x) => window.__vc.setTheme(x), "light"); };
await page.goto(`http://127.0.0.1:${process.env.VC_VITE ?? 1420}/?bridge=${process.env.VC_BRIDGE ?? 17420}`); await page.waitForFunction(() => window.__vc && window.__vc.ui.ready);
await page.evaluate(async () => { await window.__vc.setLanguage("de"); }); await page.waitForTimeout(300);
await page.evaluate(async (p) => { await window.__vc.openProject(p); }, proj); await page.waitForTimeout(600);
writeFileSync(join(proj, "system.bas"), '#define STERNE 5\n#if SYSTEM_C128\nprint "nur c128"\n#else\nprint "c64"\n#endif\n@start: for i=1 to STERNE\nprint "*";\nnext i\n#asm "code.asm"\n');
await page.evaluate(async () => { window.__vc.ui.extra = null; const t = await window.__vc.call("tree_reload"); window.__vc.ui.project.tree = t.tree; await window.__vc.openFile("system.bas"); });
await page.waitForSelector(".cm-editor .cm-content"); await page.waitForTimeout(900);
const where = (sub, off, lineHas = "") => page.evaluate(([sub, off, lineHas]) => { const w = document.createTreeWalker(document.querySelector(".cm-content"), NodeFilter.SHOW_TEXT); let n; while ((n = w.nextNode())) { const i = n.textContent.indexOf(sub); if (i >= 0 && (!lineHas || n.parentElement.closest(".cm-line").textContent.includes(lineHas))) { const r = document.createRange(); r.setStart(n, i + off); r.setEnd(n, i + off + 1); const b = r.getBoundingClientRect(); return { x: b.x + b.width / 2, y: b.y + b.height / 2 }; } } return null; }, [sub, off, lineHas]);

// 1 Dialekt in der Statuszeile
ok(/CBM BASIC V2|BASIC V2/i.test(await page.getByTestId("status-dialekt").innerText().catch(() => "")), "Statuszeile nennt den Dialekt: " + await page.getByTestId("status-dialekt").innerText().catch(() => "-"));
// 2 ausgeschlossene Zeile grau (C64: der C128-Zweig, Zeile 3)
await page.waitForFunction(() => document.querySelectorAll(".cm-excluded").length > 0, null, { timeout: 5000 }).catch(() => {});
const ex = await page.locator(".cm-excluded").allInnerTexts();
ok(ex.length === 1 && /nur c128/.test(ex[0]), "Ausgeschlossene Zeile (#if SYSTEM_C128) ist grau: " + JSON.stringify(ex));
// 3 Hover auf #define-Name
let pt = await where("STERNE", 2, "for i=1");
await page.mouse.move(pt.x, pt.y);
await page.waitForSelector(".cm-tooltip .vc-card", { timeout: 4000 }).catch(() => {});
const c1 = await page.locator(".cm-tooltip .vc-card").first().innerText().catch(() => "");
ok(/STERNE/.test(c1) && /#define/.test(c1) && /5/.test(c1), "Hover auf STERNE zeigt die Definition: " + c1.replace(/\n/g, " ").slice(0, 100));
await snap("info-1-hover-define-ausgeschlossen-dialekt");
await page.mouse.move(5, 5); await page.waitForTimeout(400);
// 4 Hover auf Direktive
pt = await where("#", 0, "SYSTEM_C128");
await page.mouse.move(pt.x, pt.y); await page.waitForSelector(".cm-tooltip .vc-card", { timeout: 4000 }).catch(() => {});
const c2 = await page.locator(".cm-tooltip .vc-card").first().innerText().catch(() => "");
ok(/#if/.test(c2) && /grau/.test(c2), "Hover auf #if erklärt die Direktive: " + c2.replace(/\n/g, " ").slice(0, 100));
await page.mouse.move(5, 5); await page.waitForTimeout(400);
pt = await where("#", 0, "code.asm");
await page.mouse.move(pt.x, pt.y); await page.waitForSelector(".cm-tooltip .vc-card", { timeout: 4000 }).catch(() => {});
const c3 = await page.locator(".cm-tooltip .vc-card").first().innerText().catch(() => "");
ok(/#asm/.test(c3) && /Maschinencode/.test(c3), "Hover auf #asm erklärt die Direktive: " + c3.replace(/\n/g, " ").slice(0, 80));
await page.mouse.move(5, 5); await page.waitForTimeout(400);
// 5 Gliederung
await page.evaluate(() => { void window.__vc.runCommand("schublade"); }); await page.waitForTimeout(300);
await page.locator(".dock [data-tab=outline]").click(); await page.waitForSelector(".dock .o .row", { timeout: 4000 }).catch(() => {});
const ol = (await page.locator(".dock .o .row").allInnerTexts()).join(" | ").replace(/\n/g, " ");
ok(/@start/.test(ol) && /STERNE/.test(ol), "Gliederung zeigt Marke und #define: " + ol);
await snap("info-2-gliederung");
// 6 ändert man das Zielsystem im Text (#dialect), ändert sich der Dialekt-Name; Englisch
await page.evaluate(async () => { await window.__vc.setLanguage("en"); }); await page.waitForTimeout(400);
pt = await where("STERNE", 2, "for i=1");
await page.mouse.move(pt.x, pt.y); await page.waitForSelector(".cm-tooltip .vc-card", { timeout: 4000 }).catch(() => {});
const c4 = await page.locator(".cm-tooltip .vc-card").first().innerText().catch(() => "");
ok(/defined with #define/.test(c4), "Hover auf Englisch: " + c4.replace(/\n/g, " ").slice(0, 90));
await snap("info-3-hover-englisch");
// 7 Apostroph: in Commodore-BASIC kein Kommentar (B167), also keine Kommentarfärbung
await page.evaluate(async () => { await window.__vc.setLanguage("de"); });
writeFileSync(join(proj, "apostroph.bas"), '10 print "a" \' kein kommentar in commodore basic\n20 rem das ist ein kommentar\n');
await page.evaluate(async () => { window.__vc.ui.extra = null; const t = await window.__vc.call("tree_reload"); window.__vc.ui.project.tree = t.tree; await window.__vc.openFile("apostroph.bas"); });
await page.waitForSelector(".cm-editor .cm-content"); await page.waitForTimeout(900);
const kom = await page.locator(".cm-line", { hasText: "kein kommentar in commodore" }).locator(".tok-comment").allInnerTexts();
ok(kom.length === 0, "Apostroph färbt in Commodore-BASIC keinen Kommentar: " + JSON.stringify(kom));
await snap("info-4-apostroph");
console.log(`${n} Prüfungen, ${fails.length} Fehler`);
await browser.close();
process.exit(fails.length ? 1 : 0);
