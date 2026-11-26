import { webkit } from "playwright"; import { join, dirname } from "node:path"; import { fileURLToPath } from "node:url";
const d = process.argv[2]; const b = await webkit.launch(); const page = await (await b.newContext({ viewport: { width: 1456, height: 900 }, deviceScaleFactor: 1.5 })).newPage();
for (const v of ["A", "B", "C"]) for (const t of ["dark", "light", "contrast"]) { await page.goto(`file://${join(d, "hover-entwurf.html")}?v=${v}&t=${t}`); await page.waitForTimeout(250); await page.screenshot({ path: join(d, `hover-${v}-${t}.png`), fullPage: true }); }
await b.close();
