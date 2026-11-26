// SPDX-License-Identifier: GPL-3.0-or-later
import { describe, expect, it } from "vitest";
import { zerlegeHover } from "./md";

describe("zerlegeHover", () => {
  it("Befehl: Titel, Kurzbeschreibung, Flags, Tabelle, Fußzeile", () => {
    const t = zerlegeHover("**LDA** – Akkumulator laden\n\nFlags: `N Z`\n\n| Modus | Opcode |\n|---|---|\n| ▶ Sofortwert (imm) | $A9 |\n\nDiese Zeile: **Sofortwert (imm)** · Bytes `A9 00` · 2 Zyklen\n");
    expect(t.title).toBe("LDA"); expect(t.subtitle).toBe("Akkumulator laden"); expect(t.flags).toEqual(["N", "Z"]);
    expect(t.body).toContain("| Modus | Opcode |"); expect(t.footer).toMatch(/^Diese Zeile:/);
  });
  it("Direktive: Schreibweise als Titel, Beschreibung als Rumpf", () => {
    const t = zerlegeHover("`#if BEDINGUNG`\n\nNimmt die folgenden Zeilen nur auf, wenn die Bedingung gilt.");
    expect(t.title).toBe("#if BEDINGUNG"); expect(t.body).toMatch(/^Nimmt/); expect(t.subtitle).toBe("");
  });
  it("Konstante: Name als Titel, Rest als Kurzbeschreibung und Wert im Rumpf", () => {
    const t = zerlegeHover("`STERNE` ist per `#define` definiert (Zeile 2): `5`\n\nVor dem Nummerieren wird der Name ersetzt.");
    expect(t.title).toBe("STERNE"); expect(t.subtitle).toContain("ist per #define definiert"); expect(t.body).toMatch(/^Vor dem/);
  });
  it("unbekannte Form: alles im Rumpf, kein Titel", () => {
    const t = zerlegeHover("Irgendein Text ohne Kopf.\n\nNoch ein Absatz.");
    expect(t.title).toBe(""); expect(t.body).toContain("Irgendein Text"); expect(t.body).toContain("Noch ein Absatz");
  });
  it("leer", () => { expect(zerlegeHover("").title).toBe(""); });
});
