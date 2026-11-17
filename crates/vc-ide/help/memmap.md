# Memory Map

Das Panel **Karte** (rechtes Panel, Symbol ▦) zeigt die Belegung des Adressraums.

## Matrix
Die Matrix zeigt den Adressraum als 16×16 Seiten zu je 256 Byte. Jede Seite ist nach der Art des Speichers gefärbt (RAM, ROM, BASIC-ROM, System-ROM, Zeichen-ROM, E/A, Modul) und nach ihrer Aufgabe (Stack, Bildschirm, Sprites, Zeichensatz …), so wie es das gewählte **Bank-Preset** des Ziels festlegt. Darüber liegt das Programm aus der **letzten Übersetzung**: Segmente, Bänke und Symbole stammen aus der `.vcdbg` neben dem gebauten Programm; ein Balken in der Seite zeigt, wie viel davon belegt ist. Ohne Bau steht dort „Noch nichts gebaut: Nach dem Bauen zeigt die Karte, wo dein Programm im Speicher liegt.“

- **Filter:** Alle, Programm, RAM, ROM oder E/A blenden den Rest aus.
- **Seite wählen:** Ein Klick auf eine Seite zeigt darunter Adresse, Art des Speichers, wie viele Byte das Programm dort belegt, die Symbole auf der Seite und die Quellzeilen.
- **Prüfung:** zum Beispiel die Zahl der Überlappungen, ob der Stack frei ist und wie viel der Zeropage das Programm belegt.
- Eine Legende unter der Matrix erklärt die Farben.

### Bank-Presets
Für C64, C128 und Atari 800 XL/XE liefert das Ziel (`data/targets/*.target.json`, Abschnitt `banking`) Presets für das Register, das den Speicher umschaltet:

| System | Register | Bits |
|---|---|---|
| C64 | Prozessorport `$0001` | LORAM, HIRAM, CHAREN |
| C128 | MMU-Konfigurationsregister `$FF00` | IO, ROM_LOW, ROM_MID0/1, ROM_HIGH0/1, BANK |
| Atari 800 XL/XE | PORTB `$D301` | OSROM, BASIC, BANK0/1, CPUBANK, ANTICBANK, SELFTEST |

Unter der Matrix steht das Register mit seiner Adresse und die Liste der Presets (zum Beispiel „Standard (BASIC, E/A, KERNAL)“ beim C64 mit `$37` oder „Alles RAM“ mit `$34`); ein Klick wählt die Sicht. Ein Hinweis zum Preset steht darunter. Ein Preset kann mit dem Hinweis „nur 130XE“ auf ein Modell beschränkt sein. Hat ein Ziel keine Presets (VC-20, C16/Plus/4, MEGA65), zeigt die Matrix nur die Segmente des Programms.
