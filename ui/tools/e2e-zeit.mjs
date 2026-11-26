// SPDX-License-Identifier: GPL-3.0-or-later
// Zeitfaktor für Ablauftests: VC_E2E_SLOW=5 verlängert jedes page.waitForTimeout(ms) auf das Fünffache (langsame Rechner, Software-Rendering, Container).
// Wird von e2e-alle.sh mit `node --import ./tools/e2e-zeit.mjs` vorgeladen; ohne die Variable (oder mit 1) ändert sich nichts.
import { webkit } from "playwright";
const F = Number(process.env.VC_E2E_SLOW || 1);
if (F > 1) {
  const starte = webkit.launch.bind(webkit);
  webkit.launch = async (...a) => {
    const b = await starte(...a); const nc = b.newContext.bind(b);
    b.newContext = async (...x) => { const c = await nc(...x); const np = c.newPage.bind(c);
      c.newPage = async (...y) => { const p = await np(...y); const w = p.waitForTimeout.bind(p); p.waitForTimeout = (ms) => w(ms * F); return p; }; return c; };
    return b;
  };
}
