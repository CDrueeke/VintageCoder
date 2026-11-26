// SPDX-License-Identifier: GPL-3.0-or-later
import { defineConfig } from "vitest/config";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// Mit VC_CSP=release liefert der Entwicklungsserver die Inhaltsrichtlinie des Auslieferungsbaus (aus tauri.conf.json, nur Verbindungen für Prüfbrücke und HMR ergänzt):
// so zeigt der Ablauftest im Browser dasselbe Verhalten für Stile wie das echte Fenster.
function releaseCsp(): Record<string, string> {
  // Node-Dinge über globalThis: das Projekt führt keine Node-Typen
  const g = globalThis as unknown as { process: { env: Record<string, string | undefined>; getBuiltinModule: (n: string) => { readFileSync(p: URL, e: string): string } } };
  if (g.process.env.VC_CSP !== "release") return {};
  const conf = JSON.parse(g.process.getBuiltinModule("node:fs").readFileSync(new URL("../crates/vc-shell/tauri.conf.json", import.meta.url), "utf8"));
  const csp = String(conf.app.security.csp).replace("connect-src 'self'", "connect-src 'self' ws://127.0.0.1:*");
  return { "Content-Security-Policy": csp };
}

export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  server: { port: 1420, strictPort: true, host: "127.0.0.1", headers: releaseCsp() },
  // Alle Anfragen bleiben lokal; im Bau liegen keine Schriften oder Skripte von außen
  build: { target: "safari16", outDir: "dist", emptyOutDir: true, assetsInlineLimit: 0, modulePreload: { polyfill: false } },
  test: { environment: "node", include: ["src/**/*.test.ts"] },
});
