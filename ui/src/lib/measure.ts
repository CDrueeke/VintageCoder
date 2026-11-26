// SPDX-License-Identifier: GPL-3.0-or-later
// Messungen für Prüfpunkt G1 (Konzept Abschnitt 9, E0): Rundlauf eines Kernbefehls und Weg vom Zeigerereignis bis zum gezeichneten Bild.
// Die Zeigerereignisse sind synthetisch (ohne Eingabegerät): sie messen Verarbeitung, Rust-Rundlauf und Zeichnen, nicht die Latenz der Maus selbst.
import { call, callBinary, transportKind } from "./api";

export interface Stat { id: string; n: number; median: number; max: number; min: number; p95: number; mean: number }
export interface MeasureResult { env: Record<string, string | number>; stats: Stat[] }

/** `total`: Gesamtzeit der Schleife; der Mittelwert daraus ist genauer als die Einzelzeiten (WebKit rundet `performance.now()` auf 1 ms) */
export function stats(id: string, ms: number[], total?: number): Stat {
  const s = [...ms].sort((a, b) => a - b);
  const at = (p: number) => s[Math.min(s.length - 1, Math.floor(p * s.length))] ?? 0;
  return { id, n: s.length, median: at(0.5), max: s[s.length - 1] ?? 0, min: s[0] ?? 0, p95: at(0.95), mean: (total ?? s.reduce((a, b) => a + b, 0)) / Math.max(1, s.length) };
}

const nextFrame = () => new Promise<void>((r) => requestAnimationFrame(() => r()));
export const W = 24, H = 21, CELL = 12;

function paint(ctx: CanvasRenderingContext2D, x: number, y: number, c: number) {
  ctx.fillStyle = ["#0000", "#7cc4ff", "#ff8a65", "#9ccc65"][c] ?? "#fff";
  ctx.clearRect(x * CELL, y * CELL, CELL, CELL);
  if (c) ctx.fillRect(x * CELL, y * CELL, CELL, CELL);
}

/** Ein Strichpunkt: lokaler Vorschaustrich, bei `bundled` zusätzlich Vorgang an Rust und Zeichnen der Bestätigung (Delta) */
export async function stroke(ctx: CanvasRenderingContext2D, x: number, y: number, bundled: boolean): Promise<void> {
  paint(ctx, x, y, 1);
  if (bundled) { const d = await call("raster_apply", { cells: [[x, y, 1 + ((x + y) % 3)]] }); for (const [cx, cy, c] of d.cells) paint(ctx, cx, cy, c); }
}

/** Zeichenfläche mit lokalem Vorschaustrich; `bundled` schickt den Strich zusätzlich an Rust und zeichnet die Bestätigung (Delta) */
export function attachStroke(canvas: HTMLCanvasElement, bundled: () => boolean): () => void {
  const ctx = canvas.getContext("2d")!;
  const onMove = async (e: PointerEvent) => {
    const r = canvas.getBoundingClientRect();
    const x = Math.floor((e.clientX - r.left) / CELL), y = Math.floor((e.clientY - r.top) / CELL);
    if (x < 0 || y < 0 || x >= W || y >= H) return;
    await stroke(ctx, x, y, bundled());
  };
  canvas.addEventListener("pointermove", onMove);
  return () => canvas.removeEventListener("pointermove", onMove);
}

/** Rohe Bildintervalle (requestAnimationFrame) ohne jede Last: zeigt die Bildrate, die das Fenster tatsächlich bekommt (Drosselung, Takt) */
export async function frameProbe(n = 120): Promise<Record<string, string | number>> {
  const ts: number[] = [];
  await new Promise<void>((res) => { const f = (t: number) => { ts.push(t); if (ts.length > n) res(); else requestAnimationFrame(f); }; requestAnimationFrame(f); });
  const d = ts.slice(1).map((v, i) => v - (ts[i] ?? 0)).sort((a, b) => a - b);
  const total = (ts[ts.length - 1] ?? 0) - (ts[0] ?? 0);
  const r = (v: number | undefined) => +(v ?? 0).toFixed(2);
  return { raf_median_ms: r(d[d.length >> 1]), raf_min_ms: r(d[0]), raf_max_ms: r(d[d.length - 1]), raf_mean_ms: r(total / Math.max(1, d.length)), raf_hz: +(1000 * d.length / Math.max(1, total)).toFixed(1), visibility: document.visibilityState, hasFocus: String(document.hasFocus()) };
}

/** Prüft die Inhaltsrichtlinie des echten Fensters: werden zur Laufzeit eingefügte <style>-Elemente (wie bei CodeMirror) angewendet? */
export function styleProbe(): Record<string, string | number> {
  const s = document.createElement("style"); s.textContent = ".vc-probe{width:123px;position:absolute}"; document.head.append(s);
  const d = document.createElement("div"); d.className = "vc-probe"; document.body.append(d);
  const w = getComputedStyle(d).width; d.remove(); s.remove();
  return { style_inline_ok: w === "123px" ? "ok" : "blocked", style_inline_width: w };
}

export async function run(canvas: HTMLCanvasElement, setBundled: (b: boolean) => void, progress: (s: string) => void): Promise<MeasureResult> {
  const out: Stat[] = [];
  const kind = await transportKind();
  const info = await call("app_info").catch(() => null);

  progress("ping");
  const ping: number[] = [];
  const tp = performance.now();
  for (let i = 0; i < 1000; i++) { const t0 = performance.now(); await call("ping", { n: i, s: "x" }); ping.push(performance.now() - t0); }
  out.push(stats("measure-ping-small", ping, performance.now() - tp));

  progress("raster");
  await call("raster_reset");
  const rd: number[] = [];
  const tr = performance.now();
  for (let i = 0; i < 1000; i++) { const t0 = performance.now(); await call("raster_apply", { cells: [[i % W, (i >> 2) % H, 1 + (i % 3)], [(i + 1) % W, (i >> 2) % H, 2], [(i + 2) % W, (i >> 2) % H, 3]] }); rd.push(performance.now() - t0); }
  out.push(stats("measure-raster-delta", rd, performance.now() - tr));

  if (kind === "tauri") {
    progress("binary");
    const bin: number[] = [];
    const tb = performance.now();
    for (let i = 0; i < 200; i++) { const t0 = performance.now(); await callBinary(65536); bin.push(performance.now() - t0); }
    out.push(stats("measure-binary", bin, performance.now() - tb));
  }

  const ctx = canvas.getContext("2d")!;
  // Ohne Wartezeit auf das nächste Bild: reine Verarbeitung je Strichpunkt (lokaler Strich; bei „gebündelt“ mit Rundlauf zu Rust und Zeichnen der Bestätigung)
  for (const bundled of [false, true]) {
    await call("raster_reset");
    const v: number[] = []; const t1 = performance.now();
    for (let i = 0; i < 1000; i++) { const t0 = performance.now(); await stroke(ctx, (i * 5) % W, (i * 7) % H, bundled); v.push(performance.now() - t0); }
    out.push(stats(bundled ? "measure-pointer-bundled-sync" : "measure-pointer-local-sync", v, performance.now() - t1));
  }
  for (const bundled of [false, true]) {
    progress(bundled ? "bundled" : "local");
    setBundled(bundled);
    await call("raster_reset");
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    const times: number[] = [];
    const r = canvas.getBoundingClientRect();
    for (let i = 0; i < 200; i++) {
      const x = (i * 5) % W, y = (i * 7) % H;
      const t0 = performance.now();
      const done = new Promise<void>((res) => {
        // Bild abwarten: bei „gebündelt“ nach der Antwort von Rust, sonst nach dem Ereignis
        const check = async () => { await nextFrame(); res(); };
        if (bundled) setTimeout(check, 0); else check();
      });
      canvas.dispatchEvent(new PointerEvent("pointermove", { clientX: r.left + x * CELL + 3, clientY: r.top + y * CELL + 3, bubbles: true }));
      if (bundled) { await call("ping", null); }
      await done; await nextFrame();
      times.push(performance.now() - t0);
    }
    out.push(stats(bundled ? "measure-pointer-bundled" : "measure-pointer-local", times));
  }
  setBundled(false);
  const probe = { ...(await frameProbe()), ...styleProbe() };
  const env: Record<string, string | number> = { ...probe, transport: kind, dpr: window.devicePixelRatio, ua: navigator.userAgent, ...(info ? { os: info.os, arch: info.arch, pid: info.pid } : {}) };
  return { env, stats: out };
}
