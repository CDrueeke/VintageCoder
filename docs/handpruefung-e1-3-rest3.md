# Handprüfung E1.3 Rest 3 (Prozessbaum beenden)

Ursache aus der Handprüfung von Rest 2: Unter macOS ist `x64sc` eine Kette von Wrappern (`x64sc` → `VICE.sh` → eigentliches VICE). Beenden traf nur den ersten Prozess. Jetzt wird der ganze Baum beendet (erst SIGTERM, nach 1,5 s SIGKILL); der Emulator läuft in einer eigenen Prozessgruppe; das Ende gilt erst, wenn auch der letzte Nachkomme weg ist.

Voraussetzung: VICE installiert (`/Applications/vice-arm64-gtk3-3.10/bin/x64sc`). Vor jedem Punkt in einem Terminal `pgrep -fl x64sc` bzw. `ps -axo pid,ppid,pgid,command | grep -i vice` — nach dem Beenden darf **kein** VICE-Prozess mehr übrig sein.

| Nr | Schritt | Erwartung | Ergebnis |
|---|---|---|---|
| 1 | F5; im Maschine-Panel „Beenden“ | VICE-Fenster geht zu, `pgrep -fl "VICE|x64sc"` ohne Treffer, Zustand „bereit“, Ausgabe „Emulator beendet.“ | |
| 2 | F5; in der Kopfzeile das Stopp-Symbol „Beenden“ | Wie 1 | |
| 3 | F5; Befehlspalette „Debuggen beenden“ (⇧F5) | Wie 1 | |
| 4 | F5 zweimal kurz hintereinander | Nur **ein** VICE-Fenster, ein Prozessbaum; der erste ist wirklich weg | |
| 5 | F5; in VICE ⌘Q | Innerhalb etwa einer Sekunde „bereit“, Ausgabe „Emulator beendet.“; kein Prozess übrig | |
| 6 | F5; VICE-Fenster mit dem Schließknopf schließen (rot) | Wie 5 | |
| 7 | F5; die IDE beenden (⌘Q) | VICE geht mit der IDE zu (kein verwaister Prozess) | |
| 8 | Falls Atari (atari800/Altirra) oder MEGA65 (Xemu) eingerichtet: F5, dann „Beenden“ | Wie 1 | |
