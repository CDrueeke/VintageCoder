# Memory Map

Der Tab **Memory Map** (Inspektor, Arbeitsplatz Code) hat die Reiter **Matrix** und **Liste**.

## Matrix
Die Matrix zeigt den Adressraum als 16×16 Seiten zu je 256 Byte. Jede Seite ist nach der Art des Speichers gefärbt (RAM, ROM, BASIC-ROM, System-ROM, Zeichen-ROM, E/A, Modul) und nach ihrer Aufgabe (Stack, Bildschirm, Sprites, Zeichensatz …), so wie es das gewählte **Bank-Preset** des Ziels festlegt. Darüber liegt das Programm aus der **letzten Übersetzung**: Segmente, Bänke und Symbole stammen aus der `.vcdbg` neben dem gebauten Programm. Ohne Bau steht dort „Bauen, um die Belegung zu sehen“.

- **Ansicht:** „256 Seiten“, „4-KB-Block“ oder „Byte“ (die gewählte Seite byteweise).
- **Filter:** Alle, Programm, RAM, ROM oder E/A blenden den Rest aus.
- **Segment-Inspektor:** Klick auf eine Seite zeigt Adresse, Art des Speichers, das belegende Segment mit Quelle und die Symbole auf der Seite. Berühren mehrere Segmente eine Seite, steht dort „gemischt“.
- **Prüfungen:** zum Beispiel die Zahl der Überlappungen, ob der Stack frei ist und wie viel der Zeropage das Programm belegt.

### Bank-Presets
Für C64, C128 und Atari 800 XL/XE liefert das Ziel (`data/targets/*.target.json`, Abschnitt `banking`) Presets für das Register, das den Speicher umschaltet:

| System | Register | Bits |
|---|---|---|
| C64 | Prozessorport `$0001` | LORAM, HIRAM, CHAREN |
| C128 | MMU-Konfigurationsregister `$FF00` | IO, ROM_LOW, ROM_MID0/1, ROM_HIGH0/1, BANK |
| Atari 800 XL/XE | PORTB `$D301` | OSROM, BASIC, BANK0/1, CPUBANK, ANTICBANK, SELFTEST |

Die Bitfeld-Schalter oben setzen den Registerwert; passt er zu einem Preset (zum Beispiel „Standard (BASIC, E/A, KERNAL)“ beim C64 mit `$37` oder „Alles RAM“ mit `$34`), erscheint dessen Name, sonst „kein Preset“. Gibt es für den Wert keine hinterlegte Belegung, zeigt die Matrix das Standard-Preset und sagt das dazu. Ein Preset kann mit dem Hinweis „nur 130XE“ auf ein Modell beschränkt sein. Hat ein Ziel keine Presets (VC-20, C16/Plus/4, MEGA65), zeigt die Matrix nur die Segmente des Programms.

## Liste
Die Liste führt die **Symbole der Systembibliothek** (`data/sys`) des Ziels auf, nach Abschnitten gruppiert. Das Feld filtert nach Name, Adresse oder Text; ein Klick auf einen Eintrag öffnet ihn im Tab Referenz.
