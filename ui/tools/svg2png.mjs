// SPDX-License-Identifier: GPL-3.0-or-later
// SVG als PNG ausgeben (WebKit über Playwright; Hintergrund bleibt durchsichtig, wo die SVG keinen setzt).
// Aufruf (im Ordner ui): node tools/svg2png.mjs <Eingabe.svg> <Ausgabe.png> [Breite in Pixeln]
import { webkit } from "playwright";
import { readFileSync } from "node:fs";
const [ein, aus, breite] = process.argv.slice(2);
if (!ein || !aus) { console.error("Aufruf: node tools/svg2png.mjs <Eingabe.svg> <Ausgabe.png> [Breite]"); process.exit(2); }
const svg = readFileSync(ein, "utf8"); const m = svg.match(/viewBox="0 0 ([\d.]+) ([\d.]+)"/); const [w0, h0] = [Number(m[1]), Number(m[2])];
const w = Number(breite) || w0, h = Math.round((w * h0) / w0);
const b = await webkit.launch(); const p = await b.newPage({ viewport: { width: w, height: h } });
await p.setContent(`<html><body style="margin:0;background:transparent"><img style="display:block;width:${w}px;height:${h}px" src="data:image/svg+xml;base64,${Buffer.from(svg).toString("base64")}"></body></html>`);
await p.waitForTimeout(150); await p.screenshot({ path: aus, omitBackground: true, clip: { x: 0, y: 0, width: w, height: h } }); await b.close();
console.log(aus, w + "×" + h);
