# Memory Map

Die **Speicherkarte** (Hauptbereich **Daten ▸ Speicher**) zeigt die Belegung des Adressraums nach dem letzten Bau.

## Karte
Die Karte zeigt den Adressraum als 16×16 Seiten zu je 256 Byte; links steht die Anfangsadresse jeder Reihe. Jede Seite ist nach der Art des Speichers und ihrer Aufgabe gefärbt (zum Beispiel Zeropage, Stapel, Bildschirm, Sprites, Zeichensatz, Musik, E/A, ROM, BASIC, frei), so wie es die gewählte **Bankumschaltung** des Ziels festlegt. Darüber liegt das Programm aus dem **letzten Bau**: Segmente, Bänke und Symbole stammen aus der `.vcdbg` neben dem gebauten Programm; ein Balken in der Seite zeigt, wie viel davon belegt ist. Ohne Bau steht dort „Noch nichts gebaut. Die Karte entsteht aus dem letzten Bau. Erst bauen (F7).“

- **Filter:** **Alle**, **Programm**, **RAM**, **ROM** oder **E/A** blenden den Rest aus (die übrigen Seiten werden blass).
- **Seite wählen:** Klick oder Pfeiltasten. Rechts zeigt **Gewählte Seite** Adresse, Art des Speichers, wie viele Byte das Programm dort belegt, die Symbole auf der Seite und die Quellzeilen (ein Klick springt in den Editor).
- **Bereiche** (links): je Art ein Eintrag mit Farbe und Anfangsadresse; ein Klick wählt die erste Seite dieser Art. Darunter stehen die vom Programm belegten Byte und das freie RAM.
- **Prüfungen** (rechts): Hinweise mit Zeichen für die Schwere (✓, ·, ⚠, ✗), zum Beispiel die Zahl der Überlappungen, ob der Stapel frei ist und wie viel der Zeropage das Programm belegt.

### Bankumschaltung
Für C64, C128 und Atari 800 XL/XE liefert das Ziel (`data/targets/*.target.json`, Abschnitt `banking`) Presets für das Register, das den Speicher umschaltet:

| System | Register | Bits |
|---|---|---|
| C64 | Prozessorport `$0001` | LORAM, HIRAM, CHAREN |
| C128 | MMU-Konfigurationsregister `$FF00` | IO, ROM_LOW, ROM_MID0/1, ROM_HIGH0/1, BANK |
| Atari 800 XL/XE | PORTB `$D301` | OSROM, BASIC, BANK0/1, CPUBANK, ANTICBANK, SELFTEST |

Rechts unten steht das Register mit seiner Adresse und die Liste der Presets (zum Beispiel „Standard (BASIC, E/A, KERNAL)“ beim C64 mit `$37` oder „Alles RAM“ mit `$34`); ein Klick wählt die Sicht. Ein Hinweis zum Preset steht darunter. Ein Preset kann mit dem Hinweis „nur 130XE“ auf ein Modell beschränkt sein. Hat ein Ziel keine Presets (VC-20, C16/Plus/4, MEGA65), zeigt die Karte nur die Segmente des Programms.
