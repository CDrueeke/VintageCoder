// SPDX-License-Identifier: GPL-3.0-or-later
import { describe, expect, it } from "vitest";
import { Translator } from "./i18n";
import { pseudo, guard } from "./pseudo";

const DE = "hello = Hallo\nfiles = { $n ->\n    [one] 1 Datei\n   *[other] { $n } Dateien\n }\nopen = Datei { $name } öffnen\nonlyde = Nur deutsch\n";
const EN = "hello = Hello\nfiles = { $n ->\n    [one] 1 file\n   *[other] { $n } files\n }\nopen = Open file { $name }\n";

describe("Translator", () => {
  it("formatiert Deutsch mit Mehrzahl und Parametern", () => {
    const t = new Translator("de", DE, null);
    expect(t.t("hello")).toBe("Hallo");
    expect(t.t("files", { n: 1 })).toBe("1 Datei");
    expect(t.t("files", { n: 3 })).toBe("3 Dateien");
    expect(t.t("open", { name: "a.asm" })).toBe("Datei a.asm öffnen");
  });
  it("Englisch mit Rückfall auf Deutsch und sichtbarem Hinweis bei fehlendem Schlüssel", () => {
    const t = new Translator("en", DE, EN);
    expect(t.t("files", { n: 2 })).toBe("2 files");
    expect(t.t("onlyde")).toBe("Nur deutsch");
    expect(t.t("gibtsnicht")).toBe("⟦gibtsnicht⟧");
    expect([...t.missing]).toEqual(["gibtsnicht"]);
  });
  it("Pseudo-Sprache verfremdet und verlängert, Parameter bleiben unverändert", () => {
    const t = new Translator("xx", DE, null);
    const s = t.t("open", { name: "a.asm" });
    expect(s.startsWith("[") && s.endsWith("]")).toBe(true);
    expect(s).toContain("a.asm");
    expect(s).not.toContain("öffnen");
    expect(s.length).toBeGreaterThan("Datei a.asm öffnen".length);
  });
});

describe("pseudo", () => {
  it("entspricht der Rust-Fassung (Beispiel)", () => {
    expect(pseudo("Datei")).toBe("[Ðáţéí~~]");
    expect(pseudo(`x${guard("ä")}`)).toBe("[ẋä~]");
  });
});
