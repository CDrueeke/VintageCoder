# Ein Release zusammenstellen

Diese Seite beschreibt den Weg von der Auswahl in `FEATURES.md` bis zu den fertigen Installern. Die Bedienung des Skripts und was abgewählte Teile bewirken, steht auch in der IDE-Hilfe (Seite „Release-Auswahl“, `crates/vc-ide/help/release-auswahl.md`). Entwicklungsstände und Release-Kriterien stehen in `STATUS.md`.

Wer die Versionsnummer, den Tag und die Veröffentlichung festlegt, ist allein der Projektinhaber. Beispiel: Version `0.1.0`, Tag `v0.1.0` (auch für ein Alpha-Release); `Cargo.toml` ändert nur der Projektinhaber. Alles hier endet vor diesem Schritt.

## Überblick

```
FEATURES.md ──┐
              ├─▶ tools/release.py check ─▶ plan ─▶ stage ─▶ Staging-Ordner ─▶ Installer
release/registry.json ─┘                                    (data/release.json,
                                                             release-report.md)
```

- `FEATURES.md`: die Auswahl, einzige Stelle mit Häkchen.
- `release/registry.json`: Abhängigkeiten und Dateien je Kennung (nicht Teil des Pakets).
- Staging: Verzeichnis mit Programmen, `data/`, `examples/`, `data/release.json` und `release-report.md`. Die Installer packen nur noch dieses Verzeichnis ein: Windows `installer/vintagecoder.iss` (Inno Setup), macOS `installer/build-macos.sh`, Linux zunächst ein Archiv `tar.gz`. Die Linux-Pakete `.deb` und `.rpm` sind in Arbeit (Stand: `STATUS.md`).
- Eine Auswahl gilt für alle Betriebssysteme; `--os` ändert nur die Verpackung.

## Schritt für Schritt

1. **Stände prüfen.** Stimmt die Spalte „Stand“ in `FEATURES.md` noch? Gewählte Zeilen müssen mindestens **Beta** sein. Die Stände der festen Bestandteile stehen in `STATUS.md`.
2. **Häkchen setzen** in `FEATURES.md` (`[x]`). Der Kopf der Datei erklärt das Muster.
3. **Prüfen:** `python3 tools/release.py check`. Fehler beheben; fehlt eine Abhängigkeit, nennt die Meldung, was anzukreuzen ist. Mit `--explain KENNUNG` sieht man, wer eine Zeile braucht.
4. **Ansehen:** `python3 tools/release.py plan -v` zeigt Zeilen, Dateizahlen und Größen.
5. **Staging erzeugen**, je Betriebssystem auf dem eigenen Rechner (ein Bau je Betriebssystem):

   ```bash
   python3 tools/release.py stage --os macos   --out build/stage-macos   --build
   python3 tools/release.py stage --os windows --out build/stage-windows --build
   python3 tools/release.py stage --os linux   --out build/stage-linux   --build --archive
   ```

   Für gleiche Prüfsummen bei zwei Läufen `--reproducible`. Mit `--verify-build` baut `stage` jedes ausgelieferte Beispiel mit dem Staging-`vcc` (nur auf dem eigenen Betriebssystem).
6. **Bericht lesen:** `release-report.md` im Staging. Warnungen und „Hinweise für die Release-Notizen“ gehören in die Release-Notiz; `release/notes-template.md` ist die Vorlage dafür.
7. **Stichproben** (siehe unten).
8. **Installer bauen** und **einmal von Hand starten** (Prüfschritt, den kein Skript ersetzt): Installieren, Programm starten, ein Beispiel bauen, Deinstallieren.
9. Übergabe an den Projektinhaber: Version, Tag, Veröffentlichung, Website.

## Checkliste

- [ ] `cargo build --workspace` ohne Warnungen, `cargo test --workspace` grün (Kriterium 1).
- [ ] `python3 tools/release.py check` ohne Fehler und ohne unerklärte Warnungen.
- [ ] `python3 -m unittest discover -s tools -p 'test_*.py'` grün.
- [ ] Jede gewählte Zeile hat mindestens den Stand Beta, eine Hilfeseite oder einen Hilfeabschnitt und einen Eintrag im `CHANGELOG.md` (Kriterien 2 und 4).
- [ ] Für jedes gewählte Zielsystem gibt es ein Beispiel oder einen Test (Kriterium 3; `check` warnt sonst).
- [ ] Staging für jedes Betriebssystem erzeugt, Verbotsscan ohne Treffer (keine ROMs, keine Emulator-Programme).
- [ ] `release-report.md` gelesen; Hinweise in die Release-Notiz übernommen.
- [ ] `python3 tools/ci_staging.py` grün (Stichproben unten).
- [ ] IDE mit der Auswahl gestartet (`VC_RELEASE=<Staging>/data/release.json`), nichts Abgewähltes sichtbar.
- [ ] Installer je Betriebssystem einmal von Hand gestartet.
- [ ] `CHANGELOG.md`: Abschnitt „Unveröffentlicht“ vollständig; die Versionsnummer (z. B. `0.1.0`) und den Tag (`v0.1.0`) setzt der Projektinhaber.
- [ ] Außerhalb des Pakets: Website `vintagecoder.drueeke.de` und Handbuch aktualisiert.

## Stichproben mit `tools/ci_staging.py`

```bash
cargo build --workspace
python3 tools/ci_staging.py                     # alle Testauswahlen
python3 tools/ci_staging.py --only nur-cli-atari -v
python3 tools/ci_staging.py --bin-dir target/release
```

Je Datei unter `tools/testdata/*.md` (Testauswahlen, **keine** Voreinstellungen für Releases) wird ein Staging für das eigene Betriebssystem erzeugt und mit dem Staging-`vcc` geprüft:

| Testauswahl | Zweck |
|---|---|
| `voll.md` | alles an |
| `nur-cli-atari.md` | nur Kommandozeile, nur Atari 8 Bit |
| `nur-cli-c64.md` | nur Kommandozeile, nur C64 |
| `nur-ide-c64-ohne-emulator.md` | IDE mit C64 ohne jede Anbindung |

Geprüft wird: `data/release.json` nennt genau die gewählten Anbindungen und Zielsysteme; `vcc emulator list` zeigt genau die gewählten Emulatoren, abgewählte melden „nicht im Paket enthalten“ (Rückgabewert 1); `vcc new` und `vcc build` gelingen für jedes gewählte Zielsystem, ein abgewähltes wird mit Rückgabewert 2 abgelehnt; `vcc help` läuft. Es braucht kein Fenster, keinen Emulator und kein Netz. Dieselben Schritte laufen im CI.

Von Hand zusätzlich:

```bash
VC_RELEASE=build/stage-macos/data/release.json cargo run -p vc-app     # IDE mit der Auswahl
VC_RELEASE=build/stage-macos/data/release.json vcc emulator list
python3 tools/release.py stage --os linux --out build/pruefung --features tools/testdata/nur-cli-c64.md --no-bin --force
```

Der IDE-Selbsttest `mindestauswahl_zeigt_nichts_abgewaehltes` (`cargo test -p vc-app`) prüft Menüs, Befehle, Reiter, Hilfe und Anbindungen für drei Mindestauswahlen.

## Grenzen

Abgewähltes ist ausgeblendet, nicht entfernt (`VC_RELEASE=all` oder das Löschen von `data/release.json` bringt es zurück). Geteilte Datendateien nennen auch nicht gewählte Systeme. Emulatoren und ROMs werden nie ausgeliefert. Linux-Pakete `.deb`/`.rpm`: in Arbeit, siehe `STATUS.md`.
