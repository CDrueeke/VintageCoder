# Befehle ohne Funktion und offene Punkte der Oberfläche 0.3.0 (Stand 2026-11-25, `claude/ui-e5`)

Grundlage: `vc_commands::BEFEHLE` (155 Einträge) gegen die Zweige in `ui/src/lib/commands.ts` (programmatisch; der Test `jeder_sichtbare_befehl_hat_eine_funktion` in `crates/vc-shell/src/cmds.rs` hält es fest: Jeder Befehl, den Menü oder Befehlspalette zeigen, hat eine Funktion). Markdown-Befehle (`md_*`) laufen über `ui/src/lib/markdown.svelte.ts`, `lang:`/`nav:`/`file:`/`recent:`/`ext:` über Namensräume.

## Befehle, die in 0.3.0 neu eine Funktion haben

| Befehl | Menü | Wirkung | Prüfung |
|---|---|---|---|
| `zahlenwerkzeuge` | Bearbeiten ▸ Zahlen und BASIC | Dialog: Zahl oder Ausdruck (`$D020`, `$D000+$20`, `1<<5`, `-1`), alle Schreibweisen, Bit-Editor 8/16 Bit, Kopieren, Einsetzen; Kern `vc_ide_model::zahlen` (Befehl `zahlen_werte`) | Rust-Test, `e2e-befehle` |
| `als_include` | Baum (Kontextmenü) | schreibt `.include "<Pfad relativ zur Quelldatei>"` in den aktiven Assembler-Reiter; nichts doppelt, keine Projektdateien/Diskettenabbilder | `e2e-befehle` |
| `kopieren`, `ausschneiden`, `einfuegen` | Bearbeiten | außerhalb des Sprite-Editors im Code-Editor und in Eingabefeldern (Einfügen über die Palette, wo das Fenster es erlaubt, sonst Hinweis auf Cmd/Strg+V; die Tastenkürzel gingen schon) | `e2e-befehle` |
| `projekt_ordner_oeffnen` | Datei | Ordnerwahl wie „Projekt öffnen“ | Rust-Test (Zweig vorhanden) |

## Befehle ohne Funktion: aus Menü und Befehlspalette ausgeblendet

Die Befehlstabelle bleibt vollständig (Kennungen, Katalogtexte); `vc_commands::NOCH_NICHT_GEBAUT` blendet sie aus. Ein Befehl kommt aus der Liste, sobald `commands.ts` ihn ausführt (der Test meldet es).

| Befehl | Menüeintrag | Vorgesehen für 0.3.0? | Aufwand | Anmerkung |
|---|---|---|---|---|
| `basic_nach_asm` | Datei ▸ BASIC in Assembler umwandeln | nein (Handbuch 07 nennt Bauoption `convert` und `vcc basic2asm`) | mittel | Kern `vc_ide::b2a` und `vcc` vorhanden; fehlt: Shell-Befehl, Dialog (Ausgabedatei, Ziel, Optionen), Test |
| `kodierung` | Bearbeiten ▸ Art und Kodierung | nein | mittel | der Kern liest nur UTF-8 (`datei-kein-utf8`); ISO-8859-1/BOM braucht Kern, Reiterzustand, Test |
| `hex_vorschlaege` | Bearbeiten ▸ Zahlen und BASIC | nein (Kommandozeile `vcc basic --suggest-hex`) | mittel | Kern `vc_ide::hexhint` vorhanden; fehlt: Shell-Befehl, Dialog mit Zeilenauswahl, ein Rückgängig-Schritt |
| `std_zeichensatz` | Datei ▸ Standardzeichensatz hinzufügen … | offen (Entscheidung) | mittel | Weg zu den eigenen CC0-Sätzen; Kern `vc_media::project::std_charset_list` vorhanden |
| `zeichensatz_rom` | Datei ▸ Zeichensatz aus ROM importieren … | nein | mittel | Dialog und Kern-Anbindung fehlen |
| `zeichensatz_png` | Datei ▸ Zeichensatz aus PNG-Raster importieren … | nein | mittel | hängt am Handbuchbild `zeichensatz-png-import` |
| `studio_vergroessern`, `studio_verkleinern`, `studio_einpassen` | Ansicht ▸ Editor: Fläche … | nein | mittel | Knöpfe und Tasten (+, −, 0) gibt es in allen Editoren; Menüanbindung braucht je Editor eine Zoom-Schnittstelle |
| `studio_raster`, `studio_zwiebel` | Ansicht ▸ Editor: Raster / Onion Skin | nein | mittel | Taste G und O und Knöpfe vorhanden |

Nicht über `commands.ts`, aber nur beschrieben: Ziehen aus dem Projektbaum in den Quelltext (`include-ziehen`, `include-eingefuegt`, `include-abgelehnt`) ist nicht gebaut (Handbuch 05 sagt es); „Als Include einfügen“ im Export-Dialog ist gebaut (Bilder `basic-medium-include`). Das Untermenü **Projekt ▸ Erweiterungen** erscheint nur mit dem Entwicklerschalter `dev-extensions` (`VC_EXTENSIONS_DEV`, nie im Auslieferungsbau); `ext:`-Befehle zeigen „Werkzeuge folgen“.

## Kurzfassung: offene und teilweise Punkte aus der Paritätsprüfung

Die Einzelzeilen stehen in [paritaetsmatrix-0.3.0.md](paritaetsmatrix-0.3.0.md); hier die Entscheidungsliste daraus (Buchstaben wie dort):

**B. Zeichensatz-Import** (alle drei Dialoge fehlen, der Kern `std_charset_list` liegt fertig): Standard-Zeichensatz-Dialog (Weg zu den freigegebenen eigenen CC0-Sätzen, mittel) in 0.3.0 bauen; ROM und PNG-Raster bauen oder nach 0.3.1 schieben (hängt am Handbuchbild `zeichensatz-png-import`).

**C. Klein, in 0.3.0 nachziehbar:** Grundtext in der Palette für ausgegraute Befehle; Schalter für die Absturzsicherung; Zeichenpanel; Zwischenablage im Code-Editor über die Palette; ROM-Import „Alle wählen/abwählen“; ROM-Seite „Portabler Modus“; Dateidialog an den Emulatorpfaden; Projekt ▸ Ziele: Standard-Emulator und Speicherausbau; Projekt ▸ Quellen: Reihenfolge der Gruppen; Info: Tabelle der eigenen Zeichensätze (Katalogschlüssel da, nirgends benutzt); Diskette: Programmauswahl (`disk.files`) und Ausgabedatei; Zeichensatz: Anzeige „ROM fehlt“ (Kerntext `view.status` wird nirgends gezeigt).

**D. Entscheidung des Inhabers:** BASIC nach Assembler als Bauoption (Kern kann es, 0.3.0 mittel oder 0.3.1); Menüleisten-Schalter (nur Hinweistext; entfällt, da die Menüleiste nativ ist?).

**E. Bewusst nach 0.3.1 oder entfällt (mit Begründung in den Detailtabellen):** Datei-Symbole und Gruppen im Baum, exportierte Dateien markieren, Hinweis „Definition außerhalb“, Diagnosen ein/aus, Setup-Zeile der Tests, Takt und Norm in der Statusleiste, Export beim Bauen, Hardware-Probe, Startseite „neue Datei“, Import-Dialog (Ablegen kopiert und benennt bei Namensgleichheit selbst um), Projektbild in den Projekteinstellungen, Takt-Anzeige, Editorwünsche (Einrückungslinien, sichtbare Leerzeichen, Signaturhilfe, Snippets, Farbvorschau), MEGA65-Diskettenübertragung, VIC-IV-Sprites, grafisches Chip-Fenster im Debug; entfallen: Menüleiste im Fenster ein/aus, „ausgegraut mit Grund“ im nativen Menü (technisch nicht möglich).

**F. Unklar, nur am Gerät klärbar:** Bildschirmleser (VoiceOver-Handtest zeigt Namen, Rollen, Zustände in 14 Bereichen ✔); Aktionen je Editor in der Palette, Kontextmenü Zeichenfläche, Kontextmenü Aktionen, Kopfzeile in Breitenstufen, Tests parallel zur Debug-Sitzung, Kachelkarte umwandeln, Rückfrage beim Debug-Beenden, Assistent „Erstes Projekt“.
